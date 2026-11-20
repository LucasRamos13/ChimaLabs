//! F054 — precisão e aritmética especial (incremento 1).
//!
//! Oráculos: definições do IEEE 754 (bits exatos), aritmética inteira exata (i128) e idas e
//! voltas.

use chimalab_math::numeric::{
    IntType, NumericError, Rounding, bin2dec, dec2bin, dec2bin_signed, dec2hex, eps, eps_f32, flintmax,
    flintmax_f32, from_base, hex2dec, hex2num, idivide, next_after, num2hex, realmax, realmin, to_base,
};
use proptest::prelude::*;

#[test]
fn ca1_dec2bin_hex2dec_ida_e_volta() {
    for v in [0u128, 1, 2, 5, 255, 256, 1 << 52, u64::MAX as u128, u128::MAX] {
        assert_eq!(bin2dec(&dec2bin(v, 0)).unwrap(), v);
        assert_eq!(hex2dec(&dec2hex(v, 0)).unwrap(), v);
        for b in 2..=36 {
            assert_eq!(from_base(&to_base(v, b, 0).unwrap(), b).unwrap(), v, "base {b}");
        }
    }
    assert_eq!(dec2bin(5, 0), "101");
    assert_eq!(dec2bin(5, 8), "00000101");
    assert_eq!(dec2bin(0, 0), "0");
    assert_eq!(dec2hex(255, 4), "00FF");
    assert_eq!(to_base(35, 36, 0).unwrap(), "Z");
    // Entrada tolerante: espaços, sublinhados, minúsculas e prefixos.
    assert_eq!(bin2dec(" 1010_1010 ").unwrap(), 170);
    assert_eq!(bin2dec("0b11").unwrap(), 3);
    assert_eq!(hex2dec("0xff").unwrap(), 255);
    assert_eq!(hex2dec("DeadBeef").unwrap(), 0xdead_beef);
    assert_eq!(from_base("0o17", 8).unwrap(), 15);
    // Prefixo de outra base é dígito inválido.
    assert_eq!(
        bin2dec("0x1").unwrap_err().id(),
        "chimalab:numeric:digitoInvalido"
    );
}

#[test]
fn ca2_flintmax_correto() {
    let f = flintmax();
    assert_eq!(f, 2f64.powi(53));
    // f − 1 e f são exatos; f + 1 não é representável (arredonda para f).
    assert_eq!((f - 1.0) + 1.0, f);
    assert_eq!(f + 1.0, f);
    assert_ne!(f + 2.0, f);
    assert_eq!(eps(f), 2.0);
    assert_eq!(eps(f - 1.0), 1.0);
    let g = flintmax_f32();
    assert_eq!(g, 2f32.powi(24));
    assert_eq!(g + 1.0, g);
    assert_eq!(eps_f32(g - 1.0), 1.0);
}

#[test]
fn representacao_ieee() {
    assert_eq!(eps(1.0), f64::EPSILON);
    assert_eq!(eps(-1.0), f64::EPSILON);
    assert_eq!(eps(0.0), f64::from_bits(1));
    assert_eq!(eps(-0.0), 5e-324);
    assert_eq!(eps(1e-310), 5e-324); // subnormal: espaçamento constante
    assert_eq!(eps(f64::MAX), 2f64.powi(971));
    assert!(eps(f64::INFINITY).is_nan() && eps(f64::NAN).is_nan());
    assert_eq!(eps_f32(1.0), f32::EPSILON);
    assert_eq!(eps_f32(f32::MAX), 2f32.powi(104));
    assert!(eps_f32(f32::NAN).is_nan());
    assert_eq!(realmax(), f64::MAX);
    assert_eq!(realmin(), 2f64.powi(-1022));
    // next_after.
    assert_eq!(next_after(1.0, 2.0), 1.0 + f64::EPSILON);
    assert_eq!(next_after(1.0, 0.0), 1.0 - f64::EPSILON / 2.0);
    assert_eq!(next_after(0.0, 1.0), 5e-324);
    assert_eq!(next_after(0.0, -1.0), -5e-324);
    assert_eq!(next_after(-1.0, -2.0), -1.0 - f64::EPSILON);
    assert_eq!(next_after(f64::MAX, f64::INFINITY), f64::INFINITY);
    assert_eq!(next_after(3.0, 3.0), 3.0);
    assert!(next_after(f64::NAN, 1.0).is_nan() && next_after(1.0, f64::NAN).is_nan());
    assert_eq!(next_after(-5e-324, 1.0), -0.0);
}

