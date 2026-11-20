//! O tipo [`NdArray`].

use std::sync::Arc;

use num_complex::Complex64;

use crate::{ArrayError, Element, Scalar};

/// Normaliza uma forma: no mínimo 2 dimensões; remove singletons finais além da segunda.
pub fn normalize_shape(shape: &[usize]) -> Vec<usize> {
    let mut s = shape.to_vec();
    while s.len() < 2 {
        s.push(1);
    }
    while s.len() > 2 && s.last() == Some(&1) {
        s.pop();
    }
    s
}

/// Forma resultante da expansão implícita: em cada dimensão os tamanhos devem ser iguais ou um
/// deles igual a 1 (que é repetido).
pub fn broadcast_shape(a: &[usize], b: &[usize]) -> Result<Vec<usize>, ArrayError> {
    let n = a.len().max(b.len());
    let mut out = Vec::with_capacity(n);
    for d in 0..n {
        let x = a.get(d).copied().unwrap_or(1);
        let y = b.get(d).copied().unwrap_or(1);
        out.push(match (x, y) {
            _ if x == y => x,
            (1, _) => y,
            (_, 1) => x,
            _ => {
                return Err(ArrayError::IncompatibleShapes {
                    operation: "expansão implícita",
                    left: a.to_vec(),
                    right: b.to_vec(),
                });
            }
        });
    }
    Ok(normalize_shape(&out))
}

/// Array N-D column-major com dados compartilhados e cópia sob demanda.
#[derive(Clone, Debug)]
pub struct NdArray<T: Element> {
    dims: Vec<usize>,
    data: Arc<Vec<T>>,
}

impl<T: Element> PartialEq for NdArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.dims == other.dims && (Arc::ptr_eq(&self.data, &other.data) || self.data == other.data)
    }
}

// ------------------------------------------------------------------ N-D

impl<T: Element> NdArray<T> {
    /// Cria a partir de forma e dados column-major.
    pub fn from_shape_vec(shape: &[usize], data: Vec<T>) -> Result<Self, ArrayError> {
        let expected: usize = shape.iter().product();
        if expected != data.len() {
            return Err(ArrayError::ShapeMismatch {
                expected,
                actual: data.len(),
            });
        }
        Ok(Self {
            dims: normalize_shape(shape),
            data: Arc::new(data),
        })
    }

    /// Array com todos os elementos iguais a `value`.
    pub fn filled_nd(shape: &[usize], value: T) -> Self {
        let n = shape.iter().product();
        Self {
            dims: normalize_shape(shape),
            data: Arc::new(vec![value; n]),
        }
    }

    /// Cria avaliando `f(subscritos)` para cada elemento, em ordem column-major.
    pub fn from_fn_nd(shape: &[usize], mut f: impl FnMut(&[usize]) -> T) -> Self {
        let dims = normalize_shape(shape);
        let n: usize = dims.iter().product();
        let mut data = Vec::with_capacity(n);
        let mut sub = vec![0usize; dims.len()];
        for _ in 0..n {
            data.push(f(&sub));
            for (d, s) in sub.iter_mut().enumerate() {
                *s += 1;
                if *s < dims[d] {
                    break;
                }
                *s = 0;
            }
        }
        Self {
            dims,
            data: Arc::new(data),
        }
    }

    /// Forma (≥ 2 dimensões).
    pub fn dims(&self) -> &[usize] {
        &self.dims
    }

    /// Número de dimensões (≥ 2).
    pub fn ndims(&self) -> usize {
        self.dims.len()
    }

    /// Tamanho da dimensão `d` (1 além das existentes).
    pub fn dim(&self, d: usize) -> usize {
        self.dims.get(d).copied().unwrap_or(1)
    }

