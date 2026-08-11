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

/// ⚠️ **A travessia é cronometrada DIRETAMENTE, não por subtração.**
///
/// A primeira versão desta sonda media *remesh com máscara* menos *remesh sem*
/// — dois números de ~2,4 s para extrair um item de ~0,3 s. Com a máquina
/// compartilhada isso devolveu **travessia NEGATIVA (−299,7 ms, −55,5%)**: o
/// ruído do maior engoliu o menor. *Uma diferença entre dois números grandes
/// não mede um número pequeno.*
///
/// A entrada é a MESMA do produto — a malha de origem que o artista esculpiu e
/// a saída do Surface Nets daquela resolução —, então isto continua a sair pela
/// porta do produto; o que mudou foi o relógio, que agora cerca só o passo 5.
#[test]
#[ignore = "sonda de medição"]
fn what_the_authored_channels_cost_on_a_rebuild() {
    let with_mask = masked_sphere(48);
    let virgin = ph2d_mesh::shapes::uv_sphere(48, 96, 1.0);

    println!("\n  res | verts saida |   remesh |  travessia | fracao |  ns/vertice");
    for res in [64u32, 150, 256, 384, 512] {
        // O remesh de uma malha VIRGEM devolve a mesma casca sem pagar a
        // travessia (o passo 5 sai no primeiro `if`), então ele mede o gesto
        // sem o item, e a saída dele é exatamente o destino que o item recebe.
        let t = Instant::now();
        let (mut out, _) = ph2d_sdf::remesh(&virgin, res).expect("reconstroi");
        let ms_remesh = t.elapsed().as_secs_f64() * 1e3;
        assert!(out.masks().is_none(), "a malha virgem nao carrega plano");
        let n = out.vert_count();

        // Cada amostra faz o mesmo trabalho ⇒ o mínimo é o redutor certo.
        let mut ms_transfer = f64::MAX;
        for _ in 0..5 {
            let t = Instant::now();
            ph2d_mesh::transfer_authored(&with_mask, &mut out);
            ms_transfer = ms_transfer.min(t.elapsed().as_secs_f64() * 1e3);
        }
        assert!(out.masks().is_some(), "a mascara tem de ter atravessado");

        println!(
            "  {res:3} | {n:11} | {ms_remesh:6.1} ms | {ms_transfer:7.1} ms | {:5.1}% | {:6.1}",
            ms_transfer / (ms_remesh + ms_transfer) * 100.0,
            ms_transfer * 1e6 / n as f64,
        );
    }
}
