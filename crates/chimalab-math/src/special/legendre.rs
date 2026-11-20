//! Funções de Legendre associadas P_n^m(x), m = 0..n, |x| ≤ 1 (DLMF 14.3.1, com a fase de
//! Condon-Shortley (−1)^m). Calculadas pela recorrência estável das funções totalmente
//! normalizadas (Holmes e Featherstone, J. Geodesy 76, 2002) e convertidas para a normalização
//! pedida.

use super::{SpecialError, gammaln};

/// Normalização das funções de Legendre associadas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegendreNorm {
    /// Sem normalização: P_n^m (com fase de Condon-Shortley).
    Unnormalized,
    /// Semi-normalizada de Schmidt (sem a fase): S_n^0 = P_n^0, S_n^m = √(2(n−m)!/(n+m)!)·|P_n^m|·sinal sem fase.
    Schmidt,
    /// Totalmente normalizada: N_n^m = √((n + ½)(n−m)!/(n+m)!)·P_n^m (∫₋₁¹ (N_n^m)² dx = 1).
    Full,
}

/// Valores de P_n^m(x) para m = 0..n (vetor de comprimento n + 1).
pub fn legendre(n: usize, x: f64, norm: LegendreNorm) -> Result<Vec<f64>, SpecialError> {
    if !(-1.0..=1.0).contains(&x) {
        return Err(SpecialError::Domain("legendre exige −1 ≤ x ≤ 1"));
    }
    let u = ((1.0 - x) * (1.0 + x)).sqrt();
    if norm == LegendreNorm::Unnormalized && n <= 150 {
        return Ok(unnormalized_direct(n, x, u));
    }
    // N_m^m e recorrência em grau para cada m, só até o grau n.
    let mut out = vec![0.0; n + 1];
    let mut nmm = (0.5f64).sqrt(); // N_0^0
    #[allow(clippy::needless_range_loop)] // m é a ordem da função, não só um índice
    for m in 0..=n {
        if m > 0 {
            let mm = m as f64;
            nmm *= -((2.0 * mm + 1.0) / (2.0 * mm)).sqrt() * u;
        }
        // Sobe em grau de l = m até n
        let mut prev = nmm;
        if n > m {
            let mm = m as f64;
            let mut cur = (2.0 * mm + 3.0).sqrt() * x * nmm;
            let mut prev2 = prev;
            prev = cur;
            for l in (m + 2)..=n {
                let (ll, mmf) = (l as f64, m as f64);
                let a = ((4.0 * ll * ll - 1.0) / (ll * ll - mmf * mmf)).sqrt();
                let b = (((ll - 1.0).powi(2) - mmf * mmf) * (2.0 * ll + 1.0)
                    / ((ll * ll - mmf * mmf) * (2.0 * ll - 3.0)))
                    .sqrt();
                cur = a * x * prev - b * prev2;
                prev2 = prev;
                prev = cur;
            }
        }
        let full = prev; // N_n^m
        let nf = n as f64;
        out[m] = match norm {
            LegendreNorm::Full => full,
            LegendreNorm::Unnormalized => {
                // P = N / √((n+½)(n−m)!/(n+m)!)
                let lf = 0.5 * (gammaln(nf + m as f64 + 1.0) - gammaln(nf - m as f64 + 1.0));
                full * (lf - 0.5 * (nf + 0.5).ln()).exp()
            }
            LegendreNorm::Schmidt => {
                let s = if m == 0 {
                    full / (nf + 0.5).sqrt()
                } else {
                    full * 2.0 / (2.0 * nf + 1.0).sqrt()
                };
                if m % 2 == 1 { -s } else { s } // remove a fase de Condon-Shortley
            }
        };
    }
    Ok(out)
}

/// Recorrência direta das funções não normalizadas (DLMF 14.10.1 e 14.7.15), sem risco de
/// overflow para n ≤ 150; exata em x = ±1.
fn unnormalized_direct(n: usize, x: f64, u: f64) -> Vec<f64> {
    let mut out = vec![0.0; n + 1];
    let mut pmm = 1.0; // P_m^m = (−1)^m (2m−1)!! u^m
    #[allow(clippy::needless_range_loop)] // m é a ordem da função, não só um índice
    for m in 0..=n {
        if m > 0 {
            pmm *= -((2 * m - 1) as f64) * u;
        }
        if m == n {
            out[m] = pmm;
            continue;
        }
        let mut p0 = pmm;
        let mut p1 = x * (2 * m + 1) as f64 * pmm;
        for l in (m + 2)..=n {
            let p2 = ((2 * l - 1) as f64 * x * p1 - (l + m - 1) as f64 * p0) / (l - m) as f64;
            p0 = p1;
            p1 = p2;
        }
        out[m] = p1;
    }
    out
}
