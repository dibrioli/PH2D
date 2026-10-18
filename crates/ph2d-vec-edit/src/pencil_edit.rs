//! **O EDIT SHAPE no lápis** (F7e) — a forma TIPADA que o QuickShape cometeu sobrevive ao soltar,
//! e o artista pode afiná-la pelos nós dela ([`crate::quickshape::edit`]) até ao gesto seguinte.
//!
//! O ciclo é o do Procreate (o manual, R4): ao soltar, a oferta (o botão na barra de ações da
//! seleção); o botão ABRE os nós; arrastar um nó afina, arrastar o corpo escala; um toque fora,
//! outro traço, `Esc`, outra ferramenta ou outra seleção FECHAM. Cada afinação é um passo de undo
//! (o diff que a shell tira ao soltar o botão).
//!
//! ⚠️ A forma vive em MUNDO; o caminho, depois de assentado, em LOCAL — quem escreve converte pelo
//! afim do caminho. E a forma só vale enquanto o caminho for o que ela escreveu: qualquer outro
//! escritor (a edição de nós, um flip, apagar) a desfaz; o undo dentro do Edit Shape reencontra a
//! forma que tinha escrito aquela geometria ([`ShapeEdit::history`]).

use super::{Pencil, arrow_ends};
use crate::quickshape::{QuickShape, edit};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, Xform};

type P2 = [f64; 2];

/// O que a barra de ações mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeEditState {
    /// Nada a afinar.
    None,
    /// A forma acabou de nascer: o botão oferece o Edit Shape.
    Offered,
    /// Os nós estão à vista.
    Open,
}

/// A forma afinável e o que se lhe escreveu.
pub(super) struct ShapeEdit {
    path: VecPathId,
    /// Cada forma que este Edit Shape escreveu; `at` é a de agora (o undo anda para trás nela).
    history: Vec<QuickShape>,
    at: usize,
    /// O recuo da seta em cada extremo (mundo), o do nascimento: a ponta cai no bico.
    inset: [f64; 2],
    open: bool,
    grab: Option<Grab>,
}

/// O arrasto em curso: o nó (ou `None` = o corpo), a forma de quando foi agarrado e onde.
struct Grab {
    node: Option<usize>,
    shape: QuickShape,
    pointer: P2,
    node_at: P2,
    last: P2,
    regular: bool,
    xf: Xform,
}

impl ShapeEdit {
    fn shape(&self) -> &QuickShape {
        &self.history[self.at]
    }
}

/// O caminho que a forma escreve, em MUNDO (a seta recua pelas pontas, como no lápis).
fn written(shape: &QuickShape, inset: [f64; 2]) -> VecPath {
    let p = shape.to_path();
    if inset == [0.0; 2] {
        return p;
    }
    ph2d_vec_scene::trim_path(&p, inset[0], inset[1]).unwrap_or(p)
}

fn map_vert(v: &VecVertex, xf: &Xform) -> VecVertex {
    let mut v = *v;
    v.anchor = xf.apply(v.anchor);
    v.in_handle = xf.apply(v.in_handle);
    v.out_handle = xf.apply(v.out_handle);
    v
}

