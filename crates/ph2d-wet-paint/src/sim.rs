//! Simulation orchestrator (port of `sim.js`, SPEC §5): fixed 40 Hz steps
//! with per-pass cadences, plus the adaptive drying cadence.
//!
//! Per step (frame counter n, incremented first):
//!   n % 2 == 0        rebuild the active region; if no fluid remains, idle
//!   n % dry_every == 0 drying / settle / re-wet
//!   n % 4 == 0        flow-field build WITH the absorbency brake
//!   other frames      cheap velocity smoothing, never braked (+ diffusion ext)
//!   every frame       advection + gravity injection, then the drain boundaries
//!   n % 3 == 0        pressure projection, then velocity boundaries again
//!
//! The caller (facade or test) owns the accumulator and the "paused while the
//! pointer is down" rule; this module only advances single fixed steps.

use crate::colorops::ColorMix;
use crate::drying::fast_dry;
use crate::grid::Grid;
use crate::solver::{
    advect, apply_boundaries, build_flow_field, diffusion_pass, project, rebuild_active_region,
    smooth_velocity,
};
use crate::tuning::{KNOB_COUNT, Knob, Tuning};
use ph2d_det_math::DetMath as _;

pub const SIM_HZ: f64 = 40.0;

/// **Qual lei da TINTA o motor corre** (F4-2 §5, ADR-0197): `Velha` = massa + cor por célula (a única que
/// pinta hoje); `Pigmento` = volumes e somas ópticas ([`crate::grid::tinta`]). Fora de [`Sim::lei`], que o
/// preset reescreve a cada aplicação. Enquanto nenhuma porta a lê (E4–E8), `Pigmento` só aloca os campos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LeiDaTinta {
    #[default]
    Velha,
    Pigmento,
}

impl LeiDaTinta {
    /// O nome da sonda (`PH2D_SONDA_LEI`).
    #[must_use]
    pub const fn nome(self) -> &'static str {
        match self {
            Self::Velha => "velha",
            Self::Pigmento => "pigmento",
        }
    }
}

impl std::str::FromStr for LeiDaTinta {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "velha" => Ok(Self::Velha),
            "pigmento" => Ok(Self::Pigmento),
            o => Err(format!("lei da tinta desconhecida: {o:?} (velha|pigmento)")),
        }
    }
}

/// Snapshot of every knob the solver reads, taken once per step so inner
/// loops never touch the registry (the JS `gatherParams`), plus the color
/// mixer routing.
///
/// `Clone` porque um passo INTERROMPIDO o carrega ([`StepCursor`]): os params são
/// capturados **uma vez por passo**, e retomar um passo tem de continuar com os
/// mesmos — se ele os re-colhesse no meio, um knob mexido entre dois frames
/// aplicaria metade de um passo com a lei nova.
#[derive(Clone)]
pub struct Params {
    values: [f64; KNOB_COUNT],
    pub mix: ColorMix,
    /// The advection's incoming-color mean also routes through K–M when ON.
    pub km_mixing: bool,
    /// A transparência do pigmento nas junções de duas tintas ([`crate::colorops::velatura`]).
    pub glaze: f64,
    /// A velatura do filme em suspensão sobre o assentado, no render ([`crate::render`]).
    pub glaze_filme: f64,
    /// A lei da gota deste passo ([`crate::solver::Gota`]) — `None` no port ou com a tela direita.
    pub(crate) gota: Option<crate::solver::Gota>,
    /// O pigmento que desce mais que a água ([`Sim::sedimenta`]).
    pub sedimenta: f64,
    /// O peso do ASSENTADO no que o pincel sujo apanha ([`Sim::pickup_assentado`]).
    pub pickup_assentado: f64,
    /// A lei do ecrã que assentar e re-molhar preservam ([`crate::assenta`]), com a velatura do filme.
    pub ecra: crate::assenta::Ecra,
    /// O «Glaze layering (K-M)» do pincel ([`Sim::camadas_transparentes`]): `1` = ligado. Pousa na célula
    /// ([`crate::grid::aparencia`]); a lei de cada célula lê o dela ([`crate::assenta::Ecra::com_vidrado`]).
    pub vidrado: f32,
    /// A Paint Texture do pincel ([`Sim::textura`]): `1` = ligada. Pousa na célula ([`crate::grid::aparencia`]).
    pub textura: f32,
    /// O Smooth Edges do pincel ([`Sim::suave`]), que pousa na célula ([`crate::grid::aparencia`]).
    pub suave: f32,
    /// O **Real Oil** do pincel ([`Sim::oleo`]): `1` = ligado. Pousa na célula ([`crate::grid::oleo_celula`]).
    pub oleo: f32,
    /// [`Sim::semente_do_po`].
    pub semente_do_po: u32,
    /// [`Sim::relevo_do_tamanho`].
    pub relevo_do_tamanho: f64,
    /// [`Sim::px_por_celula`].
    pub px_por_celula: f64,
    /// A água do dab ÷ a de um dab inteiro (`Engine::agua_do_dab`, `1 ÷` o peso do traço fino que a porta do produto
    /// dobra na opacidade): o FILME do traço ([`crate::tuning::Knob::StrokeFilm`]) desfá-lo para ter a mesma espessura a
    /// todo o tamanho. `1` fora da porta do produto.
    pub agua_do_dab: f64,
    /// O pigmento por unidade de depósito da RECEITA (`φ·h`, [`crate::painter::receita`]); `None` = o
    /// `PigmentPerDab` cru (quem não passa pela receita).
    pub pigmento_do_dab: Option<f64>,
    /// [`Sim::tinta`].
    pub tinta: LeiDaTinta,
    /// [`Sim::pigment_mix`].
    pub pigment_mix: f32,
    /// [`Sim::absorcao`].
    pub absorcao: f64,
    /// Os pesos `(x, y)` do emboss pela luz do cartão ([`crate::render::pesos_do_relevo`] de [`Sim::luz_do_relevo`]).
    pub relevo: [f64; 2],
    /// [`Sim::freio_na_grade_fina`].
    pub freio_na_grade_fina: bool,
}

