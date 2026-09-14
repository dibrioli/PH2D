//! Os gates do **LADO DA DOBRA** — a terceira fatia do [`super`], pelo teto de 600 LOC do HR-18 e
//! pelo mesmo corte por RESPONSABILIDADE das outras duas (a agenda, o que o dedo aponta).
//!
//! Aqui mede-se uma coisa só: *para que lado o joelho aponta, e quem decide isso*. A LEI está
//! gateada em `ph2d-skeleton` (incluindo a prova de que a fixtura PRODUZ a inversão); o que só
//! existe com um mundo ECS é a **captura no nascimento** e o defeito de ponta a ponta.

use super::*;
use crate::goal::braco;
// ⚠️ Declarado aqui: o pai deixou de nomear o `Bone` quando a lei foi para a folha.
use ph2d_skeleton_ecs::Bone;

/// O mesmo braço, mas **dobrado** por `theta` radianos no cotovelo.
///
/// ⚠️ O [`braco`] nasce **recto**, e uma corrente recta **não tem lado** — ela não produz o
/// fenómeno que os dois gates de baixo medem. É a mesma armadilha que a fila deste módulo já
/// registou quatro vezes, e é por isso que esta fixtura existe em vez de um `theta` emprestado.
fn braco_dobrado(theta: f32) -> (SimWorld, [Entity; 2]) {
    let (mut sim, ossos) = braco();
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ossos[1]) {
        t.rotation = theta;
    }
    (sim, ossos)
}

/// ⭐⭐⭐ **A ÂNCORA NASCE A DEFENDER A DOBRA QUE O ARTISTA JÁ POSOU.**
///
/// ⚠️ **O lado é CAPTURADO, não escolhido** — e é isso que faz criar a âncora continuar a ser um
/// no-op visual **e** curar o defeito ao mesmo tempo. Se ela nascesse em `Keep`, a primeira vez que
/// o dono esticasse o membro o joelho inverteria sozinho.
///
/// ⛔ E o lado esperado sai da **própria fixtura** (dobras opostas ⇒ capturas opostas), nunca de um
/// nome que eu escolhi: foi assim que a 1.ª redacção do `the_elbow_keeps_the_side_it_is_already_
/// bent_to` ficou verde a pinar a inversão que dizia proibir.
#[test]
fn the_bend_side_is_captured_when_the_anchor_is_born() {
    let mut capturados = Vec::new();
    for theta in [0.5_f32, -0.5] {
        let (mut sim, [_, cotovelo]) = braco_dobrado(theta);
        add(&mut sim, cotovelo).expect("a âncora nasce");
        let g = *sim
            .world()
            .get::<IkGoal>(cotovelo)
            .expect("a âncora foi escrita no osso da ponta");
        assert_ne!(
            g.bend,
            ph2d_skeleton::BendSide::Keep,
            "uma corrente DOBRADA (theta={theta}) tem lado, e ele tinha de ser capturado"
        );
        capturados.push(g.bend);
    }
    assert_eq!(
        capturados[0],
        capturados[1].flipped(),
        "dobras opostas capturaram o MESMO lado ({capturados:?}) — a captura não está a ler a pose"
    );
}

/// ⭐ **Uma corrente RECTA não tem lado, e a âncora não inventa um.**
///
/// ⛔ Escolher um lado ali seria fabricar uma decisão do artista a partir de ruído de `f32`: perto
/// da extensão máxima um erro de posição de `1e-6` vira `1e-3` de ângulo, e o sinal dele é sorteio.
#[test]
fn a_straight_chain_gives_the_anchor_no_side_to_defend() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    assert_eq!(
        sim.world().get::<IkGoal>(cotovelo).expect("existe").bend,
        ph2d_skeleton::BendSide::Keep
    );
}

/// ⭐⭐⭐ **O DEFEITO DO DONO, DE PONTA A PONTA NO MUNDO** — e é o gate que fecha a wave.
///
/// O artista dobra o cotovelo, **estica o membro** arrastando a âncora para longe (a corrente fica
/// recta, e uma recta não tem lado) e traz a mão de volta ao mesmo sítio. Com o lado capturado ela
/// volta **ao mesmo lado**; sem ele, ao contrário.
///
/// ⚠️ A régua é a da LEI ([`ph2d_skeleton::dominant_side`]), e não um produto vectorial escrito
/// aqui: os dois medem o mesmo facto com **sinais opostos**, e escrever o segundo foi o que fez a
/// 1.ª redacção do gate irmão acusar a implementação certa.
#[test]
fn the_anchor_defends_its_side_across_the_straight_pose() {
    for theta in [0.5_f32, -0.5] {
        let (mut sim, [ombro, cotovelo]) = braco_dobrado(theta);
        add(&mut sim, cotovelo).expect("a âncora nasce");
        let poiso = ponta(&sim, cotovelo);

        let lado_agora = |sim: &SimWorld| {
            let juntas = [
                ph2d_skeleton_live::skin_live::bone_segments(sim)
                    .iter()
                    .find(|(x, _, _)| *x == ombro.to_bits())
                    .map(|(_, a, _)| *a)
                    .expect("o ombro tem segmento"),
                ponta(sim, ombro),
                ponta(sim, cotovelo),
            ];
            ph2d_skeleton::dominant_side(&juntas, juntas[2])
        };

        let antes = lado_agora(&sim);
        assert!(
            antes.abs() > 0.0,
            "a fixtura tem de ter lado (theta={theta}), senão este gate é vácuo"
        );
        // Estica: o alvo vai muito para além do alcance ⇒ a corrente deita-se na recta.
        quadro(&mut sim, cotovelo, [400.0, 0.0]);
        // E volta exactamente para onde estava.
        quadro(&mut sim, cotovelo, poiso);
        let depois = lado_agora(&sim);
        assert!(
            antes.signum() == depois.signum(),
            "o joelho INVERTEU ao passar pela recta (theta={theta}): {antes} -> {depois}"
        );
    }
}

