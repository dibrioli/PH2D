//! Os painéis da equipa no canto (módulo filho de [`super`]): a lista dos comentários, as versões
//! (R1–R5) e o exportar (E1–E7). Um de cada vez, na coluna da facilitação — à esquerda dela quando um
//! cartão da facilitação está aberto (nunca por cima).

use super::super::super::HeroScreen;
use super::paint::text_button;
use super::{Hit, TeamButton, ago, node, now_unix, register};
use crate::documents::team::{ExportChoice, Format, IoRequest, Scope};
use crate::icons::IconId;
use crate::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_vector::VectorScene;

fn btn() -> f32 {
    super::super::facilitate::btn()
}

/// A coluna dos painéis da equipa `(x, y, largura)`: a da facilitação, ou à esquerda dela quando um
/// cartão dela está aberto.
fn column(hero: &HeroScreen, area: Rect) -> (f32, f32, f32) {
    let nav = &hero.documents.live.nav;
    let (x, y, w) = super::super::facilitate::column(area, nav.frames_open, nav.search_open);
    let fac = super::super::facilitate::timer_open(hero) || super::super::facilitate::vote_open(hero);
    let w2 = btn() * 10.0; // LITERAL-PX-OK: a largura do cartão do fio, em botões
    if fac {
        (x - w2 - Spacing::Md.px(), y, w2)
    } else {
        (x + w - w2, y, w2)
    }
}

/// ⭐ Pinta o painel aberto (a lista, as versões, ou o exportar).
pub(super) fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let (x, y, w) = column(hero, area);
    let room = Rect::new(x, y, w, area.y + area.h - y);
    let t = &hero.documents.live.team;
    if t.export.is_some() {
        paint_export(hero, room, scene, ts);
    } else if t.versions_open {
        paint_versions(hero, room, scene, ts);
    } else if t.list_open {
        super::paint::paint_list(hero, room, scene, ts);
    }
}

fn card(scene: &mut VectorScene, r: Rect, theme: ph2d_tokens::Theme) {
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    crate::paint::fill_rounded_rect(scene, r, radius, crate::paint::resolve(ColorToken::BgElev, theme));
}

/// O título e o × de um painel; devolve o y da fila seguinte.
fn header(hero: &mut HeroScreen, at: Rect, scene: &mut VectorScene, ts: &mut TextSystem, key: &str, close: TeamButton) -> f32 {
    let theme = hero.theme;
    let (s, pad) = (btn(), Spacing::Sm.px());
    let y = at.y + pad;
    let fg = crate::paint::resolve(ColorToken::Text1, theme);
    crate::paint::paint_text_elided(ts, scene, tr(key), at.x + pad, y + s * 0.25, TypeToken::Sm.px(), at.w - 3.0 * pad - s, fg);
    let r = Rect::new(at.x + at.w - pad - s, y, s, s);
    super::super::super::board_nav::button(scene, &mut hero.hit_index, &hero.store, node(Hit::Button(close)), IconId::Close, r, false, theme);
    y + s + Spacing::Xs.px()
}

/// R1 — o painel das versões: abre-se com o campo do nome já pronto («Version N»).
pub(super) fn open_versions(hero: &mut HeroScreen) {
    let n = hero.documents.active_board().map_or(0, |b| b.team.versions.len());
    let t = &mut hero.documents.live.team;
    t.versions_open = true;
    t.export = None;
    t.list_open = false;
    t.renaming_version = None;
    let name = tr_with("board.version.default", &[("n", &(n + 1).to_string())]);
    super::super::super::board_nav::open_field(&mut hero.store, crate::ids::BOARD_VERSION_INPUT, name);
}

