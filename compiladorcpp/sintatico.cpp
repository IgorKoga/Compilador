#include "sintatico.hpp"
#include <iostream>

//Inicializa o parser
Parser::Parser(Scanner& scan) : scanner(scan), currentToken(TokenType::T_EOF, "", 0) {
    advance(); //avança para o primeiro token
}

//Avança para o próximo token
void Parser::advance() {
    currentToken = scanner.nextToken();
}

//Verifica se o token atual é do tipo esperado e avança se for
bool Parser::match(TokenType expected) {
    if (currentToken.type == expected) {
        advance();
        return true;
    }
    return false;
}

//Consome o token atual se for do tipo esperado, senão lança um erro
void Parser::consume(TokenType expected, const std::string& errorMessage) {
    if (currentToken.type == expected) {
        advance();
    } else {
        error(errorMessage);
    }
}

    //Lança um erro sintático com a mensagem fornecida
void Parser::error(const std::string& message) {
    throw std::runtime_error("Erro Sintatico na linha " + std::to_string(currentToken.line) + ": " + message);
}

//Sincroniza o parser após um erro, avançando até encontrar um token válido
void Parser::synchronize() {
    advance();
    while (currentToken.type != TokenType::T_EOF) {
        if (currentToken.type == TokenType::T_SEMICOLON) return;

        switch(currentToken.type) {
            case TokenType::T_LET:
            case TokenType::T_IF:
            case TokenType::T_WHILE:
            case TokenType::T_PRINTLN:
            case TokenType::T_FN:
                return;
            default:
                break;
        }
        advance();
    }
}

//Analisa o programa inteiro
std::unique_ptr<Program> Parser::parseProgram() {
    auto program = std::make_unique<Program>();
    while (currentToken.type != TokenType::T_EOF) {
        try {
            program->addStatement(parseStatement());
        } catch (const std::runtime_error& e) {
            std::cerr << e.what() << std::endl;
            synchronize();
        }
    }
    return program;
}

//Analisa um bloco de código
std::unique_ptr<BlockStmt> Parser::parseBlock() {
    consume(TokenType::T_LBRACE, "Esperado '{' no inicio do bloco.");
    auto block = std::make_unique<BlockStmt>();
    while (currentToken.type != TokenType::T_RBRACE && currentToken.type != TokenType::T_EOF) {
        block->addStatement(parseStatement());
    }
    consume(TokenType::T_RBRACE, "Esperado '}' no final do bloco.");
    return block;
}

    //Analisa uma instrução
std::unique_ptr<Statement> Parser::parseStatement() {
    if (currentToken.type == TokenType::T_LET) return parseDeclaration();
    if (currentToken.type == TokenType::T_PRINTLN) return parsePrintStmt();
    if (currentToken.type == TokenType::T_IF) return parseIfStmt();
    if (currentToken.type == TokenType::T_WHILE) return parseWhileStmt();
    if (currentToken.type == TokenType::T_FN) return parseFnDecl();

    // Se não é nenhuma das keywords acima, tenta um assignment
    return parseAssignment();
}

    //Analisa uma declaração
std::unique_ptr<Statement> Parser::parseDeclaration() {
    consume(TokenType::T_LET, "Esperado 'let'.");

    bool isMut = false;
    if (match(TokenType::T_MUT)) {
        isMut = true;
    }

    std::string varName = currentToken.lexeme;
    consume(TokenType::T_ID, "Esperado nome da variavel apos 'let'.");

    std::unique_ptr<Expr> initExpr = nullptr;
    if (match(TokenType::T_ASSIGN)) {
        initExpr = parseExpression();
    }
    consume(TokenType::T_SEMICOLON, "Esperado ';' apos declaracao de variavel.");

    return std::make_unique<LetDeclStmt>(varName, isMut, std::move(initExpr));
}
    //Analisa uma atribuição
std::unique_ptr<Statement> Parser::parseAssignment() {
    std::string varName = currentToken.lexeme;
    consume(TokenType::T_ID, "Esperado identificador para atribuicao ou instrucao valida.");

    consume(TokenType::T_ASSIGN, "Esperado '=' na atribuicao.");
    auto expr = parseExpression();
    consume(TokenType::T_SEMICOLON, "Esperado ';' apos atribuicao.");

    return std::make_unique<AssignmentStmt>(varName, std::move(expr));
}

    //Analisa uma instrução de impressão
