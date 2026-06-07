#include "lexico.hpp"
#include <iomanip>
#include <iostream>
#include <string>

#ifndef SCANNER_ONLY
#include "ast.hpp"
#include "parser.hpp"
#endif

#include <fstream> // lê arquivos
#include <sstream> // lê o arquivo como string
using namespace std;

int main(int argc, char *argv[]) {
  // Verifica se o executável foi chamado como "scanner"
  string execName = argv[0];
  bool isScannerOnly = (execName.find("scanner") != string::npos);

#ifdef SCANNER_ONLY
  isScannerOnly = true;
#endif

  // 1. Nome do arquivo fixo
  string folder = "codigoRust/"; // endereço padrão
  string fileName;
  cout << "Digite o nome/endereco do arquivo: ";
  cin >> fileName;
  fileName =
      folder + fileName; // concatena o endereço padrão com o nome do arquivo

  // 2. Tenta abrir o arquivo
  ifstream file(fileName);
  if (!file.is_open()) {
    cerr << "Erro: O arquivo '" << fileName << "' nao foi encontrado na pasta!"
         << endl;
    return 1;
  }

  // 3. Lê o conteúdo
  stringstream buffer;
  buffer << file.rdbuf();
  string code = buffer.str();

  // Cria o scanner para a análise léxica (tabela de tokens)
  Scanner lexScanner(code);

  try {
    // Cabeçalho da tabela
    cout << endl;
    cout << string(60, '-') << endl;
    cout << left << setw(20) << "Token" << setw(30) << "Lexema" << "Linha"
         << endl;
    cout << string(60, '-') << endl;

    // Continua analisando até encontrar o fim da entrada para exibir a tabela
    Token token = lexScanner.nextToken();
    while (token.type != TokenType::T_EOF) {
      // Exibe os dados formatados em colunas
      cout << left << setw(20) << tokenTypeToString(token.type) << setw(30)
           << token.lexeme << token.line << endl;

      // Busca o próximo token
      token = lexScanner.nextToken();
    }

    cout << string(60, '-') << endl;
    cout << "Fim da analise lexica." << endl;

#ifndef SCANNER_ONLY
    if (!isScannerOnly) {
      // Cria um scanner separado para o parser
      Scanner parserScanner(code);
      Parser parser(parserScanner);
      auto ast = parser.parseProgram();

      cout << endl << string(60, '-') << endl;
      cout << "Analise sintatica concluida com sucesso!" << endl;
      cout << "Arvore Sintatica Abstrata (JSON):" << endl;
      cout << ast->toJson() << endl;
    }
#endif

  } catch (exception &e) {
    // Caso ocorra erro, a mensagem será exibida aqui
    cerr << e.what() << endl;
  }

  return 0;
}
