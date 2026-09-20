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
            keep_on_restart: false,
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

/// ⛔⛔⛔ **E o número de SEGMENTOS pintados é o número de modos da lei.**
///
/// ⚠️⚠️ **A 1.ª redacção deste gate contava os RÓTULOS no fonte do pintor e era CEGA ao defeito
/// que o dono reportou** (*«só tem as opções de keep e Stretch. Não expand»*): o pintor passava os
/// três rótulos e o array `INSP_HUD_FIT` tinha **dois** ids — e o segmentado pinta **um segmento
/// por id**, logo o terceiro rótulo era simplesmente ignorado. *Uma régua que conta rótulos não vê
/// quantos SEGMENTOS são pintados.*
///
/// ⇒ a régua passa a ser o **ARRAY DE IDS**, que é o que os TRÊS consumidores lêem (o pintor, o
/// `populate` que os regista, e o despacho do clique). Os rótulos ficam como a segunda metade.
#[test]
fn o_menu_do_fit_tem_uma_linha_por_modo() {
    let src = include_str!("../../ph2d-panel-inspector/src/sections/hud_corpo.rs");
    // ⚠️⚠️ **A agulha é montada por pedaços de propósito:** escrita inteira, ela LÊ-SE COMO UMA
    // CHAVE e o censo do HR-15 acusa este ficheiro de usar um `tr` que não existe na tabela — foi
    // exactamente o que ele fez na 1.ª corrida. *Um gate que procura chaves não pode conter uma.*
    // ⚠️ **A metade que decide o que se VÊ — o array de ids — mora no PAINEL**, porque esta crate
    // não o vê (seria subir uma camada): `o_segmentado_do_fit_tem_um_id_por_modo`, na
    // `ph2d-panel-inspector`. Aqui fica a dos RÓTULOS, que é a segunda.
    let prefixo = concat!("panel.inspector.", "hud.fit_");
    let i = src
        .find(&format!("{prefixo}keep"))
        .expect("o menu do Fit mudou de forma");
    let bloco = &src[i..src.len().min(i + 300)];
    let n = bloco.matches(prefixo).count();
    assert_eq!(
        n,
        ph2d_ecs::Fit::ALL.len(),
        "o menu pinta {n} rotulos e a lei tem {} modos",
        ph2d_ecs::Fit::ALL.len()
    );
}

/// ⭐⭐⭐ **A secção DIZ onde está a metade que ela não mostra** — report do dono, 20/09, com foto.
///
/// ⚠️⚠️ As linhas do canvas (`Fit`, a caixa de referência) pintam-se **só na raiz**; com um rótulo
/// escolhido a secção mostrava a metade dele e **calava-se** sobre a outra. *Uma secção que mostra
/// metade e não diz onde está a outra faz o artista concluir que ela não existe* — e foi
/// exactamente o que aconteceu.
///
/// ⚠️ **As DUAS metades**, e a negativa é a que impede o ruído: numa raiz as linhas estão à vista,
/// e um aviso ali seria uma frase que nunca ajuda ninguém.
///
/// **Mutação que deve sangrar:** devolver `canvas_parent` sempre, mesmo na raiz.
#[test]
fn a_seccao_diz_onde_estao_as_linhas_do_canvas() {
    use ph2d_ecs::{ChildOf, Name, Transform, UiCanvas};
    let mut sim = SimWorld::default();
    let raiz = sim
        .world_mut()
        .spawn((Transform::default(), UiCanvas::default(), Name::new("HUD")))
        .id();
    let filho = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(raiz),
            UiLabel {
                source: LabelSource::Counter("pontos".into()),
                prefix: String::new(),
                suffix: String::new(),
            },
        ))
        .id();
    let tree = TagTree::default();

    // (a) no FILHO, a nota nomeia a raiz.
    let i = build_info(&mut sim, &tree, filho.to_bits(), false).expect("info do filho");
    assert!(!i.has_canvas, "o filho nao devia ter as linhas do canvas");
    assert_eq!(
        i.canvas_parent.as_deref(),
        Some("HUD"),
        "a seccao nao diz ONDE estao as linhas que ela nao mostra"
    );

    // (b) na RAIZ, ela cala-se — as linhas estão à vista.
    let r = build_info(&mut sim, &tree, raiz.to_bits(), false).expect("info da raiz");
    assert!(r.has_canvas);
    assert_eq!(
        r.canvas_parent, None,
        "a raiz mostra as linhas E um aviso a dizer onde elas estao"
    );

    // ⚠️⚠️ **(c) um canvas DEBAIXO de outro canvas — e é este o caso que o guarda existe para
    // cobrir.** A 1.ª redacção deste gate parava em (b), e a mutação que trocava
    // `if canvas.is_some()` por `if false` **SOBREVIVEU**: a raiz de (b) não tem pai nenhum, logo
    // o ramo mutado devolvia `None` na mesma. *Uma fixtura que não contém o fenómeno não o
    // reprova* — e um canvas aninhado é legal, logo o caso é real e não um espantalho.
    let dentro = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(raiz),
            UiCanvas::default(),
            Name::new("HUD de dentro"),
        ))
        .id();
    let d = build_info(&mut sim, &tree, dentro.to_bits(), false).expect("info do aninhado");
    assert!(d.has_canvas, "ele TEM canvas proprio");
    assert_eq!(
        d.canvas_parent, None,
        "um canvas dentro de outro mostra as proprias linhas E um aviso a apontar para o pai"
    );
}

