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
use crate::field3d_smoke::{Drag, Grip, Smoke, with_smoke};
use ph2d_field_render::Screen;

/// **As alças do gizmo, projetadas para o enquadramento deste quadro** — ou vazio quando não há
/// nada selecionado.
///
/// ⚠️ Recalculadas a cada pergunta, e não guardadas: a câmera muda a cada quadro do prato giratório,
/// e uma lista de alças em cache seria a resposta de um enquadramento anterior — o mesmo congelador
/// que o traçado já pagou, só que a apontar em vez de a desenhar.
pub(crate) fn handles(s: &Smoke) -> Vec<field3d_gizmo::Projected> {
    let (Some(anchor), Some(screen)) = (s.gizmo, area_screen(s)) else {
        return Vec::new();
    };
    field3d_gizmo::project(anchor, &s.vp().cam, screen, s.gizmo_mode)
}

/// ⭐ **O enquadramento deste módulo é o da ÁREA — nunca o do traçado.**
///
/// ⚠️ **Os dois foram sempre o mesmo número e deixaram de o ser na W24**: o preview passou a traçar
/// mais grosso enquanto a mão mexe (ver [`crate::field3d_preview`]), então `Screen::new(tw, th, …)`
/// projetaria as alças a um terço do tamanho — o gizmo pousaria longe da superfície que ele move,
/// e só durante o movimento. *Uma projeção, um dono.*
pub(crate) fn area_screen(s: &Smoke) -> Option<Screen> {
    let area = s.vp().area?;
    Some(Screen::new(
        area.w.round().max(1.0) as u32,
        area.h.round().max(1.0) as u32,
        s.vp().cam.half_extent,
    ))
}

