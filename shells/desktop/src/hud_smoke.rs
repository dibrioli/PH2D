//! ⭐⭐⭐ **A cena do HUD** (TOP-20 #20) — `PH2D_HUD_SMOKE=1`.
//!
//! O que ela mostra: **o mundo rola e o HUD não se move.** É a frase inteira da feature, e é por
//! isso que a cena COMPÕE com a da câmera de jogo (TOP-20 #7) em vez de montar um mundo próprio —
//! sem um mundo que se desloque debaixo dele, um HUD é indistinguível de qualquer desenho parado.
//!
//! | o que está na tela | o que prova |
//! |---|---|
//! | os postes e a cerca, que passam ao lado | a câmera segue o herói (a cena do #7) |
//! | **Pontos: N** no canto de cima | o rótulo lê um [`ph2d_ecs::Counter`] que o relógio soma |
//! | **N.N s** no canto oposto | um rótulo de [`ph2d_ecs::LabelSource::TimerLeft`], que desce sozinho |
//! | o botão **+10** | o clique publica um sinal, e a tabela do #5 soma ao MESMO contador |
//!
//! # ⚠️ TRÊS quadros, e a razão do terceiro é uma armadilha desta casa
//!
//! O `sync` do render loop é que dá **entidade** a um `VecPath` recém-nascido, e sem entidade não
//! há onde pendurar um `UiLabel`. ⇒ o mundo e as formas nascem num quadro, os componentes no
//! seguinte. É a mesma razão que o `text_fx_smoke` e o `text_wrap_smoke` já pagaram.
//!
//! # ⚠️⚠️ A vista da câmera de jogo é a da JANELA, e o editor pinta num sub-rectângulo
//!
//! O `fase_game_camera` calcula o enquadramento com o aspecto da **superfície inteira**
//! (`camera_2d::aspect_of(surface.size())`), e o editor desenha o mundo na área que sobra **entre
//! os painéis**. ⇒ no EDITOR, um HUD colado às bordas da caixa de referência cai atrás da régua e
//! dos painéis; numa janela de jogo (sem painéis) ele assenta exactamente onde foi autorado.
//!
//! ⇒ **esta cena põe as quatro peças bem dentro da caixa** (`±9` de `16`, `±5` de `9`), para o que
//! o dono vê ser o que a lei faz. *Uma cena cujo canto está atrás de um painel ensina que o
//! ancoramento está partido.*
//!
//! # ⚠️ As três peças vivem EM BAIXO, e a razão é uma medição
//!
//! Os avisos de sinal (*toasts*) empilham-se no **topo** do canvas, e o relógio desta cena publica
//! um a cada dois segundos — com o placar no canto de cima, a foto mostrou-o **tapado** metade do
//! tempo. *Uma cena em que o número que se vai ler está escondido não prova nada.*
//!
//! # ⛔ O que esta cena NÃO faz
//!
//! Ela **não** arma a ferramenta vectorial: um clique aqui é do jogo (o relógio anda), e a cena
//! tem de abrir no estado em que o botão do HUD responde. *Uma cena que abre com a ferramenta
//! errada ensina que o botão está partido.*

use ph2d_ecs::{
    ChildOf, Counter, CounterRuntime, Entity, Fit, LabelSource, Name, SignalAction, SignalActions,
    SignalTarget, SignalVerb, Timer, TimerRuntime, TimerState, Timers, Transform, UiButton,
    UiCanvas, UiLabel,
};
use ph2d_vec_scene::VecPathId;

use ph2d_app_vec::text_edit::VecTextEdit;

/// A caixa em que o HUD é desenhado, em unidades de mundo. ⚠️ **`16:9` e centrada**, como o
/// `UiCanvas::default()`: é a caixa que o `Fit::Keep` mapeia sobre a vista da câmera.
const REF_W: f32 = 32.0; // LITERAL-PX-OK: metros
const REF_H: f32 = 18.0; // LITERAL-PX-OK: metros

/// O tamanho do glyph, em unidades da caixa de referência.
const TXT: f64 = 1.1; // LITERAL-PX-OK: metros

/// Quanto o botão soma. ⚠️ **Dez e não um:** o relógio já soma `1` por segundo, e um botão que
/// somasse o mesmo seria indistinguível do relógio a tocar no mesmo instante.
const BONUS: &str = "10";

