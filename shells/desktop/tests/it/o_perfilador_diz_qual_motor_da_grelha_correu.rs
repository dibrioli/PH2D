//! ⭐⭐ **A LINHA DO PERFILADOR IDENTIFICA O MOTOR DA GRELHA** — doc 115 §29.
//!
//! ⚠️⚠️ **Ela nasceu de um report do dono** (*«motor anterior mais rápido»*, 2026-09-19): com o
//! corte da grelha a não armar, as duas rotas de um A/B imprimem `0 grande(s)` e a linha lê-se
//! **igual dos dois lados**. *Um instrumento de bissecção que não se identifica não bissecta nada*,
//! e eu recebi uma linha sem saber qual das duas corridas a produziu.
//!
//! ⛔ **É um gate de TEXTO porque o alvo não é alcançável de um teste:** a linha sai de uma fase do
//! quadro que pede um `GpuContext` e uma janela viva. O que ele afirma é a FIAÇÃO — que o número da
//! bissecção chega da corrente ao contador e do contador à linha —, com piso de população nas três
//! pontas. *Um motor com a lei certa e a shell a não a ligar lê-se como um motor sem a lei.*

const RELATORIO: &str = include_str!("../../src/render_loop/fase_frame_profile_report.rs");
const QUADRO: &str = include_str!("../../src/render_loop/fase_hero_frame.rs");
const CONTADORES: &str = include_str!("../../src/render_loop/frame_prof.rs");

#[test]
fn o_perfilador_diz_qual_motor_da_grelha_correu() {
    assert!(
        CONTADORES.contains("FRAME_PROF_BISSECCAO"),
        "o contador da bissecção desapareceu"
    );
    assert!(
        QUADRO.contains("FRAME_PROF_BISSECCAO.with(|c| c.set(u64::from(r.bisseccao)))"),
        "a corrente deixou de encher o contador da bissecção"
    );
    assert!(
        RELATORIO.contains("FRAME_PROF_BISSECCAO.with(std::cell::Cell::get)"),
        "o relatório deixou de LER o contador da bissecção"
    );
    // ⚠️ O marcador vive DENTRO da linha de formato, e não num `&str` à parte: ali ele seria texto
    // com cara de língua no fonte da shell, e o censo do HR-15 manda-o para uma isenção nomeada.
    // *Um número dentro do formato não precisa de isenção nenhuma.*
    assert!(
        RELATORIO.contains("uma-camada-por-ordem={bisseccao}"),
        "a linha do perfilador deixou de NOMEAR o motor da grelha"
    );
}
