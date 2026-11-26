//! Analisador léxico (DT-36): tokens, posições, indentação, contexto de `'`, textos, f-strings,
//! comentários e erros com identificador estável.

use chimalab_lang::lexico::{FPart, LexErrorKind, Op, Tok, incomplete, tokenize};
use proptest::prelude::*;

fn toks(src: &str) -> Vec<Tok> {
    tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
}

fn id(s: &str) -> Tok {
    Tok::Ident(s.to_string())
}

fn num(v: f64) -> Tok {
    Tok::Number {
        value: v,
        text: format!("{v}"),
    }
}

fn op(o: Op) -> Tok {
    Tok::Op(o)
}

fn erro(src: &str) -> (LexErrorKind, usize, usize, &'static str) {
    let e = tokenize(src).unwrap_err();
    (e.kind.clone(), e.line, e.col, e.id())
}

fn sem_fim(v: Vec<Tok>) -> Vec<Tok> {
    v.into_iter()
        .filter(|t| !matches!(t, Tok::Newline | Tok::Eof))
        .collect()
}

#[test]
fn numeros() {
    let n = |src: &str| match &toks(src)[0] {
        Tok::Number { value, .. } => *value,
        t => panic!("{src}: {t:?}"),
    };
    assert_eq!(n("42"), 42.0);
    assert_eq!(n("2.75"), 2.75);
    assert_eq!(n(".5"), 0.5);
    assert_eq!(n("1."), 1.0);
    assert_eq!(n("1e3"), 1000.0);
    assert_eq!(n("2.5E-3"), 0.0025);
    assert_eq!(n("1e+2"), 100.0);
    assert_eq!(n("1_000_000"), 1e6);
    assert_eq!(n("0x1F"), 31.0);
    assert_eq!(n("0b101"), 5.0);
    assert_eq!(n("0o17"), 15.0);
    assert_eq!(n("0xFF_FF"), 65535.0);
    // Inteiro acima de 2⁵³: a grafia original fica guardada.
    match &toks("9007199254740993")[0] {
        Tok::Number { text, .. } => assert_eq!(text, "9007199254740993"),
        t => panic!("{t:?}"),
    }
    // Imaginários.
    for (src, v) in [("2i", 2.0), ("3.5j", 3.5), ("1e2i", 100.0)] {
        match &toks(src)[0] {
            Tok::Imag { value, .. } => assert_eq!(*value, v, "{src}"),
            t => panic!("{src}: {t:?}"),
        }
    }
    // O ponto pertence ao operador elemento a elemento.
    assert_eq!(sem_fim(toks("1.*2")), vec![num(1.0), op(Op::DotStar), num(2.0)]);
    assert_eq!(sem_fim(toks("2.^x")), vec![num(2.0), op(Op::DotCaret), id("x")]);
    assert_eq!(sem_fim(toks("3.'")), vec![num(3.0), op(Op::DotQuote)]);
    // `e` sem dígitos é outro token; número colado a letras é erro.
    assert_eq!(sem_fim(toks("2 e")), vec![num(2.0), id("e")]);
    for src in ["2x", "0x", "0b2", "3if"] {
        assert_eq!(erro(src).3, "chimalab:lexico:numeroInvalido", "{src}");
    }
}

