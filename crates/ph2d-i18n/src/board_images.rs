//! **O QUADRO — as imagens** (MiroClone W14): os avisos de soltar e importar, e as palavras da cena 15.

/// A tradução de uma chave das imagens do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.images.skipped" => "Skipped {count}: not an image",
        "board.io.filter_images" => "Image",
        "board.io.filter_all" => "Excalidraw drawing or picture",
        "board.io.image_added" => "Added {name}",
        "board.live.too_big" => "A picture is too big to share live (over {max} MB)",
        "board.smoke.images.board" => "Pictures",
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
