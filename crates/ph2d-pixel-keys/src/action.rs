//! As acções do Pixel que uma tecla pode disparar (uma por comando), com id estável e secção.

use ph2d_editor_core::pixel_panel::{
    PixelBrushAction, PixelEditAction as E, PixelFileAction as Fi, PixelFrameAction as Fr,
    PixelLayerAction as L, PixelPatternTurn, PixelRequest as R, PixelSelectAction as S,
    PixelSpriteAction as Sp, PixelTool as T,
};

/// A secção do diálogo de atalhos (a ordem do Aseprite, O1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    Menus,
    Commands,
    Tools,
    Modifiers,
}

impl Section {
    #[must_use]
    pub const fn name_key(self) -> &'static str {
        match self {
            Self::Menus => "pixel.keys.section.menus",
            Self::Commands => "pixel.keys.section.commands",
            Self::Tools => "pixel.keys.section.tools",
            Self::Modifiers => "pixel.keys.section.modifiers",
        }
    }
}

macro_rules! actions {
    ($($v:ident = $id:literal, $sec:ident;)*) => {
        /// Um comando do Pixel que uma tecla dispara.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum PixelAction { $($v,)* }

        impl PixelAction {
            /// Todas, na ordem do diálogo.
            pub const ALL: &'static [Self] = &[$(Self::$v,)*];

            /// O id do ficheiro do utilizador (estável: não mudar).
            #[must_use]
            pub const fn id(self) -> &'static str {
                match self { $(Self::$v => $id,)* }
            }

            /// A chave i18n do nome (inglês).
            #[must_use]
            pub const fn name_key(self) -> &'static str {
                match self { $(Self::$v => concat!("pixel.keys.", $id),)* }
            }

            #[must_use]
            pub const fn section(self) -> Section {
                match self { $(Self::$v => Section::$sec,)* }
            }
        }
    };
}

