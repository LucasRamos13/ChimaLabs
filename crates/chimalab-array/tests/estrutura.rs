//! Estrutura e escalares do `chimalab-array` testados diretamente (mutação: antes, várias funções
//! só eram exercitadas pelos testes do `chimalab-math`).

use chimalab_array::{ArrayError, Complex64, NdArray, Scalar, broadcast_shape, normalize_shape};

#[test]
fn escalares_reais_e_complexos() {
    assert_eq!(f64::from_f64(2.5), 2.5);
    assert_eq!(f64::one(), 1.0);
    assert_eq!((3.0f64.re(), 3.0f64.im(), (-3.0f64).conj()), (3.0, 0.0, -3.0));
    assert!(1.0f64.is_finite() && !f64::INFINITY.is_finite() && !f64::NAN.is_finite());
    let z = Complex64::new(3.0, 4.0);
    assert_eq!(Complex64::one(), Complex64::new(1.0, 0.0));
    assert_eq!(Complex64::from_f64(2.0), Complex64::new(2.0, 0.0));
    assert_eq!(
        (z.re(), Scalar::im(z), Scalar::conj(z)),
        (3.0, 4.0, Complex64::new(3.0, -4.0))
    );
    assert!(Scalar::is_finite(z));
    assert!(!Scalar::is_finite(Complex64::new(1.0, f64::INFINITY)));
    assert!(!Scalar::is_finite(Complex64::new(f64::INFINITY, 1.0)));
    assert!(Scalar::is_nan(Complex64::new(1.0, f64::NAN)));
    assert!(!Scalar::is_nan(z));
    assert_eq!(Scalar::to_complex(z), z);
    assert_eq!(<Complex64 as Scalar>::from_complex(z), Some(z));
    assert_eq!(<f64 as Scalar>::from_complex(Complex64::new(2.0, 0.0)), Some(2.0));
    assert_eq!(<f64 as Scalar>::from_complex(z), None);
}

#[test]
fn formas_e_expansao() {
    assert_eq!(normalize_shape(&[]), vec![1, 1]);
    assert_eq!(normalize_shape(&[5]), vec![5, 1]);
    assert_eq!(normalize_shape(&[3, 1]), vec![3, 1]);
    assert_eq!(normalize_shape(&[3, 1, 1]), vec![3, 1]);
    assert_eq!(normalize_shape(&[3, 1, 2]), vec![3, 1, 2]);
    assert_eq!(broadcast_shape(&[3, 4], &[3, 4]).unwrap(), vec![3, 4]);
    assert_eq!(broadcast_shape(&[3, 1], &[1, 4]).unwrap(), vec![3, 4]);
    assert_eq!(broadcast_shape(&[2, 3], &[2, 3, 5]).unwrap(), vec![2, 3, 5]);
    assert!(broadcast_shape(&[2, 3], &[3, 2]).is_err());
}

