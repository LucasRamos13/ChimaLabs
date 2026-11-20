#!/usr/bin/env python3
"""Gera referências de Ai, Bi, Ai′ e Bi′ com argumento complexo para F042 só com a biblioteca
padrão (decimal, 120 dígitos).

Método, independente do código Rust (que usa Bessel de ordem 1/3 e 2/3): séries de Maclaurin
(DLMF 9.4.1–2), Ai = c₁f − c₂g, Bi = √3(c₁f + c₂g), com c₁ = 3^(−2/3)/Γ(2/3) e
c₂ = 3^(−1/3)/Γ(1/3), e as derivadas termo a termo. São funções inteiras, e a série converge em
todo o plano. Com |z| ≤ 10, os termos chegam a e^((2/3)|z|^(3/2)) ≈ 1e9, e Ai pode valer 1e-10:
o cancelamento é de ~20 dígitos e sobram mais de 90 corretos.

Validação (o script falha se não passar):
- no eixo real, Ai e Bi reproduzem as referências do mpmath em tests/dados/F042/referencias.csv;
- Wronskiano Ai·Bi′ − Ai′·Bi = 1/π (DLMF 9.2.7) em pontos complexos;
- simetria Ai(z̄) = conj Ai(z).

Uso: python3 -I ferramentas/oraculos/f042_airy_complexo.py > tests/dados/F042/airy_complexo.csv
"""
import sys
from decimal import Decimal as D, getcontext
from pathlib import Path

getcontext().prec = 120
TINY = D(10) ** -115

GAMMA_1_3 = D("2.6789385347077476336556929409746776441286893779573")
GAMMA_2_3 = D("1.3541179394264004169452880281545137855193272660568")


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


PI = 16 * atan_inv(5) - 4 * atan_inv(239)
SQRT3 = D(3).sqrt()
assert abs(GAMMA_1_3 * GAMMA_2_3 - 2 * PI / SQRT3) < D(10) ** -45, "constantes Γ inconsistentes"
C1 = 1 / (D(3) ** (D(2) / 3) * GAMMA_2_3)
C2 = 1 / (D(3) ** (D(1) / 3) * GAMMA_1_3)


def cmul(a, b):
    return (a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def cadd(a, b):
    return (a[0] + b[0], a[1] + b[1])


def cscale(a, s):
    return (a[0] * s, a[1] * s)


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


def airy_all(z):
    """(Ai, Bi, Ai′, Bi′) pela série de Maclaurin; z complexo (par de Decimal)."""
    z2 = cmul(z, z)
    z3 = cmul(z2, z)
    # f = Σ aₖ z^(3k), g = Σ bₖ z^(3k+1); f′ = Σ 3k aₖ z^(3k−1), g′ = Σ (3k+1) bₖ z^(3k).
    a, b = D(1), D(1)          # coeficientes aₖ, bₖ
    p = (D(1), D(0))           # z^(3k)
    sf, sg = (D(1), D(0)), z
    sfp, sgp = (D(0), D(0)), (D(1), D(0))
    pm1 = None
    k = 0
    while True:
        k += 1
        a = a / ((3 * k - 1) * (3 * k))
        b = b / ((3 * k) * (3 * k + 1))
        pm1 = cmul(p, z2)      # z^(3k−1) = z^(3(k−1))·z²
        p = cmul(p, z3)        # z^(3k)
        tf = cscale(p, a)
        tg = cscale(cmul(p, z), b)
        tfp = cscale(pm1, 3 * k * a)
        tgp = cscale(p, (3 * k + 1) * b)
        sf, sg, sfp, sgp = cadd(sf, tf), cadd(sg, tg), cadd(sfp, tfp), cadd(sgp, tgp)
        size = max(abs(x) for t in (tf, tg, tfp, tgp) for x in t)
        if k > 5 and size < TINY:
            break
    ai = cadd(cscale(sf, C1), cscale(sg, -C2))
    bi = cscale(cadd(cscale(sf, C1), cscale(sg, C2)), SQRT3)
    aip = cadd(cscale(sfp, C1), cscale(sgp, -C2))
    bip = cscale(cadd(cscale(sfp, C1), cscale(sgp, C2)), SQRT3)
    return ai, bi, aip, bip


def validate():
    refs = Path(__file__).resolve().parents[2] / "tests/dados/F042/referencias.csv"
    n = 0
    for line in refs.read_text().splitlines():
        if not line.startswith("airy_"):
            continue
        name, x, _, _, v = line.split(",")
        if abs(float(x)) > 10:
            continue
        ai, bi, _, _ = airy_all((D(float(x)), D(0)))
        got = float((ai if name == "airy_ai" else bi)[0])
        assert got == float(v) or abs(got - float(v)) <= 1e-16 * abs(float(v)), (name, x, got, v)
        n += 1
    assert n >= 18, n
    for z in [(D("1.5"), D("2.25")), (D("-4"), D("0.75")), (D("0.5"), D("-6")), (D("-7"), D("-7"))]:
        ai, bi, aip, bip = airy_all(z)
        w = cadd(cmul(ai, bip), cscale(cmul(aip, bi), -1))
        # c₁ e c₂ vêm de Γ(1/3), Γ(2/3) com 50 dígitos: o Wronskiano vale a ~1e-50.
        assert abs(w[0] - 1 / PI) < D(10) ** -45 and abs(w[1]) < D(10) ** -45, z
        aic, _, _, _ = airy_all((z[0], -z[1]))
        assert abs(aic[0] - ai[0]) < D(10) ** -90 and abs(aic[1] + ai[1]) < D(10) ** -90, z
        n += 2
    return n


def points():
    pts = []
    for r in [0.5, 0.99, 1.01, 1.5, 3.0, 5.0, 8.0, 10.0]:
        for k in range(-11, 13):
            s, c = sincos(PI * k / 12)
            x, y = float(D(r) * c), float(D(r) * s)
            if k == 12:
                y = 0.0
            if k in (6, -6):
                x = 0.0
            if k == 0:
                y = 0.0
            pts.append((x, y))
    pts += [(-2.338107410459767, 1e-3), (1e-3, -1e-3), (0.0, 1e-9), (-9.0, 0.5), (4.0, -6.9)]
    return pts


def fmt(c):
    return f"{float(c[0])!r},{float(c[1])!r}"


def main():
    n = validate()
    print("# gerado por ferramentas/oraculos/f042_airy_complexo.py (decimal, 120 dígitos; Maclaurin)")
    print(f"# validado por {n} verificações (mpmath no eixo real, Wronskiano, conjugação)")
    print("re,im,ai_re,ai_im,bi_re,bi_im,aip_re,aip_im,bip_re,bip_im")
    for x, y in points():
        ai, bi, aip, bip = airy_all((D(x), D(y)))
        print(f"{x!r},{y!r},{fmt(ai)},{fmt(bi)},{fmt(aip)},{fmt(bip)}")


if __name__ == "__main__":
    sys.exit(main())
