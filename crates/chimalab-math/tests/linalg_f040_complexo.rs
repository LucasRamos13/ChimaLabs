//! F040 com entradas complexas (DT-18). Oráculos: reconstruções com a adjunta, propriedades
//! unitárias/hermitianas, condições de Moore-Penrose e casos analíticos.

use chimalab_math::linalg::*;
use num_complex::Complex64 as C;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
    fn matrix(&mut self, m: usize, n: usize) -> ComplexMatrix {
        ComplexMatrix::from_fn(m, n, |_, _| C::new(self.next(), self.next()))
    }
}

fn c(re: f64, im: f64) -> C {
    C::new(re, im)
}
fn mm(a: &ComplexMatrix, b: &ComplexMatrix) -> ComplexMatrix {
    a.matmul(b).unwrap()
}
fn diff(a: &ComplexMatrix, b: &ComplexMatrix) -> f64 {
    a.sub(b).unwrap().max_abs()
}
fn unitary(q: &ComplexMatrix, tol: f64) -> bool {
    diff(&mm(&q.adjoint(), q), &ComplexMatrix::identity(q.cols())) < tol
}
fn hpd(rng: &mut Rng, n: usize) -> ComplexMatrix {
    let g = rng.matrix(n, n);
    let mut a = mm(&g.adjoint(), &g);
    for i in 0..n {
        a.set(i, i, a.get(i, i) + n as f64);
    }
    // Hermitiana exata
    ComplexMatrix::from_fn(n, n, |i, j| match i.cmp(&j) {
        std::cmp::Ordering::Less => a.get(i, j),
        std::cmp::Ordering::Equal => c(a.get(i, i).re, 0.0),
        std::cmp::Ordering::Greater => a.get(j, i).conj(),
    })
}

#[test]
fn solve_complexo_escolhe_metodo_e_tem_residuo_pequeno() {
    let mut rng = Rng(1);
    let n = 30;
    let b = rng.matrix(n, 2);
    let mut a = rng.matrix(n, n);
    for i in 0..n {
        a.set(i, i, a.get(i, i) + n as f64);
    }
    let s = solve(&a, &b).unwrap();
    assert_eq!(s.method, SolveMethod::Lu);
    assert!(diff(&mm(&a, &s.x), &b) < 1e-12);
    let h = hpd(&mut rng, n);
    let s = solve(&h, &b).unwrap();
    assert_eq!(s.method, SolveMethod::Cholesky);
    assert!(diff(&mm(&h, &s.x), &b) / h.max_abs() < 1e-13);
    let up = ComplexMatrix::from_fn(n, n, |i, j| if i <= j { a.get(i, j) } else { c(0.0, 0.0) });
    let s = solve(&up, &b).unwrap();
    assert_eq!(s.method, SolveMethod::UpperTriangular);
    assert!(diff(&mm(&up, &s.x), &b) < 1e-12);
}

#[test]
fn minimos_quadrados_complexos_satisfazem_equacoes_normais() {
    let mut rng = Rng(2);
    let a = rng.matrix(20, 5);
    let b = rng.matrix(20, 1);
    let s = solve(&a, &b).unwrap();
    assert_eq!(s.method, SolveMethod::LeastSquaresQr);
    // Aᴴ(Ax − b) = 0
    let g = mm(&a.adjoint(), &mm(&a, &s.x).sub(&b).unwrap());
    assert!(g.max_abs() < 1e-12, "{}", g.max_abs());
}

#[test]
fn singular_complexo_gera_aviso() {
    let a = ComplexMatrix::from_rows(&[&[c(1.0, 1.0), c(2.0, 2.0)], &[c(1.0, 0.0), c(2.0, 0.0)]]).unwrap();
    let s = solve(&a, &ComplexMatrix::column(&[c(1.0, 0.0), c(0.0, 1.0)])).unwrap();
    assert!(!s.warnings.is_empty());
    assert!(rcond(&a).unwrap() < f64::EPSILON);
}

