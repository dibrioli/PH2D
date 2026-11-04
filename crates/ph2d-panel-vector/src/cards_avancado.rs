//! ⭐ **Os cinco membros do Advanced** (fatia G6, `docs/CartoesUI/12_VECTOR_PLANO.md`): Blend, Morph,
//! Envelope, Pattern on Path e Contour. O cabeçalho e a dobra de cada um continuam nos pintores de
//! sempre, dentro do grupo; só o corpo é nativo. Mesmos ids e eventos. O que aparece continua a ser
//! resposta das mesmas perguntas: Expand e Release só com envelope, Bend só com preset, e por aí fora.

use crate::cards_fx::{com_unidade, fila};
use crate::cards_tinta::{campo, escolha, par_de_botoes, seg, semeia, spec};
use crate::contour_params::CONTOUR_STEPS_DEFAULT;
use crate::paint_sections::BodyCtx;
use crate::state::contour as cst;
use crate::{ids as p, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::card_ink;
use ph2d_i18n::tr;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::ids as t;
use ph2d_tool_vector::params::{
    BLEND_STEPS_DEFAULT, DrawMode, MORPH_T_DEFAULT, blend_steps_from_track,
};

const PASSO_FINO: f64 = 0.01; // LITERAL-PX-OK: passo do arrasto, no domínio do documento
const POR_CENTO: f64 = 100.0; // LITERAL-PX-OK: fração -> percentual
const PASSO_DO_DESVIO: f64 = 0.1; // LITERAL-PX-OK: passo do arrasto, em percentual do tamanho
const NEUTRO: f64 = 1.0; // LITERAL-PX-OK: Spacing e Accel de fábrica, no domínio do documento
const BEND_DE_FABRICA: f64 = 0.5; // LITERAL-PX-OK: `ph2d_ecs::ENVELOPE_DEFAULT_BEND`
const PINS: u8 = 2;

impl BodyCtx<'_> {
    /// O corpo nativo, quando o pintor corre dentro de um cartão. `None` é o painel de antes.
    pub(crate) fn no_cartao(
        &mut self,
        y: f32,
        f: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
    ) -> Option<f32> {
        let ink = card_ink::card_body()?;
        let mut c = CardCtx::flutuante(
            &mut *self.scene,
            &mut *self.text_system,
            self.store,
            &mut *self.hit_index,
            ink,
            self.inner_x,
            self.inner_w,
        );
        Some(f(&mut c, y))
    }
}

/// Com o store MUTÁVEL, antes de pintar: as marcas e as sementes que o corpo antigo dava ao pintar. A
/// seta anda o passo que o `populate` registou, numa fonte só.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for (chip, scrub, fabrica) in [
        (
            t::VECTOR_BLEND_STEPS_NUM,
            1.0,
            Some(f64::from(BLEND_STEPS_DEFAULT)),
        ),
        (
            t::VECTOR_MORPH_T_NUM,
            PASSO_FINO,
            Some(f64::from(MORPH_T_DEFAULT)),
        ),
        (
            t::VECTOR_ENVELOPE_BEND_NUM,
            PASSO_FINO,
            Some(BEND_DE_FABRICA),
        ),
        (p::VECTOR_PATTERNPATH_SPACING_NUM, PASSO_FINO, Some(NEUTRO)),
        (p::VECTOR_PATTERNPATH_START_NUM, PASSO_FINO, Some(0.0)),
        (p::VECTOR_PATTERNPATH_END_NUM, PASSO_FINO, Some(NEUTRO)),
        // O Slide é derivado de Start e End: não há fábrica a que voltar.
        (p::VECTOR_PATTERNPATH_SLIDE_NUM, PASSO_FINO, None),
        (p::VECTOR_PATTERNPATH_OFFSET_NUM, PASSO_FINO, Some(0.0)),
        (p::VECTOR_PATTERNPATH_ROTATION_NUM, 1.0, Some(0.0)),
        (
            p::VECTOR_CONTOUR_STEPS_NUM,
            1.0,
            Some(CONTOUR_STEPS_DEFAULT),
        ),
        (p::VECTOR_CONTOUR_OFFSET_NUM, PASSO_DO_DESVIO, Some(0.0)),
        (p::VECTOR_CONTOUR_ACCEL_NUM, PASSO_FINO, Some(NEUTRO)),
    ] {
        let seta = store
            .number_range(chip)
            .map_or(scrub, |(_, _, passo)| passo);
        store.mark_card_field(chip, spec(seta, scrub, fabrica));
    }
    let passos = store
        .slider(p::VECTOR_BLEND_STEPS)
        .map(|(_, v)| blend_steps_from_track(f64::from(v)));
    if let Some(n) = passos {
        semeia(store, t::VECTOR_BLEND_STEPS_NUM, f64::from(n), false);
    }
    if let Some((_, v)) = store.slider(p::VECTOR_MORPH_T) {
        semeia(store, t::VECTOR_MORPH_T_NUM, f64::from(v), false);
    }
    if state::envelope_warp().is_some() {
        semeia(
            store,
            t::VECTOR_ENVELOPE_BEND_NUM,
            state::envelope_bend(),
            false,
        );
    }
    if state::pp_linked() {
        let (inicio, fim) = (state::pp_start(), state::pp_end());
        for (id, valor) in [
            (p::VECTOR_PATTERNPATH_SPACING_NUM, state::pp_spacing()),
            (p::VECTOR_PATTERNPATH_START_NUM, inicio),
            (p::VECTOR_PATTERNPATH_END_NUM, fim),
            (p::VECTOR_PATTERNPATH_SLIDE_NUM, (inicio + fim) * 0.5),
            (p::VECTOR_PATTERNPATH_OFFSET_NUM, state::pp_offset()),
            (p::VECTOR_PATTERNPATH_ROTATION_NUM, state::pp_rotation()),
        ] {
            semeia(store, id, valor, false);
        }
    }
    if cst::present() {
        for (id, valor) in [
            (p::VECTOR_CONTOUR_STEPS_NUM, cst::steps()),
            (p::VECTOR_CONTOUR_OFFSET_NUM, cst::d_frac() * POR_CENTO),
            (p::VECTOR_CONTOUR_ACCEL_NUM, cst::accel()),
        ] {
            semeia(store, id, valor, false);
        }
    }
}

