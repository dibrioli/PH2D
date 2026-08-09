//! Gates da cena do **painel gerado** (plano UI/UX W8b).

use super::*;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecPathRef, VecWidget};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene, rectangle};

/// **O mundo que a tabela autorada descreve** — a mesma fonte que a cena usa para montar a árvore.
///
/// ⚠️ Construir aqui uma segunda lista à mão seria o buraco clássico: ela divergiria da cena no
/// dia em que uma row entrasse, e o gate de staleness ficaria verde sobre o painel errado.
pub(crate) fn world_from_authored() -> (SimWorld, VecScene, Entity) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut frame = None;
    for (i, (r, name, kind)) in AUTHORED.iter().enumerate() {
        // ⚠️ A cena é construída da MESMA tabela e com a MESMA tinta que o smoke desenha — a cor
        // da swatch atravessa o `RowSpec`, então uma segunda paleta aqui faria o golden concordar
        // com uma cena que ninguém vê.
        let mut p: VecPath = rectangle([r[0], r[1]], [r[2], r[3]]);
        let c = super::authored_fill(i, *kind);
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        scene.push_path(p);
        let e = sim
            .world_mut()
            .spawn((
                Name((*name).into()),
                Transform::IDENTITY,
                VecPathRef(i as u64),
            ))
            .id();
        if i == FRAME {
            frame = Some(e);
        } else if let Some(f) = frame {
            sim.world_mut().entity_mut(e).insert(ChildOf(f));
        }
        if let Some(k) = kind {
            sim.world_mut()
                .entity_mut(e)
                .insert(VecWidget { kind: k.code() });
        }
    }
    (
        sim,
        scene,
        frame.expect("a moldura e a primeira linha da tabela"),
    )
}

/// **O CONTROLE da cena existe** — e ele não é decoração.
///
/// ⚠️ A cena prova *só quem VESTE vira row*, e a única coisa que a torna capaz disso é haver um
/// filho **sem** `VecWidget`. Alguém que "complete a tabela" vestindo todos apaga o controle em
/// silêncio: a cena continua bonita e deixa de provar a lei.
#[test]
fn the_scene_keeps_a_child_that_is_only_drawing() {
    let dressed = AUTHORED
        .iter()
        .skip(1)
        .filter(|(_, _, k)| k.is_some())
        .count();
    let plain = AUTHORED
        .iter()
        .skip(1)
        .filter(|(_, _, k)| k.is_none())
        .count();
    assert_eq!(dressed, 5, "a cena deixou de ter cinco filhos vestidos");
    assert!(
        plain >= 1,
        "a cena perdeu o filho de desenho puro — o CONTROLE"
    );
}

/// **O plano que a cena descreve é o que o golden diz** — o gate de staleness.
///
/// ⚠️ Ele é o que discharge o risco nomeado no §10.4 do plano (*"codegen que o CI recusa"*): o
/// arquivo commitado é **compilado pela crate do painel** (`ph2d_panel_authored::generated`),
/// então um gerador que emitisse Rust inválido não chega ao `main` — o build cai antes. Na W8b.1
/// ele era compilado num módulo `cfg(test)` da shell, cujas consts ninguém lia; agora é a lista
/// que o runtime de rows percorre, e a mesma prova ficou mais forte: um formato que o runtime não
/// consegue percorrer também deixa de compilar.
///
/// ⚠️ E ele compara **bytes**, não propriedades: o determinismo do emissor é o que torna isto
/// possível, e sem a comparação exata um gerador poderia mudar de formato sem ninguém notar.
#[test]
fn the_generated_panel_is_not_stale() {
    let (sim, scene, frame) = world_from_authored();
    let got = ph2d_ui_codegen::emit(&crate::ui_panel_spec::of(&sim, &scene, frame));
    // ⚠️ Do lugar onde o produto o COMPILA. Uma cópia na shell seria um segundo golden, e o gate
    // ficaria verde comparando o gerador consigo mesmo enquanto o painel pinta outra coisa.
    let want = include_str!("../../../crates/ph2d-panel-authored/src/generated/panel.rs");
    assert_eq!(
        got, want,
        "o gerador mudou e o arquivo commitado ficou para tras.\n\
         Refaca-o com: `cargo test -p ph2d-host-desktop --bins print_the_generated_panel -- \
         --ignored --nocapture` e cole a saida em \
         crates/ph2d-panel-authored/src/generated/panel.rs"
    );
}

/// **E o que o golden COMPILADO carrega é o que o plano diz.**
///
/// ⚠️ Este gate não é redundante com o de bytes acima, e a diferença é a que decide: aquele prova
/// que o **texto** não envelheceu; este lê as consts **depois de o compilador as ter aceitado** e
/// confere que elas descrevem o mesmo painel. Um emissor que produzisse Rust válido mas com as
/// rows trocadas passaria no primeiro se o golden fosse regenerado junto — e cai neste.
#[test]
fn the_compiled_golden_carries_the_same_panel() {
    use ph2d_panel_authored::generated::{PANEL_ID, PANEL_TITLE, ROWS};

    let (sim, scene, frame) = world_from_authored();
    let spec = crate::ui_panel_spec::of(&sim, &scene, frame);

    assert_eq!(PANEL_ID, spec.id);
    assert_eq!(PANEL_TITLE, spec.title);
    assert_eq!(
        ROWS.len(),
        spec.rows.len(),
        "o golden tem outro numero de rows"
    );
    for (got, want) in ROWS.iter().zip(&spec.rows) {
        assert_eq!(got.0.ident(), want.kind, "o tipo da row divergiu");
        assert_eq!(got.1, want.label, "o rotulo da row divergiu");
        assert_eq!(got.2, want.key, "a chave da row divergiu");
        assert_eq!(got.3, want.rgba, "a cor da row divergiu");
    }
}

/// Imprime o código gerado, para regenerar o golden à mão.
#[test]
#[ignore = "utilitario — roda a pedido para regenerar o golden"]
fn print_the_generated_panel() {
    let (sim, scene, frame) = world_from_authored();
    print!(
        "{}",
        ph2d_ui_codegen::emit(&crate::ui_panel_spec::of(&sim, &scene, frame))
    );
}
