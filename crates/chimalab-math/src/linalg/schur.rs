//! Formas de Hessenberg e de Schur, LDLᵀ (Bunch-Kaufman) e autovalores generalizados.

use faer::dyn_stack::{MemBuffer, MemStack};
use faer::linalg::evd::hessenberg;
use faer::linalg::householder;
use faer::linalg::qr::no_pivoting::factor::recommended_block_size;
use faer::traits::ComplexField;
use faer::{Conj, Mat, MatRef, Side, get_global_parallelism};
use num_complex::Complex64;

use super::{ComplexMatrix, LinScalar, LinalgError, Matrix, MatrixExt, NdArray, Scalar};

/// Forma de Hessenberg `A = P·H·Pᵀ`.
#[derive(Clone, Debug)]
pub struct Hess<T: LinScalar> {
    /// Matriz unitária `P` (ortogonal no caso real).
    pub p: NdArray<T>,
    /// Matriz de Hessenberg superior `H` (zeros abaixo da primeira subdiagonal).
    pub h: NdArray<T>,
}

/// Forma de Schur real `A = U·T·Uᵀ`, com `T` quase triangular superior
/// (blocos 2×2 na diagonal para pares de autovalores complexos conjugados).
#[derive(Clone, Debug)]
pub struct Schur {
    /// Matriz ortogonal `U`.
    pub u: Matrix,
    /// Matriz quase triangular superior `T`.
    pub t: Matrix,
}

/// Forma de Schur complexa `A = U·T·Uᴴ`, com `T` triangular superior.
#[derive(Clone, Debug)]
pub struct ComplexSchur {
    /// Matriz unitária `U`.
    pub u: ComplexMatrix,
    /// Matriz triangular superior `T` (autovalores na diagonal).
    pub t: ComplexMatrix,
}

/// Decomposição simétrica indefinida `P·A·Pᵀ = L·D·Lᵀ` (pivotamento de Bunch-Kaufman).
#[derive(Clone, Debug)]
pub struct Ldl<T: LinScalar> {
    /// Fator triangular inferior com diagonal unitária.
    pub l: NdArray<T>,
    /// Matriz bloco-diagonal simétrica com blocos 1×1 ou 2×2.
    pub d: NdArray<T>,
    /// Permutação simétrica: a linha/coluna `i` de `P·A·Pᵀ` é a `perm[i]` de `A`.
    pub perm: Vec<usize>,
}

impl<T: LinScalar> Ldl<T> {
    /// Matriz de permutação `P` explícita.
    pub fn p_matrix(&self) -> NdArray<T> {
        let n = self.perm.len();
        NdArray::from_fn(n, n, |i, j| if self.perm[i] == j { T::one() } else { T::zero() })
    }
}

/// Autovalores generalizados de `(A, B)`: `A·v = λ·B·v`, com `λ = α/β`.
#[derive(Clone, Debug)]
pub struct GenEig {
    /// Numeradores `α`.
    pub alpha: Vec<Complex64>,
    /// Denominadores `β` (zero indica autovalor infinito).
    pub beta: Vec<Complex64>,
    /// Autovalores `α/β` (infinitos quando `β = 0`).
    pub values: Vec<Complex64>,
    /// Autovetores à direita (colunas).
    pub vectors: ComplexMatrix,
}

fn square_finite<T: LinScalar>(a: &NdArray<T>, operation: &'static str) -> Result<(), LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation,
            shape: a.shape(),
        });
    }
    if !a.all_finite() {
        return Err(LinalgError::NonFinite { operation });
    }
    Ok(())
}

