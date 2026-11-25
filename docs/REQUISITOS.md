<!-- GERADO por ferramentas/gerar_docs.py. Não edite à mão: edite docs/dados/funcionalidades.json. -->

# Requisitos

Requisitos redigidos de forma independente a partir das necessidades funcionais. Cada RF-xxx corresponde à funcionalidade Fxxx.

## Requisitos não funcionais (RNF)

| ID | Requisito | Verificação |
|---|---|---|
| RNF-01 | Plataformas: Linux x86-64 e aarch64, Windows x64, macOS arm64 e x86-64. | CI de matriz por plataforma |
| RNF-02 | Independência: nenhum código, binário, texto, ícone ou recurso de produto proprietário no repositório. | Auditoria de proveniência a cada release (ver CLAUDE.md) |
| RNF-03 | Licenças: só dependências aprovadas em LICENCAS_E_RESTRICOES.txt; SBOM gerado a cada build. | cargo-deny / license-checker no CI |
| RNF-04 | Precisão numérica IEEE 754; desvios documentados por função. | Suítes de referência numérica |
| RNF-05 | Responsividade: UI não bloqueia durante execução de código (runtime em processo separado). | Teste E2E com laço infinito + Ctrl+C |
| RNF-06 | Robustez: falha em extensão nativa não corrompe a sessão (modo isolado disponível). | Teste de injeção de falha |
| RNF-07 | Privacidade: zero tráfego de rede sem ação explícita ou consentimento; telemetria opt-in. | Teste de rede em sandbox |
| RNF-08 | Segurança: parsers de arquivos resistentes a entradas malformadas (fuzzing); XML sem entidades externas; TLS verificado por padrão. | cargo-fuzz em todos os parsers |
| RNF-09 | Acessibilidade: UI navegável por teclado, compatível com leitores de tela, contraste WCAG 2.2 AA. | Auditoria axe-core + manual |
| RNF-10 | Localização: pt-BR e en na v1; textos externalizados. | Teste de chaves ausentes |
| RNF-11 | Desempenho: metas por funcionalidade nos critérios de aceitação; benchmarks contínuos sem regressão > 5%. | Benchmarks no CI |
| RNF-12 | Formatos próprios abertos, versionados e documentados. | Especificações em docs/formatos/ |
| RNF-13 | Testabilidade: cobertura de linhas ≥ 85% no núcleo (lang/runtime/math). | Relatório de cobertura |
| RNF-14 | Modo sem interface (headless) para todos os recursos não visuais. | Execução da suíte em CI sem display |

## Requisitos funcionais (RF)

### RF-001 — Interpretador interativo (janela de comandos) (origem: F001, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Avaliação interativa de expressões e comandos com exibição automática de resultados.

- **Entradas:** Texto de comando; teclas de controle (histórico, interrupção).
- **Saídas:** Valores exibidos; variável implícita de resultado; mensagens de erro/aviso.
- **Comportamento:** Cada linha é analisada e executada; ponto-e-vírgula suprime exibição; resultado sem atribuição vai para variável implícita 'ans'.
- **Regras:** Exibição respeita formato numérico ativo; vírgula separa comandos exibindo; Ctrl+C interrompe execução.
- **Erros a tratar:** Erro de sintaxe com posição; identificador indefinido; interrupção pelo usuário.
- **Casos especiais:** Comandos multilinha (blocos if/for abertos); sintaxe de comando ('hold on'); entrada muito longa.
- **Critérios de aceitação:**
  - CA1: Expressão '1+1' exibe 2 e define ans
  - CA2: ';' suprime exibição
  - CA3: Bloco multilinha só executa após 'end'
  - CA4: Ctrl+C interrompe laço infinito em <1s sem perder o workspace

### RF-002 — Scripts e funções em arquivo (origem: F002, status PLANEJADA)

O sistema deve oferecer: Arquivos de código-fonte contendo scripts ou funções (principal, locais, aninhadas, privadas).

- **Entradas:** Arquivos de código (.m equivalente no formato compatível e extensão própria).
- **Saídas:** Execução; valores de retorno múltiplos.
- **Comportamento:** Funções com múltiplas entradas/saídas, nargin/nargout, varargin/varargout, funções locais, aninhadas com compartilhamento de escopo, scripts operando no workspace do chamador.
- **Regras:** Nome do arquivo define a função principal; funções locais visíveis apenas no arquivo; precedência de resolução (variável > aninhada > local > privada > classe > caminho).
- **Erros a tratar:** Muitos/poucos argumentos; saída não atribuída; recursão máxima excedida.
- **Casos especiais:** Recursão profunda; arquivo modificado em disco durante a sessão (recarregamento); script com função local no final.
- **Critérios de aceitação:**
  - CA1: Função com 3 saídas retorna somente as pedidas
  - CA2: Edição do arquivo em disco é refletida na chamada seguinte
  - CA3: Limite de recursão gera erro identificável e configurável

### RF-003 — Tipos numéricos e lógicos fundamentais (origem: F003, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Ponto flutuante dupla/simples, inteiros com/sem sinal de 8–64 bits, lógico, caractere e números complexos.

- **Entradas:** Literais, conversões de tipo.
- **Saídas:** Arrays tipados.
- **Comportamento:** Aritmética mista com regras de tipo dominante; inteiros saturam e arredondam; complexo automático em operações que exigem (sqrt(-1)).
- **Regras:** Inteiro combinado com double resulta inteiro; operações entre inteiros de classes diferentes são erro; NaN/Inf seguem IEEE 754.
- **Erros a tratar:** Combinação inválida de classes; overflow tratado por saturação (sem erro).
- **Casos especiais:** -0, NaN payload, intmin/intmax, eps, realmin denormais, conversão de NaN para inteiro (0).
- **Critérios de aceitação:**
  - CA1: int8(127)+1 == 127
  - CA2: uint8(3.5) == 4
  - CA3: class(int16(1)*2.7) == 'int16'
  - CA4: sqrt(-4) == 2i

### RF-004 — Arrays N-dimensionais e indexação (origem: F004, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Arrays de qualquer dimensão com indexação por subscrito, linear, lógica, 'end', dois-pontos, exclusão e crescimento automático.

- **Entradas:** Arrays, índices.
- **Saídas:** Subarrays, arrays modificados.
- **Comportamento:** Ordem column-major; atribuição fora dos limites expande com zeros; atribuição de [] remove elementos; dimensões singleton finais ignoradas.
- **Regras:** Índices começam em 1; índice lógico deve ter tamanho compatível; 'end' resolve para o tamanho da dimensão indexada.
- **Erros a tratar:** Índice fora dos limites na leitura; índice não inteiro/negativo; dimensões incompatíveis na atribuição.
- **Casos especiais:** Arrays vazios com dimensões (0x3); indexação com mais dimensões que o array; colapso de dimensões finais.
- **Critérios de aceitação:**
  - CA1: A(end+1)=x cresce o vetor
  - CA2: A(:) retorna coluna
  - CA3: Remover linha com A(2,:)=[]
  - CA4: size(zeros(0,3)) == [0 3]

### RF-005 — Operadores matriciais, elemento a elemento e expansão implícita (origem: F005, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Operadores aritméticos, relacionais, lógicos (com curto-circuito), matriciais (*, /, \, ^) e elemento a elemento (.*, ./, .^) com expansão implícita de dimensões singleton.

- **Entradas:** Operandos arrays.
- **Saídas:** Arrays resultantes.
- **Comportamento:** Expansão implícita compatível; '\' resolve sistema linear escolhendo método por estrutura da matriz; transposta e transposta conjugada.
- **Regras:** Precedência de operadores documentada; && e || exigem escalares lógicos.
- **Erros a tratar:** Dimensões incompatíveis; operandos não escalares em &&.
- **Casos especiais:** Expansão entre 1xN e Mx1; operações com vazio; matriz singular em '\' gera aviso.
- **Critérios de aceitação:**
  - CA1: [1 2 3]+[10;20] produz 2x3
  - CA2: A\b resolve sistema com resíduo < 1e-12 em matriz bem condicionada
  - CA3: Aviso emitido para matriz singular

### RF-006 — Controle de fluxo (origem: F006, status PLANEJADA)

O sistema deve oferecer: if/elseif/else, switch/case (inclusive cell de casos e strings), for (sobre colunas), while, break, continue, return, try/catch.

- **Entradas:** Código.
- **Saídas:** Execução condicionada.
- **Comportamento:** for itera sobre colunas da expressão; switch compara números e textos; try/catch captura exceção em objeto.
- **Regras:** Condição de if com array é verdadeira somente se todos os elementos forem não-nulos e não-vazia.
- **Erros a tratar:** Uso de break fora de laço; catch sem identificador.
- **Casos especiais:** for sobre matriz vazia; switch com cell de alternativas; modificação da variável do laço no corpo.
- **Critérios de aceitação:**
  - CA1: for k=[1 2;3 4] itera 2 vezes com colunas
  - CA2: if [] é falso
  - CA3: switch 'abc' casa com {'x','abc'}

### RF-007 — Programação orientada a objetos (origem: F007, status PLANEJADA)

O sistema deve oferecer: Definição de classes com propriedades, métodos, eventos, enumerações, herança múltipla, classes de valor e de referência, atributos de acesso, validação de propriedades, métodos estáticos, sobrecarga de operadores e indexação, getters/setters, metaclasses.

- **Entradas:** Arquivos de definição de classe; pastas de classe.
- **Saídas:** Objetos, metadados de classe.
- **Comportamento:** Semântica de cópia para classes de valor e referência compartilhada para classes de identidade; destrutores em classes de referência; listeners de eventos e propriedades observáveis.
- **Regras:** Atributos Access/SetAccess/GetAccess/Hidden/Constant/Dependent/Abstract/Sealed; ordem de resolução de métodos; classes enumeradas.
- **Erros a tratar:** Acesso negado; violação de validação; método abstrato não implementado.
- **Casos especiais:** Recarregar definição com instâncias existentes; herança diamante; redefinição de indexação (subsref/subsasgn equivalente).
- **Critérios de aceitação:**
  - CA1: Classe de referência compartilha estado entre cópias
  - CA2: Validação de propriedade rejeita tipo inválido com mensagem identificável
  - CA3: Evento notifica listeners em ordem de registro
  - CA4: Metadados da classe consultáveis

### RF-008 — Validação de argumentos de função (origem: F008, status PLANEJADA)

O sistema deve oferecer: Bloco declarativo de argumentos com tamanho, classe, funções validadoras, valores padrão, argumentos nome-valor e repetidos.

- **Entradas:** Declaração de argumentos.
- **Saídas:** Argumentos convertidos/validados.
- **Comportamento:** Conversão automática quando possível; erros apontam o argumento; suporte a nome-valor com nomes parciais desabilitados/habilitados conforme regra.
- **Regras:** Validação ocorre antes do corpo; padrões avaliados em ordem.
- **Erros a tratar:** Tipo/tamanho incompatível; nome-valor desconhecido.
- **Casos especiais:** Argumentos nome-valor derivados de propriedades de classe; validadores personalizados.
- **Critérios de aceitação:**
  - CA1: Chamada f(x='a') com validador numérico falha antes do corpo
  - CA2: Valor padrão aplicado quando omitido
  - CA3: Mensagem identifica posição do argumento

### RF-009 — Arrays de células e estruturas (origem: F009, status PLANEJADA)

O sistema deve oferecer: Containers heterogêneos (cell) e registros com campos nomeados (struct), incluindo arrays de structs, campos dinâmicos, listas separadas por vírgula.

- **Entradas:** Valores quaisquer.
- **Saídas:** Containers.
- **Comportamento:** c{...} extrai conteúdo, c(...) extrai subcélula; s.(nome) acessa campo dinâmico; s(i).campo em arrays de struct produz lista separada por vírgula.
- **Regras:** Arrays de struct exigem os mesmos campos; ordem de campos preservada.
- **Erros a tratar:** Campo inexistente; conteúdo de cell com chaves em contexto inválido.
- **Casos especiais:** Struct vazio com campos; cell aninhada profunda; deal com listas.
- **Critérios de aceitação:**
  - CA1: [s.a] concatena campos de array de struct
  - CA2: fieldnames preserva ordem
  - CA3: c{2:3} expande em múltiplas saídas

### RF-010 — Texto: arrays de caracteres e strings (origem: F010, status PLANEJADA)

O sistema deve oferecer: Tipo char e tipo string com funções de manipulação (concatenação, busca, substituição, divisão, junção, formatação, comparação, conversão numérica), padrões de texto e expressões regulares.

- **Entradas:** Textos, padrões.
- **Saídas:** Textos, índices, tokens.
- **Comportamento:** Strings são escalares de texto com operador +; funções aceitam char, string e cellstr; regexp com tokens nomeados, lookaround, modos.
- **Regras:** Unicode completo; comparação sensível a maiúsculas por padrão.
- **Erros a tratar:** Padrão inválido; conversão numérica inválida resulta NaN.
- **Casos especiais:** String ausente (<missing>); texto vazio vs ausente; expressões regulares com retrocesso exponencial.
- **Critérios de aceitação:**
  - CA1: "a"+"b" == "ab"
  - CA2: regexp com tokens nomeados retorna struct
  - CA3: str2double('x') é NaN
  - CA4: split/join/replace funcionam com arrays de string

### RF-011 — Formatação e exibição de valores (origem: F011, status PLANEJADA)

O sistema deve oferecer: Formatos de exibição (curto, longo, engenharia, racional, hexadecimal, espaçamento), sprintf/fprintf/num2str/disp/display, exibição personalizada de objetos.

- **Entradas:** Valores, especificadores de formato.
- **Saídas:** Texto formatado.
- **Comportamento:** Especificadores estilo C reciclados sobre elementos; exibição de matrizes com fator comum de escala.
- **Regras:** Formato é estado de sessão e preferência.
- **Erros a tratar:** Especificador inválido.
- **Casos especiais:** Inteiros grandes, complexos, NaN/Inf, arrays vazios, exibição truncada de arrays grandes.
- **Critérios de aceitação:**
  - CA1: sprintf('%d,',[1 2 3]) == '1,2,3,'
  - CA2: format long mostra 15 dígitos significativos para double
  - CA3: Exibição personalizável por método da classe

### RF-012 — Tratamento de erros e avisos (origem: F012, status PLANEJADA)

O sistema deve oferecer: Lançamento e captura de exceções com identificadores, pilha, causas, correções sugeridas; controle de avisos por identificador; erro/aviso com formatação; limpeza garantida (onCleanup).

