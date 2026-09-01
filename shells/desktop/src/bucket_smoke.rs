//! **A cena do BALDE** — `PH2D_BUILD_SMOKE=82` (plano 40).
//!
//! Três grupos, um por espécie de fronteira, para o artista ver a lei inteira num ecrã:
//!
//! 1. **QUATRO LINHAS SOLTAS que se cruzam** — o pedido do Enio (*"linhas sobrepostas"*). Nenhuma
//!    delas é fechada e nenhuma tem dentro; é exactamente o caso que o Shape Builder **não sabe
//!    exprimir**, porque `região(M) = ∩M − ∪¬M` não se define para um traço aberto.
//! 2. **DOIS CÍRCULOS SOBREPOSTOS** — três regiões (a lente do meio e as duas luas). Cada uma é um
//!    clique diferente, e é o que mostra que a face é a MENOR que contém o ponto.
//! 3. **UM CÍRCULO ATRAVESSADO POR UMA RECTA** — duas metades, e a fronteira delas é **curva**. É
//!    a prova de que a forma sai em bézier: um balde que traçasse pixels (o do Inkscape) devolveria
//!    um polígono, e ampliar mostraria as facetas.
//!
//! ⚠️ **A ferramenta é armada no BALDE e a tinta corrente já é opaca** (o azul de fábrica): com
//! `alpha == 0` este app entende *"sem preenchimento"*, e o balde recusa-se — a cena tem de abrir
//! num estado em que o primeiro clique faz alguma coisa.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Uma quina — alças em cima da âncora ⇒ o segmento é uma RECTA.
fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn traco(verts: Vec<VecVertex>, closed: bool) -> VecPath {
    VecPath {
        id: 0,
        verts,
        closed,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(235, 235, 240, 255), 3.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    }
}

/// ⭐ **OS TRÊS GRUPOS — a porta ÚNICA da geometria da cena**, lida pelo produto e pelo gate.
pub(crate) fn grupos() -> Vec<(Vec<VecVertex>, bool)> {
    let mut out: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    // 1. Quatro linhas soltas: o miolo é um quadrado de 80×80.
    let a = 200.0;
    out.push((vec![v(-360.0 - a, -60.0), v(-360.0 + a, -60.0)], false));
    out.push((vec![v(-360.0 - a, 20.0), v(-360.0 + a, 20.0)], false));
    out.push((vec![v(-400.0, -20.0 - a), v(-400.0, -20.0 + a)], false));
    out.push((vec![v(-320.0, -20.0 - a), v(-320.0, -20.0 + a)], false));
    // 2. Dois círculos sobrepostos: três regiões.
    for cx in [-40.0_f64, 60.0] {
        out.push((ph2d_vec_scene::ellipse([cx, -20.0], 70.0, 70.0).verts, true));
    }
    // 3. Um círculo atravessado: duas metades de fronteira CURVA.
    out.push((
        ph2d_vec_scene::ellipse([280.0, -20.0], 70.0, 70.0).verts,
        true,
    ));
    out.push((vec![v(180.0, -20.0), v(380.0, -20.0)], false));
    out
}

/// Monta a cena, activa o vetor e arma o BALDE. Chamada uma vez, pelo roteador.
pub(crate) fn frame(app: &mut crate::App, f: u32) {
    if f != 0 {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (verts, closed) in grupos() {
        gfx.vec_scene.push_path(traco(verts, closed));
    }
    // ⚠️ `set_mode` escolhe o modo DENTRO da ferramenta e **não a activa** — sem esta linha o
    // painel do vetor pode nem estar em cena.
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    crate::render_loop::vector_bridge::set_mode(&mut gfx.tools, ph2d_tool_vector::DrawMode::Bucket);
    eprintln!(
        "[bucket-smoke] 3 grupos montados e o BALDE armado.\n\
         [bucket-smoke]  1) passe o cursor no MIOLO das quatro linhas (esq.): a regiao acende na\n\
         [bucket-smoke]     cor de preenchimento. Clique -> nasce uma forma ali\n\
         [bucket-smoke]  2) nos DOIS CIRCULOS (centro): a lente do meio e as duas luas sao TRES\n\
         [bucket-smoke]     regioes diferentes. Encha as tres\n\
         [bucket-smoke]  3) no CIRCULO CORTADO (dir.): as duas metades tem borda CURVA -- amplie e\n\
         [bucket-smoke]     confira que ela e' curva mesmo, nao uma escada de segmentos\n\
         [bucket-smoke]  A forma nasce ATRAS das linhas: elas continuam visiveis por cima.\n\
         [bucket-smoke]  Ctrl+Z desfaz cada preenchimento.\n\
         [bucket-smoke]  DEU ERRADO SE: nada acende; ou acende uma regiao e nasce outra; ou a\n\
         [bucket-smoke]  forma tapa as linhas"
    );
}

#[cfg(test)]
mod tests {
    /// ⭐⭐⭐ **A CENA ENSINA O QUE ELA DIZ** — os três grupos, medidos pela lei que o clique usa.
    ///
    /// ⚠️ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0). Cada promessa da mensagem é uma linha deste gate.
    #[test]
    fn the_three_groups_of_the_scene_do_what_the_message_promises() {
        let r = ph2d_vec_fill::rede(&super::grupos());
        // 1 — o miolo das quatro linhas é um quadrado de 80×80.
        let miolo = r
            .face_em([-360.0, -20.0])
            .expect("o miolo das quatro linhas");
        assert!(
            (miolo.area - 6400.0).abs() < 1.0,
            "o miolo e' 80x80: {}",
            miolo.area
        );
        assert_eq!(miolo.arcos.len(), 4, "quatro arcos inteiros");
        // 2 — a lente do meio é MENOR que qualquer das luas.
        let lente = r.face_em([10.0, -20.0]).expect("a lente do meio");
        let lua = r.face_em([-90.0, -20.0]).expect("a lua da esquerda");
        assert!(
            lente.area < lua.area,
            "a lente ({}) tem de ser menor que a lua ({})",
            lente.area,
            lua.area
        );
        // 3 — a metade de cima do círculo cortado tem fronteira CURVA.
        let meia = r.face_em([280.0, 20.0]).expect("a metade de cima");
        let g = r.geometria(&meia);
        assert!(
            g.iter().any(|v| v.out_handle != v.anchor),
            "a metade saiu sem alcas — isto seria um poligono"
        );
        assert!(
            (meia.area - 7697.0).abs() < 120.0,
            "meia bola de raio 70 mede {}",
            meia.area
        );
    }
}
