//! [`NumericInputWithUnit`] — a [`NumberInput`](super::number_input)
//! with a trailing unit chip (`px` / `m` / `deg` / `rad` / `%`).
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.2). Several Inspector fields
//! carry a physical unit (Transform position in `px`/`m`, rotation in
//! `deg`/`rad`, opacity in `%`). This widget renders the unit suffix as
//! a non-editable chip on the right edge of the field and parses a
//! typed suffix back into a value + unit so users can type `90deg`.

use super::number_input::{NumberInput, paint_number_input_with_buffer};
use super::text_input::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::Node;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Physical unit displayed (and parsed) by [`NumericInputWithUnit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Pixels.
    Px,
    /// Meters.
    Meters,
    /// Degrees.
    Degrees,
    /// Radians.
    Radians,
    /// Percent.
    Percent,
    /// ⭐ **Segundos** — a unidade de oito rótulos da §14 Platform Player (*Coyote Time*, *Jump
    /// Buffer*, *Dash Cooldown*, …), medidos em 2026-09-14.
    Seconds,
    /// ⭐ **Metros por segundo** — a unidade de onze rótulos da mesma secção (*Speed*, *Max Fall*,
    /// *Dash Speed*, …).
    ///
    /// ⚠️⚠️ **O sufixo dela CONTÉM o de [`Unit::Seconds`]**, e é por isso que a ordem do
    /// [`Unit::parse_suffix`] é load-bearing: `"5m/s"` termina em `"s"`, logo um `Seconds` testado
    /// primeiro leria a velocidade como um tempo — **sem erro nenhum**, com o número certo e a
    /// unidade errada.
    MetersPerSecond,
    /// ⭐ **Metros por segundo ao quadrado** — a aceleração da braçada (*Swim Accel*), a única
    /// desta unidade no censo de 2026-09-14.
    ///
    /// ⚠️ **O sufixo é ASCII (`m/s2`) e não `m/s²`**, porque o widget usa a MESMA string para
    /// mostrar e para ler: um expoente que o artista não tem no teclado seria um campo que ele não
    /// consegue escrever. ⏳ Separar as duas strings é a dívida que o `DisplayAngle::suffix` já
    /// nomeia (*«o que se escreve e o que se lê não têm de ser a mesma string»*).
    MetersPerSecondSquared,
    /// ⭐ **Newtons** — a unidade de **cinco** rótulos medidos em 2026-09-15 (*Break Force* da junta
    /// e da roda, *Force X*, *Force Y*).
    ///
    /// ⚠️⚠️ **É a primeira unidade MAIÚSCULA do vocabulário, e foi ela que cobrou a dívida escrita
    /// ao lado da [`Unit::MetersPerSecondSquared`].** No SI o newton é `N`; um `n` minúsculo é outra
    /// coisa (nano). Até aqui o [`Unit::parse_suffix`] comparava o texto **já em minúsculas** contra
    /// o sufixo, logo um sufixo com maiúscula nunca voltaria a ser lido — o campo mostraria `5 N` e
    /// recusaria `5 N` de volta. ⇒ a comparação passou a ser **insensível à caixa**, que paga a
    /// dívida sem criar uma segunda string para divergir da primeira.
    Newtons,
    /// ⭐ **Newton-metro** — o binário de ruptura de uma junta (*Break Torque*).
    ///
    /// ⚠️⚠️ **O sufixo dele acaba em `m`**, logo tem de ser testado **antes** de [`Unit::Meters`]:
    /// `"12 N.m"` termina em `"m"`, e um `Meters` testado primeiro leria um binário como um
    /// comprimento — o número certo e a unidade errada, sem erro nenhum.
    NewtonMetres,
    /// ⭐ **Graus por segundo** — a velocidade do motor de uma roda (*Motor*).
    ///
    /// ⚠️ **O sufixo acaba em `s`**, logo vem antes de [`Unit::Seconds`], pela mesma lei que separa
    /// `m/s` de `s`. ⚠️ E é `deg/s` e não `°/s` **pela razão de sempre**: o que se mostra é o que se
    /// tem de conseguir escrever.
    DegreesPerSecond,
}

impl Unit {
    /// Canonical suffix string, also what the parser matches on.
    pub const fn suffix(self) -> &'static str {
        match self {
            Unit::Px => "px",
            Unit::Meters => "m",
            Unit::Degrees => "deg",
            Unit::Radians => "rad",
            Unit::Percent => "%",
            Unit::Seconds => "s",
            Unit::MetersPerSecond => "m/s",
            Unit::MetersPerSecondSquared => "m/s2",
            Unit::Newtons => "N",
            Unit::NewtonMetres => "N.m",
            Unit::DegreesPerSecond => "deg/s",
        }
    }

    /// **Todas as unidades, na ordem em que o [`Self::parse_suffix`] as testa.**
    ///
    /// ⚠️ Pública para que o gate possa medir a ORDEM em vez de a repetir — uma cópia da lista no
    /// teste provaria que a cópia está ordenada, não que o parser está.
    pub const ALL: [Unit; 11] = [
        // ⚠️ `deg/s` acaba em `s` ⇒ antes de `Seconds`. (E `deg` **não** é o fim de `deg/s`, logo
        //    `Degrees` pode ficar onde está.)
        Unit::DegreesPerSecond,
        Unit::Degrees,
        Unit::Radians,
        Unit::MetersPerSecondSquared,
        Unit::MetersPerSecond,
        Unit::Px,
        // ⚠️ `N.m` acaba em `m` ⇒ antes de `Meters`.
        Unit::NewtonMetres,
        Unit::Meters,
        Unit::Seconds,
        Unit::Newtons,
        Unit::Percent,
    ];

    /// Longest-match parse of a trailing unit suffix. Returns the unit
    /// whose suffix the (lowercased, trimmed) string ends with, longest
    /// first so `rad` wins over a hypothetical shorter match.
    pub fn parse_suffix(s: &str) -> Option<Unit> {
        let s = s.trim().to_ascii_lowercase();
        // Order longest-first to avoid a short suffix shadowing a longer
        // one that shares its tail.
        // ⚠️⚠️ **A ordem é a LEI, não arrumação:** `"m/s"` termina em `"s"` e `"s"` não termina em
        // `"m/s"`, logo o mais LONGO tem de ser testado primeiro. Com a ordem trocada, `"5m/s"`
        // devolve `(5.0, Seconds)` — o número certo e a unidade errada, sem erro nenhum. O gate
        // `a_longer_suffix_is_never_shadowed_by_a_shorter_one` deriva a ordem de `ALL` e prova-a.
        // ⚠️⚠️ **Insensível à CAIXA, e isso é a dívida do `MetersPerSecondSquared` paga.** O sufixo
        // que se MOSTRA pode ter maiúscula (`N`, `N.m` — o newton do SI), e o que o artista escreve
        // não tem de a ter. ⛔ A alternativa era uma SEGUNDA string por unidade (uma para mostrar,
        // outra para ler), e duas strings para a mesma coisa divergem no dia em que alguém edita uma
        // — o defeito que este ficheiro já nomeia três vezes.
        Self::ALL.into_iter().find(|u| termina_em(&s, u.suffix()))
    }
}

/// `str::ends_with` insensível à caixa, **sem alocar**.
///
/// ⚠️ Os sufixos são todos ASCII de propósito (ver [`Unit::MetersPerSecondSquared`]), logo dobrar a
/// caixa **preserva o comprimento** — é isso que deixa o [`parse`] cortar o número por
/// `suffix().len()` sem reindexar.
fn termina_em(texto: &str, sufixo: &str) -> bool {
    texto.len() >= sufixo.len()
        && texto
            .as_bytes()
            .get(texto.len() - sufixo.len()..)
            .is_some_and(|fim| fim.eq_ignore_ascii_case(sufixo.as_bytes()))
}

/// Parse a typed field like `"90deg"` / `"12.5 px"` / `"50%"` into its
/// numeric value and (optional) unit. Returns `None` only when the
/// numeric portion fails to parse. A bare number (`"42"`) yields
/// `(42.0, None)`.
pub fn parse(input: &str) -> Option<(f64, Option<Unit>)> {
    let trimmed = input.trim();
    let unit = Unit::parse_suffix(trimmed);
    let num_part = match unit {
        Some(u) => &trimmed[..trimmed.len() - u.suffix().len()],
        None => trimmed,
    };
    num_part.trim().parse::<f64>().ok().map(|v| (v, unit))
}

#[derive(Clone, Debug)]
pub struct NumericInputWithUnit {
    pub input: NumberInput,
    pub unit: Unit,
}

impl NumericInputWithUnit {
    pub fn new(input: NumberInput, unit: Unit) -> Self {
        Self { input, unit }
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.input.state = state;
        self
    }

    /// **O campo editável — o host INTEIRO.**
    ///
    /// ⛔⛔ **Era o host menos uma coluna de `Spacing::Xl3` para o chip da unidade, e o chip SAIU**
    /// (ordem do dono, 2026-09-14: *«não ficou legal. Melhor junto ao número dentro da caixa»*).
    /// A unidade é hoje um sufixo pintado **colado ao número**, dentro do mesmo recorte
    /// ([`NumberInput::suffix`]).
    ///
    /// ⭐ **E a geometria melhorou com a aparência:** o rect que os chamadores registam no
    /// `HitIndex` passa a ser a linha toda, logo *um clique onde a unidade está põe o cursor no
    /// número* — antes aqueles 36 px não eram de ninguém.
    ///
    /// ⚠️ A antiga `unit_rect` **morreu com o chip**: não há segundo rectângulo para devolver. Ela
    /// carregava uma aparadura contra derrame num host estreito — *um cuidado que deixou de ter
    /// sujeito, e não uma protecção que se perdeu*.
    pub fn input_rect(&self, host: Rect) -> Rect {
        host
    }

    /// Build the a11y node. Signature mirrors the widget template
    /// (`x, y, w, h`); the unit is folded into the inner field's label
    /// so a screen reader announces "<label> (deg)".
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut labeled = self.input.clone();
        labeled.label = if self.input.label.is_empty() {
            self.unit.suffix().to_string()
        } else {
            format!("{} ({})", self.input.label, self.unit.suffix())
        };
        let host = Rect::new(x as f32, y as f32, w as f32, h as f32);
        let r = self.input_rect(host);
        labeled.build_a11y(r.x as f64, r.y as f64, r.w as f64, r.h as f64)
    }
}

/// ⭐⭐ **O campo com a unidade colada ao número** — ver [`NumberInput::suffix`].
///
/// ⛔ Ele deixou de pintar um chip próprio em 2026-09-14: o corpo inteiro é o campo, e a unidade é
/// um sufixo dentro dele. *Com o campo afundado, um segundo rectângulo com fundo próprio lê-se
/// como duas caixas.*
#[allow(clippy::too_many_arguments)]
pub fn paint_numeric_input_with_unit(
    widget: &NumericInputWithUnit,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    host: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_number_input_with_buffer(
        &widget.input.clone().suffix(Some(widget.unit.suffix())),
        buffer,
        caret,
        selection_anchor,
        widget.input_rect(host),
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    #[test]
    fn suffix_round_trips() {
        for u in [
            Unit::Px,
            Unit::Meters,
            Unit::Degrees,
            Unit::Radians,
            Unit::Percent,
        ] {
            assert_eq!(Unit::parse_suffix(u.suffix()), Some(u));
        }
    }

    #[test]
    fn parse_value_and_unit() {
        assert_eq!(parse("90deg"), Some((90.0, Some(Unit::Degrees))));
        assert_eq!(parse("12.5 px"), Some((12.5, Some(Unit::Px))));
        assert_eq!(parse("50%"), Some((50.0, Some(Unit::Percent))));
        assert_eq!(parse("2.25rad"), Some((2.25, Some(Unit::Radians))));
        assert_eq!(parse("7m"), Some((7.0, Some(Unit::Meters))));
    }

    #[test]
    fn parse_bare_number_has_no_unit() {
        assert_eq!(parse("42"), Some((42.0, None)));
    }

    #[test]
    fn parse_rejects_non_numeric() {
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("deg"), None);
    }

    #[test]
    fn state_propagates_to_input() {
        let w =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Rotation", 90.0), Unit::Degrees)
                .state(TextInputState::Disabled);
        assert_eq!(w.input.state, TextInputState::Disabled);
    }

    /// ⛔⛔ **Este gate media a PARTIÇÃO entre o campo e o chip, e o chip SAIU** (ordem do dono,
    /// 2026-09-14). A propriedade que ele defendia — *«os dois rectângulos não se sobrepõem»* —
    /// deixou de ter sujeito: há um rectângulo só.
    ///
    /// ⚠️ **O que fica no lugar é a propriedade NOVA, e ela é mais forte:** o campo é o host
    /// inteiro, logo *não há um pedaço da linha que não seja clicável*. Era isso que os 36 px do
    /// chip eram.
    #[test]
    fn the_field_is_the_whole_host_so_no_strip_of_the_row_is_dead() {
        let widget =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Rotation", 90.0), Unit::Degrees);
        for host in [
            Rect::new(0.0, 0.0, 200.0, 28.0),
            Rect::new(17.0, 5.0, 40.0, 22.0),
            // ⚠️ O host degenerado que a aparadura do chip antigo existia para sobreviver.
            Rect::new(0.0, 0.0, 0.0, 0.0),
        ] {
            let field = widget.input_rect(host);
            assert_eq!(field, host, "o campo deixou de ser o host inteiro");
        }
    }

    fn smoke(unit: Unit, state: TextInputState, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let widget =
            NumericInputWithUnit::new(NumberInput::new(NodeId(1), "Field", 1.0), unit).state(state);
        paint_numeric_input_with_unit(
            &widget,
            None,
            0,
            None,
            Rect::new(0.0, 0.0, 200.0, 28.0),
            &mut scene,
            &mut text,
            theme,
        );
        let _ = widget.build_a11y(0.0, 0.0, 200.0, 28.0);
    }

    #[test]
    fn paint_smoke_all_units() {
        for u in [
            Unit::Px,
            Unit::Meters,
            Unit::Degrees,
            Unit::Radians,
            Unit::Percent,
        ] {
            smoke(u, TextInputState::Normal, Theme::Forge);
        }
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(Unit::Degrees, TextInputState::Disabled, Theme::Sunstone);
    }

    /// ⛔⛔ **Este gate media a CONTENÇÃO do chip, e o chip SAIU** (2026-09-14). Ele defendia um
    /// slot de largura FIXA dentro de um host VARIÁVEL — medido na altura, um host de `0 px` punha
    /// a borda esquerda do chip **32 px fora**, sobre o campo numérico.
    ///
    /// ⭐ **A cura de aparência dissolveu a classe inteira do defeito**, e é por isso que o
    /// substituto não é uma contenção mais apertada: *não há um segundo rectângulo para conter*. A
    /// unidade é texto dentro do recorte do valor, e quem a apara é o mesmo `push_clip` que já
    /// aparava o número.
    ///
    /// ⚠️ O que fica medido é a propriedade que SOBROU: o sufixo nunca é pintado à esquerda do
    /// número, em host nenhum.
    #[test]
    fn the_unit_never_lands_left_of_the_number() {
        for hw in [200.0_f32, 40.0, 32.0, 24.0, 10.0, 0.0] {
            let host = Rect::new(100.0, 50.0, hw, 24.0);
            let w = NumericInputWithUnit::new(
                NumberInput::new(ph2d_a11y::NodeId(1), "X", 0.0),
                Unit::Degrees,
            );
            let field = w.input_rect(host);
            assert!(
                field.x >= host.x - 1e-3 && field.x + field.w <= host.x + host.w + 1e-3,
                "host.w={hw}: o campo saiu do host"
            );
        }
    }
}
