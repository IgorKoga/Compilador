#ifndef AST_HPP
#define AST_HPP

#include <iostream>
#include <string>
#include <vector>
#include <memory>

// Escapa aspas para JSON
inline std::string escapeJson(const std::string& str) {
    std::string result = "";
    for (char c : str) {
        if (c == '"') result += "\\\"";
        else if (c == '\\') result += "\\\\";
        else if (c == '\n') result += "\\n";
        else result += c;
    }
    return result;
}

class ASTNode {
public:
    virtual ~ASTNode() = default;
    virtual std::string toJson() const = 0;  // Exigência de melhoria: Exportação em JSON
};


class Expr : public ASTNode {};
//Formato do json
class NumberExpr : public Expr { //representa numero inteiro
    std::string value; 
public:
    NumberExpr(std::string val) : value(val) {} //recebe inteiro e salva na memoria
    std::string toJson() const override { return "{\"type\": \"Number\", \"value\": " + value + "}"; }
};

class FloatExpr : public Expr { //representa numero float
    std::string value; 
public:
    FloatExpr(std::string val) : value(val) {} //recebe float e salva na memoria
    std::string toJson() const override { return "{\"type\": \"Float\", \"value\": " + value + "}"; }
};

class StringExpr : public Expr { //representa string
    std::string value; 
public:
    StringExpr(std::string val) : value(val) {} //recebe string e salva na memoria
    std::string toJson() const override { return "{\"type\": \"String\", \"value\": \"" + escapeJson(value) + "\"}"; }
};

class IdentifierExpr : public Expr { //representa variavel
    std::string name;
public:
    IdentifierExpr(std::string n) : name(n) {} //recebe nome da variavel
    std::string toJson() const override { return "{\"type\": \"Identifier\", \"name\": \"" + name + "\"}"; }
};

class BinaryExpr : public Expr { //operaçoes binarias(a + b; x <5; etc)
    std::string op; // "+", "-", "*", "/", "<", ">", "=="
    std::unique_ptr<Expr> left; // lado esquerdo do nó
    std::unique_ptr<Expr> right; // lado direito do nó
public: 
    BinaryExpr(std::unique_ptr<Expr> l, std::string o, std::unique_ptr<Expr> r)
        : left(std::move(l)), op(o), right(std::move(r)) {} //recebe operaçao binaria e salva na memoria
    std::string toJson() const override {
        return "{\"type\": \"BinaryExpr\", \"op\": \"" + op + "\", \"left\": " + left->toJson() + ", \"right\": " + right->toJson() + "}";
    }
};

// --- INSTRUÇÕES (Não retornam valores) ---
class Statement : public ASTNode {};

class LetDeclStmt : public Statement {
    std::string id;
    bool isMut; // verifica se a variavel é mutável
    std::unique_ptr<Expr> initializer; // pode ser nulo se for só "let x;"
public:
    LetDeclStmt(std::string name, bool mut, std::unique_ptr<Expr> init) 
        : id(name), isMut(mut), initializer(std::move(init)) {}
    std::string toJson() const override {
        std::string initJson = initializer ? initializer->toJson() : "null";
        std::string mutStr = isMut ? "true" : "false";
        return "{\"type\": \"LetDecl\", \"id\": \"" + id + "\", \"isMut\": " + mutStr + ", \"init\": " + initJson + "}";
    }
};

class AssignmentStmt : public Statement { //representa a atribuição de um valor a uma variável que já foi declarada
    std::string id; //nome da variavel
    std::unique_ptr<Expr> expr; //expressão que será atribuida
public:
    AssignmentStmt(std::string name, std::unique_ptr<Expr> e) : id(name), expr(std::move(e)) {}
    std::string toJson() const override {
        return "{\"type\": \"Assignment\", \"id\": \"" + id + "\", \"expr\": " + expr->toJson() + "}";
    }
};

class PrintlnStmt : public Statement { //representa a impressão de um valor na tela
    std::vector<std::unique_ptr<Expr>> args; //argumentos que serão impressos
public:
    PrintlnStmt(std::vector<std::unique_ptr<Expr>> a) : args(std::move(a)) {}
    std::string toJson() const override {
        std::string json = "{\"type\": \"Println\", \"args\": [";
        for (size_t i = 0; i < args.size(); ++i) {
            json += args[i]->toJson();
            if (i < args.size() - 1) json += ", ";
        }
        json += "]}";
        return json;
    }
};

class BlockStmt : public Statement { //representa um bloco de código
    std::vector<std::unique_ptr<Statement>> statements; //vetor de instruções tudo que está dentro de {}
public:
    std::vector<std::unique_ptr<Statement>> statements;

    void addStatement(std::unique_ptr<Statement> stmt) {
        statements.push_back(std::move(stmt));
    }

    std::string toJson() const override {
        std::string json = "{\"type\": \"Block\", \"statements\": [";
        for (size_t i = 0; i < statements.size(); ++i) {
            json += statements[i]->toJson();
            if (i < statements.size() - 1) json += ", ";
        }
        json += "]}";
        return json;
    }
};

class IfStmt : public Statement { //representa um if
    std::unique_ptr<Expr> condition; //condição do if
    std::unique_ptr<BlockStmt> thenBranch; //bloco de código que será executado se a condição for verdadeira
    std::unique_ptr<BlockStmt> elseBranch; //bloco de código que será executado se a condição for falsa
public:
    IfStmt(std::unique_ptr<Expr> cond, std::unique_ptr<BlockStmt> thenB, std::unique_ptr<BlockStmt> elseB)
        : condition(std::move(cond)), thenBranch(std::move(thenB)), elseBranch(std::move(elseB)) {}

    std::string toJson() const override {
        std::string elseJson = elseBranch ? elseBranch->toJson() : "null";
        return "{\"type\": \"IfStmt\", \"condition\": " + condition->toJson() + 
               ", \"thenBranch\": " + thenBranch->toJson() + ", \"elseBranch\": " + elseJson + "}";
    }
};

class WhileStmt : public Statement { //representa um while
    std::unique_ptr<Expr> condition; //condição do while
    std::unique_ptr<BlockStmt> body; //corpo do while
public:
    WhileStmt(std::unique_ptr<Expr> cond, std::unique_ptr<BlockStmt> b)
        : condition(std::move(cond)), body(std::move(b)) {}

    std::string toJson() const override {
        return "{\"type\": \"WhileStmt\", \"condition\": " + condition->toJson() + 
               ", \"body\": " + body->toJson() + "}";
    }
};

class FnDeclStmt : public Statement { //representa a declaração de uma função
    std::string name; //nome da função
    std::unique_ptr<BlockStmt> body; //corpo da função
public:
    FnDeclStmt(std::string n, std::unique_ptr<BlockStmt> b)
        : name(n), body(std::move(b)) {}

    std::string toJson() const override {
        return "{\"type\": \"FnDecl\", \"name\": \"" + name + "\", \"body\": " + body->toJson() + "}";
    }
};

// Nó raiz do programa
class Program : public ASTNode { //representa o programa inteiro
    std::vector<std::unique_ptr<Statement>> statements; //vetor de instruções
public:
    void addStatement(std::unique_ptr<Statement> stmt) { statements.push_back(std::move(stmt)); }
    std::string toJson() const override { //converte o programa inteiro para json
        std::string json = "{\n  \"type\": \"Program\",\n  \"body\": [\n";
        for (size_t i = 0; i < statements.size(); ++i) {
            json += "    " + statements[i]->toJson();
            if (i < statements.size() - 1) json += ",";
            json += "\n";
        }
        json += "  ]\n}";
        return json;
    }
};

#endif
