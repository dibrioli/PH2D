//! ⭐⭐⭐ **A FIAÇÃO do LOD da forma** (report do Enio, 2026-09-21) — o gate que impede a 5.ª
//! ocorrência da lei desta casa: *um motor com a lei certa e a shell a não a ligar lê-se como um
//! motor sem a lei*.
//!
//! A lei vive em `ph2d_app_motion::motion_shape_lod` e tem **13 gates + 9 provas de mutação**.
//! Nenhum deles vê a SHELL: apagar as três chamadas da fase deixa-os todos verdes e o produto
//! exactamente como estava antes da cura. ⚠️ E a fase não é alcançável de um teste (ela pede
//! `surface`/`renderer`/`gpu` de um quadro real), logo o gate é de **TEXTO** — a mesma forma que
//! esta casa já usa para as costuras que só o quadro percorre.

const FASE: &str = include_str!("../../src/render_loop/fase_vector_fx_recook.rs");

/// **As três metades têm de estar LIGADAS, e as três fazem coisas diferentes:** escolher quem
/// (`geometrias_para_lod`), mover (`aplica_lod_de_forma`) e manter o despejo honesto
/// (`vivas_com_o_lod`).
///
/// ⚠️ **A terceira é a que ninguém se lembraria de repor.** Sem ela o despejo larga a tile que os
/// quads ainda amostram — e com a cena PARADA o cozimento devolve cedo, logo isso é permanente:
/// a forma desaparece ou pisca. *Um defeito que só aparece com a cena quieta é o que menos
/// hipóteses tem de ser apanhado por quem está a mexer.*
#[test]
fn o_lod_da_forma_esta_ligado_a_fase() {
    for (chamada, porque) in [
        ("motion_shape_lod::geometrias_para_lod(", "quem vira tile"),
        ("motion_shape_lod::aplica_lod_de_forma(", "a partição"),
        ("motion_shape_lod::vivas_com_o_lod(", "o despejo honesto"),
    ] {
        assert!(
            FASE.contains(chamada),
            "a fase perdeu `{chamada}` ({porque}) — a lei fica viva e o produto volta ao de antes"
        );
    }
}

/// **E o assador tem de ouvir os DOIS consumidores.** A guarda era `glows` sozinha; se ela voltar
/// a sê-lo, o LOD escolhe geometrias e **nenhuma tem tile**, logo a partição não move nada — e o
/// modo de falha é MUDO, porque a cerca `sem tile fica crisp` está desenhada para não apagar a
/// forma.
///
/// ⚠️ O CONTROLO está na segunda asserção: o `glows` tem de continuar lá. Um gate que só exigisse
/// o `quer` passaria numa redacção que tivesse trocado um consumidor pelo outro, e o glow das
/// formas paramétricas morria em silêncio.
#[test]
fn o_assador_de_tiles_ouve_os_dois_consumidores() {
    assert!(
        FASE.contains("glows || quer.contains(gid)"),
        "a guarda do assador tem de ouvir o LOD além do glow"
    );
    assert!(
        FASE.contains("let glows ="),
        "controlo: o glow continua a ser um consumidor — ele não foi substituído"
    );
}

/// ⭐⭐⭐ **O COZIMENTO corre ANTES da PARTIÇÃO, e trocar a ordem apaga a cura EM SILÊNCIO.**
///
/// O `fase_motion_bridge` chama `advance_or_scrub_scoped`, que **limpa** `instances` e
/// `vector_instances` antes de as encher. Se ele corresse DEPOIS do `fase_vector_fx_recook`, os
/// quads que a partição acabou de empurrar eram deitados fora no mesmo quadro — e o produto ficava
/// exactamente como antes da cura, com os 13 gates da lei e os 3 da fiação **todos verdes**.
///
/// ⚠️⚠️ **A régua é o texto EMENDADO do quadro e não o número de linha de um ficheiro** — a lei
/// desta casa, escrita depois de o `fase_hero_frame.rs` ter desmentido duas premissas de quem o
/// leu de cima para baixo: *um ficheiro com três fases não as corre pela ordem em que as declara*.
#[test]
fn o_cozimento_corre_antes_da_particao() {
    let t = crate::frame_text::render_frame();
    let cozer = t
        .find("fase_motion_bridge(")
        .expect("o quadro tem de cozinhar o Motion");
    let particao = t
        .find("fase_vector_fx_recook(")
        .expect("o quadro tem de assar e particionar as tiles de forma");
    assert!(
        cozer < particao,
        "o cozimento LIMPA as duas listas: correndo depois da partição, ele apaga os quads dela \
         (cozer @ {cozer}, partição @ {particao})"
    );
}
