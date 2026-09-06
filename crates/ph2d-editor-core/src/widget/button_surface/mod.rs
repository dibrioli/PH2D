//! **A SUPERFÍCIE PLANA de um botão** — a tinta de fundo de quem pinta um botão **sem construir um
//! [`Button`]**.
//!
//! Irmão do [`super::button`] pelo teto de 500 LOC dos primitivos, e o corte é por
//! RESPONSABILIDADE: ali mora *o que um `Button` É* (dados, estados, o pintor dele, o nó de
//! AccessKit); aqui, *que cor tem o fundo de um botão que alguém desenha à mão*.
//!
//! # ⚠️ Por que esta família é uma porta e não um `match`
//!
//! A rota daqui **não passa por widget nenhum** — é uma função livre resolvida dentro de um
//! pintor. Enquanto o mapa duro era público, cinco sítios resolviam-no directamente a partir do
//! estado, e a assinatura **não tem `t`**: as setas de reordenar camada, os botões da máscara, as
//! amostras do *ramp* e os chips do *stroke apply* **SALTAVAM** ao lado de todo o resto do app.
//!
//! ⚠️ **Nenhum gate podia ver isso**: os gates do eixo do hover passam o par à mão a um *widget*.
//! O elo que falhava não estava em nenhuma das duas pontas.
//!
//! ⇒ o mapa duro é **privado**, e foi a privacidade que fez o **compilador** enumerar os sítios.
//! *É a diferença entre uma convenção e uma lei.*

use super::ButtonState;
use ph2d_tokens::{ColorToken, Theme};

/// A superfície de retorno de um botão plano, por estado DISCRETO: repouso `Bg2`,
/// Hovered/Focused `BgElev` (uma elevação subtil), Pressed `AccentSoft` (uma pressão clara) — as
/// mesmas superfícies que o [`Button`] canónico usa.
///
/// ⚠️ **Ela é PRIVADA desde 2026-08-23, e a privacidade É o gate.** Enquanto era pública, cinco
/// sítios de pintura resolviam-na directamente a partir do estado — e a assinatura **não tem `t`
/// nenhum**, então as setas de reordenar camada, os botões da máscara, as amostras do *ramp* e os
/// chips do *stroke apply* **SALTAVAM** ao lado de todo o resto do app, que amacia.
///
/// ⚠️ **Nenhum gate podia ver isso.** Os gates do eixo do hover passam o par à mão a um *widget*, e
/// esta rota não passa por widget nenhum — é uma função livre resolvida dentro do pintor.
///
/// ⇒ ela é agora as **duas pontas** do eixo, e a única porta é a soft
/// [`flat_button_surface_color`]. Fechá-la fez o **compilador** enumerar os sítios, que é a
/// diferença entre uma convenção e uma lei.
#[must_use]
fn flat_button_surface(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Pressed => ColorToken::AccentSoft,
        ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
        _ => ColorToken::Bg2,
    }
}

/// **A superfície de um botão plano, MISTURADA no eixo do hover** — a porta única de *«que cor tem
/// o fundo deste botão AGORA?»* para quem pinta um botão sem construir um [`Button`].
///
/// ⚠️ **Recebe o PAR, e é esse o desenho.** É o mesmo truque do [`Button::visual`]: um par não se
/// pode passar pela metade, então o sítio seguinte **não pode esquecer o `t`** — e a rota certa é
/// tão curta de escrever quanto a errada era.
///
/// ```ignore
/// let bg = flat_button_surface_color(ctx.host.store().button_visual(id), theme);
/// ```
///
/// ⚠️ **A lei do eixo sai da porta que já a tem** ([`crate::motion::hover_axis`]): só
/// `Normal`/`Hovered` são uma *quantidade*; `Pressed`, `Focused` e `Disabled` são estados DUROS —
/// meia-desactivação não significa coisa nenhuma. E o neutro [`crate::motion::SETTLED`] cai no
/// token duro, então um id que o relógio nunca viu pinta **byte a byte** o mundo pré-substrato.
#[must_use]
pub fn flat_button_surface_color(v: (ButtonState, f32), theme: Theme) -> ph2d_vector::Color {
    let (state, t) = v;
    let soft = matches!(state, ButtonState::Normal | ButtonState::Hovered);
    crate::motion::hover_axis(
        soft,
        t,
        Some(flat_button_surface(ButtonState::Normal).resolve(theme)),
        Some(flat_button_surface(ButtonState::Hovered).resolve(theme)),
    )
    .map_or_else(
        || crate::paint::resolve(flat_button_surface(state), theme),
        crate::paint::token_to_vello,
    )
}

