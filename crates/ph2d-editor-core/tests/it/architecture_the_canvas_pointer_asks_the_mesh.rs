//! **Arch-gate: a porta de canvas do Painter pergunta pela MALHA antes de usar o afim do quad.**
//!
//! ⛔⛔ **A lei existia em duas portas e o consumidor usava uma terceira** (medido 2026-09-14). O
//! `ph2d_render::{sprite_world_to_uv, sprite_world_to_uv_unclamped}` conhecem a malha desde a W3 do
//! plano `docs/Skeleton/03` e tinham **zero** chamadores de produto; o Painter mapeia o ponteiro
//! pelo afim do **quad de repouso** (`ph2d_sprite_screen::sprite_image_to_screen_affine`), que não
//! sabe o que é uma malha. Numa arte presa ao esqueleto e **dobrada**, cada pincelada caía deslocada
//! exactamente pela deformação — em toda a arte, e não só fora dela.
//!
//! ⇒ a lei dos três estados vive na [`ph2d_render::mesh_uv`] (gateada lá, com o controlo da sprite
//! sem malha), e este gate afirma que a porta de canvas **a consulta**. Ela vive em
//! `deliver_canvas_pointer`, que exige janela, GPU e superfície: **nenhum teste de unidade a
//! alcança** — é a forma do `the_motion_path_is_offered_only_on_the_keys_tab`, e mora nesta crate
//! pela razão medida que o `the_onion_speaks_the_clip_clock` (vizinho) escreve.

use std::path::Path;

#[test]
fn the_canvas_pointer_asks_the_mesh_before_the_quad_affine() {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop/src/input_dispatch/painter_canvas_input.rs");
    let src: String = std::fs::read_to_string(&f)
        .expect("a porta de canvas do Painter")
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n");
    // Controlo positivo: é ESTE o sítio que mapeia o ponteiro — sem ele o gate mede o nada.
    assert!(
        src.contains("sprite_image_to_screen_affine("),
        "{} deixou de mapear o ponteiro pelo afim da sprite — este gate perdeu o sujeito",
        f.display()
    );
    // ⚠️ **O NOME tem de ser LIGADO à porta, e não só citado** — esta metade nasceu de uma mutação
    // que SOBREVIVEU: um `let malha = MeshUv::Quad;` ao lado de um `let _ = mesh_uv(..)` deixava as
    // outras duas asserções verdes sobre o defeito inteiro. *Citar uma porta não é consultá-la.*
    assert!(
        src.contains("let malha = ph2d_render::mesh_uv("),
        "{} mapeia o ponteiro SÓ pelo afim do quad de repouso. Numa arte presa ao esqueleto e \
         dobrada isso põe cada pincelada deslocada pela deformação — a lei da malha é a \
         `ph2d_render::mesh_uv`, e `MeshUv::Quad` deixa o caminho de sempre intocado.",
        f.display()
    );
    assert!(
        src.contains("ph2d_render::MeshUv::Use { u: mu, v: mv, .. } => (mu, mv)"),
        "{} chama a porta e DEITA FORA a resposta dela: a UV da malha tem de ser a que segue para o \
         pincel, senão o gate acima fica verde sobre o defeito inteiro.",
        f.display()
    );
    // ⭐ **E a FORMA também** — a posição certa com o dab redondo na TEXTURA sai uma lasca no ecrã
    // onde a arte comprime (report do dono com foto, 2026-09-14).
    assert!(
        // ⚠️ **`match malha`, e não `set_canvas_warp(`**: pela mesma mutação sobrevivente de cima —
        // *citar a porta não é consultá-la*, e aqui o que se exige é que o argumento SAIA da resposta.
        src.contains("painter.set_canvas_warp(match malha {"),
        "{} resolve a UV pela malha e não entrega a DEFORMAÇÃO ao pincel: o dab continua redondo na \
         textura, e no ecrã ele sai esticado pelo tanto que a arte está dobrada.",
        f.display()
    );
}
