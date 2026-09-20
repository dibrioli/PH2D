//! ⭐⭐ **AS PALAVRAS DE QUATRO MOTORES** que a população larga da fronteira trouxe — a COR, a
//! CURVA, o COMANDO e as RECUSAS DA TIMELINE.
//!
//! # Porque nenhuma régua as via, e porque elas são quatro e não uma
//!
//! As quatro crates são MOTORES: nenhuma das 30 réguas lexicais as varre, e a régua de porta segue
//! o texto até um pintor **dentro da mesma crate** — e aqui os pintores vivem noutras. Elas estão
//! juntas neste ficheiro por serem a mesma FATIA e não o mesmo assunto; se uma delas crescer, corta-se
//! pelo assunto, que é a lei dos irmãos de tabela deste repo.
//!
//! # ⚠️ O que NÃO se traduz, e está escrito ao lado de cada uma
//!
//! - **`RGB` · `HSV` · `HSL` · `CW` · `CCW`** são SIGLAS e ficam assim em toda língua, como o `SFX`
//!   do barramento de áudio. A chave existe para a frase que as rodeia poder mudar.
//! - **A extensão entre parênteses** de um formato de paleta (`.gpl`, `.aco`) é o que o artista
//!   procura no disco — a palavra à frente é que é rótulo.
//! - **`A / Cross` e `L1 / LB`** nomeiam o mesmo BOTÃO FÍSICO nas duas famílias de comando, porque
//!   um artista que só tem um dos dois não reconhece o nome do outro. A chave não existe para
//!   traduzir a letra: existe para a ordem e a barra ficarem numa lista só.
//!
//! # ⭐ E as cinco recusas da timeline mantêm a lei que as escreveu
//!
//! *Cada uma diz o que aconteceu E o que a pilha está a fazer, porque «can't key here» sem razão é
//! só um pouco melhor que silêncio.* O texto mudou de sítio, não de intenção.

/// A tradução de uma chave `color.*` / `anim.easing.*` / `input.pad.*` / `timeline.{nest,key}.*`,
/// ou `None` se ela não é daqui.
#[allow(clippy::too_many_lines)]
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        // ── a COR: o espaço da rampa, a interpolação, o caminho do matiz ──────────────────
        "color.ramp.space.rgb" => "RGB",
        "color.ramp.space.hsv" => "HSV",
        "color.ramp.space.hsl" => "HSL",
        "color.ramp.interp.ease" => "Ease",
        "color.ramp.interp.cardinal" => "Cardinal",
        "color.ramp.interp.linear" => "Linear",
        "color.ramp.interp.b_spline" => "B-Spline",
        "color.ramp.interp.constant" => "Constant",
        "color.ramp.hue.near" => "Near",
        "color.ramp.hue.far" => "Far",
        "color.ramp.hue.cw" => "CW",
        "color.ramp.hue.ccw" => "CCW",
        // os quatro gradientes de fábrica
        "color.gradient.rainbow" => "Rainbow",
        "color.gradient.heat" => "Heat",
        "color.gradient.ice" => "Ice",
        "color.gradient.grayscale" => "Grayscale",
        // os quatro formatos de paleta que o app lê e escreve
        "color.palette.format.gpl" => "GIMP palette (.gpl)",
        "color.palette.format.hex" => "Hex list (.hex)",
        "color.palette.format.ase" => "Adobe Swatch Exchange (.ase)",
        "color.palette.format.aco" => "Adobe Color (.aco)",
        // ── a CURVA: as onze famílias de easing e os três modos ───────────────────────────
        "anim.easing.family.linear" => "Linear",
        "anim.easing.family.quad" => "Quad",
        "anim.easing.family.cubic" => "Cubic",
        "anim.easing.family.quart" => "Quart",
        "anim.easing.family.quint" => "Quint",
        "anim.easing.family.back" => "Back",
        "anim.easing.family.bounce" => "Bounce",
        "anim.easing.family.sine" => "Sine",
        "anim.easing.family.expo" => "Expo",
        "anim.easing.family.circ" => "Circ",
        "anim.easing.family.elastic" => "Elastic",
        // ⚠️ Curtos de propósito: quem os pinta é uma fileira de três chips AO LADO do nome da
        // família, e ali *"Ease In"* repetiria a palavra que a linha inteira já diz.
        "anim.easing.mode.in" => "In",
        "anim.easing.mode.out" => "Out",
        "anim.easing.mode.in_out" => "In-Out",
        // ── o COMANDO: os dezassete botões e os seis eixos ────────────────────────────────
        "input.pad.button.south" => "A / Cross",
        "input.pad.button.east" => "B / Circle",
        "input.pad.button.west" => "X / Square",
        "input.pad.button.north" => "Y / Triangle",
        "input.pad.button.left_bumper" => "L1 / LB",
        "input.pad.button.right_bumper" => "R1 / RB",
        "input.pad.button.left_trigger" => "L2 / LT",
        "input.pad.button.right_trigger" => "R2 / RT",
        "input.pad.button.select" => "Select",
        "input.pad.button.start" => "Start",
        "input.pad.button.mode" => "Home",
        "input.pad.button.left_stick" => "Left Stick Press",
        "input.pad.button.right_stick" => "Right Stick Press",
        "input.pad.button.dpad_up" => "D-Pad Up",
        "input.pad.button.dpad_down" => "D-Pad Down",
        "input.pad.button.dpad_left" => "D-Pad Left",
        "input.pad.button.dpad_right" => "D-Pad Right",
        "input.pad.axis.left_stick_x" => "Left Stick X",
        "input.pad.axis.left_stick_y" => "Left Stick Y",
        "input.pad.axis.right_stick_x" => "Right Stick X",
        "input.pad.axis.right_stick_y" => "Right Stick Y",
        "input.pad.axis.left_trigger" => "Left Trigger",
        "input.pad.axis.right_trigger" => "Right Trigger",
        // ── as RECUSAS da timeline: aninhar e pousar uma chave ────────────────────────────
        "timeline.nest.self" => "Can't nest: a container cannot contain itself",
        "timeline.nest.cycle" => "Can't nest: that container already contains this one",
        "timeline.nest.missing" => "Can't nest: that container no longer exists",
        "timeline.key.not_playing" => "Can't key: the clip you are editing does not play here",
        "timeline.key.plays_twice" => "Can't key: the clip you are editing plays twice here",
        "timeline.key.overridden" => "Can't key: a lane above overrides this clip here",
        "timeline.key.expression_driven" => {
            "Can't key: an expression drives this channel \u{2014} clean or rewrite the formula"
        }
        "timeline.key.path_needs_keys_tab" => {
            "Can't key the path here: a trajectory belongs to its clip \u{2014} switch to the Keys tab"
        }
        // ⚠️ A sexta recusa chegou pela `line/Vector` no mesmo dia em que esta tabela nasceu, e
        //    ela escreveu a FRASE onde as irmãs já escreviam a CHAVE — o `message_key` devolvia um
        //    literal. *Um merge textual funde as duas metades limpo: nenhum dos dois lados contém
        //    as duas coisas.*
        "timeline.key.bone_handles_from_chain" => {
            "Can't key the bend: this bone's handles come from the chain \u{2014} set Curve Handles to Manual"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
