//! Os gates do **LIMITE DE ÂNGULO** — a quarta fatia do [`super`], pelo corte por RESPONSABILIDADE
//! que este ficheiro já fez três vezes.
//!
//! A LEI (aparar um ângulo num arco) está gateada em `ph2d-skeleton`. Aqui mede-se o que só existe
//! com um mundo ECS: que o limite apara **as duas** mãos que giram um osso (o dedo e o solver), que
//! ele é no-op quando não existe, e — o risco real desta wave — que o par *solver + limite* **assenta
//! em vez de vibrar**.

use super::*;
use crate::goal::braco;
use ph2d_skeleton_ecs::BoneLimit;

/// Põe um limite em `bone` e devolve a faixa em radianos.
fn limita(sim: &mut SimWorld, bone: Entity, min: f64, max: f64) {
    sim.world_mut()
        .entity_mut(bone)
        .insert(BoneLimit { min, max });
}

fn rot(sim: &SimWorld, e: Entity) -> f64 {
    f64::from(sim.world().get::<Transform>(e).expect("tem pose").rotation)
}

/// ⭐⭐⭐ **O LIMITE APARA O QUE O DEDO PEDE** — não só o que o solver pede.
///
/// ⚠️ É a metade que o Godot não tem: lá o limite vive na restrição de IK, então o gesto de girar o
/// osso à mão atravessa-o. Um limite que obedece ou não conforme QUEM moveu a junta é pior que
/// limite nenhum — o artista não consegue formar um modelo do que a ferramenta faz.
#[test]
fn the_limit_clamps_what_the_finger_asks_for_not_only_the_solver() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.2, 0.2);
    // Aponta o ombro para muito acima do que o limite permite.
    assert!(crate::bone_pose::pose(
        &mut sim,
        ombro,
        [0.0, 10.0],
        ph2d_skeleton_render::BonePart::Body,
    ));
    let r = rot(&sim, ombro);
    assert!(
        (r - 0.2).abs() < 1e-6,
        "o dedo pediu ~90 graus e a junta devia parar em 0,2 rad — parou em {r}"
    );
}

