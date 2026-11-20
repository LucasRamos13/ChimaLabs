//! F042: Bessel J, Y, I, K e Hankel com argumento complexo. Oráculo:
//! tests/dados/F042/bessel_complexo.csv (séries ascendentes em decimal de 250 dígitos,
//! ferramentas/oraculos/f042_bessel_complexo.py, validado por valores de tabela, uma integral
//! independente e Wronskianos; independente da recorrência de Miller, da série de Temme e da
//! integral usadas na implementação).

use chimalab_math::special::*;
use num_complex::Complex64;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

struct Ref {
    nu: f64,
    z: Complex64,
    j: Complex64,
    y: Complex64,
    i: Complex64,
    k: Complex64,
}

fn referencias() -> Vec<Ref> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/bessel_complexo.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("nu"))
        .map(|l| {
            let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
            Ref {
                nu: v[0],
                z: c(v[1], v[2]),
                j: c(v[3], v[4]),
                y: c(v[5], v[6]),
                i: c(v[7], v[8]),
                k: c(v[9], v[10]),
            }
        })
        .collect()
}

#[test]
fn contra_referencias() {
    let refs = referencias();
    assert!(refs.len() >= 1000);
    let mut piores: Vec<(&str, f64, f64, Complex64)> = vec![];
    for (nome, idx) in [("J", 0), ("Y", 1), ("I", 2), ("K", 3), ("H1", 4), ("H2", 5)] {
        let mut pior = (0.0f64, 0.0, c(0.0, 0.0));
        for r in &refs {
            let (got, want) = match idx {
                0 => (besselj_complex(r.nu, r.z), r.j),
                1 => (bessely_complex(r.nu, r.z), r.y),
                2 => (besseli_complex(r.nu, r.z, false), r.i),
                3 => (besselk_complex(r.nu, r.z, false), r.k),
                4 => (
                    besselh_complex(r.nu, HankelKind::First, r.z, false),
                    r.j + c(0.0, 1.0) * r.y,
                ),
                _ => (
                    besselh_complex(r.nu, HankelKind::Second, r.z, false),
                    r.j - c(0.0, 1.0) * r.y,
                ),
            };
            if !want.norm().is_finite() || want.norm() < 1e-290 || want.norm() > 1e290 {
                continue;
            }
            // Escala do erro (condicionamento): perto dos zeros de J e Y só o erro absoluto em
            // relação a |J| + |Y| tem significado. I oscila perto do eixo imaginário, onde
            // I_ν(iy) ∝ J_ν(y) e K_ν(iy) ∝ H⁽²⁾_ν(y): escala |I| + (2/π)|K|. K não tem zeros em
            // Re z ≥ 0; em Re z < 0 vem de e^{−iνπ}K(w) − iπI(w) e a escala é |K| + π|I|.
            let escala = match idx {
                0 | 1 | 4 | 5 => r.j.norm() + r.y.norm(),
                2 => r.i.norm() + std::f64::consts::FRAC_2_PI * r.k.norm(),
                _ if r.z.re < 0.0 => r.k.norm() + std::f64::consts::PI * r.i.norm(),
                _ => r.k.norm(),
            };
            let e = (got - want).norm() / escala;
            if e > pior.0 {
                pior = (e, r.nu, r.z);
            }
        }
        piores.push((nome, pior.0, pior.1, pior.2));
    }
    for (nome, e, nu, z) in &piores {
        eprintln!("{nome}: pior {e:e} em ν = {nu}, z = {z}");
        assert!(*e < 1e-14, "{nome}: {e:e} em ν = {nu}, z = {z}");
    }
}

const PI: f64 = std::f64::consts::PI;

fn erro(got: Complex64, want: Complex64, escala: f64) -> f64 {
    if got == want {
        return 0.0;
    }
    (got - want).norm() / escala
}

