//! Testes F040, segundo incremento: Hessenberg, Schur real/complexa, LDLᵀ, autovalores
//! generalizados, sqrtm e logm. Oráculos: identidades de reconstrução, propriedades
//! estruturais e casos com solução analítica.

use chimalab_math::linalg::*;
use num_complex::Complex64;
use std::f64::consts::PI;

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

fn mm(a: &Matrix, b: &Matrix) -> Matrix {
    a.matmul(b).unwrap()
}
fn diff(a: &Matrix, b: &Matrix) -> f64 {
    a.sub(b).unwrap().max_abs()
}
fn cdiff(a: &ComplexMatrix, b: &ComplexMatrix) -> f64 {
    a.sub(b).unwrap().max_abs()
}
fn orthonormal(q: &Matrix, tol: f64) -> bool {
    diff(&mm(&q.transpose(), q), &Matrix::identity(q.cols())) < tol
}
fn unitary(q: &ComplexMatrix, tol: f64) -> bool {
    cdiff(
        &q.adjoint().matmul(q).unwrap(),
        &ComplexMatrix::identity(q.cols()),
    ) < tol
}
fn diag(v: &[f64]) -> Matrix {
    Matrix::from_fn(v.len(), v.len(), |i, j| if i == j { v[i] } else { 0.0 })
}
fn rot(theta: f64) -> Matrix {
    Matrix::from_rows(&[&[theta.cos(), -theta.sin()], &[theta.sin(), theta.cos()]]).unwrap()
}
fn sorted_eigs(a: &Matrix) -> Vec<Complex64> {
    let mut v = match eig(a).unwrap() {
        Eig::Symmetric { values, .. } => values.iter().map(|&x| Complex64::new(x, 0.0)).collect(),
        Eig::General { values, .. } => values,
    };
    v.sort_by(|x, y| x.re.total_cmp(&y.re).then(x.im.total_cmp(&y.im)));
    v
}

// ---------- Hessenberg ----------

#[test]
fn hessenberg_reconstroi_e_tem_estrutura() {
    let mut rng = Rng(1);
    for n in [1usize, 2, 3, 6, 17, 40] {
        let a = rng.matrix(n, n);
        let f = hess(&a).unwrap();
        assert!(orthonormal(&f.p, 1e-13), "n={n}");
        for j in 0..n {
            for i in j + 2..n {
                assert_eq!(f.h.get(i, j), 0.0);
            }
        }
        assert!(diff(&mm(&mm(&f.p, &f.h), &f.p.transpose()), &a) < 1e-13, "n={n}");
    }
}

// ---------- Schur real ----------

fn check_real_schur(a: &Matrix, tol: f64) {
    let n = a.rows();
    let s = schur(a).unwrap();
    assert!(orthonormal(&s.u, tol), "U não ortogonal");
    assert!(
        diff(&mm(&mm(&s.u, &s.t), &s.u.transpose()), a) < tol * a.max_abs().max(1.0),
        "reconstrução"
    );
    // Quase triangular: nada abaixo da subdiagonal; subdiagonal não nula só em blocos 2×2
    // isolados e com autovalores complexos.
    for j in 0..n {
        for i in j + 2..n {
            assert_eq!(s.t.get(i, j), 0.0);
        }
    }
    for k in 0..n.saturating_sub(1) {
        if s.t.get(k + 1, k) != 0.0 {
            assert!(
                k + 2 >= n || s.t.get(k + 2, k + 1) == 0.0,
                "blocos 2×2 consecutivos em {k}"
            );
            let (a11, a12, a21, a22) = (
                s.t.get(k, k),
                s.t.get(k, k + 1),
                s.t.get(k + 1, k),
                s.t.get(k + 1, k + 1),
            );
            let disc = 0.25 * (a11 - a22).powi(2) + a12 * a21;
            assert!(disc < 0.0, "bloco 2×2 com autovalores reais em {k}");
        }
    }
}

#[test]
fn schur_real_em_matrizes_aleatorias() {
    let mut rng = Rng(2);
    for n in [1usize, 2, 3, 4, 7, 20, 60] {
        check_real_schur(&rng.matrix(n, n), 1e-12);
    }
}

