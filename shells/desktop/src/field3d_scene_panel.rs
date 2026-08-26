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
    let (live_sculpt, profile) =
        with_smoke(|s| (s.has_live_sculpt, s.profile_pick.is_some())).unwrap_or((false, false));
    let adds = adds_for(live_sculpt, profile);
    let ops = ops_for(world, selection);
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
        let mut m: std::collections::BTreeMap<u64, &'static str> = all
            .iter()
            .filter(|(e, _)| {
                world
                    .get::<ph2d_field_ecs::FieldProfileSource>(*e)
                    .is_some()
            })
            .map(|(e, _)| (e.to_bits(), LINK_BADGE))
            .collect();
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
                // Periódica: as pontas são da representação, e a vista não tem voto.
                Span::Turn(half) => (-half, Bound::Wrap(half)),
                // ⭐ **Sem faixa nenhuma**: a grandeza tem valor e não é editável neste estado. As
                // duas pontas colapsam no próprio valor — não há para onde arrastar — e a linha
                // segue marcada para o painel a pintar como facto.
                Span::Locked => (d.value, Bound::Wrap(d.value)),
                // ⭐ **Contagem**: as duas pontas são do DOCUMENTO — o piso é 1 porque zero cópias
                // é a peça a desaparecer (e apagar já tem botão), e o teto é o da matriz.
                Span::Count { max } => (1.0, Bound::Hard(max as f32)),
                // Simétrica e fechada pelo documento: as duas pontas são paredes.
                Span::Walls(max) => (-max, Bound::Hard(max)),
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
            }
        })
        .collect()
}

/// ⭐ **As formas que se podem acrescentar**, na ordem do seletor.
///
/// ⚠️ **É a fonte da contagem**, como o `Mode::ALL`: acrescentar uma primitiva aqui faz o painel
/// seguir sem uma linha de mudança.
pub(crate) const SHAPES: [&str; 8] = [
    "panel.model3d.add.box",
    "panel.model3d.add.sphere",
    "panel.model3d.add.cylinder",
    "panel.model3d.add.torus",
    // ⭐⭐ **AS FORMAS DE PERFIL** (W53) — o desenho do editor vetorial vira peça.
    //
    // ⚠️ Elas entram **antes** das esculturas de propósito: os slots delas são derivados do FIM da
    // lista (`len()-2`, `len()-1`), e é essa derivação que faz acrescentar aqui não partir nada.
    "panel.model3d.add.extrude",
    "panel.model3d.add.revolve",
    "panel.model3d.add.sculpt",
    "panel.model3d.add.sculpt_scene",
];

/// ⭐ **As duas formas que saem de um perfil DESENHADO**, e os slots delas — derivados, como os das
/// esculturas.
pub(crate) const EXTRUDE_SLOT: usize = SHAPES.len() - 4;
pub(crate) const REVOLVE_SLOT: usize = SHAPES.len() - 3;

/// ⭐ **A posição da escultura no seletor** — e ela é DERIVADA, nunca escrita.
///
/// ⚠️ O `shape_at` devolve `None` neste slot de propósito (uma escultura não é uma primitiva), e
/// quem o trata é o braço do `AddShape`. Um literal `4` ali sobreviveria a acrescentar uma primitiva
/// no meio da lista e passaria a abrir o diálogo no botão errado — **sem erro nenhum**.
pub(crate) const SCULPT_SLOT: usize = SHAPES.len() - 2;

/// ⭐ **A posição da escultura VIVA da cena** (W39) — a que não passa pelo disco. Derivada pela
/// mesma razão da irmã acima.
///
/// ⚠️ **Ela só é OFERECIDA quando há uma escultura na cena** (a lei da W34): um botão que diz
/// *"trazer a escultura"* sem escultura nenhuma é a affordance que mente.
pub(crate) const SCULPT_SCENE_SLOT: usize = SHAPES.len() - 1;

