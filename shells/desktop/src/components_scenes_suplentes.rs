//! **O PRÓLOGO das cenas dos SUPLENTES** — o olho (#21), o tween (#22), o gatilho e o golpe (#24).
//!
//! # ⛔ Porque este ficheiro existe: o irmão chegou ao TECTO, e a cura é o CORTE
//!
//! O [`crate::components_scenes`] ganha **um prólogo por wave e nunca perde um**, logo o tecto dele
//! não mede autor nenhum — mede *quantas cenas esta família já tem*. Em 2026-09-19 ele estava a
//! `599` de `600` e a cena do tween não cabia. ⛔ **A cura é este corte por responsabilidade, nunca
//! uma entrada nova no `FILE_OVERAGE_OK`** (que continua VAZIO para a shell salvo duas entradas
//! numeradas) — a lei que o `CLAUDE.md` §5.0 escreve para todo tecto que estoura por acumulação.
//!
//! ⚠️ **A fronteira é a que o §5 já usa para narrar estas cenas:** o TOP-20 fica no irmão, os
//! SUPLENTES ficam aqui. *Um corte por «as últimas três» envelhece na wave seguinte; um corte pelo
//! nome que a fila já lhes dá, não.*
//!
//! ⚠️⚠️ **E o corte curou uma DOC ÓRFÃ que ninguém via:** a doc do `topdown_smoke` vivia **180
//! linhas acima** da função dela, colada à do gatilho — as duas eram lidas pelo compilador como
//! **um** bloco do `trigger_smoke`, logo o `rustdoc` do gatilho abria a falar do mover de vista de
//! cima e o mover não tinha doc nenhuma. *Duas docs contíguas não dão erro: a segunda não substitui
//! a primeira, ela CONTINUA-A* — e é por isso que o defeito é mudo. Ela voltou para junto da função
//! no mesmo commit.
//!
//! ⚠️ As três guardas de cada prólogo são as do irmão (latch · env · o mundo já subiu?), e a razão
//! de nenhuma ser pergunta da família está escrita lá.

