#!/usr/bin/env python3
"""F050 — referência exata para o exemplo 1 de L. F. Shampine & S. Thompson, Appl. Numer. Math.
37 (2001) 441–458 (problema publicado):
    y1' = y1(t − 1)
    y2' = y1(t − 1) + y2(t − 0,2)
    y3' = y2(t)
com histórico y = 1 para t ≤ 0, em [0, 5].

Com histórico constante, a solução é um polinômio em cada intervalo [k/5, (k+1)/5] (método dos
passos). Os polinômios são calculados em aritmética racional exata (fractions): integração
termo a termo, deslocamentos p(t − c) pela expansão binomial e continuidade nas emendas. Os
valores saem exatos e só são arredondados para f64 no fim. Só a biblioteca padrão.

Uso: python3 -I ferramentas/oraculos/f050_dde.py > tests/dados/F050/exemplo1.csv
"""
from fractions import Fraction as F
from math import comb

H = F(1, 5)


def shift(p, c):
    """Coeficientes de q(t) = p(t − c)."""
    q = [F(0)] * len(p)
    for k, a in enumerate(p):
        for j in range(k + 1):
            q[j] += a * comb(k, j) * (-c) ** (k - j)
    return q


def add(p, q):
    n = max(len(p), len(q))
    return [(p[i] if i < len(p) else 0) + (q[i] if i < len(q) else 0) for i in range(n)]


def integrate(p, a, ya):
    """Primitiva P com P(a) = ya."""
    q = [F(0)] + [c / (k + 1) for k, c in enumerate(p)]
    q[0] = ya - ev(q, a)
    return q


def ev(p, t):
    return sum(c * t**k for k, c in enumerate(p))


def main():
    n = 25  # [0, 5] em passos de 1/5
    um = [F(1)]
    y = [[], [], []]  # y[i][k]: polinômio do componente i no intervalo k
    for k in range(n):
        a = k * H
        # Valores atrasados: y1(t − 1) usa o intervalo k − 5; y2(t − 0,2) usa k − 1.
        z1 = shift(y[0][k - 5], 1) if k >= 5 else um
        z2 = shift(y[1][k - 1], H) if k >= 1 else um
        y0 = [ev(y[i][k - 1], a) if k >= 1 else F(1) for i in range(3)]
        p1 = integrate(z1, a, y0[0])
        p2 = integrate(add(z1, z2), a, y0[1])
        p3 = integrate(p2, a, y0[2])
        for i, p in enumerate([p1, p2, p3]):
            y[i].append(p)
    print("t,y1,y2,y3")
    for j in range(1, 11):
        t = F(j, 2)
        k = min(int(t / H), n - 1)
        if t == k * H and k > 0:
            k -= 1
        print(",".join([repr(float(t))] + [repr(float(ev(y[i][k], t))) for i in range(3)]))


if __name__ == "__main__":
    main()
