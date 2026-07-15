//! As **células** — a tira propriamente dita.
//!
//! Cada célula é uma chave, e a sua LARGURA é a exposição (à TVPaint): o tempo é
//! visível como espaço. A tira **sempre cabe** (a escala é derivada do vão), então
//! nunca há barra de rolagem nem estado de pan escondido — zoom/pan é follow-up.
//!
//! A célula é um **botão canônico** (`paint_button`), não um retângulo improvisado:
//! ela é clicável, tem estados (hover/press) e semântica de a11y — e o rótulo que
//! ela carrega é o número que o animador lê, a EXPOSIÇÃO ("esse desenho fica 2
//! quadros"). A chave ATIVA vira o botão `Accent`; um breakdown (inbetween do
//! tween) fica em `Default` (discreto). Um desenho INSTANCIADO (a mesma arte em
//! várias chaves) ganha um ponto. O playhead é uma linha vertical no quadro
//! corrente — inclusive no meio de um hold, que é onde ele costuma estar.

use crate::ids;
use crate::state::FlipStripSnapshot;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, token_to_vello,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Color as TokenColor, ColorToken, Radius, Spacing, Theme, TypeToken};

/// Largura máxima de um quadro em px (senão 3 chaves viram 3 painéis gigantes).
const MAX_PX_PER_FRAME: f32 = 26.0; // LITERAL-PX-OK: strip cell scale cap
/// Largura mínima visível de uma célula.
const MIN_CELL_W: f32 = 3.0; // LITERAL-PX-OK: strip minimum cell width
/// Lado do marcador de desenho instanciado.
const INSTANCE_DOT: f32 = 4.0; // LITERAL-PX-OK: instanced-drawing marker
/// Espessura da linha do playhead.
const PLAYHEAD_W: f32 = 2.0; // LITERAL-PX-OK: playhead line width
/// A célula só ganha rótulo se couber ~um glifo e meio (senão o número sai cortado).
const LABEL_FIT: f32 = 1.6; // LITERAL-PX-OK: fator de caber-o-glifo, não medida de design

/// **Altura da régua de scrub** (o pega-mão do playhead), em px de tela. Um Spacing token
/// (medida de design, não parâmetro geométrico) — generosa o bastante para ser um alvo de
/// arrasto confortável (Enio pediu *"mais espaço da tira de tempo"*).
fn scrub_lane_h() -> f32 {
    Spacing::Xl.px()
}

/// **A faixa reservada ACIMA das células** para a régua = a régua + o respiro até elas. O
/// painel CRESCE por ela (`paint::paint`), então os frames mantêm a altura de sempre — a
/// régua não os espreme (Enio 2026-07-14: *"ficou apertado na vertical"*). É a MESMA função
/// que `paint_cells` usa para posicionar o topo das células, então o crescimento do painel e
/// o recuo das células nunca divergem.
pub(crate) fn scrub_reserved_h() -> f32 {
    scrub_lane_h() + Spacing::Xs.px()
}

pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, area: Rect, snap: &FlipStripSnapshot) {
    if area.w <= 0.0 || area.h <= 0.0 {
        return;
    }
    // Trilho de fundo (a "pista" onde as células vivem).
    fill_rounded_rect(
        ctx.scene,
        area,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );

    if snap.cells.is_empty() {
        let font = TypeToken::Sm.px();
        let hint = if snap.has_layer {
            "No keys yet — draw on the canvas to create the first one."
        } else {
            "No Flip layer — add one in the Flip panel."
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            hint,
            area.x + Spacing::Sm.px(),
            area.y + (area.h - font) * 0.5,
            font,
            area.w,
            resolve(ColorToken::Text2, theme),
        );
        return;
    }

    // O vão: do primeiro quadro exposto ao fim da última exposição. A escala é
    // derivada dele, então a tira SEMPRE cabe na faixa.
    let first = snap.cells[0].key;
    let last = &snap.cells[snap.cells.len() - 1];
    let end = last.key.saturating_add(last.exposure.max(1) as i32);
    let total = (end - first).max(1) as f32;
    let inner = Rect::new(
        area.x + Spacing::Xs.px(),
        area.y + Spacing::Xs.px(),
        (area.w - Spacing::Xs.px() * 2.0).max(1.0),
        (area.h - Spacing::Xs.px() * 2.0).max(1.0),
    );
    let ppf = (inner.w / total).min(MAX_PX_PER_FRAME);
    let font = TypeToken::Sm.px();

    // **A régua de scrub** (W7.3): uma faixa no TOPO da tira que move o playhead sem
    // tocar na seleção (a separação-padrão de toda ferramenta de animação — a régua faz
    // scrub, as células selecionam). Ela cobre exatamente a largura ÚTIL das células
    // (`total·ppf`, que pode ser menor que `inner.w` quando o `ppf` bate no teto), então
    // o handle e as células ficam alinhados 1:1. As células descem uma faixa.
    let used_w = (total * ppf).max(MIN_CELL_W);
    let lane = Rect::new(inner.x, inner.y, used_w, scrub_lane_h());
    paint_scrub_lane(ctx, theme, lane, snap, inner.x, first, ppf);
    // As células começam DEPOIS da faixa reservada. O painel já cresceu por
    // `scrub_reserved_h()` (ver `paint.rs`), então elas mantêm a altura de sempre — a régua
    // não as espreme (Enio 2026-07-14: *"ficou apertado na vertical"*).
    let cells_top = inner.y + scrub_reserved_h();
    let cell_h = (inner.y + inner.h - cells_top).max(1.0);

    for (i, cell) in snap.cells.iter().enumerate() {
        let x = inner.x + (cell.key - first) as f32 * ppf;
        let w = (cell.exposure.max(1) as f32 * ppf - 1.0).max(MIN_CELL_W);
        let r = Rect::new(x, cells_top, w, cell_h);
        let id = ids::flip_cell_id(i);
        ctx.host.store_mut().register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        // O rótulo só entra quando cabe (numa tira longa a célula é uma lasca).
        let label = if w > font * LABEL_FIT {
            format!("{}", cell.exposure)
        } else {
            String::new()
        };
        // **Uma célula marcada para o MULTIFRAME veste a cor de ACENTO** (W7, Enio 2026-07-15).
        // A marcada é pintada à mão (fundo clareado pelo falloff + número por CIMA) para o
        // número NÃO ser lavado pelo clareamento ("apagou o número"); as demais são o botão
        // canônico (`Default`, ou `Accent` para a chave que está NA TELA — sempre um alvo).
        if cell.selected {
            paint_marked_cell(ctx, theme, r, &label, cell.weight);
        } else {
            let kind = if snap.current_key == Some(cell.key) {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            };
            paint_button(
                &Button::new(id, label).state(st).kind(kind),
                r,
                ctx.scene,
                ctx.text_system,
                theme,
            );
        }
        // Desenho instanciado (a MESMA arte em várias chaves): um ponto discreto.
        if cell.instanced && w > INSTANCE_DOT * 2.0 {
            let d = Rect::new(
                r.x + Spacing::Xs.px(),
                r.y + Spacing::Xs.px(),
                INSTANCE_DOT,
                INSTANCE_DOT,
            );
            fill_rounded_rect(
                ctx.scene,
                d,
                Radius::Sm.px(),
                resolve(ColorToken::Text2, theme),
            );
        }
        ctx.host.hit_index_mut().register(id, r);
    }

    // O playhead — uma linha no quadro corrente (que costuma estar NO MEIO de uma
    // exposição; é justamente aí que ele precisa ser visível).
    let px = inner.x + (snap.current_frame - first) as f32 * ppf;
    if px >= inner.x - 1.0 && px <= inner.x + inner.w + 1.0 {
        let line = Rect::new(px.max(inner.x), area.y, PLAYHEAD_W, area.h);
        fill_rounded_rect(
            ctx.scene,
            line,
            Radius::Sm.px(),
            resolve(ColorToken::BorderEmph, theme),
        );
    }
}

/// **A régua de scrub** (W7.3): um rail fino sobre a largura útil das células + um handle
/// de playhead arrastável no quadro corrente. O rail INTEIRO é a zona de arrasto (clicar
/// em qualquer ponto leva o playhead até lá), então registramos o rect no hit index sob
/// [`ids::FLIP_SCRUB`] — o Slider registrado no `populate` dá a mecânica de arrasto (1D
/// per-Move), e o shell faz o seek SEM tocar na seleção. O handle é desenhado a partir do
/// `current_frame` (a fonte de verdade), NÃO do value do slider, e no MESMO `x` que a
/// linha do playhead das células (`inner.x + (frame − first)·ppf`) — régua e linha são o
/// mesmo playhead.
fn paint_scrub_lane(
    ctx: &mut PaintCtx,
    theme: Theme,
    lane: Rect,
    snap: &FlipStripSnapshot,
    inner_x: f32,
    first: i32,
    ppf: f32,
) {
    // O rail: um trilho fino centrado na faixa (onde o playhead corre).
    let rail_h = Spacing::Xs.px();
    let rail = Rect::new(lane.x, lane.y + (lane.h - rail_h) * 0.5, lane.w, rail_h);
    fill_rounded_rect(
        ctx.scene,
        rail,
        Radius::Sm.px(),
        resolve(ColorToken::Bg1, theme),
    );

    // O handle: uma pílula no quadro corrente, no MESMO x da linha do playhead. Clampada
    // ao rail (o playhead pode estar fora do vão exibido — num ciclo — mas o pega-mão
    // vive no rail).
    let px = inner_x + (snap.current_frame - first) as f32 * ppf;
    let handle_w = Spacing::Sm.px();
    let hx = (px - handle_w * 0.5).clamp(lane.x, lane.x + lane.w - handle_w);
    let handle = Rect::new(hx, lane.y, handle_w, lane.h);
    fill_rounded_rect(
        ctx.scene,
        handle,
        Radius::Sm.px(),
        resolve(ColorToken::BorderEmph, theme),
    );

    // A faixa INTEIRA é arrastável — o Slider (registrado no `populate`) dá o gesto.
    ctx.host.hit_index_mut().register(ids::FLIP_SCRUB, lane);
}

