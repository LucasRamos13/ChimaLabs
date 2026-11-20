//! FFT/IFFT 1-D e N-D de qualquer tamanho, deslocamento de frequência zero e `nextpow2`.

use std::sync::{Mutex, OnceLock};

use num_complex::Complex64;
use rustfft::{FftDirection, FftPlanner};

use super::{SignalError, check_shape, extend_shape, lines};

fn planner() -> &'static Mutex<FftPlanner<f64>> {
    static P: OnceLock<Mutex<FftPlanner<f64>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(FftPlanner::new()))
}

/// Transformada ao longo de `dim`, com preenchimento por zeros ou truncamento para `n`.
fn transform(
    x: &[Complex64],
    shape: &[usize],
    n: Option<usize>,
    dim: usize,
    direction: FftDirection,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    check_shape(x.len(), shape)?;
    let shape = extend_shape(shape, dim);
    let len_in = shape[dim];
    let len = n.unwrap_or(len_in);
    let mut out_shape = shape.clone();
    out_shape[dim] = len;
    let total: usize = out_shape.iter().product();
    let mut out = vec![Complex64::new(0.0, 0.0); total];
    if total == 0 {
        return Ok((out, out_shape));
    }
    let (stride_in, _, starts_in) = lines(&shape, dim);
    let (stride_out, _, starts_out) = lines(&out_shape, dim);
    let fft = planner()
        .lock()
        .expect("planner FFT envenenado")
        .plan_fft(len, direction);
    let mut buf = vec![Complex64::new(0.0, 0.0); len];
    let mut scratch = vec![Complex64::new(0.0, 0.0); fft.get_inplace_scratch_len()];
    for (&si, &so) in starts_in.iter().zip(&starts_out) {
        for (k, b) in buf.iter_mut().enumerate() {
            *b = if k < len_in {
                x[si + k * stride_in]
            } else {
                Complex64::new(0.0, 0.0)
            };
        }
        fft.process_with_scratch(&mut buf, &mut scratch);
        for (k, b) in buf.iter().enumerate() {
            out[so + k * stride_out] = *b;
        }
    }
    Ok((out, out_shape))
}

/// Transformada discreta de Fourier ao longo de `dim` (sem normalização):
/// `X[k] = Σₙ x[n]·e^{−2πi·k·n/N}`. Aceita qualquer `N`, inclusive primos.
///
/// `n` preenche com zeros ou trunca a dimensão transformada. Devolve `(dados, forma)`.
pub fn fft(
    x: &[Complex64],
    shape: &[usize],
    n: Option<usize>,
    dim: usize,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    transform(x, shape, n, dim, FftDirection::Forward)
}

/// [`fft`] de dados reais.
pub fn fft_real(
    x: &[f64],
    shape: &[usize],
    n: Option<usize>,
    dim: usize,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    let xc: Vec<Complex64> = x.iter().map(|&v| Complex64::new(v, 0.0)).collect();
    fft(&xc, shape, n, dim)
}

/// Transformada inversa ao longo de `dim`, normalizada por `1/N`.
pub fn ifft(
    x: &[Complex64],
    shape: &[usize],
    n: Option<usize>,
    dim: usize,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    let (mut y, s) = transform(x, shape, n, dim, FftDirection::Inverse)?;
    let len = s[dim] as f64;
    if len > 0.0 {
        y.iter_mut().for_each(|v| *v /= len);
    }
    Ok((y, s))
}

/// Inversa tratando a entrada como conjugado-simétrica: devolve resultado real.
///
/// Equivale à parte real de [`ifft`], isto é, à inversa da componente hermitiana da entrada.
pub fn ifft_symmetric(
    x: &[Complex64],
    shape: &[usize],
    n: Option<usize>,
    dim: usize,
) -> Result<(Vec<f64>, Vec<usize>), SignalError> {
    let (y, s) = ifft(x, shape, n, dim)?;
    Ok((y.iter().map(|v| v.re).collect(), s))
}

/// FFT ao longo de todas as dimensões (N-D). `sizes`, se dado, ajusta cada dimensão.
pub fn fftn(
    x: &[Complex64],
    shape: &[usize],
    sizes: Option<&[usize]>,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    multi(x, shape, sizes, false)
}

