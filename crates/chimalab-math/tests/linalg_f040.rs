//! Testes da funcionalidade F040 (TF-040 em docs/TESTES_POR_FUNCIONALIDADE.md).
//!
//! Oráculos independentes: soluções analíticas, identidades algébricas (reconstrução,
//! ortogonalidade, condições de Moore-Penrose) e fórmulas fechadas publicadas. Nenhum resultado
//! do produto investigado é usado como referência.

use chimalab_math::linalg::*;
use num_complex::Complex64;

// ---------- utilidades ----------

/// Gerador pseudoaleatório determinístico (xorshift64*), só para montar casos de teste.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
    fn matrix(&mut self, m: usize, n: usize) -> Matrix {
        Matrix::from_fn(m, n, |_, _| self.next())
    }
}

fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    assert_eq!(a.shape(), b.shape());
    a.sub(b).unwrap().max_abs()
}

fn mm(a: &Matrix, b: &Matrix) -> Matrix {
    a.matmul(b).unwrap()
}

/// Resíduo relativo ‖A·x − b‖₁ / (‖A‖₁‖x‖₁ + ‖b‖₁).
fn relative_residual(a: &Matrix, x: &Matrix, b: &Matrix) -> f64 {
    let r = mm(a, x).sub(b).unwrap();
    norm(&r, NormKind::One).unwrap()
        / (norm(a, NormKind::One).unwrap() * norm(x, NormKind::One).unwrap()
            + norm(b, NormKind::One).unwrap())
}

fn hilbert(n: usize) -> Matrix {
    Matrix::from_fn(n, n, |i, j| 1.0 / (i + j + 1) as f64)
}

fn binom(n: i64, k: i64) -> f64 {
    if k < 0 || k > n {
        return 0.0;
    }
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Inversa exata da matriz de Hilbert (fórmula fechada, índices a partir de 1):
/// (H⁻¹)ᵢⱼ = (−1)^(i+j) (i+j−1) C(n+i−1, n−j) C(n+j−1, n−i) C(i+j−2, i−1)².
fn hilbert_inverse(n: usize) -> Matrix {
    let n = n as i64;
    Matrix::from_fn(n as usize, n as usize, |i, j| {
        let (i, j) = (i as i64 + 1, j as i64 + 1);
        let s = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
        s * (i + j - 1) as f64
            * binom(n + i - 1, n - j)
            * binom(n + j - 1, n - i)
            * binom(i + j - 2, i - 1).powi(2)
    })
}

fn is_orthonormal_cols(q: &Matrix, tol: f64) -> bool {
    let qtq = mm(&q.transpose(), q);
    max_abs_diff(&qtq, &Matrix::identity(q.cols())) < tol
}

// ---------- CA1: resíduo < 1e-12 em matrizes bem condicionadas ----------

#[test]
fn ca1_residuo_pequeno_em_suite_de_matrizes() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for &n in &[1usize, 2, 3, 5, 10, 31, 64, 100, 200] {
        // Diagonal dominante: bem condicionada por construção.
        let mut a = rng.matrix(n, n);
        for i in 0..n {
            a.set(i, i, a.get(i, i) + n as f64);
        }
        let b = rng.matrix(n, 3);
        let s = solve(&a, &b).unwrap();
        // Matriz 1×1 é triangular; para n > 1 a matriz densa usa LU.
        if n > 1 {
            assert_eq!(s.method, SolveMethod::Lu, "n={n}");
        }
        assert!(s.warnings.is_empty(), "n={n}: {:?}", s.warnings);
        let r = relative_residual(&a, &s.x, &b);
        assert!(r < 1e-12, "n={n}: resíduo relativo {r:e}");
    }
}

