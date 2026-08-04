//! **A cena das DIFERENÇAS** — `PH2D_BUILD_SMOKE=56` (plano UI/UX W5b).
//!
//! Módulo irmão do [`crate::component_smoke`] pelo teto de LOC, e o corte é o assunto: ali prova-se
//! *"o mestre propaga"*, aqui *"e a cópia pode discordar"*.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *As doze cópias são o mesmo botão — e esta aqui é vermelha porque eu quis.*
//!
//! ⚠️ **Ela dá o MATERIAL e não arma componente nenhum.** Criar e colocar são gestos que a `=53`
//! já prova; repeti-los aqui é barato e mantém a regra que o `impasto_smoke` prega — *um smoke que
//! arma o estado por baixo do pano pula justamente a costura que existe para provar*.
//!
//! O que ela monta, e por quê:
//! - um **botão de duas peças** (caixa + etiqueta), porque um componente de uma peça só não tem
//!   lista: a pergunta *"qual peça?"* não existiria;
//! - uma **PÍLULA verde**, o segundo candidato a mestre — sem ela o *Swap* não tem para onde
//!   trocar, e o botão seria um clique que não leva a lado nenhum;
//! - **espaço à direita** para as cópias nascerem;
//! - e o **quadrado cinza solto**, o CONTROLE: nunca vira componente nem instância, então tem de
//!   ficar exactamente onde nasceu em todos os passos.

use ph2d_ecs::Entity;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// A caixa do botão-mestre: `(x0, y0, x1, y1)` em unidades de mundo.
const BOX: [f64; 4] = [-5.0, 1.0, -1.4, 2.2];
/// A etiqueta dentro dele — a peça que o roteiro recolore e esconde.
const LABEL: [f64; 4] = [-4.6, 1.35, -1.8, 1.85];
/// A PÍLULA: o segundo componente, o alvo do *Swap*.
const PILL: [f64; 4] = [-5.0, -0.6, -2.6, 0.0];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [-5.0, -2.6, -4.2, -1.8];

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

