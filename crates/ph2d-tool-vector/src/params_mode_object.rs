//! ⭐ **A partição do `DrawMode` pelos modos do OBJECTO** (spec/06 F3 ▸ Vector; D3: as 17 variantes
//! eram 2 modos — `Select` = Object, `Node` = Edit — e ferramentas). Mora ao lado do enum, num
//! ficheiro próprio, porque o do enum está no tecto de linhas.

use super::DrawMode;

impl DrawMode {
    /// As ferramentas do **Edit Mode** de uma forma (D6: *nós e alças*) — as que mexem na
    /// geometria que JÁ existe, no lugar (o Trim escreve no mesmo id). A ordem do painel.
    pub const EDIT_TOOLS: [Self; 5] = [
        Self::Node,
        Self::Fillet,
        Self::Chamfer,
        Self::Width,
        Self::Trim,
    ];

    /// ⭐ **O modo do objecto a que esta ferramenta pertence** (spec/06 F3 ▸ Vector; D3: o
    /// `DrawMode` era 2 modos + ferramentas). Edit = as [`Self::EDIT_TOOLS`]; o resto é Object —
    /// criar uma forma é criar um OBJECTO (uma linha da Hierarquia), e as que trabalham sobre várias
    /// (Build, Connect, Cut, Blend…) são de várias entidades, que o Edit tranca.
    #[must_use]
    pub const fn object_mode(self) -> ph2d_editor_core::object_mode::ObjectMode {
        use ph2d_editor_core::object_mode::ObjectMode;
        match self {
            Self::Node | Self::Fillet | Self::Chamfer | Self::Width | Self::Trim => {
                ObjectMode::Edit
            }
            Self::Select
            | Self::Pen
            | Self::Pencil
            | Self::Shape
            | Self::Text
            | Self::Build
            | Self::Connect
            | Self::PickBlend
            | Self::Bucket
            | Self::Cut
            | Self::Frame
            | Self::Bone => ObjectMode::Object,
        }
    }
}
