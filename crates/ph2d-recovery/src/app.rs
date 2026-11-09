//! A recuperação inteira de um processo, para a shell só COMPOR: arranque (sessão + varredura),
//! a hora, a oferta dos bytes, o Save, o `Restore` e o fecho.

use std::path::{Path, PathBuf};
use std::time::Instant;

use std::time::Duration;

use crate::{Autosave, Choice, Meta, Offer, Orphan, Period, Prompt, Session, now_secs, scan};

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
    /// Onde o ponteiro está (píxeis FÍSICOS), visto por [`Self::hover`] em TODO movimento.
    /// ⛔ O clique do winit não traz posição: a shell usa a do último movimento, e a janela modal
    /// esconde o movimento do editor — sem este cursor o clique caía onde o ponteiro estava ao
    /// abrir-se a janela, era comido, e o app parecia travado (smoke do dono, 08/11).
    pub cursor: Option<(f32, f32)>,
    /// **O ficheiro que este projecto tem AGORA** (`None` = ainda sem nome: o primeiro `Ctrl+S`
    /// pergunta). Mora aqui porque é a mesma vida do ficheiro: o nome da cópia, o Restore que
    /// devolve o caminho original e a pergunta do fecho lêem-no.
    pub project_path: Option<String>,
    /// Onde a janela se pintou no último quadro (o ecrã e a escala do chrome): o clique lê daqui.
    pub screen: Option<(
        ph2d_editor_core::zones::Rect,
        ph2d_editor_core::ui_scale::UiScaleMap,
    )>,
    /// A altura do corpo da pergunta do fecho, medida (já quebrado) no último quadro pintado: a
    /// janela cresce com o texto e o [`crate::card::click`] lê a MESMA disposição.
    pub quit_body_h: Option<f32>,
    /// O `Restore` respondido, à espera de o projecto abrir ([`Self::restored`]).
    pending: Option<Orphan>,
    /// O fecho é limpo (nada por gravar, ou o artista respondeu Save / Don't Save): só então a
    /// pasta sai; senão fica para o próximo arranque ([`Self::close`]).
    clean: bool,
    /// A última cópia falhou (o aviso ao artista sai uma vez por série de falhas).
    failing: bool,
    /// Avisos para a shell mostrar (toasts, já traduzidos): [`Self::take_notices`].
    notices: Vec<String>,
    /// A impressão dos sinais baratos na última hora ([`Self::pulse`]); `None` = ainda não vista.
    print: Option<u64>,
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
            cursor: None,
            project_path: None,
            screen: None,
            quit_body_h: None,
            pending: None,
            clean: false,
            failing: false,
            notices: Vec::new(),
            print: None,
        }
    }

    /// Desligada como [`Self::off`], já com o ficheiro do projecto (o arranque da shell).
    #[must_use]
    pub fn for_project(project_path: Option<String>) -> Self {
        Self {
            project_path,
            ..Self::off()
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
                r.prompt.entries = scan(root, now_secs(), Some(s.dir()));
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

    /// O ponteiro mexeu-se para `(x, y)` (píxeis FÍSICOS): o clique da janela lê daqui.
    pub fn hover(&mut self, x: f32, y: f32) {
        self.cursor = Some((x, y));
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

    /// ⭐ O relógio da cópia, chamado a cada volta do laço de eventos (`about_to_wait`), NUNCA só
    /// no quadro: no Wayland uma janela tapada ou minimizada não recebe quadros, e a cópia não pode
    /// depender deles (P12 smoke 2, 08/11). `secs` = Settings ▸ Recovery; `project` junta o
    /// projecto (no fio da janela) e devolve o que se serializa no fio da escrita e o caminho.
    pub fn tick<P, F>(&mut self, secs: u64, now: Instant, project: P)
    where
        P: FnOnce() -> Option<(F, Option<String>)>,
        F: FnOnce() -> Option<Vec<u8>> + Send + 'static,
    {
        if self.ready(secs, now)
            && let Some((make, path)) = project()
        {
            self.offer_with(make, path.as_deref(), now);
        }
    }

    /// A hora chegou (e o artista está em pausa)? A shell junta então os sinais baratos de quem muda
    /// o projecto SEM entrada local (rede, scripts, macros) e responde em [`Self::pulse`]: uma vez
    /// por intervalo, nunca por quadro.
    pub fn wants_pulse(&mut self, secs: u64, now: Instant) -> bool {
        self.follow(secs, now);
        self.session.is_some()
            && self.autosave.due(now)
            && (self.idle(now) || self.autosave.overdue(now))
    }

    /// `print` = a impressão dos sinais baratos; `changed` = um sinal que já diz «mudou» (o mundo).
    /// Mudou (ou é a primeira vez: sem impressão anterior o estado é desconhecido) ⇒ há trabalho por
    /// copiar; senão a hora adia-se um intervalo e nada se serializa.
    pub fn pulse(&mut self, print: u64, changed: bool, now: Instant) {
        if changed || self.print.replace(print) != Some(print) {
            self.touched = true;
        } else if !self.touched {
            self.autosave.postpone(now);
        }
    }

    /// [`Self::follow`] e [`Self::due`] juntos: a pergunta barata (~µs) de cada volta do laço.
    pub fn ready(&mut self, secs: u64, now: Instant) -> bool {
        self.follow(secs, now);
        self.reap(false);
        self.due(now)
    }

    /// Colhe o fio da escrita (`block` = espera-o). Falhou: o trabalho fica por copiar (a próxima
    /// hora tenta de novo) e o artista é avisado uma vez por série de falhas.
    fn reap(&mut self, block: bool) {
        if !block && !self.autosave.finished() {
            return;
        }
        match self.autosave.finish() {
            Some(Offer::Failed(e)) => {
                self.touched = true;
                if !std::mem::replace(&mut self.failing, true) {
                    let msg = ph2d_i18n::tr_with("recovery.copy_failed", &[("err", &e)]);
                    self.notices.push(msg);
                }
            }
            Some(_) => self.failing = false,
            None => {}
        }
    }

    /// Os avisos por mostrar (toasts), uma vez cada.
    pub fn take_notices(&mut self) -> Vec<String> {
        self.reap(false);
        std::mem::take(&mut self.notices)
    }

    /// Os bytes do Save, à hora: o fio grava a cópia se mudaram.
    pub fn offer(&mut self, bytes: Vec<u8>, project: Option<&str>, now: Instant) {
        self.reap(true);
        let Some(s) = self.session.as_ref() else {
            return;
        };
        self.touched = false;
        self.released = false;
        self.autosave.offer(bytes, meta_of(project), s, now);
    }

    /// [`Self::offer`] com os bytes por juntar no fio da escrita
    /// ([`crate::Autosave::offer_with`]).
    pub fn offer_with<F>(&mut self, make: F, project: Option<&str>, now: Instant)
    where
        F: FnOnce() -> Option<Vec<u8>> + Send + 'static,
    {
        self.reap(true);
        let Some(s) = self.session.as_ref() else {
            return;
        };
        self.touched = false;
        self.released = false;
        self.autosave.offer_with(make, meta_of(project), s, now);
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
        self.print = None;
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

    /// O pedido de fecho, com os bytes de agora (`None` = não serializou, e o estado desconhecido
    /// conta como por gravar): `true` = a pergunta abriu-se; não fecha ainda. `false` = nada por
    /// gravar: o fecho é limpo.
    pub fn ask_quit(&mut self, bytes: Option<&[u8]>, project: Option<&str>) -> bool {
        let unsaved = bytes.is_none_or(|b| self.unsaved(b));
        if unsaved {
            self.quit = Quit::Asking(meta_of(project).name);
        }
        self.clean = !unsaved;
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

    /// Responde à linha `i` da janela. `Restore` devolve `(cópia a abrir, caminho original)` e
    /// NADA apaga: a shell abre a cópia e responde em [`Self::restored`].
    pub fn answer(&mut self, i: usize, choice: Choice) -> Option<(PathBuf, Option<String>)> {
        let (orphan, choice) = self.prompt.answer(i, choice)?;
        if choice != Choice::Restore || self.session.is_none() {
            return None;
        }
        let path = Some(orphan.meta.path.clone()).filter(|p| !p.is_empty());
        let copy = orphan.copy.clone();
        self.pending = Some(orphan);
        Some((copy, path))
    }

    /// O `Restore` abriu (`ok`, com os `bytes` de agora) ou foi recusado. Aberto: a cópia passa a
    /// ser desta sessão, a pasta morta sai e o projecto fica por gravar. Recusado: a cópia fica no
    /// disco e volta à janela, e o artista é avisado. `true` = a shell adopta o caminho original.
    pub fn restored(&mut self, ok: bool, bytes: Option<&[u8]>) -> bool {
        let Some(orphan) = self.pending.take() else {
            return false;
        };
        let adopted = ok
            && self.session.as_ref().is_some_and(|s| {
                self.autosave.finish();
                s.adopt(&orphan, now_secs())
                    .inspect_err(|e| eprintln!("[recovery] Restore falhou: {e}"))
                    .is_ok()
            });
        if !adopted {
            self.prompt.entries.insert(0, orphan);
            let msg = ph2d_i18n::tr("recovery.restore_failed").to_owned();
            self.notices.push(msg);
            return false;
        }
        self.prompt.later_all();
        self.opened(bytes, true);
        self.autosave.forget();
        true
    }

    /// O fecho: espera a escrita; limpo (o artista decidiu ou nada estava por gravar) apaga a
    /// sessão, senão solta a tranca e deixa a cópia para o próximo arranque.
    pub fn close(&mut self) {
        self.autosave.finish();
        if let Some(s) = self.session.take() {
            if self.clean || self.quit == Quit::Go {
                s.close();
            } else {
                s.leave();
            }
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
