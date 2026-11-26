# 6. Funções embutidas

Primeiro conjunto (DT-40). Testes em `crates/chimalab-runtime/tests/interpretador.rs` e `algebra_linguagem.rs`. Funções aceitam argumentos posicionais; só `range` aceita nomeados.

## 6.1 Constantes
`pi`, `Inf` (`inf`), `NaN` (`nan`), `eps` (2⁻⁵²), `true`, `false`.

## 6.2 Construção e forma
| Função | Resultado |
|---|---|
| `zeros(n)`, `zeros(r, c, …)`, `ones(…)` | n×n ou r×c×… de zeros ou uns; sem argumentos, 1×1 |
| `eye(n)`, `eye(r, c)` | identidade |
| `range(a, b; length=n)` | n pontos igualmente espaçados, extremos exatos |
| `range(a, b; step=s)` | o mesmo que `a:s:b` |
| `size(A)`, `size(A, d)`, `r, c = size(A)` | dimensões (com várias saídas, a última absorve as restantes) |
| `numel`, `ndims`, `length` (maior dimensão; 0 se vazio), `isempty` | |

Tamanhos são inteiros ≥ 0; arrays acima de 2³¹ elementos dão `chimalab:runtime:arrayGrandeDemais`.

## 6.3 Reduções
`sum`, `mean`, `any`, `all`, `max`, `min` reduzem ao longo da primeira dimensão diferente de 1, ou da dimensão dada (`sum(A, 2)`).
- `sum` usa soma compensada (`sum([1e16 1 -1e16])` é 1); preserva inteiros (com saturação) e `single`.
- `sum([])` é 0; `mean([])` é NaN; `all([])` é verdadeiro; `any([])` é falso.
- `max`/`min` ignoram NaN; `m, i = max(v)` dá o índice (base 1) da primeira ocorrência; `max(a, b)` é elemento a elemento, com expansão implícita.
- `prod` (`prod([])` é 1), `cumsum` (soma compensada corrente) e `cumprod` seguem a mesma regra de dimensão; inteiros saturam a cada passo.
- Resultados complexos com parte imaginária nula voltam a ser reais (`prod([1i 1i])` é −1).

## 6.3.0 Estatística descritiva
| Função | Resultado |
|---|---|
| `median(x)`, `median(x, d)` | mediana ao longo da dimensão (a mesma regra de `sum`); NaN propaga |
| `mode(x)`, `mode(x, d)` | valor mais frequente; empate, o menor; NaN é ignorado |
| `var(x)`, `std(x)`, `var(x, d)`, `std(x, d)` | variância e desvio padrão amostrais (n − 1); `population=true` divide por n |
| `quantile(x, p)`, `quantile(x, p; type=k)` | quantis de todos os elementos de x nas probabilidades p (mesma forma de p); tipos 1 a 9 de Hyndman & Fan, padrão 7 (interpolação linear) |

Complexos ainda não são aceitos. Vazio: NaN.

## 6.3.1 Ordenação, busca e forma
| Função | Resultado |
|---|---|
| `sort(v)`, `s, i = sort(v)` | ordem crescente, estável, NaN no fim; `v[i]` é `s` |
| `sort(v, d)`, `sort(v, "descend")`, `sort(v, d, "descend")` | ao longo da dimensão d; decrescente (NaN continua no fim) |
| `find(x)`, `find(x, k)` | índices lineares (base 1) dos não nulos (NaN conta), os k primeiros; linha para vetor linha, coluna para o resto |
| `reshape(x, r, c, …)`, `reshape(x, [r c])` | mesma ordem de coluna, outra forma; uma dimensão pode ser `-1` (inferida, como no Python) |
| `diag(v)`, `diag(v, k)` | matriz com o vetor na k-ésima diagonal (k > 0 acima) |
| `diag(A)`, `diag(A, k)` | coluna com a k-ésima diagonal de A |
| `diff(x)`, `diff(x, k)`, `diff(x, k, d)` | diferenças sucessivas de ordem k ao longo da primeira dimensão diferente de 1 (ou de d) |
| `flip(x)`, `flip(x, d)` | inverte a ordem ao longo da primeira dimensão diferente de 1 (ou de d) |

