//! ⭐⭐⭐ **O GESTO QUE MOVE UMA ABA** — as zonas de largada, o lugar na fila, e o que se pinta
//! enquanto o dedo anda.
//!
//! Cortado do [`super::slot_tabs`] em 2026-09-08 pelo tecto de LOC (802/700), e o corte é por
//! RESPONSABILIDADE: aquele ficheiro responde *«quem se senta neste encaixe, em que ordem, e como
//! isso se desenha»* e este responde *«o que acontece quando o artista PEGA numa aba»*.
//!
//! ⚠️ **Foi este lado que cresceu**, com o report de 2026-09-08 (*«não é possível reordenar as abas
//! arrastando com o mouse»*): a largada deixou de responder só *«que encaixe?»* e passou a
//! responder também *«que LUGAR na fila?»* — mais a marca que diz onde ela vai cair.
//!
//! ⚠️ **A ordem de leitura é `resolve_tab_drop` → [`tab_row_drop`] → [`tab_drop_caret`]:** a
//! primeira é o que corre no quadro, a segunda é a decisão, e a terceira é a MESMA decisão
//! desenhada. *Uma segunda aritmética de «em que posição isto cai?» ao lado da primeira é o
//! defeito que a fila de abas já pagou uma vez.*

use super::HeroScreen;
use super::slot_tabs::{TAB_BAR_H, occupants, occupied, tab_node_id};
use crate::paint::{fill_rounded_rect, rect_to_vello, resolve};
use crate::screens::slot::Slot;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Espessura do contorno do encaixe sob o dedo.
const DROP_OUTLINE_PX: f32 = 2.0; // LITERAL-PX-OK: contorno da zona de largada (chrome)
/// Espessura da marca que diz onde a aba vai cair. ⚠️ **A mesma do contorno da zona de largada** —
/// as duas dizem *«é aqui»* no mesmo gesto, e dois pesos diferentes leriam como duas coisas.
const DROP_CARET_PX: f32 = DROP_OUTLINE_PX;

/// Largura da etiqueta fantasma que segue o dedo. ⚠️ Fixa, e não a largura da aba de origem: ela
/// atravessa encaixes de larguras diferentes, e uma etiqueta que muda de tamanho a meio do gesto
/// lê-se como o app a decidir alguma coisa que ele não decidiu.
const DRAG_LABEL_W: f32 = 120.0; // LITERAL-PX-OK: etiqueta fantasma do arrasto (chrome)

/// ⭐⭐⭐ **AS ZONAS DE LARGADA de um arrasto em curso** — e é aqui que a **D1** deixa de ser uma
/// verificação e passa a ser um `Constraint`.
///
/// > *«O erro não é detectado, é **inexprimível**.»* — `00_DECISOES_DO_ENIO.md`, D4
///
/// Um encaixe que o painel **não** permite simplesmente **não é oferecido**: não se pinta, não se
/// testa, não existe para este gesto. ⛔ A alternativa — aceitar a largada e depois recusá-la — é a
/// forma que o Enio nomeou como errada: *o artista faz o gesto, vê a resposta, e não sabe porquê.*
///
/// ⚠️ **O encaixe de ONDE ele veio é oferecido também**, e de propósito: largar de volta é como se
/// desiste de um arrasto sem precisar de saber que a tecla `Esc` existe.
///
/// Devolve `(encaixe, rect)` para cada destino legal, na ordem de [`Slot::ALL`].
#[must_use]
pub fn drop_targets(hero: &HeroScreen, panel: NodeId) -> Vec<(Slot, Rect)> {
    let Some(layout) = hero.last_layout else {
        return Vec::new();
    };
    let allowed = crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.panel_node_id == panel)
            .map(|p| p.manifest.allowed_slots)
    })
    .flatten();
    let Some(allowed) = allowed else {
        return Vec::new();
    };
    let rects = layout.slot_rects(occupied(hero));
    allowed
        .iter()
        .filter_map(|slot| {
            let r = rects.get(slot);
            (r.w > 0.0 && r.h > 0.0).then_some((slot, r))
        })
        .collect()
}

