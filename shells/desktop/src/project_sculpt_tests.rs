//! **O que um load faz com a ESCULTURA** (3D, W8.3) — filho de
//! [`super`] (`project::tests`) pelo teto de LOC, e o corte é por assunto: lá
//! *o que a sessão esquece*, aqui *o que o documento de barro atravessa*.
//!
//! As fixtures vêm do pai (`headless_app`, `write_project_full`, `tmp_path`) —
//! um segundo escritor de arquivo de projeto divergiria no próximo bump de
//! schema, e é exatamente o erro que este arquivo não pode conter.

use super::*;

/// Bytes de uma escultura de UMA peça, escritos pela porta real do módulo.
#[cfg(feature = "sculpt3d")]
fn a_sculpture() -> Vec<u8> {
    let mut stack = ph2d_mesh::Multires::new(ph2d_mesh::shapes::octahedron(1.0));
    assert!(stack.add_level(), "a fixture precisa do 2º nível");
    stack.mesh_mut().positions_mut()[0][1] += 0.25;
    crate::sculpt3d::encode_doc(&[(stack.to_data(), ph2d_mesh::Pose::IDENTITY.to_data())], 0)
}

/// **UMA ESCULTURA ILEGÍVEL RECUSA O LOAD INTEIRO** — a lei da timeline, pelo
/// mesmo motivo.
///
/// ⚠️ Abrir *sem* ela mostraria uma cena que parece certa, com a escultura
/// vazia, e o **próximo Ctrl+S gravaria esse vazio por cima do arquivo**: a obra
/// não sumiria por um bug, sumiria porque o app abriu, mentiu e salvou.
///
/// E a recusa acontece **antes de qualquer mutação da sessão** — é isto que o
/// relógio e o histórico intactos medem aqui, e é por isso que o parse mora no
/// topo do load em vez de junto da instalação.
#[cfg(feature = "sculpt3d")]
#[test]
fn an_unreadable_sculpture_refuses_the_whole_load() {
    let mut app = headless_app();
    app.playhead.play();
    app.playhead.advance_ticks(120);
    let before = app.playhead.time();
    app.undo.push_undo(empty_state());

    let path = tmp_path("sculpt_unreadable");
    // Bytes que não são um documento de escultura — o arquivo remendado.
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), vec![0xff; 24]);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        app.playhead.time(),
        before,
        "load recusado NÃO rebobina o relógio"
    );
    assert!(
        app.undo.can_undo(),
        "…nem joga fora o histórico do documento que continua aberto"
    );
    assert!(
        app.sculpt_doc.is_empty(),
        "…nem adota os bytes recusados: o próximo save gravaria o lixo de volta"
    );
    assert!(
        app.sculpt3d_pending.is_none(),
        "…e nada fica pendente para o frame instalar"
    );
}

/// **Um projeto COM escultura a deixa pendente para o frame.**
///
/// ⚠️ O load é dirigível sem janela e a cena 3D não nasce sem `wgpu::Device`,
/// então ele **decodifica e estaciona**; quem constrói é o
/// `sculpt3d_install_pending` do frame. É por isso que a asserção aqui é sobre
/// a pendência, e não sobre `gfx.sculpt3d`.
#[cfg(feature = "sculpt3d")]
#[test]
fn a_loaded_project_leaves_its_sculpture_pending_for_the_frame() {
    let mut app = headless_app();
    assert!(app.sculpt3d_pending.is_none(), "sessão em branco");

    let path = tmp_path("sculpt_pending");
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), a_sculpture());
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    let (pieces, active) = app
        .sculpt3d_pending
        .as_ref()
        .expect("a escultura do arquivo");
    assert_eq!(pieces.len(), 1, "a peça do arquivo");
    assert_eq!(*active, 0);
    assert_eq!(
        pieces[0].0.level_count(),
        2,
        "a PILHA inteira volta, não só a malha viva"
    );
}

/// **Um projeto SEM escultura não deixa nada pendente** — e é o controle que dá
/// sentido ao gate acima: sem ele, `Some` a toda hora passaria pelos dois.
#[cfg(feature = "sculpt3d")]
#[test]
fn a_project_without_a_sculpture_leaves_nothing_pending() {
    let mut app = headless_app();
    let path = tmp_path("sculpt_none");
    write_project(&path, PROJECT_SCHEMA);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert!(app.sculpt3d_pending.is_none());
    assert!(app.sculpt_doc.is_empty());
}

/// **Um build que não constrói a escultura a PASSA ADIANTE** — ele não é um
/// triturador.
///
/// ⚠️ É por isto que `ProjectFile.sculpt` é um `Vec<u8>` **incondicional**, sem
/// `cfg`: um campo condicional daria duas formas de arquivo sob um número de
/// schema. Aqui a mesma propriedade é medida no caso que de fato acontece — um
/// `App` **sem janela**, que não tem cena 3D para consultar: o save devolve os
/// bytes que o load trouxe, byte a byte, em vez de gravar vazio por cima da obra.
#[test]
fn a_session_that_cannot_build_the_sculpture_hands_its_bytes_back() {
    let mut app = headless_app();
    let stashed = vec![7u8, 1, 2, 3, 4, 5];
    let path = tmp_path("sculpt_passthrough");
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), stashed.clone());
    // O documento tem de ser LEGÍVEL para o load o aceitar; num build com a
    // feature, bytes de lixo seriam recusados (o gate acima), então a fixture
    // que atravessa é a real.
    #[cfg(feature = "sculpt3d")]
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), a_sculpture());
    #[cfg(feature = "sculpt3d")]
    let stashed = a_sculpture();

    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        app.sculpt_bytes_for_save(),
        stashed,
        "sem cena viva, o save devolve o que o load trouxe — nunca vazio"
    );
}
