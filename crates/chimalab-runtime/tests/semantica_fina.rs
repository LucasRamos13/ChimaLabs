//! F003–F005: semântica fina do modelo de valores, com valores exatos (lacunas apontadas pelo
//! teste de mutação; antes, várias operações só tinham a classe ou a forma conferida).

use chimalab_array::NdArray;
use chimalab_runtime::{
    BinOp, Class, CmpOp, Index, IntClass, LogicOp, RuntimeError, Value, colon, compare, elementwise, logical,
    mldivide, mpower, mrdivide, mtimes, neg, sqrt, subsasgn, subsdelete, subsref,
};
use num_complex::Complex64;

fn int(c: IntClass, v: f64) -> Value {
    Value::scalar(v).cast(Class::Int(c)).unwrap()
}

fn f(v: &Value) -> Vec<f64> {
    v.to_f64s().unwrap()
}

fn z(re: f64, im: f64) -> Value {
    Value::Complex(NdArray::from_shape_vec(&[1, 1], vec![Complex64::new(re, im)]).unwrap())
}

fn row(v: &[f64]) -> Value {
    Value::doubles(&[1, v.len()], v.to_vec())
}

/// Aritmética complexa elemento a elemento: (3 + 4i) com (1 − 2i).
#[test]
fn aritmetica_complexa_valores() {
    let (a, b) = (z(3.0, 4.0), z(1.0, -2.0));
    let c = |op| elementwise(op, &a, &b).unwrap().to_complex()[0];
    assert_eq!(c(BinOp::Add), Complex64::new(4.0, 2.0));
    assert_eq!(c(BinOp::Sub), Complex64::new(2.0, 6.0));
    assert_eq!(c(BinOp::Mul), Complex64::new(11.0, -2.0));
    assert_eq!(c(BinOp::Div), Complex64::new(-1.0, 2.0));
    let l = c(BinOp::LeftDiv); // (1 − 2i)/(3 + 4i) = (−5 − 10i)/25
    assert!((l - Complex64::new(-0.2, -0.4)).norm() <= 1e-16, "{l}");
    assert_eq!(
        c(BinOp::Pow),
        Complex64::new(3.0, 4.0).powc(Complex64::new(1.0, -2.0))
    );
    // Expoente inteiro: multiplicação repetida, exata em inteiros gaussianos.
    let q = elementwise(BinOp::Pow, &a, &Value::scalar(3.0))
        .unwrap()
        .to_complex()[0];
    assert_eq!(q, Complex64::new(-117.0, 44.0));
}

/// Base negativa com expoente fracionário finito dá complexo; inteiro, ou infinito, dá real.
#[test]
fn potencia_de_base_negativa() {
    let p = |e: f64| elementwise(BinOp::Pow, &Value::scalar(-8.0), &Value::scalar(e)).unwrap();
    let r = p(1.0 / 3.0);
    assert!(r.is_complex());
    let w = r.to_complex()[0];
    assert!((w - Complex64::new(1.0, 3f64.sqrt())).norm() <= 1e-14, "{w}");
    for (e, esp) in [(2.0, 64.0), (-1.0, -0.125), (f64::INFINITY, f64::INFINITY)] {
        let r = p(e);
        assert!(!r.is_complex(), "{e}");
        assert_eq!(f(&r), vec![esp]);
    }
    assert!(
        !elementwise(BinOp::Pow, &Value::scalar(8.0), &Value::scalar(0.5))
            .unwrap()
            .is_complex()
    );
}

/// Intervalos: contagem com tolerância, último elemento ajustado, sentido e vazios.
#[test]
fn intervalos_colon() {
    let v = |a, s, b| f(&colon(a, s, b));
    let d = v(0.0, 0.1, 1.0);
    assert_eq!(d.len(), 11);
    assert_eq!(*d.last().unwrap(), 1.0);
    assert_eq!(*v(0.0, 0.1, 0.3).last().unwrap(), 0.3);
    assert_eq!(v(1.0, 2.0, 10.0), vec![1.0, 3.0, 5.0, 7.0, 9.0]);
    assert_eq!(v(10.0, -3.0, 0.0), vec![10.0, 7.0, 4.0, 1.0]);
    assert_eq!(v(1.0, 1.0, 1.0), vec![1.0]);
    assert_eq!(colon(1.0, 1.0, 1.0).dims(), &[1, 1]);
    for (a, s, b) in [(1.0, 0.5, 0.0), (1.0, 0.0, 5.0), (f64::NAN, 1.0, 2.0)] {
        let e = colon(a, s, b);
        assert_eq!((e.dims(), e.numel()), (&[1usize, 0][..], 0), "{a}:{s}:{b}");
    }
}

/// Comparações de igualdade com complexos usam as duas partes; ordem usa a parte real.
#[test]
fn comparacao_complexa() {
    let t = |op, a: &Value, b: &Value| f(&compare(op, a, b).unwrap())[0] == 1.0;
    let (a, b) = (z(1.0, 2.0), z(1.0, 3.0));
    assert!(!t(CmpOp::Eq, &a, &b));
    assert!(t(CmpOp::Ne, &a, &b));
    assert!(t(CmpOp::Eq, &a, &z(1.0, 2.0)));
    assert!(!t(CmpOp::Ne, &a, &z(1.0, 2.0)));
    assert!(t(CmpOp::Le, &a, &b) && t(CmpOp::Ge, &a, &b));
}

/// Saturação nos extremos de todos os inteiros com sinal.
#[test]
fn saturacao_intmin() {
    assert_eq!(f(&int(IntClass::I32, -3e9)), vec![f64::from(i32::MIN)]);
    assert_eq!(f(&int(IntClass::I64, -1e19)), vec![i64::MIN as f64]);
    assert_eq!(f(&int(IntClass::I16, -1e6)), vec![f64::from(i16::MIN)]);
    let r = elementwise(BinOp::Sub, &int(IntClass::I32, -2e9), &int(IntClass::I32, 2e9)).unwrap();
    assert_eq!(f(&r), vec![f64::from(i32::MIN)]);
}

