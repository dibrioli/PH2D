//! ⭐⭐⭐ **QUANDO HÁ MAIS ABAS DO QUE CABEM** — a última metade do desenho das abas.
//!
//! # O alcance, medido antes de construir
//!
//! O piso de uma aba é um quadrado de [`ph2d_tokens::ROW_H_PX`], então a coluna da direita de
//! fábrica (296 px úteis) pinta **13** abas e é preciso um encaixe com 14+ ocupantes para esconder
//! alguma. ⭐ **Mas a coluna estreita-se:** no degrau do fecho (190 px úteis) cabem **8** — logo
//! `9+` painéis do mesmo lado numa coluna apertada já escondem abas.
//!
//! ⚠️ **Não é um beco sem saída, e é por isso que isto veio por último:** o painel escondido
//! continua alcançável pelo menu *Window*, e levantá-lo desliza a janela até ele. O que faltava
//! era a saída **na própria fila**.
//!
//! # ⛔ Por que as setas escolhem um PAINEL, e não «rolam» a fila
//!
//! A janela não é estado: ela é **derivada** de quem está à frente ([`super::slot_tabs::tab_plan`]
//! desliza até o escolhido caber). Uma seta que rolasse a fila teria de guardar um deslocamento
//! por encaixe — uma segunda resposta a *«que abas se vêem?»*, e as duas divergiriam no primeiro
//! clique que uma delas não visse. ⇒ **a seta escolhe o ocupante seguinte**, e a janela segue-o
//! sozinha. *É também o verbo que um teclado quereria: «aba seguinte».*
//!
//! ⚠️ **Elas só existem quando a fila transborda** — com tudo à vista não há para onde ir, e duas
//! setas mudas seriam dois controlos mortos (a espécie que esta casa varre a cada wave).

use super::HeroScreen;
use super::slot_tabs::{TAB_BAR_H, chosen, occupants};
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{paint_icon, resolve};
use crate::screens::slot::Slot;
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// ⛔ **O salto que separa o id de uma SETA de tudo o resto.** Ver [`super::slot_tabs`], que usa a
/// mesma técnica para as abas: um XOR com uma constante é uma bijecção, logo não pode criar
/// colisão que o espaço de ids já não tivesse.
const ARROW_SALT: u64 = 0x7ab5_0000_a770_0001;

/// O lado da seta — o mesmo quadrado que é o piso de uma aba.
///
/// ⚠️ **Não é um número escolhido:** uma seta é um alvo da fila, e a fila mede [`TAB_BAR_H`].
pub fn arrow_w() -> f32 {
    TAB_BAR_H
}

/// `(anterior, seguinte)` — os dois ids de um encaixe.
#[must_use]
pub fn arrow_ids(slot: Slot) -> (NodeId, NodeId) {
    let base = ARROW_SALT ^ ((slot as u64 + 1) << 8);
    (NodeId(base), NodeId(base ^ 1))
}

/// O que este id pede, se for de uma seta: o encaixe e o passo (`-1` / `+1`).
#[must_use]
pub fn arrow_of(id: NodeId) -> Option<(Slot, i32)> {
    Slot::ALL.into_iter().find_map(|s| {
        let (p, n) = arrow_ids(s);
        if id == p {
            Some((s, -1))
        } else if id == n {
            Some((s, 1))
        } else {
            None
        }
    })
}

/// Os rects das duas setas, **encostadas ao fim da faixa** — ver [`super::slot_tabs::tab_plan`],
/// que reserva exactamente este espaço antes de dispor as abas.
///
/// ⚠️ **As duas ao mesmo lado, e não uma em cada ponta:** com uma à esquerda, a primeira aba
/// mudava de sítio no instante em que a fila passasse a transbordar. *O que transborda é o fim da
/// fila; o princípio dela não se mexe.*
#[must_use]
pub fn arrow_rects(bar: Rect) -> (Rect, Rect) {
    let w = arrow_w();
    let right = bar.x + bar.w;
    (
        Rect::new(right - w * 2.0, bar.y, w, bar.h),
        Rect::new(right - w, bar.y, w, bar.h),
    )
}

/// Regista os dois ids de cada encaixe. Chamado pelo [`super::slot_tabs::populate`].
pub fn populate(store: &mut WidgetStore) {
    for slot in Slot::ALL {
        let (p, n) = arrow_ids(slot);
        for id in [p, n] {
            store.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
}

/// Pinta as duas setas e regista os alvos. Não faz nada quando a fila **não** transborda.
pub fn paint(
    bar: Rect,
    slot: Slot,
    hidden_before: usize,
    hidden_after: usize,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    if hidden_before + hidden_after == 0 || bar.w <= arrow_w() * 2.0 {
        return;
    }
    let (prev, next) = arrow_rects(bar);
    let (pid, nid) = arrow_ids(slot);
    // ⚠️ **A seta que não tem para onde ir fica APAGADA, e continua clicável a zero:** ela não é
    //    registada no índice de acerto, então o clique cai na faixa por baixo. *Um controlo que
    //    não pode agir não deve poder ser tocado.*
    for (rect, id, live, icon) in [
        (
            prev,
            pid,
            hidden_before > 0,
            crate::icons::IconId::ChevronLeft,
        ),
        (
            next,
            nid,
            hidden_after > 0,
            crate::icons::IconId::ChevronRight,
        ),
    ] {
        let token = if live {
            ColorToken::Text2
        } else {
            ColorToken::Text3
        };
        // ⚠️ **O glifo tem o tamanho dos OUTROS glifos da fila** (`INLINE_ICON_PX`), centrado no
        //    quadrado do alvo. O alvo é a linha inteira; o desenho é do tamanho de um ícone —
        //    pintar o chevron no rect todo far-lo-ia maior que os ícones das abas ao lado.
        let g = ph2d_tokens::INLINE_ICON_PX;
        let glyph = Rect::new(
            rect.x + (rect.w - g) * 0.5,
            rect.y + (rect.h - g) * 0.5,
            g,
            g,
        );
        paint_icon(
            scene,
            icon,
            glyph,
            resolve(token, theme),
            StrokeToken::Default.px(),
        );
        if live {
            hit_index.register(id, rect);
        }
    }
}

/// ⭐ **O clique numa seta levanta o ocupante seguinte** — e a janela segue-o.
///
/// Devolve `true` se o id era de uma seta (mesmo quando não havia para onde ir: o controlo é
/// dele, e deixar o evento seguir daria a outro handler um clique que não é seu).
pub fn apply_event(hero: &mut HeroScreen, id: NodeId) -> bool {
    let Some((slot, step)) = arrow_of(id) else {
        return false;
    };
    let occ = occupants(hero, slot);
    if occ.is_empty() {
        return true;
    }
    let here = chosen(hero, slot)
        .and_then(|c| occ.iter().position(|o| o.node == c))
        .unwrap_or(0) as i32;
    // CLAMP-OK: os dois limites são INTEIROS (nunca NaN) e o teto não pode ser negativo — o
    // `occ.is_empty()` acima devolveu cedo, logo `len >= 1`. Um clamp e não um wrap: a fila tem
    // pontas, e chegar ao fim não é dar a volta.
    let want = (here + step).clamp(0, occ.len() as i32 - 1) as usize; // CLAMP-OK: inteiros, e `len >= 1`
    hero.store.bump_panel_z(occ[want].node);
    true
}
