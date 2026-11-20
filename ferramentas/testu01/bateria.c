/*
 * Harness do ChimaLab para as baterias do TestU01 (L'Ecuyer & Simard, ACM TOMS 33, 2007).
 * Código próprio (MIT OR Apache-2.0); o TestU01 (Apache-2.0) é baixado e compilado fora do
 * repositório e não é redistribuído (LICENCAS D19). Ver ferramentas/testu01/rodar.sh.
 *
 * Lê da entrada padrão o fluxo de `fluxo_aleatorio` e roda a bateria pedida.
 * Uso: bateria <bits|u01> <small|crush|big> <nome>
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

#include "unif01.h"
#include "bbattery.h"

#define N 8192
static uint32_t buf32[N];
static double buf64[N];
static size_t pos = N;

static void falta(void) {
    fprintf(stderr, "fluxo terminou antes da bateria\n");
    exit(3);
}

static unsigned int ler_bits(void) {
    if (pos == N) {
        if (fread(buf32, sizeof buf32[0], N, stdin) != N) falta();
        pos = 0;
    }
    return buf32[pos++];
}

static double ler_u01(void) {
    if (pos == N) {
        if (fread(buf64, sizeof buf64[0], N, stdin) != N) falta();
        pos = 0;
    }
    return buf64[pos++];
}

int main(int argc, char **argv) {
    unif01_Gen *g;
    if (argc != 4) {
        fprintf(stderr, "uso: bateria <bits|u01> <small|crush|big> <nome>\n");
        return 2;
    }
    if (strcmp(argv[1], "bits") == 0)
        g = unif01_CreateExternGenBits(argv[3], ler_bits);
    else
        g = unif01_CreateExternGen01(argv[3], ler_u01);
    if (strcmp(argv[2], "small") == 0)
        bbattery_SmallCrush(g);
    else if (strcmp(argv[2], "crush") == 0)
        bbattery_Crush(g);
    else
        bbattery_BigCrush(g);
    if (strcmp(argv[1], "bits") == 0)
        unif01_DeleteExternGenBits(g);
    else
        unif01_DeleteExternGen01(g);
    return 0;
}
