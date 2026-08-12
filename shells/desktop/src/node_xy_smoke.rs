//! **A cena do X/Y do NÓ** — `PH2D_BUILD_SMOKE=73` (plano 25 §9, W6).
//!
//! # A pergunta da wave está no PLURAL
//!
//! ⚠️ Com **um** nó selecionado, o modelo do Blender (mediana + deslocamento) e o do
//! Illustrator (escrever o alvo no nó) dão **exatamente o mesmo resultado** — a mediana
//! de um conjunto de um É o elemento. O defeito só é visível com **dois ou mais**, e
//! nele os dois modelos discordam de forma destrutiva: escrever o alvo em cada nó
//! **junta todos no mesmo X**, um *alinhar* disfarçado de coordenada, que é a queixa
//! conhecida do Inkscape.
//!
//! Por isso a cena não é um quadrado com um nó a mover: é uma **viga larga**, cujos dois
//! nós de baixo estão longe um do outro. Se eles colapsarem numa coluna só, dá para ver
//! da outra ponta da sala.
//!
//! # O que a cena arma, e o que ela deixa para o artista
//!
//! Arma a geometria e o **modo Node** (a precondição — é o modo em que as âncoras
//! existem). Não arma nada do gesto: selecionar os nós, ler o número e digitar são o
//! que a wave existe para provar, e um smoke que os arma por baixo do pano pula
//! exatamente a costura que devia testar.

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Meia-largura da VIGA, em mundo — o número que torna o colapso visível.
const BEAM_HX: f64 = 2.0;
/// Meia-altura da viga.
const BEAM_HY: f64 = 0.5;
/// Meia-largura do marcador de referência.
const MARK_HALF: f64 = 0.15;
/// Onde o marcador está — o X que o artista vai digitar.
///
/// Escolhido redondo na unidade de default (100 px/m ⇒ **150 px**): um número redondo é o
/// que torna a conferência uma leitura, não uma conta.
const TARGET_X: f64 = 1.5;

const BLUE: [u8; 3] = [86, 132, 214];
const AMBER: [u8; 3] = [214, 150, 70];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        8 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    // A VIGA: os dois nós de baixo distam `2 * BEAM_HX` — é essa distância que o gesto
    // não pode encolher.
    gfx.vec_scene.push_path(tint(
        rectangle([-BEAM_HX, -BEAM_HY], [BEAM_HX, BEAM_HY]),
        BLUE,
    ));
    // O MARCADOR: fica onde o artista vai mandar a mediana pousar, para o passo 3 ser
    // uma conferência de olho e não de número.
    gfx.vec_scene.push_path(tint(
        rectangle(
            [TARGET_X - MARK_HALF, -BEAM_HY - 4.0 * MARK_HALF],
            [TARGET_X + MARK_HALF, -BEAM_HY - 2.0 * MARK_HALF],
        ),
        AMBER,
    ));
    // Node: é o modo em que as âncoras existem e a seção Vertex aparece.
    app.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
}

/// A mensagem — com os números MEDIDOS da cena viva e das settings vivas, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    let nodes: usize = gfx.vec_scene.paths().iter().map(|p| p.total_verts()).sum();
    // As settings VIVAS — a mesma fonte que o painel lê no frame.
    let display = gfx
        .hero_screen
        .as_ref()
        .map_or_else(ph2d_editor::LengthDisplay::default, |h| {
            ph2d_editor::LengthDisplay::of(&h.project)
        });
    let unit = display.suffix();
    // O que o painel vai dizer, pela MESMA porta que o desenha.
    let span = display.text(2.0 * BEAM_HX, 0.1);
    let median_x = display.text(0.0, 0.1);
    let bottom_y = display.text(-BEAM_HY, 0.1);
    let target = display.text(TARGET_X, 0.1);

    eprintln!("[node-xy-smoke] cena montada: {n} formas, {nodes} nos, modo NODE.");
    eprintln!(
        "[node-xy-smoke] (!) se nao forem 2 formas e 8 nos, PARE: a cena perdeu a premissa e o \
         resto do roteiro nao mede nada."
    );
    eprintln!(
        "[node-xy-smoke] escala do projeto: {:.0} px/m, unidade {unit}.",
        display.pixels_per_meter
    );
    eprintln!(
        "[node-xy-smoke] os dois nos de BAIXO da viga distam **{span} {unit}**, e a mediana \
         deles esta' em X = **{median_x} {unit}**, Y = **{bottom_y} {unit}**."
    );
    eprintln!("[node-xy-smoke] o roteiro:");
    eprintln!(
        "  1. Pegue a ferramenta VECTOR. A cena ja' abre no modo NODE (seta branca) -- as \
         ancoras da viga aparecem."
    );
    eprintln!(
        "  2. Arraste um retangulo de selecao sobre os DOIS nos de BAIXO da viga. A secao \
         Vertex do painel ganha duas fileiras: **X** e **Y**."
    );
    eprintln!(
        "     X tem de dizer **{median_x}** e Y **{bottom_y}** -- a MEDIANA do par, e nao a \
         coordenada de um deles."
    );
    eprintln!(
        "  3. Digite **{target}** no X e Enter. Os dois nos andam JUNTOS: a viga inclina para a \
         direita e a mediana pousa sobre o quadrado AMBAR."
    );
    eprintln!(
        "     (!) Os dois tem de continuar a **{span} {unit}** um do outro. Se colapsarem numa \
         coluna so', PARE: e' o alinhar disfarcado de coordenada, o defeito que esta wave recusa."
    );
    eprintln!(
        "  4. Clique num no' SOZINHO. Agora X e Y leem aquele no' exatamente, e digitar poe-no \
         exatamente ali -- o caso simples le' como o Illustrator, porque a mediana de um e' ele."
    );
    eprintln!(
        "  5. Arraste o no' com o MOUSE. Os dois numeros seguem a mao -- eles sao uma leitura \
         do documento, nao um campo que so' aceita escrita."
    );
    eprintln!(
        "  6. O CONTROLE: menu **Settings > Unit > Meters**. Os dois numeros mudam de unidade \
         junto com a REGUA e com o Transform, e digitar naquela unidade pousa no mesmo lugar. \
         Se so' um deles mudar, sao duas portas outra vez."
    );
    eprintln!(
        "  7. Clique no VAZIO (nenhum no' selecionado): as duas fileiras SOMEM. Um par de caixas \
         dizendo 0, 0 afirmaria que a selecao esta' na origem."
    );
}
