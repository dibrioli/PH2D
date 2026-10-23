//! ⭐ **O censo do preview**: cada campo do estado do pincel vai para o preview ou diz porque não vai (irmão de
//! `preview_paridade_tests`, que prova AO BYTE que o que vai pinta como a tela).

// ── O censo: nenhum campo do estado do pincel nasce fora do preview ─────────────────────────────────────

const TRACO: &str = "o traço EM CURSO (nasce e morre no gesto)";
const SESSAO: &str = "a sessão molhada da obra (o tool do preview abre a sua)";
const CACHE: &str = "cache ou versão (o tool do preview faz as suas)";
const UI: &str = "exibição ou pedido da interface, não pinta";
const SELECAO: &str = "a seleção e o colar da OBRA (o preview pinta a tela inteira)";
const METODO: &str = "o método de traço e os editores de forma (o preview é o traço à mão livre)";
const OUTRA: &str = "outra ferramenta (Clone, Fill, Inpaint, Deform, Warp, Sculpt, Mask)";
const OBRA: &str = "propriedade da obra que o preview não mostra (tiling, dobra, simetria)";

/// Os campos do `PaintState` que o preview NÃO leva, cada um com o porquê.
const FORA_DO_PREVIEW: &[(&str, &str)] = &[
    ("stroke", TRACO),
    ("hover_heading", UI),
    ("hover_pos", UI),
    ("dabs", TRACO),
    ("solid_path", TRACO),
    ("solid_fill_owed", TRACO),
    ("pending_threads", TRACO),
    (
        "seed",
        "o sorteio: o tool do preview semeia o seu — o mesmo pincel dá os mesmos bytes",
    ),
    ("tex_rng", "o sorteio do grão, idem"),
    ("stroke_undo", TRACO),
    (
        "tracos_fora",
        "o contador do tocador da macro (traços abertos fora do tool): o preview não toca macros",
    ),
    (
        "eraser",
        "o preview mostra o que o pincel DEPOSITA (posto a false)",
    ),
    (
        "paint_mode",
        "o preview pinta o PINCEL do modo (como a ferramenta molhada)",
    ),
    ("brush_by_mode", "os pincéis dos OUTROS modos"),
    ("link_shared_settings", UI),
    ("line_show_dimensions", UI),
    ("mask_brush", OUTRA),
    ("mask_overlay_color", UI),
    ("eyedropper_armed", UI),
    ("falloff_armed", UI),
    ("mask_scratch_rgba", OUTRA),
    ("mask_scratch_target", OUTRA),
    ("selection_mask", SELECAO),
    ("selection_active", SELECAO),
    ("selection_mode", SELECAO),
    ("selection_bool_op", SELECAO),
    ("selection_threshold", SELECAO),
    ("selection_drag", SELECAO),
    ("selection_pen", SELECAO),
    ("selection_base", SELECAO),
    ("selection_crisp", SELECAO),
    ("selection_stabilizer", SELECAO),
    ("selection_feather", SELECAO),
    ("selection_edit_mode", SELECAO),
    ("selection_overlay_opacity", SELECAO),
    ("selection_shapes", SELECAO),
    ("selection_raster_cache", SELECAO),
    ("selection_grab", SELECAO),
    ("selection_clipboard", SELECAO),
    ("paste_patch", SELECAO),
    ("paste_pristine", SELECAO),
    ("paste_grab", SELECAO),
    ("selection_offset_norm", SELECAO),
    ("selection_offset_active", SELECAO),
    ("selection_offset_rings", SELECAO),
    ("selection_offset_source", SELECAO),
    ("selection_offset_curves", SELECAO),
    ("selection_offset_level_cache", SELECAO),
    ("selection_ring_stack", SELECAO),
    ("selection_op_tap", SELECAO),
    ("composite_add_op", UI),
    ("composite_mask", TRACO),
    ("composite_arco", TRACO),
    ("pilha", TRACO),
    ("limite_do_smear", TRACO),
    ("rng_camada", TRACO),
    ("pilha_por_replay", "porta de bissecção"),
    ("acumulando_no_plano", TRACO),
    ("clone_source", OUTRA),
    ("clone_offset", OUTRA),
    ("clone_aligned", OUTRA),
    ("clone_sample_armed", OUTRA),
    ("last_smear_pos", TRACO),
    ("smear_scratch", TRACO),
    ("tiling", OBRA),
    ("repeat_image", OBRA),
    ("symmetry_pick", UI),
    ("symmetry_line_start", UI),
    ("symmetry_auto_center", OBRA),
    ("moved_this_frame", TRACO),
    ("drag_preview", TRACO),
    ("shape_draft", METODO),
    ("shape_draft_hold", METODO),
    ("shape_stale", METODO),
    ("line_anchor", METODO),
    ("line_constrain", METODO),
    ("scale_uniform", METODO),
    ("line_snap", METODO),
    ("grid_snap_pos", METODO),
    ("last_non_shape_method", METODO),
    ("method_by_media", METODO),
    ("curve", METODO),
    ("ellipse", METODO),
    ("line", METODO),
    ("polygon", METODO),
    ("parked_shapes", METODO),
    ("active_op", METODO),
    ("stroke_op_mode", METODO),
    ("op_tap", METODO),
    ("shape_edit_wrap", METODO),
    ("gizmo_mods", METODO),
    ("shape_grab_tol_px", METODO),
    ("canvas_warp", OBRA),
    ("stroke_authored", TRACO),
    ("shape_offset_norm", METODO),
    ("shape_offset_base_px", METODO),
    ("offset_trim", METODO),
    ("stencil_grab", UI),
    ("stencil_preview_s", UI),
    ("paper_image_version", CACHE),
    ("texture_image_pending", UI),
    ("texture_image_version", CACHE),
    ("shape_image_pending", UI),
    ("shape_image_version", CACHE),
    ("stamp_cache", CACHE),
    ("color_stamp_cache", CACHE),
    ("ramp_color_stamp_cache", CACHE),
    ("canvas_tex_cache", CACHE),
    ("texture_ramp_lut", CACHE),
    ("texture_ramp_dirty", CACHE),
    ("ramp_lut_version", CACHE),
    ("shape_ramp_lut", CACHE),
    ("shape_ramp_dirty", CACHE),
    ("shape_ramp_version", CACHE),
    ("ramp_lut_owner", CACHE),
    ("stroke_mask", TRACO),
    ("plano_do_traco", TRACO),
    ("pixel_trilhos", TRACO),
    ("relief", SESSAO),
    ("stroke_coverage", TRACO),
    ("stroke_color", TRACO),
    ("wet_mistura", TRACO),
    ("watercolor_base", TRACO),
    ("wet_backdrop", TRACO),
    ("grid_show", UI),
    ("grid_erase", TRACO),
    ("impasto_live_edit", UI),
    ("wet_soak", SESSAO),
    ("wet_soak_pos", SESSAO),
    ("wet_soak_active", SESSAO),
    ("stroke_density", TRACO),
    ("stroke_deplete", TRACO),
    ("stroke_deplete_prox", TRACO),
    ("wet_reserve_cache", CACHE),
    ("canvas_wet", SESSAO),
    ("canvas_wet_rect", SESSAO),
    ("canvas_wet_snapshot", SESSAO),
    ("canvas_wet_carry", SESSAO),
    ("wet_preview_intensity", UI),
    (
        "wet_session_wetness",
        "o «Wet the layer» molha a OBRA (um gesto, não o pincel)",
    ),
    ("wet_shape_active", METODO),
    ("wet_styles", SESSAO),
    ("stroke_water", SESSAO),
    ("wet_session_base", SESSAO),
    ("wet_session_canvas", SESSAO),
    ("wet_editable_base", SESSAO),
    ("wet_editable_backdrop", SESSAO),
    ("wet_editable_region", SESSAO),
    ("wet_editable_tex", SESSAO),
    ("wet_shape_norm", TRACO),
    ("wet_substrate", CACHE),
    ("wet_mix", TRACO),
    ("wet_smear_pos", TRACO),
    ("wet_level_smear_pos", TRACO),
    ("wet_frame_dirty", TRACO),
    ("wet_frame_linhas", TRACO),
    ("wet_cum_dirty", TRACO),
    ("wet_stroke_dirty", TRACO),
    ("inpaint_mask", OUTRA),
    ("fill_threshold", OUTRA),
    ("fill_seed", OUTRA),
    ("fill_snapshot", OUTRA),
    ("fill_last_rect", OUTRA),
    ("fill_return_mode", OUTRA),
    ("inpaint_patch_norm", OUTRA),
    ("inpaint_quality_norm", OUTRA),
    ("inpaint_search_norm", OUTRA),
    ("per_layer_stroke", TRACO),
    ("dab_groups", TRACO),
    ("shape_color_preview", UI),
    ("deform", OUTRA),
    ("warp", OUTRA),
    ("sculpt", OUTRA),
    ("wetpaint", "tem o seu censo (WET_FORA_DO_PREVIEW)"),
];

