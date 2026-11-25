//! F049 — EDOs (incremento 1: métodos explícitos, saída densa e eventos).
//!
//! Oráculos:
//! - soluções analíticas;
//! - `tests/dados/F049/referencias.csv`, de `ferramentas/oraculos/f049_referencias.py` (mpmath
//!   `odefun`, 30 dígitos);
//! - as tabelas de Runge–Kutta são conferidas à parte, pelas condições de ordem em aritmética
//!   racional (`ferramentas/oraculos/f049_tabelas_rk.py`).

use std::f64::consts::PI;

use chimalab_math::ode::{
    AbsTol, Direction, Event, Method, OdeError, OdeOptions, OdeSolution, OdeStatus, OdeWarning, solve_ivp,
};
use proptest::prelude::*;

fn opts<'a>(method: Method, rtol: f64, atol: f64) -> OdeOptions<'a> {
    OdeOptions {
        method,
        rel_tol: Some(rtol),
        abs_tol: Some(AbsTol::Scalar(atol)),
        ..Default::default()
    }
}

fn fim(s: &OdeSolution) -> (&f64, &Vec<f64>) {
    (s.t.last().unwrap(), s.y.last().unwrap())
}

/// Erro relativo misto max |a − b| / max(1, |b|).
fn erro(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs() / y.abs().max(1.0))
        .fold(0.0, f64::max)
}

// ---------------------------------------------------------------- soluções analíticas

#[test]
fn ca3_solucoes_analiticas() {
    // (nome, f, y0, t_f, solução exata em t_f)
    type Caso = (
        &'static str,
        fn(f64, &[f64], &mut [f64]),
        Vec<f64>,
        f64,
        fn(f64) -> Vec<f64>,
    );
    let casos: Vec<Caso> = vec![
        (
            "decaimento",
            |_, y, d| d[0] = -y[0],
            vec![1.0],
            5.0,
            |t| vec![(-t).exp()],
        ),
        (
            "gaussiana",
            |t, y, d| d[0] = -2.0 * t * y[0],
            vec![1.0],
            3.0,
            |t| vec![(-t * t).exp()],
        ),
        (
            "oscilador",
            |_, y, d| {
                d[0] = y[1];
                d[1] = -y[0];
            },
            vec![1.0, 0.0],
            20.0 * PI,
            |t| vec![t.cos(), -t.sin()],
        ),
        // y' = 1 + y², y(0) = 0: y = tan t (cresce até 1/cos²).
        (
            "tangente",
            |_, y, d| d[0] = 1.0 + y[0] * y[0],
            vec![0.0],
            1.5,
            |t| vec![t.tan()],
        ),
        // Logística y' = y(1 − y): y = 1/(1 + 9e^{−t}).
        (
            "logistica",
            |_, y, d| d[0] = y[0] * (1.0 - y[0]),
            vec![0.1],
            10.0,
            |t| vec![1.0 / (1.0 + 9.0 * (-t).exp())],
        ),
    ];
    for (nome, f, y0, tf, exata) in &casos {
        let mut anterior = f64::INFINITY;
        for (metodo, tols) in [
            (Method::Dopri5, &[1e-6, 1e-9, 1e-12][..]),
            (Method::Bs23, &[1e-6, 1e-8][..]),
        ] {
            for &tol in tols {
                let s = solve_ivp(f, &[0.0, *tf], y0, &mut opts(metodo, tol, tol)).unwrap();
                assert_eq!(s.status, OdeStatus::Completed);
                let (t, y) = fim(&s);
                assert_eq!(*t, *tf);
                let e = erro(y, &exata(*tf));
                // Erro global ≤ 1000·tol: o controle é local por passo e o erro se acumula
                // ao longo de dezenas a centenas de passos (o oscilador cobre 10 períodos).
                assert!(e <= 1e3 * tol, "{nome} {metodo:?} tol {tol}: {e:e}");
                if metodo == Method::Dopri5 {
                    // Proporcionalidade à tolerância: o erro cai quando ela cai.
                    assert!(e < anterior, "{nome}: {e:e} ≥ {anterior:e}");
                    anterior = e;
                }
            }
        }
    }
}

/// Kepler com excentricidade 0,5: órbita fechada com período 2π; solução pela equação de
/// Kepler E − e·sen E = M (resolvida por Newton em f64).
#[test]
fn kepler_excentrico() {
    let e = 0.5f64;
    let f = |_: f64, y: &[f64], d: &mut [f64]| {
        let r3 = (y[0] * y[0] + y[1] * y[1]).powf(1.5);
        d[0] = y[2];
        d[1] = y[3];
        d[2] = -y[0] / r3;
        d[3] = -y[1] / r3;
    };
    let y0 = [1.0 - e, 0.0, 0.0, ((1.0 + e) / (1.0 - e)).sqrt()];
    let exata = |t: f64| {
        let mut ea = t;
        for _ in 0..50 {
            ea -= (ea - e * ea.sin() - t) / (1.0 - e * ea.cos());
        }
        let (s, c) = ea.sin_cos();
        let raiz = (1.0 - e * e).sqrt();
        let edot = 1.0 / (1.0 - e * c);
        vec![c - e, raiz * s, -s * edot, raiz * c * edot]
    };
    let ts: Vec<f64> = (0..=12).map(|k| k as f64 * PI / 2.0).collect();
    let s = solve_ivp(f, &ts, &y0, &mut opts(Method::Dopri5, 1e-12, 1e-12)).unwrap();
    assert_eq!(s.t, ts);
    for (t, y) in s.t.iter().zip(&s.y) {
        assert!(erro(y, &exata(*t)) <= 1e-9, "t = {t}: {:e}", erro(y, &exata(*t)));
    }
    // Três órbitas completas: volta ao estado inicial.
    assert!(erro(s.y.last().unwrap(), &y0) <= 1e-9);
}

// ---------------------------------------------------------------- referências mpmath

fn ref_fn(nome: &str) -> fn(f64, &[f64], &mut [f64]) {
    match nome {
        "vanderpol1" => |_, y, d| {
            d[0] = y[1];
            d[1] = (1.0 - y[0] * y[0]) * y[1] - y[0];
        },
        "lorenz" => |_, y, d| {
            d[0] = 10.0 * (y[1] - y[0]);
            d[1] = y[0] * (28.0 - y[2]) - y[1];
            d[2] = y[0] * y[1] - 8.0 / 3.0 * y[2];
        },
        "pendulo" => |_, y, d| {
            d[0] = y[1];
            d[1] = -y[0].sin();
        },
        "brusselator" => |_, y, d| {
            d[0] = 1.0 + y[0] * y[0] * y[1] - 4.0 * y[0];
            d[1] = 3.0 * y[0] - y[0] * y[0] * y[1];
        },
        _ => panic!("{nome}"),
    }
}

fn ref_y0(nome: &str) -> Vec<f64> {
    match nome {
        "vanderpol1" => vec![2.0, 0.0],
        "lorenz" => vec![1.0, 1.0, 1.0],
        "pendulo" => vec![3.0, 0.0],
        "brusselator" => vec![1.5, 3.0],
        _ => panic!("{nome}"),
    }
}

#[test]
fn referencias_mpmath() {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F049/referencias.csv"
    ))
    .unwrap();
    // nome → [(t, [componentes])]
    type Pontos = Vec<(f64, Vec<f64>)>;
    let mut refs: Vec<(String, Pontos)> = Vec::new();
    for linha in csv.lines().skip(1) {
        let c: Vec<&str> = linha.split(',').collect();
        let (nome, t, i, v): (&str, f64, usize, f64) = (
            c[0],
            c[1].parse().unwrap(),
            c[2].parse().unwrap(),
            c[3].parse().unwrap(),
        );
        if refs.last().is_none_or(|r| r.0 != nome) {
            refs.push((nome.to_string(), Vec::new()));
        }
        let pts = &mut refs.last_mut().unwrap().1;
        if pts.last().is_none_or(|p| p.0 != t) {
            pts.push((t, Vec::new()));
        }
        let p = pts.last_mut().unwrap();
        assert_eq!(p.1.len(), i);
        p.1.push(v);
    }
    assert_eq!(refs.len(), 4);
    for (nome, pts) in &refs {
        let mut ts = vec![0.0];
        ts.extend(pts.iter().map(|p| p.0));
        for (metodo, tol, limite) in [(Method::Dopri5, 1e-12, 1e-8), (Method::Bs23, 1e-9, 1e-5)] {
            let s = solve_ivp(ref_fn(nome), &ts, &ref_y0(nome), &mut opts(metodo, tol, tol)).unwrap();
            assert_eq!(s.t, ts);
            for ((t, yr), y) in pts.iter().zip(&s.y[1..]) {
                let e = erro(y, yr);
                assert!(e <= limite, "{nome} {metodo:?} t = {t}: {e:e}");
            }
        }
    }
}

/// Órbita de Arenstorf (problema restrito de três corpos): periódica com período T conhecido
/// (Hairer, Nørsett & Wanner, vol. I, §II.0). Sensível: pequenos erros crescem muito.
#[test]
fn arenstorf_periodica() {
    let mu = 0.012277471;
    let mup = 1.0 - mu;
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        let d1 = ((y[0] + mu).powi(2) + y[1] * y[1]).powf(1.5);
        let d2 = ((y[0] - mup).powi(2) + y[1] * y[1]).powf(1.5);
        d[0] = y[2];
        d[1] = y[3];
        d[2] = y[0] + 2.0 * y[3] - mup * (y[0] + mu) / d1 - mu * (y[0] - mup) / d2;
        d[3] = y[1] - 2.0 * y[2] - mup * y[1] / d1 - mu * y[1] / d2;
    };
    let y0 = [0.994, 0.0, 0.0, -2.001_585_106_379_082_5];
    let periodo = 17.065_216_560_157_963;
    let s = solve_ivp(f, &[0.0, periodo], &y0, &mut opts(Method::Dopri5, 1e-12, 1e-12)).unwrap();
    let e = erro(fim(&s).1, &y0);
    println!("arenstorf: erro {e:e}, {:?}", s.stats);
    assert!(e <= 1e-6, "{e:e}");
}

// ---------------------------------------------------------------- eventos

#[test]
fn ca2_evento_terminal_no_instante_correto() {
    // Queda livre de 10 m: y'' = −g; chão em t* = √(2h/g). A solução é quadrática, que o DOPRI5
    // e a saída densa reproduzem exatamente: o evento sai com precisão de máquina.
    let g = 9.81;
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        d[0] = y[1];
        d[1] = -g;
    };
    let tstar = (2.0 * 10.0 / g).sqrt();
    for metodo in [Method::Dopri5, Method::Bs23] {
        let mut o = opts(metodo, 1e-6, 1e-9);
        o.events
            .push(Event::new(|_, y| y[0]).terminal().direction(Direction::Falling));
        let s = solve_ivp(f, &[0.0, 10.0], &[10.0, 0.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Terminated);
        assert_eq!(s.events.len(), 1);
        let ev = &s.events[0];
        assert!(
            (ev.t - tstar).abs() <= 4.0 * f64::EPSILON * tstar,
            "{metodo:?}: {}",
            ev.t - tstar
        );
        assert_eq!((*fim(&s).0, fim(&s).1), (ev.t, &ev.y));
        assert!(ev.y[0].abs() <= 1e-13);
        // Nenhum ponto de saída depois do evento.
        assert!(s.t.iter().all(|&t| t <= ev.t));
    }
    // Solução não polinomial: cos t = 0 em π/2, precisão da ordem da tolerância.
    for tol in [1e-6, 1e-10] {
        let mut o = opts(Method::Dopri5, tol, tol);
        o.events.push(Event::new(|_, y| y[0]).terminal());
        let s = solve_ivp(
            |_, y, d| {
                d[0] = y[1];
                d[1] = -y[0];
            },
            &[0.0, 10.0],
            &[1.0, 0.0],
            &mut o,
        )
        .unwrap();
        let e = (s.events[0].t - PI / 2.0).abs();
        assert!(e <= 10.0 * tol, "tol {tol}: {e:e}");
    }
}

