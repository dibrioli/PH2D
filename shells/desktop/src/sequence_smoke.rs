//! ⭐⭐⭐ **Smoke da CUTSCENE** (TOP-20 #19, W3). `PH2D_SEQUENCE_SMOKE=1`.
//!
//! # A PORTA, e o CONTROLO ao lado dela
//!
//! Duas portas idênticas, lado a lado, a ouvir **o mesmo sinal** (`tocar`, que um relógio publica
//! de quatro em quatro segundos):
//!
//! | | o que tem | o que FAZ |
//! |---|---|---|
//! | **esquerda — «Door»** | um **Sequence Player** a apontar à cutscene `Abrir` | sobe, ao ritmo do relógio DELA |
//! | **direita — «Door (no cutscene)»** | o MESMO relógio e a MESMA tabela de acções | fica parada: o relógio corre e não há sequência a correr com ele |
//!
//! ⭐⭐⭐ **É o CONTROLO que torna a wave legível.** Sem ele, a porta da esquerda parece uma porta
//! animada — com ele, vê-se *o que o componente compra*: **a timeline vira uma cutscene que um
//! objecto toca no relógio dele**, e não uma animação presa ao relógio da cena.
//!
//! # ⚠️ A cena tem DUAS cutscenes, e a segunda é o ponto do PAINEL
//!
//! `Abrir` (a porta sobe) e `Abanar` (a porta treme no sítio). Trocar uma pela outra no selector da
//! secção *Sequence* muda o que a porta faz **sem tocar na timeline** — que é a metade desta wave
//! que o artista alcança com o dedo. ⛔ *Uma só cutscene deixaria o selector a mostrar uma lista
//! com um item, e um selector sem escolha não se lê como selector.*
//!
//! # ⛔ A arrumação da timeline fica VAZIA, e isso é load-bearing
//!
//! Nenhuma das duas cutscenes é colocada na faixa da cena. A vista *Arrange* com a pilha vazia
//! **não toca nada** (a lei do `apply_scene`, ordem do dono em 2026-07-27), logo tudo o que se vê a
//! mexer veio do `SequencePlayer` — e não do playhead da cena a passar por cima de um clip. *Com
//! uma instância na faixa, a cena ensinaria o contrário do que diz.*
//!
//! # ⛔⛔ E a cena NÃO abre a timeline — a ausência é load-bearing, e a foto foi quem a mediu
//!
//! A vista **Keys** *sola* a clip que o animador edita e congela o relógio da cena, logo a fase das
//! cutscenes **não corre lá** (a condição `container.is_none() && !solo` do `timeline_bridge`). E
//! escolher um objecto **pede a aba Keys** (`selection_jumps_to_keys`) ⇒ uma cena que abrisse a
//! timeline *e* escolhesse a porta mostraria duas portas paradas, com todos os dados certos.
//!
//! ⚠️ **Com o painel fechado o `keys_mode` é publicado `false`** (a lei do `paint` dele), e é isso
//! que faz a cutscene correr. ⭐ Quem abrir a timeline para espreitar tem de escolher **Arrange** —
//! ali ela continua a correr; na **Keys**, não. *Esta é a fronteira da wave, e ela é a mesma que o
//! animador já conhece: a aba Keys é onde se AUTORA, não onde a cena corre.*
//!
//! # O que tem de acontecer
//!
//! A porta da **esquerda** sobe, volta ao chão e sobe outra vez, de quatro em quatro segundos. A da
//! **direita** nunca se mexe. ⚠️ Se a linha `[sequence-smoke]` não aparecer, **PARE**: a cena não
//! montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{
    Name, SequencePlayer, SignalAction, SignalActions, SignalTarget, SignalVerb, Timer, Timers,
    Transform,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::{PropKind, StackHost, StripSource};

