//! **O PINTOR de uma marca booleana** — a caixa de verificação e o interruptor, um desenho só.
//!
//! ⚠️ Este ficheiro nasceu de um **tecto de LOC**: o `checkbox.rs` passou os `500` quando a coluna
//! de animação entrou. O corte é por **responsabilidade**, não por linha: aqui mora *como um
//! booleano se desenha*; ao lado (`mod.rs`) mora *o que um booleano É* — estado, valor, construtor
//! e o nó de acessibilidade. ⛔ A casa não tolera folga de tecto: parte-se para um irmão.

use super::{CHECKBOX_BOX_PX, CheckboxState, CheckboxValue};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, resolve};

/// O repouso e o quente do eixo do hover, no vocabulário da porta da tinta.
const FEEL_REST: ph2d_tokens::visuals::Feel = ph2d_tokens::visuals::Feel::Rest;
const FEEL_HOT: ph2d_tokens::visuals::Feel = ph2d_tokens::visuals::Feel::Hovered;

/// **Como uma caixa se sente** — o [`CheckboxState`] no vocabulário das portas do tema.
/// Uma porta, dois leitores (a tinta do corpo e a moldura), para nunca discordarem.
fn feel_of_state(state: CheckboxState) -> ph2d_tokens::visuals::Feel {
    use ph2d_tokens::visuals::Feel;
    match state {
        CheckboxState::Disabled => Feel::Disabled,
        CheckboxState::Focused => Feel::Focused,
        CheckboxState::Hovered => Feel::Hovered,
        CheckboxState::Pressed => Feel::Active,
        CheckboxState::Normal => Feel::Rest,
    }
}
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **O pintor de uma MARCA BOOLEANA — e desde 2026-09-03 há um só no app.**
///
/// Pinta a superfície da linha (que acende) e a caixa com o glifo, **encostada à direita**.
/// Devolve o rectângulo da caixa, para quem quiser desenhar à volta dela.
///
/// ⚠️ **Existe porque o INTERRUPTOR DESLIZANTE saiu** (decisão do Enio: *«as pílulas e o
/// interruptor deslizante podem sair»*). ⛔ Ele **não se apaga**: o `WidgetKind::Toggle` tem
/// `code() == 2` e esse número **viaja em documento** (`skin/kind.rs`), então um painel autorado
/// gravado ontem tem de continuar a abrir. ⇒ a fusão é **de PINTURA**, exactamente como a pesquisa
/// `07` §5.5 mandava: *o código velho fica a apontar para o pintor novo*.
///
/// ⚠️ **A semântica NÃO se funde:** o `Toggle` continua a ser `Role::Switch` para quem não vê, e o
/// `Checkbox` continua `Role::CheckBox`. *Fundir a tinta de dois controlos não os torna o mesmo
/// controlo* — e um leitor de ecrã que passasse a anunciar «caixa de verificação» onde o documento
/// diz «interruptor» estaria a mentir sobre o modelo.
/// O que a marca precisa de saber, e que **não** é geometria — o irmão exacto do
/// [`crate::widget::PropertyBox`].
///
/// ⚠️ Ele existe porque o clippy contou **oito** argumentos: *«too many arguments»* não é um limite
/// de estilo, é a pergunta *«estes parâmetros não serão um modelo?»* — e cinco destes descrevem o
/// mesmo booleano. O `Checkbox` e o `Toggle` preenchem-no cada um à sua maneira, que é o que os
/// mantém dois widgets com um pintor só.
#[derive(Copy, Clone, Debug)]
pub(crate) struct BooleanMark {
    pub value: CheckboxValue,
    pub state: CheckboxState,
    pub hover_t: f32,
    /// A aresta da caixa. `None` = o token — ver [`Checkbox::box_px`].
    pub box_px: Option<f32>,
    /// Reserva e desenha a coluna de animação. Ver [`Checkbox::decorator`].
    pub decorator: bool,
    /// ⭐⭐⭐ **A linha de formulário a que esta marca pertence** — ver [`Checkbox::seccao`].
    ///
    /// `Some(sec)` faz a marca viver **dentro de uma CAIXA** que ocupa a coluna do controlo;
    /// `None` é a pele de canvas e o interruptor, que trazem o rect já do tamanho que querem.
    pub linha: Option<crate::widget::Seccao>,
}

