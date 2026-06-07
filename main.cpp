#include <iostream>
#include <string>
#include <iomanip>
#include "lexico.hpp"
#include "parser.hpp"
#include "ast.hpp"
#include <fstream> //lê arquivos
#include <sstream> ///lê o arquivo como string
using namespace std;

int main() {

 // 1. Nome do arquivo fixo
 string folder = "codigoRust/"; //endereço padrão
 string fileName;
 cout << "Digite o nome/endereco do arquivo: ";
 cin >> fileName;
 fileName = folder + fileName; //concatena o endereço padrão com o nome do arquivo

 // 2. Tenta abrir o arquivo
 ifstream file(fileName);
 if (!file.is_open()) {
     cerr << "Erro: O arquivo '" << fileName << "' nao foi encontrado na pasta!" << endl;
     return 1;
 }

 // 3. Lê o conteúdo
 stringstream buffer;
 buffer << file.rdbuf();
 string code = buffer.str();

// Cria o scanner com o código de entrada
  Scanner scanner(code);

  try {
    // Cabeçalho da tabela
    cout << endl;
    cout << string(60, '-') << endl;
    cout << left << setw(20) << "Token" 
         << setw(30) << "Lexema" 
         << "Linha" << endl;
    cout << string(60, '-') << endl;

    // Continua analisando até encontrar o fim da entrada
    // while (token.type != TokenType::T_EOF) {
    //   // Exibe os dados formatados em colunas
    //   cout << left << setw(20) << tokenTypeToString(token.type)
    //        << setw(30) << token.lexeme
    //        << token.line << endl;

    //   // Busca o próximo token
    //   token = scanner.nextToken();
    // }

    // cout << string(60, '-') << endl;
    // cout << "Fim da analise lexica." << endl;

    Parser parser(scanner);
    auto ast = parser.parseProgram();

    cout << endl << string(60, '-') << endl;
    cout << "Analise sintatica concluida com sucesso!" << endl;
    cout << "Arvore Sintatica Abstrata (JSON):" << endl;
    cout << ast->toJson() << endl;

  } catch (exception &e) {

    // Caso ocorra erro léxico, a mensagem será exibida aqui
    cerr << e.what() << endl;
  }

  return 0;
}
