//! Equações diferenciais ordinárias (funcionalidade F049, RF-049; DT-26).
//!
//! [`solve_ivp`] integra y' = f(t, y) a partir de y(t₀) = y₀, com passo variável controlado
//! pelas tolerâncias, saída densa e eventos.
//!
//! Métodos ([`Method`]):
//! - [`Method::Dopri5`] (padrão): Dormand–Prince 5(4), saída densa de ordem 4;
//! - [`Method::Bs23`]: Bogacki–Shampine 3(2), para tolerâncias frouxas ou f pouco suave;
//! - [`Method::Ndf`] e [`Method::Bdf`]: fórmulas implícitas de ordem variável 1–5 para problemas
//!   rígidos, com Jacobiano fornecido ([`OdeOptions::jacobian`]) ou por diferenças finitas;
//! - [`Method::Radau`]: Radau IIA de ordem 5, implícito e L-estável, para problemas rígidos com
//!   tolerância apertada e para M·y' = f com matriz de massa ([`OdeOptions::mass`]), inclusive
//!   singular (DAE de índice 1).
//! - [`Method::Rosenbrock`]: Rosenbrock modificado 2(3), linearmente implícito (uma LU por
//!   passo, sem Newton), para problemas rígidos com tolerâncias folgadas.
//!
//! Fontes: E. Hairer, S. P. Nørsett & G. Wanner, *Solving Ordinary Differential Equations I*,
//! 2.ª ed. (1993), §II.4 (passo inicial e controle de erro), §II.6 (saída densa) e §IV.2 do
//! vol. II (controle PI); L. F. Shampine & M. W. Reichelt, SIAM J. Sci. Comput. 18 (1997), para a
//! semântica de eventos.

mod bdf;
mod bvp;
mod dde;
mod pde1;
mod radau;
mod rk;
mod rosenbrock;

use std::fmt;

pub use bdf::JacFn;
pub use bvp::{BvpOptions, BvpSolution, bvp, bvp_params};
pub use dde::{dde, dde_var};
pub use pde1::{BcFn, PdeFn, pde1};

use crate::optim::{self, Start};

/// f(t, y, dy).
pub(crate) type Rhs<'r> = dyn FnMut(f64, &[f64], &mut [f64]) + 'r;

/// Interpolante de um passo aceito.
#[derive(Clone, Debug)]
pub(crate) enum Interp {
    /// Runge–Kutta: y(t + θh) pelos coeficientes r1..r5.
    Rk { t: f64, h: f64, coeffs: Vec<f64> },
    /// BDF: polinômio das diferenças D₀..D_k, com t o fim do passo.
    Bdf {
        t: f64,
        h: f64,
        order: usize,
        d: Vec<f64>,
    },
    /// Radau: polinômio de colocação pelos incrementos z dos estágios.
    Radau {
        t: f64,
        h: f64,
        y: Vec<f64>,
        z: Vec<f64>,
    },
}

impl Interp {
    fn eval(&self, tau: f64, out: &mut [f64]) {
        match self {
            Self::Rk { t, h, coeffs } => {
                rk::dense_eval(coeffs, ((tau - t) / h).clamp(0.0, 1.0), out);
            }
            Self::Radau { t, h, y, z } => {
                radau::collocation_eval(y, z, ((tau - t) / h).clamp(0.0, 1.0), out);
            }
            Self::Bdf { t, h, order, d } => {
                // y(τ) = D₀ + Σⱼ Dⱼ₊₁·Π_{m≤j} (τ − (t − m·h))/((m + 1)·h).
                let n = out.len();
                out.copy_from_slice(&d[..n]);
                let mut p = 1.0;
                for j in 0..*order {
                    p *= (tau - (t - j as f64 * h)) / ((j as f64 + 1.0) * h);
                    for (c, o) in out.iter_mut().enumerate() {
                        *o += p * d[(j + 1) * n + c];
                    }
                }
            }
        }
    }
}

/// Resultado de uma tentativa de passo.
pub(crate) enum Outcome {
    Accepted(Interp),
    Failed(OdeStatus),
}

