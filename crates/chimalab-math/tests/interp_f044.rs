//! F044 — interpolação 1-D: aceitação, oráculo, propriedades, bordas e erros.
//!
//! Oráculo: `tests/dados/F044/splines.csv`, de `ferramentas/oraculos/f044_splines.py`. São splines
//! em 40 dígitos pela formulação nos momentos com eliminação densa, independente do sistema
//! tridiagonal nas derivadas usado no código.

use chimalab_math::interp::{
    Extrap, GridInterpolant, InterpError, Interpolant1, Method1, MethodN, PPoly, ScatteredInterpolant,
    ScatteredMethod, SplineEnd, interp1,
};
use chimalab_math::random::{Algorithm, Rng};
use proptest::prelude::*;

const TODOS: [Method1; 9] = [
    Method1::Linear,
    Method1::Nearest,
    Method1::Previous,
    Method1::Next,
    Method1::Pchip,
    Method1::Akima,
    Method1::Makima,
    Method1::Spline(SplineEnd::NotAKnot),
    Method1::Spline(SplineEnd::Natural),
];

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1.0)
}

/// Cúbica a + bx + cx² + dx³ e sua derivada.
fn cubica(c: [f64; 4]) -> (impl Fn(f64) -> f64, impl Fn(f64) -> f64) {
    (
        move |x: f64| c[0] + x * (c[1] + x * (c[2] + x * c[3])),
        move |x: f64| c[1] + x * (2.0 * c[2] + 3.0 * x * c[3]),
    )
}

#[test]
fn ca1_spline_reproduz_polinomio_cubico() {
    let xs = [-1.3, -0.2, 0.4, 1.0, 2.5, 2.6, 4.0];
    for coef in [
        [1.0, -2.0, 0.5, 0.25],
        [0.0, 0.0, 0.0, -3.0],
        [2.0, 1.0, -1.0, 0.0],
    ] {
        let (p, dp) = cubica(coef);
        let ys: Vec<f64> = xs.iter().map(|&x| p(x)).collect();
        let fins = [SplineEnd::NotAKnot, SplineEnd::Clamped(dp(xs[0]), dp(xs[6]))];
        for end in fins {
            let it = Interpolant1::new(&xs, &ys, Method1::Spline(end)).unwrap();
            for k in 0..=200 {
                let x = -1.3 + 5.3 * k as f64 / 200.0;
                assert!(rel(it.eval(x, Extrap::Nan), p(x)) <= 1e-13, "{end:?} {x}");
            }
            // Derivada e integral exatas pelo PPoly.
            let pp = it.ppoly().unwrap();
            let d = pp.derivative();
            assert!(rel(d.eval(0.7, Extrap::Nan), dp(0.7)) <= 1e-12);
            let prim = |x: f64| x * (coef[0] + x * (coef[1] / 2.0 + x * (coef[2] / 3.0 + x * coef[3] / 4.0)));
            assert!(rel(pp.integral(-1.0, 3.3), prim(3.3) - prim(-1.0)) <= 1e-13);
        }
    }
    // Com 3 pontos a not-a-knot é a parábola interpoladora; com 2, a reta.
    let it = Interpolant1::new(
        &[0.0, 1.0, 3.0],
        &[1.0, 2.0, 10.0],
        Method1::Spline(SplineEnd::NotAKnot),
    )
    .unwrap();
    // Parábola por (0,1), (1,2), (3,10): 1 + 0x + x².
    for x in [0.5, 2.0, 2.9] {
        assert!(rel(it.eval(x, Extrap::Nan), 1.0 + x * x) <= 1e-14);
    }
    let it = Interpolant1::new(&[0.0, 2.0], &[1.0, 5.0], Method1::Spline(SplineEnd::Natural)).unwrap();
    assert_eq!(it.eval(0.5, Extrap::Nan), 2.0);
}

fn oraculo() -> Vec<(String, String, f64, f64)> {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F044/splines.csv"
    ))
    .unwrap();
    csv.lines()
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            (
                c[0].to_string(),
                c[1].to_string(),
                c[2].parse().unwrap(),
                c[3].parse().unwrap(),
            )
        })
        .collect()
}

fn conjunto(nome: &str) -> (Vec<f64>, Vec<f64>) {
    match nome {
        "irregular" => (
            vec![0.0, 0.3, 1.1, 1.7, 2.0, 3.5, 4.2, 6.0],
            vec![1.0, -0.4, 2.2, 2.1, 0.5, -1.3, 0.0, 3.0],
        ),
        "quatro" => (vec![-2.0, -1.0, 0.5, 3.0], vec![4.0, 1.0, -0.25, 9.0]),
        "periodico" => (
            vec![0.0, 0.7, 1.5, 2.9, 3.6, 5.0],
            vec![1.0, 0.2, -0.8, 0.3, 1.4, 1.0],
        ),
        "cinco" => (vec![1.0, 2.0, 4.0, 5.0, 8.0], vec![0.0, 1.0, -1.0, 2.0, 0.5]),
        _ => panic!("{nome}"),
    }
}

#[test]
fn splines_contra_oraculo() {
    let dados = oraculo();
    assert_eq!(dados.len(), 380);
    for (nome, cond, x, v) in &dados {
        let (xs, ys) = conjunto(nome);
        let end = match cond.as_str() {
            "notaknot" => SplineEnd::NotAKnot,
            "natural" => SplineEnd::Natural,
            "clamped" => SplineEnd::Clamped(0.5, -1.25),
            "periodic" => SplineEnd::Periodic,
            _ => panic!(),
        };
        let it = Interpolant1::new(&xs, &ys, Method1::Spline(end)).unwrap();
        // Arredondamento: alguns ulps vezes o número de condição do sistema (pequeno).
        assert!(rel(it.eval(*x, Extrap::Nan), *v) <= 1e-14, "{nome} {cond} {x}");
    }
}