/// Os nomes que a cena autora. Ficam juntos porque são um CONTRATO entre quatro sítios (o timer, a
/// tabela, o contador e o botão), e três strings soltas divergiriam na primeira edição.
const SINAL_TICK: &str = "tick";
const SINAL_BONUS: &str = "bonus";
const CONTADOR: &str = "pontos";
const RELOGIO: &str = "ronda";
const PLACAR: &str = "Placar";

/// O que o quadro 1 montou e o quadro 2 vai vestir: cada peça com a **pose local** dela dentro da
/// caixa de referência.
///
/// ⛔⛔ **A posição NÃO pode vir na geometria, e a foto foi quem o disse:** o cozimento de um texto
/// RE-CENTRA o caminho composto e deixa a pose ao `Transform` (é o que o `recook_text_object` faz,
/// e é o que faz um texto arrastar-se pelo gizmo como qualquer forma). ⇒ os três rótulos da 1.ª
/// tentativa nasceram **todos em cima uns dos outros no centro**, com a `origin` autorada a ter
/// sido absorvida no cozimento. *A pose de um filho de canvas mora no `Transform` dele.*
struct Peca {
    id: VecPathId,
    local: [f32; 2],
    /// A sessão que o cozeu — ⚠️ **guardada porque o `VecShape::Text` só se pendura no quadro
    /// SEGUINTE**, quando a entidade existe. Sem ele o rótulo é uma forma como outra qualquer e o
    /// texto vivo **nunca corre** (o `hud_label_live` pergunta por `VecShape::Text`): foi o
    /// `shape=false` do diagnóstico que o disse, com o HUD já a desenhar.
    edit: Option<VecTextEdit>,
}

struct Pendente {
    pontos: Peca,
    resta: Peca,
    botao: Peca,
    rotulo: Peca,
}

static PENDENTE: std::sync::Mutex<Option<Pendente>> = std::sync::Mutex::new(None);

impl crate::App {
    /// No prólogo do quadro. No-op sem a env.
    pub(crate) fn hud_smoke(&mut self) {
        if std::env::var_os("PH2D_HUD_SMOKE").is_none() {
            return;
        }
        match self.components_smokes.hud {
            0 => {
                self.hud_smoke_mundo();
                self.components_smokes.hud = 1;
            }
            1 => {
                self.hud_smoke_veste();
                self.components_smokes.hud = 2;
            }
            _ => {}
        }
    }

    /// Quadro 1: o mundo da cena da câmera, mais as formas do HUD.
    fn hud_smoke_mundo(&mut self) {
        // ⭐ A cena do #7 inteira — postes, cerca, herói e a câmera que o segue. Ela é o que faz o
        // mundo ROLAR, e sem isso esta cena não tem o que provar.
        if let Some(mut cx) = self.components_ctx() {
            ph2d_app_components::camera_2d_smoke::game_camera_smoke(&mut cx);
        }
        self.components_smokes.game_camera = true;
        self.game_camera_preview = true;
        // ⚠️⚠️ **O relógio tem de ANDAR, e são QUATRO linhas e não uma** — a foto apanhou-o: com só
        // o `play()` a régua ficava em `0`, o contador em `0` e a contagem em `30.0 s` para sempre,
        // e a cena lia-se como *«o rótulo vivo não funciona»* sobre um motor correcto. O passo fixo
        // (onde os relógios correm) só avança com o `simulate_physics` armado, e uma instrução que
        // manda rebobinar sobre um ecrã sem transporte devolve *«que régua?»*.
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();

        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⚠️ A geometria nasce na ORIGEM e a pose vai no `Transform` (ver o doc da `Peca`). As
        // coordenadas são as da CAIXA DE REFERÊNCIA, centrada: o canto de cima à esquerda é
        // `(-16, +9)`.
        let mut pontos = texto("Pontos: 0", [235, 235, 245]);
        let mut resta = texto("30.0 s", [235, 200, 120]);
        let mut rotulo = texto("+10", [20, 22, 28]);
        for e in [&mut pontos, &mut resta, &mut rotulo] {
            crate::vec_text::regen_into(&mut gfx.vec_scene, e);
        }
        // O corpo do botão: um rectângulo por baixo do rótulo dele. ⚠️ **Ele é o alvo do clique**
        // — um texto é feito de contornos de glyph, e acertar num deles seria uma pontaria que
        // ninguém tem.
        let mut corpo = crate::build_smoke::shape(
            ph2d_vec_scene::ShapeKind::Rectangle,
            [-2.8, -1.0],
            [2.8, 1.0],
            &[],
            [90, 170, 230],
        );
        corpo.stroke = None;
        let botao = gfx.vec_scene.push_path(corpo);
        let (Some(p), Some(r), Some(l)) = (pontos.id, resta.id, rotulo.id) else {
            return;
        };
        PENDENTE.lock().expect("smoke lock").replace(Pendente {
            // ⚠️ O `-7` e não o `-9`: a geometria de um texto é CENTRADA no cozimento, logo a
            // string cresce para os dois lados — a `-9` o «Pon» caía fora da área desenhada, e a
            // foto leu **«tos: 0»**.
            pontos: Peca {
                id: p,
                local: [-7.0, -5.0],
                edit: Some(pontos),
            },
            resta: Peca {
                id: r,
                local: [8.0, -5.0],
                edit: Some(resta),
            },
            botao: Peca {
                id: botao,
                local: [0.0, -5.0],
                edit: None,
            },
            rotulo: Peca {
                id: l,
                local: [0.0, -5.0],
                edit: Some(rotulo),
            },
        });
    }

