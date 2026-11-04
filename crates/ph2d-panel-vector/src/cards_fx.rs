//! ⭐ **Effects e Filters em cartões** (fatia G3, `docs/CartoesUI/12_VECTOR_PLANO.md`). O layout é o de
//! F43 (o «+» no cabeçalho, a lista, as propriedades à esquerda); muda a cara. Mesmos ids e eventos:
//! cada campo é o chip ligado à barra de sempre, e é a barra que avisa o painel.
//!
//! O que cada linha oferece continua a ser resposta da tabela publicada pelo motor (`FilterKindView`,
//! `FxParamView`), como em `paint_effects` e `paint_filters`.

use crate::cards_tinta::{aberto, dica, escolha, seg, seg_icone, semeia, spec};
use crate::paint_sections::filters_ramp::{RAMP_BAR_H, RAMP_HANDLE_R, selected_stop};
use crate::paint_sections::fx_list::{Pilha, anota_props, is_open};
use crate::state;
use crate::state::filters as fst;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_vector::ids as vids;

const SETA_FRACAO: f64 = 0.05; // LITERAL-PX-OK: passo da seta, o do chip no `populate_filters`
const SCRUB_FRACAO: f64 = 0.01; // LITERAL-PX-OK: passo do arrasto por pixel, no domínio do documento
const SETA_MATIZ: f64 = 5.0; // LITERAL-PX-OK: passo da seta em graus de matiz
const SCRUB_PASSOS: f64 = 100.0; // LITERAL-PX-OK: em quantos passos o arrasto atravessa a faixa
const OPACIDADE_CHEIA: f64 = 1.0; // LITERAL-PX-OK: o neutro da opacidade de um degrau (0..1)

/// Um número de um degrau de filtro: o que a pintura mostra e o que o `prepare` marca e semeia.
struct Numero {
    nome: String,
    chip: NodeId,
    barra: NodeId,
    valor: f64,
    seta: f64,
    scrub: f64,
    fabrica: Option<f64>,
}

/// O nome com a unidade no rótulo (a caixa só mostra o número).
fn com_unidade(nome: &str, unidade: &str) -> String {
    tr_with(
        "card.vector.name_unit",
        &[("name", &nome), ("unit", &unidade)],
    )
}

fn comprimento(nome: &str, ids: (NodeId, NodeId), valor: f64, fabrica: Option<f64>) -> Numero {
    Numero {
        nome: com_unidade(nome, state::length_suffix()),
        chip: ids.1,
        barra: ids.0,
        valor,
        seta: SETA_FRACAO,
        scrub: SCRUB_FRACAO,
        fabrica,
    }
}

fn contagem(nome: &str, ids: (NodeId, NodeId), valor: f64) -> Numero {
    Numero {
        nome: nome.to_owned(),
        chip: ids.1,
        barra: ids.0,
        valor,
        seta: 1.0,
        scrub: 1.0,
        fabrica: None,
    }
}

fn fracao(nome: &str, ids: (NodeId, NodeId), valor: f64, fabrica: f64) -> Numero {
    Numero {
        nome: nome.to_owned(),
        chip: ids.1,
        barra: ids.0,
        valor,
        seta: SETA_FRACAO,
        scrub: SCRUB_FRACAO,
        fabrica: Some(fabrica),
    }
}

