//! O cartão Wet Paint em linhas de cartão (fatia H3, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos,
//! mesmas condições (as tabelas são as de `paint_wetpaint`, que o painel de antes também lê). O visor do Tilt
//! é o de sempre, pela `Tela`: a lei E10 manda não o redesenhar.

use crate::cards_linhas::{
    Item, aberta, botoes, com_lei, em_pares, fila, grupo, marca, registra_lista,
};
use crate::cards_tela::Tela;
use crate::paint_wetpaint::{self, Numero, oleo, tipo_de_tinta};
use crate::{
    esmaecer, number_field, paint_brush, paint_wetpaint_presets, paint_wetpaint_tilt, state,
};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo as linhas, ferramenta};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, LinhaDaFerramenta};

/// O nome de cada grupo da física, pela ordem de [`paint_wetpaint::numeros_da_fisica`].
const GRUPOS_DA_FISICA: [&str; 3] = [
    "card.painter.wet.contact",
    "card.painter.wet.thickness",
    "card.painter.wet.pigment",
];

/// Os ids de caixa e botão deste cartão: as dicas das leis esmaecidas são pedidas todas num sítio.
const CAIXAS: [NodeId; 7] = [
    pids::PAINTER_WETPAINT_TILT_TOGGLE,
    pids::PAINTER_WETPAINT_SHOWWET,
    pids::PAINTER_WETPAINT_PAPER_VISUAL,
    pids::PAINTER_WETPAINT_OLEO_REAL,
    pids::PAINTER_WETPAINT_PIGMENT_MIXING,
    pids::PAINTER_WETPAINT_LIVE_WATER,
    pids::PAINTER_WETPAINT_TUNING,
];

/// O nome curto, porque o grupo já diz «Whole canvas» (05 §4, lei 7); a dica diz o resto.
const ACCOES: [(NodeId, &str, &str); 3] = [
    (
        pids::PAINTER_WETPAINT_WETCANVAS,
        "panel.painter_layers.watercolor.wet",
        "card.painter.wet.wet_tip",
    ),
    (
        pids::PAINTER_WETPAINT_DRYCANVAS,
        "panel.painter_layers.watercolor.dry",
        "card.painter.wet.dry_tip",
    ),
    (
        pids::PAINTER_WETPAINT_FASTDRY,
        "panel.painter_layers.wetpaint.fast_dry",
        "card.painter.wet.fast_dry_tip",
    ),
];

/// O cartão existe? Um meio, uma resposta: a do `plano_do_corpo`.
pub(crate) fn presente(brush: &BrushSettings) -> bool {
    brush.wetpaint
}

/// O título é o do painel de antes.
pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.wetpaint.wet_paint")
}

/// Todos os números deste cartão neste estado, com as leis de `leis` e os valores de `valores`.
fn todos_os_numeros(leis: &BrushSettings, valores: &BrushSettings) -> Vec<Numero> {
    let mut n: Vec<Numero> = paint_wetpaint::numeros_da_grade(valores).into();
    if !leis.eraser {
        n.extend(paint_wetpaint::numeros_da_ferramenta(leis, valores));
    }
    n.push(paint_wetpaint_tilt::numero_da_borda(valores));
    n.extend(
        paint_wetpaint::numeros_da_fisica(leis, valores)
            .into_iter()
            .flatten(),
    );
    let (optica, material) = oleo::numeros(leis, valores);
    n.extend(optica);
    n.extend(material);
    n
}

/// O valor, a faixa, o passo, a fábrica e a dica da lei de cada número.
pub(crate) fn arma_numeros(store: &mut WidgetStore, numeros: &[Numero], fabrica: &[Numero]) {
    for (n, f) in numeros.iter().zip(fabrica) {
        store.mark_card_field(
            n.id,
            CardFieldSpec {
                step: n.passo,
                scrub: n.passo,
                default: Some(f64::from(f.valor)),
            },
        );
        number_field::arm_field(store, n.id, n.valor, n.min, n.max, n.passo, n.casas);
        let _ = esmaecer::poe_dica(store, &[n.id]);
    }
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let numeros = todos_os_numeros(b, b);
    arma_numeros(
        store,
        &numeros,
        &todos_os_numeros(b, &paint_brush::FALLBACK_BRUSH),
    );
    for id in CAIXAS {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
    for (id, _, dica) in ACCOES {
        if esmaecer::poe_dica(store, &[id]).is_none() {
            store.set_tooltip(id, tr(dica));
        }
    }
}

/// Depois da lista: onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let tipo = pids::PAINTER_WETPAINT_MEDIUM;
    if aberta(ctx.host.store(), tipo)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(tipo)
    {
        tipo_de_tinta::abre_lista(r, u8::from(b.wet_knobs.e_oleo()));
    }
    let ambiente = pids::PAINTER_WETPAINT_OLEO_AMBIENTE;
    if aberta(ctx.host.store(), ambiente)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(ambiente)
    {
        oleo::abre_lista(r, b.wet_oleo_ambiente);
    }
}

