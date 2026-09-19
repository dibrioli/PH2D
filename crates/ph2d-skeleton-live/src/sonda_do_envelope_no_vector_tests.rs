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
use ph2d_ecs::{ChildOf, Name, RootOrder, SimWorld, Transform};
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
