#include "parser.hpp"
#include "lexico.hpp"
#include <sstream>
#include <stdexcept>

// Recebe a lista de tokens e inicia o cursor
Parser::Parser(const std::vector<Token> &tokenList)
    : tokens(tokenList), current(0) {}

// ------------------- Métodos auxiliares de navegação -------------------

// Retorna o token atual sem avançar.
Token Parser::peek() const {
  // Garantimos que não estamos além do fim; se estivermos, retornamos um
  // token EOF fictício (geralmente definido no lexer).
  if (isAtEnd())
    return Token{TokenType::T_EOF, "", -1};
  return tokens[current];
}

// Retorna o token já consumido anteriormente.
Token Parser::previous() const {
  if (current == 0)
    throw std::runtime_error(
        "Tentativa de acessar token anterior antes do início.");
  return tokens[current - 1];
}

// Verifica se já consumimos todos os tokens.
bool Parser::isAtEnd() const {
  return current >= tokens.size() || tokens[current].type == TokenType::T_EOF;
}

// Avança o cursor e devolve o token consumido.
Token Parser::advance() {
  if (!isAtEnd())
    ++current;
  return previous(); // token que acabou de ser consumido
}

// Se o token atual tem o tipo esperado, consome‑o; caso contrário, lança erro.
Token Parser::match(TokenType type, const std::string &errorMessage) {
  if (isAtEnd())
    error(peek(), "Fim inesperado da entrada. " + errorMessage);
  if (peek().type == type)
    return advance();
  error(peek(), "Token inesperado: esperado '" + tokenTypeToString(type) +
                    "'. " + errorMessage);
  // Nunca chega aqui, mas retornamos um token de fallback para satisfazer o
  // compilador.
  return Token{TokenType::T_EOF, "", -1};
}

// Cria uma exceção com a linha do token e mensagem informativa.
void Parser::error(const Token &token, const std::string &message) {
  std::ostringstream oss;
  oss << "Erro sintático na linha " << token.line << ": " << message;
  throw std::runtime_error(oss.str());
}

// tokenTypeToString is defined in lexico.cpp; we use the declaration from
// lexico.hpp

// -----------------------------------------------------------------------------
// Métodos de Parsing
// -----------------------------------------------------------------------------

// Programa inteiro → lista de declarações/expressões
std::unique_ptr<ProgramNode> Parser::parseProgram();

// Declaração ou comando (Statement)
std::unique_ptr<StatementNode> Parser::parseStatement();

// Declaração de variável (let, mut, int …)
std::unique_ptr<DeclarationNode> Parser::parseDeclaration();

// Atribuição (identificador = expressão)
std::unique_ptr<AssignmentNode> Parser::parseAssignment();

// Comando de impressão (println! …)
std::unique_ptr<PrintNode> Parser::parsePrint();

// Estrutura condicional
std::unique_ptr<IfNode> Parser::parseIf();

// Laço while
std::unique_ptr<WhileNode> Parser::parseWhile();

// Expressão ⇐→ nível de prioridade (operadores lógicos/aritméticos)
std::unique_ptr<ExpressionNode> Parser::parseExpression();
std::unique_ptr<ExpressionNode> Parser::parseEquality();
std::unique_ptr<ExpressionNode> Parser::parseComparison();
std::unique_ptr<ExpressionNode> Parser::parseTerm();   // + e -
std::unique_ptr<ExpressionNode> Parser::parseFactor(); // * e /
std::unique_ptr<ExpressionNode> Parser::parseUnary();  // - e !
std::unique_ptr<ExpressionNode>
Parser::parsePrimary(); // literals, id, parênteses