Todas mantêm a classe do valor (inteiros, lógicos e caracteres continuam como eram). `sort` de complexos ainda não é suportado.

## 6.3.2 Conjuntos
| Função | Resultado |
|---|---|
| `unique(v)`, `u, i, j = unique(v)` | valores distintos em ordem crescente; cada NaN é distinto e fica no fim; `i` é a primeira ocorrência de cada um e `u[j]` reconstrói `v` |
| `ismember(a, s)`, `tf, loc = ismember(a, s)` | lógico da forma de `a`; `loc` é a primeira posição em `s` (0 se não está); NaN nunca é membro |
| `union(a, b)`, `intersect(a, b)`, `setdiff(a, b)` | ordenados e sem repetição; NaN de `a` fica na união e na diferença, nunca na interseção |

Comparação exata entre classes (`ismember(int8(3), [1 2 3])` é verdadeiro; um `int64` sem representação exata em `double` não é igual a nenhum `double`). Linha para vetor linha; coluna para o resto (nas operações binárias, coluna só se as duas entradas forem colunas).

## 6.4 Elementares (elemento a elemento)
`abs sqrt exp log log10 log2 sin cos tan asin acos atan sinh cosh tanh floor ceil round fix sign mod rem atan2 hypot`.
- `sqrt`, `log`, `log10` e `log2` de negativos, e `asin`/`acos` fora de [−1, 1], dão complexos no ramo principal; funções de complexos usam o ramo principal.
- `fix` trunca em direção a zero; `rem(a, b)` tem o sinal de a (`rem(a, 0)` é NaN), ao contrário de `mod`; `hypot` não estoura (`hypot(1e300, 1e300)`).
- `isnan`, `isinf`, `isfinite` (elemento a elemento, lógicos; num complexo, qualquer parte) e `isreal` (escalar).
- `real`, `imag`, `conj`, `angle` (argumento, em radianos); `class(x)` dá o nome da classe (`"function"` para funções).
- `round` arredonda o meio para longe de zero; `round(x, n)` arredonda em n casas decimais (n negativo: à esquerda da vírgula); `sign(z)` é z/|z|.
- `mod(a, b)` tem o sinal de b; `mod(a, 0)` é a.
- Inteiros aceitam só `abs sign floor ceil round` (as demais pedem `double(…)`).

## 6.4.1 Funções especiais e de inteiros
| Função | Resultado |
|---|---|
| `gamma(x)`, `gammaln(x)`, `psi(x)`, `factorial(n)` | Γ (também complexa), ln Γ (também complexa), digama, n! |
| `erf`, `erfc` (também complexas), `erfcx`, `erfinv`, `erfcinv` | função erro, complementar, escalonada e inversas |
| `zeta(s)`, `expint(x)` | zeta de Riemann; integral exponencial E₁ |
| `beta(a, b)`, `betaln(a, b)` | beta e seu logaritmo, com expansão implícita |
| `besselj(ν, x)`, `bessely(ν, x)`, `besseli(ν, x)`, `besselk(ν, x)` | funções de Bessel de primeira e segunda espécie e modificadas, com expansão implícita |
| `gcd(a, b)`, `lcm(a, b)` | máximo divisor comum e mínimo múltiplo comum (inteiros) |
| `primes(n)`, `isprime(x)`, `factor(n)` | primos até n; teste de primalidade elemento a elemento (não inteiros e negativos dão falso); fatores primos de n |
| `nchoosek(n, k)` | coeficiente binomial; aviso `chimalab:special:perdaDePrecisao` acima de 2⁵³ |

