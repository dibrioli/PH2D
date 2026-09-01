//! **A cena do SOLDAR** — `PH2D_BUILD_SMOKE=81` (plano 39).
//!
//! Três casos lado a lado, um por espécie de encontro, para o artista poder ver a lei inteira num
//! ecrã e — sobretudo — **verificar** que o nó ficou partilhado.
//!
//! 1. **AS DUAS CURVAS QUE SE ENCONTRAM PELAS PONTAS** — o report do Enio (2026-09-01):
//!    *"ainda não consegue conectar as duas curvas … as linhas não compartilham o mesmo nó"*.
//!    Elas ficam a `4` unidades uma da outra: **perto para o ímã, longe para o olho**, senão o
//!    smoke não distingue *"soldou"* de *"já estava"*.
//! 2. **AS DUAS CURVAS QUE SE CRUZAM** — o caso de 31/08: partem-se em quatro arcos que partilham
//!    o nó do meio.
//! 3. **AS DUAS QUE NÃO SE TOCAM** — a cerca. Soldar não lhes mexe, e é isso que impede o verbo de
//!    arrastar para o meio tudo o que estiver seleccionado.
//!
//! ⚠️ **A ferramenta é armada em `Node`** (a seta branca): sem ela o artista não consegue fazer a
//! segunda metade do teste, que é **arrastar o nó** e ver os pedaços irem juntos.
//!
//! ⚠️ **As curvas são CURVAS, não retas.** Com retas em coordenadas redondas os dois lados de um
//! cruzamento calculam o mesmo ponto por acaso, e a cena não distinguiria uma solda de uma
//! coincidência — foi o mecanismo que fez três mutações sobreviverem aos primeiros gates.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Um vértice suave: âncora e as duas alças, em coordenadas de mundo.
fn c(a: [f64; 2], i: [f64; 2], o: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: a,
        in_handle: i,
        out_handle: o,
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    }
}

fn curva(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        id: 0,
        verts,
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(235, 235, 240, 255), 3.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    }
}