/// Os números que vêm ANTES da cor, pela ordem de `filter_props`: o modo escolhe a lei, e estes leem-se
/// por ela. Cada um só se a tabela do tipo o oferece.
fn numeros_do_degrau(row: usize, fx: &fst::FilterRowView, k: &fst::FilterKindView) -> Vec<Numero> {
    use crate::ids;
    let mut v = Vec::new();
    if let Some(l) = k.radius_label {
        let par = (ids::filter_radius_id(row), ids::filter_radius_num_id(row));
        v.push(comprimento(l, par, fx.radius, None));
    }
    if let Some((tam, detalhe, semente)) = k.noise_labels {
        let par = (ids::filter_scale_id(row), ids::filter_scale_num_id(row));
        v.push(comprimento(tam, par, fx.scale, None));
        let par = (ids::filter_detail_id(row), ids::filter_detail_num_id(row));
        v.push(contagem(detalhe, par, f64::from(fx.detail).max(1.0)));
        let par = (ids::filter_seed_id(row), ids::filter_seed_num_id(row));
        v.push(contagem(semente, par, f64::from(fx.seed)));
    }
    if let Some(l) = k.grow_label {
        let par = (ids::filter_grow_id(row), ids::filter_grow_num_id(row));
        v.push(comprimento(l, par, fx.grow, Some(0.0)));
    }
    if let Some((h, s, b)) = k.adjust_labels {
        v.push(Numero {
            nome: com_unidade(h, tr("card.vector.unit.degrees")),
            chip: ids::filter_hue_num_id(row),
            barra: ids::filter_hue_id(row),
            valor: fx.hue,
            seta: SETA_MATIZ,
            scrub: 1.0,
            fabrica: Some(0.0),
        });
        let par = (ids::filter_sat_id(row), ids::filter_sat_num_id(row));
        v.push(fracao(s, par, fx.sat, 0.0));
        let par = (ids::filter_bright_id(row), ids::filter_bright_num_id(row));
        v.push(fracao(b, par, fx.bright, 0.0));
    }
    if let Some((lx, ly)) = k.offset_labels {
        let par = (ids::filter_offx_id(row), ids::filter_offx_num_id(row));
        v.push(comprimento(lx, par, fx.offx, Some(0.0)));
        let par = (ids::filter_offy_id(row), ids::filter_offy_num_id(row));
        v.push(comprimento(ly, par, fx.offy, Some(0.0)));
    }
    v
}

fn opacidade_do_degrau(row: usize, fx: &fst::FilterRowView) -> Numero {
    let par = (
        crate::ids::filter_opacity_id(row),
        crate::ids::filter_opacity_num_id(row),
    );
    fracao(
        tr("panel.vector.filter.opacity"),
        par,
        fx.opacity,
        OPACIDADE_CHEIA,
    )
}

fn prepara_numero(store: &mut WidgetStore, n: &Numero) {
    store.mark_card_field(n.chip, spec(n.seta, n.scrub, n.fabrica));
    semeia(store, n.chip, n.valor, false);
}

/// O passo de um parâmetro de efeito: a contagem anda de 1 em 1; o resto, uma fração da faixa, nunca
/// tão grosso que o campo perca as casas (`decimals` lê-as do passo).
fn passos_do_parametro(p: &state::FxParamView) -> (f64, f64) {
    if p.integer {
        (1.0, 1.0)
    } else {
        let s = ((p.max - p.min) / SCRUB_PASSOS).clamp(SCRUB_FRACAO, SETA_FRACAO * 2.0);
        (s, s)
    }
}

/// Com o store MUTÁVEL, antes de pintar: as marcas, as sementes e as dicas das duas pilhas.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for p in Pilha::TODAS {
        let (linha, olho, menos, sobe, desce, mais) = match p {
            Pilha::Efeitos => (
                "card.vector.fx_settings",
                "card.vector.fx_visible",
                "card.vector.fx_remove",
                "card.vector.fx_up",
                "card.vector.fx_down",
                "card.vector.fx_add",
            ),
            Pilha::Filtros => (
                "card.vector.filter_settings",
                "card.vector.filter_visible",
                "card.vector.filter_remove",
                "card.vector.filter_up",
                "card.vector.filter_down",
                "card.vector.filter_add",
            ),
        };
        dica(store, p.add_menu(), mais);
        dica(store, p.close(), "card.vector.close_settings");
        for r in 0..p.max_rows() {
            dica(store, p.row(r), linha);
            dica(store, p.hide(r), olho);
            dica(store, p.remove(r), menos);
            dica(store, p.up(r), sobe);
            dica(store, p.down(r), desce);
        }
    }
    for (row, fx) in state::stack().iter().enumerate().take(vids::MAX_FX_ROWS) {
        for (i, p) in fx.params.iter().enumerate().take(vids::MAX_FX_ROW_PARAMS) {
            if !p.toggle {
                let chip = vids::vector_fx_param_num_id(row, i);
                let (seta, scrub) = passos_do_parametro(p);
                store.mark_card_field(chip, spec(seta, scrub, None));
                semeia(store, chip, p.value, false);
            }
        }
    }
    for (row, fx) in fst::stack()
        .iter()
        .enumerate()
        .take(crate::ids::MAX_FILTER_ROWS)
    {
        dica(
            store,
            crate::ids::filter_stop_add_id(row),
            "card.vector.stop_add",
        );
        dica(
            store,
            crate::ids::filter_stop_remove_id(row),
            "card.vector.stop_remove",
        );
        let Some(k) = fst::kind_spec(fx.kind) else {
            continue;
        };
        for n in numeros_do_degrau(row, fx, &k) {
            prepara_numero(store, &n);
        }
        prepara_numero(store, &opacidade_do_degrau(row, fx));
    }
}