/// **O `t` DE UM CHIP DE CHROME**, ou `None` quando este chip não está no eixo.
///
/// ⚠️ **Um chip ACTIVO fica fora do eixo, e não é rigor.** *Activo* não é uma quantidade — é o
/// estado que diz *"esta é a ferramenta na tua mão"* —, e desvanecê-lo entre dois valores faria o
/// rail piscar a resposta a uma pergunta que o artista lê de relance. Mesma lei do `Pressed` no
/// [`super::Button::bg_color`].
///
/// ⚠️ **Ela nasceu PRIVADA dentro do `tool_rail/paint.rs`, e foi essa privacidade que deixou três
/// cópias nascer sem ela** (as duas outras variantes de chip do rail, e o chip da barra de topo):
/// quatro pintores desenham o mesmo quadrado e **um** amaciava. Aqui ela é a porta dos quatro.
#[must_use]
pub fn chip_axis_t(state: ButtonState, active: bool, t: Option<f32>) -> Option<f32> {
    if active || !matches!(state, ButtonState::Normal | ButtonState::Hovered) {
        return None;
    }
    t
}

/// **Como um CHIP DE CHROME se sente** — o estado do botão e o «está em mãos / o modo está
/// ligado» reduzidos ao vocabulário da porta da moldura ([`ph2d_tokens::visuals::Feel`]).
///
/// ⚠️ Vive AQUI, ao lado de [`chip_axis_t`], porque é a mesma porta: o rail e os chips da barra
/// do topo declaram copiar a MESMA matriz, e uma segunda cópia desta redução divergiria no dia
/// em que um estado novo entrasse num só lado (há gate: `the_chip_axis_has_one_door`).
#[must_use]
pub fn chip_feel(state: ButtonState, is_active: bool) -> ph2d_tokens::visuals::Feel {
    use ph2d_tokens::visuals::Feel;
    match state {
        ButtonState::Pressed => Feel::Active,
        _ if is_active => Feel::Active,
        ButtonState::Hovered => Feel::Hovered,
        ButtonState::Focused => Feel::Focused,
        ButtonState::Disabled => Feel::Disabled,
        _ => Feel::Rest,
    }
}

/// **A COR DE UM CHIP DE CHROME no eixo do hover** — a mistura `rest → hot` por `t`, ou o token
/// DURO quando este chip não está no eixo (ou já assentou).
///
/// ⚠️ Mistura-se o TOKEN e converte-se depois porque [`crate::motion::blend_token_color`] é o motor
/// único deste eixo (o mesmo do `Button` e do `IconButton`) — uma segunda aritmética de cor
/// divergiria da dele no dia em que uma das duas ganhasse gama.
#[must_use]
pub fn chip_axis_color(
    t: Option<f32>,
    rest: ColorToken,
    hot: ColorToken,
    hard: ColorToken,
    theme: Theme,
) -> ph2d_vector::Color {
    if let Some(t) = t
        && let Some(c) =
            crate::motion::blend_token_color(Some(rest.resolve(theme)), Some(hot.resolve(theme)), t)
    {
        return crate::paint::token_to_vello(c);
    }
    crate::paint::resolve(hard, theme)
}

#[cfg(test)]
mod flat_surface_tests {
    use super::*;
    use crate::motion::SETTLED;

