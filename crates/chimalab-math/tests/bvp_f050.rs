//! F050 — problemas de valor de contorno (colocação de Lobatto IIIA, Hermite–Simpson).
//!
//! Oráculos: soluções exatas (oscilador; Bratu com λ = 1, solução fechada de Bratu/Gelfand;
//! camada-limite linear).

use chimalab_math::ode::{BvpOptions, bvp};

fn erro_max(s: &chimalab_math::ode::BvpSolution, exata: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    (0..=400)
        .map(|k| {
            let x = a + (b - a) * k as f64 / 400.0;
            (s.eval(x).unwrap()[0] - exata(x)).abs()
        })
        .fold(0.0, f64::max)
}

#[test]
fn ca1_bvp_de_referencia_com_erro_menor_que_tol() {
    // y'' = −y, y(0) = 0, y(π/2) = 1: y = sen x.
    let pi2 = std::f64::consts::FRAC_PI_2;
    let malha: Vec<f64> = (0..=5).map(|k| pi2 * k as f64 / 5.0).collect();
    for tol in [1e-4, 1e-7] {
        let o = BvpOptions {
            rel_tol: tol,
            abs_tol: tol * 1e-3,
            ..Default::default()
        };
        let s = bvp(
            |_, y, d| {
                d[0] = y[1];
                d[1] = -y[0];
            },
            |ya, yb, r| {
                r[0] = ya[0];
                r[1] = yb[0] - 1.0;
            },
            &malha,
            |x, y| {
                y[0] = x / pi2;
                y[1] = 1.0 / pi2;
            },
            2,
            &o,
        )
        .unwrap();
        assert!(s.converged);
        let e = erro_max(&s, f64::sin, 0.0, pi2);
        println!("BVP seno tol {tol:e}: erro {e:e}, {} pontos", s.x.len());
        assert!(e < tol, "tol {tol}: {e:e}");
    }
    // Bratu y'' + e^y = 0, y(0) = y(1) = 0 (ramo inferior): y = −2 ln(cosh((x − ½)θ/2)/cosh(θ/4)),
    // com θ = √2·cosh(θ/4).
    let mut theta = 1.0f64;
    for _ in 0..100 {
        theta = 2f64.sqrt() * (theta / 4.0).cosh();
    }
    let bratu = move |x: f64| -2.0 * (((x - 0.5) * theta / 2.0).cosh() / (theta / 4.0).cosh()).ln();
    let malha: Vec<f64> = (0..=10).map(|k| k as f64 / 10.0).collect();
    let o = BvpOptions {
        rel_tol: 1e-8,
        abs_tol: 1e-11,
        ..Default::default()
    };
    let s = bvp(
        |_, y, d| {
            d[0] = y[1];
            d[1] = -y[0].exp();
        },
        |ya, yb, r| {
            r[0] = ya[0];
            r[1] = yb[0];
        },
        &malha,
        |_, y| {
            y[0] = 0.0;
            y[1] = 0.0;
        },
        2,
        &o,
    )
    .unwrap();
    assert!(s.converged);
    let e = erro_max(&s, bratu, 0.0, 1.0);
    println!(
        "BVP Bratu: erro {e:e}, {} pontos, {} Newton",
        s.x.len(),
        s.newton_iterations
    );
    assert!(e < 1e-8, "{e:e}");
}

/// Camada-limite: ε y'' + y' = 0, y(0) = 0, y(1) = 1, ε = 0,01:
/// y = (1 − e^{−x/ε})/(1 − e^{−1/ε}). Exige refinar a malha perto de x = 0.
#[test]
fn camada_limite_exige_refino() {
    let eps = 0.01;
    let exata = move |x: f64| (1.0 - (-x / eps).exp()) / (1.0 - (-1.0 / eps).exp());
    let malha: Vec<f64> = (0..=10).map(|k| k as f64 / 10.0).collect();
    let o = BvpOptions {
        rel_tol: 1e-6,
        abs_tol: 1e-9,
        ..Default::default()
    };
    let s = bvp(
        move |_, y, d| {
            d[0] = y[1];
            d[1] = -y[1] / eps;
        },
        |ya, yb, r| {
            r[0] = ya[0];
            r[1] = yb[0] - 1.0;
        },
        &malha,
        |x, y| {
            y[0] = x;
            y[1] = 1.0;
        },
        2,
        &o,
    )
    .unwrap();
    assert!(s.converged);
    assert!(s.x.len() > 11, "a malha deve ser refinada");
    // Densidade de pontos na camada (x < 0,1) pelo menos 4× a média.
    let perto = s.x.iter().filter(|&&x| x < 0.1).count();
    assert!(
        perto as f64 / 0.1 >= 4.0 * s.x.len() as f64,
        "{perto} de {}",
        s.x.len()
    );
    let e = erro_max(&s, exata, 0.0, 1.0);
    println!("BVP camada: erro {e:e}, {} pontos", s.x.len());
    assert!(e < 1e-5, "{e:e}");
}