#[test]
fn inversa_e_determinante_complexos() {
    // det([[i, 2],[3, 4i]]) = i·4i − 6 = −4 − 6 = −10
    let a = ComplexMatrix::from_rows(&[&[c(0.0, 1.0), c(2.0, 0.0)], &[c(3.0, 0.0), c(0.0, 4.0)]]).unwrap();
    assert!((det(&a).unwrap() - c(-10.0, 0.0)).norm() < 1e-14);
    let mut rng = Rng(3);
    let x = rng.matrix(6, 6);
    let y = rng.matrix(6, 6);
    let (dx, dy) = (det(&x).unwrap(), det(&y).unwrap());
    assert!((det(&mm(&x, &y)).unwrap() - dx * dy).norm() < 1e-12 * (dx * dy).norm().max(1.0));
    let (xi, w) = inv(&x).unwrap();
    assert!(w.is_empty());
    assert!(diff(&mm(&x, &xi), &ComplexMatrix::identity(6)) < 1e-12);
}

#[test]
fn decomposicoes_complexas_reconstroem() {
    let mut rng = Rng(4);
    for &(m, n) in &[(5usize, 5usize), (7, 3), (3, 7)] {
        let a = rng.matrix(m, n);
        let f = lu(&a).unwrap();
        assert!(diff(&mm(&f.p_matrix(), &a), &mm(&f.l, &f.u)) < 1e-13);
        let f = qr(&a).unwrap();
        assert!(unitary(&f.q, 1e-13));
        assert!(f.r.is_upper_triangular());
        assert!(diff(&mm(&f.q, &f.r), &a) < 1e-13);
        let f = svd(&a, false).unwrap();
        assert!(unitary(&f.u, 1e-12) && unitary(&f.v, 1e-12));
        let k = m.min(n);
        let us = ComplexMatrix::from_fn(m, k, |i, j| f.u.get(i, j) * f.s[j]);
        let vk = f.v.submatrix(0, n, 0, k);
        assert!(diff(&mm(&us, &vk.adjoint()), &a) < 1e-12, "svd ({m},{n})");
        assert!(f.s.windows(2).all(|w| w[0] >= w[1]));
    }
    let h = hpd(&mut rng, 6);
    let r = chol(&h).unwrap().r;
    assert!(r.is_upper_triangular());
    assert!(diff(&mm(&r.adjoint(), &r), &h) / h.max_abs() < 1e-14);
}

#[test]
fn autovalores_complexos() {
    // Hermitiana [[0, i], [−i, 0]]: autovalores ±1 reais.
    let h = ComplexMatrix::from_rows(&[&[c(0.0, 0.0), c(0.0, 1.0)], &[c(0.0, -1.0), c(0.0, 0.0)]]).unwrap();
    let Eig::Symmetric { values, vectors } = eig(&h).unwrap() else {
        panic!("hermitiana")
    };
    assert!((values[0] + 1.0).abs() < 1e-15 && (values[1] - 1.0).abs() < 1e-15);
    assert!(unitary(&vectors, 1e-14));
    // Triangular [[1, i], [0, 2]]: autovalores 1 e 2.
    let t = ComplexMatrix::from_rows(&[&[c(1.0, 0.0), c(0.0, 1.0)], &[c(0.0, 0.0), c(2.0, 0.0)]]).unwrap();
    let Eig::General { mut values, .. } = eig(&t).unwrap() else {
        panic!("geral")
    };
    values.sort_by(|a, b| a.re.total_cmp(&b.re));
    assert!((values[0] - 1.0).norm() < 1e-14 && (values[1] - 2.0).norm() < 1e-14);
    // Aleatória: A·V = V·Λ
    let mut rng = Rng(5);
    let a = rng.matrix(8, 8);
    let Eig::General { values, vectors } = eig(&a).unwrap() else {
        panic!()
    };
    let av = mm(&a, &vectors);
    for (j, l) in values.iter().enumerate() {
        for i in 0..8 {
            assert!((av.get(i, j) - vectors.get(i, j) * l).norm() < 1e-12);
        }
    }
}

