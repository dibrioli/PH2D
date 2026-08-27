//! **A CENA DA SUJIDADE NA LENTE** (`PH2D_GLOW_DIRT_SMOKE=1`) — a última célula P2 da folha 11
//! (doc 89): o *Dirt Texture* do Unity URP / o *Bloom Dirt Mask* do Unreal.
//!
//! ⚠️ **Ela NÃO é um nível do `PH2D_GPU_COOK_DEMO`, e a razão é a mesma que já está escrita para
//! o `PH2D_MOTION_OBJ_SMOKE=9`:** aqueles demos montam um GRAFO e amostram um ladrilho branco
//! chapado. Uma máscara de sujidade é uma IMAGEM — se ela for uma cor plana, o halo modulado é
//! indistinguível de um halo com outra intensidade, e o smoke provaria nada. Esta cena precisa
//! de uma textura a sério na cena, e é por isso que ela mora ao lado do `PH2D_SLICE_SMOKE`, no
//! único sítio do quadro onde o `sim`, o `renderer`, o `asset_db` e o átlas estão em mão juntos.
//!
//! O que ela monta:
//!
//! ```text
//!   uma sprite chamada «Lens Dirt»  ← a imagem, visivel na cena para se saber o padrao
//!   um campo de pecas brilhantes    ← o que o halo acende
//!   um no' «Glow» ja' ligado, com «Dirt Texture» = Lens Dirt
//! ```

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome que a sprite leva na Hierarchy — e que o campo `Dirt Texture` do nó nomeia.
pub(crate) const DIRT_NAME: &str = "Lens Dirt";

/// O lado da textura de sujidade, em pixels.
const DIRT_PX: u32 = 256;

/// Quanto a máscara acende, na cena montada. Alto de propósito: o smoke tem de deixar o
/// padrão gritar, e o artista baixa depois.
const DIRT_INTENSITY: f32 = 3.0;

/// Quantas peças brilhantes o campo tem por lado.
const FIELD: f32 = 7.0;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_GLOW_DIRT_SMOKE").is_some()
}

/// **A IMAGEM** — pó, borrões e dois riscos, sintetizados.
///
/// ⚠️ **Ela é quase toda ESCURA, e isso é o que a torna uma máscara de sujidade e não um filtro
/// de cor.** A contribuição é `halo · (tint + dirt·intensidade)`: onde a imagem é preta o halo
/// fica exactamente o de sempre, e é o CONTRASTE entre as duas regiões que se lê como sujidade.
/// Uma imagem clara em toda parte só somaria brilho, que é o que a `intensity` já faz.
///
/// ⚠️ **E ela é COLORIDA, de propósito** — os borrões puxam para âmbar e para azul. É a metade
/// da referência que uma máscara em cinzento não mostra: a sujidade acrescenta COR ao halo, e é
/// por isso que ela se soma ao `tint` em vez de multiplicar o resultado.
fn dirt_pixels() -> Vec<u8> {
    let n = DIRT_PX as usize;
    let mut lum = vec![0.02_f32; n * n];
    // Os borrões: centros, raios e forças escolhidos à mão (uma lista, não um gerador — o
    // padrão tem de ser o MESMO em toda máquina, senão duas fotos do smoke não se comparam).
    const BLOBS: [(f32, f32, f32, f32); 9] = [
        (0.18, 0.22, 0.16, 0.95),
        (0.31, 0.34, 0.07, 0.70),
        (0.72, 0.19, 0.13, 0.85),
        (0.83, 0.31, 0.05, 0.55),
        (0.47, 0.61, 0.19, 0.90),
        (0.21, 0.78, 0.11, 0.65),
        (0.66, 0.81, 0.14, 0.80),
        (0.90, 0.66, 0.08, 0.60),
        (0.09, 0.52, 0.06, 0.50),
    ];
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f32 / n as f32, y as f32 / n as f32);
            let mut a = lum[y * n + x];
            for (cx, cy, r, k) in BLOBS {
                let d = ((u - cx).powi(2) + (v - cy).powi(2)).sqrt() / r;
                if d < 1.0 {
                    // Queda suave (o quadrado do complemento) — uma borda dura leria como
                    // um adesivo, não como pó.
                    a += k * (1.0 - d) * (1.0 - d);
                }
            }
            // Dois riscos finos, atravessados: o que uma lente arranhada tem e um campo de
            // borrões não tem.
            for (m, c, w, k) in [
                (0.6_f32, 0.12_f32, 0.012_f32, 0.8_f32),
                (-1.4, 1.05, 0.008, 0.6),
            ] {
                let d = (v - (m * u + c)).abs() / w;
                if d < 1.0 {
                    a += k * (1.0 - d);
                }
            }
            // E o pó fino: um hash determinístico do pixel, esparso.
            let h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
            let h = h ^ (h >> 13);
            if h.is_multiple_of(211) {
                a += 0.7;
            }
            lum[y * n + x] = a.min(1.0);
        }
    }
    // Para RGBA, com a cor a variar com a posição (âmbar em cima, azul em baixo).
    let mut out = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let a = lum[y * n + x];
            let t = y as f32 / n as f32;
            let (r, g, b) = (
                a * (1.0 - 0.35 * t),
                a * (0.85 - 0.10 * t),
                a * (0.55 + 0.45 * t),
            );
            let px = (y * n + x) * 4;
            let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
            out[px] = q(r);
            out[px + 1] = q(g);
            out[px + 2] = q(b);
            out[px + 3] = 255;
        }
    }
    out
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .ok()
}