/// Bola quicando com coeficiente de restituição 0,9: reinício a cada evento terminal. Os
/// instantes de impacto têm forma fechada: tₖ = t₁ + 2v₁(0,9 + … + 0,9^{k−1})/g.
#[test]
fn bola_quicando_com_reinicio() {
    let g = 9.81;
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        d[0] = y[1];
        d[1] = -g;
    };
    let (mut t, mut y) = (0.0, vec![10.0, 0.0]);
    let t1 = (20.0 / g).sqrt();
    let v1 = g * t1;
    let mut esperado = t1;
    for k in 0..6 {
        let mut o = opts(Method::Dopri5, 1e-10, 1e-12);
        o.events
            .push(Event::new(|_, y| y[0]).terminal().direction(Direction::Falling));
        let s = solve_ivp(f, &[t, 100.0], &y, &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Terminated);
        let ev = &s.events[0];
        assert!(
            (ev.t - esperado).abs() <= 1e-12 * esperado,
            "quique {k}: {}",
            ev.t - esperado
        );
        t = ev.t;
        y = vec![0.0, -0.9 * ev.y[1]];
        esperado += 2.0 * v1 * 0.9f64.powi(k + 1) / g;
    }
}

#[test]
fn eventos_direcao_simultaneos_e_nao_terminais() {
    let osc = |_: f64, y: &[f64], d: &mut [f64]| {
        d[0] = y[1];
        d[1] = -y[0];
    };
    // y = (sen t, cos t) em [0, 10π]: zeros de sen em kπ; sobe em 2kπ e desce em (2k+1)π.
    // O zero em t₀ não conta (g começa em 0).
    let mut o = opts(Method::Dopri5, 1e-10, 1e-10);
    o.events.push(Event::new(|_, y| y[0]));
    o.events
        .push(Event::new(|_, y| y[0]).direction(Direction::Rising));
    o.events
        .push(Event::new(|_, y| y[0]).direction(Direction::Falling));
    // Duas funções idênticas: eventos simultâneos registrados ambos.
    o.events
        .push(Event::new(|_, y| y[0]).direction(Direction::Falling));
    let s = solve_ivp(osc, &[0.0, 10.0 * PI - 0.1], &[0.0, 1.0], &mut o).unwrap();
    assert_eq!(s.status, OdeStatus::Completed);
    let por = |i: usize| {
        s.events
            .iter()
            .filter(|e| e.index == i)
            .map(|e| e.t)
            .collect::<Vec<_>>()
    };
    let ambos = por(0);
    assert_eq!(ambos.len(), 9);
    for (k, t) in ambos.iter().enumerate() {
        assert!((t - (k + 1) as f64 * PI).abs() <= 1e-8, "{k}: {t}");
    }
    assert_eq!(por(1).len(), 4);
    assert_eq!(por(2).len(), 5);
    assert_eq!(por(2), por(3));
    // Em ordem de tempo, e índices em ordem crescente no mesmo instante.
    for w in s.events.windows(2) {
        assert!(w[0].t < w[1].t || (w[0].t == w[1].t && w[0].index < w[1].index));
    }
    // Terminal e não terminal no mesmo instante: ambos registrados, termina ali.
    let mut o = opts(Method::Dopri5, 1e-10, 1e-10);
    o.events.push(Event::new(|_, y| y[0]));
    o.events.push(Event::new(|_, y| y[0]).terminal());
    let s = solve_ivp(osc, &[0.0, 10.0], &[0.0, 1.0], &mut o).unwrap();
    assert_eq!(s.events.len(), 2);
    assert_eq!(s.events[0].t, s.events[1].t);
    assert_eq!(*fim(&s).0, s.events[0].t);
    // Dois eventos terminais em instantes diferentes no mesmo passo: vale o primeiro.
    let mut o = opts(Method::Dopri5, 1e-3, 1e-3);
    o.max_step = Some(5.0);
    o.events.push(Event::new(|t, _| t - 0.7).terminal());
    o.events.push(Event::new(|t, _| t - 0.3).terminal());
    let s = solve_ivp(|_, _, d| d[0] = 1.0, &[0.0, 5.0], &[0.0], &mut o).unwrap();
    assert_eq!(s.events.len(), 1);
    assert_eq!(s.events[0].index, 1);
    assert!((s.events[0].t - 0.3).abs() <= 1e-15);
    // Evento exatamente no fim de um passo (g = 0 em t_novo): t de chegada.
    let mut o = opts(Method::Dopri5, 1e-6, 1e-9);
    o.events.push(Event::new(|t, _| t - 2.0).terminal());
    let s = solve_ivp(|_, _, d| d[0] = 0.0, &[0.0, 2.0], &[1.0], &mut o).unwrap();
    assert_eq!(s.events.len(), 1);
    assert_eq!(s.events[0].t, 2.0);
    assert_eq!(s.status, OdeStatus::Terminated);
}

// ---------------------------------------------------------------- saída

#[test]
fn saida_refine_instantes_e_densa() {
    let f = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    // refine padrão: 4 pontos por passo no DOPRI5, 1 no BS23.
    let mut o = opts(Method::Dopri5, 1e-6, 1e-9);
    o.dense = true;
    let s = solve_ivp(f, &[0.0, 5.0], &[1.0], &mut o).unwrap();
    assert_eq!(s.t.len(), 4 * s.stats.steps + 1);
    assert!(s.t.windows(2).all(|w| w[0] < w[1]));
    for (t, y) in s.t.iter().zip(&s.y) {
        assert!((y[0] - (-t).exp()).abs() <= 1e-7, "{t}");
        // A saída densa coincide com os pontos de saída.
        let d = s.eval(*t).unwrap();
        assert!((d[0] - y[0]).abs() <= 1e-15, "{t}");
    }
    assert_eq!(s.eval(0.0).unwrap(), vec![1.0]);
    assert_eq!(s.eval(5.0).unwrap(), s.y.last().unwrap().clone());
    assert!(s.eval(-0.1).is_none() && s.eval(5.1).is_none() && s.eval(f64::NAN).is_none());
    // Saída densa de ordem 4: erro no meio dos passos da ordem da tolerância.
    for k in 0..=100 {
        let t = 0.05 * k as f64;
        assert!((s.eval(t).unwrap()[0] - (-t).exp()).abs() <= 1e-7, "{t}");
    }
    let s = solve_ivp(f, &[0.0, 5.0], &[1.0], &mut opts(Method::Bs23, 1e-6, 1e-9)).unwrap();
    assert_eq!(s.t.len(), s.stats.steps + 1);
    assert!(s.eval(1.0).is_none(), "sem dense");
    let mut o = opts(Method::Dopri5, 1e-6, 1e-9);
    o.refine = Some(1);
    let s = solve_ivp(f, &[0.0, 5.0], &[1.0], &mut o).unwrap();
    assert_eq!(s.t.len(), s.stats.steps + 1);
    // Instantes pedidos: exatamente eles, mesmo muito próximos ou a cada passo vários.
    let ts: Vec<f64> = (0..=500).map(|k| k as f64 * 0.01).collect();
    let s = solve_ivp(f, &ts, &[1.0], &mut opts(Method::Dopri5, 1e-8, 1e-12)).unwrap();
    assert_eq!(s.t, ts);
    for (t, y) in s.t.iter().zip(&s.y) {
        assert!((y[0] - (-t).exp()).abs() <= 1e-8);
    }
    // Para trás: y' = y de 1 a 0.
    let mut o = opts(Method::Dopri5, 1e-10, 1e-12);
    o.dense = true;
    let s = solve_ivp(|_, y, d| d[0] = y[0], &[1.0, 0.0], &[1f64.exp()], &mut o).unwrap();
    assert!(s.t.windows(2).all(|w| w[0] > w[1]));
    assert!((fim(&s).1[0] - 1.0).abs() <= 1e-9);
    assert!((s.eval(0.5).unwrap()[0] - 0.5f64.exp()).abs() <= 1e-9);
    assert!(s.eval(1.5).is_none());
    let s = solve_ivp(
        |_, y, d| d[0] = y[0],
        &[1.0, 0.5, 0.0],
        &[1f64.exp()],
        &mut OdeOptions::default(),
    )
    .unwrap();
    assert_eq!(s.t, vec![1.0, 0.5, 0.0]);
}

// ---------------------------------------------------------------- estados, bordas, avisos

#[test]
fn falhas_dao_resultado_parcial() {
    // Explosão: y' = y², y(0) = 1 → y = 1/(1 − t), singular em t = 1.
    let s = solve_ivp(
        |_, y, d| d[0] = y[0] * y[0],
        &[0.0, 2.0],
        &[1.0],
        &mut OdeOptions::default(),
    )
    .unwrap();
    assert_eq!(s.status, OdeStatus::StepTooSmall);
    assert_eq!(s.status.warning_id(), Some("chimalab:ode:passoMinimo"));
    let (t, y) = fim(&s);
    // O polo numérico se desloca com o erro global acumulado (~tol): 1 + 2,3e-7 com rtol 1e-6.
    assert!((t - 1.0).abs() <= 1e-5, "{t}");
    assert!(y[0] > 1e6);
    // f não finito depois de t = 1: para em t ≈ 1 com NaoFinito.
    let s = solve_ivp(
        |t, _, d| d[0] = (1.0 - t).sqrt(),
        &[0.0, 2.0],
        &[0.0],
        &mut OdeOptions::default(),
    )
    .unwrap();
    assert_eq!(s.status, OdeStatus::NonFinite);
    assert_eq!(s.status.warning_id(), Some("chimalab:ode:naoFinito"));
    // Contagem exata: os rejeitados incluem os passos com f não finito (mutação).
    assert_eq!(
        (s.stats.steps, s.stats.rejected, s.stats.fun_evals),
        (36, 36, 434)
    );
    let (t, y) = fim(&s);
    assert!((t - 1.0).abs() <= 1e-6, "{t}");
    assert!((y[0] - 2.0 / 3.0).abs() <= 1e-6, "{}", y[0]);
    // Limite de passos.
    let mut o = OdeOptions {
        max_steps: Some(3),
        ..Default::default()
    };
    let s = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 100.0], &[1.0], &mut o).unwrap();
    assert_eq!((s.status, s.stats.steps), (OdeStatus::MaxSteps, 3));
    assert_eq!(s.status.warning_id(), Some("chimalab:ode:maximoDePassos"));
    assert_eq!(OdeStatus::Completed.warning_id(), None);
    assert_eq!(OdeStatus::Terminated.warning_id(), None);
}

