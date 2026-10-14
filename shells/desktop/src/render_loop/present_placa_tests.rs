//! ⭐⭐⭐ **AS FORMAS NA PLACA chegam ao quadro, e ENTRE o mundo e o chrome** (doc 121 do Motion,
//! W2) — o elo da shell. ⚠️ `include_str!` porque o presente pede uma janela e uma placa: a LEI
//! (`motion_shape_placa`, com os gates de decisão e de paridade de pixel) e *este elo* são as duas
//! metades, e cada uma reprova por um motivo diferente.
//!
//! ⭐ Desde a W4 das Macros (item 5.5) o elo é de N FONTES: o editor e cada palco de uma macro em cena
//! (`ph2d_app_motion::macro_track::palcos`) passam pelas MESMAS portas em cada sítio.
//!
//! *Mutações: apagar o `|=` (a camada é colada num acumulador que o compositor não lê) · não
//! forçar as faixas do documento (o documento fica na cena Vello, POR CIMA das formas) · apagar o
//! `!…ativa()` (as formas desenhadas DUAS vezes) · mover a colagem para depois do chrome (as formas
//! por cima dos gizmos) · desenhar a camada DEPOIS do halo (doc 121 §9.14: o halo redesenha as
//! formas DESTE quadro, e leria as do anterior) · esquecer um palco num dos sítios.*
#[test]
fn as_formas_na_placa_entram_entre_o_mundo_e_o_chrome() {
    let present = include_str!("present.rs");
    assert!(present.contains("plan.banded |= self.motion_shell.placa.ativa();"));
    let bandas = include_str!("fase_vector_bands.rs");
    assert!(bandas.contains("doc_bands_of(frame_order, placa || palcos || luz_acesa)"));
    let overlays = include_str!("fase_vector_overlays.rs");
    assert!(
        overlays
            .contains("let do_editor = motion_tool_active && !self.motion_shell.placa.ativa();")
    );
    let chrome = include_str!("present_chrome.rs");
    let em = |agulha: &str| chrome.find(agulha).expect(agulha);
    let (cima, placa, vello) = (
        em("present_bands::draw_upper_bands("),
        em("let placa = &self.motion_shell.placa;"),
        em("vello_pass.render_to_intermediate("),
    );
    // ⭐ doc 121 §9.14 — a camada DESENHA-SE antes dos passes de luz (o halo redesenha-a).
    let em_present = |agulha: &str| present.find(agulha).expect(agulha);
    assert!(
        em_present("motion_shape_placa::desenha_o_quadro(") < em_present("super::present_fx::run("),
        "as formas desenham-se ANTES do halo que as redesenha"
    );
    assert!(
        cima < placa && placa < vello,
        "as formas vão DEPOIS do mundo (as faixas de cima) e ANTES do chrome (a cena Vello)"
    );
    // ⭐ P4 (b): a camada pousa no mundo com a mistura de cada SEGMENTO (`cola_no_mundo`, a porta da
    // família: a `BandSource::Formas` e o `blit_pousa` estão no gate de texto dela).
    assert!(chrome.contains("motion_shape_placa::cola_no_mundo(placa, gpu, band_blit, alvo);"));
}

/// ⭐⭐⭐ **A rota do DISPOSITIVO chega ao quadro** (doc 121 W3 · P4): a decisão e o desenho são as
/// portas da família (`motion_shape_placa::{decide_o_quadro, desenha_o_quadro}`, com o gate de
/// texto delas a prender o `gpu_live` das duas pontas); a shell só as compõe — e o desenho recebe o
/// renderer (as texturas das imagens de um grupo) e o filtro do projecto.
#[test]
fn a_rota_do_dispositivo_chega_ao_quadro() {
    let bandas = include_str!("fase_vector_bands.rs");
    assert!(bandas.contains("ph2d_app_motion::motion_shape_placa::decide_o_quadro("));
    let present = include_str!("present.rs");
    assert!(present.contains("motion_shape_placa::desenha_o_quadro("));
    assert!(present.contains("placa, gpu, tamanho, motion, renderer, filtro,"));
    assert!(present.contains("hero_screen.as_ref().map(|h| h.project.image_filter)"));
}

