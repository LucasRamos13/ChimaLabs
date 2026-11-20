//! F042: ζ de Riemann, ζ de Hurwitz e poligama. Oráculos: formas fechadas (ζ(2k) e ζ(−n) por
//! números de Bernoulli; ψ⁽ⁿ⁾ em 1, ½ e ¼), constantes publicadas com 30+ dígitos (Apéry ζ(3),
//! ζ(5), ζ(7), ζ(±½), ζ′(½), Catalan G, Stieltjes γ₁) e identidades (recorrência, multiplicação,
//! continuidade).

// As constantes ficam com todos os dígitos publicados (30+), para conferência; o f64 usa os 17
// primeiros.
#![allow(clippy::excessive_precision)]

use chimalab_math::special::*;
use std::f64::consts::PI;

const ZETA3: f64 = 1.202_056_903_159_594_285_399_738_161_511_449_990_765;
const CATALAN: f64 = 0.915_965_594_177_219_015_054_603_514_932_384_110_774;

fn rel(got: f64, want: f64) -> f64 {
    if want == 0.0 {
        got.abs()
    } else {
        ((got - want) / want).abs()
    }
}

fn close(got: f64, want: f64, tol: f64, what: &str) {
    assert!(
        rel(got, want) <= tol,
        "{what}: {got:e} × {want:e} (erro {:e})",
        rel(got, want)
    );
}

#[test]
fn zeta_em_inteiros_pares_por_bernoulli() {
    // ζ(2k) = (−1)^(k+1)·B₂ₖ·(2π)^(2k)/(2·(2k)!) (DLMF 25.6.2).
    close(zeta(2.0), PI.powi(2) / 6.0, 2e-16, "ζ(2)");
    close(zeta(4.0), PI.powi(4) / 90.0, 4e-16, "ζ(4)");
    close(zeta(6.0), PI.powi(6) / 945.0, 4e-16, "ζ(6)");
    close(zeta(8.0), PI.powi(8) / 9450.0, 4e-16, "ζ(8)");
    close(zeta(10.0), PI.powi(10) / 93555.0, 8e-16, "ζ(10)");
    close(zeta(12.0), 691.0 * PI.powi(12) / 638_512_875.0, 4e-16, "ζ(12)");
}

#[test]
fn zeta_por_constantes_publicadas() {
    close(zeta(3.0), ZETA3, 8e-16, "ζ(3) (Apéry)");
    close(
        zeta(5.0),
        1.036_927_755_143_369_926_331_365_486_457_034_168_057,
        8e-16,
        "ζ(5)",
    );
    close(
        zeta(7.0),
        1.008_349_277_381_922_826_839_797_549_849_796_759_600,
        8e-16,
        "ζ(7)",
    );
    close(
        zeta(0.5),
        -1.460_354_508_809_586_812_889_499_152_515_298_012_5,
        4e-16,
        "ζ(½)",
    );
    close(
        zeta(-0.5),
        -0.207_886_224_977_354_566_017_306_725_397_049_302_2,
        4e-16,
        "ζ(−½)",
    );
    // s grande: 1 + 2⁻ˢ + 3⁻ˢ + … (termos exatos em potências de 2).
    close(
        zeta(30.0),
        1.0 + 2f64.powi(-30) + 3f64.powi(-30) + 4f64.powi(-30),
        2e-16,
        "ζ(30)",
    );
    assert_eq!(zeta(60.0), 1.0 + 2f64.powi(-60));
    assert_eq!(zeta(1e5), 1.0);
}

#[test]
fn zeta_em_inteiros_negativos_e_zeros_triviais() {
    // ζ(−n) = −Bₙ₊₁/(n + 1) (DLMF 25.6.3); ζ(0) = −½; ζ(−2k) = 0 exatamente.
    assert_eq!(zeta(0.0), -0.5);
    assert_eq!(zeta(-0.0), -0.5);
    for (s, want) in [
        (-1.0, -1.0 / 12.0),
        (-3.0, 1.0 / 120.0),
        (-5.0, -1.0 / 252.0),
        (-7.0, 1.0 / 240.0),
        (-9.0, -1.0 / 132.0),
        (-11.0, 691.0 / 32760.0),
        (-13.0, -1.0 / 12.0),
    ] {
        close(zeta(s), want, 8e-16, &format!("ζ({s})"));
    }
    for k in 1..=200 {
        assert_eq!(zeta(-2.0 * k as f64), 0.0, "ζ(−{})", 2 * k);
    }
}

