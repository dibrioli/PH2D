//! As listas de escolhas do vocabulário do vetor ([`super`]): a chave `ph2d-i18n` do rótulo de cada
//! variante, pela ORDEM do código que o documento guarda (F27 F, o censo em
//! `docs/UI_New_and_Simple/F27_F_plano.md`). `""` = um código sem variante.

/// A junção (miter · round · bevel) de três painéis com chaves próprias.
pub(super) const JUNCAO_TRACO: &[&str] = &[
    "panel.vector.stroke.miter",
    "panel.vector.stroke.round",
    "panel.vector.stroke.bevel",
];

pub(super) const PONTAS: &[&str] = &[
    "panel.vector.engine.ponta.none",
    "panel.vector.engine.ponta.arrow",
    "panel.vector.engine.ponta.open",
    "panel.vector.engine.ponta.diamond",
    "panel.vector.engine.ponta.diamond_hollow",
    "panel.vector.engine.ponta.circle",
    "panel.vector.engine.ponta.circle_hollow",
    "panel.vector.engine.ponta.bar",
];

pub(super) const FILTROS: &[&str] = &[
    "panel.vector.engine.filtro.blur",
    "panel.vector.engine.filtro.glow",
    "panel.vector.engine.filtro.drop_shadow",
    "panel.vector.engine.filtro.inner_shadow",
    "panel.vector.engine.filtro.inner_glow",
    "panel.vector.engine.filtro.outline",
    "panel.vector.engine.filtro.feather",
    "panel.vector.engine.filtro.bevel",
    "panel.vector.engine.filtro.color_overlay",
    "panel.vector.engine.filtro.turbulence",
    "panel.vector.engine.filtro.grow_shrink",
    "panel.vector.engine.filtro.color_adjust",
    "panel.vector.engine.filtro.duotone",
    "panel.vector.engine.filtro.luma_to_alpha",
    "panel.vector.engine.filtro.gradient_map",
];

pub(super) const PROXIMIDADE: &[&str] = &[
    "panel.vector.engine.modo.proximity",
    "panel.vector.engine.modo.contour",
];

/// O modo de um filtro depende do TIPO dele (`FxOp::spec(kind).modes`).
pub(super) const MODOS_DO_FILTRO: &[(u16, &[&str])] = &[
    (1, PROXIMIDADE),
    (3, PROXIMIDADE),
    (4, PROXIMIDADE),
    (
        9,
        &[
            "panel.vector.engine.modo.smooth",
            "panel.vector.engine.modo.creased",
        ],
    ),
    (
        14,
        &[
            "panel.vector.engine.modo.linear",
            "panel.vector.engine.modo.smooth",
        ],
    ),
];

pub(super) const WARPS: &[&str] = &[
    crate::canal_palavra::NENHUM,
    "panel.vector.engine.efeito.arc",
    "panel.vector.engine.efeito.arc_upper",
    "panel.vector.engine.efeito.arc_lower",
    "panel.vector.engine.efeito.bulge",
    "panel.vector.engine.efeito.flag",
    "panel.vector.engine.efeito.wave",
    "panel.vector.engine.efeito.squeeze",
    "panel.vector.engine.efeito.fisheye",
    "panel.vector.engine.efeito.rise",
];

/// `0` não é pele (`WidgetKind::from_code(0)`).
pub(super) const PELES: &[&str] = &[
    "",
    "panel.vector.widget.kind.button",
    "panel.vector.widget.kind.toggle",
    "panel.vector.widget.kind.checkbox",
    "panel.vector.widget.kind.slider",
    "panel.vector.widget.kind.progress",
    "panel.vector.widget.kind.tag",
    "panel.vector.widget.kind.text_input",
    "panel.vector.widget.kind.card",
    "panel.vector.widget.kind.section",
    "panel.vector.widget.kind.list_item",
    "panel.vector.widget.kind.spinner",
    "panel.vector.widget.kind.divider",
    "panel.vector.widget.kind.number_input",
    "panel.vector.widget.kind.level_meter",
    "panel.vector.widget.kind.color_swatch",
    "panel.vector.widget.kind.icon_button",
    "panel.vector.widget.kind.tabs",
    "panel.vector.widget.kind.radio_group",
    "panel.vector.widget.kind.segmented_adaptive",
    "panel.vector.widget.kind.dropdown",
];

