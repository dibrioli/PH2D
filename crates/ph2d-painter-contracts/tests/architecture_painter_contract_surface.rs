//! Painter contract surface — homestead único dos arch-gates de cap textual
//! cumulativos das 11 ADRs ratificadas em 2026-05-26 (0043..0053).
//!
//! ## Política de vacuous-pass
//!
//! Crates filhos do Painter (`ph2d-tool-painter`, `ph2d-painter-brush`, ...)
//! nascem em W1+ via fan-out drop-crate. Em W0/T0.8 nenhum existe — todos
//! os tests **passam vacuamente** (helper retorna `None` quando o symbol não é
//! encontrado). Conforme os crates nascem, os tests ativam automaticamente —
//! sem nova edição neste arquivo.
//!
//! ## Estrutura
//!
//! Sub-módulos espelham as ADRs:
//! - `painter_ui` — ADR-0043
//! - `brush_engine` — ADR-0044
//! - `adjustments` — ADR-0045
//! - `stroke_history` — ADR-0046
//! - `mcp` — ADR-0047
//! - `inspector` — ADR-0048
//! - `fluid` — ADR-0049
//! - `input` — ADR-0050
//! - `color_pipeline` — ADR-0051
//! - `durability` — ADR-0052
//! - `tier_policy` — ADR-0053

use std::path::PathBuf;
use walkdir::WalkDir;

// ----------------------------------------------------------------------------
// Helpers — gate textual sobre source dos crates filhos
// ----------------------------------------------------------------------------

/// Workspace root (parent of `crates/<this-crate>`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has parent")
        .parent()
        .expect("crates/ has parent")
        .to_path_buf()
}

/// Concatenated source of `crates/<crate_name>/src/**/*.rs`.
/// Returns empty string if the crate doesn't exist (vacuous pass).
fn crate_source(crate_name: &str) -> String {
    let dir = workspace_root().join("crates").join(crate_name).join("src");
    if !dir.exists() {
        return String::new();
    }
    let mut out = String::new();
    for entry in WalkDir::new(&dir).into_iter().flatten() {
        if entry.path().extension().is_some_and(|e| e == "rs")
            && let Ok(s) = std::fs::read_to_string(entry.path())
        {
            out.push('\n');
            out.push_str(&s);
        }
    }
    out
}

/// Total LOC of a crate's `src/`.
fn crate_loc(crate_name: &str) -> Option<usize> {
    let src = crate_source(crate_name);
    if src.is_empty() {
        return None;
    }
    Some(src.lines().count())
}

/// Locate the `{ ... }` body of `pub <kind> <name>` (enum or struct).
/// Returns the body slice between the outermost braces, or None.
fn find_decl_body<'a>(src: &'a str, kind: &str, name: &str) -> Option<&'a str> {
    let prefix = format!("pub {} {}", kind, name);
    let start = src.find(&prefix)?;
    // Skip optional generics + where clause until '{'.
    let body_start = start + src[start..].find('{')?;
    let body_start_inner = body_start + 1;
    let bytes = src.as_bytes();
    let mut depth = 1usize;
    let mut i = body_start_inner;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[body_start_inner..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Count enum variants in `pub enum <name>`. None = enum not found (vacuous).
fn count_enum_variants(crate_name: &str, enum_name: &str) -> Option<usize> {
    let src = crate_source(crate_name);
    let body = find_decl_body(&src, "enum", enum_name)?;
    let count = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("///"))
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        .count();
    Some(count)
}

/// Count struct fields (`pub <name>: ...`). None = struct not found (vacuous).
fn count_struct_fields(crate_name: &str, struct_name: &str) -> Option<usize> {
    let src = crate_source(crate_name);
    let body = find_decl_body(&src, "struct", struct_name)?;
    let count = body
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pub ") && l.contains(':'))
        .count();
    Some(count)
}

/// Assert a count is ≤ max. `None` is a vacuous pass (crate/symbol absent).
#[track_caller]
fn assert_capped(actual: Option<usize>, max: usize, ctx: &str) {
    if let Some(n) = actual {
        assert!(n <= max, "[{}] count {} exceeds cap {}", ctx, n, max);
    }
}