impl Params {
    #[inline]
    pub fn k(&self, knob: Knob) -> f64 {
        self.values[knob as usize]
    }

    /// **A receita do dab** ([`crate::painter::receita`], E4): o depósito pousa `φ·h` de pigmento por unidade.
    /// O `PigmentPerDab` cru fica — é a parte de pigmento da receita, nunca a quantidade.
    #[must_use]
    pub fn com_receita(mut self, r: &crate::painter::receita::Receita) -> Self {
        self.pigmento_do_dab = Some(r.pigmento());
        self
    }

    /// O pigmento por unidade de depósito: o da receita, ou o `PigmentPerDab` cru.
    #[inline]
    #[must_use]
    pub fn pigmento_por_deposito(&self) -> f64 {
        self.pigmento_do_dab
            .unwrap_or(self.values[Knob::PigmentPerDab as usize])
    }

    /// **A retenção do filme por passe de secagem, com a Absorbency da tela** (E10d2): `r^A`. A fuga de 1.ª ordem
    /// do filme (`Knob::Retention`, `r = e^{−kΔt}`) é a água que o papel bebe; `r^A = e^{−A·kΔt}` é a mesma fuga com
    /// a taxa `A·k` — estável a todo `Δt` (`0 < r^A ≤ 1`) e com a massa que sai a levar o pigmento como hoje.
    /// `A = 1` devolve `r` sem `powf` (o motor de antes, ao bit).
    #[inline]
    #[must_use]
    pub fn retencao(&self) -> f64 {
        let r = self.values[Knob::Retention as usize];
        if self.absorcao == ABSORCAO_NEUTRA {
            r
        } else {
            r.det_powf(self.absorcao)
        }
    }
}

/// O ponto neutro da Absorbency ([`Sim::absorcao`]): a fuga de hoje, ao bit.
pub const ABSORCAO_NEUTRA: f64 = 1.0;
/// O tecto da Absorbency (E10d2, `docs/Painter/Arquitetura/E10d2_absorbency.md` §3): a `r = 0,995` de fábrica,
/// `r^A` com `A = 64` deixa `0,726` do filme por passe — o papel bebe a poça do Wet Round em ~2 s; acima disso o
/// filme some antes de o escoamento o mover (não há aguada a medir).
pub const ABSORCAO_MAX: f64 = 64.0;