#[test]
fn bordas_e_tolerancias() {
    let dec = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    // Tolerância relativa abaixo de 100ε: elevada, com aviso.
    let s = solve_ivp(dec, &[0.0, 1.0], &[1.0], &mut opts(Method::Dopri5, 0.0, 0.0)).unwrap();
    assert_eq!(
        s.warnings,
        vec![OdeWarning::RelTolRaised {
            requested: 0.0,
            used: 100.0 * f64::EPSILON
        }]
    );
    assert!((fim(&s).1[0] - (-1f64).exp()).abs() <= 1e-12);
    // Escalas extremas: o controle relativo vale para 1e-300 e 1e300.
    for y0 in [1e-300, 1e300, -1e300, 5e-320] {
        let s = solve_ivp(dec, &[0.0, 2.0], &[y0], &mut opts(Method::Dopri5, 1e-10, 0.0)).unwrap();
        let ex = y0 * (-2f64).exp();
        let tol = if y0.abs() < 1e-310 { 1e-3 } else { 1e-8 };
        assert!(((fim(&s).1[0] - ex) / ex).abs() <= tol, "{y0}: {}", fim(&s).1[0]);
    }
    // Estado nulo constante com tolerância absoluta zero: não rejeita (erro 0/0 = 0).
    let s = solve_ivp(
        |_, _, d| d[0] = 0.0,
        &[0.0, 1.0],
        &[0.0],
        &mut opts(Method::Bs23, 1e-6, 0.0),
    )
    .unwrap();
    assert_eq!((s.status, s.stats.rejected), (OdeStatus::Completed, 0));
    assert_eq!(fim(&s).1[0], 0.0);
    // Tolerância absoluta vetorial: componente sem controle absoluto e outra com.
    let mut o = opts(Method::Dopri5, 1e-8, 0.0);
    o.abs_tol = Some(AbsTol::Vector(vec![1e-12, 1e-3]));
    let s = solve_ivp(
        |_, y, d| {
            d[0] = -y[0];
            d[1] = -10.0 * y[1];
        },
        &[0.0, 1.0],
        &[1.0, 1.0],
        &mut o,
    )
    .unwrap();
    assert!((fim(&s).1[0] - (-1f64).exp()).abs() <= 1e-8);
    // Sistema grande e desacoplado: taxas de 0,01 a 1.
    let n = 100;
    let s = solve_ivp(
        |_, y, d| {
            for i in 0..y.len() {
                d[i] = -((i + 1) as f64) / 100.0 * y[i];
            }
        },
        &[0.0, 3.0],
        &vec![1.0; n],
        &mut opts(Method::Dopri5, 1e-10, 1e-12),
    )
    .unwrap();
    for i in 0..n {
        assert!((fim(&s).1[i] - (-3.0 * (i + 1) as f64 / 100.0).exp()).abs() <= 1e-9);
    }
    // Passo inicial e passo máximo.
    let mut o = OdeOptions {
        initial_step: Some(1e-3),
        max_step: Some(0.01),
        refine: Some(1),
        ..Default::default()
    };
    let s = solve_ivp(dec, &[0.0, 1.0], &[1.0], &mut o).unwrap();
    assert!(s.t.windows(2).all(|w| w[1] - w[0] <= 0.01 * (1.0 + 1e-12)));
    assert!(s.t[1] - s.t[0] <= 1e-3 + 1e-15);
    assert!(s.stats.steps >= 100);
    // Passo inicial maior que o intervalo: limitado ao intervalo.
    let mut o = OdeOptions {
        initial_step: Some(10.0),
        ..Default::default()
    };
    let s = solve_ivp(|_, _, d| d[0] = 1.0, &[0.0, 1e-3], &[0.0], &mut o).unwrap();
    assert!((fim(&s).1[0] - 1e-3).abs() <= 1e-18);
    // f constante com y₀ = 0: o passo inicial é cauteloso (h₀ = 1e-6, pois ‖y₀‖ = 0) e cresce
    // 10× por passo (fator máximo), sem rejeição: 1e-4, 1e-3, 1e-2, 0,1 e o resto.
    let s = solve_ivp(
        |_, _, d| d[0] = 2.0,
        &[0.0, 1.0],
        &[0.0],
        &mut OdeOptions::default(),
    )
    .unwrap();
    // Σbⱼ = 1 só a menos de arredondamento em f64.
    assert!((fim(&s).1[0] - 2.0).abs() <= 4.0 * f64::EPSILON);
    assert_eq!((s.stats.steps, s.stats.rejected), (5, 0));
    // Instantes enormes: passo mínimo relativo a |t| (16·ε·|t| ≈ 3,6e-5 em 1e10).
    let s = solve_ivp(dec, &[1e10, 1e10 + 1.0], &[1.0], &mut OdeOptions::default()).unwrap();
    assert_eq!(s.status, OdeStatus::Completed);
    assert!((fim(&s).1[0] - (-1f64).exp()).abs() <= 1e-6);
    // Em 1e15 o passo mínimo (≈ 3,6) excede o intervalo: resultado parcial só com t₀.
    let s = solve_ivp(dec, &[1e15, 1e15 + 1.0], &[1.0], &mut OdeOptions::default()).unwrap();
    assert_eq!((s.status, s.t.len()), (OdeStatus::StepTooSmall, 1));
    // Ida e volta pelo Debug.
    let o = OdeOptions {
        events: vec![Event::new(|_, _| 1.0)],
        ..Default::default()
    };
    let txt = format!("{o:?}");
    assert!(txt.contains("Event") && txt.contains("Dopri5"), "{txt}");
}

#[test]
fn erros_com_identificador() {
    let f = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    let e = |tspan: &[f64], y0: &[f64], o: &mut OdeOptions| solve_ivp(f, tspan, y0, o).unwrap_err();
    let d = || OdeOptions::default();
    let casos: Vec<(OdeError, &str)> = vec![
        (e(&[0.0], &[1.0], &mut d()), "chimalab:ode:intervaloInvalido"),
        (
            e(&[0.0, f64::NAN], &[1.0], &mut d()),
            "chimalab:ode:intervaloInvalido",
        ),
        (e(&[1.0, 1.0], &[1.0], &mut d()), "chimalab:ode:intervaloInvalido"),
        (
            e(&[0.0, 2.0, 1.0], &[1.0], &mut d()),
            "chimalab:ode:intervaloInvalido",
        ),
        (
            e(&[0.0, 1.0, 1.0], &[1.0], &mut d()),
            "chimalab:ode:intervaloInvalido",
        ),
        (
            e(&[0.0, 1.0], &[], &mut d()),
            "chimalab:ode:condicaoInicialInvalida",
        ),
        (
            e(&[0.0, 1.0], &[f64::INFINITY], &mut d()),
            "chimalab:ode:condicaoInicialInvalida",
        ),
        (
            e(&[0.0, 1.0], &[1.0], &mut opts(Method::Dopri5, -1.0, 1e-9)),
            "chimalab:ode:toleranciaInvalida",
        ),
        (
            e(&[0.0, 1.0], &[1.0], &mut opts(Method::Dopri5, f64::NAN, 1e-9)),
            "chimalab:ode:toleranciaInvalida",
        ),
        (
            e(&[0.0, 1.0], &[1.0], &mut opts(Method::Dopri5, 1e-6, -1.0)),
            "chimalab:ode:toleranciaInvalida",
        ),
        (
            e(
                &[0.0, 1.0],
                &[1.0],
                &mut opts(Method::Dopri5, 1e-6, f64::INFINITY),
            ),
            "chimalab:ode:toleranciaInvalida",
        ),
        (
            e(
                &[0.0, 1.0],
                &[1.0],
                &mut OdeOptions {
                    abs_tol: Some(AbsTol::Vector(vec![1.0, 2.0])),
                    ..d()
                },
            ),
            "chimalab:ode:formaIncompativel",
        ),
        (
            e(
                &[0.0, 1.0],
                &[1.0],
                &mut OdeOptions {
                    max_step: Some(0.0),
                    ..d()
                },
            ),
            "chimalab:ode:opcaoInvalida",
        ),
        (
            e(
                &[0.0, 1.0],
                &[1.0],
                &mut OdeOptions {
                    initial_step: Some(-1.0),
                    ..d()
                },
            ),
            "chimalab:ode:opcaoInvalida",
        ),
        (
            e(
                &[0.0, 1.0],
                &[1.0],
                &mut OdeOptions {
                    refine: Some(0),
                    ..d()
                },
            ),
            "chimalab:ode:opcaoInvalida",
        ),
    ];
    for (err, id) in casos {
        assert_eq!(err.id(), id, "{err}");
        assert!(!err.to_string().is_empty());
    }
    let err = solve_ivp(|_, _, d| d[0] = f64::NAN, &[0.0, 1.0], &[1.0], &mut d()).unwrap_err();
    assert_eq!(err.id(), "chimalab:ode:condicaoInicialInvalida");
    assert!(
        OdeError::ShapeMismatch {
            expected: 1,
            actual: 2
        }
        .to_string()
        .contains('2')
    );
}

