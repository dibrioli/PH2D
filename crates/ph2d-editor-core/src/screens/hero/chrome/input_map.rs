// ph2d-chrome-sync:z=181 (dispatch priority, ADR-0107; lower = earlier)
//! **A JANELA DO INPUT MAP** — a janela flutuante que abre sobre o canvas, equivalente à do Godot
//! (plano `docs/Vector Module/30_plano_input_map.md` §0.2).
//!
//! Enio, 2026-08-24: *"precisamos do input Map completo não apenas para o jogador mas para qualquer
//! objeto do game via UI. (…) equivalente ao da godot com janela flutuante abrindo sobre o canvas"*.
//!
//! Duas metades, colocadas — o padrão do [`super::fill_modal`]:
//!
//! * [`paint_input_map_window`] — desenha o cartão em `store.input_map_pos()` (preso à viewport),
//!   com a faixa de título arrastável, o campo de nome, o **Add**, e uma linha por acção com as
//!   ligações dela por baixo;
//! * [`apply`] — a metade de despacho, ligada ao `chrome::dispatch_all` pelo `ph2d-chrome-sync`.
//!
//! # ⭐ Ele MUTA o `hero.input_map` directamente, e não pelo barramento
//!
//! O mapa é dado de **projecto** e o dono dele é o [`HeroScreen`] — o mesmo precedente das
//! `ProjectSettings`. Um `EditorAction` por gesto (criar · apagar · ligar · desligar) seria uma
//! variante por verbo para atravessar uma fronteira que **não existe**: o handler já tem
//! `&mut HeroScreen`.
//!
//! # ⛔ A escuta é um MODO, e é ela que torna o gesto possível
//!
//! `Bind…` arma [`WidgetStore::listen_for_binding`]. Enquanto isso dura, **a próxima tecla é
//! conteúdo, não atalho** — e quem tem de perguntar primeiro é o despacho de teclado. Sem essa
//! ordem, ligar `S` a uma acção **salva o projecto** e a ligação nunca acontece.

use crate::ids;
use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::screens::hero::HeroScreen;
use crate::widget::{Button, ButtonKind, paint_button};
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{Binding, InputMap};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **A largura do cartão, DERIVADA dos tokens** — não um número cravado.
///
/// ⚠️ Ela é `9 × Spacing::Xl4` (**medido**: `Xl4 = 48 px` ⇒ **432 px**), e a conta é o que a janela
/// tem de conter na linha mais larga:
///
/// | parte | largura |
/// |---|---|
/// | recuo da ligação | `Spacing::Md` = 8 |
/// | o rótulo mais longo (`Axis LeftStickX+`, 16 glifos a `TypeToken::Sm`) | ~130 |
/// | o botão **Bind…** ao lado dele | `2 × Spacing::Xl4` = 96 |
/// | a coluna do `x` à direita | `Spacing::Xl` = 16 |
/// | as duas margens | `2 × Spacing::Md` = 16 |
///
/// ⇒ ~266 px de conteúdo, e o resto é folga para um nome de acção longo **sem elidir**, que é o
/// caso que um mapa real produz. *Um múltiplo de token acompanha o design system; um `420.0` não.*
fn window_w() -> f32 {
    Spacing::Xl4.px() * 9.0 // LITERAL-PX-OK: multiplicador do token, nao um px -- derivacao acima
}

/// **A descrição legível de uma ligação** — a única porta.
///
/// ⚠️ Ela existe porque o pintor e o gate precisam da **mesma** frase: um texto construído no
/// pintor tornaria o gate uma segunda implementação, e as duas divergiriam no dia em que uma
/// variante nova entrasse no `Binding`.
#[must_use]
pub fn binding_label(b: Binding) -> String {
    match b {
        Binding::Key(k) => format!("{} 0x{:X}", tr("input_map.binding.key"), k.0),
        Binding::PadButton(p) => format!("{} {p:?}", tr("input_map.binding.pad")),
        Binding::PadAxis { axis, positive } => format!(
            "{} {axis:?}{}",
            tr("input_map.binding.axis"),
            if positive { "+" } else { "-" }
        ),
    }
}

/// Quantas linhas o cartão tem, dado o mapa: título · campo+Add · e por acção (1 + ligações + 1).
fn row_count(map: &InputMap) -> usize {
    let body: usize = map
        .actions()
        .iter()
        .map(|a| 1 + a.bindings.len() + 1)
        .sum();
    2 + if map.is_empty() { 1 } else { body }
}

