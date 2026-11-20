#!/usr/bin/env python3
"""F054 — referências para a aritmética dupla-dupla (mpmath a 300 bits, ~90 dígitos).

Gera entradas dupla-dupla pseudoaleatórias (semente fixa) com hi em escala log-uniforme e
lo ∈ (−½, ½)·ulp(hi), e o resultado exato de cada operação, arredondado para dupla-dupla
(hi = f64 mais próximo, lo = f64 mais próximo do resto). Cada número sai em repr do Python
(menor decimal que reproduz o f64 exatamente).

Linhas: op,a_hi,a_lo,b_hi,b_lo,r_hi,r_lo  (b = 0 para as operações unárias; em "powi", b_hi é
o expoente inteiro). Linhas "const": nome,valor_hi,valor_lo. Linhas "parse": texto,r_hi,r_lo.

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f054_dd.py > tests/dados/F054/dd.csv
"""
import math
import random

import mpmath as mp

mp.mp.prec = 300


def dd(v):
    hi = float(v)
    lo = float(v - mp.mpf(hi))
    return hi, lo


def aleat(rng, emin, emax, sinal=True):
    e = rng.uniform(emin, emax)
    hi = 10.0 ** e
    if sinal and rng.random() < 0.5:
        hi = -hi
    lo = rng.uniform(-0.5, 0.5) * math.ulp(hi)
    return hi, lo


def val(p):
    return mp.mpf(p[0]) + mp.mpf(p[1])


def main():
    rng = random.Random(54)
    print("op,a_hi,a_lo,b_hi,b_lo,r_hi,r_lo")
    for _ in range(200):
        a, b = aleat(rng, -20, 20), aleat(rng, -20, 20)
        # Somas com cancelamento: b ≈ −a.
        if rng.random() < 0.25:
            b = (-a[0], rng.uniform(-0.5, 0.5) * math.ulp(a[0]))
        for op, r in [("add", val(a) + val(b)), ("sub", val(a) - val(b)), ("mul", val(a) * val(b)), ("div", val(a) / val(b))]:
            print(op, *map(repr, a), *map(repr, b), *map(repr, dd(r)), sep=",")
    for _ in range(100):
        a = aleat(rng, -30, 30, sinal=False)
        print("sqrt", *map(repr, a), 0.0, 0.0, *map(repr, dd(mp.sqrt(val(a)))), sep=",")
        print("ln", *map(repr, a), 0.0, 0.0, *map(repr, dd(mp.log(val(a)))), sep=",")
        x = (rng.uniform(-700, 700), 0.0)
        x = (x[0], rng.uniform(-0.5, 0.5) * math.ulp(x[0]))
        print("exp", *map(repr, x), 0.0, 0.0, *map(repr, dd(mp.exp(val(x)))), sep=",")
        y = aleat(rng, -2, 2, sinal=False)
        n = rng.randint(-40, 40)
        print("powi", *map(repr, y), float(n), 0.0, *map(repr, dd(val(y) ** n)), sep=",")
    for nome, v in [("pi", mp.pi), ("e", mp.e), ("ln2", mp.log(2))]:
        print("const", nome, *map(repr, dd(v)), 0.0, 0.0, 0.0, 0.0, sep=",")
    for txt in ["3.14159265358979323846264338327950288", "-1.2345678901234567890123456789012e-150",
                "0.1", "1e300", "2.5e-310", "123456789012345678901234567890123456789", "-0.000000000000000000001"]:
        print("parse", txt, 0.0, 0.0, 0.0, *map(repr, dd(mp.mpf(txt))), sep=",")


if __name__ == "__main__":
    main()
