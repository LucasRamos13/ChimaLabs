//! Função genérica de matriz pelo algoritmo de Schur-Parlett em blocos
//! (Davies e Higham, "A Schur-Parlett algorithm for computing matrix functions", SIAM J.
//! Matrix Anal. Appl. 25(2), 2003).
//!
//! Passos: forma de Schur complexa → agrupamento de autovalores próximos (distância ≤ δ) →
//! reordenação da forma triangular por trocas de Givens para tornar cada grupo contíguo →
//! série de Taylor em cada bloco diagonal → recorrência de Parlett em blocos (equações de
//! Sylvester triangulares) → volta à base original.

use num_complex::Complex64;

use super::matfun::{MatFunResult, RealOrComplex};
use super::{ComplexMatrix, LinScalar, LinalgError, LinalgWarning, MatrixExt, NdArray, schur_complex};

/// Funções elementares com derivadas de qualquer ordem conhecidas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elementary {
    /// Exponencial.
    Exp,
    /// Logaritmo principal.
    Log,
    /// Seno.
    Sin,
    /// Cosseno.
    Cos,
    /// Seno hiperbólico.
    Sinh,
    /// Cosseno hiperbólico.
    Cosh,
}

impl Elementary {
    /// Derivada de ordem `k` (`k = 0` é a própria função) em `z`.
    pub fn derivative(self, z: Complex64, k: usize) -> Complex64 {
        match self {
            Self::Exp => z.exp(),
            Self::Log => {
                if k == 0 {
                    z.ln()
                } else {
                    // (−1)^(k−1)·(k−1)!/zᵏ
                    let fact: f64 = (1..k).map(|v| v as f64).product();
                    let sign = if k % 2 == 1 { 1.0 } else { -1.0 };
                    Complex64::new(sign * fact, 0.0) / z.powu(k as u32)
                }
            }
            Self::Sin => match k % 4 {
                0 => z.sin(),
                1 => z.cos(),
                2 => -z.sin(),
                _ => -z.cos(),
            },
            Self::Cos => match k % 4 {
                0 => z.cos(),
                1 => -z.sin(),
                2 => -z.cos(),
                _ => z.sin(),
            },
            Self::Sinh => {
                if k.is_multiple_of(2) {
                    z.sinh()
                } else {
                    z.cosh()
                }
            }
            Self::Cosh => {
                if k.is_multiple_of(2) {
                    z.cosh()
                } else {
                    z.sinh()
                }
            }
        }
    }
}

/// Distância abaixo da qual autovalores são agrupados no mesmo bloco.
const DELTA: f64 = 0.1;
/// Número máximo de termos da série de Taylor por bloco.
const MAX_TERMS: usize = 250;

