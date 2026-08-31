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
pub(crate) fn publish_snapshot(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    selection: &[bevy_ecs::entity::Entity],
    view_span: f32,
    ms: f32,
) {
    let all = ph2d_field_ecs::walk(world, root);
    let rows = param_rows(world, selection.first().copied(), view_span);
    // ⚠️ A lista de verbos é **derivada de `Mode::ALL`**, que é a fonte da contagem. O painel não
    // conhece o enum — acrescentar um verbo lá faz o seletor seguir sem uma linha de mudança.
    let (active, frame) = with_smoke(|s| (s.gizmo_mode, s.gizmo_frame)).unwrap_or_default();
    let modes = crate::field3d_gizmo::Mode::ALL
        .iter()
        .map(|m| ph2d_panel_model3d::ModeChip {
            key: m.key(),
            active: *m == active,
        })
        .collect();
    let frames = crate::field3d_gizmo::Frame::ALL
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
    let exports = crate::field3d_export::ExportLevel::ALL
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
    let isolated = crate::field3d_smoke::isolated();
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
            .map(|(e, _)| (e.to_bits(), LINK_BADGE))
            .collect();
        for (e, _) in &all {
            if let Some(badge) = super::verb::verb_badge(world, *e) {
                // ⚠️ **A BASE cede ao `LNK`, e só ela** — ver [`super::verb::BASE_BADGE`]. `BSE`
                // repete o que a POSIÇÃO já diz; o vínculo não é derivável de nada na tela.
                if badge == super::verb::BASE_BADGE && m.contains_key(&e.to_bits()) {
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
        if let Some(bits) = crate::field3d_smoke::isolated()
            && all.iter().any(|(e, _)| e.to_bits() == bits)
        {
            m.insert(bits, ISOLATE_BADGE);
        }
        super::acts::publish_badges(m);
    }
    ph2d_panel_model3d::publish(ph2d_panel_model3d::ModelSnapshot {
        modes,
        frames,
        adds,
        ops,
        verbs,
        verb_subject,
        characters,
        mods,
        exports,
        acts,
        // ⭐⭐ **AS VISTAS** (W47) — derivadas de `Standard::ALL`, e o `active` derivado da
        // ORIENTAÇÃO da câmera, nunca de um modo guardado (ver `field3d_views::named_view`).
        views: views_now(),
        camera: camera_now(),
        rows,
        // ⭐ **O isolamento diz-se sozinho** (W44), e a pergunta é feita ao MUNDO, não à seleção:
        // um estado da vista anunciado através de um controle da seleção some quando se escolhe
        // outra coisa — que é exactamente quando o artista precisa de o ler.
        isolated: isolated_name(world, &all),
        node_count: all.len(),
        last_trace_ms: ms,
    });
}

/// ⭐ **Os números do objeto selecionado** — o painel é o inspetor da seleção.
///
/// ⚠️ **Mudou de forma na W10.** Antes era uma linha por nó com o raio dele — uma segunda vista da
/// estrutura, a competir com a Hierarquia e sem onde pôr largura, altura e profundidade. A divisão
/// passou a ser a da casa: a Hierarquia mostra **o que existe**, o painel mostra **os números do
/// escolhido**.
///
/// `view_span` é o alcance do **gesto** (ver [`ph2d_field::Span`]): uma posição e uma largura não
/// têm teto físico, e quem escolhe até onde o slider vai é a vista — o que cabe no enquadramento. O
/// documento só contribui as **paredes** (um filete que não cabe).
///
/// ⭐ **É AQUI, num sítio só, que uma faixa aberta se fecha.** O documento diz a *forma* do que cada
/// grandeza admite; esta função é a única que sabe o enquadramento, e é ela que escreve as duas
/// pontas. Espalhar isto por linha era o que fazia toda linha começar em zero.
pub(crate) fn param_rows(
    world: &bevy_ecs::world::World,
    selected: Option<bevy_ecs::entity::Entity>,
    view_span: f32,
) -> Vec<ph2d_panel_model3d::ParamRow> {
    let Some(e) = selected else {
        return Vec::new();
    };
    // ⚠️ O valor E as pontas vêm os DOIS do nó (`params_of`). Um painel que guardasse o seu próprio
    // valor teria duas verdades sobre o mesmo número, e a que aparece na tela seria a errada sempre
    // que algo o mudasse de outro lado — um desfazer, um arquivo aberto, o gizmo.
    // ⭐⭐⭐ **A que SECÇÃO cada linha pertence** (report do Enio, 2026-08-30: *«o modificador deveria
    // ter sua própria seção no painel»*). Um cabeçalho nasce quando a **natureza** da linha muda:
    // as dimensões da forma, e depois um por modificador, com o **nome dele**.
    //
    // ⚠️ **Derivado do `Param`, e não uma segunda lista** — a ordem já é a do `params_of` (a forma
    // primeiro, os modificadores por último, na ordem em que correm), e um segundo sítio a decidir
    // secções divergiria dela no dia em que ela mudasse.
    let mods = ph2d_field_ecs::mods_of(world, e);
    let mut anterior: Option<ph2d_field::Param> = None;
    let secao = move |p: ph2d_field::Param| -> Option<&'static str> {
        let mesma = match (anterior, p) {
            // Duas linhas do MESMO modificador continuam a secção dele.
            (
                Some(ph2d_field::Param::Mod { slot: a, .. }),
                ph2d_field::Param::Mod { slot: b, .. },
            ) => a == b,
            // Tudo o que não é modificador é a forma, e ela é uma secção só.
            (Some(a), b) => {
                !matches!(a, ph2d_field::Param::Mod { .. })
                    && !matches!(b, ph2d_field::Param::Mod { .. })
            }
            (None, _) => false,
        };
        anterior = Some(p);
        if mesma {
            return None;
        }
        match p {
            ph2d_field::Param::Mod { slot, .. } => mods
                .get(slot as usize)
                .map(|m| m.key())
                // ⚠️ Um slot sem modificador não pode acontecer (as duas listas saem do mesmo nó),
                // e se acontecer o cabeçalho genérico é melhor do que nenhum.
                .or(Some("panel.model3d.section.modifier")),
            _ => Some("panel.model3d.section.shape"),
        }
    };
    let mut secao = secao;
    ph2d_field_ecs::params_of(world, e)
        .into_iter()
        .map(|(param, d)| {
            use ph2d_field::{Bound, Span};
            let (lo, bound) = match d.span {
                // Positiva: o documento recusa `≤ 0`, e o teto é o que cabe no quadro.
                Span::Positive => (0.0, Bound::Soft(view_span)),
                // A única ponta que o documento **impõe**.
                Span::Wall(w) => (0.0, Bound::Hard(w)),
                // Simétrica em torno da origem: as duas pontas são da vista, e a de baixo é
                // negativa — sem isto uma posição negativa não se digita.
                Span::Free => (-view_span, Bound::Soft(view_span)),
                // ⭐⭐⭐ **A banda de um deformador é uma posição AO LONGO DO EIXO DELE**, e o alcance
                // dela nunca sai da peça — ver [`ph2d_field::Span::Along`] e o report de 2026-08-31.
                //
                // ⚠️ **É DERIVADO do alcance do gesto, e não um segundo número**: o `view_span` é a
                // oitava de `4×` o raio da peça (ver [`gesture_span`]), logo um quarto dele é a
                // oitava do próprio raio — que majora a meia-extensão da peça em **qualquer** eixo.
                // *Um segundo parâmetro seria uma segunda resposta à mesma pergunta, e divergiria no
                // dia em que a primeira mudasse.*
                Span::Along => (-view_span * 0.25, Bound::Soft(view_span * 0.25)),
                // Periódica: as pontas são da representação, e a vista não tem voto.
                Span::Turn(half) => (-half, Bound::Wrap(half)),
                // ⭐ **Sem faixa nenhuma**: a grandeza tem valor e não é editável neste estado. As
                // duas pontas colapsam no próprio valor — não há para onde arrastar — e a linha
                // segue marcada para o painel a pintar como facto.
                Span::Locked => (d.value, Bound::Wrap(d.value)),
                // ⭐ **Contagem**: as duas pontas são do DOCUMENTO — uma matriz começa em 1 (zero
                // cópias é a peça a desaparecer, e apagar já tem botão) e um prisma em 3 (abaixo
                // não há polígono).
                //
                // ⚠️ **O piso era o literal `1.0` aqui** (W101): com ele, o slider dos lados descia
                // a 1, a porta do documento coagia para 3, e o controle **saltava para trás
                // debaixo do dedo**. *Uma faixa que oferece o que a porta recusa é uma affordance
                // que mente* — e o piso é um facto do documento, não deste arquivo.
                Span::Count { min, max } => (min as f32, Bound::Hard(max as f32)),
                // Simétrica e fechada pelo documento: as duas pontas são paredes.
                Span::Walls(max) => (-max, Bound::Hard(max)),
                // ⭐ **Positiva OU zero** — o teto é da vista, como a `Positive`, e a diferença toda
                // está no piso: aqui o zero é uma resposta (o cone fechado), não uma recusa.
                Span::FromZero => (0.0, Bound::Soft(view_span)),
                // ⭐⭐ **Parede do documento E zero alcançável** — a faixa dos dois recuos de uma
                // aresta. ⚠️ O mapeamento é o mesmo da `Wall`; o que muda é do outro lado, na porta
                // de escrita, que agora aceita o zero que este slider sempre ofereceu.
                Span::WallFromZero(w) => (0.0, Bound::Hard(w)),
            };
            ph2d_panel_model3d::ParamRow {
                entity: e.to_bits(),
                param,
                key: d.key,
                value: d.value,
                lo,
                bound,
                live: d.span != Span::Locked,
                integral: matches!(d.span, Span::Count { .. }),
                section: secao(param),
            }
        })
        .collect()
}

/// ⭐ **Os modificadores oferecidos, e quais o nó já tem** — interruptores, não ações.
///
/// ⚠️ **A lista é derivada de [`ph2d_field::UnaryKind::ALL`]**, que é a fonte da contagem: um
/// modificador novo entra lá e o painel segue sem uma linha de mudança. É a mesma lei do `Mode::ALL`
/// e do `SHAPES`.
///
/// ⚠️ Vazio sem seleção — um interruptor sem nó para ligar não tem o que dizer.
pub(crate) fn mods_for(
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
    let here = with_smoke(|s| crate::field3d_views::named_view(&s.vp().cam)).flatten();
    crate::field3d_views::Standard::ALL
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
    let quad = with_smoke(|s| matches!(s.split, crate::field3d_layout::Split::Quad { .. }))
        .unwrap_or(false);
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
pub(crate) const CAMERA_ACTS: [&str; 3] = [
    "panel.model3d.camera.ortho",
    "panel.model3d.camera.frame",
    "panel.model3d.camera.quad",
];

/// A **lente** — interruptor. ⚠️ Derivados, nunca números à mão: um gesto novo no meio da lista
/// mudaria o índice e o botão passaria a fazer outra coisa, sem erro nenhum.
pub(crate) const ORTHO_SLOT: usize = 0;
/// O **enquadrar** — ação.
pub(crate) const FRAME_SLOT: usize = 1;
/// ⭐⭐ A **divisão do canvas** (W90) — interruptor, como a lente.
///
/// ⚠️ Ele mora nesta fileira e não numa nova porque a pergunta dela é *«como estou a olhar?»* — e
/// *«de quantos sítios ao mesmo tempo»* é uma resposta a essa, exactamente como a lente.
pub(crate) const QUAD_SLOT: usize = 2;

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
    let bits = crate::field3d_smoke::isolated()?;
    let e = all.iter().map(|(e, _)| *e).find(|e| e.to_bits() == bits)?;
    Some(
        world
            .get::<ph2d_ecs::Name>(e)
            .map_or_else(|| String::from("?"), |n| n.0.to_string()),
    )
}

/// As três booleanas, na ordem do seletor.
pub(crate) const OPS: [&str; 3] = [
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
pub(crate) fn new_shape_size(half_extent: f32) -> f32 {
    (half_extent * 0.25).max(f32::MIN_POSITIVE)
}

// ⚠️ **O `shape_at` mudou-se para [`crate::field3d_shapes`]** (W100), com o catálogo inteiro: cada
// forma passou a trazer o **próprio construtor**, então não há mais um `match slot` para viver aqui.
// *Um `match` posicional sobrevive a acrescentar no fim e parte-se em silêncio ao inserir no meio* —
// e o que vem por aí é uma lista de 60.

pub(crate) fn op_at(slot: usize) -> Option<Op> {
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
/// `ph2d-editor-core` —, e é para lá que a lista foi ([`crate::field3d_shape_palette`]).
///
/// ⭐ **A disponibilidade continua a ser respeitada, e ficou MELHOR:** as três formas que dependem
/// da seleção sumiam da fileira, e com isso o artista não podia saber que existem. Na paleta elas
/// aparecem com a **razão** ao lado.
///
/// ⚠️ **Devolve uma FILEIRA de um chip** e não um `bool` — ver [`ph2d_panel_model3d::ModelSnapshot`]
/// `adds`: o `paint_chips` já sabe desenhar, medir e registar; um botão avulso seria um caminho de
/// pintura novo neste painel.
pub(crate) fn adds_for() -> Vec<ph2d_panel_model3d::ModeChip> {
    vec![ph2d_panel_model3d::ModeChip {
        key: "panel.model3d.add.open",
        active: false,
    }]
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
pub(crate) fn ops_for(
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
