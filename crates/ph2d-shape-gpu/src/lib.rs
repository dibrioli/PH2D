//! ⭐⭐⭐ **O PASSE DE FORMAS INSTANCIADO** — `N` cópias de geometrias vectoriais numa chamada da
//! placa, com a área de cada pixel calculada **como o rasterizador fino do Vello a calcula**
//! ([doc 121 do Motion](../../docs/Motion%20Nodes/121_as_formas_na_placa.md)).
//!
//! ## Por que existe
//!
//! Uma forma viva do Motion era encodada na cena Vello **uma cópia de cada vez**: o encode, a
//! rasterização e — pior — o cozimento inteiro na CPU escalavam com `N`. Aqui cada geometria
//! DISTINTA é preparada uma vez ([`ShapeGeometry::prepare`]) e a placa desenha as cópias: um quad
//! por cópia, e o fragmento soma a contribuição de cada segmento para a área do pixel.
//!
//! ## A nitidez, e por que é a MESMA
//!
//! ⭐ **A conta da área é portada do `vello_shaders` (`fine.wgsl`, `Apache-2.0 OR MIT`)** — a
//! triagem de licença parou na primeira porta aberta, e a paridade com o que o artista vê hoje vem
//! por construção. As curvas são aplanadas por níveis ([`geometry::LEVELS`]), e o shader de vértice
//! escolhe, por cópia, o mais grosso cujo erro no ecrã cabe nos `0,25 px` que o próprio Vello
//! aceita ao aplanar. ⇒ nítido em qualquer zoom, sem assar nada.
//!
//! ## O que NÃO faz
//!
//! Tinta que não é uma cor (gradiente, padrão), mistura em grupo, e o quad de IMAGEM do passe
//! vectorial — quem chama decide e manda esses ao Vello, como hoje (doc 121 §2.1).

mod blocos;
mod contorno;
mod eixo;
mod geometry;
mod pass;

pub use blocos::{BlocoDeSegmentos, SEGS_POR_BLOCO};
pub use contorno::AREA_MINIMA_CONFORME;
pub use eixo::EixoItem;
pub use geometry::{
    FLAG_EVEN_ODD, FLAG_SO_CONFORME, FillRule, GeometryRecord, LEVELS, ShapeGeometry, ShapeInput,
    StrokeInput, TOL_BASE, TOL_STEP,
};
pub use pass::{Copias, ShapeInstance, ShapePass, ShapeView, VarianteDoPasse};
