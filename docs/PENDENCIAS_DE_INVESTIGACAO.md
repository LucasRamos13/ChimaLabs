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

