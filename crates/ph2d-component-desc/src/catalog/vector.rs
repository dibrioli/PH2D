//! **A família do vetor** — os 32 `Vec*` registados.
//!
//! ⚠️ **Nenhum destes é hoje alcançável pelo Inspector**: a auditoria de 2026-08-21 mediu que
//! 31 dos 36 tipos ausentes do Inspector são desta família, editados por **outro painel
//! artesanal** (`ph2d-panel-vector`). Declará-los aqui não os põe no Inspector — põe-nos na
//! paleta do `+`, que é a pergunta *"que componentes existem para este objeto?"*, e essa
//! pergunta tinha 31 respostas invisíveis.
//!
//! ⚠️ **`applies_to` é `VECTOR` para todos, e isto é uma afirmação a MEDIR no smoke**, não uma
//! medição: o critério do ADR-0166 é *"o tipo cujo marcador o componente lê"*, e o marcador
//! aqui é o `VecPathRef`. O que não está provado é o inverso — se algum destes tem efeito
//! sobre um objeto que não é um caminho vetorial. Onde isso aparecer, corrija a linha e
//! escreva a razão.
//!
//! ⚠️ **`VecComponentMain`/`VecInstance` são a instância VETORIAL de hoje, e a F4 subsume-os**
//! (ADR-0164 §4): quando o mecanismo geral existir, estas duas linhas saem daqui e o que fica
//! é `ObjectInstance` na família `Instancing`. Ficam declaradas `Machinery` — o artista já as
//! opera por verbos (*Create/Place/Detach*), nunca anexando o componente.

use crate::{ComponentCategory as C, ComponentDesc as D, ObjectKinds as O};

/// Um `Vec*` que o `+` OFERECE: tem `Default`, logo a paleta consegue construí-lo no ponto
/// neutro. Sempre `Vector`, sempre sobre um caminho, ainda sem campos descritos.
const fn v(canonical_name: &'static str, display_name: &'static str) -> D {
    D::authored(canonical_name, display_name, C::Vector, O::VECTOR, &[])
}

/// Um `Vec*` que chega com o GESTO — **não tem `Default`**, e a lista abaixo não foi
/// escolhida: ela é a saída do compilador ao converter os registradores para
/// `register_default` (`the trait bound X: Default is not satisfied`). Para estes não há
/// neutro que signifique alguma coisa — uma `VecShape` sem geometria não é uma forma vazia,
/// não é uma forma.
const fn g(canonical_name: &'static str, display_name: &'static str) -> D {
    D::intrinsic(canonical_name, display_name, C::Vector, &[])
}

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    g("ph2d::ecs::VecAnchors", "Anchors"),
    v("ph2d::ecs::VecBindings", "Bindings"),
    g("ph2d::ecs::VecBlend", "Blend"),
    g("ph2d::ecs::VecBoolGroup", "Boolean Group"),
    g("ph2d::ecs::VecBoolOp", "Boolean Op"),
    g("ph2d::ecs::VecClipContent", "Clip Content"),
    // A instância vetorial de hoje — subsumida pela F4 (ADR-0164 §4).
    D::machinery(
        "ph2d::ecs::VecComponentMain",
        "Component Main",
        C::Instancing,
    ),
    g("ph2d::ecs::VecConnector", "Connector"),
    v("ph2d::ecs::VecContour", "Contour"),
    v("ph2d::ecs::VecCutPath", "Cut Path"),
    g("ph2d::ecs::VecEnvelope", "Envelope"),
    v("ph2d::ecs::VecFilter", "Filter"),
    g("ph2d::ecs::VecFrame", "Frame"),
    D::machinery("ph2d::ecs::VecInstance", "Instance", C::Instancing),
    // ⚠️ `VecLabel.host` é um `VecPathId` cru (correção de 2026-08-21 ao doc 01 §1.3: NÃO é
    // um hash de nome) ⇒ `RefKind::VecPath` quando o campo for descrito, e entra no remap da
    // F4 como as juntas da física.
    g("ph2d::ecs::VecLabel", "Label"),
    v("ph2d::ecs::VecLayout", "Auto Layout"),
    v("ph2d::ecs::VecLayoutAbsolute", "Layout Absolute"),
    v("ph2d::ecs::VecLayoutItem", "Layout Item"),
    v("ph2d::ecs::VecLayoutSize", "Layout Size"),
    g("ph2d::ecs::VecMorph", "Morph"),
    g("ph2d::ecs::VecMorphMachine", "Morph States"),
    g("ph2d::ecs::VecOffset", "Offset"),
    v("ph2d::ecs::VecPatternPath", "Pattern Path"),
    v("ph2d::ecs::VecPatternRotation", "Pattern Rotation"),
    g("ph2d::ecs::VecResizeBox", "Resize Box"),
    g("ph2d::ecs::VecShape", "Shape"),
    v("ph2d::ecs::VecStrokeProfile", "Stroke Profile"),
    v("ph2d::ecs::VecSymmetry", "Symmetry"),
    g("ph2d::ecs::VecTextPath", "Text on Path"),
    g("ph2d::ecs::VecWidget", "Widget"),
    v("ph2d::ecs::VecWidgetBind", "Widget Bind"),
    g("ph2d::ecs::VecWidgetIcon", "Widget Icon"),
    v("ph2d::ecs::VecWidgetValue", "Widget Value"),
];
