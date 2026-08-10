//! O **botão**: uma malha entra, uma malha nova sai.
//!
//! Adaptado de `Remesh.remesh` do SculptGL (MIT). Licença em
//! `LICENSES/sculptgl-MIT.txt`.

use ph2d_mesh::{Mesh, MeshError, fill_holes};

use crate::field::{DEFAULT_RESOLUTION, VoxelField};
use crate::surface_nets::surface_nets;

/// Por que um remesh RECUSA.
///
/// ⚠️ **Ele não vive no `MeshError`**, e a razão é de dono: aquele enum fala de
/// uma malha malformada (um índice de face fora de alcance), e *"o campo saiu
/// sem interior"* é fato do CAMPO. Pendurá-lo lá faria a `ph2d-mesh` conhecer
/// voxel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemeshError {
    /// **A onda do flood fill alcançou TODA célula** — não sobrou dentro, logo
    /// não há nível zero, logo a extração devolveria uma malha vazia.
    ///
    /// ⚠️ **É alcançável no produto e não é um teto:** varrendo a esfera
    /// `uv(96,144)`, ONZE das cem resoluções entre 100 e 200 vazam (`112, 151,
    /// 160, 161, 168, 180, 181, 193, 194, 196, 197`), e o
    /// [`DEFAULT_RESOLUTION`] que shipa é **150** — vizinho de uma delas. Quem
    /// decide é o alinhamento da grade contra os triângulos, então outra malha
    /// vaza noutros números.
    NoInterior {
        /// A resolução pedida — o número que o artista pode mudar.
        resolution: u32,
        /// Quantas células a grade tinha, para o log poder dizer o tamanho.
        cells: usize,
    },
    /// **A onda alcançou QUASE toda célula** — sobrou um interior tão pequeno
    /// que a extração devolve um caco, não a peça.
    ///
    /// ⚠️ **Este é o irmão que faltava, e o log do produto o nomeou:**
    /// `567828 -> 40 vertices`. O [`Self::NoInterior`] pergunta *sobrou
    /// alguma coisa?*, e um vazamento parcial responde *sim* — passa pelo guard
    /// e o chamador instala o caco com log de SUCESSO, que é a pior forma de
    /// errado porque parece que funcionou.
    ///
    /// ⚠️ **A régua não é um palpite sobre "pouco":** uma malha FECHADA tem
    /// volume conhecido (teorema da divergência, `O(faces)`), e o campo devia
    /// encontrá-lo a menos do erro de discretização. Quando ele encontra uma
    /// fração ínfima disso, ele vazou — e isso vale para casca fina igual,
    /// porque as duas grandezas encolhem juntas.
    Leaked {
        /// A resolução pedida — o número que o artista pode mudar.
        resolution: u32,
        /// Que fração do volume da malha o campo achou, em partes por mil.
        ///
        /// Inteiro e não `f32` para o enum continuar `Eq` — um erro que se
        /// compara é um erro que um gate pode afirmar.
        found_per_mille: u32,
    },
    /// A malha reconstruída não pôde ser montada.
    Mesh(MeshError),
}

impl From<MeshError> for RemeshError {
    fn from(e: MeshError) -> Self {
        Self::Mesh(e)
    }
}

impl core::fmt::Display for RemeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoInterior { resolution, cells } => write!(
                f,
                "o campo saiu sem interior na resolução {resolution} ({cells} células): \
                 a onda alcançou tudo e não há superfície a extrair"
            ),
            Self::Leaked {
                resolution,
                found_per_mille,
            } => write!(
                f,
                "o campo vazou na resolução {resolution}: ele achou {}% do volume \
                 que a peça encerra, então a extração devolveria um caco",
                f32::from(u16::try_from(*found_per_mille).unwrap_or(u16::MAX)) / 10.0
            ),
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for RemeshError {}

