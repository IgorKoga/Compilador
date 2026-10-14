// Exemplo 3 dos slides das Aulas 09 (Parte 01 e Parte 02)
// Compiladores - UENP 2026 - Prof. Luciano Rovanni
// Demonstração completa: IF + WHILE com 14 -> 7 instruções e saída [60]
fn main() {
    let mut total = 10 + 20;
    let mut contador = 0;
    if total == 30 {
        contador = total * 2;
        println!("{}", contador);
    }
    while contador > 0 {
        contador = contador - 10;
    }
}
