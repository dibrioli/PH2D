//! ⭐ Defeito do dono 15/11 (doc 06 §9): *«Play in editor não dá foco ao game»*. A sequência REAL do quadro, com as
//! peças de produção: o clique no Play (Down/Up fora da cena), o bus (`ph2d_transport` + o pedido da corrida), a posse
//! no início do quadro seguinte, o dreno da corrida (`take_runtime`) e o fim da sessão «Animating».

use ph2d_app_game::posse_do_jogo::{Cena, FocoDoJogo, Tapa, Tecla, Transporte};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::TransportCmd;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::PointerButton;
use ph2d_input::{InputState, Key};

const CAMPO: NodeId = NodeId(0xA161E);
/// A seta → no idioma do jogo (`ph2d_input`, o código Win32).
const SETA: Key = Key(0x27);

/// A metade da Timeline do `App::cena_do_jogo` (`shells/desktop/src/input_dispatch/posse_do_jogo.rs`), campo a campo.
fn tapa_da_timeline(
    timeline: &ph2d_timeline::TimelineState,
    pedidos: &[ph2d_timeline::TimelineIntent],
) -> Tapa {
    Tapa {
        animar: timeline.animating.0,
        corrida: ph2d_transport::corrida(timeline, pedidos),
        ..Tapa::default()
    }
}

struct Mesa {
    timeline: ph2d_timeline::TimelineState,
    pedidos: Vec<ph2d_timeline::TimelineIntent>,
    relogio: ph2d_core::Playhead,
    foco: FocoDoJogo,
    input: InputState,
    hero: HeroScreen,
}

impl Mesa {
    /// Cartões + aba Keys (a fábrica do dono): o dreno publica a sessão «Animating» com o relógio parado.
    fn nova() -> Self {
        let mut timeline = ph2d_timeline::TimelineState::default();
        timeline.animating = (true, false);
        let mut relogio = ph2d_core::Playhead::new(1.0 / 60.0);
        relogio.pause();
        Self {
            timeline,
            pedidos: Vec::new(),
            relogio,
            foco: FocoDoJogo::default(),
            input: InputState::new(),
            hero: HeroScreen::new(NodeId(1)),
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
                ..tapa_da_timeline(&self.timeline, &self.pedidos)
            }
            .a_vista(),
            palco: false,
        }
    }

    /// O início do quadro (`quadro_do_foco_do_jogo`), depois a simulação drena a corrida; a corrida no ar acaba a
    /// sessão (`fase_timeline_drain`: `vista_keys = false`).
    fn quadro(&mut self) -> bool {
        let cena = self.cena();
        let tem = self.foco.quadro(&mut self.input, cena);
        ph2d_timeline::take_runtime(&mut self.timeline, &mut self.pedidos, &mut self.relogio);
        if self.timeline.runtime {
            self.timeline.animating = (false, false);
        }
        tem
    }

    /// O botão Play de cima: o Down e o Up caem FORA da cena (`rato_do_jogo`), o widget levanta o Play no bus.
    fn clica_no_play(&mut self) {
        for desceu in [true, false] {
            let cena = self.cena();
            let so = self
                .foco
                .clique(&mut self.input, cena, PointerButton::Primary, desceu, false);
            assert!(!so, "o clique no Play nunca é do jogo");
        }
        self.bus(TransportCmd::Play);
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
        self.bus(TransportCmd::Play);
    }

    /// `fase_bus_topbar`, o braço do `Transport`.
    fn bus(&mut self, cmd: TransportCmd) {
        ph2d_transport::toma_o_teclado(cmd, Some(&mut self.hero));
        ph2d_transport::apply(cmd, &mut self.relogio);
        if let Some(i) = ph2d_transport::runtime_intent(cmd) {
            self.pedidos.push(i);
        }
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
        let chave = Some(SETA);
        self.foco
            .tecla(&mut self.input, cena, seta, chave, false, &"→")
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
    m.bus(TransportCmd::Reset);
    m.quadro();
    m.timeline.animating = (true, false);
    assert!(
        !tapa_da_timeline(&m.timeline, &m.pedidos).a_vista(),
        "fora da corrida a sessão volta a tapar"
    );
}

/// A mesa acima só vale se a shell compõe com as MESMAS peças: a posse pergunta à `corrida` e o braço do `Transport`
/// larga o campo antes de aplicar o comando.
#[test]
fn a_shell_compoe_a_posse_com_as_pecas_da_mesa() {
    let ler = |f: &str| {
        std::fs::read_to_string(format!("{}/../desktop/src/{f}", env!("CARGO_MANIFEST_DIR")))
            .unwrap()
    };
    let posse = ler("input_dispatch/posse_do_jogo.rs");
    assert!(posse.contains("animar: self.timeline.animating.0,"));
    assert!(
        posse.contains("corrida: ph2d_transport::corrida(&self.timeline, &self.timeline_intents),")
    );
    let bus = ler("render_loop/fase_bus_chrome.rs");
    let larga = bus.find("ph2d_transport::toma_o_teclado(cmd, hero_screen.as_mut());");
    let aplica = bus.find("ph2d_transport::apply(cmd, &mut self.playhead);");
    assert!(
        larga.is_some() && larga < aplica,
        "o Play larga o campo no braço do Transport"
    );
}
