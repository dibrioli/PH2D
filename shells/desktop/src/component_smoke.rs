//! **A cena dos COMPONENTES** — `PH2D_BUILD_SMOKE=53` (plano UI/UX W5).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e NÃO cria componente nenhum** — a cicatriz que o `impasto_smoke` do
//! Painter prega: um smoke que arma o estado por baixo do pano pula justamente a costura que
//! existe para provar. O botão desenhado nasce sendo **arte comum**, e é o artista que carrega no
//! *Create Component*.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Mudo o mestre uma vez, e todas as cópias mudam — menos exactamente o que eu tinha mudado
//! nelas.*
//!
//! O que ela monta, e por quê:
//! - um **botão de duas peças** (caixa + etiqueta), porque um componente de uma peça só não
//!   exercita a sub-árvore, e a sub-árvore é metade do desenho da wave;
//! - **espaço vazio à direita**, onde as cópias vão nascer — sem ele o *Place* poria a primeira
//!   por cima do mestre e o artista não veria as duas;
//! - e um **quadrado cinza solto**, que é o **CONTROLE**: ele nunca vira componente nem instância,
//!   então tem de ficar exactamente onde nasceu em todos os passos. Uma diferença que apareça
//!   nele também não é do componente.

use ph2d_ecs::Entity;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// A caixa do botão-mestre: `(x0, y0, x1, y1)` em unidades de mundo.
const BOX: [f64; 4] = [-5.0, 1.0, -1.4, 2.2];
/// A etiqueta dentro dele — a segunda peça, e a que o passo do override recolore.
const LABEL: [f64; 4] = [-4.6, 1.35, -1.8, 1.85];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [-5.0, -1.6, -4.2, -0.8];

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

/// **Os três caminhos da cena** — a caixa, a etiqueta, e o controle.
///
/// ⚠️ Porta única, e é o que torna os gates abaixo um ORÁCULO em vez de um espelho: eles medem a
/// geometria que a cena de facto empurra, e não as constantes que a descrevem. (A 1ª versão
/// comparava as consts entre si, e o clippy nomeou-a pelo que ela era — *"this assertion has a
/// constant value"*: um gate que o compilador dobra não pode falhar.)
fn paths() -> [VecPath; 3] {
    [
        tint(rectangle([BOX[0], BOX[1]], [BOX[2], BOX[3]]), [58, 96, 168]),
        tint(
            rectangle([LABEL[0], LABEL[1]], [LABEL[2], LABEL[3]]),
            [232, 236, 245],
        ),
        tint(
            rectangle([CONTROL[0], CONTROL[1]], [CONTROL[2], CONTROL[3]]),
            [80, 82, 92],
        ),
    ]
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for p in paths() {
        gfx.vec_scene.push_path(p);
    }
}

