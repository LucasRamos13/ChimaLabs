#!/usr/bin/env python3
"""Gera referências de ln Γ(z) (ramo principal) e Γ(z) complexos para F042 só com a biblioteca
padrão (decimal, 100 dígitos).

Método, escolhido para ser independente do código Rust, que usa reflexão:
- desloca z para w = z + N com Re w ≥ 200 e soma os logaritmos principais,
  ln Γ(z) = ln Γ(w) − Σₖ ln(z + k) (DLMF 5.5.1). Cada ln é principal, então o resultado é o ramo
  principal de ln Γ: contínuo fora do semieixo real negativo e, sobre ele (z = x + 0i, x < 0),
  igual ao limite por cima;
- ln Γ(w) pela série de Stirling com 40 termos de Bernoulli exatos (DLMF 5.11.1). Para
  |w| ≥ 200 o último termo é menor que 1e-120.

Validação (o script falha se não passar):
- no eixo real positivo, Re ln Γ reproduz math.lgamma e Im ln Γ = 0;
- |Γ(½ + iy)|² = π / cosh(πy) (DLMF 5.4.4);
- |Γ(iy)|² = π / (y sinh(πy)) (DLMF 5.4.3);
- Γ(z)Γ(1 − z) = π / sin(πz) (DLMF 5.5.3) em pontos fora do eixo.

Uso: python3 -I ferramentas/oraculos/f042_gama_complexa.py > tests/dados/F042/gama_complexa.csv
"""
import math
import sys
from decimal import Decimal as D, getcontext
from fractions import Fraction

getcontext().prec = 100
EPS = D(10) ** -95


def bernoulli_pares(n):
    """B₀..B_{2n} exatos por Akiyama–Tanigawa; devolve [B₂, B₄, …, B_{2n}]."""
    m = 2 * n
    a = [Fraction(0)] * (m + 1)
    b = []
    for i in range(m + 1):
        a[i] = Fraction(1, i + 1)
        for j in range(i, 0, -1):
            a[j - 1] = j * (a[j - 1] - a[j])
        b.append(a[0])
    return [b[2 * k] for k in range(1, n + 1)]


BERN = bernoulli_pares(40)


def atan_small(t):
    """atan(t) para |t| ≤ 1: três reduções pela metade e série de Taylor."""
    halvings = 0
    while abs(t) > D("0.1"):
        t = t / (1 + (1 + t * t).sqrt())
        halvings += 1
    s, p, k, t2 = D(0), t, 0, t * t
    while True:
        term = p / (2 * k + 1)
        if abs(term) < EPS * D(10) ** -5:
            break
        s += term if k % 2 == 0 else -term
        p *= t2
        k += 1
    return s * (2 ** halvings)


PI = 4 * atan_small(D(1))


def atan2(y, x):
    if x == 0 and y == 0:
        return D(0)
    if abs(y) <= abs(x):
        r = atan_small(y / x)
        if x > 0:
            return r
        return r + PI if y >= 0 else r - PI
    r = PI / 2 - atan_small(x / y)
    return r if y > 0 else r - PI


def cln(z):
    x, y = z
    return ((x * x + y * y).ln() / 2, atan2(y, x))