#[test]
fn schur_real_de_casos_especiais() {
    // Simétrica: T diagonal.
    let mut rng = Rng(3);
    let g = rng.matrix(8, 8);
    let s = Matrix::from_fn(8, 8, |i, j| g.get(i, j) + g.get(j, i));
    check_real_schur(&s, 1e-12);
    // Já triangular, Jordan (defectiva), nula, identidade.
    check_real_schur(
        &Matrix::from_rows(&[&[2.0, 1.0, 0.0], &[0.0, 2.0, 1.0], &[0.0, 0.0, 2.0]]).unwrap(),
        1e-13,
    );
    check_real_schur(&Matrix::zeros(4, 4), 1e-15);
    check_real_schur(&Matrix::identity(5), 1e-15);
    // Bloco de rotação: autovalores e^{±iθ}.
    check_real_schur(&rot(0.7), 1e-14);
    // Matriz de permutação cíclica (autovalores = raízes da unidade, difícil para QR sem
    // deslocamento excepcional).
    let p = Matrix::from_fn(6, 6, |i, j| if (i + 1) % 6 == j { 1.0 } else { 0.0 });
    check_real_schur(&p, 1e-13);
    // Dez ciclos de 3 em blocos: sem o deslocamento excepcional a cada 11 iterações, o QR estagna
    // e esgota o limite de iterações (conferido trocando a agenda).
    let c3 = Matrix::from_fn(30, 30, |i, j| {
        if i / 3 == j / 3 && (i + 1) % 3 == j % 3 {
            1.0
        } else {
            0.0
        }
    });
    check_real_schur(&c3, 1e-13);
}

#[test]
fn schur_complexa_reconstroi_e_diagonal_sao_autovalores() {
    let mut rng = Rng(4);
    for n in [1usize, 2, 5, 12, 30] {
        let a = rng.matrix(n, n);
        let s = schur_complex(&a).unwrap();
        assert!(unitary(&s.u, 1e-12), "n={n}");
        for j in 0..n {
            for i in j + 1..n {
                assert_eq!(s.t.get(i, j), Complex64::new(0.0, 0.0));
            }
        }
        let back = s.u.matmul(&s.t).unwrap().matmul(&s.u.adjoint()).unwrap();
        assert!(cdiff(&back, &ComplexMatrix::from_real(&a)) < 1e-12, "n={n}");
        // Pareia cada elemento da diagonal com o autovalor mais próximo ainda não usado
        // (ordenar falha quando pares conjugados têm partes reais iguais até o arredondamento).
        let mut e = sorted_eigs(&a);
        for i in 0..n {
            let x = s.t.get(i, i);
            let (k, dist) = e
                .iter()
                .enumerate()
                .map(|(k, y)| (k, (x - y).norm()))
                .fold((0, f64::INFINITY), |acc, c| if c.1 < acc.1 { c } else { acc });
            assert!(dist < 1e-10, "n={n}: {x} sem autovalor correspondente");
            e.remove(k);
        }
    }
}

// ---------- LDLᵀ ----------

#[test]
fn ldl_reconstroi_matriz_simetrica_indefinida() {
    let mut rng = Rng(5);
    let g = rng.matrix(9, 9);
    let a = Matrix::from_fn(9, 9, |i, j| g.get(i, j) + g.get(j, i));
    let cases = [
        a,
        Matrix::from_rows(&[&[0.0, 1.0], &[1.0, 0.0]]).unwrap(), // exige pivô 2×2
        diag(&[3.0, -2.0, 5.0]),
    ];
    for a in cases {
        let f = ldl(&a).unwrap();
        let n = a.rows();
        assert!(f.l.is_lower_triangular());
        for i in 0..n {
            assert_eq!(f.l.get(i, i), 1.0);
        }
        assert!(f.d.is_symmetric());
        let pap = mm(&mm(&f.p_matrix(), &a), &f.p_matrix().transpose());
        let ldlt = mm(&mm(&f.l, &f.d), &f.l.transpose());
        assert!(diff(&pap, &ldlt) < 1e-13, "{a:?}");
    }
}

// ---------- autovalores generalizados ----------

