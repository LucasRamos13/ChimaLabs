//! Testes da funcionalidade F045 (TF-045). Oráculos: polinômios com raízes conhecidas,
//! interpolação exata, derivadas/integrais analíticas e frações parciais calculadas à mão.

use chimalab_math::linalg::LinalgWarning;
use chimalab_math::poly::*;
use num_complex::Complex64;

fn sorted_re(mut v: Vec<Complex64>) -> Vec<Complex64> {
    v.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
    v
}
fn close(a: &[f64], b: &[f64], tol: f64) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tol)
}

#[test]
fn ca1_raizes_de_x2_menos_3x_mais_2() {
    let r = sorted_re(roots(&[1.0, -3.0, 2.0]).unwrap());
    assert!(
        (r[0] - 1.0).norm() < 1e-15 && (r[1] - 2.0).norm() < 1e-15,
        "{r:?}"
    );
}

#[test]
fn raizes_casos_especiais() {
    // Complexas conjugadas: x² + 1
    let r = sorted_re(roots(&[1.0, 0.0, 1.0]).unwrap());
    assert!((r[0] - Complex64::new(0.0, -1.0)).norm() < 1e-15);
    assert!((r[1] - Complex64::new(0.0, 1.0)).norm() < 1e-15);
    // Zeros à esquerda ignorados; à direita viram raízes nulas: 0·x³ + x² − x = x(x − 1)
    let r = sorted_re(roots(&[0.0, 1.0, -1.0, 0.0]).unwrap());
    assert_eq!(r.len(), 2);
    assert!(r[0].norm() < 1e-15 && (r[1] - 1.0).norm() < 1e-15);
    assert!(roots(&[5.0]).unwrap().is_empty());
    assert!(roots(&[0.0, 0.0]).unwrap().is_empty());
    assert_eq!(
        (roots(&[1.0, f64::NAN])).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    // Grau 10 com raízes 1..10: erro pequeno relativo (problema clássico mal condicionado).
    let rr: Vec<Complex64> = (1..=10).map(|k| Complex64::new(k as f64, 0.0)).collect();
    let (_, p) = poly(&rr);
    let r = sorted_re(roots(&p.unwrap()).unwrap());
    for (k, z) in r.iter().enumerate() {
        assert!((z - (k + 1) as f64).norm() < 1e-7, "{z}");
    }
}

#[test]
fn poly_e_roots_sao_inversos() {
    let rr = vec![
        Complex64::new(-2.0, 0.0),
        Complex64::new(1.0, 3.0),
        Complex64::new(1.0, -3.0),
        Complex64::new(0.5, 0.0),
    ];
    let (c, real) = poly(&rr);
    let p = real.expect("raízes fechadas por conjugação ⇒ real");
    assert_eq!(c.len(), 5);
    assert!((p[0] - 1.0).abs() < 1e-15);
    let back = sorted_re(roots(&p).unwrap());
    for (x, y) in back.iter().zip(sorted_re(rr)) {
        assert!((x - y).norm() < 1e-12);
    }
    // Raiz complexa sem par: resultado só complexo
    assert!(poly(&[Complex64::new(0.0, 1.0)]).1.is_none());
}

#[test]
fn polyval_horner() {
    assert_eq!(
        polyval(&[2.0, -3.0, 1.0], &[0.0, 1.0, 2.0, -1.0]),
        vec![1.0, 0.0, 3.0, 6.0]
    );
    assert_eq!(polyval(&[], &[3.0]), vec![0.0]);
    let z = polyval_complex(&[1.0, 0.0, 1.0], &[Complex64::new(0.0, 1.0)]);
    assert!(z[0].norm() < 1e-15);
}

#[test]
fn ca2_polyfit_grau_n_em_n_mais_1_pontos_interpola() {
    let x = [-1.0, 0.0, 0.5, 2.0, 3.0];
    let y = [4.0, -1.0, 2.0, 0.5, 7.0];
    let f = polyfit(&x, &y, 4).unwrap();
    assert!(close(&polyval(&f.p, &x), &y, 1e-11));
    assert_eq!(f.df, 0);
    assert!(f.normr < 1e-11);
}

#[test]
fn polyfit_recupera_coeficientes_e_mede_residuo() {
    // y = 2x² − 3x + 1 exato, mais pontos que o necessário
    let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.25 - 2.0).collect();
    let y = polyval(&[2.0, -3.0, 1.0], &x);
    let f = polyfit(&x, &y, 2).unwrap();
    assert!(close(&f.p, &[2.0, -3.0, 1.0], 1e-12), "{:?}", f.p);
    assert_eq!(f.df, 17);
    assert_eq!(f.r.shape(), (3, 3));
    // Reta de mínimos quadrados de pontos (0,0), (1,1), (2,1): p = [0.5, 1/6], normr² = 1/6
    let f = polyfit(&[0.0, 1.0, 2.0], &[0.0, 1.0, 1.0], 1).unwrap();
    assert!(close(&f.p, &[0.5, 1.0 / 6.0], 1e-14));
    assert!((f.normr - (1.0f64 / 6.0).sqrt()).abs() < 1e-14);
}

