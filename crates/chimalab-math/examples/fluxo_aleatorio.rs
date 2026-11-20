//! Escreve na saída padrão um fluxo binário de um gerador do F053, para baterias estatísticas
//! externas (ver `ferramentas/testu01/`).
//!
//! Uso: `fluxo_aleatorio <gerador> <semente> <bits|u01>`
//! - `bits`: palavras de 32 bits de `next_u32`, little-endian;
//! - `u01`: f64 de `uniform`, little-endian (o que o usuário consome).
//!
//! Termina sem erro quando o leitor fecha o tubo.

use std::io::{self, Write};

use chimalab_math::random::{Algorithm, Rng};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("uso: fluxo_aleatorio <gerador> <semente> <bits|u01>");
        std::process::exit(2);
    }
    let alg = Algorithm::from_name(&args[1]).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2)
    });
    let seed: u64 = args[2].parse().expect("semente inteira");
    let u01 = match args[3].as_str() {
        "bits" => false,
        "u01" => true,
        _ => {
            eprintln!("modo deve ser bits ou u01");
            std::process::exit(2)
        }
    };
    let mut rng = Rng::new(alg, seed);
    let mut out = io::stdout().lock();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        if u01 {
            for c in buf.as_chunks_mut::<8>().0 {
                c.copy_from_slice(&rng.uniform().to_le_bytes());
            }
        } else {
            for c in buf.as_chunks_mut::<4>().0 {
                c.copy_from_slice(&rng.next_u32().to_le_bytes());
            }
        }
        if out.write_all(&buf).is_err() {
            return;
        }
    }
}
