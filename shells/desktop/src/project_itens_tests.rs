//! **Os itens das listas no arquivo** (F30 §A1 v195 · F31 §B v196 · F32 v197): um ficheiro antigo abre
//! com os blobs na forma nova (os ids nascem no primeiro passe, na ordem da lista).

use super::*;

/// Um componente gravado à mão (`nome` canónico, os bytes de então).
fn blob(nome: &str, data: Vec<u8>) -> ph2d_asset::ComponentBlob {
    let type_id = ph2d_ecs::scene::registry::stable_type_id(nome);
    ph2d_asset::ComponentBlob { type_id, data }
}

/// `f` com uma entidade que leva `components`.
fn com_entidade(mut f: ProjectFile, components: Vec<ph2d_asset::ComponentBlob>) -> ProjectFile {
    let row = ph2d_ecs::scene::EntitySnapshotRow {
        id: ph2d_ecs::StableId(5),
        components,
        parent: None,
    };
    f.state.world.entities.push(std::sync::Arc::new(row));
    f
}

/// Grava `bytes` num ficheiro, abre-o pela porta do load e devolve o que ela leu.
fn abre(nome: &str, bytes: &[u8]) -> ProjectFile {
    let path = tmp_path(nome);
    std::fs::write(&path, bytes).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    lido.unwrap_or_else(|| panic!("{nome}: o ficheiro antigo é aceite"))
        .0
}

/// O blob `k` da entidade lida, no tipo de agora.
fn componente<T: serde::de::DeserializeOwned>(f: &ProjectFile, k: usize) -> T {
    postcard::from_bytes(&f.state.world.entities[0].components[k].data).expect("forma nova")
}

/// ⭐ **Um projecto v194 abre com os timers legíveis** — os bytes do `Timers` escritos na forma de
/// então, campo a campo (não pelo tipo vivo, senão o teste provaria o braço com a hipótese dele).
#[test]
fn um_projecto_v194_abre_com_os_itens_na_forma_nova() {
    let v194: Vec<(String, u64, bool, bool, String)> = vec![
        ("a".into(), 1_000_000, false, true, String::new()),
        ("b".into(), 3_000_000, true, false, "s".into()),
    ];
    let data = postcard::to_allocvec(&v194).expect("grava");
    let f = com_entidade(
        bare_file(empty_state()),
        vec![blob("ph2d::ecs::Timers", data)],
    );
    let back = abre(
        "itens_v194",
        &postcard::to_allocvec(&(204u32, &f)).expect("serializa"),
    );
    let t: ph2d_ecs::Timers = componente(&back, 0);
    assert_eq!(t.0[1].duration_us, 3_000_000);
    assert!(
        t.0.iter().all(|x| x.item.is_none()),
        "o passe é que dá os ids"
    );
}

/// ⭐ **Um projecto v195 lê o `VecFilter` sem ids** (F31 §B2): os bytes da pilha escritos na forma
/// de então (só os degraus, pelo `FxOp` da folha), abrem com os mesmos degraus e ids por atribuir.
#[test]
fn o_projeto_195_le_o_vec_filter_sem_ids() {
    use ph2d_ecs::FxOp;
    let mut sombra = FxOp::new(FxOp::DROP_SHADOW);
    sombra.radius = 0.25;
    let v195 = (vec![FxOp::new(FxOp::BLUR), sombra],);
    let data = postcard::to_allocvec(&v195).expect("grava");
    let f = com_entidade(
        bare_file(empty_state()),
        vec![blob("ph2d::ecs::VecFilter", data)],
    );
    let back = abre(
        "vec_filter_v195",
        &postcard::to_allocvec(&(205u32, &f)).expect("serializa"),
    );
    let filtro: ph2d_ecs::VecFilter = componente(&back, 0);
    assert_eq!(filtro.ops(), v195.0.as_slice());
    assert_eq!(filtro.ids().len(), 2);
    assert!(
        filtro.ids().iter().all(|i| i.is_none()),
        "o passe é que dá os ids"
    );
}

