//! Laço do console (F001): entrada linha a linha, blocos fechados por linha em branco,
//! workspace persistente, prompts e o executável.

use std::io::Cursor;
use std::process::Command;

use chimalab_cli::{CONTINUA, PROMPT, executa_programa, sessao};

fn console(entrada: &str, prompts: bool) -> String {
    let mut saida = Vec::new();
    sessao(Cursor::new(entrada), &mut saida, prompts).unwrap();
    String::from_utf8(saida).unwrap()
}

/// CA: '1+1' exibe 2 e define ans; ';' suprime; o workspace persiste entre entradas.
#[test]
fn ca1_ca2_expressao_ans_e_ponto_e_virgula() {
    assert_eq!(console("1+1\nans * 10\n", false), "ans = 2\nans = 20\n");
    assert_eq!(console("x = 5;\nx + 1\n", false), "ans = 6\n");
    assert_eq!(console("x = 5;\n\n\ny = x\n", false), "y = 5\n");
}

/// CA: um bloco multilinha só executa depois da linha em branco que o fecha (46.A); um
/// delimitador aberto continua na linha seguinte.
#[test]
fn ca3_bloco_so_executa_no_fim() {
    let entrada = "s = 0;\nfor k in 1:3:\n    s += k;\n    s\n\ns\n";
    // Dentro do laço, `s` sem `;` exibe a cada volta; depois, `s` final.
    assert_eq!(console(entrada, false), "s = 1\ns = 3\ns = 6\ns = 6\n");
    // Prompts: `..` enquanto o bloco está aberto.
    let t = console("if 1:\n    x = 1\n\nx\n", true);
    assert_eq!(
        t,
        format!("{PROMPT}{CONTINUA}{CONTINUA}x = 1\n{PROMPT}x = 1\n{PROMPT}")
    );
    // Delimitador aberto atravessa linhas e executa ao fechar, sem precisar de linha em branco.
    assert_eq!(
        console("A = [1 2\n     3 4]\n", false),
        "A = 2×2\n  1  2\n  3  4\n"
    );
    assert_eq!(
        console("f(1,\n", false),
        "erro: linha 1, coluna 2: abertura sem fechamento\n  f(1,\n   ^\n"
    );
    // Fim da entrada com bloco aberto: executa o que há.
    assert_eq!(console("if 1:\n    z = 2", false), "z = 2\n");
    // Linhas em branco soltas não executam nada.
    assert_eq!(
        console("\n\n   \n", true),
        format!("{PROMPT}{PROMPT}{PROMPT}{PROMPT}")
    );
}

/// CA: erro de sintaxe com posição; o erro não derruba a sessão nem apaga o workspace.
#[test]
fn ca5_erros_nao_encerram_a_sessao() {
    let t = console("a = 1;\nb = (2 +\n)\nc = a + 1\n", false);
    assert_eq!(
        t,
        "erro: linha 2, coluna 1: esperava uma expressão, veio `)`\n  )\n  ^\nc = 2\n"
    );
}

#[test]
fn scripts_e_executavel() {
    let (t, e) = executa_programa("def f(x):\n    return 2 * x\ny = f(21)\n");
    assert_eq!((t.as_str(), e), ("y = 42", None));
    let (t, e) = executa_programa("x = 1\ny = nada\n");
    assert_eq!(t, "x = 1");
    assert_eq!(
        e.unwrap(),
        "erro: linha 2, coluna 5: `nada` não está definido\n  y = nada\n      ^"
    );
    // O executável de verdade: -c, arquivo, versão, uso e código de saída.
    let exe = env!("CARGO_BIN_EXE_chimalab");
    let o = Command::new(exe).args(["-c", "x = 6 * 7"]).output().unwrap();
    assert!(o.status.success());
    assert_eq!(String::from_utf8_lossy(&o.stdout), "x = 42\n");
    let o = Command::new(exe).args(["-c", "y = z"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("`z` não está definido"));
    let dir = std::env::temp_dir().join(format!("chimalab-cli-teste-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let arq = dir.join("script.chl");
    std::fs::write(&arq, "s = 0;\nfor k in 1:4:\n    s += k;\ns\n").unwrap();
    let o = Command::new(exe).arg(&arq).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "s = 10\n");
    // Como documento: só a última instrução aparece.
    let doc = dir.join("doc.chl");
    std::fs::write(&doc, "a = 1\nb = a + 1\nb * 10\n").unwrap();
    let o = Command::new(exe)
        .args(["--documento".as_ref(), doc.as_os_str()])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "ans = 20\n");
    let o = Command::new(exe)
        .args(["--documento".as_ref(), dir.join("nada.chl").as_os_str()])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    std::fs::remove_dir_all(&dir).unwrap();
    let o = Command::new(exe)
        .arg(dir.join("nao_existe.chl"))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("não foi possível ler"));
    let o = Command::new(exe).arg("--versao").output().unwrap();
    assert!(String::from_utf8_lossy(&o.stdout).starts_with("chimalab "));
    let o = Command::new(exe).args(["-x", "a", "b"]).output().unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("uso: chimalab"));
    for a in ["--ajuda", "--help", "-h"] {
        let o = Command::new(exe).arg(a).output().unwrap();
        assert!(o.status.success());
        let t = String::from_utf8_lossy(&o.stdout);
        assert!(t.starts_with("uso: chimalab") && t.contains("help()"), "{t}");
    }
    // Sem argumentos e sem terminal: lê o console da entrada padrão.
    let mut filho = Command::new(exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        filho
            .stdin
            .take()
            .unwrap()
            .write_all(b"a = 2;\na ^ 10\n")
            .unwrap();
    }
    let o = filho.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "ans = 1024\n");
}

/// O erro mostra a linha do código e um `^` na coluna; tabulações ficam no recuo do `^`.
#[test]
fn erro_com_linha_e_marcador() {
    use chimalab_cli::formata_erro;
    use chimalab_runtime::ErroExec;
    let e = |linha, coluna| ErroExec {
        id: "x:y".into(),
        mensagem: "m".into(),
        linha,
        coluna,
    };
    assert_eq!(
        formata_erro("a = 1\nb = zz\n", &e(2, 5)),
        "erro: linha 2, coluna 5: m\n  b = zz\n      ^"
    );
    assert_eq!(
        formata_erro("\tx = y", &e(1, 6)),
        "erro: linha 1, coluna 6: m\n  \tx = y\n  \t    ^"
    );
    assert_eq!(
        formata_erro("é = ü", &e(1, 5)),
        "erro: linha 1, coluna 5: m\n  é = ü\n      ^"
    );
    // Linha fora do texto (fim da entrada): só a mensagem.
    assert_eq!(formata_erro("a", &e(3, 1)), "erro: linha 3, coluna 1: m");
}
