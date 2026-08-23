//! ⭐ **A MÁQUINA DE ESTADOS DO PONTEIRO** — o que um botão premido, arrastado e solto faz.
//!
//! ⚠️ **Separada dos métodos de `App` de propósito, e não por arrumação:** era a costura
//! ponteiro↔gizmo que ficava sem gate. A [`DIRETIVA_IMPLEMENTACAO`] §1 chama-lhe a causa nº 1 da
//! semana perdida no Painter — *"a alça está pintada, o arrasto está correto, e ninguém liga os
//! dois"* passa em todo teste de unidade dos dois lados. Aqui ela é uma função pura sobre o
//! [`Smoke`], que um gate encena sem janela nenhuma.
//!
//! ⚠️ **Módulo-filho de [`super`]**, cortado da `field3d_input` na W34 pelo teto de LOC. O corte é
//! por **assunto**: o pai possui a *câmera* (órbita, zoom, enquadramento, os métodos de `App`) e
//! este possui o *gesto* (pegar uma alça, arrastar, digitar um número, largar).
//!
//! [`DIRETIVA_IMPLEMENTACAO`]: ../../../docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md

use super::*;

/// ⭐ **O que o ponteiro FAZ**, sobre o estado do smoke e nada mais.
///
/// ⚠️ Separado dos métodos de `App` de propósito, e não por arrumação: era a costura ponteiro↔gizmo
/// que ficava sem gate. A `DIRETIVA_IMPLEMENTACAO` §1 chama-lhe a causa nº 1 da semana perdida no
/// Painter — *"a alça está pintada, o arrasto está correto, e ninguém liga os dois"* passa em todo
/// teste de unidade dos dois lados.
pub(crate) fn begin(
    s: &mut Smoke,
    button: winit::event::MouseButton,
    fallback: Drag,
    pos: (f32, f32),
) -> bool {
    // ⚠️ **Fora da área desenhada, o gesto não é meu.** O `Move` e o `Up` NÃO fazem esta pergunta,
    // de propósito: um arrasto em curso continua a ser do gesto que o abriu mesmo que o cursor
    // passeie por fora — a regra de captura que todo gizmo deste shell segue.
    let Some(area) = s.area else {
        return false;
    };
    if pos.0 < area.x || pos.1 < area.y || pos.0 >= area.x + area.w || pos.1 >= area.y + area.h {
        return false;
    }
    // ⭐⭐ **O GIZMO DE NAVEGAÇÃO ganha de tudo** (W49), e só com o botão esquerdo.
    //
    // ⚠️ Ele fica na quina, por cima da peça: sem esta pergunta, um clique numa bola seria um
    // arrasto na peça — e o widget inteiro nasceria pintado e morto.
    //
    // ⭐ **Arrastar a partir dele ORBITA**, e é o gesto principal, não um efeito colateral: a
    // pesquisa da referência mede os utilizadores *quase 2× mais rápidos* a arrastar do que a
    // clicar, «independentemente das representações examinadas» (ver `field3d_navball`). Por isso o
    // `drag` fica em `Orbit` e a bola é só **lembrada** — o `Up` sem movimento é que a usa.
    if button == winit::event::MouseButton::Left
        && let Some(p) = local(s, pos)
        && crate::field3d_navball::hits_widget(area, crate::field3d_smoke::safe_of(s), p)
    {
        let safe = crate::field3d_smoke::safe_of(s);
        s.nav_press =
            crate::field3d_navball::pick(&crate::field3d_navball::balls(&s.cam, area, safe), p);
        s.drag = Some(Drag::Orbit);
        s.drag_grip = None;
        s.gizmo_hot = None;
        s.last_pointer = pos;
        s.press_at = Some(pos);
        s.manual = true;
        return true;
    }
    // ⭐ **A alça ganha do gesto de câmera**, e só com o botão ESQUERDO: o direito continua a
    // orbitar mesmo por cima do gizmo, que é a saída para quem quer girar a vista sem primeiro
    // tirar o rato de cima da peça.
    let grabbed = (button == winit::event::MouseButton::Left)
        .then(|| local(s, pos).and_then(|p| field3d_gizmo::pick(&handles(s), p)))
        .flatten();
    s.drag = Some(grabbed.map_or(fallback, Drag::Gizmo));
    // ⭐ A pegada congela a âncora e o pixel: é contra eles que o total se mede até soltar.
    s.drag_grip = grabbed.and_then(|h| {
        let anchor = s.gizmo?;
        let from = local(s, pos)?;
        let screen = area_screen(s)?;
        Some(Grip {
            anchor,
            from,
            applied: field3d_gizmo::drag(h, anchor, &s.cam, screen, from, from).neutral(),
        })
    });
    s.gizmo_hot = grabbed;
    s.last_pointer = pos;
    // ⚠️ Guardado **antes** de qualquer movimento: é a origem contra a qual o `Up` decide se aquilo
    // foi um clique ou um arrasto.
    s.press_at = Some(pos);
    s.manual = true;
    true
}

