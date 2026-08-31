//! ⭐⭐⭐ **O CABEÇALHO DA ÁREA** — a metade 2 da decisão **D2**, e a última região obrigatória do
//! modelo de áreas.
//!
//! > *«Barra global para o que é do aplicativo inteiro (Arquivo, Editar, Ajuda); cabeçalho por
//! > área para o que é da ferramenta.»* — D2
//!
//! # ⭐ O corte é por ÂMBITO, e é ele que dá um sítio canónico a cada comando
//!
//! Um comando que vale em **todo o app** vai à barra de cima; um que vale **só naquele editor**
//! vem para aqui. É a cura da *foto 3* (o painel de propriedades como depósito por omissão): sem
//! este sítio, um comando do editor não tinha para onde ir senão um painel.
//!
//! ⇒ os dois primeiros inquilinos **não são construídos, são realojados**, e cada um sai de um
//! menu cujo âmbito é o app inteiro:
//!
//! | comando | morava em | âmbito real |
//! |---|---|---|
//! | **Rulers** | menu *View* (app) | as réguas **desta** área de desenho |
//! | **Statistics** | menu *Look* (tema/chrome) | o HUD **desta** área |
//!
//! ⛔ **Eles SAEM de lá**, não ficam nos dois sítios: *«existe UM sítio canónico para cada
//! comando»* é a frase da D2, e uma entrada repetida em dois menus é a tabela paralela outra vez —
//! com o sintoma pior, os dois estados a discordar à vista.
//!
//! # ⚠️ É uma REGIÃO, com tudo o que isso obriga
//!
//! Ela sai da **área** (entre as colunas), como a fila de ferramentas e a régua, e **subtrai**
//! altura em vez de flutuar. A ordem, de cima para baixo, é a do HIG do Blender: cabeçalho →
//! ferramentas → régua → conteúdo. ⛔ *Uma faixa que continuasse a flutuar reproduziria, num
//! modelo novo, o defeito que o modelo existe para curar* (`spec/01 §4`).
//!
//! # ⚠️ O `active` sai da MESMA tabela de verdade dos menus
//!
//! [`super::menu_bar::module_is_on`] — nunca de um estado de botão local. Uma segunda ideia de
//! *«isto está ligado?»* é o defeito que a entrega 18 já pagou: os três activadores de ferramenta
//! perguntavam ao `store.button_state(…)`, que ninguém escrevia.

