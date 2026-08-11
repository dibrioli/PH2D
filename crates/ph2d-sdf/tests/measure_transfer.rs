//! **QUANTO custa levar os canais autorados através de um remesh.**
//!
//! O passo 5 do [`ph2d_sdf::remesh`] é uma consulta de ponto mais próximo POR
//! VÉRTICE DE SAÍDA, e o §0 manda medir antes de afirmar qualquer coisa sobre
//! ele. A pergunta é uma só: *ele é ruído contra o campo, ou é o custo do
//! gesto?*
//!
//! # O que ela mediu, e o que a medição mudou
//!
//! **A primeira versão custava 62-79% do remesh** — ela TRIPLICAVA o gesto
//! (2,1 s → 5,5 s a 512). A decomposição (`ph2d-mesh/tests/measure_transfer_probe.rs`)
//! disse onde: a consulta do octree é **4%**, e o resto eram as **75 faces por
//! consulta** que ele devolve (ele responde pelas FOLHAS tocadas, não pelas
//! faces perto do ponto), cada uma com o `TriEdges` reconstruído do zero.
//!
//! Duas correções, as duas exatas:
//!
//! | | µs/vértice |
//! |---|---|
//! | como nasceu | 2,82 |
//! | triângulos PREPARADOS uma vez | 1,50 |
//! | + rejeito por esfera envolvente | **0,49** |
//!
//! ⚠️ **E aumentar a semente do raio PIORA** (893 → 1272 → 2200 ms a 256, com
//! ×1/×3/×8): a busca não estava a crescer o raio, estava a testar faces demais.
//! A medição derrubou o palpite antes de ele virar código.
//!
//! No produto, a 512: a travessia passou de **3,43 s (62%)** para **0,46 s
//! (18%)** de um remesh de 2,5 s.
//!
//! Rodar: `cargo test -p ph2d-sdf --release --test measure_transfer -- --ignored --nocapture`

use std::time::Instant;

fn masked_sphere(rings: usize) -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(rings, rings * 2, 1.0);
    let xs: Vec<f32> = m.positions().iter().map(|p| p[0]).collect();
    let mask = m.masks_mut();
    for (i, x) in xs.iter().enumerate() {
        mask[i] = if *x > 0.0 { 1.0 } else { 0.0 };
    }
    m
}

#[test]
#[ignore = "sonda de medição"]
fn what_the_authored_channels_cost_on_a_rebuild() {
    println!("\n  res | verts saida |  remesh COM |  remesh SEM |  travessia | fracao");
    for res in [64u32, 150, 256, 384, 512] {
        let with_mask = masked_sphere(48);
        let virgin = ph2d_mesh::shapes::uv_sphere(48, 96, 1.0);

        let t = Instant::now();
        let (out, _) = ph2d_sdf::remesh(&with_mask, res).expect("reconstroi");
        let ms_with = t.elapsed().as_secs_f64() * 1e3;
        assert!(out.masks().is_some(), "a mascara tem de ter atravessado");

        let t = Instant::now();
        let (out2, _) = ph2d_sdf::remesh(&virgin, res).expect("reconstroi");
        let ms_without = t.elapsed().as_secs_f64() * 1e3;
        assert!(out2.masks().is_none());

        let delta = ms_with - ms_without;
        println!(
            "  {res:3} | {:11} | {ms_with:8.1} ms | {ms_without:8.1} ms | {delta:7.1} ms | {:5.1}%",
            out.vert_count(),
            delta / ms_with * 100.0
        );
    }
}
