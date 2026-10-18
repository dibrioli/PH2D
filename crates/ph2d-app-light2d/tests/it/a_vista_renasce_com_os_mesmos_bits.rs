//! ⭐⭐ W6b/W6c — **a vista que renasce dá os MESMOS bits** que a vista nova (handoff W6a §3: o kill-criterion
//! «a imagem muda») **e nenhum quadro sai sem luz** ([W6c §1](../../../../docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md)):
//! o que o `ligar` vê (`recebida()`) existe depois de TODO `gravar`, o do renascimento incluído. O renascimento vai
//! pelo caminho do vigia ([`LuzDoQuadro::renascer_a_vista`]); o evento do driver que o dispara não se provoca daqui
//! (o vigia tem os seus testes por leitura).
//!
//! A régua: o High (a holográfica, os níveis em textura) na cena 5, `K` quadros depois de nascer — a nova
//! contra a renascida, ao bit; o controlo: a mesma vista ANTES de renascer, com o rebote mais assente, difere. A
//! asserção «sem luz» reprova o renascimento no FIM do quadro do pedido (a C1b da W6b; prova em W6c §3). E os
//! receptores: o pedido solta-os antes do `gravar` ([`LuzDoQuadro::soltar_se_renasce`]) e o `ligar` acende-os.

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
        assert!(l.recebida().is_some(), "um quadro saiu sem luz");
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
    // O pedido vale no quadro SEGUINTE: larga, espera a placa e monta nele — e `quadros` exige a luz em cada um.
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

/// O pedido de renascimento solta os receptores ANTES do `gravar` (presos, seguravam a fluência velha e a nova ia a
/// um bloco a mais do alocador, W6c §2); o `ligar` do mesmo quadro acende-os com a nova. Sem pedido, nada se solta.
#[test]
fn o_pedido_solta_os_receptores_e_o_ligar_do_mesmo_quadro_acende_os() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let mut c = crate::comum::Cadeia::nova(&gpu);
    let afim = Camera2d::default().world_to_screen_affine(WindowSize::new(TAM.0, TAM.1));
    let mut l = LuzDoQuadro::com_degrau(degrau_do(NivelDaLuz::Alta));
    let quadro = |l: &mut LuzDoQuadro, c: &mut crate::comum::Cadeia| {
        let solta = l.soltar_se_renasce(&gpu, &mut c.renderer, &mut c.colagem);
        let solta_e_apagou = !c.renderer.luz_acesa();
        assert!(l.preparar(&w, afim, TAM), "a cena 5 tem luz");
        l.gravar(&gpu, &mut PresentWorld::new(), &c.renderer);
        l.ligar(&gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
        assert!(c.renderer.luz_acesa(), "o ligar não acendeu o receptor");
        (solta, solta_e_apagou)
    };
    assert_eq!(
        quadro(&mut l, &mut c),
        (false, true),
        "o 1.º quadro: nada aceso, nada a soltar"
    );
    assert_eq!(
        quadro(&mut l, &mut c),
        (false, false),
        "sem pedido o receptor fica aceso"
    );
    l.renascer_a_vista();
    assert_eq!(
        quadro(&mut l, &mut c),
        (true, true),
        "o pedido não soltou o receptor"
    );
    assert_eq!(l.montagens(), 2, "a vista não renasceu");
    assert_eq!(
        quadro(&mut l, &mut c),
        (false, false),
        "o pedido soltou mais de uma vez"
    );
}
