#!/usr/bin/env python3
"""Gera referências de J_ν, Y_ν, I_ν e K_ν com argumento complexo para F042 só com a biblioteca
padrão (decimal, 250 dígitos).

Método, escolhido para ser independente do código Rust (que usa recorrência de Miller, integral
de K e relações de rotação): séries ascendentes (DLMF 10.2.2 e 10.25.2) com potência principal
(z/2)^ν, e
  Y_ν = (J_ν cos νπ − J_{−ν}) / sen νπ   (DLMF 10.2.3),
  K_ν = (π/2)(I_{−ν} − I_ν) / sen νπ     (DLMF 10.27.4).
Para ν inteiro usa-se a média dos valores em ν ± 1e-40: Y e K são analíticas em ν, o erro é
O(δ²) = 1e-80 e o cancelamento custa ~40 dígitos dos 250. Com |z| ≤ 30, os termos chegam a e^|z| ≈ 1e13 e K pode
valer e^(−30); sobram mais de 150 dígitos corretos.

Γ em alta precisão: deslocamento até x ≥ 200 e Stirling com 40 números de Bernoulli exatos;
reflexão para argumentos negativos.

Validação (o script falha se não passar):
- no eixo real positivo: J₀(1), Y₀(1) e I₀(1) (valores de tabela), K₀(1) pela integral
  ½∫e^(−cosh t)dt (trapézio, método independente) e J_{1/2}(x) = √(2/(πx)) sen x;
- Wronskianos J_{ν+1}Y_ν − J_νY_{ν+1} = 2/(πz) e I_νK_{ν+1} + I_{ν+1}K_ν = 1/z (DLMF 10.5.2 e
  10.28.2) em pontos complexos.

Uso: python3 -I ferramentas/oraculos/f042_bessel_complexo.py > tests/dados/F042/bessel_complexo.csv
"""
import sys
from decimal import Decimal as D, getcontext
from fractions import Fraction

getcontext().prec = 250
TINY = D(10) ** -245
DELTA = D(10) ** -40


def bernoulli_pares(n):
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
    halvings = 0
    while abs(t) > D("0.1"):
        t = t / (1 + (1 + t * t).sqrt())
        halvings += 1
    s, p, k, t2 = D(0), t, 0, t * t
    while True:
        term = p / (2 * k + 1)
        if abs(term) < TINY:
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