/// Sim state that lives OUTSIDE the grid: cadence, tilt, experimental flags.
/// (The JS holds the grid reference here; in Rust the caller passes the grid
/// to [`sim_step`] — the facade owns which layer's grid simulates.)
#[derive(Clone)]
pub struct Sim {
    pub frame: u64,
    // Adaptive drying cadence, starts calm.
    pub dry_every: u64,
    pub evap_scale: f64,
    pub rewet_base: f64,
    /// **A cadência da secagem por tile 64×64** (B13.4, [`crate::drying::por_tile`]): `true` = o tile corre (seca
    /// com `evap_scale`/`rewet_base` a cada `dry_every`), `false` = calmo (o par calmo a cada 6). Vazio = tudo calmo.
    pub corre_tile: Vec<bool>,
    /// **B13.4 ligado?** `false` = a cadência global de antes (o aprovado; o mapa por tile nem se calcula).
    /// Omissão: [`crate::drying::por_tile::secagem_por_tile_por_omissao`] (`PH2D_SECAGEM_POR_TILE=1`).
    pub secagem_por_tile: bool,
    /// **B13.10 ligado?** `false` = o freio de antes (índice linear em células de fluxo), o aprovado.
    /// Omissão: [`crate::flow::freio_na_grade_fina_por_omissao`] (`PH2D_FREIO_NA_GRADE_FINA=1`). CPU e placa leem-no.
    pub freio_na_grade_fina: bool,
    // Gravity: the tilt dial provides direction + magnitude; boots ON
    // pointing straight down at the gravity knob's magnitude.
    pub tilt_on: bool,
    pub tilt_dir_x: f64,
    pub tilt_dir_y: f64,
    /// Dial radius fraction relative to the knob value.
    pub tilt_scale: f64,
    /// Tests may pin an exact vector.
    pub gravity_override: Option<[f64; 2]>,
    /// Experimental color routing (K–M pigment mixing checkbox).
    pub km_mixing: bool,
    /// A transparência do pigmento ([`crate::colorops::velatura`]): `0` = o port (junção por cima), o
    /// produto liga [`crate::colorops::VELATURA_MEDIDA`] — só com a mistura de fábrica: a K–M já é
    /// subtractiva, e somar-lhe a velatura leva um miolo molhado a PRETO (05 §4b «Mistura»).
    pub glaze: f64,
    /// A velatura do filme MOLHADO sobre o assentado, no render ([`crate::colorops::VELATURA_DO_FILME`]):
    /// `0` = o port.
    pub glaze_filme: f64,
    /// **A água corre com o traço ABERTO** (W1e, F6): o port pausa a sim enquanto o ponteiro está em
    /// baixo (só o Blow corre). A W1e ligou-o; a W1f DESLIGOU-o no produto — a 4096² o pior quadro a
    /// pintar ia de 2–4 ms a 14–45 ms (05 §4b); a mistura vê-se a pintar pelo pouso do dab.
    pub sim_while_painting: bool,
    /// **A lei da GOTA** (W1e, F1, [`crate::solver::Gota`]): a altura crítica da conta de baixo vezes
    /// `|g|` — `0` = o port (o freio de sempre); o produto liga o valor calibrado contra o Rebelle.
    pub drip_head: f64,
    /// **O pigmento DESCE mais que a água** (W1g, «Heavy Edge», [`crate::solver::sedimenta`]): a fracção
    /// por passo, por unidade de `|g|`, que o pigmento em suspensão passa à vizinha de baixo. `0` = o port.
    pub sedimenta: f64,
    /// **O pincel sujo apanha a tinta ASSENTADA?** O peso dela na cor que o bico recolhe, ao lado do
    /// suspenso (peso 1): `0,5` = o port; o produto `0` — o pincel só apanha tinta MOLHADA, como no
    /// Rebelle (W1e: rasto − azul ≈ 0 sobre o seco). Com `0,5` o amarelo apanhava o azul seco a cada dab e
    /// nunca o cobria (prompt 15, 05 §4b «Mistura»).
    pub pickup_assentado: f64,
    /// **Assentar e re-molhar preservam o ecrã?** ([`crate::assenta`]) `Porto` = as junções de antes; o
    /// produto liga a lei do Rebelle (`Cobre`).
    pub lei: crate::assenta::Lei,
    /// **A lei da tinta** ([`LeiDaTinta`]); muda-se em execução por [`Sim::set_tinta`]. `Velha` por omissão.
    pub tinta: LeiDaTinta,
    /// O «Pigment mix» `p ∈ [0,1]` do depósito na lei `Pigmento` (E10c; `0` = Oklab, igual ao bit). Sem painel.
    pub pigment_mix: f32,
    /// **A Absorbency da TELA** (E10d2): o múltiplo da taxa com que o papel bebe a água do filme
    /// ([`Params::retencao`]). Da tela, nunca do preset; [`ABSORCAO_NEUTRA`] = o motor de antes, ao bit.
    pub absorcao: f64,
    /// **O azimute da luz do cartão** (radianos, o do [`crate::oleo_real::OleoReal::azimute`]): o emboss da tinta
    /// segue-o; [`crate::render::AZIMUTE_DE_FABRICA`] = o de antes, ao bit.
    pub luz_do_relevo: f64,
    /// **A tinta molhada parece mais forte e clareia ao secar** (o Rebelle e a aguarela real; dono 10/10):
    /// a densidade óptica do pigmento em SUSPENSÃO vale este múltiplo no render (`a = 1 − (1 − a)^k`).
    /// Assentar preserva o ecrã SECO ([`crate::assenta`]), então o traço clareia aos poucos ao secar sem
    /// mudar de tom. `1` = o port.
    pub molhada_escurece: f64,
    /// **Quanto uma camada tapa a de baixo** na [`crate::assenta::Lei::Cobre`] (`h = 1 − (1 − a)^esconde`).
    pub esconde: f64,
    /// **«Glaze layering (K-M)»: camadas transparentes** — a camada tapa só a sua força
    /// ([`crate::colorops::TINTA_ESCONDE_EM_VELATURA`] no lugar de [`Sim::esconde`]). É do PINCEL (W3b): pousa
    /// na célula ([`Params::vidrado`]) e o ecrã e as transferências verticais lêem o da célula — mudá-lo não
    /// muda a tinta já pousada.
    pub camadas_transparentes: bool,
    /// **A Paint Texture do pincel** (o papel impresso na tinta, dono 28/10: é do TRAÇO): pousa na célula
    /// ([`Params::textura`]); o composite imprime o papel só onde a célula o pede — mudá-la não muda a tinta pousada.
    pub textura: bool,
    /// **O Smooth Edges do pincel** (`0..1`): pousa na célula ([`Params::suave`]); cada borda amacia pelo da tinta dela,
    /// e mudá-lo não muda a tinta já pousada (dono 04/11).
    pub suave: f32,
    /// **O Real Oil do pincel** (dono 28/10: é do TRAÇO): pousa na célula ([`Params::oleo`]); o composite corre a
    /// óptica do óleo onde a célula o tem — ligá-lo ou desligá-lo não muda a tinta pousada.
    pub oleo: bool,
    /// **A semente do PÓ deste dab** ([`Knob::Dust`]): o hospedeiro sorteia-a por dab do fluxo do traço
    /// (dabs parados no mesmo lugar pousam pontos novos); o motor mistura-lhe o centro do dab.
    pub semente_do_po: u32,
    /// **O relevo pelo diâmetro do traço** ([`Knob::EmbossSizeExp`]): o factor que o hospedeiro põe por lote e que
    /// multiplica o `Emboss` que a célula guarda ao receber a tinta. `1` = o de antes, ao bit.
    pub relevo_do_tamanho: f64,
    /// **Os px de tela de uma célula** (a razão da grade do hospedeiro): as leis calibradas em px (o domo do filme, a
    /// pena do [`Knob::WetOpenEdge`]) convertem-se por ela, para o mesmo pincel dar o mesmo traço a toda a razão. `1` =
    /// a razão de fábrica, ao bit.
    pub px_por_celula: f64,
    /// Force-skip every SPEC §17 extension code path (neutrality test §18.10).
    pub ext_bypass: bool,
    /// **O solver é INDEPENDENTE DE ORDEM?** `true` (o default) roda o
    /// [`crate::solver::advect_jacobi`] e a
    /// [`crate::drying::drying_pass_jacobi`]; `false` roda os dois
    /// Gauss-Seidel do port 1:1.
    ///
    /// ⚠️ **UM flag para os DOIS passes, e é deliberado:** eles são a mesma
    /// mudança (todo mundo lê o estado do INÍCIO do passe), shipam juntos e o
    /// smoke os julga juntos — dois bools sugeririam uma combinação que
    /// ninguém projetou.
    ///
    /// ⚠️ **É uma ABLAÇÃO, no molde do [`crate::grid::Grid::spans_enabled`]**,
    /// e ela existe por dois motivos que nenhum outro mecanismo dá: o pino de
    /// fingerprint ANTIGO do ADR-0134 continua sendo um gate executável (a
    /// rota `false` tem de reproduzi-lo **ao byte**, provando que nada além
    /// destes dois passes mudou), e um relatório de campo se bissecta com um
    /// bool em vez de com um build.
    ///
    /// ⚠️ **NÃO é "dois motores"** — o serial é o **caminho de referência**
    /// (CLAUDE.md §0), e quem manda no produto é a forma que 32 núcleos (ou
    /// 10.000) podem responder.
    pub order_invariant: bool,
    /// **Um passo INTERROMPIDO** — ver [`StepCursor`] e [`sim_step_stage`].
    pending: Option<StepCursor>,
}

