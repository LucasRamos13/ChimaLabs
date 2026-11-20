//! Teoria dos números e combinatória sobre inteiros representados exatamente em `f64`
//! (|n| ≤ 2⁵³) ou em `u64`.

use super::SpecialError;

const MAX_EXACT: f64 = 9_007_199_254_740_992.0; // 2^53

fn as_int(x: f64, what: &'static str) -> Result<i128, SpecialError> {
    if !x.is_finite() || x.fract() != 0.0 || x.abs() > MAX_EXACT {
        return Err(SpecialError::Domain(what));
    }
    Ok(x as i128)
}

/// Máximo divisor comum (sempre ≥ 0). Exige inteiros com |x| ≤ 2⁵³.
pub fn gcd(a: f64, b: f64) -> Result<f64, SpecialError> {
    Ok(gcd_bezout(a, b)?.g)
}

/// Resultado de [`gcd_bezout`]: `g = u·a + v·b`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bezout {
    /// Máximo divisor comum (≥ 0).
    pub g: f64,
    /// Coeficiente de `a`.
    pub u: f64,
    /// Coeficiente de `b`.
    pub v: f64,
}

/// MDC com coeficientes de Bézout pelo algoritmo de Euclides estendido.
pub fn gcd_bezout(a: f64, b: f64) -> Result<Bezout, SpecialError> {
    let (a, b) = (as_int(a, "gcd exige inteiros")?, as_int(b, "gcd exige inteiros")?);
    let (mut r0, mut r1) = (a, b);
    let (mut s0, mut s1) = (1i128, 0i128);
    let (mut t0, mut t1) = (0i128, 1i128);
    while r1 != 0 {
        let q = r0.div_euclid(r1);
        (r0, r1) = (r1, r0 - q * r1);
        (s0, s1) = (s1, s0 - q * s1);
        (t0, t1) = (t1, t0 - q * t1);
    }
    if r0 < 0 {
        (r0, s0, t0) = (-r0, -s0, -t0);
    }
    Ok(Bezout {
        g: r0 as f64,
        u: s0 as f64,
        v: t0 as f64,
    })
}

/// Mínimo múltiplo comum (≥ 0); `lcm(0, x) = 0`.
pub fn lcm(a: f64, b: f64) -> Result<f64, SpecialError> {
    let g = gcd(a, b)?;
    if g == 0.0 {
        return Ok(0.0);
    }
    Ok((a / g * b).abs())
}

/// Primos ≤ `n`, em ordem crescente (crivo segmentado; ver [`primes_between`]). NaN ou n < 2 →
/// vazio. Não há limite artificial: a memória é a da própria saída (π(n) ≈ n/ln n valores) mais
/// O(√n) do crivo.
pub fn primes(n: f64) -> Vec<u64> {
    // `as` satura e leva NaN a 0: NaN e n < 2 viram hi ≤ 1 (vazio); n ≥ 2⁶⁴ vira u64::MAX.
    primes_between(0, n.floor() as u64)
}

/// Maior primo-base usado pelo crivo: acima de `BASE_LIMIT²` (= 2⁴⁸), os números que sobram do
/// crivo podem ter todos os fatores > 2²⁴ e precisam passar por [`isprime`].
const BASE_LIMIT: u64 = 1 << 24;
/// Ímpares por segmento do crivo (256 KiB de marcas).
const SEGMENT: u64 = 1 << 18;

/// ⌊√n⌋ exato. O √ em f64 nunca fica abaixo de ⌊√n⌋ = k: n só é arredondado quando
/// k² > 2⁵³, e então o erro de n (≤ ½ulp(k²)) muda √n em ≤ ¼ulp(k), que o √ corretamente
/// arredondado devolve a k. Pode ficar acima (n = k² − 1 arredondado para k²): corrige-se para
/// baixo.
fn isqrt(n: u64) -> u64 {
    let mut r = (n as f64).sqrt() as u64;
    while u128::from(r) * u128::from(r) > u128::from(n) {
        r -= 1;
    }
    r
}

/// Primeiro múltiplo de um primo-base `p` que precisa ser marcado: os menores têm fator primo
/// menor que p e já foram marcados.
fn first_marked_multiple(p: u64) -> u64 {
    p * p
}

/// Se os sobreviventes do crivo precisam de confirmação por [`isprime`]: só quando √hi passa dos
/// primos-base (hi > 2⁴⁸). Nesse caso todos os sobreviventes da janela são confirmados, inclusive
/// os ≤ 2⁴⁸, que o crivo já garante: uma janela que atravessa 2⁴⁸ e é grande o bastante para isso
/// pesar não cabe na memória da saída (π(2⁴⁸) ≈ 8·10¹²). Uma guarda `v ≤ 2⁴⁸` só pouparia custo
/// e seria um mutante equivalente.
fn needs_confirmation(root: u64) -> bool {
    root > BASE_LIMIT
}