/// ⭐⭐ **RESOLVE a largada** — corre no início do quadro, e consome o pedido uma vez só.
///
/// ⚠️ **Ele julga contra o layout do quadro ANTERIOR, e isso é o correcto**, não um compromisso: a
/// largada tem de ser medida contra a geometria que o artista estava a ver quando largou. Julgá-la
/// contra um layout já reconstruído com o painel movido seria perguntar ao futuro.
/// ⭐⭐ **A fila de abas onde o dedo largou, e como ela fica** — `None` se a largada não foi sobre
/// fila nenhuma que este painel possa habitar.
///
/// Devolve o encaixe **e a fila inteira já reordenada**, porque é a fila inteira que
/// [`crate::interaction::WidgetStore::set_tab_row_order`] tem de receber.
///
/// ⚠️⚠️ **A geometria de cada aba vem do ÍNDICE DE ACERTO, que ainda é o do quadro ANTERIOR** — o
/// `resolve_tab_drop` corre antes do `hit_index.clear_for_frame()`. Não é um acidente feliz: é o
/// mesmo princípio que o doc do `resolve_tab_drop` já declara — *a largada tem de ser medida
/// contra a geometria que o artista estava a ver quando largou*. ⛔ Re-medir as larguras aqui
/// exigiria o `TextSystem` e responderia sobre um layout que ainda não existe.
///
/// ⚠️ **Uma aba que a fila não coube (transbordo) não tem rect** e é tratada como estando à
/// esquerda do dedo. É a leitura conservadora: ela empurra a largada para mais tarde na fila, e
/// nunca para antes de uma aba que o artista **viu**. A afordância de transbordo (`⋯`) continua
/// por construir, e é ela que fecha esta ponta.
fn tab_row_drop(
    hero: &HeroScreen,
    panel: NodeId,
    x: f32,
    y: f32,
) -> Option<(Slot, Vec<NodeId>, usize)> {
    let layout = hero.last_layout?;
    let allowed = crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.panel_node_id == panel)
            .map(|p| p.manifest.allowed_slots)
    })
    .flatten()?;
    for slot in Slot::ALL {
        if !allowed.contains(slot) {
            continue;
        }
        let bar = layout.slot_tabs[slot as usize];
        // ⚠️ `h <= 0` é um encaixe VAZIO: ele não TEM fila, e largar sobre uma fila que não está
        //    no ecrã seria uma largada sobre nada. ⭐ Com um ocupante a fila existe, e é isso que
        //    torna a coluna de um painel só um alvo de largada como qualquer outra.
        if bar.h <= 0.0 || !bar.contains(x, y) {
            continue;
        }
        let mut row: Vec<NodeId> = occupants(hero, slot)
            .into_iter()
            .map(|o| o.node)
            .filter(|n| *n != panel)
            .collect();
        // O lugar é o da primeira aba cujo MEIO está à direita do dedo — largar na metade
        // esquerda de uma aba põe a arrastada antes dela, na direita põe depois.
        let at = row
            .iter()
            .position(|n| {
                hero.hit_index
                    .rect_for(tab_node_id(*n))
                    .is_some_and(|r| x < r.x + r.w * 0.5)
            })
            .unwrap_or(row.len());
        row.insert(at, panel);
        return Some((slot, row, at));
    }
    None
}

/// ⭐⭐ **A MARCA que diz ONDE a aba vai cair** — `None` quando o dedo não está sobre fila nenhuma.
///
/// ⛔⛔ **Sem ela o gesto é mudo.** As zonas de largada realçam a COLUNA, e arrastar uma aba dentro
/// da própria fila realça a coluna onde ela já está: o artista vê exactamente o mesmo antes e
/// depois de atravessar a vizinha, e larga às cegas. *Um gesto cujo resultado só se conhece depois
/// de o fazer é um gesto que se desfaz.*
///
/// ⚠️ **Ela deriva do MESMO [`tab_row_drop`] que decide** — e é essa a razão de ele devolver o
/// índice. Uma segunda aritmética de *«em que posição isto cai?»* ao lado da primeira é o defeito
/// que a fila de abas já pagou uma vez (o trilho vivia em três cópias, e um pintor horizontal com
/// um hit vertical compilava).
#[must_use]
pub fn tab_drop_caret(hero: &HeroScreen, panel: NodeId, cursor: (f32, f32)) -> Option<Rect> {
    let (slot, row, at) = tab_row_drop(hero, panel, cursor.0, cursor.1)?;
    let bar = hero.last_layout?.slot_tabs[slot as usize];
    // `row` já traz a arrastada em `at`; os vizinhos dela é que dão a coordenada.
    let x = if at + 1 < row.len() {
        hero.hit_index.rect_for(tab_node_id(row[at + 1]))?.x
    } else if at > 0 {
        let r = hero.hit_index.rect_for(tab_node_id(row[at - 1]))?;
        r.x + r.w
    } else {
        bar.x + Spacing::Xs.px()
    };
    Some(Rect::new(
        x - DROP_CARET_PX * 0.5,
        bar.y,
        DROP_CARET_PX,
        bar.h,
    ))
}

