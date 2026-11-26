# Pendências de Investigação

| ID | Pendência | Bloqueia | Depende de | Como resolver |
|---|---|---|---|---|
| P-02 | Itens em INVESTIGADA PARCIALMENTE: F015, F077, F104, F105, F138, F139, F152, F160, F182, F220, F222, F227 | Planejamento desses itens | — | Pesquisa em literatura e especificações públicas |
| P-03 | F124 (componente criptográfico) NÃO VERIFICÁVEL | — | — | Definir a necessidade a partir de casos de uso próprios |
| P-10 | Questões jurídicas em aberto (`LICENCAS_E_RESTRICOES.txt` §15) | — | Advogado | Decisões por pesquisa registradas no DT-19 e DT-21 |
| P-11 | Formato `.mat` v5/v7.3: implementar só a partir de fontes independentes e permissivas (DT-10) | F110 | — | — |
| P-12 | ~~Licença do próprio ChimaLab~~ **RESOLVIDA** 2026-11-17: MIT OR Apache-2.0 (DT-15) | — | — | — |
| P-14 | Confirmar a lista de blocos do Fluxo (F201) a partir de necessidades de engenharia | F201 | — | Especificação própria por bloco em `docs/blocos/` |

## Pergunta 25 (decidida em 2026-11-19, P → B): vale implementar o Schur com deflação agressiva agora?
A medição em release (2026-11-18, matriz aleatória densa) foi:

| n | `schur` próprio | `eig` do faer (multishift com deflação agressiva, com autovetores) |
|---|---|---|
| 100 | 8 ms | 13 ms |
| 300 | 144 ms | 420 ms |
| 600 | 641 ms | 595 ms |
| 1000 | 2,77 s | 1,74 s |

- O faer não expõe a forma de Schur real (o módulo é `pub(crate)`).
- A deflação agressiva completa exige reordenar blocos 2×2 da forma de Schur (Bai & Demmel, 1993) e restaurar a forma de Hessenberg: centenas de linhas delicadas para ganhar ~1,5–2×, e só em n ≥ 1000.
- Opções:
  - A: implementar agora;
  - B: adiar até haver uso real com n grande (registrado como 💡);
  - P: pesquisar e decidir.
- Recomendação: B.
- **Decisão (P, por delegação do responsável): B, adiar.**
  - Até n = 600 o `schur` próprio já empata com a referência (multishift com deflação agressiva) e, em n = 300, é mais rápido. Em n = 1000 a diferença é de 1,6×.
  - O custo é alto (reordenação de blocos 2×2 e restauração da forma de Hessenberg), e o risco numérico também.
  - **Reabrir quando** houver uso real com n ≥ 1000 em que o Schur domine o tempo (por exemplo, funções de matriz grandes), ou se a medição piorar.


## Perguntas 27–36 (decididas em 2026-11-20, ver DT-34): superfície da linguagem própria
**Respostas:** 27.A, 28.C, 29.P → A, 30.A, 31.A, 32.A, 33.A, 34.P → A + B, 35.C, 36.A. As decisões por delegação (29 e 34) estão justificadas no DT-34.

Os blocos numéricos do DT-21 já começaram. O próximo passo é a linguagem (F001–F006), e estas escolhas definem a cara dela para o usuário. Cada pergunta traz uma recomendação, e a opção **P** significa "pesquise e decida pela opção mais racional".
- **27. Produto de matrizes:**
  - A: `*` é o produto matricial e `.*` o elemento a elemento, como na notação matemática, em Julia e no original;
  - B: `*` é elemento a elemento e `@` o matricial, como em NumPy;
  - P.
  - *Recomendação: A.* É o mais natural para álgebra linear e para quem vem de livros. B confunde o público científico.
- **28. Literais de matriz:**
  - A: `[1 2; 3 4]`;
  - B: `[[1, 2], [3, 4]]`;
  - C: as duas formas;
  - P.
  - *Recomendação: C.* `[1 2; 3 4]` é conciso para matemática, e a forma com vírgulas é familiar a quem vem do Python.
