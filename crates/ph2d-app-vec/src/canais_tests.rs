use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal_vetor as c;
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc, autokey_campos};
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecVertex};
use std::collections::BTreeMap;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

const VERMELHO: u16 = c::TINTA_PREENCH + c::OFF_SOLIDA;

/// Uma forma vermelha de traço 4, ligada a uma entidade; a curva leva o vermelho 255→0 e o traço
/// 4→12 num segundo.
fn cena() -> (SimWorld, VecScene, u64, TimelineDoc) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([10.0, 0.0]),
            VecVertex::corner([5.0, 8.0]),
        ],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 4.0)),
        ..VecPath::default()
    });
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    for (campo, a, b) in [(VERMELHO, 255.0, 0.0), (c::TRACO_LARGURA, 4.0, 12.0)] {
        let p = PropKind::Field(k(campo, 0, 0));
        doc.insert_key(e, p, s(0.0), AnimValue::Float(a), Interp::Linear);
        doc.insert_key(e, p, s(1.0), AnimValue::Float(b), Interp::Linear);
    }
    (sim, scene, e, doc)
}

fn lidos(sim: &SimWorld, e: u64) -> Vec<(FieldKey, f32)> {
    sim.world()
        .get_resource::<CanaisLidos>()
        .unwrap()
        .campos(e)
        .collect()
}

fn quadro(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    doc: &mut TimelineDoc,
    cond: &mut ConducaoVetor,
    e: u64,
    t: f64,
) {
    apply_from_doc(sim.world_mut(), doc, t);
    let _ = aplicar(sim, scene, cond);
    publicar(sim, scene, cond, &[e]);
}

/// ⭐⭐ **A curva chega ao desenho e a meio não há chave fantasma.** O vermelho a 0,5 s vale 127,5 na
/// curva e 128 no documento (`u8`): sem o eco o autokey via a mão em todo quadro.
#[test]
fn a_curva_chega_ao_desenho_e_parar_a_meio_nao_grava() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let p = &scene.paths()[0];
    assert_eq!(p.fill, Some(Paint::Solid(Rgba8::new(128, 0, 0, 255))));
    assert_eq!(p.stroke.as_ref().unwrap().width, 8.0);

    let l = lidos(&sim, e);
    let base = l.iter().copied().collect();
    let plan = autokey_campos(&doc, e, 0.5, &l, &base, true, false);
    assert!(
        plan.is_empty(),
        "parado a meio, na curva, gravou {:?}",
        plan.keys
    );
}

/// A mão muda o traço com o Auto Key armado ⇒ a chave do traço, no valor da mão — e só dele.
#[test]
fn a_mao_no_traco_vira_chave_do_traco() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let base: BTreeMap<FieldKey, f32> = lidos(&sim, e).into_iter().collect();
    scene
        .path_mut(scene.paths()[0].id)
        .unwrap()
        .stroke
        .as_mut()
        .unwrap()
        .width = 20.0;
    publicar(&mut sim, &scene, &ConducaoVetor::default(), &[e]);
    let plan = autokey_campos(&doc, e, 0.5, &lidos(&sim, e), &base, true, false);
    assert_eq!(
        plan.keys,
        vec![(PropKind::Field(k(c::TRACO_LARGURA, 0, 0)), 20.0)]
    );
}

/// ⭐ **Undo e save fotografam o AUTORADO enquanto a curva move o desenho** (a lei do
/// `ph2d_preview_drive`); com a timeline PARADA, o mesmo instante duas vezes não muda nada e o valor
/// da curva vira documento. (Parado = a timeline não toca, não «o relógio não avançou»: a tocar, o
/// mesmo instante repetido é o gate de baixo.)
#[test]
fn a_fotografia_ve_o_autorado_e_parado_a_curva_vira_documento() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let vivo = cond.substituir(&mut sim, &mut scene);
    let p = &scene.paths()[0];
    assert_eq!(
        p.fill,
        Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        "a fotografia vê o autorado"
    );
    assert_eq!(p.stroke.as_ref().unwrap().width, 4.0);
    cond.repor(&mut sim, &mut scene, vivo);
    assert_eq!(
        scene.paths()[0].stroke.as_ref().unwrap().width,
        8.0,
        "e o vivo volta"
    );

    cond.settle();
    cond.a_tocar(false); // a timeline PAROU (pausa ou scrub parado)
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5); // o mesmo instante: nada muda
    cond.settle();
    assert!(cond.is_empty(), "parada a timeline, a curva já é documento");
}