- **Entradas:** Identificador, mensagem, estado.
- **Saídas:** Exceções; avisos exibidos/suprimidos.
- **Comportamento:** Avisos podem ser ligados/desligados/convertidos por id; último erro/aviso consultável; rethrow preserva pilha.
- **Regras:** Identificadores 'componente:mensagem'.
- **Erros a tratar:** Identificador mal formado.
- **Casos especiais:** Erro durante limpeza; avisos repetidos; exceções aninhadas (causes).
- **Critérios de aceitação:**
  - CA1: warning('off',id) suprime apenas aquele aviso
  - CA2: MException tem stack com arquivo/linha
  - CA3: onCleanup executa ao sair do escopo mesmo em erro

### RF-013 — Workspaces, escopo e avaliação dinâmica (origem: F013, status PLANEJADA)

O sistema deve oferecer: Workspace base e de função, variáveis globais e persistentes, eval/evalin/assignin/feval, exist/which/clear/who/whos, inputname.

- **Entradas:** Nomes, expressões em texto.
- **Saídas:** Valores, informação de variáveis.
- **Comportamento:** Variáveis persistentes por função; globais compartilhadas por declaração; clear remove variáveis/funções/classes do cache.
- **Regras:** exist retorna códigos de tipo; which resolve caminho e sobreposição.
- **Erros a tratar:** Nome inválido; workspace inexistente.
- **Casos especiais:** eval com erro e catch; clear dentro de função; variáveis com nomes de funções (shadowing).
- **Critérios de aceitação:**
  - CA1: Persistente mantém valor entre chamadas
  - CA2: clear all limpa persistentes
  - CA3: whos lista tamanho, classe e bytes

### RF-014 — Caminho de busca, pacotes e pastas especiais (origem: F014, status PLANEJADA)

O sistema deve oferecer: Caminho de busca de funções, pastas privadas, pastas de classe, namespaces (pastas de pacote), importação de nomes, pastas de recursos, startup/finish de sessão.

- **Entradas:** Diretórios, declarações import.
- **Saídas:** Resolução de nomes.
- **Comportamento:** Ordem de precedência configurável; caminho persistível; namespaces aninhados; import de pacote ou função específica.
- **Regras:** Pasta atual precede caminho; pastas privadas visíveis só ao pai.
- **Erros a tratar:** Pasta inexistente; nome ambíguo.
- **Casos especiais:** Duplicatas de nomes (shadowing com aviso); pasta removida em uso.
- **Critérios de aceitação:**
  - CA1: addpath/rmpath/savepath alteram e persistem caminho
  - CA2: Função em pasta privada não visível externamente
  - CA3: import de namespace resolve nome curto

### RF-015 — Módulos e namespaces modernos (origem: F015, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Agrupamento de código em módulos com descritor, visibilidade e dependências.

- **Entradas:** Pastas de módulo com descritor.
- **Saídas:** Escopos de visibilidade.
- **Comportamento:** Inferência: módulo expõe API pública e oculta implementação.
- **Regras:** A investigar em documentação pública.
- **Erros a tratar:** Dependência circular.
- **Casos especiais:** Compatibilidade com caminho tradicional.
- **Critérios de aceitação:**
  - CA1: Função interna do módulo inacessível de fora
  - CA2: Descritor inválido gera erro claro

### RF-016 — Funções anônimas, handles e funções de ordem superior (origem: F016, status PLANEJADA)

O sistema deve oferecer: Handles de função, funções anônimas com captura de valores, cellfun/arrayfun/structfun/func2str/str2func.

- **Entradas:** Expressões, handles.
- **Saídas:** Handles, resultados agregados.
- **Comportamento:** Anônimas capturam valores no momento da criação; 'UniformOutput' controla agregação; 'ErrorHandler' trata erros por elemento.
- **Regras:** Saídas não uniformes exigem UniformOutput=false.
- **Erros a tratar:** Saída não escalar com UniformOutput=true.
- **Casos especiais:** Handles para funções aninhadas; comparação de handles.
- **Critérios de aceitação:**
  - CA1: Captura por valor: alterar variável depois não muda o handle
  - CA2: arrayfun com saída não escalar exige opção

### RF-017 — Eventos, listeners e referências fracas (origem: F017, status PLANEJADA)

O sistema deve oferecer: Mecanismo de publicação/assinatura entre objetos, propriedades observáveis, referências fracas.

- **Entradas:** Objetos, nomes de evento, callbacks.
- **Saídas:** Notificações.
- **Comportamento:** Listener removido ao destruir; ordem estável.
- **Regras:** Eventos declarados na classe.
- **Erros a tratar:** Evento inexistente.
- **Casos especiais:** Listener que destrói a fonte; recursão de eventos.
- **Critérios de aceitação:**
  - CA1: notify dispara todos os listeners
  - CA2: Listener desabilitado não executa

### RF-018 — Interrupção, pausa e temporizadores (origem: F018, status PLANEJADA)

O sistema deve oferecer: Interrupção de execução, pause com/sem tempo, input/keyboard, objetos de temporizador com modos de execução (único, periódico, espaçamento fixo) e fila.

- **Entradas:** Tempo, callbacks.
- **Saídas:** Execução diferida.
- **Comportamento:** Temporizadores rodam no laço de eventos do runtime; callbacks não interrompem código em execução no meio de instrução.
- **Regras:** Período mínimo de 1 ms; modos de execução distintos.
- **Erros a tratar:** Período inválido.
- **Casos especiais:** Callback mais longo que o período; erro no callback.
- **Critérios de aceitação:**
  - CA1: Timer periódico executa N vezes com desvio <10ms
  - CA2: Erro no callback chama ErrorFcn

### RF-019 — Desempenho de execução (compilação JIT) (origem: F019, status PLANEJADA)

O sistema deve oferecer: Execução acelerada de código interpretado (compilação just-in-time) transparente ao usuário.

- **Entradas:** Código.
- **Saídas:** Mesma semântica, menor tempo.
- **Comportamento:** Motor de execução com otimização; a meta é de desempenho, sem técnica prescrita.
- **Regras:** Semântica idêntica ao modo interpretado.
- **Erros a tratar:** Nenhum visível ao usuário.
- **Casos especiais:** Código dinâmico (eval) não compilável — fallback.
- **Critérios de aceitação:**
  - CA1: Laço escalar de 1e7 iterações < 5x de C equivalente (meta)
  - CA2: Resultados idênticos entre JIT e interpretador em toda a suíte

### RF-020 — Operações de conjuntos, ordenação, busca e bits (origem: F020, status PLANEJADA)

O sistema deve oferecer: sort/sortrows/unique/union/intersect/setdiff/ismember/find/any/all/cumsum/bitand/bitor/bitshift etc.

- **Entradas:** Arrays.
- **Saídas:** Arrays e índices.
- **Comportamento:** Estabilidade de ordenação; ordem de saída 'sorted'/'stable'; NaN tratados como maiores.
- **Regras:** Saídas de índice opcionais.
- **Erros a tratar:** Tipos incompatíveis.
- **Casos especiais:** Tipos mistos; strings; NaN em unique (cada NaN distinto).
- **Critérios de aceitação:**
  - CA1: unique([3 1 NaN NaN]) mantém 2 NaN
  - CA2: sort estável
  - CA3: ismember retorna índices

### RF-021 — Datas e horas (datetime) (origem: F021, status PLANEJADA)

O sistema deve oferecer: Pontos no tempo com fuso horário, formatos de exibição, aritmética, componentes, conversão de/para texto e números.

- **Entradas:** Textos, números, componentes.
- **Saídas:** Arrays de datetime.
- **Comportamento:** Fusos IANA e horário de verão; formatos baseados em padrões Unicode; NaT para ausente.
- **Regras:** Comparação entre zonas diferentes converte para UTC.
- **Erros a tratar:** Formato inválido; zona desconhecida.
- **Casos especiais:** Horário ambíguo/inexistente na transição DST; segundos bissextos (opcional 'UTCLeapSeconds').
- **Critérios de aceitação:**
  - CA1: Soma de 1 dia atravessando DST mantém hora local
  - CA2: Leitura de texto com formato explícito
  - CA3: NaT propagado em operações

### RF-022 — Durações e durações de calendário (origem: F022, status PLANEJADA)

O sistema deve oferecer: Intervalos fixos (duration) e de calendário (meses, anos variáveis).

- **Entradas:** Números, textos.
- **Saídas:** Arrays de duração.
- **Comportamento:** Duração de calendário depende da data base; formatação dd:hh:mm:ss.
- **Regras:** Soma de calendarDuration a datetime respeita fim de mês.
- **Erros a tratar:** Formato inválido.
- **Casos especiais:** 31-jan + 1 mês; durações negativas.
- **Critérios de aceitação:**
  - CA1: datetime(2024,1,31)+calmonths(1) == 29-fev-2024
  - CA2: duração exibida no formato escolhido

### RF-023 — Dados categóricos (origem: F023, status PLANEJADA)

O sistema deve oferecer: Arrays de categorias com conjunto de categorias, ordenação opcional, contagem, mesclagem, renomeação.

- **Entradas:** Textos/números.
- **Saídas:** Arrays categóricos.
- **Comportamento:** Categorias ordinais permitem comparação relacional; <undefined> para ausentes.
- **Regras:** Categorias protegidas impedem novos valores.
- **Erros a tratar:** Valor fora das categorias protegidas.
- **Casos especiais:** Combinação de categóricos com conjuntos diferentes.
- **Critérios de aceitação:**
  - CA1: countcats correto
  - CA2: Comparação ordinal
  - CA3: Ausente exibido como <undefined>

### RF-024 — Tabelas e tabelas temporais (origem: F024, status PLANEJADA)

O sistema deve oferecer: Dados tabulares heterogêneos com nomes de variáveis/linhas, metadados, junções, agrupamento/agregação, empilhamento, sincronização e retemporização de séries temporais.

- **Entradas:** Arrays, arquivos.
- **Saídas:** Tabelas.
- **Comportamento:** Indexação por nome e posição; grouping (groupsummary, rowfun, varfun); joins (inner/outer); retime/synchronize com métodos de agregação/interpolação.
- **Regras:** Variáveis de mesma altura; nomes únicos válidos.
- **Erros a tratar:** Nome duplicado; altura incompatível.
- **Casos especiais:** Variáveis multicoluna; tabelas aninhadas; tempos duplicados/não ordenados.
- **Critérios de aceitação:**
  - CA1: join interno por chave
  - CA2: retime para dados horários com média
  - CA3: groupsummary por categoria
  - CA4: T.Var acesso por nome

### RF-025 — Dicionários e mapas (origem: F025, status PLANEJADA)

O sistema deve oferecer: Containers chave-valor tipados (dictionary) e mapas legados (containers.Map).

- **Entradas:** Chaves e valores.
- **Saídas:** Container.
- **Comportamento:** Chaves tipadas com hash; valores ausentes geram erro ou padrão; Map de referência vs dicionário de valor.
- **Regras:** Tipo de chave fixo após configuração.
- **Erros a tratar:** Chave inexistente.
- **Casos especiais:** Chaves NaN; chaves string vs char.
- **Critérios de aceitação:**
  - CA1: Inserção/busca O(1) amortizado
  - CA2: Semântica de valor do dicionário vs referência do mapa

### RF-026 — Valores ausentes padronizados (origem: F026, status PLANEJADA)

O sistema deve oferecer: Valor 'ausente' genérico conversível em NaN/NaT/<missing>/<undefined>; funções ismissing/fillmissing/rmmissing/standardizeMissing.

- **Entradas:** Arrays/tabelas.
- **Saídas:** Arrays sem ausentes ou indicadores.
- **Comportamento:** fillmissing com métodos constante/anterior/próximo/linear/spline/movmean.
- **Regras:** Cada tipo tem representação própria de ausente.
- **Erros a tratar:** Método inválido.
- **Casos especiais:** Tabelas com tipos mistos.
- **Critérios de aceitação:**
  - CA1: fillmissing linear em vetor com NaN
  - CA2: rmmissing remove linhas de tabela com ausentes

### RF-040 — Álgebra linear densa (origem: F040, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Solução de sistemas, inversa, determinante, decomposições LU/QR/Cholesky/LDL/Schur/Hessenberg/SVD/autovalores (padrão e generalizado), funções de matriz (expm, logm, sqrtm, funm), normas, condição, posto, pseudo-inversa, espaços nulo/coluna.

- **Entradas:** Matrizes.
- **Saídas:** Fatores, soluções, escalares.
- **Comportamento:** Escolha automática de algoritmo por estrutura (triangular, simétrica, banda); avisos de mal condicionamento.
- **Regras:** Resultados conforme precisão IEEE; ordem dos autovalores não garantida salvo opção.
- **Erros a tratar:** Matriz não quadrada onde exigido; NaN/Inf.
- **Casos especiais:** Matrizes vazias, singulares, complexas, grandes (multithread).
- **Critérios de aceitação:**
  - CA1: Resíduo relativo < 1e-12 em suíte de matrizes de teste
  - CA2: svd de matriz 1000x1000 em tempo comparável a LAPACK otimizado
  - CA3: Aviso de singularidade

### RF-041 — Matrizes esparsas e solvers iterativos (origem: F041, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Armazenamento esparso, operações, ordenações de redução de preenchimento, fatorações esparsas, solvers iterativos (pcg, gmres, bicgstab, minres, lsqr, ...), autovalores/valores singulares parciais, precondicionadores incompletos.

- **Entradas:** Matrizes esparsas.
- **Saídas:** Soluções, fatores.
- **Comportamento:** '\' esparso escolhe solver por estrutura; ordenações AMD/COLAMD; ILU/IC.
- **Regras:** Esparsidade preservada em operações quando possível.
- **Erros a tratar:** Não convergência (flag + aviso).
- **Casos especiais:** Matriz com zero explícito; muito grande; mal condicionada.
- **Critérios de aceitação:**
  - CA1: pcg converge em matriz de Poisson 2D
  - CA2: eigs retorna k maiores autovalores
  - CA3: Memória proporcional a nnz

### RF-042 — Funções elementares e especiais (origem: F042, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Trigonométricas (rad/graus), exponenciais, logaritmos, arredondamento, módulo, gamma/beta/erf/expint/Bessel/Airy/elípticas/Legendre, primos, fatoração, mdc/mmc, combinatória.