#[test]
fn bvp_bordas_e_erros() {
    let f = |_: f64, y: &[f64], d: &mut [f64]| {
        d[0] = y[1];
        d[1] = -y[0];
    };
    let bc = |ya: &[f64], yb: &[f64], r: &mut [f64]| {
        r[0] = ya[0];
        r[1] = yb[0] - 1.0;
    };
    let g = |_: f64, y: &mut [f64]| y.fill(0.0);
    let o = BvpOptions::default();
    let e = |r: Result<chimalab_math::ode::BvpSolution, chimalab_math::ode::OdeError>| r.unwrap_err().id();
    assert_eq!(e(bvp(f, bc, &[0.0], g, 2, &o)), "chimalab:ode:intervaloInvalido");
    assert_eq!(
        e(bvp(f, bc, &[1.0, 0.0], g, 2, &o)),
        "chimalab:ode:intervaloInvalido"
    );
    assert_eq!(
        e(bvp(f, bc, &[0.0, 1.0], g, 0, &o)),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        e(bvp(f, bc, &[0.0, 1.0], |_, y| y.fill(f64::NAN), 2, &o)),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        e(bvp(
            f,
            bc,
            &[0.0, 1.0],
            g,
            2,
            &BvpOptions {
                rel_tol: 0.0,
                ..o.clone()
            }
        )),
        "chimalab:ode:toleranciaInvalida"
    );
    assert_eq!(
        e(bvp(
            f,
            bc,
            &[0.0, 1.0],
            g,
            2,
            &BvpOptions {
                max_newton: 0,
                ..o.clone()
            }
        )),
        "chimalab:ode:opcaoInvalida"
    );
    // Sem solução (y'' = −y com y(0) = 0, y(π) = 1): não converge, sem erro.
    let pi = std::f64::consts::PI;
    let s = bvp(
        f,
        bc,
        &[0.0, pi / 2.0, pi],
        g,
        2,
        &BvpOptions {
            max_mesh: 40,
            ..o.clone()
        },
    )
    .unwrap();
    assert!(!s.converged);
    // Avaliação fora do intervalo.
    let s = bvp(f, bc, &[0.0, 1.0], g, 2, &o).unwrap();
    assert!(s.converged);
    assert!(s.eval(-0.1).is_none() && s.eval(1.1).is_none() && s.eval(f64::NAN).is_none());
}

// ================================================================ parâmetros desconhecidos

/// Autovalores de y'' + λy = 0, y(0) = y(π) = 0, normalizados por y'(0) = 1: λ = k², y = sen(kx)/k.
#[test]
fn autovalores_de_sturm_liouville() {
    use chimalab_math::ode::bvp_params;
    let pi = std::f64::consts::PI;
    let malha: Vec<f64> = (0..=10).map(|i| pi * i as f64 / 10.0).collect();
    for (k, lam0) in [(1.0f64, 1.3), (2.0, 3.4), (3.0, 8.2)] {
        let o = BvpOptions {
            rel_tol: 1e-8,
            abs_tol: 1e-10,
            ..Default::default()
        };
        let (sol, p) = bvp_params(
            |_, y, p, d| {
                d[0] = y[1];
                d[1] = -p[0] * y[0];
            },
            |ya, yb, _, r| {
                r[0] = ya[0];
                r[1] = yb[0];
                r[2] = ya[1] - 1.0;
            },
            &malha,
            |x, y| {
                y[0] = (k * x).sin() / k;
                y[1] = (k * x).cos();
            },
            2,
            &[lam0],
            &o,
        )
        .unwrap();
        assert!(sol.converged);
        assert!((p[0] - k * k).abs() <= 1e-9 * k * k, "k {k}: λ = {}", p[0]);
        let e = (0..=100)
            .map(|i| pi * i as f64 / 100.0)
            .map(|x| (sol.eval(x).unwrap()[0] - (k * x).sin() / k).abs())
            .fold(0.0, f64::max);
        assert!(e <= 1e-7, "k {k}: {e}");
        assert_eq!(sol.y[0].len(), 2);
    }
}

