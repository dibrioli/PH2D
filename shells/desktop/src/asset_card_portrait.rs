//! ⭐⭐⭐ **O RETRATO de um Prefab** — as peças dele, compostas (plano `docs/Components/07`, A6).
//!
//! # ⛔⛔ O que havia antes, e por que não chegava
//!
//! O cartão de um Prefab mostrava a miniatura da **peça maior** dele. Num prefab de uma peça — o
//! caso comum — isso *é* o prefab, e ficava exacto. Num de várias, o artista via uma peça e o
//! rótulo *«N pieces»* ao lado: honesto, e insuficiente para reconhecer o objecto.
//!
//! # ⭐ O bloqueio que o código nomeava DISSOLVEU-SE, e a razão é onde os pixels vivem
//!
//! A nota do construtor dizia: *«o retrato a sério é um render offscreen da sub-árvore, e ele está
//! BLOQUEADO — esta função corre sem `gpu`, sem `renderer` e sem `vello_pass` em mãos»*. Verdade
//! sobre o render, e **irrelevante para o retrato**: as peças de um prefab são sprites, e os
//! pixels delas já estão descodificados e **já estão reduzidos em memória**, porque a miniatura
//! por textura os pôs lá. ⇒ o retrato **compõe-se**, e não se renderiza.
//!
//! ⭐⭐ E isso paga-se duas vezes: nenhuma descodificação nova (a composição lê as miniaturas que
//! a cache já tem) e nenhum contacto com o atlas do `vello` — que é onde a nota do plano avisava
//! que *«quem recozinha pixels tem de marcar a textura suja, senão a imagem congela em silêncio»*.
//!
//! # ⚠️ O que ele NÃO desenha, declarado
//!
//! | fora | porquê |
//! |---|---|
//! | a ordem de **z** (`OrderInLayer`, `YSort`, `SortingGroup`) | ele usa a ordem do DOCUMENTO; honrar o z pede a lei do extract, que vive noutra fase |
//! | modos de mistura e `tint` | a composição é `src-over` reta; um modo por peça pede o vocabulário do renderer |
//! | peças que não são sprite (vetor, texto, campo 3D) | elas não têm pixels em memória — e uma peça invisível no retrato é melhor que um retrato que mente sobre ter tudo |
//!
//! *Um retrato parcial com os limites escritos é outra coisa que um retrato que se diz completo.*

use ph2d_asset_index::Thumb;
use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_render::Sprite;

/// O lado do retrato, em px. O mesmo tecto das miniaturas por textura — é o mesmo cartão.
const SIDE: u32 = crate::thumbnail::THUMB_MAX;

/// Uma peça pronta a compor: a miniatura dela, onde ela cai, e **que pedaço da imagem** ela mostra.
struct Placed {
    thumb: Thumb,
    /// Centro do QUAD em espaço local da receita — já com o pivô (`resolve_anchor`) somado.
    center: [f32; 2],
    /// Meia-extensão do quad, em espaço local.
    half: [f32; 2],
    rotation: f32,
    /// ⭐⭐⭐ **A janela da imagem que esta peça mostra** — `[u0, v0, u1, v1]`.
    ///
    /// Report do Enio (2026-09-01, 2.ª foto): *«não ficou idêntico»* — as peças saíam estreitas.
    /// A causa: uma sprite pode mostrar **um pedaço** da sua imagem (uma célula de folha, um
    /// sub-rectângulo de região), e o retrato desenhava a imagem INTEIRA no lugar do pedaço.
    uv: [f32; 4],
    /// Espelhamento autorado, `(x, y)`.
    flip: (bool, bool),
}

