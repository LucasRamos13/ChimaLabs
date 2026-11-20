//! Preparação de dados: normalização, reescala, outliers e histogramas.

use super::{NanFlag, QuantileType, StatsError, VarNorm, mean, median, quantile, quantile_sorted, std};

/// 1/Φ⁻¹(3/4): torna o desvio absoluto mediano (MAD) um estimador consistente do desvio-padrão
/// para dados normais.
const MAD_NORMAL: f64 = 1.482_602_218_505_602;

/// Centro para [`NormalizeMethod::Center`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CenterBy {
    /// Média.
    Mean,
    /// Mediana.
    Median,
    /// Valor dado.
    Value(f64),
}

/// Escala para [`NormalizeMethod::Scale`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaleBy {
    /// Desvio-padrão amostral.
    Std,
    /// MAD escalado (1,4826·MAD).
    Mad,
    /// Amplitude interquartil (tipo 7).
    Iqr,
    /// Primeiro elemento não NaN.
    First,
    /// Valor dado.
    Value(f64),
}

/// Método de normalização.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NormalizeMethod {
    /// (x − média)/desvio (padrão: amostral).
    ZScore(VarNorm),
    /// Reescala linear para [a, b].
    Range(f64, f64),
    /// Divide pela norma p (p ≥ 1 ou ∞).
    Norm(f64),
    /// Subtrai o centro.
    Center(CenterBy),
    /// Divide pela escala.
    Scale(ScaleBy),
}

impl Default for NormalizeMethod {
    fn default() -> Self {
        Self::ZScore(VarNorm::Sample)
    }
}

fn mad(x: &[f64]) -> f64 {
    let m = median(x, NanFlag::Omit);
    let d: Vec<f64> = x.iter().filter(|v| !v.is_nan()).map(|v| (v - m).abs()).collect();
    median(&d, NanFlag::Omit)
}

fn iqr(x: &[f64]) -> f64 {
    let q = quantile(x, &[0.25, 0.75], QuantileType::default(), NanFlag::Omit).expect("p válidos");
    q[1] - q[0]
}

/// Normaliza `x`; NaN fica NaN e não entra nas estatísticas. Com escala nula (dados constantes),
/// os valores centrados (todos 0) são devolvidos sem dividir.
///
/// Erros: `opcaoInvalida` (norma p < 1 ou NaN, intervalo [a, b] não finito ou com a > b).
pub fn normalize(x: &[f64], method: NormalizeMethod) -> Result<Vec<f64>, StatsError> {
    let (centro, escala) = match method {
        NormalizeMethod::ZScore(nm) => (mean(x, NanFlag::Omit), std(x, nm, NanFlag::Omit)),
        NormalizeMethod::Range(a, b) => return rescale(x, a, b),
        NormalizeMethod::Norm(p) => {
            if p.is_nan() || p < 1.0 {
                return Err(StatsError::InvalidOption("norma p ≥ 1"));
            }
            let v = x.iter().filter(|v| !v.is_nan()).map(|v| v.abs());
            let nrm = if p.is_infinite() {
                v.fold(0.0, f64::max)
            } else {
                // Escala pelo maior |x| para não estourar |x|^p.
                let m = x
                    .iter()
                    .filter(|v| !v.is_nan())
                    .fold(0.0f64, |a, v| a.max(v.abs()));
                if m == 0.0 {
                    0.0
                } else {
                    m * v.map(|a| (a / m).powf(p)).sum::<f64>().powf(1.0 / p)
                }
            };
            (0.0, nrm)
        }
        NormalizeMethod::Center(c) => {
            let v = match c {
                CenterBy::Mean => mean(x, NanFlag::Omit),
                CenterBy::Median => median(x, NanFlag::Omit),
                CenterBy::Value(v) => v,
            };
            (v, 1.0)
        }
        NormalizeMethod::Scale(s) => {
            let v = match s {
                ScaleBy::Std => std(x, VarNorm::Sample, NanFlag::Omit),
                ScaleBy::Mad => MAD_NORMAL * mad(x),
                ScaleBy::Iqr => iqr(x),
                ScaleBy::First => x.iter().copied().find(|v| !v.is_nan()).unwrap_or(f64::NAN),
                ScaleBy::Value(v) => v,
            };
            (0.0, v)
        }
    };
    let escala = if escala == 0.0 { 1.0 } else { escala };
    Ok(x.iter().map(|v| (v - centro) / escala).collect())
}