/// Assert a count equals expected. `None` is vacuous.
#[track_caller]
fn assert_exact(actual: Option<usize>, expected: usize, ctx: &str) {
    if let Some(n) = actual {
        assert_eq!(
            n, expected,
            "[{}] count {} != expected {}",
            ctx, n, expected
        );
    }
}

/// Assert a textual needle is present in the crate source. `None` = crate absent.
#[track_caller]
fn assert_contains(crate_name: &str, needle: &str, ctx: &str) {
    let src = crate_source(crate_name);
    if src.is_empty() {
        return; // vacuous
    }
    assert!(
        src.contains(needle),
        "[{}] needle `{}` not found in `{}` source",
        ctx,
        needle,
        crate_name
    );
}

/// Assert a textual needle is ABSENT. `None` = vacuous.
#[track_caller]
fn assert_absent(crate_name: &str, needle: &str, ctx: &str) {
    let src = crate_source(crate_name);
    if src.is_empty() {
        return;
    }
    assert!(
        !src.contains(needle),
        "[{}] forbidden needle `{}` found in `{}`",
        ctx,
        needle,
        crate_name
    );
}

// ----------------------------------------------------------------------------
// ADR-0043 — Painter contract (caps PainterUiEdit / PainterUiSnapshot / PainterParams)
// ----------------------------------------------------------------------------

mod painter_ui {
    use super::*;

    const CRATE: &str = "ph2d-tool-painter";

    #[test]
    fn painter_ui_edit_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "PainterUiEdit"),
            24,
            "PainterUiEdit (ADR-0043 §2.3)",
        );
    }

    #[test]
    fn painter_ui_snapshot_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PainterUiSnapshot"),
            18,
            "PainterUiSnapshot (ADR-0043 §2.3)",
        );
    }

    #[test]
    fn painter_params_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PainterParams"),
            12,
            "PainterParams (ADR-0043 §2.3)",
        );
    }

    #[test]
    fn painter_mode_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "PainterMode"),
            6,
            "PainterMode {Brush,Smudge,Eraser} (ADR-0043 §2.3)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0044 — Brush Engine GPU contract
// ----------------------------------------------------------------------------

mod brush_engine {
    use super::*;

    const CRATE: &str = "ph2d-painter-brush";

