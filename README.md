# Analisador Léxico, Sintático e Semântico - Compiladores

Este projeto é um compilador acadêmico (contendo as fases de Análise Léxica, Sintática, Semântica e Geração de Código via SDT) desenvolvido em **Rust** para a disciplina de Compiladores. A ferramenta é capaz de processar arquivos de código da linguagem **Rust** (simplificada), gerar uma Árvore Sintática Abstrata (AST) em formato JSON, realizar verificações semânticas estritas, exportar a Tabela de Símbolos gerada também em JSON e traduzir a árvore (RPN, TAC e PrettyPrinter).

---

### 🔍 O que é um Analisador Léxico?
De forma simples, o analisador léxico é a "primeira fase" de um compilador. Sua função é ler o código-fonte (que é apenas uma sequência de caracteres) e agrupar esses caracteres em unidades significativas chamadas **Tokens**. 

Por exemplo: ao ler `let x = 10;`, o scanner identifica:
- `let` -> Palavra Reservada (`T_LET`)
- `x` -> Identificador (`T_ID`)
- `=` -> Operador de Atribuição (`T_ASSIGN`)
- `10` -> Número Inteiro (`T_NUM`)
- `;` -> Delimitador (`T_SEMICOLON`)

---

### 🌳 O que é o Analisador Sintático (Parser) e a AST?
O analisador sintático é a segunda fase. Ele recebe os tokens gerados pelo Léxico e verifica se a ordem dessas palavras faz sentido perante a gramática matemática da linguagem, construindo uma **Árvore Sintática Abstrata (AST)**.
Por exemplo, ele estrutura a expressão `a + b` em um nó binário (`BinaryExpr`) que possui a variável `a` à esquerda e `b` à direita.

---

### 🧠 O que é o Analisador Semântico?
O analisador semântico é a terceira fase. Ele valida a lógica do programa que não pode ser capturada pela sintaxe estrutural. Ele cria e gerencia uma **Tabela de Símbolos** com escopos aninhados e realiza validações como compatibilidade de tipos, uso de variáveis não declaradas, prevenção de reatribuição de constantes, detecção de divisão por zero e emissão de alertas (warnings) sobre variáveis não utilizadas ou não inicializadas.

---

### ⚙️ O que é a Síntese e Geração de Código (SDT)?
A quarta fase (Síntese e Geração) pega a árvore validada pela semântica e realiza a **Tradução Dirigida por Sintaxe (SDT)**. Através do padrão de projeto **Visitor**, a AST intocada na memória é percorrida para emitir ações e formatar diferentes tipos de saídas de código intermediário ou canônico a partir da mesma base.

---

### 🚀 Mudanças e Evoluções Realizadas (Nova Versão em Rust)

#### 1. Migração e Modularização para Rust
O compilador foi completamente implementado/reescrito em **Rust**, tornando o processamento de código extremamente robusto e seguro. O projeto é composto por:
- [main.rs]: Ponto de entrada do compilador. Gerencia a leitura de arquivos, o fluxo das três fases de análise e a exportação dos relatórios/JSONs.
- [lexico.rs]: Lógica de scanner/tokenização em Rust para o subconjunto da linguagem.
- [sintatico.rs]: Parser descendente recursivo com tratamento de erros robusto via *Panic Mode* (sincronização por `;` ou palavras-chave).
- [semantico.rs]: Analisador semântico que percorre a AST gerada, gerencia a pilha de escopos e valida as regras de semântica.
- [sdt.rs]: Módulo com a arquitetura Visitor que realiza a tradução SDT gerando Notação Polonesa Reversa (RPN), Código de Três Endereços (TAC) e formatação (PrettyPrinting).
- [ast.rs]: Definição das estruturas e enums que representam a AST e as instruções do programa, com métodos para impressão hierárquica e conversão para JSON.

#### 2. Implementação da Fase Semântica (Novidade)
O compilador agora realiza verificações lógicas profundas:
- **Tabela de Símbolos Multiescopo**: Suporte a escopos globais e locais delimitados por blocos `{ ... }`, blocos condicionais (`if`/`else`), laços (`while`) e declarações de funções (`fn`).
- **Verificação de Escopo**: Detecção de **variáveis não declaradas** e de **declarações duplicadas** dentro de um mesmo escopo.
- **Controle de Mutabilidade e Constantes**: Como no Rust real, variáveis são imutáveis por padrão. Se declaradas sem a palavra-chave `mut`, qualquer tentativa de reatribuição gera um erro semântico de atribuição a variável constante.
- **Inferência e Verificação de Tipos**:
  - Tipagem automática baseada em expressões (como `int`, `float`, `string`).
  - Coerção segura de `int` para `float` em operações aritméticas binárias.
  - Verificação de compatibilidade em atribuições e em expressões binárias comparativas (`<`, `>`, `==`) e aritméticas.
- **Detecção de Divisão por Zero**: Análise estática que acusa erro ao tentar dividir por constantes numéricas literais de valor `0` ou `0.0`.
- **Relatório de Avisos (Warnings)**:
  - Alerta sobre **variáveis declaradas mas nunca utilizadas** ao final de cada escopo.
  - Alerta sobre **variáveis utilizadas antes de receberem um valor inicial** (não inicializadas).

