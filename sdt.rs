use crate::ast::{Expr, Program, Statement};

pub trait ASTVisitor {
    fn visit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::LetDecl { id, is_mut, initializer } => self.visit_let_decl(id, *is_mut, initializer.as_deref()),
            Statement::Assignment { id, expr } => self.visit_assignment(id, expr),
            Statement::Println { args } => self.visit_println(args),
            Statement::Block { statements } => self.visit_block(statements),
            Statement::If { condition, then_branch, else_branch } => self.visit_if(condition, then_branch, else_branch.as_deref()),
            Statement::While { condition, body } => self.visit_while(condition, body),
            Statement::FnDecl { name, body } => self.visit_fn_decl(name, body),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(val) => self.visit_number(val),
            Expr::Float(val) => self.visit_float(val),
            Expr::String(val) => self.visit_string(val),
            Expr::Identifier(name) => self.visit_identifier(name),
            Expr::Binary { left, op, right } => self.visit_binary(left, op, right),
        }
    }

    fn visit_let_decl(&mut self, id: &str, is_mut: bool, initializer: Option<&Expr>);
    fn visit_assignment(&mut self, id: &str, expr: &Expr);
    fn visit_println(&mut self, args: &[Expr]);
    fn visit_block(&mut self, statements: &[Statement]);
    fn visit_if(&mut self, condition: &Expr, then_branch: &Statement, else_branch: Option<&Statement>);
    fn visit_while(&mut self, condition: &Expr, body: &Statement);
    fn visit_fn_decl(&mut self, name: &str, body: &Statement);

    fn visit_number(&mut self, val: &str);
    fn visit_float(&mut self, val: &str);
    fn visit_string(&mut self, val: &str);
    fn visit_identifier(&mut self, name: &str);
    fn visit_binary(&mut self, left: &Expr, op: &str, right: &Expr);
}

// ---------------------------------------------------------
// Tradutor 1: PostfixTranslator (Notação Polonesa Reversa / RPN)
// ---------------------------------------------------------
pub struct PostfixTranslator {
    pub output: String,
    pub action_log: Vec<String>,
}

impl PostfixTranslator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            action_log: Vec::new(),
        }
    }

    fn emit(&mut self, token: &str) {
        if !self.output.is_empty() {
            self.output.push(' ');
        }
        self.output.push_str(token);
        self.action_log.push(format!("emit({})", token));
    }
}

impl ASTVisitor for PostfixTranslator {
    fn visit_let_decl(&mut self, id: &str, _is_mut: bool, initializer: Option<&Expr>) {
        if let Some(init) = initializer {
            self.visit_expr(init);
        }
        self.emit(id);
        self.emit("=");
    }

    fn visit_assignment(&mut self, id: &str, expr: &Expr) {
        self.visit_expr(expr);
        self.emit(id);
        self.emit("=");
    }

    fn visit_println(&mut self, args: &[Expr]) {
        for arg in args {
            self.visit_expr(arg);
        }
        self.emit("print");
    }

    fn visit_block(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Statement, else_branch: Option<&Statement>) {
        self.visit_expr(condition);
        self.visit_statement(then_branch);
        if let Some(else_b) = else_branch {
            self.visit_statement(else_b);
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Statement) {
        self.visit_expr(condition);
        self.visit_statement(body);
    }

    fn visit_fn_decl(&mut self, _name: &str, body: &Statement) {
        self.visit_statement(body);
    }

    fn visit_number(&mut self, val: &str) {
        self.emit(val);
    }

    fn visit_float(&mut self, val: &str) {
        self.emit(val);
    }

    fn visit_string(&mut self, val: &str) {
        self.emit(&format!("\"{}\"", val));
    }

    fn visit_identifier(&mut self, name: &str) {
        self.emit(name);
    }

    fn visit_binary(&mut self, left: &Expr, op: &str, right: &Expr) {
        self.visit_expr(left);
        self.visit_expr(right);
        self.emit(op);
    }
}

// ---------------------------------------------------------
// Tradutor 2: TACTranslator (Três Endereços - TAC / IR)
// ---------------------------------------------------------
pub struct TACTranslator {
    temp_count: usize,
    label_count: usize,
    pub last_place: String,
    pub instructions: Vec<String>,
    pub action_log: Vec<String>,
    
    // Melhoria 2: Pool de temporários
    temp_pool: Vec<String>,
    active_temps: Vec<String>,
}

impl TACTranslator {
    pub fn new() -> Self {
        Self {
            temp_count: 0,
            label_count: 0,
            last_place: String::new(),
            instructions: Vec::new(),
            action_log: Vec::new(),
            temp_pool: Vec::new(),
            active_temps: Vec::new(),
        }
    }

    fn new_temp(&mut self) -> String {
        if let Some(t) = self.temp_pool.pop() {
            self.active_temps.push(t.clone());
            t
        } else {
            self.temp_count += 1;
            let t = format!("t{}", self.temp_count);
            self.active_temps.push(t.clone());
            t
        }
    }