/// ⚠️ **A que distância um clique deixa de ser um clique** — e o número é o da CASA
/// ([`ph2d_editor::interaction::NUMBER_INPUT_DRAG_THRESHOLD_PX`]).
///
/// Ele tem o nome do campo numérico porque foi lá que a casa o mediu primeiro, mas a grandeza é a
/// mesma pergunta física: *quanto a mão treme entre carregar e soltar*. Um quarto número para a
/// mesma pergunta seria a quarta resposta a envelhecer — já há três no shell.
const CLICK_SLOP_PX: f32 = ph2d_editor::interaction::NUMBER_INPUT_DRAG_THRESHOLD_PX;

/// O ponteiro moveu. Devolve `true` só quando o gesto é desta janela.
/// ⭐ **O que uma tecla numérica FAZ**, sobre o estado do smoke e nada mais — o irmão do [`advance`],
/// e separado dos métodos de `App` pela mesma razão: era a costura que ficava sem gate.
///
/// Devolve `true` quando a tecla foi consumida.
pub(crate) fn typed_key(s: &mut Smoke, stroke: crate::field3d_typed::Stroke) -> bool {
    use crate::field3d_typed as typed;
    // A entrada só existe **dentro de um arrasto de alça**, e só onde um número tem um significado.
    let Some(Drag::Gizmo(handle)) = s.drag else {
        return false;
    };
    if !typed::accepts(handle) {
        return false;
    }
    match stroke {
        // ⭐ **Cancelar desfaz o gesto INTEIRO**: o mundo recebe o inverso do que já lhe foi dado, e
        // a peça volta a onde estava quando a alça foi agarrada. ⚠️ É por isso que o inverso se
        // escreve com a própria álgebra (`neutral().since(applied)`) — uma segunda conta de «como se
        // desfaz um giro» divergiria da primeira no dia em que um verbo novo entrasse.
        typed::Stroke::Cancel => {
            if let Some(grip) = s.drag_grip {
                let back = grip.applied.neutral().since(grip.applied);
                if !back.is_idle() {
                    publish(s, grip.anchor.entity, back);
                }
            }
            s.drag = None;
            s.drag_grip = None;
            s.typed = None;
            s.press_at = None;
            true
        }
        // Fechar guardando o que está — o mesmo que largar o botão.
        typed::Stroke::Commit => {
            s.typed = None;
            finish(s);
            true
        }
        stroke => {
            // ⚠️ Uma entrada só **começa** com um dígito ou um ponto: um `Backspace` sem entrada
            // aberta não é deste módulo, e engoli-lo tiraria a tecla a quem quer que a espere.
            let open = s.typed.is_some();
            if !open && matches!(stroke, typed::Stroke::Backspace) {
                return false;
            }
            let before = s.typed.clone().unwrap_or_default();
            s.typed = typed::edit(&before, stroke);
            apply_typed(s, handle);
            true
        }
    }
}

