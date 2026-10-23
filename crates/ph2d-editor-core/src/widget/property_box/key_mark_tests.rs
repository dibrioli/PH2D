//! Gates do losango de animação (plano 34 §6).

use super::*;

const X: NodeId = NodeId(0xA1);
const Y: NodeId = NodeId(0xA2);

/// Um losango por linha: o estado mais forte entre os campos dela; nenhum animado ⇒ `None`.
#[test]
fn a_linha_mostra_o_estado_mais_forte() {
    set_key_marks(BTreeMap::new());
    assert_eq!(key_mark_of(&[X, Y]), None);
    set_key_marks(BTreeMap::from([(X, KeyMark::Animado), (Y, KeyMark::Chave)]));
    assert_eq!(key_mark_of(&[X, Y]), Some(KeyMark::Chave));
    set_key_marks(BTreeMap::from([(X, KeyMark::Mudado), (Y, KeyMark::Chave)]));
    assert_eq!(key_mark_of(&[X, Y]), Some(KeyMark::Mudado));
    assert_eq!(key_mark_of(&[Y]), Some(KeyMark::Chave));
}

/// Só a linha animada vira alvo do clique — pelo id derivado que quem pinta regista — e o alvo
/// devolve os campos dela.
#[test]
fn so_o_losango_pintado_e_alvo() {
    crate::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(X, KeyMark::Animado)]));
    let mut scene = VectorScene::new();
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        paint_decorator_for(&mut scene, Theme::Dark, r, &[X]),
        Some(key_mark_id(X))
    );
    assert_eq!(
        paint_decorator_for(&mut scene, Theme::Dark, r, &[Y]),
        None,
        "o ponto de um campo parado nao e' alvo"
    );
    assert_eq!(
        key_mark_target(key_mark_id(X)),
        Some(KeyMarkClick { fields: vec![X] })
    );
    set_key_marks(BTreeMap::new());
    assert_eq!(
        key_mark_target(key_mark_id(X)),
        None,
        "um quadro novo esvazia os alvos"
    );
}

/// Um campo com canal e sem curva pinta o ponto, mas é ALVO (o 1.º clique anima); um campo sem
/// canal pinta o mesmo ponto e é inerte.
#[test]
fn o_ponto_de_um_campo_com_canal_e_clicavel() {
    crate::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(X, KeyMark::Livre)]));
    let mut scene = VectorScene::new();
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        paint_decorator_for(&mut scene, Theme::Dark, r, &[X]),
        Some(key_mark_id(X))
    );
    assert_eq!(paint_decorator_for(&mut scene, Theme::Dark, r, &[Y]), None);
    assert_eq!(key_mark_of(&[X, Y]), Some(KeyMark::Livre));
    set_key_marks(BTreeMap::new());
}

/// ⭐ **F27 B2 — a linha pintada REGISTA o rótulo do catálogo** de cada campo (um rótulo, ou um por
/// caixa pela ordem dos ids); a linha de VISTA e o texto livre não registam nada.
///
/// ⚠️ Mutação que tem de sangrar SÓ aqui: não escrever em `ROTULOS` (o funil do `ph2d-app-components`
/// também reprova, mas ele não diz em que elo).
#[test]
fn a_linha_pintada_regista_o_rotulo_do_catalogo() {
    use ph2d_i18n::TextKey;
    const CAMPOS: &[TextKey] = &[
        TextKey::new("component.field.a.0"),
        TextKey::new("component.field.b.1"),
    ];
    crate::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::new());
    let (z, w) = (NodeId(0xB2_7001), NodeId(0xB2_7002));
    let mut scene = VectorScene::new();
    let r = Rect::new(0.0, 0.0, 10.0, 10.0);
    let um = TextKey::new("component.field.camera_fields.0");
    let _ = paint_decorator_campo(&mut scene, Theme::Dark, r, &[X], um.into());
    let _ = paint_decorator_campo(
        &mut scene,
        Theme::Dark,
        r,
        &[z, w],
        Rotulo::de_campos("ab", CAMPOS),
    );
    let vista = Rotulo::vista(TextKey::new("component.field.camera_fields.1"));
    let _ = paint_decorator_campo(&mut scene, Theme::Dark, r, &[Y], vista);
    let _ = paint_decorator_campo(&mut scene, Theme::Dark, r, &[Y], "livre".into());
    assert_eq!(rotulo_de(X), Some("component.field.camera_fields.0"));
    assert_eq!(
        (rotulo_de(z), rotulo_de(w)),
        (Some("component.field.a.0"), Some("component.field.b.1"))
    );
    assert_eq!(
        rotulo_de(Y),
        None,
        "a vista e o texto livre não são campo do catálogo"
    );
    assert_eq!(rotulos_pintados().len(), 3);
    set_key_marks(BTreeMap::new());
    assert!(
        rotulos_pintados().is_empty(),
        "um quadro novo esquece o que não voltou a ser pintado"
    );
}
