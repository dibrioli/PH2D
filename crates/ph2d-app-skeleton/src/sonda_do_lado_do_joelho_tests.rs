//! ⏱️ **TROCAR O CHIP DO LADO MUDA A POSE DO JOELHO?** — o report do dono, 2026-09-18:
//! *«IK Bend não funcionou com Auto IK e trocando CCw por CW no painel lateral»*.
//!
//! ⚠️ **A fiação do chip está completa** (medido: `fase_bus_clicks` resolve o índice para a variante
//! e `fase_bone_ik_and_limits` escreve `g.bend = lado`), logo **a hipótese da fiação já caiu**. Esta
//! sonda mede o que sobra: *a lei responde ao campo?*

use crate::goal::{braco, osso};
use ph2d_ecs::{Name, Transform};
use ph2d_skeleton_ecs::{BendSide, IkGoal};

/// Monta o braço com âncora, põe o alvo onde o dono o arrastaria, e devolve a pose do cotovelo com
/// o lado pedido.
fn cotovelo_com(lado: BendSide, alvo_em: [f32; 2]) -> [f32; 2] {
    let (mut sim, [ombro, cotovelo]) = braco();
    let alvo = crate::goal::add(&mut sim, cotovelo).expect("a ancora nasce na ponta");
    // O alvo tem de estar FORA da recta, senão os dois lados dão a mesma pose por construção.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(alvo) {
        t.translation = ph2d_core::Vec2::new(alvo_em[0], alvo_em[1]);
    }
    if let Some(mut g) = sim.world_mut().get_mut::<IkGoal>(cotovelo) {
        g.bend = lado;
    }
    let mut preview = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut preview);
    // ⛔⛔ **A RÉGUA É A ROTAÇÃO, e a 1.ª redacção media a TRANSLAÇÃO** — o solver escreve ângulos, e
    // a translação de um osso filho é fixa (`(length, 0)` do pai). Ela lia `[10, 0]` nos três lados
    // e acusava o produto pelo report do dono. *Uma régua que mede o campo que a lei não escreve
    // dá sempre o mesmo número* — a quarta vez nesta sequência.
    let ang = |e| sim.world().get::<Transform>(e).map_or(0.0, |t| t.rotation);
    [ang(ombro), ang(cotovelo)]
}

/// ⭐⭐⭐ **A MEDIÇÃO: `Ccw` e `Cw` põem o cotovelo em sítios DIFERENTES?**
#[test]
fn trocar_o_lado_muda_a_pose_do_cotovelo() {
    let ccw = cotovelo_com(BendSide::Ccw, [14.0, 3.0]);
    let cw = cotovelo_com(BendSide::Cw, [14.0, 3.0]);
    let keep = cotovelo_com(BendSide::Keep, [14.0, 3.0]);
    let d = (ccw[0] - cw[0]).hypot(ccw[1] - cw[1]);
    println!("rotacoes — Ccw {ccw:?} · Cw {cw:?} · Keep(Auto) {keep:?} · distancia {d:.6}");
    assert!(
        d > 1e-3,
        "Ccw e Cw poem o cotovelo no MESMO sitio (distancia {d:.6}): trocar o chip nao muda \\
         nada, que e' o report do dono a' letra"
    );
}

/// ⚠️ **E a segunda metade do report: o osso que carrega a restrição é a PONTA.**
///
/// ⛔ O chip escreve no `IkGoal` do **osso seleccionado**; se o artista escolher outro osso da
/// corrente, o `get_mut` devolve `None` e **nada acontece, em silêncio** — que é indistinguível de
/// um chip que não funciona.
#[test]
fn so_o_osso_da_ponta_carrega_a_restricao() {
    let (mut sim, [ombro, cotovelo]) = braco();
    crate::goal::add(&mut sim, cotovelo).expect("a ancora nasce na ponta");
    assert!(
        sim.world().get::<IkGoal>(cotovelo).is_some(),
        "a ponta nao ficou com a restricao"
    );
    assert!(
        sim.world().get::<IkGoal>(ombro).is_none(),
        "o PAI tambem ficou com uma restricao: entao o chip funcionaria em qualquer osso, e o \\
         report do dono teria outra causa"
    );
    // ⚠️ E um osso solto da cena, pela mesma razão.
    let solto = osso(&mut sim, "Free", [50.0, 0.0], 10.0, None);
    assert!(sim.world().get::<IkGoal>(solto).is_none());
    let _ = Name::new("x");
}

