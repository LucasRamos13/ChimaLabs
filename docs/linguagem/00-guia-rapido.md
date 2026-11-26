# Guia rápido

Uma volta pela linguagem no console. Cada sessão abaixo é executada por um teste (`guia_rapido_executa_como_escrito`, em `crates/chimalab-runtime/tests/interpretador.rs`), então as respostas são exatamente as que o ChimaLab dá. As linhas com `>>` são digitadas; `..` continua um bloco, e um `..` sozinho (linha em branco) fecha o bloco. Para abrir o console: `chimalab`; para rodar um arquivo: `chimalab arquivo.chl`.

## Contas e variáveis
Sem `;`, o resultado aparece; uma expressão sem nome vai para `ans`.

```
>> x = 2 + 3 * 4
x = 14
>> y = x^2 / 7
y = 28
>> y + 1
ans = 29
>> z = sqrt(-4)
z = 0 + 2i
```

## Matrizes
Linhas separadas por `;` (ou por quebra de linha); `'` transpõe; `*` é o produto matricial e `.*` o elemento a elemento.

```
>> A = [1 2; 3 4]
A = 2×2
  1  2
  3  4
>> A * A
ans = 2×2
   7  10
  15  22
>> A .* A
ans = 2×2
   1   4
   9  16
>> b = [5; 6];
>> s = A \ b
s = 2×1
  -4.000000000
           4.5
```

`A \ b` resolve o sistema A·s = b. O −4 aparece com todos os algarismos porque o valor calculado não é exatamente −4 (o arredondamento da eliminação deixa uma diferença na última casa); um número só aparece sem casas decimais quando é inteiro de verdade.

## Índices
Índices começam em 1 e usam colchetes; `end` é o último; uma condição seleciona elementos.

```
>> A[2, 1]
ans = 3
>> A[end, :]
ans = 1×2
  3  4
>> v = 1:5
v = 1×5
  1  2  3  4  5
>> v[v > 2]
ans = 1×3
  3  4  5
>> v[end + 1] = 6;
>> sum(v)
ans = 21
```

## Funções
Funções anônimas com `->`; funções nomeadas com `def`, que podem devolver vários valores.

```
>> quadrado = t -> t^2
quadrado = função anônima (t)
>> map(quadrado, [1 2 3])
ans = 1×3
  1  4  9
>> def media_e_desvio(dados):
..     return mean(dados), std(dados)
..
>> m, d = media_e_desvio([2 4 4 4 5 5 7 9])
m = 5
d = 2.138089935
>> f"média = {m}, desvio = {d:.4f}"
ans = "média = 5, desvio = 2.1381"
```

## Laços e condições
Blocos por indentação, como no Python; `for` percorre as colunas.

```
>> for k in 1:3:
..     if mod(k, 2) == 1:
..         disp(f"{k} é ímpar")
..     else:
..         disp(f"{k} é par")
..
1 é ímpar
2 é par
3 é ímpar
```

## Cálculo numérico
Álgebra linear, integrais, raízes e equações diferenciais recebem matrizes e funções.

```
>> eig([2 1; 1 2])
ans = 2×1
  1
  3
>> integral(t -> exp(-t^2), -Inf, Inf)
ans = 1.772453851
>> fzero(t -> cos(t) - t, [0 1])
ans = 0.7390851332
>> t, y = ode((t, y) -> -2 * y, [0 1], 1; rel_tol=1e-10, abs_tol=1e-12);
>> y[end]
ans = 0.1353352832
```

A integral é √π; a raiz é o ponto fixo do cosseno; a última resposta é e⁻².

## Quando algo dá errado
Os erros dizem onde e, quando dá, sugerem a correção.

```
>> media = 3;
>> medai + 1
erro: linha 1, coluna 1: `medai` não está definido; você quis dizer `media`?
>> v(2)
erro: linha 1, coluna 1: `v` é um array: para indexar, use v[…]
```

A ajuda de cada função vem da especificação: `help(sum)`, `help("interp1")`, `help()`.
