//! Exponencial de matriz por escalonamento e quadratura com aproximantes de Padé.
//!
//! Algoritmo de Al-Mohy & Higham ("A new scaling and squaring algorithm for the matrix
//! exponential", SIAM J. Matrix Anal. Appl. 31(3), 2009), que refina o de Higham (SIAM J. Matrix
//! Anal. Appl. 26(4), 2005):
//! - o grau m e o escalonamento s são escolhidos por `η = max(‖Aᵏ‖^{1/k}, ‖Aᵏ⁺¹‖^{1/(k+1)})`, que
//!   pode ser muito menor que ‖A‖ em matrizes não normais, em vez de ‖A‖₁;
//! - um termo de correção ℓ(A, m) aumenta s quando a avaliação do Padé seria imprecisa;
//! - em matrizes triangulares superiores, a diagonal e a primeira superdiagonal são recalculadas
//!   exatamente em cada etapa da quadratura.
//!
//! Isso evita o superescalonamento: com s grande demais, as s quadraturas amplificam os erros de
//! arredondamento.

use faer::linalg::matmul::matmul;
use faer::{Accum, Par};
use num_complex::Complex64;

use super::rcond::{norm1, norm1_estimate};
use super::{LinScalar, LinalgError, MatrixExt, NdArray, Scalar, solve};

/// Limites θ_m para os quais o aproximante de grau m tem erro regressivo ≤ 2⁻⁵³ (Higham, 2005,
/// Tabela 2.3; os mesmos em Al-Mohy & Higham, 2009).
#[allow(clippy::excessive_precision)] // valores transcritos exatamente como publicados
const THETA: [(usize, f64); 5] = [
    (3, 1.495_585_217_958_292e-2),
    (5, 2.539_398_330_063_230e-1),
    (7, 9.504_178_996_162_932e-1),
    (9, 2.097_847_961_257_068e0),
    (13, 5.371_920_351_148_152e0),
];
const THETA13: f64 = THETA[4].1;

/// Arredondamento unitário do f64, u = 2⁻⁵³.
const UNIT_ROUNDOFF: f64 = 1.0 / 9_007_199_254_740_992.0;

/// Coeficientes do numerador do aproximante de Padé [m/m] da exponencial, normalizados com c₀ = 1:
/// c_k = (2m − k)!·m! / ((2m)!·k!·(m − k)!).
fn pade_coefficients(m: usize) -> Vec<f64> {
    let mut c = vec![1.0];
    for k in 1..=m {
        let prev = c[k - 1];
        c.push(prev * (m - k + 1) as f64 / ((2 * m - k + 1) as f64 * k as f64));
    }
    c
}

/// Módulo do coeficiente principal do erro do Padé [m/m]: eˣ − r_m(x) ≈ ±c·x^{2m+1}, com
/// c = (m!)² / ((2m)!·(2m+1)!).
fn error_coefficient(m: usize) -> f64 {
    // (m!)²/(2m)! = Π_{k=1..m} k/(m+k)
    let mut c: f64 = (1..=m).map(|k| k as f64 / (m + k) as f64).product();
    for k in 1..=2 * m + 1 {
        c /= k as f64;
    }
    c
}

