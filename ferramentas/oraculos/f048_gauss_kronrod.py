#!/usr/bin/env python3
"""Deriva os nós e pesos da regra de Gauss–Kronrod 7–15 em [−1, 1] só com a biblioteca padrão.

Método (Kronrod, 1965; a construção, não tabelas publicadas):
- G7: raízes de P₇ (Legendre) por Newton em decimal de 60 dígitos; pesos 2/((1 − x²)P₇′(x)²).
- K15: os 8 nós novos são as raízes do polinômio de Stieltjes E₈, mônico e par, definido por
  ∫ P₇(x) E₈(x) xᵏ dx = 0 para k = 0..7. Por paridade só k ímpar dá equação; o sistema 4×4 é
  resolvido em aritmética racional exata (Fraction). Com u = x², E₈ vira uma quártica em u,
  cujas raízes são separadas por bisseção e refinadas por Newton.
- Pesos de K15: exatidão nos monômios pares x⁰, x², …, x¹⁴ (8 incógnitas por simetria),
  sistema linear resolvido em decimal.

Validação (o script falha se não passar): K15 integra exatamente xᵐ para m ≤ 22 e G7 para
m ≤ 13 (erro < 1e-50); os nós de Kronrod intercalam os de Gauss.

Uso: python3 -I ferramentas/oraculos/f048_gauss_kronrod.py  (imprime as constantes em Rust)
"""
import sys
from decimal import Decimal as D, getcontext
from fractions import Fraction as F

getcontext().prec = 60
N = 7


def legendre_coeffs(n):
    """Coeficientes exatos de Pₙ (índice = grau), pela recorrência de Bonnet."""
    p0, p1 = [F(1)], [F(0), F(1)]
    if n == 0:
        return p0
    for k in range(1, n):
        nxt = [F(0)] * (k + 2)
        for i, c in enumerate(p1):
            nxt[i + 1] += F(2 * k + 1, k + 1) * c
        for i, c in enumerate(p0):
            nxt[i] -= F(k, k + 1) * c
        p0, p1 = p1, nxt
    return p1


def integral_monomial(m):
    return F(0) if m % 2 else F(2, m + 1)


def poly_eval(c, x):
    s = D(0)
    for a in reversed(c):
        s = s * x + D(a.numerator) / D(a.denominator)
    return s


def poly_deriv(c):
    return [c[i] * i for i in range(1, len(c))]


def newton(c, x):
    dc = poly_deriv(c)
    for _ in range(200):
        dx = poly_eval(c, x) / poly_eval(dc, x)
        x -= dx
        if abs(dx) < D(10) ** -58:
            break
    return x


def solve(a, b):
    """Eliminação de Gauss com pivotamento parcial (genérica: Fraction ou Decimal)."""
    n = len(b)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for col in range(n):
        piv = max(range(col, n), key=lambda r: abs(m[r][col]))
        m[col], m[piv] = m[piv], m[col]
        for r in range(n):
            if r != col and m[r][col] != 0:
                f = m[r][col] / m[col][col]
                m[r] = [x - f * y for x, y in zip(m[r], m[col])]
    return [m[i][n] / m[i][i] for i in range(n)]


