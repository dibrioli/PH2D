//! Gates do [`super`] (`marcas.rs`) — o losango de cada campo do painel do vetor.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 7;
const SEM: Contexto = Contexto {
    aberta: None,
    camadas: 0,
};
const R: u16 = v::TINTA_PREENCH + v::OFF_SOLIDA;
const SOLO: Instante = Instante {
    t_cut: 0.0,
    solo: true,
};

fn f(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, 0, 0))
}

/// O preenchimento gravado inteiro (r, g, b, a) em 0 s e 1 s, a 24 fps — o que o autokey deixa.
fn doc() -> TimelineDoc {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for t in [0.0, 1.0] {
        for c in R..R + 4 {
            d.insert_key(
                E,
                f(c),
                RationalTime::from_seconds(t),
                AnimValue::Float(100.0),
                Interp::Linear,
            );
        }
    }
    d
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(100.0)
}

fn em(t: f64) -> Instante {
    Instante { t_cut: t, ..SOLO }
}

/// A cor do preenchimento e a opacidade dele mostram o grupo: cheio numa chave, vazio entre elas;
/// um campo sem curva (a largura) mostra o ponto clicável; sem objecto, nenhum campo tem canal.
#[test]
fn o_campo_mostra_o_estado_do_grupo_dele() {
    let d = doc();
    let m = marcas(&d, SEM, &[E], em(0.0), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_FILL_OPACITY], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_WIDTH], KeyMark::Livre);
    let m = marcas(&d, SEM, &[E], em(0.5), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Animado);
    assert!(marcas(&d, SEM, &[], em(0.0), na_curva).is_empty());
}

/// Mexer SÓ o verde a meio já põe o losango da cor em «mudado» — o grupo inteiro responde.
/// (Mutação: ler só o representante do grupo ⇒ o verde passa despercebido.)
#[test]
fn um_membro_mexido_muda_o_losango_do_grupo() {
    let d = doc();
    let verde_mexido = |_, p| Some(if p == f(R + 1) { 7.0 } else { 100.0 });
    let m = marcas(&d, SEM, &[E], em(0.5), verde_mexido);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Mudado);
}

/// Numa chave, o clique apaga as 4 chaves da cor neste quadro — e só essas — num passo de undo.
#[test]
fn clicar_numa_chave_apaga_o_grupo_neste_quadro() {
    let d = doc();
    let i = clique(
        &d,
        SEM,
        &[ferramenta::VECTOR_FILL_SWATCH],
        &[E],
        em(0.0),
        |_| Vec::new(),
        na_curva,
        |_, _| None,
    );
    let n = i
        .iter()
        .filter(|x| matches!(x, TimelineIntent::AddToSelection(_)))
        .count();
    assert_eq!(n, 4);
    assert_eq!(i.first(), Some(&TimelineIntent::BeginEdit));
    assert!(i.contains(&TimelineIntent::DeleteSelection));
    assert_eq!(i.last(), Some(&TimelineIntent::EndEdit));
}

