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
        Some(KeyMarkClick {
            fields: vec![X],
            sujeito: None,
        })
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
        TextKey::new("component.field.label_fields.3"),
        TextKey::new("component.field.label_fields.4"),
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
        (
            Some("component.field.label_fields.3"),
            Some("component.field.label_fields.4")
        )
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

// ── onda 1 — losango (plano `docs/Timeline/17` item 3): os estados do Rive ──────────────────────

/// Estilos e geometria de uma cena (a tinta fica de fora).
fn forma(scene: &VectorScene) -> (String, Vec<u32>) {
    let e = scene.inner().encoding();
    (format!("{:?}", e.styles), e.path_data.clone())
}

fn classico(mark: KeyMark) -> (String, Vec<u32>) {
    let mut s = VectorScene::new();
    paint_key_mark(&mut s, Theme::Dark, Rect::new(0.0, 0.0, 10.0, 10.0), mark);
    forma(&s)
}

/// ⭐ `Livre` é o MESMO losango vazio de `Animado` (contorno, não o ponto); `Chave` e `Mudado` cheios.
#[test]
fn o_livre_e_um_losango_vazio_como_o_animado() {
    let mut ponto = VectorScene::new();
    super::super::row::paint_decorator(
        &mut ponto,
        Theme::Dark,
        Rect::new(0.0, 0.0, 10.0, 10.0),
        false,
    );
    assert_ne!(
        classico(KeyMark::Livre),
        forma(&ponto),
        "o Livre voltou a ser o ponto"
    );
    assert_eq!(classico(KeyMark::Livre), classico(KeyMark::Animado));
    assert_eq!(classico(KeyMark::Chave), classico(KeyMark::Mudado));
    assert_ne!(
        classico(KeyMark::Chave).0,
        classico(KeyMark::Animado).0,
        "cheio × contorno"
    );
}

/// O cartão pinta pela mesma porta: `Livre` = losango vazio, alvo do MESMO tamanho de antes
/// (`KEY_MARK_BOX`); sem canal = o ponto inerte, sem alvo.
#[test]
fn no_cartao_o_livre_e_losango_e_o_alvo_nao_encolhe() {
    use crate::interaction::HitIndex;
    use crate::widget::card_ink::CardInk;
    use ph2d_card_stack::metrics as m;
    let pinta = |mark: Option<KeyMark>| {
        set_key_marks(mark.map(|k| BTreeMap::from([(X, k)])).unwrap_or_default());
        let (mut s, mut hits) = (VectorScene::new(), HitIndex::new());
        let ink = CardInk::of_list(Theme::Dark);
        crate::widget::card_field::paint_key_mark(&mut s, ink, &mut hits, 20.0, 20.0, &[X]);
        (forma(&s), hits.rect_for(key_mark_id(X)))
    };
    let (livre, alvo) = pinta(Some(KeyMark::Livre));
    let (animado, _) = pinta(Some(KeyMark::Animado));
    let (sem, sem_alvo) = pinta(None);
    assert_eq!(livre, animado, "o Livre é o contorno do Animado");
    assert_ne!(livre, sem, "o campo sem canal fica o ponto");
    assert_eq!(sem_alvo, None);
    let alvo = alvo.expect("o Livre é clicável");
    assert!(
        (alvo.w - m::KEY_MARK_BOX).abs() < f32::EPSILON
            && (alvo.h - m::KEY_MARK_BOX).abs() < f32::EPSILON
    );
    set_key_marks(BTreeMap::new());
}

/// ⭐ `Livre` × `Animado` leem-se diferentes em TODO tema, nos dois Inspectors (o clássico apaga em
/// `Text3`, o cartão em `CardLabel`, que num cartão de tema próprio é `Text3`). Distância OKLab;
/// piso [`PISO_LIVRE_ANIMADO`].
#[test]
fn o_livre_e_o_animado_leem_se_diferentes_em_todo_tema() {
    let lab = |c: ph2d_tokens::Color| {
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c.r, c.g, c.b);
        let h = h.to_radians();
        (l, ch * h.cos(), ch * h.sin())
    };
    let mut pior = f64::MAX;
    for theme in Theme::ALL {
        for apagada in [ColorToken::Text3, ColorToken::CardLabel] {
            let p = lab(tinta_do_losango(KeyMark::Livre, apagada).resolve(theme));
            let q = lab(tinta_do_losango(KeyMark::Animado, apagada).resolve(theme));
            let d = ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt();
            pior = pior.min(d);
            assert!(d >= PISO_LIVRE_ANIMADO, "{theme:?} {apagada:?}: {d:.3}");
        }
    }
    assert!(pior < 1.0, "a régua mediu alguma coisa: {pior}");
}

/// Medido 10/11: o par mais perto é `PlumberGreen`/`Text3` a 0,190 (o melhor, `Forge`/`CardLabel`, 0,380);
/// a maquete B1 tem `--fg2` × `--c13-k` a 0,214 sólido e 0,31 com a opacidade 0,75 sobre o cartão.
const PISO_LIVRE_ANIMADO: f64 = 0.15;
