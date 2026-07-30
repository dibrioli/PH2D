//! **As alças de uma RODLANA chegam ao barramento** (W-Pulley W6) — arch-gates
//! sobre as costuras window-gated que um unit test não dirige.
//!
//! O que É testável por comportamento já está: `ph2d_editor::gizmo::point` prova
//! que as três alças desenham e registram hit, `render_loop::point_gizmo` prova a
//! regra de publicação (o aro de saída só existe quando há um segundo raio), e
//! `ph2d-physics-ecs::pulley_mount` prova o que uma re-colocação de eixo FAZ. O
//! que nenhum deles alcança é a GLUE — o `advance_joint_anchor_drag` precisa de um
//! `AppGfx` vivo (janela + GPU), e o commit de Position mora dentro do
//! `render_loop`. Estas são as duas metades restantes da lista de UI, como
//! arch-gates sobre o fonte.

use std::fs;

/// A metade do gesto que é da RODLANA — módulo filho desde o split do W6, e é
/// nele que estas afirmações moram agora.
fn wheel_module() -> String {
    fs::read_to_string("src/joint_anchor_drag_wheel.rs").expect("joint_anchor_drag_wheel.rs")
}

/// **Mover o eixo de uma roldana MONTADA passa pela porta de re-colocação.**
///
/// Sem isso o gesto é **morto e silencioso**: o centro de uma montada é derivado
/// (`corpo · local`) e o `sync_mounted_wheels` o reescreve no frame seguinte — a
/// alça anda com o dedo e volta ao soltar, sem erro e sem aviso. O `Transform`
/// sozinho não basta, e é exatamente essa a metade que faltava.
#[test]
fn dragging_a_wheel_centre_reseats_a_mounted_axle() {
    let arm = wheel_module();
    assert!(
        arm.contains("t.translation = ph2d_core::Vec2::new(to[0], to[1])"),
        "o arrasto tem de escrever o Transform da roldana"
    );
    assert!(
        arm.contains("ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), wheel)"),
        "…e tem de desarmar o sentinela pela porta compartilhada, senão numa \
         roldana MONTADA o próximo reconcile reescreve o número e o gesto some"
    );
}

/// **A row Position é o SEGUNDO gesto que re-coloca o mesmo eixo**, e passa pela
/// MESMA porta.
///
/// Dois gestos que dizem a mesma coisa não podem discordar sobre o que ela
/// significa — e uma regra escrita uma vez por chamador é a regra que o terceiro
/// chamador nasce sem. O sítio do joint (`set_joint_anchor_world`) está
/// imediatamente acima, e o da roldana tem de estar no mesmo bloco.
#[test]
fn committing_a_wheel_position_reseats_the_same_axle() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let start = src
        .find("if let Some((bits, world)) = joint_pivot_commit {")
        .expect("o bloco de commit de pivô sumiu");
    let block = &src[start..start + 1400];
    assert!(
        block.contains("reseat_mounted_axle"),
        "o commit de Position numa roldana montada tem de re-colocar o eixo pela \
         mesma porta do dot de canvas"
    );
}

/// **As duas alças de raio perguntam a MESMA porta** — no agarre e na escrita.
///
/// O modo de falha que isto barra não é um crash: é agarrar o aro de SAÍDA,
/// medir o deslocamento contra o raio de ENTRADA e escrever o resultado no de
/// saída — a alça salta ao pegar e o número sai errado por uma diferença de
/// raios. Enumerar o par à mão nos dois lados é como o terceiro raio nasceria
/// mexendo em metade dos sítios.
#[test]
fn both_radius_handles_ask_one_door() {
    let parent = fs::read_to_string("src/joint_anchor_drag.rs").expect("joint_anchor_drag.rs");
    let wheel = wheel_module();
    assert!(
        parent.contains("wheel::open_grab(sim, joint, kind, cursor)")
            && parent.contains("wheel::resize_wheel(&mut gfx.sim, entity, drag.kind, cursor, off)"),
        "o agarre e o apply do raio têm de ir para a MESMA metade-roldana"
    );
    assert!(
        wheel.contains("if kind.is_wheel_radius() {"),
        "o agarre tem de perguntar a porta, não enumerar os dois kinds"
    );
    assert!(
        wheel.contains("wheel_radius_of(&w, kind)"),
        "o agarre tem de medir contra o raio DESTA alça"
    );
    // E o piso é UM: os dois raios passam pelo mesmo `max`, então o de saída não
    // pode chegar a zero — zero é o sentinela de *roldana comum*, e o arrasto
    // apagaria a própria alça sob o dedo.
    assert!(
        wheel.contains("let r = (distance(centre, cursor) + off).max(ph2d_physics_ecs::PulleyWheel::MIN_RADIUS);"),
        "o aro de saída tem de ter o mesmo piso do de entrada"
    );
}

