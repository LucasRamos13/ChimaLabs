//! F051 — otimização básica e raízes: testes de aceitação, oráculo, propriedades, bordas e erros.
//!
//! Oráculo: `tests/dados/F051/referencias.csv`, de `ferramentas/oraculos/f051_raizes_minimos.py`
//! (mpmath, 60 dígitos). As funções abaixo têm os mesmos nomes do script.

use std::cell::Cell;

use chimalab_math::linalg::Matrix;
use chimalab_math::optim::{
    Control, ExitFlag, OptimError, Options, Procedure, Start, ZeroMethod, fminbnd, fminsearch, fzero, nnls,
};
use proptest::prelude::*;

const EPS: f64 = f64::EPSILON;

/// exp e sen deterministas (só +, −, ×, ÷, corretamente arredondados em IEEE 754), para testes que
/// fixam a contagem exata de avaliações: a `exp`/`sin` da libm variam alguns ulp entre
/// plataformas (no Windows, `exp_a` dava 23 avaliações em vez de 24), e o caminho do Brent muda.
fn exp_det(x: f64) -> f64 {
    // Redução de Cody–Waite: x = k·ln 2 + r, |r| ≤ ln2/2, com ln 2 em duas partes (a alta tem
    // zeros finais, então k·LN2_HI é exato); Taylor de 22 termos em r; 2^k exato.
    const LN2_HI: f64 = 0.693_147_180_369_123_8;
    const LN2_LO: f64 = 1.908_214_929_270_587_7e-10;
    let k = (x / std::f64::consts::LN_2).round();
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let mut s = 1.0;
    for j in (1..=22).rev() {
        s = 1.0 + s * r / j as f64;
    }
    s * 2f64.powi(k as i32)
}

fn sin_det(x: f64) -> f64 {
    let dois_pi = 2.0 * std::f64::consts::PI;
    let r = x - (x / dois_pi).round() * dois_pi;
    // Taylor ímpar até r⁴¹ em |r| ≤ π (resto < 1e-17).
    let r2 = r * r;
    let mut s = 1.0;
    for k in (1..=20).rev() {
        s = 1.0 - s * r2 / ((2 * k) * (2 * k + 1)) as f64;
    }
    r * s
}

fn raiz_fn(nome: &str) -> fn(f64) -> f64 {
    match nome {
        "wallis" => |x| x * x * x - 2.0 * x - 5.0,
        "cos_x" => |x| x.cos() - x,
        "exp_2" => |x| x.exp() - 2.0,
        "raiz2" => |x| x * x - 2.0,
        "cubo_multiplo" => |x| (x - 1.0).powi(3),
        "raiz_cubica" => |x| (x - 1.0).cbrt(),
        "seno" => |x| x.sin() - x / 2.0,
        "tan_x" => |x| x.tan() - x,
        "kepler" => |x| x - 0.9 * x.sin() - 0.1,
        "exp_enorme" => |x| x.exp() - 1e300,
        "minusculo" => |x| x - 1e-300,
        "atan_ingreme" => |x| (1e6 * (x - 0.3)).atan(),
        "potencia9" => |x| (x - 0.5).powi(9),
        "xexp" => |x| x * (-x).exp() - 1e-8,
        "log_dominio" => |x| x.ln() - 1.0,
        _ => panic!("{nome}"),
    }
}

fn minimo_fn(nome: &str) -> fn(f64) -> f64 {
    match nome {
        "quadratica" => |x| (x - 2.0).powi(2),
        "quartica" => |x| x.powi(4) - x,
        "seno_neg" => |x| -x.sin(),
        "xlogx" => |x| x * x.ln(),
        "cosseno" => |x| x.cos(),
        "monotona" => |x| x.exp(),
        "monotona_dir" => |x| -x,
        "abs_desloc" => |x| (x - 0.7).abs(),
        "escala_grande" => |x| (x - 1e6).powi(2) + 1.0,
        _ => panic!("{nome}"),
    }
}

struct Caso {
    tipo: String,
    nome: String,
    a: f64,
    b: f64,
    x: f64,
    fx: f64,
}

fn referencias() -> Vec<Caso> {
    let txt = include_str!("../../../tests/dados/F051/referencias.csv");
    txt.lines()
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            Caso {
                tipo: c[0].into(),
                nome: c[1].into(),
                a: c[2].parse().unwrap(),
                b: c[3].parse().unwrap(),
                x: c[4].parse().unwrap(),
                fx: c[5].parse().unwrap(),
            }
        })
        .collect()
}

fn opts() -> Options<'static> {
    Options::default()
}

/// Tolerância das raízes com Brent (`tol_x` = 0): o critério de parada dá |x − r| ≤ 4ε|x|; o erro
/// de avaliação de f desloca a mudança de sinal em até ~ε|x|/|f′| relativo, < 12ε|x| em todos os
/// casos (o pior, kepler, tem f′ = 0,27).
fn tol_brent(r: f64) -> f64 {
    16.0 * EPS * r.abs() + f64::MIN_POSITIVE
}

#[test]
fn ca2_fzero_com_intervalo_brent_e_itp() {
    let mut tabela = Vec::new();
    for c in referencias().iter().filter(|c| c.tipo == "raiz") {
        let f = raiz_fn(&c.nome);
        let rb = fzero(f, Start::Interval(c.a, c.b), &mut opts()).unwrap();
        assert_eq!(rb.exit, ExitFlag::Converged, "{}", c.nome);
        assert!(
            (rb.x - c.x).abs() <= tol_brent(c.x),
            "brent {}: {} vs {} ({:e})",
            c.nome,
            rb.x,
            c.x,
            (rb.x - c.x).abs() / c.x.abs()
        );
        assert_eq!(rb.fval, f(rb.x));
        let (lo, hi) = rb.interval.unwrap();
        assert!(lo <= rb.x && rb.x <= hi, "{}", c.nome);
        // ITP: tolerância absoluta ε = ε_máq·max(|a|, |b|). Garantia do artigo, em aritmética
        // exata: no máximo ⌈log₂((b − a)/2ε)⌉ + n₀ iterações, com n₀ = 1. Em ponto flutuante,
        // quando 2ε vale poucos ulps, os pontos médios arredondados não dividem a largura
        // exatamente ao meio, e pode ser preciso mais uma (exp_enorme: 2ε = 2,7 ulps).
        let mut o = Options {
            zero_method: ZeroMethod::Itp,
            ..opts()
        };
        let ri = fzero(f, Start::Interval(c.a, c.b), &mut o).unwrap();
        assert_eq!(ri.exit, ExitFlag::Converged, "{}", c.nome);
        let eps = EPS * c.a.abs().max(c.b.abs());
        assert!(
            (ri.x - c.x).abs() <= 2.0 * eps + tol_brent(c.x),
            "itp {}: {} vs {}",
            c.nome,
            ri.x,
            c.x
        );
        let n_max = ((c.b - c.a) / (2.0 * eps)).log2().ceil() as usize + 2;
        assert!(ri.iterations <= n_max, "{}: {} > {n_max}", c.nome, ri.iterations);
        tabela.push((c.nome.clone(), rb.fun_evals, ri.fun_evals));
    }
    // Avaliações por caso (registradas em docs/progresso/F051.md): `--nocapture`.
    for (n, b, i) in &tabela {
        println!("{n:>14} brent {b:>4} itp {i:>4}");
    }
}