#[test]
fn geig_satisfaz_a_v_igual_lambda_b_v() {
    let mut rng = Rng(6);
    let n = 7;
    let a = rng.matrix(n, n);
    let mut b = rng.matrix(n, n);
    for i in 0..n {
        b.set(i, i, b.get(i, i) + 3.0);
    }
    let g = geig(&a, &b).unwrap();
    let (ac, bc) = (ComplexMatrix::from_real(&a), ComplexMatrix::from_real(&b));
    let av = ac.matmul(&g.vectors).unwrap();
    let bv = bc.matmul(&g.vectors).unwrap();
    for j in 0..n {
        for i in 0..n {
            // β·A·v = α·B·v (forma que vale também para autovalores infinitos)
            let r = av.get(i, j) * g.beta[j] - bv.get(i, j) * g.alpha[j];
            assert!(r.norm() < 1e-12, "({i},{j}): {r}");
        }
    }
}

#[test]
fn geig_com_b_identidade_e_com_b_singular() {
    let a = Matrix::from_rows(&[&[2.0, 0.0], &[0.0, 5.0]]).unwrap();
    let g = geig(&a, &Matrix::identity(2)).unwrap();
    let mut v: Vec<f64> = g.values.iter().map(|z| z.re).collect();
    v.sort_by(f64::total_cmp);
    assert!((v[0] - 2.0).abs() < 1e-14 && (v[1] - 5.0).abs() < 1e-14);
    // A = I, B = diag(1, 0): λ = 1 e λ = ∞
    let g = geig(&Matrix::identity(2), &diag(&[1.0, 0.0])).unwrap();
    assert_eq!(g.values.iter().filter(|z| z.re.is_infinite()).count(), 1);
    assert!(g.values.iter().any(|z| (z - 1.0).norm() < 1e-14));
}

// ---------- sqrtm ----------

fn square_complex(x: &ComplexMatrix) -> ComplexMatrix {
    x.matmul(x).unwrap()
}

#[test]
fn sqrtm_de_spd_e_real_e_eleva_ao_quadrado() {
    let mut rng = Rng(7);
    let g = rng.matrix(6, 6);
    let mut a = mm(&g.transpose(), &g);
    for i in 0..6 {
        a.set(i, i, a.get(i, i) + 1.0);
    }
    let r = sqrtm(&a).unwrap();
    let RealOrComplex::Real(x) = r.value else {
        panic!("esperado real")
    };
    assert!(diff(&mm(&x, &x), &a) < 1e-12);
    assert!(r.warnings.is_empty());
    // Raiz principal de SPD é SPD: autovalores positivos.
    for z in sorted_eigs(&x) {
        assert!(z.re > 0.0);
    }
}

#[test]
fn sqrtm_casos_analiticos() {
    let RealOrComplex::Real(x) = sqrtm(&diag(&[4.0, 9.0, 0.25])).unwrap().value else {
        panic!()
    };
    assert!(diff(&x, &diag(&[2.0, 3.0, 0.5])) < 1e-15);
    // Raiz de rotação de θ é rotação de θ/2 (θ ∈ (−π, π)).
    for &th in &[0.3, 1.5, 2.9] {
        let RealOrComplex::Real(x) = sqrtm(&rot(th)).unwrap().value else {
            panic!("θ={th}")
        };
        assert!(diff(&x, &rot(th / 2.0)) < 1e-14, "θ={th}");
    }
    // Autovalor negativo: resultado complexo, diag(2i, 3).
    let r = sqrtm(&diag(&[-4.0, 9.0])).unwrap();
    let RealOrComplex::Complex(x) = r.value else {
        panic!("esperado complexo")
    };
    assert!((x.get(0, 0) - Complex64::new(0.0, 2.0)).norm() < 1e-15);
    assert!((x.get(1, 1) - 3.0).norm() < 1e-15);
    // Matriz nilpotente [[0,1],[0,0]] não tem raiz quadrada: aviso.
    let r = sqrtm(&Matrix::from_rows(&[&[0.0, 1.0], &[0.0, 0.0]]).unwrap()).unwrap();
    assert_eq!(r.warnings, vec![LinalgWarning::Singular]);
}

#[test]
fn sqrtm_aleatoria_ao_quadrado_reproduz_a() {
    let mut rng = Rng(8);
    for n in [3usize, 8, 20] {
        let a = rng.matrix(n, n);
        let x = sqrtm(&a).unwrap().value.to_complex();
        assert!(
            cdiff(&square_complex(&x), &ComplexMatrix::from_real(&a)) < 1e-11,
            "n={n}"
        );
    }
}

// ---------- logm ----------

