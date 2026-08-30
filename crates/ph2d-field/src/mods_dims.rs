//! ⭐⭐ **OS NÚMEROS DE UM MODIFICADOR** — quais são, em que ordem, e como se escrevem.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::mods`] passou as `700` linhas do gate de LOC da workspace quando a **junta entre as
//! cópias** entrou (pedido do Enio, 2026-08-30). ⛔ *Split, nunca allowlist* — e o corte é por
//! assunto: aqui está tudo o que responde *«que números este modificador tem, e o que acontece
//! quando alguém arrasta um»*, e nada do que responde *«o que um modificador É»*.
//!
//! ⚠️ **A `impl Unary` continua a mesma** — Rust deixa um tipo ter blocos `impl` em módulos
//! diferentes da mesma crate, então `Unary::dims()` continua a chamar-se assim de fora. *Cortar um
//! arquivo não pode custar uma reescrita em cada sítio que o chamava.*
//!
//! ⚠️⚠️ **O par [`Unary::dims`] / [`Unary::set_dim`] é UM contrato, e por isso os dois estão aqui.**
//! Uma `Dim` emitida sem braço correspondente no `match` da escrita cai no `_ => Err(bad("mod"))`:
//! o slider pinta, arrasta, o intent nasce, e o `let _ =` do shell engole o erro. É o **dreno de um
//! braço só** do `CLAUDE.md` §5.0, e mantê-los no mesmo ficheiro é o que o torna visível.

use crate::{FieldError, Joint, Span, Unary};

use crate::mods::{MAX_ARRAY_COUNT, MAX_BEND_TURNS, MAX_TAPER_SLOPE, MAX_TWIST_TURNS};

/// ⭐⭐⭐ **AS DUAS FILEIRAS DE UMA JUNTA** — a mesma lei nos dois modificadores que geram cópias.
///
/// ⚠️ **Uma função, e não a mesma escada escrita duas vezes.** A matriz e a coroa mostram as mesmas
/// duas fileiras com as mesmas faixas, e enquanto isso foi copiado noutros sítios desta casa os dois
/// lados acabaram por discordar sobre o que um zero faz.
///
/// ⚠️ **Elas vão no FIM da lista, e isso é load-bearing:** o índice `field` de um
/// [`crate::Param::Mod`] é **posicional e não serializado**, então acrescentar no fim é seguro e
/// inserir no meio renumera — um arrasto em curso passaria a escrever noutro campo.
///
/// ⚠️ **Sem parede**, e a ausência é declarada: uma junta maior do que meio espaçamento funde as
/// cópias numa massa, o que é uma forma legítima e não um erro — como a casca mais grossa do que a
/// peça. ⛔ Um teto aqui teria de nomear de que recurso ele é, e não há nenhum medido.
fn joint_dims(joint: Joint) -> [crate::Dim; 2] {
    [
        crate::Dim {
            key: "field.mod.joint_chamfer",
            value: joint.chamfer,
            span: Span::Positive,
        },
        crate::Dim {
            key: "field.mod.joint_fillet",
            value: joint.fillet,
            span: Span::Positive,
        },
    ]
}

/// Escreve um dos dois números de uma junta. `field` é `0` para o chanfro e `1` para o filete — a
/// mesma ordem da [`joint_dims`], que é a ordem em que o pedido os nomeia.
fn set_joint(joint: &mut Joint, field: u8, value: f32) {
    // ⚠️ **Coage em vez de recusar** (a lei da banda da torção): um número negativo não é alcançável
    // pelo painel, e recusá-lo faria o `FieldDoc::new` recusar a PEÇA quando o documento revalida.
    let value = value.max(0.0);
    if field == 0 {
        joint.chamfer = value;
    } else {
        joint.fillet = value;
    }
}

