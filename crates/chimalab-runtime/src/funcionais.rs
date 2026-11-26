//! Funções que recebem funções (DT-40): `map`, `integral` (F048), `fzero` e `fminbnd` (F051) e
//! `ode` (F049; nome decidido na resposta 53.A).
//!
//! As rotinas do `chimalab-math` recebem uma closure que chama o interpretador. Um erro dentro
//! da função do usuário (inclusive a interrupção) fica guardado: a rotina recebe NaN, para
//! (`fzero`/`fminbnd`, pela função de saída) ou termina, e o erro sai com a posição da chamada
//! interna.

use std::cell::Cell;
use std::rc::Rc;

use chimalab_array::Complex64;
use chimalab_lang::sintaxe::Node;
use chimalab_math::integrate::{IntegralOptions, integral, integral2};
use chimalab_math::ode::{AbsTol, Method, OdeOptions, OdeStatus, OdeWarning, solve_ivp};
use chimalab_math::optim::{Control, Options, Start, fminbnd, fminsearch, fzero};

use crate::avaliador::{ErroExec, Interpretador, Obj, Saida, de_runtime, erro_em};
use crate::{Value, exibicao, horzcat};

/// Nomes atendidos por este módulo.
pub const NOMES: &[&str] = &[
    "map",
    "integral",
    "fzero",
    "fminbnd",
    "ode",
    "fminsearch",
    "integral2",
];

/// `true` se `nome` é deste módulo.
pub fn atende(nome: &str) -> bool {
    NOMES.contains(&nome)
}

fn err(m: impl Into<String>, n: &Node) -> ErroExec {
    erro_em("chimalab:execucao:argumentos", m, n)
}

fn funcao<'a>(o: &'a Obj, nome: &str, n: &Node) -> Result<&'a Obj, ErroExec> {
    match o {
        Obj::Array(_) => Err(err(
            format!("o primeiro argumento de `{nome}` é uma função (por exemplo, x -> x^2)"),
            n,
        )),
        f => Ok(f),
    }
}

fn real(o: &Obj, papel: &str, n: &Node) -> Result<Vec<f64>, ErroExec> {
    match o {
        Obj::Array(v) => v.to_f64s().map_err(|e| de_runtime(e, n)),
        _ => Err(err(format!("{papel} precisa ser numérico"), n)),
    }
}

fn escalar(o: &Obj, papel: &str, n: &Node) -> Result<f64, ErroExec> {
    let x = real(o, papel, n)?;
    if x.len() != 1 {
        return Err(err(format!("{papel} precisa ser um escalar"), n));
    }
    Ok(x[0])
}

/// Aplica `f` a um escalar e exige um escalar de volta.
fn aplica(it: &mut Interpretador, f: &Obj, x: Value, n: &Node) -> Result<Value, ErroExec> {
    let mut r = it.chama(f, vec![Obj::Array(x)], Vec::new(), 1, n)?;
    match r.pop() {
        Some(Obj::Array(v)) if v.numel() == 1 => Ok(v),
        Some(Obj::Array(v)) => Err(erro_em(
            "chimalab:execucao:resultadoNaoEscalar",
            format!(
                "a função deve devolver um escalar, devolveu {}",
                exibicao::tamanho(v.dims())
            ),
            n,
        )),
        _ => Err(erro_em(
            "chimalab:execucao:semValor",
            "a função não devolveu valor",
            n,
        )),
    }
}

