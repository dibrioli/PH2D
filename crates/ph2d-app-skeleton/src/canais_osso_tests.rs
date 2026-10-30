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
    assert!(aplicar(&mut sim));
    let os = osso(&sim, b);
    assert!((os.length - 42.0).abs() < 1e-6, "{}", os.length);
    assert!((os.strength - 2.5).abs() < 1e-6, "{}", os.strength);
    assert_eq!(os.segments, 4);
    manda(&mut sim, b, o::COMPRIMENTO, -5.0);
    aplicar(&mut sim);
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
    let clica = |sel: &[u64]| {
        let autorar = |_: u64, _: PropKind| Some(TimelineIntent::BeginEdit);
        let campo = [painel::VECTOR_BONE_LENGTH];
        clique(
            &doc,
            &campo,
            &sim,
            sel,
            inst,
            |_| Vec::new(),
            |_, _| Some(1.0),
            autorar,
        )
    };
    assert!(
        !clica(&[outro, osso]).is_empty(),
        "controlo: com osso grava"
    );
    assert!(clica(&[outro]).is_empty());
    assert_eq!(
        marcas(&doc, &sim, &[osso], inst, |_, _| Some(1.0)).len(),
        CAMPOS.len()
    );
}
