//! **A cena das ÂNCORAS** — `PH2D_BUILD_SMOKE=52` (plano UI/UX W3).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e NÃO arma regra nenhuma** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. Os quatro filhos nascem SEM `VecAnchors`, e é o artista que escolhe *Right* e *Stretch*
//! no painel.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Um HUD desenhado num telefone continua a fazer sentido num desktop — sem o artista mexer numa
//! única peça.*
//!
//! O que ela monta, e por quê:
//! - **UMA moldura** larga (um HUD é paisagem), com quatro peças coladas onde um artista as poria;
//! - a **pontuação** no canto direito — é ela que fica para trás quando a moldura cresce, e é o
//!   primeiro sintoma que o artista reconhece;
//! - a **barra** que atravessa quase toda a largura — o caso do *Stretch*, que nenhuma âncora de
//!   ponto resolve;
//! - e o **quadrado cinza no meio**, que é o **CONTROLE**: ele nunca recebe regra, então tem de
//!   ficar exactamente onde nasceu em todos os passos. Uma diferença que apareça nele também não
//!   é da âncora.

use ph2d_ecs::{ChildOf, Entity, VecClipContent, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Meia-largura e meia-altura da moldura — o "telefone deitado" com que a cena abre.
const HALF: [f64; 2] = [4.0, 2.2];
/// As quatro peças: `(meia-largura, meia-altura, dx do canto ESQUERDO, dy do canto INFERIOR, cor)`.
type Piece = (f64, f64, f64, f64, [u8; 3]);
const PIECES: &[Piece] = &[
    // VIDA — canto superior esquerdo. Fica onde está mesmo sem regra (a aresta que ela segue não
    // se move), e é por isso que ela é a peça que NÃO precisa de âncora.
    (1.1, 0.22, 0.35, 3.85, [110, 200, 130]),
    // PONTUAÇÃO — canto superior DIREITO. É a que se descola.
    (0.7, 0.22, 6.55, 3.85, [235, 200, 120]),
    // A BARRA — atravessa quase toda a largura, junto ao fundo. O caso do Stretch.
    (3.5, 0.16, 0.35, 0.3, [90, 140, 210]),
    // O CONTROLE — no meio, sem regra nenhuma, nunca.
    (0.4, 0.4, 3.6, 1.6, [80, 82, 92]),
];

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

/// Os caminhos: as QUATRO peças primeiro, a moldura por último (o fundo do card — a mesma ordem
/// de pilha que as cenas da W0 e da W2 usam, e pela mesma razão).
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    let (left, bottom) = (-HALF[0], -HALF[1]);
    for &(hw, hh, dx, dy, rgb) in PIECES {
        let (x, y) = (left + dx + hw, bottom + dy + hh);
        s.push_path(tint(rectangle([x - hw, y - hh], [x + hw, y + hh]), rgb));
    }
    s.push_path(tint(
        rectangle([-HALF[0], -HALF[1]], [HALF[0], HALF[1]]),
        [40, 42, 50],
    ));
}

