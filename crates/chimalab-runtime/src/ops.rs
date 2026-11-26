//! Operações: elemento a elemento com expansão implícita e regras de classe (F003, F005),
//! comparações, produto matricial e resolução de sistemas.

use chimalab_array::{Complex64, NdArray, broadcast_shape};
use chimalab_math::linalg::{LinalgWarning, RealOrComplex, funm, inv, solve};

use crate::{Class, IntClass, IntData, RuntimeError, Value};

/// Operação aritmética elemento a elemento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    /// Soma.
    Add,
    /// Subtração.
    Sub,
    /// Produto elemento a elemento.
    Mul,
    /// Divisão à direita, a/b.
    Div,
    /// Divisão à esquerda, b/a.
    LeftDiv,
    /// Potência elemento a elemento.
    Pow,
}

impl BinOp {
    fn name(self) -> &'static str {
        match self {
            Self::Add => "soma",
            Self::Sub => "subtração",
            Self::Mul => "produto elemento a elemento",
            Self::Div => "divisão",
            Self::LeftDiv => "divisão à esquerda",
            Self::Pow => "potência",
        }
    }

    fn f64(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Add => a + b,
            Self::Sub => a - b,
            Self::Mul => a * b,
            Self::Div => a / b,
            Self::LeftDiv => b / a,
            Self::Pow => a.powf(b),
        }
    }

    fn complex(self, a: Complex64, b: Complex64) -> Complex64 {
        match self {
            Self::Add => a + b,
            Self::Sub => a - b,
            Self::Mul => a * b,
            Self::Div => a / b,
            Self::LeftDiv => b / a,
            Self::Pow => {
                // Expoente inteiro pequeno: multiplicação repetida (exata para inteiros gaussianos).
                if b.im == 0.0 && b.re.fract() == 0.0 && b.re.abs() <= 64.0 {
                    a.powi(b.re as i32)
                } else {
                    a.powc(b)
                }
            }
        }
    }
}

/// Classe do resultado de uma operação aritmética entre duas classes (F003).
fn result_class(op: &'static str, a: &Value, b: &Value) -> Result<Class, RuntimeError> {
    let (ca, cb) = (a.class(), b.class());
    let erro = || RuntimeError::IncompatibleClasses {
        operation: op,
        left: ca.name().into(),
        right: cb.name().into(),
    };
    Ok(match (ca, cb) {
        (Class::Int(x), Class::Int(y)) => {
            if x == y {
                Class::Int(x)
            } else {
                return Err(erro());
            }
        }
        (Class::Int(x), _) | (_, Class::Int(x)) => {
            if a.is_complex() || b.is_complex() {
                return Err(erro());
            }
            Class::Int(x)
        }
        (Class::Single, _) | (_, Class::Single) => {
            if a.is_complex() || b.is_complex() {
                return Err(erro());
            }
            Class::Single
        }
        _ => Class::Double,
    })
}

/// Expande os índices lineares de cada operando para a forma de saída.
pub(crate) fn broadcast_index(dims: &[usize], out: &[usize]) -> Vec<usize> {
    let total: usize = out.iter().product();
    let mut idx = Vec::with_capacity(total);
    let mut sub = vec![0usize; out.len()];
    for _ in 0..total {
        let mut lin = 0;
        let mut stride = 1;
        for (d, &s) in sub.iter().enumerate() {
            let n = dims.get(d).copied().unwrap_or(1);
            lin += if n == 1 { 0 } else { s } * stride;
            stride *= n;
        }
        idx.push(lin);
        for (d, s) in sub.iter_mut().enumerate() {
            *s += 1;
            if *s < out[d] {
                break;
            }
            *s = 0;
        }
    }
    idx
}

