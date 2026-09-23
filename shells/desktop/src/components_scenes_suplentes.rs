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
            self.components.smokes.trigger_raise =
                self.levanta_o_inspector(self.components.smokes.trigger_raise);
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
            crate::components_scenes::abre_a_regua_da_corrida(hero);
            // ⛔ **O HERÓI nasce ESCOLHIDO** — o roteiro manda ver a secção *Trigger* no painel da
            // direita, e com ninguém escolhido o Inspector diz *«Select an entity in the
            // Hierarchy»*. ⚠️ O `clear()` anda colado ao `selection` (a lei da cena de física).
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O FIM DE JOGO** — `PH2D_RESTART_SMOKE=1`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️⚠️ **Ele faz DUAS coisas que a cena não pode fazer:**
    ///
    /// 1. **Põe o relógio a andar.** ⛔ Sem isto nada nesta cena acontece: a física não corre, o
    ///    espinho não bate, o relógio da batida não anda, e o dono vê um pátio parado. *A corrida é
    ///    o relógio a andar* — a lei que a fábrica e o gatilho já pagam.
    /// 2. **Abre a régua do transporte e escolhe o HERÓI.** O roteiro manda ver as secções
    ///    *Signal Actions* e *Counter Watch*, e com ninguém escolhido o Inspector diz *«Select an
    ///    entity in the Hierarchy»*.
    ///
    /// ⛔ **E ele NÃO toma a vista da câmera do jogo**, ao contrário do irmão do abanão: esta cena
    /// não tem câmera nenhuma — o pátio inteiro cabe na banda, e há gate a medi-lo.
    pub(crate) fn restart_smoke(&mut self) {
        if self.components.smokes.restart {
            self.components.smokes.restart_raise =
                self.levanta_o_inspector(self.components.smokes.restart_raise);
            return;
        }
        if std::env::var_os("PH2D_RESTART_SMOKE").is_none() {
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::restart_smoke::montar(cx.sim.world_mut(), 1);
        self.components.smokes.restart = true;
        self.components.smokes.restart_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("inspector", true);
            // ⚠️ **A RÉGUA DO TRANSPORTE abre junto** — o dono tem de ver que a corrida ANDA, e o
            // recomeço é o relógio a voltar ao princípio: sem a régua ele não vê a prova.
            crate::components_scenes::abre_a_regua_da_corrida(hero);
            // ⛔ **O HERÓI nasce ESCOLHIDO** — os passos (4) a (6) do roteiro nomeiam secções dele.
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        // ⛔⛔⛔ **E ela abre no `Arrange`, não no `Keys` — MEDIDO em 2026-09-20.**
        //
        // O `Keys` é o separador de FÁBRICA, e ali a régua mostra o relógio do **CLIPE**
        // ([`crate::render_loop::fase_timeline_drain`]: *«o clip clock em Keys mode, o da
        // timeline em Arrange»*). ⇒ com a corrida a `5,750 s` — medido por sonda no
        // `advance_ticks`, com `playing=true` — o painel lia **`Time(s) 0`** e o cursor ficava
        // colado ao zero. Foi isto que a foto de 19/09 registou como *«não confirmado»*.
        //
        // ⚠️⚠️ **O relógio do jogo NUNCA esteve parado, e a cena estava certa:** o defeito era
        // esta linha abrir a régua no separador que mostra OUTRO relógio. *Uma cena que abre uma
        // prova e mostra a prova errada é pior que uma cena sem prova nenhuma* — o dono lê «o
        // jogo não anda» sobre um jogo que anda.
        // ⭐ **A porta já existia** — ela nasceu no `SequencePlayer` para exactamente isto: *uma
        // cena que escolhe um objecto E pede o Arrange quer dizer a segunda coisa*, e a ordem em
        // que o `publish_view` as honra é o que o exprime sem um campo de prioridade.
        ph2d_panel_timeline::state::request_arrange_tab();
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25) — `PH2D_SHAKE_SMOKE=1`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️⚠️ **Ele faz TRÊS coisas que a cena não pode fazer, e sem qualquer uma delas o smoke
    /// ensina o contrário do que diz:**
    ///
    /// 1. ⭐⭐⭐ **TOMA a vista da câmera do jogo.** O abanão é um offset do `CameraRuntime`, que só
    ///    chega ao ecrã com a pré-visualização ligada — sem isto a cena monta tudo certo e **nada
    ///    treme**, e o dono lê *«o abanão não funciona»* sobre um motor que funciona. ⛔ É a mesma
    ///    linha que a `=2` da fábrica já paga, e pela mesma família de razão.
    /// 2. **Cria a acção `boom` e liga-a ao `Q`.** Nenhuma das sete do `with_player_defaults` é
    ///    explodir, e a lei do gatilho cala uma acção que o mapa não conhece.
    /// 3. **Põe o relógio a andar.** O `dt` do abanão é o do passo fixo: com a corrida parada ele
    ///    **congela**, de propósito.
    pub(crate) fn shake_smoke(&mut self) {
        if self.components.smokes.shake {
            self.components.smokes.shake_raise =
                self.levanta_o_inspector(self.components.smokes.shake_raise);
            return;
        }
        if std::env::var_os("PH2D_SHAKE_SMOKE").is_none() {
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::shake_smoke::montar(cx.sim.world_mut(), 1);
        self.components.smokes.shake = true;
        self.components.smokes.shake_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        // ⭐⭐⭐ Ver o ponto 1 do doc — sem isto a wave inteira é invisível.
        self.game_camera_preview = true;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            // ⚠️ **A acção nasce AQUI e não na cena** — o mapa vive no `HeroScreen`, que é do
            // editor. ⭐ O `create` devolve a que já existe se o nome repetir.
            let id = hero
                .input_map
                .create(ph2d_app_components::shake_smoke::ACCAO);
            if let Some(a) = hero.input_map.get_mut(id) {
                a.bindings.push(ph2d_input::Binding::Key(ph2d_input::Key(
                    ph2d_app_components::shake_smoke::TECLA,
                )));
            }
            hero.panel_visibility.insert("inspector", true);
            // ⚠️ **A RÉGUA DO TRANSPORTE abre junto** — o passo (6) manda parar a corrida, e uma
            // instrução sobre o transporte num ecrã sem ele devolve *«que régua?»*.
            crate::components_scenes::abre_a_regua_da_corrida(hero);
            // ⛔ **A BOMBA nasce ESCOLHIDA** — o roteiro manda ver a secção *Shake Emitter*.
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
            self.components.smokes.dano_raise =
                self.levanta_o_inspector(self.components.smokes.dano_raise);
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
            crate::components_scenes::abre_a_regua_da_corrida(hero);
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
            self.components.smokes.ray_raise =
                self.levanta_o_inspector(self.components.smokes.ray_raise);
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
            crate::components_scenes::abre_a_regua_da_corrida(hero);
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
            self.components.smokes.tween_raise =
                self.levanta_o_inspector(self.components.smokes.tween_raise);
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
            crate::components_scenes::abre_a_regua_da_corrida(hero);
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

    /// ⭐⭐⭐ **A cena do SEGUIDOR DE CAMINHO** (suplente #23) — `PH2D_PATHFOLLOW_SMOKE=1`.
    ///
    /// ⚠️ **Ela precisa de mais do que o mundo**, ao contrário da irmã do tween: a pista é uma
    /// FORMA DESENHADA, logo a cena escreve na `VecScene` e adopta a entidade dela pelo mesmo passe
    /// que o editor vectorial corre. É por isso que ela recebe o `SceneCtx` inteiro.
    ///
    /// ⛔ **E o relógio TEM de andar** (`simulate_physics` + `play`): o seguidor é uma função pura
    /// do `Timer`, que avança no passo fixo — com o transporte parado a cena abre com quatro
    /// quadrados imóveis, e o dono lê *«o seguidor não funciona»* sobre um componente que está
    /// certo. *A espécie que o §5.0 chama de pior que uma cena ausente.*
    pub(crate) fn path_follow_smoke(&mut self) {
        if self.components.smokes.path_follow {
            self.components.smokes.path_follow_raise =
                self.levanta_o_inspector(self.components.smokes.path_follow_raise);
            return;
        }
        let Some(v) = std::env::var_os("PH2D_PATHFOLLOW_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::path_follow_smoke::montar(&mut cx, nivel);
        self.components.smokes.path_follow = true;
        let Some(montada) = montada else {
            return;
        };
        ph2d_app_components::path_follow_smoke::anuncia();
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            crate::components_scenes::abre_a_regua_da_corrida(hero);
            hero.panel_visibility.insert("inspector", true);
            // ⛔ **Alguém nasce ESCOLHIDO** — com ninguém escolhido o Inspector diz *«Select an
            // entity in the Hierarchy»* e o passo (2) nomeia uma secção que não está na tela.
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.components.smokes.path_follow_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **A cena da PARALAXE** (plano 24, W7) — `PH2D_PARALLAX_SMOKE=1|2`.
    ///
    /// ⚠️⚠️ **Ele faz TRÊS coisas que a cena não pode fazer, e as três são obrigatórias:**
    ///
    /// 1. **TOMA a vista da câmera do jogo.** A paralaxe é medida contra o rectângulo que a
    ///    [`super::render_loop::fase_game_camera`] devolve — com a pré-visualização desligada o
    ///    ecrã mostra a câmera do EDITOR e as camadas andam contra outra coisa: *o dono vê o fundo
    ///    a deslizar sozinho enquanto ele está parado*, que é ensinar o contrário.
    /// 2. **FECHA a régua do transporte.** ⛔ Não *«deixa de a abrir»*: a arrumação vive em
    ///    `~/.ph2d/layout.txt`, fora do repositório, e pode trazê-la aberta. E ela custa ~45 % da
    ///    altura da janela, enquanto a meia-vista da câmera é da JANELA — com a régua aberta o céu
    ///    e o chão desta cena saem do ecrã pelos dois lados.
    /// 3. **Põe o relógio a andar.** A deriva do céu é uma função pura do playhead, e o herói só
    ///    anda com a corrida a correr.
    pub(crate) fn parallax_smoke(&mut self) {
        if self.components.smokes.parallax {
            self.components.smokes.parallax_raise =
                self.levanta_o_inspector(self.components.smokes.parallax_raise);
            return;
        }
        let Some(v) = std::env::var_os("PH2D_PARALLAX_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::parallax_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.parallax = true;
        self.components.smokes.parallax_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        // ⭐⭐⭐ Ver o ponto 1 do doc — sem isto a wave inteira mede outra câmera.
        self.game_camera_preview = true;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("inspector", true);
            // ⭐⭐ Ver o ponto 2 do doc — o orçamento vertical desta cena não cabe com a régua.
            hero.panel_visibility.insert("timeline", false);
            // ⛔ **Alguém nasce ESCOLHIDO** — o roteiro nomeia a secção *Parallax* no painel da
            // direita, e com ninguém escolhido o Inspector diz *«Select an entity in the
            // Hierarchy»*.
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.playhead.rewind();
        self.playhead.play();
    }
}

impl crate::App {
    /// ⭐⭐⭐ **A ARMA DO JOGADOR** — `PH2D_WEAPON_SMOKE=1`. Prólogo do quadro, uma vez.
    ///
    /// ⚠️⚠️ **Ele faz TRÊS coisas que a cena não pode fazer:**
    ///
    /// 1. **Cria a acção no Input Map e liga-a à tecla.** É a MESMA acção e a MESMA tecla da cena
    ///    do gatilho, lidas das consts dela — ⛔ re-declará-las aqui seria a segunda resposta a
    ///    *«que tecla dispara neste app?»*.
    /// 2. **Põe o relógio a andar.** As teclas do jogo são as teclas do editor, logo uma arma só
    ///    dispara com a corrida a correr — e a cerca vive na PONTE.
    /// 3. **Abre a régua do transporte e escolhe a ARMA.** O passo (5) manda ver a secção *Weapon*
    ///    no painel da direita, e com ninguém escolhido o Inspector diz *«Select an entity in the
    ///    Hierarchy»*; e o passo (6) fala do `Pause`, que sem a régua é *«que régua?»*.
    ///
    /// ⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0).
    pub(crate) fn weapon_smoke(&mut self) {
        if self.components.smokes.weapon {
            self.components.smokes.weapon_raise =
                self.levanta_o_inspector(self.components.smokes.weapon_raise);
            return;
        }
        if std::env::var_os("PH2D_WEAPON_SMOKE").is_none() {
            return;
        }
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::weapon_smoke::montar(cx.sim.world_mut(), 1);
        self.components.smokes.weapon = true;
        self.components.smokes.weapon_raise = crate::components_scenes::LEVANTA_O_INSPECTOR;
        self.timeline.flags.simulate_physics = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            // ⭐ O `create` devolve a que já existe se o nome repetir, logo isto é idempotente.
            let id = hero
                .input_map
                .create(ph2d_app_components::weapon_smoke::ACCAO);
            if let Some(a) = hero.input_map.get_mut(id) {
                a.bindings.push(ph2d_input::Binding::Key(ph2d_input::Key(
                    ph2d_app_components::weapon_smoke::TECLA,
                )));
            }
            hero.panel_visibility.insert("inspector", true);
            crate::components_scenes::abre_a_regua_da_corrida(hero);
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        self.playhead.rewind();
        self.playhead.play();
    }
}
