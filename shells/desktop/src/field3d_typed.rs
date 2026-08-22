//! ⭐ **O NÚMERO DIGITADO NO MEIO DO GESTO** — o `G X 0,5` do Blender, aqui ([ADR-0161], plano W6).
//!
//! # O que faltava, e porque é uma wave e não um campo de texto
//!
//! A ficha do gesto **mostra** o número desde a W8 (*"o número do gesto aparece ao lado do gizmo"*)
//! e nunca o **aceitou**: para pôr uma peça a 0,5 exactos era preciso largar a alça e ir procurar a
//! linha certa no painel. O painel continua a ser a porta para *"quanto ela mede"*; isto é a porta
//! para *"anda exactamente isto, agora"*, que é um gesto diferente e é o do modelador.
//!
//! # ⭐ A lei: o que se digita é o TOTAL, e é ele que a ficha já mostrava
//!
//! O arrasto deste módulo mede sempre **o total desde a pegada** (`Grip::applied`, W6), e o que vai
//! ao mundo é `total.since(applied)`. Um número digitado é **outro total** — a mesma álgebra, outra
//! fonte. É isso que faz digitar `0,5` depois de arrastar `0,37` mandar `0,13` ao mundo, sem uma
//! linha de caso especial e sem o gesto saltar.
//!
//! ⚠️ **As unidades são as da FICHA**, nunca outras: unidades de mundo numa seta, **graus** numa
//! argola, **fator** no punho. Um número que se digita e outro que se lê seria a segunda verdade
//! clássica, num sítio onde ela é invisível até alguém medir a peça.
//!
//! # ⚠️ Só onde um número tem UM significado
//!
//! Uma seta, uma argola e o punho aceitam; os **planos** e o **plano da tela** não ([`accepts`]) —
//! ali um número sozinho não diz para onde. Não é uma limitação disfarçada: é a mesma razão pela
//! qual o Blender pede um eixo antes do número.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use crate::field3d_gizmo::{Anchor, Handle, Motion};

/// O que uma tecla faz à entrada numérica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stroke {
    Digit(u8),
    /// A vírgula decimal — **o ponto e a vírgula são a mesma tecla** para quem escreve em português.
    Dot,
    /// ⚠️ **Troca o sinal**, não escreve um traço: é o que o Blender faz, e é o que permite corrigir
    /// a direção sem apagar o número que já se digitou.
    Sign,
    Backspace,
    /// Fecha o gesto guardando o que está.
    Commit,
    /// Desfaz o gesto INTEIRO — a peça volta a onde estava quando a alça foi agarrada.
    Cancel,
}

/// A tecla, traduzida. `None` quando não é deste teclado numérico.
pub(crate) fn stroke_for(code: winit::keyboard::KeyCode) -> Option<Stroke> {
    use winit::keyboard::KeyCode as K;
    let digit = |n: u8| Some(Stroke::Digit(n));
    match code {
        K::Digit0 | K::Numpad0 => digit(0),
        K::Digit1 | K::Numpad1 => digit(1),
        K::Digit2 | K::Numpad2 => digit(2),
        K::Digit3 | K::Numpad3 => digit(3),
        K::Digit4 | K::Numpad4 => digit(4),
        K::Digit5 | K::Numpad5 => digit(5),
        K::Digit6 | K::Numpad6 => digit(6),
        K::Digit7 | K::Numpad7 => digit(7),
        K::Digit8 | K::Numpad8 => digit(8),
        K::Digit9 | K::Numpad9 => digit(9),
        K::Period | K::NumpadDecimal | K::Comma => Some(Stroke::Dot),
        K::Minus | K::NumpadSubtract => Some(Stroke::Sign),
        K::Backspace => Some(Stroke::Backspace),
        K::Enter | K::NumpadEnter => Some(Stroke::Commit),
        K::Escape => Some(Stroke::Cancel),
        _ => None,
    }
}

