//! **As palavras da tela dos tilemaps do PIXEL ART, P10b** — os nomes do Aseprite, em Plain English.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel_tiles.title" => "Tilemap",
        "panel.pixel.tiles.title_new" => "New Tilemap Layer",
        "panel.pixel.tiles.title_convert" => "Convert to Tilemap",
        "panel.pixel.tiles.no_document" => "Open a Pixel Sprite in Draw to see its tilemap.",
        "panel.pixel.tiles.empty" => {
            "No tilemap layer. Choose New Tilemap Layer in the Layers menu to make one."
        }
        "panel.pixel.tiles.tile_w" => "Tile width (px)",
        "panel.pixel.tiles.tile_h" => "Tile height (px)",
        "panel.pixel.tiles.new_set" => "New tileset",
        "panel.pixel.tiles.set_label" => "Tileset {i}: {n} {n?tile|tiles}, {w}×{h} px",
        "panel.pixel.tiles.new_hint" => "A new empty tileset is made for this layer.",
        "panel.pixel.tiles.shared_hint" => {
            "This layer draws with the same tiles: editing a tile changes it in every layer."
        }
        "panel.pixel.tiles.convert_hint_shared" => {
            "The image is cut into tiles. Parts equal to a tile use it; the rest are added."
        }
        "panel.pixel.tiles.convert_hint" => "The image is cut into tiles. Equal tiles become one.",
        "panel.pixel.tiles.match_flips" => "Match flipped tiles",
        "panel.pixel.tiles.create" => "Create",
        "panel.pixel.tiles.convert" => "Convert",
        "panel.pixel.tiles.cancel" => "Cancel",
        "panel.pixel.tiles.draw" => "Draw",
        "panel.pixel.tiles.pixels" => "Pixels",
        "panel.pixel.tiles.tiles" => "Tiles",
        "panel.pixel.tiles.manual" => "Manual",
        "panel.pixel.tiles.auto" => "Auto",
        "panel.pixel.tiles.stack" => "Stack",
        "panel.pixel.tiles.hint_tiles" => {
            "Pencil places the chosen tile. Eraser clears the cell. Alt+click picks a tile."
        }
        "panel.pixel.tiles.hint_manual" => "Painting edits the tile: every copy changes.",
        "panel.pixel.tiles.hint_auto" => {
            "Painting edits the tile if used once, or makes a new tile if used more."
        }
        "panel.pixel.tiles.hint_stack" => {
            "Painting always makes a new tile, unless an equal one exists."
        }
        "panel.pixel.tiles.tileset" => "Tileset",
        "panel.pixel.tiles.summary" => "{n} {n?tile|tiles} · {w}×{h} px",
        "panel.pixel.tiles.flip_x" => "Flip X",
        "panel.pixel.tiles.flip_y" => "Flip Y",
        "panel.pixel.tiles.flip_d" => "Flip diagonal",
        "panel.pixel.tiles.add" => "Add tile",
        "panel.pixel.tiles.delete" => "Delete tile",
        "panel.pixel.tiles.shape" => "Grid shape",
        "panel.pixel.tiles.shape_square" => "Square",
        "panel.pixel.tiles.shape_iso" => "Iso",
        "panel.pixel.tiles.shape_hex" => "Hex",
        "panel.pixel.tiles.shape_hex_flat" => "Hex Flat",
        "panel.pixel.tiles.terrain" => "Terrain",
        "panel.pixel.tiles.new_terrain" => "New",
        "panel.pixel.tiles.terrain_template" => "Template",
        "panel.pixel.tiles.delete_terrain" => "Delete",
        "panel.pixel.tiles.terrain_help" => {
            "A terrain has 47 tiles, one for each shape of its neighbours. Template opens a layer with them all, to paint over."
        }
        "panel.pixel.tiles.hint_terrain" => {
            "Pencil paints an area: each cell gets the tile that fits its neighbours. Eraser removes it."
        }
        "pixel.tiles.terrain_name" => "Terrain {n}",
        "pixel.tiles.template_layer" => "{name} template",
        "pixel.tiles.no_terrain" => "Make a terrain first: New, in the Terrain card",
        "pixel.tiles.refuse.terrain_shape" => {
            "Terrains work on Square and Iso grids: a hex cell has 6 neighbours, not 8"
        }
        "pixel.tiles.refuse.shape_terrain" => {
            "This tileset has terrains, and they do not work on a Hex grid"
        }
        "panel.pixel.layer.new_tilemap" => "New Tilemap Layer",
        "panel.pixel.layer.to_tilemap" => "Convert to Tilemap",
        "panel.pixel.layer.to_image" => "Convert to Image",
        "pixel.tiles.not_tilemap" => "This shortcut works on a tilemap layer",
        "pixel.tiles.only_pencil" => "In Tiles and Terrain, use the Pencil or the Eraser",
        "pixel.tiles.refuse.convert" => {
            "Only a plain image layer converts: not a group, a reference or the Background"
        }
        "pixel.tiles.refuse.delete" => {
            "This tile cannot be deleted: it is the empty tile or part of a terrain"
        }
        "pixel.tiles.refuse.sprite" => "Not available while this sprite has a tilemap layer",
        "pixel.tiles.refuse.cel_drag" => {
            "A tilemap cel can only move to a layer with the same tileset"
        }
        _ => return None,
    })
}
