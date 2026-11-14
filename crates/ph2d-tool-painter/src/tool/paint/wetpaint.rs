//! **Wet Paint** ([`PaintMode::WetPaint`]) — the `ph2d-wet-paint` fluid engine
//! as a paint mode (ADR-0134). The MODE is the master switch: no `BrushSpec`
//! flag exists on purpose (two doors to one question diverge — the Knife
//! precedent), so "off" means "any other mode", and the OFF contract is that
//! no other mode ever constructs a session (gated below).
//!
//! ## The session model (the watercolor wet-session, generalized)
//!
//! A session freezes the canvas (`base`, a shared `Arc` — O(1)) and owns an
//! engine grid sized to it; every composite re-renders `pigment OVER base`
//! into `canvas_rgba` for exactly the engine's dirty rect. The session spans
//! STROKES — the water stays live between pen-ups, which is the module's
//! whole point — and it is **display-state, not document-state**: the pixels
//! the artist sees are always IN `canvas_rgba`, so ending a session is the
//! bake (nothing to write), and the per-stroke undo capture at pen-down
//! already holds the look. The engine grid therefore stays OUT of
//! `ModelSnapshot` — a `GridSnapshot` per undo step would be ~14 f32 planes
//! per canvas (~235 MB at 2048², the ADR-0117 disease).
//!
//! What guards that stance is the **canvas-identity guard** (the watercolor's
//! `wet_session_canvas`, made eager): any foreign mutation — undo, layer
//! switch, fill, resize, another tool — swaps the `canvas_rgba` Arc, and the
//! next wet-paint touch (dab OR tick) sees `Arc::ptr_eq` fail and ends the
//! session. The tick half is load-bearing: the sim composites WITHOUT a
//! pen-down, so a lazy at-pen-down check would let a live session repaint
//! over a canvas the undo just restored.
//!
//! ## Coordinates and dab mapping
//!
//! Engine cells are 1-based with a pad ring: canvas pixel `(0..W-1)` maps to
//! cell `(1..W)` (the reference app's `view.toCell`: `x = px + 1`). Dabs go
//! through the engine's OWN §9 parameter mapping (`dispatch_pressure_dab`)
//! with exactly two host substitutions: pressure = `coverage / strength`
//! (the dab's real pressure response, rescaled to §8's ~0..10 range) and
//! radius = the dab's real `radius_px`. Colour is taken at stroke start
//! (per-dab Randomize colour is a W2 seam).
//!
//! ## Authoring vs deposit (doc 21 — deposit-at-commit)
//!
//! The re-stamping methods (DragDot / Anchored / the shape editors) AUTHOR
//! through the normal flat pipeline (their batches are un-owned — see
//! [`PainterTool::wet_owns_the_dabs`]) and the fluid receives the FINAL dab
//! list exactly once, at commit (pen-up / Enter / Apply), through the
//! commit door (`wetpaint_commit`). The impasto height pass never runs in
//! this mode (relief pinned to a footprint would outlive paint that flows
//! away — gated in `stamp_dabs_inner`). A live deposit still only happens
//! inside a real incremental gesture: re-stamp previews piling paint into a
//! non-idempotent fluid while the artist just LOOKS (the I2 disease) is the
//! exact failure the ownership split + the route's belt refuse.

use super::*;
use offthread::{EngineSlot, SimWorker};
use ph2d_wet_paint::painter::{Dirty, Engine};
use std::sync::Weak;

