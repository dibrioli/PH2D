use super::*;

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Bone::default()).id();
    (sim, e.to_bits())
}

fn manda(sim: &mut SimWorld, b: u64, campo: u16, v: f32) {
    let w = sim.world_mut();
    w.get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(b, k(campo), v);
}

fn osso(sim: &SimWorld, b: u64) -> Bone {
    *sim.world().get::<Bone>(Entity::from_bits(b)).expect("osso")
}

/// ⭐ A curva chega ao osso, campo a campo; os segmentos arredondam e a cerca do painel vale.
#[test]
fn a_curva_escreve_o_osso_pelas_cercas_do_painel() {
    let (mut sim, b) = cena();
    manda(&mut sim, b, o::COMPRIMENTO, 42.0);
    manda(&mut sim, b, o::FORCA, 2.5);
    manda(&mut sim, b, o::SEGMENTOS, 3.6);
    assert!(aplicar(&mut sim, &mut PreviewDrive::default()));
    let os = osso(&sim, b);
    assert!((os.length - 42.0).abs() < 1e-6, "{}", os.length);
    assert!((os.strength - 2.5).abs() < 1e-6, "{}", os.strength);
    assert_eq!(os.segments, 4);
    manda(&mut sim, b, o::COMPRIMENTO, -5.0);
    aplicar(&mut sim, &mut PreviewDrive::default());
    assert!(osso(&sim, b).length.abs() < f64::EPSILON);
}

/// Publicar põe os números do osso em `CanaisLidos`; sem osso, nada.
#[test]
fn publicar_le_os_tres_numeros_do_osso() {
    let (mut sim, b) = cena();
    let sem = sim.world_mut().spawn_empty().id().to_bits();
    publicar(&mut sim, &[b, sem]);
    let lidos = sim.world().get_resource::<CanaisLidos>().expect("lidos");
    assert_eq!(lidos.valor(b, k(o::COMPRIMENTO)), Some(1.0));
    assert_eq!(lidos.valor(b, k(o::SEGMENTOS)), Some(1.0));
    assert_eq!(lidos.valor(sem, k(o::FORCA)), None);
}

