//! Números aleatórios (funcionalidade F053, RF-053; desenho em DT-23).
//!
//! [`Rng`] combina um gerador de bits ([`Algorithm`]) com as distribuições:
//! - uniforme em (0, 1) aberto, com 53 bits;
//! - inteiros sem viés em [a, b] (Lemire, ACM TOMACS 29, 2019);
//! - normal pelo ziggurat de 256 camadas (Marsaglia & Tsang, J. Stat. Softw. 5, 2000), com camada
//!   e uniforme tirados de bits independentes (Doornik, 2005);
//! - permutações (Fisher–Yates).
//!
//! O estado é salvável e restaurável ([`Rng::state`], [`Rng::from_state`]). Fluxos independentes:
//! [`Rng::with_stream`] (Philox, Threefry e MRG32k3a, este também com subfluxos).

mod engines;
mod ziggurat;

use std::fmt;

use engines::{MRG_M1, Mrg32k3a, Mt19937, Philox, Threefry, Xoshiro256pp};

/// Algoritmo gerador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    /// Philox4x32-10 (padrão): baseado em contador; fluxos pela parte alta do contador.
    Philox,
    /// Threefry4x64-20: baseado em contador, 64 bits.
    Threefry,
    /// Mersenne Twister MT19937.
    Twister,
    /// MRG32k3a, com fluxos (saltos de 2¹²⁷) e subfluxos (2⁷⁶).
    Mrg32k3a,
    /// xoshiro256++.
    Xoshiro256pp,
}

impl Algorithm {
    /// Nome estável (usado no estado salvo e na linguagem).
    pub fn name(self) -> &'static str {
        match self {
            Self::Philox => "philox",
            Self::Threefry => "threefry",
            Self::Twister => "twister",
            Self::Mrg32k3a => "mrg32k3a",
            Self::Xoshiro256pp => "xoshiro256pp",
        }
    }

    /// Algoritmo pelo nome; erro `geradorDesconhecido` se não existir.
    pub fn from_name(name: &str) -> Result<Self, RandomError> {
        [
            Self::Philox,
            Self::Threefry,
            Self::Twister,
            Self::Mrg32k3a,
            Self::Xoshiro256pp,
        ]
        .into_iter()
        .find(|a| a.name() == name)
        .ok_or(RandomError::UnknownGenerator)
    }
}

/// Erros de números aleatórios.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomError {
    /// Nome de gerador desconhecido.
    UnknownGenerator,
    /// Estado salvo inválido (tamanho errado, valores fora da faixa).
    InvalidState,
    /// Argumento inválido (intervalo vazio, k > n, fluxos não suportados).
    InvalidArgument(&'static str),
}

impl RandomError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::UnknownGenerator => "chimalab:random:geradorDesconhecido",
            Self::InvalidState => "chimalab:random:estadoInvalido",
            Self::InvalidArgument(_) => "chimalab:random:argumentoInvalido",
        }
    }
}

impl fmt::Display for RandomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownGenerator => write!(f, "gerador desconhecido"),
            Self::InvalidState => write!(f, "estado de gerador inválido"),
            Self::InvalidArgument(m) => write!(f, "argumento inválido: {m}"),
        }
    }
}

impl std::error::Error for RandomError {}

/// Estado completo de um [`Rng`], para salvar e restaurar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RngState {
    /// Algoritmo.
    pub algorithm: Algorithm,
    /// Palavras do estado (formato próprio de cada algoritmo).
    pub words: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
enum Engine {
    Philox(Philox),
    Threefry(Threefry),
    Twister(Mt19937),
    Mrg(Mrg32k3a),
    Xoshiro(Xoshiro256pp),
}

/// Gerador de números aleatórios com distribuições. Dois geradores são iguais quando têm o mesmo
/// estado lógico ([`Rng::state`]), isto é, quando produzem as mesmas sequências; buffers internos
/// já consumidos não contam.
#[derive(Clone, Debug)]
pub struct Rng {
    engine: Engine,
    /// Metade alta guardada de um `next_u64` para servir o próximo `next_u32` (geradores de 64
    /// bits) — não usada pelos de 32 bits.
    spare: Option<u32>,
}

impl Rng {
    /// Gerador `algorithm` com a semente `seed`.
    pub fn new(algorithm: Algorithm, seed: u64) -> Self {
        let engine = match algorithm {
            Algorithm::Philox => Engine::Philox(Philox::new(seed, 0)),
            Algorithm::Threefry => Engine::Threefry(Threefry::new(seed, 0)),
            Algorithm::Twister => Engine::Twister(Mt19937::new(seed)),
            Algorithm::Mrg32k3a => Engine::Mrg(Mrg32k3a::new(seed, 0, 0)),
            Algorithm::Xoshiro256pp => Engine::Xoshiro(Xoshiro256pp::new(seed)),
        };
        Self { engine, spare: None }
    }

