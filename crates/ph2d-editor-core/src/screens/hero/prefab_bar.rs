//! ⭐⭐⭐ **A BARRA DO MODO DE RECEITA** — *«está a editar o prefab X; N cópias seguem; [Done]»*.
//!
//! # O defeito que ela cura
//!
//! O *Edit Prefab* põe o app num MODO: o mundo vai para trás de um vidro jateado, a receita sobe ao
//! palco, e cada peça mexida chega a todas as cópias. ⚠️ **E o único sinal disso era o borrão** — o
//! nome da receita vinha num *toast* que dura três segundos, e a saída era **adivinhar** que clicar
//! no vazio desfaz a selecção.
//!
//! *Um modo em que se entra por um verbo e de que se sai por acidente é um modo sem saída* — e é o
//! mesmo defeito que este repo já nomeou no pill que muda o dono do ponteiro
//! ([`super::chrome::model3d_toggle`], que documenta precisamente ter de ter *«uma saída visível»*).
//!
//! # As três coisas que ela diz, e porque são estas três
//!
//! | o que | porquê |
//! |---|---|
//! | o **nome** da receita | o artista abre uma cópia e a receita pode chamar-se outra coisa; sem o nome ele não sabe o que está a mexer |
//! | **quantas cópias seguem** | é a promessa do modo, e a única coisa que o distingue de editar um objecto qualquer. `0` diz *«ninguém segue ainda»*, que é informação e não erro |
//! | **Done** | a saída, e a razão de a barra existir |
//!
//! # ⚠️ Ela não guarda estado nenhum
//!
//! A barra é **derivada**: a shell publica o que ela mostra a cada quadro
//! ([`super::HeroScreen::set_prefab_edit`]), e `None` = não há modo. Um bool próprio aqui seria a
//! segunda porta que diverge — a barra a dizer *«a editar»* sobre um canvas que já fechou.
//!
//! ⚠️ **E o clique não emite acção nenhuma no barramento**: fechar o modo é **largar a selecção**, e
//! a selecção é da `HeroScreen`. O `MasterEditing` é derivado dela ⇒ o mundo, o vidro e o palco
//! desfazem-se sozinhos no quadro seguinte. *Uma acção nova seria um segundo caminho para o mesmo
//! facto.*

use super::HeroScreen;
use crate::ids;
use crate::interaction::{HitIndex, WidgetEvent};
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O que a barra mostra — publicado pela shell a cada quadro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabEditView {
    /// O nome da receita aberta.
    pub name: String,
    /// Quantas cópias dela existem na cena.
    pub copies: usize,
}

/// A largura do botão de saída. ⚠️ Fixa e não derivada do texto: a barra inteira é centrada, e um
/// botão que encolhesse com o rótulo faria a barra **saltar** de largura entre receitas.
const DONE_W: f32 = 72.0; // LITERAL-PX-OK: largura do botão de saída, entre `Spacing::Xl4` (48) e o dobro dela

/// A largura mínima da barra — abaixo disto o nome de uma receita curta ficaria colado ao botão.
const MIN_W: f32 = 260.0; // LITERAL-PX-OK: largura mínima da barra (≈ 5,4 × `Spacing::Xl4`)

/// **Onde a barra fica** — no topo da área de desenho, centrada.
///
/// ⚠️ **Centrada e não de bordo a bordo:** uma faixa de largura inteira lê-se como chrome
/// PERMANENTE, e esta existe durante um modo. ⛔ E ela não pode ancorar na janela: o canvas é
/// *full-bleed*, então o topo da JANELA está debaixo da barra de menus.
#[must_use]
pub fn bar_rect(draw_area: Rect, text: &mut TextSystem, view: &PrefabEditView) -> Rect {
    let label = title(view);
    let w = (text_w(text, &label) + DONE_W + Spacing::Xl2.px() * 2.0).max(MIN_W);
    // ⚠️ **A altura é o PASSO DE UMA LINHA** (`row_pitch_px`), e não `ROW_H + um espaçamento
    // escolhido aqui`: a barra é uma linha com a respiração dela, e essa pergunta já tem uma
    // resposta na casa — escrevê-la no sítio da pintura é a segunda (e há gate a dizê-lo).
    let h = ph2d_tokens::row_pitch_px();
    Rect::new(
        draw_area.x + (draw_area.w - w) * 0.5,
        draw_area.y + Spacing::Lg.px(),
        w,
        h,
    )
}

