//! Right-click context-menu overlay.
//!
//! Painted by the hero orchestrator after every panel painter so the
//! floating menu lands above everything. Reads the open-menu state
//! from [`crate::interaction::WidgetStore::context_menu`] and walks a
//! per-kind option list (one match arm per [`ContextMenuKind`]).

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{ContextMenuKind, HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Standard context-menu width. Exported so cascade-opening handlers
/// in `chrome/` can decide whether the submenu fits to the right of
/// its parent row or needs to flip to the left.
pub const MENU_W: f32 = 200.0; // LITERAL-PX-OK: context menu fixed width (chrome-specific)
pub(super) const ROW_H: f32 = ROW_H_PX;
/// Quanto o centro óptico de uma linha de texto desce do centro da caixa, em fração do corpo
/// (medido nas fotos da onda 2: o «›» ficava ~2 px acima do texto de 13 px).
const OPTICAL_DROP: f32 = 0.12; // LITERAL-PX-OK: fração do corpo, não pixels
const BULLET_COL_W: f32 = 10.0; // LITERAL-PX-OK: chrome-specific bullet→text gap

/// ⚠️ `fn` e nao `const`: a escala e AUTORAVEL (plano UI/UX W4c.2), entao este numero e lido
/// por quadro em vez de assado na compilacao.
pub(super) fn pad_y() -> f32 {
    Spacing::Sm.px()
}
/// Clamp a menu rect anchored at `(anchor_x, anchor_y)` so it stays
/// inside `viewport`. Used by both the simple-row menu and the scene
/// list — cascading submenus (Settings → PPM) anchored near the
/// right edge would otherwise render off-screen.
///
/// ⚠️ **A lei mudou-se para o [`crate::widget::panel_chrome::clamp_menu_to_viewport`]** quando
/// ganhou um consumidor numa crate de PAINEL (a lista do "+ Track" da timeline). Isto delega:
/// duas cópias divergiriam, e o modo de falha seria *um* menu do app a sair da tela.
pub(super) fn clamp_to_viewport(
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
    viewport: Rect,
) -> Rect {
    crate::widget::panel_chrome::clamp_menu_to_viewport(anchor_x, anchor_y, w, h, viewport)
}

/// Com a fila ② embaixo, um menu dela acaba em `req.y`, o topo do chip (`row_menu_anchor_y`).
pub(super) fn anchor_y(
    store: &WidgetStore,
    req: crate::interaction::ContextMenuRequest,
    h: f32,
) -> f32 {
    if store.tool_row_bottom() && req.kind.opens_from_tool_row() {
        req.y - h
    } else {
        req.y
    }
}

/// Five common highlighter colors used for note backgrounds and
/// section outlines. Wave 8 Phase 2.A re-export from
/// `ph2d-editor-core::widget::panel_chrome::HIGHLIGHTER_RGBA` so panel
/// crates don't need to reach into `ph2d_editor::screens::hero` to
/// paint user-placed notes.
pub use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;

use super::menu_row_look as look;
use super::menu_row_mark::{MenuRow, contributed_row_is_current, id_is_currently_selected};

/// Paint the open context menu (if any) and register hit rects for each item. Called last in the hero
/// paint pipeline so the menu always sits on top. `viewport` clamps the menu rect so a cascade submenu
/// anchored near the right/bottom edge (e.g. Settings → PPM from the topbar gear) stays on-screen.
// ⚠️ O `motion` é o 8º argumento, e passá-lo é deliberado: `UiMotion` é o DONO do carácter, e
// espelhá-lo no `WidgetStore` (como o `present_vsync` faz) daria uma terceira cópia do mesmo facto
// — a viva, o espelho e o ficheiro de preferências. Precedente do argumento extra: `body_desc` na
// física. E o painter recebe `&UiMotion`, nunca `&mut`: quem pinta lê, nunca alveja.
#[allow(clippy::too_many_arguments)]
pub fn paint_context_menu_overlay(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    motion: &crate::motion::UiMotion,
    luz: &crate::luz_edits::PreferenciaDaLuz,
    viewport: Rect,
) {
    // ⭐ A cascata: os pais primeiro, a ponta por cima — cada nível com a linha-mãe acesa.
    for (depth, req) in store.menu_chain().into_iter().enumerate() {
        let mae = store.menu_open_row(depth);
        paint_nivel(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            project,
            motion,
            luz,
            viewport,
            req,
            mae,
        );
    }
}

/// Um nível da cascata (ou o menu sozinho). `mae` é a linha dele com o submenu aberto.
#[allow(clippy::too_many_arguments)]
fn paint_nivel(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    motion: &crate::motion::UiMotion,
    luz: &crate::luz_edits::PreferenciaDaLuz,
    viewport: Rect,
    req: crate::interaction::ContextMenuRequest,
    mae: Option<ph2d_a11y::NodeId>,
) {
    // ⚠️ A tabela mora no `menu_rows` desde que a paleta de comandos passou a oferecer os
    // mesmos verbos: uma segunda lista aqui divergiria da que a paleta lê.
    //
    // ⭐⭐⭐ **A ÚNICA excepção são as rows da ÁREA**, e ela é de natureza: quem as tem é o módulo
    // que está com o canvas, e ele publica-as no store uma vez por quadro
    // (`WidgetStore::area_entries`). Uma tabela estática teria de conhecer os ids de todo módulo do
    // app — o acoplamento que a **D2** existe para não ter.
    //
    // ⚠️ O `Vec` vive nesta função de propósito: o empréstimo é do `store`, que sobrevive a ela, e
    // o laço de rows a seguir é o MESMO. *A row pintada é `(id, rótulo, swatch)`* — a tabela guarda
    // a CHAVE (`MenuRow`), e o texto sai AQUI, no sítio que o pinta (HR-15). Um menu só é pintado
    // aberto, e a lista tem dezenas de linhas: o `Vec` por quadro é esse preço.
    let statics = super::menu_rows::menu_rows(req.kind);
    // ⭐⭐ **As duas metades da D2 caem na MESMA aritmética.** Um pulldown de área é o caso em que
    // as estáticas são `&[]` (ver `menu_rows`), e o *File* é o caso em que as duas existem — as
    // linhas do app primeiro, o que o módulo acrescenta depois. ⛔ Um `if` por menu seria a segunda
    // porta a apodrecer.
    let contrib: &[crate::widget::ToolRailEntry] = match req.kind {
        ContextMenuKind::AreaCommands { slot } => store.area_menu_rows(slot),
        kind => store.menu_contrib(kind),
    };
    // ⚠️ `filter_map` e não `map`: o `Divider` não tem id nem rótulo, e uma linha de menu sem verbo
    // seria um alvo que consome o clique e não faz nada.
    //
    // ⚠️⚠️ **DUAS linhas mudaram esta expressão no mesmo dia, e as duas mudanças ficam** (fusão de
    // 2026-09-17). A `line/UIUX` migrou a tabela estática para CHAVES (a `crate::ids::MenuRow` é a
    // tupla `(NodeId, TextKey, …)` e o texto sai aqui, por `.tr()`, HR-15); a `line/3DModeling`
    // trocou o par cru pelo enum [`MenuRow`] do `menu_row_mark`, porque *um `ToolRailEntry` sem id
    // é o RISCO* e a redacção anterior — um `filter_map` sobre `(node_id()?, label()?)` — fazia
    // **todo** divisor contribuído evaporar-se em silêncio.
    // ⛔ Ficar com uma só das duas dá um defeito calado de cada lado: sem o `.tr()` o menu pinta a
    // CHAVE crua; sem o enum, o risco desaparece. ⚠️ E os dois tipos **chamam-se `MenuRow`** — o
    // daqui é o do `menu_row_mark` (importado no topo); o da tabela chega qualificado, pelo
    // `menu_rows::menu_rows`.
    // ⭐ Uma linha que não vale para ESTE alvo não se pinta, e o que não se pinta não entra no
    // `hit_index` — a lista que se vê e a que o clique acerta são a mesma (`menu_row_is_hidden`).
    let todas = statics
        .iter()
        .filter(|&&(id, ..)| !store.menu_row_is_hidden(req.kind, id))
        .map(|&(id, key, swatch)| MenuRow::Item(id, key.tr(), swatch))
        .chain(contrib.iter().map(|e| {
            match (e.node_id(), e.label()) {
                (Some(id), Some(label)) => MenuRow::Item(id, label, None),
                // Um alvo sem id consumiria o clique e não faria nada; o que ele é, é um risco.
                _ => MenuRow::Divider,
            }
        }))
        // Os comandos da aba de família activa (o editor de scripts), no fim do menu deles.
        .chain(
            store
                .doc_menu_rows(req.kind)
                .map(|(id, l)| MenuRow::Item(id, l, None)),
        );
    // ⭐ O RISCO entre grupos sai da cara da linha (`menu_row_look::risco_antes`, que lê também os
    // `GROUP_BREAKS` da barra), nunca da tabela: a tabela é também a lista da paleta, que não tem grupos.
    let mut rows: Vec<MenuRow<'_>> = Vec::new();
    for row in todas {
        if let MenuRow::Item(id, ..) = row
            && look::risco_antes(id)
            && matches!(rows.last(), Some(MenuRow::Item(..)))
        {
            rows.push(MenuRow::Divider);
        }
        rows.push(row);
    }
    // Uma linha com ícone alinha TODAS as do menu pela coluna do ícone.
    let tem_icones = rows
        .iter()
        .any(|r| matches!(r, MenuRow::Item(id, ..) if look::icone(*id).is_some()));

    // ⭐ A chave da timeline abre o CARTÃO da maquete do dono (13/11), não a lista.
    if crate::interaction::key_card_alvo::e_cartao(req.kind) {
        super::key_card::paint(scene, text_system, theme, hit_index, store, req, viewport);
        return;
    }
    if matches!(req.kind, ContextMenuKind::SceneList) {
        super::context_menu_scene_list::paint_scene_list(
            req,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            viewport,
        );
        return;
    }
    // ⭐⭐ **O transbordo da fila** — os chips que não couberam, com os MESMOS ids. Ver
    // `hero::tool_bar::bar_split`.
    if matches!(req.kind, ContextMenuKind::ToolBarOverflow) {
        paint_tool_bar_overflow(
            req,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            viewport,
        );
        return;
    }
    // A pergunta de apagar um quadro precisa do NOME, que o `store` não tem: pinta-a o
    // `document_tabs_menu::paint_confirm_delete`, logo a seguir a este overlay.
    // O mesmo para o modal «Send to board»: pinta-o o `send_to_board::paint`, que tem os quadros.
    if matches!(
        req.kind,
        ContextMenuKind::ConfirmDeleteBoard { .. }
            | ContextMenuKind::ConfirmDeleteKanban { .. }
            | ContextMenuKind::ConfirmDeletePointsAnimation { .. }
            | ContextMenuKind::SendToBoard { .. }
            | ContextMenuKind::SendSelection { .. }
            | ContextMenuKind::KanbanBackground { .. }
            | ContextMenuKind::NotesToCard
    ) {
        return;
    }
    // Centered single-panel dialogs all share one painter signature — dispatch by kind.
    type DialogFn = fn(&mut VectorScene, &mut TextSystem, Theme, &mut HitIndex, &WidgetStore, Rect);
    let dialog: Option<DialogFn> = match req.kind {
        ContextMenuKind::RenamePaletteDialog => {
            Some(super::context_menu_dialogs::paint_palette_rename_dialog)
        }
        ContextMenuKind::NewImageDialog => {
            Some(super::context_menu_dialogs::paint_new_image_dialog)
        }
        ContextMenuKind::SheetSizeDialog => {
            Some(super::context_menu_dialogs::paint_sheet_size_dialog)
        }
        ContextMenuKind::NewSpriteDialog => {
            Some(super::context_menu_dialogs::paint_new_sprite_dialog)
        }
        ContextMenuKind::PixelColorModeDialog => {
            Some(super::context_menu_dialogs::paint_color_mode_dialog)
        }
        ContextMenuKind::PixelEditDialog => {
            Some(super::context_menu_dialogs::paint_pixel_edit_dialog)
        }
        _ => None,
    };
    if let Some(paint_dialog) = dialog {
        paint_dialog(scene, text_system, theme, hit_index, store, viewport);
        return;
    }
    // ⚠️ A altura SOMA as linhas uma a uma em vez de multiplicar pela contagem: um risco não é do
    // tamanho de um comando, e a conta antiga (`ROW_H * len`) deixaria a coluna a transbordar do
    // cartão exactamente pela altura dos riscos.
    // ⭐ Os menus da barra (onda 8) têm a cara da maquete C: ícone, atalho em coluna, caminho aceso.
    if super::menu_barra_linha::veste(store, req.kind) {
        let nivel = super::menu_barra_nivel::Nivel { req, mae, contrib };
        return super::menu_barra_nivel::pinta(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            (project, motion, luz),
            viewport,
            &rows,
            nivel,
        );
    }
    let total_h = rows.iter().map(MenuRow::h).sum::<f32>() + pad_y() * 2.0;
    // ⭐ Com ícones o rótulo começa uma coluna mais à direita: o menu ALARGA ao rótulo mais longo
    // (medido), em vez de cortar «Convert to Separate Axes» em reticências.
    let menu_w = if tem_icones {
        let lead = Spacing::Xs.px() + Spacing::Md.px() + BULLET_COL_W;
        let ic = INLINE_ICON_PX + ph2d_tokens::icon_label_gap_px();
        rows.iter()
            .filter_map(|r| match r {
                MenuRow::Item(id, l, _) => Some(
                    lead + ic
                        + text_system.prefix_width(l, TypeToken::Sm.px())
                        + Spacing::Md.px()
                        + Spacing::Xs.px()
                        + if look::abre_submenu(*id) { ic } else { 0.0 },
                ),
                MenuRow::Divider => None,
            })
            .fold(MENU_W, f32::max)
            .ceil()
    } else {
        MENU_W
    };
    let rect = clamp_to_viewport(
        req.x,
        anchor_y(store, req, total_h),
        menu_w,
        total_h,
        viewport,
    );

    let superficie = Superficie::de(theme);
    superficie.moldura(scene, rect);

    // Rows.
    let row_x = rect.x + Spacing::Xs.px();
    let row_w = rect.w - Spacing::Xs.px() * 2.0;
    // Bullet column width — same gap SceneList uses (bullet x +
    // 10 px → text x). Painted on every row so labels align whether
    // the row is current or not.
    let bullet_col_w = BULLET_COL_W;
    let mut row_y = rect.y + pad_y();
    for row in &rows {
        let h = row.h();
        let r = Rect::new(row_x, row_y, row_w, h);
        row_y += h;
        let MenuRow::Item(id, label, swatch) = row else {
            // ⭐ **O RISCO** — a mesma linha de `1 px` em `Border` que o trilho desenha
            // (`widget::tool_rail`), centrada na banda que ela ocupa. ⛔ Não entra no índice de
            // acerto: um separador que aceitasse o clique seria um alvo que não faz nada.
            fill_rounded_rect(
                scene,
                Rect::new(r.x, r.y + (h - 1.0) * 0.5, r.w, 1.0),
                0.0,
                superficie.cor(ColorToken::Border),
            );
            continue;
        };
        hit_index.register(*id, r);
        // ⭐ Uma linha INDISPONÍVEL pinta-se apagada e sem realce — a mesma porta que o despacho lê
        //   para engolir o clique ([`WidgetStore::menu_row_is_unavailable`]).
        let indisponivel = store.menu_row_is_unavailable(*id);
        if (Some(*id) == store.hot_id() || Some(*id) == mae) && !indisponivel {
            superficie.realce(scene, r);
        }
        let pad_x = Spacing::Md.px();
        let icon_size = INLINE_ICON_PX;
        let icon_y = r.y + (r.h - icon_size) * 0.5;
        // Bullet for currently-selected item (SceneList parity,
        // 2026-05-24 menu standardization).
        //
        // ⭐⭐⭐ **As DUAS metades**: as linhas que esta crate conhece respondem por tabela, e as que
        // um MÓDULO contribui respondem pelo `ButtonState` que ele próprio publica — ver
        // [`contributed_row_is_current`]. ⛔ Uma tabela só teria de conhecer os ids de todo editor
        // do app, que é o acoplamento que a **D2** existe para não ter.
        let is_current = id_is_currently_selected(*id, theme, store, project, motion, luz)
            || contributed_row_is_current(*id, contrib, store)
            || store.menu_row_is_marked(req.kind, *id);
        let destrutiva = look::destrutiva(*id);
        let bullet_x = r.x + pad_x;
        if is_current {
            let dot = ph2d_vector::Circle::new(
                ph2d_vector::Point::new(bullet_x as f64, (r.y + r.h * 0.5) as f64),
                3.0, // LITERAL-PX-OK: bullet dot radius (chrome accent, matches SceneList)
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(superficie.cor(ColorToken::Accent)),
                None,
                &dot,
            );
        }
        let glyph_x = bullet_x + bullet_col_w;
        // Leading visual: color swatch for outline picks, "+" icon
        // for create-note. Painted to the right of the bullet column
        // so the bullet always lines up flush-left.
        let tema = super::theme_menu::tema_da_linha(*id, theme);
        let has_glyph = tema.is_some()
            || swatch.is_some()
            || matches!(req.kind, ContextMenuKind::CreateNote { .. })
            || tem_icones;
        if let Some(icone) = look::icone(*id) {
            let cor = match (indisponivel, destrutiva, is_current) {
                (true, ..) => ColorToken::Text3,
                (_, true, _) => ColorToken::Danger,
                (_, _, true) => ColorToken::Accent,
                _ => ColorToken::Text2,
            };
            paint_icon(
                scene,
                icone,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                superficie.cor(cor),
                StrokeToken::Default.px(),
            );
        } else if let Some(t) = tema {
            paint_theme_swatch(
                scene,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                t,
                theme,
            );
        } else if let Some(rgba) = swatch {
            let sw = Rect::new(glyph_x, icon_y, icon_size, icon_size);
            fill_rounded_rect(
                scene,
                sw,
                3.0, // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
                VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]), // LITERAL-COLOR-OK: user-color — swatch shows an outline-pick rgba, not a theme token
            );
            // FRAME-RAW-OK: a AMOSTRA de uma cor escolhida pelo utilizador. ⚠️ O tema nao pode
            // decidir se este contorno e' preciso, porque ele NAO SABE a cor: uma amostra branca
            // sobre um fundo claro desaparece sem ele. E' o mesmo argumento do halo do cursor no
            // selector de cor.
            stroke_rounded_rect(scene, sw, 3.0, 1.0, resolve(ColorToken::Border, theme)); // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
        } else if matches!(req.kind, ContextMenuKind::CreateNote { .. }) {
            paint_icon(
                scene,
                IconId::Add,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                superficie.cor(ColorToken::Text2),
                StrokeToken::Default.px(),
            );
        }
        let text_x = if has_glyph {
            glyph_x + icon_size + ph2d_tokens::icon_label_gap_px()
        } else {
            glyph_x
        };
        let text_y = r.y + (r.h - TypeToken::Sm.px()) * 0.5;
        // ⭐ **O atalho numa COLUNA à direita** (dono 12/11): o rótulo traz «Save · Ctrl+S» e o
        //   corte sai da MESMA porta da paleta (`shortcut_column`), nunca de uma segunda tabela.
        let coluna = crate::widget::command_palette::shortcut_column(label);
        let (nome, atalho) = crate::widget::command_palette::split_shortcut(&coluna);
        let fim_do_texto = r.x + r.w - pad_x;
        let fim_do_nome = match atalho {
            Some(k) => {
                let kw = text_system.prefix_width(k, TypeToken::Sm.px());
                let kx = fim_do_texto - kw;
                paint_text(
                    text_system,
                    scene,
                    k,
                    kx,
                    text_y,
                    TypeToken::Sm.px(),
                    kw,
                    superficie.cor(ColorToken::Text3),
                );
                kx - Spacing::Md.px()
            }
            None if look::abre_submenu(*id) => {
                fim_do_texto - icon_size - ph2d_tokens::icon_label_gap_px()
            }
            None => fim_do_texto,
        };
        paint_text(
            text_system,
            scene,
            nome,
            text_x,
            text_y,
            TypeToken::Sm.px(),
            (fim_do_nome - text_x).max(0.0),
            superficie.cor(if indisponivel {
                ColorToken::Text3
            } else if destrutiva {
                ColorToken::Danger
            } else if is_current {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            }),
        );
        // «›» só onde HÁ submenu (dono 12/11) — a porta é `menu_cascade::opens_submenu`.
        if look::abre_submenu(*id) {
            // O centro óptico do texto fica abaixo do centro da caixa: o «›» desce com ele.
            let cy = icon_y + TypeToken::Sm.px() * OPTICAL_DROP;
            let chev = Rect::new(r.x + r.w - pad_x - icon_size, cy, icon_size, icon_size);
            paint_icon(
                scene,
                IconId::ChevronRight,
                chev,
                superficie.cor(ColorToken::Text2),
                StrokeToken::Default.px(),
            );
        }
    }
}

