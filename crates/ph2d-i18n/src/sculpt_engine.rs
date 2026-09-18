//! ⭐⭐ **AS PALAVRAS DO MOTOR DA ESCULTURA** — a 3.ª fatia da fronteira dos motores, a seguir ao
//! [`super::component_catalog`] e ao [`super::paint_engines`].
//!
//! ⛔⛔ **O mesmo defeito, pela terceira vez e no maior painel do app:** o `ph2d-panel-sculpt3d` tem
//! censo de HR-15 e fecha **VERDE** — ele já estava a zero quando a régua lá chegou — enquanto os
//! **35 verbos**, as **12 quedas**, os **10 alfas**, os **9 filtros**, os modos do pano e os três
//! verbos de transformação são literais em `ph2d-sculpt3d`, que **não tem censo nenhum**. *Um censo
//! cuja crate não é DONA do texto que ela pinta fica verde sobre texto cru.*
//!
//! ⚠️⚠️ **E nenhuma das 30 réguas lexicais o podia ver**: elas varrem os painéis, as `ph2d-app-*`,
//! a `ph2d-editor-core` e a shell — o motor não está na lista. Quem o achou foi o **idioma de
//! teste**, na 2.ª fotografia do dono (*«vários botões de sculpt»*).
//!
//! # ⭐ O motor guarda a CHAVE; o inglês vive aqui
//!
//! `Verb::label_key()` devolve `sculpt3d.verb.draw` e o [`crate::tr`] resolve-o. A
//! `Verb::label()` que 216 sítios já chamam **continua a devolver inglês** — ela passou a ser
//! `tr_em(Ingles, label_key())`, ou seja um ACESSÓRIO derivado desta tabela. ⇒ *uma lei só*, e os
//! testes do motor que comparam rótulos (as tabelas de proveniência do dyntopo, com o veredito do
//! dono escrito ao lado) continuam a ler inglês e a ser legíveis.
//!
//! ⚠️ **A chave deriva do par (enum, variante)** e o FICHEIRO não entra: o `cloth_mode.rs` e o
//! `cloth_filter_kind.rs` declaram DOIS enums cada, e uma chave derivada do ficheiro poria as duas
//! famílias no mesmo espaço de nomes.
//!
//! ⚠️ **O texto NÃO é derivável do nome da variante** — `Scales::Mono` mostra-se *Wide*,
//! `ProjectMode::Plane` mostra-se *Surface*, `Verb::EraseMultires` mostra-se *Erase Displacement* e
//! os três da `TrimForma` estão em português no código (`Caixa`) e em inglês na tela (`Box`). A
//! chave é derivada; o texto vive aqui.
//!
//! ⚠️ **As decisões de PRODUTO sobre um nome ficam no motor, ao lado da variante que as carrega** —
//! o porquê de `Verb::Plane` se chamar *Plane* e não *Trim*, e o de `DrawSharp` dizer o efeito e não
//! o mecanismo, estão em `ph2d-sculpt3d/src/brush_verb_label.rs`. *O texto muda de idioma; a decisão
//! não.*

