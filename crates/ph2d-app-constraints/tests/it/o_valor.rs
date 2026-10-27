//! ⭐ **Valor segue valor** (`docs/BlenderConstraints/06_W5_resto.md` §3.1): um número do alvo, faixa
//! → faixa e com mola, num número do dono — pose ou campo.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::valor::valores;
use ph2d_app_constraints::{canais, quadro};
use ph2d_constraints::{Canal, Kind, Mola, Prop, Spring, Valor};
use ph2d_constraints_ecs::{Constraints, Estado, Estados, Lei, Simulador};
use ph2d_ecs::canal_constraint::{FORCA, chave};
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_preview_drive::{FieldDrive, PreviewDrive};
use ph2d_timeline::TimelineDoc;

/// O alvo `B` (id 1) na altura `y` e o dono `A` (id 2) com o `valor` a seguir B, e uma Spring
/// desligada atrás (o campo que um destino de campo conduz).
fn mundo(y: f32, valor: Valor) -> (SimWorld, Entity, u32, u32) {
    let mut sim = SimWorld::new();
    let mut b = Transform::IDENTITY;
    b.translation.y = y;
    sim.world_mut().spawn((StableId(1), b));
    let mut c = Constraints::default();
    let v = c.push(Kind::Valor(valor), Some(StableId(1)));
    let s = c.push(Kind::Spring(Spring::default()), None);
    let a = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    (sim, a, v, s)
}

fn sem_mola() -> Valor {
    Valor {
        mola: None,
        ..Valor::default()
    }
}

fn por_y(sim: &mut SimWorld, y: f32) {
    let w = sim.world_mut();
    let b = ph2d_ecs::entity_with_stable_id(w, StableId(1)).unwrap();
    w.get_mut::<Transform>(b).unwrap().translation.y = y;
}

fn quadro_em(sim: &mut SimWorld, drive: &mut PreviewDrive, r: Option<Relogio<'_>>) {
    valores(sim, r);
    quadro(sim, &mut ph2d_vec_scene::VecScene::new(), drive, None);
}

#[test]
fn a_value_follows_another_value_through_the_range() {
    let (mut sim, a, _, _) = mundo(0.5, sem_mola());
    let mut drive = PreviewDrive::default();
    quadro_em(&mut sim, &mut drive, None);
    let rot = sim.world().get::<Transform>(a).unwrap().rotation;
    assert!(
        (f64::from(rot) - std::f64::consts::FRAC_PI_4).abs() < 1e-6,
        "{rot}"
    );
    // Fora da faixa apara (sem extrapolar).
    por_y(&mut sim, 7.0);
    quadro_em(&mut sim, &mut drive, None);
    let rot = sim.world().get::<Transform>(a).unwrap().rotation;
    assert!(
        (f64::from(rot) - std::f64::consts::FRAC_PI_2).abs() < 1e-6,
        "{rot}"
    );
}

/// O destino é um CAMPO (a força da Spring do próprio dono): chega ao dreno da família no mesmo
/// quadro, e gravar grava o autorado — é pré-visualização, nunca documento.
#[test]
fn a_followed_field_is_a_preview_never_the_document() {
    let (mut sim, a, _, s) = mundo(0.25, sem_mola());
    let k = chave(FORCA, s);
    let campo = Prop::Campo([k.familia, k.campo, k.i, k.j]);
    let mut c = sim.world().get::<Constraints>(a).unwrap().clone();
    let Kind::Valor(v) = &mut c.items[0].kind else {
        unreachable!()
    };
    v.destino = campo;
    v.para_faixa = [0.0, 1.0];
    sim.world_mut().entity_mut(a).insert(c);
    let mut drive = PreviewDrive::default();
    canais::publicar(&mut sim, &[a.to_bits()]);
    quadro_em(&mut sim, &mut drive, None);
    let forca = |sim: &SimWorld| {
        sim.world()
            .get::<Constraints>(a)
            .unwrap()
            .item(s)
            .unwrap()
            .influence
    };
    assert!((forca(&sim) - 0.25).abs() < 1e-6, "{}", forca(&sim));
    let lidos: Vec<u64> = sim
        .world()
        .resource::<ph2d_ecs::LeitoresDosCanais>()
        .todas()
        .collect();
    assert_eq!(
        lidos,
        [a.to_bits()],
        "o dono de um destino de campo é publicado, selecionado ou não"
    );
    let vivo = drive.fields.substitute(&mut sim);
    assert_eq!(forca(&sim), 1.0, "o documento guarda o autorado");
    FieldDrive::restore(&mut sim, &vivo);
}

