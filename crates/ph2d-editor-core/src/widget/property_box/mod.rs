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

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

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

/// ⭐⭐⭐ **A COLUNA DE ANIMAÇÃO está LIGADA nas linhas de formulário** (Enio, 2026-09-03:
/// *«a bolinha de animação — só desenhá-la»*).
///
/// ⚠️ **É um INDICADOR, não um controlo** — e a diferença é deliberada, não um esquecimento. Ele
/// **não regista hit nenhum**, logo não há clique a cair no vazio: um ponto que se pintasse como
/// alvo e não pusesse chave nenhuma seria um **controlo morto**, a espécie que o `CLAUDE.md` §5.0
/// caça. Ele diz *«esta propriedade é animável»*, que é verdade para todas
/// (*«nessa engine vou querer animar tudo»*), e cala-se sobre o resto.
///
/// ⚠️ **Ele CUSTA 14 px de largura em cada linha do app**, e foi por isso que ficou desligado até
/// haver decisão. A decisão é do dono e está tomada.
///
/// ⏳ Os outros estados do Blender — losango cheio (chave neste quadro), losango vazio (chave
/// noutro), ícone de driver — precisam da **timeline**, não de desenho.
pub const FORM_ROWS_SHOW_DECORATOR: bool = true;

/// **ONDE cai a coluna de animação** — a lei, com dois leitores: a caixa única e a linha de
/// verificação. ⛔ Derivada da [`surface_rect`], para que reservar e desenhar não possam divergir.
#[must_use]
pub(crate) fn decorator_rect(rect: Rect) -> Rect {
    let s = surface_rect(rect, true);
    Rect::new(s.x + s.w, rect.y, DECORATOR_W, rect.h)
}

/// ⭐⭐⭐ **A PORTA de uma linha de formulário construída à mão** — devolve a largura que sobra para
/// os controlos e **onde** pôr o ponto da coluna de animação.
///
/// ⚠️ **Ela existe porque o app tem ~20 construtores de linha à mão** (só o Inspector), cada um com
/// a sua aritmética de larguras, e a alternativa era cada um subtrair `DECORATOR_W` por sua conta.
/// ⛔ *Vinte subtracções são vinte oportunidades de a coluna ficar com um `x` diferente* — e a
/// coluna só quer dizer alguma coisa se for **uma**.
///
/// Uso, em duas linhas:
/// ```ignore
/// let (control_w, dot) = form_row_columns(x, w, row_y, row_h);
/// // …desenhe os controlos dentro de `control_w`…
/// paint_decorator_dot(scene, theme, dot);
/// ```
///
/// ⚠️ **Um ponto por LINHA, nunca por campo:** um par `X`/`Y` é *uma* propriedade com duas
/// componentes, e dois pontos diriam que são duas.
#[must_use]
pub fn form_row_columns(x: f32, w: f32, row_y: f32, row_h: f32) -> (f32, Rect) {
    // ⛔⛔ **A APARÊNCIA decide, e decide AQUI.** Report do Enio, 2026-09-03: sem
    // `PH2D_UI_NEW=1` o app *«abriu o desenho novo»* — porque os três PINTORES perguntavam a
    // aparência e as **linhas construídas à mão** não: são 19 sítios só no Inspector, e nenhum
    // deles a consultava.
    //
    // ⚠️ **A cura vai na porta, nunca nos 19** — pela mesma razão que a porta existe. E vai
    // **também** no [`paint_decorator_dot`]: um chamador que ignore a largura devolvida ainda assim
    // não pode pintar o ponto. *Duas metades, para que esquecer uma não chegue.*
    if !crate::paint::ui_is_redesign() {
        return (w.max(1.0), Rect::new(x + w, row_y, 0.0, row_h));
    }
    let control_w = (w - DECORATOR_W).max(1.0);
    (
        control_w,
        Rect::new(x + w - DECORATOR_W, row_y, DECORATOR_W, row_h),
    )
}

/// ⭐ **A porta da coluna de animação para quem NÃO usa a caixa única.**
///
/// ⚠️ Ela existe porque o app tem **três** famílias de linha de formulário, e só duas passam por
/// aqui: a caixa única e a linha de verificação. A terceira — rótulo à esquerda + campos numéricos
/// soltos, o Transform do Inspector — é construída à mão em cada painel, com a sua própria
/// aritmética de larguras. ⛔ Sem esta porta cada uma delas desenharia o seu próprio ponto, e a
/// coluna que o dono pediu ficaria com um `x` por painel.
///
/// O chamador **reserva** a coluna (encolhendo a largura das suas colunas em [`DECORATOR_W`]) e
/// chama isto com o rect dela.
pub fn paint_decorator_dot(scene: &mut VectorScene, theme: Theme, r: Rect) {
    // ⛔ A segunda metade da guarda — ver [`form_row_columns`].
    if !crate::paint::ui_is_redesign() {
        return;
    }
    paint_decorator(scene, theme, r, false);
}

/// A coluna de animação.
///
/// ⚠️ Aqui ela é só o estado «animável, sem chave» (o ponto vazio). Os outros do Blender — losango
/// cheio (chave neste quadro), losango vazio (chave noutro), ícone de driver — são trabalho a
/// seguir e precisam da **timeline**, não de desenho.
pub(crate) fn paint_decorator(scene: &mut VectorScene, theme: Theme, r: Rect, disabled: bool) {
    let d = Spacing::Xs.px();
    let dot = Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let c = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    fill_rounded_rect(scene, dot, d * 0.5, resolve(c, theme));
}

/// Trunca o rótulo para caber, com reticências.
///
/// ⚠️ Devolve string VAZIA quando nem duas letras cabem — e isso é uma resposta, não uma falha: a
/// caixa fica só com o número, que é o degrau seguinte da escada do estreito (pesquisa §6.1).
///
/// ⏳ A alternativa é o **esbatimento** (`Scene::push_luminance_mask_layer`, zero consumidores
/// hoje): em vez de `…`, o rótulo desvanece nos últimos px. É mais bonito e não come letras —
/// nomeado na pesquisa §7.3, com o custo por medir.
///
/// ⚠️ **`pub(crate)` porque a lei tem um SEGUNDO leitor desde 2026-09-03: a caixa de verificação**
/// (o widget mais usado do app, 81 sítios). *Uma lei de truncagem copiada para o vizinho é a
/// primeira linha de um formulário em que metade das linhas cede e a outra metade transborda.*
pub(crate) fn fit_label(
    text_system: &mut TextSystem,
    label: &str,
    size: f32,
    budget: f32,
) -> String {
    if budget <= 0.0 {
        return String::new();
    }
    if text_system.layout(label, size, f32::INFINITY).width() <= budget {
        return label.to_string();
    }
    let ell = "\u{2026}";
    let mut chars: Vec<char> = label.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let cand: String = chars.iter().collect::<String>() + ell;
        if text_system.layout(&cand, size, f32::INFINITY).width() <= budget {
            return cand;
        }
    }
    String::new()
}