#[test]
fn divisao_inteira() {
    use Rounding::*;
    let t = IntType::I32;
    let casos = [
        (7, 2, [3, 4, 3, 4]),
        (-7, 2, [-3, -4, -4, -3]),
        (7, -2, [-3, -4, -4, -3]),
        (-7, -2, [3, 4, 3, 4]),
        (6, 3, [2, 2, 2, 2]),
        (5, 4, [1, 1, 1, 2]),
        (-5, 4, [-1, -1, -2, -1]),
    ];
    for (a, b, [fix, round, floor, ceil]) in casos {
        assert_eq!(idivide(a, b, Fix, t), fix, "{a}/{b}");
        assert_eq!(idivide(a, b, Round, t), round, "{a}/{b}");
        assert_eq!(idivide(a, b, Floor, t), floor, "{a}/{b}");
        assert_eq!(idivide(a, b, Ceil, t), ceil, "{a}/{b}");
    }
    // Divisão por zero e saturação.
    assert_eq!(idivide(5, 0, Fix, IntType::I8), 127);
    assert_eq!(idivide(-5, 0, Fix, IntType::I8), -128);
    assert_eq!(idivide(0, 0, Fix, IntType::I8), 0);
    assert_eq!(idivide(-128, -1, Fix, IntType::I8), 127);
    assert_eq!(idivide(-5, 1, Fix, IntType::U8), 0);
    assert_eq!(idivide(i128::MIN, -1, Fix, IntType::I64), i64::MAX as i128);
    // Arredondamento sem estouro com operandos enormes.
    assert_eq!(idivide(i128::MAX, i128::MAX - 1, Round, IntType::I64), 1);
    // Limites dos tipos.
    let lim = [
        (IntType::I8, -128, 127),
        (IntType::I16, -32768, 32767),
        (IntType::I32, i32::MIN as i128, i32::MAX as i128),
        (IntType::I64, i64::MIN as i128, i64::MAX as i128),
        (IntType::U8, 0, 255),
        (IntType::U16, 0, 65535),
        (IntType::U32, 0, u32::MAX as i128),
        (IntType::U64, 0, u64::MAX as i128),
    ];
    for (ty, lo, hi) in lim {
        assert_eq!((ty.min(), ty.max()), (lo, hi));
        assert_eq!(ty.saturate(i128::MAX), hi);
    }
}

#[test]
fn complemento_de_dois_e_bits_ieee() {
    assert_eq!(dec2bin_signed(-1, 8).unwrap(), "11111111");
    assert_eq!(dec2bin_signed(-128, 8).unwrap(), "10000000");
    // Extremos exatos do intervalo com sinal (127 e −128 em 8 bits) são aceitos.
    assert_eq!(dec2bin_signed(127, 8).unwrap(), "01111111");
    assert_eq!(
        dec2bin_signed(i128::MAX, 128).unwrap(),
        format!("0{}", "1".repeat(127))
    );
    assert_eq!(dec2bin_signed(5, 4).unwrap(), "0101");
    assert_eq!(
        dec2bin_signed(i128::MIN, 128).unwrap(),
        format!("1{}", "0".repeat(127))
    );
    assert_eq!(
        dec2bin_signed(128, 8).unwrap_err().id(),
        "chimalab:numeric:estouro"
    );
    assert_eq!(
        dec2bin_signed(-129, 8).unwrap_err().id(),
        "chimalab:numeric:estouro"
    );
    assert_eq!(
        dec2bin_signed(0, 0).unwrap_err().id(),
        "chimalab:numeric:bitsInvalidos"
    );
    assert_eq!(
        dec2bin_signed(0, 129).unwrap_err().id(),
        "chimalab:numeric:bitsInvalidos"
    );
    // Bits IEEE.
    assert_eq!(num2hex(1.0), "3ff0000000000000");
    assert_eq!(num2hex(-0.0), "8000000000000000");
    assert_eq!(hex2num("3ff").unwrap(), 1.0);
    assert_eq!(hex2num("0x4000000000000000").unwrap(), 2.0);
    assert!(hex2num("7ff8").unwrap().is_nan());
    for x in [0.1, -2.5e-310, f64::MAX, f64::INFINITY] {
        assert_eq!(hex2num(&num2hex(x)).unwrap().to_bits(), x.to_bits());
    }
    for bad in ["", "xyz", "12345678901234567", "0x"] {
        assert_eq!(
            hex2num(bad).unwrap_err().id(),
            "chimalab:numeric:digitoInvalido",
            "{bad}"
        );
    }
}

