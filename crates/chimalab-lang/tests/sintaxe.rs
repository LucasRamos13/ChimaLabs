//! Expressões (DT-37): precedência, associatividade, pós-fixos, matrizes, índices, chamadas,
//! f-strings e erros com posição.

use chimalab_lang::sintaxe::{Expr, parse_expr};
use proptest::prelude::*;

fn s(src: &str) -> String {
    parse_expr(src)
        .unwrap_or_else(|e| panic!("{src}: {e}"))
        .to_string()
}

fn erro(src: &str) -> (&'static str, usize, usize) {
    let e = parse_expr(src).unwrap_err();
    (e.id(), e.line, e.col)
}

#[test]
fn precedencia_e_associatividade() {
    assert_eq!(s("1 + 2 * 3"), "(+ 1 (* 2 3))");
    assert_eq!(s("(1 + 2) * 3"), "(* (+ 1 2) 3)");
    assert_eq!(s("1 - 2 - 3"), "(- (- 1 2) 3)");
    assert_eq!(s("8 / 4 / 2"), "(/ (/ 8 4) 2)");
    assert_eq!(s("a .* b ./ c .\\ d \\ e"), "(\\ (.\\ (./ (.* a b) c) d) e)");
    // Potência à direita, acima do sinal; expoente com sinal.
    assert_eq!(s("2^3^2"), "(^ 2 (^ 3 2))");
    assert_eq!(s("-2^2"), "(neg (^ 2 2))");
    assert_eq!(s("2^-1"), "(^ 2 (neg 1))");
    assert_eq!(s("a.^b'"), "(.^ a (' b))");
    assert_eq!(s("--x"), "(neg (neg x))");
    assert_eq!(s("+x * -y"), "(* (pos x) (neg y))");
    // Intervalo abaixo dos aritméticos e acima da comparação.
    assert_eq!(s("1:n+1"), "(: 1 (+ n 1))");
    assert_eq!(s("0:0.1:1"), "(: 0 0.1 1)");
    assert_eq!(s("a:b == c"), "(== (: a b) c)");
    assert_eq!(s("x + 1 >= 2 * y"), "(>= (+ x 1) (* 2 y))");
    assert_eq!(s("a != b"), "(!= a b)");
    for (src, esp) in [
        ("a > b", "(> a b)"),
        ("a < b", "(< a b)"),
        ("a <= b", "(<= a b)"),
        ("a >= b", "(>= a b)"),
        ("a == b", "(== a b)"),
    ] {
        assert_eq!(s(src), esp);
    }
}

#[test]
fn posfixos() {
    assert_eq!(s("A'"), "(' A)");
    assert_eq!(s("A.'"), "(.' A)");
    assert_eq!(s("A''"), "(' (' A))");
    assert_eq!(s("A.T"), "(. A T)");
    assert_eq!(s("x.campo.sub"), "(. (. x campo) sub)");
    assert_eq!(s("f(x)[2]'"), "(' (idx (call f x) 2))");
    assert_eq!(s("A[i, j] * B"), "(* (idx A i j) B)");
    assert_eq!(s("f()"), "(call f)");
    assert_eq!(s("2i * z"), "(* 2i z)");
    assert_eq!(s("\"texto\""), "\"texto\"");
}

#[test]
fn indices_com_end_e_dois_pontos() {
    assert_eq!(s("A[end]"), "(idx A end)");
    assert_eq!(s("A[end-1, :]"), "(idx A (- end 1) :)");
    assert_eq!(s("A[:, 2:end]"), "(idx A : (: 2 end))");
    assert_eq!(s("A[B[end]]"), "(idx A (idx B end))");
    // `end` dentro de chamada dentro de índice continua valendo (A[f(end)]).
    assert_eq!(s("A[min(end, 3)]"), "(idx A (call min end 3))");
    assert_eq!(erro("end + 1"), ("chimalab:sintaxe:endForaDeIndice", 1, 1));
    assert_eq!(erro("f(end)").0, "chimalab:sintaxe:endForaDeIndice");
}

