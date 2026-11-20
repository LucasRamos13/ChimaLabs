//! Função erro e relacionadas.
//!
//! - |x| ≤ 1,3: série de Maclaurin de erf (DLMF 7.6.1);
//! - |x| > 1,3: erfc por fração contínua da gama incompleta Q(1/2, x²) (DLMF 7.11.2, 8.9.2);
//! - erfcx = e^{x²}·erfc(x) avaliada sem exponencial para x grande;
//! - inversas por iteração de Halley a partir de aproximações assintóticas.

use super::lentz;

const SQRT_PI: f64 = 1.772_453_850_905_516;

fn erf_series(x: f64) -> f64 {
    if x == 0.0 {
        return x; // preserva o sinal de ±0
    }
    let x2 = x * x;
    let mut term = x;
    let mut sum = x;
    for n in 1..200 {
        let n = n as f64;
        term *= -x2 / n;
        let t = term / (2.0 * n + 1.0);
        sum += t;
        if t.abs() <= 1e-17 * sum.abs() {
            break;
        }
    }
    2.0 / SQRT_PI * sum
}

/// Fração contínua de Q(1/2, X) sem o prefator: Q = e^{−X}·√X/√π · CF.
fn erfc_cf(x: f64) -> f64 {
    let xx = x * x;
    lentz(0.0, |k| {
        let kk = k as f64;
        let ak = if k == 1 { 1.0 } else { -(kk - 1.0) * (kk - 1.5) };
        (ak, xx + 2.0 * kk - 1.5)
    })
}

/// Função erro.
pub fn erf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.abs() <= 1.3 {
        return erf_series(x);
    }
    let c = erfc(x.abs());
    (1.0 - c).copysign(x)
}

/// Função erro complementar `1 − erf(x)`, com precisão relativa nas caudas.
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x < -1.3 {
        return 2.0 - erfc(-x);
    }
    if x <= 1.3 {
        return 1.0 - erf_series(x);
    }
    if x > 27.3 {
        return 0.0;
    }
    (-x * x).exp() * x / SQRT_PI * erfc_cf(x)
}

/// Função erro complementar escalada `e^{x²}·erfc(x)` (sem underflow para x grande).
pub fn erfcx(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x > 1.3 {
        if x > 1e8 {
            return 1.0 / (SQRT_PI * x); // assintótico; CF não necessário
        }
        return x / SQRT_PI * erfc_cf(x);
    }
    if x < -26.7 {
        return f64::INFINITY;
    }
    (x * x).exp() * erfc(x)
}

/// Inversa de erfc: `erfc(erfcinv(z)) = z`, z ∈ [0, 2].
pub fn erfcinv(z: f64) -> f64 {
    if z.is_nan() || !(0.0..=2.0).contains(&z) {
        return f64::NAN;
    }
    if z == 0.0 {
        return f64::INFINITY;
    }
    if z == 2.0 {
        return f64::NEG_INFINITY;
    }
    if z > 1.0 {
        return -erfcinv(2.0 - z); // 2 − z é exato para z ∈ [1, 2]
    }
    if z == 1.0 {
        return 0.0;
    }
    // Chute inicial: perto de 1, linear; na cauda, iteração de x = √(−ln(z·x·√π)).
    let mut x = if z > 0.5 {
        (1.0 - z) * SQRT_PI / 2.0
    } else {
        let mut t = (-z.ln()).sqrt();
        for _ in 0..3 {
            t = (-(z * t * SQRT_PI).ln()).max(0.01).sqrt();
        }
        t
    };
    for _ in 0..60 {
        // f = erfc(x) − z ; f' = −2/√π e^{−x²} ; Halley: Δ = (f/f') / (1 + x·f/f')
        let f_over_fp = if x > 1.3 {
            // erfc(x) − z = e^{−x²}(erfcx(x) − z·e^{x²}); divide por f'
            -(SQRT_PI / 2.0) * (erfcx(x) - (z.ln() + x * x).exp())
        } else {
            -(SQRT_PI / 2.0) * (erfc(x) - z) * (x * x).exp()
        };
        let delta = f_over_fp / (1.0 + x * f_over_fp);
        x -= delta;
        if delta.abs() <= 1e-16 * x.abs() {
            break;
        }
    }
    x
}

/// Inversa de erf: `erf(erfinv(y)) = y`, y ∈ [−1, 1].
pub fn erfinv(y: f64) -> f64 {
    if y.is_nan() || !(-1.0..=1.0).contains(&y) {
        return f64::NAN;
    }
    if y.abs() == 1.0 {
        return f64::INFINITY.copysign(y);
    }
    if y.abs() > 0.5 {
        return erfcinv(1.0 - y.abs()).copysign(y); // 1 − |y| exato para |y| ≥ 0,5
    }
    let mut x = y * SQRT_PI / 2.0;
    for _ in 0..30 {
        let f_over_fp = (erf(x) - y) * SQRT_PI / 2.0 * (x * x).exp();
        let delta = f_over_fp / (1.0 + x * f_over_fp);
        x -= delta;
        if delta.abs() <= 1e-16 * x.abs().max(1e-300) {
            break;
        }
    }
    x
}