#[test]
fn polyfit_escalonado_lida_com_x_grande() {
    // x em torno de 1e6: sem centralização a Vandermonde de grau 3 é muito mal condicionada.
    let x: Vec<f64> = (0..30).map(|i| 1e6 + i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| ((v - 1e6) / 10.0).powi(3) - (v - 1e6)).collect();
    let f = polyfit_scaled(&x, &y, 3).unwrap();
    let c = f.centering.unwrap();
    assert!((c.mean - (1e6 + 14.5)).abs() < 1e-6);
    let xh: Vec<f64> = x.iter().map(|v| (v - c.mean) / c.std).collect();
    assert!(close(&polyval(&f.p, &xh), &y, 1e-8));
    assert!(f.warnings.is_empty());
    // Sem escala: aviso de mal condicionamento/posto
    let g = polyfit(&x, &y, 3).unwrap();
    assert!(!g.warnings.is_empty(), "esperado aviso sem centralização");
}

#[test]
fn polyfit_erros_e_poucos_pontos() {
    assert_eq!(
        (polyfit(&[1.0, 2.0], &[1.0], 1)).unwrap_err().id(),
        "chimalab:linalg:dimensoesIncompativeis"
    );
    let f = polyfit(&[1.0, 2.0], &[3.0, 5.0], 3).unwrap();
    assert!(
        f.warnings
            .iter()
            .any(|w| matches!(w, LinalgWarning::RankDeficient { .. }))
    );
    assert!(close(&polyval(&f.p, &[1.0, 2.0]), &[3.0, 5.0], 1e-12));
}

#[test]
fn derivadas_e_integral() {
    assert_eq!(polyder(&[3.0, -2.0, 5.0, 7.0]), vec![9.0, -4.0, 5.0]);
    assert_eq!(polyder(&[4.0]), vec![0.0]);
    // d/dx[(x+1)(x−1)] = 2x
    assert_eq!(polyder_product(&[1.0, 1.0], &[1.0, -1.0]), vec![2.0, 0.0]);
    // d/dx[(x)/(x+1)] = 1/(x+1)²
    let (n, d) = polyder_quotient(&[1.0, 0.0], &[1.0, 1.0]);
    assert_eq!(n, vec![1.0]);
    assert_eq!(d, vec![1.0, 2.0, 1.0]);
    // ∫(3x² + 2x + 1) = x³ + x² + x + k
    assert_eq!(polyint(&[3.0, 2.0, 1.0], 4.0), vec![1.0, 1.0, 1.0, 4.0]);
    assert_eq!(polyder(&polyint(&[6.0, -2.0, 0.5], 9.0)), vec![6.0, -2.0, 0.5]);
}

