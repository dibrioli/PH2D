//! Os gates do **LIMITE DE ÂNGULO** — a quarta fatia do [`super`], pelo corte por RESPONSABILIDADE
//! que este ficheiro já fez três vezes.
//!
//! A LEI (aparar um ângulo num arco) está gateada em `ph2d-skeleton`. Aqui mede-se o que só existe
//! com um mundo ECS: que o limite apara **as duas** mãos que giram um osso (o dedo e o solver), que
//! ele é no-op quando não existe, e — o risco real desta wave — que o par *solver + limite* **assenta
//! em vez de vibrar**.

use super::*;
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
    let cedo = movs[1];
    let tarde = movs[11];
    assert!(
        tarde <= cedo + 1e-9,
        "a corrente limitada está a OSCILAR: o movimento por quadro foi {cedo} e ficou {tarde} \
         (série {movs:?})"
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
    use crate::vec_bone_smoke::TENTACLE_LIMIT_HALF;
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

/// Onde o dedo tem de tocar para pegar uma parede, e o que ele apanha lá.
fn agarra(
    sim: &SimWorld,
    p: [f64; 2],
    foco: Option<Entity>,
) -> Option<ph2d_skeleton_render::BoneHover> {
    // ⚠️ **`px_to_world` REALISTA, e não `1,0`**: com um pixel por unidade, um osso de 10 unidades
    // mede 10 px e a tolerância do dedo (12 px) engole o esqueleto inteiro — todo alvo colidiria
    // com todo alvo, e o gate mediria a tolerância em vez do desenho. `0,1` põe o mesmo osso a
    // 100 px, que é a ordem em que ele de facto aparece na cena de smoke (medido: 107,52 px).
    agarra_a(sim, p, foco, PX_PERTO)
}

/// O mesmo, com o zoom escolhido — ver [`PX_LONGE`].
fn agarra_a(
    sim: &SimWorld,
    p: [f64; 2],
    foco: Option<Entity>,
    px_to_world: f64,
) -> Option<ph2d_skeleton_render::BoneHover> {
    crate::bone_gesture::hover(sim, p, px_to_world, foco.map(Entity::to_bits))
}

/// O zoom de trabalho: um osso de 10 unidades a ~100 px, que é a ordem em que ele de facto aparece
/// na cena de smoke (medido 2026-09-06: `107,52` px).
const PX_PERTO: f64 = 0.1;

/// ⭐ **O zoom AFASTADO** — o mesmo osso a ~20 px, que é o rig inteiro visível de uma vez.
///
/// ⚠️ **É só aqui que as alças se sobrepõem, e o número diz porquê:** a tolerância do dedo são
/// `12 px`, logo em MUNDO ela vale `12 × px_to_world`. A `0,1` isso são `1,2` unidades e as alças
/// da fixtura distam `5,0` — não colidem. A `0,5` a tolerância passa a `6,0` e elas colidem. *O
/// defeito da ordem era real e só aparecia quando o artista afastava a câmera*, que é exactamente
/// quando ele está a olhar o rig inteiro.
const PX_LONGE: f64 = 0.5;

/// ⭐⭐⭐ **AS DUAS PAREDES SÃO AGARRÁVEIS** — o report do dono (2026-09-07): *«os limites devem ser
/// visíveis e manipuláveis no canvas através de gizmos»*.
///
/// ⚠️ Um limite que só existe como dois números num painel é um limite **invisível**: o artista não
/// vê onde a parede está nem a pode empurrar, e tem de traduzir graus de cabeça.
#[test]
fn both_walls_of_the_limit_can_be_grabbed_on_the_canvas() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.4, 0.6);
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    for (p, esperado) in [
        (arc.handle_min, ph2d_skeleton_render::BonePart::LimitMin),
        (arc.handle_max, ph2d_skeleton_render::BonePart::LimitMax),
    ] {
        let h = agarra(&sim, p, Some(ombro)).expect("o dedo apanha alguma coisa");
        assert_eq!(
            h.part, esperado,
            "em {p:?} o dedo apanhou {:?} em vez da parede",
            h.part
        );
        assert_eq!(h.bone, ombro.to_bits());
    }
}