- **Entradas:** Arrays numéricos.
- **Saídas:** Arrays.
- **Comportamento:** Saídas complexas quando domínio exige; precisão próxima a 1 ulp.
- **Regras:** Funções de grau (sind) exatas em múltiplos de 90.
- **Erros a tratar:** Domínio inválido gera NaN/complexo conforme regra.
- **Casos especiais:** Argumentos extremos; complexos; inteiros grandes.
- **Critérios de aceitação:**
  - CA1: sind(180)==0 exato
  - CA2: gamma em teste de referência com erro relativo < 1e-14
  - CA3: besselj ordens fracionárias

### RF-043 — Estatística descritiva e pré-processamento de dados (origem: F043, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Média, mediana, moda, desvio, variância, quantis, histogramas/contagens, somas e produtos cumulativos, estatísticas móveis, normalização, reescala, detecção/remoção/preenchimento de outliers, suavização, detecção de mudanças e extremos locais.

- **Entradas:** Arrays/tabelas; dimensão; flags NaN.
- **Saídas:** Estatísticas.
- **Comportamento:** Opções 'omitnan'/'includenan', 'all', vecdim; janelas móveis com tratamento de bordas.
- **Regras:** Dimensão padrão é a primeira não singleton.
- **Erros a tratar:** Opção inválida.
- **Casos especiais:** Entrada vazia (mean de vazio é NaN); tipos inteiros.
- **Critérios de aceitação:**
  - CA1: mean(x,'omitnan') ignora NaN
  - CA2: movmean com janela par/ímpar
  - CA3: rmoutliers método mediana

### RF-044 — Interpolação e ajuste de curvas básico (origem: F044, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Interpolação 1-D/2-D/3-D/N-D (linear, nearest, cúbica, spline, pchip, makima), interpolantes em grade e dispersos, splines, extrapolação.

- **Entradas:** Pontos, valores, consultas.
- **Saídas:** Valores interpolados, objetos interpolantes.
- **Comportamento:** Objetos interpolantes reutilizáveis; métodos de extrapolação configuráveis.
- **Regras:** Pontos de grade monotônicos.
- **Erros a tratar:** Grade não monotônica.
- **Casos especiais:** Pontos duplicados; consulta fora do domínio (NaN padrão).
- **Critérios de aceitação:**
  - CA1: interp1 spline reproduz polinômio cúbico
  - CA2: scattered interpolant com Delaunay

### RF-045 — Polinômios (origem: F045, status IMPLEMENTADA)

O sistema deve oferecer: Avaliação, raízes, ajuste por mínimos quadrados, convolução/deconvolução, derivação/integração, frações parciais.

- **Entradas:** Coeficientes.
- **Saídas:** Coeficientes, raízes.
- **Comportamento:** Coeficientes em ordem decrescente; polyfit com escalonamento opcional (S, mu).
- **Regras:** roots via autovalores da companheira.
- **Erros a tratar:** Dados insuficientes.
- **Casos especiais:** Grau alto mal condicionado (aviso).
- **Critérios de aceitação:**
  - CA1: roots([1 -3 2]) == [2;1]
  - CA2: polyfit grau n em n+1 pontos interpola

### RF-046 — Transformadas de Fourier e filtragem básica (origem: F046, status EM DESENVOLVIMENTO)

O sistema deve oferecer: FFT/IFFT 1-D/2-D/N-D com qualquer tamanho, deslocamento, filtragem IIR/FIR direta (filter), convolução 1-D/2-D/N-D, correlação básica.

- **Entradas:** Arrays; tamanho; dimensão.
- **Saídas:** Espectros, sinais filtrados.
- **Comportamento:** Tamanhos arbitrários (não só potências de 2); 'symmetric' em ifft; planejamento de desempenho opcional.
- **Regras:** Normalização sem fator na FFT direta.
- **Erros a tratar:** Dimensão inválida.
- **Casos especiais:** Tamanho primo grande; entradas reais vs complexas.
- **Critérios de aceitação:**
  - CA1: ifft(fft(x)) ≈ x (1e-12)
  - CA2: fft de tamanho primo 10007 correta
  - CA3: filter equivale a equação de diferenças

### RF-047 — Geometria computacional (origem: F047, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Triangulação de Delaunay 2-D/3-D, fecho convexo, diagramas de Voronoi, objetos de triangulação, polígonos (polyshape: booleanas, área, buffer), alpha shapes, ponto-em-polígono.

- **Entradas:** Pontos, polígonos.
- **Saídas:** Triangulações, polígonos.
- **Comportamento:** Consultas de vizinhança e localização; operações booleanas robustas.
- **Regras:** Pontos colineares/coplanares tratados com aviso.
- **Erros a tratar:** Pontos insuficientes.
- **Casos especiais:** Pontos duplicados; precisão finita.
- **Critérios de aceitação:**
  - CA1: delaunay de 1e5 pontos aleatórios satisfaz critério de círculo vazio
  - CA2: polyshape union/intersect com área correta

### RF-048 — Integração numérica e quadratura (origem: F048, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Integral adaptativa 1-D/2-D/3-D (incl. limites infinitos e singularidades), regra trapezoidal, cumulativa, Gauss-Kronrod.

- **Entradas:** Funções/handles, limites, tolerâncias.
- **Saídas:** Valores integrais, estimativas de erro.
- **Comportamento:** Tolerâncias absoluta/relativa; avaliação vetorizada da função.
- **Regras:** Limites infinitos via transformação.
- **Erros a tratar:** Tolerância não alcançada (aviso).
- **Casos especiais:** Singularidade nos extremos; função não vetorizada.
- **Critérios de aceitação:**
  - CA1: integral(@(x) exp(-x.^2),-Inf,Inf) ≈ sqrt(pi)
  - CA2: trapz em dados amostrados

### RF-049 — Equações diferenciais ordinárias e algébrico-diferenciais (origem: F049, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Solvers explícitos/implícitos de passo variável (Runge-Kutta 4/5 e 2/3, Adams, BDF/NDF, Rosenbrock, trapezoidal, TR-BDF2), problemas implícitos totalmente, DAE de índice 1, eventos, matriz de massa, Jacobiano/esparsidade, saída densa, objeto de problema EDO.

- **Entradas:** Função derivada, intervalo, condição inicial, opções.
- **Saídas:** Tempo e estado; eventos; solução avaliável.
- **Comportamento:** Controle de passo por tolerâncias; detecção de eventos com direção e terminal; refino de saída.
- **Regras:** Interface de opções compatível com conjunto de parâmetros nomeados.
- **Erros a tratar:** Falha de tolerância mínima (aviso + resultado parcial).
- **Casos especiais:** Problemas rígidos; descontinuidades; eventos simultâneos.
- **Critérios de aceitação:**
  - CA1: Oscilador de Van der Pol rígido resolvido por BDF
  - CA2: Evento terminal para no instante correto ±tol
  - CA3: Comparação com solução analítica em problemas de referência

### RF-050 — Problemas de contorno, retardo e EDP 1-D (origem: F050, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Solvers de valor de contorno (colocação), equações com retardo (constante e variável), EDPs parabólicas/elípticas 1-D.

- **Entradas:** Funções, malha, condições.
- **Saídas:** Soluções.
- **Comportamento:** Continuação e refino de malha; históricos de retardo.
- **Regras:** Malhas monotônicas.
- **Erros a tratar:** Não convergência.
- **Casos especiais:** Retardos que se anulam; malha inicial ruim.
- **Critérios de aceitação:**
  - CA1: bvp de referência com erro < tol
  - CA2: dde23 em problema publicado

### RF-051 — Otimização básica e raízes (origem: F051, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Minimização escalar limitada, Nelder-Mead sem restrições, zero de função escalar, mínimos quadrados não negativos, estrutura de opções de otimização, funções de saída/plotagem de iteração.

- **Entradas:** Função objetivo, chute inicial, opções.
- **Saídas:** Solução, valor, flag de saída, saída de diagnóstico.
- **Comportamento:** Critérios de parada por tolerância/iterações; exibição iterativa.
- **Regras:** Flag de saída padronizada.
- **Erros a tratar:** Intervalo inválido.
- **Casos especiais:** Função descontínua; sem mudança de sinal em fzero.
- **Critérios de aceitação:**
  - CA1: fminsearch na Rosenbrock converge
  - CA2: fzero encontra raiz com bracket
  - CA3: Funções de plotagem de iteração chamadas a cada passo

### RF-052 — Grafos e redes (origem: F052, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Grafos direcionados/não direcionados com nós/arestas tabulares, caminhos mínimos, BFS/DFS, componentes conexos, árvore geradora, fluxo máximo, centralidade, ordenação topológica, isomorfismo, plotagem com layouts.

- **Entradas:** Listas de arestas, matrizes de adjacência.
- **Saídas:** Objetos de grafo, resultados de algoritmos.
- **Comportamento:** Pesos; multigrafos; propriedades de nós/arestas em tabelas.
- **Regras:** Nós nomeados ou numerados.
- **Erros a tratar:** Nó inexistente.
- **Casos especiais:** Grafo vazio; laços; pesos negativos (Bellman-Ford).
- **Critérios de aceitação:**
  - CA1: shortestpath Dijkstra correto
  - CA2: maxflow em rede de teste
  - CA3: plot com layout force

### RF-053 — Números aleatórios (origem: F053, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Geradores configuráveis (Mersenne Twister, SIMD-MT, multiplicative lagged Fibonacci, Philox, Threefry, combined recursive), sementes, estados, fluxos e subfluxos, distribuições uniforme/normal/inteira, permutações.

- **Entradas:** Semente, gerador, dimensões.
- **Saídas:** Arrays aleatórios; estado.
- **Comportamento:** Reprodutibilidade por semente; fluxos independentes para paralelismo.
- **Regras:** Estado salvável e restaurável.
- **Erros a tratar:** Gerador desconhecido.
- **Casos especiais:** Reprodução bit a bit de sequências de outros produtos: NÃO exigida (requisito: algoritmos padrão publicados).
- **Critérios de aceitação:**
  - CA1: rng(42) reprodutível na mesma versão
  - CA2: Testes estatísticos (TestU01 SmallCrush) aprovados
  - CA3: Subfluxos não sobrepostos

### RF-054 — Precisão estendida e aritmética especial (origem: F054, status EM DESENVOLVIMENTO)

O sistema deve oferecer: Funções de precisão e de representação (eps, flintmax, realmax), aritmética inteira (idivide), conversões de base/binário/hexadecimal, aritmética de precisão estendida interna.

- **Entradas:** Valores.
- **Saídas:** Valores.
- **Comportamento:** Uso interno de bibliotecas de precisão múltipla (sem GMP/MPFR, que são LGPL).
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Valores no limite.
- **Critérios de aceitação:**
  - CA1: dec2bin/hex2dec ida-e-volta
  - CA2: flintmax correto

### RF-055 — Funções de data e tempo legadas (origem: F055, status PLANEJADA)

O sistema deve oferecer: Funções numéricas de datas (números seriais de data, conversão de texto, calendário, cronômetro tic/toc, clock, cputime).

- **Entradas:** Números seriais/textos.
- **Saídas:** Números/textos.
- **Comportamento:** Número serial de data com época própria documentada publicamente.
- **Regras:** —
- **Erros a tratar:** Formato inválido.
- **Casos especiais:** Anos bissextos; fuso local.
- **Critérios de aceitação:**
  - CA1: tic/toc mede intervalo com resolução < 1µs
  - CA2: Conversão serial↔texto

### RF-060 — Sistema de objetos gráficos (origem: F060, status PLANEJADA)

O sistema deve oferecer: Hierarquia de objetos (raiz, figura, eixos, primitivas, grupos, transformações) com propriedades consultáveis/alteráveis, valores padrão hierárquicos, callbacks, busca de objetos, ciclo de vida.

- **Entradas:** Chamadas de criação; pares propriedade-valor.
- **Saídas:** Handles de objetos.
- **Comportamento:** Notação de ponto e get/set; padrões herdados por nível; handles inválidos após exclusão.
- **Regras:** Ordem de filhos determina empilhamento.
- **Erros a tratar:** Propriedade inexistente/valor inválido.
- **Casos especiais:** Muitos objetos (1e5); exclusão durante callback.
- **Critérios de aceitação:**
  - CA1: set/get e notação de ponto equivalentes
  - CA2: Padrão definido na raiz afeta novas figuras
  - CA3: findobj por propriedade

### RF-061 — Gráficos 2-D (origem: F061, status PLANEJADA)

O sistema deve oferecer: Linhas, dispersão, barras (agrupadas/empilhadas/horizontais), área, escada, haste, barras de erro, histograma (1-D/2-D), pizza/rosca, polares, escalas log, eixos duplos, preenchimentos.

- **Entradas:** Vetores/matrizes, estilos.
- **Saídas:** Figuras.
- **Comportamento:** Ciclo automático de cores e estilos; limites automáticos; 'hold' para sobreposição.
- **Regras:** Especificadores de linha curtos (cor/marcador/estilo).
- **Erros a tratar:** Dimensões incompatíveis.
- **Casos especiais:** NaN cria lacunas; datetime/duration/categorical nos eixos.
- **Critérios de aceitação:**
  - CA1: plot com NaN gera lacuna
  - CA2: eixo datetime formatado
  - CA3: bar empilhado soma correta

### RF-062 — Gráficos 3-D, superfícies e volumes (origem: F062, status PLANEJADA)

O sistema deve oferecer: Linhas 3-D, superfícies/malhas, contornos 2-D/3-D, campos vetoriais, fatias de volume, isossuperfícies, streamlines, patches, iluminação, materiais, câmera, transparência.

- **Entradas:** Grades, volumes.
- **Saídas:** Figuras 3-D.
- **Comportamento:** Rotação interativa; projeção ortográfica/perspectiva; colormaps em dados.
- **Regras:** —
- **Erros a tratar:** Tamanhos incompatíveis.
- **Casos especiais:** Grades irregulares; NaN em superfícies; transparência ordenada.
- **Critérios de aceitação:**
  - CA1: surf com colormap e iluminação
  - CA2: isosurface correto em volume de teste
  - CA3: Rotação de câmera interativa a ≥30 fps em 1e6 triângulos (meta)

### RF-063 — Gráficos especializados de alto nível (origem: F063, status PLANEJADA)

O sistema deve oferecer: Mapas de calor, coordenadas paralelas, gráficos empilhados, enxame, bolhas, caixas, dispersão em matriz, nuvem de palavras básica, gráficos geográficos com mapas base (online).