/// Correção ℓ(A, m) = max(⌈log₂(α/u) / (2m)⌉, 0), α = c·‖|A|^{2m+1}‖₁ / ‖A‖₁ (Al-Mohy & Higham,
/// 2009, eq. 5.1). ‖|A|ᵖ‖₁ é calculada exatamente: para B ≥ 0, ‖B‖₁ = max(1ᵀ·B). `None` se α
/// não for finito (estouro).
fn ell<T: LinScalar>(a: &NdArray<T>, m: usize) -> Option<u32> {
    let n = a.rows();
    // vᵀ ← vᵀ·|A|, isto é, v ← |A|ᵀ·v, pelo produto do backend.
    let abs_t = faer::Mat::<f64>::from_fn(n, n, |i, j| a.get(j, i).abs());
    let mut v = faer::Mat::<f64>::from_fn(n, 1, |_, _| 1.0);
    let mut w = faer::Mat::<f64>::zeros(n, 1);
    for _ in 0..2 * m + 1 {
        matmul(&mut w, Accum::Replace, &abs_t, &v, 1.0, Par::Seq);
        std::mem::swap(&mut v, &mut w);
    }
    // ∞·0 = NaN quando |A|ᵖ estoura; f64::max ignoraria o NaN.
    let abs_pow = (0..n).map(|i| finite_or_inf(v[(i, 0)])).fold(0.0, f64::max);
    let alpha = error_coefficient(m) * abs_pow / norm1(a);
    if !alpha.is_finite() {
        return None;
    }
    // α = 0 (subestouro) dá −∞, que vira 0.
    Some(((alpha / UNIT_ROUNDOFF).log2() / (2 * m) as f64).ceil().max(0.0) as u32)
}

/// Estimativa de ‖F₁·F₂·…·F_k‖₁ sem formar o produto (estimador de Hager/Higham).
fn product_norm1<T: LinScalar>(factors: &[&NdArray<T>]) -> f64 {
    let n = factors[0].rows();
    // Produtos matriz-vetor do backend, sequenciais: o estimador faz dezenas deles, e a
    // paralelização de cada um custaria mais que o próprio produto.
    let mul = |f: faer::MatRef<'_, T>, x: &mut [T]| {
        let mut y = faer::Mat::<T>::zeros(n, 1);
        let xm = faer::MatRef::from_column_major_slice(x, n, 1);
        matmul(&mut y, Accum::Replace, f, xm, T::from_f64(1.0), Par::Seq);
        x.iter_mut().enumerate().for_each(|(i, xi)| *xi = y[(i, 0)]);
    };
    let apply = |x: &mut [T]| factors.iter().rev().for_each(|f| mul(f.as_faer(), x));
    let apply_adj = |x: &mut [T]| {
        for f in factors {
            let mut y = faer::Mat::<T>::zeros(n, 1);
            let xm = faer::MatRef::from_column_major_slice(x, n, 1);
            matmul(
                &mut y,
                Accum::Replace,
                f.as_faer().adjoint(),
                xm,
                T::from_f64(1.0),
                Par::Seq,
            );
            x.iter_mut().enumerate().for_each(|(i, xi)| *xi = y[(i, 0)]);
        }
    };
    finite_or_inf(norm1_estimate(n, &apply, &apply_adj))
}

/// NaN (de ∞ − ∞ em potências que estouraram) vira +∞, para que max/min não o ignorem.
fn finite_or_inf(x: f64) -> f64 {
    if x.is_nan() { f64::INFINITY } else { x }
}

/// Escalonamento de Higham (2005): o menor s ≥ 0 com ‖2⁻ˢA‖₁ ≤ θ₁₃. Limite superior para o s
/// escolhido aqui (as normas de potências nunca excedem ‖A‖₁).
fn s_from_norm<T: LinScalar>(a: &NdArray<T>) -> u32 {
    let nrm = norm1(a);
    // Se a soma estoura, ‖A‖₁ ≤ n·max|aᵢⱼ| ainda é finito.
    let lg = if nrm.is_finite() {
        nrm.log2()
    } else {
        a.max_abs().log2() + (a.rows() as f64).log2()
    };
    (lg - THETA13.log2()).ceil().max(0.0) as u32
}

/// Escolha do grau m do Padé e do escalonamento s (s > 0 só com m = 13), com as potências pares
/// de A já calculadas (I, A², A⁴, …).
struct Plan<T: LinScalar> {
    m: usize,
    s: u32,
    pows: Vec<NdArray<T>>,
}