impl Unary {
    /// ⭐ **Os números deste modificador**, na ordem em que o painel os mostra.
    ///
    /// ⚠️ **Vários, e não um** — foi o que a matriz forçou, e é a forma certa: uma matriz tem
    /// quantas cópias **e** que espaçamento, e enfiá-las em dois modificadores separados seria
    /// partir uma coisa em duas para caber num campo. É a mesma forma que [`crate::dims`] já usa
    /// para uma primitiva — *um vocabulário, não dois*.
    ///
    /// Um modificador **sem números** (o espelho) devolve vazio, e o painel não pinta linha nenhuma
    /// para ele: o chip aceso já diz tudo o que há para dizer.
    #[must_use]
    pub fn dims(self) -> Vec<crate::Dim> {
        match self {
            Unary::Shell { thickness } => vec![crate::Dim {
                key: "field.mod.thickness",
                value: thickness,
                // Sem parede: uma casca mais grossa do que a peça deixa de ser oca, o que é uma
                // forma legítima e não um erro. O alcance útil é o da vista.
                span: Span::Positive,
            }],
            // ⚠️ **Simétrica**, e é metade da razão de o afastamento existir: encolher é o gesto de
            // folga de encaixe. Uma faixa só positiva mataria metade da ferramenta.
            Unary::Offset { distance } => vec![crate::Dim {
                key: "field.mod.distance",
                value: distance,
                span: Span::Free,
            }],
            Unary::Mirror | Unary::MirrorY | Unary::MirrorZ => Vec::new(),
            Unary::Array {
                count,
                spacing,
                joint,
            } => {
                let mut linhas = vec![
                    crate::Dim {
                        key: "field.mod.count",
                        value: count as f32,
                        span: Span::Count {
                            min: 1,
                            max: MAX_ARRAY_COUNT,
                        },
                    },
                    crate::Dim {
                        key: "field.mod.spacing",
                        value: spacing,
                        span: Span::Positive,
                    },
                ];
                linhas.extend(joint_dims(joint));
                linhas
            }
            // ⚠️ **Sem espaçamento**: numa coroa o espaçamento é o próprio ângulo, e ele já está
            // dito pela contagem (`2π/n`). Um segundo número aqui seria uma forma de pedir uma
            // coroa incompleta — que é outra feature, com outro nome.
            Unary::Taper { slope } => vec![crate::Dim {
                key: "field.mod.slope",
                value: slope,
                // ⚠️ Uma parede dos **dois** lados: inclinar para dentro e para fora são os dois
                // gestos, e o teto é do CUSTO da marcha (ver `MAX_TAPER_SLOPE`).
                span: Span::Walls(MAX_TAPER_SLOPE),
            }],
            Unary::Radial { count, joint } => {
                let mut linhas = vec![crate::Dim {
                    key: "field.mod.count",
                    value: count as f32,
                    span: Span::Count {
                        min: 1,
                        max: MAX_ARRAY_COUNT,
                    },
                }];
                linhas.extend(joint_dims(joint));
                linhas
            }
            // ⚠️ **Os dois sentidos** (como a inclinação): torcer para um lado e para o outro são os
            // dois gestos, e o teto é do CUSTO da marcha — ver [`MAX_TWIST_TURNS`].
            // ⚠️ E os limites são **posições** (`Span::Free`): a origem não é um canto do mundo.
            Unary::Twist {
                turns,
                lower,
                upper,
                falloff,
            } => vec![
                crate::Dim {
                    key: "field.mod.turns",
                    value: turns,
                    span: Span::Walls(MAX_TWIST_TURNS),
                },
                crate::Dim {
                    key: "field.mod.from",
                    value: lower,
                    span: Span::Free,
                },
                crate::Dim {
                    key: "field.mod.to",
                    value: upper,
                    span: Span::Free,
                },
                // ⚠️ **`FromZero` e não `Positive`**: o zero é uma resposta (o corte duro), e não
                // uma recusa — é a mesma cerca que o cone fechado já usa.
                crate::Dim {
                    key: "field.mod.falloff",
                    value: falloff,
                    span: Span::FromZero,
                },
            ],
            // ⚠️ **As mesmas quatro linhas da torção, na mesma ordem** — ver o doc da variante.
            Unary::Bend {
                turns,
                lower,
                upper,
                falloff,
            } => vec![
                crate::Dim {
                    key: "field.mod.turns",
                    value: turns,
                    span: Span::Walls(MAX_BEND_TURNS),
                },
                crate::Dim {
                    key: "field.mod.from",
                    value: lower,
                    span: Span::Free,
                },
                crate::Dim {
                    key: "field.mod.to",
                    value: upper,
                    span: Span::Free,
                },
                crate::Dim {
                    key: "field.mod.falloff",
                    value: falloff,
                    span: Span::FromZero,
                },
            ],
        }
    }