#[test]
fn matrizes() {
    assert_eq!(s("[1 2; 3 4]"), "[1 2;3 4]");
    assert_eq!(s("[1, 2; 3, 4]"), "[1 2;3 4]");
    assert_eq!(s("[1 2\n 3 4]"), "[1 2;3 4]");
    assert_eq!(s("[]"), "[]");
    assert_eq!(s("[1;]"), "[1]");
    // Espaço separa elementos; sinal colado depois de espaço é outro elemento.
    assert_eq!(s("[1 -2]"), "[1 (neg 2)]");
    assert_eq!(s("[1 - 2]"), "[(- 1 2)]");
    assert_eq!(s("[1 -2 +3]"), "[1 (neg 2) (pos 3)]");
    assert_eq!(s("[a' b']"), "[(' a) (' b)]");
    assert_eq!(s("[a (1)]"), "[a 1]");
    assert_eq!(s("[a(1)]"), "[(call a 1)]");
    assert_eq!(s("[A [1]]"), "[A [1]]");
    assert_eq!(s("[x[1] 2]"), "[(idx x 1) 2]");
    assert_eq!(s("[1:3 5]"), "[(: 1 3) 5]");
    // Dentro de parênteses, o espaço volta a não importar.
    assert_eq!(s("[(1 -2) 3]"), "[(- 1 2) 3]");
    // Forma aninhada com vírgulas: linhas (DT-34); com espaços, concatenação horizontal.
    assert_eq!(s("[[1, 2], [3, 4]]"), "[[1 2];[3 4]]");
    assert_eq!(s("[[1 2] [3 4]]"), "[[1 2] [3 4]]");
    assert_eq!(s("[[1, 2], x]"), "[[1 2] x]");
    assert_eq!(erro("[1 2").0, "chimalab:lexico:delimitadorNaoFechado");
}

#[test]
fn chamadas_com_nomeados() {
    assert_eq!(s("f(x, y=2)"), "(call f x y=2)");
    assert_eq!(s("range(0, 1; length=5)"), "(call range 0 1 length=5)");
    assert_eq!(s("g(a == b)"), "(call g (== a b))");
    assert_eq!(
        erro("f(y=2, x)"),
        ("chimalab:sintaxe:argumentoPosicionalDepoisDeNomeado", 1, 8)
    );
    assert_eq!(
        erro("range(0; 1)").0,
        "chimalab:sintaxe:argumentoPosicionalDepoisDeNomeado"
    );
    assert_eq!(erro("f(1 2)").0, "chimalab:sintaxe:tokenInesperado");
}