/// Algoritmo 5.1 de Al-Mohy & Higham (2009): o menor grau m ∈ {3, 5, 7, 9} com η ≤ θ_m e
/// ℓ(A, m) = 0; senão m = 13 com s = ⌈log₂(η/θ₁₃)⌉ + ℓ(2⁻ˢA, 13). As normas ‖Aᵏ‖₁ de potências
/// ainda não formadas são estimadas.
fn plan<T: LinScalar>(a: &NdArray<T>) -> Result<Plan<T>, LinalgError> {
    let mut pows = vec![NdArray::identity(a.rows()), a.matmul(a)?];
    let d6 = product_norm1(&[&pows[1], &pows[1], &pows[1]]).powf(1.0 / 6.0);
    let eta1 = product_norm1(&[&pows[1], &pows[1]]).powf(0.25).max(d6);
    if eta1 <= THETA[0].1 && ell(a, 3) == Some(0) {
        return Ok(Plan { m: 3, s: 0, pows });
    }
    pows.push(pows[1].matmul(&pows[1])?);
    let d4 = finite_or_inf(norm1(&pows[2])).powf(0.25);
    if d4.max(d6) <= THETA[1].1 && ell(a, 5) == Some(0) {
        return Ok(Plan { m: 5, s: 0, pows });
    }
    pows.push(pows[2].matmul(&pows[1])?);
    let d6 = finite_or_inf(norm1(&pows[3])).powf(1.0 / 6.0);
    let d8 = product_norm1(&[&pows[2], &pows[2]]).powf(0.125);
    let eta3 = d6.max(d8);
    for &(m, theta) in &THETA[2..4] {
        if eta3 <= theta && ell(a, m) == Some(0) {
            return Ok(Plan { m, s: 0, pows });
        }
    }
    let d10 = product_norm1(&[&pows[2], &pows[3]]).powf(0.1);
    let eta5 = eta3.min(d8.max(d10));
    // s pelas normas de potências mais a correção ℓ; nunca acima do escalonamento de 2005.
    let s_max = s_from_norm(a);
    let s = if eta5.is_finite() {
        let s0 = ((eta5 / THETA13).log2().ceil().max(0.0) as u32).min(s_max);
        match ell(&a.scale(2f64.powi(-(s0 as i32))), 13) {
            Some(l) => (s0 + l).min(s_max),
            None => s_max,
        }
    } else {
        s_max
    };
    Ok(Plan { m: 13, s, pows })
}

/// Exponencial de matriz (ver o módulo).
pub fn expm<T: LinScalar>(a: &NdArray<T>) -> Result<NdArray<T>, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "expm",
            shape: a.shape(),
        });
    }
    if !a.all_finite() {
        return Err(LinalgError::NonFinite { operation: "expm" });
    }
    let n = a.rows();
    if n == 0 {
        return Ok(NdArray::zeros(0, 0));
    }
    let tri = a.is_upper_triangular();
    let Plan { m, s, mut pows } = plan(a)?;
    if m < 13 {
        let x = pade(a, &mut pows, m)?;
        return square(x, a, 0, tri);
    }
    let sc = 2f64.powi(-(s as i32));
    let a_s = a.scale(sc);
    // Potências escalonadas: A²ᵏ·2^{−2ks}, reaproveitadas se finitas e se o fator 2^{−6s} não
    // subestoura; senão recalculadas a partir de 2⁻ˢA.
    let mut pows_s = if s <= 170 && pows.iter().all(|p| p.all_finite()) {
        pows.iter()
            .enumerate()
            .map(|(k, p)| {
                if k == 0 {
                    p.clone()
                } else {
                    p.scale(2f64.powi(-2 * k as i32 * s as i32))
                }
            })
            .collect()
    } else {
        let a2 = a_s.matmul(&a_s)?;
        let a4 = a2.matmul(&a2)?;
        let a6 = a4.matmul(&a2)?;
        vec![pows[0].clone(), a2, a4, a6]
    };
    let x = pade(&a_s, &mut pows_s, 13)?;
    square(x, a, s, tri)
}