    fn free_temp(&mut self, temp: &str) {
        if temp.starts_with('t') && temp[1..].parse::<usize>().is_ok() {
            if let Some(pos) = self.active_temps.iter().position(|x| x == temp) {
                self.active_temps.remove(pos);
                self.temp_pool.push(temp.to_string());
            }
        }
    }

    fn new_label(&mut self) -> String {
        self.label_count += 1;
        format!("L_{}", self.label_count)
    }

    fn emit(&mut self, instruction: &str) {
        self.instructions.push(instruction.to_string());
        self.action_log.push(instruction.to_string());
    }
}

impl ASTVisitor for TACTranslator {
    fn visit_let_decl(&mut self, id: &str, _is_mut: bool, initializer: Option<&Expr>) {
        if let Some(init) = initializer {
            self.visit_expr(init);
            let val = self.last_place.clone();
            self.emit(&format!("{} = {}", id, val));
            self.free_temp(&val);
        }
    }

    fn visit_assignment(&mut self, id: &str, expr: &Expr) {
        self.visit_expr(expr);
        let val = self.last_place.clone();
        self.emit(&format!("{} = {}", id, val));
        self.free_temp(&val);
    }

    fn visit_println(&mut self, args: &[Expr]) {
        for arg in args {
            self.visit_expr(arg);
            let val = self.last_place.clone();
            self.emit(&format!("print {}", val));
            self.free_temp(&val);
        }
    }

    fn visit_block(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Statement, else_branch: Option<&Statement>) {
        let l_else = self.new_label();
        let l_end = self.new_label();
        
        self.visit_expr(condition);
        let cond_val = self.last_place.clone();
        self.free_temp(&cond_val);
        
        if else_branch.is_some() {
            self.emit(&format!("ifFalse {} goto {}", cond_val, l_else));
        } else {
            self.emit(&format!("ifFalse {} goto {}", cond_val, l_end));
        }
        
        self.visit_statement(then_branch);
        
        if let Some(else_b) = else_branch {
            self.emit(&format!("goto {}", l_end));
            self.emit(&format!("{}:", l_else));
            self.visit_statement(else_b);
        }
        
        self.emit(&format!("{}:", l_end));
    }

    fn visit_while(&mut self, condition: &Expr, body: &Statement) {
        let l_start = self.new_label();
        let l_end = self.new_label();
        
        self.emit(&format!("{}:", l_start));
        self.visit_expr(condition);
        let cond_val = self.last_place.clone();
        self.free_temp(&cond_val);
        
        self.emit(&format!("ifFalse {} goto {}", cond_val, l_end));
        self.visit_statement(body);
        self.emit(&format!("goto {}", l_start));
        self.emit(&format!("{}:", l_end));
    }

    fn visit_fn_decl(&mut self, name: &str, body: &Statement) {
        self.emit(&format!("function {}:", name));
        self.visit_statement(body);
    }

    fn visit_number(&mut self, val: &str) {
        self.last_place = val.to_string();
    }

    fn visit_float(&mut self, val: &str) {
        self.last_place = val.to_string();
    }

    fn visit_string(&mut self, val: &str) {
        let t = self.new_temp();
        self.emit(&format!("{} = \"{}\"", t, val));
        self.last_place = t;
    }

    fn visit_identifier(&mut self, name: &str) {
        self.last_place = name.to_string();
    }

