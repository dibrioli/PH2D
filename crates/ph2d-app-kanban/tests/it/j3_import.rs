//! J3 — «Import from Trello…» e «Export Kanban…» pelo menu File do hero (`apply_event`), com os
//! diálogos respondidos por `KanbanDocs::answer_next_dialogs` (a MESMA função que o diálogo nativo
//! alimenta): o Kanban importado abre na aba, o relatório diz o que se perdeu, e os três formatos
//! gravam-se no caminho escolhido.

use crate::Mesa;
use ph2d_app_kanban::KanbanDocs;
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_host::Modifiers;
use ph2d_kanban_import::{Format, LossKind, import_trello};
use ph2d_kanban_model::KanbanId;
use std::path::PathBuf;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ph2d-kanban-import/tests/fixtures/trello_board_synthetic.json"
);
const NOME: &str = "Produção do curta «Lua»";

fn pasta(caso: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d-kanban-j3-{}-{caso}", std::process::id()));
    std::fs::create_dir_all(&d).expect("pasta");
    d
}

/// O menu File → «Import from Trello…», com estas respostas aos diálogos.
fn importa(m: &mut Mesa, respostas: Vec<Option<PathBuf>>) -> Option<KanbanId> {
    KanbanDocs::answer_next_dialogs(respostas);
    assert!(
        m.hero
            .apply_event(WidgetEvent::Click(KanbanDocs::CMD_IMPORT)),
        "o comando foi tomado"
    );
    serve(m);
    m.hero
        .documents
        .kanbans()
        .iter()
        .find(|k| k.header.name == NOME)
        .map(|k| k.id)
}

/// Dois quadros e a fila do mundo (os avisos chegam ao topo pelo `service`, como na shell).
fn serve(m: &mut Mesa) {
    m.quadros(2);
    ph2d_app_kanban::service(&mut m.hero);
}

fn relatorio(m: &mut Mesa, id: KanbanId) -> Option<Vec<String>> {
    m.docs().probe_import_report(id.0)
}

#[test]
fn j3_import_abre_a_aba_e_o_relatorio_diz_o_que_se_perdeu() {
    let mut m = Mesa::new();
    m.quadro();
    let comandos: Vec<_> = m
        .docs()
        .family_commands()
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(comandos.contains(&ph2d_i18n::tr("kanban.import.cmd_import").to_owned()));
    assert!(comandos.contains(&ph2d_i18n::tr("kanban.import.cmd_export").to_owned()));
    // O quadro tem anexos: a pasta pergunta-se, e cancelar = entram como links.
    let id = importa(&mut m, vec![Some(FIXTURE.into()), None]).expect("o Kanban importado");
    assert_eq!(
        m.activa(),
        Some(id.0),
        "a aba activa é a do Kanban importado"
    );
    let k = m.kanban(id);
    assert_eq!(k.lists.values().filter(|l| !l.archived).count(), 3);
    let texto = relatorio(&mut m, id).expect("o relatório abre sobre a aba");
    assert!(
        texto.iter().all(|t| !t.contains("kanban.import.")),
        "chave crua: {texto:?}"
    );
    assert!(texto.contains(&ph2d_i18n::tr("kanban.import.report_title").to_owned()));
    assert!(
        texto.iter().any(|t| t.contains(NOME)),
        "o nome no relatório"
    );
    assert!(texto.contains(&ph2d_i18n::tr_with(
        "kanban.import.count.cards",
        &[("n", &9)]
    )));
    // Cada perda da importação aparece agrupada pelo porquê, com a contagem.
    let imp = import_trello(&std::fs::read(FIXTURE).expect("fixture"), KanbanId(1), None)
        .expect("quadro");
    assert!(!imp.report.losses.is_empty());
    let mut tipos: Vec<LossKind> = imp.report.losses.iter().map(|l| l.kind).collect();
    tipos.sort_unstable();
    tipos.dedup();
    let grupos = texto
        .iter()
        .filter(|s| s.contains(") (") || s.ends_with(')'))
        .count();
    assert!(grupos >= tipos.len(), "um grupo por porquê: {texto:?}");
    let power_ups = ph2d_i18n::tr_with(
        "kanban.import.loss_group",
        &[
            ("what", &ph2d_i18n::tr("kanban.import.loss.power_up")),
            ("n", &imp.report.count(LossKind::PowerUp)),
        ],
    );
    assert!(texto.contains(&power_ups), "{power_ups} em {texto:?}");
    assert!(
        texto.contains(&ph2d_i18n::tr_with(
            "kanban.import.losses_caption",
            &[("n", &imp.report.losses.len())]
        )),
        "o total de perdas"
    );
    // `Esc` fecha o relatório; a aba fica.
    assert!(m.tecla(BoardKey::Escape, Modifiers::default(), None));
    m.quadro();
    assert_eq!(relatorio(&mut m, id), None);
    assert_eq!(m.activa(), Some(id.0));
}

