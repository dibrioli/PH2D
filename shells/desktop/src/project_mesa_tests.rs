//! **A MESA DE SOM no arquivo** (F30 §B2, v194): o blob atravessa o `.ph2dproj`, e um v193 (ou
//! anterior, sem o campo) abre com a mesa no padrão.

use super::*;

/// ⭐ **A mesa atravessa o arquivo** — o que o artista deixou na mesa volta igual ao abrir.
#[test]
fn a_mesa_viaja_no_projecto() {
    let mut mesa = ph2d_audio::Mesa::default();
    mesa.bus[1].ganho = 0.4;
    mesa.ducking_chave = 0;
    let file = ProjectFile {
        mesa: mesa.to_bytes(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("mesa_v194");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa"),
    )
    .expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("aceite");
    assert_eq!(
        ph2d_audio::Mesa::from_bytes(&back.mesa).expect("mesa"),
        mesa
    );
}

/// ⭐ **Um v193 e um v192 abrem, com a mesa no padrão.** ⚠️ Os bytes são escritos campo a campo (a
/// forma de então) — não a partir do tipo vivo, senão o teste provaria o braço com a hipótese dele.
#[test]
fn um_projecto_antigo_abre_com_a_mesa_no_padrao() {
    for ver in [193u32, 192] {
        let f = bare_file(empty_state());
        let ate_aos_scripts = (
            &f.assets,
            &f.painted,
            &f.motion,
            &f.timeline,
            &f.physics,
            &f.tokens,
            &f.settings,
            &f.player_tape,
            &f.sprite_pixels,
            &f.stable_id_counter,
            &f.input_map,
            &f.pattern_art,
            &f.boards,
            &f.scripts,
        );
        let marcas = vec![1u8, 2, 3];
        // A cena vetorial de um ficheiro destes é a v23 (sem as regras de CAD, que nasceram no 200): a vazia
        // viva acaba nos dois zeros das regras.
        let st = &f.state;
        let vec_vivo = postcard::to_allocvec(&*st.vec).expect("serializa");
        let mut bytes = postcard::to_allocvec(&(ver, &st.world)).expect("serializa");
        bytes.extend_from_slice(&vec_vivo[..vec_vivo.len() - 2]);
        let resto = (&st.flip, &st.guides, &st.ui_states, &st.library, &st.tags);
        bytes.extend(postcard::to_allocvec(&resto).expect("serializa"));
        bytes.extend(postcard::to_allocvec(&ate_aos_scripts).expect("serializa"));
        if ver == 193 {
            bytes.extend(postcard::to_allocvec(&marcas).expect("serializa"));
        }
        let path = tmp_path(&format!("mesa_v{ver}"));
        std::fs::write(&path, bytes).expect("grava");
        let lido = headless_app().read_project_file(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let (back, _) = lido.expect("um projecto antigo é recusado");
        assert!(back.mesa.is_empty(), "v{ver}: a mesa nasce no padrão");
        let esperadas = if ver == 193 { marcas } else { Vec::new() };
        assert_eq!(
            back.marcas_do_artista, esperadas,
            "v{ver}: as marcas não se desalinham"
        );
    }
}