/// Reescala linear de [min, max] dos dados para [a, b]; NaN fica NaN. Dados constantes vão para
/// o ponto médio (a + b)/2 (não há informação para escolher um extremo).
///
/// Erros: `opcaoInvalida` (a ou b não finitos, ou a > b).
pub fn rescale(x: &[f64], a: f64, b: f64) -> Result<Vec<f64>, StatsError> {
    if !(a.is_finite() && b.is_finite() && a <= b) {
        return Err(StatsError::InvalidOption("intervalo [a, b] finito com a ≤ b"));
    }
    let lo = super::min(x, NanFlag::Omit);
    let hi = super::max(x, NanFlag::Omit);
    Ok(x.iter()
        .map(|&v| {
            if v.is_nan() {
                f64::NAN
            } else if hi == lo {
                a / 2.0 + b / 2.0
            } else {
                // Forma que acerta exatamente os extremos: v = lo → a, v = hi → b.
                let t = (v - lo) / (hi - lo);
                if t == 1.0 { b } else { a + t * (b - a) }
            }
        })
        .collect())
}

/// Método de detecção de outliers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutlierMethod {
    /// Mais de `t` MADs escalados da mediana (robusto; padrão t = 3).
    Median(f64),
    /// Mais de `t` desvios-padrão da média.
    Mean(f64),
    /// Abaixo de Q1 − t·IQR ou acima de Q3 + t·IQR (Tukey; padrão t = 1,5).
    Quartiles(f64),
}

impl Default for OutlierMethod {
    fn default() -> Self {
        Self::Median(3.0)
    }
}

/// Limites usados na detecção.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlierBounds {
    /// Limite inferior.
    pub lower: f64,
    /// Limite superior.
    pub upper: f64,
    /// Centro (mediana, média ou mediana dos quartis).
    pub center: f64,
}

/// Marca outliers (NaN nunca é outlier) e devolve os limites.
///
/// Erros: `opcaoInvalida` (limiar negativo ou não finito).
pub fn isoutlier(x: &[f64], method: OutlierMethod) -> Result<(Vec<bool>, OutlierBounds), StatsError> {
    let t = match method {
        OutlierMethod::Median(t) | OutlierMethod::Mean(t) | OutlierMethod::Quartiles(t) => t,
    };
    if !(t.is_finite() && t >= 0.0) {
        return Err(StatsError::InvalidOption("limiar finito e ≥ 0"));
    }
    let b = match method {
        OutlierMethod::Median(t) => {
            let c = median(x, NanFlag::Omit);
            let s = MAD_NORMAL * mad(x);
            OutlierBounds {
                lower: c - t * s,
                upper: c + t * s,
                center: c,
            }
        }
        OutlierMethod::Mean(t) => {
            let c = mean(x, NanFlag::Omit);
            let s = std(x, VarNorm::Sample, NanFlag::Omit);
            OutlierBounds {
                lower: c - t * s,
                upper: c + t * s,
                center: c,
            }
        }
        OutlierMethod::Quartiles(t) => {
            let mut v: Vec<f64> = x.iter().copied().filter(|v| !v.is_nan()).collect();
            v.sort_by(f64::total_cmp);
            let (q1, q3) = if v.is_empty() {
                (f64::NAN, f64::NAN)
            } else {
                (
                    quantile_sorted(&v, 0.25, QuantileType::default()),
                    quantile_sorted(&v, 0.75, QuantileType::default()),
                )
            };
            OutlierBounds {
                lower: q1 - t * (q3 - q1),
                upper: q3 + t * (q3 - q1),
                center: q1 / 2.0 + q3 / 2.0,
            }
        }
    };
    let marca = x
        .iter()
        .map(|&v| !v.is_nan() && (v < b.lower || v > b.upper))
        .collect();
    Ok((marca, b))
}

/// Remove os outliers: devolve os valores mantidos e a máscara dos removidos.
pub fn rmoutliers(x: &[f64], method: OutlierMethod) -> Result<(Vec<f64>, Vec<bool>), StatsError> {
    let (m, _) = isoutlier(x, method)?;
    let kept = x.iter().zip(&m).filter(|(_, o)| !**o).map(|(v, _)| *v).collect();
    Ok((kept, m))
}

/// Preenchimento de outliers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FillMethod {
    /// Valor constante.
    Constant(f64),
    /// Centro do método (mediana, média ou meio dos quartis).
    Center,
    /// Limite mais próximo (inferior ou superior).
    Clip,
    /// Valor não outlier anterior (o seguinte, no início).
    Previous,
    /// Valor não outlier seguinte (o anterior, no fim).
    Next,
    /// Valor não outlier mais próximo em posição (empate: o anterior).
    Nearest,
    /// Interpolação linear entre os vizinhos não outliers (extremos: o mais próximo).
    Linear,
}

