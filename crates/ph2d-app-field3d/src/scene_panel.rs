//! ⭐ **A ponte com o PAINEL** — o retrato que a peça publica, e o que cada controle oferece.
//!
//! ⚠️ É um **módulo-filho** de [`super`] e não um irmão de topo: ele lê o mundo pelas mesmas portas
//! do pai e os caminhos antigos continuam a valer pelo re-export. O corte é por **assunto** — o
//! irmão possui a cena (nascer, apagar, cozinhar, apontar) e este responde *"o que o painel mostra
//! e o que ele oferece"*.
//!
//! ⭐ **É aqui, num sítio só, que uma faixa aberta se fecha.** O documento diz a *forma* do que cada
//! grandeza admite ([`ph2d_field::Span`]); esta é a única metade que sabe o enquadramento, e é ela
//! que escreve as duas pontas de cada linha. Espalhar isto era o que fazia toda linha começar em
//! zero — e uma posição negativa ser indigitável, em silêncio.

use super::acts::{ACT_ISOLATE, ISOLATE_BADGE, LINK_BADGE, acts_for};
use super::*;

/// **A ponte com o painel**: publica o retrato da peça.
///
/// ⭐ **A ordem é load-bearing.** Drenar ANTES de publicar é o que faz a edição aparecer no mesmo
/// quadro: se o retrato saísse primeiro, o painel pintaria o valor antigo por um quadro e o
/// controle daria um salto para trás debaixo do dedo — o sintoma clássico de um espelho publicado
/// cedo demais.
pub fn publish_snapshot(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    selection: &[bevy_ecs::entity::Entity],
    view_span: f32,
    ms: f32,
) {
    let all = ph2d_field_ecs::walk(world, root);
    let mut rows = param_rows(world, selection, view_span);
    // ⭐⭐⭐ **E AS FILEIRAS DO ESTILO DA CENA** (`docs/Render3d/03`, a `W8`) — no FIM, porque elas
    // não são do objecto escolhido: são da cena, e um artista que clicou numa forma lê primeiro os
    // números dela.
    //
    // ⚠️ **Só no modo Render**, e a ausência é a lei — ver o [`crate::estilo::rows`]: no matcap o
    // estilo não corre, e *uma affordance que não pode ser honrada é pior do que nenhuma*.
    let no_render = matches!(
        with_smoke(|s| s.vp().shading),
        Some(crate::shading::Shading::Render)
    );
    rows.extend(crate::estilo::rows(
        with_smoke(|s| s.style).unwrap_or_default(),
        no_render,
    ));
    // ⭐⭐⭐ **E AS FILEIRAS DO BRILHO** (`docs/Render3d/12`, a `W7`) — a seguir às do estilo, que é
    // a ordem do pipeline: o estilo é por pixel, o brilho é o passe que vem depois dele.
    //
    // ✅ **A terceira resposta ERA `no_dispositivo` e foi APAGADA em 19/09** — o brilho vivia só na
    // cauda do sombreamento de CPU e este módulo pinta no dispositivo por omissão, logo as fileiras
    // apareciam à vista e apagadas a dizê-lo. Desde que o gémeo em WGSL existe
    // ([`ph2d_bloom::wgsl`]) elas acendem nos dois caminhos.
    rows.extend(crate::brilho_painel::rows(
        with_smoke(|s| s.bloom).unwrap_or_default(),
        no_render,
    ));
    let rows = rows;
    // ⚠️ A lista de verbos é **derivada de `Mode::ALL`**, que é a fonte da contagem. O painel não
    // conhece o enum — acrescentar um verbo lá faz o seletor seguir sem uma linha de mudança.
    let (active, frame, mut subtracts) =
        with_smoke(|s| (s.gizmo_mode, s.gizmo_frame, s.lasso_subtracts)).unwrap_or_default();
    // ⭐⭐⭐ **UM VERBO QUE NÃO FAZ NADA NÃO SE OFERECE** (`docs/Render3d/05` §28) — e a forma é a
    // que a fileira do laço, dez linhas abaixo, já usa: o chip some **e o modo volta atrás**.
    let oferecidos = offered_verbs(world, selection);
    let mut active = active;
    if !oferecidos.contains(&active) {
        active = crate::gizmo::Mode::Move;
        with_smoke(|s| s.gizmo_mode = active);
    }
    let modes = oferecidos
        .iter()
        .map(|m| ph2d_panel_model3d::ModeChip {
            key: m.key(),
            active: *m == active,
        })
        .collect();
    // ⭐⭐ **O modo do LAÇO** (W112) — dois chips, e o aceso é o que está em vigor.
    //
    // ⚠️ **A ordem é a lei**, e não a lista: `0` soma, `1` tira. Ela vive em UM sítio — aqui e no
    // braço do intent — porque o painel viaja com a POSIÇÃO, nunca com o enum.
    //
    // ⭐⭐⭐ **Ela só aparece com DUAS ou mais peças escolhidas**, e o preço é medido: uma fileira
    // permanente custa `+66 px` de conteúdo, **`+11,9 %`** do painel cheio, para um gesto que só
    // faz sentido contra um CONJUNTO. ⚠️ E quando ela some, o modo **volta a somar** — senão
    // ficaria armado e invisível, que é o modo de falha que esta casa mede desde 30/08.
    let oferece = selection.len() >= 2;
    if !oferece && subtracts {
        with_smoke(|s| s.lasso_subtracts = false);
        subtracts = false;
    }
    let selects = if !oferece {
        Vec::new()
    } else {
        [
            ("panel.model3d.select.add", false),
            ("panel.model3d.select.subtract", true),
        ]
        .into_iter()
        .map(|(key, sub)| ph2d_panel_model3d::ModeChip {
            key,
            active: sub == subtracts,
        })
        .collect()
    };
    let frames = crate::gizmo::Frame::ALL
        .iter()
        .map(|f| ph2d_panel_model3d::ModeChip {
            key: f.key(),
            active: *f == frame,
        })
        .collect();
    // ⚠️ **O retrato deixou de carregar a disponibilidade das formas** (W100): quem a lê é a paleta,
    // no instante em que abre. Publicá-la aqui seria um espelho a envelhecer entre o quadro em que
    // o painel pinta e o quadro em que o artista escolhe.
    let adds = adds_for();
    let ops = ops_for(world, selection);
    let (verbs, verb_subject) = super::verb::verbs_for(world, selection);
    let characters = super::verb::characters_for(world, selection);
    let mods = mods_for(world, selection);
    // ⚠️ **Derivado de `ExportLevel::ALL`**, que é a fonte da contagem — a mesma lei do `Mode::ALL`
    // e do `SHAPES`. E sem `active` nenhum: são ações, não um modo.
    let exports = crate::export::ExportLevel::ALL
        .iter()
        .map(|l| ph2d_panel_model3d::ModeChip {
            key: l.key(),
            active: false,
        })
        .collect();
    // ⚠️ Vazio quando o escolhido **não se destaca da peça**, pela mesma razão da fileira de
    // operações: um controle que aparece e não faz nada é pior do que um que não aparece.
    //
    // ⭐ **A RAIZ era o caso que a lia errado** (W34): `selection.is_empty()` deixava a fileira
    // aparecer com a peça inteira escolhida, e ali **`duplicate` e `remove` recusam os dois** — por
    // decisão escrita, não por acaso (a raiz *é* a peça). Dois botões pintados e mudos na linha de
    // topo da Hierarquia. ⚠️ Quem responde é [`ph2d_field_ecs::can_detach`], a mesma função que os
    // dois gestos consomem: *a recusa era uma decisão; a affordance que a ignorava era um defeito.*
    // ⭐ O `active` do isolamento diz o ESTADO: aceso quer dizer *"é este que estás a ver"*.
    let isolated = crate::smoke::isolated();
    let acts: Vec<_> = acts_for(world, selection)
        .into_iter()
        .map(|key| ph2d_panel_model3d::ModeChip {
            key,
            active: key == ACT_ISOLATE
                && isolated.is_some()
                && isolated == selection.first().map(|e| e.to_bits()),
        })
        .collect();
    // ⭐⭐ **O selo do vínculo sai desta MESMA travessia** (W57) — ver [`link_badges`].
    {
        // ⭐⭐⭐ **O VERBO é o selo de base desta lista** (W97): ordem + verbo **são** a receita, e é
        // isto que torna uma peça de cinco formas legível sem cinco cliques.
        //
        // ⛔ **E ele GANHA do `LNK`, que é uma perda deliberada.** O campo do selo é **um por
        // linha**: o `LNK` responde *«de onde veio esta forma?»*, que se pergunta uma vez e tem
        // gesto no painel do escolhido; o verbo responde *«o que ela FAZ à peça?»*, que é o que o
        // olho lê ao percorrer a lista, sempre. ⚠️ **O gatilho da cura está nomeado:** no dia em que
        // a linha da Hierarquia pintar **dois** selos, os dois cabem — e é aí que isto se revê.
        let mut m: std::collections::BTreeMap<u64, &'static str> = all
            .iter()
            .filter(|(e, _)| {
                world
                    .get::<ph2d_field_ecs::FieldProfileSource>(*e)
                    .is_some()
            })
            .map(|(e, _)| (e.to_bits(), LINK_BADGE.tr()))
            .collect();
        for (e, _) in &all {
            if let Some(badge) = super::verb::verb_badge(world, *e) {
                // ⚠️ **A BASE cede ao `LNK`, e só ela** — ver [`super::verb::BASE_BADGE`]. `BSE`
                // repete o que a POSIÇÃO já diz; o vínculo não é derivável de nada na tela.
                if badge == super::verb::BASE_BADGE.tr() && m.contains_key(&e.to_bits()) {
                    continue;
                }
                m.insert(e.to_bits(), badge);
            }
        }
        // ⭐⭐⭐ **E O SELO DO ISOLAMENTO** (2026-08-25) — ver [`ISOLATE_BADGE`] para a precedência.
        //
        // ⚠️ **A pergunta é feita à travessia, não ao mundo**: um isolamento pendurado numa entidade
        // morta (o undo respawna tudo com bits novos) selaria uma linha que já não é aquela — é a
        // mesma cerca que o [`isolated_name`] documenta, e por isso as duas leem a MESMA lista.
        if let Some(bits) = crate::smoke::isolated()
            && all.iter().any(|(e, _)| e.to_bits() == bits)
        {
            m.insert(bits, ISOLATE_BADGE.tr());
        }
        super::acts::publish_badges(m);
    }
    ph2d_panel_model3d::publish(ph2d_panel_model3d::ModelSnapshot {
        modes,
        frames,
        selects,
        adds,
        ops,
        verbs,
        verb_subject,
        characters,
        mods,
        exports,
        acts,
        // ⭐⭐ **AS VISTAS** (W47) — derivadas de `Standard::ALL`, e o `active` derivado da
        // ORIENTAÇÃO da câmera, nunca de um modo guardado (ver `views::named_view`).
        views: views_now(),
        camera: camera_now(),
        rows,
        // ⭐ **O isolamento diz-se sozinho** (W44), e a pergunta é feita ao MUNDO, não à seleção:
        // um estado da vista anunciado através de um controle da seleção some quando se escolhe
        // outra coisa — que é exactamente quando o artista precisa de o ler.
        isolated: isolated_name(world, &all),
        node_count: all.len(),
        last_trace_ms: ms,
        // ⭐ **A face do pulldown da área** — o rótulo NU da vista (`Front`, `User`), derivado da
        // câmera. ⚠️ É a chave `viewport.*` e não a `panel.*`: a do painel traz o atalho entre
        // parênteses (`Front (1)`), e um `(1)` dentro de um chip de 36 px é ruído — o atalho vive
        // nas linhas do menu que ele abre.
        view_label: with_smoke(|s| crate::views::label_key(&s.vp().cam))
            .unwrap_or("viewport.model3d.view.user"),
        // ⭐⭐⭐ **O SOMBREAMENTO** (`docs/Render3d/05`) — o modo do viewport ACTIVO e o olhar da
        // CENA, com o aceso derivado do estado, nunca de um botão guardado.
        shadings: crate::shading::shading_chips(with_smoke(|s| s.vp().shading).unwrap_or_default()),
        looks: crate::shading::look_chips(with_smoke(|s| s.look).unwrap_or_default()),
        exposures: crate::shading::exposure_chips(with_smoke(|s| s.look).unwrap_or_default()),
        shading_label: with_smoke(|s| s.vp().shading.key()).unwrap_or(""),
    });
}

