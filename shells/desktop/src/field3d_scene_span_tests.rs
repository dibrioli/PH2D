//! ⭐⭐⭐ **O ALCANCE DOS CONTROLOS DO PAINEL** — de quem ele é, e o que o pode mexer.
//!
//! # Por que um arquivo irmão
//!
//! O `field3d_scene_gesture_tests` responde a *«o que um gesto faz à pose»*; este responde a *«até
//! onde cada controlo vai, e quem decide isso»*. O corte é por assunto, e nasceu do tecto de LOC do
//! shell quando o report do zoom entrou (Enio, 2026-08-30). ⛔ *Split, nunca allowlist.*

/// ⭐⭐⭐ **A CÂMERA NÃO TEM VOTO NOS CONTROLOS DO OBJETO** — report do Enio, 2026-08-30: *«o ZOOM
/// muda os parâmetros do objeto no painel»*.
///
/// ⚠️ **Ele mudava mesmo**, e não era uma ilusão: o alcance de cada slider saía de
/// `cam.half_extent * 2.0`, então rodar a roda re-escalava a faixa de **todas** as linhas — a
/// posição, a largura, a banda da dobra. Quem estivesse a arrastar uma delas via o número mudar de
/// escala debaixo do dedo, e ajustar a mesma coisa com dois enquadramentos dava dois resultados.
///
/// ⚠️ **A régua é o TEXTO da chamada**, e não um número: o defeito é uma *dependência*, e uma
/// dependência lê-se na chamada. Um gate de valor precisaria de duas câmeras e de um quadro inteiro
/// para provar o que esta linha diz sozinha.
#[test]
fn the_panel_span_never_reads_the_camera() {
    let fonte = include_str!("field3d_scene.rs");
    let i = fonte
        .find("publish_snapshot(")
        .expect("a cena publica o snapshot do painel");
    let chamada = &fonte[i..fonte[i..].find(");").map_or(fonte.len(), |j| i + j)];
    assert!(
        chamada.contains("gesture_span"),
        "o alcance dos sliders tem de vir da PEÇA (`panel::gesture_span`): {chamada}"
    );
    assert!(
        !chamada.contains("cam"),
        "⛔ a câmera voltou a decidir o alcance dos controlos do objeto: {chamada}"
    );
}

/// ⭐⭐ **E ele também não se mexe enquanto o artista arrasta uma largura** — a metade simétrica.
///
/// ⛔ Um alcance **contínuo** na peça curaria o report e traria o defeito espelhado: arrastar uma
/// largura mudaria o alcance, e o botão fugiria do dedo. Por isso a lei é em **oitavas** — dentro de
/// uma, o alcance é uma constante.
#[test]
fn the_gesture_span_holds_still_inside_an_octave() {
    use crate::field3d_scene::panel::gesture_span;
    // ⚠️ **Uma oitava de verdade**: com `4×` de folga, os raios cujo alvo cai em `(1, 2]` são
    // `(0,25 · 0,5]` — e a 1.ª versão deste gate metia `0,55` no meio deles, que já é a oitava
    // seguinte. *A fixtura é que estava errada, e o gate acusou-a antes do código.*
    let base = gesture_span(0.26);
    for r in [0.26f32, 0.30, 0.35, 0.40, 0.45, 0.50] {
        assert!(
            (gesture_span(r) - base).abs() < f32::EPSILON,
            "o alcance mexeu-se dentro de uma oitava ({r} → {}, contra {base})",
            gesture_span(r)
        );
    }
    // ⭐ E ele **cobre** a peça com folga: um slider que acaba antes da forma é inútil.
    for r in [0.05f32, 0.3, 1.0, 3.0, 10.0] {
        assert!(
            gesture_span(r) >= r,
            "o alcance ({}) não chega à própria peça ({r})",
            gesture_span(r)
        );
    }
    // ⚠️ **O piso**: uma peça minúscula não pode dar um slider cujo curso inteiro é invisível.
    assert!(
        gesture_span(0.0) >= 1.0,
        "sem piso, uma peça de raio zero daria um alcance zero"
    );
    // ⛔ **O CONTROLE**: se ele fosse constante, o gate acima passaria sem nada a defender.
    assert!(
        gesture_span(10.0) > gesture_span(0.3),
        "uma peça dez vezes maior tem de ter um alcance maior"
    );
    // ⭐ E a oitava seguinte **existe**: passar do fim de uma tem de mover o alcance uma vez, e
    // exactamente para o dobro.
    assert!(
        (gesture_span(0.51) - base * 2.0).abs() < f32::EPSILON,
        "a oitava seguinte tem de ser o dobro ({} contra {})",
        gesture_span(0.51),
        base * 2.0
    );
}