#[test]
fn zeta_perto_do_polo() {
    // ζ(1 + h) = 1/h + γ − γ₁·h + O(h²) (DLMF 25.2.4), γ₁ = −0,0728158454836767… (Stieltjes).
    let gamma1 = -0.072_815_845_483_676_724_860_586;
    // h ≤ 2⁻²⁰: o termo seguinte, γ₂h²/2, fica abaixo de 1e-20 relativo.
    for k in [20, 27, 40] {
        let h = 2f64.powi(-k);
        let want = 1.0 / h + std::f64::consts::EULER_GAMMA - gamma1 * h;
        close(zeta(1.0 + h), want, 4e-16, &format!("ζ(1 + 2^−{k})"));
        close(
            zeta(1.0 - h),
            -1.0 / h + std::f64::consts::EULER_GAMMA + gamma1 * h,
            4e-16,
            "ζ(1 − h)",
        );
    }
}

#[test]
fn zeta_continua_na_troca_de_metodo() {
    // s = ½ separa Euler-Maclaurin da equação funcional. A diferença central de passo δ deve dar
    // ζ′(½) = −3,92264613920915172747… (constante publicada).
    let d = 1e-6;
    let deriv = (zeta(0.5 + d) - zeta(0.5 - d)) / (2.0 * d);
    assert!((deriv + 3.922_646_139_209_151_727_47).abs() < 1e-8, "{deriv}");
}

#[test]
fn zeta_muito_negativa_sem_estouro_intermediario() {
    // Para 1 − s > 171, Γ(1 − s) estoura e o cálculo passa a log. A razão
    // ζ(s − 2)/ζ(s) = −(1 − s)(2 − s)/(4π²)·ζ(3 − s)/ζ(1 − s) (da equação funcional) confere a
    // continuidade entre os dois caminhos.
    for s in [-165.5, -169.5, -171.5, -175.5, -201.0] {
        let ratio = zeta(s - 2.0) / zeta(s);
        let want = -(1.0 - s) * (2.0 - s) / (4.0 * PI * PI) * zeta(3.0 - s) / zeta(1.0 - s);
        close(ratio, want, 2e-13, &format!("razão em s = {s}"));
    }
    // |ζ(−400,5)| ~ 10⁶²⁰: estoura, com o sinal de sin(πs/2).
    assert!(zeta(-400.5).is_infinite());
}

#[test]
fn zeta_casos_de_borda() {
    assert!(zeta(f64::NAN).is_nan());
    assert!(zeta(f64::NEG_INFINITY).is_nan());
    assert_eq!(zeta(f64::INFINITY), 1.0);
    assert_eq!(zeta(1.0), f64::INFINITY);
    // Regressão: com s subnormal, s/2 subestourava, sin(πs/2) = 0 e ζ saía 0 (como um zero
    // trivial). ζ(s) = −½ − ½·ln(2π)·s + O(s²).
    assert_eq!(zeta(5e-324), -0.5);
    assert_eq!(zeta(-5e-324), -0.5);
    let s = 1e-9;
    close(zeta(s), -0.5 - 0.5 * (2.0 * PI).ln() * s, 2e-16, "ζ(1e-9)");
    close(zeta(-s), -0.5 + 0.5 * (2.0 * PI).ln() * s, 2e-16, "ζ(−1e-9)");
    // Logo acima do limiar da série, pela equação funcional: ζ′(0) = −½·ln(2π), ζ″(0)/2 ≈ −1,0031.
    // Regressão: com 1 − s arredondado, o termo do polo a/(t − 1) do Euler-Maclaurin errava em
    // 5e-10; agora t − 1 = −s entra exato.
    let s = 2e-8;
    close(
        zeta(s),
        -0.5 - 0.5 * (2.0 * PI).ln() * s - 1.003_1 * s * s,
        4e-16,
        "ζ(2e-8)",
    );
}

#[test]
fn hurwitz_formas_fechadas() {
    // ζ(2, ¼) = π² + 8G, ζ(2, ¾) = π² − 8G (G de Catalan); ζ(s, ½) = (2ˢ − 1)ζ(s); ζ(s, 1) = ζ(s).
    close(hurwitz_zeta(2.0, 0.25), PI * PI + 8.0 * CATALAN, 4e-16, "ζ(2, ¼)");
    close(hurwitz_zeta(2.0, 0.75), PI * PI - 8.0 * CATALAN, 4e-16, "ζ(2, ¾)");
    close(hurwitz_zeta(3.0, 0.5), 7.0 * ZETA3, 4e-16, "ζ(3, ½)");
    close(hurwitz_zeta(4.0, 1.0), PI.powi(4) / 90.0, 4e-16, "ζ(4, 1)");
    // q enorme: ζ(s, q) ≈ q^(1−s)/(s − 1) + q^(−s)/2, sem subestouro de q⁻ˢ.
    close(hurwitz_zeta(2.0, 1e300), 1e-300, 2e-16, "ζ(2, 1e300)");
    close(hurwitz_zeta(3.0, 1e150), 0.5e-300, 2e-16, "ζ(3, 1e150)");
}

