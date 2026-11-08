//! A tabela de fábrica (a do Aseprite com as excepções do dono, `P12_plano.md` §4.2 e §9), as
//! teclas do app que o Pixel nunca toma, e a tabela medida do Aseprite contra a qual se prova.

use winit::keyboard::KeyCode as K;

use crate::{Chord, PixelAction as A};

/// Onde a tecla vale. A mesma tecla em contextos diferentes não é conflito (Aseprite O7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Context {
    /// Sem nada levantado nem gesto a meio.
    Normal,
    /// Há selecção (máscara, intervalo de quadros, ou o modo de definir o pincel do `Ctrl+B`).
    Selection,
    /// Células levantadas na moldura solta (`solta`).
    Transformation,
    /// Um gesto a meio (traço, forma, laço poligonal aberto).
    Stroking,
    /// A Slice Tool na mão com slices seleccionadas.
    Slices,
}

impl Context {
    pub const ALL: [Self; 5] = [
        Self::Normal,
        Self::Selection,
        Self::Transformation,
        Self::Stroking,
        Self::Slices,
    ];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Selection => "Selection",
            Self::Transformation => "Transformation",
            Self::Stroking => "Stroking",
            Self::Slices => "Slices",
        }
    }

    #[must_use]
    pub fn from_id(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|c| c.id().eq_ignore_ascii_case(s))
    }

    /// O contexto do estado, por prioridade: gesto a meio, moldura solta, slices, selecção.
    #[must_use]
    pub const fn of(stroking: bool, floating: bool, slices: bool, selection: bool) -> Self {
        match (stroking, floating, slices, selection) {
            (true, ..) => Self::Stroking,
            (_, true, ..) => Self::Transformation,
            (_, _, true, _) => Self::Slices,
            (.., true) => Self::Selection,
            _ => Self::Normal,
        }
    }

    #[must_use]
    pub const fn name_key(self) -> &'static str {
        match self {
            Self::Normal => "pixel.keys.context.normal",
            Self::Selection => "pixel.keys.context.selection",
            Self::Transformation => "pixel.keys.context.transformation",
            Self::Stroking => "pixel.keys.context.stroking",
            Self::Slices => "pixel.keys.context.slices",
        }
    }
}

const fn k(key: K) -> Chord {
    Chord::new(key)
}

use Context::{Normal as N, Selection as Sel, Slices as Sl, Stroking as St, Transformation as Tr};