fn find(r: &Residue, pole: f64, power_index: usize) -> Complex64 {
    // power_index: 0 → 1/(s−p), 1 → 1/(s−p)², ...
    let idx = r
        .poles
        .iter()
        .position(|p| (p - pole).norm() < 1e-6)
        .expect("polo")
        + power_index;
    r.residues[idx]
}

#[test]
fn residue_polos_simples() {
    // (s + 3)/((s + 1)(s + 2)) = 2/(s+1) − 1/(s+2)
    let r = residue(&[1.0, 3.0], &[1.0, 3.0, 2.0]).unwrap();
    assert!((find(&r, -1.0, 0) - 2.0).norm() < 1e-12);
    assert!((find(&r, -2.0, 0) + 1.0).norm() < 1e-12);
    assert!(r.direct.is_empty());
}

#[test]
fn residue_polo_repetido_e_termo_direto() {
    // 1/((s + 1)²(s + 2)) = 1/(s+1)² − 1/(s+1) + 1/(s+2)
    let r = residue(&[1.0], &[1.0, 4.0, 5.0, 2.0]).unwrap();
    assert!((find(&r, -1.0, 0) + 1.0).norm() < 1e-6, "{r:?}");
    assert!((find(&r, -1.0, 1) - 1.0).norm() < 1e-6);
    assert!((find(&r, -2.0, 0) - 1.0).norm() < 1e-6);
    // (s³ + 1)/(s² + 1) = s + (−s + 1)/(s² + 1); polos ±i com resíduos (−1 ∓ i)/2
    let r = residue(&[1.0, 0.0, 0.0, 1.0], &[1.0, 0.0, 1.0]).unwrap();
    assert!(close(&r.direct, &[1.0, 0.0], 1e-14));
    for (p, res) in r.poles.iter().zip(&r.residues) {
        let expected = if p.im > 0.0 {
            Complex64::new(-0.5, -0.5)
        } else {
            Complex64::new(-0.5, 0.5)
        };
        assert!((res - expected).norm() < 1e-12, "{p}: {res}");
    }
}

#[test]
fn residue_ida_e_volta() {
    for (b, a) in [
        (vec![1.0, 3.0], vec![1.0, 3.0, 2.0]),
        (vec![1.0], vec![1.0, 4.0, 5.0, 2.0]),
        (vec![1.0, 0.0, 0.0, 1.0], vec![1.0, 0.0, 1.0]),
        (vec![2.0, -1.0, 4.0], vec![1.0, 2.0, 5.0, 0.0]),
    ] {
        let r = residue(&b, &a).unwrap();
        let (bb, aa) = residue_inverse(&r);
        // Compara B/A normalizados pelo coeficiente líder de A.
        let bn: Vec<f64> = b.iter().map(|v| v / a[0]).collect();
        let an: Vec<f64> = a.iter().map(|v| v / a[0]).collect();
        assert!(close(&aa, &an, 1e-9), "{aa:?} vs {an:?}");
        assert!(close(&bb, &bn, 1e-6), "{bb:?} vs {bn:?}");
    }
}

#[test]
fn polyvalm_cayley_hamilton() {
    use chimalab_math::linalg::Matrix;
    // Teorema de Cayley-Hamilton: o polinômio característico anula a matriz.
    let x = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]).unwrap();
    // p(λ) = λ² − 5λ − 2
    let z = polyvalm(&[1.0, -5.0, -2.0], &x).unwrap();
    assert!(z.max_abs() < 1e-14);
    let i3 = polyvalm(&[2.0], &Matrix::identity(3)).unwrap();
    assert_eq!(i3, Matrix::identity(3).scale(2.0));
    assert_eq!(
        (polyvalm(&[1.0], &Matrix::zeros(2, 3))).unwrap_err().id(),
        "chimalab:linalg:matrizNaoQuadrada"
    );
}
