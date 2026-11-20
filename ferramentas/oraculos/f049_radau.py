#!/usr/bin/env python3
"""F049 — constantes do Radau IIA de 3 estágios (ordem 5), derivadas e conferidas em 50 dígitos
(mpmath, só em desenvolvimento; LICENCAS D13).

Fontes: E. Hairer & G. Wanner, *Solving Ordinary Differential Equations II*, 2.ª ed. (1996),
§IV.5 (Radau IIA: nós são os zeros de d²/dx²[x²(x−1)³]... equivalentemente c = (4∓√6)/10, 1) e
§IV.8 (implementação: transformação de A⁻¹, estimador embutido).

Deriva, a partir só dos nós:
- A pela colocação: aᵢⱼ = ∫₀^{cᵢ} ℓⱼ(s) ds (ℓⱼ: base de Lagrange nos nós c);
- autovalores de A⁻¹: um real γ e um par α ± iβ, e a matriz T com A⁻¹ = T·Λ·T⁻¹, Λ na forma
  real em blocos [[γ, 0, 0], [0, α, −β], [0, β, α]];
- estimador embutido de ordem 3: ŷ = y₀ + h(γ₀ f(x₀, y₀) + Σ b̂ᵢ f(Yᵢ)) com γ₀ = 1/γ; então
  ŷ − y = γ₀ h f₀ + Σ eᵢ zᵢ, com e = A⁻ᵀ(b̂ − b).

Confere: condições de ordem 5 de (A, b) (árvores até ordem 5 e falha na 6) e ordem 3 do embutido
(com o nó extra c₀ = 0). Imprime as constantes em Rust (35 dígitos significativos).

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f049_radau.py
"""
import sys

import mpmath as mp

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from f049_tabelas_rk import arvores, gamma  # noqa: E402

mp.mp.dps = 50


def phi_estagios(t, a, c):
    s = len(c)
    v = [mp.mpf(1)] * s
    for filho in t:
        pf = phi_estagios(filho, a, c)
        v = [v[i] * mp.fsum(a[i][j] * pf[j] for j in range(s)) for i in range(s)]
    return v


def residuo_ordem(b, a, c, p):
    pior = mp.mpf(0)
    for n in range(1, p + 1):
        for t in set(arvores(n)):
            phi = mp.fsum(b[i] * x for i, x in enumerate(phi_estagios(t, a, c)))
            pior = max(pior, abs(phi - mp.mpf(1) / gamma(t)))
    return pior


def lagrange_integral(c, j, x):
    """∫₀ˣ ℓⱼ(s) ds, ℓⱼ o polinômio de Lagrange de grau len(c)−1 nos nós c."""
    # coeficientes de ℓⱼ por expansão do produto
    coef = [mp.mpf(1)]
    den = mp.mpf(1)
    for m, cm in enumerate(c):
        if m == j:
            continue
        coef = [mp.mpf(0)] + coef  # multiplica por s
        for k in range(len(coef) - 1):
            coef[k] -= cm * coef[k + 1]
        den *= c[j] - cm
    return mp.fsum(coef[k] * x ** (k + 1) / (k + 1) for k in range(len(coef))) / den


