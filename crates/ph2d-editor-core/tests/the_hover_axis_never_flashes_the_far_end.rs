//! **A cor de um botão viaja de um extremo ao outro — ela nunca SALTA para o destino e volta.**
//!
//! Report do smoke (Enio, 2026-08-15): *«Botões do audio mixer piscam estranho ao retirar o
//! mouse»*. Medido antes de uma linha, no par `Danger → DangerSoft` (o Mute, o maior contraste do
//! app): assente no hover `rgb(56,30,28)` → **UM quadro em `rgb(236,91,87)`** → `rgb(90,42,39)` → o
//! desvanecer normal. **180 níveis de 255 de ida e volta num quadro.**
//!
//! ⚠️ **O defeito era do SUBSTRATO, não do painel que o reportou.** O tique publicava o valor
//! PRÉ-voo, então no quadro da saída o par `(estado, t)` dizia *frio* e *no extremo quente* ao
//! mesmo tempo; o `hover_axis` lê `t >= SETTLED` como *«este id não tem relógio»* e devolve o
//! controlo ao estado DISCRETO, que já esfriou. Os **nove** consumidores do eixo têm a mesma forma
//! — o mixer só tornou o salto visível porque os outros pares de tokens são vizinhos.
//!
//! ⚠️ **Por isso o oráculo aqui é o `widget::Button`, e não o pintor do mixer:** ele é a lei que o
//! app inteiro herda. Um gate escrito contra o painel teria pinado o sintoma no sítio onde ele foi
//! visto, e deixado os outros oito a piscar.
//!
//! ⚠️ **E o oráculo é a MONOTONICIDADE, não um número.** Uma barra («o salto é menor que N») teria
//! de escolher o N, e o que o artista vê não é a magnitude de um quadro — é a cor **voltar atrás**.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState};
use ph2d_tokens::{ColorToken, Theme};

const DT: f64 = 1.0 / 60.0;
const ID: NodeId = NodeId(4242);

/// O canal vermelho do fundo que o `Button` canónico pinta AGORA para o [`ID`].
///
/// ⚠️ **Vermelho porque a família é `Danger`:** ali os dois extremos distam 180 níveis, então uma
/// reversão de um quadro é inequívoca. Numa família neutra (`Bg3 → BgElev`) o mesmo defeito move
/// oito níveis e afogar-se-ia em arredondamento.
fn red(hero: &HeroScreen, theme: Theme) -> u8 {
    Button::new(ID, "x")
        .kind(ButtonKind::Danger)
        .visual(hero.store.button_visual(ID))
        .bg_color(theme)
        .expect("um kind cheio tem sempre fundo")
        .r
}

fn hero_with_button() -> HeroScreen {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.register(
        ID,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    hero
}

fn set_state(hero: &mut HeroScreen, state: ButtonState) {
    if let Some(InteractiveState::Button { state: slot }) = hero.store.get_mut(ID) {
        *slot = state;
    } else {
        panic!("o botao da fixture desapareceu do store");
    }
    hero.store
        .set_hot((state == ButtonState::Hovered).then_some(ID));
}

fn tick_and_sample(hero: &mut HeroScreen, theme: Theme, frames: usize) -> Vec<u8> {
    (0..frames)
        .map(|_| {
            hero.tick_motion(DT);
            red(hero, theme)
        })
        .collect()
}

/// **Cada amostra anda para o destino, ou fica onde está — nunca para trás.**
///
/// O `from` entra na sequência como PRIMEIRO elemento pelo chamador: é a cor que o artista estava
/// a ver quando o gesto mudou, e é contra ela que o primeiro quadro novo é julgado. ⚠️ Sem essa
/// âncora um salto que acontecesse **no quadro zero** ficaria fora do que o gate observa.
fn assert_travels_without_turning_back(seq: &[u8], to: u8, label: &str) {
    let from = *seq.first().expect("a sequencia tem a ancora");
    assert!(
        (i32::from(*seq.last().expect("a sequencia tem cauda")) - i32::from(from)).abs() > 100,
        "{label}: a fixture nao contem o fenomeno — a cor mal se moveu ({from} -> {:?})",
        seq.last()
    );
    let forward = to > from;
    for pair in seq.windows(2) {
        let (a, b) = (i32::from(pair[0]), i32::from(pair[1]));
        let step = b - a;
        assert!(
            if forward { step >= 0 } else { step <= 0 },
            "{label}: a cor voltou atras ({a} -> {b}) a caminho de {to} — o PISCA.\n\
             sequencia: {seq:?}"
        );
    }
}

/// ⭐ **Tirar o rato NÃO acende o botão.**
///
/// *Mutação que deve sangrar:* pôr o `motion.advance` de volta ANTES do `animate` no `tick_hover`
/// ⇒ o primeiro quadro da saída publica `t = 1.0` com o estado já `Normal`, o `hover_axis` devolve
/// `None`, o `Button` cai no token duro de `Normal` e a sequência sai `56, 236, 90, …` — medido.
#[test]
fn the_colour_does_not_jump_to_the_far_end_when_the_pointer_leaves() {
    let theme = Theme::default();
    let mut hero = hero_with_button();
    hero.tick_motion(DT);

    let rest = red(&hero, theme);
    assert_eq!(
        rest,
        ColorToken::Danger.resolve(theme).r,
        "em repouso o botao tem de pintar o token de repouso, ao bit"
    );

    set_state(&mut hero, ButtonState::Hovered);
    let entering = tick_and_sample(&mut hero, theme, 30);
    let hot = *entering.last().expect("a entrada tem cauda");
    assert_eq!(
        hot,
        ColorToken::DangerSoft.resolve(theme).r,
        "assente no hover o botao tem de pintar o token quente, ao bit"
    );

    set_state(&mut hero, ButtonState::Normal);
    let mut leaving = vec![hot];
    leaving.extend(tick_and_sample(&mut hero, theme, 30));

    assert_travels_without_turning_back(&leaving, rest, "ao SAIR");
}

/// **E a entrada continua a ser a mesma viagem** — o controlo da wave.
///
/// A cura reordena o tique para TODA a família, então a metade que já funcionava tem de ser
/// afirmada ao lado da que não funcionava: *uma correcção que só é medida do lado do defeito não
/// diz nada sobre o lado que ela também tocou.*
#[test]
fn the_colour_does_not_jump_to_the_far_end_when_the_pointer_arrives() {
    let theme = Theme::default();
    let mut hero = hero_with_button();
    hero.tick_motion(DT);

    let rest = red(&hero, theme);
    set_state(&mut hero, ButtonState::Hovered);
    let mut entering = vec![rest];
    entering.extend(tick_and_sample(&mut hero, theme, 30));

    assert_travels_without_turning_back(
        &entering,
        ColorToken::DangerSoft.resolve(theme).r,
        "ao ENTRAR",
    );
}
