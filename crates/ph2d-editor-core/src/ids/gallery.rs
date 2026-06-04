use super::*;

// ── Inspector widget samples ───────────────────────────────────────────────
// One of each canonical widget, parented to the Inspector panel.
// These are *demonstration* widgets; their state lives on the store
// but is not wired to any simulation. The placeholder fixture-driven
// rows that used to live in 300..370 were removed pre-samples.
pub const INSP_SAMPLE_TEXT: NodeId = hash_node_id("insp_sample_text");
pub const INSP_SAMPLE_TEXTAREA: NodeId = hash_node_id("insp_sample_textarea");
pub const INSP_SAMPLE_COMBO: NodeId = hash_node_id("insp_sample_combo");
pub const INSP_SAMPLE_COMBO_OPT_A: NodeId = hash_node_id("insp_sample_combo_opt_a");
pub const INSP_SAMPLE_COMBO_OPT_B: NodeId = hash_node_id("insp_sample_combo_opt_b");
pub const INSP_SAMPLE_COMBO_OPT_C: NodeId = hash_node_id("insp_sample_combo_opt_c");
pub const INSP_SAMPLE_NUMBER: NodeId = hash_node_id("insp_sample_number");
pub const INSP_SAMPLE_SLIDER: NodeId = hash_node_id("insp_sample_slider");
pub const INSP_SAMPLE_SLIDER_CHIP: NodeId = hash_node_id("insp_sample_slider_chip");
pub const INSP_SAMPLE_CHECKBOX: NodeId = hash_node_id("insp_sample_checkbox");
pub const INSP_SAMPLE_TOGGLE: NodeId = hash_node_id("insp_sample_toggle");
pub const INSP_SAMPLE_RADIO_A: NodeId = hash_node_id("insp_sample_radio_a");
pub const INSP_SAMPLE_RADIO_B: NodeId = hash_node_id("insp_sample_radio_b");
pub const INSP_SAMPLE_RADIO_C: NodeId = hash_node_id("insp_sample_radio_c");
pub const INSP_SAMPLE_DROPDOWN: NodeId = hash_node_id("insp_sample_dropdown");
pub const INSP_SAMPLE_DD_OPT_A: NodeId = hash_node_id("insp_sample_dd_opt_a");
pub const INSP_SAMPLE_DD_OPT_B: NodeId = hash_node_id("insp_sample_dd_opt_b");
pub const INSP_SAMPLE_DD_OPT_C: NodeId = hash_node_id("insp_sample_dd_opt_c");
pub const INSP_SAMPLE_TAB_A: NodeId = hash_node_id("insp_sample_tab_a");
pub const INSP_SAMPLE_TAB_B: NodeId = hash_node_id("insp_sample_tab_b");
pub const INSP_SAMPLE_TAB_C: NodeId = hash_node_id("insp_sample_tab_c");
pub const INSP_SAMPLE_TREE_ROOT: NodeId = hash_node_id("insp_sample_tree_root");
pub const INSP_SAMPLE_TREE_LEAF_A: NodeId = hash_node_id("insp_sample_tree_leaf_a");
pub const INSP_SAMPLE_TREE_LEAF_B: NodeId = hash_node_id("insp_sample_tree_leaf_b");
pub const INSP_SAMPLE_V3_X: NodeId = hash_node_id("insp_sample_v3_x");
pub const INSP_SAMPLE_V3_Y: NodeId = hash_node_id("insp_sample_v3_y");
pub const INSP_SAMPLE_V3_Z: NodeId = hash_node_id("insp_sample_v3_z");
pub const INSP_SAMPLE_SWATCH: NodeId = hash_node_id("insp_sample_swatch");
pub const INSP_SAMPLE_BTN_PRIMARY: NodeId = hash_node_id("insp_sample_btn_primary");
pub const INSP_SAMPLE_BTN_SECONDARY: NodeId = hash_node_id("insp_sample_btn_secondary");
pub const INSP_SAMPLE_BTN_DANGER: NodeId = hash_node_id("insp_sample_btn_danger");
pub const INSP_SAMPLE_BTN_ICON: NodeId = hash_node_id("insp_sample_btn_icon");
pub const INSP_SAMPLE_LIST_ITEM: NodeId = hash_node_id("insp_sample_list_item");
pub const INSP_SAMPLE_TAG_REMOVE: NodeId = hash_node_id("insp_sample_tag_remove");

// Section header ids — clicking toggles the section's collapsed
// state on the WidgetStore. Each maps 1:1 to the corresponding
// `paint_*_section` function in `inspector.rs`.
pub const INSP_SECTION_INPUTS: NodeId = hash_node_id("insp_section_inputs");
pub const INSP_SECTION_SLIDER: NodeId = hash_node_id("insp_section_slider");
pub const INSP_SECTION_SWITCHES: NodeId = hash_node_id("insp_section_switches");
pub const INSP_SECTION_LISTS: NodeId = hash_node_id("insp_section_lists");
pub const INSP_SECTION_VECTOR: NodeId = hash_node_id("insp_section_vector");
pub const INSP_SECTION_STATUS: NodeId = hash_node_id("insp_section_status");
pub const INSP_SECTION_COLOR: NodeId = hash_node_id("insp_section_color");
pub const INSP_SECTION_ACTIONS: NodeId = hash_node_id("insp_section_actions");
pub const INSP_SECTION_IDENTITY: NodeId = hash_node_id("insp_section_identity");
pub const INSP_SECTION_CARD: NodeId = hash_node_id("insp_section_card");

