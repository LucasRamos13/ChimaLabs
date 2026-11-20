//! Balanceamento de matrizes e autovalores com opções.
//!
//! Duas etapas (Parlett & Reinsch, "Balancing a matrix for calculation of eigenvalues and
//! eigenvectors", Numer. Math. 13, 1969):
//! 1. **Permutação:** linhas cujos elementos fora da diagonal, dentro do bloco ativo, são todos
//!    nulos vão para o fim, e colunas assim vão para o início, por permutações simétricas. Cada
//!    uma isola um autovalor exato (o elemento diagonal), e B = PᵀAP fica com a forma
//!    [T₁ X Y; 0 B₂₂ Z; 0 0 T₃], com T₁ e T₃ triangulares superiores e o bloco ativo B₂₂ nas
//!    posições `ilo..ihi`.
//! 2. **Escalonamento** diagonal do bloco ativo com potências de 2, com o critério em norma 2 de
//!    James, Langou & Lowery ("On matrix balancing and eigenvector computation", arXiv:1401.5766,
//!    2014). Escalas potências de 2 não introduzem erro.
//!
//! B = D⁻¹PᵀAPD tem os mesmos autovalores de A e, em geral, norma menor, o que reduz o erro de
//! arredondamento do QR.

use num_complex::Complex64;

use super::decomp::{Eig, eig_unbalanced};
use super::{ComplexMatrix, LinScalar, LinalgError, Matrix, NdArray, Scalar};

/// Resultado de [`balance`]: `b = D⁻¹·Pᵀ·a·P·D`, com `D = diag(scale)` e P a permutação
/// `perm` (a posição i de `b` corresponde à linha e coluna `perm[i]` de `a`), isto é,
/// `b[i][k] = a[perm[i]][perm[k]]·scale[k]/scale[i]`.
#[derive(Clone, Debug)]
pub struct Balance<T: LinScalar> {
    /// Fatores de escala (potências de 2), por posição em `b`; iguais a 1 fora de `ilo..ihi`.
    pub scale: Vec<f64>,
    /// Permutação: `perm[i]` é o índice original da posição i.
    pub perm: Vec<usize>,
    /// Início do bloco ativo (as posições `0..ilo` têm autovalores isolados).
    pub ilo: usize,
    /// Fim (exclusivo) do bloco ativo (as posições `ihi..n` têm autovalores isolados).
    pub ihi: usize,
    /// Matriz balanceada.
    pub b: NdArray<T>,
}

impl<T: LinScalar> Balance<T> {
    /// Matriz de transformação T = P·D, com `a·T = T·b` exatamente (T tem um só elemento não nulo,
    /// potência de 2, por linha e por coluna).
    pub fn transform(&self) -> Matrix {
        let n = self.scale.len();
        let mut t = Matrix::zeros(n, n);
        for (i, (&p, &s)) in self.perm.iter().zip(&self.scale).enumerate() {
            t.set(p, i, s);
        }
        t
    }
}

/// Opções de [`balance_with`]. O padrão faz as duas etapas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BalanceOptions {
    /// Isolar autovalores por permutação.
    pub permute: bool,
    /// Escalonar o bloco ativo.
    pub scale: bool,
}

impl Default for BalanceOptions {
    fn default() -> Self {
        Self {
            permute: true,
            scale: true,
        }
    }
}

/// Limites dos fatores de escala acumulados, para não haver estouro nem subestouro.
const SCALE_MAX: f64 = 1.0e150;
const SCALE_MIN: f64 = 1.0e-150;
/// Passos de 2× suficientes para cobrir toda a faixa do f64 (razões até 2^2100).
const MAX_STEPS: usize = 1100;

/// Balanceia a matriz quadrada `a` com permutação e escalonamento (ver o módulo). Matrizes
/// hermitianas sem autovalores isolados já estão balanceadas em norma 2, e o resultado é `D = I`.
pub fn balance<T: LinScalar>(a: &NdArray<T>) -> Result<Balance<T>, LinalgError> {
    balance_with(a, BalanceOptions::default())
}