/// O estado autorado do Wet Paint — o arm, a sessão viva, os knobs, o tool e o
/// stash do commit.
///
/// ⚠️ **Nem o passo nem o orçamento de tempo moram mais aqui.** A sim roda numa
/// thread própria ([`offthread`]) e o relógio dela é o do WORKER, então este
/// tool não tem mais `acc`, não tem cap de contagem (`WET_MAX_STEPS`) e não tem
/// orçamento de milissegundos (`SimBudget`): o frame não paga passo nenhum, ele
/// MOSTRA o que o worker já fez. Os seis defeitos que a era do agendamento
/// fechou — realimentação em `dt`, orçamento fixo, atribuição, catraca, régua
/// pregada e o passo atômico — ficam registrados no doc 28 §5.31-§5.37, e o
/// passo interrompível que a última delas construiu é **o que torna esta wave
/// possível**: o worker devolve o motor em fronteira de ESTÁGIO (3-10 ms), não
/// de passo (33-38).
#[derive(Clone)]
pub(crate) struct WetPaintState {
    /// The Wet Paint **checkbox** — the authored, persistent ARM (Enio
    /// 2026-07-21, the Watercolor/Impasto pattern): while `true`, the
    /// `"brush"` wire resolves to [`PaintMode::WetPaint`], so leaving to the
    /// eraser / selection / any tool and coming back returns to the FLUID
    /// instead of the plain digital brush. One fact, mode-independent, not a
    /// `BrushSpec` field (per-slot copies of one truth would disagree).
    /// Entering the mode by ANY door arms it — a checkbox that reads OFF
    /// while the paint is wet would be the lying radio this file refuses.
    pub(super) armed: bool,
    /// The live session; `None` until the first Wet Paint dab lands.
    pub(super) session: SessaoDoWet,
    /// A tinta viva das sessões que saíram do Wet ou da camada ([`adormecida`]), a mais antiga primeiro. Sem tique
    /// nem worker.
    pub(super) adormecidas: Vec<adormecida::TintaAdormecida>,
    /// O sulco do traço aberto da Borracha na tinta morta ([`borracha_morta`]).
    pub(super) sulco: Option<borracha_morta::Sulco>,
    /// Os dabs vão medir a cobertura num escudo, não pintar ([`PainterTool::mede_o_escudo`]).
    pub(super) medindo_o_escudo: bool,
    /// A miniatura da Borracha mostra a folha sob o S: ela apaga a direito, sem tinta para amassar.
    pub(super) na_miniatura: bool,
    /// A live freehand paint GESTURE is open (`paint_begin` .. pen-up). This —
    /// not `paint.stroke` — is the deposit gate: the lifecycle `mem::take`s
    /// the stroke while stamping, and the shape editors' per-frame re-stamps
    /// never run `paint_begin` at all, so the flag is exactly "dabs are the
    /// artist's hand, once".
    pub(super) live_gesture: bool,
    /// The authored knob values (W3) — survive the session, the mode and the
    /// tool round-trips; the section reset restores [`WetKnobs::default`].
    pub(super) knobs: WetKnobs,
    /// Os knobs do pincel que abriu (B35.8): o ↺ da linha e do grupo do Tuning voltam a ELE; a fábrica até um preset.
    pub(super) preset_base: WetKnobs,
    /// Doc 21 (deposit-at-commit): the commit-deposit STASH — the exact batch
    /// the last flat authoring preview painted (`stamp_drag_preview`'s own
    /// parameter, untiled/mirrored; the dispatcher re-tiles at deposit
    /// exactly as it did at preview). The deposit IS the preview by
    /// construction — nothing is re-derived at commit. Transient, never
    /// serialized; cleared by pen-down, cancel, mode-leave and teardown.
    pub(super) pending_deposit: Vec<Dab>,
    /// Doc 21: TRUE only across the single `stamp_dabs` replay inside
    /// `wetpaint_commit_deposit` — the third key on the wet arm's gate. A
    /// leak of this flag while a preview re-stamp runs is I2 resurrected
    /// (every refill becomes a fluid deposit); it is written in exactly two
    /// adjacent statements around that one call, nowhere else.
    pub(super) deposit_pass: bool,
    /// The wet TOOL (doc 22): which of the model's tools the brush wire
    /// drives while armed. Paint by default; Erase is NOT here — it is the
    /// rail eraser's other view.
    pub(super) tool: WetTool,
    /// The tilt DIAL (doc 22): on/off + ring (0..8, magnitude `ring/4`) +
    /// spoke (0..11, 30° steps). Boot = the model's boot = the engine's
    /// boot: ON, straight down, half radius (gate-pinned no-op reconcile).
    pub(super) tilt_on: bool,
    pub(super) tilt_ring: u8,
    pub(super) tilt_spoke: u8,
    /// Show-wet overlay (display-only; NEVER baked — the end-session door
    /// recomposites clean first).
    pub(super) show_wet: bool,
    /// A **Paint Texture** (doc 22 §2.8): o papel impresso na tinta (granulação + emboss nas cores do pigmento). É do
    /// TRAÇO (dono 28/10): pousa na célula (`grid::aparencia::textura`), logo viaja com o preset e com a ferramenta
    /// ([`Self::textura_por_slot`]), como o Glaze.
    pub(super) paper_visual: bool,
    /// O Smooth Edges de uma macro gravada quando ele era da TELA: vale para os presets sem a chave `smoothEdges` que ela
    /// aplica a tocar, como valia então. Fora de uma dessas macros, `None`. O valor vivo é o `Knob::SmoothEdges`.
    pub(super) suave_da_macro_antiga: Option<f64>,
    /// **Absorbency** (E10d2): a taxa com que o papel bebe a água do filme, em múltiplos da de fábrica
    /// (`Sim::absorcao`, [`ph2d_wet_paint::sim::ABSORCAO_NEUTRA`] = o motor de antes, ao bit). É da TELA (secção
    /// comum Paper): fora dos `WetKnobs` e de todo preset.
    pub(super) absorcao: f64,
    /// **Óleo Real** (doc 49): a caixa do PINCEL — do TRAÇO (dono 28/10): pousa na célula (`grid::oleo_celula`), logo
    /// viaja com o preset e com a ferramenta ([`Self::oleo_por_slot`]), como o Glaze. `None` de fábrica.
    pub(super) oleo_real: Option<ph2d_wet_paint::oleo_real::OleoReal>,
    /// A tela tem tinta com óleo (alguma camada visível tem o campo `grid::oleo`): o composite corre a óptica com
    /// os números do cartão e o cartão mostra-os mesmo com a caixa do pincel desligada. Escrito no composite.
    pub(super) oleo_na_tela: bool,
    /// O preview do preset adia a óptica do óleo até ao quadro final (U4 W3b: custo); `false` fora dele.
    pub(super) oleo_adiado: bool,
    /// A caixa [`Self::oleo_real`] de cada slot do pincel (`switch_brush_slot`).
    pub(super) oleo_por_slot: [bool; super::SLOT_COUNT],
    /// Os últimos valores dos knobs do **Real Oil** (os do painel): desligar e voltar a ligar a caixa
    /// devolve-os; ligada, `oleo_real == Some(oleo_real_valores)`.
    pub(super) oleo_real_valores: ph2d_wet_paint::oleo_real::OleoReal,
    /// **Live Water** (W1g): a água corre com o traço aberto. Desligado de fábrica = a W1f, ao byte.
    pub(super) agua_viva: bool,
    /// **Heavy Edge** (W1g, `0..1`): o pigmento desce mais que a água na tela inclinada. `0` = a W1f, ao byte.
    pub(super) borda_que_pesa: f64,
    /// **Pigment mixing** (K–M, `sim.km_mixing`): do pincel, ligado de fábrica (dono 30/10).
    pub(super) km_mixing: bool,
    /// O **Glaze layering (K-M)** do pincel: é do TRAÇO (pousa na célula como `vidrado`), logo viaja com o preset
    /// e com a ferramenta ([`Self::glaze_por_slot`]), como os knobs.
    pub(super) km_glaze: bool,
    /// The Tuning side panel's visibility (the basic section's checkbox).
    pub(super) tuning_open: bool,
    /// **Quantos pixels de canvas medem uma célula de fluido** (1..=30, o
    /// slider no topo da seção; ver [`grid_map`]). `1` é a grade de sempre.
    ///
    /// ⚠️ **Autorado, e trocá-lo ENCERRA a sessão** — a grade tem dimensão, e
    /// uma sessão viva já a tem congelada em `WetSession::ratio`. Encerrar é o
    /// BAKE (a tinta que se vê já está no `canvas_rgba`), então nada é perdido;
    /// a alternativa — reamostrar catorze planos de `f32` para a resolução
    /// nova — inventaria água que o solver não produziu, e faria da razão um
    /// parâmetro que altera o passado em vez do futuro.
    pub(super) grid_ratio: u8,
    /// Quantas células de FLUIDO medem uma célula de FLUXO (plano 30). `1` = a
    /// grade de fluxo É a fina, que é o motor que sempre shipou.
    pub(super) flow_ratio: u8,
    /// **O relógio FIXO** (gates e renderização fora do ecrã): `Some(dívida)` ⇒ o tique avança a água
    /// SINCRONAMENTE pelo [`Self::relogio_fixo`] e nunca a entrega ao worker. O worker anda ao relógio
    /// REAL, e sob carga duas corridas iguais divergiam em 1 148 texels (o censo dos controlos,
    /// 2026-10-03): um gate que pergunta «este controlo muda a tinta?» precisa da mesma tinta duas vezes.
    /// A miniatura de um preset (`wetpaint_preset_miniatura.rs`) pinta por ele pela mesma razão.
    pub(super) relogio_fixo: Option<f64>,
    /// N10 F2j: o pen-up ADIADO — `Some(traço)` = o `close_stroke` espera a sessão assentar ([`ciclo`]); toda porta
    /// que lê o documento ou o undo conclui-o antes (`conclui_o_fecho_adiado`).
    pub(super) fecho_adiado: Option<Option<ph2d_painter_brush::Stroke>>,
    /// O pedido do bloco de presets que espera a ponte da app (o disco é dela, não do tool).
    pub(super) preset_pedido: Option<super::wetpaint_preset::PedidoPreset>,
    /// **A largura prometida (W2)**: a lei raio do motor → largura do pincel de agora (a do preset; sem
    /// preset, a do Wet Round, que é o pincel de fábrica). `None` = a lei CRUA — só a sonda que a mede.
    pub(super) largura: Option<largura::LeiDaLargura>,
    /// A [`Self::largura`] de cada slot do pincel (a das ferramentas fora da mão, `switch_brush_slot`).
    pub(super) largura_por_slot: [Option<largura::LeiDaLargura>; super::SLOT_COUNT],
    /// Os knobs de cada slot do pincel; só os da mão ([`super::mao::knobs_da_mao`]) voltam com ele.
    pub(super) knobs_por_slot: [[f64; ph2d_wet_paint::tuning::KNOB_COUNT]; super::SLOT_COUNT],
    /// O [`Self::km_glaze`] de cada slot do pincel (`switch_brush_slot`).
    pub(super) glaze_por_slot: [bool; super::SLOT_COUNT],
    /// O [`Self::paper_visual`] de cada slot do pincel (`switch_brush_slot`).
    pub(super) textura_por_slot: [bool; super::SLOT_COUNT],
    /// O Water do topo (`knobs.water`) de cada slot do pincel: a ferramenta Wet lê-o como o Pincel
    /// ([`crate::LinhaDaFerramenta::AguaDoVolume`]), e o preset dela não o pode deixar no Pincel (`switch_brush_slot`).
    pub(super) agua_por_slot: [f64; super::SLOT_COUNT],
    /// A fita da macro ([`fita`]) que cada sessão nova herda.
    pub(super) fita: Option<fita::FitaPartilhada>,
    /// O ritmo escrito do worker ([`fita::RitmoDaAgua`]) que cada sessão nova herda — só gates.
    pub(super) ritmo: Option<fita::RitmoDaAgua>,
    /// O estado da água que a tela mostra (o do último composite da sessão viva) — W14.
    pub(super) composta: Option<fita::MarcaDaAgua>,
    /// A recomposição que o painel pediu e o tick ainda varre ([`oleo_no_quadro`]).
    pub(super) varredura: oleo_no_quadro::Varredura,
    /// Os dabs deste quadro deixaram a tela por compor ([`por_quadro`]): o sujo do motor acumula a união.
    pub(super) composite_pendente: bool,
    /// A lei da tinta que cada sessão nova herda ([`PainterTool::set_wet_lei_da_tinta`]).
    pub(super) lei_da_tinta: ph2d_wet_paint::sim::LeiDaTinta,
    /// O motor e o papel da próxima sessão, já prontos ([`preparo`]).
    pub(super) preparo: preparo::PreparoDoToque,
}