/// ⭐ doc 121 §9.14 (c) — **o halo, uma vez por FONTE**: o passe (e o gate de texto dele) vive em
/// `ph2d_app_motion::motion_glow_passe`; a shell chama-o para o editor e para cada palco, com a placa
/// de cada um. *Mutação: apagar o laço (a macro em cena deixa de brilhar).*
#[test]
fn o_halo_corre_para_o_editor_e_para_cada_palco() {
    let fx = include_str!("present_fx.rs");
    assert!(fx.contains("motion_glow_passe::passe(gpu, &mut brilho, motion, formas, ativo);"));
    assert!(fx.contains("for (palco, placa) in palcos.fontes() {"));
    assert!(fx.contains("motion_glow_passe::passe(gpu, &mut brilho, palco, placa, true);"));
}

/// ⭐ **AS MACROS EM CENA** (ADR-0186, W4 item 5.5): os palcos seguem o relógio DEPOIS de o mundo ser
/// publicado e ANTES de o editor cozer (que larga os donos deles), e passam por cada sítio do quadro
/// — os assados, a decisão da placa, as formas da CPU, o planeamento, o desenho, a colagem por cima
/// da do editor e antes do chrome, a lente e a luz. ⛔ Excluir o dono sem desenhar o palco faria o
/// objecto SUMIR no Play: as duas metades estão no mesmo gate.
#[test]
fn os_palcos_das_macros_passam_por_cada_sitio_do_quadro() {
    let ponte = include_str!("fase_motion_bridge.rs");
    let em = |agulha: &str| ponte.find(agulha).expect(agulha);
    assert!(
        em("motion_mundo::publica_o_mundo(") < em("palcos.segue(")
            && em("palcos.segue(") < em("motion_bridge::dispatch("),
        "o palco segue com o mundo deste quadro, antes de o editor cozer"
    );
    assert!(ponte.contains("motion_objeto::tocados_por_macro(motion, tocados);"));
    let recook = include_str!("fase_vector_fx_recook.rs");
    assert!(recook.contains("for (palco, _) in self.motion_shell.palcos.fontes_mut() {"));
    let bandas = include_str!("fase_vector_bands.rs");
    assert!(bandas.contains(".decide(surface.gpu(), ligadas, cam_affine);"));
    let overlays = include_str!("fase_vector_overlays.rs");
    assert!(overlays.contains("if do_editor || self.motion_shell.palcos.tocam() {"));
    assert!(overlays.contains("palcos.codifica(&mut art, cam_affine, Some(janela), filtro);"));
    let present = include_str!("present.rs");
    assert!(present.contains("let palcos = self.motion_shell.palcos.liga();"));
    assert!(present.contains("plan.banded |= lente || filtrada || palcos;"));
    let em_present = |agulha: &str| present.find(agulha).expect(agulha);
    assert!(
        em_present("palcos.desenha(gpu, tamanho, renderer, filtro);")
            < em_present("super::present_fx::run("),
        "a camada de cada palco desenha-se antes do halo que a redesenha"
    );
    assert!(present.contains("palcos: &self.motion_shell.palcos,"));
    let chrome = include_str!("present_chrome.rs");
    let em_chrome = |agulha: &str| chrome.find(agulha).expect(agulha);
    let (editor, palcos, vello) = (
        em_chrome("motion_shape_placa::cola_no_mundo(placa, gpu, band_blit, alvo);"),
        em_chrome("palcos.pousa(ph2d_app_motion::macro_track::palcos::Colagem {"),
        em_chrome("vello_pass.render_to_intermediate("),
    );
    assert!(
        editor < palcos && palcos < vello,
        "os palcos pousam depois do editor e antes do chrome"
    );
    assert!(
        em_chrome("motion.lente.aplica(")
            < em_chrome("palcos.lentes(surface.gpu(), world_rt, cena);")
    );
    let luz = include_str!("present_luz.rs");
    assert!(luz.contains(".com_oclusoes(&mut [&mut flip_luz, &mut motion_luz], |familias| {"));
}
