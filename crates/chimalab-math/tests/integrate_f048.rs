//! F048: integração numérica. Oráculos: integrais em forma fechada, Γ(s) = ∫₀^∞ x^{s−1}e^{−x}dx
//! contra `gamma` (implementação independente, validada contra mpmath), representação integral de
//! Bessel J_n (DLMF 10.9.2) contra `besselj`, e a fórmula exata do erro da regra trapezoidal em
//! polinômios.

use chimalab_math::integrate::*;
use chimalab_math::special::{besselj, gamma};
use num_complex::Complex64;
use std::f64::consts::PI;

fn tight() -> IntegralOptions {
    IntegralOptions {
        abs_tol: 1e-14,
        rel_tol: 1e-13,
        ..Default::default()
    }
}

fn rel(got: f64, want: f64) -> f64 {
    if got == want {
        0.0
    } else {
        (got - want).abs() / want.abs().max(1e-300)
    }
}

#[test]
fn ca1_gaussiana_na_reta_toda() {
    let r = integral(
        |x: f64| (-x * x).exp(),
        f64::NEG_INFINITY,
        f64::INFINITY,
        &IntegralOptions::default(),
    )
    .unwrap();
    assert!(r.warning.is_none());
    assert!(rel(r.value, PI.sqrt()) < 1e-6, "{}", r.value);
    let r = integral(
        |x: f64| (-x * x).exp(),
        f64::NEG_INFINITY,
        f64::INFINITY,
        &tight(),
    )
    .unwrap();
    assert!(
        rel(r.value, PI.sqrt()) < 1e-14,
        "{} ({:e})",
        r.value,
        rel(r.value, PI.sqrt())
    );
}

#[test]
fn ca2_trapz_em_dados_amostrados() {
    // x² em [0, 1] com N trechos iguais: trapz = 1/3 + 1/(6N²) exatamente (erro da regra).
    for n in [1usize, 4, 10, 1000] {
        let y: Vec<f64> = (0..=n).map(|k| (k as f64 / n as f64).powi(2)).collect();
        let (v, s) = trapz(&y, &[n + 1], Spacing::Uniform(1.0 / n as f64), 0).unwrap();
        let want = 1.0 / 3.0 + 1.0 / (6.0 * (n * n) as f64);
        assert_eq!(s, vec![1]);
        assert!(rel(v[0], want) < 1e-14, "n = {n}: {} {want}", v[0]);
    }
    // Abscissas não uniformes: exata para funções afins.
    let x = [0.0, 0.1, 0.5, 0.55, 2.0];
    let y: Vec<f64> = x.iter().map(|t| 3.0 * t - 1.0).collect();
    let (v, _) = trapz(&y, &[5], Spacing::Points(&x), 0).unwrap();
    assert!((v[0] - (1.5 * 4.0 - 2.0)).abs() < 1e-15);
}

/// (descrição, integrando, a, b, valor exato, pontos de quebra)
type Caso = (&'static str, fn(f64) -> f64, f64, f64, f64, &'static [f64]);

fn casos() -> Vec<Caso> {
    let inf = f64::INFINITY;
    vec![
        ("x^20", |x| x.powi(20), 0.0, 1.0, 1.0 / 21.0, &[]),
        ("ln x (singular)", |x| x.ln(), 0.0, 1.0, -1.0, &[]),
        ("1/√x (singular)", |x| 1.0 / x.sqrt(), 0.0, 1.0, 2.0, &[]),
        ("√(1−x²)", |x| (1.0 - x * x).sqrt(), -1.0, 1.0, PI / 2.0, &[]),
        ("e^−x em [0, ∞)", |x| (-x).exp(), 0.0, inf, 1.0, &[]),
        ("1/x² em [1, ∞)", |x| 1.0 / (x * x), 1.0, inf, 1.0, &[]),
        ("e^x em (−∞, 0]", |x| x.exp(), -inf, 0.0, 1.0, &[]),
        ("1/(1+x²) na reta", |x| 1.0 / (1.0 + x * x), -inf, inf, PI, &[]),
        ("sen em [0, π]", |x| x.sin(), 0.0, PI, 2.0, &[]),
        (
            "cos²(10x) em [0, 2π]",
            |x| (10.0 * x).cos().powi(2),
            0.0,
            2.0 * PI,
            PI,
            &[],
        ),
        ("|x| com quebra em 0", |x| x.abs(), -1.0, 1.0, 1.0, &[0.0]),
        (
            "degrau com quebra",
            |x| if x < 0.3 { 1.0 } else { 2.0 },
            0.0,
            1.0,
            1.7,
            &[0.3],
        ),
        (
            "e^−|x| com quebra, na reta",
            |x| (-x.abs()).exp(),
            -inf,
            inf,
            2.0,
            &[0.0],
        ),
        (
            "1/√|x−0.5| singular interior",
            |x| 1.0 / (x - 0.5).abs().sqrt(),
            0.0,
            1.0,
            2.0 * 2f64.sqrt(),
            &[0.5],
        ),
        (
            "x e^−x² em (−∞, ∞) (ímpar)",
            |x| x * (-x * x).exp(),
            -inf,
            inf,
            0.0,
            &[],
        ),
    ]
}