/// ⛔ **NADA AGARRÁVEL ONDE NADA É DESENHADO** — as duas metades.
///
/// O arco só é pintado para o osso em FOCO e só existe com limite. Uma alça que respondesse fora
/// disso faria o artista pegar no vazio e o app fazer uma coisa que ele não pediu — é a mesma lei
/// que a alça da força já segue.
#[test]
fn no_wall_is_grabbable_where_none_is_drawn() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.4, 0.6);
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    // (a) sem FOCO nenhum, a parede não é do dedo.
    let h = agarra(&sim, arc.handle_min, None);
    assert!(
        h.is_none_or(|h| h.part != ph2d_skeleton_render::BonePart::LimitMin),
        "a parede foi apanhada sem o osso estar em foco"
    );
    // (b) sem LIMITE não há arco nenhum.
    let (sim2, [outro, _]) = braco();
    assert!(
        crate::bone_limit::arc(&sim2, outro, PX_PERTO).is_none(),
        "um osso sem limite devolveu um arco"
    );
}

/// ⭐⭐ **ARRASTAR UMA PAREDE ESCREVE-A**, e ela segue o dedo.
#[test]
fn dragging_a_wall_writes_the_limit() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.4, 0.6);
    // Arrasta a parede `max` para cima, para um ângulo bem maior.
    assert!(crate::bone_pose::pose(
        &mut sim,
        ombro,
        [0.0, 10.0],
        ph2d_skeleton_render::BonePart::LimitMax,
    ));
    let l = *sim
        .world()
        .get::<BoneLimit>(ombro)
        .expect("o limite continua lá");
    assert!(
        (l.max - std::f64::consts::FRAC_PI_2).abs() < 1e-6,
        "a parede devia ter ido para ~90 graus, foi para {}",
        l.max
    );
    assert!(
        (l.min - -0.4).abs() < 1e-12,
        "a OUTRA parede mexeu-se: {}",
        l.min
    );
}

/// ⛔⛔ **UMA PAREDE NUNCA ATRAVESSA A OUTRA.**
///
/// ⚠️ Sem esta cerca um puxão a mais inverteria a faixa, e a lei responde a uma faixa invertida
/// travando a junta **no centro**: o artista veria o osso saltar para o meio e deixar de rodar, sem
/// nada que explicasse porquê. Ela pára **colada** à outra — apertar até não sobrar nada é uma coisa
/// que ele pode querer; inverter não é.
#[test]
fn a_wall_never_crosses_the_other_one() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.2, 0.2);
    // Puxa o `max` para MUITO abaixo do `min`.
    assert!(crate::bone_pose::pose(
        &mut sim,
        ombro,
        [0.0, -10.0],
        ph2d_skeleton_render::BonePart::LimitMax,
    ));
    let l = *sim.world().get::<BoneLimit>(ombro).expect("existe");
    assert!(
        l.max >= l.min - 1e-12,
        "a faixa INVERTEU: min={} max={}",
        l.min,
        l.max
    );
}

/// ⭐ **O ARCO É O CAMINHO DA PONTA** — o raio é o comprimento do osso, e é isso que faz *«arrastar
/// a parede»* e *«levar a ponta até aqui»* serem o mesmo gesto.
#[test]
fn the_arc_radius_is_the_bone_length_so_the_walls_sit_where_the_tip_would() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.4, 0.6);
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    let comp = sim.world().get::<Bone>(ombro).expect("é osso").length;
    for e in [arc.edge_min, arc.edge_max] {
        let r = (e[0] - arc.apex[0]).hypot(e[1] - arc.apex[1]);
        assert!(
            (r - comp).abs() < 1e-9,
            "a parede está a {r} do vértice e o osso mede {comp}"
        );
    }
}

