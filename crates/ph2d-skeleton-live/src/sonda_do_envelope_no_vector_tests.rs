//! ⏱️ **O ENVELOPE MANDA NUMA FORMA VECTORIAL?** — a pergunta que a cena dedicada obriga a medir.
//!
//! ⛔⛔ **A minha resposta ao dono em 2026-09-18 foi *«nas formas vectoriais ele manda como
//! sempre»*, e ela nunca foi medida do lado do VECTOR.** O [`crate::skin_live::bind`] chama
//! `ph2d_vec_skin::pesos::pesos_do_caminho`, que é o **padrão-ouro** — a mesma lei que torna o
//! envelope inerte numa imagem. Ou seja: a premissa de que uma forma vectorial fica na lei
//! euclidiana pode ser falsa, e é ela que decide o que a cena dedicada tem de mostrar.
//!
//! ⚠️ A sonda mede **geometria deformada** pela porta do produto (`bind` + `recook`), nunca pesos:
//! *o censo dos knobs já media pesos e foi exactamente por isso que o report do dono o apanhou.*

use crate::test_support::{pior_desvio, quadro};
use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{ShapeKind, VecScene, cook};

/// Uma forma do `kind` pedido com uma corrente de TRÊS ossos ao longo dela, com o do MEIO a levar
/// `forca`. Devolve o pior desvio entre a pose de repouso e a pose dobrada.
///
/// ⚠️ **Três ossos e não dois:** a sonda irmã mediu que com UM osso os pesos renormalizam para `1`
/// e o alcance é inerte por construção. O alcance só decide quando dois ossos disputam o mesmo
/// ponto.
fn excursao(kind: ShapeKind, forca: f64, graus: f32) -> f64 {
    excursao_com(kind, forca, graus, ph2d_skeleton_ecs::SkinLaw::Auto)
}

