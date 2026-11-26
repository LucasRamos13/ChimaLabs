//! Funções embutidas do primeiro interpretador (DT-40): construção, tamanho, reduções,
//! funções elementares, conversões de classe, `range`, `error`, `disp` e `isequal`.
//!
//! As reduções sem dimensão usam a primeira dimensão diferente de 1; as de um array 0×0 dão o
//! elemento neutro (`sum([])` é 0, `all([])` é verdadeiro).

use chimalab_array::{Complex64, NdArray, broadcast_shape};
use chimalab_lang::sintaxe::Node;
use chimalab_math::stats;

use crate::avaliador::{ErroExec, Interpretador, Obj, Saida, de_runtime, erro_em, iguais};
use crate::ops::broadcast_index;
use crate::{
    Class, Index, IntClass, IntData, Value, exibicao, linspace, sqrt, subsasgn, subsref, verifica_tamanho,
};

const NOMES: &[&str] = &[
    "zeros",
    "ones",
    "eye",
    "numel",
    "size",
    "length",
    "ndims",
    "isempty",
    "sum",
    "mean",
    "any",
    "all",
    "max",
    "min",
    "abs",
    "sqrt",
    "exp",
    "log",
    "sin",
    "cos",
    "tan",
    "floor",
    "ceil",
    "round",
    "sign",
    "mod",
    "range",
    "double",
    "single",
    "logical",
    "char",
    "int8",
    "int16",
    "int32",
    "int64",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "error",
    "disp",
    "isequal",
    "prod",
    "cumsum",
    "cumprod",
    "sort",
    "find",
    "reshape",
    "diag",
    "fix",
    "log10",
    "log2",
    "sinh",
    "cosh",
    "tanh",
    "asin",
    "acos",
    "atan",
    "isnan",
    "isinf",
    "isfinite",
    "isreal",
    "real",
    "imag",
    "conj",
    "angle",
    "atan2",
    "hypot",
    "rem",
    "class",
    "help",
    "unique",
    "ismember",
    "union",
    "intersect",
    "setdiff",
    "who",
    "whos",
    "clear",
    "median",
    "mode",
    "var",
    "std",
    "quantile",
    "dot",
    "cross",
    "kron",
    "flip",
    "diff",
];

/// Constantes por nome.
pub fn constante(nome: &str) -> Option<Value> {
    let l = |b: bool| Value::Logical(NdArray::filled_nd(&[1, 1], b));
    Some(match nome {
        "pi" => Value::scalar(std::f64::consts::PI),
        "Inf" | "inf" => Value::scalar(f64::INFINITY),
        "NaN" | "nan" => Value::scalar(f64::NAN),
        "eps" => Value::scalar(f64::EPSILON),
        "true" => l(true),
        "false" => l(false),
        _ => return None,
    })
}

/// Todos os nomes de funções embutidas (para a ajuda e os testes de conformidade).
pub fn todas() -> Vec<&'static str> {
    NOMES
        .iter()
        .chain(crate::algebra::NOMES)
        .chain(crate::funcionais::NOMES)
        .chain(crate::numericos::NOMES)
        .chain(crate::especiais::NOMES)
        .chain(crate::aleatorios::NOMES)
        .copied()
        .collect()
}

/// O nome, se for uma função embutida.
pub fn nome(nome: &str) -> Option<&'static str> {
    NOMES
        .iter()
        .chain(crate::algebra::NOMES)
        .chain(crate::funcionais::NOMES)
        .chain(crate::numericos::NOMES)
        .chain(crate::especiais::NOMES)
        .chain(crate::aleatorios::NOMES)
        .copied()
        .find(|&n| n == nome)
}

fn err(id: &str, m: impl Into<String>, n: &Node) -> ErroExec {
    erro_em(id, m, n)
}

fn arr<'a>(o: &'a Obj, n: &Node) -> Result<&'a Value, ErroExec> {
    match o {
        Obj::Array(v) => Ok(v),
        _ => Err(err(
            "chimalab:execucao:funcaoComoValor",
            "esperava um array, veio uma função",
            n,
        )),
    }
}

fn conta(nome: &str, pos: &[Obj], min: usize, max: usize, n: &Node) -> Result<(), ErroExec> {
    if pos.len() < min || pos.len() > max {
        let faixa = if min == max {
            format!("{min}")
        } else if max == usize::MAX {
            format!("pelo menos {min}")
        } else {
            format!("de {min} a {max}")
        };
        return Err(err(
            "chimalab:execucao:argumentos",
            format!("`{nome}` aceita {faixa} argumento(s), recebeu {}", pos.len()),
            n,
        ));
    }
    Ok(())
}

fn escalar_f64(o: &Obj, papel: &str, n: &Node) -> Result<f64, ErroExec> {
    let v = arr(o, n)?;
    let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
    if x.len() != 1 {
        return Err(err(
            "chimalab:execucao:argumentos",
            format!(
                "{papel} precisa ser um escalar (veio {})",
                exibicao::tamanho(v.dims())
            ),
            n,
        ));
    }
    Ok(x[0])
}

fn inteiro(o: &Obj, papel: &str, n: &Node) -> Result<usize, ErroExec> {
    let x = escalar_f64(o, papel, n)?;
    if !(x.is_finite() && x.fract() == 0.0 && x >= 0.0) {
        return Err(err(
            "chimalab:execucao:argumentos",
            format!(
                "{papel} precisa ser um inteiro ≥ 0 (veio {})",
                exibicao::numero(x)
            ),
            n,
        ));
    }
    Ok(x as usize)
}

fn logico(b: bool) -> Obj {
    Obj::Array(Value::Logical(NdArray::filled_nd(&[1, 1], b)))
}

fn dobro(x: f64) -> Obj {
    Obj::Array(Value::scalar(x))
}

/// Dimensões a partir dos argumentos de `zeros`/`ones`: nenhum (1×1), um (n×n) ou vários.
fn forma(nome: &str, pos: &[Obj], n: &Node) -> Result<Vec<usize>, ErroExec> {
    let d = pos
        .iter()
        .map(|o| inteiro(o, &format!("cada tamanho de `{nome}`"), n))
        .collect::<Result<Vec<_>, _>>()?;
    // Sem tamanhos, a forma vazia normaliza para 1×1.
    let d = match d.len() {
        1 => vec![d[0], d[0]],
        _ => d,
    };
    verifica_tamanho(&d).map_err(|e| de_runtime(e, n))?;
    Ok(d)
}

fn dim_padrao(dims: &[usize]) -> usize {
    dims.iter().position(|&d| d != 1).unwrap_or(0)
}

/// Aplica `f` a cada fibra ao longo da dimensão `d` (0-based); devolve os resultados e a forma.
fn reduz<T: Copy, U>(data: &[T], dims: &[usize], d: usize, f: impl Fn(&[T]) -> U) -> (Vec<U>, Vec<usize>) {
    let nd = dims.get(d).copied().unwrap_or(1);
    let passo: usize = dims[..d.min(dims.len())].iter().product();
    let fora: usize = dims.get(d + 1..).map_or(1, |r| r.iter().product());
    let mut out_dims = dims.to_vec();
    if d < out_dims.len() {
        out_dims[d] = 1;
    }
    let mut out = Vec::with_capacity(passo * fora);
    let mut fibra = Vec::with_capacity(nd);
    for o in 0..fora {
        for i in 0..passo {
            fibra.clear();
            fibra.extend((0..nd).map(|k| data[i + passo * (k + nd * o)]));
            out.push(f(&fibra));
        }
    }
    (out, out_dims)
}

