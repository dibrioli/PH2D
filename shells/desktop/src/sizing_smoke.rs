//! **A cena do SIZING** — `PH2D_BUILD_SMOKE=66` (o vocabulário de tamanho do Figma).
//!
//! Módulo irmão do [`crate::layout_smoke`], que é a cena do fluxo; esta é a do **tamanho**. Aquela
//! pergunta *"como os filhos se arrumam?"*; esta, *"quem decide o tamanho da caixa?"*.
//!
//! ⚠️ **Ela dá o MATERIAL e NÃO arma nada** — a cicatriz que o `impasto_smoke` do Painter prega: um
//! smoke que arma o estado por baixo do pano pula justamente a costura que existe para provar.
//!
//! # As três perguntas de olho
//!
//! 1. **O abraço.** Duas molduras com conteúdos de larguras DIFERENTES nascem com a mesma caixa
//!    larga. Ligado o `Hug`, cada uma encolhe até o próprio conteúdo — e ficam de tamanhos
//!    diferentes uma da outra. *É o botão que cresce com o rótulo.*
//! 2. **O piso.** A moldura de conteúdo curto, com `Min W`, para de encolher onde o artista disse.
//! 3. **O fora-do-fluxo.** O selo âmbar sobre a quina não entra na fila — e os outros três
//!    arrumam-se como se ele não existisse.
//!
//! ⚠️ E a quarta forma da cena é o **CONTROLE**: a moldura de baixo tem o mesmo conteúdo e nunca é
//! tocada. Uma diferença que aparecer nas duas não é do sizing.