#[test]
fn integrais_em_forma_fechada() {
    for (nome, f, a, b, want, wp) in casos() {
        for opts in [IntegralOptions::default(), tight()] {
            let opts = IntegralOptions {
                waypoints: wp.to_vec(),
                ..opts
            };
            let r = integral(f, a, b, &opts).unwrap();
            let err = (r.value - want).abs();
            let tol = opts.abs_tol.max(opts.rel_tol * want.abs());
            eprintln!(
                "{nome}: erro {err:e} (cota {:e}, tol {tol:e}, {} avaliações)",
                r.error_bound, r.evaluations
            );
            // Tolerância pedida atendida (com folga para o arredondamento da soma) e estimativa de
            // erro coerente (até 2×: em singularidades o arredondamento de x pesa).
            assert!(r.warning.is_none(), "{nome}: {:?}", r.warning);
            assert!(
                err <= tol + 4e-16 * want.abs().max(1.0),
                "{nome}: erro {err:e} > {tol:e}"
            );
            assert!(
                err <= 2.0 * r.error_bound + 4e-16 * want.abs().max(1.0),
                "{nome}: cota {:e} < erro {err:e}",
                r.error_bound
            );
        }
    }
}

#[test]
fn gama_e_bessel_por_integrais() {
    // Γ(s) = ∫₀^∞ x^{s−1} e^{−x} dx: implementações independentes.
    for s in [0.3, 1.0, 2.5, 7.0, 15.5] {
        let r = integral(
            |x: f64| x.powf(s - 1.0) * (-x).exp(),
            0.0,
            f64::INFINITY,
            &tight(),
        )
        .unwrap();
        assert!(
            rel(r.value, gamma(s)) < 2e-13,
            "s = {s}: {} {}",
            r.value,
            gamma(s)
        );
    }
    // J_n(x) = (1/π) ∫₀^π cos(nθ − x sen θ) dθ (DLMF 10.9.2).
    for (n, x) in [(0, 1.0), (1, 5.5), (3, 10.0), (7, 2.0)] {
        let nn = n as f64;
        let r = integral(|t: f64| (nn * t - x * t.sin()).cos(), 0.0, PI, &tight()).unwrap();
        let j = besselj(nn, x);
        assert!(
            (r.value / PI - j).abs() < 1e-14,
            "J_{n}({x}): {} {j}",
            r.value / PI
        );
    }
}

#[test]
fn integrandos_complexos() {
    let r = integral(|x: f64| Complex64::new(0.0, x).exp(), 0.0, PI, &tight()).unwrap();
    assert!((r.value - Complex64::new(0.0, 2.0)).norm() < 1e-14, "{}", r.value);
    let w = Complex64::new(1.0, 1.0);
    let r = integral(|x: f64| (-w * x).exp(), 0.0, f64::INFINITY, &tight()).unwrap();
    assert!((r.value - w.inv()).norm() < 1e-14, "{}", r.value);
}

#[test]
fn orientacao_e_limites_iguais() {
    let f = |x: f64| x.exp() * x.sin();
    let r1 = integral(f, 0.3, 2.0, &tight()).unwrap();
    let r2 = integral(f, 2.0, 0.3, &tight()).unwrap();
    assert_eq!(r2.value, -r1.value);
    let r3 = integral(
        |x: f64| (-x * x).exp(),
        f64::INFINITY,
        f64::NEG_INFINITY,
        &tight(),
    )
    .unwrap();
    assert!(rel(r3.value, -PI.sqrt()) < 1e-14);
    for (a, b) in [(1.0, 1.0), (f64::INFINITY, f64::INFINITY), (-0.0, 0.0)] {
        let r = integral(|_x: f64| 1.0, a, b, &IntegralOptions::default()).unwrap();
        assert_eq!(
            (r.value, r.error_bound, r.evaluations, r.warning),
            (0.0, 0.0, 0, None)
        );
    }
}