- **Entradas:** Tabelas/arrays.
- **Saídas:** Gráficos.
- **Comportamento:** Objetos de gráfico com propriedades próprias (não primitivas).
- **Regras:** Mapas base exigem serviço de tiles.
- **Erros a tratar:** Variável inexistente.
- **Casos especiais:** Dados ausentes.
- **Critérios de aceitação:**
  - CA1: heatmap a partir de tabela agrega contagens
  - CA2: boxchart com grupos

### RF-064 — Anotações, legendas, rótulos e texto matemático (origem: F064, status PLANEJADA)

O sistema deve oferecer: Títulos, rótulos, legendas, barras de cor, texto, setas, formas, interpretadores TeX/LaTeX para fórmulas.

- **Entradas:** Textos.
- **Saídas:** Objetos de texto.
- **Comportamento:** Interpretador LaTeX com subconjunto matemático; legenda automática.
- **Regras:** —
- **Erros a tratar:** LaTeX inválido (aviso, texto bruto).
- **Casos especiais:** Unicode; fórmulas longas.
- **Critérios de aceitação:**
  - CA1: Legenda lista séries visíveis
  - CA2: LaTeX \frac renderizado

### RF-065 — Mapas de cores e estilos (origem: F065, status PLANEJADA)

O sistema deve oferecer: Mapas de cores padrão e personalizados (incl. perceptuais), limites de cor, temas claro/escuro para gráficos, ordem de cores.

- **Entradas:** Matrizes Nx3.
- **Saídas:** Estado de cor.
- **Comportamento:** Mapas próprios do projeto; TURBO (Apache-2.0) permitido com atribuição.
- **Regras:** Nomes de mapas próprios — não copiar paletas proprietárias do original.
- **Erros a tratar:** Formato inválido.
- **Casos especiais:** Mapas com poucos níveis.
- **Critérios de aceitação:**
  - CA1: Tema escuro altera cores padrão de figura
  - CA2: colormap personalizado aplicado

### RF-066 — Layouts de múltiplos gráficos (origem: F066, status PLANEJADA)

O sistema deve oferecer: Subdivisão de figura em grades, layouts em mosaico com espaçamento e títulos compartilhados, eixos ligados.

- **Entradas:** Dimensões de grade.
- **Saídas:** Eixos posicionados.
- **Comportamento:** Redimensionamento responsivo; eixos ligados sincronizam limites.
- **Regras:** —
- **Erros a tratar:** Índice fora da grade.
- **Casos especiais:** Mosaicos aninhados; spans.
- **Critérios de aceitação:**
  - CA1: Layout 2x3 com título compartilhado
  - CA2: linkaxes sincroniza zoom

### RF-067 — Interação com gráficos (origem: F067, status PLANEJADA)

O sistema deve oferecer: Zoom, pan, rotação, dicas de dados, escovação e ligação de dados, barra de ferramentas de eixos, callbacks de mouse/teclado.

- **Entradas:** Eventos de usuário.
- **Saídas:** Estado de visão; seleções.
- **Comportamento:** Dicas personalizáveis; escovação marca dados em variáveis ligadas.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Gráficos com 1e6 pontos.
- **Critérios de aceitação:**
  - CA1: Zoom por roda do mouse
  - CA2: Dica de dados exibe valores
  - CA3: Escovação propaga para gráficos ligados

### RF-068 — Animação e captura de quadros (origem: F068, status PLANEJADA)

O sistema deve oferecer: Atualização de gráficos em laço, linhas animadas, captura de quadros, reprodução de filmes, exportação para vídeo/GIF.

- **Entradas:** Atualizações de propriedades.
- **Saídas:** Quadros, arquivos.
- **Comportamento:** drawnow com limitação de taxa ('limitrate').
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Animação com 1e4 quadros.
- **Critérios de aceitação:**
  - CA1: drawnow limitrate não excede 20 fps
  - CA2: getframe retorna imagem RGB

### RF-069 — Exportação e impressão de figuras (origem: F069, status PLANEJADA)

O sistema deve oferecer: Salvar/exportar figuras e eixos em PNG, JPEG, TIFF, PDF vetorial, EPS, SVG, com resolução, cor de fundo, recorte; impressão; copiar para área de transferência; formato de figura editável próprio.

- **Entradas:** Figura, formato, opções.
- **Saídas:** Arquivos.
- **Comportamento:** Vetorial preserva texto; raster em DPI escolhido.
- **Regras:** —
- **Erros a tratar:** Formato não suportado.
- **Casos especiais:** Figuras com transparência em vetor; fontes ausentes.
- **Critérios de aceitação:**
  - CA1: PDF vetorial contém texto selecionável
  - CA2: PNG 300 dpi com dimensões corretas
  - CA3: Figura salva e recarregada idêntica

### RF-070 — Exibição de imagens (origem: F070, status PLANEJADA)

O sistema deve oferecer: Exibição de matrizes como imagens (indexada, intensidade, RGB), escala de cores, eixos de imagem.

- **Entradas:** Matrizes.
- **Saídas:** Figuras.
- **Comportamento:** Imagens grandes com reamostragem; proporção de pixel.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Imagens 16 bits; com alfa.
- **Critérios de aceitação:**
  - CA1: imagesc escala automática
  - CA2: RGB uint8 exibido sem conversão de cor

### RF-075 — Componentes de interface programáticos (origem: F075, status PLANEJADA)

O sistema deve oferecer: Janelas de aplicativo com componentes (botões, campos, listas, tabelas, árvores, abas, painéis, layouts em grade, menus, barras de ferramentas, sliders, knobs, medidores, lâmpadas, seletores de data, HTML embutido, eixos), callbacks, estilos.

- **Entradas:** Chamadas de criação e propriedades.
- **Saídas:** Janelas interativas.
- **Comportamento:** Callbacks com objeto de evento; layout responsivo em grade.
- **Regras:** —
- **Erros a tratar:** Propriedade inválida.
- **Casos especiais:** Muitos componentes; callbacks lentos (interrupção/fila).
- **Critérios de aceitação:**
  - CA1: Aplicativo com 50 componentes abre < 1 s
  - CA2: Callback de botão recebe evento
  - CA3: Tabela editável sincroniza dados

### RF-076 — Editor visual de aplicativos (origem: F076, status PLANEJADA)

O sistema deve oferecer: Ambiente de projeto visual com paleta de componentes, canvas, alinhamento, inspetor de propriedades, vista de código com regiões geradas/editáveis, gerenciamento de callbacks, executar/depurar, layouts responsivos.

- **Entradas:** Interações de arrastar/soltar.
- **Saídas:** Arquivo de app (formato próprio) com código e layout.
- **Comportamento:** Código gerado e código do usuário separados; renomeação propaga.
- **Regras:** Formato de arquivo próprio do projeto.
- **Erros a tratar:** Arquivo corrompido.
- **Casos especiais:** Conversão de apps legados.
- **Critérios de aceitação:**
  - CA1: Criar app com botão e callback sem escrever layout manualmente
  - CA2: Reabrir preserva código do usuário

### RF-077 — Interfaces legadas baseadas em figura (origem: F077, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Controles de interface em figuras tradicionais, editor de layout legado e conversão para o novo modelo.

- **Entradas:** Arquivos de layout legado.
- **Saídas:** Figuras com controles.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: uicontrol cria botão funcional

### RF-078 — Diálogos padrão (origem: F078, status PLANEJADA)

O sistema deve oferecer: Seleção de arquivo/pasta, mensagens, perguntas, entrada de texto, lista, progresso, cor, fonte; diálogos dentro de app.

- **Entradas:** Opções.
- **Saídas:** Escolhas do usuário.
- **Comportamento:** Modais e não modais; cancelamento retorna valor distinto.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Cancelamento; multisseleção.
- **Critérios de aceitação:**
  - CA1: Cancelar retorna 0/vazio
  - CA2: Barra de progresso atualiza e fecha

### RF-079 — Empacotamento e compartilhamento de apps (origem: F079, status PLANEJADA)

O sistema deve oferecer: Empacotar app com dependências em instalador de app, galeria de apps instalados.

- **Entradas:** Projeto de app.
- **Saídas:** Pacote instalável.
- **Comportamento:** Análise automática de dependências.
- **Regras:** —
- **Erros a tratar:** Arquivo ausente.
- **Casos especiais:** Dependências de outras extensões.
- **Critérios de aceitação:**
  - CA1: Empacotar e instalar app em outra máquina

### RF-085 — Ambiente desktop com painéis (origem: F085, status PLANEJADA)

O sistema deve oferecer: Janela principal com janela de comandos, navegador de arquivos, workspace, histórico, editor, figuras ancoradas, barras de ferramentas, layouts salvos, tema claro/escuro, zoom de fonte, múltiplos monitores.

- **Entradas:** Interações de UI.
- **Saídas:** Estado de layout persistido.
- **Comportamento:** Painéis ancoráveis/desacopláveis; layouts nomeados.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Telas HiDPI; tela pequena.
- **Critérios de aceitação:**
  - CA1: Layout persiste entre sessões
  - CA2: Tema escuro aplicado a todos os painéis
  - CA3: Painel desacoplado em outro monitor

### RF-086 — Editor de código (origem: F086, status PLANEJADA)

O sistema deve oferecer: Edição com realce, dobra, múltiplos cursores, busca/substituição com regex, indentação automática, formatação, autocompletar contextual, dicas de assinatura, ir para definição, renomeação de variáveis, seções de código executáveis, execução de seleção.

- **Entradas:** Texto do usuário.
- **Saídas:** Arquivos salvos.
- **Comportamento:** Análise contínua em segundo plano; seções delimitadas por marcador de comentário.
- **Regras:** —
- **Erros a tratar:** Arquivo somente leitura.
- **Casos especiais:** Arquivos > 10k linhas; codificações.
- **Critérios de aceitação:**
  - CA1: Autocompletar sugere variáveis e funções do caminho
  - CA2: Executar seção avalia apenas a seção
  - CA3: Renomear variável atualiza ocorrências no escopo

### RF-087 — Análise estática de código (origem: F087, status PLANEJADA)

O sistema deve oferecer: Mensagens de problemas/melhorias em tempo real, correções automáticas, supressão por linha/arquivo, regras configuráveis, relatório por pasta, métricas (complexidade).

- **Entradas:** Código.
- **Saídas:** Diagnósticos.
- **Comportamento:** Identificadores de mensagem estáveis; configuração por arquivo de projeto.
- **Regras:** Regras próprias (não copiar textos de mensagens do original).
- **Erros a tratar:** —
- **Casos especiais:** Código com erro de sintaxe parcial.
- **Critérios de aceitação:**
  - CA1: Variável não usada sinalizada
  - CA2: Supressão por comentário respeitada
  - CA3: Relatório de pasta exportável

### RF-088 — Depurador (origem: F088, status PLANEJADA)

O sistema deve oferecer: Pontos de parada (linha, condicionais, em erro/aviso/NaN), passo a passo, entrar/sair, pilha de chamadas, inspeção e alteração de variáveis, continuar até cursor, depuração de apps e callbacks.

- **Entradas:** Pontos de parada, comandos.
- **Saídas:** Estado pausado.
- **Comportamento:** Pontos de parada persistem entre edições quando possível.
- **Regras:** —
- **Erros a tratar:** Linha inválida.
- **Casos especiais:** Parada em código JIT; em callback de timer.
- **Critérios de aceitação:**
  - CA1: Parada condicional dispara só quando verdadeira
  - CA2: 'parar em erro' pausa no local do erro
  - CA3: Variável alterada no modo depuração afeta execução

### RF-089 — Profiler de desempenho (origem: F089, status PLANEJADA)

O sistema deve oferecer: Medição de tempo por função/linha, contagem de chamadas, árvore de chamadas, gráfico de chama, comparação, relatório HTML, profiler de memória.

- **Entradas:** Execução de código.
- **Saídas:** Relatórios.
- **Comportamento:** Overhead controlado; histórico de execuções.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Código recursivo; multithread.
- **Critérios de aceitação:**
  - CA1: Relatório lista tempo por linha
  - CA2: Gráfico de chama navegável

### RF-090 — Navegador de arquivos e pasta atual (origem: F090, status PLANEJADA)

O sistema deve oferecer: Navegação, criação, renomeação, exclusão, pré-visualização, filtros, indicação de status de controle de versão, busca de arquivos.

- **Entradas:** Ações de arquivo.
- **Saídas:** Sistema de arquivos atualizado.
- **Comportamento:** Sincroniza com 'cd' do runtime.
- **Regras:** —
- **Erros a tratar:** Permissão negada.
- **Casos especiais:** Pastas grandes (1e5 arquivos).
- **Critérios de aceitação:**
  - CA1: cd no prompt atualiza painel
  - CA2: Excluir pede confirmação

### RF-091 — Navegador de workspace e editor de variáveis (origem: F091, status PLANEJADA)

O sistema deve oferecer: Lista de variáveis com tamanho/classe/estatísticas, edição em planilha de arrays/tabelas/structs, gráficos rápidos a partir da seleção, inspeção de objetos, limpeza de dados interativa.

- **Entradas:** Variáveis.
- **Saídas:** Variáveis modificadas; código gerado.
- **Comportamento:** Edições geram código equivalente reproduzível.
- **Regras:** —
- **Erros a tratar:** Valor inválido.
- **Casos especiais:** Variáveis enormes (paginação).
- **Critérios de aceitação:**
  - CA1: Editar célula atualiza variável
  - CA2: Seleção gera gráfico
  - CA3: Edições registradas como código

### RF-092 — Ferramenta de importação de dados (origem: F092, status PLANEJADA)

O sistema deve oferecer: Assistente para importar texto/planilha com pré-visualização, detecção de tipos, seleção de intervalo, regras para valores inválidos, geração de script/função de importação.

- **Entradas:** Arquivos.
- **Saídas:** Variáveis e código gerado.
- **Comportamento:** Usa mesmas opções que funções de leitura (F112, F113).
- **Regras:** —
- **Erros a tratar:** Arquivo ilegível.
- **Casos especiais:** Arquivos grandes; cabeçalhos múltiplos.
- **Critérios de aceitação:**
  - CA1: Importar CSV com datas detectadas
  - CA2: Script gerado reproduz importação

### RF-093 — Histórico de comandos (origem: F093, status PLANEJADA)

O sistema deve oferecer: Registro persistente de comandos com busca, reexecução, criação de script a partir de seleção, agrupamento por sessão.

