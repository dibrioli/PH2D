//! ⭐⭐⭐ **DUAS SECÇÕES NUNCA SE PINTAM UMA POR CIMA DA OUTRA.**
//!
//! # O defeito que este gate existe para apagar, e porque nenhum outro o via
//!
//! Report do dono, 2026-09-09, com foto: *«secção Signal toda embolada e mal formatada»*. As
//! secções **TIMERS** e **SIGNAL ACTIONS** desenhavam-se exactamente no mesmo `y` — cabeçalho sobre
//! cabeçalho, o campo de um por cima do botão do outro.
//!
//! A causa: no orquestrador das secções com estado de painel, o `y` devolvido por
//! `paint_anchor_section` e por `paint_timer_section` era **descartado** (`f(...)` em vez de
//! `y = f(...)`), e as seguintes recebiam o `y` de antes delas. ⚠️ **Foi introduzido na wave do
//! `Timer` e repetido na do `SignalActions`** — e ficou invisível porque só se vê quando **duas**
//! daquelas secções estão presentes no MESMO objecto, o que exige um objecto com dois desses
//! componentes.
//!
//! ⛔⛔ **E nenhum portão desta casa o via, por construção:**
//!
//! | portão | o que ele pergunta | porque é cego a isto |
//! |---|---|---|
//! | `architecture_panel_wiring_parity` | *este id chega ao `hit_index`?* | chega — **os dois** chegam, sobrepostos |
//! | `hr12_widgets_a11y` | *o ficheiro delega a11y?* | delega |
//! | `architecture_panel_loc_cap` | *quantas linhas tem?* | outra grandeza |
//! | `seam_*` | *o clique chega à ferramenta?* | chega — ao que ficar por cima |
//!
//! *Nenhum instrumento do repo perguntava ONDE uma coisa é desenhada.* O `player_card_spans` é o
//! único vizinho, e mede a §14 sozinha.
//!
//! ⇒ a régua é a **GEOMETRIA das bandas de cabeçalho**, lida do índice de acerto que o painel de
//! facto publica — nunca de uma soma de alturas escrita ao lado do pintor, que seria a segunda
//! resposta à mesma pergunta.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{
    InspectorActionInfo, InspectorActionRow, InspectorTimerInfo, InspectorTimerRow,
    InspectorTransformInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_action, set_current_inspector_camera,
    set_current_inspector_shake, set_current_inspector_tags, set_current_inspector_timer,
    set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn transform() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

fn timers() -> InspectorTimerInfo {
    InspectorTimerInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![InspectorTimerRow {
            name: "gatilho".into(),
            duration_s: 3.0,
            repeat: false,
            autostart: true,
            signal: "abre".into(),
        }],
        selected_count: 1,
    }
}

fn actions() -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: 0xABCD_1234,
        rows: vec![
            InspectorActionRow {
                on: "abre".into(),
                target: "Porta".into(),
                verb_tag: 3,
                arg: String::new(),
                uses_arg: false,
                uses_target: true,
                target_mode: 0,
                from_tag: 0,
                target_tag: None,
                target_tag_path: String::new(),
            },
            InspectorActionRow {
                on: "abre".into(),
                target: "Motor".into(),
                verb_tag: 0,
                arg: String::new(),
                uses_arg: true,
                uses_target: true,
                target_mode: 0,
                from_tag: 0,
                target_tag: None,
                target_tag_path: String::new(),
            },
        ],
        verb_labels: vec![
            "Start Timer".into(),
            "Stop Timer".into(),
            "Show".into(),
            "Hide".into(),
            "Toggle Visibility".into(),
        ],
        selected_count: 1,
    }
}

