//! ⭐ **A apresentação** (P1–P6, módulo filho de [`super`]): os frames de topo na ordem do painel, um
//! de cada vez, por cima da JANELA inteira (o «full screen» do Miro), a câmara a viajar pela curva
//! medida, fora do diapositivo preto, sem ferramentas, títulos, comentários nem selecção. As leis
//! (`slides`, `first_slide`, `slide_camera`, `camera_at`) vivem no `ph2d-board-edit`.

use std::time::Instant;

use super::super::super::HeroScreen;
use super::{Hit, TeamButton, node};
use crate::documents::team::Show;
use crate::icons::IconId;
use crate::zones::Rect;
use ph2d_board_model::Camera;
use ph2d_board_render::{EXPORT_PAPER, ExportLook};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// P3 — fora do diapositivo, preto (o «Dark background» do Miro, medido no GIF oficial), o mesmo em
/// qualquer tema.
const ROOM: Color = Color::from_rgba8(0, 0, 0, 255); // LITERAL-COLOR-OK: a sala, igual em todo tema

/// A área da apresentação: a janela inteira.
fn area(hero: &HeroScreen) -> [f64; 4] {
    let v = hero.last_viewport;
    [
        f64::from(v.x),
        f64::from(v.y),
        f64::from(v.w),
        f64::from(v.h),
    ]
}

/// A apresentação está a correr?
#[must_use]
pub fn showing(hero: &HeroScreen) -> bool {
    hero.documents.live.team.show.is_some()
}

/// ⭐ P1/P2 — começa no quadro activo: os diapositivos, o 1.º (o seleccionado, ou o que o contém).
/// Sem frames visíveis, avisa (sem frames, ou todos escondidos) e não abre.
pub fn start(hero: &mut HeroScreen) {
    let selection: Vec<_> = hero
        .documents
        .live
        .editor
        .as_ref()
        .map(|e| e.selection().iter().copied().collect())
        .unwrap_or_default();
    let Some(b) = hero.documents.active_board() else {
        return;
    };
    let slides = ph2d_board_edit::slides(b);
    if slides.is_empty() {
        let key = if b.doc.has_frames() {
            "board.present.all_hidden"
        } else {
            "board.present.no_frames"
        };
        super::notice(hero, key);
        return;
    }
    let at = ph2d_board_edit::first_slide(b, &slides, &selection);
    let (board, before) = (b.id, b.camera);
    let t = &mut hero.documents.live.team;
    let was_commenting = std::mem::take(&mut t.commenting);
    t.show = Some(Show {
        board,
        slides,
        at,
        from: before,
        to: before,
        since: Instant::now(),
        before,
        was_commenting,
    });
    hero.store.set_focus(None);
}

/// A câmara de agora: a viagem do `from` para o diapositivo actual (P5; *reduced motion* salta).
fn camera(hero: &HeroScreen) -> Option<Camera> {
    let s = hero.documents.live.team.show.as_ref()?;
    let b = hero.documents.boards().get(s.board)?;
    let to = ph2d_board_edit::slide_camera(b, *s.slides.get(s.at)?, area(hero))?;
    if hero.motion.reduced_motion() {
        return Some(to);
    }
    Some(ph2d_board_edit::camera_at(
        s.from,
        to,
        s.since.elapsed().as_secs_f64(),
    ))
}

/// P4 — o seguinte (`true`) ou o anterior; nas pontas não mexe.
pub fn step(hero: &mut HeroScreen, next: bool) {
    let cam = camera(hero);
    let Some(s) = hero.documents.live.team.show.as_mut() else {
        return;
    };
    let to = if next {
        (s.at + 1).min(s.slides.len() - 1)
    } else {
        s.at.saturating_sub(1)
    };
    go(s, to, cam);
}

fn go(s: &mut Show, to: usize, cam: Option<Camera>) {
    if to == s.at {
        return;
    }
    s.from = cam.unwrap_or(s.from);
    s.at = to;
    s.since = Instant::now();
}

/// P6 — sair: a vista de antes volta (e a ferramenta de comentar, se estava armada).
pub fn stop(hero: &mut HeroScreen) {
    let Some(s) = hero.documents.live.team.show.take() else {
        return;
    };
    hero.documents.live.team.commenting = s.was_commenting;
    if let Some(b) = hero
        .documents
        .active_board_mut()
        .filter(|b| b.id == s.board)
    {
        b.camera = s.before;
    }
}

/// ⭐ P4 — a apresentação toma TODAS as teclas: `→ ↓ PageDown Espaço` · `← ↑ PageUp` · `Home`/`End` ·
/// `Esc`. `None` = não está a correr.
pub fn key(
    hero: &mut HeroScreen,
    k: super::super::super::board_keys::BoardKey,
) -> Option<super::super::super::board_keys::KeyOutcome> {
    use super::super::super::board_keys::{BoardKey, KeyOutcome};
    if !showing(hero) {
        return None;
    }
    let cam = camera(hero);
    match k {
        BoardKey::Right | BoardKey::Down | BoardKey::PageDown | BoardKey::Space => step(hero, true),
        BoardKey::Left | BoardKey::Up | BoardKey::PageUp => step(hero, false),
        BoardKey::Home | BoardKey::End => {
            if let Some(s) = hero.documents.live.team.show.as_mut() {
                let to = if k == BoardKey::Home {
                    0
                } else {
                    s.slides.len() - 1
                };
                go(s, to, cam);
            }
        }
        BoardKey::Escape => stop(hero),
        _ => {}
    }
    Some(KeyOutcome {
        consumed: true,
        copy: None,
    })
}