/// Fora de uma chave, o clique grava o GRUPO inteiro que a família publicou, pela porta da K —
/// a largura grava só a largura; o tracejado grava o «há tracejado» e os pares.
#[test]
fn clicar_fora_de_uma_chave_grava_o_grupo_inteiro() {
    let d = TimelineDoc::new();
    let publicados = |_| {
        vec![
            FieldKey::new(VETOR, v::TRACO_LARGURA, 0, 0),
            FieldKey::new(VETOR, v::TRACO_TRACEJADO, 0, 0),
            FieldKey::new(VETOR, v::TRACO_TRACO, 0, 0),
            FieldKey::new(VETOR, v::TRACO_VAO, 0, 0),
            FieldKey::new(VETOR, R, 0, 0),
        ]
    };
    let k = |e, p| {
        Some(TimelineIntent::AddKey {
            entity: e,
            prop: p,
            t: RationalTime::from_seconds(0.5),
            value: AnimValue::Float(1.0),
            interp: Interp::Linear,
        })
    };
    let gravados = |campo| {
        clique(&d, SEM, &[campo], &[E], em(0.5), publicados, |_, _| None, k)
            .into_iter()
            .filter_map(|x| match x {
                TimelineIntent::AddKey { prop, .. } => Some(prop),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        gravados(ferramenta::VECTOR_WIDTH),
        vec![f(v::TRACO_LARGURA)]
    );
    assert_eq!(
        gravados(ferramenta::VECTOR_GAP_IDS[1]),
        vec![f(v::TRACO_TRACEJADO), f(v::TRACO_TRACO), f(v::TRACO_VAO)]
    );
    assert!(
        gravados(NodeId(1)).is_empty(),
        "um campo fora da tabela não grava nada"
    );
}

/// ⭐ O mesmo id da camada ABERTA mostra a entrada que está aberta: a 1.ª entrada (`i = 1`) e a
/// 2.ª (`i = 2`) são canais diferentes; fechada, o campo não tem canal. A swatch da entrada `k`
/// mostra a tinta dela (preenchimento OU traço), e o Contour tem canal próprio.
#[test]
fn o_campo_da_camada_aberta_mostra_a_entrada_aberta() {
    let op = |i| PropKind::Field(FieldKey::new(VETOR, v::PILHA_OPACIDADE, i, 0));
    let aberta = |k| Contexto {
        aberta: Some(k),
        camadas: 2,
    };
    assert_eq!(
        canal_do_campo(painel::VECTOR_PAINT_OPACITY, aberta(0)),
        vec![op(1)]
    );
    assert_eq!(
        canal_do_campo(painel::VECTOR_PAINT_OPACITY, aberta(1)),
        vec![op(2)]
    );
    assert!(canal_do_campo(painel::VECTOR_PAINT_OPACITY, SEM).is_empty());
    let sw = canal_do_campo(painel::vector_paint_swatch_id(1), aberta(0));
    assert_eq!(
        sw,
        vec![
            PropKind::Field(FieldKey::new(VETOR, v::TINTA_PREENCH + v::OFF_SOLIDA, 2, 0)),
            PropKind::Field(FieldKey::new(VETOR, v::TINTA_TRACO + v::OFF_SOLIDA, 2, 0)),
        ]
    );
    assert!(canal_do_campo(painel::vector_paint_swatch_id(2), aberta(0)).is_empty());
    assert_eq!(
        canal_do_campo(painel::VECTOR_CONTOUR_TO, SEM),
        vec![f(v::CONTORNO_COR)]
    );
}

/// A opacidade da 2.ª entrada com chaves: o losango acende só com ELA aberta — com a 1.ª aberta o
/// mesmo campo é outro canal. (Mutação: ignorar `aberta` e usar `i = 0` ⇒ nunca acende.)
#[test]
fn o_losango_da_camada_aberta_segue_a_entrada() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    let op2 = PropKind::Field(FieldKey::new(VETOR, v::PILHA_OPACIDADE, 2, 0));
    d.insert_key(
        E,
        op2,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(100.0),
        Interp::Linear,
    );
    let ctx = |k| Contexto {
        aberta: Some(k),
        camadas: 2,
    };
    let m = marcas(&d, ctx(1), &[E], em(0.0), na_curva);
    assert_eq!(m[&painel::VECTOR_PAINT_OPACITY], KeyMark::Chave);
    let m = marcas(&d, ctx(0), &[E], em(0.0), na_curva);
    assert_eq!(m[&painel::VECTOR_PAINT_OPACITY], KeyMark::Livre);
    assert!(!marcas(&d, SEM, &[E], em(0.0), na_curva).contains_key(&painel::VECTOR_PAINT_OPACITY));
}

fn fij(c: u16, i: u16, j: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, i, j))
}

/// ⭐ F24d — os campos que não tinham a coluna do losango mostram o canal deles: a mistura do
/// objecto e a da entrada aberta, o Fill Type e «tem traço» (membros da tinta), a ponta do fim, a
/// tinta da barra, o olho da entrada `k`.
#[test]
fn os_campos_sem_coluna_mostram_o_canal_deles() {
    let ctx = Contexto {
        aberta: Some(1),
        camadas: 2,
    };
    assert_eq!(
        canal_do_campo(painel::VECTOR_OBJ_BLEND, SEM),
        vec![f(v::MISTURA)]
    );
    assert_eq!(
        canal_do_campo(ferramenta::VECTOR_FILL_KIND_PATTERN, SEM),
        vec![f(R)]
    );
    let traco = f(v::TINTA_TRACO + v::OFF_SOLIDA);
    assert_eq!(
        canal_do_campo(painel::VECTOR_STROKE_PRESENT, SEM),
        vec![traco]
    );
    assert_eq!(canal_do_campo(painel::VECTOR_BAR_KIND, SEM), vec![traco]);
    assert_eq!(
        canal_do_campo(ferramenta::VECTOR_MARKER_END_DD, SEM),
        vec![f(v::TRACO_MARCA_FIM)]
    );
    assert_eq!(
        canal_do_campo(painel::VECTOR_PAINT_BLEND, ctx),
        vec![fij(v::PILHA_MISTURA, 2, 0)]
    );
    assert_eq!(
        canal_do_campo(painel::vector_paint_eye_id(1), ctx),
        vec![fij(v::PILHA_LIGADA, 2, 0)]
    );
    assert!(canal_do_campo(painel::vector_paint_eye_id(2), ctx).is_empty());
}