/// Os campos do `WetPaintState` que o preview NÃO leva.
const WET_FORA_DO_PREVIEW: &[(&str, &str)] = &[
    ("armed", "o meio: o `set_paint_media` do tool do preview"),
    ("session", SESSAO),
    ("live_gesture", TRACO),
    ("pending_deposit", TRACO),
    ("deposit_pass", TRACO),
    (
        "tool",
        "o preview pinta o PINCEL (`o_preview_pinta_o_pincel_mesmo_com_o_smear_ativo`)",
    ),
    ("tuning_open", UI),
    ("relogio_fixo", "o preview corre no relógio fixo dele"),
    ("preset_pedido", UI),
    ("fita", SESSAO),
    ("ritmo", SESSAO),
    (
        "composta",
        "a marca da água na tela da OBRA (a macro): o preview mostra o pincel, não a obra",
    ),
];

/// Os campos que entram por uma PORTA (que re-assa as LUTs): `(campo, a porta na instalação, o token da chave)`.
const PELA_PORTA: &[(&str, &str, &str)] = &[
    (
        "shape_camadas",
        "set_shape_color_estado",
        "shape_ramp_version",
    ),
    (
        "shape_color_ramp",
        "set_shape_color_estado",
        "shape_ramp_version",
    ),
    (
        "shape_color_ramp_enabled",
        "set_shape_color_estado",
        "shape_color_ramp_enabled",
    ),
    (
        "shape_color_ramp_bw",
        "set_shape_color_estado",
        "shape_color_ramp_bw",
    ),
];