/// ⭐ P3 — pinta a apresentação por cima de TUDO (a última coisa do ecrã): a sala preta, o quadro pela
/// câmara da viagem no modo limpo (sem grelha, títulos, selos — o do export, sem papel), o preto à
/// volta do diapositivo, e os comandos no canto de baixo à esquerda com o contador.
pub fn paint(hero: &mut HeroScreen, viewport: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    if !showing(hero) {
        return;
    }
    let Some(cam) = camera(hero) else {
        stop(hero);
        return;
    };
    let theme = hero.theme;
    let a = area(hero);
    let full = crate::paint::rect_to_vello(viewport);
    scene.fill_rect(full, ROOM);
    // A sala inteira toma o clique (o que está por baixo — abas, barras — não se carrega).
    hero.hit_index
        .register(node(Hit::Button(TeamButton::ShowSurface)), viewport);
    let (at, n, slide) = {
        let s = hero.documents.live.team.show.as_ref().expect("visto acima");
        (s.at, s.slides.len(), s.slides[s.at])
    };
    if let Some((board, live)) = hero.documents.active_parts() {
        board.camera = cam;
        let ed = super::super::super::board_view::editor(&mut live.editor, theme);
        let routes = ed.routes(&board.doc);
        let cache = &mut live.render_cache;
        cache.export = Some(ExportLook {
            paper: None,
            vector: false,
        });
        cache.hide_votes = true;
        let corners = board.doc.get(slide).map(|f| {
            (
                cam.to_screen(a, [f.x, f.y]),
                cam.to_screen(a, [f.x + f.w, f.y + f.h]),
            )
        });
        // O diapositivo é papel (um frame sem fundo não fica preto: o papel do exportar).
        if let Some((p0, p1)) = corners {
            scene.fill_rect(
                ph2d_vector::Rect::new(p0[0], p0[1], p1[0], p1[1]),
                EXPORT_PAPER,
            );
        }
        ph2d_board_render::paint(board, a, scene, theme, ts, cache, routes, None);
        cache.export = None;
        // Fora do diapositivo, preto (os vizinhos não espreitam).
        if let Some((p0, p1)) = corners {
            let (vx0, vy0, vx1, vy1) = (a[0], a[1], a[0] + a[2], a[1] + a[3]);
            for [x0, y0, x1, y1] in [
                [vx0, vy0, vx1, p0[1]],
                [vx0, p1[1], vx1, vy1],
                [vx0, p0[1], p0[0], p1[1]],
                [p1[0], p0[1], vx1, p1[1]],
            ] {
                if x1 > x0 && y1 > y0 {
                    scene.fill_rect(ph2d_vector::Rect::new(x0, y0, x1, y1), ROOM);
                }
            }
        }
    }
    // P4 — ◀ n/N ▶ e ✕, no canto de baixo à esquerda.
    let (s, g, m) = (Spacing::Xl2.px(), Spacing::Xs.px(), Spacing::Md.px());
    let y = viewport.y + viewport.h - m - s;
    let x = viewport.x + m;
    let label = ph2d_i18n::tr_with(
        "board.present.count",
        &[("at", &(at + 1).to_string()), ("n", &n.to_string())],
    );
    let font = TypeToken::Sm.px();
    // A caixa pela porta do rótulo (o orçamento que o pintor centrado usa): «1/3 · Esc» nunca se corta.
    let counter_w = crate::paint::rect_for_label(ts.prefix_width(&label, font));
    let prev = Rect::new(x, y, s, s);
    let count = Rect::new(x + s + g, y, counter_w, s);
    let next = Rect::new(count.x + counter_w + g, y, s, s);
    let exit = Rect::new(next.x + s + m, y, s, s);
    // Os comandos num painel (cinzento sobre preto não se via: a foto dos estados).
    super::super::super::board_nav::panel(scene, &[prev, count, next, exit], theme);
    let nav = |scene: &mut VectorScene,
               hit: &mut crate::interaction::HitIndex,
               store: &crate::interaction::WidgetStore,
               id,
               icon,
               r,
               on,
               theme| {
        super::super::super::board_nav::button_ink(
            scene,
            hit,
            store,
            id,
            icon,
            r,
            on,
            theme,
            ColorToken::Text1,
        );
    };
    nav(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ShowPrev)),
        IconId::ChevronLeft,
        prev,
        false,
        theme,
    );
    nav(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ShowNext)),
        IconId::ChevronRight,
        next,
        false,
        theme,
    );
    nav(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ShowExit)),
        IconId::Close,
        exit,
        false,
        theme,
    );
    let fg = crate::paint::resolve(ColorToken::Text1, theme);
    crate::paint::paint_text_centered(ts, scene, &label, count, font, fg);
}