/// **Desenha a janela** (no-op quando fechada).
pub fn paint_input_map_window(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    map: &InputMap,
    viewport: Rect,
) {
    let Some((x, y)) = store.input_map_pos() else {
        return;
    };
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let pad_y = Spacing::Sm.px();
    let rows = row_count(map);
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de linhas cabe em f32
    let total_h = pad_y * 2.0 + row_h * rows as f32 + gap * (rows.saturating_sub(1)) as f32;

    let window_w = window_w();
    let max_x = (viewport.x + viewport.w - window_w).max(viewport.x);
    let max_y = (viewport.y + viewport.h - total_h).max(viewport.y);
    let rect_x = x.clamp(viewport.x, max_x); // CLAMP-OK: bounds ordered + non-NaN
    let rect_y = y.clamp(viewport.y, max_y); // CLAMP-OK: bounds ordered + non-NaN
    let rect = Rect::new(rect_x, rect_y, window_w, total_h);

    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let mut cy = rect.y + pad_y;

    // ── Faixa de título = alça de arrasto, com o X à direita. ──
    let handle_rect = Rect::new(rect.x, rect.y, rect.w, pad_y + row_h);
    hit_index.register(ids::INPUT_MAP_HANDLE, handle_rect);
    paint_text(
        text_system,
        scene,
        tr("input_map.title"),
        inner_x,
        cy + (row_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    let close_rect = Rect::new(rect.x + rect.w - Spacing::Xl.px(), cy, row_h, row_h);
    hit_index.register(ids::INPUT_MAP_CLOSE, close_rect);
    paint_button(
        &Button::new(ids::INPUT_MAP_CLOSE, "x").kind(ButtonKind::Default),
        close_rect,
        scene,
        text_system,
        theme,
    );
    cy += row_h + gap;

    // ── Campo de nome + Add. ──
    let add_w = Spacing::Xl4.px();
    let name_rect = Rect::new(inner_x, cy, inner_w - add_w - gap, row_h);
    hit_index.register(ids::INPUT_MAP_NEW_NAME, name_rect);
    let typed = store.text(ids::INPUT_MAP_NEW_NAME).unwrap_or_default();
    let shown = if typed.is_empty() {
        tr("input_map.new_name.placeholder")
    } else {
        typed
    };
    stroke_rounded_rect(
        scene,
        name_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        text_system,
        scene,
        shown,
        name_rect.x + Spacing::Xs.px(),
        cy + (row_h - font) * 0.5,
        font,
        name_rect.w,
        resolve(
            if typed.is_empty() {
                ColorToken::Text3
            } else {
                ColorToken::Text1
            },
            theme,
        ),
    );
    let add_rect = Rect::new(inner_x + inner_w - add_w, cy, add_w, row_h);
    hit_index.register(ids::INPUT_MAP_ADD, add_rect);
    paint_button(
        &Button::new(ids::INPUT_MAP_ADD, tr("input_map.add")),
        add_rect,
        scene,
        text_system,
        theme,
    );
    cy += row_h + gap;

    // ── A FACE VAZIA: um mapa sem acções diz o que fazer a seguir. ──
    //
    // ⚠️ *A cura de "não há rota" é a face vazia, nunca o desaparecimento* — uma janela que abre
    // num rectângulo em branco lê-se como avariada.
    if map.is_empty() {
        paint_text(
            text_system,
            scene,
            tr("input_map.empty"),
            inner_x,
            cy + (row_h - font) * 0.5,
            font,
            inner_w,
            resolve(ColorToken::Text3, theme),
        );
        return;
    }

    let listening = store.input_map_listening();
    for (row, action) in map.actions().iter().enumerate() {
        // A linha da ACÇÃO: o nome, e o X que a apaga.
        paint_text(
            text_system,
            scene,
            &action.name,
            inner_x,
            cy + (row_h - font) * 0.5,
            font,
            inner_w,
            resolve(ColorToken::Text1, theme),
        );
        let del = ids::input_map_delete_action_id(row);
        let del_rect = Rect::new(rect.x + rect.w - Spacing::Xl.px(), cy, row_h, row_h);
        hit_index.register(del, del_rect);
        paint_button(
            &Button::new(del, "x").kind(ButtonKind::Default),
            del_rect,
            scene,
            text_system,
            theme,
        );
        cy += row_h + gap;

        // As LIGAÇÕES, indentadas, cada uma com o seu X.
        for (bi, b) in action.bindings.iter().enumerate() {
            paint_text(
                text_system,
                scene,
                &binding_label(*b),
                inner_x + Spacing::Md.px(),
                cy + (row_h - font) * 0.5,
                font,
                inner_w,
                resolve(ColorToken::Text2, theme),
            );
            let db = ids::input_map_delete_binding_id(row, bi);
            let db_rect = Rect::new(rect.x + rect.w - Spacing::Xl.px(), cy, row_h, row_h);
            hit_index.register(db, db_rect);
            paint_button(
                &Button::new(db, "x").kind(ButtonKind::Default),
                db_rect,
                scene,
                text_system,
                theme,
            );
            cy += row_h + gap;
        }

        // O **Bind…** — e ele DIZ que está à escuta, em vez de mudar só de cor.
        let listen = ids::input_map_listen_id(row);
        let listen_rect = Rect::new(inner_x + Spacing::Md.px(), cy, Spacing::Xl4.px() * 2.0, row_h);
        hit_index.register(listen, listen_rect);
        let armed = listening == Some(action.id);
        paint_button(
            &Button::new(
                listen,
                if armed {
                    tr("input_map.listening")
                } else {
                    tr("input_map.listen")
                },
            )
            .kind(if armed {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            }),
            listen_rect,
            scene,
            text_system,
            theme,
        );
        cy += row_h + gap;
    }
}

/// A metade de DESPACHO — ligada ao `dispatch_all` pelo `ph2d-chrome-sync`.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // ⭐ **O ABRIDOR, e ele é o PRIMEIRO ramo de propósito** — antes da guarda de "janela aberta".
    // Um abridor atrás dessa guarda seria a piada de precisar da janela aberta para a abrir, e é a
    // forma exacta de uma feature ficar inalcançável com todos os gates verdes.
    if id == ids::CTX_MENU_SETTINGS_INPUT_MAP {
        // ⚠️ A janela nasce no canto útil da viewport, não sob o cursor: ela é grande e o cursor
        // está no menu, no topo — abrir ali poria metade dela fora do ecrã antes do clamp.
        let v = hero.last_viewport;
        hero.store
            .open_input_map(v.x + Spacing::Xl4.px(), v.y + Spacing::Xl4.px());
        return true;
    }
    // ⭐ **A TECLA APANHADA** — o `Click` sintético que o despacho de teclado emitiu. Ele vem antes
    // da guarda da janela pelo mesmo motivo do abridor: se a janela fechasse entre a tecla e este
    // ramo, a captura ficaria pendurada para sempre.
    if id == ids::INPUT_MAP_BIND_CAPTURED {
        let key = hero.store.take_captured_key();
        let armed = hero.store.input_map_listening();
        hero.store.stop_listening();
        if let (Some(k), Some(aid)) = (key, armed)
            && let Some(a) = hero.input_map.get_mut(aid)
        {
            let b = Binding::Key(k);
            // ⚠️ **Ligar duas vezes a mesma tecla não a duplica.** A lista é um conjunto de
            // caminhos até a acção, e dois caminhos idênticos não são dois caminhos — seriam duas
            // linhas iguais no painel, uma delas impossível de distinguir da outra ao apagar.
            if !a.bindings.contains(&b) {
                a.bindings.push(b);
            }
        }
        return true;
    }
    if hero.store.input_map_pos().is_none() {
        return false;
    }
    if id == ids::INPUT_MAP_CLOSE {
        hero.store.close_input_map();
        return true;
    }
    // Um clique nu na faixa do título (sem arrasto) — consome, para nunca vazar para outro chrome.
    if id == ids::INPUT_MAP_HANDLE {
        return true;
    }
    if id == ids::INPUT_MAP_ADD {
        let name = hero
            .store
            .text(ids::INPUT_MAP_NEW_NAME)
            .unwrap_or_default()
            .trim()
            .to_string();
        // ⚠️ **Nome vazio não cria nada**, e não é validação defensiva: uma acção sem nome é
        // inalcançável por código (a leitura é `pressed("...")`), então criá-la produziria uma
        // linha no painel que nada pode usar.
        if !name.is_empty() {
            hero.input_map.create(&name);
            // ⚠️ Limpar o campo pela MESMA porta que o lê — `InteractiveState::TextInput`. Um
            // `set_text` proprio seria uma segunda escrita do mesmo facto.
            if let Some(crate::interaction::InteractiveState::TextInput { text, caret, .. }) =
                hero.store.get_mut(ids::INPUT_MAP_NEW_NAME)
            {
                text.clear();
                *caret = 0;
            }
        }
        return true;
    }
    // As linhas: o índice vem da POSIÇÃO na lista, e o id é derivado dela — então a resolução é
    // percorrer a lista a comparar, que é o que o pintor fez para a registar.
    let ids_of: Vec<_> = hero.input_map.actions().iter().map(|a| a.id).collect();
    for (row, aid) in ids_of.into_iter().enumerate() {
        if id == ids::input_map_delete_action_id(row) {
            hero.input_map.remove(aid);
            // ⚠️ Apagar a acção à escuta **desarma** a escuta: senão a próxima tecla iria para um
            // id que já não existe, e sumiria sem que nada na tela dissesse porquê.
            if hero.store.input_map_listening() == Some(aid) {
                hero.store.stop_listening();
            }
            return true;
        }
        if id == ids::input_map_listen_id(row) {
            // Carregar de novo no botão já armado **desarma** — o gesto tem de ter volta.
            if hero.store.input_map_listening() == Some(aid) {
                hero.store.stop_listening();
            } else {
                hero.store.listen_for_binding(aid);
            }
            return true;
        }
        let n = hero
            .input_map
            .get(aid)
            .map_or(0, |a| a.bindings.len());
        for bi in 0..n {
            if id == ids::input_map_delete_binding_id(row, bi) {
                if let Some(a) = hero.input_map.get_mut(aid) {
                    a.bindings.remove(bi);
                }
                return true;
            }
        }
    }
    false
}