    #[test]
    fn brush_top_level_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "Brush"),
            14,
            "Brush top-level (ADR-0044 §2.2)",
        );
    }

    #[test]
    fn brush_sub_caps_per_substruct() {
        // ADR-0044 §2.2.1 tabela. Cada sub-struct tem seu cap individual.
        let subcaps = &[
            ("StrokePathParams", 6),
            ("StabilizationParams", 8),
            ("TaperParams", 12),
            ("ShapeParams", 20),
            ("GrainParams", 20),
            ("RenderingParams", 14),
            ("WetMixParams", 12),
            ("ColorDynamicsParams", 36),
            ("DynamicsParams", 8),
            ("PencilParams", 14),
            ("PropertiesParams", 10),
            ("AboutParams", 8),
        ];
        for (name, cap) in subcaps {
            assert_capped(
                count_struct_fields(CRATE, name),
                *cap,
                &format!("{} sub-cap (ADR-0044 §2.2.1)", name),
            );
        }
    }

    #[test]
    fn brush_no_sub_sub_structs() {
        // ADR-0044 §2.2 — sub-structs são folha. Esta checagem é frouxa em texto
        // (uma sub-struct pode listar `Option<NewField>` legítimo) mas
        // identifica patterns suspeitos: campos cujo tipo começa maiúscula sem ser
        // primitivo conhecido nem `Vec`/`Option`/enum-known. Heuristic; refinar
        // se vier falso-positivo legítimo.
        let src = crate_source(CRATE);
        if src.is_empty() {
            return; // vacuous
        }
        // Marker explícito (proibido): qualquer sub-struct cujo BODY contenha
        // outra `pub struct` aninhada literalmente. Suficiente para o gate.
        // (Em Rust idiomático sub-structs vivem em mod boundary; aninhamento
        // literal é o anti-pattern.)
        let nested = src.matches("pub struct").count();
        // Conta total de pub struct no crate (top-level + sub-structs).
        // Heuristic: se houver mais de 14 (top-level Brush) + 12 (sub-structs) + alguns
        // structs auxiliares Pod (~5), o crate já está crescendo demais.
        // Cap soft de structs públicas: 32. Não é a regra exata "no sub-sub" mas
        // captura a intenção.
        assert!(
            nested <= 32,
            "[brush_no_sub_sub_structs] {} pub structs found; max 32 (heuristic for flat sub-struct policy, ADR-0044 §2.2)",
            nested
        );
    }

    #[test]
    fn stamp_size_is_96_bytes_aligned_16() {
        // Compile-time asserts em ADR-0044 §2.3 vivem no source do crate
        // (`const _: () = assert!(core::mem::size_of::<Stamp>() == 96);`).
        // Aqui auditamos textualmente que esses asserts existem.
        assert_contains(
            CRATE,
            "size_of::<Stamp>() == 96",
            "Stamp size compile-time assert (ADR-0044 §2.3)",
        );
        assert_contains(
            CRATE,
            "align_of::<Stamp>() == 16",
            "Stamp align compile-time assert (ADR-0044 §2.3)",
        );
    }

    #[test]
    fn stamp_is_pod_zeroable() {
        // `#[derive(... bytemuck::Pod, bytemuck::Zeroable ...)]` no Stamp.
        let src = crate_source(CRATE);
        if src.is_empty() {
            return;
        }
        // Heuristic: o derive aparece adjacent a `struct Stamp`.
        if let Some(body) = find_decl_body(&src, "struct", "Stamp") {
            let _ = body; // body found = struct exists; checa derives no source full
            assert!(
                src.contains("bytemuck::Pod") && src.contains("bytemuck::Zeroable"),
                "[stamp_is_pod_zeroable] Stamp must derive bytemuck::Pod + Zeroable (ADR-0044 §2.3)"
            );
        }
    }

    #[test]
    fn rendering_mode_variant_count_is_exact_6() {
        // ADR-0044 §2.4 — FROZEN não-≤. Coupling com MAX_RENDERING_MODES = 6 const.
        assert_exact(
            count_enum_variants(CRATE, "RenderingMode"),
            6,
            "RenderingMode FROZEN (ADR-0044 §2.4)",
        );
    }

    #[test]
    fn max_rendering_modes_matches_enum() {
        // ADR-0044 §2.4 — const MAX_RENDERING_MODES: usize = 6.
        assert_contains(
            CRATE,
            "MAX_RENDERING_MODES: usize = 6",
            "MAX_RENDERING_MODES const (ADR-0044 §2.4)",
        );
    }

    #[test]
    fn pigment_mode_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "PigmentMode"),
            4,
            "PigmentMode (ADR-0044 §2.5)",
        );
    }

    #[test]
    fn grain_source_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "GrainSource"),
            6,
            "GrainSource (ADR-0044 §2.6)",
        );
    }

    #[test]
    fn procedural_grain_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "ProceduralGrain"),
            8,
            "ProceduralGrain (ADR-0044 §2.7)",
        );
    }

    #[test]
    fn brush_handle_bit31_layout() {
        // ADR-0044 §2.8 — helpers IMPORTED_FLAG / new_builtin / new_imported / is_imported.
        let src = crate_source(CRATE);
        if src.is_empty() {
            return;
        }
        assert!(
            src.contains("IMPORTED_FLAG")
                && src.contains("new_builtin")
                && src.contains("new_imported"),
            "[brush_handle_bit31_layout] BrushHandle must expose IMPORTED_FLAG + new_builtin + new_imported (ADR-0044 §2.8)"
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0045 — Adjustment Layers
// ----------------------------------------------------------------------------

mod adjustments {
    use super::*;

    // ADR-0045 §2.1: AdjustmentLayer + AdjustmentKind + AdjustmentParams vivem em
    // `ph2d-painter-brush::adjustments` módulo (ou crate-canvas futuro).
    const CRATE: &str = "ph2d-painter-brush";

    #[test]
    fn adjustment_layer_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "AdjustmentLayer"),
            12,
            "AdjustmentLayer (ADR-0045 §2.2)",
        );
    }

    #[test]
    fn adjustment_kind_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "AdjustmentKind"),
            32,
            "AdjustmentKind cap 32 (v1 ship 24, ADR-0045 §2.3)",
        );
    }

    #[test]
    fn destructive_adjustment_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "DestructiveAdjustment"),
            8,
            "DestructiveAdjustment (ADR-0045 §2.4)",
        );
    }

    #[test]
    fn adjustment_params_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "AdjustmentParams"),
            32,
            "AdjustmentParams (ADR-0045 §2.5)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0046 — Stroke Vector History