/// **Onde um passo interrompido parou.**
///
/// Um passo é uma sequência de sete estágios com cadências próprias, e na escala do produto ele custa
/// **38,7 ms** (medido, `ph2d-wet-paint/tests/it/measure_pass_cost.rs`) — mais que **dois quadros** de
/// 60 Hz. Enquanto ele era ATÔMICO, o frame que o continha estourava por construção, e nenhum
/// orçamento conserta isso: o smoke do Enio mediu `tool-tick pico 73,70ms` com o app a 55 fps.
///
/// O maior estágio SOZINHO custa 10,26 ms (`build_flow_field`), que **cabe** na folga de um quadro.
/// Então o passo deixou de ser atômico: o chamador roda estágios enquanto tem orçamento e retoma no
/// frame seguinte.
///
/// ⚠️ **Byte-idêntico por CONSTRUÇÃO, e é o desenho inteiro:** os mesmos estágios, na mesma ordem, com
/// os MESMOS params — quem os capturou foi o início do passo, não cada estágio. [`sim_step`] passou a
/// ser *o laço sobre os estágios*, então não existe uma segunda implementação para divergir.
#[derive(Clone)]
pub struct StepCursor {
    /// Os params CAPTURADOS no início do passo (a razão de [`Params`] ser `Clone`).
    p: Params,
    grav: [f64; 2],
    /// O `sim.frame` deste passo — as cadências (`n % 2`, `n % 3`, `n % 4`) o leem.
    n: u64,
    /// O próximo estágio a rodar (ver [`sim_step_stage`]).
    stage: u8,
    /// O `vmax` que o `advect` devolveu — o estágio final o converte em cadência de secagem.
    vmax: f64,
    /// O mapa «corre?» por tile deste advect ([`Sim::corre_tile`] dos passos seguintes).
    corre: Vec<bool>,
}

