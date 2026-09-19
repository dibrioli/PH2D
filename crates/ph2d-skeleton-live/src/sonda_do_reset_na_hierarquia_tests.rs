//! ⏱️ **QUANTO É QUE O *Reset Transform* MOVIA A ARTE?** — o defeito do dono, medido no caminho do
//! produto, e a cura medida ao lado dele.
//!
//! ⛔⛔⛔ **O defeito:** a tabela do menu de contexto da Hierarquia é **plana** — ela não sabe o que
//! a linha é —, e sobre um osso o verbo escrevia `Transform::IDENTITY`. Num osso, a **direcção**
//! mora na `rotation` e a **posição** na `translation`, logo aquilo não é *«repor»*: é mandar o
//! osso para a origem, virado para leste, com uma mensagem **verde** a dizer que correu bem.
//! Medido aqui: **26,48 unidades num desenho de 60** — quase metade dele.
//!
//! ⚠️ **A régua é a ARTE e não o osso**, e é a escolha que faz o número significar alguma coisa: o
//! que o artista vê é o desenho a saltar, e um desvio medido no `Transform` do osso não diz quanto
//! disso chega à tinta. A unidade é a do documento, sobre uma forma de **60 × 10**.
//!
//! ⚠️ **As duas leis correm sobre o MESMO palco**, montado duas vezes do zero — comparar dois
//! números tirados de mundos diferentes seria comparar duas fixturas.

use crate::test_support::{pior_desvio, quadro};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{ShapeKind, VecScene, cook};

/// O comprimento da forma, em unidades do documento — o denominador do report.
const LARGURA_DA_FORMA: f64 = 60.0;

/// Quanto a arte se afasta do repouso depois de o artista dobrar a corrente e carregar em *Reset
/// Transform* no osso do meio, com a lei que `lei_antiga` escolhe.
///
/// `true` ⇒ a lei que o app tinha (a identidade). `false` ⇒ a desta wave (o repouso).
fn salto_da_arte(lei_antiga: bool) -> f64 {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(
        ShapeKind::Rectangle,
        [0.0, 0.0],
        [LARGURA_DA_FORMA, 10.0],
        &[],
    ));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);

    // ⭐ Os ossos nascem pela PORTA DO PRODUTO (`bone::create`), que é também quem lhes dá o
    // repouso — montá-los à mão aqui mediria uma fixtura abaixo da lei que se está a medir.
    let mut ossos = Vec::new();
    let mut pai = None;
    for k in 0..3 {
        let x = 20.0 * f64::from(k);
        let bits =
            crate::bone::create(&mut sim, pai, [x, 5.0], [x + 20.0, 5.0]).expect("o osso nasce");
        let e = Entity::from_bits(bits);
        pai = Some(e);
        ossos.push(e);
    }
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);

    let repouso = quadro(&sim, &mut scene, id);
    // O artista experimenta uma pose: dobra a corrente a partir do osso do meio.
    for osso in ossos.iter().skip(1) {
        sim.world_mut()
            .get_mut::<Transform>(*osso)
            .expect("Transform")
            .rotation = 35.0_f32.to_radians();
    }
    // E carrega em *Reset Transform* no osso do meio.
    if lei_antiga {
        *sim.world_mut()
            .get_mut::<Transform>(ossos[1])
            .expect("Transform") = Transform::IDENTITY;
    } else {
        assert_eq!(
            crate::pose_de_repouso::repor_transformacao(&mut sim, ossos[1]),
            crate::pose_de_repouso::Reposicao::Reposta { ossos: 2 },
            "a porta nao reconheceu o osso: a sonda mediria outra coisa"
        );
    }
    let depois = quadro(&sim, &mut scene, id);
    pior_desvio(&repouso, &depois)
}

/// ⭐⭐⭐ **A TABELA DO REPORT — e ela não pode envelhecer sozinha.**
///
/// | lei | quanto a arte salta | em fracção da forma |
/// |---|---|---|
/// | a que o app tinha (identidade) | **26,484841 unidades** | **44 %** |
/// | a desta wave (o repouso) | **0,000000** | nada |
///
/// ⚠️ **A metade de baixo é o que prova a cura; a de cima é o que prova que havia o que curar.**
/// Sem a de cima, um `assert` de `0` passaria também numa sonda que não estivesse a dobrar nada —
/// *uma régua que não vê o fenómeno acontecer não prova que ele não aconteceu*.
#[test]
fn o_reset_de_um_osso_movia_um_terco_do_desenho_e_agora_nao_move_nada() {
    let antigo = salto_da_arte(true);
    let novo = salto_da_arte(false);
    // ⚠️ A sonda IMPRIME o que mediu: a tabela do doc acima é derivada desta linha, e não de uma
    // memória de quem a escreveu.
    eprintln!(
        "[sonda] reset num osso: identidade {antigo:.6} · repouso {novo:.6} (forma de \
         {LARGURA_DA_FORMA:.0} unidades)"
    );
    assert!(
        antigo > LARGURA_DA_FORMA / 4.0,
        "a lei antiga saltou {antigo:.6} — abaixo do defeito reportado; a fixtura deixou de conter \
         o fenomeno, e este gate passaria a nao afirmar nada"
    );
    assert!(
        novo < 1e-9,
        "voltar ao repouso tinha de devolver a arte EXACTAMENTE onde ela estava, e moveu {novo:.9}"
    );
}
