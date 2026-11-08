//! O cartão Shape em linhas de cartão (fatia H5, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos, as
//! condições de `paint_shape_section`. Os gizmos (curva, Taper, Flatten) pintam pela `Tela`.

use crate::cards_linhas::{
    Item, aberta, botoes, fila, filete, grupo, grupo_sobre_controlo, numero, par_xy,
    rotulo_sobre_controlo,
};
use crate::cards_tela::Tela;
use crate::paint_texture::Previa;
use crate::paint_wetpaint::Numero;
use crate::{cards_penteado, esmaecer, number_field as nf, paint_brush, ponta_linhas, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BrushSettings, CartaoDaPonta, Falloff, LINHAS_DA_PONTA, LinhaDaFerramenta,
    MAX_FALL_OFF_DIAMETERS, MAX_TAPER_DIAMETERS, PaintMedia, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX,
    TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
};

const UNIDADE: (f32, f32) = (0.0, 1.0);

/// A resposta do `plano_do_corpo`: o Inpaint marca um disco, e não lê a silhueta.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.shape.shape")
}

/// A ferramenta lê a silhueta. Sem isso, o cartão é só o Taper, o Flatten e o Follow.
fn le_a_forma(b: &BrushSettings) -> bool {
    b.le(LinhaDaFerramenta::Forma)
}

/// «Automatic» ligado: a aguada usa a silhueta de fábrica, e a caixa fecha tudo o resto.
fn automatica(b: &BrushSettings) -> bool {
    tem_automatica(b) && b.watercolor_shape_auto
}

fn tem_automatica(b: &BrushSettings) -> bool {
    le_a_forma(b) && b.watercolor && b.le(LinhaDaFerramenta::Aguada)
}

fn imagem(b: &BrushSettings) -> bool {
    TextureKind::from_u8(b.shape_kind) == TextureKind::Image
}

/// Há linhas por camada: a mesma pergunta do `paint_shape_per_layer_color`.
fn linhas_por_camada(b: &BrushSettings) -> bool {
    b.shape_layer_count > 1 && b.le(LinhaDaFerramenta::CorPorCamada) && b.shape_per_layer_color
}

/// O que se desenha uma vez por quadro: a prévia da forma.
pub(crate) type Pecas = Option<Previa>;

/// A silhueta, se a forma está à vista.
pub(crate) fn pecas(w: f32, brush: &BrushSettings) -> Pecas {
    (le_a_forma(brush) && !automatica(brush))
        .then(|| crate::paint_shape::previa_da_forma(w, *brush))
}

fn dureza(b: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.shape.hardness",
        pids::PAINTER_BRUSH_HARDNESS,
        b.hardness,
        UNIDADE,
        nf::FINE_STEP,
        2,
    )
}

/// As linhas do Taper. Condições de `l`, valores de `v`: a fábrica lê-se pela mesma tabela.
fn do_taper(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let t = v.taper;
    let f = nf::FINE_STEP;
    let com_cauda = PaintMedia::from_u8(l.media) == PaintMedia::Digital
        && l.le(LinhaDaFerramenta::TracoComoCamada);
    let mut n = vec![
        numero(
            "panel.painter_layers.taper.tip",
            pids::PAINTER_TAPER_TIP_START,
            t.tip_start.clamp(0.0, 1.0),
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.taper.opacity",
            pids::PAINTER_TAPER_OPACITY,
            t.opacity.clamp(0.0, 1.0),
            UNIDADE,
            f,
            2,
        ),
    ];
    let dentro = |v: f32, hi: f32| ph2d_editor_core::math::safe_clamp(v, 0.0, hi);
    if com_cauda {
        n.push(numero(
            "panel.painter_layers.taper.end",
            pids::PAINTER_TAPER_END,
            dentro(t.end, MAX_TAPER_DIAMETERS),
            (0.0, MAX_TAPER_DIAMETERS),
            f,
            2,
        ));
        n.push(numero(
            "panel.painter_layers.taper.tip_end",
            pids::PAINTER_TAPER_TIP_END,
            dentro(t.tip_end, 1.0),
            UNIDADE,
            f,
            2,
        ));
    }
    n.push(numero(
        "panel.painter_layers.taper.fall_off",
        pids::PAINTER_TAPER_FALL_OFF,
        dentro(t.fall_off, MAX_FALL_OFF_DIAMETERS),
        (0.0, MAX_FALL_OFF_DIAMETERS),
        f,
        2,
    ));
    n.push(numero(
        "panel.painter_layers.taper.fall_off_start",
        pids::PAINTER_TAPER_FALL_OFF_START,
        dentro(t.fall_off_start, MAX_FALL_OFF_DIAMETERS),
        (0.0, MAX_FALL_OFF_DIAMETERS),
        f,
        2,
    ));
    n
}

