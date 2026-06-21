#include "lexico.hpp"
#include <iomanip>
#include <iostream>
#include <string>

#ifndef LEXICO_ONLY
#include "ast.hpp"
#include "sintatico.hpp"
#include "semantico.hpp"
#endif

#include <fstream> // lê arquivos
#include <limits>  // necessário para limpar o buffer do cin
#include <sstream> // lê o arquivo como string
using namespace std;

int main(int argc, char *argv[]) {
  // Verifica se o executável foi chamado como "lexico" ou "sintatico"
  string execName = argv[0];
  bool isLexicoOnly = (execName.find("lexico") != string::npos);
  bool isSintaticoOnly = (execName.find("sintatico") != string::npos);

#ifdef LEXICO_ONLY
  isLexicoOnly = true;
#endif

  // Se não for especificamente nenhum (ex: compilador.exe ou main.exe), executa
  // ambos
  bool runBoth = !isLexicoOnly && !isSintaticoOnly;

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
    cout << "\nPressione Enter para fechar a janela..." << endl;
    cin.ignore(numeric_limits<streamsize>::max(), '\n');
    cin.get();
    return 1;
  }

  // 3. Lê o conteúdo
  stringstream buffer;
  buffer << file.rdbuf();
  string code = buffer.str();

  // Cria o scanner para a análise léxica (tabela de tokens)
  Scanner lexScanner(code);

  try {
    if (isLexicoOnly || runBoth) {
      if (runBoth) {
        cout << "\n============================================================"
             << endl;
        cout << "                  1. FASE LEXICA (TOKENS)                   "
             << endl;
        cout << "============================================================"
             << endl;
      } else {
        cout << endl;
      }

      // Cabeçalho da tabela
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
    }

#ifndef LEXICO_ONLY
    if (isSintaticoOnly || runBoth) {
      if (runBoth) {
        cout << "\n============================================================"
             << endl;
        cout << "                  2. FASE SINTATICA (ARVORE E JSON)         "
             << endl;
        cout << "============================================================"
             << endl;
      }

      // Cria um scanner separado para o parser
      Scanner parserScanner(code);
      Parser parser(parserScanner);
      auto ast = sintatico.parseProgram();

      // Exibe a representação estruturada da árvore sintática (AST) no terminal
      cout << "\n--- Representacao da Arvore Sintatica (AST) ---" << endl;
      ast->print(); // Chamada para exibir a árvore de forma hierárquica e
                    // identada
      cout << "-----------------------------------------------" << endl;

      // Determina o nome do arquivo JSON a ser gerado com base no arquivo de
      // entrada
      size_t lastSlash = fileName.find_last_of("/\\");
      string baseName = (lastSlash == string::npos)
                            ? fileName
                            : fileName.substr(lastSlash + 1);

      // Remove a extensão original (.rs) e adiciona .json
      size_t lastDot = baseName.find_last_of(".");
      string jsonFileName = (lastDot == string::npos)
                                ? baseName + ".json"
                                : baseName.substr(0, lastDot) + ".json";
      string jsonPath =
          "json/" +
          jsonFileName; // Caminho da pasta de destino para os arquivos JSON

      // Cria e abre o arquivo JSON para escrita
      ofstream jsonFile(jsonPath);
      if (jsonFile.is_open()) {
        jsonFile << ast->toJson(); // Grava a representação em JSON gerada
                                   // recursivamente pela AST
        jsonFile.close();
        cout << "\n[Sucesso] Arquivo AST JSON salvo em: " << jsonPath << endl;
      } else {
        cerr << "\n[Erro] Nao foi possivel criar o arquivo JSON em: "
             << jsonPath << endl;
      }

      if (runBoth) {
        cout << "============================================================"
             << endl;
      }
      cout << "Fim da analise sintatica." << endl;

      if (runBoth) {
        cout << "\n============================================================"
             << endl;
        cout << "                  3. FASE SEMANTICA                         "
             << endl;
        cout << "============================================================"
             << endl;
      }

      // Executa a analise
      SemanticAnalyzer semanticAnalyzer;
      semanticAnalyzer.analyze(ast.get());
      semanticAnalyzer.report();

      // Gravar o JSON da Tabela de Simbolos gerado pela analise semantica
      string symJsonPath = "json/symtable_" + jsonFileName;
      ofstream symJsonFile(symJsonPath);
      if (symJsonFile.is_open()) {
        symJsonFile << semanticAnalyzer.getSymbolTableJson();
        symJsonFile.close();
        cout << "\n[Sucesso] Arquivo Tabela de Simbolos JSON salvo em: " << symJsonPath << endl;
      } else {
        cerr << "\n[Erro] Nao foi possivel criar o arquivo JSON da Tabela de Simbolos em: "
             << symJsonPath << endl;
      }

      if (runBoth) {
        cout << "============================================================"
             << endl;
      }
      cout << "Fim da analise semantica." << endl;
    }
#endif

  } catch (exception &e) {
    // Caso ocorra erro, a mensagem será exibida aqui
    cerr << e.what() << endl;
  }

  cout << "\nPressione Enter para fechar a janela..." << endl;
  cin.ignore(numeric_limits<streamsize>::max(), '\n');
  cin.get();

  return 0;
}
