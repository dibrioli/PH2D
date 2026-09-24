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
use ph2d_mesh_colors::Tinta;
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
pub(crate) const SCULPT_DOC_VERSION: u32 = 3;

/// A versão que ganhou o plano de tinta fina — e a primeira que este módulo
/// teve de MIGRAR. Ver [`decode`].
const V_ANTES_DA_TINTA: u32 = 1;

/// A versão em que o plano tinha **um nível só** para a peça inteira, antes de
/// a graduação por área (a P2) chegar ao artista. Ver [`decode`].
const V_ANTES_DA_GRADUACAO: u32 = 2;

/// ⭐⭐⭐⭐ **O PLANO DE TINTA FINA de uma peça, como o arquivo o guarda.**
///
/// ⭐ **Só o NÍVEL e as AMOSTRAS.** A [`ph2d_mesh_colors::Topologia`] é
/// **derivada** das faces da malha que viaja ao lado — guardá-la seria guardar
/// uma resposta que a malha já dá, e é a mesma lei que faz esta porta re-derivar
/// normais, adjacência e octree em vez de as gravar.
///
/// ⚠️ **As amostras vão em CORRIDAS** e o porquê tem números: ver o cabeçalho
/// do [`doc_tinta`]. Em resumo — o `8x` da peça de fábrica são `75,5 MB` crus e
/// `~0` quando o plano ainda não foi pintado.
#[derive(Serialize, Deserialize)]
struct TintaDoc {
    nivel: u8,
    amostras: doc_tinta::AmostrasDoc,
    /// ⭐⭐⭐⭐ **O nível de CADA FACE — vazio quer dizer UNIFORME.**
    ///
    /// ⛔⛔ **A lista é gravada e não re-derivada, e a decisão tem número:**
    /// `1` byte por face (uns `18` KB numa peça do dono) contra um plano que a
    /// `8x` mede dezenas de MB. Re-derivar era a outra saída e ela tem um
    /// defeito que nenhuma régua vê — *uma mudança na lei da graduação
    /// relayouta um ficheiro já gravado em silêncio*, e as amostras estão
    /// guardadas POR ÍNDICE.
    ///
    /// ⚠️ **Vazio e não `Option`**: um plano uniforme é o caso comum e o
    /// postcard escreve um `Vec` vazio num byte.
    niveis: Vec<u8>,
}

/// Uma peça, como o arquivo a guarda.
#[derive(Serialize, Deserialize)]
struct ObjectDoc {
    /// A PILHA inteira, não a malha viva — os níveis abaixo são trabalho
    /// autorado, e um documento que guardasse só o nível de cima faria o
    /// artista perder a multiresolução ao reabrir.
    stack: StackData,
    pose: PoseData,
    /// `None` = a peça não tinha detalhe fino quando foi gravada.
    ///
    /// ⚠️ **Ele NÃO é redundante com a cor por vértice**, que viaja dentro do
    /// `stack`: aquela é a PROJECÇÃO deste plano nos vértices (o `devolve`
    /// escreve-a no fim de cada traço), e re-semear a partir dela devolve um
    /// plano exacto nos vértices e **interpolado no resto** — que é literalmente
    /// a tinta a voltar à resolução da malha.
    tinta: Option<TintaDoc>,
}

/// O plano de tinta de um documento **v2** — congelado, e lido só pela migração.
///
/// ⛔ Ele existe pela MESMA razão do [`ObjectDocV1`]: o campo `niveis` que o v3
/// acrescentou não está lá, e ler os bytes de um v2 com a forma do v3 **não
/// falha** — devolve lixo bem-formado.
#[derive(Deserialize)]
struct TintaDocV2 {
    nivel: u8,
    amostras: doc_tinta::AmostrasDoc,
}

/// A peça de um documento **v2** — congelada, e lida só pela migração.
#[derive(Deserialize)]
struct ObjectDocV2 {
    stack: StackData,
    pose: PoseData,
    tinta: Option<TintaDocV2>,
}

/// Um documento **v2** — congelado, e lido só pela migração.
#[derive(Deserialize)]
struct SculptDocV2 {
    #[allow(dead_code)]
    version: u32,
    objects: Vec<ObjectDocV2>,
    active: u32,
}