/// ⭐ F24d — os ids DERIVADOS de um índice voltam ao canal: o padrão da tinta `slot` (só os
/// controlos que são membros dela), o parâmetro, o número e a caixinha do efeito `r`, o olho dele,
/// o campo `j` da forma e o gémeo que cicla. (Mutação: trocar `slot` por 0 ⇒ o traço lê o
/// preenchimento.)
#[test]
fn os_ids_derivados_voltam_ao_canal_deles() {
    use painel::TexPatKnob as K;
    let c = |id| canal_do_campo(id, SEM);
    assert_eq!(c(painel::texpat_id(0, K::Width)), vec![f(R)]);
    assert_eq!(
        c(painel::texpat_id(1, K::AngleNum)),
        vec![f(v::TINTA_TRACO + v::OFF_SOLIDA)]
    );
    assert_eq!(
        c(painel::texpat_id(1, K::Tile(2))),
        c(painel::texpat_id(1, K::Mode(0)))
    );
    for k in [K::Lock, K::GapLink, K::Source, K::PickShape] {
        assert!(c(painel::texpat_id(0, k)).is_empty(), "{k:?} não é campo");
    }
    // O desfasamento (bloco 2): membro da tinta, como o tamanho.
    assert_eq!(c(painel::texpat_id(0, K::OffsetNum)), vec![f(R)]);
    let p = fij(v::EFEITO_PARAM, 2, 3);
    assert_eq!(c(ferramenta::vector_fx_param_id(2, 3)), vec![p]);
    assert_eq!(c(ferramenta::vector_fx_param_num_id(2, 3)), vec![p]);
    assert_eq!(
        c(ferramenta::vector_fx_toggle_id(1, 0)),
        vec![fij(v::EFEITO_PARAM, 1, 0)]
    );
    assert_eq!(
        c(ferramenta::vector_fx_hide_id(3)),
        vec![fij(v::EFEITO_LIGADO, 3, 0)]
    );
    let campo = fij(v::PARAM_VALOR, 0, 4);
    assert_eq!(c(ferramenta::vector_shape_field_id(4)), vec![campo]);
    assert_eq!(c(ferramenta::vector_shape_choice_id(4)), vec![campo]);
}

/// Uma chave num parâmetro do efeito 1 acende o losango de todo parâmetro DELE (o efeito é um
/// grupo) e não o do efeito 0; o olho do efeito 1 é outro canal.
#[test]
fn a_chave_de_um_efeito_acende_os_parametros_dele() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    d.insert_key(
        E,
        fij(v::EFEITO_PARAM, 1, 2),
        RationalTime::from_seconds(0.0),
        AnimValue::Float(100.0),
        Interp::Linear,
    );
    let m = marcas(&d, SEM, &[E], em(0.0), na_curva);
    assert_eq!(m[&ferramenta::vector_fx_param_id(1, 0)], KeyMark::Chave);
    assert_eq!(m[&ferramenta::vector_fx_toggle_id(1, 5)], KeyMark::Chave);
    assert_eq!(m[&ferramenta::vector_fx_param_id(0, 2)], KeyMark::Livre);
    assert_eq!(m[&ferramenta::vector_fx_hide_id(1)], KeyMark::Livre);
}