#[test]
fn erros_e_avisos() {
    let d = IntegralOptions::default();
    let id = |r: Result<IntegralResult<f64>, IntegrateError>| r.unwrap_err().id();
    assert_eq!(
        id(integral(|x| x, f64::NAN, 1.0, &d)),
        "chimalab:integral:limitesInvalidos"
    );
    assert_eq!(
        id(integral(|x| x, 0.0, f64::NAN, &d)),
        "chimalab:integral:limitesInvalidos"
    );
    for opts in [
        IntegralOptions {
            abs_tol: -1.0,
            ..d.clone()
        },
        IntegralOptions {
            rel_tol: f64::NAN,
            ..d.clone()
        },
        IntegralOptions {
            abs_tol: f64::INFINITY,
            ..d.clone()
        },
        IntegralOptions {
            max_intervals: 0,
            ..d.clone()
        },
    ] {
        assert_eq!(
            id(integral(|x| x, 0.0, 1.0, &opts)),
            "chimalab:integral:toleranciaInvalida"
        );
    }
    let opts = IntegralOptions {
        waypoints: vec![f64::NAN],
        ..d.clone()
    };
    assert_eq!(
        id(integral(|x| x, 0.0, 1.0, &opts)),
        "chimalab:integral:limitesInvalidos"
    );
    let opts = IntegralOptions {
        waypoints: vec![f64::INFINITY],
        ..d.clone()
    };
    assert_eq!(
        id(integral(|x| x, 0.0, 1.0, &opts)),
        "chimalab:integral:limitesInvalidos"
    );
    // Pontos de quebra nos próprios extremos não criam trechos vazios (nem aviso).
    let opts = IntegralOptions {
        waypoints: vec![0.0, 1.0],
        ..tight()
    };
    let r = integral(|x: f64| x.exp(), 0.0, 1.0, &opts).unwrap();
    assert!(
        r.warning.is_none() && rel(r.value, 1f64.exp() - 1.0) < 1e-15,
        "{r:?}"
    );
    // Pontos de quebra fora de (a, b) ou repetidos são ignorados.
    let opts = IntegralOptions {
        waypoints: vec![-3.0, 0.5, 0.5, 7.0],
        ..tight()
    };
    assert!(
        rel(
            integral(|x: f64| x * x, 0.0, 1.0, &opts).unwrap().value,
            1.0 / 3.0
        ) < 1e-15
    );

    // Integrando com NaN: valor NaN e aviso.
    let r = integral(|x: f64| if x > 0.5 { f64::NAN } else { x }, 0.0, 1.0, &d).unwrap();
    assert!(r.value.is_nan() && r.warning == Some(IntegralWarning::NonFinite));
    assert_eq!(
        IntegralWarning::NonFinite.id(),
        "chimalab:integral:valorNaoFinito"
    );
    // Não integrável (1/x em [0, 1]): nunca "converge" em silêncio.
    let r = integral(|x: f64| 1.0 / x, 0.0, 1.0, &d).unwrap();
    assert!(r.warning.is_some(), "{:?}", r);
    let r = integral(|x: f64| 1.0 / x, 1.0, f64::INFINITY, &d).unwrap();
    assert!(r.warning.is_some(), "{:?}", r);
    assert_eq!(
        IntegralWarning::MaxIntervals.id(),
        "chimalab:integral:limiteDeIntervalos"
    );
    // Oscilação rápida com poucos subintervalos permitidos.
    let opts = IntegralOptions {
        max_intervals: 12,
        ..tight()
    };
    let r = integral(|x: f64| (200.0 * x).sin(), 0.0, 10.0, &opts).unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::MaxIntervals));
    assert!(r.evaluations > 0 && r.error_bound > 0.0);
    // Degrau em 1/3 sem ponto de quebra e tolerância zero: o subintervalo que contém o salto é
    // dividido até ter 1 ulp de largura e não poder mais ser dividido.
    let opts = IntegralOptions {
        abs_tol: 0.0,
        rel_tol: 0.0,
        max_intervals: 100_000,
        ..d.clone()
    };
    let r = integral(|x: f64| if x < 1.0 / 3.0 { 0.0 } else { 1.0 }, 0.0, 1.0, &opts).unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::MinimumStep), "{r:?}");
    assert!((r.value - 2.0 / 3.0).abs() < 1e-14 && r.evaluations < 100_000 * 15);
    assert_eq!(IntegralWarning::MinimumStep.id(), "chimalab:integral:passoMinimo");
    let e = IntegrateError::ShapeMismatch {
        expected: 3,
        actual: 2,
    };
    assert_eq!(e.id(), "chimalab:integral:formaIncompativel");
    assert!(e.to_string().contains('3'));
    assert!(IntegrateError::InvalidLimits("x").to_string().contains('x'));
    assert!(IntegrateError::InvalidTolerance("y").to_string().contains('y'));
}

