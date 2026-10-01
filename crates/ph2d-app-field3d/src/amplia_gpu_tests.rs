//! ⭐⭐⭐⭐ **A IMAGEM AMPLIADA NA PLACA é a do gémeo de CPU sobre o traçado pequeno** — e chega no
//! tamanho da ÁREA ([`ph2d_field_gpu::amplia`], [`crate::gpu_frame::Sonda::entrega`]).
//!
//! As duas metades: o TAMANHO que sai (sem ela, uma entrega que a pintura ignorasse devolvia o
//! traçado pequeno e o ecrã voltava a esticá-lo em bilinear — o report do dono) e a LEI (os bytes
//! são os da [`ph2d_field_gpu::amplia::amplia_cpu`] aplicada ao mesmo quadro sem entrega).

const PEQUENO: (u32, u32) = (640, 360);
const CHEIO: (u32, u32) = (1024, 576);

#[test]
#[ignore = "precisa de GPU"]
fn a_placa_amplia_o_quadro_pequeno_pela_lei_da_cpu() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let pinta = |entrega: Option<(u32, u32)>| {
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &pres,
            [40, 40, 40, 255],
            Some(ph2d_field_render::Ground { height: -1.0 }),
            PEQUENO.0,
            PEQUENO.1,
            false,
            // ⚠️ Sem a oclusão no TEMPO: com ela a 1.ª chamada GRAVA a tabela e a 2.ª herda, e as
            // duas imagens comparadas seriam dois quadros diferentes.
            crate::gpu_frame::Sonda {
                entrega,
                ceu_no_tempo: false,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor do dispositivo")
        .rgba
    };
    let pequeno = pinta(None);
    let grande = pinta(Some(CHEIO));
    assert_eq!(
        pinta(None),
        pequeno,
        "CONTROLO: o mesmo pedido repetido não dá os mesmos bytes — a comparação abaixo mediria ruído"
    );
    assert_eq!(
        pequeno.len(),
        (PEQUENO.0 * PEQUENO.1 * 4) as usize,
        "CONTROLO: sem entrega a imagem é a do traçado"
    );
    assert_eq!(
        grande.len(),
        (CHEIO.0 * CHEIO.1 * 4) as usize,
        "a entrega não chegou à imagem — o ecrã voltaria a esticar o quadro pequeno"
    );
    let gemeo = ph2d_field_gpu::amplia::amplia_cpu(&pequeno, PEQUENO, CHEIO);
    let pior = grande
        .iter()
        .zip(&gemeo)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    // ⚠️ `1` nível: o arredondamento de `f32` entre os dois motores na soma dos 16 pesos.
    assert!(
        pior <= 1,
        "a ampliação da placa não é a lei da CPU: pior {pior} níveis"
    );
}

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
