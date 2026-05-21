#include <iostream>
#include <string>
#include "lexico.cpp"

int main() {

  // Código-fonte de exemplo que será analisado.
  // A string raw (R"( ... )") permite escrever o texto em múltiplas linhas.
  string code = R"(

int soma = 10 + 20;

if (soma == 30) {
print(soma);
}

// comentario ignorado)";

  // Cria o scanner com o código de entrada
  Scanner scanner(code);

  try {

    // Lê o primeiro token
    Token token = scanner.nextToken();

    // Continua analisando até encontrar o fim da entrada
    while (token.type != TokenType::T_EOF) {

      // Exibe o tipo do token, o lexema e a linha correspondente
      cout << tokenTypeToString(token.type) << " -> " << token.lexeme
           << " (linha " << token.line << ")" << endl;

      // Busca o próximo token
      token = scanner.nextToken();
    }

    cout << endl;
    cout << "Fim da analise lexica." << endl;

  } catch (exception &e) {

    // Caso ocorra erro léxico, a mensagem será exibida aqui
    cerr << e.what() << endl;
  }

  return 0;
}