/// Pendura a etiqueta na caixa — e mais nada.
///
/// ⚠️ **Nenhum `VecComponentMain`, nenhum `VecInstance`.** O botão é arte comum até o artista o
/// promover, e é essa promoção que esta cena existe para exercitar.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 3 {
        return;
    }
    let (Some(&bb), Some(&lb)) = (app.vec_entities.get(&ids[0]), app.vec_entities.get(&ids[1]))
    else {
        return;
    };
    // ⚠️ Pela PORTA, e o `ChildOf` cru que estava aqui era um defeito MEDIDO desta cena: quando o
    // `arm` corre, o `settle_origins` já pôs cada forma-raiz a carregar a própria translação, e
    // prender uma à outra as SOMAVA — a etiqueta saltava o centro da caixa (3,2 × 1,6 unidades num
    // botão de 3,6 × 1,2) e aterrava fora dela. Ver `vec_transform::reparent_keeping_world`.
    crate::vec_transform::reparent_keeping_world(
        &mut gfx.sim,
        Entity::from_bits(lb),
        Entity::from_bits(bb),
    );
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let w = BOX[2] - BOX[0];
    let h = BOX[3] - BOX[1];
    eprintln!(
        "[component] cena montada: {} formas — um botao de DUAS pecas ({w:.1}x{h:.1}) e um \
         quadrado de controle. NENHUM componente criado.",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[component] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique na caixa AZUL do botao. Na secao **Component** ha' UM botao:");
    eprintln!("     **Create Component**. Carregue nele. ⚠️ Nada se move — um mestre continua a");
    eprintln!("     ser a arte onde ela esta'.");
    eprintln!("  2. O botao mudou para **Place Instance**. Carregue nele TRES vezes. ⚠️ As tres");
    eprintln!("     copias nascem COLADAS ao mestre -- um degrau de paste (12 px de TELA) cada,");
    eprintln!("     em cascata, como um Ctrl+V repetido. Elas NAO podem nascer longe (o artista");
    eprintln!("     nao saberia que existem) nem UMA EM CIMA DA OUTRA. De zoom: a folga entre");
    eprintln!("     elas tem de continuar do mesmo tamanho na TELA.");
    eprintln!("  3. Arraste cada copia para onde quiser (o gizmo move-as normalmente).");
    eprintln!("  4. ⚠️ **A PROVA DA WAVE**: selecione a ETIQUETA branca DENTRO do mestre (clique");
    eprintln!("     nela) e, na secao **Fill**, mude a cor. As TRES copias mudam junto, no mesmo");
    eprintln!("     instante. Ninguem chamou um 'atualizar' — a copia e' derivada por frame.");
    eprintln!("  5. ⚠️ **A OUTRA METADE DA PROVA**: estique o MESTRE pelo gizmo. As tres copias");
    eprintln!("     esticam junto — a FORMA do mestre propaga. Agora ARRASTE o mestre para outro");
    eprintln!("     canto: as copias **nao se mexem** — o LUGAR dele nao propaga. Se as copias");
    eprintln!("     seguirem o arrasto, a separacao entre forma e lugar quebrou.");
    eprintln!("  6. Selecione UMA copia. Aparecem **Detach Instance** (e, se ela tivesse");
    eprintln!("     overrides, **Reset Overrides**). Carregue em **Detach**: ela deixa de seguir");
    eprintln!("     o mestre e vira arte de duas pecas. ⚠️ Ela tem de ficar **EXACTAMENTE onde");
    eprintln!("     estava** (um Detach que move a arte e' um Detach que voce tem de desfazer), e");
    eprintln!("     na Hierarquia a CAIXA continua a ser o pai da ETIQUETA — nao o contrario.");
    eprintln!("     Mude o mestre outra vez — as OUTRAS duas seguem, e a destacada nao.");
    eprintln!(
        "  7. Selecione o MESTRE e apague-o (Delete). ⚠️ As copias **nao desaparecem**: cada"
    );
    eprintln!("     uma mostra o retangulo-suporte dela e o painel diz **Main missing**. Ctrl+Z");
    eprintln!("     traz o mestre de volta e elas voltam a desenhar.");
    eprintln!("  8. ⚠️ **O CONTROLE**: o quadrado CINZA da esquerda nunca virou nada e tem de");
    eprintln!("     estar exactamente onde nasceu, em todos os passos. Se ele se mexeu, o que");
    eprintln!("     voce viu nao foi o componente.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A caixa de mundo de um dos caminhos que a cena empurra.
    fn bbox(i: usize) -> ([f64; 2], [f64; 2]) {
        let p = &paths()[i];
        let mut lo = [f64::MAX; 2];
        let mut hi = [f64::MIN; 2];
        for v in p.verts_all() {
            for a in 0..2 {
                lo[a] = lo[a].min(v.anchor[a]);
                hi[a] = hi[a].max(v.anchor[a]);
            }
        }
        (lo, hi)
    }

    /// **A etiqueta cabe DENTRO da caixa** — senão o passo 4 mandaria o artista clicar numa peça
    /// que transborda, e ele julgaria a propagação por um desenho que já estava errado.
    #[test]
    fn the_label_sits_inside_the_box() {
        let (blo, bhi) = bbox(0);
        let (llo, lhi) = bbox(1);
        for a in 0..2 {
            assert!(
                llo[a] > blo[a] && lhi[a] < bhi[a],
                "a etiqueta sai da caixa no eixo {a}: {llo:?}..{lhi:?} contra {blo:?}..{bhi:?}"
            );
        }
    }

    /// **O mestre nasce no lado ESQUERDO, com espaço à direita** — as cópias nascem deslocadas
    /// para lá, e sem folga a primeira cairia por cima dele.
    #[test]
    fn there_is_room_to_the_right_for_the_copies() {
        let (_, bhi) = bbox(0);
        assert!(
            bhi[0] < 0.0,
            "o mestre acaba em {:.1}: nao ha' campo livre a' direita para as copias",
            bhi[0]
        );
    }

    /// **O controle está LONGE do mestre e das cópias** — um controle que a primeira cópia tapasse
    /// não poderia responder à pergunta que ele existe para responder.
    #[test]
    fn the_control_is_far_from_the_action() {
        let (blo, _) = bbox(0);
        let (_, chi) = bbox(2);
        assert!(
            chi[1] < blo[1],
            "o controle ({:.1}) encosta na faixa do botao ({:.1})",
            chi[1],
            blo[1]
        );
    }

    /// **As três peças são DISTINTAS na tela** — duas com a mesma cor fariam o roteiro nomear uma
    /// coisa que o artista não consegue apontar.
    #[test]
    fn the_three_pieces_are_told_apart_by_colour() {
        let fills: Vec<_> = paths().iter().map(|p| p.fill.clone()).collect();
        assert!(fills.iter().all(Option::is_some), "uma peca sem tinta");
        assert_ne!(fills[0], fills[1], "caixa e etiqueta iguais");
        assert_ne!(fills[0], fills[2], "caixa e controle iguais");
    }
}
