//! Testes do NdArray (DT-18).

use chimalab_array::*;

#[test]
fn normalizacao_de_forma() {
    assert_eq!(normalize_shape(&[]), vec![1, 1]);
    assert_eq!(normalize_shape(&[3]), vec![3, 1]);
    assert_eq!(normalize_shape(&[2, 3, 1, 1]), vec![2, 3]);
    assert_eq!(normalize_shape(&[2, 1, 4]), vec![2, 1, 4]);
    assert_eq!(normalize_shape(&[0, 5]), vec![0, 5]);
    let a = NdArray::from_shape_vec(&[4], vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!(a.dims(), &[4, 1]);
    assert_eq!(a.ndims(), 2);
    assert_eq!(a.dim(7), 1);
}

#[test]
fn erros_de_construcao() {
    let e = NdArray::from_shape_vec(&[2, 2], vec![1.0; 3]).unwrap_err();
    assert_eq!(e.id(), "chimalab:array:formaIncompativel");
    assert_eq!(
        (NdArray::<f64>::from_rows(&[&[1.0, 2.0], &[3.0]]))
            .unwrap_err()
            .id(),
        "chimalab:array:dimensoesIncompativeis"
    );
}

#[test]
fn ordem_column_major_e_subscritos() {
    // 2×3×2 com valor = índice linear
    let a = NdArray::from_fn_nd(&[2, 3, 2], |s| (s[0] + 2 * s[1] + 6 * s[2]) as f64);
    assert_eq!(
        a.as_slice(),
        (0..12).map(|v| v as f64).collect::<Vec<_>>().as_slice()
    );
    assert_eq!(*a.get_nd(&[1, 2, 1]).unwrap(), 11.0);
    assert_eq!(a.linear_index(&[1, 0, 1]).unwrap(), 7);
    assert_eq!(a.subscripts(7), vec![1, 0, 1]);
    // Subscritos extras iguais a zero são aceitos; fora dos limites, erro
    assert_eq!(*a.get_nd(&[1, 2, 1, 0]).unwrap(), 11.0);
    assert_eq!(
        a.get_nd(&[2, 0, 0]).unwrap_err().id(),
        "chimalab:array:indiceForaDosLimites"
    );
    assert_eq!(
        (a.get_nd(&[0, 0, 0, 1])).unwrap_err().id(),
        "chimalab:array:indiceForaDosLimites"
    );
    // Visão 2-D de N-D: colunas = produto das dimensões finais
    assert_eq!(a.shape(), (2, 6));
    assert_eq!(a.get(1, 5), 11.0);
}

#[test]
fn copia_sob_demanda() {
    let a = NdArray::from_fn(3, 3, |i, j| (i * 3 + j) as f64);
    let mut b = a.clone();
    assert!(a.shares_data_with(&b));
    b.set(0, 0, 100.0);
    assert!(!a.shares_data_with(&b));
    assert_eq!(a.get(0, 0), 0.0);
    assert_eq!(b.get(0, 0), 100.0);
    // reshape compartilha dados até a primeira escrita
    let r = a.reshape(&[9]).unwrap();
    assert!(r.shares_data_with(&a));
    assert_eq!(r.dims(), &[9, 1]);
    assert_eq!(
        (a.reshape(&[2, 4])).unwrap_err().id(),
        "chimalab:array:formaIncompativel"
    );
}

#[test]
fn permute_e_transpose() {
    let a = NdArray::from_fn_nd(&[2, 3, 4], |s| (100 * s[0] + 10 * s[1] + s[2]) as f64);
    let p = a.permute(&[2, 0, 1]).unwrap();
    assert_eq!(p.dims(), &[4, 2, 3]);
    for (i, j, k) in [(0, 0, 0), (3, 1, 2), (2, 0, 1)] {
        assert_eq!(*p.get_nd(&[i, j, k]).unwrap(), *a.get_nd(&[j, k, i]).unwrap());
    }
    assert_eq!(
        a.permute(&[0, 0, 1]).unwrap_err().id(),
        "chimalab:array:permutacaoInvalida"
    );
    assert_eq!(
        (a.permute(&[1, 0])).unwrap_err().id(),
        "chimalab:array:permutacaoInvalida"
    ); // falta a dimensão 2
    let m = NdArray::from_rows(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]).unwrap();
    assert_eq!(
        m.transpose(),
        NdArray::from_rows(&[&[1.0, 4.0], &[2.0, 5.0], &[3.0, 6.0]]).unwrap()
    );
    assert_eq!(m.permute(&[1, 0]).unwrap(), m.transpose());
}

#[test]
fn expansao_implicita() {
    let col = NdArray::column(&[1.0, 2.0]);
    let row = NdArray::from_rows(&[&[10.0, 20.0, 30.0]]).unwrap();
    let s = col.zip_with(&row, |a, b| a + b).unwrap();
    assert_eq!(
        s,
        NdArray::from_rows(&[&[11.0, 21.0, 31.0], &[12.0, 22.0, 32.0]]).unwrap()
    );
    // N-D: (2×1×3) com (1×4)
    let a = NdArray::from_fn_nd(&[2, 1, 3], |s| (s[0] + 10 * s[2]) as f64);
    let b = NdArray::from_fn_nd(&[1, 4], |s| (100 * s[1]) as f64);
    let c = a.zip_with(&b, |x, y| x + y).unwrap();
    assert_eq!(c.dims(), &[2, 4, 3]);
    assert_eq!(*c.get_nd(&[1, 3, 2]).unwrap(), 1.0 + 20.0 + 300.0);
    let bad = NdArray::from_rows(&[&[1.0, 2.0]]).unwrap();
    assert_eq!(
        row.zip_with(&bad, |a, b| a + b).unwrap_err().id(),
        "chimalab:array:dimensoesIncompativeis"
    );
    assert_eq!(broadcast_shape(&[3, 1], &[1, 4]).unwrap(), vec![3, 4]);
}

