//! Quanto custa ler uma tabela — a medição que decide se este leitor precisa de um TECTO.
use std::time::Instant;
fn main() {
    println!(" linhas  colunas     bytes    leitura   % de um quadro (16,7 ms)");
    for rows in [100usize, 1_000, 10_000, 100_000, 1_000_000] {
        for cols in [4usize, 16] {
            let mut s = String::with_capacity(rows * cols * 8);
            for c in 0..cols {
                if c > 0 {
                    s.push(',');
                }
                s.push_str(&format!("c{c}"));
            }
            s.push('\n');
            for r in 0..rows {
                for c in 0..cols {
                    if c > 0 {
                        s.push(',');
                    }
                    s.push_str(&format!("{}.{}", r % 97, c));
                }
                s.push('\n');
            }
            let mut ms = vec![];
            for _ in 0..3 {
                let t = Instant::now();
                let out = std::hint::black_box(ph2d_table::parse(&s));
                ms.push(t.elapsed().as_secs_f64() * 1e3);
                assert_eq!(out.rows, rows);
            }
            ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!(
                "{rows:7} {cols:8} {:9} {:8.2} ms {:14.1} %",
                s.len(),
                ms[1],
                ms[1] / 16.7 * 100.0
            );
        }
    }
}
