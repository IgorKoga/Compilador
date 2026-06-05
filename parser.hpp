#ifndef PARSER_HPP
#define PARSER_HPP

#include "ast.hpp"
#include "lexico.hpp"
#include <memory>
#include <string>
#include <vector>


class Parser {
private:
  std::vector<Token> tokens; // Lista de tokens gerada pelo scanner
  size_t current;            // Posição do token atual sendo analisado

  // --- Métodos Auxiliares de Navegação ---

  // Retorna o token atual sem consumi-lo
  Token peek() const;

  // Retorna o token anterior
  Token previous() const;

  // Verifica se chegamos ao fim dos tokens
  bool isAtEnd() const;

  // Avança para o próximo token e o retorna
  Token advance();

  // Verifica se o token atual é do tipo esperado.
  // Se for, consome e avança. Se não, lança erro com a mensagem fornecida.
  Token match(TokenType type, const std::string &errorMessage);

  // Lança um erro sintático formatado com a linha correspondente
  void error(const Token &token, const std::string &message);

public:
  // O construtor recebe a lista de tokens do scanner
  Parser(const std::vector<Token> &tokenList);

  // --- Métodos de Parsing (declararemos no Passo 7) ---
};

#endif // PARSER_HPP
