//! ⭐⭐ **O ESQUELETO VIVO NO DOCUMENTO** — prender uma forma (ou uma imagem) aos ossos, e
//! responder o que a corrente deles é.
//!
//! # Por que esta folha existe, e por que ela NÃO é a `ph2d-skeleton-ecs`
//!
//! O esqueleto é módulo próprio desde [ADR-0169], com a lei em `ph2d-skeleton` e os componentes em
//! `ph2d-skeleton-ecs`. O que vivia na shell era o degrau do meio: as operações **sobre o mundo**
//! — `bind`, `bind_image`, `bone_segments`, `chain_to` — e elas são **puras** (`SimWorld` mais
//! tipos de crates; zero `App`, zero `gfx`).
//!
//! ⛔⛔ **Elas não podem morar na `ph2d-skeleton-ecs`, e o motivo é um CICLO:** o `bind` precisa do
//! mapa `caminho ⟺ entidade` da [`ph2d_vec_entities`], e essa crate **já depende** da
//! `ph2d-skeleton-ecs` (o `settle_origins` dela salta uma forma presa a um osso). Pôr a lei lá
//! fecharia `skeleton-ecs → vec-entities → skeleton-ecs`, que o cargo recusa. *Uma folha nova não
//! é escolha de gosto quando a alternativa é um ciclo.*
//!
//! # O que a tirou da shell
//!
//! A cena `PH2D_VEC_BONE_SMOKE` é um dos **cinco** roteadores da família `vec`, e a catraca
//! `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` é **all-or-nothing por família**: enquanto uma cena
//! precisasse de um módulo da shell, **nenhum** dos cinco podia ser declarado. Esta crate é o que
//! fecha essa conta.
//!
//! ⚠️ **Os módulos da shell NÃO foram apagados** — `skeleton_live` e `skeleton_skin_image` ficaram
//! como re-exportação de uma linha, e os **21** ficheiros que os nomeiam continuam byte a byte
//! iguais ([HOWTO §1.2]).
//!
//! [ADR-0169]: ../../../docs/architecture/decisions/0169-the-skeleton-is-its-own-module-and-each-medium-answers-only-what-a-point-is.md
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

pub mod bone;
pub mod goal;
pub mod skin_image;
pub mod skin_live;
