//! **QUANTO custa trocar de matcap** — o preço que o artista paga por CLIQUE.
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test measure_matcap_decode -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`--release` não é preferência:** decodificar um PNG é aritmética por
//! texel, e em debug isto mede o `opt-level=0` em vez do produto.
//!
//! A pergunta que ela responde é se o desenho *"uma textura, reescrita na
//! troca"* se paga contra o alternativo *"nove camadas residentes"*. O
//! alternativo custa 9 MB de VRAM e nove decodificações no boot; este custa uma
//! decodificação por clique — e um clique é o evento mais lento que a UI tem.

/// O custo de decodificar cada uma das nove, e o total que o alternativo
/// pagaria no boot.
#[test]
#[ignore = "sonda de medição"]
fn what_a_matcap_switch_costs() {
    let n = ph2d_mesh_render::MATCAPS.len();
    println!("\n  matcap          ms   KiB(png)   MiB(texels)");
    let mut total = 0.0;
    let mut worst = 0.0f64;
    for i in 0..n {
        // Cada amostra faz o mesmo trabalho ⇒ o mínimo é o redutor certo.
        let mut ms = f64::MAX;
        let mut px = Vec::new();
        for _ in 0..5 {
            let t = std::time::Instant::now();
            px = ph2d_mesh_render::matcap::decode(i);
            ms = ms.min(t.elapsed().as_secs_f64() * 1e3);
        }
        total += ms;
        worst = worst.max(ms);
        println!(
            "  {:<14} {ms:5.2}  {:8}  {:10.2}",
            ph2d_mesh_render::MATCAPS[i],
            ph2d_mesh_render::matcap::MATCAPS[i].png.len() / 1024,
            px.len() as f64 / (1024.0 * 1024.0),
        );
    }
    let side = f64::from(ph2d_mesh_render::MATCAP_SIDE);
    println!(
        "\n  por CLIQUE: {worst:.2} ms no PIOR dos nove (media {:.2})\n  \
         se os nove fossem residentes: {total:.2} ms no boot e {:.1} MiB de VRAM\n  \
         como está: {:.1} MiB de VRAM\n",
        total / n as f64,
        9.0 * side * side * 4.0 / (1024.0 * 1024.0),
        side * side * 4.0 / (1024.0 * 1024.0),
    );
}