/// O rectângulo do botão de saída, dentro da barra.
#[must_use]
pub fn done_rect(bar: Rect) -> Rect {
    let inset = Spacing::Xxs.px();
    Rect::new(
        bar.x + bar.w - DONE_W - inset,
        bar.y + inset,
        DONE_W,
        bar.h - inset - inset,
    )
}

/// ⭐ **A frase, montada num sítio só** — o pintor e a medida da largura leem a MESMA.
///
/// ⚠️ **O plural é escolhido, não concatenado:** *«1 copies»* é o erro que toda contagem de UI
/// comete uma vez, e ele lê-se como um defeito do app inteiro.
#[must_use]
pub fn title(view: &PrefabEditView) -> String {
    let follows = match view.copies {
        0 => "no copies yet".to_string(),
        1 => "1 copy follows".to_string(),
        n => format!("{n} copies follow"),
    };
    format!(
        "Editing prefab \u{201c}{}\u{201d} \u{2014} {follows}",
        view.name
    )
}

/// ⭐⭐ **Pinta a barra e regista o botão.**
///
/// ⛔ Sem receita aberta (`None`) nada é pintado e nada é registado — o quadro comum não paga um
/// rectângulo nem um hit-rect.
pub fn paint(
    scene: &mut VectorScene,
    draw_area: Rect,
    view: &PrefabEditView,
    theme: Theme,
    text: &mut TextSystem,
    hit_index: &mut HitIndex,
) {
    let bar = bar_rect(draw_area, text, view);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, bar, radius, resolve(ColorToken::BgElev, theme));
    // ⚠️ **A moldura passa pela porta do TEMA** (`stroke_frame`), e não por um traço directo: num
    // tema moderno a pele plana apaga contornos, e um traço à mão desenharia justamente o que ela
    // apagou. ⚠️ O `Feel` é **`Active`**: a barra não está em repouso — ela É o estado.
    crate::paint::stroke_frame(
        scene,
        bar,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Active,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Accent, theme),
    );
    let done = done_rect(bar);
    // O texto ocupa o que sobra à esquerda do botão.
    let label_rect = Rect::new(
        bar.x + Spacing::Lg.px(),
        bar.y,
        (done.x - bar.x - Spacing::Lg.px() * 2.0).max(0.0),
        bar.h,
    );
    paint_text_centered(
        text,
        scene,
        &title(view),
        label_rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    fill_rounded_rect(
        scene,
        done,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Accent, theme),
    );
    paint_text_centered(
        text,
        scene,
        "Done",
        done,
        TypeToken::Sm.px(),
        resolve(ColorToken::AccentFg, theme),
    );
    hit_index.register(ids::PREFAB_EDIT_DONE, done);
}

/// ⭐⭐⭐ **Sair do modo é LARGAR A SELECÇÃO** — e mais nada.
///
/// ⚠️ Chamado pelo irmão em `chrome/prefab_bar.rs`, que é quem entra na cadeia gerada do
/// [`super::chrome::dispatch_all`].
///
/// O `MasterEditing` é derivado dela, então o vidro, o palco e a receita na cena desfazem-se no
/// quadro seguinte, pela mesma lei que os montou. ⛔ Uma acção própria no barramento seria um
/// segundo caminho para o mesmo facto, e o dia em que um deles ganhasse um filtro elas divergiriam.
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::PREFAB_EDIT_DONE {
        return false;
    }
    hero.gizmo.replace_selection(None);
    true
}

/// A largura que o texto ocupa.
///
/// ⚠️ **O mesmo tamanho e o mesmo peso com que ele é PINTADO** — medir num peso e pintar noutro faz
/// o pintor cortar o texto, e o sintoma é um rótulo que some ao afastar (memória desta casa).
fn text_w(text: &mut TextSystem, s: &str) -> f32 {
    text.prefix_width(s, TypeToken::Sm.px())
}

#[cfg(test)]
#[path = "prefab_bar_tests.rs"]
mod tests;