    /// Número de elementos.
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Verdadeiro se não há elementos.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Dados em ordem column-major.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Dados mutáveis (copia se estiverem compartilhados).
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        Arc::make_mut(&mut self.data).as_mut_slice()
    }

    /// Consome o array e devolve os dados (sem cópia se não compartilhados).
    pub fn into_vec(self) -> Vec<T> {
        Arc::try_unwrap(self.data).unwrap_or_else(|a| (*a).clone())
    }

    /// Verdadeiro se este array compartilha dados com outro (útil em testes de cópia sob demanda).
    pub fn shares_data_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    /// Índice linear de subscritos (subscritos ausentes valem 0; extras devem ser 0).
    pub fn linear_index(&self, sub: &[usize]) -> Result<usize, ArrayError> {
        let mut idx = 0;
        let mut stride = 1;
        for d in 0..self.dims.len().max(sub.len()) {
            let s = sub.get(d).copied().unwrap_or(0);
            let n = self.dim(d);
            if s >= n {
                return Err(ArrayError::IndexOutOfBounds {
                    index: sub.to_vec(),
                    shape: self.dims.clone(),
                });
            }
            idx += s * stride;
            stride *= n;
        }
        Ok(idx)
    }

    /// Subscritos a partir do índice linear.
    pub fn subscripts(&self, mut linear: usize) -> Vec<usize> {
        self.dims
            .iter()
            .map(|&n| {
                let s = linear % n.max(1);
                linear /= n.max(1);
                s
            })
            .collect()
    }

    /// Elemento por subscritos N-D.
    pub fn get_nd(&self, sub: &[usize]) -> Result<&T, ArrayError> {
        Ok(&self.data[self.linear_index(sub)?])
    }

    /// Define elemento por subscritos N-D.
    pub fn set_nd(&mut self, sub: &[usize], value: T) -> Result<(), ArrayError> {
        let i = self.linear_index(sub)?;
        self.as_mut_slice()[i] = value;
        Ok(())
    }

    /// Muda a forma mantendo a ordem column-major dos elementos.
    pub fn reshape(&self, shape: &[usize]) -> Result<Self, ArrayError> {
        let expected: usize = shape.iter().product();
        if expected != self.numel() {
            return Err(ArrayError::ShapeMismatch {
                expected,
                actual: self.numel(),
            });
        }
        Ok(Self {
            dims: normalize_shape(shape),
            data: Arc::clone(&self.data),
        })
    }

    /// Permuta dimensões: a dimensão `d` da saída é a `order[d]` da entrada.
    pub fn permute(&self, order: &[usize]) -> Result<Self, ArrayError> {
        let n = order.len();
        let mut seen = vec![false; n];
        if n < self.ndims()
            || order
                .iter()
                .any(|&o| o >= n || std::mem::replace(&mut seen[o], true))
        {
            return Err(ArrayError::InvalidPermutation(order.to_vec()));
        }
        let in_dims: Vec<usize> = (0..n).map(|d| self.dim(d)).collect();
        let out_dims: Vec<usize> = order.iter().map(|&o| in_dims[o]).collect();
        let mut in_strides = vec![1usize; n];
        for d in 1..n {
            in_strides[d] = in_strides[d - 1] * in_dims[d - 1];
        }
        let out = Self::from_fn_nd(&out_dims, |sub| {
            let li: usize = sub.iter().zip(order).map(|(&s, &o)| s * in_strides[o]).sum();
            self.data[li].clone()
        });
        Ok(out)
    }

    /// Remove dimensões singleton (resultado mantém pelo menos 2 dimensões).
    pub fn squeeze(&self) -> Self {
        let d: Vec<usize> = self.dims.iter().copied().filter(|&n| n != 1).collect();
        let d = if self.dims.len() == 2 {
            self.dims.clone()
        } else {
            d
        };
        Self {
            dims: normalize_shape(&d),
            data: Arc::clone(&self.data),
        }
    }

    /// Aplica `f` a cada elemento.
    pub fn map<U: Element>(&self, f: impl Fn(&T) -> U) -> NdArray<U> {
        NdArray {
            dims: self.dims.clone(),
            data: Arc::new(self.data.iter().map(f).collect()),
        }
    }

    /// Combina elemento a elemento com expansão implícita de dimensões singleton.
    pub fn zip_with<U: Element, V: Element>(
        &self,
        other: &NdArray<U>,
        f: impl Fn(&T, &U) -> V,
    ) -> Result<NdArray<V>, ArrayError> {
        if self.dims == other.dims {
            return Ok(NdArray {
                dims: self.dims.clone(),
                data: Arc::new(
                    self.data
                        .iter()
                        .zip(other.data.iter())
                        .map(|(a, b)| f(a, b))
                        .collect(),
                ),
            });
        }
        let out = broadcast_shape(&self.dims, &other.dims)?;
        let a_lin = |sub: &[usize]| -> usize { broadcast_index(sub, &self.dims) };
        let b_lin = |sub: &[usize]| -> usize { broadcast_index(sub, &other.dims) };
        Ok(NdArray::from_fn_nd(&out, |sub| {
            f(&self.data[a_lin(sub)], &other.data[b_lin(sub)])
        }))
    }

    /// Concatena arrays ao longo de `dim`; as demais dimensões devem coincidir.
    pub fn concat(arrays: &[&Self], dim: usize) -> Result<Self, ArrayError> {
        let Some(first) = arrays.first() else {
            return Ok(Self {
                dims: vec![0, 0],
                data: Arc::new(vec![]),
            });
        };
        let nd = arrays.iter().map(|a| a.ndims()).max().unwrap_or(2).max(dim + 1);
        for a in arrays {
            for d in (0..nd).filter(|&d| d != dim) {
                if a.dim(d) != first.dim(d) {
                    return Err(ArrayError::IncompatibleShapes {
                        operation: "concatenação",
                        left: first.dims.clone(),
                        right: a.dims.clone(),
                    });
                }
            }
        }
        let mut out_dims: Vec<usize> = (0..nd).map(|d| first.dim(d)).collect();
        out_dims[dim] = arrays.iter().map(|a| a.dim(dim)).sum();
        let offsets: Vec<usize> = arrays
            .iter()
            .scan(0, |acc, a| {
                let o = *acc;
                *acc += a.dim(dim);
                Some(o)
            })
            .collect();
        Ok(Self::from_fn_nd(&out_dims, |sub| {
            let k = offsets
                .iter()
                .rposition(|&o| o <= sub[dim])
                .expect("deslocamento");
            let mut s = sub.to_vec();
            s[dim] -= offsets[k];
            arrays[k].get_nd(&s).expect("dentro dos limites").clone()
        }))
    }
}