/// ⭐ **SEM LIMITE, O GESTO É O DE SEMPRE AO BIT** — a lei da casa, e a prova de que a junta sem
/// limite (a esmagadora maioria) não paga nada por esta wave existir.
#[test]
fn a_bone_without_a_limit_moves_exactly_as_before() {
    let alvo = [3.0, 7.0];
    let (mut sim_a, [a, _]) = braco();
    let (mut sim_b, [b, _]) = braco();
    limita(
        &mut sim_b,
        b,
        -ph2d_skeleton::FULL_TURN / 2.0,
        ph2d_skeleton::FULL_TURN / 2.0,
    );
    assert!(crate::bone_pose::pose(
        &mut sim_a,
        a,
        alvo,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(crate::bone_pose::pose(
        &mut sim_b,
        b,
        alvo,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert_eq!(
        rot(&sim_a, a),
        rot(&sim_b, b),
        "um limite da volta inteira mudou a pose — ele devia ser o no-op exacto"
    );
}

/// ⭐⭐⭐ **O LIMITE APARA O SOLVER, e a ponta deixa de alcançar o alvo — que é o CERTO.**
///
/// ⚠️ Uma restrição de IK que atravessasse o limite para chegar ao alvo é precisamente o defeito:
/// o membro alcança por um caminho que um corpo não faz. O Blender responde igual — com *IK
/// limits*, a ponta fica onde a anatomia deixa.
#[test]
fn the_limit_stops_the_solver_and_the_tip_falls_short_on_purpose() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    // Sem limite a corrente alcança um alvo lá em cima.
    let alto = [4.0, 16.0];
    quadro(&mut sim, cotovelo, alto);
    let livre = (ponta(&sim, cotovelo)[0] - alto[0]).hypot(ponta(&sim, cotovelo)[1] - alto[1]);
    // Com o ombro preso perto de zero, ela não chega lá.
    limita(&mut sim, ombro, -0.05, 0.05);
    for _ in 0..8 {
        quadro(&mut sim, cotovelo, alto);
    }
    let preso = (ponta(&sim, cotovelo)[0] - alto[0]).hypot(ponta(&sim, cotovelo)[1] - alto[1]);
    assert!(
        preso > livre + 1.0,
        "com o ombro limitado a ponta devia ficar LONGE do alvo (livre {livre}, preso {preso})"
    );
    assert!(
        rot(&sim, ombro).abs() <= 0.05 + 1e-6,
        "o solver atravessou o limite do ombro: {}",
        rot(&sim, ombro)
    );
}

/// ⭐⭐⭐ **O PAR *SOLVER + LIMITE* ASSENTA, E NÃO VIBRA** — o risco real desta wave.
///
/// ⚠️ O solver resolve, o limite apara, e o quadro seguinte parte da pose **aparada**. Se aparar
/// mudasse a entrada da resolução seguinte o bastante para ela pedir outra coisa, a junta oscilaria
/// a 60 Hz entre duas poses — e nada num teste que resolva UMA vez o veria.
///
/// ⚠️ A régua é o movimento entre quadros **decrescer**, não ser zero: o FABRIK sai cedo quando o
/// erro cai abaixo da tolerância, então uma corrente que ainda refina é sã. *Convergir e oscilar são
/// coisas diferentes* — foi a mesma correcção que a régua do lado da dobra precisou.
#[test]
fn a_limited_chain_settles_instead_of_oscillating() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    limita(&mut sim, ombro, -0.3, 0.3);
    limita(&mut sim, cotovelo, -0.8, 0.8);
    let alvo = [6.0, 14.0];
    let mut movs = Vec::new();
    let mut ant = (rot(&sim, ombro), rot(&sim, cotovelo));
    for _ in 0..12 {
        quadro(&mut sim, cotovelo, alvo);
        let agora = (rot(&sim, ombro), rot(&sim, cotovelo));
        movs.push((agora.0 - ant.0).abs().max((agora.1 - ant.1).abs()));
        ant = agora;
    }
    // ⛔⛔ **A régua ERA VÁCUA, e a medição di-lo** (auditoria de 2026-09-08). Ela lia `cedo =
    // movs[1]` e exigia `movs[11] <= cedo`; a série real desta fixtura é
    // `[0,800000011920929, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]` — a corrente bate na parede no quadro
    // **0** e fica **exactamente** parada. ⇒ a asserção era `0 <= 0`, verde por construção, sobre
    // uma fixtura que não pode produzir o fenómeno que o nome do gate promete.
    //
    // ⇒ duas metades, as duas medidas: a fixtura **moveu-se** (senão não há convergência a medir), e
    // o movimento tardio é **desprezável contra o primeiro**. ⛔ Não «não cresceu»: uma oscilação a
    // amplitude constante também não cresce.
    let cedo = movs[0];
    assert!(
        cedo > 1e-9,
        "a fixtura não produziu movimento nenhum (série {movs:?}) — não há convergência a medir"
    );
    let tarde = movs[11];
    assert!(
        tarde <= cedo / 100.0,
        "a corrente limitada não ASSENTOU: o 1.º quadro moveu {cedo} e o 12.º ainda move {tarde} \
         (série {movs:?}) — uma oscilação a amplitude constante passa por «não cresceu»"
    );
    assert!(
        rot(&sim, ombro).abs() <= 0.3 + 1e-6 && rot(&sim, cotovelo).abs() <= 0.8 + 1e-6,
        "uma das juntas acabou fora do próprio limite"
    );
}

/// ⭐⭐⭐ **A CENA DE SMOKE DÁ AO DONO UMA PAREDE PARA SENTIR — e um vizinho livre ao lado.**
///
/// ⚠️ É o irmão do `the_smoke_scene_gives_the_anchor_a_side_to_defend`, e existe pela mesma razão
/// (`CLAUDE.md` §5.0): uma cena em que **tudo** tem limite não distingue *«o limite funciona»* de
/// *«o osso não roda»*, e uma em que **nada** tem deixa o dono a clicar num botão sem ver efeito.
/// O que ensina é o CONTRASTE, e é ele que este gate fixa.
#[test]
fn the_smoke_scene_has_one_limited_bone_and_a_free_neighbour() {
    use ph2d_skeleton_demo::TENTACLE_LIMIT_HALF;
    // A cena põe o limite no `TENTACLE_LIMITED_BONE`-ésimo osso e deixa os outros livres. Uma
    // corrente de dois ossos reproduz a estrutura: o limitado e o vizinho.
    let (mut sim, [livre, preso]) = braco();
    limita(&mut sim, preso, -TENTACLE_LIMIT_HALF, TENTACLE_LIMIT_HALF);
    let longe = [0.0, 20.0];
    assert!(crate::bone_pose::pose(
        &mut sim,
        preso,
        longe,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(crate::bone_pose::pose(
        &mut sim,
        livre,
        longe,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(
        rot(&sim, preso).abs() <= TENTACLE_LIMIT_HALF + 1e-6,
        "o osso limitado passou a parede: {}",
        rot(&sim, preso)
    );
    assert!(
        rot(&sim, livre).abs() > TENTACLE_LIMIT_HALF + 1e-6,
        "o vizinho devia girar LIVRE, e parou em {} — sem contraste o smoke não ensina nada",
        rot(&sim, livre)
    );
}

#[path = "bone_limit_gizmo_tests.rs"]
mod gizmo;

/// ⭐⭐⭐ **O GESTO *Add Angle Limit* NÃO MOVE UM GRAU** — a afirmação que o doc dele faz e que
/// ninguém media.
///
/// ⛔⛔ **`add_limit` shipou com ZERO gates** (auditoria de 2026-09-08), com o doc a afirmar *«É um
/// no-op exacto na mesma, e não por sorte»* sobre nada. O irmão da família tem o gate desde que
/// existe (`adding_an_anchor_moves_nothing`) — e **é o mesmo report**: um verbo que move a pose ao
/// ser carregado lê-se como *«o botão estragou o meu rig»*.
///
/// ⚠️ Duas mutações que a suíte inteira deixava passar antes deste gate: `meia = 0.0` (o botão
/// **congela** a junta para sempre) e `min: -meia, max: meia` (o botão dá um **solavanco** no osso).
#[test]
fn adding_a_limit_moves_nothing_and_is_centred_on_the_pose() {
    let (mut sim, [_, cotovelo]) = braco();
    for pose in [0.0_f32, 0.9, -2.5] {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(cotovelo) {
            t.rotation = pose;
        }
        sim.world_mut().entity_mut(cotovelo).remove::<BoneLimit>();
        assert!(
            crate::bone_limit::add_limit(&mut sim, cotovelo),
            "o verbo recusou numa junta sem limite"
        );
        assert!(
            (rot(&sim, cotovelo) - f64::from(pose)).abs() < 1e-9,
            "carregar em Add Angle Limit MOVEU o osso de {pose} para {}",
            rot(&sim, cotovelo)
        );
        let l = *sim
            .world()
            .get::<BoneLimit>(cotovelo)
            .expect("o limite nasceu");
        assert!(
            l.min < f64::from(pose) && f64::from(pose) < l.max,
            "a pose ({pose}) não ficou DENTRO da faixa que nasceu ({}..{}) — o osso trava no \
             instante em que o artista carrega no botão",
            l.min,
            l.max
        );
        // ⚠️ E a faixa tem largura: `meia = 0` seria uma junta congelada, e a asserção de cima
        // sozinha não a apanha (ela usa `<` estrito, mas uma faixa de largura 1e-12 passaria).
        assert!(
            l.max - l.min > 1.0,
            "a faixa nasceu com {} rad de largura — a junta está praticamente congelada",
            l.max - l.min
        );
    }
    // E o verbo recusa a segunda vez: só pode haver um limite por junta.
    assert!(
        !crate::bone_limit::add_limit(&mut sim, cotovelo),
        "o verbo aceitou um SEGUNDO limite na mesma junta"
    );
}

/// ⭐⭐⭐ **UMA PAREDE NUNCA PASSA A OUTRA** — nem pelo arrasto, nem pelos CAMPOS.
///
/// ⛔⛔ **O guarda existia no gesto e não nos campos** (auditoria de 2026-09-08): digitar
/// `Limit Min = 90` com `Limit Max = 45` escrevia cru, e a lei lê `min > max` como faixa de
/// meia-largura **zero** ⇒ a junta congela no ponto médio. ⚠️ E o desenho **normaliza** os dois
/// extremos (`arc()` ordena-os), então o canvas continua a pintar um sector perfeitamente normal
/// enquanto o osso não roda um grau — *nada na tela explicava o que aconteceu*.
///
/// ⇒ a cura não foi escrever o guarda no segundo sítio, foi as duas superfícies passarem pela mesma
/// porta ([`crate::bone_limit::set_edge`]).
#[test]
fn a_wall_never_crosses_its_neighbour_from_either_surface() {
    for (is_max, pedido) in [(false, 3.0_f64), (true, -3.0)] {
        let mut l = BoneLimit {
            min: -0.5,
            max: 0.5,
        };
        crate::bone_limit::set_edge(&mut l, is_max, pedido);
        assert!(
            l.min <= l.max,
            "empurrar a parede {} para {pedido} inverteu a faixa ({}..{}) — a junta congela e o \
             desenho continua bonito",
            if is_max { "MAX" } else { "MIN" },
            l.min,
            l.max
        );
        // A parede para NA vizinha, não a atravessa nem a arrasta.
        let (esperado_min, esperado_max) = if is_max { (-0.5, -0.5) } else { (0.5, 0.5) };
        assert!(
            (l.min - esperado_min).abs() < 1e-12 && (l.max - esperado_max).abs() < 1e-12,
            "a parede empurrada não parou na vizinha: {}..{}",
            l.min,
            l.max
        );
    }
}
