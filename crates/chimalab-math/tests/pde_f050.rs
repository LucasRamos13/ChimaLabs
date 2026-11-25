//! F050 — EDP 1-D pelo método das linhas: soluções exatas e convergência de ordem 2.

use chimalab_math::ode::{OdeStatus, pde1};
use std::f64::consts::PI;

fn malha(a: f64, b: f64, n: usize) -> Vec<f64> {
    (0..n).map(|k| a + (b - a) * k as f64 / (n - 1) as f64).collect()
}

/// Calor: u_t = u_xx, u(0) = u(1) = 0, u₀ = sen πx ⇒ u = e^{−π²t}·sen πx. Erro O(h²).
#[test]
fn calor_dirichlet_ordem_2() {
    let mut erros = Vec::new();
    for n in [11, 21, 41] {
        let x = malha(0.0, 1.0, n);
        let (u, st) = pde1(
            0,
            &|_, _, _, ux| (1.0, ux, 0.0),
            &|x| (PI * x).sin(),
            &|_, ua, _, ub, _| (ua, 0.0, ub, 0.0),
            &x,
            &[0.0, 0.1, 0.5],
            1e-9,
        )
        .unwrap();
        assert_eq!(st, OdeStatus::Completed);
        let mut e = 0.0f64;
        for (k, &t) in [0.1, 0.5].iter().enumerate() {
            for (i, xi) in x.iter().enumerate() {
                e = e.max((u[k + 1][i] - (-PI * PI * t).exp() * (PI * xi).sin()).abs());
            }
        }
        erros.push(e);
    }
    println!("PDE calor: {erros:?}");
    // Ordem 2: o erro cai ~4× ao dobrar a resolução.
    for w in erros.windows(2) {
        let razao = w[0] / w[1];
        assert!(razao > 3.5 && razao < 4.5, "{erros:?}");
    }
    assert!(erros[2] < 2e-4);
}

/// Neumann à direita (fluxo nulo): u_t = u_xx, u(0) = 0, u_x(1) = 0, u₀ = sen(πx/2) ⇒
/// u = e^{−π²t/4}·sen(πx/2).
#[test]
fn calor_neumann() {
    let x = malha(0.0, 1.0, 41);
    let (u, _) = pde1(
        0,
        &|_, _, _, ux| (1.0, ux, 0.0),
        &|x| (PI * x / 2.0).sin(),
        // Em b: p + q·f = 0 com p = 0, q = 1 ⇒ fluxo f = uₓ = 0.
        &|_, ua, _, _, _| (ua, 0.0, 0.0, 1.0),
        &x,
        &[0.0, 0.3],
        1e-9,
    )
    .unwrap();
    for (i, xi) in x.iter().enumerate() {
        let ex = (-PI * PI * 0.3 / 4.0).exp() * (PI * xi / 2.0).sin();
        assert!((u[1][i] - ex).abs() < 5e-4, "{xi}: {} vs {ex}", u[1][i]);
    }
}

/// Esfera (m = 2): u_t = r^{−2}(r² u_r)_r, u(1) = 0, simetria em r = 0, u₀ = sen(πr)/(πr) ⇒
/// u = e^{−π²t}·sen(πr)/(πr).
#[test]
fn calor_esferico() {
    let mut erros = Vec::new();
    for n in [21, 41] {
        let x = malha(0.0, 1.0, n);
        let j0 = |r: f64| if r == 0.0 { 1.0 } else { (PI * r).sin() / (PI * r) };
        let (u, st) = pde1(
            2,
            &|_, _, _, ux| (1.0, ux, 0.0),
            &j0,
            &|_, _, _, ub, _| (0.0, 1.0, ub, 0.0),
            &x,
            &[0.0, 0.1],
            1e-9,
        )
        .unwrap();
        assert_eq!(st, OdeStatus::Completed);
        let e = x
            .iter()
            .enumerate()
            .map(|(i, &r)| (u[1][i] - (-PI * PI * 0.1).exp() * j0(r)).abs())
            .fold(0.0, f64::max);
        erros.push(e);
    }
    println!("PDE esfera: {erros:?}");
    // Medido: razão 2,75 ao dobrar a malha (ordem ~1,5). O ponto médio aritmético no
    // primeiro intervalo junto da origem reduz a ordem ali; a forma exata de Skeel–Berzins na
    // origem fica para um incremento seguinte (limitação documentada em F050.md).
    assert!(erros[1] < erros[0] / 2.5, "{erros:?}");
    assert!(erros[1] < 2e-3);
}