    /// Fluxo `stream` (e subfluxo `substream`, só no MRG32k3a) da semente `seed`. Fluxos distintos
    /// não se sobrepõem: Philox e Threefry usam a parte alta do contador (2⁶⁴ blocos por fluxo);
    /// o MRG32k3a salta 2¹²⁷ passos por fluxo e 2⁷⁶ por subfluxo; o xoshiro256++ salta 2¹²⁸
    /// passos por fluxo (matriz de transição em GF(2), calculada uma vez). O MT19937 não tem
    /// fluxos (erro `argumentoInvalido`).
    pub fn with_stream(
        algorithm: Algorithm,
        seed: u64,
        stream: u64,
        substream: u64,
    ) -> Result<Self, RandomError> {
        let engine = match algorithm {
            Algorithm::Philox if substream == 0 => Engine::Philox(Philox::new(seed, stream)),
            Algorithm::Threefry if substream == 0 => Engine::Threefry(Threefry::new(seed, stream)),
            Algorithm::Mrg32k3a => Engine::Mrg(Mrg32k3a::new(seed, stream, substream)),
            Algorithm::Xoshiro256pp if substream == 0 => {
                let mut g = Xoshiro256pp::new(seed);
                g.jump_streams(stream);
                Engine::Xoshiro(g)
            }
            Algorithm::Philox | Algorithm::Threefry | Algorithm::Xoshiro256pp => {
                return Err(RandomError::InvalidArgument("subfluxos só existem no mrg32k3a"));
            }
            Algorithm::Twister => {
                return Err(RandomError::InvalidArgument("este gerador não tem fluxos"));
            }
        };
        Ok(Self { engine, spare: None })
    }

    /// Algoritmo em uso.
    pub fn algorithm(&self) -> Algorithm {
        match self.engine {
            Engine::Philox(_) => Algorithm::Philox,
            Engine::Threefry(_) => Algorithm::Threefry,
            Engine::Twister(_) => Algorithm::Twister,
            Engine::Mrg(_) => Algorithm::Mrg32k3a,
            Engine::Xoshiro(_) => Algorithm::Xoshiro256pp,
        }
    }

    /// Próximos 32 bits.
    pub fn next_u32(&mut self) -> u32 {
        match &mut self.engine {
            Engine::Philox(g) => g.next_u32(),
            Engine::Twister(g) => g.next_u32(),
            // MRG32k3a: z ∈ [0, m₁), m₁ = 2³² − 209: os 209 valores mais altos de u32 nunca saem
            // (viés relativo < 5e-8, documentado).
            Engine::Mrg(g) => g.next_z() as u32,
            Engine::Threefry(_) | Engine::Xoshiro(_) => match self.spare.take() {
                Some(v) => v,
                None => {
                    let x = self.next_u64_raw();
                    self.spare = Some((x >> 32) as u32);
                    x as u32
                }
            },
        }
    }

    fn next_u64_raw(&mut self) -> u64 {
        match &mut self.engine {
            Engine::Threefry(g) => g.next_u64(),
            Engine::Xoshiro(g) => g.next_u64(),
            _ => unreachable!("só geradores de 64 bits"),
        }
    }

    /// Próximos 64 bits (geradores de 32 bits: duas palavras, a primeira na parte alta).
    pub fn next_u64(&mut self) -> u64 {
        match self.engine {
            Engine::Threefry(_) | Engine::Xoshiro(_) if self.spare.is_none() => self.next_u64_raw(),
            _ => (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32()),
        }
    }

    /// Uniforme em (0, 1) aberto: ((x >> 11) + ½)·2⁻⁵³, nunca 0 nem 1. O MRG32k3a usa a sua
    /// saída nativa (z + ½)/m₁, com resolução de 32 bits.
    pub fn uniform(&mut self) -> f64 {
        if let Engine::Mrg(g) = &mut self.engine {
            return (g.next_z() as f64 + 0.5) / MRG_M1 as f64;
        }
        ((self.next_u64() >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64)
    }

    /// `n` uniformes em (0, 1).
    pub fn uniforms(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.uniform()).collect()
    }

