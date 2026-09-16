//! ⭐⭐⭐ **AS BARRAS DO CARD LINE — a tabela que o PINTOR, o REGISTO e o ENCAMINHAMENTO leem.**
//!
//! Irmão do [`crate::paint_line`] pela linha que separa os dois assuntos: aquele diz *como o cartão
//! se desenha*; este diz *que barras o cartão tem, e o que cada número significa*.
//!
//! # ⛔⛔ Porque esta tabela existe
//!
//! Até 2026-09-16 as barras do cartão eram `rótulo | trilho nu | readout` — a forma que a spec §2
//! recusa para um valor com fracção (lá o nome vive DENTRO da barra) e que a ordem do dono de
//! 2026-06-26 já tinha tirado de todo o resto do pincel (*«todos usam o slider-with-chip
//! canónico»*). E a mesma pergunta — *que barras há?* — tinha **três** respostas escritas à mão: a
//! tabela do pintor, a lista do `populate` e a lista do `event_brush_forward`. *Uma barra nova
//! esquecida numa delas nasce pintada e morta, ou registada e invisível.*
//!
//! # ⚠️ O NÚMERO que o chip mostra é a pista vezes UMA escala
//!
//! Cada setter da ferramenta é `valor = pista × MAX` (`line_settings.rs`), logo o número que o
//! artista lê e escreve é **uma** escala da pista, sem deslocamento. A escala vive aqui, uma vez, e
//! o chip e o rótulo derivam dela — ⛔ uma função «o que o readout mostra» ao lado de uma escala
//! «o que o chip projecta» seriam dois mapeamentos que só concordam por sorte, e um mapeamento
//! errado edita o valor errado **em silêncio** (o gate `o_chip_de_cada_barra_projecta_o_que_a_ferramenta_grava`
//! mede isso contra os setters reais).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_tool_painter::{
    BrushSettings, LineKind, ROUGH_AMOUNT_MAX_D, ROUGH_PASSES_MAX, SKETCHY_DENSITY_MAX,
    SKETCHY_REACH_MAX, THREAD_WIDTH_MAX_PX, WIRE_HISTORY_MAX, ids,
};

/// ⭐ **Uma barra do card Line** — o slider, o chip, o nome, a pista e a escala do número.
#[derive(Clone, Copy)]
pub struct Barra {
    /// O slider (a pista `0..1`); é o id que a ferramenta recebe.
    pub slider: NodeId,
    /// O número editável ligado ao slider.
    pub chip: NodeId,
    /// A chave de i18n do nome.
    pub chave: &'static str,
    /// Onde a pista está agora, lida do snapshot do pincel.
    pub pista: fn(BrushSettings) -> f32,
    /// ⭐ **O número que o artista lê é `pista × escala`** — a mesma escala do setter da ferramenta.
    pub escala: f32,
    /// O número é uma CONTAGEM (o chip arredonda e o piso é `1`).
    pub inteiro: bool,
}

impl Barra {
    /// O número que o chip mostra para este pincel.
    #[must_use]
    pub fn numero(&self, b: BrushSettings) -> f32 {
        (self.pista)(b) * self.escala
    }
}

/// A Density é lida em PORCENTAGEM na face do artista.
const PCT: f32 = 100.0; // LITERAL-PX-OK: fator de leitura (fração → porcentagem), não medida de UI
/// As amplitudes do Rough são lidas em **décimos de diâmetro**, para a pista não mostrar sempre `0.x`.
const DECIMOS: f32 = 10.0; // LITERAL-PX-OK: fator de leitura, não medida de UI