fn do_follow(l: &BrushSettings) -> Option<Item> {
    l.le(LinhaDaFerramenta::Orientacao).then(|| {
        let texto = pids::PAINTER_SHAPE_FOLLOW_MODES
            .iter()
            .find(|(v, _)| *v == l.shape_follow)
            .map_or(tr("panel.painter_layers.shape.off"), |(_, n)| n);
        Item::Lista {
            id: pids::PAINTER_SHAPE_FOLLOW,
            chave: "panel.painter_layers.shape.follow",
            texto,
        }
    })
}

/// As linhas da ponta do cartão `cartao` que estão à vista.
pub(crate) fn da_ponta(cartao: CartaoDaPonta, l: &BrushSettings, v: &BrushSettings) -> Vec<Item> {
    ponta_linhas::linhas_do_cartao(cartao)
        .filter(|&i| ponta_linhas::visivel(i, l))
        .map(|i| {
            let linha = LINHAS_DA_PONTA[i];
            let id = pids::PAINTER_PONTA_FIELDS[i];
            let chave = ponta_linhas::ROTULOS_DA_PONTA[i];
            let opcoes = linha.opcoes();
            if opcoes.is_empty() {
                let (lo, hi, passo) = linha.faixa();
                Item::Num(numero(
                    chave,
                    id,
                    v.ponta[i] as f32,
                    (lo as f32, hi as f32),
                    passo,
                    linha.casas(),
                ))
            } else {
                Item::Lista {
                    id,
                    chave,
                    texto: tr(opcoes[usize::from(ponta_linhas::opcao_atual(i, l))]),
                }
            }
        })
        .collect()
}

/// As pontas do Wet Paint: o Ends fade antes da caixa, para a caixa dividir a linha com o Ends beyond dela.
fn do_molhado(l: &BrushSettings, v: &BrushSettings) -> Vec<Item> {
    if !(l.wetpaint && l.le(LinhaDaFerramenta::DepositoMolhado)) {
        return Vec::new();
    }
    let f = nf::FINE_STEP;
    let mut n = vec![Item::Num(numero(
        "panel.painter_layers.shape.square_tip",
        pids::PAINTER_BRUSH_DAB_QUINA,
        v.dab_quina,
        UNIDADE,
        f,
        2,
    ))];
    if l.dab_follows_stroke {
        n.push(Item::Num(numero(
            "panel.painter_layers.shape.rotation_smoothing",
            pids::PAINTER_BRUSH_DAB_ROTATION_LAG,
            v.dab_rotation_lag,
            (0.0, pids::PAINTER_BRUSH_DAB_ROTATION_LAG_MAX),
            f,
            2,
        )));
    }
    n.push(Item::Num(numero(
        "panel.painter_layers.shape.ends_fade",
        pids::PAINTER_BRUSH_PONTAS_ESBATE,
        v.pontas_esbate,
        (
            pids::PAINTER_BRUSH_PONTAS_ESBATE_MIN,
            pids::PAINTER_BRUSH_PONTAS_ESBATE_MAX,
        ),
        f,
        2,
    )));
    n.push(Item::Caixa {
        id: pids::PAINTER_BRUSH_PONTAS_RECTAS,
        chave: "panel.painter_layers.shape.flat_ends",
        on: l.pontas_rectas,
    });
    if l.pontas_rectas {
        n.push(Item::Num(numero(
            "panel.painter_layers.shape.flat_ends_beyond",
            pids::PAINTER_BRUSH_PONTAS_ALEM,
            v.pontas_alem,
            (0.0, pids::PAINTER_BRUSH_PONTAS_ALEM_MAX),
            f,
            2,
        )));
    }
    n
}

/// O que divide linhas debaixo do gizmo: Follow, a ponta de píxel, as pontas do molhado.
fn da_pegada(l: &BrushSettings, v: &BrushSettings) -> Vec<Item> {
    let mut itens: Vec<Item> = do_follow(l).into_iter().collect();
    if le_a_forma(l) {
        if PaintMedia::from_u8(l.media) == PaintMedia::Digital {
            itens.extend(da_ponta(CartaoDaPonta::Pincel, l, v));
        }
        itens.extend(do_molhado(l, v));
    }
    itens
}

