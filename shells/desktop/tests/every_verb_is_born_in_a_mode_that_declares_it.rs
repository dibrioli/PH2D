//! **A SHELL NÃO CARIMBA UM MODO ONDE ELE NÃO TEM LEI** (ADR-0150).
//!
//! O `Sculpt3dUi::default()` do painel deriva o modo de nascimento de cada verbo
//! desde a W6 — e **não é ele quem faz o estado nascer**. O
//! `shells/desktop/src/sculpt3d_birth.rs` escrevia
//! `[RefMode::default(); Verb::ALL.len()]`, um `S` chapado, e a segunda resposta
//! era a que shipava: medido em `crates/ph2d-sculpt3d/tests/measure_layer_law.rs`
//! (P8), **7 dos 23 verbos** nasciam num modo que não os declara — os sete cuja
//! referência é o Blender.
//!
//! ⚠️ **O preço não era o chip apagado, era a FORMA DO BARRO.** O
//! `Brush::weight` pergunta `profile(mode)`, e `profile(S, Layer)` é `None` ⇒ o
//! slider virava o peso CRU onde o `layer.cc` o eleva ao quadrado: **0,5000
//! contra 0,2500**, o dobro da taxa de depósito. Numa lei que satura, o dobro da
//! taxa não deposita mais alto — ele **colapsa o ombro que o falloff desenha**.
//!
//! ⚠️ **Este gate é de FONTE porque o produto não é alcançável sem device:** o
//! `Sculpt3dScene::new` recebe um `wgpu::Device`, então não há como construir o
//! nascimento num teste headless. O que ele afirma é a PROPRIEDADE (*a shell
//! pergunta à porta*), não um endereço — e traz controle positivo para não
//! passar por varredura vazia.

use std::path::Path;

#[test]
fn every_verb_is_born_in_a_mode_that_declares_it() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sculpt3d_birth.rs");
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("o dono do nascimento mudou-se: {} ({e})", path.display()));

    // CONTROLE POSITIVO: sem ele, um arquivo renomeado deixaria as asserções
    // abaixo verdes sobre uma varredura vazia.
    assert!(
        src.contains("Sculpt3dScene {"),
        "o dono do nascimento mudou-se — reaponte o gate antes de confiar nele"
    );

    // ⚠️ **A PORTA mudou de nome em 2026-08-17 e a PROPRIEDADE não.** Antes o
    // nascimento chamava `RefMode::birth_for` direto; hoje ele semeia a tabela
    // POR VERBO (`VerbSlot::for_verb`), que é quem pergunta àquela porta — o
    // modo deixou de ser o único fato que um verbo lembra, e passou a viajar
    // dentro do pincel dele. O que este gate afirma continua sendo *a shell
    // DELEGA*, e a metade comportamental (*o que a porta devolve declara o
    // verbo?*) mora onde é alcançável sem device:
    // `ph2d-panel-sculpt3d::state_tests::the_panel_opens_every_verb_in_a_mode_that_declares_it`.
    assert!(
        src.contains("VerbSlot::for_verb"),
        "o nascimento tem de PERGUNTAR a porta (`VerbSlot::for_verb`), e não \
         escolher um estado de fábrica por conta própria"
    );
    assert!(
        !src.contains("[ph2d_sculpt3d::RefMode::default();"),
        "o `[RefMode::default(); N]` é exactamente o defeito: ele carimba o `S` \
         em sete verbos que o `S` não declara"
    );
    // ⚠️ **E o `Brush::default()` chapado é o MESMO defeito uma camada acima:**
    // ele daria a TODO verbo a força, a curva e a dureza do Draw, que é o
    // estado de onde a propagação entre ferramentas nasce.
    assert!(
        !src.contains("[Brush::default();"),
        "um `[Brush::default(); N]` faria os 23 verbos nascerem com o pincel do \
         Draw — a tabela existe justamente para eles nascerem diferentes"
    );
}
