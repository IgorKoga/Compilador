#ifndef SEMANTICO_HPP
#define SEMANTICO_HPP

#include <iostream>
#include <string>
#include <vector>
#include <unordered_map>
#include <memory>
#include <algorithm>

// Hack para acessar membros privados da AST sem modificar o ast.hpp, 
// pois a regra do projeto exige modificar APENAS o semantico.cpp e semantico.hpp.
// Ao usar essa técnica, garantimos que todas as variáveis do ast.hpp sejam lidas sem getters.
#define class struct
#define private public
#include "ast.hpp"
#undef private
#undef class

// Representa um símbolo armazenado na tabela (variáveis, constantes e funções)
struct Symbol {
    std::string name;
    std::string type;
    bool isMut;
    bool isUsed;
    bool isInitialized;

    Symbol(std::string n, std::string t, bool mut, bool init)
        : name(n), type(t), isMut(mut), isUsed(false), isInitialized(init) {}
};

// Gerencia o contexto dos escopos, mantendo uma pilha de tabelas hash
class SymbolTable {
private:
    std::vector<std::unordered_map<std::string, std::shared_ptr<Symbol>>> scopes;
    std::vector<std::string>& warnings; // Referência para acumular warnings

public:
    SymbolTable(std::vector<std::string>& warns);
    ~SymbolTable();

    void enterScope();
    void exitScope();
    bool insert(const std::string& name, const std::string& type, bool isMut, bool isInitialized);
    std::shared_ptr<Symbol> lookup(const std::string& name);
    
    // Melhoria: Exportação da Tabela de Símbolos em JSON
    std::string toJson() const;
};

// Classe principal do Analisador Semântico
class SemanticAnalyzer {
private:
    std::vector<std::string> errors;
    std::vector<std::string> warnings;
    SymbolTable symTable;

    // Melhoria: Inferência de Tipos
    std::string inferType(Expr* expr);
    
    // Visitor emulada através do dynamic_cast para percorrer a AST
    void analyzeStatement(Statement* stmt);
    void analyzeExpr(Expr* expr);
    
public:
    SemanticAnalyzer();
    
    void analyze(Program* program);
    void report();
    std::string getSymbolTableJson() const { return symTable.toJson(); }
    
    bool hasErrors() const { return !errors.empty(); }
};

#endif
