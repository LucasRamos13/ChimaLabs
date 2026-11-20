#!/usr/bin/env python3
"""Primeiras saídas de MT19937, MRG32k3a e xoshiro256++ para F053, só com a biblioteca padrão.

- MT19937: o estado de `init_genrand(s)` (recorrência do artigo, 624 palavras) é carregado no
  gerador em C do módulo `random` do Python por `setstate`, e as saídas vêm de `getrandbits(32)`.
  A geração e a têmpera são, portanto, do código do Python, independente do nosso.
- MRG32k3a (L'Ecuyer, 1999) e xoshiro256++ (Blackman & Vigna, 2021): aritmética inteira exata a
  partir das definições dos artigos; as sementes são expandidas por SplitMix64 (Steele et al.,
  2014) como no Rust (DT-23).

Uso: python3 -I ferramentas/oraculos/f053_geradores.py > tests/dados/F053/geradores.csv
"""
import random
import sys

M64 = (1 << 64) - 1


def splitmix64(state):
    state = (state + 0x9E3779B97F4A7C15) & M64
    z = state
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M64
    return state, z ^ (z >> 31)


def mt_init(seed):
    mt = [seed & 0xFFFFFFFF]
    for i in range(1, 624):
        mt.append((1812433253 * (mt[-1] ^ (mt[-1] >> 30)) + i) & 0xFFFFFFFF)
    return mt


def mt_outputs(seed, n):
    r = random.Random()
    r.setstate((3, tuple(mt_init(seed) + [624]), None))
    return [r.getrandbits(32) for _ in range(n)]


def mrg_outputs(seed, n):
    m1, m2 = 4294967087, 4294944443
    s = seed
    vals = []
    for m in (m1, m1, m1, m2, m2, m2):
        s, z = splitmix64(s)
        vals.append(1 + z % (m - 1))
    x1, x2 = vals[:3], vals[3:]
    out = []
    for _ in range(n):
        p1 = (1403580 * x1[1] - 810728 * x1[0]) % m1
        x1 = [x1[1], x1[2], p1]
        p2 = (527612 * x2[2] - 1370589 * x2[0]) % m2
        x2 = [x2[1], x2[2], p2]
        out.append((p1 - p2) % m1)
    return out


def rotl(x, k):
    return ((x << k) | (x >> (64 - k))) & M64


def xoshiro_outputs(seed, n):
    st = seed
    s = []
    for _ in range(4):
        st, z = splitmix64(st)
        s.append(z)
    out = []
    for _ in range(n):
        out.append((rotl((s[0] + s[3]) & M64, 23) + s[0]) & M64)
        t = (s[1] << 17) & M64
        s[2] ^= s[0]
        s[3] ^= s[1]
        s[1] ^= s[2]
        s[0] ^= s[3]
        s[2] ^= t
        s[3] = rotl(s[3], 45)
    return out


def main():
    print("# gerado por ferramentas/oraculos/f053_geradores.py (biblioteca padrão; MT19937 pelo módulo random)")
    print("gerador,semente,indice,valor")
    for seed in (5489, 0, 42, 4294967295):
        for i, v in enumerate(mt_outputs(seed, 2000)):
            if i < 20 or i in (623, 624, 625, 1247, 1248, 1999):
                print(f"twister,{seed},{i},{v}")
    # Sementes ≥ 2³²: `random.seed(n)` do Python faz init_by_array com as palavras de 32 bits de n
    # (a menos significativa primeiro), o mesmo esquema do Rust.
    for seed in (2**32 + 7, 2**63 + 12345, 2**64 - 1):
        r = random.Random(seed)
        for i, v in enumerate(r.getrandbits(32) for _ in range(1300)):
            if i < 10 or i in (623, 624, 1247, 1248, 1299):
                print(f"twister,{seed},{i},{v}")
    for seed in (0, 42, 12345, 18446744073709551615):
        for i, v in enumerate(mrg_outputs(seed, 20)):
            print(f"mrg32k3a,{seed},{i},{v}")
        for i, v in enumerate(xoshiro_outputs(seed, 20)):
            print(f"xoshiro256pp,{seed},{i},{v}")


if __name__ == "__main__":
    sys.exit(main())
