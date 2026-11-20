<!-- GERADO por ferramentas/gerar_docs.py. Não edite à mão: edite docs/dados/funcionalidades.json. -->

# Testes por Funcionalidade

Casos derivados dos critérios de aceitação. Complementa `PLANO_DE_TESTES.md`. Nenhum caso usa saída de produto proprietário como oráculo. Os oráculos são soluções analíticas, implementações de referência independentes ou especificações públicas.

## TF-001 — Interpretador interativo (janela de comandos)

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F001/` (a criar).
- **Entradas:** Texto de comando; teclas de controle (histórico, interrupção).
- **Casos de erro:** Erro de sintaxe com posição; identificador indefinido; interrupção pelo usuário.
- **Casos extremos:** Comandos multilinha (blocos if/for abertos); sintaxe de comando ('hold on'); entrada muito longa.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-001.1 | Expressão '1+1' exibe 2 e define ans | CA1 |
| TF-001.2 | ';' suprime exibição | CA2 |
| TF-001.3 | Bloco multilinha só executa após 'end' | CA3 |
| TF-001.4 | Ctrl+C interrompe laço infinito em <1s sem perder o workspace | CA4 |
| TF-001.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-002 — Scripts e funções em arquivo

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F002/` (a criar).
- **Entradas:** Arquivos de código (.m equivalente no formato compatível e extensão própria).
- **Casos de erro:** Muitos/poucos argumentos; saída não atribuída; recursão máxima excedida.
- **Casos extremos:** Recursão profunda; arquivo modificado em disco durante a sessão (recarregamento); script com função local no final.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-002.1 | Função com 3 saídas retorna somente as pedidas | CA1 |
| TF-002.2 | Edição do arquivo em disco é refletida na chamada seguinte | CA2 |
| TF-002.3 | Limite de recursão gera erro identificável e configurável | CA3 |
| TF-002.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-003 — Tipos numéricos e lógicos fundamentais

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F003/` (a criar).
- **Entradas:** Literais, conversões de tipo.
- **Casos de erro:** Combinação inválida de classes; overflow tratado por saturação (sem erro).
- **Casos extremos:** -0, NaN payload, intmin/intmax, eps, realmin denormais, conversão de NaN para inteiro (0).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-003.1 | int8(127)+1 == 127 | CA1 |
| TF-003.2 | uint8(3.5) == 4 | CA2 |
| TF-003.3 | class(int16(1)*2.7) == 'int16' | CA3 |
| TF-003.4 | sqrt(-4) == 2i | CA4 |
| TF-003.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-004 — Arrays N-dimensionais e indexação

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F004/` (a criar).
- **Entradas:** Arrays, índices.
- **Casos de erro:** Índice fora dos limites na leitura; índice não inteiro/negativo; dimensões incompatíveis na atribuição.
- **Casos extremos:** Arrays vazios com dimensões (0x3); indexação com mais dimensões que o array; colapso de dimensões finais.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-004.1 | A(end+1)=x cresce o vetor | CA1 |
| TF-004.2 | A(:) retorna coluna | CA2 |
| TF-004.3 | Remover linha com A(2,:)=[] | CA3 |
| TF-004.4 | size(zeros(0,3)) == [0 3] | CA4 |
| TF-004.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-005 — Operadores matriciais, elemento a elemento e expansão implícita

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F005/` (a criar).
- **Entradas:** Operandos arrays.
- **Casos de erro:** Dimensões incompatíveis; operandos não escalares em &&.
- **Casos extremos:** Expansão entre 1xN e Mx1; operações com vazio; matriz singular em '\' gera aviso.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-005.1 | [1 2 3]+[10;20] produz 2x3 | CA1 |
| TF-005.2 | A\b resolve sistema com resíduo < 1e-12 em matriz bem condicionada | CA2 |
| TF-005.3 | Aviso emitido para matriz singular | CA3 |
| TF-005.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-006 — Controle de fluxo

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F006/` (a criar).
- **Entradas:** Código.
- **Casos de erro:** Uso de break fora de laço; catch sem identificador.
- **Casos extremos:** for sobre matriz vazia; switch com cell de alternativas; modificação da variável do laço no corpo.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-006.1 | for k=[1 2;3 4] itera 2 vezes com colunas | CA1 |
| TF-006.2 | if [] é falso | CA2 |
| TF-006.3 | switch 'abc' casa com {'x','abc'} | CA3 |
| TF-006.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-007 — Programação orientada a objetos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F007/` (a criar).
- **Entradas:** Arquivos de definição de classe; pastas de classe.
- **Casos de erro:** Acesso negado; violação de validação; método abstrato não implementado.
- **Casos extremos:** Recarregar definição com instâncias existentes; herança diamante; redefinição de indexação (subsref/subsasgn equivalente).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-007.1 | Classe de referência compartilha estado entre cópias | CA1 |
| TF-007.2 | Validação de propriedade rejeita tipo inválido com mensagem identificável | CA2 |
| TF-007.3 | Evento notifica listeners em ordem de registro | CA3 |
| TF-007.4 | Metadados da classe consultáveis | CA4 |
| TF-007.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-008 — Validação de argumentos de função

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F008/` (a criar).
- **Entradas:** Declaração de argumentos.
- **Casos de erro:** Tipo/tamanho incompatível; nome-valor desconhecido.
- **Casos extremos:** Argumentos nome-valor derivados de propriedades de classe; validadores personalizados.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-008.1 | Chamada f(x='a') com validador numérico falha antes do corpo | CA1 |
| TF-008.2 | Valor padrão aplicado quando omitido | CA2 |
| TF-008.3 | Mensagem identifica posição do argumento | CA3 |
| TF-008.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-009 — Arrays de células e estruturas

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F009/` (a criar).
- **Entradas:** Valores quaisquer.
- **Casos de erro:** Campo inexistente; conteúdo de cell com chaves em contexto inválido.
- **Casos extremos:** Struct vazio com campos; cell aninhada profunda; deal com listas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-009.1 | [s.a] concatena campos de array de struct | CA1 |
| TF-009.2 | fieldnames preserva ordem | CA2 |
| TF-009.3 | c{2:3} expande em múltiplas saídas | CA3 |
| TF-009.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-010 — Texto: arrays de caracteres e strings

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F010/` (a criar).
- **Entradas:** Textos, padrões.
- **Casos de erro:** Padrão inválido; conversão numérica inválida resulta NaN.
- **Casos extremos:** String ausente (<missing>); texto vazio vs ausente; expressões regulares com retrocesso exponencial.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-010.1 | "a"+"b" == "ab" | CA1 |
| TF-010.2 | regexp com tokens nomeados retorna struct | CA2 |
| TF-010.3 | str2double('x') é NaN | CA3 |
| TF-010.4 | split/join/replace funcionam com arrays de string | CA4 |
| TF-010.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-011 — Formatação e exibição de valores

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F011/` (a criar).
- **Entradas:** Valores, especificadores de formato.
- **Casos de erro:** Especificador inválido.
- **Casos extremos:** Inteiros grandes, complexos, NaN/Inf, arrays vazios, exibição truncada de arrays grandes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-011.1 | sprintf('%d,',[1 2 3]) == '1,2,3,' | CA1 |
| TF-011.2 | format long mostra 15 dígitos significativos para double | CA2 |
| TF-011.3 | Exibição personalizável por método da classe | CA3 |
| TF-011.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-012 — Tratamento de erros e avisos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F012/` (a criar).
- **Entradas:** Identificador, mensagem, estado.
- **Casos de erro:** Identificador mal formado.
- **Casos extremos:** Erro durante limpeza; avisos repetidos; exceções aninhadas (causes).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-012.1 | warning('off',id) suprime apenas aquele aviso | CA1 |
| TF-012.2 | MException tem stack com arquivo/linha | CA2 |
| TF-012.3 | onCleanup executa ao sair do escopo mesmo em erro | CA3 |
| TF-012.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-013 — Workspaces, escopo e avaliação dinâmica

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F013/` (a criar).
- **Entradas:** Nomes, expressões em texto.
- **Casos de erro:** Nome inválido; workspace inexistente.
- **Casos extremos:** eval com erro e catch; clear dentro de função; variáveis com nomes de funções (shadowing).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-013.1 | Persistente mantém valor entre chamadas | CA1 |
| TF-013.2 | clear all limpa persistentes | CA2 |
| TF-013.3 | whos lista tamanho, classe e bytes | CA3 |
| TF-013.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-014 — Caminho de busca, pacotes e pastas especiais

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F014/` (a criar).
- **Entradas:** Diretórios, declarações import.
- **Casos de erro:** Pasta inexistente; nome ambíguo.
- **Casos extremos:** Duplicatas de nomes (shadowing com aviso); pasta removida em uso.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-014.1 | addpath/rmpath/savepath alteram e persistem caminho | CA1 |
| TF-014.2 | Função em pasta privada não visível externamente | CA2 |
| TF-014.3 | import de namespace resolve nome curto | CA3 |
| TF-014.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-015 — Módulos e namespaces modernos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F015/` (a criar).
- **Entradas:** Pastas de módulo com descritor.
- **Casos de erro:** Dependência circular.
- **Casos extremos:** Compatibilidade com caminho tradicional.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-015.1 | Função interna do módulo inacessível de fora | CA1 |
| TF-015.2 | Descritor inválido gera erro claro | CA2 |
| TF-015.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-016 — Funções anônimas, handles e funções de ordem superior

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F016/` (a criar).
- **Entradas:** Expressões, handles.
- **Casos de erro:** Saída não escalar com UniformOutput=true.
- **Casos extremos:** Handles para funções aninhadas; comparação de handles.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-016.1 | Captura por valor: alterar variável depois não muda o handle | CA1 |
| TF-016.2 | arrayfun com saída não escalar exige opção | CA2 |
| TF-016.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-017 — Eventos, listeners e referências fracas

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F017/` (a criar).
- **Entradas:** Objetos, nomes de evento, callbacks.
- **Casos de erro:** Evento inexistente.
- **Casos extremos:** Listener que destrói a fonte; recursão de eventos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-017.1 | notify dispara todos os listeners | CA1 |
| TF-017.2 | Listener desabilitado não executa | CA2 |
| TF-017.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-018 — Interrupção, pausa e temporizadores

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F018/` (a criar).
- **Entradas:** Tempo, callbacks.
- **Casos de erro:** Período inválido.
- **Casos extremos:** Callback mais longo que o período; erro no callback.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-018.1 | Timer periódico executa N vezes com desvio <10ms | CA1 |
| TF-018.2 | Erro no callback chama ErrorFcn | CA2 |
| TF-018.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-019 — Desempenho de execução (compilação JIT)

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F019/` (a criar).
- **Entradas:** Código.
- **Casos de erro:** Nenhum visível ao usuário.
- **Casos extremos:** Código dinâmico (eval) não compilável — fallback.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-019.1 | Laço escalar de 1e7 iterações < 5x de C equivalente (meta) | CA1 |
| TF-019.2 | Resultados idênticos entre JIT e interpretador em toda a suíte | CA2 |
| TF-019.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-020 — Operações de conjuntos, ordenação, busca e bits

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F020/` (a criar).
- **Entradas:** Arrays.
- **Casos de erro:** Tipos incompatíveis.
- **Casos extremos:** Tipos mistos; strings; NaN em unique (cada NaN distinto).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-020.1 | unique([3 1 NaN NaN]) mantém 2 NaN | CA1 |
| TF-020.2 | sort estável | CA2 |
| TF-020.3 | ismember retorna índices | CA3 |
| TF-020.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-021 — Datas e horas (datetime)

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F021/` (a criar).
- **Entradas:** Textos, números, componentes.
- **Casos de erro:** Formato inválido; zona desconhecida.
- **Casos extremos:** Horário ambíguo/inexistente na transição DST; segundos bissextos (opcional 'UTCLeapSeconds').

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-021.1 | Soma de 1 dia atravessando DST mantém hora local | CA1 |
| TF-021.2 | Leitura de texto com formato explícito | CA2 |
| TF-021.3 | NaT propagado em operações | CA3 |
| TF-021.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-022 — Durações e durações de calendário

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F022/` (a criar).
- **Entradas:** Números, textos.
- **Casos de erro:** Formato inválido.
- **Casos extremos:** 31-jan + 1 mês; durações negativas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-022.1 | datetime(2024,1,31)+calmonths(1) == 29-fev-2024 | CA1 |
| TF-022.2 | duração exibida no formato escolhido | CA2 |
| TF-022.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-023 — Dados categóricos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F023/` (a criar).
- **Entradas:** Textos/números.
- **Casos de erro:** Valor fora das categorias protegidas.
- **Casos extremos:** Combinação de categóricos com conjuntos diferentes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-023.1 | countcats correto | CA1 |
| TF-023.2 | Comparação ordinal | CA2 |
| TF-023.3 | Ausente exibido como <undefined> | CA3 |
| TF-023.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-024 — Tabelas e tabelas temporais

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F024/` (a criar).
- **Entradas:** Arrays, arquivos.
- **Casos de erro:** Nome duplicado; altura incompatível.
- **Casos extremos:** Variáveis multicoluna; tabelas aninhadas; tempos duplicados/não ordenados.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-024.1 | join interno por chave | CA1 |
| TF-024.2 | retime para dados horários com média | CA2 |
| TF-024.3 | groupsummary por categoria | CA3 |
| TF-024.4 | T.Var acesso por nome | CA4 |
| TF-024.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-025 — Dicionários e mapas

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F025/` (a criar).
- **Entradas:** Chaves e valores.
- **Casos de erro:** Chave inexistente.
- **Casos extremos:** Chaves NaN; chaves string vs char.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-025.1 | Inserção/busca O(1) amortizado | CA1 |
| TF-025.2 | Semântica de valor do dicionário vs referência do mapa | CA2 |
| TF-025.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-026 — Valores ausentes padronizados

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F026/` (a criar).
- **Entradas:** Arrays/tabelas.
- **Casos de erro:** Método inválido.
- **Casos extremos:** Tabelas com tipos mistos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-026.1 | fillmissing linear em vetor com NaN | CA1 |
| TF-026.2 | rmmissing remove linhas de tabela com ausentes | CA2 |
| TF-026.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-040 — Álgebra linear densa

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F040/` (a criar).
- **Entradas:** Matrizes.
- **Casos de erro:** Matriz não quadrada onde exigido; NaN/Inf.
- **Casos extremos:** Matrizes vazias, singulares, complexas, grandes (multithread).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-040.1 | Resíduo relativo < 1e-12 em suíte de matrizes de teste | CA1 |
| TF-040.2 | svd de matriz 1000x1000 em tempo comparável a LAPACK otimizado | CA2 |
| TF-040.3 | Aviso de singularidade | CA3 |
| TF-040.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-041 — Matrizes esparsas e solvers iterativos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F041/` (a criar).
- **Entradas:** Matrizes esparsas.
- **Casos de erro:** Não convergência (flag + aviso).
- **Casos extremos:** Matriz com zero explícito; muito grande; mal condicionada.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-041.1 | pcg converge em matriz de Poisson 2D | CA1 |
| TF-041.2 | eigs retorna k maiores autovalores | CA2 |
| TF-041.3 | Memória proporcional a nnz | CA3 |
| TF-041.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-042 — Funções elementares e especiais

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F042/` (a criar).
- **Entradas:** Arrays numéricos.
- **Casos de erro:** Domínio inválido gera NaN/complexo conforme regra.
- **Casos extremos:** Argumentos extremos; complexos; inteiros grandes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-042.1 | sind(180)==0 exato | CA1 |
| TF-042.2 | gamma em teste de referência com erro relativo < 1e-14 | CA2 |
| TF-042.3 | besselj ordens fracionárias | CA3 |
| TF-042.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-043 — Estatística descritiva e pré-processamento de dados

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F043/` (a criar).
- **Entradas:** Arrays/tabelas; dimensão; flags NaN.
- **Casos de erro:** Opção inválida.
- **Casos extremos:** Entrada vazia (mean de vazio é NaN); tipos inteiros.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-043.1 | mean(x,'omitnan') ignora NaN | CA1 |
| TF-043.2 | movmean com janela par/ímpar | CA2 |
| TF-043.3 | rmoutliers método mediana | CA3 |
| TF-043.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-044 — Interpolação e ajuste de curvas básico

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F044/` (a criar).
- **Entradas:** Pontos, valores, consultas.
- **Casos de erro:** Grade não monotônica.
- **Casos extremos:** Pontos duplicados; consulta fora do domínio (NaN padrão).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-044.1 | interp1 spline reproduz polinômio cúbico | CA1 |
| TF-044.2 | scattered interpolant com Delaunay | CA2 |
| TF-044.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-045 — Polinômios

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F045/` (a criar).
- **Entradas:** Coeficientes.
- **Casos de erro:** Dados insuficientes.
- **Casos extremos:** Grau alto mal condicionado (aviso).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-045.1 | roots([1 -3 2]) == [2;1] | CA1 |
| TF-045.2 | polyfit grau n em n+1 pontos interpola | CA2 |
| TF-045.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-046 — Transformadas de Fourier e filtragem básica

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F046/` (a criar).
- **Entradas:** Arrays; tamanho; dimensão.
- **Casos de erro:** Dimensão inválida.
- **Casos extremos:** Tamanho primo grande; entradas reais vs complexas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-046.1 | ifft(fft(x)) ≈ x (1e-12) | CA1 |
| TF-046.2 | fft de tamanho primo 10007 correta | CA2 |
| TF-046.3 | filter equivale a equação de diferenças | CA3 |
| TF-046.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-047 — Geometria computacional

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F047/` (a criar).
- **Entradas:** Pontos, polígonos.
- **Casos de erro:** Pontos insuficientes.
- **Casos extremos:** Pontos duplicados; precisão finita.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-047.1 | delaunay de 1e5 pontos aleatórios satisfaz critério de círculo vazio | CA1 |
| TF-047.2 | polyshape union/intersect com área correta | CA2 |
| TF-047.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-048 — Integração numérica e quadratura

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F048/` (a criar).
- **Entradas:** Funções/handles, limites, tolerâncias.
- **Casos de erro:** Tolerância não alcançada (aviso).
- **Casos extremos:** Singularidade nos extremos; função não vetorizada.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-048.1 | integral(@(x) exp(-x.^2),-Inf,Inf) ≈ sqrt(pi) | CA1 |
| TF-048.2 | trapz em dados amostrados | CA2 |
| TF-048.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-049 — Equações diferenciais ordinárias e algébrico-diferenciais

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F049/` (a criar).
- **Entradas:** Função derivada, intervalo, condição inicial, opções.
- **Casos de erro:** Falha de tolerância mínima (aviso + resultado parcial).
- **Casos extremos:** Problemas rígidos; descontinuidades; eventos simultâneos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-049.1 | Oscilador de Van der Pol rígido resolvido por BDF | CA1 |
| TF-049.2 | Evento terminal para no instante correto ±tol | CA2 |
| TF-049.3 | Comparação com solução analítica em problemas de referência | CA3 |
| TF-049.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-050 — Problemas de contorno, retardo e EDP 1-D

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F050/` (a criar).
- **Entradas:** Funções, malha, condições.
- **Casos de erro:** Não convergência.
- **Casos extremos:** Retardos que se anulam; malha inicial ruim.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-050.1 | bvp de referência com erro < tol | CA1 |
| TF-050.2 | dde23 em problema publicado | CA2 |
| TF-050.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-051 — Otimização básica e raízes

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F051/` (a criar).
- **Entradas:** Função objetivo, chute inicial, opções.
- **Casos de erro:** Intervalo inválido.
- **Casos extremos:** Função descontínua; sem mudança de sinal em fzero.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-051.1 | fminsearch na Rosenbrock converge | CA1 |
| TF-051.2 | fzero encontra raiz com bracket | CA2 |
| TF-051.3 | Funções de plotagem de iteração chamadas a cada passo | CA3 |
| TF-051.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-052 — Grafos e redes

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F052/` (a criar).
- **Entradas:** Listas de arestas, matrizes de adjacência.
- **Casos de erro:** Nó inexistente.
- **Casos extremos:** Grafo vazio; laços; pesos negativos (Bellman-Ford).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-052.1 | shortestpath Dijkstra correto | CA1 |
| TF-052.2 | maxflow em rede de teste | CA2 |
| TF-052.3 | plot com layout force | CA3 |
| TF-052.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-053 — Números aleatórios

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F053/` (a criar).
- **Entradas:** Semente, gerador, dimensões.
- **Casos de erro:** Gerador desconhecido.
- **Casos extremos:** Reprodução bit a bit de sequências de outros produtos: NÃO exigida (requisito: algoritmos padrão publicados).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-053.1 | rng(42) reprodutível na mesma versão | CA1 |
| TF-053.2 | Testes estatísticos (TestU01 SmallCrush) aprovados | CA2 |
| TF-053.3 | Subfluxos não sobrepostos | CA3 |
| TF-053.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-054 — Precisão estendida e aritmética especial

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F054/` (a criar).
- **Entradas:** Valores.
- **Casos de erro:** —
- **Casos extremos:** Valores no limite.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-054.1 | dec2bin/hex2dec ida-e-volta | CA1 |
| TF-054.2 | flintmax correto | CA2 |
| TF-054.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-055 — Funções de data e tempo legadas

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F055/` (a criar).
- **Entradas:** Números seriais/textos.
- **Casos de erro:** Formato inválido.
- **Casos extremos:** Anos bissextos; fuso local.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-055.1 | tic/toc mede intervalo com resolução < 1µs | CA1 |
| TF-055.2 | Conversão serial↔texto | CA2 |
| TF-055.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-060 — Sistema de objetos gráficos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F060/` (a criar).
- **Entradas:** Chamadas de criação; pares propriedade-valor.
- **Casos de erro:** Propriedade inexistente/valor inválido.
- **Casos extremos:** Muitos objetos (1e5); exclusão durante callback.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-060.1 | set/get e notação de ponto equivalentes | CA1 |
| TF-060.2 | Padrão definido na raiz afeta novas figuras | CA2 |
| TF-060.3 | findobj por propriedade | CA3 |
| TF-060.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-061 — Gráficos 2-D

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F061/` (a criar).
- **Entradas:** Vetores/matrizes, estilos.
- **Casos de erro:** Dimensões incompatíveis.
- **Casos extremos:** NaN cria lacunas; datetime/duration/categorical nos eixos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-061.1 | plot com NaN gera lacuna | CA1 |
| TF-061.2 | eixo datetime formatado | CA2 |
| TF-061.3 | bar empilhado soma correta | CA3 |
| TF-061.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-062 — Gráficos 3-D, superfícies e volumes

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F062/` (a criar).
- **Entradas:** Grades, volumes.
- **Casos de erro:** Tamanhos incompatíveis.
- **Casos extremos:** Grades irregulares; NaN em superfícies; transparência ordenada.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-062.1 | surf com colormap e iluminação | CA1 |
| TF-062.2 | isosurface correto em volume de teste | CA2 |
| TF-062.3 | Rotação de câmera interativa a ≥30 fps em 1e6 triângulos (meta) | CA3 |
| TF-062.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-063 — Gráficos especializados de alto nível

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F063/` (a criar).
- **Entradas:** Tabelas/arrays.
- **Casos de erro:** Variável inexistente.
- **Casos extremos:** Dados ausentes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-063.1 | heatmap a partir de tabela agrega contagens | CA1 |
| TF-063.2 | boxchart com grupos | CA2 |
| TF-063.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-064 — Anotações, legendas, rótulos e texto matemático

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F064/` (a criar).
- **Entradas:** Textos.
- **Casos de erro:** LaTeX inválido (aviso, texto bruto).
- **Casos extremos:** Unicode; fórmulas longas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-064.1 | Legenda lista séries visíveis | CA1 |
| TF-064.2 | LaTeX \frac renderizado | CA2 |
| TF-064.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-065 — Mapas de cores e estilos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F065/` (a criar).
- **Entradas:** Matrizes Nx3.
- **Casos de erro:** Formato inválido.
- **Casos extremos:** Mapas com poucos níveis.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-065.1 | Tema escuro altera cores padrão de figura | CA1 |
| TF-065.2 | colormap personalizado aplicado | CA2 |
| TF-065.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-066 — Layouts de múltiplos gráficos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F066/` (a criar).
- **Entradas:** Dimensões de grade.
- **Casos de erro:** Índice fora da grade.
- **Casos extremos:** Mosaicos aninhados; spans.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-066.1 | Layout 2x3 com título compartilhado | CA1 |
| TF-066.2 | linkaxes sincroniza zoom | CA2 |
| TF-066.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-067 — Interação com gráficos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F067/` (a criar).
- **Entradas:** Eventos de usuário.
- **Casos de erro:** —
- **Casos extremos:** Gráficos com 1e6 pontos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-067.1 | Zoom por roda do mouse | CA1 |
| TF-067.2 | Dica de dados exibe valores | CA2 |
| TF-067.3 | Escovação propaga para gráficos ligados | CA3 |
| TF-067.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-068 — Animação e captura de quadros

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F068/` (a criar).
- **Entradas:** Atualizações de propriedades.
- **Casos de erro:** —
- **Casos extremos:** Animação com 1e4 quadros.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-068.1 | drawnow limitrate não excede 20 fps | CA1 |
| TF-068.2 | getframe retorna imagem RGB | CA2 |
| TF-068.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-069 — Exportação e impressão de figuras

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F069/` (a criar).
- **Entradas:** Figura, formato, opções.
- **Casos de erro:** Formato não suportado.
- **Casos extremos:** Figuras com transparência em vetor; fontes ausentes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-069.1 | PDF vetorial contém texto selecionável | CA1 |
| TF-069.2 | PNG 300 dpi com dimensões corretas | CA2 |
| TF-069.3 | Figura salva e recarregada idêntica | CA3 |
| TF-069.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-070 — Exibição de imagens

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F070/` (a criar).
- **Entradas:** Matrizes.
- **Casos de erro:** —
- **Casos extremos:** Imagens 16 bits; com alfa.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-070.1 | imagesc escala automática | CA1 |
| TF-070.2 | RGB uint8 exibido sem conversão de cor | CA2 |
| TF-070.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-075 — Componentes de interface programáticos

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F075/` (a criar).
- **Entradas:** Chamadas de criação e propriedades.
- **Casos de erro:** Propriedade inválida.
- **Casos extremos:** Muitos componentes; callbacks lentos (interrupção/fila).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-075.1 | Aplicativo com 50 componentes abre < 1 s | CA1 |
| TF-075.2 | Callback de botão recebe evento | CA2 |
| TF-075.3 | Tabela editável sincroniza dados | CA3 |
| TF-075.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-076 — Editor visual de aplicativos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F076/` (a criar).
- **Entradas:** Interações de arrastar/soltar.
- **Casos de erro:** Arquivo corrompido.
- **Casos extremos:** Conversão de apps legados.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-076.1 | Criar app com botão e callback sem escrever layout manualmente | CA1 |
| TF-076.2 | Reabrir preserva código do usuário | CA2 |
| TF-076.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-077 — Interfaces legadas baseadas em figura

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F077/` (a criar).
- **Entradas:** Arquivos de layout legado.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-077.1 | uicontrol cria botão funcional | CA1 |
| TF-077.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-078 — Diálogos padrão

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F078/` (a criar).
- **Entradas:** Opções.
- **Casos de erro:** —
- **Casos extremos:** Cancelamento; multisseleção.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-078.1 | Cancelar retorna 0/vazio | CA1 |
| TF-078.2 | Barra de progresso atualiza e fecha | CA2 |
| TF-078.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-079 — Empacotamento e compartilhamento de apps

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F079/` (a criar).
- **Entradas:** Projeto de app.
- **Casos de erro:** Arquivo ausente.
- **Casos extremos:** Dependências de outras extensões.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-079.1 | Empacotar e instalar app em outra máquina | CA1 |
| TF-079.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-085 — Ambiente desktop com painéis

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F085/` (a criar).
- **Entradas:** Interações de UI.
- **Casos de erro:** —
- **Casos extremos:** Telas HiDPI; tela pequena.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-085.1 | Layout persiste entre sessões | CA1 |
| TF-085.2 | Tema escuro aplicado a todos os painéis | CA2 |
| TF-085.3 | Painel desacoplado em outro monitor | CA3 |
| TF-085.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-086 — Editor de código

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F086/` (a criar).
- **Entradas:** Texto do usuário.
- **Casos de erro:** Arquivo somente leitura.
- **Casos extremos:** Arquivos > 10k linhas; codificações.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-086.1 | Autocompletar sugere variáveis e funções do caminho | CA1 |
| TF-086.2 | Executar seção avalia apenas a seção | CA2 |
| TF-086.3 | Renomear variável atualiza ocorrências no escopo | CA3 |
| TF-086.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-087 — Análise estática de código

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F087/` (a criar).
- **Entradas:** Código.
- **Casos de erro:** —
- **Casos extremos:** Código com erro de sintaxe parcial.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-087.1 | Variável não usada sinalizada | CA1 |
| TF-087.2 | Supressão por comentário respeitada | CA2 |
| TF-087.3 | Relatório de pasta exportável | CA3 |
| TF-087.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-088 — Depurador

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F088/` (a criar).
- **Entradas:** Pontos de parada, comandos.
- **Casos de erro:** Linha inválida.
- **Casos extremos:** Parada em código JIT; em callback de timer.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-088.1 | Parada condicional dispara só quando verdadeira | CA1 |
| TF-088.2 | 'parar em erro' pausa no local do erro | CA2 |
| TF-088.3 | Variável alterada no modo depuração afeta execução | CA3 |
| TF-088.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-089 — Profiler de desempenho

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F089/` (a criar).
- **Entradas:** Execução de código.
- **Casos de erro:** —
- **Casos extremos:** Código recursivo; multithread.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-089.1 | Relatório lista tempo por linha | CA1 |
| TF-089.2 | Gráfico de chama navegável | CA2 |
| TF-089.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-090 — Navegador de arquivos e pasta atual

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F090/` (a criar).
- **Entradas:** Ações de arquivo.
- **Casos de erro:** Permissão negada.
- **Casos extremos:** Pastas grandes (1e5 arquivos).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-090.1 | cd no prompt atualiza painel | CA1 |
| TF-090.2 | Excluir pede confirmação | CA2 |
| TF-090.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-091 — Navegador de workspace e editor de variáveis

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F091/` (a criar).
- **Entradas:** Variáveis.
- **Casos de erro:** Valor inválido.
- **Casos extremos:** Variáveis enormes (paginação).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-091.1 | Editar célula atualiza variável | CA1 |
| TF-091.2 | Seleção gera gráfico | CA2 |
| TF-091.3 | Edições registradas como código | CA3 |
| TF-091.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-092 — Ferramenta de importação de dados

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F092/` (a criar).
- **Entradas:** Arquivos.
- **Casos de erro:** Arquivo ilegível.
- **Casos extremos:** Arquivos grandes; cabeçalhos múltiplos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-092.1 | Importar CSV com datas detectadas | CA1 |
| TF-092.2 | Script gerado reproduz importação | CA2 |
| TF-092.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-093 — Histórico de comandos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F093/` (a criar).
- **Entradas:** Comandos.
- **Casos de erro:** —
- **Casos extremos:** Histórico corrompido.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-093.1 | Seta para cima com prefixo filtra histórico | CA1 |
| TF-093.2 | Criar script a partir da seleção | CA2 |
| TF-093.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-094 — Projetos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F094/` (a criar).
- **Entradas:** Pasta de projeto.
- **Casos de erro:** Metadados inválidos.
- **Casos extremos:** Projetos aninhados/referenciados; arquivos fora do projeto.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-094.1 | Abrir projeto adiciona pastas ao caminho | CA1 |
| TF-094.2 | Análise de dependências encontra arquivos faltantes | CA2 |
| TF-094.3 | Exportar como arquivo compactado | CA3 |
| TF-094.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-095 — Controle de versão integrado

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F095/` (a criar).
- **Entradas:** Repositórios.
- **Casos de erro:** Falha de autenticação.
- **Casos extremos:** Conflitos; repositórios grandes; LFS.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-095.1 | Commit/branch/merge via UI | CA1 |
| TF-095.2 | Indicador de modificado no navegador | CA2 |
| TF-095.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-096 — Comparação e mesclagem de arquivos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F096/` (a criar).
- **Entradas:** Dois/três arquivos.
- **Casos de erro:** Tipos incompatíveis.
- **Casos extremos:** Arquivos binários grandes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-096.1 | Diff de texto com realce intra-linha | CA1 |
| TF-096.2 | Comparar dois arquivos de dados lista variáveis diferentes | CA2 |
| TF-096.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-097 — Preferências e configurações

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F097/` (a criar).
- **Entradas:** Valores de configuração.
- **Casos de erro:** Valor inválido.
- **Casos extremos:** Arquivo corrompido (restauração).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-097.1 | Valor de sessão sobrepõe usuário | CA1 |
| TF-097.2 | Exportar/importar preferências | CA2 |
| TF-097.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-098 — Atalhos de teclado e ações personalizáveis

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F098/` (a criar).
- **Entradas:** Mapeamentos.
- **Casos de erro:** Conflito de atalho.
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-098.1 | Atalho personalizado executa ação | CA1 |
| TF-098.2 | Favorito executa trecho de código | CA2 |
| TF-098.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-099 — Visualizador de diagramas de classes e dependências

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F099/` (a criar).
- **Entradas:** Pastas/classes.
- **Casos de erro:** —
- **Casos extremos:** Projetos grandes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-099.1 | Diagrama de classes mostra herança | CA1 |
| TF-099.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-100 — Opções de inicialização e modos sem interface

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F100/` (a criar).
- **Entradas:** Argumentos de linha de comando.
- **Casos de erro:** Argumento inválido.
- **Casos extremos:** Sem display (headless) com gráficos (renderização offscreen).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-100.1 | chimalab -batch "disp(1)" imprime 1 e sai com 0 | CA1 |
| TF-100.2 | Erro sai com código ≠ 0 | CA2 |
| TF-100.3 | Gráficos exportáveis sem display | CA3 |
| TF-100.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-101 — Barra de ferramentas contextual (faixa de opções)

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F101/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-101.1 | Aba contextual aparece ao focar editor | CA1 |
| TF-101.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-102 — Notificações e mensagens do sistema

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F102/` (a criar).
- **Entradas:** Eventos.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-102.1 | Notificação de tarefa em segundo plano concluída | CA1 |
| TF-102.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-103 — Busca global

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F103/` (a criar).
- **Entradas:** Consulta.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-103.1 | Busca encontra função por nome parcial | CA1 |
| TF-103.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-104 — Ambiente via navegador e acesso remoto

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F104/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-104.1 | Interface acessível por navegador local com autenticação por token | CA1 |
| TF-104.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-105 — Assistente de IA/sugestões de código

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F105/` (a criar).
- **Entradas:** Contexto do editor.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-105.1 | Sugestões opcionais e desativáveis; nenhum dado enviado sem consentimento | CA1 |
| TF-105.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-106 — Documentos executáveis (scripts vivos)

- **Tipos:** Unitário, Integração, Ponta a ponta, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F106/` (a criar).
- **Entradas:** Documento.
- **Casos de erro:** Arquivo corrompido (recuperação).
- **Casos extremos:** Documentos grandes; saídas pesadas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-106.1 | Gráfico aparece inline abaixo do código | CA1 |
| TF-106.2 | Slider reexecuta seção ao mudar | CA2 |
| TF-106.3 | Salvar/reabrir preserva saídas | CA3 |
| TF-106.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-107 — Exportação de documentos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F107/` (a criar).
- **Entradas:** Documento.
- **Casos de erro:** —
- **Casos extremos:** Equações e imagens em todos os formatos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-107.1 | Exportar para PDF com equações | CA1 |
| TF-107.2 | Exportar ipynb abre no Jupyter | CA2 |
| TF-107.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-108 — Publicação de scripts com marcação

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F108/` (a criar).
- **Entradas:** Script.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-108.1 | Publicar script com seções gera HTML com índice | CA1 |
| TF-108.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-109 — Verificação ortográfica

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F109/` (a criar).
- **Entradas:** Texto.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-109.1 | Palavra incorreta sublinhada | CA1 |
| TF-109.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-110 — Arquivos de dados nativos (MAT)

- **Tipos:** Unitário, Integração, Segurança, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F110/` (a criar).
- **Entradas:** Variáveis/arquivo.
- **Casos de erro:** Arquivo corrompido; versão não suportada.
- **Casos extremos:** Arquivos > 2 GB; objetos de classes desconhecidas (carregar como struct + aviso).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-110.1 | Ida-e-volta de todos os tipos básicos | CA1 |
| TF-110.2 | Ler arquivo v5/v7/v7.3 de referência gerado por ferramenta de terceiros | CA2 |
| TF-110.3 | Leitura parcial de matriz 10 GB sem carregar inteira | CA3 |
| TF-110.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-111 — E/S de arquivo de baixo nível

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F111/` (a criar).
- **Entradas:** Arquivos, formatos.
- **Casos de erro:** Arquivo inexistente; permissão.
- **Casos extremos:** Arquivos > 4 GB; fim de linha misto.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-111.1 | fread com ordem big-endian | CA1 |
| TF-111.2 | memmapfile acesso aleatório | CA2 |
| TF-111.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-112 — Texto delimitado e de largura fixa

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F112/` (a criar).
- **Entradas:** Arquivos texto.
- **Casos de erro:** Arquivo malformado (regra configurável: erro/omitir/preencher).
- **Casos extremos:** Aspas, delimitadores múltiplos, linhas malformadas, BOM, codificações.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-112.1 | CSV com cabeçalho → tabela com tipos corretos | CA1 |
| TF-112.2 | Opções editadas mudam tipo de coluna | CA2 |
| TF-112.3 | textscan com formato | CA3 |
| TF-112.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-113 — Planilhas

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F113/` (a criar).
- **Entradas:** Arquivos de planilha.
- **Casos de erro:** Folha inexistente.
- **Casos extremos:** Células mescladas; fórmulas (valor calculado); datas 1900/1904.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-113.1 | Ler intervalo nomeado | CA1 |
| TF-113.2 | Escrever tabela preservando formatação existente | CA2 |
| TF-113.3 | xls legado lido | CA3 |
| TF-113.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-114 — JSON

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F114/` (a criar).
- **Entradas:** Texto/arquivos JSON.
- **Casos de erro:** JSON inválido com posição.
- **Casos extremos:** Números grandes; Unicode; profundidade.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-114.1 | Ida-e-volta preserva estrutura | CA1 |
| TF-114.2 | Pretty print opcional | CA2 |
| TF-114.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-115 — XML