/// `f(A)` para uma função analítica dada por suas derivadas: `f(z, k)` é a derivada de ordem `k`
/// em `z`. Para entrada real, o resultado é real quando as partes imaginárias são desprezíveis.
///
/// Emite [`LinalgWarning::NearlySingular`] (com `rcond` = NaN) se a série de Taylor de algum
/// bloco não convergir em 250 termos.
pub fn funm<T: LinScalar>(
    a: &NdArray<T>,
    f: impl Fn(Complex64, usize) -> Complex64,
) -> Result<MatFunResult, LinalgError> {
    let s = schur_complex(a)?;
    let n = s.t.rows();
    if n == 0 {
        return Ok(MatFunResult {
            value: RealOrComplex::Real(NdArray::zeros(0, 0)),
            warnings: vec![],
        });
    }
    let (mut t, mut u) = (s.t, s.u);
    // 1. Agrupa autovalores (componentes conexas com distância ≤ δ).
    let eigs: Vec<Complex64> = (0..n).map(|i| t.get(i, i)).collect();
    let mut group: Vec<usize> = (0..n).collect();
    fn find(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        g[i] = r;
        r
    }
    for i in 0..n {
        for j in i + 1..n {
            if (eigs[i] - eigs[j]).norm() <= DELTA {
                let (ri, rj) = (find(&mut group, i), find(&mut group, j));
                group[ri.max(rj)] = ri.min(rj);
            }
        }
    }
    let mut label: Vec<usize> = (0..n).map(|i| find(&mut group, i)).collect();
    // 2. Reordena para grupos contíguos (ordenação estável por rótulo com trocas adjacentes).
    for pass in 0..n {
        let mut swapped = false;
        for k in 0..n - 1 - pass.min(n - 1) {
            if label[k] > label[k + 1] {
                swap_diagonal(&mut t, &mut u, k);
                label.swap(k, k + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
    let mut blocks: Vec<(usize, usize)> = vec![];
    let mut start = 0;
    for k in 1..=n {
        if k == n || label[k] != label[start] {
            blocks.push((start, k));
            start = k;
        }
    }
    // 3. Blocos diagonais por Taylor; 4. recorrência de Parlett em blocos.
    let mut fm = ComplexMatrix::zeros(n, n);
    let mut warnings = vec![];
    for &(b0, b1) in &blocks {
        let (fb, ok) = taylor_block(&t, b0, b1, &f);
        if !ok && warnings.is_empty() {
            warnings.push(LinalgWarning::NearlySingular { rcond: f64::NAN });
        }
        for i in b0..b1 {
            for j in b0..b1 {
                fm.set(i, j, fb.get(i - b0, j - b0));
            }
        }
    }
    for jb in 1..blocks.len() {
        for ib in (0..jb).rev() {
            let (i0, i1) = blocks[ib];
            let (j0, j1) = blocks[jb];
            // RHS = F_ii·T_ij − T_ij·F_jj + Σ_k (F_ik·T_kj − T_ik·F_kj)
            let mut rhs = ComplexMatrix::zeros(i1 - i0, j1 - j0);
            for r in i0..i1 {
                for c in j0..j1 {
                    let mut acc = Complex64::new(0.0, 0.0);
                    for k in i0..i1 {
                        acc += fm.get(r, k) * t.get(k, c);
                    }
                    for k in j0..j1 {
                        acc -= t.get(r, k) * fm.get(k, c);
                    }
                    for &(k0, k1) in &blocks[ib + 1..jb] {
                        for k in k0..k1 {
                            acc += fm.get(r, k) * t.get(k, c) - t.get(r, k) * fm.get(k, c);
                        }
                    }
                    rhs.set(r - i0, c - j0, acc);
                }
            }
            // Resolve T_ii·X − X·T_jj = RHS coluna a coluna (ambos triangulares superiores).
            let x = sylvester_triangular(&t, (i0, i1), (j0, j1), &rhs);
            for r in i0..i1 {
                for c in j0..j1 {
                    fm.set(r, c, x.get(r - i0, c - j0));
                }
            }
        }
    }
    let x = u.matmul(&fm)?.matmul(&u.adjoint())?;
    let value = match x.to_real(1e3 * f64::EPSILON).filter(|_| !T::IS_COMPLEX) {
        Some(m) => RealOrComplex::Real(m),
        None => RealOrComplex::Complex(x),
    };
    Ok(MatFunResult { value, warnings })
}

/// [`funm`] para uma função elementar.
pub fn funm_elementary<T: LinScalar>(a: &NdArray<T>, f: Elementary) -> Result<MatFunResult, LinalgError> {
    funm(a, |z, k| f.derivative(z, k))
}

/// Troca os elementos diagonais `k` e `k+1` da forma triangular `t` com uma rotação unitária,
/// atualizando `u` para preservar `A = U·T·Uᴴ`.
fn swap_diagonal(t: &mut ComplexMatrix, u: &mut ComplexMatrix, k: usize) {
    let n = t.rows();
    let (a, b, x) = (t.get(k, k), t.get(k + 1, k + 1), t.get(k, k + 1));
    // Autovetor do bloco para o autovalor b: v = [x; b − a]. G·v = [r; 0].
    let (c, s) = givens(x, b - a);
    for j in k..n {
        let (p, q) = (t.get(k, j), t.get(k + 1, j));
        t.set(k, j, p * c + s * q);
        t.set(k + 1, j, -s.conj() * p + q * c);
    }
    for i in 0..(k + 2) {
        let (p, q) = (t.get(i, k), t.get(i, k + 1));
        t.set(i, k, p * c + s.conj() * q);
        t.set(i, k + 1, -s * p + q * c);
    }
    for i in 0..n {
        let (p, q) = (u.get(i, k), u.get(i, k + 1));
        u.set(i, k, p * c + s.conj() * q);
        u.set(i, k + 1, -s * p + q * c);
    }
    t.set(k + 1, k, Complex64::new(0.0, 0.0));
}

/// Rotação de Givens complexa `G = [[c, s], [−s̄, c]]` (c real) com `G·[x; y] = [r; 0]`.
fn givens(x: Complex64, y: Complex64) -> (f64, Complex64) {
    let (ax, ay) = (x.norm(), y.norm());
    if ay == 0.0 {
        return (1.0, Complex64::new(0.0, 0.0));
    }
    if ax == 0.0 {
        return (0.0, y.conj() / ay);
    }
    let r = ax.hypot(ay);
    (ax / r, (x / ax) * y.conj() / r)
}

/// Série de Taylor de `f` no bloco diagonal `t[b0..b1, b0..b1]`, em torno da média dos autovalores.
fn taylor_block(
    t: &ComplexMatrix,
    b0: usize,
    b1: usize,
    f: &impl Fn(Complex64, usize) -> Complex64,
) -> (ComplexMatrix, bool) {
    let m = b1 - b0;
    let blk = ComplexMatrix::from_fn(m, m, |i, j| t.get(b0 + i, b0 + j));
    if m == 1 {
        return (ComplexMatrix::from_fn(1, 1, |_, _| f(blk.get(0, 0), 0)), true);
    }
    let sigma: Complex64 = (0..m).map(|i| blk.get(i, i)).sum::<Complex64>() / m as f64;
    let mshift = ComplexMatrix::from_fn(m, m, |i, j| {
        if i == j {
            blk.get(i, j) - sigma
        } else {
            blk.get(i, j)
        }
    });
    // Bloco nilpotente (todos os autovalores iguais a σ): a série é finita e Mᵐ = 0 exatamente.
    let nilpotent = (0..m).all(|i| mshift.get(i, i) == Complex64::new(0.0, 0.0));
    let mut result = ComplexMatrix::identity(m).map(|z| z * f(sigma, 0));
    let mut power = ComplexMatrix::identity(m);
    let mut fact = 1.0;
    let mut small_run = 0;
    for k in 1..MAX_TERMS {
        power = power.matmul(&mshift).expect("formas iguais");
        fact *= k as f64;
        let coef = f(sigma, k) / fact;
        // Regressões: k! estoura em k = 171 (coeficiente viraria 0) e coeficientes de derivadas
        // podem estourar; em ambos os casos a série NÃO convergiu.
        if !fact.is_finite() || !coef.is_finite() {
            return (result, false);
        }
        let term = power.map(|z| z * coef);
        result = result.add(&term).expect("formas iguais");
        if term.max_abs() <= f64::EPSILON * result.max_abs() && k >= m {
            small_run += 1;
            if small_run >= 2 {
                return (result, true);
            }
        } else {
            small_run = 0;
        }
        if power.max_abs() == 0.0 {
            // Potência nula: série finita se o bloco for nilpotente; senão foi underflow de Mᵏ
            // com coeficientes ainda grandes (regressão: truncamento silencioso).
            return (result, nilpotent);
        }
    }
    (result, false)
}

/// Resolve `A·X − X·B = C` com `A = t[i0..i1, i0..i1]` e `B = t[j0..j1, j0..j1]` triangulares
/// superiores com espectros disjuntos (garantido pelo agrupamento).
fn sylvester_triangular(
    t: &ComplexMatrix,
    (i0, i1): (usize, usize),
    (j0, j1): (usize, usize),
    c: &ComplexMatrix,
) -> ComplexMatrix {
    let (p, q) = (i1 - i0, j1 - j0);
    let mut x = ComplexMatrix::zeros(p, q);
    for col in 0..q {
        // (A − b_cc·I)·x_c = c_c + Σ_{r<c} x_r·b_rc
        let bcc = t.get(j0 + col, j0 + col);
        let mut rhs: Vec<Complex64> = (0..p).map(|r| c.get(r, col)).collect();
        for r in 0..col {
            let brc = t.get(j0 + r, j0 + col);
            for (i, v) in rhs.iter_mut().enumerate() {
                *v += x.get(i, r) * brc;
            }
        }
        for i in (0..p).rev() {
            let s: Complex64 = (i + 1..p).map(|k| t.get(i0 + i, i0 + k) * x.get(k, col)).sum();
            x.set(i, col, (rhs[i] - s) / (t.get(i0 + i, i0 + i) - bcc));
        }
    }
    x
}
