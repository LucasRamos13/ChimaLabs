//! Álgebra linear densa na linguagem (F040 pelo interpretador): `det inv norm rank cond trace
//! eig svd chol lu qr expm pinv null orth`, para matrizes reais e complexas.
//!
//! As contas são as do `chimalab-math` (`linalg`); aqui só se convertem valores, escolhem-se as
//! saídas pelo número pedido e se traduzem erros (identificadores `chimalab:linalg:…`) e avisos.

use chimalab_array::{Complex64, NdArray};
use chimalab_lang::sintaxe::Node;
use chimalab_math::linalg::{self, Eig, LinScalar, LinalgError, NormKind};

use crate::Value;
use crate::avaliador::{ErroExec, Interpretador, Obj, erro_em};
use crate::ops::{converte_avisos, simplify};

/// Nomes atendidos por este módulo.
pub const NOMES: &[&str] = &[
    "det", "inv", "norm", "rank", "cond", "trace", "eig", "svd", "chol", "lu", "qr", "expm", "pinv", "null",
    "orth",
];

/// Elemento de matriz da linguagem: `double` real ou complexo.
trait Elem: LinScalar {
    fn valor(a: NdArray<Self>) -> Value;
    fn escalar(x: Self) -> Value;
}

impl Elem for f64 {
    fn valor(a: NdArray<f64>) -> Value {
        Value::Double(a)
    }
    fn escalar(x: f64) -> Value {
        Value::scalar(x)
    }
}

impl Elem for Complex64 {
    fn valor(a: NdArray<Complex64>) -> Value {
        simplify(a)
    }
    fn escalar(x: Complex64) -> Value {
        simplify(NdArray::filled_nd(&[1, 1], x))
    }
}

fn de_linalg(e: LinalgError, n: &Node) -> ErroExec {
    erro_em(e.id(), e.to_string(), n)
}

fn coluna(v: Vec<f64>) -> Value {
    let k = v.len();
    Value::doubles(&[k, 1], v)
}

fn coluna_complexa(v: Vec<Complex64>) -> Value {
    let k = v.len();
    simplify(NdArray::from_shape_vec(&[k, 1], v).expect("forma"))
}

/// Matriz diagonal r×c com `d` (de `min(r, c)` elementos) na diagonal.
fn diagonal<T: Elem>(d: &[T], r: usize, c: usize) -> NdArray<T> {
    debug_assert_eq!(d.len(), r.min(c));
    NdArray::from_fn(r, c, |i, j| if i == j { d[i] } else { T::zero() })
}

fn um(v: Value) -> Result<Vec<Obj>, ErroExec> {
    Ok(vec![Obj::Array(v)])
}

/// Tolerância opcional (segundo argumento de `rank` e `pinv`).
fn tolerancia(extra: Option<&Obj>, nome: &str, n: &Node) -> Result<Option<f64>, ErroExec> {
    let Some(o) = extra else { return Ok(None) };
    let x = match o {
        Obj::Array(v) => v.to_f64s().ok().filter(|x| x.len() == 1).map(|x| x[0]),
        _ => None,
    };
    match x {
        Some(t) if t >= 0.0 => Ok(Some(t)),
        _ => Err(erro_em(
            "chimalab:execucao:argumentos",
            format!("a tolerância de `{nome}` precisa ser um escalar ≥ 0"),
            n,
        )),
    }
}

/// Tipo de norma: número (1, 2, Inf ou p) ou o texto "fro".
fn tipo_de_norma(extra: Option<&Obj>, n: &Node) -> Result<NormKind, ErroExec> {
    let erro = || {
        erro_em(
            "chimalab:execucao:argumentos",
            "o tipo de `norm` é um número p > 0, Inf ou \"fro\"",
            n,
        )
    };
    match extra {
        None => Ok(NormKind::Two),
        Some(Obj::Array(Value::Char(a))) => {
            let s: String = a.as_slice().iter().collect();
            if s == "fro" {
                Ok(NormKind::Fro)
            } else {
                Err(erro())
            }
        }
        Some(Obj::Array(v)) => {
            let x = v.to_f64s().map_err(|_| erro())?;
            match x.as_slice() {
                [p] if *p == 1.0 => Ok(NormKind::One),
                [p] if *p == 2.0 => Ok(NormKind::Two),
                [p] if *p == f64::INFINITY => Ok(NormKind::Inf),
                [p] if *p > 0.0 => Ok(NormKind::P(*p)),
                _ => Err(erro()),
            }
        }
        Some(_) => Err(erro()),
    }
}

/// `true` se `nome` é deste módulo.
pub fn atende(nome: &str) -> bool {
    NOMES.contains(&nome)
}