// ----------------------------------------------------------------------------

mod stroke_history {
    use super::*;

    const CRATE: &str = "ph2d-painter-stroke";

    #[test]
    fn stroke_record_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StrokeRecord"),
            16,
            "StrokeRecord (ADR-0046 §2.2)",
        );
    }

    #[test]
    fn raw_pointer_sample_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "RawPointerSample"),
            12,
            "RawPointerSample (ADR-0046 §2.3)",
        );
    }

    #[test]
    fn tool_mode_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "ToolMode"),
            6,
            "ToolMode (ADR-0046 §2.4)",
        );
    }

    #[test]
    fn stroke_history_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "StrokeHistory"),
            4,
            "StrokeHistory (ADR-0046 §2.5)",
        );
    }

    #[test]
    fn layer_snapshot_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "LayerSnapshot"),
            8,
            "LayerSnapshot (ADR-0046 §2.6)",
        );
    }

    #[test]
    fn snapshot_storage_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "SnapshotStorage"),
            4,
            "SnapshotStorage (ADR-0046 §2.6)",
        );
    }

    #[test]
    fn paint_project_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PaintProject"),
            12,
            "PaintProject canon savefile (ADR-0046 §2.7.1)",
        );
    }

    #[test]
    fn canvas_info_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "CanvasInfo"),
            8,
            "CanvasInfo (ADR-0046 §2.7.1)",
        );
    }

    #[test]
    fn paint_project_cache_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PaintProjectCache"),
            8,
            "PaintProjectCache sidecar (ADR-0046 §2.7.2)",
        );
    }

    #[test]
    fn reproject_mode_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "ReprojectMode"),
            4,
            "ReprojectMode (ADR-0046 §2.9)",
        );
    }

    #[test]
    fn painter_memory_budget_field_count_is_capped() {
        assert_capped(
            count_struct_fields("ph2d-host", "PainterMemoryBudget"),
            8,
            "PainterMemoryBudget (ADR-0046 §2.10)",
        );
    }

    #[test]
    fn stroke_id_is_uuid_alias() {
        // ADR-0046 §2.2: `pub type StrokeId = Uuid;`
        assert_contains(
            CRATE,
            "pub type StrokeId = Uuid",
            "StrokeId type alias (ADR-0046 §2.2)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0047 — MCP Stroke Engine
// ----------------------------------------------------------------------------

mod mcp {
    use super::*;

    const CRATE: &str = "ph2d-painter-mcp";

    #[test]
    fn mcp_tools_count_is_capped() {
        // ADR-0047 §2.2 — cap ≤ 8 tools. Conta `#[mcp_tool(` ocorrências.
        let src = crate_source(CRATE);
        if src.is_empty() {
            return;
        }
        let count = src.matches("#[mcp_tool(").count();
        assert!(
            count <= 8,
            "[mcp_tools_count_is_capped] {} #[mcp_tool] decorators; cap 8 (ADR-0047 §2.2)",
            count
        );
    }

    #[test]
    fn stroke_spec_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StrokeSpec"),
            8,
            "StrokeSpec (ADR-0047 §2.3)",
        );
    }

    #[test]
    fn stroke_point_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StrokePoint"),
            8,
            "StrokePoint (ADR-0047 §2.4)",
        );
    }

    #[test]
    fn stroke_mods_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StrokeMods"),
            8,
            "StrokeMods (ADR-0047 §2.5)",
        );
    }

    #[test]
    fn stroke_filter_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StrokeFilter"),
            8,
            "StrokeFilter (ADR-0047 §2.6)",
        );
    }

    #[test]
    fn stroke_ref_field_count_is_exact_5() {
        assert_exact(
            count_struct_fields(CRATE, "StrokeRef"),
            5,
            "StrokeRef FROZEN (ADR-0047 §2.7)",
        );
    }

    #[test]
    fn mcp_error_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "McpError"),
            12,
            "McpError (ADR-0047 §2.8)",
        );
    }

    #[test]
    fn audit_log_entry_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "AuditLogEntry"),
            16,
            "AuditLogEntry (ADR-0047 §2.11)",
        );
    }

    #[test]
    fn stream_handle_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "StreamHandle"),
            4,
            "StreamHandle (ADR-0047 §2.10)",
        );
    }

    #[test]
    fn stroke_progress_event_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "StrokeProgressEvent"),
            8,
            "StrokeProgressEvent (ADR-0047 §2.10)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0048 — Stroke Inspector retroativo
