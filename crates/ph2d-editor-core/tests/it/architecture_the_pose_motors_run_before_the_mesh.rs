//! **Arch-gate: os motores que ESCREVEM pose de esqueleto correm antes de quem a LÊ.**
//!
//! ⛔⛔ **O defeito** (report do dono, 2026-09-14, com duas fotos: *«ao acrescentar o IK, de algum
//! modo o osso perde influência sobre a ponta da malha»*): o osso inteligente e a âncora de IK
//! viviam na fase de CANVAS, imediatamente antes da pele **vectorial** — e o comentário que os punha
//! ali já escrevia a lei: *«ela escreve a pose dos ossos, e o recook é quem transforma a pose em
//! geometria. Ao contrário, a pele mostraria a pose do quadro anterior.»*
//!
//! ⚠️ **A SEGUNDA MÍDIA chegou depois e não herdou a arrumação:** a malha de uma imagem presa é
//! posta na `fase_sim_extract`, que corre na metade da **simulação**, antes daquela fase inteira.
//! ⇒ a malha lia a pose de ANTES do solver. E não era um atraso de um quadro: o apply da timeline
//! corre na MESMA metade e repõe todo osso keyado pela curva mesmo a tempo — o gizmo mostrava a IK
//! e a arte mostrava a curva, para sempre.
//!
//! ⚠️ **A ordem é medida DENTRO DE UM FICHEIRO SÓ**, e isso é deliberado: concatenar dois ficheiros
//! para medir uma ordem é fraude (tudo o que está no segundo vem depois de tudo o que está no
//! primeiro, logo a asserção passaria por construção — a lição que o corte do teclado do sculpt
//! pagou em 2026-09-04). As duas chamadas vivem na `fase_frame_simulation`.
//!
//! ⛔ E há a metade que impede o defeito de voltar pela outra porta: os motores **não podem**
//! continuar a ser chamados da fase de canvas, senão eles correriam DUAS vezes por quadro.

use std::path::Path;

fn shell_src() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src")
}

#[test]
fn the_skeleton_pose_motors_run_before_the_mesh_is_built() {
    let f = shell_src().join("render_loop/fase_frame_open.rs");
    let src = std::fs::read_to_string(&f).expect("a fase que abre o quadro");
    let motores = src
        .find("self.fase_skeleton_drives();")
        .unwrap_or_else(|| panic!("{} não chama os motores de pose do esqueleto", f.display()));
    let malha = src
        .find("self.fase_sim_extract(")
        .unwrap_or_else(|| panic!("{} não põe a malha das sprites", f.display()));
    assert!(
        motores < malha,
        "{} constrói a malha ANTES de o osso inteligente e a âncora de IK escreverem a pose: a arte \
         de uma imagem presa mostra a curva que o apply acabou de repor, e o gizmo mostra a pose \
         resolvida. Duas leituras da mesma pose, e a que o artista vê é a errada.",
        f.display()
    );
}

/// ⛔ **A outra porta:** se a fase de canvas voltar a chamá-los, eles correm DUAS vezes por quadro —
/// o dobro do custo, e o segundo solve vê uma pose que o primeiro já mexeu.
#[test]
fn the_canvas_half_does_not_drive_the_skeleton_a_second_time() {
    let f = shell_src().join("render_loop/fase_vector_view_and_drives.rs");
    let src: String = std::fs::read_to_string(&f)
        .expect("a fase das vistas e dos motores do vector")
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n");
    // Controlo positivo: é ESTE o ficheiro que recozinha a pele vectorial — sem ele o gate mede o nada.
    assert!(
        src.contains("skeleton_live::recook("),
        "{} deixou de recozinhar a pele vectorial — este gate perdeu o sujeito",
        f.display()
    );
    for porta in ["skeleton_goal::solve(", "skeleton_smart::drive("] {
        assert!(
            !src.contains(porta),
            "{} chama `{porta}` outra vez: os motores de pose mudaram-se para a metade da SIMULAÇÃO \
             (ver o gate irmão), e deixá-los aqui fá-los correr DUAS vezes por quadro.",
            f.display()
        );
    }
}