    /// O tema por omissão — os gates são sobre o EIXO, não sobre a paleta.
    const DARK: Theme = Theme::Forge;

    fn hard(state: ButtonState) -> ph2d_vector::Color {
        crate::paint::resolve(flat_button_surface(state), DARK)
    }

    /// ⭐ **O NEUTRO PINTA O MUNDO PRÉ-SUBSTRATO, BYTE A BYTE.**
    ///
    /// ⚠️ É a metade que faz esta wave não custar nada: um id que o relógio nunca viu devolve
    /// [`SETTLED`], e a porta nova tem de dar **exactamente** o token duro. Sem este gate, migrar
    /// cinco sítios de pintura seria uma mudança de aparência disfarçada de refactor.
    #[test]
    fn the_neutral_paints_the_hard_token_byte_for_byte() {
        for state in [
            ButtonState::Normal,
            ButtonState::Hovered,
            ButtonState::Pressed,
            ButtonState::Focused,
            ButtonState::Disabled,
        ] {
            assert_eq!(
                flat_button_surface_color((state, SETTLED), DARK),
                hard(state),
                "o neutro mudou a cor de {state:?}"
            );
        }
    }

    /// ⭐ **E O EIXO DE FACTO ANDA** — o controlo do gate acima.
    ///
    /// ⚠️ Sem ele, uma implementação que ignorasse o `t` por completo passaria no primeiro: ela
    /// devolveria sempre o token duro, que é o que o primeiro exige. *Um neutro correcto não prova
    /// que existe um eixo.*
    #[test]
    fn the_axis_actually_travels_between_the_two_ends() {
        let rest = hard(ButtonState::Normal);
        let hot = hard(ButtonState::Hovered);
        assert_ne!(rest, hot, "a fixture não separa as pontas");

        // A meio caminho não é nenhuma das duas.
        let mid = flat_button_surface_color((ButtonState::Hovered, 0.5), DARK);
        assert_ne!(mid, rest, "o meio ficou no repouso: o eixo não anda");
        assert_ne!(mid, hot, "o meio saltou para o hover: o eixo não anda");

        // ⚠️ **A SAÍDA é a mesma expressão da entrada**, e é a razão de o eixo ser conduzido pelo
        // ESCALAR: no quadro em que o rato sai, o estado já é `Normal` — se o estado escolhesse,
        // não haveria nada entre onde a cor está e onde ela vai.
        let leaving = flat_button_surface_color((ButtonState::Normal, 0.5), DARK);
        assert_eq!(
            leaving, mid,
            "sair e entrar dão cores diferentes a meio caminho: o eixo está preso ao ESTADO"
        );
    }

    /// ⛔ **OS ESTADOS DUROS IGNORAM O `t`.**
    ///
    /// ⚠️ `Pressed`, `Focused` e `Disabled` não são uma *quantidade* de nada — meia-desactivação
    /// não significa coisa nenhuma. É a lei que o [`crate::motion::hover_axis`] já escreve, e este
    /// gate é o que impede alguém de a "uniformizar" ao migrar o sexto sítio.
    #[test]
    fn the_hard_states_ignore_the_clock() {
        for state in [
            ButtonState::Pressed,
            ButtonState::Focused,
            ButtonState::Disabled,
        ] {
            for t in [0.0, 0.25, 0.5, 1.0] {
                assert_eq!(
                    flat_button_surface_color((state, t), DARK),
                    hard(state),
                    "{state:?} deixou-se conduzir pelo relógio em t={t}"
                );
            }
        }
    }
}

/// Os gates do EIXO de um chip de chrome.
///
/// ⚠️ **Eles vieram do `tool_rail/tests.rs` com a lei** (auditoria de 2026-08-23): enquanto o par
/// era privado do rail, três outros pintores do mesmo quadrado nasceram sem ele. A lei mudou de
/// casa, e os gates dela mudaram junto — deixá-los para trás faria a casa nova nascer sem prova.
#[cfg(test)]
mod chip_axis_tests {
    use super::*;
    use crate::widget::ButtonState;

