//! **A BOOLEANA VIVA** — um grupo cujos filhos se combinam, e continuam editáveis.
//!
//! Irmão do [`crate::VecOffset`] / [`crate::VecSymmetry`] no padrão que esta linha já usou meia
//! dúzia de vezes: o componente guarda a **relação** (*estes filhos combinam com esta operação*) e
//! a aparência é uma **função pura** dela, re-cozida pela shell a cada frame. Consequência de
//! graça: **undo e save cobrem a booleana sem uma linha a mais** — os dois capturam o mundo ECS, e
//! este componente está registrado no `ComponentRegistry`.
//!
//! # Ele mora no GRUPO, e o grupo NÃO tem geometria
//!
//! É o modelo do Illustrator e do Figma: uma booleana é um **grupo cujos filhos são os
//! operandos**, e é isso que os mantém selecionáveis, editáveis e reordenáveis depois da operação.
//!
//! ⚠️ Um grupo neste editor é *"uma entidade **sem geometria própria** (nem `VecPathRef` nem
//! sprite) que tem filhos"* (`vec_entities::ungroup_entities`) — ele nasce com `Transform`, `Name`
//! e `RootOrder`, e mais nada. Logo **o grupo não tem `VecPathId`**, e a `LiveGeometry` — que é
//! um mapa `VecPathId -> Vec<VecPath>` — não pode ser chaveada por ele.
//!
//! O resultado é então desenhado no id da **BASE** (o operando mais ao FUNDO), e os demais
//! operandos recebem uma lista **vazia**, que o `dispatch` desenha como nada. Não é um arranjo:
//! é exatamente a regra que a booleana DESTRUTIVA já segue — *"a base é a de trás: o resultado
//! ocupa a fatia de z dela (não salta pro topo)"* (`apply_vec_boolean`) — e é o que faz o **Apply**
//! não mover a arte um pixel.
//!
//! # A operação é o vocabulário do ARTISTA
//!
//! `op` é o discriminante de `ph2d_vec_boolean::PathfinderOp` (as quatro de conjunto + as quatro
//! receitas), **não** o do `BoolOp` do motor. O painel oferece oito botões, e um grupo vivo tem de
//! poder ser qualquer um deles — inclusive os que não são operações de conjunto (o Trim devolve
//! uma forma por fonte). `ph2d-ecs` não depende do vetor, então o que viaja aqui é o `u8` cru,
//! como o `VecShape::Param{kind}` faz com o catálogo de formas.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A booleana viva de um grupo.** A entidade que o carrega é um grupo comum (`Transform` +
/// `Name` + filhos); este componente diz apenas *com que operação* os filhos dele se combinam.
///
/// Ausência do componente = grupo comum, e os filhos desenham cada um por si — que é o mundo
/// byte-idêntico ao de antes desta feature.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBoolGroup {
    /// O discriminante de `ph2d_vec_boolean::PathfinderOp` (append-only: é o que fica gravado no
    /// save). Um código que este build não conhece faz o grupo desenhar **como grupo comum** —
    /// degradar para *os filhos aparecem* é a única leitura que não perde arte.
    pub op: u8,
}

impl SimComponent for VecBoolGroup {}