/// ⭐⭐⭐ **A CENA DE SMOKE DÁ À ÂNCORA UM LADO PARA DEFENDER** — e sem este gate ela não daria.
///
/// ⛔⛔ **O `CLAUDE.md` §5.0 nomeia este modo de falha e ele quase aconteceu aqui:** o braço da cena
/// nascia **RECTO**, e uma corrente recta **não tem lado**. O `add` capturaria `Auto` — que é
/// precisamente o modo em que o joelho inverte ao passar pela posição esticada — e o dono faria o
/// smoke da cura vendo o defeito, com tudo verde deste lado.
///
/// ⚠️ Ele mede a MESMA geometria que a cena monta (a tabela `ARM_*`), nunca uma cópia dos números:
/// uma sonda que alimenta outros números mede outro programa.
#[test]
fn the_smoke_scene_gives_the_anchor_a_side_to_defend() {
    use ph2d_skeleton_demo::{ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND};
    let mut sim = SimWorld::default();
    let raiz = ph2d_skeleton_demo::cadeia(&mut sim, ARM_A, ARM_B, ARM_BONES)
        .expect("a cadeia do braço monta-se");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // A cena dobra o ÚLTIMO osso — é ele que está dentro da corrente de `DEFAULT_CHAIN`.
    let mut ponta = raiz;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(ponta).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<Bone>(**c).is_some())
            .copied()
    }) {
        ponta = f;
    }
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ponta) {
        t.rotation = ARM_ELBOW_BEND;
    }
    add(&mut sim, ponta).expect("a âncora nasce na ponta do braço");
    let g = *sim.world().get::<IkGoal>(ponta).expect("a âncora existe");
    assert_ne!(
        g.bend,
        ph2d_skeleton::BendSide::Keep,
        "a cena de smoke monta o braço sem dobra: o dono arrastaria o losango e veria o joelho \
         INVERTER, que é o defeito que esta wave cura. Ponha `ARM_ELBOW_BEND` fora do zero."
    );
}

/// Uma corrente de quatro ossos em ZIG-ZAG — a pose que só o modo MISTO sabe defender.
fn zigzag() -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::default();
    let mut ossos = Vec::new();
    for i in 0..4 {
        let pai = ossos.last().copied();
        let pos = if i == 0 { [0.0, 0.0] } else { [10.0, 0.0] };
        let e = crate::goal::osso(&mut sim, &format!("B{i}"), pos, 10.0, pai);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
            t.rotation = if i % 2 == 0 { 0.35 } else { -0.35 };
        }
        ossos.push(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, ossos)
}

/// O sinal de cada junta interior, lido dos segmentos VIVOS.
fn lados(sim: &SimWorld) -> Vec<f64> {
    let segs = ph2d_skeleton_live::skin_live::bone_segments(sim);
    (1..segs.len())
        .map(|i| {
            let (_, a0, b0) = segs[i - 1];
            let (_, a1, b1) = segs[i];
            let v1 = [b0[0] - a0[0], b0[1] - a0[1]];
            let v2 = [b1[0] - a1[0], b1[1] - a1[1]];
            let cruz = v1[0] * v2[1] - v1[1] * v2[0];
            if cruz.abs() <= 1e-9 {
                0.0
            } else {
                cruz.signum()
            }
        })
        .collect()
}

