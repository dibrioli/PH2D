//! ⭐⭐⭐ **O ALCANCE DOS CONTROLOS DO PAINEL** — de quem ele é, e o que o pode mexer.
//!
//! # Por que um arquivo irmão
//!
//! O `field3d_scene_gesture_tests` responde a *«o que um gesto faz à pose»*; este responde a *«até
//! onde cada controlo vai, e quem decide isso»*. O corte é por assunto, e nasceu do tecto de LOC do
//! shell quando o report do zoom entrou (Enio, 2026-08-30). ⛔ *Split, nunca allowlist.*

use super::*;

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
        chamada.contains("latched_span"),
        "o alcance dos sliders tem de vir da PEÇA e vir TRAVADO (`panel::latched_span`): {chamada}"
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
            "o alcance mexeu-se dentro de uma oitava ({r} -> {}, contra {base})",
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

/// ⭐⭐⭐ **E ELE NÃO SE MEXE ENQUANTO A MÃO ESTÁ NO CONTROLO** — o report que a 1.ª versão desta
/// wave causou: *«arrastar os sliders ficou bizarro mudando valores aos pulos»*.
///
/// ⛔ A oitava resolvia o caso comum e **saltava para o dobro** quando virava, a meio do arrasto: a
/// escala do mapeamento cursor→valor caía a metade e o número pulava. *Trocar um incómodo contínuo
/// por um salto discreto é pior* — com a câmera, o alcance ao menos era constante durante o gesto.
#[test]
fn the_span_holds_while_the_hand_is_on_the_control() {
    use crate::field3d_scene::panel::{gesture_span, latched_span_for};
    // Uma seleção qualquer, e uma peça que CRESCE muito durante o arrasto.
    let alvo = 7_u64;
    let inicial = latched_span_for(alvo, 0.30);
    for raio in [0.30f32, 0.45, 0.60, 0.90, 1.50, 3.00, 9.00] {
        assert!(
            (latched_span_for(alvo, raio) - inicial).abs() < f32::EPSILON,
            "o alcance mexeu-se com a peça a crescer (raio {raio} -> {}, contra {inicial})",
            latched_span_for(alvo, raio)
        );
    }
    // ⛔ **O CONTROLE**: sem a trava, esses raios dariam alcances diferentes — se `gesture_span`
    // fosse constante, o gate acima passaria sem nada a defender.
    assert!(
        (gesture_span(9.00) - gesture_span(0.30)).abs() > f32::EPSILON,
        "a lei de base tem de responder ao tamanho — senão a trava não defende nada"
    );
    // ⭐ E escolher OUTRO objeto re-ajusta: a trava é por seleção, não para sempre.
    let outro = latched_span_for(alvo + 1, 9.00);
    assert!(
        (outro - inicial).abs() > f32::EPSILON,
        "trocar de seleção tem de re-ajustar o alcance ({outro} contra {inicial})"
    );
}

/// ⭐⭐⭐ **COM A MÃO PARADA, O VALOR TEM DE FICAR PARADO** — o gate que a auditoria de 2026-08-30
/// nomeou, e o único que vê o defeito que o report do Enio é.
///
/// # ⛔⛔ O mecanismo: um LAÇO DE REALIMENTAÇÃO, e não um salto
///
/// Três factos que sozinhos parecem certos fecham um laço:
///
/// 1. o slider mapeia `valor = lo + track·(hi − lo)`, com `track` a **posição absoluta** do dedo;
/// 2. o alcance saía de `gesture_span(piece_radius())`;
/// 3. `piece_radius` é o raio da peça — que é **o que este slider escreve**.
///
/// ⇒ `v ← t · span(4v)`, e para `t > ¼` a recorrência **diverge**. Medido no caminho de produção,
/// com o ponteiro **completamente parado** em `track = 0,8`: `2,4 → 9,6 → 19,2 → … → 2 457,6`. Em
/// vinte quadros, `track = 0,76` leva o valor a `1 090 518` e `track = 1,0` a `1,07e9`.
///
/// ⚠️ **A câmera era um insumo EXTERNO ao objecto; a peça não é.** Trocar uma pela outra fechou o
/// laço — e é por isso que a cura não é escolher melhor a fonte, é **travar** o alcance enquanto a
/// mão está no controlo.
///
/// ⚠️ **Uma função pura nunca revela um laço.** Os dois gates que nasceram com a wave do zoom medem
/// `gesture_span` isolada, com o raio FIXO — e ficam verdes sobre a divergência. Só a **composição
/// fechada** (valor → peça → alcance → valor) a mostra.
#[test]
fn the_value_is_a_fixed_point_when_the_hand_does_not_move() {
    use crate::field3d_scene::panel::{latched_span_for, param_rows};
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    sync_scene(&mut sim, Some(&scene(1)), 0.0);
    let root = the_root(&mut sim);
    let world = sim.world_mut();
    let alvo = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .nth(1)
        .expect("a cena tem um filho");
    // O dedo pousa a 80 % da trilha e **não se mexe mais**.
    const TRACK: f32 = 0.8;
    let mut valores = Vec::new();
    for quadro in 0..20 {
        // O alcance do quadro, pela MESMA porta que o produto usa.
        let cozida = ph2d_field_ecs::cook(world, root)
            .expect("há peça")
            .expect("a peça coze");
        let raio = ph2d_field_eval::bounds::bounding_ball(
            &cozida,
            &ph2d_field_eval::hybrid::Registry::new(),
        )
        .map_or(0.0, |b| b.radius);
        let span = latched_span_for(alvo.to_bits(), raio);
        let linhas = param_rows(world, Some(alvo), span);
        let Some(linha) = linhas.iter().find(|r| r.key == "field.dim.pos_x") else {
            // Sem uma posição nesta cena não há o que medir — o controle abaixo apanha-o.
            break;
        };
        let (lo, hi) = (linha.lo, linha.bound.value());
        let v = lo + TRACK * (hi - lo);
        valores.push(v);
        let _ = quadro;
        ph2d_field_ecs::set_param(world, alvo, linha.param, v).ok();
    }
    assert!(
        valores.len() >= 10,
        "só {} quadros — a fixtura não tem uma linha de posição para medir",
        valores.len()
    );
    let primeiro = valores[0];
    let ultimo = *valores.last().expect("mediu");
    assert!(
        (ultimo - primeiro).abs() <= primeiro.abs() * 0.01 + 1.0e-4,
        "com a mão PARADA o valor andou de {primeiro} para {ultimo} em {} quadros — o alcance do \
         slider está a ser derivado do valor que o próprio slider escreve",
        valores.len()
    );
}
