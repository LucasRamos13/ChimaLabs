#!/usr/bin/env python3
"""F043 — referências exatas para estatística descritiva (biblioteca padrão: fractions).

Cada f64 dos dados é convertido em fração exata. As estatísticas são calculadas em aritmética
racional e só no fim arredondadas para f64 (repr de 17 dígitos). Os quantis seguem as definições
de R. J. Hyndman & Y. Fan, Am. Stat. 50 (1996) 361–365, implementadas aqui de forma direta e
independente do código Rust. A probabilidade p é o decimal pretendido (0,1 = 1/10 exato). O Rust
trata n·p a poucos ε de um inteiro como inteiro (como a "fuzz" de R), e por isso coincide.

Saídas: `tests/dados/F043/dados.csv` (conjunto, índice, valor) e
`tests/dados/F043/referencias.csv` (conjunto, estatística, parâmetro, valor).

Uso: python3 -I ferramentas/oraculos/f043_estatisticas.py tests/dados/F043
"""
import math
import sys
from fractions import Fraction as F


def lcg(seed):
    s = seed
    while True:
        s = (s * 6364136223846793005 + 1442695040888963407) % 2**64
        yield (s >> 11) / 2**53


def conjuntos():
    g = lcg(7)
    yield "pequeno", [3.0, -1.5, 2.25, 7.0, 0.5, 2.25, -4.0]
    yield "media_grande", [1e9 + next(g) for _ in range(200)]
    # Soma mal condicionada: valores enormes que se cancelam e pequenos que sobrevivem.
    yield "cancelamento", [1e16, 1.0, -1e16, 1.0, 3e15, 0.5, -3e15, 0.25] * 5
    yield "perto_do_overflow", [1.7e308, 1.6e308, 1.5e308, 1.2e308]
    yield "uniforme", [next(g) * 10 - 5 for _ in range(101)]
    yield "par", [5.0, 1.0, 4.0, 2.0, 3.0, 6.0]


def quantil(v, p, tipo):
    n = len(v)
    x = sorted(F(a) for a in v)
    p = F(p)

    def at(j):
        return x[min(max(j, 1), n) - 1]

    if tipo in (1, 2, 3):
        m = {1: F(0), 2: F(0), 3: F(-1, 2)}[tipo]
        np_m = n * p + m
        j = math.floor(np_m)
        g = np_m - j
        if tipo == 1:
            return at(j) if g == 0 else at(j + 1)
        if tipo == 2:
            return (at(j) + at(j + 1)) / 2 if g == 0 else at(j + 1)
        return at(j) if (g == 0 and j % 2 == 0) else at(j + 1)
    m = {4: F(0), 5: F(1, 2), 6: p, 7: 1 - p, 8: (p + 1) / 3, 9: p / 4 + F(3, 8)}[tipo]
    np_m = n * p + m
    j = math.floor(np_m)
    g = np_m - j
    return (1 - g) * at(j) + g * at(j + 1)


def f64(x):
    """Fração para f64; acima do maior finito, ±inf (o valor verdadeiro não é representável)."""
    try:
        return repr(float(x))
    except OverflowError:
        return "inf" if x > 0 else "-inf"


def main():
    pasta = sys.argv[1]
    with open(f"{pasta}/dados.csv", "w") as fd, open(f"{pasta}/referencias.csv", "w") as fr:
        fd.write("conjunto,i,valor\n")
        fr.write("conjunto,estatistica,parametro,valor\n")
        for nome, v in conjuntos():
            for i, a in enumerate(v):
                fd.write(f"{nome},{i},{a!r}\n")
            q = [F(a) for a in v]
            n = len(q)
            s = sum(q)
            m = s / n
            ss = sum((a - m) ** 2 for a in q)
            fr.write(f"{nome},soma,0,{f64(s)}\n")
            fr.write(f"{nome},media,0,{f64(m)}\n")
            fr.write(f"{nome},var_amostral,0,{f64(ss / (n - 1))}\n")
            fr.write(f"{nome},var_populacional,0,{f64(ss / n)}\n")
            for tipo in range(1, 10):
                for p in ["0", "0.1", "0.25", "0.5", "0.7", "0.9", "1"]:
                    fr.write(f"{nome},q{tipo},{p},{f64(quantil(v, F(p), tipo))}\n")


if __name__ == "__main__":
    main()
