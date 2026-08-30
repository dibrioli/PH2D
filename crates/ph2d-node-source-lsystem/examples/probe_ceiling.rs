//! ⭐ O PIOR CASO REAL da rota fraccionária — e ele NÃO é o Dragon.
//!
//! A cadeia satura no mesmo número de módulos para toda a gramática (o orçamento), mas a
//! fracção DESENHADA muda tudo: o Dragon é metade viragens; um `F -> FFFF` é tudo `F`.
//! ⚠️ E `F -> FFFF` a `8,5` gerações está **dentro do arrasto do slider** (0..12).
use ph2d_node_source_lsystem::probe_build;
use ph2d_nodegraph::attr::Column;
use std::time::Instant;
fn main() {
    println!("gramática              gens   desenhados   cozedura   % de um quadro");
    for (rules, g) in [
        ("F -> F+G ; G -> F-G", 12.0f32),
        ("F -> F+G ; G -> F-G", 16.99),
        ("F -> F[+F]F[-F]F", 6.5),
        ("F -> FFFF", 8.5),
        ("F -> FF", 17.5),
    ] {
        let mut ms = vec![];
        let mut n = 0;
        for _ in 0..3 {
            let t = Instant::now();
            let s = probe_build("F", rules, g, &[("angle", 90.0), ("step", 0.02)]);
            ms.push(t.elapsed().as_secs_f64() * 1e3);
            n = match s.get("P") {
                Some(Column::Vec2(v)) => v.len(),
                _ => 0,
            };
        }
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "{rules:22} {g:5.2} {n:12}   {:7.2} ms   {:8.0} %",
            ms[1],
            ms[1] / 16.7 * 100.0
        );
    }
}
