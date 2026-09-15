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
        src.contains("let malha = malha_sob_o_cursor("),
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
        src.contains("painter.set_canvas_warp(warp_da_malha(malha));"),
        "{} resolve a UV pela malha e não entrega a DEFORMAÇÃO ao pincel: o dab continua redondo na \
         textura, e no ecrã ele sai esticado pelo tanto que a arte está dobrada.",
        f.display()
    );
    // ⭐⭐⭐ **E o TAMANHO DO DAB entra na pergunta** (3.º report do dono, 2026-09-14: *«quase bom …
    // talvez artefato inevitável»*). Uma malha é afim POR TRIÂNGULO: perguntar num PONTO dá ao dab
    // inteiro a deformação de um pedaço dele, e com um pincel grande sobre uma malha grossa isso
    // chega a deixar a marca MENOS redonda do que não corrigir nada (`1,38` contra `1,19`, medido).
    assert!(
        src.contains("let raio_px = painter.dab_footprint_px();")
            && src.contains("let footprint_uv = [raio_px / iw as f32, raio_px / ih as f32];"),
        "{} não pergunta ao pincel que tamanho o dab vai ter: sem isso a porta da malha só sabe \
         responder por um PONTO.",
        f.display()
    );
    // ⚠️ **A ligação prova-se nos ARGUMENTOS da chamada, nunca pela linha ao lado** — uma agulha
    // ancorada na adjacência é um proxy que expira na primeira linha que alguém acrescenta (a lei
    // que a W9 desta linha pagou). Aqui recorta-se a chamada e lê-se o que ela de facto recebe.
    let chamada = {
        let ini = src
            .find("ph2d_render::mesh_uv(")
            .expect("a chamada da porta de canvas, afirmada acima");
        let fim = src[ini..].find(");").expect("a chamada fecha") + ini;
        &src[ini..fim]
    };
    assert!(
        chamada.contains("footprint_uv"),
        "{} chama a porta da malha SEM o footprint do dab — ela responde por um ponto, e a \
         deformação que o pincel recebe passa a ser a de um pedaço do dab: {chamada}",
        f.display()
    );
    // ⚠️⚠️ **E o NOME ligado a um literal nulo lê-se igual ao nome ligado ao raio** — uma mutação que
    // escreveu `let footprint_uv = [0.0, 0.0];` e deixou a chamada intacta SOBREVIVEU à primeira
    // redacção deste gate. *Citar uma porta não é consultá-la, e nomear um argumento não é
    // alimentá-lo.* ⇒ exige-se a DERIVAÇÃO, que é o único sítio onde o raio do pincel entra.
    assert!(
        src.contains("let footprint_uv = [raio_px / iw as f32, raio_px / ih as f32];"),
        "{} nomeia o footprint do dab e não o DERIVA do raio do pincel — um `[0, 0]` ali é \
         literalmente *«não vou pintar»*, e a porta volta a responder por um ponto.",
        f.display()
    );
}

