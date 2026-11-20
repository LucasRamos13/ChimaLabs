#!/usr/bin/env python3
"""Gera valores de referência para F042 com mpmath (BSD-3-Clause) a 50 dígitos.

Só para testes: mpmath NÃO é dependência do produto (LICENCAS_E_RESTRICOES.txt D13).
Uso: <python-com-mpmath> -I ferramentas/oraculos/f042_referencias.py > tests/dados/F042/referencias.csv
Cada linha: funcao,arg1,arg2,arg3,valor  (argumentos e valor em repr de float64; o valor é o
float64 mais próximo do resultado em alta precisão).
"""
import mpmath as mp

mp.mp.dps = 50
rows = []


def out(f, args, v):
    a = [repr(float(x)) for x in args] + [""] * (3 - len(args))
    rows.append(f"{f},{a[0]},{a[1]},{a[2]},{repr(float(v))}")


for x in [-4.5, -1.3, -0.5, 1e-3, 0.3, 0.5, 1.0, 1.5, 2.0, 2.5, 3.7, 7.2, 14.9, 15.0, 20.5, 50.3, 100.1, 170.5, 23.0, 30.0]:
    out("gamma", [x], mp.gamma(mp.mpf(x)))
for x in [1e-5, 0.3, 0.8, 0.9, 1.0, 1.1, 1.25, 1.9, 2.0, 2.1, 3.0, 10.0, 100.0, 1e5, 1e10, -0.5, -2.5]:
    out("gammaln", [x], mp.log(abs(mp.gamma(mp.mpf(x)))))
for x in [0.1, 0.5, 1.0, 2.0, 5.5, 10.0, 100.0, -0.5, -2.5, 1e-8]:
    out("psi", [x], mp.digamma(mp.mpf(x)))
for a, b in [(0.5, 0.5), (2.0, 3.0), (1e-3, 2.0), (10.5, 20.5), (100.0, 100.0), (200.0, 300.0)]:
    out("beta", [a, b], mp.beta(a, b))
    out("betaln", [a, b], mp.log(mp.beta(a, b)))
for a, x in [(0.5, 0.1), (1.0, 1.0), (2.5, 2.0), (5.0, 10.0), (10.0, 3.0), (10.0, 30.0), (50.0, 45.0), (100.0, 100.0), (0.1, 5.0)]:
    out("gammainc_lower", [x, a], mp.gammainc(a, 0, x, regularized=True))
    out("gammainc_upper", [x, a], mp.gammainc(a, x, mp.inf, regularized=True))
for x, a, b in [(0.1, 2.0, 3.0), (0.5, 0.5, 0.5), (0.9, 10.0, 2.0), (0.3, 50.0, 60.0), (0.999, 2.0, 2.0), (0.001, 0.5, 5.0)]:
    out("betainc_lower", [x, a, b], mp.betainc(a, b, 0, x, regularized=True))
    out("betainc_upper", [x, a, b], mp.betainc(a, b, x, 1, regularized=True))
for x in [-3.0, -1.0, -0.5, 1e-10, 0.1, 0.5, 1.0, 1.3, 1.31, 2.0, 3.0, 5.0, 10.0, 26.0]:
    out("erf", [x], mp.erf(x))
    out("erfc", [x], mp.erfc(x))
for x in [-3.0, -0.5, 0.0, 0.5, 1.3, 1.31, 2.0, 10.0, 50.0, 1e3]:
    out("erfcx", [x], mp.exp(mp.mpf(x) ** 2) * mp.erfc(x))
for y in [-0.999999, -0.5, 1e-10, 0.3, 0.5, 0.9, 0.999, 0.999999999]:
    out("erfinv", [y], mp.erfinv(y))
for z in [1e-300, 1e-100, 1e-20, 1e-5, 0.1, 0.5, 1.5, 1.9999]:
    with mp.workdps(400):  # 1 − z precisa de precisão suficiente para z minúsculo
        out("erfcinv", [z], mp.erfinv(1 - mp.mpf(z)))
for x in [1e-10, 0.1, 0.5, 1.0, 1.5, 5.0, 20.0, 100.0, 700.0]:
    out("expint", [x], mp.e1(x))
for x in [-50.0, -1.0, -0.1, 0.1, 1.0, 5.0, 39.0, 41.0, 100.0, 700.0]:
    out("ei", [x], mp.ei(x))
for n, x in [(5, 0.3), (20, -0.7), (3, 0.999)]:  # x = 1 é singular na fórmula do mpmath
    for m in range(n + 1):
        out("legendre", [n, m, x], mp.legenp(n, m, x, type=2))
for m in [-2.0, 0.0, 0.1, 0.5, 0.9, 0.999999]:
    out("ellipk", [m], mp.ellipk(m))
    out("ellipe", [m], mp.ellipe(m))
for u, m in [(0.5, 0.3), (2.0, 0.9), (10.0, 0.5), (1.2, 0.999), (50.0, 0.5), (100.0, 0.3), (14.333134940830401, 0.09419168035869603), (-37.5, 0.95), (3.0, 0.99)]:
    for name in ("sn", "cn", "dn"):
        out("ellipj_" + name, [u, m], mp.ellipfun(name, u, m=m))

for nu in [0.0, 0.5, 1.0, 1.5, 2.3, 5.0, 10.7, 20.0, -0.5, -2.3, -3.0]:
    for x in [1e-3, 0.1, 1.0, 2.5, 5.0, 10.0, 18.0, 30.0, 50.0, 100.0, 1000.0]:
        out("besselj", [nu, x], mp.besselj(nu, x))
        out("bessely", [nu, x], mp.bessely(nu, x))
        out("besseli_scaled", [nu, x], mp.besseli(nu, x) * mp.exp(-x))
        out("besselk_scaled", [nu, x], mp.besselk(nu, x) * mp.exp(x))
for nu in [0.0, 0.5, 1.0, 2.3, 10.7, -2.3, -3.0]:
    for x in [1e-3, 0.1, 1.0, 5.0, 30.0, 100.0, 1000.0]:
        for k, h in ((1, mp.hankel1(nu, x) * mp.exp(-1j * x)), (2, mp.hankel2(nu, x) * mp.exp(1j * x))):
            out(f"hankel{k}_scaled_re", [nu, x], mp.re(h))
            out(f"hankel{k}_scaled_im", [nu, x], mp.im(h))
for x in [-30.0, -10.0, -3.7, -1.0, -0.5, 0.0, 0.3, 1.0, 1.5, 4.0, 10.0, 25.0]:
    out("airy_ai", [x], mp.airyai(x))
    out("airy_bi", [x], mp.airybi(x))

print("# gerado por ferramentas/oraculos/f042_referencias.py com mpmath", mp.__version__)
print("funcao,arg1,arg2,arg3,valor")
print("\n".join(rows))
