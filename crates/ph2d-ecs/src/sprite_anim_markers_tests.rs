use super::*;
use crate::timer::step_ticks;
use crate::{AnimDirection, FrameRef, SpriteAnimFrames, advance_frames};

const FRAME_MS: u32 = 100;

fn tag(direction: AnimDirection, repeat: Option<u32>) -> AnimationTag {
    AnimationTag {
        name: "walk".into(),
        from: 0,
        to: 3,
        frame_ms: FRAME_MS,
        direction,
        repeat,
        hold_ms: 0,
        repeat_delay_ms: 0,
        signal_on_finish: String::new(),
        per_frame_ms: Vec::new(),
        signal_on_loop: String::new(),
        color: 0,
        sheet: 0,
    }
}

fn marca(step: u32) -> SpriteAnimMarkers {
    SpriteAnimMarkers(vec![FrameMarker {
        tag: "walk".into(),
        step,
        signal: "foot".into(),
    }])
}

fn tocando() -> SpriteAnimator {
    let mut a = SpriteAnimator::new("walk".to_owned());
    a.playing = true;
    a
}

/// Toca `tiques` passos de `ms` e devolve quantos gritos houve.
fn gritos(
    t: &AnimationTag,
    list: Option<&[FrameRef]>,
    m: &SpriteAnimMarkers,
    ms: f64,
    tiques: u32,
) -> usize {
    let (mut a, mut frame, mut step) =
        (tocando(), list.map_or(FrameRef::cell(t.from), |l| l[0]), 0);
    let mut out = Vec::new();
    let dt = step_ticks(ms / 1000.0);
    for _ in 0..tiques {
        advance_frames_marked(
            &mut a,
            t,
            list,
            &mut frame,
            &mut step,
            &[16],
            dt,
            Some(m),
            &mut out,
        );
    }
    out.len()
}

/// 4 passos de 100 ms = 400 ms por ciclo. 10 ciclos em tiques de 10 ms: os tiques caem exatamente nas
/// fronteiras, e a volta do laço cruza o quadro `0` no tique exato. O passo `0` é o caso que conta em dobro
/// (o começo mais a volta) ou em falta, se a lei errar.
#[test]
fn marcador_dispara_uma_vez_por_ciclo() {
    let t = tag(AnimDirection::Forward, None);
    for passo in 0..4 {
        // 4000 ms cobrem 10 ciclos inteiros; o tique seguinte entra no ciclo 11.
        assert_eq!(
            gritos(&t, None, &marca(passo), 10.0, 399),
            10,
            "passo {passo}"
        );
    }
}

/// O tique grande atravessa vários passos de uma vez: nenhum grito se perde, e o total não muda.
#[test]
fn um_tique_atrasado_nao_perde_nem_duplica_o_marcador() {
    let t = tag(AnimDirection::Forward, None);
    for passo in 0..4 {
        assert_eq!(
            gritos(&t, None, &marca(passo), 130.0, 30),
            10,
            "passo {passo}"
        );
    }
}

#[test]
fn marcador_vale_com_lista_de_quadros() {
    let t = tag(AnimDirection::Forward, None);
    let lista = FrameRef::run(0, [7, 2, 9]);
    for passo in 0..3 {
        assert_eq!(
            gritos(&t, Some(&lista), &marca(passo), 10.0, 299),
            10,
            "passo {passo}"
        );
    }
}

#[test]
fn animacao_que_nao_repete_grita_o_primeiro_e_o_ultimo_uma_vez() {
    let t = tag(AnimDirection::Forward, Some(1));
    assert_eq!(gritos(&t, None, &marca(0), 10.0, 1000), 1);
    assert_eq!(gritos(&t, None, &marca(3), 10.0, 1000), 1);
}

#[test]
fn outra_tag_ou_sinal_vazio_nao_gritam() {
    let t = tag(AnimDirection::Forward, None);
    let outra = SpriteAnimMarkers(vec![FrameMarker {
        tag: "idle".into(),
        step: 1,
        signal: "foot".into(),
    }]);
    assert_eq!(gritos(&t, None, &outra, 10.0, 400), 0);
    let vazio = SpriteAnimMarkers(vec![FrameMarker {
        tag: "walk".into(),
        step: 1,
        signal: String::new(),
    }]);
    assert_eq!(gritos(&t, None, &vazio, 10.0, 400), 0);
}

