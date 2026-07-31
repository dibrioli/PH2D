//! **Um fade em CADA ponta divide a travessia da costura** (Enio, 2026-07-31: *"se o
//! usuário coloca um em cada ponta acontece um pequeno problema: o fade da direita é
//! descartado e o objeto simplesmente fica parado enquanto o playhead encontra-se ali"*).
//!
//! Os dois arquivos irmãos (`loop_wrap.rs`, `loop_wrap_out.rs`) já provam **uma** ponta de
//! cada vez, e é exatamente por isso que o defeito sobreviveu: as fixtures deles armam um
//! fade e deixam o outro em zero, e com um só o dono da costura é óbvio. Com os DOIS,
//! ninguém era dono e o desenho antigo escolhia a pose da ÚLTIMA strip para os dois lados —
//! o que faz o fade de saída cruzar **para ele mesmo** e congelar.
//!
//! ⚠️ O oráculo é a POSE ao longo da janela, nunca a regra: um teste que afirmasse *"o
//! `hold_at` devolve duas fontes"* ficaria verde sobre um objeto parado.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

/// Um frame a 60 fps — a distância em que um salto de loop é medido.
const FRAME: f64 = 1.0 / 60.0;

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f64 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    let e = ph2d_ecs::Entity::from_bits(bits);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

struct Scene {
    sim: SimWorld,
    st: TimelineState,
    bits: u64,
}

/// A foto do report: **Main** com um fade EXTERNO à esquerda e **Clip 2** com um fade
/// EXTERNO à direita, num loop `[0, 8]`. Os dois clipes são chapados (Main em `−3`, Clip 2
/// em `+5`), então toda variação de pose que se medir é a TRAVESSIA, e não a animação de
/// dentro de um clipe.
///
/// ⚠️ **As pontas são posicionadas para que o fade ALCANCE a costura** (`t_start = lead_in`,
/// `t_end = 8 − lead_out`), que é o que a foto mostra e o que faz `lead = 0` cair no caso
/// SEM vão: com um vão no topo do loop nem o desenho antigo nem o novo têm o que cruzar, e
/// o controle abaixo mediria outra coisa.
fn both_ends_fade(lead_in: f64, lead_out: f64) -> Scene {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Loop")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Main".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 4.0, -3.0);
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, PropKind::TranslationX, 0.0, 5.0);
    key(doc, bits, PropKind::TranslationX, 4.0, 5.0);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let main = doc.add_strip(lane, 0, lead_in, 4.0).unwrap();
    let clip2 = doc.add_strip(lane, c2, 4.0, 8.0 - lead_out).unwrap();
    doc.strip_mut(lane, main).unwrap().lead_in = lead_in;
    doc.strip_mut(lane, clip2).unwrap().lead_out = lead_out;
    doc.set_active_loop_for(false, Some((0.0, 8.0)));
    Scene { sim, st, bits }
}

/// **O fade da direita ANDA** — era ele que o desenho antigo descartava.
///
/// Com as duas janelas do mesmo tamanho (0,5 s cada) a travessia `+5 → −3` se parte ao
/// meio: a saída leva o objeto até o ponto médio (**+1**) antes da volta, e a entrada
/// termina o caminho depois dela. As três asserções são independentes de propósito — o
/// fade da direita move, a volta é contínua, e o da esquerda termina —, porque um desenho
/// que sacrificasse qualquer uma delas passaria nas outras duas.
#[test]
fn both_outward_fades_split_the_seam_journey() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = both_ends_fade(0.5, 0.5);

    let at_end = x_at(&mut sim, &mut st, bits, 7.5); // a Clip 2 acabou de terminar
    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0);
    let at_start = x_at(&mut sim, &mut st, bits, 0.5); // a Main começa a tocar limpa

    assert!(
        (at_end - 5.0).abs() < 0.1,
        "a travessia começa na pose da Clip 2 (+5): {at_end}"
    );
    // O DEFEITO reportado: sem a divisão, isto fica em +5 o trecho inteiro.
    assert!(
        (before_wrap - 1.0).abs() < 0.4,
        "o fade da DIREITA tem de levar o objeto até o meio do caminho (+1); \
         parado em +5 é o fade descartado do report: {before_wrap}"
    );
    // A volta continua sem salto — os dois lados concordam sobre a pose da costura.
    let jump = (after_wrap - before_wrap).abs();
    assert!(
        jump < 0.3,
        "a costura tem de ser contínua: {before_wrap} -> {after_wrap} (salto {jump})"
    );
    assert!(
        (at_start + 3.0).abs() < 0.1,
        "e o fade da ESQUERDA termina a travessia na pose da Main (−3): {at_start}"
    );
}

/// **A divisão é PROPORCIONAL às duas janelas**, e é isso que faz os dois casos de UMA
/// ponta só continuarem exatamente como eram.
///
/// Com a saída três vezes maior que a entrada, a costura cai a três quartos do caminho
/// (`+5 → −3` ⇒ `−1`), não no meio. Sem esta asserção, uma divisão fixa em 50% passaria no
/// gate acima e mentiria em toda geometria assimétrica — que é a comum.
#[test]
fn the_split_follows_the_two_window_lengths() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = both_ends_fade(0.2, 0.6);
    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    assert!(
        (before_wrap + 1.0).abs() < 0.5,
        "janela de saída 3x a de entrada ⇒ a costura cai a 3/4 do caminho (−1): {before_wrap}"
    );
}

/// **CONTROLE: uma ponta só continua fazendo a travessia INTEIRA.** É o que os dois
/// arquivos irmãos já provam, repetido aqui sobre a MESMA fixture — sem isto, uma divisão
/// que sempre partisse ao meio passaria nos gates acima e quebraria em silêncio o caso que
/// o Enio disse que *"funciona muito bem"*.
#[test]
fn a_lone_outward_fade_still_makes_the_whole_journey() {
    // Só a SAÍDA: ela leva o objeto até a pose da Main (−3) antes da volta.
    let Scene {
        mut sim,
        mut st,
        bits,
    } = both_ends_fade(0.0, 0.5);
    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    assert!(
        (before_wrap + 3.0).abs() < 0.3,
        "sozinho, o fade de saída chega ao início da Main (−3): {before_wrap}"
    );

    // Só a ENTRADA: a volta acontece na pose da Clip 2 (+5) e a travessia é toda depois.
    let Scene {
        mut sim,
        mut st,
        bits,
    } = both_ends_fade(0.5, 0.0);
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0);
    assert!(
        (after_wrap - 5.0).abs() < 0.3,
        "sozinho, o fade de entrada começa na pose da Clip 2 (+5): {after_wrap}"
    );
}
