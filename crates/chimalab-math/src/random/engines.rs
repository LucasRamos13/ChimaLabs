//! Geradores de bits pseudoaleatórios, implementados a partir das publicações (DT-23).

/// Saída do SplitMix64 (Steele, Lea & Flood, OOPSLA 2014): usado para expandir sementes.
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

// ---------------------------------------------------------------------------------------------
// MT19937 (Matsumoto & Nishimura, ACM TOMACS 8, 1998)

const MT_N: usize = 624;
const MT_M: usize = 397;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mt19937 {
    pub(crate) mt: Vec<u32>,
    pub(crate) index: usize,
}

impl Mt19937 {
    /// `init_genrand` do artigo.
    pub(crate) fn new_u32(seed: u32) -> Self {
        let mut mt = vec![0u32; MT_N];
        mt[0] = seed;
        for i in 1..MT_N {
            mt[i] = 1_812_433_253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, index: MT_N }
    }

    /// `init_by_array` (versão de 2002 do código de referência dos autores).
    pub(crate) fn new_array(key: &[u32]) -> Self {
        let mut g = Self::new_u32(19_650_218);
        let mt = &mut g.mt;
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..MT_N.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= MT_N {
                mt[0] = mt[MT_N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..MT_N - 1 {
            mt[i] =
                (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_566_083_941)).wrapping_sub(i as u32);
            i += 1;
            if i >= MT_N {
                mt[0] = mt[MT_N - 1];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        g
    }

    /// Semente de 64 bits: `init_genrand(s)` se cabe em 32 bits (sequências iguais às do padrão
    /// C++ `std::mt19937`); senão `init_by_array([lo, hi])`.
    pub(crate) fn new(seed: u64) -> Self {
        match u32::try_from(seed) {
            Ok(s) => Self::new_u32(s),
            Err(_) => Self::new_array(&[seed as u32, (seed >> 32) as u32]),
        }
    }

    fn twist(&mut self) {
        const UPPER: u32 = 0x8000_0000;
        const LOWER: u32 = 0x7fff_ffff;
        for k in 0..MT_N {
            let y = (self.mt[k] & UPPER) | (self.mt[(k + 1) % MT_N] & LOWER);
            let mag = if y & 1 == 1 { 0x9908_b0df } else { 0 };
            self.mt[k] = self.mt[(k + MT_M) % MT_N] ^ (y >> 1) ^ mag;
        }
        self.index = 0;
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        if self.index >= MT_N {
            self.twist();
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }
}

// ---------------------------------------------------------------------------------------------
// Philox4x32-10 (Salmon, Moraes, Dror & Shaw, SC'11, 2011)

const PHILOX_M0: u32 = 0xD251_1F53;
const PHILOX_M1: u32 = 0xCD9E_8D57;
const PHILOX_W0: u32 = 0x9E37_79B9;
const PHILOX_W1: u32 = 0xBB67_AE85;

/// Bloco de Philox4x32 com 10 rodadas: f(contador, chave).
pub fn philox4x32_10(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let (mut c, mut k) = (ctr, key);
    for r in 0..10 {
        if r > 0 {
            k[0] = k[0].wrapping_add(PHILOX_W0);
            k[1] = k[1].wrapping_add(PHILOX_W1);
        }
        let p0 = u64::from(PHILOX_M0) * u64::from(c[0]);
        let p1 = u64::from(PHILOX_M1) * u64::from(c[2]);
        c = [
            ((p1 >> 32) as u32) ^ c[1] ^ k[0],
            p1 as u32,
            ((p0 >> 32) as u32) ^ c[3] ^ k[1],
            p0 as u32,
        ];
    }
    c
}

/// Gerador por contador: o contador de 128 bits é (bloco: 64 bits, fluxo: 64 bits); cada bloco
/// dá 4 palavras de 32 bits.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Philox {
    pub(crate) key: [u32; 2],
    pub(crate) block: u64,
    pub(crate) stream: u64,
    pub(crate) buf: [u32; 4],
    pub(crate) pos: usize,
}

impl Philox {
    pub(crate) fn new(seed: u64, stream: u64) -> Self {
        Self {
            key: [seed as u32, (seed >> 32) as u32],
            block: 0,
            stream,
            buf: [0; 4],
            pos: 4,
        }
    }

    pub(crate) fn counter(&self) -> [u32; 4] {
        [
            self.block as u32,
            (self.block >> 32) as u32,
            self.stream as u32,
            (self.stream >> 32) as u32,
        ]
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        if self.pos == 4 {
            self.buf = philox4x32_10(self.counter(), self.key);
            self.block = self.block.wrapping_add(1);
            self.pos = 0;
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }
}

// ---------------------------------------------------------------------------------------------
// Threefry4x64-20 (Salmon et al., 2011; cifra Threefish de Ferguson et al.)

const THREEFRY_PARITY: u64 = 0x1BD1_1BDA_A9FC_1A22;
const THREEFRY_R: [[u32; 2]; 8] = [
    [14, 16],
    [52, 57],
    [23, 40],
    [5, 37],
    [25, 33],
    [46, 12],
    [58, 22],
    [32, 32],
];

/// Bloco de Threefry4x64 com 20 rodadas, injeção de chave a cada 4 rodadas.
pub fn threefry4x64_20(ctr: [u64; 4], key: [u64; 4]) -> [u64; 4] {
    let ks = [
        key[0],
        key[1],
        key[2],
        key[3],
        THREEFRY_PARITY ^ key[0] ^ key[1] ^ key[2] ^ key[3],
    ];
    let mut x = [
        ctr[0].wrapping_add(ks[0]),
        ctr[1].wrapping_add(ks[1]),
        ctr[2].wrapping_add(ks[2]),
        ctr[3].wrapping_add(ks[3]),
    ];
    for r in 0..20 {
        let rot = THREEFRY_R[r % 8];
        if r % 2 == 0 {
            x[0] = x[0].wrapping_add(x[1]);
            x[1] = x[1].rotate_left(rot[0]) ^ x[0];
            x[2] = x[2].wrapping_add(x[3]);
            x[3] = x[3].rotate_left(rot[1]) ^ x[2];
        } else {
            x[0] = x[0].wrapping_add(x[3]);
            x[3] = x[3].rotate_left(rot[0]) ^ x[0];
            x[2] = x[2].wrapping_add(x[1]);
            x[1] = x[1].rotate_left(rot[1]) ^ x[2];
        }
        if r % 4 == 3 {
            let s = r / 4 + 1;
            for (i, xi) in x.iter_mut().enumerate() {
                *xi = xi.wrapping_add(ks[(s + i) % 5]);
            }
            x[3] = x[3].wrapping_add(s as u64);
        }
    }
    x
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Threefry {
    pub(crate) key: [u64; 4],
    pub(crate) block: u64,
    pub(crate) stream: u64,
    pub(crate) buf: [u64; 4],
    pub(crate) pos: usize,
}

impl Threefry {
    pub(crate) fn new(seed: u64, stream: u64) -> Self {
        Self {
            key: [seed, 0, 0, 0],
            block: 0,
            stream,
            buf: [0; 4],
            pos: 4,
        }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        if self.pos == 4 {
            self.buf = threefry4x64_20([self.block, 0, self.stream, 0], self.key);
            self.block = self.block.wrapping_add(1);
            self.pos = 0;
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }
}

// ---------------------------------------------------------------------------------------------
// MRG32k3a (L'Ecuyer, Oper. Res. 47, 1999), com fluxos e subfluxos (L'Ecuyer et al., 2002)

pub(crate) const MRG_M1: u64 = 4_294_967_087;
pub(crate) const MRG_M2: u64 = 4_294_944_443;
/// Matrizes de transição de um passo das duas componentes (estado [x(n−3), x(n−2), x(n−1)]).
const MRG_A1: [[u64; 3]; 3] = [[0, 1, 0], [0, 0, 1], [MRG_M1 - 810_728, 1_403_580, 0]];
const MRG_A2: [[u64; 3]; 3] = [[0, 1, 0], [0, 0, 1], [MRG_M2 - 1_370_589, 0, 527_612]];

type Mat3 = [[u64; 3]; 3];

fn matmul_mod(a: &Mat3, b: &Mat3, m: u64) -> Mat3 {
    let mut c = [[0u64; 3]; 3];
    for (i, row) in c.iter_mut().enumerate() {
        for (j, cij) in row.iter_mut().enumerate() {
            let mut s: u128 = 0;
            for k in 0..3 {
                s += u128::from(a[i][k]) * u128::from(b[k][j]);
            }
            *cij = (s % u128::from(m)) as u64;
        }
    }
    c
}

fn matvec_mod(a: &Mat3, v: &[u64; 3], m: u64) -> [u64; 3] {
    let mut out = [0u64; 3];
    for (i, o) in out.iter_mut().enumerate() {
        let s: u128 = (0..3).map(|k| u128::from(a[i][k]) * u128::from(v[k])).sum();
        *o = (s % u128::from(m)) as u64;
    }
    out
}

/// A^(2^e) mod m por quadrados sucessivos.
fn pow2_mod(a: &Mat3, e: u32, m: u64) -> Mat3 {
    let mut p = *a;
    for _ in 0..e {
        p = matmul_mod(&p, &p, m);
    }
    p
}

/// A^k mod m (k qualquer), por quadrados e produtos.
fn pow_mod(a: &Mat3, mut k: u64, m: u64) -> Mat3 {
    let mut r = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
    let mut p = *a;
    while k > 0 {
        if k & 1 == 1 {
            r = matmul_mod(&r, &p, m);
        }
        p = matmul_mod(&p, &p, m);
        k >>= 1;
    }
    r
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mrg32k3a {
    pub(crate) s1: [u64; 3],
    pub(crate) s2: [u64; 3],
}

impl Mrg32k3a {
    /// Estado inicial pela semente (SplitMix64, valores em [1, m)), avançado ao fluxo `stream`
    /// (saltos de 2¹²⁷) e ao subfluxo `substream` (saltos de 2⁷⁶).
    pub(crate) fn new(seed: u64, stream: u64, substream: u64) -> Self {
        let mut sm = seed;
        let mut draw = |m: u64| 1 + splitmix64(&mut sm) % (m - 1);
        let s1 = [draw(MRG_M1), draw(MRG_M1), draw(MRG_M1)];
        let s2 = [draw(MRG_M2), draw(MRG_M2), draw(MRG_M2)];
        let mut g = Self { s1, s2 };
        g.advance_pow2(127, stream);
        g.advance_pow2(76, substream);
        g
    }

    /// Avança k·2^e passos: aplica (A^(2^e))^k às duas componentes.
    pub(crate) fn advance_pow2(&mut self, e: u32, k: u64) {
        if k == 0 {
            return;
        }
        let j1 = pow_mod(&pow2_mod(&MRG_A1, e, MRG_M1), k, MRG_M1);
        let j2 = pow_mod(&pow2_mod(&MRG_A2, e, MRG_M2), k, MRG_M2);
        self.s1 = matvec_mod(&j1, &self.s1, MRG_M1);
        self.s2 = matvec_mod(&j2, &self.s2, MRG_M2);
    }

    /// Avança k passos (para testes dos saltos).
    pub(crate) fn advance(&mut self, k: u64) {
        let j1 = pow_mod(&MRG_A1, k, MRG_M1);
        let j2 = pow_mod(&MRG_A2, k, MRG_M2);
        self.s1 = matvec_mod(&j1, &self.s1, MRG_M1);
        self.s2 = matvec_mod(&j2, &self.s2, MRG_M2);
    }

    /// Próximo z = (x₁ − x₂) mod m₁ ∈ [0, m₁).
    pub(crate) fn next_z(&mut self) -> u64 {
        // Em 128 bits: (m₁ − a₁₃)·s₀ chega a ~1,84e19 e somado ao outro produto passa de 2⁶⁴
        // (com estado perto de m₁, em release a soma dava a volta e o valor saía errado).
        let w = |x: u64| u128::from(x);
        let p1 = ((w(1_403_580) * w(self.s1[1]) + w(MRG_M1 - 810_728) * w(self.s1[0])) % w(MRG_M1)) as u64;
        self.s1 = [self.s1[1], self.s1[2], p1];
        let p2 = ((w(527_612) * w(self.s2[2]) + w(MRG_M2 - 1_370_589) * w(self.s2[0])) % w(MRG_M2)) as u64;
        self.s2 = [self.s2[1], self.s2[2], p2];
        (p1 + MRG_M1 - p2) % MRG_M1
    }
}

// ---------------------------------------------------------------------------------------------
// xoshiro256++ (Blackman & Vigna, ACM TOMS 47, 2021)

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Xoshiro256pp {
    pub(crate) s: [u64; 4],
}

impl Xoshiro256pp {
    /// Estado pela semente expandida por SplitMix64 (nunca todo nulo).
    pub(crate) fn new(seed: u64) -> Self {
        let mut sm = seed;
        Self {
            s: [
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
            ],
        }
    }

    /// Salta n·2¹²⁸ passos (fluxo n): com o período 2²⁵⁶ − 1, os fluxos 0, 1, 2, … não se
    /// sobrepõem antes de 2¹²⁸ saídas cada.
    pub(crate) fn jump_streams(&mut self, n: u64) {
        let j = xo_jump_matrix();
        if n <= 64 {
            for _ in 0..n {
                self.s = gf2_apply(j, self.s);
            }
        } else {
            self.s = gf2_apply(&gf2_pow(j, n), self.s);
        }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let s = self.s;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        // A mesma transição que define a matriz de salto (um só código para os dois usos).
        self.s = xo_step(s);
        result
    }
}

// ---------------------------------------------------------------------------------------------
// Saltos do xoshiro256++ por álgebra linear em GF(2)
//
// A transição de estado do xoshiro256++ é linear sobre GF(2) (xor, deslocamentos e rotações).
// A matriz T (256 × 256) é montada coluna a coluna, aplicando um passo a cada vetor da base, e
// T^(2^k) sai por k quadrados. Nada de constantes publicadas: tudo é derivado do próprio passo.
// Teste algébrico: o período máximo 2²⁵⁶ − 1 implica T^(2²⁵⁶) = T.

/// Matriz 256 × 256 em GF(2), guardada por colunas (cada coluna, um vetor de 256 bits).
pub(crate) type Gf2Mat = Vec<[u64; 4]>;

/// Um passo da transição de estado (sem a saída).
pub(crate) fn xo_step(mut s: [u64; 4]) -> [u64; 4] {
    let t = s[1] << 17;
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = s[3].rotate_left(45);
    s
}

pub(crate) fn gf2_apply(m: &Gf2Mat, v: [u64; 4]) -> [u64; 4] {
    let mut out = [0u64; 4];
    for (w, &palavra) in v.iter().enumerate() {
        let mut bits = palavra;
        while bits != 0 {
            let b = bits.trailing_zeros() as usize;
            let col = &m[64 * w + b];
            for k in 0..4 {
                out[k] ^= col[k];
            }
            bits &= bits - 1;
        }
    }
    out
}

/// A·B (aplica B, depois A).
pub(crate) fn gf2_mul(a: &Gf2Mat, b: &Gf2Mat) -> Gf2Mat {
    b.iter().map(|&c| gf2_apply(a, c)).collect()
}

pub(crate) fn gf2_identity() -> Gf2Mat {
    (0..256)
        .map(|j| {
            let mut c = [0u64; 4];
            c[j / 64] = 1u64 << (j % 64);
            c
        })
        .collect()
}

/// Matriz de transição T.
pub(crate) fn xo_matrix() -> Gf2Mat {
    gf2_identity().into_iter().map(xo_step).collect()
}

/// M elevado a 2^k (k quadrados).
pub(crate) fn gf2_pow2k(m: &Gf2Mat, k: u32) -> Gf2Mat {
    let mut r = m.clone();
    for _ in 0..k {
        r = gf2_mul(&r, &r);
    }
    r
}

/// Mⁿ por quadrados sucessivos.
pub(crate) fn gf2_pow(m: &Gf2Mat, mut n: u64) -> Gf2Mat {
    let mut base = m.clone();
    let mut acc = gf2_identity();
    while n > 0 {
        if n & 1 == 1 {
            acc = gf2_mul(&acc, &base);
        }
        base = gf2_mul(&base, &base);
        n >>= 1;
    }
    acc
}

/// T^(2¹²⁸), calculada uma vez.
fn xo_jump_matrix() -> &'static Gf2Mat {
    static J: std::sync::OnceLock<Gf2Mat> = std::sync::OnceLock::new();
    J.get_or_init(|| gf2_pow2k(&xo_matrix(), 128))
}