/// **A opacidade do véu de clareamento** para um peso de falloff, em `0..=255`.
///
/// `(1 − peso)`: peso CHEIO (`1.0` — o quadro ativo, ou o falloff desligado) ⇒ `0` (véu
/// nulo, acento puro); peso baixo (vizinho distante) ⇒ véu forte (célula clara). É o
/// `1 − peso` porque o véu é BRANCO: quanto mais véu, mais claro o acento. O peso é o do
/// pincel (`falloff_at`, seed = sample), então a cor não mente sobre a força do gesto.
fn falloff_veil(weight: f32) -> u8 {
    ((1.0 - weight.clamp(0.0, 1.0)) * 255.0) as u8 // CLAMP-OK: 1−[0,1] ∈ [0,1] ⇒ [0,255]
}

/// Quanto o número escurece o acento para virar TINTA legível (fração do acento em direção
/// ao preto). Escuro o bastante para contrastar tanto no acento cheio quanto no bem
/// clareado — o véu branco deixava o número claro-no-claro, invisível ("apagou o número").
const ACCENT_INK_MUL: f32 = 0.35; // LITERAL-OK: fração de escurecimento, não medida de design

/// **Pinta uma célula MARCADA para o multiframe** (W7): fundo no ACENTO clareado pelo
/// falloff + o número da exposição por CIMA, em tinta ESCURA.
///
/// A ordem importa: acento → véu de clareamento → número. O número entra DEPOIS do véu, então
/// nunca é lavado por ele (Enio 2026-07-15: *"apagou o número"*). O clareamento segue o
/// `weight` (= `falloff_at`, seed = sample): cheio no ativo, mais claro nos vizinhos (falloff
/// off ⇒ peso `1.0` ⇒ véu nulo ⇒ acento cheio e uniforme). O piso do falloff mantém a célula
/// visível (o véu nunca chega ao branco).
fn paint_marked_cell(ctx: &mut PaintCtx, theme: Theme, r: Rect, label: &str, weight: f32) {
    let radius = Radius::Md.px();
    // Fundo: acento cheio.
    fill_rounded_rect(ctx.scene, r, radius, resolve(ColorToken::Accent, theme));
    // Véu de clareamento (o falloff): branco com opacidade `1 − peso`.
    let veil = falloff_veil(weight);
    if veil > 0 {
        let white = TokenColor {
            r: 255,
            g: 255,
            b: 255,
            a: veil,
        }; // LITERAL-COLOR-OK: véu de clareamento — o branco é o LIMITE do "mais claro", não uma cor de design
        fill_rounded_rect(ctx.scene, r, radius, token_to_vello(white));
    }
    // O número, por CIMA, em tinta escura do acento (contraste garantido do claro ao cheio).
    if !label.is_empty() {
        let a = ColorToken::Accent.resolve(theme);
        let ink = |c: u8| (c as f32 * ACCENT_INK_MUL) as u8;
        let dark = TokenColor {
            r: ink(a.r),
            g: ink(a.g),
            b: ink(a.b),
            a: 255,
        };
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            r,
            TypeToken::Base.px(),
            token_to_vello(dark),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **O véu CLAREIA com a distância** (o peso do falloff cai) — é o que responde a
    /// *"coloque graduações da cor de acento… 50% mais claro a cada frame de distância"*
    /// (Enio 2026-07-15). Peso cheio (`1.0`, ativo / falloff off) ⇒ véu ZERO (acento puro);
    /// peso menor ⇒ véu MAIOR (mais claro), monotônico. Mutação que sangra: ignorar o `weight`
    /// (véu constante) — cheio e fraco colapsam no mesmo valor.
    #[test]
    fn the_lightening_veil_grows_as_the_falloff_weight_falls() {
        assert_eq!(
            falloff_veil(1.0),
            0,
            "peso cheio tem de ser acento PURO (véu zero)"
        );
        // Menos peso ⇒ mais véu (mais claro), estritamente monotônico.
        assert!(falloff_veil(0.5) > falloff_veil(1.0));
        assert!(falloff_veil(0.15) > falloff_veil(0.5));
        // O piso do falloff (0.15) nunca deixa o véu chegar ao branco puro (a célula some).
        assert!(
            falloff_veil(0.15) < 255,
            "no piso do falloff a célula ainda tem de aparecer (véu < branco pleno)"
        );
    }
}
