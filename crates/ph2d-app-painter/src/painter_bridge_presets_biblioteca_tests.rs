//! **A biblioteca do navegador** sobre o disco (23/10): a ordem estável e a do artista, o Delete que nunca
//! toca a fábrica, e o Export → Import que devolve o MESMO pincel (campos ao bit, imagens ao byte) e a
//! MESMA pintura.

use super::tests::{abre, pasta, sem_erros};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PaintMedia;

const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";
const FLAT: &str = "WetPainter/Watercolor/Fluid/Wet Flat.ph2d-brush.png";
const AGUARELA: &str = "Watercolor/Watercolor/Basic/Watercolor Basic.ph2d-brush.png";
const LAPIS: &str = "Digital/Basic/Hard/Hard Round.ph2d-brush.png";

fn ordem(p: &Presets) -> Vec<String> {
    p.vista
        .entradas
        .iter()
        .map(|v| v.relativo.clone())
        .collect()
}

/// ⭐ **A mesma biblioteca dá sempre a mesma ordem** (report do dono 23/10: «a ordem muda sozinha»). Medido
/// antes da cura: escolher o «Watercolor Basic» (meio Aquarela) punha-o no TOPO da categoria Watercolor e
/// escolher o «Wet Round» (Wet Paint) mandava-o para o FIM — o «meio de agora primeiro» da vista. Nenhum gesto
/// que não seja arrastar muda a ordem: trocar de meio, escolher, favoritar e desfavoritar, reabrir; e um
/// «Save as New» só acrescenta, ao lado do de onde veio.
#[test]
fn a_ordem_nao_muda_sozinha() {
    let (mut p, mut t, mut toasts, uti) = abre("ordem_estavel");
    let antes = ordem(&p);
    let i_aguarela = antes.iter().position(|r| r == AGUARELA).unwrap();
    for rel in [AGUARELA, ROUND, LAPIS, WASH] {
        p.atender(&mut t, PedidoPreset::Escolher(rel.into()), &mut toasts);
        assert_eq!(ordem(&p), antes, "depois de escolher {rel}");
    }
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert_eq!(ordem(&p), antes, "favoritar e desfavoritar");
    t.set_paint_media(PaintMedia::Digital);
    drive_em(&mut p, &mut t, &mut toasts);
    assert_eq!(ordem(&p), antes, "trocar de meio pelo chip");
    let mut t2 = PainterTool::default();
    let p2 = Presets::abrir_em(pasta_de_fabrica(), Some(uti), &mut t2, &mut toasts);
    assert_eq!(ordem(&p2), antes, "reabrir");
    assert_eq!(
        antes[i_aguarela], AGUARELA,
        "controlo: a fixtura tem dois meios na mesma categoria"
    );

    p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let novo = "WetPainter/Watercolor/Fluid/Wet Wash 2.ph2d-brush.png";
    let depois = ordem(&p);
    let sem_o_novo: Vec<String> = depois.iter().filter(|r| *r != novo).cloned().collect();
    assert_eq!(sem_o_novo, antes, "o Save as New só acrescenta");
    let i = depois.iter().position(|r| r == novo).unwrap();
    assert_eq!(depois[i - 1], WASH, "e nasce logo depois do de onde veio");
}

/// A parte do `drive` que reage a uma troca de gaveta (ferramenta × meio) (sem a célula da thread).
fn drive_em(p: &mut Presets, t: &mut PainterTool, toasts: &mut ToastQueue) {
    if t.gaveta_de_presets() != p.gaveta {
        p.abrir_meio(t, toasts);
        p.gaveta = t.gaveta_de_presets();
        p.publicar();
    }
}

