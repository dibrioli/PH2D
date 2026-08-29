//! **O PREÇO da rampa normalizada** — o que uma geração FRACCIONÁRIA custa contra uma inteira.
//!
//! A lei mede (percorre a geração anterior e a nova duas vezes) e por isso a travessia custa
//! mais. ⚠️ **Numa geração INTEIRA nada disto corre** — é o caminho de uma cena parada.
use ph2d_node_source_lsystem as ls;
use std::time::Instant;

fn ms(axiom: &str, rules: &str, g: f32, over: &[(&str, f32)]) -> (f32, usize) {
    let mut best = f32::MAX;
    let mut n = 0;
    for _ in 0..7 {
        let t = Instant::now();
        let s = ls::probe_build(axiom, rules, g, over);
        best = best.min(t.elapsed().as_secs_f32() * 1000.0);
        n = s.count();
    }
    (best, n)
}

fn main() {
    println!("orcamento de um quadro: 16,67 ms\n");
    println!("{:26} {:>8} {:>10} {:>10} {:>9}", "caso", "elems", "inteira", "fraccao", "x");
    for p in ls::PRESETS {
        let over: Vec<(&str, f32)> = vec![
            (ls::param::MODE, ls::MODE_GRAMMAR as f32),
            (ls::param::ANGLE, p.angle),
            (ls::param::STEP, p.step),
        ];
        let g = p.generations;
        let (whole, n) = ms(p.axiom, p.rules, g, &over);
        let (frac, _) = ms(p.axiom, p.rules, g - 0.5, &over);
        println!(
            "{:26} {n:>8} {whole:>9.3}ms {frac:>9.3}ms {:>8.1}x",
            p.label,
            frac / whole.max(1e-6)
        );
    }
    // E o pior caso do teto.
    let over = [(ls::param::MODE, ls::MODE_GRAMMAR as f32)];
    let (whole, n) = ms("F", "F -> FF", 20.0, &over);
    let (frac, _) = ms("F", "F -> FF", 19.5, &over);
    println!(
        "\n{:26} {n:>8} {whole:>9.3}ms {frac:>9.3}ms {:>8.1}x   <== o TECTO",
        "F -> FF saturado",
        frac / whole.max(1e-6)
    );
    println!(
        "   ({:.0}% de um quadro na inteira, {:.0}% na fraccao)",
        whole / 16.67 * 100.0,
        frac / 16.67 * 100.0
    );
}
