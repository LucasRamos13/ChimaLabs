//! Convolução 1-D/2-D/N-D, correlação 2-D e divisão polinomial (deconvolução).

use num_complex::Complex64;

use super::fourier::{fftn, ifftn};
use super::{SignalError, check_shape};
use chimalab_array::{NdArray, Scalar};

/// Parte da convolução devolvida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvShape {
    /// Convolução completa: tamanho `na + nb − 1` em cada dimensão.
    Full,
    /// Parte central com o tamanho do primeiro argumento. Para núcleo de tamanho par, o centro
    /// é deslocado: a saída começa no índice `⌊nb/2⌋` da convolução completa.
    Same,
    /// Só as posições calculadas sem preenchimento: `max(na − nb + 1, 0)`.
    Valid,
}

/// Produto `na·nb` acima do qual (e com ambos os lados ≥ 32) a convolução usa FFT.
const FFT_THRESHOLD: usize = 1 << 16;

/// Convolução N-D de arrays reais column-major. Formas são completadas com 1 até o mesmo
/// número de dimensões. Devolve `(dados, forma)`.
pub fn convn<T: Scalar>(
    a: &[T],
    shape_a: &[usize],
    b: &[T],
    shape_b: &[usize],
    shape: ConvShape,
) -> Result<(Vec<T>, Vec<usize>), SignalError> {
    check_shape(a.len(), shape_a)?;
    check_shape(b.len(), shape_b)?;
    let nd = shape_a.len().max(shape_b.len()).max(1);
    let pad = |s: &[usize]| -> Vec<usize> { (0..nd).map(|d| s.get(d).copied().unwrap_or(1)).collect() };
    let (sa, sb) = (pad(shape_a), pad(shape_b));
    let full_shape: Vec<usize> = sa
        .iter()
        .zip(&sb)
        .map(|(&x, &y)| if x == 0 || y == 0 { 0 } else { x + y - 1 })
        .collect();
    let full = if a.len() >= 32 && b.len() >= 32 && a.len().saturating_mul(b.len()) > FFT_THRESHOLD {
        conv_fft(a, &sa, b, &sb, &full_shape)?
    } else {
        conv_direct(a, &sa, b, &sb, &full_shape)
    };
    Ok(crop(&full, &full_shape, &sa, &sb, shape))
}

fn strides(shape: &[usize]) -> Vec<usize> {
    let mut s = vec![1; shape.len()];
    for d in 1..shape.len() {
        s[d] = s[d - 1] * shape[d - 1];
    }
    s
}

fn unravel(mut i: usize, shape: &[usize], out: &mut [usize]) {
    for (d, &n) in shape.iter().enumerate() {
        out[d] = i % n;
        i /= n;
    }
}

fn conv_direct<T: Scalar>(a: &[T], sa: &[usize], b: &[T], sb: &[usize], full_shape: &[usize]) -> Vec<T> {
    let total: usize = full_shape.iter().product();
    let mut out = vec![T::zero(); total];
    if total == 0 {
        return out;
    }
    let fs = strides(full_shape);
    let mut ia = vec![0; sa.len()];
    let mut ib = vec![0; sb.len()];
    // Deslocamento linear, na saída, de cada elemento de b.
    let b_off: Vec<usize> = (0..b.len())
        .map(|j| {
            unravel(j, sb, &mut ib);
            ib.iter().zip(&fs).map(|(x, s)| x * s).sum()
        })
        .collect();
    for (i, &av) in a.iter().enumerate() {
        if av == T::zero() {
            continue;
        }
        unravel(i, sa, &mut ia);
        let base: usize = ia.iter().zip(&fs).map(|(x, s)| x * s).sum();
        for (bv, off) in b.iter().zip(&b_off) {
            out[base + off] += av * *bv;
        }
    }
    out
}

fn embed<T: Scalar>(x: &[T], sx: &[usize], target: &[usize]) -> Vec<Complex64> {
    let total: usize = target.iter().product();
    let ts = strides(target);
    let mut out = vec![Complex64::new(0.0, 0.0); total];
    let mut idx = vec![0; sx.len()];
    for (i, &v) in x.iter().enumerate() {
        unravel(i, sx, &mut idx);
        let p: usize = idx.iter().zip(&ts).map(|(a, s)| a * s).sum();
        out[p] = v.to_complex();
    }
    out
}

fn conv_fft<T: Scalar>(
    a: &[T],
    sa: &[usize],
    b: &[T],
    sb: &[usize],
    full_shape: &[usize],
) -> Result<Vec<T>, SignalError> {
    let fa = fftn(&embed(a, sa, full_shape), full_shape, None)?.0;
    let fb = fftn(&embed(b, sb, full_shape), full_shape, None)?.0;
    let prod: Vec<Complex64> = fa.iter().zip(&fb).map(|(x, y)| x * y).collect();
    Ok(ifftn(&prod, full_shape, None)?
        .0
        .iter()
        .map(|&z| from_complex::<T>(z))
        .collect())
}

