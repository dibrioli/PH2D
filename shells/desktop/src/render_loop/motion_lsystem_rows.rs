//! **AS LINHAS QUE UMA PLANTA PUBLICA** — a planta, as folhas, e em que MÉDIA cada uma desenha.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! responsabilidade: o irmão [`super::motion_lsystem_leaves`] responde *onde uma folha nasce e o
//! que o app diz quando ela não nasce*, e este *que linhas saem, com que cara, e em que ordem*.
//!
//! ⭐⭐⭐ **A TERCEIRA MÉDIA vive aqui** (2026-08-30): com o `Leaves In Front` acima de `0`, a
//! copa inteira deixa de ser desenhada no passe das sprites e passa à cena VECTORIAL, como quads
//! texturados — a mesma camada em que a planta vive, e é ali que a ordem das linhas manda.

use super::motion_lsystem_leaves::{Anchor, named_appearance};
use ph2d_nodegraph::attr::{Column, Stream};

/// **A planta MAIS as folhas, num stream só.**
///
/// ⚠️ **Mídia MISTA na mesma corrente, e o lowering já a sabe rotear:** uma linha com
/// `geometry_id > 0` vai ao passe VECTORIAL (a planta), e as outras são quads amostrados do
/// atlas (as folhas). Publicá-las em correntes separadas obrigaria o artista a juntá-las com um
/// `motion.combine` para as mover como uma planta só.
/// **Abaixo disto a folha não se desenha.** Não é um limiar de gosto: `1/256` de um quad é
/// menos de um pixel em qualquer zoom que o editor oferece, e o custo de uma linha é o mesmo
/// visível ou não. ⚠️ Ele **não** é um degrau visível — o peso passa por aqui a subir, e a
/// folha que ele esconde tem `size` abaixo de meio pixel.
pub(crate) const GROW_FLOOR: f32 = 1.0 / 256.0;

