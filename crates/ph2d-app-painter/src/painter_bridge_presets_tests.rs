//! A ponte dos presets sobre o disco: a fábrica REAL (`assets/brushes/`, só lida) e uma pasta do
//! artista temporária. Cada pedido do painel → o pincel e os ficheiros mudam como o botão diz.

use super::*;
use ph2d_editor_core::toast::ToastSeverity;
use ph2d_tool_painter::{MEIO_DO_PRESET, PaintMedia};

const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";

fn pasta(nome: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_bridge_presets_{}_{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn abre(nome: &str) -> (Presets, PainterTool, ToastQueue, PathBuf) {
    let uti = pasta(nome);
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let mut toasts = ToastQueue::default();
    let p = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t, &mut toasts);
    (p, t, toasts, uti)
}

fn sem_erros(toasts: &ToastQueue) {
    let erros: Vec<&str> = toasts
        .iter()
        .filter(|t| t.severity == ToastSeverity::ErrorState)
        .map(|t| t.message.as_str())
        .collect();
    assert!(erros.is_empty(), "{erros:?}");
}

/// Um knob do pincel pela chave do registo (a mesma que o preset grava).
fn knob(t: &PainterTool, chave: &str) -> f64 {
    let base = Parametros::novo(MEIO_DO_PRESET, "", "");
    t.wet_preset_capturar(&base).0.engine[chave]
}

#[test]
fn escolher_poe_o_preset_no_pincel_e_lembra_o_ativo() {
    let (mut p, mut t, mut toasts, uti) = abre("escolher");
    // O número da fábrica CONTA-SE (`the_factory_presets_are_counted`), nunca se escreve aqui.
    let fabrica = ph2d_brush_presets::Biblioteca::carregar(&pasta_de_fabrica(), None);
    assert_eq!(
        p.vista.entradas.len(),
        fabrica.entradas.len(),
        "a fábrica inteira no navegador"
    );
    assert_eq!(p.vista.ativo, None);
    p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
    sem_erros(&toasts);
    assert_eq!(
        knob(&t, "pigmentPerDab"),
        2.8,
        "o Wash com o filme (`93e0cc73bd`, J4 aguarela: pigmento 2,8; era 21 do 02 §6.9)"
    );
    // As cerdas moram no Shape (plano 27 W2): o Wash de antes chega com o Shape em Bristles e as dele.
    let b = t.brush_settings();
    assert_eq!(
        b.shape_kind,
        ph2d_painter_brush::TextureKind::Bristles.to_u8()
    );
    let mut prm = [0.5; ph2d_painter_brush::MAX_TEX_PARAMS];
    prm.copy_from_slice(&b.shape_params);
    assert_eq!(
        ph2d_painter_brush::texture::bristle_knobs(&prm).bristle_count,
        3000.0
    );
    assert_eq!(p.vista.ativo.as_deref(), Some(WASH));
    // Reaberto (outra sessão): o pincel abre com o Wash.
    let mut t2 = PainterTool::default();
    t2.set_paint_media(PaintMedia::WetPaint);
    let p2 = Presets::abrir_em(pasta_de_fabrica(), Some(uti), &mut t2, &mut toasts);
    assert_eq!(p2.vista.ativo.as_deref(), Some(WASH));
    assert_eq!(knob(&t2, "pigmentPerDab"), 2.8);
}

#[test]
fn a_politica_decide_o_que_volta_com_o_preset() {
    for (politica, esperado) in [(0u8, 0.3), (1, 0.3), (2, 1.0)] {
        let (mut p, mut t, mut toasts, _) = abre(&format!("politica{politica}"));
        p.atender(&mut t, PedidoPreset::Politica(politica), &mut toasts);
        p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
        t.wet_preset_aplicar_volumes(&ph2d_brush_presets::Volumes {
            water: Some(0.3),
            ..Default::default()
        });
        p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
        p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
        assert_eq!(t.wet_knobs().water, esperado, "política {politica}");
        sem_erros(&toasts);
    }
}

#[test]
fn guardar_como_novo_cria_um_ficheiro_do_artista_e_passa_a_activo() {
    let (mut p, mut t, mut toasts, uti) = abre("novo");
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    t.set_brush_size_px(9.0);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let rel = "WetPainter/Watercolor/Fluid/Wet Round 2.ph2d-brush.png";
    assert!(uti.join(rel).is_file(), "o ficheiro do artista existe");
    assert_eq!(p.vista.ativo.as_deref(), Some(rel));
    let e = p.bib.procurar(rel).unwrap();
    assert_eq!(e.origem, Origem::Utilizador);
    assert_eq!(
        e.preset.parametros.volumes.size_px,
        Some(18.0),
        "guardou o diâmetro"
    );
    assert_ne!(e.preset.miniatura.largura, 1, "a miniatura foi pintada");
    assert!(p.bib.procurar(ROUND).is_some(), "o de fábrica continua lá");
}

#[test]
fn guardar_como_padrao_sobrepoe_e_o_reset_volta_a_fabrica() {
    let (mut p, mut t, mut toasts, uti) = abre("padrao");
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    let fabrica = knob(&t, "pigmentPerDab");
    // O artista mexe no Pigment (pelo mesmo caminho que um preset usa para escrever um knob).
    let mut prm = p.bib.procurar(ROUND).unwrap().preset.clone();
    prm.parametros.engine.insert("pigmentPerDab".into(), 1500.0);
    let _ = t.wet_preset_aplicar(&prm);
    p.atender(&mut t, PedidoPreset::GuardarComoPadrao, &mut toasts);
    sem_erros(&toasts);
    assert!(uti.join(ROUND).is_file());
    let e = p.bib.procurar(ROUND).unwrap();
    assert_eq!((e.origem, e.tem_fabrica), (Origem::Utilizador, true));
    assert_eq!(e.preset.parametros.engine["pigmentPerDab"], 1500.0);

    p.atender(&mut t, PedidoPreset::Repor, &mut toasts);
    sem_erros(&toasts);
    assert!(
        !uti.join(ROUND).exists(),
        "o Reset apagou a cópia do artista"
    );
    assert_eq!(p.bib.procurar(ROUND).unwrap().origem, Origem::Fabrica);
    assert_eq!(
        knob(&t, "pigmentPerDab"),
        fabrica,
        "e o pincel voltou ao de fábrica"
    );
}

#[test]
fn o_favorito_aparece_na_vista() {
    let (mut p, mut t, mut toasts, _) = abre("favorito");
    p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert!(p.vista.procurar(WASH).unwrap().favorito);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert!(!p.vista.procurar(WASH).unwrap().favorito);
}

/// HR-15: as pastas da FÁBRICA (meio e grupo) que o navegador mostra como título passam pela tabela de
/// texto — no idioma de teste nenhuma sai crua. Controlo: uma pasta do artista sai como ele a chamou.
#[test]
fn cada_pasta_de_fabrica_tem_rotulo() {
    let bib = ph2d_brush_presets::Biblioteca::carregar(&pasta_de_fabrica(), None);
    assert!(!bib.entradas.is_empty());
    for e in &bib.entradas {
        let [_, meio, grupo, _] = e.partes();
        for pasta in [meio, grupo] {
            assert_ne!(
                rotulo_da_pasta(Idioma::Teste, pasta),
                pasta,
                "{}: a pasta {pasta:?} não tem rótulo em `rotulo_da_pasta`",
                e.relativo
            );
        }
    }
    assert_eq!(
        rotulo_da_pasta(Idioma::Teste, "Os meus pincéis"),
        "Os meus pincéis"
    );
}

// ── L1: os presets nos quatro meios ─────────────────────────────────────────────────────────────────

/// Os três presets de EXEMPLO (um por meio antigo, `docs/Painter/Pinceis/fixtures/presets_exemplo`) copiados
/// para a pasta do artista: a biblioteca passa a ter os quatro meios.
fn com_exemplos(nome: &str, meio: PaintMedia) -> (Presets, PainterTool, ToastQueue, PathBuf) {
    let uti = pasta(nome);
    let ex = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/fixtures/presets_exemplo");
    for e in &ph2d_brush_presets::Biblioteca::carregar(&ex, None).entradas {
        let destino = uti.join(&e.relativo);
        std::fs::create_dir_all(destino.parent().unwrap()).unwrap();
        std::fs::copy(&e.caminho, destino).unwrap();
    }
    let mut t = PainterTool::default();
    t.set_paint_media(meio);
    let mut toasts = ToastQueue::default();
    let p = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t, &mut toasts);
    (p, t, toasts, uti)
}

