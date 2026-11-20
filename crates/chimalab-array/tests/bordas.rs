//! Padrão rigoroso (CLAUDE.md §7.4–7.5): bordas e erros do NdArray.

use chimalab_array::*;

#[test]
fn erros_tem_id_e_mensagem() {
    let erros = [
        (
            ArrayError::ShapeMismatch {
                expected: 1,
                actual: 2,
            },
            "chimalab:array:formaIncompativel",
        ),
        (
            ArrayError::IncompatibleShapes {
                operation: "x",
                left: vec![1],
                right: vec![2],
            },
            "chimalab:array:dimensoesIncompativeis",
        ),
        (
            ArrayError::InvalidPermutation(vec![1, 1]),
            "chimalab:array:permutacaoInvalida",
        ),
        (
            ArrayError::IndexOutOfBounds {
                index: vec![3],
                shape: vec![2, 2],
            },
            "chimalab:array:indiceForaDosLimites",
        ),
    ];
    for (e, id) in erros {
        assert_eq!(e.id(), id);
        assert!(!e.to_string().is_empty());
        let _: &dyn std::error::Error = &e;
    }
}

#[test]
fn bordas_de_dados_e_indices() {
    let a = NdArray::from_fn(2, 2, |i, j| (i + 2 * j) as f64);
    let b = a.clone();
    assert_eq!(b.into_vec(), vec![0.0, 1.0, 2.0, 3.0]); // compartilhado: copia
    assert_eq!(a.clone().into_vec(), vec![0.0, 1.0, 2.0, 3.0]);
    let mut c = NdArray::<f64>::zeros_nd(&[2, 2, 2]);
    c.set_nd(&[1, 1, 1], 7.0).unwrap();
    assert_eq!(*c.get_nd(&[1, 1, 1]).unwrap(), 7.0);
    assert_eq!(
        (c.set_nd(&[2, 0, 0], 1.0)).unwrap_err().id(),
        "chimalab:array:indiceForaDosLimites"
    );
    let e = NdArray::<f64>::concat(&[], 0).unwrap();
    assert_eq!(e.dims(), &[0, 0]);
    let v = NdArray::<f64>::filled_nd(&[3], 2.5);
    assert_eq!(v.as_slice(), &[2.5, 2.5, 2.5]);
    assert_eq!(
        (NdArray::<f64>::zeros(2, 3).sub(&NdArray::zeros(3, 2)))
            .unwrap_err()
            .id(),
        "chimalab:array:dimensoesIncompativeis"
    );
    assert!(!NdArray::<f64>::zeros(2, 3).is_square());
    assert!(!NdArray::<f64>::zeros_nd(&[2, 2, 2]).is_upper_triangular());
    assert!(NdArray::from_rows(&[&[f64::NAN]]).unwrap().max_abs().is_nan());
    assert!(!NdArray::from_rows(&[&[f64::INFINITY]]).unwrap().all_finite());
    // subscripts com dimensão nula não divide por zero
    assert_eq!(NdArray::<f64>::zeros(0, 3).subscripts(0), vec![0, 0]);
    let mut d = NdArray::<f64>::zeros(1, 1);
    d.as_mut_slice()[0] = 4.0;
    assert_eq!(d.get(0, 0), 4.0);
}
