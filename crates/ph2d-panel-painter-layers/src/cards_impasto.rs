//! O cartão Impasto em linhas de cartão (fatia H4, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos. As
//! caixas viram grupos, a lista TOOL desce em linhas, e o rig de luz fica pela lâmpada escolhida.

use crate::cards_linhas::{escolha, grupo, numero, numero_e_cor};
use crate::cards_wet::{arma_numeros, caixa, campo, numeros};
use crate::paint_wetpaint::Numero;
use crate::{
    esmaecer, number_field as nf, paint, paint_brush, paint_impasto as pi, paint_impasto_tool,
    state,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, PaintMedia};

const UNIDADE: (f32, f32) = (0.0, pi::UNIT_MAX);
/// As opções que a ferramenta pode dizer inertes: a dica é pedida com o store mutável.
const OPCOES: [NodeId; 5] = [
    pids::PAINTER_IMPASTO_SOURCE_UNIFORM,
    pids::PAINTER_IMPASTO_SOURCE_GRAIN,
    pids::PAINTER_IMPASTO_DRAW_BOTH,
    pids::PAINTER_IMPASTO_DRAW_COLOR,
    pids::PAINTER_IMPASTO_DRAW_DEPTH,
];
const LAMPADAS: [NodeId; 4] = [
    pids::PAINTER_IMPASTO_LIGHT_1,
    pids::PAINTER_IMPASTO_LIGHT_2,
    pids::PAINTER_IMPASTO_LIGHT_3,
    pids::PAINTER_IMPASTO_LIGHT_4,
];

/// A resposta do `plano_do_corpo`, com a do tool (`impasto_section_applies`).
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint
        && PaintMedia::from_u8(b.media) == PaintMedia::Impasto
        && b.impasto_section_applies
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.impasto.title")
}

fn do_corpo(b: &BrushSettings) -> [Numero; 4] {
    let f = nf::FINE_STEP;
    [
        numero(
            "panel.painter_layers.impasto.depth",
            pids::PAINTER_IMPASTO_DEPTH,
            b.impasto_depth,
            (pi::DEPTH_MIN, pi::DEPTH_MAX),
            f,
            2,
        ),
        numero(
            "panel.painter_layers.impasto.body",
            pids::PAINTER_IMPASTO_BODY,
            b.impasto_body,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.impasto.push",
            pids::PAINTER_IMPASTO_PUSH,
            b.impasto_push,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.impasto.smoothing",
            pids::PAINTER_IMPASTO_SMOOTHING,
            b.impasto_smoothing,
            UNIDADE,
            f,
            2,
        ),
    ]
}

fn da_faca(b: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.impasto.plow",
        pids::PAINTER_IMPASTO_PLOW,
        b.impasto_plow,
        UNIDADE,
        nf::FINE_STEP,
        2,
    )
}

fn do_material(b: &BrushSettings) -> [Numero; 3] {
    let f = nf::FINE_STEP;
    [
        numero(
            "panel.painter_layers.impasto.shine",
            pids::PAINTER_IMPASTO_SHINE,
            b.impasto_shine,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.impasto.roughness",
            pids::PAINTER_IMPASTO_ROUGHNESS,
            b.impasto_roughness,
            UNIDADE,
            f,
            2,
        ),
        numero(
            "panel.painter_layers.impasto.metallic",
            pids::PAINTER_IMPASTO_METALLIC,
            b.impasto_metallic,
            UNIDADE,
            f,
            2,
        ),
    ]
}

fn da_cera(b: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.impasto.wax",
        pids::PAINTER_IMPASTO_WAX,
        b.impasto_wax,
        UNIDADE,
        nf::FINE_STEP,
        2,
    )
}

/// A lâmpada escolhida em `leis`, com os valores de `valores`.
fn lampada(leis: &BrushSettings, valores: &BrushSettings) -> ph2d_tool_painter::ImpastoLight {
    let i = (leis.impasto_rig.selected as usize).min(LAMPADAS.len() - 1);
    valores.impasto_rig.lights[i]
}

