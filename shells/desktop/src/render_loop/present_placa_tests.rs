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
    // GEF 19 E3: a colagem corre DENTRO das faixas (`motion: &mut cola`), cada faixa Motion no rank dela.
    let (placa, cima, vello) = (
        em("faixa::cola(colagem, (placa, motion, palcos), e, cena, (band, sprites));"),
        em("present_bands::draw_upper_bands("),
        em("vello_pass.render_to_intermediate("),
    );
    assert!(chrome.contains("motion: &mut cola,") && present.contains("motion: &mut cola,"));
    // ⭐ doc 121 §9.14 — a camada DESENHA-SE antes dos passes de luz (o halo redesenha-a).
    let em_present = |agulha: &str| present.find(agulha).expect(agulha);
    assert!(
        em_present("motion_shape_placa::desenha_o_quadro(")
            < em_present("super::present_bands::draw_lower_bands("),
        "a camada desenha-se antes das faixas de baixo (uma faixa Motion pode colar lá)"
    );
    assert!(
        em_present("motion_shape_placa::desenha_o_quadro(") < em_present("super::present_fx::run("),
        "as formas desenham-se ANTES do halo que as redesenha"
    );
    assert!(
        placa < cima && cima < vello,
        "as formas vão DEPOIS do mundo (as faixas de cima) e ANTES do chrome (a cena Vello)"
    );
    // ⭐ P4 (b): a camada pousa com a mistura de cada SEGMENTO — a porta da família (`cola_no_mundo` e
    // `faixa::Colagem::cola_a_faixa`, com o gate de texto delas).
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
    assert!(overlays.contains("if do_editor || da_vista || self.motion_shell.palcos.tocam() {"));
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
        em_chrome("faixa::cola(colagem, (placa, motion, palcos), e, cena, (band, sprites));"),
        em_chrome("palcos.pousa(\n            ph2d_app_motion::macro_track::palcos::Colagem {"),
        em_chrome("vello_pass.render_to_intermediate("),
    );
    assert!(
        editor < palcos && palcos < vello,
        "os palcos pousam depois do editor e antes do chrome"
    );
    assert!(
        em_chrome("faixa::cola(")
            < em_chrome("palcos.lentes(surface.gpu(), world_rt, cena, fora);")
    );
    let luz = include_str!("present_luz.rs");
    assert!(luz.contains(".com_oclusoes(&mut [&mut flip_luz, &mut motion_luz], |familias| {"));
}

/// **As macros do Painter em cena** (W4 item 5.5 D): o palco segue o `MacroTempo` que o dreno da
/// timeline escreveu NESTE quadro, antes do extract, e a troca dele vai À FRENTE das pré-visualizações
/// do editor (o extract fica com o primeiro override de cada sprite). *Mutações: chamar antes do dreno
/// (a pintura um quadro atrás do Motion) · `do_editor` à frente (o traço do editor tapa a macro).*
#[test]
fn as_macros_do_painter_trocam_a_sprite_antes_do_extract_e_a_frente_do_editor() {
    let abre = include_str!("fase_frame_open.rs");
    let em = |agulha: &str| abre.find(agulha).expect(agulha);
    assert!(
        em("self.fase_timeline_drain(") < em("self.fase_macros_do_painter(preview_overrides, dt)")
            && em("self.fase_macros_do_painter(preview_overrides, dt)")
                < em("self.fase_sim_extract("),
    );
    let insumos = include_str!("fase_extract_inputs.rs");
    assert!(insumos.contains("            .chain(do_editor)\n"));
}

