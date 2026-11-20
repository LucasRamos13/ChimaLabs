#!/usr/bin/env python3
"""Gera referências de w(z) (Faddeeva), erf, erfc e erfcx complexos para F042 só com a biblioteca
padrão (decimal, 250 dígitos).

Método, escolhido para ser independente do código Rust (que usa soma trapezoidal com correção de
polo e fração contínua): série de Taylor erf z = (2/√π) Σ (−1)ⁿ z^(2n+1) / (n!(2n+1))
(DLMF 7.6.1), que converge em todo o plano. Para |z| ≤ 12 os termos chegam a e^(|z|²) ≈ 1e63 e
erfc pode valer 1e-64; com 250 dígitos sobram mais de 100 dígitos corretos. Daí:
erfc = 1 − erf, erfcx = e^(z²)·erfc(z) e w(z) = e^(−z²)·erfc(−iz) (DLMF 7.2.3).

Validação (o script falha se não passar):
- no eixo real, erf e erfc reproduzem math.erf e math.erfc;
- w(x) real tem parte real e^(−x²) (DLMF 7.19);
- erf(z̄) = conj(erf z) e erf(−z) = −erf z (DLMF 7.4.1 e 7.4.2) com π de Machin.

Uso: python3 -I ferramentas/oraculos/f042_erro_complexo.py > tests/dados/F042/erro_complexo.csv
"""
import math
import sys
from decimal import Decimal as D, getcontext

getcontext().prec = 250
TINY = D(10) ** -240


def pi():
    def atan_inv(n):
        x = D(1) / n
        x2 = x * x
        s, t, k = D(0), x, 0
        while True:
            term = t / (2 * k + 1)
            if abs(term) < TINY:
                break
            s += term if k % 2 == 0 else -term
            t *= x2
            k += 1
        return s
    return 16 * atan_inv(5) - 4 * atan_inv(239)


PI = pi()
TWO_OVER_SQRT_PI = 2 / PI.sqrt()


def cmul(a, b):
    return (a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def sincos(t):
    t = t % (2 * PI)
    if t > PI:
        t -= 2 * PI
    s, c, p, k = D(0), D(0), D(1), 0
    while True:
        if abs(p) < TINY and k > 2:
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


def erf(z):
    z2 = cmul(z, z)
    mz2 = (-z2[0], -z2[1])
    term = z            # (−1)ⁿ z^(2n+1) / n!
    s = z
    n = 0
    while True:
        n += 1
        term = cmul(term, mz2)
        term = (term[0] / n, term[1] / n)
        add = (term[0] / (2 * n + 1), term[1] / (2 * n + 1))
        s = (s[0] + add[0], s[1] + add[1])
        if n > 10 and max(abs(add[0]), abs(add[1])) < TINY:
            break
    return (TWO_OVER_SQRT_PI * s[0], TWO_OVER_SQRT_PI * s[1])


def all_funcs(x, y):
    z = (D(x), D(y))
    e = erf(z)
    erfc = (1 - e[0], -e[1])
    z2 = cmul(z, z)
    erfcx = cmul(cexp(z2), erfc)
    # w(z) = e^(−z²) erfc(−iz); −iz = (y, −x)
    e2 = erf((z[1], -z[0]))
    erfc_miz = (1 - e2[0], -e2[1])
    w = cmul(cexp((-z2[0], -z2[1])), erfc_miz)
    return w, e, erfc, erfcx


def validate():
    n = 0
    for x in [0.0, 0.3, 1.0, 2.5, -1.7, 5.0]:
        _, e, ec, _ = all_funcs(x, 0.0)
        assert e[1] == 0 and abs(float(e[0]) - math.erf(x)) <= 2e-16 * max(abs(math.erf(x)), 1e-300), x
        assert abs(float(ec[0]) - math.erfc(x)) <= 4e-16 * math.erfc(x), x
        n += 2
    for x in [0.5, 2.0, 6.0]:
        w, _, _, _ = all_funcs(x, 0.0)
        assert abs(w[0] - (-D(x) ** 2).exp()) < D(10) ** -100, x
        n += 1
    for x, y in [(1.3, 0.7), (-2.2, 3.1), (0.4, -5.5)]:
        _, a, _, _ = all_funcs(x, y)
        _, b, _, _ = all_funcs(x, -y)
        _, m, _, _ = all_funcs(-x, -y)
        assert abs(a[0] - b[0]) < D(10) ** -100 and abs(a[1] + b[1]) < D(10) ** -100
        assert abs(a[0] + m[0]) < D(10) ** -100 and abs(a[1] + m[1]) < D(10) ** -100
        n += 2
    return n


def points():
    pts = []
    vals = [0.0, 1e-9, 0.01, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.5, 7.0, 7.9, 8.1, 10.0, 12.0]
    for x in vals:
        for y in vals:
            if x * x + y * y > 144.0:
                continue
            for sx, sy in [(1, 1), (-1, 1), (1, -1), (-1, -1)]:
                pts.append((sx * x, sy * y))
    # perto dos nós das duas malhas (h = 0,35 e meio-passo), perto do eixo real
    for k in range(1, 20):
        for d in [0.0, 1e-7, 0.05]:
            pts.append((0.35 * k + d, 1e-3))
            pts.append((0.35 * (k + 0.5) + d, 0.0))
    pts += [(1.4, 1.9), (1.45061616, 1.88094300), (-0.3, 6.1), (6.3, 0.2), (3.0, -2.0)]
    seen, out = set(), []
    for p in pts:
        key = (float(p[0]), float(p[1]))
        if key not in seen:
            seen.add(key)
            out.append(key)
    return out


def fmt(c):
    return f"{float(c[0])!r},{float(c[1])!r}"


def main():
    n = validate()
    print("# gerado por ferramentas/oraculos/f042_erro_complexo.py (decimal, 250 dígitos; Taylor)")
    print(f"# validado por {n} verificações (math.erf/erfc, Re w(x) = e^(-x²), simetrias)")
    print("re,im,w_re,w_im,erf_re,erf_im,erfc_re,erfc_im,erfcx_re,erfcx_im")
    for x, y in points():
        w, e, ec, ex = all_funcs(x, y)
        print(f"{x!r},{y!r},{fmt(w)},{fmt(e)},{fmt(ec)},{fmt(ex)}")


if __name__ == "__main__":
    sys.exit(main())