#[test]
fn parada_nao_grita() {
    let t = tag(AnimDirection::Forward, None);
    let (mut a, mut frame, mut step) = (tocando(), FrameRef::cell(0), 0);
    a.playing = false;
    let mut out = Vec::new();
    for _ in 0..10 {
        advance_frames_marked(
            &mut a,
            &t,
            None,
            &mut frame,
            &mut step,
            &[16],
            1000,
            Some(&marca(0)),
            &mut out,
        );
    }
    assert!(out.is_empty());
}

/// O relógio e o quadro são os mesmos com e sem marcadores: ler o marcador não muda a animação.
#[test]
fn os_marcadores_nao_mudam_a_animacao() {
    let t = tag(AnimDirection::PingPong, None);
    let (mut a, mut f, mut s) = (tocando(), FrameRef::cell(0), 0);
    let (mut b, mut g, mut r) = (tocando(), FrameRef::cell(0), 0);
    let mut out = Vec::new();
    for _ in 0..500 {
        advance_frames(&mut a, &t, None, &mut f, &mut s, &[16], 700);
        advance_frames_marked(
            &mut b,
            &t,
            None,
            &mut g,
            &mut r,
            &[16],
            700,
            Some(&marca(2)),
            &mut out,
        );
        assert_eq!((f, s, a.elapsed_ticks), (g, r, b.elapsed_ticks));
    }
}

#[test]
fn renomear_a_tag_leva_o_marcador() {
    let mut m = marca(1);
    m.rename_tag("walk", "run");
    assert_eq!(m.0[0].tag, "run");
    m.rename_tag("idle", "x");
    assert_eq!(m.0[0].tag, "run");
    let _ = SpriteAnimFrames::default();
}

// ── As edições de marcador, as mesmas que o cartão da animação pede ──

mod edicoes {
    use super::*;

    fn passos(m: &SpriteAnimMarkers, tag: &str) -> Vec<u32> {
        m.of_tag(tag).map(|x| x.step).collect()
    }

    #[test]
    fn acrescentar_pega_o_primeiro_quadro_livre_de_cada_tag() {
        let mut m = SpriteAnimMarkers::default();
        m.add("walk", 4);
        m.add("walk", 4);
        m.add("run", 4);
        assert_eq!(passos(&m, "walk"), [0, 1]);
        assert_eq!(passos(&m, "run"), [0]);
        assert!(m.0.iter().all(|x| x.signal.is_empty()));
        // Sem quadro livre, o primeiro; e uma animação que não resolve (`0` quadros) ainda tem o `0`.
        m.add("run", 1);
        assert_eq!(passos(&m, "run"), [0, 0]);
        m.add("sem", 0);
        assert_eq!(passos(&m, "sem"), [0]);
    }

    #[test]
    fn o_indice_conta_dentro_da_tag_e_o_passo_tem_teto() {
        let mut m = SpriteAnimMarkers::default();
        m.add("run", 4);
        m.add("walk", 4);
        m.add("walk", 4);
        assert_eq!(m.set_step("walk", 1, 3, 4), Some(()));
        assert_eq!(m.set_signal("walk", 1, "footstep".into()), Some(()));
        assert_eq!(m.set_step("run", 0, 99, 4), Some(()));
        let walk: Vec<_> = m.of_tag("walk").cloned().collect();
        assert_eq!((walk[0].step, walk[0].signal.as_str()), (0, ""));
        assert_eq!((walk[1].step, walk[1].signal.as_str()), (3, "footstep"));
        assert_eq!(passos(&m, "run"), [3]);
        // Um passo sobre uma animação que não resolve fica no `0`.
        assert_eq!(m.set_step("run", 0, 2, 0), Some(()));
        assert_eq!(passos(&m, "run"), [0]);
    }

    #[test]
    fn marcador_fora_nao_muda_nada() {
        let mut m = SpriteAnimMarkers::default();
        m.add("walk", 4);
        let antes = m.clone();
        assert_eq!(m.set_step("walk", 1, 2, 4), None);
        assert_eq!(m.set_signal("run", 0, "x".into()), None);
        assert_eq!(m.remove_at("walk", 1), None);
        assert!(!m.remove_tag("run"));
        assert_eq!(m, antes);
    }

    #[test]
    fn remover_tira_so_o_pedido() {
        let mut m = SpriteAnimMarkers::default();
        m.add("walk", 4);
        m.add("walk", 4);
        m.add("run", 4);
        assert_eq!(m.remove_at("walk", 0), Some(()));
        assert_eq!(passos(&m, "walk"), [1]);
        assert!(m.remove_tag("walk"));
        assert_eq!(m.of_tag("walk").count(), 0);
        assert_eq!(passos(&m, "run"), [0]);
    }
}
