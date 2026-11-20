//! Função gama e relacionadas.
//!
//! - Γ: série de Stirling com coeficientes de Bernoulli (DLMF 5.11.1) após deslocamento para
//!   z ≥ 15 pela recorrência Γ(z+1) = zΓ(z) (DLMF 5.5.1); reflexão (DLMF 5.5.3) para x < 1/2;
//!   valores exatos para inteiros até 22.
//! - ln Γ: Stirling em log; perto das raízes 1 e 2, série de Taylor com ζ(k) (DLMF 5.7.3).
//! - ψ: série assintótica (DLMF 5.11.2) após deslocamento; reflexão (DLMF 5.5.4).
//! - Gama incompleta regularizada: série (DLMF 8.11.4) ou fração contínua (DLMF 8.9.2).
//! - Beta incompleta regularizada: fração contínua (DLMF 8.17.22) com simetria (DLMF 8.17.4).

use std::f64::consts::PI;
use std::sync::OnceLock;

use super::dd::{self, Dd};
use super::erf::erfc;
use super::{lentz, lentz_n};

const EULER_GAMMA: f64 = std::f64::consts::EULER_GAMMA;
pub(crate) const LN_SQRT_2PI: f64 = 0.918_938_533_204_672_8;
/// Números de Bernoulli B₂, B₄, …, B₂₀ como frações exatas (numerador, denominador).
const BERNOULLI: [(f64, f64); 10] = [
    (1.0, 6.0),
    (-1.0, 30.0),
    (1.0, 42.0),
    (-1.0, 30.0),
    (5.0, 66.0),
    (-691.0, 2730.0),
    (7.0, 6.0),
    (-3617.0, 510.0),
    (43867.0, 798.0),
    (-174611.0, 330.0),
];

pub(crate) fn bernoulli(k: usize) -> f64 {
    let (n, d) = BERNOULLI[k - 1];
    n / d
}

/// Correção de Stirling: Σ B₂ₖ / (2k(2k−1) z^(2k−1)), para z ≥ 15.
fn stirling_correction(z: f64) -> f64 {
    let z2 = z * z;
    let mut zp = z;
    let mut s = 0.0;
    for k in 1..=9 {
        let kk = k as f64;
        s += bernoulli(k) / (2.0 * kk * (2.0 * kk - 1.0) * zp);
        zp *= z2;
    }
    s
}

/// sin(πx) com redução exata do argumento.
pub(crate) fn sinpi(x: f64) -> f64 {
    let n = x.round();
    let r = x - n;
    let s = (PI * r).sin();
    if (n as i64) % 2 == 0 { s } else { -s }
}

pub(crate) fn cospi(x: f64) -> f64 {
    let n = x.round();
    let r = x - n;
    // cos(πr) = sin(π(½ − |r|)): exato (zero) em meio-inteiros e com erro relativo pequeno perto
    // deles, onde cos(π·r) perderia dígitos.
    let c = (PI * (0.5 - r.abs())).sin();
    if (n as i64) % 2 == 0 { c } else { -c }
}

fn factorial_table() -> &'static [f64; 23] {
    static T: OnceLock<[f64; 23]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [1.0; 23];
        let mut acc: u128 = 1;
        for (n, v) in t.iter_mut().enumerate().skip(1) {
            acc *= n as u128;
            *v = acc as f64;
        }
        t
    })
}

/// Γ(z) para z ≥ 15 por Stirling, sem transbordar em passos intermediários.
fn gamma_stirling(z: f64) -> f64 {
    if z > 171.7 {
        return f64::INFINITY;
    }
    let half = z.powf(0.5 * z - 0.25);
    (2.0 * PI).sqrt() * (half * (-z).exp()) * half * stirling_correction(z).exp()
}

/// Função gama. Polos: Γ(±0) = ±Inf; inteiros negativos → NaN; Γ(+Inf) = Inf; Γ(−Inf) = NaN.
pub fn gamma(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::INFINITY.copysign(x);
    }
    if x.fract() == 0.0 {
        if x < 0.0 {
            return f64::NAN;
        }
        if x <= 23.0 {
            return factorial_table()[x as usize - 1];
        }
    }
    if x < 0.5 {
        let s = sinpi(x);
        return PI / (s * gamma(1.0 - x));
    }
    if x >= 15.0 {
        return gamma_stirling(x);
    }
    let m = (15.0 - x).ceil();
    let mut prod = 1.0;
    let mut k = 0.0;
    while k < m {
        prod *= x + k;
        k += 1.0;
    }
    gamma_stirling(x + m) / prod
}

