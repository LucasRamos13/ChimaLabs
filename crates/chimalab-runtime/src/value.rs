//! Valores: arrays N-D tipados (F003).

use chimalab_array::{Complex64, NdArray};

use crate::RuntimeError;

/// Classes inteiras.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntClass {
    /// int8.
    I8,
    /// int16.
    I16,
    /// int32.
    I32,
    /// int64.
    I64,
    /// uint8.
    U8,
    /// uint16.
    U16,
    /// uint32.
    U32,
    /// uint64.
    U64,
}

impl IntClass {
    /// Menor valor.
    pub fn min(self) -> i128 {
        match self {
            Self::I8 => i8::MIN.into(),
            Self::I16 => i16::MIN.into(),
            Self::I32 => i32::MIN.into(),
            Self::I64 => i64::MIN.into(),
            _ => 0,
        }
    }

    /// Maior valor.
    pub fn max(self) -> i128 {
        match self {
            Self::I8 => i8::MAX.into(),
            Self::I16 => i16::MAX.into(),
            Self::I32 => i32::MAX.into(),
            Self::I64 => i64::MAX.into(),
            Self::U8 => u8::MAX.into(),
            Self::U16 => u16::MAX.into(),
            Self::U32 => u32::MAX.into(),
            Self::U64 => u64::MAX.into(),
        }
    }

    /// Nome da classe.
    pub fn name(self) -> &'static str {
        match self {
            Self::I8 => "int8",
            Self::I16 => "int16",
            Self::I32 => "int32",
            Self::I64 => "int64",
            Self::U8 => "uint8",
            Self::U16 => "uint16",
            Self::U32 => "uint32",
            Self::U64 => "uint64",
        }
    }

    /// Satura um inteiro no intervalo da classe.
    pub fn saturate(self, v: i128) -> i128 {
        v.clamp(self.min(), self.max())
    }

    /// Converte um real: arredonda o meio para longe de zero, satura e leva NaN a 0.
    pub fn from_f64(self, v: f64) -> i128 {
        if v.is_nan() {
            return 0;
        }
        let r = v.round(); // round() do Rust arredonda o meio para longe de zero
        if r >= self.max() as f64 {
            self.max()
        } else if r <= self.min() as f64 {
            self.min()
        } else {
            r as i128
        }
    }
}

/// Classe de um valor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    /// Ponto flutuante de precisão dupla (real ou complexo).
    Double,
    /// Precisão simples (real).
    Single,
    /// Inteiro.
    Int(IntClass),
    /// Lógico.
    Logical,
    /// Caractere.
    Char,
}

impl Class {
    /// Nome da classe ("double", "int16"...).
    pub fn name(self) -> &'static str {
        match self {
            Self::Double => "double",
            Self::Single => "single",
            Self::Int(c) => c.name(),
            Self::Logical => "logical",
            Self::Char => "char",
        }
    }
}

macro_rules! int_data {
    ($($v:ident($t:ty)),*) => {
        /// Dados inteiros, um tipo de armazenamento por classe.
        #[derive(Clone, Debug, PartialEq)]
        pub enum IntData {
            $(
                #[doc = concat!("Elementos ", stringify!($t), ".")]
                $v(NdArray<$t>),
            )*
        }

        impl IntData {
            /// Classe.
            pub fn class(&self) -> IntClass {
                match self { $(Self::$v(_) => IntClass::$v,)* }
            }

            /// Dimensões.
            pub fn dims(&self) -> &[usize] {
                match self { $(Self::$v(a) => a.dims(),)* }
            }

            /// Elementos como i128 (column-major).
            pub fn to_i128(&self) -> Vec<i128> {
                match self { $(Self::$v(a) => a.as_slice().iter().map(|&x| i128::from(x)).collect(),)* }
            }

            /// Constrói a partir de i128 já saturados na classe.
            pub fn from_i128(class: IntClass, dims: &[usize], v: Vec<i128>) -> Self {
                match class {
                    $(IntClass::$v => Self::$v(
                        NdArray::from_shape_vec(dims, v.into_iter().map(|x| <$t>::try_from(class.saturate(x)).expect("saturado")).collect())
                            .expect("forma consistente"),
                    ),)*
                }
            }
        }
    };
}

int_data!(
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64)
);

/// Valor da linguagem: um array N-D de uma classe.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// `double` real.
    Double(NdArray<f64>),
    /// `double` complexo.
    Complex(NdArray<Complex64>),
    /// `single` real.
    Single(NdArray<f32>),
    /// Inteiros.
    Int(IntData),
    /// Lógicos.
    Logical(NdArray<bool>),
    /// Caracteres.
    Char(NdArray<char>),
}

impl Value {
    /// Escalar double.
    pub fn scalar(v: f64) -> Self {
        Self::Double(NdArray::filled_nd(&[1, 1], v))
    }