/// A tradução de uma chave `sculpt3d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "sculpt3d.alpha.cracks" => "Cracks",
        "sculpt3d.alpha.grain" => "Grain",
        "sculpt3d.alpha.image" => "Image",
        "sculpt3d.alpha.noise" => "Noise",
        "sculpt3d.alpha.pores" => "Pores",
        "sculpt3d.alpha.ridges" => "Ridges",
        "sculpt3d.alpha.scales" => "Scales",
        "sculpt3d.alpha.scratches" => "Scratches",
        "sculpt3d.alpha.strata" => "Strata",
        "sculpt3d.alpha.weave" => "Weave",
        "sculpt3d.boundary_modo.agarrar" => "Grab",
        "sculpt3d.boundary_modo.dobrar" => "Bend",
        "sculpt3d.boundary_modo.expandir" => "Expand",
        "sculpt3d.boundary_modo.inflar" => "Inflate",
        "sculpt3d.boundary_modo.suavizar" => "Smooth",
        "sculpt3d.boundary_modo.torcer" => "Twist",
        "sculpt3d.boundary_queda.constante" => "Constant",
        "sculpt3d.boundary_queda.laco" => "Loop",
        "sculpt3d.boundary_queda.laco_invertido" => "Loop and Invert",
        "sculpt3d.boundary_queda.raio" => "Radius",
        "sculpt3d.cloth_area.dynamic" => "Dynamic",
        "sculpt3d.cloth_area.global" => "Global",
        "sculpt3d.cloth_area.local" => "Local",
        "sculpt3d.cloth_filter_kind.expand" => "Expand",
        "sculpt3d.cloth_filter_kind.gravity" => "Gravity",
        "sculpt3d.cloth_filter_kind.inflate" => "Inflate",
        "sculpt3d.cloth_filter_kind.pinch" => "Pinch",
        "sculpt3d.cloth_filter_kind.scale" => "Scale",
        "sculpt3d.cloth_filter_orientation.local" => "Local",
        "sculpt3d.cloth_filter_orientation.view" => "View",
        "sculpt3d.cloth_filter_orientation.world" => "World",
        "sculpt3d.cloth_force_falloff.plane" => "Plane",
        "sculpt3d.cloth_force_falloff.radial" => "Radial",
        "sculpt3d.cloth_mode.drag" => "Drag",
        "sculpt3d.cloth_mode.expand" => "Expand",
        "sculpt3d.cloth_mode.grab" => "Grab",
        "sculpt3d.cloth_mode.inflate" => "Inflate",
        "sculpt3d.cloth_mode.pinch_perpendicular" => "Pinch Perp",
        "sculpt3d.cloth_mode.pinch_point" => "Pinch Point",
        "sculpt3d.cloth_mode.push" => "Push",
        "sculpt3d.cloth_mode.snake_hook" => "Snake Hook",
        "sculpt3d.falloff.constant" => "Constant",
        "sculpt3d.falloff.dome" => "Dome",
        "sculpt3d.falloff.dome_4" => "Dome 4",
        "sculpt3d.falloff.inv_square" => "Inv Square",
        "sculpt3d.falloff.linear" => "Linear",
        "sculpt3d.falloff.plateau" => "Plateau",
        "sculpt3d.falloff.root" => "Root",
        "sculpt3d.falloff.sharp" => "Sharp",
        "sculpt3d.falloff.sharper" => "Sharper",
        "sculpt3d.falloff.smooth" => "Smooth",
        "sculpt3d.falloff.smoother" => "Smoother",
        "sculpt3d.falloff.sphere" => "Sphere",
        "sculpt3d.filter_kind.enhance_details" => "Enhance Details",
        "sculpt3d.filter_kind.inflate" => "Inflate",
        "sculpt3d.filter_kind.random" => "Random",
        "sculpt3d.filter_kind.relax" => "Relax",
        "sculpt3d.filter_kind.scale" => "Scale",
        "sculpt3d.filter_kind.sharpen" => "Sharpen",
        "sculpt3d.filter_kind.smooth" => "Smooth",
        "sculpt3d.filter_kind.sphere" => "Sphere",
        "sculpt3d.filter_kind.surface_smooth" => "Surface Smooth",
        "sculpt3d.plano_inversao.afastar" => "Push Away",
        "sculpt3d.plano_inversao.trocar_tectos" => "Swap Limits",
        "sculpt3d.pose_deformacao.escalar_transladar" => "Scale / Translate",
        "sculpt3d.pose_deformacao.espremer_esticar" => "Squash / Stretch",
        "sculpt3d.pose_deformacao.girar_torcer" => "Rotate / Twist",
        "sculpt3d.pose_modo.escalar" => "Scale",
        "sculpt3d.pose_modo.espremer" => "Squash / Stretch",
        "sculpt3d.pose_modo.rodar" => "Rotate",
        "sculpt3d.pose_modo.torcer" => "Twist",
        "sculpt3d.pose_modo.transladar" => "Translate",
        "sculpt3d.project_mode.plane" => "Surface",
        "sculpt3d.project_mode.view" => "View",
        "sculpt3d.ref_mode.b" => "B",
        "sculpt3d.ref_mode.l" => "L",
        "sculpt3d.ref_mode.s" => "S",
        "sculpt3d.scales.bi" => "Medium",
        "sculpt3d.scales.mono" => "Wide",
        "sculpt3d.scales.tri" => "Tight",
        "sculpt3d.smear_mode.drag" => "Drag",
        "sculpt3d.smear_mode.expand" => "Expand",
        "sculpt3d.smear_mode.pinch" => "Pinch",
        "sculpt3d.transform_kind.move" => "Move",
        "sculpt3d.transform_kind.rotate" => "Rotate",
        "sculpt3d.transform_kind.scale" => "Scale",
        "sculpt3d.trim_forma.caixa" => "Box",
        "sculpt3d.trim_forma.circulo" => "Circle",
        "sculpt3d.trim_forma.laco" => "Lasso",
        "sculpt3d.verb.blob" => "Blob",
        "sculpt3d.verb.boundary" => "Boundary",
        "sculpt3d.verb.box_trim" => "Box Trim",
        "sculpt3d.verb.clay" => "Clay",
        "sculpt3d.verb.clay_strips" => "Clay Strips",
        "sculpt3d.verb.clay_thumb" => "Clay Thumb",
        "sculpt3d.verb.cloth" => "Cloth",
        "sculpt3d.verb.crease" => "Crease",
        "sculpt3d.verb.density" => "Density",
        "sculpt3d.verb.draw" => "Draw",
        "sculpt3d.verb.draw_sharp" => "Draw Sharp",
        "sculpt3d.verb.erase_multires" => "Erase Displacement",
        "sculpt3d.verb.fill" => "Fill",
        "sculpt3d.verb.flatten" => "Flatten",
        "sculpt3d.verb.inflate" => "Inflate",
        "sculpt3d.verb.layer" => "Layer",
        "sculpt3d.verb.local_scale" => "Local Scale",
        "sculpt3d.verb.magnify" => "Magnify",
        "sculpt3d.verb.mask" => "Mask",
        "sculpt3d.verb.move" => "Move / Grab",
        "sculpt3d.verb.multiplane_scrape" => "Multiplane Scrape",
        "sculpt3d.verb.nudge" => "Nudge",
        "sculpt3d.verb.pinch" => "Pinch",
        "sculpt3d.verb.plane" => "Plane",
        "sculpt3d.verb.pose" => "Pose",
        "sculpt3d.verb.scene_project" => "Scene Project",
        "sculpt3d.verb.scrape" => "Scrape",
        "sculpt3d.verb.sharpen" => "Sharpen",
        "sculpt3d.verb.slide_relax" => "Slide Relax",
        "sculpt3d.verb.smear_multires" => "Smear Displacement",
        "sculpt3d.verb.smooth" => "Smooth",
        "sculpt3d.verb.snake_hook" => "Snake Hook",
        "sculpt3d.verb.surface_smooth" => "Surface Smooth",
        "sculpt3d.verb.thumb" => "Thumb",
        "sculpt3d.verb.twist" => "Twist",
        _ => return None,
    })
}
