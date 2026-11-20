//! Padrão rigoroso (CLAUDE.md §7.3): propriedades do NdArray com entradas aleatórias.

use chimalab_array::*;
use proptest::prelude::*;

fn forma() -> impl Strategy<Value = Vec<usize>> {
    proptest::collection::vec(1usize..5, 1..5)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn reshape_ida_e_volta(s in forma()) {
        let n: usize = s.iter().product();
        let a = NdArray::from_shape_vec(&s, (0..n).map(|v| v as f64).collect()).unwrap();
        let r = a.reshape(&[n]).unwrap().reshape(&s).unwrap();
        prop_assert_eq!(r, a);
    }

    #[test]
    fn permute_e_sua_inversa(s in forma(), seed in 0u64..1000) {
        let n: usize = s.iter().product();
        let a = NdArray::from_shape_vec(&s, (0..n).map(|v| v as f64).collect()).unwrap();
        let nd = a.ndims();
        // permutação pseudoaleatória das dimensões
        let mut order: Vec<usize> = (0..nd).collect();
        for i in (1..nd).rev() {
            order.swap(i, (seed as usize + i * 7) % (i + 1));
        }
        let mut inv = vec![0; nd];
        for (i, &o) in order.iter().enumerate() {
            inv[o] = i;
        }
        let p = a.permute(&order).unwrap();
        prop_assert_eq!(p.numel(), a.numel());
        prop_assert_eq!(p.permute(&inv).unwrap(), a);
    }

    #[test]
    fn transposta_involutiva(r in 0usize..6, c in 0usize..6) {
        let a = NdArray::from_fn(r, c, |i, j| (i * 10 + j) as f64);
        prop_assert_eq!(a.transpose().transpose(), a);
    }

    #[test]
    fn expansao_implicita_igual_a_expansao_explicita(r in 1usize..6, c in 1usize..6) {
        let col = NdArray::from_fn(r, 1, |i, _| i as f64);
        let row = NdArray::from_fn(1, c, |_, j| 100.0 * j as f64);
        let s = col.zip_with(&row, |a, b| a + b).unwrap();
        let explicit = NdArray::from_fn(r, c, |i, j| i as f64 + 100.0 * j as f64);
        prop_assert_eq!(s, explicit);
    }

    #[test]
    fn concat_preserva_partes(r1 in 0usize..5, r2 in 0usize..5, c in 1usize..5) {
        let a = NdArray::from_fn(r1, c, |i, j| (i + 10 * j) as f64);
        let b = NdArray::from_fn(r2, c, |i, j| (100 + i + 10 * j) as f64);
        let v = NdArray::concat(&[&a, &b], 0).unwrap();
        prop_assert_eq!(v.shape(), (r1 + r2, c));
        prop_assert_eq!(v.submatrix(0, r1, 0, c), a);
        prop_assert_eq!(v.submatrix(r1, r1 + r2, 0, c), b);
    }

    #[test]
    fn indice_linear_e_subscritos_sao_inversos(s in forma(), k in 0usize..1000) {
        let a = NdArray::<f64>::zeros_nd(&s);
        prop_assume!(a.numel() > 0);
        let li = k % a.numel();
        let sub = a.subscripts(li);
        prop_assert_eq!(a.linear_index(&sub).unwrap(), li);
    }

    #[test]
    fn copia_sob_demanda_isola_escritas(v in proptest::collection::vec(-5.0f64..5.0, 1..20)) {
        let a = NdArray::column(&v);
        let mut b = a.clone();
        b.set(0, 0, 1e9);
        prop_assert_eq!(a.get(0, 0), v[0]);
        prop_assert!(!a.shares_data_with(&b));
    }
}
