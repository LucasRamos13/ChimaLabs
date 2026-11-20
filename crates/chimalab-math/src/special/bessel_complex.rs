//! Funções de Bessel J, Y, I, K e Hankel de ordem real e argumento complexo (ramo principal,
//! −π < ph z ≤ π; sobre o corte z = −x + 0i, o lado de cima; com Im z = −0, também).
//!
//! Núcleo, para Re z ≥ 0 e ν ≥ 0, em forma escalada (sem estouro):
//! - e^{−Re z}·I_ν(z): série ascendente (DLMF 10.25.2) para |z|²/4 ≤ ν + 1; recorrência de Miller
//!   descendente, normalizada pela expansão de Gegenbauer
//!   e^z = Γ(α)(z/2)^{−α} Σₖ (α+k) Γ(2α+k)/(k! Γ(2α)) I_{α+k}(z) (DLMF 10.23.8 em θ = 0);
//!   expansão assintótica com os dois exponenciais (DLMF 10.40.5) para |z| ≥ 20 + ν²/2.
//! - e^{z}·K_ν(z): K_α e K_{α+1} com |α| ≤ ½ e recorrência ascendente (DLMF 10.29.1), estável
//!   para K. Para |z| ≥ 21, expansão assintótica (DLMF 10.40.2); para |z| ≤ 2, série de Temme (N. M. Temme, J. Comput. Phys. 19, 1975, 324–337),
//!   uniforme em α inclusive em α = 0; e a integral DLMF 10.32.8 com t = 1 + v/z,
//!   K_ν(z) = √π 2^{−ν}/Γ(ν+½)·e^{−z} z^{−½} ∫₀^∞ e^{−v} v^{ν−½} (2 + v/z)^{ν−½} dv
//!   (no meio, 2 < |z| < 21),
//!   que decai como e^{−v} em todo o semiplano Re z ≥ 0 (sem oscilar), por quadratura DE.
//!
//! Demais casos:
//! - J_ν(z) = e^{iνπ/2} I_ν(−iz) e H⁽¹⁾_ν(z) = (2/(πi)) e^{−iνπ/2} K_ν(−iz) para Im z ≥ 0
//!   (DLMF 10.27.6, 10.27.8); Y = (H⁽¹⁾ − J)/i e H⁽²⁾ = 2J − H⁽¹⁾; conjugação para Im z < 0
//!   (DLMF 10.11.9).
//! - I e K com Re z < 0: continuação DLMF 10.34.1–2.
//! - Ordens negativas: DLMF 10.4.6–7 (J, Y, H) e 10.27.2–3 (I, K).
//! - No eixo real com Re z ≥ 0, devolve exatamente as funções reais.
//! - Airy (Ai, Bi, Ai′, Bi′): série de Maclaurin (DLMF 9.4) para |z| ≤ 1; para Re z ≥ 0,
//!   Ai = √(z/3) K_{1/3}(ζ)/π, Ai′ = −z K_{2/3}(ζ)/(π√3), Bi = √(z/3)(2I_{1/3}(ζ) + (2/π)
//!   sen(π/3) K_{1/3}(ζ)), Bi′ = (z/√3)(2I_{2/3}(ζ) + (2/π) sen(2π/3) K_{2/3}(ζ)), ζ = (2/3)z^{3/2}
//!   (DLMF 9.6.1–3 com 10.27.2); para Re z < 0, com w = −z e ξ = (2/3)w^{3/2},
//!   Ai = (√w/2)(J_{1/3} − Y_{1/3}/√3), Bi = −(√w/2)(J_{1/3}/√3 + Y_{1/3}),
//!   Ai′ = (w/2)(J_{2/3} + Y_{2/3}/√3), Bi′ = (w/2)(J_{2/3}/√3 − Y_{2/3}) (DLMF 9.6.6–9 com
//!   10.4.7). Os dois setores ficam dentro de |ph| < 2π/3, onde ζ^{1/3} e √z combinam sem
//!   cruzar cortes. No eixo real, devolve as funções reais.
//! - Ordem ou argumento não finitos (fora desse semieixo) dão NaN. Quando o valor transborda
//!   (por exemplo |J₀(iy)| = I₀(y) com y > 713), as partes saem ±Inf ou NaN: Inf·0 = NaN na
//!   multiplicação complexa.

