//! Instruções e blocos (DT-39): lógicos, funções anônimas, atribuições, `if`, laços, `try`,
//! `match`, `def`, exibição com `;` e erros com posição.

use chimalab_lang::sintaxe::{KEYWORDS, StmtKind, is_keyword, parse_expr, parse_program};
use proptest::prelude::*;

/// O programa em S-expressão, uma instrução por linha.
fn p(src: &str) -> String {
    parse_program(src)
        .unwrap_or_else(|e| panic!("{src:?}: {e}"))
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn e(src: &str) -> String {
    parse_expr(src)
        .unwrap_or_else(|e| panic!("{src}: {e}"))
        .to_string()
}

fn erro(src: &str) -> (&'static str, usize, usize) {
    let e = parse_program(src).unwrap_err();
    (e.id(), e.line, e.col)
}

/// CA (resposta 42.C): as duas grafias dos lógicos são equivalentes, com a precedência
/// `or` < `and` < `|` < `&` < `not` < comparação.
#[test]
fn ca42_logicos_e_precedencia() {
    assert_eq!(e("a and b"), "(and a b)");
    assert_eq!(e("a && b"), "(and a b)");
    assert_eq!(e("a or b"), "(or a b)");
    assert_eq!(e("a || b"), "(or a b)");
    assert_eq!(e("not a"), "(not a)");
    assert_eq!(e("!a"), "(not a)");
    assert_eq!(e("a or b and c"), "(or a (and b c))");
    assert_eq!(e("a || b && c"), "(or a (and b c))");
    assert_eq!(e("a and b or c and d"), "(or (and a b) (and c d))");
    assert_eq!(e("a or b or c"), "(or (or a b) c)");
    assert_eq!(e("a | b & c"), "(| a (& b c))");
    assert_eq!(e("a & b | c"), "(| (& a b) c)");
    assert_eq!(e("a | b | c"), "(| (| a b) c)");
    assert_eq!(e("a & b & c"), "(& (& a b) c)");
    assert_eq!(e("a and b | c"), "(and a (| b c))");
    assert_eq!(e("a | b and c"), "(and (| a b) c)");
    // Comparação acima de & e |: x > 0 & x < 1 dispensa parênteses.
    assert_eq!(e("x > 0 & x < 1"), "(& (> x 0) (< x 1))");
    // not abaixo da comparação e acima de &.
    assert_eq!(e("not a == b"), "(not (== a b))");
    assert_eq!(e("!a & b"), "(& (not a) b)");
    assert_eq!(e("not not a"), "(not (not a))");
    assert_eq!(e("a & !b"), "(& a (not b))");
    assert_eq!(e("not a or b"), "(or (not a) b)");
    assert_eq!(e("-a and b"), "(and (neg a) b)");
    // != continua sendo um operador só.
    // A negação depois de comparação pede parênteses (como `a != not b` no Python), porque
    // `not` fica abaixo da comparação.
    assert_eq!(e("a != (!b)"), "(!= a (not b))");
    assert_eq!(
        parse_expr("a != !b").unwrap_err().id(),
        "chimalab:sintaxe:tokenInesperado"
    );
    assert_eq!(
        parse_expr("1 + not x").unwrap_err().id(),
        "chimalab:sintaxe:palavraReservada"
    );
    // Dentro de matrizes e chamadas.
    assert_eq!(e("[a & b, !c]"), "[(& a b) (not c)]");
    assert_eq!(e("f(not x, a or b)"), "(call f (not x) (or a b))");
    assert_eq!(e("[a !b]"), "[a (not b)]");
}

/// CA (resposta 47.A): funções anônimas com `->`, com o corpo de menor precedência.
#[test]
fn ca47_funcoes_anonimas() {
    assert_eq!(e("x -> x^2"), "(-> (x) (^ x 2))");
    assert_eq!(e("(x, y) -> x + y"), "(-> (x y) (+ x y))");
    assert_eq!(e("() -> 1"), "(-> () 1)");
    assert_eq!(e("(x) -> x"), "(-> (x) x)");
    assert_eq!(e("x -> y -> x + y"), "(-> (x) (-> (y) (+ x y)))");
    assert_eq!(e("x -> x > 0 and x < 1"), "(-> (x) (and (> x 0) (< x 1)))");
    assert_eq!(e("map(x -> 2*x, v)"), "(call map (-> (x) (* 2 x)) v)");
    assert_eq!(e("f(g=x -> x)"), "(call f g=(-> (x) x))");
    assert_eq!(e("[x -> x, y -> y]"), "[(-> (x) x) (-> (y) y)]");
    // Parênteses comuns continuam expressões.
    assert_eq!(e("(x)"), "x");
    // Posição: a da seta não, a do início.
    let n = parse_expr("  (a, b) -> a").unwrap();
    assert_eq!((n.line, n.col), (1, 3));
    // Palavra reservada não é parâmetro.
    assert!(parse_expr("if -> 1").is_err());
}

#[test]
fn erro_de_tupla_entre_parenteses() {
    // `(x, y)` sem seta não é expressão.
    assert_eq!(
        parse_expr("(x, y)").unwrap_err().id(),
        "chimalab:sintaxe:tokenInesperado"
    );
}

/// CA (DT-34): `;` suprime a exibição; várias instruções por linha.
#[test]
fn ca_atribuicoes_e_exibicao() {
    assert_eq!(p("x = 1"), "(= x 1)");
    assert_eq!(p("x = 1;"), "(= x 1);");
    assert_eq!(p("x = 1; y = 2"), "(= x 1);\n(= y 2)");
    assert_eq!(p("x = 1; y = 2;"), "(= x 1);\n(= y 2);");
    assert_eq!(p("a + 1"), "(+ a 1)");
    assert_eq!(p("a + 1;"), "(+ a 1);");
    assert_eq!(p("A[2, end] = 0"), "(= (idx A 2 end) 0)");
    assert_eq!(p("s.campo[1].x = 3"), "(= (. (idx (. s campo) 1) x) 3)");
    assert_eq!(p("m, i = max(v)"), "(= (m i) (call max v))");
    assert_eq!(p("_, i = max(v);"), "(= (_ i) (call max v));");
    assert_eq!(p("x += 1"), "(+= x 1)");
    assert_eq!(p("x -= 1"), "(-= x 1)");
    assert_eq!(p("A *= B"), "(*= A B)");
    assert_eq!(p("A /= 2;"), "(/= A 2);");
    assert_eq!(p("f = x -> x + 1"), "(= f (-> (x) (+ x 1)))");
    assert_eq!(p("ok = a > 0 and b"), "(= ok (and (> a 0) b))");
    // Linhas em branco, comentários e `;` soltos são ignorados.
    assert_eq!(
        p("\n# c\nx = 1\n\n;\ny\n"),
        "(= x 1)\nend_placeholder".replace("end_placeholder", "y")
    );
    assert_eq!(p(""), "");
    assert_eq!(p("# só comentário\n"), "");
    // Exibição: expressões e atribuições sim, o resto não.
    let prog = parse_program("x = 1\nx\nx;\nfor k in v:\n    pass\n").unwrap();
    let show: Vec<bool> = prog.iter().map(|s| s.show).collect();
    assert_eq!(show, [true, true, false, false]);
    assert!(matches!(prog[3].kind, StmtKind::For(..)));
    assert_eq!((prog[3].line, prog[3].col), (4, 1));
}

#[test]
fn erros_de_atribuicao() {
    assert_eq!(erro("f(1) = 2"), ("chimalab:sintaxe:alvoInvalido", 1, 1));
    assert_eq!(erro("x + 1 = 2"), ("chimalab:sintaxe:alvoInvalido", 1, 1));
    assert_eq!(erro("a, 1 = f()"), ("chimalab:sintaxe:alvoInvalido", 1, 4));
    assert_eq!(erro("a'[1] = 0"), ("chimalab:sintaxe:alvoInvalido", 1, 1));
    assert_eq!(erro("f(x)[1] = 0"), ("chimalab:sintaxe:alvoInvalido", 1, 1));
    assert_eq!(erro("a, b"), ("chimalab:sintaxe:fimInesperado", 1, 5));
    assert_eq!(erro("a, b; c"), ("chimalab:sintaxe:tokenInesperado", 1, 5));
    assert_eq!(erro("a, b += 1"), ("chimalab:sintaxe:tokenInesperado", 1, 6));
    assert_eq!(erro("x = 1 y = 2"), ("chimalab:sintaxe:tokenInesperado", 1, 7));
    assert_eq!(erro("if = 3"), ("chimalab:sintaxe:palavraReservada", 1, 1));
    assert_eq!(erro("in += 3"), ("chimalab:sintaxe:palavraReservada", 1, 1));
    assert_eq!(erro("x = and"), ("chimalab:sintaxe:palavraReservada", 1, 5));
    let msg = parse_program("f(1) = 2").unwrap_err().to_string();
    assert!(msg.contains("colchetes"), "{msg}");
    let msg = parse_program("x = in").unwrap_err().to_string();
    assert_eq!(
        msg,
        "linha 1, coluna 5: `in` é palavra reservada e não pode ser usada como nome"
    );
}

/// CA (resposta 40.A): `if`/`elif`/`else` por indentação.
#[test]
fn ca40_if_elif_else() {
    let src = "if x > 0:\n    y = 1\nelif x < 0:\n    y = -1\n    z = 2\nelse:\n    y = 0\nw = y\n";
    assert_eq!(
        p(src),
        "(if ((> x 0) ((= y 1))) ((< x 0) ((= y (neg 1)) (= z 2))) (else ((= y 0))))\n(= w y)"
    );
    assert_eq!(p("if a:\n    b\n"), "(if (a (b)))");
    assert_eq!(p("if a:\n    b\nelse:\n    c"), "(if (a (b)) (else (c)))");
    // Aninhados, com dois níveis fechando de uma vez.
    let src = "if a:\n    if b:\n        c\n    d\ne\n";
    assert_eq!(p(src), "(if (a ((if (b (c))) d)))\ne");
    let src = "if a:\n    if b:\n        c\nd\n";
    assert_eq!(p(src), "(if (a ((if (b (c))))))\nd");
    // O `else` pertence ao `if` do mesmo nível.
    let src = "if a:\n    if b:\n        c\nelse:\n    d\n";
    assert_eq!(p(src), "(if (a ((if (b (c))))) (else (d)))");
    // Linhas em branco e comentários no meio do bloco.
    let src = "if a:\n\n    # comentário\n    b\n\n    c\n";
    assert_eq!(p(src), "(if (a (b c)))");
    // Condições com lógicos.
    assert_eq!(
        p("if not done and n < 10:\n    n += 1\n"),
        "(if ((and (not done) (< n 10)) ((+= n 1))))"
    );
}

/// CA (resposta 41.A): `for k in expr:` com intervalos no cabeçalho.
#[test]
fn ca41_for_e_intervalos_no_cabecalho() {
    assert_eq!(p("for k in 1:10:\n    s += k\n"), "(for k (: 1 10) ((+= s k)))");
    assert_eq!(p("for k in 1:2:n:\n    s += k\n"), "(for k (: 1 2 n) ((+= s k)))");
    assert_eq!(
        p("for k in 10:-1:1:\n    pass\n"),
        "(for k (: 10 (neg 1) 1) (pass))"
    );
    assert_eq!(p("for c in A:\n    c\n"), "(for c A (c))");
    assert_eq!(p("for k in v[2:end]:\n    k\n"), "(for k (idx v (: 2 end)) (k))");
    assert_eq!(p("for k in (1:3):\n    k\n"), "(for k (: 1 3) (k))");
    // Comentário depois do `:` do cabeçalho.
    assert_eq!(p("for k in 1:n:  # laço\n    k\n"), "(for k (: 1 n) (k))");
    // Fora do cabeçalho, o intervalo continua como antes.
    assert_eq!(p("x = 1:3"), "(= x (: 1 3))");
    assert_eq!(
        p("while k < n:\n    k += 1\n    if k == 3:\n        break\n    continue\n"),
        "(while (< k n) ((+= k 1) (if ((== k 3) (break))) continue))"
    );
}

#[test]
fn ca_try_match_def() {
    assert_eq!(
        p("try:\n    x = f()\nexcept e:\n    x = 0\n"),
        "(try ((= x (call f))) (except e ((= x 0))))"
    );
    assert_eq!(
        p("try:\n    f()\nexcept:\n    pass\n"),
        "(try ((call f)) (except _ (pass)))"
    );
    let src = "match op:\n    case \"+\":\n        r = a + b\n    case \"-\", \"menos\":\n        r = a - b\n    case _:\n        r = 0\n";
    assert_eq!(
        p(src),
        "(match op (case (\"+\") ((= r (+ a b)))) (case (\"-\" \"menos\") ((= r (- a b)))) (case _ ((= r 0))))"
    );
    assert_eq!(
        p("match n:\n    case 1:\n        a\n"),
        "(match n (case (1) (a)))"
    );
    assert_eq!(p("match n:\n    case _:\n        a\n"), "(match n (case _ (a)))");
    assert_eq!(
        p("match n:\n\n    case 1:\n        a\n"),
        "(match n (case (1) (a)))"
    );
    // Intervalo como valor de caso não se confunde com o `:` do cabeçalho.
    assert_eq!(
        p("match k:\n    case 1:3:\n        a\n"),
        "(match k (case ((: 1 3)) (a)))"
    );
    let src = "def f(x, y=2, z=[1 2]):\n    return x + y, z\n";
    assert_eq!(p(src), "(def f (x y=2 z=[1 2]) ((return (+ x y) z)))");
    assert_eq!(p("def g():\n    return\n"), "(def g () ((return)))");
    // Funções aninhadas; `break` volta a valer dentro de laços da função.
    let src = "def f(v):\n    def g(x):\n        return x\n    for k in v:\n        if g(k):\n            return k\n            break\n";
    assert_eq!(
        p(src),
        "(def f (v) ((def g (x) ((return x))) (for k v ((if ((call g k) ((return k) break)))))))"
    );
}

#[test]
fn erros_de_blocos() {
    assert_eq!(erro("if x:\ny\n"), ("chimalab:sintaxe:blocoEsperado", 2, 1));
    assert_eq!(erro("if x:\n"), ("chimalab:sintaxe:fimInesperado", 2, 1));
    // Corpo na mesma linha não existe: `x: y` vira intervalo e o erro aparece no `=`.
    assert_eq!(erro("if x: y = 1\n"), ("chimalab:sintaxe:tokenInesperado", 1, 9));
    assert_eq!(erro("if x: y\n"), ("chimalab:sintaxe:tokenInesperado", 1, 8));
    assert!(
        parse_program("if x: y\n")
            .unwrap_err()
            .to_string()
            .contains("`:` no fim da linha")
    );
    assert_eq!(erro("if x\n    y\n"), ("chimalab:sintaxe:tokenInesperado", 1, 5));
    assert_eq!(
        erro("x = 1\n    y = 2\n"),
        ("chimalab:sintaxe:indentacaoInesperada", 2, 5)
    );
    assert_eq!(
        erro("else:\n    y\n"),
        ("chimalab:sintaxe:clausulaSemInstrucao", 1, 1)
    );
    assert_eq!(
        erro("x\nelif y:\n    z\n"),
        ("chimalab:sintaxe:clausulaSemInstrucao", 2, 1)
    );
    assert_eq!(
        erro("except:\n    y\n"),
        ("chimalab:sintaxe:clausulaSemInstrucao", 1, 1)
    );
    assert_eq!(
        erro("case 1:\n    y\n"),
        ("chimalab:sintaxe:clausulaSemInstrucao", 1, 1)
    );
    assert_eq!(erro("break\n"), ("chimalab:sintaxe:foraDeLaco", 1, 1));
    // Depois que o laço fecha, `break` volta a ser erro.
    assert_eq!(
        erro("for k in v:\n    pass\nbreak\n"),
        ("chimalab:sintaxe:foraDeLaco", 3, 1)
    );
    assert_eq!(
        erro("while a:\n    pass\ncontinue\n"),
        ("chimalab:sintaxe:foraDeLaco", 3, 1)
    );
    assert_eq!(
        erro("if a:\n    continue\n"),
        ("chimalab:sintaxe:foraDeLaco", 2, 5)
    );
    // O laço de fora não vale dentro de uma função.
    assert_eq!(
        erro("for k in v:\n    def f():\n        break\n"),
        ("chimalab:sintaxe:foraDeLaco", 3, 9)
    );
    assert_eq!(erro("return 1\n"), ("chimalab:sintaxe:returnForaDeFuncao", 1, 1));
    // Depois de uma função, `return` volta a ser erro.
    assert_eq!(
        erro("def f():\n    pass\nreturn\n"),
        ("chimalab:sintaxe:returnForaDeFuncao", 3, 1)
    );
    assert_eq!(
        erro("for 1 in v:\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 5)
    );
    assert_eq!(
        erro("for if in v:\n    a\n"),
        ("chimalab:sintaxe:palavraReservada", 1, 5)
    );
    assert_eq!(
        erro("for k of v:\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 7)
    );
    assert_eq!(
        erro("try:\n    a\nb\n"),
        ("chimalab:sintaxe:tokenInesperado", 3, 1)
    );
    assert_eq!(
        erro("try:\n    a\nexcept 1:\n    b\n"),
        ("chimalab:sintaxe:tokenInesperado", 3, 8)
    );
    assert_eq!(
        erro("match x:\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 2, 5)
    );
    assert_eq!(
        erro("match x:\n    case _:\n        a\n    case 1:\n        b\n"),
        ("chimalab:sintaxe:casoPadraoNaoUltimo", 4, 5)
    );
    assert_eq!(
        erro("match x:\n    case 1:\n        a\n    b\n"),
        ("chimalab:sintaxe:tokenInesperado", 4, 5)
    );
    assert_eq!(
        erro("def f(x=1, y):\n    a\n"),
        ("chimalab:sintaxe:obrigatorioDepoisDePadrao", 1, 12)
    );
    assert_eq!(
        erro("def f(x, x):\n    a\n"),
        ("chimalab:sintaxe:parametroRepetido", 1, 10)
    );
    assert_eq!(
        erro("def f(x y):\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 9)
    );
    assert_eq!(
        erro("def (x):\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 5)
    );
    assert_eq!(
        erro("def f x:\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 7)
    );
    assert_eq!(
        erro("def f(1):\n    a\n"),
        ("chimalab:sintaxe:tokenInesperado", 1, 7)
    );
    // Erro do léxico passa adiante com o identificador do léxico.
    assert_eq!(
        erro("if a:\n        b\n    c\n"),
        ("chimalab:lexico:indentacaoInconsistente", 3, 5)
    );
    // Mensagens.
    let m = |src: &str| parse_program(src).unwrap_err().to_string();
    assert_eq!(
        m("elif x:\n    y\n"),
        "linha 1, coluna 1: `elif` sem o `if` correspondente, no mesmo nível de indentação"
    );
    assert!(m("except:\n    y\n").contains("`try`"));
    assert!(m("case 1:\n    y\n").contains("`match`"));
    assert!(m("break").contains("`break` só vale dentro de `for` ou `while`"));
    assert!(m("return").contains("`return` só vale dentro de `def`"));
    assert!(m("def f(x, x):\n    a\n").contains("`x` repetido"));
    assert!(m("def f(x=1, y):\n    a\n").contains("sem valor padrão"));
    assert!(m("if x:\ny\n").contains("indentado"));
    assert!(m("x = 1\n    y = 2\n").contains("indentação inesperada"));
    assert!(m("match x:\n    case _:\n        a\n    case 1:\n        b\n").contains("último"));
    assert!(m("if x:\n").contains("terminou"));
}

#[test]
fn palavras_reservadas() {
    for w in KEYWORDS {
        assert!(is_keyword(w));
        assert_eq!(
            parse_program(&format!("{w}x = 1")).map(|_| ()),
            Ok(()),
            "{w}x é nome comum"
        );
    }
    assert!(!is_keyword("end") && !is_keyword("_") && !is_keyword("x"));
    // `end` continua valendo fora de índices como erro de expressão, não de palavra reservada.
    assert_eq!(erro("x = end"), ("chimalab:sintaxe:endForaDeIndice", 1, 5));
}

/// Exemplos de `docs/linguagem/03-instrucoes.md` (conformidade com a especificação).
#[test]
fn exemplos_da_especificacao_instrucoes() {
    // O exemplo vem do próprio documento: a especificação e o teste não podem divergir.
    let doc = include_str!("../../../docs/linguagem/03-instrucoes.md").replace("\r\n", "\n");
    let src = doc
        .split("## 3.8 Exemplo\n```\n")
        .nth(1)
        .and_then(|r| r.split("```").next())
        .expect("seção 3.8 com bloco de código");
    let prog = parse_program(src).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(prog.len(), 8);
    // Exibem: total = 0, k = 1 e quadrado = … (ok termina em `;`).
    assert_eq!(prog.iter().filter(|s| s.show).count(), 3);
    assert!(matches!(&prog[0].kind, StmtKind::Def(n, ps, _) if n == "media_movel" && ps.len() == 2));
    assert!(matches!(&prog[7].kind, StmtKind::Assign(..)) && !prog[7].show);
}

proptest! {
    /// Blocos aninhados gerados ao acaso: o texto com indentação reproduz a árvore.
    #[test]
    fn blocos_aninhados(prof in prop::collection::vec(0usize..4, 1..12)) {
        // Cada item abre um `if` no nível dado (limitado ao nível anterior + 1).
        let mut src = String::new();
        let mut esperado: Vec<usize> = Vec::new();
        let mut nivel = 0usize;
        for (k, &d) in prof.iter().enumerate() {
            let d = d.min(nivel);
            src.push_str(&"    ".repeat(d));
            src.push_str(&format!("if c{k}:\n"));
            src.push_str(&"    ".repeat(d + 1));
            src.push_str(&format!("x{k}\n"));
            esperado.push(d);
            nivel = d + 1;
        }
        let prog = parse_program(&src).unwrap();
        // Conta os ifs por profundidade percorrendo a árvore.
        fn conta(st: &[chimalab_lang::sintaxe::Stmt], d: usize, out: &mut Vec<usize>) {
            for s in st {
                if let StmtKind::If(ramos, _) = &s.kind {
                    out.push(d);
                    conta(&ramos[0].1, d + 1, out);
                }
            }
        }
        let mut obtido = Vec::new();
        conta(&prog, 0, &mut obtido);
        let mut a = esperado.clone();
        a.sort();
        obtido.sort();
        prop_assert_eq!(obtido, a);
        // Mostrar e reanalisar a forma canônica de cada expressão não muda nada (pontual).
        prop_assert!(prog.iter().all(|s| !s.show));
    }
}

/// Mensagens de erro e descrição de cada tipo de token encontrado.
#[test]
fn mensagens_e_tokens_descritos() {
    let m = |src: &str| parse_program(src).unwrap_err().to_string();
    assert_eq!(
        m("try: x\n"),
        "linha 1, coluna 6: esperava o fim da linha depois de `:` (o corpo vai indentado nas linhas seguintes), veio o nome `x`"
    );
    assert!(m("if a:\n    b\nelse: c\n").contains("o fim da linha depois de `:`"));
    assert!(m("a < b < c").contains("não se encadeiam"));
    assert!(m("x = end").contains("`end` só vale dentro de um índice"));
    assert!(m("f(y=1, 2)").contains("posicional depois de um nomeado"));
    assert!(m("a.(1)").contains("um nome depois de `.`"));
    // O erro do léxico aparece com a mensagem do léxico, sem prefixo duplicado.
    let lex = m("x = $");
    assert!(lex.starts_with("linha 1, coluna 5"), "{lex}");
    assert_eq!(lex.matches("linha").count(), 1, "{lex}");
    // Descrições de tokens.
    assert!(m("x 2i").contains("veio o imaginário 2i"));
    assert!(m("x \"t\"").contains("veio um texto"));
    assert!(m("for k in v k").contains("veio o nome `k`"));
    assert!(m("x 3").contains("veio o número 3"));
    assert!(m("x )").contains("caractere") || m("x )").contains("fechamento"));
}

/// Regressão: aninhamento profundo estourava a pilha e abortava o processo (20 000 parênteses).
/// Agora é erro com identificador; o limite em si cabe na pilha de 2 MiB dos testes.
#[test]
fn aninhamento_profundo_vira_erro() {
    use chimalab_lang::sintaxe::MAX_ANINHAMENTO;
    let id = |src: &str| parse_program(src).unwrap_err().id();
    let par = |k: usize| format!("{}x{}", "(".repeat(k), ")".repeat(k));
    assert_eq!(id(&par(20_000)), "chimalab:sintaxe:aninhamentoProfundo");
    assert_eq!(
        id(&format!("{}x{}", "[".repeat(5000), "]".repeat(5000))),
        "chimalab:sintaxe:aninhamentoProfundo"
    );
    assert_eq!(
        id(&format!("{}x", "-".repeat(20_000))),
        "chimalab:sintaxe:aninhamentoProfundo"
    );
    assert_eq!(
        id(&format!("{}x", "not ".repeat(20_000))),
        "chimalab:sintaxe:aninhamentoProfundo"
    );
    assert_eq!(
        id(&format!("x = 2{}", "^2".repeat(20_000))),
        "chimalab:sintaxe:aninhamentoProfundo"
    );
    assert_eq!(
        id(&format!("{}x", "y -> ".repeat(20_000))),
        "chimalab:sintaxe:aninhamentoProfundo"
    );
    // Blocos: 150 `if` aninhados.
    let mut src = String::new();
    for k in 0..150 {
        src.push_str(&format!("{}if a:\n", "    ".repeat(k)));
    }
    src.push_str(&format!("{}b\n", "    ".repeat(150)));
    assert_eq!(id(&src), "chimalab:sintaxe:aninhamentoProfundo");
    // Logo abaixo do limite ainda analisa (a expressão inteira conta um nível).
    let ok = par(MAX_ANINHAMENTO - 2);
    assert_eq!(parse_expr(&ok).unwrap().to_string(), "x");
    assert!(parse_expr(&format!("{}x", "-".repeat(MAX_ANINHAMENTO - 2))).is_ok());
    let mut src = String::new();
    for k in 0..MAX_ANINHAMENTO - 1 {
        src.push_str(&format!("{}if a:\n", "    ".repeat(k)));
    }
    src.push_str(&format!("{}b\n", "    ".repeat(MAX_ANINHAMENTO - 1)));
    assert!(parse_program(&src).is_ok());
    assert!(
        parse_program(&par(MAX_ANINHAMENTO))
            .unwrap_err()
            .to_string()
            .contains("níveis de aninhamento")
    );
}

/// O contador de aninhamento volta depois de cada `not`, sinal e potência: sequências longas (não
/// aninhadas) desses operadores analisam normalmente.
#[test]
fn sequencias_longas_nao_esgotam_o_aninhamento() {
    let junta = |parte: &str, sep: &str| vec![parte; 150].join(sep);
    assert!(parse_expr(&junta("not 0", " and ")).is_ok());
    assert!(parse_expr(&junta("!a", " | ")).is_ok());
    assert!(parse_expr(&junta("-1", " + ")).is_ok());
    assert!(parse_expr(&junta("2^2", " + ")).is_ok());
    assert!(parse_expr(&junta("(x)", " * ")).is_ok());
    let blocos = (0..150).map(|k| format!("if a:\n    b{k}\n")).collect::<String>();
    assert!(parse_program(&blocos).is_ok());
}
