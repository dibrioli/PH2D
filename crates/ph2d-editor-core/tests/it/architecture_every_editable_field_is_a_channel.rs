//! ⭐⭐⭐ **TODO CAMPO EDITÁVEL TEM CANAL** (plano 35 §5.2) — o gate que torna «o Auto Key grava
//! absolutamente tudo» (dono, 19/10) verificável: um campo novo sem canal reprova.
//!
//! Lê o FONTE, como o `architecture_panel_wiring_parity`. A população é cada chamada de uma PORTA
//! com coluna de losango ([`PORTAS`] — as linhas de `property_row`, o slider com chip, a escolha, o
//! chip numérico do rail, o próprio `paint_decorator_for`) ou de um helper de linha de um painel
//! que só a repassa ([`LINHAS`]). Cada chamada tem de cair numa classe:
//!
//! - **canal**: um dos ids dela está numa tabela de canais ([`TABELAS`]: `ph2d_app_vec::marcas`,
//!   `ph2d_app_components::marcas_inspector`, `ph2d_app_flip::marcas_camada`);
//! - **⛔ não-animável** ou **dívida**: um id (ou prefixo) dela está em [`POR_ID`], a função dela
//!   em [`POR_FUNCAO`] (o id vem numa variável), ou o ficheiro em [`DIVIDA_FICHEIRO`].
//!
//! Fora do [`ESCOPO`], cada crate que pinta campos está em [`DIVIDA_CRATE`] (o bloco 2 do plano 35
//! §10 zera-as) — uma crate de painel NOVA que pinte campos reprova até escolher o seu lado.
//!
//! ⛔ Toda tabela tem censo de obsolescência: uma entrada que já não casa nada reprova (uma isenção
//! que sobrevive ao código que a explicava é licença).
//!
//! E a lei da COLUNA (F24d): os primitivos de campo que não a trazem ([`SEM_COLUNA`] — a caixa de
//! marcar, o chip de menu, o seletor pintado direto, o olho) só se pintam numa função que pinta a
//! coluna do decorator — sem isto o campo ficava fora da população, invisível a este gate.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Classe {
    /// O canal existe por um caminho que o gate não lê como literal (o id derivado de um índice
    /// cuja inversa mora na tabela).
    Canal,
    /// ⛔ Não é propriedade da cena (plano 35 §8): estado da ferramenta ou da vista, estrutura, ou
    /// o parâmetro da própria animação.
    NaoAnimavel,
    /// Propriedade sem canal ainda — a zerar (plano 35 §10, bloco 2).
    Divida,
}
use Classe::{Canal, Divida, NaoAnimavel};

/// As portas com a coluna do losango (o nome da função chamada).
const PORTAS: &[&str] = &[
    "paint_fields_row",
    "paint_field_row",
    "paint_field_row_value",
    "paint_text_row",
    "paint_color_row",
    "paint_slider_with_chip",
    "paint_slider_with_chip_layout_adaptive",
    "paint_choice_row",
    "paint_decorator_for",
    // `ToolRailEntry::number` / `::compound` — o chip numérico e o de opção do rail (o losango à
    // direita deles).
    "number",
    "compound",
];

/// As portas que só valem prefixadas pelo tipo (`ToolRailEntry::number`, não um `number(` solto).
const DO_RAIL: &[&str] = &["number", "compound"];

/// Os primitivos de campo SEM a coluna do losango: só se chamam numa função que a pinta
/// ([`COLUNA`]). O olho conta quando a função pinta um ícone com ele.
const SEM_COLUNA: &[&str] = &[
    "paint_checkbox(",
    "paint_dropdown_chip(",
    "paint_segmented_group_adaptive(",
    "IconId::Eye",
    "eye_icon(",
];

/// O que uma função escreve quando pinta a coluna do decorator.
const COLUNA: &[&str] = &["paint_decorator_for(", ".decorator("];

/// As crates cujos campos este gate classifica um a um; `None` = todos os ficheiros de `src/`,
/// `Some` = só estes (o resto da crate é dívida do bloco 2).
const ESCOPO: &[(&str, Option<&[&str]>)] = &[
    ("ph2d-panel-vector", None),
    (
        "ph2d-panel-inspector",
        Some(&[
            "sections/transform.rs",
            "sections/color_tint.rs",
            "sections/emissive_row.rs",
            "sections/joint_cards.rs",
            "sections/joint_kind_rows.rs",
            "sections/joint_custom.rs",
            "sections/physics.rs",
            "sections/physics_area_rows.rs",
            "sections/physics_body.rs",
            "sections/physics_doors.rs",
            "sections/physics_join_rows.rs",
            "sections/physics_part.rs",
            "sections/physics_rows.rs",
            "sections/physics_tag_row.rs",
        ]),
    ),
    ("ph2d-panel-flip", None),
];

/// As tabelas de canais (caminhos a partir de `crates/`).
const TABELAS: &[&str] = &[
    "ph2d-app-vec/src/marcas.rs",
    "ph2d-app-vec/src/marcas_vivos.rs",
    "ph2d-app-components/src/marcas_inspector.rs",
    "ph2d-app-flip/src/marcas_camada.rs",
];

