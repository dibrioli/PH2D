//! Os gates do instantâneo e do dreno da secção HUD.

use ph2d_ecs::{Counter, CounterRuntime, Fit, LabelSource, SimWorld, UiButton, UiCanvas, UiLabel};
use ph2d_editor_core::hud_edits::{HudFieldEdit as E, HudNumber as N, HudText as T};
use ph2d_tags::TagTree;

use super::{apply, build_info};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Keep,
            },
            UiLabel {
                source: LabelSource::Counter("pontos".into()),
                prefix: "Pontos: ".into(),
                suffix: String::new(),
            },
            UiButton {
                signal: "bonus".into(),
                disabled: false,
            },
        ))
        .id();
    sim.world_mut().spawn((
        Counter {
            name: "pontos".into(),
            start: 0,
        },
        CounterRuntime { value: 12 },
    ));
    let bits = e.to_bits();
    (sim, bits)
}

/// ⛔ **Um objecto sem nenhum dos quatro não tem secção** — ADR-0166.
#[test]
fn um_objecto_sem_hud_nao_tem_seccao() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(ph2d_ecs::Transform::default()).id();
    assert!(build_info(&mut sim, &TagTree::default(), e.to_bits(), true).is_none());
    // O CONTROLO: com um componente do HUD, ela existe.
    let (mut sim, bits) = cena();
    assert!(build_info(&mut sim, &TagTree::default(), bits, true).is_some());
}

/// ⭐ O instantâneo traz o que o rótulo MOSTRA agora — pela mesma porta que o desenho usa.
#[test]
fn o_instantaneo_diz_o_que_o_rotulo_mostra_agora() {
    let (mut sim, bits) = cena();
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.vivo, "Pontos: 12");
    assert_eq!(i.counter_value, 0, "este objecto não é o contador");
    assert!(i.has_canvas && i.has_label && i.has_button && !i.has_counter);
}

/// ⭐ E diz que NÃO há câmera — a razão de o canvas não se colar a nada.
#[test]
fn o_instantaneo_diz_quando_nao_ha_camera_de_jogo() {
    let (mut sim, bits) = cena();
    let sem = build_info(&mut sim, &TagTree::default(), bits, false).expect("tem HUD");
    assert!(!sem.tem_camera);
    let com = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert!(com.tem_camera, "o CONTROLO");
}

/// ⭐⭐ **Trocar a fonte CONSERVA o nome** — senão um engano custa a digitação.
#[test]
fn trocar_a_fonte_nao_apaga_o_nome() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Source(2)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.source, 2, "passou a ler um relógio");
    assert_eq!(i.source_name, "pontos", "⛔ o nome sobreviveu à troca");
    // E de volta: o nome continua lá.
    assert!(apply(&mut sim, bits, &E::Source(1)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!((i.source, i.source_name.as_str()), (1, "pontos"));
}

/// ⛔ **Um lado ZERO na caixa de referência é recusado no COMPONENTE**, e não no painel: a lei do
/// `ph2d_hud::Canvas::new` recusa-o, e deixá-lo entrar poria o canvas a conduzir com escala
/// infinita até alguém reparar.
#[test]
fn a_caixa_nunca_fica_com_um_lado_zero() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Number(N::RefWidth, 0.0)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert!(i.ref_w > 0.0, "o zero não entra");
    assert!(
        ph2d_hud::Canvas::new(i.ref_w, i.ref_h, ph2d_hud::Fit::Keep).is_some(),
        "e o que fica é uma caixa que a lei aceita"
    );
}

/// Uma edição de um bloco AUSENTE é inerte, e não um erro.
#[test]
fn uma_edicao_de_um_bloco_ausente_e_inerte() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(ph2d_ecs::Transform::default()).id();
    let bits = e.to_bits();
    for edit in [
        E::Number(N::RefWidth, 4.0),
        E::Number(N::CounterStart, 3.0),
        E::Text(T::Signal, "x".into()),
        E::Fit(1),
        E::Source(1),
        E::Disabled(true),
    ] {
        assert!(!apply(&mut sim, bits, &edit), "{edit:?} não tem dono aqui");
    }
}

