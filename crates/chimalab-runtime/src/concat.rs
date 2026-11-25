//! Concatenação de valores (`[A B]`, `[A; B]` e ao longo de qualquer dimensão) — DT-38.
//!
//! Regras de classe: as da aritmética (inteiro domina, inteiros de classes diferentes dão erro,
//! `single` domina `double`, lógico vira `double` quando misturado), com duas exceções:
//! operandos todos lógicos ou todos caracteres mantêm a classe, e caractere com qualquer outra
//! classe é erro (sem a conversão silenciosa de números em caracteres). Vazios 0×0 (`[]`) são
//! ignorados; outros vazios participam da verificação de dimensões.

use chimalab_array::NdArray;

use crate::{Class, IntData, RuntimeError, Value};

const OP: &str = "concatenação";

/// Concatena ao longo da dimensão `dim` (1-based: 1 = linhas, 2 = colunas).
///
/// Erros: `classesIncompativeis` (caractere com outra classe, inteiros de classes diferentes,
/// inteiro ou `single` com complexo), `dimensoesIncompativeis` (tamanhos diferentes fora de `dim`),
/// `indiceInvalido` (`dim` = 0).
pub fn concat(dim: usize, parts: &[Value]) -> Result<Value, RuntimeError> {
    if dim == 0 {
        return Err(RuntimeError::InvalidIndex(
            "dimensão de concatenação começa em 1".into(),
        ));
    }
    let uteis: Vec<&Value> = parts.iter().filter(|v| v.dims() != [0, 0]).collect();
    let Some(primeiro) = uteis.first() else {
        // Só vazios: o primeiro (mantém a classe) ou o vazio double.
        return Ok(parts
            .first()
            .cloned()
            .unwrap_or_else(|| Value::doubles(&[0, 0], vec![])));
    };
    let alvo = classe(&uteis)?;
    let complexo = alvo == Class::Double && uteis.iter().any(|v| v.is_complex());
    // Dimensões completadas com 1 até cobrir `dim`; todas iguais fora de `dim`.
    let nd = uteis.iter().map(|v| v.dims().len()).max().unwrap_or(2).max(dim);
    let completa = |v: &Value| {
        let mut d = v.dims().to_vec();
        d.resize(nd, 1);
        d
    };
    let base = completa(primeiro);
    for v in &uteis[1..] {
        let d = completa(v);
        if (0..nd).any(|k| k != dim - 1 && d[k] != base[k]) {
            return Err(RuntimeError::IncompatibleShapes {
                operation: OP,
                left: primeiro.dims().to_vec(),
                right: v.dims().to_vec(),
            });
        }
    }
    let mut dims_out = base.clone();
    dims_out[dim - 1] = uteis.iter().map(|v| completa(v)[dim - 1]).sum();
    let blocos: Vec<(Vec<usize>, &Value)> = uteis.iter().map(|v| (completa(v), *v)).collect();
    let k = dim - 1;
    Ok(match alvo {
        Class::Double if complexo => {
            let v = junta(&blocos, blocos.iter().map(|(_, v)| v.to_complex()).collect(), k);
            Value::Complex(NdArray::from_shape_vec(&dims_out, v).expect("forma"))
        }
        Class::Double => {
            let v: Vec<f64> = junta(
                &blocos,
                blocos
                    .iter()
                    .map(|(_, v)| v.to_f64s())
                    .collect::<Result<_, _>>()?,
                k,
            );
            Value::Double(NdArray::from_shape_vec(&dims_out, v).expect("forma"))
        }
        Class::Single => {
            let datas = blocos
                .iter()
                .map(|(_, v)| match v.cast(Class::Single)? {
                    Value::Single(a) => Ok(a.as_slice().to_vec()),
                    _ => unreachable!("cast para single"),
                })
                .collect::<Result<Vec<Vec<f32>>, RuntimeError>>()?;
            Value::Single(NdArray::from_shape_vec(&dims_out, junta(&blocos, datas, k)).expect("forma"))
        }
        Class::Int(c) => {
            let datas = blocos
                .iter()
                .map(|(_, v)| match v.cast(Class::Int(c))? {
                    Value::Int(d) => Ok(d.to_i128()),
                    _ => unreachable!("cast para inteiro"),
                })
                .collect::<Result<Vec<Vec<i128>>, RuntimeError>>()?;
            Value::Int(IntData::from_i128(c, &dims_out, junta(&blocos, datas, k)))
        }
        Class::Logical => {
            let datas = blocos
                .iter()
                .map(|(_, v)| match v {
                    Value::Logical(a) => a.as_slice().to_vec(),
                    _ => unreachable!("todos lógicos"),
                })
                .collect();
            Value::Logical(NdArray::from_shape_vec(&dims_out, junta(&blocos, datas, k)).expect("forma"))
        }
        Class::Char => {
            let datas = blocos
                .iter()
                .map(|(_, v)| match v {
                    Value::Char(a) => a.as_slice().to_vec(),
                    _ => unreachable!("todos caracteres"),
                })
                .collect();
            Value::Char(NdArray::from_shape_vec(&dims_out, junta(&blocos, datas, k)).expect("forma"))
        }
    })
}

