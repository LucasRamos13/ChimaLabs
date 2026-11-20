//! Funções zeta e poligama.
//!
//! - ζ de Hurwitz ζ(s, q) = Σₖ (q + k)⁻ˢ: soma direta até q + N ≥ s + 20 (com parada antecipada
//!   quando a cauda Σ ≤ termo·(q + N)/(s − 1) fica desprezível) e fórmula de Euler-Maclaurin no
//!   restante (DLMF 25.11.5 e 2.10.1), com B₂…B₂₀. A condição q + N ≥ s + 20 limita a razão entre
//!   termos de correção consecutivos a ≈ 1/(2π)², e o erro de truncamento fica abaixo de ε.
//!   Calcula-se qˢ·ζ(s, q) (termos (q/(q + k))ˢ ≤ 1), para não haver subestouro intermediário.
//! - ζ de Riemann: Euler-Maclaurin (ζ(s) = ζ(s, 1), válida por continuação analítica para
//!   s > −19) quando s ≥ 1/2; equação funcional (DLMF 25.4.1) quando s < 1/2.
//! - Poligama ψ⁽ⁿ⁾(x) = (−1)ⁿ⁺¹·n!·ζ(n + 1, x) (DLMF 25.11.12) para x > 0; para x < 0, a
//!   recorrência ψ⁽ⁿ⁾(x + 1) = ψ⁽ⁿ⁾(x) + (−1)ⁿ·n!·x⁻ⁿ⁻¹ (DLMF 5.15.5) leva a x + m > 0.

use std::f64::consts::PI;

use super::gamma::{bernoulli, cospi, gamma, gammaln, psi, sinpi};

/// `qˢ·ζ(s, q)` para `s > 0`, `s ≠ 1`, `q > 0` finitos (para `s < 1`, o valor da continuação
/// analítica da fórmula de Euler-Maclaurin; só usado com `q = 1`). `sm1 = s − 1` vem calculado
/// pelo chamador: perto do polo, recalcular `s − 1` a partir de um `s` já arredondado (como
/// `1 − s` na equação funcional) perderia dígitos.
fn hurwitz_scaled(s: f64, sm1: f64, q: f64) -> f64 {
    let target = 10f64.max(s + 20.0);
    let mut sum = 0.0;
    let mut a = q;
    while a < target {
        let term = (q / a).powf(s);
        if term == 0.0 {
            // Os termos decrescem: todos os seguintes, e a correção de Euler-Maclaurin (múltiplos
            // de (q/a')ˢ ≤ termo), também são nulos em f64. Com s grande (ζ(10⁶) = 1 + 2⁻¹⁰⁶…),
            // isso evita O(s) iterações até a ≥ s + 20.
            return sum;
        }
        sum += term;
        a += 1.0;
    }
    // Euler-Maclaurin a partir de a = q + N, multiplicada por qˢ.
    let r = (q / a).powf(s);
    sum += r * a / sm1 + 0.5 * r;
    // Σⱼ B₂ⱼ/(2j)!·s(s+1)…(s+2j−2)·a^(1−2j), vezes r.
    let mut rising = s;
    let mut fact = 2.0;
    let mut apow = 1.0 / a;
    for j in 1..=10 {
        sum += r * bernoulli(j) / fact * rising * apow;
        let jj = j as f64;
        rising *= (s + 2.0 * jj - 1.0) * (s + 2.0 * jj);
        fact *= (2.0 * jj + 1.0) * (2.0 * jj + 2.0);
        apow /= a * a;
    }
    sum
}

/// ζ de Hurwitz ζ(s, q) = Σ_{k ≥ 0} (q + k)⁻ˢ, para `s > 1` e `q > 0`.
///
/// `s = 1` (série divergente) → +Inf; `s < 1`, `q ≤ 0` ou NaN → NaN; `s = +Inf` → 0, 1 ou +Inf
/// conforme q > 1, q = 1 ou q < 1; `q = +Inf` → 0.
pub fn hurwitz_zeta(s: f64, q: f64) -> f64 {
    if !(s >= 1.0 && q > 0.0) {
        return f64::NAN; // inclui NaN em s ou q
    }
    if s == 1.0 {
        return f64::INFINITY;
    }
    if q == f64::INFINITY {
        return 0.0;
    }
    if s == f64::INFINITY {
        return if q > 1.0 {
            0.0
        } else if q == 1.0 {
            1.0
        } else {
            f64::INFINITY
        };
    }
    mul_pow(hurwitz_scaled(s, s - 1.0, q), q, s)
}

/// `v·x⁻ᵉ` (x > 0, e > 0) sem estouro nem subestouro intermediário evitável: x⁻ᵉ é aplicado em
/// 2ᵏ fatores x^(−e/2ᵏ) (divisão exata), cada um dentro da faixa normal. O caminho de v até o
/// resultado é monótono em escala log, então só estoura se o resultado estourar.
fn mul_pow(v: f64, x: f64, e: f64) -> f64 {
    let bits = (e * x.log2()).abs();
    let pieces = ((bits / 900.0).ceil().max(1.0) as u64).next_power_of_two();
    let f = x.powf(-e / pieces as f64);
    (0..pieces).fold(v, |acc, _| acc * f)
}