#[test]
fn eixo_real_coincide_com_as_funcoes_reais() {
    for nu in [0.0, 0.5, 1.0, 2.3, -0.7, -3.0, 40.0] {
        for x in [0.0, 1e-300, 0.3, 2.0, 7.5, 30.0, 1e4] {
            let z = c(x, 0.0);
            assert_eq!(besselj_complex(nu, z), c(besselj(nu, x), 0.0));
            assert_eq!(bessely_complex(nu, z), c(bessely(nu, x), 0.0));
            assert_eq!(besseli_complex(nu, z, true), c(besseli(nu, x, true), 0.0));
            assert_eq!(besselk_complex(nu, z, false), c(besselk(nu, x, false), 0.0));
            for kind in [HankelKind::First, HankelKind::Second] {
                for scaled in [false, true] {
                    assert_eq!(besselh_complex(nu, kind, z, scaled), besselh(nu, kind, x, scaled));
                }
            }
        }
    }
}

/// Formas fechadas de ordem ½ (DLMF 10.16.1, 10.39.2): J = √(2/(πz)) sen z, Y = −√(2/(πz)) cos z,
/// I = √(2/(πz)) senh z, K = √(π/(2z)) e^{−z}; cobrem todos os métodos, de |z| = 1e-3 a 1e8.
#[test]
fn ordem_meio_em_forma_fechada() {
    let mut zs = vec![];
    for r in [1e-3, 0.7, 1.0, 1.9, 2.1, 5.0, 19.0, 21.0, 300.0, 1e8] {
        for k in -11..=12 {
            let a = PI * k as f64 / 12.0;
            zs.push(c(r * a.cos(), r * a.sin()));
        }
    }
    let mut pior = [0.0f64; 4];
    for z in zs {
        if z.im == 0.0 && z.re >= 0.0 {
            continue;
        }
        let f = (2.0 / (PI * z)).sqrt();
        let (j, y) = (besselj_complex(0.5, z), bessely_complex(0.5, z));
        let (jw, yw) = (f * z.sin(), -f * z.cos());
        let esc = jw.norm() + yw.norm();
        if !esc.is_finite() {
            continue;
        }
        pior[0] = pior[0].max(erro(j, jw, esc));
        pior[1] = pior[1].max(erro(y, yw, esc));
        let ks = besselk_complex(0.5, z, true);
        let kw = (PI / (2.0 * z)).sqrt();
        pior[3] = pior[3].max(erro(ks, kw, kw.norm()));
        if z.re.abs() < 600.0 {
            let iw = f * z.sinh();
            let kk = besselk_complex(0.5, z, false);
            let esc_i = iw.norm() + 2.0 / PI * kk.norm();
            pior[2] = pior[2].max(erro(besseli_complex(0.5, z, false), iw, esc_i));
        }
    }
    eprintln!(
        "ordem ½: J {:e}, Y {:e}, I {:e}, K {:e}",
        pior[0], pior[1], pior[2], pior[3]
    );
    assert!(pior.iter().all(|&e| e < 4e-15), "{pior:?}");
}