#[test]
fn ca1_escolhe_metodo_pela_estrutura() {
    let mut rng = Rng(42);
    let n = 40;
    let b = rng.matrix(n, 2);
    let mut up = Matrix::from_fn(n, n, |i, j| if i <= j { rng.next() } else { 0.0 });
    let mut lo = up.transpose();
    for i in 0..n {
        up.set(i, i, 4.0 + up.get(i, i));
        lo.set(i, i, 4.0 + lo.get(i, i));
    }
    let su = solve(&up, &b).unwrap();
    assert_eq!(su.method, SolveMethod::UpperTriangular);
    assert!(relative_residual(&up, &su.x, &b) < 1e-13);
    let sl = solve(&lo, &b).unwrap();
    assert_eq!(sl.method, SolveMethod::LowerTriangular);
    assert!(relative_residual(&lo, &sl.x, &b) < 1e-13);

    // Simétrica definida positiva: M = GᵀG + n·I
    let g = rng.matrix(n, n);
    let mut spd = mm(&g.transpose(), &g);
    for i in 0..n {
        spd.set(i, i, spd.get(i, i) + n as f64);
    }
    // Força simetria exata (o produto já é simétrico em aritmética exata).
    let spd = Matrix::from_fn(n, n, |i, j| if i <= j { spd.get(i, j) } else { spd.get(j, i) });
    let sc = solve(&spd, &b).unwrap();
    assert_eq!(sc.method, SolveMethod::Cholesky);
    assert!(relative_residual(&spd, &sc.x, &b) < 1e-13);

    // Simétrica indefinida com diagonal positiva: Cholesky falha e cai para LU.
    let ind = Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 1.0]]).unwrap();
    let si = solve(&ind, &Matrix::column(&[3.0, 3.0])).unwrap();
    assert_eq!(si.method, SolveMethod::Lu);
    assert!(max_abs_diff(&si.x, &Matrix::column(&[1.0, 1.0])) < 1e-14);
}

#[test]
fn solucao_exata_conhecida() {
    // 2x + y = 3; x + 3y = 5  →  x = 0.8, y = 1.4
    let a = Matrix::from_rows(&[&[2.0, 1.0], &[1.0, 3.0]]).unwrap();
    let s = solve(&a, &Matrix::column(&[3.0, 5.0])).unwrap();
    assert!(max_abs_diff(&s.x, &Matrix::column(&[0.8, 1.4])) < 1e-15);
}

// ---------- CA3: avisos de singularidade ----------

#[test]
fn ca3_aviso_para_matriz_singular() {
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 4.0]]).unwrap();
    let s = solve(&a, &Matrix::column(&[1.0, 2.0])).unwrap();
    assert!(
        s.warnings
            .iter()
            .any(|w| matches!(w, LinalgWarning::Singular | LinalgWarning::NearlySingular { .. })),
        "{:?}",
        s.warnings
    );
    let (_, w) = inv(&a).unwrap();
    assert!(!w.is_empty());
    assert!(rcond(&a).unwrap() < f64::EPSILON);
}

#[test]
fn ca3_aviso_para_matriz_mal_condicionada() {
    // Hilbert 14: número de condição ~1e17-1e19, rcond abaixo de eps.
    let h = hilbert(14);
    let s = solve(&h, &Matrix::column(&[1.0; 14])).unwrap();
    assert!(
        s.warnings
            .iter()
            .any(|w| matches!(w, LinalgWarning::NearlySingular { .. })),
        "{:?}",
        s.warnings
    );
    assert_eq!(s.warnings[0].id(), "chimalab:linalg:quaseSingular");
}

#[test]
fn triangular_com_zero_na_diagonal_e_singular() {
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[0.0, 0.0]]).unwrap();
    let s = solve(&a, &Matrix::column(&[1.0, 1.0])).unwrap();
    assert_eq!(s.method, SolveMethod::UpperTriangular);
    assert_eq!(s.warnings, vec![LinalgWarning::Singular]);
}

// ---------- rcond ----------

