//! **As macros como o artista as faz** (ADR-0186, W5d; `docs/Macros/02_plano_W5.md` §5): Gravar ▸
//! Parar nas duas famílias, as etapas que o painel MACROS e a linha da Timeline mostram, e as edições
//! delas. A shell compõe: drena os pedidos ([`Macros::pede`]), chama [`Macros::quadro`] uma vez por
//! quadro, DEPOIS de as famílias consumirem a entrada, e publica [`Macros::info`].
//!
//! O estado é da família ([`com`], um `thread_local`), nunca um campo da `App` (ADR-0075).

pub mod erros;
pub mod etapas;
pub mod instala;
pub mod sessao;

use ph2d_app_motion::motion_state::MotionState;
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
/// a ferramenta activa (`painter_activo`).
pub struct Familias<'a> {
    pub painter: Option<&'a mut PainterTool>,
    pub painter_activo: bool,
    pub motion: &'a mut MotionState,
}

impl Familias<'_> {
    fn quem_grava(&self) -> Result<FamiliaDaMacro, &'static str> {
        quem_grava(
            self.painter.as_deref().filter(|_| self.painter_activo),
            self.motion,
        )
    }
}

/// O que um pedido muda fora das famílias.
pub struct Cena<'a> {
    pub sim: &'a mut SimWorld,
    /// O documento E a história: criar o clipe é um passo da pilha da Timeline.
    pub timeline: &'a mut TimelineState,
    pub intents: &'a mut Vec<TimelineIntent>,
}

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
    /// A trilha com uma edição de valor por conferir: `(bits, trilha)`.
    por_assentar: Option<(u64, usize)>,
    vistas: etapas::Cache,
    duracoes: instala::Duracoes,
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
/// A chave da frase ([`m::MACRO_RECUSA_MEIO`], …).
pub fn quem_grava(
    painter: Option<&PainterTool>,
    motion: &MotionState,
) -> Result<FamiliaDaMacro, &'static str> {
    if let Some(p) = painter {
        if !p.wetpaint_armed() {
            return Err(m::MACRO_RECUSA_MEIO);
        }
        if p.bound_document().is_none() {
            return Err(m::MACRO_RECUSA_SEM_TELA);
        }
        return Ok(FamiliaDaMacro::Painter);
    }
    if motion.objeto.editing.is_some() {
        return Ok(FamiliaDaMacro::Motion);
    }
    Err(m::MACRO_RECUSA_SEM_FAMILIA)
}

impl Macros {
    pub fn pede(&mut self, r: MacroRequest) {
        self.pedidos.push(r);
    }

    #[must_use]
    pub fn gravando(&self) -> Option<FamiliaDaMacro> {
        self.sessao.as_ref().map(Sessao::familia)
    }

    /// **O quadro**: os pedidos por ordem, depois os passos que as famílias gravaram neste quadro.
    /// `agora_us` é o relógio do chamador (o núcleo nunca lê o de parede).
    pub fn quadro(&mut self, f: &mut Familias<'_>, c: &mut Cena<'_>, agora_us: u64) {
        let pendente = self.por_assentar.take();
        for r in std::mem::take(&mut self.pedidos) {
            self.aplica(r, f, c, agora_us);
        }
        // A conferência corre no 1.º quadro em que a trilha já não recebeu edição.
        if let Some((bits, trilha)) = pendente
            && self.por_assentar != Some((bits, trilha))
            && let Err(e) = etapas::assenta(c, bits, trilha)
        {
            self.erro_de_edicao = Some((bits, e));
        }
        if let Some(s) = self.sessao.as_mut() {
            s.quadro(f, agora_us);
        }
        self.sincroniza(c);
    }

    /// **O clipe acompanha a macro**: a duração dele é DERIVADA da trilha que toca, e re-cronometra-se
    /// a cada quadro enquanto ele tiver a forma que o Parar lhe deu
    /// ([`TimelineDoc::macro_clip_identity_end`]). Uma só verdade (a `Macro`, no diff do mundo): o undo
    /// do mundo devolve a macro antiga e o clipe segue-a no quadro seguinte, sem passo de undo
    /// próprio que pudesse discordar.
    fn sincroniza(&mut self, c: &mut Cena<'_>) {
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
            let d = self.duracoes.de(bits, &bytes);
            if (d - fim).abs() > 1e-6 {
                c.timeline.doc.retime_macro_clip(clip, bits, d);
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
                c.intents.push(TimelineIntent::Scrub(t_s));
            }
            MacroRequest::Edita {
                entity_bits,
                trilha,
                k,
                campo,
            } => match etapas::edita(c, entity_bits, trilha, k, &campo) {
                Ok(confere) => {
                    self.erro_de_edicao = None;
                    if confere {
                        self.por_assentar = Some((entity_bits, trilha));
                    }
                }
                Err(e) => self.erro_de_edicao = Some((entity_bits, e)),
            },
            MacroRequest::Apaga {
                entity_bits,
                trilha,
                k,
            } => match etapas::apaga(c, entity_bits, trilha, k) {
                Ok(()) => self.erro_de_edicao = None,
                Err(e) => self.erro_de_edicao = Some((entity_bits, e)),
            },
        }
    }

    /// **A linha «Macro steps» da Timeline**: as etapas da `Macro` do dono do clipe activo, quando o
    /// clipe activo é o da macro dele (o tempo do clipe é o da macro: a faixa Macro Time é a
    /// identidade). Mesma cache do painel.
    pub fn etapas_na_linha(
        &mut self,
        sim: &SimWorld,
        tl: &TimelineDoc,
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
            .de(dono, &m.doc)
            .etapas
            .iter()
            .map(|e| ph2d_timeline::EtapaDaMacroView {
                entity_bits: dono,
                trilha: e.trilha,
                k: e.k,
                t0: e.t0_s,
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
    ) -> Option<ResumoDaMacro> {
        let bytes = &sim
            .world()
            .get::<ph2d_ecs::Macro>(ph2d_ecs::Entity::from_bits(bits))?
            .doc;
        let edicao = self.erro_da_edicao(bits);
        let v = self.vistas.de(bits, bytes);
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
            let v = self.vistas.de(bits, bytes);
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
