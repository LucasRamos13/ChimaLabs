//! F042: derivadas de Airy. Oráculos: tests/dados/F042/airy_derivadas.csv (séries de Maclaurin
//! em decimal de 100 dígitos, ferramentas/oraculos/f042_airy_derivadas.py, validado contra as
//! referências de Ai/Bi do mpmath), formas fechadas em 0, Wronskiano (DLMF 9.2.7), equação de
//! Airy y″ = x·y e assintótica (DLMF 9.7.6, 9.7.10).

use chimalab_math::special::*;
use std::f64::consts::PI;

fn referencias() -> Vec<(f64, f64, f64)> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/airy_derivadas.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with('x'))
        .map(|l| {
            let c: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
            (c[0], c[1], c[2])
        })
        .collect()
}

#[test]
fn derivadas_contra_referencias_de_alta_precisao() {
    // Erro relativo; na região oscilatória (x < −1), contra a amplitude local |x|^(1/4)/√π
    // (DLMF 9.7.10), porque perto dos zeros o erro relativo não tem significado.
    let refs = referencias();
    assert!(refs.len() >= 80);
    let (mut pior_a, mut pior_b): (f64, f64) = (0.0, 0.0);
    for (x, ra, rb) in refs {
        let (a, b) = airy_prime(x);
        let amp = if x < -1.0 {
            x.abs().powf(0.25) / PI.sqrt()
        } else {
            0.0
        };
        let ea = (a - ra).abs() / ra.abs().max(amp);
        let eb = (b - rb).abs() / rb.abs().max(amp);
        pior_a = pior_a.max(ea);
        pior_b = pior_b.max(eb);
        assert!(
            ea < 1e-14 && eb < 1e-14,
            "x = {x}: Ai′ {a:e} × {ra:e}; Bi′ {b:e} × {rb:e}"
        );
    }
    println!("pior erro: Ai′ {pior_a:e}, Bi′ {pior_b:e}");
}

#[test]
fn valores_em_zero() {
    // Ai′(0) = −3^(−1/3)/Γ(1/3), Bi′(0) = 3^(1/6)/Γ(1/3) (DLMF 9.2.4, 9.2.6).
    assert_eq!(
        airy_prime(0.0),
        (-0.258_819_403_792_806_8, 0.448_288_357_353_826_4)
    );
    // Regressão: Ai(0) e −Ai′(0) vinham de gamma(1/3), com erro ~2e-15.
    assert_eq!(airy(0.0).0, 0.355_028_053_887_817_2);
}

#[test]
fn wronskiano() {
    // Ai·Bi′ − Ai′·Bi = 1/π (DLMF 9.2.7). Erro relativo à soma dos módulos dos dois produtos.
    for i in -4000..=600 {
        let x = f64::from(i) * 0.01;
        let ((ai, bi), (ap, bp)) = (airy(x), airy_prime(x));
        let w = ai * bp - ap * bi;
        let escala = (ai * bp).abs() + (ap * bi).abs();
        assert!((w - 1.0 / PI).abs() <= 1e-14 * escala, "x = {x}: {w}");
    }
}

#[test]
fn equacao_de_airy_e_consistencia_com_airy() {
    // Diferenças centrais de quarta ordem: Ai′ ≈ d Ai/dx e d Ai′/dx = x·Ai (o mesmo para Bi).
    let h = 1e-3;
    let d = |f: &dyn Fn(f64) -> f64, x: f64| {
        (8.0 * (f(x + h) - f(x - h)) - (f(x + 2.0 * h) - f(x - 2.0 * h))) / (12.0 * h)
    };
    for x in [-15.3, -3.0, -1.0001, -0.9999, 0.5, 0.9999, 1.0001, 3.0, 8.0] {
        let (ap, bp) = airy_prime(x);
        let (ai, bi) = airy(x);
        let tol = 1e-10;
        assert!(
            (d(&|t| airy(t).0, x) - ap).abs() <= tol * ap.abs().max(1e-3),
            "Ai′({x})"
        );
        assert!(
            (d(&|t| airy(t).1, x) - bp).abs() <= tol * bp.abs().max(1e-3),
            "Bi′({x})"
        );
        assert!(
            (d(&|t| airy_prime(t).0, x) - x * ai).abs() <= tol * (x * ai).abs().max(1e-3),
            "Ai″({x})"
        );
        assert!(
            (d(&|t| airy_prime(t).1, x) - x * bi).abs() <= tol * (x * bi).abs().max(1e-3),
            "Bi″({x})"
        );
    }
}

#[test]
fn assintotica_para_x_grande() {
    // DLMF 9.7.6 e 9.7.8, com ζ = (2/3)x^(3/2), v₁ = −7/72, v₂ = −455/10368:
    // Ai′(x) ~ −x^(1/4)·e^(−ζ)/(2√π)·(1 − v₁/ζ + v₂/ζ²), Bi′(x) ~ x^(1/4)·e^ζ/√π·(1 + v₁/ζ + v₂/ζ²).
    // Em x = 40, ζ ≈ 168,6 e o termo seguinte (~0,06/ζ³) fica perto de 1e-8.
    let (v1, v2) = (-7.0 / 72.0, -455.0 / 10368.0);
    let x: f64 = 40.0;
    let z = 2.0 / 3.0 * x.powf(1.5);
    let (ap, bp) = airy_prime(x);
    let want_a = -x.powf(0.25) * (-z).exp() / (2.0 * PI.sqrt()) * (1.0 - v1 / z + v2 / (z * z));
    let want_b = x.powf(0.25) * z.exp() / PI.sqrt() * (1.0 + v1 / z + v2 / (z * z));
    assert!(((ap - want_a) / want_a).abs() < 5e-8, "{ap:e} × {want_a:e}");
    assert!(((bp - want_b) / want_b).abs() < 5e-8, "{bp:e} × {want_b:e}");
}

#[test]
fn casos_de_borda() {
    let (a, b) = airy_prime(f64::NAN);
    assert!(a.is_nan() && b.is_nan());
    // Regressão: em ±Inf, airy e airy_prime davam NaN mesmo onde o limite existe.
    assert_eq!(airy(f64::INFINITY), (0.0, f64::INFINITY));
    assert_eq!(airy(f64::NEG_INFINITY), (0.0, 0.0));
    let (a, b) = airy_prime(f64::INFINITY);
    assert_eq!((a, a.is_sign_negative(), b), (0.0, true, f64::INFINITY));
    let (a, b) = airy_prime(f64::NEG_INFINITY);
    assert!(a.is_nan() && b.is_nan());
    // x grande: Ai′ subestoura para −0, Bi′ estoura.
    let (a, b) = airy_prime(200.0);
    assert_eq!((a, a.is_sign_negative(), b), (0.0, true, f64::INFINITY));
    // Subnormal: Ai′(x) = Ai′(0) + O(x²).
    assert_eq!(airy_prime(5e-324), airy_prime(0.0));
}
