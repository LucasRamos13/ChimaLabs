//! Otimização básica e raízes (funcionalidade F051, RF-051; desenho em DT-24).
//!
//! - [`fzero`]: zero de função escalar a partir de um intervalo com mudança de sinal ou de um
//!   chute (busca do intervalo dos dois lados). Brent–Dekker por padrão, ITP como opção.
//! - [`fminbnd`]: mínimo de função escalar num intervalo (Brent: seção áurea + parábola).
//! - [`fminsearch`]: mínimo sem restrições pelo simplex de Nelder–Mead (regras de Lagarias et al.,
//!   parâmetros adaptativos opcionais, reinício contra estagnação).
//! - [`nnls`]: mínimos quadrados com x ≥ 0 (Lawson–Hanson).
//!
//! Todas aceitam [`Options`]: tolerâncias, limites, histórico e uma função de saída chamada a cada
//! iteração, que pode interromper. O resultado traz um [`ExitFlag`] padronizado.

mod fminbnd;
mod fminsearch;
mod fzero;
mod nnls;

pub use fminbnd::fminbnd;
pub use fminsearch::fminsearch;
pub use fzero::{Start, ZeroMethod, fzero};
pub use nnls::{NnlsResult, nnls};

use std::fmt;

/// Etapa executada numa iteração (exibição iterativa e gráficos).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Procedure {
    /// Avaliação inicial.
    Initial,
    /// Busca do intervalo com mudança de sinal (zero com chute escalar).
    Search,
    /// Bissecção.
    Bisection,
    /// Secante ou interpolação quadrática inversa (Brent).
    Interpolation,
    /// Passo ITP.
    Itp,
    /// Seção áurea.
    Golden,
    /// Interpolação parabólica.
    Parabolic,
    /// Extremo do intervalo melhor que o ponto interior (mínimo limitado).
    Boundary,
    /// Nelder–Mead: reflexão.
    Reflect,
    /// Nelder–Mead: expansão.
    Expand,
    /// Nelder–Mead: contração externa.
    ContractOutside,
    /// Nelder–Mead: contração interna.
    ContractInside,
    /// Nelder–Mead: encolhimento.
    Shrink,
    /// Nelder–Mead: reinício a partir do melhor ponto.
    Restart,
    /// NNLS: variável entra no conjunto livre.
    Add,
    /// NNLS: variáveis voltam a zero.
    Remove,
}

impl Procedure {
    /// Nome curto para a exibição iterativa.
    pub fn name(self) -> &'static str {
        match self {
            Self::Initial => "inicial",
            Self::Search => "busca",
            Self::Bisection => "bissecção",
            Self::Interpolation => "interpolação",
            Self::Itp => "itp",
            Self::Golden => "áurea",
            Self::Parabolic => "parabólica",
            Self::Boundary => "extremo",
            Self::Reflect => "reflexão",
            Self::Expand => "expansão",
            Self::ContractOutside => "contração externa",
            Self::ContractInside => "contração interna",
            Self::Shrink => "encolhimento",
            Self::Restart => "reinício",
            Self::Add => "entra",
            Self::Remove => "sai",
        }
    }
}

/// Estado de uma iteração, passado à função de saída e guardado no histórico.
#[derive(Clone, Debug, PartialEq)]
pub struct Iteration {
    /// Número da iteração (0 = avaliação inicial).
    pub iteration: usize,
    /// Avaliações da função até aqui.
    pub fun_evals: usize,
    /// Melhor ponto atual.
    pub x: Vec<f64>,
    /// Valor no melhor ponto (resíduo ao quadrado no NNLS).
    pub fval: f64,
    /// Etapa executada.
    pub procedure: Procedure,
    /// Intervalo atual (zero e mínimo escalares).
    pub interval: Option<(f64, f64)>,
}

impl fmt::Display for Iteration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>6} {:>8} {:>24.16e} {:>24.16e}  {}",
            self.iteration,
            self.fun_evals,
            self.x.first().copied().unwrap_or(f64::NAN),
            self.fval,
            self.procedure.name()
        )
    }
}

/// Resposta da função de saída.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    /// Continuar.
    Continue,
    /// Interromper; o resultado sai com [`ExitFlag::Stopped`].
    Stop,
}

/// Função de saída: chamada após cada iteração.
pub type OutputFn<'a> = Box<dyn FnMut(&Iteration) -> Control + 'a>;

