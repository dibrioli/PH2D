//! ⭐⭐ W6b/W6c — **a vista que renasce dá os MESMOS bits** que a vista nova (handoff W6a §3: o kill-criterion
//! «a imagem muda») **e nenhum quadro sai sem luz** ([W6c §1](../../../../docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md)):
//! o que o `ligar` vê (`recebida()`) existe depois de TODO `gravar`, o do renascimento incluído. O renascimento vai
//! pelo caminho do vigia ([`LuzDoQuadro::renascer_a_vista`]); o evento do driver que o dispara não se provoca daqui
//! (o vigia tem os seus testes por leitura).
//!
//! A régua: o High (a holográfica, os níveis em textura) na cena 5, `K` quadros depois de nascer — a nova
//! contra a renascida, ao bit; os controlos: a mesma vista ANTES de renascer, com o rebote mais assente, difere; e
//! a C1b (no fim do quadro do pedido) deixa o quadro do pedido sem luz.

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

    // Controlo: a C1b larga a vista no FIM do quadro do pedido — o `ligar` dele não vê luz nenhuma.
    let mut c1b = LuzDoQuadro::com_degrau(degrau).renasce_ao("fim");
    quadros(&gpu, &mut c1b, &w, &r, K);
    c1b.renascer_a_vista();
    let afim = Camera2d::default().world_to_screen_affine(WindowSize::new(TAM.0, TAM.1));
    c1b.preparar(&w, afim, TAM);
    c1b.gravar(&gpu, &mut PresentWorld::new(), &r);
    assert!(
        c1b.recebida().is_none(),
        "controlo: a C1b tem de deixar o quadro do pedido sem luz (senão a asserção é cega)"
    );

    for momento in ["adota", "solta"] {
        let mut l = LuzDoQuadro::com_degrau(degrau).renasce_ao(momento);
        let antes = quadros(&gpu, &mut l, &w, &r, K + 1);
        let velha = l.solver().expect("solver").fluencia().clone();
        // O pedido vale no quadro SEGUINTE: larga, espera a placa e monta nele — e `quadros` exige a luz em cada um.
        l.renascer_a_vista();
        let depois = quadros(&gpu, &mut l, &w, &r, K);
        assert_eq!(l.montagens(), 2, "{momento}: a vista não renasceu");
        assert_eq!(
            *l.solver().expect("solver").fluencia() == velha,
            momento == "adota",
            "{momento}: só a (a) adota a textura da fluência"
        );
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
            "{momento}: a vista renascida difere da nova em {diferentes} bytes"
        );
    }

    // A adotada NASCE com os bits de uma nova (zerada). A cena 5 reescreve a fluência inteira antes de a ler, e só
    // uma leitura ANTES do 1.º quadro vê a diferença (sem o zerar, isto reprova e o resto passa).
    let suja = nova.solver().expect("solver").fluencia().clone();
    assert!(
        ler(&gpu, &suja, 8).iter().any(|&b| b != 0),
        "controlo: a fluência da nova tem luz"
    );
    let fabrica = ph2d_light2d_gpu::Fabrica::default();
    let adotada =
        ph2d_light2d_gpu::Solver::novo_adotando(&gpu, *cfg, &fabrica, suja.clone()).expect("adota");
    assert!(*adotada.fluencia() == suja, "a fluência não foi adotada");
    let limpa = ph2d_light2d_gpu::Solver::novo_com(&gpu, *cfg, &fabrica).expect("nova");
    assert!(
        ler(&gpu, adotada.fluencia(), 8) == ler(&gpu, limpa.fluencia(), 8),
        "a fluência adotada não nasce com os bits de uma nova"
    );
}
