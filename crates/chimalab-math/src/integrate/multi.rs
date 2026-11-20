//! Integrais duplas e triplas iteradas, e a dupla por ladrilhos ([`integral2_tiled`]).
//!
//! ∫∫ f = ∫ₓ (∫ᵧ f(x, y) dy) dx, com a integral interna feita por [`integral`] para cada x pedido
//! pela externa. A interna usa tolerâncias 10× menores, para que o seu erro não perturbe a
//! estimativa de erro da externa. A cota de erro devolvida soma o erro da integral externa e
//! |xb − xa|·(maior erro interno); em intervalos externos infinitos, só o externo.

use std::cell::Cell;

use chimalab_array::Scalar;

use super::IntegrateError;
use super::adaptive::{IntegralOptions, IntegralResult, IntegralWarning, integral};
use super::gk::gk15_nodes;

/// Estado compartilhado entre a integral externa e as internas.
struct Inner {
    error: Cell<Option<IntegrateError>>,
    warning: Cell<Option<IntegralWarning>>,
    max_error: Cell<f64>,
    evaluations: Cell<usize>,
}

impl Inner {
    fn new() -> Self {
        Self {
            error: Cell::new(None),
            warning: Cell::new(None),
            max_error: Cell::new(0.0),
            evaluations: Cell::new(0),
        }
    }

    /// Registra o resultado de uma integral interna e devolve o seu valor (zero se falhou).
    fn record<T: Scalar>(&self, r: Result<IntegralResult<T>, IntegrateError>) -> T {
        match r {
            Ok(r) => {
                self.max_error.set(self.max_error.get().max(r.error_bound));
                self.evaluations.set(self.evaluations.get() + r.evaluations);
                if self.warning.get().is_none() {
                    self.warning.set(r.warning);
                }
                r.value
            }
            Err(e) => {
                let first = self.error.take().unwrap_or(e);
                self.error.set(Some(first));
                T::zero()
            }
        }
    }

    /// Combina o resultado externo (sobre [xa, xb]) com o estado das internas.
    fn finish<T: Scalar>(
        self,
        outer: Result<IntegralResult<T>, IntegrateError>,
        xa: f64,
        xb: f64,
    ) -> Result<IntegralResult<T>, IntegrateError> {
        let mut r = outer?;
        if let Some(e) = self.error.take() {
            return Err(e);
        }
        let length = (xb - xa).abs();
        if length.is_finite() {
            r.error_bound += length * self.max_error.get();
        }
        r.evaluations = self.evaluations.get();
        r.warning = r.warning.or(self.warning.get());
        Ok(r)
    }
}

fn tighter(opts: &IntegralOptions) -> IntegralOptions {
    IntegralOptions {
        abs_tol: 0.1 * opts.abs_tol,
        rel_tol: 0.1 * opts.rel_tol,
        max_intervals: opts.max_intervals,
        waypoints: vec![],
    }
}

/// ∫_{xa}^{xb} ∫_{ya(x)}^{yb(x)} f(x, y) dy dx. Os pontos de quebra das opções valem para x.
/// Erros de limite nas integrais internas (por exemplo ya(x) NaN) são devolvidos como erro.
pub fn integral2<T: Scalar>(
    mut f: impl FnMut(f64, f64) -> T,
    xa: f64,
    xb: f64,
    ya: impl Fn(f64) -> f64,
    yb: impl Fn(f64) -> f64,
    opts: &IntegralOptions,
) -> Result<IntegralResult<T>, IntegrateError> {
    let inner_opts = tighter(opts);
    let inner = Inner::new();
    let outer = integral(
        |x| inner.record(integral(|y| f(x, y), ya(x), yb(x), &inner_opts)),
        xa,
        xb,
        opts,
    );
    inner.finish(outer, xa, xb)
}

/// ∫_{xa}^{xb} ∫_{ya(x)}^{yb(x)} ∫_{za(x,y)}^{zb(x,y)} f(x, y, z) dz dy dx.
#[allow(clippy::too_many_arguments)]
pub fn integral3<T: Scalar>(
    mut f: impl FnMut(f64, f64, f64) -> T,
    xa: f64,
    xb: f64,
    ya: impl Fn(f64) -> f64,
    yb: impl Fn(f64) -> f64,
    za: impl Fn(f64, f64) -> f64,
    zb: impl Fn(f64, f64) -> f64,
    opts: &IntegralOptions,
) -> Result<IntegralResult<T>, IntegrateError> {
    let inner_opts = tighter(opts);
    let inner = Inner::new();
    let outer = integral(
        |x| {
            inner.record(integral2(
                |y, z| f(x, y, z),
                ya(x),
                yb(x),
                |y| za(x, y),
                |y| zb(x, y),
                &inner_opts,
            ))
        },
        xa,
        xb,
        opts,
    );
    inner.finish(outer, xa, xb)
}