#[test]
fn logm_casos_analiticos() {
    let e = std::f64::consts::E;
    let RealOrComplex::Real(l) = logm(&diag(&[e, e * e, 1.0])).unwrap().value else {
        panic!()
    };
    assert!(diff(&l, &diag(&[1.0, 2.0, 0.0])) < 1e-14);
    // log de rotação θ = [[0, −θ], [θ, 0]]
    for &th in &[0.2, 1.0, 3.0] {
        let RealOrComplex::Real(l) = logm(&rot(th)).unwrap().value else {
            panic!("θ={th}")
        };
        let exact = Matrix::from_rows(&[&[0.0, -th], &[th, 0.0]]).unwrap();
        assert!(diff(&l, &exact) < 1e-13, "θ={th}: {}", diff(&l, &exact));
    }
    // log(−I) = iπ·I (ramo principal)
    let RealOrComplex::Complex(l) = logm(&diag(&[-1.0, -1.0])).unwrap().value else {
        panic!()
    };
    assert!((l.get(0, 0) - Complex64::new(0.0, PI)).norm() < 1e-14);
    // Jordan: log([[1,1],[0,1]]) = [[0,1],[0,0]]
    let RealOrComplex::Real(l) = logm(&Matrix::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]]).unwrap())
        .unwrap()
        .value
    else {
        panic!()
    };
    assert!(diff(&l, &Matrix::from_rows(&[&[0.0, 1.0], &[0.0, 0.0]]).unwrap()) < 1e-15);
    // Singular: aviso
    assert_eq!(
        logm(&diag(&[0.0, 1.0])).unwrap().warnings,
        vec![LinalgWarning::Singular]
    );
}

#[test]
fn logm_e_expm_sao_inversas() {
    let mut rng = Rng(9);
    for n in [2usize, 5, 12] {
        // X com espectro pequeno: logm(expm(X)) = X
        let x = rng.matrix(n, n).scale(0.5);
        let RealOrComplex::Real(l) = logm(&expm(&x).unwrap()).unwrap().value else {
            panic!("n={n}")
        };
        assert!(diff(&l, &x) < 1e-12, "n={n}: {}", diff(&l, &x));
        // A com autovalores positivos: expm(logm(A)) = A
        let g = rng.matrix(n, n);
        let mut a = mm(&g.transpose(), &g);
        for i in 0..n {
            a.set(i, i, a.get(i, i) + 0.5);
        }
        let RealOrComplex::Real(l) = logm(&a).unwrap().value else {
            panic!("n={n}")
        };
        let back = expm(&l).unwrap();
        assert!(diff(&back, &a) / a.max_abs() < 1e-12, "n={n}");
    }
}