#[test]
fn erros_com_identificador() {
    let casos: Vec<(NumericError, &str)> = vec![
        (to_base(1, 1, 0).unwrap_err(), "chimalab:numeric:baseInvalida"),
        (to_base(1, 37, 0).unwrap_err(), "chimalab:numeric:baseInvalida"),
        (from_base("1", 40).unwrap_err(), "chimalab:numeric:baseInvalida"),
        (bin2dec("102").unwrap_err(), "chimalab:numeric:digitoInvalido"),
        (bin2dec("").unwrap_err(), "chimalab:numeric:digitoInvalido"),
        (bin2dec("_1").unwrap_err(), "chimalab:numeric:digitoInvalido"),
        (bin2dec("1_").unwrap_err(), "chimalab:numeric:digitoInvalido"),
        (hex2dec(&"F".repeat(33)).unwrap_err(), "chimalab:numeric:estouro"),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
        assert!(!e.to_string().is_empty());
    }
    assert!(!NumericError::InvalidBits(0).to_string().is_empty());
}

proptest! {
    /// Ida e volta em qualquer base e com dígitos mínimos.
    #[test]
    fn ida_e_volta(v in any::<u128>(), base in 2u32..=36, min in 0usize..140) {
        let s = to_base(v, base, min).unwrap();
        prop_assert!(s.len() >= min.max(1));
        prop_assert_eq!(from_base(&s, base).unwrap(), v);
        prop_assert_eq!(from_base(&s.to_lowercase(), base).unwrap(), v);
    }

    /// idivide contra a definição em racionais exatos: |q − a/b| < 1 e o arredondamento pedido.
    #[test]
    fn idivide_contra_definicao(a in -1_000_000i128..1_000_000, b in -1000i128..1000) {
        prop_assume!(b != 0);
        let t = IntType::I64;
        let fl = idivide(a, b, Rounding::Floor, t);
        let ce = idivide(a, b, Rounding::Ceil, t);
        let fx = idivide(a, b, Rounding::Fix, t);
        let rd = idivide(a, b, Rounding::Round, t);
        // floor: fl·b ≤ a < (fl+1)·b em termos de a/b (multiplica pelo sinal de b).
        let s = b.signum();
        prop_assert!(fl * b * s <= a * s && a * s < (fl + 1) * b * s);
        prop_assert!(ce - fl == i128::from(a % b != 0));
        prop_assert_eq!(fx, if (a < 0) != (b < 0) { ce } else { fl });
        // round: |a/b − rd| ≤ ½, empate afastando de zero.
        let d2 = (2 * (a - rd * b)).abs();
        prop_assert!(d2 <= b.abs());
        if d2 == b.abs() {
            prop_assert!(rd.abs() > (a / b).abs() || a % b == 0);
        }
    }

    /// eps(x) é o espaçamento: x + eps(x) é o próximo f64, para x finito positivo não máximo.
    #[test]
    fn eps_e_o_espacamento(bits in 0u64..0x7fef_ffff_ffff_ffff) {
        let x = f64::from_bits(bits);
        prop_assert_eq!(x + eps(x), next_after(x, f64::INFINITY));
    }
}

// ================================================================ incremento 2: dupla-dupla

use chimalab_math::numeric::DoubleDouble as Dd;

/// Erro em unidades de 2⁻¹⁰⁴ relativo ao valor de referência (com piso absoluto de 2⁻¹⁰⁷⁴·8, para
/// resultados cuja parte baixa é subnormal).
fn ulps(nosso: Dd, r: Dd) -> f64 {
    let d = (nosso - r).abs().to_f64();
    let escala = r.abs().to_f64() * Dd::EPSILON;
    if d <= 8.0 * f64::from_bits(1) {
        return 0.0;
    }
    d / escala
}

