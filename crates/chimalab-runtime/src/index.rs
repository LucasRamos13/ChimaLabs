//! Indexação na base 1 (F004): leitura, atribuição com crescimento e remoção.
//!
//! Regras:
//! - Um índice só é linear (column-major): `A(:)` dá uma coluna. Uma lista aplicada a um vetor
//!   segue a orientação dele; aplicada a uma matriz, dá uma linha.
//! - Com n subscritos e mais dimensões que n, o último subscrito percorre as dimensões
//!   restantes juntas. Subscritos além das dimensões existentes devem valer 1.
//! - Atribuir fora dos limites cresce o array, preenchendo com o zero da classe. Pelo índice
//!   linear, só cresce um vetor (ou vazio, que vira linha).
//! - O lado direito é convertido para a classe do alvo. Um alvo vazio adota a classe do lado
//!   direito, e um complexo atribuído a um `double` real torna o alvo complexo.

use chimalab_array::{Complex64, Element, NdArray};

use crate::{IntData, RuntimeError, Value};

/// Um subscrito.
#[derive(Clone, Debug, PartialEq)]
pub enum Index {
    /// Todos (`:`).
    All,
    /// Um índice (base 1).
    Scalar(usize),
    /// Lista de índices (base 1).
    List(Vec<usize>),
    /// Máscara lógica.
    Logical(Vec<bool>),
}

impl Value {
    /// Valor de `end` no subscrito `k` (0-based) de `n` subscritos.
    pub fn end_index(&self, k: usize, n: usize) -> usize {
        end_index(self.dims(), k, n)
    }
}

/// Valor de `end` no subscrito `k` (0-based) de `n` subscritos de um array com forma `d`.
pub fn end_index(d: &[usize], k: usize, n: usize) -> usize {
    if n == 1 {
        d.iter().product()
    } else if k + 1 < n {
        d.get(k).copied().unwrap_or(1)
    } else {
        d[k.min(d.len())..]
            .iter()
            .product::<usize>()
            .max(if k >= d.len() { 1 } else { 0 })
    }
}

/// Posições (0-based) selecionadas por um subscrito numa dimensão de tamanho `n`.
fn positions(ix: &Index, n: usize) -> Result<Vec<usize>, RuntimeError> {
    let check = |i: usize| {
        if i == 0 {
            Err(RuntimeError::InvalidIndex("índices começam em 1".into()))
        } else {
            Ok(i - 1)
        }
    };
    match ix {
        Index::All => Ok((0..n).collect()),
        Index::Scalar(i) => Ok(vec![check(*i)?]),
        Index::List(v) => v.iter().map(|&i| check(i)).collect(),
        Index::Logical(m) => {
            // Máscara mais longa que a dimensão só é aceita se o excesso for falso.
            if m.iter().skip(n).any(|&b| b) {
                return Err(RuntimeError::IndexOutOfBounds {
                    index: m.iter().rposition(|&b| b).map_or(0, |p| p + 1),
                    bound: n,
                });
            }
            Ok(m.iter()
                .enumerate()
                .filter(|(_, b)| **b)
                .map(|(i, _)| i)
                .collect())
        }
    }
}

/// Dimensões efetivas para n subscritos: as n − 1 primeiras e o produto das demais.
fn folded(dims: &[usize], n: usize) -> Vec<usize> {
    let mut out: Vec<usize> = (0..n.saturating_sub(1))
        .map(|d| dims.get(d).copied().unwrap_or(1))
        .collect();
    out.push(dims.get(n - 1..).map_or(1, |r| {
        r.iter().product::<usize>().max(if r.is_empty() { 1 } else { 0 })
    }));
    out
}

fn is_vector(dims: &[usize]) -> bool {
    dims.len() == 2 && (dims[0] == 1 || dims[1] == 1)
}

