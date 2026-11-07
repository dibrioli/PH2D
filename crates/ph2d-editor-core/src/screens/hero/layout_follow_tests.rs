//! A aba acompanha o tipo do objecto (dono 07/11).

use super::*;
use crate::tool::ToolRegistry;
use ph2d_a11y::NodeId;
use ph2d_component_desc::ObjectKind;

#[test]
fn cada_tipo_tem_a_sua_aba() {
    for (kind, aba) in [
        (ObjectKind::Empty, TaskLayout::Scene),
        (ObjectKind::Image, TaskLayout::Scene),
        (ObjectKind::Skeleton, TaskLayout::Scene),
        (ObjectKind::PixelSprite, TaskLayout::Scene),
        (ObjectKind::Vector, TaskLayout::Vector),
        (ObjectKind::Painted, TaskLayout::Drawing2d),
        (ObjectKind::Flip, TaskLayout::Flip),
        (ObjectKind::Motion, TaskLayout::Nodes),
    ] {
        assert_eq!(TaskLayout::of_kind(kind), aba, "{kind:?}");
    }
}

/// Mutação: tirar o teste da aba Animate ⇒ animar um objecto de outro tipo tira o artista da Timeline.
#[test]
fn em_animate_a_aba_nao_muda() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.set_active_layout(TaskLayout::Animation);
    follow_object(&mut hero, &mut ToolRegistry::default(), ObjectKind::Vector);
    assert_eq!(hero.store.active_layout(), TaskLayout::Animation);
}

/// A aba muda e o painel que o artista abriu fica; só troca o que é da aba.
/// Mutação: fechar tudo como o clique na aba ⇒ os Assets abertos somem a cada objecto clicado.
#[test]
fn acompanhar_nao_fecha_o_painel_do_artista() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.set_active_layout(TaskLayout::Vector);
    hero.panel_visibility.insert("asset_browser", true);
    follow_object(&mut hero, &mut ToolRegistry::default(), ObjectKind::Motion);
    assert_eq!(hero.store.active_layout(), TaskLayout::Nodes);
    assert_eq!(hero.panel_visibility.get("asset_browser"), Some(&true));
    assert_eq!(hero.panel_visibility.get("timeline"), Some(&true));
    follow_object(&mut hero, &mut ToolRegistry::default(), ObjectKind::Image);
    assert_eq!(hero.store.active_layout(), TaskLayout::Scene);
    assert_eq!(hero.panel_visibility.get("timeline"), Some(&false));
    assert_eq!(hero.panel_visibility.get("asset_browser"), Some(&true));
}
