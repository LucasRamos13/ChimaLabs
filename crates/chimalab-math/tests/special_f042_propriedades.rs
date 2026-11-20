//! Testes F042: trigonometria em graus, arredondamentos, inteiros, combinatória e identidades.

use chimalab_math::special::*;

#[test]
fn ca1_trigonometria_em_graus_exata_nos_multiplos_de_90() {
    assert_eq!(sind(180.0), 0.0);
    assert_eq!(sind(360.0), 0.0);
    assert_eq!(sind(90.0), 1.0);
    assert_eq!(sind(-90.0), -1.0);
    assert_eq!(sind(270.0), -1.0);
    assert_eq!(sind(1e6 * 360.0 + 30.0), sind(30.0));
    assert_eq!(cosd(90.0), 0.0);
    assert_eq!(cosd(180.0), -1.0);
    assert_eq!(cosd(-720.0), 1.0);
    assert_eq!(tand(45.0), 1.0);
    assert_eq!(tand(-45.0), -1.0);
    assert_eq!(tand(135.0), -1.0);
    assert!(tand(90.0).is_infinite());
    assert_eq!(tand(180.0), 0.0);
    assert!((sind(30.0) - 0.5).abs() <= f64::EPSILON / 2.0);
    assert!(sind(f64::INFINITY).is_nan());
    assert_eq!(sind(-0.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(asind(0.5), 30.0);
    assert_eq!(acosd(-1.0), 180.0);
    assert_eq!(atand(f64::NEG_INFINITY), -90.0);
    assert_eq!(atan2d(1.0, -1.0), 135.0);
    assert!((deg2rad(180.0) - std::f64::consts::PI).abs() < 1e-15);
}

#[test]
fn arredondamentos_restos_e_raizes() {
    assert_eq!(fix(-2.7), -2.0);
    assert_eq!(round_digits(2.345, 1), 2.3); // 2.345 é 2.34499… em binário
    assert_eq!(round_digits(1234.0, -2), 1200.0);
    assert_eq!(round_digits(-2.5, 0), -3.0); // meio para longe do zero
    assert_eq!(mod_floor(-7.0, 3.0), 2.0);
    assert_eq!(mod_floor(7.0, -3.0), -2.0);
    assert_eq!(mod_floor(5.5, 0.0), 5.5);
    assert_eq!(rem_trunc(-7.0, 3.0), -1.0);
    assert!(rem_trunc(1.0, 0.0).is_nan());
    // Exatidão: resto de valores grandes
    assert_eq!(mod_floor(1e17 + 2.0, 10.0), (1e17f64 + 2.0) % 10.0);
    assert_eq!(nthroot(-27.0, 3).unwrap(), -3.0);
    assert_eq!(nthroot(1024.0, 10).unwrap(), 2.0);
    assert_eq!(
        nthroot(-16.0, 4).unwrap_err().id(),
        "chimalab:special:resultadoComplexo"
    );
    assert_eq!(
        (realsqrt(-1.0)).unwrap_err().id(),
        "chimalab:special:resultadoComplexo"
    );
    assert_eq!(
        (realpow(-8.0, 1.0 / 3.0)).unwrap_err().id(),
        "chimalab:special:resultadoComplexo"
    );
    assert_eq!(realpow(-2.0, 3.0).unwrap(), -8.0);
    assert_eq!(sign(-0.0), 0.0);
    for x in [1.0, -3.5, 1e-310, 6.02e23, f64::MAX] {
        let (f, e) = frexp(x);
        assert!((0.5..1.0).contains(&f.abs()), "{x}");
        assert_eq!(ldexp(f, e), x);
    }
}

#[test]
fn mdc_mmc_e_bezout() {
    assert_eq!(gcd(12.0, -18.0).unwrap(), 6.0);
    assert_eq!(gcd(0.0, 0.0).unwrap(), 0.0);
    assert_eq!(lcm(4.0, 6.0).unwrap(), 12.0);
    assert_eq!(lcm(0.0, 5.0).unwrap(), 0.0);
    let b = gcd_bezout(240.0, 46.0).unwrap();
    assert_eq!(b.g, 2.0);
    assert_eq!(b.u * 240.0 + b.v * 46.0, 2.0);
    assert_eq!((gcd(1.5, 3.0)).unwrap_err().id(), "chimalab:special:dominio");
}

#[test]
fn primos_e_fatoracao() {
    assert_eq!(primes(30.0), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    assert_eq!(primes(1e6).len(), 78_498); // π(10⁶)
    assert!(primes(1.0).is_empty());
    assert!(isprime((1 << 61) - 1)); // primo de Mersenne
    assert!(!isprime(561)); // número de Carmichael
    assert!(!isprime(3_215_031_751)); // pseudoprimo forte para as bases 2, 3, 5 e 7
    assert!(!isprime(1) && isprime(2) && !isprime(0));
    assert_eq!(factor(600_851_475_143), vec![71, 839, 1471, 6857]);
    assert_eq!(
        factor(1_000_000_007 * 998_244_353),
        vec![998_244_353, 1_000_000_007]
    );
    assert_eq!(factor(1 << 40), vec![2; 40]);
    assert_eq!(factor(97), vec![97]);
    assert_eq!(factor(1), vec![1]);
    let n = 18_446_744_073_709_551_557u64; // maior primo < 2⁶⁴
    assert!(isprime(n));
    assert_eq!(factor(n), vec![n]);
}

#[test]
fn combinatoria() {
    assert_eq!(nchoosek(5.0, 2.0).unwrap(), (10.0, true));
    assert_eq!(nchoosek(52.0, 5.0).unwrap(), (2_598_960.0, true));
    assert_eq!(nchoosek(10.0, 0.0).unwrap(), (1.0, true));
    let (v, exact) = nchoosek(100.0, 50.0).unwrap();
    assert!(!exact);
    assert!((v / 1.008_913_445_455_642e29 - 1.0).abs() < 1e-14);
    assert_eq!((nchoosek(3.0, 5.0)).unwrap_err().id(), "chimalab:special:dominio");
    let c = combinations(&['a', 'b', 'c', 'd'], 2);
    assert_eq!(c.len(), 6);
    assert_eq!(c[0], vec!['a', 'b']);
    assert_eq!(c[5], vec!['c', 'd']);
    let p = perms(&[1, 2, 3]);
    assert_eq!(
        p,
        vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1]
        ]
    );
    assert_eq!(perms(&[0; 5]).len(), 120);
    assert_eq!(factorial(20.0), 2_432_902_008_176_640_000.0);
    assert!(factorial(171.0).is_infinite());
    assert!(factorial(-1.0).is_nan());
}

#[test]
fn identidades_das_funcoes_especiais() {
    // Γ em inteiros: exata
    for n in 1..=22 {
        let expected: f64 = (1..n).map(|k| k as f64).product();
        assert_eq!(gamma(n as f64), expected);
    }
    // Γ(x+1) = xΓ(x), Γ(1/2) = √π, polos
    for x in [0.37, 2.5, 7.9, 33.3] {
        assert!((gamma(x + 1.0) / (x * gamma(x)) - 1.0).abs() < 1e-14);
    }
    assert!((gamma(0.5) - std::f64::consts::PI.sqrt()).abs() < 1e-15);
    assert!(gamma(-3.0).is_nan());
    assert_eq!(gamma(-0.0), f64::NEG_INFINITY);
    assert!(gamma(172.0).is_infinite());
    // P + Q = 1; simetria da beta incompleta
    for (x, a) in [(0.3, 0.7), (4.0, 2.5), (30.0, 25.0)] {
        assert!((gammainc(x, a, false) + gammainc(x, a, true) - 1.0).abs() < 1e-14);
    }
    for (x, a, b) in [(0.2, 2.0, 5.0), (0.8, 0.5, 3.0)] {
        let l = betainc(x, a, b, false);
        assert!((l - (1.0 - betainc(1.0 - x, b, a, false))).abs() < 1e-14);
        assert!((l + betainc(x, a, b, true) - 1.0).abs() < 1e-15);
    }
    // erf/erfinv ida e volta; erfc(−x) = 2 − erfc(x)
    for y in [-0.9999, -0.3, 0.0, 0.1, 0.7, 0.99999] {
        assert!((erf(erfinv(y)) - y).abs() <= 2.0 * f64::EPSILON);
    }
    for z in [1e-250, 1e-30, 0.01, 0.9, 1.7] {
        assert!((erfc(erfcinv(z)) / z - 1.0).abs() < 1e-13, "{z}");
    }
    assert_eq!(erfcinv(0.0), f64::INFINITY);
    assert!(erfinv(1.5).is_nan());
    // E₁ e Ei: Ei(−x) = −E₁(x); E₁ complexa para x < 0
    for x in [0.2, 3.0, 40.0] {
        assert!((ei(-x) + expint(x)).abs() <= 1e-15 * expint(x).abs());
    }
    let z = expint_complex(-1.0);
    assert!((z.re + ei(1.0)).abs() < 1e-15 && (z.im + std::f64::consts::PI).abs() < 1e-15);
    assert!(expint(-1.0).is_nan());
    // Jacobi: sn² + cn² = 1, dn² + m·sn² = 1; casos m = 0 e 1
    for (u, m) in [(0.3, 0.2), (4.0, 0.7), (-2.0, 0.99)] {
        let (sn, cn, dn) = ellipj(u, m);
        assert!((sn * sn + cn * cn - 1.0).abs() < 1e-15);
        assert!((dn * dn + m * sn * sn - 1.0).abs() < 1e-15);
    }
    assert_eq!(ellipj(1.0, 0.0), (1f64.sin(), 1f64.cos(), 1.0));
    // K(0) = E(0) = π/2; E(1) = 1; K(1) = ∞
    let (k0, e0) = ellipke(0.0);
    assert!(
        (k0 - std::f64::consts::FRAC_PI_2).abs() < 1e-16 && (e0 - std::f64::consts::FRAC_PI_2).abs() < 1e-16
    );
    assert_eq!(ellipke(1.0), (f64::INFINITY, 1.0));
    assert!(ellipke(1.5).0.is_nan());
    // ψ(1) = −γ; ψ(x+1) = ψ(x) + 1/x
    assert!((psi(1.0) + std::f64::consts::EULER_GAMMA).abs() < 1e-15);
    for x in [0.3, 4.7, 25.0] {
        assert!((psi(x + 1.0) - psi(x) - 1.0 / x).abs() < 1e-14);
    }
}

#[test]
fn ca3_bessel_de_ordem_fracionaria_e_identidades() {
    use std::f64::consts::PI;
    // Formas fechadas de ordem ±1/2 (DLMF 10.16.1, 10.39.2)
    for x in [0.01, 0.7, 3.3, 12.0, 47.0, 250.0] {
        let a = (2.0 / (PI * x)).sqrt();
        assert!((besselj(0.5, x) - a * x.sin()).abs() < 1e-14 * a, "J½({x})");
        assert!((besselj(-0.5, x) - a * x.cos()).abs() < 1e-14 * a, "J−½({x})");
        assert!((bessely(0.5, x) + a * x.cos()).abs() < 1e-14 * a, "Y½({x})");
        let k = (PI / (2.0 * x)).sqrt() * (-x).exp();
        assert!((besselk(0.5, x, false) / k - 1.0).abs() < 1e-14, "K½({x})");
    }
    // Wronskianos (DLMF 10.5.2, 10.28.2) e recorrência (DLMF 10.6.1)
    for (nu, x) in [(0.3, 2.0), (2.7, 9.0), (7.5, 40.0), (0.0, 0.5)] {
        let w = besselj(nu + 1.0, x) * bessely(nu, x) - besselj(nu, x) * bessely(nu + 1.0, x);
        assert!((w - 2.0 / (PI * x)).abs() < 1e-13 / x, "W(J,Y) ν={nu} x={x}");
        let wi = besseli(nu, x, true) * besselk(nu + 1.0, x, true)
            + besseli(nu + 1.0, x, true) * besselk(nu, x, true);
        assert!((wi * x - 1.0).abs() < 1e-13, "W(I,K) ν={nu} x={x}");
        let lhs = besselj(nu, x) + besselj(nu + 2.0, x);
        assert!(
            (lhs - 2.0 * (nu + 1.0) / x * besselj(nu + 1.0, x)).abs() < 1e-13,
            "recorrência ν={nu} x={x}"
        );
    }
    // Argumento negativo (ordem inteira) e valores especiais
    assert!((besselj(3.0, -2.5) + besselj(3.0, 2.5)).abs() < 1e-16);
    assert!(besselj(0.5, -1.0).is_nan());
    assert_eq!(besselj(0.0, 0.0), 1.0);
    assert_eq!(besselj(2.0, 0.0), 0.0);
    assert_eq!(bessely(1.0, 0.0), f64::NEG_INFINITY);
    assert_eq!(besselk(0.0, 0.0, false), f64::INFINITY);
    // Airy: Wronskiano Ai·Bi' − Ai'·Bi = 1/π, verificado por diferença central
    for x in [-5.0, -0.5, 0.0, 2.0] {
        let h = 1e-5;
        let (ai, bi) = airy(x);
        let (aip, bip) = airy(x + h);
        let (aim, bim) = airy(x - h);
        let w = ai * (bip - bim) / (2.0 * h) - (aip - aim) / (2.0 * h) * bi;
        assert!((w - 1.0 / PI).abs() < 1e-8, "x={x}: {w}");
    }
}

/// Primos de uma janela pelo Miller-Rabin determinístico (`isprime`): algoritmo independente do
/// crivo segmentado.
fn primos_por_miller_rabin(lo: u64, hi: u64) -> Vec<u64> {
    (lo..=hi).filter(|&v| isprime(v)).collect()
}

#[test]
fn primos_contagens_conhecidas() {
    // π(x) publicados: π(10⁷) = 664 579, π(10⁸) = 5 761 455.
    assert_eq!(primes(1e7).len(), 664_579);
    assert_eq!(primes(1e8).len(), 5_761_455);
    assert_eq!(primes(2.0), vec![2]);
    assert_eq!(primes(3.0), vec![2, 3]);
    assert_eq!(primes(4.0), vec![2, 3]);
    assert_eq!(primes(30.9), primes(30.0));
    assert!(primes(f64::NAN).is_empty() && primes(-5.0).is_empty() && primes(1.999).is_empty());
    assert!(primes(f64::NEG_INFINITY).is_empty() && primes(-0.0).is_empty());
    assert_eq!(primes(2.0), vec![2]);
}

#[test]
fn regressao_primos_acima_de_um_bilhao() {
    // Antes, n era cortado em 10⁹ sem aviso: primes(1e9 + 100) perdia os primos entre 10⁹ e
    // 10⁹ + 100. Os primos logo acima de 10⁹ são 1 000 000 007, 1 000 000 009, 1 000 000 021…
    let w = primes_between(1_000_000_000, 1_000_000_100);
    assert_eq!(&w[..3], &[1_000_000_007, 1_000_000_009, 1_000_000_021]);
    assert_eq!(w, primos_por_miller_rabin(1_000_000_000, 1_000_000_100));
}

#[test]
fn primos_em_janelas_de_todas_as_escalas() {
    // Inclui a fronteira 2⁴⁸ (acima dela os sobreviventes do crivo passam pelo isprime), o fim do
    // intervalo u64 e janelas que atravessam o fim de um segmento (2¹⁸ ímpares).
    let casos = [
        (0u64, 20_000u64),
        (1 << 19, (1 << 19) + 4000),
        (999_999_000_000, 999_999_000_000 + 20_000),
        ((1u64 << 48) - 10_000, (1u64 << 48) + 10_000),
        ((1u64 << 50) - 3, (1u64 << 50) + 6000),
        (u64::MAX - 20_000, u64::MAX),
    ];
    for (lo, hi) in casos {
        assert_eq!(
            primes_between(lo, hi),
            primos_por_miller_rabin(lo, hi),
            "[{lo}, {hi}]"
        );
    }
    // Janelas degeneradas.
    assert!(primes_between(10, 9).is_empty());
    assert!(primes_between(0, 1).is_empty());
    assert_eq!(primes_between(2, 2), vec![2]);
    assert_eq!(primes_between(9, 9), Vec::<u64>::new());
    assert_eq!(primes_between(u64::MAX - 58, u64::MAX - 58), vec![u64::MAX - 58]); // maior primo u64
    // Mais de um segmento: [0, 10⁶] tem 78 498 primos, igual a primes(1e6).
    assert_eq!(primes_between(0, 1_000_000), primes(1e6));
}

mod propriedades_primos {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Crivo segmentado = Miller-Rabin em janelas aleatórias de até 3000 números, em escalas
        /// de 2⁰ a 2⁶³.
        #[test]
        fn crivo_igual_a_miller_rabin(e in 0u32..64, frac in 0.0f64..1.0, w in 0u64..3000) {
            let lo = ((2f64.powi(e as i32) * (1.0 + frac)) as u64).min(u64::MAX - w);
            prop_assert_eq!(primes_between(lo, lo + w), primos_por_miller_rabin(lo, lo + w));
        }
    }
}

/// Medição de 2026-11-18 (esta máquina, `--release`): primes(1e7/1e8/1e9) em 16 ms / 155 ms /
/// 1,55 s; o crivo anterior (1 byte por número, até 1 GB) levava 197 ms / 1,75 s / 17,9 s.
#[test]
#[ignore = "medição de desempenho; rodar com --release -- --ignored --nocapture"]
fn desempenho_primes() {
    for n in [1e7, 1e8, 1e9] {
        let t = std::time::Instant::now();
        let c = primes(n).len();
        println!("primes({n:e}): {c} primos em {:?}", t.elapsed());
    }
}