#[test]
fn spline_continuidade_c2_e_periodica() {
    for (nome, end) in [
        ("irregular", SplineEnd::NotAKnot),
        ("irregular", SplineEnd::Natural),
        ("cinco", SplineEnd::Clamped(1.0, -2.0)),
        ("periodico", SplineEnd::Periodic),
    ] {
        let (xs, ys) = conjunto(nome);
        let pp = Interpolant1::new(&xs, &ys, Method1::Spline(end))
            .unwrap()
            .ppoly()
            .unwrap()
            .clone();
        let (d1, d2, d3) = (
            pp.derivative(),
            pp.derivative().derivative(),
            pp.derivative().derivative().derivative(),
        );
        // Valor nos dois lados de cada quebra interna: avaliando o polinômio da esquerda no
        // extremo direito dele (via extrapolação de um PPoly de um só intervalo).
        let lado = |p: &PPoly, i: usize, x: f64| {
            let one = PPoly::new(vec![p.breaks()[i], p.breaks()[i + 1]], vec![p.coefs()[i].clone()]).unwrap();
            one.eval(x, Extrap::Extrapolate)
        };
        for (i, &x) in xs.iter().enumerate().take(xs.len() - 1).skip(1) {
            for (k, p) in [&pp, &d1, &d2].into_iter().enumerate() {
                let (e, dd) = (lado(p, i - 1, x), lado(p, i, x));
                assert!(
                    rel(e, dd) <= 1e-12,
                    "{nome} {end:?} derivada {k} em {x}: {e} vs {dd}"
                );
            }
        }
        match end {
            SplineEnd::Natural => {
                assert!(d2.eval(xs[0], Extrap::Nan).abs() <= 1e-12);
                assert!(d2.eval(*xs.last().unwrap(), Extrap::Nan).abs() <= 1e-12);
            }
            SplineEnd::Clamped(a, b) => {
                assert!(rel(d1.eval(xs[0], Extrap::Nan), a) <= 1e-13);
                assert!(rel(d1.eval(*xs.last().unwrap(), Extrap::Nan), b) <= 1e-13);
            }
            SplineEnd::NotAKnot => {
                for i in [1, xs.len() - 2] {
                    assert!(rel(lado(&d3, i - 1, xs[i]), lado(&d3, i, xs[i])) <= 1e-11);
                }
            }
            SplineEnd::Periodic => {
                let l = *xs.last().unwrap();
                for p in [&d1, &d2] {
                    assert!(rel(p.eval(xs[0], Extrap::Nan), p.eval(l, Extrap::Nan)) <= 1e-12);
                }
            }
        }
    }
}

#[test]
fn metodos_locais_semantica() {
    let x = [0.0, 1.0, 3.0];
    let y = [10.0, 20.0, 40.0];
    let f = |m, q: f64| interp1(&x, &y, &[q], m, Extrap::Nan).unwrap()[0];
    // Nos nós, todos devolvem o próprio valor.
    for m in TODOS {
        for (xi, yi) in x.iter().zip(&y) {
            assert_eq!(f(m, *xi), *yi, "{m:?}");
        }
    }
    assert_eq!(f(Method1::Linear, 2.0), 30.0);
    assert_eq!(f(Method1::Previous, 2.9), 20.0);
    assert_eq!(f(Method1::Next, 1.1), 40.0);
    // Vizinho: o mais próximo; no ponto médio, o da direita.
    assert_eq!(f(Method1::Nearest, 1.9), 20.0);
    assert_eq!(f(Method1::Nearest, 2.0), 40.0);
    assert_eq!(f(Method1::Nearest, 0.5), 20.0);
    // Um único ponto basta para os degraus.
    assert_eq!(
        interp1(&[2.0], &[7.0], &[2.0], Method1::Nearest, Extrap::Nan).unwrap(),
        vec![7.0]
    );
    assert_eq!(
        interp1(&[2.0], &[7.0], &[5.0], Method1::Previous, Extrap::Extrapolate).unwrap(),
        vec![7.0]
    );
    assert_eq!(
        interp1(&[2.0], &[7.0], &[-5.0], Method1::Next, Extrap::Extrapolate).unwrap(),
        vec![7.0]
    );
}

#[test]
fn extrapolacao_uniforme() {
    let x = [0.0, 1.0, 2.0, 4.0];
    let y = [0.0, 1.0, 4.0, 16.0];
    for m in TODOS {
        let it = Interpolant1::new(&x, &y, m).unwrap();
        assert!(it.eval(-1.0, Extrap::Nan).is_nan(), "{m:?}");
        assert!(it.eval(4.5, Extrap::Nan).is_nan(), "{m:?}");
        assert_eq!(it.eval(5.0, Extrap::Value(-7.0)), -7.0);
        assert_eq!(it.eval(-3.0, Extrap::Clamp), 0.0, "{m:?}");
        assert_eq!(it.eval(9.0, Extrap::Clamp), 16.0, "{m:?}");
        assert!(it.eval(f64::NAN, Extrap::Extrapolate).is_nan());
        // Os extremos pertencem ao intervalo (não são extrapolação).
        assert_eq!(it.eval(4.0, Extrap::Nan), 16.0, "{m:?}");
    }
    let lin = Interpolant1::new(&x, &y, Method1::Linear).unwrap();
    assert!(lin.ppoly().is_none());
    assert_eq!(lin.eval(-1.0, Extrap::Extrapolate), -1.0);
    assert_eq!(lin.eval(5.0, Extrap::Extrapolate), 22.0);
    let prev = Interpolant1::new(&x, &y, Method1::Previous).unwrap();
    assert_eq!(prev.eval(-1.0, Extrap::Extrapolate), 0.0);
    assert_eq!(prev.eval(9.0, Extrap::Extrapolate), 16.0);
    let next = Interpolant1::new(&x, &y, Method1::Next).unwrap();
    assert_eq!(next.eval(9.0, Extrap::Extrapolate), 16.0);
    let near = Interpolant1::new(&x, &y, Method1::Nearest).unwrap();
    assert_eq!(
        (
            near.eval(-9.0, Extrap::Extrapolate),
            near.eval(9.0, Extrap::Extrapolate)
        ),
        (0.0, 16.0)
    );
    // Cúbicos: prolongam o polinômio extremo (a spline reproduz x² exatamente).
    let sp = Interpolant1::new(&x, &y, Method1::Spline(SplineEnd::NotAKnot)).unwrap();
    assert!(rel(sp.eval(6.0, Extrap::Extrapolate), 36.0) <= 1e-13);
    assert!(rel(sp.eval(-2.0, Extrap::Extrapolate), 4.0) <= 1e-13);
}