#[test]
fn fminbnd_contra_oraculo() {
    for c in referencias().iter().filter(|c| c.tipo == "minimo") {
        let f = minimo_fn(&c.nome);
        let r = fminbnd(f, c.a, c.b, &mut opts()).unwrap();
        assert_eq!(r.exit, ExitFlag::Converged, "{}", c.nome);
        // Precisão em x de ordem √ε·|x|: perto de um mínimo suave, f varia com o quadrado do erro
        // em x (Brent, cap. 5). 1e-7·max(1, |x|) cobre o critério de parada 2(√ε|x| + tol/3) com
        // o erro de arredondamento de f.
        assert!(
            (r.x - c.x).abs() <= 1e-7 * c.x.abs().max(1.0),
            "{}: {} vs {}",
            c.nome,
            r.x,
            c.x
        );
        // Nunca abaixo do mínimo verdadeiro (a menos de arredondamento).
        assert!(r.fval >= c.fx - 4.0 * EPS * c.fx.abs().max(1.0), "{}", c.nome);
        assert_eq!(r.fval, f(r.x));
        // Avaliações exatas (regressão do caminho de Brent; inclui as 2 dos extremos). Funções
        // suaves convergem por parábolas em ≤ 16; a seção áurea pura levaria ~50. Os casos
        // monótonos e |x − 0,7| exercitam a seção áurea.
        let esperado = match c.nome.as_str() {
            "quadratica" | "seno_neg" | "escala_grande" => 8,
            "quartica" => 15,
            "xlogx" => 16,
            "cosseno" => 11,
            "monotona" => 40,
            "monotona_dir" => 39,
            "abs_desloc" => 23,
            _ => unreachable!(),
        };
        assert_eq!(r.fun_evals, esperado, "{}", c.nome);
        assert_eq!(r.iterations, esperado - 3, "{}", c.nome);
    }
    // Mínimo no extremo: exato (os extremos são conferidos).
    let r = fminbnd(|x| x.exp(), -1.0, 2.0, &mut opts()).unwrap();
    assert_eq!((r.x, r.fval), (-1.0, (-1f64).exp()));
    let r = fminbnd(|x| -x, 0.0, 3.0, &mut opts()).unwrap();
    assert_eq!(r.x, 3.0);
}

fn rosenbrock(x: &[f64]) -> f64 {
    100.0 * (x[1] - x[0] * x[0]).powi(2) + (1.0 - x[0]).powi(2)
}

#[test]
fn ca1_fminsearch_rosenbrock() {
    let r = fminsearch(rosenbrock, &[-1.2, 1.0], &mut opts()).unwrap();
    assert_eq!(r.exit, ExitFlag::Converged);
    assert!(
        (r.x[0] - 1.0).abs() < 1e-6 && (r.x[1] - 1.0).abs() < 1e-6,
        "{:?}",
        r.x
    );
    assert!(r.fval < 1e-12, "{}", r.fval);
    // Parâmetros adaptativos também convergem.
    let mut o = Options {
        adaptive: true,
        ..opts()
    };
    let r = fminsearch(rosenbrock, &[-1.2, 1.0], &mut o).unwrap();
    assert!((r.x[0] - 1.0).abs() < 1e-6 && (r.x[1] - 1.0).abs() < 1e-6);
}

/// McKinnon (SIAM J. Optim. 9, 1998), τ = 2, θ = 6, φ = 60: com o simplex inicial do artigo, o
/// Nelder–Mead clássico converge para (0, 0), que não é estacionário; o mínimo é (0, −½), f = −¼.
#[test]
fn mckinnon_reinicio_escapa_do_ponto_nao_estacionario() {
    let f = |p: &[f64]| {
        let (x, y) = (p[0], p[1]);
        let base = y + y * y;
        if x <= 0.0 {
            6.0 * 60.0 * x * x + base
        } else {
            6.0 * x * x + base
        }
    };
    let l1 = (1.0 + 33f64.sqrt()) / 8.0;
    let l2 = (1.0 - 33f64.sqrt()) / 8.0;
    let simplex = || Some(vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![l1, l2]]);
    let mut o = Options {
        initial_simplex: simplex(),
        restarts: Some(0),
        ..opts()
    };
    let r = fminsearch(f, &[0.0, 0.0], &mut o).unwrap();
    assert!(
        r.x[0].abs() < 1e-6 && r.x[1].abs() < 1e-6,
        "sem reinício: {:?}",
        r.x
    );
    let mut o = Options {
        initial_simplex: simplex(),
        ..opts()
    };
    let r = fminsearch(f, &[0.0, 0.0], &mut o).unwrap();
    assert!((r.x[1] + 0.5).abs() < 1e-6 && r.x[0].abs() < 1e-6, "{:?}", r.x);
    assert!((r.fval + 0.25).abs() < 1e-12);
}