/// A **Heavy Edge** no `1` do slider, em `Sim::sedimenta` (a fracção por passo por unidade de `|g|`).
/// Medida na rodada da W1g (05 §4b): a meia, o motor a 5 dá a orla de baixo ÷ de cima do Rebelle (1,56;
/// Rebelle 1,6) ⇒ o meio do slider; a 10 ela é 4,1 e a régua do recto continua verde.
/// ⚠️ **10 → 0,3 com a tinta do Rebelle (pigmento baixo, 10/10)**: a tinta clara não satura, e o mesmo
/// deslocamento esvaziava o topo (orla 7,5 a 5). Medido com a régua relativa (`gota_tests::le`), Cold a meia:
/// sedimenta 0 · 0,1 · 0,15 · 0,2 · 0,3 · 0,5 → 1,27 · 1,41 · ~1,48 · 1,55 · 1,70 · 2,00 (sem papel 1,62 →
/// 2,56). O meio do slider (0,15) dá ~1,48 (Rebelle 1,6).
pub(super) const SEDIMENTA_DA_BORDA: f64 = 0.3;

impl Default for WetPaintState {
    fn default() -> Self {
        Self {
            armed: false,
            session: SessaoDoWet::default(),
            adormecidas: Vec::new(),
            sulco: None,
            medindo_o_escudo: false,
            na_miniatura: false,
            live_gesture: false,
            knobs: WetKnobs::default(),
            preset_base: WetKnobs::default(),
            pending_deposit: Vec::new(),
            deposit_pass: false,
            tool: WetTool::default(),
            tilt_on: true,
            tilt_ring: 4,
            tilt_spoke: 3,
            show_wet: false,
            paper_visual: false,
            suave_da_macro_antiga: None,
            absorcao: ph2d_wet_paint::sim::ABSORCAO_NEUTRA,
            oleo_real: None,
            oleo_na_tela: false,
            oleo_adiado: false,
            oleo_por_slot: [false; super::SLOT_COUNT],
            oleo_real_valores: ph2d_wet_paint::oleo_real::OleoReal::default(),
            agua_viva: false,
            borda_que_pesa: 0.0,
            km_mixing: true,
            km_glaze: false,
            tuning_open: false,
            grid_ratio: grid_map::DEFAULT_RATIO,
            flow_ratio: 1,
            relogio_fixo: None,
            fecho_adiado: None,
            preset_pedido: None,
            largura: Some(largura::LeiDaLargura::de_omissao()),
            largura_por_slot: std::array::from_fn(|_| Some(largura::LeiDaLargura::de_omissao())),
            knobs_por_slot: [super::WetKnobs::DEFAULT.knobs; super::SLOT_COUNT],
            glaze_por_slot: [false; super::SLOT_COUNT],
            textura_por_slot: [false; super::SLOT_COUNT],
            agua_por_slot: [super::WetKnobs::DEFAULT.water; super::SLOT_COUNT],
            fita: None,
            ritmo: None,
            composta: None,
            varredura: oleo_no_quadro::Varredura::default(),
            composite_pendente: false,
            lei_da_tinta: ph2d_wet_paint::sim::LeiDaTinta::Velha,
            preparo: Default::default(),
        }
    }
}