/// Dimensão da redução: o segundo argumento (base 1) ou a padrão.
fn dim_arg(nome: &str, v: &Value, pos: &[Obj], n: &Node) -> Result<usize, ErroExec> {
    match pos.get(1) {
        None => Ok(dim_padrao(v.dims())),
        Some(o) => {
            let d = inteiro(o, &format!("a dimensão de `{nome}`"), n)?;
            if d == 0 {
                return Err(err("chimalab:execucao:argumentos", "dimensões começam em 1", n));
            }
            Ok(d - 1)
        }
    }
}

/// Soma compensada de Neumaier.
fn soma(x: &[f64]) -> f64 {
    let (mut s, mut c) = (0.0f64, 0.0f64);
    for &v in x {
        let t = s + v;
        c += if s.abs() >= v.abs() {
            (s - t) + v
        } else {
            (v - t) + s
        };
        s = t;
    }
    s + c
}

fn com_forma(dims: &[usize], v: Vec<f64>) -> Value {
    Value::doubles(dims, v)
}

fn soma_valor(v: &Value, d: usize, n: &Node) -> Result<Value, ErroExec> {
    if v.dims() == [0, 0] {
        return Ok(match v {
            Value::Int(i) => Value::Int(IntData::from_i128(i.class(), &[1, 1], vec![0])),
            _ => Value::scalar(0.0),
        });
    }
    Ok(match v {
        Value::Complex(a) => {
            let re: Vec<f64> = a.as_slice().iter().map(|z| z.re).collect();
            let im: Vec<f64> = a.as_slice().iter().map(|z| z.im).collect();
            let (r, dims) = reduz(&re, a.dims(), d, soma);
            let (i, _) = reduz(&im, a.dims(), d, soma);
            let z = r.into_iter().zip(i).map(|(a, b)| Complex64::new(a, b)).collect();
            crate::ops::simplify(NdArray::from_shape_vec(&dims, z).expect("forma"))
        }
        Value::Int(i) => {
            let x = i.to_i128();
            let c = i.class();
            let (s, dims) = reduz(&x, i.dims(), d, |f| c.saturate(f.iter().sum::<i128>()));
            Value::Int(IntData::from_i128(c, &dims, s))
        }
        outro => {
            let x = outro.to_f64s().map_err(|e| de_runtime(e, n))?;
            let (s, dims) = reduz(&x, outro.dims(), d, soma);
            let r = com_forma(&dims, s);
            if matches!(outro, Value::Single(_)) {
                r.cast(Class::Single).map_err(|e| de_runtime(e, n))?
            } else {
                r
            }
        }
    })
}

/// `max`/`min` de um argumento: valores e índices (base 1), ignorando NaN.
fn extremo(v: &Value, d: usize, maior: bool, n: &Node) -> Result<(Value, Value), ErroExec> {
    if v.is_complex() {
        return Err(err(
            "chimalab:execucao:classeNaoSuportada",
            "max/min de complexos ainda não é suportado",
            n,
        ));
    }
    if v.numel() == 0 {
        let vazio = Value::doubles(&[0, 0], Vec::new());
        return Ok((vazio.clone(), vazio));
    }
    let melhor = |a: f64, b: f64| if maior { a > b } else { a < b };
    let escolhe = |f: &[f64]| -> (f64, f64) {
        let mut k = None;
        for (i, &x) in f.iter().enumerate() {
            if x.is_nan() {
                continue;
            }
            if k.is_none_or(|j: usize| melhor(x, f[j])) {
                k = Some(i);
            }
        }
        match k {
            Some(j) => (f[j], (j + 1) as f64),
            None => (f64::NAN, 1.0),
        }
    };
    if let Value::Int(i) = v {
        let x = i.to_i128();
        let (r, dims) = reduz(&x, i.dims(), d, |f| {
            let mut j = 0;
            for (k, &e) in f.iter().enumerate() {
                if (maior && e > f[j]) || (!maior && e < f[j]) {
                    j = k;
                }
            }
            (f[j], (j + 1) as f64)
        });
        let (val, idx): (Vec<i128>, Vec<f64>) = r.into_iter().unzip();
        return Ok((
            Value::Int(IntData::from_i128(i.class(), &dims, val)),
            com_forma(&dims, idx),
        ));
    }
    let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
    let (r, dims) = reduz(&x, v.dims(), d, escolhe);
    let (val, idx): (Vec<f64>, Vec<f64>) = r.into_iter().unzip();
    let val = com_forma(&dims, val);
    let val = if matches!(v, Value::Single(_)) {
        val.cast(Class::Single).map_err(|e| de_runtime(e, n))?
    } else {
        val
    };
    Ok((val, com_forma(&dims, idx)))
}

/// Operação binária real com expansão implícita (`mod`, `max(a, b)`, `min(a, b)`).
fn binaria_real(
    a: &Value,
    b: &Value,
    nome: &str,
    f: impl Fn(f64, f64) -> f64,
    n: &Node,
) -> Result<Value, ErroExec> {
    let real = |v: &Value| matches!(v, Value::Double(_) | Value::Logical(_) | Value::Char(_));
    if !real(a) || !real(b) {
        return Err(err(
            "chimalab:execucao:classeNaoSuportada",
            format!("`{nome}` ainda só aceita double, lógico e caractere"),
            n,
        ));
    }
    let out = broadcast_shape(a.dims(), b.dims()).map_err(|_| {
        err(
            "chimalab:runtime:dimensoesIncompativeis",
            format!(
                "{nome}: dimensões incompatíveis {} e {}",
                exibicao::tamanho(a.dims()),
                exibicao::tamanho(b.dims())
            ),
            n,
        )
    })?;
    let (x, y) = (
        a.to_f64s().map_err(|e| de_runtime(e, n))?,
        b.to_f64s().map_err(|e| de_runtime(e, n))?,
    );
    let (ia, ib) = (broadcast_index(a.dims(), &out), broadcast_index(b.dims(), &out));
    let v = ia.iter().zip(&ib).map(|(&i, &j)| f(x[i], y[j])).collect();
    Ok(Value::doubles(&out, v))
}

/// Resto com o sinal do divisor; `mod(a, 0)` é `a`.
fn resto(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return a;
    }
    if b.is_infinite() && a.is_finite() {
        return if a == 0.0 || (a < 0.0) == (b < 0.0) { a } else { b };
    }
    let r = a % b;
    if r != 0.0 && (r < 0.0) != (b < 0.0) {
        r + b
    } else {
        r
    }
}

