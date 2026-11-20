//! F003 (tipos), F004 (indexação) e F005 (operadores): semântica do modelo de valores.
//!
//! Os testes seguem os CAs do catálogo literalmente, escritos na API do runtime (a sintaxe da
//! linguagem é decidida nas perguntas 27–36).

use chimalab_array::NdArray;
use chimalab_runtime::{
    BinOp, Class, CmpOp, Index, IntClass, RuntimeError, Value, compare, elementwise, mldivide, mtimes, neg,
    sqrt, subsasgn, subsdelete, subsref,
};
use proptest::prelude::*;

fn int(c: IntClass, v: f64) -> Value {
    Value::scalar(v).cast(Class::Int(c)).unwrap()
}

fn f(v: &Value) -> Vec<f64> {
    v.to_f64s().unwrap()
}

fn row(v: &[f64]) -> Value {
    Value::doubles(&[1, v.len()], v.to_vec())
}

fn col(v: &[f64]) -> Value {
    Value::doubles(&[v.len(), 1], v.to_vec())
}

// ---------------------------------------------------------------- F003

#[test]
fn ca_f003_int8_127_mais_1_satura() {
    let r = elementwise(BinOp::Add, &int(IntClass::I8, 127.0), &Value::scalar(1.0)).unwrap();
    assert_eq!(r.class(), Class::Int(IntClass::I8));
    assert_eq!(f(&r), vec![127.0]);
}

#[test]
fn ca_f003_uint8_de_3_5_arredonda_para_4() {
    assert_eq!(f(&int(IntClass::U8, 3.5)), vec![4.0]);
    // Meio para longe de zero também nos negativos; NaN vira 0; saturação nas pontas.
    assert_eq!(f(&int(IntClass::I8, -2.5)), vec![-3.0]);
    assert_eq!(f(&int(IntClass::I8, f64::NAN)), vec![0.0]);
    assert_eq!(f(&int(IntClass::U8, -7.0)), vec![0.0]);
    assert_eq!(f(&int(IntClass::U8, 1e300)), vec![255.0]);
    assert_eq!(f(&int(IntClass::I64, f64::INFINITY)), vec![i64::MAX as f64]);
}

#[test]
fn ca_f003_classe_de_int16_vezes_2_7() {
    let r = elementwise(BinOp::Mul, &int(IntClass::I16, 1.0), &Value::scalar(2.7)).unwrap();
    assert_eq!(r.class().name(), "int16");
    assert_eq!(f(&r), vec![3.0]);
}

#[test]
fn ca_f003_sqrt_de_menos_4_e_2i() {
    let r = sqrt(&Value::scalar(-4.0)).unwrap();
    assert!(r.is_complex());
    let z = r.to_complex()[0];
    assert_eq!((z.re, z.im), (0.0, 2.0));
    // Positivos continuam reais.
    assert!(!sqrt(&Value::scalar(9.0)).unwrap().is_complex());
}