/// Redução de Hessenberg com acumulação da matriz unitária: devolve `(H, Z)` com `A = Z·H·Zᴴ`.
fn hessenberg_factor<T: ComplexField>(a: MatRef<'_, T>) -> (Mat<T>, Mat<T>) {
    let n = a.nrows();
    let mut h = a.to_owned();
    let mut z = Mat::<T>::identity(n, n);
    if n <= 2 {
        return (h, z);
    }
    let par = get_global_parallelism();
    let bs = recommended_block_size::<T>(n - 1, n - 1);
    let mut hh = Mat::<T>::zeros(bs, n - 1);
    let req = hessenberg::hessenberg_in_place_scratch::<T>(n, bs, par, Default::default()).or(
        householder::apply_block_householder_sequence_on_the_right_in_place_scratch::<T>(n - 1, bs, n - 1),
    );
    let mut buf = MemBuffer::new(req);
    let stack = MemStack::new(&mut buf);
    hessenberg::hessenberg_in_place(h.as_mut(), hh.as_mut(), par, stack, Default::default());
    householder::apply_block_householder_sequence_on_the_right_in_place_with_conj(
        h.as_ref().submatrix(1, 0, n - 1, n - 1),
        hh.as_ref(),
        Conj::No,
        z.as_mut().submatrix_mut(1, 1, n - 1, n - 1),
        par,
        stack,
    );
    for j in 0..n {
        for i in j + 2..n {
            h[(i, j)] = T::zero_impl();
        }
    }
    (h, z)
}

/// Forma de Hessenberg `A = P·H·Pᵀ`.
pub fn hess<T: LinScalar>(a: &NdArray<T>) -> Result<Hess<T>, LinalgError> {
    square_finite(a, "hess")?;
    let (h, z) = hessenberg_factor(a.as_faer());
    Ok(Hess {
        p: NdArray::from_faer(z.as_ref()),
        h: NdArray::from_faer(h.as_ref()),
    })
}

/// Forma de Schur real (quase triangular): redução de Hessenberg seguida de iterações QR
/// implícitas de passo duplo (Francis), com deflação; blocos 2×2 com autovalores reais são
/// triangularizados, de modo que só pares complexos conjugados permanecem em blocos 2×2.
pub fn schur(a: &Matrix) -> Result<Schur, LinalgError> {
    square_finite(a, "schur")?;
    let n = a.rows();
    if n == 0 {
        return Ok(Schur {
            u: Matrix::zeros(0, 0),
            t: Matrix::zeros(0, 0),
        });
    }
    let (h, z) = hessenberg_factor(a.as_faer());
    let mut t = Matrix::from_faer(h.as_ref());
    let mut u = Matrix::from_faer(z.as_ref());
    francis(t.as_mut_slice(), u.as_mut_slice(), n)?;
    Ok(Schur { u, t })
}

/// Forma de Schur complexa (triangular superior), obtida da forma real triangularizando cada
/// bloco 2×2 com uma rotação unitária construída a partir de um autovetor do bloco.
pub fn schur_complex<T: LinScalar>(a: &NdArray<T>) -> Result<ComplexSchur, LinalgError> {
    if T::IS_COMPLEX {
        square_finite(a, "schur")?;
        let ac = a.to_complex();
        return complex_qr_schur(&ac);
    }
    let a: Matrix = a.map(|x| x.re());
    let Schur { u, t } = schur(&a)?;
    let n = t.rows();
    let mut tc = ComplexMatrix::from_real(&t);
    let mut uc = ComplexMatrix::from_real(&u);
    let mut k = 0;
    while k + 1 < n {
        if t.get(k + 1, k) == 0.0 {
            k += 1;
            continue;
        }
        let (a11, a12, a21, a22) = (t.get(k, k), t.get(k, k + 1), t.get(k + 1, k), t.get(k + 1, k + 1));
        let p = 0.5 * (a11 - a22);
        let disc = p * p + a12 * a21; // negativo: par complexo
        let lambda = Complex64::new(0.5 * (a11 + a22), (-disc).max(0.0).sqrt());
        // Autovetor (a12, λ − a11) normalizado.
        let v0 = Complex64::new(a12, 0.0);
        let v1 = lambda - a11;
        let nv = (v0.norm_sqr() + v1.norm_sqr()).sqrt();
        let (v0, v1) = (v0 / nv, v1 / nv);
        // Q = [[v0, −conj(v1)], [v1, conj(v0)]] (unitária, primeira coluna = autovetor).
        let q = [[v0, -v1.conj()], [v1, v0.conj()]];
        // T ← Qᴴ·T (linhas k, k+1)
        for j in 0..n {
            let (x, y) = (tc.get(k, j), tc.get(k + 1, j));
            tc.set(k, j, q[0][0].conj() * x + q[1][0].conj() * y);
            tc.set(k + 1, j, q[0][1].conj() * x + q[1][1].conj() * y);
        }
        // T ← T·Q e U ← U·Q (colunas k, k+1)
        for m in [&mut tc, &mut uc] {
            for i in 0..n {
                let (x, y) = (m.get(i, k), m.get(i, k + 1));
                m.set(i, k, x * q[0][0] + y * q[1][0]);
                m.set(i, k + 1, x * q[0][1] + y * q[1][1]);
            }
        }
        tc.set(k + 1, k, Complex64::new(0.0, 0.0));
        k += 2;
    }
    for j in 0..n {
        for i in j + 1..n {
            tc.set(i, j, Complex64::new(0.0, 0.0));
        }
    }
    Ok(ComplexSchur { u: uc, t: tc })
}