pub fn resolve_tab_drop(hero: &mut HeroScreen) {
    let Some((panel, (x, y))) = hero.store.take_tab_drop() else {
        return;
    };
    // ⭐⭐⭐ **Largar SOBRE UMA FILA DE ABAS diz duas coisas: o encaixe E o lugar na fila.**
    //
    // > *«não é possível reordenar as abas arrastando com o mouse»* — Enio, 2026-09-08.
    //
    // Até aqui a largada só sabia responder *«que encaixe?»*, então arrastar uma aba dentro da
    // própria fila era um no-op silencioso: o painel já estava naquele encaixe.
    //
    // ⚠️ **Este ramo vem PRIMEIRO** porque a fila de abas fica **por cima** da coluna: os dois
    // rects contêm o ponto, e quem está à frente do dedo é a fila. Julgar pela coluna daria o
    // resultado antigo — mover sem ordenar.
    if let Some((slot, row, _)) = tab_row_drop(hero, panel, x, y) {
        hero.store.set_panel_slot(panel, slot);
        hero.store.set_tab_row_order(&row);
        hero.store.bump_panel_z(panel);
        return;
    }
    for (slot, r) in drop_targets(hero, panel) {
        if r.contains(x, y) {
            hero.store.set_panel_slot(panel, slot);
            // ⭐ E o painel largado fica à FRENTE no encaixe novo — senão ele desaparece atrás de
            // quem já lá estava, e o artista conclui que o gesto falhou.
            hero.store.bump_panel_z(panel);
            return;
        }
    }
    // ⚠️ Largar fora de todo destino legal **não faz nada**, e não é um erro: é a forma de
    // desistir. ⛔ Nenhuma mensagem — um aviso por cada gesto abandonado seria ruído.
}

/// Pinta as zonas de largada e a etiqueta que segue o dedo. No-op sem arrasto em curso.
#[allow(clippy::too_many_arguments)]
pub fn paint_drag_overlay(
    hero: &HeroScreen,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let Some((panel, cursor)) = hero.store.tab_being_dragged() else {
        return;
    };
    // ⚠️ **O fantasma leva a CARA da aba, não só o nome** — ele é a aba a viajar, e uma etiqueta
    //    sem glifo largada sobre uma fila de glifos leria como outra coisa.
    let (title, icon) = crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.panel_node_id == panel)
            .map(|p| (p.manifest.title, p.manifest.icon))
    })
    .flatten()
    .unwrap_or(("", crate::icons::IconId::Inspector));

    for (_, r) in drop_targets(hero, panel) {
        let under = r.contains(cursor.0, cursor.1);
        let token = if under {
            ColorToken::AccentSoft
        } else {
            ColorToken::BgElev
        };
        fill_rounded_rect(
            scene,
            r,
            crate::paint::frame_radius(theme, Radius::Sm.px()),
            resolve(token, theme),
        );
        if under {
            // FRAME-RAW-OK: o contorno da aba DESTINO enquanto se arrasta outra: a mensagem, nao moldura de repouso
            crate::paint::stroke_rounded_rect(
                scene,
                r,
                crate::paint::frame_radius(theme, Radius::Sm.px()),
                DROP_OUTLINE_PX,
                resolve(ColorToken::Accent, theme),
            );
        }
    }

    // ⭐⭐ **A marca do LUGAR, por cima das zonas e por baixo da etiqueta** — ver [`tab_drop_caret`].
    // As zonas dizem *em que coluna*; esta diz *em que posição*, que é a metade que o gesto de
    // reordenar acrescentou em 2026-09-08.
    if let Some(caret) = tab_drop_caret(hero, panel, cursor) {
        scene.fill_rect(rect_to_vello(caret), resolve(ColorToken::Accent, theme));
    }

    // A etiqueta segue o dedo — é o que diz *o que* está a ser movido.
    let w = DRAG_LABEL_W;
    let ghost = Rect::new(cursor.0 - w * 0.5, cursor.1 - TAB_BAR_H * 0.5, w, TAB_BAR_H);
    fill_rounded_rect(
        scene,
        ghost,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Bg2, theme),
    );
    super::slot_tabs_face::paint(
        scene,
        text_system,
        ghost,
        icon,
        title,
        ColorToken::Text1,
        theme,
    );
}
