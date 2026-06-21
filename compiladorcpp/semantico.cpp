#include "semantico.hpp"

SymbolTable::SymbolTable(std::vector<std::string>& warns) : warnings(warns) {
    enterScope(); // Inicializa o escopo global
}

SymbolTable::~SymbolTable() {
    while (!scopes.empty()) {
        exitScope();
    }
}

void SymbolTable::enterScope() {
    scopes.push_back({});
}

void SymbolTable::exitScope() {
    if (scopes.empty()) return;
    // Melhoria: Variáveis não utilizadas (Relatório de warnings)
    for (const auto& pair : scopes.back()) {
        if (!pair.second->isUsed && pair.second->type != "function") {
            warnings.push_back("Aviso Semantico: variavel '" + pair.second->name + "' declarada mas nunca utilizada.");
        }
    }
    scopes.pop_back();
}

bool SymbolTable::insert(const std::string& name, const std::string& type, bool isMut, bool isInitialized) {
    if (scopes.empty()) return false;
    // Melhoria: Declaração Duplicada e Controle de escopo aninhado
    if (scopes.back().count(name) > 0) {
        return false; 
    }
    scopes.back()[name] = std::make_shared<Symbol>(name, type, isMut, isInitialized);
    return true;
}

std::shared_ptr<Symbol> SymbolTable::lookup(const std::string& name) {
    for (auto it = scopes.rbegin(); it != scopes.rend(); ++it) {
        if (it->count(name) > 0) {
            return (*it)[name];
        }
    }
    return nullptr;
}

std::string SymbolTable::toJson() const {
    std::string json = "{\n  \"symbol_table\": [\n";
    for (size_t i = 0; i < scopes.size(); ++i) {
        json += "    {\n      \"scope\": " + std::to_string(i) + ",\n      \"symbols\": [\n";
        int count = 0;
        for (const auto& pair : scopes[i]) {
            json += "        {\"name\": \"" + pair.second->name + "\", \"type\": \"" + pair.second->type 
                 + "\", \"isMut\": " + (pair.second->isMut ? "true" : "false") 
                 + ", \"isUsed\": " + (pair.second->isUsed ? "true" : "false") + "}";
            if (++count < scopes[i].size()) json += ",";
            json += "\n";
        }
        json += "      ]\n    }";
        if (i < scopes.size() - 1) json += ",";
        json += "\n";
    }
    json += "  ]\n}";
    return json;
}

SemanticAnalyzer::SemanticAnalyzer() : symTable(warnings) {}

void SemanticAnalyzer::analyze(Program* program) {
    if (!program) return;
    for (auto& stmt : program->statements) {
        analyzeStatement(stmt.get());
    }
}

void SemanticAnalyzer::report() {
    std::cout << "\n--- Relatorio do Analisador Semantico ---\n";
    if (errors.empty() && warnings.empty()) {
        std::cout << "Analise Semantica concluida com sucesso.\n";
    } else {
        for (const auto& w : warnings) {
            std::cout << w << "\n";
        }
        for (const auto& e : errors) {
            std::cout << "Erro Semantico: " << e << "\n";
        }
        std::cout << "\nQuantidade total de erros encontrados: " << errors.size() << "\n";
    }
    std::cout << "-----------------------------------------\n";
}

void SemanticAnalyzer::analyzeStatement(Statement* stmt) {
    if (!stmt) return;

    if (auto letDecl = dynamic_cast<LetDeclStmt*>(stmt)) {
        std::string inferredType = "unknown";
        bool isInit = false;
        if (letDecl->initializer) {
            isInit = true;
            analyzeExpr(letDecl->initializer.get());
            inferredType = inferType(letDecl->initializer.get());
        }
        
        if (!symTable.insert(letDecl->id, inferredType, letDecl->isMut, isInit)) {
            errors.push_back("variavel '" + letDecl->id + "' ja declarada.");
        }
    } 
    else if (auto assignStmt = dynamic_cast<AssignmentStmt*>(stmt)) {
        analyzeExpr(assignStmt->expr.get());
        std::string exprType = inferType(assignStmt->expr.get());
        
        auto sym = symTable.lookup(assignStmt->id);
        if (!sym) {
            errors.push_back("variavel '" + assignStmt->id + "' nao declarada.");
        } else {
            // Melhoria: Constantes
            if (!sym->isMut && sym->isInitialized) {
                errors.push_back("atribuicao a variavel constante (imutavel) '" + assignStmt->id + "'.");
            }
            // Verificação de Tipos
            if (sym->type != "unknown" && exprType != "unknown" && sym->type != exprType) {
                errors.push_back("atribuicao incompativel.");
            }
            sym->isInitialized = true;
        }
    }
    else if (auto printStmt = dynamic_cast<PrintlnStmt*>(stmt)) {
        for (auto& arg : printStmt->args) {
            analyzeExpr(arg.get());
        }
    }
    else if (auto blockStmt = dynamic_cast<BlockStmt*>(stmt)) {
        symTable.enterScope();
        for (auto& s : blockStmt->statements) {
            analyzeStatement(s.get());
        }
        symTable.exitScope();
    }
    else if (auto ifStmt = dynamic_cast<IfStmt*>(stmt)) {
        analyzeExpr(ifStmt->condition.get());
        
        symTable.enterScope();
        analyzeStatement(ifStmt->thenBranch.get());
        symTable.exitScope();

        if (ifStmt->elseBranch) {
            symTable.enterScope();
            analyzeStatement(ifStmt->elseBranch.get());
            symTable.exitScope();
        }
    }
    else if (auto whileStmt = dynamic_cast<WhileStmt*>(stmt)) {
        analyzeExpr(whileStmt->condition.get());
        
        symTable.enterScope();
        analyzeStatement(whileStmt->body.get());
        symTable.exitScope();
    }
    else if (auto fnDecl = dynamic_cast<FnDeclStmt*>(stmt)) {
        // Melhoria: Funções
        if (!symTable.insert(fnDecl->name, "function", false, true)) {
            errors.push_back("funcao '" + fnDecl->name + "' ja declarada.");
        }
        symTable.enterScope();
        analyzeStatement(fnDecl->body.get());
        symTable.exitScope();
    }
}