use ph2d_ecs::{ChildOf, Entity, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// `(centro y, meia-largura da moldura, nº de filhos, meia-largura do filho, rótulo)`.
///
/// ⚠️ As duas primeiras têm a MESMA caixa e conteúdos de larguras diferentes — é essa diferença
/// que o abraço torna visível, e uma cena com conteúdos iguais não a mostraria.
type Row = (f64, f64, usize, f64, &'static str);
const ROWS: &[Row] = &[
    (3.4, 4.5, 2, 0.8, "CURTO"),
    (1.0, 4.5, 4, 0.8, "LONGO"),
    (-1.4, 4.5, 3, 0.8, "SELO"),
    (-3.8, 4.5, 3, 0.8, "CONTROLE"),
];
/// Meia-altura de toda moldura.
const HALF_H: f64 = 0.85;
/// Meia-altura de um filho.
const KID_H: f64 = 0.5;
/// O desnível vertical que os filhos nascem a ter.
///
/// ⚠️ Ele existe para o passo 1 ter o que consertar: uma fila que já nascesse alinhada tornaria o
/// *Row* invisível, e o artista não saberia se o ligou. É a mesma razão da desordem da cena `=50`.
const STAGGER: f64 = 0.22;
/// O vão que a cena desenha entre os filhos — o mesmo que o artista vai ver no campo Gap.
const GAP: f64 = 0.25;

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

/// Quantos caminhos cada fileira empurra: os filhos, o selo da terceira, e a moldura.
fn per_row(i: usize) -> usize {
    ROWS[i].2 + usize::from(i == 2) + 1
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    for (i, &(cy, half_w, n, kid_w, _)) in ROWS.iter().enumerate() {
        let left = -half_w;
        for k in 0..n {
            let x = left + 0.3 + kid_w + (k as f64) * (kid_w * 2.0 + GAP);
            // O desnível alterna, e a última fileira (o CONTROLE) o tem igual: ela tem de nascer
            // exactamente como as outras para que "não mudou" queira dizer alguma coisa.
            let y = cy + if k % 2 == 0 { STAGGER } else { -STAGGER };
            s.push_path(tint(
                rectangle([x - kid_w, y - KID_H], [x + kid_w, y + KID_H]),
                [90, 140, 210],
            ));
        }
        // O SELO da terceira fileira: pousado sobre a quina superior-direita da moldura, e é
        // exactamente ali que ele tem de FICAR quando o resto se arruma.
        if i == 2 {
            let (sx, sy) = (half_w - 0.45, cy + HALF_H - 0.2);
            s.push_path(tint(
                rectangle([sx - 0.35, sy - 0.35], [sx + 0.35, sy + 0.35]),
                [235, 170, 90],
            ));
        }
        s.push_path(tint(
            rectangle([-half_w, cy - HALF_H], [half_w, cy + HALF_H]),
            [48, 48, 56],
        ));
    }
}

/// Pendura os filhos e marca as molduras. ⚠️ **Nenhuma recebe `VecLayout` nem `VecLayoutSize`**: o
/// fluxo e o tamanho são o que o artista arma, e são a costura inteira desta wave.
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
            e.insert(VecFrame { clip: false });
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
    // O que cada moldura vai medir depois do abraço: os filhos + os vãos + o recuo (que nasce em
    // zero). É o oráculo do passo 2, e ele sai da MESMA aritmética que a cena desenhou.
    let hug = |n: usize, kid_w: f64| (n as f64) * kid_w * 2.0 + (n as f64 - 1.0) * GAP;
    eprintln!(
        "[sizing] cena montada: {} formas, {} molduras — a de BAIXO e' o CONTROLE e nao pode mudar \
         em passo nenhum.",
        gfx.vec_scene.paths().len(),
        ROWS.len()
    );
    eprintln!(
        "[sizing] as molduras nascem TODAS com {:.1} de largura; abracadas, a CURTO passa a medir \
         ~{:.2} e a LONGO ~{:.2}.",
        ROWS[0].1 * 2.0,
        hug(ROWS[0].2, ROWS[0].3),
        hug(ROWS[1].2, ROWS[1].3)
    );
    eprintln!("[sizing] AS MOLDURAS NASCEM SEM FLUXO E SEM TAMANHO — e' voce que os liga.");
    eprintln!("[sizing] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Ponha Direction -> Row nas TRES molduras de cima (CURTO, LONGO e SELO):");
    eprintln!(
        "     clique no fundo escuro de cada uma e escolha Row. ⚠️ A SELO tambem — o passo 5"
    );
    eprintln!("     depende dela, e um filho de moldura PARADA nao tem fluxo de que sair.");
    eprintln!(
        "  2. ⚠️ O ABRACO: em CURTO e em LONGO, ponha **Width -> Hug**. Cada moldura encolhe"
    );
    eprintln!(
        "     ate' o proprio conteudo, e as duas ficam com LARGURAS DIFERENTES uma da outra."
    );
    eprintln!("     E' o botao que cresce com o rotulo. (O fundo escuro encolhe junto: o tamanho");
    eprintln!("     novo e' DESENHADO, nao so' calculado.)");
    eprintln!("  3. Ainda em CURTO, escreva **Min W = 6**. Ela para de encolher e fica em 6,");
    eprintln!("     enquanto a LONGO continua no tamanho do conteudo dela.");
    eprintln!("  4. Ponha **Height -> Hug** numa delas: a altura desce ate' a dos filhos, e a");
    eprintln!("     largura NAO se mexe — os dois eixos sao independentes.");
    eprintln!("  5. ⚠️ O FORA-DO-FLUXO (exige o Row na SELO, do passo 1 — sem ele o painel diz");
    eprintln!("     'Set the parent frame to Row or Column first' e nao oferece o toggle):");
    eprintln!("     na moldura SELO, clique no quadrado AMBAR (a quina) e");
    eprintln!("     marque **Absolute position**. Ele fica onde esta'; os tres azuis arrumam-se");
    eprintln!("     como se ele nao existisse. E as linhas Grow/Shrink DESAPARECEM enquanto ele");
    eprintln!("     esta' marcado — quem saiu do fluxo nao reparte sobra nenhuma.");
    eprintln!("  6. Desmarque: ele volta para a fila. E confira o CONTROLE (a de baixo): ela tem");
    eprintln!("     de estar exactamente como comecou.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A aritmética que o roteiro promete existe na cena.**
    ///
    /// ⚠️ Ela nasceu VERMELHA: com cinco filhos a `LONGO` abraçada media 9,0 — exactamente a
    /// largura da caixa —, e o passo do abraço não mostraria nada nela.
    #[test]
    fn every_frame_shrinks_visibly_when_it_hugs() {
        let width = ROWS[0].1 * 2.0;
        for &(_, half_w, n, kid_w, name) in ROWS {
            let hug = (n as f64) * kid_w * 2.0 + (n as f64 - 1.0) * GAP;
            assert!(
                hug < width - 1.0,
                "a {name} abracada mede {hug:.2} contra uma caixa de {:.1}: o abraco seria \
                 invisivel",
                half_w * 2.0
            );
        }
    }

    /// **As duas primeiras encolhem para tamanhos DIFERENTES uma da outra.** É a pergunta do passo
    /// 2 — *o botão cresce com o rótulo* —, e duas molduras que encolhessem para o mesmo número
    /// não a responderiam.
    #[test]
    fn the_two_hugging_frames_end_up_different_widths() {
        let hug = |i: usize| (ROWS[i].2 as f64) * ROWS[i].3 * 2.0 + (ROWS[i].2 as f64 - 1.0) * GAP;
        assert!(
            (hug(0) - hug(1)).abs() > 1.0,
            "CURTO {:.2} e LONGO {:.2} tem de ficar visivelmente diferentes",
            hug(0),
            hug(1)
        );
        // E o piso do passo 3 tem de cair ENTRE as duas: acima do abraço da CURTO (senão não
        // segura nada) e abaixo do da LONGO (senão as duas ficam iguais e o passo não diz nada).
        const FLOOR: f64 = 6.0;
        assert!(
            hug(0) < FLOOR && FLOOR < hug(1),
            "o Min W = {FLOOR} do roteiro tem de ficar entre {:.2} e {:.2}",
            hug(0),
            hug(1)
        );
    }

    /// **O selo da fileira 2 é contado como caminho próprio** — se ele não for, o `adopt` pendura
    /// a moldura errada e o passo 5 manda o artista clicar num quadrado que não é filho de nada.
    #[test]
    fn the_seal_is_one_of_the_paths_of_its_row() {
        assert_eq!(per_row(2), ROWS[2].2 + 2, "3 filhos + o selo + a moldura");
        let total: usize = (0..ROWS.len()).map(per_row).sum();
        assert_eq!(total, 17, "o build empurra 17 caminhos");
    }
}