- **Entradas:** Comandos.
- **Saídas:** Histórico.
- **Comportamento:** Configurável (tamanho, duplicatas).
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Histórico corrompido.
- **Critérios de aceitação:**
  - CA1: Seta para cima com prefixo filtra histórico
  - CA2: Criar script a partir da seleção

### RF-094 — Projetos (origem: F094, status PLANEJADA)

O sistema deve oferecer: Projetos com arquivos rastreados, rótulos, atalhos, caminho e scripts de inicialização/encerramento, análise de dependências, verificações de integridade, referências a outros projetos, exportação/arquivamento, integração com controle de versão.

- **Entradas:** Pasta de projeto.
- **Saídas:** Metadados de projeto (formato próprio).
- **Comportamento:** Abrir projeto configura caminho e executa inicialização.
- **Regras:** —
- **Erros a tratar:** Metadados inválidos.
- **Casos especiais:** Projetos aninhados/referenciados; arquivos fora do projeto.
- **Critérios de aceitação:**
  - CA1: Abrir projeto adiciona pastas ao caminho
  - CA2: Análise de dependências encontra arquivos faltantes
  - CA3: Exportar como arquivo compactado

### RF-095 — Controle de versão integrado (origem: F095, status PLANEJADA)

O sistema deve oferecer: Git (clonar, commit, branch, merge, push/pull, histórico, stash, conflitos) e Subversion; indicadores no navegador de arquivos; diff/merge visual de arquivos de código e binários do projeto.

- **Entradas:** Repositórios.
- **Saídas:** Operações VCS.
- **Comportamento:** Credenciais pelo armazenamento do sistema.
- **Regras:** —
- **Erros a tratar:** Falha de autenticação.
- **Casos especiais:** Conflitos; repositórios grandes; LFS.
- **Critérios de aceitação:**
  - CA1: Commit/branch/merge via UI
  - CA2: Indicador de modificado no navegador

### RF-096 — Comparação e mesclagem de arquivos (origem: F096, status PLANEJADA)

O sistema deve oferecer: Diferença lado a lado de texto, código, pastas, arquivos de dados (variáveis), documentos executáveis e modelos; mesclagem de três vias; relatório exportável.

- **Entradas:** Dois/três arquivos.
- **Saídas:** Relatório; arquivo mesclado.
- **Comportamento:** Comparação semântica de tipos estruturados.
- **Regras:** —
- **Erros a tratar:** Tipos incompatíveis.
- **Casos especiais:** Arquivos binários grandes.
- **Critérios de aceitação:**
  - CA1: Diff de texto com realce intra-linha
  - CA2: Comparar dois arquivos de dados lista variáveis diferentes

### RF-097 — Preferências e configurações (origem: F097, status PLANEJADA)

O sistema deve oferecer: Sistema hierárquico de configurações (fábrica, instalação, usuário, sessão) com API programática, diálogo de preferências, importação/exportação.

- **Entradas:** Valores de configuração.
- **Saídas:** Configurações persistidas.
- **Comportamento:** Precedência de níveis; validação de tipos.
- **Regras:** —
- **Erros a tratar:** Valor inválido.
- **Casos especiais:** Arquivo corrompido (restauração).
- **Critérios de aceitação:**
  - CA1: Valor de sessão sobrepõe usuário
  - CA2: Exportar/importar preferências

### RF-098 — Atalhos de teclado e ações personalizáveis (origem: F098, status PLANEJADA)

O sistema deve oferecer: Mapa de atalhos editável, conjuntos predefinidos (incl. estilo de outros editores), favoritos (botões de código), barra de acesso rápido.

- **Entradas:** Mapeamentos.
- **Saídas:** Ações.
- **Comportamento:** Conflitos detectados.
- **Regras:** —
- **Erros a tratar:** Conflito de atalho.
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Atalho personalizado executa ação
  - CA2: Favorito executa trecho de código

### RF-099 — Visualizador de diagramas de classes e dependências (origem: F099, status PLANEJADA)

O sistema deve oferecer: Diagramas de classes gerados de código, grafo de dependências de arquivos/projetos.

- **Entradas:** Pastas/classes.
- **Saídas:** Diagramas.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Projetos grandes.
- **Critérios de aceitação:**
  - CA1: Diagrama de classes mostra herança

### RF-100 — Opções de inicialização e modos sem interface (origem: F100, status PLANEJADA)

O sistema deve oferecer: Execução em lote ('-batch'), sem desktop, script de inicialização, variáveis de ambiente, log de sessão, código de saída, execução de script por linha de comando.

- **Entradas:** Argumentos de linha de comando.
- **Saídas:** Execução sem UI; código de saída.
- **Comportamento:** Erro não tratado em lote retorna código ≠ 0.
- **Regras:** —
- **Erros a tratar:** Argumento inválido.
- **Casos especiais:** Sem display (headless) com gráficos (renderização offscreen).
- **Critérios de aceitação:**
  - CA1: chimalab -batch "disp(1)" imprime 1 e sai com 0
  - CA2: Erro sai com código ≠ 0
  - CA3: Gráficos exportáveis sem display

### RF-101 — Barra de ferramentas contextual (faixa de opções) (origem: F101, status PLANEJADA)

O sistema deve oferecer: Faixa de opções com abas contextuais por ferramenta/editor, galeria de apps, pesquisa de ações.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Disposição própria (não reproduzir layout distintivo do original).
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Aba contextual aparece ao focar editor

### RF-102 — Notificações e mensagens do sistema (origem: F102, status PLANEJADA)

O sistema deve oferecer: Central de notificações (atualizações, tarefas concluídas).

- **Entradas:** Eventos.
- **Saídas:** Notificações.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Notificação de tarefa em segundo plano concluída

### RF-103 — Busca global (origem: F103, status PLANEJADA)

O sistema deve oferecer: Busca de arquivos, símbolos, ações e documentação a partir de campo único.

- **Entradas:** Consulta.
- **Saídas:** Resultados.
- **Comportamento:** Índice incremental.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Busca encontra função por nome parcial

### RF-104 — Ambiente via navegador e acesso remoto (origem: F104, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Uso do ambiente pelo navegador conectado a sessão local/remota.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Inferência: componentes 'connector' indicam servidor web interno para UI.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Interface acessível por navegador local com autenticação por token

### RF-105 — Assistente de IA/sugestões de código (origem: F105, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Sugestões de código baseadas em modelos (componentes 'suggestion_models' presentes).

- **Entradas:** Contexto do editor.
- **Saídas:** Sugestões.
- **Comportamento:** Hipótese: autocompletar com modelo local e/ou serviço.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Sugestões opcionais e desativáveis; nenhum dado enviado sem consentimento

### RF-106 — Documentos executáveis (scripts vivos) (origem: F106, status PLANEJADA)

O sistema deve oferecer: Documento com texto formatado, equações, imagens, links, código e saídas embutidas (valores, gráficos, tabelas); execução por seção; controles interativos (sliders, listas, botões) ligados a variáveis; tarefas interativas que geram código; modo de leitura/ocultar código.

- **Entradas:** Documento.
- **Saídas:** Documento com saídas; formato de arquivo próprio aberto.
- **Comportamento:** Saídas associadas às linhas; execução incremental; alteração de controle reexecuta seção.
- **Regras:** Formato de arquivo próprio (ZIP+JSON documentado); importação de formatos abertos (ipynb, Markdown).
- **Erros a tratar:** Arquivo corrompido (recuperação).
- **Casos especiais:** Documentos grandes; saídas pesadas.
- **Critérios de aceitação:**
  - CA1: Gráfico aparece inline abaixo do código
  - CA2: Slider reexecuta seção ao mudar
  - CA3: Salvar/reabrir preserva saídas

### RF-107 — Exportação de documentos (origem: F107, status PLANEJADA)

O sistema deve oferecer: Exportar documento executável/script para PDF, HTML, Word (docx), LaTeX, Markdown, notebook Jupyter.

- **Entradas:** Documento.
- **Saídas:** Arquivos.
- **Comportamento:** Opções de página e inclusão de código.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Equações e imagens em todos os formatos.
- **Critérios de aceitação:**
  - CA1: Exportar para PDF com equações
  - CA2: Exportar ipynb abre no Jupyter

### RF-108 — Publicação de scripts com marcação (origem: F108, status PLANEJADA)

O sistema deve oferecer: Gerar relatório a partir de script com comentários de marcação (títulos, listas, código, saídas, imagens) em HTML/PDF/DOC/LaTeX/XML.

- **Entradas:** Script.
- **Saídas:** Relatório.
- **Comportamento:** Estilo via folha de transformação personalizável.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Publicar script com seções gera HTML com índice

### RF-109 — Verificação ortográfica (origem: F109, status PLANEJADA)

O sistema deve oferecer: Verificação ortográfica em texto de documentos e comentários.

- **Entradas:** Texto.
- **Saídas:** Sugestões.
- **Comportamento:** Dicionários por idioma.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Palavra incorreta sublinhada

### RF-110 — Arquivos de dados nativos (MAT) (origem: F110, status PLANEJADA)

O sistema deve oferecer: Salvar/carregar variáveis em arquivo binário de dados (versões 4, 5/6/7 compactada, 7.3 baseada em HDF5), carga/salvamento parcial, acesso a partes de variáveis sem carregar tudo, texto ASCII.

- **Entradas:** Variáveis/arquivo.
- **Saídas:** Arquivo/variáveis.
- **Comportamento:** Compressão; objetos serializados; leitura parcial para arquivos v7.3.
- **Regras:** Formato v5 tem especificação pública do fabricante; v7.3 é HDF5 com convenções.
- **Erros a tratar:** Arquivo corrompido; versão não suportada.
- **Casos especiais:** Arquivos > 2 GB; objetos de classes desconhecidas (carregar como struct + aviso).
- **Critérios de aceitação:**
  - CA1: Ida-e-volta de todos os tipos básicos
  - CA2: Ler arquivo v5/v7/v7.3 de referência gerado por ferramenta de terceiros
  - CA3: Leitura parcial de matriz 10 GB sem carregar inteira

### RF-111 — E/S de arquivo de baixo nível (origem: F111, status PLANEJADA)

O sistema deve oferecer: fopen/fclose/fread/fwrite/fprintf/fscanf/fgetl/fgets/fseek/ftell/feof/ferror com codificações e ordem de bytes; mapeamento de arquivo em memória.

- **Entradas:** Arquivos, formatos.
- **Saídas:** Dados.
- **Comportamento:** Precisões com conversão ('int16=>double'); codificação de texto configurável.
- **Regras:** —
- **Erros a tratar:** Arquivo inexistente; permissão.
- **Casos especiais:** Arquivos > 4 GB; fim de linha misto.
- **Critérios de aceitação:**
  - CA1: fread com ordem big-endian
  - CA2: memmapfile acesso aleatório

### RF-112 — Texto delimitado e de largura fixa (origem: F112, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de tabelas, matrizes, células e linhas de texto/CSV com detecção automática de opções (delimitadores, cabeçalhos, tipos, formatos de data, locale), objetos de opções de importação editáveis, leitura formatada com especificadores.

- **Entradas:** Arquivos texto.
- **Saídas:** Tabelas/arrays.
- **Comportamento:** Detecção heurística reproduzível; opções explicitamente configuráveis.
- **Regras:** —
- **Erros a tratar:** Arquivo malformado (regra configurável: erro/omitir/preencher).
- **Casos especiais:** Aspas, delimitadores múltiplos, linhas malformadas, BOM, codificações.
- **Critérios de aceitação:**
  - CA1: CSV com cabeçalho → tabela com tipos corretos
  - CA2: Opções editadas mudam tipo de coluna
  - CA3: textscan com formato

### RF-113 — Planilhas (origem: F113, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de planilhas (xlsx, xlsm, xls, ods) com folhas, intervalos, formatos de data, preservação de formatação ao escrever; informação de folhas.

- **Entradas:** Arquivos de planilha.
- **Saídas:** Tabelas/arrays.
- **Comportamento:** Sem depender de software de planilha instalado.
- **Regras:** —
- **Erros a tratar:** Folha inexistente.
- **Casos especiais:** Células mescladas; fórmulas (valor calculado); datas 1900/1904.
- **Critérios de aceitação:**
  - CA1: Ler intervalo nomeado
  - CA2: Escrever tabela preservando formatação existente
  - CA3: xls legado lido

### RF-114 — JSON (origem: F114, status PLANEJADA)

O sistema deve oferecer: Codificação/decodificação JSON ↔ estruturas, leitura/escrita de arquivos JSON em structs/tabelas com opções (formatação, nomes de campos).

- **Entradas:** Texto/arquivos JSON.
- **Saídas:** Structs/arrays.
- **Comportamento:** Arrays homogêneos viram arrays numéricos; null → [] ou NaN conforme opção.
- **Regras:** Nomes inválidos ajustados reversivelmente.
- **Erros a tratar:** JSON inválido com posição.
- **Casos especiais:** Números grandes; Unicode; profundidade.
- **Critérios de aceitação:**
  - CA1: Ida-e-volta preserva estrutura
  - CA2: Pretty print opcional

### RF-115 — XML (origem: F115, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de XML em structs, DOM completo (parse, navegação, XPath, transformação XSLT, validação por esquema), escrita.

- **Entradas:** Arquivos XML.
- **Saídas:** Structs/DOM.
- **Comportamento:** Namespaces; atributos mapeados.
- **Regras:** —
- **Erros a tratar:** XML malformado.
- **Casos especiais:** XML grande (streaming); entidades externas desativadas por segurança.
- **Critérios de aceitação:**
  - CA1: XPath retorna nós
  - CA2: XXE bloqueado por padrão

### RF-116 — HTML e documentos de texto (origem: F116, status PLANEJADA)

O sistema deve oferecer: Leitura de tabelas de páginas HTML e de documentos Word (tabelas/texto) para tabelas.

- **Entradas:** Arquivos HTML/DOCX.
- **Saídas:** Tabelas.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Tabelas aninhadas.
- **Critérios de aceitação:**
  - CA1: readtable de tabela HTML
  - CA2: readtable de tabela em .docx

### RF-117 — Formatos colunares (Parquet, Arrow/Feather) (origem: F117, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de Parquet com compressão, metadados, leitura seletiva de colunas/linhas; Feather.

- **Entradas:** Arquivos.
- **Saídas:** Tabelas.
- **Comportamento:** Mapeamento de tipos documentado.
- **Regras:** —
- **Erros a tratar:** Tipo sem mapeamento.
- **Casos especiais:** Tipos aninhados; datas com fuso.
- **Critérios de aceitação:**
  - CA1: Ida-e-volta Parquet com datetime
  - CA2: Leitura de colunas selecionadas