#[test]
fn normas_posto_e_pinv_complexos() {
    let v = ComplexMatrix::column(&[c(3.0, 4.0), c(0.0, 0.0)]);
    assert_eq!(norm(&v, NormKind::Two).unwrap(), 5.0);
    assert_eq!(norm(&v, NormKind::One).unwrap(), 5.0);
    let mut rng = Rng(6);
    let b = rng.matrix(6, 2);
    let d = rng.matrix(2, 4);
    let a = mm(&b, &d); // posto 2
    assert_eq!(rank(&a, None).unwrap(), 2);
    let p = pinv(&a, None).unwrap();
    assert!(diff(&mm(&mm(&a, &p), &a), &a) < 1e-12);
    assert!(diff(&mm(&mm(&p, &a), &p), &p) < 1e-12);
    let ap = mm(&a, &p);
    assert!(diff(&ap, &ap.adjoint()) < 1e-12);
    let z = null_space(&a).unwrap();
    assert_eq!(z.shape(), (4, 2));
    assert!(mm(&a, &z).max_abs() < 1e-12);
    let q = qr(&rng.matrix(5, 5)).unwrap().q;
    assert!((cond(&q).unwrap() - 1.0).abs() < 1e-13);
}

#[test]
fn expm_complexa() {
    // exp(iθ·I) = e^{iθ}·I
    let th = 0.7;
    let a = ComplexMatrix::from_fn(3, 3, |i, j| if i == j { c(0.0, th) } else { c(0.0, 0.0) });
    let e = expm(&a).unwrap();
    assert!(
        diff(
            &e,
            &ComplexMatrix::identity(3).map(|z| z * C::from_polar(1.0, th))
        ) < 1e-15
    );
    // exp(i·N) com N nilpotente = I + i·N
    let n = ComplexMatrix::from_rows(&[&[c(0.0, 0.0), c(0.0, 1.0)], &[c(0.0, 0.0), c(0.0, 0.0)]]).unwrap();
    let e = expm(&n).unwrap();
    assert!(diff(&e, &ComplexMatrix::identity(2).add(&n).unwrap()) < 1e-15);
    // Diagonal com módulo grande (escalonamento)
    let d = ComplexMatrix::from_fn(2, 2, |i, j| {
        if i == j {
            c(10.0 * (i + 1) as f64, 3.0)
        } else {
            c(0.0, 0.0)
        }
    });
    let e = expm(&d).unwrap();
    for i in 0..2 {
        let exact = c(10.0 * (i + 1) as f64, 3.0).exp();
        assert!(((e.get(i, i) - exact) / exact).norm() < 1e-12);
    }
}

#[test]
fn hessenberg_ldl_e_geig_complexos() {
    let mut rng = Rng(7);
    let a = rng.matrix(9, 9);
    let f = hess(&a).unwrap();
    assert!(unitary(&f.p, 1e-13));
    assert!(diff(&mm(&mm(&f.p, &f.h), &f.p.adjoint()), &a) < 1e-13);
    for j in 0..9 {
        for i in j + 2..9 {
            assert_eq!(f.h.get(i, j), c(0.0, 0.0));
        }
    }
    // LDLᴴ de hermitiana indefinida
    let g = rng.matrix(6, 6);
    let h = ComplexMatrix::from_fn(6, 6, |i, j| g.get(i, j) + g.get(j, i).conj());
    let h = ComplexMatrix::from_fn(6, 6, |i, j| {
        if i == j {
            c(h.get(i, i).re, 0.0)
        } else {
            h.get(i, j)
        }
    });
    let f = ldl(&h).unwrap();
    let p = f.p_matrix();
    let lhs = mm(&mm(&p, &h), &p.adjoint());
    let rhs = mm(&mm(&f.l, &f.d), &f.l.adjoint());
    assert!(diff(&lhs, &rhs) < 1e-13, "{}", diff(&lhs, &rhs));
    // Generalizados
    let a = rng.matrix(5, 5);
    let mut b = rng.matrix(5, 5);
    for i in 0..5 {
        b.set(i, i, b.get(i, i) + 3.0);
    }
    let gz = geig(&a, &b).unwrap();
    let av = mm(&a, &gz.vectors);
    let bv = mm(&b, &gz.vectors);
    for j in 0..5 {
        for i in 0..5 {
            assert!((av.get(i, j) * gz.beta[j] - bv.get(i, j) * gz.alpha[j]).norm() < 1e-12);
        }
    }
}

