//! Matrizes de banda: detecção da largura de banda e fatorações em armazenamento compacto.
//!
//! Algoritmos (implementação própria a partir da literatura):
//! - LU de banda com pivotamento parcial: Golub & Van Loan, *Matrix Computations*, 4ª ed., §4.3.
//!   Com pivotamento, o fator U ganha largura superior `ku + kl`; L é guardado como a sequência
//!   de transformações de Gauss (multiplicadores abaixo da diagonal, até `kl` por coluna), com as
//!   trocas de linha intercaladas, P₀L₀P₁L₁… — por isso L não é armazenada permutada.
//! - Cholesky de banda (A = RᴴR, R triangular superior com largura `kd`): Golub & Van Loan, §4.3.
//!
//! Custo O(n·kl·(kl + ku)) e O(n·kd²) em vez de O(n³).

use super::{LinScalar, NdArray};

/// Largura de banda `(kl, ku)` de `a`: maior distância de um elemento não nulo abaixo e acima da
/// diagonal. Matriz diagonal → (0, 0); matriz vazia → (0, 0).
pub fn bandwidth<T: LinScalar>(a: &NdArray<T>) -> (usize, usize) {
    let (m, n) = a.shape();
    let (mut kl, mut ku) = (0, 0);
    // Colunas contíguas (ordem de colunas): sem o custo de `get` por elemento.
    for (j, col) in a.as_slice().chunks_exact(m.max(1)).take(n).enumerate() {
        // Varre só fora da banda já conhecida: começa pelos extremos da coluna.
        if let Some(i) = col.iter().skip(j + kl).rposition(|&x| x != T::zero()) {
            kl += i;
        }
        if let Some(i) = col[..j.saturating_sub(ku).min(m)]
            .iter()
            .position(|&x| x != T::zero())
        {
            ku = j - i;
        }
    }
    (kl, ku)
}

/// Armazenamento compacto por colunas: a coluna `j` guarda as linhas `j − up ..= j + kl`
/// (as que caem fora de `0..n` ficam sem uso).
#[derive(Clone, Debug)]
struct BandStore<T> {
    n: usize,
    kl: usize,
    up: usize,
    data: Vec<T>,
}

impl<T: LinScalar> BandStore<T> {
    fn new(n: usize, kl: usize, up: usize) -> Self {
        Self {
            n,
            kl,
            up,
            data: vec![T::zero(); (kl + up + 1) * n],
        }
    }

    #[inline]
    fn idx(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.n && j < self.n, "({i}, {j}) fora da matriz");
        debug_assert!(i + self.up >= j && i <= j + self.kl, "({i}, {j}) fora da banda");
        (i + self.up - j) + j * (self.kl + self.up + 1)
    }

    #[inline]
    fn get(&self, i: usize, j: usize) -> T {
        self.data[self.idx(i, j)]
    }

    #[inline]
    fn set(&mut self, i: usize, j: usize, v: T) {
        let k = self.idx(i, j);
        self.data[k] = v;
    }

    /// Copia a parte `−kl ..= ku` de `a`.
    fn from_dense(a: &NdArray<T>, kl: usize, ku: usize, up: usize) -> Self {
        let n = a.rows();
        let mut s = Self::new(n, kl, up);
        for j in 0..n {
            for i in j.saturating_sub(ku)..(j + kl + 1).min(n) {
                s.set(i, j, a.get(i, j));
            }
        }
        s
    }
}

/// Fatoração LU de banda com pivotamento parcial.
#[derive(Clone, Debug)]
pub(crate) struct BandLu<T> {
    f: BandStore<T>,
    piv: Vec<usize>,
}