#### 3. Exportação de Resultados Avançada
Além do JSON da árvore sintática (AST), o compilador agora gera e exporta a **Tabela de Símbolos em JSON** (`<nome_do_arquivo>_symbols.json`) dentro do diretório `json/`, contendo o mapeamento de variáveis, escopos, tipos, mutabilidade e uso.

#### 4. Fase de Síntese e Geração de Código SDT
Etapa de tradução em `sdt.rs`. A mesma AST produz 3 saídas independentes em memória:
- **Notação Polonesa Reversa (RPN):** Tradutor que percorre expressões em pós-ordem para emitir código baseado em pilhas.
- **Gerador de Código Intermediário (TAC):** Emite o Código de Três Endereços (Three-Address Code) canônico e não-otimizado com linearização de controle de fluxo (`if`/`else`, `while`), rótulos (`L_x`) e variáveis temporárias (`t_x`).
- **PrettyPrinter:** Reconstrutor canônico com espaçamentos explícitos em blocos (`{ ... }`) para auditoria da árvore.

#### 5. Fase de Otimização de Código Intermediário (Novidade - Aulas 09 P1 e P2)
Implementado no módulo dedicado [otimizador.rs], aplicando rigorosamente a metodologia do Prof. Luciano Rovanni:
- **Líderes e Blocos Básicos:** Identificação formal de blocos básicos (B1, B2, ...) usando a Regra dos Líderes (início do programa, alvos de saltos e instruções pós-salto).
- **As 9 Passagens por Rodada:**
  1. **Propagação de Constantes e Cópias:** Tabela de conhecimento local por bloco com invalidação correta em redefinições, saltos e entradas de rótulos.
  2. **Dobramento de Constantes (Constant Folding):** Avaliação em tempo de compilação de operações aritméticas, relacionais (0 ou 1) e lógicas. Respeita a regra de ouro e **NUNCA dobra divisão por zero** para preservar o comportamento em runtime.
  3. **Identidades Algébricas e Redução de Força:** Aplicação de $x + 0 \to x$, $x \times 1 \to x$, $x \times 0 \to 0$, $x - x \to 0$ e redução de força $x \times 2 \to x + x$.
  4. **Eliminação de Subexpressões Comuns (CSE):** Reúso de cálculos idênticos com invalidação quando operandos são modificados.
  5. **Simplificação de Desvios:** Eliminação de `ifFalse 1` (nunca salta), conversão de `ifFalse 0` em `goto` e remoção de `goto L` seguido imediatamente de `L:`.
  6. **Eliminação de Código Inalcançável:** Remoção de instruções mortas após desvios incondicionais.
  7. **Peephole (Janela Deslizante):** Fusão de temporários `tK` usados uma única vez na instrução seguinte direto na variável de destino (ex: `t5 = c - 10; c = t5` $\to$ `c = c - 10`).
  8. **Eliminação de Código Morto (DCE):** Limpeza de temporários e variáveis nunca lidas (ex: `w = 8`) ou sobrescritas antes de leitura (ex: `contador = 0` antes de `contador = 60`). **NUNCA remove `print`/efeitos colaterais observáveis**.
  9. **Limpeza de Rótulos Não Utilizados:** Remoção de rótulos que deixaram de ser alvos de saltos.
- **Motor de Ponto Fixo:** Execução cíclica das 9 passagens até que uma rodada inteira não produza nenhuma alteração (limite de 20 rodadas).
- **Tabela Comparativa Antes vs Depois:** Exibe métricas de instruções eliminadas, temporários economizados e blocos básicos.
- **Interpretador de TAC e Comprovação Semântica:** Executa o TAC original e o TAC otimizado, comprovando que a **Regra de Ouro** (equivalência de comportamento observável) foi 100% satisfeita e medindo a redução real de instruções executadas na CPU.

---

### 💻 Como Compilar e Rodar

O comportamento do programa é definido com base no nome do arquivo executável no momento da execução:

#### 1. Compilação Geral (Pipeline Completo com Otimizador)
Executa todo o pipeline do compilador: Fase Léxica, Fase Sintática (AST e JSON), Fase Semântica (Tabela de Símbolos), Fase SDT (TAC e RPN) e a **Fase de Otimização com Simulação e Relatório Comparativo**:
```bash
rustc main.rs -o compilador
```
**Para rodar:**
```bash
./compilador
```

#### 2. Testando os Exemplos dos Slides da Aula 09:
- `exemplo-aula.rs`: Exemplo 3 dos slides (IF + WHILE com $14 \to 7$ instruções e saída `[60]`).
- `exemplo-algebra.rs`: Teste de identidades algébricas, CSE e remoção de `w = 8`.
- `num-compare.rs`: Desvios estáticos e eliminação de ramo inalcançável.
- `soma.rs`: Propagação, dobramento e eliminação de variáveis mortas.

---

## Grupo: Anna Flavia Tsurushima, Giovanna Beatriz Ramos, Gustavo Pelissari, Igor Henrique Koga, Letícia Aparecida.