/// Refletor de Householder `P = I − β·v·vᵀ` com `P·x = α·e₁` (tamanho fixo: sem alocação).
fn householder_vec<const N: usize>(x: [f64; N]) -> ([f64; N], f64) {
    let norm = x.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm == 0.0 {
        return ([0.0; N], 0.0);
    }
    let alpha = if x[0] >= 0.0 { -norm } else { norm };
    let mut v = x;
    v[0] -= alpha;
    let vtv: f64 = v.iter().map(|a| a * a).sum();
    (v, if vtv == 0.0 { 0.0 } else { 2.0 / vtv })
}

/// Aplica `P` pela esquerda em `m` (n×n, ordem de colunas): linhas `r0..r0+N`, colunas `c0..c1`.
/// Cada coluna é um trecho contíguo de N elementos.
fn reflect_left<const N: usize>(
    m: &mut [f64],
    n: usize,
    r0: usize,
    v: &[f64; N],
    beta: f64,
    c0: usize,
    c1: usize,
) {
    for j in c0..c1 {
        let col = &mut m[j * n + r0..j * n + r0 + N];
        let s = v.iter().zip(col.iter()).map(|(vi, x)| vi * x).sum::<f64>() * beta;
        for (x, vi) in col.iter_mut().zip(v) {
            *x -= s * vi;
        }
    }
}

/// Aplica `P` pela direita em `m` (n×n, ordem de colunas): colunas `c0..c0+N`, linhas `r0..r1`.
/// O laço interno percorre as N colunas, que são trechos contíguos.
fn reflect_right<const N: usize>(
    m: &mut [f64],
    n: usize,
    c0: usize,
    v: &[f64; N],
    beta: f64,
    r0: usize,
    r1: usize,
) {
    let block = &mut m[c0 * n..(c0 + N) * n];
    let (c_0, rest) = block.split_at_mut(n);
    let c_0 = &mut c_0[r0..r1];
    if N == 2 {
        let c_1 = &mut rest[r0..r1];
        for (a, b) in c_0.iter_mut().zip(c_1.iter_mut()) {
            let s = (v[0] * *a + v[1] * *b) * beta;
            *a -= s * v[0];
            *b -= s * v[1];
        }
    } else {
        let (c_1, c_2) = rest.split_at_mut(n);
        let (c_1, c_2) = (&mut c_1[r0..r1], &mut c_2[r0..r1]);
        for ((a, b), c) in c_0.iter_mut().zip(c_1.iter_mut()).zip(c_2.iter_mut()) {
            let s = (v[0] * *a + v[1] * *b + v[2] * *c) * beta;
            *a -= s * v[0];
            *b -= s * v[1];
            *c -= s * v[2];
        }
    }
}