/// R1/R2 — o campo do nome: guardar uma versão nova, ou renomear a que se renomeia.
pub(super) fn version_submit(hero: &mut HeroScreen) {
    let name = super::super::super::board_nav::field_text(&hero.store, crate::ids::BOARD_VERSION_INPUT);
    let renaming = hero.documents.live.team.renaming_version.take();
    let Some(b) = hero.documents.active_board_mut() else {
        return;
    };
    let name = name.trim().to_owned();
    let n = b.team.versions.len();
    let name = if name.is_empty() {
        tr_with("board.version.default", &[("n", &(n + 1).to_string())])
    } else {
        name
    };
    match renaming {
        Some(i) => {
            if let Some(v) = b.team.versions.get_mut(i) {
                v.name = name;
            }
        }
        None => {
            b.save_version(name, now_unix());
            super::notice(hero, "board.version.saved");
        }
    }
    let next = hero.documents.active_board().map_or(0, |b| b.team.versions.len()) + 1;
    let fresh = tr_with("board.version.default", &[("n", &next.to_string())]);
    super::super::super::board_nav::open_field(&mut hero.store, crate::ids::BOARD_VERSION_INPUT, fresh);
}

/// R2/R3 — restaurar (como quadro NOVO), apagar, renomear (duplo clique no nome).
pub(super) fn version_click(hero: &mut HeroScreen, h: Hit, n: usize, double: bool) {
    let Some(id) = hero.documents.active() else {
        return;
    };
    match h {
        Hit::Restore(_) => match hero.documents.restore_version(id, n) {
            Ok(new) => {
                super::super::super::document_tabs::register_board(&mut hero.store, new);
                hero.documents.live.team.versions_open = false;
                super::notice(hero, "board.version.restored");
            }
            Err(_) => super::notice(hero, "board.version.unreadable"),
        },
        Hit::DropVersion(_) => {
            if let Some(b) = hero.documents.active_board_mut()
                && n < b.team.versions.len()
            {
                b.team.versions.remove(n);
            }
        }
        Hit::RenameVersion(_) if double => {
            let name = hero
                .documents
                .active_board()
                .and_then(|b| b.team.versions.get(n))
                .map(|v| v.name.clone())
                .unwrap_or_default();
            hero.documents.live.team.renaming_version = Some(n);
            super::super::super::board_nav::open_field(&mut hero.store, crate::ids::BOARD_VERSION_INPUT, name);
        }
        _ => {}
    }
}