- **29. Intervalos:**
  - A: `1:10` e `1:2:10`, inclusivos, coerentes com a base 1;
  - B: `range(1, 10)` exclusivo, como no Python;
  - C: `1..10`;
  - P.
  - *Recomendação: A,* com `range` disponível como função.
- **30. Definição de funções:**
  - A: `def f(x, y=2):` com `return a, b` e `a, b = f(x)`, como no Python;
  - B: `function [a, b] = f(x):`, com saídas nomeadas;
  - P.
  - *Recomendação: A,* mais `nargout` implícito: a função sabe quantas saídas foram pedidas, para cumprir o CA de F002.
- **31. Comentários:**
  - A: `#` e `#= … =#` para blocos;
  - B: `%`;
  - P.
  - *Recomendação: A.*
- **32. Textos:**
  - A: aspas simples e duplas equivalentes, mais f-strings `f"x = {x}"`;
  - B: só aspas duplas;
  - P.
  - *Recomendação: A.*
- **33. Indexação e chamada:**
  - A: `A[i, j]` indexa e `f(x)` chama, sem ambiguidade, como no Python;
  - B: parênteses para as duas coisas, como no original;
  - P.
  - *Recomendação: A.* Elimina uma classe de erros e permite mensagens melhores.
- **34. Exibição no console:**
  - A: toda expressão mostra o valor, e `;` no fim suprime;
  - B: como no Python, só a última expressão da célula aparece;
  - P.
  - *Recomendação: A* no console (CA de F001), com B nos documentos executáveis.
- **35. Transposta:**
  - A: `A'` conjugada e `A.'` simples;
  - B: `A.T` e `A.H`;
  - C: as duas formas;
  - P.
  - *Recomendação: C.*
- **36. Último índice:**
  - A: palavra-chave `end`, como em `A[end]` e `A[end-1]`;
  - B: índices negativos, com `A[-1]` sendo o último, como no Python;
  - C: as duas formas;
  - P.
  - *Recomendação: A.* Com base 1, índices negativos confundem: −1 e 0 têm significados próximos demais.

## Pergunta 37 (decidida em 2026-11-20): consumo do CI
- **Resposta: A + B + C.** No push só Linux x86; matriz completa e cobertura em PR, semanalmente e sob demanda; pushes agrupados; repositório público (DT-35).

## Perguntas 38–39 (decididas em 2026-11-20): publicação
- **38: repositório público novo** (`LucasRamos13/ChimaLabs`), criado pelo responsável, com o estado atual num commit inicial, sem o histórico privado (DT-35).
- **39: o material da investigação em sala limpa fica fora do repositório público**, guardado localmente pelo responsável.


## Perguntas 40–48 (decididas em 2026-11-25, ver DT-39): construções da linguagem e primeiro interpretador
**Respostas:** 40.A, 41.A, 42.C, 43.A, 44.A, 45.B pedida e A aplicada (conflito com a barra invertida, conforme a instrução do responsável), 46.A, 47.A, 48.A.

O léxico (DT-36) já está pronto e não depende destas respostas; o parser e o interpretador (F001, F002, F006) dependem. Cada pergunta traz uma recomendação; **P** significa "pesquise e decida pela opção mais racional".
- **40. Palavras de bloco:**
  - A: como no Python: `if c:` / `elif c:` / `else:`, `for k in expr:`, `while c:`, `break`, `continue`, `try:` / `except e:`, `match x:` / `case v:` para o `switch`;
  - B: palavras do original com dois-pontos (`elseif`, `switch`/`case`/`otherwise`, `catch`);
  - P.
  - *Recomendação: A.* Coerente com a indentação e com `def`/`return` (DT-34).
- **41. `for` sobre matriz:**
  - A: percorre as colunas (o CA do F006: `for k in [1 2; 3 4]` itera 2 vezes, com colunas);
  - B: percorre os elementos em ordem de coluna;
  - P.
  - *Recomendação: A*, que coincide com B para vetores linha, o caso mais comum.
