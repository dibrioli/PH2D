//! ⭐⭐⭐ **A CAIXA ÚNICA** — o pintor de uma linha de propriedade, e o padrão do app desde
//! 2026-09-02.
//!
//! Uma linha deixa de ser `rótulo | trilho | caixa numérica` (`154 px` de cromo fixo, medido em
//! `docs/UI_New_and_Simple/pesquisa/07` §2) e passa a ser **uma caixa**: rótulo à esquerda dentro,
//! valor à direita dentro, e o preenchimento a dizer a fracção.
//!
//! O desenho, o raio e a altura vêm do [`SliderStyle`](ph2d_tokens::SliderStyle) — a aparência que
//! o artista escolhe, publicada uma vez por quadro como o [`TextRendering`](ph2d_tokens::TextRendering).
//!
//! # ⚠️ Uma PORTA, não uma cópia
//!
//! Este ficheiro é o **único** sítio que sabe desenhar a caixa. O Widget Lab pinta as amostras dele
//! chamando aqui; a linha do produto passa por aqui. ⛔ *Um segundo pintor «só para a bancada» faria
//! o estudo divergir do produto sem ninguém notar* — que é literalmente o bug que criou o
//! [`slider_with_chip`](super::slider_with_chip) (*"the slider in panel X looks different from the
//! one in panel Y"*).
//!
//! # A lei do rótulo
//!
//! ⚠️ **O rótulo é o que CEDE.** Se não couber, trunca; o valor **nunca** trunca, porque um número
//! cortado é um número errado. É a inversão exacta do widget antigo, onde o rótulo tinha `70 px`
//! fixos e o trilho encolhia até desaparecer.

mod paint;
pub use paint::paint_property_box;

/// ⭐⭐⭐ A coluna do PAINEL — a união dos pedidos das secções dele (ver o topo dele).
mod coluna_do_painel;
/// ⭐ A geometria de uma LINHA de formulário — irmã por responsabilidade (ver o topo dela).
mod label;
mod row;
/// ⭐ O que a SECÇÃO declara — irmão por responsabilidade (ver o topo dele).
mod seccao;
pub use coluna_do_painel::ColunaDoPainel;
pub use label::{paint_property_label, property_label_origin};
pub use row::{
    FORM_ROWS_SHOW_DECORATOR, PropertyRow, form_row_columns, paint_decorator_dot,
    property_fields_layout, property_label_col_w, property_label_col_w_for, property_row_columns,
    property_row_columns_for,
};
pub use seccao::{Seccao, colunas_da_linha};
// ⚠️ Os três internos que o pintor da caixa e a checkbox consomem — re-exportados no MESMO
//    caminho de antes, para o corte não mudar um chamador.
pub(crate) use label::fit_label;
pub(crate) use row::{decorator_rect, paint_decorator};

use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, Role};
use ph2d_tokens::{ColorToken, Spacing};

/// Em que estado a caixa é pintada.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PropertyBoxState {
    #[default]
    Normal,
    Hovered,
    Dragging,
    Disabled,
    /// A escrever — a caixa virou campo de texto (clicar edita, como no Blender).
    Editing,
}

impl PropertyBoxState {
    pub const ALL: [PropertyBoxState; 5] = [
        PropertyBoxState::Normal,
        PropertyBoxState::Hovered,
        PropertyBoxState::Dragging,
        PropertyBoxState::Disabled,
        PropertyBoxState::Editing,
    ];

    /// O nome que aparece no ecrã (inglês — regra do app).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PropertyBoxState::Normal => "normal",
            PropertyBoxState::Hovered => "hover",
            PropertyBoxState::Dragging => "drag",
            PropertyBoxState::Disabled => "disabled",
            PropertyBoxState::Editing => "typing",
        }
    }
}

/// A largura da coluna de animação — o *decorator* do Blender.
///
/// ⭐ Enio, 2026-09-01: *"em todas as propriedades que podem ser animadas, e nessa engine vou querer
/// animar tudo"* ⇒ ela é permanente, e sai de **todas** as linhas. Por isso é medida aqui, uma vez,
/// e não escolhida em cada sítio.
pub const DECORATOR_W: f32 = 14.0; // LITERAL-PX-OK: coluna de animacao (o decorator do Blender)

