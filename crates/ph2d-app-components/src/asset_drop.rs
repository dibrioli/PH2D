//! ⭐⭐⭐ **O QUE ACONTECE AO LARGAR** — a lei da queda (plano `docs/Components/07`, etapa B).
//!
//! A queda decide pelo TIPO DO ALVO: o canvas cria, um campo do Inspector (ranhura, objecto, som)
//! troca o valor dele.
//!
//! # A lei, em três linhas
//!
//! | largo… | …em cima de | e acontece |
//! |---|---|---|
//! | um **prefab** | qualquer sítio do canvas | nasce uma cópia **ali** |
//! | uma **imagem** | qualquer sítio do canvas | nasce uma sprite nova **ali** |
//! | uma **imagem** | a ranhura Image do Inspector | aquela sprite passa a mostrá-la |
//!
//! ⛔ Uma imagem largada SOBRE uma sprite no canvas não a retextura (dono 26/10, smoke GEF 13): o céu cobre
//! a vista inteira e engolia toda a queda. Como a Unity: o canvas cria, o Inspector troca.
//!
//! ⛔ E **fora do canvas, RECUSA** — visível, nunca silêncio. Largar num sítio que não sabe receber
//! tem de se ver, senão o artista conclui que funcionou.
//!
//! ⚠️ **Este módulo é PURO.** Ele não instancia, não carrega pixels, não toca no mundo: recebe o
//! que se arrasta e o que está debaixo do cursor, e devolve **o que fazer**. É isso que o torna
//! gateável sem GPU — e é onde o defeito de *«o alvo adivinha»* moraria.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::drag_payload::{DragPayload, DragVerdict, DropFieldMark};
use ph2d_i18n::tr;

/// Onde o cursor estava quando a mão largou.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DropTarget {
    /// Sobre o canvas, no ponto de MUNDO `world`.
    ///
    /// `over` é a entidade debaixo do cursor, já RESOLVIDA pela porta de pick do shell (nunca um
    /// segundo pick aqui).
    Canvas {
        world: [f32; 2],
        over: Option<DropOver>,
    },
    /// ⭐⭐ **Sobre uma LINHA DE CATÁLOGO** da coluna do navegador (wave A3).
    ///
    /// Alvo próprio (a queda decide pelo TIPO DO ALVO), sem bandas: aqui há uma resposta só, *põe
    /// isto nesta gaveta*.
    CatalogRow {
        /// `None` = a linha *Unassigned* (tirar de qualquer catálogo).
        catalog: Option<u128>,
    },
    /// ⭐⭐⭐ **Sobre a RANHURA DA TEXTURA do Inspector** (plano `docs/Components/07`, wave B3).
    ///
    /// ⚠️ **Alvo próprio, e não uma excepção dentro do [`DropTarget::Chrome`]** — é o que o
    /// comentário do `Chrome` prometia desde a wave A3 (*«quando um campo souber receber, ele
    /// entra aqui como alvo próprio»*), e é a lei D7 do plano: *a queda decide pelo TIPO DO ALVO*.
    ///
    /// `entity_bits` é a sprite que a ranhura descreve — resolvida pelo painel que a pintou.
    InspectorTexture { entity_bits: u64 },
    /// ⭐ P14 — a ranhura **Normal Map** do cartão Image: a normal da imagem que a sprite `entity_bits` mostra.
    InspectorNormalMap { entity_bits: u64 },
    /// ⭐ O campo de uma propriedade de OBJECTO no cartão Script (8c do CodeEditor): o objecto do
    /// cartão e a linha.
    ScriptObject { entity_bits: u64, row: u16 },
    /// ⭐ GEF 20 C2 / GEF 06 — um campo que guarda um OBJECTO pelo nome (o Target do Follow da câmera, o
    /// Target do Homing da bala) no objecto `entity_bits`.
    ObjectRef { entity_bits: u64, field: RefField },
    /// ⭐ GEF 21 E2 — o campo Sound do Audio Source 2D de `entity_bits`.
    InspectorSound { entity_bits: u64 },
    /// ⭐⭐ **Sobre uma aba de SCRIPT** (item 8 do CodeEditor, `docs/CodeEditor/07_arrastar_para_o_codigo.md`).
    /// `on_text` = o cursor está sobre o TEXTO (a aba mostra onde cairia); na calha ou na barra, não.
    Code { on_text: bool },
    /// ⭐ D17 — sobre o canvas de um QUADRO aberto, no ponto de ECRÃ `at` (a câmara do quadro
    /// converte-o; é o mesmo ponto que a queda de imagens do sistema usa).
    BoardCanvas { at: [f32; 2] },
    /// Sobre o chrome (um painel, a barra, o rail) — nada aqui sabe receber um asset.
    Chrome,
    /// ⭐ **De volta ao painel de onde saiu** — isto é um CANCELAR, não uma recusa.
    ///
    /// ⚠️ **A diferença não é cosmética.** Arrastar para fora e voltar é o gesto universal de
    /// desistir, e ele é *silencioso* em todo o software que o tem. Tratá-lo como recusa daria um
    /// aviso a quem fez exactamente a coisa certa — e um aviso que aparece quando não há nada
    /// errado ensina o artista a ignorar os avisos.
    Source,
}

