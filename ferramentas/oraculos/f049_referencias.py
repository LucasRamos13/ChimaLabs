#!/usr/bin/env python3
"""Soluções de referência para F049, a 25 dígitos (mpmath.odefun: série de Taylor de grau
variável com controle de erro em alta precisão; só em desenvolvimento, LICENCAS D13).

Problemas não lineares sem solução fechada. As funções têm o mesmo nome no teste Rust
(`tests/ode_f049.rs`).

Uso: .venv-oraculos/bin/python -I ferramentas/oraculos/f049_referencias.py > tests/dados/F049/referencias.csv
"""
import mpmath as mp

mp.mp.dps = 30

PROBLEMAS = [
    # nome, f, y0, instantes
    ("vanderpol1", lambda t, y: [y[1], (1 - y[0] ** 2) * y[1] - y[0]], [2, 0], [1, 2, 5, 10]),
    ("lorenz", lambda t, y: [10 * (y[1] - y[0]), y[0] * (28 - y[2]) - y[1],
                             y[0] * y[1] - mp.mpf(8) / 3 * y[2]], [1, 1, 1], [0.5, 1, 2]),
    ("pendulo", lambda t, y: [y[1], -mp.sin(y[0])], [mp.mpf(3), 0], [5, 10, 20]),
    ("brusselator", lambda t, y: [1 + y[0] ** 2 * y[1] - 4 * y[0], 3 * y[0] - y[0] ** 2 * y[1]],
     [mp.mpf("1.5"), 3], [5, 10, 20]),
]


def main():
    print("nome,t,componente,valor")
    for nome, f, y0, ts in PROBLEMAS:
        sol = mp.odefun(f, 0, [mp.mpf(v) for v in y0])
        for t in ts:
            y = sol(mp.mpf(t))
            for i, v in enumerate(y):
                print(f"{nome},{t},{i},{mp.nstr(v, 25, min_fixed=-mp.inf, max_fixed=mp.inf)}")


if __name__ == "__main__":
    main()
