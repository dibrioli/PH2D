struct Onda { amplitude: f64 }
fn soma(a: i32, b: i32) -> i32 { a + b }
fn dobro(a: i32) -> i32 { a * 2 }
fn main() {
    let o = Onda { amplitude: 1.0 };
    pirnt!("x");
    let y = o.ampiltude * 2.0;
    let s = soma(1);
    let d = dobro("texto");
    let nao_usada = 3;
    println!("{y} {s} {d}");
}
