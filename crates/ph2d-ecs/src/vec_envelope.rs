//! **O Envelope vivo** (ADR-0123): a forma cuja geometria é a fonte autorada **deformada por uma
//! gaiola de 4 cantos**, re-cozida a cada frame.
//!
//! Irmão do [`crate::VecMorph`] e do [`crate::VecBlend`] no padrão, e diferente no que deforma: o
//! Blend/Morph interpolam DUAS formas; o Envelope deforma UMA, por um mapa `R2→R2` (aqui, a
//! homografia do gesto Quad — `ph2d_vec_envelope::QuadWarp`). A entidade que o carrega tem um
//! [`crate::VecPathRef`], e o `VecPath` dela é a forma **cozida** (deformada), geometria de verdade
//! na cena, re-escrita *em lugar* a cada frame — como o Morph.
//!
//! # A fonte AUTORADA vive aqui, e por que em BYTES
//!
//! O `recook` sobrescreve o path da cena com a geometria deformada. Se a fonte não estivesse
//! guardada, ela morreria no 1º frame — é o bug *"funciona e depois esquece"* que o ADR-0121 §3
//! documentou (uma Live Shape não pode ter raio pelo mesmo motivo). Então a **fonte afiada** viaja
//! dentro do componente, como no `inkscape:original-d`.
//!
//! Ela é `Vec<u8>` (postcard de um `VecPath`), **não** um `VecPath`, de propósito: assim o
//! `ph2d-ecs` **não depende do `ph2d-vec-scene`**. O `ph2d-ecs` é a fundação; puxar a geometria
//! vetorial para dentro dele acoplaria o núcleo a uma crate satélite. A shell — que já conhece as
//! duas — serializa na criação e desserializa no recook. O componente só carrega bytes.
//!
//! Consequência de graça (a mesma do Morph): **undo e save cobrem o envelope sem uma linha a
//! mais** — os dois capturam o mundo ECS, e este componente está registrado no `ComponentRegistry`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O Envelope Object.** Guarda a fonte autorada (postcard) + os 4 cantos da gaiola que a deformam.
///
/// A geometria que o mundo vê está no `VecPath` da entidade (a cozida); esta struct é a **relação**
/// da qual essa geometria é função pura, re-cozida por frame pela shell.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecEnvelope {
    /// A **fonte afiada**: os bytes postcard do `VecPath` autorado, em coordenadas de MUNDO
    /// (assadas na criação, quando a pose ainda existia). O recook a desserializa, deforma pela
    /// gaiola e escreve o resultado no path da cena.
    ///
    /// Guardar a fonte é o que impede o *"funciona e depois esquece"* — sem ela, o 1º recook
    /// varreria o que o artista desenhou e não haveria de onde recuperá-lo.
    pub source: Vec<u8>,
    /// Os 4 cantos da gaiola-destino, em coordenadas de MUNDO, na ordem `[BL, BR, TR, TL]` — a mesma
    /// que o `QuadWarp` espera. Em **repouso** eles coincidem com os cantos do bbox da fonte, e a
    /// deformação é a identidade (a forma não muda). Arrastá-los deforma.
    ///
    /// São 4 números por canto que a UI (fatia seguinte) move; a **convexidade** deles é o que
    /// mantém a linha de fuga fora da gaiola (ADR-0123 §5, `QuadWarp::is_convex`).
    pub corners: [[f64; 2]; 4],
}

impl SimComponent for VecEnvelope {}

impl VecEnvelope {
    /// Um envelope novo sobre `source` (bytes postcard de um `VecPath` de mundo), com a gaiola em
    /// **repouso** (`corners` = cantos do bbox da fonte) — a forma não muda até o artista arrastar
    /// um canto. É o certo: um envelope que nasce deformado desorienta; um que nasce transparente
    /// mostra a gaiola e espera o gesto.
    #[must_use]
    pub fn at_rest(source: Vec<u8>, corners: [[f64; 2]; 4]) -> Self {
        Self { source, corners }
    }
}
