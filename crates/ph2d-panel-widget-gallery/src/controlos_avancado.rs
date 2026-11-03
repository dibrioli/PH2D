//! A tabela dos controlos ([`super`]) dos cartões Advanced e Render (doc `CartoesUI/08` §1.2).

use super::{Act, Control, Kind, icon, minus, opt, plain};
use crate::ids as g;
use crate::state::{Flag, Pick};
use ph2d_editor_core::IconId;

pub const META_ROWS: [Flag; 6] = [
    Flag::Meta0,
    Flag::Meta1,
    Flag::Meta2,
    Flag::Meta3,
    Flag::Meta4,
    Flag::Meta5,
];
pub const META_TYPES: [Pick; 6] = [
    Pick::Meta0,
    Pick::Meta1,
    Pick::Meta2,
    Pick::Meta3,
    Pick::Meta4,
    Pick::Meta5,
];
pub const PROP_ROWS: [Flag; 3] = [Flag::Prop0, Flag::Prop1, Flag::Prop2];
/// A primeira amostra de cor de Metadata (`controlos::SWATCHES`).
pub const META_SWATCH: usize = 13;
const SEM_AMOSTRA: usize = usize::MAX;

const fn entrada(i: usize) -> [Control; 6] {
    let r = g::META[i];
    [
        plain(r.kind, Kind::Box),
        opt(
            r.options[0],
            META_TYPES[i],
            0,
            "panel.widget_gallery.adv.type.float",
        ),
        opt(
            r.options[1],
            META_TYPES[i],
            1,
            "panel.widget_gallery.adv.type.text",
        ),
        opt(
            r.options[2],
            META_TYPES[i],
            2,
            "panel.widget_gallery.adv.type.color",
        ),
        plain(r.swatch, Kind::Swatch(META_SWATCH + i)),
        minus(
            r.remove,
            META_ROWS[i],
            META_SWATCH + i,
            "panel.widget_gallery.adv.tip.meta_remove",
        ),
    ]
}

const fn propriedade(i: usize) -> Control {
    icon(
        g::PROPS[i].remove,
        Kind::Remove(PROP_ROWS[i], SEM_AMOSTRA),
        IconId::Trash,
        "panel.widget_gallery.adv.tip.prop_remove",
    )
}

const fn add_meta(id: ph2d_a11y::NodeId, v: u8, text: &'static str) -> Control {
    Control {
        text: Some(text),
        ..plain(id, Kind::AddMeta(v))
    }
}

const M0: [Control; 6] = entrada(0);
const M1: [Control; 6] = entrada(1);
const M2: [Control; 6] = entrada(2);
const M3: [Control; 6] = entrada(3);
const M4: [Control; 6] = entrada(4);
const M5: [Control; 6] = entrada(5);
const P: [Control; 3] = [propriedade(0), propriedade(1), propriedade(2)];

const CAMADAS: [Control; 32] = {
    let mut a = [plain(g::LAYERS[0], Kind::Layer(0)); 32];
    let mut i = 1;
    while i < a.len() {
        a[i] = plain(g::LAYERS[i], Kind::Layer(i as u8));
        i += 1;
    }
    a
};

/// As linhas geradas por família.
pub(super) fn linhas() -> impl Iterator<Item = &'static Control> {
    M0.iter()
        .chain(&M1)
        .chain(&M2)
        .chain(&M3)
        .chain(&M4)
        .chain(&M5)
        .chain(&P)
        .chain(&CAMADAS)
}

pub(super) const AVANCADO: &[Control] = &[
    opt(
        g::SLICE[0],
        Pick::SliceMode,
        0,
        "panel.widget_gallery.adv.slice.simple",
    ),
    opt(
        g::SLICE[1],
        Pick::SliceMode,
        1,
        "panel.widget_gallery.adv.slice.sliced",
    ),
    opt(
        g::SLICE[2],
        Pick::SliceMode,
        2,
        "panel.widget_gallery.adv.slice.tiled",
    ),
    opt(
        g::SLICE[3],
        Pick::SliceMode,
        3,
        "panel.widget_gallery.adv.slice.tiled_fit",
    ),
    icon(
        g::META_ADD,
        Kind::Box,
        IconId::Add,
        "panel.widget_gallery.adv.tip.meta_add",
    ),
    add_meta(g::META_ADD_FLOAT, 0, "panel.widget_gallery.adv.type.float"),
    add_meta(g::META_ADD_TEXT, 1, "panel.widget_gallery.adv.type.text"),
    add_meta(g::META_ADD_COLOR, 2, "panel.widget_gallery.adv.type.color"),
    icon(
        g::PROP_ADD,
        Kind::Add(PROP_ROWS),
        IconId::Add,
        "panel.widget_gallery.adv.tip.prop_add",
    ),
    // Render.
    plain(g::RN_FORMAT, Kind::Box),
    opt(
        g::RN_FORMATS[0],
        Pick::RenderFormat,
        0,
        "panel.widget_gallery.render.format.png",
    ),
    opt(
        g::RN_FORMATS[1],
        Pick::RenderFormat,
        1,
        "panel.widget_gallery.render.format.mp4",
    ),
    opt(
        g::RN_FORMATS[2],
        Pick::RenderFormat,
        2,
        "panel.widget_gallery.render.format.webp",
    ),
    opt(
        g::RN_FORMATS[3],
        Pick::RenderFormat,
        3,
        "panel.widget_gallery.render.format.gif",
    ),
    opt(
        g::RN_QUALITY[0],
        Pick::RenderQuality,
        0,
        "panel.widget_gallery.render.quality.draft",
    ),
    opt(
        g::RN_QUALITY[1],
        Pick::RenderQuality,
        1,
        "panel.widget_gallery.render.quality.good",
    ),
    opt(
        g::RN_QUALITY[2],
        Pick::RenderQuality,
        2,
        "panel.widget_gallery.render.quality.best",
    ),
    plain(g::RN_RUN, Kind::Act(Act::RenderStart)),
    plain(g::RN_CANCEL, Kind::Act(Act::RenderCancel)),
];
