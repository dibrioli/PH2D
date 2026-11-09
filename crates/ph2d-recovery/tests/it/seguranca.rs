//! A segurança dos dados da recuperação (auditoria do fecho da linha P12, lote R).

use std::time::{Duration, Instant};

use ph2d_recovery::{Choice, Period, Recovery, Session, is_held, now_secs, scan};

use super::{meta, tmpdir};

/// Uma sessão morta (a tranca solta) com `bytes` na cópia.
fn morta(root: &std::path::Path, started: u64, bytes: &[u8]) -> std::path::PathBuf {
    let s = Session::start(root, started, 7).expect("s");
    s.write_copy(bytes, &meta(now_secs())).expect("w");
    let dir = s.dir().to_path_buf();
    s.leave();
    dir
}

/// R1: o `Restore` recusado guarda a cópia e volta à janela; só o aceite apaga a pasta morta, com
/// a hora refrescada.
#[test]
fn um_restore_recusado_guarda_a_copia_e_volta_a_janela() {
    let root = tmpdir("restore-refused");
    let dir = morta(&root, 1, b"orfa");
    let mut r = Recovery::start(Some(&root), Period::default());
    assert_eq!(r.prompt.entries.len(), 1);
    let (copy, path) = r.answer(0, Choice::Restore).expect("restore");
    assert_eq!(std::fs::read(&copy).expect("r"), b"orfa");
    assert_eq!(path.as_deref(), Some("/home/x/Hero.ph2dproj"));
    assert!(dir.exists(), "responder não apaga nada antes de abrir");
    assert!(
        !r.restored(false, None),
        "recusado: a shell não adopta o caminho"
    );
    assert!(dir.exists(), "a cópia recusada fica no disco");
    assert_eq!(r.prompt.entries.len(), 1, "e volta à janela");
    assert_eq!(r.take_notices().len(), 1, "o artista é avisado");
    let t0 = now_secs();
    r.answer(0, Choice::Restore).expect("de novo");
    assert!(r.restored(true, Some(b"orfa")));
    assert!(!dir.exists() && !r.prompt.is_open());
    let mine = r.session.as_ref().expect("s").dir().join("project.meta");
    let text = std::fs::read_to_string(mine).expect("meta");
    let saved: u64 = text
        .lines()
        .find_map(|l| l.strip_prefix("saved="))
        .and_then(|v| v.parse().ok())
        .expect("saved");
    assert!(saved >= t0, "a cópia adoptada tem a hora de agora");
    r.close();
    let _ = std::fs::remove_dir_all(root);
}

/// R3: sem bytes (a serialização recusou) o fecho pergunta, e um fecho sem decisão guarda a cópia;
/// só o fecho limpo apaga.
#[test]
fn o_estado_desconhecido_pergunta_e_o_fecho_sem_decisao_guarda_a_copia() {
    let root = tmpdir("quit-unknown");
    let mut r = Recovery::start(Some(&root), Period::default());
    r.offer(b"obra".to_vec(), Some("/x/A.ph2dproj"), Instant::now());
    r.autosave.finish();
    assert!(r.ask_quit(None, None), "estado desconhecido = por gravar");
    let dir = r.session.as_ref().expect("s").dir().to_path_buf();
    r.close();
    assert!(dir.join("project.ph2dproj").is_file(), "a cópia ficou");
    let mut r = Recovery::start(Some(&root), Period::default());
    r.prompt.later_all();
    r.saved(b"obra", Instant::now());
    assert!(!r.ask_quit(Some(b"obra"), None), "nada por gravar");
    let mine = r.session.as_ref().expect("s").dir().to_path_buf();
    r.close();
    assert!(!mine.exists(), "o fecho limpo apaga a sessão");
    let _ = std::fs::remove_dir_all(root);
}

/// R4 + R5: viva = a tranca do SO, nunca o `pid`. Um `pid` vivo (o 1) numa pasta sem tranca é uma
/// morta; um `pid` que não existe numa pasta trancada é uma viva.
#[test]
fn a_vida_e_a_tranca_do_so_nao_o_pid() {
    let root = tmpdir("lock");
    let solta = Session::start(&root, 1, 1).expect("a");
    solta.write_copy(b"morta", &meta(now_secs())).expect("w");
    solta.leave();
    let presa = Session::start(&root, 2, 4_000_000).expect("b");
    presa.write_copy(b"viva", &meta(now_secs())).expect("w");
    assert!(is_held(presa.dir()));
    let found = scan(&root, now_secs(), None);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(std::fs::read(&found[0].copy).expect("r"), b"morta");
    presa.close();
    let _ = std::fs::remove_dir_all(root);
}

/// R7: a escrita que falha deixa o trabalho por copiar (a próxima hora tenta de novo) e avisa o
/// artista uma vez.
#[test]
fn a_copia_que_falha_tenta_de_novo_e_avisa_uma_vez() {
    let root = tmpdir("fail");
    let mut r = Recovery::start(Some(&root), Period::from_secs(10));
    r.prompt.later_all();
    let t0 = Instant::now();
    r.input(true, t0);
    let falha = || Some((|| None::<Vec<u8>>, None::<String>));
    r.tick(10, t0 + Duration::from_secs(11), falha);
    while !r.autosave.finished() {
        std::thread::yield_now();
    }
    assert_eq!(r.take_notices().len(), 1, "avisa");
    assert!(r.touched, "continua por copiar");
    assert!(
        r.ready(10, t0 + Duration::from_secs(22)),
        "a próxima hora tenta"
    );
    r.tick(10, t0 + Duration::from_secs(22), falha);
    while !r.autosave.finished() {
        std::thread::yield_now();
    }
    assert!(r.take_notices().is_empty(), "uma vez por série");
    r.close();
    let _ = std::fs::remove_dir_all(root);
}

/// R9: sem `project.meta` a hora é a do ficheiro da cópia — a cópia de agora oferece-se, nunca se
/// poda como se tivesse uma semana.
#[test]
fn a_copia_sem_meta_usa_a_hora_do_ficheiro() {
    let root = tmpdir("no-meta");
    let dir = morta(&root, 1, b"sem meta");
    std::fs::remove_file(dir.join("project.meta")).expect("rm");
    let found = scan(&root, now_secs(), None);
    assert_eq!(found.len(), 1, "a cópia de agora fica");
    assert!(now_secs().saturating_sub(found[0].meta.saved) < 60);
    let _ = std::fs::remove_dir_all(root);
}

/// R13: cada opção do submenu Settings ▸ Recovery é EXACTAMENTE um intervalo do relógio.
#[test]
fn cada_opcao_do_menu_e_um_intervalo() {
    let menu = ph2d_editor_core::recovery_pref::OPTIONS_SECS;
    assert_eq!(menu[0], 0, "a primeira é Off");
    assert!(Period::from_secs(0).is_off());
    assert_eq!(
        &menu[1..],
        &Period::OPTIONS_SECS[..],
        "as mesmas, pela mesma ordem"
    );
    for s in &menu[1..] {
        assert_eq!(Period::from_secs(*s).duration().as_secs(), *s);
    }
    let fabrica = ph2d_editor_core::recovery_pref::DEFAULT_SECS;
    assert_eq!(Period::from_secs(fabrica), Period::default());
}