/// ζ de Riemann para `s` real. Polo em s = 1 → +Inf; ζ(0) = −1/2; zeros triviais
/// (inteiros pares negativos) exatos; ζ(+Inf) = 1; ζ(−Inf) = NaN.
pub fn zeta(s: f64) -> f64 {
    // NaN e −Inf seguem até a equação funcional e dão NaN (sin(π·(−Inf)/2) = NaN).
    if s == f64::INFINITY {
        return 1.0;
    }
    if s == 1.0 {
        return f64::INFINITY;
    }
    if s == 0.0 {
        return -0.5;
    }
    if s >= 0.5 {
        return hurwitz_scaled(s, s - 1.0, 1.0);
    }
    if s.abs() < 1e-8 {
        // ζ(s) = −½ − ½·ln(2π)·s + O(s²) (DLMF 25.6.1 e 25.6.11); aqui s/2 poderia subestourar.
        return -0.5 - 0.5 * (2.0 * PI).ln() * s;
    }
    // Equação funcional: ζ(s) = 2·(2π)^(s−1)·sin(πs/2)·Γ(1 − s)·ζ(1 − s).
    // Nos inteiros pares negativos, sin(πs/2) = 0 exatamente (redução exata do argumento): zeros
    // triviais exatos.
    let sin = sinpi(0.5 * s);
    let t = 1.0 - s;
    let zt = hurwitz_scaled(t, -s, 1.0);
    let g = gamma(t);
    if g.is_finite() {
        2.0 * (2.0 * PI).powf(-t) * sin * g * zt
    } else {
        // Aqui t > 171 e ζ(t) = 1 + 2⁻ᵗ + … arredonda para 1 em f64: ln ζ(t) = 0.
        let ln = 2f64.ln() - t * (2.0 * PI).ln() + sin.abs().ln() + gammaln(t);
        ln.exp().copysign(sin)
    }
}

/// Função poligama ψ⁽ⁿ⁾(x), a n-ésima derivada da digama; `n = 0` é [`psi`].
///
/// Polos (x inteiro ≤ 0) → NaN, como em `psi`; x = +Inf → 0 (n ≥ 1); x = −Inf → NaN.
pub fn polygamma(n: u32, x: f64) -> f64 {
    if n == 0 {
        return psi(x);
    }
    if x.is_nan() {
        return f64::NAN;
    }
    let s = f64::from(n) + 1.0;
    let sign = if n.is_multiple_of(2) { -1.0 } else { 1.0 }; // (−1)ⁿ⁺¹
    if x > 0.0 {
        if x == f64::INFINITY {
            return 0.0;
        }
        return sign * nfact_times(n, hurwitz_scaled(s, f64::from(n), x), x, s);
    }
    if x.fract() == 0.0 {
        return f64::NAN; // polos: 0, −0 e inteiros negativos (−Inf segue e dá NaN em cot(πx))
    }
    // x < 0, reflexão (DLMF 5.15.6): ψ⁽ⁿ⁾(1 − x) + (−1)ⁿ⁺¹ψ⁽ⁿ⁾(x) = (−1)ⁿ·π·dⁿ/dxⁿ cot(πx),
    // com dⁿ/dxⁿ cot(πx) = πⁿ·Pₙ(cot πx).
    let cot = cospi(x) / sinpi(x);
    let parity = -sign; // (−1)ⁿ
    let d = PI.powi(n as i32 + 1) * cot_derivative_poly(n, cot);
    sign * (parity * d - polygamma(n, 1.0 - x))
}

/// Pₙ(c), com dⁿ/dzⁿ cot z = Pₙ(cot z): P₀ = c, Pₖ₊₁ = −(1 + c²)·Pₖ′ (polinômio de grau n + 1
/// e paridade de n + 1; para c ≥ 0 os termos têm todos o mesmo sinal, sem cancelamento).
fn cot_derivative_poly(n: u32, c: f64) -> f64 {
    let mut p = vec![0.0, 1.0]; // coeficientes em potências crescentes de c
    for _ in 0..n {
        let deg = p.len(); // grau atual + 1
        let mut q = vec![0.0; deg + 1];
        // −(1 + c²)·p′: coeficiente de cᶦ = −((i + 1)·p[i+1] + (i − 1)·p[i−1]).
        for (i, qi) in q.iter_mut().enumerate() {
            let a = if i + 1 < deg {
                (i + 1) as f64 * p[i + 1]
            } else {
                0.0
            };
            let b = if i >= 1 { (i as f64 - 1.0) * p[i - 1] } else { 0.0 };
            *qi = -(a + b);
        }
        p = q;
    }
    p.iter().rev().fold(0.0, |acc, &k| acc * c + k)
}

/// n! como f64 (Inf acima de 170).
fn factorial_u(n: u32) -> f64 {
    gamma(f64::from(n) + 1.0)
}

/// `n!·c·x⁻ˢ`. Para n > 170 (n! estoura), em log (erro relativo ~|ln resultado|·ε).
fn nfact_times(n: u32, c: f64, x: f64, s: f64) -> f64 {
    let f = factorial_u(n);
    if f.is_finite() {
        mul_pow(f * c, x, s)
    } else {
        (gammaln(f64::from(n) + 1.0) - s * x.ln() + c.ln()).exp()
    }
}
