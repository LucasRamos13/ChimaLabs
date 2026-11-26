//! Console e execução de scripts do ChimaLab (F001).
//!
//! O console lê linha a linha e executa quando a entrada está completa: delimitadores e
//! comentários fechados e, se abriu um bloco, depois da linha em branco que o encerra (DT-39,
//! resposta 46.A). O workspace persiste entre as entradas.

use std::io::{self, BufRead, Write};

use chimalab_lang::lexico::incomplete;
use chimalab_runtime::{ErroExec, Interpretador, Resultado};

/// Prompt de entrada nova.
pub const PROMPT: &str = ">> ";
/// Prompt de continuação (bloco ou delimitador aberto).
pub const CONTINUA: &str = ".. ";

/// Sessão de console: lê de `entrada` até o fim e escreve o que cada entrada exibe em `saida`.
/// Com `prompts`, escreve `>> ` e `.. ` antes de cada linha (modo interativo).
pub fn sessao<R: BufRead, W: Write>(entrada: R, saida: &mut W, prompts: bool) -> io::Result<()> {
    let mut it = Interpretador::new();
    let mut pendente = String::new();
    let mut linhas = entrada.lines();
    loop {
        if prompts {
            write!(saida, "{}", if pendente.is_empty() { PROMPT } else { CONTINUA })?;
            saida.flush()?;
        }
        let Some(linha) = linhas.next() else {
            // Fim da entrada: executa o que sobrou (um bloco sem a linha em branco final).
            if !pendente.trim().is_empty() {
                executa(&mut it, &pendente, saida)?;
            }
            return Ok(());
        };
        let linha = linha?;
        if pendente.is_empty() && linha.trim().is_empty() {
            continue;
        }
        pendente.push_str(&linha);
        pendente.push('\n');
        if !incomplete(&pendente) {
            executa(&mut it, &pendente, saida)?;
            pendente.clear();
        }
    }
}

fn executa<W: Write>(it: &mut Interpretador, src: &str, saida: &mut W) -> io::Result<()> {
    let t = texto_com_erro(src, it.executa(src));
    if !t.is_empty() {
        writeln!(saida, "{t}")?;
    }
    Ok(())
}

/// Erro com a linha do código e um `^` na coluna (tabulações preservadas no recuo).
pub fn formata_erro(src: &str, e: &ErroExec) -> String {
    let mut t = format!("erro: {e}");
    if let Some(linha) = src.lines().nth(e.linha.saturating_sub(1)) {
        let recuo: String = linha
            .chars()
            .take(e.coluna.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        t.push_str(&format!("\n  {linha}\n  {recuo}^"));
    }
    t
}

fn texto_com_erro(src: &str, r: Resultado) -> String {
    let mut t: Vec<String> = r.saidas.iter().map(|s| s.to_string()).collect();
    if let Some(e) = &r.erro {
        t.push(formata_erro(src, e));
    }
    t.join("\n")
}

/// Executa um programa inteiro (script ou `-c`). Devolve o texto exibido e, se houver, o erro
/// (para a saída de erro e o código de saída 1).
pub fn executa_programa(src: &str) -> (String, Option<String>) {
    executa_como(src, false)
}

/// Como [`executa_programa`], mas como documento: só a última instrução do nível de cima é
/// exibida (DT-34); `disp` e avisos continuam.
pub fn executa_documento(src: &str) -> (String, Option<String>) {
    executa_como(src, true)
}

fn executa_como(src: &str, documento: bool) -> (String, Option<String>) {
    let mut it = Interpretador::new();
    let r = if documento {
        it.executa_documento(src)
    } else {
        it.executa(src)
    };
    let saidas: Vec<String> = r.saidas.iter().map(|s| s.to_string()).collect();
    (saidas.join("\n"), r.erro.map(|e| formata_erro(src, &e)))
}
