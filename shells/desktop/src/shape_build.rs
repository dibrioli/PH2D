//! **Shape Builder** — o cursor pinta REGIÕES, e o que ele pinta vira forma.
//!
//! Com 2+ formas fechadas selecionadas, o plano fica dividido nas faces do arranjo
//! ([`ph2d_vec_boolean::Arrangement`]): "dentro da A e fora da B", "dentro das duas", e
//! assim por diante. O Shape Builder deixa o dedo passar por cima dessas faces:
//!
//! - **Arrastar** (ou clicar): as faces tocadas viram **UMA** forma.
//! - **Alt + arrastar**: as faces tocadas **somem**.
//!
//! É a diferença entre pedir uma operação e **desenhar o resultado**. Um Pathfinder obriga
//! o artista a traduzir o que ele quer ("a lua crescente") numa sequência de ops ("subtrai
//! o círculo B do A"); aqui ele passa o dedo na parte que quer e pronto.
//!
//! ## A regra do resultado (uma só, e é a que a booleana já usava)
//!
//! Tudo que o Shape Builder emite herda o **estilo da forma do TOPO** — a mesma convenção
//! do `apply_many` e do Illustrator. Sem essa regra a face herdaria o estilo do último
//! argumento de uma SUBTRAÇÃO, que é justamente uma forma que ela **não** contém: a região
//! sairia pintada com a cor de quem não a cobre.
//!
//! ## O que NÃO é feito aqui, e por quê
//!
//! A sobra (o que não foi pintado) sai como **uma** região — as componentes desconexas dela
//! se separam sozinhas, mas duas faces não-pintadas que se ENCOSTAM viram uma forma só.
//! Decompor a sobra em todas as faces individuais exigiria enumerar as `2^N` pertinências
//! possíveis, e o custo explode. É um trade consciente: a sobra é "o que você não levou",
//! não um inventário.

use ph2d_vec_boolean::{Arrangement, BoolOp, FaceId, apply_many};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};

/// O gesto de Shape Builder em curso.
pub(crate) struct BuildSession {
    /// As faces, em MUNDO (as formas entram assadas — a booleana precisa de um frame só).
    pub(crate) arr: Arrangement,
    /// Os ids das formas de origem, na ordem de z (fundo → topo). São eles que somem no
    /// apply.
    pub(crate) sources: Vec<VecPathId>,
    /// As faces que o dedo já pintou, na ordem em que foram tocadas.
    pub(crate) marked: Vec<FaceId>,
    /// A face sob o cursor agora (o realce que segue o mouse mesmo sem botão apertado).
    pub(crate) hover: Option<FaceId>,
    /// Alt: o gesto SUBTRAI em vez de unir. Fixado no press — trocar de modo no meio do
    /// arrasto faria o mesmo gesto significar duas coisas.
    pub(crate) subtract: bool,
    /// Entre o press e o release.
    pub(crate) dragging: bool,
}

impl BuildSession {
    /// Abre a sessão para as formas `ids` (ordem de z, fundo → topo), assando cada uma no
    /// MUNDO. `None` com menos de 2 formas — não há região para pintar.
    pub(crate) fn open(
        scene: &VecScene,
        xforms: &VecXforms,
        ids: &[VecPathId],
    ) -> Option<BuildSession> {
        if ids.len() < 2 {
            return None;
        }
        let world: Vec<VecPath> = ids
            .iter()
            .filter_map(|id| scene.paths().iter().find(|p| p.id == *id))
            .filter(|p| p.closed)
            .map(|p| {
                let mut w = p.clone();
                ph2d_vec_scene::bake_xform(&mut w, &ph2d_vec_scene::xform_of(xforms, p.id));
                w
            })
            .collect();
        if world.len() < 2 {
            return None;
        }
        Some(BuildSession {
            arr: Arrangement::new(world),
            sources: ids.to_vec(),
            marked: Vec::new(),
            hover: None,
            subtract: false,
            dragging: false,
        })
    }

    /// O cursor andou (mundo): atualiza o realce e, se estiver arrastando, PINTA a face.
    pub(crate) fn touch(&mut self, world: [f64; 2]) {
        let face = self.arr.face_at(world);
        self.hover = face;
        if let (true, Some(f)) = (self.dragging, face)
            && !self.marked.contains(&f)
        {
            self.marked.push(f);
        }
    }

    /// As formas que este gesto produz, em MUNDO. Vazio = nada a fazer.
    ///
    /// - **Unir:** as faces pintadas viram uma forma; a sobra fica.
    /// - **Subtrair:** as faces pintadas somem; sobra a sobra.
    ///
    /// (As duas são a MESMA conta — a única diferença é se o que foi pintado é entregue ou
    /// jogado fora. Escrever assim é o que garante que "pintar tudo e unir" e "pintar tudo
    /// e subtrair" sejam exatamente complementares.)
    pub(crate) fn resolve(&mut self) -> Vec<VecPath> {
        if self.marked.is_empty() {
            return Vec::new();
        }
        let faces: Vec<VecPath> = self
            .marked
            .clone()
            .into_iter()
            .filter_map(|f| self.arr.face_path(f).cloned())
            .collect();
        if faces.is_empty() {
            return Vec::new();
        }
        let merged = union_of(&faces);
        let leftover = subtract_from_whole(self.arr.sources(), &merged);

        let mut out = if self.subtract {
            leftover
        } else {
            let mut v = merged;
            v.extend(leftover);
            v
        };
        // **O estilo do topo, para tudo** (a convenção do `apply_many` e do Illustrator).
        // Sem isto, o estilo de uma face viria do último argumento da SUBTRAÇÃO que a
        // produziu — uma forma que ela justamente NÃO contém.
        if let Some(top) = self.arr.sources().last() {
            for p in &mut out {
                p.fill = top.fill.clone();
                p.stroke = top.stroke;
            }
        }
        out
    }
}

/// A união de um punhado de formas. Uma só é ela mesma (o `apply_many` exige duas).
fn union_of(paths: &[VecPath]) -> Vec<VecPath> {
    if paths.len() == 1 {
        return paths.to_vec();
    }
    apply_many(&paths.iter().collect::<Vec<_>>(), BoolOp::Union)
}

/// `(∪ de todas as formas) − (o que foi pintado)` — a sobra.
fn subtract_from_whole(sources: &[VecPath], taken: &[VecPath]) -> Vec<VecPath> {
    if taken.is_empty() {
        return union_of(sources);
    }
    let whole = union_of(sources);
    let mut out = Vec::new();
    for w in &whole {
        let mut args: Vec<&VecPath> = Vec::with_capacity(1 + taken.len());
        args.push(w);
        args.extend(taken.iter());
        out.extend(apply_many(&args, BoolOp::Subtract));
    }
    out
}

#[cfg(test)]
#[path = "shape_build_tests.rs"]
mod tests;