fn botao<'a>(id: NodeId, text: &'a str) -> Cell<'a> {
    Cell::new(Widget::Button {
        id,
        text,
        primary: false,
    })
}

fn numero<'a>(id: NodeId, nome: &'a str, key: &'a [NodeId]) -> Cell<'a> {
    Cell::new(campo(id, Glyph::Text(corpo::letra_da_grandeza(nome))))
        .label(nome)
        .key(key)
}

/// ⭐ **Blend**: escolher e ligar na 1.ª linha, os passos com o botão que repõe a espinha, e as duas
/// saídas lado a lado, porque a escolha é entre elas. O Pick Shapes acende enquanto o modo está armado.
pub(crate) fn blend(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    const PASSOS: &[NodeId] = &[p::VECTOR_BLEND_STEPS];
    let y = c.row(
        &[
            Cell::new(Widget::Button {
                id: t::VECTOR_MODE_PICKBLEND,
                text: tr("panel.vector.blend.pick_shapes"),
                primary: snap.mode == DrawMode::PickBlend,
            }),
            botao(p::VECTOR_BLEND_RUN, tr("panel.vector.blend.blend")),
        ],
        y,
    );
    let y = c.row(
        &[
            numero(
                t::VECTOR_BLEND_STEPS_NUM,
                tr("panel.vector.blend.steps"),
                PASSOS,
            ),
            botao(
                p::VECTOR_BLEND_RESET_SPINE,
                tr("panel.vector.blend.reset_spine"),
            ),
        ],
        y,
    );
    par_de_botoes(
        c,
        (p::VECTOR_BLEND_EXPAND, tr("panel.vector.blend.expand")),
        (p::VECTOR_BLEND_RELEASE, tr("panel.vector.blend.release")),
        y,
    )
}

/// ⭐ **Morph**: o botão que cria e, ao lado, o `t` que a timeline keya.
pub(crate) fn morph(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const T: &[NodeId] = &[p::VECTOR_MORPH_T];
    c.row(
        &[
            botao(p::VECTOR_MORPH_RUN, tr("panel.vector.blend.morph")),
            numero(t::VECTOR_MORPH_T_NUM, tr("panel.vector.blend.morph_t"), T),
        ],
        y,
    )
}

