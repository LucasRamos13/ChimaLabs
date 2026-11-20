//! Funções de Bessel J, Y, I, K de ordem real e argumento real, e funções de Airy.
//!
//! - J_ν, I_ν para argumento pequeno: séries ascendentes (DLMF 10.2.2, 10.25.2);
//! - J_ν, Y_ν para argumento grande: expansão assintótica de Hankel (DLMF 10.17.3–4);
//! - J_ν, Y_ν na faixa intermediária: integrais de Schläfli (DLMF 10.9.6–7), avaliadas por
//!   quadratura de dupla exponencial;
//! - K_ν: integral ∫₀^∞ e^{−x cosh t} cosh(νt) dt (DLMF 10.32.9);
//! - I_ν para argumento grande: Wronskiano I_ν K_{ν+1} + I_{ν+1} K_ν = 1/x (DLMF 10.28.2) com a
//!   razão I_{ν+1}/I_ν por fração contínua (DLMF 10.33.1);
//! - ordens negativas: DLMF 10.4.7 e 10.27.2–3;
//! - Hankel: H⁽¹⁾ = J + iY e H⁽²⁾ = J − iY (DLMF 10.4.3);
//! - Airy: série de Maclaurin (DLMF 9.4.1–2) e relações com Bessel de ordem 1/3 (DLMF 9.6).

use std::f64::consts::PI;

use num_complex::Complex64;

use super::gamma::sinpi;
use super::quad::{exp_sinh, tanh_sinh};
use super::{gamma, gammaln, lentz};

const TOL: f64 = 1e-15;

fn cospi(x: f64) -> f64 {
    sinpi(x + 0.5)
}

/// (x/2)^ν / Γ(ν+1), sem transbordar em passos intermediários.
fn series_prefactor(nu: f64, x: f64) -> f64 {
    if nu == 0.0 {
        return 1.0;
    }
    let g = gamma(nu + 1.0);
    if g.is_finite() && nu < 100.0 {
        return (0.5 * x).powf(nu) / g;
    }
    (nu * (0.5 * x).ln() - gammaln(nu + 1.0)).exp()
}

/// Série ascendente de J_ν (sign = −1) ou I_ν (sign = +1), ν ≥ 0. Devolve o valor e o fator de
/// cancelamento Σ|termos|/|soma| (1 = sem cancelamento).
fn ascending_with_cancel(nu: f64, x: f64, sign: f64) -> (f64, f64) {
    let q = sign * 0.25 * x * x;
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut abs_sum = 1.0;
    for k in 1..2000 {
        let kk = k as f64;
        term *= q / (kk * (nu + kk));
        sum += term;
        abs_sum += term.abs();
        if term.abs() <= 1e-17 * sum.abs() {
            break;
        }
    }
    (series_prefactor(nu, x) * sum, abs_sum / sum.abs())
}

fn ascending(nu: f64, x: f64, sign: f64) -> f64 {
    ascending_with_cancel(nu, x, sign).0
}

/// Expansão de Hankel: devolve (J_ν, Y_ν) se convergir.
fn hankel(nu: f64, x: f64) -> Option<(f64, f64)> {
    let mu = 4.0 * nu * nu;
    let (mut p, mut q) = (1.0, 0.0);
    let mut a = 1.0; // a_k(ν)/x^k
    let mut prev = f64::INFINITY;
    let mut converged = false;
    for k in 1..200 {
        let kk = k as f64;
        a *= (mu - (2.0 * kk - 1.0).powi(2)) / (kk * 8.0 * x);
        if a.abs() > prev {
            break; // série assintótica começou a divergir
        }
        prev = a.abs();
        // P = Σ (−1)^k a_{2k}, Q = Σ (−1)^k a_{2k+1}
        let sgn = if (k / 2) % 2 == 0 { 1.0 } else { -1.0 };
        if k % 2 == 0 {
            p += sgn * a;
        } else {
            q += sgn * a;
        }
        if a.abs() <= 1e-17 * p.abs().max(q.abs()) || a == 0.0 {
            converged = true;
            break;
        }
    }
    if !converged {
        return None;
    }
    // ω = x − (ν/2 + 1/4)π; cos/sin por soma de arcos (x grande reduzido pela libm)
    let c = (0.5 * nu + 0.25) * PI;
    let (sx, cx) = x.sin_cos();
    let (sc, cc) = c.sin_cos();
    let (sw, cw) = (sx * cc - cx * sc, cx * cc + sx * sc);
    let amp = (2.0 / (PI * x)).sqrt();
    Some((amp * (p * cw - q * sw), amp * (p * sw + q * cw)))
}