// Section header color-circle hit ids. Each section displays a
// small colored circle on the right of its title (replacing the
// old count chip); clicking the circle opens the global color
// picker for that section. Index ordering matches `SECTION_IDS`.
pub const INSP_SECTION_INPUTS_COLOR: NodeId = hash_node_id("insp_section_inputs_color");
pub const INSP_SECTION_SLIDER_COLOR: NodeId = hash_node_id("insp_section_slider_color");
pub const INSP_SECTION_SWITCHES_COLOR: NodeId = hash_node_id("insp_section_switches_color");
pub const INSP_SECTION_LISTS_COLOR: NodeId = hash_node_id("insp_section_lists_color");
pub const INSP_SECTION_VECTOR_COLOR: NodeId = hash_node_id("insp_section_vector_color");
pub const INSP_SECTION_STATUS_COLOR: NodeId = hash_node_id("insp_section_status_color");
pub const INSP_SECTION_COLOR_COLOR: NodeId = hash_node_id("insp_section_color_color");
pub const INSP_SECTION_ACTIONS_COLOR: NodeId = hash_node_id("insp_section_actions_color");
pub const INSP_SECTION_IDENTITY_COLOR: NodeId = hash_node_id("insp_section_identity_color");
pub const INSP_SECTION_CARD_COLOR: NodeId = hash_node_id("insp_section_card_color");

// Sprite Inspector v2 W6 (spec §15.7): Widget Gallery showcase section
// for the new Inspector v2 foundational widgets (Rect2Editor,
// BitmaskGrid32, NumericInputWithUnit, VariantEditor, KeyValueList,
// SegmentedAdaptive).
pub const INSP_SECTION_W6: NodeId = hash_node_id("insp_section_w6");
pub const INSP_SECTION_W6_COLOR: NodeId = hash_node_id("insp_section_w6_color");
/// Rect2Editor demo: X/Y/W/H number inputs.
pub const INSP_SAMPLE_W6_RECT: [NodeId; 4] = [
    hash_node_id("insp_sample_w6_rect_x"),
    hash_node_id("insp_sample_w6_rect_y"),
    hash_node_id("insp_sample_w6_rect_w"),
    hash_node_id("insp_sample_w6_rect_h"),
];
/// BitmaskGrid32 demo: 32 per-bit checkbox hit ids.
pub const INSP_SAMPLE_W6_MASK: [NodeId; 32] = [
    hash_node_id("insp_sample_w6_mask_0"),
    hash_node_id("insp_sample_w6_mask_1"),
    hash_node_id("insp_sample_w6_mask_2"),
    hash_node_id("insp_sample_w6_mask_3"),
    hash_node_id("insp_sample_w6_mask_4"),
    hash_node_id("insp_sample_w6_mask_5"),
    hash_node_id("insp_sample_w6_mask_6"),
    hash_node_id("insp_sample_w6_mask_7"),
    hash_node_id("insp_sample_w6_mask_8"),
    hash_node_id("insp_sample_w6_mask_9"),
    hash_node_id("insp_sample_w6_mask_10"),
    hash_node_id("insp_sample_w6_mask_11"),
    hash_node_id("insp_sample_w6_mask_12"),
    hash_node_id("insp_sample_w6_mask_13"),
    hash_node_id("insp_sample_w6_mask_14"),
    hash_node_id("insp_sample_w6_mask_15"),
    hash_node_id("insp_sample_w6_mask_16"),
    hash_node_id("insp_sample_w6_mask_17"),
    hash_node_id("insp_sample_w6_mask_18"),
    hash_node_id("insp_sample_w6_mask_19"),
    hash_node_id("insp_sample_w6_mask_20"),
    hash_node_id("insp_sample_w6_mask_21"),
    hash_node_id("insp_sample_w6_mask_22"),
    hash_node_id("insp_sample_w6_mask_23"),
    hash_node_id("insp_sample_w6_mask_24"),
    hash_node_id("insp_sample_w6_mask_25"),
    hash_node_id("insp_sample_w6_mask_26"),
    hash_node_id("insp_sample_w6_mask_27"),
    hash_node_id("insp_sample_w6_mask_28"),
    hash_node_id("insp_sample_w6_mask_29"),
    hash_node_id("insp_sample_w6_mask_30"),
    hash_node_id("insp_sample_w6_mask_31"),
];
/// NumericInputWithUnit demo.
pub const INSP_SAMPLE_W6_UNIT: NodeId = hash_node_id("insp_sample_w6_unit");
/// VariantEditor demo root id (per-row kind dropdowns derive from this).
pub const INSP_SAMPLE_W6_VARIANT: NodeId = hash_node_id("insp_sample_w6_variant");
/// KeyValueList demo: one entry's key/value/remove ids + the add button.
pub const INSP_SAMPLE_W6_KV_KEY: NodeId = hash_node_id("insp_sample_w6_kv_key");
pub const INSP_SAMPLE_W6_KV_VAL: NodeId = hash_node_id("insp_sample_w6_kv_val");
pub const INSP_SAMPLE_W6_KV_REMOVE: NodeId = hash_node_id("insp_sample_w6_kv_remove");
pub const INSP_SAMPLE_W6_KV_ADD: NodeId = hash_node_id("insp_sample_w6_kv_add");
/// SegmentedAdaptive demo: four 9-slice draw-mode options.
pub const INSP_SAMPLE_W6_SEG: [NodeId; 4] = [
    hash_node_id("insp_sample_w6_seg_0"),
    hash_node_id("insp_sample_w6_seg_1"),
    hash_node_id("insp_sample_w6_seg_2"),
    hash_node_id("insp_sample_w6_seg_3"),
];

