//! A tabela dos controlos ([`super`]) dos cartões de casos reais (doc `CartoesUI/08` §1.3).

use super::{Act, Control, Kind, act, add_of, eye, icon, opt, plain, seg};
use crate::ids as g;
use crate::state::{Flag, Pick, RACKS, RULES};
use ph2d_editor_core::IconId;

const fn toggle(id: ph2d_a11y::NodeId, f: Flag) -> Control {
    plain(id, Kind::Toggle(f))
}

/// O «−» das linhas sem amostra de cor (regras, rack).
const SEM_AMOSTRA: usize = usize::MAX;

const WHEN: [Pick; 3] = [Pick::RuleWhen0, Pick::RuleWhen1, Pick::RuleWhen2];
const PLAY: [Pick; 3] = [Pick::RulePlay0, Pick::RulePlay1, Pick::RulePlay2];
const RACK_PICKS: [Pick; 3] = [Pick::Rack0, Pick::Rack1, Pick::Rack2];
const RACK_EYES: [Flag; 3] = [Flag::RackEye0, Flag::RackEye1, Flag::RackEye2];
const POSES: [&str; 4] = [
    "panel.inspector.anim_card.when_pose_0",
    "panel.inspector.anim_card.when_pose_1",
    "panel.inspector.anim_card.when_pose_2",
    "panel.inspector.anim_card.when_pose_3",
];
const CLIPS: [&str; 4] = [
    "panel.widget_gallery.real.clip.idle",
    "panel.widget_gallery.real.clip.run",
    "panel.widget_gallery.real.clip.jump",
    "panel.widget_gallery.real.clip.fall",
];
const RACK_KINDS: [&str; 4] = [
    "audio.fx.kind.reverb",
    "audio.fx.kind.echo",
    "audio.fx.kind.chorus",
    "audio.fx.kind.limiter",
];

/// A linha `i` das regras: as duas listas (caixa + 4 opções) e o «−».
const fn regra(i: usize) -> [Control; 11] {
    let r = g::RULES[i];
    [
        plain(r.when, Kind::Box),
        opt(r.whens[0], WHEN[i], 0, POSES[0]),
        opt(r.whens[1], WHEN[i], 1, POSES[1]),
        opt(r.whens[2], WHEN[i], 2, POSES[2]),
        opt(r.whens[3], WHEN[i], 3, POSES[3]),
        plain(r.play, Kind::Box),
        opt(r.plays[0], PLAY[i], 0, CLIPS[0]),
        opt(r.plays[1], PLAY[i], 1, CLIPS[1]),
        opt(r.plays[2], PLAY[i], 2, CLIPS[2]),
        opt(r.plays[3], PLAY[i], 3, CLIPS[3]),
        icon(
            r.remove,
            Kind::Remove(RULES[i], SEM_AMOSTRA),
            IconId::Minus,
            "panel.inspector.anim_card.rule_del_tip",
        ),
    ]
}

/// A linha `i` do rack: abre o lado, olho, «−», a lista do tipo (caixa + 4 opções).
const fn rack(i: usize) -> [Control; 8] {
    let r = g::RACK[i];
    [
        plain(r.open, Kind::Expand(Pick::SideOpen, 3 + i as u8)),
        eye(
            r.eye,
            RACK_EYES[i],
            "panel.widget_gallery.real.tip.rack_visible",
        ),
        icon(
            r.remove,
            Kind::Remove(RACKS[i], SEM_AMOSTRA),
            IconId::Minus,
            "panel.widget_gallery.real.tip.rack_remove",
        ),
        plain(r.kind, Kind::Box),
        opt(r.options[0], RACK_PICKS[i], 0, RACK_KINDS[0]),
        opt(r.options[1], RACK_PICKS[i], 1, RACK_KINDS[1]),
        opt(r.options[2], RACK_PICKS[i], 2, RACK_KINDS[2]),
        opt(r.options[3], RACK_PICKS[i], 3, RACK_KINDS[3]),
    ]
}

/// A paleta: uma cor por linha da tabela.
const fn paleta() -> [Control; 16] {
    let mut out = [plain(g::PALETTE[0], Kind::Palette(0)); 16];
    let mut i = 0;
    while i < 16 {
        out[i] = plain(g::PALETTE[i], Kind::Palette(i as u8));
        i += 1;
    }
    out
}

const R0: [Control; 11] = regra(0);
const R1: [Control; 11] = regra(1);
const R2: [Control; 11] = regra(2);
const K0: [Control; 8] = rack(0);
const K1: [Control; 8] = rack(1);
const K2: [Control; 8] = rack(2);
const PALETA: [Control; 16] = paleta();

/// As linhas geradas por família (regras, rack, paleta).
pub(super) fn linhas() -> impl Iterator<Item = &'static Control> {
    R0.iter()
        .chain(&R1)
        .chain(&R2)
        .chain(&K0)
        .chain(&K1)
        .chain(&K2)
        .chain(&PALETA)
}

