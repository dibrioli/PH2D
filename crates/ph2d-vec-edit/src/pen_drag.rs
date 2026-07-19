//! **O ARRASTO** de um ponto agarrado — módulo irmão do `PenTool` (LOC cap).
//!
//! Um bloco `impl PenTool` inerente pode viver em qualquer módulo da crate; a API pública
//! fica idêntica. O que mora aqui é o que acontece ENTRE o press e o release: mover uma
//! âncora (com snap e em grupo), girar um handle bézier (com a restrição do `VertexKind`)
//! e puxar a alça de raio de quina.

use super::{Part, PenTool, VecScene, VertexKind, corner_handle};

impl PenTool {
    /// Arrasto: puxa os handles do vértice em desenho, OU move o elemento agarrado.
    /// Devolve `true` se consumiu.
    ///
    /// `snap` só se aplica ao arrasto de ÂNCORA — um handle é uma tangente, encaixá-lo
    /// numa âncora vizinha só produziria curva torta. Sem snap, passe `&mut |p| p`.
    pub fn on_drag(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        snap: &mut dyn FnMut([f64; 2]) -> [f64; 2],
    ) -> bool {
        // Edição de ponto agarrado.
        if let Some(g) = self.grab {
            let xf = self.xf(g.path);
            let Some(inv) = xf.inverse() else {
                return false; // forma colapsada: nada a arrastar
            };
            let Some(path) = scene.path_mut(g.path) else {
                return false;
            };
            // Arrasto de ÂNCORA: se a agarrada está na multi-seleção, TODAS as
            // âncoras selecionadas transladam pelo mesmo delta (âncora + handles);
            // senão só a agarrada. Handles ficam de fora (são per-vértice).
            if g.part == Part::Anchor {
                let Some(grabbed) = path.vert(g.vert) else {
                    return false;
                };
                let p = snap(p);
                // O snap acontece no MUNDO; o delta desce pro espaço local do path.
                let gw = xf.apply(grabbed.anchor);
                let d = inv.apply_vec([p[0] - gw[0], p[1] - gw[1]]);
                let group = self.selected_verts.contains(&g.vert);
                for i in 0..path.total_verts() {
                    if i != g.vert && !(group && self.selected_verts.contains(&i)) {
                        continue;
                    }
                    let Some(v) = path.vert_mut(i) else { continue };
                    v.anchor = [v.anchor[0] + d[0], v.anchor[1] + d[1]];
                    v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
                    v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
                }
                return true;
            }
            // Um handle é uma tangente: sem snap, e no espaço local do path.
            let p = inv.apply(p);
            // O RAIO DE QUINA: o cursor projetado na bissetriz vira recuo, e o recuo vira
            // raio pela MESMA conta que o cozimento usa ao contrário. Sem snap (um raio
            // não encaixa em nada) e antes do `vert_mut` — o frame vem da quina inteira
            // (os dois vizinhos), não do vértice sozinho.
            if g.part == Part::Radius {
                let Some(frame) = corner_handle::frame_at_flat(path, g.vert) else {
                    return false; // a quina deixou de existir no meio do arrasto
                };
                let d = [p[0] - frame.anchor[0], p[1] - frame.anchor[1]];
                let proj = d[0] * frame.bisector[0] + d[1] * frame.bisector[1];
                let r = frame.radius_for_setback(proj + g.radius_offset);
                let Some(v) = path.vert_mut(g.vert) else {
                    return false;
                };
                // A alça muda o TAMANHO; o estilo (chanfro vs arredondado) sobrevive ao arrasto.
                v.set_corner_size(r);
                return true;
            }
            let Some(v) = path.vert_mut(g.vert) else {
                return false;
            };
            match g.part {
                Part::Anchor | Part::Radius => unreachable!("handled above"),
                // O handle oposto segue a restrição do tipo: Symmetric espelha,
                // Smooth mantém colinear preservando o comprimento, Corner é livre.
                Part::In => {
                    v.in_handle = p;
                    match v.kind {
                        VertexKind::Symmetric => v.out_handle = mirror(v.anchor, p),
                        VertexKind::Smooth => {
                            v.out_handle = colinear_opposite(v.anchor, p, v.out_handle)
                        }
                        VertexKind::Corner => {}
                    }
                }
                Part::Out => {
                    v.out_handle = p;
                    match v.kind {
                        VertexKind::Symmetric => v.in_handle = mirror(v.anchor, p),
                        VertexKind::Smooth => {
                            v.in_handle = colinear_opposite(v.anchor, p, v.in_handle)
                        }
                        VertexKind::Corner => {}
                    }
                }
            }
            return true;
        }
        // Arrasto de handle do vértice em desenho: o Pen cria handles simétricos
        // (Symmetric) — o clássico "arrasta pra curvar", quebrável depois (Alt).
        if self.dragging
            && let Some(id) = self.active
        {
            let p = self.to_local(id, p);
            let Some(path) = scene.path_mut(id) else {
                return false;
            };
            let Some(v) = path.verts.last_mut() else {
                return false;
            };
            v.out_handle = p;
            v.in_handle = mirror(v.anchor, p);
            v.kind = VertexKind::Symmetric;
            return true;
        }
        false
    }
}

fn mirror(anchor: [f64; 2], h: [f64; 2]) -> [f64; 2] {
    [2.0 * anchor[0] - h[0], 2.0 * anchor[1] - h[1]]
}

/// Handle oposto colinear (regra Smooth): aponta na direção contrária ao handle
/// movido, **preservando o próprio comprimento**. Se o handle movido coincidir
/// com a âncora (sem direção), devolve o oposto inalterado.
fn colinear_opposite(anchor: [f64; 2], moved: [f64; 2], opposite: [f64; 2]) -> [f64; 2] {
    let d = [moved[0] - anchor[0], moved[1] - anchor[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if l < 1e-9 {
        return opposite;
    }
    let t = [d[0] / l, d[1] / l];
    let o = [opposite[0] - anchor[0], opposite[1] - anchor[1]];
    let opp_len = (o[0] * o[0] + o[1] * o[1]).sqrt();
    [anchor[0] - t[0] * opp_len, anchor[1] - t[1] * opp_len]
}
