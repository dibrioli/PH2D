//! **O DOCUMENTO da escultura** — a cena como bytes, e os bytes como cena.
//!
//! Filho (`#[path]`) de [`super`] para alcançar `objects`/`active`/`next_id` e as
//! filas de desfazer; o corte é *o que a cena É* (lá) contra *o que dela
//! SOBREVIVE a fechar o app* (aqui).
//!
//! ## Onde ele viaja, e por que não é dentro do `ProjectState`
//!
//! O blob mora em `ProjectFile.sculpt`, **fora** do `ProjectState` — que é a
//! unidade do undo GLOBAL. A escultura tem fila própria (`Entry`/`StrokeUndo`),
//! e enfiá-la ali faria cada Ctrl+Z do canvas rebobinar uma pincelada de barro,
//! e vice-versa. É o mesmo lugar, pelo mesmo motivo, de `motion`, `timeline` e
//! `physics`.
//!
//! ## Ele carrega a própria versão
//!
//! [`SCULPT_DOC_VERSION`] mora **dentro** do blob, então o módulo pode evoluir
//! muitas waves sem tocar o `PROJECT_SCHEMA` — o precedente exato do
//! `TimelineDoc` ([[docs/3D/02.3]] previu isto por escrito). O `PROJECT_SCHEMA`
//! bumpa **uma vez**, quando o campo nasce, e é isso.
//!
//! ## Uma escultura ilegível RECUSA o load inteiro
//!
//! A mesma lei da timeline, e a razão é a mesma: abrir *sem* ela mostraria uma
//! cena que parece certa, com a escultura vazia — e o **próximo Ctrl+S gravaria
//! esse vazio por cima do arquivo**. A obra não sumiria por um bug; sumiria
//! porque o app abriu, mentiu e salvou. O parse acontece **antes** de qualquer
//! mutação da sessão, então recusar não custa nada ao documento aberto.

use ph2d_mesh::{DocError, Multires, Pose, PoseData, StackData};
use serde::{Deserialize, Serialize};

use super::{SceneObject, Sculpt3dScene};

/// A versão do documento de escultura.
///
/// ⚠️ **Bumpe-a quando qualquer tipo dentro do blob mudar de forma** — inclusive
/// os da `ph2d-mesh` (`StackData`/`MeshData`/`DetailData`), que este arquivo só
/// referencia. O postcard é POSICIONAL: um campo novo lido por um binário velho
/// não falha, devolve lixo bem-formado. O gate `the_shape_of_a_saved_scene_is_pinned`
/// prende o tamanho codificado de uma cena-fixture justamente para transformar
/// "lembre-se" em vermelho.
pub(crate) const SCULPT_DOC_VERSION: u32 = 1;

/// Uma peça, como o arquivo a guarda.
#[derive(Serialize, Deserialize)]
struct ObjectDoc {
    /// A PILHA inteira, não a malha viva — os níveis abaixo são trabalho
    /// autorado, e um documento que guardasse só o nível de cima faria o
    /// artista perder a multiresolução ao reabrir.
    stack: StackData,
    pose: PoseData,
}

/// A cena, como o arquivo a guarda.
///
/// ⚠️ **O `ObjectId` NÃO viaja, e é decisão.** Ele existe para que uma entrada
/// de desfazer nomeie a peça certa *dentro de uma sessão*, e a fila de desfazer
/// não atravessa um load (ver [`Sculpt3dScene::install_doc`]). Gravá-lo seria
/// guardar a chave de uma tabela que não existe do outro lado.
#[derive(Serialize, Deserialize)]
struct SculptDoc {
    version: u32,
    objects: Vec<ObjectDoc>,
    /// Qual peça estava em mãos. Clampado na leitura — um índice fora de
    /// alcance é entrada de terceiro, não um estado que se possa confiar.
    active: u32,
}

