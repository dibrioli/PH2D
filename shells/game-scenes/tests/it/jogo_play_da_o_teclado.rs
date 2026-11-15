//! ⭐ Defeito do dono 15/11 (doc 06 §9): *«Play in editor não dá foco ao game»*. A sequência do quadro com as peças de
//! produção — a posse (`FocoDoJogo::quadro`), o dreno da corrida (`take_runtime`), o fim da sessão «Animating» e o bus
//! (`ph2d_transport`) — corrida na ORDEM que a shell compõe: [`ordem_do_quadro`] lê-a de `run_render_frame` e das fases
//! (auditoria 07 C9: uma mesa com a ordem escrita à mão não via a ordem de que o defeito depende).

use ph2d_app_game::posse_do_jogo::{Cena, FocoDoJogo, Tapa, Tecla, Transporte};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::TransportCmd;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::PointerButton;
use ph2d_input::{InputState, Key};

const CAMPO: NodeId = NodeId(0xA161E);
/// A seta → no idioma do jogo (`ph2d_input`, o código Win32).
const SETA: Key = Key(0x27);

/// Os passos do quadro que mexem na posse do teclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Passo {
    /// `quadro_do_foco_do_jogo`: a posse vê a subida do relógio.
    Posse,
    /// `take_runtime` (`fase_frame_simulation`): o pedido da corrida liga-a.
    Corrida,
    /// `fase_timeline_drain`: a corrida ligada acaba a sessão «Animating».
    Sessao,
    /// `fase_hero_frame` → `fase_bus_drain` → `fase_bus_topbar`: o Play do clique ou do F5.
    Bus,
}

fn ler(f: &str) -> String {
    std::fs::read_to_string(format!("{}/../desktop/src/{f}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{f}: {e}"))
}

fn em(fonte: &str, marca: &str, onde: &str) -> usize {
    fonte
        .find(marca)
        .unwrap_or_else(|| panic!("{onde} já não tem `{marca}`: reconte a ordem do quadro"))
}

/// ⭐ A ordem REAL do quadro, lida da shell: as fases pela ordem do `run_render_frame`, e dentro da simulação o
/// dreno da corrida e o da sessão pela ordem em que lá estão.
fn ordem_do_quadro() -> Vec<Passo> {
    let mod_rs = ler("render_loop/mod.rs");
    let quadro = &mod_rs[em(&mod_rs, "fn run_render_frame(", "render_loop/mod.rs")..];
    let open = ler("render_loop/fase_frame_open.rs");
    let ini = em(&open, "fn fase_frame_simulation(", "fase_frame_open.rs");
    let fim = em(&open, "fn fase_frame_canvas(", "fase_frame_open.rs");
    let sim = &open[ini..fim];
    let mut dentro = [
        (
            em(sim, "ph2d_timeline::take_runtime(", "fase_frame_simulation"),
            Passo::Corrida,
        ),
        (
            em(sim, "self.fase_timeline_drain(", "fase_frame_simulation"),
            Passo::Sessao,
        ),
    ];
    dentro.sort_by_key(|p| p.0);
    // O braço do Transport mora no hero frame: hero → bus_drain → bus_topbar → larga o campo, aplica.
    em(
        &ler("render_loop/fase_hero_frame.rs"),
        "self.fase_bus_drain()",
        "fase_hero_frame",
    );
    em(
        &ler("render_loop/fase_bus_drain.rs"),
        "self.fase_bus_topbar(a)",
        "fase_bus_drain",
    );
    let chrome = ler("render_loop/fase_bus_chrome.rs");
    let topbar = &chrome[em(&chrome, "fn fase_bus_topbar(", "fase_bus_chrome.rs")..];
    assert!(
        em(topbar, "ph2d_transport::toma_o_teclado(cmd", "fase_bus_topbar")
            < em(topbar, "ph2d_transport::apply(cmd", "fase_bus_topbar"),
        "o Play larga o campo antes de aplicar o comando"
    );
    let mut fases = vec![
        (
            em(quadro, "self.quadro_do_foco_do_jogo();", "run_render_frame"),
            vec![Passo::Posse],
        ),
        (
            em(quadro, "self.fase_frame_simulation(", "run_render_frame"),
            dentro.iter().map(|p| p.1).collect(),
        ),
        (
            em(quadro, ".fase_hero_frame(", "run_render_frame"),
            vec![Passo::Bus],
        ),
    ];
    fases.sort_by_key(|f| f.0);
    fases.into_iter().flat_map(|f| f.1).collect()
}

/// A metade da Timeline do `App::cena_do_jogo` (`shells/desktop/src/input_dispatch/posse_do_jogo.rs`); `lei_antiga`
/// = a de antes da cura (sem a `corrida`), o controlo.
fn tapa_da_timeline(
    timeline: &ph2d_timeline::TimelineState,
    pedidos: &[ph2d_timeline::TimelineIntent],
    lei_antiga: bool,
) -> Tapa {
    Tapa {
        animar: timeline.animating.0,
        corrida: !lei_antiga && ph2d_transport::corrida(timeline, pedidos),
        ..Tapa::default()
    }
}

