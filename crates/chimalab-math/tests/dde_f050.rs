//! F050 — equações com retardo (incremento 1): problema publicado e solução fechada.
//!
//! Oráculo: `tests/dados/F050/exemplo1.csv`, de `ferramentas/oraculos/f050_dde.py`. É o exemplo
//! 1 de Shampine & Thompson (2001) resolvido em aritmética racional exata (polinômios por partes
//! do método dos passos).

use chimalab_math::ode::{AbsTol, OdeOptions, OdeStatus, dde};

fn opts(tol: f64) -> OdeOptions<'static> {
    OdeOptions {
        rel_tol: Some(tol),
        abs_tol: Some(AbsTol::Scalar(tol * 1e-2)),
        dense: true,
        ..Default::default()
    }
}

#[test]
fn ca2_dde_em_problema_publicado() {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F050/exemplo1.csv"
    ))
    .unwrap();
    let refs: Vec<Vec<f64>> = csv
        .lines()
        .skip(1)
        .map(|l| l.split(',').map(|x| x.parse().unwrap()).collect())
        .collect();
    assert_eq!(refs.len(), 10);
    let f = |_: f64, y: &[f64], z: &[Vec<f64>], d: &mut [f64]| {
        d[0] = z[0][0];
        d[1] = z[0][0] + z[1][1];
        d[2] = y[1];
    };
    let mut anterior = f64::INFINITY;
    for tol in [1e-4, 1e-7, 1e-10] {
        let s = dde(
            f,
            &[1.0, 0.2],
            |_, out| out.fill(1.0),
            [0.0, 5.0],
            3,
            &mut opts(tol),
        )
        .unwrap();
        assert_eq!(s.status, OdeStatus::Completed);
        let mut emax = 0.0f64;
        for r in &refs {
            let y = s.eval(r[0]).unwrap();
            for c in 0..3 {
                emax = emax.max((y[c] - r[c + 1]).abs() / r[c + 1].abs());
            }
        }
        println!("DDE tol {tol:e}: erro relativo {emax:e}, {:?}", s.stats);
        assert!(emax <= 100.0 * tol, "tol {tol}: {emax:e}");
        assert!(emax < anterior);
        anterior = emax;
    }
}

/// y'(t) = −y(t − 1), y = 1 para t ≤ 0: pelo método dos passos, em [n − 1, n],
/// y(t) = Σ_{k=0}^{n} (−1)ᵏ (t − k + 1)ᵏ / k!.
#[test]
fn retardo_unitario_solucao_fechada() {
    let exata = |t: f64| -> f64 {
        let n = t.floor() as i32 + 1;
        let mut s = 0.0;
        let mut fat = 1.0;
        for k in 0..=n {
            if k > 0 {
                fat *= k as f64;
            }
            let base = t - k as f64 + 1.0;
            if base > 0.0 {
                s += (-1f64).powi(k) * base.powi(k) / fat;
            }
        }
        s
    };
    let s = dde(
        |_, _, z, d| d[0] = -z[0][0],
        &[1.0],
        |_, o| o[0] = 1.0,
        [0.0, 8.0],
        1,
        &mut opts(1e-9),
    )
    .unwrap();
    for k in 0..=80 {
        let t = 0.1 * k as f64;
        let y = s.eval(t).unwrap()[0];
        assert!((y - exata(t)).abs() <= 1e-7, "t {t}: {y} vs {}", exata(t));
    }
    // As descontinuidades (t = 1, 2, 3) são pontos de malha.
    for d in [1.0, 2.0, 3.0] {
        assert!(s.t.contains(&d), "{d}");
    }
    // Histórico variável: y = cos t para t ≤ 0 com y' = −y(t − π/2) tem solução cos t.
    let p = std::f64::consts::FRAC_PI_2;
    let s = dde(
        |_, _, z, d| d[0] = -z[0][0],
        &[p],
        |t, o| o[0] = t.cos(),
        [0.0, 10.0],
        1,
        &mut opts(1e-9),
    )
    .unwrap();
    for k in 0..=100 {
        let t = 0.1 * k as f64;
        assert!((s.eval(t).unwrap()[0] - t.cos()).abs() <= 1e-7, "{t}");
    }
}

