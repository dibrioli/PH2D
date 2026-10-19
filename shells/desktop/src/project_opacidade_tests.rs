//! ⭐ **F22b — um ficheiro `<= 191` abre com a opacidade da FORMA nas poses dos estados** (degrau
//! `192`). Até lá o `capture` gravava sempre `1`; sem a migração, o Show de um projeto antigo poria a
//! 100 % uma forma que o artista deixou a 40 %.

use super::*;

#[test]
fn um_v191_abre_com_a_opacidade_da_forma_nas_poses() {
    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    scene.path_mut(id).expect("forma").opacity = ph2d_vec_scene::Opacity::new(0.4);
    let mut states = ph2d_ui_state::StateSets::default();
    let mut st = ph2d_ui_state::UiState::new(ph2d_ui_state::StateRole::Default);
    // A pose como o `capture` antigo a gravava: opacidade `1`.
    st.objects = vec![ph2d_ui_state::ObjectPose::new(id)];
    states.set(id, st);
    let state = ProjectState {
        vec: std::sync::Arc::new(scene),
        ui_states: states,
        ..empty_state()
    };
    // O controlo: um ficheiro do degrau corrente NÃO é migrado (a pose diz o que o Rec gravou).
    for (ver, esperado) in [(191u32, 0.4f32), (PROJECT_SCHEMA, 1.0)] {
        let path = tmp_path(&format!("opacidade_v{ver}"));
        let bytes = postcard::to_allocvec(&(ver, &bare_file(state.clone()))).expect("serializa");
        std::fs::write(&path, bytes).expect("grava");
        let mut app = headless_app();
        let (file, _) = app
            .read_project_file(path.to_str().expect("caminho"))
            .expect("o ficheiro abre");
        let op = file.state.ui_states.get(id)[0].objects[0].opacity;
        assert!((op - esperado).abs() < 1e-6, "v{ver}: a pose abriu a {op}");
        let _ = std::fs::remove_file(&path);
    }
}
