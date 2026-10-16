//! ⭐ **E6 — Ctrl+S aplica o script E grava o projeto** (decisão do dono, opção «a»): o texto vai para a
//! `ScriptLibrary`, a cena recarrega no quadro seguinte e o selo diz quantos objectos; um texto
//! partido é recusado e a versão anterior continua. ⛔ O Ctrl+S da aba NUNCA abre diálogo (smoke do
//! dono 16/10): com ficheiro de projecto o pedido de gravar sobe pela MESMA bandeira do File ▸ Save
//! (que então grava em silêncio); sem ficheiro aplica e o selo diz que o projecto não foi gravado.

use crate::{Mesa, ctrl};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_panel_code::{Badge, Severity};

const BOB: &str = "function update(self, dt)\nend\n";

fn salva(m: &mut Mesa, id: ph2d_script::ScriptId) {
    m.tecla(id, Key::Char('s'), ctrl(), Some("s"));
    m.quadro();
}

#[test]
fn ctrl_s_aplica_o_texto_grava_o_projeto_e_o_selo_conta_os_objectos() {
    let (m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let mut m = m.com_cena(ids[0], 3);
    m.ficheiro = Some(std::env::temp_dir().join("nunca_escrito.ph2dproj")); // o projecto tem ficheiro
    m.cena(); // a 1.ª leitura (não é recarga)
    m.pede(ids[0]);
    m.escreve(ids[0], "-- novo\n");
    assert!(m.docs().is_dirty(ids[0]));
    assert!(!m.hero.file_menu.save);
    salva(&mut m, ids[0]);
    let texto = m.docs().text_of(ids[0]).expect("aberto");
    assert_eq!(
        m.texto_salvo(ids[0]),
        texto,
        "o texto foi para a biblioteca"
    );
    assert!(
        m.hero.file_menu.save,
        "com ficheiro, o projeto é gravado pela bandeira do File ▸ Save"
    );
    assert!(!m.docs().is_dirty(ids[0]), "salvo: o ● sai");
    m.cena(); // a cena recarrega a partir da biblioteca
    m.quadro();
    assert_eq!(
        m.docs().badge_of(ids[0]),
        Some(Badge::Reloaded { objects: 3 })
    );

    // Gravar SEM mudar nada ainda dá o selo (o editor pede a recarga).
    m.hero.file_menu.save = false;
    salva(&mut m, ids[0]);
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().badge_of(ids[0]),
        Some(Badge::Reloaded { objects: 3 })
    );
    // Ctrl+Shift+S é o «Save as».
    let shift = Modifiers {
        shift: true,
        ..ctrl()
    };
    m.tecla(ids[0], Key::Char('s'), shift, Some("S"));
    m.quadro();
    assert!(m.hero.file_menu.save_as);
}

#[test]
fn um_texto_partido_e_recusado_e_a_linha_fica_marcada() {
    let (m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let mut m = m.com_cena(ids[0], 2);
    m.cena();
    m.pede(ids[0]);
    // Apaga o `end` (o smoke do dono): Ctrl+End, Shift+↑ selecciona a última linha, Delete.
    m.tecla(ids[0], Key::End, ctrl(), None);
    m.tecla(
        ids[0],
        Key::Up,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        None,
    );
    m.tecla(ids[0], Key::Delete, Modifiers::default(), None);
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some("function update(self, dt)\n")
    );
    salva(&mut m, ids[0]);
    m.cena();
    m.quadro();
    let Some(Badge::Refused { line }) = m.docs().badge_of(ids[0]) else {
        panic!("sem selo de recusa: {:?}", m.docs().badge_of(ids[0]))
    };
    assert!(line.is_some(), "a recusa diz a linha");
    assert!(
        m.host.as_ref().unwrap().scene().runs_previous(ids[0]),
        "a versão anterior continua a correr"
    );
    let p = m.docs().problems_of(ids[0]);
    assert!(
        p.iter().any(|p| p.severity == Severity::Error),
        "a calha marca o erro: {p:?}"
    );
    // Controlo: corrigir volta a recarregar.
    m.escreve(ids[0], "end\n");
    salva(&mut m, ids[0]);
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().badge_of(ids[0]),
        Some(Badge::Reloaded { objects: 2 })
    );
    assert!(m.docs().problems_of(ids[0]).is_empty());
}

/// Um script que nenhum objecto corre: gravado, e o selo di-lo (não há recarga a relatar).
#[test]
fn sem_objectos_o_selo_diz_gravado() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.escreve(ids[0], "-- x\n");
    salva(&mut m, ids[0]);
    assert_eq!(m.docs().badge_of(ids[0]), Some(Badge::Saved));
}

/// File ▸ Save com a aba de código ACTIVA aplica-a; ⭐ e com a Cena activa também aplica as abas com
/// ● (decisão do dono 15/10: «salvar = guardado» — era a lei oposta até à Onda 1).
#[test]
fn o_save_do_menu_aplica_a_aba_activa() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.escreve(ids[0], "-- x\n");
    m.hero.documents.activate(None);
    m.hero.file_menu.save = true;
    m.quadro();
    assert_ne!(
        m.texto_salvo(ids[0]),
        BOB,
        "com a Cena activa a aba com ● é aplicada"
    );
    assert!(!m.docs().is_dirty(ids[0]));
    m.pede(ids[0]); // volta a activar a aba
    m.escreve(ids[0], "-- y\n");
    m.hero.file_menu.save = true;
    m.quadro();
    assert!(m.texto_salvo(ids[0]).contains("-- y"));
    assert!(!m.docs().is_dirty(ids[0]));
}