#[test]
fn fminsearch_dimensao_alta_dominio_e_unidimensional() {
    // Quadrática mal escalada em 10-D com parâmetros adaptativos.
    let q = |x: &[f64]| {
        x.iter()
            .enumerate()
            .map(|(i, v)| (i + 1) as f64 * (v - (i + 1) as f64).powi(2))
            .sum::<f64>()
    };
    let mut o = Options {
        adaptive: true,
        max_fun_evals: Some(100_000),
        max_iter: Some(100_000),
        ..opts()
    };
    let r = fminsearch(q, &[0.0; 10], &mut o).unwrap();
    assert_eq!(r.exit, ExitFlag::Converged);
    for (i, v) in r.x.iter().enumerate() {
        assert!((v - (i + 1) as f64).abs() < 1e-4, "{i}: {v}");
    }
    // NaN fora do domínio conta como +∞.
    let g = |x: &[f64]| (x[0].ln() - 1.0).powi(2) + x[1] * x[1];
    let r = fminsearch(g, &[5.0, 1.0], &mut opts()).unwrap();
    assert!((r.x[0] - std::f64::consts::E).abs() < 1e-5, "{:?}", r.x);
    // 1-D.
    let r = fminsearch(|x| (x[0] - 3.0).powi(2), &[0.0], &mut opts()).unwrap();
    assert!((r.x[0] - 3.0).abs() < 1e-7);
    // Passo inicial dado; ponto inicial no mínimo exato.
    let mut o = Options {
        initial_step: Some(vec![0.5, -0.5]),
        ..opts()
    };
    let r = fminsearch(rosenbrock, &[1.0, 1.0], &mut o).unwrap();
    assert_eq!((r.x.clone(), r.fval), (vec![1.0, 1.0], 0.0));
}