- **Tipos:** Unitário, Integração, Segurança, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F115/` (a criar).
- **Entradas:** Arquivos XML.
- **Casos de erro:** XML malformado.
- **Casos extremos:** XML grande (streaming); entidades externas desativadas por segurança.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-115.1 | XPath retorna nós | CA1 |
| TF-115.2 | XXE bloqueado por padrão | CA2 |
| TF-115.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-116 — HTML e documentos de texto

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F116/` (a criar).
- **Entradas:** Arquivos HTML/DOCX.
- **Casos de erro:** —
- **Casos extremos:** Tabelas aninhadas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-116.1 | readtable de tabela HTML | CA1 |
| TF-116.2 | readtable de tabela em .docx | CA2 |
| TF-116.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-117 — Formatos colunares (Parquet, Arrow/Feather)

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F117/` (a criar).
- **Entradas:** Arquivos.
- **Casos de erro:** Tipo sem mapeamento.
- **Casos extremos:** Tipos aninhados; datas com fuso.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-117.1 | Ida-e-volta Parquet com datetime | CA1 |
| TF-117.2 | Leitura de colunas selecionadas | CA2 |
| TF-117.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-118 — Imagens

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F118/` (a criar).
- **Entradas:** Arquivos.
- **Casos de erro:** Formato desconhecido.
- **Casos extremos:** Imagens enormes; CMYK; paletas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-118.1 | PNG 16 bits com alfa ida-e-volta | CA1 |
| TF-118.2 | TIFF multipágina lido por índice | CA2 |
| TF-118.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-119 — Áudio e vídeo

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F119/` (a criar).
- **Entradas:** Arquivos/dispositivos.
- **Casos de erro:** Codec não suportado.
- **Casos extremos:** Codecs indisponíveis na plataforma.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-119.1 | audioread WAV/FLAC correto | CA1 |
| TF-119.2 | Gravar e reproduzir com dispositivo padrão | CA2 |
| TF-119.3 | VideoWriter MP4 legível em players comuns | CA3 |
| TF-119.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-120 — Formatos científicos (HDF5, HDF4, NetCDF, CDF, FITS)

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F120/` (a criar).
- **Entradas:** Arquivos.
- **Casos de erro:** Arquivo inválido.
- **Casos extremos:** Arquivos > memória; tipos compostos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-120.1 | h5write/h5read ida-e-volta com atributos | CA1 |
| TF-120.2 | NetCDF leitura de variável parcial | CA2 |
| TF-120.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-121 — Sistema de arquivos e arquivos compactados

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F121/` (a criar).
- **Entradas:** Caminhos.
- **Casos de erro:** Permissão; inexistente.
- **Casos extremos:** Links simbólicos; nomes Unicode; caminhos longos no Windows.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-121.1 | dir recursivo com ** | CA1 |
| TF-121.2 | zip/unzip ida-e-volta | CA2 |
| TF-121.3 | Leitura de CSV em armazenamento de objetos compatível S3 | CA3 |
| TF-121.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-122 — Comandos do sistema e ambiente

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F122/` (a criar).
- **Entradas:** Comandos.
- **Casos de erro:** Comando inexistente.
- **Casos extremos:** Comandos interativos; codificação.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-122.1 | system retorna status e saída | CA1 |
| TF-122.2 | clipboard copiar/colar texto | CA2 |
| TF-122.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-123 — Gerenciamento de segredos