/// Integrador: controla o próprio passo e só devolve passos aceitos.
pub(crate) trait Stepper {
    fn t(&self) -> f64;
    fn y(&self) -> &[f64];
    fn step(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) -> Outcome;
    /// Instante em que o integrador detectou rigidez (só os explícitos que sabem detectar).
    fn stiffness_detected(&self) -> Option<f64> {
        None
    }
}

/// Método de integração.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Method {
    /// Dormand–Prince 5(4), explícito (padrão).
    #[default]
    Dopri5,
    /// Bogacki–Shampine 3(2), explícito.
    Bs23,
    /// Fórmulas numéricas de diferenciação (NDF) de ordem 1–5, implícito, para problemas
    /// rígidos (Shampine & Reichelt, 1997).
    Ndf,
    /// Fórmulas de diferenças regressivas (BDF) de ordem 1–5, implícito, para problemas rígidos.
    Bdf,
    /// Radau IIA de 3 estágios (ordem 5), implícito e L-estável; aceita matriz de massa.
    Radau,
    /// Rosenbrock modificado 2(3), linearmente implícito e L-estável: problemas rígidos com
    /// tolerâncias folgadas (Shampine & Reichelt, 1997).
    Rosenbrock,
}

impl Method {
    fn tableau(self) -> Option<&'static rk::Tableau> {
        match self {
            Self::Dopri5 => Some(&rk::DOPRI5),
            Self::Bs23 => Some(&rk::BS23),
            Self::Ndf | Self::Bdf | Self::Radau | Self::Rosenbrock => None,
        }
    }
}

/// Tolerância absoluta: a mesma para todas as componentes ou uma por componente.
#[derive(Clone, Debug, PartialEq)]
pub enum AbsTol {
    /// Igual para todas.
    Scalar(f64),
    /// Uma por componente.
    Vector(Vec<f64>),
}

/// Sentido de cruzamento que dispara um evento (ao longo da integração).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Qualquer mudança de sinal.
    #[default]
    Both,
    /// g passa de negativo a não negativo.
    Rising,
    /// g passa de positivo a não positivo.
    Falling,
}

/// g(t, y) de um evento.
pub type EventFn<'a> = Box<dyn FnMut(f64, &[f64]) -> f64 + 'a>;

/// Função de evento g(t, y): o evento ocorre quando g muda de sinal.
pub struct Event<'a> {
    /// g(t, y).
    pub g: EventFn<'a>,
    /// Sentido.
    pub direction: Direction,
    /// Encerra a integração no instante do evento.
    pub terminal: bool,
}

impl<'a> Event<'a> {
    /// Evento não terminal em qualquer sentido.
    pub fn new(g: impl FnMut(f64, &[f64]) -> f64 + 'a) -> Self {
        Self {
            g: Box::new(g),
            direction: Direction::Both,
            terminal: false,
        }
    }

    /// Torna o evento terminal.
    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }

    /// Define o sentido.
    pub fn direction(mut self, d: Direction) -> Self {
        self.direction = d;
        self
    }
}

impl fmt::Debug for Event<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("direction", &self.direction)
            .field("terminal", &self.terminal)
            .finish_non_exhaustive()
    }
}

/// Opções. `None` usa o padrão documentado.
#[derive(Default)]
pub struct OdeOptions<'a> {
    /// Método (padrão [`Method::Dopri5`]).
    pub method: Method,
    /// Tolerância relativa (padrão 1e-6; mínimo efetivo 100·ε_máq, com aviso).
    pub rel_tol: Option<f64>,
    /// Tolerância absoluta (padrão 1e-9).
    pub abs_tol: Option<AbsTol>,
    /// Passo inicial (padrão: estimado pelo algoritmo de Hairer).
    pub initial_step: Option<f64>,
    /// Passo máximo (padrão: |t_f − t₀|).
    pub max_step: Option<f64>,
    /// Máximo de passos aceitos (padrão: sem limite).
    pub max_steps: Option<usize>,
    /// Pontos de saída por passo quando `tspan` tem dois instantes (padrão 4 no DOPRI5, 1 no BS23).
    pub refine: Option<usize>,
    /// Guardar a solução contínua ([`OdeSolution::eval`]).
    pub dense: bool,
    /// Eventos.
    pub events: Vec<Event<'a>>,
    /// Métodos implícitos: Jacobiano ∂f/∂y (padrão: diferenças finitas).
    pub jacobian: Option<JacFn<'a>>,
    /// NDF e BDF: ordem máxima, de 1 a 5 (padrão 5).
    pub max_order: Option<usize>,
    /// Radau: matriz de massa M constante (n×n) do problema M·y' = f(t, y). Pode ser singular
    /// (DAE de índice 1); y₀ deve então ser consistente com as equações algébricas.
    pub mass: Option<crate::linalg::Matrix>,
}