fn select<T: Element>(a: &NdArray<T>, idx: &[Index]) -> Result<NdArray<T>, RuntimeError> {
    let dims = a.dims();
    let data = a.as_slice();
    if idx.len() == 1 {
        let n = a.numel();
        let pos = positions(&idx[0], n)?;
        if let Some(&p) = pos.iter().find(|&&p| p >= n) {
            return Err(RuntimeError::IndexOutOfBounds {
                index: p + 1,
                bound: n,
            });
        }
        let v: Vec<T> = pos.iter().map(|&p| data[p].clone()).collect();
        let k = v.len();
        let shape = match &idx[0] {
            Index::All => vec![k, 1],
            _ if is_vector(dims) && dims[1] == 1 && !(dims[0] == 1 && dims[1] == 1) => vec![k, 1],
            Index::Logical(_) if !(is_vector(dims) && dims[0] == 1) => vec![k, 1],
            _ => vec![1, k],
        };
        return Ok(NdArray::from_shape_vec(&shape, v).expect("forma"));
    }
    let fd = folded(dims, idx.len());
    let sel: Vec<Vec<usize>> = idx
        .iter()
        .zip(&fd)
        .map(|(ix, &n)| positions(ix, n))
        .collect::<Result<_, _>>()?;
    for (s, &n) in sel.iter().zip(&fd) {
        if let Some(&p) = s.iter().find(|&&p| p >= n) {
            return Err(RuntimeError::IndexOutOfBounds {
                index: p + 1,
                bound: n,
            });
        }
    }
    let out_dims: Vec<usize> = sel.iter().map(Vec::len).collect();
    let mut strides = vec![1usize; fd.len()];
    for d in 1..fd.len() {
        strides[d] = strides[d - 1] * fd[d - 1];
    }
    let total: usize = out_dims.iter().product();
    let mut v = Vec::with_capacity(total);
    let mut sub = vec![0usize; sel.len()];
    for _ in 0..total {
        let lin: usize = sub.iter().enumerate().map(|(d, &s)| sel[d][s] * strides[d]).sum();
        v.push(data[lin].clone());
        for (d, s) in sub.iter_mut().enumerate() {
            *s += 1;
            if *s < out_dims[d] {
                break;
            }
            *s = 0;
        }
    }
    Ok(NdArray::from_shape_vec(&out_dims, v).expect("forma"))
}

fn assign<T: Element>(
    a: &mut NdArray<T>,
    idx: &[Index],
    rhs: &NdArray<T>,
    zero: T,
) -> Result<(), RuntimeError> {
    let dims = a.dims().to_vec();
    if idx.len() == 1 {
        let n = a.numel();
        let pos = positions(&idx[0], n)?;
        check_count(pos.len(), rhs.numel())?;
        let maior = pos.iter().copied().max().map_or(0, |p| p + 1);
        if maior > n {
            // Crescimento linear: só vetores (ou vazio, que vira linha).
            let novo = if n == 0 || (dims.len() == 2 && dims[0] == 1) {
                vec![1, maior]
            } else if dims.len() == 2 && dims[1] == 1 {
                vec![maior, 1]
            } else {
                return Err(RuntimeError::InvalidIndex(
                    "atribuição linear fora dos limites só cresce vetores".into(),
                ));
            };
            let mut v = a.as_slice().to_vec();
            v.resize(maior, zero);
            *a = NdArray::from_shape_vec(&novo, v).expect("forma");
        }
        let r = rhs.as_slice();
        let dados = a.as_mut_slice();
        for (k, &p) in pos.iter().enumerate() {
            dados[p] = if r.len() == 1 { r[0].clone() } else { r[k].clone() };
        }
        return Ok(());
    }
    // Subscritos: cresce cada dimensão até o maior índice pedido.
    let fd = folded(&dims, idx.len());
    let sel: Vec<Vec<usize>> = idx
        .iter()
        .zip(&fd)
        .map(|(ix, &n)| positions(ix, n))
        .collect::<Result<_, _>>()?;
    let total: usize = sel.iter().map(Vec::len).product();
    check_count(total, rhs.numel())?;
    let novo: Vec<usize> = sel
        .iter()
        .zip(&fd)
        .map(|(s, &n)| s.iter().map(|p| p + 1).max().unwrap_or(0).max(n))
        .collect();
    if novo != fd {
        if idx.len() < dims.len() && novo[..] != fd[..] {
            return Err(RuntimeError::InvalidIndex(
                "crescimento com dimensões dobradas no último subscrito".into(),
            ));
        }
        let mut b = NdArray::filled_nd(&novo, zero);
        // Copia o antigo para o canto.
        let old = NdArray::from_shape_vec(&fd, a.as_slice().to_vec()).expect("forma");
        let ntot: usize = fd.iter().product();
        for lin in 0..ntot {
            let sub = old.subscripts(lin);
            b.set_nd(&sub, old.as_slice()[lin].clone())
                .expect("dentro do novo");
        }
        *a = b;
    }
    let ad = a.dims().to_vec();
    let fd = folded(&ad, idx.len());
    let mut strides = vec![1usize; fd.len()];
    for d in 1..fd.len() {
        strides[d] = strides[d - 1] * fd[d - 1];
    }
    let r = rhs.as_slice();
    let lens: Vec<usize> = sel.iter().map(Vec::len).collect();
    let mut sub = vec![0usize; sel.len()];
    let dados = a.as_mut_slice();
    for k in 0..total {
        let lin: usize = sub.iter().enumerate().map(|(d, &s)| sel[d][s] * strides[d]).sum();
        dados[lin] = if r.len() == 1 { r[0].clone() } else { r[k].clone() };
        for (d, s) in sub.iter_mut().enumerate() {
            *s += 1;
            if *s < lens[d] {
                break;
            }
            *s = 0;
        }
    }
    Ok(())
}