mod portas_da_agua; // relógio fixo, lei da tinta, semente, fita, largura crua — filho por LOC

mod copias; // a lane por (cópia, ponta): spray, Rough e simetria (L13) — filho de dab_route por LOC
mod dab_route; // a ROTA DO DAB (pincel -> fluido) — filho por LOC
mod esbate; // as pontas que ESBATEM ao comprido (J6t)
#[cfg(test)]
mod esbate_tests;
pub(crate) mod fita; // a FITA do worker: a água de uma macro idêntica à gravada
pub(super) mod grid_map; // a grade do fluido != a grade de PIXELS — filho por LOC
pub(super) mod largura; // a LARGURA PROMETIDA (W2): o raio do motor sai da tabela do preset
mod lote; // o papel e o sub-passo da cápsula de cada lote — filho de dab_route por LOC
mod lote_do_dab; // o que a rota apura para o lote e cada dab lê — filho de pousa_o_dab por LOC
pub(super) use lote::media_do_grao;
#[cfg(test)]
mod adormecer_so_o_que_falta_tests;
pub(crate) mod adormecida; // a tinta viva parada fora do Wet (dono 06/11)
#[cfg(test)]
mod adormecida_regua_tests;
#[cfg(test)]
mod adormecida_tests;
mod borracha_morta; // a Borracha na tinta morta imita a da viva (dono 06/11)
#[cfg(test)]
mod borracha_morta_tests;
#[cfg(test)]
mod borracha_pressao_tests;
#[cfg(test)]
mod camada_nova_paridade_tests;
#[cfg(test)]
mod camada_nova_toque_tests;
#[cfg(test)]
mod captura_da_caixa_tests;
#[cfg(test)]
mod dentro_do_wet_tests;
mod dormente; // a tinta adormecida comprimida em RAM (dono 06/11)
mod po; // o PÓ SOLTO pousado à parte do rasto (J4 pastel_carvao) — filho de dab_route por LOC
mod ponta_das_cerdas; // a ponta RECTA e DESFIADA das cerdas (J2 §7 c) — filho de dab_route por LOC
mod pousa_o_dab; // o corpo do laço da rota do dab: um dab entra na água — filho de dab_route por LOC
mod preparo; // o motor e o papel prontos antes do toque (Bug #72)
#[cfg(test)]
mod preparo_tests;
#[cfg(test)]
mod regua_do_fecho_tests;
mod rota_da_lane; // a lane de cada dab, a tinta do motor e o b/opacidade — filho de dab_route por LOC
mod sessao_do_wet;
mod session; // a sessão que pode estar com o operário do depósito (N10 F2d)
pub(in crate::tool::paint) use sessao_do_wet::SessaoDoWet;
#[cfg(test)]
mod sonda_borracha_morta;
mod tampa; // as PONTAS RECTAS do traço (J6) — filho de dab_route por LOC
#[cfg(test)]
mod tampa_tests; // what a wet SESSION is (data model) — child file (LOC cap)
#[cfg(test)]
mod undo_largura_do_pool_tests;
pub use offthread::instala_o_despertador_do_wet;
use session::{Lane, PaperKey};
pub(super) use session::{WetEngineFacts, WetSession};

