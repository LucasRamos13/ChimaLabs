//! Função erro com argumento complexo, pela função de Faddeeva w(z) = e^(−z²) erfc(−iz)
//! (DLMF 7.2.3).
//!
//! - Im z ≥ 0, |z| < 8: regra do trapézio aplicada a w(z) = (i/π) ∫ e^(−t²)/(z − t) dt
//!   (DLMF 7.7.2) com passo h e a correção exata do polo t = z. Com nós em nh:
//!   w(z) ≈ (ih/π)[1/z + 2z Σₙ e^(−n²h²)/(z² − n²h²)] + 2e^(−z²)/(1 − e^(−2πiz/h));
//!   com nós em (n + ½)h, o mesmo sem o termo 1/z e com 1 + e^(−2πiz/h). Usa-se a malha cujos
//!   nós estão mais longe de Re z, para não haver cancelamento entre a soma e a correção.
//!   O erro de discretização é O(e^(−π²/h²)); com h = 0,35, menor que 1e-34.
//! - Im z ≥ 0, |z| ≥ 8: fração contínua de Laplace w(z) = (i/√π)/(z − ½/(z − 1/(z − (3/2)/…)))
//!   (DLMF 7.9.3). No eixo real ela não vê a parcela e^(−x²) de Re w, que em |x| ≥ 8 é < 2e-28
//!   relativo.
//! - Im z < 0: w(z) = 2e^(−z²) − w(−z) (DLMF 7.4.3).
//! - erf, erfc e erfcx por erfc(z) = e^(−z²) w(iz) para Re z ≥ 0 e erfc(−z) = 2 − erfc(z)
//!   (DLMF 7.4.2); erf com |z| ≤ ½ pela série de Taylor (DLMF 7.6.1), sem o cancelamento de
//!   1 − erfc. No eixo real, as versões complexas devolvem exatamente as reais.

use std::f64::consts::{FRAC_2_SQRT_PI, PI};

use num_complex::Complex64;

use super::erf::{erf, erfc, erfcx};

/// Passo da regra do trapézio.
const H: f64 = 0.35;
/// Termos da soma: e^(−n²h²) < 1e-19 a partir de n = 19.
const TERMS: usize = 19;
/// |z| a partir do qual se usa a fração contínua.
const CF_RADIUS: f64 = 8.0;
/// Número de níveis da fração contínua (avaliada de trás para a frente).
const CF_LEVELS: usize = 60;
/// |z| até o qual erf usa a série de Taylor.
const TAYLOR_RADIUS: f64 = 0.5;
/// Termos da série de Taylor: |z|^(2n)/n! < 1e-17 em n = 14 para |z| ≤ ½.
const TAYLOR_TERMS: usize = 16;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

/// e^(−z²), com a parte real do expoente calculada como (y − x)(y + x).
fn exp_minus_sq(z: Complex64) -> Complex64 {
    c((z.im - z.re) * (z.im + z.re), -2.0 * z.re * z.im).exp()
}

/// Função de Faddeeva w(z) = e^(−z²) erfc(−iz).
pub fn faddeeva(z: Complex64) -> Complex64 {
    if z.im < 0.0 {
        return 2.0 * exp_minus_sq(z) - upper(-z);
    }
    upper(z)
}

fn upper(z: Complex64) -> Complex64 {
    if z.norm() >= CF_RADIUS {
        return continued_fraction(z);
    }
    trapezoid(z)
}

/// Regra do trapézio com correção de polo (ver o cabeçalho do módulo).
fn trapezoid(z: Complex64) -> Complex64 {
    let t = z.re / H;
    let shifted = (t - t.round()).abs() < 0.25;
    let z2 = z * z;
    let mut s = if shifted { c(0.0, 0.0) } else { z.inv() };
    let offset = if shifted { 0.5 } else { 1.0 };
    for n in 0..TERMS {
        let node = (n as f64 + offset) * H;
        let node2 = node * node;
        s += 2.0 * z * ((-node2).exp() / (z2 - node2));
    }
    let e = c(2.0 * PI * z.im / H, -2.0 * PI * z.re / H).exp();
    let denom = if shifted { 1.0 + e } else { 1.0 - e };
    c(0.0, H / PI) * s + 2.0 * exp_minus_sq(z) / denom
}

/// Fração contínua de Laplace (DLMF 7.9.3), para |z| ≥ 8 e Im z ≥ 0.
fn continued_fraction(z: Complex64) -> Complex64 {
    // `finv` escala pelo módulo: a divisão comum calcula |r|² e transborda para |z| > 1e154.
    let mut r = z;
    for k in (1..=CF_LEVELS).rev() {
        r = z - (0.5 * k as f64) * r.finv();
    }
    c(0.0, 0.5 * FRAC_2_SQRT_PI) * r.finv()
}

/// erfcx(z) = e^(z²) erfc(z) complexa. No eixo real coincide com [`erfcx`].
pub fn erfcx_complex(z: Complex64) -> Complex64 {
    if z.im == 0.0 {
        return c(erfcx(z.re), 0.0);
    }
    // erfcx(z) = w(iz); para Re z < 0, erfcx(z) = 2e^(z²) − erfcx(−z).
    // `re < 0` (e não `re ≥ 0` negado) para que NaN siga pelo caminho direto.
    let iz = c(-z.im, z.re);
    if z.re < 0.0 {
        return 2.0 * exp_minus_sq(iz) - upper(-iz);
    }
    upper(iz)
}

/// erfc(z) complexa. No eixo real coincide com [`erfc`].
pub fn erfc_complex(z: Complex64) -> Complex64 {
    if z.im == 0.0 {
        return c(erfc(z.re), 0.0);
    }
    // Com `re ≥ 0` na reflexão, NaN recursaria sem fim.
    if z.re < 0.0 {
        return 2.0 - erfc_complex(-z);
    }
    exp_minus_sq(z) * upper(c(-z.im, z.re))
}

/// erf(z) complexa. No eixo real coincide com [`erf`].
pub fn erf_complex(z: Complex64) -> Complex64 {
    if z.im == 0.0 {
        return c(erf(z.re), 0.0);
    }
    if z.norm() <= TAYLOR_RADIUS {
        return taylor(z);
    }
    if z.re < 0.0 {
        return -erf_complex(-z);
    }
    1.0 - exp_minus_sq(z) * upper(c(-z.im, z.re))
}

/// erf z = (2/√π) Σ (−1)ⁿ z^(2n+1) / (n!(2n+1)), |z| ≤ ½.
fn taylor(z: Complex64) -> Complex64 {
    let mz2 = -z * z;
    let mut term = z;
    let mut s = z;
    for n in 1..TAYLOR_TERMS {
        term *= mz2 / n as f64;
        s += term / (2 * n + 1) as f64;
    }
    FRAC_2_SQRT_PI * s
}
