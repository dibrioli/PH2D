//! Os painéis da equipa no canto (módulo filho de [`super`]): a lista dos comentários, as versões
//! (R1–R5) e o exportar (E1–E7). Um de cada vez, na coluna da facilitação — à esquerda dela quando um
//! cartão da facilitação está aberto (nunca por cima).

use super::super::super::HeroScreen;
use super::paint::text_button;
use super::{Hit, TeamButton, ago, node, now_unix, register, text_y};
use crate::documents::team::{Confirm, ExportChoice, Format, IoRequest, Scope};
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
    let fac =
        super::super::facilitate::timer_open(hero) || super::super::facilitate::vote_open(hero);
    let w2 = btn() * 10.0; // LITERAL-PX-OK: a largura do cartão do fio, em botões
    if fac {
        (x - w2 - Spacing::Md.px(), y, w2)
    } else {
        (x + w - w2, y, w2)
    }
}

/// As filas fixas por cima da lista (o título, o nome, «Show resolved») e das versões (o título, o
/// campo, a dica) — filas, não px.
pub(super) const HEAD_ROWS: usize = 3;

/// ⭐ Pinta o painel aberto (a lista, as versões, ou o exportar). Ele pára por cima das filas de
/// botões do canto (nunca as tapa): o que não cabe pagina.
pub(super) fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let (x, y, w) = column(hero, area);
    let room = Rect::new(x, y, w, (bottom(area) - y).max(0.0));
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
    crate::paint::fill_rounded_rect(
        scene,
        r,
        radius,
        crate::paint::resolve(ColorToken::BgElev, theme),
    );
}

/// O fundo dos painéis: por cima da fila de botões de cima do canto.
fn bottom(area: Rect) -> f32 {
    super::super::super::board_nav::button_rects(area)
        .first()
        .map_or(area.y + area.h, |(_, r)| r.y)
        - Spacing::Md.px()
}

/// Quantas filas de lista cabem num painel de altura `h` depois das [`HEAD_ROWS`] (pelo menos uma).
pub(super) fn fit_rows(h: f32) -> usize {
    let (s, g, pad) = (btn(), ph2d_tokens::control_gap_px(), Spacing::Sm.px());
    let all = ((h - 2.0 * pad) / (s + g)).floor().max(0.0) as usize;
    all.saturating_sub(HEAD_ROWS).max(1)
}

/// A janela de `n` filas que cabe (`fit`): guarda o passo da página, corrige o início, e diz se há
/// filas antes e depois.
pub(super) fn window(hero: &mut HeroScreen, n: usize, fit: usize) -> (usize, bool, bool) {
    let t = &mut hero.documents.live.team;
    t.page = fit;
    t.from = t.from.min(n.saturating_sub(fit));
    (t.from, t.from > 0, t.from + fit < n)
}

/// O título, as setas da página (quando há mais filas que cabem) e o × de um painel; devolve o y
/// da fila seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn header(
    hero: &mut HeroScreen,
    at: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    key: &str,
    close: TeamButton,
    paging: (bool, bool),
) -> f32 {
    let theme = hero.theme;
    let (s, pad, g) = (btn(), Spacing::Sm.px(), ph2d_tokens::control_gap_px());
    let y = at.y + pad;
    let r = Rect::new(at.x + at.w - pad - s, y, s, s);
    let mut left = r.x;
    let nav = super::super::super::board_nav::button;
    nav(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(close)),
        IconId::Close,
        r,
        false,
        theme,
    );
    if paging.0 || paging.1 {
        for (b, icon, live) in [
            (TeamButton::PageDown, IconId::ChevronDown, paging.1),
            (TeamButton::PageUp, IconId::ChevronUp, paging.0),
        ] {
            left -= s + g;
            if live {
                let pr = Rect::new(left, y, s, s);
                nav(
                    scene,
                    &mut hero.hit_index,
                    &hero.store,
                    node(Hit::Button(b)),
                    icon,
                    pr,
                    false,
                    theme,
                );
            }
        }
    }
    let font = TypeToken::Sm.px();
    crate::paint::paint_text_elided(
        ts,
        scene,
        tr(key),
        at.x + pad,
        text_y(y, s, font),
        font,
        left - g - at.x - pad,
        crate::paint::resolve(ColorToken::Text1, theme),
    );
    y + s + ph2d_tokens::control_gap_px()
}

