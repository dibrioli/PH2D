//! ⭐⭐⭐ **A TABELA DAS ARMAÇÕES — que painel a varredura sabe pintar com um DOCUMENTO na mão.**
//!
//! # ⛔⛔ O buraco, medido
//!
//! A varredura de elisões pinta todo painel do registo **no estado de FÁBRICA**, e em 2026-09-19
//! ninguém tinha contado quantos painéis isso deixa **vazios**. Contado:
//!
//! | painel | rótulos medidos, de fábrica |
//! |---|---:|
//! | `motion_graph` · `motion_params` · `inspector` (e os dois painéis 3D, que saíram com o ADR-0179) | **0** |
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
//! ⛔ **E o `desarma` é obrigatório**: estas portas são `thread_local`, e uma régua que pinte o
//! mesmo painel duas vezes NO MESMO teste (vazio e armado) mediria na segunda o que a primeira
//! deixou. *O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.* (Entre
//! testes não vaza: o libtest dá a cada um a sua thread, e o estado do painel é por fio.)

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
    // Os cartões do som: a camada de mais campos, o corpo vibrante.
    Armacao {
        painel: "sfxlab",
        arma: |_| arma_o_som(),
        desarma: || ph2d_panel_sfxlab::publica(None),
    },
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
    // O painel MACROS (ADR-0186, W5d) com um objecto gravado e editado: o botão Gravar, a linha de
    // pronto, o resumo, a linha «editada desde» e as três etapas.
    #[cfg(feature = "panel-macros")]
    Armacao {
        painel: "macros",
        arma: |_| arma_as_macros(),
        desarma: desarma_as_macros,
    },
    // O painel PIXEL (ADR-0196) com um sprite em Edit: as quatro ferramentas, as duas cores, o
    // Trocar e a paleta.
    #[cfg(feature = "panel-pixel-tools")]
    Armacao {
        painel: "pixel_tools",
        arma: |_| arma_o_pixel(),
        desarma: desarma_o_pixel,
    },
    // A Preview do Pixel (P5) com uma imagem 16×8 publicada.
    #[cfg(feature = "panel-pixel-preview")]
    Armacao {
        painel: "pixel_preview",
        arma: |_| arma_a_preview(),
        desarma: || ph2d_panel_pixel_preview::publish(None),
    },
    // As janelas de Export e Import do Pixel (P9b): abre no Export As, com duas camadas e uma tag.
    #[cfg(feature = "panel-pixel-export")]
    Armacao {
        painel: "pixel_export",
        arma: |_| arma_o_export(),
        desarma: || ph2d_panel_pixel_export::canal::publish_contexto(Default::default()),
    },
    // A tela dos tilemaps (P10b) com uma camada de tilemap activa e dois tiles.
    #[cfg(feature = "panel-pixel-tiles")]
    Armacao {
        painel: "pixel_tiles",
        arma: |_| arma_os_tiles(),
        desarma: || ph2d_panel_pixel_tiles::canal::publish_contexto(Default::default()),
    },
    // A janela SLICES (P11b) com a ferramenta na mão e uma slice selecionada.
    #[cfg(feature = "panel-pixel-slices")]
    Armacao {
        painel: "pixel_slices",
        arma: |_| arma_as_slices(),
        desarma: || ph2d_panel_pixel_slices::canal::publish_contexto(Default::default()),
    },
    // A linha do tempo do Pixel (P4) com duas camadas × três frames, um par ligado e uma tag.
    #[cfg(feature = "panel-pixel-timeline")]
    Armacao {
        painel: "pixel_timeline",
        arma: |_| arma_a_linha_do_tempo(),
        desarma: desarma_a_linha_do_tempo,
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
    // ⭐ Sem a «Scene Root» de fixtura desde 08/10: a raiz CENA é do PAINEL agora (doc 01 §4.6,
    //    `rows::expand`), e ele põe tudo um nível abaixo dela — a árvore medida é a MESMA de antes.
    let linhas: [Linha; 5] = [
        ("Hero (tall variant)", 0, Some("Prefab"), true, false, false),
        (
            "Enemy Spawner · left wing",
            0,
            Some("Variant"),
            true,
            true,
            false,
        ),
        (
            "Background Parallax Layer",
            0,
            Some("Overridden"),
            false,
            false,
            true,
        ),
        ("Collision Geometry", 1, Some("Locked"), true, true, true),
        ("Footsteps (audio source)", 1, None, true, false, false),
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
                    selected: indent == 0,
                    hovered: false,
                    muted: !visivel,
                    locked: trancado,
                    group_locked: grupo,
                    role: Default::default(),
                    components: Vec::new(),
                    content: Vec::new(),
                    has: Default::default(),
                    entity: None,
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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// AS MACROS
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Um objecto com três pinceladas gravadas, editado desde a 1.ª, e o Painter pronto a gravar.
#[cfg(feature = "panel-macros")]
fn arma_as_macros() {
    use ph2d_editor_core::macro_edits::{
        EtapaView, FamiliaDaMacro, MacroObjetoView, MacroPanelInfo, PropsDaEtapa, TipoDaEtapaView,
    };
    let etapa = |k: usize| {
        #[expect(clippy::cast_precision_loss, reason = "k < 3")]
        let t = k as f64 * 0.5;
        EtapaView {
            trilha: 0,
            k,
            familia: FamiliaDaMacro::Painter,
            tipo: TipoDaEtapaView::Pincelada(k + 1),
            nome: None,
            dica: None,
            t0_s: t,
            t1_s: t + 0.5,
            espera_s: 0.0,
            props: PropsDaEtapa::Pincelada {
                cor: [10, 20, 30],
                tamanho_px: 12.0,
            },
        }
    };
    ph2d_panel_macros::set_current_macros(MacroPanelInfo {
        pode_gravar: Some(FamiliaDaMacro::Painter),
        objeto: Some(MacroObjetoView {
            entity_bits: 1,
            duracao_s: 2.0,
            etapas: (0..3).map(etapa).collect(),
            editada_desde: Some(1),
            erro: None,
        }),
        ..Default::default()
    });
}

#[cfg(feature = "panel-macros")]
fn desarma_as_macros() {
    ph2d_panel_macros::set_current_macros(ph2d_editor_core::macro_edits::MacroPanelInfo::default());
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O PIXEL
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Um Pixel Sprite em Edit, com o lápis na mão e uma paleta de 32 cores.
#[cfg(feature = "panel-pixel-tools")]
fn arma_o_pixel() {
    use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelTool};
    ph2d_panel_pixel_tools::set_current_pixel(PixelPanelInfo {
        editing: true,
        tool: PixelTool::Pencil,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 128],
        palette: (0..32u8)
            .map(|i| [i * 7, 255 - i * 5, i * 3, 255])
            .collect(),
        options: Default::default(),
        layers: vec![camada_do_pixel(1, "Layer 1")],
        active_layer: Some(1),
        user: ph2d_editor_core::pixel_panel::PixelUserInfo {
            layer: Some(user_data_armado()),
            sprite: Some(user_data_armado()),
            ..Default::default()
        },
        ..Default::default()
    });
    ph2d_editor_core::panel::cards::user_data::set_open(
        &ph2d_panel_pixel_tools::seam_user::ids(),
        true,
    );
    // Sprite ▸ Properties… aberta, com o cartão User Data do sprite aberto (P12 U).
    ph2d_editor_core::pixel_panel::sprite_props::set_open(true);
    ph2d_editor_core::panel::cards::user_data::set_open(
        &ph2d_panel_pixel_tools::seam_user::ids_sprite(),
        true,
    );
}

/// Uma camada da lista do Pixel.
#[cfg(any(feature = "panel-pixel-tools", feature = "panel-pixel-timeline"))]
fn camada_do_pixel(id: u32, name: &str) -> ph2d_editor_core::pixel_panel::PixelLayerRow {
    ph2d_editor_core::pixel_panel::PixelLayerRow {
        id,
        name: name.to_owned(),
        depth: 0,
        group: false,
        collapsed: false,
        visible: true,
        locked: false,
        continuous: false,
        reference: false,
        background: false,
        opacity: 255,
        blend: Default::default(),
    }
}

/// O cartão User Data com as três espécies de linha: editável, só de leitura e de extensão (P12 U).
#[cfg(any(
    feature = "panel-pixel-tools",
    feature = "panel-pixel-timeline",
    feature = "panel-pixel-tiles"
))]
fn user_data_armado() -> ph2d_editor_core::pixel_panel::user_data::UserDataView {
    use ph2d_editor_core::pixel_panel::user_data::{PropRow, UserDataView};
    let linha = |name: &str, value: &str, kind: &'static str, editable: bool| PropRow {
        name: name.to_owned(),
        value: value.to_owned(),
        kind,
        editable,
    };
    UserDataView {
        color: [200, 60, 60, 255],
        text: "hero".to_owned(),
        rows: vec![
            linha("hp", "3", "pixel.user.type.int", true),
            linha("size", "16x8", "Size", false),
        ],
        extensions: vec![(
            "ext".to_owned(),
            vec![linha("id", "7", "pixel.user.type.int", true)],
        )],
    }
}