#[test]
fn erros_de_dimensao() {
    let r = Matrix::zeros(2, 3);
    assert_eq!(schur(&r).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(hess(&r).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(sqrtm(&r).unwrap_err().id(), "chimalab:linalg:matrizNaoQuadrada");
    assert_eq!(
        geig(&Matrix::identity(2), &Matrix::identity(3)).unwrap_err().id(),
        "chimalab:linalg:dimensoesIncompativeis"
    );
}

#[test]
fn schur_sqrtm_e_logm_em_matrizes_maiores() {
    // Tamanhos em que cada varredura percorre muitas colunas (antes, o maior caso era n = 60).
    let mut rng = Rng(21);
    check_real_schur(&rng.matrix(150, 150), 1e-11);
    let n = 100;
    let c = ComplexMatrix::from_fn(n, n, |_, _| Complex64::new(rng.next(), rng.next()));
    let s = schur_complex(&c).unwrap();
    assert!(unitary(&s.u, 1e-12));
    for j in 0..n {
        for i in j + 1..n {
            assert_eq!(s.t.get(i, j), Complex64::new(0.0, 0.0));
        }
    }
    let back = s.u.matmul(&s.t).unwrap().matmul(&s.u.adjoint()).unwrap();
    assert!(cdiff(&back, &c) < 1e-12);
    // sqrtm: X² = A, com A = GᵀG/n + I (espectro em [1, ~5]).
    let g = rng.matrix(n, n);
    let a = mm(&g.transpose(), &g)
        .scale(1.0 / n as f64)
        .add(&Matrix::identity(n))
        .unwrap();
    let x = sqrtm(&a).unwrap().value.to_complex();
    let err = cdiff(&square_complex(&x), &ComplexMatrix::from_real(&a)) / a.max_abs();
    assert!(err < 1e-13, "sqrtm: {err:e}");
    // logm(expm(X)) = X com espectro pequeno.
    let x = rng.matrix(n, n).scale(0.5 / (n as f64).sqrt());
    let RealOrComplex::Real(l) = logm(&expm(&x).unwrap()).unwrap().value else {
        panic!("logm de matriz real com espectro positivo deveria ser real")
    };
    assert!(diff(&l, &x) < 1e-12, "logm: {:e}", diff(&l, &x));
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    /// Matriz n×n: densa aleatória (`kind` 0, com escala 2^e) ou de inteiros pequenos com
    /// metade dos elementos nulos (`kind` 1: provoca deflações e blocos 2×2 com autovalores
    /// reais ou repetidos).
    fn matriz(n: usize, seed: u64, kind: u8, e: i32) -> Matrix {
        let mut rng = Rng(seed | 1);
        Matrix::from_fn(n, n, |_, _| {
            let v = rng.next();
            if kind == 0 {
                v * 2f64.powi(e)
            } else if v.abs() < 0.5 {
                0.0
            } else {
                (v * 4.0).round()
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// U ortogonal, A = U·T·Uᵀ e T quase triangular com blocos 2×2 só para pares complexos.
        #[test]
        fn schur_real_reconstroi(n in 1usize..40, seed: u64, kind in 0u8..2, e in -30i32..30) {
            check_real_schur(&matriz(n, seed, kind, e), 1e-11);
        }

        /// Entrada complexa (QR complexo): U unitária, T triangular, A = U·T·Uᴴ.
        #[test]
        fn schur_complexa_reconstroi(n in 1usize..30, seed: u64, kind in 0u8..2) {
            let re = matriz(n, seed, kind, 0);
            let im = matriz(n, seed.wrapping_add(1), kind, 0);
            let a = ComplexMatrix::from_fn(n, n, |i, j| Complex64::new(re.get(i, j), im.get(i, j)));
            let s = schur_complex(&a).unwrap();
            prop_assert!(unitary(&s.u, 1e-12));
            for j in 0..n {
                for i in j + 1..n {
                    prop_assert_eq!(s.t.get(i, j), Complex64::new(0.0, 0.0));
                }
            }
            let back = s.u.matmul(&s.t).unwrap().matmul(&s.u.adjoint()).unwrap();
            prop_assert!(cdiff(&back, &a) < 1e-12 * a.max_abs().max(1.0));
        }

        /// sqrtm de matriz com espectro no semiplano direito: X² = A.
        #[test]
        fn sqrtm_ao_quadrado_reproduz(n in 1usize..30, seed: u64) {
            let g = matriz(n, seed, 0, 0);
            let a = mm(&g.transpose(), &g).add(&Matrix::identity(n)).unwrap();
            let x = sqrtm(&a).unwrap().value.to_complex();
            let err = cdiff(&square_complex(&x), &ComplexMatrix::from_real(&a)) / a.max_abs();
            prop_assert!(err < 1e-13, "{:e}", err);
        }
    }
}

/// Medição manual: `cargo test --release -p chimalab-math -- --ignored --nocapture tempo_schur`.
#[test]
#[ignore = "medição de desempenho; executar manualmente em --release"]
fn tempo_schur() {
    let mut rng = Rng(300);
    for n in [100usize, 300, 600] {
        let a = rng.matrix(n, n);
        let t0 = std::time::Instant::now();
        let s = schur(&a).unwrap();
        let t_schur = t0.elapsed();
        let t1 = std::time::Instant::now();
        let _ = eig(&a).unwrap();
        let t_eig = t1.elapsed();
        assert_eq!(s.t.rows(), n);
        let c = ComplexMatrix::from_fn(n, n, |_, _| Complex64::new(rng.next(), rng.next()));
        let t2 = std::time::Instant::now();
        let sc = schur_complex(&c).unwrap();
        let t_complex = t2.elapsed();
        assert_eq!(sc.t.rows(), n);
        println!(
            "n={n}: schur (Francis próprio) {t_schur:?}; eig (faer) {t_eig:?}; schur complexa {t_complex:?}"
        );
    }
}