/// −a: negação IEEE em ponto flutuante (sinal do zero) e exata em inteiros (regressão).
#[test]
fn negacao() {
    let r = neg(&Value::scalar(0.0)).unwrap();
    assert!(f(&r)[0].is_sign_negative(), "−(+0) deve ser −0");
    assert!(f(&neg(&Value::scalar(-0.0)).unwrap())[0].is_sign_positive());
    let s = neg(&Value::scalar(0.0).cast(Class::Single).unwrap()).unwrap();
    assert_eq!(s.class(), Class::Single);
    assert!(f(&s)[0].is_sign_negative());
    let c = neg(&z(0.0, 2.0)).unwrap().to_complex()[0];
    assert!(c.re.is_sign_negative() && c.im == -2.0);
    // int64 acima de 2⁵³: exato (um zero double perderia o último bit).
    let big = Value::Int(chimalab_runtime::IntData::from_i128(
        IntClass::I64,
        &[1, 1],
        vec![9_007_199_254_740_993],
    ));
    match neg(&big).unwrap() {
        Value::Int(d) => assert_eq!(d.to_i128(), vec![-9_007_199_254_740_993]),
        o => panic!("{o:?}"),
    }
    assert_eq!(f(&neg(&int(IntClass::I8, -128.0)).unwrap()), vec![127.0]);
    assert_eq!(f(&neg(&int(IntClass::U8, 5.0)).unwrap()), vec![0.0]);
    let l = neg(&Value::Logical(
        NdArray::from_shape_vec(&[1, 1], vec![true]).unwrap(),
    ))
    .unwrap();
    assert_eq!((l.class(), f(&l)), (Class::Double, vec![-1.0]));
}

/// Raiz de zero em single é válida; complexo não converte para single.
#[test]
fn raiz_e_conversoes() {
    let s = sqrt(&Value::scalar(0.0).cast(Class::Single).unwrap()).unwrap();
    assert_eq!((s.class(), f(&s)), (Class::Single, vec![0.0]));
    assert!(z(1.0, 1.0).cast(Class::Single).is_err());
    assert!(z(1.0, 1.0).cast(Class::Int(IntClass::I8)).is_err());
    let d = z(1.0, 1.0).cast(Class::Double).unwrap();
    assert!(d.is_complex());
}

/// Valor lógico de complexos: qualquer parte não nula; NaN em qualquer parte é erro.
#[test]
fn logica_com_complexos() {
    let um = Value::scalar(1.0);
    assert_eq!(f(&logical(LogicOp::And, &z(0.0, 1.0), &um).unwrap()), vec![1.0]);
    assert_eq!(f(&logical(LogicOp::And, &z(0.0, 0.0), &um).unwrap()), vec![0.0]);
    assert!(logical(LogicOp::Or, &z(0.0, f64::NAN), &um).is_err());
    assert!(logical(LogicOp::Or, &z(f64::NAN, 0.0), &um).is_err());
}

/// Divisão inteira: arredondamento do meio para longe de zero, exato.
#[test]
fn divisao_inteira_arredonda() {
    let d = |a: f64, b: f64| {
        f(&elementwise(BinOp::Div, &int(IntClass::I32, a), &int(IntClass::I32, b)).unwrap())[0]
    };
    for (a, b, q) in [
        (7.0, 2.0, 4.0),
        (-7.0, 2.0, -4.0),
        (5.0, 3.0, 2.0),
        (4.0, 3.0, 1.0),
        (-5.0, 3.0, -2.0),
        (-4.0, 3.0, -1.0),
        (6.0, 4.0, 2.0),
        (9.0, -6.0, -2.0),
    ] {
        assert_eq!(d(a, b), q, "{a}/{b}");
    }
}

