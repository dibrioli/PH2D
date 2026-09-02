//! **A cena do SOLDAR** — `PH2D_BUILD_SMOKE=81` (plano 39).
//!
//! Três casos lado a lado, um por espécie de encontro, para o artista poder ver a lei inteira num
//! ecrã e — sobretudo — **verificar** que o nó ficou partilhado.
//!
//! 1. **AS DUAS CURVAS QUE SE ENCONTRAM PELAS PONTAS** — o report do Enio (2026-09-01):
//!    *"ainda não consegue conectar as duas curvas … as linhas não compartilham o mesmo nó"*.
//!    As pontas ficam a **2 unidades** uma da outra — dentro do ímã em qualquer zoom razoável, e
//!    ⛔ **não zero**: duas pontas já coincidentes **já** são um nó, e a cena não mostraria nada a
//!    acontecer.
//! 2. **AS DUAS CURVAS QUE SE CRUZAM** — o caso de 31/08: partem-se em quatro arcos que partilham
//!    o nó do meio.
//! 3. **AS DUAS QUE NÃO SE TOCAM** — a cerca. Soldar não lhes mexe, e é isso que impede o verbo de
//!    arrastar para o meio tudo o que estiver seleccionado.
//!
//! ⛔⛔ **A ferramenta é armada em `Select` (a seta PRETA), e a 1ª versão desta cena armava `Node`
//! — o que a tornou impossível de seguir** (report do Enio, 2026-09-01: *"o smoke não tinha nada do
//! que vc falou e não funcionou ainda o Weld"*).
//!
//! ⚠️ **Medido:** somar uma forma à selecção é `Shift`+clique, e no modo **Node** esse gesto é
//! tentado PRIMEIRO como *"alterna este PONTO na multi-selecção de pontos"*
//! (`input_dispatch.rs`, raio de 10 px). Num par de curvas que se encontram pelas pontas, o sítio
//! natural de clicar **é** um ponto ⇒ o segundo traço nunca entrava na selecção, o Weld via UM
//! caminho, não achava cruzamento nenhum e não fazia nada. *O motor estava certo e o gesto que eu
//! pedi era inexecutável.*
//!
//! ⚠️ **E o par 1 nasce SELECCIONADO**: com a selecção vazia a seção Path não é pintada (ela é um
//! comando sobre a selecção), então o botão **Weld** não estaria sequer na tela quando o smoke
//! abre — e a primeira coisa que a cena pede é carregar nele.
//!
//! ⚠️ **O anel do nó é chrome de NÓ**: ele desenha-se com a seta branca, não com a preta
//! (`vec_overlay`: o modo Select não mostra âncoras). Por isso a cena manda trocar de ferramenta
//! depois de soldar — que é o mesmo clique de que o arrasto precisa.
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

/// ⭐⭐⭐ **OS TRÊS PARES — a porta ÚNICA da geometria da cena.**
///
/// ⚠️ **O produto e o gate leem esta MESMA lista.** A 1.ª redacção tinha a cena a empurrar os
/// vértices e o gate a reescrevê-los no módulo de teste: *duas cópias da mesma fixtura, e a que o
/// artista vê é a que pode envelhecer sozinha* — o gate ficaria verde sobre uma cena que já não
/// existe.
///
/// 1. **PONTA COM PONTA** (vão de 2 unidades) · 2. **CRUZADAS** · 3. **LONGE** (a cerca).
fn pares() -> [[Vec<VecVertex>; 2]; 3] {
    [
        [
            vec![
                c([-360.0, -60.0], [-360.0, -60.0], [-300.0, -160.0]),
                c([-240.0, -100.0], [-280.0, -20.0], [-240.0, -100.0]),
            ],
            vec![
                c([-238.4, -101.2], [-238.4, -101.2], [-190.0, -170.0]),
                c([-120.0, -60.0], [-160.0, -10.0], [-120.0, -60.0]),
            ],
        ],
        [
            vec![
                c([-40.0, -40.0], [-40.0, -40.0], [50.0, -160.0]),
                c([170.0, -100.0], [100.0, -20.0], [170.0, -100.0]),
            ],
            vec![
                c([30.0, -180.0], [30.0, -180.0], [130.0, -100.0]),
                c([70.0, 20.0], [10.0, -30.0], [70.0, 20.0]),
            ],
        ],
        [
            vec![
                c([-360.0, 120.0], [-360.0, 120.0], [-300.0, 60.0]),
                c([-260.0, 120.0], [-300.0, 180.0], [-260.0, 120.0]),
            ],
            vec![
                c([-140.0, 120.0], [-140.0, 120.0], [-80.0, 60.0]),
                c([-40.0, 120.0], [-80.0, 180.0], [-40.0, 120.0]),
            ],
        ],
    ]
}