void SemanticAnalyzer::analyzeExpr(Expr* expr) {
    if (!expr) return;

    if (auto binExpr = dynamic_cast<BinaryExpr*>(expr)) {
        analyzeExpr(binExpr->left.get());
        analyzeExpr(binExpr->right.get());
        
        std::string leftType = inferType(binExpr->left.get());
        std::string rightType = inferType(binExpr->right.get());
        
        if (leftType != "unknown" && rightType != "unknown") {
            if (binExpr->op == "+" || binExpr->op == "-" || binExpr->op == "*" || binExpr->op == "/") {
                if (leftType == "string" || rightType == "string") {
                    if (binExpr->op != "+") {
                         errors.push_back("operacao '" + binExpr->op + "' incompativel entre tipos " + leftType + " e " + rightType + ".");
                    } else if (leftType != "string" || rightType != "string") {
                         errors.push_back("operacao '+' incompativel entre tipos " + leftType + " e " + rightType + ".");
                    }
                }
            } else if (binExpr->op == "<" || binExpr->op == ">" || binExpr->op == "==") {
                if (leftType != rightType && !( (leftType=="int" && rightType=="float") || (leftType=="float" && rightType=="int") )) {
                    errors.push_back("operacao relacional incompativel entre tipos " + leftType + " e " + rightType + ".");
                }
            }
        }

        // Melhoria: Verificação de divisão por zero
        if (binExpr->op == "/") {
            if (auto numRight = dynamic_cast<NumberExpr*>(binExpr->right.get())) {
                if (numRight->value == "0") {
                    errors.push_back("divisao por zero detectavel em tempo de compilacao.");
                }
            } else if (auto floatRight = dynamic_cast<FloatExpr*>(binExpr->right.get())) {
                if (floatRight->value == "0.0" || floatRight->value == "0") {
                    errors.push_back("divisao por zero detectavel em tempo de compilacao.");
                }
            }
        }
    }
    else if (auto idExpr = dynamic_cast<IdentifierExpr*>(expr)) {
        auto sym = symTable.lookup(idExpr->name);
        if (!sym) {
            errors.push_back("variavel '" + idExpr->name + "' nao declarada.");
        } else {
            sym->isUsed = true;
            // Melhoria: Uso antes da inicialização
            if (!sym->isInitialized) {
                warnings.push_back("Aviso Semantico: variavel '" + idExpr->name + "' utilizada antes de receber valor.");
            }
        }
    }
}

// Melhoria: Inferência de Tipos Automática
std::string SemanticAnalyzer::inferType(Expr* expr) {
    if (!expr) return "unknown";
    
    if (dynamic_cast<NumberExpr*>(expr)) return "int";
    if (dynamic_cast<FloatExpr*>(expr)) return "float";
    if (dynamic_cast<StringExpr*>(expr)) return "string";
    if (auto idExpr = dynamic_cast<IdentifierExpr*>(expr)) {
        auto sym = symTable.lookup(idExpr->name);
        if (sym) return sym->type;
    }
    if (auto binExpr = dynamic_cast<BinaryExpr*>(expr)) {
        std::string leftType = inferType(binExpr->left.get());
        std::string rightType = inferType(binExpr->right.get());
        
        if (leftType == "unknown" || rightType == "unknown") return "unknown";
        
        if (leftType == rightType) return leftType;
        if ((leftType == "float" && rightType == "int") || (leftType == "int" && rightType == "float")) {
            return "float"; // Coerção segura de int para float
        }
    }
    return "unknown";
}
