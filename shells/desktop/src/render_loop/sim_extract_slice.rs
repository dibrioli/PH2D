//! **A emissão do 9-slice** — irmão do [`super::sim_extract`], que está no tecto de LOC.
//!
//! O extract emite UMA `RenderInstance` por sprite. Um sprite com 9-slice emite até **nove**, e
//! este módulo é a costura entre a geometria pura ([`ph2d_render::nine_slice`]) e a instância
//! que a GPU consome.
//!
//! # Os nove partilham o `SimRef`, e isso é a decisão
//!
//! Cada quad é uma entidade de PRESENTE própria, mas todas carregam o **mesmo** `SimRef`. É isso
//! que faz o passe pós-caminhada — o que carimba `z_order` e o agrupamento de clip — servir os
//! nove de uma vez, sem saber que existe 9-slice: ele já resolve *por entidade de simulação*.
//! Dar-lhes `SimRef` diferentes obrigaria esse passe a aprender uma segunda regra.
//!
//! ⚠️ Os oito extra levam [`SlicePatchMirror`] para que o HUD não os conte como entidades. A
//! contagem de INSTÂNCIAS sobe de propósito — nove quads são nove quads, e essa é a verdade que
//! um contador de desenho deve dizer.

use ph2d_render::nine_slice::{PATCH_COUNT, SlicePatch, nine_slice_patches};
use ph2d_render::{RenderInstance, Sprite};

/// **Onde os pixels deste sprite estão na FONTE** — os três que o corte lê juntos.
///
/// ⚠️ Eles andam sempre em trio (a região recorta a imagem, a grelha recorta a célula, as
/// dimensões dizem em que escala isso é medido), e um deles sozinho não responde nada. Ter um tipo
/// é o que impede a próxima assinatura de os separar — e foi o que o teto de argumentos apanhou.
#[derive(Copy, Clone, Debug)]
pub(super) struct SliceSource {
    pub region: Option<ph2d_ecs::SpriteRegion>,
    pub grid: ph2d_ecs::SpriteGrid,
    pub src_dims: Option<(u32, u32)>,
}

/// Dimensões, em pixels da fonte, do rect que o `atlas_uv` **já recortado** cobre.
///
/// ⚠️ Tem de ser o rect FINAL (região aplicada, célula da folha aplicada), não a imagem inteira:
/// as bordas do 9-slice são medidas na imagem que o artista vê no Inspector. Medir na textura
/// completa faria a borda de um sprite numa folha 4×2 sair **oito vezes** maior do que se pediu.
pub(super) fn sub_rect_source_px(src: SliceSource) -> Option<[f32; 2]> {
    let SliceSource {
        region,
        grid,
        src_dims,
    } = src;
    let (sw, sh) = src_dims?;
    // ⭐⭐ **A conta DESCEU para o motor em 2026-09-17** ([`ph2d_render::SourceCells`]), quando ela
    // ganhou um terceiro leitor de outra crate: quem prende a imagem ao esqueleto tem de traçar a
    // malha sobre **a mesma** célula. *Uma lei que três famílias usam desce para o motor que a
    // executa* — e ficar aqui uma segunda cópia poria a borda do 9-slice numa célula e a malha
    // noutra, sem um erro de compilação a dizê-lo.
    ph2d_render::SourceCells::of([sw, sh], region.map(|r| r.rect), grid.hframes, grid.vframes)
        .map(|c| c.cell)
}