/// Iterações QR de passo duplo de Francis sobre `h` (Hessenberg, n×n em ordem de colunas),
/// acumulando em `z`.
fn francis(h: &mut [f64], z: &mut [f64], n: usize) -> Result<(), LinalgError> {
    let at = |i: usize, j: usize| i + j * n;
    let eps = f64::EPSILON;
    let hnorm = h
        .iter()
        .fold(0.0_f64, |m, x| m.max(x.abs()))
        .max(f64::MIN_POSITIVE);
    let mut hi = n as isize - 1;
    let mut iter = 0usize;
    let mut total = 0usize;
    while hi >= 0 {
        let hiu = hi as usize;
        // Procura o início `l` do bloco não reduzido que termina em `hi`.
        let mut l = hiu;
        while l > 0 {
            let mut s = h[at(l - 1, l - 1)].abs() + h[at(l, l)].abs();
            if s == 0.0 {
                s = hnorm;
            }
            if h[at(l, l - 1)].abs() <= eps * s {
                h[at(l, l - 1)] = 0.0;
                break;
            }
            l -= 1;
        }
        if l == hiu {
            hi -= 1;
            iter = 0;
            continue;
        }
        if l + 1 == hiu {
            standardize_2x2(h, z, n, l);
            hi -= 2;
            iter = 0;
            continue;
        }
        iter += 1;
        total += 1;
        if total > 40 * n.max(10) {
            return Err(LinalgError::NoConvergence { operation: "schur" });
        }
        let m = hiu;
        let (s, t) = double_shift(h, n, m, exceptional_step(iter));
        let mut x = h[at(l, l)] * h[at(l, l)] + h[at(l, l + 1)] * h[at(l + 1, l)] - s * h[at(l, l)] + t;
        let mut y = h[at(l + 1, l)] * (h[at(l, l)] + h[at(l + 1, l + 1)] - s);
        let mut w = h[at(l + 1, l)] * h[at(l + 2, l + 1)];
        for k in l..=m - 2 {
            let (v, beta) = householder_vec([x, y, w]);
            let q = k.saturating_sub(1).max(l); // k − 1, ou l no primeiro passo
            reflect_left(h, n, k, &v, beta, q, n);
            let r = (k + 3).min(m);
            reflect_right(h, n, k, &v, beta, 0, r + 1);
            reflect_right(z, n, k, &v, beta, 0, n);
            if k > l {
                h[at(k + 1, k - 1)] = 0.0;
                h[at(k + 2, k - 1)] = 0.0;
            }
            x = h[at(k + 1, k)];
            y = h[at(k + 2, k)];
            if k + 3 <= m {
                w = h[at(k + 3, k)];
            }
        }
        let (v, beta) = householder_vec([x, y]);
        reflect_left(h, n, m - 1, &v, beta, m - 2, n);
        reflect_right(h, n, m - 1, &v, beta, 0, m + 1);
        reflect_right(z, n, m - 1, &v, beta, 0, n);
        h[at(m, m - 2)] = 0.0;
    }
    Ok(())
}

/// Verdadeiro na `iter`-ésima iteração seguida sem deflação em que se usa o deslocamento
/// excepcional: 10, 21, 32, … (uma a cada 11).
fn exceptional_step(iter: usize) -> bool {
    iter % 11 == 10
}

/// Soma `s` e produto `t` do par de deslocamentos para o bloco ativo que termina em `m ≥ 2`:
/// os autovalores do bloco 2×2 final ou, a cada 11 iterações sem deflação, o deslocamento
/// excepcional `s = 1,5·w`, `t = w²` com `w = |h(m, m−1)| + |h(m−1, m−2)|`, que rompe ciclos.
fn double_shift(h: &[f64], n: usize, m: usize, exceptional: bool) -> (f64, f64) {
    let at = |i: usize, j: usize| i + j * n;
    if exceptional {
        let w = h[at(m, m - 1)].abs() + h[at(m - 1, m - 2)].abs();
        (1.5 * w, w * w)
    } else {
        (
            h[at(m - 1, m - 1)] + h[at(m, m)],
            h[at(m - 1, m - 1)] * h[at(m, m)] - h[at(m - 1, m)] * h[at(m, m - 1)],
        )
    }
}

