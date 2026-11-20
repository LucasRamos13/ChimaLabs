//! Testes F042 (incremento 1). Oráculo: valores do mpmath a 50 dígitos em
//! tests/dados/F042/referencias.csv (gerados por ferramentas/oraculos/f042_referencias.py).

use chimalab_math::special::*;
use std::collections::BTreeMap;

fn referencias() -> Vec<(String, Vec<f64>, f64)> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/referencias.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("funcao"))
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            let args = c[1..4]
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap())
                .collect();
            (c[0].to_string(), args, c[4].parse().unwrap())
        })
        .collect()
}

fn avalia(f: &str, a: &[f64]) -> f64 {
    match f {
        "gamma" => gamma(a[0]),
        "gammaln" => gammaln(a[0]),
        "psi" => psi(a[0]),
        "beta" => beta(a[0], a[1]),
        "betaln" => betaln(a[0], a[1]),
        "gammainc_lower" => gammainc(a[0], a[1], false),
        "gammainc_upper" => gammainc(a[0], a[1], true),
        "betainc_lower" => betainc(a[0], a[1], a[2], false),
        "betainc_upper" => betainc(a[0], a[1], a[2], true),
        "erf" => erf(a[0]),
        "erfc" => erfc(a[0]),
        "erfcx" => erfcx(a[0]),
        "erfinv" => erfinv(a[0]),
        "erfcinv" => erfcinv(a[0]),
        "expint" => expint(a[0]),
        "ei" => ei(a[0]),
        "legendre" => legendre(a[0] as usize, a[2], LegendreNorm::Unnormalized).unwrap()[a[1] as usize],
        "ellipk" => ellipke(a[0]).0,
        "ellipe" => ellipke(a[0]).1,
        "ellipj_sn" => ellipj(a[0], a[1]).0,
        "ellipj_cn" => ellipj(a[0], a[1]).1,
        "ellipj_dn" => ellipj(a[0], a[1]).2,
        "besselj" => besselj(a[0], a[1]),
        "bessely" => bessely(a[0], a[1]),
        "besseli_scaled" => besseli(a[0], a[1], true),
        "besselk_scaled" => besselk(a[0], a[1], true),
        "hankel1_scaled_re" => besselh(a[0], HankelKind::First, a[1], true).re,
        "hankel1_scaled_im" => besselh(a[0], HankelKind::First, a[1], true).im,
        "hankel2_scaled_re" => besselh(a[0], HankelKind::Second, a[1], true).re,
        "hankel2_scaled_im" => besselh(a[0], HankelKind::Second, a[1], true).im,
        "airy_ai" => airy(a[0]).0,
        "airy_bi" => airy(a[0]).1,
        _ => panic!("função desconhecida {f}"),
    }
}

/// Erro relativo; para funções com raízes reais (ψ, ln Γ) usa |v − r| / max(|r|, 1), isto é,
/// erro absoluto perto das raízes.
fn erro(f: &str, a: &[f64], v: f64, r: f64) -> f64 {
    if v == r {
        return 0.0;
    }
    let pi = std::f64::consts::PI;
    let escala = match f {
        "psi" | "gammaln" => r.abs().max(1.0),
        // sn e cn oscilam entre −1 e 1: perto dos zeros o erro é absoluto (condicionamento ~ ε·|u|).
        "ellipj_sn" | "ellipj_cn" => r.abs().max(1.0),
        // J, Y e Airy oscilam: na região oscilatória o erro é medido contra a amplitude local.
        "besselj" | "bessely" if a[1] > a[0].abs() => r.abs().max((2.0 / (pi * a[1])).sqrt()),
        "airy_ai" | "airy_bi" if a[0] < -1.0 => r.abs().max(a[0].abs().powf(-0.25) / pi.sqrt()),
        _ => r.abs(),
    };
    if escala == 0.0 {
        v.abs()
    } else {
        ((v - r) / escala).abs()
    }
}

/// Tolerâncias por função (erro relativo máximo aceito).
fn tolerancia(f: &str) -> f64 {
    match f {
        // beta e gammainc: até 2026-11-18 erravam 7,5e-14 e 4,4e-14 (exp de ln B e do prefator em
        // f64); hoje ≤ 2e-15, com o expoente em dupla-dupla e log1pmx.
        "gamma" | "erf" | "erfc" | "erfcx" | "erfinv" | "erfcinv" | "ellipk" | "ellipe" | "beta"
        | "gammainc_lower" | "gammainc_upper" => 1e-14,
        _ => TOL_PADRAO,
    }
}
const TOL_PADRAO: f64 = 1e-13;

