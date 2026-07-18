//! **A pintura de UM strip** — a caixa, as cunhas do blend, os grips de fade e o nome.
//!
//! Mora fora do `stack_lane_paint` pelo cap de 600 LOC dos painéis, e porque é uma unidade: tudo
//! aqui responde "como este retângulo se explica sozinho ao artista".

use std::borrow::Cow;

use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::TimeView;
use crate::stack_ease_grip::{EASE_BAR_W, blend_px, strip_grips};

/// Below this, a rate is real time and the strip says nothing about it.
const SPEED_EPS: f64 = 1e-6; // LITERAL-PX-OK: not a length — a "is it exactly 1" epsilon

/// One strip: its box, its name, and the two blend wedges — which are the
/// crossfade, drawn from the lane's own numbers.
pub(crate) fn paint_strip(
    ctx: &mut PaintCtx,
    theme: Theme,
    view: TimeView,
    body: Rect,
    s: &ph2d_timeline::StripView,
    dim: bool,
) {
    fill_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        resolve(
            if dim {
                ColorToken::Bg3
            } else {
                ColorToken::TimelineKey
            },
            theme,
        ),
    );
    stroke_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::BorderStrong, theme),
    );

    // The blend windows. A window at an end means the strip is fading there — into
    // its neighbour if it has one (the overlap), out of nothing if it does not.
    // Drawn as a darker wedge, so the eye reads the crossfade the ear will hear.
    let wedge = resolve(ColorToken::Bg3, theme);
    for (t_from, secs) in [
        (s.t_start, s.blend_in),
        (s.t_end - s.blend_out, s.blend_out),
    ] {
        if secs <= 0.0 {
            continue;
        }
        let a = view.x(t_from).max(body.x);
        let b = view.x(t_from + secs).min(body.x + body.w);
        if b <= a {
            continue;
        }
        fill_rounded_rect(
            ctx.scene,
            Rect::new(a, body.y, b - a, body.h),
            Radius::Xs.px(),
            wedge,
        );
    }

    // **The OUTWARD fade-in wedge — the travel band in the gap BEFORE the strip.** It
    // sits to the LEFT of the box (over empty lane time), so it cannot reuse the loop
    // above, which clamps every wedge inside the body. It reads as "this clip's entry
    // reaches back here": the object crosses from the previous strip's pose to this
    // clip's first frame over this band, and then the clip plays clean (Enio,
    // 2026-07-16). Drawn within the lane's time-band clip, so a lead that runs off the
    // left is clipped, not spilled.
    if s.lead_in > 0.0 {
        let a = view.x(s.t_start - s.lead_in);
        let b = view.x(s.t_start);
        if b > a {
            fill_rounded_rect(
                ctx.scene,
                Rect::new(a, body.y, b - a, body.h),
                Radius::Xs.px(),
                wedge,
            );
        }
    }

    // **As alças de fade (B4): uma BARRA na ponta de cada cunha**, não um pontinho na quina.
    //
    // A barra vai de cima a baixo do strip porque é isso que ela É — uma borda arrastável, como a
    // de aparar. Desenhá-la como um ponto de 7×7 num strip de 22 px de altura foi o erro que o
    // smoke pegou: dava pra ver e não dava pra pegar. O que se vê e o que se agarra são a mesma
    // coisa, e a barra fica NA PONTA DA CUNHA — arrastar a barra é arrastar o fade.
    //
    // CINZA onde um vizinho define a janela: ali a sobreposição É o fade, e o jeito de mudá-la é
    // mover os strips. A barra continua pintada (o artista precisa VER que a borda está falada) e
    // não é registrada — botão dimmed que ainda despacha é botão que mente.
    if let Some((a, b)) = strip_grips(
        body,
        blend_px(view, s.t_start, s.lead_in),
        blend_px(view, s.t_start, s.blend_in),
        blend_px(view, s.t_start, s.blend_out),
    ) {
        for (g, locked, at_left) in [(a, s.ease_locked_in, true), (b, s.ease_locked_out, false)] {
            let color = resolve(
                if locked || dim {
                    ColorToken::Border
                } else {
                    ColorToken::TimelinePlayhead
                },
                theme,
            );
            // A barra mora na BORDA do grip que fica sobre a ponta da cunha (a esquerda do grip de
            // entrada, a direita do de saída) — o resto do grip é folga para o ponteiro, do lado
            // de dentro do strip, onde não disputa nada com a borda de aparar.
            let x = if at_left { g.x } else { g.x + g.w - EASE_BAR_W };
            fill_rounded_rect(
                ctx.scene,
                Rect::new(x, g.y, EASE_BAR_W, g.h),
                Radius::Xs.px(),
                color,
            );
        }
    }

    // The rate leads the name. A retimed strip is pixel-identical to a trimmed one
    // — the box says how long it plays, never how fast — so the surprising fact
    // goes FIRST, where elision cannot eat it on a narrow strip. The guessable one
    // (which clip) can be cut.
    let label: Cow<'_, str> = if (s.speed - 1.0).abs() > SPEED_EPS {
        Cow::Owned(format!("{:.2}\u{d7} {}", s.speed, s.clip_name))
    } else {
        Cow::Borrowed(s.clip_name.as_str())
    };
    let font = TypeToken::Sm.px();
    let budget = (body.w - Spacing::Xs.px() * 2.0).max(0.0);
    // **CENTRADO**, e por um motivo, não por gosto: as duas pontas do strip são onde moram as
    // cunhas de fade e os quatro grips. O meio é a única faixa do strip que nada disputa — é lá
    // que um rótulo pode ficar inteiro e legível enquanto o artista mexe nas bordas.
    let text_w = ctx.text_system.prefix_width(&label, font).min(budget);
    // **ACCENT, e NÃO `Text1`.** O corpo do strip é pintado com `TimelineKey`, que é o MESMO valor
    // de `Text1` em todos os temas (o token da chave é de PRIMEIRO PLANO — no forge os dois são
    // `oklch(0.940 …)`, no sunstone os dois são `oklch(0.220 …)`). Escrever `Text1` ali é escrever
    // branco no branco: o nome só aparecia onde a cunha do fade escurecia o fundo (Enio, smoke).
    //
    // E não basta inverter para `Bg0`: o rótulo atravessa DOIS fundos de polaridades opostas — o
    // corpo (claro) e a cunha (escura) — e a polaridade ainda troca entre temas. O accent é o
    // único token de meia-luminosidade E saturado: ele se destaca dos dois extremos, nos dois
    // temas. (Consertar isto de verdade seria o corpo do strip usar um token de SUPERFÍCIE em vez
    // de um de primeiro plano — mas isso muda o visual que o Enio aprovou.)
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &label,
        body.x + (body.w - text_w) * 0.5,
        body.y + (body.h - font) * 0.5,
        font,
        budget,
        resolve(ColorToken::Accent, theme),
    );
}
