//! **A COSTURA do gizmo dos deformadores de quadrilátero está LIGADA** — o censo de
//! fonte que impede as alças de virarem decoração.
//!
//! ⚠️ **Por que um gate de FONTE e não um de comportamento.** A geometria já tem dez
//! gates puros (`render_loop::warp_gizmo::tests`) e o desenho é tinta; o que nenhum deles
//! vê é a **ligação**: publicar o retrato no prólogo, desenhá-lo, e chamar as três pontas
//! do ponteiro. Cada uma dessas linhas pode ser apagada sem que um único gate de
//! geometria fique vermelho — e o resultado seria um gizmo perfeito que nunca aparece.
//! É a mesma classe que o censo das tomadas de sinal deste mesmo diretório guarda, e a
//! razão de ele existir lá: *a costura é o que se perde num merge, não a matemática.*

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("src/{name}")).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// ⚠️ O lado da FAMÍLIA — o `warp_overlay` mudou-se para `ph2d-app-motion` na Fase C
/// (2026-09-12), porque o gizmo de quadrilátero é do `motion.bezier_warp`/`four_point_warp`.
/// O `warp_gizmo_drag` FICOU: ele segura o estado do arrasto, que é da shell.
fn fam(name: &str) -> String {
    fs::read_to_string(format!("../../crates/ph2d-app-motion/src/{name}"))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O RETRATO é publicado, e é publicado com a modalidade da tool.**
#[test]
fn the_view_is_published_once_per_frame_gated_by_the_motion_tool() {
    // ⚠️ **ACHATADO desde a Fase C (2026-09-12):** a chamada passou a ser qualificada
    // (`ph2d_app_motion::warp_gizmo::…`) e o `cargo fmt` partiu-a em quatro linhas. Uma agulha
    // que casa uma linha inteira mede a FORMATAÇÃO junto com a lei; achatar mede só a lei.
    // ⚠️ O QUADRO emendado (`frame_text::render_frame`), e não o `render_loop/mod.rs` (OBRA 2 da `line/render-loop`,
    // 2026-09-13): a publicação ainda mora no `mod.rs`, e o desenho mudou-se para a `fase_vector_overlays`
    // (doc 109 §6 — ele tem de vir DEPOIS da arte) — os dois testes deste par leem a mesma lente.
    let s = crate::frame_text::render_frame()
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        s.contains("ph2d_app_motion::warp_gizmo::publish(ph2d_app_motion::warp_gizmo::resolve( motion, motion_tool_active, ))"),
        "o retrato tem de ser publicado no prólogo, e gateado pela tool Motion — sem a \
         modalidade, as alças de um nó apareceriam sobre o canvas de outra ferramenta"
    );
}

/// **E ELE É DESENHADO.**
#[test]
fn the_published_view_is_actually_drawn() {
    // ⚠️ O QUADRO emendado: o desenho do retrato vive na `fase_vector_overlays`, logo a seguir à arte.
    let s = crate::frame_text::render_frame();
    assert!(
        s.contains("warp_overlay::draw_warp_gizmo("),
        "o retrato publicado tem de chegar à tinta"
    );
    assert!(
        s.contains("warp_gizmo::view()"),
        "e o pintor lê o retrato publicado, nunca re-decide a tool"
    );
}

/// **AS TRÊS PONTAS DO PONTEIRO.**
///
/// ⚠️ As três, e não uma: sem o `down` a alça não agarra; sem o `move` ela agarra e não
/// segue; sem o `up` o arrasto nunca larga e o próximo clique continua a escrever no nó
/// anterior. Cada ausência é um defeito diferente, e nenhuma delas é visível num gate de
/// geometria.
#[test]
fn all_three_pointer_ends_are_wired() {
    let s = crate::input_text::dispatch();
    for needle in [
        "self.warp_gizmo_down(",
        "self.warp_gizmo_move(",
        "self.warp_gizmo_up()",
    ] {
        assert!(s.contains(needle), "a costura `{needle}` não está ligada");
    }
}