#[test]
fn pchip_forma_e_continuidade() {
    // Degrau suave: o pchip não ultrapassa; a spline ultrapassa.
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let p = Interpolant1::new(&x, &y, Method1::Pchip).unwrap();
    let s = Interpolant1::new(&x, &y, Method1::Spline(SplineEnd::NotAKnot)).unwrap();
    let mut s_ultrapassa = false;
    let mut anterior = -1.0;
    for k in 0..=500 {
        let t = 5.0 * k as f64 / 500.0;
        let v = p.eval(t, Extrap::Nan);
        assert!((0.0..=1.0).contains(&v), "{t}: {v}");
        assert!(v >= anterior);
        anterior = v;
        let w = s.eval(t, Extrap::Nan);
        s_ultrapassa |= !(-1e-12..=1.0 + 1e-12).contains(&w);
    }
    assert!(s_ultrapassa);
    // Extremo local nos dados: derivada nula ali.
    let p = Interpolant1::new(&[0.0, 1.0, 2.0], &[0.0, 1.0, 0.0], Method1::Pchip).unwrap();
    assert_eq!(p.ppoly().unwrap().derivative().eval(1.0, Extrap::Nan), 0.0);
    // C¹ nas quebras.
    let (xs, ys) = conjunto("irregular");
    for m in [Method1::Pchip, Method1::Akima, Method1::Makima] {
        let pp = Interpolant1::new(&xs, &ys, m).unwrap().ppoly().unwrap().clone();
        let d = pp.derivative();
        for i in 1..xs.len() - 1 {
            let esquerda = PPoly::new(vec![xs[i - 1], xs[i]], vec![d.coefs()[i - 1].clone()])
                .unwrap()
                .eval(xs[i], Extrap::Nan);
            assert!(rel(esquerda, d.coefs()[i][0]) <= 1e-12, "{m:?} em {}", xs[i]);
        }
    }
    // Extremo: derivada limitada a 3δ quando as secantes trocam de sinal (Fritsch–Carlson).
    let p = Interpolant1::new(&[0.0, 1.0, 1.1], &[0.0, 1.0, -5.0], Method1::Pchip).unwrap();
    assert_eq!(p.ppoly().unwrap().derivative().eval(0.0, Extrap::Nan), 3.0);
    // Extremo com sinal contrário ao da primeira secante: derivada nula.
    let p = Interpolant1::new(&[0.0, 1.0, 1.5], &[0.0, 0.1, 5.0], Method1::Pchip).unwrap();
    assert_eq!(p.ppoly().unwrap().derivative().eval(0.0, Extrap::Nan), 0.0);
}

#[test]
fn akima_e_makima() {
    // Platô: o makima fica exatamente plano onde há três ou mais valores iguais; o Akima original
    // não garante isso quando as secantes vizinhas se anulam de um lado só.
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let y = [0.0, 1.0, 1.0, 1.0, 1.0, 3.0, 2.0];
    let m = Interpolant1::new(&x, &y, Method1::Makima).unwrap();
    for k in 0..=100 {
        let t = 1.0 + 3.0 * k as f64 / 100.0;
        assert_eq!(m.eval(t, Extrap::Nan), 1.0, "{t}");
    }
    // Ambos reproduzem retas exatamente (todas as secantes iguais).
    for meth in [Method1::Akima, Method1::Makima, Method1::Pchip] {
        let it = Interpolant1::new(&x, &x.map(|v| 2.0 * v - 1.0), meth).unwrap();
        for k in 0..=60 {
            let t = 0.1 * k as f64;
            assert!(
                rel(it.eval(t, Extrap::Nan), 2.0 * t - 1.0) <= 1e-14,
                "{meth:?} {t}"
            );
        }
    }
    // Pesos nulos dos dois lados (secantes alternando ±1 com |δ| constante no Akima original):
    // média simples.
    let it = Interpolant1::new(
        &[0.0, 1.0, 2.0, 3.0, 4.0],
        &[0.0, 1.0, 0.0, 1.0, 0.0],
        Method1::Akima,
    )
    .unwrap();
    assert!(it.eval(2.0, Extrap::Nan).abs() <= 1e-15);
    // Com 2 pontos: a reta.
    for meth in [Method1::Akima, Method1::Makima, Method1::Pchip] {
        let it = Interpolant1::new(&[1.0, 3.0], &[2.0, 6.0], meth).unwrap();
        assert_eq!(it.eval(2.0, Extrap::Nan), 4.0);
    }
}

