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
        // ⭐ **Os CHIPS das ferramentas de imagem** — vieram do `match` do pai em
        // 2026-09-17, quando ele passou o tecto de LOC. ⚠️ O cabeçalho deste ficheiro já os
        // nomeava por escrito (*«e' o mesmo contrato dos rótulos abreviados das ferramentas
        // de imagem»*): eles são a fila horizontal desta barra, logo o assunto sempre foi daqui.
        // Image Tools — action row pills. Labels abreviados (Enio
        // 2026-05-25): cabem na coluna do chip (44 px) sem clip; o
        // tooltip mantém o nome completo + descrição.
        "tool.trim_transparency.label" => "TRIM",
        "tool.trim_transparency.tooltip" => "Trim Transparency",
        "tool.make_square.label" => "SQUAR",
        "tool.make_square.tooltip" => "Make Square",
        "tool.bgremoval.label" => "BGRMV",
        "tool.bgremoval.tooltip" => "Background Removal · 3",
        "tool.real_size.label" => "SIZE",
        "tool.real_size.tooltip" => "Real Size · reset scale to 1:1",
        "tool.padding.label" => "PAD",
        "tool.padding.tooltip" => "Padding · expand or crop canvas edges",
        "tool.color_equalization.label" => "CEQ",
        "tool.color_equalization.tooltip" => {
            "Color Equalization · CLAHE + brightness/contrast/saturation + auto-WB"
        }
        "tool.equalize_sizes.label" => "EQSZ",
        "tool.equalize_sizes.tooltip" => {
            "Equalize Sizes · normalize selection to Max / Fixed / Grid target"
        }
        "tool.rasterize.label" => "RASTR",
        "tool.rasterize.tooltip" => {
            "Rasterize · bake scale + rotation into pixels (reset Transform)"
        }
        "tool.upscale.label" => "UPSC",
        "tool.upscale.tooltip" => "Upscale · resize image up 1x..16x (Lanczos3 / Nearest / xBR)",
        "tool.painter.label" => "PNTR",
        "tool.painter.tooltip" => {
            "Painter · sucessor do Procreate (brush engine GPU, history vetorial, MCP)"
        }
        // Toast strings — Trim / Make Square outcomes. Wiring goes
        // through `tr()` so the shell drainer doesn't hardcode the
        // English copies (HR-15). Format strings ("Trimmed → {w} × {h} px")
        // stay at call-site for now; the Fluent migration moves them
        // here as `format(key, args)`.
        "tool.trim_transparency.toast.nothing" => "Nothing to trim",
        "tool.trim_transparency.toast.unavailable" => "Trim unavailable for this sprite",
        "tool.make_square.toast.already_square" => "Sprite is already square",
        "tool.make_square.toast.unavailable" => "Make Square unavailable for this sprite",
        _ => return None,
    })
}
