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
        match self.components.smokes.hud {
            0 => {
                self.hud_smoke_mundo();
                self.components.smokes.hud = 1;
            }
            1 => {
                self.hud_smoke_veste();
                self.components.smokes.hud = 2;
            }
            _ => self.hud_smoke_traz_o_inspector(),
        }
    }

    /// Traz o Inspector à frente no encaixe dele, por alguns quadros.
    fn hud_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.hud_raise == 0 {
            return;
        }
        self.components.smokes.hud_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
        if self.components.smokes.hud_raise == 0 {
            self.hud_smoke_confere_o_dedo();
        }
    }

    /// ⭐⭐⭐ **O botão é alcançável PELO DEDO?** — a pergunta que o TOP-20 #15 pagou caro
    /// (*«a cena estava certa como DADOS e era impossível como GESTO»*), corrida aqui porque o
    /// gesto real não é reproduzível no ecrã virtual (o XTest é ignorado e o `ydotool` move o rato
    /// REAL do dono).
    ///
    /// ⚠️ **No ÚLTIMO quadro da subida, e não no da montagem:** a pose do canvas é conduzida pela
    /// `fase_hud`, logo no quadro em que as peças nascem o botão ainda está na pose autorada — a
    /// conferência ali mediria outro programa.
    fn hud_smoke_confere_o_dedo(&mut self) {
        let mapa = self.vec.entities.clone();
        let tol = 10.0 * self.vec_px_to_world();
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        // O caminho do botão: o único cujo dono carrega um `UiButton`.
        let Some((&id, &bits)) = mapa.iter().find(|&(_, &b)| {
            gfx.sim
                .world()
                .get::<UiButton>(Entity::from_bits(b))
                .is_some()
        }) else {
            eprintln!("[hud-smoke] ⛔ nenhum caminho carrega um UiButton");
            return;
        };
        let t = ph2d_vec_entities::transform::world_transform(&gfx.sim, Entity::from_bits(bits));
        let centro = [f64::from(t.translation.x), f64::from(t.translation.y)];
        let achou = self.vec.pen.path_at(&gfx.vec_scene, centro, tol);
        // ⚠️ **As COLUNAS do porquê, e não só o veredito** — um `NAO` sem elas manda procurar em
        // três camadas de uma vez (a pose, o afim, a elegibilidade). O afim é reconstruído pela
        // MESMA porta do quadro (`transform::build`), logo isto não é uma segunda resposta.
        let xf_todos = ph2d_vec_entities::transform::build(&gfx.sim, &mapa);
        let xf = ph2d_vec_scene::xform_of(&xf_todos, id);
        let local = xf.inverse().map_or(centro, |inv| inv.apply(centro));
        let dist = gfx
            .vec_scene
            .paths()
            .iter()
            .find(|q| q.id == id)
            .and_then(|q| ph2d_vec_scene::nearest_point_on_path(q, local, 64))
            .map_or(f64::INFINITY, |(_, _, d2)| d2.sqrt() * xf.mean_scale());
        // ⭐⭐ **O veredito passa pela LEI do produto, e não por uma comparação de ids** — o dedo
        // aterra no RÓTULO e é a subida da cadeia que faz disso o botão. Comparar `achou == id`
        // media outro programa: reprovava a cena com o clique a funcionar, e aprovaria um dia em
        // que o rótulo saísse de cima do corpo com a fiação partida.
        let dono = achou
            .and_then(|a| mapa.get(&a).copied())
            .and_then(|b| ph2d_ecs::hud::botao_de(gfx.sim.world(), Entity::from_bits(b)));
        eprintln!(
            "[hud-smoke] o dedo alcanca o botao: {} (centro de mundo {centro:?}, achou={achou:?}, \
             esperado={id:?}, dono={dono:?}, tolerancia={tol:.3}, local={local:?}, escala={:.3}, \
             dist_mundo={dist:.3})",
            if dono == Some(Entity::from_bits(bits)) {
                "SIM"
            } else {
                "NAO"
            },
            xf.mean_scale()
        );
        if std::env::var_os("PH2D_HUD_PROBE").is_some() {
            self.hud_smoke_conduz_o_clique(centro, Entity::from_bits(bits));
        }
    }

    /// ⭐⭐⭐ **O GESTO INTEIRO, pela porta do produto** — `PH2D_HUD_PROBE=1`.
    ///
    /// ⛔⛔ **A conferência de cima mede só a RESOLUÇÃO** (*que forma está sob o dedo, e de quem
    /// ela é*) — e ela leu `SIM` sobre um botão que o dono reportou como **não funcionando**.
    /// *Uma régua que mede metade de uma corrente aprova a corrente partida na outra metade.*
    /// ⇒ isto conduz o `ramo_botao_do_hud` REAL com um `Baixo` e um `Cima` na posição de ECRÃ do
    /// botão, e imprime **cada guarda** mais o contador antes e depois.
    ///
    /// ⚠️ **Fora da omissão de propósito:** ele SOMA pontos, logo mudaria a cena que o dono vê.
    fn hud_smoke_conduz_o_clique(&mut self, centro: [f64; 2], alvo: Entity) {
        use ph2d_host::{PointerButton, PointerKind};
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        // ⚠️ **Pela BANDA da cena, e não pela janela** — foi exactamente aqui que a 1.ª redacção
        // desta sonda mediu outro sítio (`787` contra os `432` da foto). A porta é a mesma que o
        // pick usa ([`crate::App::scene_window`]), senão o arnês e o produto discordam por
        // construção.
        let janela = self.scene_window().unwrap_or_else(|| gfx.surface.size());
        let tela = gfx
            .camera
            .world_to_screen([centro[0] as f32, centro[1] as f32], janela);
        let painel = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| h.store.panel_at(tela.0, tela.1).is_some());
        let widget = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| h.hit_index.hit(tela.0, tela.1).is_some());
        let on_canvas = painel == Some(false) && widget == Some(false);
        // ⚠️ A VOLTA: se `screen_to_world(world_to_screen(p)) == p`, então o espaço do PICK e o
        // espaço que esta sonda usa são o mesmo — e uma discordância com o que a FOTO mostra passa
        // a ser do desenho, não da sonda.
        let volta = self.vec_world_at(tela);
        let tamanho = self.gfx.as_ref().map(|g| {
            let s = g.surface.size();
            (s.width, s.height)
        });
        // ⚠️ **A posição que a FOTO mediu** (o centro do rectângulo azul, `964,432`) contra a que a
        // câmera calcula: se as duas discordarem, o HUD é PINTADO num sítio e PICADO noutro.
        let na_foto = self.vec_world_at((964.0, 432.0));
        // ⭐⭐ **A VARREDURA: ONDE, no ecrã, o dedo de facto encontra o botão.** ⛔ Derivar a
        // posição de ecrã por `world_to_screen` foi o que me pôs a medir OUTRO sítio — isto não
        // deriva nada: pergunta ao mesmo `path_at` que o produto usa, ponto a ponto, e devolve a
        // CAIXA das posições que resolvem para o botão. Comparada com a caixa que a FOTO mede, ela
        // diz numa corrida se o que se vê é o que se pega.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let (lw, lh) = tamanho.unwrap_or((0, 0));
        let mut n = 0u32;
        let mut sy = 0;
        while sy < lh {
            let mut sx = 0;
            while sx < lw {
                let ponto = (sx as f32, sy as f32);
                if self
                    .vec_world_at(ponto)
                    .and_then(|w| {
                        let gfx = self.gfx.as_ref()?;
                        let tol = 10.0 * self.vec_px_to_world();
                        let id = self.vec.pen.path_at(&gfx.vec_scene, w, tol)?;
                        let b = *self.vec.entities.get(&id)?;
                        ph2d_ecs::hud::botao_de(gfx.sim.world(), ph2d_ecs::Entity::from_bits(b))
                    })
                    .is_some_and(|e| e == alvo)
                {
                    n += 1;
                    x0 = x0.min(ponto.0);
                    y0 = y0.min(ponto.1);
                    x1 = x1.max(ponto.0);
                    y1 = y1.max(ponto.1);
                }
                sx += 8;
            }
            sy += 8;
        }
        eprintln!(
            "[hud-smoke] o botao e' alcancavel na caixa de ECRA x {x0}..{x1} y {y0}..{y1} \
             ({n} pontos de sonda) · camera no PROLOGO centro={:?} altura={:?} · preview={}",
            self.gfx.as_ref().map(|g| g.camera.center),
            self.gfx.as_ref().map(|g| g.camera.height_world),
            self.game_camera_preview
        );
        // ⚠️ **O SPLIT do centro** — se a cena desenha numa banda e o cursor é mapeado contra a
        // JANELA, o que se vê e o que se pega vivem em espaços diferentes.
        let split = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| h.view.center_split);
        let banda = split.and_then(|sp| {
            let (w, h) = tamanho.unwrap_or((0, 0));
            sp.scene_viewport(w as f32, h as f32)
        });
        eprintln!("[hud-smoke] split do centro={split:?} banda da cena={banda:?}");
        let antes = self.hud_smoke_pontos();
        self.last_pointer = tela;
        let baixo = self.ramo_botao_do_hud(PointerKind::Down, PointerButton::Primary, on_canvas);
        let cima = self.ramo_botao_do_hud(PointerKind::Up, PointerButton::Primary, on_canvas);
        eprintln!(
            "[hud-smoke] gesto no botao: tela={tela:?} a_correr={} on_canvas={on_canvas} \
             (painel={painel:?} widget={widget:?}) consumiu_baixo={baixo} consumiu_cima={cima} \
             pontos {antes:?} -> {:?} volta={volta:?} surface={tamanho:?} mundo_na_foto={na_foto:?}",
            self.playhead.is_playing(),
            self.hud_smoke_pontos()
        );
    }

    /// O valor VIVO do contador do placar, se a cena o tiver.
    fn hud_smoke_pontos(&self) -> Option<i64> {
        let gfx = self.gfx.as_ref()?;
        let mut q = gfx.sim.world().try_query::<&ph2d_ecs::CounterRuntime>()?;
        q.iter(gfx.sim.world()).next().map(|c| c.value)
    }

    /// Quadro 1: o mundo da cena da câmera, mais as formas do HUD.
    fn hud_smoke_mundo(&mut self) {
        // ⭐ A cena do #7 inteira — postes, cerca, herói e a câmera que o segue. Ela é o que faz o
        // mundo ROLAR, e sem isso esta cena não tem o que provar.
        if let Some(mut cx) = self.components_ctx() {
            ph2d_app_components::camera_2d_smoke::game_camera_smoke(&mut cx);
        }
        self.components.smokes.game_camera = true;
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
        ] {
            world.entity_mut(e).insert((
                ChildOf(canvas),
                Transform::from_translation(ph2d_core::Vec2::new(p[0], p[1])),
            ));
        }
        // ⛔⛔ **O rótulo é filho do BOTÃO, e isso é a FIAÇÃO e não arrumação** — a
        // auto-conferência mediu-o: o dedo no centro do `+10` devolve o caminho do TEXTO, porque o
        // hit-test de objecto entrega a forma mais ao topo que contém o ponto. É a subida da cadeia
        // (`ph2d_ecs::hud::botao_de`) que faz disso um clique no botão, e ela precisa que a cadeia
        // EXISTA. ⚠️ Pose local `(0, 0)`: a posição é herdada, e escrever `(0, −5)` outra vez
        // somaria duas vezes o mesmo deslocamento — o rótulo sairia da tela por baixo.
        world.entity_mut(e_rotulo).insert((
            ChildOf(e_botao),
            Transform::from_translation(ph2d_core::Vec2::ZERO),
        ));

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
        world
            .entity_mut(e_rotulo)
            .insert(Name::new("Rotulo do botao"));

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

        // ⭐ **A cena abre com o RÓTULO escolhido e o Inspector à frente** (a lição do #18): a
        // secção HUD é o que o dono vem cá ver, e um passo que manda olhar para um painel que está
        // por baixo de outro nomeia uma superfície que ele não tem à vista.
        // ⚠️ A arrumação vive FORA do repositório (`~/.ph2d/layout.txt`), logo isto não é
        // defensivo: é a única forma de a cena não depender do que ficou aberto ontem.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("inspector", true);
            hero.gizmo.selection = Some(e_pontos.to_bits());
            hero.gizmo.extra_selection.clear();
        }
        self.components.smokes.hud_raise = 3;
        eprintln!(
            "[hud-smoke] o mundo ROLA e o HUD NAO: setas movem o heroi · o relogio soma 1 ponto a \
             cada 2 s · o botao +10 soma dez · a contagem desce sozinha. O rotulo dos pontos abre \
             ESCOLHIDO, e a seccao HUD do Inspector mostra a fonte dele."
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
        from: ph2d_ecs::SignalFrom::Anyone,
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
