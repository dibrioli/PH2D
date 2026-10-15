//! Os gates do balão do tutorial «fazer» (Macros W6): o anel no rect do alvo, o cartão perto dele, e
//! a costura inteira de cada botão — pintado → o ponto do meio dele acerta-o por cima de um painel →
//! `dispatch_all` → o pedido no barramento.

use super::*;
use crate::macro_tutorial::{PedidoDoTutorial, set_balao};
use ph2d_a11y::NodeId;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1280.0,
    h: 800.0,
};
const PORTA: NodeId = NodeId(0xB0A7);
const PAINEL: NodeId = NodeId(0x9A1);

fn balao_de(pedido: Option<PedidoDoTutorial>, alvos: Vec<NodeId>) -> BalaoDoTutorial {
    BalaoDoTutorial {
        entity_bits: 77,
        passo: 2,
        total: 4,
        pedido,
        dica: None,
        alvos,
        area: None,
        erro: None,
    }
}

fn liga() -> Option<PedidoDoTutorial> {
    Some(PedidoDoTutorial::Liga {
        de: "field.box".into(),
        para: "force.wind".into(),
    })
}

/// Um quadro: o painel por baixo regista-se e a porta dentro dele; depois o balão.
fn quadro(hero: &mut HeroScreen, b: &BalaoDoTutorial) -> VectorScene {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    hero.hit_index.clear_for_frame();
    hero.hit_index.register(PAINEL, VIEWPORT);
    hero.hit_index
        .register(PORTA, Rect::new(400.0, 300.0, 12.0, 12.0));
    regista(hero);
    paint_balao(hero, b, VIEWPORT, &mut scene, &mut ts);
    scene
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

#[test]
fn o_cartao_fica_ao_lado_do_alvo_e_sem_alvo_no_canto() {
    let mut hero = HeroScreen::new(NodeId(1));
    quadro(&mut hero, &balao_de(liga(), vec![PORTA]));
    let r = hero.hit_index.rect_for(TUTORIAL_BALAO).expect("cartão");
    assert!(r.x >= 412.0 && r.y >= 312.0, "ao lado da porta: {r:?}");

    let mut hero = HeroScreen::new(NodeId(1));
    quadro(&mut hero, &balao_de(liga(), vec![NodeId(0xDEAD)]));
    let r = hero.hit_index.rect_for(TUTORIAL_BALAO).expect("cartão");
    assert!(
        r.x + r.w > VIEWPORT.w * 0.75 && r.y + r.h > VIEWPORT.h * 0.75,
        "alvo fora de vista: no canto de baixo: {r:?}"
    );
}

/// ⭐ A costura dos dois botões: o ponto do meio deles vence o painel por baixo e o clique chega à
/// família como o pedido certo, com a entidade do tutorial.
#[test]
fn os_botoes_chegam_a_familia_como_pedidos() {
    let b = balao_de(liga(), vec![PORTA]);
    set_balao(Some(b.clone()));
    for (id, esperado) in [
        (TUTORIAL_MOSTRA, MacroRequest::MostraMe { entity_bits: 77 }),
        (
            TUTORIAL_RECOMECA,
            MacroRequest::Recomeca { entity_bits: 77 },
        ),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        quadro(&mut hero, &b);
        let (x, y) = centro(hero.hit_index.rect_for(id).expect("pintado"));
        assert_eq!(hero.hit_index.hit(x, y), Some(id), "vence o painel");
        assert!(super::super::dispatch_all(
            &mut hero,
            WidgetEvent::Click(id)
        ));
        let pedidos: Vec<_> = hero
            .bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::Macro(r) => Some(r),
                _ => None,
            })
            .collect();
        assert_eq!(pedidos, [esperado]);
    }
    // Um clique no cartão (fora dos botões) é comido; sem balão publicado, nada.
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(super::super::dispatch_all(
        &mut hero,
        WidgetEvent::Click(TUTORIAL_BALAO)
    ));
    set_balao(None);
    assert!(!apply(&mut hero, WidgetEvent::Click(TUTORIAL_MOSTRA)));
}

/// «Mostra-me» só existe com uma etapa do grafo por fazer: nem no fim, nem antes de entrar em Edit.
#[test]
fn mostra_me_so_com_uma_etapa_por_fazer() {
    for (pedido, tem) in [
        (liga(), true),
        (None, false),
        (Some(PedidoDoTutorial::Edita), false),
        (Some(PedidoDoTutorial::AbreParaPintar), false),
        (
            Some(PedidoDoTutorial::Meio {
                chave_do_meio: "tool.painter.media.wet_paint",
            }),
            false,
        ),
        (Some(PedidoDoTutorial::Pinta), true),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        quadro(&mut hero, &balao_de(pedido.clone(), vec![]));
        assert_eq!(
            hero.hit_index.rect_for(TUTORIAL_MOSTRA).is_some(),
            tem,
            "{pedido:?}"
        );
        assert!(hero.hit_index.rect_for(TUTORIAL_RECOMECA).is_some());
    }
}

#[test]
fn a_frase_diz_o_passo_e_o_que_ligar() {
    let (titulo, corpo) = balao_de(liga(), vec![]).frase();
    assert_eq!(titulo, "Step 2 of 4");
    assert!(corpo.starts_with("Connect "), "{corpo}");
    assert!(!corpo.contains('{'), "sem chave por preencher: {corpo}");
    let (titulo, _) = balao_de(None, vec![]).frase();
    assert_eq!(titulo, ph2d_i18n::tr("panel.macros.tutorial.done_title"));
}

/// As frases do Painter (W6b): o modo Paint, o meio pelo NOME que o seletor mostra, a área marcada.
#[test]
fn as_frases_do_painter_dizem_o_modo_o_meio_e_a_area() {
    let corpo = |p| balao_de(Some(p), vec![]).frase().1;
    assert!(corpo(PedidoDoTutorial::AbreParaPintar).contains("Paint"));
    let meio = corpo(PedidoDoTutorial::Meio {
        chave_do_meio: "tool.painter.media.wet_paint",
    });
    assert!(meio.contains("Wet Paint") && !meio.contains('{'), "{meio}");
    assert!(corpo(PedidoDoTutorial::Pinta).contains("marked area"));
}
