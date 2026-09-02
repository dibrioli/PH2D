//! ⭐⭐⭐ **O QUE ACONTECE AO LARGAR** — a lei da queda (plano `docs/Components/07`, etapa B).
//!
//! ⛔⛔ **O plano supunha um «campo de textura do Inspector» e ele NÃO EXISTE.** Medido em
//! 2026-08-30: a secção *Render Source* mostra o armazenamento e a fonte como **texto** — sem
//! rectângulo de acerto, sem id, sem botão de escolher. O único *«escolhe uma imagem»* do app
//! inteiro é o do padrão vetorial, e é um diálogo de ficheiro.
//!
//! ⇒ **a superfície de queda é o CANVAS**, e ela é melhor: o artista larga a imagem **em cima do
//! objecto que ele vê**, e *«qual objecto recebe?»* passa a ser respondido pelo dedo em vez de por
//! uma lista. É a mesma razão por que o duplo-clique recusa uma imagem — aquele gesto não tem onde
//! apontar.
//!
//! # A lei, em três linhas
//!
//! | largo… | …em cima de | e acontece |
//! |---|---|---|
//! | um **prefab** | qualquer sítio do canvas | nasce uma cópia **ali** |
//! | uma **imagem** | uma **sprite** | aquela sprite passa a mostrá-la |
//! | uma **imagem** | canvas vazio | nasce uma sprite nova ali |
//!
//! ⛔ E **fora do canvas, RECUSA** — visível, nunca silêncio. Largar num sítio que não sabe receber
//! tem de se ver, senão o artista conclui que funcionou.
//!
//! ⚠️ **Este módulo é PURO.** Ele não instancia, não carrega pixels, não toca no mundo: recebe o
//! que se arrasta e o que está debaixo do cursor, e devolve **o que fazer**. É isso que o torna
//! gateável sem GPU — e é onde o defeito de *«o alvo adivinha»* moraria.

use ph2d_editor::interaction::drag_payload::DragPayload;

/// Onde o cursor estava quando a mão largou.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum DropTarget {
    /// Sobre o canvas, no ponto de MUNDO `world`.
    ///
    /// `over` é a entidade debaixo do cursor, se houver — já resolvida pela porta de pick do
    /// shell. ⚠️ **Ela chega RESOLVIDA e não como um ponteiro a picar aqui**: um segundo pick, com
    /// outras entradas, é exactamente a segunda resposta que este repo já pagou noutros sítios.
    Canvas {
        world: [f32; 2],
        over: Option<DropOver>,
    },
    /// ⭐⭐ **Sobre uma LINHA DE CATÁLOGO** da coluna do navegador (wave A3).
    ///
    /// ⚠️ **Ela é um alvo PRÓPRIO, e não uma excepção dentro do `Chrome`** — é o que o comentário
    /// do `Chrome` já prometia (*«quando um campo souber receber, ele entra aqui como alvo
    /// próprio»*). E é a lei D7 do plano: *a queda decide pelo TIPO DO ALVO*.
    ///
    /// ⛔ **Sem bandas.** O reparentar da Hierarquia reparte a linha em 30/40/30 porque ali há três
    /// respostas (irmão-acima · dentro · irmão-abaixo); aqui há **uma** — *põe isto nesta gaveta* —,
    /// e inventar bandas seria a terceira cópia de uma lei que já está escrita em dois sítios.
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

/// O que o cursor encontrou no canvas, na forma de que a lei precisa.
///
/// ⚠️ **É um resumo, e não a entidade**: o que decide é *«isto é uma sprite?»*, não *«qual
/// componente ela tem?»*. Passar a entidade obrigaria esta lei a consultar o mundo, e ela deixaria
/// de ser pura.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct DropOver {
    pub entity_bits: u64,
    /// Ela mostra pixels? ⇒ pode receber uma imagem.
    pub is_sprite: bool,
}

