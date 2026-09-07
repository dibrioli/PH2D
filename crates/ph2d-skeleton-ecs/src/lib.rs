#![forbid(unsafe_code)]
//! **O ESQUELETO NA CENA** (estudo 42 item 5, doc 47): o osso é uma ENTIDADE, e a ligação de uma
//! coisa a ele é um componente dessa coisa.
//!
//! # Por que o osso não é um dado dentro de um componente
//!
//! Porque a cinemática direta **já corre**: `ph2d_ecs::propagate_transforms` compõe a pose de um
//! filho com a do pai, que é a definição de FK. Uma árvore de ossos guardada dentro de um
//! componente seria uma **segunda hierarquia** — exactamente o que a ADR-0110 rejeita pelo nome — e
//! teria de reimplementar, sozinha, o undo, o save, o olho, o cadeado, o reparentar e a timeline.
//!
//! ⇒ Um osso é uma entidade com [`ph2d_ecs::Transform`] mais este [`Bone`], que carrega só o que a
//! pose não sabe dizer: **o comprimento** (onde acaba o osso) e **a força** (até onde ele manda).
//! ⛔ Ele **não** é uma forma: não tem tinta, não exporta, não entra na cena vectorial. O que se
//! vê no canvas é overlay, como a gaiola do Envelope.
//!
//! # Por que os componentes moram AQUI e não na fundação
//!
//! Porque o esqueleto serve **várias mídias** (vector, raster, 3D, Flip), e um componente por
//! mídia dentro do `ph2d-ecs` faria a fundação crescer uma vez por cliente. O precedente é a
//! [`ph2d_physics_ecs`], que possui `RigidBody`/`Collider` e os regista pela porta dela.
//!
//! ⚠️ **Registar é obrigatório e o esquecimento é MUDO:** sem a chamada a
//! [`register_skeleton_components`] o `WorldSnapshot` **descarta** estes componentes em silêncio —
//! o desenho perde o esqueleto no primeiro undo e no primeiro save, sem uma linha de erro.
//!
//! # A ligação, e o que ela NÃO guarda
//!
//! O [`SkinBind`] segue o padrão do `ph2d_ecs::VecEnvelope` no que é da casa — a fonte autorada
//! viaja em **bytes postcard**, e a shell re-escreve a forma a cada quadro.
//!
//! ⚠️⚠️ **E ele NÃO guarda pesos.** O doc do `VecVertex::corner_radius` já escreveu a razão, sobre
//! por que o raio mora dentro do vértice: *"e não num vetor paralelo ao lado dos `verts`, de
//! propósito: dezenas de operações inserem, apagam, invertem e soldam vértices, e cada uma delas
//! teria de lembrar de mexer no vetor paralelo também."* Uma tabela de pesos indexada por ordem de
//! varredura **é** esse vector paralelo. Aqui guarda-se o **BIND** (a fonte + a matriz de repouso
//! de cada osso) e o peso é derivado dele a cada quadro — então editar a forma re-pesa sozinho.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use ph2d_ecs::SimComponent;
use ph2d_ecs::StableId;
use ph2d_ecs::scene::ComponentRegistry;

/// **UM OSSO.** A pose dele é o [`ph2d_ecs::Transform`] da entidade; a hierarquia dela é o
/// esqueleto.
///
/// O eixo do osso é o **+X local**, de `(0,0)` a `(length, 0)` — a convenção de toda a indústria
/// (Rive, Spine, Blender), e a que faz um filho pendurado na ponta ser só um `Transform` com
/// `translation.x = length`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bone {
    /// O comprimento, em unidades **locais** do osso — logo ele herda a escala do pai, como tudo o
    /// resto da hierarquia.
    pub length: f64,
    /// ⭐ **A FORÇA** — o raio de influência, em **comprimentos deste osso** (o *Bone Strength* do
    /// Moho). `1` = ele alcança um comprimento dele a partir do próprio eixo.
    ///
    /// ⚠️ **É um múltiplo e não uma distância, de propósito:** assim a lei é adimensional e o mesmo
    /// rig desenhado dez vezes maior deforma-se igual (gate
    /// `the_same_rig_ten_times_bigger_weighs_exactly_the_same`, em `ph2d-skeleton`).
    pub strength: f64,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            length: 1.0,
            strength: 1.0,
        }
    }
}

