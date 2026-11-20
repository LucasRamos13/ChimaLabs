//! F042: funções de Airy com argumento complexo. Oráculo: tests/dados/F042/airy_complexo.csv
//! (séries de Maclaurin em decimal de 120 dígitos, ferramentas/oraculos/f042_airy_complexo.py,
//! validado pelas referências do mpmath no eixo real, Wronskiano e conjugação; independente das
//! relações com Bessel de ordem 1/3 e 2/3 usadas para |z| > 1).

use chimalab_math::special::*;
use num_complex::Complex64;
use std::f64::consts::PI;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

fn referencias() -> Vec<[Complex64; 5]> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/airy_complexo.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("re"))
        .map(|l| {
            let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
            [
                c(v[0], v[1]),
                c(v[2], v[3]),
                c(v[4], v[5]),
                c(v[6], v[7]),
                c(v[8], v[9]),
            ]
        })
        .collect()
}

fn erro(got: Complex64, want: Complex64, escala: f64) -> f64 {
    if got == want {
        return 0.0;
    }
    (got - want).norm() / escala
}

#[test]
fn contra_referencias() {
    let refs = referencias();
    assert!(refs.len() >= 190);
    let mut pior = [(0.0f64, c(0.0, 0.0)); 4];
    for r in &refs {
        let z = r[0];
        let (ai, bi) = airy_complex(z);
        let (aip, bip) = airy_prime_complex(z);
        // Escala: Ai e Bi formam um par (Wronskiano 1/π); perto dos zeros de um deles só o erro
        // em relação a |Ai| + |Bi| tem significado. Idem para as derivadas.
        let e1 = r[1].norm() + r[2].norm();
        let e2 = r[3].norm() + r[4].norm();
        for (idx, (got, want, esc)) in [(ai, r[1], e1), (bi, r[2], e1), (aip, r[3], e2), (bip, r[4], e2)]
            .into_iter()
            .enumerate()
        {
            let e = erro(got, want, esc);
            if e > pior[idx].0 {
                pior[idx] = (e, z);
            }
        }
    }
    for (nome, (e, z)) in ["Ai", "Bi", "Ai′", "Bi′"].iter().zip(pior) {
        eprintln!("{nome}: pior {e:e} em {z}");
        assert!(e < 1e-14, "{nome}: {e:e} em {z}");
    }
}

#[test]
fn eixo_real_coincide_com_as_funcoes_reais() {
    for x in [
        -30.0,
        -5.5,
        -1.0,
        -0.0,
        0.0,
        0.7,
        1.0,
        3.0,
        25.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let (a, b) = airy(x);
        let (ap, bp) = airy_prime(x);
        let (ca, cb) = airy_complex(c(x, 0.0));
        let (cap, cbp) = airy_prime_complex(c(x, -0.0));
        assert!(
            ca.re.to_bits() == a.to_bits() && cb.re.to_bits() == b.to_bits() && ca.im == 0.0,
            "{x}"
        );
        assert!(
            cap.re.to_bits() == ap.to_bits() && cbp.re.to_bits() == bp.to_bits(),
            "{x}"
        );
    }
}