/// a ⊙ b elemento a elemento, com expansão implícita.
///
/// Erros: `classesIncompativeis`, `dimensoesIncompativeis`.
pub fn elementwise(op: BinOp, a: &Value, b: &Value) -> Result<Value, RuntimeError> {
    let class = result_class(op.name(), a, b)?;
    let out = broadcast_shape(a.dims(), b.dims()).map_err(|_| RuntimeError::IncompatibleShapes {
        operation: op.name(),
        left: a.dims().to_vec(),
        right: b.dims().to_vec(),
    })?;
    verifica_tamanho(&out)?;
    let (ia, ib) = (broadcast_index(a.dims(), &out), broadcast_index(b.dims(), &out));
    match class {
        Class::Int(c) => {
            let (xa, xb) = (int_operand(a)?, int_operand(b)?);
            let v: Vec<i128> = ia
                .iter()
                .zip(&ib)
                .map(|(&i, &j)| int_op(op, c, xa.get(i), xb.get(j)))
                .collect();
            Ok(Value::Int(IntData::from_i128(c, &out, v)))
        }
        Class::Single => {
            let (xa, xb) = (a.to_f64s()?, b.to_f64s()?);
            // Calcula em f64 e arredonda para f32 (cada operação tem um único arredondamento em f32).
            let v: Vec<f32> = ia
                .iter()
                .zip(&ib)
                .map(|(&i, &j)| op.f64(xa[i], xb[j]) as f32)
                .collect();
            Ok(Value::Single(NdArray::from_shape_vec(&out, v).expect("forma")))
        }
        _ => {
            let complexo = a.is_complex() || b.is_complex() || {
                // Potência de base negativa com expoente não inteiro: resultado complexo.
                op == BinOp::Pow && {
                    let (xa, xb) = (a.to_f64s()?, b.to_f64s()?);
                    ia.iter()
                        .zip(&ib)
                        .any(|(&i, &j)| xa[i] < 0.0 && xb[j].fract() != 0.0 && xb[j].is_finite())
                }
            };
            if complexo {
                let (xa, xb) = (a.to_complex(), b.to_complex());
                let v: Vec<Complex64> = ia
                    .iter()
                    .zip(&ib)
                    .map(|(&i, &j)| op.complex(xa[i], xb[j]))
                    .collect();
                Ok(simplify(NdArray::from_shape_vec(&out, v).expect("forma")))
            } else {
                let (xa, xb) = (a.to_f64s()?, b.to_f64s()?);
                let v: Vec<f64> = ia.iter().zip(&ib).map(|(&i, &j)| op.f64(xa[i], xb[j])).collect();
                Ok(Value::Double(NdArray::from_shape_vec(&out, v).expect("forma")))
            }
        }
    }
}

/// Complexo com todas as partes imaginárias nulas volta a ser real.
pub(crate) fn simplify(a: NdArray<Complex64>) -> Value {
    if a.as_slice().iter().all(|z| z.im == 0.0) {
        Value::Double(a.map(|z| z.re))
    } else {
        Value::Complex(a)
    }
}

/// Operando de uma operação inteira: inteiros exatos, ou reais.
enum IntOperand {
    Exact(Vec<i128>),
    Real(Vec<f64>),
}

impl IntOperand {
    fn get(&self, i: usize) -> (Option<i128>, f64) {
        match self {
            Self::Exact(v) => (Some(v[i]), v[i] as f64),
            Self::Real(v) => {
                let x = v[i];
                // Real com valor inteiro representável: também vale como exato.
                let exato = (x.fract() == 0.0 && x.abs() < 1.7e38).then_some(x as i128);
                (exato, x)
            }
        }
    }
}

fn int_operand(v: &Value) -> Result<IntOperand, RuntimeError> {
    Ok(match v {
        Value::Int(d) => IntOperand::Exact(d.to_i128()),
        other => IntOperand::Real(other.to_f64s()?),
    })
}