/// Quadratura: X ← X² s vezes. Em matriz triangular superior, a diagonal e a superdiagonal de
/// X ≈ exp(2⁻ⁱA) são substituídas pelos valores exatos antes de cada quadratura e no fim
/// (Al-Mohy & Higham, 2009, §2).
fn square<T: LinScalar>(
    mut x: NdArray<T>,
    a: &NdArray<T>,
    s: u32,
    tri: bool,
) -> Result<NdArray<T>, LinalgError> {
    for i in (0..=s).rev() {
        if tri {
            fix_triangular(&mut x, a, 2f64.powi(-(i as i32)));
        }
        if i > 0 {
            x = x.matmul(&x)?;
        }
    }
    Ok(x)
}

/// sinh(z)/z, com o limite 1 em z = 0.
fn sinhc(z: Complex64) -> Complex64 {
    if z == Complex64::new(0.0, 0.0) {
        Complex64::new(1.0, 0.0)
    } else {
        z.sinh() / z
    }
}

fn from_c64<T: LinScalar>(z: Complex64) -> T {
    // Entrada real: a parte imaginária é 0 (ou NaN, se o valor estourou); fica a parte real.
    T::from_complex(z).unwrap_or_else(|| T::from_f64(z.re))
}

/// Diagonal exp(c·aⱼⱼ) e superdiagonal de exp(c·A) para A triangular superior. Para o bloco
/// [[λ₁, t], [0, λ₂]]: f₁₂ = t·(e^λ₂ − e^λ₁)/(λ₂ − λ₁) = t·e^{(λ₁+λ₂)/2}·sinhc((λ₂ − λ₁)/2),
/// forma sem cancelamento quando λ₁ ≈ λ₂.
fn fix_triangular<T: LinScalar>(x: &mut NdArray<T>, a: &NdArray<T>, c: f64) {
    let n = a.rows();
    let lam: Vec<Complex64> = (0..n).map(|j| a.get(j, j).to_complex().scale(c)).collect();
    for (j, l) in lam.iter().enumerate() {
        x.set(j, j, from_c64(l.exp()));
    }
    for j in 1..n {
        let (l1, l2) = (lam[j - 1], lam[j]);
        let t = a.get(j - 1, j).to_complex().scale(c);
        x.set(
            j - 1,
            j,
            from_c64(t * ((l1 + l2) / 2.0).exp() * sinhc((l2 - l1) / 2.0)),
        );
    }
}

