#ifndef lexico_hpp
#define lexico_hpp

#include <string>
#include <unordered_map>

enum class TokenType {//Todos os tipos de token reconhecidos pelo analisador léxico
  T_LET, T_MUT, T_INT, T_IF, T_ELSE, T_WHILE, T_PRINTLN, T_EXCL, T_FN, T_VIRG, // Palavras reservadas
  T_ID, T_NUM, T_FLOAT, T_STRING, // Identificadores, números e strings
  T_ASSIGN, T_EQ, // Operadores de atribuição e comparação
  T_PLUS, T_MINUS, T_MULT, T_DIV, // Operadores aritméticos
  T_LT, T_GT,  // Operadores relacionais
  T_LPAREN, T_RPAREN, T_LBRACE, T_RBRACE, // Símbolos de agrupamento
  T_SEMICOLON, // Delimitador de instrução
  T_EOF // Fim do arquivo/entrada
};


struct Token { // Estrutura que representa um token encontrado na análise léxica.
  TokenType type; // Tipo do token
  std::string lexeme;  // Texto exato encontrado na entrada
  int line;       // Linha em que o token foi encontrado
  // Construtor para inicializar um token
  Token(TokenType t, std::string l, int ln) : type(t), lexeme(l), line(ln) {}
};

class Scanner { //responsável por percorrer o código-fonte e transformar caracteres em tokens.

private:
  std::string input; // Código-fonte de entrada
  size_t pos;   // Posição atual de leitura
  int line;     // Linha atual da análise

  // Mapa de palavras reservadas:
  // associa texto (ex.: "if") ao tipo do token correspondente
  std::unordered_map<std::string, TokenType> keywords;

  //definição apenas das assinaturas
  char peek();
  char next();
  void skipWhitespace();
  void skipComment();
  void skipMultilineComment();
  Token scanNumber(char start);
  Token scanIdentifier(char start);
  Token scanString(); // Método para ler strings

public:
    Scanner(std::string source);
    Token nextToken();
};

std::string tokenTypeToString(TokenType type);

#endif
