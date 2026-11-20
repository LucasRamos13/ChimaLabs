//! Padrão rigoroso (CLAUDE.md §7, camadas 4 e 5): casos de borda e caminhos de erro de
//! chimalab-math, mais regressões de lacunas encontradas por medição de cobertura.

use chimalab_array::ArrayError;
use chimalab_math::linalg::*;
use chimalab_math::poly::*;
use chimalab_math::signal::*;
use num_complex::Complex64;

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;

fn m(rows: &[&[f64]]) -> Matrix {
    Matrix::from_rows(rows).unwrap()
}

// ------------------------------------------------------------------ erros: ids e mensagens

#[test]
fn todas_as_variantes_de_erro_tem_id_e_mensagem() {
    let erros = [
        (
            LinalgError::DimensionMismatch {
                operation: "op",
                left: (1, 2),
                right: (3, 4),
            },
            "chimalab:linalg:dimensoesIncompativeis",
        ),
        (
            LinalgError::NotSquare {
                operation: "op",
                shape: (2, 3),
            },
            "chimalab:linalg:matrizNaoQuadrada",
        ),
        (
            LinalgError::NotPositiveDefinite,
            "chimalab:linalg:naoDefinidaPositiva",
        ),
        (
            LinalgError::NonFinite { operation: "op" },
            "chimalab:linalg:valoresNaoFinitos",
        ),
        (
            LinalgError::NoConvergence { operation: "op" },
            "chimalab:linalg:semConvergencia",
        ),
        (
            LinalgError::InvalidArgument {
                operation: "op",
                reason: "motivo",
            },
            "chimalab:linalg:argumentoInvalido",
        ),
    ];
    for (e, id) in erros {
        assert_eq!(e.id(), id);
        assert!(!e.to_string().is_empty());
        let _: &dyn std::error::Error = &e;
    }
    let avisos = [
        (LinalgWarning::Singular, "chimalab:linalg:matrizSingular"),
        (
            LinalgWarning::NearlySingular { rcond: 1e-20 },
            "chimalab:linalg:quaseSingular",
        ),
        (
            LinalgWarning::RankDeficient { rank: 1, tol: 1e-10 },
            "chimalab:linalg:postoDeficiente",
        ),
    ];
    for (w, id) in avisos {
        assert_eq!(w.id(), id);
        assert!(!w.to_string().is_empty());
    }
    let s = [
        (
            SignalError::ShapeMismatch {
                expected: 2,
                actual: 3,
            },
            "chimalab:signal:formaIncompativel",
        ),
        (
            SignalError::InvalidArgument("x"),
            "chimalab:signal:argumentoInvalido",
        ),
    ];
    for (e, id) in s {
        assert_eq!(e.id(), id);
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn conversao_de_erros_de_array() {
    let conv = [
        ArrayError::IncompatibleShapes {
            operation: "x",
            left: vec![2, 3, 4],
            right: vec![5],
        },
        ArrayError::ShapeMismatch {
            expected: 4,
            actual: 3,
        },
        ArrayError::InvalidPermutation(vec![0, 0]),
        ArrayError::IndexOutOfBounds {
            index: vec![9],
            shape: vec![2, 2],
        },
    ];
    for e in conv {
        let l: LinalgError = e.into();
        assert_eq!(l.id(), "chimalab:linalg:dimensoesIncompativeis");
    }
    let l: LinalgError = ArrayError::IncompatibleShapes {
        operation: "x",
        left: vec![2, 3, 4],
        right: vec![5, 1],
    }
    .into();
    assert_eq!(
        l,
        LinalgError::DimensionMismatch {
            operation: "x",
            left: (2, 12),
            right: (5, 1)
        }
    );
}

#[test]
fn matmul_bordas() {
    let a = Matrix::zeros(2, 3);
    assert_eq!(
        a.matmul(&Matrix::zeros(2, 2)).unwrap_err().id(),
        "chimalab:linalg:dimensoesIncompativeis"
    );
    assert_eq!(
        Matrix::zeros(2, 0).matmul(&Matrix::zeros(0, 4)).unwrap(),
        Matrix::zeros(2, 4)
    );
    let f = m(&[&[1.0, 2.0], &[3.0, 4.0]]).to_faer();
    assert_eq!(f[(1, 0)], 3.0);
}

// ------------------------------------------------------------------ decomposições

#[test]
fn decomposicoes_com_entradas_vazias_e_invalidas() {
    let e = Matrix::zeros(0, 0);
    let r = Matrix::zeros(3, 0);
    assert_eq!(lu(&r).unwrap().l.shape(), (3, 0));
    assert_eq!(qr(&r).unwrap().q, Matrix::identity(3));
    assert_eq!(chol(&e).unwrap().r.shape(), (0, 0));
    assert_eq!(svd(&r, true).unwrap().u.shape(), (3, 0));
    assert_eq!(svd(&r, false).unwrap().u.shape(), (3, 3));
    assert!(singular_values(&r).unwrap().is_empty());
    assert!(matches!(eig(&e).unwrap(), Eig::Symmetric { values, .. } if values.is_empty()));
    let nan = m(&[&[1.0, NAN], &[0.0, 1.0]]);
    for err in [
        lu(&nan).err(),
        qr(&nan).err(),
        chol(&nan).err(),
        eig(&nan).err(),
        svd(&nan, true).err(),
    ] {
        assert_eq!(err.unwrap().id(), "chimalab:linalg:valoresNaoFinitos");
    }
    assert_eq!(
        singular_values(&nan).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    assert_eq!(
        chol(&Matrix::zeros(2, 3)).unwrap_err().id(),
        "chimalab:linalg:matrizNaoQuadrada"
    );
    assert_eq!(det(&nan).unwrap_err().id(), "chimalab:linalg:valoresNaoFinitos");
}

#[test]
fn schur_hess_ldl_geig_bordas() {
    let e = Matrix::zeros(0, 0);
    assert_eq!(schur(&e).unwrap().t.shape(), (0, 0));
    assert_eq!(schur_complex(&e).unwrap().t.shape(), (0, 0));
    assert_eq!(
        schur_complex(&ComplexMatrix::zeros(0, 0)).unwrap().t.shape(),
        (0, 0)
    );
    assert_eq!(ldl(&e).unwrap().l.shape(), (0, 0));
    assert!(geig(&e, &e).unwrap().values.is_empty());
    let nan = m(&[&[NAN]]);
    assert_eq!(schur(&nan).unwrap_err().id(), "chimalab:linalg:valoresNaoFinitos");
    assert_eq!(hess(&nan).unwrap_err().id(), "chimalab:linalg:valoresNaoFinitos");
    assert_eq!(
        geig(&nan, &nan).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    // 1×1 e 2×2 (redução de Hessenberg trivial)
    let one = m(&[&[3.0]]);
    let s = schur(&one).unwrap();
    assert_eq!((s.u.get(0, 0), s.t.get(0, 0)), (1.0, 3.0));
    let two = m(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let h = hess(&two).unwrap();
    assert_eq!(h.h, two);
    // Bloco com subdiagonal nula: o caminho de padronização retorna cedo
    let blk = m(&[&[1.0, 2.0, 0.0], &[0.0, 3.0, 1.0], &[0.0, 0.0, 5.0]]);
    let s = schur(&blk).unwrap();
    assert!(
        s.u.matmul(&s.t)
            .unwrap()
            .matmul(&s.u.transpose())
            .unwrap()
            .sub(&blk)
            .unwrap()
            .max_abs()
            < 1e-14
    );
}

// ------------------------------------------------------------------ solve, normas, funções

#[test]
fn solve_e_funcoes_bordas() {
    // Triangular quase singular: aviso de mal condicionamento (não singular exato)
    let t = m(&[&[1.0, 1e20], &[0.0, 1.0]]);
    let s = solve(&t, &Matrix::column(&[1.0, 1.0])).unwrap();
    assert!(
        matches!(s.warnings.as_slice(), [LinalgWarning::NearlySingular { .. }]),
        "{:?}",
        s.warnings
    );
    // Cholesky de matriz SPD mal condicionada: aviso
    let spd = m(&[&[1.0, 1.0 - 1e-17], &[1.0 - 1e-17, 1.0]]);
    let s = solve(&spd, &Matrix::column(&[1.0, 2.0])).unwrap();
    assert!(!s.warnings.is_empty());
    // rcond de matriz nula, vazia, não quadrada, com NaN
    assert_eq!(rcond(&Matrix::zeros(3, 3)).unwrap(), 0.0);
    assert_eq!(rcond(&Matrix::zeros(0, 0)).unwrap(), INF);
    assert_eq!(
        (rcond(&Matrix::zeros(2, 3))).unwrap_err().id(),
        "chimalab:linalg:matrizNaoQuadrada"
    );
    assert_eq!(
        (rcond(&m(&[&[NAN]]))).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    // Normas
    let v = Matrix::column(&[3.0, -1.0, 2.0]);
    assert_eq!(norm(&v, NormKind::P(f64::NEG_INFINITY)).unwrap(), 1.0);
    assert_eq!(norm(&v, NormKind::P(INF)).unwrap(), 3.0);
    assert_eq!(norm(&Matrix::column(&[0.0, 0.0]), NormKind::Two).unwrap(), 0.0);
    assert_eq!(norm(&Matrix::column(&[INF, 1.0]), NormKind::Two).unwrap(), INF);
    assert_eq!(norm(&m(&[&[1.0, INF], &[0.0, 1.0]]), NormKind::Two).unwrap(), INF);
    assert!(
        norm(&m(&[&[1.0, NAN], &[0.0, 1.0]]), NormKind::Fro)
            .unwrap()
            .is_nan()
    );
    assert_eq!(cond(&Matrix::zeros(2, 2)).unwrap(), INF);
    assert_eq!(orth(&Matrix::zeros(3, 0)).unwrap().shape(), (3, 0));
    assert_eq!(rank(&m(&[&[1.0, 0.0], &[0.0, 1e-3]]), Some(1e-2)).unwrap(), 1);
    assert_eq!(expm(&Matrix::zeros(0, 0)).unwrap().shape(), (0, 0));
    assert_eq!(
        expm(&m(&[&[NAN]])).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    let RealOrComplex::Real(l) = logm(&Matrix::zeros(0, 0)).unwrap().value else {
        panic!()
    };
    assert_eq!(l.shape(), (0, 0));
    let RealOrComplex::Real(f) = funm_elementary(&Matrix::zeros(0, 0), Elementary::Exp)
        .unwrap()
        .value
    else {
        panic!()
    };
    assert_eq!(f.shape(), (0, 0));
}

#[test]
fn regressao_funm_reordena_grupos_nao_contiguos() {
    // Diagonal de Schur [1, 5, 1.01]: 1 e 1,01 formam um grupo e não são contíguos, o que
    // exige trocas de Givens. Lacuna encontrada pela medição de cobertura.
    let a = m(&[&[1.0, 1.0, 1.0], &[0.0, 5.0, 1.0], &[0.0, 0.0, 1.01]]);
    let f = funm_elementary(&a, Elementary::Exp).unwrap();
    let RealOrComplex::Real(x) = f.value else { panic!() };
    let e = expm(&a).unwrap();
    assert!(
        x.sub(&e).unwrap().max_abs() / e.max_abs() < 1e-13,
        "{}",
        x.sub(&e).unwrap().max_abs()
    );
    // Grupos intercalados maiores: [0, 3, 0.05, 3.02, 0.08]
    let d = [0.0, 3.0, 0.05, 3.02, 0.08];
    let t = Matrix::from_fn(5, 5, |i, j| {
        if i == j {
            d[i]
        } else if j > i {
            0.3
        } else {
            0.0
        }
    });
    let RealOrComplex::Real(x) = funm_elementary(&t, Elementary::Sin).unwrap().value else {
        panic!()
    };
    let c = funm_elementary(&t, Elementary::Cos).unwrap().value.to_complex();
    let xc = x.to_complex();
    let id = xc.matmul(&xc).unwrap().add(&c.matmul(&c).unwrap()).unwrap();
    assert!(id.sub(&ComplexMatrix::identity(5)).unwrap().max_abs() < 1e-12);
}

#[test]
fn funm_avisa_quando_taylor_nao_converge() {
    // log em torno de σ ≈ 0,05 com autovalor em 0,0001: raio de convergência quase atingido.
    let t = m(&[&[0.0001, 1.0], &[0.0, 0.09]]);
    let r = funm_elementary(&t, Elementary::Log).unwrap();
    assert!(
        matches!(r.warnings.as_slice(), [LinalgWarning::NearlySingular { .. }]),
        "{:?}",
        r.warnings
    );
}

#[test]
fn derivadas_das_funcoes_elementares() {
    // Confere f^(k)(z) por diferenças centrais de f^(k−1), para k = 1..5.
    let z = Complex64::new(0.7, 0.3);
    let h = 1e-5;
    for f in [
        Elementary::Exp,
        Elementary::Log,
        Elementary::Sin,
        Elementary::Cos,
        Elementary::Sinh,
        Elementary::Cosh,
    ] {
        for k in 1..=5 {
            let num = (f.derivative(z + h, k - 1) - f.derivative(z - h, k - 1)) / (2.0 * h);
            let ana = f.derivative(z, k);
            assert!(
                (num - ana).norm() < 1e-6 * ana.norm().max(1.0),
                "{f:?} k={k}: {num} vs {ana}"
            );
        }
    }
}

// ------------------------------------------------------------------ sinais

#[test]
fn sinais_bordas() {
    let (y, s) = fft(&[], &[0], None, 0).unwrap();
    assert!(y.is_empty() && s == vec![0]);
    let x = [1.0, 2.0, 3.0, 4.0];
    // Dimensões fora da forma: tratadas como singleton
    assert_eq!(fftshift(&x, &[4], Some(&[3])).unwrap(), x.to_vec());
    assert_eq!(fftshift::<f64>(&[], &[0], None).unwrap(), Vec::<f64>::new());
    assert_eq!(
        (fftshift(&x, &[3], None)).unwrap_err().id(),
        "chimalab:signal:formaIncompativel"
    );
    assert!(nextpow2(NAN).is_nan());
    assert_eq!(nextpow2(INF), INF);
    assert_eq!(nextpow2(2f64.powi(40)), 40.0);
    assert_eq!(nextpow2(2f64.powi(40) + 1024.0), 41.0);
    assert_eq!(nextpow2(f64::MIN_POSITIVE), -1022.0);
    assert_eq!(
        (filter(&[], &[1.0], &x, None)).unwrap_err().id(),
        "chimalab:signal:argumentoInvalido"
    );
    assert_eq!(
        (filter(&[1.0], &[], &x, None)).unwrap_err().id(),
        "chimalab:signal:argumentoInvalido"
    );
    assert_eq!(
        (fftn(&[Complex64::new(1.0, 0.0)], &[2], None)).unwrap_err().id(),
        "chimalab:signal:formaIncompativel"
    );
    assert_eq!(
        (convn(&x, &[3], &x, &[4], ConvShape::Full)).unwrap_err().id(),
        "chimalab:signal:formaIncompativel"
    );
    assert_eq!(
        (convn(&x, &[4], &x, &[5], ConvShape::Full)).unwrap_err().id(),
        "chimalab:signal:formaIncompativel"
    );
}

// ------------------------------------------------------------------ polinômios

#[test]
fn polinomios_bordas() {
    assert_eq!(
        (residue(&[1.0], &[0.0])).unwrap_err().id(),
        "chimalab:linalg:argumentoInvalido"
    );
    assert_eq!(
        (residue(&[1.0], &[0.0, 0.0])).unwrap_err().id(),
        "chimalab:linalg:argumentoInvalido"
    );
    assert_eq!(
        (roots(&[INF, 1.0])).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    // polyfit_scaled com um único ponto: desvio indefinido → usa 1
    let f = polyfit_scaled(&[5.0], &[2.0], 0).unwrap();
    assert_eq!(f.centering.unwrap().std, 1.0);
    // Todos os x iguais: desvio 0 → 1
    let f = polyfit_scaled(&[3.0, 3.0, 3.0], &[1.0, 2.0, 3.0], 0).unwrap();
    assert_eq!(f.centering.unwrap().std, 1.0);
    assert!((f.p[0] - 2.0).abs() < 1e-15);
    assert_eq!(polyder(&[]), vec![0.0]);
    assert_eq!(polyint(&[], 3.0), vec![3.0]);
}