/// Opções comuns. `None` usa o padrão de cada função (documentado nela).
#[derive(Default)]
pub struct Options<'a> {
    /// Tolerância em x.
    pub tol_x: Option<f64>,
    /// Tolerância no valor da função (Nelder–Mead e NNLS).
    pub tol_fun: Option<f64>,
    /// Máximo de iterações.
    pub max_iter: Option<usize>,
    /// Máximo de avaliações da função.
    pub max_fun_evals: Option<usize>,
    /// Guardar o histórico das iterações no resultado.
    pub history: bool,
    /// Função de saída chamada a cada iteração.
    pub output: Option<OutputFn<'a>>,
    /// [`fzero`]: método.
    pub zero_method: ZeroMethod,
    /// [`fminsearch`]: parâmetros adaptativos de Gao & Han (recomendados para n ≥ 5).
    pub adaptive: bool,
    /// [`fminsearch`]: número máximo de reinícios (padrão 1).
    pub restarts: Option<usize>,
    /// [`fminsearch`]: passo do simplex inicial em cada coordenada.
    pub initial_step: Option<Vec<f64>>,
    /// [`fminsearch`]: simplex inicial completo (n + 1 pontos); tem precedência sobre o passo.
    pub initial_simplex: Option<Vec<Vec<f64>>>,
}

impl fmt::Debug for Options<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("tol_x", &self.tol_x)
            .field("tol_fun", &self.tol_fun)
            .field("max_iter", &self.max_iter)
            .field("max_fun_evals", &self.max_fun_evals)
            .field("history", &self.history)
            .field("output", &self.output.as_ref().map(|_| "<função>"))
            .field("zero_method", &self.zero_method)
            .field("adaptive", &self.adaptive)
            .field("restarts", &self.restarts)
            .field("initial_step", &self.initial_step)
            .field("initial_simplex", &self.initial_simplex)
            .finish()
    }
}

/// Motivo do término. [`ExitFlag::code`] dá o código numérico padronizado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitFlag {
    /// Convergiu dentro das tolerâncias (1).
    Converged,
    /// Atingiu o máximo de iterações (0).
    MaxIterations,
    /// Atingiu o máximo de avaliações (0).
    MaxFunEvals,
    /// Interrompido pela função de saída (−1).
    Stopped,
    /// A função devolveu NaN ou ±∞ onde o método precisa de um valor finito (−3).
    NonFinite,
    /// O intervalo encolheu sobre uma mudança de sinal que não é raiz: polo ou salto (−5).
    SignChangeNotRoot,
}

impl ExitFlag {
    /// Código numérico: positivo = sucesso, 0 = limite atingido, negativo = falha.
    pub fn code(self) -> i32 {
        match self {
            Self::Converged => 1,
            Self::MaxIterations | Self::MaxFunEvals => 0,
            Self::Stopped => -1,
            Self::NonFinite => -3,
            Self::SignChangeNotRoot => -5,
        }
    }

    /// Mensagem para o usuário.
    pub fn message(self) -> &'static str {
        match self {
            Self::Converged => "convergiu dentro das tolerâncias",
            Self::MaxIterations => "atingiu o máximo de iterações",
            Self::MaxFunEvals => "atingiu o máximo de avaliações da função",
            Self::Stopped => "interrompido pela função de saída",
            Self::NonFinite => "a função devolveu um valor não finito",
            Self::SignChangeNotRoot => {
                "o intervalo convergiu para uma mudança de sinal que não é raiz (polo ou salto)"
            }
        }
    }
}

/// Resultado de [`fzero`], [`fminbnd`] e [`fminsearch`].
#[derive(Clone, Debug, PartialEq)]
pub struct OptimResult<X> {
    /// Solução.
    pub x: X,
    /// Valor da função na solução.
    pub fval: f64,
    /// Motivo do término.
    pub exit: ExitFlag,
    /// Iterações.
    pub iterations: usize,
    /// Avaliações da função.
    pub fun_evals: usize,
    /// Intervalo final (zero e mínimo escalares).
    pub interval: Option<(f64, f64)>,
    /// Histórico, se pedido em [`Options::history`].
    pub history: Vec<Iteration>,
}