#[test]
fn rcond_da_identidade_e_ortogonal() {
    assert!((rcond(&Matrix::identity(7)).unwrap() - 1.0).abs() < 1e-15);
    // Para uma matriz diagonal, rcond na norma 1 = min|d| / max|d|.
    let d = Matrix::from_fn(4, 4, |i, j| if i == j { [1.0, 10.0, 0.5, 4.0][i] } else { 0.0 });
    let r = rcond(&d).unwrap();
    assert!((r - 0.05).abs() < 1e-15, "{r}");
}

#[test]
fn rcond_estima_dentro_de_fator_3_do_valor_exato() {
    let mut rng = Rng(7);
    for n in [5usize, 20, 60] {
        let a = rng.matrix(n, n);
        let exact =
            1.0 / (norm(&a, NormKind::One).unwrap() * norm(&inv(&a).unwrap().0, NormKind::One).unwrap());
        let est = rcond(&a).unwrap();
        // O estimador nunca subestima ‖A⁻¹‖₁ por mais que o vetor testado permite,
        // então rcond estimado ≥ exato; na prática fica dentro de um fator pequeno.
        assert!(
            est >= exact * (1.0 - 1e-12),
            "n={n}: est {est:e} < exato {exact:e}"
        );
        assert!(est <= exact * 3.0, "n={n}: est {est:e} > 3×exato {exact:e}");
    }
}

// ---------- mínimos quadrados ----------

#[test]
fn minimos_quadrados_sobredeterminado_bate_com_equacoes_normais() {
    let mut rng = Rng(11);
    let a = rng.matrix(30, 4);
    let b = rng.matrix(30, 1);
    let s = solve(&a, &b).unwrap();
    assert_eq!(s.method, SolveMethod::LeastSquaresQr);
    assert!(s.warnings.is_empty());
    // Condição de otimalidade: Aᵀ(Ax − b) = 0.
    let g = mm(&a.transpose(), &mm(&a, &s.x).sub(&b).unwrap());
    assert!(g.max_abs() < 1e-12, "{}", g.max_abs());
}

#[test]
fn minimos_quadrados_ajusta_reta_exata() {
    // Pontos sobre y = 2 + 3t: o ajuste deve recuperar os coeficientes.
    let t: Vec<f64> = (0..10).map(|i| i as f64 / 3.0).collect();
    let a = Matrix::from_fn(10, 2, |i, j| if j == 0 { 1.0 } else { t[i] });
    let b = Matrix::from_fn(10, 1, |i, _| 2.0 + 3.0 * t[i]);
    let s = solve(&a, &b).unwrap();
    assert!(max_abs_diff(&s.x, &Matrix::column(&[2.0, 3.0])) < 1e-13);
}

#[test]
fn posto_deficiente_gera_aviso_e_solucao_basica() {
    // Terceira coluna = soma das duas primeiras: posto 2.
    let mut rng = Rng(3);
    let base = rng.matrix(8, 2);
    let a = Matrix::from_fn(8, 3, |i, j| {
        if j < 2 {
            base.get(i, j)
        } else {
            base.get(i, 0) + base.get(i, 1)
        }
    });
    let b = rng.matrix(8, 1);
    let s = solve(&a, &b).unwrap();
    assert!(
        matches!(
            s.warnings.as_slice(),
            [LinalgWarning::RankDeficient { rank: 2, .. }]
        ),
        "{:?}",
        s.warnings
    );
    let nonzero = (0..3).filter(|&i| s.x.get(i, 0) != 0.0).count();
    assert!(
        nonzero <= 2,
        "solução básica deve ter no máximo 2 componentes não nulas"
    );
    let g = mm(&a.transpose(), &mm(&a, &s.x).sub(&b).unwrap());
    assert!(g.max_abs() < 1e-12);
}

#[test]
fn subdeterminado_resolve_exatamente() {
    let mut rng = Rng(5);
    let a = rng.matrix(3, 6);
    let b = rng.matrix(3, 2);
    let s = solve(&a, &b).unwrap();
    assert!(max_abs_diff(&mm(&a, &s.x), &b) < 1e-13);
}