#[test]
fn igualdade_vazio_e_acesso() {
    let a = NdArray::from_shape_vec(&[2, 3], (1..=6).map(f64::from).collect()).unwrap();
    let b = NdArray::from_shape_vec(&[3, 2], (1..=6).map(f64::from).collect()).unwrap();
    assert_ne!(a, b, "mesmos dados, formas diferentes");
    let mut c = a.clone();
    assert_eq!(a, c);
    c.set(1, 2, 60.0);
    assert_ne!(a, c, "mesma forma, dados diferentes");
    assert_eq!((c.get(1, 2), c.as_slice()[5]), (60.0, 60.0));
    let mut d = NdArray::<f64>::zeros(3, 3);
    d.set(1, 2, 7.0);
    assert_eq!(d.as_slice()[1 + 2 * 3], 7.0);
    assert_eq!(d.as_slice().iter().filter(|&&x| x != 0.0).count(), 1);
    assert!(NdArray::<f64>::zeros(0, 3).is_empty());
    assert!(!a.is_empty());
    // Submatriz, identidade, subtração e formas incompatíveis.
    let m = NdArray::from_fn(3, 3, |i, j| (10 * i + j) as f64);
    let s = m.submatrix(1, 3, 0, 2);
    assert_eq!((s.rows(), s.cols()), (2, 2));
    assert_eq!((s.get(0, 0), s.get(1, 1)), (10.0, 21.0));
    let s2 = m.submatrix(0, 2, 1, 3);
    assert_eq!(
        (s2.rows(), s2.cols(), s2.get(0, 0), s2.get(1, 1)),
        (2, 2, 1.0, 12.0)
    );
    let id = NdArray::<f64>::identity(3);
    assert_eq!(id.as_slice(), &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    let x = NdArray::from_shape_vec(&[1, 2], vec![5.0, 7.0]).unwrap();
    let y = NdArray::from_shape_vec(&[1, 2], vec![2.0, 3.0]).unwrap();
    assert_eq!(x.sub(&y).unwrap().as_slice(), &[3.0, 4.0]);
    assert!(matches!(x.sub(&a), Err(ArrayError::IncompatibleShapes { .. })));
    assert!(matches!(x.add(&a), Err(ArrayError::IncompatibleShapes { .. })));
    // Mesmo número de elementos, formas diferentes (1×2 contra 2×1): também é erro.
    let xt = NdArray::from_shape_vec(&[2, 1], vec![2.0, 3.0]).unwrap();
    assert!(matches!(x.sub(&xt), Err(ArrayError::IncompatibleShapes { .. })));
    assert!(x.all_finite());
    assert!(
        !NdArray::from_shape_vec(&[1, 2], vec![1.0, f64::INFINITY])
            .unwrap()
            .all_finite()
    );
}

#[test]
fn simetria_e_hermiticidade() {
    let sim = NdArray::from_fn(3, 3, |i, j| (i + j) as f64);
    assert!(sim.is_symmetric() && sim.is_hermitian());
    // Assimetria só no par (2, 1)/(1, 2): o laço tem de chegar lá.
    let mut quase = sim.clone();
    quase.set(2, 1, 99.0);
    assert!(!quase.is_symmetric() && !quase.is_hermitian());
    let mut quase2 = sim.clone();
    quase2.set(2, 0, 99.0);
    assert!(!quase2.is_symmetric());
    assert!(!NdArray::<f64>::zeros(2, 3).is_symmetric());
    // Complexo: hermitiana exige o conjugado e diagonal real.
    let c = |v: &[(f64, f64)]| {
        NdArray::from_shape_vec(&[2, 2], v.iter().map(|&(r, i)| Complex64::new(r, i)).collect()).unwrap()
    };
    assert!(c(&[(1.0, 0.0), (1.0, -1.0), (1.0, 1.0), (2.0, 0.0)]).is_hermitian());
    assert!(!c(&[(1.0, 0.0), (1.0, 1.0), (1.0, 1.0), (2.0, 0.0)]).is_hermitian());
    assert!(!c(&[(1.0, 0.5), (1.0, -1.0), (1.0, 1.0), (2.0, 0.0)]).is_hermitian());
    assert!(!c(&[(1.0, 0.0), (1.0, -1.0), (1.0, 1.0), (2.0, 0.5)]).is_hermitian());
    let h3 = NdArray::from_fn(3, 3, |i, j| {
        if i == j {
            Complex64::new(i as f64, 0.0)
        } else {
            let s = if i > j { 1.0 } else { -1.0 };
            Complex64::new((i + j) as f64, s * (i * j + 1) as f64)
        }
    });
    assert!(h3.is_hermitian());
    let mut h3b = h3.clone();
    h3b.set(2, 1, Complex64::new(0.0, 0.0));
    assert!(!h3b.is_hermitian());
    // Parte real com tolerância relativa ao maior módulo (2): 1,5e-12 ≤ 1e-12·2.
    let quase_real = NdArray::from_shape_vec(
        &[1, 2],
        vec![Complex64::new(1.0, 1.5e-12), Complex64::new(2.0, 0.0)],
    )
    .unwrap();
    assert_eq!(quase_real.to_real(1e-12).unwrap().as_slice(), &[1.0, 2.0]);
    assert!(quase_real.to_real(1e-13).is_none());
    assert!(!quase_real.is_real());
    assert!(
        NdArray::from_shape_vec(&[1, 1], vec![Complex64::new(3.0, 0.0)])
            .unwrap()
            .is_real()
    );
}
