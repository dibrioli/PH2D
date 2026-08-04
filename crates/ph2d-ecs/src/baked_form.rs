//! `BakedForm` — a entidade tem uma **forma assada** (o G-buffer de uma malha, virado canal do
//! sprite).
//!
//! Gêmeo exato do [`crate::PaintedDoc`], e pela mesma razão: um sprite assado é
//! `SpriteSource::Individual { texture_id }`, e esse `texture_id` é um id de runtime da GPU — noutra
//! sessão ele aponta para um slot vazio. Sem uma identidade que sobreviva ao restore não há a quem
//! devolver os canais, e o objeto reabriria com a textura morta do save.
//!
//! ## O que ela compra, e por que não é o mesmo que o Painter
//!
//! O documento do Painter guarda **como continuar pintando**. Este guarda **como continuar
//! ACENDENDO**: o `base` (os pixels antes de qualquer luz) e a `form` (as normais que a malha doou).
//! Com os dois, mover a lâmpada re-acende o objeto sem re-rasterizar malha nenhuma — e é isso que
//! `docs/3D/02.2` chama de *rota A*: a geometria some do build, o objeto continua reluminável.
//!
//! ⚠️ **É o oposto de assar pixels acesos.** Um bake que só gravasse o resultado entregaria uma
//! sprite que o artista não pode mais iluminar, e iluminar é a palavra inteira do objetivo 2.
//!
//! O `u32` é caller-supplied (a shell aloca o próximo livre), como a célula de atlas e como o
//! `PaintedDoc` — e, como toda componente, viaja no `WorldSnapshot`, logo o **undo** também a
//! preserva de graça.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Identidade estável dos canais assados desta entidade.
///
/// Um `u32` cru (e não um tipo do módulo 3D) para manter `ph2d-ecs` sem dependência dele — a direção
/// da seta importa, e aqui ela importa **duas vezes**: o módulo 3D é removível por feature, então
/// uma componente que soubesse o que ele é sairia do build junto e levaria embora o objeto assado
/// de todo projeto já salvo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BakedForm(pub u32);

impl SimComponent for BakedForm {}