#[test]
fn j3_import_com_a_pasta_traz_os_ficheiros_e_um_clique_fora_fecha() {
    let mut m = Mesa::new();
    m.quadro();
    let dir = pasta("pasta");
    std::fs::write(dir.join("storyboard.pdf"), b"%PDF storyboard").expect("w");
    let id = importa(&mut m, vec![Some(FIXTURE.into()), Some(dir.clone())]).expect("importado");
    let texto = relatorio(&mut m, id).expect("relatório");
    assert!(
        texto.contains(&ph2d_i18n::tr_with(
            "kanban.import.count.files",
            &[("n", &1)]
        )),
        "{texto:?}"
    );
    assert!(m.hero.documents.kanbans().iter().count() >= 1);
    // Um clique no véu (fora do relatório) fecha-o.
    m.clica((2.0, 2.0));
    assert_eq!(relatorio(&mut m, id), None);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn j3_import_ficheiro_que_nao_e_quadro_avisa_e_nao_cria_nada() {
    let mut m = Mesa::new();
    m.quadro();
    let dir = pasta("mau");
    let mau = dir.join("notas.json");
    std::fs::write(&mau, b"isto nao e json").expect("w");
    assert_eq!(importa(&mut m, vec![Some(mau.clone())]), None);
    assert_eq!(m.hero.documents.kanbans().iter().count(), 0);
    let dito = m.hero.documents.told().expect("um aviso").to_owned();
    assert!(dito.contains("notas.json"), "{dito}");
    // Cancelar o diálogo não faz nada nem avisa.
    assert_eq!(importa(&mut m, vec![None]), None);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn j3_import_export_grava_os_tres_formatos_pela_extensao() {
    let mut m = Mesa::new();
    m.quadro();
    let id = importa(&mut m, vec![Some(FIXTURE.into()), None]).expect("importado");
    let dir = pasta("export");
    let casos = [
        ("lua.csv", Format::Csv, "lua.csv"),
        ("lua.md", Format::Markdown, "lua.md"),
        ("lua.json", Format::TrelloJson, "lua.json"),
        // Sem extensão: o 1.º formato do diálogo (CSV), e a extensão junta-se.
        ("lua", Format::Csv, "lua.csv"),
    ];
    for (escolha, formato, gravado) in casos {
        let alvo = dir.join(gravado);
        let _ = std::fs::remove_file(&alvo);
        KanbanDocs::answer_next_dialogs(vec![Some(dir.join(escolha))]);
        assert!(
            m.hero
                .apply_event(WidgetEvent::Click(KanbanDocs::CMD_EXPORT))
        );
        serve(&mut m);
        let esperado =
            ph2d_kanban_import::export(&m.hero.documents.kanbans(), id, formato).expect("export");
        let lido = std::fs::read_to_string(&alvo).unwrap_or_else(|_| panic!("{gravado} gravado"));
        assert_eq!(lido, esperado, "{escolha}");
        let dito = m.hero.documents.told().expect("aviso").to_owned();
        assert!(dito.contains(&alvo.display().to_string()), "{dito}");
    }
    // O JSON exportado volta a importar o mesmo quadro.
    let json = std::fs::read(dir.join("lua.json")).expect("json");
    assert!(import_trello(&json, KanbanId(1), None).is_ok());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn j3_import_export_sem_kanban_aberto_avisa() {
    let mut m = Mesa::new();
    m.quadro();
    KanbanDocs::answer_next_dialogs(vec![]);
    assert!(
        m.hero
            .apply_event(WidgetEvent::Click(KanbanDocs::CMD_EXPORT))
    );
    serve(&mut m);
    assert_eq!(
        m.hero.documents.told(),
        Some(ph2d_i18n::tr("kanban.import.export_none"))
    );
}

/// Os caminhos que o hero INTEIRO pinta neste quadro (o topo, a aba, os avisos).
fn caminhos_do_hero(m: &mut Mesa) -> u32 {
    use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
    let vp = ph2d_editor_core::zones::Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    let mut s = ph2d_vector::VectorScene::new();
    paint_hero_screen(&mut m.hero, vp, &mut s, &mut m.ts);
    s.inner().encoding().n_paths
}

/// D9 (10.3) — o aviso «Exported … as CSV to …» pinta-se POR CIMA da aba do Kanban: os avisos não
/// são só do ecrã do Quadro.
#[test]
fn j3_import_o_aviso_do_export_aparece_sobre_a_aba_do_kanban() {
    let (mut m, _) = Mesa::cena1();
    let antes = caminhos_do_hero(&mut m);
    assert_eq!(antes, caminhos_do_hero(&mut m), "a pintura é estável");
    assert!(m.hero.documents.told().is_none(), "nada dito ainda");
    let dir = pasta("aviso");
    KanbanDocs::answer_next_dialogs(vec![Some(dir.join("lua.csv"))]);
    assert!(
        m.hero
            .apply_event(WidgetEvent::Click(KanbanDocs::CMD_EXPORT))
    );
    serve(&mut m);
    assert!(
        m.hero.documents.surface_active().is_some(),
        "a aba do Kanban continua activa"
    );
    assert!(
        m.hero
            .documents
            .told()
            .is_some_and(|t| t.contains("lua.csv"))
    );
    let depois = caminhos_do_hero(&mut m);
    assert!(
        depois > antes,
        "o balão do aviso foi pintado ({antes} → {depois})"
    );
    let _ = std::fs::remove_dir_all(dir);
}
