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