// ---------------------------------------------------------------- propriedades

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Escala por potência de 2: y0 e atol multiplicados por 2^k dão a solução multiplicada por
    /// 2^k, bit a bit (sistema linear; toda a aritmética escala exatamente).
    #[test]
    fn escala_por_potencia_de_dois(k in -60i32..60, lam in -3.0f64..1.0, tf in 0.1f64..4.0) {
        let a = |_: f64, y: &[f64], d: &mut [f64]| {
            d[0] = lam * y[0] + y[1];
            d[1] = -y[0] + 0.5 * lam * y[1];
        };
        let s1 = solve_ivp(a, &[0.0, tf], &[1.0, -0.5], &mut opts(Method::Dopri5, 1e-7, 1e-9)).unwrap();
        let p = 2f64.powi(k);
        let s2 = solve_ivp(a, &[0.0, tf], &[p, -0.5 * p], &mut opts(Method::Dopri5, 1e-7, 1e-9 * p)).unwrap();
        prop_assert_eq!(&s1.t, &s2.t);
        for (y1, y2) in s1.y.iter().zip(&s2.y) {
            prop_assert_eq!(y1[0] * p, y2[0]);
            prop_assert_eq!(y1[1] * p, y2[1]);
        }
    }

    /// y' = λy: erro global relativo limitado pela tolerância (com folga para o acúmulo).
    #[test]
    fn linear_escalar(lam in -5.0f64..1.0, tf in 0.1f64..5.0, tol_exp in 4i32..12) {
        let tol = 10f64.powi(-tol_exp);
        for metodo in [Method::Dopri5, Method::Bs23] {
            let s = solve_ivp(move |_, y, d| d[0] = lam * y[0], &[0.0, tf], &[1.0],
                              &mut opts(metodo, tol, 0.0)).unwrap();
            let ex = (lam * tf).exp();
            let e = ((fim(&s).1[0] - ex) / ex).abs();
            prop_assert!(e <= 100.0 * tol * (1.0 + lam.abs() * tf), "{:?} {}", metodo, e);
        }
    }

    /// Rosenbrock em y' = λy (inclusive muito rígido, λ até −1e6): erro relativo limitado pelo
    /// número de passos vezes a tolerância (ordem 2, erro controlado por passo).
    #[test]
    fn rosenbrock_linear_escalar(lexp in -1.0f64..6.0, tf in 0.1f64..3.0, tol_exp in 3i32..7) {
        let tol = 10f64.powi(-tol_exp);
        let lam = -(10f64.powf(lexp));
        let s = solve_ivp(move |_, y, d| d[0] = lam * y[0], &[0.0, tf], &[1.0],
                          &mut opts(Method::Rosenbrock, tol, tol * 1e-8)).unwrap();
        prop_assert_eq!(s.status, OdeStatus::Completed);
        let ex = (lam * tf).exp();
        let y = fim(&s).1[0];
        // Decaimento limitado: |y| ≤ 1 (estabilidade). O sinal pode inverter em valores minúsculos:
        // R(z) → 0 quando z → −∞, mas pode ser ligeiramente negativo para |z| grande e finito
        // (contraexemplo do proptest: y = −4e-322 com λ = −1,5e5).
        prop_assert!(y.abs() <= 1.0 && y >= -tol, "{}", y);
        // Erro misto (relativo acima de 1e-8, absoluto abaixo), como o das tolerâncias.
        let e = (y - ex).abs() / ex.max(1e-8);
        prop_assert!(e <= 10.0 * tol * s.stats.steps as f64, "{} passos {}", e, s.stats.steps);
    }

    /// Ida e volta: integrar para frente e depois para trás recupera y0 (oscilador).
    #[test]
    fn ida_e_volta(tf in 0.5f64..20.0, a in -2.0f64..2.0, b in -2.0f64..2.0) {
        let osc = |_: f64, y: &[f64], d: &mut [f64]| {
            d[0] = y[1];
            d[1] = -y[0];
        };
        let s = solve_ivp(osc, &[0.0, tf], &[a, b], &mut opts(Method::Dopri5, 1e-11, 1e-11)).unwrap();
        let s2 = solve_ivp(osc, &[tf, 0.0], fim(&s).1, &mut opts(Method::Dopri5, 1e-11, 1e-11)).unwrap();
        prop_assert!(erro(fim(&s2).1, &[a, b]) <= 1e-8);
    }
}

/// Custo (registrado em docs/progresso/F049.md): `--nocapture`.
#[test]
fn custo_por_tolerancia() {
    for nome in ["vanderpol1", "pendulo", "brusselator"] {
        for metodo in [Method::Dopri5, Method::Bs23] {
            for tol in [1e-4, 1e-6, 1e-8, 1e-10] {
                let s = solve_ivp(
                    ref_fn(nome),
                    &[0.0, 20.0],
                    &ref_y0(nome),
                    &mut opts(metodo, tol, tol),
                )
                .unwrap();
                println!(
                    "CUSTO {nome:>12} {metodo:?} tol {tol:e}: passos {} rejeitados {} avaliações {}",
                    s.stats.steps, s.stats.rejected, s.stats.fun_evals
                );
            }
        }
    }
}

/// Caminho exato (passos, rejeições, avaliações): fixa o passo inicial, a norma do erro e o
/// controle PI, que só afetam o custo e não seriam vistos por testes de precisão.
#[test]
fn contagens_fixadas() {
    let vdp = ref_fn("vanderpol1");
    let dec = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    // (nome, método, f, tspan, y0, tol, max_step, (passos, rejeitados, avaliações))
    type Caso = (
        &'static str,
        Method,
        fn(f64, &[f64], &mut [f64]),
        [f64; 2],
        Vec<f64>,
        f64,
        Option<f64>,
        (usize, usize, usize),
    );
    let casos: Vec<Caso> = vec![
        (
            "vdp_dopri",
            Method::Dopri5,
            vdp,
            [0.0, 20.0],
            vec![2.0, 0.0],
            1e-6,
            None,
            (147, 31, 1070),
        ),
        (
            "vdp_bs23",
            Method::Bs23,
            vdp,
            [0.0, 20.0],
            vec![2.0, 0.0],
            1e-6,
            None,
            (936, 10, 2840),
        ),
        (
            "dec_deslocado",
            Method::Dopri5,
            dec,
            [2.0, 7.0],
            vec![1.0],
            1e-8,
            None,
            (38, 0, 230),
        ),
        (
            "dec_para_tras",
            Method::Dopri5,
            dec,
            [7.0, 2.0],
            vec![1.0],
            1e-8,
            None,
            (55, 0, 332),
        ),
        (
            "dec_bs23_deslocado",
            Method::Bs23,
            dec,
            [-3.0, 1.0],
            vec![2.0],
            1e-5,
            None,
            (35, 0, 107),
        ),
        (
            "vdp_passo_max",
            Method::Dopri5,
            vdp,
            [1.0, 6.0],
            vec![2.0, 0.0],
            1e-6,
            Some(0.3),
            (37, 5, 254),
        ),
    ];
    for (nome, metodo, f, tspan, y0, tol, max_step, esperado) in casos {
        let mut o = opts(metodo, tol, tol);
        o.max_step = max_step;
        let s = solve_ivp(f, &tspan, &y0, &mut o).unwrap();
        let c = (s.stats.steps, s.stats.rejected, s.stats.fun_evals);
        assert_eq!(c, esperado, "{nome}");
        // Avaliações: 2 no início (f₀ e o teste do passo inicial) + (estágios − 1) por tentativa.
        let estagios = if metodo == Method::Dopri5 { 6 } else { 3 };
        assert_eq!(c.2, 2 + estagios * (c.0 + c.1), "{nome}");
    }
}

#[test]
fn bordas_de_desempate() {
    let dec = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    // Tolerância relativa exatamente 100ε: aceita sem aviso.
    let s = solve_ivp(
        dec,
        &[0.0, 1.0],
        &[1.0],
        &mut opts(Method::Dopri5, 100.0 * f64::EPSILON, 1e-12),
    )
    .unwrap();
    assert!(s.warnings.is_empty());
    // Passo inicial nulo: erro.
    let e = solve_ivp(
        dec,
        &[0.0, 1.0],
        &[1.0],
        &mut OdeOptions {
            initial_step: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(e.id(), "chimalab:ode:opcaoInvalida");
    // g que começa em 0 e se afasta dele não dispara, em nenhum sentido.
    for (g, dirn) in [
        (-1.0, Direction::Falling),
        (-1.0, Direction::Both),
        (1.0, Direction::Rising),
        (1.0, Direction::Both),
    ] {
        let mut o = OdeOptions::default();
        o.events.push(Event::new(move |t, _| g * t).direction(dirn));
        let s = solve_ivp(dec, &[0.0, 1.0], &[1.0], &mut o).unwrap();
        assert!(s.events.is_empty(), "{g} {dirn:?}");
    }
}

// ================================================================ incremento 2: rígidos

fn vdp(mu: f64) -> impl Fn(f64, &[f64], &mut [f64]) + Copy {
    move |_, y, d| {
        d[0] = y[1];
        d[1] = mu * (1.0 - y[0] * y[0]) * y[1] - y[0];
    }
}

fn robertson(_: f64, y: &[f64], d: &mut [f64]) {
    d[0] = -0.04 * y[0] + 1e4 * y[1] * y[2];
    d[1] = 0.04 * y[0] - 1e4 * y[1] * y[2] - 3e7 * y[1] * y[1];
    d[2] = 3e7 * y[1] * y[1];
}

fn rigido(metodo: Method, rtol: f64, atol: f64) -> OdeOptions<'static> {
    opts(metodo, rtol, atol)
}

/// Van der Pol com μ = 1000 em [0, 3000] (oscilação de relaxação). O período tem a assíntota
/// T(μ) = (3 − 2 ln 2)μ + 3a·μ^{−1/3} − (2/3)(ln μ)/μ + O(μ⁻¹), com a = 2,338107… o primeiro zero
/// de Ai(−x) (J. Grasman, *Asymptotic Methods for Relaxation Oscillations*, 1987). Para μ = 1000,
/// T ≈ 1614,8728, com resto O(1e-3).
#[test]
fn ca1_van_der_pol_rigido_por_bdf() {
    let mu = 1000.0;
    let periodo = (3.0 - 2.0 * 2f64.ln()) * mu + 3.0 * 2.338_107_410_459_767 * mu.powf(-1.0 / 3.0)
        - 2.0 / 3.0 * mu.ln() / mu;
    for metodo in [Method::Bdf, Method::Ndf, Method::Radau] {
        let mut o = rigido(metodo, 1e-8, 1e-10);
        // Cruzamentos descendentes de y₁ = 0: um por período.
        o.events
            .push(Event::new(|_, y| y[0]).direction(Direction::Falling));
        let s = solve_ivp(vdp(mu), &[0.0, 3500.0], &[2.0, 0.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Completed);
        // Um explícito precisaria de ~10⁶ passos (estabilidade, |λ| ~ 2μ); o implícito acompanha
        // só a escala da solução.
        assert!(s.stats.steps < 6000, "{metodo:?}: {:?}", s.stats);
        let cruz: Vec<f64> = s.events.iter().map(|e| e.t).collect();
        assert_eq!(cruz.len(), 2, "{metodo:?}");
        let t_medido = cruz[1] - cruz[0];
        // Medido 1614,4006 contra 1614,4025: a diferença está dentro do resto O(1e-3).
        assert!(
            (t_medido - periodo).abs() <= 0.01,
            "{metodo:?}: {t_medido} vs {periodo}"
        );
        println!(
            "CA1 {metodo:?}: período {t_medido} (assíntota {periodo}), {:?}",
            s.stats
        );
    }
}

/// Robertson (cinética química, rigidez ~10⁴ a 10⁷ ao longo do tempo). Referência: DOPRI5 com
/// tolerância 1e-10 (método explícito independente do código implícito, validado à parte; o
/// passo pequeno que a estabilidade exige não prejudica a precisão). Concorda com o valor clássico
/// y₁(40) = 0,7158270687… a 1e-13.
#[test]
fn robertson_contra_dopri5() {
    let atol = |tol: f64| AbsTol::Vector(vec![tol * 1e-4, tol * 1e-8, tol * 1e-4]);
    let ts = [0.0, 0.4, 4.0, 40.0];
    let mut o = opts(Method::Dopri5, 1e-10, 0.0);
    o.abs_tol = Some(atol(1e-10));
    let r = solve_ivp(robertson, &ts, &[1.0, 0.0, 0.0], &mut o).unwrap();
    assert!((r.y[3][0] - 0.715_827_068_719_4).abs() <= 1e-12);
    for metodo in [Method::Ndf, Method::Bdf, Method::Radau] {
        let mut anterior = f64::INFINITY;
        for tol in [1e-6, 1e-8, 1e-10] {
            let mut o = rigido(metodo, tol, 0.0);
            o.abs_tol = Some(atol(tol));
            let s = solve_ivp(robertson, &ts, &[1.0, 0.0, 0.0], &mut o).unwrap();
            assert_eq!(s.status, OdeStatus::Completed);
            let mut e = 0.0f64;
            for (y, yr) in s.y.iter().zip(&r.y) {
                for c in 0..3 {
                    // Erro relativo por componente (y₂ ~ 1e-5).
                    e = e.max((y[c] - yr[c]).abs() / yr[c].abs().max(1e-300));
                }
                // Conservação de massa: a soma é invariante linear, preservada pelos métodos
                // lineares de passo múltiplo até o arredondamento e a tolerância de Newton.
                assert!((y.iter().sum::<f64>() - 1.0).abs() <= 1e-14);
            }
            assert!(e <= 100.0 * tol, "{metodo:?} tol {tol}: {e:e}");
            assert!(e < anterior, "{metodo:?}");
            anterior = e;
            assert!(s.stats.steps < 2000, "{:?}", s.stats);
        }
    }
}

/// Sistema linear rígido y' = Ay com A = QΛQᵀ, Λ = diag(−1, −10³, −10⁶) e Q uma rotação:
/// solução exata Q·e^{Λt}·Qᵀy₀. Jacobiano dado e por diferenças, todas as ordens máximas.
#[test]
fn linear_rigido_exato() {
    let (c, s) = (0.6f64, 0.8f64);
    // Q = rotação em (1,2) seguida de rotação em (2,3).
    let q = [[c, -s * c, s * s], [s, c * c, -c * s], [0.0, s, c]];
    let lam = [-1.0, -1e3, -1e6];
    let a: Vec<Vec<f64>> = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| (0..3).map(|k| q[i][k] * lam[k] * q[j][k]).sum())
                .collect()
        })
        .collect();
    let a2 = a.clone();
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        for i in 0..3 {
            d[i] = (0..3).map(|j| a2[i][j] * y[j]).sum();
        }
    };
    let y0 = [1.0, 2.0, -1.0];
    let exata = |t: f64| -> Vec<f64> {
        let z: Vec<f64> = (0..3)
            .map(|k| (0..3).map(|j| q[j][k] * y0[j]).sum::<f64>() * (lam[k] * t).exp())
            .collect();
        (0..3).map(|i| (0..3).map(|k| q[i][k] * z[k]).sum()).collect()
    };
    let ts: Vec<f64> = vec![0.0, 1e-4, 1e-3, 0.01, 0.1, 1.0, 5.0];
    // passos[método][ordem máxima − 1]
    let mut passos = [[0usize; 5]; 2];
    for (im, metodo) in [Method::Ndf, Method::Bdf].into_iter().enumerate() {
        for max_order in 1..=5 {
            for dado in [false, true] {
                // Ordens baixas precisam de tolerância frouxa para um número razoável de passos
                // (Euler implícito: passos ~ 1/tol).
                let tol = [1e-4, 1e-6, 1e-7, 1e-7, 1e-7][max_order - 1];
                // atol = 1e-9: com A de norma 10⁶, f = Ay tem erro de arredondamento ~1e-10 em
                // componentes que passam por zero; atol abaixo disso faz o Newton estagnar no
                // ruído (medido: com 1e-12, ~550 Jacobianos recalculados em vez de 1).
                let mut o = rigido(metodo, tol, 1e-9);
                o.max_order = Some(max_order);
                if dado {
                    let a3 = a.clone();
                    o.jacobian = Some(Box::new(move |_, _, j| {
                        for (r, linha) in a3.iter().enumerate() {
                            for (cc, v) in linha.iter().enumerate() {
                                j.set(r, cc, *v);
                            }
                        }
                    }));
                }
                let s = solve_ivp(f.clone(), &ts, &y0, &mut o).unwrap();
                assert_eq!(s.status, OdeStatus::Completed);
                let mut emax = 0.0f64;
                for (t, y) in s.t.iter().zip(&s.y) {
                    let ex = exata(*t);
                    emax = emax.max((0..3).map(|i| (y[i] - ex[i]).abs()).fold(0.0, f64::max));
                }
                // Erro global ~ 100·tol (mais na ordem 1, que dá ~1000 passos).
                let lim = [1e-2, 1e-4, 5e-6, 5e-6, 5e-6][max_order - 1];
                assert!(
                    emax <= lim,
                    "{metodo:?} ordem ≤ {max_order} dado {dado}: {emax:e}"
                );
                // Linear com Jacobiano exato: o Newton converge sem recalcular o Jacobiano.
                assert!(s.stats.jac_evals <= if dado { 1 } else { 2 }, "{:?}", s.stats);
                passos[im][max_order - 1] = s.stats.steps;
            }
        }
    }
    // Ordens mais altas dão menos passos, e as NDF menos que as BDF em toda ordem (Shampine &
    // Reichelt: ~20% a menos com a mesma estabilidade). Medido: NDF 1087, 1025, 631, 401, 331;
    // BDF 1361, 1277, 776, 444, 340.
    for p in &passos {
        assert!(p.windows(2).all(|w| w[1] < w[0]), "{p:?}");
    }
    assert!((0..5).all(|k| passos[0][k] < passos[1][k]), "{passos:?}");
}