/// Os quads deste sprite, ou `None` quando ele não é um 9-slice desenhável — e aí o chamador
/// segue o caminho de sempre, byte-idêntico.
pub(super) fn patches_for(
    slice: Option<&ph2d_ecs::SliceNine>,
    spr: &Sprite,
    src: SliceSource,
    atlas_uv: [f32; 4],
    pixels_per_meter: f32,
    basis: [f32; 4],
) -> Option<[Option<SlicePatch>; PATCH_COUNT]> {
    let slice = slice?;
    if !slice.draw_mode.is_nine() {
        return None;
    }
    let src_px = sub_rect_source_px(src)?;
    let patches = nine_slice_patches(
        atlas_uv,
        src_px,
        slice,
        spr.size,
        pixels_per_meter,
        scale_of(basis),
    );
    // Um 9-slice que não produziu quad nenhum (bordas maiores que o alvo nos dois eixos, tudo
    // Blank) NÃO pode devolver `Some([None; 9])`: isso faria o sprite desaparecer em silêncio.
    // Devolver `None` fá-lo cair no caminho normal — visível, e o artista vê o que fez.
    patches.iter().any(Option::is_some).then_some(patches)
}

/// A escala do mundo, tirada da `basis` 2×2 (colunas `[col0, col1]`).
///
/// ⚠️ **É o COMPRIMENTO de cada coluna**, não o elemento da diagonal: sob rotação a diagonal
/// deixa de ser a escala (uma rotação de 90° põe zeros lá) enquanto o comprimento da coluna se
/// mantém. Foi por ignorar a escala inteira que os cantos esticavam (smoke do Enio, 2026-08-22).
pub(super) fn scale_of(basis: [f32; 4]) -> [f32; 2] {
    [basis[0].hypot(basis[1]), basis[2].hypot(basis[3])]
}

/// A instância de UM quad, derivada da instância base do sprite.
///
/// Herda tudo — tinta, opacidade, base, ordenação, amostragem — e substitui só o que o quad
/// muda: o sub-rect, o tamanho, o centro e a repetição.
pub(super) fn apply_patch(base: &RenderInstance, p: &SlicePatch) -> RenderInstance {
    let mut ri = *base;
    ri.atlas_uv = p.uv;
    ri.size = p.size;
    // ⚠️ **O FLIP DO SPRITE espelha a GRELHA, não cada célula no seu lugar** (auditoria de fecho,
    // 2026-08-22 — defeito que nenhum smoke reportou).
    //
    // O `flip_x` da sprite é um bit na instância, e o shader inverte o `quv` de **cada** quad.
    // Com um quad só isso é o espelho certo; com nove, inverte o conteúdo de cada célula *dentro
    // dela própria* e deixa-a onde estava — o canto de cima-esquerda fica em cima-à-esquerda com
    // o arco virado ao contrário, e a moldura sai partida.
    //
    // A metade que faltava é geométrica: **negar o deslocamento**. O conteúdo já vem invertido
    // pelo bit da sprite; pôr cada célula do outro lado completa o espelho do sprite inteiro.
    //
    // ⚠️ Lê-se o `base.flip_uv` **antes** do XOR do quad, mais abaixo: o que espelha a grelha é o
    // flip da SPRITE, não a correção de paridade que a célula possa trazer.
    let mirror = [
        base.flip_uv & RenderInstance::FLIP_X_BIT != 0,
        base.flip_uv & RenderInstance::FLIP_Y_BIT != 0,
    ];
    // O `anchor` É o centro do quad em metros locais (`local = anchor + quad_pos * size` no
    // shader), por isso deslocar o quad é somar — e somar ao anchor do sprite preserva o pivô
    // autorado (centered/offset) por baixo do 9-slice.
    ri.anchor = [
        base.anchor[0]
            + if mirror[0] {
                -p.center_offset[0]
            } else {
                p.center_offset[0]
            },
        base.anchor[1]
            + if mirror[1] {
                -p.center_offset[1]
            } else {
                p.center_offset[1]
            },
    ];
    ri.uv_xform = p.uv_xform;
    // ⚠️ **XOR, nunca OR.** O sprite pode já estar espelhado, e o quad **troca** esse estado em
    // vez de o impor: dois espelhos seguidos são a orientação original, e um OR faria a borda
    // de um sprite invertido apontar para o lado errado. Os bits vêm do empacotador, não de
    // literais — quem sabe onde `flip_x`/`flip_y` moram é a `RenderInstance`.
    if p.flip[0] || p.flip[1] {
        ri.flip_uv ^= RenderInstance::pack_flip_flags(p.flip[0], p.flip[1], false);
    }
    if let Some(tag) = p.repeat_tag {
        // Limpa os bits de repeat do nó antes de pôr os do quad: um OR simples deixaria o modo
        // herdado misturado com o pedido, e `Disabled | Mirror` não é nenhum dos dois.
        let mask = !(0b11u32 << RenderInstance::REPEAT_SHIFT);
        ri.flip_uv = (ri.flip_uv & mask) | RenderInstance::pack_repeat_bits(tag);
    }
    ri
}