/// As ações sobre o objeto escolhido, na ordem do seletor.
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
    let here = with_smoke(|s| crate::field3d_views::named_view(&s.cam)).flatten();
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
        with_smoke(|s| matches!(s.cam.lens, ph2d_field_render::Lens::Ortho)).unwrap_or(false);
    vec![
        ph2d_panel_model3d::ModeChip {
            key: CAMERA_ACTS[ORTHO_SLOT],
            active: ortho,
        },
        ph2d_panel_model3d::ModeChip {
            key: CAMERA_ACTS[FRAME_SLOT],
            active: false,
        },
    ]
}

/// Os gestos de câmera, na ordem do seletor.
pub(crate) const CAMERA_ACTS: [&str; 2] =
    ["panel.model3d.camera.ortho", "panel.model3d.camera.frame"];

/// A **lente** — interruptor. ⚠️ Derivados, nunca números à mão: um gesto novo no meio da lista
/// mudaria o índice e o botão passaria a fazer outra coisa, sem erro nenhum.
pub(crate) const ORTHO_SLOT: usize = 0;
/// O **enquadrar** — ação.
pub(crate) const FRAME_SLOT: usize = 1;

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

/// A primitiva que cada posição do seletor cria, no tamanho do enquadramento.
///
/// ⚠️ **Todas nascem com o `round` que têm direito**, e não a zero: este é o módulo cujo argumento é
/// o arredondamento, e uma caixa de aresta viva ao nascer esconderia exatamente aquilo que ele faz
/// melhor do que o Blender. O valor é uma fração do tamanho, então ele cabe sempre.
pub(crate) fn shape_at(slot: usize, r: f32) -> Option<Primitive> {
    let round = r * 0.1;
    Some(match slot {
        0 => Primitive::Box {
            half: [r; 3],
            round,
        },
        1 => Primitive::Sphere { radius: r },
        2 => Primitive::Cylinder {
            radius: r,
            half_height: r * 1.2,
            round,
        },
        3 => Primitive::Torus {
            major: r,
            minor: r * 0.35,
        },
        // ⚠️ **As formas de PERFIL e as ESCULTURAS não saem daqui** (W53/W22/W39): elas não são
        // construíveis a partir de um raio — precisam do contorno desenhado ou de um arquivo, que
        // vivem fora do mundo. Quem as trata é o braço próprio do `AddShape`, e o gate
        // `the_sculpt_slot_points_at_the_sculpt_button` prende as quatro exceções.
        _ => return None,
    })
}

pub(crate) fn op_at(slot: usize) -> Option<Op> {
    Some(match slot {
        0 => Op::Union(Blend::Sharp),
        1 => Op::Difference(Blend::Sharp),
        2 => Op::Intersection(Blend::Sharp),
        _ => return None,
    })
}

/// ⭐ **As formas que se podem acrescentar AGORA** — e uma delas não está sempre disponível.
///
/// ⚠️ **A escultura da CENA sai da lista quando não há cena esculpida** (W39): é a lei da W34
/// aplicada à única forma cuja disponibilidade não é constante — *um botão que diz «trazer a
/// escultura» sem escultura nenhuma é a affordance que mente*. As outras cinco são sempre
/// possíveis: uma caixa não depende de nada.
///
/// ⚠️ **Função pura**, pela razão do `next_isolation`: o facto vive no `Smoke`, que só nasce com o
/// módulo armado e cujo estado é `thread_local` — armá-lo num gate contaminaria os vizinhos.
pub(crate) fn adds_for(live_sculpt: bool, profile: bool) -> Vec<ph2d_panel_model3d::ModeChip> {
    SHAPES
        .iter()
        .enumerate()
        // ⚠️ **As formas de perfil só aparecem com um contorno FECHADO escolhido** (a lei da W34):
        // um botão *Extrude* sem nada para extrudar é a affordance que mente — e o gesto teria de
        // falhar em silêncio ou com um aviso, que é pior do que não estar lá.
        .filter(|(i, _)| (*i != EXTRUDE_SLOT && *i != REVOLVE_SLOT) || profile)
        .filter(|(i, _)| *i != SCULPT_SCENE_SLOT || live_sculpt)
        .map(|(_, key)| ph2d_panel_model3d::ModeChip { key, active: false })
        .collect()
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