struct Mesa {
    ordem: Vec<Passo>,
    lei_antiga: bool,
    timeline: ph2d_timeline::TimelineState,
    pedidos: Vec<ph2d_timeline::TimelineIntent>,
    relogio: ph2d_core::Playhead,
    foco: FocoDoJogo,
    input: InputState,
    hero: HeroScreen,
    /// O que o widget levantou no bus desde o último quadro (o clique e o F5 chegam ENTRE quadros).
    no_bus: Vec<TransportCmd>,
    /// No passo da posse do último quadro: (o relógio anda, a sessão «Animating» aberta, o jogo tem o teclado).
    na_posse: (bool, bool, bool),
}

impl Mesa {
    /// Cartões + aba Keys (a fábrica do dono): o dreno publica a sessão «Animating» com o relógio parado.
    fn nova() -> Self {
        let mut timeline = ph2d_timeline::TimelineState::default();
        timeline.animating = (true, false);
        let mut relogio = ph2d_core::Playhead::new(1.0 / 60.0);
        relogio.pause();
        Self {
            ordem: ordem_do_quadro(),
            lei_antiga: false,
            timeline,
            pedidos: Vec::new(),
            relogio,
            foco: FocoDoJogo::default(),
            input: InputState::new(),
            hero: HeroScreen::new(NodeId(1)),
            no_bus: Vec::new(),
            na_posse: (false, false, false),
        }
    }

    /// O Angle do raio no Inspector com o teclado.
    fn com_o_campo_focado(mut self) -> Self {
        let v = 30.0;
        self.hero.store.register(
            CAMPO,
            ph2d_editor_core::InteractiveState::NumberInput {
                state: ph2d_editor_core::widget::TextInputState::Focused,
                value: v,
                buffer: "30".into(),
                caret: 2,
                last_committed: v,
                selection_anchor: None,
            },
        );
        self.hero.store.set_focus(Some(CAMPO));
        assert!(self.hero.text_entry_focused());
        self
    }

    /// `App::cena_do_jogo`: o que tapa a cena neste instante.
    fn cena(&self) -> Cena {
        Cena {
            a_correr: self.relogio.is_playing(),
            a_vista: Tapa {
                texto: self.hero.text_entry_focused(),
                ..tapa_da_timeline(&self.timeline, &self.pedidos, self.lei_antiga)
            }
            .a_vista(),
            palco: false,
        }
    }

    /// Um quadro, passo a passo pela ordem da shell.
    fn quadro(&mut self) {
        for passo in self.ordem.clone() {
            match passo {
                Passo::Posse => {
                    let cena = self.cena();
                    let tem = self.foco.quadro(&mut self.input, cena);
                    self.na_posse = (self.relogio.is_playing(), self.timeline.animating.0, tem);
                }
                Passo::Corrida => ph2d_timeline::take_runtime(
                    &mut self.timeline,
                    &mut self.pedidos,
                    &mut self.relogio,
                ),
                // `fase_timeline_drain`: com a corrida ligada a vista Keys sai (`vista_keys = false`).
                Passo::Sessao if self.timeline.runtime => self.timeline.animating = (false, false),
                Passo::Sessao => {}
                Passo::Bus => {
                    for cmd in std::mem::take(&mut self.no_bus) {
                        ph2d_transport::toma_o_teclado(cmd, Some(&mut self.hero));
                        ph2d_transport::apply(cmd, &mut self.relogio);
                        self.pedidos.extend(ph2d_transport::runtime_intent(cmd));
                    }
                }
            }
        }
    }

    /// O botão Play de cima: o Down e o Up caem FORA da cena (`rato_do_jogo`), o widget levanta o Play no bus, e o
    /// quadro corre-o.
    fn clica_no_play(&mut self) {
        for desceu in [true, false] {
            let cena = self.cena();
            let so = self
                .foco
                .clique(&mut self.input, cena, PointerButton::Primary, desceu, false);
            assert!(!so, "o clique no Play nunca é do jogo");
        }
        self.no_bus.push(TransportCmd::Play);
        self.quadro();
    }

    /// O F5 (`tecla_do_jogo`): fora da corrida é a porta do Play de cima.
    fn carrega_f5(&mut self) {
        let t = Tecla {
            desceu: true,
            acorde: false,
            shift: false,
            f5: true,
            repete: false,
        };
        let cena = self.cena();
        let d = self
            .foco
            .tecla(&mut self.input, cena, t, None, false, &"F5");
        assert_eq!(d.transporte, Some(Transporte::Alterna));
        self.no_bus.push(TransportCmd::Play);
        self.quadro();
    }