pub(crate) fn plant_and_leaves(
    origin: [f32; 2],
    handle: u32,
    anchors: &[Anchor],
    names: &[String; 3],
    cook: &ph2d_nodegraph::cook::Cook,
    look_law: LeafLook,
) -> Stream {
    let (front, keep_own_colour) = (look_law.front, look_law.keep_own_colour);
    let looks: Vec<_> = names.iter().map(|n| named_appearance(cook, n)).collect();
    // **Uma linha**, antes de saber em que ordem ela entra.
    struct Row {
        p: [f32; 2],
        size: [f32; 2],
        rot: f32,
        geom: f32,
        tint: [f32; 4],
        uv: [f32; 4],
        tex: f32,
        premul: f32,
        tint_mask: f32,
        /// ⭐⭐⭐ `1` = esta linha desenha-se no passe VECTORIAL, como quad texturado.
        vector_pass: f32,
    }
    let plant = Row {
        p: origin,
        size: [1.0, 1.0],
        rot: 0.0,
        geom: handle as f32,
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        tex: 0.0,
        premul: 0.0,
        tint_mask: 1.0,
        vector_pass: 0.0,
    };
    // ⭐⭐ **TRÊS BALDES, e a ordem entre eles É o z** — report do Enio (2026-08-30): *"não temos
    // a opção de escolher quantas folhas são desenhadas na frente ou atrás dos galhos"*.
    //
    // ⚠️ **A casa desenha os sprites ANTES do vector** (declarado em `mod.rs`: *«Fase 1: vector
    // over sprite»*), então uma folha-SPRITE fica sempre atrás da planta, e nenhuma ordem de
    // linhas a move. Uma folha-VECTOR vive na mesma passagem que a planta, e ali quem manda é a
    // ordem: as de trás vêm antes da linha da planta, as da frente depois.
    let (mut atras, mut frente, mut sprites) = (Vec::new(), Vec::new(), Vec::new());
    for a in anchors {
        let Some((sz, tn, rect, tid, pm, gid)) = looks[a.slot] else {
            continue;
        };
        // ⚠️ **A marca fechada não vira linha nenhuma** — um quad de tamanho `0` custaria o
        // mesmo que um visível, e a árvore de fábrica traz `62` marcas para `31` pontas.
        if a.grow <= GROW_FLOOR {
            continue;
        }
        // ⭐ **O tamanho final e os dois sorteios** (report do Enio, 2026-08-30).
        let (scale, shove) = look_law.at(a.seed);
        let sized = [sz[0] * a.grow * scale, sz[1] * a.grow * scale];
        let row = Row {
            // ⚠️ **O empurrão é em FRACÇÃO do tamanho da folha**, e não em unidades de mundo:
            // uma planta a `0,3` de passo e outra a `3` teriam de ser afinadas à mão, e o que
            // o artista quer dizer é *«desencostada do ramo por meia folha»*.
            p: [a.p[0] + shove[0] * sized[0], a.p[1] + shove[1] * sized[1]],
            size: sized,
            rot: a.rot,
            // ⚠️ `0` = SEM geometria vectorial ⇒ a linha vai pelo caminho do quad. É a mesma
            // convenção que o `source.object` usa para separar um vector vivo de uma sprite.
            geom: gid,
            tint: tn,
            uv: rect,
            tex: tid,
            premul: pm,
            // ⭐⭐ **A folha fora do TINT da árvore, e só do tint.**
            //
            // ⛔⛔ **A 1.ª cura escrevia `falloff` e PARTIU a planta** (report do Enio,
            // 2026-08-30: *"Keep own color não funciona, as folhas não aparecem"*): o
            // `falloff` é a máscara de TODOS os modificadores, e o `motion.move` faz
            // `P' = P + (dx, dy) · falloff` — as folhas ficavam PARADAS enquanto a planta se
            // movia, e a cena `=108` move cada coluna. *O canal que escolhi era muito mais
            // largo do que a pergunta que fiz.*
            tint_mask: f32::from(!keep_own_colour),
            // O braço abaixo decide-o; aqui a linha nasce sprite, como sempre foi.
            vector_pass: 0.0,
        };
        // ⭐⭐⭐ **A TERCEIRA MÉDIA, e é ela que faz o «à frente» valer para uma IMAGEM.**
        //
        // Report do Enio (2026-08-30, três vezes): *"Leaves in front ainda não funciona quando a
        // folha é IMG"*. A casa desenha os sprites no passe 1 (HDR) e o vector no passe 3 (a cena
        // Vello), então **todo vector fica por cima de todo sprite** — nenhuma ordem de linhas
        // move uma imagem para a frente de um galho.
        //
        // ⇒ com a fracção ligada, a folha-imagem passa a desenhar-se **na cena vectorial**, como
        // quad texturado (`vector_pass`), e ali a ordem manda.
        //
        // ⚠️ **E vão TODAS as folhas daquela planta, não só as da frente.** O tonemap desta casa
        // é passagem pura para 8 bits (há gate a medi-la byte-exacta), então mudar de passe não
        // muda um pixel — mas dividir a copa entre os dois passes deixaria as duas metades a
        // depender de um invariante que uma wave futura pode quebrar. *Uma copa, um caminho.*
        //
        // ⚠️ **Com `front = 0` nada disto acontece** e a corrente é byte-idêntica à de antes.
        let quad = gid <= 0.0 && front > 0.0;
        let row = Row {
            vector_pass: f32::from(quad),
            ..row
        };
        if gid <= 0.0 && !quad {
            sprites.push(row);
        } else if is_in_front(a.seed, front) {
            frente.push(row);
        } else {
            atras.push(row);
        }
    }
    let n = 1 + atras.len() + frente.len() + sprites.len();
    let mut p = Vec::with_capacity(n);
    let (mut size, mut rot, mut geom) = (Vec::new(), Vec::new(), Vec::new());
    let (mut tint, mut uv, mut tex) = (Vec::new(), Vec::new(), Vec::new());
    let (mut premul, mut tint_mask) = (Vec::new(), Vec::new());
    let mut vector_pass = Vec::new();
    for r in atras
        .into_iter()
        .chain(std::iter::once(plant))
        .chain(frente)
        .chain(sprites)
    {
        p.push(r.p);
        size.push(r.size);
        rot.push(r.rot);
        geom.push(r.geom);
        tint.push(r.tint);
        uv.push(r.uv);
        tex.push(r.tex);
        premul.push(r.premul);
        tint_mask.push(r.tint_mask);
        vector_pass.push(r.vector_pass);
    }
    // ⚠️ **A coluna só nasce quando alguma linha a usa** — ausente ⇒ toda linha é sprite, que é
    // a convenção de sempre ⇒ byte-idêntico.
    let alguma = vector_pass.iter().any(|v| *v > 0.5);
    let stream = Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("size", Column::Vec2(size))
        // ⛔ **`rot`, e não `rotation`** — é este o nome que a convenção de instâncias lê
        // (`ph2d-eval-motion`, em GRAUS). O outro era ignorado em silêncio.
        .with("rot", Column::Scalar(rot))
        .with("premultiplied", Column::Scalar(premul))
        .with("geometry_id", Column::Scalar(geom))
        .with("tint", Column::Vec4(tint))
        .with("uv_rect", Column::Vec4(uv))
        .with("texture_id", Column::Scalar(tex))
        .with(
            "Index",
            Column::Scalar((0..n).map(|i| i as f32).collect::<Vec<_>>()),
        )
        .with("Count", Column::Scalar(vec![n as f32; n]));
    // ⚠️ **A coluna só nasce quando ela DIZ alguma coisa.** Uma coluna de uns responderia a uma
    // pergunta que ninguém fez e apagaria uma máscara que um nó a montante tivesse escrito —
    // ausente ⇒ `1` em toda a casa ⇒ byte-idêntico ao que havia antes deste param.
    let stream = if alguma {
        stream.with(
            ph2d_eval_motion::VECTOR_PASS_COLUMN,
            Column::Scalar(vector_pass),
        )
    } else {
        stream
    };
    if keep_own_colour {
        stream.with(
            ph2d_nodegraph::attr::TINT_MASK_COLUMN,
            Column::Scalar(tint_mask),
        )
    } else {
        stream
    }
}

