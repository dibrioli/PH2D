//! A figura DESLOCA-SE durante o arrasto? Um salto de posição lê-se da cadeira como um salto
//! de tamanho, e a lei do crescimento não olha para onde a figura está.
use ph2d_node_source_lsystem::{PRESETS, probe_build};
use ph2d_nodegraph::attr::Column;
fn main() {
    println!("molde     excursão do centroide  ÷ tamanho final   pior salto de um passo ÷ tamanho");
    for p in PRESETS {
        let ov = [("angle", p.angle), ("step", p.step), ("width", p.width)];
        let g0 = (p.generations - 3.0).max(1.0);
        let n = ((p.generations - g0) / 0.02).round() as usize;
        let mut c: Vec<[f32; 2]> = vec![];
        let mut size = 0.0f32;
        for k in 0..=n {
            let s = probe_build(p.axiom, p.rules, g0 + k as f32 * 0.02, &ov);
            let Some(Column::Vec2(v)) = s.get("P") else {
                continue;
            };
            let m = v.len() as f32;
            c.push([
                v.iter().map(|q| q[0]).sum::<f32>() / m,
                v.iter().map(|q| q[1]).sum::<f32>() / m,
            ]);
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            size = (x1 - x0).max(y1 - y0);
        }
        let d = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
        let total = d(c[0], c[c.len() - 1]);
        let worst = c.windows(2).map(|w| d(w[0], w[1])).fold(0.0f32, f32::max);
        println!(
            "{:8}  {:20.1} %  {:32.2} %",
            p.label,
            total / size * 100.0,
            worst / size * 100.0
        );
    }
}