/// Primos ímpares ≤ `limit` (crivo simples; `limit` ≤ 2²⁴).
fn small_odd_primes(limit: u64) -> Vec<u64> {
    let n = limit as usize;
    let mut composite = vec![false; n + 1];
    let mut out = vec![];
    for i in (3..=n).step_by(2) {
        if !composite[i] {
            out.push(i as u64);
            for j in (first_marked_multiple(i as u64) as usize..=n).step_by(2 * i) {
                composite[j] = true;
            }
        }
    }
    out
}

/// Primos `p` com `lo ≤ p ≤ hi`, em ordem crescente (Eratóstenes segmentado só nos ímpares,
/// primos-base até min(√hi, 2²⁴); acima de 2⁴⁸ os sobreviventes são confirmados por
/// [`isprime`], determinístico). Custo O((hi − lo)·log log hi + √hi) para hi ≤ 2⁴⁸.
pub fn primes_between(lo: u64, hi: u64) -> Vec<u64> {
    let mut out = vec![];
    if hi < 2 || lo > hi {
        return out;
    }
    if lo <= 2 {
        out.push(2);
    }
    let root = isqrt(hi);
    let base = small_odd_primes(root.min(BASE_LIMIT));
    let confirm = needs_confirmation(root);
    // Primeiro ímpar ≥ max(lo, 3).
    let mut seg_lo = lo.max(3) | 1;
    let mut marks = vec![false; SEGMENT as usize];
    while seg_lo <= hi {
        // Segmento: ímpares seg_lo, seg_lo + 2, …, até hi (índice i ↔ seg_lo + 2i).
        let len = ((hi - seg_lo) / 2 + 1).min(SEGMENT);
        marks[..len as usize].fill(false);
        let seg_hi = seg_lo + 2 * (len - 1);
        for &p in &base {
            let sq = first_marked_multiple(p);
            if sq > seg_hi {
                break;
            }
            // Primeiro múltiplo ímpar de p que seja ≥ max(p², seg_lo).
            // Perto de 2⁶⁴ o múltiplo pode não existir em u64: nesse caso não há nada a marcar.
            let start = if sq >= seg_lo {
                Some(sq)
            } else {
                seg_lo.div_ceil(p).checked_mul(p)
            };
            let Some(start) = start.and_then(|m| if m % 2 == 0 { m.checked_add(p) } else { Some(m) }) else {
                continue;
            };
            let mut i = (start - seg_lo) / 2;
            while i < len {
                marks[i as usize] = true;
                i += p;
            }
        }
        for (i, &m) in marks[..len as usize].iter().enumerate() {
            let v = seg_lo + 2 * i as u64;
            if !m && (!confirm || isprime(v)) {
                out.push(v);
            }
        }
        match seg_hi.checked_add(2) {
            Some(next) => seg_lo = next,
            None => break,
        }
    }
    out
}

fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

fn pow_mod(mut b: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1 % m;
    b %= m;
    while e > 0 {
        if e & 1 == 1 {
            r = mul_mod(r, b, m);
        }
        b = mul_mod(b, b, m);
        e >>= 1;
    }
    r
}

/// Teste de primalidade determinístico para `u64` (Miller-Rabin com as 12 primeiras bases primas,
/// suficiente para n < 3,3·10²⁴).
pub fn isprime(n: u64) -> bool {
    const BASES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    if n < 2 {
        return false;
    }
    for &p in &BASES {
        if n.is_multiple_of(p) {
            return n == p;
        }
    }
    let (mut d, mut s) = (n - 1, 0);
    while d.is_multiple_of(2) {
        d /= 2;
        s += 1;
    }
    'base: for &a in &BASES {
        let mut x = pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mul_mod(x, x, n);
            if x == n - 1 {
                continue 'base;
            }
        }
        return false;
    }
    true
}

fn gcd_u(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Pollard-Brent rho: devolve um fator não trivial de `n` composto ímpar.
fn rho(n: u64) -> u64 {
    let mut c = 1u64;
    loop {
        let f = |x: u64| (mul_mod(x, x, n) + c) % n;
        let (mut x, mut y, mut g, mut q) = (2u64, 2u64, 1u64, 1u64);
        let (mut r, m) = (1u64, 128u64);
        let mut ys = 0u64;
        while g == 1 {
            x = y;
            for _ in 0..r {
                y = f(y);
            }
            let mut k = 0;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..m.min(r - k) {
                    y = f(y);
                    q = mul_mod(q, x.abs_diff(y), n);
                }
                g = gcd_u(q, n);
                k += m;
            }
            r *= 2;
        }
        if g == n {
            loop {
                ys = f(ys);
                g = gcd_u(x.abs_diff(ys), n);
                if g > 1 {
                    break;
                }
            }
        }
        if g != n {
            return g;
        }
        c += 1;
    }
}

