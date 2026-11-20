//! F040: funm (Schur-Parlett em blocos). Oráculos: expm/logm já validados, identidades
//! trigonométricas e hiperbólicas, casos com autovalores repetidos e quase repetidos.

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
    fn real(&mut self, n: usize) -> Matrix {
        Matrix::from_fn(n, n, |_, _| self.next())
    }
    fn complex(&mut self, n: usize) -> ComplexMatrix {
        ComplexMatrix::from_fn(n, n, |_, _| C::new(self.next(), self.next()))
    }
}

fn real(r: MatFunResult) -> Matrix {
    match r.value {
        RealOrComplex::Real(m) => m,
        RealOrComplex::Complex(c) => panic!("esperado real: {c:?}"),
    }
}
fn cplx(r: MatFunResult) -> ComplexMatrix {
    r.value.to_complex()
}
fn rel(a: &Matrix, b: &Matrix) -> f64 {
    a.sub(b).unwrap().max_abs() / b.max_abs().max(1.0)
}
fn crel(a: &ComplexMatrix, b: &ComplexMatrix) -> f64 {
    a.sub(b).unwrap().max_abs() / b.max_abs().max(1.0)
}

#[test]
fn funm_exp_igual_a_expm() {
    let mut rng = Rng(1);
    for n in [1usize, 2, 4, 8, 15] {
        let a = rng.real(n);
        let f = funm_elementary(&a, Elementary::Exp).unwrap();
        assert!(f.warnings.is_empty());
        assert!(rel(&real(f), &expm(&a).unwrap()) < 1e-12, "n={n}");
        let z = rng.complex(n);
        assert!(
            crel(
                &cplx(funm_elementary(&z, Elementary::Exp).unwrap()),
                &expm(&z).unwrap()
            ) < 1e-12,
            "n={n}"
        );
    }
}

#[test]
fn funm_com_autovalores_repetidos_e_proximos() {
    // Bloco de Jordan 3×3 (autovalor triplo): Parlett simples dividiria por zero.
    let j = Matrix::from_rows(&[&[2.0, 1.0, 0.0], &[0.0, 2.0, 1.0], &[0.0, 0.0, 2.0]]).unwrap();
    let e2 = 2f64.exp();
    let exact = Matrix::from_rows(&[&[e2, e2, e2 / 2.0], &[0.0, e2, e2], &[0.0, 0.0, e2]]).unwrap();
    assert!(rel(&real(funm_elementary(&j, Elementary::Exp).unwrap()), &exact) < 1e-14);
    // sin de Jordan 2×2: [[sin λ, cos λ], [0, sin λ]]
    let l = 0.7;
    let j2 = Matrix::from_rows(&[&[l, 1.0], &[0.0, l]]).unwrap();
    let exact = Matrix::from_rows(&[&[l.sin(), l.cos()], &[0.0, l.sin()]]).unwrap();
    assert!(rel(&real(funm_elementary(&j2, Elementary::Sin).unwrap()), &exact) < 1e-15);
    // Autovalores 1 e 1 + 1e-9: f(A) para triangular [[a, 1], [0, b]] tem (1,2) = (f(b) − f(a))/(b − a)
    let (a, b) = (1.0, 1.0 + 1e-9);
    let t = Matrix::from_rows(&[&[a, 1.0], &[0.0, b]]).unwrap();
    let f = real(funm_elementary(&t, Elementary::Exp).unwrap());
    // Diferença dividida ≈ exp(a)·(1 + h/2): calculada de forma estável com expm1
    let dd = a.exp() * (b - a).exp_m1() / (b - a);
    assert!((f.get(0, 1) - dd).abs() < 1e-12, "{} vs {dd}", f.get(0, 1));
    // Matriz com grupos separados e um grupo de autovalores próximos
    let mut rng = Rng(2);
    let q = qr(&rng.real(6)).unwrap().q;
    let d = Matrix::from_fn(6, 6, |i, j| {
        if i == j {
            [0.5, 0.52, 0.55, 3.0, -2.0, -2.01][i]
        } else if j == i + 1 {
            0.3
        } else {
            0.0
        }
    });
    let a = q.matmul(&d).unwrap().matmul(&q.transpose()).unwrap();
    assert!(
        rel(
            &real(funm_elementary(&a, Elementary::Exp).unwrap()),
            &expm(&a).unwrap()
        ) < 1e-12
    );
}

#[test]
fn identidades_trigonometricas_e_hiperbolicas() {
    let mut rng = Rng(3);
    let a = rng.real(7);
    let s = real(funm_elementary(&a, Elementary::Sin).unwrap());
    let c = real(funm_elementary(&a, Elementary::Cos).unwrap());
    // sin² + cos² = I
    let id = s.matmul(&s).unwrap().add(&c.matmul(&c).unwrap()).unwrap();
    assert!(rel(&id, &Matrix::identity(7)) < 1e-12);
    // cosh² − sinh² = I
    let sh = real(funm_elementary(&a, Elementary::Sinh).unwrap());
    let ch = real(funm_elementary(&a, Elementary::Cosh).unwrap());
    let id = ch.matmul(&ch).unwrap().sub(&sh.matmul(&sh).unwrap()).unwrap();
    assert!(rel(&id, &Matrix::identity(7)) < 1e-11);
    // cos(A) = (exp(iA) + exp(−iA))/2
    let ia = a.to_complex().map(|z| z * C::new(0.0, 1.0));
    let e1 = expm(&ia).unwrap();
    let e2 = expm(&ia.map(|z| -z)).unwrap();
    let cos_ref = e1.add(&e2).unwrap().scale(0.5);
    assert!(crel(&c.to_complex(), &cos_ref) < 1e-12);
}

#[test]
fn funm_log_e_funcao_definida_pelo_usuario() {
    let mut rng = Rng(4);
    let g = rng.real(5);
    let mut a = g.transpose().matmul(&g).unwrap();
    for i in 0..5 {
        a.set(i, i, a.get(i, i) + 1.0);
    }
    let l1 = real(funm_elementary(&a, Elementary::Log).unwrap());
    let RealOrComplex::Real(l2) = logm(&a).unwrap().value else {
        panic!()
    };
    assert!(rel(&l1, &l2) < 1e-11);
    // f(z) = z² via derivadas: funm deve coincidir com A·A
    let square = |z: C, k: usize| match k {
        0 => z * z,
        1 => z * 2.0,
        2 => C::new(2.0, 0.0),
        _ => C::new(0.0, 0.0),
    };
    let b = rng.real(6);
    assert!(rel(&real(funm(&b, square).unwrap()), &b.matmul(&b).unwrap()) < 1e-12);
}