use std::f64::consts::{FRAC_2_PI, PI};

use num_complex::Complex64;

use super::bessel::{
    AIRY_C1, AIRY_C2, HankelKind, airy, airy_prime, besselh, besseli, besselj, besselk, bessely,
};
use super::elementary::{frexp, ldexp};
use super::gamma::{cospi, gamma, sinpi, zeta_table};
use super::quad::exp_sinh_complex;

/// Termos da série de I, usada quando |z|²/4 ≤ ν + 1: a razão entre termos é
/// (|z|²/4)/(k(ν+k)) ≤ 1/k, e o termo 20 é < 1/20! < 5e-19.
const I_SERIES_TERMS: usize = 20;
/// |z| até o qual K usa a série de Temme.
const TEMME_RADIUS: f64 = 2.0;
/// Termos da série de Temme: os termos decrescem como 1/(k!)², < 1e-40 em k = 30.
const TEMME_TERMS: usize = 30;
/// Máximo de termos da expansão assintótica de I.
const ASYM_MAX_TERMS: usize = 80;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

/// e^{iπt} = cos πt + i sen πt, exato nos múltiplos de ½.
fn cis_pi(t: f64) -> Complex64 {
    c(cospi(t), sinpi(t))
}

/// Limiar da expansão assintótica de I: o primeiro termo omitido decresce enquanto ν² ≲ |z|.
/// Subir o limiar só troca a assintótica pela recorrência de Miller, também exata ali (mutante
/// equivalente registrado em `.cargo/mutants.toml`).
fn asymptotic_radius(nu: f64) -> f64 {
    20.0 + nu * nu / 2.0
}

/// Ordem negativa: reflexões DLMF 10.4.6–7. Em ν = 0 as reflexões são a identidade (cos 0 = 1,
/// sen 0 = 0), de modo que `<` ou `<=` dão o mesmo valor (mutante equivalente registrado).
fn negative_order(nu: f64) -> bool {
    nu < 0.0
}

// ---------------------------------------------------------------------------------------------
// I escalada

/// e^{−Re z} I_ν(z) para Re z ≥ 0, ν ≥ 0.
fn i_scaled(nu: f64, z: Complex64) -> Complex64 {
    let r = z.norm();
    // Série quando |z|²/4 ≤ ν + 1: sem cancelamento relevante (Σ|termos|/|soma| ≤ e²) e sem o
    // caminho longo da recorrência de Miller, que acumula ~nε para ν ≫ |z|.
    if r * r <= 4.0 * (nu + 1.0) {
        return i_series(nu, z) * (-z.re).exp();
    }
    if r >= asymptotic_radius(nu) {
        return i_asymptotic_scaled(nu, z);
    }
    i_miller_scaled(nu, z)
}

/// z^p no ramo principal, com |z|^p pela potência real e fase p·arg z. A forma exp(p ln z) erra
/// ~|p ln|z||·ε no módulo: 1e-13 em |z| = 1e-200.
fn polar_pow(z: Complex64, p: f64) -> Complex64 {
    Complex64::from_polar(z.norm().powf(p), p * z.arg())
}

/// (z/2)^ν / Γ(ν+1), com o módulo |z/2|^ν/Γ(ν+1) por potência real quando Γ(ν+1) é finita e,
/// senão (ν > 170), pelo produto |z/2|^α/Γ(α+1)·Π_{k=1}^{n} |z/2|/(α+k), ν = α + n, normalizado
/// por `frexp` a cada passo (exato, sem estouro nem subestouro intermediários).
fn power_over_gamma(nu: f64, z: Complex64) -> Complex64 {
    let h = 0.5 * z;
    let r = h.norm();
    let g = gamma(nu + 1.0);
    let modulus = if g.is_finite() {
        r.powf(nu) / g
    } else {
        let n = nu.floor();
        let a = nu - n;
        let mut m = r.powf(a) / gamma(a + 1.0);
        let mut e2 = 0;
        let mut k = 1.0;
        while k <= n {
            let (mant, e) = frexp(m * (r / (a + k)));
            m = mant;
            e2 += e;
            k += 1.0;
        }
        ldexp(m, e2)
    };
    Complex64::from_polar(modulus, nu * h.arg())
}