/// [`balance`] com as etapas escolhidas.
pub fn balance_with<T: LinScalar>(a: &NdArray<T>, opts: BalanceOptions) -> Result<Balance<T>, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "balance",
            shape: a.shape(),
        });
    }
    if !a.all_finite() {
        return Err(LinalgError::NonFinite { operation: "balance" });
    }
    let n = a.rows();
    let mut b = a.clone();
    let mut perm: Vec<usize> = (0..n).collect();
    let data = b.as_mut_slice(); // ordem de colunas: (i, k) em i + k·n
    let (ilo, ihi) = if opts.permute {
        permute(data, n, &mut perm)
    } else {
        (0, n)
    };
    let mut scale = vec![1.0; n];
    if opts.scale {
        scale_block(data, n, ilo, ihi, &mut scale);
    }
    Ok(Balance {
        scale,
        perm,
        ilo,
        ihi,
        b,
    })
}

/// Troca simétrica das posições i e j (linhas e colunas).
fn swap<T: Copy>(data: &mut [T], n: usize, i: usize, j: usize) {
    if i == j {
        return;
    }
    for k in 0..n {
        data.swap(i + k * n, j + k * n);
    }
    for k in 0..n {
        data.swap(k + i * n, k + j * n);
    }
}

/// Etapa de permutação. Linhas isoladas vão para o fim (busca de baixo para cima, que acha a
/// próxima logo em matrizes triangulares) e colunas isoladas para o início. Devolve (ilo, ihi).
fn permute<T: LinScalar>(data: &mut [T], n: usize, perm: &mut [usize]) -> (usize, usize) {
    let zero = T::zero();
    let (mut lo, mut hi) = (0, n);
    // Linha j isolada: a[j][k] = 0 para k ∈ lo..hi, k ≠ j.
    while let Some(j) = (lo..hi)
        .rev()
        .find(|&j| (lo..hi).all(|k| k == j || data[j + k * n] == zero))
    {
        swap(data, n, j, hi - 1);
        perm.swap(j, hi - 1);
        hi -= 1;
    }
    // Coluna j isolada: a[k][j] = 0 para k ∈ lo..hi, k ≠ j.
    while let Some(j) = (lo..hi).find(|&j| (lo..hi).all(|k| k == j || data[k + j * n] == zero)) {
        swap(data, n, j, lo);
        perm.swap(j, lo);
        lo += 1;
    }
    (lo, hi)
}

/// Escalonamento do bloco ativo `lo..hi`: as normas usam só os elementos do bloco, e a escala
/// se aplica à coluna e à linha inteiras (fora do bloco elas têm zeros nas posições que a
/// estrutura triangular exige, que continuam zero).
fn scale_block<T: LinScalar>(data: &mut [T], n: usize, lo: usize, hi: usize, scale: &mut [f64]) {
    // Cada passada que altera alguma escala reduz Σ(c + r) em ≥ 5%: termina; o limite é só guarda.
    for _ in 0..1000 {
        let mut changed = false;
        for i in lo..hi {
            let mut c2 = 0.0;
            let mut r2 = 0.0;
            for k in (lo..hi).filter(|&k| k != i) {
                c2 += data[k + i * n].abs().powi(2);
                r2 += data[i + k * n].abs().powi(2);
            }
            let (c, r) = (c2.sqrt(), r2.sqrt());
            if c == 0.0 || r == 0.0 {
                continue;
            }
            // Coluna i multiplicada por f = 2^k e linha i dividida por f: normas c·f e r/f. Escolhe k
            // com c·f ∈ [r/(2f), 2r/f). Cada passo muda a razão por 4, então k para no primeiro
            // ponto do intervalo; o limite de passos só protege de estouro e subestouro.
            let mut f = 1.0;
            for _ in 0..MAX_STEPS {
                if c * f < r / f / 2.0 {
                    f *= 2.0;
                } else if c * f >= 2.0 * r / f {
                    f /= 2.0;
                } else {
                    break;
                }
            }
            let (cf, rf) = (c * f, r / f);
            if cf + rf >= 0.95 * (c + r) {
                continue;
            }
            let s = scale[i] * f;
            if !(SCALE_MIN..=SCALE_MAX).contains(&s) {
                continue;
            }
            scale[i] = s;
            changed = true;
            for k in 0..n {
                data[k + i * n] = data[k + i * n].scale(f);
                data[i + k * n] = data[i + k * n].scale(1.0 / f);
            }
        }
        if !changed {
            break;
        }
    }
}