### RF-118 — Imagens (origem: F118, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita/informação de BMP, GIF (animado), JPEG, JPEG 2000, PCX, PNG (alfa, 16 bits), TIFF (multipágina, tiles, metadados), PPM/PGM, ICO/CUR, HDF raster, e formatos científicos de imagem.

- **Entradas:** Arquivos.
- **Saídas:** Arrays.
- **Comportamento:** Metadados acessíveis; qualidade/compressão configuráveis.
- **Regras:** —
- **Erros a tratar:** Formato desconhecido.
- **Casos especiais:** Imagens enormes; CMYK; paletas.
- **Critérios de aceitação:**
  - CA1: PNG 16 bits com alfa ida-e-volta
  - CA2: TIFF multipágina lido por índice

### RF-119 — Áudio e vídeo (origem: F119, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de áudio (WAV, FLAC, OGG, MP3 leitura, MP4/M4A), reprodução e gravação de áudio com dispositivos, leitura/escrita de vídeo (AVI, MP4, MJ2, etc.) quadro a quadro, informação de mídia.

- **Entradas:** Arquivos/dispositivos.
- **Saídas:** Arrays de amostras/quadros.
- **Comportamento:** Seleção de dispositivo; taxas de amostragem.
- **Regras:** —
- **Erros a tratar:** Codec não suportado.
- **Casos especiais:** Codecs indisponíveis na plataforma.
- **Critérios de aceitação:**
  - CA1: audioread WAV/FLAC correto
  - CA2: Gravar e reproduzir com dispositivo padrão
  - CA3: VideoWriter MP4 legível em players comuns

### RF-120 — Formatos científicos (HDF5, HDF4, NetCDF, CDF, FITS) (origem: F120, status PLANEJADA)

O sistema deve oferecer: Leitura/escrita de alto nível e APIs de baixo nível para HDF5, NetCDF; leitura de HDF4/HDF-EOS, CDF, FITS; informação/estrutura de arquivos.

- **Entradas:** Arquivos.
- **Saídas:** Arrays/estruturas.
- **Comportamento:** Atributos, grupos, chunking, compressão.
- **Regras:** —
- **Erros a tratar:** Arquivo inválido.
- **Casos especiais:** Arquivos > memória; tipos compostos.
- **Critérios de aceitação:**
  - CA1: h5write/h5read ida-e-volta com atributos
  - CA2: NetCDF leitura de variável parcial

### RF-121 — Sistema de arquivos e arquivos compactados (origem: F121, status PLANEJADA)

O sistema deve oferecer: Listar, criar, copiar, mover, excluir, atributos, caminhos; zip/unzip, tar/untar, gzip/gunzip; caminhos remotos (armazenamento em nuvem e HTTP) em funções de E/S.

- **Entradas:** Caminhos.
- **Saídas:** Arquivos/listagens.
- **Comportamento:** Curingas; recursão; caminhos remotos por prefixo de esquema.
- **Regras:** —
- **Erros a tratar:** Permissão; inexistente.
- **Casos especiais:** Links simbólicos; nomes Unicode; caminhos longos no Windows.
- **Critérios de aceitação:**
  - CA1: dir recursivo com **
  - CA2: zip/unzip ida-e-volta
  - CA3: Leitura de CSV em armazenamento de objetos compatível S3

### RF-122 — Comandos do sistema e ambiente (origem: F122, status PLANEJADA)

O sistema deve oferecer: Executar comandos do SO com captura de saída e status, variáveis de ambiente, informação do computador, área de transferência.

- **Entradas:** Comandos.
- **Saídas:** Saídas.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** Comando inexistente.
- **Casos especiais:** Comandos interativos; codificação.
- **Critérios de aceitação:**
  - CA1: system retorna status e saída
  - CA2: clipboard copiar/colar texto

### RF-123 — Gerenciamento de segredos (origem: F123, status PLANEJADA)

O sistema deve oferecer: Armazenamento seguro de segredos (tokens, senhas) no cofre do sistema, com funções de definir/obter/remover e entrada mascarada.

- **Entradas:** Nome do segredo, valor.
- **Saídas:** Segredos.
- **Comportamento:** Cofre de credenciais do SO.
- **Regras:** —
- **Erros a tratar:** Segredo inexistente.
- **Casos especiais:** Cofre indisponível (Linux sem serviço).
- **Critérios de aceitação:**
  - CA1: Segredo persiste entre sessões e não aparece em texto

### RF-124 — Funções criptográficas/hash internas (origem: F124, status NÃO VERIFICÁVEL)

O sistema deve oferecer: Componente criptográfico presente; finalidade exposta ao usuário não determinada.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Requisito a definir após investigação em documentação pública

### RF-130 — Datastores (origem: F130, status PLANEJADA)

O sistema deve oferecer: Fontes de dados particionadas para coleções de arquivos (texto tabular, planilha, imagem, arquivo genérico com leitor personalizado, chave-valor, Parquet, conjuntos de arquivos, arrays em memória), combinadas/transformadas, particionamento, embaralhamento, escrita de saída.

- **Entradas:** Pastas/arquivos/opções.
- **Saídas:** Blocos de dados.
- **Comportamento:** read/hasdata/reset/preview/readall; partições para paralelismo.
- **Regras:** —
- **Erros a tratar:** Arquivo ilegível (pular/errar).
- **Casos especiais:** Arquivos heterogêneos; remotos.
- **Critérios de aceitação:**
  - CA1: Datastore de 1000 CSV itera em blocos
  - CA2: transform aplica função lazy
  - CA3: partition(n,i) cobre todos os arquivos

### RF-131 — Arrays altos (avaliação adiada) e MapReduce (origem: F131, status PLANEJADA)

O sistema deve oferecer: Arrays 'altos' com operações adiadas e execução otimizada em passagens; framework MapReduce.

- **Entradas:** Datastore.
- **Saídas:** Resultados agregados.
- **Comportamento:** Combinação de operações em poucas passagens; gather dispara execução.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Operações não suportadas em modo adiado (erro explícito).
- **Critérios de aceitação:**
  - CA1: mean de tall array igual ao mean em memória
  - CA2: Número de passagens minimizado

### RF-135 — Acesso web (HTTP/REST) (origem: F135, status PLANEJADA)

O sistema deve oferecer: Funções de alto nível de leitura/escrita/download web com opções (tempo-limite, cabeçalhos, autenticação, certificados); API HTTP de baixo nível com mensagens, cabeçalhos, corpo, cookies, progresso, redirecionamentos, multipart; proxy.

- **Entradas:** URLs, opções.
- **Saídas:** Dados decodificados conforme tipo de conteúdo.
- **Comportamento:** Conversão automática JSON/imagem/texto; validação de certificado configurável.
- **Regras:** —
- **Erros a tratar:** Status HTTP de erro; tempo-limite; TLS inválido.
- **Casos especiais:** Respostas grandes (streaming); proxy com autenticação.
- **Critérios de aceitação:**
  - CA1: webread JSON → struct
  - CA2: Upload multipart
  - CA3: Proxy configurável nas preferências

### RF-136 — FTP, e-mail e sockets (origem: F136, status PLANEJADA)

O sistema deve oferecer: Cliente FTP/SFTP, envio de e-mail SMTP, clientes/servidores TCP, UDP, WebSocket.

- **Entradas:** Endereços, portas, dados.
- **Saídas:** Dados recebidos.
- **Comportamento:** Callbacks por bytes disponíveis/terminador; tempo-limite.
- **Regras:** —
- **Erros a tratar:** Host inacessível.
- **Casos especiais:** Conexão interrompida.
- **Critérios de aceitação:**
  - CA1: tcpclient eco local
  - CA2: sendmail via servidor de teste
  - CA3: SFTP listar/baixar

### RF-137 — Portas seriais e Bluetooth (origem: F137, status PLANEJADA)

O sistema deve oferecer: Comunicação por portas seriais (configuração, terminadores, callbacks), descoberta de portas; Bluetooth/BLE.

- **Entradas:** Porta, parâmetros.
- **Saídas:** Dados.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** Porta ocupada.
- **Casos especiais:** Desconexão física.
- **Critérios de aceitação:**
  - CA1: serialport loopback
  - CA2: Listar portas disponíveis

### RF-138 — Sensores móveis e IoT (origem: F138, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Aquisição de sensores de dispositivo móvel e envio/leitura de canais IoT em serviços de nuvem.

- **Entradas:** Conta/dispositivo.
- **Saídas:** Dados de sensores.
- **Comportamento:** Depende de serviços externos; oferecer integração com serviços abertos.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Requisito equivalente com protocolo aberto (MQTT/HTTP) a definir

### RF-139 — Gerenciador de hardware e pacotes de suporte (origem: F139, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Descoberta e configuração de hardware e instalação de pacotes de suporte a dispositivos.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Requisito a definir

### RF-140 — Funções compiladas externas (C/C++/Fortran) (origem: F140, status PLANEJADA)

O sistema deve oferecer: API C/C++/Fortran para criar funções compiladas chamáveis pelo runtime, comando de compilação com detecção de compiladores, API de arrays de dados (C e C++ moderna).

- **Entradas:** Código-fonte nativo.
- **Saídas:** Bibliotecas carregáveis.
- **Comportamento:** API própria estável e documentada; camada de compatibilidade de origem com API publicamente documentada do original é questão jurídica pendente.
- **Regras:** —
- **Erros a tratar:** Falha de compilação/carregamento.
- **Casos especiais:** Compiladores ausentes; ABI.
- **Critérios de aceitação:**
  - CA1: Compilar e chamar função C que soma arrays
  - CA2: Erro nativo não derruba o processo (isolamento opcional)

### RF-141 — Motor embutível (chamar o runtime de outras linguagens) (origem: F141, status PLANEJADA)

O sistema deve oferecer: APIs para iniciar/conectar ao runtime a partir de C, C++, Java, Python, .NET e Fortran; sessões compartilhadas; avaliação assíncrona.

- **Entradas:** Chamadas de API.
- **Saídas:** Resultados convertidos.
- **Comportamento:** Conversões de tipos documentadas por linguagem.
- **Regras:** —
- **Erros a tratar:** Sessão perdida.
- **Casos especiais:** Múltiplas sessões; threads.
- **Critérios de aceitação:**
  - CA1: Python inicia runtime, avalia e recebe array NumPy
  - CA2: Conectar a sessão compartilhada

### RF-142 — API de arquivos de dados para outras linguagens (origem: F142, status PLANEJADA)

O sistema deve oferecer: Biblioteca C/Fortran para ler/escrever arquivos de dados nativos fora do runtime.

- **Entradas:** Arquivos.
- **Saídas:** Dados.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Programa C lê arquivo de dados escrito pelo runtime

### RF-143 — Interoperabilidade com Python (origem: F143, status PLANEJADA)

O sistema deve oferecer: Chamar bibliotecas Python diretamente, executar código/arquivos Python, conversão automática de tipos (incl. arrays NumPy), seleção de ambiente/interpretador, execução em processo ou fora dele.

- **Entradas:** Módulos/expressões Python.
- **Saídas:** Objetos convertidos.
- **Comportamento:** Conversão zero-cópia quando possível; ambiente configurável.
- **Regras:** —
- **Erros a tratar:** Módulo inexistente.
- **Casos especiais:** Versões múltiplas de Python; GIL; travamentos.
- **Critérios de aceitação:**
  - CA1: py.math.sqrt(4) == 2
  - CA2: ndarray ↔ array numérico sem cópia
  - CA3: Modo fora de processo isola falhas

### RF-144 — Interoperabilidade com Java (origem: F144, status PLANEJADA)

O sistema deve oferecer: JVM embutida opcional: criar objetos Java, chamar métodos, conversão de tipos, caminho de classes dinâmico/estático.

- **Entradas:** Classes Java.
- **Saídas:** Objetos.
- **Comportamento:** —
- **Regras:** Runtime Java fornecido pelo usuário (não redistribuir JRE comercial).
- **Erros a tratar:** Classe inexistente.
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Criar java.util.ArrayList e adicionar elemento

### RF-145 — Interoperabilidade com .NET e COM (origem: F145, status PLANEJADA)

O sistema deve oferecer: Carregar assemblies .NET, criar objetos, eventos; automação COM/ActiveX (Windows) incluindo controle de aplicativos de escritório; servidor de automação.

- **Entradas:** Assemblies/ProgIDs.
- **Saídas:** Objetos.
- **Comportamento:** —
- **Regras:** Somente Windows para COM.
- **Erros a tratar:** Assembly inexistente.
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Chamar System.DateTime.Now
  - CA2: Automação COM de planilha (se instalada)

### RF-146 — Bibliotecas compartilhadas C e interfaces C++ geradas (origem: F146, status PLANEJADA)

O sistema deve oferecer: Carregar bibliotecas C via cabeçalho e chamar funções; gerar interface para bibliotecas C++ a partir de cabeçalhos.

- **Entradas:** Cabeçalhos, bibliotecas.
- **Saídas:** Interfaces.
- **Comportamento:** Análise de cabeçalhos C/C++.
- **Regras:** —
- **Erros a tratar:** Tipo não suportado.
- **Casos especiais:** Templates; ponteiros; callbacks.
- **Critérios de aceitação:**
  - CA1: loadlibrary + calllib em libm
  - CA2: Interface C++ gerada para classe simples

### RF-150 — Multithreading implícito (origem: F150, status PLANEJADA)

O sistema deve oferecer: Paralelização automática de operações elementares e álgebra linear em múltiplos núcleos, com controle do número de threads.

- **Entradas:** Operações.
- **Saídas:** Mesmos resultados, mais rápido.
- **Comportamento:** Limiar de tamanho para paralelizar; maxNumCompThreads.
- **Regras:** Resultados determinísticos (ou documentar diferenças de arredondamento).
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Speedup ≥ 3x em 4 núcleos para soma elementar de 1e8
  - CA2: Resultados idênticos com 1 e N threads (onde garantido)

### RF-151 — Execução em segundo plano e futures (origem: F151, status PLANEJADA)

O sistema deve oferecer: Pool de segundo plano para avaliar funções assíncronas, futures com estado, cancelamento, callbacks de conclusão (afterEach/afterAll), filas de dados entre tarefas.