/// I_ν(z) = (z/2)^ν Σ (z²/4)ᵏ / (k! Γ(ν+k+1)) (DLMF 10.25.2), |z|²/4 ≤ ν + 1.
fn i_series(nu: f64, z: Complex64) -> Complex64 {
    let q = 0.25 * z * z;
    let mut term = c(1.0, 0.0);
    let mut sum = term;
    for k in 1..I_SERIES_TERMS {
        let kk = k as f64;
        term *= q / (kk * (nu + kk));
        sum += term;
    }
    power_over_gamma(nu, z) * sum
}

/// aₖ(ν) = ((4ν² − 1²)(4ν² − 3²)…(4ν² − (2k−1)²)) / (k! 8ᵏ) (DLMF 10.17.1), dividido por zᵏ;
/// devolve (Σ aₖ/zᵏ, Σ (−1)ᵏ aₖ/zᵏ) até o menor termo.
fn hankel_sums(nu: f64, z: Complex64) -> (Complex64, Complex64) {
    let mu = 4.0 * nu * nu;
    let inv = z.finv();
    let mut term = c(1.0, 0.0);
    let (mut plus, mut minus) = (term, term);
    let mut last = f64::INFINITY;
    for k in 1..ASYM_MAX_TERMS {
        let kk = k as f64;
        let odd = 2.0 * kk - 1.0;
        term *= inv * ((mu - odd * odd) / (8.0 * kk));
        let size = term.norm();
        if size >= last {
            break;
        }
        last = size;
        plus += term;
        minus += if k % 2 == 0 { term } else { -term };
    }
    (plus, minus)
}

/// e^{−Re z} I_ν(z) pela expansão assintótica (DLMF 10.40.5), Re z ≥ 0:
/// I_ν(z) ≈ e^z/√(2πz) Σ (−1)ᵏaₖ/zᵏ ± i e^{±iνπ} e^{−z}/√(2πz) Σ aₖ/zᵏ, sinal de Im z.
fn i_asymptotic_scaled(nu: f64, z: Complex64) -> Complex64 {
    let (plus, minus) = hankel_sums(nu, z);
    let root = (2.0 * PI * z).sqrt();
    let first = c(0.0, z.im).exp() * minus / root;
    let s = if z.im >= 0.0 { 1.0 } else { -1.0 };
    // e^{−z − Re z} = e^{−2 Re z − i Im z}
    let second = c(0.0, s) * cis_pi(s * nu) * c(-2.0 * z.re, -z.im).exp() * plus / root;
    first + second
}

/// Divide os valores da recorrência de Miller por 1e250 (sem mudar as razões entre eles).
fn shrink(values: &mut [Complex64]) {
    for v in values {
        *v *= 1e-250;
    }
}

/// Índice inicial da recorrência de Miller: acima de max(n, |z|), a largura de transição de
/// I_k(|z|) é ~|z|^{1/3}, e 30 + 8|z|^{1/3} passos a mais tornam I_top/I_n < 1e-17. Aumentá-lo só
/// custa tempo (mutantes `+` → `*` equivalentes, registrados em `.cargo/mutants.toml`).
fn miller_start(n: usize, r: f64) -> usize {
    n.max(r.ceil() as usize) + 30 + (8.0 * r.cbrt()).ceil() as usize
}

