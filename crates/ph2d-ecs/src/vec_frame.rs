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
//! # O recorte NÃO é o [`crate::ClipChildren`]
//!
//! ⚠️ O plano supunha delegar a ele; a leitura do código diz o contrário. `ClipChildren` é do
//! pipeline de **SPRITE** (o passe de stencil em `ph2d-render`), e ele não alcança um caminho
//! vetorial — quem desenha vetor é o Vello, por `ph2d-vec-render::dispatch`. O recorte de uma
//! moldura é uma **camada de clip do Vello** (`VectorScene::push_clip`), a mesma que os painéis
//! roláveis do editor já usam; quem a abre e fecha é o renderer, sobre o intervalo contíguo que a
//! sub-árvore da moldura ocupa na pilha de z.
//!
//! Corolário: uma moldura recorta os **descendentes VETORIAIS**. Um sprite filho é do outro
//! renderer e continua com o `ClipChildren` dele — os dois recortes existem, e cada um é do
//! desenhista que o entende.
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
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecFrame {
    /// Recorta os descendentes à silhueta da moldura.
    ///
    /// ⚠️ Desligado, a moldura **continua sendo uma moldura** (a dona de um tamanho autorado, a
    /// raiz do que for responsivo) — ela só deixa de esconder o transbordo. É o *"Clip content"* do
    /// Figma: a pergunta *"isto é um contêiner?"* (a presença do componente) e a pergunta *"ele
    /// esconde o que sai?"* (este campo) são independentes, e colapsá-las tiraria do artista a
    /// moldura que mostra o que transborda enquanto ele compõe.
    pub clip: bool,
}

impl SimComponent for VecFrame {}
