//! ⭐⭐⭐ **A família LÓGICA** — o que faz um jogo acontecer **sem uma linha de script**.
//!
//! # Porque ela é um módulo próprio, e não uma linha no `core`
//!
//! É a lei declarada no [`super`]: **uma família, um módulo** (isolamento, DIRETRIZ §1.5.2.1).
//! O `core` descreve o que todo objecto **É** — identidade, pose, ordem; esta descreve o que ele
//! **FAZ**. E o corte paga-se já: a fila do
//! [levantamento](https://github.com/dibrioli/PH2D/blob/main/docs/Components/00_levantamento_componentes.md)
//! §7 traz mais **catorze** para aqui (`SensorZone` · `SignalActions` · `Spawner` · `Lifetime` ·
//! `Tags` · `StateMachine` · `ProjectileMotion` · …), e cada uma apende **sem encostar em mais
//! nada**.
//!
//! ⛔ **A alternativa era a categoria `Scripting`, e seria mentir**: o valor inteiro desta família
//! é *acontecer sem script*. Um artista que procure o relógio na secção de scripts conclui que
//! precisa de programar para o ter.
//!
//! ⚠️ **A lista está ORDENADA por `canonical_name`** — há gate (`the_catalog_is_sorted_and_unique`),
//! e fora de ordem a busca binária devolve `None` para um tipo que existe. *Um descritor que não é
//! encontrado lê-se exactamente como um descritor que não existe.*

use crate::{ComponentCategory as C, ComponentDesc, ComponentDesc as D, ObjectKinds as O};

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    // ⚠️ **`O::ANY`, e é a decisão**: um relógio serve a um sprite, a uma forma, a um objecto
    // VAZIO e a um grupo. Restringi-lo a `DRAWABLE` faria o objecto vazio — a entidade que o
    // artista usa como *«o cérebro da cena»* — não poder ter um.
    D::authored("ph2d::ecs::Timers", "Timers", C::Logic, O::ANY, &[]),
];