/// Avalia o aproximante de Padé [m/m] de exp(A): resolve (V − U)·X = (V + U). `pows` traz as
/// potências pares I, A², A⁴, … já calculadas e é estendida se preciso.
fn pade<T: LinScalar>(
    a: &NdArray<T>,
    pows: &mut Vec<NdArray<T>>,
    m: usize,
) -> Result<NdArray<T>, LinalgError> {
    let n = a.rows();
    let c = pade_coefficients(m);
    let lin = |x: &[(f64, &NdArray<T>)]| -> Result<NdArray<T>, LinalgError> {
        let mut acc = NdArray::zeros(n, n);
        for &(k, mat) in x {
            acc = acc.add(&mat.scale(k))?;
        }
        Ok(acc)
    };
    let (u, v) = if m == 13 {
        let (id, a2, a4, a6) = (&pows[0], &pows[1], &pows[2], &pows[3]);
        let u_in = a6.matmul(&lin(&[(c[13], a6), (c[11], a4), (c[9], a2)])?)?;
        let u_in = u_in.add(&lin(&[(c[7], a6), (c[5], a4), (c[3], a2), (c[1], id)])?)?;
        let v = a6.matmul(&lin(&[(c[12], a6), (c[10], a4), (c[8], a2)])?)?;
        let v = v.add(&lin(&[(c[6], a6), (c[4], a4), (c[2], a2), (c[0], id)])?)?;
        (a.matmul(&u_in)?, v)
    } else {
        let half = m / 2;
        while pows.len() <= half {
            let next = pows[pows.len() - 1].matmul(&pows[1])?;
            pows.push(next);
        }
        let odd: Vec<(f64, &NdArray<T>)> = (0..=half).map(|k| (c[2 * k + 1], &pows[k])).collect();
        let even: Vec<(f64, &NdArray<T>)> = (0..=half).map(|k| (c[2 * k], &pows[k])).collect();
        (a.matmul(&lin(&odd)?)?, lin(&even)?)
    };
    Ok(solve(&v.sub(&u)?, &v.add(&u)?)?.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(c: f64) -> NdArray<f64> {
        NdArray::from_fn(3, 3, |i, j| if i == j { c } else { 0.0 })
    }

    #[test]
    fn coeficiente_do_erro_por_fatoriais() {
        // (m!)²/((2m)!·(2m+1)!), conferido com fatoriais exatos.
        assert_eq!(error_coefficient(3), 36.0 / (720.0 * 5040.0));
        let (f5, f10, f11) = (120.0, 3_628_800.0, 39_916_800.0);
        let exact = f5 * f5 / (f10 * f11);
        assert!((error_coefficient(5) - exact).abs() < 1e-14 * exact);
    }

    #[test]
    fn grau_troca_exatamente_nos_limiares() {
        // Em c·I, ‖Aᵏ‖^{1/k} = c: o grau m é o primeiro com c ≤ θ_m.
        let expected = [(3, 5), (5, 7), (7, 9), (9, 13)];
        for (&(m, th), &(below, above)) in THETA.iter().zip(&expected) {
            assert_eq!(m, below);
            let p = plan(&scalar(th * 0.999)).unwrap();
            assert_eq!((p.m, p.s), (below, 0), "abaixo de θ_{m}");
            let p = plan(&scalar(th * 1.001)).unwrap();
            assert_eq!((p.m, p.s), (above, 0), "acima de θ_{m}");
        }
        assert_eq!(plan(&scalar(THETA13 * 0.999)).unwrap().s, 0);
        assert_eq!(plan(&scalar(THETA13 * 1.001)).unwrap().s, 1);
        // s = ⌈log₂(c/θ₁₃)⌉: 100/θ₁₃ ≈ 18,6 → 5; 1e4/θ₁₃ ≈ 1862 → 11.
        assert_eq!(plan(&scalar(100.0)).unwrap().s, 5);
        assert_eq!(plan(&scalar(1e4)).unwrap().s, 11);
    }

    #[test]
    fn correcao_ell() {
        // c·I: α = c_m·c^{2m}; em c = θ₃, α ≈ u (θ₃ é justamente onde o erro chega a u).
        let th3 = THETA[0].1;
        assert_eq!(ell(&scalar(th3 * 0.999), 3), Some(0));
        assert_eq!(ell(&scalar(th3 * 1.001), 3), Some(1));
        // α = c₃·c⁶ = 2^{6k}·u ⇒ ℓ = k (c escolhido para dar α exatamente nessa forma).
        for k in 1..4 {
            let c = (2f64.powi(6 * k) * UNIT_ROUNDOFF / error_coefficient(3)).powf(1.0 / 6.0);
            assert_eq!(ell(&scalar(c * 1.0001), 3), Some(k as u32 + 1), "k={k}");
            assert_eq!(ell(&scalar(c * 0.9999), 3), Some(k as u32), "k={k}");
        }
        // Estouro de |A|^{27}: sem valor.
        assert_eq!(ell(&scalar(1e20), 13), None);
    }

    #[test]
    fn nao_superescalona_matriz_nao_normal() {
        // [[1, b], [0, −1]]: ‖A‖₁ = b + 1, mas A² = I. O algoritmo de 2005 usaria s ≈ log₂(b/θ₁₃).
        for b in [1e4, 1e8, 1e12] {
            for a in [
                NdArray::from_rows(&[&[1.0, b], &[0.0, -1.0]]).unwrap(),
                NdArray::from_rows(&[&[-1.0, 0.0], &[b, 1.0]]).unwrap(),
            ] {
                let p = plan(&a).unwrap();
                assert_eq!((p.m, p.s), (9, 0), "b={b}");
                assert!(s_from_norm(&a) >= 10);
            }
        }
    }

    #[test]
    fn escalonamento_limitado_pelo_de_2005() {
        // Nilpotente de posto 1: η = 0, mas ℓ(A, 13) é enorme (|A| não é nilpotente); s fica no
        // valor de 2005. Com b = 1e15, |A|^{27} estoura e ℓ não tem valor: também s de 2005.
        for b in [1e6, 1e15] {
            let a = NdArray::from_rows(&[&[b, -b], &[b, -b]]).unwrap();
            let p = plan(&a).unwrap();
            assert_eq!((p.m, p.s), (13, s_from_norm(&a)), "b={b}");
        }
        // Norma que estoura: ‖A‖₁ ≤ n·max|aᵢⱼ| = 2^{1024}·… dá s finito.
        let big = NdArray::from_rows(&[&[1e308, 1e308], &[1e308, 1e308]]).unwrap();
        let expected = ((1e308f64).log2() + 1.0 - THETA13.log2()).ceil() as u32;
        assert_eq!(s_from_norm(&big), expected);
        assert_eq!(plan(&big).unwrap().s, expected);
    }

    #[test]
    fn primeira_etapa_usa_a_sexta_potencia() {
        // Ciclo 4×4 com pesos (a, a, b, b): A⁴ = (ab)²·I e A⁶ = (ab)²·A², logo
        // d₄ = √(ab) e d₆ = √(ab)·(a/b)^{1/6} > d₄. Com d₄ < θ₃ < d₆, o grau 3 é recusado
        // só por causa de d₆.
        let th3 = THETA[0].1;
        let g = 0.99 * th3;
        let (a, b) = (2.0 * g, g / 2.0);
        let w = [a, a, b, b];
        let m = NdArray::<f64>::from_fn(4, 4, |i, j| if j == (i + 1) % 4 { w[i] } else { 0.0 });
        let p = plan(&m).unwrap();
        assert_eq!((p.m, p.s), (5, 0));
        let d4 = product_norm1(&[&p.pows[1], &p.pows[1]]).powf(0.25);
        assert!((d4 - g).abs() < 1e-15 && ell(&m, 3) == Some(0));
    }

    #[test]
    fn escalonamento_pelas_normas_de_potencias() {
        // [[c, b], [0, −c]]: A² = c²·I, η = c = 10 ⇒ s₀ = ⌈log₂(10/θ₁₃)⌉ = 1; ℓ(2⁻¹A, 13) = 1.
        // O critério de 2005 daria s = 11 e s = 25.
        for b in [1e4, 1e8] {
            let a = NdArray::<f64>::from_rows(&[&[10.0, b], &[0.0, -10.0]]).unwrap();
            let p = plan(&a).unwrap();
            assert_eq!((p.m, p.s), (13, 2), "b={b}");
            assert_eq!(ell(&a.scale(0.5), 13), Some(1));
        }
    }

    #[test]
    fn potencias_que_estouram_com_s_pequeno_sao_recalculadas() {
        // c·[[0, −1], [1, 0]] com c⁶ > máx. do f64 e s = 169 ≤ 170: A⁶ estoura e precisa ser
        // recalculada a partir de 2⁻ˢA (reaproveitá-la daria NaN).
        let c = 2.4e51 * 1.01;
        let a = NdArray::<f64>::from_rows(&[&[0.0, -c], &[c, 0.0]]).unwrap();
        let p = plan(&a).unwrap();
        assert_eq!(p.s, 169);
        assert!(!p.pows[3].all_finite());
        assert!(expm(&a).unwrap().all_finite());
    }

    #[test]
    fn sinhc_e_limite() {
        assert_eq!(sinhc(Complex64::new(0.0, 0.0)), Complex64::new(1.0, 0.0));
        let z = Complex64::new(1.0, 0.0);
        assert!((sinhc(z).re - 1f64.sinh()).abs() < 1e-16);
    }
}
