#ifndef AST_HPP
#define AST_HPP

#include <memory>  // Necessário para std::unique_ptr
#include <sstream> //Adicionado para formatação de strings/JSON
#include <string>
#include <vector> //Necessario para o uso de ponteiros inteligentes

// A raiz de qualquer elemento que exista na nossa árvore sintática.
class ASTNode {
public:
  // Destrutor virtual para garantir que ao deletar um nó, as subclasses também
  // sejam limpas da memória corretamente.
  virtual ~ASTNode() = default;

  // Método que cada nó concreto terá que implementar para imprimir a si mesmo
  // como JSON. O parâmetro 'indent' serve para formatar o JSON de forma legível
  // (com espaços).
  virtual std::string toJSON(int indent = 0) const = 0;

protected:
  // Função auxiliar que gera espaços em branco de recuo (indentação)
  std::string getIndent(int indent) const {
    return std::string(indent * 2, ' ');
  }
};

// Classe intermediaria para Expressões
class ExpressionNode : public ASTNode {};

// Nó para números (ex:10, 5.5)
class NumberNode : public ExpressionNode {
public:
  std::string value;

  NumberNode(std::string val) : value(val) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"NumberNode\",\n"
       << getIndent(indent + 1) << "\"value\": " << value << "\n"
       << getIndent(indent) << "}";
    return ss.str();
  };
};

// Nó para identificadores/variáveis
class VariableNode : public ExpressionNode {
public:
  std::string name;

  VariableNode(std::string n) : name(n) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"VariableNode\",\n"
       << getIndent(indent + 1) << "\"name\": \"" << name << "\"\n"
       << getIndent(indent) << "}";
    return ss.str();
  };
};

// Nó para operações binárias
// Ele possui um operador (+,-,*,/,<,>,==) e dois nós filhos (esquerda e
// direita)
class BinaryOpNode : public ExpressionNode {
public:
  std::string op;
  std::unique_ptr<ExpressionNode> left;
  std::unique_ptr<ExpressionNode> right;

  BinaryOpNode(std::string o, std::unique_ptr<ExpressionNode> l,
               std::unique_ptr<ExpressionNode> r)
      : op(o), left(std::move(l)), right(std::move(r)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"BinaryOpNode\",\n"
       << getIndent(indent + 1) << "\"op\": \"" << op << "\",\n"
       << getIndent(indent + 1) << "\"left\": \n"
       << left->toJSON(indent + 2) << ",\n"
       << getIndent(indent + 1) << "\"right\": \n"
       << right->toJSON(indent + 2) << "\n"
       << getIndent(indent) << "}";
    return ss.str();
  }
};

// Classe intermediária para comandos
class StatementNode : public ASTNode {};

// Nó para comando de impressão
class PrintNode : public StatementNode {
public:
  std::unique_ptr<ExpressionNode> expression;

  PrintNode(std::unique_ptr<ExpressionNode> expr)
      : expression(std::move(expr)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"PrintNode\",\n"
       << getIndent(indent + 1) << "\"expression\": \n"
       << expression->toJSON(indent + 2) << "\n"
       << getIndent(indent) << "}";
    return ss.str();
  }
};

// Nó para atribuição
class AssignmentNode : public StatementNode {
public:
  std::string id;
  std::unique_ptr<ExpressionNode> expression;

  AssignmentNode(std::string identifier, std::unique_ptr<ExpressionNode> expr)
      : id(identifier), expression(std::move(expr)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"AssignmentNode\",\n"
       << getIndent(indent + 1) << "\"id\": \"" << id << "\",\n"
       << getIndent(indent + 1) << "\"expression\": \n"
       << expression->toJSON(indent + 2) << "\n"
       << getIndent(indent) << "}";
    return ss.str();
  }
};

// Nó para declaração de variáveis
class DeclarationNode : public StatementNode {
public:
  std::string type; // Adicionado para guardar o tipo da variável
  std::string id;
  std::unique_ptr<ExpressionNode>
      initializer; // pode ser nulo se não houver inicialização