#[test]
fn concatenacao() {
    let a = NdArray::from_rows(&[&[1.0, 2.0]]).unwrap();
    let b = NdArray::from_rows(&[&[3.0, 4.0], &[5.0, 6.0]]).unwrap();
    let v = NdArray::concat(&[&a, &b], 0).unwrap();
    assert_eq!(
        v,
        NdArray::from_rows(&[&[1.0, 2.0], &[3.0, 4.0], &[5.0, 6.0]]).unwrap()
    );
    let h = NdArray::concat(&[&b, &b], 1).unwrap();
    assert_eq!(h.shape(), (2, 4));
    let p = NdArray::concat(&[&b, &b, &b], 2).unwrap();
    assert_eq!(p.dims(), &[2, 2, 3]);
    assert_eq!(*p.get_nd(&[1, 0, 2]).unwrap(), 5.0);
    assert_eq!(
        (NdArray::concat(&[&a, &b], 1)).unwrap_err().id(),
        "chimalab:array:dimensoesIncompativeis"
    );
}

#[test]
fn squeeze_remove_singletons() {
    let a = NdArray::<f64>::zeros_nd(&[1, 1, 3]);
    assert_eq!(a.squeeze().dims(), &[3, 1]);
    let b = NdArray::<f64>::zeros_nd(&[2, 1, 1, 4]);
    assert_eq!(b.squeeze().dims(), &[2, 4]);
    assert_eq!(NdArray::<f64>::zeros(1, 5).squeeze().dims(), &[1, 5]);
}

#[test]
fn operacoes_numericas_e_complexas() {
    let m = NdArray::from_rows(&[&[1.0, -2.0], &[-3.0, 4.0]]).unwrap();
    assert_eq!(m.trace(), 5.0);
    assert_eq!(m.norm1(), 6.0);
    assert_eq!(m.max_abs(), 4.0);
    assert!(NdArray::<f64>::identity(3).is_symmetric());
    let z = NdArray::from_rows(&[
        &[Complex64::new(2.0, 0.0), Complex64::new(1.0, 1.0)],
        &[Complex64::new(1.0, -1.0), Complex64::new(3.0, 0.0)],
    ])
    .unwrap();
    assert!(z.is_hermitian());
    assert!(!z.is_symmetric());
    assert_eq!(z.adjoint(), z);
    assert_eq!(z.trace(), Complex64::new(5.0, 0.0));
    let c = NdArray::from_real(&m);
    assert!(c.is_real());
    assert_eq!(c.to_real(0.0).unwrap(), m);
    assert!(z.to_real(1e-12).is_none());
    assert_eq!(m.add(&m).unwrap(), m.scale(2.0));
    assert_eq!(
        (m.add(&NdArray::zeros(3, 2))).unwrap_err().id(),
        "chimalab:array:dimensoesIncompativeis"
    );
}

#[test]
fn scalar_trait() {
    const { assert!(!f64::IS_COMPLEX && Complex64::IS_COMPLEX) };
    assert_eq!(Scalar::signum(-3.0f64), -1.0);
    assert_eq!(Scalar::signum(0.0f64), 1.0);
    let s = Scalar::signum(Complex64::new(3.0, 4.0));
    assert!((s - Complex64::new(0.6, 0.8)).norm() < 1e-15);
    assert_eq!(f64::from_complex(Complex64::new(1.0, 0.0)), Some(1.0));
    assert_eq!(f64::from_complex(Complex64::new(1.0, 1.0)), None);
    assert!(Scalar::sqrt(-1.0f64).is_nan());
    assert_eq!(Scalar::sqrt(Complex64::new(-1.0, 0.0)), Complex64::new(0.0, 1.0));
}

#[test]
fn estrutura_triangular_por_definicao() {
    // Um único elemento não nulo em (p, q), em matrizes quadradas e retangulares: triangular
    // superior ⇔ p ≤ q; inferior ⇔ p ≥ q (definição). Também a matriz nula e a vazia.
    for (m, n) in [(3usize, 3usize), (3, 5), (5, 3), (1, 4), (4, 1)] {
        for p in 0..m {
            for q in 0..n {
                let a = NdArray::<f64>::from_fn(m, n, |i, j| if (i, j) == (p, q) { -2.5 } else { 0.0 });
                assert_eq!(a.is_upper_triangular(), p <= q, "{m}×{n}, ({p}, {q})");
                assert_eq!(a.is_lower_triangular(), p >= q, "{m}×{n}, ({p}, {q})");
            }
        }
        let z = NdArray::<f64>::zeros(m, n);
        assert!(z.is_upper_triangular() && z.is_lower_triangular());
    }
    let e = NdArray::<f64>::zeros(0, 3);
    assert!(e.is_upper_triangular() && e.is_lower_triangular());
    assert_eq!(e.norm1(), 0.0);
    // N-D (mesmo nulo) não é matriz: nenhuma das duas formas.
    assert!(!NdArray::<f64>::zeros_nd(&[2, 2, 2]).is_lower_triangular());
    // norm1 em retangular: maior soma de módulos de coluna.
    let r = NdArray::from_rows(&[&[1.0, -7.0], &[-2.0, 0.5], &[3.0, 0.0]]).unwrap();
    assert_eq!(r.norm1(), 7.5);
}
