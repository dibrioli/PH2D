//! **O ESQUELETO do desenho** (estudo 42 item 5, doc 47): o osso é uma ENTIDADE, e a ligação de uma
//! forma a ele é um componente da forma.
//!
//! # Por que o osso não é um dado dentro de um componente
//!
//! Porque a cinemática direta **já corre**: [`crate::propagate_transforms`] compõe a pose de um
//! filho com a do pai, que é a definição de FK. Uma árvore de ossos guardada dentro de um
//! componente seria uma **segunda hierarquia** — exactamente o que a ADR-0110 rejeita pelo nome — e
//! teria de reimplementar, sozinha, o undo, o save, o olho, o cadeado, o reparentar e a timeline.
//!
//! ⇒ Um osso é uma entidade com [`crate::Transform`] mais este [`VecBone`], que carrega só o que a
//! pose não sabe dizer: **o comprimento** (onde acaba o osso) e **a força** (até onde ele manda).
//! ⛔ Ele **não** é um `VecPath`: não tem tinta, não exporta, não entra na cena vectorial. O que se
//! vê no canvas é overlay, como a gaiola do Envelope.
//!
//! # A ligação, e o que ela NÃO guarda
//!
//! O [`VecSkin`] segue o padrão do [`crate::VecEnvelope`] no que é da casa — a fonte autorada viaja
//! em **bytes postcard** para o `ph2d-ecs` não depender do `ph2d-vec-scene` (a fundação não puxa
//! uma crate satélite), e a shell re-escreve a forma da cena a cada quadro.
//!
//! ⚠️⚠️ **E ele NÃO guarda pesos.** O doc do `VecVertex::corner_radius` já escreveu a razão, sobre
//! por que o raio mora dentro do vértice: *"e não num vetor paralelo ao lado dos `verts`, de
//! propósito: dezenas de operações inserem, apagam, invertem e soldam vértices, e cada uma delas
//! teria de lembrar de mexer no vetor paralelo também."* Uma tabela de pesos indexada por ordem de
//! varredura **é** esse vector paralelo. Aqui guarda-se o **BIND** (a fonte + a matriz de repouso de
//! cada osso) e o peso é derivado dele a cada quadro — então editar a forma re-pesa sozinho.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **UM OSSO.** A pose dele é o [`crate::Transform`] da entidade; a hierarquia dela é o esqueleto.
///
/// O eixo do osso é o **+X local**, de `(0,0)` a `(length, 0)` — a convenção de toda a indústria
/// (Rive, Spine, Blender), e a que faz um filho pendurado na ponta ser só um `Transform` com
/// `translation.x = length`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecBone {
    /// O comprimento, em unidades **locais** do osso — logo ele herda a escala do pai, como tudo o
    /// resto da hierarquia.
    pub length: f64,
    /// ⭐ **A FORÇA** — o raio de influência, em **comprimentos deste osso** (o *Bone Strength* do
    /// Moho). `1` = ele alcança um comprimento dele a partir do próprio eixo.
    ///
    /// ⚠️ **É um múltiplo e não uma distância, de propósito:** assim a lei é adimensional e o mesmo
    /// rig desenhado dez vezes maior deforma-se igual (gate
    /// `the_same_rig_ten_times_bigger_weighs_exactly_the_same`, em `ph2d-vec-skin`).
    pub strength: f64,
}

impl Default for VecBone {
    fn default() -> Self {
        Self {
            length: 1.0,
            strength: 1.0,
        }
    }
}

impl SimComponent for VecBone {}

/// Um osso a que **esta** forma está presa, com a pose de repouso dele no instante do bind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecSkinBone {
    /// Os bits da entidade do osso (u64 cru — o `ph2d-ecs` não exporta `Entity` para o postcard, e
    /// o `VecEnvelope` já usa a mesma convenção para o path do filho).
    pub bone: u64,
    /// ⭐ **O TENDÃO** — o afim `osso → forma` no instante em que se ligou, em `[a,b,c,d,e,f]`.
    ///
    /// É daqui que sai TUDO: o eixo de repouso (`rest·(0,0)` até `rest·(length,0)`, que é o que a
    /// distância mede) e a matriz da pose (`S⁻¹ ∘ B ∘ rest⁻¹`). ⚠️ E é por ele ser o composto
    /// `forma⁻¹ ∘ osso` que a pose de repouso é a **identidade** sem uma guarda escrita à mão.
    pub rest: [f64; 6],
}

/// **A PELE DE UMA FORMA** — a que ossos ela responde, e o que ela era antes de responder.
///
/// ⛔ **Não é um container**, ao contrário do [`crate::VecEnvelope`], e a diferença é medida: aquele
/// precisa de um container porque a **gaiola não tem outra casa** (não é entidade). Aqui o esqueleto
/// já são entidades, então não há nada de partilhado à procura de dono — e uma forma presa fica
/// exactamente onde o artista a pôs na Hierarquia.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecSkin {
    /// Os bytes postcard do `VecPath` **autorado**, em coordenadas locais da forma no bind.
    ///
    /// Sem ela a fonte morria no 1.º quadro — o recook sobrescreve o caminho da cena com a
    /// deformada, e é o bug *"funciona e depois esquece"* que o ADR-0121 §3 documentou.
    pub source: Vec<u8>,
    /// Os ossos, na ordem em que foram ligados. Um cuja entidade desapareceu é **saltado** no
    /// recook e os outros renormalizam-se sozinhos — apagar um osso não pode apagar a forma.
    pub bones: Vec<VecSkinBone>,
}

impl SimComponent for VecSkin {}

impl VecSkin {
    /// Uma pele nova. `bones` vazio é legal e significa *"presa a nada"* — o recook deixa a forma
    /// em paz, que é a leitura certa de um esqueleto inteiro apagado.
    #[must_use]
    pub fn new(source: Vec<u8>, bones: Vec<VecSkinBone>) -> Self {
        Self { source, bones }
    }
}
