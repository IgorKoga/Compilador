# Analisador Léxico e Sintático - Compiladores

Este projeto é um compilador acadêmico (contendo Analisador Léxico e Sintático) desenvolvido para a disciplina de Compiladores. Ele foi evoluído de um protótipo simples para uma ferramenta modular capaz de processar arquivos de código da linguagem **Rust** e gerar uma Árvore Sintática Abstrata (AST) no formato JSON.

---

### 🔍 O que é um Analisador Léxico?
De forma simples, o analisador léxico é a "primeira fase" de um compilador. Sua função é ler o código-fonte (que é apenas uma sequência de caracteres) e agrupar esses caracteres em unidades significativas chamadas **Tokens**. 

Por exemplo: ao ler `let x = 10;`, o scanner identifica:
- `let` -> Palavra Reservada
- `x` -> Identificador
- `=` -> Operador de Atribuição
- `10` -> Número
- `;` -> Delimitador

---

### 🌳 O que é o Analisador Sintático (Parser) e a AST?
O analisador sintático é a segunda fase. Ele recebe os tokens gerados pelo Léxico e verifica se a ordem dessas palavras faz sentido perante a gramática matemática da linguagem, agrupando as instruções numa **Árvore Sintática Abstrata (AST)**.
Por exemplo, ele descobre que um sinal de `+` possui uma variável `a` na esquerda e `b` na direita. 

---

### 🚀 Mudanças e Evoluções Realizadas

#### 1. Modularização do Código
O projeto foi dividido em arquivos separados para facilitar a manutenção e organização:
- `lexico.hpp` e `lexico.cpp`: Implementação de toda a lógica de reconhecimento de caracteres e geração de Tokens.
- `ast.hpp`: Declarações das classes e da hierarquia de nós da Árvore Sintática Abstrata.
- `parser.hpp` e `parser.cpp`: Lógica do Parser Descendente Recursivo e tratamento de erros (Panic Mode).
- `main.cpp`: Ponto de entrada que gerencia a leitura de arquivos, aciona o Scanner, o Parser e exibe os resultados.

#### 2. Leitura de Arquivos Externos
O programa solicita ao usuário o nome de um arquivo (localizado na pasta `codigoRust/`) e processa o conteúdo real desse arquivo de forma dinâmica.

#### 3. Suporte Expandido para Rust
O sistema foi especializado para reconhecer padrões e a gramática da linguagem Rust, incluindo:
- **Palavras-chave e Blocos**: Suporte a `let`, `let mut`, `fn main() { ... }`, condicionais (`if`/`else`) e laços (`while`).
- **Símbolos e Expressões**: Reconhecimento e tratamento de precedência para matemática básica, além do operador de exclamação `!` (usado em macros como `println!`) e controle de múltiplos argumentos separados por vírgulas.
- **Tipagem Automática**: Suporte estendido ao uso e reconhecimento de Inteiros, Floats (Números Decimais) e Strings (`"exemplo"`).

#### 4. Exportação JSON e Tratamento de Erros
A exibição final constrói a árvore do código inteiro e exporta automaticamente no formato **JSON**, facilitando integrações visuais externas. Caso falte um ponto e vírgula ou caractere na sua sintaxe Rust, o *Panic Mode* informará o erro com a linha exata no console e continuará avaliando o restante do arquivo.

---

### 💻 Como Compilar e Rodar

Você pode compilar todos os arquivos juntos. O programa detecta se o executável gerado contém a palavra `scanner` no nome e, se contiver, oculta a árvore JSON automaticamente:

#### 1. Para gerar o `compiler.exe` (Exibe Tabela de Tokens + JSON):
```bash
g++ main.cpp lexico.cpp parser.cpp -o compiler.exe
```
**Para rodar:**
```bash
./compiler.exe
```

#### 2. Para gerar o `scanner.exe` (Exibe APENAS a Tabela de Tokens):
```bash
g++ main.cpp lexico.cpp parser.cpp -o scanner.exe
```
**Para rodar:**
```bash
./scanner.exe
```

*Nota: O programa buscará os arquivos dentro da pasta local `codigoRust/`. Tente rodar os arquivos de teste como o `soma.rs`.*

## Grupo: Anna Flavia Tsurushima, Giovanna Beatriz Ramos, Gustavo Pelissari, Igor Henrique Koga.
