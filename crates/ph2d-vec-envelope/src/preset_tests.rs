//! Gates dos presets de gaiola (ADR-0129 Fatia C). Módulo irmão do [`super`] — teto de LOC.
//!
//! A tabela de barrigas mora no `ph2d_ecs::EnvelopeWarp` (é dado); esta crate não a conhece. Então
//! os gates aqui usam barrigas **escritas à mão**, e o irmão que varre os 7 presets reais vive no
//! shell, onde as duas metades se encontram.

use super::*;
use crate::cage_folds;

/// A barriga do *Bulge*: os dois lados horizontais para FORA.
const BULGE: EdgeBows = [[AMP, AMP], [0.0, 0.0], [AMP, AMP], [0.0, 0.0]];

/// **`bend = 0` é a gaiola em REPOUSO, ao bit.** O preset com força zero não é "quase" a identidade
/// — é ela. Sem isto, arrastar o slider até o zero deixaria um resíduo, e o artista veria a forma
/// não voltar.
/// Sem shear — o argumento novo de `preset_cage` que quase todo preset passa em zero.
const NO_SHIFT: [[f64; 2]; 4] = [[0.0; 2]; 4];

#[test]
fn a_zero_bend_is_the_rest_cage_exactly() {
    let (corners, edges) = preset_cage(&BULGE, &NO_SHIFT, 0.0);
    assert_eq!(corners, UNIT_CAGE);
    assert_eq!(edges, rest_edges(&UNIT_CAGE));
}

/// **Um preset NÃO move canto.** Ele enverga os lados — e é isso que mantém a bbox da gaiola estável
/// quando o slider anda, e que torna a convexidade dos cantos um não-assunto aqui.
#[test]
fn a_preset_never_moves_a_corner() {
    for bend in [-1.0, -0.5, 0.25, 1.0] {
        let (corners, _) = preset_cage(&BULGE, &NO_SHIFT, bend);
        assert_eq!(
            corners, UNIT_CAGE,
            "o preset mexeu num canto em bend={bend}"
        );
    }
}

/// **A barriga sai pela normal EXTERNA, e o sinal manda.** No *Bulge* positivo o lado de baixo desce
/// (para fora, `y < 0`) e o de cima sobe; no negativo, o contrário. É a asserção que dá sentido à
/// tabela: se a normal fosse a interna, todo preset sairia espelhado e ninguém notaria numa forma
/// simétrica.
#[test]
fn a_positive_bow_leaves_the_cage_by_the_outward_normal() {
    let (_, out) = preset_cage(&BULGE, &NO_SHIFT, 1.0);
    assert!(out[0][0][1] < 0.0, "o lado de baixo devia descer: {out:?}");
    assert!(out[2][0][1] > 1.0, "o lado de cima devia subir: {out:?}");
    let (_, inn) = preset_cage(&BULGE, &NO_SHIFT, -1.0);
    assert!(
        inn[0][0][1] > 0.0 && inn[2][0][1] < 1.0,
        "bend negativo não inverteu: {inn:?}"
    );
}

/// **O `bend` é LINEAR na barriga** — meia força é meio deslocamento. Um slider que acelerasse não
/// seria lido como "o quanto"; e a linearidade é o que faz a garantia de não-dobra no extremo valer
/// para todo o intervalo.
#[test]
fn the_bend_is_linear() {
    let (_, half) = preset_cage(&BULGE, &NO_SHIFT, 0.5);
    let (_, full) = preset_cage(&BULGE, &NO_SHIFT, 1.0);
    let rest = rest_edges(&UNIT_CAGE);
    let d_half = half[0][0][1] - rest[0][0][1];
    let d_full = full[0][0][1] - rest[0][0][1];
    assert!(
        (d_full - 2.0 * d_half).abs() < 1e-12,
        "meia força não deu meio deslocamento: {d_half} vs {d_full}"
    );
}

/// **A faixa do slider é FECHADA.** Um `bend` além de ±1 é clampado — a garantia de não-dobra é
/// sobre essa faixa, e um chamador distraído não pode comprá-la de volta.
#[test]
fn the_bend_range_is_clamped() {
    let (_, at_one) = preset_cage(&BULGE, &NO_SHIFT, 1.0);
    let (_, beyond) = preset_cage(&BULGE, &NO_SHIFT, 7.5);
    assert_eq!(at_one, beyond, "bend fora da faixa não foi clampado");
}

/// **Nenhuma barriga da amplitude máxima dobra o patch** — nem sozinha, nem as duas em oposição.
///
/// É o que separa um slider honesto de um que "para de funcionar" no fim do curso: a alça do gesto
/// Mesh para na fronteira porque a MÃO pode pedir o impossível; a faixa de um preset é **desenhada**,
/// então ela nunca pede. O irmão que varre os 7 presets de verdade está no shell (`ph2d-host-desktop`,
/// `envelope_kind_tests`) — esta crate não conhece a tabela deles.
#[test]
fn a_full_amplitude_bow_never_folds() {
    let cases: [(&str, EdgeBows); 3] = [
        ("bulge", BULGE),
        (
            // Os dois lados horizontais vindo UM CONTRA O OUTRO — o caso que mais aperta o patch.
            "pinch",
            [[-AMP, -AMP], [0.0, 0.0], [-AMP, -AMP], [0.0, 0.0]],
        ),
        (
            // Todos os quatro lados para dentro ao mesmo tempo.
            "all-in",
            [[-AMP, -AMP], [-AMP, -AMP], [-AMP, -AMP], [-AMP, -AMP]],
        ),
    ];
    for (name, bows) in cases {
        for step in -20..=20 {
            let bend = f64::from(step) / 20.0;
            let (corners, edges) = preset_cage(&bows, &NO_SHIFT, bend);
            assert!(
                !cage_folds(&corners, &edges),
                "o preset {name} dobrou em bend={bend}"
            );
        }
    }
}

/// **E o amostrador VÊ uma dobra quando a amplitude passa do teto** — a metade presença. Sem ela, o
/// gate acima ficaria verde num `cage_folds` que responde `false` sempre, e o `AMP` seria um número
/// sem justificação.
#[test]
fn an_amplitude_past_the_ceiling_does_fold() {
    let over = 4.0 * AMP;
    let bows: EdgeBows = [[-over, -over], [0.0, 0.0], [-over, -over], [0.0, 0.0]];
    let (corners, edges) = preset_cage(&bows, &NO_SHIFT, 1.0);
    assert!(
        cage_folds(&corners, &edges),
        "barriga de {over} não dobrou — o teto AMP={AMP} não está a defender nada"
    );
}