    /// ⭐ **Escreve um dos números**, ou recusa — a porta única.
    ///
    /// # Errors
    /// [`FieldError::NonPositive`] para um valor não-finito, para um índice que não é deste
    /// modificador, e para os números cujo zero não quer dizer nada (uma casca sem parede não é uma
    /// casca; uma matriz de espaçamento zero é N cópias no mesmo sítio).
    pub fn set_dim(&mut self, node: u32, field: u8, value: f32) -> Result<(), FieldError> {
        let bad = |what: &'static str| FieldError::NonPositive { node, what };
        if !value.is_finite() {
            return Err(bad("mod"));
        }
        match (&mut *self, field) {
            (Unary::Shell { thickness }, 0) => {
                if value <= 0.0 {
                    return Err(bad("thickness"));
                }
                *thickness = value;
            }
            // ⚠️ **Zero é legítimo aqui**: um afastamento de zero é o campo intacto, e é o ponto por
            // onde o número passa ao ir de encolher para crescer. Recusá-lo faria o slider ter um
            // buraco no meio.
            (Unary::Offset { distance }, 0) => *distance = value,
            (Unary::Array { count, .. }, 0) => {
                // ⚠️ **O documento é quem arredonda.** O painel mostra um inteiro porque a faixa diz
                // que ele é um (`Span::Count`), mas quem garante é esta linha — um valor fracionário
                // que chegasse por outra porta viraria `count` na mesma, e não meia cópia.
                if value < 1.0 {
                    return Err(bad("count"));
                }
                *count = (value.round() as u32).min(MAX_ARRAY_COUNT);
            }
            (Unary::Array { spacing, .. }, 1) => {
                if value <= 0.0 {
                    return Err(bad("spacing"));
                }
                *spacing = value;
            }
            (Unary::Taper { slope }, 0) => {
                *slope = value.clamp(-MAX_TAPER_SLOPE, MAX_TAPER_SLOPE);
            }
            // ⚠️ **Aceita zero e negativo**: a faixa é dos dois lados e o zero é a peça intacta.
            // Recusar aqui faria o `FieldDoc::new` recusar a PEÇA INTEIRA quando o documento se
            // revalida — a armadilha que o `Offset` já nomeia acima.
            (Unary::Twist { turns, .. }, 0) => {
                *turns = value.clamp(-MAX_TWIST_TURNS, MAX_TWIST_TURNS);
            }
            // ⚠️ **Zero é legítimo**: é o corte duro, e é o estado de onde o ombro se arrasta.
            (Unary::Twist { falloff, .. }, 3) => {
                *falloff = value.max(0.0);
            }
            // ⚠️ **A dobra escreve pela MESMA lei da torção**, linha a linha — ver o doc dela.
            (Unary::Bend { turns, .. }, 0) => {
                *turns = value.clamp(-MAX_BEND_TURNS, MAX_BEND_TURNS);
            }
            (Unary::Bend { lower, upper, .. }, 1) => {
                *lower = value;
                *upper = upper.max(value);
            }
            (Unary::Bend { lower, upper, .. }, 2) => {
                *upper = value;
                *lower = lower.min(value);
            }
            (Unary::Bend { falloff, .. }, 3) => {
                *falloff = value.max(0.0);
            }
            // ⚠️ **A banda COAGE em vez de recusar**, e as duas pontas são simétricas na lei: quem
            // escreve um `from` acima do `to` empurra o outro, em vez de ver o número saltar para
            // trás debaixo do dedo. *Uma porta que recusa uma ordem legítima ensina o artista a não
            // usar o controle.*
            (Unary::Twist { lower, upper, .. }, 1) => {
                *lower = value;
                *upper = upper.max(value);
            }
            (Unary::Twist { lower, upper, .. }, 2) => {
                *upper = value;
                *lower = lower.min(value);
            }
            (Unary::Radial { count, .. }, 0) => {
                if value < 1.0 {
                    return Err(bad("count"));
                }
                *count = (value.round() as u32).min(MAX_ARRAY_COUNT);
            }
            // ⭐ **A junta entre as cópias**, nos dois modificadores que as geram — e nos dois pela
            // MESMA função, com o índice base a ser a única diferença. Ver [`joint_dims`].
            //
            // ⚠️ **Zero é legítimo e é o estado de nascimento** (a costura viva), pela razão que o
            // `Offset` acima já nomeia: recusá-lo faria o slider ter um buraco na ponta de baixo.
            (Unary::Array { joint, .. }, 2 | 3) => set_joint(joint, field - 2, value),
            (Unary::Radial { joint, .. }, 1 | 2) => set_joint(joint, field - 1, value),
            _ => return Err(bad("mod")),
        }
        Ok(())
    }
}
