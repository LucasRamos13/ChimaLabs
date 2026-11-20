//! F042: gama com argumento complexo. Oráculos: tests/dados/F042/gama_complexa.csv (deslocamento
//! e Stirling em decimal de 100 dígitos, ferramentas/oraculos/f042_gama_complexa.py, validado por
//! identidades e por math.lgamma; independente da reflexão usada na implementação), além de
//! identidades de módulo (DLMF 5.4.3–4), reflexão (5.5.3), recorrência (5.5.1) e duplicação
//! (5.5.5).

use chimalab_math::special::*;
use num_complex::Complex64;
use std::f64::consts::PI;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

struct Ref {
    z: Complex64,
    lng: Complex64,
    g: Complex64,
}

fn referencias() -> Vec<Ref> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/gama_complexa.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("re"))
        .map(|l| {
            let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
            Ref {
                z: c(v[0], v[1]),
                lng: c(v[2], v[3]),
                g: c(v[4], v[5]),
            }
        })
        .collect()
}

/// Erro relativo na norma complexa.
fn erro(got: Complex64, want: Complex64) -> f64 {
    if got == want {
        return 0.0;
    }
    (got - want).norm() / want.norm()
}

#[test]
fn lngamma_e_gamma_contra_referencias() {
    let refs = referencias();
    assert!(refs.len() >= 250);
    let (mut pior_l, mut pior_g) = (0.0f64, 0.0f64);
    let (mut zl, mut zg) = (c(0.0, 0.0), c(0.0, 0.0));
    for r in &refs {
        let el = erro(gammaln_complex(r.z), r.lng);
        // exp amplifica o erro absoluto de ln Γ: normaliza por max(1, |ln Γ|).
        let eg = erro(gamma_complex(r.z), r.g) / r.lng.norm().max(1.0);
        if el > pior_l {
            (pior_l, zl) = (el, r.z);
        }
        if eg > pior_g {
            (pior_g, zg) = (eg, r.z);
        }
    }
    eprintln!("pior ln Γ: {pior_l:e} em {zl}; pior Γ: {pior_g:e} em {zg}");
    assert!(pior_l < 8e-15, "ln Γ: {pior_l:e} em {zl}");
    assert!(pior_g < 1e-14, "Γ: {pior_g:e} em {zg}");
}

#[test]
fn eixo_real_coincide_com_a_gama_real() {
    for x in [0.1, 0.5, 1.0, 1.5, 2.0, 7.25, 30.0, 171.5, 1e10] {
        assert_eq!(gammaln_complex(c(x, 0.0)), c(gammaln(x), 0.0), "x = {x}");
        assert_eq!(gamma_complex(c(x, 0.0)), c(gamma(x), 0.0), "x = {x}");
    }
    // Lado negativo: limite por cima do corte, Im = −π⌈−x⌉; Γ continua real.
    for (x, voltas) in [(-0.25, 1.0), (-1.5, 2.0), (-2.75, 3.0), (-120.5, 121.0)] {
        let l = gammaln_complex(c(x, 0.0));
        assert_eq!(l, c(gammaln(x), -PI * voltas), "x = {x}");
        assert_eq!(gamma_complex(c(x, 0.0)), c(gamma(x), 0.0));
        // −0 na parte imaginária também é o limite por cima.
        assert_eq!(gammaln_complex(c(x, -0.0)), l);
        // Continuidade a partir do semiplano superior.
        assert!(erro(gammaln_complex(c(x, 1e-12)), l) < 1e-11, "x = {x}");
    }
}

