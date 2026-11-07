//! Uma aba: a cópia de trabalho da receita, o desfazer, o render e o gesto em curso.

use std::sync::Arc;

use ph2d_ecs::StableId;
use ph2d_editor_core::zones::Rect;
use ph2d_sfx_explore::Exploracao;
use ph2d_sfx_explore_tela::AlvoE;
use ph2d_sfx_lab::aprovar::{EstadoDoAprovado, estado};
use ph2d_sfx_recipe::{LayerId, Recipe, Takes};

use ph2d_sfx_lab::historia::Historia;
use ph2d_sfx_lab::job::{Feito, Pedido, RenderJob};
use ph2d_sfx_lab::novo::Fonte;

/// O que a tela sabe receber. O id da camada é o da receita.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Play,
    Stop,
    PlayOnChange,
    UseInGame,
    /// O que a janela do Use in Game registrou, com a lista pelo `StableId`.
    Janela(ph2d_object_picker::Alvo),
    Regua,
    Linha(LayerId),
    Alca(LayerId),
    Mudo(LayerId),
    Solo(LayerId),
    Onda(LayerId),
    Add,
    Item(Fonte),
    /// O item sem efeito: o clique fica nele e o motivo está escrito.
    ItemOff(Fonte),
    /// O L da linha da camada.
    Trava(LayerId),
    /// A barra, a faixa Heard e a grade do Explore.
    Ex(AlvoE),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Gesto {
    Inicio {
        id: LayerId,
        x0: f32,
        start0: f32,
    },
    Ordem {
        id: LayerId,
    },
    /// Arrastando uma variação no Explore. `andou` = já não é um clique.
    Filho {
        i: usize,
        x0: f32,
        y0: f32,
        t0: f64,
        andou: bool,
    },
    Mudanca,
}

pub(crate) struct Aba {
    pub id: u64,
    pub nome: String,
    pub receita: Recipe,
    /// O que o mundo tem desta receita; `None` = ainda não foi escrita.
    pub escrita: Option<Recipe>,
    pub hist: Historia,
    pub job: RenderJob,
    pub takes: Arc<Takes>,
    pub ticket: u64,
    pub pronto: Option<Feito>,
    pub erro: Option<String>,
    pub sel: Option<LayerId>,
    pub solo: Option<LayerId>,
    pub play_on_change: bool,
    pub quer_tocar: bool,
    pub quer_parar: bool,
    pub cursor: f32,
    pub gesto: Option<Gesto>,
    /// O gesto de desfazer em curso é de um campo dos cartões.
    pub gesto_do_cartao: bool,
    pub prensa: Option<Alvo>,
    pub hover: Option<Alvo>,
    /// Onde a mão estava no último `Move`.
    pub ponteiro: Option<(f32, f32)>,
    pub menu_add: bool,
    /// O objeto a que este som se destina, pela identidade.
    pub ligado: Option<StableId>,
    pub quer_escolher: bool,
    pub quer_usar: bool,
    pub objeto_sumiu: bool,
    pub recusa: Option<String>,
    pub janela: Option<ph2d_object_picker::ObjectPicker>,
    pub rolagem: f32,
    pub rolagem_max: f32,
    pub alvos: Vec<(Alvo, Rect)>,
    /// A pista do tempo, da última pintura.
    pub pista: Rect,
    pub linhas: Rect,
    pub corpo: Rect,
    pub ex: Exploracao,
    /// O próximo toque é de um gesto de exploração: sai no mesmo volume.
    pub quer_nivelado: bool,
    pub so_kept: bool,
    pub rolagem_heard: f32,
    pub rolagem_heard_max: f32,
    pub heard: Rect,
    pub clique_anterior: Option<(u32, u128)>,
    pub agora_ns: u128,
    pub mao: (f32, f32),
}

impl Aba {
    pub fn new(
        id: u64,
        nome: String,
        receita: Recipe,
        escrita: Option<Recipe>,
        takes: Arc<Takes>,
    ) -> Self {
        let nada = Rect::new(0.0, 0.0, 0.0, 0.0);
        let mut a = Self {
            id,
            nome,
            sel: receita.layers.first().map(|l| l.id),
            receita,
            escrita,
            hist: Historia::default(),
            job: RenderJob::default(),
            takes,
            ticket: 0,
            pronto: None,
            erro: None,
            solo: None,
            play_on_change: false,
            quer_tocar: false,
            quer_parar: false,
            cursor: 0.0,
            gesto: None,
            gesto_do_cartao: false,
            prensa: None,
            hover: None,
            ponteiro: None,
            menu_add: false,
            ligado: None,
            quer_escolher: false,
            quer_usar: false,
            objeto_sumiu: false,
            recusa: None,
            janela: None,
            rolagem: 0.0,
            rolagem_max: 0.0,
            alvos: Vec::new(),
            pista: nada,
            linhas: nada,
            corpo: nada,
            ex: Exploracao::default(),
            quer_nivelado: false,
            so_kept: false,
            rolagem_heard: 0.0,
            rolagem_heard_max: 0.0,
            heard: nada,
            clique_anterior: None,
            agora_ns: 0,
            mao: (0.0, 0.0),
        };
        a.pede();
        a
    }

