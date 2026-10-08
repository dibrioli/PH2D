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
    let j = (pos + 0.5).floor().clamp(0.0, (k - 1) as f32) as usize; // CLAMP-OK: k ≥ 1 (saída acima)
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
        paint_label(
            scene,
            text_system,
            theme,
            at,
            &child.label,
            fan.hot == Some(j),
        );
    }
}

#[cfg(test)]
mod tests {
    //! Os gates do **LEQUE do radial** (F4) — o filho que se desenha é o que se escolhe, uma família
    //! nunca executa, e nenhum rótulo tapa outro.

    use super::*;
    use crate::widget::radial_menu::label_rect;

    const C: [f32; 2] = [800.0, 600.0];

    fn leaf(i: u64) -> RadialItem {
        RadialItem::leaf(format!("L{i}"), NodeId(5000 + i))
    }

    /// Um anel de `n` sectores em que o sector `fam` é uma família de `k` filhos.
    fn ring(n: usize, fam: usize, k: usize) -> RadialOpen {
        let items = (0..n)
            .map(|i| {
                if i == fam {
                    RadialItem::family("F", (0..k as u64).map(|j| leaf(100 + j)).collect())
                } else {
                    leaf(i as u64)
                }
            })
            .collect();
        RadialOpen {
            center: C,
            items,
            hot: None,
            fan: None,
        }
    }

    /// Onde o filho `j` assenta, no ecrã.
    fn child_at(open: &RadialOpen, fam: usize, j: usize, k: usize) -> [f32; 2] {
        let fc = fan_center(open, fam);
        let o = fan_offset(j, k, outward(fam, open.items.len()));
        [fc[0] + o[0], fc[1] + o[1]]
    }

    /// ⭐ **O FILHO QUE SE DESENHA É O QUE SE ESCOLHE** — em toda contagem do anel, toda família, todo
    /// tamanho de leque, e com o ponteiro a CHEGAR pelo caminho (do centro ao pai, do pai ao filho).
    ///
    /// ⚠️ O caminho conta: os filhos das pontas assentam além da fronteira do sector vizinho, e um leque
    /// que fugisse para o vizinho deixá-los-ia inalcançáveis — o gate atravessa essa fronteira.
    #[test]
    fn every_child_is_chosen_where_it_is_drawn() {
        for n in 2..=MAX_SECTORS {
            for fam in 0..n {
                for k in 1..=FAN_MAX {
                    for j in 0..k {
                        let mut open = ring(n, fam, k);
                        let fc0 = fan_center(&open, fam);
                        point(&mut open, fc0);
                        let target = child_at(&open, fam, j, k);
                        for t in 1..=8u8 {
                            let f = f32::from(t) / 8.0;
                            let fc = fan_center(&open, fam);
                            let p = [
                                fc[0] + (target[0] - fc[0]) * f,
                                fc[1] + (target[1] - fc[1]) * f,
                            ];
                            point(&mut open, p);
                        }
                        let got = chosen(&open).map(|c| c.id);
                        assert_eq!(
                            got,
                            Some(NodeId(5100 + j as u64)),
                            "n={n} família={fam} k={k}: o filho {j} desenhado ali não é o escolhido"
                        );
                    }
                }
            }
        }
    }

    /// ⛔ **UMA FAMÍLIA NUNCA EXECUTA** — soltar sobre ela antes de o leque abrir, ou na zona morta
    /// do leque, cancela. ⚠️ Com o CONTROLO: a folha vizinha, no mesmo raio, escolhe-se.
    #[test]
    fn a_family_is_never_a_command() {
        let mut open = ring(7, 2, 4);
        let o = item_offset(2, 7);
        point(&mut open, [C[0] + o[0] * 0.6, C[1] + o[1] * 0.6]);
        assert_eq!(open.hot, Some(2), "a família não acendeu");
        assert_eq!(
            chosen(&open),
            None,
            "soltar na família executou alguma coisa"
        );
        let fc = fan_center(&open, 2);
        point(&mut open, fc);
        assert!(open.fan.is_some(), "atravessar o anel não abriu o leque");
        assert_eq!(
            chosen(&open),
            None,
            "a zona morta do leque executou alguma coisa"
        );
        let o = item_offset(1, 7);
        point(&mut open, [C[0] + o[0] * 0.6, C[1] + o[1] * 0.6]);
        assert_eq!(
            chosen(&open).map(|c| c.id),
            Some(NodeId(5001)),
            "o controlo"
        );
    }

    /// **VOLTAR PARA DENTRO DO ANEL FECHA O LEQUE** — é o gesto de desistir da família.
    #[test]
    fn coming_back_inside_the_ring_closes_the_fan() {
        let mut open = ring(7, 3, 3);
        let t = child_at(&open, 3, 0, 3);
        let fc = fan_center(&open, 3);
        point(&mut open, fc);
        point(&mut open, t);
        assert!(open.fan.is_some_and(|f| f.hot == Some(0)));
        point(&mut open, C);
        assert_eq!(
            open.fan, None,
            "o leque ficou aberto com o ponteiro no centro"
        );
        assert_eq!(chosen(&open), None, "o centro escolheu um filho");
    }

    /// ⛔ **NENHUM RÓTULO DO LEQUE TAPA OUTRO, NEM O DO PAI** — em toda direcção de pai e todo tamanho.
    ///
    /// ⚠️ É o gate do raio derivado: no raio do anel (96 px) os dois filhos do meio de um leque virado
    /// para cima ficam lado a lado a 73 px com 92 de rótulo.
    #[test]
    fn no_fan_label_covers_another() {
        let hit = |a: crate::zones::Rect, b: crate::zones::Rect| {
            a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
        };
        for n in 2..=MAX_SECTORS {
            for fam in 0..n {
                for k in 1..=FAN_MAX {
                    let open = ring(n, fam, k);
                    let mut rects = vec![label_rect(fan_center(&open, fam))];
                    rects.extend((0..k).map(|j| label_rect(child_at(&open, fam, j, k))));
                    for a in 0..rects.len() {
                        for b in a + 1..rects.len() {
                            assert!(
                                !hit(rects[a], rects[b]),
                                "n={n} família={fam} k={k}: o rótulo {a} tapa o {b}"
                            );
                        }
                    }
                }
            }
        }
    }
}
