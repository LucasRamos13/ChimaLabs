//! `help`: a ajuda vem da própria especificação (`docs/linguagem/06-funcoes.md`, embutida na
//! compilação), então a ajuda e a documentação não divergem.

const ESPEC: &str = include_str!("../../../docs/linguagem/06-funcoes.md");

/// Remove a marcação de uma linha da especificação (tabela ou item).
fn limpa(l: &str) -> String {
    let l = l.trim().trim_start_matches("- ");
    let l = if l.starts_with('|') {
        let campos: Vec<&str> = l.trim_matches('|').split(" | ").map(str::trim).collect();
        campos.join(" — ")
    } else {
        l.to_string()
    };
    l.replace('`', "").replace("\\|", "|")
}

/// Se a linha menciona a função `nome` (como `nome(` ou `` `nome` ``).
fn menciona(l: &str, nome: &str) -> bool {
    [
        format!("`{nome}("),
        format!("`{nome}`"),
        format!(" {nome}("),
        format!("`{nome} "),
    ]
    .iter()
    .any(|p| l.contains(p.as_str()))
        || l.split(['`', ' ', ',', '(']).any(|p| p == nome) && l.starts_with('`')
}

/// Texto de ajuda de uma função embutida (ou `None` se a especificação não a descreve).
pub fn de(nome: &str) -> Option<String> {
    let mut secao = "";
    let mut achados: Vec<(String, String)> = Vec::new();
    for l in ESPEC.lines() {
        if let Some(t) = l.strip_prefix("## ") {
            secao = t;
            continue;
        }
        // Linhas de tabela, itens e linhas que começam por código; cabeçalhos e separadores de
        // tabela não mencionam funções e caem no filtro de `menciona`.
        if (l.starts_with('|') || l.starts_with("- ") || l.starts_with('`')) && menciona(l, nome) {
            achados.push((secao.to_string(), limpa(l)));
        }
    }
    if achados.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    let mut ultima = String::new();
    for (s, l) in achados {
        if s != ultima {
            out.push(s.clone());
            ultima = s;
        }
        out.push(format!("  {l}"));
    }
    Some(out.join("\n"))
}

/// Índice: as seções da especificação.
pub fn indice() -> String {
    let mut out = vec!["Funções embutidas (help(nome) mostra uma delas):".to_string()];
    out.extend(
        ESPEC
            .lines()
            .filter_map(|l| l.strip_prefix("## "))
            .map(|s| format!("  {s}")),
    );
    out.join("\n")
}
