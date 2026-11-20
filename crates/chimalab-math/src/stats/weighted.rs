//! Estatísticas ponderadas: média, variância, desvio-padrão e mediana com pesos wᵢ ≥ 0.
//!
//! Normalizações da variância (Σw(x − m)² dividido por):
//! - `Population`: Σw (segundo momento ponderado);
//! - `Frequency`: Σw − 1. Pesos inteiros como contagens de repetição; coincide com a variância
//!   amostral dos dados replicados;
//! - `Reliability`: Σw − Σw²/Σw. Pesos como precisões relativas; invariante a w → c·w, e com pesos
//!   iguais coincide com a amostral.
//!
//! A mediana ponderada é o menor x cujo peso acumulado alcança metade do total, com a média dos
//! dois vizinhos quando o acumulado é exatamente a metade. Com pesos inteiros, coincide com a
//! mediana dos dados replicados.
//!
//! Pares com peso zero não contam. Com `NanFlag::Omit`, pares com x NaN são descartados.

use super::{NanFlag, StatsError, neumaier};

/// Normalização da variância ponderada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeightNorm {
    /// Σw (populacional).
    Population,
    /// Σw − 1 (pesos de frequência; padrão).
    #[default]
    Frequency,
    /// Σw − Σw²/Σw (pesos de confiabilidade).
    Reliability,
}

/// Valida e devolve os pares usáveis (x, w) com w > 0; `None` se houver NaN em x com `Include`.
fn pares(x: &[f64], w: &[f64], nan: NanFlag) -> Result<Option<Vec<(f64, f64)>>, StatsError> {
    if x.len() != w.len() {
        return Err(StatsError::ShapeMismatch {
            expected: x.len(),
            actual: w.len(),
        });
    }
    if w.iter().any(|v| !(v.is_finite() && *v >= 0.0)) {
        return Err(StatsError::InvalidOption("pesos finitos e ≥ 0"));
    }
    let mut out = Vec::with_capacity(x.len());
    for (&xi, &wi) in x.iter().zip(w) {
        if xi.is_nan() {
            match nan {
                NanFlag::Include => return Ok(None),
                NanFlag::Omit => continue,
            }
        }
        if wi > 0.0 {
            out.push((xi, wi));
        }
    }
    Ok(Some(out))
}

/// Média ponderada Σwx/Σw. Sem peso positivo: NaN.
///
/// Erros: `formaIncompativel` (tamanhos), `opcaoInvalida` (peso negativo, NaN ou infinito).
pub fn mean_weighted(x: &[f64], w: &[f64], nan: NanFlag) -> Result<f64, StatsError> {
    let Some(p) = pares(x, w, nan)? else {
        return Ok(f64::NAN);
    };
    Ok(media(&p))
}

fn media(p: &[(f64, f64)]) -> f64 {
    let sw = neumaier(p.iter().map(|q| q.1));
    if sw == 0.0 {
        return f64::NAN;
    }
    neumaier(p.iter().map(|q| q.0 * q.1)) / sw
}

/// Variância ponderada; um só par com peso: 0. Sem peso positivo: NaN.
///
/// Erros: os de [`mean_weighted`].
pub fn var_weighted(x: &[f64], w: &[f64], norm: WeightNorm, nan: NanFlag) -> Result<f64, StatsError> {
    let Some(p) = pares(x, w, nan)? else {
        return Ok(f64::NAN);
    };
    let m = media(&p);
    if !m.is_finite() {
        return Ok(f64::NAN);
    }
    let sw = neumaier(p.iter().map(|q| q.1));
    // Dois passos corrigidos (como em `var`): Σw·d² − (Σw·d)²/Σw.
    let s2 = neumaier(p.iter().map(|&(x, w)| w * (x - m) * (x - m)));
    let s1 = neumaier(p.iter().map(|&(x, w)| w * (x - m)));
    let ss = (s2 - s1 * s1 / sw).max(0.0);
    let den = match norm {
        WeightNorm::Population => sw,
        WeightNorm::Frequency => sw - 1.0,
        WeightNorm::Reliability => sw - neumaier(p.iter().map(|q| q.1 * q.1)) / sw,
    };
    if ss == 0.0 {
        return Ok(0.0);
    }
    Ok(if den > 0.0 { ss / den } else { f64::NAN })
}

/// Desvio-padrão ponderado: √var.
pub fn std_weighted(x: &[f64], w: &[f64], norm: WeightNorm, nan: NanFlag) -> Result<f64, StatsError> {
    Ok(var_weighted(x, w, norm, nan)?.sqrt())
}

/// Mediana ponderada (ver o módulo). Sem peso positivo: NaN.
pub fn median_weighted(x: &[f64], w: &[f64], nan: NanFlag) -> Result<f64, StatsError> {
    let Some(mut p) = pares(x, w, nan)? else {
        return Ok(f64::NAN);
    };
    if p.is_empty() {
        return Ok(f64::NAN);
    }
    p.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total = neumaier(p.iter().map(|q| q.1));
    let metade = total / 2.0;
    // Acumulado compensado (Neumaier corrente, mesma ordem da soma total), para que pesos
    // inteiros caiam exatamente na metade.
    let (mut soma, mut comp) = (0.0f64, 0.0f64);
    for i in 0..p.len() {
        let v = p[i].1;
        let t = soma + v;
        comp += if soma.abs() >= v.abs() {
            (soma - t) + v
        } else {
            (v - t) + soma
        };
        soma = t;
        let acc = soma + comp;
        if acc >= metade {
            if acc == metade && i + 1 < p.len() {
                return Ok(p[i].0 / 2.0 + p[i + 1].0 / 2.0);
            }
            return Ok(p[i].0);
        }
    }
    Ok(p[p.len() - 1].0)
}