#[test]
fn casos_de_borda() {
    let nan = |w: Complex64| w.re.is_nan() && w.im.is_nan();
    assert!(nan(gammaln_complex(c(f64::NAN, 1.0))));
    assert!(nan(gammaln_complex(c(1.0, f64::NAN))));
    assert!(nan(gamma_complex(c(1.0, f64::NAN))));
    // Polos: 0 e inteiros negativos.
    for x in [0.0, -0.0, -1.0, -7.0, -1e20] {
        let l = gammaln_complex(c(x, 0.0));
        assert!(l.re == f64::INFINITY && l.im.is_nan(), "x = {x}");
    }
    assert!(gammaln_complex(c(f64::NEG_INFINITY, 0.0)).re == f64::INFINITY);
    assert_eq!(gammaln_complex(c(f64::INFINITY, 0.0)), c(f64::INFINITY, 0.0));
    // Todas as combinações com parte infinita ou NaN fora do eixo real dão NaN + NaN·i.
    let especiais = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        -3.0,
        -2.5,
        0.3,
        1.0,
        2.0,
        3.0,
        25.0,
        1e300,
        -1e300,
        1e-300,
    ];
    for &x in &especiais {
        for &y in &especiais {
            if x.is_finite() && y.is_finite() {
                continue;
            }
            assert!(nan(gammaln_complex(c(x, y))), "z = ({x}, {y})");
        }
    }
    assert!(nan(gammaln_complex(c(f64::NAN, 0.0))));
    // z minúsculo: ln Γ(z) = −ln z − γz + O(z²).
    for z in [c(0.0, 1e-300), c(1e-200, -1e-200), c(-1e-10, 1e-10)] {
        let want = -z.ln() - std::f64::consts::EULER_GAMMA * z;
        assert!(erro(gammaln_complex(z), want) < 1e-15, "z = {z}");
    }
    // Regressão: |sen(πz)|² sofria underflow para |z| < 1e-154 e ln Γ dava Inf.
    assert!(gammaln_complex(c(-1e-160, 1e-160)).re.is_finite());
    // Subnormal: Γ(z) ≈ 1/z.
    let z = c(0.0, 5e-324);
    assert!(gamma_complex(z).im.is_infinite() || erro(gamma_complex(z), z.inv()) < 1e-15);
    // |z| enorme: termo dominante de Stirling.
    for z in [c(1e10, 1e10), c(-1e8, 3e8), c(5.0, -1e12)] {
        let want = (z - 0.5) * z.ln() - z + 0.5 * (2.0 * PI).ln();
        assert!(erro(gammaln_complex(z), want) < 1e-14, "z = {z}");
    }
    // Γ transborda e some sem NaN.
    let g = gamma_complex(c(200.0, 1.0));
    assert!(g.norm().is_infinite());
    // |Γ(−½ + 600i)| ≈ √(2π)·600⁻¹·e^(−300π) < 1e-400: underflow para zero.
    assert_eq!(gamma_complex(c(-0.5, 600.0)), c(0.0, 0.0));
}

#[test]
fn modulos_conhecidos() {
    // |Γ(½+iy)|² = π/cosh(πy), |Γ(iy)|² = π/(y senh(πy)), |Γ(1+iy)|² = πy/senh(πy).
    for y in [1e-6, 0.3, 1.0, 4.5, 15.0, 19.99, 20.01, 60.0] {
        let m1 = gammaln_complex(c(0.5, y)).re;
        let w1 = 0.5 * (PI.ln() - (PI * y).cosh().ln());
        assert!((m1 - w1).abs() <= 4e-15 * w1.abs().max(1.0), "y = {y}: {m1} {w1}");
        let m2 = gammaln_complex(c(0.0, -y)).re;
        let w2 = 0.5 * (PI.ln() - y.ln() - (PI * y).sinh().ln());
        assert!((m2 - w2).abs() <= 2e-15 * w2.abs().max(1.0), "y = {y}: {m2} {w2}");
        // −½ ln(senh(t)/t), t = πy; série para t pequeno (a forma direta cancela).
        let t = PI * y;
        let r = if t < 1e-3 {
            t * t / 6.0 + t.powi(4) / 120.0
        } else {
            t.sinh() / t - 1.0
        };
        let m3 = gammaln_complex(c(1.0, y)).re;
        let w3 = -0.5 * r.ln_1p();
        let tol = if t < 1e-3 || t > 1.0 {
            2e-15
        } else {
            1e-14 / t.powi(2)
        };
        assert!((m3 - w3).abs() <= tol * w3.abs(), "y = {y}: {m3} {w3}");
    }
}