// ---------- inversa e determinante ----------

#[test]
fn inversa_de_hilbert_bate_com_formula_fechada() {
    for n in 2..=6 {
        let (x, w) = inv(&hilbert(n)).unwrap();
        assert!(w.is_empty(), "n={n}: {w:?}");
        let exact = hilbert_inverse(n);
        let rel = max_abs_diff(&x, &exact) / exact.max_abs();
        // Erro relativo esperado ~ cond(H)·eps; cond(H6) ≈ 1.5e7.
        assert!(rel < 1e-8, "n={n}: erro relativo {rel:e}");
    }
}

#[test]
fn inversa_vezes_matriz_e_identidade() {
    let mut rng = Rng(99);
    let mut a = rng.matrix(50, 50);
    for i in 0..50 {
        a.set(i, i, a.get(i, i) + 10.0);
    }
    let (x, _) = inv(&a).unwrap();
    assert!(max_abs_diff(&mm(&a, &x), &Matrix::identity(50)) < 1e-13);
}

#[test]
fn determinantes_conhecidos() {
    assert_eq!(det(&Matrix::zeros(0, 0)).unwrap(), 1.0);
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]).unwrap();
    assert!((det(&a).unwrap() + 2.0).abs() < 1e-14);
    // Permutação ímpar (troca de duas linhas da identidade): det = −1.
    let p = Matrix::from_rows(&[&[0.0, 1.0, 0.0], &[1.0, 0.0, 0.0], &[0.0, 0.0, 1.0]]).unwrap();
    assert!((det(&p).unwrap() + 1.0).abs() < 1e-15);
    // det(H3) = 1/2160.
    assert!((det(&hilbert(3)).unwrap() - 1.0 / 2160.0).abs() < 1e-17);
    // det(cA) = cⁿ det(A); det(AB) = det(A) det(B)
    let mut rng = Rng(8);
    let x = rng.matrix(6, 6);
    let y = rng.matrix(6, 6);
    let (dx, dy, dxy) = (det(&x).unwrap(), det(&y).unwrap(), det(&mm(&x, &y)).unwrap());
    assert!((dxy - dx * dy).abs() < 1e-12 * dxy.abs().max(1.0));
    assert!((det(&x.scale(2.0)).unwrap() - 64.0 * dx).abs() < 1e-12 * dx.abs().max(1.0));
}

// ---------- decomposições ----------

#[test]
fn lu_reconstroi_pa_igual_lu() {
    let mut rng = Rng(21);
    for &(m, n) in &[(5usize, 5usize), (7, 4), (4, 7), (1, 1)] {
        let a = rng.matrix(m, n);
        let f = lu(&a).unwrap();
        let pa = mm(&f.p_matrix(), &a);
        assert!(max_abs_diff(&pa, &mm(&f.l, &f.u)) < 1e-14, "({m},{n})");
        for i in 0..f.l.cols() {
            assert_eq!(f.l.get(i, i), 1.0);
        }
        assert!(f.u.is_upper_triangular());
        assert!(f.l.is_lower_triangular());
    }
}

#[test]
fn qr_reconstroi_e_q_ortogonal() {
    let mut rng = Rng(22);
    for &(m, n) in &[(6usize, 6usize), (9, 4), (3, 7)] {
        let a = rng.matrix(m, n);
        let f = qr(&a).unwrap();
        assert_eq!(f.q.shape(), (m, m));
        assert_eq!(f.r.shape(), (m, n));
        assert!(is_orthonormal_cols(&f.q, 1e-14));
        assert!(f.r.is_upper_triangular());
        assert!(max_abs_diff(&mm(&f.q, &f.r), &a) < 1e-14, "({m},{n})");
    }
}