#[test]
fn regras_de_classe() {
    let i8v = int(IntClass::I8, 5.0);
    let i16v = int(IntClass::I16, 5.0);
    // Inteiros de classes diferentes: erro.
    let e = elementwise(BinOp::Add, &i8v, &i16v).unwrap_err();
    assert_eq!(e.id(), "chimalab:runtime:classesIncompativeis");
    // Inteiro com complexo: erro.
    let z = sqrt(&Value::scalar(-1.0)).unwrap();
    assert_eq!(
        elementwise(BinOp::Add, &i8v, &z).unwrap_err().id(),
        "chimalab:runtime:classesIncompativeis"
    );
    // single domina double; lógico e char viram double.
    let s = Value::scalar(1.5).cast(Class::Single).unwrap();
    assert_eq!(
        elementwise(BinOp::Add, &s, &Value::scalar(1.0)).unwrap().class(),
        Class::Single
    );
    let l = Value::scalar(1.0).cast(Class::Logical).unwrap();
    assert_eq!(elementwise(BinOp::Add, &l, &l).unwrap().class(), Class::Double);
    let c = Value::text("A");
    assert_eq!(
        f(&elementwise(BinOp::Add, &c, &Value::scalar(1.0)).unwrap()),
        vec![66.0]
    );
    // Inteiro com single: inteiro.
    assert_eq!(
        elementwise(BinOp::Mul, &i8v, &s).unwrap().class(),
        Class::Int(IntClass::I8)
    );
    // Divisão inteira: arredonda; por zero satura; 0/0 = 0.
    let d = |a: f64, b: f64| {
        f(&elementwise(BinOp::Div, &int(IntClass::I8, a), &int(IntClass::I8, b)).unwrap())[0]
    };
    assert_eq!(d(7.0, 2.0), 4.0);
    assert_eq!(d(-7.0, 2.0), -4.0);
    assert_eq!(d(5.0, 0.0), 127.0);
    assert_eq!(d(-5.0, 0.0), -128.0);
    assert_eq!(d(0.0, 0.0), 0.0);
    assert_eq!(
        f(&elementwise(BinOp::LeftDiv, &int(IntClass::I8, 2.0), &int(IntClass::I8, 7.0)).unwrap()),
        vec![4.0]
    );
    // int64 exato acima de 2⁵³ (melhoria sobre calcular em double).
    let grande = Value::Int(chimalab_runtime::IntData::I64(NdArray::filled_nd(
        &[1, 1],
        (1i64 << 60) + 1,
    )));
    let soma = elementwise(BinOp::Add, &grande, &Value::scalar(1.0)).unwrap();
    match soma {
        Value::Int(chimalab_runtime::IntData::I64(a)) => assert_eq!(a.as_slice()[0], (1i64 << 60) + 2),
        other => panic!("{other:?}"),
    }
    // Potência inteira: em double, arredonda e satura.
    assert_eq!(
        f(&elementwise(BinOp::Pow, &int(IntClass::U8, 2.0), &Value::scalar(9.0)).unwrap()),
        vec![255.0]
    );
    // −int8(−128) satura em 127.
    assert_eq!(f(&neg(&int(IntClass::I8, -128.0)).unwrap()), vec![127.0]);
    assert_eq!(f(&neg(&Value::scalar(2.0)).unwrap()), vec![-2.0]);
    assert_eq!(neg(&s).unwrap().class(), Class::Single);
}

#[test]
fn conversoes() {
    let x = Value::doubles(&[1, 4], vec![0.0, 2.0, -1.0, f64::NAN]);
    assert!(x.cast(Class::Logical).is_err());
    let l = Value::doubles(&[1, 3], vec![0.0, 2.0, -1.0])
        .cast(Class::Logical)
        .unwrap();
    assert_eq!(f(&l), vec![0.0, 1.0, 1.0]);
    assert_eq!(l.class().name(), "logical");
    assert_eq!(Value::scalar(72.0).cast(Class::Char).unwrap(), Value::text("H"));
    assert_eq!(
        Value::text("Hi").cast(Class::Double).unwrap(),
        row(&[72.0, 105.0])
    );
    let z = sqrt(&Value::scalar(-1.0)).unwrap();
    assert_eq!(
        z.cast(Class::Int(IntClass::I8)).unwrap_err().id(),
        "chimalab:runtime:conversaoInvalida"
    );
    assert!(z.to_f64s().is_err());
    // Entre inteiros: satura.
    assert_eq!(
        f(&int(IntClass::I16, 300.0).cast(Class::Int(IntClass::U8)).unwrap()),
        vec![255.0]
    );
    // Nomes de todas as classes.
    let nomes: Vec<&str> = [
        IntClass::I8,
        IntClass::I16,
        IntClass::I32,
        IntClass::I64,
        IntClass::U8,
        IntClass::U16,
        IntClass::U32,
        IntClass::U64,
    ]
    .iter()
    .map(|c| Class::Int(*c).name())
    .collect();
    assert_eq!(
        nomes,
        [
            "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64"
        ]
    );
    assert_eq!(
        (Class::Double.name(), Class::Single.name(), Class::Char.name()),
        ("double", "single", "char")
    );
}

// ---------------------------------------------------------------- F005

#[test]
fn ca_f005_expansao_1x3_mais_2x1_da_2x3() {
    let r = elementwise(BinOp::Add, &row(&[1.0, 2.0, 3.0]), &col(&[10.0, 20.0])).unwrap();
    assert_eq!(r.dims(), &[2, 3]);
    assert_eq!(f(&r), vec![11.0, 21.0, 12.0, 22.0, 13.0, 23.0]);
    let e = elementwise(BinOp::Add, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0])).unwrap_err();
    assert_eq!(e.id(), "chimalab:runtime:dimensoesIncompativeis");
}