/// Por que um documento de escultura foi recusado.
#[derive(Debug)]
pub(crate) enum SculptDocError {
    /// Os bytes não são um documento (truncado, de outro formato).
    Bytes(postcard::Error),
    /// O documento é de outra versão do módulo.
    Version { found: u32, expected: u32 },
    /// A geometria de dentro não valida — ver [`DocError`].
    Content(DocError),
}

impl core::fmt::Display for SculptDocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bytes(e) => write!(f, "bytes ilegiveis: {e}"),
            Self::Version { found, expected } => {
                write!(f, "documento v{found}, este binario le v{expected}")
            }
            Self::Content(e) => write!(f, "{e}"),
        }
    }
}

/// Uma peça já reconstruída — o que o load entrega e o device consome.
pub(crate) type LoadedPiece = (Multires, Pose);

/// **Lê um documento**, derivando de novo tudo o que é derivável.
///
/// ⚠️ **É PURA e não toca a cena**, e essa é a metade que importa: ela roda no
/// load antes de qualquer mutação (a recusa não pode custar o documento aberto)
/// e é dirigível **sem janela**, então o gate que a exercita é o mesmo caminho
/// que o Ctrl+O executa.
///
/// # Errors
/// Bytes ilegíveis, versão de outro módulo, ou geometria que não valida.
pub(crate) fn decode(bytes: &[u8]) -> Result<(Vec<LoadedPiece>, usize), SculptDocError> {
    let doc: SculptDoc = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
    if doc.version != SCULPT_DOC_VERSION {
        return Err(SculptDocError::Version {
            found: doc.version,
            expected: SCULPT_DOC_VERSION,
        });
    }
    let mut pieces = Vec::with_capacity(doc.objects.len());
    for o in doc.objects {
        let stack = Multires::from_data(o.stack).map_err(SculptDocError::Content)?;
        pieces.push((stack, Pose::from_data(o.pose)));
    }
    // Clamp e não erro: a lista pode estar vazia (documento de projeto sem
    // escultura), e "quem estava em mãos" é conforto de sessão — recusar o
    // arquivo inteiro por causa dele seria desproporcional.
    let active = (doc.active as usize).min(pieces.len().saturating_sub(1));
    Ok((pieces, active))
}

/// **Escreve um documento** — a metade PURA de [`Sculpt3dScene::to_doc_bytes`].
///
/// ⚠️ Ela existe separada por uma razão só, e é a que decide o gate: uma
/// `Sculpt3dScene` **não nasce sem um `wgpu::Device`**, então um round-trip
/// escrita→leitura preso ao método seria um teste de GPU — `#[ignore]`, fora da
/// varredura normal, sobre a única propriedade que o artista sente (*o que eu
/// salvei é o que eu abro*). Com a porta aqui, o par `encode`/[`decode`] é
/// dirigível **sem janela**, e o método fica sendo o que ele de fato é: a
/// coleta. O arch-gate `the_writer_goes_through_the_one_encoder` impede que
/// ele volte a montar o `SculptDoc` por conta própria.
pub(crate) fn encode(pieces: &[(StackData, PoseData)], active: usize) -> Vec<u8> {
    let doc = SculptDoc {
        version: SCULPT_DOC_VERSION,
        objects: pieces
            .iter()
            .map(|(stack, pose)| ObjectDoc {
                stack: stack.clone(),
                pose: *pose,
            })
            .collect(),
        active: active as u32,
    };
    postcard::to_allocvec(&doc).unwrap_or_else(|e| {
        // Um documento que não serializa é bug nosso, não entrada do artista —
        // mas emitir bytes pela metade seria gravar um arquivo que não abre.
        // Vazio + a razão no log é a única saída honesta.
        eprintln!("[sculpt3d] documento nao serializou, projeto salvo SEM a escultura: {e}");
        Vec::new()
    })
}