/// A tabela de fábrica: `(acção, tecla, contexto)`.
pub const FACTORY: &[(A, Chord, Context)] = &[
    (A::ExportAs, k(K::KeyS).ctrl().alt().shift(), N),
    (A::ExportSheet, k(K::KeyE).ctrl(), N),
    (A::ImportSheet, k(K::KeyI).ctrl(), N),
    (A::RepeatExport, k(K::KeyX).ctrl().shift(), N),
    (A::Undo, k(K::KeyZ).ctrl(), N),
    (A::Redo, k(K::KeyY).ctrl(), N),
    (A::Redo, k(K::KeyZ).ctrl().shift(), N),
    (A::Redo, k(K::KeyR).ctrl(), N),
    (A::Cut, k(K::KeyX).ctrl(), N),
    (A::Cut, k(K::Delete).shift(), N),
    (A::Copy, k(K::KeyC).ctrl(), N),
    (A::Copy, k(K::Insert).ctrl(), N),
    (A::CopyMerged, k(K::KeyC).ctrl().shift(), N),
    (A::Paste, k(K::KeyV).ctrl(), N),
    (A::Paste, k(K::Insert).shift(), N),
    (A::Clear, k(K::Delete), N),
    (A::Clear, k(K::Backspace), N),
    (A::Fill, k(K::KeyF), N),
    (A::Stroke, k(K::KeyS), N),
    (A::FlipH, k(K::KeyH).shift(), N),
    (A::FlipV, k(K::KeyV).shift(), N),
    (A::Transform, k(K::KeyT).ctrl(), N),
    (A::NewBrush, k(K::KeyB).ctrl(), N),
    (A::ReplaceColor, k(K::KeyR).shift(), N),
    (A::Outline, k(K::KeyO).shift(), N),
    (A::ColorCurve, k(K::KeyM).ctrl(), N),
    (A::ColorCurve, k(K::F10), N),
    (A::HueSaturation, k(K::KeyU).ctrl(), N),
    (A::KeyboardShortcuts, k(K::KeyK).ctrl().alt().shift(), N),
    (A::SelectAll, k(K::KeyA).ctrl(), N),
    (A::Deselect, k(K::KeyD).ctrl(), N),
    (A::Reselect, k(K::KeyD).ctrl().shift(), N),
    (A::InvertSelection, k(K::KeyI).ctrl().shift(), N),
    (A::CanvasSize, k(K::KeyC), N),
    (A::SpriteSize, k(K::KeyI).ctrl().alt(), N),
    (A::NewLayer, k(K::KeyN).shift(), N),
    (A::NewGroup, k(K::KeyN).alt().shift(), N),
    (A::NewTilemapLayer, k(K::KeyN).space(), N),
    (A::LayerViaCopy, k(K::KeyJ).ctrl(), N),
    (A::LayerViaCut, k(K::KeyJ).ctrl().shift(), N),
    (A::LayerFromClipboard, k(K::KeyV).ctrl().shift(), N),
    (A::LayerVisibility, k(K::KeyX).shift(), N),
    (A::LayerRename, k(K::F2), N),
    (A::LayerOpacity25, k(K::Digit1).shift(), N),
    (A::LayerOpacity50, k(K::Digit2).shift(), N),
    (A::LayerOpacity75, k(K::Digit3).shift(), N),
    (A::LayerOpacity100, k(K::Digit4).shift(), N),
    (A::NewFrame, k(K::KeyN).alt(), N),
    (A::NewEmptyFrame, k(K::KeyB).alt(), N),
    (A::DuplicateLinked, k(K::KeyM).alt(), N),
    (A::DuplicateLinked, k(K::KeyD).alt().shift(), N),
    (A::DeleteFrame, k(K::KeyC).alt(), N),
    (A::ReverseFrames, k(K::KeyI).alt(), N),
    (A::FirstFrame, k(K::Home), N),
    (A::PreviousFrame, k(K::ArrowLeft), N),
    (A::NextFrame, k(K::ArrowRight), N),
    (A::LastFrame, k(K::End), N),
    (A::Play, k(K::Enter), N),
    (A::Play, k(K::NumpadEnter), N),
    (A::PlayPreview, k(K::Enter).shift(), N),
    (A::PlayPreview, k(K::NumpadEnter).shift(), N),
    (A::Preview, k(K::F7), N),
    (A::OnionSkin, k(K::F3), N),
    (A::SwapColors, k(K::KeyX), N),
    (A::ShowGrid, k(K::Quote).ctrl(), N),
    (A::PixelGrid, k(K::Quote).ctrl().shift(), N),
    (A::Timeline, k(K::F4), N),
    (A::BrushSlot1, k(K::Digit1).alt(), N),
    (A::BrushSlot2, k(K::Digit2).alt(), N),
    (A::BrushSlot3, k(K::Digit3).alt(), N),
    (A::BrushSlot4, k(K::Digit4).alt(), N),
    (A::BrushSlot5, k(K::Digit5).alt(), N),
    (A::BrushSlot6, k(K::Digit6).alt(), N),
    (A::BrushSlot7, k(K::Digit7).alt(), N),
    (A::BrushSlot8, k(K::Digit8).alt(), N),
    (A::BrushSlot9, k(K::Digit9).alt(), N),
    (A::BrushFlipH, k(K::KeyH).space(), N),
    (A::BrushFlipH, k(K::KeyX).space(), N),
    (A::BrushFlipV, k(K::KeyV).space(), N),
    (A::BrushFlipV, k(K::KeyY).space(), N),
    (A::BrushRotate, k(K::KeyR).space(), N),
    (A::TileModeToggle, k(K::Tab).space(), N),
    (A::TileManual, k(K::Digit1).space(), N),
    (A::TileAuto, k(K::Digit2).space(), N),
    (A::TileStack, k(K::Digit3).space(), N),
    (A::CommitTransform, k(K::Enter), Tr),
    (A::CommitTransform, k(K::NumpadEnter), Tr),
    (A::CommitTransform, k(K::Enter).shift(), Tr),
    (A::CancelTransform, k(K::Escape), Tr),
    (A::DropFloating, k(K::KeyZ).ctrl(), Tr),
    (A::DropFloating, k(K::KeyZ).ctrl().shift(), Tr),
    (A::ClearSelection, k(K::Escape), Sel),
    (A::FinishStroke, k(K::Enter), St),
    (A::FinishStroke, k(K::NumpadEnter), St),
    (A::FinishStroke, k(K::Enter).shift(), St),
    (A::CancelStroke, k(K::Escape), St),
    (A::DeleteSlices, k(K::Delete), Sl),
    (A::DeleteSlices, k(K::Backspace), Sl),
    (A::DeselectSlices, k(K::Escape), Sl),
    (A::Zoom100, k(K::Digit1), N),
    (A::Zoom200, k(K::Digit2), N),
    (A::Zoom400, k(K::Digit3), N),
    (A::Zoom800, k(K::Digit4), N),
    (A::Zoom1600, k(K::Digit5), N),
    (A::Zoom3200, k(K::Digit6), N),
    (A::Zoom50, k(K::Backquote), N),
    (A::ZoomIn, k(K::Equal).ctrl(), N),
    (A::ZoomIn, k(K::Equal).ctrl().shift(), N),
    (A::ZoomOut, k(K::Minus).ctrl(), N),
    (A::BrushSmaller, k(K::Minus), N),
    (A::BrushSmaller, k(K::NumpadSubtract), N),
    (A::BrushBigger, k(K::Equal).shift(), N),
    (A::BrushBigger, k(K::NumpadAdd), N),
    (A::ColorPrevious, k(K::BracketLeft), N),
    (A::ColorPrevious, k(K::Digit9), N),
    (A::ColorNext, k(K::BracketRight), N),
    (A::ColorNext, k(K::Digit0), N),
    (A::LayerAbove, k(K::ArrowUp), N),
    (A::LayerBelow, k(K::ArrowDown), N),
    (A::FramePreviousInTag, k(K::Comma), N),
    (A::FrameNextInTag, k(K::Period), N),
    (A::ToolRectMarquee, k(K::KeyM), N),
    (A::ToolEllipseMarquee, k(K::KeyM).shift(), N),
    (A::ToolLasso, k(K::KeyQ), N),
    (A::ToolPolygonLasso, k(K::KeyQ).shift(), N),
    (A::ToolMagicWand, k(K::KeyW), N),
    (A::ToolTransform, k(K::KeyV), N),
    (A::ToolSlice, k(K::KeyC).shift(), N),
    (A::ToolPencil, k(K::KeyB), N),
    (A::ToolSpray, k(K::KeyB).shift(), N),
    (A::ToolEraser, k(K::KeyE), N),
    (A::ToolEyedropper, k(K::KeyI), N),
    (A::ToolBucket, k(K::KeyP), N),
    (A::ToolGradient, k(K::KeyP).shift(), N),
    (A::ToolBlurJumble, k(K::KeyR), N),
    (A::ToolLine, k(K::KeyL), N),
    (A::ToolCurve, k(K::KeyL).shift(), N),
    (A::ToolRectangle, k(K::KeyU), N),
    (A::ToolEllipse, k(K::KeyU).shift(), N),
    (A::ToolContour, k(K::KeyD), N),
    (A::ToolPolygon, k(K::KeyD).shift(), N),
    (A::ToolHand, k(K::KeyH), N),
    (A::ToolText, k(K::KeyT), N),
];