/// (J_ν, Y_ν) pelas integrais de Schläfli, ν ≥ 0, x > 0.
fn schlafli(nu: f64, x: f64) -> (f64, f64) {
    let j1 = tanh_sinh(|t| (x * t.sin() - nu * t).cos(), 0.0, PI, TOL, 1.0) / PI;
    let y1 = tanh_sinh(|t| (x * t.sin() - nu * t).sin(), 0.0, PI, TOL, 1.0) / PI;
    let s = sinpi(nu);
    let c = cospi(nu);
    let j2 = if s == 0.0 {
        0.0
    } else {
        exp_sinh(|t| (-x * t.sinh() - nu * t).exp(), TOL, 0.0)
    };
    // (e^{νt} + e^{−νt}·cos νπ)·e^{−x sinh t}, cada parcela num único exp (evita 0·Inf)
    let y2 = exp_sinh(
        |t| (nu * t - x * t.sinh()).exp() + c * (-nu * t - x * t.sinh()).exp(),
        TOL,
        0.0,
    );
    (j1 - s / PI * j2, y1 - y2 / PI)
}

/// (J_ν, Y_ν) para ν ≥ 0, x > 0.
fn jy_positive(nu: f64, x: f64) -> (f64, f64) {
    if x >= 25.0
        && x >= 0.5 * nu * nu + 25.0
        && let Some(r) = hankel(nu, x)
    {
        return r;
    }
    let (j, y) = schlafli(nu, x);
    // Fora da região oscilatória J é pequeno e a integral só tem precisão absoluta; a série
    // ascendente é preferida sempre que o cancelamento medido for pequeno (≤ 64, < 2 dígitos).
    if x <= nu + 12.0 {
        let (js, cancel) = ascending_with_cancel(nu, x, -1.0);
        if cancel <= 64.0 {
            return (js, y);
        }
    }
    (j, y)
}

/// e^x·K_ν(x) para ν ≥ 0, x > 0.
fn k_scaled(nu: f64, x: f64) -> f64 {
    // e^{−x(cosh t − 1)}·cosh(νt) = ½(e^{νt − x(cosh t − 1)} + e^{−νt − x(cosh t − 1)})
    exp_sinh(
        |t| {
            let base = -x * (t.cosh() - 1.0);
            0.5 * ((nu * t + base).exp() + (-nu * t + base).exp())
        },
        TOL,
        0.0,
    )
}

/// e^{−x}·I_ν(x) para ν ≥ 0, x > 0.
fn i_scaled(nu: f64, x: f64) -> f64 {
    if x <= 30.0 {
        return ascending(nu, x, 1.0) * (-x).exp();
    }
    // r = I_{ν+1}/I_ν = 1/(2(ν+1)/x + 1/(2(ν+2)/x + …))
    let r = lentz(0.0, |k| (1.0, 2.0 * (nu + k as f64) / x));
    1.0 / (x * (k_scaled(nu + 1.0, x) + r * k_scaled(nu, x)))
}

fn is_int(nu: f64) -> bool {
    nu.fract() == 0.0
}

/// Bessel de primeira espécie J_ν(x). Para x < 0 e ν não inteiro o valor é complexo (NaN; ver
/// `besselj_complex`).
pub fn besselj(nu: f64, x: f64) -> f64 {
    if nu.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if x < 0.0 {
        if !is_int(nu) {
            return f64::NAN;
        }
        let v = besselj(nu, -x);
        return if (nu as i64) % 2 == 0 { v } else { -v };
    }
    if x == 0.0 {
        return if nu == 0.0 {
            1.0
        } else if nu > 0.0 || is_int(nu) {
            0.0
        } else {
            f64::INFINITY / gamma(nu + 1.0).signum()
        };
    }
    if x.is_infinite() {
        return 0.0;
    }
    if nu >= 0.0 {
        return jy_positive(nu, x).0;
    }
    let m = -nu;
    if is_int(m) {
        let v = jy_positive(m, x).0;
        return if (m as i64) % 2 == 0 { v } else { -v };
    }
    let (j, y) = jy_positive(m, x);
    cospi(m) * j - sinpi(m) * y
}

/// Bessel de segunda espécie Y_ν(x), x > 0 (Y_ν(0) = −Inf; x < 0 → NaN).
pub fn bessely(nu: f64, x: f64) -> f64 {
    if nu.is_nan() || x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x.is_infinite() {
        return 0.0;
    }
    if nu >= 0.0 {
        return jy_positive(nu, x).1;
    }
    let m = -nu;
    let (j, y) = jy_positive(m, x);
    if is_int(m) {
        return if (m as i64) % 2 == 0 { y } else { -y };
    }
    sinpi(m) * j + cospi(m) * y
}