/// A `Line Width` — a espessura de UM fio, em pixels. Partilhada pelos três tipos que costuram.
const LARGURA: Barra = Barra {
    slider: ids::PAINTER_LINE_SKETCHY_WIDTH,
    chip: ids::PAINTER_LINE_SKETCHY_WIDTH_CHIP,
    chave: "panel.painter_layers.line.line_width",
    pista: |b| b.thread_width_px / THREAD_WIDTH_MAX_PX,
    escala: THREAD_WIDTH_MAX_PX,
    inteiro: false,
};
/// A `Opacity` de UM fio — a pista É o valor.
const OPACIDADE: Barra = Barra {
    slider: ids::PAINTER_LINE_SKETCHY_OPACITY,
    chip: ids::PAINTER_LINE_SKETCHY_OPACITY_CHIP,
    chave: "panel.painter_layers.line.opacity",
    pista: |b| b.thread_opacity,
    escala: 1.0,
    inteiro: false,
};

/// **Sketchy** — o alcance e o orçamento, mais a tinta do fio.
const SKETCHY: [Barra; 4] = [
    Barra {
        slider: ids::PAINTER_LINE_SKETCHY_REACH,
        chip: ids::PAINTER_LINE_SKETCHY_REACH_CHIP,
        chave: "panel.painter_layers.line.reach",
        pista: |b| b.sketchy_reach / SKETCHY_REACH_MAX,
        escala: SKETCHY_REACH_MAX,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_SKETCHY_DENSITY,
        chip: ids::PAINTER_LINE_SKETCHY_DENSITY_CHIP,
        chave: "panel.painter_layers.line.density",
        pista: |b| b.sketchy_density / SKETCHY_DENSITY_MAX,
        escala: SKETCHY_DENSITY_MAX * PCT,
        inteiro: false,
    },
    LARGURA,
    OPACIDADE,
];

/// **Wire** — a janela, mais a tinta do fio.
const WIRE: [Barra; 3] = [
    Barra {
        slider: ids::PAINTER_LINE_WIRE_HISTORY,
        chip: ids::PAINTER_LINE_WIRE_HISTORY_CHIP,
        chave: "panel.painter_layers.line.history",
        pista: |b| b.wire_history / WIRE_HISTORY_MAX,
        escala: WIRE_HISTORY_MAX,
        inteiro: false,
    },
    LARGURA,
    OPACIDADE,
];

/// **Ribbon** — a mola (quanto atrasa · como assenta · quanto pende · as travessas), mais a tinta.
const RIBBON: [Barra; 6] = [
    Barra {
        slider: ids::PAINTER_LINE_RIBBON_WEIGHT,
        chip: ids::PAINTER_LINE_RIBBON_WEIGHT_CHIP,
        chave: "panel.painter_layers.line.weight",
        pista: |b| b.ribbon_weight,
        escala: 1.0,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_RIBBON_FRICTION,
        chip: ids::PAINTER_LINE_RIBBON_FRICTION_CHIP,
        chave: "panel.painter_layers.line.friction",
        pista: |b| b.ribbon_friction,
        escala: 1.0,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_RIBBON_GRAVITY,
        chip: ids::PAINTER_LINE_RIBBON_GRAVITY_CHIP,
        chave: "panel.painter_layers.line.gravity",
        pista: |b| b.ribbon_gravity,
        escala: 1.0,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_RIBBON_RUNGS,
        chip: ids::PAINTER_LINE_RIBBON_RUNGS_CHIP,
        chave: "panel.painter_layers.line.rungs",
        pista: |b| b.ribbon_rungs,
        escala: 1.0,
        inteiro: false,
    },
    LARGURA,
    OPACIDADE,
];