/// Recorrência de Miller descendente para I_{α+k}, normalizada pela soma de Gegenbauer.
fn i_miller_scaled(nu: f64, z: Complex64) -> Complex64 {
    let n = nu.floor();
    let alpha = nu - n;
    let n = n as usize;
    let r = z.norm();
    let top = miller_start(n, r);
    // d̂ₖ = dₖ/Γ(α+1): d̂₀ = 1, d̂₁ = 2(α+1), d̂ₖ = d̂ₖ₋₁ (α+k)(2α+k−1) / ((α+k−1)k).
    let mut d = vec![0.0; top + 1]; // top ≥ 30
    d[0] = 1.0;
    d[1] = 2.0 * (alpha + 1.0);
    for k in 2..=top {
        let kk = k as f64;
        d[k] = d[k - 1] * (alpha + kk) * (2.0 * alpha + kk - 1.0) / ((alpha + kk - 1.0) * kk);
    }
    let zi = z.finv();
    let mut next = c(0.0, 0.0); // p_{k+1}
    let mut cur = c(1e-30, 0.0); // p_k, começando em k = top
    // A soma começa sem o termo p_top·d̂_top, desprezível por construção (I_top/I_n < 1e-17);
    // top ≥ n + 30, então p_n é sempre atribuído dentro do laço.
    let mut sum = c(0.0, 0.0);
    let mut at_n = c(0.0, 0.0);
    for k in (1..=top).rev() {
        let prev = cur * (2.0 * (alpha + k as f64)) * zi + next;
        next = cur;
        cur = prev;
        sum += cur * d[k - 1];
        if k - 1 == n {
            at_n = cur;
        }
        if cur.norm() > 1e250 {
            // Todos juntos: a razão p_n/Σ não muda.
            let mut state = [cur, next, sum, at_n];
            shrink(&mut state);
            [cur, next, sum, at_n] = state;
        }
    }
    // e^{−Re z} I_{α+n} = p_n · e^{i Im z} (z/2)^α / (Γ(α+1) Σ d̂ₖ pₖ)
    let pow = polar_pow(0.5 * z, alpha);
    // `finv`: a soma chega a 1e250 e a divisão comum calcularia |soma|², que transborda.
    at_n * c(0.0, z.im).exp() * pow * (sum * gamma(alpha + 1.0)).finv()
}

// ---------------------------------------------------------------------------------------------
// K escalada

/// e^{z} K_ν(z) para Re z ≥ 0, z ≠ 0, ν ≥ 0.
fn k_scaled(nu: f64, z: Complex64) -> Complex64 {
    let floor = nu.floor();
    let (alpha, n) = if nu - floor > 0.5 {
        (nu - floor - 1.0, floor + 1.0)
    } else {
        (nu - floor, floor)
    };
    let r = z.norm();
    let (k0, k1) = if r <= TEMME_RADIUS {
        temme_scaled(alpha, z)
    } else if r >= asymptotic_radius(1.5) {
        (
            k_asymptotic_scaled(alpha.abs(), z),
            k_asymptotic_scaled(alpha + 1.0, z),
        )
    } else {
        (
            k_integral_scaled(alpha.abs(), z),
            k_integral_scaled(alpha + 1.0, z),
        )
    };
    if n == 0.0 {
        return k0;
    }
    // K_{μ+1} = K_{μ−1} + (2μ/z) K_μ, de μ = α + 1 até ν − 1.
    let zi = z.finv();
    let (mut prev, mut cur) = (k0, k1);
    let mut m = 1.0;
    while m < n {
        let next = prev + cur * (2.0 * (alpha + m)) * zi;
        prev = cur;
        cur = next;
        m += 1.0;
    }
    cur
}

/// e^{z} K_ν(z) ≈ √(π/(2z)) Σ aₖ(ν)/zᵏ (DLMF 10.40.2), ν ∈ [0, 3/2], |z| ≥ 21.
fn k_asymptotic_scaled(nu: f64, z: Complex64) -> Complex64 {
    (PI / (2.0 * z)).sqrt() * hankel_sums(nu, z).0
}

/// e^{z} K_ν(z) pela integral (ver o cabeçalho), ν ∈ [0, 3/2], 2 < |z| < 21.
fn k_integral_scaled(nu: f64, z: Complex64) -> Complex64 {
    let e = nu - 0.5;
    let zi = z.finv();
    let integral = exp_sinh_complex(|v| (2.0 + v * zi).powf(e) * ((-v).exp() * v.powf(e)), 1e-15, 0.0);
    PI.sqrt() * 2f64.powf(-nu) / gamma(nu + 0.5) * z.sqrt().finv() * integral
}

/// 1/Γ(1+α), 1/Γ(1−α), Γ₁(α) = (1/Γ(1−α) − 1/Γ(1+α))/(2α) e Γ₂(α) = (1/Γ(1−α) + 1/Γ(1+α))/2,
/// para |α| ≤ ½. Γ₁ sem cancelamento: com L(x) = ln Γ(1+x) = −γx + Σ (−1)ᵏ ζ(k) xᵏ/k
/// (DLMF 5.7.3), L(α) − L(−α) = 2αg, g = −γ − Σ_{k ímpar ≥ 3} ζ(k) α^{k−1}/k, e
/// Γ₁ = (1/Γ(1+α))·g·expm1(2αg)/(2αg).
fn temme_gammas(alpha: f64) -> (f64, f64, f64, f64) {
    let zt = zeta_table();
    let mut g = -std::f64::consts::EULER_GAMMA;
    let a2 = alpha * alpha;
    let mut p = a2; // α^{k−1}, k = 3
    for k in (3..zt.len()).step_by(2) {
        g -= zt[k] * p / k as f64;
        p *= a2;
    }
    let rp = 1.0 / gamma(1.0 + alpha);
    let rm = 1.0 / gamma(1.0 - alpha);
    let d = 2.0 * alpha * g;
    let ratio = if d == 0.0 { 1.0 } else { d.exp_m1() / d };
    (rp, rm, rp * g * ratio, 0.5 * (rm + rp))
}

