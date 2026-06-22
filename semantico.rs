use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use super::ast::{Expr, Program, Statement};

// Representa um símbolo armazenado na tabela (variáveis, constantes e funções)
pub struct Symbol {
    pub name: String,
    pub r#type: String,
    pub isMut: bool,
    pub isUsed: bool,
    pub isInitialized: bool,
}

impl Symbol {
    pub fn new(n: String, t: String, mut_: bool, init: bool) -> Self {
        Symbol {
            name: n,
            r#type: t,
            isMut: mut_,
            isUsed: false,
            isInitialized: init,
        }
    }
}

// Gerencia o contexto dos escopos, mantendo uma pilha de tabelas hash
pub struct SymbolTable {
    pub scopes: Vec<HashMap<String, Rc<RefCell<Symbol>>>>,
    pub warnings: Rc<RefCell<Vec<String>>>, // Referência para acumular warnings
}

impl SymbolTable {
    pub fn new(warns: Rc<RefCell<Vec<String>>>) -> Self {
        let mut table = SymbolTable {
            scopes: Vec::new(),
            warnings: warns,
        };
        table.enterScope(); // Inicializa o escopo global
        table
    }

    pub fn enterScope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exitScope(&mut self) {
        if self.scopes.is_empty() {
            return;
        }
        // Melhoria: Variáveis não utilizadas (Relatório de warnings)
        if let Some(back) = self.scopes.last() {
            for (_, symbol) in back {
                let sym = symbol.borrow();
                if !sym.isUsed && sym.r#type != "function" {
                    self.warnings.borrow_mut().push(format!(
                        "Aviso Semantico: variavel '{}' declarada mas nunca utilizada.",
                        sym.name
                    ));
                }
            }
        }
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, r#type: String, isMut: bool, isInitialized: bool) -> bool {
        if self.scopes.is_empty() {
            return false;
        }
        let back = self.scopes.last_mut().unwrap();
        // Melhoria: Declaração Duplicada e Controle de escopo aninhado
        if back.contains_key(&name) {
            return false;
        }
        back.insert(
            name.clone(),
            Rc::new(RefCell::new(Symbol::new(name, r#type, isMut, isInitialized))),
        );
        true
    }

    pub fn lookup(&self, name: &str) -> Option<Rc<RefCell<Symbol>>> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(Rc::clone(sym));
            }
        }
        None
    }

    // Melhoria: Exportação da Tabela de Símbolos em JSON
    pub fn toJson(&self) -> String {
        let mut json = String::from("{\n  \"symbol_table\": [\n");
        for (i, scope) in self.scopes.iter().enumerate() {
            json.push_str(&format!("    {{\n      \"scope\": {},\n      \"symbols\": [\n", i));
            let mut count = 0;
            for (_, symbol) in scope {
                let sym = symbol.borrow();
                json.push_str(&format!(
                    "        {{\"name\": \"{}\", \"type\": \"{}\", \"isMut\": {}, \"isUsed\": {}}}",
                    sym.name,
                    sym.r#type,
                    if sym.isMut { "true" } else { "false" },
                    if sym.isUsed { "true" } else { "false" }
                ));
                count += 1;
                if count < scope.len() {
                    json.push_str(",");
                }
                json.push_str("\n");
            }
            json.push_str("      ]\n    }");
            if i < self.scopes.len() - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }
        json.push_str("  ]\n}");
        json
    }
}

impl Drop for SymbolTable {
    fn drop(&mut self) {
        while !self.scopes.is_empty() {
            self.exitScope();
        }
    }
}