/// ⭐⭐⭐ **Os números do objecto escolhido** — ver [`rows`].
#[path = "scene_panel_rows.rs"]
mod rows;
pub use rows::param_rows;

/// ⭐ **Os modificadores oferecidos, e quais o nó já tem** — interruptores, não ações.
///
/// ⚠️ **A lista é derivada de [`ph2d_field::UnaryKind::ALL`]**, que é a fonte da contagem: um
/// modificador novo entra lá e o painel segue sem uma linha de mudança. É a mesma lei do `Mode::ALL`
/// e do `SHAPES`.
///
/// ⚠️ Vazio sem seleção — um interruptor sem nó para ligar não tem o que dizer.
pub fn mods_for(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
) -> Vec<ph2d_panel_model3d::ModeChip> {
    let Some(&one) = selection.first() else {
        return Vec::new();
    };
    // ⭐ **Uma escultura não aceita modificadores** (W25) — e a fileira dela **não é pintada**.
    //
    // ⚠️ É a mesma lei que a fileira de operações já segue: *um controle que aparece e não faz nada
    // é pior do que um que não aparece*. Antes desta linha, clicar em `Shell` com uma escultura
    // selecionada escrevia um documento que o cozimento recusa, e a peça **inteira** sumia da tela
    // sem uma palavra.
    let shape = world.get::<FieldNode>(one).map(|n| &n.shape);
    match shape {
        None | Some(ph2d_field::NodeShape::Sampled { .. }) => return Vec::new(),
        Some(_) => {}
    }
    let have = ph2d_field_ecs::mods_of(world, one);
    ph2d_field::UnaryKind::ALL
        .iter()
        .map(|k| ph2d_panel_model3d::ModeChip {
            key: k.key(),
            // ⭐ **Aceso = o nó JÁ TEM um daquela natureza.** É o que faz o botão dizer o estado em
            // vez de só disparar — e é a diferença entre um interruptor e um botão que empilha
            // cascas sem o artista perceber.
            active: have.iter().any(|u| u.kind() == *k),
        })
        .collect()
}