/// ⭐ **A miniatura da câmera leva os palcos das macros e as formas do Motion fora da placa** (A6, plano 18
/// §«Fora da miniatura»): cada palco decide, codifica, desenha, pousa e aplica a lente com a placa e a camada DA
/// VISTA (trocadas à volta de cada porta), e as formas do editor que a placa não leva vão na cena da vista, colada
/// por cima do mundo dela antes de a troca as devolver. *Mutações: repor um `if !na_vista` nos palcos (somem da
/// miniatura) · apagar uma troca à volta do `decide`/`codifica` (o palco com a câmera do editor) · colar depois da
/// troca de volta (a cena da vista no mundo do EDITOR).*
#[test]
fn a_vista_da_camera_leva_os_palcos_e_as_formas_fora_da_placa() {
    let vista = include_str!("vista_da_camera.rs");
    assert!(vista.contains("gfx.motion.troca_a_vista(&mut self.motion);"));
    assert!(vista.contains("shell.palcos.troca_a_vista(&mut self.palcos);"));
    let em_vista = |agulha: &str| vista.rfind(agulha).expect(agulha);
    assert!(
        em_vista("vista.cola_o_motion(gfx);")
            < em_vista("vista.troca(gfx, &mut self.motion_shell);"),
        "a cena do Motion da vista cola-se no mundo DELA, antes de a troca o devolver ao editor"
    );
    for (nome, codigo, porta) in [
        (
            "fase_vector_bands.rs",
            include_str!("fase_vector_bands.rs"),
            "palcos.decide(gpu, ligadas, cam);",
        ),
        (
            "fase_vector_overlays.rs",
            include_str!("fase_vector_overlays.rs"),
            "palcos.codifica(&mut art, cam, janela, filtro);",
        ),
    ] {
        let troca = "palcos.troca_a_vista(&mut v.palcos);\n";
        let em = codigo
            .find(porta)
            .unwrap_or_else(|| panic!("{nome}: {porta}"));
        let antes = codigo[..em].rfind(troca).expect("a troca de ida");
        let depois = em + codigo[em..].find(troca).expect("a troca de volta");
        assert!(
            !codigo[antes + troca.len()..em].contains(';')
                && !codigo[em + porta.len()..depois].contains(';'),
            "{nome}: a porta dos palcos corre ENTRE as duas trocas"
        );
    }
    let overlays = include_str!("fase_vector_overlays.rs");
    assert!(overlays.contains("let cena = v.motion.alvo_da_cena("));
    for (nome, codigo) in [
        ("present.rs", include_str!("present.rs")),
        ("present_chrome.rs", include_str!("present_chrome.rs")),
    ] {
        // Cada porta dos palcos no corpo da função: entre o `fn` e ela, uma chaveta aberta só (sem `if` à volta).
        for porta in ["palcos.desenha(", "palcos.pousa(", "palcos.lentes("] {
            let Some(em) = codigo.find(porta) else {
                continue;
            };
            let corpo = &codigo[codigo[..em].rfind(" fn ").expect("a função")..em];
            let abertas = corpo.matches('{').count() - corpo.matches('}').count();
            assert_eq!(abertas, 1, "{nome}: `{porta}` sob uma guarda");
        }
        assert!(
            !codigo.contains("if na_vista {") && !codigo.contains("if self.na_vista_da_camera {"),
            "{nome}: um retorno cedo na vista da câmera salta os palcos"
        );
    }
}

/// ⭐ GEF 19 E3 — **uma faixa Motion ABAIXO de uma faixa de sprites cola ANTES dela**: com a placa, o
/// objecto Motion acima da imagem na Hierarchy cai no passe de baixo (antes da última faixa de
/// sprites, que corre o pipeline inteiro); sem a placa, por cima de tudo. *Mutação: tirar o
/// `(g.motion)` do braço Motion do passe de baixo (a faixa não cola em sítio nenhum).*
#[test]
fn uma_faixa_motion_abaixo_dos_sprites_cola_antes_deles() {
    use ph2d_render::draw_bands::{Family, FrameOrder};
    let mut o = FrameOrder::default();
    o.record_motion(0, 7);
    o.record(1, None);
    o.motion_na_placa = true;
    let plan = super::plan_frame(&o);
    let ultima = plan.last_sprite.expect("faixa de sprites");
    let m = plan.bands.iter().position(|b| b.family == Family::Motion);
    assert!(m.is_some_and(|m| m < ultima), "no passe de BAIXO");
    assert!(plan.banded);
    o.motion_na_placa = false;
    let plan = super::plan_frame(&o);
    let m = plan.bands.iter().position(|b| b.family == Family::Motion);
    assert!(m > plan.last_sprite, "sem placa: por cima");
    // A faixa do TOPO abaixo da última de sprites leva a camada filtrada e a lente no sítio dela.
    let present = include_str!("present.rs");
    assert!(
        present
            .contains("faixa::cola(colagem, (placa, motion, palcos), e, cena, (band, sprites));")
    );
    let src = include_str!("present_bands.rs");
    let em_src = |agulha: &str| src.find(agulha).expect(agulha);
    assert!(
        em_src("(g.motion)(\n            None,")
            > em_src("for band in plan.bands.iter().skip(from)"),
        "o topo sem faixa cola DEPOIS das faixas de cima"
    );
    assert!(src.contains("passe_de_baixo(&plan.bands[..plan.main.0])"));
}