#[test]
fn integrais_duplas_e_triplas() {
    let o = tight();
    // ∫₀¹∫₀ˣ xy dy dx = 1/8.
    let r = integral2(|x: f64, y: f64| x * y, 0.0, 1.0, |_| 0.0, |x| x, &o).unwrap();
    assert!(rel(r.value, 0.125) < 1e-13 && r.warning.is_none(), "{}", r.value);
    // Área do círculo unitário.
    let r = integral2(
        |_x: f64, _y: f64| 1.0,
        -1.0,
        1.0,
        |x| -(1.0 - x * x).sqrt(),
        |x| (1.0 - x * x).sqrt(),
        &o,
    )
    .unwrap();
    assert!(rel(r.value, PI) < 1e-12, "{}", r.value);
    // e^{−x²−y²} no plano todo = π.
    let inf = f64::INFINITY;
    let r = integral2(
        |x: f64, y: f64| (-x * x - y * y).exp(),
        -inf,
        inf,
        |_| -inf,
        |_| inf,
        &o,
    )
    .unwrap();
    assert!(rel(r.value, PI) < 1e-12, "{}", r.value);
    // Volume da bola unitária e ∫∫∫ xyz no cubo.
    let r = integral3(
        |_x: f64, _y: f64, _z: f64| 1.0,
        -1.0,
        1.0,
        |x| -(1.0 - x * x).sqrt(),
        |x| (1.0 - x * x).sqrt(),
        |x, y| -(1.0 - x * x - y * y).max(0.0).sqrt(),
        |x, y| (1.0 - x * x - y * y).max(0.0).sqrt(),
        &IntegralOptions::default(),
    )
    .unwrap();
    assert!(rel(r.value, 4.0 * PI / 3.0) < 1e-6, "{}", r.value);
    let r = integral3(
        |x: f64, y: f64, z: f64| x * y * z,
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        |_, _| 0.0,
        |_, _| 1.0,
        &o,
    )
    .unwrap();
    assert!(rel(r.value, 0.125) < 1e-13 && r.error_bound < 1e-10);
    assert!(r.evaluations >= 15 * 15 * 15);
    // Complexo: ∫₀¹∫₀¹ e^{i(x+y)} = (e^i − 1)²/i² ... = −(e^i − 1)².
    let r = integral2(
        |x: f64, y: f64| Complex64::new(0.0, x + y).exp(),
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &o,
    )
    .unwrap();
    let e1 = Complex64::new(0.0, 1.0).exp() - 1.0;
    assert!((r.value + e1 * e1).norm() < 1e-13, "{}", r.value);
    // Erro de limite na integral interna: devolvido como erro.
    let r = integral2(
        |x: f64, y: f64| x + y,
        0.0,
        1.0,
        |x| if x > 0.5 { f64::NAN } else { 0.0 },
        |_| 1.0,
        &o,
    );
    assert_eq!(r.unwrap_err().id(), "chimalab:integral:limitesInvalidos");
    // Aviso de uma integral interna é propagado.
    let r = integral2(
        |_x: f64, y: f64| 1.0 / y,
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &IntegralOptions::default(),
    )
    .unwrap();
    assert!(r.warning.is_some());
}

/// Singularidades fortes no extremo: perto de 0 a mudança de variável resolve x até ~1e-32, e a
/// massa de x^{−α} abaixo disso vale 1e-32^{1−α}/(1−α). Quando ela passa da tolerância, o
/// refinamento chega ao extremo e o resultado vem com `MinimumStep` (regressão: antes o valor
/// errado de x^{−0,9} saía sem aviso).
#[test]
fn singularidades_fortes_no_extremo() {
    let r = integral(|x: f64| x.powf(-0.75), 0.0, 1.0, &IntegralOptions::default()).unwrap();
    assert!(r.warning.is_none() && (r.value - 4.0).abs() <= 4e-6, "{r:?}");
    let r = integral(|x: f64| x.powf(-0.75), 0.0, 1.0, &tight()).unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::MinimumStep), "{r:?}");
    assert!((r.value - 4.0).abs() < 1e-7);
    let r = integral(|x: f64| x.powf(-0.9), 0.0, 1.0, &IntegralOptions::default()).unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::MinimumStep), "{r:?}");
    assert!((r.value - 10.0).abs() < 2e-2);
}

