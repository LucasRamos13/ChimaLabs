//! Filtro digital racional (IIR/FIR) por equação de diferenças, forma direta II transposta.

use chimalab_array::Scalar;

use super::{SignalError, check_shape, extend_shape, lines};

/// Saída de [`filter`]: sinal filtrado e estado final (para continuar a filtragem).
#[derive(Clone, Debug, PartialEq)]
pub struct FilterResult<T: Scalar> {
    /// Sinal filtrado.
    pub y: Vec<T>,
    /// Estado final, `max(len(a), len(b)) − 1` valores por linha filtrada.
    pub zf: Vec<T>,
}

/// Filtra o vetor `x` com a função de transferência `B(z)/A(z)`:
/// `a₀·y[n] = Σₖ bₖ·x[n−k] − Σₖ₌₁ aₖ·y[n−k]`.
///
/// `zi` é o estado inicial (zeros se omitido). Coeficientes são normalizados por `a₀`.
pub fn filter<T: Scalar>(
    b: &[T],
    a: &[T],
    x: &[T],
    zi: Option<&[T]>,
) -> Result<FilterResult<T>, SignalError> {
    let r = filter_along(b, a, x, &[x.len()], 0, zi)?;
    Ok(r)
}

/// Filtra cada linha de um array N-D ao longo de `dim`. `zi`, se dado, tem `ordem` valores
/// por linha, na mesma ordem das linhas (column-major sobre as demais dimensões).
pub fn filter_along<T: Scalar>(
    b: &[T],
    a: &[T],
    x: &[T],
    shape: &[usize],
    dim: usize,
    zi: Option<&[T]>,
) -> Result<FilterResult<T>, SignalError> {
    check_shape(x.len(), shape)?;
    if b.is_empty() || a.is_empty() {
        return Err(SignalError::InvalidArgument(
            "os coeficientes b e a não podem ser vazios",
        ));
    }
    if a[0] == T::zero() {
        return Err(SignalError::InvalidArgument(
            "o primeiro coeficiente de a não pode ser zero",
        ));
    }
    let shape = extend_shape(shape, dim);
    let order = a.len().max(b.len()) - 1;
    let a0 = a[0];
    let bn: Vec<T> = (0..=order)
        .map(|k| b.get(k).copied().unwrap_or(T::zero()) / a0)
        .collect();
    let an: Vec<T> = (0..=order)
        .map(|k| a.get(k).copied().unwrap_or(T::zero()) / a0)
        .collect();
    let (stride, len, starts) = lines(&shape, dim);
    if let Some(z) = zi
        && z.len() != order * starts.len()
    {
        return Err(SignalError::InvalidArgument(
            "zi deve ter max(len(a), len(b)) − 1 valores por linha",
        ));
    }
    let mut y = vec![T::zero(); x.len()];
    let mut zf = Vec::with_capacity(order * starts.len());
    for (line, &s) in starts.iter().enumerate() {
        let mut z: Vec<T> = match zi {
            Some(v) => v[line * order..(line + 1) * order].to_vec(),
            None => vec![T::zero(); order],
        };
        for k in 0..len {
            let xn = x[s + k * stride];
            let yn = bn[0] * xn + z.first().copied().unwrap_or(T::zero());
            for i in 0..order {
                let next = if i + 1 < order { z[i + 1] } else { T::zero() };
                z[i] = bn[i + 1] * xn + next - an[i + 1] * yn;
            }
            y[s + k * stride] = yn;
        }
        zf.extend_from_slice(&z);
    }
    Ok(FilterResult { y, zf })
}