/// Pendura as quatro peças na moldura e marca-a como moldura.
///
/// ⚠️ **Nenhuma recebe `VecAnchors`**: a regra é o que o artista arma, e é a costura inteira desta
/// wave. Uma cena que já a trouxesse ligada provaria o passe e não o produto.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < PIECES.len() + 1 {
        return;
    }
    let Some(&fb) = app.vec_entities.get(&ids[PIECES.len()]) else {
        return;
    };
    let frame = Entity::from_bits(fb);
    if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
        e.insert((VecFrame, VecClipContent));
    }
    for id in ids.iter().take(PIECES.len()) {
        let Some(&kb) = app.vec_entities.get(id) else {
            continue;
        };
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(kb)) {
            e.insert(ChildOf(frame));
        }
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let width = HALF[0] * 2.0;
    let (score_hw, score_dx) = (PIECES[1].0, PIECES[1].2);
    let gap_right = width - (score_dx + score_hw * 2.0);
    eprintln!(
        "[anchors] cena montada: {} formas — UMA moldura de {width:.1} de largura com quatro \
         pecas, e NENHUMA regra armada.",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[anchors] a pontuacao acaba a {gap_right:.2} da borda direita, e a barra atravessa \
         {:.1} dos {width:.1}.",
        PIECES[2].0 * 2.0
    );
    eprintln!("[anchors] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no fundo escuro da moldura. Selecione-a e, na secao **Transform**,");
    eprintln!("     ponha **W = 14** (Enter). ⚠️ A moldura estica e as quatro pecas FICAM ONDE");
    eprintln!("     ESTAO: a pontuacao afasta-se da borda direita e a barra deixa um rombo. E'");
    eprintln!("     este o problema que a wave resolve — olhe-o antes de o consertar.");
    eprintln!("  2. Ctrl+Z ate' a moldura voltar a {width:.1}.");
    eprintln!("  3. Clique na **PONTUACAO** (o retangulo amarelo em cima, a' direita). Aparece a");
    eprintln!("     secao **Constraints**, com Horizontal em **Left** e Vertical em **Top** —");
    eprintln!("     que e' o estado em que ela ja' esta'. Escolha Horizontal -> **Right**.");
    eprintln!("  4. Clique na **BARRA** (a azul, em baixo). Horizontal -> **Stretch**,");
    eprintln!("     Vertical -> **Bottom**.");
    eprintln!("  5. ⚠️ Selecione a moldura e ponha **W = 14** outra vez. Agora a pontuacao GRUDA");
    eprintln!("     na borda direita e a barra ESTICA com ela. A vida (verde) fica no canto");
    eprintln!("     esquerdo, que e' o certo — a aresta que ela segue nao se moveu.");
    eprintln!("  6. ⚠️ **O CONTROLE**: o quadrado CINZA do meio nunca recebeu regra e tem de");
    eprintln!("     estar exactamente onde nasceu, em todos os passos. Se ele se mexeu, o que");
    eprintln!("     voce viu nao foi a ancora.");
    eprintln!("  7. Arraste a **PONTUACAO** com o gizmo. Ela move-se normalmente, e continua");
    eprintln!("     grudada a' direita no proximo redimensionamento — a posicao continua a ser");
    eprintln!("     autorada por si; a regra so' diz o que ACONTECE quando a moldura muda.");
    eprintln!("  8. Volte a pontuacao para Horizontal -> **Left**. A regra desaparece do arquivo");
    eprintln!("     (e' o neutro) e ela volta a ficar para tras quando a moldura cresce.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém o VÃO que o roteiro promete.** Sem folga entre a pontuação e a borda
    /// direita, o passo 1 mandaria o artista olhar para uma peça que já está colada — e o
    /// *Right* do passo 3 não teria nada a mostrar.
    #[test]
    fn the_score_starts_near_the_right_edge_with_a_visible_gap() {
        let width = HALF[0] * 2.0;
        let right = PIECES[1].2 + PIECES[1].0 * 2.0;
        let gap = width - right;
        assert!(gap > 0.0, "a pontuacao transborda a moldura");
        assert!(
            gap < 0.6,
            "a pontuacao esta' a {gap:.2} da borda — ela nao le' como 'canto direito'"
        );
    }

    /// **A barra atravessa a maior parte da largura** — é isso que faz o rombo do passo 1 e o
    /// esticão do passo 5 serem visíveis. Uma barrinha curta pareceria uma peça qualquer.
    #[test]
    fn the_bar_spans_most_of_the_frame() {
        let width = HALF[0] * 2.0;
        assert!(
            PIECES[2].0 * 2.0 > width * 0.8,
            "a barra mede {:.1} de {width:.1} — nao le' como uma barra",
            PIECES[2].0 * 2.0
        );
    }

    /// **Toda peça cabe DENTRO da moldura.** A moldura recorta (`clip: true`), então uma peça que
    /// transbordasse nasceria cortada e o artista julgaria a âncora por um desenho que já estava
    /// errado antes de ele tocar em nada.
    #[test]
    fn every_piece_is_inside_the_frame() {
        let (w, h) = (HALF[0] * 2.0, HALF[1] * 2.0);
        for (i, &(hw, hh, dx, dy, _)) in PIECES.iter().enumerate() {
            assert!(dx >= 0.0 && dx + hw * 2.0 <= w, "peca {i} sai em x");
            assert!(dy >= 0.0 && dy + hh * 2.0 <= h, "peca {i} sai em y");
        }
    }
}
