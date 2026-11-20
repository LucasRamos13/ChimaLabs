//! Regra trapezoidal sobre dados amostrados, ao longo de uma dimensão de um array N-D
//! column-major `(dados, forma)`. Dimensões além das existentes são singleton.

use chimalab_array::Scalar;

use super::IntegrateError;

/// Abscissas ao longo da dimensão integrada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spacing<'a> {
    /// Espaçamento 1.
    Unit,
    /// Espaçamento constante.
    Uniform(f64),
    /// Abscissas explícitas (comprimento = tamanho da dimensão; não precisam ser monotônicas).
    Points(&'a [f64]),
}

impl Spacing<'_> {
    /// Largura do k-ésimo trecho (entre as amostras k e k + 1).
    fn width(&self, k: usize) -> f64 {
        match self {
            Spacing::Unit => 1.0,
            Spacing::Uniform(h) => *h,
            Spacing::Points(x) => x[k + 1] - x[k],
        }
    }
}

/// Geometria comum: (forma estendida, passo da dimensão, tamanho da dimensão).
fn layout(
    len: usize,
    shape: &[usize],
    spacing: &Spacing,
    dim: usize,
) -> Result<(Vec<usize>, usize, usize), IntegrateError> {
    let expected: usize = shape.iter().product();
    if expected != len {
        return Err(IntegrateError::ShapeMismatch {
            expected,
            actual: len,
        });
    }
    let mut full = shape.to_vec();
    if full.len() <= dim {
        full.resize(dim + 1, 1);
    }
    let n = full[dim];
    if let Spacing::Points(x) = spacing
        && x.len() != n
    {
        return Err(IntegrateError::ShapeMismatch {
            expected: n,
            actual: x.len(),
        });
    }
    let stride: usize = full[..dim].iter().product();
    Ok((full, stride, n))
}

/// ∫ y ao longo de `dim` pela regra trapezoidal: Σ (x_{k+1} − x_k)(y_k + y_{k+1})/2. Devolve
/// `(dados, forma)` com a dimensão `dim` reduzida a 1; com 0 ou 1 amostra o resultado é 0.
pub fn trapz<T: Scalar>(
    y: &[T],
    shape: &[usize],
    spacing: Spacing,
    dim: usize,
) -> Result<(Vec<T>, Vec<usize>), IntegrateError> {
    let (mut out_shape, stride, n) = layout(y.len(), shape, &spacing, dim)?;
    let outer: usize = out_shape[dim + 1..].iter().product();
    out_shape[dim] = 1;
    // Com n = 0 a saída tem as demais dimensões e vale zero (soma vazia).
    let mut out = vec![T::zero(); out_shape.iter().product()];
    for o in 0..outer {
        for i in 0..stride {
            let base = o * stride * n + i;
            let mut s = T::zero();
            for k in 0..n.saturating_sub(1) {
                s += (y[base + k * stride] + y[base + (k + 1) * stride]).scale(0.5 * spacing.width(k));
            }
            out[o * stride + i] = s;
        }
    }
    Ok((out, out_shape))
}

/// Integral acumulada pela regra trapezoidal ao longo de `dim`: mesma forma que `y`, com zero na
/// primeira amostra.
pub fn cumtrapz<T: Scalar>(
    y: &[T],
    shape: &[usize],
    spacing: Spacing,
    dim: usize,
) -> Result<(Vec<T>, Vec<usize>), IntegrateError> {
    let (full, stride, n) = layout(y.len(), shape, &spacing, dim)?;
    let mut out = vec![T::zero(); y.len()];
    let outer: usize = full[dim + 1..].iter().product();
    for o in 0..outer {
        for i in 0..stride {
            let base = o * stride * n + i;
            let mut s = T::zero();
            for k in 1..n {
                s += (y[base + (k - 1) * stride] + y[base + k * stride]).scale(0.5 * spacing.width(k - 1));
                out[base + k * stride] = s;
            }
        }
    }
    Ok((out, shape.to_vec()))
}
