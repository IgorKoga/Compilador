#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

/// Representação estruturada de uma instrução de Código de Três Endereços (TAC)
#[derive(Debug, Clone, PartialEq)]
pub enum TacInstr {
    Assign { dest: String, src: String },
    Binary { dest: String, left: String, op: String, right: String },
    IfFalse { cond: String, target: String },
    Goto(String),
    Label(String),
    Print(String),
    Param(String),
    Call { func: String, args: usize },
    Raw(String),
}

impl TacInstr {
    /// Faz o parse de uma linha de texto TAC para a representação estruturada
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return TacInstr::Raw(String::new());
        }

        // ifFalse cond goto target
        if trimmed.starts_with("ifFalse ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 4 && parts[2] == "goto" {
                return TacInstr::IfFalse {
                    cond: parts[1].to_string(),
                    target: parts[3].to_string(),
                };
            }
        }

        // goto target
        if trimmed.starts_with("goto ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return TacInstr::Goto(parts[1].to_string());
            }
        }

        // function nome: ou L1:
        if trimmed.ends_with(':') {
            return TacInstr::Label(trimmed.to_string());
        }

        // print valor
        if trimmed.starts_with("print ") {
            return TacInstr::Print(trimmed[6..].trim().to_string());
        }

        // param valor
        if trimmed.starts_with("param ") {
            return TacInstr::Param(trimmed[6..].trim().to_string());
        }

        // call funcao, n_args
        if trimmed.starts_with("call ") {
            let rest = trimmed[5..].trim();
            let parts: Vec<&str> = rest.split(',').collect();
            let func = parts[0].trim().to_string();
            let args = if parts.len() > 1 {
                parts[1].trim().parse::<usize>().unwrap_or(0)
            } else {
                0
            };
            return TacInstr::Call { func, args };
        }

        // dest = ...
        if let Some(eq_idx) = trimmed.find('=') {
            let dest = trimmed[..eq_idx].trim().to_string();
            let rhs = trimmed[eq_idx + 1..].trim();

            // Verifica se é uma operação binária: left op right
            let tokens: Vec<&str> = rhs.split_whitespace().collect();
            if tokens.len() == 3 {
                let left = tokens[0].to_string();
                let op = tokens[1].to_string();
                let right = tokens[2].to_string();
                return TacInstr::Binary {
                    dest,
                    left,
                    op,
                    right,
                };
            } else {
                return TacInstr::Assign {
                    dest,
                    src: rhs.to_string(),
                };
            }
        }

        TacInstr::Raw(trimmed.to_string())
    }

    /// Retorna a representação em texto padronizada da instrução
    pub fn to_tac_string(&self) -> String {
        match self {
            TacInstr::Assign { dest, src } => format!("{} = {}", dest, src),
            TacInstr::Binary { dest, left, op, right } => {
                format!("{} = {} {} {}", dest, left, op, right)
            }
            TacInstr::IfFalse { cond, target } => format!("ifFalse {} goto {}", cond, target),
            TacInstr::Goto(target) => format!("goto {}", target),
            TacInstr::Label(label) => label.clone(),
            TacInstr::Print(arg) => format!("print {}", arg),
            TacInstr::Param(arg) => format!("param {}", arg),
            TacInstr::Call { func, args } => format!("call {}, {}", func, args),
            TacInstr::Raw(s) => s.clone(),
        }
    }

    /// Retorna a variável de destino definida pela instrução (se houver)
    pub fn get_defined_var(&self) -> Option<&str> {
        match self {
            TacInstr::Assign { dest, .. } => Some(dest.as_str()),
            TacInstr::Binary { dest, .. } => Some(dest.as_str()),
            _ => None,
        }
    }

    /// Retorna se a instrução possui efeito colateral observável (não pode ser morta)
    pub fn has_observable_side_effect(&self) -> bool {
        match self {
            TacInstr::Print(_) | TacInstr::Param(_) | TacInstr::Call { .. } => true,
            TacInstr::IfFalse { .. } | TacInstr::Goto(_) | TacInstr::Label(_) => true,
            _ => false,
        }
    }
}

/// Representação de um Bloco Básico (Aula 09 - Parte 01, Slides 18-27)
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub instructions: Vec<TacInstr>,
}

/// Registro das transformações aplicadas em uma rodada do otimizador
#[derive(Debug, Clone)]
pub struct RoundLog {
    pub round: usize,
    pub actions: Vec<String>,
}

