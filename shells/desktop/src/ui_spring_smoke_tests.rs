//! Os gates da cena da MOLA — *a fixture contém o fenômeno, e os vereditos que ela imprime são
//! verdadeiros.*
//!
//! ⚠️ Irmão de [`super`] pelo teto de 600 LOC da shell (HR-18), e **FILHO por `#[path]`**: o
//! `use super::*` continua a alcançar os itens privados da cena, então nada teve de mudar de
//! visibilidade para caber.

use super::*;

/// **As quatro faixas não se tocam** — sem folga vertical a foto lê como colisão.
#[test]
fn the_lanes_are_clear_of_each_other() {
    for w in LANES.windows(2) {
        let gap = (w[0].1 - w[1].1).abs();
        assert!(
            gap > MARK_H,
            "as faixas {} e {} distam {gap} — menos que a altura da marca ({MARK_H})",
            w[0].0,
            w[1].0
        );
    }
}

/// ⭐ **A viagem é longa o suficiente para um overshoot ser VISÍVEL.**
///
/// ⚠️ É a premissa que se apaga em silêncio: alguém encurta a cena para ela caber melhor no
/// ecrã e o pico de 10% do `Back` passa a valer menos que a espessura da marca — a cena
/// continua bonita e deixa de mostrar a única coisa que o Enio tem de julgar.
#[test]
fn the_smallest_overshoot_is_wider_than_the_target_mark() {
    // O menor pico das três curvas é o do `Back Out`: 1,100.
    let travel = X_HOVER - X_REST;
    let overshoot = travel * 0.100;
    assert!(
        overshoot > MARK_W * 4.0,
        "o overshoot do Back mede {overshoot} e a marca {MARK_W}: nao daria para ver"
    );
}

/// **A mola da cena é SUB-amortecida** — o default de produto é o crítico, e o crítico não
/// mostra a metade que esta cena existe para mostrar.
#[test]
fn the_scene_spring_is_underdamped_unlike_the_product_default() {
    // ⚠️ Em bloco `const`: as duas pontas são constantes, então a premissa da cena falha na
    // COMPILAÇÃO em vez de numa corrida — que é o mais cedo que uma fixture pode morder.
    const {
        assert!(
            SPRING_CFG.damping < ph2d_ui_state::DEFAULT_DAMPING,
            "a mola da cena esta' no critico ou acima — ela nao passaria da marca"
        );
    }
    assert_eq!(
        SPRING_CFG.clamped(),
        SPRING_CFG,
        "a mola da cena esta' fora da faixa dos sliders — o artista nao conseguiria reproduzi-la"
    );
}

/// **As quatro faixas mudam de cor visivelmente**, e a viagem é a MESMA nas quatro.
///
/// ⚠️ A segunda metade é o que faz disto um A/B: se uma faixa viajasse mais que outra, a
/// comparação estaria a medir duas coisas ao mesmo tempo.
#[test]
fn every_lane_travels_the_same_distance_and_changes_colour() {
    for i in 0..LANES.len() {
        assert_ne!(
            REST[i], HOVER[i],
            "a faixa {} pinta a mesma cor nos dois papeis",
            LANES[i].0
        );
    }
    const {
        assert!(X_HOVER > X_REST + BOX_W, "o destino sobrepoe a origem");
    }
}

/// **Cada faixa tem um motor DIFERENTE** — quatro faixas com a mesma curva seriam a mesma
/// faixa quatro vezes.
#[test]
fn the_four_lanes_are_four_different_engines() {
    let drives: Vec<_> = (0..LANES.len()).map(drive_of).collect();
    for i in 0..drives.len() {
        for j in (i + 1)..drives.len() {
            assert_ne!(
                drives[i], drives[j],
                "as faixas {} e {} correm o mesmo motor",
                LANES[i].0, LANES[j].0
            );
        }
    }
    assert!(drives[SPRING].is_none(), "a faixa Spring nao e' uma mola");
}

/// **O roteiro cabe no terminal** — a mesma régua dos irmãos.
#[test]
fn the_script_fits_the_terminal() {
    crate::smoke_script::assert_fits("spring", STEPS);
}

/// A mesma viagem que a cena autora, sem janela: `0` → [`TRAVEL`].
fn lane_states() -> Vec<ph2d_ui_state::UiState> {
    use ph2d_ui_state::{ObjectPose, UiState};
    [(StateRole::Default, 0.0), (StateRole::Hover, TRAVEL)]
        .into_iter()
        .map(|(role, x)| {
            let mut s = UiState::new(role);
            s.objects = vec![ObjectPose {
                translation: [x, 0.0],
                ..ObjectPose::new(1)
            }];
            s
        })
        .collect()
}