#[test]
fn identificadores_e_operadores() {
    assert_eq!(
        sem_fim(toks("x_1 = α + β2")),
        vec![id("x_1"), op(Op::Assign), id("α"), op(Op::Plus), id("β2")]
    );
    let todos = "+ - * / \\ ^ .* ./ .\\ .^ = == != ~= < <= > >= : , ; . @ & && | || ! ~ += -= *= /= ->";
    let esperado = [
        Op::Plus,
        Op::Minus,
        Op::Star,
        Op::Slash,
        Op::Backslash,
        Op::Caret,
        Op::DotStar,
        Op::DotSlash,
        Op::DotBackslash,
        Op::DotCaret,
        Op::Assign,
        Op::Eq,
        Op::Ne,
        Op::TildeEq,
        Op::Lt,
        Op::Le,
        Op::Gt,
        Op::Ge,
        Op::Colon,
        Op::Comma,
        Op::Semicolon,
        Op::Dot,
        Op::At,
        Op::Amp,
        Op::AmpAmp,
        Op::Pipe,
        Op::PipePipe,
        Op::Bang,
        Op::Tilde,
        Op::PlusAssign,
        Op::MinusAssign,
        Op::StarAssign,
        Op::SlashAssign,
        Op::Arrow,
    ];
    let t = sem_fim(toks(todos));
    assert_eq!(t, esperado.iter().map(|&o| op(o)).collect::<Vec<_>>());
    // A grafia de cada operador lexa para ele mesmo.
    for &o in &esperado {
        assert_eq!(sem_fim(toks(o.text())), vec![op(o)], "{}", o.text());
    }
    assert_eq!(erro("x $ y").0, LexErrorKind::InvalidChar('$'));
    assert_eq!(erro("x $ y").3, "chimalab:lexico:caractereInvalido");
    // Acesso a membro e transpostas por nome.
    assert_eq!(sem_fim(toks("A.T")), vec![id("A"), op(Op::Dot), id("T")]);
    // `->` colado (DT-39); com espaço no meio são dois operadores.
    assert_eq!(sem_fim(toks("x->1")), vec![id("x"), op(Op::Arrow), num(1.0)]);
    assert_eq!(
        sem_fim(toks("x - > 1")),
        vec![id("x"), op(Op::Minus), op(Op::Gt), num(1.0)]
    );
    assert_eq!(
        sem_fim(toks("x-=1")),
        vec![id("x"), op(Op::MinusAssign), num(1.0)]
    );
}

#[test]
fn aspa_transposta_ou_texto() {
    assert_eq!(sem_fim(toks("a'")), vec![id("a"), op(Op::Quote)]);
    assert_eq!(sem_fim(toks("a''")), vec![id("a"), op(Op::Quote), op(Op::Quote)]);
    assert_eq!(
        sem_fim(toks("f(x)'")),
        vec![id("f"), op(Op::LParen), id("x"), op(Op::RParen), op(Op::Quote)]
    );
    assert_eq!(
        sem_fim(toks("A[1]'")),
        vec![
            id("A"),
            op(Op::LBracket),
            num(1.0),
            op(Op::RBracket),
            op(Op::Quote)
        ]
    );
    assert_eq!(sem_fim(toks("a.'")), vec![id("a"), op(Op::DotQuote)]);
    assert_eq!(sem_fim(toks("2'")), vec![num(2.0), op(Op::Quote)]);
    // Depois de operador ou no início: texto.
    assert_eq!(
        sem_fim(toks("x = 'texto'")),
        vec![id("x"), op(Op::Assign), Tok::Str("texto".into())]
    );
    assert_eq!(
        sem_fim(toks("'a', 'b'")),
        vec![Tok::Str("a".into()), op(Op::Comma), Tok::Str("b".into())]
    );
    // Fora de colchetes, um texto também termina um valor: 'a' 'b' é a transposta de 'a'
    // seguida de b e outra transposta (a regra do DT-34 não olha o espaço fora de colchetes).
    assert_eq!(
        sem_fim(toks("'a' 'b'")),
        vec![Tok::Str("a".into()), op(Op::Quote), id("b"), op(Op::Quote)]
    );
    // Fora de colchetes, o espaço não muda: a ' é transposta depois de um valor.
    assert_eq!(sem_fim(toks("a '")), vec![id("a"), op(Op::Quote)]);
    // Dentro de colchetes, espaço antes de ' abre texto; colado, é transposta.
    assert_eq!(
        sem_fim(toks("[a' b']")),
        vec![
            op(Op::LBracket),
            id("a"),
            op(Op::Quote),
            id("b"),
            op(Op::Quote),
            op(Op::RBracket)
        ]
    );
    assert_eq!(
        sem_fim(toks("[a 'b']")),
        vec![op(Op::LBracket), id("a"), Tok::Str("b".into()), op(Op::RBracket)]
    );
    // `end` termina um valor (A[end]').
    assert_eq!(
        sem_fim(toks("A[end]'")),
        vec![
            id("A"),
            op(Op::LBracket),
            id("end"),
            op(Op::RBracket),
            op(Op::Quote)
        ]
    );
}

