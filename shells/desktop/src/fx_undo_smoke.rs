//! **O UNDO da pilha de efeitos, AUTO-DIRIGIDO** (`PH2D_BUILD_SMOKE=20`) — o aparelho que
//! faltava ao report do Enio (*"undo/redo ainda não implementado para efeitos"*, 3×).
//!
//! # Por que este arquivo existe
//!
//! A sessão de 2026-07-19 varreu o caminho `input→bus→mutação→diff→restore` com dois agentes e
//! quatro passagens, escreveu um gate por tipo de efeito, e **não conseguiu reproduzir** —
//! encerrando com *"Não afirmo que fechou"*. O gate dela (`undo_tests.rs`) chama
//! `fx_bridge::add` **diretamente** e prova que o ESTADO ida-e-volta: pôr efeito muda a captura,
//! restaurar tira, e recapturar é ponto fixo. Tudo verdade, e nada disso toca a pergunta que o
//! artista faz — *"o meu CLIQUE virou um passo de undo?"*.
//!
//! Entre o clique e o passo há uma máquina que aquele gate não contém: o Click nasce no `Up`,
//! atravessa o bus, é aplicado **dentro do `render_frame`**, e o `post_frame_undo` decide com
//! base em dois flags (`any_input_this_frame`, `held_button`) que vivem no ritmo dos EVENTOS,
//! não no do drain. Uma fixture que chama a mutação à mão nunca vê essa costura.
//!
//! Este roteiro clica o botão **de verdade** — pelo hit-index, com Down e Up em frames
//! separados, como um dedo — e imprime, por frame, o que decide a questão.
//!
//! # Como se lê a telemetria
//!
//! ```text
//! [fx-undo] f=NN fx=<nº de efeitos no caminho> undo=<profundidade> redo=<0|1> alvo=<sim|nao>
//! ```
//!
//! - `fx` sobe de 0→1 no frame do clique e **`undo` sobe junto**: o passo foi registado.
//! - `fx` sobe e `undo` **não**: o efeito foi aplicado e o passo não existe — é o report, e a
//!   causa está a montante do diff (o frame do drain não coincidiu com o frame do input).
//! - `undo` continua a subir em frames onde nada foi clicado: passo ESPÚRIO — o 1º Ctrl+Z
//!   gasta-se nele, e do lado de fora isso é indistinguível de *"o undo não faz nada"*.
//!   (É a classe do `vec_zorder_fixpoint_tests`.)
//! - Depois do Ctrl+Z, `fx` tem de voltar a 0 **numa só** aplicação.

use std::cell::Cell;

use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

