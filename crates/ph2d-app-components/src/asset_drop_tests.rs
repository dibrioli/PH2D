//! Os gates da lei da queda ([`super::resolve`]) — irmão de `asset_drop.rs` pelo tecto de LOC (GEF 21 E2).

mod tests {
    use super::super::*;

    fn canvas(over: Option<DropOver>) -> DropTarget {
        DropTarget::Canvas {
            world: [3.0, 4.0],
            over,
        }
    }

    fn sprite(bits: u64) -> Option<DropOver> {
        Some(DropOver { entity_bits: bits })
    }

    /// ⭐ Um prefab largado no canvas nasce **no ponto**, não com um deslocamento.
    #[test]
    fn a_prefab_lands_where_the_hand_let_go() {
        assert_eq!(
            resolve(DragPayload::Prefab { stable_id: 7 }, canvas(None)),
            DropAction::PlacePrefab {
                stable_id: 7,
                world: [3.0, 4.0],
                over: None
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
                world: [3.0, 4.0],
                over: Some(99)
            }
        );
    }

    /// ⭐⭐ **Uma imagem sobre uma sprite NASCE ao lado dela** (dono 26/10): o céu de fundo cobre a vista
    /// inteira, e a regra antiga trocava-lhe a imagem a cada queda.
    #[test]
    fn an_image_over_a_sprite_is_born_and_leaves_the_sprite_alone() {
        assert_eq!(
            resolve(DragPayload::Image { asset: [5; 32] }, canvas(sprite(42))),
            DropAction::SpawnImage {
                asset: [5; 32],
                world: [3.0, 4.0]
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

    /// ⭐ D17 — no Quadro aberto, um Kanban e uma imagem nascem no ponto; um Quadro (não há quadro
    /// dentro de quadro) e um prefab recusam, e a recusa do Quadro di-lo.
    #[test]
    fn asset_drop_on_the_board_canvas_places_a_kanban_and_refuses_a_board() {
        let on = DropTarget::BoardCanvas { at: [5.0, 6.0] };
        assert_eq!(
            resolve(DragPayload::Kanban { id: 3 }, on),
            DropAction::PlaceKanban {
                kanban: 3,
                at: [5.0, 6.0]
            }
        );
        assert_eq!(
            resolve(DragPayload::Image { asset: [7; 32] }, on),
            DropAction::PlaceImage {
                asset: [7; 32],
                at: [5.0, 6.0]
            }
        );
        for p in [
            DragPayload::Board { id: 1 },
            DragPayload::Prefab { stable_id: 1 },
        ] {
            assert_eq!(resolve(p, on), DropAction::Refuse);
        }
        let line = DropAction::refusal_line(DragPayload::Board { id: 1 });
        assert!(line.contains("inside a Board"), "{line}");
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
            for over in [None, sprite(1)] {
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

mod catalog_target_tests {
    use super::super::*;

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
                world: [2.0, 3.0],
                over: None
            }
        );
        assert_eq!(resolve(p, DropTarget::Chrome), DropAction::Refuse);
        assert_eq!(resolve(p, DropTarget::Source), DropAction::Cancel);
    }
}

/// Os gates da RANHURA DA TEXTURA (wave B3). ⚠️ Módulo irmão do `catalog_target_tests`, e pela
/// mesma razão: cada alvo novo traz os seus, e um ficheiro de testes que cresce sem costura é o
/// que faz ninguém achar o gate que interessa.
mod texture_slot_target_tests {
    use super::super::*;

    /// ⭐⭐⭐ **Uma IMAGEM na ranhura retextura a sprite que ela descreve** (wave B3) — o único arrastar
    /// que troca a imagem de uma sprite que já existe.
    #[test]
    fn an_image_on_the_slot_retextures_its_sprite() {
        let asset = [7u8; 32];
        let by_slot = resolve(
            DragPayload::Image { asset },
            DropTarget::InspectorTexture { entity_bits: 42 },
        );
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

    /// ⭐⭐ **Um objecto da cena só se larga no TEXTO de um script** (item 8 do CodeEditor); em todo
    /// outro sítio recusa e vê-se — e voltar à Hierarquia é desistir. Um mestre ou uma imagem no
    /// texto escreve o nome (8b); fora dele, recusa.
    #[test]
    fn um_objecto_so_se_larga_no_texto_de_um_script() {
        let o = DragPayload::Object { stable_id: 7 };
        assert_eq!(
            resolve(o, DropTarget::Code { on_text: true }),
            DropAction::WriteCode
        );
        for t in [
            DropTarget::Code { on_text: false },
            DropTarget::Canvas {
                world: [0.0, 0.0],
                over: None,
            },
            DropTarget::Canvas {
                world: [0.0, 0.0],
                over: Some(DropOver { entity_bits: 3 }),
            },
            DropTarget::InspectorTexture { entity_bits: 3 },
            DropTarget::CatalogRow { catalog: None },
            DropTarget::Chrome,
        ] {
            assert_eq!(resolve(o, t), DropAction::Refuse, "{t:?}");
        }
        assert_eq!(resolve(o, DropTarget::Source), DropAction::Cancel);
        for p in [
            DragPayload::Prefab { stable_id: 1 },
            DragPayload::Image { asset: [1; 32] },
        ] {
            assert_eq!(
                resolve(p, DropTarget::Code { on_text: true }),
                DropAction::WriteCode
            );
            assert_eq!(
                resolve(p, DropTarget::Code { on_text: false }),
                DropAction::Refuse
            );
        }
        assert!(!DropAction::refusal_line(o).is_empty());
    }

    /// Uma linha do Inspector (8b) só sabe ir para o texto; em qualquer outro alvo desiste calada.
    #[test]
    fn a_property_row_writes_code_on_text_and_cancels_everywhere_else() {
        let p = DragPayload::Property { script: 0 };
        assert_eq!(
            resolve(p, DropTarget::Code { on_text: true }),
            DropAction::WriteCode
        );
        for t in [
            DropTarget::Code { on_text: false },
            DropTarget::Chrome,
            DropTarget::Canvas {
                world: [0.0, 0.0],
                over: None,
            },
            DropTarget::CatalogRow { catalog: None },
            DropTarget::InspectorTexture { entity_bits: 1 },
            DropTarget::ScriptObject {
                entity_bits: 1,
                row: 0,
            },
        ] {
            assert_eq!(resolve(p, t), DropAction::Cancel, "{t:?}");
        }
    }
}