/// O que o cursor encontrou no canvas: só a receita de MACRO o usa (aplica-se por cima dele).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DropOver {
    pub entity_bits: u64,
}

/// O que fazer. ⛔ Uma variante por EFEITO, não por gesto.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DropAction {
    /// Instanciar o prefab `stable_id` com a raiz em `world`.
    /// `over` = o objecto debaixo da mão: uma receita de MACRO aplica-se por cima dele (W18); qualquer
    /// outro prefab ignora-o.
    PlacePrefab {
        stable_id: u64,
        world: [f32; 2],
        over: Option<u64>,
    },
    /// A sprite `entity_bits` passa a mostrar `asset`.
    RetextureSprite { entity_bits: u64, asset: [u8; 32] },
    /// ⭐ P14 — a imagem que a sprite `entity_bits` mostra passa a ter a normal `asset` (o MESMO pedido que o
    /// seletor: `ImageUseEdit::SetNormalMap`).
    SetNormalMap { entity_bits: u64, asset: [u8; 32] },
    /// Nasce uma sprite nova em `world`, mostrando `asset`.
    SpawnImage { asset: [u8; 32], world: [f32; 2] },
    /// ⭐⭐ **Arrumar** — o asset passa a pertencer a `catalog` (`None` = a nenhum).
    Filecatalog {
        /// Quem foi largado.
        payload: DragPayload,
        /// A gaveta.
        catalog: Option<u128>,
    },
    /// ⭐⭐ A aba de script escreve o código que aponta os objectos largados (item 8 do CodeEditor).
    WriteCode,
    /// ⭐ O objecto `stable_id` passa a ser o valor da propriedade de objecto na linha `row` do
    /// script de `entity_bits` (8c do CodeEditor).
    SetScriptObject {
        entity_bits: u64,
        row: u16,
        stable_id: u64,
    },
    /// ⭐ GEF 20 C2 / GEF 06 — o campo `field` de `entity_bits` passa a apontar o objecto `stable_id` (pelo NOME,
    /// a referência durável).
    SetObjectRef {
        entity_bits: u64,
        field: RefField,
        stable_id: u64,
    },
    /// ⭐ GEF 21 E2 — a fonte `entity_bits` passa a tocar o som do projeto `asset` (pelo NOME, como o campo).
    SetSound { entity_bits: u64, asset: [u8; 32] },
    /// ⭐ D17 — uma referência NOVA ao Kanban `kanban` no quadro aberto, com o meio no ponto de ecrã
    /// `at`, seleccionada (UM passo de desfazer, que tira só a referência).
    PlaceKanban { kanban: u64, at: [f32; 2] },
    /// ⭐ D17 — a imagem `asset` entra no quadro aberto com o meio no ponto de ecrã `at`, como uma imagem
    /// largada do sistema (`ph2d_app_board::images::drop_asset`), seleccionada, UM passo de desfazer.
    PlaceImage { asset: [u8; 32], at: [f32; 2] },
    /// ⛔ **Nada acontece, e VÊ-SE.** Ver o doc do módulo.
    Refuse,
    /// ⭐ **Nada acontece, e é SILENCIOSO** — o artista desistiu. Ver [`DropTarget::Source`].
    Cancel,
}

