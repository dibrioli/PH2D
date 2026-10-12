//! **O QUADRO — as imagens** (MiroClone W14): os avisos de soltar e importar, e as palavras da cena 15.

/// A tradução de uma chave das imagens do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.images.skipped" => "Skipped {count}: not an image",
        // W17 — o cabeçalho leu-se, os dados não: a imagem fica partida no quadro.
        "board.images.unreadable" => "A picture on this board is damaged and cannot be shown",
        "board.style.gray" => "Black and white",
        "board.style.flip_h" => "Flip horizontal (Shift+H)",
        "board.style.flip_v" => "Flip vertical (Shift+V)",
        "board.style.crop" => "Crop (double-click or Enter)",
        "board.images.svg_texts" => {
            "{count} text(s) inside the SVG will not show: SVG pictures draw without fonts"
        }
        "board.io.filter_images" => "Image",
        "board.io.filter_all" => "Excalidraw drawing or picture",
        "board.io.image_added" => "Added {name}",
        "board.live.too_big" => "A picture is too big to share live (over {max} MB)",
        "board.smoke.images.board" => "Pictures",
        "board.smoke.frames.board" => "Frames & B/W",
        "board.smoke.frames.title" => "Picture frames, black and white, and SVG pictures",
        "board.smoke.frames.frames" => "Frames",
        "board.smoke.frames.bw" => "Black and white",
        "board.smoke.frames.svg" => "SVG",
        "board.smoke.frames.l1" => {
            "1. Click a picture: the bar above it has the frame. Pick a colour (the crossed square = no frame), then the thickness and solid, dashed or dotted."
        }
        "board.smoke.frames.l2" => {
            "2. The half-black circle in the same bar turns the picture black and white. Click it again for the colours. Ctrl+Z takes it back."
        }
        "board.smoke.frames.l3" => {
            "3. With a framed picture selected, click Sketch on the toolbar: the frame is drawn by hand, like a shape."
        }
        "board.smoke.frames.l4" => {
            "4. Drop an SVG file (a logo, an icon) on this board, then zoom in with the wheel: it stays sharp, like the big star logo."
        }
        "board.smoke.frames.l5" => {
            "5. Scene tab: right-click a picture, vector or Flip in the Hierarchy, then Send to board… and pick a board. A Flip goes as its current frame."
        }
        "board.smoke.frames.l6" => {
            "6. Export the board (PNG, SVG or PDF): the frames and the black and white come out the same as here."
        }
        "board.smoke.w16.board" => "Crop, flip & GIF",
        "board.smoke.w16.title" => "Crop a picture, flip it, and a GIF that plays",
        "board.smoke.w16.crop" => "Crop",
        "board.smoke.w16.flip" => "Flip",
        "board.smoke.w16.gif" => "GIF",
        "board.smoke.w16.l1" => {
            "1. Double-click the left photo (or select it, Enter). Drag a corner or a side to crop; drag inside to slide it. Enter or Esc ends."
        }
        "board.smoke.w16.l2" => {
            "2. Click a photo: the two flip buttons in its bar mirror it (or Shift+H and Shift+V). The two on the right are flipped copies."
        }
        "board.smoke.w16.l3" => {
            "3. The GIF plays on the board. In an SVG export it keeps playing in a browser; PNG and PDF take its first frame."
        }
        "board.smoke.w16.l4" => {
            "4. In Dolphin, copy one or more pictures (Ctrl+C), come back to this board and press Ctrl+V: they come in side by side."
        }
        "board.smoke.w16.l5" => {
            "5. Scene tab: select several objects in the Hierarchy, right-click one, Send to board…: all go, turned and mirrored as in the scene."
        }
        "board.smoke.w16.l6" => {
            "6. Export this board as SVG: it is quick even with big photos, and the pictures go in as they arrived."
        }
        "board.smoke.w17.board" => "Known picture fixes",
        "board.smoke.w17.title" => "A damaged picture, Shift on a cropped one, and a quicker PDF",
        "board.smoke.w17.damaged" => "Damaged",
        "board.smoke.w17.crop" => "Crop with Shift",
        "board.smoke.w17.same" => "Same photo",
        "board.smoke.w17.l1" => {
            "1. The right picture's file is cut in half. It shows the rows that could be read, with a red mark in the corner, and the bar above the board says so once \u{2014} before, it stayed grey forever."
        }
        "board.smoke.w17.l2" => {
            "2. Double-click the tall picture, hold Shift and drag a corner: it keeps its own tall shape, not the shape of the whole photo."
        }
        "board.smoke.w17.l3" => {
            "3. Export this board as PDF: three pages with the same photo, and the photo is read once for the whole file, not once per page."
        }
        "board.smoke.w17.l4" => {
            "4. Scene tab, Send to board\u{2026}: an object with no name, or deleted meanwhile, is named in the warning \u{2014} never a blank."
        }
        "board.smoke.images.title" => "Pictures on the board: drop, paste, resize, export",
        "board.smoke.images.frame" => "Moodboard",
        "board.smoke.images.cover" => "Cover idea",
        "board.smoke.images.l1" => {
            "1. Drag a picture (PNG, JPEG, WebP, GIF…) from your file manager and drop it anywhere on this board: it lands where you let go, selected."
        }
        "board.smoke.images.l2" => {
            "2. Drag a corner of a picture: it grows or shrinks without stretching. The arrow stays on the photo when you move it."
        }
        "board.smoke.images.l3" => {
            "3. Click a picture: the bar above it offers sharp or round corners and the opacity."
        }
        "board.smoke.images.l4" => {
            "4. Copy a screenshot (or an image in the browser) and press Ctrl+V here: it lands in the middle of the view."
        }
        "board.smoke.images.l5" => {
            "5. Ctrl+C on a picture, open another board tab, Ctrl+V: the copy brings the picture with it."
        }
        "board.smoke.images.l6" => {
            "6. Right-click this tab, Export…: PNG, SVG, PDF and .excalidraw take the pictures along."
        }
        _ => return None,
    })
}