/// sinh(σ)/σ sem perder dígitos para σ pequeno; `up` = e^{σ} e `down` = e^{−σ}.
fn sinhc(s: Complex64, up: Complex64, down: Complex64) -> Complex64 {
    if s.norm() > 1.0 {
        return 0.5 * (up - down) / s;
    }
    // Σ σ^{2k}/(2k+1)!: com |σ| ≤ 1, o termo k = 10 é < 2e-20.
    let s2 = s * s;
    let mut term = c(1.0, 0.0);
    let mut sum = term;
    for k in 1..=10 {
        let kk = k as f64;
        term *= s2 / ((2.0 * kk) * (2.0 * kk + 1.0));
        sum += term;
    }
    sum
}

/// (e^{z}K_α(z), e^{z}K_{α+1}(z)) pela série de Temme, |α| ≤ ½, |z| ≤ 2.
fn temme_scaled(alpha: f64, z: Complex64) -> (Complex64, Complex64) {
    let (rp, rm, g1, g2) = temme_gammas(alpha);
    let ln2z = -(0.5 * z).ln(); // ln(2/z)
    let sigma = alpha * ln2z;
    // e^{±σ} = (z/2)^{∓α} por potência polar: com |z| minúsculo, |σ| chega a 345 e e^σ por exp
    // perderia ~|σ|ε.
    let up = polar_pow(0.5 * z, -alpha); // (z/2)^{−α} = e^{σ}
    let down = polar_pow(0.5 * z, alpha); // e^{−σ}
    let cosh = 0.5 * (up + down);
    let frac = if alpha == 0.0 {
        1.0
    } else {
        alpha * PI / sinpi(alpha)
    };
    let mut f = frac * (cosh * g1 + sinhc(sigma, up, down) * ln2z * g2);
    // p₀ = ½(z/2)^{−α}Γ(1+α), q₀ = ½(z/2)^{α}Γ(1−α)
    let mut p = 0.5 * up / rp;
    let mut q = 0.5 * down / rm;
    let q4 = 0.25 * z * z;
    let mut ck = c(1.0, 0.0);
    let mut sum0 = f;
    let mut sum1 = p;
    for k in 1..TEMME_TERMS {
        let kk = k as f64;
        f = (kk * f + p + q) / (kk * kk - alpha * alpha);
        p /= kk - alpha;
        q /= kk + alpha;
        ck *= q4 / kk;
        sum0 += ck * f;
        sum1 += ck * (p - kk * f);
    }
    let ez = z.exp();
    (sum0 * ez, 2.0 * sum1 * z.finv() * ez)
}

// ---------------------------------------------------------------------------------------------
// J, Y, H no semiplano superior (ν ≥ 0, Im z ≥ 0, z ≠ 0)

/// (J, Y, e^{−iz}H⁽¹⁾, e^{iz}H⁽²⁾) para ν ≥ 0 e Im z ≥ 0.
fn jyh_upper(nu: f64, z: Complex64) -> (Complex64, Complex64, Complex64, Complex64) {
    let zeta = c(z.im, -z.re); // −iz, Re ζ = Im z ≥ 0
    let is = i_scaled(nu, zeta);
    let ks = k_scaled(nu, zeta);
    let j = j_upper(nu, z);
    let h1s = c(0.0, -FRAC_2_PI) * cis_pi(-0.5 * nu) * ks;
    let eiz = c(-z.im, z.re).exp();
    let h1 = h1s * eiz;
    let y = c(0.0, -1.0) * (h1 - j);
    // e^{iz}H⁽²⁾ = 2e^{iz}J − e^{2iz}(e^{−iz}H⁽¹⁾), com e^{iz}e^{Im z} = e^{i Re z}
    let h2s = 2.0 * cis_pi(0.5 * nu) * c(0.0, z.re).exp() * is - eiz * eiz * h1s;
    (j, y, h1s, h2s)
}

