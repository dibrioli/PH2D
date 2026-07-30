//! **Smoke das expressões de propriedade** (ADR-0144/0145 · FASE 0.5 do plano 12).
//! `PH2D_EXPR_SMOKE=1`.
//!
//! ⚠️ **Esta cena foi REESCRITA porque a anterior mentia duas vezes** (auditoria de
//! 2026-07-29):
//!
//! * **D-L — o roteiro mandava usar um widget DELETADO**: ele descrevia o campo de texto
//!   inline (morto na W1 — grep pelo id dele dá **zero**) e mandava limpá-lo para voltar
//!   aos keyframes. Esse passo é **exatamente o gesto do D-I**: o artista não acha o
//!   controle, usa o card, apaga a linha, aperta Apply — e numa prop sem keys o objeto FICA
//!   onde estava. O roteiro ensinava o bug. ⚠️ As frases exatas ficam no doc 13 §4 D-L, e
//!   **não** aqui de propósito: um roteiro que carrega a instrução morta está a um
//!   copiar-colar de voltar a ser ela — o gate
//!   `the_script_never_tells_the_artist_to_use_the_deleted_inline_field` recusa as duas, e
//!   recusou esta linha na primeira versão dela.
//! * **D-F — a cena exercitava ZERO das 50 receitas.** Ela autorava três fórmulas
//!   escritas à mão (`time*1.2`, `value + wiggle(3, 1.2)`, `Slider.x + 2.5`), então o
//!   catálogo que o artista de fato usa — a galeria, as linhas, os knobs — nunca era
//!   tocado por um smoke.
//!
//! Agora as fórmulas saem do **CATÁLOGO** (`RecipeStack::to_formula`, a mesma porta que o
//! card projeta), a cena **ABRE o card** no primeiro objeto (`request_expr_card`, a mesma
//! porta que a linha do menu chama) e o roteiro só nomeia UI que existe.
//!
//! ---
//!
//! Três objetos, em unidades de MUNDO (~±6), tocando em loop `[0, 4]`:
//!
//! * **"Shaker"** — `shake` do catálogo sobre X **sem keyframe nenhum**: a receita mais
//!   usada, na prop mais crua. É o objeto em que o card abre.
//! * **"Swayer"** — X keyado (rampa −4→4) + `sway` sobre uma Y keyada FLAT. A receita
//!   soma por cima: anda na horizontal, oscila na vertical.
//! * **"Jitterer"** — `jitter` sobre X. ⚠️ **É a receita do D-J**: o deslocamento dela
//!   depende do `__seed` do binding, então este objeto e o Shaker **têm de tremer
//!   diferente** — e a FITA do card tem de desenhar o do objeto que ela descreve.
//!
//! ## O que provar (o card já está aberto no Shaker)
//!
//! 1. **Os três se movem ao abrir**, cada um com o seu ritmo.
//! 2. **O card é MODAL** (FASE 0.1): clique no meio da barra de fórmula, no rodapé do
//!    card, e **digite um número** — a caixa **Dur(s)** do transporte, que fica embaixo,
//!    NÃO pode mudar. Role a roda sobre o card: a timeline atrás **não** pode dar zoom.
//! 3. **A fita é do OBJETO** (FASE 0.2): a curva desenhada no card tem de ser o tremor
//!    que o Shaker faz. Selecione o **Jitterer** na cena — o card segue a seleção e a
//!    fita tem de MUDAR (outro seed, outro tremor).
//! 4. **Esconder o painel PARA o preview** (FASE 0.3): aperte `L` para fechar a
//!    timeline. Os objetos param na pose autorada; nada continua andando.
//! 5. **Apagar a linha DEVOLVE a propriedade** (FASE 0.4): no card do Shaker, clique o
//!    **X** da linha e depois **Apply**. O Shaker tem de voltar para onde estava (`x=0`)
//!    e FICAR lá — este é o gesto que o roteiro antigo ensinava e que ficava a 250.
//! 6. **Arrange janela a fórmula**: troque para a aba **Arrange** — há um strip do clip 0
//!    em `[1, 3]s`, então as três receitas só tocam DENTRO do strip, no tempo local dele,
//!    e ficam QUIETAS fora.
//!
//! ⚠️ Se a linha `[expr-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, StackHost, StripSource, TimelineDoc};

/// Bind `(entity, prop)` and author the CATALOG recipe `id` as its per-clip expression.
///
/// ⚠️ Through `RecipeStack::to_formula` — the one door the card projects through — so the
/// smoke drives the object with the same text the gallery would produce. A hand-written
/// formula here would be a scene that proves the evaluator and says nothing about the
/// catalog the artist actually clicks (auditoria §4 D-F).
fn drive_recipe(
    doc: &mut TimelineDoc,
    bits: u64,
    prop: PropKind,
    id: ph2d_expr_recipes::RecipeId,
) -> String {
    let src = ph2d_expr_recipes::RecipeStack::of(&[id]).to_formula();
    let target = doc.bind(bits, prop);
    let active = doc.active_index();
    doc.set_clip_expr(active, target, Some(src.clone()));
    src
}