impl SimComponent for Bone {}

/// Um osso a que **esta** coisa está presa, com a pose de repouso dele no instante do bind.
///
/// ⭐ O nome é o do Rive, que chama a mesma coisa exactamente assim — um *Tendon* liga um osso a
/// uma pele.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tendon {
    /// ⭐⭐⭐ **A IDENTIDADE DURÁVEL do osso** — o [`StableId`], nunca o `Entity::to_bits()`.
    ///
    /// ⚠️⚠️ **A 1.ª redacção guardava os bits da entidade, e MEDIU-SE que a pele morria em
    /// silêncio no primeiro Ctrl+Z** (gate `a_skin_survives_the_respawn_that_undo_and_save_do`,
    /// `0 de 2` tendões a resolver): o undo e o salvar são a mesma máquina, ela **despawna e
    /// re-spawna no mesmo mundo**, e o `to_bits` é um **id de alocação** — a geração sobe e os bits
    /// deixam de nomear nada. O sintoma não é um erro: a forma fica com a última geometria boa e
    /// deixa de responder aos ossos.
    ///
    /// ⛔ E os bits guardados eram também um **pânico à espera**: o `Entity::from_bits` do
    /// `bevy_ecs` aborta o processo com bits que nunca vieram de um `to_bits`, e é exactamente
    /// isso que um ficheiro gravado noutra sessão entrega.
    ///
    /// ⭐ **O tipo é a cerca**, e não o nome do campo: com `StableId` aqui, escrever um
    /// `e.to_bits()` neste sítio é **erro de compilação**. Foi a lição do anel da ponta — *um valor
    /// que muda de significado e mantém a forma não avisa ninguém*.
    ///
    /// A porta de escrita é o [`ph2d_ecs::stable_id_of`] e a de leitura o
    /// [`ph2d_ecs::entity_of_stable_id`]; é a mesma identidade por que a `PhysicsJoint` nomeia os
    /// corpos dela desde a wave das instâncias.
    pub bone: StableId,
    /// ⭐ **O TENDÃO** — o afim `osso → coisa` no instante em que se ligou, em `[a,b,c,d,e,f]`.
    ///
    /// É daqui que sai TUDO: o eixo de repouso (`rest·(0,0)` até `rest·(length,0)`, que é o que a
    /// distância mede) e a matriz da pose (`S⁻¹ ∘ B ∘ rest⁻¹`). ⚠️ E é por ele ser o composto
    /// `coisa⁻¹ ∘ osso` que a pose de repouso é a **identidade** sem uma guarda escrita à mão.
    pub rest: [f64; 6],
}

/// **A PELE DE UMA COISA** — a que ossos ela responde, e o que ela era antes de responder.
///
/// ⚠️ **O nome do TIPO é `SkinBind` e o rótulo que o artista lê é "Skin"** — o tipo diz o que se
/// GUARDA (o bind: a fonte mais as matrizes de repouso), e o rótulo diz o que a coisa É. O par
/// vive no catálogo do `ph2d-component-desc`, que é onde os dois nomes se encontram.
///
/// ⛔ **Não é um container**, ao contrário do `ph2d_ecs::VecEnvelope`, e a diferença é medida:
/// aquele precisa de um container porque a **gaiola não tem outra casa** (não é entidade). Aqui o
/// esqueleto já são entidades, então não há nada de partilhado à procura de dono — e uma forma
/// presa fica exactamente onde o artista a pôs na Hierarquia.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkinBind {
    /// Os bytes postcard da fonte **autorada**, em coordenadas locais da coisa no bind.
    ///
    /// Sem ela a fonte morria no 1.º quadro — o recook sobrescreve a geometria da cena com a
    /// deformada, e é o bug *"funciona e depois esquece"* que o ADR-0121 §3 documentou.
    ///
    /// ⚠️ **Bytes opacos, de propósito:** é o que permite a este componente servir um `VecPath`
    /// hoje e uma malha raster amanhã sem uma variante nova nem um schema por mídia.
    pub source: Vec<u8>,
    /// Os ossos, na ordem em que foram ligados. Um cuja entidade desapareceu é **saltado** no
    /// recook e os outros renormalizam-se sozinhos — apagar um osso não pode apagar a forma.
    pub tendons: Vec<Tendon>,
}