fn crop<T: Scalar>(
    full: &[T],
    full_shape: &[usize],
    sa: &[usize],
    sb: &[usize],
    shape: ConvShape,
) -> (Vec<T>, Vec<usize>) {
    let (start, out_shape): (Vec<usize>, Vec<usize>) = match shape {
        ConvShape::Full => return (full.to_vec(), full_shape.to_vec()),
        ConvShape::Same => (sb.iter().map(|&n| n / 2).collect(), sa.to_vec()),
        ConvShape::Valid => (
            sb.iter().map(|&n| n.saturating_sub(1)).collect(),
            sa.iter()
                .zip(sb)
                .map(|(&x, &y)| if y == 0 { 0 } else { (x + 1).saturating_sub(y) })
                .collect(),
        ),
    };
    let total: usize = out_shape.iter().product();
    let fs = strides(full_shape);
    let mut idx = vec![0; out_shape.len()];
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        unravel(i, &out_shape, &mut idx);
        let p: usize = idx
            .iter()
            .zip(&start)
            .zip(&fs)
            .map(|((x, st), s)| (x + st) * s)
            .sum();
        out.push(if full.is_empty() { T::zero() } else { full[p] });
    }
    (out, out_shape)
}

/// Convolução de vetores (também multiplicação de polinômios por coeficientes).
pub fn conv<T: Scalar>(u: &[T], v: &[T], shape: ConvShape) -> Vec<T> {
    convn(u, &[u.len()], v, &[v.len()], shape)
        .expect("formas 1-D consistentes por construção")
        .0
}

/// Convolução 2-D de matrizes.
pub fn conv2<T: Scalar>(a: &NdArray<T>, b: &NdArray<T>, shape: ConvShape) -> NdArray<T> {
    let (d, s) = convn(
        a.as_slice(),
        &[a.rows(), a.cols()],
        b.as_slice(),
        &[b.rows(), b.cols()],
        shape,
    )
    .expect("formas 2-D consistentes por construção");
    NdArray::from_col_major(s[0], s[1], d).expect("tamanho consistente")
}

/// Convolução 2-D separável: convolui as colunas de `a` com o vetor `u` e, em seguida, as
/// linhas do resultado com o vetor `v`. Equivale a `conv2(a, u·vᵀ, shape)` (o núcleo é o produto
/// externo), com custo O(m·n·(nu + nv)) em vez de O(m·n·nu·nv). Em `Same`, a saída tem a forma
/// de `a`.
pub fn conv2_separable<T: Scalar>(u: &[T], v: &[T], a: &NdArray<T>, shape: ConvShape) -> NdArray<T> {
    let (d1, s1) = convn(a.as_slice(), &[a.rows(), a.cols()], u, &[u.len(), 1], shape)
        .expect("formas 2-D consistentes por construção");
    let (d2, s2) = convn(&d1, &s1, v, &[1, v.len()], shape).expect("formas 2-D consistentes por construção");
    NdArray::from_col_major(s2[0], s2[1], d2).expect("tamanho consistente")
}

/// Correlação 2-D: filtra `x` com o núcleo `h` sem rotacioná-lo
/// (equivale a `conv2(x, rot180(h), shape)`).
pub fn filter2<T: Scalar>(h: &NdArray<T>, x: &NdArray<T>, shape: ConvShape) -> NdArray<T> {
    let (m, n) = h.shape();
    let rot = NdArray::from_fn(m, n, |i, j| h.get(m - 1 - i, n - 1 - j));
    conv2(x, &rot, shape)
}

/// Deconvolução (divisão polinomial): devolve `(q, r)` com `b = conv(a, q) + r`, `len(r) = len(b)`.
pub fn deconv<T: Scalar>(b: &[T], a: &[T]) -> Result<(Vec<T>, Vec<T>), SignalError> {
    if a.is_empty() || a[0] == T::zero() {
        return Err(SignalError::InvalidArgument(
            "o primeiro coeficiente do divisor não pode ser zero",
        ));
    }
    if b.len() < a.len() {
        return Ok((vec![T::zero()], b.to_vec()));
    }
    let nq = b.len() - a.len() + 1;
    let mut r = b.to_vec();
    let mut q = vec![T::zero(); nq];
    for k in 0..nq {
        let c = r[k] / a[0];
        q[k] = c;
        for (j, &aj) in a.iter().enumerate() {
            r[k + j] -= c * aj;
        }
        r[k] = T::zero(); // zero exato no termo eliminado
    }
    Ok((q, r))
}

/// Converte o resultado complexo da FFT para o tipo de saída (parte real, para tipos reais).
fn from_complex<T: Scalar>(z: Complex64) -> T {
    if T::IS_COMPLEX {
        T::from_complex(z).expect("tipo complexo")
    } else {
        T::from_f64(z.re)
    }
}