fn paint_versions(hero: &mut HeroScreen, at: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let (s, pad, g) = (btn(), Spacing::Sm.px(), Spacing::Xs.px());
    let list: Vec<(String, i64)> = hero
        .documents
        .active_board()
        .map(|b| b.team.versions.iter().map(|v| (v.name.clone(), v.at)).collect())
        .unwrap_or_default();
    let now = now_unix();
    let renaming = hero.documents.live.team.renaming_version;
    let fit = (((at.h - Spacing::Md.px()) / (s + g)).floor() as usize).saturating_sub(4);
    let shown = list.len().min(fit);
    let h = pad * 2.0 + (s + g) * (3.0 + shown.max(1) as f32);
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme);
    hero.hit_index.register(node(Hit::Button(TeamButton::VersionsSurface)), r);
    let mut y = header(hero, at, scene, ts, "board.version.title", TeamButton::VersionsClose);
    // R1 — o nome e «Save version» (o campo renomeia quando se renomeia uma).
    let save_w = s * 3.0; // LITERAL-PX-OK: «Save» a `Sm`, em botões
    let field = Rect::new(at.x + pad, y, at.w - 2.0 * pad - save_w - g, s);
    super::super::super::board_nav::paint_field(crate::ids::BOARD_VERSION_INPUT, field, scene, ts, theme, &mut hero.hit_index, &hero.store);
    let save = Rect::new(field.x + field.w + g, y, save_w, s);
    let label = if renaming.is_some() { tr("board.version.rename") } else { tr("board.version.save_short") };
    text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(TeamButton::VersionsSave)), label, save, true, theme);
    y += s + g;
    let hint = tr("board.version.hint");
    crate::paint::paint_text_elided(ts, scene, hint, at.x + pad, y + s * 0.25, TypeToken::Xs.px(), at.w - 2.0 * pad, crate::paint::resolve(ColorToken::Text3, theme));
    y += s + g;
    if list.is_empty() {
        let none = tr("board.version.none");
        crate::paint::paint_text_elided(ts, scene, none, at.x + pad, y + s * 0.25, TypeToken::Sm.px(), at.w - 2.0 * pad, crate::paint::resolve(ColorToken::Text3, theme));
    }
    // R2 — da mais nova à mais velha.
    for (n, (name, when)) in list.iter().enumerate().rev().take(shown) {
        let small = s;
        let del = Rect::new(at.x + at.w - pad - small, y, small, small);
        let restore_w = s * 3.0; // LITERAL-PX-OK: «Restore» a `Sm`, em botões
        let restore = Rect::new(del.x - g - restore_w, y, restore_w, s);
        let row = Rect::new(at.x + pad, y, restore.x - g - at.x - pad, s);
        let rid = node(Hit::RenameVersion(n));
        register(&mut hero.store, rid);
        hero.store.set_tooltip(rid, tr("board.version.rename_tip"));
        let line = format!("{name} · {}", ago(*when, now));
        crate::paint::paint_text_elided(ts, scene, &line, row.x, row.y + s * 0.25, TypeToken::Sm.px(), row.w, crate::paint::resolve(ColorToken::Text1, theme));
        hero.hit_index.register(rid, row);
        let res = node(Hit::Restore(n));
        register(&mut hero.store, res);
        hero.store.set_tooltip(res, tr("board.version.restore_tip"));
        text_button(scene, ts, &mut hero.hit_index, &hero.store, res, tr("board.version.restore"), restore, false, theme);
        let d = node(Hit::DropVersion(n));
        register(&mut hero.store, d);
        hero.store.set_tooltip(d, tr("board.version.delete"));
        super::super::super::board_nav::button(scene, &mut hero.hit_index, &hero.store, d, IconId::Trash, del, false, theme);
        y += s + g;
    }
}

/// E3 — o tamanho em píxeis do PNG de agora, para o botão do tamanho (`None` = nada a exportar).
fn png_px(hero: &HeroScreen, c: ExportChoice) -> Option<(u32, u32)> {
    let b = hero.documents.active_board()?;
    let mut routes = ph2d_board_route::RouteCache::default();
    routes.sync(&b.doc);
    let sel = selection(hero);
    let page = match c.scope {
        Scope::Board => ph2d_board_export::board_page(b, &routes),
        Scope::Selection => ph2d_board_export::selection_page(b, &routes, &sel),
        Scope::Frames => ph2d_board_export::pdf_pages(b, &routes, &sel).into_iter().next(),
    }?;
    let k = ph2d_board_export::png_scale(&page, ph2d_board_export::PNG_MEGAPIXELS[c.size.min(2)]);
    let [w, h] = page.size();
    Some(((w * k).round() as u32, (h * k).round() as u32))
}

fn selection(hero: &HeroScreen) -> Vec<ph2d_board_model::ElementId> {
    hero.documents
        .live
        .editor
        .as_ref()
        .map(|e| e.selection().iter().copied().collect())
        .unwrap_or_default()
}

/// ⭐ E1–E7 — «Export»: o pedido à família do quadro (o diálogo de ficheiro é dela).
pub(super) fn export_go(hero: &mut HeroScreen) {
    let sel = selection(hero);
    let t = &mut hero.documents.live.team;
    let (Some(choice), Some(board)) = (t.export, hero.documents.active()) else {
        return;
    };
    hero.documents.live.team.io = Some(IoRequest::Export {
        board,
        choice,
        selection: sel,
    });
    hero.documents.live.team.export = None;
}

