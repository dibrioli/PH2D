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
        // ⭐ **A face VAZIA** (2026-09-09): o painel aberto sem cena 3D. ⚠️ Ela diz o que
        // FAZER — «nothing to show» repetiria ao artista o que ele já está a ver.
        "panel.sculpt3d.empty" => {
            "No 3D scene open \u{00b7} start one from Sculpt 3D in the top bar."
        }
        "panel.sculpt3d.section.tool" => "Tool",
        "panel.sculpt3d.section.brush" => "Brush",
        "panel.sculpt3d.section.symmetry" => "Symmetry",
        "panel.sculpt3d.section.topology" => "Topology",
        "panel.sculpt3d.section.shading" => "Shading",
        "panel.sculpt3d.section.scene" => "Scene",
        "panel.sculpt3d.section.bake" => "Bake to Sprite",
        "panel.sculpt3d.radius" => "Radius",
        "panel.sculpt3d.surface_only" => "Connected Only",
        "panel.sculpt3d.strength" => "Strength",
        "panel.sculpt3d.falloff" => "Falloff",
        // ⭐⭐⭐ **A RAZÃO À VISTA** — a fileira da curva é a única que o painel do
        // sculpt pinta SEMPRE (cerca de produto medida e gateada), e três
        // pincéis não a leem. Cada linha nomeia o FACTO e, onde há uma, a CURA.
        // ⚠️ Uma frase só para os três seria mais curta e diria menos: *«não faz
        // nada»* e *«não faz nada ATÉ subir os segmentos»* levam a gestos
        // diferentes.
        "panel.sculpt3d.falloff_inert.mask" => {
            "Not used by Mask — the mask has its own curve (Mask hardness)"
        }
        "panel.sculpt3d.falloff_inert.density" => {
            "Not used by Density — it changes topology, not vertex positions"
        }
        "panel.sculpt3d.falloff_inert.pose" => "Only used by Twist, and only with Segments above 1",
        "panel.sculpt3d.falloff_inert.trim" => "Not used: the cut is bounded by the shape you draw",
        // ⚠️ Os rótulos do alvo, sem tradução: *Deformation* e *Simulation
        // Area* são o que o painel dele diz (espec §8.4), e um artista que vem
        // de lá procura essas duas palavras.
        "panel.sculpt3d.smear_mode" => "Deformation",
        "panel.sculpt3d.trim_forma" => "Shape",
        "panel.sculpt3d.trim_smooth" => "Smooth Stroke",
        // ⭐⭐ O PINCEL DE PLANO: os dois tectos, a extensão do centro e o que o
        // `Ctrl` faz. ⚠️ **«Height»/«Depth» e não «cima»/«baixo»**: o que eles
        // medem é *quanto* o pincel alcança de cada lado do plano, e é um
        // COMPRIMENTO — o lado já está dito pela posição das duas pistas.
        "panel.sculpt3d.plano_altura" => "Height",
        "panel.sculpt3d.plano_profundidade" => "Depth",
        "panel.sculpt3d.plano_area" => "Area Radius",
        "panel.sculpt3d.plano_firmeza_normal" => "Hold Tilt",
        "panel.sculpt3d.plano_firmeza_centro" => "Hold Height",
        "panel.sculpt3d.plano_inversao" => "Ctrl Does",
        "panel.sculpt3d.project_mode" => "Ray Direction",
        "panel.sculpt3d.project_bidir" => "Search Both Ways",
        "panel.sculpt3d.project_min_dist" => "Gap",
        "panel.sculpt3d.cloth_mode" => "Deformation",
        // ── O pincel de POSE ────────────────────────────────────────────────
        // ⚠️ **«Deformation» é a MESMA palavra do tecido, e está certo:** as
        // duas fileiras respondem à mesma pergunta — *como é que este pincel
        // deforma?* — e só uma delas é desenhada de cada vez, porque cada uma
        // pergunta ao seu verbo.
        "panel.sculpt3d.pose_mode" => "Deformation",
        // ⚠️ **«Deformation» outra vez, e é de propósito:** é a MESMA pergunta
        // que a pose faz — *o que este pincel faz com o gesto* —, e os dois
        // nunca aparecem juntos (cada fileira só é pintada com o seu verbo na
        // mão). *Dois rótulos diferentes para a mesma pergunta é que seriam
        // duas coisas a aprender.*
        "panel.sculpt3d.boundary_mode" => "Deformation",
        "panel.sculpt3d.boundary_falloff" => "Falloff along the edge",
        "panel.sculpt3d.boundary_offset" => "Origin offset",
        "panel.sculpt3d.pose_segments" => "Segments",
        // ⚠️ **«do cursor» está no rótulo de propósito:** o número é em raios de
        // pincel e o que ele afasta é o PIVÔ, não a região. Sem isso ele lê-se
        // como um deslocamento da malha.
        "panel.sculpt3d.pose_offset" => "Pivot offset from cursor",
        "panel.sculpt3d.pose_transition" => "Transition",
        // ⚠️ **O rótulo diz o que a caixa FAZ, não o nome interno da opção:** o
        // que ela prende é a ponta distante da cadeia, e é isso que faz o gesto
        // rodar em torno de um pivô fixo.
        "panel.sculpt3d.pose_anchored" => "Pin far end",
        "panel.sculpt3d.pose_rot_lock" => "Scale without rotating",
        "panel.sculpt3d.cloth_area" => "Simulation Area",
        "panel.sculpt3d.cloth_force_falloff" => "Force Falloff",
        "panel.sculpt3d.cloth_pin" => "Pin Simulation Boundary",
        "panel.sculpt3d.cloth_persistent" => "Persistent",
        "panel.sculpt3d.cloth_collisions" => "Use Collisions",
        "panel.sculpt3d.cloth_set_base" => "Set Persistent Base",
        "panel.sculpt3d.cloth_limit" => "Simulation Limit",
        "panel.sculpt3d.cloth_falloff" => "Simulation Falloff",
        "panel.sculpt3d.cloth_mass" => "Cloth Mass",
        "panel.sculpt3d.cloth_damping" => "Cloth Damping",
        // ⚠️ **O pincel mantem o nome do ALVO**, e a divergencia e' deliberada:
        // ali cada rotulo tem uma fixture do oraculo com o mesmo nome, e um
        // artista que siga um tutorial do alvo tem de o encontrar. O FILTRO nao
        // tem esse laco — ver `cfilter_plasticity`.
        "panel.sculpt3d.cloth_plasticity" => "Soft Body Plasticity",
        // ⭐ A *Quality* do pincel — as varreduras que o alvo FIXA em 5.
        "panel.sculpt3d.cloth_sweeps" => "Cloth Quality",
        // ⭐⭐ Os quatro do FILTRO. ⚠️ O prefixo "Filter" é o que os separa dos do
        // PINCEL na mesma coluna: os dois conjuntos existem, com omissões e
        // faixas diferentes, e um rótulo repetido faria o artista concluir que
        // são o mesmo número.
        "panel.sculpt3d.cfilter_mass" => "Filter Mass",
        "panel.sculpt3d.cfilter_damping" => "Filter Damping",
        // ⭐⭐⭐ **«Shape Memory» e nao «Plasticity»** — report do dono de
        // 2026-09-09: *«Plasticity parece ter efeito parecido com Damping,
        // resistindo a` simulacao»*. Ele leu o controlo certo: com o valor ALTO o
        // vertice e' puxado de volta a` forma inicial, logo ele RESISTE.
        //
        // ⛔⛔ **E a palavra dizia o CONTRARIO do que o controlo faz.** Em
        // materiais, *plasticidade* e' deformacao PERMANENTE — o oposto do
        // retorno elastico. Aqui `1` = volta inteira a` forma, `0` = a memoria
        // segue o vertice e nada volta. *O nome vinha do alvo; o efeito e' o
        // inverso do que a palavra promete, e quem le^ o painel e' o artista.*
        //
        // ⚠️ A LEI mantem o nome do alvo (`Solver::plasticidade`) — e' por ele
        // que as 103 fixtures do oraculo falam.
        "panel.sculpt3d.cfilter_plasticity" => "Shape Memory",
        "panel.sculpt3d.cfilter_sweeps" => "Filter Quality",
        "panel.sculpt3d.cfilter_stretch" => "Stretch Limit",
        "panel.sculpt3d.cfilter_volume" => "Preserve Volume",
        "panel.sculpt3d.cfilter_strength" => "Filter Strength",
        "panel.sculpt3d.cfilter_bend" => "Bend Stiffness",
        "panel.sculpt3d.cfilter_axis" => "Force Axis",
        "panel.sculpt3d.cfilter_collisions" => "Filter Collisions",
        // ⚠️ **A row lê `Reference`, e os chips leem `S` · `B` · `L`** (§1.4 do
        // plano): o artista não sabe o que é o SculptGL, e o nome de um produto
        // de terceiro num botão é ruído que envelhece. Trocar para os nomes por
        // extenso é uma linha aqui, se o Enio preferir.
        "panel.sculpt3d.reference" => "Reference",
        "panel.sculpt3d.reference_all" => "Apply to all tools",
        "panel.sculpt3d.filter" => "Filter Whole Mesh",
        "panel.sculpt3d.filter_kind" => "Filter",
        "panel.sculpt3d.cloth_filter_kind" => "Cloth",
        "panel.sculpt3d.cloth_filter_orient" => "Orientation",
        "panel.sculpt3d.elastic_scales" => "Field width",
        "panel.sculpt3d.tip_roundness" => "Tip roundness",
        "panel.sculpt3d.strip_length" => "Strip length",
        "panel.sculpt3d.scrape_angle" => "Plane angle",
        "panel.sculpt3d.layer_height" => "Layer height",
        "panel.sculpt3d.normal_radius" => "Normal radius",
        "panel.sculpt3d.grab_anchor" => "Anchor on vertex",
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
        "panel.sculpt3d.dyn_detail" => "Detail",
        "panel.sculpt3d.density_detail" => "Detail",
        "panel.sculpt3d.level" => "Level",
        "panel.sculpt3d.subdivide" => "Subdivide",
        "panel.sculpt3d.reverse" => "Reverse",
        "panel.sculpt3d.flatten" => "Flatten Levels",
        "panel.sculpt3d.remesh" => "Remesh",
        "panel.sculpt3d.remesh_res" => "Remesh Resolution",
        "panel.sculpt3d.quad_remesh" => "Quad Retopology",
        "panel.sculpt3d.retopo_mode" => "Engine",
        "panel.sculpt3d.retopo_mode.global" => "Even Grid",
        "panel.sculpt3d.retopo_mode.local" => "Fast",
        "panel.sculpt3d.quad_detail" => "Detail",
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
