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
/// `Text3` sobre o `Bg1`, o cartão em `CardLabel` sobre o `KanbanCard`), com o `Livre` JÁ COMPOSTO a
/// [`TIMELINE_LOSANGO_LIVRE_ALPHA`] sobre o fundo de quem pinta — é o que o olho vê. Distância OKLab;
/// piso [`PISO_LIVRE_ANIMADO`].
#[test]
fn o_livre_e_o_animado_leem_se_diferentes_em_todo_tema() {
    let lab = |c: ph2d_tokens::Color| {
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c.r, c.g, c.b);
        let h = h.to_radians();
        (l, ch * h.cos(), ch * h.sin())
    };
    let sobre = |c: ph2d_vector::Color, fundo: ph2d_vector::Color| {
        let (c, f) = (vello(c), vello(fundo));
        let a = f32::from(c.a) / f32::from(u8::MAX);
        let mix = |x: u8, y: u8| (f32::from(x) * a + f32::from(y) * (1.0 - a)).round() as u8;
        ph2d_tokens::Color {
            r: mix(c.r, f.r),
            g: mix(c.g, f.g),
            b: mix(c.b, f.b),
            a: u8::MAX,
        }
    };
    let mut pior = (f64::MAX, String::new());
    for theme in Theme::ALL {
        let card = crate::widget::card_ink::CardInk::of_list(theme);
        let classico = |t: ColorToken| resolve(t, theme);
        let cartao = |t: ColorToken| card.color(t);
        let quem: [(&str, ColorToken, Tinta<'_>, ColorToken); 2] = [
            ("clássico", ColorToken::Text3, &classico, ColorToken::Bg1),
            (
                "cartão",
                ColorToken::CardLabel,
                &cartao,
                ColorToken::KanbanCard,
            ),
        ];
        for (nome, apagada, tok, fundo) in quem {
            let p = lab(sobre(
                cor_do_losango(KeyMark::Livre, apagada, theme, tok),
                tok(fundo),
            ));
            let q = lab(vello(cor_do_losango(KeyMark::Animado, apagada, theme, tok)));
            let d = ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt();
            if std::env::var_os("PH2D_MEDE_LOSANGO").is_some() {
                eprintln!("{theme:?} {nome}: {d:.3}");
            }
            if d < pior.0 {
                pior = (d, format!("{theme:?} {nome}"));
            }
            assert!(d >= PISO_LIVRE_ANIMADO, "{theme:?} {nome}: {d:.3}");
        }
    }
    assert!(pior.0 < 1.0, "a régua mediu alguma coisa: {pior:?}");
}

/// Medido 10/11 (Livre composto a 0,75, chave `TimelineTinta::Chave`): pior `PlumberGreen` clássico 0,294,
/// `PlumberRed` 0,308; cartão ≥ 0,393 (o melhor, `Oled` clássico, 0,462). Antes: 0,190 sem opacidade e
/// com a chave `TimelineKey` (o Livre e o Animado liam-se iguais na foto 24_lado_a_lado_losango).
const PISO_LIVRE_ANIMADO: f64 = 0.25;

/// ⭐ **O glifo tem o tamanho da maquete e cabe na sua caixa** (B1 `.c13dm`: 9 px de canto 1,5 girado
/// 45°): a caixa PINTADA (pontos de controlo no transform, mais meio traço no vazio) mede ≥ 11 px de
/// ponta a ponta — o de antes, 6 px de lado, media 8,5 — e não sai da [`m::KEY_MARK_BOX`] no cartão
/// (o rótulo acaba onde ela começa: `cards_rotulos::largura_do_rotulo`) nem da coluna `DECORATOR_W`
/// do clássico. *Mutação: `KEY_MARK_GLYPH` 9 → 6, ou o raio 0 (a ponta aguda sai da caixa).*
#[test]
fn o_glifo_tem_o_tamanho_da_maquete_e_nao_sai_da_caixa() {
    use crate::interaction::HitIndex;
    use crate::widget::card_ink::CardInk;
    use ph2d_card_stack::metrics as m;
    let caixa = |s: &VectorScene, mark: KeyMark| {
        let e = s.inner().encoding();
        let t = *e.transforms.last().expect("o losango gira");
        let pts: Vec<f32> = e.path_data.iter().map(|&b| f32::from_bits(b)).collect();
        let meio = if losango_vazio(mark) {
            m::KEY_MARK_STROKE * 0.5
        } else {
            0.0
        };
        let mut c = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for p in pts.as_chunks::<2>().0 {
            let x = t.matrix[0] * p[0] + t.matrix[2] * p[1] + t.translation[0];
            let y = t.matrix[1] * p[0] + t.matrix[3] * p[1] + t.translation[1];
            c = [
                c[0].min(x - meio),
                c[1].min(y - meio),
                c[2].max(x + meio),
                c[3].max(y + meio),
            ];
        }
        c
    };
    let (cx, cy) = (20.0, 20.0);
    for mark in [
        KeyMark::Livre,
        KeyMark::Animado,
        KeyMark::Chave,
        KeyMark::Mudado,
    ] {
        set_key_marks(BTreeMap::from([(X, mark)]));
        let (mut s, mut hits) = (VectorScene::new(), HitIndex::new());
        let ink = CardInk::of_list(Theme::Dark);
        crate::widget::card_field::paint_key_mark(&mut s, ink, &mut hits, cx, cy, &[X]);
        let c = caixa(&s, mark);
        let (w, h) = (c[2] - c[0], c[3] - c[1]);
        assert!(
            w >= 11.0 && h >= 11.0,
            "{mark:?}: {w:.2}×{h:.2} — o pontinho voltou"
        );
        let metade = m::KEY_MARK_BOX * 0.5;
        assert!(
            c[0] >= cx - metade
                && c[2] <= cx + metade
                && c[1] >= cy - metade
                && c[3] <= cy + metade,
            "{mark:?}: {c:?} sai da caixa de {} px (invade o rótulo)",
            m::KEY_MARK_BOX
        );
        let mut k = VectorScene::new();
        let col = Rect::new(0.0, 0.0, super::super::DECORATOR_W, m::FIELD_H);
        paint_key_mark(&mut k, Theme::Dark, col, mark);
        let c = caixa(&k, mark);
        assert!(
            c[0] >= col.x && c[2] <= col.x + col.w,
            "clássico {mark:?}: {c:?}"
        );
        assert!(c[2] - c[0] >= 11.0, "clássico {mark:?} pequeno");
    }
    assert!(
        (meia_ponta_do_losango() * 2.0 - 11.49).abs() < 0.01,
        "ponta a ponta da maquete"
    );
    set_key_marks(BTreeMap::new());
}