/// Um campo dentro de um bloco de tinta: a tinta inteira é um grupo (por `i`).
pub(super) const TIPOS_DO_PREENCHIMENTO: &[&str] = &[
    "panel.vector.arrange.solid",
    "panel.vector.arrange.linear",
    "panel.vector.arrange.radial",
    "panel.vector.arrange.multi",
    "panel.vector.arrange.pattern",
];

/// `1..=3` não existem no traço.
pub(super) const TIPOS_DO_TRACO: &[&str] = &[
    "panel.vector.stroke.solid",
    "",
    "",
    "",
    "panel.vector.stroke.pattern",
    "panel.vector.stroke.brush",
];

pub(super) const MOSAICOS: &[&str] = &[
    "panel.vector.texpat.grid",
    "panel.vector.texpat.brick",
    "panel.vector.texpat.column",
    "panel.vector.texpat.hex",
];

pub(super) const REPETICOES: &[&str] = &[
    "panel.vector.texpat.tile",
    "panel.vector.texpat.mirror",
    "panel.vector.texpat.clamp",
];

pub(super) const TIPO_VERTICE: &[&str] = &[
    "panel.vector.vertex.corner",
    "panel.vector.vertex.smooth",
    "panel.vector.vertex.symm",
];

pub(super) const REGRA: &[&str] = &[
    "panel.vector.arrange.non_zero",
    "panel.vector.arrange.even_odd",
];

pub(super) const TRACO_PONTA: &[&str] = &[
    "panel.vector.stroke.butt",
    "panel.vector.stroke.round",
    "panel.vector.stroke.square",
];

pub(super) const TRACO_ALINHAMENTO: &[&str] = &[
    "panel.vector.stroke.centre",
    "panel.vector.stroke.inner",
    "panel.vector.stroke.outer",
];

pub(super) const PILHA_DILATA_JUNCAO: &[&str] = &[
    "panel.vector.appearance.miter",
    "panel.vector.appearance.round",
    "panel.vector.appearance.bevel",
];

pub(super) const TEXTO_ALINHAMENTO: &[&str] = &[
    "panel.vector.text.left",
    "panel.vector.text.center",
    "panel.vector.text.right",
];

pub(super) const TEXTO_QUEBRA_FIXA: &[&str] =
    &["panel.vector.text.auto", "panel.vector.text.fixed"];

pub(super) const PADRAO_CAMINHO_ESPELHO: &[&str] = &[
    "panel.vector.patternpath.this_side",
    "panel.vector.patternpath.other_side",
];

pub(super) const TEXTO_CAMINHO_ESPELHO: &[&str] = &[
    "panel.vector.textpath.this_side",
    "panel.vector.textpath.other_side",
];

pub(super) const ENVELOPE_TIPO: &[&str] = &[
    "panel.vector.envelope.persp",
    "panel.vector.envelope.mesh",
    "panel.vector.envelope.pins",
];

pub(super) const LAYOUT_DIR: &[&str] = &[
    "panel.vector.layout.dir.off",
    "panel.vector.layout.dir.row",
    "panel.vector.layout.dir.col",
    "panel.vector.layout.dir.wrap",
    "panel.vector.layout.dir.grid",
];

pub(super) const LAYOUT_ALINHA: &[&str] = &[
    "panel.vector.layout.align.start",
    "panel.vector.layout.align.center",
    "panel.vector.layout.align.end",
    "panel.vector.layout.align.stretch",
];

pub(super) const LAYOUT_DISTRIBUI: &[&str] = &[
    "panel.vector.layout.justify.start",
    "panel.vector.layout.justify.center",
    "panel.vector.layout.justify.end",
    "panel.vector.layout.justify.between",
    "panel.vector.layout.justify.around",
];

pub(super) const LAYOUT_TAMANHO: &[&str] = &[
    "panel.vector.layout.size.fixed",
    "panel.vector.layout.size.hug",
];

pub(super) const CONTORNO_JUNCAO: &[&str] = &[
    "panel.vector.contour.miter",
    "panel.vector.contour.round",
    "panel.vector.contour.bevel",
];

pub(super) const CONTORNO_LADO: &[&str] = &[
    "panel.vector.contour.outer",
    "panel.vector.contour.inner",
    "panel.vector.contour.both",
];

pub(super) const OFFSET_JUNCAO: &[&str] = &[
    "panel.vector.expand.miter",
    "panel.vector.expand.round",
    "panel.vector.expand.bevel",
];

pub(super) const OFFSET_LADO: &[&str] = &[
    "panel.vector.expand.outer",
    "panel.vector.expand.inner",
    "panel.vector.expand.both",
];