/// Opções nomeadas aceitas; devolve os valores na ordem de `aceitas`.
fn opcoes<'a>(
    nome: &str,
    nomeados: &'a [(String, Obj)],
    aceitas: &[&str],
    n: &Node,
) -> Result<Vec<Option<&'a Obj>>, ErroExec> {
    for (k, _) in nomeados {
        if !aceitas.contains(&k.as_str()) {
            return Err(err(
                format!(
                    "`{nome}` não aceita o argumento nomeado `{k}` (aceita: {})",
                    aceitas.join(", ")
                ),
                n,
            ));
        }
    }
    Ok(aceitas
        .iter()
        .map(|a| nomeados.iter().find(|(k, _)| k == a).map(|(_, v)| v))
        .collect())
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
    let (min, max) = match nome {
        "map" | "fzero" | "fminsearch" => (2, 2),
        "integral2" => (5, 5),
        _ => (3, 3),
    };
    if nome == "ode" && pos.len() == 3 {
        let f = funcao(&pos[0], nome, n)?.clone();
        return edo(it, &f, &pos, &nomeados, n);
    }
    if pos.len() < min || pos.len() > max {
        return Err(err(
            format!("`{nome}` aceita {min} argumento(s), recebeu {}", pos.len()),
            n,
        ));
    }
    let f = funcao(&pos[0], nome, n)?.clone();
    match nome {
        "fminsearch" => {
            let o = opcoes(nome, &nomeados, &["tol_x", "tol_fun", "max_iter"], n)?;
            let Obj::Array(x0v) = &pos[1] else {
                return Err(err("o ponto inicial de `fminsearch` é numérico", n));
            };
            let dims = x0v.dims().to_vec();
            let x0 = real(&pos[1], "o ponto inicial", n)?;
            let mut opts = Options::default();
            if let Some(v) = o[0] {
                opts.tol_x = Some(escalar(v, "`tol_x`", n)?);
            }
            if let Some(v) = o[1] {
                opts.tol_fun = Some(escalar(v, "`tol_fun`", n)?);
            }
            if let Some(v) = o[2] {
                let k = escalar(v, "`max_iter`", n)?;
                if !(k >= 1.0 && k.fract() == 0.0 && k.is_finite()) {
                    return Err(err("`max_iter` é um inteiro ≥ 1", n));
                }
                opts.max_iter = Some(k as usize);
            }
            let parou = Rc::new(Cell::new(false));
            let p2 = parou.clone();
            opts.output = Some(Box::new(move |_| {
                if p2.get() {
                    Control::Stop
                } else {
                    Control::Continue
                }
            }));
            let mut erro: Option<ErroExec> = None;
            let r = fminsearch(
                |x: &[f64]| {
                    if erro.is_some() {
                        return f64::NAN;
                    }
                    let xv = Value::doubles(&dims, x.to_vec());
                    let r = aplica(it, &f, xv, n).and_then(|v| {
                        v.to_f64s().map(|x| x[0]).map_err(|_| {
                            erro_em(
                                "chimalab:execucao:resultadoNaoEscalar",
                                "a função deve devolver um escalar real",
                                n,
                            )
                        })
                    });
                    match r {
                        Ok(y) => y,
                        Err(e) => {
                            erro = Some(e);
                            parou.set(true);
                            f64::NAN
                        }
                    }
                },
                &x0,
                &mut opts,
            );
            drop(opts);
            if let Some(e) = erro {
                return Err(e);
            }
            let r = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            let mut out = vec![Obj::Array(Value::doubles(&dims, r.x))];
            if nargout >= 2 {
                out.push(Obj::Array(Value::scalar(r.fval)));
            }
            Ok(out)
        }
        "integral2" => {
            // integral2(f, xa, xb, ya, yb) com limites numéricos; f(x, y) escalar.
            let o = opcoes(nome, &nomeados, &["abs_tol", "rel_tol"], n)?;
            let lim = |k: usize, papel: &str| -> Result<f64, ErroExec> {
                match &pos[k] {
                    Obj::Array(_) => escalar(&pos[k], papel, n),
                    _ => Err(erro_em(
                        "chimalab:execucao:naoSuportado",
                        "limites em y que dependem de x ainda não são suportados em `integral2`",
                        n,
                    )),
                }
            };
            let (xa, xb) = (lim(1, "xa")?, lim(2, "xb")?);
            let (ya, yb) = (lim(3, "ya")?, lim(4, "yb")?);
            let mut opts = IntegralOptions::default();
            if let Some(v) = o[0] {
                opts.abs_tol = escalar(v, "`abs_tol`", n)?;
            }
            if let Some(v) = o[1] {
                opts.rel_tol = escalar(v, "`rel_tol`", n)?;
            }
            let mut erro: Option<ErroExec> = None;
            let r = integral2(
                |x, y| {
                    if erro.is_some() {
                        return f64::NAN;
                    }
                    let r = it
                        .chama(
                            &f,
                            vec![Obj::Array(Value::scalar(x)), Obj::Array(Value::scalar(y))],
                            Vec::new(),
                            1,
                            n,
                        )
                        .and_then(|mut v| match v.pop() {
                            Some(Obj::Array(d)) if d.numel() == 1 => {
                                d.to_f64s().map(|x| x[0]).map_err(|e| de_runtime(e, n))
                            }
                            _ => Err(erro_em(
                                "chimalab:execucao:resultadoNaoEscalar",
                                "f(x, y) deve devolver um escalar real",
                                n,
                            )),
                        });
                    match r {
                        Ok(v) => v,
                        Err(e) => {
                            erro = Some(e);
                            f64::NAN
                        }
                    }
                },
                xa,
                xb,
                move |_| ya,
                move |_| yb,
                &opts,
            );
            if let Some(e) = erro {
                return Err(e);
            }
            let r = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            if let Some(w) = r.warning {
                it.saidas.push(Saida::Aviso {
                    id: w.id(),
                    mensagem: format!(
                        "a integral pode não ter a precisão pedida (erro estimado {})",
                        exibicao::numero(r.error_bound)
                    ),
                });
            }
            Ok(vec![Obj::Array(Value::scalar(r.value))])
        }
        "map" => {
            opcoes(nome, &nomeados, &[], n)?;
            let Obj::Array(a) = &pos[1] else {
                return Err(err("o segundo argumento de `map` é um array", n));
            };
            let k = a.numel();
            if k == 0 {
                return Ok(vec![Obj::Array(Value::doubles(a.dims(), Vec::new()))]);
            }
            let mut res = Vec::with_capacity(k);
            for i in 0..k {
                let x = crate::subsref(a, &[crate::Index::Scalar(i + 1)]).map_err(|e| de_runtime(e, n))?;
                res.push(aplica(it, &f, x, n)?);
            }
            let linha = horzcat(&res).map_err(|e| de_runtime(e, n))?;
            Ok(vec![Obj::Array(remodela(linha, a.dims()))])
        }
        "integral" => {
            let o = opcoes(nome, &nomeados, &["abs_tol", "rel_tol", "waypoints"], n)?;
            let a = escalar(&pos[1], "o limite inferior", n)?;
            let b = escalar(&pos[2], "o limite superior", n)?;
            let mut opts = IntegralOptions::default();
            if let Some(v) = o[0] {
                opts.abs_tol = escalar(v, "`abs_tol`", n)?;
            }
            if let Some(v) = o[1] {
                opts.rel_tol = escalar(v, "`rel_tol`", n)?;
            }
            if let Some(v) = o[2] {
                opts.waypoints = real(v, "`waypoints`", n)?;
            }
            let mut erro: Option<ErroExec> = None;
            let r = integral(
                |x| {
                    if erro.is_some() {
                        return Complex64::new(f64::NAN, 0.0);
                    }
                    match aplica(it, &f, Value::scalar(x), n) {
                        Ok(v) => v.to_complex()[0],
                        Err(e) => {
                            erro = Some(e);
                            Complex64::new(f64::NAN, 0.0)
                        }
                    }
                },
                a,
                b,
                &opts,
            );
            if let Some(e) = erro {
                return Err(e);
            }
            let r = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            if let Some(w) = r.warning {
                it.saidas.push(Saida::Aviso {
                    id: w.id(),
                    mensagem: format!(
                        "a integral pode não ter a precisão pedida (erro estimado {})",
                        exibicao::numero(r.error_bound)
                    ),
                });
            }
            let v = if r.value.im == 0.0 {
                Value::scalar(r.value.re)
            } else {
                crate::ops::simplify(chimalab_array::NdArray::filled_nd(&[1, 1], r.value))
            };
            let mut out = vec![Obj::Array(v)];
            if nargout >= 2 {
                out.push(Obj::Array(Value::scalar(r.error_bound)));
            }
            Ok(out)
        }
        _ => {
            // fzero(f, x0 | [a b]; tol_x=…) e fminbnd(f, a, b; tol_x=…).
            let o = opcoes(nome, &nomeados, &["tol_x"], n)?;
            let mut opts = Options::default();
            if let Some(v) = o[0] {
                opts.tol_x = Some(escalar(v, "`tol_x`", n)?);
            }
            let parou = Rc::new(Cell::new(false));
            let p2 = parou.clone();
            opts.output = Some(Box::new(move |_| {
                if p2.get() {
                    Control::Stop
                } else {
                    Control::Continue
                }
            }));
            let mut erro: Option<ErroExec> = None;
            let mut g = |x: f64| -> f64 {
                if erro.is_some() {
                    return f64::NAN;
                }
                let r = aplica(it, &f, Value::scalar(x), n).and_then(|v| {
                    v.to_f64s().map(|x| x[0]).map_err(|_| {
                        erro_em(
                            "chimalab:execucao:resultadoNaoEscalar",
                            "a função deve devolver um escalar real",
                            n,
                        )
                    })
                });
                match r {
                    Ok(y) => y,
                    Err(e) => {
                        erro = Some(e);
                        parou.set(true);
                        f64::NAN
                    }
                }
            };
            let r = if nome == "fzero" {
                let x0 = real(&pos[1], "o ponto inicial", n)?;
                let inicio = match x0.as_slice() {
                    [x] => Start::Guess(*x),
                    [a, b] => Start::Interval(*a, *b),
                    _ => return Err(err("o início de `fzero` é um ponto x0 ou um intervalo [a, b]", n)),
                };
                fzero(&mut g, inicio, &mut opts)
            } else {
                let a = escalar(&pos[1], "o limite inferior", n)?;
                let b = escalar(&pos[2], "o limite superior", n)?;
                fminbnd(&mut g, a, b, &mut opts)
            };
            drop(opts);
            if let Some(e) = erro {
                return Err(e);
            }
            let r = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
            let mut out = vec![Obj::Array(Value::scalar(r.x))];
            if nargout >= 2 {
                out.push(Obj::Array(Value::scalar(r.fval)));
            }
            Ok(out)
        }
    }
}

