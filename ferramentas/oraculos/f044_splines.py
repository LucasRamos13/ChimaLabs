#!/usr/bin/env python3
"""F044 — splines cúbicas de referência, a 40 dígitos (mpmath, só em desenvolvimento; LICENCAS D13).

Formulação independente da implementação Rust. O Rust resolve um sistema tridiagonal nas primeiras
derivadas sᵢ. Aqui as incógnitas são as segundas derivadas (momentos) Mᵢ, num sistema denso
resolvido por eliminação em alta precisão. As condições de extremo vêm das definições:
- natural: M₀ = M_{n−1} = 0;
- derivadas dadas: S'(x₀) = a e S'(x_{n−1}) = b;
- not-a-knot: S''' contínua em x₁ e x_{n−2};
- periódica: S, S' e S'' iguais nos extremos.

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f044_splines.py > tests/dados/F044/splines.csv
"""
import mpmath as mp

mp.mp.dps = 40

CONJUNTOS = [
    # nome, x, y
    ("irregular", [0, 0.3, 1.1, 1.7, 2.0, 3.5, 4.2, 6.0],
     [1.0, -0.4, 2.2, 2.1, 0.5, -1.3, 0.0, 3.0]),
    ("quatro", [-2, -1, 0.5, 3], [4, 1, -0.25, 9]),
    ("periodico", [0, 0.7, 1.5, 2.9, 3.6, 5.0], [1.0, 0.2, -0.8, 0.3, 1.4, 1.0]),
    ("cinco", [1, 2, 4, 5, 8], [0.0, 1.0, -1.0, 2.0, 0.5]),
]
CONDICOES = ["notaknot", "natural", "clamped", "periodic"]
DERIV = (mp.mpf("0.5"), mp.mpf("-1.25"))  # derivadas dadas nos extremos


def spline(x, y, cond):
    n = len(x)
    h = [x[i + 1] - x[i] for i in range(n - 1)]
    d = [(y[i + 1] - y[i]) / h[i] for i in range(n - 1)]
    a = mp.zeros(n, n)
    r = mp.zeros(n, 1)
    # Continuidade de S' nos nós internos: h_{i−1}M_{i−1} + 2(h_{i−1}+h_i)M_i + h_i M_{i+1} = 6(δ_i − δ_{i−1}).
    for i in range(1, n - 1):
        a[i, i - 1] = h[i - 1]
        a[i, i] = 2 * (h[i - 1] + h[i])
        a[i, i + 1] = h[i]
        r[i] = 6 * (d[i] - d[i - 1])
    if cond == "natural":
        a[0, 0] = 1
        a[n - 1, n - 1] = 1
    elif cond == "clamped":
        # S'(x₀) = δ₀ − h₀(2M₀ + M₁)/6 = a;  S'(x_{n−1}) = δ + h(M_{n−2} + 2M_{n−1})/6 = b.
        a[0, 0] = 2 * h[0]
        a[0, 1] = h[0]
        r[0] = 6 * (d[0] - DERIV[0])
        a[n - 1, n - 2] = h[-1]
        a[n - 1, n - 1] = 2 * h[-1]
        r[n - 1] = 6 * (DERIV[1] - d[-1])
    elif cond == "notaknot":
        # S''' = (M_{i+1} − M_i)/h_i igual nos dois lados de x₁ e de x_{n−2}.
        a[0, 0] = -1 / h[0]
        a[0, 1] = 1 / h[0] + 1 / h[1]
        a[0, 2] = -1 / h[1]
        a[n - 1, n - 3] = -1 / h[-2]
        a[n - 1, n - 2] = 1 / h[-2] + 1 / h[-1]
        a[n - 1, n - 1] = -1 / h[-1]
    else:  # periódica: M₀ = M_{n−1} e S'(x₀) = S'(x_{n−1})
        a[0, 0] = 1
        a[0, n - 1] = -1
        a[n - 1, 0] = 2 * h[0]
        a[n - 1, 1] = h[0]
        a[n - 1, n - 2] = h[-1]
        a[n - 1, n - 1] = 2 * h[-1]
        r[n - 1] = 6 * (d[0] - d[-1])
    m = mp.lu_solve(a, r)

    def s(t):
        i = max(0, min(n - 2, max(k for k in range(n - 1) if x[k] <= t) if t >= x[0] else 0))
        hi = h[i]
        u = x[i + 1] - t
        v = t - x[i]
        return (m[i] * u**3 + m[i + 1] * v**3) / (6 * hi) + (y[i] - m[i] * hi**2 / 6) * u / hi + \
            (y[i + 1] - m[i + 1] * hi**2 / 6) * v / hi

    return s


def main():
    print("conjunto,condicao,x,valor")
    for nome, x, y in CONJUNTOS:
        x = [mp.mpf(v) for v in x]
        y = [mp.mpf(v) for v in y]
        conds = ["periodic"] if nome == "periodico" else ["notaknot", "natural", "clamped"]
        for cond in conds:
            s = spline(x, y, cond)
            k = 37
            for j in range(k + 1):
                t = x[0] + (x[-1] - x[0]) * mp.mpf(j) / k
                print(f"{nome},{cond},{mp.nstr(t, 25)},{mp.nstr(s(t), 25)}")


if __name__ == "__main__":
    main()