/// Expansão implícita com passo > 1 (coluna × linha) e em 3-D; produto matricial com escalar e
/// recusa de 3-D.
#[test]
fn expansao_e_produto() {
    let c = Value::doubles(&[2, 1], vec![10.0, 20.0]);
    let r = elementwise(BinOp::Add, &c, &row(&[1.0, 2.0, 3.0])).unwrap();
    assert_eq!(r.dims(), &[2, 3]);
    assert_eq!(f(&r), vec![11.0, 21.0, 12.0, 22.0, 13.0, 23.0]);
    let a = Value::doubles(&[2, 1, 2], vec![1.0, 2.0, 3.0, 4.0]);
    let b = Value::doubles(&[1, 3, 1], vec![10.0, 20.0, 30.0]);
    let r = elementwise(BinOp::Mul, &a, &b).unwrap();
    assert_eq!(r.dims(), &[2, 3, 2]);
    assert_eq!(
        f(&r),
        vec![
            10.0, 20.0, 20.0, 40.0, 30.0, 60.0, 30.0, 40.0, 60.0, 80.0, 90.0, 120.0
        ]
    );
    let m = Value::doubles(&[2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        f(&mtimes(&m, &Value::scalar(2.0)).unwrap()),
        vec![2.0, 4.0, 6.0, 8.0]
    );
    assert_eq!(
        f(&mtimes(&Value::scalar(3.0), &m).unwrap()),
        vec![3.0, 6.0, 9.0, 12.0]
    );
    let tres_d = Value::doubles(&[2, 2, 2], vec![1.0; 8]);
    assert!(mtimes(&tres_d, &tres_d).is_err());
}

/// Indexação: formas da indexação linear, valores em 3-D, `end` em dimensão vazia e as posições
/// informadas nos erros.
#[test]
fn indexacao_formas_valores_e_erros() {
    let fora = |r: Result<Value, RuntimeError>| match r {
        Err(RuntimeError::IndexOutOfBounds { index, bound }) => (index, bound),
        o => panic!("{o:?}"),
    };
    // 3×1×2 indexado por lista: não é vetor 2-D, resultado em linha.
    let a = Value::doubles(&[3, 1, 2], (1..=6).map(f64::from).collect());
    let r = subsref(&a, &[Index::List(vec![2, 5])]).unwrap();
    assert_eq!((r.dims(), f(&r)), (&[1usize, 2][..], vec![2.0, 5.0]));
    // Coluna indexada por lista: coluna.
    let c = Value::doubles(&[3, 1], vec![1.0, 2.0, 3.0]);
    assert_eq!(subsref(&c, &[Index::List(vec![3, 1])]).unwrap().dims(), &[2, 1]);
    // 2×3×4: A(2, 3, 4) e uma fatia.
    let b = Value::doubles(&[2, 3, 4], (1..=24).map(f64::from).collect());
    assert_eq!(
        f(&subsref(&b, &[Index::Scalar(2), Index::Scalar(3), Index::Scalar(4)]).unwrap()),
        vec![24.0]
    );
    let s = subsref(
        &b,
        &[Index::Scalar(1), Index::List(vec![2, 3]), Index::List(vec![2, 4])],
    )
    .unwrap();
    assert_eq!(
        (s.dims(), f(&s)),
        (&[1usize, 2, 2][..], vec![9.0, 11.0, 21.0, 23.0])
    );
    // end com uma dimensão nula.
    let vazio = Value::doubles(&[3, 0], vec![]);
    assert_eq!(vazio.end_index(1, 2), 0);
    assert_eq!(vazio.end_index(0, 2), 3);
    // Posições nos erros: linear, por subscrito e por máscara longa.
    assert_eq!(fora(subsref(&c, &[Index::Scalar(5)])), (5, 3));
    assert_eq!(
        fora(subsref(
            &b,
            &[Index::Scalar(1), Index::Scalar(4), Index::Scalar(1)]
        )),
        (4, 3)
    );
    assert_eq!(
        fora(subsref(
            &c,
            &[Index::Logical(vec![true, false, false, false, true])]
        )),
        (5, 3)
    );
}

/// Atribuição: escalar repetido em várias posições, no último índice sem crescer, e em 3-D;
/// remoção com subscrito de faixa completa (equivale a `:`) e posições dos erros.
#[test]
fn atribuicao_e_remocao() {
    let mut v = row(&[1.0, 2.0, 3.0]);
    subsasgn(&mut v, &[Index::List(vec![1, 3])], &Value::scalar(9.0)).unwrap();
    assert_eq!(f(&v), vec![9.0, 2.0, 9.0]);
    subsasgn(&mut v, &[Index::Scalar(3)], &Value::scalar(7.0)).unwrap();
    assert_eq!((v.dims(), f(&v)), (&[1usize, 3][..], vec![9.0, 2.0, 7.0]));
    let mut b = Value::doubles(&[2, 3, 2], vec![0.0; 12]);
    subsasgn(
        &mut b,
        &[Index::List(vec![1, 2]), Index::Scalar(2), Index::List(vec![1, 2])],
        &Value::doubles(&[2, 1, 2], vec![1.0, 2.0, 3.0, 4.0]),
    )
    .unwrap();
    assert_eq!(
        f(&b),
        vec![0.0, 0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 4.0, 0.0, 0.0]
    );
    // Crescimento por subscrito em 3-D.
    let mut g = Value::doubles(&[1, 1], vec![5.0]);
    subsasgn(
        &mut g,
        &[Index::Scalar(2), Index::Scalar(1), Index::Scalar(2)],
        &Value::scalar(8.0),
    )
    .unwrap();
    assert_eq!((g.dims(), f(&g)), (&[2usize, 1, 2][..], vec![5.0, 0.0, 0.0, 8.0]));
    // Remoção: A(1:2, 2) num 2×3 equivale a A(:, 2).
    let m = Value::doubles(&[2, 3], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let mut m1 = m.clone();
    subsdelete(&mut m1, &[Index::List(vec![1, 2]), Index::Scalar(2)]).unwrap();
    assert_eq!((m1.dims(), f(&m1)), (&[2usize, 2][..], vec![1.0, 2.0, 5.0, 6.0]));
    let mut m2 = m.clone();
    assert!(subsdelete(&mut m2, &[Index::List(vec![2, 1]), Index::Scalar(2)]).is_err());
    let mut m3 = m.clone();
    match subsdelete(&mut m3, &[Index::Scalar(7)]) {
        Err(RuntimeError::IndexOutOfBounds { index, bound }) => assert_eq!((index, bound), (7, 6)),
        o => panic!("{o:?}"),
    }
    let mut m4 = m.clone();
    match subsdelete(&mut m4, &[Index::All, Index::Scalar(4)]) {
        Err(RuntimeError::IndexOutOfBounds { index, bound }) => assert_eq!((index, bound), (4, 3)),
        o => panic!("{o:?}"),
    }
}

/// Subtração de doubles, nome da operação nas mensagens, formas da indexação linear em escalar e
/// em 1×2×2 com máscara, e atribuição linear no último elemento de uma matriz (sem crescer).
#[test]
fn subtracao_mensagens_e_formas() {
    assert_eq!(
        f(&elementwise(BinOp::Sub, &Value::scalar(5.0), &Value::scalar(3.0)).unwrap()),
        vec![2.0]
    );
    assert_eq!(
        f(&elementwise(BinOp::Sub, &row(&[1.0, 2.0]), &Value::scalar(0.5)).unwrap()),
        vec![0.5, 1.5]
    );
    match elementwise(BinOp::Add, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0])) {
        Err(RuntimeError::IncompatibleShapes { operation, .. }) => assert_eq!(operation, "soma"),
        o => panic!("{o:?}"),
    }
    match elementwise(BinOp::Sub, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0])) {
        Err(RuntimeError::IncompatibleShapes { operation, .. }) => assert_eq!(operation, "subtração"),
        o => panic!("{o:?}"),
    }
    // Escalar indexado por lista: linha.
    let e = subsref(&Value::scalar(7.0), &[Index::List(vec![1, 1, 1])]).unwrap();
    assert_eq!((e.dims(), f(&e)), (&[1usize, 3][..], vec![7.0, 7.0, 7.0]));
    // 1×2×2 com máscara lógica: coluna.
    let a = Value::doubles(&[1, 2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    let r = subsref(&a, &[Index::Logical(vec![true, false, true, true])]).unwrap();
    assert_eq!((r.dims(), f(&r)), (&[3usize, 1][..], vec![1.0, 3.0, 4.0]));
    // Matriz 2×2: A(4) = 9 não cresce nem falha.
    let mut m = Value::doubles(&[2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    subsasgn(&mut m, &[Index::Scalar(4)], &Value::scalar(9.0)).unwrap();
    assert_eq!((m.dims(), f(&m)), (&[2usize, 2][..], vec![1.0, 2.0, 3.0, 9.0]));
    assert!(subsasgn(&mut m, &[Index::Scalar(5)], &Value::scalar(1.0)).is_err());
}

/// Zero com expoente fracionário continua real; raiz de −0 é −0 (IEEE); produto matricial
/// recusa 3-D mesmo quando a forma "dobrada" seria compatível (mutação).
#[test]
fn zeros_e_produto_3d() {
    let r = elementwise(BinOp::Pow, &Value::scalar(0.0), &Value::scalar(0.5)).unwrap();
    assert!(!r.is_complex());
    assert_eq!(f(&r), vec![0.0]);
    let s = sqrt(&Value::scalar(-0.0)).unwrap();
    assert!(!s.is_complex());
    assert!(f(&s)[0] == 0.0 && f(&s)[0].is_sign_negative());
    let a = Value::doubles(&[2, 2, 2], vec![1.0; 8]);
    let b = Value::doubles(&[4, 1], vec![1.0; 4]);
    assert!(mtimes(&a, &b).is_err());
}

/// Concatenação (DT-38): valores e formas, vazios, classes e erros.
#[test]
fn concatenacao() {
    use chimalab_runtime::{concat, horzcat, vertcat};
    let m = |d: &[usize], v: &[f64]| Value::doubles(d, v.to_vec());
    // [1 2 3] e [1 2; 3 4] (column-major: 1, 3, 2, 4).
    let h = horzcat(&[row(&[1.0, 2.0]), Value::scalar(3.0)]).unwrap();
    assert_eq!((h.dims(), f(&h)), (&[1usize, 3][..], vec![1.0, 2.0, 3.0]));
    let v = vertcat(&[row(&[1.0, 2.0]), row(&[3.0, 4.0])]).unwrap();
    assert_eq!((v.dims(), f(&v)), (&[2usize, 2][..], vec![1.0, 3.0, 2.0, 4.0]));
    // [A B] com A 2×2 e B 2×1; [A; C] com C 1×2.
    let a = m(&[2, 2], &[1.0, 2.0, 3.0, 4.0]);
    let r = horzcat(&[a.clone(), m(&[2, 1], &[5.0, 6.0])]).unwrap();
    assert_eq!(
        (r.dims(), f(&r)),
        (&[2usize, 3][..], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
    );
    let r = vertcat(&[a.clone(), row(&[7.0, 8.0])]).unwrap();
    assert_eq!(
        (r.dims(), f(&r)),
        (&[3usize, 2][..], vec![1.0, 2.0, 7.0, 3.0, 4.0, 8.0])
    );
    // [] é ignorado; outros vazios contam nas dimensões.
    let r = horzcat(&[m(&[0, 0], &[]), row(&[1.0, 2.0]), m(&[0, 0], &[])]).unwrap();
    assert_eq!(f(&r), vec![1.0, 2.0]);
    let r = vertcat(&[m(&[0, 2], &[]), row(&[1.0, 2.0])]).unwrap();
    assert_eq!(r.dims(), &[1, 2]);
    assert!(vertcat(&[m(&[0, 3], &[]), row(&[1.0, 2.0])]).is_err());
    assert_eq!(horzcat(&[]).unwrap().dims(), &[0, 0]);
    let vazio_i8 = Value::doubles(&[0, 0], vec![])
        .cast(Class::Int(IntClass::I8))
        .unwrap();
    assert_eq!(horzcat(&[vazio_i8]).unwrap().class(), Class::Int(IntClass::I8));
    // Dimensões incompatíveis informam as duas formas.
    match horzcat(&[a.clone(), row(&[1.0, 2.0, 3.0])]) {
        Err(RuntimeError::IncompatibleShapes {
            operation,
            left,
            right,
        }) => {
            assert_eq!((operation, left, right), ("concatenação", vec![2, 2], vec![1, 3]));
        }
        o => panic!("{o:?}"),
    }
    // 3-D: ao longo da terceira dimensão e da segunda.
    let r = concat(3, &[a.clone(), m(&[2, 2], &[5.0, 6.0, 7.0, 8.0])]).unwrap();
    assert_eq!(
        (r.dims(), f(&r)),
        (&[2usize, 2, 2][..], (1..=8).map(f64::from).collect::<Vec<_>>())
    );
    let p = m(&[2, 1, 2], &[1.0, 2.0, 3.0, 4.0]);
    let q = m(&[2, 1, 2], &[5.0, 6.0, 7.0, 8.0]);
    let r = horzcat(&[p.clone(), q.clone()]).unwrap();
    assert_eq!(
        (r.dims(), f(&r)),
        (&[2usize, 2, 2][..], vec![1.0, 2.0, 5.0, 6.0, 3.0, 4.0, 7.0, 8.0])
    );
    let r = vertcat(&[p, q]).unwrap();
    assert_eq!(
        (r.dims(), f(&r)),
        (&[4usize, 1, 2][..], vec![1.0, 2.0, 5.0, 6.0, 3.0, 4.0, 7.0, 8.0])
    );
    assert!(concat(0, std::slice::from_ref(&a)).is_err());
    // Classes.
    let i8v = |x: f64| int(IntClass::I8, x);
    let r = horzcat(&[i8v(1.0), Value::scalar(1.6), Value::scalar(300.0)]).unwrap();
    assert_eq!(
        (r.class(), f(&r)),
        (Class::Int(IntClass::I8), vec![1.0, 2.0, 127.0])
    );
    assert!(horzcat(&[i8v(1.0), int(IntClass::I16, 2.0)]).is_err());
    let r = horzcat(&[i8v(1.0), i8v(-2.0)]).unwrap();
    assert_eq!((r.class(), f(&r)), (Class::Int(IntClass::I8), vec![1.0, -2.0]));
    let r = horzcat(&[i8v(1.0), Value::scalar(2.6).cast(Class::Single).unwrap()]).unwrap();
    assert_eq!((r.class(), f(&r)), (Class::Int(IntClass::I8), vec![1.0, 3.0]));
    let s1 = Value::scalar(1.5).cast(Class::Single).unwrap();
    let r = horzcat(&[Value::scalar(0.1), s1]).unwrap();
    assert_eq!((r.class(), f(&r)), (Class::Single, vec![0.1f32 as f64, 1.5]));
    let l = |b: bool| Value::Logical(NdArray::from_shape_vec(&[1, 1], vec![b]).unwrap());
    let r = horzcat(&[l(true), l(false)]).unwrap();
    assert_eq!((r.class(), f(&r)), (Class::Logical, vec![1.0, 0.0]));
    let r = horzcat(&[l(true), Value::scalar(5.0)]).unwrap();
    assert_eq!((r.class(), f(&r)), (Class::Double, vec![1.0, 5.0]));
    let r = horzcat(&[Value::text("ab"), Value::text("cd")]).unwrap();
    assert_eq!(r, Value::text("abcd"));
    let r = vertcat(&[Value::text("ab"), Value::text("cd")]).unwrap();
    assert_eq!(r.dims(), &[2, 2]);
    match horzcat(&[Value::text("a"), Value::scalar(66.0)]) {
        Err(RuntimeError::IncompatibleClasses {
            operation,
            left,
            right,
        }) => {
            assert_eq!(
                (operation, left.as_str(), right.as_str()),
                ("concatenação", "char", "double")
            );
        }
        o => panic!("{o:?}"),
    }
    assert!(horzcat(&[Value::scalar(66.0), Value::text("a")]).is_err());
    let r = horzcat(&[Value::scalar(1.0), z(0.0, 2.0)]).unwrap();
    assert_eq!(
        r.to_complex(),
        vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 2.0)]
    );
    assert!(horzcat(&[i8v(1.0), z(0.0, 2.0)]).is_err());
    assert!(horzcat(&[Value::scalar(1.0).cast(Class::Single).unwrap(), z(0.0, 2.0)]).is_err());
    // Inteiro depois de double e de single: a classe inteira domina em qualquer ordem.
    let r = horzcat(&[
        Value::scalar(2.4),
        Value::scalar(1.0).cast(Class::Single).unwrap(),
        i8v(3.0),
    ])
    .unwrap();
    assert_eq!(
        (r.class(), f(&r)),
        (Class::Int(IntClass::I8), vec![2.0, 1.0, 3.0])
    );
}

/// Potência de matriz (F005): inteira exata, negativa pela inversa, fracionária por
/// Schur–Parlett, escalar elevado a matriz, e os erros.
#[test]
fn potencia_de_matriz() {
    let m = |d: &[usize], v: &[f64]| Value::doubles(d, v.to_vec());
    let p = |a: &Value, b: f64| mpower(a, &Value::scalar(b)).unwrap();
    // Fibonacci: [1 1; 1 0]^n = [F(n+1) F(n); F(n) F(n−1)], exato até F(71) < 2⁵³.
    let fib = m(&[2, 2], &[1.0, 1.0, 1.0, 0.0]);
    let (r, w) = p(&fib, 10.0);
    assert!(w.is_empty());
    assert_eq!(f(&r), vec![89.0, 55.0, 55.0, 34.0]);
    let (r, _) = p(&fib, 70.0);
    assert_eq!(
        f(&r),
        vec![
            308061521170129.0,
            190392490709135.0,
            190392490709135.0,
            117669030460994.0
        ]
    );
    assert_eq!(f(&p(&fib, 1.0).0), f(&fib));
    assert_eq!(f(&p(&fib, 0.0).0), vec![1.0, 0.0, 0.0, 1.0]);
    // Negativa pela inversa; singular avisa.
    let d = m(&[2, 2], &[2.0, 0.0, 0.0, 4.0]);
    assert_eq!(f(&p(&d, -2.0).0), vec![0.25, 0.0, 0.0, 0.0625]);
    let (_, w) = p(&m(&[2, 2], &[1.0, 2.0, 2.0, 4.0]), -1.0);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].id(), "chimalab:runtime:matrizSingular");
    // Fracionária: raiz de SPD ao quadrado volta a A; autovalor negativo dá complexo.
    let a = m(&[2, 2], &[2.0, 1.0, 1.0, 2.0]);
    let (r, _) = p(&a, 0.5);
    assert!(!r.is_complex());
    let r2 = f(&mtimes(&r, &r).unwrap());
    assert!(
        r2.iter().zip(f(&a)).all(|(x, y)| (x - y).abs() <= 1e-14),
        "{r2:?}"
    );
    let (r, _) = p(&m(&[2, 2], &[-4.0, 0.0, 0.0, 9.0]), 0.5);
    let z = r.to_complex();
    assert!(
        (z[0] - Complex64::new(0.0, 2.0)).norm() <= 1e-14
            && (z[3] - Complex64::new(3.0, 0.0)).norm() <= 1e-14,
        "{z:?}"
    );
    // A^0 de matriz singular: identidade, sem aviso (não inverte).
    let (r, w) = p(&m(&[2, 2], &[1.0, 2.0, 2.0, 4.0]), 0.0);
    assert!(w.is_empty());
    assert_eq!(f(&r), vec![1.0, 0.0, 0.0, 1.0]);
    // Blocos de Jordan (autovalor repetido: entram as derivadas de z^p). Para J = 4I + N,
    // J^½ = 2I + ¼N − (1/64)N², com f(4) = 2, f′(4) = ¼ e f″(4)/2 = −1/64.
    let (r, _) = p(&m(&[2, 2], &[4.0, 0.0, 1.0, 4.0]), 0.5);
    let v = f(&r);
    assert!(
        (v[0] - 2.0).abs() <= 1e-14
            && v[1].abs() <= 1e-14
            && (v[2] - 0.25).abs() <= 1e-14
            && (v[3] - 2.0).abs() <= 1e-14,
        "{v:?}"
    );
    let (r, _) = p(&m(&[3, 3], &[4.0, 0.0, 0.0, 1.0, 4.0, 0.0, 0.0, 1.0, 4.0]), 0.5);
    let v = f(&r);
    let esp = [2.0, 0.0, 0.0, 0.25, 2.0, 0.0, -1.0 / 64.0, 0.25, 2.0];
    assert!(v.iter().zip(esp).all(|(x, y)| (x - y).abs() <= 1e-13), "{v:?}");
    // Escalar elevado a matriz nilpotente: 2^B = I + ln2·B.
    let b = m(&[2, 2], &[0.0, 0.0, 1.0, 0.0]);
    let (r, _) = mpower(&Value::scalar(2.0), &b).unwrap();
    let v = f(&r);
    assert!(
        (v[0] - 1.0).abs() <= 1e-15
            && v[1] == 0.0
            && (v[2] - 2f64.ln()).abs() <= 1e-15
            && (v[3] - 1.0).abs() <= 1e-15,
        "{v:?}"
    );
    // (−1)^diag(1, 2) = diag(−1, 1), pelo ramo principal.
    let (r, _) = mpower(&Value::scalar(-1.0), &m(&[2, 2], &[1.0, 0.0, 0.0, 2.0])).unwrap();
    let z = r.to_complex();
    assert!(
        (z[0] + 1.0).norm() <= 1e-14 && (z[3] - 1.0).norm() <= 1e-14,
        "{z:?}"
    );
    // Escalar com escalar: a potência comum.
    assert_eq!(
        f(&mpower(&Value::scalar(2.0), &Value::scalar(10.0)).unwrap().0),
        vec![1024.0]
    );
    // Erros: não quadrada, matriz com matriz, complexo.
    let erro_forma = |r: Result<(Value, Vec<chimalab_runtime::Warning>), RuntimeError>| {
        matches!(
            r,
            Err(RuntimeError::IncompatibleShapes {
                operation: "potência matricial",
                ..
            })
        )
    };
    assert!(erro_forma(mpower(&row(&[1.0, 2.0]), &Value::scalar(2.0))));
    assert!(erro_forma(mpower(&fib, &fib)));
    assert!(erro_forma(mpower(&Value::scalar(2.0), &row(&[1.0, 2.0]))));
    let zc = Value::Complex(NdArray::from_shape_vec(&[2, 2], vec![Complex64::new(1.0, 1.0); 4]).unwrap());
    assert!(matches!(
        mpower(&zc, &Value::scalar(2.0)),
        Err(RuntimeError::IncompatibleClasses { .. })
    ));
}