/// ⭐⭐ A secção CAMERA na fixtura — ela está aqui porque a mutação que deita fora o `y` dela só é
/// observável se **alguma secção for pintada DEPOIS**: um `paint_<x>_section` sobre um snapshot
/// `None` devolve o `y` que recebeu, logo trocar `y = f(…)` por `let _ = f(…)` num vizinho mudo é
/// um no-op. *A fixtura tem de conter a secção que a cadeia empilha, não só a que se acrescentou.*
///
/// ⛔⛔ **E QUEM está depois dela MUDOU em 2026-09-22.** O doc desta função dizia *«sem ela a TAGS
/// não tem nada por cima»* — verdade enquanto a ordem das opcionais era a ordem em que foram
/// construídas, e **falsa** desde que ela passou a ser a das FAMÍLIAS do catálogo: a TAGS é hoje a
/// **primeira** (família IDENTIDADE) e a CAMERA está na última (SAÍDA), seguida de `shake`,
/// `shake_emitter` e `script`. ⇒ o consumidor do `y` da câmera nesta fixtura é o [`shake`], e é por
/// isso que ele entrou aqui no mesmo dia.
fn camera() -> ph2d_editor_core::screens::hero::InspectorCameraInfo {
    ph2d_editor_core::screens::hero::InspectorCameraInfo {
        entity_bits: 1,
        camera: ph2d_editor_core::screens::hero::InspectorGameCamera {
            height_world: 10.0,
            offset: [0.0, 0.0],
            priority: 0,
            active: true,
            cull_mask: u32::MAX,
        },
        follow: None,
        limits: None,
        camera_count: 1,
        is_active_camera: true,
        preview_on: false,
        selected_count: 1,
    }
}

/// ⭐⭐ A secção SHAKE na fixtura — **o consumidor do `y` da CAMERA desde a ordem por famílias**.
///
/// ⚠️ Ela não está aqui por ser interessante: está porque é a primeira secção armável **abaixo** da
/// câmera na família SAÍDA (`audio → camera → shake → shake_emitter → script`). *Sem ela, a mutação
/// que deita fora o `y` da câmera volta a ser um no-op, e este gate deixava de defender a metade
/// mais funda da cadeia.*
fn shake() -> ph2d_editor_core::shake_edits::InspectorShakeInfo {
    ph2d_editor_core::shake_edits::InspectorShakeInfo {
        entity_bits: 1,
        amplitude: 0.25,
        frequencia: 24.0,
        decaimento: 1.5,
        expoente: 2,
        semente: 7,
        trauma: 0.0,
        activa: true,
        clock_playing: false,
        selected_count: 1,
    }
}

/// ⭐ A secção TAGS na fixtura.
///
/// ⛔ **A premissa que a trouxe MORREU em 2026-09-22.** Ela dizia *«ela é a ÚLTIMA da cadeia desde
/// a W3a, e é por isso que entra aqui: a que fecha a lista é a única cujo `y` ninguém consome»* —
/// com a ordem por FAMÍLIAS a TAGS passou a ser a **PRIMEIRA** das opcionais, logo o `y` dela é
/// consumido por todas as outras. Ela fica: *o papel trocou-se de «a que ninguém consome» para «a
/// que alimenta todas», e nos dois casos ela é a fronteira que o gate precisa de ver.*
fn tags() -> ph2d_editor_core::screens::hero::InspectorTagsInfo {
    let linha = |id: u64, path: &str, label: &str, depth: usize| {
        ph2d_editor_core::screens::hero::InspectorTagRow {
            id,
            path: path.into(),
            label: label.into(),
            depth,
        }
    };
    ph2d_editor_core::screens::hero::InspectorTagsInfo {
        entity_bits: 1,
        on_object: vec![linha(1, "Enemy", "Enemy", 0)],
        full: false,
        selected_count: 1,
    }
}

