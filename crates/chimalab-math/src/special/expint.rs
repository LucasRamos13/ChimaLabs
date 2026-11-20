//! Integral exponencial E₁ (DLMF 6.2.1) e Ei (DLMF 6.2.5).
//!
//! - E₁(x), 0 < x ≤ 1: série (DLMF 6.6.2); x > 1: fração contínua (DLMF 6.9.1).
//! - Ei(x), x > 0: série convergente de termos positivos (DLMF 6.6.1) até x = 40; acima,
//!   expansão assintótica truncada no menor termo (DLMF 6.12.2).

use num_complex::Complex64;
use std::f64::consts::PI;

use super::lentz;

const EULER_GAMMA: f64 = std::f64::consts::EULER_GAMMA;

fn e1_positive(x: f64) -> f64 {
    if x <= 1.0 {
        let mut term = 1.0;
        let mut sum = 0.0;
        for k in 1..500 {
            let k = k as f64;
            term *= -x / k;
            let t = -term / k;
            sum += t;
            if t.abs() <= 1e-17 * sum.abs() {
                break;
            }
        }
        return -EULER_GAMMA - x.ln() + sum;
    }
    if x > 745.0 {
        return 0.0;
    }
    // E₁(x) = e^{−x}·1/(x+1 − 1/(x+3 − 4/(x+5 − 9/(x+7 − …))))
    let cf = lentz(0.0, |k| {
        let kk = k as f64;
        let ak = if k == 1 { 1.0 } else { -(kk - 1.0) * (kk - 1.0) };
        (ak, x + 2.0 * kk - 1.0)
    });
    (-x).exp() * cf
}

/// Integral exponencial Ei(x) (valor principal), x real ≠ 0.
pub fn ei(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x < 0.0 {
        return -e1_positive(-x);
    }
    if x > 709.8 {
        return f64::INFINITY;
    }
    if x <= 40.0 {
        let mut term = 1.0;
        let mut sum = 0.0;
        for k in 1..5000 {
            let k = k as f64;
            term *= x / k;
            let t = term / k;
            sum += t;
            if t <= 1e-17 * sum {
                break;
            }
        }
        return EULER_GAMMA + x.ln() + sum;
    }
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..1000 {
        let next = term * k as f64 / x;
        if next >= term || next <= 1e-17 * sum {
            break;
        }
        term = next;
        sum += term;
    }
    x.exp() / x * sum
}

/// E₁(x) real para x ≥ 0 (E₁(0) = +Inf). Para x < 0 o valor é complexo: devolve NaN; use
/// [`expint_complex`].
pub fn expint(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::INFINITY;
    }
    e1_positive(x)
}

/// E₁(x) para x real qualquer: para x < 0, `E₁(x) = −Ei(−x) − iπ` (ramo principal).
pub fn expint_complex(x: f64) -> Complex64 {
    if x < 0.0 {
        Complex64::new(-ei(-x), -PI)
    } else {
        Complex64::new(expint(x), 0.0)
    }
}
