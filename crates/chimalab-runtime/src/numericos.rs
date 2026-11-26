//! Polinômios (F045), FFT e convolução (F046) e interpolação 1-D (F044) na linguagem.
//!
//! Polinômios em ordem decrescente de grau (`p[1]·xⁿ + … + p[end]`), como no `chimalab-math`.

use chimalab_array::NdArray;
use chimalab_lang::sintaxe::Node;
use chimalab_math::integrate::{Spacing, cumtrapz, trapz};
use chimalab_math::interp::{Extrap, Method1, SplineEnd, interp1};
use chimalab_math::poly::{polyder, polyfit, polyint, polyval, polyval_complex, roots};
use chimalab_math::signal::{ConvShape, conv, fft, ifft};

use crate::Value;
use crate::avaliador::{ErroExec, Interpretador, Obj, de_runtime, erro_em};
use crate::ops::{converte_avisos, simplify};

/// Nomes atendidos por este módulo.
pub const NOMES: &[&str] = &[
    "polyval", "roots", "polyfit", "conv", "fft", "ifft", "interp1", "polyder", "polyint", "trapz",
    "cumtrapz",
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

fn reais(o: &Obj, papel: &str, n: &Node) -> Result<Vec<f64>, ErroExec> {
    let v = valor(o, n)?;
    if v.is_complex() {
        return Err(erro_em(
            "chimalab:execucao:classeNaoSuportada",
            format!("{papel} ainda não aceita complexos"),
            n,
        ));
    }
    v.to_f64s().map_err(|e| de_runtime(e, n))
}

fn texto(o: &Obj, papel: &str, n: &Node) -> Result<String, ErroExec> {
    match o {
        Obj::Array(Value::Char(a)) => Ok(a.as_slice().iter().collect()),
        _ => Err(err(format!("{papel} é um texto"), n)),
    }
}

fn opcoes<'a>(
    nome: &str,
    nomeados: &'a [(String, Obj)],
    aceitas: &[&str],
    n: &Node,
) -> Result<Vec<Option<&'a Obj>>, ErroExec> {
    if let Some((k, _)) = nomeados.iter().find(|(k, _)| !aceitas.contains(&k.as_str())) {
        let lista = if aceitas.is_empty() {
            "nenhum".to_string()
        } else {
            aceitas.join(", ")
        };
        return Err(err(
            format!("`{nome}` não aceita o argumento nomeado `{k}` (aceita: {lista})"),
            n,
        ));
    }
    Ok(aceitas
        .iter()
        .map(|a| nomeados.iter().find(|(k, _)| k == a).map(|(_, v)| v))
        .collect())
}

fn vetor(v: &Value) -> bool {
    v.dims().len() == 2 && (v.dims()[0] == 1 || v.dims()[1] == 1)
}

