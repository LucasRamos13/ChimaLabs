#!/usr/bin/env python3
"""F044 — interpolantes de Hermite cúbicos de referência (pchip, Akima, makima), a 40 dígitos
(mpmath, só em desenvolvimento; LICENCAS D13).

As derivadas nos nós vêm das definições publicadas, escritas aqui de forma independente da
implementação Rust:
- pchip: média harmônica ponderada de F. N. Fritsch & J. Butland, SIAM J. Sci. Stat. Comput. 5
  (1984) 300–304, nos nós internos (0 em extremos locais), e a fórmula de três pontos nos
  extremos com o limitador de forma (C. Moler, "Numerical Computing with MATLAB", 2004, §3.4);
- Akima: H. Akima, J. ACM 17 (1970) 589–602, com as duas secantes fictícias de cada extremo por
  extrapolação quadrática (m₋₁ = 2m₀ − m₁ etc.) e a média simples quando os dois pesos são nulos;
- makima: pesos |mᵢ₊₁ − mᵢ| + |mᵢ₊₁ + mᵢ|/2 (C. Moler, blog "Makima Piecewise Cubic
  Interpolation", 2019).
O valor em t ∈ [xᵢ, xᵢ₊₁] é a cúbica de Hermite com yᵢ, yᵢ₊₁, sᵢ, sᵢ₊₁.

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f044_hermite.py > tests/dados/F044/hermite.csv
"""
import mpmath as mp

mp.mp.dps = 40

CONJUNTOS = [
    ("irregular", [0, 0.3, 1.1, 1.7, 2.0, 3.5, 4.2, 6.0], [1.0, -0.4, 2.2, 2.1, 0.5, -1.3, 0.0, 3.0]),
    ("quatro", [-2, -1, 0.5, 3], [4, 1, -0.25, 9]),
    ("tres", [0, 1, 3], [1, -1, 2]),
    # Extremo com secantes de sinais opostos e |v| > 3|δ₀|: o limitador do pchip age.
    ("limitador", [0, 1, 1.1, 2], [0, 1, -4, -3]),
    # Platôs e monotonia (pesos nulos no Akima; makima plano).
    ("plato", [0, 1, 2, 3, 4, 5, 6], [0, 0, 0, 1, 1, 1, 2]),
    ("monotono", [0, 0.5, 0.6, 2, 2.2, 4], [0, 0.1, 1.5, 1.6, 3, 3.1]),
    # Secantes s, s, t, t: no nó do meio os dois pesos do Akima são nulos (média simples).
    ("quebra", [0, 1, 2, 3, 4], [0, 1, 2, 4, 6]),
]


def sinal(v):
    return (v > 0) - (v < 0)


def pchip(x, y):
    n = len(x)
    h = [x[i + 1] - x[i] for i in range(n - 1)]
    d = [(y[i + 1] - y[i]) / h[i] for i in range(n - 1)]
    if n == 2:
        return [d[0], d[0]]
    s = [mp.mpf(0)] * n
    for k in range(1, n - 1):
        if d[k - 1] != 0 and d[k] != 0 and sinal(d[k - 1]) == sinal(d[k]):
            w1 = 2 * h[k] + h[k - 1]
            w2 = h[k] + 2 * h[k - 1]
            s[k] = (w1 + w2) / (w1 / d[k - 1] + w2 / d[k])

    def extremo(h0, h1, d0, d1):
        v = ((2 * h0 + h1) * d0 - h0 * d1) / (h0 + h1)
        if sinal(v) != sinal(d0) or d0 == 0:
            return mp.mpf(0)
        if sinal(d0) != sinal(d1) and abs(v) > 3 * abs(d0):
            return 3 * d0
        return v

    s[0] = extremo(h[0], h[1], d[0], d[1])
    s[-1] = extremo(h[-1], h[-2], d[-1], d[-2])
    return s


def akima(x, y, modificado):
    n = len(x)
    d = [(y[i + 1] - y[i]) / (x[i + 1] - x[i]) for i in range(n - 1)]
    if n == 2:
        return [d[0], d[0]]
    # Secantes estendidas: e[j] = δ_{j−2}.
    e = [None, None] + d + [None, None]
    e[1] = 2 * e[2] - e[3]
    e[0] = 2 * e[1] - e[2]
    e[-2] = 2 * e[-3] - e[-4]
    e[-1] = 2 * e[-2] - e[-3]
    s = []
    for i in range(n):
        m1, m2, m3, m4 = e[i], e[i + 1], e[i + 2], e[i + 3]
        if modificado:
            wa = abs(m4 - m3) + abs(m4 + m3) / 2
            wb = abs(m2 - m1) + abs(m2 + m1) / 2
        else:
            wa, wb = abs(m4 - m3), abs(m2 - m1)
        s.append((m2 + m3) / 2 if wa + wb == 0 else (wa * m2 + wb * m3) / (wa + wb))
    return s


def hermite(x, y, s, t):
    n = len(x)
    i = 0
    while i < n - 2 and t >= x[i + 1]:
        i += 1
    h = x[i + 1] - x[i]
    u = (t - x[i]) / h
    h00 = 2 * u**3 - 3 * u**2 + 1
    h10 = u**3 - 2 * u**2 + u
    h01 = -2 * u**3 + 3 * u**2
    h11 = u**3 - u**2
    return h00 * y[i] + h10 * h * s[i] + h01 * y[i + 1] + h11 * h * s[i + 1]


def main():
    print("conjunto,metodo,x,valor")
    for nome, x, y in CONJUNTOS:
        x = [mp.mpf(v) for v in x]
        y = [mp.mpf(v) for v in y]
        for metodo, s in [("pchip", pchip(x, y)), ("akima", akima(x, y, False)), ("makima", akima(x, y, True))]:
            k = 29
            for j in range(k + 1):
                t = x[0] + (x[-1] - x[0]) * mp.mpf(j) / k
                print(f"{nome},{metodo},{mp.nstr(t, 25)},{mp.nstr(hermite(x, y, s, t), 25)}")


if __name__ == "__main__":
    main()
