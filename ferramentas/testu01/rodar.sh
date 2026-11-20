#!/bin/sh
# Roda uma bateria do TestU01 sobre um gerador do F053.
# Uso: TESTU01=<prefixo da instalação> ferramentas/testu01/rodar.sh <gerador> <semente> <bits|u01> <small|crush|big>
# O TestU01 1.2.3 (Apache-2.0) é obtido em https://simul.iro.umontreal.ca/testu01/tu01.html,
# compilado fora do repositório (./configure --prefix=... --disable-shared) e nunca versionado.
set -eu
: "${TESTU01:?defina TESTU01 com o prefixo da instalação do TestU01}"
raiz=$(cd "$(dirname "$0")/../.." && pwd)
saida=${SAIDA:-$raiz/target/testu01}
mkdir -p "$saida"
cc -O2 -w -I"$TESTU01/include" "$raiz/ferramentas/testu01/bateria.c" \
    "$TESTU01/lib/libtestu01.a" "$TESTU01/lib/libprobdist.a" "$TESTU01/lib/libmylib.a" -lm \
    -o "$saida/bateria"
cargo build -q --release -p chimalab-math --example fluxo_aleatorio
"$raiz/target/release/examples/fluxo_aleatorio" "$1" "$2" "$3" | "$saida/bateria" "$3" "$4" "$1-$2-$3"
