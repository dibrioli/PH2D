//! **As macros como o artista as faz** (ADR-0186, W5d; `docs/Macros/02_plano_W5.md` §5): Gravar ▸
//! Parar nas duas famílias, as etapas que o painel MACROS e a linha da Timeline mostram, e as edições
//! delas. A shell compõe: drena os pedidos ([`Macros::pede`]), chama [`Macros::quadro`] uma vez por
//! quadro, DEPOIS de as famílias consumirem a entrada, e publica [`Macros::info`].
//!
//! O estado é da família ([`com`], um `thread_local`), nunca um campo da `App` (ADR-0075).

mod conferencia;
pub mod erros;
pub mod espera;
pub mod etapas;
pub mod instala;
pub mod sessao;
mod tutorial;

use etapas::Injectadas;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::macro_edits::{
    self as m, ErroDaMacroView, FamiliaDaMacro, MacroObjetoView, MacroPanelInfo, MacroRequest,
};
use ph2d_timeline::{TimelineDoc, TimelineIntent, TimelineState};
use ph2d_tool_painter::PainterTool;
use sessao::Sessao;
use std::cell::RefCell;

/// As famílias que gravam, como a shell as tem neste quadro. O Painter chega SEMPRE (uma gravação
/// em curso tem de receber o Parar mesmo depois de o artista trocar de ferramenta); só arma quando é
/// a ferramenta activa (`painter_activo`). O Motion é outra família: a composição passa a gravadora
/// dele, o objecto em edição e as etapas ([`etapas::Injectadas`]) — esta crate não depende dela.
pub struct Familias<'a> {
    pub painter: Option<&'a mut PainterTool>,
    pub painter_activo: bool,
    /// O Motion grava e ensina (o tutorial «fazer», W6).
    pub motion: &'a mut dyn ph2d_editor_core::macro_tutorial::GravadoraQueEnsina,
    /// O `StableId` do objecto Motion em edição: só com ele o Motion arma, e é ele o dono da `Macro`.
    pub motion_em_edicao: Option<u64>,
    /// A ferramenta Motion está na mão: com o objecto em edição, é o modo Edit (o tutorial pede-o).
    pub motion_na_mao: bool,
    pub etapas: Injectadas<'a>,
}

impl Familias<'_> {
    fn quem_grava(&self) -> Result<FamiliaDaMacro, &'static str> {
        quem_grava(
            self.painter.as_deref().filter(|_| self.painter_activo),
            self.motion_em_edicao,
        )
    }
}

/// O que um pedido muda fora das famílias.
pub struct Cena<'a> {
    pub sim: &'a mut SimWorld,
    /// O documento E a história: criar o clipe é um passo da pilha da Timeline.
    pub timeline: &'a mut TimelineState,
    pub intents: &'a mut Vec<TimelineIntent>,
    /// De onde a conferência de uma edição de valor recomeça (o palco do Painter em cena).
    pub partidas: Option<conferencia::Partidas<'a>>,
}

pub use conferencia::Partidas;

/// As cenas de `PH2D_MACRO_SMOKE`: `=1` o Splash (Painter), `=2` o Motion nó a nó, `=3` o tutorial
/// «fazer» do Motion, `=4` o do Painter, `=5` a imagem é do artista (W7 item 1), `=6` o caminho do dono
/// na `=5`, feito pelo app sozinho (Gravar, 2 pinceladas, Parar, ⏮, ▷). Cada uma monta-se na família que
/// grava; o clipe que conduz `=1`, `=2`, `=5` e `=6` monta-o a composição (`render_loop/macro_smoke.rs`).
pub const CENAS_DO_SMOKE: u32 = 6;

/// **A cena abre PARADA?** — `=5` (`docs/Macros/04_plano_W7.md` item 1): o clipe sem Autoplay, a cena
/// sem correr e a imagem em modo Paint, porque o 1.º passo do smoke é o artista pintar; `=6` é a mesma
/// cena. As outras com clipe correm sozinhas.
#[must_use]
pub fn a_cena_abre_parada(nivel: &str) -> bool {
    matches!(nivel, "5" | "6")
}

