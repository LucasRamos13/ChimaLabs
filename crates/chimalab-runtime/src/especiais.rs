//! Funções especiais (F042) e de inteiros na linguagem: gama, erro, zeta, integral exponencial,
//! beta, Bessel, máximo divisor comum, primos e combinações.

use chimalab_array::{Complex64, NdArray};
use chimalab_lang::sintaxe::Node;
use chimalab_math::special;

use crate::avaliador::{ErroExec, Interpretador, Obj, Saida, de_runtime, erro_em};
use crate::ops::{broadcast_index, simplify};
use crate::{Class, Value, exibicao, verifica_tamanho};

/// Nomes atendidos por este módulo.
pub const NOMES: &[&str] = &[
    "gamma",
    "gammaln",
    "psi",
    "factorial",
    "erf",
    "erfc",
    "erfcx",
    "erfinv",
    "erfcinv",
    "expint",
    "zeta",
    "beta",
    "betaln",
    "besselj",
    "bessely",
    "besseli",
    "besselk",
    "gcd",
    "lcm",
    "primes",
    "isprime",
    "factor",
    "nchoosek",
];

/// `true` se `nome` é deste módulo.
pub fn atende(nome: &str) -> bool {
    NOMES.contains(&nome)
}

fn err(m: impl Into<String>, n: &Node) -> ErroExec {
    erro_em("chimalab:execucao:argumentos", m, n)
}

fn valor<'a>(o: &'a Obj, n: &Node) -> Result<&'a Value, ErroExec> {
    match o {
        Obj::Array(v) => Ok(v),
        _ => Err(erro_em(
            "chimalab:execucao:funcaoComoValor",
            "esperava um array, veio uma função",
            n,
        )),
    }
}

fn de_special(e: special::SpecialError, n: &Node) -> ErroExec {
    erro_em(e.id(), e.to_string(), n)
}

/// Funções de um argumento real, elemento a elemento (`single` continua `single`).
fn unaria(nome: &str, v: &Value, n: &Node) -> Result<Value, ErroExec> {
    if let Value::Complex(a) = v {
        let f: fn(Complex64) -> Complex64 = match nome {
            "gamma" => special::gamma_complex,
            "gammaln" => special::gammaln_complex,
            "erf" => special::erf_complex,
            "erfc" => special::erfc_complex,
            _ => {
                return Err(erro_em(
                    "chimalab:execucao:classeNaoSuportada",
                    format!("`{nome}` de complexos ainda não é suportado"),
                    n,
                ));
            }
        };
        return Ok(simplify(a.map(|z| f(*z))));
    }
    if matches!(v, Value::Int(_)) {
        return Err(erro_em(
            "chimalab:execucao:classeNaoSuportada",
            format!(
                "`{nome}` não se aplica a {}; converta com double(…)",
                v.class().name()
            ),
            n,
        ));
    }
    let f: fn(f64) -> f64 = match nome {
        "gamma" => special::gamma,
        "gammaln" => special::gammaln,
        "psi" => special::psi,
        "factorial" => special::factorial,
        "erf" => special::erf,
        "erfc" => special::erfc,
        "erfcx" => special::erfcx,
        "erfinv" => special::erfinv,
        "erfcinv" => special::erfcinv,
        "expint" => special::expint,
        _ => special::zeta,
    };
    let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
    let r = Value::doubles(v.dims(), x.into_iter().map(f).collect());
    if matches!(v, Value::Single(_)) {
        r.cast(Class::Single).map_err(|e| de_runtime(e, n))
    } else {
        Ok(r)
    }
}