/// ⭐ **Envelope**: o botão age sobre a seleção e fica sempre; o resto só existe sobre um envelope. Os
/// presets vêm da tabela publicada, dois por linha, e Expand e Release fecham o cartão.
pub(crate) fn envelope(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const GAIOLA: [NodeId; 3] = [
        p::VECTOR_ENVELOPE_PERSPECTIVE,
        p::VECTOR_ENVELOPE_MESH,
        p::VECTOR_ENVELOPE_PINS,
    ];
    const BEND: &[NodeId] = &[p::VECTOR_ENVELOPE_BEND];
    let mut y = c.full(
        None,
        Widget::Button {
            id: p::VECTOR_ENVELOPE_RUN,
            text: tr("panel.vector.envelope.envelope"),
            primary: false,
        },
        y,
    );
    if !state::has_envelope() {
        return y;
    }
    let modo = state::envelope_mode();
    let opcoes = [
        seg(GAIOLA[0], tr("panel.vector.envelope.persp"), modo == 0),
        seg(GAIOLA[1], tr("panel.vector.envelope.mesh"), modo == 1),
        seg(GAIOLA[2], tr("panel.vector.envelope.pins"), modo == PINS),
    ];
    y = escolha(c, tr("panel.vector.envelope.cage"), &GAIOLA, &opcoes, y);
    if modo == PINS {
        // Pinos não têm gaiola, logo nem preset nem Bend: têm uma saída.
        y = c.full(
            None,
            Widget::Button {
                id: p::VECTOR_ENVELOPE_CLEAR_PINS,
                text: tr("panel.vector.envelope.clear_pins"),
                primary: false,
            },
            y,
        );
    } else {
        let nomes = state::envelope_presets();
        let celulas: Vec<Cell<'_>> = nomes
            .iter()
            .enumerate()
            .map(|(i, nome)| botao(p::vector_envelope_preset_id(i), nome))
            .collect();
        y = celulas.chunks(2).fold(y, |y, par| c.row(par, y));
        if state::envelope_warp().is_some() {
            y = c.row(
                &[numero(
                    t::VECTOR_ENVELOPE_BEND_NUM,
                    tr("panel.vector.envelope.bend"),
                    BEND,
                )],
                y,
            );
        }
    }
    par_de_botoes(
        c,
        (
            p::VECTOR_ENVELOPE_EXPAND,
            tr("panel.vector.envelope.expand"),
        ),
        (
            p::VECTOR_ENVELOPE_RELEASE,
            tr("panel.vector.envelope.release"),
        ),
        y,
    )
}

/// ⭐ **Pattern on Path**: sem vínculo, a única porta que a seleção permite. Com ele, os números aos
/// pares que se lêem juntos: densidade e posição, o trecho, e como a cópia se põe na curva.
pub(crate) fn pattern_on_path(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const LADO: [NodeId; 2] = [p::VECTOR_PATTERNPATH_FLIP_OFF, p::VECTOR_PATTERNPATH_FLIP];
    if !state::pp_linked() {
        let (id, nome) = if state::pp_can_link() {
            (
                p::VECTOR_PATTERNPATH_LINK,
                tr("panel.vector.patternpath.pattern_on_path"),
            )
        } else {
            (
                p::VECTOR_PATTERNPATH_PICK,
                tr("panel.vector.patternpath.pick_path"),
            )
        };
        return c.full(
            None,
            Widget::Button {
                id,
                text: nome,
                primary: false,
            },
            y,
        );
    }
    let densidade = com_unidade(
        tr("panel.vector.patternpath.spacing"),
        tr("card.vector.unit.times"),
    );
    let y = fila(
        c,
        &[
            (
                densidade.as_str(),
                p::VECTOR_PATTERNPATH_SPACING_NUM,
                p::VECTOR_PATTERNPATH_SPACING,
            ),
            (
                tr("panel.vector.patternpath.slide"),
                p::VECTOR_PATTERNPATH_SLIDE_NUM,
                p::VECTOR_PATTERNPATH_SLIDE,
            ),
        ],
        y,
    );
    let y = fila(
        c,
        &[
            (
                tr("panel.vector.patternpath.start"),
                p::VECTOR_PATTERNPATH_START_NUM,
                p::VECTOR_PATTERNPATH_START,
            ),
            (
                tr("panel.vector.patternpath.end"),
                p::VECTOR_PATTERNPATH_END_NUM,
                p::VECTOR_PATTERNPATH_END,
            ),
        ],
        y,
    );
    let desvio = com_unidade(
        tr("panel.vector.patternpath.offset"),
        tr("card.vector.unit.metres"),
    );
    let giro = com_unidade(
        tr("panel.vector.patternpath.rotation"),
        tr("card.vector.unit.degrees"),
    );
    let y = fila(
        c,
        &[
            (
                desvio.as_str(),
                p::VECTOR_PATTERNPATH_OFFSET_NUM,
                p::VECTOR_PATTERNPATH_OFFSET,
            ),
            (
                giro.as_str(),
                p::VECTOR_PATTERNPATH_ROTATION_NUM,
                p::VECTOR_PATTERNPATH_ROTATION,
            ),
        ],
        y,
    );
    let flip = state::pp_flip();
    let lados = [
        seg(LADO[0], tr("panel.vector.patternpath.this_side"), !flip),
        seg(LADO[1], tr("panel.vector.patternpath.other_side"), flip),
    ];
    let y = escolha(c, tr("panel.vector.patternpath.side"), &LADO, &lados, y);
    c.full(
        None,
        Widget::Button {
            id: p::VECTOR_PATTERNPATH_DETACH,
            text: tr("panel.vector.patternpath.detach_from_path"),
            primary: false,
        },
        y,
    )
}