#[test]
fn diagnostico_e_tolerancias() {
    let mut pior: BTreeMap<String, (f64, Vec<f64>)> = BTreeMap::new();
    let mut falhas = vec![];
    let refs = referencias();
    // |H| = √(re² + im²) a partir das duas referências do oráculo (mesmos argumentos).
    let modulo_hankel = |f: &str, args: &[f64]| -> f64 {
        let base = &f[..f.len() - 3];
        refs.iter()
            .filter(|(g, a, _)| g.starts_with(base) && g.len() == f.len() && a == args)
            .map(|(_, _, r)| r * r)
            .sum::<f64>()
            .sqrt()
    };
    for (f, args, r) in refs.iter().cloned() {
        let v = avalia(&f, &args);
        // H·e^{∓ix}: as partes real e imaginária podem ser nulas (ex.: ν = ½), então o erro é
        // medido contra o módulo |H|.
        let e = if f.starts_with("hankel") {
            (v - r).abs() / modulo_hankel(&f, &args)
        } else {
            erro(&f, &args, v, r)
        };
        let ent = pior.entry(f.clone()).or_insert((0.0, vec![]));
        if e > ent.0 || e.is_nan() {
            *ent = (e, args.clone());
        }
        if e.is_nan() || e > tolerancia(&f) {
            falhas.push(format!("{f}{args:?}: obtido {v:e}, referência {r:e}, erro {e:e}"));
        }
    }
    for (f, (e, a)) in &pior {
        println!("{f:16} pior erro relativo {e:9.2e} em {a:?}");
    }
    assert!(falhas.is_empty(), "\n{}", falhas.join("\n"));
}

#[test]
fn legendre_em_x_igual_a_1_e_normalizacoes() {
    let p = legendre(3, 1.0, LegendreNorm::Unnormalized).unwrap();
    assert_eq!(p[0], 1.0);
    assert!(p[1..].iter().all(|v| *v == 0.0));
    // Ortonormalidade da forma totalmente normalizada: ∫ N_n^m N_k^m dx = δ_nk (Gauss-Legendre).
    let (nodes, weights) = gauss_legendre(40);
    for m in 0..4 {
        for n in m..8 {
            for k in m..8 {
                let integral: f64 = nodes
                    .iter()
                    .zip(&weights)
                    .map(|(x, w)| {
                        w * legendre(n, *x, LegendreNorm::Full).unwrap()[m]
                            * legendre(k, *x, LegendreNorm::Full).unwrap()[m]
                    })
                    .sum();
                let expected = if n == k { 1.0 } else { 0.0 };
                assert!(
                    (integral - expected).abs() < 1e-12,
                    "m={m} n={n} k={k}: {integral}"
                );
            }
        }
    }
    // Schmidt: S_n^m = √(2(n−m)!/(n+m)!)·P_n^m sem a fase (−1)^m
    let pu = legendre(4, 0.3, LegendreNorm::Unnormalized).unwrap();
    let ps = legendre(4, 0.3, LegendreNorm::Schmidt).unwrap();
    for m in 0..=4usize {
        let ratio = if m == 0 {
            1.0
        } else {
            (2.0 * factorial((4 - m) as f64) / factorial((4 + m) as f64)).sqrt()
        };
        let phase = if m % 2 == 1 { -1.0 } else { 1.0 };
        assert!((ps[m] - phase * ratio * pu[m]).abs() < 1e-15, "m={m}");
    }
    assert_eq!(
        (legendre(2, 1.5, LegendreNorm::Full)).unwrap_err().id(),
        "chimalab:special:dominio"
    );
}

/// Nós e pesos de Gauss-Legendre em [−1, 1] (Newton nas raízes de P_n), só para o teste.
fn gauss_legendre(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = vec![0.0; n];
    let mut w = vec![0.0; n];
    for i in 0..n {
        let mut z = (std::f64::consts::PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
        for _ in 0..100 {
            let (mut p0, mut p1) = (1.0, z);
            for k in 2..=n {
                let p2 = ((2 * k - 1) as f64 * z * p1 - (k - 1) as f64 * p0) / k as f64;
                p0 = p1;
                p1 = p2;
            }
            let dp = n as f64 * (z * p1 - p0) / (z * z - 1.0);
            let dz = p1 / dp;
            z -= dz;
            if dz.abs() < 1e-16 {
                let (mut q0, mut q1) = (1.0, z);
                for k in 2..=n {
                    let q2 = ((2 * k - 1) as f64 * z * q1 - (k - 1) as f64 * q0) / k as f64;
                    q0 = q1;
                    q1 = q2;
                }
                let d = n as f64 * (z * q1 - q0) / (z * z - 1.0);
                x[i] = z;
                w[i] = 2.0 / ((1.0 - z * z) * d * d);
                break;
            }
        }
    }
    (x, w)
}