/// Qual campo de objecto recebe a queda ([`DropTarget::ObjectRef`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefField {
    /// O Target do Follow da câmera.
    CameraFollow,
    /// O Target do Homing do Projectile Motion.
    Homing,
}

impl DropAction {
    /// A frase que o artista lê quando a queda é recusada.
    ///
    /// ⚠️ Ela nomeia **o que ele largou**, e não *«alvo inválido»*: a mensagem tem de o ajudar a
    /// perceber onde aquilo servia.
    #[must_use]
    pub fn refusal_line(payload: DragPayload) -> String {
        match payload {
            DragPayload::Prefab { .. } => tr("shell.asset_drop.drop_a_prefab_on_the").to_string(),
            DragPayload::Image { .. } => tr("shell.asset_drop.drop_an_image_on_a").to_string(),
            DragPayload::Object { .. } => {
                tr("shell.asset_drop.drop_an_object_on_a_script").to_string()
            }
            // A lei nunca a recusa (fora do texto, desiste); a frase é a do objecto.
            DragPayload::Property { .. } => {
                tr("shell.asset_drop.drop_an_object_on_a_script").to_string()
            }
            DragPayload::Kanban { .. } => {
                tr("shell.asset_drop.drop_a_kanban_on_a_board").to_string()
            }
            DragPayload::Board { .. } => {
                tr("shell.asset_drop.a_board_does_not_go_on_a_board").to_string()
            }
            DragPayload::Sound { .. } => {
                tr("app.components.asset_drop.drop_a_sound_on_the_sound_field").to_string()
            }
        }
    }
}

/// ⭐ GEF 20b — o CAMPO do Inspector `id` como alvo de queda (ranhura de textura, objecto do Script,
/// alvo da câmera e da bala); `None` = o widget não é um campo que recebe. A porta ÚNICA: a queda, o fantasma
/// e as caixas marcadas perguntam aqui.
#[must_use]
pub fn field_target(id: NodeId) -> Option<DropTarget> {
    use ph2d_panel_inspector as insp;
    insp::texture_slot_pick(id)
        .map(|entity_bits| DropTarget::InspectorTexture { entity_bits })
        .or_else(|| {
            insp::normal_map_slot_pick(id)
                .map(|entity_bits| DropTarget::InspectorNormalMap { entity_bits })
        })
        .or_else(|| {
            insp::script_object_slot_pick(id)
                .map(|(entity_bits, row)| DropTarget::ScriptObject { entity_bits, row })
        })
        .or_else(|| {
            insp::camera_target_slot_pick(id).map(|entity_bits| DropTarget::ObjectRef {
                entity_bits,
                field: RefField::CameraFollow,
            })
        })
        .or_else(|| {
            insp::projectile_target_slot_pick(id).map(|entity_bits| DropTarget::ObjectRef {
                entity_bits,
                field: RefField::Homing,
            })
        })
        .or_else(|| {
            insp::audio_sound_slot_pick(id)
                .map(|entity_bits| DropTarget::InspectorSound { entity_bits })
        })
}

/// A edição do Inspector que grava `nome` no campo de objecto `field` de `entity_bits` (o mesmo passo de desfazer
/// que escrever o nome ou usar o conta-gotas).
#[must_use]
pub fn edicao_do_campo(entity_bits: u64, field: RefField, nome: String) -> EditorAction {
    match field {
        RefField::CameraFollow => EditorAction::InspectorCameraEdit {
            entity_bits,
            edit: ph2d_editor_core::CameraFieldEdit::Target(nome),
        },
        RefField::Homing => EditorAction::InspectorComponentEdit {
            entity_bits,
            edit: ph2d_editor_core::action_bus::ComponentEdit::Projectile(
                ph2d_editor_core::projectile_edits::ProjectileFieldEdit::HomingTarget(nome),
            ),
        },
    }
}