Erros de domínio chegam com `chimalab:special:dominio`.

## 6.4.2 Números aleatórios
| Função | Resultado |
|---|---|
| `rand(…)`, `randn(…)` | uniformes em (0, 1) e normais padrão, com as formas de `zeros` (`rand()`, `rand(n)`, `rand(r, c, …)`) |
| `randi(hi, …)`, `randi([lo, hi], …)` | inteiros uniformes em [1, hi] ou [lo, hi], sem viés |
| `randperm(n)`, `randperm(n, k)` | permutação de 1..n; k elementos distintos |
| `seed(n)`, `seed(n; algorithm="twister")` | recomeça a sequência com a semente n (e, se dado, o algoritmo: `philox`, padrão, `threefry`, `twister`, `mrg32k3a`, `xoshiro256pp`) |

Cada sessão começa com o gerador padrão e semente 0, então repete os mesmos números: resultados reprodutíveis sem configurar nada.

## 6.5 Classes
`double single logical char int8 int16 int32 int64 uint8 uint16 uint32 uint64` convertem com as regras da §4.2 (arredondamento do meio para longe de zero, saturação).

## 6.6 Álgebra linear
| Função | Resultado |
|---|---|
| `det(A)`, `inv(A)`, `trace(A)` | determinante, inversa (aviso se singular), traço |
| `norm(x)`, `norm(x, p)` | p = 1, 2, Inf, `"fro"` ou p > 0 (vetores) |
| `rank(A)`, `rank(A, tol)`, `cond(A)` | posto numérico, condicionamento na norma 2 |
| `eig(A)`, `V, D = eig(A)` | autovalores em coluna; A·V = V·D |
| `svd(A)`, `U, S, V = svd(A)` | valores singulares decrescentes; A = U·S·Vᴴ |
| `chol(A)` | R triangular superior com A = Rᴴ·R |
| `L, U, P = lu(A)`, `L, U = lu(A)` | P·A = L·U; com duas saídas, L já permutado (A = L·U) |
| `Q, R = qr(A)`, `qr(A)` | A = Q·R; com uma saída, R |
| `dot(a, b)`, `cross(a, b)` | produto interno (conjuga o primeiro: Σ conj(aᵢ)·bᵢ) e produto vetorial (vetores de 3 elementos) |
| `kron(A, B)` | produto de Kronecker |
| `expm(A)`, `pinv(A)`, `null(A)`, `orth(A)` | exponencial, pseudoinversa, bases do núcleo e da imagem |

Valem para matrizes reais e complexas; resultados complexos com parte imaginária nula voltam a ser reais. Erros da biblioteca chegam com os identificadores `chimalab:linalg:…` (matriz não quadrada, não definida positiva, valores não finitos).

## 6.6.1 Polinômios, sinais e interpolação
| Função | Resultado |
|---|---|
| `polyval(p, x)` | valor do polinômio (coeficientes em potências **decrescentes**, `p[1]·xⁿ + … + p[end]`) em cada elemento de x, inclusive complexo |
| `roots(p)` | raízes em coluna (zeros à esquerda ignorados); reais quando a parte imaginária some |
| `polyfit(x, y, n)`, `p, nr = polyfit(…)` | ajuste de grau n por mínimos quadrados, em linha; `nr` é a norma do resíduo; aviso se o problema for mal condicionado |
| `polyder(p)`, `polyint(p)`, `polyint(p, k)` | derivada e integral do polinômio (k: constante de integração, padrão 0) |
| `trapz(y)`, `trapz(x, y)`, `cumtrapz(…)` | regra dos trapézios (acumulada) ao longo da primeira dimensão diferente de 1; x são as abscissas, ou um escalar com o espaçamento |
| `conv(u, v)`, `conv(u, v; shape="same")`, `shape="valid"` | convolução de vetores (produto de polinômios); coluna se u for coluna |
| `fft(x)`, `fft(x, n)`, `fft(x, n, d)`, `ifft(…)` | transformada discreta de Fourier e inversa ao longo da primeira dimensão diferente de 1 (ou d), com preenchimento ou corte em n (`[]` para o tamanho natural) |
| `interp1(x, y, xq)` | interpolação 1-D com a forma de xq; `method=` `"linear"` (padrão), `"nearest"`, `"previous"`, `"next"`, `"pchip"`, `"akima"`, `"makima"`, `"spline"`; `extrap=` `"nan"` (padrão), `"extrapolate"`, `"clamp"` ou um número |