- **Entradas:** Funções.
- **Saídas:** Futures.
- **Comportamento:** Execução em threads do runtime; resultados obtidos sob demanda.
- **Regras:** —
- **Erros a tratar:** Erro propagado ao obter resultado.
- **Casos especiais:** Erro em tarefa; cancelamento.
- **Critérios de aceitação:**
  - CA1: parfeval no pool de fundo não bloqueia UI
  - CA2: cancel interrompe tarefa

### RF-152 — Comunicação entre processos (origem: F152, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Comunicação entre o runtime e outros processos (canais internos, sessões compartilhadas).

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Inferência a partir de componentes 'interprocess'.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Protocolo próprio documentado

### RF-155 — Framework de testes unitários (origem: F155, status PLANEJADA)

O sistema deve oferecer: Testes baseados em scripts, funções e classes; asserções/verificações/suposições/fatais; fixtures (configuração/limpeza, compartilhadas); parametrização; tags; seleção; executores com plugins (TAP, JUnit XML, cobertura, diagnóstico); execução paralela; navegador de testes.

- **Entradas:** Arquivos de teste.
- **Saídas:** Resultados de teste; relatórios.
- **Comportamento:** Diagnósticos ricos em falhas; resultados tabuláveis.
- **Regras:** —
- **Erros a tratar:** Teste com erro vs falha distintos.
- **Casos especiais:** Testes que alteram estado global; dependências.
- **Critérios de aceitação:**
  - CA1: Classe de teste parametrizada gera N casos
  - CA2: Relatório JUnit válido
  - CA3: Filtro por tag

### RF-156 — Mocks e testes de desempenho (origem: F156, status PLANEJADA)

O sistema deve oferecer: Criação de objetos simulados com comportamento definido e verificação de interações; framework de medição de desempenho com amostragem estatística.

- **Entradas:** Interfaces/classes.
- **Saídas:** Mocks; métricas.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Mock registra chamadas
  - CA2: Medição com intervalo de confiança

### RF-157 — Testes de interface de aplicativos (origem: F157, status PLANEJADA)

O sistema deve oferecer: Gestos programáticos (pressionar, digitar, escolher, arrastar) em componentes de apps para testes automatizados.

- **Entradas:** Apps.
- **Saídas:** Resultados.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Teste pressiona botão e verifica rótulo

### RF-158 — Cobertura de código (origem: F158, status PLANEJADA)

O sistema deve oferecer: Cobertura por função/linha/decisão/condição, relatórios HTML e Cobertura XML.

- **Entradas:** Código, testes.
- **Saídas:** Relatórios.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Relatório Cobertura XML válido

### RF-159 — Ferramenta de build (origem: F159, status PLANEJADA)

O sistema deve oferecer: Arquivo de build com tarefas, dependências, entradas/saídas para build incremental, tarefas padrão (análise de código, testes, limpeza, empacotamento), execução por linha de comando.

- **Entradas:** Arquivo de build.
- **Saídas:** Execução de tarefas.
- **Comportamento:** Pula tarefas atualizadas.
- **Regras:** —
- **Erros a tratar:** Tarefa inexistente.
- **Casos especiais:** Ciclos de dependência.
- **Critérios de aceitação:**
  - CA1: Segunda execução pula tarefas inalteradas
  - CA2: Falha de tarefa retorna código ≠ 0

### RF-160 — Automação e diagnóstico de execução (origem: F160, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Infraestrutura de relatórios/diagnósticos de automação compartilhada por testes e build.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Requisito a definir

### RF-165 — Empacotamento de extensões (origem: F165, status PLANEJADA)

O sistema deve oferecer: Empacotar código, apps, documentação e exemplos em arquivo de extensão instalável com metadados, versão, dependências e compatibilidade de plataforma.

- **Entradas:** Pasta de projeto.
- **Saídas:** Pacote.
- **Comportamento:** Instalação adiciona ao caminho; desinstalação remove.
- **Regras:** —
- **Erros a tratar:** Metadados inválidos.
- **Casos especiais:** Conflitos de versão.
- **Critérios de aceitação:**
  - CA1: Empacotar, instalar, desinstalar
  - CA2: Dependência ausente reportada

### RF-166 — Gerenciador de pacotes (origem: F166, status PLANEJADA)

O sistema deve oferecer: Instalar/atualizar/remover pacotes a partir de repositórios, resolução de dependências por versão semântica, repositórios locais/remotos, lockfile.

- **Entradas:** Nomes/versões.
- **Saídas:** Pacotes instalados.
- **Comportamento:** Resolução determinística.
- **Regras:** —
- **Erros a tratar:** Pacote inexistente.
- **Casos especiais:** Conflitos insolúveis.
- **Critérios de aceitação:**
  - CA1: Instalar pacote com dependências transitivas
  - CA2: Lockfile reproduz ambiente

### RF-167 — Explorador de extensões (catálogo online) (origem: F167, status PLANEJADA)

O sistema deve oferecer: Navegar, buscar e instalar extensões de catálogo comunitário/oficial.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Depende de serviço externo — equivalente próprio.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Offline.
- **Critérios de aceitação:**
  - CA1: Catálogo próprio navegável e instalável

### RF-168 — Verificação de compatibilidade de versões (origem: F168, status PLANEJADA)

O sistema deve oferecer: Análise de código para identificar usos de funcionalidades removidas/alteradas entre versões e sugerir atualizações.

- **Entradas:** Código.
- **Saídas:** Relatório.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Relatório lista funções obsoletas

### RF-170 — Instalador interativo e silencioso (origem: F170, status PLANEJADA)

O sistema deve oferecer: Instalação com seleção de produtos/componentes, pasta de destino, atalhos; modo silencioso com arquivo de respostas; instalação a partir de mídia offline; verificação de integridade de pacotes.

- **Entradas:** Seleções/arquivo de respostas.
- **Saídas:** Instalação.
- **Comportamento:** Pacotes assinados verificados; dependências entre componentes resolvidas.
- **Regras:** Instalação silenciosa por arquivo de respostas; pacotes assinados.
- **Erros a tratar:** Assinatura inválida.
- **Casos especiais:** Espaço insuficiente; instalação parcial (rollback).
- **Critérios de aceitação:**
  - CA1: Instalação silenciosa com arquivo de respostas
  - CA2: Pacote adulterado rejeitado
  - CA3: Remover componente individual

### RF-171 — Atualizações (origem: F171, status PLANEJADA)

O sistema deve oferecer: Verificação, download e aplicação de atualizações e correções com reversão.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Canal de atualização próprio assinado.
- **Regras:** —
- **Erros a tratar:** Falha de download.
- **Casos especiais:** Offline.
- **Critérios de aceitação:**
  - CA1: Atualização assinada aplicada e revertida

### RF-172 — Desinstalação (origem: F172, status PLANEJADA)

O sistema deve oferecer: Remoção completa ou por componente.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Desinstalar remove arquivos e atalhos

### RF-173 — Licenciamento e ativação (origem: F173, status PLANEJADA)

O sistema deve oferecer: Em produtos comerciais: ativação, servidor de licenças em rede, verificação de licença por produto.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Decisão: NÃO reproduzir mecanismo; projeto aberto sem ativação. Investigação NÃO tentou contornar o mecanismo.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Aplicação funciona sem ativação (decisão de projeto)

### RF-174 — Conta de usuário e login (origem: F174, status PLANEJADA)

O sistema deve oferecer: Login em conta para serviços online.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Equivalente próprio opcional (contas apenas para serviços online do projeto).
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Uso offline completo sem conta

### RF-175 — Telemetria de uso e relatório de falhas (origem: F175, status PLANEJADA)

O sistema deve oferecer: Coleta de uso e relatórios de travamento.

- **Entradas:** Consentimento.
- **Saídas:** Relatórios.
- **Comportamento:** Novo projeto: telemetria DESLIGADA por padrão, opt-in explícito; relatórios de falha locais com envio manual.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Nenhum tráfego de rede sem consentimento
  - CA2: Relatório de falha salvo localmente

### RF-180 — Ajuda em linha de comando e navegador de documentação (origem: F180, status PLANEJADA)

O sistema deve oferecer: Ajuda a partir de comentários de código, documentação de referência navegável e pesquisável offline/online, exemplos executáveis, 'ver também', documentação de extensões de terceiros integrada.

- **Entradas:** Consultas.
- **Saídas:** Páginas.
- **Comportamento:** Documentação própria escrita do zero (NÃO traduzir/parafrasear documentação do original).
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Documentação offline instalada como componente.
- **Critérios de aceitação:**
  - CA1: help f mostra comentário inicial de f
  - CA2: Busca offline retorna resultados em < 300 ms
  - CA3: Exemplo abre e executa

### RF-181 — Localização da interface (origem: F181, status PLANEJADA)

O sistema deve oferecer: Interface e mensagens traduzidas (meta: pt-BR e inglês na v1, outros idiomas depois); formatação dependente de locale.

- **Entradas:** Configuração de idioma.
- **Saídas:** UI localizada.
- **Comportamento:** Novo projeto: pt-BR e en como mínimo; arquitetura para outros idiomas.
- **Regras:** —
- **Erros a tratar:** Recurso ausente → fallback inglês.
- **Casos especiais:** Texto mais longo que layout.
- **Critérios de aceitação:**
  - CA1: Troca de idioma sem reinstalar
  - CA2: Fallback para chave ausente

### RF-182 — Exemplos e materiais de aprendizado (origem: F182, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Exemplos executáveis e cursos/tutoriais interativos integrados (componentes 'academy', 'grader').

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Conteúdo próprio.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Exemplos próprios abrem e executam

### RF-200 — Editor de diagramas de blocos (origem: F200, status PLANEJADA)

O sistema deve oferecer: Canvas para modelos hierárquicos: inserir blocos, conectar portas com roteamento automático e manual, ramificações, rótulos de sinais, anotações (texto/imagem/equações), áreas, zoom/pan, desfazer/refazer, copiar/colar entre modelos, navegação hierárquica com trilha, alinhamento/distribuição, busca rápida de blocos digitando no canvas.

- **Entradas:** Interações de usuário.
- **Saídas:** Modelo editado.
- **Comportamento:** Roteamento evita sobreposição; conexões inválidas destacadas.
- **Regras:** Aparência visual própria (não reproduzir ícones/estilo do original).
- **Erros a tratar:** Conexão de tipos incompatíveis (diagnóstico na atualização).
- **Casos especiais:** Modelos com 1e4 blocos; desfazer extenso.
- **Critérios de aceitação:**
  - CA1: Modelo com 1000 blocos edita a 60 fps
  - CA2: Desfazer/refazer ilimitado na sessão
  - CA3: Copiar subsistema entre modelos preserva parâmetros

### RF-201 — Biblioteca de blocos padrão (origem: F201, status PLANEJADA)

O sistema deve oferecer: Fontes (constante, degrau, rampa, seno, pulso, relógio, ruído, de workspace/arquivo, sinal de entrada), sumidouros (scope, display, para workspace/arquivo, terminator, stop), contínuos (integrador com limites/reset, derivada, função de transferência, espaço de estados, zero-polo, atraso de transporte, PID), descontinuidades (saturação, zona morta, relé, quantizador, backlash, rate limiter), discretos (atraso unitário, integrador discreto, filtro, FT discreta, ZOH, memória), operações matemáticas (soma, ganho, produto, funções, trigonometria, mínimo/máximo, matriz, polinômio), lógica e bits, tabelas de busca 1-D/2-D/N-D, roteamento (mux/demux, bus, switch, multiport switch, selector, goto/from, merge), atributos de sinal (conversão de tipo, IC, rate transition, largura), funções definidas pelo usuário, verificação de modelo (asserções), utilitários de modelo, strings, mensagens.

- **Entradas:** Seleção de blocos.
- **Saídas:** Blocos com parâmetros.
- **Comportamento:** Cada bloco tem semântica de tempo, tipos e dimensões documentada pelo projeto.
- **Regras:** Parâmetros avaliados como expressões do runtime.
- **Erros a tratar:** Parâmetro inválido.
- **Casos especiais:** Parâmetros variáveis no tempo (tunáveis); blocos com estados.
- **Critérios de aceitação:**
  - CA1: Cada bloco tem teste de referência contra solução analítica/numérica independente
  - CA2: Integrador com reset externo
  - CA3: Lookup N-D com extrapolação configurável

### RF-202 — Motor de simulação (origem: F202, status PLANEJADA)

O sistema deve oferecer: Compilação do modelo (ordenação de blocos, propagação de tempos de amostragem, tipos, dimensões, complexidade), simulação contínua/discreta/híbrida multitaxa, solvers de passo fixo e variável (explícitos e rígidos), detecção e localização de cruzamentos por zero, laços algébricos (detecção e solução), tarefas única/multitarefa, tempos de amostragem herdados/constantes/disparados.

- **Entradas:** Modelo, configuração.
- **Saídas:** Sinais no tempo, estados.
- **Comportamento:** Ordem de execução determinística; diagnóstico configurável de laços algébricos e transições de taxa.
- **Regras:** Passo mínimo/máximo; tolerâncias.
- **Erros a tratar:** Passo mínimo violado; laço algébrico insolúvel.
- **Casos especiais:** Sistemas rígidos; descontinuidades; múltiplas taxas.
- **Critérios de aceitação:**
  - CA1: Modelo massa-mola-amortecedor coincide com solução analítica (tol)
  - CA2: Cruzamento por zero de bola quicando com erro < 1e-6
  - CA3: Sistema multitaxa com transições corretas

### RF-203 — Atributos de sinal: tipos, dimensões, unidades e barramentos (origem: F203, status PLANEJADA)

O sistema deve oferecer: Tipos numéricos e de ponto fixo, enumerações, dimensões (escalar, vetor, matriz, N-D, tamanho variável), complexidade, unidades físicas com verificação/conversão, barramentos (virtuais e não virtuais) com objetos de definição e editor, portas de elementos de barramento, exibição de tipos/dimensões nas linhas.

- **Entradas:** Definições de tipos/barramentos.
- **Saídas:** Sinais tipados.
- **Comportamento:** Propagação de atributos para frente e para trás; conflitos reportados.
- **Regras:** Unidades incompatíveis geram aviso/erro conforme configuração.
- **Erros a tratar:** Incompatibilidade de tipo/dimensão.
- **Casos especiais:** Barramentos aninhados; arrays de barramentos.
- **Critérios de aceitação:**
  - CA1: Propagação de tipos em cadeia
  - CA2: Conversão automática de unidades onde permitido
  - CA3: Editor de barramentos cria/edita definições