/// R1 — o painel das versões: abre-se com o campo do nome já pronto («Version N»).
pub(super) fn open_versions(hero: &mut HeroScreen) {
    let n = hero
        .documents
        .active_board()
        .map_or(0, |b| b.team.versions.len());
    let t = &mut hero.documents.live.team;
    super::only_panel(t);
    t.versions_open = true;
    let name = tr_with("board.version.default", &[("n", &(n + 1).to_string())]);
    super::super::super::board_nav::open_field(
        &mut hero.store,
        crate::ids::BOARD_VERSION_INPUT,
        name,
    );
}

/// R1/R2 — o campo do nome: guardar uma versão nova, ou renomear a que se renomeia.
pub(super) fn version_submit(hero: &mut HeroScreen) {
    let name =
        super::super::super::board_nav::field_text(&hero.store, crate::ids::BOARD_VERSION_INPUT);
    let renaming = hero.documents.live.team.renaming_version.take();
    let Some(b) = hero.documents.active_board_mut() else {
        return;
    };
    // O renomear é do quadro onde começou: noutro quadro o campo guarda uma versão nova.
    let renaming = renaming.filter(|(id, _)| *id == b.id).map(|(_, i)| i);
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
    let next = hero
        .documents
        .active_board()
        .map_or(0, |b| b.team.versions.len())
        + 1;
    let fresh = tr_with("board.version.default", &[("n", &next.to_string())]);
    super::super::super::board_nav::open_field(
        &mut hero.store,
        crate::ids::BOARD_VERSION_INPUT,
        fresh,
    );
}

/// R2/R3 — restaurar (como quadro NOVO), apagar (o 2.º clique), renomear (duplo clique no nome).
pub(super) fn version_click(
    hero: &mut HeroScreen,
    h: Hit,
    n: usize,
    double: bool,
    armed: Option<Confirm>,
) {
    let Some(id) = hero.documents.active() else {
        return;
    };
    match h {
        Hit::Restore(_) => match hero.documents.restore_version(id, n) {
            Some(new) => {
                super::super::super::document_tabs::register_board(&mut hero.store, new);
                hero.documents.live.team.versions_open = false;
                super::notice(hero, "board.version.restored");
            }
            None => super::notice(hero, "board.version.unreadable"),
        },
        Hit::DropVersion(_) => {
            if armed != Some(Confirm::Version(id, n)) {
                hero.documents.live.team.confirm = Some(Confirm::Version(id, n));
                return;
            }
            let Some(b) = hero.documents.active_board_mut() else {
                return;
            };
            if n >= b.team.versions.len() {
                return;
            }
            b.team.versions.remove(n);
            // O renomear segue a SUA versão: a apagada larga-o, uma de cima desce-lhe o índice.
            let t = &mut hero.documents.live.team;
            let was = t.renaming_version;
            t.renaming_version = match t.renaming_version {
                Some((bid, m)) if bid == id && m == n => None,
                Some((bid, m)) if bid == id && m > n => Some((bid, m - 1)),
                r => r,
            };
            if was.is_some() && t.renaming_version.is_none() {
                let fresh = hero
                    .documents
                    .active_board()
                    .map_or(0, |b| b.team.versions.len());
                let name = tr_with("board.version.default", &[("n", &(fresh + 1).to_string())]);
                super::super::super::board_nav::open_field(
                    &mut hero.store,
                    crate::ids::BOARD_VERSION_INPUT,
                    name,
                );
            }
        }
        Hit::RenameVersion(_) if double => {
            let name = hero
                .documents
                .active_board()
                .and_then(|b| b.team.versions.get(n))
                .map(|v| v.name.clone())
                .unwrap_or_default();
            hero.documents.live.team.renaming_version = Some((id, n));
            super::super::super::board_nav::open_field(
                &mut hero.store,
                crate::ids::BOARD_VERSION_INPUT,
                name,
            );
        }
        _ => {}
    }
}