/// A peça de um documento **v1** — congelada, e lida só pela migração.
///
/// ⛔ **Ela existe porque o postcard é POSICIONAL:** o campo `tinta` que o v2
/// acrescentou não está lá, e ler os bytes de um v1 com a forma do v2 não falha
/// — *devolve lixo bem-formado*, que é a frase que o cabeçalho deste módulo já
/// escrevia sobre um binário velho a ler um ficheiro novo, agora do outro lado.
#[derive(Deserialize)]
struct ObjectDocV1 {
    stack: StackData,
    pose: PoseData,
}

/// A cena de um documento **v1** — ver [`ObjectDocV1`].
#[derive(Deserialize)]
struct SculptDocV1 {
    /// ⚠️ **Lido pelo POSTCARD e por mais ninguém.** Ele é posicional: sem este
    /// campo a leitura sai deslocada por um varint e devolve lixo bem-formado.
    /// *Chamar-lhe `_version` esconderia que ele é obrigatório*, e apagá-lo
    /// parte a migração em silêncio.
    #[allow(dead_code)]
    version: u32,
    objects: Vec<ObjectDocV1>,
    active: u32,
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
pub enum SculptDocError {
    /// Os bytes não são um documento (truncado, de outro formato).
    Bytes(postcard::Error),
    /// O documento é de outra versão do módulo.
    Version { found: u32, expected: u32 },
    /// A geometria de dentro não valida — ver [`DocError`].
    Content(DocError),
    /// ⛔⛔ **O plano de tinta fina da peça `peca` não descreve a malha dela.**
    ///
    /// A mesma lei da geometria, e pela mesma razão: abrir *sem* ele mostraria
    /// a peça com a tinta na resolução da MALHA — que é o que o artista vê
    /// quando perde o detalhe fino — e o **próximo Ctrl+S gravaria essa perda
    /// por cima**. Recusar em voz alta é a única saída honesta.
    Tinta { peca: usize, esperadas: usize },
}

impl core::fmt::Display for SculptDocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bytes(e) => write!(f, "bytes ilegiveis: {e}"),
            Self::Version { found, expected } => {
                write!(f, "documento v{found}, este binario le v{expected}")
            }
            Self::Content(e) => write!(f, "{e}"),
            Self::Tinta { peca, esperadas } => write!(
                f,
                "o plano de tinta fina da peca {peca} nao descreve a malha dela \
                 (ela pede {esperadas} amostras)"
            ),
        }
    }
}

/// Uma peça já reconstruída — o que o load entrega e o device consome.
#[derive(Debug)]
pub struct LoadedPiece {
    pub stack: Multires,
    pub pose: Pose,
    /// `None` = a peça não tinha detalhe fino quando foi gravada.
    pub tinta: Option<Tinta>,
}