/// Quão longe a faixa `lane` de facto vai — o mesmo laço do [`peak_x`], sem a cena.
fn peak(lane: usize) -> f64 {
    let mut m = Machine::new(lane_states()).expect("dois estados nunca sao vazios");
    match drive_of(lane) {
        None => m.go_to_role_spring(StateRole::Hover, SPRING_CFG),
        Some(e) => m.go_to_role(StateRole::Hover, CURVE_SECONDS, e),
    }
    let mut hi = f64::NEG_INFINITY;
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
        hi = hi.max(m.pose()[0].translation[0]);
        if !m.is_animating() {
            break;
        }
    }
    hi
}

/// ⭐⭐ **Os três vereditos que o `announce` imprime, medidos aqui.**
///
/// ⚠️ **Sem este gate a cena só falha na mão do Enio.** O `announce` afirma três coisas — a
/// mola passa, o Back passa, o controle NÃO passa — e as três dependem do clamp por canal
/// continuar por canal. Um `t.clamp(0,1)` reposto no `Transition::at` deixa a cena a imprimir
/// `!! PARE` num sábado, com a suíte inteira verde.
///
/// ⚠️ E a terceira é a metade que responde à ordem *"não prejudique nada do sistema de
/// easing"*: uma curva contida em `[0, 1]` tem de chegar ao alvo e ficar lá, **exactamente**.
#[test]
fn the_three_verdicts_the_scene_prints_are_true() {
    let spring = peak(SPRING);
    let back = peak(BACK);
    let control = peak(LANES.len() - 1);
    assert!(
        spring > TRAVEL * 1.02,
        "a MOLA picou em {spring} contra um alvo de {TRAVEL}: ela nao passa da marca, e a \
         cena vai imprimir PARE"
    );
    assert!(
        back > TRAVEL * 1.02,
        "o BACK picou em {back} contra {TRAVEL}: a mudanca desta wave nao esta' la'"
    );
    assert!(
        (control - TRAVEL).abs() < 1.0e-6,
        "o CONTROLE picou em {control} contra {TRAVEL}: uma curva contida em [0,1] tem de \
         ficar byte-identica ao que ja' shipava"
    );
}

/// ⭐ **E a REVERSÃO carrega o momento — no caminho do PRODUTO, não no da mola nua.**
///
/// ⚠️ O gate irmão em `ph2d-ui-state` prova que `SpringState::resuming(v)` arranca mais
/// depressa que `at_rest()`. Isso é sobre o INTEGRADOR. Este prova a mesma coisa sobre a
/// `Machine` — que é quem projeta a velocidade no eixo do caminho novo —, e é o único sítio
/// onde um erro de SINAL nessa projeção apareceria.
///
/// O oráculo é a comparação com a faixa CURVE sob o mesmo gesto: ela é o `Cubic In-Out` que a
/// medição nomeia a **0,00×**, e é literalmente a razão de a mola existir.
#[test]
fn reverting_mid_flight_carries_momentum_and_the_curve_does_not() {
    // Anda até meio caminho, inverte, e mede quanto a forma AINDA avança na direção antiga.
    let after_reversal = |lane: usize| {
        let mut m = Machine::new(lane_states()).expect("dois estados nunca sao vazios");
        match drive_of(lane) {
            None => m.go_to_role_spring(StateRole::Hover, SPRING_CFG),
            Some(e) => m.go_to_role(StateRole::Hover, CURVE_SECONDS, e),
        }
        while m.pose()[0].translation[0] < TRAVEL * 0.5 && m.is_animating() {
            m.advance(1.0 / 60.0);
        }
        let at_reversal = m.pose()[0].translation[0];
        match drive_of(lane) {
            None => m.go_to_role_spring(StateRole::Default, SPRING_CFG),
            Some(e) => m.go_to_role(StateRole::Default, CURVE_SECONDS, e),
        }
        m.advance(1.0 / 60.0);
        m.pose()[0].translation[0] - at_reversal
    };
    let spring = after_reversal(SPRING);
    let curve = after_reversal(CURVE);
    assert!(
        spring > 0.0,
        "a MOLA revertida recuou logo no primeiro quadro ({spring}): ela nao carregou o \
         momento, e a wave nao entrega nada que uma curva ja' nao entregasse"
    );
    assert!(
        curve <= 0.0,
        "a CURVA revertida avancou ({curve}) — ela e' o controle, e um controle que se \
         comporta como o experimento nao separa nada"
    );
}