fn broadcast_index(sub: &[usize], dims: &[usize]) -> usize {
    let mut idx = 0;
    let mut stride = 1;
    for (d, &n) in dims.iter().enumerate() {
        let s = if n == 1 {
            0
        } else {
            sub.get(d).copied().unwrap_or(0)
        };
        idx += s * stride;
        stride *= n;
    }
    idx
}

// ------------------------------------------------------------------ 2-D

impl<T: Element> NdArray<T> {
    /// Cria matriz `rows × cols` avaliando `f(i, j)`.
    pub fn from_fn(rows: usize, cols: usize, mut f: impl FnMut(usize, usize) -> T) -> Self {
        let mut data = Vec::with_capacity(rows * cols);
        for j in 0..cols {
            for i in 0..rows {
                data.push(f(i, j));
            }
        }
        Self {
            dims: vec![rows, cols],
            data: Arc::new(data),
        }
    }

    /// Cria matriz a partir de dados column-major.
    pub fn from_col_major(rows: usize, cols: usize, data: Vec<T>) -> Result<Self, ArrayError> {
        Self::from_shape_vec(&[rows, cols], data)
    }

    /// Cria matriz a partir de linhas de mesmo comprimento.
    pub fn from_rows(rows: &[&[T]]) -> Result<Self, ArrayError> {
        let m = rows.len();
        let n = rows.first().map_or(0, |r| r.len());
        if let Some(bad) = rows.iter().find(|r| r.len() != n) {
            return Err(ArrayError::IncompatibleShapes {
                operation: "from_rows",
                left: vec![1, n],
                right: vec![1, bad.len()],
            });
        }
        Ok(Self::from_fn(m, n, |i, j| rows[i][j].clone()))
    }

    /// Vetor coluna.
    pub fn column(values: &[T]) -> Self {
        Self {
            dims: vec![values.len(), 1],
            data: Arc::new(values.to_vec()),
        }
    }

    /// Número de linhas (tamanho da dimensão 0).
    pub fn rows(&self) -> usize {
        self.dims[0]
    }

    /// Número de colunas; para N-D, o produto das dimensões a partir da segunda
    /// (visão 2-D column-major).
    pub fn cols(&self) -> usize {
        self.dims[1..].iter().product()
    }

    /// `(linhas, colunas)` da visão 2-D.
    pub fn shape(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }

    /// Verdadeiro se 2-D e quadrada.
    pub fn is_square(&self) -> bool {
        self.ndims() == 2 && self.dims[0] == self.dims[1]
    }

    /// Elemento `(i, j)` da visão 2-D.
    pub fn get(&self, i: usize, j: usize) -> T {
        let r = self.rows();
        assert!(
            i < r && j < self.cols(),
            "índice ({i}, {j}) fora de {:?}",
            self.dims
        );
        self.data[i + j * r].clone()
    }

    /// Define o elemento `(i, j)` da visão 2-D.
    pub fn set(&mut self, i: usize, j: usize, v: T) {
        let r = self.rows();
        assert!(
            i < r && j < self.cols(),
            "índice ({i}, {j}) fora de {:?}",
            self.dims
        );
        self.as_mut_slice()[i + j * r] = v;
    }

    /// Transposta da visão 2-D.
    pub fn transpose(&self) -> Self {
        Self::from_fn(self.cols(), self.rows(), |i, j| self.get(j, i))
    }

    /// Submatriz com linhas `r0..r1` e colunas `c0..c1`.
    pub fn submatrix(&self, r0: usize, r1: usize, c0: usize, c1: usize) -> Self {
        Self::from_fn(r1 - r0, c1 - c0, |i, j| self.get(r0 + i, c0 + j))
    }
}

// ------------------------------------------------------------------ numérico

impl<T: Scalar> NdArray<T> {
    /// Array N-D de zeros.
    pub fn zeros_nd(shape: &[usize]) -> Self {
        Self::filled_nd(shape, T::zero())
    }

