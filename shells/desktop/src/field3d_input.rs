//! **A navegação da janela 3D de modelagem** — órbita, pan e zoom ([ADR-0161], W4).
//!
//! # ⚠️ Mora no SHELL, e isso é o que mantém o contrato de ferramentas fora do caminho
//!
//! [ADR-0150] já resolveu esta pergunta para a escultura: *a navegação orbital mora no SHELL, nunca
//! numa `Tool`*. É o que impede que uma janela 3D obrigue a mexer no `Tool=12` — que está
//! **congelado** (`CLAUDE.md §6`) e cuja alteração exigiria ADR e Coordenador. Este arquivo é o
//! gémeo do [`crate::sculpt3d_input`] para o outro módulo 3D, com a mesma forma e os mesmos botões.
//!
//! # A superfície partilhada é de QUATRO linhas
//!
//! O `input_dispatch.rs` é o arquivo onde duas linhas paralelas mais se encontram. Por isso o que
//! entra lá são **só as chamadas** — quatro `if` que devolvem `false` quando o smoke não está
//! armado —, e toda a decisão mora aqui. É a forma que a `line/sculpt3d` já usa, e segui-la faz os
//! dois diffs ficarem **adjacentes** em vez de sobrepostos.
//!
//! # As leis do gesto são as MESMAS da escultura
//!
//! Esquerdo e direito orbitam, o do meio faz pan, a roda aproxima — e as constantes são as de lá
//! (`ORBIT_RAD_PER_PX = 0,01`, `1,1` por passo de roda). ⚠️ Não é herança por analogia (o
//! [`project-memory`] avisa contra isso): é que são **duas janelas 3D no mesmo aplicativo**, e uma
//! mão que aprendeu a girar numa tem de girar na outra. Divergir aqui seria uma decisão, e não há
//! nenhuma a tomar.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0150]: ../../../docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md
//! [`project-memory`]: ../../../project-memory/feedback_inherited_affordance_must_be_rederived.md

use crate::app_state::App;
use crate::field3d_gizmo::{self, Handle};
use crate::field3d_smoke::{Drag, Smoke, with_smoke};
use ph2d_field_render::Screen;

/// **As alças do gizmo, projetadas para o enquadramento deste quadro** — ou vazio quando não há
/// nada selecionado.
///
/// ⚠️ Recalculadas a cada pergunta, e não guardadas: a câmera muda a cada quadro do prato giratório,
/// e uma lista de alças em cache seria a resposta de um enquadramento anterior — o mesmo congelador
/// que o traçado já pagou, só que a apontar em vez de a desenhar.
fn handles(s: &Smoke) -> Vec<field3d_gizmo::Projected> {
    let (Some(anchor), Some(area)) = (s.gizmo, s.area) else {
        return Vec::new();
    };
    let screen = Screen::new(
        area.w.round().max(1.0) as u32,
        area.h.round().max(1.0) as u32,
        s.cam.half_extent,
    );
    field3d_gizmo::project(anchor, &s.cam, screen, s.gizmo_mode)
}

/// O ponto do cursor no referencial da **área desenhada** — que é o referencial em que o gizmo foi
/// projetado. ⚠️ Esquecer esta subtração faz as alças agarrarem deslocadas do tamanho da moldura do
/// app, e o defeito só aparece quando a janela 3D não começa em (0, 0).
fn local(s: &Smoke, p: (f32, f32)) -> Option<[f32; 2]> {
    let area = s.area?;
    Some([p.0 - area.x, p.1 - area.y])
}

/// Radianos de órbita por pixel de arrasto — **o mesmo número do módulo de escultura**
/// (`sculpt3d_rulers::ORBIT_RAD_PER_PX`).
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// Fator de zoom por passo de roda — **a mesma lei do `dolly`** da câmera da casa
/// (`ph2d_mesh_render::camera`): multiplicativa, para que cada passo aproxime a mesma **fração**.
const ZOOM_PER_STEP: f32 = 1.1;

/// O quanto se pode aproximar, e o recurso de que este limite é: a **precisão da representação**.
///
/// O campo é avaliado em `f32`, o que dá ~10⁻⁷ de erro absoluto à escala de uma peça unitária.
/// Quando um pixel passa a medir menos do que isso em unidades de mundo, a imagem deixa de ser
/// forma e passa a ser ruído de arredondamento. Com um quadro de ~1000 px, o pixel vale
/// `2·half_extent/1000`, e `10⁻⁴` põe-no em 2·10⁻⁷ — no fio.
///
/// ⚠️ São ~8000× de aproximação a partir do enquadramento inicial (0,8). O que costumava limitar o
/// zoom era a tolerância de acerto, e essa **deixou de ser um limite**: ela desce com o pixel
/// (`ph2d_field_render::Sharpness`). *Este é o único piso que sobrou, e ele nomeia o seu recurso.*
const MIN_HALF_EXTENT: f32 = 1.0e-4;

