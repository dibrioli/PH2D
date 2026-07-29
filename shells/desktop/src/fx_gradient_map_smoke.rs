//! **GRADIENT MAP** (plano 24 W11) — a cena `PH2D_BUILD_SMOKE=39`.
//!
//! Irmã da `fx_duotone_smoke` (=38) e das outras, no mesmo molde: um **A/B**, não um catálogo.
//!
//! ⚠️ **A arte tem de ter VARIAÇÃO de brilho, e isso é a lei, não a cena** — este degrau pergunta
//! *quão claro é este texel* e mapeia a resposta numa rampa. Sobre uma chapa de cor sólida a
//! resposta é a mesma em todo lado. Por isso cada forma leva um **Bevel** antes: é o degrau que esta
//! pilha já tem para esculpir claro-e-escuro dentro de uma silhueta chapada.
//!
//! ⚠️ **O par 1 é o que carrega a wave:** um Gradient Map de DOIS stops nas pontas é o Duotone **ao
//! byte** (medido: 0 de 6144 bytes diferem, em três opacidades), e é isso que faz desta wave uma
//! generalização em vez de um segundo efeito que responde à mesma pergunta.
//!
//! Rodar: `cd <worktree> && env PH2D_BUILD_SMOKE=39 cargo run -p ph2d-host-desktop --release`.

use ph2d_ecs::{FxOp, VecFilter};
use ph2d_vec_scene::{ShapeKind, VecPathId};

use crate::build_smoke::shape;

/// Os pontos de estrela — os mesmos das cenas irmãs.
const STAR_V: &[f64] = &[5.0, 0.45, 0.0];

const COLS: [f64; 4] = [-3.6, -1.95, -0.3, 1.35];
const ROWS: [f64; 2] = [1.2, -1.0];
const SIDE: f64 = 1.35;
/// Quantas estrelas a cena monta: quatro pares.
const STARS: usize = 8;
/// A cor das estrelas — cinza claro, para o relevo do Bevel se ler sem competir com a rampa.
const STONE: [u8; 3] = [200, 200, 200];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => arm(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    for i in 0..STARS {
        let x = COLS[i % 4];
        let y = ROWS[i / 4];
        gfx.vec_scene.push_path(shape(
            ShapeKind::Star,
            [x, y],
            [x + SIDE, y + SIDE],
            STAR_V,
            STONE,
        ));
    }
}

/// O degrau que dá VARIAÇÃO de brilho à silhueta chapada — ver o cabeçalho.
fn bevel() -> FxOp {
    FxOp {
        radius: 0.12,
        opacity: 1.0,
        ..FxOp::new(FxOp::BEVEL)
    }
}

/// O Duotone, para o par de subsunção.
fn duotone(shadow: [f32; 4], highlight: [f32; 4]) -> FxOp {
    FxOp {
        color: shadow,
        color_b: highlight,
        opacity: 1.0,
        ..FxOp::new(FxOp::DUOTONE)
    }
}

/// Uma rampa autorada. As cores chegam em `[0,1]` retas, como a swatch as escreve; o **alfa é a
/// FORÇA** daquele stop.
///
/// ⚠️ As posições chegam na ordem que o autor quiser — quem ordena é o consumidor
/// (`FxOp::ramp_for_device`), e a cena EXERCITA isso na estrela 4.
fn ramp(stops: &[([f32; 4], f32)], mode: u8) -> FxOp {
    let mut op = FxOp {
        opacity: 1.0,
        mode,
        ..FxOp::new(FxOp::GRADIENT_MAP)
    };
    op.stop_count = u8::try_from(stops.len()).unwrap_or(2);
    for (i, (colour, pos)) in stops.iter().enumerate() {
        op.stops[i] = *colour;
        op.stop_pos[i] = *pos;
    }
    op
}