/// ζ(k) para k = 2..64 por Euler-Maclaurin (N = 10, 8 termos de correção).
pub(crate) fn zeta_table() -> &'static [f64; 65] {
    static T: OnceLock<[f64; 65]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0.0; 65];
        let n = 10.0f64;
        for (k, slot) in t.iter_mut().enumerate().skip(2) {
            let s = k as f64;
            let mut z: f64 = (1..10).map(|i| (i as f64).powf(-s)).sum();
            z += n.powf(1.0 - s) / (s - 1.0) + 0.5 * n.powf(-s);
            // Σ B₂ⱼ/(2j)! · s(s+1)…(s+2j−2) · N^(−s−2j+1)
            let mut rising = s; // s(s+1)…(s+2j−2), começa com j=1: s
            let mut fact = 2.0; // (2j)!
            for j in 1..=8 {
                z += bernoulli(j) / fact * rising * n.powf(-s - 2.0 * j as f64 + 1.0);
                let jj = j as f64;
                rising *= (s + 2.0 * jj - 1.0) * (s + 2.0 * jj);
                fact *= (2.0 * jj + 1.0) * (2.0 * jj + 2.0);
            }
            *slot = z;
        }
        t
    })
}

/// ln Γ(1 + ε) para |ε| ≤ 0,25 (DLMF 5.7.3).
fn lngamma1p(e: f64) -> f64 {
    let z = zeta_table();
    let mut s = -EULER_GAMMA * e;
    let mut p = -e; // p = (−ε)^k, começando em k = 1
    for (k, zk) in z.iter().enumerate().skip(2) {
        p *= -e;
        let term = zk * p / k as f64;
        s += term;
        if term.abs() < 1e-18 * s.abs().max(1e-300) {
            break;
        }
    }
    s
}

/// ln|Γ(x)|. Polos (x inteiro ≤ 0) → +Inf.
pub fn gammaln(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return f64::INFINITY;
    }
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::INFINITY;
    }
    if x < 0.0 {
        return (PI / sinpi(x).abs()).ln() - gammaln(1.0 - x);
    }
    if x < 0.5 {
        // ln Γ(x) = ln Γ(1 + x) − ln x. A reflexão π/sen(πx) estourava para x subnormal
        // (ln Γ(1e-310) dava Inf em vez de 713,8).
        return gammaln(1.0 + x) - x.ln();
    }
    if (x - 1.0).abs() <= 0.25 {
        return lngamma1p(x - 1.0);
    }
    if (x - 2.0).abs() <= 0.25 {
        return (x - 1.0).ln() + lngamma1p(x - 2.0);
    }
    if x < 15.0 {
        return gamma(x).ln();
    }
    // Para x enorme a fórmula estoura junto com o valor verdadeiro (ln Γ(1,7e308) ≈ 1,2e311): não
    // há ramo assintótico separado.
    (x - 0.5) * x.ln() - x + LN_SQRT_2PI + stirling_correction(x)
}

/// Função digama ψ(x) = Γ'(x)/Γ(x). Polos (inteiros ≤ 0) → NaN.
pub fn psi(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return f64::INFINITY;
    }
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return psi(1.0 - x) - PI * cospi(x) / sinpi(x);
    }
    let mut z = x;
    let mut acc = 0.0;
    while z < 10.0 {
        acc -= 1.0 / z;
        z += 1.0;
    }
    let z2 = z * z;
    let mut zp = z2;
    let mut s = z.ln() - 0.5 / z;
    for k in 1..=8 {
        s -= bernoulli(k) / (2.0 * k as f64 * zp);
        zp *= z2;
    }
    acc + s
}

/// Fatorial n! (Inf acima de 170; exato até 22!). NaN para não inteiros ou negativos.
pub fn factorial(n: f64) -> f64 {
    if n.is_nan() || n < 0.0 || n.fract() != 0.0 {
        return f64::NAN;
    }
    gamma(n + 1.0)
}