/// Divisão à direita A/B (F005): solução de X·B = A, com X conhecido.
#[test]
fn divisao_a_direita() {
    let m = |d: &[usize], v: &[f64]| Value::doubles(d, v.to_vec());
    // X = [1 2; 3 4], B = [2 0; 1 1] ⇒ A = X·B = [4 2; 10 4] (column-major abaixo).
    let b = m(&[2, 2], &[2.0, 1.0, 0.0, 1.0]);
    let a = m(&[2, 2], &[4.0, 10.0, 2.0, 4.0]);
    let (x, w) = mrdivide(&a, &b).unwrap();
    assert!(w.is_empty());
    assert_eq!(x.dims(), &[2, 2]);
    assert!(
        f(&x)
            .iter()
            .zip([1.0, 3.0, 2.0, 4.0])
            .all(|(p, q)| (p - q).abs() <= 1e-14),
        "{:?}",
        f(&x)
    );
    // Linha dividida por matriz: [4 2]/B = [1 2].
    let (x, _) = mrdivide(&row(&[4.0, 2.0]), &b).unwrap();
    assert_eq!(x.dims(), &[1, 2]);
    assert!(f(&x).iter().zip([1.0, 2.0]).all(|(p, q)| (p - q).abs() <= 1e-14));
    // Escalar no divisor: elemento a elemento.
    assert_eq!(
        f(&mrdivide(&row(&[2.0, 4.0]), &Value::scalar(2.0)).unwrap().0),
        vec![1.0, 2.0]
    );
    // Colunas incompatíveis; divisor singular avisa.
    match mrdivide(&m(&[2, 3], &[0.0; 6]), &b) {
        Err(RuntimeError::IncompatibleShapes {
            operation,
            left,
            right,
        }) => {
            assert_eq!((operation, left, right), ("/", vec![2, 3], vec![2, 2]));
        }
        o => panic!("{o:?}"),
    }
    let (_, w) = mrdivide(&row(&[1.0, 1.0]), &m(&[2, 2], &[1.0, 2.0, 2.0, 4.0])).unwrap();
    assert_eq!(w.first().map(|x| x.id()), Some("chimalab:runtime:matrizSingular"));
}