/// O quanto se pode afastar, e o recurso: o **alcance da marcha**.
///
/// Os raios partem a 4 unidades do alvo e andam 8, logo cobrem uma profundidade de ±4 em torno do
/// plano do alvo. Enquadrar mais largo do que isso mostraria uma cena que os raios não alcançam.
const MAX_HALF_EXTENT: f32 = 4.0;

/// O enquadramento a que a tecla de repor volta — o mesmo com que o módulo abre.
///
/// ⚠️ **Existe por causa da rotação livre.** Uma câmera de dois ângulos nunca inclina o horizonte;
/// esta inclina, porque é isso que *livre* significa. Sem uma volta a um enquadramento nomeado, o
/// preço da liberdade seria ficar perdido — e é o tipo de armadilha que só se nota depois de
/// entregue.
const HOME_YAW_PITCH: (f32, f32) = (0.72, 0.52);

/// **As três leis da câmera, puras** — sem `App`, sem ponteiro, sem estado do smoke.
///
/// ⭐ A separação não é estética: é o que torna as leis **testáveis pela porta do produto**. Um gate
/// que precisasse de um `App` inteiro para perguntar *"arrastar para a direita vira o modelo para a
/// direita?"* não seria escrito — e foi exatamente essa pergunta que a `line/sculpt3d` respondeu
/// errado nos dois sinais até um smoke a pegar. Aqui o gate traça a peça e **mede-a na tela**.
pub(crate) mod law {
    use ph2d_field_render::Orbit;

    use super::{
        HOME_YAW_PITCH, MAX_HALF_EXTENT, MIN_HALF_EXTENT, ORBIT_RAD_PER_PX, ZOOM_PER_STEP,
    };

    /// ⭐ **Rotação LIVRE** por um arrasto de `(dx, dy)` pixels.
    ///
    /// O arrasto nomeia um eixo **na tela**, e a peça gira em torno dele: o eixo é perpendicular ao
    /// movimento, no plano da imagem. Um arrasto horizontal cai no eixo vertical da câmera, um
    /// vertical cai no horizontal — e qualquer diagonal cai onde tem de cair, que é a metade que
    /// uma câmera de dois ângulos não consegue exprimir.
    ///
    /// ⚠️ **Nenhum eixo do MUNDO entra nesta conta**, e é daí que vem a ausência de polo. A câmera
    /// antiga girava `yaw` em torno do Y do mundo, e era esse Y que criava a parede a ±90°.
    ///
    /// O sinal é o da manipulação direta — *o modelo segue a mão* — e quem o prende é um gate que
    /// mede a peça **na tela**.
    pub(crate) fn orbit(cam: &mut Orbit, dx: f32, dy: f32) {
        let angle = dx.hypot(dy) * ORBIT_RAD_PER_PX;
        if angle <= 0.0 {
            return;
        }
        cam.turn_local([-dy, -dx, 0.0], angle);
    }

    /// Repõe a orientação e o enquadramento, mantendo o alvo onde está.
    pub(crate) fn home(cam: &mut Orbit) {
        let fresh = Orbit::from_yaw_pitch(HOME_YAW_PITCH.0, HOME_YAW_PITCH.1);
        cam.rotation = fresh.rotation;
        cam.half_extent = fresh.half_extent;
        cam.target = [0.0; 3];
    }

    /// Pan por um arrasto de `(dx, dy)` pixels, num quadro cujo lado menor mede `half_px` de meia
    /// altura.
    pub(crate) fn pan(cam: &mut Orbit, dx: f32, dy: f32, half_px: f32) {
        let k = cam.half_extent / half_px.max(1.0);
        let (right, up, _) = cam.basis();
        for i in 0..3 {
            cam.target[i] += -right[i] * dx * k + up[i] * dy * k;
        }
    }

    /// Zoom por `steps` linhas de roda.
    pub(crate) fn zoom(cam: &mut Orbit, steps: f32) {
        cam.half_extent =
            (cam.half_extent / ZOOM_PER_STEP.powf(steps)).clamp(MIN_HALF_EXTENT, MAX_HALF_EXTENT);
    }
}