/// **Que fração do volume da peça o campo tem de encontrar** para a
/// reconstrução ser aceita — ver [`RemeshError::Leaked`].
///
/// ⚠️ **MEDIDO, e o fosso é de duas ordens de grandeza**
/// (`tests/probe_leak.rs::what_fraction_of_the_volume_the_field_finds`, 361
/// resoluções de 40 a 400 em três malhas):
///
/// | malha | banda SÃ | fora da banda |
/// |---|---|---|
/// | esfera uv(96,144) | 0,9989 .. 1,0049 | 0 |
/// | cubo | 1,0000 .. 1,0000 | 0 |
/// | tubo aberto | 0,9939 .. 1,0111 | 2, **ambos em 0,0** |
///
/// O erro de discretização não passa de **1,1%** e um vazamento dá **zero**, então
/// meio volume fica 2× abaixo da pior amostra SÃ e ordens de grandeza acima de
/// qualquer colapso. Um número apertado seria escolher risco onde a medição não
/// pede nenhum.
///
/// ⚠️ **A assimetria é deliberada:** só o lado PEQUENO recusa. Uma malha com
/// winding invertido em parte da casca faz o volume assinado encolher e a razão
/// SUBIR — e uma razão alta não é sintoma de nada que a extração não saiba
/// tratar.
const MIN_INTERIOR_FRACTION: f32 = 0.5;

/// A fase da segunda tentativa, em frações de passo — o conjugado da razão
/// áurea, `1 − 1/φ`.
///
/// ⚠️ **Irracional de propósito.** O regime que vaza é o *remesh de um remesh*:
/// a saída do Surface Nets nasce alinhada à grade ANTERIOR, e a grade nova cai
/// quase-comensurável com ela. Uma fase racional (½, ¼) tem as próprias
/// coincidências nessa família; esta não é comensurável com nenhuma razão
/// simples entre as duas grades.
///
/// ⚠️ E o valor não é o que importa — **toda** fase não-nula curou os dois casos
/// medidos (a tabela vive no [`VoxelField::for_bounds_phased`]). Ele é escolhido
/// para não ter azar sistemático, não para acertar um alvo.
const GOLDEN_PHASE: f32 = 0.381_966;

/// O que o remesh fez, para quem quiser dizê-lo ao artista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemeshReport {
    /// Vértices antes e depois.
    pub verts: (usize, usize),
    /// Faces antes e depois.
    pub faces: (usize, usize),
    /// Quantos buracos foram tapados para o campo poder ter um dentro.
    pub holes_filled: usize,
    /// Células da grade — o número que explica o custo.
    pub cells: usize,
    /// A grade teve de ser DESLOCADA para o campo achar o interior?
    ///
    /// ⚠️ Ela é publicada em vez de ficar calada porque uma cura que ninguém vê
    /// é uma cura que ninguém pode reconferir: se este `true` passar a aparecer
    /// sempre, a degenerescência deixou de ser rara e a nota que diz *"~0,55% das
    /// resoluções"* precisa de outra medição.
    pub nudged: bool,
}