std::unique_ptr<Statement> Parser::parsePrintStmt() {
    consume(TokenType::T_PRINTLN, "Esperado 'println'.");
    consume(TokenType::T_EXCL, "Esperado '!' apos println.");
    consume(TokenType::T_LPAREN, "Esperado '(' apos '!'.");

    std::vector<std::unique_ptr<Expr>> args;
    if (currentToken.type != TokenType::T_RPAREN) {
        args.push_back(parseExpression());
        while (match(TokenType::T_VIRG)) {
            args.push_back(parseExpression());
        }
    }

    consume(TokenType::T_RPAREN, "Esperado ')' apos argumentos do println.");
    consume(TokenType::T_SEMICOLON, "Esperado ';' no final da instrucao.");

    return std::make_unique<PrintlnStmt>(std::move(args));
}

    //Analisa uma instrução condicional
std::unique_ptr<Statement> Parser::parseIfStmt() {
    consume(TokenType::T_IF, "Esperado 'if'.");

    auto condition = parseExpression();
    auto thenBranch = parseBlock();

    std::unique_ptr<BlockStmt> elseBranch = nullptr;
    if (match(TokenType::T_ELSE)) {
        elseBranch = parseBlock();
    }

    return std::make_unique<IfStmt>(std::move(condition), std::move(thenBranch), std::move(elseBranch));
}

    //Analisa uma instrução while
std::unique_ptr<Statement> Parser::parseWhileStmt() {
    consume(TokenType::T_WHILE, "Esperado 'while'.");

    auto condition = parseExpression();
    auto body = parseBlock();

    return std::make_unique<WhileStmt>(std::move(condition), std::move(body));
}

    //Analisa uma declaração de função
std::unique_ptr<Statement> Parser::parseFnDecl() {
    consume(TokenType::T_FN, "Esperado 'fn'.");

    std::string name = currentToken.lexeme;
    consume(TokenType::T_ID, "Esperado nome da funcao.");

    consume(TokenType::T_LPAREN, "Esperado '(' apos nome da funcao.");
    consume(TokenType::T_RPAREN, "Esperado ')' apos argumentos da funcao.");

    auto body = parseBlock();

    return std::make_unique<FnDeclStmt>(name, std::move(body));
}

    //Analisa uma expressão
std::unique_ptr<Expr> Parser::parseExpression() {
    return parseComparison();
}

    //Analisa uma comparação
std::unique_ptr<Expr> Parser::parseComparison() {
    auto expr = parseTerm();

    while (currentToken.type == TokenType::T_LT ||
           currentToken.type == TokenType::T_GT ||
           currentToken.type == TokenType::T_EQ) {

        std::string op = currentToken.lexeme;
        advance();
        auto right = parseTerm();
        expr = std::make_unique<BinaryExpr>(std::move(expr), op, std::move(right));
    }

    return expr;
}

    // Analisa soma e subtração, mas primeiro tenta ler multiplicaões para ordem de operações
std::unique_ptr<Expr> Parser::parseTerm() {
    auto expr = parseFactor();

    while (currentToken.type == TokenType::T_PLUS ||
           currentToken.type == TokenType::T_MINUS) {

        std::string op = currentToken.lexeme;
        advance();
        auto right = parseFactor();
        expr = std::make_unique<BinaryExpr>(std::move(expr), op, std::move(right));
    }

    return expr;
}

    //Analisa fator (função responsável por analisar números, strings, identificadores e parênteses)
    //Se achar parenteses resolve primeiro
std::unique_ptr<Expr> Parser::parseFactor() {
    if (currentToken.type == TokenType::T_NUM) {
        auto expr = std::make_unique<NumberExpr>(currentToken.lexeme);
        advance();
        return expr;
    }
    if (currentToken.type == TokenType::T_FLOAT) {
        auto expr = std::make_unique<FloatExpr>(currentToken.lexeme);
        advance();
        return expr;
    }
    if (currentToken.type == TokenType::T_STRING) {
        auto expr = std::make_unique<StringExpr>(currentToken.lexeme);
        advance();
        return expr;
    }
    if (currentToken.type == TokenType::T_ID) {
        auto expr = std::make_unique<IdentifierExpr>(currentToken.lexeme);
        advance();
        return expr;
    }
    if (currentToken.type == TokenType::T_LPAREN) {
        advance();
        auto expr = parseExpression();
        consume(TokenType::T_RPAREN, "Esperado ')' apos expressao.");
        return expr;
    }

    error("Fator invalido: " + currentToken.lexeme);
    return nullptr;
}