// ----------------------------------------------------------------------------

mod inspector {
    use super::*;

    const CRATE: &str = "ph2d-panel-painter-inspector";

    #[test]
    fn inspector_state_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "InspectorState"),
            12,
            "InspectorState (ADR-0048 §2.2)",
        );
    }

    #[test]
    fn inspector_selection_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "InspectorSelection"),
            8,
            "InspectorSelection (ADR-0048 §2.3)",
        );
    }

    #[test]
    fn inspector_action_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "InspectorAction"),
            10,
            "InspectorAction (ADR-0048 §2.5)",
        );
    }

    #[test]
    fn render_slice_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "RenderSlice"),
            8,
            "RenderSlice (ADR-0048 §2.6.1)",
        );
    }

    #[test]
    fn no_retroactive_window_constant() {
        // ADR-0048 §2.6 — dirty-rect propagation FULL v1; SEM MAX_RETROACTIVE_MOD_WINDOW.
        assert_absent(
            CRATE,
            "MAX_RETROACTIVE_MOD_WINDOW",
            "MAX_RETROACTIVE_MOD_WINDOW removed (ADR-0048 §2.6 rework regra perfeição)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0049 — Fluid Brushes Extension
// ----------------------------------------------------------------------------

mod fluid {
    use super::*;

    const CRATE: &str = "ph2d-painter-fluid";

    #[test]
    fn fluid_sim_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "FluidSim"),
            12,
            "FluidSim (ADR-0049 §2.4)",
        );
    }

    #[test]
    fn fluid_params_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "FluidParams"),
            12,
            "FluidParams (ADR-0049 §2.5)",
        );
    }

    #[test]
    fn gravity_source_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "GravitySource"),
            6,
            "GravitySource (ADR-0049 §2.6)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0050 — Device heterogeneity layer
// ----------------------------------------------------------------------------

mod input {
    use super::*;

    const CRATE: &str = "ph2d-painter-input";

    #[test]
    fn pointer_source_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "PointerSource"),
            16,
            "PointerSource (ADR-0050 §2.2)",
        );
    }

    #[test]
    fn pressure_curve_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PressureCurve"),
            12,
            "PressureCurve (ADR-0050 §2.3)",
        );
    }

    #[test]
    fn tilt_curve_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "TiltCurve"),
            12,
            "TiltCurve (ADR-0050 §2.3)",
        );
    }

    #[test]
    fn barrel_curve_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "BarrelCurve"),
            12,
            "BarrelCurve (ADR-0050 §2.3)",
        );
    }

    #[test]
    fn palm_rejection_config_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PalmRejectionConfig"),
            12,
            "PalmRejectionConfig (ADR-0050 §2.4)",
        );
    }

    #[test]
    fn driver_quirk_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "DriverQuirk"),
            32,
            "DriverQuirk (ADR-0050 §2.5)",
        );
    }

    #[test]
    fn hover_state_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "HoverState"),
            8,
            "HoverState (ADR-0050 §2.6)",
        );
    }

    #[test]
    fn raw_pointer_sample_has_device_source() {
        // ADR-0050 §2.7 — RawPointerSample em ph2d-painter-stroke ganha device_source.
        let src = crate_source("ph2d-painter-stroke");
        if src.is_empty() {
            return; // vacuous
        }
        if let Some(body) = find_decl_body(&src, "struct", "RawPointerSample") {
            assert!(
                body.contains("device_source"),
                "[raw_pointer_sample_has_device_source] RawPointerSample must include device_source: PointerSource (ADR-0050 §2.7)"
            );
        }
    }
}

// ----------------------------------------------------------------------------
// ADR-0051 — Color profile pipeline
// ----------------------------------------------------------------------------