#[test]
fn trapz_e_cumtrapz_em_arrays() {
    // Matriz 3×4 (column-major): integra colunas (dim 0) e linhas (dim 1).
    let y: Vec<f64> = (0..12).map(|k| k as f64).collect();
    let (v, s) = trapz(&y, &[3, 4], Spacing::Unit, 0).unwrap();
    assert_eq!(s, vec![1, 4]);
    assert_eq!(v, vec![2.0, 8.0, 14.0, 20.0]);
    let (v, s) = trapz(&y, &[3, 4], Spacing::Uniform(0.5), 1).unwrap();
    assert_eq!(s, vec![3, 1]);
    assert_eq!(v, vec![0.5 * 13.5, 0.5 * 16.5, 0.5 * 19.5]);
    let (c, s) = cumtrapz(&y, &[3, 4], Spacing::Points(&[0.0, 1.0, 3.0, 6.0]), 1).unwrap();
    assert_eq!(s, vec![3, 4]);
    // Linha 0: amostras 0, 3, 6, 9 em x = 0, 1, 3, 6.
    assert_eq!([c[0], c[3], c[6], c[9]], [0.0, 1.5, 10.5, 33.0]);
    // A última amostra da acumulada é o trapz.
    let (t, _) = trapz(&y, &[3, 4], Spacing::Points(&[0.0, 1.0, 3.0, 6.0]), 1).unwrap();
    assert_eq!([c[9], c[10], c[11]], [t[0], t[1], t[2]]);
    // Dimensão além das existentes: singleton → zero; acumulada também zero.
    let (v, s) = trapz(&y, &[3, 4], Spacing::Unit, 2).unwrap();
    assert_eq!((v, s), (vec![0.0; 12], vec![3, 4, 1]));
    let (c, s) = cumtrapz(&y, &[3, 4], Spacing::Unit, 3).unwrap();
    assert_eq!((c, s), (vec![0.0; 12], vec![3, 4]));
    // Dimensão vazia: zeros com as demais dimensões.
    let (v, s) = trapz::<f64>(&[], &[0, 3], Spacing::Unit, 0).unwrap();
    assert_eq!((v, s), (vec![0.0; 3], vec![1, 3]));
    // Complexos e NaN.
    let z = [Complex64::new(1.0, 2.0), Complex64::new(3.0, -1.0)];
    assert_eq!(
        trapz(&z, &[2], Spacing::Unit, 0).unwrap().0,
        vec![Complex64::new(2.0, 0.5)]
    );
    assert!(trapz(&[1.0, f64::NAN, 2.0], &[3], Spacing::Unit, 0).unwrap().0[0].is_nan());
    // Erros.
    assert_eq!(
        trapz(&y, &[5, 2], Spacing::Unit, 0).unwrap_err().id(),
        "chimalab:integral:formaIncompativel"
    );
    assert_eq!(
        cumtrapz(&y, &[3, 4], Spacing::Points(&[0.0, 1.0]), 1).unwrap_err(),
        IntegrateError::ShapeMismatch {
            expected: 4,
            actual: 2
        }
    );
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        // Linearidade e aditividade num ponto interior qualquer.
        #[test]
        fn linearidade_e_aditividade(a in -3.0f64..0.0, b in 0.5f64..4.0, c in 0.01f64..0.99, al in -2.0f64..2.0) {
            let f = |x: f64| (x * 1.7).sin() + 0.3 * x * x;
            let g = |x: f64| (-x * x).exp();
            let o = tight();
            let i = |h: &dyn Fn(f64) -> f64, lo, hi| integral(h, lo, hi, &o).unwrap().value;
            let fg = i(&|x| al * f(x) + g(x), a, b);
            prop_assert!((fg - (al * i(&f, a, b) + i(&g, a, b))).abs() < 1e-13);
            let m = a + c * (b - a);
            prop_assert!((i(&f, a, b) - (i(&f, a, m) + i(&f, m, b))).abs() < 1e-13);
        }

        // Polinômios de grau ≤ 6 em intervalos quaisquer (mudança de variável cúbica → grau 18,
        // ainda exato numa só regra 7–15).
        #[test]
        fn polinomios(c in proptest::collection::vec(-3.0f64..3.0, 7), a in -5.0f64..5.0, len in 0.1f64..5.0) {
            let b = a + len;
            let p = |x: f64| c.iter().rev().fold(0.0, |s, &k| s * x + k);
            let prim = |x: f64| c.iter().enumerate().rev().fold(0.0, |s, (k, &ck)| s * x + ck / (k + 1) as f64) * x;
            let want = prim(b) - prim(a);
            let r = integral(p, a, b, &tight()).unwrap();
            let escala: f64 = c.iter().enumerate().map(|(k, ck)| ck.abs() * a.abs().max(b.abs()).powi(k as i32 + 1)).sum();
            prop_assert!((r.value - want).abs() <= 1e-14 * escala.max(1.0), "{} {}", r.value, want);
        }

        // Arrays 3-D, todas as dimensões: contra uma soma escrita por índices (i, j, k).
        #[test]
        fn trapz_3d_por_indices(d in proptest::collection::vec(1usize..4, 3), dim in 0usize..3, h in 0.1f64..2.0) {
            let n: usize = d.iter().product();
            let y: Vec<f64> = (0..n).map(|k| ((k * 7919) % 101) as f64 - 50.0).collect();
            let at = |i: usize, j: usize, k: usize| y[i + d[0] * (j + d[1] * k)];
            let (t, ts) = trapz(&y, &d, Spacing::Uniform(h), dim).unwrap();
            let (c, _) = cumtrapz(&y, &d, Spacing::Uniform(h), dim).unwrap();
            let mut od = d.clone();
            od[dim] = 1;
            prop_assert_eq!(&ts, &od);
            for i in 0..d[0] {
                for j in 0..d[1] {
                    for k in 0..d[2] {
                        let mut idx = [i, j, k];
                        // Acumulada até a posição idx[dim].
                        let mut s = 0.0;
                        for m in 1..=idx[dim] {
                            let mut a = idx;
                            a[dim] = m - 1;
                            let mut b = idx;
                            b[dim] = m;
                            s += 0.5 * h * (at(a[0], a[1], a[2]) + at(b[0], b[1], b[2]));
                        }
                        prop_assert!((c[i + d[0] * (j + d[1] * k)] - s).abs() < 1e-12);
                        if idx[dim] == d[dim] - 1 {
                            idx[dim] = 0;
                            let o = idx[0] + od[0] * (idx[1] + od[1] * idx[2]);
                            prop_assert!((t[o] - s).abs() < 1e-12);
                        }
                    }
                }
            }
        }

        // cumtrapz: diferenças consecutivas são os trapézios; o último valor é o trapz.
        #[test]
        fn cumtrapz_consistente(y in proptest::collection::vec(-10.0f64..10.0, 2..40), h in 0.01f64..3.0) {
            let n = y.len();
            let (c, _) = cumtrapz(&y, &[n], Spacing::Uniform(h), 0).unwrap();
            let (t, _) = trapz(&y, &[n], Spacing::Uniform(h), 0).unwrap();
            prop_assert_eq!(c[0], 0.0);
            prop_assert_eq!(c[n - 1], t[0]);
            for k in 1..n {
                prop_assert!((c[k] - c[k - 1] - 0.5 * h * (y[k] + y[k - 1])).abs() <= 1e-12 * (1.0 + c[k].abs()));
            }
        }
    }
}