impl<T: LinScalar> BandLu<T> {
    /// Fatora `a` (quadrada, largura `(kl, ku)`). Devolve `None` se algum pivô for exatamente
    /// zero (matriz singular): quem chama recorre à LU densa, que trata esse caso.
    pub(crate) fn factor(a: &NdArray<T>, kl: usize, ku: usize) -> Option<Self> {
        let n = a.rows();
        let up = ku + kl;
        let mut f = BandStore::from_dense(a, kl, ku, up);
        let mut piv = Vec::with_capacity(n);
        for k in 0..n {
            let last = (k + kl).min(n - 1);
            // Pivô: maior módulo na coluna; em empate, a primeira linha (sem troca).
            let (p, best) = (k..=last)
                .map(|i| (i, f.get(i, k).abs()))
                .fold((k, -1.0), |acc, (i, v)| if v > acc.1 { (i, v) } else { acc });
            piv.push(p);
            if best == 0.0 {
                return None;
            }
            let jmax = (k + up).min(n - 1);
            if p != k {
                for j in k..=jmax {
                    let (x, y) = (f.get(k, j), f.get(p, j));
                    f.set(k, j, y);
                    f.set(p, j, x);
                }
            }
            let pivot = f.get(k, k);
            for i in k + 1..=last {
                let l = f.get(i, k) / pivot;
                f.set(i, k, l);
                if l != T::zero() {
                    for j in k + 1..=jmax {
                        let v = f.get(i, j) - l * f.get(k, j);
                        f.set(i, j, v);
                    }
                }
            }
        }
        Some(Self { f, piv })
    }

    /// Resolve `A x = b` no lugar.
    pub(crate) fn solve_in_place(&self, b: &mut [T]) {
        let f = &self.f;
        let n = f.n;
        for k in 0..n {
            b.swap(k, self.piv[k]);
            let bk = b[k];
            for (i, bi) in b.iter_mut().enumerate().take(k + f.kl + 1).skip(k + 1) {
                *bi -= f.get(i, k) * bk;
            }
        }
        for i in (0..n).rev() {
            let jmax = (i + f.up).min(n - 1);
            let s = b[i] - (i + 1..=jmax).map(|j| f.get(i, j) * b[j]).sum::<T>();
            b[i] = s / f.get(i, i);
        }
    }

    /// Resolve `Aᴴ x = b` no lugar: Uᴴ y = b e depois (P₀L₀…)⁻ᴴ, em ordem inversa.
    pub(crate) fn solve_adjoint_in_place(&self, b: &mut [T]) {
        let f = &self.f;
        let n = f.n;
        for i in 0..n {
            let s = b[i]
                - (i.saturating_sub(f.up)..i)
                    .map(|j| f.get(j, i).conj() * b[j])
                    .sum::<T>();
            b[i] = s / f.get(i, i).conj();
        }
        for k in (0..n).rev() {
            let last = (k + f.kl).min(n - 1);
            let s = (k + 1..=last).map(|i| f.get(i, k).conj() * b[i]).sum::<T>();
            b[k] -= s;
            b.swap(k, self.piv[k]);
        }
    }
}

/// Fatoração de Cholesky de banda: A = RᴴR, R triangular superior com largura `kd`.
#[derive(Clone, Debug)]
pub(crate) struct BandChol<T> {
    r: BandStore<T>,
}

impl<T: LinScalar> BandChol<T> {
    /// Fatora a matriz hermitiana `a` (usa o triângulo superior). Devolve `None` se `a` não for
    /// definida positiva (pivô ≤ 0 ou não finito).
    pub(crate) fn factor(a: &NdArray<T>, kd: usize) -> Option<Self> {
        let n = a.rows();
        let mut r = BandStore::from_dense(a, 0, kd, kd);
        for j in 0..n {
            let j0 = j.saturating_sub(kd);
            for i in j0..j {
                let s = r.get(i, j) - (j0..i).map(|k| r.get(k, i).conj() * r.get(k, j)).sum::<T>();
                r.set(i, j, s / r.get(i, i));
            }
            let d = r.get(j, j).re() - (j0..j).map(|k| r.get(k, j).abs().powi(2)).sum::<f64>();
            if !(d.is_finite() && d > 0.0) {
                return None;
            }
            r.set(j, j, T::from_f64(d.sqrt()));
        }
        Some(Self { r })
    }