#[test]
fn hurwitz_casos_de_borda() {
    assert!(hurwitz_zeta(f64::NAN, 1.0).is_nan());
    assert!(hurwitz_zeta(2.0, f64::NAN).is_nan());
    assert!(hurwitz_zeta(0.5, 1.0).is_nan());
    assert!(hurwitz_zeta(2.0, 0.0).is_nan());
    assert!(hurwitz_zeta(2.0, -0.0).is_nan());
    assert!(hurwitz_zeta(2.0, -1.5).is_nan());
    assert_eq!(hurwitz_zeta(1.0, 2.0), f64::INFINITY);
    assert_eq!(hurwitz_zeta(2.0, f64::INFINITY), 0.0);
    assert_eq!(hurwitz_zeta(f64::INFINITY, 2.0), 0.0);
    assert_eq!(hurwitz_zeta(f64::INFINITY, 1.0), 1.0);
    assert_eq!(hurwitz_zeta(f64::INFINITY, 0.5), f64::INFINITY);
    // q subnormal: ζ ≈ q⁻ˢ estoura.
    assert_eq!(hurwitz_zeta(2.0, 5e-324), f64::INFINITY);
    // s enorme e q < 1: domina q⁻ˢ.
    close(hurwitz_zeta(400.0, 0.5), 2f64.powi(400), 2e-16, "ζ(400, ½)");
}

#[test]
fn poligama_formas_fechadas() {
    close(polygamma(1, 1.0), PI * PI / 6.0, 2e-16, "ψ′(1)");
    close(polygamma(1, 0.5), PI * PI / 2.0, 4e-16, "ψ′(½)");
    close(polygamma(1, 0.25), PI * PI + 8.0 * CATALAN, 4e-16, "ψ′(¼)");
    close(polygamma(2, 1.0), -2.0 * ZETA3, 8e-16, "ψ″(1)");
    close(polygamma(2, 0.5), -14.0 * ZETA3, 4e-16, "ψ″(½)");
    close(polygamma(3, 1.0), PI.powi(4) / 15.0, 4e-16, "ψ‴(1)");
    close(polygamma(5, 1.0), 120.0 * PI.powi(6) / 945.0, 4e-16, "ψ⁽⁵⁾(1)");
    // x negativo (reflexão): pela recorrência a partir de x = ½.
    close(polygamma(1, -0.5), PI * PI / 2.0 + 4.0, 2e-16, "ψ′(−½)");
    close(polygamma(2, -0.5), -14.0 * ZETA3 + 16.0, 4e-16, "ψ″(−½)");
    // n = 0 é a digama.
    for x in [-2.5, 0.1, 1.0, 7.25, 1e10] {
        assert_eq!(polygamma(0, x), psi(x));
    }
}

#[test]
fn poligama_assintotica_e_extremos() {
    // ψ′(x) ~ 1/x + 1/(2x²) + 1/(6x³) − 1/(30x⁵) (DLMF 5.15.8).
    let x: f64 = 1e3;
    let want = 1.0 / x + 0.5 / (x * x) + 1.0 / (6.0 * x.powi(3)) - 1.0 / (30.0 * x.powi(5));
    close(polygamma(1, x), want, 2e-16, "ψ′(1000)");
    // Sem subestouro de x⁻ⁿ⁻¹ quando o resultado é representável.
    close(polygamma(1, 1e300), 1e-300, 2e-16, "ψ′(1e300)");
    close(polygamma(3, 1e100), 2e-300, 2e-16, "ψ‴(1e100)");
    // Meio-inteiro negativo enorme: cot(πx) = 0 exatamente e ψ″(x) = ψ″(1 − x) ≈ −1/x².
    close(
        polygamma(2, -1e15 + 0.5),
        -1.0 / (1e15f64 + 0.5).powi(2),
        2e-15,
        "ψ″(−1e15 + ½)",
    );
    // n > 170 (n! estoura): caminho em log. Pela recorrência, ψ⁽ⁿ⁾(x) − ψ⁽ⁿ⁾(x + 1) = (−1)ⁿ⁺¹·n!/xⁿ⁺¹,
    // com o lado direito calculado como produto Π(k/x)/x, sem logaritmos. Em x = 50 os dois
    // valores são da ordem de 10²², e o segundo é ~3% do primeiro (cancelamento pequeno).
    let (n, x) = (180u32, 50.0_f64);
    let term = (1..=n).fold(1.0 / x, |acc, k| acc * f64::from(k) / x);
    let diff = polygamma(n, x) - polygamma(n, x + 1.0);
    close(diff, -term, 2e-13, "ψ⁽¹⁸⁰⁾(50) − ψ⁽¹⁸⁰⁾(51)");
    assert!(polygamma(n, x).is_finite() && polygamma(n, x) < 0.0);
}