impl crate::App {
    /// ⭐⭐⭐ **O GATILHO** (suplente #24). Prólogo do quadro, uma vez.
    ///
    /// ⚠️⚠️ **Ele faz DUAS coisas que a cena não pode fazer, e sem qualquer uma delas o smoke
    /// ensina o contrário do que diz:**
    ///
    /// 1. **Cria a acção `fire` no Input Map e liga-a ao ESPAÇO.** O
    ///    `InputMap::with_player_defaults` tem sete acções e **nenhuma é disparar** (medido) —
    ///    e a lei do gatilho cala uma acção que o mapa não conhece, de propósito. Sem este passo o
    ///    dono carrega na tecla, nada sai, e ele lê *«o gatilho não funciona»* sobre um componente
    ///    que está certo.
    /// 2. **Põe o relógio a andar.** As teclas do jogo são as teclas do editor, logo um gatilho só
    ///    fala com a corrida a correr — e a fábrica dele também.
    ///
    /// ⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0).
    pub(crate) fn trigger_smoke(&mut self) {
        if self.components.smokes.trigger {
            self.trigger_smoke_traz_o_inspector();
            return;
        }
        if std::env::var_os("PH2D_TRIGGER_SMOKE").is_none() {
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::trigger_smoke::montar(cx.sim.world_mut(), 1);
        self.components.smokes.trigger = true;
        self.components.smokes.trigger_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            // ⚠️ **A acção nasce AQUI e não na cena** — o mapa vive no `HeroScreen`, que é do
            // editor, e a cena só vê o mundo. ⭐ O `create` devolve a que já existe se o nome
            // repetir, logo isto é idempotente por construção.
            let id = hero
                .input_map
                .create(ph2d_app_components::trigger_smoke::ACCAO);
            if let Some(a) = hero.input_map.get_mut(id) {
                // ⛔⛔ **A tecla é MEDIDA e vive na cena** — ver [`trigger_smoke::TECLA`]. A 1.ª
                // redacção usava o ESPAÇO, e o dono devolveu-a: *«espaço é o atalho do play da
                // timeline e há conflito»*. Ele é o **Play/Pause do transporte**, logo um toque
                // parava a corrida E disparava.
                a.bindings.push(ph2d_input::Binding::Key(ph2d_input::Key(
                    ph2d_app_components::trigger_smoke::TECLA,
                )));
            }
            // ⭐⭐⭐ **E uma acção SEM TECLA, de propósito** — o sujeito do passo (6) do roteiro.
            //
            // ⛔⛔ **Ela tem de ser criada aqui, e isso foi MEDIDO:** as sete acções do
            // `with_player_defaults` têm todas ligação, logo nenhuma serve de exemplo. Ela é o
            // estado que qualquer artista alcança ao criar uma acção e esquecer a tecla — o
            // gatilho fica calado, e o painel dizia que estava tudo bem.
            let _ = hero
                .input_map
                .create(ph2d_app_components::trigger_smoke::ACCAO_SEM_TECLA);
            hero.panel_visibility.insert("inspector", true);
            // ⚠️⚠️ **A RÉGUA DO TRANSPORTE abre junto, e a FOTO é que o disse:** esta cena inteira
            // é sobre uma cerca do RELÓGIO (*Play → a arma dispara · Stop → o teclado volta a ser
            // do editor*), e sem a timeline o dono não vê que a corrida anda nem tem onde a parar.
            // *Uma instrução que fala do transporte sobre um ecrã sem ele devolve «que régua?»* —
            // a lição da cena 67 da física, que as irmãs `=1` do topdown e do projéctil já pagam.
            hero.panel_visibility.insert("timeline", true);
            // ⛔ **O HERÓI nasce ESCOLHIDO** — o roteiro manda ver a secção *Trigger* no painel da
            // direita, e com ninguém escolhido o Inspector diz *«Select an entity in the
            // Hierarchy»*. ⚠️ O `clear()` anda colado ao `selection` (a lei da cena de física).
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O GOLPE** (suplente #24, 19/09) — `PH2D_DANO_SMOKE=1`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **Ele é o do gatilho mais o relógio a andar**, e as duas metades são obrigatórias: a
    /// acção `fire` não existe de fábrica (as sete do `with_player_defaults` são outras), e sem a
    /// corrida nem os alvos nascem nem o `Q` dispara.
    pub(crate) fn dano_smoke(&mut self) {
        if self.components.smokes.dano {
            self.dano_smoke_traz_o_inspector();
            return;
        }
        if std::env::var_os("PH2D_DANO_SMOKE").is_none() {
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        // ⚠️ **A árvore de tags entra aqui** — a cena autora três tags (os dois postos e a `Bala`),
        // e ela é documento do PROJECTO, não do mundo.
        let montada = ph2d_app_components::dano_smoke::montar(cx.sim.world_mut(), cx.tags, 1);
        self.components.smokes.dano = true;
        self.components.smokes.dano_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            // ⚠️ **A acção nasce AQUI e a TECLA vem da cena do gatilho**, que é a fonte: ela foi
            // MEDIDA (o espaço é o Play/Pause do transporte, e o dono devolveu a 1.ª redacção por
            // isso). Escrever o código da tecla aqui daria a segunda resposta a *«qual é a tecla?»*.
            let id = hero
                .input_map
                .create(ph2d_app_components::dano_smoke::ACCAO);
            if let Some(a) = hero.input_map.get_mut(id) {
                a.bindings.push(ph2d_input::Binding::Key(ph2d_input::Key(
                    ph2d_app_components::trigger_smoke::TECLA,
                )));
            }
            hero.panel_visibility.insert("inspector", true);
            // ⚠️ **A RÉGUA abre junto** — esta cena inteira depende do relógio A ANDAR (os alvos
            // nascem de um `Timer`), e sem a timeline o dono não VÊ que a corrida anda. *Uma
            // instrução que fala do transporte sobre um ecrã sem ele devolve «que régua?»* — a
            // lição da cena 67 da física.
            // ⛔ **Mas quem o roteiro manda carregar são os chips `Pause`/`Reset` da barra de
            // CIMA**, e não esta régua: ela pinta ÍCONES, e a 1.ª redacção mandava carregar num
            // «STOP» que não é pintado em lado nenhum (report do dono, 19/09).
            hero.panel_visibility.insert("timeline", true);
            // ⛔ **O HERÓI nasce ESCOLHIDO** — o roteiro manda ver a secção no painel da direita, e
            // com ninguém escolhido o Inspector diz *«Select an entity in the Hierarchy»*.
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O OLHO** (suplente #21, W6) — `PH2D_RAY_SMOKE=1`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **As três metades do prólogo são obrigatórias**, e cada uma por uma razão medida: sem a
    /// física ARMADA o raio nunca casta (o toggle nasce desmarcado); sem o relógio a ANDAR a caixa
    /// não chega; e ⭐⭐ **sem o `show_colliders` a cena inteira é invisível** — o desenho do raio
    /// acompanha o MESMO interruptor do contorno, porque é a mesma pergunta (*mostre-me a física
    /// que não se vê*), e uma cena cuja lição é uma LINHA que o artista tem de saber ligar é uma
    /// cena que ensina o contrário do que promete.
    pub(crate) fn ray_smoke(&mut self) {
        if std::env::var_os("PH2D_RAY_SMOKE").is_none() {
            return;
        }
        if self.components.smokes.ray {
            // ⚠️ **A ordem dos dois guardas é a lei**: a cena monta-se uma vez e o Inspector tem de
            // subir em VÁRIOS quadros, logo a saída antecipada não pode ser a primeira.
            self.ray_smoke_traz_o_inspector();
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::ray_smoke::montar(cx.sim.world_mut(), 1);
        self.components.smokes.ray = true;
        self.timeline.flags.simulate_physics = true;
        // ⭐⭐ **O overlay da física LIGADO** — ver o doc acima. É ele que desenha a linha do raio.
        self.show_colliders = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("inspector", true);
            // ⚠️ A régua abre junto — os passos (2) e (3) falam do relógio a andar, e *uma
            // instrução que fala do transporte sobre um ecrã sem ele devolve «que régua?»* (a
            // lição da cena 67 da física).
            hero.panel_visibility.insert("timeline", true);
            // ⛔ **O OLHO DA FRENTE nasce ESCOLHIDO** — o roteiro manda ler a secção `Ray Sensor`
            // no painel da direita, e com ninguém escolhido o Inspector diz *«Select an entity in
            // the Hierarchy»*.
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        // ⛔⛔ **E o Inspector tem de SUBIR durante alguns quadros, não só ficar visível** — a
        // `panel_visibility` diz *«existe»* e não *«está à frente»*. O `reconcile_z` acrescenta os
        // painéis em falta no **início de cada quadro**, logo um `bump` feito no quadro do arranque
        // fica **por baixo** do que ele acrescenta a seguir. ⚠️ A FOTO desta cena abriu com o painel
        // do **Sculpt 3D** à frente, sobre um roteiro que manda ler a secção `Ray Sensor` — a lição
        // que a wave das PARTÍCULAS pagou, e a primeira cura dela também não chegou.
        self.components.smokes.ray_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.playhead.rewind();
        self.playhead.play();
    }

    /// Traz o Inspector à frente por alguns quadros — ver o irmão da vigia do contador.
    pub(crate) fn ray_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.ray_raise == 0 {
            return;
        }
        self.components.smokes.ray_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }

    /// ⭐⭐⭐ **O TWEEN** (suplente #22) — `PH2D_TWEEN_SMOKE=1|2`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️⚠️ **As duas metades do prólogo são obrigatórias, e cada uma por uma razão medida:**
    ///
    /// 1. **O relógio tem de ANDAR.** Um tween é função do `Timer` do mesmo índice, e um timer só
    ///    avança no **passo fixo** — com o transporte parado a galeria abre com quatro quadrados
    ///    imóveis, e o dono lê *«o tween não funciona»* sobre um componente que está certo.
    /// 2. **A RÉGUA abre junto.** O roteiro manda carregar em `Pause` e ver tudo congelar, e *uma
    ///    instrução que fala do transporte sobre um ecrã sem ele devolve «que régua?»* — a lição da
    ///    cena 67 da física, que as irmãs `=1` do topdown e do projéctil já pagam.
    ///
    /// ⛔ **E o Inspector é trazido à FRENTE nos quadros SEGUINTES** (`LEVANTA_O_INSPECTOR`): o
    /// passo (3) fala da secção *Tween* «no painel da direita», e a arrumação vive **fora do
    /// repositório** (`~/.ph2d/layout.txt`) — sem isto a cena depende do que ficou aberto ontem.
    pub(crate) fn tween_smoke(&mut self) {
        if self.components.smokes.tween {
            self.tween_smoke_traz_o_inspector();
            return;
        }
        let Some(v) = std::env::var_os("PH2D_TWEEN_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::tween_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.tween = true;
        // ⚠️ **A `=2` precisa disto e a `=1` não** — as cópias nascem de uma fábrica, que corre no
        // passo fixo. ⛔ Ligar só na `=2` daria duas respostas a *«o que é uma corrida?»*.
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
            hero.panel_visibility.insert("inspector", true);
            // ⛔ **Alguém nasce ESCOLHIDO** — com ninguém escolhido o Inspector diz *«Select an
            // entity in the Hierarchy»* e o passo (3) nomeia uma secção que não está na tela.
            // ⚠️ O `clear()` anda colado ao `selection` (a lei da cena de física).
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.components.smokes.tween_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.playhead.rewind();
        self.playhead.play();
    }

    /// Traz o Inspector à frente no encaixe dele, por alguns quadros. Ver `tween_raise`.
    fn tween_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.tween_raise == 0 {
            return;
        }
        self.components.smokes.tween_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }
}

impl crate::App {
    /// Traz o Inspector à frente no encaixe dele, por alguns quadros. Ver `trigger_raise`.
    fn trigger_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.trigger_raise == 0 {
            return;
        }
        self.components.smokes.trigger_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }
}

impl crate::App {
    /// Traz o Inspector à frente no encaixe dele, por alguns quadros. Ver `dano_raise`.
    fn dano_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.dano_raise == 0 {
            return;
        }
        self.components.smokes.dano_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }
}
