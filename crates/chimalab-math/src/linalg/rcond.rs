//! Estimativa do recíproco do número de condição na norma 1.
//!
//! Usa o estimador iterativo de norma 1 de Hager (1984), com o refinamento de Higham (1988):
//! estima ‖B‖₁ de um operador B dado só por suas aplicações a vetores (B·x e Bᴴ·x). Para
//! ‖A⁻¹‖₁, as aplicações são resoluções com A e Aᴴ, sem formar a inversa. Para matrizes
//! complexas, o "sinal" de cada componente é `y/|y|`.

use super::{LinScalar, LinalgError, NdArray, solve};

/// Norma 1 de uma matriz (maior soma de módulos de coluna).
pub(crate) fn norm1<T: LinScalar>(a: &NdArray<T>) -> f64 {
    a.norm1()
}

/// Estima ‖B‖₁ dadas funções que aplicam `B` e `Bᴴ` a um vetor, no lugar (para ‖A⁻¹‖₁: resolver
/// `A x = b` e `Aᴴ x = b`). A estimativa nunca excede o valor verdadeiro.
pub(crate) fn norm1_estimate<T: LinScalar>(
    n: usize,
    solve_a: &dyn Fn(&mut [T]),
    solve_ah: &dyn Fn(&mut [T]),
) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let mut x = vec![T::from_f64(1.0 / n as f64); n];
    let mut est = 0.0;
    let mut last_j = usize::MAX;
    for iter in 0..5 {
        let mut y = x.clone();
        solve_a(&mut y);
        let est_new: f64 = y.iter().map(|v| v.abs()).sum();
        if !est_new.is_finite() {
            return f64::INFINITY;
        }
        if iter > 0 && est_new <= est {
            break;
        }
        est = est_new;
        let mut z: Vec<T> = y.iter().map(|&v| v.signum()).collect();
        solve_ah(&mut z);
        let (j, zmax) =
            z.iter().enumerate().fold(
                (0, 0.0_f64),
                |acc, (i, v)| if v.abs() > acc.1 { (i, v.abs()) } else { acc },
            );
        let ztx: f64 = z.iter().zip(&x).map(|(a, b)| (a.conj() * *b).re()).sum();
        if iter > 0 && (zmax <= ztx || j == last_j) {
            break;
        }
        last_j = j;
        x = vec![T::zero(); n];
        x[j] = T::one();
    }
    // Vetor alternativo de Higham, que protege contra casos em que a iteração subestima.
    let mut alt: Vec<T> = (0..n)
        .map(|i| {
            // (−1)ⁱ·(1 + i/(n − 1)); com n = 1, só x₀ = 1.
            let sign = 1.0 - 2.0 * (i % 2) as f64;
            T::from_f64(sign * (1.0 + i as f64 / (n - 1).max(1) as f64))
        })
        .collect();
    solve_a(&mut alt);
    let est_alt = 2.0 * alt.iter().map(|v| v.abs()).sum::<f64>() / (3.0 * n as f64);
    est.max(est_alt)
}

