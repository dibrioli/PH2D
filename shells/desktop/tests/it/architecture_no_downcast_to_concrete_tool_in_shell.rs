//! Wave 10 / Etapa 3 arch-gate: scanning `shells/desktop/src/` for
//! downcasts to concrete `ph2d_tool_*` types.
//!
//! ## What this gate enforces
//!
//! ADR-0040 §2.1 + ADR-0041 establish `RasterEditTool` as the
//! generic channel for raster I/O lifecycle. Bridges that drive raster
//! tools should reach them via `as_raster_edit_mut()` (the upcast),
//! NOT via `downcast_mut::<SomeConcreteTool>()`.
//!
//! Genuine exceptions exist (documented in ADR-0040 §3) where a
//! tool-specific affordance can't fit the generic contract:
//!   - eyedropper.rs — BgR-specific colour pick UI
//!   - protect_brush.rs — BgR-specific painting input dispatch
//!   - bridges with panel-snapshot / overlay-tint / brush-ring needs
//!
//! The allowlist below freezes the current legitimate-exception set.
//! NEW bridges or NEW exceptions require either:
//!   1. Adding to the allowlist with explicit justification (Coord-A
//!      decision), OR
//!   2. Reworking to use the trait surface instead.
//!
//! The gate counts downcasts in non-allowlisted files. Adding a NEW
//! downcast to a non-allowlisted file fails. Adding a downcast inside
//! an allowlisted file is OK — those files are documented as
//! tool-specific.

use std::fs;
use std::path::{Path, PathBuf};