impl App {
    /// O ponteiro desceu. Devolve `true` se a janela 3D tomou o gesto.
    pub(crate) fn field3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        // ⚠️ **A moldura do app não é da cena.** A mesma lei do `sculpt3d_pointer_down`, e pelo
        // mesmo motivo medido lá: painel é só uma espécie de UI — a faixa do topo e o rail não
        // publicam `panel_rect`, e sem esta pergunta a cena engoliria o clique em todo pill do topo.
        if crate::forwarding::cursor_over_hero_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        let fallback = match button {
            winit::event::MouseButton::Left | winit::event::MouseButton::Right => Drag::Orbit,
            winit::event::MouseButton::Middle => Drag::Pan,
            _ => return false,
        };
        with_smoke(|s| begin(s, button, fallback, pos)).unwrap_or(false)
    }

    /// O ponteiro moveu. **Só consome com um arrasto em curso** — senão a janela 3D engoliria todo
    /// hover do app 2D.
    pub(crate) fn field3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let (took, authored) = with_smoke(|s| {
            let took = advance(s, x, y);
            (took, took && matches!(s.drag, Some(Drag::Gizmo(_))))
        })
        .unwrap_or((false, false));
        // ⚠️ **O gesto AUTORA a cena, então o diff de undo tem de ver o quadro em que a pose
        // mudou** — a mesma razão do `advance_body_fk`, e a mesma frase. Este gancho consome o
        // `Down` e volta **antes** da linha que escreve o `held_button` do shell, então nem a
        // marca de entrada nem a de gesto-em-curso chegam aqui sozinhas.
        self.any_input_this_frame |= authored;
        took
    }

    /// O ponteiro subiu. Fecha o arrasto, se havia um.
    pub(crate) fn field3d_pointer_up(&mut self) -> bool {
        let (took, authored) = with_smoke(finish).unwrap_or((false, false));
        // ⚠️ **E no SOLTAR também**, senão um arrasto cujo último movimento caiu num quadro
        // anterior fica sem quadro nenhum a marcar entrada — e o passo só se registaria colado à
        // próxima ação do artista, seja ela qual for.
        self.any_input_this_frame |= authored;
        took
    }

    /// **Repõe a vista.** Devolve `true` se a tecla era desta janela.
    ///
    /// ⚠️ **Só morde com o ponteiro SOBRE a janela 3D** — ver a nota dentro da função. A porta
    /// perguntava apenas *"o smoke está armado?"*, o que bastava enquanto a única entrada era a
    /// variável de ambiente; o pill do topo tornou isso falso no mesmo dia.
    pub(crate) fn field3d_home_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::Home {
            return false;
        }
        let pos = self.last_pointer;
        with_smoke(|s| {
            // ⚠️ **Só com o ponteiro SOBRE a janela 3D**, e esta linha é a diferença entre uma tecla
            // e um sequestro. Enquanto o módulo entrava só por variável de ambiente, perguntar "o
            // smoke está armado?" bastava; com o pill ele pode estar ligado numa sessão normal, e
            // aí um `Home` engolido é um `Home` que não chega ao campo de texto onde o artista está
            // a escrever.
            //
            // É exatamente a nota que a `line/sculpt3d` viu envelhecer: a porta dela perguntava "a
            // cena existe?", o dia em que a cena passou a nascer sozinha chegou, e a partir dali ela
            // comia as teclas de todo painel do app. *Quem move o número que tornava uma nota
            // verdadeira tem de reconferir a nota* — e aqui quem o moveu fui eu, no mesmo dia.
            if !s.area.is_some_and(|a| {
                pos.0 >= a.x && pos.1 >= a.y && pos.0 < a.x + a.w && pos.1 < a.y + a.h
            }) {
                return false;
            }
            law::home(&mut s.cam);
            // Repor não é "voltar ao prato giratório": a mão continua no comando.
            s.manual = true;
            true
        })
        .unwrap_or(false)
    }

    /// ⭐ **`G` / `R` / `S` trocam o verbo do gizmo** — mover, rodar, escalar.
    ///
    /// ⚠️ **Só com o ponteiro SOBRE a janela 3D**, pela mesma razão do `Home` ao lado: uma tecla
    /// engolida é uma tecla que não chega ao campo de texto onde o artista está a escrever. `G`,
    /// `R` e `S` são letras comuns, e esta guarda é o que separa um atalho de um sequestro.
    ///
    /// As letras são as do Blender — G de *grab*, R de *rotate*, S de *scale*. ⚠️ Lá elas **começam** um gesto
    /// modal; aqui trocam o gizmo. É a mesma memória de dedo para o mesmo verbo, e a diferença de
    /// mecânica é a que o Blender também tem entre a tecla e a barra de ferramentas dele.
    ///
    /// O seletor do PAINEL é a outra porta, e é a que se encontra sem saber que existe.
    pub(crate) fn field3d_mode_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        let Some(mode) = mode_for_key(code, self.modifiers) else {
            return false;
        };
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !s.area.is_some_and(|a| {
                pos.0 >= a.x && pos.1 >= a.y && pos.0 < a.x + a.w && pos.1 < a.y + a.h
            }) {
                return false;
            }
            s.gizmo_mode = mode;
            // Trocar de verbo no meio de um arrasto deixaria uma alça agarrada que já não existe.
            s.drag = None;
            s.gizmo_hot = None;
            true
        })
        .unwrap_or(false)
    }

    /// A roda aproxima. `steps` em linhas de roda.
    pub(crate) fn field3d_wheel(&mut self, steps: f32) -> bool {
        let pos = self.last_pointer;
        // A pergunta é feita INTEIRA e neste arquivo de propósito — quem decide de quem é o gesto é
        // o módulo da cena, não o roteador (a nota do `sculpt3d_wheel`).
        if crate::forwarding::cursor_over_hero_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        with_smoke(|s| {
            let Some(area) = s.area else {
                return false;
            };
            if pos.0 < area.x
                || pos.1 < area.y
                || pos.0 >= area.x + area.w
                || pos.1 >= area.y + area.h
            {
                return false;
            }
            law::zoom(&mut s.cam, steps);
            s.manual = true;
            true
        })
        .unwrap_or(false)
    }
}

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
    // ⭐ **A alça ganha do gesto de câmera**, e só com o botão ESQUERDO: o direito continua a
    // orbitar mesmo por cima do gizmo, que é a saída para quem quer girar a vista sem primeiro
    // tirar o rato de cima da peça.
    let grabbed = (button == winit::event::MouseButton::Left)
        .then(|| local(s, pos).and_then(|p| field3d_gizmo::pick(&handles(s), p)))
        .flatten();
    s.drag = Some(grabbed.map_or(fallback, Drag::Gizmo));
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
pub(crate) fn advance(s: &mut Smoke, x: f32, y: f32) -> bool {
    let Some(drag) = s.drag else {
        // ⚠️ **Sem arrasto, o hover ainda é atualizado — e o evento NÃO é consumido.** As
        // duas metades importam: sem a primeira a alça nunca acende e o artista não sabe o
        // que vai agarrar; com a segunda invertida, a janela 3D engoliria todo movimento de
        // rato do app 2D.
        s.gizmo_hot = local(s, (x, y)).and_then(|p| field3d_gizmo::pick(&handles(s), p));
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
            let (Some(anchor), Some(area)) = (s.gizmo, s.area) else {
                return true;
            };
            let screen = Screen::new(
                area.w.round().max(1.0) as u32,
                area.h.round().max(1.0) as u32,
                s.cam.half_extent,
            );
            let d = field3d_gizmo::drag(
                handle,
                anchor,
                &s.cam,
                screen,
                [x - dx - area.x, y - dy - area.y],
                [x - area.x, y - area.y],
            );
            // ⚠️ Um pedido inerte **não é guardado**: uma alça degenerada devolve zero, e escrever
            // esse zero acordaria o traçado para redesenhar exatamente o mesmo quadro.
            if !d.is_idle() {
                // ⚠️ **Acumula com o que já estava**, e por verbo: somar translações, somar
                // ângulos, MULTIPLICAR fatores de escala. Ver `Motion::merge`.
                s.pending_move = Some((
                    anchor.entity,
                    s.pending_move
                        .filter(|(e, _)| *e == anchor.entity)
                        .map_or(d, |(_, acc)| acc.merge(d)),
                ));
            }
        }
    }
    true
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
fn mode_for_key(
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
    if was == Some(Drag::Orbit)
        && let (Some(from), Some(area)) = (s.press_at, s.area)
        && (s.last_pointer.0 - from.0).abs() <= CLICK_SLOP_PX
        && (s.last_pointer.1 - from.1).abs() <= CLICK_SLOP_PX
    {
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

#[cfg(test)]
#[path = "field3d_input_tests.rs"]
mod tests;
