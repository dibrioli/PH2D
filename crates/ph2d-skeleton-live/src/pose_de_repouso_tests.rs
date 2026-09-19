//! Os gates da POSE DE REPOUSO — o nascimento, a volta, a sub-árvore e as três respostas.

use super::*;
use crate::esqueletos_tests_support::cadeias;
use ph2d_core::Vec2;

/// Põe um osso numa pose bem longe do repouso dele.
fn posa(sim: &mut SimWorld, e: Entity, dx: f32, graus: f32) {
    let mut t = sim.world_mut().get_mut::<Transform>(e).expect("Transform");
    t.translation.x += dx;
    t.rotation += graus.to_radians();
}

fn pose(sim: &SimWorld, e: Entity) -> Transform {
    sim.world().get::<Transform>(e).copied().expect("Transform")
}

/// A corrente `[raiz, meio, ponta]`, descida pela **árvore**.
///
/// ⛔⛔ **E não pela ordem que a [`crate::esqueletos::ossos_desde`] devolve, que a 1.ª redacção
/// destes gates presumiu ser a hierárquica e NÃO é:** ela ordena por `to_bits`, que é um id de
/// alocação — aqui ele saiu ao contrário da criação, e o gate da sub-árvore reprovou a acusar a
/// LEI de mexer no pai quando quem estava trocado era a fixtura. *Uma porta que promete um
/// CONJUNTO determinístico não promete uma ordem com sentido, e um teste que lhe pede a segunda
/// mede outra coisa.*
fn corrente(sim: &SimWorld, raiz: Entity) -> [Entity; 3] {
    let filho = |e: Entity| {
        sim.world()
            .get::<ph2d_ecs::Children>(e)
            .and_then(|f| f.iter().next().copied())
            .expect("a cadeia da fixtura tem tres ossos")
    };
    let meio = filho(raiz);
    [raiz, meio, filho(meio)]
}

/// ⭐⭐⭐ **UM OSSO NASCE COM O REPOUSO DELE** — sem isto, todo rig já autorado ficaria sem destino
/// para onde voltar, e o verbo recusaria sobre a cena inteira.
#[test]
fn um_osso_nasce_com_a_pose_em_que_nasceu_como_repouso() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let ossos = crate::esqueletos::ossos_desde(&sim, raizes[0]);
    assert_eq!(ossos.len(), 3, "a fixtura e' uma cadeia de tres");
    for e in ossos {
        let t = pose(&sim, e);
        let repouso = sim
            .world()
            .get::<BoneRest>(e)
            .copied()
            .unwrap_or_else(|| panic!("o osso {e:?} nasceu SEM repouso guardado"));
        assert_eq!(
            repouso,
            BoneRest::de(&t),
            "o repouso de nascimento nao e' a pose de nascimento"
        );
    }
}

/// ⭐⭐⭐ **REPOR DEVOLVE A CORRENTE INTEIRA, AO BIT** — e é isso que faz «experimentar uma pose»
/// deixar de ser um caminho sem volta.
#[test]
fn repor_devolve_a_corrente_inteira_ao_repouso() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let ossos = crate::esqueletos::ossos_desde(&sim, raizes[0]);
    let antes: Vec<Transform> = ossos.iter().map(|e| pose(&sim, *e)).collect();
    for (k, e) in ossos.iter().enumerate() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "k vai a 3: a fixtura so' precisa de tres poses diferentes"
        )]
        posa(&mut sim, *e, 3.0 + k as f32, 25.0);
    }
    for (k, e) in ossos.iter().enumerate() {
        assert_ne!(
            pose(&sim, *e),
            antes[k],
            "a fixtura nao mexeu no osso {k}: sem o fenomeno, este gate nao afirma nada"
        );
    }
    let n = repor(&mut sim, raizes[0]);
    assert_eq!(n, 3, "os tres ossos da corrente tinham de voltar");
    for (k, e) in ossos.iter().enumerate() {
        assert_eq!(
            pose(&sim, *e),
            antes[k],
            "o osso {k} nao voltou ao repouso ao bit"
        );
    }
}