/// Uma operação na classe inteira c: exata em i128 quando os dois operandos são inteiros e a
/// operação é +, − ou ×; senão em f64, com arredondamento do meio para longe de zero e
/// saturação. Divisão por zero satura (0/0 = 0).
fn int_op(op: BinOp, c: IntClass, a: (Option<i128>, f64), b: (Option<i128>, f64)) -> i128 {
    if let (Some(x), Some(y)) = (a.0, b.0) {
        match op {
            BinOp::Add => return c.saturate(x.saturating_add(y)),
            BinOp::Sub => return c.saturate(x.saturating_sub(y)),
            BinOp::Mul => return c.saturate(x.saturating_mul(y)),
            BinOp::Div | BinOp::LeftDiv => {
                let (n, d) = if op == BinOp::Div { (x, y) } else { (y, x) };
                if d == 0 {
                    return match n.signum() {
                        1 => c.max(),
                        -1 => c.min(),
                        _ => 0,
                    };
                }
                // Quociente arredondado com o meio para longe de zero, exato.
                let (q, r) = (n / d, n % d);
                let ajuste = if r.unsigned_abs() >= d.unsigned_abs() - r.unsigned_abs() && r != 0 {
                    if (n < 0) != (d < 0) { -1 } else { 1 }
                } else {
                    0
                };
                return c.saturate(q + ajuste);
            }
            BinOp::Pow => {}
        }
    }
    let r = op.f64(a.1, b.1);
    c.from_f64(r)
}

/// −a. Inteiros saturam (−int8(−128) = 127), em aritmética exata; ponto flutuante usa a negação
/// IEEE (−(+0) = −0, 1/(−0) = −∞); lógico e caractere viram double.
///
/// Regressão: −a era calculado como 0 − a, que dá +0 para a = +0 (perdia o sinal do zero).
pub fn neg(a: &Value) -> Result<Value, RuntimeError> {
    Ok(match a {
        Value::Int(d) => {
            let c = d.class();
            let v = d.to_i128().into_iter().map(|x| c.saturate(-x)).collect();
            Value::Int(IntData::from_i128(c, d.dims(), v))
        }
        Value::Single(x) => Value::Single(x.map(|v| -v)),
        Value::Complex(z) => Value::Complex(z.map(|w| -w)),
        other => {
            let x = other.to_f64s()?;
            Value::Double(
                NdArray::from_shape_vec(other.dims(), x.into_iter().map(|v| -v).collect()).expect("forma"),
            )
        }
    })
}

/// Raiz quadrada; reais negativos dão complexo (sqrt(−4) = 2i). Inteiros: arredonda e satura.
pub fn sqrt(a: &Value) -> Result<Value, RuntimeError> {
    match a {
        Value::Int(d) => {
            let v: Vec<i128> = d
                .to_i128()
                .into_iter()
                .map(|x| d.class().from_f64((x as f64).sqrt()))
                .collect();
            Ok(Value::Int(IntData::from_i128(d.class(), d.dims(), v)))
        }
        Value::Single(s) => {
            if s.as_slice().iter().any(|&x| x < 0.0) {
                return Err(RuntimeError::IncompatibleClasses {
                    operation: "raiz quadrada",
                    left: "single".into(),
                    right: "complex".into(),
                });
            }
            Ok(Value::Single(s.map(|x| x.sqrt())))
        }
        Value::Complex(z) => Ok(simplify(z.map(|w| w.sqrt()))),
        other => {
            let x = other.to_f64s()?;
            let dims = other.dims().to_vec();
            if x.iter().any(|&v| v < 0.0) {
                let z: Vec<Complex64> = x.iter().map(|&v| Complex64::new(v, 0.0).sqrt()).collect();
                Ok(simplify(NdArray::from_shape_vec(&dims, z).expect("forma")))
            } else {
                Ok(Value::Double(
                    NdArray::from_shape_vec(&dims, x.iter().map(|v| v.sqrt()).collect()).expect("forma"),
                ))
            }
        }
    }
}

/// Comparação elemento a elemento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    /// ==
    Eq,
    /// ~= / !=
    Ne,
    /// <
    Lt,
    /// <=
    Le,
    /// >
    Gt,
    /// >=
    Ge,
}

