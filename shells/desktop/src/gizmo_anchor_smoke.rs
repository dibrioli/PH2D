//! **A cena da ÂNCORA DE ESCALA** — `PH2D_BUILD_SMOKE=54`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modificador nenhum** — a cicatriz que o `impasto_smoke`
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. Quem carrega no Ctrl é o artista, no meio do arrasto.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Arrastando a MESMA alça, sem Ctrl um canto fica imóvel; com Ctrl o CENTRO fica imóvel — e
//! largar a tecla no meio do gesto devolve o canto, no mesmo frame.*
//!
//! O que ela monta, e por quê:
//! - uma **forma comum** com dois **pinos SOLTOS** — um no canto inferior-esquerdo dela, outro no
//!   centro. Eles não são filhos, então não viajam com a pose: são a régua. Sem pino a pergunta
//!   *"o que ficou parado?"* depende de o artista se lembrar de onde a borda estava;
//! - uma **MOLDURA** com dois filhos, porque ela é a outra metade e falha por outro mecanismo —
//!   uma moldura **redimensiona** (a caixa muda, os filhos não esticam), então a âncora tem de
//!   chegar lá por dentro, e não pela pose;
//! - um **quadrado CINZA solto**, que é o **CONTROLE**: nunca é arrastado, então tem de ficar
//!   exactamente onde nasceu em todos os passos. Uma diferença que apareça nele não é da âncora.

use ph2d_ecs::{ChildOf, Entity, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, ellipse, rectangle};

/// A forma comum: `(x0, y0, x1, y1)` em unidades de mundo.
const SHAPE: [f64; 4] = [-6.0, -1.5, -2.0, 1.5];
/// A moldura, à direita.
const FRAME: [f64; 4] = [1.0, -2.0, 5.0, 2.0];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [-6.0, -4.2, -5.2, -3.4];
/// O raio de um pino. Pequeno de propósito: ele marca um PONTO, não uma região.
const PIN: f64 = 0.16;

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

/// O canto inferior-esquerdo da forma — o pino que fica imóvel SEM tecla ao arrastar a alça NE.
fn corner_pin() -> [f64; 2] {
    [SHAPE[0], SHAPE[1]]
}