/// As caixas da textura: de onde vem a transparência, e a cor por camada.
fn da_cor_da_textura(l: &BrushSettings) -> Vec<Item> {
    let mut itens = Vec::new();
    if l.shape_layer_count == 0 {
        return itens;
    }
    if l.shape_has_alpha_choice {
        itens.push(Item::Caixa {
            id: pids::PAINTER_SHAPE_ALPHA_FROM_IMAGE,
            chave: "panel.painter_layers.shape.alpha_from_image",
            on: l.shape_alpha_from_image,
        });
    }
    if l.le(LinhaDaFerramenta::CorPorCamada) {
        itens.push(Item::Caixa {
            id: pids::PAINTER_SHAPE_PER_LAYER_COLOR,
            chave: if l.shape_layer_count == 1 {
                "panel.painter_layers.shape.use_texture_colors"
            } else {
                "panel.painter_layers.shape.per_layer_color"
            },
            on: l.shape_per_layer_color,
        });
    }
    itens
}

fn com_textura(b: &BrushSettings) -> bool {
    TextureKind::from_u8(b.shape_kind) != TextureKind::None
}

const OFFSET: [NodeId; 2] = [pids::PAINTER_SHAPE_OFFSET_X, pids::PAINTER_SHAPE_OFFSET_Y];
const TAMANHO: [NodeId; 2] = [pids::PAINTER_SHAPE_SIZE_X, pids::PAINTER_SHAPE_SIZE_Y];

/// Os dois pares da moldura da textura.
fn dos_eixos(v: &BrushSettings) -> [Numero; 4] {
    let par = |id, valor, faixa, passo| numero("", id, valor, faixa, passo, 2);
    let (o, t) = (
        (TEX_OFFSET_MIN, TEX_OFFSET_MAX),
        (TEX_SIZE_MIN, TEX_SIZE_MAX),
    );
    [
        par(OFFSET[0], v.shape_offset[0], o, nf::FINE_STEP),
        par(OFFSET[1], v.shape_offset[1], o, nf::FINE_STEP),
        par(TAMANHO[0], v.shape_size[0], t, nf::SIZE_STEP),
        par(TAMANHO[1], v.shape_size[1], t, nf::SIZE_STEP),
    ]
}

/// O Angle, os parâmetros do padrão e o depósito: curtos, dividem as linhas.
fn da_moldura(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let mut n = Vec::new();
    let kind = TextureKind::from_u8(l.shape_kind);
    if com_textura(l) {
        n.push(numero(
            "card.painter.shape.angle",
            pids::PAINTER_SHAPE_ANGLE,
            f32::from(v.shape_angle_deg),
            (0.0, f32::from(TEX_ANGLE_MAX_DEG)),
            nf::ANGLE_STEP,
            0,
        ));
        if !imagem(l) {
            n.extend(crate::cards_linhas::do_padrao(
                kind,
                &pids::PAINTER_SHAPE_PARAMS,
                &v.shape_params,
            ));
        }
    }
    if l.impasto_applies {
        n.push(numero(
            "panel.painter_layers.shape.relief",
            pids::PAINTER_SHAPE_RELIEF,
            v.shape_relief,
            UNIDADE,
            nf::FINE_STEP,
            2,
        ));
        if l.shape_relief > 0.0 {
            n.push(numero(
                "panel.painter_layers.shape.shine",
                pids::PAINTER_SHAPE_SHINE,
                v.impasto_shine,
                UNIDADE,
                nf::FINE_STEP,
                2,
            ));
        }
    }
    n
}