#[test]
fn fstrings_com_expressoes() {
    assert_eq!(s(r#"f"x = {x + 1:.3f}!""#), r#"(fstr "x = " {(+ x 1):.3f} "!")"#);
    assert_eq!(s(r#"f"{A[end]}""#), r#"(fstr {(idx A end)})"#);
    // A posição de um erro dentro da expressão aponta para o texto original.
    assert_eq!(
        erro(r#"y = 1; f"ab {x + }""#).0,
        "chimalab:sintaxe:tokenInesperado"
    );
    assert_eq!(erro(r#"f"ab {x + }""#), ("chimalab:sintaxe:fimInesperado", 1, 11));
    match &parse_expr(r#"f"{x}""#).unwrap().expr {
        Expr::FStr(segs) => match &segs[0] {
            chimalab_lang::sintaxe::FSeg::Expr(n, _) => assert_eq!((n.line, n.col), (1, 4)),
            o => panic!("{o:?}"),
        },
        o => panic!("{o:?}"),
    } // Nós aninhados também ficam na posição do texto original (f"{a + b[c]:x}").
    let n = parse_expr(r#"f"{a + b[c]:x}""#).unwrap();
    let Expr::FStr(segs) = &n.expr else { panic!() };
    let chimalab_lang::sintaxe::FSeg::Expr(e, _) = &segs[0] else {
        panic!()
    };
    let Expr::Binary(_, a, b) = &e.expr else {
        panic!("{e}")
    };
    assert_eq!(((a.line, a.col), (b.line, b.col)), ((1, 4), (1, 8)));
    let Expr::Index(_, idx) = &b.expr else { panic!() };
    assert_eq!((idx[0].line, idx[0].col), (1, 10));
}

#[test]
fn erros_com_posicao() {
    assert_eq!(erro("a < b < c"), ("chimalab:sintaxe:comparacaoEncadeada", 1, 7));
    // Pergunta 44.A: `~=` é erro que sugere `!=`.
    assert_eq!(erro("a ~= b"), ("chimalab:sintaxe:diferenteComTil", 1, 3));
    assert_eq!(
        parse_expr("a ~= b").unwrap_err().to_string(),
        "linha 1, coluna 3: o diferente se escreve `!=`"
    );
    assert_eq!(erro("1 +"), ("chimalab:sintaxe:fimInesperado", 1, 4));
    assert_eq!(erro("1 + * 2"), ("chimalab:sintaxe:tokenInesperado", 1, 5));
    assert_eq!(erro("(1 + 2"), ("chimalab:lexico:delimitadorNaoFechado", 1, 1));
    assert_eq!(erro("x y"), ("chimalab:sintaxe:tokenInesperado", 1, 3));
    assert_eq!(erro("a.1").0, "chimalab:sintaxe:tokenInesperado");
    assert_eq!(erro("2x").0, "chimalab:lexico:numeroInvalido");
    let e = parse_expr("1 + * 2").unwrap_err();
    assert_eq!(
        e.to_string(),
        "linha 1, coluna 5: esperava uma expressão, veio `*`"
    );
    // Posições dos nós: a do primeiro token (ou do parêntese).
    let n = parse_expr("  (a + b) * c").unwrap();
    assert_eq!((n.line, n.col), (1, 3));
}

/// Árvore aleatória escrita com parênteses completos: a análise devolve a mesma árvore.
#[derive(Clone, Debug)]
enum Arv {
    Folha(u8),
    Un(bool, Box<Arv>),
    Bin(usize, Box<Arv>, Box<Arv>),
}

const OPS: [&str; 10] = ["+", "-", "*", "/", ".*", "./", "^", ".^", "==", "<="];

fn arv() -> impl Strategy<Value = Arv> {
    let folha = (0u8..26).prop_map(Arv::Folha);
    folha.prop_recursive(5, 40, 2, |inner| {
        prop_oneof![
            (any::<bool>(), inner.clone()).prop_map(|(n, a)| Arv::Un(n, Box::new(a))),
            (0..OPS.len(), inner.clone(), inner).prop_map(|(o, a, b)| Arv::Bin(o, Box::new(a), Box::new(b))),
        ]
    })
}

fn fonte(a: &Arv) -> String {
    match a {
        Arv::Folha(k) => format!("v{k}"),
        Arv::Un(neg, x) => format!("({}{})", if *neg { "-" } else { "+" }, fonte(x)),
        Arv::Bin(o, x, y) => format!("({} {} {})", fonte(x), OPS[*o], fonte(y)),
    }
}

fn canonica(a: &Arv) -> String {
    match a {
        Arv::Folha(k) => format!("v{k}"),
        Arv::Un(neg, x) => format!("({} {})", if *neg { "neg" } else { "pos" }, canonica(x)),
        Arv::Bin(o, x, y) => format!("({} {} {})", OPS[*o], canonica(x), canonica(y)),
    }
}

proptest! {
    #[test]
    fn ida_e_volta_parentizada(a in arv()) {
        prop_assert_eq!(parse_expr(&fonte(&a)).unwrap().to_string(), canonica(&a));
    }

    /// O parser nunca entra em pânico.
    #[test]
    fn nunca_entra_em_panico(src in "[-+*/^ .'a-c0-9()\\[\\],;:=<>!e\"]{0,40}") {
        let _ = parse_expr(&src);
    }
}

/// Todos os tipos de nó dentro de uma f-string ficam com posição no texto original: a mesma
/// expressão analisada sozinha dá as colunas relativas, e na f-string cada nó tem de estar
/// exatamente deslocado (mutação: cada tipo de nó é reposicionado).
#[test]
fn fstring_reposiciona_todos_os_nos() {
    use chimalab_lang::sintaxe::{FSeg, Node};
    fn posicoes(n: &Node, out: &mut Vec<(usize, usize)>) {
        out.push((n.line, n.col));
        match &n.expr {
            Expr::Unary(_, a) | Expr::Transpose { expr: a, .. } | Expr::Member(a, _) => posicoes(a, out),
            Expr::Binary(_, a, b) => {
                posicoes(a, out);
                posicoes(b, out);
            }
            Expr::Range { start, step, stop } => {
                posicoes(start, out);
                if let Some(x) = step {
                    posicoes(x, out);
                }
                posicoes(stop, out);
            }
            Expr::Index(a, xs) => {
                posicoes(a, out);
                xs.iter().for_each(|x| posicoes(x, out));
            }
            Expr::Call(a, xs) => {
                posicoes(a, out);
                xs.iter().for_each(|x| posicoes(&x.value, out));
            }
            Expr::Matrix(l) => l.iter().flatten().for_each(|x| posicoes(x, out)),
            Expr::FStr(segs) => segs.iter().for_each(|s| {
                if let FSeg::Expr(x, _) = s {
                    posicoes(x, out);
                }
            }),
            _ => {}
        }
    }
    let inner = "-a' + (1:2:3)[f(x, y=z)] .* [p q; r] + g.h + f'{w}'";
    let src = format!("f\"..........{{{inner}}}\"");
    let n = parse_expr(&src).unwrap();
    let Expr::FStr(segs) = &n.expr else { panic!() };
    let FSeg::Expr(e, _) = &segs[1] else {
        panic!("{segs:?}")
    };
    let (mut dentro, mut sozinha) = (Vec::new(), Vec::new());
    posicoes(e, &mut dentro);
    posicoes(&parse_expr(inner).unwrap(), &mut sozinha);
    assert!(sozinha.len() >= 20, "{}", sozinha.len());
    let desloc = 13; // `f"` + 10 pontos + `{`
    let esperado: Vec<(usize, usize)> = sozinha.iter().map(|&(l, c)| (l, c + desloc)).collect();
    assert_eq!(dentro, esperado);
}

/// Exemplos da especificação (`docs/linguagem/`): cada um analisa como descrito.
#[test]
fn exemplos_da_especificacao() {
    assert_eq!(s("A'"), "(' A)");
    assert_eq!(s("[a 'b']"), "[a \"b\"]");
    assert_eq!(s("[1 2\n     3 4]"), "[1 2;3 4]");
    assert_eq!(s(r#"f"{(1:3)}""#), "(fstr {(: 1 3)})");
    assert_eq!(s(r#"f"{d['chave']}""#), r#"(fstr {(idx d "chave")})"#);
    assert_eq!(s("1:n+1"), "(: 1 (+ n 1))");
    assert_eq!(s("A[end-1, :]"), "(idx A (- end 1) :)");
    assert_eq!(s("range(0, 1; step=0.25)"), "(call range 0 1 step=0.25)");
    assert_eq!(erro("a < b < c").0, "chimalab:sintaxe:comparacaoEncadeada");
    assert_eq!(
        erro("if x > 0:\n    y = 1\n  z = 2").0,
        "chimalab:lexico:indentacaoInconsistente"
    );
}
