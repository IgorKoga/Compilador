#![allow(non_snake_case)] // Permite que variáveis e funções tenham nomes não-padrão (snake_case)
#![allow(dead_code)] // Permite que funções e variáveis não sejam usadas

// Importações presumidas dos módulos do analisador léxico e da AST.
// Ajustar esses caminhos caso a estrutura de módulos do projeto mude.
use super::lexico::{Scanner, Token, TokenType};
use super::ast::{
    Expr, Statement, Program, BlockStmt, LetDeclStmt, AssignmentStmt,
    PrintlnStmt, IfStmt, WhileStmt, FnDeclStmt, BinaryExpr, NumberExpr,
    FloatExpr, StringExpr, IdentifierExpr,
};

/// Estrutura do Analisador Sintático (Parser)
pub struct Parser<'a> {
    scanner: &'a mut Scanner,
    currentToken: Token,
}

impl<'a> Parser<'a> {
    // Inicializa o parser com a referência ao scanner e avança para o primeiro token
    pub fn new(scanner: &'a mut Scanner) -> Self {
        let mut parser = Parser {
            scanner,
            // Inicializa com um token temporário que será imediatamente substituído
            currentToken: Token::new(TokenType::T_EOF, String::new(), 0),
        };
        parser.advance(); // avança para o primeiro token
        parser
    }

    // Avança para o próximo token retornado pelo scanner
    fn advance(&mut self) {
        self.currentToken = self.scanner.nextToken();
    }