/// ⭐⭐ **O que o pintor da marca DEVOLVE** — a marca e, quando a linha é de formulário, a CAIXA.
///
/// ⚠️ **A caixa vem daqui e não é re-derivada pelo chamador:** quem escreve a palavra do valor ao
/// lado da marca ([`super::label`]) precisa de saber onde ela acaba, e uma segunda chamada à
/// `colunas_da_linha` seria a segunda resposta a *«onde é que este campo acaba?»* — o defeito que a
/// `surface_rect` existe para impedir, um nível abaixo.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MarcaPintada {
    pub marca: Rect,
    pub caixa: Option<Rect>,
}

pub(crate) fn paint_boolean_mark(
    rect: Rect,
    m: BooleanMark,
    scene: &mut VectorScene,
    theme: Theme,
) -> MarcaPintada {
    let BooleanMark {
        value,
        state,
        hover_t,
        box_px,
        decorator,
        linha,
    } = m;
    // A moldura é o TETO em qualquer dos dois casos (a caixa não transborda o que a contém);
    // o que `box_px` troca é a BASE — o token, ou o que o chamador mediu. `None` reduz à
    // expressão que shipava, ao bit.
    let box_size = box_px.unwrap_or(CHECKBOX_BOX_PX).min(rect.h);
    let box_y = rect.y + (rect.h - box_size) * 0.5;

    // ⭐⭐⭐ **A LINHA INTEIRA É O ALVO, e a marca vai para a DIREITA** (Enio, redesenho de
    // 2026-09; pesquisa `07` §5.2 e §15). O padrão é o do Blender/GNOME, e a razão aqui é o
    // ALINHAMENTO: a marca cai na **mesma coluna** em que a linha de propriedade põe o número
    // (`property_box::value_column`), então um formulário passa a ter **uma** margem direita em vez
    // de duas linguagens empilhadas.
    //
    // ⭐ **Isto é só PINTURA — nenhum chamador muda.** Medido em 2026-09-03: 17 de 19 chamadores
    // amostrados já registam `Rect::new(x, y, w, h)` — *a linha inteira já era o alvo de clique há
    // muito tempo; o que faltava era o desenho dizê-lo.* Os 2 restantes passam meia-linha (dois
    // checkboxes lado a lado), e encostar à direita da meia-linha é igualmente correcto.
    // ⚠️ **A coluna de animação vale para a linha de checkbox também** — senão o alinhamento que a
    // §16.1 comprou desfaz-se: as linhas de propriedade recuariam `14 px` e as de marcar não, e o
    // formulário voltaria a ter duas margens direitas.
    //
    // ⭐ **Quem NÃO a leva é a pele de canvas** — ali a moldura é o que o *artista* desenhou, não
    // uma linha de formulário, e uma bolinha de animação não significa nada. Ela di-lo por um campo
    // PRÓPRIO: ⛔ a 1.ª tentativa derivava-o de `box_px.is_none()` e partia o contrato daquele
    // campo (*«pedir o token é igual a não pedir nada»*), apanhada pelo gate na 1.ª corrida.
    // ⭐⭐⭐ **A APARÊNCIA escolhe a âncora** (Enio, 2026-09-03: *«por enquanto permanece a
    // antiga»*). No clássico a marca fica em `rect.x` e não há coluna de animação — é a linha que
    // o app pinta desde sempre, e é o caminho de OMISSÃO.
    let redesign = crate::paint::ui_is_redesign();
    let decorator = decorator && redesign;
    // ⭐⭐⭐ **A MARCA VIVE DENTRO DE UMA CAIXA, encostada à ESQUERDA dela** — ordem do dono,
    // 2026-09-15, com a foto do inspector do **Godot**: *«coloca um box em todo o lado direito da
    // linha e dentro do box o checkbox alinhado à esquerda. Vamos adotar essa aparência»*.
    //
    // ⭐⭐ **E com isso a linha de marcar deixa de ser a excepção do §3 do manual:** a caixa ocupa
    // a coluna do CONTROLO — começa no meio da linha e acaba na margem direita, exactamente como a
    // caixa de um número. *A «uma margem direita» que a âncora à direita comprava passa a sair da
    // própria caixa, e o meio da linha passa a valer também aqui.*
    //
    // ⚠️ A superfície é a PORTA do campo ([`crate::widget::paint_field_surface`]) — ⛔ nunca uma
    // cópia das quatro linhas que o `paint_number_input_with_buffer` escreve.
    let caixa = match (redesign, linha) {
        (true, Some(sec)) => Some(
            crate::widget::property_box::colunas_da_linha(rect.x, rect.w, rect.y, rect.h, sec)
                .control,
        ),
        _ => None,
    };
    let box_rect = match (redesign, caixa) {
        (true, Some(campo)) => {
            // ⛔⛔ **Report do dono, 2026-09-15, com foto:** *«checkbox ficou maior que a caixa e
            // não foi bem alinhado à esquerda. Godot melhor»*.
            //
            // ⚠️ **Duas causas, e a primeira era aritmética:** a marca vale
            // [`CHECKBOX_BOX_PX`] = `18` e as dez secções do Inspector davam à linha de marcar
            // **`18` de altura** (o mesmo literal, copiado dez vezes) ⇒ `min(18, 18) = 18`: a
            // marca ocupava a caixa TODA, e com o traço da moldura por cima lia-se **maior** do
            // que ela. Hoje a linha mede [`ph2d_tokens::ROW_H_PX`], como toda linha de
            // propriedade, e a marca é **inset um degrau em cada lado**.
            //
            // ⚠️ **E o recuo NÃO é o do texto.** Ele era o [`crate::widget::field_pad_x`] (`12`),
            // que é onde o VALOR de um campo começa — e o valor precisa de folga para o caret e
            // para a selecção. *Uma marca não tem caret*: `12` lia-se como «não está à esquerda».
            let recuo = Spacing::Xs.px();
            let lado = box_size.min(campo.h - 2.0 * recuo).max(1.0);
            Rect::new(
                campo.x + recuo,
                campo.y + (campo.h - lado) * 0.5,
                lado,
                lado,
            )
        }
        (true, None) => Rect::new(
            crate::widget::property_box::value_column(rect, box_size, decorator).x,
            box_y,
            box_size,
            box_size,
        ),
        (false, _) => Rect::new(rect.x, box_y, box_size, box_size),
    };

    // ⭐ **A superfície da linha ACENDE**, e é ela que ensina que o alvo é largo. Emerge do nada
    // (o `hover_axis` com repouso `None` faz *fade*, como o botão *ghost*) ⇒ em repouso a linha
    // continua a não desenhar caixa nenhuma, que é a lei do §5.3: *dentro de um painel, molduras
    // não se desenham*.
    let row_hot = redesign
        && matches!(
            state,
            CheckboxState::Hovered | CheckboxState::Focused | CheckboxState::Pressed
        );
    if row_hot || (redesign && hover_t < crate::motion::SETTLED) {
        if let Some(c) = crate::motion::hover_axis(
            matches!(state, CheckboxState::Normal | CheckboxState::Hovered),
            hover_t,
            None,
            Some(ColorToken::Bg2.resolve(theme)),
        ) {
            fill_rounded_rect(
                scene,
                rect,
                crate::paint::slider_style().radius_px(),
                crate::paint::token_to_vello(c),
            );
        } else if row_hot {
            fill_rounded_rect(
                scene,
                rect,
                crate::paint::slider_style().radius_px(),
                resolve(ColorToken::Bg2, theme),
            );
        }
    }

    // ⭐⭐⭐ **A CAIXA** — ver o bloco da geometria acima. Ela vem depois do realce da linha (que é
    //    o fundo) e antes da marca (que assenta nela).
    if let Some(campo) = caixa {
        crate::widget::paint_field_surface(
            scene,
            campo,
            match state {
                CheckboxState::Disabled => crate::widget::TextInputState::Disabled,
                CheckboxState::Focused => crate::widget::TextInputState::Focused,
                // ⚠️ **`Pressed` lê-se como `Hovered` na CAIXA**, e a diferença fica na marca: um
                //    campo não tem estado «carregado» (ver o `TextInputState`), e inventar um aqui
                //    poria a linha de marcar a responder ao dedo de uma maneira que nenhum outro
                //    campo do app responde.
                CheckboxState::Hovered | CheckboxState::Pressed => {
                    crate::widget::TextInputState::Hovered
                }
                CheckboxState::Normal => crate::widget::TextInputState::Normal,
            },
            hover_t,
            theme,
        );
    }

    let radius = crate::paint::frame_radius(theme, Radius::Xs.px());
    let (bg_token, border_token) = match (state, value) {
        (CheckboxState::Disabled, _) => (ColorToken::Bg2, ColorToken::Border),
        (_, CheckboxValue::Checked | CheckboxValue::Indeterminate) => {
            (ColorToken::Accent, ColorToken::Accent)
        }
        (CheckboxState::Hovered | CheckboxState::Focused, _) => {
            (ColorToken::Bg2, ColorToken::BorderEmph)
        }
        _ => (ColorToken::Bg1, ColorToken::Border),
    };
    // ⚠️ **O eixo do hover é o par NÃO-MARCADO** (`Bg1 → Bg2`, `Border → BorderEmph`): uma caixa
    //    MARCADA é `Accent` em qualquer estado, então ali não há eixo nenhum a percorrer. O
    //    `Focused` fica de fora com o `Disabled` — é estado duro, e o traço dele mede 2 px.
    let soft = matches!(state, CheckboxState::Normal | CheckboxState::Hovered)
        && matches!(value, CheckboxValue::Unchecked);
    // ⭐⭐⭐ **A tinta do corpo é do TEMA, pela porta** — report do dono, 2026-09-14, com foto:
    //    *«Checkbox invisível»*. A caixa DESMARCADA enchia `Bg1`, que é a cor do cartão em que ela
    //    assenta, e num tema moderno não há moldura de repouso: ela não existia. A marcada lia-se
    //    porque é `Accent`, e é por isso que o report é sobre metade das caixas do painel.
    //
    // ⛔⛔⛔ **E em 2026-09-15 o dono reportou-a OUTRA VEZ** (*«O Checkbox desmarcado é
    //    invisível»*) — porque a wave da véspera **mudou a superfície debaixo da marca**: ela
    //    deixou de assentar no cartão e passou a assentar numa CAIXA DE CAMPO, cujo fundo é
    //    exactamente o que a [`crate::paint::body_fill`] devolve em repouso. *O mesmo pixel duas
    //    vezes*, `0`/255 nos três temas modernos sem moldura.
    //
    // ⇒ **a porta segue a SUPERFÍCIE, e é a mesma `caixa` que decidiu a geometria que a escolhe:**
    //    dentro de um campo, [`crate::paint::on_field_fill`]; fora dele (a pele de canvas e a
    //    aparência clássica), a de sempre. ⚠️ *Uma peça que se move de superfície tem de trocar de
    //    régua de contraste, senão herda a calibração da superfície antiga* (`CLAUDE.md` §0.0).
    let corpo = |feel, classic| {
        if caixa.is_some() {
            crate::paint::on_field_fill(theme, feel, classic)
        } else {
            crate::paint::body_fill(theme, feel, classic)
        }
    };
    // ⚠️ **O eixo do hover SOBREVIVE** porque a porta responde ao `Feel`: afundar só o repouso
    //    deixaria as duas pontas iguais e o rato deixaria de dizer nada.
    let bg = crate::motion::hover_axis(
        soft,
        hover_t,
        Some(corpo(FEEL_REST, ColorToken::Bg1)),
        Some(corpo(FEEL_HOT, ColorToken::Bg2)),
    )
    .map_or_else(
        || {
            crate::paint::token_to_vello(if bg_token == ColorToken::Accent {
                // ⛔ Uma caixa MARCADA é acento cheio em qualquer aparência — ela não é um corpo
                //    afundado, é a resposta.
                ColorToken::Accent.resolve(theme)
            } else {
                corpo(feel_of_state(state), bg_token)
            })
        },
        crate::paint::token_to_vello,
    );
    let border = crate::motion::hover_axis(
        soft,
        hover_t,
        Some(ColorToken::Border.resolve(theme)),
        Some(ColorToken::BorderEmph.resolve(theme)),
    )
    .map_or_else(
        || resolve(border_token, theme),
        crate::paint::token_to_vello,
    );
    fill_rounded_rect(scene, box_rect, radius, bg);
    // ⭐ A moldura pela porta do TEMA: no clássico a de sempre; num tema moderno a caixa é plana
    //    (marcada = acento cheio, desmarcada = um degrau abaixo do painel) e só o foco traça.
    let feel = feel_of_state(state);
    crate::paint::stroke_frame(
        scene,
        box_rect,
        radius,
        theme,
        feel,
        if state == CheckboxState::Focused {
            2.0
        } else {
            1.0
        },
        border,
    );

    let glyph_color = if state == CheckboxState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::AccentFg
    };
    let g = resolve(glyph_color, theme);
    match value {
        CheckboxValue::Checked => {
            paint_icon(scene, IconId::Check, box_rect, g, StrokeToken::Default.px())
        }
        // Indeterminate paints a horizontal dash (Minus glyph) — the
        // platform convention for "some children selected". Previous
        // code used `Plus` here, which painted a `+` and read as
        // "add" rather than "mixed".
        CheckboxValue::Indeterminate => paint_icon(scene, IconId::Minus, box_rect, g, 2.0),
        CheckboxValue::Unchecked => {}
    }

    if decorator {
        crate::widget::property_box::paint_decorator(
            scene,
            theme,
            crate::widget::property_box::decorator_rect(rect),
            state == CheckboxState::Disabled,
        );
    }

    MarcaPintada {
        marca: box_rect,
        caixa,
    }
}

#[cfg(test)]
#[path = "mark_tests.rs"]
mod tests;