/// Ordem dos autovalores devolvidos por [`eig_with`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EigOrder {
    /// Sem garantia de ordem (a do algoritmo; crescente no caso hermitiano).
    #[default]
    Unspecified,
    /// Crescente pela parte real e, em empate, pela imaginária.
    Ascending,
    /// Decrescente pela parte real e, em empate, pela imaginária.
    Descending,
    /// Decrescente pelo módulo; em empate, pela parte real e depois pela imaginária (crescentes).
    DescendingMagnitude,
}

/// Opções de [`eig_with`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EigOptions {
    /// Balancear antes do QR (padrão: sim). Desligar pode dar autovetores mais precisos quando a
    /// matriz já tem elementos de magnitudes comparáveis com erros de arredondamento relevantes
    /// (James, Langou & Lowery, 2014). Sem efeito em matrizes hermitianas.
    pub balance: bool,
    /// Ordem dos autovalores (e das colunas dos autovetores).
    pub order: EigOrder,
}

impl Default for EigOptions {
    fn default() -> Self {
        Self {
            balance: true,
            order: EigOrder::Unspecified,
        }
    }
}

/// Autovalores e autovetores com opções de balanceamento e ordenação. [`eig`](super::eig)
/// equivale a `eig_with(a, EigOptions::default())`.
pub fn eig_with<T: LinScalar>(a: &NdArray<T>, opts: EigOptions) -> Result<Eig<T>, LinalgError> {
    let mut e = if opts.balance && a.is_square() && a.all_finite() && !a.is_hermitian() {
        let bal = balance(a)?;
        match eig_unbalanced(&bal.b)? {
            Eig::General { values, vectors } => {
                // A·(P·D·W) = P·D·B·W = (P·D·W)·Λ; renormaliza as colunas (norma 2 unitária).
                let n = a.rows();
                let mut v = ComplexMatrix::zeros(n, n);
                for j in 0..n {
                    for i in 0..n {
                        v.set(bal.perm[i], j, vectors.get(i, j).scale(bal.scale[i]));
                    }
                }
                for j in 0..n {
                    // Nunca nula: W tem colunas de norma 1 e as escalas estão em [1e-150, 1e150].
                    let nrm = (0..n).map(|i| v.get(i, j).norm_sqr()).sum::<f64>().sqrt();
                    for i in 0..n {
                        v.set(i, j, v.get(i, j).scale(1.0 / nrm));
                    }
                }
                Eig::General { values, vectors: v }
            }
            sym => sym,
        }
    } else {
        eig_unbalanced(a)?
    };
    sort_eig(&mut e, opts.order);
    Ok(e)
}

fn sort_eig<T: LinScalar>(e: &mut Eig<T>, order: EigOrder) {
    let key = |z: Complex64, w: Complex64| -> std::cmp::Ordering {
        match order {
            EigOrder::Unspecified => std::cmp::Ordering::Equal,
            EigOrder::Ascending => z.re.total_cmp(&w.re).then(z.im.total_cmp(&w.im)),
            EigOrder::Descending => w.re.total_cmp(&z.re).then(w.im.total_cmp(&z.im)),
            EigOrder::DescendingMagnitude => w
                .norm()
                .total_cmp(&z.norm())
                .then(z.re.total_cmp(&w.re))
                .then(z.im.total_cmp(&w.im)),
        }
    };
    if order == EigOrder::Unspecified {
        return;
    }
    match e {
        Eig::Symmetric { values, vectors } => {
            let mut idx: Vec<usize> = (0..values.len()).collect();
            idx.sort_by(|&i, &j| key(Complex64::new(values[i], 0.0), Complex64::new(values[j], 0.0)));
            *values = idx.iter().map(|&i| values[i]).collect();
            *vectors = NdArray::from_fn(vectors.rows(), idx.len(), |r, c| vectors.get(r, idx[c]));
        }
        Eig::General { values, vectors } => {
            let mut idx: Vec<usize> = (0..values.len()).collect();
            idx.sort_by(|&i, &j| key(values[i], values[j]));
            *values = idx.iter().map(|&i| values[i]).collect();
            *vectors = NdArray::from_fn(vectors.rows(), idx.len(), |r, c| vectors.get(r, idx[c]));
        }
    }
}