### RF-204 — Dados e parâmetros de modelo (origem: F204, status PLANEJADA)

O sistema deve oferecer: Workspaces de modelo/base, dicionários de dados (arquivo próprio), objetos de parâmetro/sinal, parâmetros ajustáveis durante simulação, editor de dados do modelo, explorador de modelos, inspetor de propriedades.

- **Entradas:** Variáveis/objetos.
- **Saídas:** Dados resolvidos.
- **Comportamento:** Ordem de resolução de símbolos documentada.
- **Regras:** —
- **Erros a tratar:** Símbolo não resolvido.
- **Casos especiais:** Símbolos duplicados em fontes diferentes.
- **Critérios de aceitação:**
  - CA1: Parâmetro em dicionário usado por vários modelos
  - CA2: Ajuste de ganho durante simulação reflete imediatamente

### RF-205 — Subsistemas e execução condicional (origem: F205, status PLANEJADA)

O sistema deve oferecer: Subsistemas virtuais e atômicos, habilitados, disparados, chamada de função, ação if/switch-case, iteradores for/while, for-each, subsistemas reutilizáveis/referenciados, reset.

- **Entradas:** Blocos de subsistema.
- **Saídas:** Hierarquia.
- **Comportamento:** Semântica de estados ao desabilitar (manter/resetar).
- **Regras:** —
- **Erros a tratar:** Configuração inconsistente.
- **Casos especiais:** Subsistemas aninhados profundos.
- **Critérios de aceitação:**
  - CA1: Subsistema habilitado mantém/reseta estados conforme opção
  - CA2: For-each processa vetor elemento a elemento

### RF-206 — Máscaras e blocos personalizados (origem: F206, status PLANEJADA)

O sistema deve oferecer: Editor de máscaras (parâmetros, diálogos, validações, ícones desenhados por comandos/imagens, código de inicialização), máscaras dinâmicas, bibliotecas personalizadas com links e controle de versão de links.

- **Entradas:** Definições de máscara.
- **Saídas:** Blocos personalizados.
- **Comportamento:** Links de biblioteca propagam alterações; links desativados/restaurados.
- **Regras:** —
- **Erros a tratar:** Parâmetro de máscara inválido.
- **Casos especiais:** Bibliotecas modificadas com instâncias abertas.
- **Critérios de aceitação:**
  - CA1: Máscara com parâmetro validado
  - CA2: Atualizar bloco de biblioteca reflete nas instâncias ligadas

### RF-207 — Referência de modelos e modelos protegidos (origem: F207, status PLANEJADA)

O sistema deve oferecer: Usar modelos como blocos dentro de outros, modos de simulação por referência, arquivos de cache de compilação, modelos protegidos (distribuição com proteção de IP: só simulação/código).

- **Entradas:** Modelos.
- **Saídas:** Hierarquia de modelos.
- **Comportamento:** Recompilação incremental baseada em verificação de alterações.
- **Regras:** —
- **Erros a tratar:** Modelo referenciado ausente.
- **Casos especiais:** Referências circulares; múltiplas instâncias.
- **Critérios de aceitação:**
  - CA1: Alterar modelo referenciado invalida apenas seu cache
  - CA2: Modelo protegido simula sem expor conteúdo

### RF-208 — Variantes (origem: F208, status PLANEJADA)

O sistema deve oferecer: Subsistemas/fontes/sumidouros variantes com condições, ativação em tempo de atualização/código/arranque, gerenciador de variantes.

- **Entradas:** Condições de variante.
- **Saídas:** Modelo ativo.
- **Comportamento:** Apenas variante ativa compilada (conforme modo).
- **Regras:** —
- **Erros a tratar:** Nenhuma variante ativa.
- **Casos especiais:** Variantes aninhadas.
- **Critérios de aceitação:**
  - CA1: Troca de variante muda resultado da simulação

### RF-209 — Blocos definidos pelo usuário (origem: F209, status PLANEJADA)

O sistema deve oferecer: Bloco de função em linguagem do runtime, blocos de sistema baseados em objeto, funções S em C e em linguagem do runtime, chamada direta de código C (C caller), ferramenta de integração de código legado.

- **Entradas:** Código do usuário.
- **Saídas:** Blocos.
- **Comportamento:** Interface de estados, saídas, derivadas e tempos de amostragem própria e documentada.
- **Regras:** —
- **Erros a tratar:** Erro de compilação.
- **Casos especiais:** Código C com falha (isolamento).
- **Critérios de aceitação:**
  - CA1: Bloco de função compila e simula
  - CA2: Função C integrada via C caller

### RF-210 — Visualização e registro de sinais (origem: F210, status PLANEJADA)

O sistema deve oferecer: Scope (múltiplos eixos, cursores, medições, triggers), display, painel de controle interativo (botões, knobs, medidores, lâmpadas ligados a parâmetros/sinais), registro de sinais, inspetor de dados de simulação (comparar execuções, tolerâncias, exportar), formato de dados de saída estruturado.

- **Entradas:** Sinais.
- **Saídas:** Visualizações, conjuntos de dados.
- **Comportamento:** Registro sem blocos adicionais; streaming durante simulação.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Simulações longas (decimação).
- **Critérios de aceitação:**
  - CA1: Comparar duas execuções com tolerância relativa
  - CA2: Scope atualiza durante simulação
  - CA3: Knob altera parâmetro em tempo de simulação

### RF-211 — Entrada de dados e cenários de teste (origem: F211, status PLANEJADA)

O sistema deve oferecer: Importar sinais de workspace/arquivo para portas raiz, editor de sinais, mapeamento de entradas.

- **Entradas:** Dados de sinal.
- **Saídas:** Entradas mapeadas.
- **Comportamento:** Interpolação/extrapolação configuráveis.
- **Regras:** —
- **Erros a tratar:** Dimensão incompatível.
- **Casos especiais:** Tempos não monotônicos.
- **Critérios de aceitação:**
  - CA1: Mapear dataset para portas por nome
  - CA2: Editor cria sinal de degrau

### RF-212 — Controle programático de simulação (origem: F212, status PLANEJADA)

O sistema deve oferecer: Iniciar/pausar/continuar/parar, simulação por API com objetos de entrada/saída, simulações em lote e paralelas, reinício rápido sem recompilar, passo-a-passo e retrocesso, estados operacionais (salvar/restaurar).

- **Entradas:** Objetos de entrada.
- **Saídas:** Objetos de saída.
- **Comportamento:** Saída consolidada com metadados e diagnósticos.
- **Regras:** —
- **Erros a tratar:** Parâmetro inexistente.
- **Casos especiais:** Erro em uma simulação de um lote.
- **Critérios de aceitação:**
  - CA1: Lote de 100 simulações paralelas com resultados indexados
  - CA2: Fast restart evita recompilação
  - CA3: Retroceder passos restaura estado

### RF-213 — Modos de simulação acelerados (origem: F213, status PLANEJADA)

O sistema deve oferecer: Modos com geração e compilação de código para acelerar simulação, com e sem interface de depuração.

- **Entradas:** Modelo.
- **Saídas:** Executável/biblioteca de simulação.
- **Comportamento:** Mesmos resultados do modo normal (dentro de tolerância).
- **Regras:** —
- **Erros a tratar:** Compilador ausente.
- **Casos especiais:** Blocos não suportados no modo.
- **Critérios de aceitação:**
  - CA1: Speedup ≥ 5x em modelo de referência
  - CA2: Resultados equivalentes ao modo normal

### RF-214 — Depuração e diagnóstico de modelos (origem: F214, status PLANEJADA)

O sistema deve oferecer: Depurador de simulação (pontos de parada em sinais/blocos), visualizador de diagnósticos, verificação em tempo de edição, perfilador de solver, consultor de desempenho, exibição de ordem de execução e cores/legenda de tempos de amostragem, consultor de modelo com verificações básicas.

- **Entradas:** Modelo.
- **Saídas:** Diagnósticos.
- **Comportamento:** Mensagens com links para blocos.
- **Regras:** Textos de verificação próprios.
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Ponto de parada condicional em sinal
  - CA2: Legenda de tempos de amostragem
  - CA3: Diagnóstico clicável destaca bloco

### RF-215 — Configuração de modelo (origem: F215, status PLANEJADA)

O sistema deve oferecer: Conjuntos de configuração: solver, importação/exportação de dados, otimização, diagnósticos, implementação de hardware (tamanhos de tipos, endianness), referências de configuração compartilhadas.

- **Entradas:** Parâmetros.
- **Saídas:** Configuração.
- **Comportamento:** Configurações referenciáveis entre modelos.
- **Regras:** —
- **Erros a tratar:** Valor inválido.
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Configuração referenciada compartilhada entre 2 modelos

### RF-216 — Formato de arquivo de modelo (origem: F216, status PLANEJADA)

O sistema deve oferecer: Formato de arquivo próprio aberto (ZIP + JSON/XML documentado) para modelos e bibliotecas; carregamento parcial; recuperação automática (autosave); exportação para versões anteriores do próprio formato.

- **Entradas:** Modelos.
- **Saídas:** Arquivos.
- **Comportamento:** Formato estável versionado e documentado.
- **Regras:** Leitura de formatos de modelo de terceiros apenas como importador, sujeita a avaliação jurídica (fora do escopo atual, DT-10).
- **Erros a tratar:** Arquivo corrompido.
- **Casos especiais:** Arquivos grandes; autosave após falha.
- **Critérios de aceitação:**
  - CA1: Salvar/abrir ida-e-volta sem perda
  - CA2: Autosave recupera após encerramento forçado
  - CA3: Importador experimental de formato de terceiros (se aprovado)

### RF-217 — Comparação e mesclagem de modelos (origem: F217, status PLANEJADA)

O sistema deve oferecer: Diff estrutural de modelos (blocos, linhas, parâmetros, configuração, workspaces), mesclagem de três vias, relatório.

- **Entradas:** Dois/três modelos.
- **Saídas:** Relatório/merge.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Modelos grandes.
- **Critérios de aceitação:**
  - CA1: Diff identifica bloco com parâmetro alterado
  - CA2: Merge 3-vias de alterações não conflitantes

### RF-218 — API programática de construção de modelos (origem: F218, status PLANEJADA)

O sistema deve oferecer: Criar/abrir/salvar modelos, adicionar/remover blocos e linhas, obter/definir parâmetros, buscar sistema, layout automático, callbacks de modelo e bloco (pré-carga, inicialização, início, término, etc.).

- **Entradas:** Comandos.
- **Saídas:** Modelos.
- **Comportamento:** Callbacks em pontos definidos do ciclo de vida.
- **Regras:** —
- **Erros a tratar:** Caminho de bloco inválido.
- **Casos especiais:** Callback com erro.
- **Critérios de aceitação:**
  - CA1: Script constrói modelo de 3 blocos e simula
  - CA2: Callback InitFcn executa antes da compilação

### RF-219 — Funções e mensagens em modelos (origem: F219, status PLANEJADA)

O sistema deve oferecer: Funções definidas em modelo chamáveis de outros pontos, portas de função, sinais de mensagem com filas, eventos discretos.

- **Entradas:** Blocos.
- **Saídas:** Execução.
- **Comportamento:** Filas com capacidade e política.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** Fila cheia.
- **Critérios de aceitação:**
  - CA1: Chamada de função entre subsistemas
  - CA2: Fila de mensagens FIFO com capacidade

### RF-220 — Linearização e equilíbrio básicos (origem: F220, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Extração de modelo linear em ponto de operação, busca de ponto de equilíbrio (trim).

- **Entradas:** Modelo, ponto.
- **Saídas:** Matrizes A,B,C,D; estados de equilíbrio.
- **Comportamento:** Perturbação numérica.
- **Regras:** —
- **Erros a tratar:** Não convergência.
- **Casos especiais:** Modelos com descontinuidades.
- **Critérios de aceitação:**
  - CA1: Linearização de modelo linear reproduz matrizes

### RF-221 — Co-simulação e FMI (origem: F221, status PLANEJADA)

O sistema deve oferecer: Importar unidades de modelo funcional (FMI 1/2/3, troca de modelo e co-simulação) como blocos; exportar modelos como FMU; serviços de co-simulação.

- **Entradas:** Arquivos FMU.
- **Saídas:** Blocos/FMUs.
- **Comportamento:** Padrão FMI aberto (Modelica Association).
- **Regras:** —
- **Erros a tratar:** FMU inválida.
- **Casos especiais:** FMU binária de plataforma diferente.
- **Critérios de aceitação:**
  - CA1: Importar FMU de referência (FMI Cross-Check) e simular
  - CA2: Exportar FMU validada por ferramenta de terceiros

### RF-222 — Particionamento, tarefas e agendamento (origem: F222, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Editor de agendamento de partições/tarefas, mapeamento de taxas, editor de interfaces.

- **Entradas:** Modelo.
- **Saídas:** Agendamento.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Ordem de partições editável afeta execução

### RF-223 — Busca e navegação em modelos (origem: F223, status PLANEJADA)

O sistema deve oferecer: Busca de blocos/parâmetros/sinais com filtros, substituição, navegador de hierarquia, pré-visualização de arquivos de modelo.

- **Entradas:** Consultas.
- **Saídas:** Resultados.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Busca por valor de parâmetro em hierarquia

### RF-224 — Modelos de partida e página inicial (origem: F224, status PLANEJADA)

O sistema deve oferecer: Galeria de modelos-modelo, criação a partir de modelo, página inicial com recentes.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** Conteúdo próprio.
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Criar modelo a partir de modelo-base próprio

### RF-225 — Documentação de sistemas em modelos (origem: F225, status PLANEJADA)

O sistema deve oferecer: Notas e documentação anexadas a sistemas no modelo.

- **Entradas:** Texto.
- **Saídas:** Notas.
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Nota persiste com o modelo

### RF-226 — Integração com projetos e controle de versão (origem: F226, status PLANEJADA)

O sistema deve oferecer: Modelos em projetos, análise de impacto de alterações, recarregamento após operações de VCS.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Recarregar modelo após checkout

### RF-227 — Simulação em segundo plano e cache (origem: F227, status INVESTIGADA PARCIALMENTE)

O sistema deve oferecer: Simulações em segundo plano sem bloquear o editor; cache de artefatos de compilação.

- **Entradas:** —
- **Saídas:** —
- **Comportamento:** —
- **Regras:** —
- **Erros a tratar:** —
- **Casos especiais:** —
- **Critérios de aceitação:**
  - CA1: Simulação longa não bloqueia edição