/// O que fazer. ⛔ Uma variante por EFEITO, não por gesto.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum DropAction {
    /// Instanciar o prefab `stable_id` com a raiz em `world`.
    PlacePrefab { stable_id: u64, world: [f32; 2] },
    /// A sprite `entity_bits` passa a mostrar `asset`.
    RetextureSprite { entity_bits: u64, asset: [u8; 32] },
    /// Nasce uma sprite nova em `world`, mostrando `asset`.
    SpawnImage { asset: [u8; 32], world: [f32; 2] },
    /// ⭐⭐ **Arrumar** — o asset passa a pertencer a `catalog` (`None` = a nenhum).
    Filecatalog {
        /// Quem foi largado.
        payload: DragPayload,
        /// A gaveta.
        catalog: Option<u128>,
    },
    /// ⛔ **Nada acontece, e VÊ-SE.** Ver o doc do módulo.
    Refuse,
    /// ⭐ **Nada acontece, e é SILENCIOSO** — o artista desistiu. Ver [`DropTarget::Source`].
    Cancel,
}

impl DropAction {
    /// A frase que o artista lê quando a queda é recusada.
    ///
    /// ⚠️ Ela nomeia **o que ele largou**, e não *«alvo inválido»*: a mensagem tem de o ajudar a
    /// perceber onde aquilo servia.
    #[must_use]
    pub fn refusal_line(payload: DragPayload) -> String {
        match payload {
            DragPayload::Prefab { .. } => "Drop a prefab on the canvas to place it".to_string(),
            DragPayload::Image { .. } => {
                "Drop an image on a sprite to retexture it, or on empty canvas".to_string()
            }
        }
    }
}