/// Todos os números à vista neste estado. Só serve quem os arma: a ordem é a da tabela, não a da pintura.
fn todos(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    let so_numeros = |itens: Vec<Item>| {
        itens.into_iter().filter_map(|i| match i {
            Item::Num(n) => Some(n),
            _ => None,
        })
    };
    if automatica(l) {
        return Vec::new();
    }
    let mut n = do_taper(l, v);
    n.extend(so_numeros(da_pegada(l, v)));
    if !le_a_forma(l) {
        return n;
    }
    if !imagem(l) {
        n.push(dureza(v));
    }
    n.extend(so_numeros(crate::paint_shape_pontas::do_dab_fixo(l, v)));
    if imagem(l) && l.shape_has_image {
        n.extend(so_numeros(da_ponta(CartaoDaPonta::Forma, l, v)));
    }
    n.extend(so_numeros(da_ponta(CartaoDaPonta::Dual, l, v)));
    if com_textura(l) {
        n.extend(dos_eixos(v));
    }
    n.extend(da_moldura(l, v));
    n
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let store = ctx.host.store_mut();
    let numeros = todos(b, b);
    crate::cards_wet::arma_numeros(store, &numeros, &todos(b, &paint_brush::FALLBACK_BRUSH));
    if le_a_forma(b) {
        crate::cards_wet::arma_numeros(
            store,
            &cards_penteado::numeros(b),
            &cards_penteado::numeros(&paint_brush::FALLBACK_BRUSH),
        );
    }
    for id in [
        pids::PAINTER_SHAPE_WATERCOLOR_AUTO,
        pids::PAINTER_BRUSH_FALLOFF,
        pids::PAINTER_SHAPE_KIND,
        pids::PAINTER_SHAPE_FOLLOW,
        pids::PAINTER_BRUSH_PONTAS_RECTAS,
        pids::PAINTER_BRUSH_DAB_FIXO,
        pids::PAINTER_SHAPE_ALPHA_FROM_IMAGE,
        pids::PAINTER_SHAPE_PER_LAYER_COLOR,
    ] {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
    for i in (0..LINHAS_DA_PONTA.len()).filter(|&i| ponta_linhas::e_menu(i)) {
        let _ = esmaecer::poe_dica(store, &[pids::PAINTER_PONTA_FIELDS[i]]);
    }
}

/// Onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let aberto = |ctx: &mut PaintCtx, id| {
        if aberta(ctx.host.store(), id) {
            ctx.host.hit_index_mut().rect_for(id)
        } else {
            None
        }
    };
    if let Some(r) = aberto(ctx, pids::PAINTER_BRUSH_FALLOFF) {
        state::set_pending_brush_falloff_dd(Some((r, b.falloff)));
    }
    if let Some(r) = aberto(ctx, pids::PAINTER_SHAPE_KIND) {
        state::set_pending_brush_shape_kind_dd(Some((r, b.shape_kind)));
    }
    if let Some(r) = aberto(ctx, pids::PAINTER_SHAPE_FOLLOW) {
        state::set_pending_brush_shape_follow_dd(Some((r, b.shape_follow)));
    }
    for i in (0..LINHAS_DA_PONTA.len()).filter(|&i| ponta_linhas::e_menu(i)) {
        if let Some(r) = aberto(ctx, pids::PAINTER_PONTA_FIELDS[i]) {
            ponta_linhas::abre_lista(r, i, ponta_linhas::opcao_atual(i, b));
        }
    }
}

fn numeros_em_fila(c: &mut CardCtx<'_>, lista: Vec<Numero>, y: f32) -> f32 {
    let itens: Vec<Item> = lista.into_iter().map(Item::Num).collect();
    fila(c, &itens, y)
}

fn queda(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let lista = Item::Lista {
        id: pids::PAINTER_BRUSH_FALLOFF,
        chave: "panel.painter_layers.shape.falloff",
        texto: tr(Falloff::from_u8(b.falloff).name_key()),
    };
    let y = fila(c, &[lista, Item::Num(dureza(b))], y);
    if !le_a_forma(b) || automatica(b) || imagem(b) {
        return y;
    }
    let (x, w) = (c.x, c.w);
    crate::paint_falloff::curva(&mut Tela::do_cartao(c), x, w, y + m::FIELD_ROW_GAP, *b)
}

fn taper_e_pegada(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let viva = !automatica(b);
    let (x, w) = (c.x, c.w);
    let y = if viva {
        use ph2d_editor_core::zones::Rect;
        let topo = y + m::FIELD_ROW_GAP;
        ph2d_editor_core::panel::cards::censo::controlo_maior_que_a_pega(
            pids::PAINTER_TAPER_TIP_START,
            Rect::new(x, topo, w, crate::paint_taper::CANVAS_H),
        );
        crate::paint_taper::visor(&mut Tela::do_cartao(c), x, w, topo, *b)
    } else {
        y
    };
    let y = numeros_em_fila(c, do_taper(b, b), y);
    let y = grupo(c, "panel.painter_layers.shape.flatten_and_rotate", y);
    let y = if viva {
        crate::paint_shape_dab::gizmo(&mut Tela::do_cartao(c), x, w, y + m::FIELD_ROW_GAP, *b)
    } else {
        y
    };
    fila(c, &da_pegada(b, b), y)
}