/// BDF com eventos, saída densa, instantes pedidos e integração para trás.
#[test]
fn rigidos_eventos_e_saida() {
    let dec = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    for metodo in [Method::Ndf, Method::Bdf, Method::Radau, Method::Rosenbrock] {
        // Evento terminal em y = 0,5: t = ln 2.
        let mut o = rigido(metodo, 1e-9, 1e-12);
        o.dense = true;
        o.events.push(Event::new(|_, y| y[0] - 0.5).terminal());
        let s = solve_ivp(dec, &[0.0, 5.0], &[1.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Terminated);
        // Rosenbrock (ordem 2, erro controlado por passo): o erro global cresce com o número de
        // passos, ~700·tol aqui (medido: 7,5e-7 no instante do evento).
        let ros = metodo == Method::Rosenbrock;
        assert!(
            (s.events[0].t - 2f64.ln()).abs() <= if ros { 2e-6 } else { 1e-8 },
            "{metodo:?}: {}",
            s.events[0].t
        );
        // Saída densa no meio dos passos: polinômio das diferenças (BDF) ou de colocação (Radau, de
        // ordem 3, a ordem de estágio, abaixo da ordem 5 do passo).
        for k in 0..=60 {
            let t = s.events[0].t * k as f64 / 60.0;
            let y = s.eval(t).unwrap()[0];
            assert!(
                (y - (-t).exp()).abs()
                    <= match metodo {
                        Method::Radau => 1e-7,
                        Method::Rosenbrock => 2e-6,
                        _ => 1e-8,
                    },
                "{metodo:?} t {t}: {}",
                y - (-t).exp()
            );
        }
        // Instantes pedidos e para trás.
        let ts: Vec<f64> = (0..=20).map(|k| 3.0 - 0.15 * k as f64).collect();
        let s = solve_ivp(
            |_, y, d| d[0] = y[0],
            &ts,
            &[3f64.exp()],
            &mut rigido(metodo, 1e-9, 1e-12),
        )
        .unwrap();
        assert_eq!(s.t, ts);
        for (t, y) in s.t.iter().zip(&s.y) {
            assert!(
                (y[0] - t.exp()).abs() <= if ros { 5e-6 } else { 1e-7 } * t.exp(),
                "{metodo:?} {t}: {}",
                (y[0] - t.exp()).abs() / t.exp()
            );
        }
    }
}

#[test]
fn rigidos_falhas_e_erros() {
    for metodo in [Method::Ndf, Method::Bdf, Method::Radau, Method::Rosenbrock] {
        // Explosão: passo mínimo.
        let s = solve_ivp(
            |_, y, d| d[0] = y[0] * y[0],
            &[0.0, 2.0],
            &[1.0],
            &mut rigido(metodo, 1e-6, 1e-9),
        )
        .unwrap();
        assert_eq!(s.status, OdeStatus::StepTooSmall, "{metodo:?}");
        assert!((fim(&s).0 - 1.0).abs() <= 1e-3, "{}", fim(&s).0);
        // f não finito além de t = 1.
        let s = solve_ivp(
            |t, _, d| d[0] = (1.0 - t).sqrt(),
            &[0.0, 2.0],
            &[0.0],
            &mut rigido(metodo, 1e-6, 1e-9),
        )
        .unwrap();
        // Rosenbrock: perto de t = 1, ∂f/∂t → ∞ e a verificação da linearização recusa os passos
        // antes de f ficar não finito; as duas falhas param no mesmo lugar.
        if metodo == Method::Rosenbrock {
            assert!(matches!(s.status, OdeStatus::NonFinite | OdeStatus::StepTooSmall));
        } else {
            assert_eq!(s.status, OdeStatus::NonFinite, "{metodo:?}");
        }
        // Regressão: o Jacobiano por diferenças calculado além de t = 1 dava NaN e travava todos
        // os passos seguintes (parava em t ≈ 0,998). Agora o Jacobiano finito anterior é mantido.
        assert!((fim(&s).0 - 1.0).abs() <= 1e-9, "{}", fim(&s).0);
        assert!((fim(&s).1[0] - 2.0 / 3.0).abs() <= 1e-4);
        // Limite de passos.
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.max_steps = Some(5);
        let s = solve_ivp(vdp(1000.0), &[0.0, 10.0], &[2.0, 0.0], &mut o).unwrap();
        assert_eq!((s.status, s.stats.steps), (OdeStatus::MaxSteps, 5));
        // Ordem máxima inválida.
        for mo in [0, 6] {
            let mut o = rigido(metodo, 1e-6, 1e-9);
            o.max_order = Some(mo);
            let e = solve_ivp(vdp(1.0), &[0.0, 1.0], &[2.0, 0.0], &mut o).unwrap_err();
            assert_eq!(e.id(), "chimalab:ode:opcaoInvalida");
        }
    }
}

#[test]
fn rigidos_bordas() {
    for metodo in [Method::Ndf, Method::Bdf, Method::Radau, Method::Rosenbrock] {
        // Componente nula constante com atol = 0: escala nula, erro exatamente 0 aceito.
        let s = solve_ivp(
            |_, y, d| {
                d[0] = -y[0];
                d[1] = 0.0;
            },
            &[0.0, 1.0],
            &[1.0, 0.0],
            &mut rigido(metodo, 1e-6, 0.0),
        )
        .unwrap();
        assert_eq!(s.status, OdeStatus::Completed);
        assert_eq!(fim(&s).1[1], 0.0);
        // Rosenbrock: ordem 2 com erro por passo, ~100 passos (medido abaixo de 1e-4).
        let lim = if metodo == Method::Rosenbrock { 1e-4 } else { 1e-5 };
        assert!(
            (fim(&s).1[0] - (-1f64).exp()).abs() <= lim,
            "{metodo:?}: {}",
            (fim(&s).1[0] - (-1f64).exp()).abs()
        );
        // Passo máximo respeitado.
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.max_step = Some(0.05);
        o.refine = Some(1);
        let s = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 2.0], &[1.0], &mut o).unwrap();
        assert!(s.t.windows(2).all(|w| w[1] - w[0] <= 0.05 * (1.0 + 1e-12)));
        assert!(s.stats.steps >= 40);
        // Jacobiano absurdo (1e308): a iteração de Newton não avança. Falha controlada por passo
        // mínimo, nunca um resultado errado com estado de sucesso.
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, 1e308)));
        let s = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 1.0], &[1.0], &mut o).unwrap();
        // Regressão: o passo de Newton sumia no arredondamento (dy = 0) e o preditor era aceito,
        // dando 1e-16 em vez de e⁻¹ com estado Completed.
        assert_eq!(s.status, OdeStatus::StepTooSmall, "{metodo:?}: {:?}", fim(&s));
        // Com passo inicial 10, c·J estoura para ∞ e a LU tem pivô não finito: mesma falha.
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, 1e308)));
        o.initial_step = Some(10.0);
        let s = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 100.0], &[1.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::StepTooSmall);
    }
}

