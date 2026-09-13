//! ⭐⭐ **QUANTO DO BUFFER FIXO DO VELLO O EDITOR GASTA NUM QUADRO** — a reserva que o orçamento
//! da pele de imagem do esqueleto tem de deixar.
//!
//! # Porque existe (esqueleto, F6-d, 2026-09-13)
//!
//! O Vello guarda a informação de TODO desenho de um quadro num buffer de tamanho FIXO
//! (`bin_data = 1 << 18` palavras, `vello_encoding::BufferSizes::new`), e passar dele dá a volta
//! a um `u32` lá dentro: pânico em debug, **quadro em branco** em release — painéis incluídos.
//! Uma peça da pele de imagem custa **11** palavras (`ph2d-vector::atlas_probe_pieces_tests`), e
//! a sonda de GPU (`ph2d-render::skin_pieces_gpu_cost`) viu o quadro ficar em branco entre
//! `17 496` e `21 600` peças **numa cena só com a pele**.
//!
//! ⇒ o orçamento da pele é o que o QUADRO deixa. Esta sonda mede a outra metade: o chrome do
//! editor, pintado sem ecrã pelo registo real de painéis.
//!
//! ⚠️ **É um PISO do quadro real, nunca um tecto:** os painéis pintam o estado de omissão (sem
//! selecção, sem documento), e a arte do canvas não passa pelo `paint_hero_screen`.
//!
//! Rodar: `cargo test -p ph2d-editor-core --test it -- --ignored --nocapture vello_bin_budget`

use ph2d_editor_core::panel::{PanelHostInternal, with_registry_ref};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, paint_hero_screen};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// O buffer fixo do Vello, em palavras.
const BIN_DATA: u32 = 1 << 18;

/// `(palavras, caminhos)` do TERCEIRO quadro — o primeiro faz layout e caches.
fn um_quadro(todos_os_paineis: bool, largura: f32, altura: f32) -> (u32, u32) {
    ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    if todos_os_paineis {
        let ids: Vec<&'static str> =
            with_registry_ref(|reg| reg.panels().iter().map(|p| p.manifest.id).collect());
        for id in ids {
            <HeroScreen as PanelHostInternal>::set_panel_visible(&mut hero, id, true);
        }
    }
    let mut texto = TextSystem::without_system_fonts();
    let janela = Rect::new(0.0, 0.0, largura, altura);
    let mut ultimo = (0, 0);
    for _ in 0..3 {
        let mut cena = VectorScene::new();
        paint_hero_screen(&mut hero, janela, &mut cena, &mut texto);
        ultimo = (cena.probe_bin_info_words(), cena.inner().encoding().n_paths);
    }
    ultimo
}

#[test]
#[ignore = "sonda: imprime quanto do buffer fixo do Vello o chrome gasta, nao afirma"]
fn measure_the_vello_bin_info_words_of_an_editor_frame() {
    println!(
        "\n{:<34} {:>9} {:>11} {:>9}",
        "quadro", "palavras", "% de 1<<18", "caminhos"
    );
    for (nome, todos, largura, altura) in [
        ("paineis de omissao, 1920x1080", false, 1920.0, 1080.0),
        ("paineis de omissao, 2560x1440", false, 2560.0, 1440.0),
        ("TODOS os paineis, 1920x1080", true, 1920.0, 1080.0),
        ("TODOS os paineis, 2560x1440", true, 2560.0, 1440.0),
    ] {
        let (palavras, caminhos) = um_quadro(todos, largura, altura);
        println!(
            "{:<34} {:>9} {:>10.1}% {:>9}",
            nome,
            palavras,
            100.0 * f64::from(palavras) / f64::from(BIN_DATA),
            caminhos
        );
    }
    println!("(uma peca da pele de imagem = 11 palavras)\n");
}