/// Comparação com expansão implícita; o resultado é lógico. Inteiros comparam exatamente entre
/// si; complexos só por igualdade (as ordens usam a parte real, como na prática comum).
pub fn compare(op: CmpOp, a: &Value, b: &Value) -> Result<Value, RuntimeError> {
    let out = broadcast_shape(a.dims(), b.dims()).map_err(|_| RuntimeError::IncompatibleShapes {
        operation: "comparação",
        left: a.dims().to_vec(),
        right: b.dims().to_vec(),
    })?;
    verifica_tamanho(&out)?;
    let (ia, ib) = (broadcast_index(a.dims(), &out), broadcast_index(b.dims(), &out));
    let ord = |x: std::cmp::Ordering| match op {
        CmpOp::Eq => x.is_eq(),
        CmpOp::Ne => !x.is_eq(),
        CmpOp::Lt => x.is_lt(),
        CmpOp::Le => x.is_le(),
        CmpOp::Gt => x.is_gt(),
        CmpOp::Ge => x.is_ge(),
    };
    let v: Vec<bool> = if let (Value::Int(x), Value::Int(y)) = (a, b) {
        let (x, y) = (x.to_i128(), y.to_i128());
        ia.iter().zip(&ib).map(|(&i, &j)| ord(x[i].cmp(&y[j]))).collect()
    } else if a.is_complex() || b.is_complex() {
        let (x, y) = (a.to_complex(), b.to_complex());
        ia.iter()
            .zip(&ib)
            .map(|(&i, &j)| match op {
                CmpOp::Eq => x[i] == y[j],
                CmpOp::Ne => x[i] != y[j],
                _ => x[i].re.partial_cmp(&y[j].re).is_some_and(ord),
            })
            .collect()
    } else {
        let (x, y) = (a.to_f64s()?, b.to_f64s()?);
        ia.iter()
            .zip(&ib)
            .map(|(&i, &j)| match x[i].partial_cmp(&y[j]) {
                Some(o) => ord(o),
                // NaN: só ~= é verdadeiro.
                None => op == CmpOp::Ne,
            })
            .collect()
    };
    Ok(Value::Logical(NdArray::from_shape_vec(&out, v).expect("forma")))
}

/// Avisos não fatais.
#[derive(Clone, Debug, PartialEq)]
pub enum Warning {
    /// Matriz singular ou mal condicionada no `\` (rcond estimado).
    SingularMatrix {
        /// Recíproco do número de condição estimado.
        rcond: f64,
    },
}

impl Warning {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::SingularMatrix { .. } => "chimalab:runtime:matrizSingular",
        }
    }
}

fn real_matrix(v: &Value, op: &'static str) -> Result<NdArray<f64>, RuntimeError> {
    if v.dims().len() > 2 {
        return Err(RuntimeError::IncompatibleShapes {
            operation: op,
            left: v.dims().to_vec(),
            right: vec![],
        });
    }
    if v.is_complex() || matches!(v, Value::Int(_) | Value::Single(_)) {
        return Err(RuntimeError::IncompatibleClasses {
            operation: op,
            left: v.class().name().into(),
            right: "double".into(),
        });
    }
    Ok(NdArray::from_shape_vec(v.dims(), v.to_f64s()?).expect("forma"))
}

/// Classe de uma operação matricial: (complexo, single). Inteiros e arrays N-D são recusados;
/// `single` complexo também (incremento futuro).
fn classe_matricial(a: &Value, b: &Value, op: &'static str) -> Result<(bool, bool), RuntimeError> {
    for v in [a, b] {
        if v.dims().len() > 2 {
            return Err(RuntimeError::IncompatibleShapes {
                operation: op,
                left: a.dims().to_vec(),
                right: b.dims().to_vec(),
            });
        }
        if matches!(v, Value::Int(_)) {
            return Err(RuntimeError::IncompatibleClasses {
                operation: op,
                left: a.class().name().into(),
                right: b.class().name().into(),
            });
        }
    }
    let complexo = a.is_complex() || b.is_complex();
    let single = matches!(a, Value::Single(_)) || matches!(b, Value::Single(_));
    if complexo && single {
        return Err(RuntimeError::IncompatibleClasses {
            operation: op,
            left: a.class().name().into(),
            right: b.class().name().into(),
        });
    }
    Ok((complexo, single))
}

fn produto<T>(x: &NdArray<T>, y: &NdArray<T>) -> NdArray<T>
where
    T: chimalab_array::Scalar + Copy + std::iter::Sum + std::ops::Mul<Output = T>,
{
    let (m, k, n) = (x.rows(), x.cols(), y.cols());
    NdArray::from_fn(m, n, |i, j| (0..k).map(|l| x.get(i, l) * y.get(l, j)).sum())
}