/// ⭐⭐ **O gizmo do COLISOR da forma (doc 109 §5) está ligado como o do warp** — publicado com a
/// modalidade da tool, desenhado a partir do retrato, as três pontas do ponteiro, e o `down` só
/// sobre o canvas. *A costura é o que se perde num merge, não a matemática.*
#[test]
fn the_collider_gizmo_is_wired_like_the_warp() {
    let quadro = crate::frame_text::render_frame()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        quadro.contains(
            "ph2d_app_motion::collider_gizmo::publish(ph2d_app_motion::collider_gizmo::resolve_at( motion, motion_tool_active,"
        ),
        "o retrato do colisor tem de ser publicado com a modalidade da tool Motion. \
         ⚠️ Ele deixou de morar no PROLOGO em 2026-09-18 (retratava o cozido do quadro \
         anterior) — hoje vive na fase_motion_gizmos, e a ORDEM e o gate irmao \
         o_gizmo_do_colisor_le_o_cozido_deste_quadro"
    );
    assert!(
        quadro.contains("collider_gizmo::view()")
            && quadro.contains("collider_gizmo_overlay::draw("),
        "e desenhado a partir do retrato publicado"
    );
    let s = crate::input_text::dispatch();
    for needle in [
        "self.collider_gizmo_down(",
        "self.collider_gizmo_move(",
        "self.collider_gizmo_up()",
    ] {
        assert!(s.contains(needle), "a costura `{needle}` não está ligada");
    }
    let at = s
        .find("self.collider_gizmo_down(")
        .expect("o colisor está lá");
    assert!(
        s[at.saturating_sub(400)..at].contains("&& on_canvas"),
        "o `down` do colisor tem de exigir `on_canvas`"
    );
}

/// ⭐⭐⭐ **O CONTORNO DO COLISOR É PINTADO DEPOIS DA ARTE DAS FORMAS** — o *z-index* que o dono
/// pediu (2026-09-13: *«o collider deve aparecer na frente da shape»*), lido na ORDEM do quadro.
///
/// ⚠️ **A régua é o quadro emendado, não a ordem dos ficheiros** — e foi ela que apanhou o defeito:
/// com o desenho na `fase_selection_highlight`, o gizmo era pintado em `484 970` e a arte em
/// `623 773`, ou seja **por baixo** dela. Hoje as duas chamadas vivem na `fase_vector_overlays`, a
/// tinta do gizmo logo a seguir à da arte; inverter isso é o defeito que este gate apanha, e nenhum
/// gate de geometria o vê.
#[test]
fn the_collider_outline_is_painted_after_the_shape_art() {
    let s = crate::frame_text::render_frame();
    let arte = s
        .find("motion_shape_gen::encode(")
        .expect("a arte das formas é codificada no quadro");
    // ⚠️ **O gizmo de WARP entra no mesmo gate**: ele tinha o MESMO defeito (report de 2026-09-08,
    // *«está sendo desenhado por trás das shapes»*), curado então com casing sobre uma nota —
    // *«o gizmo ESTÁ por cima»* — que esta régua mediu como falsa.
    for (nome, agulha) in [
        ("o contorno do colisor", "collider_gizmo_overlay::draw("),
        ("o gizmo de warp", "warp_overlay::draw_warp_gizmo("),
    ] {
        let gizmo = s
            .find(agulha)
            .unwrap_or_else(|| panic!("{nome} é desenhado no quadro"));
        assert!(
            arte < gizmo,
            "{nome} tem de ser pintado DEPOIS da arte (arte em {arte}, gizmo em {gizmo})"
        );
    }
}

/// **O `down` do warp vem ANTES do do field e do genérico.**
///
/// ⚠️ Uma alça alcançada pelo caminho genérico escreveria um `Transform` de ENTIDADE em
/// vez do param do nó — o mesmo defeito que fez o gizmo do Flip e o do field virem antes.
/// A ordem no arquivo É a precedência.
#[test]
fn the_warp_grab_is_tried_before_the_generic_gizmo() {
    let s = crate::input_text::dispatch();
    let warp = s.find("self.warp_gizmo_down(").expect("o warp está lá");
    let field = s.find("self.field_gizmo_down(").expect("o field está lá");
    assert!(
        warp < field,
        "o `down` do warp tem de ser tentado antes do do field"
    );
}

