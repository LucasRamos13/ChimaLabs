//! Detecção de mudanças abruptas: segmentação ótima com penalidade por PELT
//! (R. Killick, P. Fearnhead & I. A. Eckley, J. Am. Stat. Assoc. 107 (2012) 1590–1598).
//!
//! Minimiza Σ custo(segmento) + β·(número de mudanças) sobre todas as partições. O custo de um
//! segmento depende do que muda:
//! - média: soma dos quadrados dos desvios da média do segmento;
//! - média e variância (normal): k·ln(σ̂²) com σ̂² a variância do segmento (piso para evitar ln 0);
//! - reta (inclinação e intercepto): resíduo quadrático do ajuste linear no segmento.
//!
//! As somas acumuladas tornam cada custo O(1). A poda de PELT descarta candidatos que nunca mais
//! podem ser ótimos, sem perder a otimalidade da partição ótima (Jackson et al., IEEE Signal
//! Process. Lett. 12, 2005).

use super::StatsError;

/// O que muda entre os segmentos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChangeKind {
    /// Média (padrão).
    #[default]
    Mean,
    /// Média e variância (verossimilhança normal).
    MeanVariance,
    /// Reta: inclinação e intercepto.
    Linear,
}

struct Acumulados {
    s1: Vec<f64>,
    s2: Vec<f64>,
    // Para a reta: Σt, Σt², Σt·y (t = índice).
    st: Vec<f64>,
    stt: Vec<f64>,
    sty: Vec<f64>,
}

impl Acumulados {
    fn new(y: &[f64]) -> Self {
        let n = y.len();
        let mut a = Self {
            s1: vec![0.0; n + 1],
            s2: vec![0.0; n + 1],
            st: vec![0.0; n + 1],
            stt: vec![0.0; n + 1],
            sty: vec![0.0; n + 1],
        };
        for (i, &v) in y.iter().enumerate() {
            let t = i as f64;
            a.s1[i + 1] = a.s1[i] + v;
            a.s2[i + 1] = a.s2[i] + v * v;
            a.st[i + 1] = a.st[i] + t;
            a.stt[i + 1] = a.stt[i] + t * t;
            a.sty[i + 1] = a.sty[i] + t * v;
        }
        a
    }

    /// Custo do segmento y[a..b].
    fn custo(&self, kind: ChangeKind, a: usize, b: usize) -> f64 {
        let k = (b - a) as f64;
        let s1 = self.s1[b] - self.s1[a];
        let s2 = self.s2[b] - self.s2[a];
        let ss = (s2 - s1 * s1 / k).max(0.0);
        match kind {
            ChangeKind::Mean => ss,
            ChangeKind::MeanVariance => {
                let var = (ss / k).max(1e-12 * (s2 / k).max(f64::MIN_POSITIVE));
                k * var.ln()
            }
            ChangeKind::Linear => {
                if b - a < 2 {
                    return 0.0;
                }
                let st = self.st[b] - self.st[a];
                let stt = self.stt[b] - self.stt[a];
                let sty = self.sty[b] - self.sty[a];
                let sxx = stt - st * st / k;
                let sxy = sty - st * s1 / k;
                (ss - if sxx > 0.0 { sxy * sxy / sxx } else { 0.0 }).max(0.0)
            }
        }
    }
}

/// Índices (0-based) onde começa cada novo segmento, pela penalidade β por mudança e com
/// segmentos de pelo menos `min_len` pontos.
///
/// Erros: `opcaoInvalida` (β negativo ou não finito, min_len = 0, dados com NaN).
pub fn changepoints(
    y: &[f64],
    kind: ChangeKind,
    penalty: f64,
    min_len: usize,
) -> Result<Vec<usize>, StatsError> {
    pelt(y, kind, penalty, min_len, true)
}

/// Igual a [`changepoints`], sem a poda (partição ótima por programação dinâmica O(n²)); serve de
/// referência nos testes e para dados pequenos.
pub fn changepoints_exact(
    y: &[f64],
    kind: ChangeKind,
    penalty: f64,
    min_len: usize,
) -> Result<Vec<usize>, StatsError> {
    pelt(y, kind, penalty, min_len, false)
}

fn pelt(
    y: &[f64],
    kind: ChangeKind,
    penalty: f64,
    min_len: usize,
    podar: bool,
) -> Result<Vec<usize>, StatsError> {
    if !(penalty.is_finite() && penalty >= 0.0) || min_len == 0 {
        return Err(StatsError::InvalidOption("penalidade finita ≥ 0 e min_len ≥ 1"));
    }
    if y.iter().any(|v| !v.is_finite()) {
        return Err(StatsError::InvalidOption("dados finitos (sem NaN/±∞)"));
    }
    let n = y.len();
    if n < 2 * min_len {
        return Ok(Vec::new());
    }
    let ac = Acumulados::new(y);
    // A poda exige C(a, b) + C(b, c) ≤ C(a, c) (constante K = 0 de Killick et al.), o que vale
    // para somas de quadrados (média e reta), mas não para a verossimilhança da variância: ali o
    // cálculo é a programação dinâmica completa.
    let podar = podar && kind != ChangeKind::MeanVariance;
    // f[t] = melhor custo para y[0..t]; ult[t] = início do último segmento.
    let mut f = vec![f64::INFINITY; n + 1];
    let mut ult = vec![0usize; n + 1];
    f[0] = -penalty;
    // Candidatos (início, instante a partir do qual pode sair). A poda de Killick et al. supõe
    // que o segmento (t, T) seja admissível; com comprimento mínimo, isso só vale para
    // T ≥ t + min_len. Por isso um candidato marcado em t só sai em t + min_len − 1
    // (regressão: com a remoção imediata, o proptest achou partições diferentes da ótima).
    let mut candidatos: Vec<(usize, Option<usize>)> = vec![(0, None)];
    for t in 1..=n {
        candidatos.retain(|&(_, exp)| exp.is_none_or(|e| e >= t));
        let mut melhor = f64::INFINITY;
        let mut arg = 0;
        let mut custos = Vec::with_capacity(candidatos.len());
        for &(s, _) in &candidatos {
            if t - s < min_len || f[s].is_infinite() {
                custos.push(None);
                continue;
            }
            let c = f[s] + ac.custo(kind, s, t) + penalty;
            custos.push(Some(c));
            // Desempate pelo maior s: igual nas duas variantes (com e sem poda).
            if c < melhor || (c == melhor && s > arg) {
                melhor = c;
                arg = s;
            }
        }
        f[t] = melhor;
        ult[t] = arg;
        if podar && melhor.is_finite() {
            for (cand, c) in candidatos.iter_mut().zip(&custos) {
                if cand.1.is_none() && c.is_some_and(|c| c - penalty > melhor) {
                    cand.1 = Some(t + min_len - 1);
                }
            }
        }
        // Um segmento novo pode começar em t (se y[0..t] tem partição válida).
        if f[t].is_finite() {
            candidatos.push((t, None));
        }
    }
    let mut cps = Vec::new();
    let mut t = n;
    while t > 0 {
        let s = ult[t];
        if s > 0 {
            cps.push(s);
        }
        t = s;
    }
    cps.reverse();
    Ok(cps)
}
