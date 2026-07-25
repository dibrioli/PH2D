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

/// **Qual mapa a gaiola aplica** — os dois gestos do ADR-0129 §4.
///
/// Não são um mesmo mapa com um knob: eles **divergem no miolo**. Com os 4 lados retos,
/// [`EnvelopeKind::Mesh`] é *bilinear* (uma reta interior vira parábola) e
/// [`EnvelopeKind::Perspective`] é *projetivo* (toda reta continua reta) — que é o que "perspectiva"
/// quer dizer, e por que Photoshop separa *Distort* de *Warp*. Em **repouso** os dois são a
/// identidade, então trocar de gesto numa gaiola intocada não move um pixel; trocar depois de
/// deformar **muda o desenho**, e isso é o mapa mudando, não um bug.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvelopeKind {
    /// Homografia dos 4 cantos. Os lados são **retos** (o invariante que `rest_edges` mantém) e as
    /// retas interiores continuam retas. É o default: é como o envelope nascia antes da Fatia D, e
    /// arte anterior continua a cozinhar exatamente igual.
    #[default]
    Perspective,
    /// Patch de Coons das 4 curvas de bordo — os lados **dobram**. O *Mesh Warp* do Affinity.
    Mesh,
    /// **Pinos** (MLS-rigid, Schaefer 2006) — o *puppet warp*. Não há gaiola: o artista prega pontos
    /// e arrasta. ⚠️ **Duas coisas contra-intuitivas, as duas são o método e não bugs:** o suporte é
    /// **global** (o container é o escopo, não um raio), e com **2 pinos não se deforma nada** — uma
    /// isometria de um par determina uma rigidez única, então é preciso um **3º pino não-colinear**.
    Pins,
}

/// **O preset de gaiola é o CATÁLOGO ÚNICO** [`ph2d_warp_style::WarpStyle`] (Enio 2026-07-25) — a
/// MESMA lista que o efeito Warp (`ph2d-vec-scene`), para os dois não divergirem. `EnvelopeWarp` é
/// só um apelido dele aqui.
///
/// ⚠️ **A `ph2d-ecs` é a fundação e evita crate satélite** — mas o catálogo é uma FOLHA pura (só
/// `serde`, nem `bevy_ecs`), então puxá-lo não fere a regra que mantém o `VecEnvelopeChild::source`
/// em bytes. A alternativa (um enum em cada lado) já custou o drift que este catálogo desfaz (o
/// "Wave" de um era o "Flag" do outro).
///
/// A gaiola de um estilo vem de [`WarpStyle::cage`] (`bows` + `shift`); a matemática que a carimba é
/// o `ph2d_vec_envelope::preset_cage`. Save-compat: os 7 primeiros variants do catálogo estão na
/// ordem exata do antigo `EnvelopeWarp` (Fisheye/Rise apendados), então `VecEnvelope.warp` de
/// projetos salvos relê certo.
pub use ph2d_warp_style::WarpStyle as EnvelopeWarp;

/// A força com que um preset de envelope é carimbado da primeira vez.
///
/// Não é gosto: com `bend = 0` o preset É a gaiola em repouso, então um envelope que nascesse em
/// zero faria o primeiro clique em "Arc" não mover NADA — um botão que parece morto na estreia.
/// (Era `EnvelopeWarp::DEFAULT_BEND`; virou const livre quando o enum virou apelido do catálogo.)
pub const ENVELOPE_DEFAULT_BEND: f64 = 0.5;

