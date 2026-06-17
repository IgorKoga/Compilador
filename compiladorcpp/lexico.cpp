#include <cctype>
#include "lexico.hpp"
#include <stdexcept>
#include <string>
#include <unordered_map>
using namespace std;

Scanner::Scanner(string source) : input(source), pos(0), line(1) {
  // Cadastro das palavras reservadas da linguagem
  keywords["int"] = TokenType::T_INT;
  keywords["if"] = TokenType::T_IF;
  keywords["else"] = TokenType::T_ELSE;
  keywords["while"] = TokenType::T_WHILE;
  keywords["println"] = TokenType::T_PRINTLN; // cadastro de println do rust
  keywords["fn"] = TokenType::T_FN; // cadastro de fn do rust
  keywords["let"] = TokenType::T_LET; //cadastro de let do rust
  keywords["mut"] = TokenType::T_MUT; //cadastro de mut do rust
}

  // Retorna o caractere atual sem avançar na leitura
  // Se chegar ao fim da entrada, retorna '\0'
  char Scanner::peek() {
    if (pos >= input.length()) {
      return '\0';
    }
    return input[pos];
  }

  // Retorna o caractere atual e avança para a próxima posição.
  char Scanner::next() {
    char c = peek();
    if (c != '\0') {
      pos++;
    }
    return c;
  }

  // Ignora espaços em branco, tabulações e quebras de linha.
  // Sempre que encontra '\n', incrementa o contador de linhas.
  void Scanner::skipWhitespace() {
    while (isspace(peek())) {
      if (next() == '\n') {
        line++;
      }
    }
  }

  // Ignora comentários de uma linha iniciados por "//"
  // Continua lendo até o final da linha ou fim da entrada
  void Scanner::skipComment() {
    while (peek() != '\n' && peek() != '\0') {
      next();
    }
  }

  // Ignora comentários multilinha (/* ... */)
  void Scanner::skipMultilineComment() {
    while (peek() != '\0') {
      if (peek() == '\n') line++;

      if (peek() == '*') {
        next();
        if (peek() == '/') {
          next(); // Consome a barra final
          return;
        }
      } else {
        next();
      }
    }
    throw runtime_error("Erro Lexico: Comentario multilinha nao fechado na linha " + to_string(line));
  }

  // Lê um número inteiro/float a partir do primeiro dígito já encontrado.
  Token Scanner::scanNumber(char start) {
    string buffer;
    buffer += start;
    bool isFloat = false;

    while (isdigit(peek()) || peek() == '.') {
      if (peek() == '.') {
        if (isFloat) throw runtime_error("Erro Lexico: Multiplos pontos decimais na linha " + to_string(line));
        isFloat = true;
      }
      buffer += next();
    }

    return Token(isFloat ? TokenType::T_FLOAT : TokenType::T_NUM, buffer, line);
  }

  // Lê identificadores ou palavras reservadas.
  // Um identificador pode conter letras, números e underscore.
  Token Scanner::scanIdentifier(char start) {
    string buffer;
    // Adiciona o primeiro caractere já lido
    buffer += start;
    // Continua lendo enquanto o padrão for válido para identificador
    while (isalnum(peek()) || peek() == '_') {
      buffer += next();
    }
    // Verifica se o texto lido é uma palavra reservada
    if (keywords.count(buffer)) {
      return Token(keywords[buffer], buffer, line);
    }
    // Caso contrário, trata como identificador comum
    return Token(TokenType::T_ID, buffer, line);
  }

  // Método para ler strings entre aspas " "
  Token Scanner::scanString() {
    string buffer;
    // O loop continua até encontrar outra aspa ou fim do arquivo
    while (peek() != '"' && peek() != '\0') {
      if (peek() == '\n') line++;
      buffer += next();
    }

    if (peek() == '"') {
      next(); // consome a aspa de fechamento
      return Token(TokenType::T_STRING, buffer, line);
    }
    throw runtime_error("Erro Lexico: String nao fechada na linha " + to_string(line));
  }

  // Método principal do scanner:

  // retorna o próximo token encontrado na entrada.
  Token Scanner::nextToken() {

    // Primeiro, ignora espaços em branco
    skipWhitespace();

    // Se chegou ao fim da entrada, retorna EOF
    if (pos >= input.length()) {
      return Token(TokenType::T_EOF, "", line);
    }

    // Lê o próximo caractere
    char c = next();

    // Se começar com dígito, tenta formar um número
    if (isdigit(c)) {
      return scanNumber(c);
    }

    // Se começar com letra ou underscore, tenta formar identificador
    if (isalpha(c) || c == '_') {
      return scanIdentifier(c);
    }

    // Se começar com aspas, tenta formar uma string
    if (c == '"') {
      return scanString();
    }

    // Analisa símbolos e operadores

    switch (c) {
    case ',':
      return Token(TokenType::T_VIRG, ",", line);
    case '!': // possibilita a leitura do println! - Igor
      return Token(TokenType::T_EXCL, "!", line);
    case '+':
      return Token(TokenType::T_PLUS, "+", line);

    case '-':
      return Token(TokenType::T_MINUS, "-", line);

    case '*':
      return Token(TokenType::T_MULT, "*", line);

    case '/':

      // Se houver outro '/', então é comentário de linha
      if (peek() == '/') {
        next();             // consome o segundo '/'
        skipComment();      // ignora o restante da linha
        return nextToken(); // busca o próximo token válido
      }
      if (peek() == '*') { // Comentário multilinha (/*)
      next();
      skipMultilineComment();
      return nextToken();
      }
      return Token(TokenType::T_DIV, "/", line);

    case '=':

      // Verifica se é "==" (igualdade)
      if (peek() == '=') {
        next();
        return Token(TokenType::T_EQ, "==", line);
      }

      // Caso contrário, é "=" (atribuição)
      return Token(TokenType::T_ASSIGN, "=", line);

    case '<':
      return Token(TokenType::T_LT, "<", line);

    case '>':
      return Token(TokenType::T_GT, ">", line);

    case '(':
      return Token(TokenType::T_LPAREN, "(", line);

    case ')':
      return Token(TokenType::T_RPAREN, ")", line);

    case '{':
      return Token(TokenType::T_LBRACE, "{", line);

    case '}':
      return Token(TokenType::T_RBRACE, "}", line);

    case ';':
      return Token(TokenType::T_SEMICOLON, ";", line);

    default:

      // Se encontrar um caractere que não pertence à linguagem,
      // lança erro léxico informando o símbolo e a linha.
      throw runtime_error("Erro Lexico: caractere invalido '" + string(1, c) +
                          "' na linha " + to_string(line));
    }
  }
