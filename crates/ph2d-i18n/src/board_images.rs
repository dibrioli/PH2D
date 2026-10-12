//! **O QUADRO — as imagens** (MiroClone W14): os avisos de soltar e importar, e as palavras da cena 15.

/// A tradução de uma chave das imagens do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.images.skipped" => "Skipped {count}: not an image",
        "board.style.gray" => "Black and white",
        "board.images.svg_texts" => "{count} text(s) inside the SVG will not show: SVG pictures draw without fonts",
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
            "5. Open the Scene tab, right-click a picture, a vector drawing or a Flip in the Hierarchy, choose Send to board… and pick this board (or New board). A Flip goes as the frame you are on, not the animation."
        }
        "board.smoke.frames.l6" => {
            "6. Export the board (PNG, SVG or PDF): the frames and the black and white come out the same as here."
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