#[test]
fn poligama_casos_de_borda() {
    for n in [1u32, 2, 5] {
        assert!(polygamma(n, f64::NAN).is_nan());
        assert!(polygamma(n, f64::NEG_INFINITY).is_nan());
        assert!(polygamma(n, 0.0).is_nan());
        assert!(polygamma(n, -0.0).is_nan());
        assert!(polygamma(n, -3.0).is_nan());
        assert_eq!(polygamma(n, f64::INFINITY), 0.0);
    }
    // x subnormal: ψ⁽ⁿ⁾(x) ≈ (−1)ⁿ⁺¹·n!/xⁿ⁺¹.
    assert_eq!(polygamma(1, 5e-324), f64::INFINITY);
    assert_eq!(polygamma(2, 5e-324), f64::NEG_INFINITY);
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn fact(n: u32) -> f64 {
        (1..=n).map(f64::from).product()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// ψ⁽ⁿ⁾(x + 1) − ψ⁽ⁿ⁾(x) = (−1)ⁿ·n!·x⁻ⁿ⁻¹ (DLMF 5.15.5), dos dois lados de zero. Tolerância
        /// relativa ao maior termo envolvido.
        #[test]
        fn poligama_recorrencia(n in 1u32..8, x in -40.0f64..40.0) {
            prop_assume!(x.fract().abs() > 1e-3 && (x + 1.0).fract().abs() > 1e-3);
            let (a, b) = (polygamma(n, x + 1.0), polygamma(n, x));
            let term = (if n % 2 == 0 { 1.0 } else { -1.0 }) * fact(n) * x.powi(-(n as i32) - 1);
            let scale = a.abs().max(b.abs()).max(term.abs());
            prop_assert!(((a - b) - term).abs() <= 1e-13 * scale, "{} {} {}", a, b, term);
        }

        /// Σ_{r<m} ψ⁽ⁿ⁾(x + r/m) = mⁿ⁺¹·ψ⁽ⁿ⁾(mx) (DLMF 5.15.7), m = 2 e 4. x diádico (k/64):
        /// x + r/m e mx são exatos, e o arredondamento do argumento (amplificado por ~n + 1) não
        /// entra na comparação.
        #[test]
        fn poligama_multiplicacao(n in 1u32..8, k in 1u32..2000, m in prop::sample::select(vec![2u32, 4])) {
            let x = f64::from(k) / 64.0;
            let sum: f64 = (0..m).map(|r| polygamma(n, x + f64::from(r) / f64::from(m))).sum();
            let abs: f64 = (0..m).map(|r| polygamma(n, x + f64::from(r) / f64::from(m)).abs()).sum();
            let want = f64::from(m).powi(n as i32 + 1) * polygamma(n, f64::from(m) * x);
            prop_assert!((sum - want).abs() <= 4e-15 * abs, "{} × {}", sum, want);
        }

        /// ζ(s, q) = ζ(s, q + 1) + q⁻ˢ (todos positivos: sem cancelamento). q diádico (k/64), para
        /// que q + 1 seja exato: a sensibilidade ∂ln ζ/∂ln q ≈ s amplificaria o arredondamento.
        #[test]
        fn hurwitz_recorrencia(s in 1.01f64..60.0, k in 1u32..6400) {
            let q = f64::from(k) / 64.0;
            let lhs = hurwitz_zeta(s, q);
            let rhs = hurwitz_zeta(s, q + 1.0) + q.powf(-s);
            prop_assert!(rel(lhs, rhs) <= 2e-15, "{} × {}", lhs, rhs);
        }

        /// Σ_{r<m} ζ(s, q + r/m) = mˢ·ζ(s, mq) (DLMF 25.11.15), m = 2 e 4, q diádico.
        #[test]
        fn hurwitz_multiplicacao(s in 1.01f64..40.0, k in 1u32..3200, m in prop::sample::select(vec![2u32, 4])) {
            let q = f64::from(k) / 64.0;
            let sum: f64 = (0..m).map(|r| hurwitz_zeta(s, q + f64::from(r) / f64::from(m))).sum();
            let want = f64::from(m).powf(s) * hurwitz_zeta(s, f64::from(m) * q);
            prop_assert!(rel(sum, want) <= 4e-15, "{} × {}", sum, want);
        }

        /// ζ(s) = ζ(s, 1) e ζ(s, ½) = (2ˢ − 1)ζ(s) (DLMF 25.11.11), para s > 1.
        #[test]
        fn zeta_e_hurwitz_em_meio(s in 1.01f64..60.0) {
            prop_assert!(rel(hurwitz_zeta(s, 1.0), zeta(s)) <= 2e-16);
            let want = (2f64.powf(s) - 1.0) * zeta(s);
            prop_assert!(rel(hurwitz_zeta(s, 0.5), want) <= 4e-15);
        }
    }
}