/// IFFT ao longo de todas as dimensões (N-D), normalizada pelo produto dos tamanhos.
pub fn ifftn(
    x: &[Complex64],
    shape: &[usize],
    sizes: Option<&[usize]>,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    multi(x, shape, sizes, true)
}

fn multi(
    x: &[Complex64],
    shape: &[usize],
    sizes: Option<&[usize]>,
    inverse: bool,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    check_shape(x.len(), shape)?;
    let ndims = sizes.map_or(shape.len(), |s| s.len().max(shape.len()));
    let mut data = x.to_vec();
    let mut cur = shape.to_vec();
    for d in 0..ndims {
        let n = sizes.and_then(|s| s.get(d).copied());
        let r = if inverse {
            ifft(&data, &cur, n, d)?
        } else {
            fft(&data, &cur, n, d)?
        };
        data = r.0;
        cur = r.1;
    }
    Ok((data, cur))
}

fn circshift_dim<T: Clone>(x: &[T], shape: &[usize], dim: usize, k: usize) -> Vec<T> {
    let (stride, len, starts) = lines(shape, dim);
    let mut out = x.to_vec();
    if len == 0 {
        return out;
    }
    for s in starts {
        for i in 0..len {
            out[s + ((i + k) % len) * stride] = x[s + i * stride].clone();
        }
    }
    out
}

fn shift_all<T: Clone>(
    x: &[T],
    shape: &[usize],
    dims: Option<&[usize]>,
    inverse: bool,
) -> Result<Vec<T>, SignalError> {
    check_shape(x.len(), shape)?;
    let all: Vec<usize> = (0..shape.len()).collect();
    let dims = dims.unwrap_or(&all);
    let mut out = x.to_vec();
    for &d in dims {
        if d >= shape.len() {
            continue; // dimensão singleton implícita: nada a deslocar
        }
        let len = shape[d];
        let k = if inverse { len.div_ceil(2) } else { len / 2 };
        out = circshift_dim(&out, shape, d, k);
    }
    Ok(out)
}

/// Move a componente de frequência zero para o centro (ao longo de `dims`, ou de todas).
pub fn fftshift<T: Clone>(x: &[T], shape: &[usize], dims: Option<&[usize]>) -> Result<Vec<T>, SignalError> {
    shift_all(x, shape, dims, false)
}

/// Inversa de [`fftshift`].
pub fn ifftshift<T: Clone>(x: &[T], shape: &[usize], dims: Option<&[usize]>) -> Result<Vec<T>, SignalError> {
    shift_all(x, shape, dims, true)
}

/// Menor expoente `p` inteiro com `2^p ≥ |x|` (0 para x = 0; NaN/Inf propagados).
pub fn nextpow2(x: f64) -> f64 {
    let a = x.abs();
    if a == 0.0 {
        return 0.0;
    }
    if !a.is_finite() {
        return a;
    }
    let mut p = a.log2().ceil();
    // Corrige arredondamentos do log2 nas potências exatas e vizinhanças.
    if 2f64.powf(p - 1.0) >= a {
        p -= 1.0;
    }
    if 2f64.powf(p) < a {
        p += 1.0;
    }
    p
}

/// FFT 2-D: transforma apenas as dimensões 0 e 1 (em arrays com mais dimensões, cada "página"
/// é transformada separadamente). `size` ajusta `(linhas, colunas)`.
pub fn fft2(
    x: &[Complex64],
    shape: &[usize],
    size: Option<(usize, usize)>,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    let (a, s) = fft(x, shape, size.map(|s| s.0), 0)?;
    fft(&a, &s, size.map(|s| s.1), 1)
}

/// IFFT 2-D (dimensões 0 e 1), normalizada por `linhas·colunas`.
pub fn ifft2(
    x: &[Complex64],
    shape: &[usize],
    size: Option<(usize, usize)>,
) -> Result<(Vec<Complex64>, Vec<usize>), SignalError> {
    let (a, s) = ifft(x, shape, size.map(|s| s.0), 0)?;
    ifft(&a, &s, size.map(|s| s.1), 1)
}