/// ⭐⭐ **A amostra de uma linha de TEMA: quatro ladrilhos `2×2` com as cores principais dele**
/// ([`super::theme_menu::cores_do_tema`]) — no quadrado onde as outras linhas põem a cor única,
/// para os rótulos continuarem alinhados.
///
/// ⚠️ Cada ladrilho arredonda **só o canto de fora**: o conjunto lê-se como UMA amostra com a quina
/// do menu, e a moldura por cima é a da amostra de cor (um tema escuro sobre um menu escuro sumiria
/// sem ela — o mesmo argumento da moldura da amostra de cor, ao lado).
pub(super) fn paint_theme_swatch(scene: &mut VectorScene, sw: Rect, tema: Theme, do_menu: Theme) {
    let raio = 3.0; // LITERAL-PX-OK: o raio da amostra de cor do menu, o mesmo da linha vizinha
    let (mw, mh) = (sw.w * 0.5, sw.h * 0.5);
    let cores = super::theme_menu::cores_do_tema(tema);
    // Ordem de leitura: painel · cartão em cima, acento · texto em baixo.
    let ladrilhos = [
        (Rect::new(sw.x, sw.y, mw, mh), (raio, 0.0, 0.0, 0.0)),
        (Rect::new(sw.x + mw, sw.y, mw, mh), (0.0, raio, 0.0, 0.0)),
        (Rect::new(sw.x, sw.y + mh, mw, mh), (0.0, 0.0, 0.0, raio)),
        (
            Rect::new(sw.x + mw, sw.y + mh, mw, mh),
            (0.0, 0.0, raio, 0.0),
        ),
    ];
    for ((r, raios), cor) in ladrilhos.into_iter().zip(cores) {
        crate::paint::fill_rounded_rect_radii(scene, r, raios, cor);
    }
    // FRAME-RAW-OK: a moldura da AMOSTRA de um tema, pela mesma razão da amostra de cor.
    stroke_rounded_rect(scene, sw, raio, 1.0, resolve(ColorToken::Border, do_menu)); // LITERAL-PX-OK: o traço de 1 px da amostra
}

