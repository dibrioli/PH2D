//! Os gates da topologia da árvore de ossos.

use super::*;
use ph2d_ecs::Transform;

use crate::esqueletos_tests_support::cadeias;

/// ⭐ **Contar RAÍZES é o que distingue «uma cadeia de três» de «três esqueletos».**
///
/// ⚠️ **As duas metades são dois defeitos:** contar ossos leria `3` numa cadeia só (e o botão
/// recusaria sempre), e contar só a primeira raiz leria `1` com três esqueletos (e ele nunca
/// recusaria).
#[test]
fn as_raizes_contam_esqueletos_e_nao_ossos() {
    for n in 1..=3_usize {
        let mut sim = SimWorld::default();
        let raizes = cadeias(&mut sim, n);
        let lidas = bone_roots(&sim);
        assert_eq!(
            lidas.len(),
            n,
            "com {n} cadeia(s) de TRES ossos a porta leu {} esqueleto(s) — ela esta' a contar \
             ossos, e o botao passaria a recusar sobre uma cena com um esqueleto so'",
            lidas.len()
        );
        for r in raizes {
            assert!(
                lidas.contains(&r),
                "a raiz {r:?} nao esta' na lista: a porta subiu para o sitio errado"
            );
        }
    }
}

/// ⚠️ **Um esqueleto pendurado dentro de um GRUPO continua a ser UM esqueleto** — a subida pára no
/// primeiro pai que não é osso, e é isso que permite arrumar um personagem numa pasta.
#[test]
fn um_esqueleto_dentro_de_um_grupo_continua_a_ser_um() {
    let mut sim = SimWorld::default();
    let raizes = cadeias(&mut sim, 1);
    let grupo = sim.world_mut().spawn(Transform::IDENTITY).id();
    sim.world_mut()
        .entity_mut(raizes[0])
        .insert(ph2d_ecs::ChildOf(grupo));
    let lidas = bone_roots(&sim);
    assert_eq!(
        lidas, raizes,
        "pendurar o esqueleto num grupo mudou a raiz dele: a subida nao parou no primeiro pai \
         que nao e' osso, e o botao passaria a ver esqueletos onde ha' um so'"
    );
}

/// ⭐⭐⭐ **UM PALCO COM AS TRÊS PELES, presas pela PORTA DO PRODUTO** — a fixtura das leis abaixo.
///
/// Devolve `(sim, osso_da_forma_fechada, osso_do_caminho_aberto, osso_da_imagem)`, cada uma presa a
/// um esqueleto de **três** ossos próprio (o alcance só decide quando dois ossos disputam o mesmo
/// ponto, e com um só ele é inerte por construção — medido na [`crate::sonda_do_envelope_tests`]).
///
/// ⛔⛔ **Ela passa pelo `bind`/`bind_image` e NUNCA monta a `SkinBind` à mão.** A 1.ª redacção
/// destes gates escrevia `source: Vec::new()` e nomeava as peles pela MÍDIA — e por isso não podia
/// ver o defeito que a medição achou: *uma fixtura montada à mão fica abaixo da rotura que se está
/// a medir*, que é a lei que esta casa já pagou no controlador de topo.
fn palco_das_tres_peles() -> (SimWorld, Entity, Entity, Entity) {
    use ph2d_vec_scene::{ShapeKind, VecScene, cook};

    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let fechada = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [60.0, 10.0], &[]));
    let aberta = scene.push_path(cook(ShapeKind::Line, [0.0, 40.0], [60.0, 50.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);

    let mut corrente = |y: f64| -> Entity {
        let mut pai = None;
        let mut raiz = None;
        for k in 0..3 {
            let x = if k == 0 { 0.0 } else { 20.0 };
            let e = Entity::from_bits(
                crate::bone::create(
                    &mut sim,
                    pai,
                    [x, if k == 0 { y } else { 0.0 }],
                    [x + 20.0, if k == 0 { y } else { 0.0 }],
                )
                .expect("o osso nasce"),
            );
            raiz.get_or_insert(e);
            pai = Some(e);
        }
        raiz.expect("a raiz")
    };
    let (a, b, c) = (corrente(5.0), corrente(45.0), corrente(85.0));

    crate::skin_live::bind(&mut sim, &scene, &map, &[fechada], Some(a));
    crate::skin_live::bind(&mut sim, &scene, &map, &[aberta], Some(b));

    // A imagem: um quadrado opaco, preso ao terceiro esqueleto.
    let px = 32_u32;
    let arte = vec![255u8; (px * px * 4) as usize];
    let sprite = ph2d_render::Sprite {
        size: [60.0, 10.0],
        ..ph2d_render::Sprite::atlas(0, [60.0, 10.0], [1.0; 4])
    };
    let img = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(30.0, 85.0),
                ..Transform::IDENTITY
            },
            sprite,
        ))
        .id();
    assert!(
        crate::skin_live::bind_image(
            &mut sim,
            img,
            &arte,
            [px, px],
            1.0,
            ph2d_poly2d::GridOptions::default(),
            Some(c),
        ),
        "a imagem nao prendeu — a fixtura deixa de conter o fenomeno"
    );
    (sim, a, b, c)
}