/// ⭐ **Um projecto v195 lê a pilha do Vetor sem ids** (F31 §B1): a `VecScene` dentro do ficheiro na
/// forma v23 (os bytes gravados pelo código v23, `vec_scene_v23_pilha.bin`) abre com as mesmas
/// tintas e efeitos e os ids por atribuir. Sem o âmbito `PilhaLayoutV23` o ficheiro não se lê igual.
#[test]
fn o_projeto_199_le_a_pilha_do_vetor_sem_ids() {
    const V23: &[u8] =
        include_bytes!("../../../crates/ph2d-vec-scene/tests/it/fixtures/vec_scene_v23_pilha.bin");
    let cena = ph2d_vec_scene::VecScene::from_bytes(V23).expect("a fixtura v23 abre");
    let mut st = empty_state();
    st.vec = std::sync::Arc::new(cena.clone());
    let f = bare_file(st);
    // A cena VIVA dentro do ficheiro troca-se pelos bytes v23 (o mesmo valor, a forma antiga).
    let vivo = postcard::to_allocvec(&(199u32, &f)).expect("serializa");
    let cena_viva = postcard::to_allocvec(&cena).expect("serializa");
    let iguais = |w: &&[u8]| *w == cena_viva.as_slice();
    assert_eq!(vivo.windows(cena_viva.len()).filter(iguais).count(), 1);
    let at = vivo
        .windows(cena_viva.len())
        .position(|w| iguais(&w))
        .expect("a cena está lá");
    let back = abre(
        "pilha_v199",
        &[&vivo[..at], &V23[1..], &vivo[at + cena_viva.len()..]].concat(),
    );
    assert_eq!(
        *back.state.vec, cena,
        "as mesmas tintas e efeitos, ids NONE"
    );
    let p = &back.state.vec.paths()[0];
    assert_eq!(p.paints.len(), 2);
    assert!(
        p.effects.iter().all(|e| e.id.is_none()),
        "o passe é que dá os ids"
    );
}

/// ⭐ **Um projecto v196 lê os relógios e as poses sem ids** (F32, `197`): `Tweens`/`PathFollow` sem o
/// último campo e a pose gravada no âmbito `PoseLayoutV196` abrem com relógio e ids por atribuir.
#[test]
fn o_projeto_196_le_os_relogios_e_as_poses_sem_ids() {
    let tw = ph2d_ecs::Tweens(vec![Default::default()]);
    let pf = ph2d_ecs::PathFollow {
        relogio: 1,
        ..Default::default()
    };
    let sem_o_id = |nome: &str, mut data: Vec<u8>| {
        assert_eq!(data.pop(), Some(0), "o id NONE do fim é um byte 0");
        blob(nome, data)
    };
    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let mut pose = ph2d_ui_state::ObjectPose::new(id);
    pose.filters = vec![ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BLUR)];
    let mut st = ph2d_ui_state::UiState::new(ph2d_ui_state::StateRole::Default);
    st.objects = vec![pose];
    let mut ui_states = ph2d_ui_state::StateSets::default();
    ui_states.set(id, st);
    let vec = std::sync::Arc::new(scene);
    let components = vec![
        sem_o_id("ph2d::ecs::Tweens", postcard::to_allocvec(&tw).expect("t")),
        sem_o_id(
            "ph2d::ecs::PathFollow",
            postcard::to_allocvec(&pf).expect("p"),
        ),
    ];
    let f = com_entidade(
        bare_file(ProjectState {
            vec,
            ui_states,
            ..empty_state()
        }),
        components,
    );
    let v196 = ph2d_ui_state::PoseLayoutV196::enter();
    let bytes = postcard::to_allocvec(&(206u32, &f)).expect("serializa");
    drop(v196);
    let back = abre("relogios_v196", &bytes);
    assert_eq!(componente::<ph2d_ecs::Tweens>(&back, 0), tw);
    assert_eq!(componente::<ph2d_ecs::PathFollow>(&back, 1), pf);
    let pose = &back.state.ui_states.get(id)[0].objects[0];
    assert_eq!((pose.filters.len(), pose.filter_ids.len()), (1, 0));
}