/// **A EDIÇÃO SAI PELA PORTA DO PAINEL, e não por uma segunda.**
///
/// ⚠️ O arrasto escreve `set_param` — a mesma função que o slider chama. Um segundo
/// caminho de escrita divergiria do commit, do undo e do que o painel mostra.
#[test]
fn the_drag_writes_through_the_same_port_the_panel_uses() {
    let s = src("warp_gizmo_drag.rs");
    assert!(
        s.contains("graph.set_param("),
        "a edição sai por `set_param`, a porta do painel"
    );
    // ⚠️ E o CONTROLE: nada aqui pode tocar num `Transform` de entidade. O gizmo é de um
    // NÓ; escrever no mundo ECS seria ele a mexer nos sprites que a outra ferramenta
    // manipula — a prova de isolamento que o `field_gizmo` documenta.
    assert!(
        !s.contains("Transform"),
        "o gizmo de um nó não pode escrever num Transform de entidade"
    );
}

/// **AS DUAS SUPERFÍCIES PROJECTAM PELA MESMA PORTA.**
///
/// ⚠️ **O defeito de 2026-08-23**, e a razão de esta linha existir: a tinta projectava com
/// a janela CHEIA e o hit-test com a da CENA, então a alça que se via não era a alça que
/// existia — desenho deslocado *e* clique a errar, dois sintomas de uma causa. A cura
/// principal é ESTRUTURAL (o overlay recebe o split e resolve a janela por dentro, então
/// o chamador não tem como errar); esta linha é a segunda cerca, para o dia em que alguém
/// reintroduzir um parâmetro de janela.
#[test]
fn paint_and_grab_project_through_the_same_door() {
    let overlay = fam("warp_overlay.rs");
    let drag = src("warp_gizmo_drag.rs");
    for (name, s) in [("overlay", &overlay), ("drag", &drag)] {
        assert!(
            s.contains("warp_gizmo::scene_window("),
            "o {name} tem de projectar pela porta única (`scene_window`)"
        );
    }
    // ⚠️ E nenhum dos dois pode voltar a chamar a derivação CRUA: ela devolve dimensões
    // soltas, e foi assim que a janela errada entrou.
    for (name, s) in [("overlay", &overlay), ("drag", &drag)] {
        assert!(
            !s.contains("scene_window_wh("),
            "o {name} não pode usar a derivação crua — a porta única existe para isso"
        );
    }
}

/// **O AGARRE SÓ VALE SOBRE O CANVAS.**
///
/// ⚠️ **O defeito (Enio, 2026-08-23):** *"se colocar transform antes, não é possível
/// conectar transform em Bezier Warp"*. Os gizmos irmãos consomem pelo HIT-INDEX
/// (`GizmoTarget::…`), que já sabe das regiões; este faz o seu PRÓPRIO hit-test em
/// coordenadas de mundo. Sem guarda, um clique **no painel do grafo** era convertido para
/// o mundo, calhava de cair sobre uma alça, e o `Down` era consumido — o gesto de ligar um
/// fio nunca começava. *Um consumidor que decide sozinho tem de saber sozinho onde ele
/// vale.*
///
/// A guarda é o `on_canvas` que o próprio arquivo já computa (nenhum painel e nenhum
/// widget sob o cursor), e não uma nova.
#[test]
fn the_grab_only_applies_over_the_canvas() {
    let s = crate::input_text::dispatch();
    let at = s.find("self.warp_gizmo_down(").expect("o warp está lá");
    // A condição do `if` que precede a chamada — 400 chars para trás cobrem-na de sobra.
    let window = &s[at.saturating_sub(400)..at];
    assert!(
        window.contains("&& on_canvas"),
        "o `down` do warp tem de exigir `on_canvas` — sem isso ele engole cliques do \
         painel do grafo e o artista não consegue ligar um fio"
    );
}