/// Produto matricial A·B: double real ou complexo, ou `single` (acumulado em double e
/// arredondado uma vez); escalares viram produto elemento a elemento. Complexo com parte
/// imaginária nula volta a real.
///
/// Erros: `dimensoesIncompativeis` (colunas de A ≠ linhas de B, ou N-D), `classesIncompativeis`
/// (inteiros, ou `single` com complexo).
pub fn mtimes(a: &Value, b: &Value) -> Result<Value, RuntimeError> {
    if a.numel() == 1 || b.numel() == 1 {
        return elementwise(BinOp::Mul, a, b);
    }
    const OP: &str = "produto matricial";
    let (complexo, single) = classe_matricial(a, b, OP)?;
    let (ra, ca) = (a.dims()[0], a.dims()[1]);
    let (rb, cb) = (b.dims()[0], b.dims()[1]);
    if ca != rb {
        return Err(RuntimeError::IncompatibleShapes {
            operation: OP,
            left: a.dims().to_vec(),
            right: b.dims().to_vec(),
        });
    }
    verifica_tamanho(&[ra, cb])?;
    if complexo {
        let x = NdArray::from_shape_vec(&[ra, ca], a.to_complex()).expect("forma");
        let y = NdArray::from_shape_vec(&[rb, cb], b.to_complex()).expect("forma");
        return Ok(simplify(produto(&x, &y)));
    }
    let x = NdArray::from_shape_vec(&[ra, ca], a.to_f64s()?).expect("forma");
    let y = NdArray::from_shape_vec(&[rb, cb], b.to_f64s()?).expect("forma");
    let c = produto(&x, &y);
    Ok(if single {
        Value::Single(c.map(|v| *v as f32))
    } else {
        Value::Double(c)
    })
}

pub(crate) fn converte_avisos(ws: &[LinalgWarning]) -> Vec<Warning> {
    ws.iter()
        .map(|w| match w {
            LinalgWarning::Singular | LinalgWarning::RankDeficient { .. } => {
                Warning::SingularMatrix { rcond: 0.0 }
            }
            LinalgWarning::NearlySingular { rcond } => Warning::SingularMatrix { rcond: *rcond },
        })
        .collect()
}

fn de_matfun(r: RealOrComplex) -> Value {
    match r {
        RealOrComplex::Real(m) => Value::Double(m),
        RealOrComplex::Complex(z) => simplify(z),
    }
}