/// Teclas GLOBAIS do app: o Pixel nunca as toma (dono 08/11, §9.7). Censo da shell:
/// - `Ctrl+K` paleta: `shells/desktop/src/input_handlers.rs:137`
/// - `Ctrl+Space` zen: `input_handlers.rs:130` (hoje o Pixel engole-o em `input::space`: bug)
/// - `Tab`/`Shift+Tab`/`Ctrl+Tab` modo: `input_handlers.rs:124` (`mode_key`, sem olhar o Shift)
/// - `Ctrl+S`/`Ctrl+Shift+S`: `input_dispatch/keyboard_files.rs:47`; `Ctrl+O`: `:57`
/// - `Ctrl+N`: `input_dispatch/handlers_teclas_editor.rs:233`
/// - `G`/`Shift+G` grade: `handlers_teclas_editor.rs:69` (`!cmd_chord`, sem olhar o Shift)
/// - `F5`/`Shift+F5` jogo: `input_dispatch/posse_do_jogo.rs:46` (antes de todo atalho)
/// - `F9` chrome legado: `input_dispatch/keyboard.rs:115` (antes do Pixel: o F9 nunca lá chega)
/// - `Ctrl+P` prender ao esqueleto: `handlers_teclas_editor.rs:188`
/// - `Ctrl+0` enquadrar: `handlers_teclas_editor.rs:40`
/// - `Ctrl+Enter` assentar o traço: `handlers_teclas_editor.rs:226`
/// - `Shift+A` menu Add: `handlers_teclas_editor.rs:240`
pub const RESERVED_GLOBAL: &[(Chord, &str)] = &[
    (k(K::KeyK).ctrl(), "pixel.keys.app.palette"),
    (k(K::Space).ctrl(), "pixel.keys.app.zen"),
    (k(K::Tab), "pixel.keys.app.mode"),
    (k(K::Tab).shift(), "pixel.keys.app.mode"),
    (k(K::Tab).ctrl(), "pixel.keys.app.mode_list"),
    (k(K::KeyS).ctrl(), "pixel.keys.app.save"),
    (k(K::KeyS).ctrl().shift(), "pixel.keys.app.save_as"),
    (k(K::KeyO).ctrl(), "pixel.keys.app.open"),
    (k(K::KeyN).ctrl(), "pixel.keys.app.new"),
    (k(K::KeyG), "pixel.keys.app.grid"),
    (k(K::KeyG).shift(), "pixel.keys.app.grid"),
    (k(K::F5), "pixel.keys.app.play"),
    (k(K::F5).shift(), "pixel.keys.app.play"),
    (k(K::F9), "pixel.keys.app.legacy_chrome"),
    (k(K::KeyP).ctrl(), "pixel.keys.app.bind"),
    (k(K::Digit0).ctrl(), "pixel.keys.app.fit"),
    (k(K::Enter).ctrl(), "pixel.keys.app.commit"),
    (k(K::KeyA).shift(), "pixel.keys.app.add"),
];