/// Chama a função `nome` (já conferida com [`atende`]).
pub fn chama(
    it: &mut Interpretador,
    nome: &'static str,
    pos: Vec<Obj>,
    nomeados: Vec<(String, Obj)>,
    nargout: usize,
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let um = |v: Value| Ok(vec![Obj::Array(v)]);
    let (min, max) = match nome {
        "roots" | "polyder" => (1, 1),
        "polyint" | "trapz" | "cumtrapz" => (1, 2),
        "polyval" | "conv" => (2, 2),
        "polyfit" | "interp1" => (3, 3),
        _ => (1, 3),
    };
    if pos.len() < min || pos.len() > max {
        let faixa = if min == max {
            min.to_string()
        } else {
            format!("de {min} a {max}")
        };
        return Err(err(
            format!("`{nome}` aceita {faixa} argumento(s), recebeu {}", pos.len()),
            n,
        ));
    }
    match nome {
        "polyval" => {
            opcoes(nome, &nomeados, &[], n)?;
            let p = reais(&pos[0], "o polinômio", n)?;
            let x = valor(&pos[1], n)?;
            if x.is_complex() {
                let y = polyval_complex(&p, &x.to_complex());
                return um(simplify(NdArray::from_shape_vec(x.dims(), y).expect("forma")));
            }
            let xs = x.to_f64s().map_err(|e| de_runtime(e, n))?;
            um(Value::doubles(x.dims(), polyval(&p, &xs)))
        }
        "polyder" | "polyint" => {
            opcoes(nome, &nomeados, &[], n)?;
            let p = reais(&pos[0], "o polinômio", n)?;
            let r = if nome == "polyder" {
                polyder(&p)
            } else {
                let k = match pos.get(1) {
                    Some(o) => match reais(o, "a constante", n)?.as_slice() {
                        [k] => *k,
                        _ => return Err(err("a constante de `polyint` é um escalar", n)),
                    },
                    None => 0.0,
                };
                polyint(&p, k)
            };
            let k = r.len();
            um(Value::doubles(&[1, k], r))
        }
        "trapz" | "cumtrapz" => {
            opcoes(nome, &nomeados, &[], n)?;
            // trapz(y) com espaçamento 1; trapz(x, y) com abscissas (ou escalar: espaçamento).
            let (x, yo) = if pos.len() == 2 {
                (Some(reais(&pos[0], "x", n)?), &pos[1])
            } else {
                (None, &pos[0])
            };
            let y = valor(yo, n)?;
            let dim = y.dims().iter().position(|&d| d != 1).unwrap_or(0);
            let esp = match &x {
                None => Spacing::Unit,
                Some(v) if v.len() == 1 => Spacing::Uniform(v[0]),
                Some(v) => Spacing::Points(v),
            };
            let e = |e: chimalab_math::integrate::IntegrateError| erro_em(e.id(), e.to_string(), n);
            if y.is_complex() {
                let z = y.to_complex();
                let (r, f) = if nome == "trapz" {
                    trapz(&z, y.dims(), esp, dim)
                } else {
                    cumtrapz(&z, y.dims(), esp, dim)
                }
                .map_err(e)?;
                return um(simplify(
                    NdArray::from_shape_vec(&chimalab_array::normalize_shape(&f), r).expect("forma"),
                ));
            }
            let ys = y.to_f64s().map_err(|e| de_runtime(e, n))?;
            let (r, f) = if nome == "trapz" {
                trapz(&ys, y.dims(), esp, dim)
            } else {
                cumtrapz(&ys, y.dims(), esp, dim)
            }
            .map_err(e)?;
            um(Value::doubles(&chimalab_array::normalize_shape(&f), r))
        }
        "roots" => {
            opcoes(nome, &nomeados, &[], n)?;
            let p = reais(&pos[0], "o polinômio", n)?;
            let r = roots(&p).map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            let k = r.len();
            um(simplify(NdArray::from_shape_vec(&[k, 1], r).expect("forma")))
        }
        "polyfit" => {
            opcoes(nome, &nomeados, &[], n)?;
            let x = reais(&pos[0], "x", n)?;
            let y = reais(&pos[1], "y", n)?;
            let g = reais(&pos[2], "o grau", n)?;
            let grau = match g.as_slice() {
                [g] if *g >= 0.0 && g.fract() == 0.0 && g.is_finite() => *g as usize,
                _ => return Err(err("o grau de `polyfit` é um inteiro ≥ 0", n)),
            };
            let f = polyfit(&x, &y, grau).map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            it.avisos(converte_avisos(&f.warnings));
            let k = f.p.len();
            let mut out = vec![Obj::Array(Value::doubles(&[1, k], f.p))];
            if nargout >= 2 {
                out.push(Obj::Array(Value::scalar(f.normr)));
            }
            Ok(out)
        }
        "conv" => {
            let o = opcoes(nome, &nomeados, &["shape"], n)?;
            let forma = match o[0].map(|v| texto(v, "`shape`", n)).transpose()?.as_deref() {
                None | Some("full") => ConvShape::Full,
                Some("same") => ConvShape::Same,
                Some("valid") => ConvShape::Valid,
                Some(s) => {
                    return Err(err(
                        format!("`shape` é \"full\", \"same\" ou \"valid\" (veio \"{s}\")"),
                        n,
                    ));
                }
            };
            let (u, v) = (valor(&pos[0], n)?, valor(&pos[1], n)?);
            if !vetor(u) || !vetor(v) {
                return Err(err(
                    "`conv` recebe dois vetores (para matrizes, conv2 virá depois)",
                    n,
                ));
            }
            let coluna = u.dims()[1] == 1 && u.numel() > 1;
            let forma_saida = |k: usize| if coluna { [k, 1] } else { [1, k] };
            if u.is_complex() || v.is_complex() {
                let c = conv(&u.to_complex(), &v.to_complex(), forma);
                let k = c.len();
                return um(simplify(
                    NdArray::from_shape_vec(&forma_saida(k), c).expect("forma"),
                ));
            }
            let a = u.to_f64s().map_err(|e| de_runtime(e, n))?;
            let b = v.to_f64s().map_err(|e| de_runtime(e, n))?;
            let c = conv(&a, &b, forma);
            let k = c.len();
            um(Value::doubles(&forma_saida(k), c))
        }
        "fft" | "ifft" => {
            opcoes(nome, &nomeados, &[], n)?;
            let x = valor(&pos[0], n)?;
            let tam = match pos.get(1) {
                None => None,
                Some(o) => {
                    let t = reais(o, "o tamanho", n)?;
                    match t.as_slice() {
                        [] => None,
                        [k] if *k >= 1.0 && k.fract() == 0.0 && k.is_finite() => Some(*k as usize),
                        _ => {
                            return Err(err(
                                format!("o tamanho de `{nome}` é um inteiro ≥ 1 (ou [] para o natural)"),
                                n,
                            ));
                        }
                    }
                }
            };
            let dim = match pos.get(2) {
                None => x.dims().iter().position(|&d| d != 1).unwrap_or(0),
                Some(o) => {
                    let d = reais(o, "a dimensão", n)?;
                    match d.as_slice() {
                        [d] if *d >= 1.0 && d.fract() == 0.0 => *d as usize - 1,
                        _ => return Err(err("dimensões começam em 1", n)),
                    }
                }
            };
            if let Some(t) = tam {
                let mut d = x.dims().to_vec();
                if dim < d.len() {
                    d[dim] = t;
                }
                crate::verifica_tamanho(&d).map_err(|e| de_runtime(e, n))?;
            }
            let z = x.to_complex();
            let r = if nome == "fft" {
                fft(&z, x.dims(), tam, dim)
            } else {
                ifft(&z, x.dims(), tam, dim)
            };
            let (y, forma) = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            let forma = chimalab_array::normalize_shape(&forma);
            um(simplify(NdArray::from_shape_vec(&forma, y).expect("forma")))
        }
        _ => {
            // interp1(x, y, xq; method=…, extrap=…)
            let o = opcoes(nome, &nomeados, &["method", "extrap"], n)?;
            let metodo = match o[0].map(|v| texto(v, "`method`", n)).transpose()?.as_deref() {
                None | Some("linear") => Method1::Linear,
                Some("nearest") => Method1::Nearest,
                Some("previous") => Method1::Previous,
                Some("next") => Method1::Next,
                Some("pchip") => Method1::Pchip,
                Some("akima") => Method1::Akima,
                Some("makima") => Method1::Makima,
                Some("spline") => Method1::Spline(SplineEnd::NotAKnot),
                Some(s) => {
                    return Err(err(
                        format!(
                            "método `{s}` desconhecido (linear, nearest, previous, next, pchip, akima, makima, spline)"
                        ),
                        n,
                    ));
                }
            };
            let extrap = match o[1] {
                None => Extrap::Nan,
                Some(Obj::Array(Value::Char(_))) => match texto(o[1].expect("dado"), "`extrap`", n)?.as_str()
                {
                    "nan" => Extrap::Nan,
                    "extrapolate" => Extrap::Extrapolate,
                    "clamp" => Extrap::Clamp,
                    s => {
                        return Err(err(
                            format!(
                                "`extrap` é \"nan\", \"extrapolate\", \"clamp\" ou um número (veio \"{s}\")"
                            ),
                            n,
                        ));
                    }
                },
                Some(v) => {
                    let x = reais(v, "`extrap`", n)?;
                    match x.as_slice() {
                        [c] => Extrap::Value(*c),
                        _ => return Err(err("`extrap` numérico é um escalar", n)),
                    }
                }
            };
            let x = reais(&pos[0], "x", n)?;
            let y = reais(&pos[1], "y", n)?;
            let xq = valor(&pos[2], n)?;
            let q = xq.to_f64s().map_err(|e| de_runtime(e, n))?;
            let r = interp1(&x, &y, &q, metodo, extrap).map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            um(Value::doubles(xq.dims(), r))
        }
    }
}