/// Produto matricial e A\b complexos e `single` (F005); inteiros e `single` complexo recusados.
#[test]
fn produto_e_divisao_complexos_e_single() {
    let c = |d: &[usize], v: &[(f64, f64)]| {
        Value::Complex(
            NdArray::from_shape_vec(d, v.iter().map(|&(r, i)| Complex64::new(r, i)).collect()).unwrap(),
        )
    };
    // [1 i; 0 1]·[1; i] = [1 + i·i; i] = [0; i].
    let a = c(&[2, 2], &[(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 0.0)]);
    let r = mtimes(&a, &c(&[2, 1], &[(1.0, 0.0), (0.0, 1.0)])).unwrap();
    assert_eq!(
        r.to_complex(),
        vec![Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0)]
    );
    // iI·(−iI) = I: parte imaginária nula volta a real.
    let ii = c(&[2, 2], &[(0.0, 1.0), (0.0, 0.0), (0.0, 0.0), (0.0, 1.0)]);
    let mi = c(&[2, 2], &[(0.0, -1.0), (0.0, 0.0), (0.0, 0.0), (0.0, -1.0)]);
    let r = mtimes(&ii, &mi).unwrap();
    assert!(!r.is_complex());
    assert_eq!(f(&r), vec![1.0, 0.0, 0.0, 1.0]);
    // Real com complexo.
    let r = mtimes(&row(&[1.0, 2.0]), &c(&[2, 1], &[(0.0, 1.0), (3.0, 0.0)])).unwrap();
    assert_eq!(r.to_complex(), vec![Complex64::new(6.0, 1.0)]);
    // single: classe single, acumulado em double e arredondado uma vez.
    let s1 = Value::doubles(&[1, 2], vec![0.1, 0.2])
        .cast(Class::Single)
        .unwrap();
    let r = mtimes(&s1, &Value::doubles(&[2, 1], vec![3.0, 4.0])).unwrap();
    assert_eq!(r.class(), Class::Single);
    let esp = (f64::from(0.1f32) * 3.0 + f64::from(0.2f32) * 4.0) as f32;
    assert_eq!(f(&r), vec![f64::from(esp)]);
    // Recusados: inteiros e single complexo; N-D.
    let i8m = Value::doubles(&[2, 2], vec![1.0; 4])
        .cast(Class::Int(IntClass::I8))
        .unwrap();
    assert!(matches!(
        mtimes(&i8m, &i8m),
        Err(RuntimeError::IncompatibleClasses { .. })
    ));
    assert!(matches!(
        mtimes(&s1, &c(&[2, 1], &[(1.0, 1.0), (0.0, 0.0)])),
        Err(RuntimeError::IncompatibleClasses { .. })
    ));
    assert!(matches!(
        mtimes(&row(&[1.0, 2.0]), &row(&[1.0, 2.0])),
        Err(RuntimeError::IncompatibleShapes { .. })
    ));
    // A\b complexo: x = [1; 1] com A = [1 i; 0 2] ⇒ b = [1 + i; 2].
    let a = c(&[2, 2], &[(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (2.0, 0.0)]);
    let b = c(&[2, 1], &[(1.0, 1.0), (2.0, 0.0)]);
    let (x, w) = mldivide(&a, &b).unwrap();
    assert!(w.is_empty());
    assert!(
        x.to_complex()
            .iter()
            .all(|z| (z - Complex64::new(1.0, 0.0)).norm() <= 1e-15),
        "{:?}",
        x.to_complex()
    );
    // Complexo com solução de fato complexa: A = [2 0; 0 1], b = [2i; 1 − i].
    let (x, _) = mldivide(
        &Value::doubles(&[2, 2], vec![2.0, 0.0, 0.0, 1.0]),
        &c(&[2, 1], &[(0.0, 2.0), (1.0, -1.0)]),
    )
    .unwrap();
    assert_eq!(
        x.to_complex(),
        vec![Complex64::new(0.0, 1.0), Complex64::new(1.0, -1.0)]
    );
    assert!(matches!(
        mldivide(&i8m, &i8m),
        Err(RuntimeError::IncompatibleClasses { .. })
    ));
    let s2 = Value::doubles(&[2, 2], vec![1.0, 0.0, 0.0, 1.0])
        .cast(Class::Single)
        .unwrap();
    assert!(matches!(
        mldivide(&s2, &s2),
        Err(RuntimeError::IncompatibleClasses { .. })
    ));
    assert!(matches!(
        mldivide(&a, &c(&[3, 1], &[(1.0, 0.0); 3])),
        Err(RuntimeError::IncompatibleShapes { .. })
    ));
}