impl fmt::Debug for OdeOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OdeOptions")
            .field("method", &self.method)
            .field("rel_tol", &self.rel_tol)
            .field("abs_tol", &self.abs_tol)
            .field("initial_step", &self.initial_step)
            .field("max_step", &self.max_step)
            .field("max_steps", &self.max_steps)
            .field("refine", &self.refine)
            .field("dense", &self.dense)
            .field("events", &self.events)
            .field("jacobian", &self.jacobian.is_some())
            .field("max_order", &self.max_order)
            .field("mass", &self.mass.as_ref().map(|m| m.shape()))
            .finish()
    }
}

/// Evento localizado.
#[derive(Clone, Debug, PartialEq)]
pub struct EventHit {
    /// Índice em [`OdeOptions::events`].
    pub index: usize,
    /// Instante.
    pub t: f64,
    /// Estado no instante.
    pub y: Vec<f64>,
}

/// Como a integração terminou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OdeStatus {
    /// Chegou a t_f.
    Completed,
    /// Um evento terminal encerrou a integração.
    Terminated,
    /// O passo ficou abaixo de 16·ε·|t| (singularidade provável); resultado parcial.
    StepTooSmall,
    /// f deu NaN ou ±∞ e o passo não pôde ser reduzido mais; resultado parcial.
    NonFinite,
    /// Atingiu [`OdeOptions::max_steps`]; resultado parcial.
    MaxSteps,
}

impl OdeStatus {
    /// Identificador estável do aviso correspondente (None se completou ou parou num evento).
    pub fn warning_id(self) -> Option<&'static str> {
        match self {
            Self::Completed | Self::Terminated => None,
            Self::StepTooSmall => Some("chimalab:ode:passoMinimo"),
            Self::NonFinite => Some("chimalab:ode:naoFinito"),
            Self::MaxSteps => Some("chimalab:ode:maximoDePassos"),
        }
    }
}

/// Avisos não fatais.
#[derive(Clone, Debug, PartialEq)]
pub enum OdeWarning {
    /// A tolerância relativa pedida estava abaixo de 100·ε_máq e foi elevada.
    RelTolRaised {
        /// Valor pedido.
        requested: f64,
        /// Valor usado.
        used: f64,
    },
    /// O problema parece rígido para o método explícito (DOPRI5): em 15 passos aceitos, h·ρ
    /// estimado passou do limite de estabilidade. Um método implícito (`Ndf`, `Radau`,
    /// `Rosenbrock`) deve ser bem mais eficiente.
    StiffnessSuspected {
        /// Instante em que a rigidez foi detectada.
        t: f64,
    },
}

/// Contadores.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OdeStats {
    /// Passos aceitos.
    pub steps: usize,
    /// Passos rejeitados.
    pub rejected: usize,
    /// Avaliações de f (inclusive as do Jacobiano por diferenças).
    pub fun_evals: usize,
    /// Jacobianos calculados (métodos implícitos).
    pub jac_evals: usize,
    /// Fatorações LU (métodos implícitos).
    pub lu_decomps: usize,
}

/// Resultado de [`solve_ivp`].
#[derive(Clone, Debug)]
pub struct OdeSolution {
    /// Instantes de saída.
    pub t: Vec<f64>,
    /// Estados nesses instantes.
    pub y: Vec<Vec<f64>>,
    /// Eventos localizados, em ordem de tempo.
    pub events: Vec<EventHit>,
    /// Término.
    pub status: OdeStatus,
    /// Contadores.
    pub stats: OdeStats,
    /// Avisos.
    pub warnings: Vec<OdeWarning>,
    dense: Option<Dense>,
}