mod ciclo; // o ciclo da sessão no tool: dono dos dabs, fim do traço, tique, guarda — filho por LOC

mod authored_actions; // canvas actions + session birth + facts — child file (LOC cap)
mod authoring; // o rascunho é a PRÓPRIA água (o preview de autoria) — filho por LOC
mod cauda; // a água depois do fim de um clipe (W14)
pub(super) mod faixas; // as lanes de um traço guardado (W16)

mod composite;
mod composite_oleo; // os planos do Óleo Real no composite (sombra, dente, tinta sem óleo) — filho por LOC
mod composite_otica; // a metade de cima do composite (a óptica) como função pura — filho por LOC
pub(in crate::tool::paint) mod grao_no_filme; // W3b: o dente do papel assado na tinta do Óleo Real (Relevo do papel, Transmission)
#[cfg(test)]
mod oleo_arrasto_tests; // arrastar um número do Real Oil: uma faixa por quadro, a tela final ao byte
pub(in crate::tool::paint) mod oleo_no_quadro; // a recomposição do painel corre no tick, em faixas
mod oleo_plano;
mod otica_factos; // o que a óptica tira da ferramenta, como valor que viaja (J13b P3)
#[cfg(test)]
mod otica_fora_porta; // J13b P4: a óptica pelo ESPELHO, síncrona, no fio do teste
#[cfg(test)]
mod otica_fora_tests; // J13b P4: a óptica pelo espelho == a tela de casa, ao byte
mod otica_no_tique; // J13b P6: o tique pede e aplica as imagens do compositor da óptica
#[cfg(test)]
mod otica_no_tique_tests; // J13b P6: a água pelo compositor == a tela de casa, e o fio da tela sem a óptica
#[cfg(test)]
mod otica_pen_down_tests; // J13b P8: o sujo da água herdado no pen-down vai pela dívida
mod por_quadro; // R3: a tela do Wet compõe-se uma vez por quadro, na união dos dabs dele
#[cfg(test)]
mod por_quadro_tests;
#[cfg(test)]
mod sonda_r3_facas; // R3: células e ms da composição nas facas do oráculo 4K
#[cfg(test)]
mod suave_do_pincel_tests; // cada pincel tem o seu: o preset, a miniatura e a macro antiga (dono 04/11)
#[cfg(test)]
mod suave_por_traco_tests; // o Smooth Edges pousa com a tinta: cada borda lê o seu (dono 04/11)
mod veu; // o véu do Show-wet sobre o composite — filho por LOC // o papel liso da sombra do Óleo Real: uma conta por linha, não por célula
#[cfg(test)]
pub(in crate::tool::paint) use oleo_plano::{
    curas_desligadas as oleo_curas_desligadas, desligado as oleo_plano_desligado,
};
/// O Solid e os fios na água, pela porta da MÁSCARA do motor (doc 46 item 9).
mod mascara;

