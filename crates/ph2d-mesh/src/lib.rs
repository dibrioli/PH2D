#![forbid(unsafe_code)]
//! `ph2d-mesh` — **a malha residente** do módulo de escultura 3D.
//!
//! O ADR-0150 decide que *a malha é a representação primária e o campo (SDF) é
//! um gerador*. Esta crate é essa malha: buffers SoA, adjacência CSR, normais e
//! um octree de consulta — mais o import de OBJ, para haver o que esculpir.
//!
//! # O que esta crate NÃO sabe
//!
//! Que existe uma GPU, uma `Tool`, um painel ou um documento. Ela é **leaf**, e
//! a única dependência é o `rayon` — que entrou porque a sonda do M3 mediu as
//! normais em 67% do custo de um dab, não porque estava previsto. Quem fala com
//! o device é a `ph2d-mesh-render`;
//! quem esculpe é a `ph2d-sculpt3d`. Apagar as três apaga o módulo, que é a
//! promessa de removibilidade do `docs/3D/02.3` — e uma promessa dessas só é
//! verdadeira se for construída antes da primeira linha, nunca depois.
//!
//! # A referência
//!
//! O [SculptGL](https://github.com/stephomi/sculptgl) de Stéphane Ginier (o
//! autor do Nomad Sculpt) é **MIT**, então pode ser lido *e adaptado*. Todo
//! arquivo que adapta um algoritmo dele **nomeia o arquivo-fonte no cabeçalho**,
//! e o aviso de licença viaja junto em `LICENSES/sculptgl-MIT.txt`. Onde
//! divergimos — e divergimos em quatro lugares — o comentário diz *por quê*,
//! porque uma divergência silenciosa de uma referência é indistinguível de um
//! erro de porte.
//!
//! # Onde este código roda
//!
//! Na **CPU**. Não por falta de ambição e não por herança do SculptGL (que é CPU
//! porque o WebGL não lhe deu escolha): um dab é limitado pela **PEGADA** — ele
//! toca os vértices sob o pincel, e esse número não cresce quando a malha
//! cresce —, enquanto o que é de malha inteira (remesh, subdivisão, decimação)
//! é sequencial e alocador, que é o trabalho que uma GPU faz *mal*. A decisão
//! completa, com a porta única e o kill-criterion escrito ANTES do build, está
//! em `docs/3D/03.5`.

mod aabb;
mod adjacency;
mod edges;
mod face;
mod mesh;
mod multires;
mod normals;
mod obj;
mod octree;
mod ray;
mod subdivide;

pub use aabb::Aabb;
pub use adjacency::{Adjacency, Csr};
pub use edges::Edges;
pub use face::{Face, TRI};
pub use mesh::{DEFAULT_COLOR, DEFAULT_MASK, Mesh, MeshError, QueryScratch, RegionScratch};
pub use multires::{DetachedLevel, Multires};
pub use normals::{
    PAR_MIN, face_normal, face_normals_of, recompute_face_normals, recompute_vertex_normals,
    vertex_normals_of,
};
pub use obj::{ObjError, import_obj};
pub use octree::{Octree, RefitScratch};
pub use ray::{Hit, Ray};
pub use subdivide::{Lerpable, Predicted, predict, subdivide};

/// Geometria de teste — um cubo e uma esfera UV.
///
/// Mora no `src/` e não num `tests/` porque **as sondas de medição também
/// precisam dela**, e uma fixture duplicada entre o gate e a sonda é como as
/// duas passam a medir malhas diferentes sem ninguém notar.
pub mod shapes;

/// Geometria de teste **malformada** — borda, valência baixa, face degenerada,
/// sliver.
///
/// Módulo irmão da [`shapes`] e não uma extensão dela: aquela só sabe fazer
/// malha fechada e bem formada, e é por isso que a classe de defeito inteira que
/// vive aqui era invisível a todo gate do módulo. A separação é por
/// responsabilidade — *o que a malha é quando está certa* × *o que ela é quando
/// o arquivo de terceiro está errado*.
pub mod shapes_open;