/// O nome da cutscene com que a porta nasce — e a que o selector mostra escolhida.
const ABRIR: &str = "Abrir";
/// A segunda, a que o artista escolhe para ver a troca.
const ABANAR: &str = "Abanar";
/// O `x` da porta que tem o componente.
const X_PORTA: f32 = -2.6;
/// O `x` do CONTROLO.
const X_CONTROLO: f32 = 2.6;
/// Quantos segundos a cutscene dura — e o relógio da porta tem EXACTAMENTE isto, senão o painel
/// avisaria (com razão) que ela nunca chega ao fim.
const DURACAO_S: f64 = 2.0;
/// Quantos quadros a auto-conferência amostra. ⚠️ **Mais que um ciclo inteiro do botão**
/// (`PERIODO_S` a 60 fps = 240), senão ela podia medir só a metade em que a porta está pousada.
const AMOSTRAS: u32 = 330;
/// De quanto em quanto tempo o relógio-botão publica `tocar`.
const PERIODO_S: f64 = 4.0;

/// Uma key na clip ACTIVA, para o objecto `bits`.
fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, prop: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// ⭐ **Uma cutscene: um clip com as keys já escritas, dentro de um container com o nome dela.**
///
/// ⚠️ **O `set_active` volta ao `0` no fim** — o `upsert_key` escreve na clip ACTIVA, e deixá-la
/// noutra faria a cena seguinte (ou o artista) keyar dentro de uma cutscene sem saber.
fn cutscene(
    doc: &mut ph2d_timeline::TimelineDoc,
    nome: &str,
    clip: usize,
    escreve: impl FnOnce(&mut ph2d_timeline::TimelineDoc),
) {
    doc.set_active(clip);
    escreve(doc);
    doc.set_active(0);
    let c = doc.add_container(nome.to_string());
    let host = StackHost::Container(c);
    let lane = doc
        .add_lane_in(host, "Body".to_string())
        .expect("a lane da cutscene");
    let src = StripSource::Clip(u16::try_from(clip).expect("cabe"));
    doc.add_strip_to(host, lane, src, 0.0, DURACAO_S)
        .expect("a peça dentro da cutscene");
}

/// Uma porta: o corpo, o relógio e a linha que o arranca ao sinal. `cutscene` vazio = o CONTROLO.
fn porta(
    world: &mut ph2d_ecs::World,
    nome: &str,
    x: f32,
    tint: [f32; 4],
    cutscene: Option<&str>,
) -> u64 {
    let e = world
        .spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 2.2], tint),
            Name::new(nome),
            // ⚠️ **`autostart: false`**: quem o arranca é o sinal. Com `true` a porta subiria
            // sozinha no quadro 1 e a cena deixaria de mostrar o que um sinal compra.
            Timers(vec![Timer {
                name: "cutscene".to_string(),
                duration_us: (DURACAO_S * 1_000_000.0) as u64,
                repeat: false,
                autostart: false,
                signal: String::new(),
            }]),
            // ⚠️ `target` VAZIO = **este objecto**, que é o caso comum e o que faz uma cópia de
            // prefab funcionar sem re-fiar nada.
            SignalActions(vec![SignalAction {
                on: "tocar".to_string(),
                target: String::new(),
                verb: SignalVerb::StartTimer,
                arg: String::new(),
                target_by: SignalTarget::Named,
                from: ph2d_ecs::SignalFrom::Anyone,
            }]),
        ))
        .id();
    if let Some(nome) = cutscene {
        world.entity_mut(e).insert(SequencePlayer {
            container: nome.to_string(),
        });
    }
    e.to_bits()
}

impl crate::App {
    /// No prólogo do quadro. No-op sem a env.
    pub(crate) fn sequence_smoke(&mut self) {
        if std::env::var_os("PH2D_SEQUENCE_SMOKE").is_none() {
            return;
        }
        if self.components.smokes.sequence {
            self.components.smokes.sequence_raise =
                self.levanta_o_inspector(self.components.smokes.sequence_raise);
            self.sequence_smoke_confere();
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no quadro seguinte
        }
        self.components.smokes.sequence = true;
        self.sequence_smoke_monta();
    }