#[test]
fn casos_de_borda() {
    let nan = |w: Complex64| w.re.is_nan() && w.im.is_nan();
    for z in [
        c(f64::NAN, 1.0),
        c(1.0, f64::NAN),
        c(f64::INFINITY, 1.0),
        c(2.0, f64::NEG_INFINITY),
    ] {
        let (a, b) = airy_complex(z);
        let (ap, bp) = airy_prime_complex(z);
        assert!(nan(a) && nan(b) && nan(ap) && nan(bp), "{z}");
    }
    // z minúsculo: Ai(z) ≈ Ai(0) + Ai′(0)z, Bi(z) ≈ Bi(0) + Bi′(0)z.
    let (a0, b0) = airy(0.0);
    let (ap0, bp0) = airy_prime(0.0);
    for z in [c(1e-300, 1e-300), c(0.0, -1e-20), c(-1e-10, 1e-10)] {
        let (a, b) = airy_complex(z);
        assert!(
            erro(a, a0 + ap0 * z, a0) < 4e-16 && erro(b, b0 + bp0 * z, b0) < 4e-16,
            "{z}"
        );
    }
    // |z| grande: Ai(z) ≈ e^{−ζ}/(2√π z^{1/4}) Σ (−1)ᵏuₖ/ζᵏ (DLMF 9.7.5), ζ = (2/3)z^{3/2},
    // |ph z| < π, uₖ = uₖ₋₁(6k−5)(6k−3)(6k−1)/((2k−1)216k) (DLMF 9.7.2); 8 termos, o omitido é
    // < 1e-16 relativo em |ζ| ≥ 150.
    for z in [c(50.0, 20.0), c(-30.0, 40.0), c(0.0, -60.0)] {
        let zeta = 2.0 / 3.0 * z.powf(1.5);
        let (mut u, mut termo, mut serie) = (1.0, c(1.0, 0.0), c(1.0, 0.0));
        for k in 1..8 {
            let kk = k as f64;
            u *= (6.0 * kk - 5.0) * (6.0 * kk - 3.0) * (6.0 * kk - 1.0) / ((2.0 * kk - 1.0) * 216.0 * kk);
            termo *= -1.0 / zeta;
            serie += u * termo;
        }
        let want = (-zeta).exp() / (2.0 * PI.sqrt() * z.powf(0.25)) * serie;
        let got = airy_complex(z).0;
        // Condicionamento de e^{−ζ}: o arredondamento de z vira erro relativo ~|ζ|ε.
        assert!(
            erro(got, want, want.norm()) < 1e-15 * zeta.norm(),
            "{z}: {got:e} {want:e}"
        );
    }
    // Transbordamento de Bi para |z| grande no setor direito: não finito, sem pânico.
    assert!(!airy_complex(c(150.0, 1.0)).1.norm().is_finite());
}

#[test]
fn continuidade_nas_trocas_de_metodo() {
    // |z| = 1 (Maclaurin ↔ Bessel) e Re z = 0 (K e I ↔ J e Y).
    let pares = [
        (c(0.6, 0.8), c(0.6f64.next_up(), 0.8f64.next_up())),
        (c(-0.8, -0.6), c(-(0.8f64.next_up()), -(0.6f64.next_up()))),
        (c(1e-300, 4.0), c(-1e-300, 4.0)),
        (c(-1e-300, -2.5), c(1e-300, -2.5)),
    ];
    for (a, b) in pares {
        let (fa, fb) = (airy_complex(a), airy_complex(b));
        let (ga, gb) = (airy_prime_complex(a), airy_prime_complex(b));
        let esc = fa.0.norm() + fa.1.norm();
        let escp = ga.0.norm() + ga.1.norm();
        assert!(
            erro(fa.0, fb.0, esc) < 1e-14 && erro(fa.1, fb.1, esc) < 1e-14,
            "{a} {b}"
        );
        assert!(
            erro(ga.0, gb.0, escp) < 1e-14 && erro(ga.1, gb.1, escp) < 1e-14,
            "{a} {b}"
        );
    }
}

