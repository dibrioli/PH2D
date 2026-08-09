//! Gates da cena do **painel gerado** (plano UI/UX W8b).

use super::*;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecPathRef, VecWidget};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene};

/// **O mundo que a tabela autorada descreve** — a mesma fonte que a cena usa para montar a árvore.
///
/// ⚠️ Construir aqui uma segunda lista à mão seria o buraco clássico: ela divergiria da cena no
/// dia em que uma row entrasse, e o gate de staleness ficaria verde sobre o painel errado.
pub(crate) fn world_from_authored() -> (SimWorld, VecScene, Entity) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut frame = None;
    let mut ents: Vec<Entity> = Vec::new();
    for (i, (r, name, kind)) in AUTHORED.iter().enumerate() {
        // ⚠️ A cena é construída da MESMA tabela, com a MESMA tinta e a MESMA geometria que o
        // smoke desenha — a cor da swatch e o SVG do ícone atravessam o `RowSpec`, então uma
        // segunda paleta (ou uma segunda forma) aqui faria o golden concordar com uma cena que
        // ninguém vê.
        let mut p: VecPath = super::authored_path(r, *kind);
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
        ents.push(e);
        if i == FRAME {
            frame = Some(e);
        }
        // ⚠️ **O parentesco vem da porta única**, não de um `else` local: a família de LISTA
        // pendura as opções no CONTROLE, e um harness que parenteia tudo na moldura emite um
        // golden com a faixa vazia — verde sobre um painel que ninguém desenhou.
        if let Some(pi) = super::authored_parent(i)
            && let Some(&pe) = ents.get(pi)
        {
            sim.world_mut().entity_mut(e).insert(ChildOf(pe));
        }
        if let Some(k) = kind {
            sim.world_mut()
                .entity_mut(e)
                .insert(VecWidget { kind: k.code() });
        }
        if let Some(slug) = super::authored_icon(name) {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::VecWidgetIcon { slug: slug.into() });
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
    let (sim, scene, frame) = world_from_authored();
    let plain = AUTHORED
        .iter()
        .skip(1)
        .filter(|(_, _, k)| k.is_none())
        .count();
    // ⚠️ **A contagem é DERIVADA da tabela, nunca um literal.** Ela já foi `7` cravado, e um
    // literal aqui só sabe dizer *"o número mudou"* — não *"a lei quebrou"*. O que este gate
    // afirma é a lei: **todo filho vestido vira row, e nenhum filho de desenho puro vira**.
    let rows = crate::ui_panel_spec::of(&sim, &scene, frame).rows.len();
    // Um filho de um controle de LISTA é OPÇÃO, não row — a lei de posse. Ele é vestido ou não
    // conforme o artista; o que conta é que ele não aparece no painel.
    let expected = AUTHORED
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(i, (_, _, k))| {
            k.is_some() && crate::ui_panel_smoke::authored_parent(*i) == Some(0)
        })
        .count();
    assert_eq!(
        rows, expected,
        "o painel não tem uma row por filho VESTIDO da MOLDURA (esperadas {expected}, rows {rows})"
    );
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
        assert_eq!(got.kind.ident(), want.kind, "o tipo da row divergiu");
        assert_eq!(got.label, want.label, "o rotulo da row divergiu");
        assert_eq!(got.key, want.key, "a chave da row divergiu");
        assert_eq!(got.rgba, want.rgba, "a cor da row divergiu");
        assert_eq!(got.icon, want.icon.as_deref(), "o icone da row divergiu");
        assert_eq!(
            got.icon_slug,
            want.icon_slug.as_deref(),
            "o icone ESCOLHIDO da row divergiu"
        );
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