fn da_direcao(leis: &BrushSettings, valores: &BrushSettings) -> [Numero; 2] {
    let l = lampada(leis, valores);
    [
        numero(
            "card.painter.impasto.angle",
            pids::PAINTER_IMPASTO_LIGHT_ANGLE,
            f32::from(l.angle_deg),
            (0.0, pi::ANGLE_MAX_DEG),
            pi::DEG_STEP,
            0,
        ),
        numero(
            "card.painter.impasto.elevation",
            pids::PAINTER_IMPASTO_LIGHT_ELEV,
            f32::from(l.elev_deg),
            (pi::ELEV_MIN_DEG, pi::ELEV_MAX_DEG),
            pi::DEG_STEP,
            0,
        ),
    ]
}

fn da_potencia(leis: &BrushSettings, valores: &BrushSettings) -> Numero {
    numero(
        "panel.painter_layers.impasto.intensity",
        pids::PAINTER_IMPASTO_LIGHT_POWER,
        lampada(leis, valores).intensity,
        (0.0, pi::LIGHT_POWER_MAX),
        nf::FINE_STEP,
        2,
    )
}

fn todos(leis: &BrushSettings, valores: &BrushSettings) -> Vec<Numero> {
    let mut n: Vec<Numero> = do_corpo(valores).into();
    n.push(da_faca(valores));
    n.extend(do_material(valores));
    n.push(da_cera(valores));
    n.extend(da_direcao(leis, valores));
    n.push(da_potencia(leis, valores));
    n
}

fn em_bytes(c: [f32; 3]) -> [u8; 3] {
    c.map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8) // LITERAL-PX-OK: sRGB 8-bit normalize
}

/// A amostra: fechada segue o pincel; com o seletor aberto, o seletor manda e o valor volta à ferramenta.
fn prepara_cor(ctx: &mut PaintCtx, id: NodeId, cor: [f32; 3]) {
    let [r, g, b] = em_bytes(cor);
    let store = ctx.host.store_mut();
    paint::register_button(store, id);
    if store.picker_target() != Some(id) {
        store.set_widget_color(id, [r, g, b, u8::MAX]);
    } else if let Some(p) = store.widget_color(id)
        && [r, g, b] != [p[0], p[1], p[2]]
    {
        ctx.host
            .bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                id,
                format!("{},{},{}", p[0], p[1], p[2]),
            )));
    }
}

/// Com o store e o barramento, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) || !b.impasto {
        return;
    }
    let store = ctx.host.store_mut();
    arma_numeros(store, &todos(b, b), &todos(b, &paint_brush::FALLBACK_BRUSH));
    let caixas = [
        pids::PAINTER_IMPASTO_LIVE_EDIT,
        pids::PAINTER_IMPASTO_SMOOTH_EDGES,
        pids::PAINTER_IMPASTO_SHOW,
        pids::PAINTER_IMPASTO_LIGHT_ON,
    ];
    for id in caixas.into_iter().chain(OPCOES) {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
    prepara_cor(ctx, pids::PAINTER_IMPASTO_WAX_COLOR, b.impasto_wax_color);
    prepara_cor(ctx, pids::PAINTER_IMPASTO_LIGHT_COLOR, lampada(b, b).color);
}

fn ferramentas(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let segs: Vec<Seg<'_>> = pids::PAINTER_IMPASTO_TOOL_IDS
        .iter()
        .zip(paint_impasto_tool::NOMES_DAS_FERRAMENTAS)
        .enumerate()
        .map(|(i, (id, chave))| Seg {
            id: *id,
            icon: None,
            text: tr(chave),
            on: usize::from(b.impasto_tool) == i,
        })
        .collect();
    escolha(c, tr("card.painter.impasto.tool"), &segs, y)
}

