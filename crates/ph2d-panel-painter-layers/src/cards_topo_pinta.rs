//! A pintura das linhas do topo e do Basics em linhas de cartão — filho de [`super`] por `#[path]`
//! (lê o `Numero` e o `Linha` dele); cortado do `cards_topo` pelo tecto de LOC.

use super::*;

fn lista(c: &mut CardCtx<'_>, chave: &str, id: NodeId, texto: &str, y: f32) -> f32 {
    c.full(
        Some(tr(chave)),
        Widget::Dropdown {
            id,
            text: texto,
            options: &[],
        },
        y,
    )
}

/// Uma lista e o Sync, irmãos curtos na mesma linha.
fn lista_e_sync(
    c: &mut CardCtx<'_>,
    b: &BrushSettings,
    com_sync: bool,
    (chave, id): (&str, NodeId),
    texto: &str,
    opcoes: &[&str],
    y: f32,
) -> f32 {
    if !com_sync {
        return lista(c, chave, id, texto, y);
    }
    let celula = Cell::new(Widget::Dropdown {
        id,
        text: texto,
        options: &[],
    })
    .label(tr(chave));
    let sync = marca(
        pids::PAINTER_BRUSH_SYNC,
        "card.painter.brush.sync",
        b.link_shared,
    );
    // Lei 5 de 05 §4: o par desce em vez de cortar «Sync tools» ou a opção mais longa da lista. Mede-se
    // a mais longa, não a de agora: escolher outra opção não muda a arrumação.
    let coluna = (c.w - m::FIELD_GAP) * 0.5;
    let nome = c
        .text_system
        .prefix_width(tr("card.painter.brush.sync"), m::VALUE_PX);
    let longa = opcoes
        .iter()
        .map(|o| c.text_system.prefix_width(o, m::VALUE_PX).ceil())
        .fold(0.0_f32, f32::max);
    let caixa = longa + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH;
    if nome + m::ICON_CELL + m::FIELD_PAD_R > coluna || caixa > coluna {
        let y = c.row(std::slice::from_ref(&celula), y);
        return c.row(std::slice::from_ref(&sync), y);
    }
    c.row(&[celula, sync], y)
}

fn numero_em_caixa(rotulo: &str, id: NodeId) -> Widget<'_> {
    Widget::Field {
        id,
        glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
    }
}

fn numeros(c: &mut CardCtx<'_>, par: &[Numero], y: f32) -> f32 {
    let cells: Vec<Cell<'_>> = par
        .iter()
        .map(|n| {
            let (chave, id, ids) = n.caixa();
            let rotulo = tr(chave);
            Cell::new(numero_em_caixa(rotulo, id))
                .label(rotulo)
                .key(ids)
        })
        .collect();
    c.row(&cells, y)
}

fn cor(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let id = core_ids::PAINTER_COLOR_THUMB;
    let [r, g, bl, a] = c.store.widget_color(id).unwrap_or([0, 0, 0, u8::MAX]);
    let codigo = format!("{r:02X}{g:02X}{bl:02X}");
    let linha = |c: &mut CardCtx<'_>, y: f32| {
        let cell = Cell::new(Widget::Swatch {
            id,
            rgba: Some([r, g, bl, a]),
            text: &codigo,
            edit: None,
            detach: None,
        })
        .label(tr("panel.painter_layers.brush.color"))
        .key(&[core_ids::PAINTER_COLOR_THUMB]);
        c.row(&[cell], y)
    };
    if !b.color_inactive {
        return linha(c, y);
    }
    // Per-Layer Color: cada camada é dona da sua cor. A linha vê-se e não tem alvo.
    let mut fora = HitIndex::new();
    std::mem::swap(c.hit_index, &mut fora);
    let fim = esmaecida(c, y, linha);
    std::mem::swap(c.hit_index, &mut fora);
    fim
}

/// A lista do Preset e, com preset activo, a estrela à direita: mesmo id e mesmo clique da caixa `Favorite`.
fn preset(c: &mut CardCtx<'_>, vista: Option<&VistaPresets>, y: f32) -> f32 {
    let (chave, id) = (
        "panel.painter_layers.wetpaint.preset",
        pids::PAINTER_WETPAINT_PRESET,
    );
    let ativo = vista.and_then(|v| v.procurar(v.ativo.as_deref()?));
    let Some(a) = ativo else {
        let nada = tr("panel.painter_layers.wetpaint.preset_none");
        return lista(c, chave, id, nada, y);
    };
    let celula = Cell::new(Widget::Dropdown {
        id,
        text: a.nome.as_str(),
        options: &[],
    })
    .label(tr(chave));
    let estrela = [Seg {
        id: pids::PAINTER_WETPAINT_PRESET_FAVORITE,
        icon: Some(if a.favorito {
            IconId::CardStarOn
        } else {
            IconId::CardStar
        }),
        text: "",
        on: a.favorito,
    }];
    c.row(&[celula, Cell::new(Widget::Bare(&estrela))], y)
}

