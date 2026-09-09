//! ⭐⭐⭐ **A CARA DE UMA ABA** — o ícone, o nome, e o que sobra quando ela encolhe.
//!
//! > *«as abas … não reduzem de tamanho e colocam os `...` no nome»* — Enio, 2026-09-08.
//!
//! O encolhimento chegou nessa wave ([`super::slot_tabs::fitted_widths`]); o que ficou por
//! responder foi *o que uma aba DIZ depois de encolher*. Uma aba no piso é um quadrado de
//! [`ph2d_tokens::ROW_H_PX`] e um nome não cabe lá — sobrava um `…`, **igual em todas**.
//!
//! # ⭐ O piso já era a forma do ícone antes de haver ícone
//!
//! `22 − 14` ([`ph2d_tokens::INLINE_ICON_PX`]) dá **4 px de cada lado**, que é o
//! [`ph2d_tokens::Spacing::Xs`] com que esta casa deixa tudo respirar. *O quadrado do piso não
//! foi escolhido para receber o glifo: ele mede exactamente o glifo mais o respiro.*
//!
//! # ⛔⛔ O nome SAI, e não é elidido até ao osso
//!
//! O [`crate::text_elide::fit`] devolve o texto **CRU** quando nem as reticências cabem — e o doc
//! dele diz porquê: *«um rótulo cortado a zero é um botão mudo, e é melhor transbordar
//! visivelmente do que desaparecer»*. Numa **linha de abas** essa lei inverte-se: quem transborda
//! escreve por cima da aba vizinha, e o botão não fica mudo porque **o ícone ficou**. ⇒ aqui o
//! rótulo só é pintado quando sobrevive **pelo menos um carácter do nome** *e* o resultado **cabe**
//! no orçamento. Fora disso a aba é o glifo, centrado.
//!
//! ⚠️ **É um ficheiro irmão e não mais código no [`super::slot_tabs`]** porque as perguntas são
//! duas: aquele responde *quem está na fila e onde*, este *o que se vê dentro de uma*. O tecto de
//! LOC foi o gatilho; a fronteira é a que já lá estava.

use crate::icons::IconId;
use crate::paint::{paint_icon, paint_text, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, INLINE_ICON_PX, Spacing, StrokeToken, Theme, TypeToken, icon_label_gap_px,
};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **O recuo de uma aba, de cada lado** — `base_margin · 4` do modelo, que aqui é `Spacing::Md`.
///
/// Portado de `theme_modern.cpp` (Godot 4.6, MIT): `style_tab_selected` declara
/// `content_margin_individual(base_margin*4, base_margin*2.1, base_margin*4, base_margin*2.1)`.
/// Com `base_margin = 4` isso dá **16 px** de recuo horizontal total — exactamente o que o
/// [`crate::paint::label_budget`] desta casa já descontava, e o mesmo número que as abas de LAYOUT
/// usam ([`super::layout_tabs`]).
///
/// ⛔ **`fn` e não `const`**: `Spacing::px` não é `const fn` (a densidade é autorada).
pub fn tab_pad_x() -> f32 {
    Spacing::Md.px()
}

/// O lado do glifo de uma aba — o ícone **em linha** desta casa, o mesmo dos botões e das listas.
pub fn tab_icon_px() -> f32 {
    INLINE_ICON_PX
}

/// ⭐ **A largura NATURAL de uma aba:** recuo + glifo + vão + nome + recuo.
///
/// ⚠️ O vão vem da porta ([`ph2d_tokens::icon_label_gap_px`], `4` por veredito do dono de
/// 2026-09-07 *«para o app todo»*) — ⛔ não do recuo, que é outra grandeza e mede o dobro.
pub fn natural_w(title: &str, text_system: &mut TextSystem) -> f32 {
    let font = TypeToken::Sm.px();
    text_system.prefix_width(title, font) + tab_icon_px() + icon_label_gap_px() + tab_pad_x() * 2.0
}

/// O que uma aba de largura `r.w` mostra: o glifo sempre, o nome quando ele ainda diz alguma coisa.
pub struct TabFace {
    /// O quadrado do glifo, já centrado verticalmente.
    pub icon: Rect,
    /// O nome **já elidido** e o canto onde ele começa — `None` quando nem uma letra sobrevive.
    pub label: Option<(String, f32, f32)>,
}

/// ⭐⭐⭐ **A ÚNICA porta da geometria de dentro de uma aba** — o pintor e os gates leem daqui.
///
/// O conteúdo é um GRUPO (glifo + vão + nome) **centrado** na aba. Na largura natural isso é o
/// mesmo que encostá-lo ao recuo da esquerda; espremida, mantém-se equilibrado — que é o que o
/// `TabBar` do modelo faz.
pub fn face(r: Rect, title: &str, text_system: &mut TextSystem) -> TabFace {
    let font = TypeToken::Sm.px();
    let ico = tab_icon_px();
    let inner = (r.w - tab_pad_x() * 2.0).max(0.0);
    let budget = inner - ico - icon_label_gap_px();

    let shown = (budget > 0.0)
        .then(|| crate::text_elide::fit(text_system, title, font, budget))
        .filter(|s| keeps_a_letter(title, s))
        .filter(|s| text_system.prefix_width(s, font) <= budget);

    let (text_w, text_h) = match &shown {
        Some(s) => {
            let l = text_system.layout(s, font, f32::INFINITY);
            (l.width(), l.height())
        }
        None => (0.0, 0.0),
    };
    let group_w = match &shown {
        Some(_) => ico + icon_label_gap_px() + text_w,
        None => ico,
    };
    let x0 = r.x + (r.w - group_w) / 2.0;
    TabFace {
        icon: Rect::new(x0, r.y + (r.h - ico) / 2.0, ico, ico),
        label: shown.map(|s| {
            let x = x0 + ico + icon_label_gap_px();
            let y = r.y + (r.h - text_h) / 2.0;
            (s, x, y)
        }),
    }
}

/// **Sobrou alguma letra do nome?** — ver o cabeçalho.
///
/// ⚠️ A pergunta é feita ao PRIMEIRO carácter e não às reticências: o `fit` tem **duas** saídas
/// que não dizem nada (`…` sozinho, e o texto cru quando nem ele cabia), e a única propriedade
/// comum às saídas ÚTEIS é começarem pelo princípio do nome.
fn keeps_a_letter(title: &str, shown: &str) -> bool {
    title.chars().next().is_some_and(|c| shown.starts_with(c))
}

/// Pinta o conteúdo de uma aba. A moldura e o fundo são de quem chama.
pub fn paint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    r: Rect,
    icon: IconId,
    title: &str,
    fg: ColorToken,
    theme: Theme,
) {
    let f = face(r, title, text_system);
    let color = resolve(fg, theme);
    paint_icon(scene, icon, f.icon, color, StrokeToken::Default.px());
    if let Some((shown, x, y)) = f.label {
        paint_text(
            text_system,
            scene,
            &shown,
            x,
            y,
            TypeToken::Sm.px(),
            r.w,
            color,
        );
    }
}