/// **O Envelope Object.** Um container que guarda a gaiola (cantos + lados) + a fonte autorada de
/// cada filho. A geometria que o mundo vê está no `VecPath` de cada filho (a cozida); esta struct é a
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
    /// Os 2 pontos de controle interiores de cada **lado** da gaiola: `edges[i]` vai de `corners[i]`
    /// a `corners[(i+1) % 4]`, nas mesmas coordenadas LOCAIS.
    ///
    /// ⚠️ **Em [`EnvelopeKind::Perspective`] eles são um FATO DERIVADO, não estado livre:** valem
    /// sempre os controles canônicos (⅓, ⅔) da gaiola atual, re-emitidos a cada movimento de canto
    /// (`ph2d_vec_envelope::rest_edges`, a porta única). Guardá-los mesmo assim é o que permite
    /// trocar para [`EnvelopeKind::Mesh`] e encontrar as alças **sobre os lados**, em vez de
    /// penduradas na gaiola que existia antes do último arrasto.
    pub edges: [[[f64; 2]; 2]; 4],
    /// Qual dos dois mapas esta gaiola aplica (ver [`EnvelopeKind`] — eles NÃO são o mesmo mapa).
    pub kind: EnvelopeKind,
    /// O preset que **gerou** esta gaiola, se ela veio de um — e o quanto (`bend ∈ [-1, 1]`).
    ///
    /// ⚠️ **Não é um segundo dono da gaiola.** `corners`/`edges` seguem sendo a única verdade sobre a
    /// forma; isto é a lembrança de *de onde eles vieram*, e existe por um motivo só: o slider **Bend**
    /// precisa saber o que re-carimbar. A derivação é de **mão única e por evento** (mudou o preset ou
    /// o bend ⇒ re-escreve a gaiola), nunca por frame — se fosse por frame, o preset e a mão do
    /// artista disputariam a mesma gaiola em todo `recook`.
    ///
    /// Arrastar QUALQUER alça **solta o preset** (`warp = None`): a gaiola passa a ser manual e o
    /// slider Bend deixa de ser oferecido. É o *"promovível"* que o ADR-0129 §4 pediu — e sem essa
    /// regra o próximo movimento do slider apagaria o que a mão acabou de fazer.
    pub warp: Option<EnvelopeWarp>,
    /// Os **pinos** do gesto `Pins`: `[onde estava, para onde foi]`, em coordenadas LOCAIS do
    /// container. Ignorados nos outros dois gestos — e preservados por eles, para trocar de gesto e
    /// voltar não apagar o que o artista pregou.
    pub pins: Vec<[[f64; 2]; 2]>,
    /// O `bend` do preset, `[-1, 1]`. Sem sentido (e ignorado) quando `warp` é `None`.
    ///
    /// Nasce em [`ENVELOPE_DEFAULT_BEND`], não em zero: um preset carimbado com força zero é a
    /// gaiola em repouso ao bit (há gate), então o **primeiro** clique num preset não moveria um
    /// pixel e pareceria um botão morto.
    pub bend: f64,
    /// Os filhos deformados por esta gaiola — um para um envelope de forma única (`N=1`), vários para
    /// um *warp group*. Cada um carrega o seu path + a sua fonte (ver [`VecEnvelopeChild`]).
    pub children: Vec<VecEnvelopeChild>,
}

impl SimComponent for VecEnvelope {}

impl VecEnvelope {
    /// Um envelope novo sobre `children` (cada um com os bytes postcard da sua fonte LOCAL), com a
    /// gaiola em **repouso** (`corners` = cantos da bbox-união das fontes, `edges` = os controles
    /// retos deles) — as formas não mudam até o artista arrastar uma alça. É o certo: um envelope que
    /// nasce deformado desorienta; um que nasce transparente mostra a gaiola e espera o gesto.
    ///
    /// Os `edges` chegam **de fora** (`ph2d_vec_envelope::rest_edges`) em vez de serem computados
    /// aqui: "o que é um lado reto" é matemática do motor de deformação, e recalculá-la nesta crate
    /// seria uma segunda porta para a pergunta que decide se o repouso é a identidade EXATA.
    /// Nasce em [`EnvelopeKind::Perspective`] — o gesto que o envelope sempre teve.
    #[must_use]
    pub fn at_rest(
        children: Vec<VecEnvelopeChild>,
        corners: [[f64; 2]; 4],
        edges: [[[f64; 2]; 2]; 4],
    ) -> Self {
        Self {
            corners,
            edges,
            kind: EnvelopeKind::default(),
            warp: None,
            pins: Vec::new(),
            bend: ENVELOPE_DEFAULT_BEND,
            children,
        }
    }
}