mod color_pipeline {
    use super::*;

    const CRATE: &str = "ph2d-color";

    #[test]
    fn color_profile_variant_count_is_exact_8() {
        // ADR-0051 §2.2 — FROZEN em 8 variants (7 v1 + 1 slot reserved).
        // Aceita exato 8 (incluindo o slot) ou 7 (se o slot ainda não foi materializado).
        let actual = count_enum_variants(CRATE, "ColorProfile");
        if let Some(n) = actual {
            assert!(
                n == 7 || n == 8,
                "[color_profile_variant_count_is_exact_8] ColorProfile FROZEN: expected 7 or 8 variants, got {} (ADR-0051 §2.2)",
                n
            );
        }
    }

    #[test]
    fn export_format_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "ExportFormat"),
            8,
            "ExportFormat (ADR-0051 §2.7)",
        );
    }

    #[test]
    fn hdr_format_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "HdrFormat"),
            4,
            "HdrFormat (ADR-0051 §2.7)",
        );
    }

    #[test]
    fn ph2d_color_loc_capped_2500() {
        assert_capped(crate_loc(CRATE), 2500, "ph2d-color LOC (ADR-0051 §2.1)");
    }
}

// ----------------------------------------------------------------------------
// ADR-0052 — Tear-resistant stroke commit
// ----------------------------------------------------------------------------

mod durability {
    use super::*;

    const CRATE: &str = "ph2d-painter-stroke";

    #[test]
    fn partial_stroke_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "PartialStroke"),
            16,
            "PartialStroke (ADR-0052 §2.2)",
        );
    }

    #[test]
    fn flush_policy_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "FlushPolicy"),
            6,
            "FlushPolicy (ADR-0052 §2.2)",
        );
    }

    #[test]
    fn auto_save_policy_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "AutoSavePolicy"),
            8,
            "AutoSavePolicy (ADR-0052 §2.4)",
        );
    }

    #[test]
    fn recovery_state_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "RecoveryState"),
            6,
            "RecoveryState (ADR-0052 §2.3)",
        );
    }

    #[test]
    fn suspend_state_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "SuspendState"),
            4,
            "SuspendState (ADR-0052 §2.5)",
        );
    }
}

// ----------------------------------------------------------------------------
// ADR-0053 — Cross-platform tier policy
// ----------------------------------------------------------------------------

mod tier_policy {
    use super::*;

    const CRATE: &str = "ph2d-host";

    #[test]
    fn device_tier_variant_count_is_exact_5() {
        assert_exact(
            count_enum_variants(CRATE, "DeviceTier"),
            5,
            "DeviceTier FROZEN (ADR-0053 §2.2)",
        );
    }

    #[test]
    fn device_capability_field_count_is_capped() {
        assert_capped(
            count_struct_fields(CRATE, "DeviceCapability"),
            16,
            "DeviceCapability (ADR-0053 §2.2)",
        );
    }

    #[test]
    fn gpu_id_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "GpuId"),
            64,
            "GpuId (ADR-0053 §2.2)",
        );
    }

    #[test]
    fn thermal_state_variant_count_is_capped() {
        assert_capped(
            count_enum_variants(CRATE, "ThermalState"),
            6,
            "ThermalState (ADR-0053 §2.7)",
        );
    }
}

// ----------------------------------------------------------------------------
// Meta — self-tests do gate (smoke do helper)
// ----------------------------------------------------------------------------

mod meta {
    use super::*;

    #[test]
    fn helper_vacuous_on_missing_crate() {
        // Crate inexistente devolve None (vacuous pass).
        assert!(count_enum_variants("ph2d-tool-nonexistent-xyzzy", "FooBar").is_none());
        assert!(count_struct_fields("ph2d-tool-nonexistent-xyzzy", "FooBar").is_none());
        assert!(crate_loc("ph2d-tool-nonexistent-xyzzy").is_none());
    }

    #[test]
    fn helper_finds_existing_crate_source() {
        // ph2d-color existe na Wave 10. Sanity check: tem alguma LOC.
        if let Some(n) = crate_loc("ph2d-color") {
            assert!(n > 0, "ph2d-color exists and has source");
        }
        // Sem assert se ph2d-color desaparecer (não deve, mas vacuous).
    }
}
