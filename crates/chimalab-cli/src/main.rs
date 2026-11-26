//! Executável `chimalab`: console interativo, `chimalab arquivo` ou `chimalab -c "código"`.

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use chimalab_cli::{executa_documento, executa_programa, sessao};

const USO: &str = "uso: chimalab [arquivo | --documento arquivo | -c código | --versao | --ajuda]

  chimalab               console interativo (linha em branco fecha um bloco; Ctrl+D sai)
  chimalab arquivo.chl   executa o arquivo
  chimalab -c \"código\"   executa o código dado
  --documento arquivo    executa como documento: só a última expressão aparece
  --versao               mostra a versão
  --ajuda                mostra esta ajuda

Dentro do console, help() lista as funções embutidas.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut documento = false;
    let src = match args.as_slice() {
        [] => {
            let stdin = io::stdin();
            let interativo = stdin.is_terminal();
            if interativo {
                println!(
                    "ChimaLab {} — linha em branco fecha um bloco; Ctrl+D sai.",
                    env!("CARGO_PKG_VERSION")
                );
            }
            return match sessao(stdin.lock(), &mut io::stdout(), interativo) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("chimalab: {e}");
                    ExitCode::FAILURE
                }
            };
        }
        [a] if a == "--ajuda" || a == "--help" || a == "-h" => {
            println!("{USO}");
            return ExitCode::SUCCESS;
        }
        [v] if v == "--versao" || v == "--version" => {
            println!("chimalab {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        [c, codigo] if c == "-c" => codigo.clone(),
        [d, arquivo] if d == "--documento" => {
            documento = true;
            match std::fs::read_to_string(arquivo) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("chimalab: não foi possível ler {arquivo}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        [arquivo] if !arquivo.starts_with('-') => match std::fs::read_to_string(arquivo) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("chimalab: não foi possível ler {arquivo}: {e}");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("{}", USO.lines().next().unwrap_or(USO));
            return ExitCode::from(2);
        }
    };
    let (texto, erro) = if documento {
        executa_documento(&src)
    } else {
        executa_programa(&src)
    };
    if !texto.is_empty() {
        println!("{texto}");
    }
    let _ = io::stdout().flush();
    match erro {
        Some(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
        None => ExitCode::SUCCESS,
    }
}