#[test]
fn cholesky_reconstroi_e_rejeita_indefinida() {
    let a = Matrix::from_rows(&[&[4.0, 12.0, -16.0], &[12.0, 37.0, -43.0], &[-16.0, -43.0, 98.0]]).unwrap();
    let r = chol(&a).unwrap().r;
    // Fator exato conhecido deste exemplo clássico: R = [[2,6,−8],[0,1,5],[0,0,3]].
    let exact = Matrix::from_rows(&[&[2.0, 6.0, -8.0], &[0.0, 1.0, 5.0], &[0.0, 0.0, 3.0]]).unwrap();
    assert!(max_abs_diff(&r, &exact) < 1e-14);
    assert!(max_abs_diff(&mm(&r.transpose(), &r), &a) < 1e-13);
    let ind = Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 1.0]]).unwrap();
    let e = chol(&ind).unwrap_err();
    assert_eq!(e, LinalgError::NotPositiveDefinite);
    assert_eq!(e.id(), "chimalab:linalg:naoDefinidaPositiva");
}

#[test]
fn svd_reconstroi_ordena_e_e_ortogonal() {
    let mut rng = Rng(23);
    for &(m, n) in &[(6usize, 6usize), (10, 3), (3, 10)] {
        let a = rng.matrix(m, n);
        for economy in [false, true] {
            let f = svd(&a, economy).unwrap();
            let k = m.min(n);
            assert_eq!(f.s.len(), k);
            assert!(f.s.windows(2).all(|w| w[0] >= w[1]));
            assert!(is_orthonormal_cols(&f.u, 1e-13));
            assert!(is_orthonormal_cols(&f.v, 1e-13));
            let us = Matrix::from_fn(m, k, |i, j| f.u.get(i, j) * f.s[j]);
            let vk = f.v.submatrix(0, n, 0, k);
            assert!(
                max_abs_diff(&mm(&us, &vk.transpose()), &a) < 1e-13,
                "({m},{n}) economy={economy}"
            );
        }
    }
    // Valores singulares de diag(3, −2, 1) = (3, 2, 1).
    let d = Matrix::from_fn(3, 3, |i, j| if i == j { [3.0, -2.0, 1.0][i] } else { 0.0 });
    let s = singular_values(&d).unwrap();
    assert!((s[0] - 3.0).abs() < 1e-15 && (s[1] - 2.0).abs() < 1e-15 && (s[2] - 1.0).abs() < 1e-15);
}

#[test]
fn autovalores_de_tridiagonal_toeplitz_batem_com_formula_analitica() {
    // T = tridiag(−1, 2, −1): λ_k = 2 − 2cos(kπ/(n+1)), k = 1..n.
    let n = 25;
    let t = Matrix::from_fn(n, n, |i, j| match i.abs_diff(j) {
        0 => 2.0,
        1 => -1.0,
        _ => 0.0,
    });
    match eig(&t).unwrap() {
        Eig::Symmetric { values, vectors } => {
            for (k, v) in values.iter().enumerate() {
                let exact = 2.0 - 2.0 * (((k + 1) as f64) * std::f64::consts::PI / (n as f64 + 1.0)).cos();
                assert!((v - exact).abs() < 1e-13, "k={k}: {v} vs {exact}");
            }
            assert!(is_orthonormal_cols(&vectors, 1e-13));
            let d = Matrix::from_fn(n, n, |i, j| if i == j { values[i] } else { 0.0 });
            assert!(max_abs_diff(&mm(&t, &vectors), &mm(&vectors, &d)) < 1e-13);
        }
        other => panic!("esperado caso simétrico: {other:?}"),
    }
}

#[test]
fn autovalores_complexos_da_rotacao() {
    let a = Matrix::from_rows(&[&[0.0, -1.0], &[1.0, 0.0]]).unwrap();
    match eig(&a).unwrap() {
        Eig::General { values, vectors } => {
            let mut v = values.clone();
            v.sort_by(|x, y| x.im.total_cmp(&y.im));
            assert!((v[0] - Complex64::new(0.0, -1.0)).norm() < 1e-15);
            assert!((v[1] - Complex64::new(0.0, 1.0)).norm() < 1e-15);
            let ac = ComplexMatrix::from_real(&a);
            let av = ac.matmul(&vectors).unwrap();
            for (j, lambda) in values.iter().enumerate() {
                for i in 0..2 {
                    assert!((av.get(i, j) - vectors.get(i, j) * lambda).norm() < 1e-14);
                }
            }
        }
        other => panic!("esperado caso geral: {other:?}"),
    }
}

