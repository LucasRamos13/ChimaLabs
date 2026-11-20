#!/usr/bin/env python3
"""Coeficientes de Taylor de c₀(η) e c₁(η) da expansão uniforme de Temme para a gama incompleta
(DLMF 8.12.8–8.12.10), derivados em aritmética racional exata (só biblioteca padrão), e valores
de referência de P(a, x) para a grande, por soma direta da série em mpmath (D13), se disponível.

Definições (DLMF 8.12): λ = x/a, μ = λ − 1, η²/2 = μ − ln(1 + μ), sinal(η) = sinal(μ);
  c₀(η) = 1/μ − 1/η,
  c₁(η) = 1/η³ − 1/μ³ − 1/μ² − 1/(12μ).
Perto de η = 0 as formas fechadas cancelam; as séries vêm da reversão μ(η) de
η²/2 = μ²/2 − μ³/3 + μ⁴/4 − …

Validação (falha se não passar): c₀(0) = −1/3 e c₁(0) = −1/540 (valores de DLMF 8.12.10 e
8.12.11); série contra forma fechada em η = ±0,3 a 1e-25 (decimal de 60 dígitos).

Uso: python3 -I ferramentas/oraculos/f042_temme.py   (imprime as constantes em Rust)
     .venv-oraculos/bin/python -I ferramentas/oraculos/f042_temme.py --referencias
"""
import sys
from decimal import Decimal as D, getcontext
from fractions import Fraction as F

getcontext().prec = 60
N = 34  # ordem das séries em η


def mul(a, b):
    out = [F(0)] * N
    for i, x in enumerate(a):
        if x == 0:
            continue
        for j, y in enumerate(b[: N - i]):
            out[i + j] += x * y
    return out


def compose_mu():
    """μ(η) = η + Σ_{k≥2} m_k η^k por iteração de ponto fixo exata: η²/2 = Σ_{j≥2} (−1)^j μ^j/j."""
    # μ = η·sqrt(1 + 2Σ_{j≥3}(−1)^{j+1}... ) — mais simples: resolver coeficiente a coeficiente.
    m = [F(0)] * N
    m[1] = F(1)
    for n in range(2, N):
        # coeficiente de η^{n+1} em Σ_{j≥2} (−1)^j μ^j / j deve ser 0 (lado esquerdo só tem η²/2).
        # O termo linear em m_n vem de j = 2: (2·m_1·m_n)/2 = m_n.
        trial = m[:]
        trial[n] = F(0)
        pw = trial[:]  # μ^1
        total = [F(0)] * N
        for j in range(2, N):
            pw = mul(pw, trial)
            if all(c == 0 for c in pw):
                break
            sign = 1 if j % 2 == 0 else -1
            for k in range(N):
                total[k] += sign * pw[k] / j
        m[n] = -total[n + 1] if n + 1 < N else F(0)
    return m


def inverse_series(s):
    """1/(η·s(η)) − 1/η como série, com s = μ/η (s₀ = 1). Devolve coeficientes de η^k."""
    # 1/s: inverso de série com s[0] = 1.
    inv = [F(0)] * N
    inv[0] = F(1)
    for k in range(1, N):
        inv[k] = -sum(s[i] * inv[k - i] for i in range(1, k + 1))
    # 1/μ − 1/η = (1/η)(inv − 1) = Σ inv[k+1] η^k
    return [inv[k + 1] for k in range(N - 1)]


