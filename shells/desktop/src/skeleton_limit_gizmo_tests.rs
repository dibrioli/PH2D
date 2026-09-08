//! Os gates do **GIZMO do limite** — irmão do [`super`] pelo teto de 600 LOC do HR-18, com o corte
//! por RESPONSABILIDADE: ali mede-se a LEI (o que o limite apara, e que a corrente assenta), aqui o
//! que o DEDO apanha e onde as alças estão.
//!
//! ⚠️ **As duas perguntas falham de maneiras diferentes**, e os três reports do dono provaram-no: a
//! lei estava certa nas três vezes, e o que falhava era o gizmo — a alça em cima da ponta, o
//! desenho a perguntar por outro osso, e o alvo a `17 px` de onde a parede parece estar.

use super::*;

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

/// ⭐⭐⭐ **A PAREDE AGARRA-SE ONDE ELA PARECE ESTAR** — na borda do setor, não só no triângulo.
///
/// ⛔⛔ **Report do dono (2026-09-08): *«não consigo mover os gizmos dos ângulos»*.** Depois de a
/// alça sair para fora do alcance do osso (a cura do report anterior), o único alvo ficou a
/// `17 px` ALÉM da borda do setor — e é a borda que se lê como *«a parede»*. O artista mirava no
/// que via e não havia alvo nenhum ali. *Um alvo que não está onde a coisa PARECE estar é um alvo
/// ausente.*
///
/// ⇒ o alvo é o SEGMENTO inteiro, que é exactamente o traço desenhado.
#[test]
fn the_wall_can_be_grabbed_along_its_whole_length_not_only_at_the_handle() {
    let (mut sim, [ombro, _]) = braco();
    // Uma faixa larga, para a parede ficar longe do osso e o gate medir a PAREDE.
    limita(&mut sim, ombro, -1.4, 1.4);
    let arc = crate::bone_limit::arc(&sim, ombro, PX_PERTO).expect("o arco existe");
    // Três pontos AO LONGO da parede `max`: a meio, a três quartos, e na borda do setor.
    for t in [0.5_f64, 0.75, 1.0] {
        let p = [
            arc.apex[0] + (arc.handle_max[0] - arc.apex[0]) * t,
            arc.apex[1] + (arc.handle_max[1] - arc.apex[1]) * t,
        ];
        assert_eq!(
            agarra(&sim, p, Some(ombro)).map(|h| h.part),
            Some(ph2d_skeleton_render::BonePart::LimitMax),
            "a {}% do comprimento da parede o dedo não a apanhou",
            t * 100.0
        );
    }
}

/// ⛔ **E O OSSO CONTINUA A GANHAR ONDE ELE ESTÁ** — a cura de cima não pode devolver o defeito
/// anterior ao contrário.
///
/// ⚠️ As paredes **cruzam** o osso sempre que ele se aproxima de uma delas; sem a competição por
/// proximidade a parede roubaria o gesto de girar em toda a faixa, que é exactamente o report de
/// 2026-09-07 outra vez. *Ganha o que está mais perto do dedo.*
#[test]
fn the_bone_still_wins_where_the_bone_is() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -1.4, 1.4);
    let seg = crate::skeleton_live::bone_segments(&sim);
    let (_, a, b) = seg
        .iter()
        .copied()
        .find(|(x, _, _)| *x == ombro.to_bits())
        .expect("o osso tem segmento");
    // O meio do osso — o sítio onde se agarra para girar.
    let meio = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    assert_eq!(
        agarra(&sim, meio, Some(ombro)).map(|h| h.part),
        Some(ph2d_skeleton_render::BonePart::Body),
        "o meio do osso deixou de girar o osso"
    );
}