/// ⭐⭐⭐ **A REPRODUÇÃO DO REPORT, na geometria da CENA do dono** — três ossos **em S** (juntas
/// `[-1,0, +1,0]`), que é o que o `PH2D_VEC_BONE_SMOKE=1` monta, e o lado capturado por ela é `Cw`
/// (lido do log dela).
///
/// ⚠️ **A fixtura tem de ser a da cena que o dono usou** — a quinta vez que este repo o cobra. Um
/// braço de dois ossos rectos responde a outra pergunta.
fn braco_em_s(chain: u32, lado: BendSide) -> Vec<f32> {
    let mut sim = ph2d_ecs::SimWorld::default();
    let a = osso(&mut sim, "A", [0.0, 0.0], 10.0, None);
    let b = osso(&mut sim, "B", [10.0, 0.0], 10.0, Some(a));
    let c = osso(&mut sim, "C", [10.0, 0.0], 10.0, Some(b));
    // O S: as duas juntas para lados opostos.
    for (e, r) in [(b, -1.0_f32), (c, 1.0_f32)] {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
            t.rotation = r;
        }
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let alvo = crate::goal::add(&mut sim, c).expect("a ancora nasce na ponta");
    if let Some(mut g) = sim.world_mut().get_mut::<IkGoal>(c) {
        g.bend = lado;
        g.chain = chain;
    }
    let _ = alvo;
    let mut preview = ph2d_preview_drive::PreviewDrive::default();
    crate::goal::solve(&mut sim, &mut preview);
    [a, b, c]
        .into_iter()
        .map(|e| sim.world().get::<Transform>(e).map_or(0.0, |t| t.rotation))
        .collect()
}

/// ⭐⭐⭐ **O REPORT REPRODUZIDO, E A LEI QUE ELE EXPÔS: com o `Chain` de FÁBRICA, `Auto` **É** `Cw`.**
///
/// ⛔⛔⛔ Este é o gate que explica *«IK Bend não funcionou com Auto IK e trocando CCw por CW»*: a
/// cena do osso captura `Cw`, e a `2` (o valor de nascimento do `Chain`) o `Auto` dá a **MESMA pose,
/// ao bit**. ⇒ o artista clica em **dois** dos quatro chips e não vê nada — *só o `Ccw` move*.
///
/// ⚠️ **E isso NÃO é um defeito da lei:** o `Auto` significa *«deriva o lado da pose que chega»*, e
/// a pose que chega está em `Cw`. A cura é o painel **dizer** (`rotulo_do_auto`, em
/// `ph2d-panel-skeleton`), nunca mexer no solver.
///
/// ⚠️ **As TRÊS metades são três leituras diferentes do mesmo report**, e nenhuma sozinha o explica.
#[test]
fn o_chain_decide_quanto_o_lado_muda() {
    let medir = |chain: u32| {
        let ccw = braco_em_s(chain, BendSide::Ccw);
        let cw = braco_em_s(chain, BendSide::Cw);
        let auto = braco_em_s(chain, BendSide::Keep);
        let soma = |a: &Vec<f32>, b: &Vec<f32>| -> f32 {
            a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
        };
        (soma(&ccw, &cw), soma(&auto, &cw))
    };
    let (ccw_cw_2, auto_cw_2) = medir(2);
    let (ccw_cw_3, auto_cw_3) = medir(3);
    println!(
        "chain 2: Ccw↔Cw {ccw_cw_2:.4} · Auto↔Cw {auto_cw_2:.4} | \
         chain 3: Ccw↔Cw {ccw_cw_3:.4} · Auto↔Cw {auto_cw_3:.4}"
    );
    // (1) A lei RESPONDE ao chip: trocar Ccw↔Cw move a corrente, nos dois `chain`.
    for (c, d) in [(2, ccw_cw_2), (3, ccw_cw_3)] {
        assert!(
            d > 1.0,
            "com chain={c}, Ccw e Cw poem a corrente quase no mesmo sitio ({d:.4}): entao o chip \
             NAO funciona, e o report do dono e' um defeito de lei e nao de rotulo"
        );
    }
    // (2) E a razão do report: a `2` o `Auto` **é** o `Cw`, ao bit.
    assert!(
        auto_cw_2 < 1e-6,
        "a chain=2 o Auto deixou de coincidir com o Cw ({auto_cw_2:.6}): a premissa do rotulo \
         `Auto (CW)` morreu, e ele passa a dizer uma coisa que nao acontece"
    );
    // (3) ⚠️ E o CONTROLO: a `3` ele DEIXA de coincidir — sem esta metade, alguem leria «o Auto é
    // sempre o Cw» e esconderia um dos chips.
    assert!(
        auto_cw_3 > 1.0,
        "a chain=3 o Auto continua a coincidir com o Cw ({auto_cw_3:.4}): entao a coincidencia nao \
         e' do `chain` e a explicacao do report esta' errada"
    );
}
