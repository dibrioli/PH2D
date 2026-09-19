//! ⭐⭐⭐ **A LINHA DA COR** — os três canais dobrados numa amostra, e a travessia sRGB ↔ linear.
//!
//! Enio, 2026-09-14: *«em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*.
//!
//! # ⚠️ As três perguntas, e porque são três gates
//!
//! | pergunta | gate |
//! |---|---|
//! | a linha nasce **uma** e traz a cor da peça? | [`a_colour_is_one_row_and_it_carries_the_swatch`] |
//! | o que o artista escolhe **chega ao documento**? | [`the_colour_intent_reaches_the_document`] |
//! | escolher a cor que já lá está **não escreve nada**? | [`the_round_trip_through_the_document_is_exact`] |
//!
//! ⛔ A terceira parece cosmética e é a que impede um **passo de desfazer por quadro**: o painel
//! pergunta *«mudou?»* comparando bytes, e um ida-e-volta que não fecha pede a edição para sempre.
//!
//! ⚠️ **Irmão por assunto** (o `scene_tests.rs` está a `552` de `600`) — ⛔ *split, nunca allowlist*.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

/// Uma peça de UMA folha — uma esfera. A cor é da folha, e é ela que o painel mostra.
pub(crate) fn a_ball() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("uma esfera");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folha = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .and_then(|c| c.iter().next().copied())
        .unwrap_or(root);
    (sim, folha)
}

/// As linhas que o painel publica para uma entidade — pelo caminho de produção, que **drena os
/// pedidos antes de publicar**.
///
/// ⛔ **Ela NÃO drena por si.** A 1.ª redacção começava por um `drain_intents()` defensivo e com
/// isso **comia o pedido que o gate acabara de empurrar** — o `the_colour_intent_reaches_the_document`
/// leu a cor de omissão e acusou uma costura partida que estava inteira. *Um arnês que limpa a fila
/// mede o programa em que ninguém pediu nada.*
pub(crate) fn rows_of(sim: &mut SimWorld, e: Entity) -> Vec<ph2d_panel_model3d::ParamRow> {
    crate::scene::sync_scene_and_birth(sim, None, &[e], 0.0, &crate::scene::no_drawing());
    ph2d_panel_model3d::state::current().rows
}

