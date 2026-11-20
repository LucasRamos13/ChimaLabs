//! Estatísticas em janelas móveis.

use std::collections::VecDeque;

use super::{NanFlag, StatsError, VarNorm, mean_of, median_mut, neumaier, var_of};

/// Janela em torno do elemento i.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    /// k elementos centrados em i. k ímpar: (k − 1)/2 de cada lado. k par: k/2 antes e k/2 − 1
    /// depois (centrada entre i − 1 e i).
    Centered(usize),
    /// (antes, depois): elementos i − antes até i + depois.
    Asymmetric(usize, usize),
}

impl Window {
    fn sides(self) -> Result<(usize, usize), StatsError> {
        match self {
            Self::Centered(0) => Err(StatsError::InvalidOption("janela com pelo menos 1 elemento")),
            Self::Centered(k) if k % 2 == 1 => Ok(((k - 1) / 2, (k - 1) / 2)),
            Self::Centered(k) => Ok((k / 2, k / 2 - 1)),
            Self::Asymmetric(b, a) => Ok((b, a)),
        }
    }
}

/// Tratamento das bordas, onde a janela não cabe inteira.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Endpoints {
    /// Usa só os elementos disponíveis (padrão).
    #[default]
    Shrink,
    /// Preenche com o valor dado.
    Fill(f64),
    /// Descarta: a saída só tem as posições com janela completa.
    Discard,
}

/// Estatística da janela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovingStat {
    /// Soma.
    Sum,
    /// Média.
    Mean,
    /// Mediana.
    Median,
    /// Máximo.
    Max,
    /// Mínimo.
    Min,
    /// Produto.
    Prod,
    /// Variância (normalização dada).
    Var(VarNorm),
    /// Desvio-padrão.
    Std(VarNorm),
}

/// Estatística móvel de `x`. Com `NanFlag::Include`, uma janela com NaN dá NaN; com `Omit`, o
/// NaN é ignorado (janela só de NaN dá NaN; soma vazia dá 0).
///
/// Soma e média usam soma deslizante compensada, recalculada a cada 1024 passos para limitar a
/// deriva; máximo e mínimo usam fila monótona (O(n) para qualquer janela).
///
/// Erros: `opcaoInvalida` (janela centrada de tamanho 0).
pub fn moving(
    x: &[f64],
    window: Window,
    stat: MovingStat,
    endpoints: Endpoints,
    nan: NanFlag,
) -> Result<Vec<f64>, StatsError> {
    let (b, a) = window.sides()?;
    let n = x.len();
    let lo = |i: usize| i.saturating_sub(b);
    let hi = |i: usize| (i + a).min(n.saturating_sub(1));
    let completa = |i: usize| i >= b && i + a < n;
    let mut out: Vec<f64> = match stat {
        MovingStat::Sum | MovingStat::Mean => sliding_sum(x, b, a, stat == MovingStat::Mean, nan),
        MovingStat::Max | MovingStat::Min => sliding_ext(x, b, a, stat == MovingStat::Max, nan),
        _ => (0..n)
            .map(|i| {
                let w = &x[lo(i)..=hi(i)];
                let tem_nan = w.iter().any(|v| v.is_nan());
                if tem_nan && nan == NanFlag::Include {
                    return f64::NAN;
                }
                let mut v: Vec<f64> = w.iter().copied().filter(|v| !v.is_nan()).collect();
                // Soma, média, máximo e mínimo saíram acima; resta a mediana no último braço.
                match stat {
                    MovingStat::Prod => v.iter().product(),
                    MovingStat::Var(nm) => var_of(&v, nm),
                    MovingStat::Std(nm) => var_of(&v, nm).sqrt(),
                    _ => median_mut(&mut v),
                }
            })
            .collect(),
    };
    match endpoints {
        Endpoints::Shrink => {}
        Endpoints::Fill(v) => {
            for (i, o) in out.iter_mut().enumerate() {
                if !completa(i) {
                    *o = v;
                }
            }
        }
        Endpoints::Discard => {
            out = out
                .into_iter()
                .enumerate()
                .filter(|(i, _)| completa(*i))
                .map(|(_, v)| v)
                .collect();
        }
    }
    Ok(out)
}

fn sliding_sum(x: &[f64], b: usize, a: usize, media: bool, nan: NanFlag) -> Vec<f64> {
    let n = x.len();
    let mut out = Vec::with_capacity(n);
    // Janela [l, r] mantida com soma compensada (s, c), contagem de válidos e de NaN.
    let (mut l, mut r) = (0usize, 0usize); // [l, r)
    let (mut s, mut c) = (0.0f64, 0.0f64);
    let (mut validos, mut nans) = (0usize, 0usize);
    let add = |s: &mut f64, c: &mut f64, v: f64| {
        let t = *s + v;
        if s.abs() >= v.abs() {
            *c += (*s - t) + v;
        } else {
            *c += (v - t) + *s;
        }
        *s = t;
    };
    for i in 0..n {
        let (nl, nr) = (i.saturating_sub(b), (i + a + 1).min(n));
        while r < nr {
            if x[r].is_nan() {
                nans += 1;
            } else {
                add(&mut s, &mut c, x[r]);
                validos += 1;
            }
            r += 1;
        }
        while l < nl {
            if x[l].is_nan() {
                nans -= 1;
            } else {
                add(&mut s, &mut c, -x[l]);
                validos -= 1;
            }
            l += 1;
        }
        // Recalcula periodicamente (e sempre que houver ±∞, que a remoção não desfaz).
        if i % 1024 == 0 || !s.is_finite() {
            let w = x[l..r].iter().copied().filter(|v| !v.is_nan());
            s = neumaier(w);
            c = 0.0;
        }
        let total = if s.is_finite() { s + c } else { s };
        let v = if nans > 0 && nan == NanFlag::Include {
            f64::NAN
        } else if media {
            if validos == 0 {
                f64::NAN
            } else if total.is_finite() {
                total / validos as f64
            } else {
                // Overflow da soma com dados finitos: média reescalada.
                let w: Vec<f64> = x[l..r].iter().copied().filter(|v| !v.is_nan()).collect();
                mean_of(&w)
            }
        } else {
            total
        };
        out.push(v);
    }
    out
}

fn sliding_ext(x: &[f64], b: usize, a: usize, maior: bool, nan: NanFlag) -> Vec<f64> {
    let n = x.len();
    let mut out = Vec::with_capacity(n);
    let mut fila: VecDeque<usize> = VecDeque::new(); // índices com valores monótonos
    let mut r = 0usize;
    let melhor = |u: f64, v: f64| if maior { u >= v } else { u <= v };
    let mut nan_pos: VecDeque<usize> = VecDeque::new();
    for i in 0..n {
        let (nl, nr) = (i.saturating_sub(b), (i + a + 1).min(n));
        while r < nr {
            if x[r].is_nan() {
                nan_pos.push_back(r);
            } else {
                while fila.back().is_some_and(|&j| melhor(x[r], x[j])) {
                    fila.pop_back();
                }
                fila.push_back(r);
            }
            r += 1;
        }
        while fila.front().is_some_and(|&j| j < nl) {
            fila.pop_front();
        }
        while nan_pos.front().is_some_and(|&j| j < nl) {
            nan_pos.pop_front();
        }
        let v = if !nan_pos.is_empty() && nan == NanFlag::Include {
            f64::NAN
        } else {
            fila.front().map_or(f64::NAN, |&j| x[j])
        };
        out.push(v);
    }
    out
}
