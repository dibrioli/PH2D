//! **O Envelope vivo** (ADR-0129): um **container** cujos filhos têm a geometria deformada por uma
//! **gaiola de 4 cantos** comum, re-cozida a cada frame.
//!
//! Irmão do [`crate::VecMorph`] e do [`crate::VecBlend`] no padrão, e diferente no que deforma: o
//! Blend/Morph interpolam DUAS formas; o Envelope deforma N formas (1..) por um mapa `R2→R2` (aqui, a
//! homografia do gesto Quad — `ph2d_vec_envelope::QuadWarp`). A entidade que o carrega é um
//! **container** (sem `VecPathRef` próprio); cada FILHO tem o seu `VecPath` cozido (deformado),
//! geometria de verdade na cena, re-escrita *em lugar* a cada frame — como o Morph.
//!
//! # Um container, não um path — e por quê
//!
//! Um envelope de uma forma só é o caso `N=1`; um de várias é o Affinity/Illustrator *warp group*.
//! Modelar o de-um como path-com-componente e o de-vários como container seriam DOIS modelos do mesmo
//! fato — e o gesto/gizmo/recook teriam de perguntar "qual dos dois é este?" em cada sítio. Então há
//! **um só modelo: o container sempre**. A gaiola envolve a bbox-união dos filhos; o `Transform` do
//! container é a pose que o gizmo de sprite move no Select (a dos filhos é identidade, ADR-0111).
//!
//! # A fonte AUTORADA de cada filho vive aqui, e por que em BYTES
//!
//! O `recook` sobrescreve o path de cada filho com a geometria deformada. Se a fonte não estivesse
//! guardada, ela morreria no 1º frame — é o bug *"funciona e depois esquece"* que o ADR-0121 §3
//! documentou (uma Live Shape não pode ter raio pelo mesmo motivo). Então a **fonte afiada** de cada
//! filho viaja dentro do componente, como no `inkscape:original-d`.
//!
//! Ela é `Vec<u8>` (postcard de um `VecPath`), **não** um `VecPath`, de propósito: assim o
//! `ph2d-ecs` **não depende do `ph2d-vec-scene`**. O `ph2d-ecs` é a fundação; puxar a geometria
//! vetorial para dentro dele acoplaria o núcleo a uma crate satélite. A shell — que já conhece as
//! duas — serializa na criação e desserializa no recook. O componente só carrega bytes. Pela mesma
//! razão o filho é referido pelo seu `path` como `u64` cru (os bits do `VecPathId`), não pelo tipo.
//!
//! Consequência de graça (a mesma do Morph): **undo e save cobrem o envelope sem uma linha a
//! mais** — os dois capturam o mundo ECS, e este componente está registrado no `ComponentRegistry`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Um **filho** de um envelope: o path que ele dirige + a fonte autorada que a gaiola deforma nele.
///
/// O `path` são os bits do `VecPathId` (u64 cru — ver o motivo no doc do módulo). A `source` são os
/// bytes postcard do `VecPath` **autorado**, em coordenadas **LOCAIS do container** (a pose vive no
/// `Transform` do container — ADR-0111). Guardar a fonte é o que impede o *"funciona e depois
/// esquece"*: sem ela, o 1º recook varreria o que o artista desenhou e não haveria de onde recuperá-lo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecEnvelopeChild {
    /// Os bits do `VecPathId` do filho — o path da cena que este envelope reescreve a cada frame.
    pub path: u64,
    /// Os bytes postcard do `VecPath` autorado do filho, em coordenadas **LOCAIS do container**.
    pub source: Vec<u8>,
}

/// **O Envelope Object.** Um container que guarda os 4 cantos da gaiola + a fonte autorada de cada
/// filho. A geometria que o mundo vê está no `VecPath` de cada filho (a cozida); esta struct é a
/// **relação** da qual essa geometria é função pura, re-cozida por frame pela shell.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecEnvelope {
    /// Os 4 cantos da gaiola-destino, em coordenadas **LOCAIS do container** (o mesmo espaço das
    /// fontes dos filhos), na ordem `[BL, BR, TR, TL]` — a que o `QuadWarp` espera. Em **repouso**
    /// coincidem com os cantos da bbox-união das fontes, e a deformação é a identidade (as formas não
    /// mudam). Arrastá-los (modo Node) deforma TODOS os filhos pelo mesmo mapa.
    ///
    /// A **convexidade** deles é o que mantém a linha de fuga fora da gaiola (ADR-0129 §5,
    /// `QuadWarp::is_convex`) — e ela é invariante à pose (afim), então checá-la em local basta.
    pub corners: [[f64; 2]; 4],
    /// Os filhos deformados por esta gaiola — um para um envelope de forma única (`N=1`), vários para
    /// um *warp group*. Cada um carrega o seu path + a sua fonte (ver [`VecEnvelopeChild`]).
    pub children: Vec<VecEnvelopeChild>,
}

impl SimComponent for VecEnvelope {}

impl VecEnvelope {
    /// Um envelope novo sobre `children` (cada um com os bytes postcard da sua fonte LOCAL), com a
    /// gaiola em **repouso** (`corners` = cantos da bbox-união das fontes) — as formas não mudam até o
    /// artista arrastar um canto. É o certo: um envelope que nasce deformado desorienta; um que nasce
    /// transparente mostra a gaiola e espera o gesto.
    #[must_use]
    pub fn at_rest(children: Vec<VecEnvelopeChild>, corners: [[f64; 2]; 4]) -> Self {
        Self { corners, children }
    }
}