/// A mesma corrida, com a LEI escolhida pelo artista.
fn excursao_com(kind: ShapeKind, forca: f64, graus: f32, lei: ph2d_skeleton_ecs::SkinLaw) -> f64 {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(kind, [0.0, 0.0], [60.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);

    let mut pai = None;
    let mut ossos = Vec::new();
    for k in 0..3 {
        let x = if k == 0 { 0.0 } else { 20.0 };
        let e = sim
            .world_mut()
            .spawn((
                Transform {
                    translation: ph2d_core::Vec2::new(x, if k == 0 { 5.0 } else { 0.0 }),
                    ..Transform::IDENTITY
                },
                Name::new(format!("Bone {k}")),
                RootOrder(0),
                Bone {
                    length: 20.0,
                    // ⭐ SÓ o do meio muda: é o único jeito de a diferença medida ser do ENVELOPE
                    // e não de um esqueleto inteiro noutra escala.
                    strength: if k == 1 { forca } else { 1.0 },
                    ..Default::default()
                },
            ))
            .id();
        if let Some(p) = pai {
            sim.world_mut().entity_mut(e).insert(ChildOf(p));
        }
        pai = Some(e);
        ossos.push(e);
    }

    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    // ⭐ A ESCOLHA do artista, escrita depois de prender — que é exactamente como o painel a faz:
    // a tabela do padrão-ouro fica guardada, e a lei diz se o quadro a lê.
    if let Some(e) = map.get(&id).and_then(|b| Entity::try_from_bits(*b))
        && let Some(mut skin) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::SkinBind>(e)
    {
        skin.law = lei;
    }
    let repouso = quadro(&sim, &mut scene, id);
    for osso in ossos.iter().skip(1) {
        sim.world_mut()
            .get_mut::<Transform>(*osso)
            .expect("Transform")
            .rotation = graus.to_radians();
    }
    let dobrado = quadro(&sim, &mut scene, id);
    pior_desvio(&repouso, &dobrado)
}

/// ⭐⭐⭐ **A TABELA QUE O DOC DA LEI CITA — e ela não pode envelhecer sozinha.**
///
/// ⛔⛔⛔ **É a medição que refutou a minha resposta ao dono.** Eu disse-lhe *«nas formas vectoriais
/// o envelope manda como sempre»*, e o doc de `o_envelope_deste_osso_manda` repetia-o com o
/// argumento de que *«o padrão-ouro precisa de uma malha do domínio, e uma Bézier não tem uma»*.
/// **Essa premissa expirou em 2026-09-15**, quando o `ph2d_vec_skin::pesos::pesos_do_caminho`
/// passou a construir a malha do INTERIOR de um contorno fechado.
///
/// ⚠️ **A partição é FECHADA contra ABERTA, nunca imagem contra forma** — e é a partição que a lei
/// passou a ler.
#[test]
fn so_um_caminho_aberto_ainda_sente_o_envelope() {
    // ⚠️ A faixa vai de `0,1` a `8,0`: **80×**. Uma varredura estreita leria «inerte» sobre uma lei
    // que só acorda quando o alcance chega a cobrir o osso vizinho.
    let amplitude = |kind: ShapeKind| -> f64 {
        let col: Vec<f64> = [0.1_f64, 0.3, 1.0, 2.0, 4.0, 8.0]
            .into_iter()
            .map(|f| excursao(kind, f, 40.0))
            .collect();
        let (lo, hi) = col
            .iter()
            .fold((f64::MAX, f64::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        println!("{kind:?}: {col:?} | amplitude {:.6}", hi - lo);
        hi - lo
    };

    for kind in [
        ShapeKind::Rectangle,
        ShapeKind::Ellipse,
        ShapeKind::Star,
        ShapeKind::Polygon,
        // ⭐ O `Segment` é o ARCO fechado pela corda — a peça de CONTROLO da cena `=2`: ela tem de
        // ser a mesma curva das cordas e mesmo assim ficar INERTE ao alcance.
        ShapeKind::Segment,
        ShapeKind::Pie,
    ] {
        let d = amplitude(kind);
        assert!(
            d < 1e-9,
            "{kind:?} e' FECHADA e o envelope moveu-a {d:.6} — entao o padrao-ouro deixou de \
             resolver aqui, e a lei que esconde o alcance nela passou a apagar um controlo VIVO"
        );
    }
    // ⭐ O CONTROLO POSITIVO: sem ele o gate acima ficava verde sobre um produto em que o envelope
    // é inerte em TODA parte — e aí a cura certa seria tirar o controlo do painel, não escondê-lo.
    for (kind, minimo) in [
        (ShapeKind::Line, 1.0),
        (ShapeKind::Arc, 2.0),
        (ShapeKind::Spiral, 1.0),
    ] {
        let d = amplitude(kind);
        assert!(
            d > minimo,
            "{kind:?} e' ABERTA e o envelope so' a moveu {d:.6} (barra {minimo}) — se nem aqui ele \
             manda, ele nao manda em lado nenhum"
        );
    }
}

/// ⭐⭐⭐ **A ESCOLHA DO DONO: uma forma FECHADA corre na lei do envelope quando o artista o pede**
/// (ordem de 2026-09-19: *«construa. por desenho»*).
///
/// ⛔⛔ **Este gate é a razão de a wave existir.** Sem ele, *«o botão está lá»* e *«o botão faz
/// alguma coisa»* leem-se igual — e esta família já pagou isso três vezes num mês (o chip morto sob
/// o dedo, o `Density` sem efeito, os dois botões de deformação).
///
/// ⚠️ **As TRÊS metades são três defeitos:** sem a 1.ª a escolha é um controlo morto; sem a 2.ª ela
/// não tem volta (e uma escolha sem volta é uma armadilha); sem a 3.ª nada prova que a tabela
/// guardada **sobreviveu** — e se ela fosse apagada, voltar ao `Auto` custaria dezenas de
/// milissegundos a re-resolver, que é o engasgo que este desenho existe para não ter.
#[test]
fn a_escolha_do_artista_poe_uma_forma_fechada_na_lei_do_envelope() {
    use ph2d_skeleton_ecs::SkinLaw;

    for kind in [ShapeKind::Rectangle, ShapeKind::Segment, ShapeKind::Ellipse] {
        // (1) No `Auto` — o nascimento — o alcance é INERTE, como sempre foi.
        let auto = (excursao_com(kind, 1.0, 40.0, SkinLaw::Auto)
            - excursao_com(kind, 4.0, 40.0, SkinLaw::Auto))
        .abs();
        assert!(
            auto < 1e-9,
            "{kind:?} no Auto: o alcance moveu {auto:.6} — o padrao-ouro deixou de resolver aqui"
        );

        // (2) Escolhido *por alcance*, a MESMA forma passa a sentir o envelope.
        let a = excursao_com(kind, 1.0, 40.0, SkinLaw::Envelope);
        let b = excursao_com(kind, 4.0, 40.0, SkinLaw::Envelope);
        let d = (a - b).abs();
        println!("{kind:?} por alcance: {a:.4} vs {b:.4} | d {d:.4}");
        assert!(
            d > 1.0,
            "{kind:?} escolhida «por alcance» NAO sentiu o alcance (d {d:.6}): o botao existe e nao \
             faz nada, que e' o controlo morto que esta familia ja' pagou tres vezes"
        );

        // (3) E a VOLTA é exacta: o `Auto` devolve o padrão-ouro ao bit, porque a tabela guardada
        // nunca foi tocada.
        let volta = excursao_com(kind, 4.0, 40.0, SkinLaw::Auto);
        let nunca_mexeu = excursao_com(kind, 1.0, 40.0, SkinLaw::Auto);
        assert!(
            (volta - nunca_mexeu).abs() < 1e-12,
            "{kind:?}: voltar ao Auto nao devolveu a MESMA deformacao ({volta:.9} contra \
             {nunca_mexeu:.9}) — a tabela guardada foi tocada, e voltar passa a custar uma \
             re-resolucao"
        );
    }
}

/// ⭐⭐⭐ **E A MANCHA SEGUE A ESCOLHA** — senão o artista ganha o efeito e perde o controlo dele.
///
/// ⛔ *Uma forma preenchida escolhida «por alcance» que deformasse pelo envelope SEM mancha e SEM
/// alça seria pior do que não ter a escolha:* o alcance passaria a mandar num número que o artista
/// não consegue ver nem agarrar.
#[test]
fn a_mancha_segue_a_escolha_e_nao_a_forma() {
    use ph2d_skeleton_ecs::{SkinBind, SkinLaw};

    let mundo_com = |lei: SkinLaw| -> bool {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
        // Uma forma FECHADA: no `Auto` o envelope é inerte e a mancha não existe.
        let id = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [60.0, 10.0], &[]));
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let mut pai = None;
        let mut raiz = None;
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
            raiz.get_or_insert(e);
            pai = Some(e);
        }
        crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
        let forma = map
            .get(&id)
            .and_then(|b| Entity::try_from_bits(*b))
            .expect("a forma");
        sim.world_mut()
            .get_mut::<SkinBind>(forma)
            .expect("pele")
            .law = lei;
        crate::esqueletos::o_envelope_deste_osso_manda(&sim, raiz.expect("raiz"))
    };

    assert!(
        !mundo_com(SkinLaw::Auto),
        "uma forma FECHADA no Auto mostrou a mancha: ali o alcance nao entra na conta"
    );
    assert!(
        mundo_com(SkinLaw::Envelope),
        "a MESMA forma escolhida «por alcance» NAO mostrou a mancha: o artista ganhou o efeito e \
         perdeu o controlo dele — pior do que nao ter a escolha"
    );
}