    // Verifica se o token atual é do tipo esperado e avança se for
    fn r#match(&mut self, expected: TokenType) -> bool {
        if self.currentToken.r#type == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    // Consome o token atual se for do tipo esperado, senão lança um erro
    fn consume(&mut self, expected: TokenType, errorMessage: &str) -> Result<(), String> {
        if self.currentToken.r#type == expected {
            self.advance();
            Ok(())
        } else {
            Err(self.error(errorMessage))
        }
    }

    // Retorna uma mensagem formatada de erro sintático com o número da linha correspondente
    fn error(&self, message: &str) -> String {
        format!("Erro Sintatico na linha {}: {}", self.currentToken.line, message)
    }

    // Sincroniza o parser após um erro, avançando os tokens até encontrar um ponto seguro (sincronização por Panic Mode)
    fn synchronize(&mut self) {
        self.advance();
        while self.currentToken.r#type != TokenType::T_EOF {
            if self.currentToken.r#type == TokenType::T_SEMICOLON {
                return;
            }

            match self.currentToken.r#type {
                TokenType::T_LET
                | TokenType::T_IF
                | TokenType::T_WHILE
                | TokenType::T_PRINTLN
                | TokenType::T_FN => return,
                _ => {}
            }
            self.advance();
        }
    }

    // Analisa o programa inteiro, recuperando-se de erros stmt por stmt
    pub fn parseProgram(&mut self) -> Box<Program> {
        let mut program = Box::new(Program::new());
        while self.currentToken.r#type != TokenType::T_EOF {
            match self.parseStatement() {
                Ok(statement) => {
                    program.addStatement(statement);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    self.synchronize();
                }
            }
        }
        program
    }

    // Analisa um bloco de código delimitado por chaves '{}'
    fn parseBlock(&mut self) -> Result<Box<BlockStmt>, String> {
        self.consume(TokenType::T_LBRACE, "Esperado '{' no inicio do bloco.")?;
        let mut block = Box::new(BlockStmt::new());
        while self.currentToken.r#type != TokenType::T_RBRACE && self.currentToken.r#type != TokenType::T_EOF {
            block.addStatement(self.parseStatement()?);
        }
        self.consume(TokenType::T_RBRACE, "Esperado '}' no final do bloco.")?;
        Ok(block)
    }

    // Analisa uma instrução identificando a palavra-chave inicial
    fn parseStatement(&mut self) -> Result<Box<Statement>, String> {
        if self.currentToken.r#type == TokenType::T_LET {
            return self.parseDeclaration();
        }
        if self.currentToken.r#type == TokenType::T_PRINTLN {
            return self.parsePrintStmt();
        }
        if self.currentToken.r#type == TokenType::T_IF {
            return self.parseIfStmt();
        }
        if self.currentToken.r#type == TokenType::T_WHILE {
            return self.parseWhileStmt();
        }
        if self.currentToken.r#type == TokenType::T_FN {
            return self.parseFnDecl();
        }

        // Se não é nenhuma das keywords acima, tenta uma atribuição (assignment)
        self.parseAssignment()
    }

    // Analisa uma declaração de variável ('let')
    fn parseDeclaration(&mut self) -> Result<Box<Statement>, String> {
        self.consume(TokenType::T_LET, "Esperado 'let'.")?;

        let mut isMut = false;
        if self.r#match(TokenType::T_MUT) {
            isMut = true;
        }

        let varName = self.currentToken.lexeme.clone();
        self.consume(TokenType::T_ID, "Esperado nome da variavel apos 'let'.")?;

        let mut initExpr = None;
        if self.r#match(TokenType::T_ASSIGN) {
            initExpr = Some(self.parseExpression()?);
        }
        self.consume(TokenType::T_SEMICOLON, "Esperado ';' apos declaracao de variavel.")?;

        Ok(Box::new(LetDeclStmt::new(varName, isMut, initExpr)))
    }

    // Analisa uma atribuição de valor a uma variável existente
    fn parseAssignment(&mut self) -> Result<Box<Statement>, String> {
        let varName = self.currentToken.lexeme.clone();
        self.consume(TokenType::T_ID, "Esperado identificador para atribuicao ou instrucao valida.")?;

        self.consume(TokenType::T_ASSIGN, "Esperado '=' na atribuicao.")?;
        let expr = self.parseExpression()?;
        self.consume(TokenType::T_SEMICOLON, "Esperado ';' apos atribuicao.")?;

        Ok(Box::new(AssignmentStmt::new(varName, expr)))
    }

    // Analisa uma instrução de impressão 'println!'
    fn parsePrintStmt(&mut self) -> Result<Box<Statement>, String> {
        self.consume(TokenType::T_PRINTLN, "Esperado 'println'.")?;
        self.consume(TokenType::T_EXCL, "Esperado '!' apos println.")?;
        self.consume(TokenType::T_LPAREN, "Esperado '(' apos '!'.")?;

        let mut args = Vec::new();
        if self.currentToken.r#type != TokenType::T_RPAREN {
            args.push(self.parseExpression()?);
            while self.r#match(TokenType::T_VIRG) {
                args.push(self.parseExpression()?);
            }
        }

        self.consume(TokenType::T_RPAREN, "Esperado ')' apos argumentos do println.")?;
        self.consume(TokenType::T_SEMICOLON, "Esperado ';' no final da instrucao.")?;

        Ok(Box::new(PrintlnStmt::new(args)))
    }

    // Analisa uma instrução condicional 'if/else'
    fn parseIfStmt(&mut self) -> Result<Box<Statement>, String> {
        self.consume(TokenType::T_IF, "Esperado 'if'.")?;

        let condition = self.parseExpression()?;
        let thenBranch = self.parseBlock()?;

        let mut elseBranch = None;
        if self.r#match(TokenType::T_ELSE) {
            elseBranch = Some(self.parseBlock()?);
        }

        Ok(Box::new(IfStmt::new(condition, thenBranch, elseBranch)))
    }

    // Analisa uma instrução de laço 'while'
    fn parseWhileStmt(&mut self) -> Result<Box<Statement>, String> {
        self.consume(TokenType::T_WHILE, "Esperado 'while'.")?;

        let condition = self.parseExpression()?;
        let body = self.parseBlock()?;

        Ok(Box::new(WhileStmt::new(condition, body)))
    }

    // Analisa uma declaração de função 'fn'
    fn parseFnDecl(&mut self) -> Result<Box<Statement>, String> {
        self.consume(TokenType::T_FN, "Esperado 'fn'.")?;

        let name = self.currentToken.lexeme.clone();
        self.consume(TokenType::T_ID, "Esperado nome da funcao.")?;

        self.consume(TokenType::T_LPAREN, "Esperado '(' apos nome da funcao.")?;
        self.consume(TokenType::T_RPAREN, "Esperado ')' apos argumentos da funcao.")?;

        let body = self.parseBlock()?;

        Ok(Box::new(FnDeclStmt::new(name, body)))
    }

    // Analisa uma expressão genérica (que inicia resolvendo comparações)
    fn parseExpression(&mut self) -> Result<Box<Expr>, String> {
        self.parseComparison()
    }

    // Analisa expressões de comparação (<, >, ==)
    fn parseComparison(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.parseTerm()?;

        while self.currentToken.r#type == TokenType::T_LT
            || self.currentToken.r#type == TokenType::T_GT
            || self.currentToken.r#type == TokenType::T_EQ
        {
            let op = self.currentToken.lexeme.clone();
            self.advance();
            let right = self.parseTerm()?;
            expr = Box::new(BinaryExpr::new(expr, op, right));
        }

        Ok(expr)
    }

    // Analisa termos de soma (+) e subtração (-)
    fn parseTerm(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.parseFactor()?;

        while self.currentToken.r#type == TokenType::T_PLUS
            || self.currentToken.r#type == TokenType::T_MINUS
        {
            let op = self.currentToken.lexeme.clone();
            self.advance();
            let right = self.parseFactor()?;
            expr = Box::new(BinaryExpr::new(expr, op, right));
        }

        Ok(expr)
    }

    // Analisa fatores de expressão (números inteiros/decimais, strings, identificadores e expressões entre parênteses)
    fn parseFactor(&mut self) -> Result<Box<Expr>, String> {
        if self.currentToken.r#type == TokenType::T_NUM {
            let expr = Box::new(NumberExpr::new(self.currentToken.lexeme.clone()));
            self.advance();
            return Ok(expr);
        }
        if self.currentToken.r#type == TokenType::T_FLOAT {
            let expr = Box::new(FloatExpr::new(self.currentToken.lexeme.clone()));
            self.advance();
            return Ok(expr);
        }
        if self.currentToken.r#type == TokenType::T_STRING {
            let expr = Box::new(StringExpr::new(self.currentToken.lexeme.clone()));
            self.advance();
            return Ok(expr);
        }
        if self.currentToken.r#type == TokenType::T_ID {
            let expr = Box::new(IdentifierExpr::new(self.currentToken.lexeme.clone()));
            self.advance();
            return Ok(expr);
        }
        if self.currentToken.r#type == TokenType::T_LPAREN {
            self.advance();
            let expr = self.parseExpression()?;
            self.consume(TokenType::T_RPAREN, "Esperado ')' apos expressao.")?;
            return Ok(expr);
        }

        let err_msg = format!("Fator invalido: {}", self.currentToken.lexeme);
        Err(self.error(&err_msg))
    }
}