/// ⭐ **As seis vistas nomeadas**, com a atual acesa (W47).
///
/// ⚠️ Sempre oferecidas: olhar a peça de frente não depende de nada estar escolhido. E a lista é
/// **derivada de `Standard::ALL`** — a fonte da contagem —, como a dos verbos do gizmo.
fn views_now() -> Vec<ph2d_panel_model3d::ModeChip> {
    let here = with_smoke(|s| crate::views::named_view(&s.vp().cam)).flatten();
    crate::views::Standard::ALL
        .iter()
        .map(|v| ph2d_panel_model3d::ModeChip {
            key: v.key(),
            active: here == Some(*v),
        })
        .collect()
}

/// ⭐ **A lente e o enquadrar** (W47) — os dois gestos de câmera que não são uma vista.
///
/// ⚠️ **A lente é um ESTADO** (o `active` diz que está na paralela) e o enquadrar é uma **ação**
/// (nunca acende). Misturá-los na mesma fileira é deliberado: a pergunta que ela responde é *"como
/// estou a olhar?"*, e as duas são resposta a ela — a mesma decisão que a fileira de ações já tomou
/// com o *Isolate*.
fn camera_now() -> Vec<ph2d_panel_model3d::ModeChip> {
    let ortho =
        with_smoke(|s| matches!(s.vp().cam.lens, ph2d_field_render::Lens::Ortho)).unwrap_or(false);
    let quad =
        with_smoke(|s| matches!(s.split, crate::layout::Split::Quad { .. })).unwrap_or(false);
    vec![
        ph2d_panel_model3d::ModeChip {
            key: CAMERA_ACTS[ORTHO_SLOT],
            active: ortho,
        },
        ph2d_panel_model3d::ModeChip {
            key: CAMERA_ACTS[FRAME_SLOT],
            active: false,
        },
        ph2d_panel_model3d::ModeChip {
            key: CAMERA_ACTS[QUAD_SLOT],
            active: quad,
        },
    ]
}

