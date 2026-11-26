//! Exibição de valores no console e em f-strings (DT-40).
//!
//! - Números com até 10 algarismos significativos (7 em `single`), sem zeros à direita;
//!   inteiros exatos (< 1e15) sem casa decimal; os demais em notação científica quando o
//!   expoente sai de [−5, algarismos).
//! - Escalares na mesma linha do nome; matrizes abaixo, com o tamanho (e a classe, se não for
//!   `double`) no cabeçalho e colunas alinhadas à direita.

use crate::Value;
use crate::avaliador::Obj;

/// `r×c×…`.
pub fn tamanho(dims: &[usize]) -> String {
    dims.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("×")
}

/// Número com `sig` algarismos significativos, sem zeros à direita.
fn com_algarismos(x: f64, sig: usize) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf".into() } else { "-Inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    if x.fract() == 0.0 && x.abs() < 1e15 {
        return format!("{x:.0}");
    }
    // Arredonda para `sig` algarismos e decide a notação pelo expoente já arredondado.
    let e = format!("{:.*e}", sig - 1, x);
    let (mant, exp) = e.split_once('e').expect("formato científico");
    let exp: i32 = exp.parse().expect("expoente");
    let (sinal, digitos) = match mant.strip_prefix('-') {
        Some(m) => ("-", m.replace('.', "")),
        None => ("", mant.replace('.', "")),
    };
    let digitos = digitos.trim_end_matches('0');
    let digitos = if digitos.is_empty() { "0" } else { digitos };
    // Fixa só quando a parte inteira cabe nos algarismos exibidos: 12345678901,5 com 10
    // algarismos seria "12345678900", que parece exato e erra as unidades.
    // Um não inteiro nunca aparece sem casa decimal: se o arredondamento comeu as casas
    // (0,9999999999999998 → "1"), mostra todos os algarismos ("1.000000000"); se nem assim
    // sobra casa decimal (1234567890,5), vai para a notação científica.
    let sem_decimais = exp >= 0 && digitos.len() <= exp as usize + 1;
    let digitos_cheios;
    let digitos = if sem_decimais && exp < sig as i32 - 1 {
        digitos_cheios = format!("{digitos}{}", "0".repeat(sig - digitos.len()));
        digitos_cheios.as_str()
    } else {
        digitos
    };
    let sem_decimais = sem_decimais && exp >= sig as i32 - 1;
    if (-5..sig as i32).contains(&exp) && !sem_decimais {
        let s = if exp < 0 {
            format!("0.{}{}", "0".repeat((-exp - 1) as usize), digitos)
        } else {
            let int_len = exp as usize + 1;
            if digitos.len() <= int_len {
                format!("{}{}", digitos, "0".repeat(int_len - digitos.len()))
            } else {
                format!("{}.{}", &digitos[..int_len], &digitos[int_len..])
            }
        };
        format!("{sinal}{s}")
    } else {
        let m = if digitos.len() > 1 {
            format!("{}.{}", &digitos[..1], &digitos[1..])
        } else {
            digitos.to_string()
        };
        format!("{sinal}{m}e{exp}")
    }
}

/// Número `double` como exibido.
pub fn numero(x: f64) -> String {
    com_algarismos(x, 10)
}

fn complexo(re: f64, im: f64, sig: usize) -> String {
    let sinal = if im.is_sign_negative() && !im.is_nan() {
        "-"
    } else {
        "+"
    };
    format!(
        "{} {sinal} {}i",
        com_algarismos(re, sig),
        com_algarismos(im.abs(), sig)
    )
}

/// Acima disto, matrizes aparecem resumidas (as 3 primeiras e as 3 últimas linhas e colunas).
pub const LIMITE_EXIBICAO: usize = 1000;

