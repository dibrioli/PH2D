//! Uma aba: a cópia de trabalho da receita, o desfazer, o render e o gesto em curso.

use std::sync::Arc;

use ph2d_ecs::StableId;
use ph2d_editor_core::zones::Rect;
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
    /// Uma linha da lista de objetos, pelo `StableId`.
    Objeto(u64),
    /// A linha que só explica: o clique fica nela.
    ObjetoOff,
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Gesto {
    Inicio { id: LayerId, x0: f32, start0: f32 },
    Ordem { id: LayerId },
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
    pub menu_add: bool,
    /// O objeto a que este som se destina, pela identidade.
    pub ligado: Option<StableId>,
    pub quer_usar: bool,
    pub objeto_sumiu: bool,
    pub recusa: Option<String>,
    pub menu_objetos: bool,
    pub objetos: Vec<(u64, String)>,
    pub menu_ini: usize,
    pub menu_max: usize,
    pub menu_caixa: Rect,
    pub x_usar: f32,
    pub rolagem: f32,
    pub rolagem_max: f32,
    pub alvos: Vec<(Alvo, Rect)>,
    /// A pista do tempo, da última pintura.
    pub pista: Rect,
    pub linhas: Rect,
    pub corpo: Rect,
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
            menu_add: false,
            ligado: None,
            quer_usar: false,
            objeto_sumiu: false,
            recusa: None,
            menu_objetos: false,
            objetos: Vec::new(),
            menu_ini: 0,
            menu_max: 0,
            menu_caixa: nada,
            x_usar: 0.0,
            rolagem: 0.0,
            rolagem_max: 0.0,
            alvos: Vec::new(),
            pista: nada,
            linhas: nada,
            corpo: nada,
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

    fn sobreviventes(&mut self) {
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
    }

    /// O que a barra diz, pelo render mais novo que chegou.
    #[must_use]
    pub fn estado_do_aprovado(&self) -> EstadoDoAprovado {
        estado(&self.receita, self.pronto.as_ref().map(|f| &f.impressao))
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
