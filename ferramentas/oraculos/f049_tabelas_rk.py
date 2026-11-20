#!/usr/bin/env python3
"""F049 — confere as tabelas de Runge–Kutta pelas condições de ordem, em aritmética racional exata.

Para cada árvore enraizada t com |t| ≤ p, a solução numérica concorda com a exata até a ordem p se
Φ(t) = Σ bᵢ Φᵢ(t) = 1/γ(t) (J. C. Butcher, *Numerical Methods for Ordinary Differential
Equations*, 3.ª ed., 2016, cap. 3). Para a saída densa de ordem q, com pesos bᵢ(θ), a condição é
Φ_θ(t) = θ^|t|/γ(t) para todo θ; como bᵢ(θ) é polinomial em θ de grau ≤ q + 1, basta conferir em
q + 2 valores distintos de θ.

As tabelas são transcritas das publicações:
- Bogacki & Shampine, Appl. Math. Lett. 2 (1989) 321–325 (BS 3(2));
- Dormand & Prince, J. Comput. Appl. Math. 6 (1980) 19–26 (DOPRI 5(4)), com a extensão contínua
  de ordem 4 de Hairer, Nørsett & Wanner, *Solving ODE I*, 2.ª ed. (1993), §II.6.

Uso: python3 -I ferramentas/oraculos/f049_tabelas_rk.py — termina com código 1 se alguma condição
falhar. Só usa a biblioteca padrão.
"""

import sys
from fractions import Fraction as F
from functools import lru_cache


def arvores(n):
    """Árvores enraizadas com n vértices, como tuplas ordenadas de subárvores."""
    return _arvores(n)


@lru_cache(maxsize=None)
def _arvores(n):
    if n == 1:
        return ((),)
    res = set()
    for filhos in _florestas(n - 1, n - 1):
        res.add(tuple(sorted(filhos)))
    return tuple(sorted(res))


@lru_cache(maxsize=None)
def _florestas(n, maximo):
    """Multiconjuntos de árvores com n vértices no total, cada uma com ≤ maximo vértices."""
    if n == 0:
        return ((),)
    res = []
    for k in range(min(n, maximo), 0, -1):
        for t in _arvores(k):
            for resto in _florestas(n - k, k):
                # Ordem canônica: evita repetir o mesmo multiconjunto.
                if resto and (len(resto[0]) and _tam(resto[0]) == k and resto[0] > t):
                    continue
                res.append((t,) + resto)
    return tuple(res)


@lru_cache(maxsize=None)
def _tam(t):
    return 1 + sum(_tam(c) for c in t)


@lru_cache(maxsize=None)
def gamma(t):
    g = _tam(t)
    for c in t:
        g *= gamma(c)
    return g


def phi_estagios(t, a, c):
    """Φᵢ(t) para cada estágio i: produto sobre os filhos de Σⱼ aᵢⱼ Φⱼ(filho)."""
    s = len(c)
    v = [F(1)] * s
    for filho in t:
        pf = phi_estagios(filho, a, c)
        v = [v[i] * sum(a[i][j] * pf[j] for j in range(s)) for i in range(s)]
    return v


def condicoes(b, a, c, p, theta=F(1)):
    """Lista das árvores (até ordem p) que falham."""
    falhas = []
    for n in range(1, p + 1):
        vistos = set()
        for t in arvores(n):
            if t in vistos:
                continue
            vistos.add(t)
            phi = sum(b[i] * x for i, x in enumerate(phi_estagios(t, a, c)))
            if phi != theta**n / gamma(t):
                falhas.append((n, t))
    return falhas


def numero_de_arvores(n):
    return len(set(arvores(n)))


def bs23():
    c = [F(0), F(1, 2), F(3, 4), F(1)]
    a = [[F(0)] * 4 for _ in range(4)]
    a[1][0] = F(1, 2)
    a[2][1] = F(3, 4)
    a[3][:3] = [F(2, 9), F(1, 3), F(4, 9)]
    b = [F(2, 9), F(1, 3), F(4, 9), F(0)]
    bh = [F(7, 24), F(1, 4), F(1, 3), F(1, 8)]
    return c, a, b, bh