mod smooth_edges; // o Smooth Edges do Watercolor sobre a mancha do Wet Paint (dono 09/10)
#[cfg(test)]
pub(in crate::tool::paint) use composite::split as composite_split;
#[cfg(test)]
pub(in crate::tool::paint) mod composite_contagem; // as células e os ms de cada composite (sondas)
mod offthread; // a sim FORA da thread do frame (o slot + o worker) — filho por LOC // the composite half (visual terms + veil) — child file (LOC cap)
mod offthread_ritmo; // o worker RITMADO dos gates (W7) — filho por LOC

#[cfg(test)]
#[path = "wetpaint/grid_ratio_tests.rs"]
mod grid_ratio_tests; // os gates de PRODUTO da razão da grade
#[cfg(test)]
#[path = "wetpaint/material_do_pincel_tests.rs"]
mod material_do_pincel_tests; // auditoria W3b C3/C4/U2/U4: o material do pincel fica na tinta e no preview
#[cfg(test)]
#[path = "wetpaint/oleo_por_traco_tests.rs"]
mod oleo_por_traco_tests; // dono 28/10: Depth, Gloss e a caixa Real Oil são do traço // W3b item 3: o painel do pincel só vale para os traços seguintes
#[cfg(test)]
#[path = "wetpaint/painel_so_no_futuro_tests.rs"]
mod painel_so_no_futuro_tests;
#[cfg(test)]
#[path = "wetpaint/relevo_tests.rs"]
mod relevo_tests;
#[path = "wetpaint/sil_memo.rs"]
mod sil_memo; // a silhueta do dab guardada para o pente do relevo (W3b) // o Emboss do Tuning morde sem papel (dono 11/10)

#[cfg(test)]
#[path = "wetpaint/flow_ratio_tests.rs"]
mod flow_ratio_tests; // os gates de PRODUTO da razão da grade de FLUXO
#[cfg(test)]
#[path = "wetpaint/grao_a_mao_tests.rs"]
mod grao_a_mao_tests;
#[cfg(test)]
#[path = "wetpaint/grao_nos_slots_tests.rs"]
mod grao_nos_slots_tests;
#[cfg(test)]
#[path = "wetpaint/hardness_por_ferramenta_tests.rs"]
mod hardness_por_ferramenta_tests;
#[cfg(test)]
#[path = "wetpaint/manchas_liner_tests.rs"]
mod manchas_liner_tests;
#[cfg(test)]
#[path = "wetpaint/offthread_espera_tests.rs"]
mod offthread_espera_tests; // B12.A1: o tique não espera o TICK_WAIT fora do mostra
#[cfg(test)]
#[path = "wetpaint/offthread_tests.rs"]
mod offthread_tests; // os gates da sim FORA da thread do frame
#[cfg(test)]
#[path = "wetpaint/raio_igual_ao_anel_tests.rs"]
mod raio_igual_ao_anel_tests;
#[cfg(test)]
#[path = "wetpaint/same_as_paper_volume_tests.rs"]
mod same_as_paper_volume_tests;
#[cfg(test)]
#[path = "wetpaint/suave_no_arrasto_tests.rs"]
mod suave_no_arrasto_tests;
#[cfg(test)]
#[path = "wetpaint/verde_no_soltar_tests.rs"]
mod verde_no_soltar_tests; // azul sobre azul não fica verde no soltar (Bug #49) // o Same as Paper e a quantidade de tinta (smoke 04/11)