/// ⭐⭐⭐ **AS DUAS ENTRADAS DE CANVAS PERGUNTAM À MALHA — a pincelada E o passeio.**
///
/// ⛔⛔ **Medido em 2026-09-14: o `deliver_canvas_hover` NÃO perguntava.** A wave anterior curou a
/// pincelada e deixou o passeio a mapear o cursor pelo afim do quad de **REPOUSO**, sem deformação
/// nenhuma. ⇒ numa arte dobrada, enquanto o artista apenas passeia o rato, a caneta da selecção
/// mirava no sítio errado, a orientação do traço era calculada noutro espaço, e a deformação que o
/// anel do pincel lê ficava **presa no que a última pincelada deixou**.
///
/// ⚠️ **Uma lei escrita numa das duas entradas ainda não é uma lei** — e é por isso que este gate
/// exige a PORTA (`malha_sob_o_cursor`) e os **dois** chamadores, e não duas cópias que concordam
/// hoje.
#[test]
fn both_canvas_entries_ask_the_mesh_through_one_door() {
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
    // A porta existe, e é ela que compõe a pegada — não os chamadores.
    assert!(
        src.contains("pub(crate) fn malha_sob_o_cursor(")
            && src.contains("ph2d_render::mesh_uv(present, bits, world, starting, footprint_uv)"),
        "{} deixou de ter a porta única da malha: sem ela a lei volta a ser duas cópias.",
        f.display()
    );
    // Controlo positivo: as duas entradas existem.
    for entrada in ["fn deliver_canvas_pointer(", "fn deliver_canvas_hover("] {
        assert!(
            src.contains(entrada),
            "{} deixou de ter `{entrada}` — este gate perdeu um sujeito",
            f.display()
        );
    }
    // ⚠️ **Duas chamadas, não uma**: a pincelada e o passeio. Uma só deixa metade do app a mapear
    // pelo quad de repouso, que é exactamente o defeito medido.
    let chamadas = src.matches("malha_sob_o_cursor(").count();
    assert!(
        chamadas >= 3,
        "{} chama a porta da malha {} vez(es) (a definição + os dois chamadores = 3): uma das \
         entradas de canvas voltou a mapear o cursor pelo quad de REPOUSO.",
        f.display(),
        chamadas
    );
    // E o passeio tem de USAR a resposta, não só pedi-la — a mutação que este gate mata é o
    // `let _ = malha_sob_o_cursor(..)` ao lado do afim de sempre.
    assert!(
        src.contains("ph2d_render::MeshUv::Use { u, v, .. } => (u * iw as f32, v * ih as f32)"),
        "{} pergunta à malha no passeio e DEITA FORA a posição que ela devolve.",
        f.display()
    );
}

/// ⭐⭐⭐ **O ANEL DO CURSOR NÃO TEM LEI DE ORIENTAÇÃO PRÓPRIA — ele percorre UMA porta.**
///
/// ⛔⛔⛔ **E o que a porta devolve é a pegada que o ARTISTA VÊ, não a que o motor emite** (2.º
/// report do dono com foto, 2026-09-14: *«o gizmo do pincel se deforma ao passar por cima das faces
/// dobradas»*). Sobre arte dobrada o motor pinta na textura a elipse que a deformação **endireita**
/// — no ecrã ela sai redonda —, e desenhar essa directamente no ecrã mostra-a torta. *A 1.ª
/// redacção desta wave corrigiu uma coisa que já estava certa, e a foto foi a prova.*
///
/// ⚠️ **Este gate é ESTRUTURAL e não bastava sozinho** — ele ficou verde sobre a inversão, porque
/// o que ele mede é *«o anel lê a porta»* e não *«a forma está certa»*. A metade que faltava é a
/// identidade `W · (W⁻¹·E) = E`, medida na `ph2d-painter-brush`
/// (`the_painted_dab_seen_through_the_warp_is_the_authored_ellipse`).
#[test]
fn the_cursor_ring_walks_the_engine_footprint() {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("crates/ph2d-app-painter/src/painter_bridge_brush_ring.rs");
    let src: String = std::fs::read_to_string(&f)
        .expect("o anel do cursor do Painter")
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n");
    // Controlo positivo: é ESTE o sítio que desenha o anel.
    assert!(
        src.contains("BRUSH_RING_SEGS"),
        "{} deixou de desenhar o anel — este gate perdeu o sujeito",
        f.display()
    );
    assert!(
        src.contains("painter.cursor_dab()"),
        "{} não pergunta à ferramenta que pegada mostrar: sem a porta, o anel volta a ter uma lei \
         de orientação própria — e foi ela que inverteu o sentido sem nada acusar.",
        f.display()
    );
    assert!(
        src.contains("fp.outline_at(t as f32)"),
        "{} pede a pegada e DEITA FORA o contorno dela — remontar a elipse aqui é escrever a lei \
         uma segunda vez, que é o defeito que esta porta cura.",
        f.display()
    );
    // ⚠️ E o TAMANHO sai da mesma porta, pela mesma razão: uma segunda fonte para o raio é uma
    // segunda lei a envelhecer.
    assert!(
        !src.contains("bs.size_px"),
        "{} volta a tirar o raio de um segundo sítio: o raio e a forma têm de vir da MESMA porta, \
         senão eles discordam no dia em que uma delas aprender alguma coisa.",
        f.display()
    );
}