/// Estatísticas comparativas do processo de otimização
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub initial_instructions: usize,
    pub final_instructions: usize,
    pub initial_temps: usize,
    pub final_temps: usize,
    pub rounds_executed: usize,
    pub round_logs: Vec<RoundLog>,
    pub initial_tac: Vec<String>,
    pub optimized_tac: Vec<String>,
    pub blocks_before: Vec<BasicBlock>,
    pub blocks_after: Vec<BasicBlock>,
}

/// Otimizador de Código Intermediário (TAC) baseado nas Aulas 09 Parte 01 e Parte 02
pub struct TacOptimizer {
    pub instructions: Vec<TacInstr>,
}

impl TacOptimizer {
    pub fn new(raw_instructions: &[String]) -> Self {
        let instructions: Vec<TacInstr> = raw_instructions
            .iter()
            .map(|s| TacInstr::parse(s))
            .filter(|instr| match instr {
                TacInstr::Raw(s) if s.is_empty() => false,
                _ => true,
            })
            .collect();
        TacOptimizer { instructions }
    }

    /// Identifica os líderes e divide as instruções em Blocos Básicos
    /// Regra dos Líderes (Aula 09 Parte 01, Slide 20):
    /// 1. A primeira instrução do programa é líder.
    /// 2. Qualquer alvo de um salto (rótulo) é líder.
    /// 3. A instrução imediatamente após um salto (goto ou ifFalse) é líder.
    pub fn identify_basic_blocks(instructions: &[TacInstr]) -> Vec<BasicBlock> {
        if instructions.is_empty() {
            return Vec::new();
        }

        let mut is_leader = vec![false; instructions.len()];
        is_leader[0] = true; // Regra 1: primeira instrução

        // Mapeia onde cada rótulo está
        let mut label_map: HashMap<String, usize> = HashMap::new();
        for (i, instr) in instructions.iter().enumerate() {
            if let TacInstr::Label(l) = instr {
                let clean = l.trim_end_matches(':').trim();
                label_map.insert(clean.to_string(), i);
                is_leader[i] = true; // Regra 2: rótulo é alvo potencial
            }
        }

        for (i, instr) in instructions.iter().enumerate() {
            match instr {
                TacInstr::Goto(target) => {
                    let clean = target.trim_end_matches(':').trim();
                    if let Some(&idx) = label_map.get(clean) {
                        is_leader[idx] = true; // Regra 2: alvo de salto
                    }
                    if i + 1 < instructions.len() {
                        is_leader[i + 1] = true; // Regra 3: após salto
                    }
                }
                TacInstr::IfFalse { target, .. } => {
                    let clean = target.trim_end_matches(':').trim();
                    if let Some(&idx) = label_map.get(clean) {
                        is_leader[idx] = true; // Regra 2: alvo de salto
                    }
                    if i + 1 < instructions.len() {
                        is_leader[i + 1] = true; // Regra 3: após salto
                    }
                }
                _ => {}
            }
        }

        let mut leader_indices: Vec<usize> = Vec::new();
        for (i, &leader) in is_leader.iter().enumerate() {
            if leader {
                leader_indices.push(i);
            }
        }

        let mut blocks = Vec::new();
        for (b_idx, &start) in leader_indices.iter().enumerate() {
            let end = if b_idx + 1 < leader_indices.len() {
                leader_indices[b_idx + 1] - 1
            } else {
                instructions.len() - 1
            };
            blocks.push(BasicBlock {
                id: b_idx + 1,
                name: format!("B{}", b_idx + 1),
                start,
                end,
                instructions: instructions[start..=end].to_vec(),
            });
        }

        blocks
    }

    /// Conta quantas variáveis temporárias únicas (t1, t2, ...) estão presentes
    pub fn count_temps(instructions: &[TacInstr]) -> usize {
        let mut temps = HashSet::new();
        for instr in instructions {
            let text = instr.to_tac_string();
            for word in text.split(|c: char| !c.is_alphanumeric() && c != '_') {
                if word.starts_with('t') && word.len() > 1 && word[1..].chars().all(|c| c.is_ascii_digit()) {
                    temps.insert(word.to_string());
                }
            }
        }
        temps.len()
    }

    // =========================================================================
    // AS NOVE PASSAGENS DO OTIMIZADOR (Aula 09 - Parte 02, Slide 23)
    // =========================================================================

