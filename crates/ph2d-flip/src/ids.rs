//! Ids tipados do documento Flip.
//!
//! Duas naturezas, de propósito:
//! - [`DrawingId`] é **POSICIONAL** — um índice no array de drawings do objeto,
//!   espelhando o `drawing_index` do Grease Pencil (GPv3). Muda quando
//!   [`crate::FlipObject::remove_unused_drawings`] compacta (que remapeia todos
//!   os frames). Nunca guarde um `DrawingId` atravessando uma compactação.
//! - [`LayerId`], [`MaterialId`] e [`FlipObjectId`] são **ESTÁVEIS** — não mudam
//!   com reorder/remoção de vizinhos, então referências (máscara aponta camada,
//!   entidade ECS aponta objeto) sobrevivem à edição.

use serde::{Deserialize, Serialize};

/// Índice de um `FlipDrawing` no array de drawings do objeto (posicional —
/// remapeado na compactação). Um `Frame` referencia o desenho por este índice;
/// vários frames podem apontar o mesmo (instância / ciclo).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DrawingId(pub u32);

/// Id estável de uma camada dentro de um objeto. Máscaras referenciam camadas
/// por este id (o GP referencia por nome; id é mais robusto a rename).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u32);

/// Id estável de um material (paleta de traço/fill). `0` = material default
/// (o que `Default` devolve). A paleta em si não é modelada no W0 (materiais
/// ricos vêm sob demanda, W1/W4); o stroke só carrega a referência.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MaterialId(pub u32);

/// Id estável de um objeto Flip dentro da cena ([`crate::FlipDoc`]). É o que a
/// entidade ECS carrega (`ph2d_ecs::FlipObjectRef(u64)`), por isso `u64` — a
/// direção da seta importa: a shell conhece objeto e entidade, nenhum conhece o
/// outro (espelha `VecPathRef`/`VecPathId`, ADR-0110).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FlipObjectId(pub u64);

/// Número do quadro na tira — a **chave** do mapa de frames, e o início de um
/// desenho (a duração é implícita: segura até a próxima chave). `i32` como no GP
/// (`FramesMapKeyT`), aceita quadros negativos.
pub type Frame = i32;