/// **Os quatro caminhos da cena** — porta única, e é o que torna os gates abaixo um ORÁCULO em vez
/// de um espelho: eles medem a geometria que a cena de facto empurra, e não as constantes.
fn paths() -> [VecPath; 4] {
    [
        tint(rectangle([BOX[0], BOX[1]], [BOX[2], BOX[3]]), [58, 96, 168]),
        tint(
            rectangle([LABEL[0], LABEL[1]], [LABEL[2], LABEL[3]]),
            [232, 236, 245],
        ),
        tint(
            rectangle([PILL[0], PILL[1]], [PILL[2], PILL[3]]),
            [56, 150, 96],
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
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 4 {
        return;
    }
    let (Some(&bb), Some(&lb)) = (app.vec_entities.get(&ids[0]), app.vec_entities.get(&ids[1]))
    else {
        return;
    };
    // ⚠️ Pela PORTA (`reparent_keeping_world`): o `settle_origins` já pôs cada forma-raiz a
    // carregar a própria translação, e um `ChildOf` cru SOMA as duas — a etiqueta saltaria para
    // fora da caixa. O defeito está medido no `component_smoke`.
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
        "[pieces] cena montada: {} formas — um botao de DUAS pecas ({w:.1}x{h:.1}), uma PILULA \
         (o 2o componente) e um quadrado de controle. NENHUM componente criado.",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[pieces] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique na caixa AZUL, **Create Component**, e depois **Place Instance**");
    eprintln!("     TRES vezes. Arraste as copias para longe umas das outras.");
    eprintln!("  2. ⚠️ **A PROVA DA WAVE**: selecione UMA copia. A secao Component mostra agora");
    eprintln!("     uma lista **Pieces** — uma linha por peca do mestre, com um interruptor e");
    eprintln!("     uma swatch. Mude a COR da etiqueta nessa copia. ⚠️ So' ELA muda: o mestre e");
    eprintln!("     as irmas ficam como estavam, e a linha passa a dizer **Colour (own)**.");
    eprintln!("  3. Desmarque o interruptor da etiqueta NESSA copia: a etiqueta some ali e fica");
    eprintln!("     nas outras. ⚠️ **A linha CONTINUA na lista** — marque outra vez e ela volta.");
    eprintln!("     (Uma lista que perdesse a linha da peca escondida seria de mao unica.)");
    eprintln!("  4. Com a copia recolorida selecionada, carregue em **Update Main**: a cor sobe");
    eprintln!("     ao mestre e as OUTRAS duas copias mudam junto. ⚠️ Se a copia tambem tinha uma");
    eprintln!("     peca escondida, ela FICA escondida so' nela — e o terminal diz porque (um");
    eprintln!("     mestre nao guarda 'peca escondida').");
    eprintln!("  5. **Swap Main**: selecione outra copia, carregue no botao (ele passa a dizer");
    eprintln!("     'Click a component to swap') e clique na PILULA verde — mas antes faca dela");
    eprintln!("     um componente (selecione-a e **Create Component**). A copia passa a desenhar");
    eprintln!("     a pilula; as outras continuam botoes. ⚠️ Os overrides que ela tinha caem, e o");
    eprintln!("     terminal diz quantos: as pecas do mestre antigo nao existem no novo.");
    eprintln!("  6. **Esc** durante o Swap desiste, e clicar no vazio tambem.");
    eprintln!("  7. ⚠️ **O CONTROLE**: o quadrado CINZA nunca virou nada e tem de estar");
    eprintln!("     exactamente onde nasceu, em todos os passos.");
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

    /// **A etiqueta cabe DENTRO da caixa** — senão o passo 2 mandaria o artista recolorir uma peça
    /// que transborda, e ele julgaria o override por um desenho que já estava errado.
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

    /// **A PÍLULA não encosta no botão** — o *Swap* pede clicar NELA, e um alvo que se sobrepõe ao
    /// mestre faria o clique escolher o de cima sem que o artista soubesse qual.
    #[test]
    fn the_pill_is_clear_of_the_button() {
        let (blo, _) = bbox(0);
        let (_, phi) = bbox(2);
        assert!(
            phi[1] < blo[1],
            "a pilula ({:.2}) encosta na faixa do botao ({:.2})",
            phi[1],
            blo[1]
        );
    }

    /// **O controle está LONGE de tudo** — um controle que uma cópia tapasse não poderia responder
    /// à pergunta que ele existe para responder.
    #[test]
    fn the_control_is_far_from_the_action() {
        let (plo, _) = bbox(2);
        let (_, chi) = bbox(3);
        assert!(
            chi[1] < plo[1],
            "o controle ({:.2}) encosta na pilula ({:.2})",
            chi[1],
            plo[1]
        );
    }

    /// **As quatro peças são DISTINTAS na tela** — duas com a mesma cor fariam o roteiro nomear
    /// uma coisa que o artista não consegue apontar.
    #[test]
    fn the_four_pieces_are_told_apart_by_colour() {
        let fills: Vec<_> = paths().iter().map(|p| p.fill.clone()).collect();
        assert!(fills.iter().all(Option::is_some), "uma peca sem tinta");
        for i in 0..fills.len() {
            for j in i + 1..fills.len() {
                assert_ne!(fills[i], fills[j], "as pecas {i} e {j} tem a mesma cor");
            }
        }
    }

    /// **Há campo livre à direita** — as cópias nascem deslocadas para lá.
    #[test]
    fn there_is_room_to_the_right_for_the_copies() {
        let (_, bhi) = bbox(0);
        assert!(
            bhi[0] < 0.0,
            "o mestre acaba em {:.1}: nao ha' campo livre a' direita",
            bhi[0]
        );
    }
}
