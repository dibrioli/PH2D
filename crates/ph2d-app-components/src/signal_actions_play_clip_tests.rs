//! ⭐⭐ **O verbo `Play Clip`** (doc ObjectsAndComponents 01 §14, item 1 — escolha 4.1 do dono:
//! *«os outros tocam quando algo pedir — estado, sinal, script»*). Irmão de [`super`] por tecto de LOC.
//!
//! A costura inteira: o sinal soa → a resolução acha o alvo → a ponte põe o
//! [`ph2d_ecs::ClipPlayback`] com o relógio DESSE quadro → o `apply_autoplay` da timeline toca o
//! clipe pedido a partir dali.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{
    ClipPlayback, Disparo, Entity, Name, SignalAction, SignalActions, SignalVerb, SimWorld,
    Transform, assign_missing_stable_ids,
};
use ph2d_preview_drive::PreviewDrive;
use ph2d_tags::TagTree;
use ph2d_timeline::{PropKind, TimelineDoc};

use super::{Som, apply};

fn mudo() -> impl FnMut(&mut SimWorld, Som, Entity) -> bool {
    |_, _, _| false
}

fn linha(on: &str, target: &str, arg: &str) -> SignalAction {
    SignalAction {
        on: on.into(),
        target: target.into(),
        verb: SignalVerb::PlayClip,
        arg: arg.into(),
        ..SignalAction::default()
    }
}

/// O quadro da shell (`fase_tabela_de_accoes`): resolve os sinais e aplica-os no relógio `t`.
fn quadro(sim: &mut SimWorld, sinais: &[&str], t: f64) -> super::ActionReport {
    let disparos: Vec<Disparo<'_>> = sinais.iter().map(|n| Disparo::anonimo(n)).collect();
    let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &TagTree::new(), &disparos);
    apply(sim, &efeitos, t, &mut PreviewDrive::default(), &mut mudo())
}

/// Um Botão que, ao ouvir `pula`, pede ao Herói o clipe `arg`.
fn cena(arg: &str) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let heroi = w.spawn((Transform::IDENTITY, Name::new("Herói"))).id();
    let botao = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Botão"),
            SignalActions(vec![linha("pula", "Herói", arg)]),
        ))
        .id();
    assign_missing_stable_ids(w);
    (sim, heroi, botao)
}

/// ⭐⭐⭐ **Um sinal dispara ⇒ o ALVO ganha o pedido, datado com o relógio DESSE quadro** — e um
/// segundo sinal TROCA o pedido (não acumula).
///
/// **Mutações que devem sangrar:** tirar o braço `PlayClip` do `apply` · `desde: 0.0` em vez de `t`
/// · escrever o pedido em `fx.source` em vez de `fx.target` · não aparar o nome.
#[test]
fn um_sinal_poe_no_alvo_o_clipe_pedido_desde_o_instante_dele() {
    let (mut sim, heroi, botao) = cena(" pular ");
    let r = quadro(&mut sim, &["pula"], 2.5);
    assert_eq!((r.applied, r.inert), (1, 0));
    assert_eq!(
        sim.world().get::<ClipPlayback>(heroi),
        Some(&ClipPlayback {
            clip: "pular".into(),
            desde: 2.5
        }),
        "o alvo não ganhou o pedido com o relógio do quadro do sinal"
    );
    assert!(
        sim.world().get::<ClipPlayback>(botao).is_none(),
        "o pedido caiu em quem REAGIU e não no alvo"
    );

    sim.world_mut()
        .get_mut::<SignalActions>(botao)
        .expect("a tabela")
        .0[0]
        .arg = "andar".into();
    quadro(&mut sim, &["pula"], 4.0);
    assert_eq!(
        sim.world().get::<ClipPlayback>(heroi),
        Some(&ClipPlayback {
            clip: "andar".into(),
            desde: 4.0
        }),
        "o segundo pedido não substituiu o primeiro"
    );
}

/// ⛔ **Um nome em branco é INERTE** — nenhum pedido, e o relatório di-lo (a regra dos irmãos).
///
/// **Mutação que deve sangrar:** apagar a guarda `clip.is_empty()`.
#[test]
fn um_nome_em_branco_nao_pede_nada() {
    let (mut sim, heroi, _) = cena("   ");
    let r = quadro(&mut sim, &["pula"], 1.0);
    assert_eq!((r.applied, r.inert), (0, 1));
    assert!(sim.world().get::<ClipPlayback>(heroi).is_none());
}

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn x_de(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("transform")
        .translation
        .x
}

/// ⭐⭐⭐ **De ponta a ponta com a timeline:** o Herói toca o clipe com Autoplay (`andar`, x 0→10) sozinho; o
/// sinal em `t = 1` pede `pular` (x 0→−10), e a partir dali ele toca `pular` no relógio DO PEDIDO
/// (`t = 1.5` ⇒ 0,5 s de `pular`). Um scrub para antes do pedido volta à entrada.
///
/// **Mutações que devem sangrar:** `desde: 0.0` (o `pular` tocaria a 1,5 s e não a 0,5 s) · tirar o
/// braço `PlayClip` (o Herói continuaria a andar).
#[test]
fn o_alvo_passa_a_tocar_o_clipe_pedido_a_partir_do_instante_do_sinal() {
    let (mut sim, heroi, _) = cena("pular");
    let dono = heroi.to_bits();
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), dono).expect("andar");
    let pular = doc.add_clip_for("pular".into(), dono).expect("pular");
    for (clip, fim) in [(andar, 10.0), (pular, -10.0)] {
        doc.set_active(clip);
        for (t, x) in [(0.0, 0.0), (2.0, fim)] {
            doc.insert_key(
                dono,
                PropKind::TranslationX,
                s(t),
                AnimValue::Float(x),
                Interp::Linear,
            );
        }
    }
    // O «andar» com Autoplay (por omissão nenhum — o do Godot).
    doc.set_autoplay(andar, true);
    let toca = |sim: &mut SimWorld, t: f64| {
        assert_eq!(
            ph2d_timeline::apply_autoplay(sim.world_mut(), &doc, t, |_| false, &mut Vec::new()),
            1
        );
    };

    toca(&mut sim, 0.5);
    assert!(
        (x_de(&sim, heroi) - 2.5).abs() < 1e-4,
        "a entrada toca sozinha"
    );

    quadro(&mut sim, &["pula"], 1.0);
    toca(&mut sim, 1.5);
    assert!(
        (x_de(&sim, heroi) + 2.5).abs() < 1e-4,
        "x = {} — o pedido não toca `pular` desde o instante do sinal",
        x_de(&sim, heroi)
    );

    toca(&mut sim, 0.8);
    assert!(
        (x_de(&sim, heroi) - 4.0).abs() < 1e-4,
        "antes do pedido (scrub) volta a entrada"
    );
}