fn check_complex_schur(a: &ComplexMatrix) {
    let n = a.rows();
    let s = schur_complex(a).unwrap();
    assert!(unitary(&s.u, 1e-12), "n={n}");
    for j in 0..n {
        for i in j + 1..n {
            assert_eq!(s.t.get(i, j), c(0.0, 0.0));
        }
    }
    let back = mm(&mm(&s.u, &s.t), &s.u.adjoint());
    assert!(
        diff(&back, a) < 1e-12 * a.max_abs().max(1.0),
        "n={n}: {}",
        diff(&back, a)
    );
    // Diagonal = autovalores (pareamento pelo mais próximo)
    let mut e = match eig(a).unwrap() {
        Eig::General { values, .. } => values,
        Eig::Symmetric { values, .. } => values.iter().map(|&v| c(v, 0.0)).collect(),
    };
    for i in 0..n {
        let x = s.t.get(i, i);
        let (k, d) = e
            .iter()
            .enumerate()
            .map(|(k, y)| (k, (x - y).norm()))
            .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
        assert!(d < 1e-9 * a.max_abs().max(1.0), "n={n}: {x}");
        e.remove(k);
    }
}

#[test]
fn schur_de_matriz_complexa() {
    let mut rng = Rng(8);
    for n in [1usize, 2, 3, 5, 12, 30, 60] {
        check_complex_schur(&rng.matrix(n, n));
    }
    check_complex_schur(&hpd(&mut rng, 7));
    check_complex_schur(&ComplexMatrix::zeros(4, 4));
    // Jordan complexo e permutação cíclica multiplicada por i
    check_complex_schur(
        &ComplexMatrix::from_rows(&[&[c(1.0, 1.0), c(1.0, 0.0)], &[c(0.0, 0.0), c(1.0, 1.0)]]).unwrap(),
    );
    check_complex_schur(&ComplexMatrix::from_fn(6, 6, |i, j| {
        if (i + 1) % 6 == j {
            c(0.0, 1.0)
        } else {
            c(0.0, 0.0)
        }
    }));
}

#[test]
fn sqrtm_e_logm_de_matriz_complexa() {
    let mut rng = Rng(9);
    for n in [2usize, 5, 10] {
        let a = rng.matrix(n, n);
        let x = sqrtm(&a).unwrap();
        let RealOrComplex::Complex(x) = x.value else {
            panic!("entrada complexa ⇒ resultado complexo")
        };
        assert!(diff(&mm(&x, &x), &a) < 1e-11, "n={n}");
        // logm(expm(X)) = X para X de espectro pequeno
        let small = rng.matrix(n, n).scale(0.4);
        let RealOrComplex::Complex(l) = logm(&expm(&small).unwrap()).unwrap().value else {
            panic!()
        };
        assert!(diff(&l, &small) < 1e-12, "n={n}: {}", diff(&l, &small));
    }
    // log(e^{iθ}·I) = iθ·I
    let th = 1.2;
    let a = ComplexMatrix::from_fn(2, 2, |i, j| {
        if i == j {
            C::from_polar(1.0, th)
        } else {
            c(0.0, 0.0)
        }
    });
    let RealOrComplex::Complex(l) = logm(&a).unwrap().value else {
        panic!()
    };
    assert!((l.get(0, 0) - c(0.0, th)).norm() < 1e-14 && l.get(0, 1).norm() < 1e-15);
}
