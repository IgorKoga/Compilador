#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    T_LET, T_MUT, T_INT, T_IF, T_ELSE, T_WHILE, T_PRINTLN, T_EXCL, T_FN, T_VIRG, // Palavras reservadas
    T_ID, T_NUM, T_FLOAT, T_STRING, // Identificadores (nomes de variáveis), números e textos (strings)
    T_ASSIGN, T_EQ, // Operadores de atribuição (=) e comparação de igualdade (==)
    T_PLUS, T_MINUS, T_MULT, T_DIV, // Operadores aritméticos (+, -, *, /)
    T_LT, T_GT,  // Operadores relacionais (<, >)
    T_LPAREN, T_RPAREN, T_LBRACE, T_RBRACE, // Símbolos de agrupamento: parênteses e chaves
    T_SEMICOLON, // Delimitador de instrução (ponto e vírgula)
    T_EOF, // Fim do arquivo/entrada (End Of File)
}

// guarda o tipo, o texto exato (lexema) e a linha
#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType, // O tipo de token
    pub lexeme: String,    // O texto em si 
    pub line: usize,       // A linha onde ele apareceu
}

impl Token {
    /// Função para criar um Token novo mais facil
    pub fn new(r#type: TokenType, lexeme: String, line: usize) -> Self {
        Token { r#type, lexeme, line }
    }
}

// Estrutura principal, percorre o codigo caractere por caractere
pub struct Scanner {
    input: Vec<char>,                     // código fonte transformado em uma lista de caracteres
    pos: usize,                           // Posição atual de leitura 
    line: usize,                          // Linha atual do código
    keywords: HashMap<String, TokenType>, // Dicionário para verificar se uma palavra é uma palavra reservada
}

impl Scanner {
    // Inicializa a maquina do scanner e cadastra as palavras reservadas
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        // Cadastrando as palavras que não pode usar como nome de variável
        keywords.insert("int".to_string(), TokenType::T_INT);
        keywords.insert("if".to_string(), TokenType::T_IF);
        keywords.insert("else".to_string(), TokenType::T_ELSE);
        keywords.insert("while".to_string(), TokenType::T_WHILE);
        keywords.insert("println".to_string(), TokenType::T_PRINTLN);
        keywords.insert("fn".to_string(), TokenType::T_FN);
        keywords.insert("let".to_string(), TokenType::T_LET);
        keywords.insert("mut".to_string(), TokenType::T_MUT);

        Scanner {
            input: source.chars().collect(), // Transforma a string em um vetor de caracteres
            pos: 0,                          // Começa a ler da posição zero
            line: 1,                         // Começa na linha 1
            keywords,
        }
    }

    // "Espia" o caractere atual sem avançar a posição de leitura
    // Retorna '\0' se já chegou no final do código
    fn peek(&self) -> char {
        if self.pos >= self.input.len() {
            '\0'
        } else {
            self.input[self.pos]
        }
    }

    // Pega o caractere atual e avança a posição de leitura em 1 passo
    fn next(&mut self) -> char {
        let c = self.peek();
        if c != '\0' {
            self.pos += 1;
        }
        c
    }