/// Funções elementares elemento a elemento.
fn elementar(nome: &str, v: &Value, n: &Node) -> Result<Value, ErroExec> {
    if nome == "sqrt" {
        return sqrt(v).map_err(|e| de_runtime(e, n));
    }
    let real: fn(f64) -> f64 = match nome {
        "abs" => f64::abs,
        "exp" => f64::exp,
        "log" => f64::ln,
        "sin" => f64::sin,
        "cos" => f64::cos,
        "tan" => f64::tan,
        "floor" => f64::floor,
        "ceil" => f64::ceil,
        "round" => f64::round,
        "fix" => f64::trunc,
        "log10" => f64::log10,
        "log2" => f64::log2,
        "sinh" => f64::sinh,
        "cosh" => f64::cosh,
        "tanh" => f64::tanh,
        "asin" => f64::asin,
        "acos" => f64::acos,
        "atan" => f64::atan,
        "sign" => |x| {
            if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                x
            }
        },
        _ => unreachable!("nome conferido"),
    };
    match v {
        Value::Complex(a) => {
            let z = a.as_slice();
            if nome == "abs" {
                return Ok(Value::doubles(a.dims(), z.iter().map(|z| z.norm()).collect()));
            }
            let f = |z: &Complex64| -> Complex64 {
                match nome {
                    "exp" => z.exp(),
                    "log" => z.ln(),
                    "sin" => z.sin(),
                    "cos" => z.cos(),
                    "tan" => z.tan(),
                    "log10" => z.ln() / std::f64::consts::LN_10,
                    "log2" => z.ln() / std::f64::consts::LN_2,
                    "sinh" => z.sinh(),
                    "cosh" => z.cosh(),
                    "tanh" => z.tanh(),
                    "asin" => z.asin(),
                    "acos" => z.acos(),
                    "atan" => z.atan(),
                    "sign" => {
                        let m = z.norm();
                        if m == 0.0 { *z } else { z / m }
                    }
                    _ => Complex64::new(real(z.re), real(z.im)),
                }
            };
            Ok(Value::Complex(
                NdArray::from_shape_vec(a.dims(), z.iter().map(f).collect()).expect("forma"),
            ))
        }
        Value::Int(i) => {
            if !matches!(nome, "abs" | "sign" | "floor" | "ceil" | "round" | "fix") {
                return Err(err(
                    "chimalab:execucao:classeNaoSuportada",
                    format!(
                        "`{nome}` não se aplica a {}; converta com double(…)",
                        i.class().name()
                    ),
                    n,
                ));
            }
            let c = i.class();
            let x: Vec<i128> = i
                .to_i128()
                .into_iter()
                .map(|e| match nome {
                    "abs" => c.saturate(e.abs()),
                    "sign" => e.signum(),
                    _ => e,
                })
                .collect();
            Ok(Value::Int(IntData::from_i128(c, i.dims(), x)))
        }
        outro => {
            let x = outro.to_f64s().map_err(|e| de_runtime(e, n))?;
            // Logaritmo de negativo: resultado complexo.
            // Fora do domínio real (log de negativo, asin/acos fora de [−1, 1]): complexo, no
            // ramo principal.
            let fora = match nome {
                "log" | "log10" | "log2" => x.iter().any(|&e| e < 0.0),
                "asin" | "acos" => x.iter().any(|&e| e.abs() > 1.0),
                _ => false,
            };
            if fora {
                let f = |e: f64| -> Complex64 {
                    let z = Complex64::new(e, 0.0);
                    match nome {
                        "log" => z.ln(),
                        "log10" => z.ln() / std::f64::consts::LN_10,
                        "log2" => z.ln() / std::f64::consts::LN_2,
                        "asin" => z.asin(),
                        _ => z.acos(),
                    }
                };
                let z = x.iter().map(|&e| f(e)).collect();
                return Ok(Value::Complex(
                    NdArray::from_shape_vec(outro.dims(), z).expect("forma"),
                ));
            }
            let r = Value::doubles(outro.dims(), x.into_iter().map(real).collect());
            if matches!(outro, Value::Single(_)) {
                r.cast(Class::Single).map_err(|e| de_runtime(e, n))
            } else {
                Ok(r)
            }
        }
    }
}

fn classe(nome: &str) -> Option<Class> {
    Some(match nome {
        "double" => Class::Double,
        "single" => Class::Single,
        "logical" => Class::Logical,
        "char" => Class::Char,
        "int8" => Class::Int(IntClass::I8),
        "int16" => Class::Int(IntClass::I16),
        "int32" => Class::Int(IntClass::I32),
        "int64" => Class::Int(IntClass::I64),
        "uint8" => Class::Int(IntClass::U8),
        "uint16" => Class::Int(IntClass::U16),
        "uint32" => Class::Int(IntClass::U32),
        "uint64" => Class::Int(IntClass::U64),
        _ => return None,
    })
}

fn texto(o: &Obj, papel: &str, n: &Node) -> Result<String, ErroExec> {
    match o {
        Obj::Array(Value::Char(a)) => Ok(a.as_slice().iter().collect()),
        _ => Err(err(
            "chimalab:execucao:argumentos",
            format!("{papel} precisa ser um texto"),
            n,
        )),
    }
}

/// Arredonda para `c` casas decimais (meio para longe do zero).
fn stats_round(x: f64, c: i32) -> f64 {
    chimalab_math::special::round_digits(x, c)
}

/// Bytes ocupados pelos elementos nesta implementação (`char` guarda um caractere Unicode em
/// 4 bytes; lógico, 1).
fn bytes(v: &Value) -> usize {
    let por = match v {
        Value::Double(_) => 8,
        Value::Complex(_) => 16,
        Value::Single(_) => 4,
        Value::Logical(_) => 1,
        Value::Char(_) => 4,
        Value::Int(d) => match d.class() {
            IntClass::I8 | IntClass::U8 => 1,
            IntClass::I16 | IntClass::U16 => 2,
            IntClass::I32 | IntClass::U32 => 4,
            IntClass::I64 | IntClass::U64 => 8,
        },
    };
    por * v.numel()
}

