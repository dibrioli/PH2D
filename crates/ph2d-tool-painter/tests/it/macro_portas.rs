//! ⭐ **Gate 4 das macros: o censo das portas do Painter** (`src/macro_track/portas.rs`).
//!
//! A população vem do FONTE: cada `pub fn NOME(&mut self` num `impl … PainterTool` (fora dos ficheiros
//! de teste) e os métodos `&mut self` dos três contratos. Cada uma tem UMA decisão; a gravada passa pelo
//! `porta!`, a recusada chama `macro_recusa("NOME")`, e uma interna não tem chamador fora da crate —
//! uma chamada de fora a uma interna seria um passo que a macro perde EM SILÊNCIO.

use ph2d_tool_painter::macro_track::portas::{Decisao, INTERNAS, PORTAS, como_portas_do_nucleo};
use std::path::{Path, PathBuf};

/// Os métodos `&mut self` dos contratos (`Tool`, `RasterEditTool`, `CanvasPaintTool`) — `fn`, não `pub fn`.
const CONTRATOS: [&str; 13] = [
    "on_activate",
    "on_deactivate",
    "handle_panel_event",
    "on_tick",
    "as_any_mut",
    "as_raster_edit_mut",
    "as_canvas_paint_mut",
    "set_source",
    "current_preview",
    "take_pending_commit",
    "run_full",
    "deactivate",
    "on_canvas_pointer",
];

/// Chamadas de fora com o MESMO nome de uma porta interna, mas sobre outro tipo (o porquê ao lado).
const HOMONIMOS: [(&str, &str, &str); 3] = [
    (
        "apply_mask",
        "crates/ph2d-audio-ml/vendor/deep_filter/src/tract.rs",
        "a máscara espectral do denoise",
    ),
    (
        "duplicate_layer",
        "crates/ph2d-app-flip/src/layers.rs",
        "as camadas do Flip",
    ),
    (
        "select_layer",
        "crates/ph2d-wet-paint/src/painter.rs",
        "as camadas do motor do Wet Paint",
    ),
];

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn e_teste(p: &Path) -> bool {
    let b = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let s = p.to_string_lossy();
    b == "tests.rs"
        || b.ends_with("_tests.rs")
        || s.contains("/tests/")
        || ["sonda", "measure_", "diag_", "medida_"]
            .iter()
            .any(|x| b.starts_with(x))
        || b.ends_with("_probe.rs")
        || b.ends_with("_smoke.rs")
}

fn ficheiros(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            ficheiros(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") && !e_teste(&p) {
            out.push(p);
        }
    }
}

/// `impl PainterTool {`, `impl super::PainterTool {`, `impl Tool for PainterTool {` …
fn e_do_painter(cab: &str) -> bool {
    let sem = cab.trim_end().trim_end_matches('{').trim();
    let alvo = sem.rsplit(' ').next().unwrap_or("");
    alvo.rsplit("::").next() == Some("PainterTool")
}

/// As portas `pub fn NOME(&mut self` do Painter, com o corpo de cada uma (até à próxima `fn` do bloco).
fn presentes() -> Vec<(String, String)> {
    let mut fs = Vec::new();
    ficheiros(&raiz().join("crates/ph2d-tool-painter/src"), &mut fs);
    let mut out = Vec::new();
    for f in fs {
        let texto = std::fs::read_to_string(&f).expect("lê o fonte");
        let linhas: Vec<&str> = texto.lines().collect();
        let (mut dentro, mut ant) = (false, "");
        for (k, l) in linhas.iter().enumerate() {
            if l.starts_with("impl") {
                dentro = e_do_painter(l) && !ant.starts_with("#[cfg(test)]");
            } else if l.starts_with('}') {
                dentro = false;
            }
            let junta = if l.trim_end().ends_with('(') {
                format!("{l}{}", linhas.get(k + 1).map_or("", |x| x.trim()))
            } else {
                (*l).to_owned()
            };
            if dentro
                && let Some(resto) = junta.strip_prefix("    pub fn ")
                && let Some((nome, depois)) = resto.split_once('(')
                && depois.trim_start().starts_with("&mut self")
            {
                out.push((nome.to_owned(), corpo(&linhas, k)));
            }
            if !l.trim().is_empty() {
                ant = l;
            }
        }
    }
    out
}

fn corpo(linhas: &[&str], k: usize) -> String {
    let mut s = String::new();
    for l in &linhas[k..] {
        if !s.is_empty()
            && (l.starts_with("    pub fn ") || l.starts_with("    fn ") || l.starts_with('}'))
        {
            break;
        }
        s.push_str(l);
        s.push('\n');
    }
    s
}

/// O corpo de um método dos contratos (`    fn NOME(`) nos `impl … for PainterTool`.
fn corpo_do_contrato(nome: &str) -> String {
    let mut fs = Vec::new();
    ficheiros(&raiz().join("crates/ph2d-tool-painter/src"), &mut fs);
    for f in fs {
        let texto = std::fs::read_to_string(&f).expect("lê o fonte");
        let linhas: Vec<&str> = texto.lines().collect();
        let mut dentro = false;
        for (k, l) in linhas.iter().enumerate() {
            if l.starts_with("impl") {
                dentro = l.contains(" for PainterTool");
            }
            if dentro && l.starts_with(&format!("    fn {nome}(")) {
                return corpo(&linhas, k);
            }
        }
    }
    panic!("o contrato `{nome}` não está num `impl … for PainterTool`")
}