#[test]
fn autovalores_de_matriz_companheira_sao_as_raizes() {
    // p(x) = (x−1)(x−2)(x−3)(x−4) = x⁴ − 10x³ + 35x² − 50x + 24
    let c = [-10.0, 35.0, -50.0, 24.0];
    let a = Matrix::from_fn(4, 4, |i, j| {
        if i == 0 {
            -c[j]
        } else if i == j + 1 {
            1.0
        } else {
            0.0
        }
    });
    let Eig::General { values, .. } = eig(&a).unwrap() else {
        panic!("não simétrica")
    };
    let mut re: Vec<f64> = values
        .iter()
        .map(|z| {
            assert!(z.im.abs() < 1e-10);
            z.re
        })
        .collect();
    re.sort_by(f64::total_cmp);
    for (k, r) in re.iter().enumerate() {
        assert!((r - (k + 1) as f64).abs() < 1e-10, "{re:?}");
    }
}

// ---------- normas, condição, posto, subespaços ----------

#[test]
fn normas_conhecidas() {
    let a = Matrix::from_rows(&[&[1.0, -2.0], &[-3.0, 4.0]]).unwrap();
    assert_eq!(norm(&a, NormKind::One).unwrap(), 6.0);
    assert_eq!(norm(&a, NormKind::Inf).unwrap(), 7.0);
    assert!((norm(&a, NormKind::Fro).unwrap() - 30f64.sqrt()).abs() < 1e-15);
    // ‖A‖₂² = maior autovalor de AᵀA = 15 + √221
    assert!((norm(&a, NormKind::Two).unwrap() - (15.0 + 221f64.sqrt()).sqrt()).abs() < 1e-14);
    let v = Matrix::column(&[3.0, -4.0]);
    assert_eq!(norm(&v, NormKind::Two).unwrap(), 5.0);
    assert_eq!(norm(&v, NormKind::One).unwrap(), 7.0);
    assert_eq!(norm(&v, NormKind::Inf).unwrap(), 4.0);
    assert!((norm(&v, NormKind::P(3.0)).unwrap() - 91f64.cbrt()).abs() < 1e-14);
    assert_eq!(
        (norm(&a, NormKind::P(3.0))).unwrap_err().id(),
        "chimalab:linalg:dimensoesIncompativeis"
    );
    // Sem overflow intermediário
    let big = Matrix::column(&[1e200, 1e200]);
    assert!((norm(&big, NormKind::Two).unwrap() / (1e200 * 2f64.sqrt()) - 1.0).abs() < 1e-15);
    assert_eq!(norm(&Matrix::zeros(0, 3), NormKind::Two).unwrap(), 0.0);
}

#[test]
fn condicao_de_ortogonal_e_um_e_de_singular_e_infinito() {
    let mut rng = Rng(31);
    let q = qr(&rng.matrix(8, 8)).unwrap().q;
    assert!((cond(&q).unwrap() - 1.0).abs() < 1e-13);
    let s = Matrix::from_rows(&[&[1.0, 1.0], &[1.0, 1.0]]).unwrap();
    assert!(cond(&s).unwrap() > 1e15);
}

