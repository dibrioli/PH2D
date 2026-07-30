//! **A cena do ALCANCE DO NÓ** — `PH2D_BUILD_SMOKE=43` (plano 25 §6, W3a).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não entra no modo Node**: o gesto que este smoke prova começa no pill,
//! e uma cena que armasse o modo pularia justamente a costura que ela existe para exercer (a
//! cicatriz que o `impasto_smoke` do Painter prega).
//!
//! O que ela monta são **arcos de nó do meio bem marcado** — a única forma em que *"a curva ficou"*
//! e *"a curva morreu com o ponto"* se distinguem a olho: numa reta as duas respostas são iguais.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Escala do arco, em unidades de mundo — grande o bastante para o nó do meio e as âncoras das
/// pontas ficarem bem separados sob o cursor.
const S: f64 = 1.2;
/// Largura do traço das referências.
const STROKE_W: f64 = 0.05;

/// Um arco de 3 nós subindo e descendo a 45°, deslocado em `dy`.
///
/// ⚠️ As tangentes das PONTAS **não** são paralelas, e isso é o desenho: com elas paralelas nenhuma
/// cúbica alcança o ápice e o refit degrada honestamente para a corda — o artista veria a curva
/// achatar e concluiria que a preservação não funciona (ver o gate `node_reach.rs`).
fn arc(dy: f64, rgb: [u8; 3]) -> VecPath {
    let v = |a: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor: [a[0] * S, a[1] * S + dy],
        in_handle: [i[0] * S, i[1] * S + dy],
        out_handle: [o[0] * S, o[1] * S + dy],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    let mut p = VecPath {
        verts: vec![
            v([-1.0, 0.0], [-1.55, -0.55], [-0.45, 0.55]),
            v([0.0, 1.0], [-0.4, 1.0], [0.4, 1.0]),
            v([1.0, 0.0], [0.45, 0.55], [1.55, -0.55]),
        ],
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(
        Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
        STROKE_W,
    ));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    // DOIS arcos iguais: um para apagar o nó, outro de CONTROLE. Sem controle, "a curva ficou" e
    // "ela sempre foi assim" são indistinguíveis numa screenshot.
    for (dy, rgb) in [(1.6_f64, [70u8, 150, 220]), (0.0, [120, 190, 120])] {
        scene.push_path(arc(dy, rgb));
    }
    // E um terceiro para o arrasto de SEGMENTO — ele fica com a topologia intacta, e é o que se
    // compara com os outros dois (mesma contagem de nós no fim do smoke).
    scene.push_path(arc(-1.6, [220, 150, 90]));
}

fn announce(app: &mut crate::App) {
    let n = app.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
    eprintln!(
        "[smoke] alcance do no' (plano 25 §6): {n} arcos de TRES nos (azul/verde/laranja), cada um \
         com o no' do meio no alto. Nenhum modo esta' armado: o gesto comeca no pill. (1) na \
         fileira TOOL clique **Node** (a seta branca); (2) clique o arco VERDE e depois o no' do \
         MEIO dele; (3) aperte **Delete**: o no' some e **a curva TEM DE FICAR** -- compare com o \
         azul, que nao foi tocado; se o verde virar quase uma reta entre as duas pontas, PARE: o \
         refit nao esta' a correr; (4) desfaca com Ctrl+Z; (5) no arco LARANJA, pressione sobre a \
         CURVA (longe de qualquer no') e ARRASTE: o trecho tem de dobrar seguindo o dedo, com as \
         duas ancoras PARADAS e **sem nascer no' nenhum** -- se aparecer um ponto novo sob o \
         cursor, o press voltou a inserir; (6) o ponto que voce pegou tem de ficar debaixo do dedo \
         durante todo o arrasto (nao 'escorregar' ao longo da curva); (7) troque para o pill \
         **Pen** e clique sobre a curva do laranja: AGORA um no' novo tem de nascer -- a insercao \
         nao se perdeu, mudou de ferramenta (a divisao do Illustrator: a seta branca reforma, a \
         Pen acrescenta); (8) volte ao Node, selecione o no' de uma PONTA do azul e Delete: ali \
         nao ha o que preservar, o caminho so' fica mais curto -- e' a resposta honesta."
    );
}