/// Chama a função `nome` (já conferida com [`atende`]).
pub fn chama(
    it: &mut Interpretador,
    nome: &'static str,
    pos: &[Obj],
    nargout: usize,
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let max = if matches!(nome, "norm" | "rank" | "pinv") {
        2
    } else {
        1
    };
    if pos.is_empty() || pos.len() > max {
        let faixa = if max == 1 {
            "1".to_string()
        } else {
            "1 ou 2".to_string()
        };
        return Err(erro_em(
            "chimalab:execucao:argumentos",
            format!("`{nome}` aceita {faixa} argumento(s), recebeu {}", pos.len()),
            n,
        ));
    }
    let extra = pos.get(1);
    let Obj::Array(v) = &pos[0] else {
        return Err(erro_em(
            "chimalab:execucao:funcaoComoValor",
            "esperava uma matriz, veio uma função",
            n,
        ));
    };
    if v.dims().len() > 2 {
        return Err(erro_em(
            "chimalab:execucao:argumentos",
            format!("`{nome}` exige uma matriz 2-D"),
            n,
        ));
    }
    match v {
        Value::Complex(a) => com(it, nome, a.clone(), extra, nargout, n),
        Value::Double(a) => com(it, nome, a.clone(), extra, nargout, n),
        Value::Logical(_) | Value::Char(_) => {
            let a = NdArray::from_shape_vec(v.dims(), v.to_f64s().expect("lógico e caractere são reais"))
                .expect("forma");
            com(it, nome, a, extra, nargout, n)
        }
        outro => Err(erro_em(
            "chimalab:execucao:classeNaoSuportada",
            format!(
                "`{nome}` ainda não aceita {}; converta com double(…)",
                outro.class().name()
            ),
            n,
        )),
    }
}

fn com<T: Elem>(
    it: &mut Interpretador,
    nome: &str,
    a: NdArray<T>,
    extra: Option<&Obj>,
    nargout: usize,
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let e = |err| de_linalg(err, n);
    match nome {
        "det" => um(T::escalar(linalg::det(&a).map_err(e)?)),
        "inv" => {
            let (m, w) = linalg::inv(&a).map_err(e)?;
            it.avisos(converte_avisos(&w));
            um(T::valor(m))
        }
        "norm" => um(Value::scalar(
            linalg::norm(&a, tipo_de_norma(extra, n)?).map_err(e)?,
        )),
        "rank" => {
            let tol = tolerancia(extra, nome, n)?;
            um(Value::scalar(linalg::rank(&a, tol).map_err(e)? as f64))
        }
        "cond" => um(Value::scalar(linalg::cond(&a).map_err(e)?)),
        "trace" => {
            if !a.is_square() {
                return Err(erro_em(
                    "chimalab:linalg:matrizNaoQuadrada",
                    format!("trace exige matriz quadrada (veio {}×{})", a.rows(), a.cols()),
                    n,
                ));
            }
            um(T::escalar(a.trace()))
        }
        "eig" => {
            let r = linalg::eig(&a).map_err(e)?;
            let k = a.rows();
            match (r, nargout >= 2) {
                (Eig::Symmetric { values, .. }, false) => um(coluna(values)),
                (Eig::General { values, .. }, false) => um(coluna_complexa(values)),
                (Eig::Symmetric { values, vectors }, true) => {
                    let d: Vec<f64> = values;
                    Ok(vec![
                        Obj::Array(T::valor(vectors)),
                        Obj::Array(Value::Double(diagonal(&d, k, k))),
                    ])
                }
                (Eig::General { values, vectors }, true) => Ok(vec![
                    Obj::Array(simplify(vectors)),
                    Obj::Array(simplify(diagonal(&values, k, k))),
                ]),
            }
        }
        "svd" => {
            if nargout <= 1 {
                return um(coluna(linalg::singular_values(&a).map_err(e)?));
            }
            let s = linalg::svd(&a, false).map_err(e)?;
            Ok(vec![
                Obj::Array(T::valor(s.u)),
                Obj::Array(Value::Double(diagonal(&s.s, a.rows(), a.cols()))),
                Obj::Array(T::valor(s.v)),
            ])
        }
        "chol" => um(T::valor(linalg::chol(&a).map_err(e)?.r)),
        "lu" => {
            if nargout < 2 {
                return Err(erro_em(
                    "chimalab:execucao:saidasDemais",
                    "lu devolve os fatores: L, U = lu(A) ou L, U, P = lu(A)",
                    n,
                ));
            }
            let f = linalg::lu(&a).map_err(e)?;
            let m = a.rows();
            // P·A = L·U, com a linha i de P·A igual à linha perm[i] de A.
            let p = NdArray::from_fn(m, m, |i, j| if f.perm[i] == j { 1.0 } else { 0.0 });
            if nargout == 2 {
                // Pᵀ·L: L com as linhas devolvidas aos lugares de A.
                let k = f.l.cols();
                let mut pl = NdArray::<T>::zeros(m, k);
                for i in 0..m {
                    for j in 0..k {
                        pl.set(f.perm[i], j, f.l.get(i, j));
                    }
                }
                return Ok(vec![Obj::Array(T::valor(pl)), Obj::Array(T::valor(f.u))]);
            }
            Ok(vec![
                Obj::Array(T::valor(f.l)),
                Obj::Array(T::valor(f.u)),
                Obj::Array(Value::Double(p)),
            ])
        }
        "qr" => {
            let f = linalg::qr(&a).map_err(e)?;
            if nargout <= 1 {
                return um(T::valor(f.r));
            }
            Ok(vec![Obj::Array(T::valor(f.q)), Obj::Array(T::valor(f.r))])
        }
        "expm" => um(T::valor(linalg::expm(&a).map_err(e)?)),
        "pinv" => {
            let tol = tolerancia(extra, nome, n)?;
            um(T::valor(linalg::pinv(&a, tol).map_err(e)?))
        }
        "null" => um(T::valor(linalg::null_space(&a).map_err(e)?)),
        "orth" => um(T::valor(linalg::orth(&a).map_err(e)?)),
        _ => unreachable!("nome conferido em `atende`"),
    }
}