#[test]
fn conexao_bi_no_ponto_do_proptest() {
    // Caso achado pelo proptest (ver `formulas_de_conexao`): erro dentro de |ζ|ε relativo a
    // |Ai| + |Bi|, a precisão documentada.
    let z = c(4.48328698456405, -7.761347195424438);
    let w = c(-0.5, 0.75f64.sqrt());
    let (a1, b1) = airy_complex(w * z);
    let (a2, b2) = airy_complex(w * w * z);
    let e6 = c((PI / 6.0).cos(), (PI / 6.0).sin());
    let bi = e6 * a1 + e6.conj() * a2;
    let zeta = 2.0 / 3.0 * z.norm().powf(1.5);
    let esc = a1.norm() + b1.norm() + a2.norm() + b2.norm();
    assert!((airy_complex(z).1 - bi).norm() / esc <= 2.0 * zeta * f64::EPSILON);
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn ponto() -> impl Strategy<Value = Complex64> {
        (0.0f64..30.0, -PI..PI).prop_map(|(r, a)| c(r * a.cos(), r * a.sin()))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        // Ai·Bi′ − Ai′·Bi = 1/π (DLMF 9.2.7).
        #[test]
        fn wronskiano(z in ponto()) {
            let (a, b) = airy_complex(z);
            let (ap, bp) = airy_prime_complex(z);
            let esc = (a * bp).norm() + (ap * b).norm();
            prop_assume!(esc.is_finite() && esc < 1e250);
            let tol = 1e-15 * (2.0 / 3.0 * z.norm().powf(1.5)).max(20.0);
            prop_assert!(erro(a * bp - ap * b, c(1.0 / PI, 0.0), esc) < tol, "z = {}", z);
        }

        // Ai(z) + ωAi(ωz) + ω²Ai(ω²z) = 0 e Bi(z) = e^{iπ/6}Ai(ωz) + e^{−iπ/6}Ai(ω²z), com
        // ω = e^{2πi/3} (DLMF 9.2.10, 9.2.12): misturam os três setores e testam os ramos.
        #[test]
        fn formulas_de_conexao(z in (0.0f64..12.0, -PI..PI).prop_map(|(r, a)| c(r * a.cos(), r * a.sin()))) {
            let w = c(-0.5, 0.75f64.sqrt());
            let (a0, b0) = airy_complex(z);
            let (a1, b1) = airy_complex(w * z);
            let (a2, b2) = airy_complex(w * w * z);
            let esc = a0.norm() + a1.norm() + a2.norm();
            // Escala da precisão documentada: |Ai| + |Bi| em cada ponto. Regressão do proptest:
            // em z = 4,483 − 7,761i (ω²z quase no semieixo negativo), o erro absoluto 4,9e-15 é o
            // esperado |ζ|ε, mas passava da tolerância quando dividido só por |Ai(ωz)| + |Ai(ω²z)|.
            let esc_bi = a1.norm() + b1.norm() + a2.norm() + b2.norm();
            // Condicionamento: cada termo carrega e^{∓ζ} e erra ~|ζ|ε, ζ = (2/3)z^{3/2}.
            let tol = 1e-15 * (2.0 / 3.0 * z.norm().powf(1.5)).max(20.0);
            prop_assert!(erro(a0 + w * a1 + w * w * a2, c(0.0, 0.0), esc) < tol, "z = {}", z);
            let e6 = c((PI / 6.0).cos(), (PI / 6.0).sin());
            let bi = e6 * a1 + e6.conj() * a2;
            prop_assert!(erro(b0, bi, esc_bi) < tol, "z = {}", z);
        }

        // Simetria exata de conjugação (Ai e Bi são reais no eixo real).
        #[test]
        fn conjugacao(z in ponto()) {
            prop_assume!(z.im != 0.0);
            let (a, b) = airy_complex(z);
            let (ac, bc) = airy_complex(z.conj());
            prop_assert_eq!((ac, bc), (a.conj(), b.conj()));
        }

        // Ai″ = z·Ai: diferença central de Ai′ com passo 1e-6.
        #[test]
        fn equacao_de_airy(z in (0.0f64..15.0, -PI..PI).prop_map(|(r, a)| c(r * a.cos(), r * a.sin()))) {
            let h = 1e-6;
            let d = (airy_prime_complex(z + h).0 - airy_prime_complex(z - h).0) / (2.0 * h);
            let want = z * airy_complex(z).0;
            let esc = want.norm().max(airy_prime_complex(z).0.norm()) + airy_complex(z).1.norm() * 1e-6;
            prop_assert!(erro(d, want, esc.max(1e-300)) < 1e-6, "z = {}", z);
        }
    }
}