fn pinta_linha(c: &mut CardCtx<'_>, linha: &Linha, b: &BrushSettings, y: f32) -> f32 {
    let vista = st::current_wet_presets();
    let vista: Option<&VistaPresets> = vista.as_deref();
    match linha {
        Linha::Modo(com_sync) => lista_e_sync(
            c,
            b,
            *com_sync,
            (
                "panel.painter_layers.brush.paint_mode",
                pids::PAINTER_BRUSH_MEDIA,
            ),
            tr(PaintMedia::from_u8(b.media).name_key()),
            &(0..PaintMedia::COUNT)
                .map(|i| tr(PaintMedia::from_u8(i).name_key()))
                .collect::<Vec<_>>(),
            y,
        ),
        Linha::Preset => preset(c, vista, y),
        Linha::Acoes => vista.map_or(y, |v| {
            let mut todos = biblioteca::botoes_de_guardar(v);
            todos.extend(biblioteca::botoes_da_biblioteca(v));
            botoes(c, &todos, y)
        }),
        Linha::Politica(com_sync) => lista_e_sync(
            c,
            b,
            *com_sync,
            (
                "panel.painter_layers.wetpaint.preset_policy",
                pids::PAINTER_WETPAINT_PRESET_POLICY,
            ),
            paint_wetpaint_presets::nome_da_politica(vista.map_or(0, |v| v.politica)),
            &(0..pids::PAINTER_WETPAINT_PRESET_POLICY_COUNT)
                .map(paint_wetpaint_presets::nome_da_politica)
                .collect::<Vec<_>>(),
            y,
        ),
        Linha::Sync => ferramenta::caixa(
            c,
            pids::PAINTER_BRUSH_SYNC,
            "card.painter.brush.sync",
            b.link_shared,
            y,
        ),
        Linha::Blend => lista(
            c,
            "panel.painter_layers.brush.blend",
            pids::PAINTER_BRUSH_BLEND,
            tr(BrushBlend::from_u8(b.blend).name_key()),
            y,
        ),
        Linha::Vazio => {
            let id = pids::PAINTER_BRUSH_BLEND_ON_EMPTY;
            com_lei(c, id, y, |c, y| {
                let rotulo = tr("panel.painter_layers.brush.blend_on_empty");
                let cell = Cell::new(numero_em_caixa(rotulo, id))
                    .label(rotulo)
                    .key(&[pids::PAINTER_BRUSH_BLEND_ON_EMPTY]);
                c.row(&[cell], y)
            })
        }
        Linha::Cor => cor(c, b, y),
        Linha::Numeros(par) => numeros(c, par, y),
        Linha::Accumulate => com_lei(c, pids::PAINTER_BRUSH_ACCUMULATE, y, |c, y| {
            ferramenta::caixa(
                c,
                pids::PAINTER_BRUSH_ACCUMULATE,
                "panel.painter_layers.brush.accumulate",
                b.accumulate,
                y,
            )
        }),
        Linha::Wash => com_lei(c, pids::PAINTER_BRUSH_WASH, y, |c, y| {
            ferramenta::caixa(
                c,
                pids::PAINTER_BRUSH_WASH,
                "panel.painter_layers.brush.wash",
                b.wash,
                y,
            )
        }),
        Linha::Fio => ferramenta::caixa(
            c,
            pids::PAINTER_BRUSH_SMEAR_FIO,
            "panel.painter_layers.brush.smear_fine_thread",
            b.smear_fio,
            y,
        ),
        Linha::Mistura => crate::cards_mistura::grupo_da_mistura(c, b, y),
        Linha::Clone => cards_ferramenta::clone(c, b, y),
        Linha::Inpaint => cards_ferramenta::inpaint(c, y),
        Linha::Traco { sobra, composto } => cards_ferramenta::linha(c, b, *sobra, *composto, y),
        Linha::Composite => cards_composto::corpo(c, b, y),
        Linha::Previa => {
            if crate::preview_pino::preview_fixada(c.store) {
                return y;
            }
            let (x, w) = (c.x, c.w);
            let voa = topo_a_voar(c.store);
            let guardado = crate::preview_pino::rect_da_preview();
            let mut t = crate::cards_tela::Tela::do_cartao(c);
            let fim = crate::preview_pino::na_tela(&mut t, x, w, y + m::FIELD_ROW_GAP).unwrap_or(y);
            if voa {
                crate::preview_pino::lembra(guardado);
            }
            fim
        }
        Linha::Volume(i) => {
            let volumes = paint_wetpaint_presets::numeros_dos_volumes(b, b);
            volumes
                .get(*i)
                .map_or(y, |(n, rapido)| cards_wet::volume(c, n, *rapido, y))
        }
        Linha::Par(par) => em_pares(c, par.as_slice(), y, |c, l, y| pinta_linha(c, l, b, y)),
    }
}

/// O corpo destes cartões: as linhas, pela ordem.
pub(crate) fn corpo(c: &mut CardCtx<'_>, l: &[Linha], y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    l.iter().fold(y, |y, linha| pinta_linha(c, linha, &b, y))
}
