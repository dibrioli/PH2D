//! ⛔⛔ **O gesto da borda tem de CHEGAR ao painel — e os gates de geometria não o vêem.**
//!
//! A lei de fechar por arrasto vive no `ph2d-editor-core` e é medida lá: a área devolvida, a
//! exclusão entre a costura e a alça, o degrau uma linha abaixo do mínimo. ⚠️ **Nada disso afirma
//! que a shell os CHAMA.** Um gesto ligado a nada passa em todos aqueles testes — é a espécie de
//! defeito que este repo varre a cada wave (*«um controlo pintado que não alcança consumidor»*).
//!
//! ⚠️ **Este gate lê o FONTE, e é o idioma da casa para esta costura:** o irmão
//! `the_arrangement_is_read_at_boot_and_written_on_change` já o faz, com o motivo escrito — o
//! `dock_seam_move`/`_up` **faz `return` no `input_dispatch`** e não passa pelos hooks que um
//! teste de ponteiro conseguiria conduzir.

use std::fs;
use std::path::PathBuf;

fn src(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⭐⭐⭐ **Arrastar para além do degrau FECHA, e fecha pela porta do menu.**
#[test]
fn dragging_past_the_step_closes_the_column_through_the_menus_own_door() {
    let s = src("src/dock_resize.rs");
    assert!(
        s.contains("DOCK_W_COLLAPSE"),
        "o arrasto da borda nao consulta o degrau de fecho: ele volta a travar no minimo, e fechar \
         volta a custar dois passeios ao menu"
    );
    assert!(
        s.contains("panel_visibility.insert(tenant, false)"),
        "o arrasto nao fecha pela porta `panel_visibility` — um segundo caminho para esconder um \
         painel daria dois estados de «fechado» que podem discordar, e o interruptor do menu \
         passaria a mentir sobre o que o dedo fez"
    );
}

/// ⭐⭐ **E a ALÇA reabre** — sem isto, o gesto de fechar e' uma armadilha num tablet.
#[test]
fn the_handle_reopens_the_column_it_closed() {
    let s = src("src/dock_resize.rs");
    assert!(
        s.contains("dock_reopen_at"),
        "a shell nao pergunta pela alca: com a coluna fechada a borda fica MUDA, e num ecra~ de \
         toque nao ha' cursor que denuncie uma costura invisivel"
    );
    assert!(
        s.contains("panel_visibility.insert(tenant, true)"),
        "a alca nao reabre a coluna pela mesma porta que a fechou"
    );
}

/// ⛔ **E ela é PINTADA** — a costura vive do cursor, que num tablet não existe.
#[test]
fn the_handle_is_painted_every_frame() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ph2d-editor-core/src/screens/hero/paint.rs");
    let s = fs::read_to_string(&root).expect("hero/paint.rs");
    assert!(
        s.contains("paint_dock_reopen"),
        "ninguem pinta a alca: ela existiria como geometria e como alvo de toque, e invisivel — \
         que e' exactamente «chrome vivo e invisivel», a especie que a costura foi desenhada para \
         evitar"
    );
}