/// Medição: `cargo test --release -- --ignored desempenho_integral --nocapture`.
#[test]
#[ignore]
fn desempenho_integral() {
    let t = std::time::Instant::now();
    let mut n = 0;
    let mut acc = 0.0;
    for k in 0..1000 {
        let s = 1.0 + k as f64 * 1e-3;
        let r = integral(|x: f64| (s * x).sin() * (-x).exp(), 0.0, f64::INFINITY, &tight()).unwrap();
        n += r.evaluations;
        acc += r.value;
    }
    let dt = t.elapsed();
    eprintln!(
        "1000 integrais em [0, ∞) com tol 1e-13: {dt:?} ({:.1} µs cada, {} avaliações em média; {acc:.6})",
        dt.as_secs_f64() * 1e3,
        n / 1000
    );
}

// ================================================================ método por ladrilhos

#[test]
fn ladrilhos_contra_formas_fechadas() {
    use chimalab_math::integrate::{IntegralOptions, IntegralWarning, integral2, integral2_tiled};
    use chimalab_math::special::erf;
    let o = |a: f64, r: f64| IntegralOptions {
        abs_tol: a,
        rel_tol: r,
        ..Default::default()
    };
    // Polinômio: x⁵y⁶ vira grau ≤ 20 em u e v pela mudança cúbica (≤ 22, o grau de Kronrod).
    let r = integral2_tiled(
        |x, y| x.powi(5) * y.powi(6),
        0.0,
        1.0,
        |_| 0.0,
        |_| 2.0,
        &o(1e-14, 1e-14),
    )
    .unwrap();
    let ex = (1.0 / 6.0) * 2f64.powi(7) / 7.0;
    assert!((r.value - ex).abs() <= 4e-15 * ex, "{}", r.value - ex);
    // K é exato, mas a estimativa usa G (exato só até grau 13): subdivide até |K − G| cair.
    assert_eq!(r.warning, None);
    // Oscilatório: ∫∫ cos(x + y) em [0, π]² = −4.
    let r = integral2_tiled(
        |x, y| (x + y).cos(),
        0.0,
        std::f64::consts::PI,
        |_| 0.0,
        |_| std::f64::consts::PI,
        &o(1e-12, 1e-12),
    )
    .unwrap();
    assert!((r.value + 4.0).abs() <= 1e-12, "{}", r.value + 4.0);
    // Pico gaussiano estreito: forma fechada pela erf (F042).
    let pico = |x: f64, y: f64| (-100.0 * ((x - 0.3).powi(2) + (y - 0.6).powi(2))).exp();
    let ex = std::f64::consts::PI / 400.0 * (erf(7.0) + erf(3.0)) * (erf(4.0) + erf(6.0));
    let rt = integral2_tiled(pico, 0.0, 1.0, |_| 0.0, |_| 1.0, &o(1e-13, 1e-10)).unwrap();
    assert!((rt.value - ex).abs() <= 1e-10 * ex, "{}", (rt.value - ex) / ex);
    assert!(rt.error_bound >= (rt.value - ex).abs() || (rt.value - ex).abs() <= 1e-15);
    // O iterado concorda (caminho independente).
    let ri = integral2(pico, 0.0, 1.0, |_| 0.0, |_| 1.0, &o(1e-13, 1e-10)).unwrap();
    assert!((ri.value - rt.value).abs() <= 2e-10 * ex);
    println!(
        "pico: ladrilhos {} avaliações, iterado {}",
        rt.evaluations, ri.evaluations
    );
    // Limites variáveis: área do disco unitário (raiz nas bordas do jacobiano) e ∫∫ x·y no
    // triângulo 0 ≤ y ≤ x ≤ 1 (= 1/8).
    let r = integral2_tiled(
        |_, _| 1.0,
        -1.0,
        1.0,
        |x| -(1.0 - x * x).sqrt(),
        |x| (1.0 - x * x).sqrt(),
        &o(1e-10, 1e-8),
    )
    .unwrap();
    assert!(
        (r.value - std::f64::consts::PI).abs() <= 1e-7,
        "{}",
        r.value - std::f64::consts::PI
    );
    let r = integral2_tiled(|x, y| x * y, 0.0, 1.0, |_| 0.0, |x| x, &o(1e-14, 1e-14)).unwrap();
    assert!((r.value - 0.125).abs() <= 1e-15);
    // Complexo: ∫∫ e^{i(x+y)} em [0, π]² = (e^{iπ} − 1)²/(i²) = (−2)²·(−1) = −4.
    let r = integral2_tiled(
        |x, y| Complex64::new(0.0, x + y).exp(),
        0.0,
        std::f64::consts::PI,
        |_| 0.0,
        |_| std::f64::consts::PI,
        &o(1e-12, 1e-12),
    )
    .unwrap();
    assert!(
        (r.value - Complex64::new(-4.0, 0.0)).norm() <= 1e-11,
        "{}",
        r.value
    );
    // Orientação: limites invertidos trocam o sinal.
    let a = integral2_tiled(|x, y| x + y, 0.0, 1.0, |_| 0.0, |_| 1.0, &o(1e-14, 1e-14))
        .unwrap()
        .value;
    let b = integral2_tiled(|x, y| x + y, 1.0, 0.0, |_| 0.0, |_| 1.0, &o(1e-14, 1e-14))
        .unwrap()
        .value;
    // Mesmo valor com sinal trocado, salvo a ordem das somas (1 ulp medido).
    assert!((a + b).abs() <= 4.0 * f64::EPSILON, "{a} {b}");
    // Avisos: descontinuidade com tolerância impossível (máximo de ladrilhos); NaN.
    let r = integral2_tiled(
        |x, y| if x + y < 0.7 { 1.0 } else { 0.0 },
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &IntegralOptions {
            abs_tol: 0.0,
            rel_tol: 1e-15,
            max_intervals: 40,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::MaxIntervals));
    assert!((r.value - 0.245).abs() <= 1e-2);
    let r = integral2_tiled(
        |_, _| f64::NAN,
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &IntegralOptions::default(),
    )
    .unwrap();
    assert_eq!(r.warning, Some(IntegralWarning::NonFinite));
    // Limite de resolução: integrando singular em um canto interior com tolerância nula.
    let r = integral2_tiled(
        |x, y| 1.0 / ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt().max(1e-300),
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &IntegralOptions {
            abs_tol: 0.0,
            rel_tol: 0.0,
            max_intervals: 100_000,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r.warning.is_some());
    // Erros.
    let e = |r: Result<
        chimalab_math::integrate::IntegralResult<f64>,
        chimalab_math::integrate::IntegrateError,
    >| r.unwrap_err().id();
    assert_eq!(
        e(integral2_tiled(
            |_, _| 1.0,
            0.0,
            f64::INFINITY,
            |_| 0.0,
            |_| 1.0,
            &IntegralOptions::default()
        )),
        "chimalab:integral:limitesInvalidos"
    );
    assert_eq!(
        e(integral2_tiled(
            |_, _| 1.0,
            f64::NAN,
            1.0,
            |_| 0.0,
            |_| 1.0,
            &IntegralOptions::default()
        )),
        "chimalab:integral:limitesInvalidos"
    );
    assert_eq!(
        e(integral2_tiled(
            |_, _| 1.0,
            0.0,
            1.0,
            |_| 0.0,
            |_| f64::INFINITY,
            &IntegralOptions::default()
        )),
        "chimalab:integral:limitesInvalidos"
    );
    assert_eq!(
        e(integral2_tiled(
            |_, _| 1.0,
            0.0,
            1.0,
            |_| 0.0,
            |_| 1.0,
            &o(-1.0, 1e-6)
        )),
        "chimalab:integral:toleranciaInvalida"
    );
    assert_eq!(
        e(integral2_tiled(
            |_, _| 1.0,
            0.0,
            1.0,
            |_| 0.0,
            |_| 1.0,
            &o(1e-10, f64::NAN)
        )),
        "chimalab:integral:toleranciaInvalida"
    );
}

/// Ladrilhos: contagens exatas e fronteiras de controle (mutação). Integrandos racionais, sem
/// libm, para que as contagens não dependam da plataforma. Cada ladrilho custa 15 × 15 = 225
/// avaliações; o início tem 4 ladrilhos e cada divisão acrescenta 4.
#[test]
fn ladrilhos_contagens_e_fronteiras() {
    use chimalab_math::integrate::{IntegralOptions, IntegralWarning, integral2_tiled};
    let opc = |a: f64, r: f64, m: usize| IntegralOptions {
        abs_tol: a,
        rel_tol: r,
        max_intervals: m,
        ..Default::default()
    };
    // Limite inferior não nulo: ∫₀¹ ∫ₓ^{1+x} (x + y) dy dx = ∫₀¹ (2x + ½) dx = 3/2. Polinomial:
    // converge nos 4 ladrilhos iniciais.
    let r = integral2_tiled(
        |x, y| x + y,
        0.0,
        1.0,
        |x| x,
        |x| 1.0 + x,
        &opc(1e-13, 1e-13, 650),
    )
    .unwrap();
    assert!((r.value - 1.5).abs() <= 4.0 * f64::EPSILON, "{}", r.value - 1.5);
    assert_eq!((r.evaluations, r.warning), (900, None));
    // Tolerância relativa com |Q| grande: 10⁶/(1 + x + y), abs_tol = 0.
    // ∫∫ 1/(1+x+y) em [0,1]² = 3 ln 3 − 4 ln 2.
    let ex = 1e6 * (3.0 * 3f64.ln() - 4.0 * 2f64.ln());
    let r = integral2_tiled(
        |x, y| 1e6 / (1.0 + x + y),
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &opc(0.0, 1e-9, 650),
    )
    .unwrap();
    assert_eq!(r.warning, None);
    assert!((r.value - ex).abs() <= 1e-9 * ex, "{}", (r.value - ex) / ex);
    assert!(r.error_bound <= 1e-9 * r.value.abs());
    let n_rel = r.evaluations;
    // Mesma conta com tolerância 10⁻⁹ absoluta (|Q| ≈ 10⁶) exige muito mais.
    let r = integral2_tiled(
        |x, y| 1e6 / (1.0 + x + y),
        0.0,
        1.0,
        |_| 0.0,
        |_| 1.0,
        &opc(1e-9, 0.0, 650),
    )
    .unwrap();
    assert!(r.evaluations > n_rel, "{} {}", r.evaluations, n_rel);
    // Máximo de ladrilhos: tolerância nula nunca é alcançada com uma descontinuidade.
    let degrau = |x: f64, y: f64| if 3.0 * (x + y) < 2.0 { 1.0 } else { 0.0 };
    for (m, aval) in [(1, 900), (4, 900), (6, 900), (7, 1800), (9, 1800), (10, 2700)] {
        let r = integral2_tiled(degrau, 0.0, 1.0, |_| 0.0, |_| 1.0, &opc(0.0, 0.0, m)).unwrap();
        assert_eq!(
            (r.evaluations, r.warning),
            (aval, Some(IntegralWarning::MaxIntervals)),
            "max = {m}"
        );
    }
}

/// Limite de resolução: singularidade pontual 1/r num ponto interior fora da malha diádica, com
/// tolerância nula. O ladrilho que contém o ponto é dividido até o ponto médio coincidir com uma
/// borda em ponto flutuante; aí o resultado sai com `MinimumStep` e o valor já está convergido.
/// ∫∫ 1/r em [0,1]² com r medido a partir de (a, b): soma das quatro parcelas retangulares
/// F(p, q) = p·asinh(q/p) + q·asinh(p/q). Perto do limite os nós podem cair exatamente no ponto
/// singular; 1/r é limitado a 10²⁰ para que isso não vire `NonFinite` (num ladrilho de lado ~10⁻¹⁶ a
/// contribuição do corte é ~10⁻¹²).
#[test]
fn ladrilhos_passo_minimo() {
    use chimalab_math::integrate::{IntegralOptions, IntegralWarning, integral2_tiled};
    let opc = IntegralOptions {
        abs_tol: 0.0,
        rel_tol: 0.0,
        max_intervals: 100_000,
        ..Default::default()
    };
    let canto = |p: f64, q: f64| p * (q / p).asinh() + q * (p / q).asinh();
    for (a, b, n) in [
        (0.3, 0.7, 499_500),
        (0.71, 0.29, 624_600),
        (0.37, 0.6, 567_900),
        (0.62, 0.41, 263_700),
        (1.0 / 3.0, 0.6, 540_900),
        // Achados por busca: o limite de resolução cai do lado s0 = sm (primeiro) e tm = t1
        // (segundo), que os pontos acima não exercitam.
        (0.740_432_769_517_39, 0.275_964_440_278_307_13, 258_300),
        (0.128_107_776_016_960_72, 0.581_476_196_109_341_5, 359_100),
    ] {
        let r = integral2_tiled(
            |x, y| 1.0 / ((x - a) * (x - a) + (y - b) * (y - b)).max(1e-40).sqrt(),
            0.0,
            1.0,
            |_| 0.0,
            |_| 1.0,
            &opc,
        )
        .unwrap();
        let ex = canto(a, b) + canto(1.0 - a, b) + canto(a, 1.0 - b) + canto(1.0 - a, 1.0 - b);
        assert_eq!(r.warning, Some(IntegralWarning::MinimumStep), "({a}, {b})");
        assert!(
            (r.value - ex).abs() <= 1e-9 * ex,
            "({a}, {b}): {}",
            (r.value - ex) / ex
        );
        assert_eq!(r.evaluations, n, "({a}, {b})");
    }
}