/// ⭐⭐⭐ **OS LADOS DO MODO MISTO SOBREVIVEM A UM ARRASTO PARA FORA DO ALCANCE.**
///
/// ⛔⛔ **É o defeito que a pose viva não podia evitar:** fora do alcance a resposta certa é a
/// RECTA — e uma corrente recta não tem lado nenhum para ler. Lendo os lados da pose viva, o quadro
/// seguinte não tinha o que defender e as juntas caíam para onde o solver quisesse, **para sempre**
/// e sem nada na tela a dizê-lo. Lendo-os do que o artista DESENHOU, o braço volta como estava.
///
/// ⚠️ **O `PreviewDrive` é UM só, atravessando os três quadros** — e tem de ser: é ele que guarda o
/// autorado enquanto o motor escreve por cima. Um ledger novo por quadro (o idioma das outras
/// fixturas deste módulo) mediria um app que não existe.
#[test]
fn a_mixed_chain_survives_a_drag_out_of_reach() {
    let (mut sim, ossos) = zigzag();
    let tip = *ossos.last().expect("a corrente tem ossos");
    add(&mut sim, tip).expect("a âncora nasce");
    {
        let mut g = sim
            .world_mut()
            .get_mut::<IkGoal>(tip)
            .expect("a âncora foi escrita");
        g.bend = ph2d_skeleton::BendSide::Mixed;
        g.chain = 4;
    }
    let autorados = lados(&sim);
    assert!(
        autorados.iter().any(|&s| s > 0.0) && autorados.iter().any(|&s| s < 0.0),
        "a fixtura nao e' um zig-zag: {autorados:?}"
    );
    let mut pv = PreviewDrive::default();
    let perto = [22.0, 6.0];
    for alvo in [perto, [4000.0, 0.0], perto] {
        assert!(drag_anchor(&mut sim, tip, alvo), "a ancora existe");
        solve(&mut sim, &mut pv);
    }
    assert_eq!(
        lados(&sim),
        autorados,
        "depois de ir e voltar do fora-de-alcance, a corrente perdeu os lados que o artista desenhou"
    );
}

/// ⭐⭐⭐ **A CENA DE SMOKE DISTINGUE OS TRÊS MODOS DE LADO** — sem isto ela ensinaria que eles não
/// existem.
///
/// ⛔⛔ **É o modo de falha que o `CLAUDE.md` §5.0 nomeia, e ele estava a um passo:** com **uma** só
/// junta dobrada, `Ccw`, `Cw` e `Mixed` entregam a **mesma** pose — o dono escolheria os três,
/// veria a mesma coisa, e concluiria que o `IK Bend` não faz nada. O braço abre em **S**
/// (`ARM_SHOULDER_BEND` oposto ao `ARM_ELBOW_BEND`) e o `IK Chain` sobe a `3` para as duas juntas
/// ficarem sob a âncora.
///
/// ⚠️ Ele mede a MESMA geometria que a cena monta (a tabela `ARM_*`), nunca uma cópia dos números.
#[test]
fn the_smoke_scene_tells_the_three_bend_modes_apart() {
    use ph2d_skeleton_demo::{ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND, ARM_SHOULDER_BEND};
    let montar = || {
        let mut sim = SimWorld::default();
        let raiz = ph2d_skeleton_demo::cadeia(&mut sim, ARM_A, ARM_B, ARM_BONES)
            .expect("a cadeia do braço monta-se");
        let meio = sim
            .world()
            .get::<ph2d_ecs::Children>(raiz)
            .and_then(|c| c.iter().next().copied())
            .expect("o braço tem três ossos");
        let mut ponta = meio;
        while let Some(f) = sim.world().get::<ph2d_ecs::Children>(ponta).and_then(|c| {
            c.iter()
                .find(|c| sim.world().get::<Bone>(**c).is_some())
                .copied()
        }) {
            ponta = f;
        }
        for (e, r) in [(meio, ARM_SHOULDER_BEND), (ponta, ARM_ELBOW_BEND)] {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
                t.rotation = r;
            }
        }
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        (sim, ponta)
    };
    let (sim0, _) = montar();
    let autorados = lados(&sim0);
    assert!(
        autorados.iter().any(|&s| s > 0.0) && autorados.iter().any(|&s| s < 0.0),
        "a cena não põe o braço em S: as juntas são {autorados:?}, e os três modos dariam a mesma \
         pose. Ponha `ARM_SHOULDER_BEND` com o sinal oposto ao `ARM_ELBOW_BEND`."
    );
    let mut poses = Vec::new();
    for modo in [
        ph2d_skeleton::BendSide::Ccw,
        ph2d_skeleton::BendSide::Cw,
        ph2d_skeleton::BendSide::Mixed,
    ] {
        let (mut sim, ponta) = montar();
        add(&mut sim, ponta).expect("a âncora nasce");
        if let Some(mut g) = sim.world_mut().get_mut::<IkGoal>(ponta) {
            g.bend = modo;
            g.chain = 3;
        }
        let mut pv = PreviewDrive::default();
        assert!(drag_anchor(&mut sim, ponta, [-4.0, 5.0]), "a ancora existe");
        solve(&mut sim, &mut pv);
        poses.push((modo, lados(&sim)));
    }
    let misto = &poses[2].1;
    assert_eq!(
        *misto, autorados,
        "o MISTO não guardou os lados que a cena desenhou"
    );
    for (modo, lados_do_modo) in &poses[..2] {
        assert_ne!(
            lados_do_modo, misto,
            "o {modo:?} entrega os MESMOS lados que o MISTO nesta cena: o dono escolheria os dois \
             e veria a mesma pose"
        );
    }
}