/// ⭐⭐ **A tocar, um quadro em que o relógio repete o instante não vira documento** (auditoria F31,
/// medido no app: f=38 e f=39 com t=0,533 ⇒ 120 passos de undo em ~7 s e o Ctrl+Z devolvia a pose
/// da reprodução). A curva continua a conduzir o campo; só não o mudou.
#[test]
fn a_tocar_um_quadro_sem_avanco_nao_vira_documento() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    cond.a_tocar(true);
    for _ in 0..2 {
        quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
        cond.settle();
    }
    let vivo = cond.substituir(&mut sim, &mut scene);
    let p = &scene.paths()[0];
    assert_eq!(
        p.fill,
        Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        "a tocar, a fotografia vê o autorado"
    );
    assert_eq!(p.stroke.as_ref().unwrap().width, 4.0);
    cond.repor(&mut sim, &mut scene, vivo);
    assert_eq!(scene.paths()[0].stroke.as_ref().unwrap().width, 8.0);
}

/// ⭐ Os efeitos VIVOS da entidade (Contour, Offset) publicam-se e a curva conduz-nos — o encaminho
/// de `canais_efeitos` em `publicar` e `aplicar`. (Nasceu de uma mutação sobrevivente, F24c: apagar
/// a publicação dos efeitos passava os 16 testes de `canais`.)
#[test]
fn os_efeitos_vivos_publicam_se_e_a_curva_os_conduz() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let ent = ph2d_ecs::Entity::from_bits(e);
    sim.world_mut().entity_mut(ent).insert((
        ph2d_ecs::VecContour {
            steps: 3,
            d: 0.1,
            ..ph2d_ecs::VecContour::default()
        },
        ph2d_ecs::VecOffset::new(0.2, 1, 0),
    ));
    let dist = k(c::CONTORNO_DISTANCIA, 0, 0);
    let s = RationalTime::from_seconds;
    for (t, v) in [(0.0, 0.1), (1.0, 0.5)] {
        doc.insert_key(
            e,
            PropKind::Field(dist),
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let mut cond = ConducaoVetor::default();
    publicar(&mut sim, &scene, &cond, &[e]);
    let l = lidos(&sim, e);
    assert!(
        l.contains(&(k(c::CONTORNO_PASSOS, 0, 0), 3.0)),
        "o Contour não foi publicado"
    );
    assert!(
        l.contains(&(k(c::OFFSET_JUNCAO, 0, 0), 1.0)),
        "o Offset não foi publicado"
    );
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let d = sim.world().get::<ph2d_ecs::VecContour>(ent).unwrap().d;
    assert!(
        (d - 0.3).abs() < 1e-5,
        "a curva não conduziu o Contour: d = {d}"
    );
}

/// ⭐ Bloco 2, de ponta a ponta, um por DONO: um campo do caminho (a escala do pincel do traço), um
/// componente de uma entidade SEM caminho (a curvatura do Envelope no contentor) e um texto (a
/// família da fonte, pelo índice na tabela). `publicar` → a curva em `CanaisEscritos` → `aplicar`
/// muda o documento.
#[test]
fn o_bloco_2_publica_e_a_curva_conduz_cada_dono() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let id = scene.paths()[0].id;
    scene.path_mut(id).unwrap().stroke.as_mut().unwrap().paint =
        ph2d_vec_scene::StrokePaint::Brush(Box::default());
    let contentor = sim
        .world_mut()
        .spawn(ph2d_ecs::VecEnvelope::at_rest(
            Vec::new(),
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            [[[0.0, 0.0]; 2]; 4],
        ))
        .id()
        .to_bits();
    let ent = Entity::from_bits(e);
    sim.world_mut()
        .entity_mut(ent)
        .insert(VecShape::Text(ph2d_ecs::VecTextParams {
            text: "Olá".to_owned(),
            origin: [0.0, 0.0],
            family: None,
            size: 12.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: 0,
            axes: Vec::new(),
            wrap_width: None,
        }));
    let mut textos = TextosDosCanais::default();
    let roboto = textos.indice(Some("Roboto"));
    sim.world_mut().insert_resource(textos);
    let s = RationalTime::from_seconds;
    let escala = k(c::PINCEL_ESCALA, 0, 0);
    let curva = k(c::ENVELOPE_CURVA, 0, 0);
    let familia = k(c::TEXTO_FAMILIA, 0, 0);
    for (dono, kk, a, b) in [
        (e, escala, 1.0, 3.0),
        (contentor, curva, 0.0, 1.0),
        (e, familia, roboto, roboto),
    ] {
        for (t, v) in [(0.0, a), (1.0, b)] {
            doc.insert_key(
                dono,
                PropKind::Field(kk),
                s(t),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
    }
    publicar(&mut sim, &scene, &ConducaoVetor::default(), &[e, contentor]);
    let l = lidos(&sim, e);
    assert!(l.contains(&(escala, 1.0)), "o pincel não foi publicado");
    assert!(
        l.iter().any(|&(kk, _)| kk == familia),
        "a família não foi publicada"
    );
    assert!(
        lidos(&sim, contentor).contains(&(curva, 0.5)),
        "o Envelope de um contentor sem caminho não foi publicado"
    );
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let b = scene.paths()[0]
        .stroke
        .as_ref()
        .unwrap()
        .brush()
        .unwrap()
        .scale;
    assert_eq!(b, 2.0, "a curva não conduziu o pincel");
    let bend = sim
        .world()
        .get::<ph2d_ecs::VecEnvelope>(Entity::from_bits(contentor))
        .unwrap()
        .bend;
    assert_eq!(bend, 0.5, "a curva não conduziu o Envelope");
    let Some(VecShape::Text(t)) = sim.world().get::<VecShape>(ent) else {
        panic!("o texto sumiu")
    };
    assert_eq!(
        t.family.as_deref(),
        Some("Roboto"),
        "a curva não trocou a fonte"
    );
    assert!(
        sim.world().get_resource::<TextosDosCanais>().is_some(),
        "a tabela dos textos volta ao mundo"
    );
}

/// ⭐ A medida animada passa pela porta do quadro do vetor (a que a shell chama): a curva da família `CAD` estica o
/// desenho, e o documento publicado é o AUTORADO — os cantos que o solver move não são o artista a mexer (o Auto
/// Key não os grava) — com a medida da curva.
#[test]
fn a_dimension_curve_moves_the_drawing_but_the_document_publishes_the_authored_corners() {
    use ph2d_ecs::canal_cad::{MEDIDA, chave};
    use ph2d_vec_scene::VertId;
    use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [30.0, 0.0], [30.0, 40.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    });
    let v = |i| PontoCad::Vert(id, VertId(i));
    *scene.regras_mut() = vec![
        RegraCad::nova(Lei::Fixo(v(1), [0.0, 0.0])),
        RegraCad::nova(Lei::Horizontal(SegCad(v(1), v(2)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(2), v(3)))),
        RegraCad::nova(Lei::Distancia(v(2), v(3), 40.0)),
    ];
    scene.normaliza_ids();
    let kk = chave(MEDIDA, scene.regras()[3].id.0);
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    for (t, m) in [(0.0, 40.0), (1.0, 70.0)] {
        doc.insert_key(
            e,
            PropKind::Field(kk),
            s(t),
            AnimValue::Float(m),
            Interp::Linear,
        );
    }
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 1.0);
    assert!((scene.path(id).unwrap().verts[2].anchor[1] - 70.0).abs() < 1e-6);
    let l = lidos(&sim, e);
    let y3 = l
        .iter()
        .find(|(k, _)| k.campo == c::ANCORA_Y && k.j == 2)
        .map(|&(_, y)| y);
    assert_eq!(y3, Some(40.0), "o canto publicado é o autorado");
    assert!(l.contains(&(kk, 70.0)), "a medida publicada é a da curva");
    let vivo = cond.substituir(&mut sim, &mut scene);
    assert_eq!(
        scene.path(id).unwrap().verts[2].anchor[1],
        40.0,
        "o undo fotografa o autorado"
    );
    cond.repor(&mut sim, &mut scene, vivo);
    assert!((scene.path(id).unwrap().verts[2].anchor[1] - 70.0).abs() < 1e-6);
}