/// ⭐⭐⭐ **O ENVELOPE MANDA ONDE O PADRÃO-OURO NÃO RESOLVEU — e a MÍDIA nunca foi a pergunta.**
///
/// ⛔⛔⛔ **As DUAS redacções anteriores estão mortas, e a segunda ficou viva doze horas.** A 1.ª
/// perguntava à CENA (*«há aqui alguma forma vectorial?»*) e acendia o envelope em todos os ossos de
/// uma cena mista; a 2.ª passou a perguntar por OSSO e continuou a perguntar pela **mídia**, com o
/// argumento escrito de que *«uma Bézier não tem malha do domínio»*. ⚠️ **Essa premissa expirou em
/// 2026-09-15**, quando o `pesos_do_caminho` passou a construir a malha do INTERIOR de um contorno
/// fechado — medido na [`crate::sonda_do_envelope_no_vector_tests`]: `Rectangle`, `Ellipse`, `Star`
/// e `Polygon` dão amplitude **`0,000000`** sobre uma faixa de `80×` no alcance.
///
/// ⇒ *o dono tinha mais razão do que a minha resposta lhe deu*, e a lei que fica pergunta ao BIND.
///
/// ⚠️ **As TRÊS metades são três defeitos diferentes**: sem a 1.ª o envelope volta a ser oferecido
/// sobre uma lei que não o lê; sem a 2.ª ele desaparece do único sítio onde ainda governa a arte; e
/// sem a 3.ª a imagem volta a ser tratada por uma regra própria, que é como as duas mídias
/// divergem.
#[test]
fn o_envelope_manda_onde_o_padrao_ouro_nao_resolveu() {
    let (sim, fechada, aberta, imagem) = palco_das_tres_peles();
    assert!(
        !crate::esqueletos::o_envelope_deste_osso_manda(&sim, fechada),
        "o osso de uma FORMA FECHADA manteve o envelope — ali os pesos sao os do padrao-ouro e o \
         alcance nao entra na conta (medido: amplitude 0,000000 numa faixa de 80x)"
    );
    assert!(
        crate::esqueletos::o_envelope_deste_osso_manda(&sim, aberta),
        "o osso de um CAMINHO ABERTO perdeu o envelope: ali nao ha' interior, logo nao ha' dominio \
         e nao ha' pesos guardados — a lei euclidiana manda, e esconde-lo tira um controlo VIVO"
    );
    assert!(
        !crate::esqueletos::o_envelope_deste_osso_manda(&sim, imagem),
        "o osso de uma IMAGEM que resolveu manteve o envelope: ele promete um efeito que a lei dos \
         pesos guardados nao tem"
    );
}