/// Potência de matriz `A^p` (F005):
/// - escalar com escalar: a potência elemento a elemento;
/// - matriz quadrada com expoente inteiro: quadrados sucessivos (exato para matrizes inteiras
///   enquanto os valores couberem em 2⁵³); expoente negativo usa a inversa, com o aviso de matriz
///   singular; `A^0` é a identidade;
/// - expoente não inteiro: função de matriz z^p pelo método de Schur–Parlett (F040), com o ramo
///   principal (resultado complexo quando A tem autovalores negativos);
/// - escalar com matriz: s^B = exp(B·ln s), também por Schur–Parlett.
///
/// Erros: `dimensoesIncompativeis` (matriz não quadrada, ou matriz com matriz),
/// `classesIncompativeis` (complexo, inteiro ou single em potência de matriz).
pub fn mpower(a: &Value, b: &Value) -> Result<(Value, Vec<Warning>), RuntimeError> {
    const OP: &str = "potência matricial";
    if a.numel() == 1 && b.numel() == 1 {
        return Ok((elementwise(BinOp::Pow, a, b)?, Vec::new()));
    }
    let forma = || RuntimeError::IncompatibleShapes {
        operation: OP,
        left: a.dims().to_vec(),
        right: b.dims().to_vec(),
    };
    if a.numel() != 1 && b.numel() != 1 {
        return Err(forma());
    }
    let (m, escalar, base_matriz) = if b.numel() == 1 {
        (real_matrix(a, OP)?, real_matrix(b, OP)?.get(0, 0), true)
    } else {
        (real_matrix(b, OP)?, real_matrix(a, OP)?.get(0, 0), false)
    };
    if m.dims().len() != 2 || m.rows() != m.cols() {
        return Err(forma());
    }
    let n = m.rows();
    let erro = |e: chimalab_math::linalg::LinalgError| RuntimeError::Math(e.to_string());
    if base_matriz {
        let p = escalar;
        if p.fract() == 0.0 && p.abs() <= 9_007_199_254_740_992.0 {
            let (mut base, avisos) = if p < 0.0 {
                let (inv_a, w) = inv(&m).map_err(erro)?;
                (inv_a, converte_avisos(&w))
            } else {
                (m.clone(), Vec::new())
            };
            let mut k = p.abs() as u64;
            let mut acc = NdArray::from_fn(n, n, |i, j| if i == j { 1.0 } else { 0.0 });
            let produto = |x: &NdArray<f64>, y: &NdArray<f64>| {
                NdArray::from_fn(n, n, |i, j| (0..n).map(|l| x.get(i, l) * y.get(l, j)).sum())
            };
            while k > 0 {
                if k & 1 == 1 {
                    acc = produto(&acc, &base);
                }
                k >>= 1;
                if k > 0 {
                    base = produto(&base, &base);
                }
            }
            return Ok((Value::Double(acc), avisos));
        }
        // z^p e as derivadas p(p−1)…(p−k+1)·z^(p−k), ramo principal.
        let r = funm(&m, |z: Complex64, k: usize| {
            let coef: f64 = (0..k).map(|j| p - j as f64).product();
            Complex64::new(coef, 0.0) * z.powc(Complex64::new(p - k as f64, 0.0))
        })
        .map_err(erro)?;
        return Ok((de_matfun(r.value), converte_avisos(&r.warnings)));
    }
    // s^B: derivadas s^z·(ln s)^k, com o logaritmo principal (s < 0 dá complexo).
    let ln_s = Complex64::new(escalar, 0.0).ln();
    let r = funm(&m, |z: Complex64, k: usize| {
        (z * ln_s).exp() * ln_s.powu(k as u32)
    })
    .map_err(erro)?;
    Ok((de_matfun(r.value), converte_avisos(&r.warnings)))
}

/// A\B: resolve A·X = B escolhendo o método pela estrutura (F040). Matriz singular ou mal
/// condicionada gera aviso, não erro.
pub fn mldivide(a: &Value, b: &Value) -> Result<(Value, Vec<Warning>), RuntimeError> {
    if a.numel() == 1 {
        return Ok((elementwise(BinOp::LeftDiv, a, b)?, Vec::new()));
    }
    let (complexo, single) = classe_matricial(a, b, "\\")?;
    if single {
        // `single` em sistemas lineares: incremento futuro (a fatoração é em double).
        return Err(RuntimeError::IncompatibleClasses {
            operation: "\\",
            left: a.class().name().into(),
            right: b.class().name().into(),
        });
    }
    if a.dims()[0] != b.dims()[0] {
        return Err(RuntimeError::IncompatibleShapes {
            operation: "\\",
            left: a.dims().to_vec(),
            right: b.dims().to_vec(),
        });
    }
    if complexo {
        let x = NdArray::from_shape_vec(&a.dims()[..2], a.to_complex()).expect("forma");
        let y = NdArray::from_shape_vec(&b.dims()[..2], b.to_complex()).expect("forma");
        let s = solve(&x, &y).map_err(|e| RuntimeError::Math(e.to_string()))?;
        return Ok((simplify(s.x), converte_avisos(&s.warnings)));
    }
    let (x, y) = (real_matrix(a, "\\")?, real_matrix(b, "\\")?);
    let s = solve(&x, &y).map_err(|e| RuntimeError::Math(e.to_string()))?;
    Ok((Value::Double(s.x), converte_avisos(&s.warnings)))
}

