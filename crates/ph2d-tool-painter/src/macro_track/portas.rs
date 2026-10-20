//! **O censo das portas do Painter** (gate 4 do plano): cada `pub fn …(&mut self` do [`PainterTool`] e
//! cada método `&mut self` dos contratos (`Tool`, `RasterEditTool`, `CanvasPaintTool`) tem UMA decisão.
//!
//! - [`Decisao::Grava`] — chamada de fora, grava o passo (o corpo passa por `porta!`).
//! - [`Decisao::Recusa`] — chamada de fora a gravar, deixa [`super::Passo::Recusada`] e o tocador
//!   recusa a macro em voz alta (o corpo chama `macro_recusa`).
//! - [`Decisao::Ambiente`] — não muda a obra (acessor do contrato, o dispositivo de quem pinta).
//! - [`INTERNAS`] — só o painel (`handle_panel_event`, gravado) ou o próprio tool lhes chegam; o gate
//!   `nenhuma_porta_interna_tem_chamador_de_fora` reprova a primeira chamada de fora.
//!
//! O gate `toda_porta_do_painter_tem_decisao` (`tests/it/macro_portas.rs`) varre o fonte: uma porta
//! nova sem decisão reprova, e uma decisão sem porta também.

pub use super::portas_internas::INTERNAS;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Decisao {
    Grava,
    Recusa(&'static str),
    Ambiente(&'static str),
}

#[derive(Copy, Clone, Debug)]
pub struct Porta {
    pub nome: &'static str,
    pub decisao: Decisao,
}

const fn g(nome: &'static str) -> Porta {
    Porta {
        nome,
        decisao: Decisao::Grava,
    }
}

const fn r(nome: &'static str, porque: &'static str) -> Porta {
    Porta {
        nome,
        decisao: Decisao::Recusa(porque),
    }
}

const fn a(nome: &'static str, porque: &'static str) -> Porta {
    Porta {
        nome,
        decisao: Decisao::Ambiente(porque),
    }
}

const OUTRO_DOCUMENTO: &str = "liga, carrega ou desliga outro documento: a macro toca sobre UM";
const CAMADAS_DE_FORA: &str =
    "mexe nas camadas por fora do painel (Hierarchy, juntar): o Wet Paint da W2 não o reproduz";
const FORMA_OU_SELECAO: &str =
    "ferramenta de forma, curva, colagem ou seleção: fora do Wet Paint (W2)";
const CAMADAS_COMO_PINCEL: &str =
    "lê as camadas como textura do pincel ou do papel: a macro não as guarda";
const ACESSOR: &str = "acessor do contrato: não muda nada";
const ETAPA_EDITADA: &str =
    "a edição de uma etapa chega à imagem: obra da própria macro, nunca um passo de outra";
const GRAVACAO: &str =
    "arma, drena ou pára a gravação (o painel MACROS): controla a macro, não é um passo da obra";

/// As portas de FORA, uma decisão cada.
pub const PORTAS: &[Porta] = &[
    // — os contratos —
    g("on_canvas_pointer"),
    g("on_tick"),
    g("handle_panel_event"),
    g("on_activate"),
    g("on_deactivate"),
    g("current_preview"),
    g("take_pending_commit"),
    g("run_full"),
    r("set_source", OUTRO_DOCUMENTO),
    r("deactivate", OUTRO_DOCUMENTO),
    a("as_any_mut", ACESSOR),
    a("as_raster_edit_mut", ACESSOR),
    a("as_canvas_paint_mut", ACESSOR),
    a("macro_abre", GRAVACAO),
    a("macro_passos", GRAVACAO),
    a("macro_para", GRAVACAO),
    a("agua_parada", ACESSOR),
    // — inerentes, gravadas —
    g("on_canvas_hover"),
    g("undo_last"),
    g("redo_last"),
    g("request_commit"),
    g("mark_baked"),
    g("take_deferred_bake"),
    g("take_preview_arc"),
    g("take_preview_upload_bbox"),
    g("take_preview_dirty"),
    g("set_brush_falloff_point"),
    g("add_brush_falloff_point_at"),
    g("set_brush_falloff_point_handle"),
    g("remove_brush_falloff_point"),
    g("toggle_brush_eraser"),
    g("set_brush_size_px"),
    g("nudge_brush_size"),
    g("set_brush_color_srgb8"),
    g("set_line_constrain"),
    g("set_shape_grab_tol_px"),
    g("set_canvas_warp"),
    g("set_compor_por_quadro"),
    g("set_grid_snap"),
    g("set_grid_stamp_erase"),
    g("set_shape_draft_hold"),
    g("set_gizmo_modifiers"),
    g("set_brush_texture_image"),
    g("take_brush_texture_image_request"),
    g("set_brush_shape_image_rgba"),
    g("take_brush_shape_image_request"),
    g("refresh_shape_source_if_changed"),
    g("refresh_shape_color_preview"),
    g("take_shape_source_preview"),
    g("take_wet_preset_request"),
    g("wet_preset_aplicar"),
    g("wet_preset_aplicar_volumes"),
    g("wet_preset_aplicar_modo"),
    g("set_brush_texture_kind"),
    g("set_brush_shape_kind"),
    g("set_line_snap"),
    g("set_uniform_scale"),
    // o ColorDrop do C&F (W11): o preenchimento é determinístico dado o estado (`tool/paint/fill.rs`)
    g("begin_colordrop_fill"),
    g("set_fill_threshold"),
    g("fill_commit"),
    g("restore_after_colordrop"),
    // — inerentes, recusadas —
    r("bind_document", OUTRO_DOCUMENTO),
    r("reescreve_a_gravacao", ETAPA_EDITADA),
    a(
        "poe_a_marca",
        "a marca do artista: quem mostra a macro em cena dá-a ao Painter; não muda a obra (W9)",
    ),
    r("install_document", OUTRO_DOCUMENTO),
    r("baked_document_pixels", OUTRO_DOCUMENTO),
    r("open_layer_of", OUTRO_DOCUMENTO),
    r("forget_pending_layer", OUTRO_DOCUMENTO),
    r("set_layer_name", CAMADAS_DE_FORA),
    r("handle_layer_reparent", CAMADAS_DE_FORA),
    r("add_raster_layer_with_pixels", CAMADAS_DE_FORA),
    r("add_mask_to_active", CAMADAS_DE_FORA),
    r("curve_delete_selected", FORMA_OU_SELECAO),
    r("set_curve_handle_kind", FORMA_OU_SELECAO),
    r("commit_open_shape", FORMA_OU_SELECAO),
    r("cancel_open_shape", FORMA_OU_SELECAO),
    r("curve_select_point_at", FORMA_OU_SELECAO),
    r("line_finish_points", FORMA_OU_SELECAO),
    r("paste_commit", FORMA_OU_SELECAO),
    r("paste_cancel", FORMA_OU_SELECAO),
    r("clear_selection", FORMA_OU_SELECAO),
    r("selection_select_all", FORMA_OU_SELECAO),
    r("selection_cut", FORMA_OU_SELECAO),
    r("selection_copy", FORMA_OU_SELECAO),
    r("selection_paste", FORMA_OU_SELECAO),
    r("selection_curve_select_point_at", FORMA_OU_SELECAO),
    r("set_selection_curve_handle_kind", FORMA_OU_SELECAO),
    r("selection_curve_delete_selected_point", FORMA_OU_SELECAO),
    r("invert_selection", FORMA_OU_SELECAO),
    r("selection_offset_apply", FORMA_OU_SELECAO),
    r("selection_pen_commit", FORMA_OU_SELECAO),
    r("selection_pen_cancel", FORMA_OU_SELECAO),
    r("capture_layers_as_brush_shape", CAMADAS_COMO_PINCEL),
    r("use_layers_as", CAMADAS_COMO_PINCEL),
    // — ambiente —
    a(
        "set_device_stamp",
        "o carimbo na placa é da máquina de quem pinta, e o Wet Paint deposita pelo motor de fluido",
    ),
];

/// As decisões como o núcleo as conta ([`ph2d_macro::undecided`] / [`ph2d_macro::orphans`]).
#[must_use]
pub fn como_portas_do_nucleo() -> Vec<ph2d_macro::Door> {
    PORTAS
        .iter()
        .map(|p| ph2d_macro::Door {
            name: p.nome,
            recordability: match p.decisao {
                Decisao::Grava => ph2d_macro::Recordability::Recorded,
                Decisao::Recusa(why) | Decisao::Ambiente(why) => {
                    ph2d_macro::Recordability::Refused { why }
                }
            },
        })
        .chain(INTERNAS.iter().map(|&name| ph2d_macro::Door {
            name,
            recordability: ph2d_macro::Recordability::Refused {
                why: "interna: o painel chega-lhe pelo handle_panel_event, que grava",
            },
        }))
        .collect()
}