#[test]
fn posto_pinv_nulo_e_imagem() {
    let mut rng = Rng(41);
    let b = rng.matrix(7, 3);
    let c = rng.matrix(3, 5);
    let a = mm(&b, &c); // 7×5 de posto 3
    assert_eq!(rank(&a, None).unwrap(), 3);
    assert_eq!(rank(&Matrix::identity(4), None).unwrap(), 4);
    assert_eq!(rank(&Matrix::zeros(3, 3), None).unwrap(), 0);

    // Condições de Moore-Penrose.
    let p = pinv(&a, None).unwrap();
    assert_eq!(p.shape(), (5, 7));
    let apa = mm(&mm(&a, &p), &a);
    let pap = mm(&mm(&p, &a), &p);
    assert!(max_abs_diff(&apa, &a) < 1e-12);
    assert!(max_abs_diff(&pap, &p) < 1e-12);
    let ap = mm(&a, &p);
    let pa = mm(&p, &a);
    assert!(max_abs_diff(&ap, &ap.transpose()) < 1e-12);
    assert!(max_abs_diff(&pa, &pa.transpose()) < 1e-12);

    let z = null_space(&a).unwrap();
    assert_eq!(z.shape(), (5, 2));
    assert!(mm(&a, &z).max_abs() < 1e-12);
    assert!(is_orthonormal_cols(&z, 1e-13));

    let o = orth(&a).unwrap();
    assert_eq!(o.shape(), (7, 3));
    assert!(is_orthonormal_cols(&o, 1e-13));
    // As colunas de A estão no espaço gerado por O: (I − OOᵀ)A = 0.
    let proj = mm(&o, &o.transpose());
    assert!(max_abs_diff(&mm(&proj, &a), &a) < 1e-12);
}

#[test]
fn traco_e_traco_igual_soma_de_autovalores() {
    let mut rng = Rng(51);
    let g = rng.matrix(6, 6);
    let s = Matrix::from_fn(6, 6, |i, j| g.get(i, j) + g.get(j, i));
    let Eig::Symmetric { values, .. } = eig(&s).unwrap() else {
        panic!()
    };
    assert!((values.iter().sum::<f64>() - s.trace()).abs() < 1e-13);
}

// ---------- exponencial de matriz ----------

#[test]
fn expm_casos_analiticos() {
    // exp(0) = I
    assert!(max_abs_diff(&expm(&Matrix::zeros(4, 4)).unwrap(), &Matrix::identity(4)) < 1e-16);
    // Nilpotente: exp([[0,1],[0,0]]) = [[1,1],[0,1]]
    let n = Matrix::from_rows(&[&[0.0, 1.0], &[0.0, 0.0]]).unwrap();
    let en = expm(&n).unwrap();
    assert!(max_abs_diff(&en, &Matrix::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]]).unwrap()) < 1e-15);
    // Gerador de rotação: exp(t·[[0,−1],[1,0]]) = [[cos t, −sin t],[sin t, cos t]], para vários t
    // (cobre todos os graus de Padé e o escalonamento).
    for &t in &[1e-3, 0.1, 0.9, 2.0, 5.0, 30.0] {
        let g = Matrix::from_rows(&[&[0.0, -t], &[t, 0.0]]).unwrap();
        let e = expm(&g).unwrap();
        let exact = Matrix::from_rows(&[&[t.cos(), -t.sin()], &[t.sin(), t.cos()]]).unwrap();
        assert!(
            max_abs_diff(&e, &exact) < 1e-13 * t.max(1.0),
            "t={t}: {}",
            max_abs_diff(&e, &exact)
        );
    }
    // Diagonal: exp elemento a elemento, inclusive valores grandes (erro relativo).
    for &d in &[-20.0, -1.0, 0.5, 3.0, 50.0] {
        let a = Matrix::from_fn(2, 2, |i, j| if i == j { d * (i + 1) as f64 } else { 0.0 });
        let e = expm(&a).unwrap();
        for i in 0..2 {
            let exact = (d * (i + 1) as f64).exp();
            // Matriz diagonal é triangular: a diagonal é recalculada exatamente em cada etapa
            // da quadratura (Al-Mohy & Higham, 2009). Com o algoritmo de 2005, o erro relativo
            // em e⁻⁴⁰ e e¹⁰⁰ chegava a ~1e-13.
            let tol = 2e-16;
            assert!(
                ((e.get(i, i) - exact) / exact).abs() < tol,
                "d={d}: {} vs {exact}",
                e.get(i, i)
            );
        }
    }
}