    fn visit_binary(&mut self, left: &Expr, op: &str, right: &Expr) {
        // Melhoria 1: Dobra de Constantes (Constant Folding)
        if let (Expr::Number(l_val), Expr::Number(r_val)) = (left, right) {
            if let (Ok(l_num), Ok(r_num)) = (l_val.parse::<i32>(), r_val.parse::<i32>()) {
                let folded = match op {
                    "+" => Some(l_num + r_num),
                    "-" => Some(l_num - r_num),
                    "*" => Some(l_num * r_num),
                    "/" if r_num != 0 => Some(l_num / r_num),
                    _ => None,
                };
                if let Some(res) = folded {
                    self.last_place = res.to_string();
                    return;
                }
            }
        }

        // Melhoria 3: Short-Circuit para Operadores Lógicos (&&, ||)
        if op == "&&" {
            let l_false = self.new_label();
            let l_end = self.new_label();
            
            self.visit_expr(left);
            let left_val = self.last_place.clone();
            self.free_temp(&left_val);
            
            self.emit(&format!("ifFalse {} goto {}", left_val, l_false));
            
            self.visit_expr(right);
            let right_val = self.last_place.clone();
            self.free_temp(&right_val);
            
            let res = self.new_temp();
            self.emit(&format!("{} = {}", res, right_val));
            self.emit(&format!("goto {}", l_end));
            
            self.emit(&format!("{}:", l_false));
            self.emit(&format!("{} = 0", res));
            
            self.emit(&format!("{}:", l_end));
            self.last_place = res;
            return;
        } else if op == "||" {
            let l_true = self.new_label();
            let l_eval_b = self.new_label();
            let l_end = self.new_label();
            
            self.visit_expr(left);
            let left_val = self.last_place.clone();
            self.free_temp(&left_val);
            
            self.emit(&format!("ifFalse {} goto {}", left_val, l_eval_b));
            self.emit(&format!("goto {}", l_true));
            
            self.emit(&format!("{}:", l_eval_b));
            self.visit_expr(right);
            let right_val = self.last_place.clone();
            self.free_temp(&right_val);
            let res = self.new_temp();
            self.emit(&format!("{} = {}", res, right_val));
            self.emit(&format!("goto {}", l_end));
            
            self.emit(&format!("{}:", l_true));
            self.emit(&format!("{} = 1", res));
            
            self.emit(&format!("{}:", l_end));
            self.last_place = res;
            return;
        }

        self.visit_expr(left);
        let left_place = self.last_place.clone();
        
        self.visit_expr(right);
        let right_place = self.last_place.clone();
        
        let res = self.new_temp();
        self.emit(&format!("{} = {} {} {}", res, left_place, op, right_place));
        
        self.free_temp(&left_place);
        self.free_temp(&right_place);
        
        self.last_place = res;
    }
}

// ---------------------------------------------------------
// Tradutor 3: PrettyPrinter (Reimpressor Canônico)
// ---------------------------------------------------------
pub struct PrettyPrinter {
    pub output: String,
    indent_level: usize,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent_level));
    }
}

impl ASTVisitor for PrettyPrinter {
    fn visit_let_decl(&mut self, id: &str, is_mut: bool, initializer: Option<&Expr>) {
        self.emit_indent();
        if is_mut {
            self.emit(&format!("let mut {} = ", id));
        } else {
            self.emit(&format!("let {} = ", id));
        }
        if let Some(init) = initializer {
            self.visit_expr(init);
        }
        self.emit(";\n");
    }

    fn visit_assignment(&mut self, id: &str, expr: &Expr) {
        self.emit_indent();
        self.emit(&format!("{} = ", id));
        self.visit_expr(expr);
        self.emit(";\n");
    }

    fn visit_println(&mut self, args: &[Expr]) {
        self.emit_indent();
        self.emit("println!(");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            self.visit_expr(arg);
        }
        self.emit(");\n");
    }

    fn visit_block(&mut self, statements: &[Statement]) {
        self.emit("{\n");
        self.indent_level += 1;
        for stmt in statements {
            self.visit_statement(stmt);
        }
        self.indent_level -= 1;
        self.emit_indent();
        self.emit("}\n");
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Statement, else_branch: Option<&Statement>) {
        self.emit_indent();
        self.emit("if (");
        self.visit_expr(condition);
        self.emit(") ");
        
        if let Statement::Block { .. } = then_branch {
            self.visit_statement(then_branch);
        } else {
            self.emit("{\n");
            self.indent_level += 1;
            self.visit_statement(then_branch);
            self.indent_level -= 1;
            self.emit_indent();
            self.emit("}\n");
        }

        if let Some(else_b) = else_branch {
            self.emit_indent();
            self.emit("else ");
            if let Statement::Block { .. } = else_b {
                self.visit_statement(else_b);
            } else {
                self.emit("{\n");
                self.indent_level += 1;
                self.visit_statement(else_b);
                self.indent_level -= 1;
                self.emit_indent();
                self.emit("}\n");
            }
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Statement) {
        self.emit_indent();
        self.emit("while (");
        self.visit_expr(condition);
        self.emit(") ");
        
        if let Statement::Block { .. } = body {
            self.visit_statement(body);
        } else {
            self.emit("{\n");
            self.indent_level += 1;
            self.visit_statement(body);
            self.indent_level -= 1;
            self.emit_indent();
            self.emit("}\n");
        }
    }

    fn visit_fn_decl(&mut self, name: &str, body: &Statement) {
        self.emit_indent();
        self.emit(&format!("fn {}() ", name));
        if let Statement::Block { .. } = body {
            self.visit_statement(body);
        } else {
            self.emit("{\n");
            self.indent_level += 1;
            self.visit_statement(body);
            self.indent_level -= 1;
            self.emit_indent();
            self.emit("}\n");
        }
    }

    fn visit_number(&mut self, val: &str) {
        self.emit(val);
    }

    fn visit_float(&mut self, val: &str) {
        self.emit(val);
    }

    fn visit_string(&mut self, val: &str) {
        self.emit(&format!("\"{}\"", val));
    }

    fn visit_identifier(&mut self, name: &str) {
        self.emit(name);
    }

    fn visit_binary(&mut self, left: &Expr, op: &str, right: &Expr) {
        self.emit("(");
        self.visit_expr(left);
        self.emit(&format!(" {} ", op));
        self.visit_expr(right);
        self.emit(")");
    }
}