/// ⭐⭐⭐ **OS TRÊS CANAIS SÃO UMA LINHA, E ELA TRAZ A COR** — em sRGB8, que é o que se vê.
///
/// # ⛔ As duas metades, e porque nenhuma basta
///
/// *Uma linha* sem *a cor certa* é uma amostra cinzenta sobre uma peça vermelha; *a cor certa* em
/// *três linhas* é a dívida que esta wave paga. O gate afirma as duas, e afirma também que a cor
/// **SEGUE** o documento — senão ele passaria sobre uma amostra congelada no valor de omissão.
///
/// **Mutações que devem sangrar:** apagar o `filter` dos canais `1 | 2`; devolver `None` no
/// `swatch`; trocar `colour_srgb8` por uma conversão linear (a peça a `0,8` leria `204` em vez
/// de `231`).
#[test]
fn a_colour_is_one_row_and_it_carries_the_swatch() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = a_ball();
    let rows = rows_of(&mut sim, folha);
    let cor: Vec<&ph2d_panel_model3d::ParamRow> =
        rows.iter().filter(|r| r.swatch.is_some()).collect();
    // ⚠️ **QUATRO amostras, sempre** — as quatro cores do material (§22). Duas nascem vivas (a base
    // e a do realce) e duas nascem **travadas** (a do verniz e a do brilho): o que o estado da peça
    // muda é o `live`, nunca a presença — ordem do Enio, 14/09.
    // ⚠️⚠️ **O par `(param, chave)`, e não só a chave** — uma mutação que moveu a âncora do realce
    // de `7` para `6` **sobreviveu** à primeira redacção: a linha do `specular_weight` recebia o
    // rótulo da cor e a lista de chaves continuava a bater. *Um gate que lê só o rótulo não sabe
    // sobre que número ele está escrito.*
    assert_eq!(
        cor.iter().map(|r| (r.param, r.key)).collect::<Vec<_>>(),
        vec![
            (ph2d_field::Param::Material(1), "field.dim.base_color"),
            (ph2d_field::Param::Material(7), "field.dim.specular_color"),
            (ph2d_field::Param::Material(13), "field.dim.coat_color"),
            (ph2d_field::Param::Material(20), "field.dim.emission_color"),
            // ⚠️ **As duas da SUBSUPERFICIE entraram em 17/09** (`docs/Render3d/10`): a cor do
            // meio que espalha, e a escala do raio POR CANAL — que e' uma cor porque diz quanto
            // mais fundo cada canal viaja, e e' o que poe o vermelho a' frente numa orelha.
            (
                ph2d_field::Param::Material(24),
                "field.dim.subsurface_color"
            ),
            (
                ph2d_field::Param::Material(28),
                "field.dim.subsurface_scale"
            ),
        ],
        "as amostras do material mudaram: {:?}",
        rows.iter().map(|r| (r.param, r.key)).collect::<Vec<_>>()
    );
    // ⛔ **E os outros dois canais NÃO são linha** — senão o artista teria a amostra *e* dois
    // sliders do mesmo facto, que é a lei que o `ParamRow::swatch` declara.
    assert!(
        !rows
            .iter()
            .any(|r| matches!(r.param, ph2d_field::Param::Material(2 | 3))),
        "os canais verde e azul continuam a ser linhas próprias: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⚠️ **E o resto do material continua lá** — a dobra é dos três canais, não da secção.
    assert!(
        rows.iter()
            .any(|r| r.param == ph2d_field::Param::Material(10))
            && rows
                .iter()
                .any(|r| r.param == ph2d_field::Param::Material(5)),
        "a rugosidade e o metal desapareceram com a dobra"
    );

    // ⭐ **A cor de omissão, vista**: o documento guarda `0,8` LINEAR, que em sRGB8 é `231`.
    // ⛔ Um `0,8 × 255 = 204` aqui seria a conversão ingénua — e a amostra sairia visivelmente
    // mais escura do que a peça que o traçado desenha ao lado dela.
    assert_eq!(
        cor[0].swatch,
        Some([231, 231, 231]),
        "a amostra não mostra a cor de omissão do material"
    );

    // ⭐⭐ **E ela SEGUE o documento.** Sem esta metade, uma amostra congelada no default passaria.
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(2), 0.0)
        .expect("o verde");
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(3), 0.0)
        .expect("o azul");
    let rows = rows_of(&mut sim, folha);
    let agora = rows
        .iter()
        .find_map(|r| r.swatch)
        .expect("a linha da cor continua lá");
    assert_eq!(
        agora,
        [231, 0, 0],
        "a amostra não seguiu o documento — ela mostra uma cor que a peça não tem"
    );
}

/// ⭐⭐⭐ **O QUE O ARTISTA ESCOLHE CHEGA AO DOCUMENTO** — a costura do pedido, sem app.
///
/// ⚠️ **Os TRÊS canais num pedido só, e isso é o que o torna UM passo de desfazer**: o registo é por
/// diff uma vez por quadro, então o que importa não é a chamada ser uma, é não haver quadro entre as
/// três escritas.
///
/// **Mutação que deve sangrar:** escrever só o canal `0` no braço do `SetColor`.
#[test]
fn the_colour_intent_reaches_the_document() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = a_ball();
    let _ = rows_of(&mut sim, folha);
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        // ⚠️ A ÂNCORA da cor BASE — as outras três têm gates próprios nos ficheiros irmãos.
        anchor: ph2d_field::Param::Material(1),
        entity: folha.to_bits(),
        srgb: [255, 0, 128],
    });
    let rows = rows_of(&mut sim, folha);
    assert_eq!(
        rows.iter().find_map(|r| r.swatch),
        Some([255, 0, 128]),
        "a cor escolhida não voltou pela linha — ou o pedido não chegou, ou não fecha o \
         ida-e-volta"
    );
    // ⭐ **E ela mora no COMPONENTE**, que é o que o traçado lê e o que o ficheiro grava.
    let m = sim
        .world()
        .get::<ph2d_field_ecs::FieldMaterial>(folha)
        .copied()
        .expect("escrever a cor materializa o material");
    assert_eq!(
        m.base_color,
        crate::materials::colour_from_srgb8([255, 0, 128]),
        "os três canais do componente não são os três que o artista apontou"
    );
}