    /// Resolve `A x = b` no lugar (Rᴴ y = b; R x = y). Como A é hermitiana, serve também para Aᴴ.
    pub(crate) fn solve_in_place(&self, b: &mut [T]) {
        let r = &self.r;
        let n = r.n;
        for i in 0..n {
            let s = b[i]
                - (i.saturating_sub(r.up)..i)
                    .map(|k| r.get(k, i).conj() * b[k])
                    .sum::<T>();
            b[i] = s / r.get(i, i);
        }
        for i in (0..n).rev() {
            let jmax = (i + r.up).min(n - 1);
            let s = b[i] - (i + 1..=jmax).map(|j| r.get(i, j) * b[j]).sum::<T>();
            b[i] = s / r.get(i, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;

    /// Matriz de banda pseudoaleatória (sem dominância diagonal: exige pivotamento).
    fn banda<T: LinScalar>(n: usize, kl: usize, ku: usize, f: impl Fn(u64) -> T) -> NdArray<T> {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        NdArray::from_fn(n, n, |i, j| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            if i <= j + kl && j <= i + ku {
                f(s)
            } else {
                T::zero()
            }
        })
    }

    fn real(s: u64) -> f64 {
        (s >> 11) as f64 / (1u64 << 53) as f64 - 0.5
    }

    fn cplx(s: u64) -> Complex64 {
        Complex64::new(real(s), real(s.rotate_left(29)))
    }

    /// ‖M·x − b‖∞ com M densa.
    fn residuo<T: LinScalar>(m: &NdArray<T>, x: &[T], b: &[T]) -> f64 {
        (0..m.rows())
            .map(|i| ((0..m.cols()).map(|j| m.get(i, j) * x[j]).sum::<T>() - b[i]).abs())
            .fold(0.0, f64::max)
    }

    fn confere_lu<T: LinScalar>(a: &NdArray<T>, kl: usize, ku: usize) {
        let n = a.rows();
        let f = BandLu::factor(a, kl, ku).expect("não singular");
        let b: Vec<T> = (0..n).map(|i| T::from_f64(1.0 + i as f64 / n as f64)).collect();
        let mut x = b.clone();
        f.solve_in_place(&mut x);
        assert!(residuo(a, &x, &b) < 1e-11, "A x = b, kl={kl} ku={ku}");
        let mut y = b.clone();
        f.solve_adjoint_in_place(&mut y);
        assert!(residuo(&a.adjoint(), &y, &b) < 1e-11, "Aᴴ y = b, kl={kl} ku={ku}");
    }

    #[test]
    fn lu_de_banda_resolve_a_e_adjunta() {
        for (n, kl, ku) in [
            (1, 0, 0),
            (7, 0, 0),
            (9, 1, 1),
            (20, 3, 1),
            (20, 1, 4),
            (40, 5, 2),
            (6, 5, 5),
        ] {
            confere_lu(&banda(n, kl, ku, real), kl, ku);
            confere_lu(&banda(n, kl, ku, cplx), kl, ku);
        }
    }

    #[test]
    fn lu_de_banda_pivoteia() {
        // Diagonal minúscula: sem trocar linhas, os multiplicadores seriam ~1e16.
        let n = 12;
        let a = NdArray::from_fn(n, n, |i, j| match (i as isize - j as isize).abs() {
            0 => 1e-16,
            1 => 1.0 + 0.1 * i as f64,
            _ => 0.0,
        });
        confere_lu(&a, 1, 1);
        let f = BandLu::factor(&a, 1, 1).unwrap();
        assert_eq!(f.piv[0], 1);
    }

    #[test]
    fn lu_de_banda_detecta_pivo_nulo() {
        let mut a = banda(10, 2, 1, real);
        for i in 0..10 {
            a.set(i, 4, 0.0);
        }
        assert!(BandLu::factor(&a, 2, 1).is_none());
    }

    #[test]
    fn cholesky_de_banda() {
        for (n, kd) in [(1, 0), (5, 0), (30, 1), (30, 4), (8, 7)] {
            // B de banda ⇒ BᴴB + n·I é hermitiana definida positiva com largura kd.
            let b = banda(n, 0, kd, cplx);
            let a = NdArray::from_fn(n, n, |i, j| {
                let s: Complex64 = (0..n).map(|k| b.get(k, i).conj() * b.get(k, j)).sum();
                if i == j {
                    s + Complex64::new(n as f64, 0.0)
                } else {
                    s
                }
            });
            let f = BandChol::factor(&a, kd).expect("definida positiva");
            let rhs: Vec<Complex64> = (0..n).map(|i| Complex64::new(1.0, i as f64)).collect();
            let mut x = rhs.clone();
            f.solve_in_place(&mut x);
            assert!(residuo(&a, &x, &rhs) < 1e-11, "n={n} kd={kd}");
        }
        // Indefinida e diagonal nula.
        let ind = NdArray::from_fn(4, 4, |i, j| {
            if i == j {
                1.0
            } else if i.abs_diff(j) == 1 {
                2.0
            } else {
                0.0
            }
        });
        assert!(BandChol::factor(&ind, 1).is_none());
        let zero = NdArray::<f64>::zeros(3, 3);
        assert!(BandChol::factor(&zero, 0).is_none());
    }

    #[test]
    fn largura_de_banda_com_extremos_fora_da_coluna_zero() {
        // Os elementos mais distantes estão em colunas j > 0 (distinguem i − j de i + j).
        let mut a = NdArray::<f64>::identity(12);
        a.set(4, 2, 1.0);
        a.set(1, 6, 1.0);
        assert_eq!(bandwidth(&a), (2, 5));
        // A banda cresce em colunas posteriores (a varredura começa em j + kl + 1).
        a.set(3, 1, 1.0);
        a.set(9, 5, 1.0);
        assert_eq!(bandwidth(&a), (4, 5));
        // Retangular alta: linhas além de n.
        let mut t = NdArray::<f64>::zeros(10, 3);
        t.set(9, 2, 1.0);
        assert_eq!(bandwidth(&t), (7, 0));
    }

    #[test]
    fn pivo_em_empate_nao_troca_linhas() {
        // |a₀₀| = |a₁₀|: mantém a linha 0 (critério estrito). Na coluna 1 o maior é a linha 2.
        let a = NdArray::from_rows(&[&[2.0, 1.0, 0.0], &[-2.0, 1.0, 1.0], &[0.0, 5.0, 1.0]]).unwrap();
        let f = BandLu::factor(&a, 1, 1).unwrap();
        assert_eq!(f.piv, vec![0, 2, 2]);
        confere_lu(&a, 1, 1);
    }

    #[test]
    fn largura_de_banda() {
        let a = banda::<f64>(9, 2, 3, |s| real(s) + 1.0);
        assert_eq!(bandwidth(&a), (2, 3));
        assert_eq!(bandwidth(&NdArray::<f64>::identity(5)), (0, 0));
        assert_eq!(bandwidth(&NdArray::<f64>::zeros(0, 0)), (0, 0));
        assert_eq!(bandwidth(&NdArray::<f64>::zeros(3, 3)), (0, 0));
        let mut c = NdArray::<f64>::zeros(4, 6);
        c.set(3, 0, 1.0);
        c.set(0, 5, 1.0);
        assert_eq!(bandwidth(&c), (3, 5));
        // Elementos internos à banda já encontrada não a alteram.
        let mut d = NdArray::<f64>::identity(6);
        d.set(5, 0, 1.0);
        d.set(3, 2, 1.0);
        d.set(1, 4, 1.0);
        assert_eq!(bandwidth(&d), (5, 3));
    }
}