/// Mudança de variável cúbica em [0, 1], s = 3u² − 2u³ (a de [`integral`] em [−1, 1], escalada):
/// ds/du = 6u(1 − u) se anula nas bordas e suaviza singularidades de raiz ali.
fn suaviza(u: f64) -> (f64, f64) {
    (u * u * (3.0 - 2.0 * u), 6.0 * u * (1.0 - u))
}

/// ∫_{xa}^{xb} ∫_{ya(x)}^{yb(x)} f(x, y) dy dx pelo método de ladrilhos.
///
/// A região é levada ao quadrado unitário por x = xa + (xb − xa)·s e
/// y = ya(x) + (yb(x) − ya(x))·t, com jacobiano (xb − xa)·(yb(x) − ya(x)). Em s e t aplica-se a
/// mudança cúbica s = 3u² − 2u³, que enfraquece singularidades nas bordas: no disco, √(1 − x²)
/// vira suave em u. Regressão: sem ela, o erro estimado subestimava o real em 20×. Cada ladrilho
/// recebe a
/// regra produto de Kronrod 15×15 (225 avaliações), e o erro é |K⊗K − G⊗G|. O ladrilho de maior
/// erro é dividido em 4 até o erro total ficar abaixo de max(abs_tol, rel_tol·|Q|), ou até
/// `max_intervals` ladrilhos.
///
/// Comparado com o iterado ([`integral2`]): avalia f em blocos e não aninha tolerâncias. É mais
/// eficiente para integrandos suaves ou com picos localizados (medido no pico gaussiano: 2,3×
/// menos avaliações). Só limites finitos.
///
/// Erros: `limitesInvalidos` (limites não finitos, ou ya(x)/yb(x) não finito em algum nó),
/// `toleranciaInvalida`.
pub fn integral2_tiled<T: Scalar>(
    mut f: impl FnMut(f64, f64) -> T,
    xa: f64,
    xb: f64,
    ya: impl Fn(f64) -> f64,
    yb: impl Fn(f64) -> f64,
    opts: &IntegralOptions,
) -> Result<IntegralResult<T>, IntegrateError> {
    let ok = |v: f64| v.is_finite() && v >= 0.0;
    if !ok(opts.abs_tol) {
        return Err(IntegrateError::InvalidTolerance("abs_tol deve ser finita e ≥ 0"));
    }
    if !ok(opts.rel_tol) {
        return Err(IntegrateError::InvalidTolerance("rel_tol deve ser finita e ≥ 0"));
    }
    if !(xa.is_finite() && xb.is_finite()) {
        return Err(IntegrateError::InvalidLimits(
            "integral2_tiled: limites em x finitos",
        ));
    }
    let (nos, wk, wg) = gk15_nodes();
    let avaliacoes = Cell::new(0usize);
    let limite_ruim = Cell::new(false);
    let nao_finito = Cell::new(false);
    // Regra produto num ladrilho [s0, s1] × [t0, t1] do quadrado unitário: (K, G).
    let mut regra = |s0: f64, s1: f64, t0: f64, t1: f64| -> (T, T) {
        let (hs, ht) = (0.5 * (s1 - s0), 0.5 * (t1 - t0));
        let (cs, ct) = (0.5 * (s0 + s1), 0.5 * (t0 + t1));
        let (mut k, mut g) = (T::zero(), T::zero());
        for i in 0..15 {
            let u = cs + hs * nos[i];
            let (s, ds) = suaviza(u);
            let x = xa + (xb - xa) * s;
            let (y0, y1) = (ya(x), yb(x));
            if !(y0.is_finite() && y1.is_finite()) {
                limite_ruim.set(true);
                continue;
            }
            let jac = (xb - xa) * (y1 - y0) * ds;
            let (mut ki, mut gi) = (T::zero(), T::zero());
            for j in 0..15 {
                let (t, dt) = suaviza(ct + ht * nos[j]);
                let v = f(x, y0 + (y1 - y0) * t).scale(dt);
                avaliacoes.set(avaliacoes.get() + 1);
                if !(v.abs().is_finite()) {
                    nao_finito.set(true);
                }
                ki += v.scale(wk[j]);
                if wg[j] != 0.0 {
                    gi += v.scale(wg[j]);
                }
            }
            k += ki.scale(wk[i] * jac);
            if wg[i] != 0.0 {
                g += gi.scale(wg[i] * jac);
            }
        }
        (k.scale(hs * ht), g.scale(hs * ht))
    };
    // Ladrilhos: (s0, s1, t0, t1, valor, erro). Começa com 2 × 2.
    let mut ladrilhos: Vec<(f64, f64, f64, f64, T, f64)> = Vec::new();
    for (s0, s1) in [(0.0, 0.5), (0.5, 1.0)] {
        for (t0, t1) in [(0.0, 0.5), (0.5, 1.0)] {
            let (k, g) = regra(s0, s1, t0, t1);
            ladrilhos.push((s0, s1, t0, t1, k, (k - g).abs()));
        }
    }
    let mut aviso = None;
    loop {
        if limite_ruim.get() {
            return Err(IntegrateError::InvalidLimits(
                "integral2_tiled: ya(x) ou yb(x) não finito",
            ));
        }
        let q: T = ladrilhos.iter().fold(T::zero(), |a, l| a + l.4);
        let e: f64 = ladrilhos.iter().map(|l| l.5).sum();
        if nao_finito.get() || !(q.abs() + e).is_finite() {
            aviso = Some(IntegralWarning::NonFinite);
            return Ok(IntegralResult {
                value: q,
                error_bound: e,
                evaluations: avaliacoes.get(),
                warning: aviso,
            });
        }
        if e <= opts.abs_tol.max(opts.rel_tol * q.abs()) {
            return Ok(IntegralResult {
                value: q,
                error_bound: e,
                evaluations: avaliacoes.get(),
                warning: aviso,
            });
        }
        if ladrilhos.len() + 3 > opts.max_intervals.max(4) {
            aviso = Some(IntegralWarning::MaxIntervals);
            return Ok(IntegralResult {
                value: q,
                error_bound: e,
                evaluations: avaliacoes.get(),
                warning: aviso,
            });
        }
        let (im, _) = ladrilhos
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.5.total_cmp(&b.1.5))
            .expect("não vazio");
        let (s0, s1, t0, t1, _, _) = ladrilhos.swap_remove(im);
        let (sm, tm) = (0.5 * (s0 + s1), 0.5 * (t0 + t1));
        if !(s0 < sm && sm < s1 && t0 < tm && tm < t1) {
            // Ladrilho no limite de resolução: devolve o que há, com aviso.
            let (k, g) = regra(s0, s1, t0, t1);
            ladrilhos.push((s0, s1, t0, t1, k, (k - g).abs()));
            let q: T = ladrilhos.iter().fold(T::zero(), |a, l| a + l.4);
            let e: f64 = ladrilhos.iter().map(|l| l.5).sum();
            return Ok(IntegralResult {
                value: q,
                error_bound: e,
                evaluations: avaliacoes.get(),
                warning: Some(IntegralWarning::MinimumStep),
            });
        }
        for (a, b) in [(s0, sm), (sm, s1)] {
            for (c, d) in [(t0, tm), (tm, t1)] {
                let (k, g) = regra(a, b, c, d);
                ladrilhos.push((a, b, c, d, k, (k - g).abs()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(error_bound: f64, warning: Option<IntegralWarning>) -> IntegralResult<f64> {
        IntegralResult {
            value: 1.0,
            error_bound,
            evaluations: 7,
            warning,
        }
    }

    #[test]
    fn combina_erros_avaliacoes_e_avisos() {
        let inner = Inner::new();
        assert_eq!(inner.record(Ok(res(0.5, None))), 1.0);
        assert_eq!(
            inner.record(Ok(res(2.0, Some(IntegralWarning::MaxIntervals)))),
            1.0
        );
        assert_eq!(inner.record(Ok(res(1.0, Some(IntegralWarning::NonFinite)))), 1.0);
        let r = inner.finish(Ok(res(0.25, None)), 1.0, 4.0).unwrap();
        // 0,25 + |4 − 1|·2 = 6,25; avaliações = 3·7; o primeiro aviso interno.
        assert_eq!(r.error_bound, 6.25);
        assert_eq!(r.evaluations, 21);
        assert_eq!(r.warning, Some(IntegralWarning::MaxIntervals));
        // O aviso externo prevalece; intervalo infinito não soma o termo interno.
        let inner = Inner::new();
        inner.record(Ok(res(3.0, Some(IntegralWarning::MaxIntervals))));
        let r = inner
            .finish(
                Ok(res(0.5, Some(IntegralWarning::MinimumStep))),
                2.0,
                f64::INFINITY,
            )
            .unwrap();
        assert_eq!(
            (r.error_bound, r.warning),
            (0.5, Some(IntegralWarning::MinimumStep))
        );
        // O primeiro erro interno é o devolvido.
        let inner = Inner::new();
        assert_eq!(inner.record::<f64>(Err(IntegrateError::InvalidLimits("a"))), 0.0);
        inner.record::<f64>(Err(IntegrateError::InvalidLimits("b")));
        assert_eq!(
            inner.finish(Ok(res(0.0, None)), 0.0, 1.0),
            Err(IntegrateError::InvalidLimits("a"))
        );
        // Erro externo tem precedência.
        let inner = Inner::new();
        let e = IntegrateError::InvalidTolerance("x");
        assert_eq!(inner.finish::<f64>(Err(e.clone()), 0.0, 1.0), Err(e));
    }
}