/// ⭐ **A ordem do artista** — arrastar muda-a, ela sobrevive a reabrir e a um pincel novo (que entra ao
/// lado do vizinho), o «Reset positions» devolve a por omissão; fora da secção o arrasto não faz nada.
#[test]
fn a_ordem_manual_persiste_e_o_reset_a_devolve() {
    let (mut p, mut t, mut toasts, uti) = abre("ordem_manual");
    let padrao = ordem(&p);
    assert!(!p.vista.ordem_manual);
    let mover = |p: &mut Presets,
                 t: &mut PainterTool,
                 toasts: &mut ToastQueue,
                 a: &str,
                 b: &str,
                 depois| {
        p.atender(
            t,
            PedidoPreset::Mover {
                arrastado: a.into(),
                alvo: b.into(),
                depois,
            },
            toasts,
        );
    };
    mover(&mut p, &mut t, &mut toasts, FLAT, AGUARELA, false);
    let manual = ordem(&p);
    assert_ne!(manual, padrao, "o arrasto mudou a ordem");
    let i = manual.iter().position(|r| r == FLAT).unwrap();
    assert_eq!(
        manual[i + 1],
        AGUARELA,
        "o Flat ficou antes do Watercolor Basic"
    );
    assert!(p.vista.ordem_manual);

    mover(&mut p, &mut t, &mut toasts, LAPIS, ROUND, false);
    assert_eq!(ordem(&p), manual, "outra categoria: nada muda");
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    mover(&mut p, &mut t, &mut toasts, WASH, ROUND, false);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert_eq!(
        ordem(&p),
        manual,
        "um favorito e um da categoria: nada muda"
    );

    let mut t2 = PainterTool::default();
    let mut p2 = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t2, &mut toasts);
    assert_eq!(ordem(&p2), manual, "reaberto, a ordem do artista");

    p2.atender(&mut t2, PedidoPreset::Escolher(FLAT.into()), &mut toasts);
    p2.atender(&mut t2, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let novo = "WetPainter/Watercolor/Fluid/Wet Flat 2.ph2d-brush.png";
    let com_novo = ordem(&p2);
    assert_eq!(
        com_novo.len(),
        manual.len() + 1,
        "sem buracos nem duplicados"
    );
    let i = com_novo.iter().position(|r| r == novo).unwrap();
    assert_eq!(
        com_novo[i - 1],
        FLAT,
        "o novo ao lado do vizinho, na ordem do artista"
    );

    p2.atender(&mut t2, PedidoPreset::ReporPosicoes, &mut toasts);
    let reposta = ordem(&p2);
    let sem_o_novo: Vec<String> = reposta.iter().filter(|r| *r != novo).cloned().collect();
    assert_eq!(sem_o_novo, padrao, "Reset positions = a ordem por omissão");
    assert!(!p2.vista.ordem_manual);
    let p3 = Presets::abrir_em(
        pasta_de_fabrica(),
        Some(uti),
        &mut PainterTool::default(),
        &mut toasts,
    );
    assert_eq!(ordem(&p3), reposta, "e o reset também persiste");
}

/// ⭐ **Apagar só apaga o do artista**: sobre um de fábrica (ou um sobreposto) nada sai do disco nem da
/// lista; sobre um do artista o ficheiro sai, o favorito e a posição dele também, e o pincel continua na mão.
#[test]
fn apagar_nunca_toca_a_fabrica_e_o_pincel_fica_na_mao() {
    let (mut p, mut t, mut toasts, uti) = abre("apagar");
    let n = p.vista.entradas.len();
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::Apagar, &mut toasts);
    assert_eq!(p.vista.entradas.len(), n, "um de fábrica não se apaga");
    assert!(pasta_de_fabrica().join(ROUND).is_file());
    p.atender(&mut t, PedidoPreset::GuardarComoPadrao, &mut toasts);
    p.atender(&mut t, PedidoPreset::Apagar, &mut toasts);
    assert!(
        uti.join(ROUND).is_file(),
        "o sobreposto sai pelo Reset, nunca pelo Delete"
    );

    t.set_brush_size_px(7.0);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    let novo = "WetPainter/Watercolor/Fluid/Wet Round 2.ph2d-brush.png";
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    p.atender(
        &mut t,
        PedidoPreset::Mover {
            arrastado: novo.into(),
            alvo: ROUND.into(),
            depois: false,
        },
        &mut toasts,
    );
    assert!(p.vista.procurar(novo).is_some_and(|v| v.apagavel));
    assert!(!p.vista.procurar(ROUND).unwrap().apagavel);
    let pincel = t.brush_settings();
    p.atender(&mut t, PedidoPreset::Apagar, &mut toasts);
    sem_erros(&toasts);
    assert!(!uti.join(novo).exists(), "o ficheiro do artista saiu");
    assert!(p.vista.procurar(novo).is_none());
    assert_eq!(p.vista.entradas.len(), n);
    assert!(!p.estado.favoritos.contains(novo), "o favorito dele também");
    assert!(!p.estado.ordem.iter().any(|r| r == novo), "e a posição");
    assert!(!p.estado.ativo_por_meio.values().any(|r| r == novo));
    assert_eq!(p.vista.ativo, None);
    assert_eq!(t.brush_settings(), pincel, "o pincel continua na mão");
    let relido = ph2d_brush_presets::EstadoDoUtilizador::ler(&uti).unwrap();
    assert!(!relido.favoritos.contains(novo), "e no estado gravado");
}

