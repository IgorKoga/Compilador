# Analisador Léxico, Sintático e Semântico - Compiladores

Este projeto é um compilador acadêmico (contendo as fases de Análise Léxica, Sintática e Semântica) desenvolvido em **Rust** para a disciplina de Compiladores. A ferramenta é capaz de processar arquivos de código da linguagem **Rust** (simplificada), gerar uma Árvore Sintática Abstrata (AST) em formato JSON, realizar verificações semânticas estritas e exportar a Tabela de Símbolos gerada também em JSON.

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

### 🚀 Mudanças e Evoluções Realizadas (Nova Versão em Rust)

#### 1. Migração e Modularização para Rust
O compilador foi completamente implementado/reescrito em **Rust**, tornando o processamento de código extremamente robusto e seguro. O projeto é composto por:
- [main.rs]: Ponto de entrada do compilador. Gerencia a leitura de arquivos, o fluxo das três fases de análise e a exportação dos relatórios/JSONs.
- [lexico.rs]: Lógica de scanner/tokenização em Rust para o subconjunto da linguagem.
- [sintatico.rs]: Parser descendente recursivo com tratamento de erros robusto via *Panic Mode* (sincronização por `;` ou palavras-chave).
- [semantico.rs]: Analisador semântico que percorre a AST gerada, gerencia a pilha de escopos e valida as regras de semântica.
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

---

### 💻 Como Compilar e Rodar

O comportamento do programa é definido com base no nome do arquivo executável no momento da execução:

#### 1. Compilação Geral (Modo Híbrido)
Executa a Fase Léxica (exibe tabela de tokens), Fase Sintática (exibe árvore, exporta AST em JSON) e a Fase Semântica (exibe relatório de erros/avisos, exporta a Tabela de Símbolos em JSON).
```bash
rustc main.rs -o compilador.exe
```
**Para rodar:**
```bash
./compilador.exe
```

#### 2. Apenas Fase Léxica (Tabela de Tokens)
Se o nome do executável contiver `"lexico"`, o compilador executa unicamente a análise de tokens.
```bash
rustc main.rs -o lexico.exe
```
**Para rodar:**
```bash
./lexico.exe
```

#### 3. Fase Sintática e Semântica
Se o nome do executável contiver `"sintatico"`, o compilador realiza o parser da AST e a verificação semântica completa, omitindo a tabela de tokens.
```bash
rustc main.rs -o sintatico.exe
```
**Para rodar:**
```bash
./sintatico.exe
```

*Nota: Os códigos a serem analisados devem estar na pasta `codigoRust/` (por exemplo, `soma.rs`). O compilador solicitará o nome do arquivo ao ser iniciado.*

## Grupo: Anna Flavia Tsurushima, Giovanna Beatriz Ramos, Gustavo Pelissari, Igor Henrique Koga, Letícia Aparecida.
