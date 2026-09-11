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
            ph2d_mesh_render::matcap::MATCAPS[i].bytes.len() / 1024,
            px.len() as f64 / (1024.0 * 1024.0),
        );
    }
    // RGBA de meio-float = 8 bytes por texel, e o lado é POR-MATCAP.
    let mib = |m: &ph2d_mesh_render::Matcap| {
        f64::from(m.side) * f64::from(m.side) * 8.0 / (1024.0 * 1024.0)
    };
    let todos: f64 = ph2d_mesh_render::matcap::MATCAPS.iter().map(mib).sum();
    let maior = ph2d_mesh_render::matcap::MATCAPS
        .iter()
        .map(mib)
        .fold(0.0f64, f64::max);
    println!(
        "\n  por CLIQUE: {worst:.2} ms no PIOR dos dez (media {:.2})\n  \
         se os dez fossem residentes: {total:.2} ms no boot e {todos:.1} MiB de VRAM\n  \
         como está: {maior:.1} MiB de VRAM (o maior lado)\n",
        total / n as f64,
    );
}