/// Files whose tool-concrete downcasts are documented exceptions
/// (ADR-0040 §3). Paths are relative to `shells/desktop/`.
///
/// **Adding to this list is a Coord-A decision** with explicit
/// justification in the per-tool ADR or in DIRETRIZ §3.8.3.1.
const DOWNCAST_ALLOWLIST: &[&str] = &[
    // Eyedropper: BgR-specific UI affordance (clicks canvas, samples
    // pixel under cursor, feeds add_extra_color). Genuinely BgR-only.
    "src/input_dispatch/eyedropper.rs",
    // Protect-brush: BgR-specific painting input dispatch (dabs into
    // protect_mask). Genuinely BgR-only.
    "src/input_dispatch/protect_brush.rs",
    // BgR bridge: needs concrete-type access for panel snapshot publish,
    // protect-mask tint overlay, brush-size ring. Each is a documented
    // BgR-specific affordance.
    "src/render_loop/bgremoval_preview.rs",
    // CEQ bridge: needs concrete-type access for panel snapshot publish
    // + dropdown-close drain. Other bits go through the trait + helpers.
    "src/render_loop/color_equalization_bridge.rs",
    // Upscale bridge: needs concrete-type access for panel snapshot
    // publish. Other bits go through the trait + helpers.
    "src/render_loop/upscale_bridge.rs",
    // Padding bridge: tool is geometric-only (DIRETRIZ §3.8.3.1
    // exception), no RasterEditTool impl, must downcast for spec/Apply.
    "src/render_loop/padding_bridge.rs",
    // EqualizeSizes bridge: tool is multi-sprite-required (DIRETRIZ
    // §3.8.3.1 exception), no RasterEditTool impl, must downcast.
    "src/render_loop/equalize_sizes_bridge.rs",
    // ⚠️⚠️ **AS TRÊS ENTRADAS DO `painter_bridge` SAÍRAM DESTA LISTA em 2026-09-12** (W2 Fase D):
    //    o `painter_bridge.rs`, o `painter_bridge_queries.rs` e o
    //    `painter_bridge_shape_preview.rs` mudaram-se para `crates/ph2d-app-painter/src/`, e esta
    //    lista é a das licenças **da SHELL** — o gate irmão varre `shells/desktop/src` e mais nada.
    //    ⭐ Foi a metade de OBSOLESCÊNCIA deste ficheiro que as apanhou, pelo nome, em voz alta.
    //    ⛔ Os comentários abaixo continuam a dizer *«same exception class as painter_bridge.rs»*:
    //    a CLASSE da excepção é a mesma e a frase continua a explicar o porquê — mas o ficheiro que
    //    ela cita vive hoje noutra crate. *Uma prosa que cita um endereço envelhece com a mudança
    //    de casa, e é por isto que este parágrafo existe em vez de cinco edições.*
    // Painter bridge-queries: `painter_has_unflushed_strokes` + `apply_layer_
    // reparent` split out of painter_bridge.rs (HR-18 LOC cap); same downcast
    // exception class as painter_bridge.rs.
    // Painter shape-source preview: `drive_shape_source_preview` split out of
    // painter_bridge.rs (HR-18 LOC cap); same downcast exception class as
    // painter_bridge.rs. (Coord ship-fix, 2026-07-02.)
    // image_edit drain: per-tool bake dispatch. Some downcasts retire
    // in later Etapas as OneShotImageOp routes via Registry kind.
    "src/render_loop/image_edit.rs",
    // ⭐ **A entrada do `render_loop/vector_bridge.rs` SAIU em 2026-09-12 (W2 Fase D):** o ficheiro
    // mudou-se inteiro para `ph2d-app-vec`, logo a shell já não contém aquele downcast e a lei
    // deste gate — *o laço central fica livre de downcasts* — passou a ser satisfeita mais
    // fortemente do que por uma excepção. ⛔ A entrada não foi apagada por conveniência: o censo de
    // obsolescência abaixo **obriga-o**.
    // render_loop/mod.rs: PainterTool downcasts for the right-click handle-kind
    // drains (falloff / curve point handle). Same exception class as
    // painter_bridge; the central dispatch stays free of *vector* downcasts.
    "src/render_loop/mod.rs",
    // ⚠️ **O `fase_sculpt3d_bake.rs` HERDOU um downcast do `render_loop/mod.rs`** (OBRA 2 da
    // `line/render-loop`, 2026-09-12): o alpha por imagem pergunta ao `PainterTool` o que a tela
    // MOSTRA (`needs_document_bind` + `composite_to_lum`, a porta do «Use as Brush Grain»), sem o
    // activar. É a MESMA excepção que a entrada do `mod.rs` acima licencia, mudada de ficheiro com o
    // corpo que a contém — a contagem de downcasts da shell não mudou (12). ⛔ Não é uma excepção
    // nova: o quadro partiu-se em fases, e a licença segue o SUJEITO, como a entrada da precisão.
    "src/render_loop/fase_sculpt3d_bake.rs",
    // ⚠️ **O `fase_use_as_paper.rs` HERDOU os downcasts do `render_loop/mod.rs`** (OBRA 2 da
    // `line/render-loop`, 2026-09-12): o *Use as Watercolor Paper / Granulation* da Hierarquia instala
    // a imagem no `PainterTool` concreto (slot Grain), e o bloco mudou-se verbatim para a fase — a
    // MESMA excepção de classe que a entrada do `mod.rs` acima documenta, sem downcast novo.
    "src/render_loop/fase_use_as_paper.rs",
    // ⚠️ **O `fase_use_as_brush.rs` HERDOU os downcasts do `render_loop/mod.rs`** (OBRA 2 da
    // `line/render-loop`, 2026-09-12): o *Use as Brush Shape / Grain* da Hierarquia instala a imagem
    // no `PainterTool` concreto e activa o pincel; o bloco mudou-se verbatim para a fase — a MESMA
    // excepção de classe da entrada do `mod.rs`, sem downcast novo.
    "src/render_loop/fase_use_as_brush.rs",
    // ⚠️ **O `fase_hierarchy_dispatch.rs` HERDOU um downcast do `render_loop/mod.rs`** (OBRA 2 da
    // `line/render-loop`, 2026-09-12): ao duplicar uma sprite que o Painter está a pintar, o fork assa a
    // tinta pelo `PainterTool` concreto (`has_unbaked_edits` + `auto_commit_painter`) antes de dar à cópia
    // uma textura própria; o bloco mudou-se verbatim para a fase — a MESMA excepção de classe da entrada
    // do `mod.rs`, sem downcast novo.
    "src/render_loop/fase_hierarchy_dispatch.rs",
    // Pointer forwarder: the colour-picker eyedropper samples the active PainterTool's layer COMPOSITE
    // (`sample_composite_at_uv`) + reads `repeat_image()` to walk the Repeat-Image neighbour tiles —
    // a Painter-specific affordance integrating the eyedropper with the layer system. Same exception
    // class as painter_canvas_input / painter_bridge (ADR-0040 §3). (Coord ship-fix, 2026-06-24.)
    "src/forwarding.rs",
    // Removed in Wave 10 / Etapa 3 audit [C1]: hero_intents/image_edit/*.rs
    // entries were pre-emptive — none of them actually downcast today.
    // The stale-check below ensures the allowlist only contains files
    // with REAL downcasts. Future bridges that need a downcast must
    // earn the entry with explicit justification.
];