impl Sculpt3dScene {
    /// **Escreve o documento** desta cena.
    #[must_use]
    pub(crate) fn to_doc_bytes(&self) -> Vec<u8> {
        let pieces: Vec<(StackData, PoseData)> = self
            .objects
            .iter()
            .map(|o| (o.stack.to_data(), o.pose.to_data()))
            .collect();
        encode(&pieces, self.active)
    }

    /// **Instala um documento lido** — a cena passa a ser a do arquivo.
    ///
    /// ⚠️ **A FILA DE DESFAZER MORRE AQUI**, e não é higiene: toda entrada nomeia
    /// uma peça por [`ObjectId`], e as peças do arquivo são outras — desfazer
    /// através de um load aplicaria o inverso de um traço a barro que nunca o
    /// recebeu. A mesma lei que o load de projeto já aplica ao undo global.
    ///
    /// ⚠️ **Os ids são cunhados NOVOS**, em ordem, e o `next_id` segue de onde
    /// eles pararam — um id reciclado é exatamente o defeito que o `ObjectId`
    /// existe para não ter.
    ///
    /// Uma lista vazia é **recusada em silêncio** (devolve `false`): a cena
    /// nunca-vazia é o invariante que torna `obj()` total, e um projeto sem
    /// escultura simplesmente não chama isto.
    pub(crate) fn install_doc(
        &mut self,
        pieces: Vec<LoadedPiece>,
        active: usize,
        aspect: f32,
    ) -> bool {
        if pieces.is_empty() {
            return false;
        }
        self.objects.clear();
        self.next_id = 0;
        for (stack, pose) in pieces {
            let id = self.mint_id();
            self.objects.push(SceneObject::from_stack(id, stack, pose));
        }
        self.active = active.min(self.objects.len() - 1);
        self.forget_history();
        // O device ainda tem a cena ANTERIOR: o `sync_mesh` do frame sobe o que
        // `uploaded == false` pedir e o `truncate_objects` corta o excedente —
        // e é por isso que instalar não precisa de `&Device`.
        self.mesh_rebuilt();
        self.frame_all(aspect);
        true
    }

    /// Esquece o que só fazia sentido no documento anterior.
    fn forget_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

impl crate::app_state::App {
    /// **Instala a escultura que um load deixou pendente** — a porta que espera o
    /// device.
    ///
    /// ⚠️ **Ela existe porque o load é dirigível SEM janela** (o `App` nasce com
    /// `gfx` em `None`, e o winit só cria a janela no `resumed`), então o load não
    /// pode construir a cena: ele decodifica e deixa aqui. Roda no frame, ao lado
    /// do irmão que arma o smoke, e é no-op sem pendência.
    pub(crate) fn sculpt3d_install_pending(&mut self) {
        if self.sculpt3d_pending.is_none() || self.gfx.is_none() {
            return;
        }
        let Some((pieces, active)) = self.sculpt3d_pending.take() else {
            return;
        };
        if pieces.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        if let Some(scene) = gfx.sculpt3d.as_mut() {
            scene.install_doc(pieces, active, aspect);
            return;
        }
        // ⚠️ **Um projeto com escultura ARMA o módulo**, mesmo sem a env var do
        // smoke — a alternativa seria abrir o arquivo, descartar a obra em
        // silêncio e gravá-la fora no save seguinte.
        //
        // ⚠️ E a malha do `new` é uma CÓPIA que o `install_doc` joga fora logo
        // abaixo. Ela fica assim de propósito: um segundo construtor seria a
        // segunda resposta a *"como uma cena nasce"*, e o preço desta é
        // estritamente menor que o trabalho que o load já fez — reconstruir
        // octree e adjacência de **todo nível de toda peça**.
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        let first = pieces[0].0.mesh().clone();
        let mut scene = Sculpt3dScene::new(&device, first, aspect);
        scene.install_doc(pieces, active, aspect);
        gfx.sculpt3d = Some(scene);
    }
}

#[cfg(test)]
#[path = "sculpt3d_doc_tests.rs"]
mod tests;
