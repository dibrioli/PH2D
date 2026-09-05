//! ⭐⭐ **A VOZ DO ARRASTO** — o fantasma que segue o cursor (plano `docs/Components/07`, B4).
//!
//! # Ele é o primeiro do app, e a ausência tinha nome
//!
//! Medido em 2026-08-30: **nada neste editor segue o cursor como coisa carregada.** O indicador do
//! reparentar é ancorado na LINHA; o menu radial fixa o centro de propósito (*«ele não se
//! move»*); e o aviso de largar ficheiro do sistema **recebe o cursor e deita-o fora** — a
//! assinatura dele é literalmente `_cursor_px`, e o cartão desenha-se no meio do ecrã.
//!
//! ⇒ um arrasto que sai de um painel precisa de dizer **o que vai na mão**, senão o gesto é uma
//! aposta: o artista solta e descobre.
//!
//! # ⚠️ Onde ele é pintado, e por quê
//!
//! No fim do `paint_hero_screen`, **depois** do aviso de largar ficheiro — a cena do chrome é a
//! última a ser composta, então o que se desenha aqui fica por cima de tudo, inclusive dos painéis
//! e da tela. Um fantasma por baixo de um painel seria pior que nenhum.
//!
//! ⛔ **Ele não regista rectângulo nenhum** no `HitIndex`: uma coisa que segue o cursor não pode
//! ser clicável — ela taparia sempre o alvo que a mão está a procurar.

use crate::interaction::drag_payload::{DragVerdict, InFlightDrag};
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Deslocamento do cartão em relação ao cursor.
///
/// ⚠️ **Positivo nos dois eixos, de propósito**: o fantasma fica **abaixo e à direita** da ponta do
/// cursor, que é onde a seta não tapa. Centrá-lo no cursor esconderia precisamente o pixel que o
/// artista está a mirar.
const GHOST_OFFSET_PX: f32 = 14.0; // LITERAL-PX-OK: geometria do fantasma, não medida de design

/// O que o fantasma diz quando o sítio debaixo do cursor não sabe receber.
///
/// ⚠️ **Uma PALAVRA, e não só uma cor** — uma diferença que só existe em cor não chega a quem não
/// distingue vermelho de verde. ⏳ Migra com os irmãos quando o Fluent chegar (HR-15).
const REFUSED_LABEL: &str = "Won't take this";

/// Pinta o que vai na mão. No-op se não há arrasto, ou se ele **ainda não armou** (o gesto pode
/// acabar por ser um clique, e um fantasma que pisca a cada toque é ruído).
pub(super) fn paint_asset_drag_ghost(
    drag: Option<InFlightDrag>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let Some(drag) = drag.filter(|d| d.armed) else {
        return;
    };
    // ⭐⭐⭐ **O fantasma diz se o sítio ACEITA** (wave B4) — e não só o que vai na mão.
    //
    // ⛔ Até 2026-09-01 ele mostrava o mesmo cartão sobre a tela, sobre um campo que recebe e
    // sobre a barra de cima que não. A recusa existia **depois do facto**, num toast. *Largar num
    // sítio errado tem de se ver ANTES, senão o arrasto é uma aposta.*
    //
    // ⚠️ **A palavra muda com a moldura**, e não só a cor: uma diferença que só existe em cor não
    // chega a quem não distingue vermelho de verde.
    let refused = drag.verdict == DragVerdict::Refuse;
    let label = if refused {
        REFUSED_LABEL
    } else {
        drag.payload.kind_label()
    };
    let font = TypeToken::Sm.px();
    let pad = Spacing::Sm.px();
    let w = text_system.layout(label, font, f32::INFINITY).width() + pad * 2.0;
    let h = font + pad * 2.0;
    let rect = Rect::new(
        drag.cursor.0 + GHOST_OFFSET_PX,
        drag.cursor.1 + GHOST_OFFSET_PX,
        w,
        h,
    );
    // ⭐ Raio e moldura pela porta do TEMA. A RECUSA é `Feel::Error` — a única moldura que a
    //    família moderna não apaga, porque sem ela o fantasma recusado lê-se como aceite; o
    //    aceite é um fantasma plano com o rótulo (a pré-visualização de arrasto do Godot).
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        if refused {
            ph2d_tokens::visuals::Feel::Error
        } else {
            ph2d_tokens::visuals::Feel::Active
        },
        StrokeToken::Default.px(),
        resolve(
            if refused {
                ColorToken::Danger
            } else {
                ColorToken::Accent
            },
            theme,
        ),
    );
    crate::paint::paint_text(
        text_system,
        scene,
        label,
        rect.x + pad,
        rect.y + pad,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
}