/// (J, Y, e^{−iz}H⁽¹⁾, e^{iz}H⁽²⁾) para ν ≥ 0, qualquer z ≠ 0.
fn jyh_positive(nu: f64, z: Complex64) -> (Complex64, Complex64, Complex64, Complex64) {
    if z.im < 0.0 {
        // J(z̄) = conj J(z), Y idem, H⁽¹⁾(z̄) = conj H⁽²⁾(z); as escalas trocam do mesmo modo.
        let (j, y, h1s, h2s) = jyh_upper(nu, z.conj());
        return (j.conj(), y.conj(), h2s.conj(), h1s.conj());
    }
    jyh_upper(nu, z)
}

/// (J, Y, e^{−iz}H⁽¹⁾, e^{iz}H⁽²⁾) para ν real qualquer.
fn jyh(nu: f64, z: Complex64) -> (Complex64, Complex64, Complex64, Complex64) {
    if negative_order(nu) {
        let m = -nu;
        let (j, y, h1s, h2s) = jyh_positive(m, z);
        let (s, co) = (sinpi(m), cospi(m));
        // DLMF 10.4.6–7: J_{−ν} = cos νπ J_ν − sen νπ Y_ν, Y_{−ν} = sen νπ J_ν + cos νπ Y_ν,
        // H⁽¹⁾_{−ν} = e^{iνπ}H⁽¹⁾_ν, H⁽²⁾_{−ν} = e^{−iνπ}H⁽²⁾_ν.
        return (co * j - s * y, s * j + co * y, cis_pi(m) * h1s, cis_pi(-m) * h2s);
    }
    jyh_positive(nu, z)
}

fn on_nonnegative_axis(z: Complex64) -> bool {
    z.im == 0.0 && z.re >= 0.0
}

/// Ordem ou argumento não finitos fora do semieixo real não negativo (que vai para as funções
/// reais): o resultado é NaN. Sem esta guarda o resultado também seria NaN, mas a quadratura de K
/// rodaria todos os níveis sem convergir (~25 ms); por isso é excluída da mutação.
fn non_finite(nu: f64, z: Complex64) -> bool {
    !(nu.is_finite() && z.re.is_finite() && z.im.is_finite())
}

const NAN: Complex64 = Complex64::new(f64::NAN, f64::NAN);

/// J_ν(z) = e^{iνπ/2} e^{Im z}·(e^{−Re ζ} I_ν(ζ)), ζ = −iz, para ν ≥ 0 e Im z ≥ 0: não precisa de K.
fn j_upper(nu: f64, z: Complex64) -> Complex64 {
    cis_pi(0.5 * nu) * z.im.exp() * i_scaled(nu, c(z.im, -z.re))
}

/// J_ν(z) com argumento complexo. No semieixo real não negativo coincide com [`besselj`].
pub fn besselj_complex(nu: f64, z: Complex64) -> Complex64 {
    if on_nonnegative_axis(z) {
        return c(besselj(nu, z.re), 0.0);
    }
    if non_finite(nu, z) {
        return NAN;
    }
    if negative_order(nu) {
        return jyh(nu, z).0;
    }
    if z.im < 0.0 {
        return j_upper(nu, z.conj()).conj();
    }
    j_upper(nu, z)
}

/// Y_ν(z) com argumento complexo. No semieixo real não negativo coincide com [`bessely`].
pub fn bessely_complex(nu: f64, z: Complex64) -> Complex64 {
    if on_nonnegative_axis(z) {
        return c(bessely(nu, z.re), 0.0);
    }
    if non_finite(nu, z) {
        return NAN;
    }
    jyh(nu, z).1
}