/// Estimativa do recíproco do número de condição de `a` na norma 1.
///
/// Devolve um valor em `[0, 1]`: próximo de 1 para matrizes bem condicionadas e próximo de 0
/// (ou exatamente 0) para matrizes singulares. Matriz vazia devolve `Inf`.
pub fn rcond<T: LinScalar>(a: &NdArray<T>) -> Result<f64, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "rcond",
            shape: a.shape(),
        });
    }
    if a.is_empty() {
        return Ok(f64::INFINITY);
    }
    if !a.all_finite() {
        return Err(LinalgError::NonFinite { operation: "rcond" });
    }
    Ok(solve::lu_factor(a).rcond)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::MatrixExt;
    use num_complex::Complex64;

    /// Estimativa de ‖B‖₁ com B e Bᴴ aplicadas por produto explícito.
    fn est<T: LinScalar>(b: &NdArray<T>) -> f64 {
        let n = b.rows();
        let bh = NdArray::from_fn(n, n, |i, j| b.get(j, i).conj());
        let mul = |m: &NdArray<T>, x: &mut [T]| {
            let y = m.matmul(&NdArray::from_fn(n, 1, |i, _| x[i])).unwrap();
            x.iter_mut().enumerate().for_each(|(i, xi)| *xi = y.get(i, 0));
        };
        norm1_estimate(n, &|x| mul(b, x), &|x| mul(&bh, x))
    }

    fn m(rows: &[&[f64]]) -> NdArray<f64> {
        NdArray::from_rows(rows).unwrap()
    }

    #[test]
    fn exata_em_matrizes_nao_negativas_e_limitada_em_geral() {
        // B ≥ 0: x = 1/n já dá a média das colunas, e o passo seguinte escolhe a coluna de maior
        // soma: estimativa exata. Em geral, ‖Bx‖₁ com ‖x‖₁ = 1 nunca excede ‖B‖₁.
        let mut seed = 11u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64 - 0.5
        };
        for n in 1..9 {
            let b = NdArray::<f64>::from_fn(n, n, |_, _| next());
            let pos = b.map(|x| x.abs());
            assert_eq!(est(&pos), pos.norm1(), "n={n}");
            assert!(est(&b) <= b.norm1() * (1.0 + 1e-15), "n={n}");
            assert!(est(&b) >= b.norm1() / 3.0, "n={n}");
            let z = NdArray::<Complex64>::from_fn(n, n, |_, _| Complex64::new(next(), next()));
            assert!(est(&z) <= z.norm1() * (1.0 + 1e-15), "n={n}");
        }
        assert_eq!(norm1_estimate::<f64>(0, &|_| {}, &|_| {}), 0.0);
        assert_eq!(est(&m(&[&[-5.0]])), 5.0);
    }

    #[test]
    fn partida_com_produto_nulo() {
        // n = 4 (1/n exato). B = 1·vᵀ com v = (1, 1, −1, −1) ⊥ 1: B·(1/n) = 0. A primeira
        // iteração dá 0, mas não pode parar ali: z = Bᴴ·1 = 4v aponta a coluna 0, e
        // ‖B·e₀‖₁ = 4 = ‖B‖₁. (v também é ⊥ alt = (1, −4/3, 5/3, −2), então o vetor
        // alternativo não socorre.)
        let v = [1.0, 1.0, -1.0, -1.0];
        let b = NdArray::<f64>::from_fn(4, 4, |_, j| v[j]);
        assert_eq!(est(&b), 4.0);
    }

    /// Referência escrita à parte, direto da descrição publicada (Hager, 1984; Higham, 1988):
    /// iteração x ← e_j com j = argmax|Bᵀ·sinal(Bx)| (primeiro índice), parando quando a
    /// estimativa não cresce, quando max|z| ≤ zᵀx ou quando j se repete (no máximo 5 passos),
    /// e o vetor alternativo xᵢ = (−1)ⁱ(1 + i/(n−1)) com peso 2/(3n).
    fn reference(b: &[Vec<f64>]) -> f64 {
        let n = b.len();
        let mul = |x: &[f64]| -> Vec<f64> {
            b.iter()
                .map(|row| row.iter().zip(x).map(|(a, c)| a * c).sum())
                .collect()
        };
        let mul_t =
            |x: &[f64]| -> Vec<f64> { (0..n).map(|j| (0..n).map(|i| b[i][j] * x[i]).sum()).collect() };
        let l1 = |v: &[f64]| v.iter().map(|t| t.abs()).sum::<f64>();
        let mut x = vec![1.0 / n as f64; n];
        let mut best = 0.0;
        let mut previous: Option<usize> = None;
        let mut step = 0;
        loop {
            let y = mul(&x);
            if step > 0 && l1(&y) <= best {
                break;
            }
            best = l1(&y);
            let sign: Vec<f64> = y.iter().map(|&t| if t < 0.0 { -1.0 } else { 1.0 }).collect();
            let z = mul_t(&sign);
            let mut j = 0;
            for i in 1..n {
                if z[i].abs() > z[j].abs() {
                    j = i;
                }
            }
            let ztx: f64 = z.iter().zip(&x).map(|(a, c)| a * c).sum();
            if step > 0 && (z[j].abs() <= ztx || previous == Some(j)) {
                break;
            }
            step += 1;
            if step == 5 {
                break;
            }
            previous = Some(j);
            x = vec![0.0; n];
            x[j] = 1.0;
        }
        let alt: Vec<f64> = (0..n)
            .map(|i| {
                let mag = if n == 1 {
                    1.0
                } else {
                    1.0 + i as f64 / (n - 1) as f64
                };
                if i % 2 == 1 { -mag } else { mag }
            })
            .collect();
        let alt_est = 2.0 * l1(&mul(&alt)) / (3.0 * n as f64);
        if alt_est > best { alt_est } else { best }
    }

    #[test]
    fn concorda_com_referencia_independente() {
        // Matrizes de inteiros pequenos (muitas com empates e zeros), comparadas com a referência
        // acima. n ∈ {1, 2, 4, 8}: 1/n é exato, e as somas também; assim, empates exatos entre
        // estimativas não são desfeitos de modo diferente pela ordem das somas.
        let mut seed = 5u64;
        for k in 0..4000 {
            let n = [1, 2, 4, 8][k % 4];
            let rows: Vec<Vec<f64>> = (0..n)
                .map(|_| {
                    (0..n)
                        .map(|_| {
                            seed = seed
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(1442695040888963407);
                            ((seed >> 33) % 7) as f64 - 3.0
                        })
                        .collect()
                })
                .collect();
            let b = NdArray::<f64>::from_fn(n, n, |i, j| rows[i][j]);
            let (got, want) = (est(&b), reference(&rows));
            assert!(
                (got - want).abs() <= 1e-12 * want.max(1.0),
                "k={k}: {rows:?}: {got} × {want}"
            );
        }
    }

    #[test]
    fn empate_no_maximo_fica_com_o_primeiro_indice() {
        // B = [[1, 3], [1, −1]]: y = B·(½, ½) = (2, 0), z = Bᴴ·(1, 1) = (2, 2) empata, e j = 0.
        // B·e₀ = (1, 1) não melhora (2), a iteração para. O vetor alternativo (1, −2) dá
        // B·alt = (−5, 3) e 2·8/(3·2) = 8/3. ‖B‖₁ = 4: subestimativa conhecida do método.
        assert_eq!(est(&m(&[&[1.0, 3.0], &[1.0, -1.0]])), 8.0 / 3.0);
    }

    #[test]
    fn vetor_alternativo_de_higham() {
        // alt = (1; −1,5; 2) para n = 3. B·alt = (1; 10,5; 8), soma 19,5, e 2·19,5/(3·3) = 13/3,
        // maior que o resultado da iteração. ‖B‖₁ = 7.
        let b = m(&[&[0.0, 2.0, 2.0], &[2.0, -3.0, 2.0], &[-1.0, -2.0, 3.0]]);
        assert_eq!(est(&b), 13.0 / 3.0);
    }
}