fn origem_e_escrita(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let seg = |id, chave: &'static str, on| Seg {
        id,
        icon: None,
        text: tr(chave),
        on,
    };
    let origem = [
        seg(
            pids::PAINTER_IMPASTO_SOURCE_UNIFORM,
            "panel.painter_layers.impasto.uniform",
            b.impasto_source == 0,
        ),
        seg(
            pids::PAINTER_IMPASTO_SOURCE_GRAIN,
            "panel.painter_layers.impasto.grain",
            b.impasto_source == 1,
        ),
    ];
    let y = escolha(
        c,
        tr("panel.painter_layers.impasto.depth_source"),
        &origem,
        y,
    );
    let escrita = [
        seg(
            pids::PAINTER_IMPASTO_DRAW_BOTH,
            "panel.painter_layers.impasto.both",
            b.impasto_draw_to == 0,
        ),
        seg(
            pids::PAINTER_IMPASTO_DRAW_COLOR,
            "panel.painter_layers.impasto.color",
            b.impasto_draw_to == 1,
        ),
        seg(
            pids::PAINTER_IMPASTO_DRAW_DEPTH,
            "panel.painter_layers.impasto.depth",
            b.impasto_draw_to == 2,
        ),
    ];
    escolha(
        c,
        tr("panel.painter_layers.impasto.draw_group"),
        &escrita,
        y,
    )
}

/// A lâmpada apagada diz-o na própria opção: o artista não tem de abrir as quatro para saber quais estão acesas.
fn seletor_de_lampada(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let rig = &b.impasto_rig;
    let sel = usize::from(rig.selected).min(LAMPADAS.len() - 1);
    let nomes: Vec<String> = rig
        .lights
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if l.on && l.intensity > 0.0 {
                format!("{}", i + 1)
            } else {
                format!("{}·", i + 1)
            }
        })
        .collect();
    let segs: Vec<Seg<'_>> = LAMPADAS
        .iter()
        .zip(&nomes)
        .enumerate()
        .map(|(i, (id, nome))| Seg {
            id: *id,
            icon: None,
            text: nome,
            on: i == sel,
        })
        .collect();
    escolha(c, tr("panel.painter_layers.impasto.light"), &segs, y)
}

fn luz(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let y = grupo(c, "panel.painter_layers.impasto.lighting", y);
    let y = caixa(
        c,
        &(
            pids::PAINTER_IMPASTO_SHOW,
            "panel.painter_layers.impasto.show_impasto",
            b.impasto_show,
        ),
        y,
    );
    let mut y = seletor_de_lampada(c, b, y);
    if b.impasto_rig.selected > 0 {
        y = caixa(
            c,
            &(
                pids::PAINTER_IMPASTO_LIGHT_ON,
                "panel.painter_layers.impasto.enable",
                lampada(b, b).on,
            ),
            y,
        );
    }
    let y = numeros(c, &da_direcao(b, b), y);
    numero_e_cor(
        c,
        &da_potencia(b, b),
        "panel.painter_layers.impasto.light_color",
        pids::PAINTER_IMPASTO_LIGHT_COLOR,
        y,
    )
}

/// O corpo, pela ordem das perguntas do painel de antes: o gesto, a ferramenta, o corpo, a faca, o material, a sala.
pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    if !b.impasto {
        return y;
    }
    let y = caixa(
        c,
        &(
            pids::PAINTER_IMPASTO_LIVE_EDIT,
            "panel.painter_layers.impasto.adjust_last_stroke",
            b.impasto_live_edit,
        ),
        y,
    );
    let mut y = ferramentas(c, &b, y);
    if b.impasto && b.is_sculpt {
        y = grupo(c, "panel.painter_layers.impasto.sculpt", y);
        y = crate::cards_ferramenta::escultura(c, &b, y);
    }
    y = grupo(c, "panel.painter_layers.impasto.body", y);
    y = numeros(c, &do_corpo(&b), y);
    y = origem_e_escrita(c, &b, y);
    y = caixa(
        c,
        &(
            pids::PAINTER_IMPASTO_SMOOTH_EDGES,
            "panel.painter_layers.impasto.smooth_edges",
            b.impasto_smooth_edges,
        ),
        y,
    );
    if b.impasto_tool == pi::TOOL_KNIFE {
        y = grupo(c, "panel.painter_layers.impasto.knife", y);
        y = campo(c, &da_faca(&b), y);
    }
    y = grupo(c, "panel.painter_layers.impasto.material", y);
    y = numeros(c, &do_material(&b), y);
    y = numero_e_cor(
        c,
        &da_cera(&b),
        "panel.painter_layers.impasto.wax_color",
        pids::PAINTER_IMPASTO_WAX_COLOR,
        y,
    );
    luz(c, &b, y)
}