    /// Quadro 2: as entidades já existem — vestem-se os componentes.
    fn hud_smoke_veste(&mut self) {
        let Some(pend) = PENDENTE.lock().expect("smoke lock").take() else {
            return;
        };
        let mapa = self.vec.entities.clone();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐ **O `VecShape::Text` primeiro** — é ele que faz um texto ser TEXTO para o resto da
        // casa (o painel, o «Convert to Curves» e, aqui, o texto vivo do HUD). A porta é a mesma
        // que a ferramenta usa.
        for peca in [&pend.pontos, &pend.resta, &pend.rotulo] {
            if let Some(e) = peca.edit.as_ref() {
                crate::vec_text_object::upsert_text_shape(&mut gfx.sim, &mapa, e);
            }
        }
        let ent = |id: VecPathId| mapa.get(&id).map(|&b| Entity::from_bits(b));
        let (Some(e_pontos), Some(e_resta), Some(e_botao), Some(e_rotulo)) = (
            ent(pend.pontos.id),
            ent(pend.resta.id),
            ent(pend.botao.id),
            ent(pend.rotulo.id),
        ) else {
            return;
        };
        let world = gfx.sim.world_mut();

        // ── A RAIZ ───────────────────────────────────────────────────────────
        let canvas = world
            .spawn((
                // ⚠️ A pose nasce na identidade **e não interessa**: ela é CONDUZIDA a cada quadro
                // (`Driver::CanvasPose`). O que o artista autoraria aqui seria apagado no 1.º
                // quadro em que houvesse câmera de jogo — e é isso que o painel explica.
                Transform::default(),
                UiCanvas {
                    ref_w: REF_W,
                    ref_h: REF_H,
                    fit: Fit::Keep,
                },
                Name::new("HUD"),
            ))
            .id();
        // ⚠️ O `ChildOf` **e** a pose local, no mesmo gesto: um filho sem `Transform` próprio
        // herda a pose do pai e aterra no centro do canvas — que foi o defeito que a foto apanhou.
        for (e, p) in [
            (e_pontos, pend.pontos.local),
            (e_resta, pend.resta.local),
            (e_botao, pend.botao.local),
            (e_rotulo, pend.rotulo.local),
        ] {
            world.entity_mut(e).insert((
                ChildOf(canvas),
                Transform::from_translation(ph2d_core::Vec2::new(p[0], p[1])),
            ));
        }

        // ── OS DOIS RÓTULOS ──────────────────────────────────────────────────
        world.entity_mut(e_pontos).insert((
            UiLabel {
                source: LabelSource::Counter(CONTADOR.to_owned()),
                prefix: "Pontos: ".to_owned(),
                suffix: String::new(),
            },
            Name::new("Rotulo dos pontos"),
        ));
        world.entity_mut(e_resta).insert((
            UiLabel {
                source: LabelSource::TimerLeft(RELOGIO.to_owned()),
                prefix: String::new(),
                suffix: " s".to_owned(),
            },
            Name::new("Rotulo da ronda"),
        ));

        // ── O BOTÃO ──────────────────────────────────────────────────────────
        world.entity_mut(e_botao).insert((
            UiButton {
                signal: SINAL_BONUS.to_owned(),
                disabled: false,
            },
            Name::new("Botao de bonus"),
        ));
        world.entity_mut(e_rotulo).insert(Name::new("Rotulo do botao"));

        // ── O PLACAR e os RELÓGIOS ───────────────────────────────────────────
        // ⚠️ O contador vive numa entidade PRÓPRIA, e a tabela alcança-o pelo NOME: é assim que um
        // sinal de qualquer parte da cena chega ao mesmo placar.
        world.spawn((
            // ⛔⛔ **O `Transform` é OBRIGATÓRIO aqui, e não é decoração:** a identidade
            // (`StableId`) só é atribuída a quem tem `Transform` **ou** `ChildOf`, e o
            // `signal_actions::resolve` colhe os reactores com `&StableId` ⇒ **um placar sem pose
            // é invisível à tabela, em silêncio**. Gate:
            // `o_sinal_chega_ao_contador_pelo_nome_e_so_com_identidade`.
            Transform::default(),
            Counter {
                name: CONTADOR.to_owned(),
                start: 0,
            },
            CounterRuntime { value: 0 },
            SignalActions(vec![
                accao(SINAL_TICK, "1"),
                // ⭐ O MESMO contador, por outro caminho: é isto que faz a wave fechar com o #5.
                accao(SINAL_BONUS, BONUS),
            ]),
            Name::new(PLACAR),
        ));
        let relogios = vec![
            relogio(SINAL_TICK, 2_000_000, true),
            relogio(RELOGIO, 30_000_000, false),
        ];
        // ⛔⛔ **`born` e NÃO `TimerState::default()`** — o doc daquela porta di-lo por escrito
        // (*«um `autostart` nasce A CORRER, e o `Default` é parado»*), e a foto apanhou-me: com o
        // `default` os dois relógios nasciam parados, nenhum sinal soava, o contador ficava em `0`
        // e a contagem em `30.0 s` — e a cena lia-se como *«o rótulo vivo não funciona»* sobre um
        // motor correcto. *Uma cena que ensina o contrário do que acontece é pior que nenhuma.*
        let estados: Vec<TimerState> = relogios.iter().map(ph2d_ecs::timer::born).collect();
        world.spawn((
            Transform::default(),
            Timers(relogios),
            TimerRuntime(estados),
            Name::new("Relogios"),
        ));
        // ⛔⛔ **A IDENTIDADE, e sem ela a tabela do #5 é INVISÍVEL — em silêncio.** O
        // `signal_actions::resolve` consulta `(Entity, &SignalActions, &StableId)`, e um reactor
        // sem identidade simplesmente não entra na consulta: o sinal soa, o toast aparece, e
        // NADA acontece. A varredura não corre sozinha a cada quadro — quem spawna chama-a, como
        // o `asset_menu_smoke` e o `vec_tree_settle` já fazem.
        ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());