#[test]
fn ca3_funcao_de_saida_a_cada_iteracao() {
    // Chamada uma vez na avaliação inicial e uma vez por iteração, com o histórico idêntico.
    let chamadas = Cell::new(0usize);
    let mut o = Options {
        history: true,
        output: Some(Box::new(|it| {
            assert_eq!(it.iteration, chamadas.get());
            chamadas.set(chamadas.get() + 1);
            Control::Continue
        })),
        ..opts()
    };
    let r = fzero(|x| x.cos() - x, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!(chamadas.get(), r.iterations + 1);
    assert_eq!(r.history.len(), r.iterations + 1);
    assert_eq!(r.history[0].procedure, Procedure::Initial);
    assert!(
        r.history[1..]
            .iter()
            .all(|h| matches!(h.procedure, Procedure::Bisection | Procedure::Interpolation))
    );
    drop(o);
    for which in 0..3 {
        let chamadas = Cell::new(0usize);
        let mut o = Options {
            history: true,
            output: Some(Box::new(|_| {
                chamadas.set(chamadas.get() + 1);
                Control::Continue
            })),
            ..opts()
        };
        let (iters, hist) = match which {
            0 => {
                let r = fminbnd(|x| (x - 2.0).powi(2), 0.0, 5.0, &mut o).unwrap();
                (r.iterations, r.history)
            }
            1 => {
                let r = fminsearch(rosenbrock, &[-1.2, 1.0], &mut o).unwrap();
                (r.iterations, r.history)
            }
            _ => {
                let a = Matrix::from_rows(&[&[1.0, 0.0], &[0.0, 1.0], &[1.0, 1.0]]).unwrap();
                let r = nnls(&a, &[1.0, -1.0, 0.5], &mut o).unwrap();
                (r.iterations, r.history)
            }
        };
        drop(o);
        assert!(chamadas.get() > iters, "{which}");
        assert_eq!(hist.len(), chamadas.get(), "{which}");
        let numeros: Vec<usize> = hist.iter().map(|h| h.iteration).collect();
        assert!(numeros.windows(2).all(|w| w[0] <= w[1]), "{which}");
    }
    // A função de saída interrompe: ExitFlag::Stopped, código −1.
    let mut o = Options {
        output: Some(Box::new(|it| {
            if it.iteration == 3 {
                Control::Stop
            } else {
                Control::Continue
            }
        })),
        ..opts()
    };
    let r = fminsearch(rosenbrock, &[-1.2, 1.0], &mut o).unwrap();
    assert_eq!((r.exit, r.exit.code(), r.iterations), (ExitFlag::Stopped, -1, 3));
    // Exibição iterativa: uma linha por iteração.
    let linha = format!("{}", r_hist_linha());
    assert!(linha.contains("inicial"), "{linha}");
}

fn r_hist_linha() -> chimalab_math::optim::Iteration {
    let mut o = Options {
        history: true,
        ..opts()
    };
    fzero(|x| x - 0.5, Start::Interval(0.0, 1.0), &mut o)
        .unwrap()
        .history
        .remove(0)
}

#[test]
fn fzero_com_chute_e_casos_especiais() {
    // Busca dos dois lados; lado com NaN (ln de negativo) abandonado.
    let r = fzero(|x| x.cos() - x, Start::Guess(0.0), &mut opts()).unwrap();
    assert!((r.x - 0.739_085_133_215_160_6).abs() < 1e-15);
    let r = fzero(|x| x.ln() - 1.0, Start::Guess(0.5), &mut opts()).unwrap();
    assert!((r.x - std::f64::consts::E).abs() < 4.0 * EPS * 3.0, "{}", r.x);
    let r = fzero(|x| x - 1000.0, Start::Guess(-3.0), &mut opts()).unwrap();
    assert_eq!(r.x, 1000.0);
    // Chute que já é raiz.
    let r = fzero(|x| x - 2.0, Start::Guess(2.0), &mut opts()).unwrap();
    assert_eq!((r.x, r.fun_evals, r.exit), (2.0, 1, ExitFlag::Converged));
    // Raiz exata num extremo; intervalo degenerado com raiz.
    let r = fzero(|x| x - 1.0, Start::Interval(1.0, 2.0), &mut opts()).unwrap();
    assert_eq!((r.x, r.fval), (1.0, 0.0));
    let r = fzero(|x| x - 1.0, Start::Interval(0.0, 1.0), &mut opts()).unwrap();
    assert_eq!(r.x, 1.0);
    let r = fzero(|x| x - 1.0, Start::Interval(1.0, 1.0), &mut opts()).unwrap();
    assert_eq!(r.x, 1.0);
    // Polo: mudança de sinal sem raiz é sinalizada.
    for (f, a, b) in [
        ((|x: f64| 1.0 / x) as fn(f64) -> f64, -1.0, 2.0),
        (|x: f64| x.tan(), 1.0, 2.0),
        (|x: f64| if x < 0.3 { -1.0 } else { 1.0 }, 0.0, 1.0),
    ] {
        for m in [ZeroMethod::Brent, ZeroMethod::Itp] {
            let mut o = Options {
                zero_method: m,
                ..opts()
            };
            let r = fzero(f, Start::Interval(a, b), &mut o).unwrap();
            assert_eq!(r.exit, ExitFlag::SignChangeNotRoot, "{m:?} {a} {b}");
            assert_eq!(r.exit.code(), -5);
        }
    }
    // NaN no interior: termina com NonFinite.
    let f = |x: f64| {
        if (0.4..0.6).contains(&x) {
            f64::NAN
        } else {
            x - 0.5
        }
    };
    for m in [ZeroMethod::Brent, ZeroMethod::Itp] {
        let mut o = Options {
            zero_method: m,
            ..opts()
        };
        let r = fzero(f, Start::Interval(0.0, 1.0), &mut o).unwrap();
        assert_eq!((r.exit, r.exit.code()), (ExitFlag::NonFinite, -3), "{m:?}");
    }
    // Limites: iterações e avaliações (código 0).
    let mut o = Options {
        max_iter: Some(2),
        ..opts()
    };
    let r = fzero(|x| x.cos() - x, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!(
        (r.exit, r.iterations, r.exit.code()),
        (ExitFlag::MaxIterations, 2, 0)
    );
    let mut o = Options {
        max_fun_evals: Some(4),
        ..opts()
    };
    let r = fzero(|x| x.cos() - x, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!((r.exit, r.fun_evals), (ExitFlag::MaxFunEvals, 4));
    let mut o = Options {
        max_iter: Some(3),
        ..opts()
    };
    let r = fzero(|x| x * x + 1.0, Start::Guess(1.0), &mut o).unwrap();
    assert_eq!(r.exit, ExitFlag::MaxIterations);
    // tol_x grande: para cedo, dentro da tolerância.
    let mut o = Options {
        tol_x: Some(1e-3),
        ..opts()
    };
    let r = fzero(|x| x.cos() - x, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert!((r.x - 0.739_085_133_215_160_6).abs() <= 1e-3);
    for m in ExitFlag::Converged.message().chars().take(1) {
        assert!(m.is_alphabetic());
    }
}

#[test]
fn fminbnd_bordas() {
    // Intervalo degenerado: o próprio ponto.
    let r = fminbnd(|x| x * x, 2.0, 2.0, &mut opts()).unwrap();
    assert_eq!((r.x, r.fval), (2.0, 4.0));
    // NaN conta como +∞: mínimo fora da região NaN.
    let r = fminbnd(
        |x| if x < 1.0 { f64::NAN } else { (x - 2.0).powi(2) },
        0.0,
        3.0,
        &mut opts(),
    )
    .unwrap();
    assert!((r.x - 2.0).abs() < 1e-7);
    // Função constante: qualquer ponto, sem erro.
    let r = fminbnd(|_| 1.0, -1.0, 1.0, &mut opts()).unwrap();
    assert_eq!((r.fval, r.exit), (1.0, ExitFlag::Converged));
    // Empate com um extremo: fica o ponto interior (o extremo só vence se for estritamente menor).
    assert!(r.x > -1.0 && r.x < 1.0, "{}", r.x);
    let mut o = Options {
        max_iter: Some(3),
        ..opts()
    };
    let r = fminbnd(|x| x.cos(), 3.0, 4.0, &mut o).unwrap();
    assert_eq!((r.exit, r.iterations), (ExitFlag::MaxIterations, 3));
    // Interrompido: não confere os extremos.
    let mut o = Options {
        output: Some(Box::new(|_| Control::Stop)),
        ..opts()
    };
    let r = fminbnd(|x| -x, 0.0, 3.0, &mut o).unwrap();
    assert_eq!((r.exit, r.fun_evals), (ExitFlag::Stopped, 1));
}

/// Solução de referência do NNLS por enumeração dos conjuntos livres: para A de posto completo
/// em colunas, a solução é única e é o único ponto que satisfaz as condições KKT.
fn nnls_forca_bruta(a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let (m, n) = (a.len(), a[0].len());
    let mut melhor: Option<(f64, Vec<f64>)> = None;
    for mask in 0u32..(1 << n) {
        let cols: Vec<usize> = (0..n).filter(|j| mask >> j & 1 == 1).collect();
        let k = cols.len();
        // Equações normais em aritmética com eliminação gaussiana (n ≤ 5, bem condicionado).
        let mut g = vec![vec![0.0; k + 1]; k];
        for (p, &jp) in cols.iter().enumerate() {
            for (q, &jq) in cols.iter().enumerate() {
                g[p][q] = (0..m).map(|i| a[i][jp] * a[i][jq]).sum();
            }
            g[p][k] = (0..m).map(|i| a[i][jp] * b[i]).sum();
        }
        for c in 0..k {
            let piv = (c..k)
                .max_by(|&i, &j| g[i][c].abs().total_cmp(&g[j][c].abs()))
                .unwrap();
            g.swap(c, piv);
            for i in 0..k {
                if i != c {
                    let r = g[i][c] / g[c][c];
                    for col in c..=k {
                        g[i][col] -= r * g[c][col];
                    }
                }
            }
        }
        let mut x = vec![0.0; n];
        for (p, &j) in cols.iter().enumerate() {
            x[j] = g[p][k] / g[p][p];
        }
        if x.iter().any(|&v| v < 0.0) {
            continue;
        }
        let res: f64 = (0..m)
            .map(|i| (b[i] - (0..n).map(|j| a[i][j] * x[j]).sum::<f64>()).powi(2))
            .sum();
        if melhor.as_ref().is_none_or(|(r, _)| res < *r) {
            melhor = Some((res, x));
        }
    }
    melhor.unwrap().1
}

fn kkt(a: &Matrix, b: &[f64], x: &[f64], dual: &[f64]) {
    let (m, n) = a.shape();
    let escala = (0..n)
        .map(|j| (0..m).map(|i| a.get(i, j).abs()).sum::<f64>())
        .fold(0.0f64, f64::max)
        * b.iter().fold(1.0f64, |s, v| s.max(v.abs()));
    for j in 0..n {
        assert!(x[j] >= 0.0);
        if x[j] > 0.0 {
            assert!(dual[j].abs() <= 1e-9 * escala, "w[{j}] = {}", dual[j]);
        } else {
            assert!(dual[j] <= 1e-9 * escala, "w[{j}] = {}", dual[j]);
        }
    }
}

#[test]
fn nnls_exemplos_e_bordas() {
    let a = Matrix::from_rows(&[&[1.0, 0.0], &[0.0, 1.0]]).unwrap();
    let r = nnls(&a, &[-1.0, 2.0], &mut opts()).unwrap();
    assert_eq!(r.x, vec![0.0, 2.0]);
    assert_eq!(r.resnorm, 1.0);
    assert_eq!(r.residual, vec![-1.0, 0.0]);
    assert_eq!(r.exit, ExitFlag::Converged);
    // b = 0 → x = 0, sem iterações.
    let r = nnls(&a, &[0.0, 0.0], &mut opts()).unwrap();
    assert_eq!((r.x, r.iterations), (vec![0.0, 0.0], 0));
    // Coluna nula; b negativo em tudo.
    let a = Matrix::from_rows(&[&[1.0, 0.0], &[1.0, 0.0]]).unwrap();
    let r = nnls(&a, &[1.0, 3.0], &mut opts()).unwrap();
    assert_eq!(r.x, vec![2.0, 0.0]);
    let r = nnls(&a, &[-1.0, -3.0], &mut opts()).unwrap();
    assert_eq!(r.x, vec![0.0, 0.0]);
    // Sem variáveis; sem equações.
    let r = nnls(&Matrix::zeros(3, 0), &[1.0, 2.0, 3.0], &mut opts()).unwrap();
    assert!(r.x.is_empty() && r.resnorm == 14.0);
    let r = nnls(&Matrix::zeros(0, 2), &[], &mut opts()).unwrap();
    assert_eq!(r.x, vec![0.0, 0.0]);
    // Subdeterminado (m < n): KKT.
    let a = Matrix::from_rows(&[&[1.0, 2.0, -1.0, 0.5], &[0.3, -1.0, 2.0, 1.0]]).unwrap();
    let b = [1.0, 2.0];
    let r = nnls(&a, &b, &mut opts()).unwrap();
    kkt(&a, &b, &r.x, &r.dual);
    assert!(r.resnorm < 1e-20, "{}", r.resnorm);
    // Limite de iterações.
    let a = Matrix::from_rows(&[&[1.0, 0.0], &[0.0, 1.0]]).unwrap();
    let mut o = Options {
        max_iter: Some(1),
        ..opts()
    };
    let r = nnls(&a, &[1.0, 2.0], &mut o).unwrap();
    assert_eq!((r.exit, r.iterations), (ExitFlag::MaxIterations, 1));
}

#[test]
fn erros_com_identificador() {
    let casos: Vec<(OptimError, &str)> = vec![
        (
            fzero(|x| x, Start::Interval(f64::NAN, 1.0), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:intervaloInvalido",
        ),
        (
            fzero(|x| x, Start::Interval(0.0, f64::INFINITY), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:intervaloInvalido",
        ),
        (
            fzero(|x| x, Start::Interval(1.0, 0.0), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:intervaloInvalido",
        ),
        (
            fzero(|x| x * x + 1.0, Start::Interval(-1.0, 1.0), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:semMudancaDeSinal",
        ),
        (
            fzero(|x| x * x + 1.0, Start::Guess(1.0), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:semMudancaDeSinal",
        ),
        (
            fzero(|x| x, Start::Guess(f64::NAN), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        (
            fzero(|x: f64| x.ln(), Start::Guess(-1.0), &mut opts()).unwrap_err(),
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        (
            fzero(
                |x| x,
                Start::Interval(-1.0, 1.0),
                &mut Options {
                    tol_x: Some(-1.0),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:toleranciaInvalida",
        ),
        (
            fminbnd(|x| x, 1.0, 0.0, &mut opts()).unwrap_err(),
            "chimalab:otimizacao:intervaloInvalido",
        ),
        (
            fminbnd(|x| x, 0.0, f64::INFINITY, &mut opts()).unwrap_err(),
            "chimalab:otimizacao:intervaloInvalido",
        ),
        (
            fminbnd(
                |x| x,
                0.0,
                1.0,
                &mut Options {
                    tol_fun: Some(f64::NAN),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:toleranciaInvalida",
        ),
        (
            fminsearch(|x| x[0], &[], &mut opts()).unwrap_err(),
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        (
            fminsearch(|x| x[0], &[f64::INFINITY], &mut opts()).unwrap_err(),
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        (
            fminsearch(|_| f64::NAN, &[1.0], &mut opts()).unwrap_err(),
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        (
            fminsearch(
                |x| x[0],
                &[1.0, 2.0],
                &mut Options {
                    initial_step: Some(vec![1.0]),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
        (
            fminsearch(
                |x| x[0],
                &[1.0],
                &mut Options {
                    initial_step: Some(vec![0.0]),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
        (
            fminsearch(
                |x| x[0],
                &[1.0, 2.0],
                &mut Options {
                    initial_simplex: Some(vec![vec![0.0, 0.0], vec![1.0, 1.0]]),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
        (
            fminsearch(
                |x| x[0],
                &[1.0, 2.0],
                &mut Options {
                    initial_simplex: Some(vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]]),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
        (
            fminsearch(
                |x| x[0],
                &[1.0, 2.0],
                &mut Options {
                    initial_simplex: Some(vec![vec![0.0, 0.0], vec![1.0, f64::NAN], vec![2.0, 0.0]]),
                    ..opts()
                },
            )
            .unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
        (
            nnls(&Matrix::zeros(2, 2), &[1.0], &mut opts()).unwrap_err(),
            "chimalab:otimizacao:formaIncompativel",
        ),
        (
            nnls(&Matrix::zeros(1, 1), &[f64::NAN], &mut opts()).unwrap_err(),
            "chimalab:otimizacao:opcaoInvalida",
        ),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
        assert!(!e.to_string().is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// NNLS contra a enumeração dos conjuntos livres (oráculo exato para posto completo).
    #[test]
    fn nnls_contra_forca_bruta(
        n in 1usize..=5,
        extra in 0usize..4,
        dados in prop::collection::vec(-1.0f64..1.0, 9 * 5 + 9),
    ) {
        let m = n + extra;
        let a: Vec<Vec<f64>> = (0..m).map(|i| (0..n).map(|j| dados[i * 5 + j]).collect()).collect();
        let b: Vec<f64> = (0..m).map(|i| dados[45 + i]).collect();
        let am = Matrix::from_fn(m, n, |i, j| a[i][j]);
        // Posto completo e bem condicionado.
        let sv = chimalab_math::linalg::singular_values(&am).unwrap();
        prop_assume!(sv.iter().cloned().fold(f64::INFINITY, f64::min) > 1e-3);
        let r = nnls(&am, &b, &mut Options::default()).unwrap();
        let x_ref = nnls_forca_bruta(&a, &b);
        // A força bruta usa equações normais, com erro ~ ε·κ²·‖x‖ (o NNLS usa QR, mais
        // preciso): a tolerância segue esse limite. Contraexemplo achado pelo proptest com
        // κ ≈ 900 e ‖x‖ ≈ 128 (diferença de 1,2e-9), guardado em optim_f051.proptest-regressions.
        let kappa = sv[0] / sv.iter().cloned().fold(f64::INFINITY, f64::min);
        let xmax = x_ref.iter().fold(1.0f64, |acc, v| acc.max(v.abs()));
        let tol = (8.0 * f64::EPSILON * kappa * kappa * xmax).max(1e-9);
        for j in 0..n {
            prop_assert!((r.x[j] - x_ref[j]).abs() <= tol, "{:?} vs {:?}", r.x, x_ref);
        }
        kkt(&am, &b, &r.x, &r.dual);
        prop_assert_eq!(r.exit, ExitFlag::Converged);
    }

    /// Raiz de polinômio cúbico com raiz conhecida: os dois métodos dentro da tolerância.
    #[test]
    fn fzero_raiz_conhecida(r in -1e3f64..1e3, c in 0.1f64..10.0) {
        let f = move |x: f64| (x - r) * ((x - r) * (x - r) + c);
        for m in [ZeroMethod::Brent, ZeroMethod::Itp] {
            let mut o = Options { zero_method: m, ..Options::default() };
            let s = fzero(f, Start::Interval(r - 7.0, r + 11.0), &mut o).unwrap();
            let tol = match m {
                ZeroMethod::Brent => 16.0 * EPS * r.abs() + 1e-300,
                ZeroMethod::Itp => 2.0 * EPS * (r.abs() + 11.0) + 16.0 * EPS * r.abs(),
            };
            prop_assert!((s.x - r).abs() <= tol, "{:?} {} vs {}", m, s.x, r);
        }
    }

    /// Mínimo de parábola deslocada: fminbnd e fminsearch.
    #[test]
    fn minimos_de_parabolas(x0 in -100.0f64..100.0, k in 0.01f64..100.0) {
        let f = move |x: f64| k * (x - x0) * (x - x0) + 3.0;
        let r = fminbnd(f, -150.0, 150.0, &mut Options::default()).unwrap();
        prop_assert!((r.x - x0).abs() <= 1e-7 * x0.abs().max(1.0) + 1e-6 / k.sqrt());
        let r = fminsearch(|v| f(v[0]), &[0.0], &mut Options::default()).unwrap();
        prop_assert!((r.x[0] - x0).abs() <= 1e-6 * x0.abs().max(1.0), "{} vs {}", r.x[0], x0);
    }
}

/// Caminhos raros do Nelder–Mead: encolhimento, passo absorvido pelo arredondamento e simplex
/// degenerado.
#[test]
fn fminsearch_caminhos_raros() {
    // |‖x‖ − 1| tem mínimo (0) em todo o círculo unitário e força encolhimentos.
    let anel = |x: &[f64]| ((x[0] * x[0] + x[1] * x[1]).sqrt() - 1.0).abs();
    let mut o = Options {
        history: true,
        ..opts()
    };
    let r = fminsearch(anel, &[0.7, -0.3], &mut o).unwrap();
    assert!(r.history.iter().any(|h| h.procedure == Procedure::Shrink));
    assert!(r.fval < 1e-8, "{}", r.fval);
    // O encolhimento mantém o melhor ponto: o valor nunca piora ao longo do histórico.
    assert!(r.history.windows(2).all(|w| w[1].fval <= w[0].fval));
    // Passo menor que o ulp de x0: usa o vizinho em ponto flutuante (nos dois sentidos).
    for x0 in [1e20, -1e20] {
        let mut o = Options {
            initial_step: Some(vec![1.0]),
            max_iter: Some(20),
            ..opts()
        };
        // −|x − x0| é exata nessa escala: o vizinho já melhora, e o simplex expande.
        let r = fminsearch(|x| -(x[0] - x0).abs(), &[x0], &mut o).unwrap();
        assert!(r.fval < -1e4, "{x0}: {}", r.fval);
    }
    // Simplex com todos os pontos iguais.
    let mut o = Options {
        initial_simplex: Some(vec![vec![1.0, 1.0]; 3]),
        ..opts()
    };
    assert_eq!(
        fminsearch(rosenbrock, &[1.0, 1.0], &mut o).unwrap_err().id(),
        "chimalab:otimizacao:opcaoInvalida"
    );
}

#[test]
fn fzero_caminhos_raros() {
    // Intervalo achado à esquerda do chute.
    let r = fzero(|x| x + 3.0, Start::Guess(0.0), &mut opts()).unwrap();
    assert_eq!(r.x, -3.0);
    let (a, b) = r.interval.unwrap();
    assert!(a <= -3.0 && -3.0 <= b && b - a < 3.0);
    // Busca até o estouro de x: os dois lados abandonados, erro sem mudança de sinal.
    let mut o = Options {
        max_iter: Some(100_000),
        max_fun_evals: Some(100_000),
        ..opts()
    };
    let e = fzero(|_| 1.0, Start::Guess(1.0), &mut o).unwrap_err();
    assert_eq!(e.id(), "chimalab:otimizacao:semMudancaDeSinal");
    // Interrompido já na avaliação inicial.
    let mut o = Options {
        output: Some(Box::new(|_| Control::Stop)),
        ..opts()
    };
    let r = fzero(|x| x - 0.25, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!((r.exit, r.iterations, r.x), (ExitFlag::Stopped, 0, 0.0));
    // ITP com limite de iterações e interrompido.
    let mut o = Options {
        zero_method: ZeroMethod::Itp,
        max_iter: Some(3),
        ..opts()
    };
    let r = fzero(|x| x.cos() - x, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!((r.exit, r.iterations), (ExitFlag::MaxIterations, 3));
    // ITP com raiz exata num ponto interior.
    let mut o = Options {
        zero_method: ZeroMethod::Itp,
        ..opts()
    };
    let r = fzero(|x| x - 0.5, Start::Interval(0.0, 1.0), &mut o).unwrap();
    assert_eq!((r.x, r.fval, r.interval), (0.5, 0.0, Some((0.5, 0.5))));
    // Raiz em 0 dentro de um intervalo que o cruza, e polo em 0: bissecção na ordem dos floats,
    // poucas avaliações apesar da tolerância relativa.
    let r = fzero(|x: f64| x.sinh(), Start::Interval(-1.0, 3.0), &mut opts()).unwrap();
    assert!(
        r.x.abs() <= 1e-300 && r.fun_evals < 100,
        "{} {}",
        r.x,
        r.fun_evals
    );
    let r = fzero(|x: f64| 1.0 / x, Start::Interval(-1.0, 2.0), &mut opts()).unwrap();
    assert!(r.x.abs() < 1e-300 && r.fun_evals < 100, "{} {}", r.x, r.fun_evals);
}

#[test]
fn nomes_mensagens_e_depuracao() {
    use Procedure::*;
    let todos = [
        Initial,
        Search,
        Bisection,
        Interpolation,
        Itp,
        Golden,
        Parabolic,
        Boundary,
        Reflect,
        Expand,
        ContractOutside,
        ContractInside,
        Shrink,
        Restart,
        Add,
        Remove,
    ];
    let nomes: std::collections::HashSet<&str> = todos.iter().map(|p| p.name()).collect();
    assert_eq!(nomes.len(), todos.len());
    let flags = [
        (ExitFlag::Converged, 1),
        (ExitFlag::MaxIterations, 0),
        (ExitFlag::MaxFunEvals, 0),
        (ExitFlag::Stopped, -1),
        (ExitFlag::NonFinite, -3),
        (ExitFlag::SignChangeNotRoot, -5),
    ];
    let msgs: std::collections::HashSet<&str> = flags.iter().map(|f| f.0.message()).collect();
    assert_eq!(msgs.len(), flags.len());
    for (f, c) in flags {
        assert_eq!(f.code(), c);
    }
    let o = Options {
        output: Some(Box::new(|_| Control::Continue)),
        ..opts()
    };
    let d = format!("{o:?}");
    assert!(d.contains("<função>") && d.contains("tol_x"), "{d}");
    let e = OptimError::NoSignChange {
        interval: (1.0, 2.0),
        values: (3.0, 4.0),
    };
    assert!(e.to_string().contains('2') && e.to_string().contains('4'));
}

/// NNLS com colunas quase colineares e tolerância zero: os multiplicadores das colunas
/// redundantes são só arredondamento, e o método precisa rejeitá-las sem ciclar. O resultado
/// continua viável, com resíduo mínimo (comparado à solução de mínimos quadrados irrestrita,
/// que aqui já é ≥ 0).
#[test]
fn nnls_colunas_quase_colineares() {
    let mut g = chimalab_math::random::Rng::new(chimalab_math::random::Algorithm::Philox, 51);
    for _ in 0..200 {
        let m = 6;
        let base: Vec<f64> = (0..m).map(|_| g.uniform()).collect();
        let a = Matrix::from_fn(m, 4, |i, j| {
            base[i] * (1.0 + 1e-15 * j as f64) + if j == 3 { g.uniform() } else { 0.0 }
        });
        let b: Vec<f64> = (0..m).map(|i| 2.0 * base[i] + 0.1 * g.uniform()).collect();
        let mut o = Options {
            tol_x: Some(0.0),
            ..opts()
        };
        let r = nnls(&a, &b, &mut o).unwrap();
        assert!(r.x.iter().all(|&v| v >= 0.0));
        assert_eq!(r.exit, ExitFlag::Converged);
        // Resíduo não maior que o de usar só a primeira coluna (viável e ótima no subespaço).
        let c: f64 = (0..m).map(|i| a.get(i, 0) * b[i]).sum::<f64>()
            / (0..m).map(|i| a.get(i, 0).powi(2)).sum::<f64>();
        let r1: f64 = (0..m).map(|i| (b[i] - c.max(0.0) * a.get(i, 0)).powi(2)).sum();
        assert!(r.resnorm <= r1 * (1.0 + 1e-12) + 1e-28);
    }
}

/// Caminho de Brent em funções variadas: mínimo perto da borda, assimetrias, oscilação e
/// funções não suaves. Avaliações exatas fixam o caminho (parábola aceita ou rejeitada a cada
/// passo, inclusive quando ela sairia do intervalo).
#[test]
fn fminbnd_caminho_de_brent() {
    // (nome, f, a, b, x do mínimo, avaliações)
    type Caso = (&'static str, fn(f64) -> f64, f64, f64, f64, usize);
    let casos: [Caso; 12] = [
        ("borda_dir", |x| (x - 0.999).powi(2), 0.0, 1.0, 0.999, 16),
        ("borda_esq", |x| (x - 0.001).powi(2), 0.0, 1.0, 0.001, 16),
        ("inverso", |x| 1.0 / x + x, 0.01, 10.0, 1.0, 17),
        (
            "oscila",
            |x| (x - 3.0).powi(2) + 0.1 * sin_det(20.0 * x),
            0.0,
            5.0,
            3.060_048_599_941_315,
            15,
        ),
        (
            "exp_linear",
            |x| exp_det(x) - 5.0 * x,
            -10.0,
            10.0,
            1.609_437_912_434_100_3,
            16,
        ),
        ("raiz_abs", |x| (x - 1.0).abs().sqrt(), 0.0, 4.0, 1.0, 35),
        (
            "assimetrica",
            |x| {
                if x < 1.0 {
                    10.0 * (1.0 - x)
                } else {
                    (x - 1.0).powi(2)
                }
            },
            -3.0,
            4.0,
            1.0,
            9,
        ),
        ("quartica_plana", |x| (x - 0.3).powi(4), -1.0, 2.0, 0.3, 17),
        // Casos achados por busca aleatória em que a parábola sairia do intervalo pelo lado b
        // ou em que x cai exatamente no ponto médio (mudam o caminho se o teste de fronteira
        // ou o desempate x < m forem alterados).
        ("exp_a", |x| exp_det(-0.9 * x) + 0.9 * x, -1.7, 0.1, 0.0, 24),
        ("exp_b", |x| exp_det(x) - 1.1 * x, -2.0, 1.5, 1.1f64.ln(), 24),
        (
            "seno_borda",
            |x| sin_det(-2.6 * x + 1.0) + 2.2 * x * x,
            -2.0,
            -0.4,
            -0.4,
            42,
        ),
        ("exp_c", |x| exp_det(-0.3 * x) + 0.3 * x, -1.8, 0.4, 0.0, 23),
    ];
    for (nome, f, a, b, xmin, aval) in casos {
        let r = fminbnd(f, a, b, &mut opts()).unwrap();
        assert_eq!(r.exit, ExitFlag::Converged, "{nome}");
        assert_eq!(r.fun_evals, aval, "{nome}");
        // Quártica: f ~ (Δx)⁴, logo Δx ~ ε^¼ na prática; demais: Δx ~ √ε (oscila: mínimo local
        // conferido a 1e-7, raiz de 2(x − 3) + 2·cos(20x) = 0; exp_linear: ln 5).
        let tol = if nome == "quartica_plana" { 1e-4 } else { 1e-7 };
        assert!((r.x - xmin).abs() <= tol, "{nome}: {}", r.x);
        assert!(r.x >= a && r.x <= b);
    }
}

/// Caminho exato do Brent em `fzero` (mutação). Funções sem libm nem `powi` (polinômios por
/// produtos explícitos, racionais e √, todas corretamente arredondadas em IEEE 754; `powi` varia
/// entre plataformas e mudou o caminho no Windows), geradas por um LCG fixo: a sequência de
/// procedimentos (interpolação ou bissecção), as avaliações e os bits da raiz de cada caso entram
/// numa impressão digital FNV-1a. Com a interpolação quebrada a bissecção ainda converge, então só
/// o caminho denuncia o defeito. A raiz também é conferida pelo intervalo final.
#[test]
fn fzero_caminho_de_brent() {
    let mut s = 0x2545_f491_4f6c_dd1du64;
    let mut u = || {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mistura = |v: u64| {
        for b in v.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    };
    let (mut total, mut interp) = (0usize, 0usize);
    for caso in 0..3000 {
        let r = 4.0 * u() - 2.0;
        let (c1, c2, k) = (2.0 * u() - 1.0, 0.1 + 2.0 * u(), 0.1 + 5.0 * u());
        let f: Box<dyn Fn(f64) -> f64> = match caso % 6 {
            0 => Box::new(move |x| (x - r) * (x * x + c1 * x + c1 * c1 + c2)),
            1 => Box::new(move |x| (x - r) / (1.0 + k * x * x)),
            2 => Box::new(move |x| {
                let d = x - r;
                d * d * d * (1.0 + c2 * x * x)
            }),
            3 => Box::new(move |x| {
                let d = x - r;
                d * d * d * d * d + c2 * d
            }),
            4 => Box::new(move |x| {
                let d = x - r;
                (x + 5.0).sqrt() - (r + 5.0).sqrt() + c1 * d * d * d
            }),
            _ => Box::new(move |x| k * (x - r) + 1.0 / (x + 6.0) - 1.0 / (r + 6.0)),
        };
        let (a, b) = (r - 0.01 - 2.9 * u(), r + 0.01 + 2.9 * u());
        if (f(a) > 0.0) == (f(b) > 0.0) {
            continue; // número par de raízes no intervalo (famílias 0 e 4)
        }
        let tol_x = if caso % 4 == 3 { Some(1e-6) } else { None };
        let mut o = Options {
            tol_x,
            history: true,
            zero_method: ZeroMethod::Brent,
            ..opts()
        };
        let res = fzero(&*f, Start::Interval(a, b), &mut o).unwrap();
        assert_eq!(res.exit, ExitFlag::Converged, "caso {caso}");
        // Raiz: intervalo final com mudança de sinal e largura no limite pedido (algumas famílias
        // têm mais de uma raiz no intervalo inicial; qualquer uma serve).
        let (lo, hi) = res.interval.unwrap();
        let larg = 4.0 * (2.0 * EPS * res.x.abs() + 0.5 * tol_x.unwrap_or(0.0)) + f64::MIN_POSITIVE;
        assert!(
            res.fval == 0.0 || (f(lo) > 0.0) != (f(hi) > 0.0),
            "caso {caso}: sem troca de sinal"
        );
        assert!(
            res.fval == 0.0 || hi - lo <= larg,
            "caso {caso}: largura {}",
            hi - lo
        );
        mistura(res.fun_evals as u64);
        mistura(res.x.to_bits());
        for it in &res.history {
            let p = matches!(it.procedure, Procedure::Interpolation);
            interp += p as usize;
            mistura(p as u64);
            let (lo, hi) = it.interval.unwrap();
            mistura(lo.to_bits() ^ hi.to_bits().rotate_left(32));
        }
        total += res.fun_evals;
    }
    println!("brent: {total} avaliações, {interp} passos de interpolação, impressão {h:#018x}");
    assert_eq!((total, interp, h), (103_476, 74_612, 0x3c23_7e68_7b38_f568));
    // Zero exato pela secante: f(½) = 0 na primeira interpolação, e o intervalo informado é o
    // último com troca de sinal estrita.
    for (a, b, iv, n) in [
        (0.0, 1.0, (0.5, 0.501_968_383_789_062_5), 11),
        (0.25, 3.0, (0.5, 3.0), 3),
        (-1.0, 2.0, (0.5, 2.0), 4),
    ] {
        let mut o = Options {
            history: true,
            zero_method: ZeroMethod::Brent,
            ..opts()
        };
        let res = fzero(|x| x - 0.5, Start::Interval(a, b), &mut o).unwrap();
        assert_eq!(
            (res.x, res.fval, res.interval, res.fun_evals),
            (0.5, 0.0, Some(iv), n),
            "({a}, {b})"
        );
    }
}