    // Ignora espaços em branco e quebras de linha
    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() && self.peek() != '\0' {
            if self.next() == '\n' {
                self.line += 1; // Se pularmos um 'enter', atualizamos o contador de linha
            }
        }
    }

    // Pula todos os caracteres até o final da linha. Usado quando encontramos um comentário de linha `//`.
    fn skip_comment(&mut self) {
        while self.peek() != '\n' && self.peek() != '\0' {
            self.next();
        }
    }

    // Pula um comentário de múltiplas linhas `/* ... */`. Continua lendo até achar o fechamento `*/`.
    fn skip_multiline_comment(&mut self) {
        while self.peek() != '\0' {
            if self.peek() == '\n' {
                self.line += 1; // Continua contando as linhas mesmo dentro do comentário
            }

            if self.peek() == '*' {
                self.next();
                if self.peek() == '/' {
                    self.next(); // Achou o `*/`, consome a barra e encerra o pulo
                    return;
                }
            } else {
                self.next();
            }
        }
        // Se o código acabou e o comentário nunca fechou, dá erro:
        panic!("Erro Lexico: Comentario multilinha nao fechado na linha {}", self.line);
    }

    // Fica lendo até o número acabar
    // Verifica se é número inteiro (T_NUM) ou quebrado/ponto flutuante (T_FLOAT)
    fn scan_number(&mut self, start: char) -> Token {
        let mut buffer = String::new();
        buffer.push(start); // Guarda o primeiro dígito já lido
        let mut is_float = false;

        // Continua enquanto for número ou ponto
        while self.peek().is_digit(10) || self.peek() == '.' {
            if self.peek() == '.' {
                if is_float {
                    //se ja foi achado um ponto antes da erro:
                    panic!("Erro Lexico: Multiplos pontos decimais na linha {}", self.line);
                }
                is_float = true; // Marca que virou um número flutuante
            }
            buffer.push(self.next());
        }

        let token_type = if is_float { TokenType::T_FLOAT } else { TokenType::T_NUM };
        Token::new(token_type, buffer, self.line)
    }

    // LÊ nomes de variáveis ou palavras reservadas
    fn scan_identifier(&mut self, start: char) -> Token {
        let mut buffer = String::new();
        buffer.push(start);

        // Nomes podem conter letras, números e underline 
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            buffer.push(self.next());
        }

        // Consulta o dicionário: "Essa palavra é um comando do sistema?"
        if let Some(&token_type) = self.keywords.get(&buffer) {
            Token::new(token_type, buffer, self.line) // Sim, é uma palavra reservada
        } else {
            Token::new(TokenType::T_ID, buffer, self.line) // é só o nome de uma variável/função criada pelo usuário
        }
    }

    // Lê textos entre aspas (strings)
    fn scan_string(&mut self) -> Token {
        let mut buffer = String::new();

        // Lê tudo até encontrar as aspas fechando ou o arquivo acabar
        while self.peek() != '"' && self.peek() != '\0' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            buffer.push(self.next());
        }

        if self.peek() == '"' {
            self.next(); // Consome a aspa de fechamento para ela não aparecer no token
            Token::new(TokenType::T_STRING, buffer, self.line)
        } else {
            // Faltou fechar aspas
            panic!("Erro Lexico: String nao fechada na linha {}", self.line);
        }
    }

    // Processa a leitura e devolve o proximo Token valido ou acusa erro lexico
    pub fn nextToken(&mut self) -> Token {
        self.skip_whitespace(); // Limpa os espaços mortos antes de começar a procurar

        // Se chegou no fim, devolve o token de Fim de Arquivo (EOF)
        if self.pos >= self.input.len() {
            return Token::new(TokenType::T_EOF, String::new(), self.line);
        }

        let c = self.next(); // Pega a primeira letra da próxima coisa

        // Se for um dígito, deve ser um número
        if c.is_digit(10) {
            return self.scan_number(c);
        }

        // Se for uma letra ou _, deve ser uma variável ou palavra-chave
        if c.is_alphabetic() || c == '_' {
            return self.scan_identifier(c);
        }

        // Se for aspas, deve ser um texto string
        if c == '"' {
            return self.scan_string();
        }

        // Se não for nada acima, é um símbolo. Descobre qual é usando:
        match c {
            ',' => Token::new(TokenType::T_VIRG, ",".to_string(), self.line),
            '!' => Token::new(TokenType::T_EXCL, "!".to_string(), self.line),
            '+' => Token::new(TokenType::T_PLUS, "+".to_string(), self.line),
            '-' => Token::new(TokenType::T_MINUS, "-".to_string(), self.line),
            '*' => Token::new(TokenType::T_MULT, "*".to_string(), self.line),
            '/' => {
                // Como uma barra pode ser divisão (/) ou comentário (// ou /*), a gente espia a próxima letra:
                if self.peek() == '/' {
                    self.next();
                    self.skip_comment(); // É comentário de linha, pula tudo e recomeça a busca
                    self.nextToken()
                } else if self.peek() == '*' {
                    self.next();
                    self.skip_multiline_comment(); // É comentário multilinhas, pula o bloco e recomeça a busca
                    self.nextToken()
                } else {
                    Token::new(TokenType::T_DIV, "/".to_string(), self.line) // É só divisão mesmo
                }
            }
            '=' => {
                // Pode ser atribuição (=) ou igualdade (==)
                if self.peek() == '=' {
                    self.next();
                    Token::new(TokenType::T_EQ, "==".to_string(), self.line)
                } else {
                    Token::new(TokenType::T_ASSIGN, "=".to_string(), self.line)
                }
            }
            '<' => Token::new(TokenType::T_LT, "<".to_string(), self.line),
            '>' => Token::new(TokenType::T_GT, ">".to_string(), self.line),
            '(' => Token::new(TokenType::T_LPAREN, "(".to_string(), self.line),
            ')' => Token::new(TokenType::T_RPAREN, ")".to_string(), self.line),
            '{' => Token::new(TokenType::T_LBRACE, "{".to_string(), self.line),
            '}' => Token::new(TokenType::T_RBRACE, "}".to_string(), self.line),
            ';' => Token::new(TokenType::T_SEMICOLON, ";".to_string(), self.line),
            _ => panic!("Erro Lexico: caractere invalido '{}' na linha {}", c, self.line), // Símbolo não existe na linguagem
        }
    }
}