#[path = "sim_escolhas.rs"]
mod escolhas;
pub use escolhas::com_escolhas;

impl Default for Sim {
    fn default() -> Self {
        Sim {
            frame: 0,
            dry_every: 6,
            evap_scale: 0.001,
            rewet_base: 0.0001,
            corre_tile: Vec::new(),
            secagem_por_tile: escolhas::de_um_sim_novo().1,
            freio_na_grade_fina: escolhas::de_um_sim_novo().0,
            tilt_on: true,
            tilt_dir_x: 0.0,
            tilt_dir_y: 1.0,
            tilt_scale: 1.0,
            gravity_override: None,
            km_mixing: false,
            glaze: 0.0,
            glaze_filme: 0.0,
            sim_while_painting: false,
            drip_head: 0.0,
            sedimenta: 0.0,
            pickup_assentado: 0.5,
            lei: crate::assenta::Lei::Porto,
            tinta: LeiDaTinta::Velha,
            pigment_mix: 0.0,
            absorcao: ABSORCAO_NEUTRA,
            luz_do_relevo: crate::render::AZIMUTE_DE_FABRICA,
            molhada_escurece: 1.0,
            esconde: 1.0,
            camadas_transparentes: false,
            textura: false,
            suave: 0.0,
            oleo: false,
            semente_do_po: 0,
            relevo_do_tamanho: 1.0,
            px_por_celula: 1.0,
            ext_bypass: false,
            order_invariant: true,
            pending: None,
        }
    }
}

impl Sim {
    /// A cadência da secagem dos passos seguintes, do `vmax` do advect e do mapa por tile (B13.4): a porta global
    /// é 3 se algum tile corre, senão 6; o par lento é o do tile a correr.
    pub fn cadencia(&mut self, vmax: f64, corre: Vec<bool>) {
        if vmax < crate::drying::por_tile::VMAX_CORRE {
            self.dry_every = crate::drying::por_tile::CADENCIA_CALMA;
            self.evap_scale = crate::drying::por_tile::EVAP_CALMO;
            self.rewet_base = crate::drying::por_tile::REWET_CALMO;
        } else {
            self.dry_every = 3;
            self.evap_scale = 0.00025;
            self.rewet_base = 0.000025;
        }
        self.corre_tile = corre;
    }

