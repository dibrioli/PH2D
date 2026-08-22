//! **A cena da GRADE** — `PH2D_BUILD_SMOKE=69` (item 5 do estudo dos contêineres).
//!
//! Módulo irmão do [`crate::layout_smoke`] (o fluxo) e do [`crate::sizing_smoke`] (o tamanho).
//! Aquelas perguntam *"como os filhos se arrumam?"* e *"quem decide o tamanho da caixa?"*; esta
//! pergunta **o que uma grade dá que um `RowWrap` não dá** — e a resposta são duas coisas, e só
//! estas duas: **colunas alinhadas entre linhas** e **refluxo automático**.
//!
//! # O CONTROLE é metade da cena, e ele é PRÉ-ARMADO de propósito
//!
//! ⚠️ A moldura de baixo nasce em **`RowWrap`**, com os MESMOS filhos, e é a única coisa que esta
//! cena arma. A lei que o `impasto_smoke` prega — *um smoke que arma o estado por baixo do pano
//! pula justamente a costura que existe para provar* — vale para a GRADE, que é o que esta wave
//! construiu, e o wrap já shipava. Sem ele lado a lado, *"as colunas alinham"* não é uma
//! observação: alinhado contra o quê?
//!
//! # A COR é por COLUNA, e é isso que torna a propriedade visível num relance
//!
//! Cada filho é tingido pelo índice dele módulo três. Numa grade isso desenha **três faixas
//! verticais**; num wrap as mesmas cores ficam espalhadas, porque cada faixa se arruma sozinha.
//! Um conjunto de filhos de larguras IGUAIS faria as duas coincidir — e seria a fixture em que o
//! olho não distingue a feature de um wrap com sorte.

use ph2d_ecs::{ChildOf, Entity, LayoutDir, VecFrame, VecLayout};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// As meias-larguras dos seis filhos, **deliberadamente desiguais**.
///
/// ⚠️ Os números são escolhidos para o wrap partir **3+3**, e a premissa é conferida no
/// [`announce`]. Com uma partição diferente a comparação deixaria de ser justa — foi o defeito de
/// fixture que o gate do motor pagou: *um wrap quebra onde a LARGURA acaba, nunca numa contagem*.
/// Linha 1: `0,85+0,85+0,35` = 4,60 com os vãos (cabe em 5,00; com o quarto daria 5,55 e quebra).
/// Linha 2: `0,35+0,35+0,35` = 2,60.
const KID_W: [f64; 6] = [0.85, 0.85, 0.35, 0.35, 0.35, 0.35];
/// Meia-altura de um filho.
const KID_H: f64 = 0.35;
/// Meia-largura de cada moldura.
const HALF_W: f64 = 2.5;
/// Meia-altura de cada moldura — folgada de propósito, para o `Align` ter sobra sobre que agir.
const HALF_H: f64 = 1.3;
/// O vão que o artista vai ver no campo *Gap*.
const GAP: f64 = 0.25;
/// Os centros verticais das duas molduras: a de cima é a que ele arma, a de baixo é o controle.
const CY: [f64; 2] = [1.7, -1.7];
/// O desnível com que os filhos nascem — sem ele, ligar a grade não teria o que consertar.
const STAGGER: f64 = 0.18;
/// Uma cor por COLUNA (índice módulo três).
const COLUMN_RGB: [[u8; 3]; 3] = [[86, 132, 214], [214, 132, 86], [120, 196, 128]];

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

/// Quantos caminhos cada moldura empurra: os seis filhos e a própria caixa.
const PER_FRAME: usize = KID_W.len() + 1;

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    for cy in CY {
        let mut x = -HALF_W + 0.3;
        for (k, w) in KID_W.iter().enumerate() {
            let y = cy + if k % 2 == 0 { STAGGER } else { -STAGGER };
            s.push_path(tint(
                rectangle([x, y - KID_H], [x + w * 2.0, y + KID_H]),
                COLUMN_RGB[k % 3],
            ));
            x += w * 2.0 + GAP;
        }
        s.push_path(tint(
            rectangle([-HALF_W, cy - HALF_H], [HALF_W, cy + HALF_H]),
            [48, 48, 56],
        ));
    }
}

