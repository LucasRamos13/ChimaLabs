#!/usr/bin/env python3
"""Gera referências de Ai, Bi, Ai′ e Bi′ para F042 só com a biblioteca padrão (decimal).

Método: séries de Maclaurin (DLMF 9.4.1–2) em aritmética decimal de 100 dígitos,
Ai = c₁f − c₂g, Bi = √3(c₁f + c₂g), com c₁ = 3^(−2/3)/Γ(2/3), c₂ = 3^(−1/3)/Γ(1/3), e as
derivadas termo a termo. Para |x| ≤ 10 o cancelamento entre c₁f e c₂g é de no máximo ~20
dígitos, e as constantes Γ(1/3), Γ(2/3) (publicadas, 50 dígitos; conferidas pela reflexão
Γ(1/3)Γ(2/3) = 2π/√3) deixam ≥ 25 dígitos corretos, muito além do f64.

Validação: os valores de Ai e Bi reproduzem as referências do mpmath em
tests/dados/F042/referencias.csv (o script falha se não reproduzirem).

Uso: python3 -I ferramentas/oraculos/f042_airy_derivadas.py > tests/dados/F042/airy_derivadas.csv
"""
import sys
from decimal import Decimal as D, getcontext
from pathlib import Path

getcontext().prec = 100

GAMMA_1_3 = D("2.6789385347077476336556929409746776441286893779573")
GAMMA_2_3 = D("1.3541179394264004169452880281545137855193272660568")


def pi():
    """π por Machin: 16·atan(1/5) − 4·atan(1/239)."""
    def atan_inv(n):
        x = D(1) / n
        x2 = x * x
        s, t, k = D(0), x, 0
        while True:
            term = t / (2 * k + 1)
            if term == 0 or abs(term) < D(10) ** -110:
                break
            s += term if k % 2 == 0 else -term
            t *= x2
            k += 1
        return s
    return 16 * atan_inv(5) - 4 * atan_inv(239)


PI = pi()
SQRT3 = D(3).sqrt()
assert abs(GAMMA_1_3 * GAMMA_2_3 - 2 * PI / SQRT3) < D(10) ** -45, "constantes Γ inconsistentes"

C1 = 1 / (D(3) ** (D(2) / 3) * GAMMA_2_3)
C2 = 1 / (D(3) ** (D(1) / 3) * GAMMA_1_3)


def airy_all(x):
    x = D(x)
    x3 = x * x * x
    f, g = D(1), x          # termos de f e g
    fp, gp = D(0), D(1)     # termos de f′ e g′
    sf, sg, sfp, sgp = f, g, fp, gp
    k = 1
    while True:
        f *= x3 / ((3 * k - 1) * (3 * k))
        g *= x3 / ((3 * k) * (3 * k + 1))
        # f′ = Σ 3k·x^(3k−1)·aₖ, g′ = Σ (3k+1)·x^(3k)·bₖ: derivadas dos termos de f e g.
        fp = 3 * k * f / x if x != 0 else D(0)
        gp = (3 * k + 1) * g / x if x != 0 else D(0)
        sf += f
        sg += g
        sfp += fp
        sgp += gp
        if k > 10 and max(abs(f), abs(g), abs(fp), abs(gp)) < D(10) ** -90:
            break
        k += 1
    ai = C1 * sf - C2 * sg
    bi = SQRT3 * (C1 * sf + C2 * sg)
    aip = C1 * sfp - C2 * sgp
    bip = SQRT3 * (C1 * sfp + C2 * sgp)
    return ai, bi, aip, bip


def main():
    # Validação contra as referências do mpmath já versionadas.
    refs = Path(__file__).resolve().parents[2] / "tests/dados/F042/referencias.csv"
    checked = 0
    for line in refs.read_text().splitlines():
        if not line.startswith("airy_"):
            continue
        name, x, _, _, v = line.split(",")
        if abs(float(x)) > 10:
            continue
        ai, bi, _, _ = airy_all(float(x))
        got = float(ai if name == "airy_ai" else bi)
        assert got == float(v) or abs(got - float(v)) <= 1e-16 * abs(float(v)), (name, x, got, v)
        checked += 1
    assert checked >= 18, checked
    print("# gerado por ferramentas/oraculos/f042_airy_derivadas.py (decimal, 100 dígitos)")
    print(f"# validado contra {checked} referências de Ai/Bi do mpmath em referencias.csv")
    print("x,ai_prime,bi_prime")
    xs = [i / 4 for i in range(-40, 41)] + [-0.999, -1.001, 0.999, 1.001, 1e-3, -1e-3, 5.123, -7.77, 9.9]
    for x in sorted(set(xs)):
        _, _, aip, bip = airy_all(x)
        print(f"{x!r},{float(aip)!r},{float(bip)!r}")


if __name__ == "__main__":
    sys.exit(main())