/// **Lê um documento**, derivando de novo tudo o que é derivável.
///
/// ⚠️ **É PURA e não toca a cena**, e essa é a metade que importa: ela roda no
/// load antes de qualquer mutação (a recusa não pode custar o documento aberto)
/// e é dirigível **sem janela**, então o gate que a exercita é o mesmo caminho
/// que o Ctrl+O executa.
///
/// # Errors
/// Bytes ilegíveis, versão de outro módulo, ou geometria que não valida.
pub fn decode(bytes: &[u8]) -> Result<(Vec<LoadedPiece>, usize), SculptDocError> {
    // ⭐⭐⭐⭐ **A VERSÃO LÊ-SE PRIMEIRO, SOZINHA** — e não pelo `SculptDoc`
    // inteiro. Ela é o 1.º campo, logo `take_from_bytes::<u32>` lê exactamente
    // ela; ⛔ tentar a forma NOVA e cair para a velha no erro seria apostar que
    // um v1 falha a parsar como v2, e o postcard é POSICIONAL: *ele devolve
    // lixo bem-formado*.
    let (versao, _) = postcard::take_from_bytes::<u32>(bytes).map_err(SculptDocError::Bytes)?;
    let doc = match versao {
        SCULPT_DOC_VERSION => postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?,
        // ⭐⭐ **A MIGRAÇÃO.** Um documento gravado antes de a tinta fina viajar
        // abre, e as peças vêm sem plano — que é exactamente o que elas tinham.
        // ⭐⭐ **A MIGRAÇÃO da graduação.** Um plano gravado antes da P2 tinha
        // um nível só para a peça inteira ⇒ a lista vazia descreve-o
        // exactamente, e o load não muda um bit da tinta.
        V_ANTES_DA_GRADUACAO => {
            let v2: SculptDocV2 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDoc {
                version: SCULPT_DOC_VERSION,
                objects: v2
                    .objects
                    .into_iter()
                    .map(|o| ObjectDoc {
                        stack: o.stack,
                        pose: o.pose,
                        tinta: o.tinta.map(|t| TintaDoc {
                            nivel: t.nivel,
                            amostras: t.amostras,
                            niveis: Vec::new(),
                        }),
                    })
                    .collect(),
                active: v2.active,
            }
        }
        V_ANTES_DA_TINTA => {
            let v1: SculptDocV1 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDoc {
                version: SCULPT_DOC_VERSION,
                objects: v1
                    .objects
                    .into_iter()
                    .map(|o| ObjectDoc {
                        stack: o.stack,
                        pose: o.pose,
                        tinta: None,
                    })
                    .collect(),
                active: v1.active,
            }
        }
        found => {
            return Err(SculptDocError::Version {
                found,
                expected: SCULPT_DOC_VERSION,
            });
        }
    };
    let mut pieces = Vec::with_capacity(doc.objects.len());
    for (i, o) in doc.objects.into_iter().enumerate() {
        let stack = Multires::from_data(o.stack).map_err(SculptDocError::Content)?;
        // ⚠️ **O plano é montado DEPOIS da malha e a partir dela**: a topologia
        // é derivada das faces que acabaram de ser lidas, e é isso que faz o
        // documento não precisar de a guardar.
        let tinta = match o.tinta {
            None => None,
            Some(t) => Some(tinta_de(&stack, &t, i)?),
        };
        pieces.push(LoadedPiece {
            stack,
            pose: Pose::from_data(o.pose),
            tinta,
        });
    }
    // Clamp e não erro: a lista pode estar vazia (documento de projeto sem
    // escultura), e "quem estava em mãos" é conforto de sessão — recusar o
    // arquivo inteiro por causa dele seria desproporcional.
    let active = (doc.active as usize).min(pieces.len().saturating_sub(1));
    Ok((pieces, active))
}