#[test]
fn pde_erros() {
    let p = |_: f64, _: f64, _: f64, ux: f64| (1.0, ux, 0.0);
    let ic = |_: f64| 0.0;
    let bc = |_: f64, ua: f64, _: f64, ub: f64, _: f64| (ua, 0.0, ub, 0.0);
    assert_eq!(
        pde1(3, &p, &ic, &bc, &[0.0, 0.5, 1.0], &[0.0, 1.0], 1e-6)
            .unwrap_err()
            .id(),
        "chimalab:ode:opcaoInvalida"
    );
    assert_eq!(
        pde1(0, &p, &ic, &bc, &[0.0, 1.0], &[0.0, 1.0], 1e-6)
            .unwrap_err()
            .id(),
        "chimalab:ode:intervaloInvalido"
    );
    assert_eq!(
        pde1(0, &p, &ic, &bc, &[0.0, 1.0, 0.5], &[0.0, 1.0], 1e-6)
            .unwrap_err()
            .id(),
        "chimalab:ode:intervaloInvalido"
    );
    assert_eq!(
        pde1(1, &p, &ic, &bc, &[-1.0, 0.0, 1.0], &[0.0, 1.0], 1e-6)
            .unwrap_err()
            .id(),
        "chimalab:ode:opcaoInvalida"
    );
    // Cilindro com origem e fonte constante: u_t = r^{−1}(r u_r)_r + 4, u(1) = 0: estado
    // estacionário u = 1 − r².
    let x = malha(0.0, 1.0, 41);
    let (u, _) = pde1(
        1,
        &|_, _, _, ux| (1.0, ux, 4.0),
        &|_| 0.0,
        &|_, _, _, ub, _| (0.0, 1.0, ub, 0.0),
        &x,
        &[0.0, 5.0],
        1e-8,
    )
    .unwrap();
    for (i, r) in x.iter().enumerate() {
        assert!((u[1][i] - (1.0 - r * r)).abs() < 2e-3, "{r}: {}", u[1][i]);
    }
}

/// Solução manufaturada (mutação): u = e^{−t}·cos x + x², com c = 1 + x, fluxo f = k·uₓ,
/// k = 1 + x², e o termo-fonte s que faz u ser solução exata de
/// c·u_t = x^{−m}·∂ₓ(x^m·f) + s, nas três geometrias. Contornos: Robin à direita
/// (p = 2(u − u_exato) − f_exato, q = 1); à esquerda, Dirichlet não linear u³ = u_exato³ com
/// condição inicial inconsistente (corrigida pela secante) em m = 0, e simetria f = 0 em x = 0
/// para m = 1, 2. Erro O(h²), conferido em duas malhas.
#[test]
fn solucao_manufaturada_tres_geometrias() {
    let ue = |x: f64, t: f64| (-t).exp() * x.cos() + x * x;
    let ux = |x: f64, t: f64| -(-t).exp() * x.sin() + 2.0 * x;
    // uₓ/x, com o limite em x = 0.
    let ux_x = |x: f64, t: f64| {
        let sinc = if x == 0.0 { 1.0 } else { x.sin() / x };
        -(-t).exp() * sinc + 2.0
    };
    let k = |x: f64| 1.0 + x * x;
    for m in 0..=2u32 {
        let fonte = move |x: f64, t: f64| {
            let (c, ut) = (1.0 + x, -(-t).exp() * x.cos());
            let uxx = -(-t).exp() * x.cos() + 2.0;
            let div = 2.0 * x * ux(x, t) + k(x) * uxx + f64::from(m) * k(x) * ux_x(x, t);
            c * ut - div
        };
        let pde = move |x: f64, t: f64, _u: f64, d: f64| (1.0 + x, k(x) * d, fonte(x, t));
        let (a, b) = if m == 0 { (0.2, 1.3) } else { (0.0, 1.0) };
        let bc = move |_xa: f64, ua: f64, _xb: f64, ub: f64, t: f64| {
            let (pa, qa) = if m == 0 {
                (ua.powi(3) - ue(a, t).powi(3), 0.0)
            } else {
                (0.0, 1.0)
            };
            (pa, qa, 2.0 * (ub - ue(b, t)) - k(b) * ux(b, t), 1.0)
        };
        // Condição inicial com o extremo de Dirichlet errado em 0,3 (a secante corrige).
        let ic = move |x: f64| ue(x, 0.0) + if m == 0 && x == a { 0.3 } else { 0.0 };
        let mut erros = Vec::new();
        for n in [21, 41] {
            let x = malha(a, b, n);
            let tempos = [0.0, 0.5, 1.0];
            let (u, st) = pde1(m, &pde, &ic, &bc, &x, &tempos, 1e-9).unwrap();
            assert_eq!(st, OdeStatus::Completed, "m = {m}");
            if m == 0 {
                assert!((u[0][0] - ue(a, 0.0)).abs() <= 1e-12, "IC corrigida: {}", u[0][0]);
            }
            let mut e = 0.0f64;
            for (kk, &t) in tempos.iter().enumerate().skip(1) {
                for (i, &xi) in x.iter().enumerate() {
                    e = e.max((u[kk][i] - ue(xi, t)).abs());
                }
            }
            erros.push(e);
        }
        assert!(erros[1] <= 2e-3, "m = {m}: {erros:?}");
        assert!(erros[0] / erros[1] > 3.0, "m = {m}: ordem < 2: {erros:?}");
    }
}