/// Uma linha da lista, F43 com o ritmo de F44: `[glifo nome] [👁]◇ [−]`. A caixa é o que ABRE as
/// propriedades; aberta, leva o anel e fica à espera do passe diferido.
fn linha(
    c: &mut CardCtx<'_>,
    p: Pilha,
    row: usize,
    glifo: IconId,
    nome: &str,
    ligada: bool,
    y: f32,
) -> f32 {
    let y = c.legacy(y, |_, y| y);
    let h = m::FIELD_H;
    let menos = Rect::new(c.x + c.w - h, y, h, h);
    let losango = Rect::new(menos.x - m::KEY_MARK_BOX, y, m::KEY_MARK_BOX, h);
    let olho = Rect::new(losango.x - h, y, h, h);
    let caixa = Rect::new(c.x, y, (olho.x - c.x).max(1.0), h);
    let (id, hide) = (p.row(row), p.hide(row));
    let icone_do_olho = if ligada {
        IconId::Eye
    } else {
        IconId::EyeClosed
    };
    c.bare(olho, &[seg_icone(hide, icone_do_olho, false)]);
    c.bare(menos, &[seg_icone(p.remove(row), IconId::Minus, false)]);
    let alvo = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        c.scene,
        c.ink.theme,
        losango,
        &[hide],
    );
    if let Some(alvo) = alvo {
        c.hit_index.register(alvo, losango);
    }
    let aberta = is_open(c.store, id);
    let fundo = if aberta || c.store.hover_live(id) > 0.0 {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(c.scene, caixa, m::FIELD_RADIUS, c.ink.color(fundo));
    if aberta {
        // FRAME-RAW-OK: o anel da linha cujas propriedades estão abertas, o do campo em foco
        ph2d_editor_core::paint::stroke_rounded_rect(
            c.scene,
            caixa,
            m::FIELD_RADIUS,
            m::FIELD_BORDER,
            c.ink.color(ColorToken::KanbanBlueBold),
        );
        anota_props(p, row, caixa);
    }
    let tinta = c.ink.color(if ligada {
        ColorToken::Text1
    } else {
        ColorToken::CardLabel
    });
    let g = m::ICON_GLYPH;
    paint_icon(
        c.scene,
        glifo,
        Rect::new(caixa.x + (m::ICON_CELL - g) * 0.5, y + (h - g) * 0.5, g, g),
        tinta,
        StrokeToken::Default.px(),
    );
    let tx = caixa.x + m::ICON_CELL;
    paint_text_elided(
        c.text_system,
        c.scene,
        nome,
        tx,
        y + (h - m::VALUE_PX) * 0.5,
        m::VALUE_PX,
        (caixa.x + caixa.w - m::FIELD_PAD_R - tx).max(0.0),
        tinta,
    );
    c.hit_index.register(id, caixa);
    y + h
}

/// ⭐ **O cartão Effects**: as linhas e o Apply. Uma forma presa a ossos diz porquê em vez de sumir.
pub(crate) fn effects(c: &mut CardCtx<'_>, y: f32) -> f32 {
    if state::bound() {
        return c.hint(tr("panel.vector.fx.bound"), y);
    }
    let pilha = state::stack();
    let mut fim = y;
    for (row, fx) in pilha.iter().enumerate().take(vids::MAX_FX_ROWS) {
        fim = linha(
            c,
            Pilha::Efeitos,
            row,
            state::row_icon(row),
            fx.label,
            fx.enabled,
            fim,
        );
    }
    if pilha.is_empty() {
        return fim;
    }
    // O commit: assa a pilha no cozido e esvazia-a, por isso o botão de destaque e o fim da lista.
    c.full(
        None,
        Widget::Button {
            id: vids::VECTOR_FX_APPLY,
            text: tr("panel.vector.fx.apply_effects"),
            primary: true,
        },
        fim,
    )
}