/// **O plano de uma peça, reconstruído contra a malha que acabou de ser lida.**
fn tinta_de(stack: &Multires, doc: &TintaDoc, peca: usize) -> Result<Tinta, SculptDocError> {
    let mesh = stack.mesh();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    // ⭐⭐ **A lista vazia é o plano UNIFORME**, que é o que todo documento
    // anterior à P2 tem. ⛔ E uma lista que não descreve esta malha é RECUSA e
    // não um plano uniforme de consolação: *as amostras estão guardadas por
    // ÍNDICE, e um índice contra outra disposição é tinta no sítio errado*.
    let mut t = if doc.niveis.is_empty() {
        Tinta::nova(mesh.vert_count(), faces(), doc.nivel)
    } else {
        Tinta::graduada(mesh.vert_count(), faces(), &doc.niveis, doc.nivel).ok_or(
            SculptDocError::Tinta {
                peca,
                esperadas: doc.niveis.len(),
            },
        )?
    };
    let esperadas = t.amostras().len();
    let amostras = doc
        .amostras
        .amostras(esperadas)
        .ok_or(SculptDocError::Tinta { peca, esperadas })?;
    t.amostras_mut().copy_from_slice(&amostras);
    // ⭐⭐⭐⭐ **Um plano GRADUADO sai daqui UNIFORME** (2026-09-24, ordem do
    //   dono): o `Even Detail` que os criava foi retirado, e com ele o registo
    //   de `19` palavras que deixava a placa desenhá-los. A conversão LÊ cada
    //   amostra nova do plano gravado — é exacta onde a face estava no degrau
    //   pedido ou acima —, e ⛔ nunca re-semeia da cor por vértice, que era
    //   devolver a tinta à resolução da malha.
    //
    //   ⚠️ **Sem esta linha o ficheiro abria e a placa DESARMAVA** (a guarda
    //   do `tinta_cfg`): a tinta aparecia grossa, que é o report que o dono já
    //   fez três vezes por outras portas.
    if t.lado_uniforme().is_none() {
        t = t
            .uniformizada(faces())
            .ok_or(SculptDocError::Tinta { peca, esperadas })?;
    }
    Ok(t)
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
pub fn encode(pieces: &[(StackData, PoseData, Option<&Tinta>)], active: usize) -> Vec<u8> {
    let doc = SculptDoc {
        version: SCULPT_DOC_VERSION,
        objects: pieces
            .iter()
            .map(|(stack, pose, tinta)| ObjectDoc {
                stack: stack.clone(),
                pose: *pose,
                tinta: tinta.map(|t| TintaDoc {
                    nivel: t.nivel(),
                    amostras: doc_tinta::a_menor_forma(t.amostras()),
                    // ⭐ **Um plano UNIFORME grava a lista VAZIA**, e isso não é
                    //   uma optimização: é o que faz um documento sem graduação
                    //   sair byte a byte como saía antes da P2.
                    niveis: if t.lado_uniforme().is_some() {
                        Vec::new()
                    } else {
                        t.topologia().niveis().to_vec()
                    },
                }),
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
    pub fn to_doc_bytes(&self) -> Vec<u8> {
        // ⭐⭐⭐⭐ **O PLANO LÊ-SE DE ONDE ELE ESTÁ, e não do `Option` da peça**
        // ([`crate::tinta_da_peca::plano_de`]): durante um traço quem o segura
        // é o GESTO, e um `Ctrl+S` a meio de uma pincelada gravaria a peça
        // **sem o detalhe fino**. É o TERCEIRO consumidor daquela porta, e foi
        // este que a obrigou a existir.
        let pieces: Vec<(StackData, PoseData, Option<&Tinta>)> = (0..self.objects.len())
            .map(|i| {
                let o = &self.objects[i];
                (o.stack.to_data(), o.pose.to_data(), self.plano_de(i))
            })
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
        for peca in pieces {
            let id = self.mint_id();
            let mut obj = SceneObject::from_stack(id, peca.stack, peca.pose);
            // ⭐ E o plano volta com ela. `tinta_suja` fica a `true` porque o
            // device ainda tem a cena ANTERIOR — o `sync_mesh` do quadro é quem
            // o sobe, e sem esta marca ele só o faria no primeiro traço.
            obj.tinta = peca.tinta;
            obj.tinta_suja = true;
            self.objects.push(obj);
        }
        self.active = active.min(self.objects.len() - 1);
        // ⭐⭐⭐⭐ **E O DEGRAU VOLTA COM ELE** — ver
        // [`crate::tinta_da_peca::degrau_do_documento`]. Sem esta linha o plano
        // é instalado e o PRIMEIRO QUADRO deita-o fora, porque a fileira
        // `Paint Detail` nasce desarmada e a `garante` obedece-lhe: *o report
        // do dono de 21/09, «sobreviveu mas sem os detalhes 8x»*.
        self.tinta_nivel = crate::tinta_da_peca::degrau_do_documento(&self.objects, self.active);
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

/// **Instala a escultura que um load deixou pendente** — a porta que espera o
/// device.
///
/// ⚠️ **Ela existe porque o load é dirigível SEM janela** (o `App` nasce com
/// `gfx` em `None`, e o winit só cria a janela no `resumed`), então o load não
/// pode construir a cena: ele decodifica e deixa aqui. Roda no frame, ao lado
/// do irmão que arma o smoke, e é no-op sem pendência.
pub fn install_pending(
    slot: &mut Option<Sculpt3dScene>,
    pending: &mut Option<(Vec<crate::LoadedPiece>, usize)>,
    // ⚠️ **`None` = ainda não há janela, e a pendência tem de SOBREVIVER** — o `App` nasce com
    // `gfx` em `None` e o winit só cria a janela no `resumed`. É por isso que este `Option` é
    // perguntado ANTES do `take`: tomá-la sem device descartaria a obra em silêncio.
    gpu: Option<(&std::sync::Arc<wgpu::Device>, (u32, u32))>,
) {
    if pending.is_none() || gpu.is_none() {
        return;
    }
    let Some((pieces, active)) = pending.take() else {
        return;
    };
    if pieces.is_empty() {
        return;
    }
    let Some((device, size)) = gpu else {
        return;
    };
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    if let Some(scene) = slot.as_mut() {
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
    let first = pieces[0].stack.mesh().clone();
    let mut scene = Sculpt3dScene::new(device, first, aspect);
    scene.install_doc(pieces, active, aspect);
    *slot = Some(scene);
}

/// ⭐ **Como um plano de milhões de amostras cabe num ficheiro** — ver o
/// cabeçalho do irmão, que tem a medição.
#[path = "doc_tinta.rs"]
mod doc_tinta;

#[cfg(test)]
#[path = "doc_tests.rs"]
mod tests;
