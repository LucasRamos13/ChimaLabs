//! Avaliador da árvore (DT-39, resposta 48.A; DT-40): console, controle de fluxo, funções,
//! funções anônimas, índices, embutidas, exibição e erros com posição.

use chimalab_array::Complex64;
use chimalab_runtime::{Interpretador, Obj, Resultado, Value};
use proptest::prelude::*;

fn roda(src: &str) -> (Interpretador, Resultado) {
    let mut it = Interpretador::new();
    let r = it.executa(src);
    (it, r)
}

/// Texto do console; falha se houver erro.
fn console(src: &str) -> String {
    let (_, r) = roda(src);
    assert!(r.erro.is_none(), "{src:?}: {}", r.erro.unwrap());
    r.texto()
}

fn num(it: &Interpretador, nome: &str) -> Vec<f64> {
    match it.variavel(nome) {
        Some(Obj::Array(v)) => v.to_f64s().unwrap(),
        outro => panic!("{nome}: {outro:?}"),
    }
}

fn valor(it: &Interpretador, nome: &str) -> Value {
    match it.variavel(nome) {
        Some(Obj::Array(v)) => v.clone(),
        outro => panic!("{nome}: {outro:?}"),
    }
}

/// Executa e devolve as variáveis numéricas pedidas.
fn vars(src: &str, nomes: &[&str]) -> Vec<Vec<f64>> {
    let (it, r) = roda(src);
    assert!(r.erro.is_none(), "{src:?}: {}", r.erro.unwrap());
    nomes.iter().map(|n| num(&it, n)).collect()
}

fn erro(src: &str) -> (String, usize, usize) {
    let (_, r) = roda(src);
    let e = r.erro.unwrap_or_else(|| panic!("{src:?} deveria falhar"));
    (e.id, e.linha, e.coluna)
}

/// CA (DT-34): o console mostra atribuições e expressões; `;` suprime; um nome sozinho mostra a
/// variável sem mexer em `ans`.
#[test]
fn ca_console_ans_e_exibicao() {
    assert_eq!(console("x = 1 + 2"), "x = 3");
    assert_eq!(console("x = 1 + 2;"), "");
    assert_eq!(console("x = 3;\nx"), "x = 3");
    assert_eq!(console("x = 3;\n2 * x\nans + 1"), "ans = 6\nans = 7");
    assert_eq!(console("x = 3;\nx\nans = 5;\nx;\nans"), "x = 3\nans = 5");
    assert_eq!(console("a = 1; b = 2"), "b = 2");
    assert_eq!(console("m, i = max([3 9 2])"), "m = 9\ni = 2");
    assert_eq!(console("_, i = max([3 9 2])"), "i = 2");
    let (it, _) = roda("_, i = max([3 9 2])\n_ = 5\n");
    assert!(it.variavel("_").is_none(), "`_` descarta, não vira variável");
    assert_eq!(console("x = 1;\nx += 2"), "x = 3");
    assert_eq!(console("A = zeros(1, 3);\nA[2] = 7"), "A = 1×3\n  0  7  0");
    // Instruções compostas não exibem nada por si.
    assert_eq!(console("for k in 1:3:\n    pass\n"), "");
    // A variável do laço fica com o último valor.
    let (it, _) = roda("for k in 1:3:\n    pass\n");
    assert_eq!(num(&it, "k"), [3.0]);
    // Workspace persiste entre execuções; erros não apagam o que já foi feito.
    let mut it = Interpretador::new();
    assert!(it.executa("x = 1\ny = z\nw = 2").erro.is_some());
    assert_eq!(num(&it, "x"), [1.0]);
    assert!(it.variavel("w").is_none());
    assert_eq!(it.executa("x + 1").texto(), "ans = 2");
    assert_eq!(it.nomes(), ["ans", "x"]);
    it.define("v", Value::doubles(&[1, 2], vec![4.0, 5.0]));
    assert_eq!(it.executa("v").texto(), "v = 1×2\n  4  5");
}

#[test]
fn exibicao_de_valores() {
    let c = |s: &str| console(s);
    assert_eq!(c("x = 0.1"), "x = 0.1");
    assert_eq!(c("x = 1/3"), "x = 0.3333333333");
    assert_eq!(c("x = 2/3"), "x = 0.6666666667");
    assert_eq!(c("x = 0.1 + 0.2"), "x = 0.3");
    assert_eq!(c("x = 1e20"), "x = 1e20");
    assert_eq!(c("x = 1.5e-7"), "x = 1.5e-7");
    assert_eq!(c("x = 123456.789"), "x = 123456.789");
    // Regressão: a parte inteira não cabe em 10 algarismos: científica, não "12345678900".
    assert_eq!(c("x = 12345678901.5"), "x = 1.23456789e10");
    assert_eq!(c("x = 1234567890.5"), "x = 1.23456789e9");
    // Regressão: quase inteiros (resultado de `\\`) apareciam como `1e0`; agora mostram os 10
    // algarismos com as casas, deixando claro que não são exatos.
    assert_eq!(c("x = 123456789.95"), "x = 123456790.0");
    assert_eq!(c("x = 1 - eps/2"), "x = 1.000000000");
    assert_eq!(c("x = -2 - 4*eps"), "x = -2.000000000");
    assert_eq!(c("x = [1 - eps/2; 2]"), "x = 2×1\n  1.000000000\n            2");
    assert_eq!(c("x = 100 + 1e-9"), "x = 100.0000000");
    assert_eq!(c("x = single(1 - 2^-24)"), "x = 0.9999999");
    assert_eq!(c("x = single(1 + 2^-23)"), "x = 1.000000");
    // Empate exato na 10ª casa: arredonda para o par.
    assert_eq!(c("x = 123456789.25"), "x = 123456789.2");
    assert_eq!(c("x = 123456789.75"), "x = 123456789.8");
    assert_eq!(c("x = single(123456.75)"), "x = 123456.8");
    assert_eq!(c("x = single(12345678.5)"), "x = 12345678");
    assert_eq!(c("x = single(1234567.5)"), "x = 1.234568e6");
    assert_eq!(c("x = 1e15 + 0.5"), "x = 1e15");
    // Fronteira da regra dos inteiros: abaixo de 10¹⁵ por extenso; 10¹⁵ já em científica.
    assert_eq!(c("x = 1e15"), "x = 1e15");
    assert_eq!(c("x = 1e15 - 1"), "x = 999999999999999");
    assert_eq!(c("x = -1e15"), "x = -1e15");
    assert_eq!(c("x = -2.5"), "x = -2.5");
    assert_eq!(c("x = -0"), "x = -0");
    assert_eq!(c("x = [Inf -Inf NaN]"), "x = 1×3\n   Inf  -Inf   NaN");
    assert_eq!(c("x = 0.00001"), "x = 0.00001");
    assert_eq!(c("x = 0.000001"), "x = 1e-6");
    // 10 algarismos: arredonda para 1,000000000e14; a parte inteira (15 algarismos) não cabe.
    assert_eq!(c("x = 99999999999999.9"), "x = 1e14");
    assert_eq!(c("A = [1 -2; 30 4]"), "A = 2×2\n   1  -2\n  30   4");
    assert_eq!(c("z = 3 - 4i"), "z = 3 - 4i");
    // `-1i` é a negação IEEE de 0 + 1i: a parte real é −0, e a exibição mostra o sinal.
    assert_eq!(c("z = [1+2i, -1i]"), "z = 1×2\n   1 + 2i  -0 - 1i");
    assert_eq!(c("z = [1+2i, 0-1i]"), "z = 1×2\n  1 + 2i  0 - 1i");
    assert_eq!(c("b = 1 < 2"), "b = true");
    assert_eq!(c("b = [1 2 3] > 1"), "b = 1×3 logical\n  false   true   true");
    assert_eq!(c("s = 'olá'"), "s = \"olá\"");
    assert_eq!(c("s = ['ab'; 'cd']"), "s = 2×2 char\n  \"ab\"\n  \"cd\"");
    assert_eq!(c("e = []"), "e = [] (0×0)");
    assert_eq!(c("e = zeros(0, 3)"), "e = [] (0×3)");
    assert_eq!(c("e = int8(zeros(2, 0))"), "e = [] (2×0 int8)");
    assert_eq!(c("i = int8(-5)"), "i = -5");
    assert_eq!(c("i = uint8([1 250])"), "i = 1×2 uint8\n    1  250");
    assert_eq!(c("s = single(1/3)"), "s = 0.3333333");
    assert_eq!(
        c("A = ones(2, 2, 2)"),
        "A = 2×2×2\n  [:, :, 1]\n    1  1\n    1  1\n  [:, :, 2]\n    1  1\n    1  1"
    );
    assert_eq!(c("f = x -> x + 1"), "f = função anônima (x)");
    assert_eq!(c("def g(a, b=2):\n    return a\nh = g"), "h = função g(a, b=…)");
    assert_eq!(c("h = sin"), "h = função embutida sin");
    assert_eq!(c("disp([1 2])"), "1×2\n  1  2");
    assert_eq!(c("disp(5)"), "5");
    assert_eq!(c("disp('olá')"), "olá");
    assert_eq!(c("disp(['ab'; 'cd'])"), "ab\ncd");
    assert_eq!(c("disp('')"), "");
    assert_eq!(console("disp(char(zeros(0, 2)))\ndisp(1)"), "\n1");
    assert_eq!(c("disp(x -> x)"), "função anônima (x)");
}