/// Função beta B(a, b) para a, b > 0.
pub fn beta(a: f64, b: f64) -> f64 {
    if !(a > 0.0 && b > 0.0) {
        return f64::NAN;
    }
    if a.is_infinite() || b.is_infinite() {
        return 0.0;
    }
    if a + b < 170.0 {
        return gamma(a) * gamma(b) / gamma(a + b);
    }
    let e = ln_beta_large(a, b);
    // e^{hi + lo} = e^{hi}(1 + lo), |lo| ≤ ½ulp(hi)
    e.hi.exp() * (1.0 + e.lo)
}

/// ln B(a, b) para a, b > 0.
pub fn betaln(a: f64, b: f64) -> f64 {
    if !(a > 0.0 && b > 0.0) {
        return f64::NAN;
    }
    if a.is_infinite() || b.is_infinite() {
        return f64::NEG_INFINITY;
    }
    if a + b < 170.0 {
        return beta(a, b).ln();
    }
    ln_beta_large(a, b).to_f64()
}

/// ln B(a, b) em dupla-dupla para a + b ≥ 170, por Stirling com correção:
/// ln Γ(x) = (x − ½) ln x − x + ln √(2π) + corr(x) (DLMF 5.11.1), com os termos grandes em
/// dupla-dupla para que o cancelamento entre eles não custe dígitos (em f64, ln Γ(10⁶) ≈ 1,3e7
/// tem erro absoluto ~1e-9, que vira erro relativo de B).
/// - min(a, b) ≥ 15: (a − ½) ln a + (b − ½) ln b − (c − ½) ln c + ln √(2π) + corr(a) + corr(b)
///   − corr(c), com c = a + b (os termos −a − b + c se anulam);
/// - min(a, b) < 15 (o outro > 155): ln Γ(a) + (b − ½) ln b − (c − ½) ln c + a + corr(b) − corr(c).
fn ln_beta_large(a: f64, b: f64) -> Dd {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let c = Dd::sum(a, b);
    let tb = Dd::sum(b, -0.5).mul(dd::ln(Dd::from(b)));
    let tc = c.add(Dd::from(-0.5)).mul(dd::ln(c));
    if a >= 15.0 {
        let ta = Dd::sum(a, -0.5).mul(dd::ln(Dd::from(a)));
        let corr = stirling_correction(a) + stirling_correction(b) - stirling_correction(c.hi);
        return ta.add(tb).sub(tc).add(dd::LN_SQRT_2PI).add(Dd::from(corr));
    }
    let corr = stirling_correction(b) - stirling_correction(c.hi);
    Dd::from(gammaln(a))
        .add(tb)
        .sub(tc)
        .add(Dd::from(a))
        .add(Dd::from(corr))
}

/// ln(1 + t) − t sem cancelamento: série −t²/2 + t³/3 − … para |t| ≤ ½ (59 termos; o primeiro
/// omitido é < 2e-20 relativo); ln_1p(t) − t fora disso, onde não há cancelamento relevante.
fn log1pmx(t: f64) -> f64 {
    if t.abs() > 0.5 {
        return t.ln_1p() - t;
    }
    let mut p = -t; // (−t)^{k−1}, começando em k = 2
    let mut s = 0.0;
    for k in 2..=60 {
        p *= -t;
        s -= p / k as f64;
    }
    s
}

/// xᵃe^{−x}/Γ(a). Para a ≥ 15, por Stirling: (x/a)ᵃe^{a−x}·√(a/(2π))·e^{−corr(a)}, com
/// (x/a)ᵃe^{a−x} = exp(a·[ln(1+t) − t]), t = (x − a)/a. Assim o expoente não passa pela diferença
/// a·ln x − x − ln Γ(a) entre termos grandes, que errava ~|a·ln x|·ε (2e-10 em a = x = 10⁵).
fn gammainc_prefactor(x: f64, a: f64) -> f64 {
    if a < 15.0 {
        return (a * x.ln() - x - gammaln(a)).exp();
    }
    let t = (x - a) / a;
    (a * log1pmx(t)).exp() * (a / (2.0 * PI)).sqrt() * (-stirling_correction(a)).exp()
}