/// O centro da forma — o pino que fica imóvel COM a tecla.
fn centre_pin() -> [f64; 2] {
    [(SHAPE[0] + SHAPE[2]) * 0.5, (SHAPE[1] + SHAPE[3]) * 0.5]
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco e o componente só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// **Os caminhos desta cena — porta única.**
///
/// ⚠️ Os gates abaixo medem a geometria que ELA de facto empurra, e não as constantes que a
/// descrevem: comparar as consts entre si é a asserção que o compilador dobra.
fn paths() -> Vec<VecPath> {
    let [px, py] = corner_pin();
    let [cx, cy] = centre_pin();
    let mut v = vec![
        // A forma comum e os dois pinos que a medem.
        tint(
            rectangle([SHAPE[0], SHAPE[1]], [SHAPE[2], SHAPE[3]]),
            [58, 96, 168],
        ),
        tint(ellipse([px, py], PIN, PIN), [235, 120, 90]),
        tint(ellipse([cx, cy], PIN, PIN), [120, 220, 150]),
    ];
    // A moldura: os dois filhos primeiro (desenham ao fundo), a moldura por último — o DFS lista
    // o pai antes e a pilha de z é o inverso.
    let (fx0, fy0, fx1, fy1) = (FRAME[0], FRAME[1], FRAME[2], FRAME[3]);
    v.push(tint(
        rectangle([fx0 + 0.3, fy1 - 1.1], [fx1 - 0.3, fy1 - 0.3]),
        [90, 140, 210],
    ));
    v.push(tint(
        ellipse([(fx0 + fx1) * 0.5, fy0 + 1.0], 0.7, 0.7),
        [235, 200, 120],
    ));
    v.push(tint(rectangle([fx0, fy0], [fx1, fy1]), [46, 46, 54]));
    v.push(tint(
        rectangle([CONTROL[0], CONTROL[1]], [CONTROL[2], CONTROL[3]]),
        [80, 82, 92],
    ));
    v
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for p in paths() {
        gfx.vec_scene.push_path(p);
    }
}

/// Pendura os dois filhos na moldura e marca-a como moldura.
///
/// ⚠️ **A forma comum e os pinos ficam SOLTOS** — os pinos são a régua do gesto, e uma régua que
/// viaja com o que ela mede não mede nada.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 7 {
        return;
    }
    let Some(&fb) = app.vec_entities.get(&ids[5]) else {
        return;
    };
    let frame = Entity::from_bits(fb);
    if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
        e.insert(VecFrame { clip: true });
    }
    for id in &ids[3..5] {
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
    let [px, py] = corner_pin();
    let [cx, cy] = centre_pin();
    eprintln!(
        "[anchor] cena montada: {} formas — uma forma comum com pino de CANTO em ({px:.1}, \
         {py:.1}) e pino de CENTRO em ({cx:.1}, {cy:.1}), uma moldura, e um controle.",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[anchor] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique na forma AZUL. O gizmo aparece com as alcas nos cantos.");
    eprintln!("  2. Arraste a alca do canto SUPERIOR-DIREITO. ⚠️ O pino LARANJA (canto inferior-");
    eprintln!("     esquerdo) tem de ficar IMOVEL — e' o canto oposto, e ele e' a ancora por");
    eprintln!("     default. O pino VERDE (centro) anda.");
    eprintln!(
        "  3. Desfaca (Ctrl+Z). Agora arraste a MESMA alca com **Ctrl premido**. ⚠️ Agora e'"
    );
    eprintln!("     o VERDE que fica imovel e o LARANJA que anda: a forma cresce nos dois lados.");
    eprintln!("  4. ⚠️ **A PROVA DA WAVE**: desfaca, e arraste a mesma alca **largando e voltando");
    eprintln!("     a premir o Ctrl no meio do gesto, sem soltar o botao do rato**. A ancora tem");
    eprintln!("     de saltar entre laranja e verde NO MESMO FRAME, ida e volta, quantas vezes");
    eprintln!("     voce quiser. Se ela so' funcionar na IDA — se largar o Ctrl deixar a forma a");
    eprintln!("     crescer do centro para sempre — o canto foi perdido no primeiro frame.");
    eprintln!("  5. ⚠️ E a POSICAO nao pode saltar quando a tecla muda: o resultado tem de ser o");
    eprintln!("     mesmo que o gesto teria dado se tivesse COMECADO assim. Premir e largar dez");
    eprintln!("     vezes nao pode acumular nada.");
    eprintln!("  6. **A outra metade — a MOLDURA** (o card escuro a direita). Arraste uma alca de");
    eprintln!("     canto dela com e sem Ctrl. ⚠️ Nos DOIS casos a CAIXA muda de tamanho e os");
    eprintln!("     filhos **nao esticam** (uma moldura redimensiona, nao escala) — e o ponto");
    eprintln!("     fixo troca de canto para centro com a tecla, como na forma comum.");
    eprintln!("  7. **Shift continua a travar a proporcao**, e combina com Ctrl: os dois vivos ao");
    eprintln!("     mesmo tempo, cada um no seu eixo de decisao.");
    eprintln!("  8. ⚠️ **O CONTROLE**: o quadrado CINZA em baixo a esquerda nunca foi arrastado e");
    eprintln!("     tem de estar exactamente onde nasceu. Se ele se mexeu, o que voce viu nao foi");
    eprintln!("     a ancora.");
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

    /// **Os pinos estão onde o roteiro diz que estão** — medidos na geometria, não nas consts.
    ///
    /// ⚠️ Sem isto o passo 2 mandaria o artista olhar para um ponto que não é o canto oposto, e
    /// ele julgaria a âncora por uma régua torta.
    #[test]
    fn the_pins_sit_on_the_corner_and_on_the_centre() {
        let (slo, shi) = bbox(0);
        let (clo, chi) = bbox(1);
        let (nlo, nhi) = bbox(2);
        let corner = [(clo[0] + chi[0]) * 0.5, (clo[1] + chi[1]) * 0.5];
        let centre = [(nlo[0] + nhi[0]) * 0.5, (nlo[1] + nhi[1]) * 0.5];
        assert!(
            (corner[0] - slo[0]).abs() < 1e-9 && (corner[1] - slo[1]).abs() < 1e-9,
            "o pino de canto nao esta' no canto inferior-esquerdo: {corner:?} contra {slo:?}"
        );
        assert!(
            (centre[0] - (slo[0] + shi[0]) * 0.5).abs() < 1e-9
                && (centre[1] - (slo[1] + shi[1]) * 0.5).abs() < 1e-9,
            "o pino de centro nao esta' no centro: {centre:?}"
        );
    }

    /// **Os dois pinos são DISTINGUÍVEIS na tela** — dois pontos da mesma cor fariam o roteiro
    /// nomear uma coisa que o artista não consegue apontar.
    #[test]
    fn the_two_pins_are_told_apart_by_colour() {
        let ps = paths();
        assert_ne!(ps[1].fill, ps[2].fill, "os dois pinos tem a mesma tinta");
        assert!(ps[1].fill.is_some() && ps[2].fill.is_some());
    }

    /// **O controle está LONGE de tudo** — um controle que uma alça arrastada alcance não
    /// responde à pergunta que ele existe para responder.
    #[test]
    fn the_control_is_far_from_the_action() {
        let (slo, _) = bbox(0);
        let (_, ghi) = bbox(6);
        assert!(
            ghi[1] < slo[1],
            "o controle ({:.1}) encosta na faixa da forma ({:.1})",
            ghi[1],
            slo[1]
        );
    }
}