/// Os gestos de câmera, na ordem do seletor.
pub const CAMERA_ACTS: [&str; 3] = [
    "panel.model3d.camera.ortho",
    "panel.model3d.camera.frame",
    "panel.model3d.camera.quad",
];

/// A **lente** — interruptor. ⚠️ Derivados, nunca números à mão: um gesto novo no meio da lista
/// mudaria o índice e o botão passaria a fazer outra coisa, sem erro nenhum.
pub const ORTHO_SLOT: usize = 0;
/// O **enquadrar** — ação.
pub const FRAME_SLOT: usize = 1;
/// ⭐⭐ A **divisão do canvas** (W90) — interruptor, como a lente.
///
/// ⚠️ Ele mora nesta fileira e não numa nova porque a pergunta dela é *«como estou a olhar?»* — e
/// *«de quantos sítios ao mesmo tempo»* é uma resposta a essa, exactamente como a lente.
pub const QUAD_SLOT: usize = 2;

/// ⭐ **O NOME do nó isolado** (W44) — `None` quando se vê a peça inteira.
///
/// ⚠️ **Ela confirma que o nó ainda existe**, e é por isso que recebe a lista da caminhada em vez de
/// perguntar ao mundo pelos bits: o isolamento guarda `Entity::to_bits()`, e um undo respawna tudo
/// com bits novos. Um isolamento pendurado numa entidade morta anunciaria um nome que já não está
/// na Hierarquia — ou, pior, o de outro nó que herdou os bits. O cozimento já larga o alvo morto
/// (`cook_root`); esta metade garante que a **voz** larga com ele.
fn isolated_name(
    world: &bevy_ecs::world::World,
    all: &[(bevy_ecs::entity::Entity, u8)],
) -> Option<String> {
    let bits = crate::smoke::isolated()?;
    let e = all.iter().map(|(e, _)| *e).find(|e| e.to_bits() == bits)?;
    Some(
        world
            .get::<ph2d_ecs::Name>(e)
            .map_or_else(|| String::from("?"), |n| n.0.to_string()),
    )
}

