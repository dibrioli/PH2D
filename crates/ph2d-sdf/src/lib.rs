#![forbid(unsafe_code)]
//! `ph2d-sdf` — o **voxel remesh** do módulo de escultura 3D.
//!
//! Uma malha entra, vira um campo de distância com sinal, e sai outra malha:
//! topologia uniforme, sem auto-interseção, com a densidade que o artista pediu.
//! É o que devolve uma superfície esculpível depois que o barro foi esticado
//! além do que a malha original conseguia representar.
//!
//! É a **quarta crate** do módulo, ao lado de `ph2d-mesh`, `ph2d-mesh-render` e
//! `ph2d-sculpt3d`. Apagar as quatro apaga o módulo, que é a promessa de
//! removibilidade do `docs/3D/02.3` — esta crate a mantém verdadeira porque a
//! única dependência dela é a malha, e nada fora do módulo depende dela.
//!
//! ⚠️ **O doc anterior desta crate prometia outra coisa** — *"signed-distance-
//! field utilities (used by lighting + UI), empty pending M13"*. O M13 nunca
//! chegou e a crate ficou quatro linhas. O nome continua certo (um campo de
//! distância COM SINAL é literalmente o que vive aqui), mas a promessa era de um
//! consumidor que não existe, e uma promessa dessas é pior que um módulo vazio:
//! ela faz a próxima LLM procurar o código de iluminação que ninguém escreveu.
//!
//! # A referência
//!
//! `editing/{Remesh,SurfaceNets}.js` do [SculptGL](https://github.com/stephomi/sculptgl)
//! (MIT, o mesmo autor do Nomad Sculpt), com o aviso de licença em
//! `LICENSES/sculptgl-MIT.txt`. Toda divergência tem o *porquê* escrito ao lado
//! — uma divergência silenciosa de uma referência é indistinguível de um erro
//! de porte.
//!
//! # Onde este código roda
//!
//! Na **CPU**, e o remesh é **serial**: o flood fill é sequencial por semântica
//! e as caixas de dois triângulos se sobrepõem, então a escrita do voxelizador
//! não é disjunta — a condição que o ADR-0109 exige para paralelizar sem mudar
//! a resposta. O custo por resolução está na sonda `measure_remesh`, e é ele
//! que decide se vale abrir esse eixo.
//!
//! ⚠️ **O traço de AO é a exceção, e é ESTREITA** ([ADR-0156](../../../docs/architecture/decisions/0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md)):
//! ele é um *gather* por-vértice contra um campo imutável, com a soma privada e
//! de ordem fixa, então roda em `rayon` sendo **byte-idêntico ao serial** —
//! medido em 2, 4, 8, 16 e 32 threads. A autorização é do `bake_ao` e de mais
//! nada; qualquer outro uso de `rayon` aqui exige ADR próprio.

mod ao;
mod field;
mod remesh;
mod surface_nets;
mod thickness;

pub use ao::{AoParams, bake_ao};
pub use field::{DEFAULT_RESOLUTION, VoxelField};
pub use remesh::{RemeshError, RemeshReport, remesh, remesh_default};
pub use surface_nets::surface_nets;
pub use thickness::{DEFAULT_THICKNESS, at as thickness_at, bake as bake_thickness};