/// CA (DT-34): f-strings com formato.
#[test]
fn ca_fstrings() {
    let s = |src: &str| -> String {
        let (it, r) = roda(&format!("s = {src}"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        match valor(&it, "s") {
            Value::Char(a) => a.as_slice().iter().collect(),
            v => panic!("{v:?}"),
        }
    };
    assert_eq!(s("f\"x = {1/3:.3f}\""), "x = 0.333");
    assert_eq!(s("f\"{2.5:e}\""), "2.500000e+00");
    assert_eq!(s("f\"{12345.678:.2e}\""), "1.23e+04");
    assert_eq!(s("f\"{0.000123:.1e}\""), "1.2e-04");
    assert_eq!(s("f\"{1/3:.4g}\""), "0.3333");
    assert_eq!(s("f\"{1/3:.4}\""), "0.3333");
    assert_eq!(s("f\"{1/3}\""), "0.3333333333");
    assert_eq!(s("f\"{42:d}|{42:>5}|{42:<5}|{42:^6}\""), "42|   42|42   |  42  ");
    assert_eq!(s("f\"{'ab':>4}|{'ab':<4}|{'ab':s}\""), "  ab|ab  |ab");
    assert_eq!(s("f\"{[1 2; 3 4]}\""), "[1 2; 3 4]");
    assert_eq!(s("f\"{[0.5 1.25]:.1f}\""), "[0.5 1.2]");
    assert_eq!(s("f\"{1 < 2}\""), "true");
    assert_eq!(s("f\"{3 - 4i}\""), "3 - 4i");
    assert_eq!(s("f\"{2 + 0.5i:.2f}\""), "2.00 + 0.50i");
    assert_eq!(s("f\"{int8(-7)}|{int8(-7):d}|{uint8(9):.2f}\""), "-7|-7|9.00");
    assert_eq!(s("f\"{single(0.5)}\""), "0.5");
    assert_eq!(s("f\"{1/0}|{-1/0:.2f}|{0/0:e}\""), "Inf|-Inf|NaN");
    assert_eq!(s("f\"{{x}}\""), "{x}");
    assert_eq!(s("f\"{x -> x}\""), "função anônima (x)");
    for (src, frag) in [
        ("f\"{1.5:d}\"", "exige inteiro"),
        ("f\"{1:.2d}\"", "não tem precisão"),
        ("f\"{1:s}\"", "só para textos"),
        ("f\"{'a':.2f}\"", "largura e alinhamento"),
        ("f\"{1:q}\"", "inválido"),
        ("f\"{1:.99f}\"", "inválido"),
        ("f\"{1 < 2:d}\"", "lógicos"),
    ] {
        let (_, r) = roda(&format!("s = {src}"));
        let e = r.erro.unwrap();
        assert_eq!(e.id, "chimalab:execucao:formatoInvalido", "{src}");
        assert!(e.mensagem.contains(frag), "{src}: {}", e.mensagem);
    }
}

/// CA (resposta 41.A): `for` percorre colunas; `break`, `continue`, `while`, `if`.
#[test]
fn ca41_controle_de_fluxo() {
    let v = vars("s = 0\nfor c in [1 2; 3 4]:\n    s = s + c\n", &["s"]);
    assert_eq!(v[0], [3.0, 7.0], "soma das colunas [1;3] + [2;4]");
    let v = vars(
        "n = 0\nfor x in zeros(0, 3):\n    n += 1\nfor x in zeros(3, 0):\n    n += 1\n",
        &["n"],
    );
    assert_eq!(v[0], [0.0], "vazio não itera");
    let v = vars("n = 0\nfor x in ones(2, 3, 2):\n    n += 1\n", &["n"]);
    assert_eq!(v[0], [6.0], "N-D: colunas de todas as fatias");
    let (it, r) = roda("t = ''\nfor ch in 'abc':\n    t = [ch t]\n");
    assert!(r.erro.is_none());
    assert_eq!(
        valor(&it, "t"),
        Value::text("cba"),
        "for sobre texto percorre os caracteres"
    );
    let src = "s = 0\nfor k in 1:10:\n    if k == 3:\n        continue\n    elif k > 6:\n        break\n    s += k\n";
    assert_eq!(
        vars(src, &["s", "k"]),
        [vec![1.0 + 2.0 + 4.0 + 5.0 + 6.0], vec![7.0]]
    );
    let src = "n = 0\nk = 1\nwhile k < 1000:\n    k *= 2\n    n += 1\n";
    assert_eq!(vars(src, &["n", "k"]), [vec![10.0], vec![1024.0]]);
    // `break` interno só sai do laço de dentro.
    let src =
        "c = 0\nfor i in 1:3:\n    for j in 1:3:\n        if j == 2:\n            break\n        c += 1\n";
    assert_eq!(vars(src, &["c"])[0], [3.0]);
    let src = "x = 5\nif x > 3:\n    y = 1\nelif x > 1:\n    y = 2\nelse:\n    y = 3\n";
    assert_eq!(vars(src, &["y"])[0], [1.0]);
    assert_eq!(
        vars(
            "x = 2\nif x > 3:\n    y = 1\nelif x > 1:\n    y = 2\nelse:\n    y = 3\n",
            &["y"]
        )[0],
        [2.0]
    );
    assert_eq!(
        vars(
            "x = 0\nif x > 3:\n    y = 1\nelif x > 1:\n    y = 2\nelse:\n    y = 3\n",
            &["y"]
        )[0],
        [3.0]
    );
    assert_eq!(vars("y = 0\nif 0:\n    y = 1\n", &["y"])[0], [0.0]);
    // Condição: escalar obrigatório (DT-40); lógico, número e caractere valem.
    assert_eq!(
        erro("if [1 1]:\n    pass\n"),
        ("chimalab:execucao:condicaoNaoEscalar".into(), 1, 4)
    );
    // CA do F006: `if []` é falso (vazio não é erro; array de dois ou mais elementos é).
    assert_eq!(
        vars("y = 0\nif []:\n    y = 1\nelse:\n    y = 2\n", &["y"])[0],
        [2.0]
    );
    assert_eq!(vars("n = 0\nwhile zeros(1, 0):\n    n += 1\n", &["n"])[0], [0.0]);
    assert_eq!(console("b = [] or true"), "b = true");
    assert_eq!(console("b = [] and error('não')"), "b = false");
    assert_eq!(
        erro("if NaN:\n    pass\n").0,
        "chimalab:runtime:conversaoInvalida"
    );
    let (_, r) = roda("if [1 2] == [1 2]:\n    pass\n");
    assert!(r.erro.unwrap().mensagem.contains("all(…)"));
    assert_eq!(
        vars("y = 0\nif all([1 2] == [1 2]):\n    y = 1\n", &["y"])[0],
        [1.0]
    );
    assert_eq!(
        erro("for k in x -> x:\n    pass\n").0,
        "chimalab:execucao:funcaoComoValor"
    );
}

/// CA (resposta 42.C): `and`/`or` com curto-circuito; `&`/`|`/`!` elemento a elemento.
#[test]
fn ca42_logicos() {
    // O lado direito não é avaliado: o erro nunca acontece.
    assert_eq!(console("b = false and error('não')"), "b = false");
    assert_eq!(console("b = true || error('não')"), "b = true");
    assert_eq!(console("b = 1 && 2"), "b = true");
    assert_eq!(console("b = 0 or 0"), "b = false");
    assert_eq!(console("b = not 0"), "b = true");
    assert_eq!(console("b = ![1 0 2]"), "b = 1×3 logical\n  false   true  false");
    assert_eq!(
        console("b = [1 0 1] & [1 1 0] | [0 0 0]"),
        "b = 1×3 logical\n   true  false  false"
    );
    assert_eq!(erro("b = true and error('sim')").0, "chimalab:usuario:erro");
    assert_eq!(
        erro("b = [1 1] and true"),
        ("chimalab:execucao:condicaoNaoEscalar".into(), 1, 5)
    );
    assert_eq!(
        erro("b = true and [1 1]"),
        ("chimalab:execucao:condicaoNaoEscalar".into(), 1, 14)
    );
}

/// CA: funções com padrão, nomeados, várias saídas, recursão e escopo isolado.
#[test]
fn ca_funcoes() {
    let src = "def f(x, y=10, z=y+1):\n    return x + y + z\na = f(1)\nb = f(1, 2)\nc = f(1, z=0)\nd = f(y=1, x=2)\n";
    assert_eq!(
        vars(src, &["a", "b", "c", "d"]),
        [vec![22.0], vec![6.0], vec![11.0], vec![5.0]]
    );
    let src = "def mm(v):\n    return min(v), max(v)\nlo, hi = mm([4 1 9])\nx = mm([4 1 9])\n";
    assert_eq!(vars(src, &["lo", "hi", "x"]), [vec![1.0], vec![9.0], vec![1.0]]);
    // Recursão: fatorial de 20 é exato em double.
    let src = "def fat(n):\n    if n <= 1:\n        return 1\n    return n * fat(n - 1)\nr = fat(20)\n";
    assert_eq!(vars(src, &["r"])[0], [2_432_902_008_176_640_000.0]);
    // Definida depois do uso (as `def` do programa valem desde o início).
    assert_eq!(
        vars("r = dobro(4)\ndef dobro(x):\n    return 2 * x\n", &["r"])[0],
        [8.0]
    );
    // Escopo isolado: a função não vê o workspace.
    assert_eq!(
        erro("a = 1\ndef f():\n    return a\nf()\n"),
        ("chimalab:execucao:nomeIndefinido".into(), 3, 12)
    );
    // ... e não altera variáveis do workspace.
    assert_eq!(
        vars("a = 1\ndef f():\n    a = 2\n    return a\nb = f()\n", &["a", "b"]),
        [vec![1.0], vec![2.0]]
    );
    // Função sem return: chamada como instrução vale; como valor é erro.
    assert_eq!(console("def nada():\n    pass\nnada()"), "");
    assert_eq!(
        erro("def nada():\n    pass\nx = nada()\n"),
        ("chimalab:execucao:semValor".into(), 3, 5)
    );
    assert_eq!(
        erro("def f():\n    return 1\na, b = f()\n").0,
        "chimalab:execucao:saidasDemais"
    );
    // `def` aninhada passa a existir quando executa.
    assert_eq!(
        vars(
            "def ext():\n    def inte():\n        return 7\n    return inte()\nr = ext()\n",
            &["r"]
        )[0],
        [7.0]
    );
    // Recursão sem fim vira erro, não estouro de pilha.
    let (_, r) = roda("def f(n):\n    return f(n + 1)\nf(1)\n");
    let e = r.erro.unwrap();
    assert_eq!(
        (e.id.as_str(), e.linha),
        ("chimalab:execucao:recursaoProfunda", 2)
    );
    // Erros de argumentos.
    for (src, frag) in [
        (
            "def f(x):\n    return x\nf(1, 2)\n",
            "aceita 1 argumento(s), recebeu 2",
        ),
        ("def f(x):\n    return x\nf()\n", "falta o argumento `x`"),
        ("def f(x):\n    return x\nf(y=1)\n", "não tem parâmetro `y`"),
        ("def f(x):\n    return x\nf(1, x=2)\n", "recebeu dois valores"),
        ("def f(x):\n    return x\nf(x=1, x=2)\n", "repetido"),
    ] {
        let (_, r) = roda(src);
        let e = r.erro.unwrap();
        assert!(e.id.starts_with("chimalab:execucao:argument"), "{src}: {}", e.id);
        assert!(e.mensagem.contains(frag), "{src}: {}", e.mensagem);
    }
    assert_eq!(
        erro("x = 3\nx(1)\n"),
        ("chimalab:execucao:chamarArray".into(), 2, 1)
    );
    assert_eq!(erro("[1 2](1)\n"), ("chimalab:execucao:chamarArray".into(), 1, 1));
    let (_, r) = roda("x = 3\nx(1)\n");
    assert!(
        r.erro
            .unwrap()
            .mensagem
            .contains("`x` é um array: para indexar, use x[…]")
    );
    // Embutida sem saída usada como valor.
    assert_eq!(erro("x = disp(1)").0, "chimalab:execucao:semValor");
    assert_eq!(erro("x = clear()").0, "chimalab:execucao:semValor");
    assert_eq!(
        erro("f = x -> x\nf[1]\n"),
        ("chimalab:execucao:indexarFuncao".into(), 2, 1)
    );
    assert_eq!(erro("y = sin + 1").0, "chimalab:execucao:funcaoComoValor");
    assert_eq!(erro("y = [sin]").0, "chimalab:execucao:funcaoComoValor");
}

/// CA (resposta 47.A): funções anônimas capturam valores na criação (DT-40).
#[test]
fn ca47_funcoes_anonimas() {
    assert_eq!(
        vars("a = 2\nf = x -> a * x\na = 10\nr = f(3)\n", &["r"])[0],
        [6.0]
    );
    assert_eq!(vars("f = (x, y) -> x^2 + y\nr = f(3, 1)\n", &["r"])[0], [10.0]);
    assert_eq!(vars("c = () -> 42\nr = c()\n", &["r"])[0], [42.0]);
    // Fábrica de funções: a anônima leva o valor de k de cada chamada.
    let src = "def soma_de(k):\n    return x -> x + k\nmais2 = soma_de(2)\nmais5 = soma_de(5)\nr = mais2(1) + mais5(1)\n";
    assert_eq!(vars(src, &["r"])[0], [9.0]);
    // Anônima dentro de anônima; passada como argumento.
    assert_eq!(vars("som = x -> y -> x + y\nr = som(1)(2)\n", &["r"])[0], [3.0]);
    let src = "def aplica(f, v):\n    return f(v)\nr = aplica(x -> 3 * x, 4)\ns = aplica(sqrt, 16)\n";
    assert_eq!(vars(src, &["r", "s"]), [vec![12.0], vec![4.0]]);
    // Variável livre indefinida só falha na chamada.
    assert_eq!(
        erro("f = x -> x + w\nf(1)\n"),
        ("chimalab:execucao:nomeIndefinido".into(), 1, 14)
    );
    assert_eq!(erro("f = x -> x\nf(1, 2)\n").0, "chimalab:execucao:argumentos");
    assert_eq!(vars("f = x -> x\nr = f(x=5)\n", &["r"])[0], [5.0]);
    // Captura dentro de f-string e de índices.
    assert_eq!(
        vars(
            "v = [5 6 7]\ng = k -> v[k] + numel(v)\nv = []\nr = g(2)\n",
            &["r"]
        )[0],
        [9.0]
    );
}

/// CA (F004 pela linguagem): índices com colchetes, `end`, máscaras, crescimento e remoção.
#[test]
fn ca_indices_e_atribuicao() {
    let src =
        "A = [1 2 3; 4 5 6]\na = A[2, end]\nb = A[end]\nc = A[:, 2]\nd = A[end - 1, 1:2]\ne = A[A > 3]\n";
    let v = vars(src, &["a", "b", "c", "d", "e"]);
    assert_eq!(
        v,
        [
            vec![6.0],
            vec![6.0],
            vec![2.0, 5.0],
            vec![1.0, 2.0],
            vec![4.0, 5.0, 6.0]
        ]
    );
    let (it, _) = roda("A = [1 2 3; 4 5 6]\nc = A[:, 2]\n");
    assert_eq!(valor(&it, "c").dims(), [2, 1]);
    // `end` aninhado: cada índice vê o seu array.
    assert_eq!(
        vars("v = [10 20 30]\nw = [1 2]\nr = v[w[end]]\n", &["r"])[0],
        [20.0]
    );
    assert_eq!(vars("v = [10 20 30]\nr = v[min(end, 5)]\n", &["r"])[0], [30.0]);
    // Atribuição com crescimento, `end + 1` e remoção.
    let src = "v = [1 2]\nv[end + 1] = 3\nv[5] = 9\nu = []\nu[3] = 1\nw = [1 2 3 4]\nw[2] = []\n";
    let v = vars(src, &["v", "u", "w"]);
    assert_eq!(
        v,
        [
            vec![1.0, 2.0, 3.0, 0.0, 9.0],
            vec![0.0, 0.0, 1.0],
            vec![1.0, 3.0, 4.0]
        ]
    );
    assert_eq!(vars("novo[2, 2] = 5\n", &["novo"])[0], [0.0, 0.0, 0.0, 5.0]);
    assert_eq!(
        vars("A = eye(2)\nA[1, :] = [7 8]\n", &["A"])[0],
        [7.0, 0.0, 8.0, 1.0]
    );
    assert_eq!(vars("A = [1 2]\nA[1] += 10\n", &["A"])[0], [11.0, 2.0]);
    // Transpostas por membro e por apóstrofo.
    assert_eq!(
        vars("A = [1 2; 3 4]\nB = A.T\nC = A'\n", &["B", "C"]),
        [vec![1.0, 2.0, 3.0, 4.0], vec![1.0, 2.0, 3.0, 4.0]]
    );
    let (it, _) = roda("z = [1+2i 3]\nh = z.H\nt = z.'\n");
    assert_eq!(valor(&it, "h").to_complex()[0].im, -2.0);
    assert_eq!(valor(&it, "t").to_complex()[0].im, 2.0);
    // Erros.
    assert_eq!(
        erro("v = [1 2]\nv[0]\n"),
        ("chimalab:runtime:indiceInvalido".into(), 2, 1)
    );
    assert_eq!(
        erro("v = [1 2]\nv[1.5]\n"),
        ("chimalab:runtime:indiceInvalido".into(), 2, 3)
    );
    assert_eq!(erro("v = [1 2]\nv[-1]\n").0, "chimalab:runtime:indiceInvalido");
    assert_eq!(
        erro("v = [1 2]\nv[3]\n"),
        ("chimalab:runtime:indiceForaDosLimites".into(), 2, 1)
    );
    assert_eq!(
        erro("v = [1 2]\nv[2, 2, 2] = 1\nv.x\n").0,
        "chimalab:execucao:naoSuportado"
    );
    assert_eq!(
        erro("v = [1 2]\nv[1][1] = 0\n").0,
        "chimalab:execucao:naoSuportado"
    );
    assert_eq!(erro("s.a = 1\n").0, "chimalab:execucao:naoSuportado");
    assert_eq!(
        erro("f = x -> x\nf[1] = 2\n").0,
        "chimalab:execucao:funcaoComoValor"
    );
    assert_eq!(
        erro("v = [1 2]\nv[1] = x -> x\n").0,
        "chimalab:execucao:funcaoComoValor"
    );
    assert_eq!(erro("x = :").0, "chimalab:sintaxe:tokenInesperado");
}

#[test]
fn ca_try_e_match() {
    let src = "try:\n    x = [1 2] + [1 2 3]\nexcept e:\n    x = 0\n";
    let (it, r) = roda(src);
    assert!(r.erro.is_none());
    assert_eq!(num(&it, "x"), [0.0]);
    match valor(&it, "e") {
        Value::Char(a) => assert!(
            a.as_slice()
                .iter()
                .collect::<String>()
                .contains("dimensões incompatíveis")
        ),
        v => panic!("{v:?}"),
    }
    // O erro pode vir de dentro de chamadas e índices: o estado é restaurado.
    let src = "def f(n):\n    return [1 2][n]\ntry:\n    f(5)\nexcept:\n    ok = 1\nr = f(2)\n";
    assert_eq!(vars(src, &["ok", "r"]), [vec![1.0], vec![2.0]]);
    let src = "def f(n):\n    return f(n + 1)\ntry:\n    f(1)\nexcept:\n    pass\ndef g(n):\n    if n == 0:\n        return 0\n    return g(n - 1)\nr = g(50)\n";
    assert_eq!(
        vars(src, &["r"])[0],
        [0.0],
        "o contador de chamadas volta depois do erro"
    );
    // error(id, msg) propaga com o identificador do usuário.
    let (_, r) = roda("error('meu:pacote:falhou', 'deu errado')");
    let e = r.erro.unwrap();
    assert_eq!(
        (e.id.as_str(), e.mensagem.as_str()),
        ("meu:pacote:falhou", "deu errado")
    );
    assert_eq!(erro("error(1)").0, "chimalab:execucao:argumentos");
    // match com números, textos, alternativas e padrão.
    let cls = "def cls(x):\n    match x:\n        case 1, 2:\n            return 10\n        case 'ab':\n            return 20\n        case _:\n            return 30\n";
    let src = format!("{cls}a = cls(2)\nb = cls('ab')\nc = cls('a')\nd = cls([1 2])\ne = cls(1)\n");
    assert_eq!(
        vars(&src, &["a", "b", "c", "d", "e"]),
        [vec![10.0], vec![20.0], vec![30.0], vec![30.0], vec![10.0]]
    );
    assert_eq!(
        console("match 5:\n    case 1:\n        x = 1\n"),
        "",
        "sem caso e sem padrão: nada"
    );
}

/// Operações matriciais e avisos.
#[test]
fn operacoes_e_avisos() {
    assert_eq!(vars("x = [2 0; 0 4] \\ [2; 8]\n", &["x"])[0], [1.0, 2.0]);
    assert_eq!(vars("x = [2 4] / [2 0; 0 4]\n", &["x"])[0], [1.0, 1.0]);
    assert_eq!(vars("P = [1 1; 0 1]^3\n", &["P"])[0], [1.0, 0.0, 3.0, 1.0]);
    assert_eq!(
        vars("y = [1 2 3] .^ 2 ./ [1 2 3] .* 2 .\\ 4\n", &["y"])[0],
        [2.0, 1.0, 2.0 / 3.0]
    );
    let (_, r) = roda("x = [1 2; 2 4] \\ [1; 2];");
    assert!(r.erro.is_none());
    assert_eq!(r.saidas.len(), 1);
    let t = r.texto();
    assert!(
        t.starts_with("aviso (chimalab:runtime:matrizSingular): matriz singular"),
        "{t}"
    );
    assert_eq!(
        erro("[1 2] * [3 4]"),
        ("chimalab:runtime:dimensoesIncompativeis".into(), 1, 1)
    );
    assert_eq!(erro("x = 1:[1 2]").0, "chimalab:execucao:intervaloNaoEscalar");
    assert_eq!(erro("x = 1:2:(3i)").0, "chimalab:runtime:conversaoInvalida");
    assert_eq!(vars("x = 0:0.5:2\n", &["x"])[0], [0.0, 0.5, 1.0, 1.5, 2.0]);
    assert_eq!(
        vars("x = -[1 2]\ny = +3\n", &["x", "y"]),
        [vec![-1.0, -2.0], vec![3.0]]
    );
}

/// Embutidas contra valores conhecidos (identidades e contas exatas).
#[test]
fn embutidas() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    assert_eq!(v("zeros(2)"), [0.0; 4]);
    assert_eq!(v("ones(1, 3)"), [1.0; 3]);
    assert_eq!(v("zeros()"), [0.0]);
    assert_eq!(v("eye(2, 3)"), [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    assert_eq!(v("eye(2)"), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(v("size(zeros(2, 3, 4))"), [2.0, 3.0, 4.0]);
    assert_eq!(v("size(zeros(2, 3), 1)"), [2.0]);
    assert_eq!(v("size(zeros(2, 3), 5)"), [1.0]);
    assert_eq!(
        vars("a, b = size(zeros(2, 3, 4))\n", &["a", "b"]),
        [vec![2.0], vec![12.0]]
    );
    assert_eq!(
        vars("a, b, c, d = size(zeros(2, 3))\n", &["c", "d"]),
        [vec![1.0], vec![1.0]]
    );
    assert_eq!(v("numel(zeros(2, 3))"), [6.0]);
    assert_eq!(v("ndims(zeros(2, 3, 4))"), [3.0]);
    assert_eq!(v("length(zeros(2, 7))"), [7.0]);
    assert_eq!(v("length(zeros(0, 7))"), [0.0]);
    assert_eq!(v("isempty(zeros(0, 3)) + isempty(1)"), [1.0]);
    assert_eq!(v("sum([1 2; 3 4])"), [4.0, 6.0]);
    assert_eq!(v("sum([1 2; 3 4], 2)"), [3.0, 7.0]);
    // Dimensão além das existentes: cada elemento é a própria fibra.
    assert_eq!(v("sum([1 2], 3)"), [1.0, 2.0]);
    assert_eq!(v("max(sum(ones(2, 3, 4), 3))"), [4.0, 4.0, 4.0]);
    assert_eq!(v("size(sum(ones(2, 3, 4)))"), [1.0, 3.0, 4.0]);
    assert_eq!(v("sum([1 2 3])"), [6.0]);
    assert_eq!(v("sum([])"), [0.0]);
    assert_eq!(v("sum(zeros(1, 0))"), [0.0]);
    // Soma compensada: 1e16 + 1 − 1e16 dá 1, não 0.
    assert_eq!(v("sum([1e16 1 -1e16])"), [1.0]);
    assert_eq!(v("sum([true true false])"), [2.0]);
    assert_eq!(v("mean([1 2; 3 4])"), [2.0, 3.0]);
    assert_eq!(v("mean([1 2 3 4], 2)"), [2.5]);
    assert!(v("mean([])")[0].is_nan());
    assert_eq!(v("max([3 1 4 1 5])"), [5.0]);
    assert_eq!(v("min([3 NaN 1])"), [1.0]);
    assert!(v("max([NaN NaN])")[0].is_nan());
    assert_eq!(v("max([1 5; 7 2])"), [7.0, 5.0]);
    assert_eq!(
        vars("m, i = min([4 2 2 9])\n", &["m", "i"]),
        [vec![2.0], vec![2.0]]
    );
    assert_eq!(v("max([1 5 3], 4)"), [4.0, 5.0, 4.0]);
    assert_eq!(v("min([1 NaN], [NaN 0])"), [1.0, 0.0]);
    assert_eq!(v("max(zeros(0, 3))").len(), 0);
    assert_eq!(v("any([0 0 1])"), [1.0]);
    assert_eq!(v("any([0 NaN])"), [0.0]);
    assert_eq!(v("all([1 2; 0 3])"), [0.0, 1.0]);
    assert_eq!(v("all([])"), [1.0]);
    assert_eq!(v("any([])"), [0.0]);
    assert_eq!(v("abs([-2 3])"), [2.0, 3.0]);
    assert_eq!(v("abs(3 + 4i)"), [5.0]);
    assert_eq!(v("floor([-1.5 1.5]) + ceil([-1.5 1.5])"), [-3.0, 3.0]);
    assert_eq!(v("round([-2.5 2.5 0.4])"), [-3.0, 3.0, 0.0]);
    assert_eq!(v("sign([-3 0 2])"), [-1.0, 0.0, 1.0]);
    assert_eq!(v("exp(0) + log(1) + sin(0) + cos(0) + tan(0)"), [2.0]);
    assert_eq!(v("mod([5 -5 5 -5], [3 3 -3 -3])"), [2.0, 1.0, -1.0, -2.0]);
    assert_eq!(v("mod(5, 0)"), [5.0]);
    assert_eq!(v("mod([1 2 3], 2)"), [1.0, 0.0, 1.0]);
    assert_eq!(v("mod(-1, Inf)"), [f64::INFINITY]);
    assert_eq!(v("mod(1, Inf) + mod(0, -Inf)"), [1.0]);
    assert_eq!(
        v("mod([1 -1], -Inf)"),
        [f64::NEG_INFINITY, -1.0],
        "sinal do divisor"
    );
    assert_eq!(v("range(0, 1; length=5)"), [0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(v("range(0, 1; step=0.5)"), [0.0, 0.5, 1.0]);
    assert_eq!(v("double(int8(200))"), [127.0]);
    assert_eq!(v("isequal([1 2], [1 2], [1 2])"), [1.0]);
    assert_eq!(v("isequal([1 2], [1 2]')"), [0.0]);
    assert_eq!(v("isequal('a', 97)"), [1.0]);
    assert_eq!(v("pi"), [std::f64::consts::PI]);
    assert_eq!(v("eps"), [f64::EPSILON]);
    assert_eq!(v("[inf -Inf]"), [f64::INFINITY, f64::NEG_INFINITY]);
    assert!(v("nan")[0].is_nan() && v("NaN")[0].is_nan());
    assert_eq!(v("true + false"), [1.0]);
    // Complexos automáticos.
    let (it, _) =
        roda("a = sqrt(-4)\nb = log(-1)\nc = exp(1i * pi)\nd = sign(3 + 4i)\ne = floor(1.5 - 2.5i)\n");
    assert_eq!(valor(&it, "a").to_complex()[0].im, 2.0);
    assert_eq!(valor(&it, "b").to_complex()[0].im, std::f64::consts::PI);
    assert!((valor(&it, "c").to_complex()[0].re + 1.0).abs() < 1e-15);
    assert_eq!(valor(&it, "d").to_complex()[0].re, 0.6);
    assert_eq!(valor(&it, "e").to_complex()[0].im, -3.0);
    let (it, _) = roda("s = sin(1i) + cos(1i) + tan(1i) + log(2i)\nz = sign(0i)\n");
    assert!(valor(&it, "s").is_complex());
    assert_eq!(valor(&it, "z").numel(), 1);
    // Classes preservadas.
    let (it, _) = roda(
        "a = abs(int8(-128))\nb = sum(int8([100 100]))\nc = max(int16([3 -9 7]))\nd = sum(single([1 2]))\ne = round(single(2.5))\nf = sum(int8([]))\ng = sign(int8([-3 0 4]))\n",
    );
    assert_eq!(valor(&it, "a").to_f64s().unwrap(), [127.0]);
    assert_eq!(valor(&it, "b").to_f64s().unwrap(), [127.0]);
    assert_eq!(valor(&it, "c").to_f64s().unwrap(), [7.0]);
    assert_eq!(valor(&it, "c").class().name(), "int16");
    assert_eq!(valor(&it, "d").class().name(), "single");
    assert_eq!(valor(&it, "e").class().name(), "single");
    assert_eq!(valor(&it, "f").class().name(), "int8");
    assert_eq!(valor(&it, "g").to_f64s().unwrap(), [-1.0, 0.0, 1.0]);
    let (it, _) = roda("m, i = min(int8([5 -3 -3]))\nx = max(single([1 NaN 3]))\ns = sum([1+2i 3-1i])\n");
    assert_eq!(
        (
            valor(&it, "m").to_f64s().unwrap(),
            valor(&it, "i").to_f64s().unwrap()
        ),
        (vec![-3.0], vec![2.0])
    );
    assert_eq!(valor(&it, "x").class().name(), "single");
    assert_eq!(
        valor(&it, "s").to_complex()[0],
        chimalab_array::Complex64::new(4.0, 1.0)
    );
    // Erros das embutidas.
    for (src, id, frag) in [
        ("zeros(-1)", "chimalab:execucao:argumentos", "inteiro ≥ 0"),
        ("zeros(1.5)", "chimalab:execucao:argumentos", "inteiro ≥ 0"),
        ("zeros([1 2])", "chimalab:execucao:argumentos", "escalar"),
        ("eye()", "chimalab:execucao:argumentos", "de 1 a 2"),
        (
            "numel(1, 2)",
            "chimalab:execucao:argumentos",
            "aceita 1 argumento",
        ),
        ("isequal(1)", "chimalab:execucao:argumentos", "pelo menos 2"),
        ("sum([1 2], 0)", "chimalab:execucao:argumentos", "começam em 1"),
        ("size(1, 0)", "chimalab:execucao:argumentos", "começam em 1"),
        (
            "range(0, 1)",
            "chimalab:execucao:argumentos",
            "precisa de `length=` ou `step=`",
        ),
        (
            "range(0, 1; length=2, step=1)",
            "chimalab:execucao:argumentos",
            "só um",
        ),
        ("range(0, 1; passo=1)", "chimalab:execucao:argumentos", "só um"),
        (
            "zeros(2; n=1)",
            "chimalab:execucao:argumentos",
            "argumento nomeado `n`",
        ),
        ("max(1 + 1i)", "chimalab:execucao:classeNaoSuportada", "complexos"),
        (
            "exp(int8(1))",
            "chimalab:execucao:classeNaoSuportada",
            "double(…)",
        ),
        (
            "mod(int8(5), 2)",
            "chimalab:execucao:classeNaoSuportada",
            "double",
        ),
        (
            "mod([1 2], [1 2 3])",
            "chimalab:runtime:dimensoesIncompativeis",
            "1×2 e 1×3",
        ),
        ("int8(1i)", "chimalab:runtime:conversaoInvalida", "complexo"),
        ("sum(x -> x)", "chimalab:execucao:funcaoComoValor", "função"),
        ("error('a', 2)", "chimalab:execucao:argumentos", "texto"),
        (
            "nada_disso(1)",
            "chimalab:execucao:nomeIndefinido",
            "`nada_disso` não está definido",
        ),
    ] {
        let (_, r) = roda(&format!("r = {src}\n"));
        let e = r.erro.unwrap_or_else(|| panic!("{src} deveria falhar"));
        assert_eq!(e.id, id, "{src}: {}", e.mensagem);
        assert!(e.mensagem.contains(frag), "{src}: {}", e.mensagem);
    }
    assert_eq!(erro("r = max(1, 1i)").0, "chimalab:execucao:classeNaoSuportada");
}

/// Erros de sintaxe chegam pelo mesmo caminho, sem saídas.
#[test]
fn erros_de_sintaxe_e_posicoes() {
    let (_, r) = roda("x = 1\ny = (2\n");
    let e = r.erro.unwrap();
    assert_eq!(e.id, "chimalab:lexico:delimitadorNaoFechado");
    assert!(r.saidas.is_empty(), "nada executa se a sintaxe falha");
    let (_, r) = roda("if x\n    y\n");
    let e = r.erro.unwrap();
    assert_eq!(
        (e.id.as_str(), e.linha, e.coluna),
        ("chimalab:sintaxe:tokenInesperado", 1, 5)
    );
    assert_eq!(e.to_string(), format!("linha 1, coluna 5: {}", e.mensagem));
    assert!(!e.mensagem.starts_with("linha"), "{}", e.mensagem);
    let (_, r) = roda("x = 1\ny = nada\n");
    assert_eq!(
        r.texto(),
        "x = 1\nerro: linha 2, coluna 5: `nada` não está definido"
    );
}

/// O exemplo da especificação (docs/linguagem/03-instrucoes.md, §3.8) roda e dá os valores
/// calculados à mão.
#[test]
fn exemplo_da_especificacao_executa() {
    let doc = include_str!("../../../docs/linguagem/03-instrucoes.md").replace("\r\n", "\n");
    let src = doc
        .split("## 3.8 Exemplo\n```\n")
        .nth(1)
        .and_then(|r| r.split("```").next())
        .unwrap();
    let mut it = Interpretador::new();
    let r = it.executa(src);
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    // total: k par e ≠ 6 soma (2, 4, 8, 10); 7 e 9 pulam o seguinte.
    assert_eq!(num(&it, "total"), [24.0]);
    // Média móvel de 1:10 com janela 3: [1, 1.5, 2, 3, …, 9].
    let esperado: Vec<f64> = (1..=10)
        .map(|i| {
            if i == 1 {
                1.0
            } else if i == 2 {
                1.5
            } else {
                (i - 1) as f64
            }
        })
        .collect();
    assert_eq!(num(&it, "r"), esperado);
    assert_eq!(num(&it, "ok"), [1.0]);
    let r = it.executa("q = quadrado(3)\nc = classifica(-2)\n");
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    assert_eq!(num(&it, "q"), [9.0]);
    assert_eq!(r.texto(), "q = 9\nc = \"negativo\"");
    // Exibição: as três atribuições sem `;` do nível de cima e as de dentro do laço.
    let r = Interpretador::new().executa(src);
    assert!(r.texto().starts_with("total = 0\nk = 1\n"), "{}", r.texto());
}

proptest! {
    /// Expressões aritméticas inteiras aleatórias: o resultado bate com a conta em i64 (oráculo
    /// direto), inclusive precedência, parênteses e sinais.
    #[test]
    fn aritmetica_contra_inteiros(a in -50i64..50, b in -50i64..50, c in -50i64..50, d in 1i64..9, forma in 0usize..6) {
        let (src, esperado) = match forma {
            0 => (format!("{a} + {b} * {c}"), a + b * c),
            1 => (format!("({a} + {b}) * {c}"), (a + b) * c),
            2 => (format!("{a} - {b} - {c}"), a - b - c),
            3 => (format!("-{d}^2 + {a}"), -(d * d) + a),
            4 => (format!("{a} * ({b} - {c}) - {d}"), a * (b - c) - d),
            _ => (format!("mod({a}, {d}) + {b}"), a.rem_euclid(d) + b),
        };
        let mut it = Interpretador::new();
        let r = it.executa(&format!("r = {src}"));
        prop_assert!(r.erro.is_none(), "{}", src);
        prop_assert_eq!(num(&it, "r"), vec![esperado as f64], "{}", src);
    }

    /// Soma por laço contra a fórmula fechada; o `while` equivalente dá o mesmo.
    #[test]
    fn lacos_contra_formula(n in 0usize..60, passo in 1usize..4) {
        let src = format!(
            "s = 0\nfor k in 1:{passo}:{n}:\n    s += k\nt = 0\nk = 1\nwhile k <= {n}:\n    t += k\n    k += {passo}\n"
        );
        let mut it = Interpretador::new();
        let r = it.executa(&src);
        prop_assert!(r.erro.is_none());
        let esperado: usize = (1..=n).step_by(passo).sum();
        prop_assert_eq!(num(&it, "s"), vec![esperado as f64]);
        prop_assert_eq!(num(&it, "t"), vec![esperado as f64]);
    }

    /// Indexar depois de atribuir devolve o valor atribuído (ida e volta), com crescimento.
    #[test]
    fn atribuir_e_ler(k in 1usize..12, x in -1000i32..1000) {
        let mut it = Interpretador::new();
        let r = it.executa(&format!("v = [1 2 3]\nv[{k}] = {x}\ny = v[{k}]\nn = numel(v)\n"));
        prop_assert!(r.erro.is_none());
        prop_assert_eq!(num(&it, "y"), vec![f64::from(x)]);
        prop_assert_eq!(num(&it, "n"), vec![k.max(3) as f64]);
    }
}

/// Conformidade com docs/linguagem/05-console.md: o bloco da §5.3 (entradas `>>` e saídas
/// esperadas), a tabela da §5.4 e os exemplos da §5.2, lidos do próprio documento.
#[test]
fn exemplos_da_especificacao_console() {
    let doc = include_str!("../../../docs/linguagem/05-console.md").replace("\r\n", "\n");
    let secao = |s: &str| doc.split(s).nth(1).and_then(|r| r.split("\n## ").next()).unwrap();
    // §5.3: sessão de console.
    let bloco = secao("## 5.3").split("```\n").nth(1).unwrap();
    let mut it = Interpretador::new();
    let mut entradas = 0;
    let mut linhas = bloco.lines().peekable();
    while let Some(l) = linhas.next() {
        let entrada = l.strip_prefix(">> ").expect("linha de entrada");
        let mut esperado = Vec::new();
        while let Some(s) = linhas.peek().filter(|s| !s.starts_with(">> ")) {
            esperado.push(*s);
            linhas.next();
        }
        assert_eq!(it.executa(entrada).texto(), esperado.join("\n"), "{entrada}");
        entradas += 1;
    }
    assert_eq!(entradas, 4);
    // §5.4: tabela de f-strings.
    let mut casos = 0;
    for l in secao("## 5.4").lines().filter(|l| l.starts_with("| `f\"")) {
        let campos: Vec<&str> = l.split(" | ").collect();
        let alvo = campos[1].trim_end_matches(" |").trim_matches('`');
        for src in campos[0].trim_start_matches("| ").split(" ou ") {
            let src = src.trim_matches('`');
            if !src.starts_with('f') {
                continue;
            }
            let mut it = Interpretador::new();
            assert!(it.executa(&format!("s = {src}")).erro.is_none(), "{src}");
            let Some(Obj::Array(Value::Char(a))) = it.variavel("s") else {
                panic!()
            };
            assert_eq!(a.as_slice().iter().collect::<String>(), alvo, "{src}");
            casos += 1;
        }
    }
    assert_eq!(casos, 6);
    // §5.1: tabela do que o console mostra (entradas em sequência no mesmo workspace).
    let mut it = Interpretador::new();
    it.executa("v = [3 9 2];\nA = zeros(1, 3);\ndef f():\n    pass\n");
    let mut linhas51 = 0;
    for l in secao("## 5.1").lines().filter(|l| l.starts_with("| `")) {
        let campos: Vec<&str> = l.split(" | ").collect();
        let entrada = campos[0].trim_start_matches("| `").split('`').next().unwrap();
        let mostra = campos[1];
        let t = it.executa(entrada).texto();
        match mostra {
            "nada" => assert_eq!(t, "", "{entrada}"),
            m if m.starts_with('`') && !m.contains('…') => {
                let m: Vec<&str> = m
                    .split('`')
                    .filter(|s| !s.trim().is_empty() && *s != " e ")
                    .collect();
                assert_eq!(t, m.join("\n"), "{entrada}");
            }
            _ => assert!(!t.is_empty(), "{entrada}"),
        }
        linhas51 += 1;
    }
    assert_eq!(linhas51, 7);
    // §5.2: exemplos numéricos.
    for (src, alvo) in [
        ("1/3", "0.3333333333"),
        ("0.1 + 0.2", "0.3"),
        ("2^40", "1099511627776"),
        ("1.5e-7", "1.5e-7"),
        ("1e20", "1e20"),
        ("12345678901.5", "1.23456789e10"),
        ("1 - eps/2", "1.000000000"),
        ("3 - 4i", "3 - 4i"),
    ] {
        assert!(secao("## 5.2").contains(alvo), "{alvo} no documento");
        assert_eq!(
            Interpretador::new().executa(src).texto(),
            format!("ans = {alvo}"),
            "{src}"
        );
    }
}

/// Captura de variáveis livres em todas as formas de expressão, `return` de dentro de laços e
/// erros de várias saídas.
#[test]
fn capturas_retornos_e_saidas() {
    // Cada anônima usa `a`, `b` ou `s` numa forma diferente; depois as variáveis somem.
    let src = "a = 2\nb = [1 2]\ns = 'xy'\n\
        f1 = x -> -a + x\nf2 = x -> b' * x\nf3 = x -> b.T * x\nf4 = x -> a:x\nf5 = x -> [a x]\n\
        f6 = x -> f\"{s}{x}\"\nf7 = x -> b[x] + a\nf8 = x -> sum(b) * x\nf9 = x -> (y -> a + y)(x)\n\
        f10 = x -> a:a:x\n\
        a = []\nb = []\ns = []\n\
        r1 = f1(5)\nr2 = f2(3)\nr3 = f3(1)\nr4 = f4(4)\nr5 = f5(9)\nr7 = f7(2)\nr8 = f8(2)\nr9 = f9(1)\nr10 = f10(6)\n";
    let v = vars(src, &["r1", "r2", "r3", "r4", "r5", "r7", "r8", "r9", "r10"]);
    assert_eq!(
        v,
        [
            vec![3.0],
            vec![3.0, 6.0],
            vec![1.0, 2.0],
            vec![2.0, 3.0, 4.0],
            vec![2.0, 9.0],
            vec![4.0],
            vec![6.0],
            vec![3.0],
            vec![2.0, 4.0, 6.0]
        ]
    );
    let (it, r) = roda(&format!("{src}r6 = f6(1)\n"));
    assert!(r.erro.is_none());
    match valor(&it, "r6") {
        Value::Char(a) => assert_eq!(a.as_slice().iter().collect::<String>(), "xy1"),
        v => panic!("{v:?}"),
    }
    // Parâmetros sombreiam variáveis de fora; anônima interna liga o próprio parâmetro.
    assert_eq!(vars("x = 100\nf = x -> x + 1\nr = f(1)\n", &["r"])[0], [2.0]);
    assert_eq!(
        vars("y = 100\nf = x -> (y -> y * 2)(x)\nr = f(3)\n", &["r"])[0],
        [6.0]
    );
    // return de dentro de for e de while; break em while.
    let src = "def busca(v):\n    for x in v:\n        if x > 2:\n            return x\n    return -1\n\
        def conta(n):\n    k = 0\n    while true:\n        k += 1\n        if k >= n:\n            return k\n\
        def ate(n):\n    k = 0\n    while k < 100:\n        k += 1\n        if k == n:\n            break\n    return k\n\
        a = busca([1 5 3])\nb = busca([1 2])\nc = conta(4)\nd = ate(7)\n";
    assert_eq!(
        vars(src, &["a", "b", "c", "d"]),
        [vec![5.0], vec![-1.0], vec![4.0], vec![7.0]]
    );
    assert_eq!(console("b = [1 5] >= 5"), "b = 1×2 logical\n  false   true");
    assert_eq!(
        erro("a, b = 1\n"),
        ("chimalab:execucao:variasSaidas".into(), 1, 8)
    );
    assert_eq!(erro("a, b = [1 2]\n").0, "chimalab:execucao:variasSaidas");
    let (_, r) = roda("a, b = (x -> x)(1)\n");
    let e = r.erro.unwrap();
    assert_eq!(e.id, "chimalab:execucao:saidasDemais");
    assert!(e.mensagem.contains("função anônima"), "{}", e.mensagem);
    let (_, r) = roda("def um():\n    return 1\na, b = um()\n");
    assert!(
        r.erro
            .unwrap()
            .mensagem
            .contains("`um` devolve 1 valor(es), foram pedidos 2")
    );
    // Regressão: embutida de uma saída com dois alvos deixava o segundo sem valor, calada.
    assert_eq!(
        erro("a, b = numel(1)\n"),
        ("chimalab:execucao:saidasDemais".into(), 1, 8)
    );
    assert_eq!(erro("a, b, c = max([1 2])\n").0, "chimalab:execucao:saidasDemais");
    let (it, r) = roda("a, b = numel(1)\n");
    assert!(r.erro.is_some() && it.variavel("a").is_none(), "nada é atribuído");
}

/// Conversões de classe: cada nome produz a classe certa, com saturação.
#[test]
fn conversoes_de_classe() {
    for (nome, max) in [
        ("int8", 127.0),
        ("int16", 32767.0),
        ("int32", 2147483647.0),
        ("int64", 9.223372036854776e18),
        ("uint8", 255.0),
        ("uint16", 65535.0),
        ("uint32", 4294967295.0),
        ("uint64", 1.8446744073709552e19),
    ] {
        let (it, r) = roda(&format!(
            "x = {nome}(1e30)\ny = {nome}(-1e30)\nz = floor({nome}(7))\n"
        ));
        assert!(r.erro.is_none(), "{nome}");
        assert_eq!(valor(&it, "x").class().name(), nome);
        assert_eq!(valor(&it, "x").to_f64s().unwrap(), [max], "{nome}");
        let min = if nome.starts_with('u') { 0.0 } else { -max - 1.0 };
        assert_eq!(valor(&it, "y").to_f64s().unwrap(), [min], "{nome}");
        assert_eq!(valor(&it, "z").class().name(), nome);
    }
    let (it, _) = roda("a = logical([2 0])\nb = single(0.1)\nc = double('A')\nd = char([72 105])\n");
    assert_eq!(valor(&it, "d"), Value::text("Hi"));
    assert_eq!(valor(&it, "a").class().name(), "logical");
    assert_eq!(valor(&it, "b").class().name(), "single");
    assert_eq!(valor(&it, "c").to_f64s().unwrap(), [65.0]);
    assert_eq!(erro("logical(NaN)").0, "chimalab:runtime:conversaoInvalida");
}

/// CA4 (mecanismo): a interrupção para um laço infinito em menos de 1 s, sem perder o workspace;
/// um pedido feito antes da execução não vale para ela.
#[test]
fn ca4_interrupcao_para_laco_infinito() {
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};
    for src in [
        "while true:\n    pass\n",
        "for k in 1:1e12:\n    pass\n",
        "def f():\n    return f2()\ndef f2():\n    return 1\nwhile true:\n    x = f()\n",
    ] {
        let mut it = Interpretador::new();
        it.executa("guardado = 42;");
        let sinal = it.sinal_de_interrupcao();
        let t0 = Instant::now();
        let h = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            sinal.store(true, Ordering::Relaxed);
        });
        let r = it.executa(src);
        let dt = t0.elapsed();
        h.join().unwrap();
        let e = r.erro.expect("interrompido");
        assert_eq!(e.id, "chimalab:execucao:interrompido", "{src}");
        assert!(dt < Duration::from_secs(1), "{src}: {dt:?}");
        assert_eq!(num(&it, "guardado"), [42.0]);
        // O sinal foi consumido: a próxima execução roda normalmente.
        assert_eq!(it.executa("guardado + 1").texto(), "ans = 43");
    }
    let mut it = Interpretador::new();
    it.sinal_de_interrupcao().store(true, Ordering::Relaxed);
    assert!(it.executa("for k in 1:3:\n    pass\n").erro.is_none());
}

/// Desempenho do avaliador da árvore (medição, não critério): laço escalar e laço com
/// leitura e escrita indexadas num vetor de 10⁵ elementos. Antes de ler e escrever sem copiar,
/// o segundo era quadrático.
#[test]
#[ignore = "medição de desempenho; rodar com --release --ignored"]
fn desempenho_lacos() {
    use std::time::Instant;
    let mut it = Interpretador::new();
    let t = Instant::now();
    let r = it.executa("s = 0;\nfor k in 1:1000000:\n    s += k;\n");
    assert!(r.erro.is_none());
    let escalar = t.elapsed();
    assert_eq!(num(&it, "s"), [500_000_500_000.0]);
    let t = Instant::now();
    let r = it.executa("n = 100000;\nv = zeros(1, n);\nfor k in 1:n:\n    v[k] = 2 * k;\nt = 0;\nfor k in 1:n:\n    t += v[k];\n");
    assert!(r.erro.is_none());
    let indexado = t.elapsed();
    assert_eq!(num(&it, "t"), [100_000.0 * 100_001.0]);
    eprintln!("laço escalar 10⁶: {escalar:?}; laço indexado 2×10⁵: {indexado:?}");
}

/// Regressão: arrays enormes abortavam o processo (`for k in 1:1e12:` tentava alocar 8 TB).
/// Agora o `for` sobre intervalo não monta o vetor, e criar um array acima de 2³¹ elementos é
/// `chimalab:runtime:arrayGrandeDemais`.
#[test]
fn arrays_enormes_viram_erro() {
    assert_eq!(
        vars("for k in 1:1e12:\n    if k > 3:\n        break\n", &["k"])[0],
        [4.0]
    );
    assert_eq!(
        vars("n = 0;\nfor k in 1e300:-1e299:0:\n    n += 1;\n", &["n"])[0],
        [11.0]
    );
    for src in [
        "x = 1:1e12",
        "x = 0:1:1e300",
        "x = zeros(1e5, 1e5)",
        "x = ones(2^16, 2^16)",
        "x = eye(1e5)",
        "x = range(0, 1; length=1e12)",
        "x = range(0, 1; step=1e-12)",
        "v = 1;\nv[1e12] = 1",
        "A = 1;\nA[1e5, 1e5] = 1",
        "x = (1:1e6)' + (1:1e6)",
        "x = (1:1e6)' == (1:1e6)",
        "x = ((1:1e6)' > 0) & ((1:1e6) > 0)",
        "x = (1:1e5)' * (1:1e5)",
    ] {
        let (it, r) = roda(src);
        let e = r.erro.unwrap_or_else(|| panic!("{src} deveria falhar"));
        assert_eq!(
            e.id, "chimalab:runtime:arrayGrandeDemais",
            "{src}: {}",
            e.mensagem
        );
        assert!(e.mensagem.contains("2147483648"), "{}", e.mensagem);
        assert!(it.variavel("x").is_none());
    }
    // O alvo de uma atribuição recusada fica como estava.
    let (it, _) = roda("v = [1 2];\nv[1e12] = 1");
    assert_eq!(num(&it, "v"), [1.0, 2.0]);
    // Direto no runtime: contagem saturada, sem estouro de soma.
    assert_eq!(chimalab_runtime::colon_n(0.0, 1.0, 1e300), usize::MAX);
    assert_eq!(chimalab_runtime::colon_n(0.0, 1.0, f64::INFINITY), 0);
    assert_eq!(chimalab_runtime::colon_n(0.0, 0.0, 1.0), 0);
    assert!(chimalab_runtime::verifica_tamanho(&[usize::MAX, 2]).is_err());
    assert!(chimalab_runtime::verifica_tamanho(&[1 << 31]).is_ok());
    assert!(chimalab_runtime::verifica_tamanho(&[(1 << 31) + 1]).is_err());
    assert!(chimalab_runtime::verifica_tamanho(&[0, usize::MAX]).is_ok());
}

proptest! {
    /// O `for` preguiçoso sobre `a:s:b` visita exatamente os elementos do vetor montado, bit a
    /// bit (inclusive o ajuste do último elemento com passos inexatos como 0,1).
    #[test]
    fn for_sobre_intervalo_igual_ao_vetor(a in -50i32..50, s in prop::sample::select(vec![0.1, 0.25, 0.3, 1.0, -0.1, -0.7, 3.0, 1e-3]), m in 0usize..40) {
        let a = f64::from(a) / 10.0;
        let b = a + s * m as f64;
        let src = format!("v = zeros(1, 0);\nfor x in {a:?}:{s:?}:{b:?}:\n    v[end + 1] = x;\nw = {a:?}:{s:?}:{b:?};\n");
        let mut it = Interpretador::new();
        let r = it.executa(&src);
        prop_assert!(r.erro.is_none(), "{}", src);
        let (v, w) = (num(&it, "v"), num(&it, "w"));
        prop_assert_eq!(v.len(), w.len());
        prop_assert!(v.iter().zip(&w).all(|(x, y)| x.to_bits() == y.to_bits()), "{:?} {:?}", v, w);
    }
}

/// CA (DT-34): documentos mostram só a última expressão; `disp` e avisos aparecem.
#[test]
fn ca_documento_mostra_so_a_ultima() {
    let doc = |src: &str| Interpretador::new().executa_documento(src).texto();
    assert_eq!(doc("x = 1\ny = x + 1\ny * 10"), "ans = 20");
    assert_eq!(doc("x = 1\ny = x + 1"), "y = 2");
    assert_eq!(doc("x = 1\ny = x + 1;"), "");
    assert_eq!(
        doc("for k in 1:3:\n    k\n"),
        "",
        "a última é um for: o corpo não exibe"
    );
    assert_eq!(doc("disp(1)\nx = 2\n"), "1\nx = 2");
    assert_eq!(
        doc(
            "if 1:\n    a = 5\nelse:\n    b = 1\nmatch 1:\n    case 1:\n        c = 2\ntry:\n    d = 3\nexcept:\n    e = 4\nwhile false:\n    f = 1\n7"
        ),
        "ans = 7"
    );
    assert_eq!(
        doc("x = [1 2; 2 4] \\ [1; 2];\nx(1)"),
        "aviso (chimalab:runtime:matrizSingular): matriz singular ou mal condicionada; o resultado pode ser impreciso (rcond ≈ 0.000e0)\nerro: linha 2, coluna 1: `x` é um array: para indexar, use x[…]"
    );
    // Funções definidas no documento exibem o que o corpo pedir quando chamadas.
    assert_eq!(doc("def f():\n    z = 9\n    return z\nw = f();"), "z = 9");
    // O workspace é o mesmo do console.
    let mut it = Interpretador::new();
    it.executa_documento("a = 3\nb = 4");
    assert_eq!(it.executa("a + b").texto(), "ans = 7");
}

/// prod, cumsum, cumprod, sort, find, reshape e diag contra valores exatos.
#[test]
fn embutidas_de_arrays() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    let forma = |src: &str| {
        let (it, r) = roda(&format!("r = {src}\n"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        valor(&it, "r").dims().to_vec()
    };
    assert_eq!(v("prod([1 2 3 4])"), [24.0]);
    assert_eq!(v("prod([1 2; 3 4])"), [3.0, 8.0]);
    assert_eq!(v("prod([1 2; 3 4], 2)"), [2.0, 12.0]);
    assert_eq!(v("prod([])"), [1.0]);
    assert_eq!(v("cumsum([1 2 3 4])"), [1.0, 3.0, 6.0, 10.0]);
    assert_eq!(v("cumsum([1 2; 3 4])"), [1.0, 4.0, 2.0, 6.0]);
    assert_eq!(v("cumsum([1 2; 3 4], 2)"), [1.0, 3.0, 3.0, 7.0]);
    assert_eq!(v("cumsum([1e16 1 -1e16])"), [1e16, 1e16, 1.0], "compensada");
    assert_eq!(v("cumprod([1 2 3 4])"), [1.0, 2.0, 6.0, 24.0]);
    assert_eq!(forma("cumsum(zeros(0, 3))"), [0, 3]);
    let (it, _) = roda(
        "a = prod(int8([100 2]))\nb = cumsum(int8([100 100 -50]))\nc = cumprod(single([2 3]))\nz = cumsum([1i 2])\nw = prod([1i 1i])\n",
    );
    assert_eq!(valor(&it, "a").to_f64s().unwrap(), [127.0]);
    assert_eq!(valor(&it, "b").to_f64s().unwrap(), [100.0, 127.0, 77.0]);
    assert_eq!(valor(&it, "c").class().name(), "single");
    assert_eq!(valor(&it, "z").to_complex()[1], Complex64::new(2.0, 1.0));
    assert!(!valor(&it, "w").is_complex());
    assert_eq!(
        valor(&it, "w").to_f64s().unwrap(),
        [-1.0],
        "i·i = −1: complexo com parte imaginária nula volta a ser real (§4.4)"
    );
    // sort: estável, NaN no fim nas duas ordens, índices, dimensão e classe.
    assert_eq!(v("sort([3 1 2])"), [1.0, 2.0, 3.0]);
    assert_eq!(v("sort([3 NaN 1 NaN 2])")[..3], [1.0, 2.0, 3.0]);
    assert!(v("sort([3 NaN 1], 'descend')")[2].is_nan());
    assert_eq!(v("sort([3 NaN 1], 'descend')")[..2], [3.0, 1.0]);
    assert_eq!(
        vars("s, i = sort([2 1 2 1])\n", &["s", "i"]),
        [vec![1.0, 1.0, 2.0, 2.0], vec![2.0, 4.0, 1.0, 3.0]]
    );
    assert_eq!(
        vars("s, i = sort([2 1 2 1], 'descend')\n", &["i"])[0],
        [1.0, 3.0, 2.0, 4.0]
    );
    assert_eq!(v("sort([3 1; 2 4])"), [2.0, 3.0, 1.0, 4.0]);
    assert_eq!(v("sort([3 1; 2 4], 2)"), [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(v("sort([3 1; 2 4], 2, 'descend')"), [3.0, 4.0, 1.0, 2.0]);
    assert_eq!(v("sort([3 1 2], 'ascend')"), [1.0, 2.0, 3.0]);
    let (it, _) =
        roda("x = int64(9007199254740992);\na = sort([x + 1, x])\nb = sort('cab')\nc = sort([true false])\n");
    assert_eq!(valor(&it, "a").class().name(), "int64");
    match valor(&it, "a") {
        Value::Int(d) => assert_eq!(
            d.to_i128(),
            [9007199254740992, 9007199254740993],
            "inteiros exatos acima de 2⁵³"
        ),
        o => panic!("{o:?}"),
    }
    assert_eq!(valor(&it, "b"), Value::text("abc"));
    assert_eq!(valor(&it, "c").class().name(), "logical");
    assert_eq!(forma("sort(zeros(0, 3))"), [0, 3]);
    // find: linha para vetor linha, coluna para o resto; limite.
    assert_eq!(v("find([0 3 0 5])"), [2.0, 4.0]);
    assert_eq!(forma("find([0 3 0 5])"), [1, 2]);
    assert_eq!(forma("find([0; 3])"), [1, 1]);
    assert_eq!(v("find([1 0; 1 1])"), [1.0, 2.0, 4.0]);
    assert_eq!(forma("find([1 0; 1 1])"), [3, 1]);
    assert_eq!(v("find([1 1 1], 2)"), [1.0, 2.0]);
    assert_eq!(v("find([NaN 0 1i])"), [1.0, 3.0]);
    assert_eq!(forma("find([0 0])"), [1, 0]);
    assert_eq!(forma("find([])"), [0, 1]);
    // reshape com -1 inferido; forma normalizada.
    assert_eq!(forma("reshape(1:6, 2, 3)"), [2, 3]);
    assert_eq!(v("reshape(1:6, 3, 2)"), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(forma("reshape(1:6, -1, 2)"), [3, 2]);
    assert_eq!(forma("reshape(1:6, [2 -1])"), [2, 3]);
    assert_eq!(forma("reshape(1:6, 2, 3, 1)"), [2, 3]);
    assert_eq!(forma("reshape(1:8, 2, 2, 2)"), [2, 2, 2]);
    let (it, _) = roda("r = reshape(int8([1 2 3 4]), 2, 2)\n");
    assert_eq!(valor(&it, "r").class().name(), "int8");
    // diag: vetor → matriz (com deslocamento), matriz → coluna.
    assert_eq!(v("diag([1 2])"), [1.0, 0.0, 0.0, 2.0]);
    assert_eq!(v("diag([1 2], 1)"), [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0]);
    assert_eq!(
        v("diag([1 2], -1)"),
        [0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(v("diag([1 2; 3 4])"), [1.0, 4.0]);
    assert_eq!(forma("diag([1 2; 3 4])"), [2, 1]);
    assert_eq!(v("diag([1 2 3; 4 5 6], 1)"), [2.0, 6.0]);
    assert_eq!(v("diag([1 2 3; 4 5 6], -1)"), [4.0]);
    assert_eq!(forma("diag([1 2; 3 4], 5)"), [0, 1]);
    assert_eq!(forma("diag([])"), [0, 0]);
    let (it, _) = roda("d = diag([1i 2])\nl = diag([true true])\n");
    assert_eq!(valor(&it, "d").to_complex()[0], Complex64::new(0.0, 1.0));
    assert_eq!(valor(&it, "l").class().name(), "logical");
    // Erros.
    for (src, id) in [
        ("sort([1 1i])", "chimalab:execucao:classeNaoSuportada"),
        ("sort([1 2], 'x')", "chimalab:execucao:argumentos"),
        ("sort([1 2], 0)", "chimalab:execucao:argumentos"),
        ("reshape(1:6, 4, 2)", "chimalab:runtime:dimensoesIncompativeis"),
        ("reshape(1:6, -1, 4)", "chimalab:runtime:dimensoesIncompativeis"),
        ("reshape(1:6, -1, -1)", "chimalab:execucao:argumentos"),
        ("reshape(1:6, 1.5, 4)", "chimalab:execucao:argumentos"),
        ("reshape(1:6, [6])", "chimalab:execucao:argumentos"),
        ("reshape(1:6)", "chimalab:execucao:argumentos"),
        ("diag([1 2], 0.5)", "chimalab:execucao:argumentos"),
        ("diag(ones(2, 2, 2))", "chimalab:execucao:argumentos"),
        ("diag(1:1e5)", "chimalab:runtime:arrayGrandeDemais"),
        ("prod([1 2], 0)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

proptest! {
    /// sort devolve uma permutação ordenada com v[i] = s; cumsum termina na soma; reshape vai e
    /// volta; diag(diag(v)) = v.
    #[test]
    fn propriedades_de_arrays(v in prop::collection::vec(-20i32..20, 1..15)) {
        let lit = format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
        let src = format!(
            "v = {lit}\ns, i = sort(v)\nok1 = isequal(v[i], s)\nok2 = all(s[2:end] >= s[1:end-1])\n\
             c = cumsum(v)\nok3 = c[end] == sum(v)\nok4 = isequal(reshape(reshape(v, -1, 1), 1, -1), v)\n\
             ok5 = isequal(diag(diag(v)), v')\nok6 = isequal(sort(s), s)\n"
        );
        let mut it = Interpretador::new();
        let r = it.executa(&src);
        prop_assert!(r.erro.is_none(), "{}", r.erro.unwrap());
        for k in 1..=6 {
            prop_assert_eq!(num(&it, &format!("ok{k}")), vec![1.0], "ok{}", k);
        }
        let mut ordenado = v.clone();
        ordenado.sort();
        prop_assert_eq!(num(&it, "s"), ordenado.iter().map(|&x| f64::from(x)).collect::<Vec<_>>());
    }
}

/// Regressão: reduções complexas com parte imaginária nula ficavam complexas (§4.4).
#[test]
fn reducoes_complexas_voltam_a_ser_reais() {
    let (it, r) = roda(
        "a = sum([1i -1i])\nb = prod([1i 1i])\nc = cumsum([1i -1i])\nd = cumprod([1i 1i])\ne = sum([1i 2i])\n",
    );
    assert!(r.erro.is_none());
    assert!(!valor(&it, "a").is_complex() && !valor(&it, "b").is_complex());
    assert!(valor(&it, "c").is_complex(), "o primeiro elemento ainda é 1i");
    assert!(valor(&it, "d").is_complex());
    assert!(valor(&it, "e").is_complex());
    assert_eq!(num(&it, "b"), [-1.0]);
}

/// map, integral (F048), fzero e fminbnd (F051) pela linguagem, contra valores exatos.
#[test]
fn funcoes_que_recebem_funcoes() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0)[0];
    let perto = |a: f64, b: f64, tol: f64| (a - b).abs() <= tol * b.abs().max(1.0);
    assert!(perto(v("integral(x -> x^2, 0, 1)"), 1.0 / 3.0, 1e-14));
    assert!(perto(
        v("integral(x -> exp(-x^2), -Inf, Inf)"),
        std::f64::consts::PI.sqrt(),
        1e-10
    ));
    assert!(perto(v("integral(x -> 1 / sqrt(x), 0, 1)"), 2.0, 1e-8));
    assert!(perto(
        v("integral(x -> abs(x - 0.3), 0, 1; waypoints=[0.3])"),
        0.29,
        1e-14
    ));
    assert!(perto(
        v("integral(x -> sin(x), 0, pi; abs_tol=1e-12, rel_tol=1e-12)"),
        2.0,
        1e-12
    ));
    assert!(perto(v("integral(x -> x, 1, 0)"), -0.5, 1e-15));
    let (it, r) = roda("z = integral(x -> exp(1i * x), 0, pi)\nval, erro = integral(x -> x^3, 0, 2)\n");
    assert!(r.erro.is_none());
    let z = valor(&it, "z").to_complex()[0];
    assert!(z.re.abs() < 1e-14 && (z.im - 2.0).abs() < 1e-14, "{z}");
    assert!(perto(num(&it, "val")[0], 4.0, 1e-14) && num(&it, "erro")[0] >= 0.0);
    // Integrando com captura e função def.
    assert!(perto(
        vars(
            "k = 3;\ndef g(x):\n    return x^2\nr = integral(x -> k * g(x), 0, 1)\n",
            &["r"]
        )[0][0],
        1.0,
        1e-14
    ));
    // Aviso quando a tolerância não é garantida.
    let (_, r) = roda("r = integral(x -> 1 / x, 0, 1);");
    assert!(r.erro.is_none());
    assert!(
        r.texto().starts_with("aviso (chimalab:integral:"),
        "{}",
        r.texto()
    );
    // Raízes.
    assert_eq!(v("fzero(x -> x^2 - 2, [0 2])"), std::f64::consts::SQRT_2);
    assert!(perto(
        v("fzero(x -> cos(x) - x, 0.5)"),
        0.739_085_133_215_160_6,
        1e-15
    ));
    assert!(perto(v("fzero(x -> x^3 - 8, 1; tol_x=1e-12)"), 2.0, 1e-11));
    let r = vars("x, fx = fzero(x -> x - 1, [0 3])\n", &["x", "fx"]);
    assert_eq!((r[0][0], r[1][0]), (1.0, 0.0));
    // Mínimos.
    assert!(perto(v("fminbnd(x -> (x - 2)^2, 0, 5)"), 2.0, 1e-7));
    assert!(perto(v("fminbnd(x -> cos(x), 3, 4)"), std::f64::consts::PI, 1e-7));
    let r = vars(
        "x, fx = fminbnd(x -> (x - 1)^2 + 3, -2, 2; tol_x=1e-10)\n",
        &["x", "fx"],
    );
    assert!(perto(r[0][0], 1.0, 1e-9) && perto(r[1][0], 3.0, 1e-15));
    // map: forma e classe.
    let (it, r) = roda(
        "a = map(x -> x^2, [1 2; 3 4])\nb = map(x -> x > 2, 1:4)\nc = map(sqrt, [4 -1])\nd = map(x -> x, zeros(0, 3))\ne = map(x -> 'a', 1:3)\n",
    );
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    assert_eq!(num(&it, "a"), [1.0, 9.0, 4.0, 16.0]);
    assert_eq!(valor(&it, "a").dims(), [2, 2]);
    assert_eq!(valor(&it, "b").class().name(), "logical");
    assert_eq!(valor(&it, "c").to_complex()[1], Complex64::new(0.0, 1.0));
    assert_eq!(valor(&it, "d").dims(), [0, 3]);
    assert_eq!(valor(&it, "e"), Value::text("aaa"));
    // Erros: os da função do usuário saem com a posição de dentro dela.
    for (src, id) in [
        (
            "integral(x -> [x x], 0, 1)",
            "chimalab:execucao:resultadoNaoEscalar",
        ),
        ("integral(x -> nada, 0, 1)", "chimalab:execucao:nomeIndefinido"),
        ("integral(x -> x, NaN, 1)", "chimalab:integral:limitesInvalidos"),
        (
            "integral(x -> x, 0, 1; abs_tol=-1)",
            "chimalab:integral:toleranciaInvalida",
        ),
        ("integral(x -> x, 0, 1; tol=1)", "chimalab:execucao:argumentos"),
        ("integral(1, 0, 1)", "chimalab:execucao:argumentos"),
        ("integral(x -> x, 0)", "chimalab:execucao:argumentos"),
        ("integral(x -> x, [0 1], 1)", "chimalab:execucao:argumentos"),
        (
            "integral(x -> x, 0, 1; waypoints=x -> x)",
            "chimalab:execucao:argumentos",
        ),
        (
            "fzero(x -> x^2 + 1, [0 1])",
            "chimalab:otimizacao:semMudancaDeSinal",
        ),
        ("fzero(x -> x, [0 1 2])", "chimalab:execucao:argumentos"),
        ("fzero(x -> error('boom'), [0 1])", "chimalab:usuario:erro"),
        ("fzero(x -> 1i, [0 1])", "chimalab:execucao:resultadoNaoEscalar"),
        ("fzero(x -> x, 1; foo=2)", "chimalab:execucao:argumentos"),
        ("fminbnd(x -> x, 2, 1)", "chimalab:otimizacao:intervaloInvalido"),
        ("fminbnd(x -> nada, 0, 1)", "chimalab:execucao:nomeIndefinido"),
        ("map(x -> [x x], 1:2)", "chimalab:execucao:resultadoNaoEscalar"),
        ("map(x -> x, x -> x)", "chimalab:execucao:argumentos"),
        ("map(x -> x, 1; n=2)", "chimalab:execucao:argumentos"),
        ("map(def_nenhuma, 1)", "chimalab:execucao:nomeIndefinido"),
        ("map(x -> disp(x), 1)", "chimalab:execucao:semValor"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
    let (_, r) = roda("r = integral(x -> nada, 0, 1)");
    assert_eq!(
        r.erro.unwrap().coluna,
        19,
        "posição de `nada`, dentro do integrando"
    );
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 3000, ..ProptestConfig::default() })]
    /// Programas aleatórios montados de pedaços da linguagem: o interpretador nunca entra em
    /// pânico (erros com identificador são aceitos). Sem `while` e sem números grandes, para não
    /// depender de interrupção nem de memória.
    #[test]
    fn programas_aleatorios_nunca_entram_em_panico(partes in prop::collection::vec(0usize..64, 0..40)) {
        const V: [&str; 64] = [
            "x", "y", "1", "0", "2", "-1", "0.5", "[", "]", "(", ")", "+", "-", "*", "/", "\\", "^", ".*", ".^",
            ":", "end", "'", ",", ";", "\n", "\n    ", "if x:", "for k in ", "def f(a):", "return ", "x = ",
            "y = ", "==", "<", "and ", "or ", "not ", "!", "&", "f(", "zeros(", "x[", "1i", "\"a\"", "->", "sum(",
            "max(", "reshape(", "diag(", "sort(", "eig(", "inv(", "map(", "cumsum(", "find(", "try:", "except:",
            "match x:", "case 1:", "pass", "break", "A.T", "f\"{x}\"", "_",
        ];
        let src: String = partes.iter().map(|&k| V[k]).collect::<Vec<_>>().join(" ");
        let mut it = Interpretador::new();
        it.executa("x = [1 2; 3 4];\ny = 2;\nA = eye(2);");
        let r = it.executa(&src);
        if let Some(e) = r.erro {
            prop_assert!(e.id.starts_with("chimalab:"), "{}: {}", e.id, src);
            prop_assert!(e.linha >= 1 && e.coluna >= 1);
        }
        // O documento e o console aceitam o mesmo texto sem pânico.
        let _ = Interpretador::new().executa_documento(&src);
    }
}

fn expressao() -> impl Strategy<Value = String> {
    let folha = prop::sample::select(vec![
        "x",
        "y",
        "A",
        "1",
        "0",
        "-1",
        "0.5",
        "1i",
        "[]",
        "'ab'",
        "[1 2 3]",
        "(1:3)",
        "true",
        "NaN",
        "Inf",
        "int8(5)",
        "single(2)",
        "zeros(0, 2)",
        "[1; 2]",
        "(2 - 3i)",
        "uint8([3 4])",
    ])
    .prop_map(String::from);
    folha.prop_recursive(4, 40, 3, |e| {
        let un = prop::sample::select(vec!["-", "not ", "+"]);
        let bin = prop::sample::select(vec![
            "+", "-", "*", "/", "\\", "^", ".*", "./", ".^", "==", "!=", "<", ">=", "&", "|", "and", "or",
        ]);
        let f1 = prop::sample::select(vec![
            "sum", "max", "min", "abs", "sqrt", "exp", "log", "floor", "round", "sign", "sort", "find",
            "cumsum", "prod", "diag", "numel", "size", "isempty", "any", "all", "mean", "det", "inv", "eig",
            "svd", "norm", "rank", "trace", "expm", "pinv", "chol", "double", "logical", "char", "int16",
            "length", "cumprod",
        ]);
        prop_oneof![
            (un, e.clone()).prop_map(|(u, a)| format!("{u}({a})")),
            (e.clone(), bin, e.clone()).prop_map(|(a, o, b)| format!("({a} {o} {b})")),
            (f1, e.clone()).prop_map(|(f, a)| format!("{f}({a})")),
            e.clone().prop_map(|a| format!("({a})'")),
            e.clone().prop_map(|a| format!("({a}).T")),
            (e.clone(), e.clone()).prop_map(|(a, b)| format!("[{a}, {b}]")),
            (e.clone(), e.clone()).prop_map(|(a, b)| format!("[{a}; {b}]")),
            e.clone().prop_map(|a| format!("x[{a}]")),
            e.clone().prop_map(|a| format!("({a})[end]")),
            e.clone().prop_map(|a| format!("mod({a}, 2)")),
            e.clone().prop_map(|a| format!("reshape({a}, -1, 1)")),
            e.clone().prop_map(|a| format!("map(t -> t * 2, {a})")),
            e.clone().prop_map(|a| format!("f\"{{{a}}}\"")),
            (e.clone(), e).prop_map(|(a, b)| format!("max({a}, {b})")),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 2000, ..ProptestConfig::default() })]
    /// Expressões bem formadas aleatórias sobre valores de todas as classes: nunca há pânico, e
    /// todo erro tem identificador `chimalab:` e posição válida. Exercita conversões, expansão
    /// implícita, índices, álgebra linear e exibição de resultados estranhos.
    #[test]
    fn expressoes_aleatorias_nunca_entram_em_panico(e in expressao()) {
        let mut it = Interpretador::new();
        it.executa("x = [1 2; 3 4];\ny = 2;\nA = [2 1; 1 3];");
        let r = it.executa(&format!("r = {e}\nr\nf\"{{r}}\""));
        if let Some(err) = &r.erro {
            prop_assert!(err.id.starts_with("chimalab:"), "{}: {}", err.id, e);
            prop_assert!(err.linha >= 1 && err.coluna >= 1);
        }
        // A exibição de qualquer resultado não entra em pânico.
        let _ = r.texto();
    }
}

/// Elementares e partes de complexos contra valores exatos e identidades.
#[test]
fn elementares_e_complexos() {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, LN_2, LN_10, PI};
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    let z = |src: &str| {
        let (it, r) = roda(&format!("r = {src}\n"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        valor(&it, "r").to_complex()
    };
    assert_eq!(v("fix([-1.5 1.5 -0.5])"), [-1.0, 1.0, -0.0]);
    assert_eq!(v("log10(1000) + log2(8)"), [6.0]);
    assert_eq!(v("[sinh(0) cosh(0) tanh(0) tanh(Inf)]"), [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(v("[asin(1) acos(-1) atan(1)]"), [FRAC_PI_2, PI, FRAC_PI_4]);
    // Fora do domínio real: complexos no ramo principal.
    let w = z("log10(-10)");
    assert!((w[0].re - 1.0).abs() < 1e-15 && (w[0].im - PI / LN_10).abs() < 1e-15);
    assert!((z("log2(-1)")[0].im - PI / LN_2).abs() < 1e-15);
    let a = z("asin(2)")[0];
    assert!((a.re - FRAC_PI_2).abs() < 1e-15 && (a.im.abs() - (2.0 + 3f64.sqrt()).ln()).abs() < 1e-15);
    assert!((z("sin(asin(2))")[0] - Complex64::new(2.0, 0.0)).norm() < 1e-14);
    assert!(z("acos(-3)")[0].im != 0.0);
    assert!((z("cosh(1i)")[0].re - 1f64.cos()).abs() < 1e-15);
    let t = z("[sinh(1i) tanh(1i) asin(0.5i) acos(1i) atan(0.5i) log10(1i) log2(1i)]");
    assert!(t.iter().all(|x| x.re.is_finite() && x.im.is_finite()));
    assert_eq!(z("fix(2.7 - 1.2i)")[0], Complex64::new(2.0, -1.0));
    let (it, _) = roda("a = fix(int8(-3))\nb = imag(int8(5))\nc = imag(single(2))\n");
    assert_eq!(valor(&it, "a").class().name(), "int8");
    assert_eq!(valor(&it, "b").class().name(), "int8");
    assert_eq!(valor(&it, "c").class().name(), "single");
    // Testes de valor.
    assert_eq!(v("isnan([1 NaN Inf -Inf])"), [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(v("isinf([1 NaN Inf -Inf])"), [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(v("isfinite([1 NaN Inf -Inf])"), [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(v("isnan([1+2i, NaN])"), [0.0, 1.0]);
    assert_eq!(v("isinf([1+2i, Inf*1i])"), [0.0, 1.0]);
    assert_eq!(
        v("[isnan(int8(1)) isinf('a') isfinite(true) isfinite(int8([1 2]))]"),
        [0.0, 0.0, 1.0, 1.0, 1.0]
    );
    assert_eq!(v("[isreal(1i) isreal(1) isreal(1i * 1i)]"), [0.0, 1.0, 1.0]);
    // Partes de complexos.
    assert_eq!(v("[real(3+4i) imag(3+4i) imag(5) real(2)]"), [3.0, 4.0, 0.0, 2.0]);
    assert_eq!(z("conj(3+4i)"), [Complex64::new(3.0, -4.0)]);
    assert_eq!(v("conj(2)"), [2.0]);
    assert_eq!(
        v("[angle(1i) angle(-1) angle(1) angle(0)]"),
        [FRAC_PI_2, PI, 0.0, 0.0]
    );
    // Binárias.
    assert_eq!(
        v("[atan2(1, 1) atan2(1, -1) atan2(-1, -1)]"),
        [FRAC_PI_4, 3.0 * FRAC_PI_4, -3.0 * FRAC_PI_4]
    );
    assert_eq!(v("hypot(3, 4)"), [5.0]);
    assert!(
        (v("hypot(1e300, 1e300)")[0] / (1e300 * 2f64.sqrt()) - 1.0).abs() < 1e-15,
        "sem estouro"
    );
    assert_eq!(v("rem([5 -5 5 -5], [3 3 -3 -3])"), [2.0, -2.0, 2.0, -2.0]);
    assert!(v("rem(5, 0)")[0].is_nan());
    assert_eq!(v("atan2([0 1], 1)"), [0.0, FRAC_PI_4]);
    // class.
    let (it, _) = roda(
        "a = class(1)\nb = class('x')\nc = class(int8(1))\nd = class(x -> x)\ne = class(true)\nf = class(1i)\ng = class(sin)\n",
    );
    for (k, esperado) in [
        ("a", "double"),
        ("b", "char"),
        ("c", "int8"),
        ("d", "function"),
        ("e", "logical"),
        ("f", "double"),
        ("g", "function"),
    ] {
        assert_eq!(valor(&it, k), Value::text(esperado), "{k}");
    }
    for (src, id) in [
        ("exp(int8(1))", "chimalab:execucao:classeNaoSuportada"),
        ("log10(int8(1))", "chimalab:execucao:classeNaoSuportada"),
        ("atan2(1i, 1)", "chimalab:execucao:classeNaoSuportada"),
        ("rem([1 2], [1 2 3])", "chimalab:runtime:dimensoesIncompativeis"),
        ("isnan()", "chimalab:execucao:argumentos"),
        ("isreal(1, 2)", "chimalab:execucao:argumentos"),
        ("real(x -> x)", "chimalab:execucao:funcaoComoValor"),
        ("class()", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// `ode` (F049 pela linguagem; nome provisório, pergunta 53) contra soluções exatas.
#[test]
fn ode_pela_linguagem() {
    // y' = −y: y(1) = e⁻¹.
    let (it, r) = roda("t, y = ode((t, y) -> -y, [0 1], 1; rel_tol=1e-10, abs_tol=1e-12)\n");
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    let y = num(&it, "y");
    assert!((y.last().unwrap() - (-1f64).exp()).abs() < 1e-9);
    assert_eq!(*num(&it, "t").last().unwrap(), 1.0);
    assert_eq!(valor(&it, "t").dims()[1], 1, "t em coluna");
    // Oscilador harmônico como sistema: volta ao início depois de 2π; saída nos instantes pedidos.
    let (it, r) =
        roda("t, y = ode((t, y) -> [y[2]; -y[1]], 0:pi/2:2*pi, [1 0]; rel_tol=1e-10, abs_tol=1e-12);\n");
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    assert_eq!(
        valor(&it, "y").dims(),
        [5, 2],
        "uma linha por instante, uma coluna por componente"
    );
    let y = num(&it, "y");
    // Coluna 1: cos t nos instantes 0, π/2, π, 3π/2, 2π.
    for (k, esperado) in [1.0, 0.0, -1.0, 0.0, 1.0].iter().enumerate() {
        assert!((y[k] - esperado).abs() < 1e-8, "k = {k}: {}", y[k]);
    }
    assert_eq!(num(&it, "t").len(), 5);
    // Problema rígido y' = −λ(y − cos t), y(0) = 0, com solução fechada; os quatro implícitos.
    let lambda: f64 = 1000.0;
    let exata = (lambda * lambda * 1f64.cos() + lambda * 1f64.sin()) / (lambda * lambda + 1.0)
        - lambda * lambda / (lambda * lambda + 1.0) * (-lambda).exp();
    for m in ["ndf", "bdf", "radau", "rosenbrock"] {
        let (it, r) = roda(&format!(
            "t, y = ode((t, y) -> -1000 * (y - cos(t)), [0 1], 0; method=\"{m}\", rel_tol=1e-8, abs_tol=1e-10);\n"
        ));
        assert!(r.erro.is_none(), "{m}: {}", r.erro.unwrap());
        let y = num(&it, "y");
        assert!(
            (y.last().unwrap() - exata).abs() < 1e-5,
            "{m}: {} ≠ {exata}",
            y.last().unwrap()
        );
        assert!(!r.texto().contains("rigidezSuspeita"), "{m}");
    }
    // O explícito avisa da rigidez; bs23 resolve um problema simples.
    // (λ = 10⁴: com 10³ o DOPRI5 ainda anda perto do limite de estabilidade sem disparar a detecção.)
    let (_, r) = roda("t, y = ode((t, y) -> -1e4 * (y - cos(t)), [0 1], 0);\n");
    assert!(
        r.texto().contains("chimalab:ode:rigidezSuspeita"),
        "{}",
        r.texto()
    );
    let (it, _) = roda("t, y = ode((t, y) -> 2 * t, [0 2], 0; method=\"bs23\", max_step=0.1);\n");
    assert!((num(&it, "y").last().unwrap() - 4.0).abs() < 1e-9);
    // Tolerância relativa pequena demais é elevada, com aviso.
    let (_, r) = roda("t, y = ode((t, y) -> -y, [0 1], 1; rel_tol=1e-20);\n");
    assert!(
        r.texto().contains("chimalab:ode:toleranciaElevada"),
        "{}",
        r.texto()
    );
    // Singularidade: y' = y², y(0) = 1 explode em t = 1; resultado parcial com aviso.
    let (it, r) = roda("t, y = ode((t, y) -> y^2, [0 2], 1);\n");
    assert!(r.erro.is_none());
    assert!(r.texto().contains("chimalab:ode:"), "{}", r.texto());
    assert!(*num(&it, "t").last().unwrap() < 1.0 + 1e-3);
    // Erros.
    for (src, id) in [
        ("ode((t, y) -> -y, [0], 1)", "chimalab:ode:intervaloInvalido"),
        (
            "ode((t, y) -> -y, [0 1], zeros(1, 0))",
            "chimalab:ode:condicaoInicialInvalida",
        ),
        (
            "ode((t, y) -> -y, [0 1], 1; rel_tol=-1)",
            "chimalab:ode:toleranciaInvalida",
        ),
        (
            "ode((t, y) -> [y; y], [0 1], 1)",
            "chimalab:execucao:derivadaComTamanhoErrado",
        ),
        (
            "ode((t, y) -> nada, [0 1], 1)",
            "chimalab:execucao:nomeIndefinido",
        ),
        ("ode(t -> t, [0 1], 1)", "chimalab:execucao:argumentos"),
        (
            "ode((t, y) -> -y, [0 1], 1; method=\"rk4\")",
            "chimalab:execucao:argumentos",
        ),
        (
            "ode((t, y) -> -y, [0 1], 1; method=1)",
            "chimalab:execucao:argumentos",
        ),
        (
            "ode((t, y) -> -y, [0 1], 1; passo=1)",
            "chimalab:execucao:argumentos",
        ),
        ("ode((t, y) -> -y, [0 1])", "chimalab:execucao:argumentos"),
        ("ode(1, [0 1], 1)", "chimalab:execucao:argumentos"),
        ("ode((t, y) -> x -> x, [0 1], 1)", "chimalab:execucao:semValor"),
    ] {
        let (_, r) = roda(&format!("t, y = {src};"));
        let e = r.erro.unwrap_or_else(|| panic!("{src} deveria falhar"));
        assert_eq!(e.id, id, "{src}: {}", e.mensagem);
    }
}

/// Critérios de aceitação do F006 (controle de fluxo), na sintaxe do DT-39.
#[test]
fn f006_ca1_for_sobre_matriz_itera_colunas() {
    let (it, r) = roda("n = 0;\nfor k in [1 2; 3 4]:\n    n += 1;\n    ultima = k;\n");
    assert!(r.erro.is_none());
    assert_eq!(num(&it, "n"), [2.0], "duas iterações");
    assert_eq!(num(&it, "ultima"), [2.0, 4.0]);
    assert_eq!(
        valor(&it, "ultima").dims(),
        [2, 1],
        "cada iteração recebe uma coluna"
    );
    // Caso especial: alterar a variável do laço no corpo não muda as iterações seguintes.
    let (it, _) = roda("s = 0;\nfor k in 1:4:\n    s += k;\n    k = 100;\n");
    assert_eq!(num(&it, "s"), [10.0]);
    let (it, _) = roda("s = 0;\nfor k in [1 2 3]:\n    s += k;\n    k = 100;\n");
    assert_eq!(num(&it, "s"), [6.0]);
    // Matriz vazia: nenhuma iteração.
    assert_eq!(vars("n = 0;\nfor k in []:\n    n += 1;\n", &["n"])[0], [0.0]);
}

#[test]
fn f006_ca2_if_vazio_e_falso() {
    assert_eq!(vars("y = 0\nif []:\n    y = 1\n", &["y"])[0], [0.0]);
    // `not []` é um vazio lógico: também falso.
    assert_eq!(vars("y = 0\nif not []:\n    y = 1\n", &["y"])[0], [0.0]);
    assert_eq!(
        vars("y = 0\nif zeros(0, 3):\n    y = 1\nelif 1:\n    y = 2\n", &["y"])[0],
        [2.0]
    );
}

#[test]
fn f006_ca3_match_casa_texto_entre_alternativas() {
    let src = "def qual(s):\n    match s:\n        case 'x', 'abc':\n            return 1\n        case _:\n            return 0\na = qual('abc')\nb = qual('x')\nc = qual('ab')\nd = qual(\"abc\")\n";
    assert_eq!(
        vars(src, &["a", "b", "c", "d"]),
        [vec![1.0], vec![1.0], vec![0.0], vec![1.0]]
    );
    // Erros do catálogo: `break` fora de laço; `except` sem identificador é aceito.
    assert_eq!(erro("break\n").0, "chimalab:sintaxe:foraDeLaco");
    assert_eq!(
        vars("try:\n    error('x')\nexcept:\n    ok = 1\n", &["ok"])[0],
        [1.0]
    );
}

/// Arrays grandes aparecem resumidos (mais de 1000 elementos): 3 primeiras e 3 últimas linhas
/// e colunas; o cabeçalho mantém o tamanho completo. A exibição de 10⁶ elementos é rápida e curta.
#[test]
fn exibicao_resumida_de_arrays_grandes() {
    let t = console("A = reshape(1:2000, 40, 50)");
    let linhas: Vec<&str> = t.lines().collect();
    assert_eq!(linhas.len(), 8);
    assert_eq!(linhas[0], "A = 40×50");
    assert_eq!(linhas[1], "     1    41    81     …  1881  1921  1961");
    assert_eq!(linhas[4], "     ⋮     ⋮     ⋮     ⋱     ⋮     ⋮     ⋮");
    assert_eq!(linhas[7], "    40    80   120     …  1920  1960  2000");
    assert_eq!(
        console("v = 1:2000"),
        "v = 1×2000\n     1     2     3     …  1998  1999  2000"
    );
    assert_eq!(console("v = (1:2000)'").lines().count(), 8);
    // Seis colunas não se resumem (o resumo mostra 3 + … + 3); as linhas sim.
    let t = console("A = reshape(1:1200, 200, 6)");
    assert_eq!(t.lines().nth(1).unwrap().split_whitespace().count(), 6, "{t}");
    assert!(!t.lines().nth(1).unwrap().contains('…'));
    assert_eq!(t.lines().count(), 8);
    // Exatamente no limite (1000 elementos): completo.
    assert_eq!(
        console("v = 1:1000")
            .lines()
            .nth(1)
            .unwrap()
            .split_whitespace()
            .count(),
        1000
    );
    // N-D: fatias resumidas com "…" entre elas.
    let t = console("B = ones(10, 10, 20)");
    assert_eq!(t.lines().filter(|l| l.trim() == "…").count(), 1);
    assert_eq!(
        t.lines().filter(|l| l.trim_start().starts_with("[:, :,")).count(),
        6
    );
    assert!(t.contains("[:, :, 20]") && !t.contains("[:, :, 4]"));
    // Texto alto: linhas resumidas.
    let t = console("s = char(65 + mod(reshape(0:2999, 1000, 3), 26))");
    assert_eq!(t.lines().count(), 8);
    assert!(t.lines().nth(4).unwrap().trim() == "⋮");
    // 10⁶ elementos: rápido e curto.
    let t0 = std::time::Instant::now();
    let t = console("Z = zeros(1000)");
    assert!(
        t.lines().count() == 8 && t0.elapsed() < std::time::Duration::from_secs(2),
        "{:?}",
        t0.elapsed()
    );
    // disp também resume.
    assert_eq!(
        console("disp(1:2000)"),
        "1×2000\n     1     2     3     …  1998  1999  2000"
    );
}

/// `help` vem da especificação, e toda função embutida está documentada nela.
#[test]
fn ajuda_cobre_todas_as_embutidas() {
    let mut faltam = Vec::new();
    for nome in chimalab_runtime::embutidas() {
        let t = console(&format!("help(\"{nome}\")"));
        if t.starts_with("sem ajuda") || !t.contains(nome) {
            faltam.push(nome);
        }
    }
    assert!(faltam.is_empty(), "sem ajuda na especificação: {faltam:?}");
    assert!(console("help(det)").contains("determinante"));
    assert!(console("help()").contains("6.6 Álgebra linear"));
    assert_eq!(console("help('nada_disso')"), "sem ajuda para `nada_disso`");
    assert_eq!(
        console("def f(a, b=2):\n    return a\nhelp(f)"),
        "função f(a, b=…)"
    );
    assert_eq!(console("help(x -> x)"), "função anônima (x)");
    assert_eq!(erro("help(1)").0, "chimalab:execucao:argumentos");
    assert_eq!(erro("help(1, 2)").0, "chimalab:execucao:argumentos");
}

/// Critérios do F020: unique mantém os NaN, sort estável, ismember com índices; mais conjuntos.
#[test]
fn f020_ca1_unique_mantem_os_nan() {
    let (it, r) = roda("u = unique([3 1 NaN NaN])\n");
    assert!(r.erro.is_none());
    let u = num(&it, "u");
    assert_eq!(u.len(), 4);
    assert_eq!(u[..2], [1.0, 3.0]);
    assert!(u[2].is_nan() && u[3].is_nan(), "os dois NaN ficam");
    let r = vars("u, i, j = unique([3 1 3 2 1])\n", &["u", "i", "j"]);
    assert_eq!(
        r,
        [
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 1.0],
            vec![3.0, 1.0, 3.0, 2.0, 1.0]
        ]
    );
    // Coluna para o que não é vetor linha; classe preservada; −0 igual a +0.
    let (it, _) = roda(
        "a = unique([2 1; 1 2])\nb = unique(int8([5 5 -1]))\nc = unique('banana')\nd = unique([0 -0])\ne = unique([])\n",
    );
    assert_eq!(valor(&it, "a").dims(), [2, 1]);
    assert_eq!(valor(&it, "b").class().name(), "int8");
    assert_eq!(valor(&it, "c"), Value::text("abn"));
    assert_eq!(num(&it, "d").len(), 1);
    assert_eq!(valor(&it, "e").numel(), 0);
}

#[test]
fn f020_ca2_sort_estavel() {
    // Iguais mantêm a ordem de chegada, nas duas ordens.
    let r = vars(
        "s, i = sort([2 1 2 1 2])\nt, k = sort([2 1 2 1 2], 'descend')\n",
        &["i", "k"],
    );
    assert_eq!(r, [vec![2.0, 4.0, 1.0, 3.0, 5.0], vec![1.0, 3.0, 5.0, 2.0, 4.0]]);
}

#[test]
fn f020_ca3_ismember_retorna_indices() {
    let r = vars("tf, loc = ismember([1 5 3 NaN], [3 1 1])\n", &["tf", "loc"]);
    assert_eq!(
        r,
        [vec![1.0, 0.0, 1.0, 0.0], vec![2.0, 0.0, 1.0, 0.0]],
        "primeira ocorrência; NaN nunca é membro"
    );
    let (it, _) = roda("tf = ismember([1 2; 3 4], [4 1])\nm = ismember(int8(3), [1 2 3])\n");
    assert_eq!(valor(&it, "tf").dims(), [2, 2]);
    assert_eq!(valor(&it, "tf").class().name(), "logical");
    assert_eq!(num(&it, "m"), [1.0]);
    // Conjuntos.
    assert_eq!(vars("a = union([3 1], [2 1])\n", &["a"])[0], [1.0, 2.0, 3.0]);
    assert_eq!(
        vars("a = intersect([3 1 2 NaN], [2 3 9 NaN])\n", &["a"])[0],
        [2.0, 3.0]
    );
    let d = vars("a = setdiff([3 1 NaN 2 1], 2)\n", &["a"]).remove(0);
    assert_eq!(d[..2], [1.0, 3.0]);
    assert!(d[2].is_nan() && d.len() == 3);
    let (it, _) =
        roda("a = union([1; 2], [3; 1])\nb = intersect([1; 2], [2 3])\nc = setdiff(int8([1 2 3]), [2])\n");
    assert_eq!(valor(&it, "a").dims(), [3, 1], "duas colunas dão coluna");
    assert_eq!(valor(&it, "b").dims(), [1, 1]);
    assert_eq!(valor(&it, "c").class().name(), "int8");
    assert_eq!(
        erro("u = unique([1i 2])").0,
        "chimalab:execucao:classeNaoSuportada"
    );
    assert_eq!(erro("u = ismember(1)").0, "chimalab:execucao:argumentos");
}

proptest! {
    /// Conjuntos contra a mesma conta em Rust (BTreeSet) com inteiros pequenos.
    #[test]
    fn conjuntos_contra_btreeset(a in prop::collection::vec(-5i32..6, 0..12), b in prop::collection::vec(-5i32..6, 0..12)) {
        use std::collections::BTreeSet;
        let lit = |v: &[i32]| if v.is_empty() { "zeros(1, 0)".to_string() } else { format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")) };
        let src = format!("a = {};\nb = {};\nu = union(a, b)\ni = intersect(a, b)\nd = setdiff(a, b)\nq = unique(a)\nm = ismember(a, b)\n", lit(&a), lit(&b));
        let mut it = Interpretador::new();
        let r = it.executa(&src);
        prop_assert!(r.erro.is_none(), "{}", r.erro.unwrap());
        let sa: BTreeSet<i32> = a.iter().copied().collect();
        let sb: BTreeSet<i32> = b.iter().copied().collect();
        let f = |s: BTreeSet<i32>| s.into_iter().map(f64::from).collect::<Vec<_>>();
        prop_assert_eq!(num(&it, "u"), f(sa.union(&sb).copied().collect()));
        prop_assert_eq!(num(&it, "i"), f(sa.intersection(&sb).copied().collect()));
        prop_assert_eq!(num(&it, "d"), f(sa.difference(&sb).copied().collect()));
        prop_assert_eq!(num(&it, "q"), f(sa.clone()));
        prop_assert_eq!(num(&it, "m"), a.iter().map(|x| if sb.contains(x) { 1.0 } else { 0.0 }).collect::<Vec<_>>());
    }
}

/// Regressão: chaves de classes diferentes não se comparavam (`ismember(int8(3), [1 2 3])` era
/// falso). Inteiros grandes, sem representação exata em double, continuam distintos.
#[test]
fn conjuntos_entre_classes() {
    assert_eq!(vars("m = ismember(int8(3), [1 2 3])\n", &["m"])[0], [1.0]);
    assert_eq!(vars("m = ismember(3, int16([1 3]))\n", &["m"])[0], [1.0]);
    assert_eq!(vars("m = intersect(int8([1 2 3]), [3 4])\n", &["m"])[0], [3.0]);
    assert_eq!(vars("m = setdiff(int8([1 2 3]), [3 4])\n", &["m"])[0], [1.0, 2.0]);
    assert_eq!(vars("m = ismember('b', [97 98])\n", &["m"])[0], [1.0]);
    assert_eq!(vars("m = ismember(true, [0 1])\n", &["m"])[0], [1.0]);
    // 2⁵³ + 1 em int64 não é igual a nenhum double (2⁵³ + 1 vira 2⁵³ em double).
    let src = "x = int64(9007199254740992) + 1;\nm = ismember(x, 9007199254740992)\nn = ismember(x, x)\nu = unique([x, x - 1])\n";
    let (it, r) = roda(src);
    assert!(r.erro.is_none(), "{}", r.erro.unwrap());
    assert_eq!(num(&it, "m"), [0.0]);
    assert_eq!(num(&it, "n"), [1.0]);
    assert_eq!(valor(&it, "u").numel(), 2);
}

/// F013 (workspace): who, whos (tamanho, classe e bytes, critério do catálogo) e clear.
#[test]
fn f013_ca3_whos_lista_tamanho_classe_e_bytes() {
    let base = "x = 1; A = zeros(3, 4); s = 'olá'; f = t -> t; z = int8([1 2]); c = 1i; u = uint16(7); l = true; g = single([1 2 3]);\n";
    assert_eq!(console(&format!("{base}who()")), "A  c  f  g  l  s  u  x  z");
    let t = console(&format!("{base}whos()"));
    let linhas: Vec<&str> = t.lines().collect();
    assert_eq!(linhas[0], "nome  tamanho  classe    bytes");
    assert_eq!(linhas[1], "A     3×4      double       96");
    assert_eq!(linhas[2], "c     1×1      double       16");
    assert_eq!(linhas[3], "f     1×1      function      —");
    assert!(t.contains("g     1×3      single       12"));
    assert!(t.contains("l     1×1      logical       1"));
    assert!(t.contains("s     1×3      char         12"));
    assert!(t.contains("u     1×1      uint16        2"));
    assert!(t.contains("z     1×2      int8          2"));
    // clear: nomes dados (inexistente é ignorado) ou tudo.
    let mut it = Interpretador::new();
    it.executa(base);
    it.executa("clear('A', 'nada', 'z')");
    assert_eq!(it.executa("who()").texto(), "c  f  g  l  s  u  x");
    it.executa("clear()");
    assert_eq!(it.executa("who()").texto(), "");
    assert!(it.nomes().is_empty());
    // Dentro de uma função, o quadro é o da função.
    assert_eq!(
        console(
            "a = 1;\ndef g(p):\n    q = 2;\n    who()\n    clear()\n    who()\n    return 0\ng(5);\nwho()"
        ),
        "p  q\na  ans"
    );
    assert_eq!(erro("clear(1)").0, "chimalab:execucao:argumentos");
    assert_eq!(erro("who(1)").0, "chimalab:execucao:argumentos");
}

/// Nome indefinido sugere o nome conhecido mais próximo (variável, `def` ou embutida).
#[test]
fn sugestao_para_nome_indefinido() {
    let msg = |src: &str| roda(src).1.erro.unwrap().mensagem;
    assert_eq!(
        msg("y = summ([1 2])"),
        "`summ` não está definido; você quis dizer `sum`?"
    );
    // `sun` está a uma edição de `sin` e de `sum`: vale a ordem alfabética.
    assert_eq!(
        msg("y = sun([1 2])"),
        "`sun` não está definido; você quis dizer `sin`?"
    );
    assert_eq!(
        msg("valor = 3;\ny = valr"),
        "`valr` não está definido; você quis dizer `valor`?"
    );
    assert_eq!(
        msg("def dobro(x):\n    return 2 * x\ny = dobr(2)"),
        "`dobr` não está definido; você quis dizer `dobro`?"
    );
    assert_eq!(msg("y = xyz"), "`xyz` não está definido");
    // Troca de letras vizinhas custa 1 (Damerau): `medai` → `media`, não `mean`.
    assert_eq!(
        msg("media = 3;\ny = medai"),
        "`medai` não está definido; você quis dizer `media`?"
    );
    assert_eq!(
        msg("y = sm([1 2])"),
        "`sm` não está definido; você quis dizer `sum`?"
    );
    // Nomes curtos aceitam só uma edição; longos, duas.
    assert_eq!(msg("abcd = 1;\ny = abxy"), "`abxy` não está definido");
    assert_eq!(
        msg("abcdef = 1;\ny = abcdxy"),
        "`abcdxy` não está definido; você quis dizer `abcdef`?"
    );
    // Empate: o primeiro em ordem alfabética (`fix`, `max` e `min` estão a uma edição).
    assert_eq!(
        msg("y = mix(1)"),
        "`mix` não está definido; você quis dizer `fix`?"
    );
}

/// Estatística descritiva (F043) pela linguagem, contra valores exatos.
#[test]
fn estatistica_pela_linguagem() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    assert_eq!(v("median([3 1 2])"), [2.0]);
    assert_eq!(v("median([4 1 2 3])"), [2.5]);
    assert_eq!(v("median([1 3; 2 5; 9 4])"), [2.0, 4.0]);
    assert_eq!(v("median([1 3; 2 5], 2)"), [2.0, 3.5]);
    assert!(v("median([1 NaN 3])")[0].is_nan(), "NaN propaga");
    assert!(v("median([])")[0].is_nan());
    assert_eq!(v("mode([1 2 2 3 3])"), [2.0], "empate: o menor");
    assert_eq!(v("mode([5 NaN NaN])"), [5.0], "NaN é ignorado");
    assert_eq!(v("var([1 2 3 4])"), [5.0 / 3.0]);
    assert_eq!(v("var([1 2 3 4]; population=true)"), [1.25]);
    assert_eq!(v("std([2 4 4 4 5 5 7 9]; population=true)"), [2.0]);
    assert_eq!(v("std([1 3; 1 3])"), [0.0, 0.0]);
    assert_eq!(v("var([1 2; 3 4], 2)"), [0.5, 0.5]);
    assert_eq!(v("var(5)"), [0.0]);
    assert_eq!(v("quantile([1 2 3 4], 0.5)"), [2.5]);
    assert_eq!(v("quantile(1:5, [0 0.25 1])"), [1.0, 2.0, 5.0]);
    assert_eq!(v("quantile([1 2 3 4], 0.5; type=1)"), [2.0]);
    let (it, _) = roda("q = quantile(1:5, [0.1; 0.9])\ns = median(single([1 2]))\n");
    assert_eq!(valor(&it, "q").dims(), [2, 1], "forma de p");
    assert_eq!(valor(&it, "s").class().name(), "single");
    for (src, id) in [
        ("median([1i 2])", "chimalab:execucao:classeNaoSuportada"),
        ("quantile([1i 2], 0.5)", "chimalab:execucao:classeNaoSuportada"),
        ("quantile(1:4, 1.5)", "chimalab:stats:opcaoInvalida"),
        ("quantile(1:4, 0.5; type=10)", "chimalab:stats:opcaoInvalida"),
        ("quantile(1:4, 0.5; tipo=1)", "chimalab:execucao:argumentos"),
        ("var(1:4; ddof=0)", "chimalab:execucao:argumentos"),
        ("median(1:4; population=true)", "chimalab:execucao:argumentos"),
        ("std(1:4, 0)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// Polinômios (F045), FFT e convolução (F046), interpolação 1-D (F044) pela linguagem.
#[test]
fn polinomios_fft_e_interpolacao() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    let z = |src: &str| {
        let (it, r) = roda(&format!("r = {src}\n"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        valor(&it, "r").to_complex()
    };
    // Polinômios (coeficientes em potências decrescentes).
    assert_eq!(v("polyval([1 2 3], 2)"), [11.0]);
    let (it, _) = roda("r = polyval([1 0 -1], [1 2; 3 4])\n");
    assert_eq!(
        (num(&it, "r"), valor(&it, "r").dims().to_vec()),
        (vec![0.0, 8.0, 3.0, 15.0], vec![2, 2])
    );
    assert_eq!(
        v("polyval([1 0 1], 1i)"),
        [0.0],
        "complexo com parte imaginária nula volta a real"
    );
    let mut r = v("roots([1 -3 2])");
    r.sort_by(f64::total_cmp);
    assert!((r[0] - 1.0).abs() < 1e-14 && (r[1] - 2.0).abs() < 1e-14);
    let mut ri = z("roots([1 0 1])");
    ri.sort_by(|a, b| a.im.total_cmp(&b.im));
    assert!(
        (ri[0] - Complex64::new(0.0, -1.0)).norm() < 1e-14
            && (ri[1] - Complex64::new(0.0, 1.0)).norm() < 1e-14
    );
    assert_eq!(v("roots([0 0 1 -1])"), [1.0], "zeros à esquerda ignorados");
    assert!(v("roots(5)").is_empty());
    let p = v("polyfit([0 1 2 3], [1 3 5 7], 1)");
    assert!((p[0] - 2.0).abs() < 1e-14 && (p[1] - 1.0).abs() < 1e-14);
    let r = vars("p, nr = polyfit([0 1 2], [0 1 0], 1)\n", &["nr"]).remove(0);
    assert!((r[0] - (2.0f64 / 3.0).sqrt()).abs() < 1e-14, "{r:?}");
    let (_, res) = roda("p = polyfit([1 2 3], [1 2 3], 5);");
    assert!(
        res.texto().contains("chimalab:runtime:matrizSingular"),
        "{}",
        res.texto()
    );
    // Convolução.
    assert_eq!(v("conv([1 2], [1 3])"), [1.0, 5.0, 6.0]);
    assert_eq!(v("conv([1 2 3], [1 1]; shape=\"same\")"), [3.0, 5.0, 3.0]);
    assert_eq!(v("conv([1 2 3], [1 1]; shape=\"valid\")"), [3.0, 5.0]);
    let (it, _) = roda("c = conv([1; 2], [1 3])\nd = conv([1 1i], [1 1])\n");
    assert_eq!(valor(&it, "c").dims(), [3, 1]);
    assert_eq!(valor(&it, "d").to_complex()[1], Complex64::new(1.0, 1.0));
    // FFT.
    assert_eq!(v("fft([1 0 0 0])"), [1.0, 1.0, 1.0, 1.0]);
    let f = z("fft([1 2 3 4])");
    let esperado = [
        Complex64::new(10.0, 0.0),
        Complex64::new(-2.0, 2.0),
        Complex64::new(-2.0, 0.0),
        Complex64::new(-2.0, -2.0),
    ];
    assert!(
        f.iter().zip(esperado).all(|(a, b)| (a - b).norm() < 1e-14),
        "{f:?}"
    );
    assert!(
        v("abs(ifft(fft([3 1 4 1 5])) - [3 1 4 1 5])")
            .iter()
            .all(|e| *e < 1e-14)
    );
    let (it, _) = roda("a = fft([1 2], 4)\nb = fft([1 2; 3 4], [], 2)\nc = fft([1 2; 3 4])\n");
    assert_eq!(valor(&it, "a").dims(), [1, 4]);
    assert_eq!(num(&it, "b"), [3.0, 7.0, -1.0, -1.0]);
    assert_eq!(num(&it, "c"), [4.0, -2.0, 6.0, -2.0]);
    // Interpolação 1-D.
    assert_eq!(v("interp1([1 2 3], [10 20 30], 2.5)"), [25.0]);
    assert!(
        v("interp1([1 2 3], [10 20 30], 4)")[0].is_nan(),
        "fora: NaN por padrão"
    );
    assert_eq!(
        v("interp1([1 2 3], [10 20 30], 4; extrap=\"extrapolate\")"),
        [40.0]
    );
    assert_eq!(v("interp1([1 2 3], [10 20 30], [0 4]; extrap=-1)"), [-1.0, -1.0]);
    assert_eq!(
        v("interp1([1 2 3], [10 20 30], [0 4]; extrap=\"clamp\")"),
        [10.0, 30.0]
    );
    assert_eq!(
        v("interp1([1 2 3], [10 20 30], [1.4 1.6]; method=\"nearest\")"),
        [10.0, 20.0]
    );
    assert_eq!(
        v("interp1([1 2 3], [10 20 30], 1.5; method=\"previous\")"),
        [10.0]
    );
    assert_eq!(v("interp1([1 2 3], [10 20 30], 1.5; method=\"next\")"), [20.0]);
    // Spline not-a-knot reproduz cúbicas exatamente.
    assert!((v("interp1(0:4, (0:4).^3, 2.5; method=\"spline\")")[0] - 15.625).abs() < 1e-12);
    for m in ["pchip", "akima", "makima"] {
        assert!(
            (v(&format!("interp1(0:4, 2 * (0:4), 2.5; method=\"{m}\")"))[0] - 5.0).abs() < 1e-14,
            "{m}: retas são exatas"
        );
    }
    let (it, _) = roda("r = interp1([1 2 3], [10 20 30], [1.5 2; 2.5 3])\n");
    assert_eq!(valor(&it, "r").dims(), [2, 2]);
    // Erros.
    for (src, id) in [
        (
            "interp1([1 1 2], [1 2 3], 1.5)",
            "chimalab:interp:pontosDuplicados",
        ),
        (
            "interp1([1 2], [1 2 3], 1.5)",
            "chimalab:interp:formaIncompativel",
        ),
        (
            "interp1([1 2], [1 2], 1.5; method=\"cubica\")",
            "chimalab:execucao:argumentos",
        ),
        (
            "interp1([1 2], [1 2], 1.5; extrap=\"x\")",
            "chimalab:execucao:argumentos",
        ),
        (
            "interp1([1 2], [1 2], 1.5; extrap=[1 2])",
            "chimalab:execucao:argumentos",
        ),
        (
            "interp1([1 2], [1 2], 1.5; metodo=\"linear\")",
            "chimalab:execucao:argumentos",
        ),
        ("roots([1 NaN])", "chimalab:linalg:valoresNaoFinitos"),
        ("roots([1i 1])", "chimalab:execucao:classeNaoSuportada"),
        ("polyfit([1 2], [1 2], 1.5)", "chimalab:execucao:argumentos"),
        ("conv([1 2; 3 4], [1 1])", "chimalab:execucao:argumentos"),
        ("conv([1 2], [1 1]; shape=\"x\")", "chimalab:execucao:argumentos"),
        ("fft([1 2], 0)", "chimalab:execucao:argumentos"),
        ("fft([1 2], 2, 0)", "chimalab:execucao:argumentos"),
        ("fft([1 2], 1e10)", "chimalab:runtime:arrayGrandeDemais"),
        ("fft()", "chimalab:execucao:argumentos"),
        ("polyval(x -> x, 1)", "chimalab:execucao:funcaoComoValor"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// Regressão: expressão no limite de aninhamento do parser avaliada dentro de uma recursão
/// estourava a pilha e abortava o processo. Agora a guarda de pilha devolve
/// `chimalab:execucao:recursaoProfunda`; com uma thread de pilha maior e orçamento maior, o mesmo
/// programa completa.
#[test]
fn pior_caso_de_pilha_nao_estoura() {
    let prof = chimalab_lang::sintaxe::MAX_ANINHAMENTO - 3;
    let expr = format!("{}n{}", "(".repeat(prof), ")".repeat(prof));
    let src =
        format!("def f(n):\n    if n <= 0:\n        return 0\n    return f({expr} - 1) + 1\nr = f(62)\n");
    // Na pilha de 2 MiB do teste: valor ou erro identificável, nunca estouro.
    let (_, r) = roda(&src);
    if let Some(e) = &r.erro {
        assert_eq!(e.id, "chimalab:execucao:recursaoProfunda");
    }
    let src2 = format!("def f(n):\n    return f({expr} + 1)\nf(0)\n");
    assert_eq!(
        roda(&src2).1.erro.unwrap().id,
        "chimalab:execucao:recursaoProfunda"
    );
    // (Dentro de `def`, `if` e `return`, cabem menos níveis de expressão no limite do parser.)
    let sinais = format!("{}1", "-".repeat(prof - 10));
    let src3 = format!(
        "def g(n):\n    if n <= 0:\n        return map(x -> {sinais} * x, [1 2])\n    return g(n - 1)\nr = g(60)\n"
    );
    if let Some(e) = roda(&src3).1.erro {
        assert_eq!(e.id, "chimalab:execucao:recursaoProfunda");
    }
    // Thread com 64 MiB e orçamento de 48 MiB: o mesmo programa completa.
    let h = std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || {
            let mut it = Interpretador::new();
            it.define_limite_de_pilha(48 << 20);
            let r = it.executa(&src);
            assert!(r.erro.is_none(), "{}", r.erro.unwrap());
            let Some(Obj::Array(v)) = it.variavel("r") else {
                panic!()
            };
            v.to_f64s().unwrap()
        })
        .unwrap();
    assert_eq!(h.join().unwrap(), [62.0]);
}

/// F002 CA3: o limite de recursão gera erro identificável e é configurável.
#[test]
fn f002_ca3_limite_de_recursao_configuravel() {
    let src = "def f(n):\n    if n <= 0:\n        return 0\n    return f(n - 1) + 1\nr = f(30)\n";
    let mut it = Interpretador::new();
    it.define_limite_de_recursao(20);
    let e = it.executa(src).erro.unwrap();
    assert_eq!(e.id, "chimalab:execucao:recursaoProfunda");
    assert!(e.mensagem.contains("mais de 20 chamadas"), "{}", e.mensagem);
    it.define_limite_de_recursao(40);
    assert!(it.executa(src).erro.is_none());
    assert_eq!(num(&it, "r"), [30.0]);
}

/// max/min: empates dão a primeira ocorrência (reais e inteiros); o extremo não é sempre o último.
#[test]
fn extremos_empates_e_posicoes() {
    let r = vars(
        "m, i = max([1 5 5 2])\nn, j = min([3 1 1 4])\n",
        &["m", "i", "n", "j"],
    );
    assert_eq!(r, [vec![5.0], vec![2.0], vec![1.0], vec![2.0]]);
    let r = vars(
        "m, i = max(int8([9 1 9 2]))\nn, j = min(int8([3 -1 7 -1]))\n",
        &["m", "i", "n", "j"],
    );
    assert_eq!(r, [vec![9.0], vec![1.0], vec![-1.0], vec![2.0]]);
    let r = vars(
        "m, i = max(uint8([2 200 7]))\nn, j = min(int16([5 -9 0]))\n",
        &["m", "i", "n", "j"],
    );
    assert_eq!(r, [vec![200.0], vec![2.0], vec![-9.0], vec![2.0]]);
    let r = vars("m, i = max([NaN 3 NaN 3])\n", &["m", "i"]);
    assert_eq!(r, [vec![3.0], vec![2.0]], "NaN ignorado; primeira ocorrência");
}

/// polyder, polyint, trapz e cumtrapz: valores exatos e identidades.
#[test]
fn derivadas_integrais_e_trapezios() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    assert_eq!(v("polyder([1 2 3])"), [2.0, 2.0]);
    assert_eq!(v("polyder(7)"), [0.0]);
    assert_eq!(v("polyint([3 2 1], 5)"), [1.0, 1.0, 1.0, 5.0]);
    assert_eq!(v("polyint([3 2 1])"), [1.0, 1.0, 1.0, 0.0]);
    assert_eq!(
        v("polyder(polyint([4 -3 2 1]))"),
        [4.0, -3.0, 2.0, 1.0],
        "derivada da integral"
    );
    assert_eq!(v("trapz([1 2 3])"), [4.0]);
    assert_eq!(v("trapz([0 1 3], [1 1 1])"), [3.0]);
    assert_eq!(v("trapz(0.5, [1 2 3])"), [2.0]);
    assert_eq!(v("trapz([1 2; 3 4])"), [2.0, 3.0]);
    assert_eq!(v("cumtrapz([1 2 3])"), [0.0, 1.5, 4.0]);
    // trapz é o último de cumtrapz; trapézios exatos para retas.
    assert_eq!(
        v("trapz(0:0.1:1, 2 * (0:0.1:1) + 1)")[0],
        v("cumtrapz(0:0.1:1, 2 * (0:0.1:1) + 1)")[10]
    );
    assert!((v("trapz(0:0.25:1, 2 * (0:0.25:1) + 1)")[0] - 2.0).abs() < 1e-15);
    let (it, _) = roda("r = trapz([1i 1i])\n");
    assert_eq!(valor(&it, "r").to_complex()[0], Complex64::new(0.0, 1.0));
    for (src, id) in [
        ("trapz([0 1], [1 2 3])", "chimalab:integral:formaIncompativel"),
        ("polyint([1 2], [1 2])", "chimalab:execucao:argumentos"),
        ("polyder()", "chimalab:execucao:argumentos"),
        ("trapz(1, 2, 3)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// O guia rápido (docs/linguagem/00-guia-rapido.md) roda exatamente como está escrito: cada
/// entrada `>>` (com as continuações `..`) é executada no mesmo workspace e a saída é comparada
/// texto a texto com as linhas que a seguem.
#[test]
fn guia_rapido_executa_como_escrito() {
    let doc = include_str!("../../../docs/linguagem/00-guia-rapido.md").replace("\r\n", "\n");
    let mut it = Interpretador::new();
    let mut entradas = 0;
    for bloco in doc.split("```\n").skip(1).step_by(2) {
        let linhas: Vec<&str> = bloco.lines().collect();
        let mut k = 0;
        while k < linhas.len() {
            let Some(primeira) = linhas[k].strip_prefix(">> ") else {
                panic!("esperava `>> ` em {:?}", linhas[k]);
            };
            let mut src = vec![primeira.to_string()];
            k += 1;
            while k < linhas.len() && (linhas[k] == ".." || linhas[k].starts_with(".. ")) {
                src.push(linhas[k].strip_prefix(".. ").unwrap_or("").to_string());
                k += 1;
            }
            let mut esperado = Vec::new();
            while k < linhas.len() && !linhas[k].starts_with(">> ") {
                esperado.push(linhas[k]);
                k += 1;
            }
            let texto = src.join("\n");
            assert_eq!(
                it.executa(&texto).texto(),
                esperado.join("\n"),
                "entrada: {texto}"
            );
            entradas += 1;
        }
    }
    assert!(entradas >= 25, "{entradas}");
}

/// dot, cross, kron e flip: valores exatos e identidades.
#[test]
fn dot_cross_kron_flip() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    assert_eq!(v("dot([1 2 3], [4 5 6])"), [32.0]);
    assert_eq!(
        v("dot([1; 2], [3 4])"),
        [11.0],
        "orientações diferentes, mesmo tamanho"
    );
    assert_eq!(v("dot([1i 1], [1i 1])"), [2.0], "conjuga o primeiro");
    assert_eq!(v("cross([1 0 0], [0 1 0])"), [0.0, 0.0, 1.0]);
    assert_eq!(v("cross([1 2 3], [4 5 6])"), [-3.0, 6.0, -3.0]);
    // cross(a, b) é ortogonal a a e a b.
    assert_eq!(
        v("dot(cross([1 2 3], [4 5 6]), [1 2 3]) + dot(cross([1 2 3], [4 5 6]), [4 5 6])"),
        [0.0]
    );
    assert_eq!(
        v("kron([1 2; 3 4], [1 1])"),
        [1.0, 3.0, 1.0, 3.0, 2.0, 4.0, 2.0, 4.0]
    );
    assert_eq!(
        v("kron(eye(2), [1 2; 3 4])"),
        [
            1.0, 3.0, 0.0, 0.0, 2.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 0.0, 0.0, 2.0, 4.0
        ]
    );
    assert_eq!(v("flip(1:4)"), [4.0, 3.0, 2.0, 1.0]);
    assert_eq!(v("flip([1 2; 3 4])"), [3.0, 1.0, 4.0, 2.0]);
    assert_eq!(v("flip([1 2; 3 4], 2)"), [2.0, 4.0, 1.0, 3.0]);
    assert_eq!(
        v("flip([1 2; 3 4], 3)"),
        [1.0, 3.0, 2.0, 4.0],
        "dimensão além das existentes: igual"
    );
    let (it, _) =
        roda("a = flip('abc')\nb = flip(int8([1 2]))\nc = flip(flip(reshape(1:24, 2, 3, 4), 3), 3)\n");
    assert_eq!(valor(&it, "a"), Value::text("cba"));
    assert_eq!(valor(&it, "b").class().name(), "int8");
    assert_eq!(num(&it, "c"), (1..=24).map(f64::from).collect::<Vec<_>>());
    for (src, id) in [
        ("dot([1 2], [1 2 3])", "chimalab:runtime:dimensoesIncompativeis"),
        (
            "dot([1 2; 3 4], [1 2; 3 4])",
            "chimalab:runtime:dimensoesIncompativeis",
        ),
        ("cross([1 2], [3 4])", "chimalab:runtime:dimensoesIncompativeis"),
        ("kron(ones(2, 2, 2), 1)", "chimalab:execucao:argumentos"),
        (
            "kron(ones(1, 1e5), ones(1e5, 1))",
            "chimalab:runtime:arrayGrandeDemais",
        ),
        ("flip([1 2], 0)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// Funções elementares de complexos contra identidades exatas (mutação: sem o ramo próprio, a
/// conta seria feita parte a parte, o que é errado para complexos).
#[test]
fn elementares_complexas_valores() {
    use std::f64::consts::FRAC_PI_2;
    let z = |src: &str| {
        let (it, r) = roda(&format!("r = {src}\n"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        valor(&it, "r").to_complex()[0]
    };
    let perto = |a: Complex64, b: Complex64| (a - b).norm() < 1e-15;
    assert!(perto(z("log(1i)"), Complex64::new(0.0, FRAC_PI_2)));
    assert!(perto(z("sin(1i)"), Complex64::new(0.0, 1f64.sinh())));
    assert!(perto(z("cos(1i)"), Complex64::new(1f64.cosh(), 0.0)));
    assert!(perto(z("tan(1i)"), Complex64::new(0.0, 1f64.tanh())));
    assert!(perto(
        z("exp(1 + 1i)"),
        Complex64::new(1f64.exp() * 1f64.cos(), 1f64.exp() * 1f64.sin())
    ));
    assert!(perto(z("sinh(1i)"), Complex64::new(0.0, 1f64.sin())));
    assert!(perto(z("cosh(1i)"), Complex64::new(1f64.cos(), 0.0)));
    assert!(perto(z("tanh(1i)"), Complex64::new(0.0, 1f64.tan())));
    assert!(perto(
        z("log10(10i)"),
        Complex64::new(1.0, FRAC_PI_2 / std::f64::consts::LN_10)
    ));
    assert!(perto(
        z("log2(2i)"),
        Complex64::new(1.0, FRAC_PI_2 / std::f64::consts::LN_2)
    ));
    assert!(perto(z("asin(sin(0.5i))"), Complex64::new(0.0, 0.5)));
    assert!(perto(z("acos(cos(0.5 + 0.5i))"), Complex64::new(0.5, 0.5)));
    assert!(perto(z("atan(tan(0.3i))"), Complex64::new(0.0, 0.3)));
}

/// A ajuda põe o título da seção antes das linhas (uma vez por seção).
#[test]
fn ajuda_com_titulos_de_secao() {
    let t = console("help(\"trace\")");
    assert_eq!(
        t,
        "6.6 Álgebra linear\n  det(A), inv(A), trace(A) — determinante, inversa (aviso se singular), traço"
    );
    let t = console("help(\"max\")");
    let titulos: Vec<&str> = t.lines().filter(|l| !l.starts_with("  ")).collect();
    assert_eq!(titulos, ["6.3 Reduções"], "{t}");
}

/// Programas inteiros, como alguém os escreveria, contra oráculos independentes.
#[test]
fn programas_completos() {
    // Newton para √2.
    let src = "x = 1;\nwhile abs(x^2 - 2) > 1e-15:\n    x = x - (x^2 - 2) / (2 * x);\n";
    assert_eq!(vars(src, &["x"])[0], [std::f64::consts::SQRT_2]);
    // Eliminação de Gauss com laços e índices, contra a barra invertida.
    let src = "\
def gauss(A, b):
    n = size(A, 1)
    for k in 1:n-1:
        for i in k+1:n:
            m = A[i, k] / A[k, k]
            A[i, :] = A[i, :] - m * A[k, :]
            b[i] = b[i] - m * b[k]
    x = zeros(n, 1)
    for i in n:-1:1:
        x[i] = (b[i] - A[i, i+1:n] * x[i+1:n]) / A[i, i]
    return x

A = [4 -2 1; -2 4 -2; 1 -2 4];
b = [11; -16; 17];
x = gauss(A, b);
erro = norm(x - A \\ b);
";
    let r = vars(src, &["x", "erro"]);
    assert!(r[1][0] < 1e-14, "{:?}", r);
    assert!(
        r[0].iter()
            .zip([1.0, -2.0, 3.0])
            .all(|(a, b)| (a - b).abs() < 1e-14)
    );
    // Crivo de Eratóstenes com máscara lógica.
    let src = "\
n = 50;
primo = ones(1, n) > 0;
primo[1] = false;
for p in 2:floor(sqrt(n)):
    if primo[p]:
        primo[p*p:p:n] = false
primos = find(primo);
";
    let esperado: Vec<f64> = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
        .iter()
        .map(|&x| f64::from(x))
        .collect();
    assert_eq!(vars(src, &["primos"])[0], esperado);
    // Fibonacci com crescimento do vetor; exato em double até F(78).
    let src = "f = [1 1];\nwhile numel(f) < 78:\n    f[end + 1] = f[end] + f[end - 1];\nultimo = f[end];\n";
    assert_eq!(vars(src, &["ultimo"])[0], [8_944_394_323_791_464.0]);
    // Bissecção recursiva.
    let src = "\
def bissecta(f, a, b, tol):
    m = (a + b) / 2
    if b - a < tol:
        return m
    if sign(f(m)) == sign(f(a)):
        return bissecta(f, m, b, tol)
    return bissecta(f, a, m, tol)

r = bissecta(x -> x^3 - 2, 0, 2, 1e-12);
";
    assert!((vars(src, &["r"])[0][0] - 2f64.cbrt()).abs() < 1e-12);
    // Ordenação por bolha contra sort.
    let src = "\
v = [5 3 9 1 7 2 8];
trocou = true;
while trocou:
    trocou = false;
    for i in 1:numel(v)-1:
        if v[i] > v[i+1]:
            t = v[i];
            v[i] = v[i+1];
            v[i+1] = t;
            trocou = true;
igual = isequal(v, sort([5 3 9 1 7 2 8]));
";
    assert_eq!(vars(src, &["igual"])[0], [1.0]);
    // Trapézios num laço contra trapz e contra a integral exata de x² em [0, 1].
    let src = "\
x = range(0, 1; length=101);
y = x .^ 2;
s = 0;
for k in 1:numel(x)-1:
    s += (x[k+1] - x[k]) * (y[k] + y[k+1]) / 2;
diferenca = abs(s - trapz(x, y));
";
    let r = vars(src, &["s", "diferenca"]);
    assert!(r[1][0] < 1e-15);
    assert!(
        (r[0][0] - (1.0 / 3.0 + 1.0 / 60_000.0)).abs() < 1e-14,
        "erro do trapézio: h²/6·(b−a)·f''/2"
    );
}

/// Funções especiais (F042) e de inteiros pela linguagem, contra identidades exatas.
#[test]
fn funcoes_especiais_por_identidades() {
    use std::f64::consts::PI;
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    let perto = |a: f64, b: f64, tol: f64| (a - b).abs() <= tol * b.abs().max(1.0);
    assert_eq!(v("gamma([1 2 3 4 5])"), [1.0, 1.0, 2.0, 6.0, 24.0]);
    assert!(perto(v("gamma(0.5)")[0], PI.sqrt(), 1e-15));
    assert_eq!(v("factorial(10)"), [3_628_800.0]);
    assert!(perto(v("gammaln(10)")[0], 362_880f64.ln(), 1e-15));
    assert!(
        perto(v("psi(1)")[0], -std::f64::consts::EULER_GAMMA, 1e-15),
        "−γ de Euler–Mascheroni"
    );
    assert!(
        v("erf([-1 0 0.3 2]) + erfc([-1 0 0.3 2])")
            .iter()
            .all(|x| perto(*x, 1.0, 1e-15))
    );
    assert!(perto(v("erfinv(erf(0.3))")[0], 0.3, 1e-15));
    assert!(perto(v("erfcinv(erfc(0.7))")[0], 0.7, 1e-15));
    assert!(perto(v("erfcx(2)")[0], 4f64.exp() * v("erfc(2)")[0], 1e-14));
    assert!(perto(v("zeta(2)")[0], PI * PI / 6.0, 1e-15));
    assert!(perto(v("zeta(4)")[0], PI.powi(4) / 90.0, 1e-15));
    assert!(perto(
        v("beta(2.5, 1.5)")[0],
        v("gamma(2.5) * gamma(1.5) / gamma(4)")[0],
        1e-15
    ));
    assert!(perto(v("exp(betaln(30, 40))")[0], v("beta(30, 40)")[0], 1e-12));
    assert!(v("expint(1)")[0] > 0.219 && v("expint(1)")[0] < 0.2194);
    // Bessel de ordem ½: formas fechadas.
    for x in [0.5f64, 1.0, 3.0, 7.5] {
        let s = (2.0 / (PI * x)).sqrt();
        assert!(
            perto(v(&format!("besselj(0.5, {x})"))[0], s * x.sin(), 1e-14),
            "J½({x})"
        );
        assert!(
            perto(v(&format!("bessely(0.5, {x})"))[0], -s * x.cos(), 1e-14),
            "Y½({x})"
        );
        assert!(
            perto(v(&format!("besseli(0.5, {x})"))[0], s * x.sinh(), 1e-14),
            "I½({x})"
        );
        assert!(
            perto(
                v(&format!("besselk(0.5, {x})"))[0],
                (PI / (2.0 * x)).sqrt() * (-x).exp(),
                1e-14
            ),
            "K½({x})"
        );
    }
    assert_eq!(v("besselj([0 1], 0)"), [1.0, 0.0]);
    // Gama complexa: |Γ(i)|² = π / sinh(π).
    assert!(perto(v("abs(gamma(1i))^2")[0], PI / PI.sinh(), 1e-14));
    assert!(perto(v("real(erf(0.5 + 0i))")[0], v("erf(0.5)")[0], 1e-15));
    // Inteiros.
    assert_eq!(v("gcd([12 7], [18 21])"), [6.0, 7.0]);
    assert_eq!(v("lcm(4, 6)"), [12.0]);
    assert_eq!(v("primes(20)"), [2.0, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0, 19.0]);
    assert!(v("primes(1)").is_empty());
    assert_eq!(v("isprime([1 2 9 13 -3 2.5])"), [0.0, 1.0, 0.0, 1.0, 0.0, 0.0]);
    assert_eq!(v("factor(360)"), [2.0, 2.0, 2.0, 3.0, 3.0, 5.0]);
    assert_eq!(v("prod(factor(9699690))"), [9_699_690.0]);
    assert_eq!(v("nchoosek(5, 2)"), [10.0]);
    let (_, r) = roda("r = nchoosek(100, 50);");
    assert!(
        r.texto().contains("chimalab:special:perdaDePrecisao"),
        "{}",
        r.texto()
    );
    let (it, _) = roda("s = gamma(single(5))\n");
    assert_eq!(valor(&it, "s").class().name(), "single");
    for (src, id) in [
        ("gamma(int8(3))", "chimalab:execucao:classeNaoSuportada"),
        ("zeta(1i)", "chimalab:execucao:classeNaoSuportada"),
        ("besselj(1i, 1)", "chimalab:execucao:classeNaoSuportada"),
        ("gcd(1.5, 2)", "chimalab:special:dominio"),
        ("gcd([1 2], [1 2 3])", "chimalab:runtime:dimensoesIncompativeis"),
        ("factor(0)", "chimalab:execucao:argumentos"),
        ("factor([4 6])", "chimalab:execucao:argumentos"),
        ("primes([10 20])", "chimalab:execucao:argumentos"),
        ("primes(1e300)", "chimalab:runtime:arrayGrandeDemais"),
        ("nchoosek([5 6], 2)", "chimalab:execucao:argumentos"),
        ("beta(1)", "chimalab:execucao:argumentos"),
        ("gamma(x -> x)", "chimalab:execucao:funcaoComoValor"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// Números aleatórios (F053): reprodutíveis por padrão (semente 0), `seed` recomeça, formas,
/// intervalos e momentos dentro de 5 desvios-padrão da média amostral.
#[test]
fn numeros_aleatorios() {
    let gera = |src: &str| vars(src, &["x"]).remove(0);
    // Sessões novas repetem a sequência; seed recomeça; sementes diferentes diferem.
    assert_eq!(gera("x = rand(1, 5)"), gera("x = rand(1, 5)"));
    assert_eq!(gera("seed(7);\nx = rand(1, 5)"), gera("seed(7);\nx = rand(1, 5)"));
    assert_ne!(gera("seed(7);\nx = rand(1, 5)"), gera("seed(8);\nx = rand(1, 5)"));
    assert_eq!(
        gera("a = rand(1, 3);\nseed(0);\nx = rand(1, 3)"),
        gera("x = rand(1, 3)")
    );
    assert_ne!(
        gera("seed(1; algorithm=\"twister\");\nx = rand(1, 3)"),
        gera("seed(1);\nx = rand(1, 3)")
    );
    assert_eq!(
        gera("seed(1; algorithm=\"twister\");\nseed(2);\nx = rand(1, 3)"),
        gera("seed(2; algorithm=\"twister\");\nx = rand(1, 3)"),
        "seed sem algoritmo mantém o atual"
    );
    // Formas.
    let (it, _) = roda("a = rand()\nb = rand(3)\nc = randn(2, 3, 4)\nd = randi(5, 2, 7)\ne = rand(0, 4)\n");
    assert_eq!(valor(&it, "a").dims(), [1, 1]);
    assert_eq!(valor(&it, "b").dims(), [3, 3]);
    assert_eq!(valor(&it, "c").dims(), [2, 3, 4]);
    assert_eq!(valor(&it, "d").dims(), [2, 7]);
    assert_eq!(valor(&it, "e").dims(), [0, 4]);
    // Uniformes em (0, 1); momentos.
    let u = gera("x = rand(1, 100000)");
    assert!(u.iter().all(|&x| x > 0.0 && x < 1.0));
    let m = u.iter().sum::<f64>() / u.len() as f64;
    assert!((m - 0.5).abs() < 5.0 * (1.0f64 / 12.0).sqrt() / 316.2, "{m}");
    let z = gera("x = randn(1, 100000)");
    let mz = z.iter().sum::<f64>() / z.len() as f64;
    let vz = z.iter().map(|x| (x - mz).powi(2)).sum::<f64>() / (z.len() - 1) as f64;
    assert!(
        mz.abs() < 5.0 / 316.2 && (vz - 1.0).abs() < 5.0 * (2.0f64 / 1e5).sqrt(),
        "{mz} {vz}"
    );
    // Inteiros: intervalo fechado e todos os valores aparecem.
    let k = gera("x = randi(6, 1, 6000)");
    assert!(k.iter().all(|&x| (1.0..=6.0).contains(&x) && x.fract() == 0.0));
    assert!((1..=6).all(|v| k.contains(&f64::from(v))));
    let k = gera("x = randi([-2 2], 1, 1000)");
    assert!(k.iter().all(|&x| (-2.0..=2.0).contains(&x)) && k.contains(&-2.0) && k.contains(&2.0));
    // Permutações.
    let mut p = gera("x = randperm(10)");
    p.sort_by(f64::total_cmp);
    assert_eq!(p, (1..=10).map(f64::from).collect::<Vec<_>>());
    let s = gera("x = randperm(10, 3)");
    assert_eq!(s.len(), 3);
    assert!(s.iter().all(|&x| (1.0..=10.0).contains(&x)) && s[0] != s[1] && s[1] != s[2] && s[0] != s[2]);
    for (src, id) in [
        ("randi(0)", "chimalab:execucao:argumentos"),
        ("randi([3 1])", "chimalab:execucao:argumentos"),
        ("randi(1.5)", "chimalab:execucao:argumentos"),
        ("randi()", "chimalab:execucao:argumentos"),
        ("randi([1 2 3])", "chimalab:execucao:argumentos"),
        ("randperm(3, 5)", "chimalab:random:argumentoInvalido"),
        ("randperm()", "chimalab:execucao:argumentos"),
        ("rand(-1)", "chimalab:execucao:argumentos"),
        ("rand(1e10, 1e10)", "chimalab:runtime:arrayGrandeDemais"),
        ("rand(2; n=1)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("x = {src}")).0, id, "{src}");
    }
    for (src, id) in [
        ("seed(-1)", "chimalab:execucao:argumentos"),
        ("seed()", "chimalab:execucao:argumentos"),
        (
            "seed(1; algorithm=\"nada\")",
            "chimalab:random:geradorDesconhecido",
        ),
        ("seed(1; algorithm=2)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(src).0, id, "{src}");
    }
}

/// fminsearch (vários parâmetros, forma de x0 preservada) e integral2 (limites numéricos).
#[test]
fn fminsearch_e_integral2() {
    let r = vars(
        "x, fx = fminsearch(v -> (v[1] - 1)^2 + 10 * (v[2] - v[1]^2)^2, [-1.2 1]; tol_x=1e-10, tol_fun=1e-14, max_iter=5000)\n",
        &["x", "fx"],
    );
    assert!(r[0].iter().all(|x| (x - 1.0).abs() < 1e-6), "{r:?}");
    assert!(r[1][0] < 1e-12);
    let (it, _) = roda("x = fminsearch(v -> sum((v - [1; 2; 3]) .^ 2), zeros(3, 1))\n");
    assert_eq!(valor(&it, "x").dims(), [3, 1], "forma de x0");
    assert!(
        num(&it, "x")
            .iter()
            .zip([1.0, 2.0, 3.0])
            .all(|(a, b)| (a - b).abs() < 1e-3)
    );
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0)[0];
    assert!((v("integral2((x, y) -> x * y, 0, 1, 0, 2)") - 1.0).abs() < 1e-12);
    assert!(
        (v("integral2((x, y) -> exp(-(x^2 + y^2)), -Inf, Inf, -Inf, Inf)") - std::f64::consts::PI).abs()
            < 1e-6
    );
    assert!(
        (v("integral2((x, y) -> x^2 + y, 0, 1, 0, 1; abs_tol=1e-12, rel_tol=1e-12)") - (1.0 / 3.0 + 0.5))
            .abs()
            < 1e-12
    );
    for (src, id) in [
        (
            "fminsearch(v -> v, [1 2])",
            "chimalab:execucao:resultadoNaoEscalar",
        ),
        ("fminsearch(v -> nada, 1)", "chimalab:execucao:nomeIndefinido"),
        (
            "fminsearch(v -> v^2, 1; max_iter=0)",
            "chimalab:execucao:argumentos",
        ),
        (
            "fminsearch(v -> v^2, zeros(1, 0))",
            "chimalab:otimizacao:pontoInicialInvalido",
        ),
        ("fminsearch(v -> v^2, 1; passo=1)", "chimalab:execucao:argumentos"),
        (
            "integral2((x, y) -> [x y], 0, 1, 0, 1)",
            "chimalab:execucao:resultadoNaoEscalar",
        ),
        (
            "integral2((x, y) -> x, 0, 1, x -> 0, 1)",
            "chimalab:execucao:naoSuportado",
        ),
        ("integral2((x, y) -> x, 0, 1, 0)", "chimalab:execucao:argumentos"),
        (
            "integral2((x, y) -> nada, 0, 1, 0, 1)",
            "chimalab:execucao:nomeIndefinido",
        ),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// Exibição N-D com rótulos de 4 dimensões e o conteúdo de cada fatia; formatos de f-string no
/// limite (precisão 30, largura de 3 algarismos).
#[test]
fn exibicao_nd_e_limites_de_formato() {
    let t = console("A = reshape(1:16, 2, 2, 2, 2)");
    let rotulos: Vec<&str> = t
        .lines()
        .filter(|l| l.trim_start().starts_with("[:"))
        .map(str::trim)
        .collect();
    assert_eq!(
        rotulos,
        ["[:, :, 1, 1]", "[:, :, 2, 1]", "[:, :, 1, 2]", "[:, :, 2, 2]"]
    );
    let t = console("B = reshape(1:8, 2, 2, 2)");
    assert_eq!(
        t,
        "B = 2×2×2\n  [:, :, 1]\n    1  3\n    2  4\n  [:, :, 2]\n    5  7\n    6  8"
    );
    let s = |src: &str| {
        let (it, r) = roda(&format!("s = {src}"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        match valor(&it, "s") {
            Value::Char(a) => a.as_slice().iter().collect::<String>(),
            v => panic!("{v:?}"),
        }
    };
    assert_eq!(s("f\"{0.5:.30f}\"").len(), 32);
    assert_eq!(s("f\"{7:100}\"").len(), 100);
    assert_eq!(erro("s = f\"{7:1000}\"").0, "chimalab:execucao:formatoInvalido");
    assert_eq!(erro("s = f\"{7:.31f}\"").0, "chimalab:execucao:formatoInvalido");
}

/// round(x, n) e diff: valores exatos e identidades.
#[test]
fn arredondamento_em_casas_e_diferencas() {
    let v = |src: &str| vars(&format!("r = {src}\n"), &["r"]).remove(0);
    assert_eq!(v("round(pi, 2)"), [314.0 / 100.0]);
    assert_eq!(v("round(1234.5, -2)"), [1200.0]);
    assert_eq!(
        v("round([1.25 -1.25], 1)"),
        [1.3, -1.3],
        "meio para longe do zero"
    );
    assert_eq!(v("round(2.5)"), [3.0]);
    let (it, _) = roda("a = round(int16(1234), -2)\nb = round(1.25 + 2.75i, 1)\n");
    assert_eq!(valor(&it, "a").class().name(), "int16");
    assert_eq!(num(&it, "a"), [1200.0]);
    assert_eq!(valor(&it, "b").to_complex()[0], Complex64::new(1.3, 2.8));
    assert_eq!(v("diff([1 4 9 16])"), [3.0, 5.0, 7.0]);
    assert_eq!(v("diff([1 4 9 16], 2)"), [2.0, 2.0]);
    assert_eq!(
        v("diff((1:6) .^ 3, 3)"),
        [6.0, 6.0, 6.0],
        "terceira diferença de uma cúbica: 6"
    );
    assert_eq!(v("diff([1 2; 4 8])"), [3.0, 6.0]);
    assert_eq!(v("diff([1 2; 4 8], 1, 2)"), [1.0, 4.0]);
    assert_eq!(v("diff([5 7], 5)").len(), 0);
    assert_eq!(v("diff(7)").len(), 0);
    // cumsum desfaz diff a menos do primeiro elemento.
    assert_eq!(v("[1 cumsum(diff([1 4 9 16])) + 1]"), [1.0, 4.0, 9.0, 16.0]);
    let (it, _) = roda("a = diff(int8([10 -100 100]))\nb = diff([1i 3])\n");
    assert_eq!(num(&it, "a"), [-110.0, 127.0], "inteiros saturam");
    assert_eq!(valor(&it, "b").to_complex()[0], Complex64::new(3.0, -1.0));
    for (src, id) in [
        ("round(1, 0.5)", "chimalab:execucao:argumentos"),
        ("round(1, 1e5)", "chimalab:execucao:argumentos"),
        ("diff([1 2], -1)", "chimalab:execucao:argumentos"),
        ("diff([1 2], 1, 0)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
}

/// f-strings: texto com precisão é erro; texto de várias linhas sai linha a linha; inteiros
/// acima de 2⁵³ saem exatos sem formato e arredondados com precisão.
#[test]
fn fstrings_de_texto_e_inteiros_grandes() {
    let s = |src: &str| {
        let (it, r) = roda(&format!("x = int64(9007199254740992) + 1;\ns = {src}"));
        assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
        match valor(&it, "s") {
            Value::Char(a) => a.as_slice().iter().collect::<String>(),
            v => panic!("{v:?}"),
        }
    };
    assert_eq!(s("f\"{['ab'; 'cd']}\""), "ab\ncd");
    assert_eq!(s("f\"{x}\""), "9007199254740993");
    assert_eq!(s("f\"{x:d}\""), "9007199254740993");
    assert_eq!(s("f\"{x:.2}\""), "9e15");
    assert_eq!(erro("s = f\"{'abc':.2}\"").0, "chimalab:execucao:formatoInvalido");
}