fn arm(app: &mut crate::App) {
    let ids: Vec<VecPathId> = app
        .gfx
        .as_ref()
        .expect("gfx")
        .vec_scene
        .paths()
        .iter()
        .map(|p| p.id)
        .collect();
    if ids.len() < STARS {
        eprintln!(
            "[smoke] fx-gradient-map: as oito estrelas ainda nao existem — o `sync` nao correu"
        );
        return;
    }
    // O par frio→quente que o Duotone usa, para o par de subsunção comparar a MESMA rampa.
    let cool = [0.10, 0.12, 0.35, 1.0];
    let warm = [1.0, 0.86, 0.62, 1.0];
    // Uma rampa de QUATRO stops — o que a wave acrescenta, e o que duas pontas não expressam.
    let sunset: &[([f32; 4], f32)] = &[
        ([0.05, 0.02, 0.20, 1.0], 0.0),
        ([0.75, 0.15, 0.35, 1.0], 0.4),
        ([1.0, 0.62, 0.20, 1.0], 0.72),
        ([1.0, 0.98, 0.88, 1.0], 1.0),
    ];
    // A MESMA rampa autorada FORA DE ORDEM — o desenho tem de sair idêntico ao da estrela 3.
    let scrambled: &[([f32; 4], f32)] = &[
        ([1.0, 0.62, 0.20, 1.0], 0.72),
        ([0.05, 0.02, 0.20, 1.0], 0.0),
        ([1.0, 0.98, 0.88, 1.0], 1.0),
        ([0.75, 0.15, 0.35, 1.0], 0.4),
    ];
    // Uma rampa com um stop de FORÇA ZERO no meio: ali a arte fica intocada.
    let gap: &[([f32; 4], f32)] = &[(cool, 0.0), ([0.9, 0.2, 0.2, 0.0], 0.5), (warm, 1.0)];
    // Uma rampa POSTERIZADA — dois stops colados: o degrau é a feature, e é o que uma rampa de
    // duas pontas não consegue desenhar.
    let poster: &[([f32; 4], f32)] = &[
        ([0.08, 0.10, 0.30, 1.0], 0.0),
        ([0.08, 0.10, 0.30, 1.0], 0.48),
        ([0.95, 0.95, 0.85, 1.0], 0.52),
        ([0.95, 0.95, 0.85, 1.0], 1.0),
    ];

    let stacks: [(usize, Vec<FxOp>); STARS] = [
        // Par 1 — A SUBSUNÇÃO: o Duotone contra um Gradient Map de dois stops nas pontas.
        (0, vec![bevel(), duotone(cool, warm)]),
        (1, vec![bevel(), ramp(&[(cool, 0.0), (warm, 1.0)], 0)]),
        // Par 2 — O que a wave ACRESCENTA: quatro stops, e a MESMA rampa fora de ordem.
        (2, vec![bevel(), ramp(sunset, 0)]),
        (3, vec![bevel(), ramp(scrambled, 0)]),
        // Par 3 — Linear contra Smooth: o easing é por SEGMENTO, e é nos stops internos que ele
        // se vê (medido: a inclinação no stop do meio cai de 24,0 para 6,0 níveis/6 px).
        (4, vec![bevel(), ramp(sunset, 0)]),
        (5, vec![bevel(), ramp(sunset, 1)]),
        // Par 4 — a FORÇA por-stop, e a rampa POSTERIZADA.
        (6, vec![bevel(), ramp(gap, 0)]),
        (7, vec![bevel(), ramp(poster, 0)]),
    ];
    let map = &app.vec_entities;
    let sim = &mut app.gfx.as_mut().expect("gfx").sim;
    for (i, ops) in stacks {
        crate::fx_live::set_filter(sim, map, &[ids[i]], Some(VecFilter { ops }));
    }
    eprintln!(
        "[smoke] GRADIENT MAP (plano 24 W11) — a rampa de N stops. Toda estrela leva um BEVEL\n\
         \x20antes, porque uma chapa de cor solida nao tem variacao de brilho, e sem variacao esta\n\
         \x20lei nao tem o que ler.\n\
         \x20\n\
         \x20 FILEIRA 1:\n\
         \x20  1) DUOTONE (sombra fria -> luz quente) — o degrau da W9.\n\
         \x20  2) GRADIENT MAP com DOIS stops nas MESMAS pontas. As duas tem de ser\n\
         \x20     INDISTINGUIVEIS: medido no device, **0 de 6144 bytes diferem**, em tres\n\
         \x20     opacidades. E isso que faz desta wave uma generalizacao, e nao um segundo efeito\n\
         \x20     que responde a mesma pergunta.\n\
         \x20  3) QUATRO stops (noite -> magenta -> ambar -> creme) — o que duas pontas nao\n\
         \x20     expressam, e a wave inteira.\n\
         \x20  4) A MESMA rampa AUTORADA FORA DE ORDEM (os quatro stops embaralhados). Tem de sair\n\
         \x20     identica a 3: o documento guarda a ordem de AUTORIA (indice estavel por alca) e\n\
         \x20     quem consome ordena uma copia.\n\
         \x20\n\
         \x20 FILEIRA 2:\n\
         \x20  5) A rampa de 4 stops em LINEAR (o default).\n\
         \x20  6) A MESMA em SMOOTH: o easing e por SEGMENTO, entao a rampa ACHATA em cada stop\n\
         \x20     interno — medido, a inclinacao no stop do meio cai de 24,0 para 6,0 niveis/6 px.\n\
         \x20     ⚠️ Corolario: em Smooth acrescentar um stop NAO pode ser neutro (dividir um\n\
         \x20     segmento reforma a curva, 25 niveis) — em Linear ele e neutro ao byte.\n\
         \x20  7) FORCA por-stop: o stop do MEIO tem alfa ZERO, e ali a arte fica intocada\n\
         \x20     (medido: 3 niveis no zero contra 97 na ponta opaca). Sem isso o alfa de um stop\n\
         \x20     do meio seria um knob morto.\n\
         \x20  8) POSTERIZADA — dois pares de stops colados: o DEGRAU e a feature. Uma rampa de\n\
         \x20     duas pontas nao desenha isto.\n\
         \x20\n\
         \x20 NO PAINEL (e o que fecha o smoke): selecione a estrela 3 e abra FILTERS. O card\n\
         \x20 'Gradient Map' tem um TRILHO — uma barra com a rampa e um punho por stop:\n\
         \x20  a) ARRASTE um punho: a rampa segue o dedo. Arraste-o POR CIMA do vizinho e continue\n\
         \x20     — o punho sob o dedo NAO pode trocar de stop no meio do gesto.\n\
         \x20  b) Clique um punho e depois a swatch **Stop**: o picker OKLCH abre e a cor tem de\n\
         \x20     pousar NAQUELE stop (nao na primeira cor do card).\n\
         \x20  c) O botao **+**: um stop novo nasce no maior VAO com a cor que a rampa JA tem ali —\n\
         \x20     ele nao pode mudar o desenho, so acrescentar um ponto de controle.\n\
         \x20  d) O botao **-**: remove o stop em foco, e PARA em dois (uma rampa com menos de duas\n\
         \x20     pontas nao e uma rampa).\n\
         \x20 E confira que a BARRA concorda com o desenho: ela e amostrada da MESMA funcao que o\n\
         \x20 dispositivo honra (paridade medida em 1 nivel de byte)."
    );
}