/// ⭐⭐⭐ **QUANDO DUAS ALÇAS SE SOBREPÕEM, GANHA A MAIS PERTO DO DEDO** — e não a primeira da lista.
///
/// ⛔⛔ **É um defeito que a 1.ª redacção tinha e este gate apanhou.** As três alças do osso em foco
/// (a força e as duas paredes) eram testadas por ORDEM, e com `strength ≈ 1` e uma parede perto de
/// 90° elas caem a menos de um dedo uma da outra: a parede ficava **inalcançável**, e o artista via
/// o triângulo e agarrava o quadrado. ⚠️ Reordenar não cura — só troca quem fica inalcançável.
///
/// ⭐ A proximidade é a única regra que não escolhe uma vítima, e este gate mede-a **dos dois
/// lados**: sobre cada alça, apanha-se aquela alça.
#[test]
fn when_two_handles_overlap_the_nearer_one_wins() {
    let (mut sim, [ombro, _]) = braco();
    // Uma parede a 90°, que é a direcção em que a alça da força também vive.
    limita(&mut sim, ombro, -0.2, std::f64::consts::FRAC_PI_2);
    // ⚠️ **O MESMO zoom que o `agarra_a` usa** — a posição da alça DEPENDE dele (a folga é de
    // tela), e calcular o arco num zoom para o apanhar noutro mede dois programas diferentes. Foi
    // exactamente esse o erro da 1.ª redacção deste gate.
    let arc = crate::bone_limit::arc(&sim, ombro, PX_LONGE).expect("o arco existe");
    // ⭐ **A FORÇA é AJUSTADA para produzir o encontro, e o valor é DERIVADO da parede** — não
    // escolhido. ⚠️ Depois de as alças saírem para fora do alcance do osso, a sobreposição deixou
    // de acontecer por acaso (elas passaram a distar `9,86`), e uma fixtura que a espere fica
    // **vácua**. O que este gate mede é a REGRA («ganha a mais perto»), então ele constrói o caso
    // em vez de torcer por ele: a alça da força é perpendicular ao eixo, a meio do osso, à
    // distância `strength × comprimento` — pôr essa distância na altura da parede encosta as duas.
    let comp = sim.world().get::<Bone>(ombro).expect("é osso").length;
    if let Some(mut o) = sim.world_mut().get_mut::<Bone>(ombro) {
        o.strength = (arc.handle_max[1] - arc.apex[1]).abs() / comp;
    }
    let (r, a, b) = crate::skeleton_live::influence_region(&sim, ombro.to_bits())
        .expect("o osso tem região de influência");
    let forca = ph2d_skeleton_render::influence_handle(a, b, r).expect("a alça da força existe");
    let d = (forca[0] - arc.handle_max[0]).hypot(forca[1] - arc.handle_max[1]);
    assert!(
        d <= crate::bone_gesture::BONE_HIT_PX * PX_LONGE,
        "a fixtura tem de PRODUZIR a sobreposição, senão este gate é vácuo: as duas alças distam {d}"
    );
    // Sobre a PAREDE, apanha-se a parede.
    assert_eq!(
        agarra_a(&sim, arc.handle_max, Some(ombro), PX_LONGE).map(|h| h.part),
        Some(ph2d_skeleton_render::BonePart::LimitMax),
    );
    // Sobre a FORÇA, apanha-se a força — a cura não pode ter roubado o gesto que já existia.
    assert_eq!(
        agarra_a(&sim, forca, Some(ombro), PX_LONGE).map(|h| h.part),
        Some(ph2d_skeleton_render::BonePart::Influence),
    );
}

/// ⭐⭐⭐ **AGARRAR A PONTA DE UM OSSO ENCOSTADO NA PAREDE APANHA O OSSO, NUNCA A PAREDE.**
///
/// ⛔⛔ **O report do dono** (2026-09-07): *«os gizmos de limite mudam de posição sozinho após mover
/// a cadeia de ossos»*. O mecanismo, medido: como o raio do arco é o comprimento do osso, quando ele
/// **encosta na parede** a ponta e a borda do setor ocupam o mesmo ponto — `distância ponta→parede =
/// 0,000000`. O artista movia a cadeia até ao limite, agarrava para continuar, e **arrastava a
/// parede**. O gizmo mexia-se sem ele o ter pedido.
///
/// ⚠️ **Priorizar a ponta sobre a parede NÃO cura — troca a vítima** (a parede ficaria inalcançável
/// exactamente quando o osso está nela). A cura é geométrica: a alça sai para **fora** do raio que o
/// osso alcança, a uma folga derivada do dedo da casa.
#[test]
fn a_wall_handle_never_sits_where_the_bone_can_reach() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.5, 0.5);
    // O artista gira o osso ATÉ BATER na parede — é aí que os dois alvos se encontravam.
    assert!(crate::bone_pose::pose(
        &mut sim,
        ombro,
        [0.0, 10.0],
        ph2d_skeleton_render::BonePart::Body,
    ));
    let seg = crate::skeleton_live::bone_segments(&sim);
    let (_, a, b) = seg
        .iter()
        .copied()
        .find(|(x, _, _)| *x == ombro.to_bits())
        .expect("o osso tem segmento");
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    let dedo = crate::bone_gesture::BONE_HIT_PX * PX_PERTO;

    // A fixtura tem de PRODUZIR o encontro: o osso está mesmo encostado na parede.
    let na_parede = (b[0] - arc.edge_max[0]).hypot(b[1] - arc.edge_max[1]);
    assert!(
        na_parede < 1e-6,
        "a fixtura não encostou o osso na parede ({na_parede}) — o gate seria vácuo"
    );

    // ⭐ Nenhuma alça cai a menos de um dedo de QUALQUER ponto do osso.
    for (h, nome) in [(arc.handle_min, "min"), (arc.handle_max, "max")] {
        let d = ph2d_skeleton::dist2_to_segment(h, a, b).sqrt();
        assert!(
            d > dedo,
            "a alça {nome} está a {d} do osso e o dedo mede {dedo} — ela rouba o gesto de girar"
        );
    }

    // ⭐⭐ E o teste que o dono faria: tocar na ponta apanha o OSSO.
    let pego = agarra(&sim, b, Some(ombro)).map(|h| h.part);
    assert!(
        !matches!(
            pego,
            Some(ph2d_skeleton_render::BonePart::LimitMin | ph2d_skeleton_render::BonePart::LimitMax)
        ),
        "ao tocar na ponta do osso encostado na parede apanhou-se {pego:?} — o artista arrastaria a \
         parede a pensar que gira o osso"
    );
}