Erros da interpolação chegam com os identificadores `chimalab:interp:…` (pontos duplicados, formas incompatíveis); os da linguagem, com `chimalab:execucao:…`.

## 6.7 Funções que recebem funções
| Função | Resultado |
|---|---|
| `map(f, A)` | f aplicada a cada elemento; mesma forma de A; a classe segue a concatenação dos resultados |
| `integral(f, a, b)`, `v, erro = integral(…)` | ∫ₐᵇ f por quadratura adaptativa (limites infinitos, a > b, integrando complexo); opções `abs_tol`, `rel_tol`, `waypoints` |
| `fzero(f, x0)`, `fzero(f, [a, b])`, `x, fx = fzero(…)` | raiz de f a partir de um chute ou de um intervalo com troca de sinal; opção `tol_x` |
| `fminbnd(f, a, b)`, `x, fx = fminbnd(…)` | mínimo de f em [a, b]; opção `tol_x` |
| `fminsearch(f, x0)`, `x, fx = fminsearch(…)` | mínimo de f de várias variáveis (Nelder–Mead), com x na forma de x0; opções `tol_x`, `tol_fun`, `max_iter` |
| `integral2(f, xa, xb, ya, yb)` | ∫∫ f(x, y) no retângulo (limites numéricos, inclusive infinitos); opções `abs_tol`, `rel_tol` |
| `t, y = ode(f, tspan, y0)` | y' = f(t, y) (nome decidido na resposta 53.A): `t` em coluna, `y` com um estado por linha; `tspan = [t0, tf]` dá os passos do integrador, mais instantes dão a saída exatamente neles; opções `method` (`"dopri5"` padrão, `"bs23"`, `"ndf"`, `"bdf"`, `"radau"`, `"rosenbrock"`), `rel_tol`, `abs_tol` (escalar ou um por componente), `max_step` |

`f` pode ser anônima (`x -> x^2 - 2`), `def` ou embutida (`sqrt`) e deve devolver um escalar (`chimalab:execucao:resultadoNaoEscalar`). Um erro dentro de `f` interrompe a rotina e sai com a posição de dentro de `f`. Quando a integral não garante a tolerância, aparece um aviso `chimalab:integral:…`. Na `ode`, `f(t, y)` recebe `y` em coluna e devolve as n derivadas (`chimalab:execucao:derivadaComTamanhoErrado` se não); término parcial (passo mínimo, valor não finito, limite de passos), rigidez suspeita no método explícito e tolerância elevada aparecem como avisos `chimalab:ode:…`.

## 6.8 Outras
| Função | Efeito |
|---|---|
| `disp(x)` | exibe o valor sem o nome (textos sem aspas) |
| `error(msg)`, `error(id, msg)` | levanta um erro |
| `isequal(a, b, …)` | mesma forma e mesmos valores |
| `who()` | nomes das variáveis do workspace (ou da função em execução) |
| `whos()` | tabela com nome, tamanho, classe e bytes ocupados nesta implementação (`char` usa 4 bytes por caractere) |
| `clear()`, `clear("x", "y")` | apaga todas as variáveis, ou as nomeadas (nome inexistente é ignorado) |
| `help(nome)`, `help("nome")`, `help()` | ajuda tirada desta especificação; `help(f)` de uma função `def` mostra a assinatura |
