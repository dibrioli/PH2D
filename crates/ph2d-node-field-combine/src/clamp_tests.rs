//! Os gates do [`super::CLAMP`] e do [`super::MODE_AVERAGE`] — doc 89, folha 10.
//!
//! ⚠️ **O gate mais informativo aqui é um CENSO, não um caso.** A afirmação do toggle não é
//! *"o Add deixa de saturar"* — é *"ele governa exactamente os dois modos que truncavam, e
//! nenhum outro"*. Um gate por caso provaria o primeiro e deixaria a segunda metade por
//! escrever, que é onde um clamp esquecido num modo novo se esconderia.

use super::*;

/// Todos os modos do enum, na ordem do painel.
const MODES: [i32; 9] = [0, 1, 2, 3, 4, 5, 6, 7, MODE_AVERAGE];

/// **O CLAMP É O QUE FAZ O `Add` SATURAR** — e o `Subtract` cortar em zero.
#[test]
fn the_clamp_is_what_makes_add_saturate_and_subtract_floor() {
    assert_eq!(blend(0.8, 0.9, 1, true), 1.0, "ligado, o Add satura");
    let loose = blend(0.8, 0.9, 1, false);
    assert!(
        (loose - 1.7).abs() < 1e-6,
        "desligado, o Add passa de 1: {loose}"
    );
    assert_eq!(blend(0.2, 0.9, 2, true), 0.0, "ligado, o Subtract corta");
    let below = blend(0.2, 0.9, 2, false);
    assert!(
        (below + 0.7).abs() < 1e-6,
        "desligado, o Subtract passa de zero: {below}"
    );
}

/// **ELE GOVERNA EXACTAMENTE DOIS MODOS** — o censo.
///
/// ⚠️ A metade que importa é a de baixo: se um modo novo nascer com um clamp inline, este
/// gate acusa-o, porque a contagem de modos SENSÍVEIS ao toggle passa de dois.
#[test]
fn the_toggle_governs_exactly_the_two_modes_that_truncated() {
    // Um par que sai da faixa nos dois sentidos.
    let cases = [(0.8f32, 0.9f32), (0.2, 0.9)];
    let mut sensitive: Vec<i32> = Vec::new();
    for m in &MODES {
        if cases
            .iter()
            .any(|(a, b)| blend(*a, *b, *m, true) != blend(*a, *b, *m, false))
        {
            sensitive.push(*m);
        }
    }
    assert_eq!(
        sensitive,
        vec![1, 2],
        "só o Add e o Subtract truncavam; qualquer outro sensível é um clamp novo escondido"
    );
}

/// **A MÉDIA É O PONTO MÉDIO, E NÃO TRUNCA.**
#[test]
fn average_is_the_midpoint_and_never_truncates() {
    let mid = blend(0.8, 0.9, MODE_AVERAGE, true);
    assert!((mid - 0.85).abs() < 1e-6, "a média de 0,8 e 0,9: {mid}");
    // ⚠️ Com uma entrada JÁ fora da faixa (que é o que o clamp desligado deixa chegar), a
    // média está fora — e truncar aqui seria reintroduzir num modo novo o que o toggle curou.
    let high = blend(1.7, 0.1, MODE_AVERAGE, true);
    assert!((high - 0.9).abs() < 1e-6, "a média de 1,7 e 0,1: {high}");
}

/// **A CADEIA QUE O CLAMP DESTRANCOU DÁ O MESMO NÚMERO QUE O MODO** — a medição que a nota do
/// [`MODE_AVERAGE`] afirma, executável.
///
/// ⚠️ É esta igualdade que torna honesta a frase *"a média ficou exprimível a dois nós"*. Sem
/// ela, aquilo seria uma alegação sobre uma composição que ninguém correu.
#[test]
fn the_chain_the_clamp_unlocked_equals_the_average_mode() {
    for (a, b) in [(0.8f32, 0.9f32), (0.3, 0.4), (1.0, 1.0), (0.0, 0.7)] {
        // `Add` sem truncar, e depois o meio — que é o que um `field.remap(multiplier=0,5)`
        // faz à coluna.
        let chained = blend(a, b, 1, false) * 0.5;
        let direct = blend(a, b, MODE_AVERAGE, true);
        assert!(
            (chained - direct).abs() < 1e-6,
            "({a}, {b}): a cadeia deu {chained} e o modo {direct}"
        );
        // E o CONTROLE: com o clamp LIGADO a cadeia mente, que era o estado anterior.
        if a + b > 1.0 {
            assert!(
                (blend(a, b, 1, true) * 0.5 - direct).abs() > 1e-3,
                "({a}, {b}): com o clamp ligado a cadeia tinha de discordar"
            );
        }
    }
}

/// **UM NÓ ACABADO DE LARGAR AINDA TRUNCA** — o default é o que o artista recebe.
#[test]
fn a_freshly_dropped_node_still_clamps() {
    let spec = MANIFEST
        .params
        .iter()
        .find(|p| p.name == CLAMP)
        .expect("o param existe");
    assert!(
        spec.default >= 0.5,
        "o clamp nasce LIGADO — todo grafo autorado é byte-idêntico: {}",
        spec.default
    );
}

/// **OS DOIS CONTROLES NOVOS ESTÃO NO PAINEL**, e o enum tem os nove nomes.
#[test]
fn the_panel_shows_the_toggle_and_the_ninth_mode() {
    let clamp = PARAM_HINTS
        .iter()
        .find(|h| h.param == CLAMP)
        .expect("o Clamp tem de estar pintado");
    assert!(matches!(clamp.widget, ParamWidget::Toggle));
    let mode = PARAM_HINTS
        .iter()
        .find(|h| h.param == "mode")
        .expect("o Mode existe");
    match mode.widget {
        ParamWidget::Enum { labels } => {
            assert_eq!(labels.len(), 9, "nove modos: {labels:?}");
            assert_eq!(labels[MODE_AVERAGE as usize], "Average");
            // ⚠️ E o teto do slider tem de alcançar o índice novo, senão o modo existe no
            // cozimento e é inalcançável pelo painel.
            assert!(
                mode.max >= MODE_AVERAGE as f32,
                "o enum vai até {} e o modo novo é {MODE_AVERAGE}",
                mode.max
            );
        }
        _ => panic!("o Mode tem de ser um Enum"),
    }
}

/// **O KERNEL DECLARA O PARAM NOVO** — sem isto o device leria lixo onde a CPU lê o toggle,
/// e as duas metades divergiriam em silêncio.
#[test]
fn the_device_is_told_about_the_clamp() {
    assert!(
        GPU_KERNEL.params.contains(&CLAMP),
        "o kernel tem de receber o clamp: {:?}",
        GPU_KERNEL.params
    );
    // E o WGSL tem de ter o braço do modo novo (o número, não o nome).
    assert!(
        GPU_KERNEL.wgsl_lib.contains("mode == 8"),
        "o braço da média falta no shader"
    );
}