/// Teclas do app que o Pixel só pode ter, em `Normal`, com o MESMO sentido (encaminhado ao sprite).
pub const SAME_MEANING: &[(Chord, A, &str)] = &[
    (k(K::F2), A::LayerRename, "pixel.keys.app.rename"),
    (k(K::KeyZ).ctrl(), A::Undo, "pixel.keys.app.undo"),
    (k(K::KeyY).ctrl(), A::Redo, "pixel.keys.app.redo"),
    (k(K::KeyZ).ctrl().shift(), A::Redo, "pixel.keys.app.redo"),
    (k(K::KeyC).ctrl(), A::Copy, "pixel.keys.app.copy"),
    (k(K::KeyX).ctrl(), A::Cut, "pixel.keys.app.cut"),
    (k(K::KeyV).ctrl(), A::Paste, "pixel.keys.app.paste"),
    (k(K::KeyA).ctrl(), A::SelectAll, "pixel.keys.app.select_all"),
    (k(K::Delete), A::Clear, "pixel.keys.app.delete"),
];

/// O nome (chave i18n) do dono global de uma tecla, se ela é do app.
#[must_use]
pub fn reserved(chord: Chord) -> Option<&'static str> {
    RESERVED_GLOBAL
        .iter()
        .find(|(c, _)| *c == chord)
        .map(|(_, n)| *n)
}