// devolve o nome dele como uma String de texto.
// Serve pra quando a gente for mandar imprimir "T_LET" certinho na tela.
pub fn tokenTypeToString(r#type: TokenType) -> String {
    match r#type {
        TokenType::T_LET => "T_LET".to_string(),
        TokenType::T_MUT => "T_MUT".to_string(),
        TokenType::T_VIRG => "T_VIRG".to_string(),
        TokenType::T_INT => "T_INT".to_string(),
        TokenType::T_IF => "T_IF".to_string(),
        TokenType::T_ELSE => "T_ELSE".to_string(),
        TokenType::T_WHILE => "T_WHILE".to_string(),
        TokenType::T_PRINTLN => "T_PRINTLN".to_string(),
        TokenType::T_EXCL => "T_EXCL".to_string(),
        TokenType::T_FN => "T_FN".to_string(),
        TokenType::T_ID => "T_ID".to_string(),
        TokenType::T_NUM => "T_NUM".to_string(),
        TokenType::T_FLOAT => "T_FLOAT".to_string(),
        TokenType::T_STRING => "T_STRING".to_string(),
        TokenType::T_ASSIGN => "T_ASSIGN".to_string(),
        TokenType::T_EQ => "T_EQ".to_string(),
        TokenType::T_PLUS => "T_PLUS".to_string(),
        TokenType::T_MINUS => "T_MINUS".to_string(),
        TokenType::T_MULT => "T_MULT".to_string(),
        TokenType::T_DIV => "T_DIV".to_string(),
        TokenType::T_LT => "T_LT".to_string(),
        TokenType::T_GT => "T_GT".to_string(),
        TokenType::T_LPAREN => "T_LPAREN".to_string(),
        TokenType::T_RPAREN => "T_RPAREN".to_string(),
        TokenType::T_LBRACE => "T_LBRACE".to_string(),
        TokenType::T_RBRACE => "T_RBRACE".to_string(),
        TokenType::T_SEMICOLON => "T_SEMICOLON".to_string(),
        TokenType::T_EOF => "T_EOF".to_string(),
    }
}