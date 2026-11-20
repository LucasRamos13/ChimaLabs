//! Padrão rigoroso (CLAUDE.md §7.4–7.5): bordas e erros das funções especiais (F042).

use chimalab_math::special::*;

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;

#[test]
fn erros_especiais_tem_id_e_mensagem() {
    for (e, id) in [
        (SpecialError::Domain("x"), "chimalab:special:dominio"),
        (
            SpecialError::ComplexResult("x"),
            "chimalab:special:resultadoComplexo",
        ),
    ] {
        assert_eq!(e.id(), id);
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn trigonometria_em_graus_bordas() {
    assert!(cosd(NAN).is_nan() && tand(INF).is_nan() && sind(-INF).is_nan());
    assert_eq!(cotd(45.0), 1.0);
    assert_eq!(secd(0.0), 1.0);
    assert_eq!(cscd(90.0), 1.0);
    assert_eq!(cscd(0.0), INF);
    assert_eq!((asind(-0.5), asind(1.0), asind(-1.0)), (-30.0, 90.0, -90.0));
    assert!((asind(0.25) - 0.25f64.asin().to_degrees()).abs() < 1e-13);
    assert_eq!(
        (acosd(1.0), acosd(0.0), acosd(0.5), acosd(-0.5)),
        (0.0, 90.0, 60.0, 120.0)
    );
    assert!((acosd(0.2) - 0.2f64.acos().to_degrees()).abs() < 1e-13);
    assert_eq!((atand(1.0), atand(-1.0), atand(INF)), (45.0, -45.0, 90.0));
    assert!((atand(0.3) - 0.3f64.atan().to_degrees()).abs() < 1e-13);
    assert_eq!(
        (atan2d(1.0, 1.0), atan2d(-1.0, 1.0), atan2d(-1.0, -1.0)),
        (45.0, -45.0, -135.0)
    );
    assert!((atan2d(1.0, 2.0) - 1f64.atan2(2.0).to_degrees()).abs() < 1e-13);
    assert_eq!(atan2d(0.0, 0.0), 0.0);
    assert!((rad2deg(std::f64::consts::PI) - 180.0).abs() < 1e-13);
    assert_eq!(tand(-90.0), f64::NEG_INFINITY);
}

#[test]
fn arredondamento_restos_raizes_bordas() {
    assert!(round_digits(NAN, 2).is_nan());
    assert_eq!(round_digits(INF, 2), INF);
    assert_eq!(round_digits(1e300, 5), 1e300); // multiplicação transbordaria: devolve x
    assert_eq!(round_digits(3e15, 2), 3e15); // já sem casas decimais representáveis
    assert!(mod_floor(INF, 2.0).is_nan());
    assert!(mod_floor(NAN, 2.0).is_nan() && mod_floor(1.0, NAN).is_nan());
    assert_eq!(mod_floor(3.0, INF), 3.0);
    assert_eq!(mod_floor(-3.0, INF), INF);
    assert_eq!(mod_floor(0.0, INF), 0.0);
    assert_eq!(mod_floor(6.0, 3.0), 0.0);
    assert!(sign(NAN).is_nan());
    assert_eq!((sign(-2.0), sign(3.0)), (-1.0, 1.0));
    assert_eq!(nthroot(8.0, 0).unwrap_err().id(), "chimalab:special:dominio");
    assert_eq!(nthroot(0.0, 3).unwrap(), 0.0);
    assert_eq!(nthroot(2.0, -1).unwrap(), 0.5);
    assert_eq!(realsqrt(9.0).unwrap(), 3.0);
    assert_eq!(reallog(1.0).unwrap(), 0.0);
    assert_eq!(
        (reallog(-1.0)).unwrap_err().id(),
        "chimalab:special:resultadoComplexo"
    );
    assert_eq!(frexp(0.0), (0.0, 0));
    assert_eq!(frexp(INF).1, 0);
    assert_eq!(ldexp(1.0, 2000), INF);
    assert_eq!(ldexp(1.0, -2000), 0.0);
    assert_eq!(ldexp(2f64.powi(-600), 1500), 2f64.powi(900)); // expoente > 1000 em passos
    assert_eq!(ldexp(1.0, -1050), f64::from_bits(1 << 24)); // 2^−1050 (subnormal), exato
}

#[test]
fn inteiros_bordas() {
    assert_eq!(gcd(NAN, 2.0).unwrap_err().id(), "chimalab:special:dominio");
    assert_eq!((gcd(1e17, 2.0)).unwrap_err().id(), "chimalab:special:dominio"); // acima de 2^53
    assert_eq!((lcm(1.5, 2.0)).unwrap_err().id(), "chimalab:special:dominio");
    assert_eq!(gcd_bezout(-12.0, 0.0).unwrap().g, 12.0);
    assert!(primes(NAN).is_empty());
    assert_eq!(
        (nchoosek(-1.0, 0.0)).unwrap_err().id(),
        "chimalab:special:dominio"
    );
    assert_eq!((nchoosek(2.5, 1.0)).unwrap_err().id(), "chimalab:special:dominio");
    // Coeficiente que não cabe em u128 durante o cálculo: caminho aproximado
    let (v, exact) = nchoosek(1000.0, 500.0).unwrap();
    assert!(!exact && (v.ln() - 689.467_261_567_851).abs() < 1e-10);
    let (v, exact) = nchoosek(60.0, 30.0).unwrap();
    assert!(!exact && v == 118_264_581_564_861_424.0); // > 2^53: valor arredondado
    assert!(combinations(&[1, 2], 3).is_empty());
    assert_eq!(combinations::<i32>(&[], 0), vec![Vec::<i32>::new()]);
    assert_eq!(perms::<i32>(&[]), vec![Vec::<i32>::new()]);
    assert_eq!(factor(0), vec![0]);
    assert_eq!(factor(3), vec![3]);
    // Composto com fatores grandes (força Pollard-Brent com reinício de c)
    assert_eq!(
        factor(4_294_967_291 * 4_294_967_279),
        vec![4_294_967_279, 4_294_967_291]
    );
}

#[test]
fn gama_beta_bordas() {
    assert!(gamma(NAN).is_nan() && gamma(f64::NEG_INFINITY).is_nan());
    assert_eq!(gamma(INF), INF);
    assert_eq!(gamma(0.0), INF);
    assert!((gamma(-2.5) - (-0.945_308_720_482_941_9)).abs() < 1e-15);
    assert!(gammaln(NAN).is_nan());
    assert_eq!(gammaln(INF), INF);
    assert_eq!(gammaln(-3.0), INF);
    assert!((gammaln(1e305) / (1e305 * (1e305f64.ln() - 1.0)) - 1.0).abs() < 1e-15);
    assert!(psi(NAN).is_nan() && psi(f64::NEG_INFINITY).is_nan() && psi(-2.0).is_nan());
    assert_eq!(psi(INF), INF);
    assert!(factorial(2.5).is_nan() && factorial(NAN).is_nan());
    assert!(beta(-1.0, 2.0).is_nan() && betaln(0.0, 1.0).is_nan());
    assert!((betaln(300.0, 400.0) - (gammaln(300.0) + gammaln(400.0) - gammaln(700.0))).abs() < 1e-11);
    assert!(
        gammainc(-1.0, 2.0, false).is_nan()
            && gammainc(1.0, 0.0, false).is_nan()
            && gammainc(NAN, 1.0, true).is_nan()
    );
    assert_eq!((gammainc(0.0, 2.0, false), gammainc(0.0, 2.0, true)), (0.0, 1.0));
    assert_eq!((gammainc(INF, 2.0, false), gammainc(INF, 2.0, true)), (1.0, 0.0));
    assert!(betainc(1.5, 1.0, 1.0, false).is_nan() && betainc(0.5, 0.0, 1.0, false).is_nan());
    assert_eq!(
        (betainc(0.0, 2.0, 3.0, false), betainc(1.0, 2.0, 3.0, false)),
        (0.0, 1.0)
    );
    assert_eq!(betainc(0.0, 2.0, 3.0, true), 1.0);
}

#[test]
fn erro_e_expint_bordas() {
    assert!(erf(NAN).is_nan() && erfc(NAN).is_nan() && erfcx(NAN).is_nan());
    assert_eq!((erf(INF), erf(-INF)), (1.0, -1.0));
    assert_eq!((erfc(30.0), erfc(-30.0)), (0.0, 2.0));
    assert_eq!(erf(-0.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(erfcx(-30.0), INF);
    assert!((erfcx(1e9) * 1e9 * std::f64::consts::PI.sqrt() - 1.0).abs() < 1e-15);
    assert!(erfinv(NAN).is_nan() && erfcinv(-0.1).is_nan() && erfcinv(2.1).is_nan());
    assert_eq!((erfinv(1.0), erfinv(-1.0)), (INF, -INF));
    assert_eq!((erfcinv(1.0), erfcinv(2.0)), (0.0, f64::NEG_INFINITY));
    assert!(expint(NAN).is_nan());
    assert_eq!(expint(800.0), 0.0);
    assert!(ei(NAN).is_nan());
    assert_eq!(ei(0.0), f64::NEG_INFINITY);
    assert_eq!(ei(710.0), INF);
    assert_eq!(expint_complex(2.0).im, 0.0);
}

#[test]
fn bessel_airy_elipticas_legendre_bordas() {
    for f in [
        besselj(NAN, 1.0),
        besselj(1.0, NAN),
        bessely(1.0, -1.0),
        besseli(0.5, -2.0, false),
        besselk(1.0, -1.0, false),
        besselk(NAN, 1.0, true),
    ] {
        assert!(f.is_nan());
    }
    assert!(airy(NAN).0.is_nan());
    assert_eq!(
        (besselj(2.0, INF), bessely(2.0, INF), besselk(2.0, INF, false)),
        (0.0, 0.0, 0.0)
    );
    // J₋₁.₅(x) ~ (x/2)^(−1,5)/Γ(−0,5) e Γ(−0,5) < 0 ⇒ −Inf
    assert_eq!(besselj(-1.5, 0.0), f64::NEG_INFINITY);
    assert_eq!(besselj(-2.0, 0.0), 0.0);
    assert_eq!(
        (
            besseli(0.0, 0.0, false),
            besseli(1.0, 0.0, false),
            besseli(-0.5, 0.0, false)
        ),
        (1.0, 0.0, INF)
    );
    assert_eq!(bessely(1.0, 0.0), f64::NEG_INFINITY);
    // Ordens inteiras negativas e simetrias (DLMF 10.4.1, 10.27.1)
    for x in [0.7, 6.0, 40.0] {
        assert!((besselj(-3.0, x) + besselj(3.0, x)).abs() < 1e-15);
        assert!((besselj(-4.0, x) - besselj(4.0, x)).abs() < 1e-15);
        assert!((bessely(-3.0, x) + bessely(3.0, x)).abs() < 1e-13 * bessely(3.0, x).abs().max(1.0));
        assert!((bessely(-2.0, x) - bessely(2.0, x)).abs() < 1e-13 * bessely(2.0, x).abs().max(1.0));
        assert!((besseli(-2.0, x, true) - besseli(2.0, x, true)).abs() < 1e-15);
        assert!((besselk(-1.3, x, true) - besselk(1.3, x, true)).abs() < 1e-15);
        assert!((besseli(3.0, -x, true) + besseli(3.0, x, true)).abs() < 1e-15);
    }
    // Airy: Ai(x) → 0 para x grande; valores em 0
    assert_eq!(airy(200.0).0, 0.0);
    let (a0, b0) = airy(0.0);
    assert!((b0 / a0 - 3f64.sqrt()).abs() < 1e-15);
    // Elípticas
    assert!(ellipke(NAN).0.is_nan());
    assert!(ellipj(NAN, 0.5).0.is_nan() && ellipj(1.0, 1.5).0.is_nan());
    let (s, c, d) = ellipj(0.7, 1.0);
    assert!((s - 0.7f64.tanh()).abs() < 1e-16 && (c - 1.0 / 0.7f64.cosh()).abs() < 1e-16 && c == d);
    // Legendre de grau alto: forma não normalizada pela conversão de N (n > 150)
    let p = legendre(200, 0.3, LegendreNorm::Unnormalized).unwrap();
    let q = legendre(200, 0.3, LegendreNorm::Full).unwrap();
    let factor0 = (200.5f64).sqrt();
    assert!((p[0] * factor0 - q[0]).abs() < 1e-12);
}

/// Regressão (proptest): com m > 0,9, dn vinha de co/cos(φ₁ − φ₀) e o resíduo de
/// dn² + m·sn² = 1 chegava a ~1e-12 (u = −19,76, m = 0,940). Agora dn² = (1 − m) + m·cn².
#[test]
fn regressao_ellipj_identidade_dn_perto_de_m_1() {
    let (u, m) = (-19.761147500555616, 0.9402376083201542);
    let (s, _, d) = ellipj(u, m);
    assert!((d * d + m * s * s - 1.0).abs() <= f64::EPSILON);
    for (u, m) in [(-14.64, 0.952), (3.3, 0.999_999), (7.9, 0.97)] {
        let (s, c, d) = ellipj(u, m);
        assert!(
            (d * d + m * s * s - 1.0).abs() <= 2.0 * f64::EPSILON,
            "u={u} m={m}"
        );
        assert!((s * s + c * c - 1.0).abs() <= 2.0 * f64::EPSILON);
    }
}

/// Precisão de dn perto de m = 1 por oráculo analítico (DLMF 22.5.2): em u = K/2,
/// dn = √k′ e cn = √(k′/(1 + k′)), com k′ = √(1 − m). A forma 1 − m·sn² perderia ~|log₁₀ k′²|
/// dígitos por cancelamento; (1 − m) + m·cn² não perde. Para 1 − m ≲ 1e-10 o limite passa a
/// ser a precisão de φ na AGM descendente (erro 1,8e-11 em 1 − m = 1e-12; ver F042.md).
#[test]
fn ellipj_dn_preciso_perto_de_m_1() {
    for kp2 in [1e-4f64, 1e-6, 1e-8] {
        let m = 1.0 - kp2;
        let kp = (1.0 - m).sqrt();
        let (k, _) = ellipke(m);
        let (_, c, d) = ellipj(0.5 * k, m);
        let (c_ref, d_ref) = ((kp / (1.0 + kp)).sqrt(), kp.sqrt());
        assert!(
            (d - d_ref).abs() <= 1e-12 * d_ref,
            "m = 1 − {kp2:e}: dn {d:e} vs {d_ref:e}"
        );
        assert!(
            (c - c_ref).abs() <= 1e-12 * c_ref,
            "m = 1 − {kp2:e}: cn {c:e} vs {c_ref:e}"
        );
    }
}

/// Deslocamento de quarto de período (DLMF 22.4.3): dn(K − v)·dn(v) = k′. Perto de u = K,
/// sn ≈ 1 e 1 − m·sn² cancelaria (erro ~ε/k′², até 1e-5 em 1 − m = 1e-12); a forma
/// (1 − m) + m·cn² fica em ~1e-12. Distingue as duas fórmulas de dn.
#[test]
fn ellipj_dn_por_deslocamento_de_quarto_de_periodo() {
    for kp2 in [1e-6f64, 1e-8, 1e-10] {
        let m = 1.0 - kp2;
        let kp = (1.0 - m).sqrt();
        let (k, _) = ellipke(m);
        for f in [0.001, 0.01] {
            let v = f * k;
            let (_, _, d) = ellipj(k - v, m);
            let (_, _, dv) = ellipj(v, m);
            assert!((d * dv / kp - 1.0).abs() <= 5e-12, "m = 1 − {kp2:e}, v = {f}·K");
        }
    }
}

#[test]
fn hankel_bordas() {
    use HankelKind::{First, Second};
    for k in [First, Second] {
        for s in [false, true] {
            let h = besselh(1.0, k, -1.0, s);
            assert!(h.re.is_nan() && h.im.is_nan());
            let h = besselh(NAN, k, 1.0, s);
            assert!(h.re.is_nan() && h.im.is_nan());
            let h = besselh(0.5, k, INF, s);
            assert_eq!((h.re, h.im), (0.0, 0.0));
        }
    }
    // x = 0: J₀(0) = 1, Y₀(0) = −Inf → H⁽¹⁾ = 1 − i·Inf, H⁽²⁾ = 1 + i·Inf (com ou sem escala).
    for s in [false, true] {
        let h = besselh(0.0, First, 0.0, s);
        assert_eq!((h.re, h.im), (1.0, f64::NEG_INFINITY));
        let h = besselh(0.0, Second, 0.0, s);
        assert_eq!((h.re, h.im), (1.0, INF));
        let h = besselh(2.0, First, 0.0, s);
        assert_eq!((h.re, h.im), (0.0, f64::NEG_INFINITY));
    }
    // Sem escala, as partes são exatamente J e ±Y.
    for (nu, x) in [(0.0, 1.0), (2.3, 7.5), (-1.5, 0.2), (-3.0, 40.0)] {
        let h1 = besselh(nu, First, x, false);
        let h2 = besselh(nu, Second, x, false);
        assert_eq!((h1.re, h1.im), (besselj(nu, x), bessely(nu, x)));
        assert_eq!((h2.re, h2.im), (besselj(nu, x), -bessely(nu, x)));
    }
    // Ordem ½: H⁽¹⁾·e^{−ix} = −i·√(2/(πx)) (DLMF 10.16.1).
    let x = 3.0;
    let h = besselh(0.5, First, x, true);
    let amp = (2.0 / (std::f64::consts::PI * x)).sqrt();
    assert!(h.re.abs() < 1e-15 && (h.im + amp).abs() < 1e-15, "{h}");
}