/// Triangulariza o bloco 2×2 em `(k, k)` quando seus autovalores são reais.
fn standardize_2x2(h: &mut [f64], z: &mut [f64], n: usize, k: usize) {
    let at = |i: usize, j: usize| i + j * n;
    let (a, b, c, d) = (h[at(k, k)], h[at(k, k + 1)], h[at(k + 1, k)], h[at(k + 1, k + 1)]);
    if c == 0.0 {
        return;
    }
    let p = 0.5 * (a - d);
    let disc = p * p + b * c;
    if disc < 0.0 {
        return; // par complexo conjugado: mantém o bloco
    }
    let lambda = 0.5 * (a + d) + if p >= 0.0 { disc.sqrt() } else { -disc.sqrt() };
    // Autovetor de [[a, b], [c, d]] para λ.
    let (e1, e2) = ((b, lambda - a), (lambda - d, c));
    let (vx, vy) = if e1.0.hypot(e1.1) >= e2.0.hypot(e2.1) {
        e1
    } else {
        e2
    };
    let nv = vx.hypot(vy);
    let (cs, sn) = (vx / nv, vy / nv);
    // G = [[cs, −sn], [sn, cs]]; T ← Gᵀ·T·G, Z ← Z·G
    for j in k..n {
        let (x, y) = (h[at(k, j)], h[at(k + 1, j)]);
        h[at(k, j)] = cs * x + sn * y;
        h[at(k + 1, j)] = -sn * x + cs * y;
    }
    for i in 0..(k + 2) {
        let (x, y) = (h[at(i, k)], h[at(i, k + 1)]);
        h[at(i, k)] = cs * x + sn * y;
        h[at(i, k + 1)] = -sn * x + cs * y;
    }
    for i in 0..n {
        let (x, y) = (z[at(i, k)], z[at(i, k + 1)]);
        z[at(i, k)] = cs * x + sn * y;
        z[at(i, k + 1)] = -sn * x + cs * y;
    }
    h[at(k + 1, k)] = 0.0;
}

/// Decomposição `P·A·Pᴴ = L·D·Lᴴ` de matriz hermitiana (simétrica, se real), possivelmente
/// indefinida. Usa apenas o triângulo inferior de `A`.
pub fn ldl<T: LinScalar>(a: &NdArray<T>) -> Result<Ldl<T>, LinalgError> {
    square_finite(a, "ldl")?;
    let n = a.rows();
    if n == 0 {
        return Ok(Ldl {
            l: NdArray::zeros(0, 0),
            d: NdArray::zeros(0, 0),
            perm: vec![],
        });
    }
    let f = a.as_faer().lblt(Side::Lower);
    let l = f.L();
    let bd = f.B_diag().column_vector();
    let bs = f.B_subdiag().column_vector();
    let d = NdArray::from_fn(n, n, |i, j| {
        if i == j {
            bd[i]
        } else if i == j + 1 {
            bs[j]
        } else if j == i + 1 {
            Scalar::conj(bs[i])
        } else {
            T::zero()
        }
    });
    Ok(Ldl {
        l: NdArray::from_fn(n, n, |i, j| {
            if i == j {
                T::one()
            } else if i > j {
                l[(i, j)]
            } else {
                T::zero()
            }
        }),
        d,
        perm: f.P().arrays().0.to_vec(),
    })
}

/// Autovalores e autovetores generalizados de `(A, B)` (algoritmo QZ).
pub fn geig<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>) -> Result<GenEig, LinalgError> {
    square_finite(a, "geig")?;
    square_finite(b, "geig")?;
    if a.shape() != b.shape() {
        return Err(LinalgError::DimensionMismatch {
            operation: "geig",
            left: a.shape(),
            right: b.shape(),
        });
    }
    let n = a.rows();
    if n == 0 {
        return Ok(GenEig {
            alpha: vec![],
            beta: vec![],
            values: vec![],
            vectors: ComplexMatrix::zeros(0, 0),
        });
    }
    let f = a
        .as_faer()
        .generalized_eigen(b.as_faer())
        .map_err(|_| LinalgError::NoConvergence { operation: "geig" })?;
    let sa = f.S_a().column_vector();
    let sb = f.S_b().column_vector();
    let alpha: Vec<Complex64> = (0..n).map(|i| sa[i]).collect();
    let beta: Vec<Complex64> = (0..n).map(|i| sb[i]).collect();
    let values = alpha
        .iter()
        .zip(&beta)
        .map(|(&al, &be)| {
            if be.norm() == 0.0 {
                Complex64::new(f64::INFINITY, 0.0)
            } else {
                al / be
            }
        })
        .collect();
    Ok(GenEig {
        alpha,
        beta,
        values,
        vectors: ComplexMatrix::from_faer(f.U()),
    })
}