fn check_count(selected: usize, rhs: usize) -> Result<(), RuntimeError> {
    if rhs != 1 && rhs != selected {
        return Err(RuntimeError::IncompatibleShapes {
            operation: "atribuição indexada",
            left: vec![selected],
            right: vec![rhs],
        });
    }
    Ok(())
}

fn delete<T: Element>(a: &NdArray<T>, idx: &[Index]) -> Result<NdArray<T>, RuntimeError> {
    let dims = a.dims().to_vec();
    if idx.len() == 1 {
        let n = a.numel();
        let pos = positions(&idx[0], n)?;
        if let Some(&p) = pos.iter().find(|&&p| p >= n) {
            return Err(RuntimeError::IndexOutOfBounds {
                index: p + 1,
                bound: n,
            });
        }
        let mut tira = vec![false; n];
        pos.iter().for_each(|&p| tira[p] = true);
        let v: Vec<T> = a
            .as_slice()
            .iter()
            .zip(&tira)
            .filter(|(_, t)| !**t)
            .map(|(x, _)| x.clone())
            .collect();
        let k = v.len();
        let shape = if idx[0] == Index::All {
            vec![0, 0]
        } else if dims.len() == 2 && dims[1] == 1 && dims[0] != 1 {
            vec![k, 1]
        } else {
            vec![1, k]
        };
        return Ok(NdArray::from_shape_vec(&shape, v).expect("forma"));
    }
    // Só um subscrito pode não ser "tudo": remove essas fatias.
    let fd = folded(&dims, idx.len());
    let nao_todos: Vec<usize> = (0..idx.len())
        .filter(|&d| {
            !(idx[d] == Index::All
                || positions(&idx[d], fd[d])
                    .map(|p| p.len() == fd[d] && p.iter().enumerate().all(|(i, &x)| i == x))
                    .unwrap_or(false))
        })
        .collect();
    if nao_todos.len() > 1 {
        return Err(RuntimeError::InvalidIndex(
            "remoção exige ':' em todos os subscritos menos um".into(),
        ));
    }
    let Some(&d) = nao_todos.first() else {
        // Todos ':' : remove tudo, mantendo as outras dimensões com a primeira zerada.
        let mut s = fd.clone();
        s[0] = 0;
        return Ok(NdArray::from_shape_vec(&s, Vec::new()).expect("forma"));
    };
    let pos = positions(&idx[d], fd[d])?;
    if let Some(&p) = pos.iter().find(|&&p| p >= fd[d]) {
        return Err(RuntimeError::IndexOutOfBounds {
            index: p + 1,
            bound: fd[d],
        });
    }
    let mut fica = vec![true; fd[d]];
    pos.iter().for_each(|&p| fica[p] = false);
    let sel: Vec<Index> = (0..idx.len())
        .map(|k| {
            if k == d {
                Index::List((0..fd[d]).filter(|&p| fica[p]).map(|p| p + 1).collect())
            } else {
                Index::All
            }
        })
        .collect();
    let folded_arr = NdArray::from_shape_vec(&fd, a.as_slice().to_vec()).expect("forma");
    select(&folded_arr, &sel)
}