/// ⭐⭐⭐ **O IDA-E-VOLTA PELO DOCUMENTO É EXACTO, nos 256 bytes.**
///
/// # ⛔ Porque isto não é uma curiosidade de colorimetria
///
/// O painel pergunta *«a cor mudou?»* comparando **bytes**: ele lê o selector, converte o documento
/// para sRGB8, e só pede a edição se os dois diferirem. Um único byte que não voltasse ao mesmo
/// valor faria essa comparação ser **sempre verdadeira** naquela cor ⇒ um pedido de edição **por
/// quadro** enquanto o selector estivesse aberto, e um passo de desfazer por cada largar de botão.
///
/// ⚠️ **E o caminho medido é o do PRODUTO**, `f32` incluído: a cor passa pelo `FieldMaterial`, que
/// guarda `f32`. Medir só as duas funções da `ph2d-color` mediria outro programa.
#[test]
fn the_round_trip_through_the_document_is_exact() {
    let mau: Vec<u8> = (0u8..=255)
        .filter(|&b| {
            let ida = crate::materials::colour_from_srgb8([b, b, b]);
            crate::materials::colour_srgb8(ida) != [b, b, b]
        })
        .collect();
    assert!(
        mau.is_empty(),
        "⛔ {} byte(s) não sobrevivem ao ida-e-volta pelo documento: {mau:?} — nessas cores o \
         painel pediria uma edição por quadro",
        mau.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// O MATERIAL DA SELECÇÃO — pintar várias formas de uma vez (14/09)
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// Uma peça de TRÊS esferas em união — a raiz é o grupo, e as folhas são os filhos dela.
fn three_balls() -> (SimWorld, Entity, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let bola = |x: f32| ph2d_field::Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
        verb: None,
    };
    let doc = FieldDoc::new(
        vec![
            bola(-0.5),
            bola(0.0),
            bola(0.5),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(3),
    )
    .expect("três esferas");
    let grupo = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folhas: Vec<Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(grupo)
        .expect("o grupo tem filhos")
        .iter()
        .copied()
        .collect();
    assert_eq!(folhas.len(), 3);
    (sim, grupo, folhas)
}

/// A cor que cada folha mostra hoje.
fn cores(sim: &SimWorld, folhas: &[Entity]) -> Vec<[u8; 3]> {
    folhas
        .iter()
        .map(|&f| {
            crate::materials::colour_srgb8(
                sim.world()
                    .get::<ph2d_field_ecs::FieldMaterial>(f)
                    .copied()
                    .unwrap_or_default()
                    .base_color,
            )
        })
        .collect()
}

/// ⭐⭐⭐ **UM GRUPO OFERECE O MATERIAL DAS FORMAS DEBAIXO DELE** — e pintá-lo pinta-as a todas.
///
/// # ⛔ O que ele não pode fazer, e porquê
///
/// Um grupo **não tem** material: quem o traçado sabe nomear por pixel é a folha. ⇒ o que ele
/// oferece não é um material dele, é o das **folhas da sub-árvore** — e a escrita cai lá.
///
/// ⚠️ **As duas metades, e nenhuma basta:** oferecer sem escrever é o botão mudo; escrever sem
/// oferecer é o gesto inalcançável. É a lei da W34 outra vez, sobre uma fileira nova.
///
/// **Mutações que devem sangrar:** o `material_for_the_selection` a devolver cedo num grupo; o
/// `material_reach` a devolver `vec![dele]` sempre.
#[test]
fn a_group_offers_the_material_of_the_shapes_under_it_and_painting_it_paints_them_all() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, grupo, folhas) = three_balls();
    crate::scene::sync_scene_and_birth(&mut sim, None, &[grupo], 0.0, &crate::scene::no_drawing());
    let rows = ph2d_panel_model3d::state::current().rows;
    let cor = rows
        .iter()
        .find(|r| r.swatch.is_some())
        .expect("⛔ um grupo escolhido não ofereceu a cor das formas debaixo dele");
    assert_eq!(cor.key, "field.dim.base_color");
    // ⭐ **E os outros números do material vêm com ela** — a secção é uma, não um controlo solto.
    //
    // ⛔⛔ **Eram `[3, 4]` até 14/09** (a rugosidade e o metal), e a re-numeração para a ordem da
    // nodedef transformou o `3` num CANAL DOBRADO da cor base — que não tem linha nenhuma. *Uma
    // lista de índices escrita à mão sobrevive a uma re-numeração sem erro de compilação*, e foi a
    // terceira vez nesta wave (a outra está no `polygon_rows_tests`).
    for k in [10u8, 5] {
        assert!(
            rows.iter()
                .any(|r| r.param == ph2d_field::Param::Material(k)),
            "o grupo ofereceu a cor e não o resto do material"
        );
    }

    // ── E o pedido pinta as TRÊS ──
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        // ⚠️ A ÂNCORA da cor BASE — as outras três têm gates próprios nos ficheiros irmãos.
        anchor: ph2d_field::Param::Material(1),
        entity: cor.entity,
        srgb: [255, 0, 128],
    });
    crate::scene::sync_scene_and_birth(&mut sim, None, &[grupo], 0.0, &crate::scene::no_drawing());
    assert_eq!(
        cores(&sim, &folhas),
        vec![[255, 0, 128]; 3],
        "⛔ pintar o grupo não pintou as três formas debaixo dele"
    );
}