#[test]
fn casos_de_borda() {
    let nan = |w: Complex64| w.re.is_nan() && w.im.is_nan();
    for (nu, z) in [
        (f64::NAN, c(1.0, 1.0)),
        (f64::INFINITY, c(1.0, 1.0)),
        (0.0, c(f64::NAN, 1.0)),
        (0.0, c(1.0, f64::INFINITY)),
        (1.5, c(f64::NEG_INFINITY, 0.0)),
        (0.0, c(f64::INFINITY, -2.0)),
    ] {
        assert!(
            nan(besselj_complex(nu, z)) && nan(bessely_complex(nu, z)),
            "{nu} {z}"
        );
        assert!(
            nan(besseli_complex(nu, z, false)) && nan(besselk_complex(nu, z, true)),
            "{nu} {z}"
        );
        assert!(nan(besselh_complex(nu, HankelKind::First, z, false)), "{nu} {z}");
    }
    // z minúsculo: K₀(z) ≈ −ln(z/2) − γ, I_ν(z) ≈ (z/2)^ν/Γ(ν+1), Y₁(z) ≈ −2/(πz).
    for z in [
        c(1e-300, 1e-300),
        c(-1e-200, 0.0),
        c(0.0, -1e-150),
        c(-3e-20, 2e-20),
    ] {
        let k0 = -(0.5 * z).ln() - std::f64::consts::EULER_GAMMA;
        assert!(erro(besselk_complex(0.0, z, false), k0, k0.norm()) < 1e-15, "{z}");
        let i1 = 0.5 * z;
        // Regressão: (z/2)^ν por exp(ν ln) errava ~|ν ln(z/2)|ε (2,4e-14 em |z| = 1e-200).
        assert!(erro(besseli_complex(1.0, z, false), i1, i1.norm()) < 1e-15, "{z}");
        let y1 = -2.0 / PI * z.finv();
        assert!(erro(bessely_complex(1.0, z), y1, y1.norm()) < 1e-15, "{z}");
        // K_ν(z) ≈ ½Γ(ν)(z/2)^{−ν} para ν > 0 (DLMF 10.30.2); termo seguinte ~|z|^{2−2ν}.
        let kh = 0.5 * gamma(0.3) * Complex64::from_polar((0.5 * z.norm()).powf(-0.3), -0.3 * z.arg());
        let got = besselk_complex(0.3, z, false);
        let e = erro(got, kh, kh.norm());
        assert!(
            e < 4e-15 + 3.0 * z.norm().powf(0.6),
            "{z:e}: {got:e} {kh:e} erro {e:e}"
        );
    }
    // Corte: em z = −x + 0i (e com Im = −0) vale o lado de cima, J_ν(−x) = e^{iνπ}J_ν(x).
    for nu in [0.3, 1.0, -2.5] {
        for x in [0.5, 3.0, 40.0] {
            let want = c((nu * PI).cos(), (nu * PI).sin()) * besselj(nu, x);
            for z in [c(-x, 0.0), c(-x, -0.0)] {
                let got = besselj_complex(nu, z);
                assert!(
                    erro(got, want, besselj(nu, x).abs() + bessely(nu, x).abs()) < 4e-15,
                    "{nu} {z}"
                );
            }
            // Logo abaixo do corte, o outro lado: J_ν(−x − i0) = e^{−iνπ}J_ν(x).
            let below = besselj_complex(nu, c(-x, -1e-300));
            let want = c((nu * PI).cos(), -(nu * PI).sin()) * besselj(nu, x);
            assert!(erro(below, want, besselj(nu, x).abs() + bessely(nu, x).abs()) < 4e-15);
        }
    }
    // Ordem grande e transbordamento: sem pânico; valores finitos quando representáveis.
    let j = besselj_complex(100.0, c(50.0, 30.0));
    assert!(j.norm().is_finite() && j.norm() < 1e-12);
    assert!(!besselj_complex(0.0, c(0.0, 1e6)).norm().is_finite());
    assert!(
        !besselk_complex(100.0, c(1e-300, 1e-300), false)
            .norm()
            .is_finite()
    );
    // Ordem grande. Regressões: a divisão final de Miller calculava |soma|² ≈ 1e327 e dava
    // I₁₀₀(3+4i) = 0; o caminho longo de Miller (ν ≫ |z|) errava 1,2e-14 em I₂₀₀(−9i) = J₂₀₀(9),
    // hoje pela série com o prefator em produto (Γ(201) transborda). Referência: a série ascendente, que aqui converge rápido e sem cancelamento.
    // Tolerância: a fase ν·arg z amplifica o arredondamento de z em ~νε (condicionamento); em
    // z = −9i e −20i a fase é múltiplo de π e o cosseno é insensível, e ali se exige 4e-15, o
    // que detecta a regressão do caminho longo de Miller (1,2e-14). ν = 201,5 exercita o
    // prefator em produto com α ≠ 0 e número ímpar de fatores.
    for (nu, z, tol) in [
        (120.0, c(3.0, 4.0), 2.5e-14),
        (60.5, c(-2.0, 7.0), 1.5e-14),
        (200.0, c(0.0, -9.0), 4e-15),
        (200.0, c(0.0, -20.0), 4e-15),
        (201.5, c(0.0, -9.0), 5e-14),
    ] {
        let q = 0.25 * z * z;
        let (mut termo, mut soma) = (c(1.0, 0.0), c(1.0, 0.0));
        for k in 1..60 {
            termo *= q / (k as f64 * (nu + k as f64));
            soma += termo;
        }
        // (z/2)^ν/Γ(ν+1) = (z/2)^α/Γ(α+1) · Π_{k=1}^{n} (z/2)/(α+k), ν = α + n: sem Γ(ν+1),
        // que transborda para ν > 170.
        let alfa = nu - nu.floor();
        let mut pre = Complex64::from_polar((0.5 * z.norm()).powf(alfa), alfa * z.arg()) / gamma(alfa + 1.0);
        for k in 1..=(nu.floor() as usize) {
            pre *= 0.5 * z / (alfa + k as f64);
        }
        pre *= (-z.re.abs()).exp();
        let want = pre * soma;
        let got = besseli_complex(nu, z, true);
        assert!(
            want.norm() > 1e-300 && erro(got, want, want.norm()) < tol,
            "ν = {nu}: {got:e} {want:e}"
        );
    }
    // |z| pequeno fora do eixo (série, não Miller): I₀ e J₀ pela série Σ (±z²/4)ᵏ/(k!)².
    for z in [c(1e-300, 1e-300), c(-0.3, 0.4), c(0.0, -1e-100)] {
        let q = 0.25 * z * z;
        // Σ (±q)ᵏ/(k!)², 30 termos.
        let (mut ti, mut tj, mut iw, mut jw) = (c(1.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(1.0, 0.0));
        for k in 1..30 {
            let k2 = (k * k) as f64;
            ti *= q / k2;
            tj *= -q / k2;
            iw += ti;
            jw += tj;
        }
        assert!(erro(besseli_complex(0.0, z, false), iw, 1.0) < 2e-15, "{z}");
        assert!(erro(besselj_complex(0.0, z), jw, 1.0) < 2e-15, "{z}");
    }
    // |z| = 1.4e10: a forma escalada não transborda e |e^{−Re z}I₀(z)| ≈ 1/|√(2πz)|.
    let z = c(1e10, 1e10);
    let is = besseli_complex(0.0, z, true);
    assert!(
        (is.norm() * (2.0 * PI * z).sqrt().norm() - 1.0).abs() < 1e-10,
        "{is}"
    );
}

/// Wronskiano I/K nas formas escaladas com |z| de 800 a 1600 e ν > 50 (ainda abaixo do limiar
/// da assintótica, 20 + ν²/2): de k = |z| até 0 a recorrência de Miller cresce ~e^{0,47|z|} > 1e250
/// e precisa ser reescalada. Com Is = e^{−|Re z|}I e Ks = e^{z}K,
/// IsₙKsₙ₊₁ + Isₙ₊₁Ksₙ = e^{z − |Re z|}/z (DLMF 10.28.2).
#[test]
fn wronskiano_escalado_com_argumento_grande() {
    for (nu, z) in [
        (60.0, c(1500.0, 2.0)),
        (70.3, c(-1600.0, 40.0)),
        (55.5, c(1400.0, -3.0)),
        (45.0, c(10.0, 800.0)),
    ] {
        let (i0, k0) = (besseli_complex(nu, z, true), besselk_complex(nu, z, true));
        let (i1, k1) = (
            besseli_complex(nu + 1.0, z, true),
            besselk_complex(nu + 1.0, z, true),
        );
        let w = i0 * k1 + i1 * k0;
        let want = c(z.re - z.re.abs(), z.im).exp() / z;
        let esc = (i0 * k1).norm() + (i1 * k0).norm();
        assert!(erro(w, want, esc) < 1e-14, "ν = {nu}, z = {z}: {w:e} {want:e}");
    }
}

#[test]
fn continuidade_nas_trocas_de_metodo() {
    // |z| = 1 (série de I ↔ Miller), |z| = 2 (Temme ↔ integral), |z| = 20 + ν²/2 (Miller ↔
    // assintótica), Re z = 0 (continuação de I e K) e Im z = 0 (conjugação).
    let casos: [(f64, Complex64, Complex64); 8] = [
        (0.3, c(0.6, 0.8), c(0.6f64.next_up(), 0.8f64.next_up())),
        (1.7, c(1.2, -1.6), c(1.2f64.next_up(), -(1.6f64.next_up()))),
        (0.0, c(0.0, 20.0), c(0.0, 20.0f64.next_up())),
        (4.0, c(28.0, 0.5), c(28.0f64.next_down(), 0.5)),
        (2.5, c(1e-300, 3.0), c(-1e-300, 3.0)),
        (0.5, c(-1e-300, -0.7), c(1e-300, -0.7)),
        (1.0, c(-5.0, 1e-300), c(-5.0, 1e-290)),
        (6.25, c(3.0, -1e-300), c(3.0, 1e-300)),
    ];
    for (nu, a, b) in casos {
        let fs: [fn(f64, Complex64) -> Complex64; 4] = [
            besselj_complex,
            bessely_complex,
            |n, z| besseli_complex(n, z, true),
            |n, z| besselk_complex(n, z, true),
        ];
        for (idx, f) in fs.iter().enumerate() {
            let (fa, fb) = (f(nu, a), f(nu, b));
            let e = erro(fa, fb, fa.norm().max(fb.norm()));
            assert!(e < 1e-14, "função {idx}, ν = {nu}: {a} → {fa}, {b} → {fb}");
        }
    }
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn ponto() -> impl Strategy<Value = Complex64> {
        (0.05f64..60.0, -3.1f64..3.1).prop_map(|(r, a)| c(r * a.cos(), r * a.sin()))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        // J_{ν+1}Y_ν − J_νY_{ν+1} = 2/(πz) (DLMF 10.5.2).
        #[test]
        fn wronskiano_jy(nu in -6.0f64..25.0, z in ponto()) {
            let (j0, y0) = (besselj_complex(nu, z), bessely_complex(nu, z));
            let (j1, y1) = (besselj_complex(nu + 1.0, z), bessely_complex(nu + 1.0, z));
            let w = j1 * y0 - j0 * y1;
            let esc = (j1 * y0).norm() + (j0 * y1).norm();
            prop_assume!(esc.is_finite() && esc < 1e250);
            prop_assert!(erro(w, 2.0 / (PI * z), esc) < 2e-14, "ν = {}, z = {}", nu, z);
        }

        // I_νK_{ν+1} + I_{ν+1}K_ν = 1/z (DLMF 10.28.2), nas formas escaladas.
        #[test]
        fn wronskiano_ik(nu in -6.0f64..25.0, z in ponto()) {
            let (i0, k0) = (besseli_complex(nu, z, false), besselk_complex(nu, z, false));
            let (i1, k1) = (besseli_complex(nu + 1.0, z, false), besselk_complex(nu + 1.0, z, false));
            let w = i0 * k1 + i1 * k0;
            let esc = (i0 * k1).norm() + (i1 * k0).norm();
            prop_assume!(esc.is_finite() && esc < 1e250 && esc > 1e-250);
            prop_assert!(erro(w, z.finv(), esc) < 2e-14, "ν = {}, z = {}", nu, z);
        }

        // J_{ν−1} + J_{ν+1} = (2ν/z)J_ν (DLMF 10.6.1) e K_{ν+1} − K_{ν−1} = (2ν/z)K_ν (10.29.1).
        #[test]
        fn recorrencias(nu in -6.0f64..25.0, z in ponto()) {
            let (a, b, m) = (besselj_complex(nu - 1.0, z), besselj_complex(nu + 1.0, z), besselj_complex(nu, z));
            let esc = a.norm() + b.norm() + bessely_complex(nu, z).norm();
            prop_assume!(esc.is_finite() && esc < 1e250);
            prop_assert!(erro(a + b, 2.0 * nu / z * m, esc) < 2e-14, "J: ν = {}, z = {}", nu, z);
            let (a, b, m) = (
                besselk_complex(nu - 1.0, z, true),
                besselk_complex(nu + 1.0, z, true),
                besselk_complex(nu, z, true),
            );
            let esc = a.norm() + b.norm();
            prop_assume!(esc.is_finite() && esc < 1e250);
            prop_assert!(erro(b - a, 2.0 * nu / z * m, esc) < 2e-14, "K: ν = {}, z = {}", nu, z);
        }

        // Simetria de conjugação exata (DLMF 10.11.9) e consistência das formas escaladas e de
        // Hankel.
        #[test]
        fn conjugacao_e_escalas(nu in -6.0f64..25.0, z in ponto()) {
            prop_assume!(z.im != 0.0);
            prop_assert_eq!(besselj_complex(nu, z.conj()), besselj_complex(nu, z).conj());
            prop_assert_eq!(bessely_complex(nu, z.conj()), bessely_complex(nu, z).conj());
            prop_assert_eq!(besselk_complex(nu, z.conj(), true), besselk_complex(nu, z, true).conj());
            let (j, y) = (besselj_complex(nu, z), bessely_complex(nu, z));
            let h1 = besselh_complex(nu, HankelKind::First, z, false);
            let h2 = besselh_complex(nu, HankelKind::Second, z, false);
            let esc = j.norm() + y.norm();
            prop_assume!(esc.is_finite() && esc < 1e250);
            prop_assert!(erro(h1, j + c(0.0, 1.0) * y, esc) < 1e-14);
            prop_assert!(erro(h2, j - c(0.0, 1.0) * y, esc) < 1e-14);
            let h1s = besselh_complex(nu, HankelKind::First, z, true);
            prop_assert!(erro(h1s * c(-z.im, z.re).exp(), h1, h1.norm()) < 1e-13);
            let h2s = besselh_complex(nu, HankelKind::Second, z, true);
            prop_assert!(erro(h2s * c(z.im, -z.re).exp(), h2, h2.norm()) < 1e-13);
            let is = besseli_complex(nu, z, true);
            let i = besseli_complex(nu, z, false);
            prop_assume!(i.norm().is_finite());
            prop_assert!(erro(is * z.re.abs().exp(), i, i.norm()) < 1e-13);
        }
    }
}

/// Medição: `cargo test --release -- --ignored desempenho_bessel_complexo --nocapture`.
#[test]
#[ignore]
fn desempenho_bessel_complexo() {
    let zs: Vec<Complex64> = (0..2000)
        .map(|k| {
            let r = 0.01 * 1.006f64.powi(k);
            let a = 2.4 * k as f64;
            c(r * a.cos(), r * a.sin())
        })
        .collect();
    for (nome, f) in [
        ("J", besselj_complex as fn(f64, Complex64) -> Complex64),
        ("Y", bessely_complex),
        ("K", |n, z| besselk_complex(n, z, false)),
    ] {
        let t = std::time::Instant::now();
        let mut acc = 0.0;
        for &z in &zs {
            acc += f(1.3, z).norm().min(1.0);
        }
        eprintln!(
            "{nome}: {:.1} µs por chamada (|z| de 0,01 a 1,6e3; {acc:.3})",
            t.elapsed().as_secs_f64() * 1e6 / zs.len() as f64
        );
    }
}