/// `t, y = ode(f, tspan, y0; method=…, rel_tol=…, abs_tol=…, max_step=…)`: y' = f(t, y).
fn edo(
    it: &mut Interpretador,
    f: &Obj,
    pos: &[Obj],
    nomeados: &[(String, Obj)],
    n: &Node,
) -> Result<Vec<Obj>, ErroExec> {
    let o = opcoes("ode", nomeados, &["method", "rel_tol", "abs_tol", "max_step"], n)?;
    let tspan = real(&pos[1], "o intervalo de tempo", n)?;
    let y0 = real(&pos[2], "a condição inicial", n)?;
    let mut opts = OdeOptions::default();
    if let Some(Obj::Array(Value::Char(m))) = o[0] {
        let m: String = m.as_slice().iter().collect();
        opts.method = match m.as_str() {
            "dopri5" => Method::Dopri5,
            "bs23" => Method::Bs23,
            "ndf" => Method::Ndf,
            "bdf" => Method::Bdf,
            "radau" => Method::Radau,
            "rosenbrock" => Method::Rosenbrock,
            _ => {
                return Err(err(
                    format!("método `{m}` desconhecido (dopri5, bs23, ndf, bdf, radau, rosenbrock)"),
                    n,
                ));
            }
        };
    } else if o[0].is_some() {
        return Err(err("o método é um texto, por exemplo method=\"radau\"", n));
    }
    if let Some(v) = o[1] {
        opts.rel_tol = Some(escalar(v, "`rel_tol`", n)?);
    }
    if let Some(v) = o[2] {
        let a = real(v, "`abs_tol`", n)?;
        opts.abs_tol = Some(if a.len() == 1 {
            AbsTol::Scalar(a[0])
        } else {
            AbsTol::Vector(a)
        });
    }
    if let Some(v) = o[3] {
        opts.max_step = Some(escalar(v, "`max_step`", n)?);
    }
    let dim = y0.len();
    let mut erro: Option<ErroExec> = None;
    let r = solve_ivp(
        |t, y, dy| {
            if erro.is_some() {
                dy.fill(f64::NAN);
                return;
            }
            let yv = Value::doubles(&[y.len(), 1], y.to_vec());
            let r = it
                .chama(
                    f,
                    vec![Obj::Array(Value::scalar(t)), Obj::Array(yv)],
                    Vec::new(),
                    1,
                    n,
                )
                .and_then(|mut v| match v.pop() {
                    Some(Obj::Array(d)) => d.to_f64s().map_err(|e| de_runtime(e, n)),
                    _ => Err(erro_em(
                        "chimalab:execucao:semValor",
                        "f(t, y) não devolveu valor",
                        n,
                    )),
                })
                .and_then(|d| {
                    if d.len() == dim {
                        Ok(d)
                    } else {
                        Err(erro_em(
                            "chimalab:execucao:derivadaComTamanhoErrado",
                            format!("f(t, y) devolveu {} valor(es); o estado tem {dim}", d.len()),
                            n,
                        ))
                    }
                });
            match r {
                Ok(d) => dy.copy_from_slice(&d),
                Err(e) => {
                    erro = Some(e);
                    dy.fill(f64::NAN);
                }
            }
        },
        &tspan,
        &y0,
        &mut opts,
    );
    if let Some(e) = erro {
        return Err(e);
    }
    let s = r.map_err(|e| erro_em(e.id(), e.to_string(), n))?;
    if let Some(id) = s.status.warning_id() {
        let t_fim = s.t.last().copied().unwrap_or(f64::NAN);
        let motivo = match s.status {
            OdeStatus::StepTooSmall => "o passo ficou pequeno demais (singularidade provável)",
            OdeStatus::NonFinite => "f(t, y) deu valor não finito",
            _ => "o limite de passos foi atingido",
        };
        it.saidas.push(Saida::Aviso {
            id,
            mensagem: format!("{motivo}; a solução vai só até t = {}", exibicao::numero(t_fim)),
        });
    }
    for w in &s.warnings {
        let (id, mensagem) = match w {
            OdeWarning::RelTolRaised { requested, used } => (
                "chimalab:ode:toleranciaElevada",
                format!(
                    "rel_tol = {} é pequena demais; usada {}",
                    exibicao::numero(*requested),
                    exibicao::numero(*used)
                ),
            ),
            OdeWarning::StiffnessSuspected { t } => (
                "chimalab:ode:rigidezSuspeita",
                format!(
                    "o problema parece rígido a partir de t = {}; um método implícito (method=\"ndf\", \"radau\" ou \"rosenbrock\") deve ser mais eficiente",
                    exibicao::numero(*t)
                ),
            ),
        };
        it.saidas.push(Saida::Aviso { id, mensagem });
    }
    let k = s.t.len();
    let mut ys = Vec::with_capacity(k * dim);
    for j in 0..dim {
        ys.extend(s.y.iter().map(|y| y[j]));
    }
    Ok(vec![
        Obj::Array(Value::doubles(&[k, 1], s.t)),
        Obj::Array(Value::doubles(&[k, dim], ys)),
    ])
}

/// Mesmo conteúdo com outra forma (os resultados de `map` voltam à forma da entrada).
fn remodela(v: Value, dims: &[usize]) -> Value {
    match v {
        Value::Double(a) => Value::Double(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Complex(a) => Value::Complex(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Single(a) => Value::Single(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Logical(a) => Value::Logical(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Char(a) => Value::Char(a.reshape(dims).expect("mesmo número de elementos")),
        Value::Int(d) => Value::Int(crate::IntData::from_i128(d.class(), dims, d.to_i128())),
    }
}
