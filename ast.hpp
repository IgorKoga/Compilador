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

// Classe base para todos os nós da AST
class ASTNode {
public:
    virtual ~ASTNode() = default;
    virtual std::string toJson() const = 0; 
};

// --- EXPRESSÕES (Retornam valores) ---
class Expr : public ASTNode {};

class NumberExpr : public Expr {
    std::string value; 
public:
    NumberExpr(std::string val) : value(val) {}
    std::string toJson() const override { return "{\"type\": \"Number\", \"value\": " + value + "}"; }
};

class FloatExpr : public Expr {
    std::string value; 
public:
    FloatExpr(std::string val) : value(val) {}
    std::string toJson() const override { return "{\"type\": \"Float\", \"value\": " + value + "}"; }
};

class StringExpr : public Expr {
    std::string value; 
public:
    StringExpr(std::string val) : value(val) {}
    std::string toJson() const override { return "{\"type\": \"String\", \"value\": \"" + escapeJson(value) + "\"}"; }
};

class IdentifierExpr : public Expr {
    std::string name;
public:
    IdentifierExpr(std::string n) : name(n) {}
    std::string toJson() const override { return "{\"type\": \"Identifier\", \"name\": \"" + name + "\"}"; }
};

class BinaryExpr : public Expr {
    std::string op; // "+", "-", "*", "/", "<", ">", "=="
    std::unique_ptr<Expr> left;
    std::unique_ptr<Expr> right;
public:
    BinaryExpr(std::unique_ptr<Expr> l, std::string o, std::unique_ptr<Expr> r)
        : left(std::move(l)), op(o), right(std::move(r)) {}
    std::string toJson() const override {
        return "{\"type\": \"BinaryExpr\", \"op\": \"" + op + "\", \"left\": " + left->toJson() + ", \"right\": " + right->toJson() + "}";
    }
};

// --- INSTRUÇÕES (Não retornam valores) ---
class Statement : public ASTNode {};

class LetDeclStmt : public Statement {
    std::string id;
    bool isMut;
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

class AssignmentStmt : public Statement {
    std::string id;
    std::unique_ptr<Expr> expr;
public:
    AssignmentStmt(std::string name, std::unique_ptr<Expr> e) : id(name), expr(std::move(e)) {}
    std::string toJson() const override {
        return "{\"type\": \"Assignment\", \"id\": \"" + id + "\", \"expr\": " + expr->toJson() + "}";
    }
};

class PrintlnStmt : public Statement {
    std::vector<std::unique_ptr<Expr>> args;
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

class BlockStmt : public Statement {
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

class IfStmt : public Statement {
    std::unique_ptr<Expr> condition;
    std::unique_ptr<BlockStmt> thenBranch;
    std::unique_ptr<BlockStmt> elseBranch; // null se não houver else
public:
    IfStmt(std::unique_ptr<Expr> cond, std::unique_ptr<BlockStmt> thenB, std::unique_ptr<BlockStmt> elseB)
        : condition(std::move(cond)), thenBranch(std::move(thenB)), elseBranch(std::move(elseB)) {}

    std::string toJson() const override {
        std::string elseJson = elseBranch ? elseBranch->toJson() : "null";
        return "{\"type\": \"IfStmt\", \"condition\": " + condition->toJson() + 
               ", \"thenBranch\": " + thenBranch->toJson() + ", \"elseBranch\": " + elseJson + "}";
    }
};

class WhileStmt : public Statement {
    std::unique_ptr<Expr> condition;
    std::unique_ptr<BlockStmt> body;
public:
    WhileStmt(std::unique_ptr<Expr> cond, std::unique_ptr<BlockStmt> b)
        : condition(std::move(cond)), body(std::move(b)) {}

    std::string toJson() const override {
        return "{\"type\": \"WhileStmt\", \"condition\": " + condition->toJson() + 
               ", \"body\": " + body->toJson() + "}";
    }
};

class FnDeclStmt : public Statement {
    std::string name;
    std::unique_ptr<BlockStmt> body;
public:
    FnDeclStmt(std::string n, std::unique_ptr<BlockStmt> b)
        : name(n), body(std::move(b)) {}

    std::string toJson() const override {
        return "{\"type\": \"FnDecl\", \"name\": \"" + name + "\", \"body\": " + body->toJson() + "}";
    }
};

// Nó raiz do programa
class Program : public ASTNode {
    std::vector<std::unique_ptr<Statement>> statements;
public:
    void addStatement(std::unique_ptr<Statement> stmt) { statements.push_back(std::move(stmt)); }
    std::string toJson() const override {
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