#[test]
fn dupla_dupla_contra_mpmath() {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F054/dd.csv"
    ))
    .unwrap();
    let mut pior: std::collections::BTreeMap<String, f64> = Default::default();
    for linha in csv.lines().skip(1) {
        let c: Vec<&str> = linha.split(',').collect();
        let f = |i: usize| c[i].parse::<f64>().unwrap();
        let (op, r) = (c[0], Dd::from_parts(f(5), f(6)));
        let nosso = match op {
            "const" => {
                let k = match c[1] {
                    "pi" => Dd::PI,
                    "e" => Dd::E,
                    _ => Dd::LN_2,
                };
                assert_eq!((k.hi(), k.lo()), (f(2), f(3)), "{}", c[1]);
                continue;
            }
            "parse" => c[1].parse::<Dd>().unwrap(),
            _ => {
                let a = Dd::from_parts(f(1), f(2));
                let b = Dd::from_parts(f(3), f(4));
                match op {
                    "add" => a + b,
                    "sub" => a - b,
                    "mul" => a * b,
                    "div" => a / b,
                    "sqrt" => a.sqrt(),
                    "ln" => a.ln(),
                    "exp" => a.exp(),
                    "powi" => a.powi(f(3) as i32),
                    _ => unreachable!("{op}"),
                }
            }
        };
        let e = if r.hi() == 0.0 {
            (nosso - r).abs().to_f64()
        } else {
            ulps(nosso, r)
        };
        let p = pior.entry(op.to_string()).or_insert(0.0);
        *p = p.max(e);
    }
    println!("DD pior erro (ulp de 2⁻¹⁰⁴): {pior:?}");
    // Medido: soma, subtração, produto, divisão, raiz e exp ≤ 0,46 ulp; leitura 3,3; ln 6,1;
    // potência (até 40 produtos) 14.
    let lim = |op: &str| match op {
        "parse" => 8.0,
        "ln" => 16.0,
        "powi" => 32.0,
        _ => 2.0,
    };
    for (op, e) in &pior {
        assert!(*e <= lim(op), "{op}: {e}");
    }
}