/// Mathieu y'' + (λ − 2q·cos 2x)y = 0, q = 5, y'(0) = y'(π) = 0, y(0) = 1 (exemplo de Kierzenka &
/// Shampine, 2001, λ ≈ 17,097). Conferido por tiro: DOPRI5 a 1e-12 e raiz de y'(π; λ) por Brent,
/// caminho independente da colocação.
#[test]
fn mathieu_contra_tiro() {
    use chimalab_math::ode::{AbsTol, OdeOptions, bvp_params, solve_ivp};
    use chimalab_math::optim::{Options, Start, fzero};
    let pi = std::f64::consts::PI;
    let q = 5.0;
    let malha: Vec<f64> = (0..=10).map(|i| pi * i as f64 / 10.0).collect();
    let (sol, p) = bvp_params(
        |x, y, p, d| {
            d[0] = y[1];
            d[1] = -(p[0] - 2.0 * q * (2.0 * x).cos()) * y[0];
        },
        |ya, yb, _, r| {
            r[0] = ya[1];
            r[1] = yb[1];
            r[2] = ya[0] - 1.0;
        },
        &malha,
        |x, y| {
            y[0] = (4.0 * x).cos();
            y[1] = -4.0 * (4.0 * x).sin();
        },
        2,
        &[15.0],
        &BvpOptions {
            rel_tol: 1e-6,
            abs_tol: 1e-8,
            ..Default::default()
        },
    )
    .unwrap();
    // Medido: 323 pontos com rel_tol 1e-6 (85 com 1e-4, 1447 com 1e-8: o controle do resíduo
    // é O(h³)). Regressão: o Newton parado no piso de arredondamento não era dado como
    // convergido, e a malha inteira era refinada (2893 pontos sem convergir com 1e-8).
    assert!(sol.x.len() < 400, "malha {}", sol.x.len());
    assert!(sol.converged);
    let tiro = |lam: f64| {
        let mut o = OdeOptions {
            rel_tol: Some(1e-12),
            abs_tol: Some(AbsTol::Scalar(1e-14)),
            ..Default::default()
        };
        let s = solve_ivp(
            move |x, y, d| {
                d[0] = y[1];
                d[1] = -(lam - 2.0 * q * (2.0 * x).cos()) * y[0];
            },
            &[0.0, pi],
            &[1.0, 0.0],
            &mut o,
        )
        .unwrap();
        s.y.last().unwrap()[1]
    };
    let r = fzero(tiro, Start::Interval(16.5, 17.5), &mut Options::default()).unwrap();
    assert!((p[0] - r.x).abs() <= 1e-7 * r.x, "{} vs {}", p[0], r.x);
    assert!((p[0] - 17.097).abs() < 1e-3);
    // Regressão com rel_tol 1e-8: antes, não convergia (2893 pontos); agora converge com 1447.
    let (sol8, p8) = bvp_params(
        |x, y, p, d| {
            d[0] = y[1];
            d[1] = -(p[0] - 2.0 * q * (2.0 * x).cos()) * y[0];
        },
        |ya, yb, _, r| {
            r[0] = ya[1];
            r[1] = yb[1];
            r[2] = ya[0] - 1.0;
        },
        &malha,
        |x, y| {
            y[0] = (4.0 * x).cos();
            y[1] = -4.0 * (4.0 * x).sin();
        },
        2,
        &[15.0],
        &BvpOptions {
            rel_tol: 1e-8,
            abs_tol: 1e-10,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(sol8.converged && sol8.x.len() < 2000, "{}", sol8.x.len());
    assert!((p8[0] - r.x).abs() <= 1e-9 * r.x, "{} vs {}", p8[0], r.x);
}

#[test]
fn parametros_erros() {
    use chimalab_math::ode::bvp_params;
    let f = |_: f64, y: &[f64], _: &[f64], d: &mut [f64]| d[0] = y[0];
    let bc = |_: &[f64], _: &[f64], _: &[f64], r: &mut [f64]| r.fill(0.0);
    let g = |_: f64, y: &mut [f64]| y[0] = 0.0;
    let o = BvpOptions::default();
    assert_eq!(
        bvp_params(f, bc, &[0.0, 1.0], g, 1, &[f64::NAN], &o)
            .unwrap_err()
            .id(),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        bvp_params(f, bc, &[0.0, 1.0], g, 0, &[1.0], &o).unwrap_err().id(),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        bvp_params(f, bc, &[1.0, 0.0], g, 1, &[1.0], &o).unwrap_err().id(),
        "chimalab:ode:intervaloInvalido"
    );
    // Sem parâmetros: igual ao bvp comum.
    let (s, p) = bvp_params(
        |_, y, _, d| d[0] = -y[0],
        |ya, _, _, r| r[0] = ya[0] - 1.0,
        &[0.0, 0.5, 1.0],
        |_, y| y[0] = 1.0,
        1,
        &[],
        &o,
    )
    .unwrap();
    assert!(p.is_empty() && s.converged && (s.eval(1.0).unwrap()[0] - (-1f64).exp()).abs() <= 1e-6);
}
