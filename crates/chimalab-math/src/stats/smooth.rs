//! Suavização, extremos locais com proeminência e testes de outliers (Grubbs, GESD).
//!
//! Fontes:
//! - A. Savitzky & M. J. E. Golay, Anal. Chem. 36 (1964) 1627–1639 (polinômio local por mínimos
//!   quadrados; aqui ajustado diretamente em cada janela, inclusive nas bordas);
//! - W. S. Cleveland, J. Am. Stat. Assoc. 74 (1979) 829–836 (LOWESS: regressão local com pesos
//!   tricúbicos e iterações robustas com pesos bi-quadrados);
//! - B. Rosner, Technometrics 25 (1983) 165–172 (GESD); F. E. Grubbs, Technometrics 11 (1969);
//! - proeminência topográfica (altura do pico acima do maior dos dois mínimos até um pico mais
//!   alto ou até a borda).

use super::{NanFlag, StatsError, VarNorm, mean, median, std};
use crate::special::betainc;

/// Método de suavização.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmoothMethod {
    /// Média gaussiana com janela de k pontos (σ = k/5, como na prática comum de janela ±2,5σ).
    Gaussian(usize),
    /// Savitzky–Golay: polinômio de grau `degree` ajustado em janelas de k pontos (k ímpar).
    SavitzkyGolay {
        /// Pontos da janela (ímpar, > grau).
        window: usize,
        /// Grau do polinômio.
        degree: usize,
    },
    /// LOWESS (reta local) com fração `span` dos pontos; `robust` faz 4 iterações robustas.
    Lowess {
        /// Fração dos pontos em cada vizinhança, em (0, 1].
        span: f64,
        /// Iterações robustas (pesos bi-quadrados dos resíduos).
        robust: bool,
    },
    /// LOESS (parábola local).
    Loess {
        /// Fração dos pontos.
        span: f64,
        /// Iterações robustas.
        robust: bool,
    },
}