/// `[A B …]`: concatenação horizontal (dimensão 2).
pub fn horzcat(parts: &[Value]) -> Result<Value, RuntimeError> {
    concat(2, parts)
}

/// `[A; B; …]`: concatenação vertical (dimensão 1).
pub fn vertcat(parts: &[Value]) -> Result<Value, RuntimeError> {
    concat(1, parts)
}

fn classe(uteis: &[&Value]) -> Result<Class, RuntimeError> {
    let erro = |a: &Value, b: &Value| RuntimeError::IncompatibleClasses {
        operation: OP,
        left: a.class().name().into(),
        right: b.class().name().into(),
    };
    let tem_char = uteis.iter().any(|v| v.class() == Class::Char);
    if tem_char {
        return match uteis.iter().find(|v| v.class() != Class::Char) {
            Some(outro) => Err(erro(
                uteis.iter().find(|v| v.class() == Class::Char).expect("existe"),
                outro,
            )),
            None => Ok(Class::Char),
        };
    }
    if uteis.iter().all(|v| v.class() == Class::Logical) {
        return Ok(Class::Logical);
    }
    let complexo = uteis.iter().any(|v| v.is_complex());
    let mut alvo = Class::Double;
    let mut quem = uteis[0];
    for v in uteis {
        let c = v.class();
        match (alvo, c) {
            (Class::Int(x), Class::Int(y)) if x != y => return Err(erro(quem, v)),
            (Class::Int(_), _) => {}
            (_, Class::Int(_)) => {
                alvo = c;
                quem = v;
            }
            (_, Class::Single) => {
                alvo = Class::Single;
                quem = v;
            }
            _ => {}
        }
    }
    if complexo && alvo != Class::Double {
        let z = uteis.iter().find(|v| v.is_complex()).expect("existe");
        return Err(erro(quem, z));
    }
    Ok(alvo)
}

/// Junta os dados (column-major) ao longo do eixo `k` (0-based): para cada bloco externo, copia
/// em ordem o pedaço contíguo de cada operando.
fn junta<T: Clone>(blocos: &[(Vec<usize>, &Value)], datas: Vec<Vec<T>>, k: usize) -> Vec<T> {
    let inner: usize = blocos[0].0[..k].iter().product();
    let outer: usize = blocos[0].0[k + 1..].iter().product();
    let total: usize = datas.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(total);
    for o in 0..outer {
        for ((dims, _), d) in blocos.iter().zip(&datas) {
            let bloco = inner * dims[k];
            out.extend_from_slice(&d[o * bloco..(o + 1) * bloco]);
        }
    }
    out
}