use super::HeroScreen;
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **As opções de exibição desta área** — o item 5 da ordem do HIG (*«à direita, opções de
/// exibição»*).
///
/// ⚠️ **Cada uma leva o id que já existia**, e por isso o handler é o mesmo: não há porta nova.
/// O `Rulers` é despachado pelo `chrome::menu_bar` e o `Statistics` pelo `chrome::view_toggles`.
pub const DISPLAY_OPTIONS: [(NodeId, &str); 2] = [
    (crate::ids::MENUBAR_VIEW_RULERS, "Rulers"),
    (crate::ids::CTX_MENU_SHOW_STATS, "Stats"),
];

/// A altura da faixa — uma linha de controlo, mais o respiro de cima e de baixo.
///
/// ⚠️ **Derivada**, pela razão do [`super::tool_bar::tool_bar_h`]: um número próprio aqui faria o
/// cabeçalho e a fila terem alturas de linha diferentes no dia em que o token mudasse.
#[must_use]
pub fn area_header_h() -> f32 {
    ROW_H_PX + Spacing::Xxs.px() * 2.0
}

/// A largura de um controlo, pelo rótulo dele.
fn chip_w(label: &str, text_system: &mut TextSystem) -> f32 {
    text_system.prefix_width(label, TypeToken::Sm.px()) + Spacing::Md.px() * 2.0
}

/// ⭐ **A ÚNICA porta da geometria** — o pintor e o registo de hit leem daqui.
///
/// As opções são encostadas à **direita** da faixa, que é onde o HIG as põe e onde a mão as
/// procura. Devolve vazio se a área for estreita demais para as conter.
#[must_use]
pub fn option_rects(band: Rect, text_system: &mut TextSystem) -> Vec<(NodeId, &'static str, Rect)> {
    let widths: Vec<f32> = DISPLAY_OPTIONS
        .iter()
        .map(|(_, l)| chip_w(l, text_system))
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + Spacing::Xs.px() * (widths.len() - 1) as f32;
    let mut x = band.x + band.w - total - Spacing::Sm.px();
    if x < band.x {
        return Vec::new();
    }
    let y = band.y + Spacing::Xxs.px();
    DISPLAY_OPTIONS
        .iter()
        .zip(widths)
        .map(|((id, label), w)| {
            let r = Rect::new(x, y, w, ROW_H_PX);
            x += w + Spacing::Xs.px();
            (*id, *label, r)
        })
        .collect()
}

// ⛔⛔ **Não há `populate` aqui, e a ausência foi MEDIDA.** A 1.ª versão registava os dois ids — e
// a prova de mutação que o apagou ficou **verde**: eles já eram registados pelo `pre_populate` do
// hero, que os conhece como linhas de menu desde antes desta faixa existir. *Um `register` a mais
// não falha; ele só faz o gate mentir sobre quem mantém o controlo vivo.*
//
// ⚠️ **E o dono do `ButtonState` deles é outro:** o `menu_bar::publish_toggle_state` reescreve-o
// **em todo quadro** com `Pressed`/`Normal` a partir da tabela de verdade. Ver `paint`.

/// ⭐ **Quais opções estão LIGADAS** — resolvido contra a tabela de verdade dos menus.
///
/// ⚠️ Separado do [`paint`] por uma razão de empréstimo e uma de desenho: o pintor precisa de
/// `&mut hero.hit_index`, e a verdade precisa de `&HeroScreen` inteiro. Resolvê-la antes mantém a
/// **única** fonte (`menu_bar::module_is_on`) em vez de um estado de botão local.
#[must_use]
pub fn option_states(hero: &HeroScreen) -> [bool; DISPLAY_OPTIONS.len()] {
    DISPLAY_OPTIONS.map(|(id, _)| super::menu_bar::module_is_on(hero, id).unwrap_or(false))
}

/// Pinta a faixa e regista os alvos.
pub fn paint(
    band: Rect,
    states: [bool; DISPLAY_OPTIONS.len()],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    if band.w <= 0.0 || band.h <= 0.0 {
        return;
    }
    fill_rounded_rect(scene, band, 0.0, resolve(ColorToken::Bg2, theme));
    for (i, (id, label, r)) in option_rects(band, text_system).into_iter().enumerate() {
        // ⚠️ **A verdade vem da tabela dos menus**, nunca do estado do botão — ver o cabeçalho.
        let on = states[i];
        // ⛔⛔ **Não há realce de HOVER, e não é esquecimento: ele não podia disparar.** A 1.ª
        // versão lia `store.button_state(id)` — e para estes ids esse campo **não significa «sob o
        // rato»**: o `menu_bar::publish_toggle_state` reescreve-o em todo quadro com
        // `Pressed`/`Normal` tirado da tabela de verdade, e escreve **depois**. *Dois significados
        // no mesmo campo, e ganha quem escreve por último.*
        //
        // ⇒ o único estado que a faixa mostra é o que ela de facto sabe: **ligado**. Um realce que
        // nunca acende é o controlo morto pintado, que é pior do que a sua ausência.
        if on {
            fill_rounded_rect(
                scene,
                r,
                Radius::Sm.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
        }
        let fg = if on {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        };
        paint_text_centered(
            text_system,
            scene,
            label,
            r,
            TypeToken::Sm.px(),
            resolve(fg, theme),
        );
        hit_index.register(id, r);
    }
}

#[cfg(test)]
#[path = "area_header_tests.rs"]
mod tests;