/// Erros de otimização.
#[derive(Clone, Debug, PartialEq)]
pub enum OptimError {
    /// Intervalo com extremo NaN ou infinito, ou com a > b (ou a = b onde não é permitido).
    InvalidInterval(&'static str),
    /// Os extremos do intervalo dado não têm sinais opostos, ou a busca não achou mudança de sinal.
    NoSignChange {
        /// Último intervalo examinado.
        interval: (f64, f64),
        /// Valores da função nos extremos.
        values: (f64, f64),
    },
    /// Ponto inicial vazio, não finito ou com valor não finito.
    InvalidStart(&'static str),
    /// Tolerância negativa ou NaN.
    InvalidTolerance(&'static str),
    /// Opção com valor inválido.
    InvalidOption(&'static str),
    /// Dimensões incompatíveis.
    ShapeMismatch {
        /// Elementos esperados.
        expected: usize,
        /// Elementos recebidos.
        actual: usize,
    },
}

impl OptimError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::InvalidInterval(_) => "chimalab:otimizacao:intervaloInvalido",
            Self::NoSignChange { .. } => "chimalab:otimizacao:semMudancaDeSinal",
            Self::InvalidStart(_) => "chimalab:otimizacao:pontoInicialInvalido",
            Self::InvalidTolerance(_) => "chimalab:otimizacao:toleranciaInvalida",
            Self::InvalidOption(_) => "chimalab:otimizacao:opcaoInvalida",
            Self::ShapeMismatch { .. } => "chimalab:otimizacao:formaIncompativel",
        }
    }
}

impl fmt::Display for OptimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInterval(m) => write!(f, "intervalo inválido: {m}"),
            Self::NoSignChange { interval, values } => write!(
                f,
                "a função não muda de sinal em [{}, {}] (valores {} e {})",
                interval.0, interval.1, values.0, values.1
            ),
            Self::InvalidStart(m) => write!(f, "ponto inicial inválido: {m}"),
            Self::InvalidTolerance(m) => write!(f, "tolerância inválida: {m}"),
            Self::InvalidOption(m) => write!(f, "opção inválida: {m}"),
            Self::ShapeMismatch { expected, actual } => write!(
                f,
                "eram esperados {expected} elementos, mas foram recebidos {actual}"
            ),
        }
    }
}

impl std::error::Error for OptimError {}

/// Contagem, limites, histórico e função de saída, comuns a todos os métodos.
pub(crate) struct Tracker<'o, 'a> {
    pub iterations: usize,
    pub fun_evals: usize,
    max_iter: usize,
    max_fun_evals: usize,
    keep: bool,
    pub history: Vec<Iteration>,
    output: Option<&'o mut OutputFn<'a>>,
}

impl<'o, 'a> Tracker<'o, 'a> {
    pub fn new(
        opts: &'o mut Options<'a>,
        default_iter: usize,
        default_evals: usize,
    ) -> Result<Self, OptimError> {
        for (t, nome) in [(opts.tol_x, "tol_x"), (opts.tol_fun, "tol_fun")] {
            if let Some(t) = t
                && !(t >= 0.0 && t.is_finite())
            {
                return Err(OptimError::InvalidTolerance(nome));
            }
        }
        Ok(Self {
            iterations: 0,
            fun_evals: 0,
            max_iter: opts.max_iter.unwrap_or(default_iter),
            max_fun_evals: opts.max_fun_evals.unwrap_or(default_evals),
            keep: opts.history,
            history: Vec::new(),
            output: opts.output.as_mut(),
        })
    }

    /// Registra uma iteração; devolve `Some(flag)` se for para parar.
    pub fn report(
        &mut self,
        x: &[f64],
        fval: f64,
        procedure: Procedure,
        interval: Option<(f64, f64)>,
    ) -> Option<ExitFlag> {
        if self.keep || self.output.is_some() {
            let it = Iteration {
                iteration: self.iterations,
                fun_evals: self.fun_evals,
                x: x.to_vec(),
                fval,
                procedure,
                interval,
            };
            let stop = self.output.as_mut().is_some_and(|o| o(&it) == Control::Stop);
            if self.keep {
                self.history.push(it);
            }
            if stop {
                return Some(ExitFlag::Stopped);
            }
        }
        self.limit()
    }

    /// Limite de iterações ou de avaliações atingido.
    pub fn limit(&self) -> Option<ExitFlag> {
        if self.iterations >= self.max_iter {
            Some(ExitFlag::MaxIterations)
        } else if self.fun_evals >= self.max_fun_evals {
            Some(ExitFlag::MaxFunEvals)
        } else {
            None
        }
    }

    pub fn finish<X>(self, x: X, fval: f64, exit: ExitFlag, interval: Option<(f64, f64)>) -> OptimResult<X> {
        OptimResult {
            x,
            fval,
            exit,
            iterations: self.iterations,
            fun_evals: self.fun_evals,
            interval,
            history: self.history,
        }
    }
}