// Gerado por ferramentas/oraculos/f042_temme.py (racionais exatos; conferido contra as
// formas fechadas em η = ±0,3 e 0,05 a 1e-25).
/// Taylor de c₀(η) em η = 0 (DLMF 8.12.10); c₀(0) = −1/3.
const TEMME_C0: [f64; 28] = [
    -0.3333333333333333,
    0.08333333333333333,
    -0.014814814814814815,
    0.0011574074074074073,
    0.0003527336860670194,
    -0.0001787551440329218,
    3.919263178522438e-05,
    -2.185448510679992e-06,
    -1.85406221071516e-06,
    8.296711340953087e-07,
    -1.7665952736826078e-07,
    6.707853543401498e-09,
    1.0261809784240309e-08,
    -4.382036018453353e-09,
    9.14769958223679e-10,
    -2.5514193994946248e-11,
    -5.830772132550426e-11,
    2.4361948020667415e-11,
    -5.0276692801141755e-12,
    1.1004392031956135e-13,
    3.371763262400985e-13,
    -1.392388722418162e-13,
    2.8534893807047445e-14,
    -5.139111834242572e-16,
    -1.9752288294349442e-15,
    8.099521156704561e-16,
    -1.6522531216398162e-16,
    2.5305430097478883e-18,
];
/// Taylor de c₁(η) em η = 0; c₁(0) = −1/540.
const TEMME_C1: [f64; 26] = [
    -0.001851851851851852,
    -0.003472222222222222,
    0.0026455026455026454,
    -0.0009902263374485596,
    0.00020576131687242798,
    -4.018775720164609e-07,
    -1.8098550334489977e-05,
    7.64916091608111e-06,
    -1.6120900894563446e-06,
    4.647127802807434e-09,
    1.378633446915721e-07,
    -5.752545603517705e-08,
    1.1951628599778148e-08,
    -1.7543241719747647e-11,
    -1.0091543710600413e-09,
    4.162792991842583e-10,
    -8.56390702649298e-11,
    6.067215101604758e-14,
    7.1624989648114856e-12,
    -2.933186643771437e-12,
    5.996696365683689e-13,
    -2.1671786527323313e-16,
    -4.978339972369262e-14,
    2.0291628823713425e-14,
    -4.13125571381061e-15,
    8.286516239883097e-19,
];

/// A partir deste a, `gammainc` usa a expansão uniforme de Temme com dois termos: o primeiro
/// omitido, c₂/a²·e^{−aη²/2}/√(2πa), é < 1e-16 relativo (|c₂| ≤ ~4e-3).
const TEMME_MIN_A: f64 = 1e7;

/// (P, Q) pela expansão uniforme de Temme (DLMF 8.12.3–8.12.10):
/// Q = ½ erfc(η√(a/2)) + R, P = ½ erfc(−η√(a/2)) − R, R = e^{−aη²/2}/√(2πa)·(c₀(η) + c₁(η)/a),
/// com μ = (x − a)/a, η = sinal(μ)·√(−2·log1pmx(μ)) e −aη²/2 = a·log1pmx(μ). c₀ e c₁ pelas séries
/// de Taylor (ferramentas/oraculos/f042_temme.py) para |η| ≤ 0,3. Fora disso, com a ≥ 10⁷,
/// e^{−aη²/2} ≤ e^{−4,5·10⁵} e erfc(\|η\|√(a/2)) com argumento ≥ 670 valem zero em f64: R = 0
/// exatamente, e P e Q ficam 0 ou 1 (as formas fechadas de c₀ e c₁ nunca teriam efeito, e
/// avaliá-las com η enorme daria Inf·0 = NaN).
fn gammainc_temme(x: f64, a: f64) -> (f64, f64) {
    let mu = (x - a) / a;
    let l = log1pmx(mu);
    let eta = (-2.0 * l).sqrt().copysign(mu);
    let r = if eta.abs() <= 0.3 {
        let h = |c: &[f64]| c.iter().rev().fold(0.0, |s, &k| s * eta + k);
        (a * l).exp() / (2.0 * PI * a).sqrt() * (h(&TEMME_C0) + h(&TEMME_C1) / a)
    } else {
        0.0
    };
    let z = eta * (0.5 * a).sqrt();
    (0.5 * erfc(-z) - r, 0.5 * erfc(z) + r)
}