def main():
    p = legendre_coeffs(N)
    # Gauss: raízes de P₇ (chutes de Tricomi: cos(π(i − ¼)/(n + ½))).
    pi = D("3.14159265358979323846264338327950288419716939937510582097494")
    def cos(t):
        s, term, k = D(0), D(1), 0
        while abs(term) > D(10) ** -60:
            s += term
            k += 2
            term = -term * t * t / ((k - 1) * k)
        return s
    gauss = sorted(newton(p, cos(pi * (D(i) - D("0.25")) / (D(N) + D("0.5")))) for i in range(1, N + 1))
    dp = poly_deriv(p)
    gw = [2 / ((1 - x * x) * poly_eval(dp, x) ** 2) for x in gauss]

    # Stieltjes: E₈ = x⁸ + c₆x⁶ + c₄x⁴ + c₂x² + c₀, ∫ P₇ E₈ xᵏ = 0, k = 1, 3, 5, 7.
    def moment(poly, k):
        return sum(c * integral_monomial(i + k) for i, c in enumerate(poly))
    unknown_powers = [0, 2, 4, 6]
    rows, rhs = [], []
    for k in (1, 3, 5, 7):
        rows.append([moment(_mul(p, j), k) for j in unknown_powers])
        rhs.append(-moment(_mul(p, 8), k))
    coef = solve(rows, rhs)
    q = coef + [F(1)]  # quártica em u: c₀ + c₂u + c₄u² + c₆u³ + u⁴
    # Raízes em (0, 1): bisseção numa malha fina, depois Newton.
    roots_u = []
    grid = [D(i) / 4000 for i in range(4001)]
    vals = [poly_eval(q, u) for u in grid]
    for i in range(4000):
        if vals[i] == 0 or vals[i] * vals[i + 1] < 0:
            lo, hi = grid[i], grid[i + 1]
            for _ in range(60):
                mid = (lo + hi) / 2
                if poly_eval(q, lo) * poly_eval(q, mid) <= 0:
                    hi = mid
                else:
                    lo = mid
            roots_u.append(newton(q, (lo + hi) / 2))
    assert len(roots_u) == 4, roots_u
    kron_new = sorted([-u.sqrt() for u in roots_u] + [u.sqrt() for u in roots_u])
    nodes = sorted(gauss + kron_new)
    # Intercalação: Kronrod e Gauss alternam.
    is_gauss = [any(abs(x - g) < D(10) ** -40 for g in gauss) for x in nodes]
    assert all(is_gauss[i] != is_gauss[i + 1] for i in range(14)), "não intercalam"

    # Pesos de K15 por simetria: incógnitas nos nós ≥ 0 (8), equações nos monômios pares 0..14.
    pos = [x for x in nodes if x >= -D(10) ** -50]
    pos = sorted(pos)
    a, b = [], []
    for m in range(0, 15, 2):
        a.append([(D(1) if x == 0 or abs(x) < D(10) ** -50 else D(2)) * (x ** m if m else D(1)) for x in pos])
        im = integral_monomial(m)
        b.append(D(im.numerator) / D(im.denominator))
    wpos = solve(a, b)
    kw = {x: w for x, w in zip(pos, wpos)}

    def kweight(x):
        return kw[min(kw, key=lambda y: abs(y - abs(x)))]

    def gweight(x):
        return gw[min(range(N), key=lambda i: abs(gauss[i] - x))]

    # Validação de exatidão.
    for m in range(0, 23):
        exact = integral_monomial(m)
        exact = D(exact.numerator) / D(exact.denominator)
        k = sum(kweight(x) * x ** m for x in nodes)
        assert abs(k - exact) < D(10) ** -50, ("K15", m, k - exact)
        if m <= 13:
            g = sum(gweight(x) * x ** m for x in gauss)
            assert abs(g - exact) < D(10) ** -50, ("G7", m)
    # O grau 24 já não é exato (confirma que a regra é a de grau 23).
    m = 24
    exact = D(2) / (m + 1)
    assert abs(sum(kweight(x) * x ** m for x in nodes) - exact) > D(10) ** -20

    print("// Gerado por ferramentas/oraculos/f048_gauss_kronrod.py (decimal, 60 dígitos);")
    print("// exatidão conferida: K15 até x²², G7 até x¹³.")
    print("/// Nós de Kronrod em [0, 1], decrescentes; os de índice ímpar são os nós de Gauss.")
    print("const XK: [f64; 8] = [")
    for x in reversed(pos):
        print(f"    {0.0 if abs(x) < D(10) ** -50 else float(x)!r},")
    print("];")
    print("/// Pesos de Kronrod correspondentes a `XK`.")
    print("const WK: [f64; 8] = [")
    for x in reversed(pos):
        print(f"    {float(kw[x])!r},")
    print("];")
    print("/// Pesos de Gauss dos nós `XK[1]`, `XK[3]`, `XK[5]` e `XK[7]` (= 0).")
    print("const WG: [f64; 4] = [")
    for x in reversed(pos):
        if any(abs(x - g) < D(10) ** -40 for g in gauss):
            print(f"    {float(gweight(x))!r},")
    print("];")


def _mul(poly, k):
    """poly · xᵏ."""
    return [F(0)] * k + list(poly)


if __name__ == "__main__":
    sys.exit(main())