/// As três booleanas, na ordem do seletor.
pub const OPS: [&str; 3] = [
    "panel.model3d.op.union",
    "panel.model3d.op.subtract",
    "panel.model3d.op.intersect",
];

/// ⭐ **O tamanho de uma forma nova, DERIVADO do enquadramento.**
///
/// ⚠️ A condição que o fixa é a única que importa: uma forma nova tem de ser **vista**. Um tamanho
/// fixo em unidades de mundo nasce invisível numa peça grande e tapa a janela numa pequena — e nos
/// dois casos o artista conclui que o botão não funcionou. Um quarto da meia-altura do quadro põe-na
/// a metade da altura da tela, que é onde se vê o que ela é.
pub fn new_shape_size(half_extent: f32) -> f32 {
    (half_extent * 0.25).max(f32::MIN_POSITIVE)
}

// ⚠️ **O `shape_at` mudou-se para [`crate::shapes`]** (W100), com o catálogo inteiro: cada
// forma passou a trazer o **próprio construtor**, então não há mais um `match slot` para viver aqui.
// *Um `match` posicional sobrevive a acrescentar no fim e parte-se em silêncio ao inserir no meio* —
// e o que vem por aí é uma lista de 60.

pub fn op_at(slot: usize) -> Option<Op> {
    Some(match slot {
        0 => Op::Union(Blend::Sharp),
        1 => Op::Difference(Blend::Sharp),
        2 => Op::Intersection(Blend::Sharp),
        _ => return None,
    })
}

