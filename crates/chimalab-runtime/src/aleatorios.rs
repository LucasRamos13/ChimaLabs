//! Números aleatórios (F053) na linguagem: `rand`, `randn`, `randi`, `randperm` e `seed`.
//!
//! Cada interpretador tem o seu gerador, que começa com o algoritmo padrão (Philox4x32-10) e
//! semente 0: uma sessão nova repete os mesmos números, o que torna os resultados reprodutíveis.
//! `seed(n)` recomeça a sequência.

use chimalab_lang::sintaxe::Node;
use chimalab_math::random::{Algorithm, Rng};

use crate::avaliador::{ErroExec, Interpretador, Obj, de_runtime, erro_em};
use crate::{Value, verifica_tamanho};

/// Nomes atendidos por este módulo.
pub const NOMES: &[&str] = &["rand", "randn", "randi", "randperm", "seed"];

/// `true` se `nome` é deste módulo.
pub fn atende(nome: &str) -> bool {
    NOMES.contains(&nome)
}

fn err(m: impl Into<String>, n: &Node) -> ErroExec {
    erro_em("chimalab:execucao:argumentos", m, n)
}

fn escalar(o: &Obj, papel: &str, n: &Node) -> Result<f64, ErroExec> {
    let x = match o {
        Obj::Array(v) => v.to_f64s().map_err(|e| de_runtime(e, n))?,
        _ => return Err(err(format!("{papel} precisa ser numérico"), n)),
    };
    match x.as_slice() {
        [x] => Ok(*x),
        _ => Err(err(format!("{papel} precisa ser um escalar"), n)),
    }
}

fn inteiro(o: &Obj, papel: &str, n: &Node) -> Result<usize, ErroExec> {
    let x = escalar(o, papel, n)?;
    if !(x.is_finite() && x >= 0.0 && x.fract() == 0.0) {
        return Err(err(format!("{papel} precisa ser um inteiro ≥ 0"), n));
    }
    Ok(x as usize)
}

/// Forma pedida: nenhum argumento (1×1), um (n×n) ou vários.
fn forma(nome: &str, pos: &[Obj], n: &Node) -> Result<Vec<usize>, ErroExec> {
    let d = pos
        .iter()
        .map(|o| inteiro(o, &format!("cada tamanho de `{nome}`"), n))
        .collect::<Result<Vec<_>, _>>()?;
    let d = match d.len() {
        1 => vec![d[0], d[0]],
        _ => d,
    };
    verifica_tamanho(&d).map_err(|e| de_runtime(e, n))?;
    Ok(chimalab_array::normalize_shape(&d))
}

/// Chama a função `nome` (já conferida com [`atende`]).
pub fn chama(
    it: &mut Interpretador,
    nome: &'static str,
    pos: &[Obj],
    nomeados: &[(String, Obj)],
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let aceitas: &[&str] = if nome == "seed" { &["algorithm"] } else { &[] };
    if let Some((k, _)) = nomeados.iter().find(|(k, _)| !aceitas.contains(&k.as_str())) {
        return Err(err(format!("`{nome}` não aceita o argumento nomeado `{k}`"), n));
    }
    let um = |v: Value| Ok(vec![Obj::Array(v)]);
    match nome {
        "seed" => {
            if pos.len() != 1 {
                return Err(err("`seed` recebe a semente: seed(42)", n));
            }
            let s = escalar(&pos[0], "a semente", n)?;
            if !(s.is_finite() && s >= 0.0 && s.fract() == 0.0 && s < 18_446_744_073_709_551_616.0) {
                return Err(err("a semente é um inteiro ≥ 0", n));
            }
            let alg = match nomeados.first() {
                Some((_, Obj::Array(Value::Char(a)))) => {
                    let t: String = a.as_slice().iter().collect();
                    Algorithm::from_name(&t).map_err(|e| erro_em(e.id(), format!("{e}: {t}"), n))?
                }
                Some(_) => {
                    return Err(err(
                        "o algoritmo é um texto, por exemplo algorithm=\"twister\"",
                        n,
                    ));
                }
                None => it.gerador().algorithm(),
            };
            *it.gerador() = Rng::new(alg, s as u64);
            Ok(Vec::new())
        }
        "rand" | "randn" => {
            let d = forma(nome, pos, n)?;
            let k: usize = d.iter().product();
            let g = it.gerador();
            let x = if nome == "rand" {
                g.uniforms(k)
            } else {
                g.normals(k)
            };
            um(Value::doubles(&d, x))
        }
        "randi" => {
            let Some(primeiro) = pos.first() else {
                return Err(err(
                    "`randi` precisa do limite: randi(10) ou randi([lo, hi], …)",
                    n,
                ));
            };
            let lim = match primeiro {
                Obj::Array(v) => v.to_f64s().map_err(|e| de_runtime(e, n))?,
                _ => return Err(err("o limite de `randi` é numérico", n)),
            };
            let (lo, hi) = match lim.as_slice() {
                [h] => (1.0, *h),
                [l, h] => (*l, *h),
                _ => return Err(err("o limite de `randi` é hi ou [lo, hi]", n)),
            };
            let ok = |x: f64| x.is_finite() && x.fract() == 0.0 && x.abs() < 9.0e15;
            if !ok(lo) || !ok(hi) || lo > hi {
                return Err(err("o intervalo de `randi` são inteiros com lo ≤ hi", n));
            }
            let d = forma(nome, &pos[1..], n)?;
            let k: usize = d.iter().product();
            let g = it.gerador();
            let x = (0..k)
                .map(|_| g.integer(lo as i64, hi as i64).map(|v| v as f64))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            um(Value::doubles(&d, x))
        }
        _ => {
            // randperm(n), randperm(n, k)
            if pos.is_empty() || pos.len() > 2 {
                return Err(err("`randperm` recebe n ou (n, k)", n));
            }
            let m = inteiro(&pos[0], "n", n)?;
            verifica_tamanho(&[1, m]).map_err(|e| de_runtime(e, n))?;
            let g = it.gerador();
            let p = match pos.get(1) {
                None => g.permutation(m),
                Some(o) => {
                    let k = inteiro(o, "k", n)?;
                    g.sample(m, k).map_err(|e| erro_em(e.id(), e.to_string(), n))?
                }
            };
            let k = p.len();
            um(Value::doubles(
                &[1, k],
                p.into_iter().map(|i| (i + 1) as f64).collect(),
            ))
        }
    }
}