    /// Depois do Play, sem clicar na cena: as setas movem o personagem?
    fn a_seta_chega_ao_jogo(&mut self) -> bool {
        let cena = self.cena();
        let seta = Tecla {
            desceu: true,
            acorde: false,
            shift: false,
            f5: false,
            repete: false,
        };
        self.foco
            .tecla(&mut self.input, cena, seta, Some(SETA), false, &"→")
            .jogo
    }
}

fn depois_do_play(mut m: Mesa, play: fn(&mut Mesa), caso: &str) {
    m.quadro();
    play(&mut m);
    for q in 1..=3 {
        m.quadro();
        assert!(
            m.foco.tem_o_teclado(m.cena()),
            "{caso}: {q}.º quadro depois do Play, sem clique na cena, o jogo não tem o teclado"
        );
    }
    assert!(m.a_seta_chega_ao_jogo(), "{caso}: a seta não chega ao jogo");
}

/// ⭐ Auditoria 07 C9: na ordem REAL do quadro, o Play no quadro N, a subida do relógio vista pela posse no N+1 com a
/// sessão «Animating» AINDA aberta (o fenómeno: a corrida só a fecha mais adiante no mesmo quadro), e o teclado vai ao
/// jogo nesse mesmo passo. Controlo: a lei de antes da cura, na mesma ordem, perde o teclado — se um dia a shell
/// reordenar o quadro e a lei antiga passar, a mesa deixou de ter o fenómeno e o gate di-lo.
#[test]
fn a_posse_do_teclado_segue_a_ordem_do_quadro() {
    for lei_antiga in [false, true] {
        for (play, caso) in [
            (Mesa::clica_no_play as fn(&mut Mesa), "botão Play"),
            (Mesa::carrega_f5, "F5"),
        ] {
            let mut m = Mesa::nova();
            m.lei_antiga = lei_antiga;
            m.quadro();
            play(&mut m);
            assert!(
                !m.na_posse.0,
                "{caso}: no quadro do Play a posse olha antes do bus"
            );
            m.quadro();
            let (sobe, sessao, tem) = m.na_posse;
            assert!(
                sobe && sessao,
                "{caso}: a posse do quadro seguinte tem de ver o relógio a subir com a sessão «Animating» ainda \
                 aberta (ordem lida: {:?}) — sem isso a mesa não tem o defeito do dono",
                m.ordem
            );
            if lei_antiga {
                assert!(!tem, "{caso}: controlo — a lei antiga perde o teclado nesta ordem");
            } else {
                assert!(tem, "{caso}: a sessão de antes do Play tapou a corrida");
                assert!(m.a_seta_chega_ao_jogo(), "{caso}: a seta não chega ao jogo");
            }
        }
    }
}

#[test]
fn o_play_no_editor_da_o_teclado_ao_jogo_sem_clicar_na_cena() {
    depois_do_play(Mesa::nova(), Mesa::clica_no_play, "botão Play");
    depois_do_play(Mesa::nova(), Mesa::carrega_f5, "F5");
}

#[test]
fn o_play_com_um_campo_do_inspector_focado_da_o_teclado_ao_jogo() {
    depois_do_play(
        Mesa::nova().com_o_campo_focado(),
        Mesa::clica_no_play,
        "botão Play",
    );
    let mut m = Mesa::nova().com_o_campo_focado();
    m.quadro();
    m.carrega_f5();
    assert!(!m.hero.text_entry_focused(), "o Play larga o campo");
    depois_do_play(Mesa::nova().com_o_campo_focado(), Mesa::carrega_f5, "F5");
}

/// O que NÃO pode mudar: um clique fora tira o foco e a sessão volta a tapar quando a corrida sai.
#[test]
fn depois_do_play_um_clique_fora_continua_a_tirar_o_teclado() {
    let mut m = Mesa::nova();
    m.quadro();
    m.clica_no_play();
    m.quadro();
    let cena = m.cena();
    m.foco
        .clique(&mut m.input, cena, PointerButton::Primary, true, false);
    m.quadro();
    assert!(!m.foco.tem_o_teclado(m.cena()), "o clique fora tira-o");
    assert!(!m.a_seta_chega_ao_jogo());
    m.no_bus.push(TransportCmd::Reset);
    m.quadro();
    m.quadro();
    m.timeline.animating = (true, false);
    assert!(
        !tapa_da_timeline(&m.timeline, &m.pedidos, false).a_vista(),
        "fora da corrida a sessão volta a tapar"
    );
}

/// A composição da posse na shell: a `Tapa` lê a `corrida` (a mesa acima corre a ordem, não a lista de campos).
#[test]
fn a_shell_compoe_a_posse_com_as_pecas_da_mesa() {
    let posse = ler("input_dispatch/posse_do_jogo.rs");
    assert!(posse.contains("animar: self.timeline.animating.0,"));
    assert!(
        posse.contains("corrida: ph2d_transport::corrida(&self.timeline, &self.timeline_intents),")
    );
}