    /// **Um chip ACTIVO fica fora do eixo, e é decisão — não omissão.**
    ///
    /// *Activo* responde *«esta é a ferramenta na tua mão»*, e uma resposta que desvanece entre dois
    /// valores é uma resposta que se lê mal de relance. Mesma lei do `Pressed` no `Button::bg_color`.
    ///
    /// **Mutação que deve sangrar:** tirar o `is_active ||` da guarda — um chip seleccionado passaria
    /// a piscar de Accent para Text2 e de volta enquanto o rato passa por cima.
    #[test]
    fn only_a_resting_or_hovered_chip_lives_on_the_hover_axis() {
        assert_eq!(
            chip_axis_t(ButtonState::Normal, false, Some(0.4)),
            Some(0.4)
        );
        assert_eq!(
            chip_axis_t(ButtonState::Hovered, false, Some(0.4)),
            Some(0.4)
        );
        // Activo, premido e desactivado NÃO são quantidades: saem do eixo.
        assert_eq!(chip_axis_t(ButtonState::Normal, true, Some(0.4)), None);
        assert_eq!(chip_axis_t(ButtonState::Pressed, false, Some(0.4)), None);
        assert_eq!(chip_axis_t(ButtonState::Disabled, false, Some(0.4)), None);
        // E um pintor SEM relógio não tem eixo nenhum.
        assert_eq!(chip_axis_t(ButtonState::Hovered, false, None), None);
    }

    /// **Meio caminho é uma cor NOVA, não uma das duas pontas.**
    ///
    /// ⚠️ O oráculo não é *«é diferente de uma delas»*: um `blend` partido que devolvesse sempre a
    /// ponta HOT passaria nesse teste. Tem de estar **entre** as duas em cada canal, e diferir das
    /// duas — é isso que distingue misturar de escolher.
    #[test]
    fn half_a_hover_is_a_colour_between_the_two_ends() {
        use crate::paint::resolve;
        let theme = Theme::Forge;
        let rest = resolve(ColorToken::Text2, theme);
        let hot = resolve(ColorToken::Text1, theme);
        let mid = chip_axis_color(
            Some(0.5),
            ColorToken::Text2,
            ColorToken::Text1,
            ColorToken::Text2,
            theme,
        );
        assert_ne!(mid.to_rgba8().to_u8_array(), rest.to_rgba8().to_u8_array());
        assert_ne!(mid.to_rgba8().to_u8_array(), hot.to_rgba8().to_u8_array());
        let (a, b, m) = (
            rest.to_rgba8().to_u8_array(),
            hot.to_rgba8().to_u8_array(),
            mid.to_rgba8().to_u8_array(),
        );
        for i in 0..3 {
            let (lo, hi) = (a[i].min(b[i]), a[i].max(b[i]));
            assert!(
                m[i] >= lo && m[i] <= hi,
                "canal {i}: {} não está entre {lo} e {hi}",
                m[i]
            );
        }
        // E o NEUTRO devolve a cor dura, exactamente — é o que mantém byte-idêntico todo chamador
        // que não passa relógio nenhum.
        let neutral = chip_axis_color(
            None,
            ColorToken::Text2,
            ColorToken::Text1,
            ColorToken::Accent,
            theme,
        );
        assert_eq!(
            neutral.to_rgba8().to_u8_array(),
            resolve(ColorToken::Accent, theme).to_rgba8().to_u8_array()
        );
    }
}

/// ⭐ **A FORMA de um grupo de botões** — a lei do Blender, num irmão pelo tecto de 500 LOC dos
/// primitivos. O corte é por RESPONSABILIDADE: aqui em cima, *que COR tem o fundo de um botão*;
/// ali, *que FORMA ele tem quando tem vizinhos*.
mod group;
pub use group::{
    GroupCell, GroupPos, SEGMENT_HAIRLINE, block_cells, grid_cells, grid_height, segment_rects,
};