fn paint_versions(hero: &mut HeroScreen, at: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let (s, pad, g) = (btn(), Spacing::Sm.px(), ph2d_tokens::control_gap_px());
    let Some(bid) = hero.documents.active() else {
        return;
    };
    // R2 — da mais nova à mais velha: `(índice, nome, quando)`.
    let list: Vec<(usize, String, i64)> = hero
        .documents
        .active_board()
        .map(|b| {
            b.team
                .versions
                .iter()
                .enumerate()
                .rev()
                .map(|(i, v)| (i, v.name.clone(), v.at))
                .collect()
        })
        .unwrap_or_default();
    let now = now_unix();
    let t = &hero.documents.live.team;
    let renaming = t.renaming_version.is_some_and(|(b, _)| b == bid);
    let confirm = t.confirm;
    let fit = fit_rows(at.h);
    let (from, up, down) = window(hero, list.len(), fit);
    let shown = list.len().saturating_sub(from).min(fit);
    let h = pad * 2.0 + (s + g) * (HEAD_ROWS + shown.max(1)) as f32;
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme);
    hero.hit_index
        .register(node(Hit::Button(TeamButton::VersionsSurface)), r);
    let mut y = header(
        hero,
        at,
        scene,
        ts,
        "board.version.title",
        TeamButton::VersionsClose,
        (up, down),
    );
    // R1 — o nome e «Save version» (o campo renomeia quando se renomeia uma).
    let save_w = s * 3.0; // LITERAL-PX-OK: «Save» a `Sm`, em botões
    let field = Rect::new(at.x + pad, y, at.w - 2.0 * pad - save_w - g, s);
    super::super::super::board_nav::paint_field(
        crate::ids::BOARD_VERSION_INPUT,
        field,
        scene,
        ts,
        theme,
        &mut hero.hit_index,
        &hero.store,
    );
    let save = Rect::new(field.x + field.w + g, y, save_w, s);
    let label = if renaming {
        tr("board.version.rename")
    } else {
        tr("board.version.save_short")
    };
    text_button(
        scene,
        ts,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::VersionsSave)),
        label,
        save,
        true,
        theme,
    );
    y += s + g;
    let (xs, sm) = (TypeToken::Xs.px(), TypeToken::Sm.px());
    crate::paint::paint_text_elided(
        ts,
        scene,
        tr("board.version.hint"),
        at.x + pad,
        text_y(y, s, xs),
        xs,
        at.w - 2.0 * pad,
        crate::paint::resolve(ColorToken::Text3, theme),
    );
    y += s + g;
    if list.is_empty() {
        crate::paint::paint_text_elided(
            ts,
            scene,
            tr("board.version.none"),
            at.x + pad,
            text_y(y, s, sm),
            sm,
            at.w - 2.0 * pad,
            crate::paint::resolve(ColorToken::Text3, theme),
        );
    }
    for (n, name, when) in list.iter().skip(from).take(shown) {
        let n = *n;
        let del = Rect::new(at.x + at.w - pad - s, y, s, s);
        let restore_w = s * 3.0; // LITERAL-PX-OK: «Restore» a `Sm`, em botões
        let restore = Rect::new(del.x - g - restore_w, y, restore_w, s);
        let row = Rect::new(at.x + pad, y, restore.x - g - at.x - pad, s);
        let rid = node(Hit::RenameVersion(n));
        register(&mut hero.store, rid);
        hero.store.set_tooltip(rid, tr("board.version.rename_tip"));
        let line = format!("{name} · {}", ago(*when, now));
        crate::paint::paint_text_elided(
            ts,
            scene,
            &line,
            row.x,
            text_y(row.y, s, sm),
            sm,
            row.w,
            crate::paint::resolve(ColorToken::Text1, theme),
        );
        hero.hit_index.register(rid, row);
        let res = node(Hit::Restore(n));
        register(&mut hero.store, res);
        hero.store.set_tooltip(res, tr("board.version.restore_tip"));
        text_button(
            scene,
            ts,
            &mut hero.hit_index,
            &hero.store,
            res,
            tr("board.version.restore"),
            restore,
            false,
            theme,
        );
        // R2 — o 1.º clique arma (o lixo aceso), o 2.º apaga.
        let d = node(Hit::DropVersion(n));
        let armed = confirm == Some(Confirm::Version(bid, n));
        register(&mut hero.store, d);
        let tip = if armed {
            "board.version.confirm"
        } else {
            "board.version.delete"
        };
        hero.store.set_tooltip(d, tr(tip));
        super::super::super::board_nav::button(
            scene,
            &mut hero.hit_index,
            &hero.store,
            d,
            IconId::Trash,
            del,
            armed,
            theme,
        );
        y += s + g;
    }
}