/// ⭐ **E as alças continuam AGARRÁVEIS** — a cura não pode ter empurrado o alvo para fora do
/// alcance do dedo.
#[test]
fn the_wall_handles_are_still_grabbable_after_moving_them_out() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.5, 0.5);
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    for (h, esperado) in [
        (arc.handle_min, ph2d_skeleton_render::BonePart::LimitMin),
        (arc.handle_max, ph2d_skeleton_render::BonePart::LimitMax),
    ] {
        assert_eq!(
            agarra(&sim, h, Some(ombro)).map(|x| x.part),
            Some(esperado),
            "a alça saiu do alcance do dedo"
        );
    }
}

/// ⭐⭐⭐ **O DEDO E O DESENHO PERGUNTAM PELO MESMO OSSO** — e eram duas perguntas diferentes.
///
/// ⛔⛔ **O report do dono** (2026-09-08): *«gizmo não mantém ângulo fixo em relação ao osso»*. O
/// mecanismo: o dedo lê o osso da selecção **INTEIRA**
/// ([`crate::bone_gesture::selected_bone`], cujo doc explica porquê — prender uma forma a um
/// esqueleto entre vários faz-se escolhendo os dois, e aí **o primário é a forma**), e o desenho
/// lia só o **primário** do gizmo. Com uma forma seleccionada ao lado do osso, o canvas pintava o
/// arco noutro sítio — ou em sítio nenhum — enquanto o dedo operava no osso certo.
///
/// ⚠️ **Um doc afirmava que as duas eram a mesma pergunta** (*«o foco é a SELECÇÃO, e é a mesma
/// pergunta que o dedo faz»*, no `draw_influence`), e não eram. *Uma afirmação de igualdade sem um
/// gate é um comentário.*
///
/// ⇒ este gate mede a PORTA, sobre a selecção que produz a divergência: a forma primeiro, o osso
/// depois.
#[test]
fn the_finger_and_the_drawing_ask_for_the_same_bone() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.4, 0.4);
    // Uma FORMA (um objecto sem `Bone`) que o artista escolheu primeiro — é o gesto do *Bind*.
    let forma = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::Name::new("Shape"), ph2d_ecs::RootOrder(0)))
        .id();
    let selecao = [forma.to_bits(), ombro.to_bits()];

    // A porta que o DEDO usa.
    let do_dedo = crate::bone_gesture::selected_bone(&sim, selecao);
    assert_eq!(
        do_dedo,
        Some(ombro.to_bits()),
        "o dedo tem de achar o osso mesmo quando o primário é a forma"
    );

    // ⚠️ E o PRIMÁRIO, que era o que o desenho lia, é a forma — a divergência que este gate fixa.
    let primario = selecao[0];
    assert_ne!(
        primario,
        ombro.to_bits(),
        "a fixtura tem de PRODUZIR a divergência (o primário não pode ser o osso), senão é vácua"
    );
    assert!(
        crate::bone_limit::arc(&sim, Entity::from_bits(primario), PX_PERTO).is_none(),
        "o primário não tem arco — era isto que o canvas desenhava"
    );
    // ⭐ Pela porta certa, há arco.
    assert!(
        crate::bone_limit::arc(
            &sim,
            Entity::from_bits(do_dedo.expect("o dedo achou")),
            PX_PERTO,
        )
        .is_some(),
        "pela porta do dedo, o arco existe"
    );
}