macro_rules! dispatch {
    ($v:expr, $a:ident => $e:expr) => {
        match $v {
            Value::Double($a) => Value::Double($e),
            Value::Complex($a) => Value::Complex($e),
            Value::Single($a) => Value::Single($e),
            Value::Logical($a) => Value::Logical($e),
            Value::Char($a) => Value::Char($e),
            Value::Int(IntData::I8($a)) => Value::Int(IntData::I8($e)),
            Value::Int(IntData::I16($a)) => Value::Int(IntData::I16($e)),
            Value::Int(IntData::I32($a)) => Value::Int(IntData::I32($e)),
            Value::Int(IntData::I64($a)) => Value::Int(IntData::I64($e)),
            Value::Int(IntData::U8($a)) => Value::Int(IntData::U8($e)),
            Value::Int(IntData::U16($a)) => Value::Int(IntData::U16($e)),
            Value::Int(IntData::U32($a)) => Value::Int(IntData::U32($e)),
            Value::Int(IntData::U64($a)) => Value::Int(IntData::U64($e)),
        }
    };
}

/// Leitura indexada `A(idx…)`.
///
/// Erros: `indiceForaDosLimites`, `indiceInvalido` (zero).
pub fn subsref(v: &Value, idx: &[Index]) -> Result<Value, RuntimeError> {
    if idx.is_empty() {
        return Ok(v.clone());
    }
    Ok(dispatch!(v, a => select(a, idx)?))
}

/// Remoção `A(idx…) = []`.
pub fn subsdelete(v: &mut Value, idx: &[Index]) -> Result<(), RuntimeError> {
    *v = dispatch!(&*v, a => delete(a, idx)?);
    Ok(())
}

/// Atribuição indexada `A(idx…) = rhs`, com crescimento.
///
/// Em caso de erro o alvo fica intacto: as promoções de classe acontecem numa cópia, e a
/// atribuição sem promoção valida tudo antes de escrever.
///
/// Erros: `dimensoesIncompativeis` (número de elementos), `indiceInvalido`, `conversaoInvalida`.
pub fn subsasgn(v: &mut Value, idx: &[Index], rhs: &Value) -> Result<(), RuntimeError> {
    if idx.is_empty() {
        *v = rhs.clone();
        return Ok(());
    }
    // Classe do resultado: a do alvo (vazio adota a do lado direito; complexo promove double).
    let alvo = if v.numel() == 0 { rhs.class() } else { v.class() };
    let promove_vazio = v.numel() == 0 && v.class() != alvo;
    let promove_complexo = rhs.is_complex() && matches!(v, Value::Double(_));
    if promove_vazio || promove_complexo {
        let mut w = if promove_vazio { v.cast(alvo)? } else { v.clone() };
        if rhs.is_complex() && matches!(w, Value::Double(_)) {
            let z: Vec<Complex64> = w.to_complex();
            w = Value::Complex(NdArray::from_shape_vec(w.dims(), z).expect("forma"));
        }
        atribui_mesma_classe(&mut w, idx, rhs, alvo)?;
        *v = w;
        return Ok(());
    }
    atribui_mesma_classe(v, idx, rhs, alvo)
}

/// Atribuição com o alvo já na classe final.
fn atribui_mesma_classe(
    v: &mut Value,
    idx: &[Index],
    rhs: &Value,
    alvo: crate::Class,
) -> Result<(), RuntimeError> {
    let r = if v.is_complex() {
        Value::Complex(NdArray::from_shape_vec(rhs.dims(), rhs.to_complex()).expect("forma"))
    } else {
        rhs.cast(alvo)?
    };
    match (v, &r) {
        (Value::Double(a), Value::Double(b)) => assign(a, idx, b, 0.0),
        (Value::Complex(a), Value::Complex(b)) => assign(a, idx, b, Complex64::new(0.0, 0.0)),
        (Value::Single(a), Value::Single(b)) => assign(a, idx, b, 0.0),
        (Value::Logical(a), Value::Logical(b)) => assign(a, idx, b, false),
        (Value::Char(a), Value::Char(b)) => assign(a, idx, b, '\0'),
        (Value::Int(IntData::I8(a)), Value::Int(IntData::I8(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::I16(a)), Value::Int(IntData::I16(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::I32(a)), Value::Int(IntData::I32(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::I64(a)), Value::Int(IntData::I64(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::U8(a)), Value::Int(IntData::U8(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::U16(a)), Value::Int(IntData::U16(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::U32(a)), Value::Int(IntData::U32(b))) => assign(a, idx, b, 0),
        (Value::Int(IntData::U64(a)), Value::Int(IntData::U64(b))) => assign(a, idx, b, 0),
        _ => unreachable!("lado direito convertido para a classe do alvo"),
    }
}