/// Quantos `pad` separam o fim do rótulo do início do valor: um de cada bordo, mais **um** entre os
/// dois. ⚠️ É uma CONTAGEM de folgas, não uma medida — o px vem do `Spacing::Md`.
const PAD_UNITS_BETWEEN_LABEL_AND_VALUE: f32 = 3.0; // LITERAL-PX-OK: CONTAGEM de folgas, nao px

/// Tudo o que o pintor precisa e que **não** é geometria.
#[derive(Copy, Clone, Debug)]
pub struct PropertyBox<'a> {
    pub label: &'a str,
    /// O valor já formatado, com unidade se houver (`"0.10 m"`, `"62%"`).
    pub value: &'a str,
    /// A fracção `0..1` que o preenchimento mostra.
    pub t: f32,
    pub state: PropertyBoxState,
    /// A cor do preenchimento.
    pub accent: ColorToken,
    /// Desenha a coluna de animação à direita.
    pub decorator: bool,
    /// **A largura da coluna do valor.**
    ///
    /// - `None` — mede o texto do `value`. É o que uma AMOSTRA quer (o laboratório, a galeria).
    /// - `Some(w)` — reserva `w`. É o que uma **linha de formulário** quer, e não é conforto: com
    ///   largura medida, cada linha põe o número num `x` diferente e a coluna sai **esfarrapada**.
    ///   *Números de um formulário alinham-se, ou o olho não os compara.*
    ///
    /// ⚠️ Com `Some(w)` e `value` **vazio**, a caixa reserva e **não pinta** — quem pinta ali é o
    /// chamador, no rect que esta função devolve. É assim que o produto mete lá o campo numérico de
    /// verdade (cursor, selecção, setinhas) em vez de o reimplementar.
    pub value_w: Option<f32>,
}

impl PropertyBox<'_> {
    /// **O nó de acessibilidade da caixa** — irmão exacto do [`super::Slider::build_a11y`].
    ///
    /// ⚠️ **A caixa única é um SLIDER para quem não a vê**, e é aqui que isso fica dito: o rótulo
    /// que o vidente lê à esquerda e o valor que ele lê à direita são, para o leitor de ecrã, o
    /// `label` e o `numeric_value` do mesmo nó. ⛔ Sem isto a fusão das três colunas numa **apagava
    /// a semântica** junto com o cromo — o widget antigo tinha um nó de slider e um de campo, e
    /// perder os dois em silêncio seria o preço escondido de um redesenho que se anuncia como
    /// visual.
    ///
    /// ⏳ **BURACO NOMEADO, não fingido:** o `t` viaja como `numeric_value` (a fracção `0..1`, como
    /// no `Slider`), e o **texto** do valor — `"0.10 m"` — **não tem slot**: o nosso
    /// [`NodeBuilder`] só tem `label` e os três `numeric_value*`. ⇒ quem não vê ouve *«Speed, 62 %»*
    /// e **perde a unidade**. ⛔ Não o dobrei dentro do `label` (`"Speed 0.10 m"`) porque isso
    /// mistura duas grandezas num campo com dono, e o dia em que o builder ganhar um `value` de
    /// texto deixaria dois sítios a dizer a mesma coisa. *A cura é um campo no `ph2d-a11y`, que é
    /// foundational de outra gente e não cabe nesta wave.*
    #[must_use]
    pub fn a11y_node(&self, rect: Rect) -> Node {
        NodeBuilder::new(Role::Slider)
            .label(self.label)
            .bounds(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64)
            .focusable(self.state != PropertyBoxState::Disabled)
            .action(Action::Click)
            .numeric_value(f64::from(self.t.clamp(0.0, 1.0)))
            .numeric_value_min(0.0)
            .numeric_value_max(1.0)
            .build()
    }
}