/// A/B: resolve X·B = A pela identidade A/B = (Bᵀ \ Aᵀ)ᵀ (com B escalar, a divisão elemento a
/// elemento). Mesmos avisos e erros de [`mldivide`], com as formas na ordem de A/B.
pub fn mrdivide(a: &Value, b: &Value) -> Result<(Value, Vec<Warning>), RuntimeError> {
    if b.numel() == 1 {
        return Ok((elementwise(BinOp::Div, a, b)?, Vec::new()));
    }
    if a.dims().len() == 2 && b.dims().len() == 2 && a.dims()[1] != b.dims()[1] {
        return Err(RuntimeError::IncompatibleShapes {
            operation: "/",
            left: a.dims().to_vec(),
            right: b.dims().to_vec(),
        });
    }
    let (x, w) = mldivide(&transpose(b, false)?, &transpose(a, false)?)?;
    Ok((transpose(&x, false)?, w))
}

/// Operação lógica elemento a elemento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicOp {
    /// e.
    And,
    /// ou.
    Or,
    /// ou-exclusivo.
    Xor,
}

/// Verdade de cada elemento: não nulo é verdadeiro; NaN dá erro (não tem valor lógico).
pub(crate) fn truth(v: &Value) -> Result<Vec<bool>, RuntimeError> {
    match v {
        Value::Logical(a) => Ok(a.as_slice().to_vec()),
        Value::Complex(z) => {
            if z.as_slice().iter().any(|w| w.re.is_nan() || w.im.is_nan()) {
                return Err(RuntimeError::InvalidConversion("NaN não tem valor lógico".into()));
            }
            Ok(z.as_slice().iter().map(|w| w.re != 0.0 || w.im != 0.0).collect())
        }
        other => {
            let x = other.to_f64s()?;
            if x.iter().any(|v| v.is_nan()) {
                return Err(RuntimeError::InvalidConversion("NaN não tem valor lógico".into()));
            }
            Ok(x.into_iter().map(|v| v != 0.0).collect())
        }
    }
}

/// a op b lógico elemento a elemento, com expansão implícita.
pub fn logical(op: LogicOp, a: &Value, b: &Value) -> Result<Value, RuntimeError> {
    let out = broadcast_shape(a.dims(), b.dims()).map_err(|_| RuntimeError::IncompatibleShapes {
        operation: "operação lógica",
        left: a.dims().to_vec(),
        right: b.dims().to_vec(),
    })?;
    verifica_tamanho(&out)?;
    let (ta, tb) = (truth(a)?, truth(b)?);
    let (ia, ib) = (broadcast_index(a.dims(), &out), broadcast_index(b.dims(), &out));
    let v: Vec<bool> = ia
        .iter()
        .zip(&ib)
        .map(|(&i, &j)| match op {
            LogicOp::And => ta[i] && tb[j],
            LogicOp::Or => ta[i] || tb[j],
            LogicOp::Xor => ta[i] != tb[j],
        })
        .collect();
    Ok(Value::Logical(NdArray::from_shape_vec(&out, v).expect("forma")))
}

/// Negação lógica.
pub fn not(a: &Value) -> Result<Value, RuntimeError> {
    let t = truth(a)?;
    Ok(Value::Logical(
        NdArray::from_shape_vec(a.dims(), t.into_iter().map(|b| !b).collect()).expect("forma"),
    ))
}

/// Transposta (2-D); `conjugate` conjuga os complexos.
///
/// Erros: `dimensoesIncompativeis` para mais de 2 dimensões.
pub fn transpose(a: &Value, conjugate: bool) -> Result<Value, RuntimeError> {
    if a.dims().len() > 2 {
        return Err(RuntimeError::IncompatibleShapes {
            operation: "transposta",
            left: a.dims().to_vec(),
            right: vec![],
        });
    }
    fn t<T: chimalab_array::Element>(x: &NdArray<T>) -> NdArray<T> {
        let (m, n) = (x.dims()[0], x.dims()[1]);
        let d = x.as_slice();
        NdArray::from_shape_vec(
            &[n, m],
            (0..m * n).map(|k| d[(k / n) + m * (k % n)].clone()).collect(),
        )
        .expect("forma")
    }
    Ok(match a {
        Value::Double(x) => Value::Double(t(x)),
        Value::Complex(x) => {
            let y = t(x);
            Value::Complex(if conjugate { y.map(|z| z.conj()) } else { y })
        }
        Value::Single(x) => Value::Single(t(x)),
        Value::Logical(x) => Value::Logical(t(x)),
        Value::Char(x) => Value::Char(t(x)),
        Value::Int(d) => {
            let dims = d.dims().to_vec();
            let v = d.to_i128();
            let tv = t(&NdArray::from_shape_vec(&dims, v).expect("forma"));
            let td = tv.dims().to_vec();
            Value::Int(IntData::from_i128(d.class(), &td, tv.into_vec()))
        }
    })
}