/// A tabela cobre o vocabulário da família e as quatro alças, sem id repetido.
#[test]
fn todo_numero_do_osso_tem_canal() {
    let campos: Vec<u16> = (CAMPOS.iter())
        .filter_map(|&(p, _)| {
            if let PropKind::Field(kk) = p {
                Some(kk.campo)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(campos, o::CAMPOS.to_vec());
    let mut ids: Vec<NodeId> = CAMPOS.iter().map(|&(_, n)| n).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), CAMPOS.len(), "um id em duas linhas");
    let alca = canal_do_campo(painel::VECTOR_BONE_CURVE_OUT_Y);
    assert_eq!(alca, Some(PropKind::BoneBendOutY));
}

/// O que o clique no losango `no` AUTORA (entidade, canal), com os campos que o osso PUBLICA (o
/// `campos_lidos` da shell): sem eles um campo não tem membro, e o clique só abre e fecha a edição.
fn autorado(sim: &SimWorld, sel: &[u64], no: NodeId) -> Vec<(u64, PropKind)> {
    let inst = Instante {
        t_cut: 0.0,
        solo: false,
    };
    let lidos = |e: u64| -> Vec<FieldKey> {
        let l = sim.world().get_resource::<CanaisLidos>();
        (CAMPOS.iter())
            .filter_map(|&(p, _)| match p {
                PropKind::Field(k) if l.is_some_and(|l| l.valor(e, k).is_some()) => Some(k),
                _ => None,
            })
            .collect()
    };
    let visto = std::cell::RefCell::new(Vec::new());
    let autorar = |e: u64, p: PropKind| {
        visto.borrow_mut().push((e, p));
        Some(TimelineIntent::BeginEdit)
    };
    let doc = TimelineDoc::new();
    let _ = clique(
        &doc,
        &[no],
        sim,
        sel,
        inst,
        lidos,
        |_, _| Some(1.0),
        autorar,
    );
    visto.into_inner()
}

/// ⭐ O clique grava sobre o OSSO escolhido; sem osso na selecção, nada.
#[test]
fn o_clique_grava_no_osso_escolhido() {
    let inst = Instante {
        t_cut: 0.0,
        solo: false,
    };
    let doc = TimelineDoc::new();
    let (mut sim, osso) = cena();
    let outro = sim.world_mut().spawn_empty().id().to_bits();
    publicar(&mut sim, &[osso, outro]);
    let comp = PropKind::Field(k(o::COMPRIMENTO));
    assert_eq!(
        autorado(&sim, &[outro, osso], painel::VECTOR_BONE_LENGTH),
        [(osso, comp)],
        "controlo: com osso grava"
    );
    assert!(autorado(&sim, &[outro], painel::VECTOR_BONE_LENGTH).is_empty());
    let mix = |sim: &SimWorld| {
        let autorar = |_: u64, _: PropKind| Some(TimelineIntent::BeginEdit);
        let campo = [painel::VECTOR_BONE_IK_MIX];
        clique(
            &doc,
            &campo,
            sim,
            &[osso],
            inst,
            |_| Vec::new(),
            |_, _| Some(1.0),
            autorar,
        )
    };
    assert!(mix(&sim).is_empty(), "sem âncora a linha não existe");
    let ik = ph2d_constraints::Ik::default();
    let e = Entity::from_bits(osso);
    let id = ph2d_constraints_ecs::por_ik(sim.world_mut(), e, ph2d_ecs::StableId::NONE, ik, 1.0);
    assert!(!mix(&sim).is_empty(), "a âncora grava no item {id}");
    let m = marcas(&doc, &sim, &[osso], inst, |_, _| Some(1.0));
    assert_eq!(m.len(), CAMPOS.len() + ANCORA.len());
}

/// ⭐ O losango de **Segments** grava a chave dos SEGMENTOS do osso (F41; o de cima usa Length).
#[test]
fn o_clique_em_segments_grava_os_segmentos() {
    let (mut sim, osso) = cena();
    publicar(&mut sim, &[osso]);
    let seg = PropKind::Field(k(o::SEGMENTOS));
    assert_eq!(
        autorado(&sim, &[osso], painel::VECTOR_BONE_SEGMENTS),
        [(osso, seg)]
    );
}

/// ⭐⭐ **A curva é pré-visualização** (F40): com o comprimento (OSSO) e uma alça (`BoneBend`)
/// conduzidos sobre valores postos à mão, o `substitute` (o que o undo e o save fotografam) lê o
/// AUTORADO, e o `restore` devolve o vivo.
#[test]
fn o_substitute_le_o_autorado_e_o_restore_o_vivo() {
    let (mut sim, b) = cena();
    let e = Entity::from_bits(b);
    let mao = |os: &mut Bone| {
        os.length = 5.0;
        os.curve.inn[1] = 0.25;
    };
    mao(&mut sim.world_mut().get_mut::<Bone>(e).expect("osso"));
    let mut drive = PreviewDrive::default();
    manda(&mut sim, b, o::COMPRIMENTO, 42.0);
    assert!(aplicar(&mut sim, &mut drive));
    let was = osso(&sim, b);
    sim.world_mut().get_mut::<Bone>(e).expect("osso").curve.inn[1] = 0.75;
    let antes = ph2d_timeline_preview::BoundBefore {
        entity: e,
        pose: None,
        alpha: None,
        morph_t: None,
        joint: None,
        bone: Some(was),
    };
    ph2d_timeline_preview::declare_timeline_writes(sim.world(), &[antes], &mut drive);
    let vivo = drive.substitute(&mut sim);
    let autorado = osso(&sim, b);
    assert!((autorado.length - 5.0).abs() < 1e-9, "{}", autorado.length);
    assert!(
        (autorado.curve.inn[1] - 0.25).abs() < 1e-9,
        "{}",
        autorado.curve.inn[1]
    );
    PreviewDrive::restore(&mut sim, &vivo);
    let de_novo = osso(&sim, b);
    assert!((de_novo.length - 42.0).abs() < 1e-9, "{}", de_novo.length);
    assert!((de_novo.curve.inn[1] - 0.75).abs() < 1e-9);
}