/// **Rough** — as duas amplitudes (duas oitavas do MESMO desvio, com o MESMO teto) e as passadas.
///
/// ⚠️ **Sem tinta de fio**: o `Rough` não costura nada, desenha o traço outra vez com os dabs do
/// próprio pincel.
const ROUGH: [Barra; 3] = [
    Barra {
        slider: ids::PAINTER_LINE_ROUGH_AMOUNT,
        chip: ids::PAINTER_LINE_ROUGH_AMOUNT_CHIP,
        chave: "panel.painter_layers.line.roughness",
        pista: |b| b.rough_amount / ROUGH_AMOUNT_MAX_D,
        escala: ROUGH_AMOUNT_MAX_D * DECIMOS,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_ROUGH_BOWING,
        chip: ids::PAINTER_LINE_ROUGH_BOWING_CHIP,
        chave: "panel.painter_layers.line.bowing",
        pista: |b| b.rough_bowing / ROUGH_AMOUNT_MAX_D,
        escala: ROUGH_AMOUNT_MAX_D * DECIMOS,
        inteiro: false,
    },
    Barra {
        slider: ids::PAINTER_LINE_ROUGH_PASSES,
        chip: ids::PAINTER_LINE_ROUGH_PASSES_CHIP,
        chave: "panel.painter_layers.line.passes",
        #[allow(clippy::cast_precision_loss)]
        pista: |b| b.rough_passes as f32 / ROUGH_PASSES_MAX as f32,
        #[allow(clippy::cast_precision_loss)]
        escala: ROUGH_PASSES_MAX as f32,
        inteiro: true,
    },
];

/// As barras de um tipo. `None`/`Speed` não têm **de propósito** — o Alchemy não oferece controlo
/// sobre o arremesso (Enio 2026-08-13), e uma barra sob eles seria um controlo que não faz nada.
#[must_use]
pub fn barras_de(kind: LineKind) -> &'static [Barra] {
    match kind {
        LineKind::None | LineKind::Speed => &[],
        LineKind::Sketchy => &SKETCHY,
        LineKind::Wire => &WIRE,
        LineKind::Ribbon => &RIBBON,
        LineKind::Rough => &ROUGH,
    }
}

/// ⭐ **Todas as barras distintas do cartão** — a tinta de fio aparece em três tipos e conta UMA vez.
#[must_use]
pub fn todas() -> Vec<Barra> {
    let mut out: Vec<Barra> = Vec::new();
    for kind in crate::paint_line::LINE_KINDS {
        for b in barras_de(kind) {
            if !out.iter().any(|o| o.slider == b.slider) {
                out.push(*b);
            }
        }
    }
    out
}

/// `true` quando `id` é o slider de uma barra do cartão — a pergunta do encaminhamento.
#[must_use]
pub fn e_barra(id: NodeId) -> bool {
    crate::paint_line::LINE_KINDS
        .iter()
        .any(|k| barras_de(*k).iter().any(|b| b.slider == id))
}

/// ⭐⭐ **Regista cada barra: o slider, o chip e a LIGAÇÃO entre os dois** — uma vez, no arranque.
///
/// ⚠️ **O chip vive no espaço do NÚMERO** (`link_slider_number_mapped`): a edição dele é projectada
/// de volta à pista pela mesma escala com que ele a mostra, e chega à ferramenta como o
/// `ValueChanged` do slider — o encaminhamento de sempre.
pub(crate) fn registar(store: &mut WidgetStore) {
    for b in todas() {
        store.register(
            b.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            b.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        if b.inteiro {
            store.link_slider_number_mapped_integer(b.slider, b.chip, b.escala, 0.0);
            // ⚠️ **O piso é `1`, não `0`** — é o piso do setter (`set_rough_passes_norm`): zero
            //    passadas seria um traço que não pinta nada.
            store.set_number_range(b.chip, 1.0, f64::from(b.escala), 1.0);
        } else {
            store.link_slider_number_mapped(b.slider, b.chip, b.escala, 0.0);
            // O passo é 1 % da pista, no espaço do número — o mesmo passo que as barras irmãs usam.
            const PASSO: f64 = 0.01; // LITERAL-PX-OK: passo de 1 % da pista (comportamento, não layout)
            store.set_number_range(
                b.chip,
                0.0,
                f64::from(b.escala),
                PASSO * f64::from(b.escala),
            );
        }
    }
}