/// Exemplos da especificação, parte 4 (`docs/linguagem/04-valores.md`).
#[test]
fn exemplos_da_especificacao_valores() {
    assert_eq!(f(&int(IntClass::U8, 3.5)), vec![4.0]);
    assert_eq!(f(&int(IntClass::I8, -2.5)), vec![-3.0]);
    assert_eq!(f(&int(IntClass::I8, 300.0)), vec![127.0]);
    assert_eq!(f(&int(IntClass::U8, -5.0)), vec![0.0]);
    assert_eq!(f(&int(IntClass::I32, f64::NAN)), vec![0.0]);
    assert!(Value::scalar(f64::NAN).cast(Class::Logical).is_err());
    let r = elementwise(BinOp::Add, &int(IntClass::I8, 127.0), &Value::scalar(1.0)).unwrap();
    assert_eq!(f(&r), vec![127.0]);
    assert!(elementwise(BinOp::Add, &int(IntClass::I8, 1.0), &int(IntClass::U8, 1.0)).is_err());
    let s = sqrt(&Value::scalar(-4.0)).unwrap();
    assert_eq!(s.to_complex(), vec![Complex64::new(0.0, 2.0)]);
    let p = elementwise(BinOp::Pow, &Value::scalar(-8.0), &Value::scalar(2.0)).unwrap();
    assert_eq!((p.is_complex(), f(&p)), (false, vec![64.0]));
    let c = Value::doubles(&[2, 1], vec![1.0, 2.0]);
    assert_eq!(
        elementwise(BinOp::Add, &c, &row(&[1.0, 2.0, 3.0]))
            .unwrap()
            .dims(),
        &[2, 3]
    );
    assert!(elementwise(BinOp::Add, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0])).is_err());
    assert!(matches!(
        subsref(&row(&[1.0, 2.0]), &[Index::Scalar(0)]),
        Err(RuntimeError::InvalidIndex(_))
    ));
}