/// ⭐⭐⭐ **UMA DIMENSÃO NUNCA ESPALHA** — o controlo que separa a lei nova de um esmagamento.
///
/// # ⛔⛔ Sem este gate, a lei do material leria como «a selecção inteira recebe tudo»
///
/// Largura, raio e posição são **daquela forma**: espalhá-los por uma selecção destrói o desenho. A
/// distinção não é arbitrária — *um material é a única coisa que um artista atribui a muitos
/// objectos de uma vez*, e é o `assign material to selection` de todo DCC.
///
/// **Mutação que deve sangrar:** apagar a guarda `param @ Param::Material(_)` do braço que espalha
/// (todo `SetParam` passaria a espalhar).
#[test]
fn a_dimension_never_spreads_across_the_selection() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, _grupo, folhas) = three_balls();
    let raio = |sim: &SimWorld, e: Entity| {
        ph2d_field_ecs::params_of(sim.world(), e)
            .into_iter()
            .find(|(p, _)| *p == ph2d_field::Param::Dim(0))
            .map(|(_, d)| d.value)
            .expect("uma esfera tem raio")
    };
    let antes: Vec<f32> = folhas.iter().map(|&f| raio(&sim, f)).collect();
    // As TRÊS escolhidas, e o pedido é para a primeira.
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: folhas[0].to_bits(),
        param: ph2d_field::Param::Dim(0),
        value: 0.33,
    });
    crate::scene::sync_scene_and_birth(&mut sim, None, &folhas, 0.0, &crate::scene::no_drawing());
    let depois: Vec<f32> = folhas.iter().map(|&f| raio(&sim, f)).collect();
    assert!(
        (depois[0] - 0.33).abs() < 1.0e-5,
        "o raio pedido não chegou à forma que o pediu"
    );
    assert_eq!(
        &depois[1..],
        &antes[1..],
        "⛔ uma DIMENSÃO espalhou-se pela selecção — isso esmaga o desenho das outras formas"
    );
}