// Pre-allocated note hit-slot ids. Each note in `notes_per_panel`
// gets one of these slots assigned by position. Right-clicking a
// slot opens the `NoteBackground` context menu for that index.
pub const INSP_NOTE_SLOT_0: NodeId = hash_node_id("insp_note_slot_0");
pub const INSP_NOTE_SLOT_1: NodeId = hash_node_id("insp_note_slot_1");
pub const INSP_NOTE_SLOT_2: NodeId = hash_node_id("insp_note_slot_2");
pub const INSP_NOTE_SLOT_3: NodeId = hash_node_id("insp_note_slot_3");
pub const INSP_NOTE_SLOT_4: NodeId = hash_node_id("insp_note_slot_4");
pub const INSP_NOTE_SLOT_5: NodeId = hash_node_id("insp_note_slot_5");
pub const INSP_NOTE_SLOT_6: NodeId = hash_node_id("insp_note_slot_6");
pub const INSP_NOTE_SLOT_7: NodeId = hash_node_id("insp_note_slot_7");
pub const INSP_NOTE_SLOT_8: NodeId = hash_node_id("insp_note_slot_8");
pub const INSP_NOTE_SLOT_9: NodeId = hash_node_id("insp_note_slot_9");
pub const INSP_NOTE_SLOT_10: NodeId = hash_node_id("insp_note_slot_10");
pub const INSP_NOTE_SLOT_11: NodeId = hash_node_id("insp_note_slot_11");

// Editable text fields for each note slot. NOTE_TITLE_N and
// NOTE_BODY_N are the title TextInput and body TextArea for note N.
// Click → focus → type to edit. Double-click → select all.
pub const INSP_NOTE_TITLE_0: NodeId = hash_node_id("insp_note_title_0");
pub const INSP_NOTE_TITLE_1: NodeId = hash_node_id("insp_note_title_1");
pub const INSP_NOTE_TITLE_2: NodeId = hash_node_id("insp_note_title_2");
pub const INSP_NOTE_TITLE_3: NodeId = hash_node_id("insp_note_title_3");
pub const INSP_NOTE_TITLE_4: NodeId = hash_node_id("insp_note_title_4");
pub const INSP_NOTE_TITLE_5: NodeId = hash_node_id("insp_note_title_5");
pub const INSP_NOTE_TITLE_6: NodeId = hash_node_id("insp_note_title_6");
pub const INSP_NOTE_TITLE_7: NodeId = hash_node_id("insp_note_title_7");
pub const INSP_NOTE_TITLE_8: NodeId = hash_node_id("insp_note_title_8");
pub const INSP_NOTE_TITLE_9: NodeId = hash_node_id("insp_note_title_9");
pub const INSP_NOTE_TITLE_10: NodeId = hash_node_id("insp_note_title_10");
pub const INSP_NOTE_TITLE_11: NodeId = hash_node_id("insp_note_title_11");
pub const INSP_NOTE_BODY_0: NodeId = hash_node_id("insp_note_body_0");
pub const INSP_NOTE_BODY_1: NodeId = hash_node_id("insp_note_body_1");
pub const INSP_NOTE_BODY_2: NodeId = hash_node_id("insp_note_body_2");
pub const INSP_NOTE_BODY_3: NodeId = hash_node_id("insp_note_body_3");
pub const INSP_NOTE_BODY_4: NodeId = hash_node_id("insp_note_body_4");
pub const INSP_NOTE_BODY_5: NodeId = hash_node_id("insp_note_body_5");
pub const INSP_NOTE_BODY_6: NodeId = hash_node_id("insp_note_body_6");
pub const INSP_NOTE_BODY_7: NodeId = hash_node_id("insp_note_body_7");
pub const INSP_NOTE_BODY_8: NodeId = hash_node_id("insp_note_body_8");
pub const INSP_NOTE_BODY_9: NodeId = hash_node_id("insp_note_body_9");
pub const INSP_NOTE_BODY_10: NodeId = hash_node_id("insp_note_body_10");
pub const INSP_NOTE_BODY_11: NodeId = hash_node_id("insp_note_body_11");