pub(super) const REAIS: &[Control] = &[
    // Image.
    plain(g::IMG_ASSET, Kind::Box),
    opt(
        g::IMG_ASSET_0,
        Pick::ImgAsset,
        0,
        "panel.widget_gallery.real.asset.tile",
    ),
    opt(
        g::IMG_ASSET_1,
        Pick::ImgAsset,
        1,
        "panel.widget_gallery.real.asset.sheet",
    ),
    opt(
        g::IMG_ASSET_2,
        Pick::ImgAsset,
        2,
        "panel.widget_gallery.real.asset.sky",
    ),
    opt(
        g::USE_STILL,
        Pick::ImgUse,
        0,
        "panel.inspector.image.use_still",
    ),
    opt(
        g::USE_REPEATED,
        Pick::ImgUse,
        1,
        "panel.inspector.image.use_repeated",
    ),
    opt(
        g::USE_ANIMATED,
        Pick::ImgUse,
        2,
        "panel.inspector.image.use_animated",
    ),
    // Physics Body.
    opt(
        g::RB_DYNAMIC,
        Pick::RbBody,
        0,
        "panel.widget_gallery.real.body.dynamic",
    ),
    opt(
        g::RB_KINEMATIC,
        Pick::RbBody,
        1,
        "panel.widget_gallery.real.body.kinematic",
    ),
    opt(
        g::RB_STATIC,
        Pick::RbBody,
        2,
        "panel.widget_gallery.real.body.static",
    ),
    toggle(g::RB_LOCK, Flag::RbLock),
    // Collider.
    seg(
        g::COL_BOX,
        Pick::ColShape,
        0,
        IconId::MakeSquare,
        "panel.widget_gallery.real.tip.shape_box",
    ),
    seg(
        g::COL_CIRCLE,
        Pick::ColShape,
        1,
        IconId::Circle,
        "panel.widget_gallery.real.tip.shape_circle",
    ),
    seg(
        g::COL_CAPSULE,
        Pick::ColShape,
        2,
        IconId::CardCapRound,
        "panel.widget_gallery.real.tip.shape_capsule",
    ),
    toggle(g::COL_TRIGGER, Flag::ColTrigger),
    // Health.
    toggle(g::HP_OVERHEAL, Flag::HpOverheal),
    // Camera.
    act(
        g::CAM_PICK,
        Act::PickTarget(Flag::CamPicking, g::CAM_TARGET),
        IconId::Eyedropper,
        "panel.widget_gallery.real.tip.pick_target",
    ),
    plain(g::CAM_FORMAT, Kind::Box),
    opt(
        g::CAM_FORMAT_0,
        Pick::CamFormat,
        0,
        "panel.widget_gallery.real.format.wide",
    ),
    opt(
        g::CAM_FORMAT_1,
        Pick::CamFormat,
        1,
        "panel.widget_gallery.real.format.classic",
    ),
    opt(
        g::CAM_FORMAT_2,
        Pick::CamFormat,
        2,
        "panel.widget_gallery.real.format.square",
    ),
    opt(
        g::CAM_FRAME_ALWAYS,
        Pick::CamFrame,
        0,
        "panel.inspector.camera.frame_always",
    ),
    opt(
        g::CAM_FRAME_SELECTED,
        Pick::CamFrame,
        1,
        "panel.inspector.camera.frame_selected",
    ),
    toggle(g::CAM_ACTIVE, Flag::CamActive),
    toggle(g::CAM_LOOK, Flag::CamLook),
    // Animation.
    opt(g::ANIM_ME, Pick::AnimWho, 0, "panel.inspector.anim_card.me"),
    opt(
        g::ANIM_SCRIPT,
        Pick::AnimWho,
        1,
        "panel.inspector.anim_card.script",
    ),
    opt(
        g::ANIM_RULES,
        Pick::AnimWho,
        2,
        "panel.inspector.anim_card.rules",
    ),
    plain(g::ANIM_CLIP, Kind::Box),
    opt(g::ANIM_CLIP_0, Pick::AnimClip, 0, CLIPS[0]),
    opt(g::ANIM_CLIP_1, Pick::AnimClip, 1, CLIPS[1]),
    opt(g::ANIM_CLIP_2, Pick::AnimClip, 2, CLIPS[2]),
    opt(g::ANIM_CLIP_3, Pick::AnimClip, 3, CLIPS[3]),
    icon(
        g::ANIM_PLAY,
        Kind::Toggle(Flag::AnimPlaying),
        IconId::Play,
        "panel.widget_gallery.real.tip.anim_play",
    ),
    icon(
        g::RULE_ADD,
        Kind::Add(RULES),
        IconId::Add,
        "panel.widget_gallery.real.tip.rule_add",
    ),
    // Brush.
    plain(g::BR_PRESET, Kind::Box),
    opt(
        g::BR_PRESET_0,
        Pick::BrushPreset,
        0,
        "panel.widget_gallery.real.preset.liner",
    ),
    opt(
        g::BR_PRESET_1,
        Pick::BrushPreset,
        1,
        "panel.widget_gallery.real.preset.filbert",
    ),
    opt(
        g::BR_PRESET_2,
        Pick::BrushPreset,
        2,
        "panel.widget_gallery.real.preset.fan",
    ),
    opt(
        g::BR_PRESET_3,
        Pick::BrushPreset,
        3,
        "panel.widget_gallery.real.preset.knife",
    ),
    opt(
        g::BR_WATER,
        Pick::PaintType,
        0,
        "panel.painter_layers.wetpaint.paint_type_water",
    ),
    opt(
        g::BR_OIL,
        Pick::PaintType,
        1,
        "panel.painter_layers.wetpaint.paint_type_oil",
    ),
    // Bones.
    seg(
        g::BO_BEND_CCW,
        Pick::BoneBend,
        0,
        IconId::ChevronLeft,
        "card.bones.bend_ccw",
    ),
    seg(
        g::BO_BEND_CW,
        Pick::BoneBend,
        1,
        IconId::ChevronRight,
        "card.bones.bend_cw",
    ),
    toggle(g::BO_LIMIT, Flag::BoneLimit),
    toggle(g::BO_SMART, Flag::BoneSmart),
    toggle(g::BO_IK, Flag::BoneIk),
    // Constraint.
    icon(
        g::CS_ADD,
        Kind::Set(Flag::ConsRow, true),
        IconId::Add,
        "panel.widget_gallery.real.tip.constraint_add",
    ),
    icon(
        g::CS_REMOVE,
        Kind::Set(Flag::ConsRow, false),
        IconId::Minus,
        "panel.inspector.constraint.delete",
    ),
    eye(
        g::CS_EYE,
        Flag::ConsEye,
        "panel.inspector.constraint.enabled",
    ),
    plain(g::CS_KIND, Kind::Box),
    opt(
        g::CS_KIND_0,
        Pick::ConsKind,
        0,
        "panel.inspector.constraint.look_at",
    ),
    opt(
        g::CS_KIND_1,
        Pick::ConsKind,
        1,
        "panel.inspector.constraint.follow",
    ),
    opt(
        g::CS_KIND_2,
        Pick::ConsKind,
        2,
        "panel.inspector.constraint.follow_path",
    ),
    act(
        g::CS_PICK,
        Act::PickTarget(Flag::ConsPicking, g::CS_TARGET),
        IconId::Eyedropper,
        "panel.widget_gallery.real.tip.pick_target",
    ),
    plain(g::CS_SPACE, Kind::Box),
    opt(
        g::CS_SPACE_WORLD,
        Pick::ConsSpace,
        0,
        "panel.inspector.constraint.space.world",
    ),
    opt(
        g::CS_SPACE_LOCAL,
        Pick::ConsSpace,
        1,
        "panel.inspector.constraint.space.local",
    ),
    // Light 2D.
    opt(g::LT_POINT, Pick::LampKind, 0, "panel.inspector.luz.point"),
    opt(
        g::LT_AMBIENT,
        Pick::LampKind,
        1,
        "panel.inspector.luz.ambient",
    ),
    plain(g::LT_SWATCH, Kind::Swatch(9)),
    toggle(g::LT_SHADOWS, Flag::LampShadows),
    // Audio.
    icon(
        g::AU_MUTE,
        Kind::Toggle(Flag::AudioMute),
        IconId::Volume2,
        "panel.widget_gallery.real.tip.mute",
    ),
    icon(
        g::AU_ADD,
        Kind::Box,
        IconId::Add,
        "panel.widget_gallery.real.tip.rack_add",
    ),
    add_of(g::AU_ADD_0, RACKS, RACK_PICKS, 0, RACK_KINDS[0]),
    add_of(g::AU_ADD_1, RACKS, RACK_PICKS, 1, RACK_KINDS[1]),
    add_of(g::AU_ADD_2, RACKS, RACK_PICKS, 2, RACK_KINDS[2]),
    add_of(g::AU_ADD_3, RACKS, RACK_PICKS, 3, RACK_KINDS[3]),
    // Pixel Art.
    plain(g::PX_FG, Kind::Pick(Pick::PixelTarget, 0)),
    plain(g::PX_BG, Kind::Pick(Pick::PixelTarget, 1)),
    act(
        g::PX_SWAP,
        Act::PixelSwap,
        IconId::ConvertCurves,
        "panel.pixel.swap_tip",
    ),
    toggle(g::PX_GRID_ON, Flag::PixelGrid),
    // Script.
    plain(g::SC_FILE, Kind::Box),
    opt(
        g::SC_FILE_0,
        Pick::ScriptFile,
        0,
        "panel.widget_gallery.real.script.player",
    ),
    opt(
        g::SC_FILE_1,
        Pick::ScriptFile,
        1,
        "panel.widget_gallery.real.script.enemy",
    ),
    opt(
        g::SC_FILE_2,
        Pick::ScriptFile,
        2,
        "panel.widget_gallery.real.script.coin",
    ),
    toggle(g::SC_EDIT, Flag::ScriptEdit),
    toggle(g::SC_DOUBLE, Flag::ScriptDouble),
];