/// ⭐⭐⭐ **A PORTA DE CRIAR — um botão** (W100), que abre a paleta de formas.
///
/// # ⚠️ Ela era uma fileira com o catálogo inteiro, e não podia continuar a ser
///
/// O `paint_chips` corta em `MAX_MODES` = **8**, e o catálogo já tinha **8**: a forma nº 9 sairia da
/// tela **sem uma palavra**. E são 47 do catálogo vetorial mais 15 sólidas na fila (doc 08). O que
/// resolve isto já existe nesta casa e já shipou três vezes — a paleta genérica do
/// `ph2d-editor-core` —, e é para lá que a lista foi ([`crate::shape_palette`]).
///
/// ⭐ **A disponibilidade continua a ser respeitada, e ficou MELHOR:** as três formas que dependem
/// da seleção sumiam da fileira, e com isso o artista não podia saber que existem. Na paleta elas
/// aparecem com a **razão** ao lado.
///
/// ⚠️ **Devolve uma FILEIRA de um chip** e não um `bool` — ver [`ph2d_panel_model3d::ModelSnapshot`]
/// `adds`: o `paint_chips` já sabe desenhar, medir e registar; um botão avulso seria um caminho de
/// pintura novo neste painel.
pub fn adds_for() -> Vec<ph2d_panel_model3d::ModeChip> {
    vec![
        ph2d_panel_model3d::ModeChip {
            key: "panel.model3d.add.open",
            active: false,
        },
        // ⭐⭐⭐ **A LUZ** (ordem do dono, 14/09) — e ela está AQUI e não na paleta de formas, por
        // recusa de um gate: o `each_family_has_its_own_title_and_colour` exige **uma tinta por
        // família**, e há exactamente sete `NodeCat*`, todas tomadas. Uma oitava família pedia um
        // token novo, que é decisão de design (§7) e não desta linha.
        //
        // ⭐ E a recusa aponta para a leitura certa: *aquela paleta é de FORMAS*, e uma lâmpada não
        // é uma forma — ela nem sequer entra na árvore da peça.
        ph2d_panel_model3d::ModeChip {
            key: "panel.model3d.add.light",
            active: false,
        },
    ]
}

/// ⭐ **Quais operações fazem sentido AGORA** — e vazio quando nenhuma faz.
///
/// ⚠️ Publicar a fileira sempre daria três botões que às vezes não fazem nada, que é a affordance
/// que mente. Ela aparece em três casos, e em cada um quer dizer uma coisa precisa:
///
/// | Selecionado | O que os botões fazem | O «ativo» |
/// |---|---|---|
/// | uma **operação** | trocam-na (união vira subtração) | a operação que ela é |
/// | uma **forma sozinha** | **cria um grupo** com ela dentro (W31) | nenhum |
/// | **dois ou mais irmãos** | embrulham-nos numa operação nova | nenhum |
///
/// # ⚠️ A segunda linha faltava, e o gesto ficou inalcançável (W34)
///
/// A W31 ensinou o **tratador** a aceitar uma forma sozinha — a resposta ao *"ainda não temos como
/// criar novos grupos"* do Enio — e esta função continuou a exigir **dois irmãos**. Os três botões
/// nunca eram pintados nesse caso, então o gesto existia e ninguém lhe chegava; os gates da W31
/// empurravam a intenção diretamente e por isso não notaram. *Empurrar a intenção prova o tratador,
/// nunca a alcançabilidade.*
///
/// ⭐ **A cura estrutural é não ter aqui uma segunda cópia da regra:** quem responde *"estes nós
/// embrulham-se?"* é [`ph2d_field_ecs::can_wrap`], a mesma função que o `wrap_in_op` consome. Os
/// dois lados divergirem outra vez passa a exigir mudar a lei única. O gate-mãe é
/// `the_panel_offers_an_operation_exactly_when_the_gesture_does_something`.
pub fn ops_for(
    world: &bevy_ecs::world::World,
    selected: &[bevy_ecs::entity::Entity],
) -> Vec<ph2d_panel_model3d::ModeChip> {
    let chips = |active: Option<usize>| -> Vec<ph2d_panel_model3d::ModeChip> {
        OPS.iter()
            .enumerate()
            .map(|(i, key)| ph2d_panel_model3d::ModeChip {
                key,
                active: active == Some(i),
            })
            .collect()
    };
    if let [one] = selected
        && let Some(FieldNode {
            shape: NodeShape::Combine(op),
        }) = world.get::<FieldNode>(*one)
    {
        return chips(Some(match op {
            Op::Union(_) => 0,
            Op::Difference(_) => 1,
            Op::Intersection(_) => 2,
        }));
    }
    if ph2d_field_ecs::can_wrap(world, selected) {
        chips(None)
    } else {
        Vec::new()
    }
}