#[test]
fn expm_propriedade_de_semelhanca() {
    // exp(S D S⁻¹) = S exp(D) S⁻¹
    let mut rng = Rng(61);
    let mut s = rng.matrix(5, 5);
    for i in 0..5 {
        s.set(i, i, s.get(i, i) + 4.0);
    }
    let (si, _) = inv(&s).unwrap();
    let dvals = [-1.5, -0.3, 0.2, 1.1, 2.5];
    let d = Matrix::from_fn(5, 5, |i, j| if i == j { dvals[i] } else { 0.0 });
    let ed = Matrix::from_fn(5, 5, |i, j| if i == j { dvals[i].exp() } else { 0.0 });
    let a = mm(&mm(&s, &d), &si);
    let lhs = expm(&a).unwrap();
    let rhs = mm(&mm(&s, &ed), &si);
    assert!(max_abs_diff(&lhs, &rhs) / rhs.max_abs() < 1e-12);
}

// ---------- erros ----------

#[test]
fn erros_com_identificadores_estaveis() {
    let a = Matrix::zeros(2, 3);
    let e = solve(&a, &Matrix::zeros(3, 1)).unwrap_err();
    assert_eq!(e.id(), "chimalab:linalg:dimensoesIncompativeis");
    assert_eq!(det(&a).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(inv(&a).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(eig(&a).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(expm(&a).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    let nan = Matrix::from_rows(&[&[1.0, f64::NAN], &[0.5, 1.0]]).unwrap();
    assert_eq!(
        solve(&nan, &Matrix::column(&[1.0, 1.0])).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    assert_eq!(
        svd(&nan, false).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    assert!(norm(&nan, NormKind::Two).unwrap().is_nan());
    // Mensagens legíveis
    assert!(e.to_string().contains("dimensões incompatíveis"));
}

#[test]
fn matrizes_vazias() {
    let s = solve(&Matrix::zeros(0, 3), &Matrix::zeros(0, 2)).unwrap();
    assert_eq!(s.x.shape(), (3, 2));
    assert_eq!(s.method, SolveMethod::Empty);
    assert_eq!(inv(&Matrix::zeros(0, 0)).unwrap().0.shape(), (0, 0));
    assert_eq!(rank(&Matrix::zeros(0, 4), None).unwrap(), 0);
    assert_eq!(pinv(&Matrix::zeros(2, 0), None).unwrap().shape(), (0, 2));
    assert_eq!(null_space(&Matrix::zeros(0, 3)).unwrap().shape(), (3, 3));
    assert_eq!(svd(&Matrix::zeros(3, 0), false).unwrap().s.len(), 0);
    assert_eq!(cond(&Matrix::zeros(0, 0)).unwrap(), 0.0);
}

// ---------- CA2: desempenho (medição manual) ----------

/// CA2 pede SVD 1000×1000 em tempo comparável a LAPACK otimizado. Não há referência LAPACK
/// instalada neste ambiente: o teste só mede e imprime. Executar com
/// `cargo test --release -p chimalab-math -- --ignored --nocapture ca2`.
#[test]
#[ignore = "medição de desempenho; executar manualmente em --release"]
fn ca2_tempo_svd_1000() {
    let mut rng = Rng(1000);
    let a = rng.matrix(1000, 1000);
    let t0 = std::time::Instant::now();
    let s = singular_values(&a).unwrap();
    let t_vals = t0.elapsed();
    let t1 = std::time::Instant::now();
    let f = svd(&a, false).unwrap();
    let t_full = t1.elapsed();
    assert_eq!(s.len(), 1000);
    assert_eq!(f.s.len(), 1000);
    println!("SVD 1000×1000: só valores {t_vals:?}; completa (U,S,V) {t_full:?}");
}