/// Intervalo `início:passo:fim` como vetor linha double.
///
/// O número de elementos é ⌊(fim − início)/passo⌋ + 1, com tolerância de alguns ε relativos no
/// quociente: 0:0,1:1 tem 11 elementos, embora 0,1 não seja exato em binário. Os elementos são
/// início + k·passo, e o último é ajustado para `fim` quando a diferença está dentro da mesma
/// tolerância (0:0,1:1 termina exatamente em 1). Passo 0, NaN ou sentido contrário dão vetor
/// vazio 1×0.
///
/// Monta o vetor inteiro: para intervalos enormes, confira antes [`colon_n`] (o `for` do
/// interpretador percorre com [`colon_elemento`] sem montar).
pub fn colon(start: f64, step: f64, stop: f64) -> Value {
    let n = colon_n(start, step, stop);
    let v: Vec<f64> = (0..n).map(|k| colon_elemento(start, step, stop, n, k)).collect();
    Value::doubles(&[1, n], v)
}

/// Quociente e tolerância do intervalo (`None` se vazio).
fn colon_q(start: f64, step: f64, stop: f64) -> Option<(f64, f64)> {
    if !(start.is_finite() && step.is_finite() && stop.is_finite()) || step == 0.0 {
        return None;
    }
    let q = (stop - start) / step;
    if q < 0.0 || q.is_nan() {
        return None;
    }
    Some((q, 4.0 * f64::EPSILON * q.max(1.0)))
}

/// Número de elementos de `start:step:stop` (satura em `usize::MAX`).
pub fn colon_n(start: f64, step: f64, stop: f64) -> usize {
    colon_q(start, step, stop).map_or(0, |(q, tol)| ((q + tol).floor() as usize).saturating_add(1))
}

/// Elemento `k` (0-based) de `start:step:stop`, que tem `n` elementos: início + k·passo, com o
/// último ajustado para `stop` quando está dentro da tolerância.
pub fn colon_elemento(start: f64, step: f64, stop: f64, n: usize, k: usize) -> f64 {
    let x = start + k as f64 * step;
    if k + 1 == n
        && let Some((_, tol)) = colon_q(start, step, stop)
        && ((x - stop) / step).abs() <= tol
    {
        return stop;
    }
    x
}

/// Confere se um array com a forma `dims` cabe no limite [`crate::MAX_ELEMENTOS`].
///
/// Erros: `arrayGrandeDemais`.
pub fn verifica_tamanho(dims: &[usize]) -> Result<(), RuntimeError> {
    let n = dims.iter().try_fold(1usize, |acc, &d| acc.checked_mul(d));
    match n {
        Some(n) if n <= crate::MAX_ELEMENTOS => Ok(()),
        _ => Err(RuntimeError::TooLarge(dims.to_vec())),
    }
}

/// `n` pontos igualmente espaçados de `a` a `b`, com os dois extremos exatos (a semântica de
/// `range(a, b; length=n)`, DT-34). n = 0 dá vetor vazio 1×0; n = 1 dá `[b]`. Quando b − a
/// estoura (extremos de sinais opostos perto do maior f64), usa a·(1 − t) + b·t.
pub fn linspace(a: f64, b: f64, n: usize) -> Value {
    if n == 0 {
        return Value::doubles(&[1, 0], Vec::new());
    }
    if n == 1 {
        return Value::doubles(&[1, 1], vec![b]);
    }
    let m = (n - 1) as f64;
    let passo = (b - a) / m;
    let v: Vec<f64> = (0..n)
        .map(|k| {
            if k == n - 1 {
                b
            } else if passo.is_finite() {
                a + k as f64 * passo
            } else {
                let t = k as f64 / m;
                a * (1.0 - t) + b * t
            }
        })
        .collect();
    Value::doubles(&[1, n], v)
}