// Função auxiliar para converter o enum TokenType em texto.
// Isso facilita a exibição dos tokens no terminal.
string tokenTypeToString(TokenType type) {

  switch (type) {

    case TokenType::T_LET:
    return "T_LET";
    case TokenType::T_MUT:
    return "T_MUT";
    case TokenType::T_VIRG:
    return "T_VIRG";
    case TokenType::T_INT:
    return "T_INT";
    case TokenType::T_IF:
    return "T_IF";
    case TokenType::T_ELSE:
    return "T_ELSE";
    case TokenType::T_WHILE:
    return "T_WHILE";
    case TokenType::T_PRINTLN:
    return "T_PRINTLN";
    case TokenType::T_EXCL:
    return "T_EXCL";
    case TokenType::T_FN:
    return "T_FN";
    case TokenType::T_ID:
    return "T_ID";
    case TokenType::T_NUM:
    return "T_NUM";
    case TokenType::T_FLOAT:
    return "T_FLOAT";
    case TokenType::T_STRING:
    return "T_STRING";
    case TokenType::T_ASSIGN:
    return "T_ASSIGN";
    case TokenType::T_EQ:
    return "T_EQ";
    case TokenType::T_PLUS:
    return "T_PLUS";
    case TokenType::T_MINUS:
    return "T_MINUS";
    case TokenType::T_MULT:
    return "T_MULT";
    case TokenType::T_DIV:
    return "T_DIV";
    case TokenType::T_LT:
    return "T_LT";
    case TokenType::T_GT:
    return "T_GT";
    case TokenType::T_LPAREN:
    return "T_LPAREN";
    case TokenType::T_RPAREN:
    return "T_RPAREN";
    case TokenType::T_LBRACE:
    return "T_LBRACE";
    case TokenType::T_RBRACE:
    return "T_RBRACE";
    case TokenType::T_SEMICOLON:
    return "T_SEMICOLON";
    case TokenType::T_EOF:
    return "T_EOF";

    default:
    return "UNKNOWN";
    }
}