/// Mesmo conteúdo (ordem de coluna) com outra forma.
fn remodela(v: Value, dims: &[usize]) -> Value {
    match v {
        Value::Double(a) => Value::Double(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Complex(a) => Value::Complex(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Single(a) => Value::Single(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Logical(a) => Value::Logical(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Char(a) => Value::Char(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Int(d) => Value::Int(IntData::from_i128(d.class(), dims, d.to_i128())),
    }
}

/// Elementos nas posições lineares `pos` (base 0), com a forma `dims`; mantém a classe.
fn seleciona(v: &Value, pos: Vec<usize>, dims: &[usize], n: &Node) -> Result<Value, ErroExec> {
    if pos.is_empty() {
        return Ok(remodela(
            subsref(v, &[Index::List(Vec::new())]).map_err(|e| de_runtime(e, n))?,
            dims,
        ));
    }
    let r =
        subsref(v, &[Index::List(pos.into_iter().map(|p| p + 1).collect())]).map_err(|e| de_runtime(e, n))?;
    Ok(remodela(r, dims))
}

/// Produto ao longo de `d`, com a classe preservada (inteiros saturam a cada passo).
fn produto_valor(v: &Value, d: usize, n: &Node) -> Result<Value, ErroExec> {
    if v.dims() == [0, 0] {
        return Ok(match v {
            Value::Int(i) => Value::Int(IntData::from_i128(i.class(), &[1, 1], vec![1])),
            _ => Value::scalar(1.0),
        });
    }
    Ok(match v {
        Value::Complex(a) => {
            let (r, dims) = reduz(a.as_slice(), a.dims(), d, |f| f.iter().product::<Complex64>());
            crate::ops::simplify(NdArray::from_shape_vec(&dims, r).expect("forma"))
        }
        Value::Int(i) => {
            let c = i.class();
            let x = i.to_i128();
            let (r, dims) = reduz(&x, i.dims(), d, |f| {
                f.iter().fold(1i128, |p, &e| c.saturate(p.saturating_mul(e)))
            });
            Value::Int(IntData::from_i128(c, &dims, r))
        }
        outro => {
            let x = outro.to_f64s().map_err(|e| de_runtime(e, n))?;
            let (r, dims) = reduz(&x, outro.dims(), d, |f| f.iter().product());
            let r = Value::doubles(&dims, r);
            if matches!(outro, Value::Single(_)) {
                r.cast(Class::Single).map_err(|e| de_runtime(e, n))?
            } else {
                r
            }
        }
    })
}

/// Acumula ao longo de `d` (mesma forma): soma compensada ou produto.
fn acumula(v: &Value, d: usize, soma_: bool, n: &Node) -> Result<Value, ErroExec> {
    let dims = v.dims().to_vec();
    let nd = dims.get(d).copied().unwrap_or(1);
    let passo: usize = dims[..d.min(dims.len())].iter().product();
    let fora: usize = dims.get(d + 1..).map_or(1, |r| r.iter().product());
    let fibras = |len: usize, f: &mut dyn FnMut(&[usize])| {
        for o in 0..fora {
            for i in 0..passo {
                let idx: Vec<usize> = (0..len).map(|k| i + passo * (k + len * o)).collect();
                f(&idx);
            }
        }
    };
    match v {
        Value::Complex(a) => {
            let x = a.as_slice();
            let mut out = x.to_vec();
            fibras(nd, &mut |idx| {
                let mut acc = if soma_ {
                    Complex64::new(0.0, 0.0)
                } else {
                    Complex64::new(1.0, 0.0)
                };
                for &p in idx {
                    acc = if soma_ { acc + x[p] } else { acc * x[p] };
                    out[p] = acc;
                }
            });
            Ok(Value::Complex(
                NdArray::from_shape_vec(&dims, out).expect("forma"),
            ))
        }
        Value::Int(i) => {
            let c = i.class();
            let x = i.to_i128();
            let mut out = x.clone();
            fibras(nd, &mut |idx| {
                let mut acc: i128 = if soma_ { 0 } else { 1 };
                for &p in idx {
                    acc = c.saturate(if soma_ {
                        acc.saturating_add(x[p])
                    } else {
                        acc.saturating_mul(x[p])
                    });
                    out[p] = acc;
                }
            });
            Ok(Value::Int(IntData::from_i128(c, &dims, out)))
        }
        outro => {
            let x = outro.to_f64s().map_err(|e| de_runtime(e, n))?;
            let mut out = x.clone();
            fibras(nd, &mut |idx| {
                // Soma compensada (Neumaier) corrente; produto direto.
                let (mut s, mut c, mut p_) = (0.0f64, 0.0f64, 1.0f64);
                for &p in idx {
                    if soma_ {
                        let t = s + x[p];
                        c += if s.abs() >= x[p].abs() {
                            (s - t) + x[p]
                        } else {
                            (x[p] - t) + s
                        };
                        s = t;
                        out[p] = s + c;
                    } else {
                        p_ *= x[p];
                        out[p] = p_;
                    }
                }
            });
            let r = Value::doubles(&dims, out);
            if matches!(outro, Value::Single(_)) {
                r.cast(Class::Single).map_err(|e| de_runtime(e, n))
            } else {
                Ok(r)
            }
        }
    }
}

/// Chave de comparação exata de um elemento, a mesma para todas as classes: os bits do `f64`
/// quando o valor é exato em `double` (−0 igual a +0), e um domínio à parte para inteiros que
/// nenhum `double` representa; NaN nunca é igual a nada (`None`).
fn chaves_de(v: &Value, nome: &str, n: &Node) -> Result<Vec<Option<i128>>, ErroExec> {
    match v {
        Value::Complex(_) => Err(err(
            "chimalab:execucao:classeNaoSuportada",
            format!("`{nome}` de complexos ainda não é suportado"),
            n,
        )),
        Value::Int(i) => Ok(i
            .to_i128()
            .into_iter()
            .map(|x| {
                let f = x as f64;
                if f as i128 == x {
                    Some(i128::from((f + 0.0).to_bits()))
                } else {
                    Some((1i128 << 100) + x)
                }
            })
            .collect()),
        outro => Ok(outro
            .to_f64s()
            .map_err(|e| de_runtime(e, n))?
            .into_iter()
            .map(|x| {
                if x.is_nan() {
                    None
                } else {
                    Some(i128::from((x + 0.0).to_bits()))
                }
            })
            .collect()),
    }
}

/// `unique`: posições (base 0) dos valores distintos em ordem crescente (NaN no fim, cada um
/// distinto), a primeira ocorrência de cada um e, para cada elemento, o índice do seu valor.
fn distintos(v: &Value, nome: &str, n: &Node) -> Result<(Vec<usize>, Vec<usize>), ErroExec> {
    let chaves = chaves_de(v, nome, n)?;
    let k = v.numel();
    let flat = remodela(v.clone(), &[k, 1]);
    let (_, idx) = ordena(&flat, 0, false, n)?;
    let ordem: Vec<usize> = idx
        .to_f64s()
        .expect("índices")
        .iter()
        .map(|&x| x as usize - 1)
        .collect();
    let mut primeiros = Vec::new();
    let mut grupo = vec![0usize; k];
    for (pos, &e) in ordem.iter().enumerate() {
        let novo = pos == 0 || chaves[e].is_none() || chaves[e] != chaves[ordem[pos - 1]];
        if novo {
            primeiros.push(e);
        }
        grupo[e] = primeiros.len() - 1;
    }
    Ok((primeiros, grupo))
}

/// Forma do resultado de uma operação de conjunto: linha se a entrada é vetor linha.
fn forma_conjunto(linha: bool, k: usize) -> [usize; 2] {
    if linha { [1, k] } else { [k, 1] }
}

/// Chave de ordenação: inteiros exatos ou reais (complexos não se ordenam ainda).
enum Chaves {
    Real(Vec<f64>),
    Inteira(Vec<i128>),
}

fn ordena(v: &Value, d: usize, desc: bool, n: &Node) -> Result<(Value, Value), ErroExec> {
    let chaves = match v {
        Value::Complex(_) => {
            return Err(err(
                "chimalab:execucao:classeNaoSuportada",
                "sort de complexos ainda não é suportado",
                n,
            ));
        }
        Value::Int(i) => Chaves::Inteira(i.to_i128()),
        outro => Chaves::Real(outro.to_f64s().map_err(|e| de_runtime(e, n))?),
    };
    let dims = v.dims().to_vec();
    let nd = dims.get(d).copied().unwrap_or(1);
    let passo: usize = dims[..d.min(dims.len())].iter().product();
    let fora: usize = dims.get(d + 1..).map_or(1, |r| r.iter().product());
    let mut perm = vec![0usize; v.numel()];
    let mut ind = vec![0.0f64; v.numel()];
    for o in 0..fora {
        for i in 0..passo {
            let pos: Vec<usize> = (0..nd).map(|k| i + passo * (k + nd * o)).collect();
            let mut ordem: Vec<usize> = (0..nd).collect();
            // Estável; NaN sempre no fim, nas duas ordens.
            match &chaves {
                Chaves::Real(x) => ordem.sort_by(|&a, &b| {
                    let (xa, xb) = (x[pos[a]], x[pos[b]]);
                    match (xa.is_nan(), xb.is_nan()) {
                        (true, true) => std::cmp::Ordering::Equal,
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        _ if desc => xb.partial_cmp(&xa).expect("sem NaN"),
                        _ => xa.partial_cmp(&xb).expect("sem NaN"),
                    }
                }),
                Chaves::Inteira(x) => ordem.sort_by(|&a, &b| {
                    if desc {
                        x[pos[b]].cmp(&x[pos[a]])
                    } else {
                        x[pos[a]].cmp(&x[pos[b]])
                    }
                }),
            }
            for (k, &o_) in ordem.iter().enumerate() {
                perm[pos[k]] = pos[o_];
                ind[pos[k]] = (o_ + 1) as f64;
            }
        }
    }
    Ok((seleciona(v, perm, &dims, n)?, Value::doubles(&dims, ind)))
}

/// Chama a função embutida `nome`.
pub fn chama(
    it: &mut Interpretador,
    nome: &'static str,
    pos: Vec<Obj>,
    nomeados: Vec<(String, Obj)>,
    nargout: usize,
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    if crate::funcionais::atende(nome) {
        return crate::funcionais::chama(it, nome, pos, nomeados, nargout, n);
    }
    if crate::numericos::atende(nome) {
        return crate::numericos::chama(it, nome, pos, nomeados, nargout, n);
    }
    if crate::aleatorios::atende(nome) {
        return crate::aleatorios::chama(it, nome, &pos, &nomeados, n);
    }
    // Opções nomeadas aceitas por função (`range` confere as suas).
    let aceitas: &[&str] = match nome {
        "var" | "std" => &["population"],
        "quantile" => &["type"],
        _ => &[],
    };
    if nome != "range"
        && let Some((k, _)) = nomeados.iter().find(|(k, _)| !aceitas.contains(&k.as_str()))
    {
        return Err(err(
            "chimalab:execucao:argumentos",
            format!("`{nome}` não aceita o argumento nomeado `{k}`"),
            n,
        ));
    }
    if crate::especiais::atende(nome) {
        return crate::especiais::chama(it, nome, &pos, n);
    }
    if crate::algebra::atende(nome) {
        return crate::algebra::chama(it, nome, &pos, nargout, n);
    }
    let um = |v: Value| Ok(vec![Obj::Array(v)]);
    match nome {
        "zeros" | "ones" => {
            let d = forma(nome, &pos, n)?;
            um(Value::Double(NdArray::filled_nd(
                &d,
                if nome == "ones" { 1.0 } else { 0.0 },
            )))
        }
        "eye" => {
            conta(nome, &pos, 1, 2, n)?;
            let r = inteiro(&pos[0], "o tamanho de `eye`", n)?;
            let c = match pos.get(1) {
                Some(o) => inteiro(o, "o tamanho de `eye`", n)?,
                None => r,
            };
            verifica_tamanho(&[r, c]).map_err(|e| de_runtime(e, n))?;
            um(Value::Double(NdArray::from_fn(r, c, |i, j| {
                if i == j { 1.0 } else { 0.0 }
            })))
        }
        "numel" | "ndims" | "length" | "isempty" => {
            conta(nome, &pos, 1, 1, n)?;
            let v = arr(&pos[0], n)?;
            Ok(vec![match nome {
                "numel" => dobro(v.numel() as f64),
                "ndims" => dobro(v.dims().len() as f64),
                "length" => dobro(if v.numel() == 0 {
                    0
                } else {
                    *v.dims().iter().max().expect("≥ 2")
                } as f64),
                _ => logico(v.numel() == 0),
            }])
        }
        "size" => {
            conta(nome, &pos, 1, 2, n)?;
            let d = arr(&pos[0], n)?.dims().to_vec();
            if let Some(o) = pos.get(1) {
                let k = inteiro(o, "a dimensão de `size`", n)?;
                if k == 0 {
                    return Err(err("chimalab:execucao:argumentos", "dimensões começam em 1", n));
                }
                return Ok(vec![dobro(d.get(k - 1).copied().unwrap_or(1) as f64)]);
            }
            if nargout <= 1 {
                return um(Value::doubles(
                    &[1, d.len()],
                    d.iter().map(|&x| x as f64).collect(),
                ));
            }
            // Várias saídas: a última absorve as dimensões restantes.
            let mut out: Vec<Obj> = (0..nargout - 1)
                .map(|k| dobro(d.get(k).copied().unwrap_or(1) as f64))
                .collect();
            let resto: usize = d.get(nargout - 1..).map_or(1, |r| r.iter().product());
            out.push(dobro(resto as f64));
            Ok(out)
        }
        "sum" | "mean" | "any" | "all" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            let d = dim_arg(nome, v, &pos, n)?;
            match nome {
                "sum" => um(soma_valor(v, d, n)?),
                "mean" => {
                    if v.dims() == [0, 0] {
                        return um(Value::scalar(f64::NAN));
                    }
                    let s = soma_valor(&v.cast(Class::Double).map_err(|e| de_runtime(e, n))?, d, n)?;
                    let k = v.dims().get(d).copied().unwrap_or(1) as f64;
                    crate::elementwise(crate::BinOp::Div, &s, &Value::scalar(k))
                        .map(|r| vec![Obj::Array(r)])
                        .map_err(|e| de_runtime(e, n))
                }
                _ => {
                    let todos = nome == "all";
                    if v.dims() == [0, 0] {
                        return Ok(vec![logico(todos)]);
                    }
                    let z = v.to_complex();
                    let (r, dims) = reduz(&z, v.dims(), d, |f| {
                        if todos {
                            f.iter().all(|z| *z != Complex64::new(0.0, 0.0))
                        } else {
                            f.iter().any(|z| *z != Complex64::new(0.0, 0.0) && !z.is_nan())
                        }
                    });
                    um(Value::Logical(NdArray::from_shape_vec(&dims, r).expect("forma")))
                }
            }
        }
        "max" | "min" => {
            conta(nome, &pos, 1, 2, n)?;
            let maior = nome == "max";
            if pos.len() == 2 {
                let (a, b) = (arr(&pos[0], n)?, arr(&pos[1], n)?);
                let f = move |x: f64, y: f64| {
                    if x.is_nan() {
                        y
                    } else if y.is_nan() || (maior && x >= y) || (!maior && x <= y) {
                        x
                    } else {
                        y
                    }
                };
                return um(binaria_real(a, b, nome, f, n)?);
            }
            let v = arr(&pos[0], n)?;
            let (val, idx) = extremo(v, dim_padrao(v.dims()), maior, n)?;
            Ok(vec![Obj::Array(val), Obj::Array(idx)])
        }
        "isnan" | "isinf" | "isfinite" => {
            conta(nome, &pos, 1, 1, n)?;
            let v = arr(&pos[0], n)?;
            let t: Vec<bool> = match v {
                Value::Int(_) | Value::Logical(_) | Value::Char(_) => vec![nome == "isfinite"; v.numel()],
                outro => outro
                    .to_complex()
                    .iter()
                    .map(|z| match nome {
                        "isnan" => z.re.is_nan() || z.im.is_nan(),
                        "isinf" => z.re.is_infinite() || z.im.is_infinite(),
                        _ => z.re.is_finite() && z.im.is_finite(),
                    })
                    .collect(),
            };
            um(Value::Logical(
                NdArray::from_shape_vec(v.dims(), t).expect("forma"),
            ))
        }
        "isreal" => {
            conta(nome, &pos, 1, 1, n)?;
            Ok(vec![logico(!arr(&pos[0], n)?.is_complex())])
        }
        "real" | "imag" | "conj" | "angle" => {
            conta(nome, &pos, 1, 1, n)?;
            let v = arr(&pos[0], n)?;
            if let Value::Complex(a) = v {
                let z = a.as_slice();
                return um(match nome {
                    "real" => Value::doubles(a.dims(), z.iter().map(|z| z.re).collect()),
                    "imag" => Value::doubles(a.dims(), z.iter().map(|z| z.im).collect()),
                    "angle" => Value::doubles(a.dims(), z.iter().map(|z| z.im.atan2(z.re)).collect()),
                    _ => crate::ops::simplify(a.map(|z| z.conj())),
                });
            }
            match nome {
                "real" | "conj" => um(v.clone()),
                "imag" => {
                    let zero = Value::doubles(v.dims(), vec![0.0; v.numel()]);
                    um(match v {
                        Value::Int(_) | Value::Single(_) => {
                            zero.cast(v.class()).map_err(|e| de_runtime(e, n))?
                        }
                        _ => zero,
                    })
                }
                _ => {
                    let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
                    um(Value::doubles(
                        v.dims(),
                        x.iter().map(|&e| 0f64.atan2(e)).collect(),
                    ))
                }
            }
        }
        "atan2" | "hypot" | "rem" => {
            conta(nome, &pos, 2, 2, n)?;
            let f: fn(f64, f64) -> f64 = match nome {
                "atan2" => f64::atan2,
                "hypot" => f64::hypot,
                _ => |a, b| a % b,
            };
            um(binaria_real(arr(&pos[0], n)?, arr(&pos[1], n)?, nome, f, n)?)
        }
        "help" => {
            conta(nome, &pos, 0, 1, n)?;
            let t = match pos.first() {
                None => crate::ajuda::indice(),
                Some(Obj::Embutida(e)) => {
                    crate::ajuda::de(e).unwrap_or_else(|| format!("sem ajuda para `{e}`"))
                }
                Some(Obj::Array(Value::Char(_))) => {
                    let t = texto(&pos[0], "o nome", n)?;
                    crate::ajuda::de(&t).unwrap_or_else(|| format!("sem ajuda para `{t}`"))
                }
                Some(Obj::Array(_)) => {
                    return Err(err(
                        "chimalab:execucao:argumentos",
                        "help(nome) ou help(\"nome\")",
                        n,
                    ));
                }
                Some(f) => exibicao::corpo(f),
            };
            it.saidas.push(Saida::Texto(t));
            Ok(Vec::new())
        }
        "unique" => {
            conta(nome, &pos, 1, 1, n)?;
            let v = arr(&pos[0], n)?;
            let (primeiros, grupo) = distintos(v, nome, n)?;
            let linha = v.dims() == [1, v.numel()] && v.numel() > 0;
            let k = primeiros.len();
            let u = seleciona(v, primeiros.clone(), &forma_conjunto(linha, k), n)?;
            let i = Value::doubles(
                &forma_conjunto(linha, k),
                primeiros.iter().map(|&p| (p + 1) as f64).collect(),
            );
            let j = Value::doubles(
                &forma_conjunto(linha, grupo.len()),
                grupo.iter().map(|&g| (g + 1) as f64).collect(),
            );
            Ok(vec![Obj::Array(u), Obj::Array(i), Obj::Array(j)])
        }
        "ismember" => {
            conta(nome, &pos, 2, 2, n)?;
            let (a, s) = (arr(&pos[0], n)?, arr(&pos[1], n)?);
            let (ca, cs) = (chaves_de(a, nome, n)?, chaves_de(s, nome, n)?);
            let mut primeiro = std::collections::HashMap::new();
            for (k, c) in cs.iter().enumerate() {
                if let Some(c) = c {
                    primeiro.entry(*c).or_insert(k + 1);
                }
            }
            let achado: Vec<usize> = ca
                .iter()
                .map(|c| c.and_then(|c| primeiro.get(&c).copied()).unwrap_or(0))
                .collect();
            Ok(vec![
                Obj::Array(Value::Logical(
                    NdArray::from_shape_vec(a.dims(), achado.iter().map(|&k| k > 0).collect())
                        .expect("forma"),
                )),
                Obj::Array(Value::doubles(
                    a.dims(),
                    achado.iter().map(|&k| k as f64).collect(),
                )),
            ])
        }
        "union" | "intersect" | "setdiff" => {
            conta(nome, &pos, 2, 2, n)?;
            let (a, b) = (arr(&pos[0], n)?, arr(&pos[1], n)?);
            let coluna = |v: &Value| v.dims().len() == 2 && v.dims()[1] == 1 && v.numel() > 0;
            let linha = !(coluna(a) && (coluna(b) || b.numel() == 0));
            let ra = remodela(a.clone(), &[1, a.numel()]);
            let rb = remodela(b.clone(), &[1, b.numel()]);
            let junto = crate::horzcat(&[ra.clone(), rb.clone()]).map_err(|e| de_runtime(e, n))?;
            let (base, primeiros) = match nome {
                "union" => {
                    let (p, _) = distintos(&junto, nome, n)?;
                    (junto, p)
                }
                _ => {
                    // Distintos de a, filtrados pela presença (intersect) ou ausência (setdiff) em b.
                    let (p, _) = distintos(&ra, nome, n)?;
                    let (ca, cb) = (chaves_de(&ra, nome, n)?, chaves_de(&rb, nome, n)?);
                    let em_b: std::collections::HashSet<i128> = cb.into_iter().flatten().collect();
                    let quer = nome == "intersect";
                    let p = p
                        .into_iter()
                        .filter(|&k| {
                            ca[k].is_some_and(|c| em_b.contains(&c)) == quer || (!quer && ca[k].is_none())
                        })
                        .collect();
                    // Mesma classe da concatenação (a regra de classe vale para os dois operandos).
                    let base = ra.cast(junto.class()).map_err(|e| de_runtime(e, n))?;
                    (base, p)
                }
            };
            let k = primeiros.len();
            um(seleciona(&base, primeiros, &forma_conjunto(linha, k), n)?)
        }
        "who" | "whos" => {
            conta(nome, &pos, 0, 0, n)?;
            let vars = it.variaveis_atuais();
            let t = if nome == "who" {
                vars.iter()
                    .map(|(k, _)| k.as_str())
                    .collect::<Vec<_>>()
                    .join("  ")
            } else {
                let linhas: Vec<[String; 4]> = vars
                    .iter()
                    .map(|(k, o)| match o {
                        Obj::Array(v) => [
                            k.clone(),
                            exibicao::tamanho(v.dims()),
                            v.class().name().to_string(),
                            bytes(v).to_string(),
                        ],
                        _ => [k.clone(), "1×1".into(), "function".into(), "—".into()],
                    })
                    .collect();
                let cab = [
                    "nome".to_string(),
                    "tamanho".into(),
                    "classe".into(),
                    "bytes".into(),
                ];
                let todas: Vec<&[String; 4]> = std::iter::once(&cab).chain(linhas.iter()).collect();
                let larg: Vec<usize> = (0..4)
                    .map(|c| todas.iter().map(|l| l[c].chars().count()).max().unwrap_or(0))
                    .collect();
                todas
                    .iter()
                    .map(|l| {
                        format!(
                            "{:<w0$}  {:<w1$}  {:<w2$}  {:>w3$}",
                            l[0],
                            l[1],
                            l[2],
                            l[3],
                            w0 = larg[0],
                            w1 = larg[1],
                            w2 = larg[2],
                            w3 = larg[3]
                        )
                        .trim_end()
                        .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if !t.is_empty() {
                it.saidas.push(Saida::Texto(t));
            }
            Ok(Vec::new())
        }
        "clear" => {
            if pos.is_empty() {
                it.apaga_todas();
            } else {
                for o in &pos {
                    let k = texto(o, "cada nome de `clear`", n)?;
                    it.apaga(&k);
                }
            }
            Ok(Vec::new())
        }
        "median" | "mode" | "var" | "std" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            if v.is_complex() {
                return Err(err(
                    "chimalab:execucao:classeNaoSuportada",
                    format!("`{nome}` de complexos ainda não é suportado"),
                    n,
                ));
            }
            let d = dim_arg(nome, v, &pos, n)?;
            let pop = match nomeados.first() {
                Some((_, o)) => escalar_f64(o, "`population`", n)? != 0.0,
                None => false,
            };
            let norm = if pop {
                stats::VarNorm::Population
            } else {
                stats::VarNorm::Sample
            };
            if v.dims() == [0, 0] {
                return um(Value::scalar(f64::NAN));
            }
            let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
            let (r, dims) = reduz(&x, v.dims(), d, |f| match nome {
                "median" => stats::median(f, stats::NanFlag::Include),
                "mode" => stats::mode(f),
                "var" => stats::var(f, norm, stats::NanFlag::Include),
                _ => stats::std(f, norm, stats::NanFlag::Include),
            });
            let r = Value::doubles(&dims, r);
            um(if matches!(v, Value::Single(_)) {
                r.cast(Class::Single).map_err(|e| de_runtime(e, n))?
            } else {
                r
            })
        }
        "quantile" => {
            conta(nome, &pos, 2, 2, n)?;
            let v = arr(&pos[0], n)?;
            if v.is_complex() {
                return Err(err(
                    "chimalab:execucao:classeNaoSuportada",
                    "`quantile` de complexos ainda não é suportado",
                    n,
                ));
            }
            let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
            let pv = arr(&pos[1], n)?;
            let p = pv.to_f64s().map_err(|e| de_runtime(e, n))?;
            let tipo = match nomeados.first() {
                Some((_, o)) => {
                    let k = inteiro(o, "`type`", n)?;
                    stats::QuantileType::new(u8::try_from(k).unwrap_or(0))
                        .map_err(|e| erro_em(e.id(), e.to_string(), n))?
                }
                None => stats::QuantileType::default(),
            };
            let q = stats::quantile(&x, &p, tipo, stats::NanFlag::Include)
                .map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            um(Value::doubles(pv.dims(), q))
        }
        "dot" | "cross" => {
            conta(nome, &pos, 2, 2, n)?;
            let (a, b) = (arr(&pos[0], n)?, arr(&pos[1], n)?);
            let vetor = |v: &Value| v.dims().len() == 2 && (v.dims()[0] == 1 || v.dims()[1] == 1);
            if !vetor(a) || !vetor(b) || a.numel() != b.numel() {
                return Err(err(
                    "chimalab:runtime:dimensoesIncompativeis",
                    format!(
                        "`{nome}` recebe dois vetores do mesmo tamanho (vieram {} e {})",
                        exibicao::tamanho(a.dims()),
                        exibicao::tamanho(b.dims())
                    ),
                    n,
                ));
            }
            let (za, zb) = (a.to_complex(), b.to_complex());
            if nome == "dot" {
                // Produto interno: conjuga o primeiro (⟨a, b⟩ = Σ conj(aᵢ)·bᵢ).
                let s: Complex64 = za.iter().zip(&zb).map(|(x, y)| x.conj() * y).sum();
                return um(crate::ops::simplify(NdArray::filled_nd(&[1, 1], s)));
            }
            if a.numel() != 3 {
                return Err(err(
                    "chimalab:runtime:dimensoesIncompativeis",
                    "`cross` recebe vetores de 3 elementos",
                    n,
                ));
            }
            let c = [
                za[1] * zb[2] - za[2] * zb[1],
                za[2] * zb[0] - za[0] * zb[2],
                za[0] * zb[1] - za[1] * zb[0],
            ];
            um(crate::ops::simplify(
                NdArray::from_shape_vec(a.dims(), c.to_vec()).expect("forma"),
            ))
        }
        "kron" => {
            conta(nome, &pos, 2, 2, n)?;
            let (a, b) = (arr(&pos[0], n)?, arr(&pos[1], n)?);
            if a.dims().len() > 2 || b.dims().len() > 2 {
                return Err(err(
                    "chimalab:execucao:argumentos",
                    "`kron` recebe matrizes 2-D",
                    n,
                ));
            }
            let (ra, ca, rb, cb) = (a.dims()[0], a.dims()[1], b.dims()[0], b.dims()[1]);
            let (r, c) = (ra.saturating_mul(rb), ca.saturating_mul(cb));
            verifica_tamanho(&[r, c]).map_err(|e| de_runtime(e, n))?;
            let (za, zb) = (a.to_complex(), b.to_complex());
            let z = NdArray::from_fn(r, c, |i, j| {
                za[i / rb + (j / cb) * ra] * zb[i % rb + (j % cb) * rb]
            });
            um(crate::ops::simplify(z))
        }
        "flip" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            let d = dim_arg(nome, v, &pos, n)?;
            let dims = v.dims().to_vec();
            let nd = dims.get(d).copied().unwrap_or(1);
            let passo: usize = dims[..d.min(dims.len())].iter().product();
            let pos_: Vec<usize> = (0..v.numel())
                .map(|k| {
                    let i = (k / passo) % nd;
                    k - i * passo + (nd - 1 - i) * passo
                })
                .collect();
            um(seleciona(v, pos_, &dims, n)?)
        }
        "class" => {
            conta(nome, &pos, 1, 1, n)?;
            um(Value::text(match &pos[0] {
                Obj::Array(v) => v.class().name(),
                _ => "function",
            }))
        }
        "round" if pos.len() == 2 => {
            // round(x, n): n casas decimais (n negativo arredonda à esquerda da vírgula).
            let v = arr(&pos[0], n)?;
            let casas = escalar_f64(&pos[1], "o número de casas de `round`", n)?;
            if !(casas.fract() == 0.0 && casas.abs() <= 400.0) {
                return Err(err(
                    "chimalab:execucao:argumentos",
                    "o número de casas de `round` é um inteiro",
                    n,
                ));
            }
            let c = casas as i32;
            if let Value::Complex(a) = v {
                return um(crate::ops::simplify(
                    a.map(|z| Complex64::new(stats_round(z.re, c), stats_round(z.im, c))),
                ));
            }
            let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
            let r = Value::doubles(v.dims(), x.into_iter().map(|e| stats_round(e, c)).collect());
            um(match v {
                Value::Int(_) | Value::Single(_) => r.cast(v.class()).map_err(|e| de_runtime(e, n))?,
                _ => r,
            })
        }
        "diff" => {
            conta(nome, &pos, 1, 3, n)?;
            let v = arr(&pos[0], n)?;
            let ordem = match pos.get(1) {
                Some(o) => inteiro(o, "a ordem de `diff`", n)?,
                None => 1,
            };
            let d = match pos.get(2) {
                Some(o) => {
                    let k = inteiro(o, "a dimensão de `diff`", n)?;
                    if k == 0 {
                        return Err(err("chimalab:execucao:argumentos", "dimensões começam em 1", n));
                    }
                    k - 1
                }
                None => dim_padrao(v.dims()),
            };
            let mut dims = v.dims().to_vec();
            while dims.len() <= d {
                dims.push(1);
            }
            let mut z = v.to_complex();
            for _ in 0..ordem {
                let nd = dims[d];
                if nd == 0 {
                    break;
                }
                let passo: usize = dims[..d].iter().product();
                let fora: usize = dims[d + 1..].iter().product();
                let mut novo = Vec::with_capacity(passo * (nd - 1) * fora);
                for o in 0..fora {
                    for k in 0..nd - 1 {
                        for i in 0..passo {
                            novo.push(z[i + passo * (k + 1 + nd * o)] - z[i + passo * (k + nd * o)]);
                        }
                    }
                }
                dims[d] = nd - 1;
                z = novo;
            }
            let dims = chimalab_array::normalize_shape(&dims);
            let r = crate::ops::simplify(NdArray::from_shape_vec(&dims, z).expect("forma"));
            um(match v {
                Value::Int(_) | Value::Single(_) => r.cast(v.class()).map_err(|e| de_runtime(e, n))?,
                _ => r,
            })
        }
        "abs" | "sqrt" | "exp" | "log" | "sin" | "cos" | "tan" | "floor" | "ceil" | "round" | "sign"
        | "fix" | "log10" | "log2" | "sinh" | "cosh" | "tanh" | "asin" | "acos" | "atan" => {
            conta(nome, &pos, 1, 1, n)?;
            um(elementar(nome, arr(&pos[0], n)?, n)?)
        }
        "mod" => {
            conta(nome, &pos, 2, 2, n)?;
            um(binaria_real(arr(&pos[0], n)?, arr(&pos[1], n)?, nome, resto, n)?)
        }
        "range" => {
            conta(nome, &pos, 2, 2, n)?;
            let a = escalar_f64(&pos[0], "o início de `range`", n)?;
            let b = escalar_f64(&pos[1], "o fim de `range`", n)?;
            match nomeados.as_slice() {
                [(k, v)] if k == "length" => {
                    let m = inteiro(v, "`length`", n)?;
                    verifica_tamanho(&[1, m]).map_err(|e| de_runtime(e, n))?;
                    um(linspace(a, b, m))
                }
                [(k, v)] if k == "step" => {
                    let p = escalar_f64(v, "`step`", n)?;
                    verifica_tamanho(&[1, crate::colon_n(a, p, b)]).map_err(|e| de_runtime(e, n))?;
                    um(crate::colon(a, p, b))
                }
                [] => Err(err(
                    "chimalab:execucao:argumentos",
                    "`range` precisa de `length=` ou `step=`: range(0, 1; length=5)",
                    n,
                )),
                _ => Err(err(
                    "chimalab:execucao:argumentos",
                    "`range` aceita só um entre `length=` e `step=`",
                    n,
                )),
            }
        }
        "error" => {
            conta(nome, &pos, 1, 2, n)?;
            let (id, msg) = if pos.len() == 2 {
                (
                    texto(&pos[0], "o identificador", n)?,
                    texto(&pos[1], "a mensagem", n)?,
                )
            } else {
                (
                    "chimalab:usuario:erro".to_string(),
                    texto(&pos[0], "a mensagem", n)?,
                )
            };
            Err(ErroExec {
                id,
                mensagem: msg,
                linha: n.line,
                coluna: n.col,
            })
        }
        "disp" => {
            conta(nome, &pos, 1, 1, n)?;
            it.saidas.push(Saida::Texto(exibicao::corpo(&pos[0])));
            Ok(Vec::new())
        }
        "prod" | "cumsum" | "cumprod" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            let d = dim_arg(nome, v, &pos, n)?;
            match nome {
                "prod" => um(produto_valor(v, d, n)?),
                _ => um(acumula(v, d, nome == "cumsum", n)?),
            }
        }
        "sort" => {
            conta(nome, &pos, 1, 3, n)?;
            let v = arr(&pos[0], n)?;
            let mut d = dim_padrao(v.dims());
            let mut desc = false;
            for o in &pos[1..] {
                match o {
                    Obj::Array(Value::Char(_)) => match texto(o, "a ordem", n)?.as_str() {
                        "ascend" => desc = false,
                        "descend" => desc = true,
                        _ => {
                            return Err(err(
                                "chimalab:execucao:argumentos",
                                "a ordem é \"ascend\" ou \"descend\"",
                                n,
                            ));
                        }
                    },
                    _ => {
                        let k = inteiro(o, "a dimensão de `sort`", n)?;
                        if k == 0 {
                            return Err(err("chimalab:execucao:argumentos", "dimensões começam em 1", n));
                        }
                        d = k - 1;
                    }
                }
            }
            let (s, i) = ordena(v, d, desc, n)?;
            Ok(vec![Obj::Array(s), Obj::Array(i)])
        }
        "find" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            let limite = match pos.get(1) {
                Some(o) => inteiro(o, "o número de índices de `find`", n)?,
                None => usize::MAX,
            };
            let z = v.to_complex();
            let idx: Vec<f64> = z
                .iter()
                .enumerate()
                .filter(|(_, z)| **z != Complex64::new(0.0, 0.0))
                .map(|(k, _)| (k + 1) as f64)
                .take(limite)
                .collect();
            let k = idx.len();
            // Vetor linha dá linha; o resto, coluna.
            let linha = v.dims().len() == 2 && v.dims()[0] == 1 && v.numel() > 0;
            let forma = if linha { [1, k] } else { [k, 1] };
            um(Value::doubles(&forma, idx))
        }
        "reshape" => {
            conta(nome, &pos, 2, usize::MAX, n)?;
            let v = arr(&pos[0], n)?;
            // Dimensões em argumentos separados ou num vetor; uma delas pode ser -1 (inferida).
            let brutos: Vec<f64> = if pos.len() == 2 {
                arr(&pos[1], n)?.to_f64s().map_err(|e| de_runtime(e, n))?
            } else {
                pos[1..]
                    .iter()
                    .map(|o| escalar_f64(o, "cada dimensão de `reshape`", n))
                    .collect::<Result<_, _>>()?
            };
            let invalido = || {
                err(
                    "chimalab:execucao:argumentos",
                    "as dimensões de `reshape` são inteiros ≥ 0, com no máximo um -1",
                    n,
                )
            };
            if brutos.len() < 2 || brutos.iter().filter(|&&x| x == -1.0).count() > 1 {
                return Err(invalido());
            }
            if brutos
                .iter()
                .any(|&x| x != -1.0 && !(x >= 0.0 && x.fract() == 0.0 && x.is_finite()))
            {
                return Err(invalido());
            }
            let conhecido: usize = brutos
                .iter()
                .filter(|&&x| x != -1.0)
                .map(|&x| x as usize)
                .product();
            let mut dims: Vec<usize> = brutos
                .iter()
                .map(|&x| if x == -1.0 { 0 } else { x as usize })
                .collect();
            if let Some(k) = brutos.iter().position(|&x| x == -1.0) {
                if conhecido == 0 || v.numel() % conhecido != 0 {
                    return Err(err(
                        "chimalab:runtime:dimensoesIncompativeis",
                        format!(
                            "reshape: {} elementos não se dividem em blocos de {conhecido}",
                            v.numel()
                        ),
                        n,
                    ));
                }
                dims[k] = v.numel() / conhecido;
            }
            let total = dims.iter().try_fold(1usize, |a, &d| a.checked_mul(d));
            if total != Some(v.numel()) {
                return Err(err(
                    "chimalab:runtime:dimensoesIncompativeis",
                    format!(
                        "reshape: {} elementos não cabem em {}",
                        v.numel(),
                        exibicao::tamanho(&dims)
                    ),
                    n,
                ));
            }
            let dims = chimalab_array::normalize_shape(&dims);
            um(remodela(v.clone(), &dims))
        }
        "diag" => {
            conta(nome, &pos, 1, 2, n)?;
            let v = arr(&pos[0], n)?;
            let k = match pos.get(1) {
                Some(o) => {
                    let x = escalar_f64(o, "o deslocamento de `diag`", n)?;
                    if x.fract() != 0.0 || !x.is_finite() {
                        return Err(err(
                            "chimalab:execucao:argumentos",
                            "o deslocamento de `diag` é inteiro",
                            n,
                        ));
                    }
                    x as i64
                }
                None => 0,
            };
            if v.dims().len() > 2 {
                return Err(err(
                    "chimalab:execucao:argumentos",
                    "`diag` exige vetor ou matriz 2-D",
                    n,
                ));
            }
            let (r, c) = (v.dims()[0], v.dims()[1]);
            let ka = k.unsigned_abs() as usize;
            if r == 1 || c == 1 || (r == 0 && c == 0) {
                // Vetor (ou o vazio 0×0) → matriz quadrada com o vetor na k-ésima diagonal.
                let m = v.numel() + ka;
                verifica_tamanho(&[m, m]).map_err(|e| de_runtime(e, n))?;
                let pos_: Vec<usize> = (0..v.numel())
                    .map(|i| {
                        if k >= 0 {
                            i + (i + ka) * m + 1
                        } else {
                            (i + ka) + i * m + 1
                        }
                    })
                    .collect();
                let zero = Value::doubles(&[m, m], vec![0.0; m * m]);
                let mut out = if matches!(v, Value::Complex(_)) {
                    zero
                } else {
                    zero.cast(v.class()).map_err(|e| de_runtime(e, n))?
                };
                if !pos_.is_empty() {
                    subsasgn(&mut out, &[Index::List(pos_)], v).map_err(|e| de_runtime(e, n))?;
                }
                return um(out);
            }
            // Matriz → coluna com a k-ésima diagonal.
            let pos_: Vec<usize> = (0..r.max(c))
                .filter_map(|i| {
                    let (li, co) = if k >= 0 { (i, i + ka) } else { (i + ka, i) };
                    (li < r && co < c).then_some(li + co * r)
                })
                .collect();
            let len = pos_.len();
            um(seleciona(v, pos_, &[len, 1], n)?)
        }
        "isequal" => {
            conta(nome, &pos, 2, usize::MAX, n)?;
            let a = arr(&pos[0], n)?;
            let mut r = true;
            for o in &pos[1..] {
                r &= iguais(a, arr(o, n)?);
            }
            Ok(vec![logico(r)])
        }
        _ => {
            let c = classe(nome).expect("conversão de classe");
            conta(nome, &pos, 1, 1, n)?;
            um(arr(&pos[0], n)?.cast(c).map_err(|e| de_runtime(e, n))?)
        }
    }
}
