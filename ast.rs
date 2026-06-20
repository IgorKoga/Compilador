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
