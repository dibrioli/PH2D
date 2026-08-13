//! **O NÚMERO de um arrasto de gizmo** — quanto o gesto já andou, girou ou escalou.
//!
//! Mover, escalar e girar são os gestos mais usados do app inteiro, e até esta wave nenhum deles
//! punha um número sobre o canvas: o artista lia o Inspector, do outro lado da tela. Ver
//! [`crate::readout`] para a ficha que o desenha e para a medição que a justifica.
//!
//! # A regra: o readout é DERIVADO do resultado APLICADO, nunca re-calculado do cursor
//!
//! [`gizmo_readout`] recebe o `start_transform` que o arrasto fotografou e o `Transform` **VIVO**
//! da entidade — isto é, o número que o produto de facto escreveu, seja qual for o braço de
//! [`super::compute_gizmo_transform`] que o escreveu. Uma segunda derivação a partir do cursor
//! discordaria do encaixe (o Ctrl quantiza a POSIÇÃO, o Shift quantiza o ÂNGULO) e mostraria
//! `12,03` enquanto a forma pousou em `12,00` — a doença que [[feedback_derived_coordinate_seed_must_match_sample]]
//! nomeia, aqui entre o que se vê e o que se lê.
//!
//! É também o que faz esta porta cobrir os braços todos — incluindo os que ainda não existem — sem
//! uma lista de sítios de publicação a apodrecer.
//!
//! # Aditivo × multiplicativo
//!
//! Posição e ângulo somam-se, logo a variação deles é uma **diferença**; escala multiplica-se, logo
//! a variação dela é uma **razão**. Não são duas convenções: é a mesma frase — *o que este gesto
//! fez* — dita na álgebra de cada campo. Blender mostra exactamente isto (`D:` para mover,
//! `Scale X: 1.2000` para escalar); Illustrator e Affinity mostram `dX/dY` e uma percentagem.
//!
//! ⚠️ **O delta da translação é LOCAL**, porque `Transform.translation` é local e é isso que o
//! Inspector mostra. Sob um pai rodado o delta de MUNDO é outro número — e a ficha que o dissesse
//! contradiria o campo Position que o artista vê a mudar ao lado. Há gate com um pai rodado.

use super::drag::{GizmoDragKind, TransformSnapshot};
use crate::length::LengthDisplay;

/// Quanto um arrasto de gizmo já fez — na álgebra do campo que ele mexe.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GizmoReadout {
    /// Translate e MovePivot: a diferença de posição, no frame LOCAL da entidade.
    Moved { dx: f64, dy: f64 },
    /// Scale: a razão aplicada sobre a escala de partida.
    Scaled { rx: f64, ry: f64 },
    /// Rotate: o ângulo varrido, em graus, com sinal e **para além de ±360°** (o contador de voltas
    /// do arrasto entra na rotação escrita, então a diferença o carrega de graça).
    Turned { degrees: f64 },
}

/// Uma razão de escala precisa de um denominador; uma escala de partida nula não tem um.
const SCALE_EPS: f32 = 1e-9;

/// O que dizer sobre este arrasto, dados o instantâneo de partida e o `Transform` VIVO.
#[must_use]
pub fn gizmo_readout(
    kind: GizmoDragKind,
    start: &TransformSnapshot,
    now: &TransformSnapshot,
) -> GizmoReadout {
    match kind {
        GizmoDragKind::Translate | GizmoDragKind::MovePivot => GizmoReadout::Moved {
            dx: f64::from(now.translation[0] - start.translation[0]),
            dy: f64::from(now.translation[1] - start.translation[1]),
        },
        GizmoDragKind::ScaleCorner { .. } | GizmoDragKind::ScaleEdge { .. } => {
            let ratio = |s: f32, n: f32| {
                if s.abs() > SCALE_EPS {
                    f64::from(n / s)
                } else {
                    1.0
                }
            };
            GizmoReadout::Scaled {
                rx: ratio(start.scale[0], now.scale[0]),
                ry: ratio(start.scale[1], now.scale[1]),
            }
        }
        GizmoDragKind::Rotate => GizmoReadout::Turned {
            degrees: f64::from((now.rotation - start.rotation).to_degrees()),
        },
    }
}

