//! F042: função erro com argumento complexo e função de Faddeeva. Oráculo:
//! tests/dados/F042/erro_complexo.csv (série de Taylor em decimal de 250 dígitos,
//! ferramentas/oraculos/f042_erro_complexo.py, validado por math.erf/erfc e simetrias;
//! independente da soma trapezoidal e da fração contínua usadas na implementação).

use chimalab_math::special::*;
use num_complex::Complex64;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

struct Ref {
    z: Complex64,
    w: Complex64,
    erf: Complex64,
    erfc: Complex64,
    erfcx: Complex64,
}

fn referencias() -> Vec<Ref> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/erro_complexo.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("re"))
        .map(|l| {
            let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
            Ref {
                z: c(v[0], v[1]),
                w: c(v[2], v[3]),
                erf: c(v[4], v[5]),
                erfc: c(v[6], v[7]),
                erfcx: c(v[8], v[9]),
            }
        })
        .collect()
}

/// (nome, função, valor de referência correspondente)
type Caso = (&'static str, fn(Complex64) -> Complex64, fn(&Ref) -> Complex64);

fn erro(got: Complex64, want: Complex64) -> f64 {
    if got == want {
        return 0.0;
    }
    (got - want).norm() / want.norm()
}

#[test]
fn contra_referencias() {
    let refs = referencias();
    assert!(refs.len() >= 900);
    let fs: [Caso; 4] = [
        ("w", faddeeva, |r| r.w),
        ("erf", erf_complex, |r| r.erf),
        ("erfc", erfc_complex, |r| r.erfc),
        ("erfcx", erfcx_complex, |r| r.erfcx),
    ];
    for (nome, f, alvo) in fs {
        let mut pior = (0.0f64, c(0.0, 0.0));
        let mut pior_bruto = (0.0f64, c(0.0, 0.0));
        for r in &refs {
            let want = alvo(r);
            if !want.norm().is_finite() || want.norm() > 1e300 || want.norm() < 1e-300 {
                continue;
            }
            // Condicionamento: onde a resposta passa por e^(−z²) com fase 2xy grande, o erro de
            // arredondamento de z (½ulp) vira erro relativo ~|z|²ε; normaliza por max(1, |z|²).
            // erf perto dos seus zeros complexos: relativo a max(|erf|, |erfc|).
            let kappa = r.z.norm_sqr().max(1.0);
            let escala = if nome == "erf" {
                want.norm().max(r.erfc.norm())
            } else {
                want.norm()
            };
            let e = (f(r.z) - want).norm() / escala / kappa;
            if e > pior.0 {
                pior = (e, r.z);
            }
            let eb = erro(f(r.z), want);
            if r.z.im >= 0.0 && eb > pior_bruto.0 && nome == "w" {
                pior_bruto = (eb, r.z);
            }
        }
        eprintln!("{nome}: pior normalizado {:e} em {}", pior.0, pior.1);
        assert!(pior.0 < 4e-15, "{nome}: {:e} em {}", pior.0, pior.1);
        if nome == "w" {
            eprintln!(
                "w (Im z ≥ 0, relativo puro): pior {:e} em {}",
                pior_bruto.0, pior_bruto.1
            );
            assert!(pior_bruto.0 < 4e-15);
        }
    }
}

#[test]
fn eixo_real_coincide_com_as_funcoes_reais() {
    for x in [
        -30.0, -5.5, -1.0, -0.25, -0.0, 0.0, 1e-300, 0.3, 2.0, 7.0, 27.0, 1e10,
    ] {
        assert_eq!(erf_complex(c(x, 0.0)), c(erf(x), 0.0), "x = {x}");
        assert_eq!(erfc_complex(c(x, 0.0)), c(erfc(x), 0.0), "x = {x}");
        assert_eq!(erfcx_complex(c(x, 0.0)), c(erfcx(x), 0.0), "x = {x}");
        // Re w(x) = e^(−x²) (DLMF 7.19.1).
        let w = faddeeva(c(x, 0.0));
        assert!((w.re - (-x * x).exp()).abs() <= 2e-16 * w.norm(), "x = {x}: {w}");
    }
    assert_eq!(erf_complex(c(f64::INFINITY, 0.0)), c(1.0, 0.0));
    assert_eq!(erfc_complex(c(f64::NEG_INFINITY, 0.0)), c(2.0, 0.0));
}

#[test]
fn valores_classicos() {
    // w(0) = 1; erf(0) = 0; w(iy) = erfcx(y) (DLMF 7.2.3).
    assert_eq!(faddeeva(c(0.0, 0.0)), c(1.0, 0.0));
    assert_eq!(erf_complex(c(0.0, 1e-30)).re, 0.0);
    for y in [0.1, 1.0, 3.0, 7.9, 8.1, 50.0] {
        let w = faddeeva(c(0.0, y));
        assert!(
            (w.re - erfcx(y)).abs() <= 2e-15 * erfcx(y) && w.im.abs() <= 1e-16 * w.re,
            "y = {y}: {w}"
        );
    }
}

#[test]
fn casos_de_borda() {
    let nan = |w: Complex64| w.re.is_nan() || w.im.is_nan();
    // Regressão: com Re z = NaN, a reflexão de erf e erfc recursava sem fim (estouro de pilha).
    for z in [
        c(f64::NAN, 1.0),
        c(1.0, f64::NAN),
        c(f64::NAN, 0.0),
        c(f64::NAN, f64::NAN),
    ] {
        assert!(nan(faddeeva(z)) && nan(erf_complex(z)) && nan(erfc_complex(z)) && nan(erfcx_complex(z)));
    }
    // z minúsculo e subnormal: w(z) ≈ 1 + 2iz/√π, erf z ≈ 2z/√π.
    let k = std::f64::consts::FRAC_2_SQRT_PI;
    for z in [
        c(1e-10, 1e-10),
        c(-3e-200, 5e-200),
        c(0.0, 5e-324),
        c(5e-324, -5e-324),
    ] {
        assert!(erro(faddeeva(z), 1.0 + c(0.0, k) * z) < 1e-16, "z = {z}");
        assert_eq!(erf_complex(z), k * z, "z = {z}");
    }
    // |z| grande no semiplano superior: w(z) ≈ i/(√π z)·Σₖ (2k−1)!!/(2z²)ᵏ (DLMF 7.12.1), 7 termos
    // (o primeiro omitido é < 1e-19 em |z| ≥ 50).
    for z in [
        c(1e3, 1e3),
        c(-50.0, 30.0),
        c(0.0, 1e300),
        c(1e300, 1e300),
        c(1e200, 0.0),
    ] {
        let zi = z.finv();
        let u = 0.5 * zi * zi;
        let (mut termo, mut serie) = (c(1.0, 0.0), c(1.0, 0.0));
        for j in 1..7 {
            termo *= u * (2 * j - 1) as f64;
            serie += termo;
        }
        let want = c(0.0, 0.5 * k) * zi * serie;
        assert!(
            erro(faddeeva(z), want) < 1e-15,
            "z = {z}: {} vs {want}",
            faddeeva(z)
        );
    }
    // Regressão: a divisão complexa comum transbordava |r|² e dava w(1e300 i) = 0.
    assert!(faddeeva(c(0.0, 1e300)).re > 5e-301);
    // erfc(z) → 0 e erf(z) → 1 para Re z → +∞ com |Im z| < Re z.
    assert_eq!(erfc_complex(c(40.0, 1.0)), c(0.0, 0.0));
    assert_eq!(erf_complex(c(40.0, -1.0)), c(1.0, 0.0));
    assert_eq!(erf_complex(c(-40.0, 1.0)), c(-1.0, 0.0));
    assert_eq!(erfc_complex(c(-40.0, 1.0)), c(2.0, 0.0));
    // Transbordamento legítimo (|erf(x + iy)| ~ e^(y² − x²)): resultado não finito, sem pânico.
    for z in [c(2.0, 30.0), c(-3.0, -30.0), c(0.0, 1e300)] {
        assert!(!erf_complex(z).norm().is_finite(), "z = {z}");
        assert!(!erfc_complex(z).norm().is_finite(), "z = {z}");
    }
    assert!(!faddeeva(c(1.0, -30.0)).norm().is_finite());
}

#[test]
fn continuidade_nas_trocas_de_metodo() {
    let k8 = 8.0 / std::f64::consts::SQRT_2;
    let pares = [
        // |z| = 8: trapézio ↔ fração contínua.
        (c(8.0, 0.0), c(8.0f64.next_up(), 0.0)),
        (c(0.0, 8.0f64.next_down()), c(0.0, 8.0)),
        (c(k8, k8), c(k8.next_up(), k8.next_up())),
        // Troca de malha em |Re z/h − round| = 0,25.
        (c(0.35 * 2.25, 0.3), c((0.35f64 * 2.25).next_up(), 0.3)),
        (c(0.35 * 2.75, 1e-6), c((0.35f64 * 2.75).next_down(), 1e-6)),
    ];
    for (a, b) in pares {
        assert!(
            erro(faddeeva(a), faddeeva(b)) < 4e-15,
            "{a} → {}, {b} → {}",
            faddeeva(a),
            faddeeva(b)
        );
    }
    // erf: Taylor ↔ 1 − erfc em |z| = ½; sinal de Re z em erfc e erfcx.
    let (a, b) = (c(0.3, 0.4), c(0.3f64.next_up(), 0.4f64.next_up()));
    assert!(erro(erf_complex(a), erf_complex(b)) < 4e-15);
    for y in [0.5, 3.0, -2.0] {
        let (a, b) = (c(-1e-300, y), c(1e-300, y));
        assert!(erro(erfc_complex(a), erfc_complex(b)) < 4e-15, "y = {y}");
        assert!(erro(erfcx_complex(a), erfcx_complex(b)) < 4e-15, "y = {y}");
    }
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn ponto() -> impl Strategy<Value = Complex64> {
        (-6.0f64..6.0, -6.0f64..6.0).prop_map(|(x, y)| c(x, y))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        // erf(z̄) = conj(erf z), erf(−z) = −erf z (DLMF 7.4.1, 7.4.2); exatas por construção.
        #[test]
        fn simetrias_de_erf(z in ponto()) {
            prop_assert_eq!(erf_complex(z.conj()), erf_complex(z).conj());
            prop_assert_eq!(erf_complex(-z), -erf_complex(z));
        }

        // w(−z̄) = conj(w(z)) (DLMF 7.4.4), pelo caminho do semiplano superior.
        #[test]
        fn simetria_de_w(z in (-12.0f64..12.0, 0.0f64..12.0).prop_map(|(x, y)| c(x, y))) {
            prop_assert!(erro(faddeeva(-z.conj()), faddeeva(z).conj()) < 4e-15, "z = {}", z);
        }

        // erf + erfc = 1 e erfcx = e^(z²) erfc onde não há cancelamento nem transbordamento.
        #[test]
        fn relacoes_entre_funcoes(z in (-4.0f64..4.0, -3.0f64..3.0).prop_map(|(x, y)| c(x, y))) {
            let s = erf_complex(z) + erfc_complex(z);
            let escala = erf_complex(z).norm().max(erfc_complex(z).norm());
            prop_assert!((s - 1.0).norm() <= 8e-15 * escala, "z = {}", z);
            let ex = (z * z).exp() * erfc_complex(z);
            let kappa = z.norm_sqr().max(1.0);
            prop_assert!(erro(erfcx_complex(z), ex) <= 8e-15 * kappa, "z = {}", z);
        }

        // w′(z) = −2z w(z) + 2i/√π (DLMF 7.10): diferença central com passo 1e-5.
        #[test]
        fn equacao_diferencial(z in (-7.5f64..7.5, 0.0f64..7.5).prop_map(|(x, y)| c(x, y))) {
            let h = 1e-5;
            let d = (faddeeva(z + h) - faddeeva(z - h)) / (2.0 * h);
            let want = -2.0 * z * faddeeva(z) + c(0.0, std::f64::consts::FRAC_2_SQRT_PI);
            prop_assert!((d - want).norm() <= 1e-8 * want.norm().max(1.0), "z = {}", z);
        }
    }
}