def series_c(m):
    s = [m[k + 1] for k in range(N - 1)] + [F(0)]  # μ/η
    c0 = inverse_series(s)
    # 1/μ como série de Laurent: (1/η)·inv; potências 1/μ², 1/μ³.
    inv = [F(0)] * N
    inv[0] = F(1)
    for k in range(1, N):
        inv[k] = -sum(s[i] * inv[k - i] for i in range(1, k + 1))
    inv2 = mul(inv, inv)
    inv3 = mul(inv2, inv)
    # c₁ = 1/η³ − inv3/η³ − inv2/η² − inv/(12η)
    #    = Σ_k [ −inv3[k+3] − inv2[k+2] − inv[k+1]/12 ] η^k  (os termos negativos de η se anulam)
    c1 = []
    for k in range(N - 4):
        c1.append(-inv3[k + 3] - inv2[k + 2] - inv[k + 1] / 12)
    # Conferência: os coeficientes de η^{−3}, η^{−2}, η^{−1} somam zero.
    assert 1 - inv3[0] == 0
    assert -inv3[1] - inv2[0] == 0
    assert -inv3[2] - inv2[1] - inv[0] / 12 == 0
    return c0, c1


def dec(f):
    return D(f.numerator) / D(f.denominator)


def closed(eta):
    """μ(η) por Newton em decimal; devolve (c₀, c₁) pelas formas fechadas."""
    mu = eta
    for _ in range(200):
        g = mu - (1 + mu).ln() - eta * eta / 2
        dg = 1 - 1 / (1 + mu)
        step = g / dg
        mu -= step
        if abs(step) < D(10) ** -55:
            break
    c0 = 1 / mu - 1 / eta
    c1 = 1 / eta ** 3 - 1 / mu ** 3 - 1 / mu ** 2 - 1 / (12 * mu)
    return c0, c1


def horner(c, x):
    s = D(0)
    for a in reversed(c):
        s = s * x + dec(a)
    return s


def main():
    m = compose_mu()
    assert m[2] == F(1, 3) and m[3] == F(1, 36), m[:4]
    c0, c1 = series_c(m)
    assert c0[0] == F(-1, 3), c0[0]
    assert c1[0] == F(-1, 540), c1[0]
    for eta in [D("0.3"), D("-0.3"), D("0.05")]:
        a, b = closed(eta)
        assert abs(horner(c0[:28], eta) - a) < D(10) ** -25, ("c0", eta)
        assert abs(horner(c1[:26], eta) - b) < D(10) ** -25, ("c1", eta)
    if "--referencias" in sys.argv:
        import mpmath as mp
        # 80 dígitos: a·ln x chega a ~2e10 (10 dígitos inteiros), e Q = 1 − P nas caudas (até
        # 1e-31) precisa de P com ~45 casas decimais. Com 40 dígitos, Q saía com erro de 2e-3.
        mp.mp.dps = 80

        def p(a, x):
            a, x = mp.mpf(a), mp.mpf(x)
            t, s, k = mp.mpf(1), mp.mpf(1), 1
            while True:
                t *= x / (a + k)
                s += t
                k += 1
                if t < s * mp.mpf(10) ** -75:
                    break
            return mp.exp(a * mp.log(x) - x - mp.loggamma(a + 1)) * s

        print("a,x,p,q")
        for a, x in [(1e5, 1e5), (1e5, 1.01e5), (1e5, 0.99e5), (1e6, 1e6), (1e6, 1.003e6),
                     (1e7, 1e7), (1e7, 1e7 - 1e4), (1e8, 1e8 - 1e4), (1e9, 1e9), (1e9, 1e9 + 1e5),
                     (2.5e5, 2.4e5), (3e6, 3.02e6)]:
            v = p(a, x)
            print(f"{a!r},{x!r},{float(v)!r},{float(1 - v)!r}")
        return
    print("// Gerado por ferramentas/oraculos/f042_temme.py (racionais exatos; conferido contra as")
    print("// formas fechadas em η = ±0,3 e 0,05 a 1e-25).")
    print("/// Taylor de c₀(η) em η = 0 (DLMF 8.12.10); c₀(0) = −1/3.")
    print(f"const TEMME_C0: [f64; 28] = [{', '.join(repr(float(dec(x))) for x in c0[:28])}];")
    print("/// Taylor de c₁(η) em η = 0; c₁(0) = −1/540.")
    print(f"const TEMME_C1: [f64; 26] = [{', '.join(repr(float(dec(x))) for x in c1[:26])}];")


if __name__ == "__main__":
    sys.exit(main())