        eprintln!(
            "[hud-smoke] o mundo ROLA e o HUD NAO: setas movem o heroi · o relogio soma 1 ponto a \
             cada 2 s · o botao +10 soma dez · a contagem desce sozinha."
        );
    }
}

/// Uma linha da tabela do TOP-20 #5: este sinal soma `quanto` ao contador desta entidade.
fn accao(sinal: &str, quanto: &str) -> SignalAction {
    SignalAction {
        on: sinal.to_owned(),
        target: PLACAR.to_owned(),
        verb: SignalVerb::AddToCounter,
        arg: quanto.to_owned(),
        target_by: SignalTarget::Named,
    }
}

/// Um relógio que publica `sinal` ao fechar.
fn relogio(sinal: &str, us: u64, repeat: bool) -> Timer {
    Timer {
        name: sinal.to_owned(),
        duration_us: us,
        repeat,
        autostart: true,
        signal: sinal.to_owned(),
    }
}

/// A sessão de texto que a ferramenta produziria — a mesma porta do `text_wrap_smoke`, para o
/// objecto que nasce aqui ser indistinguível de um digitado.
fn texto(t: &str, rgb: [u8; 3]) -> VecTextEdit {
    VecTextEdit {
        origin: [0.0, 0.0],
        size: TXT,
        weight: 600.0,
        line_height: 1.25,
        tracking: 0.0,
        align: ph2d_vec_text::TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
            rgb[0], rgb[1], rgb[2], 255,
        ))),
        stroke: None,
        text: t.to_owned(),
        wrap_width: None,
        id: None,
        center: [0.0, 0.0],
    }
}