/// Os helpers de LINHA de cada painel: recebem o id do chamador e repassam-no a uma porta — a
/// chamada deles com o id numa variável não é população, a dos chamadores deles é.
const LINHAS: &[(&str, &str)] = &[
    ("ph2d-panel-vector", "slider_row"),
    ("ph2d-panel-vector", "segmented"),
    ("ph2d-panel-vector", "segmented3"),
    ("ph2d-panel-vector", "number_row"),
    ("ph2d-panel-vector", "lone_number_row"),
    ("ph2d-panel-vector", "lone_number_row_full"),
    ("ph2d-panel-vector", "number_cell"),
    ("ph2d-panel-vector", "fields_row"),
    ("ph2d-panel-vector", "colour_swatch_row"),
    ("ph2d-panel-vector", "colour_swatch_row_rect"),
    ("ph2d-panel-vector", "checkbox_row"),
    ("ph2d-panel-vector", "dropdown_row"),
    ("ph2d-panel-vector", "decorator"),
    ("ph2d-panel-vector", "chip"),
    ("ph2d-panel-vector", "list_chip"),
    ("ph2d-panel-inspector", "fields_row"),
    ("ph2d-panel-inspector", "num_row"),
    ("ph2d-panel-inspector", "num_row_unit"),
    ("ph2d-panel-inspector", "seg_row"),
    ("ph2d-panel-inspector", "paint_tint_swatch_cell"),
    ("ph2d-panel-inspector", "bloco_de_cores"),
    ("ph2d-panel-flip", "slider_row"),
    ("ph2d-panel-flip", "slider_row_linked"),
];

/// A classe de uma chamada por um dos seus ids — exacto, ou PREFIXO quando acaba em `_`.
const POR_ID: &[(&str, Classe, &str)] = &[
    (
        "VECTOR_PENCIL_",
        NaoAnimavel,
        "ajuste da ferramenta Lápis (§8)",
    ),
    ("VECTOR_SYM_", NaoAnimavel, "simetria da ferramenta (§8)"),
    (
        "VECTOR_SNAP_",
        NaoAnimavel,
        "o snap é da vista/ferramenta (§8)",
    ),
    (
        "VECTOR_BOOL_",
        NaoAnimavel,
        "a booleana é estrutura; o RESULTADO grava-se pelo canal Forma (§8)",
    ),
    (
        "VECTOR_STATE_",
        NaoAnimavel,
        "a transição de um estado da Interface É uma animação; não se anima a si própria",
    ),
    (
        "VECTOR_FRAME_PANEL_",
        NaoAnimavel,
        "«Show as Panel» é a visibilidade do painel no editor (HeroScreen), não documento",
    ),
    (
        "VECTOR_LAYOUT_PAD_ALL_MODE",
        NaoAnimavel,
        "a margem vista como um número ou quatro: estado do painel; as margens têm canal",
    ),
    (
        "VECTOR_CUT_",
        NaoAnimavel,
        "os verbos da ferramenta Cut (§8)",
    ),
    (
        "VECTOR_BAR_STROKE",
        NaoAnimavel,
        "abre o painel do traço: um verbo",
    ),
    (
        "VECTOR_TRANSFORM_RESIZE_BOX",
        NaoAnimavel,
        "o que a alça do gizmo faz ao objecto: comportamento da edição, não desenho",
    ),
    (
        "TexPatKnob::Lock",
        NaoAnimavel,
        "o cadeado do tamanho do padrão: gesto da sessão (não viaja no ficheiro)",
    ),
    (
        "TexPatKnob::GapLink",
        NaoAnimavel,
        "o elo do vão do padrão: gesto da sessão (não viaja no ficheiro)",
    ),
    (
        "INSP_PHYS_BAKE_",
        NaoAnimavel,
        "o que o Bake grava: parâmetro de um comando (§8)",
    ),
    (
        "INSP_PHYS_JOIN_KIND",
        NaoAnimavel,
        "o tipo da PRÓXIMA junta que o Join cria: parâmetro de um comando (§8)",
    ),
    (
        "INSP_PHYS_SIGNAL_TAG",
        NaoAnimavel,
        "o filtro por tag dos sinais: fiação por identidade (§8)",
    ),
    (
        "FLIP_",
        NaoAnimavel,
        "o pincel, a borracha, o balde e o Colorize do Flip: o tamanho e a cor do PRÓXIMO traço \
         são da ferramenta (§8); o traço feito é desenho (a lei do Grease Pencil, §8)",
    ),
];

/// As chamadas cujo id vem numa VARIÁVEL, pela função que as faz. Para [`Canal`], os nomes que a
/// variável pode ter estão no ficheiro da função E numa tabela.
const POR_FUNCAO: &[(&str, &str, Classe, &[&str], &str)] = &[
    (
        "ph2d-panel-vector",
        "dash_rows",
        Canal,
        &["VECTOR_DASH_IDS", "VECTOR_GAP_IDS"],
        "os pares do tracejado, das tabelas de ids",
    ),
    (
        "ph2d-panel-vector",
        "layer_row",
        Canal,
        &["vector_paint_swatch_id", "vector_paint_eye_id"],
        "a tinta e o olho da entrada `i` da pilha (`marcas::canal_do_campo`)",
    ),
    (
        "ph2d-panel-vector",
        "shape_field_chip",
        Canal,
        &["vector_shape_field_id", "vector_shape_choice_id"],
        "os campos da forma paramétrica na fila ② (`marcas::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "effect_header",
        Canal,
        &["vector_fx_hide_id"],
        "o olho do efeito da linha `r` (`marcas::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "marker_row",
        Canal,
        &["VECTOR_MARKER_START_DD", "VECTOR_MARKER_END_DD"],
        "a ponta do início OU do fim, pelo slot",
    ),
    (
        "ph2d-panel-vector",
        "texture_pattern_section",
        Canal,
        &[
            "TexPatKnob::Tile",
            "TexPatKnob::Mode",
            "TexPatKnob::ShiftX",
            "TexPatKnob::ShiftY",
        ],
        "o padrão de textura da tinta `slot` (`marcas::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "size_and_gap_rows",
        Canal,
        &[
            "TexPatKnob::Width",
            "TexPatKnob::Height",
            "TexPatKnob::Gap",
            "TexPatKnob::GapY",
        ],
        "o tamanho e o vão do padrão (`marcas::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "token_row",
        NaoAnimavel,
        &[],
        "a ligação a um token de cor é fiação, não propriedade",
    ),
    (
        "ph2d-panel-vector",
        "gesture_chips",
        NaoAnimavel,
        &[],
        "as opções do gesto na mão: ajustes da ferramenta (§8)",
    ),
    (
        "ph2d-panel-vector",
        "symmetry_chips",
        NaoAnimavel,
        &[],
        "a simetria da ferramenta (§8)",
    ),
    (
        "ph2d-panel-vector",
        "brush_section",
        Canal,
        &[
            "VECTOR_BRUSH_SCALE",
            "VECTOR_BRUSH_SPACING",
            "VECTOR_BRUSH_ROTATION",
        ],
        "os parâmetros do pincel do traço, da tabela do laço (`marcas_vivos::CAMPOS`)",
    ),
    (
        "ph2d-panel-vector",
        "fx_param",
        Canal,
        &["vector_fx_param_id", "vector_fx_toggle_id"],
        "os parâmetros do efeito da linha `r` (`marcas::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "power_stroke_rows",
        Canal,
        &[
            "VECTOR_EXPAND_W_START",
            "VECTOR_EXPAND_W_MID",
            "VECTOR_EXPAND_W_END",
            "VECTOR_EXPAND_W_POS",
        ],
        "os quatro sliders que armam o perfil de largura (`marcas_vivos::CAMPOS`)",
    ),
    (
        "ph2d-panel-vector",
        "width_preset_row",
        Canal,
        &["vector_width_preset_id"],
        "o catálogo de perfis de largura (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_card",
        Canal,
        &["filter_mode_id"],
        "o modo do degrau `row` dos filtros (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_header",
        Canal,
        &["filter_hide_id"],
        "o olho do degrau `row` dos filtros (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_radius_row",
        Canal,
        &["filter_radius_id", "filter_radius_num_id"],
        "o raio do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_noise_rows",
        Canal,
        &["filter_scale_id", "filter_detail_id", "filter_seed_id"],
        "o ruído do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_grow_row",
        Canal,
        &["filter_grow_id"],
        "o Amount do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_adjust_rows",
        Canal,
        &["filter_hue_id", "filter_sat_id", "filter_bright_id"],
        "o ajuste de cor do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_opacity_row",
        Canal,
        &["filter_opacity_id"],
        "a opacidade do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_offset_row",
        Canal,
        &["filter_offx_id", "filter_offy_id"],
        "o deslocamento do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_blend_row",
        Canal,
        &["filter_blend_id"],
        "a mistura do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "filter_color_swatch",
        Canal,
        &["filter_color_id", "filter_color_b_id"],
        "as cores do degrau `row` (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "patternpath_section",
        Canal,
        &["VECTOR_PATTERNPATH_FLIP", "VECTOR_PATTERNPATH_FLIP_OFF"],
        "o lado do padrão no caminho (`marcas_vivos::CAMPOS`)",
    ),
    (
        "ph2d-panel-vector",
        "textpath_section",
        Canal,
        &["VECTOR_TEXTPATH_FLIP", "VECTOR_TEXTPATH_FLIP_OFF"],
        "o lado do texto no caminho (`marcas_vivos::CAMPOS`)",
    ),
    (
        "ph2d-panel-vector",
        "layout_align_rows",
        Canal,
        &[
            "VECTOR_LAYOUT_JUSTIFY_START",
            "VECTOR_LAYOUT_JUSTIFY_BETWEEN",
        ],
        "a distribuição do auto layout (`marcas_vivos::CAMPOS`)",
    ),
    (
        "ph2d-panel-vector",
        "widget_skin_section",
        Canal,
        &["vector_widget_kind_id"],
        "a pele do widget (`marcas_vivos::derivados`)",
    ),
    (
        "ph2d-panel-vector",
        "signal_rows",
        NaoAnimavel,
        &[],
        "a ligação de um sinal é fiação, não propriedade",
    ),
    (
        "ph2d-panel-vector",
        "easing_rows",
        NaoAnimavel,
        &[],
        "a curva da transição de um estado É a animação",
    ),
    (
        "ph2d-panel-inspector",
        "paint_motor_rows",
        Canal,
        &["INSP_JOINT_MOTOR_TARGET", "INSP_JOINT_MOTOR_SPEED"],
        "o alvo OU a velocidade do motor, pelo modo",
    ),
    (
        "ph2d-panel-inspector",
        "paint_kind_params",
        Canal,
        &[
            "INSP_JOINT_LIMIT_MIN",
            "INSP_JOINT_LIMIT_MAX",
            "INSP_JOINT_STIFFNESS",
            "INSP_JOINT_DAMPING",
        ],
        "o par de limites e a mola da junta, num laço (canais JUNTA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_axis_rows",
        Canal,
        &["INSP_JOINT_AXIS_MIN", "INSP_JOINT_AXIS_MAX"],
        "o min/max do eixo `i` de um Custom (canais JUNTA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_per_corner_tab",
        Canal,
        &[
            "INSP_SPRITE_CORNER_TL",
            "INSP_SPRITE_CORNER_TR",
            "INSP_SPRITE_CORNER_BL",
            "INSP_SPRITE_CORNER_BR",
        ],
        "os quatro cantos do degradê: um losango, um grupo (canal OBJETO)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_shape_dims",
        Canal,
        &[
            "INSP_PHYS_RADIUS",
            "INSP_PHYS_HALF_X",
            "INSP_PHYS_HALF_Y",
            "INSP_PHYS_CAP_HALF_H",
        ],
        "as medidas da forma do colisor, pela forma (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_area_rows",
        Canal,
        &[
            "INSP_PHYS_FORCE_X",
            "INSP_PHYS_FORCE_Y",
            "INSP_PHYS_AREA_TORQUE",
            "INSP_PHYS_AREA_FALLOFF",
            "INSP_PHYS_AREA_DRAG",
            "INSP_PHYS_AREA_DENSITY",
            "INSP_PHYS_AREA_FORM_DRAG",
        ],
        "os números da zona de um sensor, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_body_face",
        Canal,
        &["INSP_PHYS_OFFSET_X", "INSP_PHYS_OFFSET_Y"],
        "o deslocamento do colisor, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_dynamics_rows",
        Canal,
        &[
            "INSP_PHYS_GRAVITY_SCALE",
            "INSP_PHYS_LINVEL_X",
            "INSP_PHYS_LINVEL_Y",
            "INSP_PHYS_ANGVEL",
            "INSP_PHYS_DOMINANCE",
        ],
        "a dinâmica de um corpo Dynamic, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_part_face",
        Canal,
        &[
            "INSP_PHYS_OFFSET_X",
            "INSP_PHYS_OFFSET_Y",
            "INSP_PHYS_DENSITY",
        ],
        "o deslocamento e a densidade de uma peça, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_damping_rows",
        Canal,
        &["INSP_PHYS_LINEAR_DAMPING", "INSP_PHYS_ANGULAR_DAMPING"],
        "o amortecimento do corpo, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_material_rows",
        Canal,
        &[
            "INSP_PHYS_RESTITUTION",
            "INSP_PHYS_FRICTION",
            "INSP_PHYS_REST_COMBINE",
            "INSP_PHYS_FRIC_COMBINE",
        ],
        "o material e as regras de combinação, num laço (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_mass_source",
        Canal,
        &["INSP_PHYS_MASS", "INSP_PHYS_DENSITY"],
        "a massa OU a densidade, pela fonte da massa (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_collision_rows",
        Canal,
        &[
            "INSP_PHYS_LAYER",
            "INSP_PHYS_SENSOR",
            "INSP_PHYS_WALK_GRIP",
            "INSP_PHYS_WALK_BELT",
        ],
        "a camada, o sensor e a superfície, em laços (canal FISICA)",
    ),
    (
        "ph2d-panel-inspector",
        "signal_row",
        NaoAnimavel,
        &[],
        "o nome do sinal ao bater/sair: fiação por nome (§8)",
    ),
    (
        "ph2d-panel-flip",
        "brush",
        NaoAnimavel,
        &[],
        "a força do pincel ou da borracha (o id pelo modo): ajuste da ferramenta (§8)",
    ),
    (
        "ph2d-panel-flip",
        "brush_size_row",
        NaoAnimavel,
        &[],
        "o tamanho do pincel ou da borracha (o id pelo modo): ajuste da ferramenta (§8)",
    ),
    (
        "ph2d-panel-flip",
        "paint_eye",
        Canal,
        &["FlipLayerWidget::Visibility"],
        "o olho da camada `row` (canal FLIP, `marcas_camada::CAMPOS`)",
    ),
    (
        "ph2d-panel-flip",
        "paint_labeled_slider_row",
        Canal,
        &["FlipLayerWidget::Opacity", "FlipLayerWidget::Depth"],
        "a opacidade OU a profundidade da camada, pelo tipo (canal FLIP)",
    ),
    (
        "ph2d-panel-flip",
        "paint_blend_chip",
        Canal,
        &["FlipLayerWidget::Blend"],
        "a mistura da camada `row` (canal FLIP)",
    ),
];

/// Ficheiros inteiros em dívida — famílias sem vocabulário ainda.
const DIVIDA_FICHEIRO: &[(&str, &str, &str)] = &[];

/// As crates que pintam campos fora do [`ESCOPO`] (plano 35 §10, bloco 2).
const DIVIDA_CRATE: &[(&str, Classe, &str)] = &[
    (
        "ph2d-panel-audio-editor",
        NaoAnimavel,
        "edita o ASSET de som (cortes, o rack que o Apply coze no ficheiro, loop, variações, \
         entrega), não a cena; o som na cena é o componente Áudio do objecto",
    ),
    (
        "ph2d-panel-audio-mixer",
        NaoAnimavel,
        "a mesa de MONITORIZAÇÃO da sessão: o estado mora no painel (thread-local) e não viaja no \
         projecto; o motor lê-o do painel, não do mundo — sem documento, não há o que animar",
    ),
    (
        "ph2d-panel-painter-layers",
        Divida,
        "as camadas e ajustes do Painter (§A.7)",
    ),
    (
        "ph2d-panel-physics",
        NaoAnimavel,
        "o MUNDO da simulação (afinação do solver, gravidade e arrasto por omissão, matriz de \
         camadas), as ferramentas de interacção e de junta e o debug: configuração e ferramenta, \
         não objecto; o que um corpo faz no tempo (a escala de gravidade incluída) é a §11 do \
         Inspector, canal FISICA",
    ),
    (
        "ph2d-panel-bgremoval",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-color-equalization",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-equalize-sizes",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-padding",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-upscale",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-grid-snap",
        NaoAnimavel,
        "a grelha é da vista (§8)",
    ),
    (
        "ph2d-panel-wet-tuning",
        NaoAnimavel,
        "a afinação do motor de pintura é da ferramenta",
    ),
];

// ── leitura do fonte ────────────────────────────────────────────────────────────────────────

fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// Os `.rs` de `src/` (sem os ficheiros de teste), como (caminho relativo a `src/`, fonte).
fn fontes(krate: &str) -> Vec<(String, String)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
                continue;
            }
            let nome = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !nome.ends_with(".rs") || nome.ends_with("_tests.rs") || nome == "tests.rs" {
                continue;
            }
            if let Ok(s) = std::fs::read_to_string(&p) {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .into_owned();
                out.push((rel, sem_testes(&s).to_owned()));
            }
        }
    }
    let base = crates_root().join(krate).join("src");
    let mut out = Vec::new();
    walk(&base, &base, &mut out);
    out.sort();
    out
}

fn sem_testes(s: &str) -> &str {
    s.find("#[cfg(test)]\nmod tests {").map_or(s, |i| &s[..i])
}

fn ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// As funções de `s`: (nome, corpo) — do `fn` até ao `fn` seguinte.
fn funcoes(s: &str) -> Vec<(String, &str)> {
    let mut inicios = Vec::new();
    let mut i = 0;
    while let Some(p) = s[i..].find("fn ") {
        let at = i + p;
        let antes = s[..at].chars().next_back();
        if antes.is_none_or(|c| !ident(c)) {
            let nome: String = s[at + 3..].chars().take_while(|&c| ident(c)).collect();
            if !nome.is_empty() {
                inicios.push((at, nome));
            }
        }
        i = at + 3;
    }
    let mut out = Vec::new();
    for (k, (at, nome)) in inicios.iter().enumerate() {
        let fim = inicios.get(k + 1).map_or(s.len(), |(f, _)| *f);
        out.push((nome.clone(), &s[*at..fim]));
    }
    out
}

/// As chamadas a `nomes` em `corpo`: (nome chamado, texto dos argumentos).
fn chamadas<'a>(corpo: &'a str, nomes: &BTreeSet<&str>) -> Vec<(String, &'a str)> {
    let b = corpo.as_bytes();
    let mut out = Vec::new();
    for (i, _) in corpo.match_indices('(') {
        let ini = corpo[..i]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| ident(c))
            .last()
            .map_or(i, |(j, _)| j);
        let nome = &corpo[ini..i];
        if !nomes.contains(nome) || corpo[..ini].ends_with("fn ") {
            continue;
        }
        if DO_RAIL.contains(&nome) && !corpo[..ini].ends_with("ToolRailEntry::") {
            continue;
        }
        let (mut d, mut j) = (1, i + 1);
        while j < b.len() && d > 0 {
            match b[j] {
                b'(' => d += 1,
                b')' => d -= 1,
                _ => {}
            }
            j += 1;
        }
        out.push((nome.to_owned(), &corpo[i + 1..j.saturating_sub(1)]));
    }
    out
}

/// O nome que começa em `s`: um identificador, ou `Tipo::Variante` (`TexPatKnob::Width`).
fn nome_em(s: &str) -> String {
    let a: String = s.chars().take_while(|&c| ident(c)).collect();
    match s[a.len()..].strip_prefix("::") {
        Some(resto) => {
            let b: String = resto.chars().take_while(|&c| ident(c)).collect();
            if b.is_empty() { a } else { format!("{a}::{b}") }
        }
        None => a,
    }
}

/// Os ids `ids::NOME` (maiúsculas) de um texto — `X[0]` conta como `X`; `ids::Tipo::Variante`
/// conta como `Tipo::Variante`.
fn ids_em(args: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in args.match_indices("ids::") {
        let nome = nome_em(&args[i + 5..]);
        if nome.len() > 2 && nome.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            out.push(nome);
        }
    }
    out
}

/// Os nomes que as tabelas de canais citam (`painel::X`, `ferramenta::X`, `insp::X`, e as portas
/// de ids derivados `painel::f(`).
fn nomes_das_tabelas() -> (BTreeSet<String>, String) {
    let mut nomes = BTreeSet::new();
    let mut texto = String::new();
    for t in TABELAS {
        let s = std::fs::read_to_string(crates_root().join(t)).expect("tabela de canais");
        let s = sem_testes(&s).to_owned();
        for pref in ["painel::", "ferramenta::", "insp::"] {
            for (i, _) in s.match_indices(pref) {
                let n = nome_em(&s[i + pref.len()..]);
                if !n.is_empty() {
                    nomes.insert(n);
                }
            }
        }
        texto.push_str(&s);
    }
    (nomes, texto)
}

fn por_id(ids: &[String]) -> Option<&'static (&'static str, Classe, &'static str)> {
    POR_ID.iter().find(|(chave, _, _)| {
        ids.iter().any(|id| {
            if chave.ends_with('_') {
                id.starts_with(chave)
            } else {
                id == chave
            }
        })
    })
}

/// Uma chamada da população: onde está e o que leva.
struct Chamada {
    krate: &'static str,
    ficheiro: String,
    funcao: String,
    porta: String,
    ids: Vec<String>,
}

fn populacao() -> Vec<Chamada> {
    let mut out = Vec::new();
    for &(krate, so) in ESCOPO {
        let mut nomes: BTreeSet<&str> = PORTAS.iter().copied().collect();
        nomes.extend(LINHAS.iter().filter(|(k, _)| *k == krate).map(|(_, f)| *f));
        for (ficheiro, s) in fontes(krate) {
            if so.is_some_and(|fs| !fs.contains(&ficheiro.as_str())) {
                continue;
            }
            for (funcao, corpo) in funcoes(&s) {
                for (porta, args) in chamadas(corpo, &nomes) {
                    out.push(Chamada {
                        krate,
                        ficheiro: ficheiro.clone(),
                        funcao: funcao.clone(),
                        porta,
                        ids: ids_em(args),
                    });
                }
            }
        }
    }
    out
}

fn e_linha(krate: &str, funcao: &str) -> bool {
    LINHAS.iter().any(|&(k, f)| k == krate && f == funcao)
}

fn na_divida(krate: &str, ficheiro: &str) -> bool {
    DIVIDA_FICHEIRO
        .iter()
        .any(|&(k, f, _)| k == krate && f == ficheiro)
}

// ── os gates ─────────────────────────────────────────────────────────────────────────────────

/// ⭐ Toda chamada da população tem classe: canal, ⛔ não-animável, ou dívida nomeada.
#[test]
fn todo_campo_pintado_com_losango_tem_canal_ou_porque() {
    let (canais, _) = nomes_das_tabelas();
    let mut sem = Vec::new();
    for c in populacao() {
        if na_divida(c.krate, &c.ficheiro) {
            continue;
        }
        let ok = if c.ids.is_empty() {
            e_linha(c.krate, &c.funcao)
                || POR_FUNCAO
                    .iter()
                    .any(|&(k, f, ..)| k == c.krate && f == c.funcao)
        } else {
            c.ids.iter().any(|id| canais.contains(id)) || por_id(&c.ids).is_some()
        };
        if !ok {
            sem.push(format!(
                "{}/src/{} · fn {} → {}({})",
                c.krate,
                c.ficheiro,
                c.funcao,
                c.porta,
                if c.ids.is_empty() {
                    "<id numa variável>".to_owned()
                } else {
                    c.ids.join(", ")
                }
            ));
        }
    }
    assert!(
        sem.is_empty(),
        "campos com coluna de losango SEM canal e sem porquê ({}). Ponha o id numa tabela de \
         canais ({TABELAS:?}) ou classifique-o em POR_ID / POR_FUNCAO (plano 35 §5.2, §8):\n  {}",
        sem.len(),
        sem.join("\n  ")
    );
}

/// ⛔ Censo: cada entrada de cada tabela de isenção ainda casa alguma coisa, e um id que ganhou
/// canal sai da isenção.
#[test]
fn nenhuma_isencao_sobrevive_ao_codigo_que_a_explicava() {
    let pop = populacao();
    let (canais, texto_tabelas) = nomes_das_tabelas();
    let mut velhas = Vec::new();
    for &(chave, _, porque) in POR_ID {
        let casa = pop.iter().any(|c| {
            !na_divida(c.krate, &c.ficheiro)
                && !c.ids.iter().any(|id| canais.contains(id))
                && por_id(&c.ids).is_some_and(|e| e.0 == chave)
        });
        if !casa {
            velhas.push(format!("POR_ID {chave} ({porque})"));
        }
    }
    for &(krate, funcao, classe, nomes, porque) in POR_FUNCAO {
        let casa = pop
            .iter()
            .any(|c| c.krate == krate && c.funcao == funcao && c.ids.is_empty());
        if !casa {
            velhas.push(format!("POR_FUNCAO {krate}::{funcao} ({porque})"));
        }
        if classe == Canal {
            let ficheiro = pop
                .iter()
                .find(|c| c.krate == krate && c.funcao == funcao)
                .map(|c| c.ficheiro.clone())
                .unwrap_or_default();
            let fonte = fontes(krate)
                .into_iter()
                .find(|(f, _)| *f == ficheiro)
                .map(|(_, s)| s)
                .unwrap_or_default();
            assert!(!nomes.is_empty(), "{krate}::{funcao} é Canal sem nomes");
            for n in nomes {
                if !fonte.contains(n) || !texto_tabelas.contains(n) {
                    velhas.push(format!(
                        "POR_FUNCAO {krate}::{funcao}: `{n}` não está no ficheiro E numa tabela"
                    ));
                }
            }
        }
    }
    for &(krate, ficheiro, porque) in DIVIDA_FICHEIRO {
        if !pop
            .iter()
            .any(|c| c.krate == krate && c.ficheiro == ficheiro)
        {
            velhas.push(format!("DIVIDA_FICHEIRO {krate}/{ficheiro} ({porque})"));
        }
    }
    for &(krate, linha) in LINHAS {
        let fonte: String = fontes(krate).into_iter().map(|(_, s)| s).collect();
        let portas: BTreeSet<&str> = PORTAS
            .iter()
            .copied()
            .chain(LINHAS.iter().filter(|(k, _)| *k == krate).map(|(_, f)| *f))
            .collect();
        // Uma linha é uma função que chama uma porta, ou o nome curto de uma (`use … as linha`).
        let chama = funcoes(&fonte)
            .iter()
            .any(|(n, corpo)| n == linha && !chamadas(corpo, &portas).is_empty())
            || PORTAS
                .iter()
                .any(|p| fonte.contains(&format!("{p} as {linha};")));
        if !chama {
            velhas.push(format!("LINHAS {krate}::{linha} não chama uma porta"));
        }
    }
    for c in &pop {
        for id in &c.ids {
            if canais.contains(id) && por_id(std::slice::from_ref(id)).is_some() {
                velhas.push(format!("POR_ID isenta `{id}`, que já tem canal"));
            }
        }
    }
    velhas.sort();
    velhas.dedup();
    assert!(
        velhas.is_empty(),
        "isenções sem objecto (apague-as ou corrija-as):\n  {}",
        velhas.join("\n  ")
    );
}

/// Toda crate que pinta campos com losango está no [`ESCOPO`] ou em [`DIVIDA_CRATE`] — e cada
/// dívida de crate ainda pinta algum.
#[test]
fn toda_crate_que_pinta_campos_escolheu_o_seu_lado() {
    let portas: BTreeSet<&str> = PORTAS.iter().copied().collect();
    let pinta = |krate: &str| {
        fontes(krate)
            .iter()
            .any(|(_, s)| !chamadas(s, &portas).is_empty())
    };
    let mut erros = Vec::new();
    let Ok(dir) = std::fs::read_dir(crates_root()) else {
        panic!("crates/ ilegível");
    };
    let mut todas: Vec<String> = dir
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|n| n.starts_with("ph2d-panel-"))
        .collect();
    todas.sort();
    for k in &todas {
        let dentro =
            ESCOPO.iter().any(|(e, _)| e == k) || DIVIDA_CRATE.iter().any(|(d, ..)| d == k);
        if pinta(k) && !dentro {
            erros.push(format!(
                "{k} pinta campos com losango e não está no ESCOPO nem em DIVIDA_CRATE"
            ));
        }
    }
    for &(k, _, porque) in DIVIDA_CRATE {
        if !pinta(k) {
            erros.push(format!("DIVIDA_CRATE {k} ({porque}) já não pinta campos"));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// Onde, no [`ESCOPO`] fora da dívida de ficheiro, cada primitivo [`SEM_COLUNA`] é pintado:
/// (ficheiro, função, primitivo, a função pinta a coluna?).
fn sem_coluna() -> Vec<(String, String, &'static str, bool)> {
    let mut out = Vec::new();
    for &(krate, so) in ESCOPO {
        for (ficheiro, s) in fontes(krate) {
            if so.is_some_and(|fs| !fs.contains(&ficheiro.as_str())) || na_divida(krate, &ficheiro)
            {
                continue;
            }
            for (funcao, corpo) in funcoes(&s) {
                let coluna = COLUNA.iter().any(|c| corpo.contains(c));
                // O ícone pintado direto, pela porta do painel (`.icon(`) ou por um helper do card
                // (`.filter_icon(`).
                let icone = ["paint_icon_button(", ".icon(", ".filter_icon("]
                    .iter()
                    .any(|p| corpo.contains(p));
                for p in SEM_COLUNA {
                    let olho = p.contains("Eye") || p.starts_with("eye_");
                    if corpo.contains(p) && (!olho || icone) {
                        out.push((ficheiro.clone(), funcao.clone(), *p, coluna));
                    }
                }
            }
        }
    }
    out
}

/// ⭐ **A lei da COLUNA** (F24d): uma caixa de marcar, um chip de menu, um seletor pintado direto ou
/// um olho só se pintam numa função que pinta a coluna do decorator — onde o losango mora e onde
/// este gate os vê. (Mutação: o `checkbox_row` sem o `decorator` ⇒ reprova.)
#[test]
fn todo_campo_sem_coluna_propria_ganha_a_do_losango() {
    let todos = sem_coluna();
    let sem: Vec<String> = todos
        .iter()
        .filter(|(.., coluna)| !coluna)
        .map(|(f, func, p, _)| format!("{f} · fn {func} pinta `{p}` sem a coluna do losango"))
        .collect();
    assert!(
        sem.is_empty(),
        "campos pintados SEM a coluna do losango — use as linhas do painel (`checkbox_row`, \
         `dropdown_row`) ou pinte o decorator na função:\n  {}",
        sem.join("\n  ")
    );
    // Controlo do leitor: os sítios conhecidos são achados.
    for (func, p) in [
        ("checkbox_row", "paint_checkbox("),
        ("dropdown_row", "paint_dropdown_chip("),
        ("fill_type_section", "paint_segmented_group_adaptive("),
        ("layer_row", "eye_icon("),
        ("effect_header", "IconId::Eye"),
        ("filter_header", "IconId::Eye"),
        ("paint_blend_chip", "paint_dropdown_chip("),
    ] {
        assert!(
            todos.iter().any(|(_, f, q, _)| f == func && *q == p),
            "a varredura não achou `{p}` em `fn {func}` — o leitor partiu"
        );
    }
}

/// Controlo do próprio leitor: a varredura acha a população conhecida (sem isto, um leitor que
/// casasse ZERO aprovaria tudo).
#[test]
fn a_varredura_acha_os_campos_que_se_conhecem() {
    let pop = populacao();
    let tem = |id: &str| pop.iter().any(|c| c.ids.iter().any(|x| x == id));
    for id in [
        "VECTOR_WIDTH",
        "VECTOR_FILL_SWATCH",
        "VECTOR_CONTOUR_STEPS",
        "VECTOR_PAINT_OPACITY",
        "VECTOR_OBJ_OPACITY_NUM",
        "INSP_TRANSFORM_POS_X",
        "INSP_SPRITE_OPACITY",
        "INSP_TRANSFORM_SKEW_X",
        "INSP_SPRITE_EMISSIVE",
        "INSP_SPRITE_TINT_SWATCH",
        "INSP_SPRITE_TINT_FILL",
        "INSP_JOINT_MOTOR",
        "INSP_JOINT_MOTOR_AXIS",
        "INSP_JOINT_BREAK_FORCE",
        "INSP_JOINT_AXIS_MODE",
        "VECTOR_OBJ_BLEND",
        "VECTOR_PAINT_BLEND",
        "VECTOR_STROKE_PRESENT",
        "VECTOR_FILL_KIND_SOLID",
        "VECTOR_BAR_KIND",
        "TexPatKnob::Angle",
        "TexPatKnob::Lock",
        "TexPatKnob::Offset",
        "VECTOR_BRUSH_FLIP",
        "VECTOR_TEXT_WRAP_AUTO",
        "VECTOR_TEXT_FONT_DD",
        "VECTOR_BAR_PROFILE",
        "VECTOR_ENVELOPE_BEND",
        "VECTOR_ANCHOR_H_START",
        "VECTOR_FRAME_CLIP_ON",
        "VECTOR_LAYOUT_DIR_ROW",
        "VECTOR_WIDGET_ICON_DD",
        "VECTOR_PATTERNPATH_SPACING",
        "VECTOR_TEXTPATH_OFFSET",
        "FLIP_SIZE",
        "FLIP_COLORIZE_SWATCH",
        "INSP_PHYS_KIND",
        "INSP_PHYS_FORCE_AXES",
        "INSP_PHYS_ONEWAY",
        "INSP_PHYS_DENSITY",
    ] {
        assert!(
            tem(id),
            "a varredura não achou `{id}` — o leitor do fonte partiu"
        );
    }
    assert!(pop.len() > 100, "população suspeita: {}", pop.len());
}