    /// Monta as duas portas, o relógio-botão e as duas cutscenes.
    fn sequence_smoke_monta(&mut self) {
        let (porta_bits, controlo_bits) = {
            let gfx = self.gfx.as_mut().expect("gfx");
            let world = gfx.sim.world_mut();
            // ⚠️ **O fundo PRIMEIRO** — a ordem de nascimento é a ordem de desenho
            // (`assign_missing_root_order`), e um chão spawnado por último tapa a cena.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, -1.6)),
                Sprite::atlas(WHITE_TILE_KEY, [14.0, 0.5], [0.16, 0.18, 0.22, 1.0]),
                Name::new("Ground"),
            ));
            // O relógio-botão: publica `tocar` de PERIODO_S em PERIODO_S.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, 2.6)),
                Sprite::atlas(WHITE_TILE_KEY, [1.0, 0.5], [0.92, 0.76, 0.26, 1.0]),
                Name::new("Button"),
                Timers(vec![Timer {
                    name: "botao".to_string(),
                    duration_us: (PERIODO_S * 1_000_000.0) as u64,
                    repeat: true,
                    autostart: true,
                    signal: "tocar".to_string(),
                }]),
            ));
            let p = porta(world, "Door", X_PORTA, [0.30, 0.55, 0.85, 1.0], Some(ABRIR));
            let c = porta(
                world,
                "Door (no cutscene)",
                X_CONTROLO,
                [0.45, 0.47, 0.52, 1.0],
                None,
            );
            (p, c)
        };

        let doc = &mut self.timeline.doc;
        doc.rename_clip(0, ABRIR.to_string());
        let abanar = doc.add_clip(ABANAR.to_string());
        // A cutscene `Abrir`: a porta sobe e volta — só `TranslationY`, logo o `x` autorado fica.
        cutscene(doc, ABRIR, 0, |d| {
            key(d, porta_bits, PropKind::TranslationY, 0.0, 0.0);
            key(d, porta_bits, PropKind::TranslationY, 1.0, 2.2);
            key(d, porta_bits, PropKind::TranslationY, 2.0, 0.0);
        });
        // A `Abanar`: só `TranslationX`, logo o `y` fica. ⚠️ **Canais DISJUNTOS de propósito** —
        // é o que torna a troca no selector legível: uma sobe, a outra treme.
        cutscene(doc, ABANAR, abanar, |d| {
            key(d, porta_bits, PropKind::TranslationX, 0.0, X_PORTA);
            key(d, porta_bits, PropKind::TranslationX, 0.5, X_PORTA + 0.5);
            key(d, porta_bits, PropKind::TranslationX, 1.5, X_PORTA - 0.5);
            key(d, porta_bits, PropKind::TranslationX, 2.0, X_PORTA);
        });

        // A porta ESCOLHIDA, para a secção *Sequence* estar à vista no primeiro quadro.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            // ⛔⛔ **O INSPECTOR É TRAZIDO À VISTA, e foi a FOTO que o disse:** naquele encaixe
            // estava o painel do Sculpt 3D por cima, e a instrução nomeava uma superfície que o
            // dono não tinha à frente. ⚠️ **A arrumação vive FORA do repositório**
            // (`~/.ph2d/layout.txt`), logo isto não é defensivo — é a única forma de a cena não
            // depender do que ficou aberto ontem.
            hero.panel_visibility.insert("inspector", true);
            // ⭐ A timeline FICA à vista: ela é a régua do tempo, e uma cena de cutscene sem ela
            // devolve *«que relógio?»* (a lição da cena 67 da física).
            hero.panel_visibility.insert("timeline", true);
            hero.gizmo.selection = Some(porta_bits);
            hero.gizmo.extra_selection.clear();
        }
        // ⛔⛔⛔ **E A ABA TEM DE SER *ARRANGE*, senão a cena ensina o contrário do que diz.** A
        // *Keys* é a de OMISSÃO (`Tab::Keys` é `#[default]`) **e** é a que escolher um objecto
        // PEDE (`selection_jumps_to_keys`) — e ali o `keys_mode` é `true`, logo a fase das
        // cutscenes não corre: duas portas paradas com todos os dados certos. ⚠️ **A foto de
        // 2026-09-17 não bastou para o ver** (uma imagem mostra um instante); quem o disse foi a
        // auto-conferência, com `MOVEU-SE 0.000`.
        ph2d_panel_timeline::state::request_arrange_tab();
        self.components.smokes.sequence_raise = 3;
        // ⭐ A auto-conferência arranca aqui e sai daqui a `AMOSTRAS` quadros — ver
        // [`crate::app_state::SequenceShell`].
        self.components.sequence = crate::app_state::SequenceShell {
            resta: AMOSTRAS,
            porta: porta_bits,
            controlo: controlo_bits,
            faixa_porta: (f32::MAX, f32::MIN),
            faixa_controlo: (f32::MAX, f32::MIN),
        };
        // ⚠️ **O relógio TEM de andar**: o `Timer` corre no passo fixo, e é ele que dá o tempo à
        // cutscene. Com o transporte parado o artista vê duas portas imóveis e lê *«não faz
        // nada»* — o veredito errado sobre um componente que funciona.
        self.playhead.rewind();
        self.playhead.play();
        self.sequence_smoke_anuncia();
    }

    /// A linha que o doc manda procurar — uma cena que monta em silêncio é uma cena que o smoke
    /// julga errado.
    fn sequence_smoke_anuncia(&self) {
        let nomes: Vec<&str> = self
            .timeline
            .doc
            .containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        eprintln!(
            "[sequence-smoke] 2 portas · cutscenes {nomes:?} · arrumação da cena: {} faixas (tem \
             de ser 0) · a conferir durante {AMOSTRAS} quadros…",
            self.timeline.doc.stack().len()
        );
    }

    /// ⭐⭐⭐ **A cena confere-se a si mesma: a porta MOVEU-SE e o controlo NÃO.**
    ///
    /// ⚠️⚠️ **A foto não decide esta pergunta** — ela mostra um instante, e *«mexeu-se»* é uma
    /// propriedade de um INTERVALO. É a mesma lei que o HUD pagou (lá, *«o dedo alcança o
    /// botão?»*): o que um gate não consegue medir e uma foto não mostra, a cena mede sobre si
    /// mesma e IMPRIME.
    ///
    /// ⭐ **E o CONTROLO é metade da conferência.** Sem ele, uma porta que se mexesse por qualquer
    /// outra razão (a cena a tocar um clip, um passe a escrever) leria como aprovação.
    fn sequence_smoke_confere(&mut self) {
        if self.components.sequence.resta == 0 {
            return;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let y = |bits: u64| {
            gfx.sim
                .world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .map(|t| t.translation.y)
        };
        let (porta, controlo) = (
            y(self.components.sequence.porta),
            y(self.components.sequence.controlo),
        );
        let s = &mut self.components.sequence;
        if let Some(v) = porta {
            s.faixa_porta = (s.faixa_porta.0.min(v), s.faixa_porta.1.max(v));
        }
        if let Some(v) = controlo {
            s.faixa_controlo = (s.faixa_controlo.0.min(v), s.faixa_controlo.1.max(v));
        }
        s.resta -= 1;
        if s.resta > 0 {
            return;
        }
        let dp = s.faixa_porta.1 - s.faixa_porta.0;
        let dc = s.faixa_controlo.1 - s.faixa_controlo.0;
        // ⚠️ **O diagnóstico separa os TRÊS elos** que podem estar partidos, e sai só com a env:
        // o relógio (o sinal chegou e arrancou-o?), a vista (a aba deixa correr?) e o transporte.
        // Sem isto, um `NAO` manda procurar em três camadas de uma vez.
        if std::env::var_os("PH2D_SEQUENCE_LOG").is_some() {
            let rt = self
                .gfx
                .as_ref()
                .and_then(|g| {
                    g.sim
                        .world()
                        .get::<ph2d_ecs::timer::TimerRuntime>(ph2d_ecs::Entity::from_bits(s.porta))
                })
                .and_then(|r| r.0.first().copied());
            eprintln!(
                "[sequence-smoke] diag · relógio da porta: {rt:?} · keys_mode: {} · a tocar: {}",
                self.timeline.keys_mode,
                self.playhead.is_playing()
            );
        }
        eprintln!(
            "[sequence-smoke] a porta com o componente MOVEU-SE {dp:.3} (tem de ser ~2,2) · o CONTROLO mexeu-se {dc:.3} (tem de ser 0,000) · veredito: {}",
            if dp > 1.0 && dc < 1e-3 { "SIM" } else { "NAO" }
        );
    }
}