/// ⛔ **E um filho de um objecto QUALQUER não inventa uma raiz** — a nota é sobre um canvas, e só.
#[test]
fn um_filho_de_um_objecto_comum_nao_nomeia_raiz_nenhuma() {
    use ph2d_ecs::{ChildOf, Name, Transform};
    let mut sim = SimWorld::default();
    let pai = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Nao e um canvas")))
        .id();
    let filho = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(pai),
            UiLabel {
                source: LabelSource::Counter("pontos".into()),
                prefix: String::new(),
                suffix: String::new(),
            },
        ))
        .id();
    let i = build_info(&mut sim, &TagTree::default(), filho.to_bits(), false).expect("info");
    assert_eq!(i.canvas_parent, None);
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ A CAIXA «Keep on Restart» — a IDA e a VOLTA (2026-09-20)
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐ **A edição chega ao componente, e o instantâneo lê-a de volta.**
///
/// ⚠️ **As duas metades, e nenhuma basta:** sem a IDA a caixa é decoração; sem a VOLTA ela
/// desmarca-se sozinha no quadro seguinte (o painel semeia do instantâneo a cada quadro), que é
/// o defeito que se lê como *«a caixa não fica marcada»*.
///
/// **Mutação que deve sangrar:** o braço `CounterKeep` a cravar `false`; ou o `build_info` a
/// devolver `counter_keep: false`.
#[test]
fn a_caixa_do_recomeco_faz_a_ida_e_a_volta() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn(Counter {
            name: "pontos".into(),
            start: 0,
            keep_on_restart: false,
        })
        .id();
    let tree = TagTree::default();

    // CONTROLO: ele nasce desligado dos dois lados.
    let antes = build_info(&mut sim, &tree, e.to_bits(), false).expect("info");
    assert!(!antes.counter_keep, "CONTROLO: nasce desligado");

    assert!(
        apply(&mut sim, e.to_bits(), &E::CounterKeep(true)),
        "a edicao tem de ser aceite"
    );
    assert!(
        sim.world()
            .get::<Counter>(e)
            .expect("o contador")
            .keep_on_restart,
        "IDA: a edicao tem de chegar ao componente"
    );
    let depois = build_info(&mut sim, &tree, e.to_bits(), false).expect("info");
    assert!(depois.counter_keep, "VOLTA: o instantaneo tem de a ler");

    // E desmarcar volta atras — senão a caixa é um interruptor de um sentido só.
    assert!(apply(&mut sim, e.to_bits(), &E::CounterKeep(false)));
    assert!(
        !sim.world()
            .get::<Counter>(e)
            .expect("o contador")
            .keep_on_restart
    );
}