#[cfg(feature = "panel-pixel-tools")]
fn desarma_o_pixel() {
    ph2d_panel_pixel_tools::set_current_pixel(
        ph2d_editor_core::pixel_panel::PixelPanelInfo::default(),
    );
}

/// A Preview do Pixel com uma imagem publicada (a família publica-a com a janela aberta).
#[cfg(feature = "panel-pixel-preview")]
fn arma_a_preview() {
    ph2d_panel_pixel_preview::publish(Some(ph2d_panel_pixel_preview::PreviewFrame {
        rgba: std::sync::Arc::new(vec![255; 16 * 8 * 4]),
        w: 16,
        h: 8,
        seq: 1,
        frame: 1,
        playing: false,
        checker: Default::default(),
    }));
}

/// O Export As com um sprite de duas camadas e uma tag publicado.
#[cfg(feature = "panel-pixel-export")]
fn arma_o_export() {
    use ph2d_panel_pixel_export::canal;
    canal::publish_contexto(canal::Contexto {
        lados: (16, 8),
        quadros: 3,
        camadas: vec![(ph2d_pixel_core::LayerId(1), "Layer 1".to_owned())],
        tags: vec!["walk".to_owned()],
        tilesets: Vec::new(),
        slices: vec![(ph2d_pixel_core::SliceId(1), "button".to_owned())],
    });
    canal::abrir(canal::Tela::ExportAs);
}