#[test]
fn continuidade_nas_trocas_de_metodo() {
    // Re z = ½ (reflexão ↔ Stirling), Im z = 20 (ln sen exato ↔ assintótico), discos de Taylor.
    let pares = [
        (c(0.5 - 1e-15, 0.7), c(0.5, 0.7)),
        (c(0.5 - 1e-15, -9.0), c(0.5, -9.0)),
        (c(-3.3, 20.0), c(-3.3, 20.0f64.next_up())),
        (c(-3.3, -20.0), c(-3.3, -(20.0f64.next_up()))),
        (c(1.5 - 1e-15, 0.0001), c(1.5 + 1e-15, 0.0001)),
        (c(2.5 - 1e-15, 0.0001), c(2.5 + 1e-15, 0.0001)),
        (c(1.0, 0.5 - 1e-15), c(1.0, 0.5 + 1e-15)),
    ];
    for (a, b) in pares {
        let (la, lb) = (gammaln_complex(a), gammaln_complex(b));
        assert!(erro(la, lb) < 1e-14, "{a} → {la}, {b} → {lb}");
    }
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn ponto() -> impl Strategy<Value = Complex64> {
        (-40.0f64..40.0, -40.0f64..40.0)
            .prop_filter("longe dos polos", |(x, y)| y.abs() > 1e-3 || *x > 0.0)
            .prop_map(|(x, y)| c(x, y))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(400))]

        #[test]
        fn conjugacao_exata(z in ponto()) {
            prop_assert_eq!(gammaln_complex(z.conj()), gammaln_complex(z).conj());
        }

        // Ramo principal: a recorrência vale sem múltiplos de 2πi (DLMF 5.5.1).
        #[test]
        fn recorrencia_no_ramo_principal(z in ponto()) {
            let lhs = gammaln_complex(z + 1.0);
            let rhs = gammaln_complex(z) + z.ln();
            let escala = lhs.norm().max(gammaln_complex(z).norm()).max(z.ln().norm()).max(1.0);
            prop_assert!((lhs - rhs).norm() <= 2e-14 * escala, "z = {}: {} vs {}", z, lhs, rhs);
        }

        // Γ(z)Γ(1−z) = π / sen(πz) (DLMF 5.5.3).
        #[test]
        fn reflexao(z in (-8.0f64..8.0, 0.01f64..8.0).prop_map(|(x, y)| c(x, y))) {
            let lhs = gamma_complex(z) * gamma_complex(1.0 - z);
            let rhs = PI / (z * PI).sin();
            let l = gammaln_complex(z).norm().max(gammaln_complex(1.0 - z).norm()).max(1.0);
            prop_assert!(erro(lhs, rhs) <= 4e-15 * l, "z = {}", z);
        }

        // Γ(2z) = 2^(2z−1) Γ(z) Γ(z+½) / √π (DLMF 5.5.5), em ln: diferença múltipla de 2πi.
        #[test]
        fn duplicacao(z in ponto()) {
            let lhs = gammaln_complex(2.0 * z);
            let rhs = (2.0 * z - 1.0) * std::f64::consts::LN_2 + gammaln_complex(z)
                + gammaln_complex(z + 0.5) - 0.5 * PI.ln();
            let d = lhs - rhs;
            let voltas = (d.im / (2.0 * PI)).round();
            let escala = lhs.norm().max(rhs.norm()).max(1.0);
            prop_assert!((d - c(0.0, 2.0 * PI * voltas)).norm() <= 2e-14 * escala, "z = {}", z);
        }
    }
}