/// ⚠️ **O sujeito é a SUB-ÁRVORE, e o pai NÃO se mexe** — é isso que torna *«repor só este braço»*
/// exprimível. ⛔ Uma lei que subisse à raiz sozinha apagaria este gesto sem deixar alternativa.
#[test]
fn repor_desde_o_meio_leva_o_filho_e_deixa_o_pai_onde_esta() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let [raiz, meio, ponta] = corrente(&sim, raizes[0]);
    posa(&mut sim, raiz, 7.0, 30.0);
    posa(&mut sim, meio, 5.0, 40.0);
    posa(&mut sim, ponta, 4.0, 50.0);
    let raiz_posada = pose(&sim, raiz);
    let n = repor(&mut sim, meio);
    assert_eq!(n, 2, "o do meio e o filho dele — e mais ninguem");
    assert_eq!(
        pose(&sim, raiz),
        raiz_posada,
        "repor a partir do meio mexeu no PAI: o gesto deixou de ser sobre um braco"
    );
    assert_eq!(
        sim.world().get::<BoneRest>(meio).copied().map(|r| {
            let mut t = Transform::IDENTITY;
            r.aplica(&mut t);
            t
        }),
        Some(pose(&sim, meio)),
        "o osso do meio nao voltou ao repouso"
    );
}

/// ⭐⭐⭐ **O DEFEITO, NA FORMA MÍNIMA: um osso reposto NÃO vai para a identidade.**
///
/// ⚠️ **As duas metades são dois defeitos diferentes:** ir para a identidade é o que o app fazia
/// (e é a causa das 20 unidades medidas na sonda irmã); não ir a lado nenhum seria um botão morto.
#[test]
fn repor_um_osso_nao_o_manda_para_a_identidade() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let meio = corrente(&sim, raizes[0])[1];
    let repouso = pose(&sim, meio);
    assert_ne!(
        repouso,
        Transform::IDENTITY,
        "a fixtura tem de ter o osso FORA da identidade, senao as duas respostas leem-se iguais"
    );
    posa(&mut sim, meio, 9.0, 60.0);
    assert_eq!(
        repor_transformacao(&mut sim, meio),
        Reposicao::Reposta { ossos: 2 }
    );
    assert_eq!(pose(&sim, meio), repouso, "nao voltou ao repouso");
    assert_ne!(
        pose(&sim, meio),
        Transform::IDENTITY,
        "foi para a ORIGEM: e' o defeito que esta wave cura, de volta"
    );
}

/// ⚠️ **O CONTROLO:** uma coisa que não é osso não é desta lei, e a Hierarquia continua a mandá-la
/// para a identidade como sempre. ⛔ Sem esta metade, a cura teria mudado o gesto para o app todo.
#[test]
fn uma_coisa_que_nao_e_osso_nao_e_desta_lei() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn(Transform {
            translation: Vec2::new(4.0, 4.0),
            ..Transform::IDENTITY
        })
        .id();
    assert_eq!(repor_transformacao(&mut sim, e), Reposicao::NaoEOsso);
    assert_eq!(
        pose(&sim, e).translation,
        Vec2::new(4.0, 4.0),
        "esta porta mexeu numa coisa que nao e' osso"
    );
}

/// ⛔⛔ **Um osso SEM repouso guardado não cai na identidade — ele RECUSA, e a pose fica.**
///
/// É a terceira resposta, e a razão de ela existir: sem ela o defeito voltaria por esta porta para
/// todo osso que chegasse de um ficheiro anterior a esta wave.
#[test]
fn um_osso_sem_repouso_guardado_recusa_e_nao_se_mexe() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let ossos = crate::esqueletos::ossos_desde(&sim, raizes[0]);
    for e in &ossos {
        sim.world_mut().entity_mut(*e).remove::<BoneRest>();
    }
    let posada = pose(&sim, ossos[0]);
    assert_eq!(
        repor_transformacao(&mut sim, ossos[0]),
        Reposicao::SemRepouso
    );
    assert_eq!(
        pose(&sim, ossos[0]),
        posada,
        "recusou e mexeu na mesma — a recusa tem de ser inerte"
    );
}

/// **Guardar sobrescreve**, e é a frase inteira do gesto: *«a partir de agora, ESTA é a pose de
/// repouso»*.
#[test]
fn guardar_faz_da_pose_de_agora_o_destino_do_voltar() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let meio = corrente(&sim, raizes[0])[1];
    posa(&mut sim, meio, 6.0, 45.0);
    let nova = pose(&sim, meio);
    let n = guardar(&mut sim, meio);
    assert_eq!(n, 2, "o do meio e o filho dele");
    posa(&mut sim, meio, -20.0, -80.0);
    repor(&mut sim, meio);
    assert_eq!(
        pose(&sim, meio),
        nova,
        "o voltar foi parar ao repouso ANTIGO: o guardar nao sobrescreveu"
    );
}