/// Uma camada de tilemap activa, com o tile vazio e dois desenhados.
#[cfg(feature = "panel-pixel-tiles")]
fn arma_os_tiles() {
    use ph2d_panel_pixel_tiles::canal;
    let tile = vec![255; 4 * 4 * 4];
    canal::publish_contexto(canal::Contexto {
        mao: Some(canal::Mao::default()),
        tile_w: 4,
        tile_h: 4,
        imagens: std::sync::Arc::new(vec![vec![0; 4 * 4 * 4], tile.clone(), tile]),
        user: Some(user_data_armado()),
        ..canal::Contexto::default()
    });
    ph2d_editor_core::panel::cards::user_data::set_open(
        &ph2d_panel_pixel_tiles::ids::user_data(),
        true,
    );
}

/// A Slice Tool na mão, com uma slice selecionada.
#[cfg(feature = "panel-pixel-slices")]
fn arma_as_slices() {
    use ph2d_panel_pixel_slices::canal;
    canal::publish_contexto(canal::Contexto {
        na_mao: true,
        linhas: vec![canal::Linha {
            id: ph2d_pixel_core::SliceId(1),
            nome: "button".to_owned(),
            selecionada: true,
        }],
        vista: Some(canal::Vista {
            nome: Some("button".to_owned()),
            cor: Some([0, 0, 255, 255]),
            bounds: [Some(1), Some(2), Some(3), Some(4)],
            ..canal::Vista::default()
        }),
        transform: false,
        user: None,
    });
}

/// Um Pixel Sprite em Draw com frames, cels ligados e uma tag (a linha do tempo armada).
#[cfg(feature = "panel-pixel-timeline")]
fn arma_a_linha_do_tempo() {
    use ph2d_editor_core::pixel_panel::{
        PixelCel, PixelDirection, PixelFrameCol, PixelPanelInfo, PixelTagInfo,
    };
    let camada = camada_do_pixel;
    let cel = |layer: u32, frame: u32, owner: u32| PixelCel {
        layer,
        frame,
        owner,
        opacity: 255,
        z: 0,
        empty: false,
    };
    ph2d_panel_pixel_timeline::set_current_pixel(PixelPanelInfo {
        editing: true,
        layers: vec![camada(2, "Ink"), camada(1, "Background")],
        active_layer: Some(2),
        frames: (1..=3)
            .map(|id| PixelFrameCol {
                id,
                duration_ms: 100 * id,
            })
            .collect(),
        active_frame: Some(1),
        cels: vec![cel(2, 1, 1), cel(2, 2, 1), cel(1, 1, 1), cel(1, 3, 3)],
        tags: vec![PixelTagInfo {
            name: "walk".to_owned(),
            from: 0,
            to: 1,
            direction: PixelDirection::PingPong,
            repeat: 2,
            color: [200, 60, 60, 255],
        }],
        user: ph2d_editor_core::pixel_panel::PixelUserInfo {
            cel: Some(user_data_armado()),
            tag: Some(user_data_armado()),
            ..Default::default()
        },
        ..Default::default()
    });
    for ids in [
        ph2d_panel_pixel_timeline::seam_user::cel_ids(),
        ph2d_panel_pixel_timeline::seam_user::tag_ids(),
    ] {
        ph2d_editor_core::panel::cards::user_data::set_open(&ids, true);
    }
}

#[cfg(feature = "panel-pixel-timeline")]
fn desarma_a_linha_do_tempo() {
    ph2d_panel_pixel_timeline::set_current_pixel(
        ph2d_editor_core::pixel_panel::PixelPanelInfo::default(),
    );
}

fn arma_o_som() {
    use ph2d_sfx_recipe::{Body, Envelope, ModeLaw, Recipe, Source};
    let mut receita = Recipe::new(1, 1.0);
    let sel = receita.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Stretched { stretch: 1.5 },
            base_hz: 420.0,
            modes: 12,
            t60: 0.8,
            damping: 0.1,
            rolloff: 0.8,
            contact_ms: 1.0,
            noise: 0.2,
        }),
        Envelope::hit(0.001, 0.4, 0.01),
    );
    ph2d_panel_sfxlab::publica(Some(ph2d_panel_sfxlab::Instantaneo {
        receita,
        sel: Some(sel),
        materiais: Vec::new(),
    }));
}