// o veu do Show Wet e VIVO (report do Enio, 2026-08-07)
#[cfg(test)]
mod achatamento_tests; // o escorrimento não se corta em rectângulos (W1f)
#[cfg(test)]
mod agua_no_oleo_regua_tests; // J11: a água × o óleo seco (atrito do leito, a água parada dissolve)
#[cfg(test)]
mod agua_no_oleo_tests; // sonda J11: a água × o óleo seco (velocidade, dissolver, pixelado)
#[cfg(test)]
mod agua_sobre_oleo_tests; // smoke do dono 30/10: o molhado sobre o óleo com papel de cor
#[cfg(test)]
mod borda_do_traco_tests; // o Edge Darkening é de cada traço (Bug #69)
#[cfg(test)]
mod borracha_de_fabrica_tests; // a Relief Eraser é a Borracha que abre sem preset (dono 06/11)
#[cfg(test)]
mod borracha_defeitos_tests; // o relato do dono 06/11: rectângulos ao voltar, outros modos de uma vez
#[cfg(test)]
mod borracha_fronteira_tests; // o relato do dono 06/11: a fronteira viva × morta em degraus
mod borracha_na_base; // a Borracha também apaga a tinta antiga (Bug #70)
#[cfg(test)]
mod borracha_na_base_tests;
#[cfg(test)]
mod chao_forca_regua_tests; // J10: a aguada empoça, contorna e passa por cima com volume
#[cfg(test)]
mod chao_forca_tests; // sonda J10: o chão como força no momento
#[cfg(test)]
mod controlos_medidos_tests; // Color, Solid, Type e On empty medidos num traço de fábrica
#[cfg(test)]
mod desempenho_oleo_tests; // sonda J8: o óleo diluído é lento por correr mais tempo ou por passo?
#[cfg(test)]
mod dry_brush_tests; // o Dry brush só age com o portão seco aberto (a cobertura que cai)
#[cfg(test)]
mod escorre_contorna_tests; // sonda J8: o escorrimento contorna o relevo já pousado?
#[cfg(test)]
mod fps_j13_tests; // sonda J13: o FPS do Real Oil — o que a janela do composite refaz e o que mudou
#[cfg(test)]
mod gota_tests;
#[cfg(test)]
mod janela_do_rasto_tests; // a janela do rasto não corta o dab (smoke do dono 16/10)
#[cfg(test)]
mod lamina_tests; // a lâmina que gira e o rebordo da faca (Óleo Real W2 E4)
#[cfg(test)]
mod lei_b_tests; // plano 27 W3: o padrão do Shape multiplica DEPOIS do portão do papel
#[cfg(test)]
mod medida_undo; // o Ctrl+Z com as opções da W1g (dono 10/10)
#[cfg(test)]
mod mistura_camadas_tests; // N camadas de amarelo sobre azul seco convergem sem preto (prompt 15)
#[cfg(test)]
mod mistura_tests; // a barra do Rebelle para a mistura (W1e, F6)
#[cfg(test)]
mod mistura_wet_drip_tests; // uma mistura nunca é mais clara que as tintas (o Wet Drip, prompt 16)
#[cfg(test)]
mod oleo_diluido_tests; // E10g (dono 01/11): o óleo diluído escorre, a tinta do tubo fica
#[cfg(test)]
mod oleo_sem_real_oil_tests; // J9 §3: a tinta sem Real Oil não paga a óptica do óleo
#[cfg(test)]
mod opcoes_tests; // as opções da W1g: desligadas = o traço de hoje ao byte
#[cfg(test)]
mod pastel_seco_tests; // o Soft Pastel não se mexe depois do traço e não depende da altura (prompt 35)
#[cfg(test)]
mod queda_na_ladeira_tests; // smoke do dono 30/10: a queda na ladeira do óleo pela diluição e com o Real Oil
mod relevo_da_borracha; // o relevo segue o alfa que a Borracha do Wet deixa (B06.1)
#[cfg(test)]
mod remolha_tests; // J10 (dono 30/10): a aguada re-molha o óleo solúvel (Re-wet × Solubility)
#[cfg(test)]
mod shape_color_tests; // prompt 41: a Shape Color pinta cada célula do depósito; desligada, ao byte
#[cfg(test)]
#[path = "wetpaint/showwet_tests.rs"]
mod showwet_tests;
#[cfg(test)]
mod sobe_a_ladeira_porta_tests; // sonda J9 §2: por que porta a água entra no miolo do óleo
#[cfg(test)]
mod sonda_achatamento; // as rectas do escorrimento no S (W1f, prompt 12)
#[cfg(test)]
mod sonda_agua_oleo_4k; // B11.3/B13.5: a água a 4K com o Óleo Real, ms por quadro e o hash
#[cfg(test)]
mod sonda_agua_quieta; // B13.3: quanto da água ainda mexe depois de soltar
#[cfg(test)]
mod sonda_b32_espectral; // B32.1: K–M × espectral, prancha e custo no mesmo processo
#[cfg(test)]
mod sonda_bordas_longe; // um traço novo longe não muda um byte do antigo (fotos do dono 05/11)
#[cfg(test)]
mod sonda_camadas; // as sondas das camadas de amarelo sobre azul (prompt 15)
#[cfg(test)]
mod sonda_corte; // o corte vertical no início do traço (smoke do dono 16/10)
#[cfg(test)]
mod sonda_fios; // O3: os fios finos do Rebelle saem dos knobs que o motor já tem?
#[cfg(test)]
mod sonda_fotos_do_dono; // as duas fotos de 05/11: branco em papel de cor e Knife Stain pequeno
#[cfg(test)]
mod sonda_gota; // a varredura da lei da gota contra o Rebelle (W1e, F1+F5)
#[cfg(test)]
mod sonda_macro_tests; // as MACROS: reproduzir ao byte contando passos da sim, não segundos
#[cfg(test)]
mod sonda_mistura; // a mistura em tempo real, molhado × seco (dono 09/10)
#[cfg(test)]
mod sonda_mistura_wet_drip;
#[cfg(test)]
mod sonda_opcoes; // as opções da W1g numa rodada só
#[cfg(test)]
mod sonda_papel_seco;
#[cfg(test)]
mod sonda_pastel_tempo; // o Soft Pastel no TEMPO contra o Rebelle Dry Thin (prompt 35)
#[cfg(test)]
mod sonda_presets;
#[cfg(test)]
mod sonda_primeiro_traco; // o travamento do começo do 1.º traço (dono 05/11)
#[cfg(test)]
mod sonda_relevo_negativo; // a tinta afundada no papel de alto relevo (Bug #71)
#[cfg(test)]
mod sonda_shape_casa; // W0 do plano 27: a tinta de cada tipo do Shape e as leis candidatas do #47 // o Wet Drip com as caixas K–M e a lei de antes (prompt 16, Fases B–E) // os 4 pincéis de fábrica na bancada do Rebelle (conferência do dono 10/10) // o pincel seco nos picos do papel (dono 09/10) // a barra da meia inclinação e a tela direita ao byte (W1e, F1)
#[cfg(test)]
mod sonda_shape_color; // prompt 41: a régua, os números das leis e a prancha da Shape Color