/// **Autorar o RAIO pela §13 re-abre o comprimento da corda** — a segunda porta
/// do mesmo fato (2026-07-29).
///
/// O arrasto da alça já passa por `reseat_wheel_geometry` (gate acima). A row
/// **Radius** da §13 escreve o componente por uma FILA de comandos, então a
/// re-abertura tem de acontecer **depois do flush** — e o `apply_wheel_edit`
/// devolve *"a rota mudou?"* justamente para o chamador saber quando.
///
/// ⚠️ **Não é *"o componente mudou"*, e a diferença é load-bearing:** a cena `=59`
/// autora o **Motor** com o relógio ANDANDO de propósito, e re-derivar o `L0` ali
/// prenderia a corda na configuração do instante — um teleporte da restrição no
/// meio do movimento. Por isso a pergunta é sobre a ROTA (raio, 2º raio, ordem,
/// lado do abraço, o corpo do eixo), nunca sobre a igualdade do componente.
///
/// Mutação: apagar o `if route_changed` ⇒ o defeito volta pela row (o arrasto
/// segue curado, que é exactamente como este buraco nasceria de novo).
#[test]
fn authoring_the_radius_through_the_panel_reopens_the_rope_length() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let at = src
        .find("for &(bits, edit) in &wheel_edits {")
        .expect("o laço da §13 sumiu");
    // A janela é o BLOCO do laço, achado pelo `}` que o fecha — nunca uma
    // distância em bytes, que é o proxy que já expirou duas vezes nesta linha.
    let block = &src[at..];
    let end = block
        .find("if join_draw_arm")
        .expect("o laço da §13 é seguido pelo gesto de desenho de joint");
    let block = &block[..end];
    // ⚠️ **A FORMA da guarda, não só a presença da chamada.** A 1ª versão deste
    // gate afirmava que o bloco *contém* `reseat_wheel_geometry`, e a mutação
    // `if false && route_changed` **passou** — a mesma lição que o
    // `wheel_radius_of` pagou nesta linha: *um gate que pina a CHAMADA não pina a
    // RESPOSTA*. A guarda é o `route_changed` e nada mais.
    assert!(
        block.contains("if route_changed {"),
        "a re-abertura tem de ser guardada por `if route_changed {{` — qualquer coisa \
         entre o `if` e o nome é a guarda sendo neutralizada"
    );
    assert!(
        block.contains("let route_changed ="),
        "o chamador tem de CAPTURAR a resposta de `apply_wheel_edit` — sem ela não \
         há como saber que a rota mudou"
    );
    assert!(
        block.contains("reseat_wheel_geometry"),
        "…e re-abrir o comprimento da corda, senão digitar um raio maior deixa a \
         restrição violada e o solver a come num salto"
    );
    let flush = block
        .find("apply_editor_commands")
        .expect("o laço da §13 dá flush por edição");
    let reseat = block
        .find("reseat_wheel_geometry")
        .expect("já afirmado acima");
    assert!(
        reseat > flush,
        "a re-abertura tem de vir DEPOIS do flush: antes dele o componente novo \
         ainda não está no mundo, e o reconcile semearia o L0 da geometria VELHA"
    );
}

/// **O eixo de uma roldana montada tem ÍMÃ, e ele escreve o ponto COLADO** — o
/// item que o W6 deixou aberto (2026-07-29).
///
/// A isenção que morava no `Grab::World` dizia, com o porquê: *"o ímã cola a alça
/// nos pontos do COLLIDER do corpo daquela ponta, e uma roldana não pertence a
/// corpo nenhum — não há a que colar"*. Era verdade quando foi escrita; **o W3 a
/// falsificou** ao dar corpo à roldana, e a frase virou uma condição que enumerava
/// os donos de collider da época.
///
/// ⚠️ **A segunda asserção é a que carrega o peso.** Pedir só a CHAMADA de
/// `wheel_snap_targets` deixa passar o modo de falha real — computar o candidato e
/// escrever `free` de qualquer jeito —, que na tela é indistinguível de não haver
/// ímã: a marca do encaixe acende e a roda pousa fora dele. O que se afirma é
/// **qual valor chega ao apply**.
///
/// A recusa da roldana de CENÁRIO segue viva e é gateada do outro lado da
/// fronteira (`ph2d-physics-ecs::pulley_wheel_snap`): a porta devolve zero e o
/// `nearest_within` de uma fatia vazia é `None` — aritmética, não um ramo que
/// alguém tem de lembrar de atualizar.
#[test]
fn the_mounted_axle_snaps_and_the_snapped_point_is_what_lands() {
    let src = fs::read_to_string("src/joint_anchor_drag.rs").expect("joint_anchor_drag.rs");
    // A janela é o braço `Grab::World`, delimitado pelo braço seguinte — nunca
    // uma distância em bytes, o proxy que já expirou duas vezes neste repo.
    let at = src
        .find("Grab::World(off) => {")
        .expect("o braço de agarre no MUNDO sumiu");
    let arm = &src[at..];
    let end = arm
        .find("Grab::Angle(off) => {")
        .expect("o braço do mundo é seguido pelo do ângulo");
    let arm = &arm[..end];

    // Controle positivo: se o scanner não achar as duas rotas, ele não pode
    // falhar pelo motivo que alega.
    assert!(
        arm.contains("joint_snap_targets"),
        "controle positivo: o braço do mundo tem de conter a rota do JOINT — sem \
         ela este gate está lendo o lugar errado"
    );
    assert!(
        arm.contains("wheel_snap_targets(&gfx.sim, entity, &mut cands)"),
        "uma roldana MONTADA tem nove pontos a que colar (o W3 lhe deu um corpo); \
         o braço do mundo tem de perguntar a porta DELA"
    );
    assert!(
        arm.contains("wheel::move_wheel(&mut gfx.sim, entity, target)"),
        "o apply da roldana tem de receber o ponto COLADO (`target`); escrever \
         `free` acende a marca do encaixe e pousa a roda fora dele"
    );
    // E o ímã só arma com o modificador — o mesmo Ctrl do resto do editor.
    assert!(
        arm.contains("let n = if !ctrl {"),
        "o ímã tem de ser gateado no modificador, senão ele deixa de ser opcional"
    );
}