/// ⭐⭐⭐ **Compõe o retrato**, ou `None` quando não há peça com pixels.
///
/// `piece_art` entrega a miniatura **de uma PEÇA** — é a porta da cache, e é o que garante que
/// nenhuma descodificação nova acontece aqui.
///
/// # ⛔⛔⛔ Ela recebe a ENTIDADE, e não a textura — a 1.ª versão custou um report
///
/// Report do Enio (2026-09-01, com foto): um prefab de 4 peças ficou com o **cartão cinzento** —
/// nem retrato, nem cor. A causa: eu lia `SpritePixels` para achar a textura de uma peça, e esse
/// carimbo é o da **MINORIA**. O caminho normal de todo import e de todo canvas novo é o **átlas**
/// (`Sprite { source: Atlas { key } }` + `atlas_asset_map`), e o objecto dele era exactamente
/// isso: um objecto vazio com três texturas por baixo.
///
/// ⚠️ **O ficheiro vizinho já tinha pago isto duas vezes** e escreveu-o no doc do
/// [`crate::asset_index_build::texture_of`]: *«UMA porta, dois leitores»*. Eu fiz-me o **terceiro
/// leitor, com a metade errada** — e o sintoma foi o mesmo dos dois anteriores.
///
/// ⇒ o retrato deixou de saber **como** uma peça aponta para os pixels. Quem resolve é o chamador,
/// pela porta única; uma terceira forma amanhã não volta a passar por aqui.
///
/// ⚠️ **`None` é a resposta certa para um prefab sem sprites**, e o cartão fica com a cor
/// dominante: *ele não tem retrato*, e inventar um cinzento seria dizer que tem.
pub(crate) fn compose(
    sim: &SimWorld,
    root: Entity,
    pieces: &[Entity],
    // Pixels por metro do projecto — o `resolve_anchor` converte o `offset` autorado com ele.
    ppm: f32,
    mut piece_art: impl FnMut(Entity) -> Option<Thumb>,
) -> Option<Thumb> {
    let root_xf = ph2d_ecs::world_transform(sim.world(), root)?;
    let mut placed: Vec<Placed> = Vec::new();
    // ⚠️ **Ordem do DOCUMENTO, estabilizada pelo `StableId`** — sem uma ordem total o retrato
    // trocava as peças de camada entre quadros ao sabor da ordem de arquétipo, e um cartão que
    // pisca é pior que um cartão parcial.
    let mut ordered: Vec<Entity> = pieces.to_vec();
    ordered.sort_by_key(|e| sim.world().get::<StableId>(*e).map_or(u64::MAX, |s| s.0));
    for &p in &ordered {
        let Some(sprite) = sim.world().get::<Sprite>(p) else {
            continue;
        };
        let Some(thumb) = piece_art(p) else {
            continue;
        };
        let Some(xf) = ph2d_ecs::world_transform(sim.world(), p) else {
            continue;
        };
        // Posição **relativa à receita**: o mundo dela é irrelevante (uma receita está escondida,
        // e pode estar em qualquer sítio).
        let rel = local_of(&root_xf, &xf);
        // ⭐⭐⭐ **O PIVÔ, pela porta partilhada** (`Sprite::resolve_anchor`) — o quad não está
        // centrado na translação quando a sprite não é `centered` ou tem `offset`/`anchor`.
        // *A 1.ª versão usava a translação e o objecto saía deslocado.*
        let a = sprite.resolve_anchor(ppm);
        let (sin, cos) = rel.rotation.sin_cos();
        let (ax, ay) = (a[0] * rel.scale.x, a[1] * rel.scale.y);
        let uv = uv_window(sim, p, &thumb);
        placed.push(Placed {
            thumb,
            center: [
                rel.translation.x + ax * cos - ay * sin,
                rel.translation.y + ax * sin + ay * cos,
            ],
            half: [
                (sprite.size[0] * rel.scale.x).abs() * 0.5,
                (sprite.size[1] * rel.scale.y).abs() * 0.5,
            ],
            rotation: rel.rotation,
            // ⭐⭐⭐ **A JANELA, pelas MESMAS funções que a tela usa** — região e depois célula,
            // exactamente na ordem do `sim_extract`. ⚠️ Aqui a base é o rectângulo UNITÁRIO e não
            // a UV do átlas, porque a miniatura em mãos **é a imagem de origem**, não a página.
            uv,
            flip: (sprite.flip_x, sprite.flip_y),
        });
    }
    if placed.is_empty() {
        return None;
    }
    let (min, max) = bounds(&placed);
    let span = [
        (max[0] - min[0]).max(f32::EPSILON),
        (max[1] - min[1]).max(f32::EPSILON),
    ];
    // ⚠️ **Escala UNIFORME**, do lado mais longo: esticar por eixo deformaria o objecto que o
    // artista está a tentar reconhecer, que é a única coisa que um retrato tem de fazer.
    let s = SIDE as f32 / span[0].max(span[1]);
    let (w, h) = (
        ((span[0] * s).round() as u32).clamp(1, SIDE),
        ((span[1] * s).round() as u32).clamp(1, SIDE),
    );
    // ⚠️ **Acumula em PRÉ-MULTIPLICADO** — compor cor reta sobre alfa parcial escurece a borda,
    // que é a armadilha que a redução por caixa já paga no irmão `thumbnail::reduce`.
    let mut acc = vec![0f32; (w * h * 4) as usize];
    for pl in &placed {
        blit(&mut acc, w, h, pl, min, s);
    }
    let mut out = vec![0u8; (w * h * 4) as usize];
    for (o, a) in out
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(acc.as_chunks::<4>().0)
    {
        let al = a[3].clamp(0.0, 1.0);
        let un = if al > 0.0 { 1.0 / al } else { 0.0 };
        for c in 0..3 {
            o[c] = ((a[c] * un).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        o[3] = (al * 255.0).round() as u8;
    }
    Some(Thumb {
        rgba: std::sync::Arc::new(out),
        w,
        h,
    })
}

/// ⭐⭐⭐ **Que pedaço da imagem esta peça mostra** — `[u0, v0, u1, v1]`.
///
/// ⚠️ **Pelas MESMAS funções que o `sim_extract` usa, e na MESMA ordem** (região, depois célula da
/// folha). Reescrever a conta aqui seria a quarta vez que este trabalho re-deriva uma lei que já
/// tem porta — e as três anteriores custaram um report cada.
fn uv_window(sim: &SimWorld, p: Entity, thumb: &Thumb) -> [f32; 4] {
    let unit = [0.0, 0.0, 1.0, 1.0];
    let after_region = match sim.world().get::<ph2d_ecs::SpriteRegion>(p) {
        // ⚠️ **Sem `filter_clip`**: o meio-texel existe para o sampler bilinear não sangrar do
        // átlas vizinho, e aqui a fonte é a imagem inteira — não há vizinho de onde sangrar.
        Some(r) => crate::render_loop::sim_extract::region_subrect(
            unit,
            r.rect,
            f32::from(u16::try_from(thumb.w).unwrap_or(u16::MAX)),
            f32::from(u16::try_from(thumb.h).unwrap_or(u16::MAX)),
            None,
        ),
        None => unit,
    };
    match sim.world().get::<ph2d_ecs::SpriteGrid>(p) {
        Some(g) => crate::render_loop::sim_extract::sprite_sheet_subrect(
            after_region,
            g.hframes,
            g.vframes,
            g.frame,
        ),
        None => after_region,
    }
}

/// A pose de `child` **relativa** a `parent` — só o que o retrato usa.
///
/// ⚠️ Ela não é a inversa completa (sem skew): o retrato compõe quads eixo-alinhados rodados, e um
/// skew entraria como uma mentira silenciosa. *Declarado no cabeçalho.*
fn local_of(parent: &Transform, child: &Transform) -> Transform {
    let inv_s = [
        if parent.scale.x.abs() > f32::EPSILON {
            1.0 / parent.scale.x
        } else {
            0.0
        },
        if parent.scale.y.abs() > f32::EPSILON {
            1.0 / parent.scale.y
        } else {
            0.0
        },
    ];
    let d = child.translation - parent.translation;
    let (sin, cos) = (-parent.rotation).sin_cos();
    Transform {
        translation: ph2d_core::Vec2::new(
            (d.x * cos - d.y * sin) * inv_s[0],
            (d.x * sin + d.y * cos) * inv_s[1],
        ),
        rotation: child.rotation - parent.rotation,
        scale: ph2d_core::Vec2::new(child.scale.x * inv_s[0], child.scale.y * inv_s[1]),
        ..Transform::IDENTITY
    }
}

/// A caixa que envolve todas as peças, já com a rotação de cada uma contada.
fn bounds(placed: &[Placed]) -> ([f32; 2], [f32; 2]) {
    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in placed {
        let (sin, cos) = p.rotation.sin_cos();
        // A meia-extensão de um quad rodado é a soma das projecções dos dois semi-eixos.
        let ex = p.half[0] * cos.abs() + p.half[1] * sin.abs();
        let ey = p.half[0] * sin.abs() + p.half[1] * cos.abs();
        min[0] = min[0].min(p.center[0] - ex);
        min[1] = min[1].min(p.center[1] - ey);
        max[0] = max[0].max(p.center[0] + ex);
        max[1] = max[1].max(p.center[1] + ey);
    }
    (min, max)
}

/// Desenha uma peça no acumulador, amostrando a miniatura dela pela inversa da pose.
///
/// ⚠️ **A varredura é do DESTINO**, e não da fonte: percorrer a fonte deixaria buracos assim que a
/// peça fosse ampliada, e o retrato ficaria com falhas que ninguém saberia explicar.
fn blit(acc: &mut [f32], w: u32, h: u32, p: &Placed, min: [f32; 2], s: f32) {
    if p.thumb.w == 0 || p.thumb.h == 0 || p.half[0] <= 0.0 || p.half[1] <= 0.0 {
        return;
    }
    let (sin, cos) = (-p.rotation).sin_cos();
    for y in 0..h {
        for x in 0..w {
            // Pixel do retrato → espaço local da receita (o centro do pixel).
            let lx = min[0] + (x as f32 + 0.5) / s;
            // ⚠️ **O Y do retrato cresce para BAIXO e o do mundo para CIMA** — sem esta inversão o
            // objecto sai espelhado, e um retrato espelhado passa por bom até alguém o comparar.
            let ly = min[1] + (h as f32 - y as f32 - 0.5) / s;
            let (dx, dy) = (lx - p.center[0], ly - p.center[1]);
            let (rx, ry) = (dx * cos - dy * sin, dx * sin + dy * cos);
            let u = (rx / p.half[0] + 1.0) * 0.5;
            let v = (1.0 - (ry / p.half[1] + 1.0) * 0.5).clamp(0.0, 1.0);
            if !(0.0..=1.0).contains(&u) || !(0.0..1.0).contains(&(1.0 - v)) {
                continue;
            }
            // ⭐⭐ **O espelho autorado, e depois a JANELA** — nesta ordem: espelhar depois de
            // escolher a célula viraria a folha inteira em vez da célula.
            let u = if p.flip.0 { 1.0 - u } else { u };
            let v = if p.flip.1 { 1.0 - v } else { v };
            let [u0, v0, u1, v1] = p.uv;
            let (uu, vv) = (u0 + u * (u1 - u0), v0 + v * (v1 - v0));
            let sx = ((uu * p.thumb.w as f32) as u32).min(p.thumb.w - 1);
            let sy = ((vv * p.thumb.h as f32) as u32).min(p.thumb.h - 1);
            let si = ((sy * p.thumb.w + sx) * 4) as usize;
            let Some(src) = p.thumb.rgba.get(si..si + 4) else {
                continue;
            };
            let a = f32::from(src[3]) / 255.0;
            if a <= 0.0 {
                continue;
            }
            let di = ((y * w + x) * 4) as usize;
            let inv = 1.0 - a;
            for c in 0..3 {
                acc[di + c] = f32::from(src[c]) / 255.0 * a + acc[di + c] * inv;
            }
            acc[di + 3] = a + acc[di + 3] * inv;
        }
    }
}

#[cfg(test)]
#[path = "asset_card_portrait_tests.rs"]
mod tests;
