//! **A cena do TRIM** — `PH2D_BUILD_SMOKE=80` (plano 38).
//!
//! Quatro casos lado a lado, um por espécie de fronteira, para o artista poder ver a lei inteira
//! num ecrã:
//!
//! 1. **A CRUZ** — duas retas sobrepostas. É o *"entre linhas sobrepostas"* do pedido: aparar um
//!    toco tira do cruzamento até à ponta. ⚠️ Ela é desenhada em coordenadas REDONDAS de propósito
//!    — foi exactamente essa a fixtura que apanhou o falso negativo do `seg_cross` (uma travessia
//!    sobre uma amostra da poligonal era recusada), e é o que um artista de facto desenha.
//! 2. **O POLÍGONO** — sem cruzamento nenhum. É o *"entre pontos"*: aparar tira **um lado**.
//!    ⭐ É aqui que se vê a diferença para o Fusion, onde um contorno sem travessia é apagado
//!    INTEIRO (a queixa nº 1 dele) — aqui os nós são fronteira, então sai um lado.
//! 3. **O ZIGUE-ZAGUE aberto** — aparar no meio parte-o em **dois**.
//! 4. **A RETA SOLTA** — não cruza nada e tem dois nós só, então o pedaço é a peça toda: aparar
//!    **apaga-a**. É a resposta do Fusion, e a única em que a ferramenta remove o objecto.
//!
//! ⚠️ **A ferramenta é ARMADA pela cena** (`DrawMode::Trim`): sem isto o artista abre o smoke, não
//! vê realce nenhum e conclui que a feature não existe.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Uma quina em `(x, y)` — alças coincidentes com a âncora ⇒ o segmento é uma RETA.
fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn linha(verts: Vec<VecVertex>, closed: bool) -> VecPath {
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

/// Monta a cena e arma a ferramenta. Chamada uma vez, pelo roteador.
pub(crate) fn frame(app: &mut crate::App, f: u32) {
    if f != 0 {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;

    // 1. A CRUZ — coordenadas redondas, e é de propósito.
    s.push_path(linha(vec![v(-360.0, -120.0), v(-200.0, -120.0)], false));
    s.push_path(linha(vec![v(-280.0, -200.0), v(-280.0, -40.0)], false));

    // 2. O POLÍGONO — sem cruzamento nenhum: as fronteiras são os NÓS.
    s.push_path(linha(
        vec![
            v(-100.0, -200.0),
            v(20.0, -200.0),
            v(20.0, -80.0),
            v(-100.0, -80.0),
        ],
        true,
    ));

    // 3. O ZIGUE-ZAGUE aberto — aparar no meio parte-o em dois.
    s.push_path(linha(
        vec![
            v(120.0, -80.0),
            v(180.0, -200.0),
            v(240.0, -80.0),
            v(300.0, -200.0),
        ],
        false,
    ));

    // 4. A RETA SOLTA — a peça toda; aparar apaga-a.
    s.push_path(linha(vec![v(-360.0, 80.0), v(-200.0, 80.0)], false));

    crate::render_loop::vector_bridge::set_mode(&mut gfx.tools, ph2d_tool_vector::DrawMode::Trim);
    eprintln!(
        "[trim-smoke] 5 caminhos montados e a ferramenta ARMADA. Passe o cursor sobre uma linha: o \
         pedaco entre as duas fronteiras mais proximas acende a VERMELHO; clique e ele some.\n\
         [trim-smoke]  1) a CRUZ (esq., em cima): apare um toco — vai do cruzamento ate' a ponta\n\
         [trim-smoke]  2) o RECTANGULO: sem cruzamento, as fronteiras sao os NOS -> sai UM LADO\n\
         [trim-smoke]  3) o ZIGUE-ZAGUE: apare o meio -> ele parte-se em DOIS\n\
         [trim-smoke]  4) a RETA SOLTA (esq., em baixo): e' a peca toda -> ela SOME"
    );
}