/// O campo de um número: rótulo em cima, a letra da grandeza na pega, a marca de animação de antes.
pub(crate) fn campo(c: &mut CardCtx<'_>, n: &Numero, y: f32) -> f32 {
    com_lei(c, n.id, y, |c, y| {
        let rotulo = tr(n.chave);
        let ids = [n.id];
        let cell = Cell::new(Widget::Field {
            id: n.id,
            glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
        })
        .label(rotulo)
        .key(&ids);
        c.row(&[cell], y)
    })
}

pub(crate) fn numeros(c: &mut CardCtx<'_>, lista: &[Numero], y: f32) -> f32 {
    em_pares(c, lista, y, campo)
}

pub(crate) fn caixa(
    c: &mut CardCtx<'_>,
    (id, chave, on): &(NodeId, &'static str, bool),
    y: f32,
) -> f32 {
    com_lei(c, *id, y, |c, y| ferramenta::caixa(c, *id, chave, *on, y))
}

/// Um campo e uma caixa de marcar irmã na mesma linha (05 §4): a fundação desce a caixa se o nome dela não cabe.
pub(crate) fn campo_e_caixa(
    c: &mut CardCtx<'_>,
    n: &Numero,
    &(id, chave, on): &(NodeId, &'static str, bool),
    y: f32,
) -> f32 {
    let rotulo = tr(n.chave);
    let campo = Cell::new(Widget::Field {
        id: n.id,
        glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
    })
    .label(rotulo)
    .key(std::slice::from_ref(&n.id));
    c.pair(campo, marca(id, chave, on), y)
}

/// Uma lista suspensa com a marca de animação de antes: Paint type e Environment.
pub(crate) fn lista(c: &mut CardCtx<'_>, chave: &str, id: NodeId, texto: &str, y: f32) -> f32 {
    com_lei(c, id, y, |c, y| {
        let ids = [id];
        let cell = Cell::new(Widget::Dropdown {
            id,
            text: texto,
            options: &[],
        })
        .label(tr(chave))
        .key(&ids);
        c.row(&[cell], y)
    })
}

fn accoes_na_tela(c: &mut CardCtx<'_>, y: f32) -> f32 {
    botoes(c, &ACCOES.map(|(id, chave, _)| (id, tr(chave))), y)
}

/// Os números do Real Oil, o Environment e o material, debaixo de todas as caixas: ligar a caixa não troca as outras
/// de lugar. Óleo na tela mantém os números.
fn oleo_real(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let (optica, material) = oleo::numeros(b, b);
    if optica.is_empty() {
        return y;
    }
    let y = grupo(c, "panel.painter_layers.wetpaint.oleo_real", y);
    let y = numeros(c, &optica, y);
    let y = lista(
        c,
        "panel.painter_layers.wetpaint.oleo_environment",
        pids::PAINTER_WETPAINT_OLEO_AMBIENTE,
        oleo::nome_do_ambiente(b.wet_oleo_ambiente),
        y,
    );
    let y = grupo(c, "card.painter.wet.material", y);
    numeros(c, &material, y)
}

/// O corpo: pela ordem do painel de antes, com o que cabe aos pares.
pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let pousa = b.le(LinhaDaFerramenta::DepositoMolhado);
    let y = numeros(c, &paint_wetpaint::numeros_da_grade(&b), y);
    let y = c.hint(&paint_wetpaint::texto_da_resolucao(&b), y);
    let y = if b.eraser {
        y
    } else {
        numeros(c, &paint_wetpaint::numeros_da_ferramenta(&b, &b), y)
    };
    let y = caixa(
        c,
        &(
            pids::PAINTER_WETPAINT_TILT_TOGGLE,
            "panel.painter_layers.wetpaint.tilt",
            b.wet_tilt_on,
        ),
        y,
    );
    let (x, w) = (c.x, c.w);
    let y = paint_wetpaint_tilt::visor(&mut Tela::do_cartao(c), x, w, y + m::FIELD_ROW_GAP, &b);
    let borda = paint_wetpaint_tilt::numero_da_borda(&b);
    let quantos = if pousa { 2 } else { 1 };
    let mut y = em_pares(c, &[0, 1][..quantos], y, |c, i, y| {
        if *i == 0 {
            campo(c, &borda, y)
        } else {
            lista(
                c,
                "panel.painter_layers.wetpaint.paint_type",
                pids::PAINTER_WETPAINT_MEDIUM,
                tipo_de_tinta::nome(u8::from(b.wet_knobs.e_oleo())),
                y,
            )
        }
    });
    for (chave, grupo_de) in GRUPOS_DA_FISICA
        .into_iter()
        .zip(paint_wetpaint::numeros_da_fisica(&b, &b))
    {
        if !grupo_de.is_empty() {
            y = grupo(c, chave, y);
            y = numeros(c, &grupo_de, y);
        }
    }
    let y = grupo(c, "card.painter.wet.canvas", y);
    let y = accoes_na_tela(c, y);
    let mut mostrar = vec![(
        pids::PAINTER_WETPAINT_SHOWWET,
        "panel.painter_layers.wetpaint.show_wet",
        b.wet_show_wet,
    )];
    if pousa {
        mostrar.push((
            pids::PAINTER_WETPAINT_PAPER_VISUAL,
            "panel.painter_layers.paper.texture_on_paint",
            b.wet_paper_visual,
        ));
    }
    if pousa {
        mostrar.push((
            pids::PAINTER_WETPAINT_OLEO_REAL,
            "panel.painter_layers.wetpaint.oleo_real",
            b.wet_oleo_real,
        ));
    }
    if b.le(LinhaDaFerramenta::MisturaDoPigmento) {
        mostrar.push((
            pids::PAINTER_WETPAINT_PIGMENT_MIXING,
            "panel.painter_layers.wetpaint.pigment_mixing",
            b.wet_km_mixing,
        ));
    }
    if b.le(LinhaDaFerramenta::AguaViva) {
        mostrar.push((
            pids::PAINTER_WETPAINT_LIVE_WATER,
            "panel.painter_layers.wetpaint.live_water",
            b.wet_live_water,
        ));
    }
    mostrar.push((
        pids::PAINTER_WETPAINT_TUNING,
        "panel.painter_layers.wetpaint.tuning",
        b.wet_tuning_open,
    ));
    let itens: Vec<Item> = mostrar
        .into_iter()
        .map(|(id, chave, on)| Item::Caixa { id, chave, on })
        .collect();
    let y = fila(c, &itens, y);
    oleo_real(c, &b, y)
}

/// Os volumes do Wet Paint: o que o painel de antes semeava ao pintar, e os chips dos valores rápidos.
pub(crate) fn prepara_volumes(store: &mut WidgetStore, b: &BrushSettings) {
    let fabrica = paint_wetpaint_presets::numeros_dos_volumes(b, &paint_brush::FALLBACK_BRUSH);
    for ((n, rapido), (f, _)) in paint_wetpaint_presets::numeros_dos_volumes(b, b)
        .iter()
        .zip(fabrica)
    {
        store.mark_card_field(
            n.id,
            CardFieldSpec {
                step: n.passo,
                scrub: n.passo,
                default: Some(f64::from(f.valor)),
            },
        );
        number_field::arm_field(store, n.id, n.valor, n.min, n.max, n.passo, n.casas);
        if let Some(vol) = rapido {
            registra_lista(store, pids::PAINTER_WETPAINT_VOL_QUICK[*vol], 0);
        }
    }
}

/// Um volume: a caixa e, à direita dela, o chip dos valores rápidos, sem rótulo próprio.
pub(crate) fn volume(c: &mut CardCtx<'_>, n: &Numero, rapido: Option<usize>, y: f32) -> f32 {
    let Some(vol) = rapido else {
        return campo(c, n, y);
    };
    let (x, w) = (c.x, c.w);
    c.w = w - m::FIELD_H - m::FIELD_GAP;
    let fundo = campo(c, n, y);
    c.x = x + c.w + m::FIELD_GAP;
    c.w = m::FIELD_H;
    let chip = Cell::new(Widget::Dropdown {
        id: pids::PAINTER_WETPAINT_VOL_QUICK[vol],
        text: "",
        options: &[],
    });
    let _ = c.row(&[chip], fundo - m::FIELD_H - m::FIELD_ROW_GAP);
    c.x = x;
    c.w = w;
    fundo
}