/// A textura, de cima para baixo: a fonte, de onde vem a cor, as imagens, a prévia, a moldura.
fn textura(c: &mut CardCtx<'_>, b: &BrushSettings, p: &Pecas, y: f32) -> f32 {
    let kind = TextureKind::from_u8(b.shape_kind);
    let fonte = Item::Lista {
        id: pids::PAINTER_SHAPE_KIND,
        chave: "panel.painter_layers.shape.texture",
        texto: tr(kind.name_key()),
    };
    let com_imagens = imagem(b) && b.shape_has_image;
    let entre = b.document_layer_count > 1
        || !da_cor_da_textura(b).is_empty()
        || (le_a_forma(b) && !automatica(b) && linhas_por_camada(b))
        || com_imagens;
    // Sem nada entre elas, a fonte e o Dual brush dividem a linha.
    if !entre {
        let mut itens = vec![fonte];
        itens.extend(da_ponta(CartaoDaPonta::Dual, b, b));
        let y = fila(c, &itens, y);
        return fim_da_textura(c, b, p, y);
    }
    let mut y = fila(c, &[fonte], y);
    if b.document_layer_count > 1 {
        y = botoes(
            c,
            &[(
                pids::PAINTER_SHAPE_USE_LAYERS,
                tr("panel.painter_layers.shape.use_document_layers"),
            )],
            y,
        );
    }
    y = fila(c, &da_cor_da_textura(b), y);
    if le_a_forma(b) && !automatica(b) && linhas_por_camada(b) {
        y = crate::cards_rampa::linhas_por_camada(c, b, y);
    }
    if com_imagens {
        let mut verbos = vec![(
            pids::PAINTER_SHAPE_HOSE_ADD,
            tr("panel.painter_ponta.hose_add"),
        )];
        if b.shape_hose_count > 1 {
            verbos.push((
                pids::PAINTER_SHAPE_HOSE_CLEAR,
                tr("panel.painter_ponta.hose_clear"),
            ));
        }
        y = botoes(c, &verbos, y);
        y = fila(c, &da_ponta(CartaoDaPonta::Forma, b, b), y);
    }
    y = fila(c, &da_ponta(CartaoDaPonta::Dual, b, b), y);
    fim_da_textura(c, b, p, y)
}

/// A imagem da 2.ª ponta, a prévia e a moldura.
fn fim_da_textura(c: &mut CardCtx<'_>, b: &BrushSettings, p: &Pecas, y: f32) -> f32 {
    let mut y = y;
    let duas = ponta_linhas::indice(ph2d_tool_painter::LinhaDaPonta::DualMode);
    let forma = ponta_linhas::indice(ph2d_tool_painter::LinhaDaPonta::DualTip);
    if b.ponta[duas] > 0.0 && b.ponta[forma] as u8 == TextureKind::Image.to_u8() {
        y = botoes(
            c,
            &[(
                pids::PAINTER_DUAL_IMAGE,
                tr("panel.painter_ponta.dual_image"),
            )],
            y,
        );
    }
    y = crate::cards_rampa::previa(c, p.as_ref(), y);
    if com_textura(b) {
        y = par_xy(c, "panel.painter_layers.shape.offset", OFFSET, y);
        y = par_xy(c, "panel.painter_layers.shape.size", TAMANHO, y);
    }
    numeros_em_fila(c, da_moldura(b, b), y)
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, p: &Pecas, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let mut y = y;
    if tem_automatica(&b) {
        y = fila(
            c,
            &[Item::Caixa {
                id: pids::PAINTER_SHAPE_WATERCOLOR_AUTO,
                chave: "panel.painter_layers.shape.automatic",
                on: b.watercolor_shape_auto,
            }],
            y,
        );
        if automatica(&b) {
            return cards_penteado::grupo_do_penteado(c, &b, y);
        }
    }
    if !le_a_forma(&b) {
        let y = rotulo_sobre_controlo(c, tr("panel.painter_layers.taper.taper"), y - m::LABEL_TOP);
        return taper_e_pegada(c, &b, y);
    }
    if !imagem(&b) {
        y = queda(c, &b, y);
    }
    y = fila(c, &crate::paint_shape_pontas::do_dab_fixo(&b, &b), y);
    y = grupo_sobre_controlo(c, "panel.painter_layers.taper.taper", y);
    y = taper_e_pegada(c, &b, y);
    y = filete(c, y);
    let y = textura(c, &b, p, y);
    cards_penteado::grupo_do_penteado(c, &b, y)
}