#[test]
fn ordenacao_decrescente_nan_e_escalas() {
    // Fora de ordem e decrescente: ordenado junto com y.
    let a = interp1(
        &[3.0, 1.0, 2.0],
        &[9.0, 1.0, 4.0],
        &[1.5, 2.5],
        Method1::Linear,
        Extrap::Nan,
    )
    .unwrap();
    assert_eq!(a, vec![2.5, 6.5]);
    let it = Interpolant1::new(&[3.0, 2.0, 1.0], &[9.0, 4.0, 1.0], Method1::Pchip).unwrap();
    assert_eq!(it.x(), &[1.0, 2.0, 3.0]);
    assert_eq!(it.y(), &[1.0, 4.0, 9.0]);
    // NaN em y: afeta só os intervalos vizinhos nos métodos locais.
    let it = Interpolant1::new(&[0.0, 1.0, 2.0, 3.0], &[0.0, f64::NAN, 2.0, 3.0], Method1::Linear).unwrap();
    assert!(it.eval(0.5, Extrap::Nan).is_nan());
    assert_eq!(it.eval(2.5, Extrap::Nan), 2.5);
    // Escalas extremas: a interpolação é invariante por escala (potência de 2, exata).
    let x = [0.0, 1.0, 2.5, 4.0];
    let y = [1.0, -2.0, 0.5, 3.0];
    for e in [-1000, -300, 300, 1000] {
        let sx = 2f64.powi(e.clamp(-1000, 1000) / 4);
        let sy = 2f64.powi(e);
        let xs: Vec<f64> = x.iter().map(|v| v * sx).collect();
        let ys: Vec<f64> = y.iter().map(|v| v * sy).collect();
        for m in TODOS {
            let a = Interpolant1::new(&x, &y, m).unwrap().eval(1.7, Extrap::Nan);
            let b = Interpolant1::new(&xs, &ys, m)
                .unwrap()
                .eval(1.7 * sx, Extrap::Nan);
            assert!(rel(b / sy, a) <= 1e-14, "{m:?} escala {e}: {b} vs {a}");
        }
    }
    // Subnormais em y.
    let it = Interpolant1::new(&[0.0, 1.0], &[0.0, 4e-320], Method1::Linear).unwrap();
    assert_eq!(it.eval(0.5, Extrap::Nan), 2e-320);
}

