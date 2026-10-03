//! **OS FORMATOS ANTERIORES, CONGELADOS, E A CADEIA DE MIGRAÇÃO** até ao v5 — filho (`#[path]`) de
//! [`super`] (`doc.rs`), cortado dele pelo tecto de LOC do ficheiro: lá *o formato de hoje e quem o
//! lê*, aqui *o que um ficheiro antigo trazia e como chega à forma v5*, que o leitor do v6 migra.

use super::*;

/// A versão que ganhou o plano de tinta fina — e a primeira que este módulo
/// teve de MIGRAR. Ver [`decode`].
pub(super) const V_ANTES_DA_TINTA: u32 = 1;

/// A versão em que o plano tinha **um nível só** para a peça inteira, antes de
/// a graduação por área (a P2) chegar ao artista. Ver [`decode`].
pub(super) const V_ANTES_DA_GRADUACAO: u32 = 2;

/// A versão em que o plano não tinha RELEVO — antes do impasto do Painter na
/// peça (`docs/3D/29`). Ver [`decode`].
pub(super) const V_ANTES_DO_RELEVO: u32 = 3;

/// A versão em que o relevo era só a ALTURA — antes do CORPO (`docs/3D/29` §6,
/// o anel na borda do traço). Ver [`decode`].
pub(super) const V_ANTES_DO_CORPO: u32 = 4;

/// A versão em que o plano era UMA imagem de cor `f32` — antes das camadas do
/// Painter na peça (`docs/3D/30`). Ver [`decode`].
pub(super) const V_ANTES_DAS_CAMADAS: u32 = 5;

/// ⭐⭐⭐⭐ **O PLANO DE TINTA FINA de um documento v5** — congelado, e lido só
/// pela migração.
///
/// ⭐ **Só o NÍVEL e as AMOSTRAS.** A [`ph2d_mesh_colors::Topologia`] é
/// **derivada** das faces da malha que viaja ao lado — guardá-la seria guardar
/// uma resposta que a malha já dá, e é a mesma lei que faz esta porta re-derivar
/// normais, adjacência e octree em vez de as gravar.
///
/// ⚠️ **As amostras vão em CORRIDAS** e o porquê tem números: ver o cabeçalho
/// do [`doc_tinta`]. Em resumo — o `8x` da peça de fábrica são `75,5 MB` crus e
/// `~0` quando o plano ainda não foi pintado.
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub(super) struct TintaDocV5 {
    pub(super) nivel: u8,
    pub(super) amostras: doc_tinta::AmostrasDoc,
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
    pub(super) niveis: Vec<u8>,
    /// ⭐ **O RELEVO do plano, `[altura, corpo]`** (o impasto do Painter na
    /// peça, `docs/3D/29`), nas mesmas corridas da cor. `None` = o plano nunca
    /// levou impasto, e custa UM byte.
    pub(super) relevo: Option<doc_tinta::RelevoDoc>,
}

/// A peça de um documento **v5** — congelada, e lida só pela migração.
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub(super) struct ObjectDocV5 {
    pub(super) stack: StackData,
    pub(super) pose: PoseData,
    pub(super) tinta: Option<TintaDocV5>,
}

/// Um documento **v5** — congelado, e lido só pela migração (e escrito pelos
/// gates dela).
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub(super) struct SculptDocV5 {
    #[allow(dead_code)]
    pub(super) version: u32,
    pub(super) objects: Vec<ObjectDocV5>,
    pub(super) active: u32,
}

/// O plano de tinta de um documento **v4** — congelado, e lido só pela
/// migração: o relevo era só a ALTURA (ver [`doc_tinta::relevo_de_alturas`]).
#[derive(Deserialize)]
pub(super) struct TintaDocV4 {
    nivel: u8,
    amostras: doc_tinta::AmostrasDoc,
    niveis: Vec<u8>,
    alturas: Option<doc_tinta::AlturasDoc>,
}

/// A peça de um documento **v4** — congelada, e lida só pela migração.
#[derive(Deserialize)]
pub(super) struct ObjectDocV4 {
    stack: StackData,
    pose: PoseData,
    tinta: Option<TintaDocV4>,
}

/// Um documento **v4** — congelado, e lido só pela migração.
#[derive(Deserialize)]
pub(super) struct SculptDocV4 {
    #[allow(dead_code)]
    version: u32,
    objects: Vec<ObjectDocV4>,
    active: u32,
}

/// O plano de tinta de um documento **v3** — congelado, e lido só pela
/// migração (o campo `alturas` do v4 não está lá: ver [`TintaDocV2`]).
#[derive(Deserialize)]
pub(super) struct TintaDocV3 {
    nivel: u8,
    amostras: doc_tinta::AmostrasDoc,
    niveis: Vec<u8>,
}

/// A peça de um documento **v3** — congelada, e lida só pela migração.
#[derive(Deserialize)]
pub(super) struct ObjectDocV3 {
    stack: StackData,
    pose: PoseData,
    tinta: Option<TintaDocV3>,
}

/// Um documento **v3** — congelado, e lido só pela migração.
#[derive(Deserialize)]
pub(super) struct SculptDocV3 {
    #[allow(dead_code)]
    version: u32,
    objects: Vec<ObjectDocV3>,
    active: u32,
}

