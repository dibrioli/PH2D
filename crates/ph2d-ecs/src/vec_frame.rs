//! **A MOLDURA** — o contêiner: uma tela, um card, um painel.
//!
//! É a peça que o censo do plano de UI/UX mostrou **não existir** (`grep artboard` = 0), e sem ela
//! não há *"redimensionar para quê?"*: layout, âncora, exportação, preview de dispositivo e a
//! própria noção de *"uma tela"* penduram-se nela.
//!
//! # Uma moldura é um RETÂNGULO VIVO que ganhou um componente
//!
//! Não é um tipo novo de objeto. A entidade é a mesma que a ferramenta de forma produz — um
//! `VecPathRef` com `VecShape::Param { kind: Rectangle, w, h, .. }` —, e este componente só
//! acrescenta *o que ela FAZ com os filhos*.
//!
//! ⚠️ **Ele NÃO tem `size`, e isso é a decisão inteira.** O tamanho é o `w`/`h` do
//! [`crate::VecShape`] que a entidade já carrega. Dois tamanhos divergem no primeiro arrasto de
//! alça, e o modo de falha é o pior que existe: o desenho concorda com um e o layout com o outro, e
//! nada parece errado. Como consequência de ser um retângulo, saem **de graça** o fill, o gradiente,
//! o traço, o raio de canto vivo, a pilha de efeitos, o gizmo de escala, o hit-test, o z-order, o
//! undo e o save.
//!
//! # O recorte SAIU daqui (2026-08-21)
//!
//! ⚠️ Este componente teve um campo `clip: bool`, e ele mora agora no [`crate::VecClipContent`].
//! O motivo é o pedido do Enio — *"coloque a feature Clip Content para qualquer forma vetorial
//! fechada"* — encontrando o que a moldura de facto é: **ser uma moldura não é só recortar**. A
//! presença deste componente também dá o rótulo flutuante com o nome, as alças de redimensionar,
//! e a elegibilidade a auto layout e a âncoras. Uma estrela que quisesse recortar teria de virar
//! moldura para consegui-lo, e receberia um nome flutuante em cima.
//!
//! O doc-comment do campo já dizia que as duas perguntas eram independentes (*"«isto é um
//! contêiner?» e «ele esconde o que sai?»"*); elas eram independentes na prosa e **acopladas no
//! tipo**. Hoje são dois componentes, e uma moldura é simplesmente uma entidade que carrega os
//! dois — como nasce, por default.
//!
//! O mecanismo do recorte (a camada do Vello, o intervalo de z, e por que não é o
//! `ClipChildren` do pipeline de sprite) está escrito uma vez só, lá.
//!
//! # O que NÃO está aqui, e por quê
//!
//! ⚠️ O plano previa um segundo campo, `is_screen` (*"esta moldura é uma raiz de exportação"*).
//! Ele **não foi construído**: hoje nada a jusante o consumiria, e um checkbox que não muda nada é
//! exatamente o controle morto que a política de UI deste repo existe para impedir (*a SEQUÊNCIA
//! tem de levar a algum lugar*). O campo nasce junto com a exportação, que é wave própria — e como
//! apender campo a componente EXISTENTE bumpa o schema, o custo desse adiamento está nomeado: um
//! bump na wave que o trouxer.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A moldura.** A entidade que a carrega é um retângulo vivo comum; este componente diz que ela
/// CONTÉM — que os filhos dela são *conteúdo*, e não vizinhos que por acaso estão por cima.
///
/// Ausência do componente = retângulo comum, e o mundo é byte-idêntico ao de antes desta feature.
///
/// ⚠️ **Sem recorte aqui.** Uma moldura que não esconde o transbordo continua inteiramente uma
/// moldura (a dona de um tamanho autorado, a raiz do que for responsivo, a que mostra o que
/// transborda enquanto o artista compõe) — ela só não carrega o [`crate::VecClipContent`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecFrame;

impl SimComponent for VecFrame {}