def dopri5():
    c = [F(0), F(1, 5), F(3, 10), F(4, 5), F(8, 9), F(1), F(1)]
    a = [[F(0)] * 7 for _ in range(7)]
    a[1][0] = F(1, 5)
    a[2][:2] = [F(3, 40), F(9, 40)]
    a[3][:3] = [F(44, 45), F(-56, 15), F(32, 9)]
    a[4][:4] = [F(19372, 6561), F(-25360, 2187), F(64448, 6561), F(-212, 729)]
    a[5][:5] = [F(9017, 3168), F(-355, 33), F(46732, 5247), F(49, 176), F(-5103, 18656)]
    a[6][:6] = [F(35, 384), F(0), F(500, 1113), F(125, 192), F(-2187, 6784), F(11, 84)]
    b = a[6][:6] + [F(0)]
    bh = [F(5179, 57600), F(0), F(7571, 16695), F(393, 640), F(-92097, 339200), F(187, 2100),
          F(1, 40)]
    return c, a, b, bh


DOPRI5_D = [F(-12715105075, 11282082432), F(0), F(87487479700, 32700410799),
            F(-10690763975, 1880347072), F(701980252875, 199316789632),
            F(-1453857185, 822651844), F(69997945, 29380423)]


def pesos_densos(b, d, theta):
    """bᵢ(θ) da forma y(θ) = r1 + θ(r2 + (1−θ)(r3 + θ(r4 + (1−θ) r5))), com
    r2 = Σ bᵢkᵢ, r3 = k₁ − r2, r4 = r2 − k_s − r3 (k_s = último estágio, FSAL) e r5 = Σ dᵢkᵢ
    (os k já multiplicados por h)."""
    s = len(b)
    e1 = [F(1) if i == 0 else F(0) for i in range(s)]
    es = [F(1) if i == s - 1 else F(0) for i in range(s)]
    r2 = b
    r3 = [e1[i] - r2[i] for i in range(s)]
    r4 = [r2[i] - es[i] - r3[i] for i in range(s)]
    r5 = d
    th = theta
    return [th * (r2[i] + (1 - th) * (r3[i] + th * (r4[i] + (1 - th) * r5[i]))) for i in range(s)]


def main():
    ok = True
    # Contagem de árvores (OEIS A000081): confere o gerador.
    esperado = [1, 1, 2, 4, 9, 20, 48, 115, 286]
    obtido = [numero_de_arvores(n) for n in range(1, 10)]
    print("árvores por ordem:", obtido)
    ok &= obtido == esperado

    for nome, (c, a, b, bh), p, ph in [("BS23", bs23(), 3, 2), ("DOPRI5", dopri5(), 5, 4)]:
        f1 = condicoes(b, a, c, p)
        f2 = condicoes(bh, a, c, ph)
        f3 = condicoes(b, a, c, p + 1)
        cs = all(sum(a[i]) == c[i] for i in range(len(c)))
        print(f"{nome}: ordem {p} {'ok' if not f1 else f1}; embutido {ph} "
              f"{'ok' if not f2 else f2}; c = soma das linhas {cs}; "
              f"falha na ordem {p + 1} (esperado) {bool(f3)}")
        ok &= not f1 and not f2 and cs and bool(f3)

    # Saída densa: Hermite cúbica (BS23, ordem 3) e extensão de ordem 4 (DOPRI5).
    for nome, (c, a, b, _), d, q in [("BS23", bs23(), [F(0)] * 4, 3),
                                    ("DOPRI5", dopri5(), DOPRI5_D, 4)]:
        thetas = [F(k, q + 3) for k in range(1, q + 3)]
        falhas = [(th, condicoes(pesos_densos(b, d, th), a, c, q, th)) for th in thetas]
        falhas = [x for x in falhas if x[1]]
        # Em θ = 1 a saída densa reproduz o passo.
        fim = pesos_densos(b, d, F(1)) == b
        print(f"{nome} denso: ordem {q} {'ok' if not falhas else falhas}; θ=1 reproduz o passo {fim}")
        ok &= not falhas and fim
    print("TUDO OK" if ok else "FALHOU")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