/// Manda ao mundo o que o número digitado pede — **o total**, contra o que já foi aplicado.
fn apply_typed(s: &mut Smoke, handle: Handle) {
    let (Some(text), Some(grip)) = (s.typed.clone(), s.drag_grip) else {
        return;
    };
    let (_, _, fwd) = s.cam.basis();
    let Some(total) = crate::field3d_typed::value_of(&text)
        .and_then(|v| crate::field3d_typed::total(handle, &grip.anchor, fwd, v))
    else {
        return;
    };
    let delta = total.since(grip.applied);
    if !delta.is_idle() {
        publish(s, grip.anchor.entity, delta);
        if let Some(g) = s.drag_grip.as_mut() {
            g.applied = total;
        }
    }
}

/// O pedido que a ponte com a cena vai aplicar no início do quadro seguinte, acumulado.
///
/// ⚠️ **Um só sítio a escrever `pending_move`**: o ponteiro e o teclado mandam a mesma coisa pelo
/// mesmo cano, e duas cópias da acumulação divergiriam no dia em que os dois acontecessem no mesmo
/// quadro — que é exactamente o que digitar durante um arrasto é.
fn publish(s: &mut Smoke, entity: u64, delta: field3d_gizmo::Motion) {
    s.pending_move = Some((
        entity,
        s.pending_move
            .filter(|(e, _)| *e == entity)
            .map_or(delta, |(_, acc)| acc.merge(delta)),
    ));
}

pub(crate) fn advance(s: &mut Smoke, x: f32, y: f32) -> bool {
    let Some(drag) = s.drag else {
        // ⚠️ **Sem arrasto, o hover ainda é atualizado — e o evento NÃO é consumido.** As
        // duas metades importam: sem a primeira a alça nunca acende e o artista não sabe o
        // que vai agarrar; com a segunda invertida, a janela 3D engoliria todo movimento de
        // rato do app 2D.
        // ⭐ **O realce do gizmo de NAVEGAÇÃO** (W49), pela mesma lei e no mesmo sítio: sem ele o
        // artista não sabe que bola vai pegar — e o widget lê como decoração.
        s.nav_hot = match (s.area, local(s, (x, y))) {
            (Some(area), Some(p))
                if crate::field3d_navball::hits_widget(
                    area,
                    crate::field3d_smoke::safe_of(s),
                    p,
                ) =>
            {
                let safe = crate::field3d_smoke::safe_of(s);
                crate::field3d_navball::pick(&crate::field3d_navball::balls(&s.cam, area, safe), p)
            }
            _ => None,
        };
        // ⚠️ **Com o cursor no gizmo de navegação, a alça do gizmo 3D não acende.** Os dois
        // realces ao mesmo tempo diriam que um clique faz duas coisas.
        s.gizmo_hot = if s.nav_hot.is_some() {
            None
        } else {
            local(s, (x, y)).and_then(|p| field3d_gizmo::pick(&handles(s), p))
        };
        return false;
    };
    let (dx, dy) = (x - s.last_pointer.0, y - s.last_pointer.1);
    s.last_pointer = (x, y);
    match drag {
        // ⚠️ **Manipulação direta: o modelo segue a mão.** Os sinais são os que a
        // `line/sculpt3d` já pagou para descobrir, e o gate que os prende aqui mede **o
        // modelo na tela**, nunca o sinal: foi argumentando sobre sinais que o erro entrou
        // lá.
        Drag::Orbit => law::orbit(&mut s.cam, dx, dy),
        // O alvo anda ao CONTRÁRIO da mão: mover o ponto olhado para a esquerda é o que faz
        // o modelo aparecer mais à direita.
        //
        // ⚠️ O passo é em **fração do lado menor do quadro**, vezes `half_extent` — é isso
        // que faz arrastar o mesmo tanto de tela mover o mesmo tanto de modelo em qualquer
        // zoom. Um passo em unidades de mundo fixas ficaria absurdo assim que se aproxima.
        Drag::Pan => {
            let Some(area) = s.area else {
                return true;
            };
            law::pan(&mut s.cam, dx, dy, area.w.min(area.h) * 0.5);
        }
        // ⭐ O arrasto do gizmo **não escreve na peça aqui**: ele acumula um PEDIDO que a ponte
        // com a cena aplica no início do quadro seguinte. É o mesmo caminho dos intents do
        // painel, e pela mesma razão — o mundo tem um só escritor.
        Drag::Gizmo(handle) => {
            // ⭐ **Com um número em cima da mesa, o rato CEDE** (W26). Sem esta linha o movimento
            // seguinte do ponteiro sobrescreveria o que acabou de ser digitado, e o defeito leria
            // como *"digitar não faz nada"* — porque o dedo nunca está completamente parado.
            if s.typed.is_some() {
                return true;
            }
            let (Some(grip), Some(screen), Some(area)) = (s.drag_grip, area_screen(s), s.area)
            else {
                return true;
            };
            // ⭐ **O TOTAL desde a pegada**, contra a âncora congelada — nunca um incremento contra
            // a pose de agora, que é o que este gesto está a mudar.
            let total = field3d_gizmo::drag(
                handle,
                grip.anchor,
                &s.cam,
                screen,
                grip.from,
                [x - area.x, y - area.y],
            );
            let total = if s.snapping {
                total.snapped(field3d_gizmo::snap_step(screen))
            } else {
                total
            };
            // O que falta aplicar. ⚠️ Um pedido inerte **não é guardado**: uma alça degenerada
            // devolve zero, e escrever esse zero acordaria o traçado para redesenhar o mesmo quadro.
            let delta = total.since(grip.applied);
            if !delta.is_idle() {
                s.pending_move = Some((
                    grip.anchor.entity,
                    s.pending_move
                        .filter(|(e, _)| *e == grip.anchor.entity)
                        .map_or(delta, |(_, acc)| acc.merge(delta)),
                ));
                if let Some(g) = s.drag_grip.as_mut() {
                    g.applied = total;
                }
            }
        }
    }
    true
}