/// O plano de tinta de um documento **v2** — congelado, e lido só pela migração.
///
/// ⛔ Ele existe pela MESMA razão do [`ObjectDocV1`]: o campo `niveis` que o v3
/// acrescentou não está lá, e ler os bytes de um v2 com a forma do v3 **não
/// falha** — devolve lixo bem-formado.
#[derive(Deserialize)]
pub(super) struct TintaDocV2 {
    nivel: u8,
    amostras: doc_tinta::AmostrasDoc,
}

/// A peça de um documento **v2** — congelada, e lida só pela migração.
#[derive(Deserialize)]
pub(super) struct ObjectDocV2 {
    stack: StackData,
    pose: PoseData,
    tinta: Option<TintaDocV2>,
}

/// Um documento **v2** — congelado, e lido só pela migração.
#[derive(Deserialize)]
pub(super) struct SculptDocV2 {
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
pub(super) struct ObjectDocV1 {
    stack: StackData,
    pose: PoseData,
}

/// A cena de um documento **v1** — ver [`ObjectDocV1`].
#[derive(Deserialize)]
pub(super) struct SculptDocV1 {
    /// ⚠️ **Lido pelo POSTCARD e por mais ninguém.** Ele é posicional: sem este
    /// campo a leitura sai deslocada por um varint e devolve lixo bem-formado.
    /// *Chamar-lhe `_version` esconderia que ele é obrigatório*, e apagá-lo
    /// parte a migração em silêncio.
    #[allow(dead_code)]
    version: u32,
    objects: Vec<ObjectDocV1>,
    active: u32,
}

/// **Um documento ANTERIOR às camadas, lido até à forma v5** — a cadeia de
/// migrações de cada degrau. ⛔ Uma versão que nenhum degrau conhece é recusa.
pub(super) fn ate_v5(versao: u32, bytes: &[u8]) -> Result<SculptDocV5, SculptDocError> {
    Ok(match versao {
        V_ANTES_DAS_CAMADAS => postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?,
        // ⭐ **A MIGRAÇÃO do corpo.** Um relevo gravado antes do corpo era só a
        // altura, e a luz inclinava pela altura inteira ⇒ corpo `1` onde havia
        // espessura descreve-o exactamente.
        V_ANTES_DO_CORPO => {
            let v4: SculptDocV4 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDocV5 {
                version: V_ANTES_DAS_CAMADAS,
                objects: v4
                    .objects
                    .into_iter()
                    .map(|o| ObjectDocV5 {
                        stack: o.stack,
                        pose: o.pose,
                        tinta: o.tinta.map(|t| TintaDocV5 {
                            nivel: t.nivel,
                            amostras: t.amostras,
                            niveis: t.niveis,
                            relevo: t.alturas.map(doc_tinta::relevo_de_alturas),
                        }),
                    })
                    .collect(),
                active: v4.active,
            }
        }
        // ⭐ **A MIGRAÇÃO do relevo.** Um plano gravado antes do impasto na
        // peça não tinha relevo ⇒ `None` descreve-o exactamente.
        V_ANTES_DO_RELEVO => {
            let v3: SculptDocV3 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDocV5 {
                version: V_ANTES_DAS_CAMADAS,
                objects: v3
                    .objects
                    .into_iter()
                    .map(|o| ObjectDocV5 {
                        stack: o.stack,
                        pose: o.pose,
                        tinta: o.tinta.map(|t| TintaDocV5 {
                            nivel: t.nivel,
                            amostras: t.amostras,
                            niveis: t.niveis,
                            relevo: None,
                        }),
                    })
                    .collect(),
                active: v3.active,
            }
        }
        // ⭐⭐ **A MIGRAÇÃO.** Um documento gravado antes de a tinta fina viajar
        // abre, e as peças vêm sem plano — que é exactamente o que elas tinham.
        // ⭐⭐ **A MIGRAÇÃO da graduação.** Um plano gravado antes da P2 tinha
        // um nível só para a peça inteira ⇒ a lista vazia descreve-o
        // exactamente, e o load não muda um bit da tinta.
        V_ANTES_DA_GRADUACAO => {
            let v2: SculptDocV2 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDocV5 {
                version: V_ANTES_DAS_CAMADAS,
                objects: v2
                    .objects
                    .into_iter()
                    .map(|o| ObjectDocV5 {
                        stack: o.stack,
                        pose: o.pose,
                        tinta: o.tinta.map(|t| TintaDocV5 {
                            nivel: t.nivel,
                            amostras: t.amostras,
                            niveis: Vec::new(),
                            relevo: None,
                        }),
                    })
                    .collect(),
                active: v2.active,
            }
        }
        V_ANTES_DA_TINTA => {
            let v1: SculptDocV1 = postcard::from_bytes(bytes).map_err(SculptDocError::Bytes)?;
            SculptDocV5 {
                version: V_ANTES_DAS_CAMADAS,
                objects: v1
                    .objects
                    .into_iter()
                    .map(|o| ObjectDocV5 {
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
    })
}