def cmul(a, b):
    return (a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def cdiv(a, b):
    d = b[0] * b[0] + b[1] * b[1]
    return ((a[0] * b[0] + a[1] * b[1]) / d, (a[1] * b[0] - a[0] * b[1]) / d)


def cscale(a, s):
    return (a[0] * s, a[1] * s)


def cadd(a, b):
    return (a[0] + b[0], a[1] + b[1])


def csub(a, b):
    return (a[0] - b[0], a[1] - b[1])


def cexp(z):
    m = z[0].exp()
    s, c = sincos(z[1])
    return (m * c, m * s)


def cln(z):
    return ((z[0] * z[0] + z[1] * z[1]).ln() / 2, atan2(z[1], z[0]))


def lngamma_pos(x):
    acc = D(0)
    while x < 200:
        acc += x.ln()
        x += 1
    r = (x - D("0.5")) * x.ln() - x + (2 * PI).ln() / 2
    p = 1 / x
    for k, b in enumerate(BERN, start=1):
        r += D(b.numerator) / D(b.denominator) / (2 * k * (2 * k - 1)) * p
        p /= x * x
    return r - acc


def rgamma(x):
    """1/Γ(x) para x real (0 nos polos)."""
    if x > 0:
        return (-lngamma_pos(x)).exp()
    if x == x.to_integral_value():
        return D(0)
    # 1/Γ(x) = sen(πx)Γ(1 − x)/π
    s, _ = sincos(PI * x)
    return s * lngamma_pos(1 - x).exp() / PI


def series(nu, z, sign):
    """Σ (sign·z²/4)^k / (k! Γ(ν+k+1)) · (z/2)^ν: J (sign = −1) ou I (sign = +1)."""
    half = cscale(z, D("0.5"))
    q = cscale(cmul(half, half), sign)
    rg = rgamma(nu + 1)
    term = (rg, D(0))
    s = term
    k = 0
    # Coeficientes por recorrência: 1/Γ(ν+k+1) = 1/Γ(ν+k) / (ν+k); com 1/Γ = 0 num polo,
    # recomeça pelo valor direto.
    while True:
        k += 1
        if rg == 0 or nu + k <= 0 and (nu + k) == (nu + k).to_integral_value():
            rgk = rgamma(nu + k + 1)
            fact = D(1)
            for j in range(1, k + 1):
                fact *= j
            pq = (D(1), D(0))
            for _ in range(k):
                pq = cmul(pq, q)
            term = cscale(pq, rgk / fact)
            rg = rgk
        else:
            term = cscale(cmul(term, q), 1 / (k * (nu + k)))
        s = cadd(s, term)
        if k > 10 and abs(term[0]) + abs(term[1]) < TINY * (abs(s[0]) + abs(s[1]) + TINY):
            break
        if k > 3000:
            raise RuntimeError("série não convergiu")
    if z == (0, 0):
        pw = (D(1), D(0)) if nu == 0 else (D(0), D(0))
    else:
        lh = cln(half)
        pw = cexp(cscale(lh, nu))
    return cmul(pw, s)


def y_and_k(nn, z):
    s, c = sincos(PI * nn)
    y = cscale(csub(cscale(series(nn, z, -1), c), series(-nn, z, -1)), 1 / s)
    k = cscale(csub(series(-nn, z, 1), series(nn, z, 1)), PI / 2 / s)
    return y, k


def all_funcs(nu, z):
    nu = D(nu)
    j = series(nu, z, -1)
    i = series(nu, z, 1)
    if nu == nu.to_integral_value():
        # média em ν ± δ: o erro O(δ) se cancela e fica O(δ²) = 1e-80
        ya, ka = y_and_k(nu + DELTA, z)
        yb, kb = y_and_k(nu - DELTA, z)
        y = cscale(cadd(ya, yb), D("0.5"))
        k = cscale(cadd(ka, kb), D("0.5"))
    else:
        y, k = y_and_k(nu, z)
    return j, y, i, k


def k0_integral():
    """K₀(1) = ½∫e^(−cosh t)dt na reta toda (DLMF 10.32.9), pela regra do trapézio com passo
    0,04: para integrando analítico numa faixa, o erro é ~e^(−π²/h) < 1e-200."""
    h, s, k = D("0.04"), D(0), 0
    while True:
        e = (h * k).exp()
        v = (-(e + 1 / e) / 2).exp()
        s += v if k == 0 else 2 * v
        if v < D(10) ** -120:
            return str(s * h / 2)
        k += 1


def close(a, b, rel):
    return abs(a[0] - b[0]) + abs(a[1] - b[1]) <= rel * (abs(b[0]) + abs(b[1]))


def validate():
    one = (D(1), D(0))
    j, y, i, k = all_funcs(0, one)
    for got, want in [(j, "0.7651976865579665514497175261026632209092742897553252418615475491"),
                      (y, "0.08825696421567695798292676602351516282781752309067554671104384761"),
                      (i, "1.266065877752008335598244625214717537607670311354962206808135331"),
                      (k, k0_integral())]:
        assert abs(got[0] - D(want)) < D(10) ** -60 and abs(got[1]) < D(10) ** -60, (got, want)
    for x in ["0.5", "3", "17.25"]:
        z = (D(x), D(0))
        j, _, _, _ = all_funcs(D("0.5"), z)
        s, _ = sincos(D(x))
        want = (2 / (PI * D(x))).sqrt() * s
        assert abs(j[0] - want) < D(10) ** -100, x
    n = 4 + 3
    for nu, z in [(0, (D("1.5"), D("2.25"))), (D("2.3"), (D("-4"), D("0.75"))),
                  (D("-0.7"), (D("0.5"), D("-6"))), (3, (D("-12"), D("9")))]:
        j0, y0, i0, k0 = all_funcs(nu, z)
        j1, y1, i1, k1 = all_funcs(D(nu) + 1, z)
        wr = csub(cmul(j1, y0), cmul(j0, y1))
        assert close(wr, cdiv((2 / PI, D(0)), z), D(10) ** -70), (nu, z)  # O(δ²) = 1e-80
        wk = cadd(cmul(i0, k1), cmul(i1, k0))
        assert close(wk, cdiv((D(1), D(0)), z), D(10) ** -70), (nu, z)
        n += 2
    return n


def points():
    orders = [0.0, 0.5, 1.0, 2.0, 2.3, 5.0, 12.75, -0.7, -3.0, -2.5]
    radii = [0.1, 0.9, 1.5, 3.0, 7.0, 15.0, 25.0]
    # ângulos em múltiplos de π/12, inclusive os eixos e o corte (π)
    angles = [k / 12 for k in range(-11, 13)]
    pts = []
    for nu in orders:
        for r in radii:
            for a in angles:
                if (abs(a) * 12) % 2 == 1 and r not in (0.9, 7.0):
                    continue  # ângulos ímpares só em dois raios, para conter o tamanho
                pts.append((nu, r, a))
    return pts


def fmt(c):
    return f"{float(c[0])!r},{float(c[1])!r}"


def main():
    n = validate()
    print("# gerado por ferramentas/oraculos/f042_bessel_complexo.py (decimal, 250 dígitos; séries)")
    print(f"# validado por {n} verificações (valores clássicos, J_(1/2), Wronskianos)")
    print("nu,re,im,j_re,j_im,y_re,y_im,i_re,i_im,k_re,k_im")
    for nu, r, a in points():
        # z = r·e^(iπa), arredondado para f64 e usado com o valor binário exato.
        s, c = sincos(PI * D(a))
        x, y = float(D(r) * c), float(D(r) * s)
        if a == 1.0:
            y = 0.0  # sobre o corte, lado de cima
        if a == 0.0 or a == 0.5 or a == -0.5:
            x = 0.0 if a != 0.0 else x
            y = y if a != 0.0 else 0.0
        z = (D(x), D(y))
        j, yy, i, k = all_funcs(nu, z)
        print(f"{nu!r},{x!r},{y!r},{fmt(j)},{fmt(yy)},{fmt(i)},{fmt(k)}")


if __name__ == "__main__":
    sys.exit(main())