thread_local! {
    /// Onde o slider foi agarrado — os frames de arrasto movem-se a partir DAQUI, não da
    /// posição corrente do cursor (que o compositor pode mexer entre frames).
    static GRAB: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

/// O Down num ícone do cabeçalho de uma linha da pilha. `None` no hit-index é reportado em vez
/// de silenciado — um roteiro que não clica nada e não diz nada leria como "passou".
fn click_row_icon(app: &mut crate::App, id: ph2d_editor::NodeId, name: &str) {
    match app.smoke_find_widget(id) {
        Some((x, y)) => {
            eprintln!("[fx-undo] DOWN no {name} da linha 0 em ({x}, {y})");
            app.smoke_pointer_down(x, y);
        }
        None => eprintln!("[fx-undo] ⚠️ o {name} da linha 0 NÃO está no hit-index"),
    }
}

/// O frame em que o Down do botão "Add" cai. Depois do `sync` (que dá entidade à forma) e da
/// seleção, e com folga para a seção Effects já ter sido pintada uma vez — um widget só está no
/// hit-index depois de o painel o desenhar.
const ADD_DOWN: u32 = 12;

/// O Up do mesmo clique. **Frame separado do Down, de propósito**: é onde o `Click` nasce, e um
/// press+release no mesmo evento não contém a corrida que um dedo contém.
const ADD_UP: u32 = 15;

/// **O ARRASTO do 1º slider de parâmetro.** É o gesto mais frequente da pilha — e o de risco
/// próprio: durante um arrasto o `held_button` está `Some` e o `post_frame_undo` **suprime** o
/// passo de propósito (senão seriam N passos, um por frame). Todo o peso cai no frame do `Up`,
/// e é justamente essa a costura que uma fixture que chama a mutação à mão nunca exercita.
const PARAM_DOWN: u32 = 24;
const PARAM_UP: u32 = 36;

/// O **Hide** (o olho do card) — um clique de um frame, num widget que muta a pilha sem mexer
/// em geometria nenhuma. Se algum gesto da pilha não registasse passo, é um candidato natural:
/// o diff dele é o mais pequeno de todos (um `bool`).
const HIDE_DOWN: u32 = 44;
const HIDE_UP: u32 = 47;

/// O **Remove** (o X do card) — a pilha volta a 0 entradas.
const REMOVE_DOWN: u32 = 55;
const REMOVE_UP: u32 = 58;

/// Um SEGUNDO Add, só para haver o que assar no passo seguinte.
const ADD2_DOWN: u32 = 62;
const ADD2_UP: u32 = 65;

/// Um SEGUNDO arrasto de parâmetro, **antes do Apply**. ⚠️ Sem ele o teste do Apply é vazio: o
/// efeito recém-posto nasce NEUTRO (`p0 = 0.00`), o cozido é igual à fonte, e o bake é a
/// identidade — `verts` não se mexeria e o gate ficaria verde sem nunca ter assado nada.
const PARAM2_DOWN: u32 = 68;
const PARAM2_UP: u32 = 80;

/// O **Apply Effects** — o *commit* da pilha (assa o cozido na geometria e esvazia a pilha). É o
/// gesto mais pesado de todos: ele muda `verts` **e** `effects` de uma vez, então é o único em
/// que "desfazer" tem de restaurar duas coisas ao mesmo tempo.
const APPLY_DOWN: u32 = 88;
const APPLY_UP: u32 = 91;

/// O 1º de SEIS Ctrl+Z. Cada um tem de desfazer exatamente um gesto, na ordem inversa:
/// apply → o 2º efeito → remover → esconder → o parâmetro → o 1º efeito. Se algum gesto não
/// registou passo, a caminhada chega ao fim cedo demais e a telemetria mostra-o.
const UNDO_FIRST: u32 = 100;

/// Quantos Ctrl+Z a caminhada dá — um por gesto.
const UNDO_STEPS: u32 = 7;

/// Quantos frames separam um Ctrl+Z do seguinte (Down e Up ocupam 2, o resto é folga para se
/// ver se um passo espúrio nasce entre eles).
const UNDO_EVERY: u32 = 8;

/// Onde o roteiro para de falar.
const LAST: u32 = 172;

/// **O que tem de ser verdade, e QUANDO** — o roteiro verifica-se a si mesmo.
///
/// Cada linha é `(frame, profundidade de undo, nº de efeitos, nº de vértices)`, lida DEPOIS de o
/// gesto daquele frame ter assentado. Sem isto o probe é uma parede de números que alguém tem de
/// ler com atenção — e um humano cansado lê "sobe" onde está escrito "não sobe".
///
/// A tabela conta a história inteira: 7 gestos, 7 passos, e a caminhada de volta pelos 7.
const EXPECTED: &[(u32, usize, usize, usize)] = &[
    (11, 1, 0, 4),  // a forma, no seu próprio passo
    (20, 2, 1, 4),  // Add — a pilha ganhou uma entrada
    (40, 3, 1, 4),  // o arrasto do parâmetro — UM passo para o gesto inteiro
    (52, 4, 1, 4),  // Hide
    (61, 5, 0, 4),  // Remove
    (67, 6, 1, 4),  // Add (2º)
    (86, 7, 1, 4),  // o 2º arrasto (deixa o efeito NÃO-neutro, para o bake ter o que assar)
    (99, 8, 0, 2),  // Apply — assou a geometria (4→2 verts) E esvaziou a pilha
    (104, 7, 1, 4), // Ctrl+Z #1 — repõe as DUAS metades num passo só
    (112, 6, 1, 4), // #2 — desfaz o 2º arrasto
    (120, 5, 0, 4), // #3 — desfaz o 2º Add
    (128, 4, 1, 4), // #4 — desfaz o Remove
    (136, 3, 1, 4), // #5 — desfaz o Hide
    (144, 2, 1, 4), // #6 — desfaz o 1º arrasto
    (152, 1, 0, 4), // #7 — desfaz o 1º Add; sobra a forma
];

thread_local! {
    /// Quantas linhas da [`EXPECTED`] falharam. O veredito final lê-o.
    static FAILED: Cell<u32> = const { Cell::new(0) };
}

/// Confere a linha da [`EXPECTED`] deste frame, se houver, e imprime o veredito no fim.
fn verify(app: &crate::App, f: u32, undo: usize, fx: usize, verts: usize) {
    for &(at, e_undo, e_fx, e_verts) in EXPECTED {
        if at != f {
            continue;
        }
        let ok = (undo, fx, verts) == (e_undo, e_fx, e_verts);
        if !ok {
            FAILED.with(|c| c.set(c.get() + 1));
        }
        eprintln!(
            "[fx-undo] {} f={f}: undo={undo}/{e_undo} fx={fx}/{e_fx} verts={verts}/{e_verts}",
            if ok { "OK  " } else { "FALHA" },
        );
    }
    if f == LAST {
        let bad = FAILED.with(Cell::get);
        let paths = app.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
        eprintln!(
            "[fx-undo] ===== VEREDITO: {} ===== ({} de {} conferências falharam; paths={paths})",
            if bad == 0 {
                "o undo da pilha de efeitos FUNCIONA nos 7 gestos"
            } else {
                "REPRODUZIDO — leia as linhas FALHA acima"
            },
            bad,
            EXPECTED.len(),
        );
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    // Os quatro Ctrl+Z, espaçados. Fora do `match` principal porque são um RITMO, não frames
    // avulsos — escrevê-los como braços seria a mesma linha quatro vezes com números à mão.
    if f >= UNDO_FIRST {
        let k = (f - UNDO_FIRST) / UNDO_EVERY;
        let phase = (f - UNDO_FIRST) % UNDO_EVERY;
        if k < UNDO_STEPS {
            match phase {
                0 => {
                    eprintln!("[fx-undo] Ctrl+Z #{}", k + 1);
                    app.smoke_key_z(false, true);
                }
                1 => app.smoke_key_z(false, false),
                _ => {}
            }
        }
    }
    match f {
        3 => build(app),
        4 => select_the_shape(app),
        // ⚠️ **Fecha a forma no seu PRÓPRIO passo de undo**, e sem isto o roteiro mente.
        //
        // O baseline é armado no 1º frame com `gfx` — antes de a elipse existir —, e a elipse
        // nasce por código, sem input. O `post_frame_undo` só compara quando houve input, então
        // o 1º input do roteiro (o Up do Add) via um diff que continha a forma INTEIRA **e** o
        // efeito, colapsados num passo só: o Ctrl+Z apagava a elipse, e a leitura ingênua era
        // *"o undo do efeito leva a forma junto"*. É artefacto do harness, não do produto — no
        // produto o artista DESENHA a forma (input ⇒ passo) antes de lhe pôr um efeito.
        //
        // Um pulso de input aqui reproduz esse passo e deixa o clique do Add ser o passo 2 — a
        // única forma de o gate poder afirmar *"o Ctrl+Z tira o efeito e a forma FICA"*.
        5 => app.any_input_this_frame = true,
        // A seção Effects pode nascer fora da vista: rola o painel até o botão entrar no
        // hit-index (o mesmo gesto do roteiro do Expand).
        6..=11 => scroll_until_reachable(app, ph2d_editor::ids::vector_fx_add_id(0)),
        // Depois do 2º Add o card voltou e empurrou o Apply para fora — re-rola até ele.
        82..=87 => scroll_until_reachable(app, ph2d_editor::ids::VECTOR_FX_APPLY),
        ADD_DOWN => match app.smoke_find_widget(ph2d_editor::ids::vector_fx_add_id(0)) {
            Some((x, y)) => {
                eprintln!("[fx-undo] DOWN no botão Add (kind 0) em ({x}, {y})");
                app.smoke_pointer_down(x, y);
            }
            None => eprintln!(
                "[fx-undo] ⚠️ o botão Add NÃO está no hit-index — roteiro morto (a seção \
                 Effects não foi pintada, ou não há alvo único selecionado)"
            ),
        },
        ADD_UP => {
            eprintln!("[fx-undo] UP — é AQUI que o Click nasce e o efeito tem de entrar");
            app.smoke_pointer_up();
        }
        // O ARRASTO do parâmetro: agarra o slider da linha 0 e leva-o para a direita.
        PARAM_DOWN => match app.smoke_find_widget(ph2d_editor::ids::vector_fx_param_id(0, 0)) {
            Some((x, y)) => {
                GRAB.with(|c| c.set((x, y)));
                eprintln!("[fx-undo] DOWN no slider do parâmetro 0 em ({x}, {y})");
                app.smoke_pointer_down(x, y);
            }
            None => eprintln!("[fx-undo] ⚠️ slider do parâmetro fora do hit-index"),
        },
        PARAM2_DOWN => match app.smoke_find_widget(ph2d_editor::ids::vector_fx_param_id(0, 0)) {
            Some((x, y)) => {
                GRAB.with(|c| c.set((x, y)));
                eprintln!("[fx-undo] DOWN no slider do parâmetro 0 (2o) em ({x}, {y})");
                app.smoke_pointer_down(x, y);
            }
            None => eprintln!("[fx-undo] ⚠️ slider do parâmetro (2o) fora do hit-index"),
        },
        n if (n > PARAM_DOWN && n < PARAM_UP) || (n > PARAM2_DOWN && n < PARAM2_UP) => {
            let (x, y) = GRAB.with(std::cell::Cell::get);
            let base = if n < PARAM_UP {
                PARAM_DOWN
            } else {
                PARAM2_DOWN
            };
            app.smoke_pointer_move(x + ((n - base) * 6) as f32, y);
        }
        PARAM_UP | PARAM2_UP => {
            // Re-afirma a posição NO frame do release (a lição do KWin: o cursor físico pode
            // ter falado depois do último move, e o Up solta ONDE o ponteiro está).
            let (x, y) = GRAB.with(std::cell::Cell::get);
            app.smoke_pointer_move(x + 72.0, y);
            eprintln!("[fx-undo] UP do arrasto — UM passo para o gesto inteiro");
            app.smoke_pointer_up();
        }
        HIDE_DOWN => click_row_icon(app, ph2d_editor::ids::vector_fx_hide_id(0), "HIDE"),
        REMOVE_DOWN => click_row_icon(app, ph2d_editor::ids::vector_fx_remove_id(0), "REMOVE"),
        ADD2_DOWN => click_row_icon(app, ph2d_editor::ids::vector_fx_add_id(0), "ADD (2o)"),
        APPLY_DOWN => click_row_icon(app, ph2d_editor::ids::VECTOR_FX_APPLY, "APPLY"),
        HIDE_UP | REMOVE_UP | ADD2_UP | APPLY_UP => app.smoke_pointer_up(),
        _ => {}
    }
    if f <= LAST {
        telemetry(app, f);
    }
}

/// O que decide a questão, por frame: quantos efeitos o caminho tem, e o que a fila de undo diz.
fn telemetry(app: &crate::App, f: u32) {
    let fx = app.gfx.as_ref().map_or(0, |g| {
        g.vec_scene.paths().first().map_or(0, |p| p.effects.len())
    });
    // O 1º parâmetro da 1ª linha + se ela está escondida: é o que torna o arrasto e o Hide
    // legíveis (os dois mudam a pilha sem mudar o NÚMERO de entradas).
    let (p0, on) = app.gfx.as_ref().map_or((f64::NAN, true), |g| {
        g.vec_scene
            .paths()
            .first()
            .and_then(|p| p.effects.first())
            .map_or((f64::NAN, true), |e| (e.effect.get(0), e.enabled))
    });
    let target = crate::fx_bridge::sole_path(app.vec_pen.selected_paths()).is_some();
    let paths = app.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
    // Os VÉRTICES autorados — o Apply assa o cozido neles, então é aqui que se vê o bake (e se
    // vê o Ctrl+Z desfazê-lo).
    let verts = app.gfx.as_ref().map_or(0, |g| {
        g.vec_scene.paths().first().map_or(0, |p| p.verts.len())
    });
    eprintln!(
        "[fx-undo] f={f} paths={paths} verts={verts} fx={fx} p0={p0:.2} on={on} undo={} redo={} alvo={}",
        app.undo.depth(),
        u8::from(app.undo.can_redo()),
        if target { "sim" } else { "nao" },
    );
    verify(app, f, app.undo.depth(), fx, verts);
}

/// Rola o painel até `id` ser alcançável. Não faz nada quando já está.
///
/// ⚠️ **É preciso re-rolar entre fases.** Pôr um efeito faz nascer um card, o card empurra o
/// resto da seção para baixo, e o botão que a fase seguinte quer clicar sai da vista — foi o que
/// aconteceu com o **Apply Effects** na 1ª corrida (`⚠️ NÃO está no hit-index`), e a leitura
/// ingênua disso seria *"o Apply está morto"*. Um widget fora da janela não é um widget morto.
fn scroll_until_reachable(app: &mut crate::App, id: ph2d_editor::NodeId) {
    if app.smoke_find_widget(id).is_some() {
        return;
    }
    let Some(w) = app.gfx.as_ref().map(|g| g.surface.size()) else {
        return;
    };
    let (px, py) = (w.width as f32 - 120.0, w.height as f32 * 0.5);
    app.smoke_pointer_move(px, py);
    app.on_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, -24.0));
}

/// UMA elipse, **sem efeito nenhum** — a pilha tem de começar vazia para o clique ser o que a
/// enche. (O nível 14 nasce com um Zig Zag já posto; ali o "0→1" não existiria.)
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    gfx.vec_scene.push_path(shape(
        ShapeKind::Ellipse,
        [-1.6, -1.6],
        [1.6, 1.6],
        &[],
        [90, 150, 220],
    ));
}

/// Seleciona a forma — a seção Effects só publica alvo com **um** caminho selecionado
/// (`fx_bridge::sole_path`), e sem alvo os botões Add nem são pintados.
fn select_the_shape(app: &mut crate::App) {
    let ids: Vec<VecPathId> = app
        .gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default();
    app.vec_pen.select_many(&ids);
    eprintln!(
        "[smoke] UNDO DA PILHA DE EFEITOS (auto-dirigido) — não toque no mouse.\n\
         \x20 O roteiro clica **Add** (o 1º efeito da lista) e depois dá **Ctrl+Z**.\n\
         \x20 Leia o terminal: `fx` tem de ir 0->1 no clique, `undo` tem de subir NO MESMO\n\
         \x20 frame, e depois do Ctrl+Z `fx` tem de voltar a 0 numa só aplicação."
    );
}
