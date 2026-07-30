//! **A CHAVE do memo do FX raster** — a pergunta *"os pixels desta forma ainda servem?"*, feita
//! num só lugar.
//!
//! # O defeito que este módulo existe para fechar (medido 2026-07-29)
//!
//! O memo do [`crate::fx_live`] era `(pilha resolvida, largura, altura)`. Mas o que está na textura
//! é o resultado de **rasterizar a forma** e correr a pilha sobre esses pixels — e a forma não
//! entrava na chave. Consequência alcançável: **mudar a cor do preenchimento de uma forma filtrada
//! não muda a tela.** A pilha é a mesma, a caixa é a mesma, então o memo acerta e o artista
//! continua a ver a cor antiga, sem erro e sem warning. O mesmo valia para o traço, para a pilha de
//! Live Path Effects e para qualquer edição de geometria que não movesse a caixa (arrastar um
//! vértice interior).
//!
//! ⚠️ **O memo não era otimização, era uma afirmação**: *"nada que eu desenho mudou"*. Ela estava
//! escrita em três números que não sabem nada sobre o desenho.
//!
//! # A lei, e por que a chave carrega VALORES e não um hash
//!
//! A chave é **exatamente o que os dois consumidores leem** — o
//! [`ph2d_vec_render::draw_path_isolated`] (que põe a forma na célula do atlas) e o
//! [`ph2d_vec_render::silhouette_segments`] (que dá a fronteira aos degraus que a usam). Ela guarda
//! os **valores** e compara com `==`, como o `ops` já fazia: é **exato** (sem colisão, sem discutir
//! bits de `f64`) e o clone só é pago no **miss**, isto é, no frame em que já vamos rasterizar —
//! onde ele é ruído contra um render do Vello.
//!
//! ⚠️ **A TRANSLAÇÃO fica FORA da chave, de propósito.** O conteúdo da célula é a forma desenhada em
//! `-ex0,-ey0`, então mover a forma (ou panhar a câmera) dá **a mesma arte na mesma posição dentro
//! da célula** — o que muda é o `rect` onde a célula é desenhada, e esse é recomputado todo frame.
//! Incluir a translação faria **toda forma filtrada re-cozinhar em todo frame de pan**, que é
//! precisamente o gesto onde o memo se paga. O que se perde é a **fase sub-pixel**: um pan de meio
//! pixel reusa um antialiasing calculado para a fase anterior. É diferença de AA, e o preço de a
//! corrigir é re-cozinhar a cena inteira a cada frame de pan.
//!
//! A parte **LINEAR** do afim, ao contrário, entra: rodar ou escalar muda os pixels de verdade.
//! Entram as duas metades dela (a da câmera e a da pose da forma) porque os dois consumidores
//! cascateiam diferente — a geometria derivada é desenhada pela câmera, a autorada pela pose — e
//! perguntar por uma só deixaria um vão exactamente onde a outra governa.
//!
//! ⚠️ **Mas `cam`/`screen` são CINTO, não gate, e a diferença está medida:** uma mudança linear que
//! altera os pixels altera a **caixa** também (girar 45° cresce `w`/`h`; escalar 2× dobra-os), e uma
//! que não altera a caixa é uma **simetria da forma**, onde os pixels são os mesmos. Não consegui
//! construir um caso em que estes dois campos decidam sozinhos — então eles ficam por serem 8 `f64`
//! de graça contra um modo de falha que é *pixels velhos que ninguém vê que são velhos*, e **nenhum
//! gate afirma que eles estão aqui**. Dizer o contrário seria vender um gate que não pode falhar.

use ph2d_ecs::SimWorld;
use ph2d_render::{FxOpGpu, stack_reach};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};
use ph2d_vector::Affine;

use crate::vec_entities::VecEntityMap;

/// O maior lado de scratch/saída que pedimos à GPU — o `maxTextureDimension2D` baseline do WebGPU
/// (8192). Limite de RECURSO (a dimensão de textura garantida), não de gosto.
pub(crate) const MAX_FX_SIDE: u32 = 8192;

