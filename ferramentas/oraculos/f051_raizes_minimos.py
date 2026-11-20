#!/usr/bin/env python3
"""Raízes e mínimos de referência para F051, a 40 dígitos (mpmath, só em desenvolvimento; LICENCAS D13).

Raízes: bissecção em 60 dígitos até largura relativa 1e-45, robusta para raízes múltiplas e
funções íngremes. Mínimos: seção áurea em 60 dígitos até 1e-28·max(1, |x|) (as funções são
unimodais no intervalo), depois comparação com os extremos.

As funções têm o mesmo nome no teste Rust (`tests/optim_f051.rs`), que as implementa em f64.

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f051_raizes_minimos.py > tests/dados/F051/referencias.csv
"""
import mpmath as mp

mp.mp.dps = 60

RAIZES = [
    # nome, f, a, b
    ("wallis", lambda x: x**3 - 2 * x - 5, 2, 3),
    ("cos_x", lambda x: mp.cos(x) - x, 0, 1),
    ("exp_2", lambda x: mp.exp(x) - 2, 0, 2),
    ("raiz2", lambda x: x * x - 2, 0, 2),
    ("cubo_multiplo", lambda x: (x - 1) ** 3, 0, 3),
    ("raiz_cubica", lambda x: mp.sign(x - 1) * mp.cbrt(abs(x - 1)), 0, 3),
    ("seno", lambda x: mp.sin(x) - x / 2, 1, 3),
    ("tan_x", lambda x: mp.tan(x) - x, 4, 4.6),
    ("kepler", lambda x: x - mp.mpf("0.9") * mp.sin(x) - mp.mpf("0.1"), 0, 2),
    ("exp_enorme", lambda x: mp.exp(x) - mp.mpf("1e300"), 600, 700),
    ("minusculo", lambda x: x - mp.mpf("1e-300"), -1, 1),
    ("atan_ingreme", lambda x: mp.atan(mp.mpf("1e6") * (x - mp.mpf("0.3"))), 0, 1),
    ("potencia9", lambda x: (x - mp.mpf("0.5")) ** 9, 0, 2),
    ("xexp", lambda x: x * mp.exp(-x) - mp.mpf("1e-8"), 0, 1),
    ("log_dominio", lambda x: mp.log(x) - 1, 1, 10),
]

MINIMOS = [
    ("quadratica", lambda x: (x - 2) ** 2, 0, 5),
    ("quartica", lambda x: x**4 - x, 0, 2),
    ("seno_neg", lambda x: -mp.sin(x), 0, mp.pi),
    ("xlogx", lambda x: x * mp.log(x), mp.mpf("1e-10"), 1),
    ("cosseno", lambda x: mp.cos(x), 3, 4),
    ("monotona", lambda x: mp.exp(x), -1, 2),
    ("monotona_dir", lambda x: -x, 0, 3),
    ("abs_desloc", lambda x: abs(x - mp.mpf("0.7")), 0, 1),
    ("escala_grande", lambda x: (x - mp.mpf("1e6")) ** 2 + 1, 0, mp.mpf("2e6")),
]


def raiz(f, a, b):
    a, b = mp.mpf(a), mp.mpf(b)
    fa, fb = f(a), f(b)
    assert fa * fb < 0, "sem mudança de sinal"
    # Bissecção em 60 dígitos: robusta para raízes múltiplas e funções íngremes.
    for _ in range(3000):
        m = (a + b) / 2
        fm = f(m)
        if fm == 0:
            return m
        if (fm > 0) == (fa > 0):
            a, fa = m, fm
        else:
            b = m
        if b - a < mp.mpf(10) ** (-45) * max(mp.mpf(10) ** -320, abs(m)):
            break
    return (a + b) / 2


def minimo(f, a, b):
    a, b = mp.mpf(a), mp.mpf(b)
    # Seção áurea em 60 dígitos (exata para funções unimodais), depois os extremos.
    g = (mp.sqrt(5) - 1) / 2
    lo, hi = a, b
    for _ in range(400):
        x1 = hi - g * (hi - lo)
        x2 = lo + g * (hi - lo)
        if f(x1) <= f(x2):
            hi = x2
        else:
            lo = x1
        if hi - lo < mp.mpf(10) ** (-28) * max(1, abs(lo)):
            break
    x = (lo + hi) / 2
    for e in (a, b):
        if f(e) < f(x):
            x = e
    return x


print("tipo,nome,a,b,x,fx")
for nome, f, a, b in RAIZES:
    r = raiz(f, a, b)
    print(f"raiz,{nome},{mp.nstr(mp.mpf(a), 17)},{mp.nstr(mp.mpf(b), 17)},{mp.nstr(r, 40)},0")
for nome, f, a, b in MINIMOS:
    x = minimo(f, a, b)
    print(f"minimo,{nome},{mp.nstr(mp.mpf(a), 20)},{mp.nstr(mp.mpf(b), 20)},{mp.nstr(x, 40)},{mp.nstr(f(x), 40)}")