    /// Passagem 1: Propagação de Constantes e Cópias (Slides Parte 1: 34-39; Parte 2: 2)
    /// Tabela de conhecimento por bloco básico. O conhecimento morre em:
    /// - Redefinição (x = ... apaga o que sabíamos de x)
    /// - Fim de bloco (goto ou ifFalse)
    /// - Entrada de bloco (rótulo L: apaga o conhecimento)
    fn pass_propagation(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut knowledge: HashMap<String, String> = HashMap::new();

        let mut new_instructions = Vec::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::Label(_) => {
                    // Entrada de bloco: apaga a tabela de conhecimento (Slide 39 Parte 1, Slide 2 Parte 2)
                    knowledge.clear();
                    new_instructions.push(instr.clone());
                }
                TacInstr::Goto(_) => {
                    // Fim de bloco: apaga o conhecimento
                    knowledge.clear();
                    new_instructions.push(instr.clone());
                }
                TacInstr::IfFalse { cond, target } => {
                    let mut new_cond = cond.clone();
                    if let Some(val) = knowledge.get(cond) {
                        new_cond = val.clone();
                        actions.push(format!("Propagou {} -> {} em ifFalse", cond, val));
                        changed = true;
                    }
                    new_instructions.push(TacInstr::IfFalse {
                        cond: new_cond,
                        target: target.clone(),
                    });
                    // Fim de bloco: apaga conhecimento
                    knowledge.clear();
                }
                TacInstr::Print(arg) => {
                    let mut new_arg = arg.clone();
                    if let Some(val) = knowledge.get(arg) {
                        new_arg = val.clone();
                        actions.push(format!("Propagou {} -> {} em print", arg, val));
                        changed = true;
                    }
                    new_instructions.push(TacInstr::Print(new_arg));
                }
                TacInstr::Param(arg) => {
                    let mut new_arg = arg.clone();
                    if let Some(val) = knowledge.get(arg) {
                        new_arg = val.clone();
                        actions.push(format!("Propagou {} -> {} em param", arg, val));
                        changed = true;
                    }
                    new_instructions.push(TacInstr::Param(new_arg));
                }
                TacInstr::Call { .. } | TacInstr::Raw(_) => {
                    new_instructions.push(instr.clone());
                }
                TacInstr::Assign { dest, src } => {
                    let mut new_src = src.clone();
                    if let Some(val) = knowledge.get(src) {
                        new_src = val.clone();
                        actions.push(format!("Propagou {} -> {} em {} = {}", src, val, dest, src));
                        changed = true;
                    }

                    // Redefinição: apaga o que sabíamos de dest
                    knowledge.remove(dest);
                    // E se dest era valor de outra cópia, invalida essa cópia (Slide 38 Parte 1)
                    knowledge.retain(|_, v| v != dest);

                    // Se new_src é constante numérica ou identificador simples, armazena no conhecimento
                    if is_constant(&new_src) || is_identifier(&new_src) {
                        knowledge.insert(dest.clone(), new_src.clone());
                    }

                    new_instructions.push(TacInstr::Assign {
                        dest: dest.clone(),
                        src: new_src,
                    });
                }
                TacInstr::Binary { dest, left, op, right } => {
                    let mut new_left = left.clone();
                    let mut new_right = right.clone();

                    if let Some(val) = knowledge.get(left) {
                        new_left = val.clone();
                        actions.push(format!("Propagou {} -> {} em {} = ...", left, val, dest));
                        changed = true;
                    }
                    if let Some(val) = knowledge.get(right) {
                        new_right = val.clone();
                        actions.push(format!("Propagou {} -> {} em {} = ...", right, val, dest));
                        changed = true;
                    }

                    // Redefinição: apaga dest
                    knowledge.remove(dest);
                    knowledge.retain(|_, v| v != dest);

                    new_instructions.push(TacInstr::Binary {
                        dest: dest.clone(),
                        left: new_left,
                        op: op.clone(),
                        right: new_right,
                    });
                }
            }
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 2: Dobramento de Constantes (Constant Folding - Parte 1: 29-33)
    /// Comparações viram 1 ou 0. Divisão por zero NUNCA é dobrada (deixa para execução).
    fn pass_constant_folding(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut new_instructions = Vec::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::Binary { dest, left, op, right } => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        // Cuidado especial: Divisão por zero NÃO dobrar (Slide 31 Parte 1)
                        if (op == "/" || op == "%") && r == 0 {
                            new_instructions.push(instr.clone());
                            continue;
                        }

                        let result = match op.as_str() {
                            "+" => Some(l + r),
                            "-" => Some(l - r),
                            "*" => Some(l * r),
                            "/" => Some(l / r),
                            "%" => Some(l % r),
                            "==" => Some(if l == r { 1 } else { 0 }),
                            "!=" => Some(if l != r { 1 } else { 0 }),
                            "<" => Some(if l < r { 1 } else { 0 }),
                            "<=" => Some(if l <= r { 1 } else { 0 }),
                            ">" => Some(if l > r { 1 } else { 0 }),
                            ">=" => Some(if l >= r { 1 } else { 0 }),
                            "&&" => Some(if l != 0 && r != 0 { 1 } else { 0 }),
                            "||" => Some(if l != 0 || r != 0 { 1 } else { 0 }),
                            _ => None,
                        };

                        if let Some(val) = result {
                            actions.push(format!(
                                "Dobrou {} {} {} -> {} ({} = {})",
                                left, op, right, val, dest, val
                            ));
                            new_instructions.push(TacInstr::Assign {
                                dest: dest.clone(),
                                src: val.to_string(),
                            });
                            changed = true;
                            continue;
                        }
                    }
                    new_instructions.push(instr.clone());
                }
                _ => new_instructions.push(instr.clone()),
            }
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 3: Identidades Algébricas e Redução de Força (Parte 2: 3-7)
    /// x + 0 -> x, x * 1 -> x, x * 0 -> 0, x - x -> 0, x * 2 -> x + x
    fn pass_algebra_and_strength(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut new_instructions = Vec::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::Binary { dest, left, op, right } => {
                    let mut replaced = None;

                    // x + 0 -> x ou 0 + x -> x
                    if op == "+" {
                        if right == "0" {
                            replaced = Some(TacInstr::Assign {
                                dest: dest.clone(),
                                src: left.clone(),
                            });
                            actions.push(format!("Identidade algébrica: {} + 0 -> {}", left, left));
                        } else if left == "0" {
                            replaced = Some(TacInstr::Assign {
                                dest: dest.clone(),
                                src: right.clone(),
                            });
                            actions.push(format!("Identidade algébrica: 0 + {} -> {}", right, right));
                        }
                    }
                    // x - 0 -> x
                    else if op == "-" && right == "0" {
                        replaced = Some(TacInstr::Assign {
                            dest: dest.clone(),
                            src: left.clone(),
                        });
                        actions.push(format!("Identidade algébrica: {} - 0 -> {}", left, left));
                    }
                    // x - x -> 0
                    else if op == "-" && left == right && !left.is_empty() {
                        replaced = Some(TacInstr::Assign {
                            dest: dest.clone(),
                            src: "0".to_string(),
                        });
                        actions.push(format!("Identidade algébrica: {} - {} -> 0", left, right));
                    }
                    // x * 1 -> x ou 1 * x -> x
                    else if op == "*" && (right == "1" || left == "1") {
                        let other = if right == "1" { left } else { right };
                        replaced = Some(TacInstr::Assign {
                            dest: dest.clone(),
                            src: other.clone(),
                        });
                        actions.push(format!("Identidade algébrica: multiplicação por 1 -> {}", other));
                    }
                    // x * 0 -> 0 ou 0 * x -> 0 (seguro no MicroC/MicroRust para inteiros)
                    else if op == "*" && (right == "0" || left == "0") {
                        replaced = Some(TacInstr::Assign {
                            dest: dest.clone(),
                            src: "0".to_string(),
                        });
                        actions.push(format!("Identidade algébrica: multiplicação por 0 -> 0"));
                    }
                    // Redução de força: x * 2 -> x + x ou 2 * x -> x + x (Slide 6 Parte 2)
                    else if op == "*" && (right == "2" || left == "2") {
                        let var = if right == "2" { left } else { right };
                        // Se não for literal puro (já teria dobrado), reduz força
                        if !is_constant(var) {
                            replaced = Some(TacInstr::Binary {
                                dest: dest.clone(),
                                left: var.clone(),
                                op: "+".to_string(),
                                right: var.clone(),
                            });
                            actions.push(format!("Redução de força: {} * 2 -> {} + {}", var, var, var));
                        }
                    }

                    if let Some(new_instr) = replaced {
                        new_instructions.push(new_instr);
                        changed = true;
                    } else {
                        new_instructions.push(instr.clone());
                    }
                }
                _ => new_instructions.push(instr.clone()),
            }
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 4: Eliminação de Subexpressões Comuns (CSE - Parte 2: 9-12)
    /// Se t1 = a * b e depois t2 = a * b no mesmo bloco (e nem a nem b mudaram): t2 = t1
    fn pass_cse(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut new_instructions = Vec::new();

        // Mapa de (op, operando1, operando2) -> variável que guardou o resultado
        let mut expr_map: HashMap<(String, String, String), String> = HashMap::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::Label(_) | TacInstr::Goto(_) | TacInstr::IfFalse { .. } => {
                    expr_map.clear();
                    new_instructions.push(instr.clone());
                }
                TacInstr::Binary { dest, left, op, right } => {
                    // Normaliza para comutativos (+, *, ==, !=)
                    let key = if op == "+" || op == "*" || op == "==" || op == "!=" {
                        if left <= right {
                            (op.clone(), left.clone(), right.clone())
                        } else {
                            (op.clone(), right.clone(), left.clone())
                        }
                    } else {
                        (op.clone(), left.clone(), right.clone())
                    };

                    if let Some(prev_dest) = expr_map.get(&key) {
                        actions.push(format!(
                            "Eliminação de Subexpressão Comum: {} = {} {} {} virou {} = {}",
                            dest, left, op, right, dest, prev_dest
                        ));
                        new_instructions.push(TacInstr::Assign {
                            dest: dest.clone(),
                            src: prev_dest.clone(),
                        });
                        changed = true;
                    } else {
                        expr_map.insert(key, dest.clone());
                        new_instructions.push(instr.clone());
                    }

                    // Se dest era operando em alguma expressão da tabela, remove
                    expr_map.retain(|(_, l, r), _| l != dest && r != dest);
                }
                TacInstr::Assign { dest, .. } => {
                    // Se dest é alterado, qualquer expressão que o usava é invalidada
                    expr_map.retain(|(_, l, r), _| l != dest && r != dest);
                    new_instructions.push(instr.clone());
                }
                _ => new_instructions.push(instr.clone()),
            }
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 5: Simplificação de Desvios (Parte 2: 15)
    /// ifFalse 0 goto L -> goto L
    /// ifFalse 1 goto L -> nada (removido)
    /// goto L seguido de L: -> nada (removido)
    fn pass_branch_simplification(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut filtered: Vec<TacInstr> = Vec::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::IfFalse { cond, target } => {
                    if cond == "0" || cond == "false" {
                        actions.push(format!("Desvio decidido: ifFalse 0 goto {} virou goto {}", target, target));
                        filtered.push(TacInstr::Goto(target.clone()));
                        changed = true;
                    } else if cond == "1" || cond == "true" {
                        actions.push(format!("Desvio decidido: ifFalse 1 goto {} foi removido", target));
                        changed = true; // removido!
                    } else {
                        filtered.push(instr.clone());
                    }
                }
                _ => filtered.push(instr.clone()),
            }
        }

        // Verifica goto L seguido imediatamente de L:
        let mut final_instructions = Vec::new();
        let len = filtered.len();
        for i in 0..len {
            if let TacInstr::Goto(target) = &filtered[i] {
                let clean_target = target.trim_end_matches(':').trim();
                if i + 1 < len {
                    if let TacInstr::Label(l) = &filtered[i + 1] {
                        let clean_label = l.trim_end_matches(':').trim();
                        if clean_target == clean_label {
                            actions.push(format!("Salto redundante: goto {} seguido de {}: foi removido", clean_target, clean_label));
                            changed = true;
                            continue;
                        }
                    }
                }
            }
            final_instructions.push(filtered[i].clone());
        }

        self.instructions = final_instructions;
        changed
    }

    /// Passagem 6: Eliminação de Código Inalcançável (Parte 2: 14-16)
    /// Depois de um goto incondicional, tudo até o próximo rótulo é inalcançável.
    fn pass_unreachable_code(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut new_instructions = Vec::new();
        let mut skipping = false;

        for instr in &self.instructions {
            if skipping {
                if let TacInstr::Label(_) = instr {
                    skipping = false;
                    new_instructions.push(instr.clone());
                } else {
                    actions.push(format!("Código inalcançável removido: {}", instr.to_tac_string()));
                    changed = true;
                }
            } else {
                new_instructions.push(instr.clone());
                if let TacInstr::Goto(_) = instr {
                    skipping = true;
                }
            }
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 7: Peephole / Janela Deslizante (Parte 2: 18-20)
    /// Se tK é usado uma única vez, logo na instrução seguinte, calcule direto em x.
    /// Ex: t5 = contador - 10; contador = t5 => contador = contador - 10
    fn pass_peephole(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut i = 0;
        let mut new_instructions = Vec::new();

        while i < self.instructions.len() {
            if i + 1 < self.instructions.len() {
                let first = &self.instructions[i];
                let second = &self.instructions[i + 1];

                if let Some(dest_first) = first.get_defined_var() {
                    // Verifica se dest_first é um temporário do compilador (t1, t2, ...)
                    if is_temp(dest_first) {
                        if let TacInstr::Assign { dest: dest_second, src: src_second } = second {
                            if src_second == dest_first {
                                // Verifica se dest_first NÃO é lido em nenhum ponto posterior do programa
                                let mut used_elsewhere = false;
                                for instr in &self.instructions[i + 2..] {
                                    let s = instr.to_tac_string();
                                    for word in s.split(|c: char| !c.is_alphanumeric() && c != '_') {
                                        if word == dest_first {
                                            used_elsewhere = true;
                                            break;
                                        }
                                    }
                                    if used_elsewhere {
                                        break;
                                    }
                                }

                                if !used_elsewhere {
                                    match first {
                                        TacInstr::Binary { left, op, right, .. } => {
                                            actions.push(format!(
                                                "Peephole: fundiu temporário {} direto em {}",
                                                dest_first, dest_second
                                            ));
                                            new_instructions.push(TacInstr::Binary {
                                                dest: dest_second.clone(),
                                                left: left.clone(),
                                                op: op.clone(),
                                                right: right.clone(),
                                            });
                                            changed = true;
                                            i += 2;
                                            continue;
                                        }
                                        TacInstr::Assign { src, .. } => {
                                            actions.push(format!(
                                                "Peephole: fundiu cópia de {} direto em {}",
                                                dest_first, dest_second
                                            ));
                                            new_instructions.push(TacInstr::Assign {
                                                dest: dest_second.clone(),
                                                src: src.clone(),
                                            });
                                            changed = true;
                                            i += 2;
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            new_instructions.push(self.instructions[i].clone());
            i += 1;
        }

        self.instructions = new_instructions;
        changed
    }

    /// Passagem 8: Eliminação de Código Morto (DCE - Parte 2: 14, 17, 32)
    /// Remove variáveis ou temporários que nunca são lidos ou são sobrescritos sem leitura.
    /// NUNCA remove param, print, call ou saltos!
    fn pass_dead_code(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;

        // 1. Coleta todas as variáveis lidas no programa
        let mut read_vars = HashSet::new();
        for instr in &self.instructions {
            match instr {
                TacInstr::Assign { src, .. } => {
                    if is_identifier(src) {
                        read_vars.insert(src.clone());
                    }
                }
                TacInstr::Binary { left, right, .. } => {
                    if is_identifier(left) {
                        read_vars.insert(left.clone());
                    }
                    if is_identifier(right) {
                        read_vars.insert(right.clone());
                    }
                }
                TacInstr::IfFalse { cond, .. } => {
                    if is_identifier(cond) {
                        read_vars.insert(cond.clone());
                    }
                }
                TacInstr::Print(arg) | TacInstr::Param(arg) => {
                    if is_identifier(arg) {
                        read_vars.insert(arg.clone());
                    }
                }
                _ => {}
            }
        }

        // 2. Remove atribuições a temporários tK ou variáveis de usuário que NUNCA são lidas (Slide 32: w = 8)
        let mut filtered = Vec::new();
        for instr in &self.instructions {
            if let Some(dest) = instr.get_defined_var() {
                if !instr.has_observable_side_effect() && !read_vars.contains(dest) {
                    actions.push(format!("Código Morto: removida instrução '{}' (nunca lida)", instr.to_tac_string()));
                    changed = true;
                    continue;
                }
            }
            filtered.push(instr.clone());
        }

        // 3. Remove sobrescritas no mesmo bloco antes de qualquer leitura (Slide 14 e 25: contador = 0 e depois contador = 60)
        let mut final_instructions = Vec::new();
        let len = filtered.len();
        for i in 0..len {
            let instr = &filtered[i];
            if let Some(dest) = instr.get_defined_var() {
                if !instr.has_observable_side_effect() {
                    // Olha adiante até o fim do bloco
                    let mut overwritten_without_read = false;
                    for j in (i + 1)..len {
                        let next = &filtered[j];
                        if next.has_observable_side_effect() || matches!(next, TacInstr::Label(_) | TacInstr::Goto(_) | TacInstr::IfFalse { .. }) {
                            // Encontrou rótulo, salto ou impressão: interrompe análise local
                            break;
                        }
                        // Verifica se dest foi lido
                        let is_read = match next {
                            TacInstr::Assign { src, .. } => src == dest,
                            TacInstr::Binary { left, right, .. } => left == dest || right == dest,
                            TacInstr::Print(p) | TacInstr::Param(p) => p == dest,
                            _ => false,
                        };
                        if is_read {
                            break;
                        }
                        // Verifica se dest foi redefinido
                        if let Some(next_dest) = next.get_defined_var() {
                            if next_dest == dest {
                                overwritten_without_read = true;
                                break;
                            }
                        }
                    }

                    if overwritten_without_read {
                        actions.push(format!(
                            "Código Morto: removida sobrescrita redundante '{}' (redefinida antes de ser lida)",
                            instr.to_tac_string()
                        ));
                        changed = true;
                        continue;
                    }
                }
            }
            final_instructions.push(instr.clone());
        }

        self.instructions = final_instructions;
        changed
    }

    /// Passagem 9: Limpeza de Rótulos Não Utilizados (Parte 2: 25)
    /// Rótulos L1, L2 que não são alvos de nenhum salto e não são funções podem sumir.
    fn pass_unused_labels(&mut self, actions: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut target_labels = HashSet::new();

        for instr in &self.instructions {
            match instr {
                TacInstr::Goto(target) => {
                    target_labels.insert(target.trim_end_matches(':').trim().to_string());
                }
                TacInstr::IfFalse { target, .. } => {
                    target_labels.insert(target.trim_end_matches(':').trim().to_string());
                }
                _ => {}
            }
        }

        let mut new_instructions = Vec::new();
        for instr in &self.instructions {
            if let TacInstr::Label(l) = instr {
                let clean = l.trim_end_matches(':').trim();
                // Preserva rótulos de função como 'function main:'
                if !clean.starts_with("function") && !target_labels.contains(clean) {
                    actions.push(format!("Rótulo não utilizado removido: {}", l));
                    changed = true;
                    continue;
                }
            }
            new_instructions.push(instr.clone());
        }

        self.instructions = new_instructions;
        changed
    }

    /// Executa o Motor de Otimização até atingir o PONTO FIXO (Aula 09 Parte 02, Slide 22 e 23)
    /// Limite de 20 rodadas por segurança.
    pub fn optimize(&mut self) -> OptimizationStats {
        let initial_tac: Vec<String> = self.instructions.iter().map(|i| i.to_tac_string()).collect();
        let initial_count = initial_tac.len();
        let initial_temps = Self::count_temps(&self.instructions);
        let blocks_before = Self::identify_basic_blocks(&self.instructions);

        let mut round_logs = Vec::new();
        let mut round = 1;
        let max_rounds = 20;

        while round <= max_rounds {
            let mut round_actions = Vec::new();
            let mut changed_this_round = false;

            // Executa as 9 passagens sequencialmente
            if self.pass_propagation(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_constant_folding(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_algebra_and_strength(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_cse(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_branch_simplification(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_unreachable_code(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_peephole(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_dead_code(&mut round_actions) {
                changed_this_round = true;
            }
            if self.pass_unused_labels(&mut round_actions) {
                changed_this_round = true;
            }

            if !round_actions.is_empty() {
                round_logs.push(RoundLog {
                    round,
                    actions: round_actions,
                });
            }

            // Ponto fixo: uma rodada inteira sem nenhuma mudança (Slide 22 Parte 2)
            if !changed_this_round {
                break;
            }

            round += 1;
        }

        let optimized_tac: Vec<String> = self.instructions.iter().map(|i| i.to_tac_string()).collect();
        let final_count = optimized_tac.len();
        let final_temps = Self::count_temps(&self.instructions);
        let blocks_after = Self::identify_basic_blocks(&self.instructions);

        OptimizationStats {
            initial_instructions: initial_count,
            final_instructions: final_count,
            initial_temps,
            final_temps,
            rounds_executed: round,
            round_logs,
            initial_tac,
            optimized_tac,
            blocks_before,
            blocks_after,
        }
    }
}

// =========================================================================
// INTERPRETADOR / SIMULADOR DE TAC (Aula 09 - Parte 02, Slide 28 e 39)
// "Saída nos dois: comportamento PRESERVADO, provado pelo interpretador de TAC"
// =========================================================================

#[derive(Debug, Clone)]
pub struct InterpreterResult {
    pub output: Vec<String>,
    pub steps: usize,
}

pub struct TacInterpreter;

impl TacInterpreter {
    pub fn execute(instructions: &[String]) -> InterpreterResult {
        let parsed: Vec<TacInstr> = instructions.iter().map(|s| TacInstr::parse(s)).collect();
        let mut env: HashMap<String, i64> = HashMap::new();
        let mut output = Vec::new();
        let mut steps = 0;
        let max_steps = 100_000;

        // Mapeia labels para linhas de instrução
        let mut label_map = HashMap::new();
        for (i, instr) in parsed.iter().enumerate() {
            if let TacInstr::Label(l) = instr {
                let clean = l.trim_end_matches(':').trim();
                label_map.insert(clean.to_string(), i);
            }
        }

        let mut pc = 0;
        let mut pending_params: Vec<String> = Vec::new();

        while pc < parsed.len() && steps < max_steps {
            steps += 1;
            let instr = &parsed[pc];

            match instr {
                TacInstr::Label(_) => {
                    pc += 1;
                }
                TacInstr::Assign { dest, src } => {
                    let val = Self::eval_operand(src, &env);
                    env.insert(dest.clone(), val);
                    pc += 1;
                }
                TacInstr::Binary { dest, left, op, right } => {
                    let l = Self::eval_operand(left, &env);
                    let r = Self::eval_operand(right, &env);
                    let res = match op.as_str() {
                        "+" => l.wrapping_add(r),
                        "-" => l.wrapping_sub(r),
                        "*" => l.wrapping_mul(r),
                        "/" => if r != 0 { l / r } else { 0 },
                        "%" => if r != 0 { l % r } else { 0 },
                        "==" => if l == r { 1 } else { 0 },
                        "!=" => if l != r { 1 } else { 0 },
                        "<" => if l < r { 1 } else { 0 },
                        "<=" => if l <= r { 1 } else { 0 },
                        ">" => if l > r { 1 } else { 0 },
                        ">=" => if l >= r { 1 } else { 0 },
                        "&&" => if l != 0 && r != 0 { 1 } else { 0 },
                        "||" => if l != 0 || r != 0 { 1 } else { 0 },
                        _ => 0,
                    };
                    env.insert(dest.clone(), res);
                    pc += 1;
                }
                TacInstr::IfFalse { cond, target } => {
                    let c = Self::eval_operand(cond, &env);
                    if c == 0 {
                        let clean = target.trim_end_matches(':').trim();
                        if let Some(&target_pc) = label_map.get(clean) {
                            pc = target_pc;
                            continue;
                        }
                    }
                    pc += 1;
                }
                TacInstr::Goto(target) => {
                    let clean = target.trim_end_matches(':').trim();
                    if let Some(&target_pc) = label_map.get(clean) {
                        pc = target_pc;
                        continue;
                    }
                    pc += 1;
                }
                TacInstr::Print(arg) => {
                    let val_str = if arg.starts_with('"') && arg.ends_with('"') {
                        arg[1..arg.len() - 1].to_string()
                    } else if let Some(&v) = env.get(arg) {
                        v.to_string()
                    } else {
                        arg.clone()
                    };
                    output.push(val_str);
                    pc += 1;
                }
                TacInstr::Param(arg) => {
                    let val_str = if arg.starts_with('"') && arg.ends_with('"') {
                        arg[1..arg.len() - 1].to_string()
                    } else if let Some(&v) = env.get(arg) {
                        v.to_string()
                    } else {
                        arg.clone()
                    };
                    pending_params.push(val_str);
                    pc += 1;
                }
                TacInstr::Call { func, .. } => {
                    if func == "print" || func.contains("print") {
                        if !pending_params.is_empty() {
                            output.push(pending_params.join(" "));
                            pending_params.clear();
                        }
                    }
                    pc += 1;
                }
                TacInstr::Raw(_) => {
                    pc += 1;
                }
            }
        }

        InterpreterResult { output, steps }
    }

    fn eval_operand(op: &str, env: &HashMap<String, i64>) -> i64 {
        if let Ok(num) = op.parse::<i64>() {
            num
        } else if let Some(&val) = env.get(op) {
            val
        } else {
            0
        }
    }
}

// Funções utilitárias de verificação léxica
fn is_constant(s: &str) -> bool {
    s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok()
}

fn is_identifier(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || is_constant(s) || (s.starts_with('"') && s.ends_with('"')) {
        return false;
    }
    let first = s.chars().next().unwrap();
    first.is_alphabetic() || first == '_'
}

fn is_temp(s: &str) -> bool {
    s.starts_with('t') && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit())
}