  DeclarationNode(std::string type, std::string id,
                  std::unique_ptr<ExpressionNode> init = nullptr)
      : type(type), id(id), initializer(std::move(init)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"DeclarationNode\",\n"
       << getIndent(indent + 1) << "\"var_type\": \"" << type << "\",\n"
       << getIndent(indent + 1) << "\"id\": \"" << id << "\"\n";

    if (initializer) {
      ss << ",\n"
         << getIndent(indent + 1) << "\"initializer\": \n"
         << initializer->toJSON(indent + 2);
    }
    ss << "\n" << getIndent(indent) << "}";
    return ss.str();
  }
};

// Nó para estrutura condicional
class IfNode : public StatementNode {
public:
  std::unique_ptr<ExpressionNode> condition;
  std::vector<std::unique_ptr<StatementNode>> thenBranch;
  std::vector<std::unique_ptr<StatementNode>> elseBranch; // Pode ser vazio
  IfNode(std::unique_ptr<ExpressionNode> cond,
         std::vector<std::unique_ptr<StatementNode>> thenB,
         std::vector<std::unique_ptr<StatementNode>> elseB = {})
      : condition(std::move(cond)), thenBranch(std::move(thenB)),
        elseBranch(std::move(elseB)) {}
  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"IfNode\",\n"
       << getIndent(indent + 1) << "\"condition\": \n"
       << condition->toJSON(indent + 2) << ",\n"
       << getIndent(indent + 1) << "\"then\": [\n";

    for (size_t i = 0; i < thenBranch.size(); ++i) {
      ss << thenBranch[i]->toJSON(indent + 2);
      if (i < thenBranch.size() - 1)
        ss << ",\n";
    }
    ss << "\n" << getIndent(indent + 1) << "]";
    if (!elseBranch.empty()) {
      ss << ",\n" << getIndent(indent + 1) << "\"else\": [\n";
      for (size_t i = 0; i < elseBranch.size(); ++i) {
        ss << elseBranch[i]->toJSON(indent + 2);
        if (i < elseBranch.size() - 1)
          ss << ",\n";
      }
      ss << "\n" << getIndent(indent + 1) << "]";
    }

    ss << "\n" << getIndent(indent) << "}";
    return ss.str();
  }
};

// Nó para laço de repetição While
class WhileNode : public StatementNode {
public:
  std::unique_ptr<ExpressionNode> condition;
  std::vector<std::unique_ptr<StatementNode>> body;

  WhileNode(std::unique_ptr<ExpressionNode> cond,
            std::vector<std::unique_ptr<StatementNode>> b)
      : condition(std::move(cond)), body(std::move(b)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"WhileNode\",\n"
       << getIndent(indent + 1) << "\"condition\": \n"
       << condition->toJSON(indent + 2) << ",\n"
       << getIndent(indent + 1) << "\"body\": [\n";
    for (size_t i = 0; i < body.size(); ++i) {
      ss << body[i]->toJSON(indent + 2);
      if (i < body.size() - 1)
        ss << ",\n";
    }
    ss << "\n" << getIndent(indent + 1) << "]\n" << getIndent(indent) << "}";
    return ss.str();
  }
};

// Nó raiz da árvore sintática, contendo toda a lista de comandos do programa
class ProgramNode : public ASTNode {
public:
  std::vector<std::unique_ptr<StatementNode>> statements;

  ProgramNode(std::vector<std::unique_ptr<StatementNode>> stmts)
      : statements(std::move(stmts)) {}

  std::string toJSON(int indent = 0) const override {
    std::ostringstream ss;
    ss << getIndent(indent) << "{\n"
       << getIndent(indent + 1) << "\"type\": \"ProgramNode\",\n"
       << getIndent(indent + 1) << "\"statements\": [\n";

    for (size_t i = 0; i < statements.size(); ++i) {
      ss << statements[i]->toJSON(indent + 2);
      if (i < statements.size() - 1)
        ss << ",\n";
    }
    ss << "\n" << getIndent(indent + 1) << "]\n" << getIndent(indent) << "}";
    return ss.str();
  }
};

#endif // AST_HPP