/// Radau no sistema linear rígido exato, com Jacobiano dado e por diferenças.
#[test]
fn radau_linear_rigido_exato() {
    let (c, s) = (0.6f64, 0.8f64);
    let q = [[c, -s * c, s * s], [s, c * c, -c * s], [0.0, s, c]];
    let lam = [-1.0, -1e3, -1e6];
    let a: Vec<Vec<f64>> = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| (0..3).map(|k| q[i][k] * lam[k] * q[j][k]).sum())
                .collect()
        })
        .collect();
    let a2 = a.clone();
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        for i in 0..3 {
            d[i] = (0..3).map(|j| a2[i][j] * y[j]).sum();
        }
    };
    let y0 = [1.0, 2.0, -1.0];
    let exata = |t: f64| -> Vec<f64> {
        let z: Vec<f64> = (0..3)
            .map(|k| (0..3).map(|j| q[j][k] * y0[j]).sum::<f64>() * (lam[k] * t).exp())
            .collect();
        (0..3).map(|i| (0..3).map(|k| q[i][k] * z[k]).sum()).collect()
    };
    let ts: Vec<f64> = vec![0.0, 1e-4, 1e-3, 0.01, 0.1, 1.0, 5.0];
    for tol in [1e-5, 1e-8] {
        for dado in [false, true] {
            let mut o = rigido(Method::Radau, tol, 1e-9);
            if dado {
                let a3 = a.clone();
                o.jacobian = Some(Box::new(move |_, _, j| {
                    for (r, linha) in a3.iter().enumerate() {
                        for (cc, v) in linha.iter().enumerate() {
                            j.set(r, cc, *v);
                        }
                    }
                }));
            }
            let s = solve_ivp(f.clone(), &ts, &y0, &mut o).unwrap();
            assert_eq!(s.status, OdeStatus::Completed);
            let mut emax = 0.0f64;
            for (t, y) in s.t.iter().zip(&s.y) {
                let ex = exata(*t);
                emax = emax.max((0..3).map(|i| (y[i] - ex[i]).abs()).fold(0.0, f64::max));
            }
            println!("RADLIN tol {tol:e} dado {dado}: {emax:e} {:?}", s.stats);
            assert!(emax <= 100.0 * tol, "tol {tol} dado {dado}: {emax:e}");
        }
    }
}

/// DAE de índice 1 com solução fechada: y₁' = −y₁ + y₂, 0 = y₂ − sen t, y₁(0) = 1:
/// y₂ = sen t e y₁ = (sen t − cos t)/2 + (3/2)e^{−t}. M = diag(1, 0).
#[test]
fn dae_indice_1_analitica() {
    use chimalab_math::linalg::Matrix;
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 0, 1.0);
    for tol in [1e-6, 1e-9] {
        let mut o = rigido(Method::Radau, tol, tol * 1e-2);
        o.mass = Some(m.clone());
        o.dense = true;
        let ts: Vec<f64> = (0..=40).map(|k| 0.25 * k as f64).collect();
        let s = solve_ivp(
            |t, y, d| {
                d[0] = -y[0] + y[1];
                d[1] = y[1] - t.sin();
            },
            &ts,
            &[1.0, 0.0],
            &mut o,
        )
        .unwrap();
        assert_eq!(s.status, OdeStatus::Completed);
        for (t, y) in s.t.iter().zip(&s.y) {
            let y1 = (t.sin() - t.cos()) / 2.0 + 1.5 * (-t).exp();
            assert!(
                (y[0] - y1).abs() <= 30.0 * tol,
                "tol {tol} t {t}: {:e}",
                y[0] - y1
            );
            // Erro global ~ 10–20·tol: o Radau controla rtol' = 0,1·rtol^{2/3} por passo.
            // A componente algébrica sai da equação de restrição a cada estágio.
            assert!((y[1] - t.sin()).abs() <= 30.0 * tol, "tol {tol} t {t}");
        }
        // Saída densa pelo polinômio de colocação.
        for k in 0..=100 {
            let t = 0.1 * k as f64;
            let y = s.eval(t).unwrap();
            assert!((y[0] - ((t.sin() - t.cos()) / 2.0 + 1.5 * (-t).exp())).abs() <= 50.0 * tol);
        }
    }
}

/// Robertson como DAE (y₃ algébrica: y₁ + y₂ + y₃ = 1) contra a forma EDO.
#[test]
fn dae_robertson_contra_edo() {
    use chimalab_math::linalg::Matrix;
    let mut m = Matrix::zeros(3, 3);
    m.set(0, 0, 1.0);
    m.set(1, 1, 1.0);
    let ts = [0.0, 0.4, 4.0, 40.0, 400.0];
    let atol = AbsTol::Vector(vec![1e-12, 1e-16, 1e-12]);
    let mut o = rigido(Method::Radau, 1e-10, 0.0);
    o.abs_tol = Some(atol.clone());
    let edo = solve_ivp(robertson, &ts, &[1.0, 0.0, 0.0], &mut o).unwrap();
    let mut o = rigido(Method::Radau, 1e-10, 0.0);
    o.abs_tol = Some(atol);
    o.mass = Some(m);
    let dae = solve_ivp(
        |_, y, d| {
            d[0] = -0.04 * y[0] + 1e4 * y[1] * y[2];
            d[1] = 0.04 * y[0] - 1e4 * y[1] * y[2] - 3e7 * y[1] * y[1];
            d[2] = y[0] + y[1] + y[2] - 1.0;
        },
        &ts,
        &[1.0, 0.0, 0.0],
        &mut o,
    )
    .unwrap();
    assert_eq!(dae.status, OdeStatus::Completed);
    for (a, b) in edo.y.iter().zip(&dae.y) {
        for c in 0..3 {
            assert!(
                (a[c] - b[c]).abs() <= 1e-8 * b[c].abs().max(1e-6),
                "{c}: {} vs {}",
                a[c],
                b[c]
            );
        }
        assert!((b.iter().sum::<f64>() - 1.0).abs() <= 1e-14);
    }
}

/// Massa não singular: M y' = f equivale a y' = M⁻¹f.
#[test]
fn massa_nao_singular_equivale_a_edo() {
    use chimalab_math::linalg::Matrix;
    // M = [[2, 1], [1, 3]]; y' = M⁻¹·(−y) tem solução exp(−M⁻¹t)·y₀; comparamos com DOPRI5 na
    // forma explícita.
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 0, 2.0);
    m.set(0, 1, 1.0);
    m.set(1, 0, 1.0);
    m.set(1, 1, 3.0);
    let mut o = rigido(Method::Radau, 1e-10, 1e-12);
    o.mass = Some(m);
    let s = solve_ivp(
        |_, y, d| {
            d[0] = -y[0];
            d[1] = -y[1];
        },
        &[0.0, 1.0, 3.0],
        &[1.0, -1.0],
        &mut o,
    )
    .unwrap();
    // M⁻¹ = [[3, −1], [−1, 2]]/5.
    let r = solve_ivp(
        |_, y, d| {
            d[0] = -(3.0 * y[0] - y[1]) / 5.0;
            d[1] = -(-y[0] + 2.0 * y[1]) / 5.0;
        },
        &[0.0, 1.0, 3.0],
        &[1.0, -1.0],
        &mut opts(Method::Dopri5, 1e-12, 1e-14),
    )
    .unwrap();
    for (a, b) in s.y.iter().zip(&r.y) {
        // rtol 1e-10: erro global ~10·tol.
        assert!(erro(a, b) <= 1e-8, "{a:?} vs {b:?}");
    }
}

#[test]
fn massa_erros() {
    use chimalab_math::linalg::Matrix;
    let f = |_: f64, y: &[f64], d: &mut [f64]| d[0] = -y[0];
    let mut o = rigido(Method::Ndf, 1e-6, 1e-9);
    o.mass = Some(Matrix::zeros(1, 1));
    assert_eq!(
        solve_ivp(f, &[0.0, 1.0], &[1.0], &mut o).unwrap_err().id(),
        "chimalab:ode:opcaoInvalida"
    );
    let mut o = rigido(Method::Radau, 1e-6, 1e-9);
    o.mass = Some(Matrix::zeros(2, 2));
    assert_eq!(
        solve_ivp(f, &[0.0, 1.0], &[1.0], &mut o).unwrap_err().id(),
        "chimalab:ode:formaIncompativel"
    );
    let mut m = Matrix::zeros(1, 1);
    m.set(0, 0, f64::NAN);
    let mut o = rigido(Method::Radau, 1e-6, 1e-9);
    o.mass = Some(m);
    assert_eq!(
        solve_ivp(f, &[0.0, 1.0], &[1.0], &mut o).unwrap_err().id(),
        "chimalab:ode:opcaoInvalida"
    );
}

/// Jacobiano NaN do usuário: descartado (fica o anterior, nulo). Sem Jacobiano útil, o Newton
/// vira iteração de ponto fixo, converge devagar ou diverge, e o passo encolhe até convergir:
/// o resultado continua correto, só mais caro.
#[test]
fn jacobiano_nan_e_descartado() {
    for metodo in [Method::Radau, Method::Ndf] {
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, f64::NAN)));
        let s = solve_ivp(|_, y, d| d[0] = -50.0 * y[0], &[0.0, 1.0], &[1.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Completed, "{metodo:?}");
        assert!(
            (fim(&s).1[0] - (-50f64).exp()).abs() <= 1e-8,
            "{metodo:?}: {}",
            fim(&s).1[0]
        );
        assert!(s.stats.rejected > 0, "{metodo:?}: {:?}", s.stats);
    }
}