/// ⭐⭐⭐ **E O CANVAS TEM DE CHAMAR ESSA PORTA** — o gate acima mede a porta, este mede o CHAMADOR.
///
/// ⚠️ Sem ele a cura de 2026-09-08 seria reversível em silêncio: alguém volta a escrever
/// `hero.gizmo.selection` no laço de desenho, o gate de cima continua verde (a porta não mudou) e o
/// arco volta a ser pintado noutro osso. *Um gate sobre a porta não cobre quem a ignora.*
#[test]
fn the_bone_overlays_are_drawn_for_the_bone_the_finger_uses() {
    let src = include_str!("render_loop/mod.rs");
    for verbo in ["draw_influence(", "draw_limit("] {
        let i = src
            .find(verbo)
            .unwrap_or_else(|| panic!("{verbo} sumiu do laço de desenho"));
        let janela = &src[i..(i + 200).min(src.len())];
        assert!(
            janela.contains("osso_focado"),
            "{verbo} não recebe o osso da porta do dedo — os 200 chars seguintes são:\n{janela}"
        );
    }
    assert!(
        src.contains("crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected())"),
        "o `osso_focado` deixou de sair do `selected_bone` — a divergência pode ter voltado"
    );
}

/// ⭐⭐ **A PAREDE NO ÂNGULO QUE O OSSO TEM CAI NA PONTA DELE** — a identidade que liga o arco ao
/// osso, e a régua que ilibou a geometria quando o report de 2026-09-08 chegou.
///
/// ⚠️ Ela corre sobre a cadeia montada pela porta REAL (`vec_bone_smoke::cadeia`), e não por uma
/// fixtura à mão: as duas montam `Transform`s diferentes, e foi por medir a errada que três
/// hipóteses minhas saíram ilibadas antes de a divergência aparecer noutro sítio.
#[test]
fn the_wall_at_the_bones_own_angle_lands_on_its_tip() {
    use crate::vec_bone_smoke::{ARM_A, ARM_B};
    let mut sim = SimWorld::default();
    let raiz = crate::vec_bone_smoke::cadeia(&mut sim, ARM_A, ARM_B, 6).expect("cadeia");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut ossos = vec![raiz];
    while let Some(f) = sim
        .world()
        .get::<ph2d_ecs::Children>(*ossos.last().expect("há raiz"))
        .and_then(|c| c.iter().find(|c| sim.world().get::<Bone>(**c).is_some()).copied())
    {
        ossos.push(f);
    }
    let alvo = ossos[2];
    for r in [0.0_f32, 0.4, -0.7, 1.9] {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(alvo) {
            t.rotation = r;
        }
        limita(&mut sim, alvo, f64::from(r) - 0.5, f64::from(r));
        let seg = crate::skeleton_live::bone_segments(&sim);
        let (_, a, b) = seg
            .iter()
            .copied()
            .find(|(x, _, _)| *x == alvo.to_bits())
            .expect("o osso tem segmento");
        let arc = crate::bone_limit::arc(&sim, alvo, PX_PERTO).expect("o arco existe");
        // ⚠️ A barra é o ruído do `f32` da pose da casa, não um número escolhido: a rotação viaja
        // em `f32` e a geometria é `f64`.
        let barra = 20.0 * f64::from(f32::EPSILON) * (b[0].abs() + b[1].abs()).max(1.0);
        let d_apex = (arc.apex[0] - a[0]).hypot(arc.apex[1] - a[1]);
        let d_edge = (arc.edge_max[0] - b[0]).hypot(arc.edge_max[1] - b[1]);
        assert!(
            d_apex < barra,
            "com rot={r} o vértice do arco caiu a {d_apex} da origem do osso (barra {barra})"
        );
        assert!(
            d_edge < barra,
            "com rot={r} a parede caiu a {d_edge} da ponta do osso (barra {barra})"
        );
    }
}