/// A meia extensão do losango de ponta a ponta (a ponta arredondada recua `r·(√2−1)`).
fn meia_ponta_do_losango() -> f64 {
    let (lado, raio) = (
        f64::from(ph2d_card_stack::metrics::KEY_MARK_GLYPH) * 0.5,
        f64::from(ph2d_card_stack::metrics::KEY_MARK_GLYPH_RADIUS),
    );
    (lado - raio) * std::f64::consts::SQRT_2 + raio
}

/// A tinta de quem pinta (o clássico ou o cartão).
type Tinta<'a> = &'a dyn Fn(ColorToken) -> ph2d_vector::Color;

/// A cor vello de volta a sRGB de 8 bits (a régua OKLab mede-a).
fn vello(c: ph2d_vector::Color) -> ph2d_tokens::Color {
    let c = c.to_rgba8();
    ph2d_tokens::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

/// ⭐ **O `Mudado` lê-se LARANJA em todo tema** (maquete B1 `#FF9F5A`, matiz OKLCh ~55°): o matiz cai na
/// faixa [`MATIZ_LARANJA`], e o losango afasta-se da chave e do fundo do cartão e do painel clássico de
/// pelo menos [`PISO_MUDADO`] em OKLab, nos dois Inspectors.
#[test]
fn o_mudado_e_laranja_em_todo_tema() {
    let lab = |c: ph2d_tokens::Color| {
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c.r, c.g, c.b);
        let hr = h.to_radians();
        ((l, ch * hr.cos(), ch * hr.sin()), h)
    };
    let dist = |p: (f64, f64, f64), q: (f64, f64, f64)| {
        ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt()
    };
    let mut pior = f64::MAX;
    for theme in Theme::ALL {
        let (m, h) = lab(vello(cor_mudado(theme)));
        assert!(
            MATIZ_LARANJA.contains(&h),
            "{theme:?}: matiz {h:.1}° não é laranja"
        );
        let (_, hv) = lab(vello(contorno_mudado(theme)));
        assert!(
            MATIZ_LARANJA.contains(&hv),
            "{theme:?}: contorno a {hv:.1}°"
        );
        let card = crate::widget::card_ink::CardInk::of_list(theme);
        let vizinhos = [
            ("chave", ph2d_tokens::TimelineTinta::Chave.resolve(theme)),
            ("cartão", vello(card.color(ColorToken::KanbanCard))),
            ("campo", vello(card.color(ColorToken::CardField))),
            ("painel", ColorToken::Bg1.resolve(theme)),
        ];
        for (nome, c) in vizinhos {
            let d = dist(m, lab(c).0);
            pior = pior.min(d);
            assert!(d >= PISO_MUDADO, "{theme:?}: Mudado × {nome} a {d:.3}");
        }
    }
    assert!(pior < 1.0, "a régua mediu alguma coisa: {pior}");
}

/// O laranja em OKLCh: `#FF9F5A` mede ~55°; o amarelo do `Warn` de antes caía em ~90°–100° («creme»).
const MATIZ_LARANJA: std::ops::RangeInclusive<f64> = 35.0..=75.0;
/// Piso OKLab do Mudado à chave e aos fundos. Medido 10/11 nos 12 temas: o par mais perto é `Oled` × chave a
/// 0,153; cartão ≥ 0,533, campo ≥ 0,506, painel ≥ 0,375. Matiz do losango 41,7°–54,6°, do contorno 53,4°–59,4°.
const PISO_MUDADO: f64 = 0.15;