impl SimComponent for SkinBind {}

impl SkinBind {
    /// Uma pele nova. `tendons` vazio é legal e significa *"presa a nada"* — o recook deixa a coisa
    /// em paz, que é a leitura certa de um esqueleto inteiro apagado.
    #[must_use]
    pub fn new(source: Vec<u8>, tendons: Vec<Tendon>) -> Self {
        Self { source, tendons }
    }
}

/// Regista os componentes que a `ph2d-skeleton-ecs` possui. A shell chama isto uma vez no arranque,
/// ao lado do `register_ecs_components` e do `register_physics_components`.
///
/// ⚠️ **Sem esta chamada o `WorldSnapshot` descarta-os em SILÊNCIO** — é o bug
/// `Locked`/`GroupedChildren`/`VecPathRef` que a física já pagou, e aqui ele apareceria como *"o
/// personagem perdeu o esqueleto ao desfazer"*.
pub fn register_skeleton_components(reg: &mut ComponentRegistry) {
    // `register_default`: um osso de comprimento 1 e força 1 é um osso legítimo, então a paleta do
    // Inspector pode pendurá-lo.
    reg.register_default::<Bone>("ph2d::skeleton::Bone");
    // `register`: uma pele sem a fonte autorada dentro não é uma pele, é uma forma prestes a sumir
    // — ela chega pelo GESTO (*Bind*) e nunca por um botão de "acrescentar componente".
    reg.register::<SkinBind>("ph2d::skeleton::Skin");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **Esta contagem existe para doer** (espelha `registers_every_physics_component`): um
    /// componente do esqueleto que salte o registo é descartado de todo snapshot em silêncio. Se
    /// acrescentar um, suba o número.
    #[test]
    fn registers_every_skeleton_component() {
        let mut reg = ComponentRegistry::new();
        register_skeleton_components(&mut reg);
        assert_eq!(reg.len(), 2);
        assert!(reg.get_by_name("ph2d::skeleton::Bone").is_some());
        assert!(reg.get_by_name("ph2d::skeleton::Skin").is_some());
    }

    /// ⭐ **O NOME CANÓNICO NÃO DIZ "VECTOR"** — e é a metade destrutiva-depois desta wave.
    ///
    /// O `ComponentBlob` é endereçado por `blake3(nome canónico)`, então trocar o nome depois de
    /// existirem projectos gravados faria cada um deles **perder o esqueleto em silêncio** ao
    /// abrir. Medido em 2026-09-06: os dois `.ph2dproj` da máquina do dono são de 26/08, onze dias
    /// antes de os ossos existirem ⇒ nenhum tem esqueleto, e a troca custou zero.
    ///
    /// Este gate impede que alguém devolva a palavra ao nome sem reabrir aquela conta.
    #[test]
    fn the_canonical_names_belong_to_the_module_not_to_one_medium() {
        let mut reg = ComponentRegistry::new();
        register_skeleton_components(&mut reg);
        for d in reg.iter() {
            assert!(
                d.canonical_name.starts_with("ph2d::skeleton::"),
                "`{}` nao mora no modulo - um nome por midia volta a prender o esqueleto a ela",
                d.canonical_name
            );
            assert!(
                !d.canonical_name.to_ascii_lowercase().contains("vec"),
                "`{}` ainda diz VECTOR, e o esqueleto serve raster, 3D e Flip",
                d.canonical_name
            );
        }
    }

    /// Um osso nasce legítimo: comprimento 1, força 1 — e a força é um MÚLTIPLO, o que faz a lei
    /// ser adimensional.
    #[test]
    fn a_fresh_bone_is_one_long_and_reaches_one_of_itself() {
        let b = Bone::default();
        assert_eq!(b.length, 1.0);
        assert_eq!(b.strength, 1.0);
    }
}