// The centered dialog-style popovers (API-key / vector-prompt / palette-rename) live in the sibling
// `super::context_menu_dialogs` (file-LOC split); this module dispatches to them above.

/// ⭐⭐⭐ **O CORPO DO TRANSBORDO** — os chips que não couberam na fila, numa grelha.
///
/// > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
///
/// ⚠️ **Os ids são os MESMOS da fila**, e é isso que torna este corpo barato: quem despacha
/// continua a ser o `chrome::rail_*`, e um verbo copiado para aqui seria a segunda porta que o
/// `CLAUDE.md` §5.0 cataloga como a espécie mais cara de controlo morto.
///
/// ⚠️ **E a lista sai da MESMA função que a faixa usa** (`tool_bar::bar_split`) — nunca de uma
/// segunda conta. Uma cópia da aritmética poria um chip nos dois sítios, ou em nenhum.
#[allow(clippy::too_many_arguments)]
fn paint_tool_bar_overflow(
    req: crate::interaction::ContextMenuRequest,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
    viewport: Rect,
) {
    // ⭐ **Publicado por quem o calculou** — a fila, uma vez por quadro (`tool_bar::bar_split`).
    // ⛔ Uma segunda conta aqui poria um chip nos dois sítios, ou em nenhum: a faixa é a única que
    // sabe a largura dela.
    let over: Vec<_> = store.tool_overflow().to_vec();
    if over.is_empty() {
        return;
    }
    if crate::widget::lista_flutuante::cromo_em_cartoes() {
        return super::tool_bar::paint_transbordo_em_lista(
            req,
            over,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            viewport,
        );
    }
    let size = store.rail_button_size();
    let pad = Spacing::Sm.px();
    // ⚠️ Uma COLUNA, e não uma grelha: o menu abre debaixo de um chip da faixa, e uma grelha larga
    // taparia a área de desenho que esta wave existe para poupar.
    let rail = crate::widget::ToolRail::new(
        ph2d_a11y::NodeId(204),
        tr("chrome.menu.more_tools"),
        over.into_iter().collect(),
    );
    // A largura de uma coluna de chips, e a altura pelo passo de linha — as mesmas contas do
    // trilho vertical, para o menu não inventar métrica própria.
    let inner_w = crate::widget::CHIP_X_OFFSET_PX + size.chip_px() + Spacing::Xs.px();
    let inner_h = crate::widget::line_pitch(size.chip_px()) * rail.entries.len() as f32;
    let rect = clamp_to_viewport(
        req.x,
        anchor_y(store, req, inner_h + pad * 2.0),
        inner_w + pad * 2.0,
        inner_h + pad * 2.0,
        viewport,
    );
    Superficie::de(theme).moldura(scene, rect);
    let content = Rect::new(rect.x + pad, rect.y + pad, inner_w, inner_h);
    crate::widget::paint_tool_rail_axis(
        &rail,
        content,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        crate::widget::RailAxis::Vertical,
    );
    // ⚠️ O fundo ENGOLE o clique (a mesma razão do `RAIL_BACKDROP` da faixa), e vai ANTES dos
    // chips: o `HitIndex` caminha de trás para a frente.
    hit_index.register(ids::RAIL_BACKDROP, rect);
    for slot in crate::widget::entry_rects(&rail, content, size, crate::widget::RailAxis::Vertical)
    {
        if let Some(id) = slot.id {
            hit_index.register(id, slot.rect);
        }
    }
}
