//! Função gama com argumento complexo.
//!
//! - ln Γ(z) no **ramo principal**: a continuação analítica de ln Γ(x), x > 0, ao plano cortado
//!   no semieixo real negativo, com o limite por cima sobre o corte. Satisfaz
//!   ln Γ(z + 1) = ln Γ(z) + ln z com ln principal (DLMF 5.5.1) e ln Γ(z̄) = conj(ln Γ(z)).
//! - Re z ≥ ½: deslocamento até |w| ≥ 10 pela recorrência e série de Stirling (DLMF 5.11.1);
//!   perto das raízes 1 e 2, série de Taylor com ζ(k) (DLMF 5.7.3).
//! - Re z < ½, Im z ≥ 0: reflexão (DLMF 5.5.3) com o ramo de ln sen(πz) contínuo no semiplano
//!   superior, ln sen(πz) = πy − ln 2 + i(π/2 − πx) + ln(1 − e^{2πiz}), em que o último termo tem
//!   parte imaginária em (−π/2, π/2). Esse ramo torna a reflexão exata para o ramo principal.
//! - Im z < 0: simetria de conjugação.
//! - Γ(z) = exp(ln Γ(z)); no eixo real, a gama real.

use std::f64::consts::{LN_2, PI};
use std::sync::OnceLock;

use num_complex::Complex64;

use super::gamma::{LN_SQRT_2PI, bernoulli, cospi, gamma, gammaln, psi, sinpi, zeta_table};
use super::zeta::polygamma;

/// |w| mínimo para a série de Stirling com 10 termos. O primeiro termo omitido,
/// B₂₂/(22·21·w²¹), vale < 1,4e-20 em |w| = 10; em Re w ≥ ½ (|ph w| < π/2) a cota do resto
/// ganha no máximo o fator sec²¹(π/4) ≈ 1,5e3 (DLMF 5.11.ii), ainda < 3e-17. Um raio menor
/// diminui o cancelamento entre Stirling e a soma dos ln(z + k) perto das raízes 1 e 2.
const STIRLING_RADIUS: f64 = 10.0;
/// Raio dos discos em torno de 1 e 2 em que se usa a série de Taylor.
const TAYLOR_RADIUS: f64 = 0.5;
/// Raio do disco em torno de 3/2. Cobre a região em volta dos dois discos de Taylor em que |ln Γ|
/// é menor que 1; ali o erro absoluto de Stirling (alguns ulp de ~20) viraria erro relativo de
/// até 2e-14.
const MIDDLE_RADIUS: f64 = 1.1;
/// Número de coeficientes da série em 3/2: com |ε| ≤ 1,1 os termos são ~(11/15)ᵏ/k, < 1e-20 em
/// k = 140.
const MIDDLE_TERMS: usize = 144;
/// Faixa em que se desce pela recorrência até o disco em torno de 3/2, em vez de Stirling.
const DOWNWARD_MAX_RE: f64 = 4.5;
const DOWNWARD_MAX_IM: f64 = 1.0;
/// Acima deste Im z, |ln(1 − e^{2πiz})| ≤ 2e^{−2πy} < 1e-54, desprezível em f64 diante de
/// πy > 62: ln sen(πz) = πy − ln 2 + i(π/2 − πx) exatamente em ponto flutuante.
const LARGE_IM: f64 = 20.0;

/// ln Γ(z) no ramo principal. Polos (z inteiro ≤ 0 real) → `Inf + NaN·i`; z com parte
/// infinita ou NaN → NaN, exceto z = +Inf real (→ +Inf) e z = −Inf real (→ `Inf + NaN·i`). Não há
/// guarda explícita: NaN e infinitos se propagam pelas fórmulas (teste exaustivo nos casos de
/// borda).
pub fn gammaln_complex(z: Complex64) -> Complex64 {
    if z.im < 0.0 {
        return upper(z.conj()).conj();
    }
    upper(z)
}

/// Γ(z) complexa. No eixo real coincide com [`gamma`].
pub fn gamma_complex(z: Complex64) -> Complex64 {
    if z.im == 0.0 {
        return Complex64::new(gamma(z.re), 0.0);
    }
    gammaln_complex(z).exp()
}

/// ln Γ(z) para Im z ≥ 0.
fn upper(z: Complex64) -> Complex64 {
    let (x, y) = (z.re, z.im);
    if y == 0.0 {
        return on_real_axis(x);
    }
    if x < 0.5 {
        // ln Γ(z) = ln π − ln sen(πz) − ln Γ(1 − z)
        return Complex64::new(PI.ln(), 0.0) - ln_sin_pi(x, y) - gammaln_complex(1.0 - z);
    }
    right_half(z)
}

/// Limite por cima no eixo real: Re = ln|Γ(x)| e Im = −π·⌈−x⌉ para x < 0 (cada fator
/// ln(x + k) negativo da recorrência contribui com iπ).
fn on_real_axis(x: f64) -> Complex64 {
    if x > 0.0 {
        return Complex64::new(gammaln(x), 0.0);
    }
    if x.fract() == 0.0 || x.is_infinite() {
        return Complex64::new(f64::INFINITY, f64::NAN);
    }
    Complex64::new(gammaln(x), -PI * (-x).ceil())
}

