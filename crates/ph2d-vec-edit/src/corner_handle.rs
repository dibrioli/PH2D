//! A alça do **raio de quina** (Live Corners) — módulo irmão do `PenTool` (LOC cap).
//!
//! Uma alça por quina do path selecionado, correndo pela **bissetriz**: arrastar para
//! dentro arredonda o canto, arrastar de volta até a âncora o afia de novo. É o widget do
//! Illustrator (Live Corners) / o Corner Tool do Affinity.
//!
//! ## As três regras que este módulo existe para não quebrar
//!
//! 1. **Uma posição só.** [`handle_pos_local`] é chamada pelo DESENHO e pelo HIT-TEST — as
//!    duas, sempre. E o raio da bolinha é a constante única
//!    [`ph2d_vec_scene::corner_live::CORNER_HANDLE_R_PX`], que mora na crate que as duas
//!    pontas veem. Dois números fariam o usuário clicar no meio da bolinha e não pegar
//!    nada — e a tela estaria certa, que é o sintoma mais enlouquecedor que existe.
//!
//! 2. **A alça só existe onde o cozimento vai arredondar.** O predicado é o MESMO
//!    ([`ph2d_vec_scene::corner_live::corner_at`]): um vértice suave, uma quina colinear ou
//!    a ponta de um caminho aberto não têm alça — porque não têm quina. Uma alça que
//!    aparece e não faz nada é pior que alça nenhuma.
//!
//! 3. **O arrasto é RELATIVO.** Ver `Grab::radius_offset` no `lib.rs`.

use ph2d_vec_scene::corner_live::{CornerFrame, CornerHandleView};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// Onde a alça do vértice `i` é desenhada, em coordenadas **locais** do path.
///
/// **`park + setback`, e o `park` é um deslocamento CONSTANTE** — não um piso.
///
/// A alça de uma quina afiada (recuo 0) não pode ser desenhada em cima da âncora: sumiria
/// dentro dela e brigaria com o hit-test do vértice. Então ela fica `park` adiante, sobre
/// a bissetriz.
///
/// O reflexo é fazer disso um PISO (`max(setback, park)`) — e é errado, de um jeito que só
/// aparece no dedo. O arrasto lê `setback = projeção_do_cursor − park`; com o piso, agarrar
/// a alça estacionada e levar o cursor a 2,0 deixa a bolinha em 1,86 (2,0 − park): ela
/// **escorrega do dedo**, e o usuário vê a alça fugir enquanto arrasta. Com o park como
/// deslocamento constante, o desenho (`park + setback`) e a leitura (`− park`) se cancelam,
/// a bolinha pousa exatamente no cursor, e o raio ainda cresce de zero.
///
/// A bolinha fica, portanto, um pouco ADIANTE do ponto onde a curva de fato recua — é o
/// widget flutuando um respiro fora do arco, como no Illustrator. Quem manda na geometria é
/// o `setback`; a alça é a asa dele.
#[must_use]
pub fn handle_pos_local(frame: &CornerFrame, park: f64) -> [f64; 2] {
    frame.handle_at(park + frame.setback)
}

/// A quina do vértice PLANO `i` deste path, se existir. Traduz o índice plano (que o
/// hit-test e a seleção usam) para o `(contorno, índice local)` que o motor quer.
#[must_use]
pub fn frame_at_flat(path: &VecPath, i: usize) -> Option<CornerFrame> {
    let (c, local) = path.locate_vert(i)?;
    let (verts, closed) = path.contour(c)?;
    ph2d_vec_scene::corner_live::corner_at(verts, closed, local)
}

/// Todas as alças de raio de um path, no MUNDO — o que o passe de render desenha.
///
/// `park` (em unidades de mundo) e o afim vêm da shell, que é quem sabe do zoom e da
/// câmera; a geometria vem do documento. Um path sem quina nenhuma devolve vazio.
#[must_use]
pub fn view(
    scene: &VecScene,
    id: VecPathId,
    xf: &ph2d_vec_scene::Xform,
    park: f64,
) -> Vec<CornerHandleView> {
    let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    (0..path.total_verts())
        .filter_map(|i| {
            let frame = frame_at_flat(path, i)?;
            Some(CornerHandleView {
                pos: xf.apply(handle_pos_local(&frame, park)),
                anchor: xf.apply(frame.anchor),
                rounded: frame.setback > 0.0,
                vert: i,
            })
        })
        .collect()
}