/// ⭐ **O ponteiro está sobre a janela 3D?** — a guarda de TODA tecla deste módulo.
///
/// ⚠️ **É a diferença entre um atalho e um sequestro.** Enquanto o módulo entrava só por variável de
/// ambiente, perguntar *"o smoke está armado?"* bastava; com o pill do topo ele pode estar ligado
/// numa sessão normal, e aí uma tecla engolida é uma tecla que não chega ao campo de texto onde o
/// artista está a escrever.
///
/// ⚠️ **Vive num sítio de propósito.** Ela estava escrita à mão em cada porta de tecla, com a nota
/// só numa delas — e a tecla seguinte a nascer teria copiado a condição e deixado a nota para trás.
/// É exatamente o que a `line/sculpt3d` viu envelhecer: a porta dela perguntava *"a cena existe?"*,
/// o dia em que a cena passou a nascer sozinha chegou, e a partir dali ela comia as teclas de todo
/// painel do app. *Quem move o número que tornava uma nota verdadeira tem de reconferir a nota.*
pub(super) fn over_window(s: &Smoke, pos: (f32, f32)) -> bool {
    s.area
        .is_some_and(|a| pos.0 >= a.x && pos.1 >= a.y && pos.0 < a.x + a.w && pos.1 < a.y + a.h)
}