/// E2/E4 — a área que vale: a escolhida, ou o quadro quando ela já não existe (a selecção largada,
/// os frames apagados com o diálogo aberto).
fn effective(hero: &HeroScreen, c: ExportChoice) -> Scope {
    let gone = match c.scope {
        Scope::Board => false,
        Scope::Selection => selection(hero).is_empty(),
        Scope::Frames => !hero
            .documents
            .active_board()
            .is_some_and(|b| b.doc.has_frames()),
    };
    if gone { Scope::Board } else { c.scope }
}

/// E3 — o tamanho em píxeis do PNG de agora, para o botão do tamanho (`None` = nada a exportar).
/// Refeito só quando a escolha, a selecção ou o quadro mudam (a página percorre o quadro).
fn png_px(hero: &mut HeroScreen, c: ExportChoice) -> Option<(u32, u32)> {
    let sel = selection(hero);
    let scope = effective(hero, c);
    let theme = hero.theme;
    let (board, live) = hero.documents.active_parts()?;
    let key = (board.id, board.doc.rev(), c, sel.clone());
    if let Some((k, px)) = &live.team.px
        && *k == key
    {
        return *px;
    }
    let routes =
        super::super::super::board_view::editor(&mut live.editor, theme).routes(&board.doc);
    let page = match scope {
        Scope::Board => ph2d_board_export::board_page(board, routes),
        Scope::Selection => ph2d_board_export::selection_page(board, routes, &sel),
        Scope::Frames => ph2d_board_export::pdf_pages(board, routes, &sel)
            .into_iter()
            .next(),
    };
    let px = page.map(|page| {
        let k =
            ph2d_board_export::png_scale(&page, ph2d_board_export::PNG_MEGAPIXELS[c.size.min(2)]);
        let [w, h] = page.size();
        ((w * k).round() as u32, (h * k).round() as u32)
    });
    live.team.px = Some((key, px));
    px
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
    let (Some(mut choice), Some(board)) =
        (hero.documents.live.team.export, hero.documents.active())
    else {
        return;
    };
    choice.scope = effective(hero, choice);
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
    let (s, pad, g) = (btn(), Spacing::Sm.px(), ph2d_tokens::control_gap_px());
    let has_sel = !selection(hero).is_empty();
    let has_frames = hero
        .documents
        .active_board()
        .is_some_and(|b| b.doc.has_frames());
    let png = c.format == Format::Png;
    let vector_alpha = matches!(c.format, Format::Png | Format::Svg);
    // Título · formatos · `.excalidraw` · «Export…», e as filas que o formato pede.
    let rows = 4
        + usize::from(c.format != Format::Excalidraw)
        + usize::from(png) * 2
        + usize::from(vector_alpha);
    let h = pad * 2.0 + (s + g) * rows as f32;
    let scope = effective(hero, c);
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme);
    hero.hit_index
        .register(node(Hit::Button(TeamButton::ExportSurface)), r);
    let mut y = header(
        hero,
        at,
        scene,
        ts,
        "board.export.title",
        TeamButton::ExportClose,
        (false, false),
    );
    let row = |y: f32, n: usize, i: usize| {
        let w = (at.w - 2.0 * pad - g * (n as f32 - 1.0)) / n as f32;
        Rect::new(at.x + pad + i as f32 * (w + g), y, w, s)
    };
    let formats = [
        (TeamButton::ExportPng, "board.export.png_short", Format::Png),
        (TeamButton::ExportSvg, "board.export.svg_short", Format::Svg),
        (TeamButton::ExportPdf, "board.export.pdf_short", Format::Pdf),
    ];
    for (i, (b, label, f)) in formats.into_iter().enumerate() {
        text_button(
            scene,
            ts,
            &mut hero.hit_index,
            &hero.store,
            node(Hit::Button(b)),
            tr(label),
            row(y, 3, i),
            c.format == f,
            theme,
        );
    }
    y += s + g;
    // O `.excalidraw` numa fila só dele (na fila dos quatro o nome cortava-se: «.exc…»).
    let exc = node(Hit::Button(TeamButton::ExportExcalidraw));
    text_button(
        scene,
        ts,
        &mut hero.hit_index,
        &hero.store,
        exc,
        tr("board.export.excalidraw_short"),
        row(y, 1, 0),
        c.format == Format::Excalidraw,
        theme,
    );
    y += s + g;
    if c.format != Format::Excalidraw {
        // E2/E4 — a área; a selecção e os frames só quando existem.
        let mut scopes = vec![(
            TeamButton::ScopeBoard,
            "board.export.scope_board",
            Scope::Board,
        )];
        if has_sel {
            scopes.push((
                TeamButton::ScopeSelection,
                "board.export.scope_selection",
                Scope::Selection,
            ));
        }
        if has_frames {
            scopes.push((
                TeamButton::ScopeFrames,
                "board.export.scope_frames",
                Scope::Frames,
            ));
        }
        let n = scopes.len();
        for (i, (b, key, sc)) in scopes.into_iter().enumerate() {
            text_button(
                scene,
                ts,
                &mut hero.hit_index,
                &hero.store,
                node(Hit::Button(b)),
                tr(key),
                row(y, n, i),
                scope == sc,
                theme,
            );
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
            text_button(
                scene,
                ts,
                &mut hero.hit_index,
                &hero.store,
                node(Hit::Button(b)),
                tr(key),
                row(y, 3, i),
                c.size == i,
                theme,
            );
        }
        y += s + g;
        let px = png_px(hero, c).map_or_else(String::new, |(w, h)| {
            tr_with(
                "board.export.px",
                &[("w", &w.to_string()), ("h", &h.to_string())],
            )
        });
        let xs = TypeToken::Xs.px();
        crate::paint::paint_text_elided(
            ts,
            scene,
            &px,
            at.x + pad,
            text_y(y, s, xs),
            xs,
            at.w - 2.0 * pad,
            crate::paint::resolve(ColorToken::Text2, theme),
        );
        y += s + g;
    }
    if vector_alpha {
        text_button(
            scene,
            ts,
            &mut hero.hit_index,
            &hero.store,
            node(Hit::Button(TeamButton::Transparent)),
            tr("board.export.transparent"),
            row(y, 1, 0),
            c.transparent,
            theme,
        );
        y += s + g;
    }
    let go = tr("board.export.go");
    text_button(
        scene,
        ts,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ExportGo)),
        go,
        row(y, 1, 0),
        true,
        theme,
    );
}
