//! ⭐⭐⭐⭐ **A IMAGEM AMPLIADA NA PLACA é a do gémeo de CPU sobre o traçado pequeno** — e chega no
//! tamanho da ÁREA ([`ph2d_field_gpu::amplia`], [`crate::gpu_frame::Sonda::entrega`]).
//!
//! As duas metades: o TAMANHO que sai (sem ela, uma entrega que a pintura ignorasse devolvia o
//! traçado pequeno e o ecrã voltava a esticá-lo em bilinear — o report do dono) e a LEI (os bytes
//! são os da [`ph2d_field_gpu::amplia::amplia_cpu`] aplicada ao mesmo quadro sem entrega).

const PEQUENO: (u32, u32) = (640, 360);
const CHEIO: (u32, u32) = (1024, 576);

/// ⭐⭐⭐⭐ **E o MATCAP também** — o modo de omissão do modelador entra na resolução dinâmica pela
/// mesma porta ([`ph2d_field_gpu::matcap::MatcapSetup::entrega`]), e sem ela o quadro grosso voltava
/// a ser esticado em bilinear no ecrã.
#[test]
#[ignore = "precisa de GPU"]
fn a_placa_amplia_o_matcap_pela_lei_da_cpu() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    // Uma fotografia de `2×2` texels com quatro cores — chega para a imagem ter arestas a ampliar.
    let rgb = [
        0.9f32, 0.2, 0.1, 0.1, 0.8, 0.2, 0.2, 0.1, 0.9, 0.7, 0.7, 0.7,
    ];
    let look = ph2d_view_transform::Look::default();
    let pinta = |entrega: Option<(u32, u32)>| {
        let mc = ph2d_field_gpu::matcap::MatcapSetup {
            rgb_linear: &rgb,
            side: 2,
            chave: 0xA0B1_C2D3_E4F5_0617,
            stops: look.exposure_stops,
            view: ph2d_view_transform::wgsl::view_code(look.view),
            background: [40, 40, 40, 255],
            entrega,
        };
        crate::gpu_frame::pinta_matcap(t, &doc, &reg, &cam, &mc, PEQUENO.0, PEQUENO.1)
            .expect("o matcap do dispositivo")
            .rgba
    };
    let pequeno = pinta(None);
    let grande = pinta(Some(CHEIO));
    assert_eq!(
        pinta(None),
        pequeno,
        "CONTROLO: o mesmo pedido repetido não dá os mesmos bytes"
    );
    assert_eq!(
        grande.len(),
        (CHEIO.0 * CHEIO.1 * 4) as usize,
        "a entrega não chegou à imagem do matcap — o ecrã voltaria a esticar o quadro pequeno"
    );
    let gemeo = ph2d_field_gpu::amplia::amplia_cpu(&pequeno, PEQUENO, CHEIO);
    let pior = grande
        .iter()
        .zip(&gemeo)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 1,
        "a ampliação do matcap não é a lei da CPU: pior {pior} níveis"
    );
}
