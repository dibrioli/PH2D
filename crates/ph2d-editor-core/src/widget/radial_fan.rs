//! **O LEQUE do radial** (F4, D10) — um sector que é uma FAMÍLIA abre as ferramentas dela quando o
//! ponteiro atravessa o anel; soltar num filho escolhe-o. É o menu de marcação hierárquico
//! (medição e decisão: handoff de 08/10 da `line/UIUX`, §F4).
//!
//! ⚠️ **Uma família NÃO é um comando** ([`FAMILY_ID`]): soltar sobre ela, antes de o leque abrir,
//! cancela — a mesma lei da zona morta. Só as folhas executam.
//!
//! ⚠️ **Os filhos voltam-se para FORA**, em passos de `TAU / MAX_SECTORS` (as direcções da bússola
//! que a mão reproduz sem olhar): voltar para o centro é o gesto de desistir da família, não o de
//! escolher um filho.

use super::radial_menu::{
    MAX_SECTORS, RadialItem, RadialOpen, dead_zone_px, item_offset, label_rect, paint_label,
    radius_px,
};
use ph2d_a11y::NodeId;
use std::f32::consts::{PI, TAU};

/// O id de uma família — nunca um comando, nunca devolvido a quem executa.
pub const FAMILY_ID: NodeId = NodeId(0x0052_4144_4641_4D21);

/// Quantos filhos cabem num leque: a meia volta virada para fora, em passos de bússola.
pub const FAN_MAX: usize = MAX_SECTORS / 2 + 1;

/// O leque aberto: a família pai (índice no anel) e o filho aceso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadialFan {
    pub parent: usize,
    pub hot: Option<usize>,
}

#[allow(clippy::cast_precision_loss)]
fn step() -> f32 {
    TAU / MAX_SECTORS as f32
}

/// A direcção do item `i` de `count` — a de [`item_offset`] (0 = cima, horário).
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn outward(i: usize, count: usize) -> f32 {
    TAU * i as f32 / count.max(1) as f32
}

/// O desvio do filho `j` de `k` em relação à direcção do pai.
#[allow(clippy::cast_precision_loss)]
fn offset(j: usize, k: usize) -> f32 {
    (j as f32 - (k.saturating_sub(1)) as f32 * 0.5) * step()
}

/// O centro do leque da família `parent`: onde o rótulo dela assenta.
#[must_use]
pub fn fan_center(open: &RadialOpen, parent: usize) -> [f32; 2] {
    let o = item_offset(parent, open.items.len());
    [open.center[0] + o[0], open.center[1] + o[1]]
}

/// O raio do leque: o MENOR em que dois filhos vizinhos não se tocam em orientação nenhuma — a
/// corda de um passo (`2 r sen(passo/2)`) igual à DIAGONAL do rótulo (a corda inclinada tem de
/// vencer a largura OU a altura, e o pior ângulo pede as duas). ⚠️ O raio do anel (96) não serve:
/// no leque virado para cima os dois filhos do meio ficam lado a lado a 73 px, com 92 de rótulo.
#[must_use]
pub fn fan_radius_px() -> f32 {
    let r = label_rect([0.0, 0.0]);
    r.w.hypot(r.h) / (2.0 * (step() * 0.5).sin())
}

/// Onde o filho `j` de `k` assenta, em px de tela relativos ao centro do leque.
#[must_use]
pub fn fan_offset(j: usize, k: usize, out: f32) -> [f32; 2] {
    let a = out + offset(j, k);
    [fan_radius_px() * a.sin(), -fan_radius_px() * a.cos()]
}

/// **QUE FILHO ESTÁ SOB O PONTEIRO** — a lei pura do leque. `None` na zona morta do centro do
/// leque, ou a mais de um passo do filho mais próximo (o ponteiro a voltar para trás).
#[must_use]
pub fn fan_child_at(fc: [f32; 2], out: f32, pointer: [f32; 2], k: usize) -> Option<usize> {
    if k == 0 {
        return None;
    }
    let (dx, dy) = (pointer[0] - fc[0], pointer[1] - fc[1]);
    if dx.hypot(dy) < dead_zone_px() {
        return None;
    }
    let mut rel = dx.atan2(-dy) - out;
    while rel > PI {
        rel -= TAU;
    }
    while rel <= -PI {
        rel += TAU;
    }
    #[allow(clippy::cast_precision_loss)]
    let pos = rel / step() + (k - 1) as f32 * 0.5;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let j = (pos + 0.5).floor().clamp(0.0, (k - 1) as f32) as usize;
    ((rel - offset(j, k)).abs() <= step()).then_some(j)
}

/// **O PONTEIRO MEXEU-SE** — acende o sector, abre ou fecha o leque, acende o filho.
///
/// ⚠️ O leque fica aberto enquanto o ponteiro estiver FORA do anel, mesmo que atravesse a
/// fronteira de outro sector do anel: os filhos das pontas assentam além dela, e o leque a fugir
/// para o vizinho tornaria o filho da ponta inalcançável. Voltar para dentro do anel fecha-o.
pub fn point(open: &mut RadialOpen, pointer: [f32; 2]) {
    let n = open.items.len();
    let far = (pointer[0] - open.center[0]).hypot(pointer[1] - open.center[1]) >= radius_px();
    if let Some(fan) = open.fan
        && far
    {
        let k = open.items.get(fan.parent).map_or(0, |p| p.children.len());
        let hot = fan_child_at(
            fan_center(open, fan.parent),
            outward(fan.parent, n),
            pointer,
            k,
        );
        open.fan = Some(RadialFan { hot, ..fan });
        return;
    }
    open.fan = None;
    open.hot = super::radial_menu::sector_at(open.center, pointer, n);
    if let Some(i) = open.hot
        && far
        && open.items.get(i).is_some_and(|it| !it.children.is_empty())
    {
        let k = open.items[i].children.len();
        let hot = fan_child_at(fan_center(open, i), outward(i, n), pointer, k);
        open.fan = Some(RadialFan { parent: i, hot });
    }
}

/// **O QUE A SOLTURA ESCOLHE** — uma folha do anel, ou o filho aceso do leque. Nunca uma família.
#[must_use]
pub fn chosen(open: &RadialOpen) -> Option<RadialItem> {
    let item = open.items.get(open.hot?)?;
    if item.children.is_empty() {
        return Some(item.clone());
    }
    let fan = open.fan.filter(|f| Some(f.parent) == open.hot)?;
    item.children.get(fan.hot?).cloned()
}

/// Pinta os filhos do leque aberto, pela porta do rótulo do anel.
pub fn paint_fan(
    open: &RadialOpen,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
) {
    let Some(fan) = open.fan else { return };
    let Some(parent) = open.items.get(fan.parent) else {
        return;
    };
    let (fc, out, k) = (
        fan_center(open, fan.parent),
        outward(fan.parent, open.items.len()),
        parent.children.len(),
    );
    for (j, child) in parent.children.iter().enumerate() {
        let o = fan_offset(j, k, out);
        let at = [fc[0] + o[0], fc[1] + o[1]];
        paint_label(scene, text_system, theme, at, &child.label, fan.hot == Some(j));
    }
}

#[cfg(test)]
#[path = "radial_fan_tests.rs"]
mod tests;
