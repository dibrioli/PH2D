//! ⭐⭐⭐ **A TABELA DAS ARMAÇÕES — que painel a varredura sabe pintar com um DOCUMENTO na mão.**
//!
//! # ⛔⛔ O buraco, medido
//!
//! A varredura de elisões pinta todo painel do registo **no estado de FÁBRICA**, e em 2026-09-19
//! ninguém tinha contado quantos painéis isso deixa **vazios**. Contado:
//!
//! | painel | rótulos medidos, de fábrica |
//! |---|---:|
//! | `model3d` · `motion_graph` · `motion_params` · `sculpt3d` · `inspector` | **0** |
//! | `tags` | 2 |
//! | `hierarchy` | 3 |
//! | `skeleton` | 7 |
//!
//! ⚠️ **O piso da varredura é GLOBAL** (`2 800` rótulos), logo ela fica verde com **oito** painéis
//! invisíveis — e um deles é o Inspector, o painel com mais fileiras do app. *Zero lê-se como
//! aprovação, e um piso sobre a soma não pergunta por ninguém.*
//!
//! # ⭐⭐ O que esta tabela é
//!
//! Um par `(armar, desarmar)` por painel. A varredura pinta cada painel **duas** vezes quando ele
//! tem armação: vazio e armado.
//!
//! ⚠️ **`arma` corre ANTES do `populate`**, e isso não é ordem de conveniência: as `populate_*` dos
//! painéis semeiam os widgets a partir do instantâneo publicado, logo um `populate` corrido antes
//! veria o painel vazio e a segunda passagem mediria as mesmas fileiras da primeira.
//!
//! ⛔ **E o `desarma` é obrigatório**: estas portas são `thread_local` e o binário de teste corre
//! todos os módulos na mesma thread. *O estado que uma fixtura deixa para trás é o estado que a
//! régua seguinte mede.*

use ph2d_editor_core::interaction::WidgetStore;

/// Uma armação: o painel, como se lhe dá um documento, e como se lho tira.
pub struct Armacao {
    pub painel: &'static str,
    pub arma: fn(&mut WidgetStore),
    pub desarma: fn(),
}

/// ⭐ **A população que a varredura sabe armar.** Uma entrada nova aqui é um painel que deixa de
/// ser medido vazio.
pub const TABELA: &[Armacao] = &[
    Armacao {
        painel: "inspector",
        arma: |_| super::o_inspector_armado::arma_tudo(),
        desarma: super::o_inspector_armado::desarma_tudo,
    },
    // ⭐⭐ **O PAINTER com um padrão na mão** — ver [`super::o_painter_armado`]. Sem ele a
    //    varredura mede a secção `SHAPE ▸ Texture` VAZIA, que é onde o report de 2026-09-21 vivia.
    #[cfg(feature = "panel-painter-layers")]
    Armacao {
        painel: "painter_layers",
        arma: |_| super::o_painter_armado::arma(),
        desarma: super::o_painter_armado::desarma,
    },
    Armacao {
        painel: "hierarchy",
        arma: arma_a_arvore,
        desarma: desarma_a_arvore,
    },
    Armacao {
        painel: "tags",
        arma: |_| arma_as_tags(),
        desarma: desarma_as_tags,
    },
    // ⛔⛔ **O painel de params do Motion SAIU do app** (`line/motion-value`, ordem do dono de
    //    2026-09-17: os params vivem NO CARTÃO). A armação dele vivia aqui e mediu as `828`
    //    entradas do catálogo de params por largura — ⚠️ e essa medição **perdeu a superfície**,
    //    porque ela pintava ATRAVÉS do painel (`set_current_params` era a porta `thread_local`
    //    dele). *Não é dívida de tradução: é uma régua sem sujeito.*
    //
    //    ⭐ **O consumidor NOVO já existe e já é medido em parte:** o cartão do grafo expõe
    //    `ph2d_panel_motion_graph::nome_cabe_na_capsula`, e a
    //    `ph2d_app_motion::motion_param_reach_tests` usa-a sobre os nomes de TIPO. ⏳ O que fica
    //    ABERTO é a mesma medição sobre os `828` rótulos de PARAM no cartão — ela pede uma fixtura
    //    que monte um cartão, que é desenho e não integração.
    // ⭐⭐ **O painel do MODELADOR 3D** — ele estava declarado como «precisa de um mundo ECS para
    //    cozer o `FieldDoc`», e precisa de um SNAPSHOT: ver [`super::o_model3d_armado`].
    Armacao {
        painel: "model3d",
        arma: |_| super::o_model3d_armado::arma(),
        desarma: super::o_model3d_armado::desarma,
    },
    // ⭐⭐ **O painel da ESCULTURA** — ele estava declarado como «a cena segura uma surface de
    //    wgpu», o que é verdade sobre quem PUBLICA e falso sobre o que atravessa: ver
    //    [`super::o_sculpt3d_armado`].
    Armacao {
        painel: "sculpt3d",
        arma: |_| super::o_sculpt3d_armado::arma(),
        desarma: super::o_sculpt3d_armado::desarma,
    },
];

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A HIERARQUIA
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐ **Uma cena com os SETE SELOS da linha da Hierarquia ao mesmo tempo.**
///
/// ⛔ A fixtura de fábrica dela é **uma** linha (`Scene Root`, `fixture::hierarchy()`), logo a
/// varredura media `3` rótulos — e o handoff de 19/09 já nomeava, por escrito, que *«seis dos sete
/// selos da Hierarquia»* tinham um defeito de largura que **esta varredura não podia ver**.
///
/// ⚠️ Os nomes são compridos de propósito: o que se mede aqui é a coluna, e um `Cube` de quatro
/// letras cabe em qualquer sítio.
fn arma_a_arvore(store: &mut WidgetStore) {
    use ph2d_editor_core::NodeId;
    use ph2d_editor_core::icons::IconId;
    use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;

    /// `(nome, recuo, selo, visível, trancado, grupo trancado)` — a linha da árvore como o
    /// artista a vê.
    type Linha = (&'static str, u8, Option<&'static str>, bool, bool, bool);
    let linhas: [Linha; 6] = [
        ("Scene Root", 0, None, true, false, false),
        ("Hero (tall variant)", 1, Some("Prefab"), true, false, false),
        (
            "Enemy Spawner · left wing",
            1,
            Some("Variant"),
            true,
            true,
            false,
        ),
        (
            "Background Parallax Layer",
            1,
            Some("Overridden"),
            false,
            false,
            true,
        ),
        ("Collision Geometry", 2, Some("Locked"), true, true, true),
        ("Footsteps (audio source)", 2, None, true, false, false),
    ];
    let ids: Vec<NodeId> = (0..linhas.len())
        .map(|i| NodeId(9_000 + i as u64))
        .collect();
    let mapa = ids
        .iter()
        .zip(linhas)
        .map(|(&id, (nome, indent, selo, visivel, trancado, grupo))| {
            (
                id,
                HierarchyEntity {
                    name: nome.to_string(),
                    icon: IconId::Folder,
                    indent,
                    badge: selo.map(str::to_string),
                    swatch: Some([200, 120, 60, 255]),
                    visible: visivel,
                    selected: indent == 1,
                    hovered: false,
                    muted: !visivel,
                    locked: trancado,
                    group_locked: grupo,
                },
            )
        })
        .collect();
    ph2d_panel_hierarchy::sync_from_hierarchy(store, &ids, mapa);
    ph2d_panel_hierarchy::set_live_component_count(42);
}

