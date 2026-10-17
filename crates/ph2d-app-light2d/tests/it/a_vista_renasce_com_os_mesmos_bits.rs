//! ⭐⭐ W6b — **a vista que renasce dá os MESMOS bits** que a vista nova (handoff W6a §3: o kill-criterion «a
//! imagem muda») **e nenhum quadro sai sem luz**. O renascimento vai pelo caminho do vigia
//! ([`LuzDoQuadro::renascer_a_vista`]); o evento do driver que o dispara não se provoca daqui (o vigia tem os seus
//! testes por leitura).
//!
//! A régua: o High (a holográfica, os níveis em textura) na cena 5, `K` quadros depois de nascer — a nova
//! contra a renascida, ao bit; o controlo: a mesma vista ANTES de renascer, com o rebote mais assente, difere.

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_ecs::PresentWorld;
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::comum::{gpu, ler};

const TAM: (u32, u32) = (1280, 720);
/// Quadros depois de cada nascimento: o rebote do High ainda não assentou (1 por quadro, K9 = 11).
const K: usize = 3;

fn quadros(
    gpu: &ph2d_gpu::GpuContext,
    l: &mut LuzDoQuadro,
    w: &World,
    r: &ph2d_render::SpriteRenderer,
    n: usize,
) -> Vec<u8> {
    let afim = Camera2d::default().world_to_screen_affine(WindowSize::new(TAM.0, TAM.1));
    for _ in 0..n {
        assert!(l.preparar(w, afim, TAM), "a cena 5 tem luz");
        l.gravar(gpu, &mut PresentWorld::new(), r);
        assert!(l.acesa(), "o solver recusou: {:?}", l.erro());
    }
    ler(gpu, l.solver().expect("solver").fluencia(), 8)
}

#[test]
fn a_vista_renascida_da_os_mesmos_bits_que_a_nova() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let r = crate::comum::renderer(&gpu);
    let degrau = degrau_do(NivelDaLuz::Alta);

    let mut nova = LuzDoQuadro::com_degrau(degrau);
    let bits_nova = quadros(&gpu, &mut nova, &w, &r, K);
    let cfg = nova.solver().expect("solver").config();
    assert!(
        cfg.holografica.is_some() && cfg.hrc_niveis_em_textura(),
        "a régua tem de ser o caminho do produto no High (holográfica em textura)"
    );

    let mut l = LuzDoQuadro::com_degrau(degrau);
    let antes = quadros(&gpu, &mut l, &w, &r, K + 1);
    // O pedido vale no quadro SEGUINTE (o do vigia também): larga, espera a placa e monta nele — e `quadros`
    // exige a luz acesa em cada um, o do renascimento incluído.
    l.renascer_a_vista();
    let depois = quadros(&gpu, &mut l, &w, &r, K);
    assert_eq!(l.montagens(), 2, "a vista não renasceu");
    assert_ne!(
        antes, bits_nova,
        "controlo: K+1 quadros sem renascer têm de diferir de K (senão a régua é cega)"
    );
    let diferentes = depois
        .iter()
        .zip(&bits_nova)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        (depois.len(), diferentes),
        (bits_nova.len(), 0),
        "a vista renascida difere da nova em {diferentes} bytes"
    );
}