- **42. Operadores lógicos:**
  - A: `and`, `or`, `not` com curto-circuito sobre escalares, mais `&`, `|`, `!` elemento a elemento;
  - B: `&&`, `||`, `!` com curto-circuito e `&`, `|` elemento a elemento;
  - C: as duas formas;
  - P.
  - *Recomendação: A.* Lê como o Python e separa bem o lógico escalar do elemento a elemento.
- **43. Literal imaginário:**
  - A: `2i` e `2j`;
  - B: só `2im` (Julia);
  - C: só `2j` (Python);
  - P.
  - *Recomendação: A*, sem prejuízo para nomes como `i` e `j` em laços (o sufixo só vale colado ao número).
- **44. Diferente:**
  - A: `!=`, com `~=` dando erro que sugere `!=`;
  - B: `~=`;
  - C: os dois;
  - P.
  - *Recomendação: A.*
- **45. Continuação de linha:**
  - A: só implícita, dentro de `( )`, `[ ]` e `{ }`;
  - B: também `\` no fim da linha;
  - C: também `...`;
  - P.
  - *Recomendação: A.* `\` já é a divisão à esquerda, e `...` lembra o original sem ganho.
- **46. Fim de bloco no console:**
  - A: uma linha em branco encerra o bloco (como no console do Python);
  - B: palavra `end` opcional;
  - P.
  - *Recomendação: A.* O léxico já detecta a entrada incompleta (`incomplete`); o CA do F001 vira "o bloco só executa depois da linha em branco".
- **47. Funções anônimas:**
  - A: `x -> x^2` e `(x, y) -> x + y` (Julia);
  - B: `lambda x: x^2` (Python);
  - C: `@(x) x^2`;
  - P.
  - *Recomendação: A.* Curta e legível; o `@` fica livre para decoradores ou anotações.
- **48. Primeiro interpretador:**
  - A: avaliador que percorre a árvore de sintaxe (AST), em `chimalab-runtime`, já com `ans`, exibição e `;`, para o console funcionar logo; o bytecode do DT-03 vem depois, quando houver medição de desempenho que o justifique;
  - B: bytecode e máquina virtual desde o início (DT-03), com o console só depois;
  - P.
  - *Recomendação: A.* Entrega o console (CAs do F001) cedo e define a semântica por testes; o bytecode passa a ser uma otimização verificada contra o avaliador da árvore, que vira oráculo.

## Perguntas 49–51 (decididas em 2026-11-26: 49.A, 50.A, 51.A): semântica do avaliador (DT-40)
As recomendações já estão implementadas e testadas; trocar é barato enquanto não há código de usuário.
- **49. Condição de `if`/`while` com array:**
  - A: vazio é falso (cumpre o critério "if [] é falso" do F006); escalar vale pelo valor; array com dois ou mais elementos é erro, com a sugestão `all(…)`/`any(…)`;
  - B: verdadeiro se não vazio e todos os elementos não nulos (comportamento tradicional dos ambientes de matriz);
  - P.
  - *Recomendação: A* (implementada). `if A == B` com matrizes é uma fonte comum de erro silencioso; o erro explícito ensina a forma certa. Uma versão anterior desta recomendação tornava `if []` erro também, o que mudaria um critério de aceitação do catálogo; por isso o vazio ficou falso.
- **50. Variáveis livres em funções anônimas:**
  - A: capturam o valor na criação;
  - B: leem o valor atual na chamada (como no Python);
  - P.
  - *Recomendação: A.* Uma função anônima passada a um integrador ou otimizador não muda se a variável mudar depois.
- **51. Escopo das funções `def`:**
  - A: isolado (só parâmetros e o que a função cria);
  - B: leitura das variáveis do workspace/módulo, como no Python;
  - P.
  - *Recomendação: A.* Funções reproduzíveis, sem dependência escondida do estado do console; constantes globais entram depois por módulos.

## Perguntas 52–53 (decididas em 2026-11-26: 52.A, 53.A)
- **52. Ctrl+C no console:** o mecanismo de interrupção existe e está testado (`Interpretador::sinal_de_interrupcao`); falta o tratador do sinal no executável, que exige uma dependência (Rust sem `unsafe` não instala tratador de sinal só com a biblioteca padrão).
  - A: adicionar o crate `ctrlc` (MIT/Apache-2.0), cumprindo o §6 do CLAUDE.md (licenças das transitivas, `cargo deny`, LICENCAS Parte B);
  - B: esperar a IDE (Tauri), que interrompe pelo próprio canal;
  - P.
  - *Recomendação: A.* O console de linha de comando é o caminho de teste do F001 e o CA4 pede Ctrl+C.
- **53. Nomes das funções de EDO na linguagem (F049):** as rotinas existem no `chimalab-math`, mas falta a interface.
  - A: uma função só, `ode(f, [t0, tf], y0; method="dopri5", rel_tol=…, abs_tol=…)`, devolvendo `t, y` (implementada provisoriamente assim);
  - B: uma função por método, com nomes próprios (`rk45`, `rk23`, `bdf`, `rosenbrock`…);
  - P.
  - *Recomendação: A.* Um nome só e o método como opção é mais fácil de ensinar e de trocar; o padrão pode escolher entre explícito e implícito pela rigidez detectada.

## Perguntas 54–56 (decididas em 2026-11-26: 54.A, 55.P, 56.A): texto e estruturas
- **54. Texto (F010):** hoje `'abc'` e `"abc"` são vetores de caracteres (DT-34), e `"a" + "b"` soma os códigos (195). O catálogo pede `"a" + "b" == "ab"`.
  - A: `+` entre dois textos concatena; texto com número é erro (`chimalab:runtime:classesIncompativeis`, com a sugestão de `double(…)` para a conta com códigos); funções `upper`, `lower`, `strip`, `replace`, `startswith`, `endswith`, `contains`, `split`/`join` (com F009) e `str2double` (texto inválido dá NaN);
  - B: manter a aritmética de códigos e concatenar só com `[a b]`;
  - P.
  - *Recomendação: A.* É o que o critério do F010 e a ergonomia do Python pedem; a conta com códigos continua possível explicitamente.
- **55. Estruturas, células e dicionários (F009, F025):** nenhuma sintaxe foi fixada.
  - A: estruturas com campos `s.a = 1` e literal `struct(a=1, b="x")`; células com `{1, "a", [1 2]}` e índice `c{2}`; dicionários com `dict("a" => 1)`;
  - B: estruturas e dicionários pelo mesmo literal do Python `{"a": 1}`, sem células (listas `[[…]]` heterogêneas);
  - P.
  - *Recomendação: P.* A escolha afeta F009, F024 (tabelas) e F025 de uma vez; vale pesquisar a ergonomia antes de fixar.
- **56. Unidade de indexação de texto (DT-06 × implementação):** o DT-06 manda indexar texto por unidades UTF-16, mas o modelo de valores implementado desde o início guarda um escalar Unicode por elemento (`'olá'` tem 3 elementos, e um emoji é 1 elemento, não 2).
  - A: oficializar o escalar Unicode (atualizar o DT-06): o comprimento é o número de caracteres que a pessoa vê na maioria dos casos, sem pares substitutos;
  - B: mudar a implementação para UTF-16, como o DT-06 diz (compatibilidade de índices com Java/.NET);
  - P.
  - *Recomendação: A.* É o comportamento atual, coberto por testes, e evita metade de um emoji como "caractere"; a interoperabilidade com UTF-16 fica nas funções de conversão.

**Respostas de 2026-11-26 (responsável: "pode usar suas recomendações"):** 49.A, 50.A, 51.A, 52.A, 53.A, 54.A, 55.P, 56.A.
- Já implementadas: 49, 50, 51 e 53.
- A fazer: 52 (adicionar `ctrlc` seguindo o §6 do CLAUDE.md e ligar ao `sinal_de_interrupcao`); 54 (`+` entre textos concatena, texto com número é erro, funções de texto); 55 (pesquisar a ergonomia de estruturas, células e dicionários e decidir, registrando em DT); 56 (atualizar o DT-06 para o escalar Unicode).