/// **O eyedropper de CORDA arma um pick MODAL, e o alvo dele é a ROTA** (W1).
///
/// A costura mora dentro do `input_dispatch`, que exige janela — nenhum unit test a
/// dirige, e as duas metades testáveis já estão (`seam_wheel` prova que o botão
/// arma; `inspector_joint_wheel::rope_pick_tests` prova o que a escrita FAZ).
///
/// ⚠️ **A asserção que carrega o peso é `rope_at_world` e não `pick_sprites_at_world`.**
/// Copiar o gesto do CORPO é o erro natural aqui — os três picks são irmãos no
/// gesto — e ele resolveria `None` sobre a corda **para sempre**: uma corda é uma
/// LINHA e a entidade dela não tem sprite. Um botão que arma e nunca acerta é pior
/// que um botão que falta, e nada na tela diria por quê.
///
/// ⚠️ **E a ordem é load-bearing:** o guard tem de PRECEDER o picking/gizmo, senão o
/// clique é consumido pela seleção e o pick fica armado para sempre.
#[test]
fn the_rope_eyedropper_arms_a_modal_pick_against_the_route() {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    // Controle positivo: o irmão de MONTAGEM tem de estar aqui, senão este gate
    // está lendo o arquivo errado e não pode falhar pelo motivo que alega.
    assert!(
        src.contains("self.wheel_body_pick.is_some()"),
        "controle positivo: o guard do pick de montagem sumiu deste arquivo"
    );
    // ⚠️ **A FORMA do guard, não só a presença do nome.** `if false && self.…` deixa
    // o literal no arquivo e neutraliza o guard — a lição que o gate irmão do
    // `route_changed` pagou nesta mesma suíte: *um gate que pina a CHAMADA não pina
    // a RESPOSTA*. O que se afirma é o `if` colado no nome.
    assert!(
        src.contains("if self.wheel_rope_pick.is_some()"),
        "o pick de corda tem de ter guard próprio no Down, e a guarda é o \
         `if self.wheel_rope_pick.is_some()` — qualquer coisa entre o `if` e o nome \
         é a guarda sendo neutralizada; sem ela o clique cai na seleção e o \
         eyedropper fica armado para sempre"
    );
    assert!(
        src.contains("gfx.physics.rope_at_world(world_pos, tol)"),
        "o alvo tem de ser a ROTA: `pick_sprites_at_world` exige um sprite, e a \
         entidade-corda não tem nenhum"
    );
    assert!(
        !src.contains("fn wheel_rope_pick_click")
            || src[src.find("fn wheel_rope_pick_click").expect("achado")..]
                .lines()
                .take(40)
                .all(|l| !l.contains("pick_sprites_at_world")),
        "o handler de corda não pode resolver por SPRITE — seria um pick que nunca \
         acerta"
    );
    // A ordem: o guard de corda vem antes do picking genérico de canvas.
    let rope_guard = src
        .find("self.wheel_rope_pick.is_some()")
        .expect("já afirmado");
    let generic = src
        .find("fn canvas_pick")
        .or_else(|| src.find("self.begin_selection"))
        .or_else(|| src.find("pick_sprites_at_world(gfx.present.world_mut(), world_pos)\n            .into_iter()\n            .find(|&bits| {\n                bits != joint"));
    if let Some(generic) = generic {
        assert!(
            rope_guard < generic || generic < src.find("fn wheel_rope_pick_click").expect("achado"),
            "o guard do pick de corda tem de preceder o picking genérico"
        );
    }
    // E a tolerância é a MESMA do resto do editor, lida do único lugar onde ela mora.
    assert!(
        src.contains("crate::joint_anchor_drag::SNAP_PX"),
        "a tolerância tem de vir do `SNAP_PX` compartilhado; um segundo número faria \
         dois alvos de canvas responderem a distâncias diferentes"
    );
}