    /// Inteiro uniforme em [lo, hi] sem viés (Lemire, 2019). Erro se lo > hi.
    pub fn integer(&mut self, lo: i64, hi: i64) -> Result<i64, RandomError> {
        if lo > hi {
            return Err(RandomError::InvalidArgument("intervalo vazio (lo > hi)"));
        }
        let range = (hi as i128 - lo as i128 + 1) as u128;
        if range == 1 << 64 {
            return Ok(self.next_u64() as i64);
        }
        Ok((lo as i128 + self.bounded(range as u64) as i128) as i64)
    }

    /// Uniforme em [0, s), s ≥ 1: multiplica x·s em 128 bits e rejeita só a faixa de viés
    /// (l < 2⁶⁴ mod s), calculando o resto só quando é preciso.
    fn bounded(&mut self, s: u64) -> u64 {
        let mut m = u128::from(self.next_u64()) * u128::from(s);
        if (m as u64) < s {
            let t = s.wrapping_neg() % s;
            while (m as u64) < t {
                m = u128::from(self.next_u64()) * u128::from(s);
            }
        }
        (m >> 64) as u64
    }

    /// Normal padrão N(0, 1) pelo ziggurat.
    pub fn normal(&mut self) -> f64 {
        ziggurat::normal(self)
    }

    /// `n` normais padrão.
    pub fn normals(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.normal()).collect()
    }

    /// Permutação aleatória de 0..n (Fisher–Yates).
    pub fn permutation(&mut self, n: usize) -> Vec<usize> {
        let mut p: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = self.bounded(i as u64 + 1) as usize;
            p.swap(i, j);
        }
        p
    }

    /// `k` elementos distintos de 0..n em ordem aleatória (Fisher–Yates parcial). Erro se k > n.
    pub fn sample(&mut self, n: usize, k: usize) -> Result<Vec<usize>, RandomError> {
        if k > n {
            return Err(RandomError::InvalidArgument("k > n"));
        }
        let mut p: Vec<usize> = (0..n).collect();
        for i in 0..k {
            let j = i + self.bounded((n - i) as u64) as usize;
            p.swap(i, j);
        }
        p.truncate(k);
        Ok(p)
    }

    /// Estado completo (algoritmo e palavras), para salvar.
    pub fn state(&self) -> RngState {
        // u32 guardado (geradores de 64 bits): sinalizador e valor em palavras separadas.
        let spare = [u64::from(self.spare.is_some()), self.spare.map_or(0, u64::from)];
        let words = match &self.engine {
            Engine::Philox(g) => {
                vec![
                    u64::from(g.key[0]),
                    u64::from(g.key[1]),
                    g.block,
                    g.stream,
                    g.pos as u64,
                ]
            }
            Engine::Threefry(g) => [vec![g.key[0], g.block, g.stream, g.pos as u64], spare.to_vec()].concat(),
            Engine::Twister(g) => {
                let mut w: Vec<u64> = g.mt.iter().map(|&x| u64::from(x)).collect();
                w.push(g.index as u64);
                w
            }
            Engine::Mrg(g) => g.s1.iter().chain(&g.s2).copied().collect(),
            Engine::Xoshiro(g) => [g.s.to_vec(), spare.to_vec()].concat(),
        };
        RngState {
            algorithm: self.algorithm(),
            words,
        }
    }

    /// Restaura um estado salvo por [`Rng::state`]. Erro `estadoInvalido` se as palavras não
    /// formam um estado válido do algoritmo.
    pub fn from_state(state: &RngState) -> Result<Self, RandomError> {
        let w = &state.words;
        let bad = Err(RandomError::InvalidState);
        let u32_max = u64::from(u32::MAX);
        let spare_of = |flag: u64, val: u64| match (flag, val) {
            (0, 0) => Ok(None),
            (1, v) if v <= u32_max => Ok(Some(v as u32)),
            _ => Err(RandomError::InvalidState),
        };
        // Philox e Threefry guardam o bloco seguinte e a posição pos ∈ 1..=4 no bloco em uso
        // (pos = 4: esgotado). Restaura recuando um bloco e consumindo pos palavras: um só caminho,
        // que também vale para pos = 4.
        let (engine, spare) = match state.algorithm {
            Algorithm::Philox => {
                let [k0, k1, block, stream, pos] = w[..] else {
                    return bad;
                };
                if !(1..=4).contains(&pos) || k0 > u32_max || k1 > u32_max {
                    return bad;
                }
                let mut g = Philox::new(0, stream);
                g.key = [k0 as u32, k1 as u32];
                g.block = block.wrapping_sub(1);
                for _ in 0..pos {
                    g.next_u32();
                }
                (Engine::Philox(g), None)
            }
            Algorithm::Threefry => {
                let [key, block, stream, pos, flag, val] = w[..] else {
                    return bad;
                };
                if !(1..=4).contains(&pos) {
                    return bad;
                }
                let mut g = Threefry::new(key, stream);
                g.block = block.wrapping_sub(1);
                for _ in 0..pos {
                    g.next_u64();
                }
                (Engine::Threefry(g), spare_of(flag, val)?)
            }
            Algorithm::Twister => {
                if w.len() != 625 || w[624] > 624 || w[..624].iter().any(|&x| x > u32_max) {
                    return bad;
                }
                let mt = w[..624].iter().map(|&x| x as u32).collect();
                (
                    Engine::Twister(Mt19937 {
                        mt,
                        index: w[624] as usize,
                    }),
                    None,
                )
            }
            Algorithm::Mrg32k3a => {
                let [a, b, c, d, e, f] = w[..] else { return bad };
                let ok1 = [a, b, c].iter().all(|&x| x < MRG_M1) && [a, b, c] != [0, 0, 0];
                let ok2 = [d, e, f].iter().all(|&x| x < engines::MRG_M2) && [d, e, f] != [0, 0, 0];
                if !(ok1 && ok2) {
                    return bad;
                }
                (
                    Engine::Mrg(Mrg32k3a {
                        s1: [a, b, c],
                        s2: [d, e, f],
                    }),
                    None,
                )
            }
            Algorithm::Xoshiro256pp => {
                let [a, b, c, d, flag, val] = w[..] else {
                    return bad;
                };
                if [a, b, c, d] == [0, 0, 0, 0] {
                    return bad;
                }
                (
                    Engine::Xoshiro(Xoshiro256pp { s: [a, b, c, d] }),
                    spare_of(flag, val)?,
                )
            }
        };
        Ok(Self { engine, spare })
    }
}