/// **Que verbo esta tecla nomeia** — `None` quando ela não é deste módulo.
///
/// ⚠️ **Com MODIFICADOR não é atalho de gizmo, e esta linha é a diferença entre um atalho e um
/// sequestro do `Ctrl+S`.** A guarda de *"ponteiro sobre a janela 3D"* protege os campos de texto;
/// ela não protege os atalhos GLOBAIS, que valem em qualquer sítio da janela — e `Ctrl+S` é o
/// salvar do app. Sem isto, guardar o projeto com o rato em cima da peça trocava o gizmo para
/// *Size* e **não salvava nada**, em silêncio.
///
/// ⚠️ O `Shift` fica de fora da proibição de propósito: `Shift+S` continua a ser um `S`, e nenhum
/// atalho global da casa o usa.
pub(super) fn mode_for_key(
    code: winit::keyboard::KeyCode,
    modifiers: winit::keyboard::ModifiersState,
) -> Option<field3d_gizmo::Mode> {
    use winit::keyboard::KeyCode;
    if modifiers.control_key() || modifiers.alt_key() || modifiers.super_key() {
        return None;
    }
    match code {
        KeyCode::KeyG => Some(field3d_gizmo::Mode::Move),
        KeyCode::KeyR => Some(field3d_gizmo::Mode::Rotate),
        KeyCode::KeyS => Some(field3d_gizmo::Mode::Scale),
        _ => None,
    }
}

/// O ponteiro subiu: fecha o gesto e decide se ele foi um **clique**.
///
/// Devolve `(o gesto era meu?, ele AUTOROU a cena?)`.
///
/// ⭐ **Soltar sem ter arrastado é um clique**, e um clique na janela 3D seleciona o objeto sob o
/// cursor — como em todo modelador. ⚠️ Só o gesto de **câmera** vira clique: soltar uma alça do
/// gizmo nunca é uma seleção, senão mover um objeto trocaria a seleção para o que estivesse por
/// baixo dele no fim do gesto, e o artista perderia o que acabou de posicionar.
pub(crate) fn finish(s: &mut Smoke) -> (bool, bool) {
    let was = s.drag.take();
    s.drag_grip = None;
    // A entrada numérica é do GESTO: ela morre com ele, senão o gesto seguinte abriria já a meio de
    // um número que ninguém digitou.
    s.typed = None;
    // ⭐⭐ **UM CLIQUE NUMA BOLA É UMA ESCOLHA DE VISTA** (W49) — e ganha do `pending_pick`, que
    // seria um clique na PEÇA por baixo do widget.
    let nav = s.nav_press.take();
    let still = s.press_at.is_some_and(|from| {
        (s.last_pointer.0 - from.0).abs() <= CLICK_SLOP_PX
            && (s.last_pointer.1 - from.1).abs() <= CLICK_SLOP_PX
    });
    if let Some(view) = nav.filter(|_| still) {
        s.cam.rotation = view.rotation();
        crate::field3d_input::frame_the_part(s);
    } else if was == Some(Drag::Orbit)
        && let (Some(from), Some(area)) = (s.press_at, s.area)
        && still
    {
        // ⚠️ **Não há aqui um `nav.is_none()`, e havia** — ele era **código morto**, e uma prova de
        // mutação foi quem o disse: para chegar a este ramo é preciso `nav.filter(still)` ser
        // `None`, isto é `nav.is_none() || !still`; e o `still` exigido aqui colapsa isso em
        // `nav.is_none()`. *Uma condição que não pode mudar o resultado é uma afirmação falsa sobre
        // o código para quem o ler a seguir* — e ela sobrevive a toda mutação, de propósito.

        s.pending_pick = Some([from.0 - area.x, from.1 - area.y]);
    }
    s.press_at = None;
    (was.is_some(), matches!(was, Some(Drag::Gizmo(_))))
}

/// A mesma porta, aberta para o gate da costura — o caminho real (`App::field3d_pointer_up`) exige
/// um `App`, que um teste não constrói.
#[cfg(test)]
pub(crate) fn finish_for_test(s: &mut Smoke) -> (bool, bool) {
    finish(s)
}

/// A alça que o gizmo tem de pintar realçada: a **agarrada** ganha da que está sob o cursor.
///
/// ⚠️ Não é detalhe: durante um arrasto o cursor sai de cima da alça — é isso que arrastar É —, e
/// sem esta precedência o realce apagava-se no instante exato em que o gesto começa a valer.
pub(crate) fn hot_handle(s: &Smoke) -> Option<Handle> {
    match s.drag {
        Some(Drag::Gizmo(h)) => Some(h),
        _ => s.gizmo_hot,
    }
}