/// A geometria de `path` é a que `shape` escreve? Em mundo, à tolerância de 1e-6 do tamanho dela
/// (o assentamento da origem soma e subtrai o mesmo número; um escritor de verdade move mais).
fn same(path: &VecPath, xf: &Xform, shape: &QuickShape, inset: [f64; 2]) -> bool {
    let w = written(shape, inset);
    if w.closed != path.closed || w.verts.len() != path.verts.len() {
        return false;
    }
    let size = w
        .verts
        .iter()
        .flat_map(|v| [v.anchor[0].abs(), v.anchor[1].abs()])
        .fold(1.0_f64, f64::max);
    let tol = 1e-6 * size;
    let near = |a: P2, b: P2| (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol;
    w.verts.iter().zip(&path.verts).all(|(a, b)| {
        let b = map_vert(b, xf);
        near(a.anchor, b.anchor)
            && near(a.in_handle, b.in_handle)
            && near(a.out_handle, b.out_handle)
    })
}

impl Pencil {
    /// O Edit Shape agora.
    #[must_use]
    pub fn shape_edit_state(&self) -> ShapeEditState {
        match &self.edit {
            None => ShapeEditState::None,
            Some(e) if e.open => ShapeEditState::Open,
            Some(_) => ShapeEditState::Offered,
        }
    }

    /// O botão: oferecido ⇒ abre; aberto ⇒ fecha (a oferta fica, como no Procreate).
    pub fn toggle_shape_edit(&mut self) {
        if let Some(e) = self.edit.as_mut() {
            e.open = !e.open;
        }
    }

    /// O caminho que o Edit Shape afina (aberto ou oferecido).
    #[must_use]
    pub fn shape_edit_path(&self) -> Option<VecPathId> {
        self.edit.as_ref().map(|e| e.path)
    }

    /// Os nós à vista (mundo) e o agarrado — vazio com o Edit Shape fechado.
    #[must_use]
    pub fn shape_edit_nodes(&self) -> (Vec<P2>, Option<usize>) {
        match &self.edit {
            Some(e) if e.open => (edit::nodes(e.shape()), e.grab.as_ref().and_then(|g| g.node)),
            _ => (Vec::new(), None),
        }
    }

    /// A forma que o Edit Shape afina (para sondas e gates).
    #[must_use]
    pub fn shape_edit_shape(&self) -> Option<&QuickShape> {
        self.edit.as_ref().map(ShapeEdit::shape)
    }

    /// Ao soltar uma forma do QuickShape: ela passa a ser oferecida.
    pub(super) fn offer_shape_edit(&mut self, path: VecPathId, shape: QuickShape) {
        let w = self.style.stroke_w;
        let inset = arrow_ends(&self.style, shape.heads(), w).inset;
        self.edit = Some(ShapeEdit {
            path,
            history: vec![shape],
            at: 0,
            inset,
            open: false,
            grab: None,
        });
    }

    /// **O press com o lápis.** Com os nós à vista: um nó (ou o corpo) debaixo do ponteiro é
    /// AGARRADO (`true` — o gesto é do Edit Shape, não um traço). Senão o Edit Shape fecha e acaba
    /// (o toque fora / o traço seguinte do Procreate) e a pressão é um traço (`false`).
    pub fn shape_edit_press(
        &mut self,
        scene: &VecScene,
        p: P2,
        px_to_world: f64,
        xf: Xform,
        regular: bool,
    ) -> bool {
        let Some(e) = self.edit.as_mut() else {
            return false;
        };
        let grabbed = e.open.then(|| {
            let shape = e.shape().clone();
            let r = crate::node_hit::NODE_HIT_PX * px_to_world;
            let nodes = edit::nodes(&shape);
            let node = nodes
                .iter()
                .enumerate()
                .map(|(i, &q)| (i, (q[0] - p[0]).hypot(q[1] - p[1])))
                .filter(|&(_, d)| d <= r)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let on_body = || {
                let w = written(&shape, e.inset);
                ph2d_vec_scene::nearest_point_on_path(&w, p, 32)
                    .is_some_and(|(_, _, d2)| d2 <= r * r)
            };
            match node {
                Some((i, _)) => Some((Some(i), nodes[i])),
                None if on_body() => Some((None, p)),
                None => None,
            }
            .map(|(node, node_at)| Grab {
                node,
                shape,
                pointer: p,
                node_at,
                last: p,
                regular,
                xf,
            })
        });
        match grabbed.flatten() {
            Some(g) if scene.path(e.path).is_some() => {
                // Uma entrada nova no histórico por afinação; um undo anterior perde o futuro.
                e.history.truncate(e.at + 1);
                e.history.push(g.shape.clone());
                e.at += 1;
                e.grab = Some(g);
                let path = e.path;
                self.active = Some(path);
                self.px_to_world = px_to_world.max(f64::MIN_POSITIVE);
                true
            }
            _ => {
                self.edit = None;
                false
            }
        }
    }

    /// O arrasto de um nó ou do corpo. `true` = é deste gesto.
    pub(super) fn shape_edit_drag(&mut self, scene: &mut VecScene, p: P2) -> bool {
        let Some(e) = self.edit.as_mut() else {
            return false;
        };
        let Some(g) = e.grab.as_mut() else {
            return false;
        };
        g.last = p;
        let shape = reshape(g);
        write(scene, e.path, &shape, e.inset, &g.xf);
        e.history[e.at] = shape;
        true
    }

    /// O `Shift` a meio do arrasto: a forma refaz-se AO VIVO (`true` se mudou).
    pub fn shape_edit_regular(&mut self, scene: &mut VecScene, regular: bool) -> bool {
        let Some(e) = self.edit.as_mut() else {
            return false;
        };
        let Some(g) = e.grab.as_mut() else {
            return false;
        };
        if g.regular == regular {
            return false;
        }
        g.regular = regular;
        let shape = reshape(g);
        write(scene, e.path, &shape, e.inset, &g.xf);
        e.history[e.at] = shape;
        true
    }

    /// Soltar o arrasto. `true` = havia um (o caminho afinado fica o selecionado).
    pub(super) fn shape_edit_release(&mut self) -> bool {
        let Some(e) = self.edit.as_mut() else {
            return false;
        };
        if e.grab.take().is_none() {
            return false;
        }
        let path = e.path;
        self.active = None;
        self.committed = Some(path);
        true
    }

    /// O botão direito / o `Esc` a meio do arrasto: a forma volta à de quando foi agarrada.
    pub(super) fn shape_edit_cancel(&mut self, scene: &mut VecScene) -> bool {
        let Some(e) = self.edit.as_mut() else {
            return false;
        };
        let Some(g) = e.grab.take() else {
            return false;
        };
        write(scene, e.path, &g.shape, e.inset, &g.xf);
        e.history.pop();
        e.at -= 1;
        self.active = None;
        true
    }

    /// `Esc` com os nós à vista e sem arrasto: fecha (`true` = consumido).
    pub fn shape_edit_escape(&mut self) -> bool {
        match self.edit.as_mut() {
            Some(e) if e.open && e.grab.is_none() => {
                e.open = false;
                true
            }
            _ => false,
        }
    }

    /// **A cada quadro**: o Edit Shape só vale com o lápis na mão, o caminho dele selecionado e a
    /// geometria a que ele escreveu (ou uma que já escreveu — o undo); senão acaba.
    pub fn shape_edit_tick(
        &mut self,
        scene: &VecScene,
        xf: Xform,
        selected: Option<VecPathId>,
        pencil_in_hand: bool,
    ) {
        let Some(e) = self.edit.as_mut() else {
            return;
        };
        if e.grab.is_some() {
            return;
        }
        let keep = pencil_in_hand
            && selected == Some(e.path)
            && scene.path(e.path).is_some_and(|path| {
                if same(path, &xf, e.shape(), e.inset) {
                    return true;
                }
                let found = (0..e.history.len())
                    .rev()
                    .find(|&i| same(path, &xf, &e.history[i], e.inset));
                if let Some(i) = found {
                    e.at = i;
                }
                found.is_some()
            });
        if !keep {
            self.edit = None;
        }
    }

    /// O traço seguinte começa: a oferta acaba (o «continue a pintar» do Procreate).
    pub(super) fn shape_edit_forget(&mut self) {
        self.edit = None;
    }
}

/// A forma do arrasto `g` agora.
fn reshape(g: &Grab) -> QuickShape {
    match g.node {
        Some(i) => {
            let to = [
                g.node_at[0] + g.last[0] - g.pointer[0],
                g.node_at[1] + g.last[1] - g.pointer[1],
            ];
            edit::drag(&g.shape, i, to, g.regular)
        }
        None => edit::scale(&g.shape, g.pointer, g.last),
    }
}

/// Escreve a forma no caminho (em LOCAL), sem tocar no estilo nem nas pontas.
fn write(scene: &mut VecScene, path: VecPathId, shape: &QuickShape, inset: [f64; 2], xf: &Xform) {
    let w = written(shape, inset);
    let to_local = xf.inverse().unwrap_or(Xform::IDENTITY);
    if let Some(live) = scene.path_mut(path) {
        live.verts = w.verts.iter().map(|v| map_vert(v, &to_local)).collect();
        live.closed = w.closed;
    }
}

#[cfg(test)]
#[path = "pencil_edit_tests.rs"]
mod tests;