/// O nível de `PH2D_MACRO_SMOKE` que nenhuma cena serve: diz ao terminal quais há.
pub fn cena_desconhecida<T>(nivel: &str) -> Option<T> {
    eprintln!(
        "[macro smoke] PH2D_MACRO_SMOKE={nivel}: the scenes are 1 (Splash), 2 (Motion node by node), \
         3 (the 'do it' tutorial), 4 (the painting tutorial), 5 (paint, then play: your paint stays) and 6 (the app records, paints, rewinds and plays by itself)"
    );
    None
}

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "macros",
    routers: &[ph2d_app_host::SmokeRouter {
        env: "PH2D_MACRO_SMOKE",
        max_level: CENAS_DO_SMOKE,
    }],
};

/// O que o cartão MACRO do Inspector mostra da gravação, além do que o componente guarda.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumoDaMacro {
    pub etapas: usize,
    /// A 1.ª etapa com edições, a contar de 0.
    pub editada_desde: Option<usize>,
    pub erro: Option<ErroDaMacroView>,
}

/// O estado da família MACROS.
#[derive(Default)]
pub struct Macros {
    sessao: Option<Sessao>,
    pedidos: Vec<MacroRequest>,
    /// A última recusa (armar, parar): fica até ao próximo Gravar.
    recusa: Option<&'static str>,
    /// A última edição recusada, pelo objecto.
    erro_de_edicao: Option<(u64, ErroDaMacroView)>,
    conferencias: conferencia::Conferencias,
    vistas: etapas::Cache,
    duracoes: instala::Duracoes,
    tutorial: tutorial::Tutorial,
    /// O relógio do quadro corrente (o de [`Macros::quadro`]).
    agora_us: u64,
}

thread_local! {
    static MACROS: RefCell<Macros> = RefCell::new(Macros::default());
}

/// O estado da família neste processo.
pub fn com<R>(f: impl FnOnce(&mut Macros) -> R) -> R {
    MACROS.with(|m| f(&mut m.borrow_mut()))
}

/// **Quem grava se o artista carregar em Gravar agora** — ou a chave do porquê não.
///
/// # Errors
/// A chave da frase ([`m::MACRO_RECUSA_SEM_TELA`], …).
pub fn quem_grava(
    painter: Option<&PainterTool>,
    motion_em_edicao: Option<u64>,
) -> Result<FamiliaDaMacro, &'static str> {
    if let Some(p) = painter {
        if p.bound_document().is_none() {
            return Err(m::MACRO_RECUSA_SEM_TELA);
        }
        return Ok(FamiliaDaMacro::Painter);
    }
    if motion_em_edicao.is_some() {
        return Ok(FamiliaDaMacro::Motion);
    }
    Err(m::MACRO_RECUSA_SEM_FAMILIA)
}

impl Macros {
    pub fn pede(&mut self, r: MacroRequest) {
        self.pedidos.push(r);
    }

    /// **Uma edição de valor está por conferir ou a conferir** (fora do quadro): o undo espera por ela,
    /// como pelo recálculo do Colorize — a edição e a decisão da conferência são UM passo.
    #[must_use]
    pub fn a_conferir(&self) -> bool {
        self.conferencias.a_conferir()
    }

    /// Os objectos (bits) cuja edição espera a conferência ([`espera`]).
    #[must_use]
    pub fn a_conferir_em(&self) -> Vec<u64> {
        self.conferencias.a_conferir_em()
    }

    /// **O tutorial do Painter mede as referências das pinceladas** (fora do quadro): até chegarem,
    /// nenhuma se dá por feita.
    #[must_use]
    pub fn a_medir(&self) -> bool {
        self.tutorial.a_medir()
    }

    /// **Depois de o Motion correr no quadro** (o lote do grafo, os params e o `Quadro`): os passos que
    /// ele gravou NESTE quadro, no relógio dele — em [`Macros::quadro`] ainda não tinham acontecido.
    pub fn depois_do_motion(&mut self, motion: &mut dyn ph2d_macro::Gravadora) {
        let agora = self.agora_us;
        if let Some(s) = self
            .sessao
            .as_mut()
            .filter(|s| s.familia() == FamiliaDaMacro::Motion)
        {
            s.passos(motion, agora);
        }
    }

    /// **A sprite que o Painter está a gravar**: o palco dela em cena cede-a ao traço vivo do editor
    /// (uma 2.ª tomada sobre uma sprite com macro mostra o que o artista pinta, não a tomada antiga).
    #[must_use]
    pub fn sprite_gravada(&self) -> Option<u64> {
        self.sessao
            .as_ref()
            .filter(|s| s.familia() == FamiliaDaMacro::Painter)
            .map(sessao::Sessao::dono)
    }