    /// O documento ou o solo mudou: um render novo, sem esperar.
    pub fn pede(&mut self) {
        self.ticket += 1;
        self.job.submit(
            Pedido {
                receita: self.receita.clone(),
                solo: self.solo,
                takes: self.takes.clone(),
            },
            self.ticket,
        );
    }

    #[must_use]
    pub fn em_voo(&self) -> bool {
        let chegou = self
            .pronto
            .as_ref()
            .is_some_and(|f| f.ticket == self.ticket);
        !chegou && self.erro.is_none()
    }

    pub fn recebe(&mut self) {
        match self.job.poll() {
            Some(Ok(f)) => {
                self.pronto = Some(f);
                self.erro = None;
            }
            Some(Err(e)) => self.erro = Some(e.to_string()),
            None => {}
        }
    }

    /// Toda edição do documento passa aqui.
    pub fn edita(&mut self, f: impl FnOnce(&mut Recipe)) {
        let antes = self.receita.clone();
        f(&mut self.receita);
        if self.receita == antes {
            return;
        }
        self.hist.anota(antes);
        self.erro = None;
        self.pede();
        if self.play_on_change && !self.hist.em_gesto() {
            self.quer_tocar = true;
        }
    }

    /// Larga o gesto em curso: um passo de desfazer.
    pub fn fecha_gesto(&mut self) {
        self.gesto = None;
        if self.hist.fecha_gesto(&self.receita) && self.play_on_change {
            self.quer_tocar = true;
        }
    }

    pub fn desfaz(&mut self, refazer: bool) -> bool {
        self.gesto = None;
        let r = if refazer {
            self.hist.refaz(&self.receita)
        } else {
            self.hist.desfaz(&self.receita)
        };
        let Some(mut r) = r else {
            return false;
        };
        // O desfazer é da receita: o som aprovado está no projeto, e voltar atrás não o desaprova.
        r.approved = self.receita.approved.clone();
        self.receita = r;
        self.sobreviventes();
        self.erro = None;
        self.pede();
        true
    }

    /// A receita veio de fora (projeto aberto): vale ela, e o desfazer recomeça.
    pub fn adota(&mut self, r: Recipe) {
        self.escrita = Some(r.clone());
        self.receita = r;
        self.hist.zera();
        self.gesto = None;
        self.sobreviventes();
        self.erro = None;
        self.pede();
    }

    pub(crate) fn sobreviventes(&mut self) {
        let existe = |id: Option<LayerId>, r: &Recipe| id.filter(|i| r.layer(*i).is_some());
        self.sel = existe(self.sel, &self.receita);
        self.solo = existe(self.solo, &self.receita);
    }

    pub fn acrescenta(&mut self, fonte: Fonte) {
        let take = self.takes.iter().next().map(|t| t.id);
        let mut novo = None;
        self.edita(|r| novo = ph2d_sfx_lab::novo::acrescenta(r, fonte, take));
        if novo.is_some() {
            self.sel = novo;
        }
    }

    pub fn apaga(&mut self, id: LayerId) {
        self.edita(|r| r.layers.retain(|l| l.id != id));
        self.sobreviventes();
    }

    pub fn alterna_solo(&mut self, id: LayerId) {
        self.solo = if self.solo == Some(id) {
            None
        } else {
            Some(id)
        };
        self.pede();
        // Toca o render do solo novo: `servir` espera o ticket que acabou de ser pedido.
        if self.play_on_change {
            self.quer_tocar = true;
        }
    }

    #[must_use]
    pub fn conta_gotas(&self) -> bool {
        self.janela.as_ref().is_some_and(|j| j.armado())
    }

    /// O que a barra diz, pelo render mais novo que chegou.
    #[must_use]
    pub fn estado_do_aprovado(&self) -> EstadoDoAprovado {
        estado(&self.receita, self.pronto.as_ref().map(|f| &f.impressao))
    }

    #[must_use]
    pub(crate) fn rect_de_ex(&self, alvo: AlvoE) -> Option<Rect> {
        self.alvos
            .iter()
            .find(|(a, _)| *a == Alvo::Ex(alvo))
            .map(|(_, r)| *r)
    }

    /// Px por segundo na pista.
    #[must_use]
    pub fn px_por_s(&self) -> f32 {
        if self.receita.seconds > 0.0 {
            self.pista.w / self.receita.seconds
        } else {
            0.0
        }
    }
}