/// As instâncias deste sprite, em ordem: a `[0]` vai para o espelho principal da entidade e as
/// restantes nascem como quads extra.
///
/// ⚠️ **Existe como função PURA de propósito.** A alternativa era fazer o fan-out inline no
/// extract, onde ele é inalcançável por teste nenhum: o `ph2d-host-desktop` é um binário, o
/// `run()` precisa de um renderer, e a costura ficaria provada só por inspeção — que é
/// exatamente a classe de erro que a DIRETIVA §2 descreve («faltar uma ponta = clique dropado em
/// silêncio, não erro de compilação»). Devolve um array fixo + contagem: zero alocação (HR-3).
pub(super) fn instances(
    base: &RenderInstance,
    patches: &[Option<SlicePatch>; PATCH_COUNT],
) -> ([RenderInstance; PATCH_COUNT], usize) {
    let mut out = [*base; PATCH_COUNT];
    let mut n = 0;
    for q in patches.iter().flatten() {
        out[n] = apply_patch(base, q);
        n += 1;
    }
    (out, n)
}

/// ⭐⭐⭐ **QUE FRACÇÃO DA CÉLULA cada quad mostra** — na MESMA ordem das [`instances`].
///
/// É o que uma imagem presa ao esqueleto precisa para cortar a malha no pedaço certo
/// ([`ph2d_render::nine_slice::SlicePatchSource`]): o extract é o dono da cadeia
/// região → célula → fatia, e publica o resultado em vez de a obrigar a redescobri-la.
///
/// `cell_uv` é o `atlas_uv` **da célula** — o mesmo que entrou no [`patches_for`] —, e é exactamente
/// o rectângulo sobre o qual a malha do bind foi traçada.
///
/// ⚠️ **Uma célula de lado nulo devolve a fracção INTEIRA** em vez de dividir por zero: ali não há
/// como localizar o pedaço, e a fracção inteira é o valor que faz a deformação cair no caminho de
/// sempre (a malha toda) em vez de desaparecer.
pub(super) fn sources(
    cell_uv: [f32; 4],
    patches: &[Option<SlicePatch>; PATCH_COUNT],
) -> (
    [ph2d_render::nine_slice::SlicePatchSource; PATCH_COUNT],
    usize,
) {
    let inteira = ph2d_render::nine_slice::SlicePatchSource {
        frac: [0.0, 0.0, 1.0, 1.0],
    };
    let mut out = [inteira; PATCH_COUNT];
    let (du, dv) = (cell_uv[2] - cell_uv[0], cell_uv[3] - cell_uv[1]);
    let mut n = 0;
    for q in patches.iter().flatten() {
        if du != 0.0 && dv != 0.0 {
            out[n] = ph2d_render::nine_slice::SlicePatchSource {
                frac: [
                    (q.uv[0] - cell_uv[0]) / du,
                    (q.uv[1] - cell_uv[1]) / dv,
                    (q.uv[2] - cell_uv[0]) / du,
                    (q.uv[3] - cell_uv[1]) / dv,
                ],
            };
        }
        n += 1;
    }
    (out, n)
}

#[cfg(test)]
#[path = "sim_extract_slice_tests.rs"]
mod tests;
