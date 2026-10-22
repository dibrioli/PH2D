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

/// Cada pasta de MEIO da fábrica cai numa categoria da tabela (nenhuma em «Other»), e cada rótulo de
/// categoria vem da tabela de texto (HR-15: no idioma de teste nenhum sai cru). Controlo: uma pasta do
/// artista cai em «Other».
#[test]
fn cada_pasta_de_meio_da_fabrica_tem_categoria() {
    use crate::painter_bridge_categorias::CATEGORIAS;
    use ph2d_i18n::{Idioma, tr_em};
    let bib = ph2d_brush_presets::Biblioteca::carregar(&pasta_de_fabrica(), None);
    assert!(!bib.entradas.is_empty());
    for e in &bib.entradas {
        let [_, meio, _, _] = e.partes();
        assert!(
            categoria_da_pasta(meio) < CATEGORIAS.len(),
            "{}: a pasta {meio:?} não tem categoria em `CATEGORIAS`",
            e.relativo
        );
    }
    for i in 0..=CATEGORIAS.len() {
        let r = rotulo_da_categoria(Idioma::Teste, i);
        assert_ne!(
            r,
            rotulo_da_categoria(Idioma::Ingles, i),
            "a categoria {i} saiu crua"
        );
    }
    assert_eq!(categoria_da_pasta("Os meus pincéis"), CATEGORIAS.len());
    assert_eq!(
        rotulo_da_categoria(Idioma::Ingles, CATEGORIAS.len()),
        tr_em(Idioma::Ingles, "app.painter.presets.category.other")
    );
}

/// ⭐ (i) O navegador agrupa a fábrica por CATEGORIA, na ordem fixa da tabela, cada categoria num bloco
/// contínuo (um título cada no popover) — e não por pasta: a fábrica tem muito mais pastas que
/// categorias. Dentro de cada categoria a ordem da biblioteca (a série de lápis pelo grau) não se parte.
#[test]
fn o_navegador_agrupa_por_categoria_na_ordem_fixa() {
    use crate::painter_bridge_categorias::CATEGORIAS;
    let (p, _, toasts, _) = abre("categorias");
    sem_erros(&toasts);
    let mut blocos: Vec<&str> = Vec::new();
    for v in &p.vista.entradas {
        if blocos.last() != Some(&v.categoria.as_str()) {
            blocos.push(&v.categoria);
        }
    }
    let ordem: Vec<&str> = (0..CATEGORIAS.len())
        .map(|i| rotulo_da_categoria(idioma(), i))
        .collect();
    assert_eq!(
        blocos, ordem,
        "as 11 categorias, cada uma num bloco, na ordem da tabela"
    );
    let pastas: BTreeSet<&str> = p
        .bib
        .entradas
        .iter()
        .map(|e| e.relativo.rsplit_once('/').map_or("", |(d, _)| d))
        .collect();
    assert!(
        pastas.len() > blocos.len() * 2,
        "a fixtura distingue pasta de categoria: {} pastas, {} categorias",
        pastas.len(),
        blocos.len()
    );
    // A ordem da biblioteca dentro da categoria (o meio de agora é o Wet Paint: o lápis é todo dele).
    let lapis = rotulo_da_categoria(idioma(), categoria_da_pasta("Pencil"));
    let na_vista: Vec<&str> = p
        .vista
        .entradas
        .iter()
        .filter(|v| v.categoria == lapis)
        .map(|v| v.relativo.as_str())
        .collect();
    let na_bib: Vec<&str> = p
        .bib
        .entradas
        .iter()
        .filter(|e| e.partes()[1] == "Pencil")
        .map(|e| e.relativo.as_str())
        .collect();
    assert_eq!(na_vista, na_bib, "a série de lápis pelo grau");
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
    // Desde as CATEGORIAS (dono 22/10) a lei é «o meio de agora primeiro DENTRO de cada categoria» (era o meio
    // de agora primeiro na lista inteira): o Sketch Pencil do exemplo, o único lápis do Digital, abre o Pencil.
    let meio_de = |rel: &str| p.bib.procurar(rel).unwrap().preset.parametros.media.clone();
    let agora = nome_do_meio(PaintMedia::Digital);
    for par in p.vista.entradas.windows(2) {
        if par[0].categoria == par[1].categoria {
            assert!(
                !(meio_de(&par[0].relativo) != agora && meio_de(&par[1].relativo) == agora),
                "{}: um pincel do meio de agora depois de um de outro meio na mesma categoria",
                par[1].relativo
            );
        }
    }
    let i = p
        .vista
        .entradas
        .iter()
        .position(|v| v.relativo == LAPIS)
        .unwrap();
    assert!(
        i == 0 || p.vista.entradas[i - 1].categoria != p.vista.entradas[i].categoria,
        "o lápis do exemplo abre a categoria Pencil"
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

const AGUADA: &str = "Watercolor/Watercolor/Round/Flat Wash.ph2d-brush.png";

/// ⭐ (v) Escolher um preset de OUTRO modo troca o Paint Mode — a partir de cada um dos quatro, para cada
/// um dos outros três (os exemplos dão um preset a cada meio).
#[test]
fn escolher_um_preset_de_outro_modo_troca_o_modo_a_partir_de_cada_modo() {
    let alvos = [
        (PaintMedia::Digital, LAPIS),
        (PaintMedia::Watercolor, AGUADA),
        (PaintMedia::Impasto, EMPASTE),
        (PaintMedia::WetPaint, ROUND),
    ];
    for (de, _) in alvos {
        for (para, rel) in alvos.iter().filter(|(m, _)| *m != de) {
            let (mut p, mut t, mut toasts, _) = com_exemplos(&format!("troca_{de:?}_{para:?}"), de);
            assert_eq!(t.paint_media(), de);
            p.atender(&mut t, PedidoPreset::Escolher((*rel).into()), &mut toasts);
            sem_erros(&toasts);
            assert_eq!(t.paint_media(), *para, "{de:?} → {rel}");
            assert_eq!(p.vista.ativo.as_deref(), Some(*rel), "{de:?} → {para:?}");
        }
    }
}