/// Operação binária real com expansão implícita e erro por elemento.
fn binaria(
    nome: &str,
    a: &Value,
    b: &Value,
    n: &Node,
    f: impl Fn(f64, f64) -> Result<f64, special::SpecialError>,
) -> Result<Value, ErroExec> {
    if a.is_complex() || b.is_complex() || matches!(a, Value::Int(_)) || matches!(b, Value::Int(_)) {
        return Err(erro_em(
            "chimalab:execucao:classeNaoSuportada",
            format!("`{nome}` ainda só aceita double, lógico e caractere"),
            n,
        ));
    }
    let out = chimalab_array::broadcast_shape(a.dims(), b.dims()).map_err(|_| {
        erro_em(
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
    let v = ia
        .iter()
        .zip(&ib)
        .map(|(&i, &j)| f(x[i], y[j]))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| de_special(e, n))?;
    Ok(Value::doubles(&out, v))
}

/// Inteiro não negativo exato (para `isprime`, `factor`).
fn natural(x: f64) -> Option<u64> {
    (x >= 0.0 && x.fract() == 0.0 && x < 18_446_744_073_709_551_616.0).then_some(x as u64)
}

/// Chama a função `nome` (já conferida com [`atende`]).
pub fn chama(
    it: &mut Interpretador,
    nome: &'static str,
    pos: &[Obj],
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let um = |v: Value| Ok(vec![Obj::Array(v)]);
    let binarias = [
        "beta", "betaln", "besselj", "bessely", "besseli", "besselk", "gcd", "lcm", "nchoosek",
    ];
    let k = if binarias.contains(&nome) { 2 } else { 1 };
    if pos.len() != k {
        return Err(err(
            format!("`{nome}` aceita {k} argumento(s), recebeu {}", pos.len()),
            n,
        ));
    }
    match nome {
        "beta" | "betaln" | "besselj" | "bessely" | "besseli" | "besselk" | "gcd" | "lcm" => {
            let (a, b) = (valor(&pos[0], n)?, valor(&pos[1], n)?);
            let r = match nome {
                "beta" => binaria(nome, a, b, n, |x, y| Ok(special::beta(x, y))),
                "betaln" => binaria(nome, a, b, n, |x, y| Ok(special::betaln(x, y))),
                "besselj" => binaria(nome, a, b, n, |x, y| Ok(special::besselj(x, y))),
                "bessely" => binaria(nome, a, b, n, |x, y| Ok(special::bessely(x, y))),
                "besseli" => binaria(nome, a, b, n, |x, y| Ok(special::besseli(x, y, false))),
                "besselk" => binaria(nome, a, b, n, |x, y| Ok(special::besselk(x, y, false))),
                "gcd" => binaria(nome, a, b, n, special::gcd),
                _ => binaria(nome, a, b, n, special::lcm),
            }?;
            um(r)
        }
        "nchoosek" => {
            let x = |o: &Obj, papel: &str| -> Result<f64, ErroExec> {
                let v = valor(o, n)?.to_f64s().map_err(|e| de_runtime(e, n))?;
                match v.as_slice() {
                    [x] => Ok(*x),
                    _ => Err(err(format!("{papel} de `nchoosek` é um escalar"), n)),
                }
            };
            let (r, exato) =
                special::nchoosek(x(&pos[0], "n")?, x(&pos[1], "k")?).map_err(|e| de_special(e, n))?;
            if !exato {
                it.saidas.push(Saida::Aviso {
                    id: "chimalab:special:perdaDePrecisao",
                    mensagem: "o coeficiente binomial passa de 2⁵³; o valor é aproximado".into(),
                });
            }
            um(Value::scalar(r))
        }
        "primes" => {
            let v = valor(&pos[0], n)?.to_f64s().map_err(|e| de_runtime(e, n))?;
            let [lim] = v.as_slice() else {
                return Err(err("o limite de `primes` é um escalar", n));
            };
            // π(n) < 1,26·n/ln n (n > 1): a saída é o que ocupa memória.
            if *lim > 16.0 {
                let estimado = 1.26 * lim / lim.ln();
                verifica_tamanho(&[1, estimado.min(1e30) as usize]).map_err(|e| de_runtime(e, n))?;
            }
            let p: Vec<f64> = special::primes(*lim).into_iter().map(|x| x as f64).collect();
            let k = p.len();
            um(Value::doubles(&[1, k], p))
        }
        "isprime" => {
            let v = valor(&pos[0], n)?;
            let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
            let r: Vec<bool> = x
                .iter()
                .map(|&e| natural(e).is_some_and(special::isprime))
                .collect();
            um(Value::Logical(
                NdArray::from_shape_vec(v.dims(), r).expect("forma"),
            ))
        }
        "factor" => {
            let v = valor(&pos[0], n)?.to_f64s().map_err(|e| de_runtime(e, n))?;
            let Some(m) = (match v.as_slice() {
                [x] => natural(*x).filter(|&m| m >= 1),
                _ => None,
            }) else {
                return Err(err("`factor` recebe um inteiro ≥ 1", n));
            };
            let f: Vec<f64> = special::factor(m).into_iter().map(|x| x as f64).collect();
            let k = f.len();
            um(Value::doubles(&[1, k], f))
        }
        _ => um(unaria(nome, valor(&pos[0], n)?, n)?),
    }
}