/// Primeiro passo aceito, bit a bit, nos ramos do algoritmo de passo inicial (Hairer, Nørsett &
/// Wanner, §II.4): normal; ‖y₀‖ pequeno; ‖f₀‖ pequeno; f identicamente nula; f(t₀ + h₀) não
/// finito; ordem 3; integração para trás.
#[test]
fn passo_inicial_fixado() {
    type Caso = (
        &'static str,
        Method,
        fn(f64, &[f64], &mut [f64]),
        [f64; 2],
        Vec<f64>,
        f64,
    );
    let casos: Vec<Caso> = vec![
        (
            "normal",
            Method::Dopri5,
            ref_fn("vanderpol1"),
            [0.0, 20.0],
            vec![2.0, 0.0],
            0.0004997501249375313,
        ),
        (
            "normal_bs23",
            Method::Bs23,
            ref_fn("vanderpol1"),
            [0.0, 20.0],
            vec![2.0, 0.0],
            0.0004997501249375313,
        ),
        (
            "para_tras",
            Method::Dopri5,
            ref_fn("vanderpol1"),
            [0.0, -20.0],
            vec![2.0, 0.0],
            -0.0004997501249375313,
        ),
        (
            "y0_nulo",
            Method::Dopri5,
            |_, _, d| d[0] = 1.0,
            [0.0, 1.0],
            vec![0.0],
            9.999999999999999e-5,
        ),
        (
            "f_nula",
            Method::Dopri5,
            |_, _, d| d[0] = 0.0,
            [0.0, 1.0],
            vec![1.0],
            1e-6,
        ),
        (
            "f0_pequena",
            Method::Dopri5,
            |t, _, d| d[0] = t,
            [0.0, 1.0],
            vec![1.0],
            9.999999999999999e-5,
        ),
        (
            "nao_finito",
            Method::Dopri5,
            |t, _, d| d[0] = (1e-3 - t).sqrt(),
            [0.0, 1.0],
            vec![1.0],
            0.0003224772247451167,
        ),
        (
            "decaimento",
            Method::Dopri5,
            |_, y, d| d[0] = -y[0],
            [0.0, 5.0],
            vec![1.0],
            0.04642362109615998,
        ),
        (
            "decaimento_bs23",
            Method::Bs23,
            |_, y, d| d[0] = -y[0],
            [0.0, 5.0],
            vec![1.0],
            0.010002499063046497,
        ),
        (
            "ndf",
            Method::Ndf,
            ref_fn("vanderpol1"),
            [0.0, 20.0],
            vec![2.0, 0.0],
            1.5352597732137486e-6,
        ),
    ];
    for (nome, metodo, f, tspan, y0, h1) in casos {
        let mut o = opts(metodo, 1e-6, 1e-9);
        o.refine = Some(1);
        let s = solve_ivp(f, &tspan, &y0, &mut o).unwrap();
        assert_eq!(s.t[1] - s.t[0], h1, "{nome}");
    }
}

/// Caminho exato dos implícitos (passos, rejeições, avaliações, Jacobianos, LUs): fixa as
/// heurísticas de Newton, de escolha de ordem e de reaproveitamento do Jacobiano e da LU.
#[test]
fn rigidos_contagens_fixadas() {
    let rob_atol = || Some(AbsTol::Vector(vec![1e-10, 1e-14, 1e-10]));
    for metodo in [Method::Ndf, Method::Bdf, Method::Radau] {
        let s = solve_ivp(
            vdp(1000.0),
            &[0.0, 100.0],
            &[2.0, 0.0],
            &mut rigido(metodo, 1e-6, 1e-9),
        )
        .unwrap();
        let mut o = rigido(metodo, 1e-6, 0.0);
        o.abs_tol = rob_atol();
        let r = solve_ivp(robertson, &[0.0, 40.0], &[1.0, 0.0, 0.0], &mut o).unwrap();
        let mut o = rigido(metodo, 1e-8, 1e-10);
        o.max_order = (metodo != Method::Radau).then_some(3);
        let l = solve_ivp(vdp(10.0), &[0.0, 5.0], &[2.0, 0.0], &mut o).unwrap();
        let st = |x: &OdeSolution| {
            let t = x.stats;
            (t.steps, t.rejected, t.fun_evals, t.jac_evals, t.lu_decomps)
        };
        let esperado = match metodo {
            Method::Ndf => [(96, 0, 204, 2, 25), (177, 3, 463, 5, 42), (385, 0, 933, 4, 102)],
            Method::Bdf => [(101, 1, 215, 2, 26), (186, 1, 519, 5, 41), (473, 3, 1123, 3, 125)],
            _ => [(27, 1, 143, 1, 42), (55, 1, 579, 44, 110), (64, 1, 538, 18, 70)],
        };
        assert_eq!([st(&s), st(&r), st(&l)], esperado, "{metodo:?}");
    }
}

/// Contagens exatas de caminhos raros, apontados pelo teste de mutação: intervalo com t₀ < 0,
/// passo inicial pequeno com matriz de massa, atol = 0 com y₀ = 0, passo máximo nos implícitos,
/// Jacobianos absurdos ou NaN e f não finito com n = 2.
#[test]
fn contagens_de_caminhos_raros() {
    use chimalab_math::linalg::Matrix;
    type C = (usize, usize, usize, usize, usize);
    let st = |s: &OdeSolution| -> C {
        let t = s.stats;
        (t.steps, t.rejected, t.fun_evals, t.jac_evals, t.lu_decomps)
    };
    // Vão padrão |t_f − t₀| com t₀ < 0 (passo máximo = 11).
    let s = solve_ivp(
        |_, _, d| d[0] = 1.0,
        &[-10.0, 1.0],
        &[0.0],
        &mut OdeOptions::default(),
    )
    .unwrap();
    assert_eq!(st(&s), (6, 0, 38, 0, 0));
    // Com massa, o passo inicial é 1e-6 (não a estimativa explícita).
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 0, 1.0);
    let mut o = rigido(Method::Radau, 1e-6, 1e-8);
    o.mass = Some(m);
    let s = solve_ivp(
        |t, y, d| {
            d[0] = -y[0] + y[1];
            d[1] = y[1] - t.sin();
        },
        &[0.0, 10.0],
        &[1.0, 0.0],
        &mut o,
    )
    .unwrap();
    assert_eq!(st(&s), (56, 1, 281, 1, 48));
    // atol = 0 e y₀ = 0: escala nula; ‖y₀‖ = 0 dá h₀ = 1e-6 e o passo cresce.
    let s = solve_ivp(
        |_, _, d| d[0] = 1.0,
        &[0.0, 1.0],
        &[0.0],
        &mut opts(Method::Dopri5, 1e-6, 0.0),
    )
    .unwrap();
    assert_eq!(s.t[1] - s.t[0], 2.5e-7);
    let esperado: [(Method, [C; 5]); 3] = [
        (
            Method::Ndf,
            [
                (53, 0, 109, 1, 14),
                (2, 92, 97, 2, 94),
                (40, 167, 225, 21, 207),
                (199, 58, 701, 22, 106),
                (62, 50, 260, 19, 58),
            ],
        ),
        (
            Method::Bdf,
            [
                (52, 0, 107, 1, 13),
                (2, 92, 97, 2, 94),
                (40, 167, 225, 21, 207),
                (203, 68, 734, 25, 118),
                (69, 51, 304, 26, 58),
            ],
        ),
        (
            Method::Radau,
            [
                (41, 0, 182, 1, 82),
                (0, 1066, 3056, 1, 2132),
                (0, 1074, 3079, 1, 2148),
                (86, 26, 1156, 86, 224),
                (34, 63, 378, 21, 190),
            ],
        ),
    ];
    for (metodo, e) in esperado {
        let mut o = rigido(metodo, 1e-6, 1e-9);
        o.max_step = Some(0.05);
        o.refine = Some(1);
        let s0 = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 2.0], &[1.0], &mut o).unwrap();
        let mut a1 = rigido(metodo, 1e-6, 1e-9);
        a1.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, 1e308)));
        let s1 = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 1.0], &[1.0], &mut a1).unwrap();
        let mut a2 = rigido(metodo, 1e-6, 1e-9);
        a2.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, 1e308)));
        a2.initial_step = Some(10.0);
        let s2 = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 100.0], &[1.0], &mut a2).unwrap();
        let mut a3 = rigido(metodo, 1e-6, 1e-9);
        a3.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, f64::NAN)));
        let s3 = solve_ivp(|_, y, d| d[0] = -50.0 * y[0], &[0.0, 1.0], &[1.0], &mut a3).unwrap();
        let s4 = solve_ivp(
            |t, _, d| {
                d[0] = (1.0 - t).sqrt();
                d[1] = 1.0;
            },
            &[0.0, 2.0],
            &[0.0, 0.0],
            &mut rigido(metodo, 1e-6, 1e-9),
        )
        .unwrap();
        assert_eq!(s4.status, OdeStatus::NonFinite);
        assert_eq!([st(&s0), st(&s1), st(&s2), st(&s3), st(&s4)], e, "{metodo:?}");
    }
    // Forma da massa errada: os campos do erro.
    let mut o = rigido(Method::Radau, 1e-6, 1e-9);
    o.mass = Some(Matrix::zeros(2, 2));
    let e = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 1.0], &[1.0], &mut o).unwrap_err();
    assert_eq!(
        e,
        OdeError::ShapeMismatch {
            expected: 1,
            actual: 4
        }
    );
}

// ================================================================ incremento 3: Rosenbrock

