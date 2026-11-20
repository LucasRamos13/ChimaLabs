#!/usr/bin/env python3
"""Referências de beta, betaln e gammainc nos regimes difíceis (argumentos grandes, transição
a ≈ x), com mpmath (BSD-3-Clause; LICENCAS D13) a 60 dígitos. Também imprime as constantes de
dupla-dupla usadas em special/dd.rs, para conferência.

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f042_beta_gammainc.py \
         > tests/dados/F042/beta_gammainc.csv
     .venv-oraculos/bin/python -I ferramentas/oraculos/f042_beta_gammainc.py --gammaln \
         > tests/dados/F042/gammaln_ramos.csv
"""
import sys

import mpmath as mp

mp.mp.dps = 60


def dd(x):
    hi = float(x)
    lo = float(x - mp.mpf(hi))
    return hi, lo


def main():
    print("# gerado por ferramentas/oraculos/f042_beta_gammainc.py (mpmath, 60 dígitos)")
    print(f"# ln2 = {dd(mp.log(2))!r}; ln(√(2π)) = {dd(mp.log(mp.sqrt(2 * mp.pi)))!r}")
    print("funcao,a,b,valor")
    pares = [
        (0.5, 1e6), (0.5, 2e3), (2.5, 1e4), (10.0, 1e6), (14.9, 300.0), (15.0, 155.0),
        (100.0, 100.0), (200.0, 300.0), (170.5, 0.25), (1e3, 3e3), (1e4, 2e4), (5e4, 7.5),
        (37.25, 1e5), (1e5, 1e5 + 1), (123.456, 654.321), (3e2, 1e-3), (1e-3, 400.0),
        (160.0, 20.0), (85.0, 85.0), (1.0, 169.0),
    ]
    for a, b in pares:
        a_, b_ = mp.mpf(a), mp.mpf(b)
        lb = mp.log(mp.beta(a_, b_))
        print(f"betaln,{a!r},{b!r},{float(lb)!r}")
        v = mp.beta(a_, b_)
        if 1e-300 < v < 1e300:
            print(f"beta,{a!r},{b!r},{float(v)!r}")
    casos = [
        (100.0, 100.0), (100.0, 95.0), (100.0, 105.0), (1e3, 1e3), (1e4, 1e4 + 50), (1e4, 9.9e3),
        (500.0, 520.0), (50.0, 45.0), (20.0, 19.0), (15.0, 15.0), (16.0, 30.0), (1e5, 1e5),
        (30.0, 60.0), (200.0, 120.0), (1e6, 1e6 - 1e3), (40.0, 40.5),
    ]
    for a, x in casos:
        a_, x_ = mp.mpf(a), mp.mpf(x)
        p = mp.gammainc(a_, 0, x_, regularized=True)
        q = mp.gammainc(a_, x_, mp.inf, regularized=True)
        print(f"gammainc_lower,{x!r},{a!r},{float(p)!r}")
        print(f"gammainc_upper,{x!r},{a!r},{float(q)!r}")




def gammaln_refs():
    """ln|Γ(x)| em pontos que exercitam cada ramo de `gammaln` (raízes 1 e 2, x pequeno,
    fronteiras 15 e 1e300, negativos), para erro relativo puro."""
    xs = [1 + 1e-8, 1 - 3e-9, 1.1, 0.9, 0.76, 1.24, 1.75, 2 + 1e-10, 2 - 2e-7, 2.24, 1.76,
          0.3, 0.49, 1e-5, 1e-300, 5e-324, 14.99, 15.0, 15.01, 30.5, 1e10, 1e300, 2e300,
          -0.5, -2.3, -7.75, -1e-7]
    print("x,lngamma")
    for x in xs:
        v = mp.log(abs(mp.gamma(mp.mpf(x)))) if x < 1e300 else mp.loggamma(mp.mpf(x)).real
        print(f"{x!r},{float(v)!r}")


if __name__ == "__main__":
    if "--gammaln" in sys.argv:
        sys.exit(gammaln_refs())
    sys.exit(main())
