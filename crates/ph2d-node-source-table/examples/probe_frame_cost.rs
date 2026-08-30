//! O que a tabela custa POR QUADRO — o leitor corre uma vez por ficheiro, mas o nó copia as
//! colunas em toda cozedura. É esta a grandeza que o orçamento de 16,7 ms mede.
use ph2d_nodegraph::attr::{Column, Stream};
use std::time::Instant;
fn main() {
    println!(" linhas  colunas    MB/quadro   copia   % de um quadro (16,7 ms)");
    for rows in [1_000usize, 10_000, 100_000, 1_000_000] {
        for cols in [4usize, 16] {
            let mut src = Stream::new(rows);
            for c in 0..cols {
                src.set(format!("c{c}"), Column::Scalar(vec![c as f32; rows]));
            }
            let mut ms = vec![];
            for _ in 0..5 {
                let t = Instant::now();
                let mut out = Stream::new(rows).with(
                    "P",
                    Column::Vec2((0..rows).map(|i| [i as f32, 0.0]).collect::<Vec<_>>()),
                );
                for (name, col) in src.columns() {
                    out.set(name.clone(), col.clone());
                }
                out.set(
                    "Index",
                    Column::Scalar((0..rows).map(|i| i as f32).collect::<Vec<_>>()),
                );
                std::hint::black_box(&out);
                ms.push(t.elapsed().as_secs_f64() * 1e3);
            }
            ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mb = (rows * (cols + 3) * 4) as f64 / 1e6;
            println!(
                "{rows:7} {cols:8} {mb:11.2} {:8.2} ms {:14.1} %",
                ms[2],
                ms[2] / 16.7 * 100.0
            );
        }
    }
}