#[test]
fn dde_erros_e_limites() {
    let f = |_: f64, _: &[f64], z: &[Vec<f64>], d: &mut [f64]| d[0] = -z[0][0];
    let h = |_: f64, o: &mut [f64]| o[0] = 1.0;
    let e = |r: Result<chimalab_math::ode::OdeSolution, chimalab_math::ode::OdeError>| r.unwrap_err().id();
    assert_eq!(
        e(dde(f, &[1.0], h, [1.0, 0.0], 1, &mut opts(1e-6))),
        "chimalab:ode:intervaloInvalido"
    );
    assert_eq!(
        e(dde(f, &[0.0], h, [0.0, 1.0], 1, &mut opts(1e-6))),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        e(dde(f, &[], h, [0.0, 1.0], 1, &mut opts(1e-6))),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        e(dde(f, &[1.0], h, [0.0, 1.0], 0, &mut opts(1e-6))),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        e(dde(
            f,
            &[1.0],
            |_, o| o[0] = f64::NAN,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:condicaoInicialInvalida"
    );
    let mut o = opts(1e-6);
    o.abs_tol = Some(AbsTol::Vector(vec![1.0, 2.0]));
    assert_eq!(
        e(dde(f, &[1.0], h, [0.0, 1.0], 1, &mut o)),
        "chimalab:ode:formaIncompativel"
    );
    let mut o = opts(1e-6);
    o.abs_tol = Some(AbsTol::Scalar(-1.0));
    assert_eq!(
        e(dde(f, &[1.0], h, [0.0, 1.0], 1, &mut o)),
        "chimalab:ode:toleranciaInvalida"
    );
    let mut o = opts(1e-6);
    o.events.push(chimalab_math::ode::Event::new(|_, y| y[0]));
    assert_eq!(
        e(dde(f, &[1.0], h, [0.0, 1.0], 1, &mut o)),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        e(dde(
            |_, _, _, d| d[0] = f64::NAN,
            &[1.0],
            h,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:condicaoInicialInvalida"
    );
    for r in [f64::NAN, -1.0] {
        let mut o = opts(1e-6);
        o.rel_tol = Some(r);
        assert_eq!(
            e(dde(f, &[1.0], h, [0.0, 1.0], 1, &mut o)),
            "chimalab:ode:toleranciaInvalida"
        );
    }
    // atol padrão; componente nula com atol 0; erro exatamente nulo (f ≡ 0).
    let s = dde(
        |_, _, _, d| d[0] = 0.0,
        &[1.0],
        |_, o| o[0] = 0.0,
        [0.0, 2.0],
        1,
        &mut OdeOptions {
            abs_tol: Some(AbsTol::Scalar(0.0)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((s.status, s.stats.rejected), (OdeStatus::Completed, 0));
    let s = dde(f, &[1.0], h, [0.0, 2.0], 1, &mut OdeOptions::default()).unwrap();
    assert_eq!(s.status, OdeStatus::Completed);
    // f não finito depois de t = 1,5: passo rejeitado e reduzido até o passo mínimo.
    let s = dde(
        |t, _, _, d| d[0] = (1.5 - t).sqrt(),
        &[1.0],
        h,
        [0.0, 3.0],
        1,
        &mut opts(1e-6),
    )
    .unwrap();
    assert_eq!(s.status, OdeStatus::StepTooSmall);
    assert!((s.t.last().unwrap() - 1.5).abs() <= 1e-6);
    // Limite de passos e explosão.
    let mut o = opts(1e-6);
    o.max_steps = Some(2);
    assert_eq!(
        dde(f, &[1.0], h, [0.0, 5.0], 1, &mut o).unwrap().status,
        OdeStatus::MaxSteps
    );
    let s = dde(
        |_, y, _, d| d[0] = y[0] * y[0],
        &[0.5],
        h,
        [0.0, 3.0],
        1,
        &mut opts(1e-6),
    )
    .unwrap();
    assert_eq!(s.status, OdeStatus::StepTooSmall);
    assert!(s.eval(0.5).is_some());
}

// ================================================================ retardos variáveis

/// Pantográfica y'(t) = y(t/2), y(0) = 1: retardo t/2 que se anula em t = 0. Solução em série
/// y = Σ cₖ tᵏ com c₀ = 1, c_{k+1} = cₖ / ((k + 1)·2ᵏ) (inteira; 30 termos bastam em [0, 3]).
#[test]
fn retardo_que_se_anula_pantografica() {
    use chimalab_math::ode::dde_var;
    let serie = |t: f64| {
        let (mut c, mut s, mut p) = (1.0f64, 0.0f64, 1.0f64);
        for k in 0..30 {
            s += c * p;
            p *= t;
            c /= (k as f64 + 1.0) * 2f64.powi(k);
        }
        s
    };
    for tol in [1e-5, 1e-8] {
        let mut o = opts(tol);
        let sol = dde_var(
            |_, _, z, dy| dy[0] = z[0][0],
            |t, _, s| s[0] = t / 2.0,
            1,
            |_, y| y[0] = 1.0,
            [0.0, 3.0],
            1,
            &mut o,
        )
        .unwrap();
        assert_eq!(sol.status, OdeStatus::Completed);
        let erro = (0..=30)
            .map(|i| {
                let t = 0.1 * i as f64;
                (sol.eval(t).unwrap()[0] - serie(t)).abs() / serie(t)
            })
            .fold(0.0, f64::max);
        // Medido: 0,03·tol (17 passos com tol = 1e-5; 159 com 1e-8).
        assert!(erro <= 5.0 * tol, "tol {tol}: {erro}");
    }
}

/// Retardo dependente do estado, solução fabricada y = sen t:
/// y'(t) = y(s) − sen(s*(t)) + cos t, s = t − 1 − y(t)²/2, s* = t − 1 − sen²t/2.
#[test]
fn retardo_dependente_do_estado_fabricado() {
    use chimalab_math::ode::dde_var;
    let mut o = opts(1e-8);
    let sol = dde_var(
        |t, _, z, dy| {
            let se = t - 1.0 - 0.5 * t.sin().powi(2);
            dy[0] = z[0][0] - se.sin() + t.cos();
        },
        |t, y, s| s[0] = t - 1.0 - 0.5 * y[0] * y[0],
        1,
        |t, y| y[0] = t.sin(),
        [0.0, 10.0],
        1,
        &mut o,
    )
    .unwrap();
    assert_eq!(sol.status, OdeStatus::Completed);
    let erro = sol
        .t
        .iter()
        .zip(&sol.y)
        .map(|(t, y)| (y[0] - t.sin()).abs())
        .fold(0.0, f64::max);
    // Medido: 4e-7 em 1629 passos ao longo de 10 unidades de tempo (erro global acumulado do
    // par de ordem 3 com controle local por passo; rel_tol = 1e-8).
    assert!(erro <= 1e-6, "{erro}");
}

/// Retardo constante pelo caminho variável: y'(t) = −y(t − 1), y = 1 para t ≤ 0. Exata por
/// partes: 1 − t em [0, 1]; 1 − t + (t − 1)²/2 em [1, 2]; e em [2, 3] soma −(t − 2)³/6.
#[test]
fn retardo_constante_pelo_caminho_variavel_e_erros() {
    use chimalab_math::ode::dde_var;
    let exata = |t: f64| {
        let mut v = 1.0 - t;
        if t > 1.0 {
            v += (t - 1.0).powi(2) / 2.0;
        }
        if t > 2.0 {
            v -= (t - 2.0).powi(3) / 6.0;
        }
        v
    };
    let mut o = opts(1e-9);
    let sol = dde_var(
        |_, _, z, dy| dy[0] = -z[0][0],
        |t, _, s| s[0] = t - 1.0,
        1,
        |_, y| y[0] = 1.0,
        [0.0, 3.0],
        1,
        &mut o,
    )
    .unwrap();
    let erro = (0..=60)
        .map(|i| (sol.eval(0.05 * i as f64).unwrap()[0] - exata(0.05 * i as f64)).abs())
        .fold(0.0, f64::max);
    // Medido: 5e-11 em 462 passos. Sem rastrear as quebras em t = 1 e 2, é o controle de erro
    // que reduz o passo junto delas.
    assert!(erro <= 1e-8, "{erro}");
    // Erros: atraso no futuro, sem retardos, intervalo, estado vazio, tolerância, eventos.
    let id = |r: Result<_, chimalab_math::ode::OdeError>| {
        r.map(|_: chimalab_math::ode::OdeSolution| ()).unwrap_err().id()
    };
    let f = |_: f64, _: &[f64], z: &[Vec<f64>], dy: &mut [f64]| dy[0] = -z[0][0];
    let h = |_: f64, y: &mut [f64]| y[0] = 1.0;
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t + 1.0,
            1,
            h,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = if t > 0.5 { f64::NAN } else { t - 1.0 },
            1,
            h,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            0,
            h,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [1.0, 0.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:intervaloInvalido"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [0.0, 1.0],
            0,
            &mut opts(1e-6)
        )),
        "chimalab:ode:condicaoInicialInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [0.0, 1.0],
            1,
            &mut OdeOptions {
                rel_tol: Some(f64::NAN),
                ..Default::default()
            }
        )),
        "chimalab:ode:toleranciaInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [0.0, 1.0],
            1,
            &mut OdeOptions {
                abs_tol: Some(AbsTol::Vector(vec![1.0; 2])),
                ..Default::default()
            }
        )),
        "chimalab:ode:formaIncompativel"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            |_, y| y[0] = f64::NAN,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:condicaoInicialInvalida"
    );
    let mut ev = opts(1e-6);
    ev.events.push(chimalab_math::ode::Event::new(|_, y| y[0]));
    assert_eq!(
        id(dde_var(f, |t, _, s| s[0] = t - 1.0, 1, h, [0.0, 1.0], 1, &mut ev)),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        id(dde_var(
            f,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [0.0, 1.0],
            1,
            &mut OdeOptions {
                abs_tol: Some(AbsTol::Scalar(-1.0)),
                ..Default::default()
            }
        )),
        "chimalab:ode:toleranciaInvalida"
    );
    assert_eq!(
        id(dde_var(
            |_, _, _, dy| dy[0] = f64::NAN,
            |t, _, s| s[0] = t - 1.0,
            1,
            h,
            [0.0, 1.0],
            1,
            &mut opts(1e-6)
        )),
        "chimalab:ode:condicaoInicialInvalida"
    );
    // Explosão em tempo finito (y' = y(s)² com s = t, y(0) = 1 ⇒ y = 1/(1 − t)): passo mínimo.
    let ex = dde_var(
        |_, _, z, dy| dy[0] = z[0][0] * z[0][0],
        |t, _, s| s[0] = t,
        1,
        h,
        [0.0, 2.0],
        1,
        &mut OdeOptions::default(),
    )
    .unwrap();
    assert!(
        matches!(ex.status, OdeStatus::StepTooSmall | OdeStatus::NonFinite),
        "{:?}",
        ex.status
    );
    // Para junto da singularidade exata t = 1 (medido: 1 + 5e-7).
    assert!(
        (ex.t.last().unwrap() - 1.0).abs() <= 1e-4,
        "{}",
        ex.t.last().unwrap()
    );
    // Limite de passos.
    let lim = dde_var(
        f,
        |t, _, s| s[0] = t - 1.0,
        1,
        h,
        [0.0, 3.0],
        1,
        &mut OdeOptions {
            max_steps: Some(3),
            ..opts(1e-9)
        },
    )
    .unwrap();
    assert_eq!(lim.status, OdeStatus::MaxSteps);
}

/// Solução exata de y′(t) = −y(t − 1), y = 1 para t ≤ 0 (método dos passos):
/// y(t) = Σₖ (−1)ᵏ (t − k + 1)ᵏ / k!, com k de 0 a ⌊t⌋ + 1.
fn y_atraso_unitario(t: f64) -> f64 {
    let mut s = 0.0;
    let mut fat = 1.0;
    for k in 0..=(t.floor() as i32 + 1) {
        if k > 0 {
            fat *= f64::from(k);
        }
        let base = t - f64::from(k) + 1.0;
        if base >= 0.0 {
            s += f64::from(if k % 2 == 0 { 1 } else { -1 }) * base.powi(k) / fat;
        }
    }
    s
}

/// Estrutura do integrador com retardos constantes (mutação): os pontos de quebra
/// t₀ + Σ mⱼτⱼ (Σ mⱼ ≤ 3) entram exatamente na malha, também com dois retardos e somas
/// coincidentes; precisão com tolerância apertada contra a solução exata; `max_steps`
/// respeitado; tolerância absoluta vetorial; intervalo vazio é erro.
#[test]
fn dde_quebras_precisao_e_opcoes() {
    use chimalab_math::ode::dde;
    // Um retardo, t₀ = 0: quebras 1, 2, 3 na malha; erro ≤ 1e-7 com rtol = 1e-10.
    let mut o = opts(1e-10);
    let s = dde(
        |_, _, z, dy| dy[0] = -z[0][0],
        &[1.0],
        |_, y| y[0] = 1.0,
        [0.0, 4.5],
        1,
        &mut o,
    )
    .unwrap();
    assert_eq!(s.status, OdeStatus::Completed);
    for q in [1.0, 2.0, 3.0] {
        assert!(s.t.contains(&q), "quebra {q} fora da malha");
    }
    for (t, y) in s.t.iter().zip(&s.y) {
        assert!(
            (y[0] - y_atraso_unitario(*t)).abs() <= 1e-8,
            "t = {t}: {} vs {}",
            y[0],
            y_atraso_unitario(*t)
        );
    }
    // t₀ = 10 (quebras deslocadas) e retardo maior que o intervalo.
    let mut o = opts(1e-10);
    let s = dde(
        |_, _, z, dy| dy[0] = -z[0][0],
        &[1.0],
        |_, y| y[0] = 1.0,
        [10.0, 13.0],
        1,
        &mut o,
    )
    .unwrap();
    for q in [11.0, 12.0] {
        assert!(s.t.contains(&q), "quebra {q} fora da malha");
    }
    assert!((s.y.last().unwrap()[0] - y_atraso_unitario(3.0)).abs() <= 1e-8);
    let mut o = opts(1e-8);
    let s = dde(
        |_, _, z, dy| dy[0] = -z[0][0],
        &[5.0],
        |_, y| y[0] = 1.0,
        [10.0, 11.0],
        1,
        &mut o,
    )
    .unwrap();
    assert!((s.y.last().unwrap()[0] - 0.0).abs() <= 1e-9, "{:?}", s.y.last());
    // Dois retardos (1 e 1,5): quebras 1; 1,5; 2; 2,5; 3 (de 1+1+1 e 1,5+1,5); 3,5; 4; 4,5.
    let mut o = opts(1e-8);
    let s = dde(
        |_, _, z, dy| dy[0] = -0.5 * z[0][0] - 0.25 * z[1][0],
        &[1.0, 1.5],
        |_, y| y[0] = 1.0,
        [0.0, 4.8],
        1,
        &mut o,
    )
    .unwrap();
    for q in [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5] {
        assert!(s.t.contains(&q), "quebra {q} fora da malha");
    }
    assert_eq!(s.t.iter().filter(|&&t| t == 3.0).count(), 1, "quebra repetida");
    // max_steps exato.
    let mut o = OdeOptions {
        max_steps: Some(3),
        ..opts(1e-8)
    };
    let s = dde(
        |_, _, z, dy| dy[0] = -z[0][0],
        &[1.0],
        |_, y| y[0] = 1.0,
        [0.0, 4.0],
        1,
        &mut o,
    )
    .unwrap();
    assert_eq!((s.status, s.stats.steps), (OdeStatus::MaxSteps, 3));
    // Tolerância absoluta vetorial com o tamanho certo; intervalo vazio.
    let mut o = OdeOptions {
        abs_tol: Some(AbsTol::Vector(vec![1e-10, 1e-10])),
        rel_tol: Some(1e-8),
        ..Default::default()
    };
    let s = dde(
        |_, y, z, dy| {
            dy[0] = -z[0][0];
            dy[1] = y[0];
        },
        &[1.0],
        |_, y| {
            y[0] = 1.0;
            y[1] = 0.0;
        },
        [0.0, 2.0],
        2,
        &mut o,
    )
    .unwrap();
    // y₂ = ∫ y₁: ∫₀¹ (1 − t) dt + ∫₁² (1 − t + (t − 1)²/2) dt = 1/2 − 1/2 + 1/6 = 1/6.
    assert!(
        (s.y.last().unwrap()[1] - 1.0 / 6.0).abs() <= 1e-7,
        "{:?}",
        s.y.last()
    );
    let r = dde(
        |_, _, z, dy| dy[0] = -z[0][0],
        &[1.0],
        |_, y| y[0] = 1.0,
        [1.0, 1.0],
        1,
        &mut opts(1e-6),
    );
    assert_eq!(r.unwrap_err().id(), "chimalab:ode:intervaloInvalido");
}