actions! {
    // Menus: File.
    ExportAs = "file.export_as", Menus;
    ExportSheet = "file.export_sheet", Menus;
    ImportSheet = "file.import_sheet", Menus;
    RepeatExport = "file.repeat_export", Menus;
    // Edit.
    Undo = "edit.undo", Menus;
    Redo = "edit.redo", Menus;
    Cut = "edit.cut", Menus;
    Copy = "edit.copy", Menus;
    CopyMerged = "edit.copy_merged", Menus;
    Paste = "edit.paste", Menus;
    Clear = "edit.clear", Menus;
    Fill = "edit.fill", Menus;
    Stroke = "edit.stroke", Menus;
    FlipH = "edit.flip_h", Menus;
    FlipV = "edit.flip_v", Menus;
    Transform = "edit.transform", Menus;
    NewBrush = "edit.new_brush", Menus;
    ReplaceColor = "edit.replace_color", Menus;
    Outline = "edit.outline", Menus;
    ColorCurve = "edit.color_curve", Menus;
    HueSaturation = "edit.hue_saturation", Menus;
    ConvolutionMatrix = "edit.convolution_matrix", Menus;
    KeyboardShortcuts = "edit.keyboard_shortcuts", Menus;
    // Select.
    SelectAll = "select.all", Menus;
    Deselect = "select.deselect", Menus;
    Reselect = "select.reselect", Menus;
    InvertSelection = "select.invert", Menus;
    // Sprite.
    CanvasSize = "sprite.canvas_size", Menus;
    SpriteSize = "sprite.sprite_size", Menus;
    // Layer.
    NewLayer = "layer.new", Menus;
    NewGroup = "layer.new_group", Menus;
    NewTilemapLayer = "layer.new_tilemap", Menus;
    LayerViaCopy = "layer.via_copy", Menus;
    LayerViaCut = "layer.via_cut", Menus;
    LayerFromClipboard = "layer.from_clipboard", Menus;
    LayerVisibility = "layer.visibility", Menus;
    LayerRename = "layer.rename", Menus;
    LayerOpacity25 = "layer.opacity_25", Menus;
    LayerOpacity50 = "layer.opacity_50", Menus;
    LayerOpacity75 = "layer.opacity_75", Menus;
    LayerOpacity100 = "layer.opacity_100", Menus;
    // Frame.
    NewFrame = "frame.new", Menus;
    NewEmptyFrame = "frame.new_empty", Menus;
    DuplicateLinked = "frame.duplicate_linked", Menus;
    DeleteFrame = "frame.delete", Menus;
    ReverseFrames = "frame.reverse", Menus;
    FirstFrame = "frame.first", Menus;
    PreviousFrame = "frame.previous", Menus;
    NextFrame = "frame.next", Menus;
    LastFrame = "frame.last", Menus;
    Play = "frame.play", Menus;
    PlayPreview = "frame.play_preview", Menus;
    Preview = "frame.preview", Menus;
    OnionSkin = "frame.onion_skin", Menus;
    LoopSection = "frame.loop_section", Menus;
    // Commands sem menu.
    SwapColors = "color.swap", Commands;
    ShowGrid = "view.grid", Commands;
    PixelGrid = "view.pixel_grid", Commands;
    GridSettings = "view.grid_settings", Commands;
    Timeline = "view.timeline", Commands;
    BrushSlot1 = "brush.slot_1", Commands;
    BrushSlot2 = "brush.slot_2", Commands;
    BrushSlot3 = "brush.slot_3", Commands;
    BrushSlot4 = "brush.slot_4", Commands;
    BrushSlot5 = "brush.slot_5", Commands;
    BrushSlot6 = "brush.slot_6", Commands;
    BrushSlot7 = "brush.slot_7", Commands;
    BrushSlot8 = "brush.slot_8", Commands;
    BrushSlot9 = "brush.slot_9", Commands;
    BrushFlipH = "brush.flip_h", Commands;
    BrushFlipV = "brush.flip_v", Commands;
    BrushRotate = "brush.rotate", Commands;
    TileModeToggle = "tiles.toggle", Commands;
    TileManual = "tiles.manual", Commands;
    TileAuto = "tiles.auto", Commands;
    TileStack = "tiles.stack", Commands;
    CommitTransform = "transform.commit", Commands;
    CancelTransform = "transform.cancel", Commands;
    DropFloating = "transform.drop", Commands;
    ClearSelection = "select.escape", Commands;
    FinishStroke = "stroke.finish", Commands;
    CancelStroke = "stroke.cancel", Commands;
    DeleteSlices = "slices.delete", Commands;
    DeselectSlices = "slices.deselect", Commands;
    Zoom50 = "view.zoom_50", Commands;
    Zoom100 = "view.zoom_100", Commands;
    Zoom200 = "view.zoom_200", Commands;
    Zoom400 = "view.zoom_400", Commands;
    Zoom800 = "view.zoom_800", Commands;
    Zoom1600 = "view.zoom_1600", Commands;
    Zoom3200 = "view.zoom_3200", Commands;
    ZoomIn = "view.zoom_in", Commands;
    ZoomOut = "view.zoom_out", Commands;
    BrushSmaller = "brush.smaller", Commands;
    BrushBigger = "brush.bigger", Commands;
    ColorPrevious = "color.fg_previous", Commands;
    ColorNext = "color.fg_next", Commands;
    LayerAbove = "layer.above", Commands;
    LayerBelow = "layer.below", Commands;
    FramePreviousInTag = "frame.previous_in_tag", Commands;
    FrameNextInTag = "frame.next_in_tag", Commands;
    // Tools.
    ToolRectMarquee = "tool.rect_marquee", Tools;
    ToolEllipseMarquee = "tool.ellipse_marquee", Tools;
    ToolLasso = "tool.lasso", Tools;
    ToolPolygonLasso = "tool.polygon_lasso", Tools;
    ToolMagicWand = "tool.magic_wand", Tools;
    ToolTransform = "tool.transform", Tools;
    ToolSlice = "tool.slice", Tools;
    ToolPencil = "tool.pencil", Tools;
    ToolSpray = "tool.spray", Tools;
    ToolEraser = "tool.eraser", Tools;
    ToolEyedropper = "tool.eyedropper", Tools;
    ToolBucket = "tool.bucket", Tools;
    ToolGradient = "tool.gradient", Tools;
    ToolBlurJumble = "tool.blur_jumble", Tools;
    ToolLine = "tool.line", Tools;
    ToolCurve = "tool.curve", Tools;
    ToolRectangle = "tool.rectangle", Tools;
    ToolEllipse = "tool.ellipse", Tools;
    ToolContour = "tool.contour", Tools;
    ToolPolygon = "tool.polygon", Tools;
    ToolHand = "tool.hand", Tools;
    ToolText = "tool.text", Tools;
}

/// O zoom que uma tecla pede (o app prende-o à escada inteira do Draw).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomKey {
    /// Em por cento (100 = uma célula por píxel de ecrã).
    Percent(u32),
    In,
    Out,
}