fn paint_export(hero: &mut HeroScreen, at: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let Some(c) = hero.documents.live.team.export else {
        return;
    };
    let (s, pad, g) = (btn(), Spacing::Sm.px(), Spacing::Xs.px());
    let has_sel = !selection(hero).is_empty();
    let has_frames = hero.documents.active_board().is_some_and(|b| b.doc.has_frames());
    let png = c.format == Format::Png;
    let vector_alpha = matches!(c.format, Format::Png | Format::Svg);
    let rows = 4.0 + f32::from(u8::from(c.format != Format::Excalidraw)) + f32::from(u8::from(png)) * 2.0 + f32::from(u8::from(vector_alpha));
    let h = pad * 2.0 + (s + g) * rows;
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme);
    hero.hit_index.register(node(Hit::Button(TeamButton::ExportSurface)), r);
    let mut y = header(hero, at, scene, ts, "board.export.title", TeamButton::ExportClose);
    let row = |y: f32, n: usize, i: usize| {
        let w = (at.w - 2.0 * pad - g * (n as f32 - 1.0)) / n as f32;
        Rect::new(at.x + pad + i as f32 * (w + g), y, w, s)
    };
    let formats = [
        (TeamButton::ExportPng, "PNG", Format::Png),
        (TeamButton::ExportSvg, "SVG", Format::Svg),
        (TeamButton::ExportPdf, "PDF", Format::Pdf),
    ];
    for (i, (b, label, f)) in formats.into_iter().enumerate() {
        text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(b)), label, row(y, 3, i), c.format == f, theme);
    }
    y += s + g;
    // O `.excalidraw` numa fila só dele (na fila dos quatro o nome cortava-se: «.exc…»).
    let exc = node(Hit::Button(TeamButton::ExportExcalidraw));
    text_button(scene, ts, &mut hero.hit_index, &hero.store, exc, tr("board.export.excalidraw_short"), row(y, 1, 0), c.format == Format::Excalidraw, theme);
    y += s + g;
    if c.format != Format::Excalidraw {
        // E2/E4 — a área; a selecção e os frames só quando existem.
        let mut scopes = vec![(TeamButton::ScopeBoard, "board.export.scope_board", Scope::Board)];
        if has_sel {
            scopes.push((TeamButton::ScopeSelection, "board.export.scope_selection", Scope::Selection));
        }
        if has_frames {
            scopes.push((TeamButton::ScopeFrames, "board.export.scope_frames", Scope::Frames));
        }
        let n = scopes.len();
        for (i, (b, key, sc)) in scopes.into_iter().enumerate() {
            text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(b)), tr(key), row(y, n, i), c.scope == sc, theme);
        }
        y += s + g;
    }
    if png {
        let sizes = [
            (TeamButton::SizeSmall, "board.export.small"),
            (TeamButton::SizeMedium, "board.export.medium"),
            (TeamButton::SizeLarge, "board.export.large"),
        ];
        for (i, (b, key)) in sizes.into_iter().enumerate() {
            text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(b)), tr(key), row(y, 3, i), c.size == i, theme);
        }
        y += s + g;
        let px = png_px(hero, c).map_or_else(String::new, |(w, h)| tr_with("board.export.px", &[("w", &w.to_string()), ("h", &h.to_string())]));
        crate::paint::paint_text_elided(ts, scene, &px, at.x + pad, y + s * 0.25, TypeToken::Xs.px(), at.w - 2.0 * pad, crate::paint::resolve(ColorToken::Text2, theme));
        y += s + g;
    }
    if vector_alpha {
        text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(TeamButton::Transparent)), tr("board.export.transparent"), row(y, 1, 0), c.transparent, theme);
        y += s + g;
    }
    let go = tr("board.export.go");
    text_button(scene, ts, &mut hero.hit_index, &hero.store, node(Hit::Button(TeamButton::ExportGo)), go, row(y, 1, 0), true, theme);
}