/// Forma de Schur complexa de matriz complexa: redução de Hessenberg seguida de iterações QR
/// implícitas de deslocamento simples (Wilkinson), com rotações de Givens complexas e deflação.
fn complex_qr_schur(a: &ComplexMatrix) -> Result<ComplexSchur, LinalgError> {
    let n = a.rows();
    if n == 0 {
        return Ok(ComplexSchur {
            u: ComplexMatrix::zeros(0, 0),
            t: ComplexMatrix::zeros(0, 0),
        });
    }
    let (h, z) = hessenberg_factor(a.as_faer());
    let mut tm = ComplexMatrix::from_faer(h.as_ref());
    let mut um = ComplexMatrix::from_faer(z.as_ref());
    complex_qr(tm.as_mut_slice(), um.as_mut_slice(), n)?;
    Ok(ComplexSchur { u: um, t: tm })
}

/// Iterações QR complexas sobre `t` (Hessenberg, n×n em ordem de colunas), acumulando em `u`;
/// ao fim, `t` é triangular superior.
fn complex_qr(t: &mut [Complex64], u: &mut [Complex64], n: usize) -> Result<(), LinalgError> {
    let at = |i: usize, j: usize| i + j * n;
    let zero = Complex64::new(0.0, 0.0);
    let eps = f64::EPSILON;
    let tnorm = t
        .iter()
        .fold(0.0_f64, |m, x| m.max(x.norm()))
        .max(f64::MIN_POSITIVE);
    let mut hi = n - 1;
    let mut iter = 0usize;
    let mut total = 0usize;
    while hi > 0 {
        let mut l = hi;
        while l > 0 {
            let mut sc = t[at(l - 1, l - 1)].norm() + t[at(l, l)].norm();
            if sc == 0.0 {
                sc = tnorm;
            }
            if t[at(l, l - 1)].norm() <= eps * sc {
                t[at(l, l - 1)] = zero;
                break;
            }
            l -= 1;
        }
        if l == hi {
            hi -= 1;
            iter = 0;
            continue;
        }
        iter += 1;
        total += 1;
        if total > 60 * n.max(10) {
            return Err(LinalgError::NoConvergence { operation: "schur" });
        }
        let mu = wilkinson_shift(t, n, hi, exceptional_step(iter));
        for k in l..hi {
            // No primeiro passo, a rotação vem do deslocamento; nos demais, persegue o bojo em
            // t(k + 1, k − 1).
            let bulge = k > l;
            let (x, y) = if bulge {
                (t[at(k, k - 1)], t[at(k + 1, k - 1)])
            } else {
                (t[at(l, l)] - mu, t[at(l + 1, l)])
            };
            let (c, s) = givens(x, y);
            // T ← G·T nas linhas k, k+1
            let c0 = if bulge { k - 1 } else { l };
            for j in c0..n {
                let (a1, a2) = (t[at(k, j)], t[at(k + 1, j)]);
                t[at(k, j)] = a1 * c + s * a2;
                t[at(k + 1, j)] = -s.conj() * a1 + a2 * c;
            }
            // T ← T·Gᴴ e U ← U·Gᴴ nas colunas k, k+1
            rotate_columns(t, n, k, c, s, (k + 2).min(hi) + 1);
            rotate_columns(u, n, k, c, s, n);
            if bulge {
                t[at(k + 1, k - 1)] = zero;
            }
        }
    }
    for j in 0..n {
        for i in j + 1..n {
            t[at(i, j)] = zero;
        }
    }
    Ok(())
}