/// ⭐⭐⭐ **E A LEI É MEDIDA NO BARRO, não no predicado** — o controlo que impede esta família de
/// gates de se tornar um espelho da implementação.
///
/// ⚠️ *Uma régua que partilha a lei do produto não acusa*: as três asserções acima leem o mesmo
/// `caiu_na_lei_derivada` que o produto lê. Esta mede a **geometria deformada** pelas duas posições
/// do alcance, que é o que o dono vê.
#[test]
fn o_predicado_concorda_com_o_que_a_deformacao_faz() {
    use crate::test_support::{pior_desvio, quadro};
    use ph2d_vec_scene::{ShapeKind, VecScene, cook};

    let excursao = |kind: ShapeKind, forca: f64| -> f64 {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
        let id = scene.push_path(cook(kind, [0.0, 0.0], [60.0, 10.0], &[]));
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let mut pai = None;
        let mut ossos = Vec::new();
        for k in 0..3 {
            let x = if k == 0 { 0.0 } else { 20.0 };
            let e = Entity::from_bits(
                crate::bone::create(
                    &mut sim,
                    pai,
                    [x, if k == 0 { 5.0 } else { 0.0 }],
                    [x + 20.0, if k == 0 { 5.0 } else { 0.0 }],
                )
                .expect("o osso nasce"),
            );
            if k == 1 {
                sim.world_mut()
                    .get_mut::<ph2d_skeleton_ecs::Bone>(e)
                    .expect("Bone")
                    .strength = forca;
            }
            ossos.push(e);
            pai = Some(e);
        }
        crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
        let repouso = quadro(&sim, &mut scene, id);
        for osso in ossos.iter().skip(1) {
            sim.world_mut()
                .get_mut::<Transform>(*osso)
                .expect("Transform")
                .rotation = 40.0_f32.to_radians();
        }
        pior_desvio(&repouso, &quadro(&sim, &mut scene, id))
    };

    let fechada = (excursao(ShapeKind::Rectangle, 1.0) - excursao(ShapeKind::Rectangle, 8.0)).abs();
    let aberta = (excursao(ShapeKind::Line, 1.0) - excursao(ShapeKind::Line, 8.0)).abs();
    println!("fechada d {fechada:.6} | aberta d {aberta:.6}");
    assert!(
        fechada < 1e-9,
        "o alcance MOVEU uma forma fechada em {fechada:.6}: entao esconder o envelope nela apaga \
         um controlo VIVO, e a lei acima esta' errada"
    );
    assert!(
        aberta > 1.0,
        "o alcance nao move um caminho ABERTO (d {aberta:.6}): entao ele e' inerte em toda parte e \
         o controlo devia sair do painel, nao ser escondido por osso"
    );
}

/// ⭐⭐⭐ **A MANCHA E A ALÇA DO ENVELOPE PASSAM PELA MESMA PORTA** (report do dono, 2026-09-18:
/// *«o gizmo do envelope fica sempre visível mesmo quando não é usado?»* — sim, ficava).
///
/// ⚠️ **A lei entra na `influence_region` e não em quem desenha, porque essa porta tem DOIS
/// consumidores** — o desenho da mancha e o **hit-test da alça**. *Curar só o pintor deixaria o
/// artista a arrastar uma alça invisível, que é pior do que a mancha a mais.*
///
/// ⛔⛔ **A premissa deste gate morreu DUAS vezes** (a lei por cena, depois a lei por mídia) e ele
/// fica com as duas mortes visíveis no diff — a medir a PORTA, que é o que o desenho e o pick
/// chamam, em vez da lei, que já tem os gates dela acima.
#[test]
fn a_mancha_e_a_alca_passam_pela_mesma_porta() {
    let (sim, fechada, aberta, imagem) = palco_das_tres_peles();
    for (osso, o_que) in [(fechada, "uma FORMA FECHADA"), (imagem, "uma IMAGEM")] {
        assert!(
            crate::skin_live::influence_region(&sim, osso.to_bits()).is_none(),
            "a mancha foi desenhada num osso que so' {o_que} usa: ela diz «ate' onde este osso \
             alcanca» sobre uma lei que nao usa alcance nenhum, e a alca dela fica agarravel por cima"
        );
    }
    assert!(
        crate::skin_live::influence_region(&sim, aberta.to_bits()).is_some(),
        "a mancha sumiu do osso de um CAMINHO ABERTO: o artista perdeu o controlo do alcance \
         exactamente onde ele decide a deformacao"
    );
}