#[derive(Clone, Debug)]
struct Dense {
    n: usize,
    /// Início de cada passo (crescente no sentido da integração).
    starts: Vec<f64>,
    /// Interpolante de cada passo.
    interps: Vec<Interp>,
}

impl OdeSolution {
    /// Estado em `t` pela saída densa. `None` sem [`OdeOptions::dense`] ou fora do intervalo
    /// integrado.
    pub fn eval(&self, t: f64) -> Option<Vec<f64>> {
        let d = self.dense.as_ref()?;
        let (first, last) = (*self.t.first()?, *self.t.last()?);
        let dir = if last >= first { 1.0 } else { -1.0 };
        if !(signed(dir, t - first) >= 0.0 && signed(dir, last - t) >= 0.0) {
            return None;
        }
        if t == last {
            return self.y.last().cloned();
        }
        let i = d
            .starts
            .partition_point(|&s| signed(dir, s - t) <= 0.0)
            .saturating_sub(1);
        let mut out = vec![0.0; d.n];
        d.interps[i].eval(t, &mut out);
        Some(out)
    }
}

/// Erros (argumentos inválidos; falhas durante a integração dão resultado parcial com
/// [`OdeStatus`]).
#[derive(Clone, Debug, PartialEq)]
pub enum OdeError {
    /// `tspan` com menos de 2 instantes, não finito, não monótono ou t₀ = t_f.
    InvalidTspan(&'static str),
    /// y₀ vazio ou não finito, ou f(t₀, y₀) não finito.
    InvalidStart(&'static str),
    /// Tolerância negativa, NaN ou infinita (ou relativa e absoluta ambas nulas).
    InvalidTolerance(&'static str),
    /// Opção inválida (passo, refine).
    InvalidOption(&'static str),
    /// Tolerância absoluta vetorial com tamanho diferente de y₀.
    ShapeMismatch {
        /// Esperado.
        expected: usize,
        /// Recebido.
        actual: usize,
    },
}

impl OdeError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::InvalidTspan(_) => "chimalab:ode:intervaloInvalido",
            Self::InvalidStart(_) => "chimalab:ode:condicaoInicialInvalida",
            Self::InvalidTolerance(_) => "chimalab:ode:toleranciaInvalida",
            Self::InvalidOption(_) => "chimalab:ode:opcaoInvalida",
            Self::ShapeMismatch { .. } => "chimalab:ode:formaIncompativel",
        }
    }
}

impl fmt::Display for OdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTspan(m) => write!(f, "intervalo de integração inválido: {m}"),
            Self::InvalidStart(m) => write!(f, "condição inicial inválida: {m}"),
            Self::InvalidTolerance(m) => write!(f, "tolerância inválida: {m}"),
            Self::InvalidOption(m) => write!(f, "opção inválida: {m}"),
            Self::ShapeMismatch { expected, actual } => {
                write!(
                    f,
                    "tolerância absoluta com {actual} elementos; esperado {expected}"
                )
            }
        }
    }
}

impl std::error::Error for OdeError {}

