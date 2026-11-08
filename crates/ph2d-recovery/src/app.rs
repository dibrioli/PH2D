//! A recuperação inteira de um processo, para a shell só COMPOR: arranque (sessão + varredura),
//! a hora, a oferta dos bytes, o Save, o `Restore` e o fecho.

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{Autosave, Choice, Meta, Offer, Period, Prompt, Session, now_secs, scan};

/// O estado da recuperação que a shell guarda.
#[derive(Debug)]
pub struct Recovery {
    /// A sessão viva (`None` = o sistema não deu pasta: não há recuperação, e diz-se no terminal).
    pub session: Option<Session>,
    /// O relógio.
    pub autosave: Autosave,
    /// A janela do arranque.
    pub prompt: Prompt,
}

impl Recovery {
    /// Sem pasta nem sessão (os testes da shell, o headless).
    #[must_use]
    pub fn off() -> Self {
        Self {
            session: None,
            autosave: Autosave::new(Period::default(), Instant::now()),
            prompt: Prompt::default(),
        }
    }

    /// Arranca em `root`: abre a sessão deste processo e varre as mortas (poda incluída).
    #[must_use]
    pub fn start(root: Option<&Path>, period: Period) -> Self {
        let mut r = Self::off();
        r.autosave = Autosave::new(period, Instant::now());
        let Some(root) = root else {
            eprintln!("[recovery] sem pasta de dados: a recuperação fica desligada");
            return r;
        };
        match Session::start(root, now_secs(), std::process::id()) {
            Ok(s) => {
                r.prompt.entries = scan(root, now_secs(), Some(s.dir()), &crate::is_alive);
                r.session = Some(s);
            }
            Err(e) => eprintln!("[recovery] a sessão não abriu em {}: {e}", root.display()),
        }
        r
    }

    /// Chegou a hora de serializar?
    #[must_use]
    pub fn due(&self, now: Instant) -> bool {
        self.session.is_some() && self.autosave.due(now)
    }

    /// Os bytes do Save, à hora: grava a cópia se mudaram.
    pub fn offer(&mut self, bytes: Vec<u8>, project: Option<&str>, now: Instant) -> Offer {
        let Some(s) = self.session.as_ref() else {
            return Offer::Unchanged;
        };
        self.autosave.offer(bytes, meta_of(project), s, now)
    }

    /// O Save gravou `bytes`: a cópia sai.
    pub fn saved(&mut self, bytes: &[u8], now: Instant) {
        if let Some(s) = self.session.as_ref() {
            self.autosave.mark_saved(bytes, s, now);
        }
    }

    /// Responde à linha `i` da janela. `Restore` devolve `(cópia a abrir, caminho original)`: a
    /// cópia já é desta sessão e a impressão foi esquecida (o projecto fica por gravar).
    pub fn answer(&mut self, i: usize, choice: Choice) -> Option<(PathBuf, Option<String>)> {
        let (orphan, choice) = self.prompt.answer(i, choice)?;
        if choice != Choice::Restore {
            return None;
        }
        let s = self.session.as_ref()?;
        self.autosave.finish();
        match s.adopt(&orphan) {
            Ok(_) => {
                self.autosave.forget();
                let path = Some(orphan.meta.path.clone()).filter(|p| !p.is_empty());
                Some((s.copy_path(), path))
            }
            Err(e) => {
                eprintln!("[recovery] Restore falhou: {e}");
                None
            }
        }
    }

    /// Pergunta cada cópia da janela, a mais nova primeiro, por `ask`. Um projecto aberto de cada
    /// vez: o primeiro `Restore` ganha e as restantes ficam para depois (`Later`).
    pub fn ask_all(
        &mut self,
        mut ask: impl FnMut(&crate::Orphan) -> Choice,
    ) -> Option<(PathBuf, Option<String>)> {
        while let Some(o) = self.prompt.entries.first() {
            let choice = ask(o);
            if let Some(restored) = self.answer(0, choice) {
                self.prompt.later_all();
                return Some(restored);
            }
        }
        None
    }

    /// O fecho normal: espera a escrita e apaga a sessão.
    pub fn close(&mut self) {
        self.autosave.finish();
        if let Some(s) = self.session.take() {
            s.close();
        }
    }
}

/// O meta da cópia a partir do caminho do projecto.
#[must_use]
pub fn meta_of(project: Option<&str>) -> Meta {
    let path = project.unwrap_or_default().to_owned();
    let name = Path::new(&path).file_name().map_or_else(
        || "Untitled".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    Meta {
        name,
        path,
        saved: now_secs(),
    }
}