/// H⁽¹⁾_ν(z) ou H⁽²⁾_ν(z) com argumento complexo; com `scaled`, H⁽¹⁾·e^{−iz} ou H⁽²⁾·e^{iz}.
/// No semieixo real não negativo coincide com [`besselh`].
pub fn besselh_complex(nu: f64, kind: HankelKind, z: Complex64, scaled: bool) -> Complex64 {
    if on_nonnegative_axis(z) {
        return besselh(nu, kind, z.re, scaled);
    }
    if non_finite(nu, z) {
        return NAN;
    }
    let (_, _, h1s, h2s) = jyh(nu, z);
    match (kind, scaled) {
        (HankelKind::First, true) => h1s,
        (HankelKind::Second, true) => h2s,
        (HankelKind::First, false) => h1s * c(-z.im, z.re).exp(),
        (HankelKind::Second, false) => h2s * c(z.im, -z.re).exp(),
    }
}

// ---------------------------------------------------------------------------------------------
// I, K em todo o plano

/// (e^{−|Re z|} I_ν(z), e^{z} K_ν(z)) para ν ≥ 0, z ≠ 0.
fn ik_positive(nu: f64, z: Complex64) -> (Complex64, Complex64) {
    if z.re >= 0.0 {
        return (i_scaled(nu, z), k_scaled(nu, z));
    }
    if z.im < 0.0 {
        let (i, k) = ik_positive(nu, z.conj());
        return (i.conj(), k.conj());
    }
    // z = w e^{iπ}, Re w > 0 (DLMF 10.34.1–2): I(z) = e^{iνπ}I(w),
    // K(z) = e^{−iνπ}K(w) − iπ I(w). Escalas: e^{−|Re z|} = e^{−Re w}, e^{z} = e^{−w}.
    let w = -z;
    let (is, ks) = (i_scaled(nu, w), k_scaled(nu, w));
    let i = cis_pi(nu) * is;
    // e^{z}K(z) = e^{−iνπ} e^{−2w} (e^{w}K(w)) − iπ e^{−i Im w} (e^{−Re w} I(w))
    let k = cis_pi(-nu) * (-2.0 * w).exp() * ks - c(0.0, PI) * c(0.0, -w.im).exp() * is;
    (i, k)
}

/// (e^{−|Re z|} I_ν(z), e^{z} K_ν(z)) para ν real qualquer.
fn ik(nu: f64, z: Complex64) -> (Complex64, Complex64) {
    let (is, ks) = ik_positive(nu.abs(), z);
    if nu >= 0.0 {
        return (is, ks);
    }
    // I_{−ν} = I_ν + (2/π) sen(νπ) K_ν (DLMF 10.27.2); K_{−ν} = K_ν. Na escala de I:
    // e^{−|Re z|}K = e^{−|Re z| − z}(e^{z}K).
    let m = -nu;
    let shift = c(-z.re.abs() - z.re, -z.im).exp();
    (is + FRAC_2_PI * sinpi(m) * shift * ks, ks)
}

/// I_ν(z) com argumento complexo; com `scaled`, e^{−|Re z|}·I_ν(z).
/// No semieixo real não negativo coincide com [`besseli`].
pub fn besseli_complex(nu: f64, z: Complex64, scaled: bool) -> Complex64 {
    if on_nonnegative_axis(z) {
        return c(besseli(nu, z.re, scaled), 0.0);
    }
    if non_finite(nu, z) {
        return NAN;
    }
    let is = ik(nu, z).0;
    if scaled { is } else { is * z.re.abs().exp() }
}

/// K_ν(z) com argumento complexo; com `scaled`, e^{z}·K_ν(z).
/// No semieixo real não negativo coincide com [`besselk`].
pub fn besselk_complex(nu: f64, z: Complex64, scaled: bool) -> Complex64 {
    if on_nonnegative_axis(z) {
        return c(besselk(nu, z.re, scaled), 0.0);
    }
    if non_finite(nu, z) {
        return NAN;
    }
    let ks = ik(nu, z).1;
    if scaled { ks } else { ks * (-z).exp() }
}

// ---------------------------------------------------------------------------------------------
// Airy

/// Termos da série de Maclaurin para |z| ≤ 1: o k-ésimo termo é ≤ Π 1/((3j−1)3j) < 1e-20 em
/// k = 12.
const AIRY_TERMS: usize = 14;