- **Tipos:** Unitário, Integração, Segurança, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F123/` (a criar).
- **Entradas:** Nome do segredo, valor.
- **Casos de erro:** Segredo inexistente.
- **Casos extremos:** Cofre indisponível (Linux sem serviço).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-123.1 | Segredo persiste entre sessões e não aparece em texto | CA1 |
| TF-123.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-124 — Funções criptográficas/hash internas

- **Tipos:** Unitário, Integração, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F124/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-124.1 | Requisito a definir após investigação em documentação pública | CA1 |
| TF-124.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-130 — Datastores

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F130/` (a criar).
- **Entradas:** Pastas/arquivos/opções.
- **Casos de erro:** Arquivo ilegível (pular/errar).
- **Casos extremos:** Arquivos heterogêneos; remotos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-130.1 | Datastore de 1000 CSV itera em blocos | CA1 |
| TF-130.2 | transform aplica função lazy | CA2 |
| TF-130.3 | partition(n,i) cobre todos os arquivos | CA3 |
| TF-130.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-131 — Arrays altos (avaliação adiada) e MapReduce

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F131/` (a criar).
- **Entradas:** Datastore.
- **Casos de erro:** —
- **Casos extremos:** Operações não suportadas em modo adiado (erro explícito).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-131.1 | mean de tall array igual ao mean em memória | CA1 |
| TF-131.2 | Número de passagens minimizado | CA2 |
| TF-131.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-135 — Acesso web (HTTP/REST)

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F135/` (a criar).
- **Entradas:** URLs, opções.
- **Casos de erro:** Status HTTP de erro; tempo-limite; TLS inválido.
- **Casos extremos:** Respostas grandes (streaming); proxy com autenticação.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-135.1 | webread JSON → struct | CA1 |
| TF-135.2 | Upload multipart | CA2 |
| TF-135.3 | Proxy configurável nas preferências | CA3 |
| TF-135.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-136 — FTP, e-mail e sockets

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F136/` (a criar).
- **Entradas:** Endereços, portas, dados.
- **Casos de erro:** Host inacessível.
- **Casos extremos:** Conexão interrompida.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-136.1 | tcpclient eco local | CA1 |
| TF-136.2 | sendmail via servidor de teste | CA2 |
| TF-136.3 | SFTP listar/baixar | CA3 |
| TF-136.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-137 — Portas seriais e Bluetooth

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F137/` (a criar).
- **Entradas:** Porta, parâmetros.
- **Casos de erro:** Porta ocupada.
- **Casos extremos:** Desconexão física.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-137.1 | serialport loopback | CA1 |
| TF-137.2 | Listar portas disponíveis | CA2 |
| TF-137.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-138 — Sensores móveis e IoT

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F138/` (a criar).
- **Entradas:** Conta/dispositivo.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-138.1 | Requisito equivalente com protocolo aberto (MQTT/HTTP) a definir | CA1 |
| TF-138.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-139 — Gerenciador de hardware e pacotes de suporte

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F139/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-139.1 | Requisito a definir | CA1 |
| TF-139.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-140 — Funções compiladas externas (C/C++/Fortran)

- **Tipos:** Unitário, Integração, Segurança, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F140/` (a criar).
- **Entradas:** Código-fonte nativo.
- **Casos de erro:** Falha de compilação/carregamento.
- **Casos extremos:** Compiladores ausentes; ABI.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-140.1 | Compilar e chamar função C que soma arrays | CA1 |
| TF-140.2 | Erro nativo não derruba o processo (isolamento opcional) | CA2 |
| TF-140.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-141 — Motor embutível (chamar o runtime de outras linguagens)

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F141/` (a criar).
- **Entradas:** Chamadas de API.
- **Casos de erro:** Sessão perdida.
- **Casos extremos:** Múltiplas sessões; threads.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-141.1 | Python inicia runtime, avalia e recebe array NumPy | CA1 |
| TF-141.2 | Conectar a sessão compartilhada | CA2 |
| TF-141.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-142 — API de arquivos de dados para outras linguagens

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F142/` (a criar).
- **Entradas:** Arquivos.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-142.1 | Programa C lê arquivo de dados escrito pelo runtime | CA1 |
| TF-142.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-143 — Interoperabilidade com Python

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F143/` (a criar).
- **Entradas:** Módulos/expressões Python.
- **Casos de erro:** Módulo inexistente.
- **Casos extremos:** Versões múltiplas de Python; GIL; travamentos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-143.1 | py.math.sqrt(4) == 2 | CA1 |
| TF-143.2 | ndarray ↔ array numérico sem cópia | CA2 |
| TF-143.3 | Modo fora de processo isola falhas | CA3 |
| TF-143.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-144 — Interoperabilidade com Java

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F144/` (a criar).
- **Entradas:** Classes Java.
- **Casos de erro:** Classe inexistente.
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-144.1 | Criar java.util.ArrayList e adicionar elemento | CA1 |
| TF-144.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-145 — Interoperabilidade com .NET e COM

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F145/` (a criar).
- **Entradas:** Assemblies/ProgIDs.
- **Casos de erro:** Assembly inexistente.
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-145.1 | Chamar System.DateTime.Now | CA1 |
| TF-145.2 | Automação COM de planilha (se instalada) | CA2 |
| TF-145.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-146 — Bibliotecas compartilhadas C e interfaces C++ geradas

- **Tipos:** Unitário, Integração, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F146/` (a criar).
- **Entradas:** Cabeçalhos, bibliotecas.
- **Casos de erro:** Tipo não suportado.
- **Casos extremos:** Templates; ponteiros; callbacks.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-146.1 | loadlibrary + calllib em libm | CA1 |
| TF-146.2 | Interface C++ gerada para classe simples | CA2 |
| TF-146.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-150 — Multithreading implícito

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F150/` (a criar).
- **Entradas:** Operações.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-150.1 | Speedup ≥ 3x em 4 núcleos para soma elementar de 1e8 | CA1 |
| TF-150.2 | Resultados idênticos com 1 e N threads (onde garantido) | CA2 |
| TF-150.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-151 — Execução em segundo plano e futures

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F151/` (a criar).
- **Entradas:** Funções.
- **Casos de erro:** Erro propagado ao obter resultado.
- **Casos extremos:** Erro em tarefa; cancelamento.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-151.1 | parfeval no pool de fundo não bloqueia UI | CA1 |
| TF-151.2 | cancel interrompe tarefa | CA2 |
| TF-151.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-152 — Comunicação entre processos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F152/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-152.1 | Protocolo próprio documentado | CA1 |
| TF-152.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-155 — Framework de testes unitários

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F155/` (a criar).
- **Entradas:** Arquivos de teste.
- **Casos de erro:** Teste com erro vs falha distintos.
- **Casos extremos:** Testes que alteram estado global; dependências.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-155.1 | Classe de teste parametrizada gera N casos | CA1 |
| TF-155.2 | Relatório JUnit válido | CA2 |
| TF-155.3 | Filtro por tag | CA3 |
| TF-155.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-156 — Mocks e testes de desempenho

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F156/` (a criar).
- **Entradas:** Interfaces/classes.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-156.1 | Mock registra chamadas | CA1 |
| TF-156.2 | Medição com intervalo de confiança | CA2 |
| TF-156.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-157 — Testes de interface de aplicativos

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F157/` (a criar).
- **Entradas:** Apps.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-157.1 | Teste pressiona botão e verifica rótulo | CA1 |
| TF-157.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-158 — Cobertura de código

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F158/` (a criar).
- **Entradas:** Código, testes.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-158.1 | Relatório Cobertura XML válido | CA1 |
| TF-158.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-159 — Ferramenta de build

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F159/` (a criar).
- **Entradas:** Arquivo de build.
- **Casos de erro:** Tarefa inexistente.
- **Casos extremos:** Ciclos de dependência.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-159.1 | Segunda execução pula tarefas inalteradas | CA1 |
| TF-159.2 | Falha de tarefa retorna código ≠ 0 | CA2 |
| TF-159.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-160 — Automação e diagnóstico de execução

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F160/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-160.1 | Requisito a definir | CA1 |
| TF-160.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-165 — Empacotamento de extensões

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F165/` (a criar).
- **Entradas:** Pasta de projeto.
- **Casos de erro:** Metadados inválidos.
- **Casos extremos:** Conflitos de versão.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-165.1 | Empacotar, instalar, desinstalar | CA1 |
| TF-165.2 | Dependência ausente reportada | CA2 |
| TF-165.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-166 — Gerenciador de pacotes

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F166/` (a criar).
- **Entradas:** Nomes/versões.
- **Casos de erro:** Pacote inexistente.
- **Casos extremos:** Conflitos insolúveis.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-166.1 | Instalar pacote com dependências transitivas | CA1 |
| TF-166.2 | Lockfile reproduz ambiente | CA2 |
| TF-166.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-167 — Explorador de extensões (catálogo online)

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F167/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** Offline.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-167.1 | Catálogo próprio navegável e instalável | CA1 |
| TF-167.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-168 — Verificação de compatibilidade de versões

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F168/` (a criar).
- **Entradas:** Código.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-168.1 | Relatório lista funções obsoletas | CA1 |
| TF-168.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-170 — Instalador interativo e silencioso

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F170/` (a criar).
- **Entradas:** Seleções/arquivo de respostas.
- **Casos de erro:** Assinatura inválida.
- **Casos extremos:** Espaço insuficiente; instalação parcial (rollback).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-170.1 | Instalação silenciosa com arquivo de respostas | CA1 |
| TF-170.2 | Pacote adulterado rejeitado | CA2 |
| TF-170.3 | Remover componente individual | CA3 |
| TF-170.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-171 — Atualizações

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F171/` (a criar).
- **Entradas:** —
- **Casos de erro:** Falha de download.
- **Casos extremos:** Offline.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-171.1 | Atualização assinada aplicada e revertida | CA1 |
| TF-171.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-172 — Desinstalação

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F172/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-172.1 | Desinstalar remove arquivos e atalhos | CA1 |
| TF-172.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-173 — Licenciamento e ativação

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F173/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-173.1 | Aplicação funciona sem ativação (decisão de projeto) | CA1 |
| TF-173.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-174 — Conta de usuário e login

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F174/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-174.1 | Uso offline completo sem conta | CA1 |
| TF-174.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-175 — Telemetria de uso e relatório de falhas