/// Deslocamento para o bloco ativo que termina em `hi ≥ 1`: o autovalor do bloco 2×2 final mais
/// próximo de `t(hi, hi)` (Wilkinson) ou, a cada 11 iterações sem deflação, o excepcional
/// `t(hi, hi) + |t(hi, hi−1)|`.
fn wilkinson_shift(t: &[Complex64], n: usize, hi: usize, exceptional: bool) -> Complex64 {
    let at = |i: usize, j: usize| i + j * n;
    if exceptional {
        return t[at(hi, hi)] + Complex64::new(t[at(hi, hi - 1)].norm(), 0.0);
    }
    let (p, q, r, s4) = (
        t[at(hi - 1, hi - 1)],
        t[at(hi - 1, hi)],
        t[at(hi, hi - 1)],
        t[at(hi, hi)],
    );
    let half = (p - s4) * 0.5;
    let disc = (half * half + q * r).sqrt();
    let (m1, m2) = ((p + s4) * 0.5 + disc, (p + s4) * 0.5 - disc);
    if (m1 - s4).norm() <= (m2 - s4).norm() {
        m1
    } else {
        m2
    }
}

/// `M ← M·Gᴴ` nas colunas `k, k+1`, linhas `0..rows` (colunas contíguas).
fn rotate_columns(m: &mut [Complex64], n: usize, k: usize, c: f64, s: Complex64, rows: usize) {
    let (left, right) = m[k * n..(k + 2) * n].split_at_mut(n);
    for (a1, a2) in left[..rows].iter_mut().zip(right[..rows].iter_mut()) {
        let (x, y) = (*a1, *a2);
        *a1 = x * c + s.conj() * y;
        *a2 = -s * x + y * c;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Matriz n×n em ordem de colunas a partir das linhas.
    fn cols(rows: &[&[f64]]) -> Vec<f64> {
        let n = rows.len();
        (0..n * n).map(|k| rows[k % n][k / n]).collect()
    }

    #[test]
    fn deslocamentos_normal_e_excepcional() {
        // 6×6 com elementos distintos, potências de 2 (somas e produtos exatos), e m = 5 (com
        // m = 3 ou 4, m − 2 coincidiria com m/2): qualquer troca de índice muda o resultado.
        let p = |k: i32| 2f64.powi(k);
        let h: Vec<f64> = (0..36)
            .map(|idx| {
                let (i, j) = (idx % 6, idx / 6);
                if i > j + 1 {
                    0.0
                } else {
                    p(i * 6 + j) * if (i + j) % 3 == 0 { -1.0 } else { 1.0 }
                }
            })
            .collect();
        let at = |i: usize, j: usize| h[i + j * 6];
        let (a, b, c, d) = (at(4, 4), at(4, 5), at(5, 4), at(5, 5));
        // Bloco final [[a, b], [c, d]]: traço e determinante.
        assert_eq!(double_shift(&h, 6, 5, false), (a + d, a * d - b * c));
        // w = |h(5, 4)| + |h(4, 3)|.
        let w = at(5, 4).abs() + at(4, 3).abs();
        assert_eq!(double_shift(&h, 6, 5, true), (1.5 * w, w * w));
    }

    #[test]
    fn agenda_do_deslocamento_excepcional() {
        let steps: Vec<usize> = (0..45).filter(|&i| exceptional_step(i)).collect();
        assert_eq!(steps, vec![10, 21, 32, 43]);
    }

    #[test]
    fn deflacao_no_limite_do_criterio() {
        // |h(1, 0)| = 1e-16 ≤ ε·(|h(0, 0)| + |h(1, 1)|) = ε·(1 + 1e-20): deflação imediata, sem
        // nenhuma rotação (Z = I exatamente). Com a escala tomada de outros elementos (por
        // exemplo |h(0, 1)| + |h(1, 1)| = 1e-20), o bloco seria rodado e Z ≠ I.
        let mut h = cols(&[&[1.0, 0.0], &[1e-16, 1e-20]]);
        let mut z = cols(&[&[1.0, 0.0], &[0.0, 1.0]]);
        francis(&mut h, &mut z, 2).unwrap();
        assert_eq!(h, cols(&[&[1.0, 0.0], &[0.0, 1e-20]]));
        assert_eq!(z, cols(&[&[1.0, 0.0], &[0.0, 1.0]]));
        // Logo acima do limite (3e-16 > ε): não há deflação; o bloco tem autovalores reais e é
        // triangularizado por rotação.
        let mut h = cols(&[&[1.0, 0.0], &[3e-16, 1e-20]]);
        let mut z = cols(&[&[1.0, 0.0], &[0.0, 1.0]]);
        francis(&mut h, &mut z, 2).unwrap();
        assert_eq!(h[1], 0.0);
        assert!(z[1] != 0.0);
    }

    fn c(re: f64) -> Complex64 {
        Complex64::new(re, 0.0)
    }

    #[test]
    fn deslocamento_de_wilkinson_e_excepcional() {
        // Bloco final [[5, 3], [4, 1]]: autovalores 3 ± 4 = 7 e −1; o mais próximo de 1 é −1.
        // Os demais elementos são distintos, para que qualquer troca de índice mude o valor.
        let t: Vec<Complex64> = cols(&[&[7.0, 9.0, 11.0], &[13.0, 5.0, 3.0], &[0.0, 4.0, 1.0]])
            .into_iter()
            .map(c)
            .collect();
        assert!((wilkinson_shift(&t, 3, 2, false) - c(-1.0)).norm() < 1e-14);
        // Com t(hi, hi) = 6 (bloco [[5, 3], [4, 6]]: 5,5 ± 3,5 = 9 e 2), o mais próximo de 6 é 9.
        let mut t2 = t.clone();
        t2[8] = c(6.0);
        assert!((wilkinson_shift(&t2, 3, 2, false) - c(9.0)).norm() < 1e-14);
        // Casos em que a comparação |m₁ − t(hi, hi)| ≤ |m₂ − t(hi, hi)| decide por pouco:
        // [[10, 2], [3, 5]] → 11 e 4 (escolhe 4, perto de 5); [[13, 4], [6, 8]] → 16 e 5 (escolhe 5,
        // perto de 8).
        for (blk, want) in [([10.0, 2.0, 3.0, 5.0], 4.0), ([13.0, 4.0, 6.0, 8.0], 5.0)] {
            let mut t3 = t.clone();
            t3[4] = c(blk[0]);
            t3[7] = c(blk[1]);
            t3[5] = c(blk[2]);
            t3[8] = c(blk[3]);
            assert!(
                (wilkinson_shift(&t3, 3, 2, false) - c(want)).norm() < 1e-14,
                "{blk:?}"
            );
        }
        // Excepcional: t(2, 2) + |t(2, 1)| = 1 + 4.
        assert_eq!(wilkinson_shift(&t, 3, 2, true), c(5.0));
    }

    #[test]
    fn deflacao_complexa_no_limite_do_criterio() {
        // |t(1, 0)| = 4e-16 ≤ ε·(|t(0, 0)| + |t(1, 1)|) ≈ 8,9e-16: deflação sem rotação (U = I).
        // Uma escala tomada de outro elemento (|t(1, 0)|, |t(0, 1)| = 0) ou ε/escala
        // (≈ 5,6e-17) não deflacionaria.
        let mut t: Vec<Complex64> = cols(&[&[4.0, 0.0], &[4e-16, 1e-20]]).into_iter().map(c).collect();
        let id: Vec<Complex64> = cols(&[&[1.0, 0.0], &[0.0, 1.0]]).into_iter().map(c).collect();
        let mut u = id.clone();
        complex_qr(&mut t, &mut u, 2).unwrap();
        assert_eq!(u, id);
        assert_eq!(t[1], c(0.0));
        assert_eq!((t[0], t[3]), (c(4.0), c(1e-20)));
        // Acima do limite (1e-15): há iteração QR e U deixa de ser a identidade.
        let mut t: Vec<Complex64> = cols(&[&[4.0, 0.0], &[1e-15, 1e-20]]).into_iter().map(c).collect();
        let mut u = id.clone();
        complex_qr(&mut t, &mut u, 2).unwrap();
        assert!(u != id);
    }
}