mod propriedades {
    use super::*;
    use chimalab_runtime::{concat, horzcat, mldivide, mpower, mrdivide, mtimes, vertcat};
    use proptest::prelude::*;

    fn matriz(r: usize, c: usize, v: &[i8]) -> Value {
        Value::doubles(&[r, c], v.iter().take(r * c).map(|&x| f64::from(x)).collect())
    }

    proptest! {
        /// [A B] seguido das fatias de colunas devolve A e B; [A; B] idem com as linhas.
        #[test]
        fn concatenacao_e_fatias(r in 1usize..4, ca in 0usize..4, cb in 1usize..4, v in prop::collection::vec(-9i8..10, 32)) {
            let a = matriz(r, ca, &v);
            let b = matriz(r, cb, &v[12..]);
            let h = horzcat(&[a.clone(), b.clone()]).unwrap();
            prop_assert_eq!(h.dims(), &[r, ca + cb][..]);
            let cols = |lo: usize, hi: usize| Index::List((lo + 1..=hi).collect());
            if ca > 0 {
                prop_assert_eq!(f(&subsref(&h, &[Index::All, cols(0, ca)]).unwrap()), f(&a));
            }
            prop_assert_eq!(f(&subsref(&h, &[Index::All, cols(ca, ca + cb)]).unwrap()), f(&b));
            // Associatividade e equivalência com concat(2, …).
            let c = matriz(r, 1, &v[20..]);
            let e1 = horzcat(&[horzcat(&[a.clone(), b.clone()]).unwrap(), c.clone()]).unwrap();
            let e2 = horzcat(&[a.clone(), horzcat(&[b.clone(), c.clone()]).unwrap()]).unwrap();
            prop_assert_eq!(f(&e1), f(&e2));
            prop_assert_eq!(f(&concat(2, &[a, b, c]).unwrap()), f(&e1));
            // Vertical: [Aᵀ; Bᵀ] tem as linhas certas.
            let at = matriz(ca.max(1), r, &v);
            let bt = matriz(cb, r, &v[12..]);
            let vv = vertcat(&[at.clone(), bt.clone()]).unwrap();
            prop_assert_eq!(vv.dims(), &[ca.max(1) + cb, r][..]);
            let linhas = |lo: usize, hi: usize| Index::List((lo + 1..=hi).collect());
            prop_assert_eq!(f(&subsref(&vv, &[linhas(0, ca.max(1)), Index::All]).unwrap()), f(&at));
        }

        /// A^(p+q) = A^p·A^q, exato para matrizes inteiras pequenas.
        #[test]
        fn potencia_inteira_soma_expoentes(n in 1usize..4, v in prop::collection::vec(-2i8..3, 9), p in 0u32..5, q in 0u32..5) {
            let a = matriz(n, n, &v);
            let pp = |k: u32| mpower(&a, &Value::scalar(f64::from(k))).unwrap().0;
            prop_assert_eq!(f(&pp(p + q)), f(&mtimes(&pp(p), &pp(q)).unwrap()));
        }

        /// (A/B)·B ≈ A e B·(B\A) ≈ A com B bem condicionada (diagonal dominante).
        #[test]
        fn divisoes_resolvem(n in 1usize..5, v in prop::collection::vec(-5i8..6, 40)) {
            let mut b = matriz(n, n, &v);
            for i in 0..n {
                let d = f(&subsref(&b, &[Index::Scalar(i + 1), Index::Scalar(i + 1)]).unwrap())[0];
                subsasgn(&mut b, &[Index::Scalar(i + 1), Index::Scalar(i + 1)], &Value::scalar(d + 40.0)).unwrap();
            }
            let a = matriz(2, n, &v[20..]);
            let x = mrdivide(&a, &b).unwrap().0;
            let ax = f(&mtimes(&x, &b).unwrap());
            prop_assert!(ax.iter().zip(f(&a)).all(|(p, q)| (p - q).abs() <= 1e-12 * (1.0 + q.abs())), "{:?}", ax);
            let at = matriz(n, 2, &v[20..]);
            let y = mldivide(&b, &at).unwrap().0;
            let by = f(&mtimes(&b, &y).unwrap());
            prop_assert!(by.iter().zip(f(&at)).all(|(p, q)| (p - q).abs() <= 1e-12 * (1.0 + q.abs())));
        }
    }
}