impl PartialEq for Rng {
    fn eq(&self, other: &Self) -> bool {
        self.state() == other.state()
    }
}

impl Default for Rng {
    /// Philox com semente 0.
    fn default() -> Self {
        Self::new(Algorithm::Philox, 0)
    }
}

/// Acesso interno aos núcleos (testes de KAT e de saltos).
#[doc(hidden)]
pub mod raw {
    pub use super::engines::{philox4x32_10, threefry4x64_20};

    /// Primeiras `n` saídas de 32 bits do MT19937 com `init_genrand(seed)`.
    pub fn mt19937_u32(seed: u32, n: usize) -> Vec<u32> {
        let mut g = super::engines::Mt19937::new_u32(seed);
        (0..n).map(|_| g.next_u32()).collect()
    }

    /// xoshiro256++: estado após `k` passos um a um e pela matriz Tᵏ em GF(2) (devem coincidir).
    pub fn xoshiro_step_vs_matrix(seed: u64, k: u64) -> ([u64; 4], [u64; 4]) {
        let mut a = super::engines::Xoshiro256pp::new(seed);
        for _ in 0..k {
            a.next_u64();
        }
        let b = super::engines::Xoshiro256pp::new(seed);
        let m = super::engines::gf2_pow(&super::engines::xo_matrix(), k);
        (a.s, super::engines::gf2_apply(&m, b.s))
    }

    /// Estado do fluxo `n` (n > 64 usa a potência da matriz de salto) e o mesmo estado por n
    /// aplicações sucessivas do salto de 2¹²⁸ (devem coincidir).
    pub fn xoshiro_jump_paths(seed: u64, n: u64) -> ([u64; 4], [u64; 4]) {
        let mut a = super::engines::Xoshiro256pp::new(seed);
        a.jump_streams(n);
        let mut b = super::engines::Xoshiro256pp::new(seed);
        for _ in 0..n {
            b.jump_streams(1);
        }
        (a.s, b.s)
    }

    /// Verdadeiro se T^(2²⁵⁶) = T, consequência do período máximo 2²⁵⁶ − 1 do xoshiro256++.
    pub fn xoshiro_period_identity() -> bool {
        let t = super::engines::xo_matrix();
        super::engines::gf2_pow2k(&t, 256) == t
    }

    /// Estados do MRG32k3a após `k` passos um a um e por salto de matriz (devem coincidir).
    pub fn mrg32k3a_step_vs_jump(seed: u64, k: u64) -> ([u64; 6], [u64; 6]) {
        let mut a = super::engines::Mrg32k3a::new(seed, 0, 0);
        let mut b = a.clone();
        for _ in 0..k {
            a.next_z();
        }
        b.advance(k);
        let st = |g: &super::engines::Mrg32k3a| [g.s1[0], g.s1[1], g.s1[2], g.s2[0], g.s2[1], g.s2[2]];
        (st(&a), st(&b))
    }
}