/// Fatoração em primos, em ordem não decrescente. `factor(0) = [0]`, `factor(1) = [1]`.
pub fn factor(n: u64) -> Vec<u64> {
    if n < 4 {
        return vec![n];
    }
    let mut out = vec![];
    let mut stack = vec![n];
    while let Some(mut m) = stack.pop() {
        for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
            while m.is_multiple_of(p) {
                out.push(p);
                m /= p;
            }
        }
        if m == 1 {
            continue;
        }
        if isprime(m) {
            out.push(m);
        } else {
            let d = rho(m);
            stack.push(d);
            stack.push(m / d);
        }
    }
    out.sort_unstable();
    out
}

/// Coeficiente binomial C(n, k). Exato enquanto o resultado couber em 2⁵³; acima disso,
/// devolve a aproximação em ponto flutuante e `exact = false`.
pub fn nchoosek(n: f64, k: f64) -> Result<(f64, bool), SpecialError> {
    let (n, k) = (
        as_int(n, "nchoosek exige inteiros")?,
        as_int(k, "nchoosek exige inteiros")?,
    );
    if n < 0 || k < 0 || k > n {
        return Err(SpecialError::Domain("nchoosek exige 0 ≤ k ≤ n"));
    }
    let k = k.min(n - k);
    let mut c: u128 = 1;
    let mut exact = true;
    let mut approx = 1.0f64;
    for i in 1..=k {
        if exact {
            match c.checked_mul((n - k + i) as u128) {
                Some(v) => c = v / i as u128,
                None => {
                    exact = false;
                    approx = c as f64;
                }
            }
        }
        if !exact {
            approx = approx * (n - k + i) as f64 / i as f64;
        }
    }
    if exact {
        let v = c as f64;
        Ok((v, v <= MAX_EXACT))
    } else {
        Ok((approx.round(), false))
    }
}

/// Todas as combinações de `k` elementos de `v`, em ordem lexicográfica de posições.
pub fn combinations<T: Clone>(v: &[T], k: usize) -> Vec<Vec<T>> {
    let n = v.len();
    if k > n {
        return vec![];
    }
    let mut idx: Vec<usize> = (0..k).collect();
    let mut out = vec![];
    loop {
        out.push(idx.iter().map(|&i| v[i].clone()).collect());
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + n - k {
                break;
            }
        }
        idx[i] += 1;
        for j in i + 1..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// Todas as permutações de `v`, em ordem lexicográfica de posições (a primeira é `v`).
pub fn perms<T: Clone>(v: &[T]) -> Vec<Vec<T>> {
    let n = v.len();
    let mut idx: Vec<usize> = (0..n).collect();
    let mut out = vec![idx.iter().map(|&i| v[i].clone()).collect::<Vec<T>>()];
    loop {
        // Próxima permutação lexicográfica dos índices.
        let Some(i) = (0..n.saturating_sub(1)).rev().find(|&i| idx[i] < idx[i + 1]) else {
            return out;
        };
        let j = (i + 1..n)
            .rev()
            .find(|&j| idx[j] > idx[i])
            .expect("existe sucessor");
        idx.swap(i, j);
        idx[i + 1..].reverse();
        out.push(idx.iter().map(|&k| v[k].clone()).collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raiz_inteira_exata() {
        // r² ≤ n < (r + 1)² (em u128), em quadrados perfeitos, vizinhos e no topo de u64.
        let check = |n: u64| {
            let r = u128::from(isqrt(n));
            assert!(
                r * r <= u128::from(n) && (r + 1) * (r + 1) > u128::from(n),
                "n = {n}"
            );
        };
        for k in [
            0u64,
            1,
            2,
            3,
            1 << 20,
            (1 << 26) + 1,
            94_906_265,
            3_037_000_499,
            (1 << 32) - 1,
        ] {
            let sq = k * k;
            check(sq);
            check(sq.saturating_sub(1));
            check(sq + 1);
        }
        check(u64::MAX);
        assert_eq!(isqrt(u64::MAX), (1 << 32) - 1);
        // n = k² − 1 com k² > 2⁵³: o f64 arredonda para k² e o √ dá k (corrigido para k − 1).
        let k = (1u64 << 32) - 5;
        assert_eq!(isqrt(k * k - 1), k - 1);
    }

    #[test]
    fn primos_base_pequenos() {
        assert_eq!(
            small_odd_primes(100),
            vec![
                3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97
            ]
        );
        assert_eq!(small_odd_primes(9), vec![3, 5, 7]);
        assert!(small_odd_primes(2).is_empty());
        assert_eq!(first_marked_multiple(7), 49);
        assert!(needs_confirmation(BASE_LIMIT + 1) && !needs_confirmation(BASE_LIMIT - 1));
    }
}