/// `linspace` (semântica de `range(a, b; length=n)`): extremos exatos, passo uniforme, casos
/// degenerados e extremos enormes sem estouro.
#[test]
fn pontos_igualmente_espacados() {
    use chimalab_runtime::linspace;
    assert_eq!(f(&linspace(0.0, 1.0, 5)), vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(linspace(0.0, 1.0, 5).dims(), &[1, 5]);
    // O último é exatamente b, mesmo com passo inexato (0,1).
    let v = f(&linspace(0.0, 1.0, 11));
    assert_eq!((v.len(), v[0], v[10]), (11, 0.0, 1.0));
    assert!(v.windows(2).all(|w| ((w[1] - w[0]) - 0.1).abs() <= 1e-15));
    // 0,1 + 3·(0,2/3) dá 0,30000000000000004: o último é forçado a ser exatamente b.
    assert_eq!(*f(&linspace(0.1, 0.3, 4)).last().unwrap(), 0.3);
    assert_eq!(f(&linspace(3.0, -1.0, 3)), vec![3.0, 1.0, -1.0]);
    assert_eq!(f(&linspace(2.0, 2.0, 3)), vec![2.0, 2.0, 2.0]);
    assert_eq!(f(&linspace(0.0, 7.0, 1)), vec![7.0]);
    let e = linspace(0.0, 1.0, 0);
    assert_eq!((e.dims(), e.numel()), (&[1usize, 0][..], 0));
    // Extremos de sinais opostos perto do maior f64: b − a estoura, e a fórmula ponderada evita ∞.
    let w = f(&linspace(-f64::MAX, f64::MAX, 3));
    assert_eq!(w, vec![-f64::MAX, 0.0, f64::MAX]);
    assert!(f(&linspace(-f64::MAX, f64::MAX, 5)).iter().all(|x| x.is_finite()));
}

/// Regressão: um erro depois da promoção de classe deixava o alvo alterado (double virava
/// complexo; vazio trocava de classe). Agora o alvo fica intacto.
#[test]
fn atribuicao_com_erro_preserva_o_alvo() {
    let mut v = Value::doubles(&[1, 2], vec![1.0, 2.0]);
    let z3 = Value::Complex(NdArray::from_shape_vec(&[1, 3], vec![Complex64::new(1.0, 1.0); 3]).unwrap());
    let e = subsasgn(&mut v, &[Index::List(vec![1, 2])], &z3).unwrap_err();
    assert_eq!(e.id(), "chimalab:runtime:dimensoesIncompativeis");
    assert_eq!(v, Value::doubles(&[1, 2], vec![1.0, 2.0]));
    let mut vazio = Value::doubles(&[0, 0], Vec::new());
    let i3 = Value::Int(chimalab_runtime::IntData::from_i128(
        chimalab_runtime::IntClass::I8,
        &[1, 3],
        vec![1, 2, 3],
    ));
    assert!(subsasgn(&mut vazio, &[Index::List(vec![1, 2])], &i3).is_err());
    assert_eq!(vazio, Value::doubles(&[0, 0], Vec::new()));
    // Erro sem promoção (índice zero, crescimento proibido) também não toca o alvo.
    let mut m = Value::doubles(&[2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    assert!(subsasgn(&mut m, &[Index::Scalar(0)], &Value::scalar(9.0)).is_err());
    assert!(subsasgn(&mut m, &[Index::Scalar(9)], &Value::scalar(9.0)).is_err());
    assert_eq!(m, Value::doubles(&[2, 2], vec![1.0, 2.0, 3.0, 4.0]));
    // Com sucesso, as promoções acontecem.
    let mut v = Value::doubles(&[1, 2], vec![1.0, 2.0]);
    subsasgn(&mut v, &[Index::Scalar(2)], &z3_um()).unwrap();
    assert!(v.is_complex());
    let mut vazio = Value::doubles(&[0, 0], Vec::new());
    subsasgn(
        &mut vazio,
        &[Index::Scalar(2)],
        &Value::Logical(NdArray::filled_nd(&[1, 1], true)),
    )
    .unwrap();
    assert_eq!(vazio.class().name(), "logical");
    // `end_index` livre bate com o método.
    for (dims, k, n) in [
        (vec![2, 3, 4], 0, 1),
        (vec![2, 3, 4], 1, 2),
        (vec![2, 3], 2, 3),
        (vec![0, 3], 0, 2),
    ] {
        let a = Value::doubles(&dims, vec![0.0; dims.iter().product()]);
        assert_eq!(a.end_index(k, n), chimalab_runtime::end_index(&dims, k, n));
    }
}

fn z3_um() -> Value {
    Value::Complex(NdArray::from_shape_vec(&[1, 1], vec![Complex64::new(0.0, 1.0)]).unwrap())
}
