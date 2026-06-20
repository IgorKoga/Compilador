#ifndef PARSER_HPP
#define PARSER_HPP

#include "lexico.hpp"
#include "ast.hpp"
#include <vector>
#include <memory>
#include <stdexcept>

class Parser {
private:
    Scanner& scanner;
    Token currentToken;

    // Funções auxiliares obrigatórias
    void advance();
    bool match(TokenType expected);
    void consume(TokenType expected, const std::string& errorMessage);
    void error(const std::string& message);
    void synchronize(); 

    // Funções de parsing por regra
    std::unique_ptr<BlockStmt> parseBlock();
    std::unique_ptr<Statement> parseStatement();
    std::unique_ptr<Statement> parseDeclaration(); 
    std::unique_ptr<Statement> parseAssignment();
    std::unique_ptr<Statement> parsePrintStmt();  
    std::unique_ptr<Statement> parseIfStmt();
    std::unique_ptr<Statement> parseWhileStmt();
    std::unique_ptr<Statement> parseFnDecl();

    // Regras de expressões (associatividade esquerda)
    std::unique_ptr<Expr> parseExpression(); 
    std::unique_ptr<Expr> parseComparison(); 
    std::unique_ptr<Expr> parseTerm();       
    std::unique_ptr<Expr> parseFactor();     

public:
    Parser(Scanner& scan);
    std::unique_ptr<Program> parseProgram();
};

#endif