/// ⭐⭐ **A NOTA DIZ SOBRE QUANTAS FORMAS, E SE ELAS DISCORDAM** — o sinal sem o qual espalhar mente.
///
/// # ⛔ *Espalhar sem sinal troca um sub-aplicar silencioso por um ESMAGAMENTO silencioso*
///
/// O controlo mostra o valor de **uma** forma. Sem a nota o artista lê *«estou a pintar esta»* e
/// pinta cinco — e quando as formas discordam, o valor mostrado é **falso** sobre as outras.
///
/// ⚠️ **E o aviso de divergência só aparece quando ela existe:** com as formas de acordo a amostra
/// descreve todas, e um *«diferem»* ali seria mentira ao contrário.
///
/// ⛔ **Uma forma só não leva nota nenhuma** — ela é o sujeito óbvio, e uma nota permanente sobre o
/// gesto mais comum do painel é ruído.
///
/// **Mutação que deve sangrar:** o `if difere` a ser sempre verdadeiro, ou o `alvos.len() < 2` a
/// deixar passar.
#[test]
fn the_note_says_how_many_shapes_and_whether_they_differ() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, grupo, folhas) = three_balls();
    let nota = |sim: &mut SimWorld, sel: &[Entity]| -> Option<String> {
        crate::scene::sync_scene_and_birth(sim, None, sel, 0.0, &crate::scene::no_drawing());
        ph2d_panel_model3d::state::current()
            .rows
            .iter()
            .find(|r| matches!(r.param, ph2d_field::Param::Material(_)))
            .and_then(|r| r.subject.clone())
    };
    // ⛔ UMA forma: sem nota.
    assert_eq!(
        nota(&mut sim, &[folhas[0]]),
        None,
        "uma forma só não precisa de nota — ela é o sujeito óbvio"
    );
    // ⭐ O grupo das três, todas de acordo: a nota conta e NÃO avisa.
    let n = nota(&mut sim, &[grupo]).expect("três formas pedem nota");
    assert!(n.contains('3'), "a nota não diz quantas formas: {n:?}");
    assert!(
        !n.contains("differ"),
        "as três estão de acordo e a nota diz que diferem: {n:?}"
    );
    // ⭐⭐ E com uma delas noutra cor, ela AVISA — é o único momento em que o valor mostrado é falso
    // sobre as outras.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[2],
        ph2d_field::Param::Material(2),
        0.0,
    )
    .expect("o verde");
    let n = nota(&mut sim, &[grupo]).expect("três formas pedem nota");
    assert!(
        n.contains("differ"),
        "as formas discordam e a nota não o diz — a amostra mostra a cor da primeira sobre três \
         cores diferentes: {n:?}"
    );
}

/// ⭐⭐⭐ **UM PEDIDO DE UM QUADRO ATRÁS PINTA SÓ A FORMA DELE** — a cerca do [`material_reach`].
///
/// # ⛔ O defeito que ela impede
///
/// O pedido traz o `entity` da **linha** que o produziu, e a selecção pode ter mudado entre o quadro
/// que a pintou e o que a drena (um clique no canvas, um desfazer). Sem a cerca, aquele pedido
/// pintaria a selecção **de agora** — e o artista veria formas que nunca escolheu mudar de cor.
///
/// *Um pedido carrega o sujeito que o produziu, e quem o executa confere-o contra o presente* — a
/// mesma família do id da amostra (`docs/Render3d/05` §12.3).
///
/// **Mutação que deve sangrar:** o `material_reach` a devolver o alcance sem conferir o `entity`.
#[test]
fn a_request_from_a_stale_selection_paints_only_its_own_shape() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, _grupo, folhas) = three_balls();
    // O pedido é da folha 0; a selecção de AGORA é a 1 e a 2 — a 0 não está nela.
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        // ⚠️ A ÂNCORA da cor BASE — as outras três têm gates próprios nos ficheiros irmãos.
        anchor: ph2d_field::Param::Material(1),
        entity: folhas[0].to_bits(),
        srgb: [255, 0, 128],
    });
    crate::scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[folhas[1], folhas[2]],
        0.0,
        &crate::scene::no_drawing(),
    );
    let c = cores(&sim, &folhas);
    assert_eq!(c[0], [255, 0, 128], "a forma que pediu não foi pintada");
    assert_ne!(
        c[1],
        [255, 0, 128],
        "⛔ um pedido velho pintou uma forma que a selecção de agora tem e ele nunca teve"
    );
    assert_eq!(c[1], c[2], "e a outra também não");
}