    /// O mapa por tile do passe de secagem `n` sobre `g`; `None` = tudo calmo (ou o mapa é de outra grade).
    pub fn seca_por_tile(
        &self,
        g: &Grid,
        p: &Params,
        n: u64,
    ) -> Option<crate::drying::SecaPorTile<'_>> {
        let (nx, ny) = crate::drying::por_tile::tiles_da_grade(g);
        (self.secagem_por_tile && !self.corre_tile.is_empty() && self.corre_tile.len() == nx * ny)
            .then(|| crate::drying::SecaPorTile {
                corre: &self.corre_tile,
                nx,
                calmo_seca: n.is_multiple_of(crate::drying::por_tile::CADENCIA_CALMA),
                evap_calmo: crate::drying::por_tile::EVAP_CALMO * p.k(Knob::Evaporation),
                rewet_calmo: crate::drying::por_tile::REWET_CALMO * p.k(Knob::Rewet),
            })
    }

    /// Troca a lei da tinta em execução (o preset não a toca). Os campos da lei nova nascem no próximo passo.
    pub fn set_tinta(&mut self, lei: LeiDaTinta) {
        self.tinta = lei;
    }

    /// Há um passo interrompido em voo?
    #[must_use]
    pub const fn step_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// O próximo estágio do passo em voo (`≥ 1`), ou `None` sem passo em voo. Com [`Sim::frame`] é o
    /// ESTADO da água entre dois estágios — a marca que uma macro grava quando o motor volta do worker.
    #[must_use]
    pub fn stage_in_flight(&self) -> Option<u8> {
        self.pending.as_ref().map(|c| c.stage)
    }

    /// O cursor do passo em voo: o registo do histórico guarda-o ao abrir (abre a meio do passo sem o drenar,
    /// [`crate::painter::Engine::capture_history`]) e o undo/redo do `Engine` repõe-no.
    #[must_use]
    pub(crate) fn cursor_do_passo(&self) -> Option<StepCursor> {
        self.pending.clone()
    }

    /// Repõe o cursor de [`Self::cursor_do_passo`].
    pub(crate) fn repoe_cursor_do_passo(&mut self, c: Option<StepCursor>) {
        self.pending = c;
    }

    /// **Termina um passo em voo** — a porta que toda operação de CANVAS chama primeiro.
    ///
    /// O grid entre dois estágios é intermediário: `wet_canvas`/`dry_canvas`/`fast_dry`/`clear` agem sobre a
    /// folha INTEIRA, e fazê-lo no meio de um passo misturaria meia física com a ação. Sem passo em voo é um
    /// `if` e nada mais. (O `capture_history` não drena: o registo é por tiles, os estágios que faltam marcam-se
    /// nele e ele guarda o cursor, que o undo/redo do engine repõe.)
    pub fn drain_step(&mut self, g: &mut Grid, tuning: &Tuning) {
        while self.pending.is_some() {
            sim_step_stage(self, g, tuning);
        }
    }
}

/// The tilt DIAL's direction for a spoke (0..11, 30° steps, 0 = +x, going
/// clockwise in screen space like the reference dial). The four cardinals
/// return EXACT unit vectors — the boot spoke is 3 (straight down) and the
/// engine boots `tilt_dir = (0, 1)` exactly, so a boot-time reconcile must be
/// a bit-exact no-op (`libm::cos(PI/2)` is `6.1e-17`, not `0`); the other
/// spokes take the reference's raw cos/sin.
#[must_use]
pub fn tilt_dir_for_spoke(spoke: u8) -> [f64; 2] {
    match spoke % 12 {
        0 => [1.0, 0.0],
        3 => [0.0, 1.0],
        6 => [-1.0, 0.0],
        9 => [0.0, -1.0],
        s => {
            let a = f64::from(s) * 30.0 * std::f64::consts::PI / 180.0;
            [libm::cos(a), libm::sin(a)]
        }
    }
}