/// O que a entrada passa a ser depois desta tecla. `None` = **a entrada acabou** e o rato volta a
/// mandar (é o que um `Backspace` sobre nada faz — a saída sem cancelar o gesto).
pub(crate) fn edit(text: &str, s: Stroke) -> Option<String> {
    let mut out = text.to_string();
    match s {
        Stroke::Digit(d) => out.push(char::from(b'0' + d)),
        // Um segundo ponto não é um número; ignorá-lo é o que todo campo numérico faz.
        Stroke::Dot => {
            if !out.contains('.') {
                if out.is_empty() || out == "-" {
                    out.push('0');
                }
                out.push('.');
            }
        }
        Stroke::Sign => {
            if let Some(rest) = out.strip_prefix('-') {
                out = rest.to_string();
            } else {
                out.insert(0, '-');
            }
        }
        Stroke::Backspace => {
            out.pop();
            // ⚠️ Apagar o último caractere **sai** da entrada em vez de ficar num campo vazio: com o
            // campo vazio e o rato mudo, o gesto ficaria preso sem nada na tela a dizer porquê.
            if out.is_empty() || out == "-" {
                return None;
            }
        }
        Stroke::Commit | Stroke::Cancel => return None,
    }
    Some(out)
}

/// O número, quando o texto já é um. `"-"` e `""` ainda não são — e é por isso que a entrada mostra
/// texto e aplica **valor**.
pub(crate) fn value_of(text: &str) -> Option<f32> {
    text.parse::<f32>().ok().filter(|v| v.is_finite())
}

/// ⭐ **Esta alça aceita um número?** Ver a nota do módulo: só onde ele tem um significado.
pub(crate) fn accepts(handle: Handle) -> bool {
    !matches!(handle, Handle::Plane(_) | Handle::View)
}

/// ⭐ **O TOTAL que este número pede**, na unidade da ficha.
///
/// `view_fwd` é a direção da vista — o eixo da argola que nunca fica de perfil.
pub(crate) fn total(
    handle: Handle,
    anchor: &Anchor,
    view_fwd: [f32; 3],
    value: f32,
) -> Option<Motion> {
    let along = |a: [f32; 3]| Motion::Translate([a[0] * value, a[1] * value, a[2] * value]);
    match handle {
        Handle::Axis(n) => Some(along(anchor.axes[n])),
        Handle::Ring(n) => Some(Motion::Rotate {
            axis: anchor.axes[n],
            angle: value.to_radians(),
        }),
        Handle::ViewRing => Some(Motion::Rotate {
            axis: view_fwd,
            angle: value.to_radians(),
        }),
        // ⚠️ Um fator **não-positivo** não é um tamanho: a peça não se vira do avesso por uma tecla,
        // e o `scale_by` do mundo recusa-o de qualquer forma. Devolver `None` deixa o texto na tela
        // (o artista continua a escrever) sem mandar nada ao mundo.
        Handle::Grip => (value > 0.0).then_some(Motion::Scale(value)),
        Handle::Plane(_) | Handle::View => None,
    }
}

/// ⭐ **A ficha, enquanto se digita** — na unidade da alça, com o texto **como ele está**.
///
/// ⚠️ Ela mostra o TEXTO e não o valor: enquanto se escreve `-0.` não há número nenhum, e uma ficha
/// que saltasse para `0,000` estaria a mentir sobre o que a tecla seguinte vai fazer.
pub(crate) fn label(handle: Handle, text: &str) -> String {
    match handle {
        Handle::Axis(n) => format!("{} {text}", ["X", "Y", "Z"][n.min(2)]),
        Handle::Ring(_) | Handle::ViewRing => format!("{text}°"),
        Handle::Grip => format!("x {text}"),
        Handle::Plane(_) | Handle::View => text.to_string(),
    }
}

#[cfg(test)]
#[path = "field3d_typed_tests.rs"]
mod tests;