/// Pinta o painel **pela porta do produto** e devolve as bandas de cabeçalho das secções vivas,
/// na ordem em que foram registadas.
///
/// ⚠️ **O oráculo é o índice de acerto**, e não uma conta paralela: é ele que o despacho consulta,
/// e é ali que uma sobreposição de facto acontece.
fn section_bands() -> Vec<(&'static str, Rect)> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_transform(Some(transform()));
    set_current_inspector_timer(Some(timers()));
    set_current_inspector_action(Some(actions()));
    set_current_inspector_camera(Some(camera()));
    set_current_inspector_shake(Some(shake()));
    set_current_inspector_tags(Some(tags()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_transform(None);
    set_current_inspector_timer(None);
    set_current_inspector_action(None);
    set_current_inspector_camera(None);
    set_current_inspector_shake(None);
    set_current_inspector_tags(None);

    // ⭐⭐⭐ **A ORDEM DESTA LISTA É A ORDEM DE LEITURA DO PAINEL, e ela mudou em 2026-09-22.**
    //
    // Até aí era `Transform · Timers · Signal Actions · Camera · Tags` — a ordem em que as secções
    // foram CONSTRUÍDAS. Hoje é a das **famílias do catálogo** (a mesma tabela por que a paleta
    // *Add Component* agrupa): o bloco FIXO primeiro, depois IDENTIDADE (Tags), LÓGICA (Timers,
    // Signal Actions) e SAÍDA (Camera, Shake).
    //
    // ⚠️ **Ela é escrita à mão de propósito, e isso NÃO é a segunda resposta à mesma pergunta:** a
    // ordem é DERIVADA de `ComponentCategory::ALL` pelo
    // `a_ordem_das_seccoes_e_a_da_paleta::as_seccoes_opcionais_saem_por_familia_na_ordem_da_paleta`
    // (`ph2d-panel-registry-init`, sobre as 24 secções opcionais). Esta lista é o **CONTROLO** dele,
    // na crate do próprio painel e sem o arnês do registo — se as duas discordarem, uma fica
    // vermelha em voz alta. *O que este gate cobre e aquele não é o bloco FIXO contra o opcional.*
    let nomes: [(&str, ph2d_a11y::NodeId); 6] = [
        ("Transform", ids::INSP_LIVE_TRANSFORM_SECTION),
        ("Tags", ids::INSP_LIVE_TAGS_SECTION),
        ("Timers", ids::INSP_LIVE_TIMER_SECTION),
        ("Signal Actions", ids::INSP_LIVE_ACTION_SECTION),
        ("Camera", ids::INSP_LIVE_CAMERA_SECTION),
        ("Shake", ids::INSP_LIVE_SHAKE_SECTION),
    ];
    let mut out = Vec::new();
    for (nome, id) in nomes {
        if let Some((_, r)) = rects.iter().find(|(n, _)| *n == id).copied() {
            out.push((nome, r));
        }
    }
    out
}

/// ⭐⭐⭐ **As bandas de cabeçalho de duas secções vivas NÃO se cruzam.**
///
/// **Mutação que deve sangrar:** trocar um `y = paint_<x>_section(…)` por `paint_<x>_section(…)`
/// no `paint_stateful`.
#[test]
fn two_live_sections_never_share_a_band() {
    let bandas = section_bands();
    assert!(
        bandas.len() >= 6,
        "a fixtura nao produziu as seis seccoes: {:?}",
        bandas.iter().map(|(n, _)| *n).collect::<Vec<_>>()
    );
    for i in 0..bandas.len() {
        for j in (i + 1)..bandas.len() {
            let (na, a) = bandas[i];
            let (nb, b) = bandas[j];
            let cruza = a.y < b.y + b.h && b.y < a.y + a.h;
            assert!(
                !cruza,
                "os cabecalhos de '{na}' (y {:.1}..{:.1}) e '{nb}' (y {:.1}..{:.1}) ocupam a MESMA \
                 banda — as duas seccoes desenham-se uma por cima da outra",
                a.y,
                a.y + a.h,
                b.y,
                b.y + b.h,
            );
        }
    }
}

/// **E elas descem na ORDEM em que se pintam** — a de baixo começa depois de a de cima acabar.
///
/// ⚠️ A metade que a anterior não cobre: duas bandas podem não se cruzar e ainda assim estar
/// trocadas, o que faria a lista de uma secção aparecer debaixo do cabeçalho da outra.
///
/// ⛔ **A ordem que ele afirma MUDOU em 2026-09-22** (ver o comentário sobre `nomes`): ela deixou
/// de ser a de construção e passou a ser a das famílias do catálogo. *A propriedade é a mesma; o
/// que envelheceu foi a lista.*
#[test]
fn each_section_starts_below_the_one_before_it() {
    let bandas = section_bands();
    for par in bandas.windows(2) {
        let (na, a) = par[0];
        let (nb, b) = par[1];
        assert!(
            b.y >= a.y + a.h,
            "'{nb}' comeca em {:.1} e '{na}' so' acaba em {:.1} — a ordem de pintura e a ordem no \
             ecra discordam",
            b.y,
            a.y + a.h,
        );
    }
}
