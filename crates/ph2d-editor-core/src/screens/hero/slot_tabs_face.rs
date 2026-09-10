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
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, INLINE_ICON_PX, Radius, Spacing, StrokeToken, Theme, TypeToken, icon_label_gap_px,
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

// ══════════════════════════════════════════════════════════════════════════════════════
// ⭐ **O TOM de uma aba** — o chão da fila, o fundo da escolhida, a quina e a divisória.
//
// ⚠️ **Vieram do [`super::slot_tabs`] em 2026-09-09, por tecto de LOC — e o corte é por
// ASSUNTO, não por tamanho:** este ficheiro já respondia *«o que uma aba mostra»*, e a cor
// dela é a mesma pergunta. O que ficou lá é *quem está na fila e onde*. Os itens continuam
// re-exportados pelo `slot_tabs`, que é a morada única da feature.
// ══════════════════════════════════════════════════════════════════════════════════════

/// ⭐⭐⭐ **A FAIXA RECUA e a aba escolhida SOBE ATÉ AO PAINEL** — os dois tons, numa porta só.
///
/// Portado de `theme_modern.cpp` (Godot 4.6, MIT): a faixa e a aba inactiva levam
/// `surface_lowest_color`; a escolhida é um **duplicado do `base_style`**, isto é, o corpo do
/// container. ⇒ nesta casa: o **chão** (que a wave 31 derivou, um degrau abaixo do painel) e o
/// **painel**.
///
/// ⛔⛔ **A faixa era `Bg1`, que é o tom do CARTÃO — 12/255 mais CLARO que o painel.** Uma faixa
/// mais clara do que a superfície em que assenta não recua: ela salta à frente, e a aba escolhida
/// não tem de onde subir. O gate [`the_tab_row_recedes_and_the_chosen_tab_rises`] mede a ordem, e
/// não os valores.
#[must_use]
pub fn tab_row_bg() -> ColorToken {
    ColorToken::WindowGround
}

/// O tom de uma aba, ou `None` para «a faixa aparece por baixo».
///
/// ⚠️ A inactiva devolve `None` **e isso É o modelo**: lá ela leva o `surface_lowest_color`, que é
/// exactamente a cor da faixa — pintá-la seria pintar o que já lá está. Ela lê-se pelo RÓTULO e
/// pela ausência de quina.
///
/// # ⭐⭐ Por que estas abas NÃO usam o acento e as de LAYOUT usam
///
/// A auditoria de 2026-09-07 nomeou a divergência e escreveu que *«uma das duas está errada e nada
/// no repo escolhe qual»*. **Escolhe agora, e as duas estão certas** — elas não são a mesma coisa:
///
/// | | [`super::layout_tabs`] | esta |
/// |---|---|---|
/// | o que a aba escolhe | a **tarefa** (Draw · Vector · Flip…) | qual painel está à frente |
/// | onde ela vive | na barra de menus, sobre **nada** | no topo de uma **coluna**, colada ao painel |
/// | como marca a escolhida | `AccentSoft`/`Accent` | veste o corpo do painel e **solda-se** a ele |
///
/// ⇒ *uma aba que assenta num container solda-se a ele; uma que escolhe um MODO não tem container a
/// que se soldar, e por isso precisa de tinta.* É a mesma lei que separa o chip activo do trilho
/// (fora do eixo do relógio) de uma linha escolhida numa lista (que sangra, sem quina).
#[must_use]
pub fn tab_bg(is_on: bool, state: ButtonState) -> Option<ColorToken> {
    if is_on {
        Some(ColorToken::PanelBg)
    } else if matches!(
        state,
        ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
    ) {
        Some(ColorToken::Bg2)
    } else {
        None
    }
}

/// ⭐⭐⭐ **A QUINA SÓ EM CIMA** — `set_corner_radius_individual(r, r, 0, 0)` do modelo.
///
/// É isto que faz de uma aba uma **aba** em vez de um botão a flutuar numa faixa: os cantos de
/// baixo quadrados **soldam-na** ao corpo do painel que começa logo abaixo. ⚠️ A porta por-canto já
/// existia — a wave 10 construiu-a para a lei do grupo do Blender ([`fill_rounded_rect_radii`]), e
/// a ordem é a do kurbo: `(cima-esq, cima-dir, baixo-dir, baixo-esq)`.
#[must_use]
pub fn tab_radii(theme: Theme) -> (f32, f32, f32, f32) {
    let r = crate::paint::frame_radius(theme, Radius::Sm.px());
    (r, r, 0.0, 0.0)
}

/// ⭐⭐⭐ **AS DIVISÓRIAS ENTRE ABAS** — o que faz uma aba encolhida continuar a LER-SE como aba.
///
/// > *«as abas … não reduzem de tamanho»* — Enio, 2026-09-08, a pedir o encolhimento; e a razão de
/// > o encolhimento ter sido revertido no dia anterior foi esta: espremidas, elas **desapareciam**.
///
/// ⛔⛔ Uma aba **inactiva não pinta fundo nenhum** ([`tab_bg`], portado do `theme_modern.cpp`), e
/// no Godot isso funciona porque as abas dele **não encolhem** — o nome inteiro é a silhueta. Aqui
/// elas encolhem, o nome elide, e sem uma marca de separação a fila vira uma tira de texto cortado
/// sobre uma cor só. *Uma aba sem nome legível e sem corpo não é uma aba: é um espaço.*
///
/// ⚠️ **Uma linha, e não um fundo.** Não há degrau disponível entre o chão da fila
/// ([`tab_row_bg`]) e o corpo do painel que a aba ESCOLHIDA veste: qualquer fundo visível para a
/// inactiva ficaria **mais claro** que a escolhida e inverteria a hierarquia. Uma divisória divide
/// sem competir.
///
/// ⚠️ **Nenhuma divisória toca a aba escolhida** — ela já tem contorno próprio (o corpo soldado ao
/// painel), e uma linha ao lado dele leria como uma segunda borda.
#[must_use]
pub fn tab_dividers(
    painted: &[(super::slot_tabs::Occupant, Rect)],
    selected: Option<NodeId>,
) -> Vec<Rect> {
    let inset = Spacing::Xs.px();
    let w = StrokeToken::Hairline.px();
    painted
        .windows(2)
        .filter(|pair| Some(pair[0].0.node) != selected && Some(pair[1].0.node) != selected)
        .map(|pair| {
            let r = pair[0].1;
            Rect::new(
                r.x + r.w - w * 0.5,
                r.y + inset,
                w,
                (r.h - inset * 2.0).max(0.0),
            )
        })
        .collect()
}