/// Tipo da função de Hankel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HankelKind {
    /// H⁽¹⁾_ν = J_ν + i·Y_ν.
    First,
    /// H⁽²⁾_ν = J_ν − i·Y_ν.
    Second,
}

/// Função de Hankel H⁽¹⁾_ν(x) ou H⁽²⁾_ν(x), ordem real e argumento real `x ≥ 0` (DLMF 10.4.3).
/// Com `scaled`, devolve H⁽¹⁾·e^{−ix} ou H⁽²⁾·e^{ix}, que não oscila para `x` grande.
/// `x < 0` dá NaN (o valor exige continuação analítica: ver `besselh_complex`); em `x = 0` a
/// parte imaginária é ∓Inf.
pub fn besselh(nu: f64, kind: HankelKind, x: f64, scaled: bool) -> Complex64 {
    // NaN em ν ou x se propaga por J e Y (e pelo fator de escala).
    if x < 0.0 {
        return Complex64::new(f64::NAN, f64::NAN);
    }
    let (j, y) = (besselj(nu, x), bessely(nu, x));
    let h = match kind {
        HankelKind::First => Complex64::new(j, y),
        HankelKind::Second => Complex64::new(j, -y),
    };
    if !scaled || x == 0.0 || x.is_infinite() {
        return h;
    }
    let (s, c) = x.sin_cos();
    match kind {
        HankelKind::First => h * Complex64::new(c, -s),
        HankelKind::Second => h * Complex64::new(c, s),
    }
}

/// Bessel modificada de primeira espécie I_ν(x); com `scaled`, devolve e^{−|x|}·I_ν(x).
pub fn besseli(nu: f64, x: f64, scaled: bool) -> f64 {
    if nu.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if x < 0.0 {
        if !is_int(nu) {
            return f64::NAN;
        }
        let v = besseli(nu, -x, scaled);
        return if (nu as i64) % 2 == 0 { v } else { -v };
    }
    if x == 0.0 {
        return if nu == 0.0 {
            1.0
        } else if nu > 0.0 || is_int(nu) {
            0.0
        } else {
            f64::INFINITY
        };
    }
    let m = nu.abs();
    let mut v = i_scaled(m, x);
    if nu < 0.0 && !is_int(m) {
        // I_{−ν} = I_ν + (2/π) sin(νπ) K_ν; em forma escalada, K entra multiplicado por e^{−2x}
        v += 2.0 / PI * sinpi(m) * k_scaled(m, x) * (-2.0 * x).exp();
    }
    if scaled { v } else { v * x.exp() }
}

/// Bessel modificada de segunda espécie K_ν(x), x > 0; com `scaled`, devolve e^{x}·K_ν(x).
pub fn besselk(nu: f64, x: f64, scaled: bool) -> f64 {
    if nu.is_nan() || x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::INFINITY;
    }
    if x.is_infinite() {
        return 0.0;
    }
    let v = k_scaled(nu.abs(), x);
    if scaled { v } else { v * (-x).exp() }
}

/// Ai(0) = 3^(−2/3)/Γ(2/3) = 0,35502805388781723926… (DLMF 9.2.3), arredondado para f64.
/// Constante em vez de cálculo: com `gamma(1/3)` (erro ~2e-15) Ai′(0) errava 2,3e-15.
pub(crate) const AIRY_C1: f64 = 0.355_028_053_887_817_2;
/// −Ai′(0) = 3^(−1/3)/Γ(1/3) = 0,25881940379280679840… (DLMF 9.2.4), arredondado para f64.
pub(crate) const AIRY_C2: f64 = 0.258_819_403_792_806_8;

