//! **Arch-gate da FORMA do gesto de região** (plano 25 §9, o LAÇO).
//!
//! O motor está gateado na `ph2d-vec-edit` (`lasso_tests.rs`) e a lei de captura em
//! `src/vec_marquee_tests.rs`. O que só um gate de FONTE alcança é a costura da shell: os dois
//! braços de press vivem dentro do `input_dispatch`, que exige janela, e nenhum teste de unidade
//! aperta um botão do rato ali.
//!
//! **Três maneiras de partir a wave deixando a suíte inteira verde:**
//!
//! 1. **só um dos braços pergunta a forma** — o outro abre um retângulo sempre, e o laço deixa de
//!    existir exatamente no gesto ADITIVO (o braço do Shift), que é onde o artista mais o quer:
//!    laçar um punhado de nós *somando* aos que já escolheu;
//! 2. **o release não ramifica** — o caminho é gravado, desenhado na tela, e a soltura selecciona
//!    pelo retângulo entre os dois cantos. O artista vê o laço e recebe a caixa;
//! 3. **o paint não ramifica** — o motor selecciona pelo polígono e a tela desenha um retângulo,
//!    então o artista julga a região por um desenho que não é o que decide.
//!
//! ⚠️ As asserções afirmam uma RELAÇÃO ou um CONTEÚDO dentro de uma janela sintática, nunca uma
//! distância em bytes — esta linha já teve dois arch-gates apodrecerem por medirem bytes.

const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
const RENDER: &str = include_str!("../src/render_loop/mod.rs");

/// A posição da 1ª ocorrência de `needle` em `src`, ou pânico com a razão — o **controle
/// positivo**: um dono que se mudou vira falha alta, e não uma varredura vazia que passa.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu — se foi renomeado, atualize este gate (e confira que o laco ainda \
             chega ao artista: `PH2D_BUILD_SMOKE=66`)"
        )
    })
}

/// **Os DOIS braços de press abrem o gesto pela MESMA porta.**
///
/// O canvas tem dois `Down` primários: o com Shift (que também trata o toggle de nó e de objeto) e
/// o sem. Os dois podem terminar a abrir a região no vazio, e uma cópia da decisão num deles é
/// como o laço nasce inalcançável numa metade do produto.
#[test]
fn both_press_arms_ask_the_one_door_for_the_shape() {
    let opens: Vec<usize> = DISPATCH
        .match_indices("crate::vec_marquee::VecMarquee::open(")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        opens.len(),
        2,
        "esperava DOIS sitios a abrir o gesto (o braco com Shift e o sem) e achei {} — se um \
         braco novo nasceu, ele tambem tem de perguntar a porta",
        opens.len()
    );
    for i in opens {
        // A janela é a chamada: o argumento da forma tem de ser a porta, não um literal.
        let window = &DISPATCH[i..(i + 200).min(DISPATCH.len())];
        assert!(
            window.contains("self.marquee_shape_for_press()"),
            "um dos bracos de press abre o gesto sem perguntar a forma — ele vai abrir sempre a \
             mesma, e o Ctrl (ou o chip) fica morto nesse gesto. Janela:\n{window}"
        );
    }
}

/// **A porta única lê o chip PEGAJOSO e o Ctrl** — as duas entradas, compostas uma vez.
#[test]
fn the_one_door_composes_the_sticky_chip_with_the_modifier() {
    const SRC: &str = include_str!("../src/vec_marquee.rs");
    let f = at(SRC, "fn marquee_shape_for_press");
    let body = &SRC[f..];
    let end = at(body, "\n    }\n");
    let body = &body[..end];
    assert!(
        body.contains("self.vec_draw_config.marquee"),
        "a porta nao le' o chip pegajoso do painel — o par Box|Lasso vira um controle morto"
    );
    assert!(
        body.contains("control_key()"),
        "a porta nao le' o Ctrl — a saida de fluxo desaparece e o laco passa a exigir ida-e-volta \
         ao painel por cada selecao"
    );
    assert!(
        body.contains("MarqueeShape::for_gesture"),
        "a composicao foi feita a mao aqui em vez de pela porta pura (que e' onde ela tem gate)"
    );
}

/// **O release ramifica na forma que o press congelou** — e as duas rotas existem.
#[test]
fn the_release_routes_the_frozen_shape_to_its_own_selection() {
    let take = at(DISPATCH, "if let Some(m) = self.vec_marquee.take()");
    let window = &DISPATCH[take..(take + 2600).min(DISPATCH.len())];
    assert!(
        window.contains("m.shape"),
        "o release nao olha a forma congelada — ele decide por outra coisa qualquer"
    );
    assert!(
        window.contains("box_select_with(") && window.contains("lasso_select_with("),
        "o release nao tem as DUAS rotas: o artista desenha um laco e recebe um retangulo"
    );
    assert!(
        window.contains("m.closed_path()"),
        "o release nao promove a amostra final — o laco fecha no ultimo ponto que o piso aceitou, \
         e nao onde a mao soltou"
    );
}

/// **O desenho ramifica na MESMA forma** — o que se vê é o que decide.
#[test]
fn the_paint_draws_the_shape_the_gesture_froze() {
    let at_paint = at(RENDER, "if let Some(m) = self.vec_marquee.as_ref()");
    let window = &RENDER[at_paint..(at_paint + 1200).min(RENDER.len())];
    assert!(
        window.contains("m.shape"),
        "o desenho nao olha a forma congelada"
    );
    assert!(
        window.contains("draw_marquee(") && window.contains("draw_lasso("),
        "o desenho nao tem as duas rotas — o motor selecciona por um poligono e a tela mostra um \
         retangulo, entao o artista julga a regiao por um desenho que nao a descreve"
    );
}