    #[must_use]
    pub fn gravando(&self) -> Option<FamiliaDaMacro> {
        self.sessao.as_ref().map(Sessao::familia)
    }

    /// **O quadro**: os pedidos por ordem, depois os passos que as famílias gravaram neste quadro.
    /// `agora_us` é o relógio do chamador (o núcleo nunca lê o de parede).
    pub fn quadro(&mut self, f: &mut Familias<'_>, c: &mut Cena<'_>, agora_us: u64) {
        self.agora_us = agora_us;
        for r in std::mem::take(&mut self.pedidos) {
            self.aplica(r, f, c, agora_us);
        }
        if let Some(e) = self.conferencias.quadro(c) {
            self.erro_de_edicao = Some(e);
        }
        // O Painter consumiu a entrada do quadro antes desta fase; o Motion corre DEPOIS dela
        // ([`Macros::depois_do_motion`]).
        if let Some(s) = self
            .sessao
            .as_mut()
            .filter(|s| s.familia() == FamiliaDaMacro::Painter)
        {
            s.quadro(f, agora_us);
        }
        self.sincroniza(c, f.etapas);
        ph2d_editor_core::macro_tutorial::set_balao(self.tutorial.balao(f, c.sim));
    }

    /// **O clipe acompanha a macro**: a duração dele é DERIVADA da trilha que toca, e re-cronometra-se
    /// a cada quadro enquanto ele tiver a forma que o Parar lhe deu
    /// ([`TimelineDoc::macro_clip_identity_end`]). Uma só verdade (a `Macro`, no diff do mundo): o undo
    /// do mundo devolve a macro antiga e o clipe segue-a no quadro seguinte, sem passo de undo
    /// próprio que pudesse discordar.
    fn sincroniza(&mut self, c: &mut Cena<'_>, outras: Injectadas<'_>) {
        let macros: Vec<(u64, Vec<u8>)> = {
            let mut q = c
                .sim
                .world_mut()
                .query::<(ph2d_ecs::Entity, &ph2d_ecs::Macro)>();
            q.iter(c.sim.world())
                .map(|(e, m)| (e.to_bits(), m.doc.clone()))
                .collect()
        };
        for (bits, bytes) in macros {
            let Some(clip) = instala::clipe_de(&c.timeline.doc, bits) else {
                continue;
            };
            let Some(fim) = c.timeline.doc.macro_clip_identity_end(clip, bits) else {
                continue;
            };
            let d = self.duracoes.de(bits, &bytes, outras);
            if (d - fim).abs() > 1e-6 {
                c.timeline.doc.retime_macro_clip(clip, bits, d);
                if clip == c.timeline.doc.active_index() {
                    instala::rearma_o_laco(c.timeline, c.intents);
                }
            }
        }
    }

    fn aplica(&mut self, r: MacroRequest, f: &mut Familias<'_>, c: &mut Cena<'_>, agora_us: u64) {
        match r {
            MacroRequest::Grava => {
                if self.sessao.is_some() {
                    self.recusa = Some(m::MACRO_RECUSA_JA_GRAVA);
                    return;
                }
                let r = f
                    .quem_grava()
                    .and_then(|fam| Sessao::abre(fam, f, c.sim, agora_us));
                match r {
                    Ok(s) => {
                        self.sessao = Some(s);
                        self.recusa = None;
                    }
                    Err(k) => self.recusa = Some(k),
                }
            }
            MacroRequest::Para => {
                let Some(s) = self.sessao.take() else {
                    return;
                };
                if let Err(k) = s.para(f, c, agora_us) {
                    self.recusa = Some(k);
                }
            }
            MacroRequest::VaiPara { t_s, .. } => {
                c.intents.push(TimelineIntent::ScrubExact(t_s));
            }
            MacroRequest::Edita {
                entity_bits,
                trilha,
                k,
                campo,
            } => match etapas::edita(c, f.etapas, entity_bits, trilha, k, &campo) {
                Ok(()) => self.erro_de_edicao = None,
                Err(e) => self.erro_de_edicao = Some((entity_bits, e)),
            },
            MacroRequest::Apaga {
                entity_bits,
                trilha,
                k,
            } => match etapas::apaga(c, f.etapas, entity_bits, trilha, k) {
                Ok(()) => self.erro_de_edicao = None,
                Err(e) => self.erro_de_edicao = Some((entity_bits, e)),
            },
            MacroRequest::MostraMe { entity_bits } => {
                self.tutorial.mostra(f, c.sim, entity_bits);
            }
            MacroRequest::Recomeca { entity_bits } => {
                self.tutorial.recomeca(f, c.sim, entity_bits);
            }
        }
    }