/// A tabela do Aseprite medida (`P12_oraculos.md`, tabela de fábrica + Rodada 2), só as linhas
/// de comandos que o Pixel tem.
pub const ASEPRITE: &[(A, Chord)] = &[
    (A::ExportAs, k(K::KeyS).ctrl().alt().shift()),
    (A::ExportSheet, k(K::KeyE).ctrl()),
    (A::ImportSheet, k(K::KeyI).ctrl()),
    (A::RepeatExport, k(K::KeyX).ctrl().shift()),
    (A::Undo, k(K::KeyZ).ctrl()),
    (A::Redo, k(K::KeyY).ctrl()),
    (A::Redo, k(K::KeyZ).ctrl().shift()),
    (A::Redo, k(K::KeyR).ctrl()),
    (A::Cut, k(K::KeyX).ctrl()),
    (A::Cut, k(K::Delete).shift()),
    (A::Copy, k(K::KeyC).ctrl()),
    (A::Copy, k(K::Insert).ctrl()),
    (A::CopyMerged, k(K::KeyC).ctrl().shift()),
    (A::Paste, k(K::KeyV).ctrl()),
    (A::Paste, k(K::Insert).shift()),
    (A::Clear, k(K::Delete)),
    (A::Clear, k(K::Backspace)),
    (A::Fill, k(K::KeyF)),
    (A::Stroke, k(K::KeyS)),
    (A::FlipH, k(K::KeyH).shift()),
    (A::FlipV, k(K::KeyV).shift()),
    (A::Transform, k(K::KeyT).ctrl()),
    (A::NewBrush, k(K::KeyB).ctrl()),
    (A::ReplaceColor, k(K::KeyR).shift()),
    (A::Outline, k(K::KeyO).shift()),
    (A::ColorCurve, k(K::KeyM).ctrl()),
    (A::ColorCurve, k(K::F10)),
    (A::HueSaturation, k(K::KeyU).ctrl()),
    (A::ConvolutionMatrix, k(K::F9)),
    (A::KeyboardShortcuts, k(K::KeyK).ctrl().alt().shift()),
    (A::SelectAll, k(K::KeyA).ctrl()),
    (A::Deselect, k(K::KeyD).ctrl()),
    (A::Reselect, k(K::KeyD).ctrl().shift()),
    (A::InvertSelection, k(K::KeyI).ctrl().shift()),
    (A::ShowGrid, k(K::Quote).ctrl()),
    (A::PixelGrid, k(K::Quote).ctrl().shift()),
    (A::OnionSkin, k(K::F3)),
    (A::Preview, k(K::F7)),
    (A::Timeline, k(K::Tab)),
    (A::NewLayer, k(K::KeyN).shift()),
    (A::NewTilemapLayer, k(K::KeyN).space()),
    (A::NewGroup, k(K::KeyN).alt().shift()),
    (A::LayerViaCopy, k(K::KeyJ).ctrl()),
    (A::LayerViaCut, k(K::KeyJ).ctrl().shift()),
    (A::LayerFromClipboard, k(K::KeyV).ctrl().shift()),
    (A::LayerRename, k(K::KeyP).shift()),
    (A::LayerRename, k(K::F2)),
    (A::LayerVisibility, k(K::KeyX).shift()),
    (A::LayerOpacity25, k(K::Digit1).shift()),
    (A::LayerOpacity50, k(K::Digit2).shift()),
    (A::LayerOpacity75, k(K::Digit3).shift()),
    (A::LayerOpacity100, k(K::Digit4).shift()),
    (A::NewFrame, k(K::KeyN).alt()),
    (A::NewEmptyFrame, k(K::KeyB).alt()),
    (A::DuplicateLinked, k(K::KeyM).alt()),
    (A::DuplicateLinked, k(K::KeyD).alt().shift()),
    (A::DeleteFrame, k(K::KeyC).alt()),
    (A::ReverseFrames, k(K::KeyI).alt()),
    (A::FirstFrame, k(K::Home)),
    (A::LastFrame, k(K::End)),
    (A::PreviousFrame, k(K::ArrowLeft)),
    (A::NextFrame, k(K::ArrowRight)),
    (A::Play, k(K::Enter)),
    (A::Play, k(K::NumpadEnter)),
    (A::PlayPreview, k(K::Enter).shift()),
    (A::LoopSection, k(K::F2)),
    (A::BrushSlot1, k(K::Digit1).alt()),
    (A::BrushSlot2, k(K::Digit2).alt()),
    (A::BrushSlot3, k(K::Digit3).alt()),
    (A::BrushSlot4, k(K::Digit4).alt()),
    (A::BrushSlot5, k(K::Digit5).alt()),
    (A::BrushSlot6, k(K::Digit6).alt()),
    (A::BrushSlot7, k(K::Digit7).alt()),
    (A::BrushSlot8, k(K::Digit8).alt()),
    (A::BrushSlot9, k(K::Digit9).alt()),
    (A::BrushFlipH, k(K::KeyH).space()),
    (A::BrushFlipH, k(K::KeyX).space()),
    (A::BrushFlipV, k(K::KeyV).space()),
    (A::BrushFlipV, k(K::KeyY).space()),
    (A::BrushRotate, k(K::KeyR).space()),
    (A::SwapColors, k(K::KeyX)),
    (A::CanvasSize, k(K::KeyC)),
    (A::SpriteSize, k(K::KeyI).ctrl().alt()),
    (A::TileModeToggle, k(K::Tab).space()),
    (A::TileManual, k(K::Digit1).space()),
    (A::TileAuto, k(K::Digit2).space()),
    (A::TileStack, k(K::Digit3).space()),
    (A::CommitTransform, k(K::Enter)),
    (A::CommitTransform, k(K::NumpadEnter)),
    (A::Zoom100, k(K::Digit1)),
    (A::Zoom200, k(K::Digit2)),
    (A::Zoom400, k(K::Digit3)),
    (A::Zoom800, k(K::Digit4)),
    (A::Zoom1600, k(K::Digit5)),
    (A::Zoom3200, k(K::Digit6)),
    (A::Zoom50, k(K::Backquote)),
    (A::ZoomIn, k(K::Equal).ctrl()),
    (A::ZoomIn, k(K::Equal).ctrl().shift()),
    (A::ZoomOut, k(K::Minus).ctrl()),
    (A::BrushSmaller, k(K::Minus)),
    (A::BrushSmaller, k(K::NumpadSubtract)),
    (A::BrushBigger, k(K::Equal).shift()),
    (A::BrushBigger, k(K::NumpadAdd)),
    (A::ColorPrevious, k(K::BracketLeft)),
    (A::ColorPrevious, k(K::Digit9)),
    (A::ColorNext, k(K::BracketRight)),
    (A::ColorNext, k(K::Digit0)),
    (A::LayerAbove, k(K::ArrowUp)),
    (A::LayerBelow, k(K::ArrowDown)),
    (A::FramePreviousInTag, k(K::Comma)),
    (A::FrameNextInTag, k(K::Period)),
    (A::ToolRectMarquee, k(K::KeyM)),
    (A::ToolEllipseMarquee, k(K::KeyM).shift()),
    (A::ToolLasso, k(K::KeyQ)),
    (A::ToolPolygonLasso, k(K::KeyQ).shift()),
    (A::ToolMagicWand, k(K::KeyW)),
    (A::ToolPencil, k(K::KeyB)),
    (A::ToolSpray, k(K::KeyB).shift()),
    (A::ToolEraser, k(K::KeyE)),
    (A::ToolEyedropper, k(K::KeyI)),
    (A::ToolHand, k(K::KeyH)),
    (A::ToolTransform, k(K::KeyV)),
    (A::ToolSlice, k(K::KeyC).shift()),
    (A::ToolBucket, k(K::KeyG)),
    (A::ToolGradient, k(K::KeyG).shift()),
    (A::ToolLine, k(K::KeyL)),
    (A::ToolCurve, k(K::KeyL).shift()),
    (A::ToolRectangle, k(K::KeyU)),
    (A::ToolEllipse, k(K::KeyU).shift()),
    (A::ToolContour, k(K::KeyD)),
    (A::ToolPolygon, k(K::KeyD).shift()),
    (A::ToolBlurJumble, k(K::KeyR)),
    (A::ToolText, k(K::KeyT)),
];

/// As linhas do Aseprite que a fábrica NÃO segue, com o porquê (`P12_teclas.md`).
pub const OWNER_EXCEPTIONS: &[(A, Chord, &str)] = &[
    (
        A::ToolBucket,
        k(K::KeyG),
        "G is the app grid (owner 07/11); bucket is P",
    ),
    (
        A::ToolGradient,
        k(K::KeyG).shift(),
        "Shift+G is the app grid; gradient is Shift+P",
    ),
    (
        A::LayerRename,
        k(K::KeyP).shift(),
        "P/Shift+P belong to the bucket (owner 08/11 §9.1)",
    ),
    (
        A::LoopSection,
        k(K::F2),
        "F2 renames, as in the whole app (§9.1)",
    ),
    (
        A::ConvolutionMatrix,
        k(K::F9),
        "F9 is the app legacy chrome, read before the Pixel",
    ),
    (
        A::Timeline,
        k(K::Tab),
        "Tab is the app mode (owner 06/11); timeline is F4",
    ),
];