/// ⭐⭐⭐ **A FRONTEIRA ENTRE DUAS CORES NÃO É UM DEGRAU** — a costura, medida no PIXEL.
///
/// # ⚠️ A régua, e as TRÊS vezes que ela se corrigiu antes do produto
///
/// 1. **Ela media duas coisas.** A 1.ª redacção contava saltos grandes entre vizinhos da peça — e
///    numa união dura o maior é o **vinco**, onde a normal muda a pique e a luz dá um degrau
///    **legítimo**. Lia `236` bytes e atribuía-os à cor. ⇒ a régua passou a ser a **DIFERENÇA contra
///    um controlo** (a mesma peça com as duas folhas do mesmo material).
/// 2. **Ela media a população errada.** No pior pixel de uma união dura os dois pontos estão a
///    **`16,3` px** um do outro em 3D: eles **não são vizinhos na superfície** — a peça salta em
///    profundidade. ⇒ só entram vizinhos que o são também na superfície.
/// 3. **E o filtro da bola era o da marcha.** Ver
///    [`ph2d_field_eval::owners::Owners::mix_at`] — sem a margem da LARGURA a lei era muda numa
///    união dura, e a régua mostrava-a a não mexer.
///
/// # A barra
///
/// Medido com a lei desligada: a cor acrescenta `+166` (dura) e `+191` (suave). Com ela: `+34` e
/// `+95`. ⇒ a barra é **`120`**, que mora no vale entre as duas populações e deixa margem ao lado
/// suave, que é o pior.
///
/// ⚠️ **Não é um gate de relógio** — ele conta bytes de uma imagem determinística, logo não flaka sob
/// carga.
#[test]
fn the_boundary_between_two_colours_is_not_a_step() {
    for (nome, blend, barra) in [
        ("dura", ph2d_field::Blend::Sharp, 120),
        ("suave", ph2d_field::Blend::Exact { radius: 0.25 }, 120),
    ] {
        let controlo = colour_boundary_step(blend, [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let duas = colour_boundary_step(blend, [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]);
        let acrescenta = duas - controlo;
        // ⛔ **O PISO do controlo**: sem ele, uma cadeia que devolvesse uma imagem chapada leria
        // `0 − 0 = 0` e passaria a medir nada.
        assert!(
            controlo > 10,
            "a mistura {nome}: o controlo leu {controlo} — a imagem está chapada e este gate não \
             está a medir nada"
        );
        assert!(
            acrescenta < barra,
            "⛔ a mistura {nome}: a COR acrescenta {acrescenta} bytes ao degrau (controlo \
             {controlo}, duas cores {duas}) — a fronteira entre dois materiais voltou a ser uma \
             escada. A barra é {barra}, e sem a lei isto lê `+166`/`+191`."
        );
    }
}

/// ⏱️ **SONDA — a fronteira de COR entre duas formas**, que a §11 tornou alcançável.
///
/// # ⚠️ Porque ela não é a mesma coisa que a silhueta
///
/// O anti-serrilhado do traçado corre nas **bordas** — pixels em que **algumas** sub-amostras acertam
/// a peça e outras não (`Gbuffer::edges`). Uma fronteira de **cor** no meio da peça não é nenhuma
/// dessas: ali **todas** as sub-amostras acertam, logo não há registo de borda nenhum.
///
/// # ⛔⛔ E a 1.ª redacção desta régua media DUAS coisas
///
/// Ela contava *«saltos grandes entre pixels vizinhos da peça»* — e numa união **dura** o maior
/// desses saltos é o **vinco**, onde a normal muda a pique e a luz dá um degrau **legítimo**. A
/// régua lia `236` bytes e atribuía-os à cor; o número era quase todo sombreamento, e por isso ela
/// não se mexeu quando a cor foi curada.
///
/// ⇒ a régua é a **DIFERENÇA contra um controlo**: a mesma peça, a mesma luz, com as duas folhas do
/// **mesmo** material. O que sobra depois de subtrair o controlo é o degrau que a COR acrescenta.
/// *Uma régua que mede duas coisas não mede nenhuma.*
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_the_colour_boundary_between_two_shapes() {
    println!("mistura ·  cores iguais (controlo) ·  cores diferentes ·  o que a COR acrescenta");
    for (nome, blend) in [
        ("dura ", ph2d_field::Blend::Sharp),
        ("suave", ph2d_field::Blend::Exact { radius: 0.25 }),
    ] {
        let controlo = colour_boundary_step(blend, [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let duas = colour_boundary_step(blend, [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]);
        println!(
            "{nome}   · {controlo:24} · {duas:17} · {:+}",
            duas - controlo
        );
    }
}

/// O maior salto de cor entre vizinhos **do meio da peça** — a régua que a sonda acima usa duas
/// vezes, com e sem a cor a mudar.
fn colour_boundary_step(blend: ph2d_field::Blend, cores: [[f32; 3]; 2]) -> i32 {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    let (w, h) = (640usize, 360usize);
    let cam = Orbit::default();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let bola = |x: f32| ph2d_field::Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.35 }),
        mods: Vec::new(),
        verb: None,
    };
    let doc = FieldDoc::new(
        vec![
            bola(-0.22),
            bola(0.22),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(blend),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("duas bolas");
    let g = trace(&doc, &reg, &cam, w as u32, h as u32);
    let postas: Vec<FieldDoc> = (0..2)
        .map(|i| FieldDoc::new(vec![doc.nodes()[i].clone()], NodeId(0)).expect("a folha"))
        .collect();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, w.min(h) as f32),
    );
    let so: Vec<ph2d_material::Surface> = cores
        .into_iter()
        .map(|base_color| {
            crate::materials::surface_of(ph2d_field_ecs::FieldMaterial {
                base_color,
                ..ph2d_field_ecs::FieldMaterial::default()
            })
        })
        .collect();
    let px = shade_render(
        &g,
        &cam,
        &ph2d_field_render::Surfaces {
            all: &so,
            owners: Some(&owners),
        },
        &Lighting {
            lamps: &crate::render_light::lamps(&ph2d_light::LightRig::default()),
            points: &[],
            sky: &crate::render_light::StudioSky,
            shadows: None,
        },
        crate::shading::OPENING_LOOK,
        [0, 0, 0, 0],
    );
    let c = px.as_chunks::<4>().0;
    let (mut pior, mut onde) = (0i32, (0usize, 0usize));
    for y in 0..h {
        for x in 0..w - 1 {
            let (i, j) = (y * w + x, y * w + x + 1);
            if !g.hit[i] || !g.hit[j] {
                continue;
            }
            // ⛔⛔ **SÓ ONDE A SUPERFÍCIE É CONTÍNUA** — a terceira correcção desta régua.
            //
            // No pior pixel de uma união dura os dois pontos estão a **16,3 px** um do outro em 3D:
            // eles **não são vizinhos na superfície**. Ali a peça salta em profundidade (um vinco
            // visto de raspão), e a diferença de cor entre os dois é tão legítima como a de dois
            // pixels em lados opostos da silhueta — *nenhuma mistura por PONTO pode, ou deve, curar
            // isso*. O que a cura desta wave endireita é a fronteira **sobre** a superfície.
            let salto = {
                let (a, b) = (g.point[i], g.point[j]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            };
            if salto > 2.0 * (2.0 * cam.half_extent / w.min(h) as f32) {
                continue;
            }
            let d = (0..3)
                .map(|k| i32::from(c[j][k]) - i32::from(c[i][k]))
                .max_by_key(|v| v.abs())
                .unwrap_or(0)
                .abs();
            if d > pior {
                pior = d;
                onde = (x, y);
            }
        }
    }
    if std::env::var("PH2D_BOUNDARY_WHERE").is_ok() {
        let (x, y) = onde;
        let (i, j) = (y * w + x, y * w + x + 1);
        let largura = 2.0 * cam.half_extent / w.min(h) as f32;
        println!(
            "      pior em ({x},{y}) · {:?} → {:?} · mix esq {:?} · mix dir {:?} · px mundo {largura:.5} \
             · SALTO 3D {:.5} ({:.1} px)",
            &c[i][..3],
            &c[j][..3],
            owners.mix_at(g.point[i], largura),
            owners.mix_at(g.point[j], largura),
            {
                let (a, b) = (g.point[i], g.point[j]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            },
            {
                let (a, b) = (g.point[i], g.point[j]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
                    / largura
            },
        );
    }
    pior
}