/// Rosenbrock 2(3) com tolerâncias folgadas, seu uso típico: Robertson contra DOPRI5 a 1e-10,
/// Van der Pol rígido (período contra a assíntota) e o sistema linear rígido exato, com
/// Jacobiano dado e por diferenças.
#[test]
fn rosenbrock_rigidos() {
    let atol = |tol: f64| AbsTol::Vector(vec![tol * 1e-4, tol * 1e-8, tol * 1e-4]);
    let ts = [0.0, 0.4, 4.0, 40.0];
    let mut o = opts(Method::Dopri5, 1e-10, 0.0);
    o.abs_tol = Some(atol(1e-10));
    let r = solve_ivp(robertson, &ts, &[1.0, 0.0, 0.0], &mut o).unwrap();
    let mut anterior = f64::INFINITY;
    for tol in [1e-3, 1e-4, 1e-5, 1e-6] {
        let mut o = rigido(Method::Rosenbrock, tol, 0.0);
        o.abs_tol = Some(atol(tol));
        let s = solve_ivp(robertson, &ts, &[1.0, 0.0, 0.0], &mut o).unwrap();
        assert_eq!(s.status, OdeStatus::Completed);
        let mut e = 0.0f64;
        for (y, yr) in s.y.iter().zip(&r.y) {
            for c in 0..3 {
                e = e.max((y[c] - yr[c]).abs() / yr[c].abs().max(1e-300));
            }
            // Invariante linear: o método é linearmente implícito e preserva a soma.
            assert!((y.iter().sum::<f64>() - 1.0).abs() <= 1e-13);
        }
        println!("ROS Robertson tol {tol}: erro {e:.2e}, {:?}", s.stats);
        assert!(e <= 100.0 * tol, "tol {tol}: {e:e}");
        assert!(e < anterior);
        anterior = e;
        assert!(s.stats.steps < 2000, "{:?}", s.stats);
    }
    // Van der Pol μ = 1000: período contra a assíntota (resto O(1e-3)). Medido: diferença 0,093
    // com rtol 1e-5 e 0,020 com 1e-6 (13 568 passos; o método é de ordem 2).
    let mu = 1000.0;
    let periodo = (3.0 - 2.0 * 2f64.ln()) * mu + 3.0 * 2.338_107_410_459_767 * mu.powf(-1.0 / 3.0)
        - 2.0 / 3.0 * mu.ln() / mu;
    let mut o = rigido(Method::Rosenbrock, 1e-6, 1e-9);
    o.events
        .push(Event::new(|_, y| y[0]).direction(Direction::Falling));
    let s = solve_ivp(vdp(mu), &[0.0, 3500.0], &[2.0, 0.0], &mut o).unwrap();
    assert_eq!(s.status, OdeStatus::Completed);
    let cruz: Vec<f64> = s.events.iter().map(|e| e.t).collect();
    assert_eq!(cruz.len(), 2);
    println!(
        "ROS VdP: período {} (assíntota {periodo}), {:?}",
        cruz[1] - cruz[0],
        s.stats
    );
    assert!(
        (cruz[1] - cruz[0] - periodo).abs() <= 0.05,
        "{}",
        cruz[1] - cruz[0]
    );
    assert!(s.stats.steps < 20000, "{:?}", s.stats);
    // Linear rígido (autovalores −1, −1e3, −1e6), Jacobiano exato dado: L-estável, sem
    // instabilidade com h·λ ~ −1e6.
    let a = [[-1.0, 0.0, 0.0], [1.0, -1e3, 0.0], [0.0, 1e3, -1e6]];
    let f = move |_: f64, y: &[f64], d: &mut [f64]| {
        for i in 0..3 {
            d[i] = (0..3).map(|j| a[i][j] * y[j]).sum();
        }
    };
    let mut erros = Vec::new();
    for tol in [1e-3, 1e-4, 1e-5] {
        for dado in [false, true] {
            let mut o = rigido(Method::Rosenbrock, tol, tol * 1e-3);
            if dado {
                o.jacobian = Some(Box::new(move |_, _, j| {
                    for (r, linha) in a.iter().enumerate() {
                        for (c, v) in linha.iter().enumerate() {
                            j.set(r, c, *v);
                        }
                    }
                }));
            }
            let s = solve_ivp(f, &[0.0, 0.01, 1.0, 5.0], &[1.0, 1.0, 1.0], &mut o).unwrap();
            // Exata do triangular inferior: y₁ = e^{−t}; y₂ e y₃ por variação de constantes.
            let ex = |t: f64| {
                let y1 = (-t).exp();
                let c2 = 1.0 / (1e3 - 1.0);
                let y2 = c2 * (-t).exp() + (1.0 - c2) * (-1e3 * t).exp();
                // y₃' = 1e3·y₂ − 1e6·y₃.
                let a1 = 1e3 * c2 / (1e6 - 1.0);
                let a2 = 1e3 * (1.0 - c2) / (1e6 - 1e3);
                let y3 = a1 * (-t).exp() + a2 * (-1e3 * t).exp() + (1.0 - a1 - a2) * (-1e6 * t).exp();
                [y1, y2, y3]
            };
            let e =
                s.t.iter()
                    .zip(&s.y)
                    .map(|(t, y)| (0..3).map(|i| (y[i] - ex(*t)[i]).abs()).fold(0.0, f64::max))
                    .fold(0.0, f64::max);
            println!("ROS linear tol {tol} dado {dado}: {e:.2e} {:?}", s.stats);
            // Medido: 1,1·tol, 2,3·tol e 5,2·tol (erro global acumulado de um método de ordem 2).
            assert!(e <= 10.0 * tol, "tol {tol} dado {dado}: {e:e}");
            if dado {
                erros.push(e);
            }
        }
    }
    assert!(erros.windows(2).all(|w| w[1] < w[0]), "{erros:?}");
}

/// Rosenbrock: casos de borda da verificação de linearização.
#[test]
fn rosenbrock_bordas_da_linearizacao() {
    // y grande com incremento abaixo do ulp: não é falha (y' = 1, y₀ = 1e20).
    let s = solve_ivp(
        |_, _, d| d[0] = 1.0,
        &[0.0, 1.0],
        &[1e20],
        &mut rigido(Method::Rosenbrock, 1e-6, 1e-9),
    )
    .unwrap();
    assert_eq!((s.status, fim(&s).1[0]), (OdeStatus::Completed, 1e20));
    // Jacobiano errado por fator 2 (não absurdo): o passo encolhe, mas a solução sai certa ou a
    // falha é declarada; nunca um resultado errado com Completed.
    let mut o = rigido(Method::Rosenbrock, 1e-6, 1e-9);
    o.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, -2.0)));
    let s = solve_ivp(|_, y, d| d[0] = -y[0], &[0.0, 1.0], &[1.0], &mut o).unwrap();
    if s.status == OdeStatus::Completed {
        assert!((fim(&s).1[0] - (-1f64).exp()).abs() <= 1e-3, "{:?}", fim(&s));
    }
    // Jacobiano absurdo começando fora de t = 0.
    let mut o = rigido(Method::Rosenbrock, 1e-6, 1e-9);
    o.jacobian = Some(Box::new(|_, _, j| j.set(0, 0, 1e308)));
    let s = solve_ivp(|_, y, d| d[0] = -y[0], &[5.0, 6.0], &[1.0], &mut o).unwrap();
    assert_eq!(s.status, OdeStatus::StepTooSmall);
    // Equilíbrio exato: f = 0 sempre, nenhum passo recusado.
    let s = solve_ivp(
        |_, y, d| d[0] = -1e6 * y[0],
        &[0.0, 1.0],
        &[0.0],
        &mut rigido(Method::Rosenbrock, 1e-6, 1e-9),
    )
    .unwrap();
    assert_eq!(
        (s.status, s.stats.rejected, fim(&s).1[0]),
        (OdeStatus::Completed, 0, 0.0)
    );
}

/// Detecção de rigidez no DOPRI5: Van der Pol μ = 1000 e Robertson avisam; oscilador e
/// Kepler não. O resultado continua correto (o aviso é só diagnóstico).
#[test]
fn deteccao_de_rigidez_no_dopri5() {
    let rigido_detectado = |s: &OdeSolution| {
        s.warnings.iter().find_map(|w| match w {
            OdeWarning::StiffnessSuspected { t } => Some(*t),
            _ => None,
        })
    };
    let s = solve_ivp(
        vdp(1000.0),
        &[0.0, 1.0],
        &[2.0, 0.0],
        &mut opts(Method::Dopri5, 1e-6, 1e-9),
    )
    .unwrap();
    let t = rigido_detectado(&s).expect("VdP μ = 1000 é rígido");
    assert!(t > 0.0 && t <= 1.0);
    // São precisos 15 passos seguidos acima do limite: o aviso não sai antes do 15º passo.
    assert!(t >= s.t[15], "aviso em {t}, antes do 15º passo ({})", s.t[15]);
    let s = solve_ivp(
        robertson,
        &[0.0, 1.0],
        &[1.0, 0.0, 0.0],
        &mut opts(Method::Dopri5, 1e-6, 1e-10),
    )
    .unwrap();
    assert!(rigido_detectado(&s).is_some());
    // Não rígidos: oscilador harmônico em 50 unidades e decaimento lento.
    let osc = |_: f64, y: &[f64], d: &mut [f64]| {
        d[0] = y[1];
        d[1] = -y[0];
    };
    let s = solve_ivp(
        osc,
        &[0.0, 50.0],
        &[1.0, 0.0],
        &mut opts(Method::Dopri5, 1e-8, 1e-10),
    )
    .unwrap();
    assert_eq!(rigido_detectado(&s), None);
    let s = solve_ivp(
        |_, y, d| d[0] = -0.5 * y[0],
        &[0.0, 20.0],
        &[1.0],
        &mut opts(Method::Dopri5, 1e-6, 1e-9),
    )
    .unwrap();
    assert_eq!(rigido_detectado(&s), None);
    // BS23 não tem o teste (estágios em pontos distintos).
    let s = solve_ivp(
        vdp(1000.0),
        &[0.0, 0.5],
        &[2.0, 0.0],
        &mut opts(Method::Bs23, 1e-4, 1e-7),
    )
    .unwrap();
    assert_eq!(rigido_detectado(&s), None);
    // Métodos implícitos nunca avisam (o padrão do `Stepper` é não detectar).
    for m in [Method::Radau, Method::Bdf, Method::Ndf, Method::Rosenbrock] {
        let s = solve_ivp(vdp(1000.0), &[0.0, 1.0], &[2.0, 0.0], &mut opts(m, 1e-6, 1e-9)).unwrap();
        assert_eq!(rigido_detectado(&s), None, "{m:?}");
        assert!(s.warnings.is_empty(), "{m:?}: {:?}", s.warnings);
    }
}

/// Rosenbrock: contabilidade e custo (mutação). Problema rígido não autônomo com termo
/// forçante polinomial (sem libm): y₁′ = −50(y₁ − g) + g′, g = t³/3 − t (solução exata y₁ = g),
/// acoplado a y₂′ = −y₂/2 + y₁ e y₃′ = y₃(1 − y₃). Com Jacobiano por diferenças (n = 3), cada
/// passo aceito custa 6 avaliações de f (3 do Jacobiano, 1 de ∂f/∂t, 2 estágios), cada rejeição
/// 2 (estágios com o mesmo Jacobiano), mais 2 no início; uma fatoração por tentativa; um
/// Jacobiano por passo aceito. O número de passos fica numa faixa de ±10% do medido (131 e 665),
/// o que pega mudanças na norma do erro sem depender do último ulp de `powf`.
#[test]
fn rosenbrock_contabilidade_e_custo() {
    for (tol, passos, fator) in [(1e-4, 131.0, 20.0), (1e-6, 665.0, 80.0)] {
        let mut o = OdeOptions {
            method: Method::Rosenbrock,
            rel_tol: Some(tol),
            abs_tol: Some(AbsTol::Scalar(tol)),
            ..Default::default()
        };
        let s = solve_ivp(
            |t, y, d| {
                let (g, gp) = (t * t * t / 3.0 - t, t * t - 1.0);
                d[0] = -50.0 * (y[0] - g) + gp;
                d[1] = -0.5 * y[1] + y[0];
                d[2] = y[2] * (1.0 - y[2]);
            },
            &[0.0, 4.0],
            &[0.0, 1.0, 0.1],
            &mut o,
        )
        .unwrap();
        let st = s.stats;
        assert_eq!(
            st.fun_evals,
            2 + 6 * st.steps + 2 * st.rejected,
            "tol {tol}: {st:?}"
        );
        assert_eq!(st.lu_decomps, st.steps + st.rejected, "tol {tol}: {st:?}");
        assert_eq!(st.jac_evals, st.steps, "tol {tol}: {st:?}");
        let p = st.steps as f64;
        assert!(p >= 0.9 * passos && p <= 1.1 * passos, "tol {tol}: {p} passos");
        let e =
            s.t.iter()
                .zip(&s.y)
                .map(|(t, y)| (y[0] - (t * t * t / 3.0 - t)).abs())
                .fold(0.0, f64::max);
        assert!(e <= fator * tol, "tol {tol}: erro {e}");
    }
}

/// Matriz de massa de tamanho errado: o erro informa os números de elementos (mutação).
#[test]
fn massa_de_tamanho_errado_informa_os_tamanhos() {
    use chimalab_math::linalg::Matrix;
    use chimalab_math::ode::OdeError;
    let mut o = OdeOptions {
        method: Method::Radau,
        mass: Some(Matrix::zeros(2, 3)),
        ..Default::default()
    };
    let r = solve_ivp(|_, y, d| d.copy_from_slice(y), &[0.0, 1.0], &[1.0, 1.0], &mut o);
    match r {
        Err(OdeError::ShapeMismatch { expected, actual }) => assert_eq!((expected, actual), (4, 6)),
        o => panic!("{:?}", o.map(|s| s.status)),
    }
}