/// Pendura os filhos, e arma **só o CONTROLE**.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < PER_FRAME * 2 {
        return;
    }
    for (row, _) in CY.iter().enumerate() {
        let base = row * PER_FRAME;
        let Some(&fb) = app.vec_entities.get(&ids[base + PER_FRAME - 1]) else {
            continue;
        };
        let frame = Entity::from_bits(fb);
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
            e.insert(VecFrame);
            // ⚠️ **Só a de BAIXO é armada.** A de cima é a costura que esta wave existe para
            // provar, e armá-la aqui pularia exactamente o passo que o artista tem de fazer.
            if row == 1 {
                e.insert(VecLayout {
                    dir: LayoutDir::RowWrap,
                    gap: [GAP, GAP],
                    ..VecLayout::default()
                });
            }
        }
        for k in 0..PER_FRAME - 1 {
            let Some(&kb) = app.vec_entities.get(&ids[base + k]) else {
                continue;
            };
            if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(kb)) {
                e.insert(ChildOf(frame));
            }
        }
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let armed = gfx
        .vec_scene
        .paths()
        .iter()
        .filter(|p| {
            app.vec_entities.get(&p.id).is_some_and(|&b| {
                gfx.sim
                    .world()
                    .get::<VecLayout>(Entity::from_bits(b))
                    .is_some()
            })
        })
        .count();
    // A premissa da comparação: com a largura desta moldura, o wrap TEM de partir 3+3.
    let first_row: f64 = KID_W[..3].iter().map(|w| w * 2.0).sum::<f64>() + GAP * 2.0;
    let with_fourth = first_row + GAP + KID_W[3] * 2.0;
    eprintln!(
        "[grid-smoke] cena montada: {} formas, {armed} moldura(s) ARMADA(s) (tem de ser 1 — so' o \
         CONTROLE).",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[grid-smoke] (!) se o numero de armadas nao for 1, PARE: a de cima tem de nascer sem \
         layout, senao o smoke pula o passo que ele existe para provar."
    );
    eprintln!(
        "[grid-smoke] a premissa do CONTROLE: a moldura mede {:.2} e a 1a faixa do wrap mede \
         {first_row:.2}; com o 4o filho daria {with_fourth:.2} > {:.2}, entao ele parte 3+3.",
        HALF_W * 2.0,
        HALF_W * 2.0
    );
    eprintln!("[grid-smoke] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique na moldura de CIMA. Secao Layout, fileira Direction: escolha 'Grid'.");
    eprintln!("     Um campo 'Cols' nasce logo abaixo — escreva  3  e tecle Enter.");
    eprintln!("  2. (!) O TESTE: as tres CORES formam COLUNAS. Na moldura de BAIXO — o CONTROLE,");
    eprintln!("     que ja' nasce em Wrap com os MESMOS filhos — as mesmas cores ficam");
    eprintln!(
        "     ESPALHADAS: cada faixa de um wrap arruma-se sozinha, entao nada fica em coluna."
    );
    eprintln!("  3. Mude 'Cols' para 2. Os seis filhos REFLUEM em tres linhas de dois, sem voce");
    eprintln!("     mexer em mais nada. Volte para 3.");
    eprintln!("  4. (!) Arraste o ULTIMO filho para a PRIMEIRA celula (a de cima, a' esquerda).");
    eprintln!("     Ele tem de ir para o COMECO da fila. Antes desta wave ele ia para o meio: a");
    eprintln!("     regua media so' o eixo X, e as tres celulas da coluna 0 partilham o mesmo x.");
    eprintln!("  5. Fileira Align: 'Center' desce o BLOCO de linhas para o meio da moldura, e");
    eprintln!(
        "     'End' encosta-o em baixo. Ele governa onde as LINHAS sentam, e nao so' o filho."
    );
    eprintln!(
        "  6. (!) Fileira Justify: na grade ela tem TRES chips. 'Between' e 'Around' nao sao"
    );
    eprintln!("     oferecidos — com colunas iguais nao sobra espaco para repartir, e um chip que");
    eprintln!("     nao move um pixel e' pior que um chip que falta. Volte a direcao para 'Row':");
    eprintln!("     os dois VOLTAM, e o valor que estava la' nunca foi reescrito.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A premissa do CONTROLE é AFIRMADA, e não esperada.**
    ///
    /// ⚠️ Se o wrap não partir 3+3, a comparação deixa de ser justa: a grade e o wrap estariam a
    /// dispor números diferentes de filhos por linha, e *"as colunas não alinham"* teria uma
    /// segunda explicação. É o defeito de fixture que o gate do motor já pagou.
    #[test]
    fn the_control_wrap_breaks_three_and_three() {
        let w = HALF_W * 2.0;
        let three: f64 = KID_W[..3].iter().map(|k| k * 2.0).sum::<f64>() + GAP * 2.0;
        let four = three + GAP + KID_W[3] * 2.0;
        assert!(
            three <= w,
            "os tres primeiros tem de caber ({three} contra {w})"
        );
        assert!(four > w, "o quarto tem de NAO caber ({four} contra {w})");
        let rest: f64 = KID_W[3..].iter().map(|k| k * 2.0).sum::<f64>() + GAP * 2.0;
        assert!(
            rest <= w,
            "os tres ultimos tem de caber na 2a faixa ({rest} contra {w})"
        );
    }

    /// **As larguras são DESIGUAIS** — com filhos iguais a grade e o wrap coincidem, e a cena não
    /// mostraria nada.
    #[test]
    fn the_children_have_different_widths() {
        assert!(
            KID_W.iter().any(|w| (w - KID_W[0]).abs() > 1e-9),
            "todos os filhos tem a mesma largura: a grade seria indistinguivel do wrap"
        );
    }
}
