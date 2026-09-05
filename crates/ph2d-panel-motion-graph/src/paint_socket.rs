//! **Os SOCKETS de um cartão** — o glifo (○ valor · ◇ coluna), o halo de alvo de ligação e o
//! domínio da porta de saída. Irmão cortado do `paint.rs` por responsabilidade quando a porta da
//! moldura do tema (wave 4 do redesenho, 2026-09-05) o fez passar o teto de LOC do painel.

use super::*;

/// Draw one socket dot: coloured by [`Domain`], SHAPED by [`Dim`] — a diamond ◇ for a
/// multi-component column, a circle ○ for a single value ([`socket_glyph`]). The one
/// door the input AND output loops go through, so the two can never disagree on how a
/// socket looks (the bug a second copy invites six months on).
pub(super) fn paint_socket_glyph(
    ctx: &mut PaintCtx,
    cx: f32,
    cy: f32,
    r: f32,
    p: &PortView,
    theme: Theme,
) {
    let color = resolve(socket_token(p), theme);
    match socket_glyph(p.dim) {
        SocketGlyph::Value => fill_circle(ctx.scene, cx, cy, r, color),
        SocketGlyph::Column => fill_diamond(ctx.scene, cx, cy, r, color),
    }
}

/// A ring around a socket (the drop-target highlight), drawn as a rounded-rect stroke
/// whose corner radius equals its half-side — i.e. a circle — so it reuses
/// `stroke_rounded_rect` (no per-frame trig, HR-5-clean). `token` is `Accent` for a
/// compatible target and `Danger` for an incompatible one: the ring is a circle
/// regardless of the socket's own glyph, because it is a halo, not the socket.
pub(super) fn highlight_socket(
    ctx: &mut PaintCtx,
    cx: f32,
    cy: f32,
    r: f32,
    theme: Theme,
    token: ColorToken,
) {
    let d = 2.0 * r;
    // ⚠️ NÃO passa pela porta do tema: é o HALO de um alvo de ligação durante um arrasto — uma
    //    mensagem sobre conteúdo (`Accent` = compatível, `Danger` = incompatível), não moldura.
    stroke_rounded_rect(
        ctx.scene,
        Rect::new(cx - r, cy - r, d, d),
        r,
        2.0,
        resolve(token, theme),
    );
}

/// The source output port's [`Domain`] (ghost color) — the snapshot carries it
/// on the port view.
pub(super) fn port_out_domain(n: &GraphNodeView, port: u16) -> Domain {
    n.outputs
        .get(port as usize)
        .map(|p| p.domain)
        .unwrap_or(Domain::Instances)
}
