//! **A BARRA DE FERRAMENTAS** — o rail esquerdo (as duas caras: objecto e Painter, com os flyouts
//! de forma e de máscara) e a fila horizontal que o substitui no redesenho.
//!
//! ⚠️ **Duas palavras por ferramenta, e são coisas diferentes**: `chrome.rail.<x>` é o NOME (o que a
//! acessibilidade e a dica dizem) e `chrome.rail.sub.<x>` é o SUB-RÓTULO vertical, abreviado para
//! caber na coluna do chip (`LIQFY`, `XFORM`, `INPNT`). Uma tradução do sub tem de caber no mesmo
//! chip — é o mesmo contrato dos rótulos abreviados das ferramentas de imagem.
//!
//! ⚠️ `C&F` é *Colour & Fill* — o poço de cor do Painter (abre o seletor; arrastar = balde).

/// A tradução de uma chave da barra de ferramentas, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "chrome.rail.brush" => "Brush",
        "chrome.rail.sub.brush" => "BRUSH",
        "chrome.rail.eyedropper" => "Eyedropper",
        "chrome.rail.sub.pick" => "PICK",
        "chrome.rail.eraser" => "Eraser",
        "chrome.rail.sub.erase" => "ERASE",
        "chrome.rail.clone" => "Clone",
        "chrome.rail.sub.clone" => "CLONE",
        "chrome.rail.smear" => "Smear",
        "chrome.rail.sub.smear" => "SMEAR",
        "chrome.rail.blur" => "Blur",
        "chrome.rail.sub.blur" => "BLUR",
        "chrome.rail.liquify" => "Liquify",
        "chrome.rail.sub.liqfy" => "LIQFY",
        "chrome.rail.transform" => "Transform",
        "chrome.rail.sub.xform" => "XFORM",
        "chrome.rail.mask" => "Mask",
        "chrome.rail.sub.mask" => "MASK",
        "chrome.rail.inpaint" => "Inpaint",
        "chrome.rail.sub.inpnt" => "INPNT",
        "chrome.rail.shapes" => "Shapes",
        "chrome.rail.sub.shape" => "SHAPE",
        "chrome.rail.select" => "Select",
        "chrome.rail.sub.sel" => "SEL",
        "chrome.rail.free_hand" => "Free Hand",
        "chrome.rail.sub.free" => "FREE",
        "chrome.rail.line" => "Line",
        "chrome.rail.sub.line" => "LINE",
        "chrome.rail.curve" => "Curve",
        "chrome.rail.sub.curve" => "CURVE",
        "chrome.rail.ellipse" => "Ellipse",
        "chrome.rail.sub.elli" => "ELLI",
        "chrome.rail.polygon" => "Polygon",
        "chrome.rail.sub.poly" => "POLY",
        "chrome.rail.c_and_f" => "C&F",
        "chrome.rail.show_inspector" => "Show Inspector",
        "chrome.rail.show_hierarchy" => "Show Hierarchy",
        "chrome.rail.translate" => "Translate",
        "chrome.rail.rotate" => "Rotate",
        "chrome.rail.scale" => "Scale",
        "chrome.rail.pivot" => "Pivot",
        "chrome.rail.coordinate_space" => "Coordinate space",
        "chrome.rail.projection" => "Projection",
        "chrome.rail.frame_view" => "Frame view",
        "chrome.rail.undo" => "Undo",
        "chrome.rail.redo" => "Redo",
        "chrome.rail.rail_backdrop" => "Rail Backdrop",
        "chrome.rail.sub.c_and_f" => "C&F",
        "chrome.rail.editor_tools" => "Editor tools",
        "chrome.rail.sub.move" => "MOVE",
        "chrome.rail.sub.rot" => "ROT",
        "chrome.rail.sub.scale" => "SCALE",
        "chrome.rail.sub.pivot" => "PIVOT",
        "chrome.rail.local" => "Local",
        "chrome.rail.global" => "Global",
        "chrome.rail.sub.space" => "SPACE",
        "chrome.rail.camera" => "Camera",
        "chrome.rail.all" => "All",
        "chrome.rail.selected" => "Selected",
        "chrome.rail.sub.view" => "VIEW",
        "chrome.rail.sub.undo" => "UNDO",
        "chrome.rail.sub.redo" => "REDO",
        "chrome.tool_bar.more" => "More",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