/// Limite de termos da série e da fração contínua de `gammainc`: perto da transição x ≈ a elas
/// precisam de ~12√a termos. As três funções abaixo isolam escolhas que só afetam o custo ou
/// trocam entre dois métodos convergentes; seus mutantes de fronteira e de custo estão
/// registrados como equivalentes em `.cargo/mutants.toml`.
fn terms_cap(a: f64) -> f64 {
    100.0 + 15.0 * a.sqrt()
}

/// Série para x < a + 1; fração contínua acima (as duas convergem dos dois lados).
fn use_series(x: f64, a: f64) -> bool {
    x < a + 1.0
}

/// Σ x^k / ((a+1)…(a+k)), parando quando o termo fica abaixo de 1e-17 da soma.
fn lower_series(x: f64, a: f64, max_terms: f64) -> f64 {
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut k = 1.0;
    while term > 1e-17 * sum && k < max_terms {
        term *= x / (a + k);
        sum += term;
        k += 1.0;
    }
    sum
}

/// Gama incompleta regularizada: `P(a, x)` (inferior) ou `Q(a, x) = 1 − P` (superior),
/// para a > 0 e x ≥ 0. NaN fora do domínio.
pub fn gammainc(x: f64, a: f64, upper: bool) -> f64 {
    if x.is_nan() || a.is_nan() || x < 0.0 || a <= 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return if upper { 1.0 } else { 0.0 };
    }
    if x.is_infinite() {
        return if upper { 0.0 } else { 1.0 };
    }
    if a >= TEMME_MIN_A {
        let (p, q) = gammainc_temme(x, a);
        return if upper { q } else { p };
    }
    let max_terms = terms_cap(a);
    if use_series(x, a) {
        let p = gammainc_prefactor(x, a) / a * lower_series(x, a, max_terms);
        let p = p.min(1.0);
        return if upper { 1.0 - p } else { p };
    }
    // Fração contínua para Q: b_k = x + 2k − 1 − a, a₁ = 1, a_{k+1} = −k(k − a)
    let cf = lentz_n(0.0, max_terms as usize, |k| {
        let kk = k as f64;
        let ak = if k == 1 { 1.0 } else { -(kk - 1.0) * (kk - 1.0 - a) };
        (ak, x + 2.0 * kk - 1.0 - a)
    });
    let q = (gammainc_prefactor(x, a) * cf).min(1.0);
    if upper { q } else { 1.0 - q }
}

/// Beta incompleta regularizada `I_x(a, b)` (ou `1 − I_x` com `upper`), para 0 ≤ x ≤ 1, a, b > 0.
pub fn betainc(x: f64, a: f64, b: f64, upper: bool) -> f64 {
    if x.is_nan() || !(0.0..=1.0).contains(&x) || !(a > 0.0 && b > 0.0) {
        return f64::NAN;
    }
    if x == 0.0 || x == 1.0 {
        let lower = x;
        return if upper { 1.0 - lower } else { lower };
    }
    // Usa a forma que converge rápido; a outra é obtida pela simetria I_x(a,b) = 1 − I_{1−x}(b,a).
    let swap = x > (a + 1.0) / (a + b + 2.0);
    let (xx, aa, bb) = if swap { (1.0 - x, b, a) } else { (x, a, b) };
    let cf = lentz(1.0, |k| {
        let m = (k / 2) as f64;
        let d = if k % 2 == 1 {
            -(aa + m) * (aa + bb + m) * xx / ((aa + 2.0 * m) * (aa + 2.0 * m + 1.0))
        } else {
            m * (bb - m) * xx / ((aa + 2.0 * m - 1.0) * (aa + 2.0 * m))
        };
        (d, 1.0)
    });
    let pref = (aa * xx.ln() + bb * (-xx).ln_1p() - betaln(aa, bb) - aa.ln()).exp();
    let i = (pref / cf).clamp(0.0, 1.0);
    // i é I_xx(aa, bb): quando houve troca, i é a cauda superior do problema original.
    match (swap, upper) {
        (false, false) | (true, true) => i,
        _ => 1.0 - i,
    }
}