/// Os campos `pub(super)` da `struct nome` em `src`.
fn campos(src: &str, nome: &str) -> Vec<String> {
    let i = src.find(&format!("struct {nome} {{")).expect("a struct");
    let corpo = &src[i..];
    let mut fundo = 0i32;
    let fim = corpo
        .char_indices()
        .find(|&(_, c)| {
            fundo += i32::from(c == '{') - i32::from(c == '}');
            c == '}' && fundo == 0
        })
        .map(|(k, _)| k)
        .expect("o fim da struct");
    corpo[..fim]
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub(super) "))
        .filter_map(|l| l.split_once(':').map(|(n, _)| n.trim().to_owned()))
        .collect()
}

/// O texto entre `de` e `ate` em `src`.
fn trecho<'a>(src: &'a str, de: &str, ate: &str) -> &'a str {
    let i = src.find(de).expect(de);
    let j = i + src[i..].find(ate).expect(ate);
    &src[i..j]
}

/// `.campo` (ou `.campo_version`) como palavra em `txt`.
fn nomeia(txt: &str, campo: &str) -> bool {
    [format!(".{campo}"), format!(".{campo}_version")]
        .iter()
        .any(|agulha| {
            txt.match_indices(agulha.as_str()).any(|(k, m)| {
                txt[k + m.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
            })
        })
}

/// ⭐ **Cada campo do estado do pincel vai para o preview — ou diz porque não vai.** Um campo NOVO do
/// `PaintState`/`WetPaintState` reprova aqui até ser classificado: levado (a instalação do preview E a chave
/// nomeiam-no), por uma porta, ou fora com o porquê. Foi assim que as Grain Colors, a ponta de N imagens, o
/// dual de imagem e o Composite ficaram fora do preview sem ninguém ver (22/10).
#[test]
fn cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao() {
    let preview = include_str!("preview.rs");
    let instala = trecho(preview, "fn instalar_cena(", "fn ponto(");
    let chave = trecho(preview, "pub fn wet_preview_chave(", "fn instalar_cena(");
    let mut erros = Vec::new();
    for (src, nome, fora) in [
        (include_str!("../state.rs"), "PaintState", FORA_DO_PREVIEW),
        (
            include_str!("../wetpaint.rs"),
            "WetPaintState",
            WET_FORA_DO_PREVIEW,
        ),
    ] {
        let todos = campos(src, nome);
        assert!(
            todos.len() > 10,
            "{nome}: o censo não leu os campos ({})",
            todos.len()
        );
        for (f, _) in fora {
            if !todos.iter().any(|c| c == f) {
                erros.push(format!(
                    "{nome}.{f}: está no censo e já não existe (tire-o)"
                ));
            }
        }
        for c in &todos {
            if fora.iter().any(|(f, _)| f == c) {
                continue;
            }
            if let Some((_, porta, token)) = PELA_PORTA.iter().find(|(f, _, _)| f == c) {
                if !instala.contains(porta) || !nomeia(chave, token) {
                    erros.push(format!(
                        "{nome}.{c}: a porta {porta} ou a chave {token} sumiu"
                    ));
                }
                continue;
            }
            if !nomeia(instala, c) {
                erros.push(format!(
                    "{nome}.{c}: o preview não o instala (leve-o ou diga porque não)"
                ));
            }
            if !nomeia(chave, c) {
                erros.push(format!("{nome}.{c}: a chave do preview não o vê"));
            }
        }
    }
    assert!(
        erros.is_empty(),
        "o censo do preview:\n{}",
        erros.join("\n")
    );
}