#[test]
fn ca_f005_barra_invertida_resolve_e_avisa_singular() {
    // Bem condicionada: resíduo < 1e-12.
    let a = Value::doubles(&[3, 3], vec![4.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0]);
    let b = col(&[1.0, 2.0, 3.0]);
    let (x, avisos) = mldivide(&a, &b).unwrap();
    assert!(avisos.is_empty());
    let ax = mtimes(&a, &x).unwrap();
    let res: f64 = f(&ax)
        .iter()
        .zip(f(&b))
        .map(|(p, q)| (p - q).abs())
        .fold(0.0, f64::max);
    assert!(res < 1e-12, "{res}");
    // Singular: aviso, não erro.
    let s = Value::doubles(&[2, 2], vec![1.0, 2.0, 2.0, 4.0]);
    let (_, avisos) = mldivide(&s, &col(&[1.0, 2.0])).unwrap();
    assert_eq!(avisos.len(), 1);
    assert_eq!(avisos[0].id(), "chimalab:runtime:matrizSingular");
    // Escalar: divisão elemento a elemento.
    let (x, _) = mldivide(&Value::scalar(2.0), &row(&[4.0, 6.0])).unwrap();
    assert_eq!(f(&x), vec![2.0, 3.0]);
    // Dimensões e classes incompatíveis.
    assert_eq!(
        mldivide(&a, &col(&[1.0, 2.0])).unwrap_err().id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    assert_eq!(
        mtimes(&a, &row(&[1.0, 2.0])).unwrap_err().id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    let ai = a.cast(Class::Int(IntClass::I32)).unwrap();
    assert_eq!(
        mtimes(&ai, &a).unwrap_err().id(),
        "chimalab:runtime:classesIncompativeis"
    );
    // Produto por escalar.
    assert_eq!(
        f(&mtimes(&Value::scalar(2.0), &row(&[1.0, 2.0])).unwrap()),
        vec![2.0, 4.0]
    );
}

#[test]
fn comparacoes_e_potencias() {
    let r = compare(CmpOp::Lt, &row(&[1.0, 2.0, 3.0]), &Value::scalar(2.0)).unwrap();
    assert_eq!(r.class(), Class::Logical);
    assert_eq!(f(&r), vec![1.0, 0.0, 0.0]);
    for (op, esperado) in [
        (CmpOp::Eq, [0.0, 1.0, 0.0]),
        (CmpOp::Ne, [1.0, 0.0, 1.0]),
        (CmpOp::Le, [1.0, 1.0, 0.0]),
        (CmpOp::Gt, [0.0, 0.0, 1.0]),
        (CmpOp::Ge, [0.0, 1.0, 1.0]),
    ] {
        assert_eq!(
            f(&compare(op, &row(&[1.0, 2.0, 3.0]), &Value::scalar(2.0)).unwrap()),
            esperado.to_vec()
        );
    }
    // NaN: só ~= é verdadeiro.
    assert_eq!(
        f(&compare(CmpOp::Eq, &Value::scalar(f64::NAN), &Value::scalar(f64::NAN)).unwrap()),
        vec![0.0]
    );
    assert_eq!(
        f(&compare(CmpOp::Ne, &Value::scalar(f64::NAN), &Value::scalar(1.0)).unwrap()),
        vec![1.0]
    );
    // Inteiros grandes comparados exatamente.
    let a = Value::Int(chimalab_runtime::IntData::I64(NdArray::filled_nd(
        &[1, 1],
        (1i64 << 60) + 1,
    )));
    let b = Value::Int(chimalab_runtime::IntData::I64(NdArray::filled_nd(
        &[1, 1],
        1i64 << 60,
    )));
    assert_eq!(f(&compare(CmpOp::Gt, &a, &b).unwrap()), vec![1.0]);
    // Complexos: igualdade exata; ordem pela parte real.
    let z = sqrt(&Value::scalar(-1.0)).unwrap();
    assert_eq!(f(&compare(CmpOp::Eq, &z, &z).unwrap()), vec![1.0]);
    assert_eq!(f(&compare(CmpOp::Ne, &z, &z).unwrap()), vec![0.0]);
    assert_eq!(
        f(&compare(CmpOp::Lt, &z, &Value::scalar(1.0)).unwrap()),
        vec![1.0]
    );
    assert_eq!(
        compare(CmpOp::Eq, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0]))
            .unwrap_err()
            .id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    // Potência de base negativa com expoente fracionário: complexo; inteiro: real.
    let p = elementwise(BinOp::Pow, &Value::scalar(-8.0), &Value::scalar(1.0 / 3.0)).unwrap();
    assert!(p.is_complex());
    assert_eq!(
        f(&elementwise(BinOp::Pow, &Value::scalar(-2.0), &Value::scalar(3.0)).unwrap()),
        vec![-8.0]
    );
    // Complexo com expoente inteiro: exato; parte imaginária nula vira real.
    let i = sqrt(&Value::scalar(-1.0)).unwrap();
    let i2 = elementwise(BinOp::Pow, &i, &Value::scalar(2.0)).unwrap();
    assert!(!i2.is_complex());
    assert_eq!(f(&i2), vec![-1.0]);
    // Operações complexas restantes.
    for op in [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div, BinOp::LeftDiv] {
        let r = elementwise(op, &i, &Value::scalar(2.0)).unwrap();
        assert!(r.is_complex() || f(&r).len() == 1);
    }
    // Raiz de i: complexo (não inteiro).
    assert!(
        elementwise(BinOp::Pow, &i, &Value::scalar(0.5))
            .unwrap()
            .is_complex()
    );
    // sqrt de complexo, de inteiro e de single negativo (erro).
    assert!(sqrt(&i).unwrap().is_complex());
    assert_eq!(f(&sqrt(&int(IntClass::U8, 10.0)).unwrap()), vec![3.0]);
    let sn = Value::scalar(-1.0).cast(Class::Single).unwrap();
    assert_eq!(
        sqrt(&sn).unwrap_err().id(),
        "chimalab:runtime:classesIncompativeis"
    );
    assert_eq!(
        sqrt(&Value::scalar(4.0).cast(Class::Single).unwrap())
            .unwrap()
            .class(),
        Class::Single
    );
    assert_eq!(
        elementwise(BinOp::Add, &sn, &i).unwrap_err().id(),
        "chimalab:runtime:classesIncompativeis"
    );
}

// ---------------------------------------------------------------- F004

fn mat3x3() -> Value {
    // [1 4 7; 2 5 8; 3 6 9] column-major.
    Value::doubles(&[3, 3], (1..=9).map(f64::from).collect())
}

#[test]
fn ca_f004_atribuir_em_end_mais_1_cresce() {
    let mut v = row(&[1.0, 2.0, 3.0]);
    let end = v.end_index(0, 1);
    subsasgn(&mut v, &[Index::Scalar(end + 1)], &Value::scalar(9.0)).unwrap();
    assert_eq!(v.dims(), &[1, 4]);
    assert_eq!(f(&v), vec![1.0, 2.0, 3.0, 9.0]);
    // Coluna cresce como coluna; vazio vira linha; buraco preenchido com zero.
    let mut c = col(&[1.0, 2.0]);
    subsasgn(&mut c, &[Index::Scalar(4)], &Value::scalar(5.0)).unwrap();
    assert_eq!((c.dims().to_vec(), f(&c)), (vec![4, 1], vec![1.0, 2.0, 0.0, 5.0]));
    let mut e = Value::doubles(&[0, 0], vec![]);
    subsasgn(&mut e, &[Index::Scalar(2)], &Value::scalar(7.0)).unwrap();
    assert_eq!((e.dims().to_vec(), f(&e)), (vec![1, 2], vec![0.0, 7.0]));
    // Matriz não cresce por índice linear.
    let mut m = mat3x3();
    assert_eq!(
        subsasgn(&mut m, &[Index::Scalar(10)], &Value::scalar(1.0))
            .unwrap_err()
            .id(),
        "chimalab:runtime:indiceInvalido"
    );
    // Mas cresce por subscritos.
    subsasgn(
        &mut m,
        &[Index::Scalar(4), Index::Scalar(2)],
        &Value::scalar(-1.0),
    )
    .unwrap();
    assert_eq!(m.dims(), &[4, 3]);
    assert_eq!(
        f(&subsref(&m, &[Index::Scalar(4), Index::All]).unwrap()),
        vec![0.0, -1.0, 0.0]
    );
}

#[test]
fn ca_f004_dois_pontos_da_coluna() {
    let r = subsref(&mat3x3(), &[Index::All]).unwrap();
    assert_eq!(r.dims(), &[9, 1]);
    assert_eq!(f(&r), (1..=9).map(f64::from).collect::<Vec<_>>());
}

#[test]
fn ca_f004_remover_linha() {
    let mut m = mat3x3();
    subsdelete(&mut m, &[Index::Scalar(2), Index::All]).unwrap();
    assert_eq!(m.dims(), &[2, 3]);
    assert_eq!(f(&m), vec![1.0, 3.0, 4.0, 6.0, 7.0, 9.0]);
    // Remover colunas; remoção linear; tudo.
    let mut m = mat3x3();
    subsdelete(&mut m, &[Index::All, Index::List(vec![1, 3])]).unwrap();
    assert_eq!((m.dims().to_vec(), f(&m)), (vec![3, 1], vec![4.0, 5.0, 6.0]));
    let mut v = row(&[1.0, 2.0, 3.0, 4.0]);
    subsdelete(&mut v, &[Index::List(vec![2, 4])]).unwrap();
    assert_eq!(f(&v), vec![1.0, 3.0]);
    let mut c = col(&[1.0, 2.0, 3.0]);
    subsdelete(&mut c, &[Index::Scalar(1)]).unwrap();
    assert_eq!(c.dims(), &[2, 1]);
    let mut m = mat3x3();
    subsdelete(&mut m, &[Index::Scalar(5)]).unwrap();
    assert_eq!(m.dims(), &[1, 8]);
    let mut m = mat3x3();
    subsdelete(&mut m, &[Index::All]).unwrap();
    assert_eq!(m.numel(), 0);
    let mut m = mat3x3();
    subsdelete(&mut m, &[Index::All, Index::All]).unwrap();
    assert_eq!(m.dims(), &[0, 3]);
    // Remoção com dois subscritos parciais: erro; fora dos limites: erro.
    let mut m = mat3x3();
    assert_eq!(
        subsdelete(&mut m, &[Index::Scalar(1), Index::Scalar(1)])
            .unwrap_err()
            .id(),
        "chimalab:runtime:indiceInvalido"
    );
    assert_eq!(
        subsdelete(&mut m, &[Index::Scalar(4), Index::All])
            .unwrap_err()
            .id(),
        "chimalab:runtime:indiceForaDosLimites"
    );
    assert_eq!(
        subsdelete(&mut m, &[Index::Scalar(10)]).unwrap_err().id(),
        "chimalab:runtime:indiceForaDosLimites"
    );
}

#[test]
fn ca_f004_tamanho_de_zeros_0x3() {
    let z = Value::doubles(&[0, 3], vec![]);
    assert_eq!(z.dims(), &[0, 3]);
    assert_eq!(z.numel(), 0);
    // Formas N-D com singletons finais colapsam.
    assert_eq!(Value::doubles(&[2, 3, 1, 1], vec![0.0; 6]).dims(), &[2, 3]);
}

#[test]
fn indexacao_leitura() {
    let m = mat3x3();
    assert_eq!(
        f(&subsref(&m, &[Index::Scalar(2), Index::Scalar(3)]).unwrap()),
        vec![8.0]
    );
    assert_eq!(f(&subsref(&m, &[Index::Scalar(6)]).unwrap()), vec![6.0]);
    let r = subsref(&m, &[Index::List(vec![1, 3]), Index::All]).unwrap();
    assert_eq!(
        (r.dims().to_vec(), f(&r)),
        (vec![2, 3], vec![1.0, 3.0, 4.0, 6.0, 7.0, 9.0])
    );
    // Lista em matriz dá linha; em coluna, coluna.
    assert_eq!(subsref(&m, &[Index::List(vec![1, 2])]).unwrap().dims(), &[1, 2]);
    assert_eq!(
        subsref(&col(&[1.0, 2.0, 3.0]), &[Index::List(vec![1, 2])])
            .unwrap()
            .dims(),
        &[2, 1]
    );
    // Máscara lógica: em matriz, coluna; em linha, linha.
    let mask: Vec<bool> = (1..=9).map(|k| k % 2 == 0).collect();
    let r = subsref(&m, &[Index::Logical(mask)]).unwrap();
    assert_eq!((r.dims().to_vec(), f(&r)), (vec![4, 1], vec![2.0, 4.0, 6.0, 8.0]));
    assert_eq!(
        subsref(&row(&[1.0, 2.0, 3.0]), &[Index::Logical(vec![true, false, true])])
            .unwrap()
            .dims(),
        &[1, 2]
    );
    // Máscara mais longa com excesso falso é aceita; verdadeiro fora dá erro.
    assert!(subsref(&row(&[1.0, 2.0]), &[Index::Logical(vec![true, false, false])]).is_ok());
    assert_eq!(
        subsref(&row(&[1.0, 2.0]), &[Index::Logical(vec![true, false, true])])
            .unwrap_err()
            .id(),
        "chimalab:runtime:indiceForaDosLimites"
    );
    // end, dimensões dobradas e subscritos extras.
    assert_eq!(
        (
            m.end_index(0, 1),
            m.end_index(0, 2),
            m.end_index(1, 2),
            m.end_index(2, 3)
        ),
        (9, 3, 3, 1)
    );
    let t = Value::doubles(&[2, 2, 2], (1..=8).map(f64::from).collect());
    assert_eq!(t.end_index(1, 2), 4);
    assert_eq!(
        f(&subsref(&t, &[Index::Scalar(1), Index::Scalar(4)]).unwrap()),
        vec![7.0]
    );
    assert_eq!(
        f(&subsref(&m, &[Index::Scalar(1), Index::Scalar(1), Index::Scalar(1)]).unwrap()),
        vec![1.0]
    );
    // Erros: zero e fora dos limites.
    assert_eq!(
        subsref(&m, &[Index::Scalar(0)]).unwrap_err().id(),
        "chimalab:runtime:indiceInvalido"
    );
    assert_eq!(
        subsref(&m, &[Index::Scalar(10)]).unwrap_err().id(),
        "chimalab:runtime:indiceForaDosLimites"
    );
    assert_eq!(
        subsref(&m, &[Index::Scalar(4), Index::Scalar(1)])
            .unwrap_err()
            .id(),
        "chimalab:runtime:indiceForaDosLimites"
    );
    assert_eq!(subsref(&m, &[]).unwrap(), m);
}

#[test]
fn atribuicao_classes_e_expansao() {
    // Escalar expandido; contagem incompatível.
    let mut m = mat3x3();
    subsasgn(&mut m, &[Index::All, Index::Scalar(1)], &Value::scalar(0.0)).unwrap();
    assert_eq!(
        f(&subsref(&m, &[Index::All, Index::Scalar(1)]).unwrap()),
        vec![0.0; 3]
    );
    assert_eq!(
        subsasgn(&mut m, &[Index::All, Index::Scalar(1)], &row(&[1.0, 2.0]))
            .unwrap_err()
            .id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    // Inteiro atribuído a double vira double; double atribuído a int8 satura.
    let mut d = row(&[1.0, 2.0]);
    subsasgn(&mut d, &[Index::Scalar(1)], &int(IntClass::I8, 100.0)).unwrap();
    assert_eq!(d.class(), Class::Double);
    let mut i = int(IntClass::I8, 0.0);
    subsasgn(&mut i, &[Index::Scalar(2)], &Value::scalar(1000.0)).unwrap();
    assert_eq!((i.class(), f(&i)), (Class::Int(IntClass::I8), vec![0.0, 127.0]));
    // Complexo promove o alvo double.
    let mut c = row(&[1.0, 2.0]);
    subsasgn(&mut c, &[Index::Scalar(2)], &sqrt(&Value::scalar(-4.0)).unwrap()).unwrap();
    assert!(c.is_complex());
    // Alvo vazio adota a classe do lado direito.
    let mut e = Value::doubles(&[0, 0], vec![]);
    subsasgn(&mut e, &[Index::Scalar(1)], &Value::text("x")).unwrap();
    assert_eq!(e.class(), Class::Char);
    // Lógico, char, single e todas as classes inteiras na atribuição e na leitura.
    let mut l = Value::scalar(1.0).cast(Class::Logical).unwrap();
    subsasgn(&mut l, &[Index::Scalar(3)], &Value::scalar(1.0)).unwrap();
    assert_eq!(f(&l), vec![1.0, 0.0, 1.0]);
    let mut s = Value::scalar(1.0).cast(Class::Single).unwrap();
    subsasgn(&mut s, &[Index::Scalar(2)], &Value::scalar(2.0)).unwrap();
    assert_eq!(f(&s), vec![1.0, 2.0]);
    for c in [
        IntClass::I16,
        IntClass::I32,
        IntClass::I64,
        IntClass::U8,
        IntClass::U16,
        IntClass::U32,
        IntClass::U64,
    ] {
        let mut v = int(c, 1.0);
        subsasgn(&mut v, &[Index::Scalar(2)], &Value::scalar(2.0)).unwrap();
        assert_eq!(f(&subsref(&v, &[Index::Scalar(2)]).unwrap()), vec![2.0]);
        subsdelete(&mut v, &[Index::Scalar(1)]).unwrap();
        assert_eq!(v.numel(), 1);
    }
    // Atribuição sem índices substitui.
    let mut x = Value::scalar(1.0);
    subsasgn(&mut x, &[], &Value::text("ab")).unwrap();
    assert_eq!(x, Value::text("ab"));
    // Erros de mensagem.
    for e in [
        RuntimeError::InvalidIndex("x".into()),
        RuntimeError::IndexOutOfBounds { index: 2, bound: 1 },
        RuntimeError::InvalidConversion("y".into()),
        RuntimeError::Math("z".into()),
        RuntimeError::IncompatibleShapes {
            operation: "o",
            left: vec![1],
            right: vec![2],
        },
        RuntimeError::IncompatibleClasses {
            operation: "o",
            left: "a".into(),
            right: "b".into(),
        },
    ] {
        assert!(!e.to_string().is_empty());
    }
}

proptest! {
    /// Conversão para inteiro: arredonda o meio para longe de zero e satura, em todas as classes.
    #[test]
    fn conversao_inteira(x in -1e20f64..1e20) {
        for c in [IntClass::I8, IntClass::I16, IntClass::I32, IntClass::I64, IntClass::U8, IntClass::U16, IntClass::U32, IntClass::U64] {
            let v = f(&int(c, x))[0];
            let esperado = x.round().clamp(c.min() as f64, c.max() as f64);
            prop_assert_eq!(v, esperado);
        }
    }

    /// Aritmética de int16 contra a definição: calcula em double, arredonda, satura.
    #[test]
    fn aritmetica_int16(a in -40000i32..40000, b in -40000i32..40000, k in 0usize..4) {
        let op = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div][k];
        let (ia, ib) = (int(IntClass::I16, f64::from(a)), int(IntClass::I16, f64::from(b)));
        let (fa, fb) = (f(&ia)[0], f(&ib)[0]);
        let r = f(&elementwise(op, &ia, &ib).unwrap())[0];
        let exato = match op {
            BinOp::Add => fa + fb,
            BinOp::Sub => fa - fb,
            BinOp::Mul => fa * fb,
            _ => if fb == 0.0 { if fa > 0.0 { 1e9 } else if fa < 0.0 { -1e9 } else { 0.0 } } else { fa / fb },
        };
        prop_assert_eq!(r, exato.round().clamp(-32768.0, 32767.0));
    }

    /// Ida e volta: atribuir e ler devolve o valor, crescendo quando preciso.
    #[test]
    fn atribuir_e_ler(n in 1usize..8, i in 1usize..12, v in -100.0f64..100.0) {
        let mut x = row(&vec![0.0; n]);
        subsasgn(&mut x, &[Index::Scalar(i)], &Value::scalar(v)).unwrap();
        prop_assert_eq!(f(&subsref(&x, &[Index::Scalar(i)]).unwrap()), vec![v]);
        prop_assert_eq!(x.numel(), n.max(i));
    }
}

#[test]
fn casos_restantes() {
    // Produto matricial com operando N-D: erro de dimensões.
    let t = Value::doubles(&[2, 2, 2], vec![1.0; 8]);
    assert_eq!(
        mtimes(&t, &t).unwrap_err().id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    // Quase singular: aviso com rcond > 0.
    let q = Value::doubles(&[2, 2], vec![1.0, 1.0, 1.0, 1.0 + 2.0 * f64::EPSILON]);
    let (_, av) = mldivide(&q, &col(&[1.0, 2.0])).unwrap();
    assert!(matches!(av[0], chimalab_runtime::Warning::SingularMatrix { rcond } if rcond > 0.0));
    // Crescer com dimensões dobradas no último subscrito: erro.
    let mut t3 = Value::doubles(&[2, 2, 2], vec![0.0; 8]);
    assert_eq!(
        subsasgn(
            &mut t3,
            &[Index::Scalar(1), Index::Scalar(9)],
            &Value::scalar(1.0)
        )
        .unwrap_err()
        .id(),
        "chimalab:runtime:indiceInvalido"
    );
    // NaN para caractere: código 0.
    assert_eq!(
        Value::scalar(f64::NAN).cast(Class::Char).unwrap(),
        Value::Char(NdArray::filled_nd(&[1, 1], '\0'))
    );
    // Inteiro dividido por real não inteiro: em double, depois arredonda.
    assert_eq!(
        f(&elementwise(BinOp::Div, &int(IntClass::I32, 7.0), &Value::scalar(2.5)).unwrap()),
        vec![3.0]
    );
    assert_eq!(
        RuntimeError::Math("x".into()).id(),
        "chimalab:runtime:erroNumerico"
    );
}

#[test]
fn logicos_transposta_e_intervalos() {
    use chimalab_runtime::{LogicOp, colon, logical, not, transpose};
    let a = row(&[0.0, 1.0, 2.0]);
    let b = col(&[1.0, 0.0]);
    let r = logical(LogicOp::And, &a, &b).unwrap();
    assert_eq!((r.class(), r.dims().to_vec()), (Class::Logical, vec![2, 3]));
    assert_eq!(f(&r), vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    assert_eq!(
        f(&logical(LogicOp::Or, &a, &Value::scalar(0.0)).unwrap()),
        vec![0.0, 1.0, 1.0]
    );
    assert_eq!(
        f(&logical(LogicOp::Xor, &a, &Value::scalar(1.0)).unwrap()),
        vec![1.0, 0.0, 0.0]
    );
    assert_eq!(f(&not(&a).unwrap()), vec![1.0, 0.0, 0.0]);
    let z = sqrt(&Value::scalar(-1.0)).unwrap();
    assert_eq!(f(&not(&z).unwrap()), vec![0.0]);
    assert_eq!(
        not(&Value::scalar(f64::NAN)).unwrap_err().id(),
        "chimalab:runtime:conversaoInvalida"
    );
    let zn = elementwise(BinOp::Mul, &z, &Value::scalar(f64::NAN)).unwrap();
    assert_eq!(not(&zn).unwrap_err().id(), "chimalab:runtime:conversaoInvalida");
    assert_eq!(
        logical(LogicOp::And, &row(&[1.0, 2.0]), &row(&[1.0, 2.0, 3.0]))
            .unwrap_err()
            .id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    let l = Value::scalar(1.0).cast(Class::Logical).unwrap();
    assert_eq!(f(&not(&l).unwrap()), vec![0.0]);
    // Transposta: 2×3 → 3×2; conjugada nos complexos; todas as classes.
    let m = Value::doubles(&[2, 3], (1..=6).map(f64::from).collect());
    let t = transpose(&m, false).unwrap();
    assert_eq!(
        (t.dims().to_vec(), f(&t)),
        (vec![3, 2], vec![1.0, 3.0, 5.0, 2.0, 4.0, 6.0])
    );
    assert_eq!(transpose(&t, false).unwrap(), m);
    let zc = elementwise(BinOp::Add, &z, &row(&[1.0, 2.0])).unwrap();
    let zt = transpose(&zc, true).unwrap();
    assert_eq!(zt.dims(), &[2, 1]);
    assert_eq!(zt.to_complex()[0].im, -1.0);
    assert_eq!(transpose(&zc, false).unwrap().to_complex()[0].im, 1.0);
    for v in [
        m.cast(Class::Single).unwrap(),
        m.cast(Class::Logical).unwrap(),
        Value::text("abc"),
        m.cast(Class::Int(IntClass::U16)).unwrap(),
    ] {
        let t = transpose(&v, false).unwrap();
        assert_eq!((t.class(), t.dims()[0]), (v.class(), v.dims()[1]));
    }
    assert_eq!(
        transpose(&Value::doubles(&[1, 1, 2], vec![1.0, 2.0]), false)
            .unwrap_err()
            .id(),
        "chimalab:runtime:dimensoesIncompativeis"
    );
    // Intervalos: 0:0,1:1 tem 11 elementos e termina exatamente em 1.
    let r = colon(0.0, 0.1, 1.0);
    assert_eq!(r.dims(), &[1, 11]);
    assert_eq!(*f(&r).last().unwrap(), 1.0);
    assert_eq!(f(&colon(1.0, 1.0, 5.0)), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(f(&colon(5.0, -2.0, 0.0)), vec![5.0, 3.0, 1.0]);
    assert_eq!(f(&colon(1.0, 2.0, 6.0)), vec![1.0, 3.0, 5.0]);
    for vazio in [
        colon(5.0, 1.0, 1.0),
        colon(0.0, 0.0, 1.0),
        colon(0.0, 1.0, f64::NAN),
        colon(1.0, -1.0, 2.0),
    ] {
        assert_eq!(vazio.dims(), &[1, 0]);
    }
    // Muitos passos decimais: contagem certa sem tolerância exagerada.
    assert_eq!(colon(0.0, 0.01, 1.0).numel(), 101);
    assert_eq!(colon(0.0, 0.3, 0.9).numel(), 4);
    assert_eq!(colon(0.0, 0.3, 0.89).numel(), 3);
}