/// **Esta folha vai à FRENTE?** — a fracção do painel, resolvida por marca e sem estado.
///
/// ⚠️ **Determinística e ESTÁVEL**: o sorteio é do índice da âncora, não de um contador de
/// linhas emitidas — senão uma folha que fechasse (peso `0`) reordenaria todas as outras entre
/// a frente e o fundo, e a árvore piscaria enquanto cresce.
///
/// ⚠️ `0` e `1` são exactos nas duas pontas: `hash ∈ [0, 1)`, logo `< 0` nunca e `< 1` sempre.
fn is_in_front(seed: u32, front: f32) -> bool {
    hash01(seed) < front
}

/// **O TAMANHO E O EMPURRÃO de cada folha** — o que o painel pede, resolvido por marca.
///
/// ⛔ Report do Enio (2026-08-30): *"não temos parâmetros para o tamanho final da folha nem
/// jitter de scale e posição"*.
///
/// ⚠️ **Os três são NEUTROS no default** (`1`, `0`, `0`), e o neutro é exacto: um `× 1.0` é a
/// identidade em `f32` e um sorteio de amplitude `0` nem é avaliado — o caminho de omissão é
/// byte a byte o que shipou antes deles.
#[derive(Clone, Copy)]
pub(crate) struct LeafLook {
    /// A fracção desenhada à frente dos galhos.
    ///
    /// ⚠️ **Ela mora aqui e não num argumento à parte** porque o clippy tem razão: sete
    /// argumentos posicionais já são um em que ninguém confia. Um `bool` e três `f32` seguidos
    /// numa chamada é uma troca à espera de acontecer.
    pub(crate) front: f32,
    /// `true` = as folhas mantêm a cor delas (os efeitos a jusante não as alcançam).
    pub(crate) keep_own_colour: bool,
    pub(crate) size: f32,
    pub(crate) size_jitter: f32,
    pub(crate) pos_jitter: f32,
}

impl LeafLook {
    /// `(multiplicador de tamanho, empurrão em fracções do tamanho)` para a marca `i`.
    ///
    /// ⚠️ **Três LANES do mesmo hash, e não três chamadas iguais:** com uma lane só, o tamanho
    /// e o empurrão de uma folha seriam o MESMO número — as maiores todas para o mesmo lado,
    /// que é um padrão visível e não um sorteio.
    pub(crate) fn at(self, i: u32) -> (f32, [f32; 2]) {
        let scale = if self.size_jitter == 0.0 {
            self.size
        } else {
            // `±jitter/2` em torno de `1`, logo `jitter = 1` dá de metade ao dobro.
            self.size * (1.0 + (hash01_lane(i, 1) - 0.5) * self.size_jitter)
        };
        let shove = if self.pos_jitter == 0.0 {
            [0.0, 0.0]
        } else {
            [
                (hash01_lane(i, 2) - 0.5) * self.pos_jitter,
                (hash01_lane(i, 3) - 0.5) * self.pos_jitter,
            ]
        };
        (scale, shove)
    }
}

/// `[0, 1)` a partir de um índice — o mesmo avalanche splitmix que o resto da casa usa.
fn hash01(i: u32) -> f32 {
    hash01_lane(i, 0)
}

/// O mesmo, numa LANE — sorteios distintos para perguntas distintas sobre a mesma marca.
fn hash01_lane(i: u32, lane: u32) -> f32 {
    let mut h = i
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(lane.wrapping_mul(0xc2b2_ae35))
        .wrapping_add(0x1eaf_1eaf);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
#[path = "motion_lsystem_front_tests.rs"]
mod tests;