/// O que o arrasto MOSTRA para `target`: desistir é silencioso (ver [`DropTarget::Source`]).
#[must_use]
pub fn verdict(payload: DragPayload, target: DropTarget) -> DragVerdict {
    match resolve(payload, target) {
        DropAction::Refuse => DragVerdict::Refuse,
        DropAction::Cancel => DragVerdict::Unknown,
        _ => DragVerdict::Accept,
    }
}

/// ⭐ GEF 20b — as caixas que o arrasto marca (o idioma do Godot): dos widgets `ids` pintados neste
/// quadro, cada campo que ACEITA a carga, e o de debaixo do cursor (`under`) seja qual for a resposta.
#[must_use]
pub fn field_marks(
    ids: impl IntoIterator<Item = NodeId>,
    payload: DragPayload,
    under: Option<NodeId>,
) -> Vec<DropFieldMark> {
    ids.into_iter()
        .filter_map(|id| {
            let verdict = verdict(payload, field_target(id)?);
            let under = under == Some(id);
            (under || verdict == DragVerdict::Accept).then_some(DropFieldMark {
                id,
                verdict,
                under,
            })
        })
        .collect()
}

/// ⭐ **A LEI.** Pura e total.
///
/// ⚠️ **Exaustiva no que importa:** os braços casam a carga **por variante**, então um `DragPayload`
/// novo é erro de compilação aqui; os `_` são sobre o ALVO, onde a carga não muda a resposta.
#[must_use]
pub fn resolve(payload: DragPayload, target: DropTarget) -> DropAction {
    match (payload, target) {
        // ⭐ Desistir é silencioso — ver [`DropTarget::Source`].
        (_, DropTarget::Source) => DropAction::Cancel,

        // ⭐⭐ **Um objecto da cena só se larga num script** (item 8 do CodeEditor): sobre o texto,
        // ele escreve o código que o aponta; em qualquer outro sítio recusa — e vê-se, como as
        // cargas da biblioteca. Antes do braço da gaveta: arrumar é de assets.
        (DragPayload::Object { .. }, DropTarget::Code { on_text: true }) => DropAction::WriteCode,
        // ⭐ No campo de uma propriedade de objecto, o objecto passa a ser o valor dela (8c).
        (DragPayload::Object { stable_id }, DropTarget::ScriptObject { entity_bits, row }) => {
            DropAction::SetScriptObject {
                entity_bits,
                row,
                stable_id,
            }
        }
        (DragPayload::Object { stable_id }, DropTarget::ObjectRef { entity_bits, field }) => {
            DropAction::SetObjectRef {
                entity_bits,
                field,
                stable_id,
            }
        }
        (DragPayload::Object { .. }, _) => DropAction::Refuse,
        // ⭐ Uma linha do Inspector (8b) só sabe ir para o texto; em qualquer outro sítio, desiste
        // calada — como o Godot, onde soltar uma propriedade fora do script não faz nada.
        (DragPayload::Property { .. }, DropTarget::Code { on_text: true }) => DropAction::WriteCode,
        (DragPayload::Property { .. }, _) => DropAction::Cancel,
        // ⭐ D17 — um Kanban largado num Quadro aberto põe lá uma referência, no ponto.
        (DragPayload::Kanban { id }, DropTarget::BoardCanvas { at }) => {
            DropAction::PlaceKanban { kanban: id, at }
        }
        // ⛔ D17 — fora disso um Kanban ou um Quadro só se ARRUMA (gaveta); a cena, a ranhura, o texto
        // e um Quadro dentro de um Quadro (não há esse elemento) recusam.
        (
            p @ (DragPayload::Kanban { .. } | DragPayload::Board { .. }),
            DropTarget::CatalogRow { catalog },
        ) => DropAction::Filecatalog {
            payload: p,
            catalog,
        },
        (DragPayload::Kanban { .. } | DragPayload::Board { .. }, _) => DropAction::Refuse,
        // ⭐ GEF 21 E2 — um som só entra no campo Sound, e o campo só o aceita a ele. ⚠️ A gaveta ainda o
        // recusa: arrumá-lo pediria a família no `SavedAssignment` do `ProjectState` (outro degrau).
        (DragPayload::Sound { asset }, DropTarget::InspectorSound { entity_bits }) => {
            DropAction::SetSound { entity_bits, asset }
        }
        // No texto de um script escreve o NOME (o que `ph2d.set(e, "sound", …)` recebe), como a imagem.
        (DragPayload::Sound { .. }, DropTarget::Code { on_text: true }) => DropAction::WriteCode,
        (DragPayload::Sound { .. }, _) | (_, DropTarget::InspectorSound { .. }) => {
            DropAction::Refuse
        }
        // ⭐ D17 — e uma imagem dos Assets entra nele no ponto, como a do sistema.
        (DragPayload::Image { asset }, DropTarget::BoardCanvas { at }) => {
            DropAction::PlaceImage { asset, at }
        }
        // ⛔ Um prefab não tem elemento no quadro.
        (_, DropTarget::BoardCanvas { .. }) => DropAction::Refuse,
        // ⛔ Um asset não é um objecto da cena: o campo de objecto recusa-o.
        (_, DropTarget::ScriptObject { .. } | DropTarget::ObjectRef { .. }) => DropAction::Refuse,
        // ⭐ Um mestre ou uma imagem no texto escreve o NOME (o 8b: `ph2d.spawn`, `sprite.image`);
        // fora do texto, nada.
        (_, DropTarget::Code { on_text: true }) => DropAction::WriteCode,
        (_, DropTarget::Code { .. }) => DropAction::Refuse,

        // ⭐⭐ **Uma linha de catálogo aceita as DUAS famílias** — arrumar não olha ao tipo, ao
        // contrário de instanciar. É a resposta que a lei D7 dá aqui: o ALVO decide, e este alvo
        // sabe receber qualquer asset.
        (payload, DropTarget::CatalogRow { catalog }) => {
            DropAction::Filecatalog { payload, catalog }
        }

        // ⭐⭐⭐ **A RANHURA recebe uma IMAGEM** (wave B3): é o único gesto de arrastar que troca a imagem de
        // uma sprite que já existe.
        (DragPayload::Image { asset }, DropTarget::InspectorTexture { entity_bits }) => {
            DropAction::RetextureSprite { entity_bits, asset }
        }

        // ⛔ **Um prefab na ranhura RECUSA** — e a recusa vê-se. Uma receita não é uma imagem, e
        // «pôr um componente dentro do campo de textura» não é um gesto que exista.
        (DragPayload::Prefab { .. }, DropTarget::InspectorTexture { .. }) => DropAction::Refuse,

        // ⭐ P14 — a ranhura Normal Map recebe uma IMAGEM (a normal da imagem da sprite); um prefab recusa.
        (DragPayload::Image { asset }, DropTarget::InspectorNormalMap { entity_bits }) => {
            DropAction::SetNormalMap { entity_bits, asset }
        }
        (DragPayload::Prefab { .. }, DropTarget::InspectorNormalMap { .. }) => DropAction::Refuse,

        // ⛔ Fora do canvas, das linhas de catálogo e da ranhura, nada sabe receber.
        (_, DropTarget::Chrome) => DropAction::Refuse,

        (DragPayload::Prefab { stable_id }, DropTarget::Canvas { world, over }) => {
            // ⚠️ **Um prefab ignora o que está por baixo**, e é deliberado: largar sobre uma sprite
            // não pode significar *«substitui aquela sprite»* — isso é destruir trabalho com um
            // gesto de colocar. O objecto viaja só para a receita de MACRO, que toca POR CIMA dele (W18).
            DropAction::PlacePrefab {
                stable_id,
                world,
                over: over.map(|o| o.entity_bits),
            }
        }

        // ⭐ No canvas a imagem nasce SEMPRE como sprite nova no ponto, haja o que houver por baixo (ver o doc
        // do módulo).
        (DragPayload::Image { asset }, DropTarget::Canvas { world, .. }) => {
            DropAction::SpawnImage { asset, world }
        }
    }
}

#[cfg(test)]
#[path = "asset_drop_tests.rs"]
mod resolve_tests;

#[cfg(test)]
#[path = "asset_drop_field_tests.rs"]
mod field_tests;