/// O grafo: um campo de peças brilhantes → **Glow** com a máscara escolhida → saída.
///
/// ⚠️ **O `threshold` desce a `0,25` de propósito.** «Emitir» é ter cor acima do branco, e as
/// peças de uma grade nascem em `1,0` — com o limiar de fábrica (`1,0`) nada acenderia e o
/// smoke mostraria um ecrã limpo sobre produto correcto.
fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: 0.0 });
    g.set_param(grid, "rows", FIELD);
    g.set_param(grid, "cols", FIELD);
    // ⚠️ `gap_x`/`gap_y` — os nomes que o manifesto DECLARA. A 1.ª versão escreveu
    // `spacing_x`/`spacing_y`, que é um `set_param` **silenciosamente inerte**: a cena montava,
    // o campo saía com o espaçamento de fábrica e nada em lado nenhum dizia porquê. O gate
    // `the_scene_only_authors_params_the_manifests_declare` existe por causa desta linha.
    g.set_param(grid, "gap_x", 0.62);
    g.set_param(grid, "gap_y", 0.62);

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 180.0, y: 0.0 });
    g.set_param(size, "amount", 0.16);
    wire(g, grid, size)?;

    let glow = g.add_node("fx.glow");
    g.set_pos(glow, Pos { x: 360.0, y: 0.0 });
    g.set_param(glow, "threshold", 0.25);
    g.set_param(glow, "knee", 0.35);
    g.set_param(glow, "intensity", 2.4);
    g.set_param(glow, "radius", 1.6);
    g.set_param(
        glow,
        ph2d_node_fx_glow::dirt::DIRT_INTENSITY,
        DIRT_INTENSITY,
    );
    g.set_text_param(
        glow,
        ph2d_node_fx_glow::dirt::DIRT_KEY,
        DIRT_NAME.to_string(),
    );
    g.set_label(glow, "Glow + Lens Dirt");
    wire(g, size, glow)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 540.0, y: 0.0 });
    wire(g, glow, out)?;
    Some(out)
}

/// Monta a cena. Devolve `false` quando o átlas recusou a imagem (e aí nada é montado — uma
/// cena com o grafo e sem a sprite ensinaria o defeito que ela existe para mostrar).
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    motion: &mut crate::motion_state::MotionState,
) -> bool {
    let pixels = dirt_pixels();
    let cell = *next_cell;
    let asset_id = asset_db.insert_image_rgba8(DIRT_PX, DIRT_PX, pixels.clone());
    atlas_asset_map.insert(cell, asset_id);
    let fetch = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        asset_db
            .get(aid)?
            .image_rgba8()
            .map(|(_, _, p)| p.into_owned())
    };
    if renderer
        .insert_atlas_sprite_with_regrow(cell, DIRT_PX, DIRT_PX, &pixels, fetch)
        .is_err()
    {
        atlas_asset_map.remove(&cell);
        eprintln!("[glow dirt smoke] o atlas recusou a imagem -- a cena NAO foi montada");
        return false;
    }
    *next_cell += 1;
    // A sprite fica no canto, pequena: ela é a LEGENDA (é o padrão que se vai reconhecer no
    // halo), não parte do efeito.
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(-3.4, 2.1)),
        Sprite::atlas(cell, [1.1, 1.1], [1.0, 1.0, 1.0, 1.0]),
        Name::new(DIRT_NAME),
    ));
    let Some(out) = build(&mut motion.doc.graph) else {
        eprintln!("[glow dirt smoke] o grafo nao montou");
        return false;
    };
    motion.sinks.push(out);
    eprintln!(
        "[glow dirt smoke] A SUJIDADE NA LENTE.

  No canto de cima-esquerda esta' a IMAGEM (a sprite «{DIRT_NAME}»): borroes, dois riscos
  e po' fino, sobre preto. No meio esta' um campo de {n}x{n} pecas a brilhar.

  O QUE TEM DE ACONTECER: o brilho NAO e' parejo. Ele acende em manchas com a forma
  dos borroes da imagem, e ha' duas riscas atravessadas. As manchas de cima puxam para
  o ambar e as de baixo para o azul -- a sujidade acrescenta COR, nao so' brilho.

  QUER MEXER? Clique no no' «Glow + Lens Dirt» no grafo e procure, no fim do painel:
    · «Dirt Intensity» -- arraste para 0: o brilho volta a ficar parejo. Esse e' o
      antes. Suba de novo: as manchas voltam.
    · «Dirt Texture» -- apague o nome: as manchas somem (e nao volta a aparecer ate'
      escrever «{DIRT_NAME}» outra vez). ⚠️ Repare que a linha «Dirt Intensity» SOME
      junto: sem imagem escolhida ela nao faz nada, entao ela nao e' pintada.

  DEU ERRADO se: o brilho ficar parejo com «Dirt Intensity» em {DIRT_INTENSITY}; se as
  manchas nao mudarem ao arrastar aquele numero; se elas ficarem ESTICADAS ao
  redimensionar a janela (a imagem e' quadrada e tem de continuar quadrada);
  ou se apagar o nome nao apagar as manchas.",
        n = FIELD as u32,
    );
    true
}

#[cfg(test)]
#[path = "glow_dirt_smoke_tests.rs"]
mod tests;