// Classe principal do Analisador Semântico
pub struct SemanticAnalyzer {
    pub errors: Vec<String>,
    pub warnings: Rc<RefCell<Vec<String>>>,
    pub symTable: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let warnings = Rc::new(RefCell::new(Vec::new()));
        SemanticAnalyzer {
            errors: Vec::new(),
            warnings: Rc::clone(&warnings),
            symTable: SymbolTable::new(Rc::clone(&warnings)),
        }
    }

    pub fn analyze(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.analyzeStatement(stmt);
        }
    }

    pub fn report(&self) {
        println!("\n--- Relatorio do Analisador Semantico ---");
        let warnings = self.warnings.borrow();
        if self.errors.is_empty() && warnings.is_empty() {
            println!("Analise Semantica concluida com sucesso.");
        } else {
            for w in warnings.iter() {
                println!("{}", w);
            }
            for e in &self.errors {
                println!("Erro Semantico: {}", e);
            }
            println!("\nQuantidade total de erros encontrados: {}", self.errors.len());
        }
        println!("-----------------------------------------");
    }

    pub fn getSymbolTableJson(&self) -> String {
        self.symTable.toJson()
    }

    pub fn hasErrors(&self) -> bool {
        !self.errors.is_empty()
    }

    // Visitor emulada através de pattern matching para percorrer a AST
    pub fn analyzeStatement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::LetDecl { id, is_mut, initializer } => {
                let mut inferred_type = String::from("unknown");
                let mut is_init = false;
                if let Some(init_expr) = initializer {
                    is_init = true;
                    self.analyzeExpr(init_expr);
                    inferred_type = self.inferType(init_expr);
                }
                
                if !self.symTable.insert(id.clone(), inferred_type, *is_mut, is_init) {
                    self.errors.push(format!("variavel '{}' ja declarada.", id));
                }
            }
            Statement::Assignment { id, expr } => {
                self.analyzeExpr(expr);
                let expr_type = self.inferType(expr);
                
                if let Some(sym_rc) = self.symTable.lookup(id) {
                    let mut sym = sym_rc.borrow_mut();
                    // Melhoria: Constantes
                    if !sym.isMut && sym.isInitialized {
                        self.errors.push(format!("atribuicao a variavel constante (imutavel) '{}'.", id));
                    }
                    // Verificação de Tipos
                    if sym.r#type != "unknown" && expr_type != "unknown" && sym.r#type != expr_type {
                        self.errors.push(String::from("atribuicao incompativel."));
                    }
                    sym.isInitialized = true;
                } else {
                    self.errors.push(format!("variavel '{}' nao declarada.", id));
                }
            }
            Statement::Println { args } => {
                for arg in args {
                    self.analyzeExpr(arg);
                }
            }
            Statement::Block { statements } => {
                self.symTable.enterScope();
                for s in statements {
                    self.analyzeStatement(s);
                }
                self.symTable.exitScope();
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.analyzeExpr(condition);
                
                self.symTable.enterScope();
                self.analyzeStatement(then_branch);
                self.symTable.exitScope();

                if let Some(else_stmt) = else_branch {
                    self.symTable.enterScope();
                    self.analyzeStatement(else_stmt);
                    self.symTable.exitScope();
                }
            }
            Statement::While { condition, body } => {
                self.analyzeExpr(condition);
                
                self.symTable.enterScope();
                self.analyzeStatement(body);
                self.symTable.exitScope();
            }
            Statement::FnDecl { name, body } => {
                // Melhoria: Funções
                if !self.symTable.insert(name.clone(), String::from("function"), false, true) {
                    self.errors.push(format!("funcao '{}' ja declarada.", name));
                }
                self.symTable.enterScope();
                self.analyzeStatement(body);
                self.symTable.exitScope();
            }
        }
    }

    pub fn analyzeExpr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, op, right } => {
                self.analyzeExpr(left);
                self.analyzeExpr(right);
                
                let left_type = self.inferType(left);
                let right_type = self.inferType(right);
                
                if left_type != "unknown" && right_type != "unknown" {
                    if op == "+" || op == "-" || op == "*" || op == "/" {
                        if left_type == "string" || right_type == "string" {
                            if op != "+" {
                                self.errors.push(format!("operacao '{}' incompativel entre tipos {} e {}.", op, left_type, right_type));
                            } else if left_type != "string" || right_type != "string" {
                                self.errors.push(format!("operacao '+' incompativel entre tipos {} e {}.", left_type, right_type));
                            }
                        }
                    } else if op == "<" || op == ">" || op == "==" {
                        if left_type != right_type && !((left_type == "int" && right_type == "float") || (left_type == "float" && right_type == "int")) {
                            self.errors.push(format!("operacao relacional incompativel entre tipos {} e {}.", left_type, right_type));
                        }
                    }
                }

                // Melhoria: Verificação de divisão por zero
                if op == "/" {
                    match right.as_ref() {
                        Expr::Number(value) => {
                            if value == "0" {
                                self.errors.push(String::from("divisao por zero detectavel em tempo de compilacao."));
                            }
                        }
                        Expr::Float(value) => {
                            if value == "0.0" || value == "0" {
                                self.errors.push(String::from("divisao por zero detectavel em tempo de compilacao."));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Expr::Identifier(name) => {
                if let Some(sym_rc) = self.symTable.lookup(name) {
                    let mut sym = sym_rc.borrow_mut();
                    sym.isUsed = true;
                    // Melhoria: Uso antes da inicialização
                    if !sym.isInitialized {
                        self.warnings.borrow_mut().push(format!("Aviso Semantico: variavel '{}' utilizada antes de receber valor.", name));
                    }
                } else {
                    self.errors.push(format!("variavel '{}' nao declarada.", name));
                }
            }
            _ => {}
        }
    }

    // Melhoria: Inferência de Tipos Automática
    pub fn inferType(&self, expr: &Expr) -> String {
        match expr {
            Expr::Number(..) => String::from("int"),
            Expr::Float(..) => String::from("float"),
            Expr::String(..) => String::from("string"),
            Expr::Identifier(name) => {
                if let Some(sym_rc) = self.symTable.lookup(name) {
                    sym_rc.borrow().r#type.clone()
                } else {
                    String::from("unknown")
                }
            }
            Expr::Binary { left, op: _, right } => {
                let left_type = self.inferType(left);
                let right_type = self.inferType(right);
                
                if left_type == "unknown" || right_type == "unknown" {
                    return String::from("unknown");
                }
                
                if left_type == right_type {
                    return left_type;
                }
                if (left_type == "float" && right_type == "int") || (left_type == "int" && right_type == "float") {
                    return String::from("float"); // Coerção segura de int para float
                }
                
                String::from("unknown")
            }
        }
    }
}