- **Tipos:** Unitário, Integração, Ponta a ponta, Segurança, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F175/` (a criar).
- **Entradas:** Consentimento.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-175.1 | Nenhum tráfego de rede sem consentimento | CA1 |
| TF-175.2 | Relatório de falha salvo localmente | CA2 |
| TF-175.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-180 — Ajuda em linha de comando e navegador de documentação

- **Tipos:** Unitário, Integração, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F180/` (a criar).
- **Entradas:** Consultas.
- **Casos de erro:** —
- **Casos extremos:** Documentação offline instalada como componente.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-180.1 | help f mostra comentário inicial de f | CA1 |
| TF-180.2 | Busca offline retorna resultados em < 300 ms | CA2 |
| TF-180.3 | Exemplo abre e executa | CA3 |
| TF-180.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-181 — Localização da interface

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F181/` (a criar).
- **Entradas:** Configuração de idioma.
- **Casos de erro:** Recurso ausente → fallback inglês.
- **Casos extremos:** Texto mais longo que layout.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-181.1 | Troca de idioma sem reinstalar | CA1 |
| TF-181.2 | Fallback para chave ausente | CA2 |
| TF-181.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-182 — Exemplos e materiais de aprendizado

- **Tipos:** Unitário, Integração, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F182/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-182.1 | Exemplos próprios abrem e executam | CA1 |
| TF-182.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-200 — Editor de diagramas de blocos

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F200/` (a criar).
- **Entradas:** Interações de usuário.
- **Casos de erro:** Conexão de tipos incompatíveis (diagnóstico na atualização).
- **Casos extremos:** Modelos com 1e4 blocos; desfazer extenso.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-200.1 | Modelo com 1000 blocos edita a 60 fps | CA1 |
| TF-200.2 | Desfazer/refazer ilimitado na sessão | CA2 |
| TF-200.3 | Copiar subsistema entre modelos preserva parâmetros | CA3 |
| TF-200.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-201 — Biblioteca de blocos padrão

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F201/` (a criar).
- **Entradas:** Seleção de blocos.
- **Casos de erro:** Parâmetro inválido.
- **Casos extremos:** Parâmetros variáveis no tempo (tunáveis); blocos com estados.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-201.1 | Cada bloco tem teste de referência contra solução analítica/numérica independente | CA1 |
| TF-201.2 | Integrador com reset externo | CA2 |
| TF-201.3 | Lookup N-D com extrapolação configurável | CA3 |
| TF-201.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-202 — Motor de simulação

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F202/` (a criar).
- **Entradas:** Modelo, configuração.
- **Casos de erro:** Passo mínimo violado; laço algébrico insolúvel.
- **Casos extremos:** Sistemas rígidos; descontinuidades; múltiplas taxas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-202.1 | Modelo massa-mola-amortecedor coincide com solução analítica (tol) | CA1 |
| TF-202.2 | Cruzamento por zero de bola quicando com erro < 1e-6 | CA2 |
| TF-202.3 | Sistema multitaxa com transições corretas | CA3 |
| TF-202.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-203 — Atributos de sinal: tipos, dimensões, unidades e barramentos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F203/` (a criar).
- **Entradas:** Definições de tipos/barramentos.
- **Casos de erro:** Incompatibilidade de tipo/dimensão.
- **Casos extremos:** Barramentos aninhados; arrays de barramentos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-203.1 | Propagação de tipos em cadeia | CA1 |
| TF-203.2 | Conversão automática de unidades onde permitido | CA2 |
| TF-203.3 | Editor de barramentos cria/edita definições | CA3 |
| TF-203.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-204 — Dados e parâmetros de modelo

- **Tipos:** Unitário, Integração, Ponta a ponta, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F204/` (a criar).
- **Entradas:** Variáveis/objetos.
- **Casos de erro:** Símbolo não resolvido.
- **Casos extremos:** Símbolos duplicados em fontes diferentes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-204.1 | Parâmetro em dicionário usado por vários modelos | CA1 |
| TF-204.2 | Ajuste de ganho durante simulação reflete imediatamente | CA2 |
| TF-204.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-205 — Subsistemas e execução condicional

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F205/` (a criar).
- **Entradas:** Blocos de subsistema.
- **Casos de erro:** Configuração inconsistente.
- **Casos extremos:** Subsistemas aninhados profundos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-205.1 | Subsistema habilitado mantém/reseta estados conforme opção | CA1 |
| TF-205.2 | For-each processa vetor elemento a elemento | CA2 |
| TF-205.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-206 — Máscaras e blocos personalizados

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F206/` (a criar).
- **Entradas:** Definições de máscara.
- **Casos de erro:** Parâmetro de máscara inválido.
- **Casos extremos:** Bibliotecas modificadas com instâncias abertas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-206.1 | Máscara com parâmetro validado | CA1 |
| TF-206.2 | Atualizar bloco de biblioteca reflete nas instâncias ligadas | CA2 |
| TF-206.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-207 — Referência de modelos e modelos protegidos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F207/` (a criar).
- **Entradas:** Modelos.
- **Casos de erro:** Modelo referenciado ausente.
- **Casos extremos:** Referências circulares; múltiplas instâncias.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-207.1 | Alterar modelo referenciado invalida apenas seu cache | CA1 |
| TF-207.2 | Modelo protegido simula sem expor conteúdo | CA2 |
| TF-207.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-208 — Variantes

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F208/` (a criar).
- **Entradas:** Condições de variante.
- **Casos de erro:** Nenhuma variante ativa.
- **Casos extremos:** Variantes aninhadas.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-208.1 | Troca de variante muda resultado da simulação | CA1 |
| TF-208.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-209 — Blocos definidos pelo usuário