    /// **A linha «Macro steps» da Timeline**: as etapas da `Macro` do dono do clipe activo, quando o
    /// clipe activo é o da macro dele (o tempo do clipe é o da macro: a faixa Macro Time é a
    /// identidade). Mesma cache do painel. `outras` = as etapas que a composição injecta.
    pub fn etapas_na_linha(
        &mut self,
        sim: &SimWorld,
        tl: &TimelineDoc,
        outras: Injectadas<'_>,
    ) -> Vec<ph2d_timeline::EtapaDaMacroView> {
        let activo = tl.active_index();
        let Some(dono) = tl
            .clips()
            .get(activo)
            .and_then(|c| c.owner)
            .map(|o| o.entity)
            .filter(|&d| instala::clipe_de(tl, d) == Some(activo))
        else {
            return Vec::new();
        };
        let Some(m) = sim
            .world()
            .get::<ph2d_ecs::Macro>(ph2d_ecs::Entity::from_bits(dono))
        else {
            return Vec::new();
        };
        self.vistas
            .de(dono, &m.doc, outras)
            .etapas
            .iter()
            .map(|e| ph2d_timeline::EtapaDaMacroView {
                entity_bits: dono,
                trilha: e.trilha,
                k: e.k,
                fim: e.t1_s,
                espera_s: e.espera_s,
            })
            .collect()
    }

    /// A última edição recusada, se foi neste objecto.
    fn erro_da_edicao(&self, bits: u64) -> Option<ErroDaMacroView> {
        self.erro_de_edicao
            .as_ref()
            .filter(|(b, _)| *b == bits)
            .map(|(_, e)| e.clone())
    }

    /// **O resumo do cartão MACRO do Inspector**: quantas etapas, a 1.ª editada e o erro — pela
    /// MESMA cache e pela MESMA precedência do painel ([`Self::info`]: a edição recusada, depois a
    /// reprodução em cena, depois a leitura). `None` = o objecto não tem `Macro`.
    pub fn resumo(
        &mut self,
        sim: &SimWorld,
        bits: u64,
        erro_em_cena: Option<ErroDaMacroView>,
        outras: Injectadas<'_>,
    ) -> Option<ResumoDaMacro> {
        let bytes = &sim
            .world()
            .get::<ph2d_ecs::Macro>(ph2d_ecs::Entity::from_bits(bits))?
            .doc;
        let edicao = self.erro_da_edicao(bits);
        let v = self.vistas.de(bits, bytes, outras);
        Some(ResumoDaMacro {
            etapas: v.etapas.len(),
            editada_desde: v.editada_desde,
            erro: edicao.or(erro_em_cena).or_else(|| v.erro.clone()),
        })
    }

    /// **O instantâneo do painel**. `escolhido` = os bits do objecto escolhido; `erro_em_cena` = o
    /// erro do palco dele (a reprodução), que só a shell alcança.
    pub fn info(
        &mut self,
        f: &Familias<'_>,
        sim: &SimWorld,
        escolhido: Option<u64>,
        erro_em_cena: Option<ErroDaMacroView>,
    ) -> MacroPanelInfo {
        let gravando = self.gravando();
        let (pode_gravar, recusa_agora) = match f.quem_grava() {
            Ok(fam) => (Some(fam), None),
            Err(k) => (None, Some(k)),
        };
        let objeto = escolhido.and_then(|bits| {
            let bytes = &sim
                .world()
                .get::<ph2d_ecs::Macro>(ph2d_ecs::Entity::from_bits(bits))?
                .doc;
            let edicao = self.erro_da_edicao(bits);
            let v = self.vistas.de(bits, bytes, f.etapas);
            let erro = edicao.or(erro_em_cena).or_else(|| v.erro.clone());
            Some(MacroObjetoView {
                entity_bits: bits,
                duracao_s: v.duracao_s,
                etapas: v.etapas.clone(),
                editada_desde: v.editada_desde,
                erro,
            })
        });
        MacroPanelInfo {
            gravando,
            pode_gravar: if gravando.is_some() {
                None
            } else {
                pode_gravar
            },
            recusa: self.recusa.or(recusa_agora),
            objeto,
        }
    }
}