#[cfg(test)]
mod molhado_tests; // molhado × molhado (Wet bloom): o sujo do levantar e as sondas dos retângulos e do FPS

#[cfg(test)]
mod tests; // the W1/W2 gates — child file (workspace file-LOC cap)

#[cfg(test)]
mod tests_doc22; // the doc-22 gates (tuning/tilt/tools/actions/flags)

/// O modo de Blend de uma sessão molhada que nasce com o pincel em `blend`: o modo dele, salvo os dois
/// que mexem no ALFA da camada por baixo (`EraseAlpha`/`AddAlpha`) — a água não apaga a tela: a
/// borracha dela é a ferramenta Erase, que tira tinta do fluido.
pub(in crate::tool::paint) fn modo_da_sessao(
    blend: ph2d_painter_brush::BrushBlend,
) -> ph2d_painter_brush::BrushBlend {
    if blend.lays_pigment() {
        blend
    } else {
        ph2d_painter_brush::BrushBlend::Mix
    }
}

#[cfg(test)]
#[path = "wetpaint/mascara_tests.rs"]
mod mascara_tests; // o Solid e os fios na água (doc 46 item 9)

#[cfg(test)]
#[path = "wetpaint/blend_tests.rs"]
mod blend_tests; // o Blend chega à água: o modo é da sessão (doc 46 §2-6)

#[cfg(test)]
#[path = "wetpaint/birth_tests.rs"]
mod birth_tests; // o nascimento da sessao nao pinta a tela inteira

#[cfg(test)]
#[path = "wetpaint/undo_drip_tests.rs"]
mod undo_drip_tests; // o escorrido que sobrou do Undo (smoke do Enio, 2026-07-26)

/// Porta de MEDIÇÃO para o composite (`super` é privado ao módulo `paint`, e
/// as sondas moram em `paint::tests`): a metade não-sim do tick, cronometrável
/// sem re-implementar nada.
#[cfg(test)]
pub(in crate::tool::paint) fn composite_for_measure(t: &mut PainterTool) {
    t.wetpaint_composite();
}

// O undo devolve a tinta viva de antes do passo (smoke do dono 30/10).
mod foto_em_tiles;
#[cfg(test)]
#[path = "wetpaint/prancha_b13_10_tests.rs"]
mod prancha_b13_10_tests;
#[cfg(test)]
#[path = "wetpaint/redo_relevo_tests.rs"]
mod redo_relevo_tests;
#[cfg(test)]
mod sonda_fases_ctrl_z_tests; // as fases do Ctrl+Z da tinta viva a 4K
#[cfg(test)]
#[path = "wetpaint/sonda_lote_tests.rs"]
mod sonda_lote_tests;
#[cfg(test)]
mod undo_auditoria_tests;
#[cfg(test)]
#[path = "wetpaint/undo_matriz_tests.rs"]
mod undo_matriz_tests;
#[cfg(test)]
mod undo_tinta_morre_tests;
#[cfg(test)]
mod undo_tinta_viva_tests;
mod undo_vivo;