#[test]
fn toda_porta_do_painter_tem_decisao() {
    let pres = presentes();
    assert!(
        pres.len() > 300,
        "o varrimento achou só {} portas: a régua partiu-se (a população real passa de 400)",
        pres.len()
    );
    let mut nomes: Vec<&str> = pres.iter().map(|(n, _)| n.as_str()).collect();
    nomes.extend(CONTRATOS);
    let decl = como_portas_do_nucleo();
    let sem = ph2d_macro::undecided(&decl, &nomes);
    assert!(
        sem.is_empty(),
        "portas do Painter SEM decisão de gravabilidade (declare-as em `macro_track/portas.rs` — grava, \
         recusa com o porquê, ou interna): {sem:?}"
    );
    let orfas = ph2d_macro::orphans(&decl, &nomes);
    assert!(
        orfas.is_empty(),
        "decisões sobre portas que já não existem: {orfas:?}"
    );
    let mut repetidas: Vec<&str> = decl.iter().map(|d| d.name).collect();
    repetidas.sort_unstable();
    repetidas.dedup();
    assert_eq!(repetidas.len(), decl.len(), "uma porta com duas decisões");
}

#[test]
fn a_porta_gravada_grava_e_a_recusada_recusa() {
    let pres = presentes();
    let corpo_de = |nome: &str| -> String {
        pres.iter()
            .find(|(n, _)| n == nome)
            .map(|(_, c)| c.clone())
            .unwrap_or_else(|| corpo_do_contrato(nome))
    };
    let mut erros = Vec::new();
    for p in PORTAS {
        let c = corpo_de(p.nome);
        match p.decisao {
            Decisao::Grava if !(c.contains("porta!(") || c.contains("macro_entra(")) => {
                erros.push(format!(
                    "{} é GRAVADA e o corpo não passa pelo `porta!`",
                    p.nome
                ));
            }
            Decisao::Recusa(_) if !c.contains(&format!("macro_recusa(\"{}\")", p.nome)) => {
                erros.push(format!(
                    "{} é RECUSADA e o corpo não chama `macro_recusa`",
                    p.nome
                ));
            }
            _ => {}
        }
    }
    assert!(erros.is_empty(), "{erros:#?}");
}

#[test]
fn nenhuma_porta_interna_tem_chamador_de_fora() {
    let mut fs = Vec::new();
    for d in ["crates", "shells"] {
        ficheiros(&raiz().join(d), &mut fs);
    }
    let mut achados = Vec::new();
    for f in &fs {
        let rel = f
            .strip_prefix(raiz())
            .unwrap_or(f)
            .to_string_lossy()
            .replace("\\", "/");
        let rel = rel.trim_start_matches("./").to_owned();
        if rel.starts_with("crates/ph2d-tool-painter/") || !rel.contains("/src/") {
            continue;
        }
        let texto = std::fs::read_to_string(f).unwrap_or_default();
        for &n in INTERNAS {
            if texto.contains(&format!(".{n}("))
                && !HOMONIMOS
                    .iter()
                    .any(|(h, p, _)| *h == n && rel.ends_with(p))
            {
                achados.push(format!("{n} ← {rel}"));
            }
        }
    }
    assert!(
        achados.is_empty(),
        "portas INTERNAS chamadas de fora (a macro perderia esse passo em silêncio — grave-a ou recuse-a \
         em `macro_track/portas.rs`, ou ponha o homónimo na lista com o porquê): {achados:#?}"
    );
}

/// O comportamento da recusa (o gate acima só lê o fonte): uma porta recusada chamada de fora a gravar
/// fica na gravação, e o tocador PÁRA nela em voz alta.
#[test]
fn uma_porta_recusada_a_gravar_para_o_tocador() {
    use ph2d_editor_core::tool::RasterEditTool;
    use ph2d_tool_painter::macro_track::tocar::{ErroDaMacro, TocadorDoPainter};
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    let (inicial, _) = t.macro_grava().expect("arma");
    t.bind_document(99, vec![0u8; 8 * 8 * 4], 8, 8);
    let cargas = t.macro_para();
    let mut toc =
        TocadorDoPainter::novo(&inicial, cargas.iter().map(Vec::as_slice)).expect("inicial");
    let erro = cargas.iter().find_map(|c| toc.aplica(c).err());
    assert_eq!(
        erro,
        Some(ErroDaMacro::PortaRecusada("bind_document".to_owned())),
        "a porta recusada não parou o tocador"
    );
}

/// ⭐ **Só a porta de FORA grava.** `wet_preset_aplicar` chama por dentro outras portas gravadas
/// (`wet_preset_aplicar_volumes` → `set_brush_size_px`, `wet_preset_aplicar_modo`): a gravação tem UM
/// passo, o do preset — gravar também os de dentro aplicá-los-ia duas vezes ao tocar (um botão de
/// alternar desfazia-se). Nasceu de uma mutação que sobreviveu a todos os outros gates.
#[test]
fn a_porta_chamada_por_dentro_de_outra_nao_grava() {
    use ph2d_editor_core::tool::RasterEditTool;
    use ph2d_tool_painter::macro_track::Passo;
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    let _ = t.macro_grava().expect("arma");
    let mut parametros = ph2d_brush_presets::Parametros::novo("WetPaint", "macro", "aninhado");
    parametros.volumes.size_px = Some(22.0);
    parametros.paint_mode = Some("Blend".to_owned());
    let preset = ph2d_brush_presets::Preset {
        parametros,
        origem: ph2d_brush_presets::Origem::default(),
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas: Default::default(),
    };
    let _ = t.wet_preset_aplicar(&preset);
    let passos: Vec<Passo> = t
        .macro_passos()
        .iter()
        .map(|c| Passo::descodifica(c).expect("lê-se"))
        .collect();
    assert!(
        matches!(passos.as_slice(), [Passo::PresetAplica { .. }]),
        "a porta de fora gravou e as de dentro também: {passos:?}"
    );
}