/// O ponto do cursor no referencial da **área desenhada** — que é o referencial em que o gizmo foi
/// projetado. ⚠️ Esquecer esta subtração faz as alças agarrarem deslocadas do tamanho da moldura do
/// app, e o defeito só aparece quando a janela 3D não começa em (0, 0).
fn local(s: &Smoke, p: (f32, f32)) -> Option<[f32; 2]> {
    let area = s.vp().area?;
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

/// ⭐ **A folga do enquadramento** (W46): quantos raios da peça cabem no meio-quadro.
///
/// ⚠️ **MEDIDO, não escolhido.** O critério não é estético: é *"nenhum pixel da peça toca a
/// moldura"*. Varredura sobre o **pior caso** — uma esfera sozinha, onde o bordo **é** a silhueta:
///
/// | folga | pixels na moldura | fração do quadro |
/// |---:|---:|---:|
/// | 0,90 | 252 | 79,3 % |
/// | 1,00 | **144** | 66,4 % |
/// | 1,05 | 72 | 60,5 % |
/// | **1,10** | **0** | 54,6 % |
/// | 1,40 | 0 | 32,3 % |
///
/// ⭐ **`1,00` NÃO chega**, e a razão é a lente: ela é convergente, e o lado da esfera virado para a
/// câmera projeta maior do que o raio. Um bordo conservador não compensa isso — ele é conservador no
/// **mundo**, e o corte acontece na **projeção**.
///
/// ⚠️ A varredura só disse isto depois de a fixtura mudar: com uma **união de duas** esferas o bordo
/// é muito maior do que a silhueta e **todas** as folgas davam zero, `0,90` incluída. *Uma fixtura
/// que concorda não prova nada.* Gate: `the_frame_margin_is_the_smallest_one_that_cuts_nothing`.
const FRAME_MARGIN: f32 = 1.10;

/// **As três leis da câmera, puras** — sem `App`, sem ponteiro, sem estado do smoke.
///
/// ⭐ A separação não é estética: é o que torna as leis **testáveis pela porta do produto**. Um gate
/// que precisasse de um `App` inteiro para perguntar *"arrastar para a direita vira o modelo para a
/// direita?"* não seria escrito — e foi exatamente essa pergunta que a `line/sculpt3d` respondeu
/// errado nos dois sinais até um smoke a pegar. Aqui o gate traça a peça e **mede-a na tela**.
#[path = "field3d_input_law.rs"]
pub(crate) mod law;

/// ⭐ **Enquadra a peça que o smoke tem em mãos** — o elo entre a lei pura e o documento.
///
/// ⚠️ **`false` quando não há peça**, e quem chama decide o que fazer com isso: o `Home` já repôs a
/// orientação e fica assim (não há o que enquadrar); o pedido de um load simplesmente não tem efeito
/// e volta a ser feito no quadro seguinte, quando o documento já estiver cozido.
pub(crate) fn frame_the_part(s: &mut Smoke) -> bool {
    let mut to = s.vp().cam;
    if !frame_into(s, &mut to) {
        return false;
    }
    crate::field3d_smoke::fly_to(s, to);
    true
}

/// A mesma conta, escrita num destino em vez de na câmera — é ela que faz o `Home` **compor** o
/// repor com o enquadrar numa viagem só, em vez de duas.
pub(crate) fn frame_into(s: &Smoke, to: &mut ph2d_field_render::Orbit) -> bool {
    let Some(doc) = s.doc.as_ref() else {
        return false;
    };
    let reg = crate::field3d_smoke::sampled_registry();
    let Some(ball) = ph2d_field_eval::bounds::bounding_ball(doc, &reg) else {
        return false;
    };
    law::frame(to, ball);
    true
}

/// ⭐ **Partir para uma vista nomeada**: a orientação dela **e** o enquadramento, numa viagem só.
pub(crate) fn fly_to_view(s: &mut Smoke, view: crate::field3d_views::Standard) {
    let mut to = s.vp().cam;
    to.rotation = view.rotation();
    frame_into(s, &mut to);
    crate::field3d_smoke::fly_to(s, to);
}

impl App {
    /// O ponteiro desceu. Devolve `true` se a janela 3D tomou o gesto.
    /// ⭐⭐⭐ **O MÓDULO 3D CALA-SE ENQUANTO UM MODAL DE TELA CHEIA ESTÁ ABERTO** — teclado, ponteiro
    /// e roda.
    ///
    /// # ⛔ O defeito, e por que ele apanhou as duas metades
    ///
    /// Smoke do Enio (2026-08-29): *«o modal não funciona, não fecha. Os modelos do modal não são
    /// criados.»* **Um mecanismo, dois sintomas.** O `field3d_pointer_down` corre **antes** do
    /// despacho de chrome (`input_dispatch.rs`) e reclama todo gesto que começa **dentro da área que
    /// a janela 3D desenhou** — e a paleta cobre o ecrã inteiro, essa área incluída. ⇒ o clique no
    /// item nunca chegava ao handler da paleta: ela não registava o pick (**nada era criado**) e não
    /// se fechava (**quem a fecha é o mesmo handler**).
    ///
    /// ⚠️ **A guarda que já existia não bastava, e é instrutivo porquê:** o
    /// `cursor_over_hero_chrome` pergunta *«há um painel por cima?»*, e um painel publica um
    /// `panel_rect`. Um **modal** não é um painel — ele não publica rect nenhum, e a pergunta
    /// respondia «não» com o modal a tapar tudo.
    ///
    /// ⭐ **Uma porta, quatro leitores** (`pointer_down`, `pointer_move`, `wheel`, `field3d_keys`).
    /// A lei escrita em quatro sítios seria a lei escrita em nenhum: a quinta entrada nasceria
    /// surda, e o sintoma dela seria este mesmo — *«o modal não faz nada»*.
    ///
    /// ⚠️ **O SOLTAR fica de FORA, de propósito.** Não se pode *começar* um gesto através do modal,
    /// mas um gesto **já em curso** tem de poder acabar: guardar o `up` deixaria o `Drag` pousado
    /// para sempre e a peça a orbitar sozinha ao fechar a paleta.
    ///
    /// ⚠️ **E há um IRMÃO por curar, que não é desta linha:** o `sculpt3d_pointer_down` corre
    /// **antes** deste no mesmo despacho e tem exactamente a mesma forma — com o `Ctrl+K` ou a
    /// biblioteca do Motion abertos sobre o módulo de escultura armado, o clique morre ali. A
    /// `line/sculpt3d` é a dona daquele módulo; está nomeado no handoff.
    pub(crate) fn field3d_yields_to_modal(&self) -> bool {
        self.command_palette_open()
    }

    pub(crate) fn field3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        if self.field3d_yields_to_modal() {
            return false;
        }
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
        // ⭐ **A MESMA tecla que o canvas 2D usa** para falar da seleção (`input_dispatch`:
        // `shift_key() || super_key() || control_key()`). Um terceiro vocabulário de modificador no
        // mesmo app é onde a mão aprende errado.
        let additive = self.modifiers.shift_key()
            || self.modifiers.super_key()
            || self.modifiers.control_key();
        with_smoke(|s| begin(s, button, fallback, additive, pos)).unwrap_or(false)
    }

    /// O ponteiro moveu. **Só consome com um arrasto em curso** — senão a janela 3D engoliria todo
    /// hover do app 2D.
    pub(crate) fn field3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        // ⛔ Ver [`Self::field3d_yields_to_modal`]: com a paleta aberta o rato é dela. Um arrasto
        // em curso simplesmente não avança — e o soltar, que não é guardado, fecha-o em paz.
        if self.field3d_yields_to_modal() {
            return false;
        }
        // ⚠️ **O `Ctrl` é lido AQUI, todo movimento** — o `winit` não o manda no evento de
        // movimento, e o shell já o guarda. Ver `Smoke::snapping` sobre por que ele não é congelado
        // na pegada.
        let snapping = self.modifiers.control_key();
        let (took, authored) = with_smoke(|s| {
            s.snapping = snapping;
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
            if !over_window(s, pos) {
                return false;
            }
            let mut to = s.vp().cam;
            law::home(&mut to);
            // ⭐⭐ **E ENQUADRA A PEÇA** (W46). ⚠️ Até aqui o `Home` punha o alvo na **origem** — e
            // uma peça longe dela continuava fora do quadro **depois** de a tecla correr. A tecla
            // que existe para desfazer «estou perdido» não encontrava a peça.
            //
            // ⚠️ É a lei da referência: no Blender, `Home` é *View All* — enquadrar tudo, não repor
            // um ângulo fixo. Nós tínhamos herdado a tecla e metade do significado.
            //
            // Sem peça (documento vazio) fica só o repor, que é a resposta certa: não há o que
            // enquadrar.
            frame_into(s, &mut to);
            crate::field3d_smoke::fly_to(s, to);
            // Repor não é "voltar ao prato giratório": a mão continua no comando.
            s.vp_mut().manual = true;
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
    /// ⭐ **A tecla que alterna a LENTE** — convergente ou paralela.
    ///
    /// ⚠️ É a comparação que a nota da câmera pedia por extenso: *"a perspectiva merece a sua própria
    /// comparação lado a lado, não uma troca silenciosa"*. Uma tecla é o lado a lado.
    ///
    /// `Numpad5` é a tecla do Blender para a mesma coisa, e a memória de dedo vale mais do que uma letra
    /// livre. ⚠️ Com modificador não é atalho deste módulo, pela mesma razão do `mode_for_key`.
    pub(crate) fn field3d_lens_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::Numpad5
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            s.vp_mut().cam.lens = law::other_lens(s.vp_mut().cam.lens);
            true
        })
        .unwrap_or(false)
    }

    /// ⭐⭐⭐ **`Ctrl+Alt+Q` abre e fecha a DIVISÃO do canvas** (W90) — a tecla do Blender para a
    /// mesma coisa (*Toggle Quad View*).
    ///
    /// ⚠️ **Com os TRÊS modificadores exigidos por nome**, e não «pelo menos estes»: um
    /// `Ctrl+Alt+Shift+Q` é de outra pessoa, e engoli-lo é o sequestro que a nota do `mode_for_key`
    /// descreve. Mesma guarda de ponteiro das irmãs.
    pub(crate) fn field3d_quad_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::KeyQ
            || !self.modifiers.control_key()
            || !self.modifiers.alt_key()
            || self.modifiers.shift_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            crate::field3d_smoke::toggle_split(s);
            true
        })
        .unwrap_or(false)
    }

    /// ⭐⭐ **`Numpad1/3/7` põem a câmera numa VISTA NOMEADA**, e `Ctrl` dá a oposta (W47).
    ///
    /// ⚠️ As teclas são as do Blender — **as teclas, não os eixos**: ele é Z para cima e este módulo
    /// é Y para cima, e copiar os eixos dele daria uma «frente» a olhar para o chão
    /// ([`crate::field3d_views`]).
    ///
    /// ⚠️ Ela **enquadra** junto (W46): uma vista de frente que deixasse a peça fora do quadro seria
    /// a mesma tela vazia que as duas waves anteriores fecharam.
    ///
    /// Mesma guarda de ponteiro das irmãs.
    pub(crate) fn field3d_view_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if self.modifiers.alt_key() || self.modifiers.super_key() {
            return false;
        }
        let Some(view) = crate::field3d_views::view_for_key(code, self.modifiers.control_key())
        else {
            return false;
        };
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            fly_to_view(s, view);
            s.vp_mut().manual = true;
            true
        })
        .unwrap_or(false)
    }

    /// ⭐ **`Shift+I` isola o escolhido — ou devolve a peça inteira** (W44).
    ///
    /// ⚠️ **A tecla é a do módulo irmão**, lida e não escolhida (`sculpt3d_keys`: `Shift+I` no bloco
    /// do shift). Duas janelas 3D no mesmo app com teclas diferentes para o mesmo gesto seria o
    /// artista a aprender duas vezes o que é uma coisa só.
    ///
    /// ⭐ **E ela é a PORTA DE SAÍDA que faltava.** O chip da fileira só é pintado quando o escolhido
    /// se destaca da peça; com a **raiz** escolhida — ou com nada — não havia gesto nenhum que
    /// devolvesse a peça isolada. A lei está em [`crate::field3d_smoke::key_isolation`], e o pedido
    /// atravessa por caixa de correio porque precisa da **seleção**, que vive na ponte com a cena.
    ///
    /// ⚠️ Mesma guarda de ponteiro das irmãs: sem ela, um `Shift+I` num campo de texto viraria um
    /// gesto de vista.
    pub(crate) fn field3d_isolate_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::KeyI
            || !self.modifiers.shift_key()
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            crate::field3d_smoke::ask_isolate_key();
            true
        })
        .unwrap_or(false)
    }

    /// ⭐⭐ **`A` ABRE A PALETA DE FORMAS** (W100) — a porta rápida do catálogo.
    ///
    /// ⚠️ **`A` e não outra letra**, e a escolha tem precedente nesta casa: é a tecla que abre a
    /// biblioteca de nós do Motion, e a paleta é literalmente o **mesmo** widget. Uma segunda letra
    /// para o mesmo modal seria o app a ensinar duas coisas onde há uma. (O `Shift+A` do Blender
    /// abre um menu, não uma paleta com busca — a analogia mais próxima é a do Motion.)
    ///
    /// ⚠️ **Sem modificador nenhum**, e com a guarda de ponteiro das irmãs: sem ela, um `A` num
    /// campo de texto abriria um modal por cima do que se está a escrever. E é por isso que o `G`,
    /// `R` e `S` já a têm — a letra solta é a mais perigosa de todas.
    pub(crate) fn field3d_add_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::KeyA
            || self.modifiers.shift_key()
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            crate::field3d_smoke::ask_shape_palette();
            true
        })
        .unwrap_or(false)
    }

    /// ⭐ **O número digitado no meio do gesto** (W26) — o `G X 0,5` do Blender.
    ///
    /// ⚠️ **Ela vem ANTES da tecla de verbo** no roteador, e a ordem é a lei: com uma entrada aberta,
    /// um `5` é um cinco. E as duas guardas continuam a valer — ponteiro sobre a janela 3D **e** uma
    /// alça agarrada —, o que torna impossível esta porta comer uma tecla de um campo de texto: para
    /// ela abrir é preciso ter o botão do rato em baixo, em cima de uma alça do gizmo.
    pub(crate) fn field3d_typed_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if self.modifiers.control_key() || self.modifiers.alt_key() || self.modifiers.super_key() {
            return false;
        }
        let Some(stroke) = crate::field3d_typed::stroke_for(code) else {
            return false;
        };
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            typed_key(s, stroke)
        })
        .unwrap_or(false)
    }

    pub(crate) fn field3d_mode_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        let Some(mode) = mode_for_key(code, self.modifiers) else {
            return false;
        };
        let pos = self.last_pointer;
        with_smoke(|s| {
            if !over_window(s, pos) {
                return false;
            }
            s.gizmo_mode = mode;
            // Trocar de verbo no meio de um arrasto deixaria uma alça agarrada que já não existe.
            s.drag = None;
            s.gizmo_hot = None;
            s.typed = None;
            true
        })
        .unwrap_or(false)
    }

    /// A roda aproxima. `steps` em linhas de roda.
    pub(crate) fn field3d_wheel(&mut self, steps: f32) -> bool {
        // ⛔ Ver [`Self::field3d_yields_to_modal`]: a paleta ROLA, e sem esta linha a roda dela
        // aproximava a peça por baixo em vez de percorrer a lista.
        if self.field3d_yields_to_modal() {
            return false;
        }
        let pos = self.last_pointer;
        // A pergunta é feita INTEIRA e neste arquivo de propósito — quem decide de quem é o gesto é
        // o módulo da cena, não o roteador (a nota do `sculpt3d_wheel`).
        if crate::forwarding::cursor_over_hero_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        with_smoke(|s| {
            let Some(area) = s.vp().area else {
                return false;
            };
            if pos.0 < area.x
                || pos.1 < area.y
                || pos.0 >= area.x + area.w
                || pos.1 >= area.y + area.h
            {
                return false;
            }
            // A roda é mão: cancela a viagem (W51), como o arrasto.
            crate::field3d_smoke::cancel_flight(s);
            law::zoom(&mut s.vp_mut().cam, steps);
            s.vp_mut().manual = true;
            true
        })
        .unwrap_or(false)
    }
}

/// ⭐ **O gesto do ponteiro** vive no irmão — ver [`field3d_input_pointer`](self::pointer).
#[path = "field3d_input_pointer.rs"]
mod pointer;
#[cfg(test)]
pub(crate) use pointer::finish_for_test;
pub(crate) use pointer::{advance, begin, finish, hot_handle, typed_key};
use pointer::{mode_for_key, over_window};
#[cfg(test)]
#[path = "field3d_input_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "field3d_frame_tests.rs"]
mod frame_tests;