/// Substitui os outliers. Devolve os dados preenchidos e a máscara.
pub fn filloutliers(
    x: &[f64],
    fill: FillMethod,
    method: OutlierMethod,
) -> Result<(Vec<f64>, Vec<bool>), StatsError> {
    let (m, b) = isoutlier(x, method)?;
    let n = x.len();
    // Índices não outliers e não NaN, para os métodos posicionais.
    let bons: Vec<usize> = (0..n).filter(|&i| !m[i] && !x[i].is_nan()).collect();
    let mut out = x.to_vec();
    for i in (0..n).filter(|&i| m[i]) {
        // Primeiro bom à direita.
        let j = bons.partition_point(|&k| k < i);
        let ant = j.checked_sub(1).map(|k| bons[k]);
        let prox = bons.get(j).copied();
        out[i] = match fill {
            FillMethod::Constant(v) => v,
            FillMethod::Center => b.center,
            FillMethod::Clip => {
                if x[i] < b.lower {
                    b.lower
                } else {
                    b.upper
                }
            }
            FillMethod::Previous => ant.or(prox).map_or(f64::NAN, |k| x[k]),
            FillMethod::Next => prox.or(ant).map_or(f64::NAN, |k| x[k]),
            FillMethod::Nearest => match (ant, prox) {
                (Some(a), Some(p)) => x[if i - a <= p - i { a } else { p }],
                (a, p) => a.or(p).map_or(f64::NAN, |k| x[k]),
            },
            FillMethod::Linear => match (ant, prox) {
                (Some(a), Some(p)) => x[a] + (x[p] - x[a]) * (i - a) as f64 / (p - a) as f64,
                (a, p) => a.or(p).map_or(f64::NAN, |k| x[k]),
            },
        };
    }
    Ok((out, m))
}

/// Contagens por intervalo [eᵢ, eᵢ₊₁), com o último fechado [e_{m−1}, e_m]. Valores fora das
/// bordas e NaN não contam.
///
/// Erros: `opcaoInvalida` (menos de 2 bordas, bordas não finitas ou não estritamente crescentes).
pub fn histcounts(x: &[f64], edges: &[f64]) -> Result<Vec<usize>, StatsError> {
    if edges.len() < 2 || edges.iter().any(|e| !e.is_finite()) || edges.windows(2).any(|w| w[1] <= w[0]) {
        return Err(StatsError::InvalidOption(
            "bordas finitas, estritamente crescentes, pelo menos 2",
        ));
    }
    let m = edges.len() - 1;
    let mut c = vec![0usize; m];
    for &v in x {
        if v.is_nan() || v < edges[0] || v > edges[m] {
            continue;
        }
        let k = edges.partition_point(|&e| e <= v).min(m) - 1;
        c[k] += 1;
    }
    Ok(c)
}

/// Regra de largura automática.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BinRule {
    /// Freedman–Diaconis: 2·IQR·n^{−1/3} (robusta; padrão).
    #[default]
    FreedmanDiaconis,
    /// Scott: 3,49·σ·n^{−1/3}.
    Scott,
    /// Sturges: ⌈log₂ n⌉ + 1 intervalos.
    Sturges,
    /// ⌈√n⌉ intervalos.
    Sqrt,
}

/// Largura "redonda": a menor de {1, 2, 2,5, 5, 10}·10ᵏ que é ≥ w.
fn nice(w: f64) -> f64 {
    let k = w.log10().floor();
    let base = 10f64.powf(k);
    // 10·base ≥ w sempre (base = 10^⌊log₁₀ w⌋).
    [1.0, 2.0, 2.5, 5.0]
        .into_iter()
        .find(|m| m * base >= w * (1.0 - 1e-12))
        .unwrap_or(10.0)
        * base
}

/// Histograma com bordas automáticas: largura pela regra, arredondada para um valor "redondo", e
/// bordas em múltiplos da largura. Só valores finitos contam. Sem valores finitos: uma borda
/// [0, 1] e contagem 0; todos iguais a v: [v − ½, v + ½].
pub fn histcounts_auto(x: &[f64], rule: BinRule) -> (Vec<usize>, Vec<f64>) {
    let v: Vec<f64> = x.iter().copied().filter(|a| a.is_finite()).collect();
    if v.is_empty() {
        return (vec![0], vec![0.0, 1.0]);
    }
    let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if lo == hi {
        let e = vec![lo - 0.5, lo + 0.5];
        return (histcounts(&v, &e).expect("bordas válidas"), e);
    }
    let n = v.len() as f64;
    let span = hi - lo;
    let w = match rule {
        BinRule::FreedmanDiaconis => {
            let q = iqr(&v);
            if q > 0.0 {
                2.0 * q * n.powf(-1.0 / 3.0)
            } else {
                3.49 * std(&v, VarNorm::Sample, NanFlag::Omit) * n.powf(-1.0 / 3.0)
            }
        }
        BinRule::Scott => 3.49 * std(&v, VarNorm::Sample, NanFlag::Omit) * n.powf(-1.0 / 3.0),
        BinRule::Sturges => span / (n.log2().ceil() + 1.0),
        BinRule::Sqrt => span / n.sqrt().ceil(),
    };
    // Largura mínima: no máximo 10⁶ intervalos.
    let w = nice(w.max(span / 1e6));
    let start = (lo / w).floor() * w;
    // start ≤ lo < hi: o laço acrescenta pelo menos uma borda.
    let mut e = vec![start];
    while *e.last().expect("não vazio") < hi {
        let k = e.len() as f64;
        e.push(start + k * w);
    }
    (histcounts(&v, &e).expect("bordas válidas"), e)
}
