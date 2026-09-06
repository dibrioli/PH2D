//! ⭐ **O QUE CADA FORMA DE FLUXOGRAMA MEDE** (W122) — o terceiro irmão do [`super::dims_table`].
//!
//! # Por que um arquivo irmão
//!
//! O [`super::dims_table_plates`] responde pelas chapas de geometria e delega os sinais ao
//! [`super::dims_table_signs`], que fechou a W120 com `528` das `700` linhas do gate de LOC.
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist** — e o corte é por assunto: aqui está a
//! família do fluxograma.
//!
//! # ⚠️ As cercas destas quatro saem de MEDIÇÃO, e vivem ao lado do número
//!
//! Ver [`crate::validate_flow`] para a tabela de cada uma.

use super::{Dim, Span};
use crate::{Primitive, round_limit};

/// O chanfro de uma chapa — a mesma parede do filete, ver [`super::dims_table::dims`].
fn chamfer_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.chamfer",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// O filete de uma chapa.
fn round_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.round",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// As linhas das quatro formas do fluxograma.
///
/// ⚠️ **Inalcançável com outra primitiva** — quem chega aqui já foi nomeado pelo braço `p @ (…)` do
/// [`super::dims_table::dims`], que continua exaustivo.
#[must_use]
pub(crate) fn dims_flow(p: &Primitive) -> Vec<Dim> {
    match p {
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Positive,
            },
            // ⭐⭐ **A ÚNICA linha desta família que aceita NEGATIVO e ZERO**: o zero é o retângulo
            // (ao bit — a inclinação entra como `k = 0`) e o sinal escolhe para que lado a peça
            // cai. ⚠️ A parede é do DOCUMENTO e não da vista, e sai medida: ver
            // [`crate::MAX_PARALLELOGRAM_SKEW`].
            Dim {
                key: "field.dim.skew",
                value: *skew,
                span: Span::Walls(half_span * crate::MAX_PARALLELOGRAM_SKEW),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::Delay {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            // ⚠️ **A envergadura tem PAREDE, e ela é a largura**: a tampa direita é um semicírculo
            // de raio `half_span`, e com ela mais alta do que a peça é comprida o centro do arco
            // passa para a esquerda da parede — a forma vira-se do avesso.
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Wall(half_width * 2.0 * crate::DELAY_SPAN_OVER_WIDTH),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::Display {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Wall(half_width * 2.0 * crate::DELAY_SPAN_OVER_WIDTH),
            },
            // ⭐ **O zero é o ATRASO**, e passa: os dois flancos do bico ficam sobre a parede
            // esquerda e a forma é a da outra porta. Uma faixa que não o alcançasse esconderia
            // metade do curso do controlo.
            Dim {
                key: "field.dim.point",
                value: *point,
                span: Span::WallFromZero(display_point_wall(*half_width, *half_span)),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::OffPage {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Positive,
            },
            // ⭐ **O zero é o RETÂNGULO** — os dois flancos deitam-se sobre a base.
            Dim {
                key: "field.dim.point",
                value: *point,
                span: Span::WallFromZero(half_span * 2.0 * crate::MAX_OFFPAGE_POINT),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        // ⚠️ **Inalcançável pelo caminho do produto** — ver o cabeçalho. `Vec` vazio e não `panic`:
        // uma forma esquecida aqui aparece como painel sem linhas no gate que a conta.
        _ => Vec::new(),
    }
}

/// ⭐ **Até onde o bico do mostrador pode avançar** — ele tem de acabar antes de a tampa direita
/// começar, senão come o arco e a peça deixa de ter face reta nenhuma.
///
/// `2·half_width − half_span` é onde o bico encosta na tangente da tampa; a fracção medida vive em
/// [`crate::MAX_DISPLAY_POINT`].
#[must_use]
pub(crate) fn display_point_wall(half_width: f32, half_span: f32) -> f32 {
    (half_width.mul_add(2.0, -half_span) * crate::MAX_DISPLAY_POINT).max(f32::MIN_POSITIVE)
}

/// As linhas das duas formas que a W123 tirou do «fica desenhada».
///
/// ⚠️ **Inalcançável com outra primitiva** — o chamador nomeia as seis.
#[must_use]
pub(crate) fn dims_curve(p: &Primitive) -> Vec<Dim> {
    match p {
        Primitive::Spiral {
            radius,
            pitch,
            turns,
            thickness,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **O passo tem PISO na espessura**: com `pitch ≤ 2·thickness` as voltas encostam-se
            // e a fita deixa de ser uma fita. A parede vive do outro lado (na espessura), porque é
            // ela que o artista arrasta para engrossar.
            Dim {
                key: "field.dim.pitch",
                value: *pitch,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.turns",
                value: *turns,
                span: Span::Wall(crate::MAX_SPIRAL_TURNS),
            },
            Dim {
                key: "field.dim.thickness",
                value: thickness * 2.0,
                span: Span::Wall(pitch * crate::MAX_SPIRAL_FILL),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Positive,
            },
            // ⭐ **O zero é o RETÂNGULO** — a base fica reta, e é uma forma.
            Dim {
                key: "field.dim.wave",
                value: *wave,
                span: Span::WallFromZero(half_span * crate::MAX_DOCUMENT_WAVE),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        _ => Vec::new(),
    }
}