/// ⭐ Com mola, a saída é a do `Simulador<1>` com a fonte de cada quadro na faixa do destino — a
/// MESMA lei da Spring; arrastar a timeline para trás dá o bit de tocar.
#[test]
fn a_springy_value_rides_the_spring_and_scrubs_to_the_same_bit() {
    let mola = Mola {
        duracao: 0.5,
        ressalto: 0.6,
    };
    let valor = Valor {
        mola: Some(mola),
        ..Valor::default()
    };
    let (mut sim, a, _, _) = mundo(0.0, valor);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    let fps = doc.fps_display;
    let fonte = |f: i64| if f < 5 { 0.0 } else { 1.0 };
    let mut tocado = Vec::new();
    for f in 0..40 {
        por_y(&mut sim, fonte(f) as f32);
        let t = f as f64 / fps;
        quadro_em(
            &mut sim,
            &mut drive,
            Some(Relogio {
                doc: &doc,
                t,
                laco: None,
            }),
        );
        tocado.push(sim.world().get::<Transform>(a).unwrap().rotation);
    }
    let lei = Lei {
        duracao: mola.duracao,
        ressalto: mola.ressalto,
        angular: [false],
    };
    let mut oraculo: Simulador<1> = Simulador::default();
    for (f, got) in tocado.iter().enumerate() {
        let f = f as i64;
        let entrada = |k: i64| [valor.mapeia(fonte(k))];
        let esperado = entrada(f)[0] + oraculo.atraso(f, lei, fps, &mut |k| entrada(k))[0];
        assert_eq!(*got, esperado as f32, "quadro {f}");
    }
    let parado = tocado[39];
    assert!(
        tocado.iter().any(|r| *r > parado * 1.01),
        "a mola passa do alvo"
    );
    // Arrastar para trás, para o quadro 12: o bit de tocar.
    let t = 12.0 / fps;
    quadro_em(
        &mut sim,
        &mut drive,
        Some(Relogio {
            doc: &doc,
            t,
            laco: None,
        }),
    );
    let r = sim.world().get::<Transform>(a).unwrap().rotation;
    assert_eq!(r.to_bits(), tocado[12].to_bits());
}

#[test]
fn an_unknown_property_is_inert_with_a_reason() {
    let valor = Valor {
        fonte: Prop::Campo([u16::MAX, 1, 0, 0]),
        ..sem_mola()
    };
    let (mut sim, a, v, _) = mundo(0.5, valor);
    let mut drive = PreviewDrive::default();
    quadro_em(&mut sim, &mut drive, None);
    let estado = sim.world().resource::<Estados>().of(a, v);
    assert_eq!(estado, Some(Estado::SemValor));
    assert_eq!(sim.world().get::<Transform>(a).unwrap().rotation, 0.0);
}

#[test]
fn a_value_link_survives_save_and_open() {
    let valor = Valor {
        fonte: Prop::Campo([6, 3, 0, 0]),
        destino: Prop::Pose(Canal::Sx),
        ..Valor::default()
    };
    let (sim, a, _, _) = mundo(0.0, valor);
    let c = sim.world().get::<Constraints>(a).unwrap();
    let texto = serde_json::to_string(c).unwrap();
    let lido: Constraints = serde_json::from_str(&texto).unwrap();
    assert_eq!(&lido, c);
}