/// ⭐ **Contour**: sem contour, só o botão que cria: uma amostra sem alvo descartaria a escolha. Com ele,
/// passos e distância, a progressão e a cor do último anel, e as duas saídas lado a lado.
pub(crate) fn contour(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const QUINA: [NodeId; 3] = [
        p::VECTOR_CONTOUR_JOIN_MITER,
        p::VECTOR_CONTOUR_JOIN_ROUND,
        p::VECTOR_CONTOUR_JOIN_BEVEL,
    ];
    const LADO: [NodeId; 3] = [
        p::VECTOR_CONTOUR_SIDE_OUTER,
        p::VECTOR_CONTOUR_SIDE_INNER,
        p::VECTOR_CONTOUR_SIDE_BOTH,
    ];
    const ACEL: &[NodeId] = &[p::VECTOR_CONTOUR_ACCEL];
    const COR: &[NodeId] = &[p::VECTOR_CONTOUR_TO];
    if !cst::present() {
        return c.full(
            None,
            Widget::Button {
                id: p::VECTOR_CONTOUR_ADD,
                text: tr("panel.vector.contour.add_contour"),
                primary: false,
            },
            y,
        );
    }
    let distancia = com_unidade(
        tr("panel.vector.contour.offset"),
        tr("card.vector.unit.percent"),
    );
    let y = fila(
        c,
        &[
            (
                tr("panel.vector.contour.steps"),
                p::VECTOR_CONTOUR_STEPS_NUM,
                p::VECTOR_CONTOUR_STEPS,
            ),
            (
                distancia.as_str(),
                p::VECTOR_CONTOUR_OFFSET_NUM,
                p::VECTOR_CONTOUR_OFFSET,
            ),
        ],
        y,
    );
    let cor = cst::to();
    let codigo = format!("{:02X}{:02X}{:02X}", cor[0], cor[1], cor[2]);
    let y = c.row(
        &[
            numero(
                p::VECTOR_CONTOUR_ACCEL_NUM,
                tr("panel.vector.contour.accel"),
                ACEL,
            ),
            Cell::new(Widget::Swatch {
                id: p::VECTOR_CONTOUR_TO,
                rgba: Some(cor),
                text: &codigo,
                edit: None,
                detach: None,
            })
            .label(tr("panel.vector.contour.to"))
            .key(COR),
        ],
        y,
    );
    let (quina, lado) = (cst::join(), cst::side());
    let quinas = [
        seg(QUINA[0], tr("panel.vector.contour.miter"), quina == 0),
        seg(QUINA[1], tr("panel.vector.contour.round"), quina == 1),
        seg(QUINA[2], tr("panel.vector.contour.bevel"), quina == 2),
    ];
    let y = escolha(c, tr("panel.vector.contour.corner"), &QUINA, &quinas, y);
    let lados = [
        seg(LADO[0], tr("panel.vector.contour.outer"), lado == 0),
        seg(LADO[1], tr("panel.vector.contour.inner"), lado == 1),
        seg(LADO[2], tr("panel.vector.contour.both"), lado == 2),
    ];
    let y = escolha(c, tr("panel.vector.contour.side"), &LADO, &lados, y);
    par_de_botoes(
        c,
        (
            p::VECTOR_CONTOUR_EXPAND,
            tr("panel.vector.contour.expand_contour"),
        ),
        (
            p::VECTOR_CONTOUR_REMOVE,
            tr("panel.vector.contour.remove_contour"),
        ),
        y,
    )
}