const LAPIS: &str = "Digital/Pencil/Graphite/Sketch Pencil.ph2d-brush.png";
const EMPASTE: &str = "Impasto/Oil/Bristle/Thick Impasto.ph2d-brush.png";

/// ⭐ O navegador abre SEM o Wet Paint, lista os quatro meios (o de agora primeiro) e escolher um preset de outro
/// meio TROCA o meio; o activo é o de cada meio.
#[test]
fn o_navegador_abre_em_qualquer_meio_e_escolher_troca_o_meio() {
    let (mut p, mut t, mut toasts, _) = com_exemplos("quatro", PaintMedia::Digital);
    sem_erros(&toasts);
    let fabrica = ph2d_brush_presets::Biblioteca::carregar(&pasta_de_fabrica(), None);
    assert_eq!(p.vista.entradas.len(), fabrica.entradas.len() + 3);
    // Desde a fábrica nos quatro meios (`938271318c`; o Flat Block-in, `3b4c2b46d3`, é hoje o 1.º do Digital) o
    // Sketch Pencil do exemplo deixou de ser o único do meio de agora: o bloco do Digital inteiro vem primeiro.
    let meio_de = |rel: &str| p.bib.procurar(rel).unwrap().preset.parametros.media.clone();
    let agora = nome_do_meio(PaintMedia::Digital);
    let n_agora = p
        .vista
        .entradas
        .iter()
        .filter(|v| meio_de(&v.relativo) == agora)
        .count();
    assert!(n_agora > 1, "a fábrica tem pincéis do Digital ({n_agora})");
    assert!(
        p.vista.entradas[..n_agora]
            .iter()
            .all(|v| meio_de(&v.relativo) == agora),
        "o meio de agora primeiro"
    );
    assert!(
        p.vista.entradas[..n_agora]
            .iter()
            .any(|v| v.relativo == LAPIS),
        "o lápis do exemplo no bloco do meio de agora"
    );
    assert_eq!(p.vista.ativo, None);
    p.atender(&mut t, PedidoPreset::Escolher(EMPASTE.into()), &mut toasts);
    sem_erros(&toasts);
    assert_eq!(t.paint_media(), PaintMedia::Impasto);
    assert_eq!(p.vista.ativo.as_deref(), Some(EMPASTE));
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    assert_eq!(t.paint_media(), PaintMedia::WetPaint);
    assert_eq!(p.vista.ativo.as_deref(), Some(ROUND));
    p.atender(&mut t, PedidoPreset::Escolher(LAPIS.into()), &mut toasts);
    assert_eq!(t.paint_media(), PaintMedia::Digital);
    assert_eq!(p.vista.ativo.as_deref(), Some(LAPIS));
    assert_eq!(
        p.estado.ativo_por_meio.get("Impasto").map(String::as_str),
        Some(EMPASTE),
        "o activo de cada meio fica lembrado"
    );
    sem_erros(&toasts);
}