/// Key `(entity, prop)` linearly `v0 -> v1` over `0..dur` (Linear).
fn ramp(doc: &mut TimelineDoc, bits: u64, prop: PropKind, v0: f32, v1: f32, dur: f64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(bits, prop, s(0.0), AnimValue::Float(v0), Interp::Linear);
    doc.insert_key(bits, prop, s(dur), AnimValue::Float(v1), Interp::Linear);
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn expr_smoke(&mut self) {
        if self.expr_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_EXPR_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.expr_smoke_done = true;

        // ⚠️ Coordenadas em UNIDADES DE MUNDO visíveis (~±6, o mesmo alcance do
        // motion_path_smoke) — não px. Sprite de 0.8 unidade.
        let spawn = |app: &mut crate::App, name: &str, x: f32, y: f32| {
            let gfx = app.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(x, y)),
                    Sprite::atlas(0, [0.8, 0.8], [1.0, 0.7, 0.3, 1.0]),
                    Name::new(name),
                ))
                .id()
                .to_bits()
        };
        let shaker = spawn(self, "Shaker", 0.0, 3.0);
        let swayer = spawn(self, "Swayer", 0.0, 0.0);
        let jitterer = spawn(self, "Jitterer", 0.0, -3.0);

        let doc = &mut self.timeline.doc;
        // Shaker: the catalog's `shake` on a BARE X — the commonest recipe on the crudest
        // property, and the one the card opens on (so step 5 deletes a row that is the only
        // thing driving the channel: exactly the D-I gesture).
        let shake_src = drive_recipe(doc, shaker, PropKind::TranslationX, "shake");
        // Swayer: a keyed X ramp so it travels, plus `sway` over a keyed FLAT Y.
        ramp(doc, swayer, PropKind::TranslationX, -4.0, 4.0, 4.0);
        ramp(doc, swayer, PropKind::TranslationY, 0.0, 0.0, 4.0); // flat -> value = 0
        let sway_src = drive_recipe(doc, swayer, PropKind::TranslationY, "sway");
        // Jitterer: `jitter` — the recipe whose whole point is that it differs PER OBJECT
        // (it reads `__seed`). Two objects with the same recipe must not agree.
        let jitter_src = drive_recipe(doc, jitterer, PropKind::TranslationX, "jitter");

        // ADR-0145 — place a strip of clip 0 in the ARRANGE scene over [1,3), so the SAME
        // per-clip expressions are WINDOWED there: switch to Arrange and they stop OUTSIDE
        // [1,3) and run at the strip-LOCAL time inside it.
        if let Some(lane) = doc.add_lane("scene".into()) {
            let _ = doc.add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 1.0, 3.0);
        }
        doc.set_active_loop_for(false, Some((0.0, 4.0)));
        let shaker_target = doc
            .binding_for(shaker, PropKind::TranslationX)
            .map(|b| b.target.get());

        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(shaker));
            hero.panel_visibility.insert("timeline", true);
        }
        // **Land the artist IN the card**, through the panel's own open door — the scene
        // that leaves them hunting for it is the scene that shipped a script for a widget
        // deleted three waves earlier.
        if let Some(t) = shaker_target {
            ph2d_panel_timeline::state::request_expr_card(t);
        }
        self.playhead.seek(0.0);
        self.playhead.play();

        eprintln!(
            "[expr-smoke] 3 objetos dirigidos por RECEITAS DO CATALOGO (per-clip, ADR-0145; \
             unidades ~±6), loop [0,4]:\n  \
             Shaker  y=+3  shake  -> {shake_src}\n  \
             Swayer  y= 0  sway   -> {sway_src}  (sobre X keyado -4..4)\n  \
             Jitterer y=-3 jitter -> {jitter_src}\n\
             O card ja esta ABERTO no Shaker. Prove, nesta ordem: (2) o card e MODAL — \
             clique a barra de formula e digite: o Dur(s) do transporte NAO muda, e a roda \
             sobre o card NAO da zoom na timeline; (3) a fita e do OBJETO — selecione o \
             Jitterer e a curva TEM de mudar (outro seed); (4) aperte L para esconder a \
             timeline: tudo PARA; (5) no card, X na linha + Apply: o Shaker volta para x=0 \
             e FICA (era isto que ficava a 250); (6) aba ARRANGE: um strip do clip 0 em \
             [1,3s] janela as tres receitas."
        );
    }
}