/// Mínimos quadrados ponderados: polinômio de grau d em (t, y, w), avaliado em t0.
fn local_poly(t: &[f64], y: &[f64], w: &[f64], d: usize, t0: f64) -> f64 {
    // Equações normais com t centrado em t0 e escalado pela maior distância (bem condicionadas
    // para os graus baixos usados aqui).
    let esc = t
        .iter()
        .map(|v| (v - t0).abs())
        .fold(0.0f64, f64::max)
        .max(f64::MIN_POSITIVE);
    let p = d + 1;
    let mut a = vec![vec![0.0; p]; p];
    let mut b = vec![0.0; p];
    for k in 0..t.len() {
        if w[k] == 0.0 {
            continue;
        }
        let u = (t[k] - t0) / esc;
        let mut pot = vec![1.0; 2 * p];
        for j in 1..2 * p {
            pot[j] = pot[j - 1] * u;
        }
        for i in 0..p {
            b[i] += w[k] * pot[i] * y[k];
            for j in 0..p {
                a[i][j] += w[k] * pot[i + j];
            }
        }
    }
    // Eliminação de Gauss com pivotamento parcial; o valor em t0 é o coeficiente constante.
    for c in 0..p {
        let piv = (c..p)
            .max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))
            .unwrap_or(c);
        a.swap(c, piv);
        b.swap(c, piv);
        if a[c][c] == 0.0 {
            continue;
        }
        for r in c + 1..p {
            let f = a[r][c] / a[c][c];
            for j in c..p {
                a[r][j] -= f * a[c][j];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = vec![0.0; p];
    for i in (0..p).rev() {
        let s: f64 = (i + 1..p).map(|j| a[i][j] * x[j]).sum();
        x[i] = if a[i][i] == 0.0 { 0.0 } else { (b[i] - s) / a[i][i] };
    }
    x[0]
}

/// Suaviza y (amostras igualmente espaçadas; posição = índice). NaN é ignorado nas janelas.
///
/// Erros: `opcaoInvalida` (janela 0 ou par no Savitzky–Golay, grau ≥ janela, span fora de (0, 1]).
pub fn smoothdata(y: &[f64], method: SmoothMethod) -> Result<Vec<f64>, StatsError> {
    let n = y.len();
    let t: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let valido: Vec<bool> = y.iter().map(|v| !v.is_nan()).collect();
    match method {
        SmoothMethod::Gaussian(k) => {
            if k == 0 {
                return Err(StatsError::InvalidOption("janela ≥ 1"));
            }
            let sigma = k as f64 / 5.0;
            let h = k / 2;
            Ok((0..n)
                .map(|i| {
                    let (lo, hi) = (i.saturating_sub(h), (i + h).min(n.saturating_sub(1)));
                    let (mut s, mut ws) = (0.0, 0.0);
                    for j in lo..=hi {
                        if valido[j] {
                            let w = (-0.5 * ((j as f64 - i as f64) / sigma).powi(2)).exp();
                            s += w * y[j];
                            ws += w;
                        }
                    }
                    if ws == 0.0 { f64::NAN } else { s / ws }
                })
                .collect())
        }
        SmoothMethod::SavitzkyGolay { window, degree } => {
            if window == 0 || window % 2 == 0 || degree >= window {
                return Err(StatsError::InvalidOption(
                    "Savitzky–Golay: janela ímpar e grau < janela",
                ));
            }
            let h = window / 2;
            Ok((0..n)
                .map(|i| {
                    // Nas bordas a janela é deslocada para caber inteira (ajuste não centrado).
                    let lo = i.saturating_sub(h).min(n.saturating_sub(window));
                    let hi = (lo + window).min(n);
                    let w: Vec<f64> = (lo..hi).map(|j| f64::from(u8::from(valido[j]))).collect();
                    let yy: Vec<f64> = (lo..hi).map(|j| if valido[j] { y[j] } else { 0.0 }).collect();
                    let validos = w.iter().filter(|v| **v > 0.0).count();
                    if validos == 0 {
                        return f64::NAN;
                    }
                    local_poly(&t[lo..hi], &yy, &w, degree.min(validos - 1), t[i])
                })
                .collect())
        }
        SmoothMethod::Lowess { span, robust } | SmoothMethod::Loess { span, robust } => {
            if !(span > 0.0 && span <= 1.0) {
                return Err(StatsError::InvalidOption("span em (0, 1]"));
            }
            let grau = if matches!(method, SmoothMethod::Lowess { .. }) {
                1
            } else {
                2
            };
            let idx: Vec<usize> = (0..n).filter(|&i| valido[i]).collect();
            let m = idx.len();
            if m == 0 {
                return Ok(vec![f64::NAN; n]);
            }
            let q = ((span * m as f64).ceil() as usize).clamp((grau + 1).min(m), m);
            let mut rob = vec![1.0; m];
            let mut fit = vec![0.0; m];
            for it in 0..if robust { 5 } else { 1 } {
                for (a, &i) in idx.iter().enumerate() {
                    // q vizinhos mais próximos (em posição) entre os válidos.
                    let lo = a.saturating_sub(q - 1).min(m - q);
                    let mut l = lo;
                    for c in lo..=a.min(m - q) {
                        let dl = t[i] - t[idx[c]];
                        let dr = t[idx[c + q - 1]] - t[i];
                        if dl.max(dr) < (t[i] - t[idx[l]]).max(t[idx[l + q - 1]] - t[i]) {
                            l = c;
                        }
                    }
                    let viz = &idx[l..l + q];
                    let dmax = viz.iter().map(|&j| (t[j] - t[i]).abs()).fold(0.0f64, f64::max) * 1.000_001;
                    let tt: Vec<f64> = viz.iter().map(|&j| t[j]).collect();
                    let yy: Vec<f64> = viz.iter().map(|&j| y[j]).collect();
                    let w: Vec<f64> = viz
                        .iter()
                        .zip(l..l + q)
                        .map(|(&j, r)| {
                            let u = if dmax == 0.0 {
                                0.0
                            } else {
                                (t[j] - t[i]).abs() / dmax
                            };
                            (1.0 - u.powi(3)).powi(3) * rob[r]
                        })
                        .collect();
                    fit[a] = local_poly(&tt, &yy, &w, grau.min(q - 1), t[i]);
                }
                if it < 4 && robust {
                    // Pesos bi-quadrados dos resíduos, escala 6·MAD.
                    let res: Vec<f64> = idx.iter().zip(&fit).map(|(&i, f)| (y[i] - f).abs()).collect();
                    let s = 6.0 * median(&res, NanFlag::Omit);
                    // Resíduos no nível do arredondamento (dados exatamente ajustáveis): não há o
                    // que robustecer; pesos de ruído degenerariam os ajustes locais (regressão).
                    let escala = idx.iter().map(|&i| y[i].abs()).fold(0.0f64, f64::max);
                    let ruido = s <= 64.0 * f64::EPSILON * escala;
                    for (r, e) in rob.iter_mut().zip(&res) {
                        *r = if ruido {
                            1.0
                        } else {
                            (1.0 - (e / s).min(1.0).powi(2)).powi(2)
                        };
                    }
                }
            }
            let mut out = vec![f64::NAN; n];
            for (a, &i) in idx.iter().enumerate() {
                out[i] = fit[a];
            }
            Ok(out)
        }
    }
}

/// Extremos locais: índices dos máximos locais (ou mínimos, com `minima`) com proeminência
/// ≥ `min_prominence` e separação ≥ `min_separation` amostras (os mais proeminentes têm
/// prioridade). Platôs contam uma vez, no centro. Devolve (índice, proeminência).
pub fn local_extrema(
    y: &[f64],
    minima: bool,
    min_prominence: f64,
    min_separation: usize,
) -> Vec<(usize, f64)> {
    let s = if minima { -1.0 } else { 1.0 };
    let v: Vec<f64> = y.iter().map(|x| s * x).collect();
    let n = v.len();
    let mut picos = Vec::new();
    let mut i = 1;
    while i + 1 < n {
        if v[i].is_nan() || v[i].partial_cmp(&v[i - 1]) != Some(std::cmp::Ordering::Greater) {
            i += 1;
            continue;
        }
        // Fim do platô.
        let mut j = i;
        while j + 1 < n && v[j + 1] == v[i] {
            j += 1;
        }
        if j + 1 < n && v[j + 1] < v[i] {
            picos.push((i + j) / 2);
        }
        i = j + 1;
    }
    // Proeminência: para cada lado, o menor valor até um ponto mais alto (ou a borda); a base
    // é o maior desses dois mínimos.
    let prom: Vec<f64> = picos
        .iter()
        .map(|&p| {
            let h = v[p];
            let lado = |iter: &mut dyn Iterator<Item = usize>| {
                let mut m = h;
                for k in iter {
                    if v[k] > h {
                        break;
                    }
                    if !v[k].is_nan() {
                        m = m.min(v[k]);
                    }
                }
                m
            };
            let esq = lado(&mut (0..p).rev());
            let dir = lado(&mut (p + 1..n));
            h - esq.max(dir)
        })
        .collect();
    let mut cand: Vec<(usize, f64)> = picos
        .into_iter()
        .zip(prom)
        .filter(|(_, p)| *p >= min_prominence)
        .collect();
    if min_separation > 0 {
        let mut ordem: Vec<usize> = (0..cand.len()).collect();
        ordem.sort_by(|&a, &b| v[cand[b].0].total_cmp(&v[cand[a].0]));
        let mut fica = vec![true; cand.len()];
        for (pos, &a) in ordem.iter().enumerate() {
            if !fica[a] {
                continue;
            }
            for &b in &ordem[pos + 1..] {
                if cand[a].0.abs_diff(cand[b].0) < min_separation {
                    fica[b] = false;
                }
            }
        }
        cand = cand
            .into_iter()
            .zip(fica)
            .filter(|(_, f)| *f)
            .map(|(c, _)| c)
            .collect();
    }
    cand
}

/// Valor crítico de Grubbs/GESD para n pontos, nível α (bilateral): λ = (n−1)·t/√((n−2+t²)n),
/// com t o quantil 1 − α/(2n) da t de Student com n − 2 graus de liberdade.
fn grubbs_critical(n: usize, alpha: f64) -> f64 {
    let nf = n as f64;
    let df = nf - 2.0;
    let p = 1.0 - alpha / (2.0 * nf);
    // Quantil da t de Student: P(|T| ≤ t) = I_x(½, df/2) com x = t²/(df + t²); I_x é monótona
    // em x, então a bissecção (100 passos, abaixo do ulp) inverte com segurança.
    let alvo = 2.0 * p - 1.0;
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if betainc(mid, 0.5, df / 2.0, false) < alvo {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let x = 0.5 * (lo + hi);
    let t = (df * x / (1.0 - x)).sqrt();
    (nf - 1.0) * t / ((df + t * t) * nf).sqrt()
}

/// Teste GESD (generalized extreme Studentized deviate, Rosner 1983): até `max_outliers`
/// outliers ao nível α. Com `max_outliers = 1` é o teste de Grubbs bilateral. Devolve a máscara.
///
/// Erros: `opcaoInvalida` (α fora de (0, 1)).
pub fn gesd(x: &[f64], max_outliers: usize, alpha: f64) -> Result<Vec<bool>, StatsError> {
    if !(alpha > 0.0 && alpha < 1.0) {
        return Err(StatsError::InvalidOption("α em (0, 1)"));
    }
    let mut ativos: Vec<usize> = (0..x.len()).filter(|&i| !x[i].is_nan()).collect();
    let mut removidos = Vec::new();
    let mut ultimo_sig = 0;
    for r in 1..=max_outliers {
        let n = ativos.len();
        if n < 3 {
            break;
        }
        let v: Vec<f64> = ativos.iter().map(|&i| x[i]).collect();
        let (m, s) = (
            mean(&v, NanFlag::Include),
            std(&v, VarNorm::Sample, NanFlag::Include),
        );
        if s == 0.0 {
            break;
        }
        let (pos, &i) = ativos
            .iter()
            .enumerate()
            .max_by(|a, b| (x[*a.1] - m).abs().total_cmp(&(x[*b.1] - m).abs()))
            .expect("não vazio");
        let estat = (x[i] - m).abs() / s;
        removidos.push(i);
        ativos.remove(pos);
        if estat > grubbs_critical(n, alpha) {
            ultimo_sig = r;
        }
    }
    let mut mask = vec![false; x.len()];
    for &i in &removidos[..ultimo_sig] {
        mask[i] = true;
    }
    Ok(mask)
}

#[cfg(test)]
mod tests {
    use super::grubbs_critical;

    /// Valores críticos bilaterais de Grubbs (α = 0,05) calculados com mpmath em 30 dígitos
    /// (quantil exato da t de Student); coincidem com as tabelas publicadas (1,1543; 2,290;
    /// 2,708; 3,384).
    #[test]
    fn valor_critico_de_grubbs() {
        for (n, g) in [
            (3, 1.154_304_851_344_04),
            (10, 2.289_954_084_479_6),
            (20, 2.708_245_645_805_75),
            (100, 3.384_082_901_154_89),
        ] {
            assert!(
                (grubbs_critical(n, 0.05) - g).abs() <= 1e-9,
                "{n}: {}",
                grubbs_critical(n, 0.05)
            );
        }
    }
}