/// ⭐ **O cartão Filters**: as linhas; sem degraus, só o título e o «+».
pub(crate) fn filters(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let mut fim = y;
    for (row, fx) in fst::stack()
        .iter()
        .enumerate()
        .take(crate::ids::MAX_FILTER_ROWS)
    {
        // Um `kind` sem spec publicada não é desenhável: seria uma linha sem nome e sem controlos.
        if let Some(k) = fst::kind_spec(fx.kind) {
            fim = linha(
                c,
                Pilha::Filtros,
                row,
                fst::kind_icon(usize::from(fx.kind)),
                k.name,
                fx.enabled,
                fim,
            );
        }
    }
    fim
}

/// O corpo do painel à esquerda da linha `row`.
pub(crate) fn props(c: &mut CardCtx<'_>, p: Pilha, row: usize, y: f32) -> f32 {
    match p {
        Pilha::Efeitos => state::stack()
            .get(row)
            .map_or(y, |fx| efeito_props(c, row, fx, y)),
        Pilha::Filtros => {
            let Some(fx) = fst::stack().get(row).cloned() else {
                return y;
            };
            fst::kind_spec(fx.kind).map_or(y, |k| filtro_props(c, row, &fx, &k, y))
        }
    }
}

/// Até dois números numa linha, com o rótulo e a marca de cada um. A marca é a da barra (os canais
/// de `marcas*.rs` têm as duas), como no corpo de antes.
fn fila(c: &mut CardCtx<'_>, par: &[(&str, NodeId, NodeId)], y: f32) -> f32 {
    let barras: Vec<[NodeId; 1]> = par.iter().map(|(_, _, b)| [*b]).collect();
    let cells: Vec<Cell<'_>> = par
        .iter()
        .zip(&barras)
        .map(|((nome, chip, _), key)| {
            let glyph = Glyph::Text(corpo::letra_da_grandeza(nome));
            Cell::new(Widget::Field { id: *chip, glyph })
                .label(nome)
                .key(key)
        })
        .collect();
    c.row(&cells, y)
}

fn efeito_props(c: &mut CardCtx<'_>, row: usize, fx: &state::FxRowView, y: f32) -> f32 {
    let nota = match fx.falloff_role {
        state::FalloffRole::ModulatesBelow => Some(tr("panel.vector.fx.falloff.modulates")),
        state::FalloffRole::Inert => Some(tr("panel.vector.fx.falloff.inert")),
        state::FalloffRole::NotFalloff => None,
    };
    let mut y = nota.map_or(y, |n| c.hint(n, y));
    let mut par: Vec<(&str, NodeId, NodeId)> = Vec::new();
    for (i, p) in fx.params.iter().enumerate().take(vids::MAX_FX_ROW_PARAMS) {
        if p.toggle {
            if !par.is_empty() {
                y = fila(c, &par, y);
                par.clear();
            }
            // A caixa é um botão no store: emite `Click`, como o «Nome: On» de antes.
            y = c.check(vids::vector_fx_toggle_id(row, i), p.name, p.value >= 0.5, y);
            continue;
        }
        par.push((
            p.name,
            vids::vector_fx_param_num_id(row, i),
            vids::vector_fx_param_id(row, i),
        ));
        if par.len() == 2 {
            y = fila(c, &par, y);
            par.clear();
        }
    }
    if par.is_empty() { y } else { fila(c, &par, y) }
}

/// A amostra que abre o seletor: a caixa inteira, com o código dentro.
fn amostra<'a>(
    id: NodeId,
    rgba: [u8; 4],
    codigo: &'a str,
    nome: &'a str,
    key: &'a [NodeId],
) -> Cell<'a> {
    Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: codigo,
        edit: None,
        detach: None,
    })
    .label(nome)
    .key(key)
}

fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn cores(
    c: &mut CardCtx<'_>,
    row: usize,
    fx: &fst::FilterRowView,
    k: &fst::FilterKindView,
    y: f32,
) -> f32 {
    let (ka, kb) = (
        [crate::ids::filter_color_id(row)],
        [crate::ids::filter_color_b_id(row)],
    );
    let (ca, cb) = (hex(fx.color), hex(fx.color_b));
    let mut cells = Vec::new();
    if let Some(l) = k.color_label {
        cells.push(amostra(ka[0], fx.color, &ca, l, &ka));
    }
    if let Some(l) = k.color_b_label {
        cells.push(amostra(kb[0], fx.color_b, &cb, l, &kb));
    }
    if cells.is_empty() {
        y
    } else {
        c.row(&cells, y)
    }
}

/// O trilho: o nome em cima, a barra com os punhos, e a cor do stop em foco com `+` e `−` ao lado.
fn rampa(c: &mut CardCtx<'_>, row: usize, fx: &fst::FilterRowView, y: f32) -> f32 {
    let y = c.full(
        Some(tr("card.insp.gradient")),
        Widget::Readout { text: "" },
        y,
    );
    let folga = (m::FIELD_H - RAMP_BAR_H - RAMP_HANDLE_R) * 0.5;
    crate::cards::linhas_do_cartao(c).filter_ramp_bar(row, fx, y - m::FIELD_H + folga);
    let (_, cor) = selected_stop(row, fx);
    let (id, codigo) = (crate::ids::filter_stop_color_id(row), hex(cor));
    let k = [id];
    let botoes = [
        seg_icone(crate::ids::filter_stop_add_id(row), IconId::Plus, false),
        seg_icone(crate::ids::filter_stop_remove_id(row), IconId::Minus, false),
    ];
    c.row(
        &[
            amostra(id, cor, &codigo, tr("panel.vector.filter.stop"), &k),
            Cell::new(Widget::Bare(&botoes)),
        ],
        y,
    )
}

/// Os controlos do degrau, pela ordem da tabela: a lei, os números, a cor, como ela se mistura e a
/// força. Só o que o tipo usa: um knob morto ensina a desconfiar dos vivos.
fn filtro_props(
    c: &mut CardCtx<'_>,
    row: usize,
    fx: &fst::FilterRowView,
    k: &fst::FilterKindView,
    y: f32,
) -> f32 {
    let mut y = y;
    if !k.modes.is_empty() {
        let segs: Vec<_> = k
            .modes
            .iter()
            .enumerate()
            .take(crate::ids::MAX_FILTER_MODES)
            .map(|(m, nome)| {
                seg(
                    crate::ids::filter_mode_id(row, m),
                    crate::nomes_do_motor::modo(nome),
                    u8::try_from(m) == Ok(fx.mode),
                )
            })
            .collect();
        let canal: Vec<NodeId> = segs.iter().map(|s| s.id).collect();
        y = escolha(c, tr("panel.vector.filter.mode"), &canal, &segs, y);
    }
    let numeros = numeros_do_degrau(row, fx, k);
    let tabela: Vec<(&str, NodeId, NodeId)> = numeros
        .iter()
        .map(|n| (n.nome.as_str(), n.chip, n.barra))
        .collect();
    for par in tabela.chunks(2) {
        y = fila(c, par, y);
    }
    y = cores(c, row, fx, k, y);
    if k.takes_ramp {
        y = rampa(c, row, fx, y);
    }
    // A força, e como ela encosta na que já está ali: a mesma linha do Appearance.
    let op = opacidade_do_degrau(row, fx);
    let barras = [op.barra];
    let campo = Cell::new(Widget::Field {
        id: op.chip,
        glyph: Glyph::Icon(IconId::CardOpacity),
    })
    .label(&op.nome)
    .key(&barras);
    if !k.takes_blend {
        return c.row(&[campo], y);
    }
    let id = crate::ids::filter_blend_id(row);
    let chave = [id];
    let y = c.row(
        &[
            campo,
            Cell::new(Widget::Dropdown {
                id,
                text: fst::blend_name(fx.blend),
                options: &[],
            })
            .label(tr("panel.vector.filter.blend"))
            .key(&chave),
        ],
        y,
    );
    if let Some(chip) = aberto(c, id) {
        state::set_pending_blend_dd(Some((row, chip)));
    }
    y
}