/// ⛔ File ▸ Save aplica também as abas FECHADAS com ● (a lista pinta-as com ●): editar → fechar no
/// × → File ▸ Save ⇒ o texto está na biblioteca — e nos bytes que vão para o ficheiro.
#[test]
fn o_save_do_menu_aplica_as_abas_fechadas_com_alteracoes() {
    use ph2d_editor_core::doc_surface::DocumentSurface;
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    m.pede(ids[0]);
    m.escreve(ids[0], "-- x\n");
    let escrito = m.docs().text_of(ids[0]).expect("aberto");
    m.docs().close(ids[0].0);
    m.quadro();
    assert!(m.docs().is_parked(ids[0]));
    m.hero.file_menu.save = true;
    m.quadro();
    assert_eq!(
        m.texto_salvo(ids[0]),
        escrito,
        "a aba fechada com ● não foi aplicada"
    );
    let bytes = m
        .sim
        .world()
        .resource::<ph2d_script::ProjectScripts>()
        .to_bytes();
    let gravado = ph2d_script::ScriptLibrary::from_bytes(&bytes).expect("lê");
    assert_eq!(
        gravado.get(ids[0]).map(|d| d.text.as_str()),
        Some(escrito.as_str())
    );
    assert!(!m.docs().is_dirty(ids[0]), "gravado: o ● sai da lista");
}

/// ⛔ Smoke do dono (16/10): «Ctrl+S, em vez de salvar o script já existente, abre o diálogo para
/// salvar». Na cena sem ficheiro de projecto o Ctrl+S da aba levantava o File ▸ Save, e a shell
/// pergunta onde gravar o primeiro save. ⭐ Lei: o Ctrl+S da aba NUNCA abre diálogo — aplica o
/// texto e o selo diz que o projecto ainda não está gravado.
#[test]
fn ctrl_s_sem_ficheiro_aplica_e_nao_abre_dialogo() {
    let (m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let mut m = m.com_cena(ids[0], 3);
    m.cena();
    m.pede(ids[0]);
    m.escreve(ids[0], "-- novo\n");
    salva(&mut m, ids[0]);
    m.shell_drena();
    assert_eq!(m.dialogos, 0, "o Ctrl+S da aba abriu o diálogo «Save as»");
    let texto = m.docs().text_of(ids[0]).expect("aberto");
    assert_eq!(
        m.texto_salvo(ids[0]),
        texto,
        "o texto foi para a biblioteca"
    );
    assert!(!m.docs().is_dirty(ids[0]), "aplicado: o ● sai");
    assert_eq!(
        m.docs().badge_text_of(ids[0]).as_deref(),
        Some("Script applied · project not saved yet — File ▸ Save As…")
    );
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().badge_of(ids[0]),
        Some(Badge::Reloaded { objects: 3 }),
        "a cena recarregou"
    );
    assert_eq!(
        m.docs().badge_text_of(ids[0]).as_deref(),
        Some("Reloaded · 3 objects · project not saved yet — File ▸ Save As…")
    );
    // Controlo: o projecto ganhou ficheiro (File ▸ Save As…) — o aviso sai.
    m.ficheiro = Some(std::env::temp_dir().join("nunca_escrito.ph2dproj"));
    m.quadro();
    assert_eq!(
        m.docs().badge_text_of(ids[0]).as_deref(),
        Some("Reloaded · 3 objects")
    );
}

/// Com o projecto JÁ num ficheiro, o Ctrl+S da aba aplica E grava, sem diálogo: o ficheiro em
/// disco tem o texto novo.
#[test]
fn ctrl_s_com_ficheiro_aplica_e_grava_sem_dialogo() {
    let dir = std::env::temp_dir().join(format!("ph2d_code_ctrl_s_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("pasta");
    let caminho = dir.join("hero.ph2dproj");
    let (m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let mut m = m.com_cena(ids[0], 3);
    m.ficheiro = Some(caminho.clone());
    m.cena();
    m.pede(ids[0]);
    m.escreve(ids[0], "-- novo\n");
    salva(&mut m, ids[0]);
    m.shell_drena();
    assert_eq!(m.dialogos, 0, "com ficheiro, gravar não pergunta");
    let texto = m.docs().text_of(ids[0]).expect("aberto");
    let bytes = std::fs::read(&caminho).expect("o projecto foi gravado em disco");
    let gravado = ph2d_script::ScriptLibrary::from_bytes(&bytes).expect("lê");
    assert_eq!(
        gravado.get(ids[0]).map(|d| d.text.as_str()),
        Some(texto.as_str()),
        "o ficheiro em disco tem o texto novo"
    );
    m.cena();
    m.quadro();
    assert_eq!(
        m.docs().badge_text_of(ids[0]).as_deref(),
        Some("Reloaded · 3 objects"),
        "gravado: o selo não fala de projecto por gravar"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