impl Sim {
    /// The effective gravity vector for this step.
    pub fn gravity(&self, tuning: &Tuning) -> [f64; 2] {
        if let Some(g) = self.gravity_override {
            return g;
        }
        if !self.tilt_on {
            return [0.0, 0.0];
        }
        let mag = tuning.get(Knob::Gravity) * self.tilt_scale;
        [self.tilt_dir_x * mag, self.tilt_dir_y * mag]
    }

    /// Snapshot every knob + the color routing (the JS `gatherParams`).
    pub fn gather_params(&self, tuning: &Tuning) -> Params {
        Params {
            values: tuning.snapshot(),
            mix: if self.km_mixing {
                ph2d_pigment::espectral::lei_do_wet()
            } else {
                ColorMix::Plain
            },
            km_mixing: self.km_mixing,
            glaze: self.glaze,
            glaze_filme: self.glaze_filme,
            gota: {
                let [gx, gy] = self.gravity(tuning);
                crate::solver::Gota::new(self.drip_head, gx, gy)
            },
            sedimenta: self.sedimenta,
            pickup_assentado: self.pickup_assentado,
            ecra: match self.lei {
                crate::assenta::Lei::Porto => crate::assenta::Ecra::Porto,
                crate::assenta::Lei::PorCima => crate::assenta::Ecra::PorCima(self.glaze_filme),
                crate::assenta::Lei::Camada => crate::assenta::Ecra::Camada,
                crate::assenta::Lei::Cobre => crate::assenta::Ecra::Cobre {
                    opaca: 0.0,
                    // O óleo é da CÉLULA (E10e1): o Real Oil na mão não entra na lei da sessão.
                    oleo: 0.0,
                    filme: self.glaze_filme,
                    esconde: self.esconde,
                    molhada: self.molhada_escurece,
                    vela: 0.0,
                },
            },
            vidrado: if self.camadas_transparentes { 1.0 } else { 0.0 },
            textura: if self.textura { 1.0 } else { 0.0 },
            suave: self.suave,
            oleo: if self.oleo { 1.0 } else { 0.0 },
            semente_do_po: self.semente_do_po,
            relevo_do_tamanho: self.relevo_do_tamanho,
            px_por_celula: self.px_por_celula,
            agua_do_dab: 1.0,
            pigmento_do_dab: None,
            tinta: self.tinta,
            pigment_mix: self.pigment_mix,
            absorcao: self.absorcao,
            relevo: crate::render::pesos_do_relevo(self.luz_do_relevo),
            freio_na_grade_fina: self.freio_na_grade_fina,
        }
    }
}

