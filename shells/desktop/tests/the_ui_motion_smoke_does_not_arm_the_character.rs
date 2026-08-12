//! **Arch-gate: o smoke da UI viva não arma o carácter por baixo da mesa.**
//!
//! ## A cicatriz que este gate herda
//!
//! Em 2026-07-22 o Enio reportou *«quando abro o painter o Wet paint ainda é o que aparece
//! primeiro»* — e o `default()` do tool estava certo: quem abria no meio errado eram os **smokes**,
//! que chamavam `set_paint_media(<o seu meio>)` no prólogo. O doc do `impasto_smoke` já pregava a
//! lição (*«nothing here is armed in code … the smoke that arms state under the table skips exactly
//! the seam it was supposed to prove»*) e o código contradizia-a.
//!
//! Aqui o risco é maior, não menor: o smoke da UI viva existe **para julgar o carácter**, e um
//! `set_character(Expressive)` no prólogo tornaria os passos 2-4 do roteiro uma demonstração de si
//! mesma — o artista veria a corda a pendurar sem nunca ter passado pelo menu, pela porta única ou
//! pela persistência. O gate lê a fonte e recusa as duas chamadas.
//!
//! ⚠️ **Gate de TEXTO porque o alvo é código atrás de uma env var**: nenhum teste de unidade entra
//! no `ui_motion_smoke` (ele sai cedo sem `gfx`, que precisa de janela + GPU) — a mesma razão do
//! `the_smokes_open_the_painter_in_digital`, de que este é irmão.

const SMOKE: &str = include_str!("../src/ui_motion_smoke.rs");

/// Nenhuma das duas portas do carácter é chamada pelo smoke.
///
/// **Mutação que deve sangrar:** pôr `hero.motion.set_character(UiCharacter::Expressive)` no
/// `ui_motion_smoke` para «poupar um clique ao artista» — que é exactamente como o smoke do Painter
/// passou a abrir no meio errado.
#[test]
fn the_smoke_makes_the_artist_choose_the_character_in_the_menu() {
    for call in ["set_character(", "set_reduced_motion("] {
        assert!(
            !SMOKE.contains(call),
            "o smoke da UI viva chama `{call}` — ele passaria a demonstrar o carácter em vez de o \
             fazer ESCOLHER, e saltaria de uma vez o menu, a porta única e a persistência, que são \
             as três costuras que ele existe para provar. O roteiro manda o artista ao \
             `Settings > Motion`."
        );
    }
}

/// O card do smoke é ARRASTADO pelas portas reais, e nunca aberto já deslocado.
///
/// ⚠️ A âncora **é** o sítio onde o card nasceu — abri-lo longe dela pediria uma segunda porta que
/// só o smoke usaria, e a corda passaria a desenhar uma relação que o produto nunca produz.
///
/// **Mutação que deve sangrar:** trocar o par `open_fill_modal(âncora)` + `move_fill_modal(delta)`
/// por um único `open_fill_modal` já na posição final.
#[test]
fn the_card_is_opened_at_its_anchor_and_then_moved_through_the_real_door() {
    assert!(
        SMOKE.contains("open_fill_modal("),
        "o smoke não abre o card — a cena 2 não tem sujeito"
    );
    assert!(
        SMOKE.contains("move_fill_modal("),
        "o smoke abre o card e não o move: a âncora e a posição ficariam no mesmo ponto, a corda \
         teria comprimento zero e não se desenharia nada — a cena julgaria o vazio"
    );
}

/// ⚠️ **CONTROLE POSITIVO.** Sem ele, um ficheiro renomeado deixaria os dois gates acima a varrer o
/// vazio e a passar por vácuo — a falha silenciosa que o `keyboard.rs` partido já produziu.
#[test]
fn the_scanned_file_is_the_real_one() {
    assert!(
        SMOKE.contains("fn ui_motion_smoke"),
        "o `ui_motion_smoke.rs` mudou de dono: os gates acima deixaram de afirmar o que dizem"
    );
    assert!(
        SMOKE.contains("[ui-motion-smoke"),
        "o smoke deixou de imprimir a sua linha — e o roteiro do artista começa por *se a linha \
         não aparecer, PARE*"
    );
}