/// Reconstrói `mesh` por voxelização, na resolução dada.
///
/// # A ordem, e por que cada passo precede o seguinte
///
/// 1. **Tapar buracos.** Uma superfície com beira não tem dentro: o flood fill
///    entra pelo furo e o campo inteiro sai positivo, devolvendo nada. A malha
///    de entrada é clonada antes — tapar é uma exigência do ALGORITMO, não uma
///    edição que o artista pediu.
/// 2. **Voxelizar** — a distância, que é local.
/// 3. **Flood fill** — o sinal, que não é.
/// 4. **Extrair** a superfície de nível zero.
///
/// ⚠️ **Não há um 5º passo de re-alinhamento**, e a referência tem: lá os
/// vértices saem em coordenadas de GRADE e um `alignMeshBound` os re-escala pela
/// diagonal da caixa no fim. Aqui a extração já emite mundo (`origem + coord ×
/// passo`), então não há para onde re-alinhar — e some junto o erro de uma
/// escala derivada de uma diagonal, que não é a mesma coisa que a caixa.
pub fn remesh(mesh: &Mesh, resolution: u32) -> Result<(Mesh, RemeshReport), RemeshError> {
    let verts_before = mesh.vert_count();
    let faces_before = mesh.face_count();

    let mut closed = Mesh::from_parts(mesh.positions().to_vec(), mesh.faces().to_vec())?;
    let fill = fill_holes(&mut closed);

    // A régua: a malha aqui já está FECHADA, então o teorema da divergência dá
    // exatamente o espaço que ela encerra, e o campo devia encontrá-lo a menos
    // do erro de discretização.
    let want = ph2d_mesh::signed_volume(&closed).abs();
    let plausible = |f: &VoxelField, inside: usize| {
        let s = f.step();
        want <= 0.0 || inside as f32 * s * s * s >= want * MIN_INTERIOR_FRACTION
    };

    let mut field = VoxelField::for_bounds(closed.bounds(), resolution);
    field.voxelize(&closed);
    let mut inside = field.flood_fill();
    let mut nudged = false;

    // ⚠️ **A SEGUNDA FASE, e ela é a CURA — a recusa abaixo é só a rede.**
    // O vazamento é uma degenerescência numérica: uma amostra da grade cai
    // EXATAMENTE sobre a superfície, a travessia pousa na fronteira entre duas
    // janelas de aresta consecutivas, o arredondamento a expulsa das duas, e o
    // interior escoa. Re-amostrar noutra FASE não muda o modelo — muda só onde
    // a grade pergunta —, e a medição diz que **toda** fase não-nula cura os
    // dois casos que reproduzem (ver [`VoxelField::for_bounds_phased`]).
    //
    // ⚠️ **O custo é pago só quando a doença aparece** (~0,55% das resoluções na
    // forma mais hostil que medimos), e a fase é IRRACIONAL de propósito: o
    // conjugado da razão áurea não é comensurável com nenhuma relação racional
    // entre a grade e uma malha que já saiu de outra grade — que é exatamente a
    // situação do remesh-de-remesh.
    if !plausible(&field, inside) {
        let mut second = VoxelField::for_bounds_phased(closed.bounds(), resolution, GOLDEN_PHASE);
        second.voxelize(&closed);
        let second_inside = second.flood_fill();
        if plausible(&second, second_inside) {
            field = second;
            inside = second_inside;
            nudged = true;
        }
    }

    let cells = field.cell_count();

    // ⚠️ **A RECUSA, e por que ela vem ANTES da extração.** Sem interior o
    // `surface_nets` devolve uma malha VAZIA sem errar — e o chamador a instala
    // no lugar da escultura, com log de sucesso. Recusar aqui custa a extração
    // que não teria o que extrair, e devolve ao artista a única coisa que ele
    // pode usar: o nome da causa e o número que ele controla.
    if inside == 0 {
        return Err(RemeshError::NoInterior { resolution, cells });
    }

    // ⚠️ **E o guard acima NÃO basta, e o produto o provou.** Um campo que vaza
    // *quase* todo deixa um punhado de células dentro: `inside` não é zero, a
    // extração devolve um caco, e o chamador o instala com log de SUCESSO. Foi
    // literalmente isto no log do Enio (2026-08-10):
    // `567828 -> 40 vertices`.
    //
    // A régua é o VOLUME, e ela não é um palpite sobre "pouco": a malha aqui já
    // está FECHADA (o passo 1), então o teorema da divergência dá exatamente o
    // espaço que ela encerra, e o campo devia encontrá-lo a menos do erro de
    // discretização.
    if !plausible(&field, inside) {
        let s = field.step();
        let got = inside as f32 * s * s * s;
        let per_mille = (got / want * 1000.0).clamp(0.0, 1000.0) as u32;
        return Err(RemeshError::Leaked {
            resolution,
            found_per_mille: per_mille,
        });
    }

    let out = surface_nets(&field)?;
    let report = RemeshReport {
        verts: (verts_before, out.vert_count()),
        faces: (faces_before, out.face_count()),
        holes_filled: fill.filled(),
        cells,
        nudged,
    };
    Ok((out, report))
}

/// [`remesh`] na resolução da referência.
pub fn remesh_default(mesh: &Mesh) -> Result<(Mesh, RemeshReport), RemeshError> {
    remesh(mesh, DEFAULT_RESOLUTION)
}

#[cfg(test)]
#[path = "remesh_tests.rs"]
mod tests;