/// ⛔⛔⛔ **UM GIZMO LÊ O COZIDO DESTE QUADRO, NÃO O DO ANTERIOR** — report do dono (2026-09-18,
/// foto): *«melhorou em relação à colisão mas tem um atraso antigo do gizmo em relação à imagem»*.
///
/// O contorno azul do colisor e as alças do warp saem das TOMADAS (`pump.tap_streams()`), que só
/// existem depois de o Motion cozinhar. Medido no texto do quadro EMENDADO — onde a posição de um
/// literal é a ordem em que ele corre — antes da cura:
///
/// | literal | posição |
/// |---|---|
/// | `collider_gizmo::resolve_at(` (o retrato) | **190 082** |
/// | `motion_bridge::dispatch(` (o cook) | 482 136 |
/// | `collider_gizmo_overlay::draw(` (o desenho) | 662 982 |
///
/// ⇒ o retrato era do cozido de **N−1** e a arte é encodada no fim, com o de **N**. Um quadro
/// inteiro de atraso, visível só com a cena em movimento — que é a `=121`/`=122`, onde tudo treme.
///
/// ⚠️⚠️ **A LEITURA ESTRUTURAL ERROU DUAS VEZES E A MEDIÇÃO ACERTOU AS DUAS.** Pelos números de
/// linha, `fase_hero_scene` (linha 67) parecia correr antes de `fase_motion_bridge` (linha 369) ⇒
/// «o desenho também é cedo». É falso: `fase_hero_frame.rs` tem **três** fases, e a linha 369 vive
/// na `fase_hero_tools`, que a `fase_hero_frame` chama na linha **66** — uma antes da
/// `fase_hero_scene`. *Números de linha no mesmo ficheiro não são ordem de execução quando o
/// ficheiro tem mais de uma fase* — e é exactamente para isso que o `frame_text` existe.
///
/// ⇒ a cura move o RESOLVE (e não o desenho, que já estava no sítio) para a `fase_motion_gizmos`,
/// logo a seguir ao cook.
///
/// ⚠️ **Nenhum gate desta casa o via:** os que existem medem a COSTURA (o retrato é publicado? é
/// desenhado? o ponteiro chega?) e ficam todos verdes com as três chamadas na ordem errada. *Uma
/// costura ligada não diz nada sobre QUANDO cada ponta corre.*
#[test]
fn o_gizmo_do_colisor_le_o_cozido_deste_quadro() {
    let quadro = crate::frame_text::render_frame();
    let uma = |agulha: &str| {
        let n = quadro.matches(agulha).count();
        assert_eq!(
            n, 1,
            "`{agulha}` tem de aparecer UMA vez no quadro, e aparece {n}"
        );
        quadro.find(agulha).expect("acabou de ser contada")
    };
    let cook = uma("ph2d_app_motion::motion_bridge::dispatch(");
    // As DUAS famílias que leem uma tomada. ⭐ O gizmo do FIELD fica de fora de propósito e está
    // MEDIDO: nenhum ficheiro `field_gizmo*` menciona `tap_streams` — ele lê params do nó.
    for (quem, resolve, desenho) in [
        (
            "colisor",
            "ph2d_app_motion::collider_gizmo::resolve_at(",
            "ph2d_app_motion::collider_gizmo_overlay::draw(",
        ),
        (
            "warp",
            "ph2d_app_motion::warp_gizmo::resolve(",
            "ph2d_app_motion::warp_overlay::draw_warp_gizmo(",
        ),
        // ⭐⭐⭐ **A TERCEIRA: o gizmo de uma corrente de POSIÇÕES** (ordem do dono, 2026-09-17/19:
        // *«sem o duplicator só aparece um gizmo de osso ou segmento de corda»*). Ele lê a tomada
        // do sink, logo herda a MESMA lei — e entra aqui, e não num gate novo, porque a lei é a
        // mesma: *um retrato tirado antes do cook mostra o quadro anterior*.
        (
            "pontos",
            "ph2d_app_motion::ponto_gizmo::resolve(",
            "ph2d_app_motion::ponto_gizmo_overlay::draw(",
        ),
    ] {
        let (r, d) = (uma(resolve), uma(desenho));
        assert!(
            cook < r && r < d,
            "o gizmo do {quem} tem de ser resolvido DEPOIS do cook e desenhado depois disso — \
             cook={cook} resolve={r} desenho={d}"
        );
    }
}
