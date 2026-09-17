//! **AS STRINGS DOS PAINÉIS DO MOTION** — `panel.motion_graph.*`, `panel.motion_params.*` e os
//! editores ricos que o cartão e a linha partilham (`panel.param_editors.*`).
//!
//! ⚠️ **Um corte por ASSUNTO**, como os irmãos. Migrado em 2026-09-16 por
//! `scripts/migrar-texto-pintado.py` (mapas `docs/UI_New_and_Simple/ferramentas/seccoes_motion_graph.tsv`,
//! `…_motion_params.tsv` e `…_param_editors.tsv`).
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave dos painéis do Motion, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⭐⭐ **O NOME DE CADA PAINEL, fora dos marcadores porque é escrito à mão** (2026-09-17): o
        // `Panel::TITLE` passou a ser um `TextKey`, e estas duas chaves são o que a ABA do encaixe
        // lê — a mesma que o cabeçalho do painel pinta.
        "panel.motion_graph.title" => "Motion Graph",
        "panel.motion_params.title" => "Motion Params",
        // ph2d-migrar-texto:begin
        "panel.motion_graph.card.on" => "On",
        "panel.motion_graph.card.off" => "Off",
        "panel.motion_graph.chrome.split_horizontally" => "Split horizontally",
        "panel.motion_graph.chrome.split_vertically" => "Split vertically",
        "panel.motion_graph.chrome.fit_view_f" => "Fit view \u{00b7} F",
        "panel.motion_graph.chrome.add_group_backdrop" => "Add group backdrop",
        "panel.motion_graph.chrome.knife" => "Knife \u{00b7} K \u{00b7} drag across wires to cut",
        "panel.motion_graph.chrome.probe" => {
            "Probe \u{00b7} P \u{00b7} click a node to read its stream"
        }
        "panel.motion_graph.chrome.group_ungroup_ctrl_g" => "Group / Ungroup \u{00b7} Ctrl+G",
        "panel.motion_graph.chrome.auto_arrange" => "Auto-arrange the graph",
        "panel.motion_graph.chrome.node_help" => "Node help on/off \u{00b7} badges + auto-fix",
        "panel.motion_graph.menu.connect_inside_group" => "Connect Inside Group",
        "panel.motion_graph.menu.backdrop" => "Backdrop",
        "panel.motion_graph.menu.node" => "Node",
        "panel.motion_graph.role.a_pulse" => "a pulse",
        "panel.motion_graph.role.a_constant" => "a constant",
        "panel.motion_graph.role.a_number" => "a number",
        "panel.motion_graph.role.a_stream" => "a stream",
        "panel.motion_graph.role.a_vector_shape" => "a vector shape",
        "panel.motion_graph.role.a_field" => "a field",
        "panel.motion_graph.role.an_audio_signal" => "an audio signal",
        "panel.motion_graph.role.control_data" => "control data",
        "panel.motion_graph.menu.red" => "Red",
        "panel.motion_graph.menu.amber" => "Amber",
        "panel.motion_graph.menu.lime" => "Lime",
        "panel.motion_graph.menu.green" => "Green",
        "panel.motion_graph.menu.teal" => "Teal",
        "panel.motion_graph.menu.blue" => "Blue",
        "panel.motion_graph.menu.violet" => "Violet",
        "panel.motion_graph.menu.grey" => "Grey",
        "panel.motion_graph.menu.enter_double_click" => "Enter (Double-click)",
        "panel.motion_graph.menu.cut_ctrl_x" => "Cut (Ctrl+X)",
        "panel.motion_graph.menu.copy_ctrl_c" => "Copy (Ctrl+C)",
        "panel.motion_graph.menu.duplicate_ctrl_d" => "Duplicate (Ctrl+D)",
        "panel.motion_graph.menu.delete_del" => "Delete (Del)",
        "panel.motion_graph.menu.toggle_mute_h" => "Toggle Mute (H)",
        "panel.motion_graph.menu.rename_f2" => "Rename (F2)",
        "panel.motion_graph.menu.ungroup_ctrl_alt_g" => "Ungroup (Ctrl+Alt+G)",
        "panel.motion_params.panel.motion" => "Motion",
        "panel.motion_params.rows.color" => "Color",
        "panel.motion_params.rows.custom" => "Custom",
        "panel.motion_params.rows.from_stream" => "From stream",
        "panel.motion_params.rows.column" => "Column",
        "panel.motion_params.rows.e_g_inv_mass_id" => "e.g. inv_mass, id",
        "panel.motion_params.rows.drawn_shapes" => "Drawn shapes",
        "panel.motion_params.rows.name" => "Name",
        "panel.motion_params.rows.e_g_a_drawn_shape" => "e.g. a drawn shape",
        "panel.motion_params.rows.browse" => "Browse\u{2026}",
        "panel.motion_params.rows.e_g_home_you_song_wav" => "e.g. /home/you/song.wav",
        "panel.motion_params.rows.file_not_found" => "File not found",
        "panel.motion_params.rows.e_g_sin_t" => "e.g. sin(t)",
        "panel.param_editors.curve.linear" => "Linear",
        "panel.param_editors.curve.smooth" => "Smooth",
        "panel.param_editors.curve.hold" => "Hold",
        // ── A biblioteca de nós e os fundos do grafo (publicados por `ph2d-app-motion`) ──
        "panel.motion_graph.library.source" => "Source",
        "panel.motion_graph.library.distribute" => "Distribute",
        "panel.motion_graph.library.transform" => "Transform",
        "panel.motion_graph.library.focus" => "Focus",
        "panel.motion_graph.library.fx" => "Fx",
        "panel.motion_graph.library.output" => "Output",
        "panel.motion_graph.library.utility" => "Utility",
        "panel.motion_graph.library.add_node" => "Add Node",
        "panel.motion_graph.library.basic_transforms" => "Basic Transforms",
        "panel.motion_graph.library.deformers" => "Deformers",
        "panel.motion_graph.library.forces_and_physics" => "Forces & Physics",
        "panel.motion_graph.library.rigging" => "Rigging",
        "panel.motion_graph.library.behaviors_and_timing" => "Behaviors & Timing",
        "panel.motion_graph.library.values_and_math" => "Values & Math",
        "panel.motion_graph.library.time_and_signal" => "Time & Signal",
        "panel.motion_graph.library.data_and_adapters" => "Data & Adapters",
        "panel.motion_graph.library.default_group" => "Group",
        "panel.motion_graph.backdrop.red" => "Red",
        "panel.motion_graph.backdrop.amber" => "Amber",
        "panel.motion_graph.backdrop.olive" => "Olive",
        "panel.motion_graph.backdrop.green" => "Green",
        "panel.motion_graph.backdrop.teal" => "Teal",
        "panel.motion_graph.backdrop.blue" => "Blue",
        "panel.motion_graph.backdrop.violet" => "Violet",
        "panel.motion_graph.backdrop.grey" => "Grey",
        "panel.motion_graph.backdrop.title" => "Backdrop",
        "panel.motion_graph.backdrop.title_row" => "Title",
        "panel.motion_graph.backdrop.color_row" => "Color",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