/// Monta a cena, activa o vetor e arma a seta PRETA. Chamada uma vez, pelo roteador.
pub(crate) fn frame(app: &mut crate::App, f: u32) {
    if f != 0 {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;

    let mut ids: Vec<u64> = Vec::new();
    for [a, b] in pares() {
        ids.push(s.push_path(curva(a)));
        ids.push(s.push_path(curva(b)));
    }
    let primeiro_par = ids[..2].to_vec();

    // ⚠️ **A ferramenta VETOR tem de estar ACTIVA, e `set_mode` não a activa** — ele só escolhe o
    // modo DENTRO dela. Sem isto o painel do vetor pode nem estar em cena, e então a contagem de
    // selecção que ele publica é `0` e a seção Path (com o Weld) não é pintada.
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    crate::render_loop::vector_bridge::set_mode(&mut gfx.tools, ph2d_tool_vector::DrawMode::Select);
    app.vec_pen.select_many(&primeiro_par);
    eprintln!(
        "[weld-smoke] 6 curvas em 3 pares, a seta PRETA armada, e o 1.o par JA' SELECCIONADO.\n\
         [weld-smoke]  1) carregue em 'Weld' no painel -> as duas pontas do par de cima-esquerda\n\
         [weld-smoke]     passam a ser UM ponto\n\
         [weld-smoke]  2) pegue a SETA BRANCA (Node): nasce um ANEL VERDE no encontro -- e' ele que\n\
         [weld-smoke]     diz 'aqui as duas sao uma so'. ARRASTE-O: as duas curvas vao juntas\n\
         [weld-smoke]  3) volte a' seta PRETA, clique numa curva do par do MEIO e Shift+clique na\n\
         [weld-smoke]     outra; 'Weld' parte-as em quatro arcos com o anel no cruzamento -- e as\n\
         [weld-smoke]     DUAS viram UMA SO' linha na Hierarquia, com um gizmo so'\n\
         [weld-smoke]  4) o par de BAIXO esta' longe um do outro: 'Weld' NAO lhes mexe (a cerca)\n\
         [weld-smoke]  DEU ERRADO SE: nao aparece anel nenhum, ou ao arrastar um pedaco fica para tras"
    );
}

#[cfg(test)]
mod tests {
    use ph2d_vec_scene::{VecScene, VecXforms};

    fn solda(par: usize, ima: f64) -> (VecScene, ph2d_vec_edit::PenTool) {
        // ⚠️ **A MESMA porta que a cena usa** — a fixtura não tem uma segunda cópia.
        let [p1, p2] = super::pares()[par].clone();
        let mut scene = VecScene::new();
        let mut hist = ph2d_vec_edit::History::default();
        let mut pen = ph2d_vec_edit::PenTool::default();
        let a = scene.push_path(super::curva(p1));
        let b = scene.push_path(super::curva(p2));
        pen.select_many(&[a, b]);
        crate::vec_weld::apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), ima);
        (scene, pen)
    }

    /// ⛔⛔ **A CENA ARMA A FERRAMENTA EM QUE O GESTO QUE ELA PEDE EXISTE.**
    ///
    /// Report do Enio (2026-09-01): *"o smoke não tinha nada do que vc falou e não funcionou ainda
    /// o Weld"*. A 1.ª versão armava `Node` e mandava `Shift`+clicar para somar a 2.ª curva — e no
    /// modo Node esse gesto é tentado primeiro como *"alterna este PONTO"* (raio de 10 px), que é
    /// exactamente onde se clica num par de curvas que se encontram pelas pontas. ⇒ o segundo traço
    /// nunca entrava na selecção e o Weld via UM caminho.
    ///
    /// ⚠️ **E a pré-selecção não é conforto**: sem selecção a seção Path não é pintada, então o
    /// botão que a cena manda carregar **não estaria na tela**.
    #[test]
    fn the_scene_arms_the_tool_whose_gesture_it_asks_for() {
        // ⚠️ **Só a metade de PRODUTO do ficheiro.** A 1.ª redacção varria o ficheiro inteiro e
        // reprovou sobre produto CORRECTO: a própria mensagem de erro deste gate contém o literal
        // que ele proíbe. *Um gate textual que se lê a si mesmo acusa-se a si mesmo* — é a mesma
        // família do que exige descascar comentários.
        let inteiro: &str = include_str!("weld_smoke.rs");
        let src = &inteiro[..inteiro.find("#[cfg(test)]").expect("o modulo de teste")];
        assert!(
            // ⚠️ **Sem a vírgula nem o parêntese**: o `cargo fmt` colapsa e re-expande a chamada
            // conforme o comprimento da linha, e um gate ancorado na PONTUAÇÃO reprova sobre
            // produto correcto no primeiro `fmt` — foi o que aconteceu ao escrevê-lo.
            src.contains("DrawMode::Select"),
            "a cena tem de armar a seta PRETA: e' nela que Shift+clique SOMA uma forma"
        );
        assert!(
            !src.contains("DrawMode::Node"),
            "a cena voltou a armar o modo Node — o Shift+clique dela deixa de somar formas"
        );
        assert!(
            src.contains("app.vec_pen.select_many(&primeiro_par);"),
            "sem a pre-seleccao a seccao Path nao e' pintada, e o botao Weld nao esta' na tela"
        );
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
        assert_eq!(
            s2.paths().len(),
            1,
            "o par 2 tem de dar UM objecto (o report de 2026-09-02)"
        );
        assert_eq!(
            s2.paths()[0].contour_count(),
            4,
            "e' UM objecto com QUATRO arcos dentro"
        );
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
