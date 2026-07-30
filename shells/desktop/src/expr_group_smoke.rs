//! **A cena de um GRUPO da FASE E** — `PH2D_EXPR_GROUP_SMOKE=<n>` (§7.1.5 do plano 12).
//!
//! Três objetos lado a lado, **um por receita**, e a cena **imprime o que montou**. Var
//! própria e não `PH2D_EXPR_SMOKE=<n>` porque aquela cena é outra coisa: ela prova os
//! INSTRUMENTOS (o card é modal, a fita é do objeto, o preview morre com o painel). Um
//! número a mais na mesma variável faria duas perguntas dividirem um interruptor.
//!
//! ⚠️ **A cena AUTORA pelo catálogo** (`RecipeStack::to_formula`, a porta que o card
//! projeta) e **abre o card** no primeiro objeto — uma cena que arma por baixo da mesa pula
//! exactamente a costura que ela deveria provar.
//!
//! ## Grupo 1 — Shake · Sway · Limit
//!
//! As três cobrem os três kinds do modelo, e a cena é montada para que cada uma seja
//! julgada pelo que ela é:
//!
//! * **"Shaker"** — `shake` num X pelado: *treme como uma câmera na mão*.
//! * **"Swayer"** — `sway` num Y pelado: *balança de um lado para o outro, no mesmo ritmo*.
//! * **"Limiter"** — um `sway` de amplitude GRANDE (4 unidades) com um **`limit` de ±1
//!   embaixo**. É a única forma de julgar um MODIFICADOR: sozinho ele não anima nada, e o
//!   que se vê é o teto cortando a onda de quem está acima.
//!
//! O que provar, nesta ordem:
//!
//! 1. Os três se movem, cada um com o seu caráter (orgânico · rítmico · rítmico CORTADO).
//! 2. O **Limiter bate num teto** — ele oscila entre −1 e +1 e não passa, enquanto o Swayer
//!    (mesma receita, sem o Limit) percorre a amplitude cheia.
//! 3. No card do Shaker, **arraste o Amount até o topo**: o objeto tem de continuar NA
//!    TELA. Era aqui que a faixa antiga (topo 40 = 1,57 canvas) o mandava embora.
//! 4. **Detail e Roughness**: suba o Detail para 3 e SÓ ENTÃO o Roughness muda o tremor —
//!    ele é a queda de amplitude entre oitavas, e com uma oitava não há o que atenuar.
//!
//! ⚠️ Se a linha `[expr-group-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_expr_recipes::{KnobValue, RecipeStack, Row};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// Autora a pilha `rows` como a expressão per-clip de `(bits, prop)`, pela porta do card.
fn drive(doc: &mut TimelineDoc, bits: u64, prop: PropKind, rows: Vec<Row>) -> String {
    let mut stack = RecipeStack::new();
    for r in rows {
        stack.push(r);
    }
    let src = stack.to_formula();
    let target = doc.bind(bits, prop);
    let active = doc.active_index();
    doc.set_clip_expr(active, target, Some(src.clone()));
    src
}

fn row(id: ph2d_expr_recipes::RecipeId, sets: &[(&str, f32)]) -> Row {
    let mut r = Row::new(id).expect("a receita está no catálogo");
    for (k, v) in sets {
        r.set(k, KnobValue::Num(*v));
    }
    r
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn expr_group_smoke(&mut self) {
        if self.expr_group_smoke_done {
            return;
        }
        let Some(group) = std::env::var("PH2D_EXPR_GROUP_SMOKE")
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok())
        else {
            return;
        };
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.expr_group_smoke_done = true;

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

        if group != 1 {
            eprintln!("[expr-group-smoke] grupo {group} ainda não tem cena — o G1 é o único.");
            return;
        }

        let shaker = spawn(self, "Shaker", -4.0, 0.0);
        let swayer = spawn(self, "Swayer", 0.0, 0.0);
        let limiter = spawn(self, "Limiter", 4.0, 0.0);

        let doc = &mut self.timeline.doc;
        let shake_src = drive(doc, shaker, PropKind::TranslationY, vec![row("shake", &[])]);
        let sway_src = drive(doc, swayer, PropKind::TranslationY, vec![row("sway", &[])]);
        // ⚠️ O Limiter é a ÚNICA forma de julgar um modificador: uma onda grande por cima e
        // o teto embaixo. Sozinho, um `Limit` não anima nada e a cena não diria nada dele.
        let limit_src = drive(
            doc,
            limiter,
            PropKind::TranslationY,
            vec![
                row("sway", &[("amount", 4.0)]),
                row("limit", &[("min", -1.0), ("max", 1.0)]),
            ],
        );

        // O card abre no Shaker — a costura que a cena existe para exercitar.
        let target = doc
            .binding_for(shaker, PropKind::TranslationY)
            .expect("acabou de bindar")
            .target
            .get();
        ph2d_panel_timeline::state::request_expr_card(target);

        eprintln!(
            "[expr-group-smoke] grupo 1: Shaker · Swayer · Limiter\n  \
             Shaker  (x=-4) shake -> {shake_src}\n  \
             Swayer  (x= 0) sway  -> {sway_src}\n  \
             Limiter (x=+4) sway 4.0 + limit +-1 -> {limit_src}\n\
             O card ja esta ABERTO no Shaker. Prove: (1) os tres se movem, cada um com o seu \
             carater; (2) o Limiter BATE NUM TETO (+-1) enquanto o Swayer percorre a \
             amplitude cheia; (3) no card do Shaker arraste o Amount ate o TOPO -- o objeto \
             tem de continuar NA TELA (a faixa antiga o mandava a 1,57 canvas); (4) suba o \
             Detail para 3 e SO ENTAO o Roughness muda o tremor."
        );
    }
}