/// Cada campo volta ao componente certo — a ida-e-volta pelo instantâneo.
#[test]
fn cada_campo_volta_ao_componente_certo() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Fit(1)));
    assert!(apply(&mut sim, bits, &E::Text(T::Prefix, "P: ".into())));
    assert!(apply(&mut sim, bits, &E::Text(T::Suffix, " pts".into())));
    assert!(apply(&mut sim, bits, &E::Text(T::Signal, "again".into())));
    assert!(apply(&mut sim, bits, &E::Disabled(true)));
    assert!(apply(&mut sim, bits, &E::Number(N::RefHeight, 9.0)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.fit, 1);
    assert_eq!((i.prefix.as_str(), i.suffix.as_str()), ("P: ", " pts"));
    assert_eq!(i.signal, "again");
    assert!(i.disabled);
    assert!((i.ref_h - 9.0).abs() <= f32::EPSILON);
}

/// ⭐⭐⭐ **O selector do `Fit` oferece TODOS os modos da lei** — um por um, e na mesma ordem.
///
/// ⚠️⚠️ **Sem isto, um modo novo existe, tem lei, tem gates — e o artista não lhe chega.** É o
/// defeito que o `Density` da escultura e o verbo `Destroy` do FIM DE JOGO pagaram, cada um por um
/// array escrito à mão ao lado de uma tabela.
///
/// ⚠️ A régua é a IDA-E-VOLTA pelo índice do painel, e não uma contagem: contar `3 == 3` ficaria
/// verde com a ordem trocada, e o artista escolheria `Stretch` e receberia `Expand`.
///
/// **Mutação que deve sangrar:** tirar a última entrada do `Fit::ALL`.
#[test]
fn o_selector_do_fit_oferece_todos_os_modos() {
    use ph2d_ecs::{Fit, UiCanvas};
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::default(), UiCanvas::default()))
        .id();
    let bits = e.to_bits();

    let tree = TagTree::default();
    for esperado in Fit::ALL {
        // o artista escolhe o índice que o painel mostra…
        apply(
            &mut sim,
            bits,
            &E::Fit(u8::try_from(esperado.index()).expect("cabe")),
        );
        // …e o documento fica com o modo que ele leu.
        assert_eq!(
            sim.world().get::<UiCanvas>(e).expect("canvas").fit,
            esperado,
            "escolher o indice de {esperado:?} deu outro modo"
        );
        // e o instantâneo devolve o MESMO índice, senão a linha abre no modo errado.
        let i = build_info(&mut sim, &tree, bits, false).expect("info");
        assert_eq!(
            usize::from(i.fit),
            esperado.index(),
            "o painel abriria {esperado:?} noutra linha"
        );
    }
}

/// ⛔ **E o número de opções PINTADAS é o número de modos da lei.**
///
/// ⚠️ Metade que o gate de cima não dá: ele prova que cada modo VIAJA, e este que nenhum fica de
/// fora do menu. A lente é o texto do pintor, porque a lista dele é literal.
#[test]
fn o_menu_do_fit_tem_uma_linha_por_modo() {
    let src = include_str!("../../ph2d-panel-inspector/src/sections/hud_corpo.rs");
    // ⚠️⚠️ **A agulha é montada por pedaços de propósito:** escrita inteira, ela LÊ-SE COMO UMA
    // CHAVE e o censo do HR-15 acusa este ficheiro de usar um `tr` que não existe na tabela — foi
    // exactamente o que ele fez na 1.ª corrida. *Um gate que procura chaves não pode conter uma.*
    let prefixo = concat!("panel.inspector.", "hud.fit_");
    let i = src
        .find(&format!("{prefixo}keep"))
        .expect("o menu do Fit mudou de forma");
    let bloco = &src[i..src.len().min(i + 300)];
    let n = bloco.matches(prefixo).count();
    assert_eq!(
        n,
        ph2d_ecs::Fit::ALL.len(),
        "o menu pinta {n} opcoes e a lei tem {} modos",
        ph2d_ecs::Fit::ALL.len()
    );
}