/// **Desarmar é DUAS portas:** o `clear_live_hierarchy` devolve a árvore à fixtura e **não toca no
/// contador de componentes**, que é outra porta (`set_live_component_count`).
///
/// ⚠️⚠️ **E a minha premissa sobre esta linha foi REFUTADA por mutação, no mesmo dia em que a
/// escrevi.** Eu escrevi que deixar o contador a `42` punha aquele número no cabeçalho de toda a
/// suíte seguinte; apagada a linha, o `uma_fixtura_nao_deixa_nada_para_tras` fica **VERDE**. A
/// razão é que o painel só lê o contador **em modo vivo**, e em modo de fixtura ele usa o
/// `fixture::hierarchy_counts()` — logo a fuga é real entre módulos e **invisível por este
/// caminho**. ⇒ a linha fica como HIGIENE, com a medição escrita ao lado, e não como lei: *uma
/// linha que a mutação não consegue matar tem de dizer que sabe disso.*
fn desarma_a_arvore() {
    ph2d_panel_hierarchy::clear_live_hierarchy();
    ph2d_panel_hierarchy::set_live_component_count(0);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// AS TAGS
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ **A árvore de tags com uma recusa viva.**
///
/// ⚠️ A `problem` entra porque ela é a **frase** que o painel pinta na linha em que o gesto foi
/// recusado — e uma frase é exactamente o que a lei da reticência corta pela metade.
fn arma_as_tags() {
    use ph2d_editor_core::tags_edits::{TagsPanelInfo, TagsPanelRow};
    ph2d_panel_tags::set_current_tags(TagsPanelInfo {
        rows: vec![
            TagsPanelRow {
                id: 1,
                label: "Enemy".to_string(),
                depth: 0,
                members: 12,
                subtree: 18,
            },
            TagsPanelRow {
                id: 2,
                label: "Flying".to_string(),
                depth: 1,
                members: 6,
                subtree: 6,
            },
            TagsPanelRow {
                id: 3,
                label: "Destructible Scenery".to_string(),
                depth: 1,
                members: 0,
                subtree: 0,
            },
        ],
        problem: Some((3, "A tag with this name already exists here.".to_string())),
    });
}

fn desarma_as_tags() {
    ph2d_panel_tags::set_current_tags(ph2d_editor_core::tags_edits::TagsPanelInfo::default());
}