/// ⭐ Bloco 2 — cada campo novo do painel volta ao GRUPO dele: o pincel à tinta do traço, a quebra
/// (o modo e a largura) ao mesmo grupo, a família, o perfil (sliders, chip da barra e catálogo),
/// o Blend, o Morph (o `PropKind` de sempre), o Envelope (um grupo para a gaiola inteira), as
/// âncoras, o recorte, o auto layout, a pele e o ícone do widget, o padrão e o texto no caminho e
/// os filtros (o olho um grupo, o resto do degrau outro, por degrau).
#[test]
fn os_campos_do_bloco_2_voltam_ao_grupo_deles() {
    let c = |id| canal_do_campo(id, SEM);
    let grupo = |id| {
        let g = c(id);
        assert_eq!(g.len(), 1, "{id:?}: {g:?}");
        match g[0] {
            PropKind::Field(k) => k.grupo().expect("campo declarado"),
            p => panic!("{id:?} → {p:?}"),
        }
    };
    let traco = grupo(ferramenta::VECTOR_STROKE_SWATCH);
    for id in [
        painel::VECTOR_BRUSH_SCALE_NUM,
        painel::VECTOR_BRUSH_ROTATION,
        painel::VECTOR_BRUSH_FLIP,
    ] {
        assert_eq!(grupo(id), traco, "o pincel é membro da tinta do traço");
    }
    assert_eq!(
        grupo(painel::VECTOR_TEXT_WRAP_FIXED),
        grupo(painel::VECTOR_TEXT_WRAP_W),
        "o modo da quebra e a largura gravam-se juntos"
    );
    assert_eq!(c(painel::VECTOR_TEXT_FONT_DD), vec![f(v::TEXTO_FAMILIA)]);
    let perfil = grupo(painel::VECTOR_BAR_PROFILE);
    for id in [
        painel::VECTOR_EXPAND_W_MID,
        ferramenta::VECTOR_EXPAND_W_POS_NUM,
        painel::vector_width_preset_id(3),
    ] {
        assert_eq!(grupo(id), perfil);
    }
    assert_eq!(
        c(ferramenta::VECTOR_BLEND_STEPS_NUM),
        vec![f(v::BLEND_PASSOS)]
    );
    assert_eq!(c(painel::VECTOR_MORPH_T), vec![PropKind::Morph]);
    let env = grupo(painel::VECTOR_ENVELOPE_MESH);
    assert_eq!(grupo(ferramenta::VECTOR_ENVELOPE_BEND_NUM), env);
    assert_eq!(
        grupo(painel::VECTOR_ANCHOR_V_STRETCH),
        grupo(painel::VECTOR_ANCHOR_H_START)
    );
    assert_eq!(c(painel::VECTOR_FRAME_CLIP_ON), vec![f(v::MOLDURA_RECORTA)]);
    assert_eq!(c(painel::VECTOR_LAYOUT_DIR_OFF), vec![f(v::LAYOUT_DIR)]);
    assert_eq!(
        grupo(painel::VECTOR_LAYOUT_PAD_L),
        grupo(painel::VECTOR_LAYOUT_PAD_ALL)
    );
    assert_eq!(
        c(painel::VECTOR_LAYOUT_ITEM_ABSOLUTE),
        vec![f(v::LAYOUT_ABSOLUTO)]
    );
    assert_eq!(c(painel::vector_widget_kind_id(5)), vec![f(v::WIDGET_PELE)]);
    assert_eq!(c(painel::VECTOR_WIDGET_ICON_DD), vec![f(v::WIDGET_ICONE)]);
    assert_eq!(
        c(painel::VECTOR_PATTERNPATH_SLIDE_NUM),
        vec![f(v::PADRAO_CAMINHO_INICIO), f(v::PADRAO_CAMINHO_FIM)],
        "o Slide move o início e o fim"
    );
    assert_eq!(
        c(painel::VECTOR_TEXTPATH_FLIP_OFF),
        vec![f(v::TEXTO_CAMINHO_ESPELHO)]
    );
    assert_eq!(
        c(painel::filter_hide_id(2)),
        vec![fij(v::FILTRO_LIGADO, 2, 0)]
    );
    let degrau = c(painel::filter_radius_id(2));
    assert_eq!(degrau, vec![fij(v::FILTRO_TIPO, 2, 0)]);
    for id in [
        painel::filter_mode_id(2, 1),
        painel::filter_color_b_id(2),
        painel::filter_stop_color_id(2),
        painel::filter_blend_id(2),
        painel::filter_hue_num_id(2),
    ] {
        assert_eq!(c(id), degrau, "{id:?}");
    }
    assert_ne!(c(painel::filter_radius_id(1)), degrau, "por degrau");
    // Fora: os verbos e o modo de vista da margem.
    for id in [
        painel::VECTOR_BRUSH_PICK_SHAPE,
        painel::VECTOR_FRAME_PANEL_ON,
        painel::VECTOR_LAYOUT_PAD_EACH_MODE,
        painel::filter_remove_id(0),
    ] {
        assert!(c(id).is_empty(), "{id:?} não é campo");
    }
}
