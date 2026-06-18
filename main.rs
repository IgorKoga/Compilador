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
    //Verifica se o executável é o "lexico" ou "sintatico"
    //para definir quais partes do código serão executadas.
    let args: Ver<String> = env::args().collect();
    let exec_name = args.get(0).cloned().unwrap_or_default();

    let mut is_lexico_only = exec_name.contains("lexico");
    let is_sintatico_only = exec_name.contains("sintatico");

    #[cfg(feature = "lexico_only")]
    {
        is_lexico_only = true;
    }
    //se não for nenhum dos dois nomes, executa tudo como um unico compilador
    let run_both = !is_lexico_only && !is_sintatico_only;

    let folder = "codigoRust/"; //endereço padrão
    print!("Digite o nome do arquivo: ");
    let _= io::stdout().flush();

    let mut file_name = String::new();
    if let Err(e) = io::stdin().read_line(&mut file_name){`
        eprintln!("Erro ao ler entrada: {}", e);
        return;
    }
    let file_name = file_name.trim(); //remove poluição do buffer
    let full_path = format!("{}{}", folder, file_name);

    //abrir e ler arquivo
    let code = match fs::read_to_string(&full_path){
        Ok(content) => content,
        Err(_) => {
            eprintln!("Erro: O arquivo '{}' nao foi encontrado na pasta!", full_path);
            println!("\nPressione Enter para fechar a janela...");
            let mut temp = String::new();
            let _ = io::stdin().read_line(&mut temp);
            std::process::exit(1);
        }
    };
}