/// Integra y' = f(t, y), y(tspan[0]) = y0.
///
/// - `f(t, y, dy)` deve preencher todo `dy`.
/// - `tspan`: com dois instantes [t₀, t_f], a saída tem cada passo (mais pontos interpolados,
///   ver [`OdeOptions::refine`]); com mais, a saída é exatamente nesses instantes. Estritamente
///   monótono; decrescente integra para trás.
/// - Eventos: localizados com precisão total em t sobre a saída densa; um evento terminal
///   encerra a integração no instante dele, que entra na saída.
///
/// Falhas durante a integração (passo mínimo, f não finito, limite de passos) não são erros:
/// o resultado parcial vem com o [`OdeStatus`] correspondente.
pub fn solve_ivp(
    mut f: impl FnMut(f64, &[f64], &mut [f64]),
    tspan: &[f64],
    y0: &[f64],
    opts: &mut OdeOptions<'_>,
) -> Result<OdeSolution, OdeError> {
    if tspan.len() < 2 {
        return Err(OdeError::InvalidTspan("pelo menos dois instantes"));
    }
    if tspan.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidTspan("instantes não finitos"));
    }
    let (t0, tf) = (tspan[0], *tspan.last().unwrap_or(&tspan[0]));
    if t0 == tf {
        return Err(OdeError::InvalidTspan("t₀ = t_f"));
    }
    let dir = if tf > t0 { 1.0 } else { -1.0 };
    if tspan.windows(2).any(|w| signed(dir, w[1] - w[0]) <= 0.0) {
        return Err(OdeError::InvalidTspan(
            "instantes devem ser estritamente monótonos",
        ));
    }
    let n = y0.len();
    if n == 0 {
        return Err(OdeError::InvalidStart("estado vazio"));
    }
    if y0.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("y₀ não finito"));
    }
    let mut warnings = Vec::new();
    let rtol_req = opts.rel_tol.unwrap_or(1e-6);
    if !(rtol_req.is_finite() && rtol_req >= 0.0) {
        return Err(OdeError::InvalidTolerance("relativa deve ser finita e ≥ 0"));
    }
    let rtol_min = 100.0 * f64::EPSILON;
    let rtol = if rtol_req < rtol_min {
        warnings.push(OdeWarning::RelTolRaised {
            requested: rtol_req,
            used: rtol_min,
        });
        rtol_min
    } else {
        rtol_req
    };
    let atol: Vec<f64> = match &opts.abs_tol {
        None => vec![1e-9; n],
        Some(AbsTol::Scalar(a)) => vec![*a; n],
        Some(AbsTol::Vector(v)) if v.len() != n => {
            return Err(OdeError::ShapeMismatch {
                expected: n,
                actual: v.len(),
            });
        }
        Some(AbsTol::Vector(v)) => v.clone(),
    };
    if atol.iter().any(|a| !(a.is_finite() && *a >= 0.0)) {
        return Err(OdeError::InvalidTolerance("absoluta deve ser finita e ≥ 0"));
    }
    let span = (tf - t0).abs();
    let hmax = opts.max_step.unwrap_or(span);
    if !(hmax.is_finite() && hmax > 0.0) {
        return Err(OdeError::InvalidOption("max_step deve ser finito e > 0"));
    }
    if let Some(h) = opts.initial_step
        && !(h.is_finite() && h > 0.0)
    {
        return Err(OdeError::InvalidOption("initial_step deve ser finito e > 0"));
    }
    let tab = opts.method.tableau();
    let refine = opts.refine.unwrap_or(tab.map_or(1, |tb| tb.refine));
    if refine == 0 {
        return Err(OdeError::InvalidOption("refine deve ser ≥ 1"));
    }
    let max_order = opts.max_order.unwrap_or(5);
    if !(1..=5).contains(&max_order) {
        return Err(OdeError::InvalidOption("max_order deve estar entre 1 e 5"));
    }
    if let Some(m) = &opts.mass {
        if opts.method != Method::Radau {
            return Err(OdeError::InvalidOption("matriz de massa só com Method::Radau"));
        }
        if m.shape() != (n, n) {
            return Err(OdeError::ShapeMismatch {
                expected: n * n,
                actual: m.rows() * m.cols(),
            });
        }
        if m.as_slice().iter().any(|v| !v.is_finite()) {
            return Err(OdeError::InvalidOption("matriz de massa não finita"));
        }
    }
    let mut stats = OdeStats::default();
    let mut rhs = |t: f64, y: &[f64], dy: &mut [f64]| f(t, y, dy);
    let mut f0 = vec![0.0; n];
    rhs(t0, y0, &mut f0);
    stats.fun_evals += 1;
    if f0.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("f(t₀, y₀) não finito"));
    }
    let order = match (tab, opts.method) {
        (Some(tb), _) => tb.order,
        (None, Method::Radau) => 5,
        (None, Method::Rosenbrock) => 2,
        _ => 1,
    };
    let h0 = match opts.initial_step {
        Some(h) => h.min(hmax).min(span),
        // Com matriz de massa, f₀ não é y' (componentes algébricas): passo inicial pequeno, que o
        // controle aumenta rapidamente (Hairer & Wanner).
        None if opts.mass.is_some() => 1e-6f64.min(hmax).min(span),
        None => initial_step(
            &mut rhs,
            t0,
            y0,
            &f0,
            dir,
            order,
            hmax.min(span),
            rtol,
            &atol,
            &mut stats,
        ),
    };
    let mut stepper: Box<dyn Stepper + '_> = match tab {
        Some(tb) => Box::new(rk::RkStepper::new(tb, t0, y0, &f0, tf, h0, hmax, rtol, atol)),
        None if opts.method == Method::Radau => Box::new(radau::Radau::new(
            &mut rhs,
            t0,
            y0,
            &f0,
            tf,
            h0,
            hmax,
            rtol,
            atol,
            opts.mass.as_ref(),
            opts.jacobian.as_mut(),
            &mut stats,
        )),
        None if opts.method == Method::Rosenbrock => Box::new(rosenbrock::Rosenbrock::new(
            t0,
            y0,
            &f0,
            tf,
            h0,
            hmax,
            rtol,
            atol,
            opts.jacobian.as_mut(),
        )),
        None => Box::new(bdf::Bdf::new(
            &mut rhs,
            t0,
            y0,
            &f0,
            tf,
            h0,
            hmax,
            rtol,
            atol,
            opts.method == Method::Ndf,
            max_order,
            opts.jacobian.as_mut(),
            &mut stats,
        )),
    };

    let mut out_t = vec![t0];
    let mut out_y = vec![y0.to_vec()];
    let mut next_out = 1usize; // próximo índice de tspan (modo de instantes pedidos)
    let explicit_times = tspan.len() > 2;
    let mut dense = opts.dense.then(|| Dense {
        n,
        starts: Vec::new(),
        interps: Vec::new(),
    });
    let mut g_old: Vec<f64> = opts.events.iter_mut().map(|e| (e.g)(t0, y0)).collect();
    let mut hits: Vec<EventHit> = Vec::new();
    let mut y_old = y0.to_vec();
    let mut tmp = vec![0.0; n];
    let status;
    loop {
        if opts.max_steps.is_some_and(|m| stats.steps >= m) {
            status = OdeStatus::MaxSteps;
            break;
        }
        let t = stepper.t();
        y_old.copy_from_slice(stepper.y());
        let it = match stepper.step(&mut rhs, &mut stats) {
            Outcome::Accepted(it) => it,
            Outcome::Failed(st) => {
                status = st;
                break;
            }
        };
        let t_new = stepper.t();
        let y_new = stepper.y().to_vec();
        // Nos extremos do passo, o interpolante devolve exatamente y e y_novo: os sinais de g
        // vistos pelo localizador são os mesmos da detecção.
        let interp = |tau: f64, out: &mut [f64]| {
            if tau == t_new {
                out.copy_from_slice(&y_new);
            } else if tau == t {
                out.copy_from_slice(&y_old);
            } else {
                it.eval(tau, out);
            }
        };
        // Eventos do passo.
        let mut step_hits: Vec<(f64, usize)> = Vec::new();
        let mut g_new = Vec::with_capacity(opts.events.len());
        for (idx, ev) in opts.events.iter_mut().enumerate() {
            let gn = (ev.g)(t_new, &y_new);
            g_new.push(gn);
            let go = g_old[idx];
            let rising = go < 0.0 && gn >= 0.0;
            let falling = go > 0.0 && gn <= 0.0;
            let fires = match ev.direction {
                Direction::Both => rising || falling,
                Direction::Rising => rising,
                Direction::Falling => falling,
            };
            if !fires {
                continue;
            }
            let te = if gn == 0.0 {
                t_new
            } else {
                let mut buf = vec![0.0; n];
                let mut gfun = |tau: f64| {
                    interp(tau, &mut buf);
                    (ev.g)(tau, &buf)
                };
                let (lo, hi) = if t < t_new { (t, t_new) } else { (t_new, t) };
                // g muda de sinal entre os extremos (são os mesmos valores da detecção), então o
                // fzero não falha; o t_novo é só uma salvaguarda.
                optim::fzero(&mut gfun, Start::Interval(lo, hi), &mut optim::Options::default())
                    .map_or(t_new, |r| r.x)
            };
            step_hits.push((te, idx));
        }
        step_hits.sort_by(|a, b| signed(dir, a.0).total_cmp(&signed(dir, b.0)).then(a.1.cmp(&b.1)));
        let t_term = step_hits
            .iter()
            .find(|(_, idx)| opts.events[*idx].terminal)
            .map(|(te, _)| *te);
        for &(te, idx) in &step_hits {
            if t_term.is_some_and(|tt| signed(dir, te - tt) > 0.0) {
                break;
            }
            let mut ye = vec![0.0; n];
            interp(te, &mut ye);
            hits.push(EventHit {
                index: idx,
                t: te,
                y: ye,
            });
        }
        let t_end = t_term.unwrap_or(t_new);
        // Saída.
        if explicit_times {
            while next_out < tspan.len() && signed(dir, tspan[next_out] - t_end) <= 0.0 {
                let to = tspan[next_out];
                interp(to, &mut tmp);
                out_t.push(to);
                out_y.push(tmp.clone());
                next_out += 1;
            }
        } else {
            for j in 1..refine {
                let to = t + (t_new - t) * j as f64 / refine as f64;
                if signed(dir, to - t_end) >= 0.0 {
                    break;
                }
                interp(to, &mut tmp);
                out_t.push(to);
                out_y.push(tmp.clone());
            }
            if t_term.is_none() {
                out_t.push(t_new);
                out_y.push(y_new.clone());
            }
        }
        if let Some(tt) = t_term
            && out_t.last() != Some(&tt)
        {
            interp(tt, &mut tmp);
            out_t.push(tt);
            out_y.push(tmp.clone());
        }
        if let Some(d) = dense.as_mut() {
            d.starts.push(t);
            d.interps.push(it);
        }
        if t_term.is_some() {
            status = OdeStatus::Terminated;
            break;
        }
        g_old = g_new;
        if t_new == tf {
            status = OdeStatus::Completed;
            break;
        }
    }
    Ok(OdeSolution {
        t: out_t,
        y: out_y,
        events: hits,
        status,
        stats,
        warnings: {
            if let Some(t) = stepper.stiffness_detected() {
                warnings.push(OdeWarning::StiffnessSuspected { t });
            }
            warnings
        },
        dense,
    })
}