/// Monta a cena e arma a seta branca. Chamada uma vez, pelo roteador.
pub(crate) fn frame(app: &mut crate::App, f: u32) {
    if f != 0 {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;

    // 1. PONTA COM PONTA — o vão é de **2 unidades** de mundo.
    //
    // ⚠️ **É o ímã da tela que decide, e ele mede 10 PIXELS**: um vão pequeno em mundo sobrevive a
    // qualquer zoom razoável, e um vão grande deixaria a cena a depender de onde a câmera estava
    // quando o smoke abriu. ⛔ Zero não serve: duas pontas já coincidentes **já** são um nó, e a
    // cena não mostraria nada a acontecer.
    s.push_path(curva(vec![
        c([-360.0, -60.0], [-360.0, -60.0], [-300.0, -160.0]),
        c([-240.0, -100.0], [-280.0, -20.0], [-240.0, -100.0]),
    ]));
    s.push_path(curva(vec![
        c([-238.4, -101.2], [-238.4, -101.2], [-190.0, -170.0]),
        c([-120.0, -60.0], [-160.0, -10.0], [-120.0, -60.0]),
    ]));

    // 2. CRUZADAS — quatro arcos, um nó no meio.
    s.push_path(curva(vec![
        c([-40.0, -40.0], [-40.0, -40.0], [50.0, -160.0]),
        c([170.0, -100.0], [100.0, -20.0], [170.0, -100.0]),
    ]));
    s.push_path(curva(vec![
        c([30.0, -180.0], [30.0, -180.0], [130.0, -100.0]),
        c([70.0, 20.0], [10.0, -30.0], [70.0, 20.0]),
    ]));

    // 3. LONGE UMA DA OUTRA — a cerca.
    s.push_path(curva(vec![
        c([-360.0, 120.0], [-360.0, 120.0], [-300.0, 60.0]),
        c([-260.0, 120.0], [-300.0, 180.0], [-260.0, 120.0]),
    ]));
    s.push_path(curva(vec![
        c([-140.0, 120.0], [-140.0, 120.0], [-80.0, 60.0]),
        c([-40.0, 120.0], [-80.0, 180.0], [-40.0, 120.0]),
    ]));

    crate::render_loop::vector_bridge::set_mode(&mut gfx.tools, ph2d_tool_vector::DrawMode::Node);
    eprintln!(
        "[weld-smoke] 6 curvas montadas, em 3 pares, e a seta branca (Node) ARMADA.\n\
         [weld-smoke]  1) par de CIMA-ESQUERDA: as pontas quase se tocam. Clique numa, Shift+clique \
         na outra, botao 'Weld' -> nasce um ANEL VERDE no encontro: e' o no' partilhado\n\
         [weld-smoke]  2) par do MEIO: elas CRUZAM-SE. Weld parte-as em quatro pedacos, com o anel \
         no cruzamento\n\
         [weld-smoke]  3) par de BAIXO-ESQUERDA: longe uma da outra. Weld NAO lhes mexe (a cerca)\n\
         [weld-smoke]  ENTAO ARRASTE o ponto do anel: todos os pedacos tem de ir JUNTOS. Se algum \
         ficar para tras, o no' nao esta' soldado.\n\
         [weld-smoke]  (se o anel nao aparecer no par 1, afaste um pouco o zoom: o ima mede 10 \
         pixels de tela)"
    );
}

#[cfg(test)]
mod tests {
    use ph2d_vec_scene::{VecScene, VecXforms};

    /// Os mesmos vértices da cena, sem o `App` — a cena e o gate leem a MESMA geometria porque ela
    /// é escrita uma vez, aqui.
    fn pares() -> [(Vec<super::VecVertex>, Vec<super::VecVertex>); 3] {
        use super::c;
        [
            (
                vec![
                    c([-360.0, -60.0], [-360.0, -60.0], [-300.0, -160.0]),
                    c([-240.0, -100.0], [-280.0, -20.0], [-240.0, -100.0]),
                ],
                vec![
                    c([-238.4, -101.2], [-238.4, -101.2], [-190.0, -170.0]),
                    c([-120.0, -60.0], [-160.0, -10.0], [-120.0, -60.0]),
                ],
            ),
            (
                vec![
                    c([-40.0, -40.0], [-40.0, -40.0], [50.0, -160.0]),
                    c([170.0, -100.0], [100.0, -20.0], [170.0, -100.0]),
                ],
                vec![
                    c([30.0, -180.0], [30.0, -180.0], [130.0, -100.0]),
                    c([70.0, 20.0], [10.0, -30.0], [70.0, 20.0]),
                ],
            ),
            (
                vec![
                    c([-360.0, 120.0], [-360.0, 120.0], [-300.0, 60.0]),
                    c([-260.0, 120.0], [-300.0, 180.0], [-260.0, 120.0]),
                ],
                vec![
                    c([-140.0, 120.0], [-140.0, 120.0], [-80.0, 60.0]),
                    c([-40.0, 120.0], [-80.0, 180.0], [-40.0, 120.0]),
                ],
            ),
        ]
    }

    fn solda(par: usize, ima: f64) -> (VecScene, ph2d_vec_edit::PenTool) {
        let [p1, p2] = {
            let (a, b) = pares()[par].clone();
            [a, b]
        };
        let mut scene = VecScene::new();
        let mut hist = ph2d_vec_edit::History::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let a = scene.push_path(super::curva(p1));
        let b = scene.push_path(super::curva(p2));
        pen.select_many(&[a, b]);
        crate::vec_weld::apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), ima);
        (scene, pen)
    }

    /// ⭐⭐⭐ **A CENA ENSINA O QUE ELA DIZ** — os três pares, com o ímã que o artista tem.
    ///
    /// ⚠️ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (a lei do §5.0, paga pela `=15` da física). Aqui o texto promete três coisas diferentes, e
    /// cada uma é uma linha deste gate.
    #[test]
    fn the_three_pairs_of_the_scene_do_what_the_message_promises() {
        // 1 — as pontas encontram-se: NADA se dissolve e nasce UM nó.
        let (s1, pen1) = solda(0, 3.0);
        assert_eq!(s1.paths().len(), 2, "o par 1 nao devia dissolver nada");
        assert_eq!(
            pen1.welded_nodes(&s1).len(),
            1,
            "o par 1 tem de dar UM no' partilhado — o anel do smoke"
        );
        // 2 — elas cruzam-se: quatro arcos, um nó.
        let (s2, pen2) = solda(1, 3.0);
        assert_eq!(s2.paths().len(), 4, "o par 2 tem de dar QUATRO arcos");
        assert_eq!(pen2.welded_nodes(&s2).len(), 1);
        // 3 — a cerca: longe uma da outra, nada acontece.
        let (s3, pen3) = solda(2, 3.0);
        assert_eq!(s3.paths().len(), 2);
        assert!(
            pen3.welded_nodes(&s3).is_empty(),
            "o par 3 e' a CERCA: soldar nao lhe mexe"
        );
    }
}