#[test]
fn textos_e_escapes() {
    let s = |src: &str| match &toks(src)[0] {
        Tok::Str(s) => s.clone(),
        t => panic!("{src}: {t:?}"),
    };
    assert_eq!(s(r#""dupla""#), "dupla");
    assert_eq!(s("'simples'"), "simples");
    assert_eq!(s(r#""a\"b""#), "a\"b");
    assert_eq!(s(r"'it\'s'"), "it's");
    assert_eq!(s(r#""l1\nl2\t\\\0\r""#), "l1\nl2\t\\\0\r");
    assert_eq!(s(r#""\u{3b1}\u{1F600}""#), "α😀");
    assert_eq!(s(r#""\u{10FFFF}""#), "\u{10FFFF}");
    assert_eq!(erro(r#""\u{0000041}""#).3, "chimalab:lexico:escapeInvalido");
    assert_eq!(s(r#""aspas 'dentro'""#), "aspas 'dentro'");
    // Crus: a barra fica, inclusive antes da aspa.
    assert_eq!(s(r#"r"C:\temp\nova""#), r"C:\temp\nova");
    assert_eq!(s(r#"r"a\"b""#), r#"a\"b"#);
    assert_eq!(s(r"r'\\'"), r"\\");
    // Erros com posição da aspa de abertura.
    assert_eq!(
        erro("x = 'aberto"),
        (
            LexErrorKind::UnterminatedString,
            1,
            5,
            "chimalab:lexico:textoNaoTerminado"
        )
    );
    assert_eq!(erro("'quebra\nlinha'").0, LexErrorKind::UnterminatedString);
    assert_eq!(erro(r#""\q""#).3, "chimalab:lexico:escapeInvalido");
    assert_eq!(erro(r#""\u{110000}""#).3, "chimalab:lexico:escapeInvalido");
    assert_eq!(erro(r#""\u{}""#).3, "chimalab:lexico:escapeInvalido");
    assert_eq!(erro(r#""\u3b1""#).3, "chimalab:lexico:escapeInvalido");
}

#[test]
fn fstrings() {
    let f = |src: &str| match &toks(src)[0] {
        Tok::FStr(p) => p.clone(),
        t => panic!("{src}: {t:?}"),
    };
    let ex = |src: &str, fmt: Option<&str>, line: usize, col: usize| FPart::Expr {
        src: src.into(),
        format: fmt.map(String::from),
        line,
        col,
    };
    assert_eq!(
        f(r#"f"x = {x}""#),
        vec![FPart::Lit("x = ".into()), ex("x", None, 1, 8)]
    );
    assert_eq!(
        f(r#"f"{x:.3f} e {y}!""#),
        vec![
            ex("x", Some(".3f"), 1, 4),
            FPart::Lit(" e ".into()),
            ex("y", None, 1, 14),
            FPart::Lit("!".into())
        ]
    );
    // Dois-pontos dentro de colchetes não são formato; texto com chave dentro da expressão.
    assert_eq!(f(r#"f"{A[1:2]}""#), vec![ex("A[1:2]", None, 1, 4)]);
    assert_eq!(f(r#"f"{d['k}']}""#), vec![ex("d['k}']", None, 1, 4)]);
    // Transposta dentro da expressão (regressão): f"{A'}" e f"{x' * 'a'}".
    assert_eq!(f(r#"f"{A'}""#), vec![ex("A'", None, 1, 4)]);
    assert_eq!(f(r#"f"{x' * 'a'}""#), vec![ex("x' * 'a'", None, 1, 4)]);
    assert_eq!(
        f(r#"f"{{lit}} {n}""#),
        vec![FPart::Lit("{lit} ".into()), ex("n", None, 1, 12)]
    );
    assert_eq!(
        f(r#"f'\t{a}'"#),
        vec![FPart::Lit("\t".into()), ex("a", None, 1, 6)]
    );
    assert_eq!(
        f(r#"rf"\d{a}""#),
        vec![FPart::Lit("\\d".into()), ex("a", None, 1, 7)]
    );
    assert_eq!(f(r#"f"""#), vec![]);
    // `}` dentro de texto e de chaves aninhadas não fecha a expressão.
    assert_eq!(f(r#"f"{'a}b'}""#), vec![ex("'a}b'", None, 1, 4)]);
    assert_eq!(f(r#"f"{g({x})}""#), vec![ex("g({x})", None, 1, 4)]);
    for src in [r#"f"{x""#, r#"f"{}""#, r#"f"x}""#, r#"f"{ }""#, r#"f"{x"y""#] {
        assert_eq!(erro(src).3, "chimalab:lexico:fstringInvalida", "{src}");
    }
    assert_eq!(erro(r#"f"aberta"#).3, "chimalab:lexico:textoNaoTerminado");
    // `f` sem aspa colada é identificador.
    assert_eq!(
        sem_fim(toks("f (x)")),
        vec![id("f"), op(Op::LParen), id("x"), op(Op::RParen)]
    );
}

#[test]
fn comentarios() {
    assert_eq!(
        sem_fim(toks("x = 1 # comentário 'com aspa'")),
        vec![id("x"), op(Op::Assign), num(1.0)]
    );
    assert_eq!(sem_fim(toks("#= bloco #= aninhado =# ainda =# y")), vec![id("y")]);
    assert_eq!(sem_fim(toks("a #= várias\nlinhas =# b")), vec![id("a"), id("b")]);
    assert_eq!(
        erro("x #= #= =#"),
        (
            LexErrorKind::UnterminatedComment,
            1,
            3,
            "chimalab:lexico:comentarioNaoTerminado"
        )
    );
    // Comentário conta como espaço (dentro de colchetes separa elementos).
    let t = tokenize("[1#=c=#2]").unwrap();
    assert!(t[2].space_before);
}

#[test]
fn indentacao_em_blocos() {
    let src = "def f(x):\n    if x:\n        y = 1\n\n    # comentário\n    return y\nz = 2\n";
    let t = toks(src);
    let esperado = vec![
        id("def"),
        id("f"),
        op(Op::LParen),
        id("x"),
        op(Op::RParen),
        op(Op::Colon),
        Tok::Newline,
        Tok::Indent,
        id("if"),
        id("x"),
        op(Op::Colon),
        Tok::Newline,
        Tok::Indent,
        id("y"),
        op(Op::Assign),
        num(1.0),
        Tok::Newline,
        Tok::Dedent,
        id("return"),
        id("y"),
        Tok::Newline,
        Tok::Dedent,
        id("z"),
        op(Op::Assign),
        num(2.0),
        Tok::Newline,
        Tok::Eof,
    ];
    assert_eq!(t, esperado);
    // Blocos abertos no fim são fechados; tabulação vale até o próximo múltiplo de 4.
    let t = toks("if a:\n\tb\n");
    assert_eq!(t[t.len() - 2..], [Tok::Dedent, Tok::Eof]);
    assert_eq!(toks("if a:\n\tb\n    c"), toks("if a:\n    b\n    c"));
    assert_eq!(toks("if a:\n  \tb\n    c"), toks("if a:\n    b\n    c"));
    assert_eq!(
        toks("if a:\n     \tb\n        c"),
        toks("if a:\n        b\n        c")
    );
    // Recuo para coluna que não abriu bloco.
    assert_eq!(
        erro("if a:\n    b\n  c"),
        (
            LexErrorKind::InconsistentIndent,
            3,
            3,
            "chimalab:lexico:indentacaoInconsistente"
        )
    );
    // Dentro de parênteses e chaves, a quebra de linha não conta; dentro de colchetes separa linhas.
    assert_eq!(
        sem_fim(toks("f(1,\n      2)")),
        vec![
            id("f"),
            op(Op::LParen),
            num(1.0),
            op(Op::Comma),
            num(2.0),
            op(Op::RParen)
        ]
    );
    assert_eq!(
        toks("[1 2\n 3 4]"),
        vec![
            op(Op::LBracket),
            num(1.0),
            num(2.0),
            Tok::Newline,
            num(3.0),
            num(4.0),
            op(Op::RBracket),
            Tok::Newline,
            Tok::Eof
        ]
    );
    assert_eq!(
        sem_fim(toks("{a,\n b}")),
        vec![op(Op::LBrace), id("a"), op(Op::Comma), id("b"), op(Op::RBrace)]
    );
    // Linhas só com espaço não geram Newline; texto vazio só tem Eof.
    assert_eq!(toks(""), vec![Tok::Eof]);
    assert_eq!(toks("   \n\n  # só comentário\n"), vec![Tok::Eof]);
    assert_eq!(
        toks("a\r\nb"),
        vec![id("a"), Tok::Newline, id("b"), Tok::Newline, Tok::Eof]
    );
}

#[test]
fn delimitadores() {
    assert_eq!(
        erro("f(1]"),
        (
            LexErrorKind::Unbalanced,
            1,
            4,
            "chimalab:lexico:delimitadorDesbalanceado"
        )
    );
    assert_eq!(erro("x)").3, "chimalab:lexico:delimitadorDesbalanceado");
    assert_eq!(
        erro("a = [1,\n(2"),
        (
            LexErrorKind::Unclosed,
            2,
            1,
            "chimalab:lexico:delimitadorNaoFechado"
        )
    );
}

#[test]
fn espaco_antes_e_posicoes() {
    let t = tokenize("[1 -2]").unwrap();
    assert!(t[2].space_before && !t[3].space_before, "{t:?}");
    let t = tokenize("[1 - 2]").unwrap();
    assert!(t[2].space_before && t[3].space_before);
    // Linha, coluna (em caracteres) e intervalo em bytes.
    let t = tokenize("α = 10\nβ").unwrap();
    assert_eq!((t[0].line, t[0].col, t[0].span), (1, 1, (0, 2)));
    assert_eq!((t[2].line, t[2].col, t[2].span), (1, 5, (5, 7)));
    assert_eq!((t[4].line, t[4].col), (2, 1));
    // Mensagem com posição.
    let e = tokenize("x\n  y = 'a").unwrap_err();
    assert_eq!(e.to_string(), "linha 2, coluna 7: texto sem aspa de fechamento");
}

#[test]
fn entrada_incompleta_no_console() {
    for src in [
        "f(1,",
        "[1 2;",
        "x = #= começo",
        "if x:",
        "def f(x):\n    return x",
    ] {
        assert!(incomplete(src), "{src:?}");
    }
    for src in ["1 + 1", "f(1)", "if x:\n    y\n\n", "x = 'aberto", "f(1]", ""] {
        assert!(!incomplete(src), "{src:?}");
    }
}

proptest! {
    /// O léxico nunca entra em pânico: qualquer texto dá tokens ou um erro com posição válida.
    #[test]
    fn nunca_entra_em_panico(src in "\\PC{0,60}") {
        match tokenize(&src) {
            Ok(t) => prop_assert_eq!(t.last().map(|x| &x.tok), Some(&Tok::Eof)),
            Err(e) => prop_assert!(e.line >= 1 && e.col >= 1),
        }
    }

    /// Ida e volta: sequências de identificadores, números e operadores separados por espaço
    /// voltam como os mesmos tokens.
    #[test]
    fn ida_e_volta(partes in prop::collection::vec(0usize..6, 1..20), sem in any::<u32>()) {
        let mut src = String::new();
        let mut esperado = Vec::new();
        for (k, &p) in partes.iter().enumerate() {
            let v = (sem as usize + 7 * k) % 1000;
            let (txt, tok) = match p {
                0 => (format!("v{v}"), id(&format!("v{v}"))),
                1 => (format!("{v}"), Tok::Number { value: v as f64, text: format!("{v}") }),
                2 => ("+".to_string(), op(Op::Plus)),
                3 => (".*".to_string(), op(Op::DotStar)),
                4 => ("==".to_string(), op(Op::Eq)),
                _ => (format!("\"t{v}\""), Tok::Str(format!("t{v}"))),
            };
            src.push_str(&txt);
            src.push(' ');
            esperado.push(tok);
        }
        prop_assert_eq!(sem_fim(toks(&src)), esperado);
    }
}

/// Regressão: depois de palavra reservada, `'` abre texto (antes virava transposta e
/// `for c in 'abc':` falhava).
#[test]
fn aspa_depois_de_palavra_reservada() {
    use chimalab_lang::lexico::KEYWORDS;
    for w in KEYWORDS {
        let t = sem_fim(toks(&format!("{w} 'x'")));
        assert_eq!(t, vec![id(w), Tok::Str("x".into())], "{w}");
        let t = sem_fim(toks(&format!("{w}'x'")));
        assert_eq!(t, vec![id(w), Tok::Str("x".into())], "{w} colado");
    }
    // Nomes comuns (inclusive `end` e prefixos de palavras) continuam transpondo.
    for w in ["end", "x", "iff", "in_", "case2"] {
        assert_eq!(sem_fim(toks(&format!("{w}'"))), vec![id(w), op(Op::Quote)], "{w}");
    }
}
