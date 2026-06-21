//Utilitário para limpar textos
pub fn escape_json(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            _ => result.push(c),
        }
    }
    result
}

//Lista de opções de Expressões
pub enum Expr {
    Number(String),
    Float(String),
    String(String),
    Identifier(String),

    Binary {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
}

//Métodos das Expressões
impl Expr {
    pub fn to_json(&self) -> String {
        match self {
            Expr::Number(val) => format!(r#"{{"type": "Number", "value": {}}}"#, val),
            Expr::Float(val) => format!(r#"{{"type": "Float", "value": {}}}"#, val),
            Expr::String(val) => {
                format!(r#"{{"type": "String", "value": "{}"}}"#, escape_json(val))
            }
            Expr::Identifier(name) => format!(r#"{{"type": "Identifier", "name": "{}"}}"#, name),
            Expr::Binary { left, op, right } => format!(
                r#"{{"type": "BinaryExpr", "op": "{}", "left": {}, "right": {}}}"#,
                op,
                left.to_json(),
                right.to_json()
            ),
        }
    }

    pub fn print(&self, indent: usize) {
        let spaces = " ".repeat(indent);
        match self {
            Expr::Number(val) => println!("{}NumberExpr: {}", spaces, val),
            Expr::Float(val) => println!("{}FloatExpr: {}", spaces, val),
            Expr::String(val) => println!("{}StringExpr: \"{}\"", spaces, escape_json(val)),
            Expr::Identifier(name) => println!("{}IdentifierExpr: {}", spaces, name),
            Expr::Binary { left, op, right } => {
                println!("{}BinaryExpr ({})", spaces, op);
                left.print(indent + 2);
                right.print(indent + 2);
            }
        }
    }
}

//Estrutura das Instruções
pub enum Statement {
    LetDecl {
        id: String,
        is_mut: bool,
        initializer: Option<Box<Expr>>,
    },
    Assignment {
        id: String,
        expr: Box<Expr>,
    },
    Println {
        args: Vec<Expr>,
    },
    Block {
        statements: Vec<Statement>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Statement>,
    },
    FnDecl {
        name: String,
        body: Box<Statement>,
    },
}

// Métodos das Instruções
impl Statement {
    pub fn to_json(&self) -> String {
        match self {
            Statement::LetDecl {
                id,
                is_mut,
                initializer,
            } => {
                let init_json = initializer
                    .as_ref()
                    .map_or("null".to_string(), |expr| expr.to_json());
                format!(
                    r#"{{"type": "LetDecl", "id": "{}", "isMut": {}, "init": {}}}"#,
                    id, is_mut, init_json
                )
            }
            Statement::Assignment { id, expr } => {
                format!(
                    r#"{{"type": "Assignment", "id": "{}", "expr": {}}}"#,
                    id,
                    expr.to_json()
                )
            }
            Statement::Println { args } => {
                let args_json: Vec<String> = args.iter().map(|arg| arg.to_json()).collect();
                format!(
                    r#"{{"type": "Println", "args": [{}]}}"#,
                    args_json.join(", ")
                )
            }
            Statement::Block { statements } => {
                let stmts_json: Vec<String> = statements.iter().map(|s| s.to_json()).collect();
                format!(
                    r#"{{"type": "Block", "statements": [{}]}}"#,
                    stmts_json.join(", ")
                )
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_json = else_branch
                    .as_ref()
                    .map_or("null".to_string(), |b| b.to_json());
                format!(
                    r#"{{"type": "IfStmt", "condition": {}, "thenBranch": {}, "elseBranch": {}}}"#,
                    condition.to_json(),
                    then_branch.to_json(),
                    else_json
                )
            }
            Statement::While { condition, body } => {
                format!(
                    r#"{{"type": "WhileStmt", "condition": {}, "body": {}}}"#,
                    condition.to_json(),
                    body.to_json()
                )
            }
            Statement::FnDecl { name, body } => {
                format!(
                    r#"{{"type": "FnDecl", "name": "{}", "body": {}}}"#,
                    name,
                    body.to_json()
                )
            }
        }
    }

    pub fn print(&self, indent: usize) {
        let spaces = " ".repeat(indent);
        match self {
            Statement::LetDecl {
                id,
                is_mut,
                initializer,
            } => {
                let mut_str = if *is_mut { " (mut)" } else { "" };
                println!("{}LetDeclStmt: {}{}", spaces, id, mut_str);
                if let Some(init) = initializer {
                    init.print(indent + 2);
                }
            }
            Statement::Assignment { id, expr } => {
                println!("{}AssignmentStmt: {}", spaces, id);
                expr.print(indent + 2);
            }
            Statement::Println { args } => {
                println!("{}PrintlnStmt", spaces);
                for arg in args {
                    arg.print(indent + 2);
                }
            }
            Statement::Block { statements } => {
                println!("{}BlockStmt", spaces);
                for stmt in statements {
                    stmt.print(indent + 2);
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                println!("{}IfStmt", spaces);
                println!("{}  Condition:", spaces);
                condition.print(indent + 4);
                println!("{}  ThenBranch:", spaces);
                then_branch.print(indent + 4);
                if let Some(else_b) = else_branch {
                    println!("{}  ElseBranch:", spaces);
                    else_b.print(indent + 4);
                }
            }
            Statement::While { condition, body } => {
                println!("{}WhileStmt", spaces);
                println!("{}  Condition:", spaces);
                condition.print(indent + 4);
                println!("{}  Body:", spaces);
                body.print(indent + 4);
            }
            Statement::FnDecl { name, body } => {
                println!("{}FnDeclStmt: {}", spaces, name);
                body.print(indent + 2);
            }
        }
    }
}

// Nó Raiz (Program)
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub fn to_json(&self) -> String {
        let stmts_json: Vec<String> = self
            .statements
            .iter()
            .map(|s| format!("    {}", s.to_json()))
            .collect();
        format!(
            "{{\n  \"type\": \"Program\",\n  \"body\": [\n{}\n  ]\n}}",
            stmts_json.join(",\n")
        )
    }

    pub fn print(&self, indent: usize) {
        let spaces = " ".repeat(indent);
        println!("{}Program", spaces);
        for stmt in &self.statements {
            stmt.print(indent + 2);
        }
    }
}