/// (Ai, Bi, Ai′, Bi′) pela série de Maclaurin (DLMF 9.4.1–2), |z| ≤ 1.
fn airy_maclaurin(z: Complex64) -> [Complex64; 4] {
    let z3 = z * z * z;
    // f = Σ aₖz^{3k}, g = Σ bₖz^{3k+1}; f′ = Σ uₖ (u₁ = z²/2), g′ = Σ vₖ (v₀ = 1).
    let (mut f, mut g) = (c(1.0, 0.0), z);
    let (mut sf, mut sg) = (f, g);
    let (mut u, mut v) = (0.5 * z * z, c(1.0, 0.0));
    let (mut su, mut sv) = (u, v);
    for k in 1..AIRY_TERMS {
        let kk = k as f64;
        f *= z3 / ((3.0 * kk - 1.0) * (3.0 * kk));
        g *= z3 / ((3.0 * kk) * (3.0 * kk + 1.0));
        u *= z3 / ((3.0 * kk + 2.0) * (3.0 * kk));
        v *= z3 / ((3.0 * kk) * (3.0 * kk - 2.0));
        sf += f;
        sg += g;
        su += u;
        sv += v;
    }
    let s3 = 3f64.sqrt();
    [
        AIRY_C1 * sf - AIRY_C2 * sg,
        s3 * (AIRY_C1 * sf + AIRY_C2 * sg),
        AIRY_C1 * su - AIRY_C2 * sv,
        s3 * (AIRY_C1 * su + AIRY_C2 * sv),
    ]
}

/// (Ai, Bi, Ai′, Bi′) para |z| > 1 fora do eixo real (ver o cabeçalho do módulo).
fn airy_bessel(z: Complex64) -> [Complex64; 4] {
    let s3 = 3f64.sqrt();
    if z.re >= 0.0 {
        let zeta = (2.0 / 3.0) * polar_pow(z, 1.5);
        let (is1, ks1) = ik(1.0 / 3.0, zeta);
        let (is2, ks2) = ik(2.0 / 3.0, zeta);
        // K = e^{−ζ}·Ks, I = e^{|Re ζ|}·Is
        let (ek, ei) = ((-zeta).exp(), zeta.re.abs().exp());
        let (k1, k2) = (ks1 * ek, ks2 * ek);
        let (i1, i2) = (is1 * ei, is2 * ei);
        let root = (z / 3.0).sqrt();
        let sin_third = 0.5 * s3; // sen(π/3) = sen(2π/3)
        return [
            root * k1 / PI,
            root * (2.0 * i1 + FRAC_2_PI * sin_third * k1),
            -z * k2 / (PI * s3),
            z / s3 * (2.0 * i2 + FRAC_2_PI * sin_third * k2),
        ];
    }
    let w = -z;
    let xi = (2.0 / 3.0) * polar_pow(w, 1.5);
    let (j1, y1, _, _) = jyh(1.0 / 3.0, xi);
    let (j2, y2, _, _) = jyh(2.0 / 3.0, xi);
    let h = 0.5 * w.sqrt();
    let hw = 0.5 * w;
    [
        h * (j1 - y1 / s3),
        -h * (j1 / s3 + y1),
        hw * (j2 + y2 / s3),
        hw * (j2 / s3 - y2),
    ]
}

/// (Ai, Bi, Ai′, Bi′) com argumento complexo.
fn airy_all(z: Complex64) -> [Complex64; 4] {
    if z.norm() <= 1.0 {
        return airy_maclaurin(z);
    }
    airy_bessel(z)
}

/// Funções de Airy (Ai(z), Bi(z)) com argumento complexo. No eixo real coincide com [`airy`].
/// Argumento não finito fora do eixo real dá NaN.
pub fn airy_complex(z: Complex64) -> (Complex64, Complex64) {
    if z.im == 0.0 {
        let (a, b) = airy(z.re);
        return (c(a, 0.0), c(b, 0.0));
    }
    if non_finite(0.0, z) {
        return (NAN, NAN);
    }
    let [a, b, _, _] = airy_all(z);
    (a, b)
}

/// Derivadas (Ai′(z), Bi′(z)) com argumento complexo. No eixo real coincide com [`airy_prime`].
/// Argumento não finito fora do eixo real dá NaN.
pub fn airy_prime_complex(z: Complex64) -> (Complex64, Complex64) {
    if z.im == 0.0 {
        let (a, b) = airy_prime(z.re);
        return (c(a, 0.0), c(b, 0.0));
    }
    if non_finite(0.0, z) {
        return (NAN, NAN);
    }
    let [_, _, a, b] = airy_all(z);
    (a, b)
}
