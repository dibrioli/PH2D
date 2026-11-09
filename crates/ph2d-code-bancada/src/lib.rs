//! ⭐ **O que o editor de scripts sabe da CENA** (plano 13 Onda B, a Bancada,
//! `docs/CodeEditor/13_plano_bancada.md` §4): os nomes dos objectos ([`names`]), quem corre o script
//! aberto e com que valores ([`who`]), o clique da grelha virado na edição do Inspector ([`act`]) e
//! o pedido de contorno à cena ([`scene`]). Folha: devolve os tipos de `ph2d-code-viewmodel`.

pub mod act;
pub mod names;
pub mod scene;
pub mod who;

pub use act::{act, parse_number};
pub use scene::{SceneOutlines, ScenePeek};
pub use who::{Bancada, Open, TextFacts, number_text, read, text_of};