/// ⭐⭐ **ONDE fica a coluna do valor** — a lei, num sítio só.
///
/// ⚠️ **Existe porque ela tem DOIS leitores**: o pintor (que reserva e devolve) e o
/// [`slider_with_chip_chip_rect`](super::slider_with_chip_chip_rect), que é **puro** (sem
/// `TextSystem`) e serve a quem precisa de desenhar POR CIMA do valor sem re-derivar a conta —
/// a rachura de *"um token cobre este número"*.
///
/// ⛔ *Uma segunda expressão para «onde está o valor?» divergiria no primeiro dia em que a caixa
/// ganhasse a coluna de animação, e a marca apareceria ao lado do número em vez de sobre ele.*
/// ⭐⭐⭐ **A SUPERFÍCIE da caixa — o rectângulo que o preenchimento atravessa, e portanto o
/// rectângulo que o ARRASTO tem de registar.**
///
/// ⚠️ **Ela existe porque a lei tem DOIS leitores em subsistemas diferentes, e enquanto teve um só
/// o app tinha deriva.** O pintor mapeia `t` sobre este rect (`fill_w = r.w * t`); o despacho de
/// ponteiro mapeia o cursor sobre o rect que o chamador REGISTOU no `HitIndex`
/// (`value = (px − rect.x) / rect.w`, em
/// [`interaction::dispatch::number_input`](crate::interaction)). Se os dois rects não forem o
/// **mesmo**, a tinta e o dedo escalam por factores diferentes: o preenchimento afasta-se do cursor
/// **proporcionalmente à distância da borda esquerda** — ⛔ não é uma folga constante que se
/// compense com um `+ pad`, é um **factor**, e por isso lê-se como *offset* perto do fim e como
/// *drift* ao longo do curso.
///
/// Medido em 2026-09-03, report do Enio (*«temos um offset e drift em relação ao cursor»*):
///
/// | sítio | registava | pintava | factor |
/// |---|---|---|---|
/// | linha do produto | caixa **menos** a coluna do valor | caixa inteira | `w/(w−pad−chip_w)` = **1,62×** a `w = 220` |
/// | bancada, *decorator* ligado | caixa **com** a coluna de animação | caixa **sem** ela | `w/(w−14)` = 1,07× |
///
/// ⚠️ **A mesma lei partida, dos dois lados opostos** — e é por isso que o defeito só apareceu no
/// produto: `84 px` de discrepância num sítio contra `14` no outro. *Duas contas que hoje
/// concordam são duas contas que amanhã divergem; aqui elas já divergiam e ninguém tinha um sítio
/// onde comparar.*
#[must_use]
pub fn surface_rect(rect: Rect, decorator: bool) -> Rect {
    if decorator {
        Rect::new(rect.x, rect.y, (rect.w - DECORATOR_W).max(1.0), rect.h)
    } else {
        rect
    }
}

#[must_use]
pub fn value_column(rect: Rect, value_w: f32, decorator: bool) -> Rect {
    let pad = Spacing::Md.px();
    let right = rect.x + rect.w - if decorator { DECORATOR_W } else { 0.0 };
    let vx = (right - pad - value_w).max(rect.x);
    Rect::new(vx, rect.y, (right - vx).max(1.0), rect.h)
}

/// ⭐⭐⭐ **A OUTRA METADE da mesma repartição: onde o NOME da caixa única é pintado.**
///
/// ⛔⛔ **Ela existe porque a lei tinha UM leitor e a pergunta tem DOIS** — exactamente o mecanismo
/// que a [`surface_rect`] paga um bloco acima, um nível abaixo. O `x` do rótulo (`box.x + pad`) e o
/// orçamento dele (`box.w − 3·pad − value_w`) viviam **dentro** do [`super::paint::paint_property_box`],
/// logo só existiam para quem PINTA uma caixa. Todo painel que desenha uma linha **que não é uma
/// caixa** — um facto travado, uma amostra de cor — tinha de escolher outra coluna, e escolhia
/// outra: medido no painel de modelagem 3D em 2026-09-19, ele tinha **três** alinhamentos de rótulo
/// ao mesmo tempo (a linha viva, a amostra e a travada).
///
/// ⚠️ **É o espelho exacto da [`value_column`]** — mesmos argumentos, mesma pureza, mesmo `pad`. Ela
/// devolve o rect em que o rótulo **cabe**; quem pinta continua a elidir dentro dele.
///
/// ⚠️ **O `value_w` é o da caixa, não o do rótulo.** As duas portas leem o MESMO número, e é isso
/// que faz o vão entre elas ser sempre um `pad`: sem esse argumento aqui, o orçamento do nome seria
/// uma segunda conta sobre a largura do valor, e as duas divergiriam no dia em que a coluna do valor
/// deixasse de ser o piso do campo.
#[must_use]
pub fn label_column(rect: Rect, value_w: f32, decorator: bool) -> Rect {
    let pad = Spacing::Md.px();
    let box_rect = surface_rect(rect, decorator);
    let budget = (box_rect.w - pad * PAD_UNITS_BETWEEN_LABEL_AND_VALUE - value_w).max(0.0);
    Rect::new(box_rect.x + pad, rect.y, budget, rect.h)
}
