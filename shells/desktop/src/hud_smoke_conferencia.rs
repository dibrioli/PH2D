//! ⭐⭐⭐ **A AUTO-CONFERÊNCIA da cena do HUD** — *o dedo alcança o botão?*
//!
//! Ela vive aqui e não no [`super`] porque é uma RESPONSABILIDADE própria: a cena monta o mundo e
//! veste os componentes; isto pergunta-lhe, depois de tudo pousado, se o gesto que o roteiro manda
//! fazer é sequer possível. ⚠️ **O corte foi cobrado pelo tecto de LOC da shell**, e é melhor do
//! que o ficheiro era — ⛔ nunca uma entrada nova no `FILE_OVERAGE_OK`.
//!
//! ⛔ **Ela existe porque o gesto real não é reproduzível no ecrã virtual:** o XTest é ignorado na
//! Xwayland e o `ydotool` move o rato REAL do dono.
//!
//! ⚠️ **Ela corre UMA vez, na TRANSIÇÃO para o estado terminal** da escada do [`super`] — e isso é
//! um report do dono (*«infinitos logs»*), não uma preferência: ela imprime uma linha por corrida.

use ph2d_ecs::{Entity, UiButton};

impl crate::App {
    /// ⭐⭐⭐ **O botão é alcançável PELO DEDO?** — a pergunta que o TOP-20 #15 pagou caro
    /// (*«a cena estava certa como DADOS e era impossível como GESTO»*), corrida aqui porque o
    /// gesto real não é reproduzível no ecrã virtual (o XTest é ignorado e o `ydotool` move o rato
    /// REAL do dono).
    ///
    /// ⚠️ **No ÚLTIMO quadro da subida, e não no da montagem:** a pose do canvas é conduzida pela
    /// `fase_hud`, logo no quadro em que as peças nascem o botão ainda está na pose autorada — a
    /// conferência ali mediria outro programa.
    /// ⚠️ `pub(super)` e não privada: quem a chama é a escada do [`super`], que decide QUANDO —
    /// este módulo decide O QUÊ.
    pub(super) fn hud_smoke_confere_o_dedo(&mut self) {
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
}