/// ⭐⭐⭐ **O MATERIAL DA SELECÇÃO** — quantas formas um pedido de material pinta, e como se diz.
/// Vive no irmão, por assunto e pelo tecto de LOC. Ver [`field3d_scene_panel_material`](self::material).
#[path = "scene_panel_material.rs"]
mod material;
use material::material_for_the_selection;
pub(crate) use material::material_reach;

/// ⭐⭐⭐ **Que verbos do gizmo têm sujeito nesta selecção** (`docs/Render3d/05` §28).
///
/// **Mover** vale sempre: tudo o que o gizmo agarra tem um *onde*. **Rodar** e **escalar** precisam
/// de uma coisa com ORIENTAÇÃO e TAMANHO, e uma lâmpada de ponto não tem nenhuma das duas — com uma
/// luz escolhida, aqueles dois verbos eram **inertes**: as alças desenhavam-se e o arrasto não movia
/// número nenhum.
///
/// ⚠️⚠️ **A nota do §25.8 dizia que a cura era um campo novo no [`crate::gizmo::Anchor`], e estava
/// errada.** Ela raciocinou a partir de *«o verbo é estado de VISTA, um por viewport»* e concluiu
/// que restringi-lo por-objecto teria de viajar no gizmo. Não tem: **o que se restringe não é o
/// gesto, é a OFERTA** — se o chip não existe, o verbo nunca fica activo, e o gizmo não precisa de
/// saber que esta pergunta existe. *Uma cura desenhada a partir de onde um valor MORA, em vez de a
/// partir de quem o OFERECE, compra o refactor errado.*
///
/// ⭐ E o molde é o da fileira do laço, no mesmo ficheiro: o chip some **e o modo volta atrás** —
/// senão ele ficaria *«armado e invisível»*, que é o modo de falha que esta casa mede desde 30/08.
///
/// ⚠️ **A regra é sobre a selecção INTEIRA, não sobre a primária:** com uma luz e uma forma
/// escolhidas, rodar tem sujeito (a forma) e o gizmo pousa no meio das duas — tirar o verbo ali
/// seria tirar um gesto legítimo por causa de um acompanhante.
///
/// ⛔⛔ **E ela é UMA lista, lida pelos DOIS lados.** O consumidor do clique fazia
/// `Mode::ALL.get(slot)` e o painel publicava `Mode::ALL` inteiro, logo as posições casavam por
/// construção. Ao filtrar a oferta isso deixa de ser verdade, e **hoje ele ainda acertaria por
/// ACIDENTE** — o `Move` é o primeiro, e é o único que sobra. *Uma correspondência que só se
/// mantém enquanto o primeiro elemento não muda é a «segunda contagem» que o
/// `ph2d_panel_model3d::area_bar` já proíbe por escrito.* ⇒ os dois lados chamam
/// [`offered_verbs`], e o `slot` indexa o que foi de facto oferecido.
#[must_use]
pub fn offered_verbs(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
) -> Vec<crate::gizmo::Mode> {
    crate::gizmo::Mode::ALL
        .iter()
        .copied()
        .filter(|m| verb_applies(world, selection, *m))
        .collect()
}

#[must_use]
pub fn verb_applies(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
    mode: crate::gizmo::Mode,
) -> bool {
    if mode == crate::gizmo::Mode::Move {
        return true;
    }
    // ⚠️ **Selecção vazia oferece tudo**, e não é indulgência: sem sujeito não há gizmo nenhum, e
    // esconder chips com o canvas vazio faria a fileira piscar ao clicar fora.
    if selection.is_empty() {
        return true;
    }
    selection
        .iter()
        .any(|e| world.get::<ph2d_field_ecs::FieldNode>(*e).is_some())
}
