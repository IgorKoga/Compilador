use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod ast;
mod lexico;
mod sintatico;

use lexico::{token_type_to_string, Scanner, TokenType};
use sintatico::Parser;

fn main() {
    // Verifica se o executável é o "lexico" ou "sintatico"
    // para definir quais partes do código serão executadas.
    let args: Vec<String> = env::args().collect();
    let exec_name = args.get(0).cloned().unwrap_or_default();

    let is_lexico_only = exec_name.contains("lexico");
    let is_sintatico_only = exec_name.contains("sintatico");

    #[cfg(feature = "lexico_only")]
    {
        is_lexico_only = true;
    }
    // se não for nenhum dos dois nomes, executa tudo como um unico compilador
    let run_both = !is_lexico_only && !is_sintatico_only;

    let folder = "codigoRust/"; // endereço padrão
    print!("Digite o nome do arquivo: ");
    let _ = io::stdout().flush();

    let mut file_name = String::new();
    if let Err(e) = io::stdin().read_line(&mut file_name) {
        eprintln!("Erro ao ler entrada: {}", e);
        return;
    }
    let file_name = file_name.trim(); // remove poluição do buffer
    let full_path = format!("{}{}", folder, file_name);

    // abrir e ler arquivo
    let code = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!(
                "Erro: O arquivo '{}' nao foi encontrado na pasta!",
                full_path
            );
            println!("\nPressione Enter para fechar a janela...");
            let mut temp = String::new();
            let _ = io::stdin().read_line(&mut temp);
            std::process::exit(1);
        }
    };

    // cria o scanner para analise lexica
    let mut lex_scanner = Scanner::new(&code);

    if is_lexico_only || run_both {
        if run_both {
            println!("\n============================================================");
            println!("                  1. FASE LEXICA (TOKENS)                   ");
            println!("============================================================");
        } else {
            println!();
        }

        // cabeçalho da tabela
        println!("{}", "-".repeat(60));
        println!("{:<20} {:<30} {}", "Token", "Lexema", "Linha");
        println!("{}", "-".repeat(60));

        // continua analise até o fim da entrada
        let mut token = lex_scanner.next_token();
        while token.r#type != TokenType::T_EOF {
            println!(
                "{:<20} {:<30} {}",
                token_type_to_string(token.r#type.clone()),
                token.lexeme,
                token.line
            );
            token = lex_scanner.next_token();
        }
        println!("{}", "-".repeat(60));
        println!("Fim da analise lexica.");
    }

    // se compilado sem o sinal lexico_only
    #[cfg(not(feature = "lexico_only"))]
    {
        if is_sintatico_only || run_both {
            if run_both {
                println!("\n============================================================");
                println!("                  2. FASE SINTATICA (ARVORE E JSON)         ");
                println!("============================================================");
            }

            // Cria um scanner separado para o parser
            let mut parser_scanner = Scanner::new(&code);
            let mut parser = Parser::new(&mut parser_scanner);

            // Tratamento de erro do parse
            let ast = parser.parseProgram();
            // Exibe a representação da árvore sintatica (AST) no terminal
            println!("\n--- Representacao da Arvore Sintatica (AST) ---");
            ast.print(0); // Exibe a árvore de forma hierárquica
            println!("-----------------------------------------------");

            let path = Path::new(&full_path);
            let base_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_name);

            // remove a extensão original .rs do arquivo e adiciona a .json
            let json_file_name = if let Some(last_dot) = base_name.rfind('.') {
                format!("{}.json", &base_name[..last_dot])
            } else {
                format!("{}.json", base_name)
            };

            let json_path = format!("json/{}", json_file_name);

            // Cria o diretorio para os arquivos json se ele não existir
            if let Some(parent) = Path::new(&json_path).parent() {
                let _ = fs::create_dir_all(parent);
            }

            // grava representação em json gerada
            match fs::write(&json_path, ast.to_json()) {
                Ok(_) => {
                    println!("\n[Sucesso] Arquivo json salvo em: {}", json_path);
                }
                Err(e) => {
                    eprintln!(
                        "\n[Erro] Nao foi possivel criar arquivo JSON em: {}. Erro: {}",
                        json_path, e
                    );
                }
            }
        }

        if run_both {
            println!("============================================================");
        }
        println!("Fim da analise sintatica.");
    }

    println!("\nPressione Enter para fechar a janela...");
    let mut temp = String::new();
    let _ = io::stdin().read_line(&mut temp);
}
