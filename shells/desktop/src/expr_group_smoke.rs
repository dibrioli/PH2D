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
//! * **"Shaker"** — o objeto do CARD, e ele nasce **sem fórmula** (ver o ⚠️ abaixo).
//! * **"Swayer"** — `sway` num Y pelado: *balança de um lado para o outro, no mesmo ritmo*.
//! * **"Limiter"** — um `sway` de amplitude GRANDE (4 unidades) com um **`limit` de ±1
//!   embaixo**. É a única forma de julgar um MODIFICADOR: sozinho ele não anima nada, e o
//!   que se vê é o teto cortando a onda de quem está acima.
//!
//! ⚠️⚠️ **O Shaker nasce SEM fórmula, e a primeira versão desta cena se auto-derrotou por
//! não saber disso** (smoke do Enio, 2026-07-30: *"todos aparecem como custom"*). O card
//! **não reconstrói as linhas a partir do texto** — é decisão declarada da crate (*"um
//! reconhecedor de fragmentos canônicos começa a MENTIR no dia em que alguém edita um
//! caractere"*), então uma fórmula autorada por FORA dele volta como uma linha
//! **`Custom Formula`** com o texto cru: sem Amount, sem Detail, sem Roughness. Uma cena
//! que pré-autora o objeto do card entrega ao artista exactamente o card que não tem os
//! knobs que ela manda arrastar.
//!
//! Então o Shaker chega **vazio** e o passo 3 é o gesto REAL: escolher `Shake` na galeria.
//!
//! O que provar, nesta ordem:
//!
//! 1. **Swayer e Limiter se movem** — rítmico e rítmico CORTADO.
//! 2. O **Limiter bate num teto** — ele oscila entre −1 e +1 e não passa, enquanto o Swayer
//!    (mesma receita, sem o Limit) percorre a amplitude cheia.
//! 3. No card (aberto no **Shaker**, vazio), abra a família **Life** e escolha **Shake**: o
//!    objeto começa a tremer, e agora existem knobs. **Arraste o Amount até o topo** — ele
//!    tem de continuar NA TELA. Era aqui que a faixa antiga (topo 40 = 1,57 canvas) o
//!    mandava embora.
//! 4. **Detail e Roughness**: suba o Detail para 3 e SÓ ENTÃO o Roughness muda o tremor —
//!    ele é a queda de amplitude entre oitavas, e com uma oitava não há o que atenuar.
//! 5. ⚠️ **O que ESTA cena expõe e a wave NÃO conserta:** aperte **Apply**, feche o card e
//!    reabra no Shaker. As linhas somem e sobra `Custom Formula` com o texto. É a lacuna de
//!    ida-e-volta da folha, e fechá-la é decisão de produto (ver o plano 12).
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
        // ⚠️ O Shaker só é BINDADO — sem fórmula. Ver o ⚠️⚠️ do doc do módulo: uma fórmula
        // autorada por fora do card volta como `Custom Formula` (texto cru, zero knobs), e
        // o passo 3 manda arrastar um knob.
        let shaker_target = doc.bind(shaker, PropKind::TranslationY).get();
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

        // ⚠️ E a cena SELECIONA o Shaker. Sem isto o card segue a seleção da cena (a feature
        // do ADR-0144) e pousa em qualquer objeto que o artista tenha clicado — o roteiro
        // diz "no card do Shaker" e a tela mostra outro nome.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(shaker));
            hero.panel_visibility.insert("timeline", true);
        }
        // O card abre no Shaker — a costura que a cena existe para exercitar.
        ph2d_panel_timeline::state::request_expr_card(shaker_target);

        eprintln!(
            "[expr-group-smoke] grupo 1: Shaker · Swayer · Limiter\n  \
             Shaker  (x=-4) SEM formula -- o card abre VAZIO nele, de proposito\n  \
             Swayer  (x= 0) sway  -> {sway_src}\n  \
             Limiter (x=+4) sway 4.0 + limit +-1 -> {limit_src}\n\
             Prove: (1) Swayer e Limiter se movem; (2) o Limiter BATE NUM TETO (+-1) \
             enquanto o Swayer percorre a amplitude cheia; (3) no card (aberto no Shaker, \
             VAZIO) abra a familia Life e escolha Shake -- agora ha knobs: arraste o Amount \
             ate o TOPO e o objeto tem de continuar NA TELA (a faixa antiga o mandava a \
             1,57 canvas); (4) suba o Detail para 3 e SO ENTAO o Roughness muda o tremor; \
             (5) Apply, feche e reabra o card: as linhas somem e sobra `Custom Formula` -- \
             a lacuna de ida-e-volta, que esta wave NAO conserta."
        );
    }
}