/// ⭐ Um preset de um meio que este PH2D não conhece é RECUSADO com aviso: o meio e o activo não mudam.
#[test]
fn um_meio_desconhecido_e_recusado_com_aviso() {
    let (mut p, mut t, mut toasts, uti) = com_exemplos("estranho", PaintMedia::Digital);
    let mut x = p.bib.procurar(LAPIS).unwrap().preset.clone();
    x.parametros.media = "Fluid".into();
    let rel = "Fluid/Any/Any/Strange.ph2d-brush.png";
    gravar(&uti, rel, &x).unwrap();
    p.recarregar(&mut toasts);
    p.atender(&mut t, PedidoPreset::Escolher(LAPIS.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::Escolher(rel.into()), &mut toasts);
    assert_eq!(t.paint_media(), PaintMedia::Digital);
    assert_eq!(p.vista.ativo.as_deref(), Some(LAPIS));
    assert!(
        toasts
            .iter()
            .any(|m| m.severity == ToastSeverity::ErrorState && m.message.contains("Fluid")),
        "o artista é avisado"
    );
}

/// «Save as New» no Digital grava um preset DIGITAL na pasta do Digital, com a miniatura do meio.
#[test]
fn guardar_como_novo_no_digital_vai_para_a_pasta_do_digital() {
    let (mut p, mut t, mut toasts, uti) = com_exemplos("novo_digital", PaintMedia::Digital);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let rel = format!(
        "Digital/User/User/{}.ph2d-brush.png",
        tr("app.painter.presets.new_name")
    );
    assert!(uti.join(&rel).is_file(), "{rel}");
    let e = p.bib.procurar(&rel).unwrap();
    assert_eq!(e.preset.parametros.media, "Digital");
    assert!(
        e.preset.miniatura.rgba.chunks(4).any(|px| px[0] < 200),
        "a miniatura pintou o traço"
    );
    assert_eq!(p.vista.ativo.as_deref(), Some(rel.as_str()));
}