/// Advance one fixed 40 Hz step. Returns false when the sim is idle (no fluid).
/// **UM passo, atomicamente** — o laço sobre [`sim_step_stage`].
///
/// ⚠️ Ele é o LAÇO e não uma segunda implementação: a suíte de aceitação e o fingerprint pinado
/// entram por aqui, então não existe caminho paralelo para divergir do que o produto roda por
/// estágios.
pub fn sim_step(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> bool {
    loop {
        if let Some(ran) = sim_step_stage(sim, g, tuning) {
            return ran;
        }
    }
}

/// Roda **UM estágio** do passo corrente (começando um, se não houver).
///
/// - `Some(ran)` — o passo COMPLETOU nesta chamada; `ran` é o que [`sim_step`] devolveria.
/// - `None` — um estágio rodou e o passo continua no próximo chamado.
///
/// O chamador roda estágios enquanto tem orçamento de frame; ver [`StepCursor`] para o porquê.
///
/// ⚠️ **Um passo em voo é estado do `Sim`, e quem faz uma operação de CANVAS tem de drená-lo**
/// ([`Sim::drain_step`] / `Engine::drain_step`) — o grid entre dois estágios é intermediário, e um
/// `wet_canvas`/`dry_canvas`/`fast_dry` sobre ele misturaria meia física com a ação. O `capture_history`
/// NÃO drena: o registo guarda o cursor e o undo/redo repõe-no com as grades.
pub fn sim_step_stage(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> Option<bool> {
    let mut c = match sim.pending.take() {
        Some(c) => c,
        None => {
            // Começo de passo: o guard, os params (UMA vez — ver `StepCursor`) e o relógio.
            if !g.has_fluid {
                return Some(false);
            }
            if sim.tinta == LeiDaTinta::Pigmento {
                crate::grid::tinta::garante(g);
            }
            let p = sim.gather_params(tuning);
            let grav = sim.gravity(tuning);
            crate::grid::sono::inicio(g, crate::grid::sono::chave(&p.values, grav));
            sim.frame += 1;
            StepCursor {
                p,
                grav,
                n: sim.frame,
                stage: 0,
                vmax: 0.0,
                corre: Vec::new(),
            }
        }
    };
    let n = c.n;
    match c.stage {
        0 => {
            if n.is_multiple_of(2) {
                rebuild_active_region(g);
                if !g.has_fluid {
                    // Aborta o passo: nada de cadência, nada de cursor.
                    return Some(false);
                }
            }
        }
        1 => {
            if n.is_multiple_of(sim.dry_every) {
                // The evaporation / re-wet knobs are straight multipliers on the
                // cadence-adaptive scales.
                let (evap, rewet) = (
                    sim.evap_scale * c.p.k(Knob::Evaporation),
                    sim.rewet_base * c.p.k(Knob::Rewet),
                );
                let t = sim.seca_por_tile(g, &c.p, n);
                if sim.order_invariant {
                    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
                    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
                    let mode =
                        crate::par::Rows::pick_medido(rows, span, crate::par::MIN_CELLS_DRYING);
                    crate::drying::drying_pass_jacobi_por_tile(
                        g,
                        &c.p,
                        evap,
                        rewet,
                        sim.ext_bypass,
                        mode,
                        t.as_ref(),
                    );
                } else {
                    crate::drying::drying_pass_por_tile(
                        g,
                        &c.p,
                        evap,
                        rewet,
                        sim.ext_bypass,
                        t.as_ref(),
                    );
                }
            }
            if c.p.tinta == LeiDaTinta::Pigmento && n.is_multiple_of(crate::drying::PASSOS_DA_CURA)
            {
                crate::drying::cura(g, crate::drying::PASSOS_DA_CURA as f32 / SIM_HZ as f32);
            }
        }
        2 => {
            crate::grid::sono::adormece(g);
            crate::grid::sono::estagio(g, 2);
            if n.is_multiple_of(4) {
                if sim.order_invariant {
                    crate::solver::build_flow_field_jacobi(
                        g,
                        &c.p,
                        c.grav[0],
                        c.grav[1],
                        sim.ext_bypass,
                    );
                } else {
                    build_flow_field(g, &c.p, c.grav[0], c.grav[1], sim.ext_bypass);
                }
            } else {
                smooth_velocity(g, &c.p);
                if !sim.ext_bypass && c.p.k(Knob::ExtDiffusion) > 0.0 {
                    diffusion_pass(g, &c.p);
                }
            }
        }
        3 => {
            crate::grid::sono::estagio(g, 3);
            c.vmax = if sim.order_invariant {
                crate::solver::advect_jacobi(g, &c.p, c.grav[0], c.grav[1])
            } else {
                advect(g, &c.p, c.grav[0], c.grav[1])
            };
            if c.p.sedimenta > 0.0 {
                let [gx, gy] = c.grav;
                let a = c.p.sedimenta * (gx * gx + gy * gy).sqrt();
                crate::solver::sedimenta(g, a, gx, gy, c.p.km_mixing);
            }
            if sim.secagem_por_tile {
                c.corre = crate::drying::por_tile::corre_por_tile(g, c.vmax);
            }
            crate::estreito::arredonda(g);
        }
        4 => {
            crate::grid::sono::estagio(g, 4);
            apply_boundaries(g, false);
        }
        5 => {
            crate::grid::sono::estagio(g, 5);
            if n.is_multiple_of(3) {
                project(g, &c.p);
                apply_boundaries(g, true);
            }
        }
        _ => {
            // Adaptive drying cadence for the NEXT frames, from this advection's max
            // velocity component: calm water dries faster per pass, flowing water is
            // dried gently but often.
            sim.cadencia(c.vmax, std::mem::take(&mut c.corre));
            crate::estreito::arredonda(g);
            crate::grid::sono::conta(g);
            return Some(true);
        }
    }
    c.stage += 1;
    sim.pending = Some(c);
    None
}

#[cfg(test)]
#[path = "sim_resumable_tests.rs"]
mod sim_resumable_tests;

/// Fast dry action (SPEC §12), routed here so it shares params.
pub fn sim_fast_dry(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) {
    if !g.has_fluid {
        return;
    }
    let p = sim.gather_params(tuning);
    fast_dry(g, &p, sim.rewet_base * p.k(Knob::Rewet), sim.ext_bypass);
}