/// Funções de Airy `(Ai(x), Bi(x))`. Em ±Inf, os limites: (0, +Inf) e (0, 0).
pub fn airy(x: f64) -> (f64, f64) {
    if x.is_nan() {
        return (f64::NAN, f64::NAN);
    }
    // Limites: Ai(+∞) = 0, Bi(+∞) = +∞; em −∞ ambas oscilam com amplitude |x|^(−1/4) → 0.
    if x == f64::INFINITY {
        return (0.0, f64::INFINITY);
    }
    if x == f64::NEG_INFINITY {
        return (0.0, 0.0);
    }
    let (c1, c2) = (AIRY_C1, AIRY_C2);
    if x.abs() <= 1.0 {
        let x3 = x * x * x;
        let (mut f, mut g) = (1.0, x);
        let (mut sf, mut sg) = (1.0, x);
        for k in 1..60 {
            let kk = k as f64;
            f *= x3 / ((3.0 * kk - 1.0) * (3.0 * kk));
            g *= x3 / ((3.0 * kk) * (3.0 * kk + 1.0));
            sf += f;
            sg += g;
            if f.abs() + g.abs() <= 1e-17 * (sf.abs() + sg.abs()) {
                break;
            }
        }
        return (c1 * sf - c2 * sg, 3f64.sqrt() * (c1 * sf + c2 * sg));
    }
    let third = 1.0 / 3.0;
    if x > 0.0 {
        let z = 2.0 / 3.0 * x.powf(1.5);
        let ks = k_scaled(third, z);
        let ai = (x / 3.0).sqrt() / PI * ks * (-z).exp();
        // Bi = √(x/3)·(2·I_{1/3}(ζ) + (2/π)·sin(π/3)·K_{1/3}(ζ))
        let bi = (x / 3.0).sqrt()
            * (2.0 * besseli(third, z, false) + 2.0 / PI * (PI / 3.0).sin() * ks * (-z).exp());
        return (ai, bi);
    }
    let zz = -x;
    let z = 2.0 / 3.0 * zz.powf(1.5);
    let (j, y) = jy_positive(third, z);
    let s3 = 3f64.sqrt();
    let h = 0.5 * zz.sqrt();
    (h * (j - y / s3), -h * (j / s3 + y))
}

/// Derivadas das funções de Airy `(Ai′(x), Bi′(x))`. Em +Inf, (−0, +Inf); em −Inf (sem limite) e
/// NaN, (NaN, NaN).
///
/// |x| ≤ 1: série de Maclaurin derivada termo a termo (DLMF 9.4.1–2). Fora: Bessel de ordem 2/3
/// (DLMF 9.6.2, 9.6.4, 9.6.7, 9.6.9), com J₋₂/₃ = −½J₂/₃ − (√3/2)Y₂/₃ e
/// I₋₂/₃ = I₂/₃ + (2/π)·sin(2π/3)·K₂/₃.
pub fn airy_prime(x: f64) -> (f64, f64) {
    if x == f64::INFINITY {
        return (-0.0, f64::INFINITY);
    }
    if !x.is_finite() {
        // NaN; e em −∞ as derivadas oscilam com amplitude |x|^(1/4) → ∞: sem limite.
        return (f64::NAN, f64::NAN);
    }
    let (c1, c2) = (AIRY_C1, AIRY_C2);
    if x.abs() <= 1.0 {
        // f′ = Σ_{k≥1} uₖ, u₁ = x²/2, uₖ/uₖ₋₁ = x³/((3k − 1)(3k − 3));
        // g′ = Σ_{k≥0} vₖ, v₀ = 1, vₖ/vₖ₋₁ = x³/(3k(3k − 2)).
        let x3 = x * x * x;
        let (mut u, mut v) = (0.5 * x * x, 1.0);
        let (mut su, mut sv) = (u, v);
        // |x³| ≤ 1: o k-ésimo termo é ≤ Π 1/((3j + 2)·3j) < 1e-20 já em k = 12.
        for k in 1..=12 {
            let kk = k as f64;
            u *= x3 / ((3.0 * kk + 2.0) * (3.0 * kk));
            v *= x3 / ((3.0 * kk) * (3.0 * kk - 2.0));
            su += u;
            sv += v;
        }
        return (c1 * su - c2 * sv, 3f64.sqrt() * (c1 * su + c2 * sv));
    }
    let two_thirds = 2.0 / 3.0;
    let s3 = 3f64.sqrt();
    // |x| > 1 aqui (x = 0 ficou na série): basta o sinal.
    if x.is_sign_positive() {
        let z = two_thirds * x.powf(1.5);
        let ks = k_scaled(two_thirds, z);
        let aip = -x / (PI * s3) * ks * (-z).exp();
        let bip = x / s3
            * (2.0 * besseli(two_thirds, z, false) + 2.0 / PI * (2.0 * PI / 3.0).sin() * ks * (-z).exp());
        return (aip, bip);
    }
    let zz = -x;
    let z = two_thirds * zz.powf(1.5);
    let (j, y) = jy_positive(two_thirds, z);
    let h = 0.5 * zz;
    (h * (j + y / s3), h * (j / s3 - y))
}