fn img(semente: u8) -> Vec<u8> {
    (0..24 * 24u32)
        .map(|i| ((i * 7 + u32::from(semente) * 31) % 251) as u8)
        .collect()
}

fn rgba(semente: u8) -> Vec<ph2d_tool_painter::SrgbRgba> {
    let px: Vec<u8> = img(semente)
        .into_iter()
        .flat_map(|v| [v, v / 2, 255 - v, 255])
        .collect();
    ph2d_tool_painter::SrgbRgba::iter_byte_slice(&px).collect()
}

/// Uma imagem em CADA slot de imagem do pincel da Aquarela (o censo de 23/10): Shape, Grain, a 2.ª ponta,
/// a mangueira (3 imagens) e o Flow da Ragged Edge. O Paper é da obra e fica de fora por lei.
fn pincel_com_todas_as_imagens(t: &mut PainterTool) {
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_tool_painter::{LINHAS_DA_PONTA, LinhaDaPonta, ids};
    t.set_brush_shape_image(img(1), 24, 24);
    t.set_brush_texture_image(img(2), 24, 24);
    let dual = LINHAS_DA_PONTA
        .iter()
        .position(|x| *x == LinhaDaPonta::DualMode)
        .unwrap();
    t.handle_panel_event(PanelEvent::SelectOption(
        ids::PAINTER_PONTA_FIELDS[dual],
        "4".into(),
    ));
    t.set_brush_dual_image(img(3), 24, 24);
    for k in 0..3 {
        t.add_brush_shape_hose_image_rgba(&rgba(4 + k), 24, 24);
    }
    t.use_layers_as_flow(img(9), 24, 24);
}

fn tela(m: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
    t.set_paint_media(m);
    t.set_brush_color_srgb8([40, 110, 220]);
    t.set_wet_relogio_fixo(true);
    t
}

/// Um traço ondulado e a tela depois de a água assentar.
fn traco(t: &mut PainterTool) -> Vec<u8> {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut ultima = Vec::new();
    let mut tique = |t: &mut PainterTool| {
        t.on_tick(1000.0 / 60.0);
        if let Some((px, _, _)) = t.take_preview_arc() {
            ultima = px.as_ref().clone();
        }
    };
    t.on_canvas_pointer(p([30.0, 60.0], PointerPhase::Down));
    let mut pos = [30.0, 60.0];
    for k in 1..=40 {
        let f = k as f32 / 40.0;
        pos = [
            30.0 + 140.0 * f,
            60.0 + 18.0 * (f * std::f32::consts::TAU).sin(),
        ];
        t.on_canvas_pointer(p(pos, PointerPhase::Move));
        tique(t);
    }
    t.on_canvas_pointer(p(pos, PointerPhase::Up));
    for _ in 0..20 {
        tique(t);
    }
    ultima
}

