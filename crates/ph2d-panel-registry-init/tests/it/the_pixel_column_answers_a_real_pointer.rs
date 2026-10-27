//! ⭐⭐⭐ **A coluna do Pixel Sprite ▸ Draw responde a um PONTEIRO real** (report do dono 25/10: as
//! ferramentas à esquerda, como no Painter, e o seletor de modo no topo).
//!
//! Com todos os painéis registados, a coluna que a família publica é pintada pelo ecrã; um Down+Up
//! sobre cada ferramenta tem de virar `Click` e chegar ao barramento como o pedido dela — com o
//! painel PIXEL FECHADO (a coluna não depende dele); cada amostra abre o seletor de cor; e a fila de
//! cima pinta o seletor de modo. P6b/P6c: as selecções e as formas moram em botões de grupo — cada
//! um pede o grupo dele, e as quatro respondem no flyout aberto (um de cada vez).

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{AreaMenu, ToolBars, WidgetEvent};
use ph2d_editor_core::pixel_panel::{
    PixelGroups, PixelPanelInfo, PixelRequest, PixelTool, PixelToolGroup,
};
use ph2d_editor_core::screens::hero::float_bar::tool_bars::placed;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{column, ids};

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn clica(hero: &mut HeroScreen, r: Rect) -> Vec<WidgetEvent> {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    ev
}

#[test]
fn every_tool_and_swatch_of_the_pixel_column_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut ferramentas = Vec::new();
    for g in PixelToolGroup::ALL {
        clica_tudo(&mut ts, Some(g), &mut ferramentas);
    }
    ferramentas.sort_by_key(|t: &PixelTool| PixelTool::ALL.iter().position(|u| u == t));
    ferramentas.dedup();
    assert_eq!(
        ferramentas,
        PixelTool::ALL,
        "a coluna (com os flyouts dos grupos) perdeu uma ferramenta"
    );
}

/// A coluna com o flyout de `aberto`: cada botão clicado, e as ferramentas que responderam.
fn clica_tudo(
    ts: &mut ph2d_text::TextSystem,
    aberto: Option<PixelToolGroup>,
    ferramentas: &mut Vec<PixelTool>,
) {
    let mut groups = PixelGroups::default();
    groups.open = aberto;
    let info = PixelPanelInfo {
        editing: true,
        tool: PixelTool::Pencil,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 255],
        palette: vec![[1, 2, 3, 255]; 32],
        options: Default::default(),
        groups,
        ..Default::default()
    };
    let mut hero = HeroScreen::new(NodeId(1));
    // O 1.º quadro arruma os encaixes; a coluna cai na área do 2.º (o mesmo que o teste do vetor).
    pinta(&mut hero, ts);
    hero.panel_visibility
        .insert(ph2d_editor_core::screens::hero::PANEL_PIXEL, false);
    ph2d_panel_pixel_tools::set_current_pixel(info.clone());
    hero.store.publish_mode_menu(Some(AreaMenu {
        label: ph2d_i18n::tr("object_mode.menu").to_owned(),
        face: ph2d_i18n::tr("object_mode.draw").to_owned(),
        faces: Vec::new(),
        rows: Vec::new(),
    }));
    hero.store.set_tool_bars(Some(ToolBars {
        pixel_rail: true,
        ..column::bars(&info)
    }));
    pinta(&mut hero, ts);
    assert!(
        hero.hit_index
            .rect_for(ph2d_editor_core::ids::area_menu_button(0))
            .is_some(),
        "o seletor de modo não foi pintado na fila de cima"
    );
    let bars = hero.store.tool_bars().cloned().expect("publicadas");
    for (b, r) in placed(&bars, hero.last_content, false, false) {
        if hero.store.is_picker_swatch(b.id) {
            let _ = clica(&mut hero, r);
            assert_eq!(
                hero.store.picker_target(),
                Some(b.id),
                "a amostra «{}» não abriu a cor",
                b.label
            );
            hero.store.set_picker_target(None);
            continue;
        }
        let ev = clica(&mut hero, r);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(i) if *i == b.id)),
            "o botão «{}» em {r:?} está pintado e o ponteiro não o clica ({ev:?}); quem responde: {:?}",
            b.label,
            ev.iter().find_map(|e| match e {
                WidgetEvent::Click(i) => Some((*i, hero.hit_index.rect_for(*i))),
                _ => None,
            })
        );
        let _ = hero.bus.drain().count();
        for e in ev {
            hero.apply_event(e);
        }
        let pedidos: Vec<_> = hero
            .bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::Pixel(r) => Some(r),
                _ => None,
            })
            .collect();
        if let Some((g, _)) = ids::GROUPS.iter().find(|(_, id)| *id == b.id) {
            assert_eq!(pedidos, [PixelRequest::Group(*g)]);
        } else if let Some((t, _)) = ids::TOOLS.iter().find(|(_, id)| *id == b.id) {
            assert_eq!(pedidos, [PixelRequest::Tool(*t)], "«{}»", b.label);
            ferramentas.push(*t);
        } else {
            assert_eq!(b.id, ids::PIXEL_SWAP);
            assert_eq!(pedidos, [PixelRequest::Swap]);
        }
    }
}

/// P6b: os campos das formas do Pixel são pintados pela MESMA ponte de nomes do painel do Vector —
/// o registo injecta-a (um painel não depende do outro).
#[test]
fn the_pixel_shape_fields_are_named_by_the_vector_panels_bridge() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for s in ph2d_editor_core::pixel_panel::PixelShape::ALL {
        for f in ph2d_panel_pixel_tools::shape_desc(s).fields {
            assert_eq!(
                ph2d_panel_pixel_tools::shape_field_name(f.label),
                ph2d_panel_vector::nomes_do_motor::campo(f.label),
                "{s:?} «{}»",
                f.label
            );
        }
    }
}

/// P6e: cada forma do Pixel chama-se como no flyout do Vector — no botão, no tooltip e no aviso do
/// atalho (o núcleo nomeia-a pela ponte que o registo injecta).
#[test]
fn the_pixel_shapes_are_named_as_in_the_vector_flyout() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for d in ph2d_tool_vector::shapes::SHAPES {
        let t = PixelTool::Shape(ph2d_editor_core::pixel_panel::PixelShape::new(d.kind));
        let quer = ph2d_panel_vector::nomes_do_motor::forma(d.label);
        assert_eq!(t.name(), quer, "{:?}", d.kind);
        let mut groups = PixelGroups::default();
        groups.open = Some(PixelToolGroup::Shape);
        let botao = column::bars(&PixelPanelInfo {
            editing: true,
            tool: t,
            groups,
            ..PixelPanelInfo::default()
        })
        .flyout
        .and_then(|(_, b)| b.into_iter().find(|b| b.on))
        .expect("a forma na mão está acesa no flyout");
        assert_eq!(botao.label, t.named(), "{:?}", d.kind);
        assert!(
            botao.label.starts_with(quer),
            "{:?}: {}",
            d.kind,
            botao.label
        );
    }
}
