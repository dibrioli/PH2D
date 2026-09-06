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

/// As linhas da mola e do gyroid (W124).
#[must_use]
pub(crate) fn dims_lattice(p: &Primitive) -> Vec<Dim> {
    match p {
        Primitive::Helix {
            radius,
            pitch,
            turns,
            thickness,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
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
            // ⚠️ **A MESMA parede da espiral, e é a mesma lei**: com o tubo a encher o passo as
            // voltas encostam-se e a mola vira um cilindro.
            Dim {
                key: "field.dim.thickness",
                value: thickness * 2.0,
                span: Span::Wall((pitch * crate::MAX_SPIRAL_FILL).min(radius * 1.8)),
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::Gyroid {
            half,
            cell,
            thickness,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            // ⚠️ **A célula tem PAREDE na peça**: uma célula maior do que o bloco não desenha uma
            // rede, desenha um pedaço de superfície solto.
            Dim {
                key: "field.dim.cell",
                value: *cell,
                span: Span::Wall(half[0].min(half[1]).min(half[2]) * 2.0 / crate::MIN_GYROID_CELLS),
            },
            Dim {
                key: "field.dim.thickness",
                value: thickness * 2.0,
                span: Span::Wall(cell * crate::MAX_GYROID_FILL),
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        _ => Vec::new(),
    }
}

/// As linhas do cilindro com barriga (W125).
///
/// ⚠️ **Três, e nenhuma de aresta**: o `bulge` já é o arredondamento — ver
/// [`ph2d_field_eval::ops_exact::sd_rounded_cylinder`].
#[must_use]
pub(crate) fn dims_exact(p: &Primitive) -> Vec<Dim> {
    match p {
        Primitive::RoundedCylinder {
            radius,
            bulge,
            half_height,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **A parede do bojo é a menor meia-medida** — acima dela a peça vira uma cápsula e
            // o número deixa de descrever um bordo.
            Dim {
                key: "field.dim.bulge",
                value: *bulge,
                span: Span::Wall(radius.min(*half_height)),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
        ],
        // ─────────────────────────── W127 ───────────────────────────
        Primitive::Superquadric {
            half,
            exponent_top,
            exponent_side,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            // ⚠️ **Adimensionais, logo a vista não tem voto** — ver [`Span::Range`]. As duas pontas
            // saem das cercas medidas do documento.
            Dim {
                key: "field.dim.exponent_top",
                value: *exponent_top,
                span: Span::Range {
                    min: crate::MIN_SUPERQUADRIC_EXPONENT,
                    max: crate::MAX_SUPERQUADRIC_EXPONENT,
                },
            },
            Dim {
                key: "field.dim.exponent_side",
                value: *exponent_side,
                span: Span::Range {
                    min: crate::MIN_SUPERQUADRIC_EXPONENT,
                    max: crate::MAX_SUPERQUADRIC_EXPONENT,
                },
            },
        ],
        // ─────────────────────────── W128 ───────────────────────────
        // ⚠️ **A ordem É a identidade** (o `set_dim` recebe o ÍNDICE): as três medidas, depois os
        // quatro números da curva de CIMA, depois os quatro do PERFIL.
        Primitive::Superformula {
            half,
            top_symmetry,
            top_n1,
            top_n2,
            top_n3,
            side_symmetry,
            side_n1,
            side_n2,
            side_n3,
        } => {
            let mut v = vec![
                Dim {
                    key: "field.dim.width",
                    value: half[0] * 2.0,
                    span: Span::Positive,
                },
                Dim {
                    key: "field.dim.height",
                    value: half[1] * 2.0,
                    span: Span::Positive,
                },
                Dim {
                    key: "field.dim.depth",
                    value: half[2] * 2.0,
                    span: Span::Positive,
                },
            ];
            for (chaves, m, n1, n2, n3) in [
                (
                    [
                        "field.dim.top_symmetry",
                        "field.dim.top_n1",
                        "field.dim.top_n2",
                        "field.dim.top_n3",
                    ],
                    top_symmetry,
                    top_n1,
                    top_n2,
                    top_n3,
                ),
                (
                    [
                        "field.dim.side_symmetry",
                        "field.dim.side_n1",
                        "field.dim.side_n2",
                        "field.dim.side_n3",
                    ],
                    side_symmetry,
                    side_n1,
                    side_n2,
                    side_n3,
                ),
            ] {
                v.push(Dim {
                    key: chaves[0],
                    value: *m,
                    span: Span::Count {
                        min: crate::MIN_SUPERFORMULA_SYMMETRY,
                        max: crate::MAX_SUPERFORMULA_SYMMETRY,
                    },
                });
                v.push(Dim {
                    key: chaves[1],
                    value: *n1,
                    span: Span::Range {
                        min: crate::MIN_SUPERFORMULA_N1,
                        max: crate::MAX_SUPERFORMULA_N1,
                    },
                });
                for (chave, valor) in [(chaves[2], n2), (chaves[3], n3)] {
                    v.push(Dim {
                        key: chave,
                        value: *valor,
                        span: Span::Range {
                            min: crate::MIN_SUPERFORMULA_N,
                            max: crate::MAX_SUPERFORMULA_N,
                        },
                    });
                }
            }
            v
        }
        _ => Vec::new(),
    }
}