/// Formatador do elemento k (ordem de coluna), sem formatar os demais.
fn formatador(v: &Value) -> Box<dyn Fn(usize) -> String + '_> {
    match v {
        Value::Double(a) => {
            let x = a.as_slice();
            Box::new(move |k| numero(x[k]))
        }
        Value::Complex(a) => {
            let z = a.as_slice();
            Box::new(move |k| complexo(z[k].re, z[k].im, 10))
        }
        Value::Single(a) => {
            let x = a.as_slice();
            Box::new(move |k| com_algarismos(f64::from(x[k]), 7))
        }
        Value::Int(d) => {
            let x = d.to_i128();
            Box::new(move |k| x[k].to_string())
        }
        Value::Logical(a) => {
            let x = a.as_slice();
            Box::new(move |k| if x[k] { "true" } else { "false" }.to_string())
        }
        Value::Char(a) => {
            let x = a.as_slice();
            Box::new(move |k| x[k].to_string())
        }
    }
}

/// Índices mostrados de uma dimensão de tamanho n (`None` marca o resumo).
fn mostrados(n: usize, resumir: bool) -> Vec<Option<usize>> {
    if resumir && n > 6 {
        vec![
            Some(0),
            Some(1),
            Some(2),
            None,
            Some(n - 3),
            Some(n - 2),
            Some(n - 1),
        ]
    } else {
        (0..n).map(Some).collect()
    }
}

/// Nome da classe, se não for `double`.
fn classe_extra(v: &Value) -> String {
    match v {
        Value::Double(_) | Value::Complex(_) => String::new(),
        outro => format!(" {}", outro.class().name()),
    }
}

/// Linhas de uma fatia 2-D (`r` linhas, `c` colunas a partir de `ini`), alinhadas; resumida se
/// pedido.
fn fatia(
    el: &dyn Fn(usize) -> String,
    r: usize,
    c: usize,
    ini: usize,
    recuo: &str,
    texto: bool,
    resumir: bool,
) -> Vec<String> {
    let linhas = mostrados(r, resumir);
    if texto {
        return linhas
            .iter()
            .map(|i| match i {
                Some(i) => {
                    let s: String = (0..c).map(|j| el(ini + i + j * r)).collect();
                    format!("{recuo}{s:?}")
                }
                None => format!("{recuo}⋮"),
            })
            .collect();
    }
    let colunas = mostrados(c, resumir);
    let celula = |i: Option<usize>, j: Option<usize>| match (i, j) {
        (Some(i), Some(j)) => el(ini + i + j * r),
        (None, Some(_)) => "⋮".to_string(),
        (Some(_), None) => "…".to_string(),
        (None, None) => "⋱".to_string(),
    };
    let tabela: Vec<Vec<String>> = linhas
        .iter()
        .map(|&i| colunas.iter().map(|&j| celula(i, j)).collect())
        .collect();
    let largura = tabela
        .iter()
        .flatten()
        .map(|t| t.chars().count())
        .max()
        .unwrap_or(0);
    tabela
        .iter()
        .map(|l| {
            let campos: Vec<String> = l.iter().map(|t| format!("{t:>largura$}")).collect();
            format!("{recuo}{}", campos.join("  "))
        })
        .collect()
}