/// Casas decimais de uma razão de escala. Duas: o passo do campo Scale do Inspector é `0,1`, e uma
/// razão é adimensional — não há resolução de tela a consultar como há num comprimento.
const SCALE_DECIMALS: usize = 2;
/// Casas de um ângulo. Uma: o campo Rotation do Inspector tem passo `1`, e um décimo de grau é o
/// limite do que um arrasto de rato resolve.
const ANGLE_DECIMALS: usize = 1;

/// Prefixa `+` a um número já formatado que não venha com sinal.
///
/// ⚠️ O sinal explícito é o que torna a ficha legível como VARIAÇÃO. Sem ele, `12.5` ao lado de um
/// Inspector que mostra `137.5` é indistinguível de um absoluto — e ler o número errado como se
/// fosse o certo é pior do que não ter número.
fn signed(s: String) -> String {
    if s.starts_with('-') {
        s
    } else {
        format!("+{s}")
    }
}

impl GizmoReadout {
    /// **O gesto ainda não fez nada** — e por isso ainda não há número a mostrar.
    ///
    /// ⚠️ Isto não é higiene: **um clique de SELEÇÃO no canvas abre um arrasto de Translate**
    /// (`input_dispatch`, o caminho do pick), então sem esta pergunta a ficha piscaria `+0.0, +0.0`
    /// a cada clique em cada objeto do app.
    ///
    /// A comparação é EXACTA de propósito, e é exacta por construção: com o `Transform` vivo igual
    /// ao instantâneo de partida a diferença é `0.0` ao bit e a razão é `x/x == 1.0` ao bit. Um
    /// épsilon aqui seria um limiar inventado — e faria a ficha aparecer num sítio que ninguém
    /// escolheu.
    #[must_use]
    pub fn is_idle(self) -> bool {
        match self {
            Self::Moved { dx, dy } => dx == 0.0 && dy == 0.0,
            Self::Scaled { rx, ry } => rx == 1.0 && ry == 1.0,
            Self::Turned { degrees } => degrees == 0.0,
        }
    }

    /// O texto da ficha.
    ///
    /// ⚠️ Comprimentos passam pela porta única [`LengthDisplay`] — a MESMA que o Inspector, a
    /// régua e o rótulo do smart guide usam. Sem ela a ficha diria `1,2` em metros enquanto o campo
    /// Position ao lado diz `120` em pixels, que é o defeito que aquela porta nasceu para matar.
    ///
    /// `px_per_world` é o zoom vivo: as casas decimais seguem a resolução que a tela de facto
    /// mostra, como no rótulo do smart guide.
    #[must_use]
    pub fn text(self, display: LengthDisplay, px_per_world: f64) -> String {
        match self {
            Self::Moved { dx, dy } => format!(
                "{}, {} {}",
                signed(display.text_at_zoom(dx, px_per_world)),
                signed(display.text_at_zoom(dy, px_per_world)),
                display.suffix()
            ),
            Self::Scaled { rx, ry } => {
                let fx = format!("{rx:.SCALE_DECIMALS$}");
                let fy = format!("{ry:.SCALE_DECIMALS$}");
                // Um arrasto uniforme (o caso do Shift, e o comum) diz UM número: repetir o mesmo
                // valor duas vezes é ruído sobre a única coisa que a ficha tem para dizer.
                if fx == fy {
                    format!("\u{d7}{fx}")
                } else {
                    format!("\u{d7}{fx}, \u{d7}{fy}")
                }
            }
            Self::Turned { degrees } => {
                format!("{}\u{b0}", signed(format!("{degrees:.ANGLE_DECIMALS$}")))
            }
        }
    }
}

#[cfg(test)]
#[path = "readout_tests.rs"]
mod tests;
