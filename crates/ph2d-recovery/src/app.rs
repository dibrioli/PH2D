//! A recuperação inteira de um processo, para a shell só COMPOR: arranque (sessão + varredura),
//! a hora, a oferta dos bytes, o Save, o `Restore` e o fecho.

use std::path::{Path, PathBuf};
use std::time::Instant;

use std::time::Duration;

use crate::{Autosave, Choice, Meta, Period, Prompt, Session, now_secs, scan};

/// A pausa que conta como «o artista parou» (sem entrada nem botão premido).
pub const IDLE: Duration = Duration::from_secs(1);

/// O estado da recuperação que a shell guarda.
#[derive(Debug)]
pub struct Recovery {
    /// A sessão viva (`None` = o sistema não deu pasta: não há recuperação, e diz-se no terminal).
    pub session: Option<Session>,
    /// O relógio.
    pub autosave: Autosave,
    /// A janela do arranque.
    pub prompt: Prompt,
    /// A pergunta do fecho com alterações por gravar.
    pub quit: Quit,
    /// Houve entrada do artista desde a última serialização? Sem ela a hora NÃO serializa (com o
    /// app parado custa zero). Medido: 128 MiB de Pixel custam codec 84 ms + cópia 22 ms no fio da
    /// janela (a impressão já é do fio) — por isso também se espera uma PAUSA ([`Self::due`]).
    pub touched: bool,
    /// A última entrada do artista (o gesto em curso adia a hora).
    last_input: Option<Instant>,
    /// Um botão do ponteiro está premido (um traço em curso, mesmo parado).
    held: bool,
    /// O último quadro não teve entrada.
    quiet: bool,
    /// Um traço acabou desde a última hora (o quadro calmo seguinte já é pausa).
    released: bool,
    /// Onde a janela se pintou no último quadro (o ecrã e a escala do chrome): o clique lê daqui.
    pub screen: Option<(
        ph2d_editor_core::zones::Rect,
        ph2d_editor_core::ui_scale::UiScaleMap,
    )>,
}

/// O fecho da janela do app com alterações por gravar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Quit {
    /// Nada pendente.
    #[default]
    Idle,
    /// A pergunta «Save changes…?» está aberta, com o nome do projecto.
    Asking(String),
    /// Respondido: o app fecha no próximo evento, sem perguntar de novo.
    Go,
}

impl Recovery {
    /// Sem pasta nem sessão (os testes da shell, o headless).
    #[must_use]
    pub fn off() -> Self {
        Self {
            session: None,
            autosave: Autosave::new(Period::default(), Instant::now()),
            prompt: Prompt::default(),
            quit: Quit::Idle,
            touched: false,
            last_input: None,
            held: false,
            quiet: true,
            released: false,
            screen: None,
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

    /// O quadro acabou: `had` = houve entrada do artista nele.
    pub fn input(&mut self, had: bool, now: Instant) {
        self.quiet = !had;
        if had {
            self.touched = true;
            self.last_input = Some(now);
        }
    }

    /// Um botão do ponteiro desceu (`true`) ou subiu: um traço não se interrompe para serializar.
    pub fn pointer(&mut self, down: bool) {
        self.released |= self.held && !down;
        self.held = down;
    }

    /// O artista está em pausa: nada premido e, ou o quadro depois do fim de um traço sem entrada,
    /// ou [`IDLE`] sem entrada.
    #[must_use]
    pub fn idle(&self, now: Instant) -> bool {
        !self.held
            && ((self.released && self.quiet)
                || self
                    .last_input
                    .is_none_or(|t| now.saturating_duration_since(t) >= IDLE))
    }

    /// Chegou a hora de serializar? Com trabalho por copiar, na primeira pausa do artista depois
    /// da hora — ou, sem pausa, no prazo duro de 2 × o intervalo ([`Autosave::overdue`]).
    #[must_use]
    pub fn due(&self, now: Instant) -> bool {
        self.session.is_some()
            && self.touched
            && self.autosave.due(now)
            && (self.idle(now) || self.autosave.overdue(now))
    }

    /// Settings ▸ Recovery mudou (`secs`, `0` = Off): vale já.
    pub fn follow(&mut self, secs: u64, now: Instant) {
        self.autosave.set_period(Period::from_secs(secs), now);
    }

    /// Os bytes do Save, à hora: o fio grava a cópia se mudaram.
    pub fn offer(&mut self, bytes: Vec<u8>, project: Option<&str>, now: Instant) {
        let Some(s) = self.session.as_ref() else {
            return;
        };
        self.touched = false;
        self.released = false;
        self.autosave.offer(bytes, meta_of(project), s, now);
    }

    /// O Save gravou `bytes`: a cópia sai.
    pub fn saved(&mut self, bytes: &[u8], now: Instant) {
        self.touched = false;
        match self.session.as_ref() {
            Some(s) => self.autosave.mark_saved(bytes, s, now),
            // Sem pasta não há cópia, mas o «por gravar» do fecho continua a valer.
            None => self.autosave.opened(bytes, false, now),
        }
    }

    /// Abriu-se um projecto (Open ou `Restore`) com estes bytes: a primeira hora não regrava uma
    /// cópia igual. Num Open a cópia do projecto anterior sai; num `Restore` ela é a do restaurado.
    pub fn opened(&mut self, bytes: Option<&[u8]>, restored: bool) {
        let Some(bytes) = bytes else { return };
        self.touched = false;
        self.autosave.opened(bytes, restored, Instant::now());
        if !restored && let Some(s) = self.session.as_ref() {
            s.clear_copy();
        }
    }

    /// Fechar agora perderia trabalho? (`bytes` = os do Save, agora)
    #[must_use]
    pub fn unsaved(&self, bytes: &[u8]) -> bool {
        self.autosave.is_unsaved(bytes)
    }

    /// O pedido de fecho, com os bytes de agora (`None` = não serializou): `true` = há trabalho por
    /// gravar e a pergunta abriu-se; não fecha ainda.
    pub fn ask_quit(&mut self, bytes: Option<&[u8]>, project: Option<&str>) -> bool {
        let unsaved = bytes.is_some_and(|b| self.unsaved(b));
        if unsaved {
            self.quit = Quit::Asking(meta_of(project).name);
        }
        unsaved
    }

    /// `Save` na pergunta do fecho correu: fecha se os bytes de agora são os do disco; senão (o
    /// artista desistiu do Save As) a pergunta fecha e o app fica.
    pub fn saved_then_quit(&mut self, bytes: Option<&[u8]>) {
        let saved = bytes.is_some_and(|b| !self.unsaved(b));
        self.quit = if saved { Quit::Go } else { Quit::Idle };
    }

    /// Alguma janela da recuperação é dona da entrada (o arranque ou a pergunta do fecho)?
    #[must_use]
    pub fn is_modal(&self) -> bool {
        self.prompt.is_open() || matches!(self.quit, Quit::Asking(_))
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