#[test]
fn erros_com_identificador() {
    let casos: Vec<(InterpError, &str)> = vec![
        (
            Interpolant1::new(&[0.0, 1.0], &[1.0], Method1::Linear).unwrap_err(),
            "chimalab:interp:formaIncompativel",
        ),
        (
            Interpolant1::new(&[0.0], &[1.0], Method1::Linear).unwrap_err(),
            "chimalab:interp:pontosInsuficientes",
        ),
        (
            Interpolant1::new(&[], &[], Method1::Nearest).unwrap_err(),
            "chimalab:interp:pontosInsuficientes",
        ),
        (
            Interpolant1::new(&[0.0, 1.0, 1.0], &[1.0, 2.0, 3.0], Method1::Linear).unwrap_err(),
            "chimalab:interp:pontosDuplicados",
        ),
        (
            Interpolant1::new(&[0.0, f64::NAN], &[1.0, 2.0], Method1::Linear).unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
        (
            Interpolant1::new(&[0.0, f64::INFINITY], &[1.0, 2.0], Method1::Pchip).unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
        (
            Interpolant1::new(
                &[0.0, 1.0, 2.0],
                &[1.0, 2.0, 3.0],
                Method1::Spline(SplineEnd::Periodic),
            )
            .unwrap_err(),
            "chimalab:interp:opcaoInvalida",
        ),
        (
            Interpolant1::new(
                &[0.0, 1.0],
                &[1.0, 2.0],
                Method1::Spline(SplineEnd::Clamped(f64::NAN, 0.0)),
            )
            .unwrap_err(),
            "chimalab:interp:opcaoInvalida",
        ),
        (
            PPoly::new(vec![0.0], vec![]).unwrap_err(),
            "chimalab:interp:pontosInsuficientes",
        ),
        (
            PPoly::new(vec![0.0, 1.0], vec![]).unwrap_err(),
            "chimalab:interp:formaIncompativel",
        ),
        (
            PPoly::new(vec![0.0, 1.0, 2.0], vec![vec![1.0], vec![1.0, 2.0]]).unwrap_err(),
            "chimalab:interp:formaIncompativel",
        ),
        (
            PPoly::new(vec![0.0, 1.0], vec![vec![]]).unwrap_err(),
            "chimalab:interp:formaIncompativel",
        ),
        (
            PPoly::new(vec![1.0, 0.0], vec![vec![1.0]]).unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
        (
            PPoly::new(vec![0.0, f64::NAN], vec![vec![1.0]]).unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn spline_periodica_pequena_e_ppoly() {
    // Periódica com 2 e 3 pontos (sistemas 1×1 e 2×2).
    let it = Interpolant1::new(&[0.0, 1.0], &[2.0, 2.0], Method1::Spline(SplineEnd::Periodic)).unwrap();
    assert_eq!(it.eval(0.3, Extrap::Nan), 2.0);
    let it = Interpolant1::new(
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0, 0.0],
        Method1::Spline(SplineEnd::Periodic),
    )
    .unwrap();
    let d = it.ppoly().unwrap().derivative();
    assert!(rel(d.eval(0.0, Extrap::Nan), d.eval(2.0, Extrap::Nan)) <= 1e-14);
    let it = Interpolant1::new(
        &[0.0, 1.0, 2.0, 3.0],
        &[0.0, 1.0, -1.0, 0.0],
        Method1::Spline(SplineEnd::Periodic),
    )
    .unwrap();
    let d2 = it.ppoly().unwrap().derivative().derivative();
    assert!(rel(d2.eval(0.0, Extrap::Nan), d2.eval(3.0, Extrap::Nan)) <= 1e-13);
    // PPoly: grau, derivada de constante, primitiva contínua e integral.
    let p = PPoly::new(vec![0.0, 1.0, 3.0], vec![vec![1.0], vec![2.0]]).unwrap();
    assert_eq!(p.degree(), 0);
    assert_eq!(p.derivative().eval(0.5, Extrap::Nan), 0.0);
    assert_eq!(p.integral(0.0, 3.0), 5.0);
    assert_eq!(p.antiderivative().eval(1.0, Extrap::Nan), 1.0);
    assert_eq!(p.eval_many(&[0.5, 2.0, 4.0], Extrap::Nan)[..2], [1.0, 2.0]);
    assert_eq!(p.eval(3.5, Extrap::Clamp), 2.0);
    assert_eq!(p.eval(-1.0, Extrap::Value(9.0)), 9.0);
    assert!(p.eval(f64::NAN, Extrap::Clamp).is_nan());
    // Spline natural com derivadas dadas e 2 pontos: a Hermite cúbica.
    let it = Interpolant1::new(
        &[0.0, 1.0],
        &[0.0, 0.0],
        Method1::Spline(SplineEnd::Clamped(1.0, 1.0)),
    )
    .unwrap();
    // y = x − 3x² + 2x³ tem y(0)=y(1)=0 e y'(0)=y'(1)=1.
    assert!((it.eval(0.5, Extrap::Nan) - (0.5 - 0.75 + 0.25)).abs() <= 1e-15);
}

/// Tridiagonal com pivotamento: espaçamento muito irregular faz as linhas not-a-knot perderem
/// a dominância diagonal; o pivotamento mantém a precisão (contra a reprodução exata do cúbico).
#[test]
fn spline_espacamento_irregular() {
    let xs = [0.0, 1e-6, 1.0, 1.0 + 1e-6, 50.0, 1000.0];
    let (p, _) = cubica([0.3, -1.0, 0.02, 1e-4]);
    let ys: Vec<f64> = xs.iter().map(|&x| p(x)).collect();
    let it = Interpolant1::new(&xs, &ys, Method1::Spline(SplineEnd::NotAKnot)).unwrap();
    for x in [0.5, 1.0 + 5e-7, 20.0, 600.0] {
        assert!(
            rel(it.eval(x, Extrap::Nan), p(x)) <= 1e-9,
            "{x}: {} vs {}",
            it.eval(x, Extrap::Nan),
            p(x)
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Nos nós, todo método devolve o valor dado; a ordem de entrada não importa.
    #[test]
    fn nos_exatos_e_ordem(pts in prop::collection::btree_map(-1000i32..1000, -100.0f64..100.0, 4..20)) {
        let x: Vec<f64> = pts.keys().map(|&k| k as f64 / 10.0).collect();
        let y: Vec<f64> = pts.values().copied().collect();
        let mut xr = x.clone();
        let mut yr = y.clone();
        xr.reverse();
        yr.reverse();
        for m in TODOS {
            let a = Interpolant1::new(&x, &y, m).unwrap();
            let b = Interpolant1::new(&xr, &yr, m).unwrap();
            for (xi, yi) in x.iter().zip(&y) {
                prop_assert!(rel(a.eval(*xi, Extrap::Nan), *yi) <= 1e-12);
            }
            prop_assert_eq!(&a, &b);
        }
    }

    /// pchip preserva monotonia de dados monótonos.
    #[test]
    fn pchip_monotono(incs in prop::collection::vec((0.01f64..5.0, 0.0f64..10.0), 2..15)) {
        let mut x = vec![0.0];
        let mut y = vec![0.0];
        for (dx, dy) in &incs {
            x.push(x.last().unwrap() + dx);
            y.push(y.last().unwrap() + dy);
        }
        let it = Interpolant1::new(&x, &y, Method1::Pchip).unwrap();
        let fim = *x.last().unwrap();
        let mut ant = f64::NEG_INFINITY;
        for k in 0..=400 {
            let v = it.eval((fim * k as f64 / 400.0).min(fim), Extrap::Nan);
            prop_assert!(v >= ant - 1e-12 * v.abs().max(1.0));
            ant = v;
        }
    }

    /// Invariância afim em y: interp(a·y + b) = a·interp(y) + b.
    #[test]
    fn afim_em_y(a in -10.0f64..10.0, b in -10.0f64..10.0, q in 0.0f64..6.0) {
        let (xs, ys) = conjunto("irregular");
        let ys2: Vec<f64> = ys.iter().map(|v| a * v + b).collect();
        for m in TODOS {
            let u = Interpolant1::new(&xs, &ys, m).unwrap().eval(q, Extrap::Nan);
            let v = Interpolant1::new(&xs, &ys2, m).unwrap().eval(q, Extrap::Nan);
            prop_assert!((v - (a * u + b)).abs() <= 1e-12 * (a.abs() * 10.0 + b.abs() + 1.0), "{:?}", m);
        }
    }
}

// ================================================================ grades N-D (incremento 2)

/// Valores column-major de f nos eixos dados.
fn grade(axes: &[Vec<f64>], f: impl Fn(&[f64]) -> f64) -> Vec<f64> {
    let total: usize = axes.iter().map(Vec::len).product();
    (0..total)
        .map(|mut k| {
            let p: Vec<f64> = axes
                .iter()
                .map(|a| {
                    let i = k % a.len();
                    k /= a.len();
                    a[i]
                })
                .collect();
            f(&p)
        })
        .collect()
}

#[test]
fn grade_multilinear_e_tensor_reproduzem_polinomios() {
    let axes = vec![
        vec![-1.0, 0.0, 0.5, 2.0, 3.0],
        vec![0.0, 1.0, 1.5, 4.0],
        vec![2.0, 2.5, 3.0, 5.0, 6.0],
    ];
    // Multilinear reproduz funções afins em cada variável (inclusive produtos xyz).
    let f = |p: &[f64]| 1.0 + 2.0 * p[0] - p[1] + 0.5 * p[2] + p[0] * p[1] * p[2] - 3.0 * p[0] * p[2];
    let g = GridInterpolant::new(axes.clone(), grade(&axes, f), MethodN::Linear).unwrap();
    // Produto tensorial de cúbicas: a spline not-a-knot reproduz cada fator.
    let c = |p: &[f64]| (p[0].powi(3) - p[0]) * (2.0 + p[1] * p[1]) * (p[2].powi(3) * 0.1 + 1.0);
    let t = GridInterpolant::new(
        axes.clone(),
        grade(&axes, c),
        MethodN::Tensor(Method1::Spline(SplineEnd::NotAKnot)),
    )
    .unwrap();
    assert_eq!(g.ndims(), 3);
    for k in 0..200 {
        let u = (k as f64 * 0.618_033_988_75).fract();
        let v = (k as f64 * 0.414_213_562_37).fract();
        let w = (k as f64 * 0.732_050_807_57).fract();
        let p = [-1.0 + 4.0 * u, 4.0 * v, 2.0 + 4.0 * w];
        // Dentro de cada célula, o multilinear é exato para funções multiafins.
        assert!(rel(g.eval(&p, Extrap::Nan), f(&p)) <= 1e-13, "{p:?}");
        assert!(rel(t.eval(&p, Extrap::Nan), c(&p)) <= 1e-12, "{p:?}");
    }
    // Extrapolação prolonga a célula ou o polinômio extremo (ainda exata).
    let p = [3.5, -0.5, 6.5];
    assert!(rel(g.eval(&p, Extrap::Extrapolate), f(&p)) <= 1e-13);
    assert!(rel(t.eval(&p, Extrap::Extrapolate), c(&p)) <= 1e-12);
}

#[test]
fn grade_1d_igual_ao_interp1_e_bordas() {
    let x = vec![0.0, 1.0, 2.5, 4.0, 4.5];
    let y = vec![1.0, -1.0, 2.0, 0.5, 3.0];
    for (mn, m1) in [
        (MethodN::Linear, Method1::Linear),
        (MethodN::Nearest, Method1::Nearest),
        (MethodN::Tensor(Method1::Pchip), Method1::Pchip),
        (MethodN::Tensor(Method1::Makima), Method1::Makima),
        (
            MethodN::Tensor(Method1::Spline(SplineEnd::Natural)),
            Method1::Spline(SplineEnd::Natural),
        ),
    ] {
        let g = GridInterpolant::new(vec![x.clone()], y.clone(), mn).unwrap();
        let it = Interpolant1::new(&x, &y, m1).unwrap();
        for k in 0..=90 {
            let q = 0.05 * k as f64;
            // Mesmo valor a menos de arredondamento: o multilinear usa (1 − w)y₀ + w·y₁.
            assert!(
                rel(g.eval(&[q], Extrap::Nan), it.eval(q, Extrap::Nan)) <= 4.0 * f64::EPSILON,
                "{mn:?} {q}"
            );
        }
    }
    // Nós: valor exato, e NaN num vértice não contamina os nós vizinhos.
    let axes = vec![vec![0.0, 1.0, 2.0], vec![0.0, 1.0]];
    let mut v = grade(&axes, |p| p[0] + 10.0 * p[1]);
    v[2] = f64::NAN; // vértice (2, 0)
    let g = GridInterpolant::new(axes.clone(), v, MethodN::Linear).unwrap();
    assert_eq!(g.eval(&[1.0, 0.0], Extrap::Nan), 1.0);
    assert_eq!(g.eval(&[1.0, 1.0], Extrap::Nan), 11.0);
    assert!(g.eval(&[1.5, 0.5], Extrap::Nan).is_nan());
    assert!(g.eval(&[f64::NAN, 0.5], Extrap::Clamp).is_nan());
    // Fora: NaN, valor, extremo.
    let g = GridInterpolant::new(
        axes.clone(),
        grade(&axes, |p| p[0] + 10.0 * p[1]),
        MethodN::Linear,
    )
    .unwrap();
    assert!(g.eval(&[3.0, 0.5], Extrap::Nan).is_nan());
    assert_eq!(g.eval(&[3.0, 0.5], Extrap::Value(-1.0)), -1.0);
    assert_eq!(g.eval(&[3.0, 2.0], Extrap::Clamp), 12.0);
    // Vizinho em 2-D: ponto médio vai para o índice maior.
    let n = GridInterpolant::new(
        axes.clone(),
        grade(&axes, |p| p[0] + 10.0 * p[1]),
        MethodN::Nearest,
    )
    .unwrap();
    assert_eq!(n.eval(&[0.5, 0.2], Extrap::Nan), 1.0);
    assert_eq!(n.eval(&[1.4, 0.7], Extrap::Nan), 11.0);
    assert_eq!(n.eval(&[-5.0, 9.0], Extrap::Extrapolate), 10.0);
}

#[test]
fn grade_erros() {
    let casos: Vec<(InterpError, &str)> = vec![
        (
            GridInterpolant::new(vec![], vec![], MethodN::Linear).unwrap_err(),
            "chimalab:interp:pontosInsuficientes",
        ),
        (
            GridInterpolant::new(vec![vec![0.0]], vec![1.0], MethodN::Linear).unwrap_err(),
            "chimalab:interp:pontosInsuficientes",
        ),
        (
            GridInterpolant::new(vec![vec![1.0, 0.0]], vec![1.0, 2.0], MethodN::Linear).unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
        (
            GridInterpolant::new(vec![vec![0.0, f64::INFINITY]], vec![1.0, 2.0], MethodN::Linear)
                .unwrap_err(),
            "chimalab:interp:naoMonotono",
        ),
        (
            GridInterpolant::new(
                vec![vec![0.0, 1.0], vec![0.0, 1.0]],
                vec![1.0, 2.0, 3.0],
                MethodN::Linear,
            )
            .unwrap_err(),
            "chimalab:interp:formaIncompativel",
        ),
        (
            GridInterpolant::new(
                vec![vec![0.0, 1.0]],
                vec![1.0, 2.0],
                MethodN::Tensor(Method1::Linear),
            )
            .unwrap_err(),
            "chimalab:interp:opcaoInvalida",
        ),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
    }
}

#[test]
#[should_panic(expected = "ponto com dimensão errada")]
fn grade_ponto_com_dimensao_errada() {
    let g = GridInterpolant::new(vec![vec![0.0, 1.0]], vec![1.0, 2.0], MethodN::Linear).unwrap();
    g.eval(&[0.5, 0.5], Extrap::Nan);
}

// ================================================================ dispersos (incremento 3)

#[test]
fn ca2_interpolante_disperso_com_delaunay() {
    let mut r = Rng::new(Algorithm::Philox, 44);
    let pts: Vec<[f64; 2]> = (0..3000)
        .map(|_| [r.uniform() * 4.0 - 2.0, r.uniform() * 3.0])
        .collect();
    // Linear por triângulo reproduz funções afins exatamente.
    let f = |p: [f64; 2]| 3.0 - 2.0 * p[0] + 0.5 * p[1];
    let vals: Vec<f64> = pts.iter().map(|&p| f(p)).collect();
    let si = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Linear).unwrap();
    let nn = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Nearest).unwrap();
    let mut dentro = 0;
    for _ in 0..2000 {
        let q = [r.uniform() * 4.4 - 2.2, r.uniform() * 3.4 - 0.2];
        let v = si.eval(q, Extrap::Nan);
        if v.is_nan() {
            continue;
        }
        dentro += 1;
        assert!(rel(v, f(q)) <= 1e-12, "{q:?}");
        // Vizinho mais próximo contra força bruta.
        let d2 = |p: [f64; 2]| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2);
        let bruto = (0..pts.len())
            .min_by(|&a, &b| d2(pts[a]).total_cmp(&d2(pts[b])).then(a.cmp(&b)))
            .unwrap();
        assert_eq!(nn.nearest_point(q), bruto);
        assert_eq!(nn.eval(q, Extrap::Nan), vals[bruto]);
    }
    assert!(dentro > 1500);
    // Função suave: erro de interpolação linear O(h²).
    let g = |p: [f64; 2]| (p[0]).sin() * (p[1]).cos();
    let gv: Vec<f64> = pts.iter().map(|&p| g(p)).collect();
    let gi = ScatteredInterpolant::new(&pts, &gv, ScatteredMethod::Linear).unwrap();
    let e = (0..400)
        .map(|k| {
            [
                -1.5 + 3.0 * ((k as f64) * 0.618).fract(),
                0.3 + 2.4 * ((k as f64) * 0.414).fract(),
            ]
        })
        .map(|q| (gi.eval(q, Extrap::Nan) - g(q)).abs())
        .fold(0.0, f64::max);
    assert!(e < 5e-3, "{e}");
}

#[test]
fn dispersos_bordas_e_erros() {
    let pts = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let vals = [0.0, 1.0, 2.0, 3.0, 5.0];
    let si = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Linear).unwrap();
    // Repetido: média (1 e 5 → 3) no vértice (1, 0).
    assert_eq!(si.eval([1.0, 0.0], Extrap::Nan), 3.0);
    // Fora do fecho: NaN, valor ou vizinho mais próximo.
    assert!(si.eval([2.0, 2.0], Extrap::Nan).is_nan());
    assert_eq!(si.eval([2.0, 2.0], Extrap::Value(-1.0)), -1.0);
    assert_eq!(si.eval([2.0, 2.0], Extrap::Extrapolate), 3.0);
    assert_eq!(si.eval([-1.0, -1.0], Extrap::Clamp), 0.0);
    assert!(si.eval([f64::NAN, 0.5], Extrap::Clamp).is_nan());
    assert_eq!(
        si.eval_many(&[[0.0, 0.0], [0.0, 1.0]], Extrap::Nan),
        vec![0.0, 2.0]
    );
    // NaN num vértice não contamina os nós vizinhos.
    let si =
        ScatteredInterpolant::new(&pts[..4], &[f64::NAN, 1.0, 2.0, 3.0], ScatteredMethod::Linear).unwrap();
    assert_eq!(si.eval([1.0, 1.0], Extrap::Nan), 3.0);
    let e = |r: Result<ScatteredInterpolant, InterpError>| r.unwrap_err().id();
    assert_eq!(
        e(ScatteredInterpolant::new(&pts, &[1.0], ScatteredMethod::Linear)),
        "chimalab:interp:formaIncompativel"
    );
    assert_eq!(
        e(ScatteredInterpolant::new(
            &pts[..2],
            &[1.0, 2.0],
            ScatteredMethod::Linear
        )),
        "chimalab:interp:pontosInsuficientes"
    );
    assert_eq!(
        e(ScatteredInterpolant::new(
            &[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]],
            &[1.0, 2.0, 3.0],
            ScatteredMethod::Nearest
        )),
        "chimalab:interp:opcaoInvalida"
    );
    assert_eq!(
        e(ScatteredInterpolant::new(
            &[[0.0, f64::NAN], [1.0, 1.0], [2.0, 0.0]],
            &[1.0, 2.0, 3.0],
            ScatteredMethod::Linear
        )),
        "chimalab:interp:naoMonotono"
    );
}

// ================================================================ incremento 4: vizinho natural

/// Sibson contra a definição: wᵢ = área(V'(q) ∩ V(i)) / área(V'(q)), com as células de Voronoi
/// recortadas (F047) e a interseção de polígonos (F047), caminho independente dos circuncentros.
#[test]
fn sibson_contra_definicao_por_voronoi() {
    use chimalab_math::geom::{BoolOp, Delaunay, polygon_boolean, voronoi_clipped};
    let mut r = Rng::new(Algorithm::Philox, 44);
    let pts: Vec<[f64; 2]> = (0..40).map(|_| [r.uniform(), r.uniform()]).collect();
    let vals: Vec<f64> = pts.iter().map(|p| (3.0 * p[0]).sin() + p[1] * p[1]).collect();
    let si = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Natural).unwrap();
    let caixa = [-5.0, -5.0, 6.0, 6.0];
    let antigas = voronoi_clipped(&Delaunay::new(&pts).unwrap(), caixa);
    let mut testados = 0;
    for _ in 0..30 {
        let q = [0.2 + 0.6 * r.uniform(), 0.2 + 0.6 * r.uniform()];
        let Some(w) = si.sibson_coordinates(q) else {
            continue;
        };
        let mut mais = pts.clone();
        mais.push(q);
        let novas = voronoi_clipped(&Delaunay::new(&mais).unwrap(), caixa);
        let celula_q = &novas.iter().find(|c| c.0 == pts.len()).unwrap().1;
        let total = celula_q.area();
        for (i, cel) in &antigas {
            let tomada = polygon_boolean(celula_q, cel, BoolOp::Intersection).area() / total;
            let nosso = w.iter().find(|p| p.0 == *i).map_or(0.0, |p| p.1);
            assert!(
                (tomada - nosso).abs() <= 1e-9,
                "q {q:?} i {i}: {tomada} vs {nosso}"
            );
        }
        testados += 1;
    }
    assert!(testados >= 25);
}

#[test]
fn natural_precisao_linear_nos_e_bordas() {
    let mut r = Rng::new(Algorithm::Philox, 45);
    let pts: Vec<[f64; 2]> = (0..300)
        .map(|_| [r.uniform() * 4.0 - 2.0, r.uniform() * 3.0])
        .collect();
    let afim = |p: [f64; 2]| 2.0 + 3.0 * p[0] - 0.5 * p[1];
    let vals: Vec<f64> = pts.iter().map(|&p| afim(p)).collect();
    let si = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Natural).unwrap();
    // Exata para funções afins (precisão linear das coordenadas de Sibson).
    let mut pior = 0.0f64;
    for _ in 0..500 {
        let q = [r.uniform() * 3.0 - 1.5, 0.3 + r.uniform() * 2.4];
        pior = pior.max((si.eval(q, Extrap::Nan) - afim(q)).abs());
    }
    assert!(pior <= 1e-11, "{pior}");
    // Interpola nos nós (exato) e é contínua perto deles.
    for (p, v) in pts.iter().zip(&vals).take(50) {
        assert_eq!(si.eval(*p, Extrap::Nan), *v);
    }
    // Quadrado: borda do fecho (linear ao longo da aresta), fora (extrapolação), centro cocircular.
    let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let sv = [0.0, 1.0, 3.0, 2.0];
    let s = ScatteredInterpolant::new(&sq, &sv, ScatteredMethod::Natural).unwrap();
    assert_eq!(s.eval([0.5, 0.0], Extrap::Nan), 0.5);
    assert!(s.sibson_coordinates([0.5, 0.0]).is_none());
    assert!(s.eval([2.0, 2.0], Extrap::Nan).is_nan());
    assert_eq!(s.eval([2.0, 2.0], Extrap::Clamp), 3.0);
    // No centro, os quatro vértices são cocirculares com q: pesos iguais, 1/4.
    let w = s.sibson_coordinates([0.5, 0.5]).unwrap();
    assert_eq!(w.len(), 4);
    assert!(w.iter().all(|p| (p.1 - 0.25).abs() <= 1e-15), "{w:?}");
    assert!((s.eval([0.5, 0.5], Extrap::Nan) - 1.5).abs() <= 1e-15);
    assert!(s.eval([f64::NAN, 0.5], Extrap::Clamp).is_nan());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Coordenadas locais: pesos ≥ 0, soma 1, Σ wᵢ·xᵢ = q; o valor fica entre o mínimo e o máximo
    /// dos vizinhos.
    #[test]
    fn sibson_coordenadas_locais(sem in any::<u64>(), n in 5usize..80, qx in 0.05f64..0.95, qy in 0.05f64..0.95) {
        let mut r = Rng::new(Algorithm::Philox, sem);
        let mut pts: Vec<[f64; 2]> = (0..n).map(|_| [r.uniform(), r.uniform()]).collect();
        pts.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        let vals: Vec<f64> = (0..pts.len()).map(|_| r.uniform() * 10.0 - 5.0).collect();
        let si = ScatteredInterpolant::new(&pts, &vals, ScatteredMethod::Natural).unwrap();
        let q = [qx, qy];
        let w = si.sibson_coordinates(q).unwrap();
        let soma: f64 = w.iter().map(|p| p.1).sum();
        prop_assert!((soma - 1.0).abs() <= 1e-12);
        prop_assert!(w.iter().all(|p| p.1 >= 0.0));
        for c in 0..2 {
            let m: f64 = w.iter().map(|p| p.1 * pts[p.0][c]).sum();
            prop_assert!((m - q[c]).abs() <= 1e-11, "{} vs {}", m, q[c]);
        }
        let v = si.eval(q, Extrap::Nan);
        let (lo, hi) = w.iter().map(|p| vals[p.0]).fold((f64::INFINITY, f64::NEG_INFINITY), |a, x| (a.0.min(x), a.1.max(x)));
        prop_assert!(v >= lo - 1e-12 && v <= hi + 1e-12);
    }
}