/// Ramo de ln sen(πz) contínuo em Im z > 0 (ver o cabeçalho do módulo).
fn ln_sin_pi(x: f64, y: f64) -> Complex64 {
    let principal_im_base = 0.5 * PI - PI * x;
    if is_large_im(y) {
        return Complex64::new(PI * y - LN_2, principal_im_base);
    }
    // sen(πz) = sen(πx)cosh(πy) + i cos(πx)senh(πy); |sen(πz)|² = sen²(πx) + senh²(πy)
    let (s, c) = (sinpi(x), cospi(x));
    let sh = (PI * y).sinh();
    // hypot: s² + senh² sofreria underflow para |z| < 1e-154 (ln 0 = −Inf).
    let re = s.hypot(sh).ln();
    let arg = (c * sh).atan2(s * (PI * y).cosh());
    // Escolhe o ramo: Im = π/2 − πx + δ com |δ| < π/2.
    let k = ((principal_im_base - arg) / (2.0 * PI)).round();
    Complex64::new(re, arg + 2.0 * PI * k)
}

/// As três fronteiras abaixo separam métodos que concordam no limite, de modo que trocar `<` por
/// `<=` nelas é um mutante equivalente (registrado em `.cargo/mutants.toml`).
fn is_large_im(y: f64) -> bool {
    y > LARGE_IM
}

fn needs_shift(w: Complex64) -> bool {
    w.norm_sqr() < STIRLING_RADIUS * STIRLING_RADIUS
}

/// Faixa em que se desce pela recorrência até o disco em torno de 3/2 em vez de usar Stirling.
fn downward_applies(z: Complex64) -> bool {
    z.re > 2.0 && z.re <= DOWNWARD_MAX_RE && z.im.abs() <= DOWNWARD_MAX_IM
}

/// ln Γ(z) para Re z ≥ ½, Im z > 0.
fn right_half(z: Complex64) -> Complex64 {
    if (z - 1.0).norm() <= TAYLOR_RADIUS {
        return lngamma1p(z - 1.0);
    }
    if (z - 2.0).norm() <= TAYLOR_RADIUS {
        return ln1p(z - 2.0) + lngamma1p(z - 2.0);
    }
    if (z - 1.5).norm() <= MIDDLE_RADIUS {
        return around_three_halves(z - 1.5);
    }
    if downward_applies(z) {
        // ln Γ(z) = ln(z − 1) + ln Γ(z − 1): parcelas de mesmo sinal, sem cancelamento.
        let w = z - 1.0;
        return w.ln() + right_half(w);
    }
    // Desloca até |z + n| ≥ 10 e subtrai Σ ln(z + k) = ln Π|z + k| + i Σ arg(z + k).
    let mut w = z;
    let mut modulus = 1.0;
    let mut args = 0.0;
    while needs_shift(w) {
        modulus *= w.norm();
        args += w.im.atan2(w.re);
        w += 1.0;
    }
    stirling(w) - Complex64::new(modulus.ln(), args)
}

/// (w − ½) ln w − w + ln √(2π) + Σ B₂ₖ / (2k(2k−1) w^(2k−1)), k = 1..10, |w| ≥ 10.
fn stirling(w: Complex64) -> Complex64 {
    let inv = w.inv();
    let inv2 = inv * inv;
    let mut p = inv;
    let mut s = Complex64::new(0.0, 0.0);
    for k in 1..=10 {
        let kk = k as f64;
        s += p * (bernoulli(k) / (2.0 * kk * (2.0 * kk - 1.0)));
        p *= inv2;
    }
    (w - 0.5) * w.ln() - w + LN_SQRT_2PI + s
}

/// ln(1 + e) sem perder dígitos para |e| pequeno.
fn ln1p(e: Complex64) -> Complex64 {
    // |1 + e|² = 1 + (2a + a² + b²)
    let t = e.re * (2.0 + e.re) + e.im * e.im;
    Complex64::new(0.5 * t.ln_1p(), e.im.atan2(1.0 + e.re))
}

/// ln Γ(1 + ε) para |ε| ≤ ½ (DLMF 5.7.3): −γε + Σ (−1)^k ζ(k) ε^k / k, k = 2..64 (o último
/// termo é < 1e-21).
fn lngamma1p(e: Complex64) -> Complex64 {
    let zt = zeta_table();
    let mut s = -std::f64::consts::EULER_GAMMA * e;
    let mut p = -e;
    for (k, zk) in zt.iter().enumerate().skip(2) {
        p *= -e;
        let term = p * (zk / k as f64);
        s += term;
    }
    s
}

/// Coeficientes de ln Γ(3/2 + ε) = ln Γ(3/2) + Σₖ cₖ εᵏ: c₁ = ψ(3/2), cₖ = ψ⁽ᵏ⁻¹⁾(3/2)/k!.
/// O raio de convergência é 3/2 (polo em 0).
fn three_halves_coeffs() -> &'static [f64; MIDDLE_TERMS] {
    static T: OnceLock<[f64; MIDDLE_TERMS]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0.0; MIDDLE_TERMS];
        t[1] = psi(1.5);
        let mut fact = 1.0;
        for (k, slot) in t.iter_mut().enumerate().skip(2) {
            fact *= k as f64;
            *slot = polygamma(k as u32 - 1, 1.5) / fact;
        }
        t
    })
}

/// ln Γ(3/2 + ε) para |ε| ≤ 1,1.
fn around_three_halves(e: Complex64) -> Complex64 {
    let mut s = Complex64::new(0.0, 0.0);
    let mut p = Complex64::new(1.0, 0.0);
    for ck in &three_halves_coeffs()[1..] {
        p *= e;
        s += p * ck;
    }
    s + gammaln(1.5)
}