def main():
    r6 = mp.sqrt(6)
    c = [(4 - r6) / 10, (4 + r6) / 10, mp.mpf(1)]
    a = [[lagrange_integral(c, j, c[i]) for j in range(3)] for i in range(3)]
    b = a[2][:]
    # Conferência com as fórmulas fechadas conhecidas (Hairer & Wanner, tabela IV.5.6).
    fechada = [[(88 - 7 * r6) / 360, (296 - 169 * r6) / 1800, (-2 + 3 * r6) / 225],
               [(296 + 169 * r6) / 1800, (88 + 7 * r6) / 360, (-2 - 3 * r6) / 225],
               [(16 - r6) / 36, (16 + r6) / 36, mp.mpf(1) / 9]]
    dif_fechada = max(abs(a[i][j] - fechada[i][j]) for i in range(3) for j in range(3))
    r5 = residuo_ordem(b, a, c, 5)
    r6_ = residuo_ordem(b, a, c, 6)
    am = mp.matrix(a)
    ainv = am ** -1
    ev, er = mp.eig(ainv)
    reais = [k for k in range(3) if abs(mp.im(ev[k])) < mp.mpf(10) ** -40]
    k0 = reais[0]
    kc = [k for k in range(3) if k != k0 and mp.im(ev[k]) > 0][0]
    gam = mp.re(ev[k0])
    alpha = mp.re(ev[kc])
    beta = mp.im(ev[kc])
    v0 = [mp.re(er[i, k0]) for i in range(3)]
    vc = [er[i, kc] for i in range(3)]
    # T = [v0, Re vc, Im vc]: A⁻¹·(u + iw) = (α + iβ)(u + iw) ⇒ A⁻¹u = αu − βw, A⁻¹w = βu + αw.
    # Na base [v0, w, u]... escolhemos colunas [v0, Re, Im] e Λ = [[γ,0,0],[0,α,β],[0,−β,α]].
    t = mp.matrix(3, 3)
    for i in range(3):
        t[i, 0] = v0[i]
        t[i, 1] = mp.re(vc[i])
        t[i, 2] = mp.im(vc[i])
    lam = mp.matrix([[gam, 0, 0], [0, alpha, beta], [0, -beta, alpha]])
    tinv = t ** -1
    dif_decomp = mp.mnorm(t * lam * tinv - ainv, 1)
    # Estimador embutido de ordem 3 com nó extra c₀ = 0 e peso γ₀ = 1/γ.
    g0 = 1 / gam
    m = mp.matrix([[1, 1, 1], [c[0], c[1], c[2]], [c[0] ** 2, c[1] ** 2, c[2] ** 2]])
    rhs = mp.matrix([1 - g0, mp.mpf(1) / 2, mp.mpf(1) / 3])
    bh = mp.lu_solve(m, rhs)
    e = (am.T) ** -1 * mp.matrix([bh[i] - b[i] for i in range(3)])
    # Conferência do embutido: condições de ordem 3 com c₀ = 0.
    cc = [mp.mpf(0)] + c
    bb = [g0] + [bh[i] for i in range(3)]
    ord3 = max(abs(mp.fsum(bb[i] * cc[i] ** k for i in range(4)) - mp.mpf(1) / (k + 1)) for k in range(3))
    print(f"// A pela colocação vs fórmula fechada: {mp.nstr(dif_fechada, 3)}")
    print(f"// resíduo das condições de ordem ≤ 5: {mp.nstr(r5, 3)}; ordem 6 falha com {mp.nstr(r6_, 3)}")
    print(f"// ‖T Λ T⁻¹ − A⁻¹‖₁ = {mp.nstr(dif_decomp, 3)}; embutido ordem 3: {mp.nstr(ord3, 3)}")
    ok = dif_fechada < 1e-45 and r5 < 1e-45 and r6_ > 1e-6 and dif_decomp < 1e-40 and ord3 < 1e-45
    f = lambda v: mp.nstr(v, 35, min_fixed=-mp.inf, max_fixed=mp.inf)  # noqa: E731
    print(f"const GAMMA: f64 = {f(gam)};")
    print(f"const ALPHA: f64 = {f(alpha)};")
    print(f"const BETA: f64 = {f(beta)};")
    for nome, mat in [("T", t), ("TINV", tinv)]:
        linhas = ", ".join("[" + ", ".join(f(mat[i, j]) for j in range(3)) + "]" for i in range(3))
        print(f"const {nome}: [[f64; 3]; 3] = [{linhas}];")
    print("const C: [f64; 3] = [" + ", ".join(f(x) for x in c) + "];")
    print("const E: [f64; 3] = [" + ", ".join(f(e[i]) for i in range(3)) + "];")
    print("OK" if ok else "FALHOU")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