def cmul(a, b):
    return (a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def cdiv(a, b):
    d = b[0] * b[0] + b[1] * b[1]
    return ((a[0] * b[0] + a[1] * b[1]) / d, (a[1] * b[0] - a[0] * b[1]) / d)


def sincos(t):
    t = t % (2 * PI)
    if t > PI:
        t -= 2 * PI
    s, c, p, k = D(0), D(0), D(1), 0
    while True:
        if abs(p) < EPS * D(10) ** -5 and k > 2:
            break
        if k % 2 == 0:
            c += p if k % 4 == 0 else -p
        else:
            s += p if k % 4 == 1 else -p
        k += 1
        p = p * t / k
    return s, c


def cexp(z):
    m = z[0].exp()
    s, c = sincos(z[1])
    return (m * c, m * s)


def lngamma(z):
    x, y = D(z[0]), D(z[1])
    acc = (D(0), D(0))
    while x < 200:
        l = cln((x, y))
        acc = (acc[0] + l[0], acc[1] + l[1])
        x += 1
    w = (x, y)
    lw = cln(w)
    # (w − ½) ln w − w + ½ ln(2π) + Σ B₂ₖ / (2k(2k−1) w^(2k−1))
    r = cmul((w[0] - D("0.5"), w[1]), lw)
    r = (r[0] - w[0] + (2 * PI).ln() / 2, r[1] - w[1])
    inv = cdiv((D(1), D(0)), w)
    inv2 = cmul(inv, inv)
    p = inv
    for k, b in enumerate(BERN, start=1):
        c = D(b.numerator) / D(b.denominator) / (2 * k * (2 * k - 1))
        r = (r[0] + c * p[0], r[1] + c * p[1])
        p = cmul(p, inv2)
    return (r[0] - acc[0], r[1] - acc[1])


def gamma(z):
    return cexp(lngamma(z))


def close(a, b, rel):
    return abs(a - b) <= rel * max(abs(b), D(10) ** -300)


def validate():
    for x in [0.25, 0.5, 1.5, 3.7, 10.0, 25.5, 100.0]:
        l = lngamma((x, 0.0))
        assert l[1] == 0, (x, l)
        assert abs(float(l[0]) - math.lgamma(x)) <= 4e-16 * max(1.0, abs(math.lgamma(x))), x
    for y in [D("0.3"), D(1), D("2.5"), D(7)]:
        g = gamma((D("0.5"), y))
        cosh = ((PI * y).exp() + (-PI * y).exp()) / 2
        assert close(g[0] ** 2 + g[1] ** 2, PI / cosh, D(10) ** -80), y
        g = gamma((D(0), y))
        sinh = ((PI * y).exp() - (-PI * y).exp()) / 2
        assert close(g[0] ** 2 + g[1] ** 2, PI / (y * sinh), D(10) ** -80), y
    for z in [(D("-2.3"), D("0.7")), (D("0.4"), D("-3.1")), (D("-7.5"), D("12"))]:
        a = gamma(z)
        b = gamma((1 - z[0], -z[1]))
        prod = cmul(a, b)
        # sin(πz) = sin(πx)cosh(πy) + i cos(πx)sinh(πy)
        s, c = sincos(PI * z[0])
        ey = (PI * z[1]).exp()
        sz = (s * (ey + 1 / ey) / 2, c * (ey - 1 / ey) / 2)
        ref = cdiv((PI, D(0)), sz)
        err = ((prod[0] - ref[0]) ** 2 + (prod[1] - ref[1]) ** 2).sqrt()
        assert err <= D(10) ** -80 * (ref[0] ** 2 + ref[1] ** 2).sqrt(), z
    return 3 * 4 + 7 + 3


def points():
    pts = []
    for x in [-20.5, -7.25, -3.5, -2.75, -1.5, -0.75, -0.25, 0.0, 0.125, 0.5, 1.0, 1.75, 2.0,
              3.5, 8.0, 20.0, 60.0]:
        for y in [0.0, 1e-8, 0.03125, 0.5, 1.0, 3.0, 12.0, 60.0]:
            if x <= 0 and float(x).is_integer() and y == 0:
                continue
            pts.append((x, y))
            if y != 0:
                pts.append((x, -y))
    # malha densa na região em que |ln Γ| é pequeno (entre e em volta das raízes 1 e 2)
    for i in range(11):
        for j in range(-6, 7):
            pts.append((0.5 + 0.25 * i, 0.125 * j + 1e-4))
    # perto das raízes 1 e 2, perto de polos e com |z| grande
    pts += [(1.0 + 1e-3, 2e-3), (1.0 - 0.1, 0.07), (2.0 + 0.05, -0.02), (2.0, 1e-5),
            (1.0, -1e-6), (-3.0 + 1e-6, 1e-6), (-1.0 - 1e-5, -2e-5), (-50.25, 3.0),
            (-120.5, 0.0), (150.0, -150.0), (-0.5, 200.0), (300.5, 1.0), (1e-10, 1e-10)]
    return pts


def clean(c, floor):
    mag = max(abs(c[0]), abs(c[1]), floor)
    return tuple(D(0) if abs(v) <= D(10) ** -90 * mag else v for v in c)


def main():
    n = validate()
    print("# gerado por ferramentas/oraculos/f042_gama_complexa.py (decimal, 100 dígitos)")
    print(f"# validado por {n} identidades e math.lgamma; lngamma = ramo principal")
    print("re,im,lngamma_re,lngamma_im,gamma_re,gamma_im")
    for x, y in points():
        l = lngamma((D(x), D(y)))  # valor binário exato do f64
        g = cexp(l)
        # Resíduos de truncamento (< 1e-90 relativo) viram zero exato: ln Γ(1) = ln Γ(2) = 0 e
        # Γ é real sobre o eixo real.
        l, g = clean(l, D(1)), clean(g, D(0))
        print(f"{x!r},{y!r},{float(l[0])!r},{float(l[1])!r},{float(g[0])!r},{float(g[1])!r}")


if __name__ == "__main__":
    sys.exit(main())