/// Passo inicial (Hairer, Nørsett & Wanner, vol. I, §II.4): h₀ pela razão ‖y₀‖/‖f₀‖, um passo
/// de Euler para estimar ‖y''‖ e h₁ = (0,01/max(‖f₀‖, ‖y''‖))^{1/(p+1)}.
#[allow(clippy::too_many_arguments)]
fn initial_step(
    f: &mut Rhs<'_>,
    t0: f64,
    y0: &[f64],
    f0: &[f64],
    dir: f64,
    order: usize,
    hlim: f64,
    rtol: f64,
    atol: &[f64],
    stats: &mut OdeStats,
) -> f64 {
    let n = y0.len();
    let rms = |v: &[f64]| -> f64 {
        (v.iter()
            .zip(y0)
            .zip(atol)
            .map(|((e, y), a)| {
                let sc = a + rtol * y.abs();
                if sc == 0.0 {
                    if *e == 0.0 { 0.0 } else { f64::INFINITY }
                } else {
                    (e / sc).powi(2)
                }
            })
            .sum::<f64>()
            / n as f64)
            .sqrt()
    };
    let d0 = rms(y0);
    let d1 = rms(f0);
    let h0 = if d0 < 1e-5 || d1 < 1e-5 {
        1e-6
    } else {
        0.01 * d0 / d1
    };
    let h0 = h0.min(hlim);
    let y1: Vec<f64> = (0..n).map(|i| y0[i] + signed(dir, h0) * f0[i]).collect();
    let mut f1 = vec![0.0; n];
    f(t0 + signed(dir, h0), &y1, &mut f1);
    stats.fun_evals += 1;
    let df: Vec<f64> = (0..n).map(|i| f1[i] - f0[i]).collect();
    let d2 = rms(&df) / h0;
    let m = d1.max(d2);
    let h1 = if !m.is_finite() {
        h0
    } else if m <= 1e-15 {
        (h0 * 1e-3).max(1e-6)
    } else {
        (0.01 / m).powf(1.0 / (order as f64 + 1.0))
    };
    (100.0 * h0).min(h1).min(hlim)
}

/// x no sentido da integração (`dir` = ±1): positivo quando x aponta para a frente.
fn signed(dir: f64, x: f64) -> f64 {
    dir * x
}