/// Corpo da exibição (sem o nome), como em `disp`. Textos saem crus, sem aspas.
pub fn corpo(o: &Obj) -> String {
    match o {
        Obj::Array(Value::Char(a)) => {
            let r = a.dims()[0];
            let c = a.numel().checked_div(r).unwrap_or(0);
            (0..r)
                .map(|i| (0..c).map(|j| a.as_slice()[i + j * r]).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        }
        Obj::Array(v) => corpo_array(v, "").join("\n"),
        f => descreve_funcao(f),
    }
}

fn descreve_funcao(o: &Obj) -> String {
    match o {
        Obj::Anonima(a) => format!("função anônima ({})", a.params.join(", ")),
        Obj::Funcao(f) => {
            let ps: Vec<String> = f
                .params
                .iter()
                .map(|p| match &p.default {
                    Some(_) => format!("{}=…", p.name),
                    None => p.name.clone(),
                })
                .collect();
            format!("função {}({})", f.nome, ps.join(", "))
        }
        Obj::Embutida(n) => format!("função embutida {n}"),
        Obj::Array(_) => unreachable!("só funções"),
    }
}

/// Linhas do corpo de um array; a primeira pode ir na linha do nome.
fn corpo_array(v: &Value, recuo: &str) -> Vec<String> {
    let dims = v.dims();
    let texto = matches!(v, Value::Char(_));
    if v.numel() == 0 {
        return vec![format!("[] ({}{})", tamanho(dims), classe_extra(v))];
    }
    let el = formatador(v);
    if texto && dims.len() == 2 && dims[0] == 1 {
        let s: String = (0..v.numel()).map(&el).collect();
        return vec![format!("{s:?}")];
    }
    if v.numel() == 1 {
        return vec![el(0)];
    }
    let resumir = v.numel() > LIMITE_EXIBICAO;
    let mut out = vec![format!("{}{}", tamanho(dims), classe_extra(v))];
    let (r, c) = (dims[0], dims[1]);
    let fatias: usize = dims[2..].iter().product();
    let dentro = format!("{recuo}  ");
    if dims.len() == 2 {
        out.extend(fatia(&*el, r, c, 0, &dentro, texto, resumir));
        return out;
    }
    for k in mostrados(fatias, resumir) {
        let Some(k) = k else {
            out.push(format!("{dentro}…"));
            continue;
        };
        // Índices das dimensões ≥ 3 desta fatia (base 1).
        let mut resto = k;
        let mut sub = Vec::new();
        for &d in &dims[2..] {
            sub.push((resto % d + 1).to_string());
            resto /= d;
        }
        out.push(format!("{dentro}[:, :, {}]", sub.join(", ")));
        out.extend(fatia(
            &*el,
            r,
            c,
            k * r * c,
            &format!("{dentro}  "),
            texto,
            resumir,
        ));
    }
    out
}

/// `nome = valor`, como no console.
pub fn com_nome(nome: &str, o: &Obj) -> String {
    let linhas = match o {
        Obj::Array(v) => corpo_array(v, ""),
        f => vec![descreve_funcao(f)],
    };
    let mut t = format!("{nome} = {}", linhas[0]);
    for l in &linhas[1..] {
        t.push('\n');
        t.push_str(l);
    }
    t
}

/// Especificação de formato de f-string: `[<>^][largura][.precisão][f|e|g|d|s]`.
struct Spec {
    alinha: Option<char>,
    largura: usize,
    precisao: Option<usize>,
    tipo: Option<char>,
}

fn le_spec(s: &str) -> Result<Spec, String> {
    let invalido = || format!("formato `{s}` inválido (use, por exemplo, `.3f`, `10.2e`, `d` ou `>8`)");
    let mut c = s.chars().peekable();
    let alinha = match c.peek() {
        Some(&a @ ('<' | '>' | '^')) => {
            c.next();
            Some(a)
        }
        _ => None,
    };
    let mut largura = String::new();
    while let Some(&d) = c.peek().filter(|d| d.is_ascii_digit()) {
        largura.push(d);
        c.next();
    }
    let mut precisao = None;
    if c.peek() == Some(&'.') {
        c.next();
        let mut p = String::new();
        while let Some(&d) = c.peek().filter(|d| d.is_ascii_digit()) {
            p.push(d);
            c.next();
        }
        precisao = Some(p.parse::<usize>().map_err(|_| invalido())?);
    }
    let tipo = c.next();
    if c.next().is_some() || tipo.is_some_and(|t| !"fegds".contains(t)) {
        return Err(invalido());
    }
    if precisao.is_some_and(|p| p > 30) || largura.len() > 3 {
        return Err(invalido());
    }
    Ok(Spec {
        alinha,
        largura: largura.parse().unwrap_or(0),
        precisao,
        tipo,
    })
}

fn formata_num(x: f64, sp: &Spec) -> Result<String, String> {
    if !x.is_finite() {
        return Ok(numero(x));
    }
    Ok(match (sp.tipo, sp.precisao) {
        (Some('f'), p) => format!("{:.*}", p.unwrap_or(6), x),
        (Some('e'), p) => {
            let s = format!("{:.*e}", p.unwrap_or(6), x);
            // Expoente com sinal e dois algarismos: 1.5e+03.
            let (m, e) = s.split_once('e').expect("científico");
            let e: i32 = e.parse().expect("expoente");
            format!("{m}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
        }
        (Some('g'), p) => com_algarismos(x, p.unwrap_or(6).max(1)),
        (Some('d'), Some(_)) => return Err("o formato `d` não tem precisão".into()),
        (Some('d'), None) => {
            if x.fract() != 0.0 {
                return Err(format!("o formato `d` exige inteiro, veio {}", numero(x)));
            }
            format!("{x:.0}")
        }
        (Some('s'), _) => return Err("o formato `s` é só para textos".into()),
        (_, Some(p)) => com_algarismos(x, p.max(1)),
        (_, None) => numero(x),
    })
}

fn alinha(s: String, sp: &Spec, padrao: char) -> String {
    let n = s.chars().count();
    if n >= sp.largura {
        return s;
    }
    let falta = sp.largura - n;
    match sp.alinha.unwrap_or(padrao) {
        '<' => format!("{s}{}", " ".repeat(falta)),
        '^' => format!("{}{s}{}", " ".repeat(falta / 2), " ".repeat(falta - falta / 2)),
        _ => format!("{}{s}", " ".repeat(falta)),
    }
}

/// Texto de um valor dentro de uma f-string, com o formato opcional.
pub fn em_fstring(o: &Obj, formato: Option<&str>) -> Result<String, String> {
    let Obj::Array(v) = o else {
        return Ok(descreve_funcao(o));
    };
    let sp = le_spec(formato.unwrap_or(""))?;
    if let Value::Char(a) = v {
        if sp.tipo.is_some_and(|t| t != 's') || sp.precisao.is_some() {
            return Err("textos só aceitam largura e alinhamento (`>10`, `<10`, `^10`, `s`)".into());
        }
        let r = a.dims()[0].max(1);
        let c = a.numel() / r;
        let linhas: Vec<String> = (0..r)
            .map(|i| (0..c).map(|j| a.as_slice()[i + j * r]).collect())
            .collect();
        return Ok(alinha(linhas.join("\n"), &sp, '<'));
    }
    let um = |k: usize| -> Result<String, String> {
        match v {
            Value::Double(a) => formata_num(a.as_slice()[k], &sp),
            Value::Single(a) => formata_num(f64::from(a.as_slice()[k]), &sp),
            Value::Int(d) => {
                let x = d.to_i128()[k];
                match sp.tipo {
                    None | Some('d') if sp.precisao.is_none() => Ok(x.to_string()),
                    _ => formata_num(x as f64, &sp),
                }
            }
            Value::Logical(a) => {
                if sp.tipo.is_some() || sp.precisao.is_some() {
                    return Err("valores lógicos só aceitam largura e alinhamento".into());
                }
                Ok(if a.as_slice()[k] { "true" } else { "false" }.into())
            }
            Value::Complex(a) => {
                let z = a.as_slice()[k];
                let im = formata_num(z.im.abs(), &sp)?;
                let sinal = if z.im.is_sign_negative() && !z.im.is_nan() {
                    '-'
                } else {
                    '+'
                };
                Ok(format!("{} {sinal} {im}i", formata_num(z.re, &sp)?))
            }
            Value::Char(_) => unreachable!("tratado acima"),
        }
    };
    let s = if v.numel() == 1 {
        um(0)?
    } else {
        // Vetores e matrizes: linhas separadas por `;`, como no literal.
        let dims = v.dims();
        let r = dims[0];
        let c = v.numel().checked_div(r).unwrap_or(0);
        let mut linhas = Vec::with_capacity(r);
        for i in 0..r {
            let campos = (0..c).map(|j| um(i + j * r)).collect::<Result<Vec<_>, _>>()?;
            linhas.push(campos.join(" "));
        }
        format!("[{}]", linhas.join("; "))
    };
    Ok(alinha(s, &sp, '>'))
}