/// ⭐⭐ **Exportar → importar numa tela nova dá o MESMO pincel e a MESMA pintura.** O pincel leva uma imagem
/// em cada slot; o ficheiro exportado entra noutra pasta de artista, cai na MESMA categoria (a pasta do meio
/// viaja no ficheiro), e o pincel aplicado é o de origem: os campos ao bit, cada imagem ao byte, o traço ao
/// píxel. Controlo: o traço com o pincel de fábrica difere (as imagens pintam).
#[test]
fn exportar_e_importar_da_o_mesmo_pincel_e_a_mesma_pintura() {
    let (mut p, mut t, mut toasts, _) = abre("exportar");
    p.atender(&mut t, PedidoPreset::Escolher(AGUARELA.into()), &mut toasts);
    assert_eq!(t.paint_media(), PaintMedia::Watercolor);
    pincel_com_todas_as_imagens(&mut t);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let rel = "Watercolor/Watercolor/Basic/Watercolor Basic 2.ph2d-brush.png";
    assert_eq!(p.vista.ativo.as_deref(), Some(rel));
    let base = p.bib.procurar(rel).unwrap().preset.parametros.clone();
    let (prm_a, tex_a) = t.preset_capturar(&base);
    for slot in ["shape", "grain", "flow", "dual_shape"] {
        assert!(
            tex_a.contains_key(slot),
            "o slot {slot} está no ficheiro: {:?}",
            tex_a.keys()
        );
    }
    assert!(
        tex_a.len() >= 4 + 3,
        "e as três imagens da mangueira: {:?}",
        tex_a.keys()
    );

    let destino = pasta("exportar_fora").join("solto.ph2d-brush.png");
    let mut q = super::biblioteca::Seletor::default();
    let d = destino.clone();
    q.exportar = Box::new(move |_| Some(d.clone()));
    p.seletor = q;
    p.atender(&mut t, PedidoPreset::Exportar, &mut toasts);
    sem_erros(&toasts);
    assert!(destino.is_file());

    let outra = pasta("importar");
    let mut t2 = tela(PaintMedia::Digital);
    let mut p2 = Presets::abrir_em(
        pasta_de_fabrica(),
        Some(outra.clone()),
        &mut t2,
        &mut toasts,
    );
    let mut q = super::biblioteca::Seletor::default();
    let d = destino.clone();
    q.importar = Box::new(move || vec![d.clone()]);
    p2.seletor = q;
    p2.atender(&mut t2, PedidoPreset::Importar, &mut toasts);
    sem_erros(&toasts);
    assert!(
        outra.join(rel).is_file(),
        "entrou no MESMO lugar da biblioteca"
    );
    assert_eq!(p2.vista.ativo.as_deref(), Some(rel), "e ficou no pincel");
    let entrada = p2.bib.procurar(rel).unwrap();
    assert_eq!(entrada.preset.origem, Origem::Importado);
    assert_eq!(
        entrada.preset.parametros.library_folder, None,
        "a pasta vive no caminho"
    );
    assert_eq!(t2.paint_media(), PaintMedia::Watercolor);
    let (prm_b, tex_b) = t2.preset_capturar(&base);
    assert_eq!(prm_b, prm_a, "os campos ao bit");
    assert_eq!(tex_b, tex_a, "as imagens ao byte");

    let mut ta = tela(PaintMedia::Watercolor);
    let _ = ta.preset_aplicar(p.bib.procurar(rel).map(|e| &e.preset).unwrap());
    let mut tb = tela(PaintMedia::Watercolor);
    let _ = tb.preset_aplicar(&entrada.preset);
    let (a, b) = (traco(&mut ta), traco(&mut tb));
    assert!(!a.is_empty());
    assert_eq!(a, b, "a mesma pintura");
    let mut tf = tela(PaintMedia::Watercolor);
    let _ = tf.preset_aplicar(&p.bib.procurar(AGUARELA).unwrap().preset);
    assert_ne!(traco(&mut tf), a, "controlo: as imagens pintam");
}

/// Um ficheiro que não é um preset, ou de um meio que este PH2D não tem, não entra — e diz porquê.
#[test]
fn importar_recusa_o_que_nao_serve_e_diz_porque() {
    let (mut p, mut t, mut toasts, uti) = abre("importar_mau");
    let fora = pasta("importar_mau_fora");
    let lixo = fora.join("lixo.ph2d-brush.png");
    std::fs::write(&lixo, b"nao e um png").unwrap();
    let mut estranho = p.bib.procurar(ROUND).unwrap().preset.clone();
    estranho.parametros.media = "Fresco".into();
    let e = fora.join("estranho.ph2d-brush.png");
    std::fs::write(&e, ph2d_brush_presets::escrever(&estranho).unwrap()).unwrap();
    let n = p.vista.entradas.len();
    let entrou = p.importar_de(&[lixo, e], &mut toasts);
    assert!(entrou.is_empty());
    assert_eq!(p.vista.entradas.len(), n);
    let erros = toasts
        .iter()
        .filter(|t| t.severity == ph2d_editor_core::toast::ToastSeverity::ErrorState)
        .count();
    assert_eq!(erros, 2, "um aviso por ficheiro recusado");
    assert!(
        std::fs::read_dir(&uti).map_or(0, Iterator::count) <= 1,
        "nada gravado"
    );
    let _ = &mut t;
}