/// ⭐ **A LEI.** Pura e total.
///
/// ⚠️ **Exaustiva no que importa:** os braços de `Canvas` e os da ranhura casam a carga **por
/// variante**, então um `DragPayload` novo é **erro de compilação** aqui — que é a propriedade que
/// interessa. Os dois `_` que existem são sobre o ALVO (`Source`, `Chrome`), onde a carga de facto
/// não muda a resposta: desistir é desistir e recusar é recusar, venha o que vier.
///
/// ⛔ A 1.ª redacção deste doc dizia *«sem um `_` no match»* — e havia dois. *Uma afirmação sobre a
/// forma do código envelhece no primeiro braço que alguém acrescenta; a que vale é sobre a
/// PROPRIEDADE.*
#[must_use]
pub(crate) fn resolve(payload: DragPayload, target: DropTarget) -> DropAction {
    match (payload, target) {
        // ⭐ Desistir é silencioso — ver [`DropTarget::Source`].
        (_, DropTarget::Source) => DropAction::Cancel,

        // ⭐⭐ **Uma linha de catálogo aceita as DUAS famílias** — arrumar não olha ao tipo, ao
        // contrário de instanciar. É a resposta que a lei D7 dá aqui: o ALVO decide, e este alvo
        // sabe receber qualquer asset.
        (payload, DropTarget::CatalogRow { catalog }) => {
            DropAction::Filecatalog { payload, catalog }
        }

        // ⭐⭐⭐ **A RANHURA recebe uma IMAGEM, e é a MESMA acção que a queda sobre a sprite no
        // canvas** (wave B3): o efeito é *«esta sprite passa a mostrar isto»*, e ele não muda por o
        // gesto ter chegado pelo painel. *Uma segunda acção com o mesmo efeito seria dois sítios a
        // discordar no dia em que um deles for corrigido.*
        (DragPayload::Image { asset }, DropTarget::InspectorTexture { entity_bits }) => {
            DropAction::RetextureSprite { entity_bits, asset }
        }

        // ⛔ **Um prefab na ranhura RECUSA** — e a recusa vê-se. Uma receita não é uma imagem, e
        // «pôr um componente dentro do campo de textura» não é um gesto que exista.
        (DragPayload::Prefab { .. }, DropTarget::InspectorTexture { .. }) => DropAction::Refuse,

        // ⛔ Fora do canvas, das linhas de catálogo e da ranhura, nada sabe receber.
        (_, DropTarget::Chrome) => DropAction::Refuse,

        (DragPayload::Prefab { stable_id }, DropTarget::Canvas { world, .. }) => {
            // ⚠️ **Um prefab ignora o que está por baixo**, e é deliberado: largar sobre uma sprite
            // não pode significar *«substitui aquela sprite»* — isso é destruir trabalho com um
            // gesto de colocar.
            DropAction::PlacePrefab { stable_id, world }
        }

        (
            DragPayload::Image { asset },
            DropTarget::Canvas {
                over: Some(over), ..
            },
        ) if over.is_sprite => DropAction::RetextureSprite {
            entity_bits: over.entity_bits,
            asset,
        },

        // ⚠️ Sobre um objecto que **não** mostra pixels (um grupo, um objecto vazio, uma forma
        // vetorial), a imagem nasce como sprite nova **no ponto** — e não «entra» no objecto.
        // Retexturar o que não tem textura seria inventar um `Sprite` que o artista não pediu.
        (DragPayload::Image { asset }, DropTarget::Canvas { world, .. }) => {
            DropAction::SpawnImage { asset, world }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas(over: Option<DropOver>) -> DropTarget {
        DropTarget::Canvas {
            world: [3.0, 4.0],
            over,
        }
    }

    fn sprite(bits: u64) -> Option<DropOver> {
        Some(DropOver {
            entity_bits: bits,
            is_sprite: true,
        })
    }

    fn group(bits: u64) -> Option<DropOver> {
        Some(DropOver {
            entity_bits: bits,
            is_sprite: false,
        })
    }

    /// ⭐ Um prefab largado no canvas nasce **no ponto**, não com um deslocamento.
    #[test]
    fn a_prefab_lands_where_the_hand_let_go() {
        assert_eq!(
            resolve(DragPayload::Prefab { stable_id: 7 }, canvas(None)),
            DropAction::PlacePrefab {
                stable_id: 7,
                world: [3.0, 4.0]
            }
        );
    }

    /// ⛔ **E ignora o que está por baixo.** Largar sobre uma sprite não pode substituí-la — isso
    /// seria destruir trabalho com um gesto de colocar.
    #[test]
    fn a_prefab_over_a_sprite_still_just_lands() {
        assert_eq!(
            resolve(DragPayload::Prefab { stable_id: 7 }, canvas(sprite(99))),
            DropAction::PlacePrefab {
                stable_id: 7,
                world: [3.0, 4.0]
            }
        );
    }

    /// ⭐⭐ **Uma imagem sobre uma sprite RE-TEXTURA aquela sprite** — é a queda que responde
    /// *«qual objecto?»* com o dedo, que é o que o duplo-clique não consegue fazer.
    #[test]
    fn an_image_over_a_sprite_retextures_that_sprite() {
        assert_eq!(
            resolve(DragPayload::Image { asset: [5; 32] }, canvas(sprite(42))),
            DropAction::RetextureSprite {
                entity_bits: 42,
                asset: [5; 32]
            }
        );
    }

    /// E no vazio ela nasce como sprite nova, no ponto.
    #[test]
    fn an_image_on_empty_canvas_is_born_there() {
        assert_eq!(
            resolve(DragPayload::Image { asset: [5; 32] }, canvas(None)),
            DropAction::SpawnImage {
                asset: [5; 32],
                world: [3.0, 4.0]
            }
        );
    }

    /// ⚠️ **Sobre um objecto que não mostra pixels, ela também nasce** — retexturar o que não tem
    /// textura seria inventar um `Sprite` que o artista não pediu.
    #[test]
    fn an_image_over_a_non_sprite_is_born_instead_of_entering_it() {
        assert_eq!(
            resolve(DragPayload::Image { asset: [5; 32] }, canvas(group(42))),
            DropAction::SpawnImage {
                asset: [5; 32],
                world: [3.0, 4.0]
            }
        );
    }

    /// ⛔ **Fora do canvas, RECUSA** — as duas cargas, e cada uma com a sua frase.
    #[test]
    fn chrome_refuses_everything_and_says_what_the_thing_was_for() {
        for p in [
            DragPayload::Prefab { stable_id: 1 },
            DragPayload::Image { asset: [0; 32] },
        ] {
            assert_eq!(resolve(p, DropTarget::Chrome), DropAction::Refuse);
        }
        assert!(DropAction::refusal_line(DragPayload::Prefab { stable_id: 1 }).contains("canvas"));
        assert!(
            DropAction::refusal_line(DragPayload::Image { asset: [0; 32] }).contains("sprite"),
            "a recusa de uma imagem tem de dizer onde ela servia"
        );
        assert_ne!(
            DropAction::refusal_line(DragPayload::Prefab { stable_id: 1 }),
            DropAction::refusal_line(DragPayload::Image { asset: [0; 32] }),
            "duas cargas com a MESMA frase e' uma recusa que nao ensina nada"
        );
    }

    /// ⚠️ **A lei é TOTAL** — toda combinação tem uma resposta, e nenhuma é `Refuse` por omissão
    /// dentro do canvas. *Um `_ => Refuse` faria a próxima família nascer muda.*
    #[test]
    fn every_combination_inside_the_canvas_does_something() {
        for p in [
            DragPayload::Prefab { stable_id: 1 },
            DragPayload::Image { asset: [0; 32] },
        ] {
            for over in [None, sprite(1), group(1)] {
                assert_ne!(
                    resolve(p, canvas(over)),
                    DropAction::Refuse,
                    "{p:?} sobre {over:?} nao faz nada dentro do canvas"
                );
            }
        }
    }

    /// ⭐ **Voltar ao painel de origem é DESISTIR, e desistir é calado.**
    ///
    /// **Mutação que deve sangrar:** mapear `Source` para `Refuse` — o artista faria a coisa certa
    /// e levaria um aviso.
    #[test]
    fn dropping_back_on_the_source_panel_is_a_silent_cancel() {
        for p in [
            DragPayload::Prefab { stable_id: 1 },
            DragPayload::Image { asset: [0; 32] },
        ] {
            assert_eq!(resolve(p, DropTarget::Source), DropAction::Cancel);
            assert_ne!(
                resolve(p, DropTarget::Source),
                resolve(p, DropTarget::Chrome),
                "desistir e recusar nao podem ser a mesma coisa"
            );
        }
    }
}

#[cfg(test)]
mod catalog_target_tests {
    use super::*;

    /// ⭐⭐ **Uma linha de catálogo aceita as DUAS famílias** — arrumar não olha ao tipo, ao
    /// contrário de instanciar.
    ///
    /// **Mutação que deve sangrar:** trocar o braço da `CatalogRow` por `Refuse`.
    #[test]
    fn a_catalog_row_takes_a_prefab_and_an_image_alike() {
        for payload in [
            DragPayload::Prefab { stable_id: 7 },
            DragPayload::Image { asset: [3; 32] },
        ] {
            assert_eq!(
                resolve(payload, DropTarget::CatalogRow { catalog: Some(9) }),
                DropAction::Filecatalog {
                    payload,
                    catalog: Some(9)
                },
                "{payload:?} devia ser arrumado"
            );
        }
    }

    /// ⭐ **A linha *Unassigned* TIRA de qualquer catálogo** — e é isso que a torna um alvo e não um
    /// mero filtro.
    #[test]
    fn the_unassigned_row_takes_the_asset_out_of_its_catalog() {
        let payload = DragPayload::Image { asset: [1; 32] };
        assert_eq!(
            resolve(payload, DropTarget::CatalogRow { catalog: None }),
            DropAction::Filecatalog {
                payload,
                catalog: None
            }
        );
    }

    /// ⚠️ **A linha de catálogo não engoliu o resto**: o canvas continua a instanciar, e o chrome
    /// continua a recusar. *Um alvo novo que mudasse a resposta dos outros seria uma regressão que
    /// nenhum gate do alvo novo veria.*
    #[test]
    fn the_new_target_did_not_change_the_answers_of_the_old_ones() {
        let p = DragPayload::Prefab { stable_id: 1 };
        assert_eq!(
            resolve(
                p,
                DropTarget::Canvas {
                    world: [2.0, 3.0],
                    over: None
                }
            ),
            DropAction::PlacePrefab {
                stable_id: 1,
                world: [2.0, 3.0]
            }
        );
        assert_eq!(resolve(p, DropTarget::Chrome), DropAction::Refuse);
        assert_eq!(resolve(p, DropTarget::Source), DropAction::Cancel);
    }
}

/// Os gates da RANHURA DA TEXTURA (wave B3). ⚠️ Módulo irmão do `catalog_target_tests`, e pela
/// mesma razão: cada alvo novo traz os seus, e um ficheiro de testes que cresce sem costura é o
/// que faz ninguém achar o gate que interessa.
#[cfg(test)]
mod texture_slot_target_tests {
    use super::*;

    /// ⭐⭐⭐ **Uma IMAGEM na ranhura retextura a sprite que ela descreve** (wave B3) — e é a MESMA
    /// acção que a queda sobre a sprite no canvas.
    ///
    /// ⚠️ **O oráculo é a IGUALDADE das duas**, e não o valor de uma: se um dia elas divergirem, o
    /// mesmo gesto passa a ter dois significados conforme o sítio por onde chegou.
    #[test]
    fn an_image_on_the_slot_is_the_same_action_as_on_the_sprite() {
        let asset = [7u8; 32];
        let by_slot = resolve(
            DragPayload::Image { asset },
            DropTarget::InspectorTexture { entity_bits: 42 },
        );
        let by_canvas = resolve(
            DragPayload::Image { asset },
            DropTarget::Canvas {
                world: [0.0, 0.0],
                over: Some(DropOver {
                    entity_bits: 42,
                    is_sprite: true,
                }),
            },
        );
        assert_eq!(by_slot, by_canvas);
        assert_eq!(
            by_slot,
            DropAction::RetextureSprite {
                entity_bits: 42,
                asset
            }
        );
    }

    /// ⛔ **Um prefab na ranhura RECUSA** — e a recusa nomeia onde ele servia.
    ///
    /// ⚠️ ⛔ **Não é `Cancel`:** desistir é voltar ao painel de origem; isto é largar num sítio que
    /// não sabe receber, e tem de se ver.
    #[test]
    fn a_prefab_on_the_slot_is_refused_out_loud() {
        assert_eq!(
            resolve(
                DragPayload::Prefab { stable_id: 9 },
                DropTarget::InspectorTexture { entity_bits: 42 },
            ),
            DropAction::Refuse
        );
        assert!(
            DropAction::refusal_line(DragPayload::Prefab { stable_id: 9 }).contains("canvas"),
            "a recusa tem de dizer onde um prefab servia"
        );
    }

    /// ⛔ **O alvo novo não mudou as respostas dos antigos** — o irmão do gate homónimo da wave A3.
    ///
    /// ⚠️ *Um braço acrescentado a um `match` pode capturar o que ia para outro*, e a ordem dos
    /// braços é a lei: sem este gate, uma queda no canvas passaria a ser recusada e ninguém saberia
    /// dizer quando.
    #[test]
    fn the_slot_did_not_change_the_answers_of_the_old_targets() {
        let asset = [3u8; 32];
        assert_eq!(
            resolve(
                DragPayload::Image { asset },
                DropTarget::Canvas {
                    world: [1.0, 2.0],
                    over: None
                }
            ),
            DropAction::SpawnImage {
                asset,
                world: [1.0, 2.0]
            }
        );
        assert_eq!(
            resolve(DragPayload::Image { asset }, DropTarget::Chrome),
            DropAction::Refuse
        );
        assert_eq!(
            resolve(DragPayload::Image { asset }, DropTarget::Source),
            DropAction::Cancel
        );
    }
}
