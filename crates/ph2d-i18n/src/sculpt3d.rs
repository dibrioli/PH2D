//! **AS STRINGS DO PAINEL DE ESCULTURA** — o irmão de tabela do [`super`].
//!
//! ⚠️ **Um corte por ASSUNTO, e o gate de LOC foi o gatilho, não a razão.** O
//! `lib.rs` cruzou o teto de 700 quando esta família ganhou mais duas chaves, e
//! o que saiu foi o bloco de UM painel — a fatia que tem dono, cresce sozinha e
//! não se mistura com as outras.
//!
//! ⚠️ **E o corte compra ISOLAMENTO, que é o que o Modo L pede de um toque
//! foundational** (`CLAUDE.md` §0.2): enquanto todas as chaves de todos os
//! painéis moravam num `match` só, duas linhas paralelas que acrescentassem uma
//! chave cada colidiam no mesmo punhado de linhas. Uma tabela por painel é um
//! ponto de extensão que várias linhas estendem sem se ver.
//!
//! ⚠️ **`Option` e não `&str`:** devolver a chave crua aqui seria uma SEGUNDA
//! resposta a *"o que fazer com uma chave desconhecida?"* — o `leak_key` do pai
//! é quem responde isso, uma vez.

/// A tradução de uma chave `panel.sculpt3d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.sculpt3d.title" => "Sculpt 3D",
        "panel.sculpt3d.section.tool" => "Tool",
        "panel.sculpt3d.section.brush" => "Brush",
        "panel.sculpt3d.section.symmetry" => "Symmetry",
        "panel.sculpt3d.section.topology" => "Topology",
        "panel.sculpt3d.section.shading" => "Shading",
        "panel.sculpt3d.section.scene" => "Scene",
        "panel.sculpt3d.section.bake" => "Bake to Sprite",
        "panel.sculpt3d.radius" => "Radius",
        "panel.sculpt3d.strength" => "Strength",
        "panel.sculpt3d.falloff" => "Falloff",
        // ⚠️ **A row lê `Reference`, e os chips leem `S` · `B` · `L`** (§1.4 do
        // plano): o artista não sabe o que é o SculptGL, e o nome de um produto
        // de terceiro num botão é ruído que envelhece. Trocar para os nomes por
        // extenso é uma linha aqui, se o Enio preferir.
        "panel.sculpt3d.reference" => "Reference",
        "panel.sculpt3d.reference_all" => "Apply to all tools",
        "panel.sculpt3d.filter" => "Filter Whole Mesh",
        "panel.sculpt3d.filter_kind" => "Filter",
        "panel.sculpt3d.elastic_scales" => "Field width",
        "panel.sculpt3d.tip_roundness" => "Tip roundness",
        "panel.sculpt3d.strip_length" => "Strip length",
        "panel.sculpt3d.scrape_angle" => "Plane angle",
        "panel.sculpt3d.layer_height" => "Layer height",
        "panel.sculpt3d.scrape_dynamic" => "Read the surface",
        "panel.sculpt3d.ui_level" => "Detail",
        "panel.sculpt3d.ui_level.basic" => "Basic",
        "panel.sculpt3d.ui_level.pro" => "Pro",
        "panel.sculpt3d.hardness" => "Hardness",
        "panel.sculpt3d.auto_smooth" => "Auto-Smooth",
        "panel.sculpt3d.plane_offset" => "Plane Offset",
        "panel.sculpt3d.pinch" => "Pinch",
        "panel.sculpt3d.hc_shape" => "Shape Preservation",
        "panel.sculpt3d.hc_vertex" => "Per Vertex Displacement",
        "panel.sculpt3d.mask_hardness" => "Hardness",
        "panel.sculpt3d.alpha" => "Alpha",
        "panel.sculpt3d.alpha.none" => "None",
        "panel.sculpt3d.alpha_scale" => "Pattern Size",
        "panel.sculpt3d.stamp_scale" => "Stamp Size",
        "panel.sculpt3d.alpha_off_x" => "Stamp Offset X",
        "panel.sculpt3d.alpha_off_y" => "Stamp Offset Y",
        "panel.sculpt3d.alpha_az" => "Pattern Angle",
        "panel.sculpt3d.alpha_elev" => "Pattern Tilt",
        "panel.sculpt3d.alpha_preview" => "Preview on Model",
        "panel.sculpt3d.alpha.too_fine" => "Finer than this mesh resolves — subdivide (K)",
        "panel.sculpt3d.mask" => "Mask",
        "panel.sculpt3d.mask.clear" => "Clear",
        "panel.sculpt3d.mask.invert" => "Invert",
        "panel.sculpt3d.mask.blur" => "Blur",
        "panel.sculpt3d.mask.sharpen" => "Sharpen",
        "panel.sculpt3d.extract" => "Extract Mask",
        "panel.sculpt3d.transform" => "Transform Free Part",
        "panel.sculpt3d.extract_thickness" => "Extract Thickness",
        "panel.sculpt3d.extract_smooth" => "Extract Smooth",
        "panel.sculpt3d.sym.x" => "X",
        "panel.sculpt3d.sym.y" => "Y",
        "panel.sculpt3d.sym.z" => "Z",
        "panel.sculpt3d.dyntopo" => "Dynamic Topology",
        "panel.sculpt3d.detail" => "Detail",
        "panel.sculpt3d.detail.coarse" => "Coarse",
        "panel.sculpt3d.detail.medium" => "Medium",
        "panel.sculpt3d.detail.fine" => "Fine",
        "panel.sculpt3d.level" => "Level",
        "panel.sculpt3d.subdivide" => "Subdivide",
        "panel.sculpt3d.reverse" => "Reverse",
        "panel.sculpt3d.flatten" => "Flatten Levels",
        "panel.sculpt3d.remesh" => "Remesh",
        "panel.sculpt3d.remesh_res" => "Remesh Resolution",
        "panel.sculpt3d.quad_remesh" => "Quad Retopology",
        "panel.sculpt3d.quad_edge" => "Quad Size",
        "panel.sculpt3d.quad_adapt" => "Follow Curvature",
        "panel.sculpt3d.close_holes" => "Close Holes",
        // "Cavity" e não "Curvature": é o nome que Blender, ZBrush e Substance
        // dão ao MESMO canal, e o artista o procura por ele.
        "panel.sculpt3d.cavity" => "Cavity",
        "panel.sculpt3d.env" => "Environment",
        "panel.sculpt3d.ao" => "Ambient Occlusion",
        "panel.sculpt3d.bake_ao" => "Bake Occlusion + Thickness",
        // ⚠️ O rótulo nomeia o ALVO, não só o verbo. Este painel tem DOIS bakes
        // e a palavra sozinha não os separa: o de cima escreve um canal na
        // MALHA, este escreve a forma inteira no SPRITE selecionado.
        "panel.sculpt3d.bake_sprite" => "Light the Selected Sprite",
        "panel.sculpt3d.alpha_sprite" => "Use Selected Sprite as Pattern",
        "panel.sculpt3d.bake_sprite.hint" => "Select a sprite on the canvas — the form lights IT",
        "panel.sculpt3d.ssao" => "Screen Occlusion",
        "panel.sculpt3d.sss" => "Subsurface",
        "panel.sculpt3d.sss_scatter" => "Scatter",
        "panel.sculpt3d.ao_stale" => "Baked channels describe the previous shape",
        "panel.sculpt3d.matcap" => "Material",
        // ⚠️ "Rig" e não "None": a primeira opção NÃO é a ausência de luz, é a
        // luz do DOCUMENTO — a mesma lâmpada que acende a tinta ao lado. Chamá-la
        // de "None" faria o artista ler o modo default como "sem sombreamento".
        "panel.sculpt3d.matcap.rig" => "Rig",
        "panel.sculpt3d.wireframe" => "Wireframe",
        "panel.sculpt3d.accumulate" => "Accumulate",
        "panel.sculpt3d.front_faces" => "Front Faces Only",
        "panel.sculpt3d.light_az" => "Light Angle",
        "panel.sculpt3d.light_elev" => "Light Height",
        "panel.sculpt3d.add" => "Add",
        "panel.sculpt3d.add.sphere" => "Sphere",
        "panel.sculpt3d.add.cube" => "Cube",
        "panel.sculpt3d.add.cylinder" => "Cylinder",
        "panel.sculpt3d.add.torus" => "Torus",
        "panel.sculpt3d.duplicate" => "Duplicate",
        "panel.sculpt3d.delete" => "Delete",
        "panel.sculpt3d.isolate" => "Isolate",
        "panel.sculpt3d.merge" => "Merge Visible",
        "panel.sculpt3d.pieces" => "Pieces",
        "panel.sculpt3d.verts" => "Vertices",
        _ => return None,
    })
}