    /// Matriz `rows × cols` de zeros.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self::filled_nd(&[rows, cols], T::zero())
    }

    /// Identidade `n × n`.
    pub fn identity(n: usize) -> Self {
        Self::from_fn(n, n, |i, j| if i == j { T::one() } else { T::zero() })
    }

    /// Soma elemento a elemento (formas iguais).
    pub fn add(&self, rhs: &Self) -> Result<Self, ArrayError> {
        self.same_shape(rhs, "add")?;
        self.zip_with(rhs, |a, b| *a + *b)
    }

    /// Subtração elemento a elemento (formas iguais).
    pub fn sub(&self, rhs: &Self) -> Result<Self, ArrayError> {
        self.same_shape(rhs, "sub")?;
        self.zip_with(rhs, |a, b| *a - *b)
    }

    fn same_shape(&self, rhs: &Self, operation: &'static str) -> Result<(), ArrayError> {
        if self.dims != rhs.dims {
            return Err(ArrayError::IncompatibleShapes {
                operation,
                left: self.dims.clone(),
                right: rhs.dims.clone(),
            });
        }
        Ok(())
    }

    /// Multiplica por um real.
    pub fn scale(&self, s: f64) -> Self {
        self.map(|x| x.scale(s))
    }

    /// Maior módulo (0 se vazio; NaN se houver NaN).
    pub fn max_abs(&self) -> f64 {
        let mut m: f64 = 0.0;
        for x in self.data.iter() {
            if x.is_nan() {
                return f64::NAN;
            }
            m = m.max(x.abs());
        }
        m
    }

    /// Verdadeiro se todos os elementos são finitos.
    pub fn all_finite(&self) -> bool {
        self.data.iter().all(|x| x.is_finite())
    }

    /// Verdadeiro se nada abaixo da diagonal principal (2-D).
    pub fn is_upper_triangular(&self) -> bool {
        self.ndims() == 2
            && self
                .columns()
                .enumerate()
                .all(|(j, col)| col.iter().skip(j + 1).all(|x| *x == T::zero()))
    }

    /// Verdadeiro se nada acima da diagonal principal (2-D).
    pub fn is_lower_triangular(&self) -> bool {
        self.ndims() == 2
            && self
                .columns()
                .enumerate()
                .all(|(j, col)| col.iter().take(j).all(|x| *x == T::zero()))
    }

    /// Verdadeiro se quadrada e exatamente simétrica (`A = Aᵀ`).
    pub fn is_symmetric(&self) -> bool {
        self.is_square()
            && (0..self.cols()).all(|j| (j + 1..self.rows()).all(|i| self.get(i, j) == self.get(j, i)))
    }

    /// Verdadeiro se quadrada e exatamente hermitiana (`A = Aᴴ`; para reais, simétrica).
    pub fn is_hermitian(&self) -> bool {
        self.is_square()
            && (0..self.cols()).all(|j| {
                self.get(j, j).im() == 0.0
                    && (j + 1..self.rows()).all(|i| self.get(i, j) == self.get(j, i).conj())
            })
    }

    /// Transposta conjugada (adjunta).
    pub fn adjoint(&self) -> Self {
        Self::from_fn(self.cols(), self.rows(), |i, j| self.get(j, i).conj())
    }

    /// Soma da diagonal principal.
    pub fn trace(&self) -> T {
        (0..self.rows().min(self.cols())).map(|i| self.get(i, i)).sum()
    }

    /// Norma 1 da visão 2-D (maior soma de módulos de coluna).
    pub fn norm1(&self) -> f64 {
        self.columns()
            .map(|col| col.iter().map(|x| x.abs()).sum::<f64>())
            .fold(0.0, f64::max)
    }

    /// Colunas da visão 2-D como fatias contíguas (vazio se não houver linhas).
    fn columns(&self) -> std::slice::ChunksExact<'_, T> {
        self.data.chunks_exact(self.rows().max(1))
    }

    /// Converte para complexo.
    pub fn to_complex(&self) -> NdArray<Complex64> {
        self.map(|x| x.to_complex())
    }
}

impl NdArray<Complex64> {
    /// Converte um array real em complexo.
    pub fn from_real(m: &NdArray<f64>) -> Self {
        m.to_complex()
    }

    /// Parte real, se todas as partes imaginárias forem ≤ `tol · max|elemento|`.
    pub fn to_real(&self, tol: f64) -> Option<NdArray<f64>> {
        let scale = self.max_abs();
        self.data
            .iter()
            .all(|z| z.im.abs() <= tol * scale)
            .then(|| self.map(|z| z.re))
    }

    /// Verdadeiro se todas as partes imaginárias são exatamente zero.
    pub fn is_real(&self) -> bool {
        self.data.iter().all(|z| z.im == 0.0)
    }
}