impl PixelAction {
    /// O zoom da acção, se é de zoom.
    #[must_use]
    pub const fn zoom(self) -> Option<ZoomKey> {
        Some(match self {
            Self::Zoom50 => ZoomKey::Percent(50),
            Self::Zoom100 => ZoomKey::Percent(100),
            Self::Zoom200 => ZoomKey::Percent(200),
            Self::Zoom400 => ZoomKey::Percent(400),
            Self::Zoom800 => ZoomKey::Percent(800),
            Self::Zoom1600 => ZoomKey::Percent(1600),
            Self::Zoom3200 => ZoomKey::Percent(3200),
            Self::ZoomIn => ZoomKey::In,
            Self::ZoomOut => ZoomKey::Out,
            _ => return None,
        })
    }

    /// A acção de um id do ficheiro.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.id() == id)
    }

    /// O nome em inglês (o do diálogo e da paleta).
    #[must_use]
    pub fn name(self) -> &'static str {
        ph2d_i18n::tr(self.name_key())
    }

    /// A ferramenta que a acção põe na mão (`ToolBlurJumble` alterna: o app decide).
    #[must_use]
    pub const fn tool(self) -> Option<T> {
        Some(match self {
            Self::ToolRectMarquee => T::RectMarquee,
            Self::ToolEllipseMarquee => T::EllipseMarquee,
            Self::ToolLasso => T::Lasso,
            Self::ToolPolygonLasso => T::PolygonLasso,
            Self::ToolMagicWand => T::MagicWand,
            Self::ToolTransform => T::Transform,
            Self::ToolSlice => T::Slice,
            Self::ToolPencil => T::Pencil,
            Self::ToolSpray => T::Spray,
            Self::ToolEraser => T::Eraser,
            Self::ToolEyedropper => T::Eyedropper,
            Self::ToolBucket => T::Bucket,
            Self::ToolGradient => T::Gradient,
            Self::ToolLine => T::Line,
            Self::ToolCurve => T::Curve,
            Self::ToolRectangle => T::RECTANGLE,
            Self::ToolEllipse => T::ELLIPSE,
            Self::ToolContour => T::Contour,
            Self::ToolPolygon => T::Polygon,
            Self::ToolHand => T::Hand,
            Self::ToolText => T::Text,
            _ => return None,
        })
    }

    /// O pedido do painel que a acção é, quando é só isso (o resto o app faz com o estado dele).
    #[must_use]
    pub fn request(self) -> Option<R> {
        if let Some(t) = self.tool() {
            return Some(R::Tool(t));
        }
        let slot = |n: usize| R::Brush(PixelBrushAction::Recall(n));
        Some(match self {
            Self::ExportAs => R::File(Fi::ExportAs),
            Self::ExportSheet => R::File(Fi::ExportSheet),
            Self::ImportSheet => R::File(Fi::ImportSheet),
            Self::RepeatExport => R::File(Fi::RepeatExport),
            Self::Cut => R::Edit(E::Cut),
            Self::Copy => R::Edit(E::Copy),
            Self::CopyMerged => R::Edit(E::CopyMerged),
            Self::Paste => R::Edit(E::Paste),
            Self::Clear => R::Edit(E::Clear),
            Self::Fill => R::Edit(E::Fill),
            Self::Stroke => R::Edit(E::Stroke),
            Self::FlipH => R::Edit(E::FlipH),
            Self::FlipV => R::Edit(E::FlipV),
            Self::Transform => R::Edit(E::Transform),
            Self::NewBrush => R::Edit(E::NewBrush),
            Self::ReplaceColor => R::ReplaceColor,
            Self::Outline => R::Edit(E::Outline),
            Self::ColorCurve => R::Edit(E::ColorCurve),
            Self::HueSaturation => R::Edit(E::HueSaturation),
            Self::ConvolutionMatrix => R::Edit(E::ConvolutionMatrix),
            Self::SelectAll => R::Select(S::All),
            Self::Deselect => R::Select(S::Deselect),
            Self::Reselect => R::Select(S::Reselect),
            Self::InvertSelection => R::Select(S::Inverse),
            Self::CanvasSize => R::Sprite(Sp::CanvasSize),
            Self::SpriteSize => R::Sprite(Sp::SpriteSize),
            Self::NewLayer => R::LayerAction(L::NewLayer),
            Self::NewGroup => R::LayerAction(L::NewGroup),
            Self::NewTilemapLayer => R::LayerAction(L::NewTilemap),
            Self::LayerViaCopy => R::LayerAction(L::ViaCopy),
            Self::LayerViaCut => R::LayerAction(L::ViaCut),
            Self::LayerFromClipboard => R::LayerAction(L::PasteAsLayer),
            Self::LayerOpacity25 => R::LayerOpacity(64),
            Self::LayerOpacity50 => R::LayerOpacity(128),
            Self::LayerOpacity75 => R::LayerOpacity(192),
            Self::LayerOpacity100 => R::LayerOpacity(255),
            Self::NewFrame => R::FrameAction(Fr::NewFrame),
            Self::NewEmptyFrame => R::FrameAction(Fr::NewEmptyFrame),
            Self::DuplicateLinked => R::FrameAction(Fr::DuplicateLinked),
            Self::DeleteFrame => R::FrameAction(Fr::Delete),
            Self::ReverseFrames => R::FrameAction(Fr::Reverse),
            Self::FirstFrame => R::FrameAction(Fr::First),
            Self::PreviousFrame => R::FrameAction(Fr::Previous),
            Self::NextFrame => R::FrameAction(Fr::Next),
            Self::LastFrame => R::FrameAction(Fr::Last),
            Self::Play => R::FrameAction(Fr::Play),
            Self::PlayPreview => R::FrameAction(Fr::PlayPreview),
            Self::Preview => R::FrameAction(Fr::Preview),
            Self::OnionSkin => R::FrameAction(Fr::Onion),
            Self::LoopSection => R::FrameAction(Fr::LoopSection),
            Self::SwapColors => R::Swap,
            Self::BrushSlot1 => slot(0),
            Self::BrushSlot2 => slot(1),
            Self::BrushSlot3 => slot(2),
            Self::BrushSlot4 => slot(3),
            Self::BrushSlot5 => slot(4),
            Self::BrushSlot6 => slot(5),
            Self::BrushSlot7 => slot(6),
            Self::BrushSlot8 => slot(7),
            Self::BrushSlot9 => slot(8),
            Self::BrushFlipH => R::Brush(PixelBrushAction::Turn(PixelPatternTurn::FlipH)),
            Self::BrushFlipV => R::Brush(PixelBrushAction::Turn(PixelPatternTurn::FlipV)),
            Self::BrushRotate => R::Brush(PixelBrushAction::Turn(PixelPatternTurn::RotateRight)),
            Self::CommitTransform => R::Edit(E::Commit),
            Self::CancelTransform => R::Edit(E::Cancel),
            _ => return None,
        })
    }

    /// Vale com um gesto a meio? (hoje: a selecção, o Edit/Sprite, o Alt e as slices esperam).
    #[must_use]
    pub const fn while_stroking(self) -> bool {
        !matches!(
            self,
            Self::ToolRectMarquee
                | Self::ToolEllipseMarquee
                | Self::ToolLasso
                | Self::ToolPolygonLasso
                | Self::ToolMagicWand
                | Self::ToolTransform
                | Self::SelectAll
                | Self::Deselect
                | Self::Reselect
                | Self::InvertSelection
                | Self::NewBrush
                | Self::Transform
                | Self::FlipH
                | Self::FlipV
                | Self::Clear
                | Self::Copy
                | Self::Cut
                | Self::CopyMerged
                | Self::Paste
                | Self::CanvasSize
                | Self::SpriteSize
                | Self::Fill
                | Self::Stroke
                | Self::Outline
                | Self::HueSaturation
                | Self::ColorCurve
                | Self::ConvolutionMatrix
                | Self::ReplaceColor
                | Self::ExportAs
                | Self::NewGroup
                | Self::NewFrame
                | Self::NewEmptyFrame
                | Self::DuplicateLinked
                | Self::DeleteFrame
                | Self::ReverseFrames
                | Self::BrushSlot1
                | Self::BrushSlot2
                | Self::BrushSlot3
                | Self::BrushSlot4
                | Self::BrushSlot5
                | Self::BrushSlot6
                | Self::BrushSlot7
                | Self::BrushSlot8
                | Self::BrushSlot9
                | Self::DeleteSlices
                | Self::DeselectSlices
                | Self::BrushSmaller
                | Self::BrushBigger
                | Self::ColorPrevious
                | Self::ColorNext
                | Self::LayerAbove
                | Self::LayerBelow
                | Self::FramePreviousInTag
                | Self::FrameNextInTag
        )
    }
}
