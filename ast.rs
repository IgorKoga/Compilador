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
            Expr::String(val) => format!(r#"{{"type": "String", "value": "{}"}}"#, escape_json(val)),
            Expr::Identifier(name) => format!(r#"{{"type": "Identifier", "name": "{}"}}"#, name),
            Expr::Binary { left, op, right } => format!(
                r#"{{"type": "BinaryExpr", "op": "{}", "left": {}, "right": {}}}"#,
                op, left.to_json(), right.to_json() 
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