/// **O que é DESENHADO na célula desta forma.** Um superconjunto conservador do que os dois
/// consumidores leem: a geometria derivada (se algum produtor vivo a substituiu), a silhueta (se
/// houver), a forma autorada, e a parte linear dos dois afins.
///
/// ⚠️ Carrega a forma autorada **mesmo quando há geometria derivada** — nesse caso o desenho não a
/// lê, mas incluí-la só pode causar um miss a mais (nunca um hit a menos), e a derivada é função
/// dela de qualquer modo. Superconjunto conservador vale mais que mínimo exacto aqui: o modo de
/// falha de um vão é **pixels velhos que ninguém vê que são velhos**.
#[derive(Clone, PartialEq)]
pub(crate) struct FxDrawn {
    /// A silhueta derivada desta forma (o produtor de silhueta), se houver.
    sil: Option<Vec<VecPath>>,
    /// A geometria derivada desta forma (offset/contour/pattern/blend…), se houver.
    live: Option<Vec<VecPath>>,
    /// A forma AUTORADA — verts, handles, raios de quina, fill, stroke, subpaths, regra e a pilha
    /// de Live Path Effects. É o `VecPath` inteiro **de propósito**: um campo novo nele viaja para
    /// dentro da chave sozinho, então não há lista a esquecer de atualizar.
    path: Option<VecPath>,
    /// Parte linear (`a b c d`) do afim mundo→tela da câmera.
    cam: [f64; 4],
    /// Parte linear do afim mundo→tela **desta forma** (a câmera composta com a pose dela).
    screen: [f64; 4],
}

/// **A chave do memo.** Igual ⇒ os pixels que estão na textura servem para este frame.
#[derive(Clone, PartialEq)]
pub(crate) struct FxKey {
    /// A pilha JÁ RESOLVIDA em pixels — guardá-la resolvida (e não o componente) é o que faz o
    /// zoom invalidar sozinho: a mesma pilha noutro zoom é outra lista.
    pub(crate) ops: Vec<FxOpGpu>,
    pub(crate) w: u32,
    pub(crate) h: u32,
    drawn: FxDrawn,
}

/// **O que uma forma filtrada precisa neste frame**, resolvido pela 1ª varredura do `recook`.
///
/// Existe porque a decisão do ATLAS é sobre a CENA: só depois de conhecer o tamanho de todas as
/// formas que erraram o memo é que se sabe em quantos renders elas cabem. Sem isto o laço teria de
/// resolver cada forma duas vezes — e a 2ª resposta é a que poderia divergir.
pub(crate) struct Job {
    pub(crate) id: VecPathId,
    pub(crate) key: FxKey,
    /// O canto do scratch desta forma, em pixels de tela (a caixa dela mais a margem da pilha).
    /// **Fora da chave**: ver o doc do módulo (a translação não muda a arte dentro da célula).
    pub(crate) ex0: f64,
    pub(crate) ey0: f64,
}

/// A resolução por-forma da 1ª varredura: a pilha em pixels, a caixa, o tamanho, e o que é
/// desenhado. `None` = esta forma não pede FX neste frame (sem componente, pilha toda desligada, ou
/// sem caixa — uma forma vazia).
///
/// **Porta única:** o `recook` e os gates perguntam AQUI. A alternativa (o laço a resolver em linha)
/// é o que tornou o defeito invisível — a decisão do memo vivia dentro de uma função que precisa de
/// GPU, então nenhum teste headless a alcançava.
pub(crate) fn job_for(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
    live: &LiveGeometry,
    sil: &LiveGeometry,
    camera: Affine,
    id: VecPathId,
) -> Option<Job> {
    let filter = crate::fx_live::spec_of(sim, map, id)?;
    let ops = crate::fx_live::resolve_ops(&filter, camera);
    if ops.is_empty() {
        return None;
    }
    let (x0, y0, x1, y1) = ph2d_vec_render::path_screen_bounds(scene, xforms, live, id, camera)?;
    // A margem é da PILHA INTEIRA (as reaches somam ao longo dela) e assimétrica (uma sombra longa
    // para a direita não paga textura à esquerda). Porta única no passe.
    let (ml, mt, mr, mb) = stack_reach(&ops);
    let ex0 = (x0 - f64::from(ml)).floor();
    let ey0 = (y0 - f64::from(mt)).floor();
    let w = (((x1 + f64::from(mr)).ceil() - ex0).max(1.0) as u32).min(MAX_FX_SIDE);
    let h = (((y1 + f64::from(mb)).ceil() - ey0).max(1.0) as u32).min(MAX_FX_SIDE);
    let screen = ph2d_vec_render::path_to_screen(xforms, id, camera).as_coeffs();
    let cam = camera.as_coeffs();
    Some(Job {
        id,
        key: FxKey {
            ops,
            w,
            h,
            drawn: FxDrawn {
                sil: sil.get(&id).cloned(),
                live: live.get(&id).cloned(),
                path: scene.paths().iter().find(|p| p.id == id).cloned(),
                cam: [cam[0], cam[1], cam[2], cam[3]],
                screen: [screen[0], screen[1], screen[2], screen[3]],
            },
        },
        ex0,
        ey0,
    })
}

#[cfg(test)]
#[path = "fx_live_memo_tests.rs"]
mod tests;
