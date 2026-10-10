//! **Presets de pincel em ficheiro** — `*.ph2d-brush.png` (`docs/Painter/NewFeaturesWetPaint/02 §5`).
//!
//! Um preset é UM ficheiro portátil: um PNG 256² cuja imagem é o traço do preset pintado pelo motor,
//! com os parâmetros num `iTXt` `ph2d-brush` (JSON, `schema` 1), a origem num `tEXt` `ph2d-origin` e as
//! texturas NOSSAS em chunks privados `phTx`. Qualquer visualizador mostra a miniatura; o artista
//! arrasta, partilha e vê o ficheiro na pasta.
//!
//! Esta crate é SÓ o formato e a biblioteca (pastas = grupos, favoritos): não conhece o
//! motor. Aplicar, capturar e pintar a miniatura vivem do lado da ferramenta, que depende desta.
//!
//! Leis do formato:
//! - `escrever(ler(b)) == b` para todo ficheiro que esta crate escreveu (JSON canónico: chaves por
//!   ordem, `f64` pelo menor texto que volta ao mesmo bit; zlib de UMA implementação).
//! - Campo desconhecido NÃO falha a leitura e sobrevive à regravação (as ondas seguintes acrescentam
//!   campos sem subir o `schema`). `schema` maior que [`SCHEMA`] é recusado com o número na mensagem.

mod biblioteca;
mod estado;
mod modelo;
mod png;

pub use biblioteca::{
    Biblioteca, EXTENSAO, Entrada, apagar, caminho_relativo, gravar, pasta_do_utilizador,
};
pub use estado::{EstadoDoUtilizador, FICHEIRO_ESTADO, Lembrado, Politica};
pub use modelo::{
    Erro, LADO_MINIATURA, Miniatura, Origem, Parametros, Preset, SCHEMA, Tilt, Volumes,
};
pub use png::{escrever, escrever_textura, ler, ler_textura};