fn collect_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target/ etc.
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" {
                    continue;
                }
                collect_rs_files_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

/// ⛔⛔ **O CENSO DE OBSOLESCÊNCIA — a metade que o comentário da lista PROMETIA e que não existia.**
///
/// A allowlist acima diz, por escrito: *«The stale-check below ensures the allowlist only contains
/// files with REAL downcasts»*. **Não havia stale-check nenhum.** Este ficheiro tinha UM teste, e ele
/// só olha os ficheiros que NÃO estão na lista — uma entrada podre é, para ele, invisível.
///
/// ⚠️ E ela apodreceu à vista em 2026-09-12: o `render_loop/vector_bridge.rs` mudou-se para
/// `ph2d-app-vec` na W2 Fase D e a entrada passou a nomear um ficheiro que não existe. Nada se
/// queixou. *Uma catraca sem censo de obsolescência não desce: ela vira LICENÇA* (`CLAUDE.md` §5.0)
/// — e uma que **diz** ter o censo é pior, porque quem lê deixa de o procurar.
///
/// Ele reprova nas duas formas de podridão: a entrada que **não existe** e a que existe mas **já não
/// tem downcast nenhum**.
#[test]
fn the_allowlist_only_names_files_that_still_downcast() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut podres: Vec<String> = Vec::new();
    for entrada in DOWNCAST_ALLOWLIST {
        let p = raiz.join(entrada);
        let Ok(conteudo) = fs::read_to_string(&p) else {
            podres.push(format!("{entrada}: o ficheiro NAO EXISTE"));
            continue;
        };
        let tem = conteudo.lines().any(|l| {
            let t = l.trim_start();
            !t.starts_with("//")
                && (l.contains("downcast_mut::<ph2d_tool_")
                    || l.contains("downcast_ref::<ph2d_tool_"))
        });
        if !tem {
            podres.push(format!(
                "{entrada}: existe, mas JA' NAO tem downcast nenhum"
            ));
        }
    }
    assert!(
        podres.is_empty(),
        "a DOWNCAST_ALLOWLIST tem entrada(s) obsoleta(s) — cada uma e' uma licenca que ninguem \
         usa e que esconde a proxima violacao:\n{}\n\nApague-as. A lista so' encolhe.",
        podres.join("\n")
    );
}

/// **Controle positivo:** a lista não está vazia, e o detector vê o que procura.
///
/// ⛔ Sem isto, apagar a lista inteira deixaria o censo acima **trivialmente verde** — a armadilha
/// do censo que mede zero e se lê como aprovado (HOWTO §2.7).
#[test]
fn the_allowlist_census_has_a_population() {
    assert!(
        DOWNCAST_ALLOWLIST.len() >= 4,
        "a allowlist tem {} entradas e esperava >= 4 — ou o censo perdeu o sujeito, ou alguem a \
         esvaziou (e ai' o gate acima deixou de afirmar seja o que for)",
        DOWNCAST_ALLOWLIST.len()
    );
}

#[test]
fn no_downcast_to_concrete_tool_in_non_allowlisted_files() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let mut files = Vec::new();
    collect_rs_files_recursive(&src_dir, &mut files);

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        // Compute path relative to crate root for allowlist check.
        let rel = path.strip_prefix(&crate_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if DOWNCAST_ALLOWLIST.iter().any(|a| *a == rel_str) {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Match patterns like `downcast_mut::<ph2d_tool_…::…>` or
            // `downcast_ref::<ph2d_tool_…::…>`. Generous match: any
            // downcast targeting a `ph2d_tool_*` path.
            if line.contains("downcast_mut::<ph2d_tool_")
                || line.contains("downcast_ref::<ph2d_tool_")
            {
                violations.push(format!("{}:{}: {}", rel_str, line_no + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Downcasts to concrete ph2d_tool_* types found in files NOT on \
         the allowlist:\n{}\n\n\
         If this is a legitimate tool-specific affordance, add the file\n\
         path to DOWNCAST_ALLOWLIST in this test with explicit\n\
         justification — and that's a Coord-A decision. Otherwise, route\n\
         through RasterEditTool / ph2d-tool-runtime helpers instead.",
        violations.join("\n")
    );
}