#[test]
fn dupla_dupla_bordas_texto_e_erros() {
    // Texto: π com 32 algarismos (mpmath: 3,14159265358979323846264338327950288…).
    assert_eq!(Dd::PI.to_string(), "3.1415926535897932384626433832795e+00");
    assert_eq!(format!("{:.4}", Dd::E), "2.7183e+00");
    assert_eq!(Dd::from_f64(-0.001).to_string_digits(3), "-1.00e-03");
    // Arredondamento que propaga o "vai um": 9,9999… → 1,0e+01.
    assert_eq!(Dd::from_f64(9.99996).to_string_digits(4), "1.000e+01");
    assert_eq!(Dd::from_f64(1.0).to_string_digits(0), "1e+00");
    assert_eq!(Dd::from_f64(0.0).to_string(), "0e+00");
    assert_eq!(Dd::from_f64(-0.0).to_string(), "-0e+00");
    assert_eq!(Dd::from_f64(f64::NAN).to_string(), "NaN");
    assert_eq!(Dd::from_f64(f64::INFINITY).to_string(), "Inf");
    assert_eq!(Dd::from_f64(f64::NEG_INFINITY).to_string(), "-Inf");
    // 0,1 lido em dupla-dupla é muito mais preciso que o f64 0,1.
    let um_decimo: Dd = "0.1".parse().unwrap();
    assert!(um_decimo.lo() != 0.0);
    assert!(
        (um_decimo * Dd::from_f64(10.0) - Dd::from_f64(1.0))
            .abs()
            .to_f64()
            < 1e-31
    );
    // Logo abaixo de uma potência de 10 (hi = 1, lo < 0): o expoente estimado por log10(hi) é
    // corrigido.
    assert_eq!(
        Dd::from_parts(1.0, -1e-20).to_string_digits(22),
        "9.999999999999999999900e-01"
    );
    for (t, v) in [
        ("+inf", f64::INFINITY),
        ("-Infinity", f64::NEG_INFINITY),
        ("1_000", 1000.0),
        (" 2.5E2 ", 250.0),
        (".5", 0.5),
        ("7.", 7.0),
    ] {
        assert_eq!(t.parse::<Dd>().unwrap().to_f64(), v, "{t}");
    }
    assert!("nan".parse::<Dd>().unwrap().is_nan());
    // Expoentes extremos: estouro e underflow sem pânico.
    assert_eq!("1e400".parse::<Dd>().unwrap().to_f64(), f64::INFINITY);
    assert_eq!("1e-400".parse::<Dd>().unwrap().to_f64(), 0.0);
    // O menor subnormal (o literal 1e-330 seria 0 em f64: comparação exata).
    assert_eq!("4.9e-324".parse::<Dd>().unwrap().to_f64(), 5e-324);
    // Mais de 36 algarismos: os excedentes só mudam a escala.
    let longo: Dd = "1234567890123456789012345678901234567890".parse().unwrap();
    assert!((longo.to_f64() - 1.2345678901234568e39).abs() <= 1e24);
    let longo_frac: Dd = "0.1234567890123456789012345678901234567890123".parse().unwrap();
    assert!(
        (longo_frac - "0.12345678901234567890123456789012345678".parse::<Dd>().unwrap())
            .abs()
            .to_f64()
            < 1e-33
    );
    for t in ["", "abc", "1.2.3", "1e", "1ex", "--1", "."] {
        assert_eq!(
            t.parse::<Dd>().unwrap_err().id(),
            "chimalab:numeric:digitoInvalido",
            "{t:?}"
        );
    }
    // Funções nas bordas.
    // Mutação: primeiro algarismo logo abaixo de hi inteiro (2 − 1e-20 começa por 1, não 2/3).
    assert_eq!(
        Dd::from_parts(2.0, -1e-20).to_string_digits(22),
        "1.999999999999999999990e+00"
    );
    // Mutação: texto com 400 algarismos não estoura (só os 36 primeiros significativos contam).
    let muitos = format!("0.{}", "1234567890".repeat(40));
    let v: Dd = muitos.parse().unwrap();
    assert!(
        (v - "0.123456789012345678901234567890123456789".parse::<Dd>().unwrap())
            .abs()
            .to_f64()
            < 1e-33
    );
    let inteiro_longo: Dd = "1234567890".repeat(30).parse().unwrap();
    assert!((inteiro_longo.to_f64() / 1.2345678901234568e299 - 1.0).abs() < 1e-15);
    assert!(Dd::from_f64(-1.0).sqrt().is_nan());
    assert_eq!(Dd::from_f64(0.0).sqrt().to_f64(), 0.0);
    assert_eq!(Dd::from_f64(f64::INFINITY).sqrt().to_f64(), f64::INFINITY);
    assert!(Dd::from_f64(-1.0).ln().is_nan());
    assert!(Dd::from_f64(f64::NAN).ln().is_nan());
    assert_eq!(Dd::from_f64(0.0).ln().to_f64(), f64::NEG_INFINITY);
    assert_eq!(Dd::from_f64(f64::INFINITY).ln().to_f64(), f64::INFINITY);
    assert_eq!(Dd::from_f64(710.0).exp().to_f64(), f64::INFINITY);
    assert_eq!(Dd::from_f64(-746.0).exp().to_f64(), 0.0);
    assert!(Dd::from_f64(f64::NAN).exp().is_nan());
    assert!(!Dd::PI.is_nan() && !Dd::from_f64(f64::INFINITY).is_nan());
    // Limiares de exp (mutação): ln(MAX) = 709,7827…; 709,78 ainda é finito e 709,8 estoura
    // (o atalho acima de 709,8 só antecipa o estouro, então > e ≥ ali são equivalentes).
    // ±∞ vão a ∞ e 0 pelos atalhos.
    let alto = Dd::from_f64(709.78).exp().to_f64();
    assert!(
        alto.is_finite() && (alto / 709.78f64.exp() - 1.0).abs() <= 4.0 * f64::EPSILON,
        "{alto}"
    );
    assert_eq!(Dd::from_f64(709.8).exp().to_f64(), f64::INFINITY);
    // Perto do menor subnormal, igual ao exp do f64: e^−745,1 ≈ 2,55e-324 arredonda para 5e-324;
    // e^−745,2 ≈ 2,3e-324 vai a 0 (abaixo da metade do menor subnormal). Por isso o atalho em
    // −745,2 também só antecipa o resultado (< e ≤ ali são equivalentes).
    for x in [-745.1f64, -745.2, -744.0, -740.0] {
        assert_eq!(Dd::from_f64(x).exp().to_f64(), x.exp(), "{x}");
    }
    assert_eq!(Dd::from_f64(f64::INFINITY).exp().to_f64(), f64::INFINITY);
    assert_eq!(Dd::from_f64(f64::NEG_INFINITY).exp().to_f64(), 0.0);
    assert_eq!(Dd::from_f64(0.0).exp().to_f64(), 1.0);
    // exp perto do limite inferior: resultado subnormal, sem NaN.
    let sub = Dd::from_f64(-740.0).exp().to_f64();
    assert!(sub > 0.0 && sub < 1e-320 && (sub / (-740f64).exp() - 1.0).abs() < 1e-3);
    assert_eq!(Dd::from_f64(3.0).powi(0).to_f64(), 1.0);
    assert_eq!(Dd::from_f64(2.0).powi(-3).to_f64(), 0.125);
    // Operadores com ±∞ e divisão por zero.
    let inf = Dd::from_f64(f64::INFINITY);
    assert_eq!((inf + Dd::from_f64(1.0)).to_f64(), f64::INFINITY);
    assert_eq!((inf * Dd::from_f64(2.0)).to_f64(), f64::INFINITY);
    assert_eq!((Dd::from_f64(1.0) / Dd::from_f64(0.0)).to_f64(), f64::INFINITY);
    assert_eq!((Dd::from_f64(1.0) / inf).to_f64(), 0.0);
    assert!((Dd::from_f64(1.0) * Dd::from_f64(f64::NAN)).is_nan());
    assert!(!inf.is_finite() && Dd::PI.is_finite());
    // Ordem, valor absoluto, ldexp, conversões.
    let a = Dd::from_parts(1.0, 1e-20);
    let b = Dd::from_parts(1.0, -1e-20);
    assert!(b < a && a > Dd::from_f64(1.0) && Dd::from_f64(1.0) > b && Dd::PI > Dd::E);
    assert_eq!(a.partial_cmp(&Dd::from_f64(f64::NAN)), None);
    // from_parts normaliza: (0, −1e-300) vira hi = −1e-300.
    assert_eq!(Dd::from_parts(0.0, -1e-300).abs().hi(), 1e-300);
    assert!(Dd::from_f64(-0.0).abs().hi().is_sign_positive());
    assert_eq!((-a).abs(), a);
    assert_eq!(
        Dd::from_f64(3.0).ldexp(-1100).to_f64(),
        3.0 * 2f64.powi(-1100 + 60) * 2f64.powi(-60)
    );
    assert_eq!(Dd::from(2.5).hi(), 2.5);
    assert_eq!(Dd::default().to_f64(), 0.0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Identidades em dupla-dupla: (a + b) − b, (a·b)/b, (√a)², exp(ln a), e ida e volta pelo
    /// texto com 34 algarismos.
    #[test]
    fn dupla_dupla_identidades(ah in -1e10f64..1e10, al in -0.5f64..0.5, bh in 1e-3f64..1e3, ex in -50i32..50) {
        let a = Dd::from_parts(ah, al * f64::EPSILON * ah.abs());
        let b = Dd::from_f64(bh).ldexp(ex);
        let rel = |x: Dd, y: Dd| (x - y).abs().to_f64() / y.abs().to_f64().max(f64::MIN_POSITIVE);
        prop_assume!(ah != 0.0);
        // (a + b) − b perde só o que a soma arredondou: erro relativo a max(|a|, |b|).
        let m = a.abs().to_f64().max(b.abs().to_f64());
        prop_assert!(((a + b) - b - a).abs().to_f64() <= 4.0 * Dd::EPSILON * m);
        prop_assert!(rel(a * b / b, a) <= 4.0 * Dd::EPSILON);
        let p = a.abs();
        prop_assert!(rel(p.sqrt() * p.sqrt(), p) <= 4.0 * Dd::EPSILON);
        prop_assert!(rel(p.ln().exp(), p) <= 64.0 * Dd::EPSILON * p.ln().abs().to_f64().max(1.0));
        let t = a.to_string_digits(34);
        prop_assert!(rel(t.parse::<Dd>().unwrap(), a) <= 8.0 * Dd::EPSILON, "{}", t);
    }

    /// Ida e volta pelo texto logo acima e logo abaixo de potências de 10 (hi = 10ᵏ com lo de
    /// qualquer sinal): o caso em que o piso e o expoente precisam do valor inteiro.
    #[test]
    fn dupla_dupla_texto_perto_de_potencias_de_10(k in -30i32..30, f in -1.0f64..1.0) {
        let p = Dd::from_f64(10.0).powi(k);
        let v = Dd::from_parts(p.hi(), f * 1e-20 * p.hi());
        let t = v.to_string_digits(34);
        let w: Dd = t.parse().unwrap();
        prop_assert!((w - v).abs().to_f64() <= 8.0 * Dd::EPSILON * v.to_f64(), "{} {}", t, k);
        // O primeiro algarismo é 9 abaixo da potência e 1 acima.
        let primeiro = t.chars().next().unwrap();
        // Comparado com 10ᵏ em dupla-dupla (p.hi() não é 10ᵏ exato para k < 0).
        prop_assert!(primeiro == if v < p { '9' } else { '1' }, "{}", t);
    }
}
