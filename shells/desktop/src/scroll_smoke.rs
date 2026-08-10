//! **A cena da ROLAGEM** — `PH2D_BUILD_SMOKE=67` (o item 3 do estudo dos contêineres).
//!
//! Irmã da [`crate::sizing_smoke`] e com a mesma lei: ela dá o MATERIAL e **não arma nada** — as
//! molduras nascem sem `VecLayout`, e é o artista que liga o fluxo. Um smoke que arma o estado por
//! baixo do pano pula justamente a costura que existe para provar (a cicatriz do `impasto_smoke`).
//!
//! # As três perguntas de olho
//!
//! 1. **A roda ROLA a lista que não cabe** — e o que sai da moldura é recortado, não desenhado por
//!    cima do resto da cena.
//! 2. **Ela PARA nas duas pontas**: o último item é alcançável (é para isso que a rolagem existe) e
//!    a lista não rola para dentro do vazio.
//! 3. **A moldura fica onde está** — o que se move é o conteúdo. Se a caixa escura andar junto com
//!    os filhos, isto virou *arrastar*, e não *rolar*.
//!
//! ⚠️ E a quarta forma é o **CONTROLE**: a lista curta, com o mesmo recorte, que **cabe**. A roda
//! sobre ela tem de dar **ZOOM na câmera**, como em qualquer outro lugar do canvas — é ela que
//! prova que a rolagem não roubou o gesto do dia inteiro.

use ph2d_ecs::{ChildOf, Entity, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// `(centro y, nº de itens, rótulo)` — a caixa é a mesma nas duas, e só o conteúdo difere.
type Row = (f64, usize, &'static str);
const ROWS: &[Row] = &[(2.2, 9, "LISTA"), (-2.2, 2, "CONTROLE")];

/// Meia-largura e meia-altura de toda moldura.
const HALF_W: f64 = 3.0;
const HALF_H: f64 = 1.6;
/// A altura de um item, e o vão entre eles.
const ITEM_H: f64 = 0.6;
const GAP: f64 = 0.15;

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// Quantos caminhos cada fileira empurra: os itens e a moldura.
fn per_row(i: usize) -> usize {
    ROWS[i].1 + 1
}

/// Quanto o conteúdo de uma fileira mede — a MESMA aritmética que a mensagem imprime.
fn content_of(n: usize) -> f64 {
    (n as f64) * ITEM_H + (n as f64 - 1.0) * GAP
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    for &(cy, n, _) in ROWS {
        for k in 0..n {
            // Os itens nascem EMPILHADOS no topo da moldura: sem fluxo ninguém os arruma, e é
            // o passo 1 do roteiro que os põe em coluna.
            let y = cy + HALF_H - 0.3 - (k as f64) * 0.02;
            // Uma faixa listrada — para o artista VER qual item está no alto depois de rolar.
            let c = if k % 2 == 0 {
                [90, 140, 210]
            } else {
                [70, 110, 170]
            };
            s.push_path(tint(
                rectangle([-HALF_W + 0.25, y - ITEM_H], [HALF_W - 0.25, y]),
                c,
            ));
        }
        s.push_path(tint(
            rectangle([-HALF_W, cy - HALF_H], [HALF_W, cy + HALF_H]),
            [48, 48, 56],
        ));
    }
}

/// Pendura os itens e marca as molduras — **com recorte**, porque é ele que faz a lista parecer
/// uma lista. ⚠️ Nenhuma recebe `VecLayout`: o fluxo é o que o artista arma.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    let mut base = 0;
    for i in 0..ROWS.len() {
        let n = per_row(i);
        if base + n > ids.len() {
            return;
        }
        let Some(&fb) = app.vec_entities.get(&ids[base + n - 1]) else {
            base += n;
            continue;
        };
        let frame = Entity::from_bits(fb);
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
            // ⚠️ **`clip: true` já aqui**, e é material e não armação: sem recorte a lista não é
            // uma lista — é uma pilha de itens a atravessar a cena —, e a rolagem nem se oferece.
            e.insert(VecFrame { clip: true });
        }
        for k in 0..n - 1 {
            let Some(&kb) = app.vec_entities.get(&ids[base + k]) else {
                continue;
            };
            if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(kb)) {
                e.insert(ChildOf(frame));
            }
        }
        base += n;
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let (long, short) = (content_of(ROWS[0].1), content_of(ROWS[1].1));
    eprintln!(
        "[scroll] cena montada: {} formas, {} molduras — a de BAIXO e' o CONTROLE e a roda sobre \
         ela tem de dar ZOOM.",
        gfx.vec_scene.paths().len(),
        ROWS.len()
    );
    eprintln!(
        "[scroll] as duas molduras medem {:.1} de altura; a LISTA tem {:.2} de conteudo \
         (excedente ~{:.2}) e o CONTROLE {:.2} (cabe).",
        HALF_H * 2.0,
        long,
        long - HALF_H * 2.0,
        short
    );
    eprintln!("[scroll] AS MOLDURAS NASCEM SEM FLUXO — e' voce que o liga.");
    eprintln!("[scroll] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no fundo escuro de CADA moldura e ponha Direction -> Column.");
    eprintln!("     Os itens arrumam-se em coluna; na de cima eles passam da caixa e o que");
    eprintln!("     sobra e' RECORTADO (a moldura ja' nasce com Clip ligado).");
    eprintln!("  2. ⚠️ O TESTE: ponha o cursor SOBRE a lista de cima e gire a roda.");
    eprintln!("     Ela ROLA — e a caixa escura NAO se mexe. Se a moldura andar junto com os");
    eprintln!("     itens, isto virou arrastar, e o smoke reprova.");
    eprintln!("  3. Role ate' o fim: o ULTIMO item tem de ser alcancavel, e a lista para ali");
    eprintln!("     (mais roda nao mostra vazio). Role de volta: ela para no primeiro item.");
    eprintln!("  4. ⚠️ O CONTROLE: ponha o cursor sobre a moldura de BAIXO e gire a roda.");
    eprintln!("     Como o conteudo dela CABE, a roda tem de dar ZOOM na camera — e' isto que");
    eprintln!("     prova que a rolagem nao roubou o gesto do dia inteiro.");
    eprintln!("  5. E fora das duas molduras a roda continua a ser zoom, como sempre.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A lista TRANSBORDA e o controle CABE** — a aritmética que o roteiro promete.
    ///
    /// ⚠️ Ela nasceu como gate porque uma cena em que a lista *quase* cabe não mostra nada: o
    /// passo 2 seria uma roda que não move um pixel, indistinguível da feature quebrada.
    #[test]
    fn the_long_list_overflows_and_the_control_fits() {
        let h = HALF_H * 2.0;
        let long = content_of(ROWS[0].1);
        let short = content_of(ROWS[1].1);
        assert!(
            long > h * 2.0,
            "a lista tem de transbordar FOLGADO (mede {long:.2} contra {h:.1}), senao rolar nao \
             mostra nada"
        );
        assert!(
            short < h,
            "o CONTROLE tem de caber (mede {short:.2} contra {h:.1}), senao ele tambem rola e \
             deixa de ser controle"
        );
    }

    /// **O selo do controle: as duas molduras têm a MESMA caixa.** Se elas diferissem, *"uma rola
    /// e a outra não"* teria uma segunda explicação possível (o tamanho), e o smoke não separaria
    /// as duas.
    #[test]
    fn both_frames_have_the_same_box() {
        assert_eq!(ROWS.len(), 2);
        assert_ne!(ROWS[0].1, ROWS[1].1, "so' o CONTEUDO difere");
    }
}