- **Tipos:** Unitário, Integração, Ponta a ponta, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F209/` (a criar).
- **Entradas:** Código do usuário.
- **Casos de erro:** Erro de compilação.
- **Casos extremos:** Código C com falha (isolamento).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-209.1 | Bloco de função compila e simula | CA1 |
| TF-209.2 | Função C integrada via C caller | CA2 |
| TF-209.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-210 — Visualização e registro de sinais

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F210/` (a criar).
- **Entradas:** Sinais.
- **Casos de erro:** —
- **Casos extremos:** Simulações longas (decimação).

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-210.1 | Comparar duas execuções com tolerância relativa | CA1 |
| TF-210.2 | Scope atualiza durante simulação | CA2 |
| TF-210.3 | Knob altera parâmetro em tempo de simulação | CA3 |
| TF-210.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-211 — Entrada de dados e cenários de teste

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F211/` (a criar).
- **Entradas:** Dados de sinal.
- **Casos de erro:** Dimensão incompatível.
- **Casos extremos:** Tempos não monotônicos.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-211.1 | Mapear dataset para portas por nome | CA1 |
| TF-211.2 | Editor cria sinal de degrau | CA2 |
| TF-211.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-212 — Controle programático de simulação

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F212/` (a criar).
- **Entradas:** Objetos de entrada.
- **Casos de erro:** Parâmetro inexistente.
- **Casos extremos:** Erro em uma simulação de um lote.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-212.1 | Lote de 100 simulações paralelas com resultados indexados | CA1 |
| TF-212.2 | Fast restart evita recompilação | CA2 |
| TF-212.3 | Retroceder passos restaura estado | CA3 |
| TF-212.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-213 — Modos de simulação acelerados

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F213/` (a criar).
- **Entradas:** Modelo.
- **Casos de erro:** Compilador ausente.
- **Casos extremos:** Blocos não suportados no modo.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-213.1 | Speedup ≥ 5x em modelo de referência | CA1 |
| TF-213.2 | Resultados equivalentes ao modo normal | CA2 |
| TF-213.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-214 — Depuração e diagnóstico de modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Desempenho, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F214/` (a criar).
- **Entradas:** Modelo.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-214.1 | Ponto de parada condicional em sinal | CA1 |
| TF-214.2 | Legenda de tempos de amostragem | CA2 |
| TF-214.3 | Diagnóstico clicável destaca bloco | CA3 |
| TF-214.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-215 — Configuração de modelo

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F215/` (a criar).
- **Entradas:** Parâmetros.
- **Casos de erro:** Valor inválido.
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-215.1 | Configuração referenciada compartilhada entre 2 modelos | CA1 |
| TF-215.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-216 — Formato de arquivo de modelo

- **Tipos:** Unitário, Integração, Ponta a ponta, Compatibilidade/E-S, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F216/` (a criar).
- **Entradas:** Modelos.
- **Casos de erro:** Arquivo corrompido.
- **Casos extremos:** Arquivos grandes; autosave após falha.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-216.1 | Salvar/abrir ida-e-volta sem perda | CA1 |
| TF-216.2 | Autosave recupera após encerramento forçado | CA2 |
| TF-216.3 | Importador experimental de formato de terceiros (se aprovado) | CA3 |
| TF-216.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-217 — Comparação e mesclagem de modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F217/` (a criar).
- **Entradas:** Dois/três modelos.
- **Casos de erro:** —
- **Casos extremos:** Modelos grandes.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-217.1 | Diff identifica bloco com parâmetro alterado | CA1 |
| TF-217.2 | Merge 3-vias de alterações não conflitantes | CA2 |
| TF-217.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-218 — API programática de construção de modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F218/` (a criar).
- **Entradas:** Comandos.
- **Casos de erro:** Caminho de bloco inválido.
- **Casos extremos:** Callback com erro.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-218.1 | Script constrói modelo de 3 blocos e simula | CA1 |
| TF-218.2 | Callback InitFcn executa antes da compilação | CA2 |
| TF-218.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-219 — Funções e mensagens em modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F219/` (a criar).
- **Entradas:** Blocos.
- **Casos de erro:** —
- **Casos extremos:** Fila cheia.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-219.1 | Chamada de função entre subsistemas | CA1 |
| TF-219.2 | Fila de mensagens FIFO com capacidade | CA2 |
| TF-219.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-220 — Linearização e equilíbrio básicos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F220/` (a criar).
- **Entradas:** Modelo, ponto.
- **Casos de erro:** Não convergência.
- **Casos extremos:** Modelos com descontinuidades.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-220.1 | Linearização de modelo linear reproduz matrizes | CA1 |
| TF-220.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-221 — Co-simulação e FMI

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F221/` (a criar).
- **Entradas:** Arquivos FMU.
- **Casos de erro:** FMU inválida.
- **Casos extremos:** FMU binária de plataforma diferente.

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-221.1 | Importar FMU de referência (FMI Cross-Check) e simular | CA1 |
| TF-221.2 | Exportar FMU validada por ferramenta de terceiros | CA2 |
| TF-221.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-222 — Particionamento, tarefas e agendamento

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F222/` (a criar).
- **Entradas:** Modelo.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-222.1 | Ordem de partições editável afeta execução | CA1 |
| TF-222.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-223 — Busca e navegação em modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F223/` (a criar).
- **Entradas:** Consultas.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-223.1 | Busca por valor de parâmetro em hierarquia | CA1 |
| TF-223.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-224 — Modelos de partida e página inicial

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F224/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-224.1 | Criar modelo a partir de modelo-base próprio | CA1 |
| TF-224.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-225 — Documentação de sistemas em modelos

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F225/` (a criar).
- **Entradas:** Texto.
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-225.1 | Nota persiste com o modelo | CA1 |
| TF-225.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-226 — Integração com projetos e controle de versão

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F226/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-226.1 | Recarregar modelo após checkout | CA1 |
| TF-226.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

## TF-227 — Simulação em segundo plano e cache

- **Tipos:** Unitário, Integração, Ponta a ponta, Regressão
- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/F227/` (a criar).
- **Entradas:** —
- **Casos de erro:** —
- **Casos extremos:** —

| Caso | Procedimento / resultado esperado | Critério |
|---|---|---|
| TF-227.1 | Simulação longa não bloqueia edição | CA1 |
| TF-227.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |

