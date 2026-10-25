//! ⭐⭐⭐ **TODO CAMPO EDITÁVEL TEM CANAL** (plano 35 §5.2) — o gate que torna «o Auto Key grava
//! absolutamente tudo» (dono, 19/10) verificável: um campo novo sem canal reprova.
//!
//! Lê o FONTE, como o `architecture_panel_wiring_parity`. A população é cada chamada de uma PORTA
//! com coluna de losango ([`PORTAS`] — as linhas de `property_row`, o slider com chip, a escolha, o
//! chip numérico do rail, o próprio `paint_decorator_for`) ou de um helper de linha de um painel
//! que só a repassa ([`LINHAS`]). Cada chamada tem de cair numa classe:
//!
//! - **canal pelo rótulo** (F27 B2): o nome dela é `TextKey::new("component.field.…")`, um campo do
//!   catálogo com canal (`ph2d_ecs::canal_componente::chaves_do_rotulo`) — a porta regista-o;
//! - **canal**: um dos ids dela está numa tabela de canais ([`TABELAS`]: `ph2d_app_vec::marcas`,
//!   `ph2d_app_components::marcas_inspector`, `ph2d_app_flip::marcas_camada`,
//!   `ph2d_app_constraints::marcas`);
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
    "paint_check_row",
    "paint_check_rows",
    "paint_decorator_for",
    "paint_decorator_campo",
    // O ponto pintado à mão (sem ids): uma linha que o desenha nunca tem losango (F27 B2).
    "paint_decorator_dot",
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
const COLUNA: &[&str] = &[
    "paint_decorator_for(",
    "paint_decorator_campo(",
    "paint_decorator_dot(",
    "paint_check_row(",
    "coluna_da_caixa(",
    ".decorator(",
];

/// As crates cujos campos este gate classifica um a um; `None` = todos os ficheiros de `src/`,
/// `Some` = só estes (o resto da crate é dívida do bloco 2).
const ESCOPO: &[(&str, Option<&[&str]>)] = &[
    ("ph2d-panel-vector", None),
    // ⭐ F27 B2: o Inspector INTEIRO — o canal sai do rótulo do catálogo que a linha pinta.
    ("ph2d-panel-inspector", None),
    ("ph2d-panel-flip", None),
    // ⭐ F30 §B: o MUNDO da física é canal da Cena (`ph2d_app_physics::marcas_mundo`).
    ("ph2d-panel-physics", None),
];

/// As tabelas de canais (caminhos a partir de `crates/`).
const TABELAS: &[&str] = &[
    "ph2d-app-vec/src/marcas.rs",
    "ph2d-app-vec/src/marcas_vivos.rs",
    "ph2d-app-components/src/marcas_inspector.rs",
    "ph2d-app-flip/src/marcas_camada.rs",
    "ph2d-app-constraints/src/marcas.rs",
    "ph2d-app-physics/src/marcas_mundo.rs",
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
    ("ph2d-panel-inspector", "check_row"),
    ("ph2d-panel-inspector", "caixa"),
    ("ph2d-panel-inspector", "segmented_row"),
    ("ph2d-panel-inspector", "text_row"),
    ("ph2d-panel-inspector", "grupo"),
    ("ph2d-panel-inspector", "nome_row"),
    ("ph2d-panel-inspector", "signal_row"),
    ("ph2d-panel-inspector", "txt_row"),
    ("ph2d-panel-inspector", "coluna_da_caixa"),
    ("ph2d-panel-flip", "slider_row"),
    ("ph2d-panel-flip", "slider_row_linked"),
    ("ph2d-panel-physics", "check"),
];

/// A classe de uma chamada por um dos seus ids — exacto, ou PREFIXO quando acaba em `_`.
const POR_ID: &[(&str, Classe, &str)] = &[
    (
        "INSP_CAMERA_CUSTOM_W",
        NaoAnimavel,
        "definição do PROJECTO (blob `jogo`, plano GEF 17/17b), não da cena: o formato do jogo, a moldura e o ícone no canvas não mudam no tempo",
    ),
    (
        "INSP_CAMERA_FRAME_GROUP",
        NaoAnimavel,
        "definição do PROJECTO (blob `jogo`, plano GEF 17/17b), não da cena: o formato do jogo, a moldura e o ícone no canvas não mudam no tempo",
    ),
    (
        "INSP_CAMERA_ICON_SIZE",
        NaoAnimavel,
        "definição do PROJECTO (blob `jogo`, plano GEF 17/17b), não da cena: o formato do jogo, a moldura e o ícone no canvas não mudam no tempo",
    ),
    (
        "INSP_IMAGE_USE",
        NaoAnimavel,
        "o seletor «Use» diz que componentes EXISTEM (animações, `TiledDraw`, repeat): acrescentar/tirar é estrutura, não um valor no tempo (plano GEF 15)",
    ),
    (
        "PHYSICS_SHOW_COLLIDERS",
        NaoAnimavel,
        "o desenho dos colisores é debug da vista (§8)",
    ),
    (
    "INSP_SCRIPT_SOURCE",
    Canal,
    "o mesmo canal que o seletor `INSP_SCRIPT_PICK` (o `ScriptId`), que leva o losango: um canal, um losango; o caminho por importar não é valor de canal (F27 F)",
),
    (
        "INSP_VIS_ON_SCREEN",
        NaoAnimavel,
        "a caixa diz se o componente EXISTE (acrescentar/tirar é estrutura); o rectângulo dele tem canal",
    ),
    (
        "INSP_VIS_MASK_SOURCE",
        NaoAnimavel,
        "a caixa diz se o componente Mask2D EXISTE: estrutura, não valor",
    ),
    (
        "INSP_LUZ_ALCANCE_LIGADO",
        NaoAnimavel,
        "o interruptor só diz se o alcance tem número; o número tem canal (`luz_fields.4`)",
    ),
    (
        "INSP_PLAYER_EMIT",
        NaoAnimavel,
        "a caixa diz se o marcador `PlayerSignals` EXISTE: estrutura, não valor (a lei do Mask2D)",
    ),
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
/// variável pode ter estão no ficheiro da função E numa tabela (ou são rótulos do catálogo com
/// canal). ⭐ `ficheiro.rs::fn` (F27 B2) nomeia UMA função e vale para todas as chamadas dela — o
/// rótulo vem de uma tabela, mesmo com o id literal.
const POR_FUNCAO: &[(&str, &str, Classe, &[&str], &str)] = &[
    (
        "ph2d-panel-inspector",
        "sections/camera_format.rs::format_row",
        NaoAnimavel,
        &[],
        "definição do PROJECTO (blob `jogo`, plano GEF 17/17b), não da cena: o formato do jogo, a moldura e o ícone no canvas não mudam no tempo",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anim_rules.rs::rule_rows",
        NaoAnimavel,
        &[],
        "a LISTA de regras «quando → toca» do cartão Animation (GEF 08) é a lógica que ESCOLHE o clip a cada quadro — animá-la seria uma regra a decidir outra",
    ),
    (
        "ph2d-panel-inspector",
        "sections/action_trigger.rs::linha_da_aresta",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions.rs::verb_row",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions.rs::tag_pick_row",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions_editor.rs::clip_row",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/counter_watch.rs::linha_da_comparacao",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/identity.rs::paint_entity_name_row",
        NaoAnimavel,
        &[],
        "o NOME do objecto é a referência durável de quem o nomeia: renomeá-lo no tempo partiria os alvos",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tags.rs::pick_row",
        NaoAnimavel,
        &[],
        "escolher uma tag para acrescentar: um verbo, não um valor",
    ),
    (
        "ph2d-panel-vector",
        "state_cad.rs::chips",
        Canal,
        &["VECTOR_CAD_VALUE"],
        "o número da cota ESCOLHIDA: o canal é o `canal_cad` da regra escolhida (j = o `RegraId`), resolvido por \
         `ph2d_app_vec::marcas::canal_do_campo` pelo `Contexto.regra` (W4.6)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/ordering.rs::number_row",
        Canal,
        &["component.field.z_index.1", "component.field.y_sort.2"],
        "o rótulo do catálogo vem do chamador (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/constraint.rs::ajustes",
        Canal,
        &["INSP_CONSTRAINT_NUMERO", "INSP_CONSTRAINT_OPCAO"],
        "o ajuste j da linha k: o campo do canal vem do tipo do item (`ph2d_app_constraints::campos`), a mesma tabela que o `marcas` lê",
    ),
    (
        "ph2d-panel-inspector",
        "sections/constraint_valor.rs::propriedade",
        NaoAnimavel,
        &[],
        "o Follow/Drive do Follow Value escolhe o ENDEREÇO de uma propriedade (estrutura do item), não um valor da cena",
    ),
    (
        "ph2d-panel-inspector",
        "sections/ordering.rs::layer_row",
        Canal,
        &["component.field.sorting_layer.1"],
        "a camada de ordenação guarda o ID inteiro (`View::Named(Layer)`): canal por degraus (F27 D)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/sprite_pose.rs::number_row",
        Canal,
        &["component.field.sprite_grid.1", "component.field.sprite.10"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/visibility.rs::number_row",
        Canal,
        &[
            "component.field.clip_children.1",
            "component.field.mask_interaction.2",
        ],
        "o rótulo do catálogo vem do chamador (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/audio.rs::source_body",
        Canal,
        &[
            "component.field.source_fields.0",
            "component.field.source_fields.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/factory.rs::factory_body",
        Canal,
        &[
            "component.field.factory_fields.1",
            "component.field.factory_fields.12",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/hud.rs::num_row",
        Canal,
        &[
            "component.field.canvas_fields.1",
            "component.field.canvas_fields.2",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/hud_corpo.rs::bloco_raiz",
        Canal,
        &["component.field.canvas_fields.3"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/hud_corpo.rs::bloco_contador",
        Canal,
        &[
            "component.field.counter_fields.1",
            "component.field.counter_fields.3",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/macro_card.rs::paint_macro_section",
        Canal,
        &["component.field.macro_fields.2"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/motion_input.rs::paint_motion_input_section",
        Canal,
        &[
            "component.field.motion_input_fields.1",
            "component.field.motion_input_fields.2",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/nav.rs::paint_nav_agent_section",
        Canal,
        &["component.field.nav_agent.1", "component.field.nav_agent.2"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/nav_custo.rs::interruptores_do_agente",
        Canal,
        &[
            "component.field.nav_agent.10",
            "component.field.nav_agent.5",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/nav_custo.rs::paint_nav_link_section",
        Canal,
        &["component.field.nav_link.0", "component.field.nav_link.1"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/ordering.rs::call",
        Canal,
        &[
            "component.field.sorting_group.1",
            "component.field.y_sort.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/parallax.rs::corpo",
        Canal,
        &[
            "component.field.limits_fields.0",
            "component.field.limits_fields.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/particles.rs::num_row",
        Canal,
        &["component.field.fields.11", "component.field.fields.12"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/particles.rs::vec_row",
        Canal,
        &["component.field.fields.11", "component.field.fields.12"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/particles_avisos.rs::corpo",
        Canal,
        &["component.field.fields.1", "component.field.fields.10"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/particles_avisos.rs::sinais",
        Canal,
        &["component.field.fields.24", "component.field.fields.25"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/path_follow.rs::corpo",
        Canal,
        &[
            "component.field.path_follow_fields.1",
            "component.field.path_follow_fields.10",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/player.rs::paint_cards",
        Canal,
        &[
            "component.field.platform_player.52",
            "component.field.platform_player.53",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/projectile.rs::corpo",
        Canal,
        &[
            "component.field.projectile_motion.0",
            "component.field.projectile_motion.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/ray.rs::corpo",
        Canal,
        &[
            "component.field.ray_sensor.0",
            "component.field.ray_sensor.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/shake.rs::corpo",
        Canal,
        &[
            "component.field.shake_fields.0",
            "component.field.shake_fields.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/slice_nine.rs::paint_slice_section",
        Canal,
        &["component.field.slice_nine.7"],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/topdown.rs::corpo",
        Canal,
        &[
            "component.field.top_down_player.0",
            "component.field.top_down_player.1",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/topdown.rs::deslize",
        Canal,
        &[
            "component.field.top_down_player.10",
            "component.field.top_down_player.8",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida.rs::numeros",
        Canal,
        &[
            "component.field.health_fields.1",
            "component.field.health_fields.10",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida.rs::nome",
        Canal,
        &[
            "component.field.health_fields.1",
            "component.field.health_fields.10",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida_barra.rs::corpo_barra",
        Canal,
        &[
            "component.field.health_bar_fields.1",
            "component.field.health_bar_fields.10",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida_dano.rs::corpo_dano",
        Canal,
        &[
            "component.field.damage_fields.1",
            "component.field.damage_fields.10",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida_impacto.rs::corpo_impacto",
        Canal,
        &[
            "component.field.health_fields.21",
            "component.field.health_fields.22",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/weapon.rs::corpo",
        Canal,
        &[
            "component.field.weapon_fields.1",
            "component.field.weapon_fields.2",
        ],
        "o rótulo do catálogo vem da tabela da secção (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/action_trigger.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions.rs::target_rows",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions_editor.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/actions_editor.rs::from_row",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anchors.rs::anchor_editor",
        Divida,
        &[],
        "item de LISTA (`VecAnchors`): a família COMPONENTE endereça campos, não o item n",
    ),
    (
        "ph2d-panel-inspector",
        "sections/counter_watch.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sync_counter_watch.rs::sync",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/timers.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/path_follow.rs::relogio",
        Divida,
        &[],
        "o relógio do PathFollow é um item da LISTA `Timers` (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tween_editor.rs::relogio",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tween_editor.rs::campos_de_para",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tween_editor.rs::curva_e_ease",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tween_editor.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/statemachine.rs::paint_statemachine_section",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/statemachine.rs::estados",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/statemachine_setas.rs::setas",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/shake_emitter.rs::editor",
        Divida,
        &[],
        "item de LISTA do componente: a família COMPONENTE endereça campos, não o item n (F27 B2 §ABERTO)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/vida_resist.rs::corpo_resistencias",
        Divida,
        &[],
        "item de LISTA (as resistências): a família COMPONENTE endereça campos, não o item n",
    ),
    (
        "ph2d-panel-inspector",
        "sections/hud_corpo.rs::bloco_rotulo",
        Canal,
        &["component.field.label_fields.2"],
        "o «Source Name» é o conteúdo da variante (`source:*`), canal de texto; o rótulo vem da tabela do bloco (F27 E)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/script_controlo.rs::controlo",
        Canal,
        &["regista_canal"],
        "as propriedades que o SCRIPT declara: o instantâneo traz a chave (família SCRIPT, pelo NOME), a `linha` regista-a e o controlo pinta a coluna (F27 E)",
    ),
    (
        "ph2d-panel-inspector",
        "event_vida.rs::apply_vida_event",
        NaoAnimavel,
        &[],
        "o `caixa(` daqui é o leitor do evento, não uma linha pintada (nome partilhado)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anim.rs::player_block",
        NaoAnimavel,
        &[],
        "a duração de UM quadro do clip: o parâmetro da própria animação (§8)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anim_rows.rs::range_and_timing_rows",
        NaoAnimavel,
        &[],
        "o intervalo e o tempo do clip: o parâmetro da própria animação (§8)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anim_rows.rs::editor",
        NaoAnimavel,
        &[],
        "o nome, a direcção e os sinais do clip: o parâmetro da própria animação (§8)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/render_source.rs::paint_strategy_row",
        NaoAnimavel,
        &[],
        "como o asset se empacota (atlas/individual): importação, não cena",
    ),
    (
        "ph2d-panel-inspector",
        "sections/render_source_precision.rs::paint_precision_row",
        NaoAnimavel,
        &[],
        "o formato de pixel do asset: importação, não cena",
    ),
    (
        "ph2d-panel-inspector",
        "sections/sprite_pose.rs::sheet_preview_row",
        NaoAnimavel,
        &[],
        "mostrar a folha no canvas: estado da vista",
    ),
    (
        "ph2d-panel-inspector",
        "sections/anim_card.rs::paint_chooser",
        NaoAnimavel,
        &[],
        "«Who Picks» diz se o `CharacterAnimations` EXISTE (e o autoplay de partida): estrutura, não \
         valor (a lei do Mask2D)",
    ),
    (
        "ph2d-panel-inspector",
        "sections/tags.rs::paint_tags_section",
        NaoAnimavel,
        &[],
        "a caixa de uma tag NOVA é um verbo (acrescentar), não um valor",
    ),
    (
        "ph2d-panel-inspector",
        "sections/motion_input.rs::nomes_do_grafo",
        NaoAnimavel,
        &[],
        "os nomes que o grafo lê: leitura do grafo, não um campo",
    ),
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
        "a curva da transição de um estado é parâmetro de uma animação; não se anima a si própria",
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
        "camera_body",
        Canal,
        &["component.field.camera_fields.1"],
        "o par offset da câmera: o rótulo do catálogo vem da lista local (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "follow_body",
        Canal,
        &[
            "component.field.follow_fields.1",
            "component.field.follow_fields.2",
            "component.field.follow_fields.3",
        ],
        "os pares do seguimento: o rótulo do catálogo vem da lista local (F27 B2)",
    ),
    (
        "ph2d-panel-inspector",
        "limits_body",
        Canal,
        &[
            "component.field.limits_fields.0",
            "component.field.limits_fields.1",
        ],
        "os pares min/max dos limites: o rótulo do catálogo vem da lista local (F27 B2)",
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
        "ph2d-panel-physics",
        "paint.rs::paint_row",
        Canal,
        &[
            "PHYSICS_GRAVITY_X",
            "PHYSICS_GRAVITY_Y",
            "PHYSICS_SUBSTEPS",
            "PHYSICS_ITERATIONS",
            "PHYSICS_CONTACT_HZ",
            "PHYSICS_AIR_DRAG",
            "PHYSICS_LINEAR_DAMPING",
            "PHYSICS_ANGULAR_DAMPING",
            "PHYSICS_SLEEP_SPEED",
            "PHYSICS_SLEEP_DELAY",
        ],
        "as linhas da tabela `rows::SECTIONS` — o mundo da física sobre a Cena (canal MUNDO, F30 §B)",
    ),
    (
        "ph2d-panel-physics",
        "paint.rs::paint_irow",
        NaoAnimavel,
        &[],
        "a ferramenta de INTERACÇÃO (a mão no jogo a correr): da ferramenta, não da cena (§8)",
    ),
    (
        "ph2d-panel-physics",
        "paint/interact.rs::seg_row",
        NaoAnimavel,
        &[],
        "os rádios das ferramentas de interacção e de junta: da ferramenta, não da cena (§8)",
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

/// ⭐ F27 B3 — **cada primitivo [`SEM_COLUNA`] de uma crate fora do [`ESCOPO`] e da
/// [`DIVIDA_CRATE`] escolheu o seu lado**: (crate, `ficheiro.rs::fn`, classe, porquê). Antes só as
/// [`PORTAS`] contavam, e um painel com um chip ou um olho e nenhuma porta passava sem lado.
const SEM_COLUNA_FORA: &[(&str, &str, Classe, &str)] = &[
    (
        "ph2d-panel-hierarchy",
        "filter.rs::paint_chip",
        NaoAnimavel,
        "o filtro da lista é da vista (§8)",
    ),
    (
        "ph2d-panel-hierarchy",
        "row.rs::paint_row_eye",
        Canal,
        "o olho escreve o `Visibility.hidden` do objecto: o canal COMPONENTE da linha «Visible» \
         do Inspector (F27 B2); o Auto Key compara-o com a curva do objecto selecionado depois \
         de toda escrita da UI (`fase_autokey`), venha ela do olho ou da linha",
    ),
    (
        "ph2d-panel-skeleton",
        "section_tip.rs::tip_row",
        NaoAnimavel,
        "quem manda na ponta do osso é fiação, não propriedade",
    ),
    (
        "ph2d-panel-skeleton",
        "section_smart.rs::smart_action_row",
        NaoAnimavel,
        "o clip que o osso inteligente dispara é fiação, não propriedade",
    ),
    (
        "ph2d-panel-timeline",
        "transport_animation.rs::chip",
        NaoAnimavel,
        "que animação a Timeline mostra é da vista (§8)",
    ),
    (
        "ph2d-panel-timeline",
        "transport_clips.rs::cluster",
        NaoAnimavel,
        "de onde a aba de clips lê é da vista (§8)",
    ),
    (
        "ph2d-panel-timeline",
        "stack_lane_paint.rs::paint_lane",
        NaoAnimavel,
        "o mudo de uma faixa da pilha é parâmetro da animação; não se anima a si própria",
    ),
    (
        "ph2d-panel-flip-frames",
        "paint_toolbar.rs::dropdown_chip",
        NaoAnimavel,
        "o ease do tween e o ciclo do Flip são parâmetros da animação; não se animam a si próprios",
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
    /// O nome é um rótulo do catálogo com canal (a porta regista os ids sob ele).
    pelo_rotulo: bool,
}

/// O prefixo literal de um rótulo do catálogo passado tipado a uma porta.
const TEXTKEY_DO_CATALOGO: &str = "TextKey::new(\"component.field.";

/// Um rótulo do catálogo que é canal (tem chaves).
fn rotulo_e_canal(r: &str) -> bool {
    ph2d_ecs::canal_componente::rotulo_e_canal(r)
}

/// Os rótulos do catálogo tipados (`TextKey::new("component.field.…")`) de um texto.
fn rotulos_em(args: &str) -> Vec<&str> {
    args.match_indices(TEXTKEY_DO_CATALOGO)
        .filter_map(|(i, _)| {
            let ini = i + "TextKey::new(\"".len();
            args[ini..].find('"').map(|f| &args[ini..ini + f])
        })
        .collect()
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
                        pelo_rotulo: rotulos_em(args).into_iter().any(rotulo_e_canal),
                    });
                }
            }
        }
    }
    out
}

/// A entrada `f` de [`POR_FUNCAO`] casa a chamada `c`: o nome curto (só id numa variável) ou
/// `ficheiro.rs::fn` (todas as chamadas da função).
fn casa_funcao(krate: &str, f: &str, c: &Chamada) -> bool {
    krate == c.krate
        && match f.split_once(".rs::") {
            Some((fic, fun)) => c.funcao == fun && c.ficheiro == format!("{fic}.rs"),
            None => f == c.funcao && c.ids.is_empty(),
        }
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
        let ok = if c.pelo_rotulo || POR_FUNCAO.iter().any(|&(k, f, ..)| casa_funcao(k, f, &c)) {
            true
        } else if c.ids.is_empty() {
            e_linha(c.krate, &c.funcao)
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
                && !c.pelo_rotulo
                && !c.ids.iter().any(|id| canais.contains(id))
                && por_id(&c.ids).is_some_and(|e| e.0 == chave)
        });
        if !casa {
            velhas.push(format!("POR_ID {chave} ({porque})"));
        }
    }
    for &(krate, funcao, classe, nomes, porque) in POR_FUNCAO {
        let casa = pop.iter().any(|c| casa_funcao(krate, funcao, c));
        if !casa {
            velhas.push(format!("POR_FUNCAO {krate}::{funcao} ({porque})"));
        }
        if classe == Canal {
            let ficheiro = pop
                .iter()
                .find(|c| casa_funcao(krate, funcao, c))
                .map(|c| c.ficheiro.clone())
                .unwrap_or_default();
            // Um rótulo do catálogo pode vir da tabela de um ficheiro irmão (`player_rows.rs`).
            let fonte: String = fontes(krate)
                .into_iter()
                .filter(|(f, _)| *f == ficheiro || funcao.contains(".rs::"))
                .map(|(_, s)| s)
                .collect();
            assert!(!nomes.is_empty(), "{krate}::{funcao} é Canal sem nomes");
            for n in nomes {
                let canal = texto_tabelas.contains(n) || rotulo_e_canal(n);
                if !fonte.contains(n) || !canal {
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
            if (canais.contains(id) || c.pelo_rotulo) && por_id(std::slice::from_ref(id)).is_some()
            {
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
    let pinta_porta = |krate: &str| {
        fontes(krate)
            .iter()
            .any(|(_, s)| !chamadas(s, &portas).is_empty())
    };
    // (ficheiro, função, primitivo) de cada SEM_COLUNA da crate.
    let sem = |krate: &str| -> Vec<(String, String, &'static str)> {
        (fontes(krate).iter())
            .flat_map(|(f, s)| {
                primitivos(s)
                    .into_iter()
                    .map(|(func, p, _)| (f.clone(), func, p))
            })
            .collect()
    };
    let pinta = |krate: &str| pinta_porta(krate) || !sem(krate).is_empty();
    let mut erros = Vec::new();
    let mut usadas = BTreeSet::new();
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
        if dentro {
            continue;
        }
        if pinta_porta(k) {
            erros.push(format!(
                "{k} pinta campos com losango e não está no ESCOPO nem em DIVIDA_CRATE"
            ));
        }
        for (f, func, p) in sem(k) {
            let lado = SEM_COLUNA_FORA
                .iter()
                .position(|&(c, ff, ..)| c == k && ff == format!("{f}::{func}"));
            match lado {
                Some(i) => {
                    usadas.insert(i);
                }
                None => erros.push(format!(
                    "{k} · {f}::{func} pinta `{p}` sem lado (SEM_COLUNA_FORA, ESCOPO ou DIVIDA_CRATE)"
                )),
            }
        }
    }
    for (i, &(k, f, _, porque)) in SEM_COLUNA_FORA.iter().enumerate() {
        if !usadas.contains(&i) {
            erros.push(format!(
                "SEM_COLUNA_FORA {k} · {f} ({porque}) já não pinta nada"
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
            out.extend((primitivos(&s).into_iter()).map(|(f, p, c)| (ficheiro.clone(), f, p, c)));
        }
    }
    out
}

/// Os primitivos [`SEM_COLUNA`] de um fonte: (função, primitivo, a função pinta a coluna?).
fn primitivos(s: &str) -> Vec<(String, &'static str, bool)> {
    let mut out = Vec::new();
    for (funcao, corpo) in funcoes(s) {
        let coluna = COLUNA.iter().any(|c| corpo.contains(c));
        // O ícone pintado direto, pela porta do painel (`.icon(`), por um helper do card
        // (`.filter_icon(`), pelo pintor de ícones (`paint_icon(`, o olho da Hierarquia) ou entregue
        // a um botão (`_button(`, o mudo da faixa da Timeline).
        let icone = ["_button(", ".icon(", ".filter_icon(", "paint_icon("]
            .iter()
            .any(|p| corpo.contains(p));
        for p in SEM_COLUNA {
            let olho = p.contains("Eye") || p.starts_with("eye_");
            if casa(corpo, p) && (!olho || icone) {
                out.push((funcao.clone(), *p, coluna));
            }
        }
    }
    out
}

/// `p` em `corpo` — ⚠️ o `IconId::Eyedropper` (o conta-gotas que ARMA um pick) não é o olho
/// `IconId::Eye`: o prefixo casava-o, e os conta-gotas da §12/§13 liam-se como campos sem coluna.
fn casa(corpo: &str, p: &str) -> bool {
    corpo
        .match_indices(p)
        .any(|(i, _)| !corpo[i + p.len()..].starts_with("dropper"))
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
        "INSP_JOINT_KIND",
        "INSP_JOINT_ACTIVE",
        "INSP_JOINT_COLLIDE",
        "INSP_JOINT_ANCHOR_B",
        "INSP_WHEEL_RADIUS",
        "INSP_WHEEL_WRAP",
        "INSP_WHEEL_BREAK_FORCE",
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

/// ⭐⭐ **O Inspector é população INTEIRO** (F27 B2) — a lista de ficheiros deixava de fora o que
/// não estava nela (smoke do dono, 22/10: a junta e a roldana não gravavam). Controlo: os ficheiros
/// da junta são vistos.
#[test]
fn o_inspector_inteiro_e_populacao() {
    assert!(
        ESCOPO.contains(&("ph2d-panel-inspector", None)),
        "o Inspector voltou a ser uma lista de ficheiros"
    );
    let pop = populacao();
    let junta = (pop.iter())
        .filter(|c| c.krate == "ph2d-panel-inspector")
        .filter(|c| {
            c.ids
                .iter()
                .any(|i| i.starts_with("INSP_JOINT_") || i.starts_with("INSP_WHEEL_"))
        })
        .map(|c| c.ficheiro.as_str())
        .collect::<BTreeSet<_>>();
    assert!(junta.len() >= 6, "controlo: só {junta:?} — o leitor partiu");
    let pelo_rotulo = pop.iter().filter(|c| c.pelo_rotulo).count();
    assert!(
        pelo_rotulo > 50,
        "controlo: só {pelo_rotulo} linhas com canal pelo rótulo"
    );
}