    /// Array double com forma.
    ///
    /// # Panics
    /// Se o número de dados não corresponde à forma.
    pub fn doubles(dims: &[usize], data: Vec<f64>) -> Self {
        Self::Double(NdArray::from_shape_vec(dims, data).expect("forma consistente"))
    }

    /// Texto como vetor linha de caracteres.
    pub fn text(s: &str) -> Self {
        let c: Vec<char> = s.chars().collect();
        let n = c.len();
        Self::Char(NdArray::from_shape_vec(&[1, n], c).expect("forma consistente"))
    }

    /// Classe.
    pub fn class(&self) -> Class {
        match self {
            Self::Double(_) | Self::Complex(_) => Class::Double,
            Self::Single(_) => Class::Single,
            Self::Int(d) => Class::Int(d.class()),
            Self::Logical(_) => Class::Logical,
            Self::Char(_) => Class::Char,
        }
    }

    /// Verdadeiro para complexos.
    pub fn is_complex(&self) -> bool {
        matches!(self, Self::Complex(_))
    }

    /// Dimensões (sempre ≥ 2).
    pub fn dims(&self) -> &[usize] {
        match self {
            Self::Double(a) => a.dims(),
            Self::Complex(a) => a.dims(),
            Self::Single(a) => a.dims(),
            Self::Int(d) => d.dims(),
            Self::Logical(a) => a.dims(),
            Self::Char(a) => a.dims(),
        }
    }

    /// Número de elementos.
    pub fn numel(&self) -> usize {
        self.dims().iter().product()
    }

    /// Elementos como f64 (complexos dão erro; inteiros grandes perdem precisão acima de 2⁵³).
    pub fn to_f64s(&self) -> Result<Vec<f64>, RuntimeError> {
        Ok(match self {
            Self::Double(a) => a.as_slice().to_vec(),
            Self::Complex(_) => {
                return Err(RuntimeError::InvalidConversion("complexo para real".into()));
            }
            Self::Single(a) => a.as_slice().iter().map(|&x| f64::from(x)).collect(),
            Self::Int(d) => d.to_i128().into_iter().map(|x| x as f64).collect(),
            Self::Logical(a) => a.as_slice().iter().map(|&b| f64::from(u8::from(b))).collect(),
            Self::Char(a) => a.as_slice().iter().map(|&c| f64::from(u32::from(c))).collect(),
        })
    }

    /// Elementos como complexos.
    pub fn to_complex(&self) -> Vec<Complex64> {
        match self {
            Self::Complex(a) => a.as_slice().to_vec(),
            other => other
                .to_f64s()
                .expect("não complexo")
                .into_iter()
                .map(|x| Complex64::new(x, 0.0))
                .collect(),
        }
    }

    /// Converte para outra classe.
    /// - Para inteiro: arredonda o meio para longe de zero, satura e leva NaN a 0. Complexo dá
    ///   erro.
    /// - Para lógico: não nulo vira verdadeiro; NaN dá erro.
    /// - Para caractere: arredonda e limita aos códigos válidos.
    /// - Para `single`: arredonda para f32. Complexo dá erro (complexo `single` é incremento
    ///   futuro).
    pub fn cast(&self, to: Class) -> Result<Value, RuntimeError> {
        let dims = self.dims().to_vec();
        if self.class() == to {
            return Ok(self.clone());
        }
        if let (Value::Int(d), Class::Int(c)) = (self, to) {
            return Ok(Value::Int(IntData::from_i128(c, &dims, d.to_i128())));
        }
        if self.is_complex() && to != Class::Double {
            return Err(RuntimeError::InvalidConversion(format!(
                "complexo para {}",
                to.name()
            )));
        }
        let v = self.to_f64s()?;
        Ok(match to {
            Class::Double => Value::Double(NdArray::from_shape_vec(&dims, v).expect("forma")),
            Class::Single => Value::Single(
                NdArray::from_shape_vec(&dims, v.into_iter().map(|x| x as f32).collect()).expect("forma"),
            ),
            Class::Int(c) => Value::Int(IntData::from_i128(
                c,
                &dims,
                v.into_iter().map(|x| c.from_f64(x)).collect(),
            )),
            Class::Logical => {
                if v.iter().any(|x| x.is_nan()) {
                    return Err(RuntimeError::InvalidConversion(
                        "NaN não converte para lógico".into(),
                    ));
                }
                Value::Logical(
                    NdArray::from_shape_vec(&dims, v.into_iter().map(|x| x != 0.0).collect()).expect("forma"),
                )
            }
            Class::Char => Value::Char(
                NdArray::from_shape_vec(
                    &dims,
                    v.into_iter()
                        .map(|x| {
                            let c = if x.is_nan() {
                                0.0
                            } else {
                                x.round().clamp(0.0, 0x10_FFFF as f64)
                            };
                            char::from_u32(c as u32).unwrap_or('\u{FFFD}')
                        })
                        .collect(),
                )
                .expect("forma"),
            ),
        })
    }
}
