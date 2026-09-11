//! **A cena pronta para o smoke do MULTIPLANO 2.5D** (`PH2D_FLIP_MULTIPLANE_SMOKE=1`,
//! ADR-0114 §Decisão 3).
//!
//! A paralaxe só se VÊ ao **panhar a câmera**: parado, os três planos coincidem
//! (a âncora é a origem do objeto). A cena é uma **paisagem em três planos**, de
//! trás para a frente:
//!
//! - **Céu** (`depth 0.15`) — uma serra baixa e pálida atravessando o fundo. Mal
//!   se move ao panhar (é o horizonte distante).
//! - **Árvore** (`depth 0.50`) — uma árvore verde no meio. Acompanha o pan pela
//!   METADE.
//! - **Cerca** (`depth 1.00`, flat = o comum) — postes de cerca saturados na
//!   frente, embaixo. Corre com a câmera à velocidade CHEIA.
//!
//! **Números MEDIDOS** (sonda headless, janela 1280×720, `height_world` 10, pan de
//! **3 unidades de mundo**): a Cerca desliza **216 px**, a Árvore **108 px** (½), o
//! Céu **32,4 px** (0,15×). O deslocamento é **`depth × pan`**, exato — é o que os
//! gates `the_far_layer_lags_the_near_one_under_pan` e `..._pins_to_the_camera`
//! provam. Panhar separa os planos nessa proporção; centrar a câmera na origem os
//! reúne.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_MULTIPLANE_SMOKE").is_some())
}

/// Um traço grosso a partir de uma lista de pontos (uma polilinha), na cor dada.
fn line(points: &[(f32, f32)], width: f32, colour: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in points {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: 1.0,
            color: colour,
        });
    }
    s.hardness = 0.85;
    s
}

/// Um polígono PREENCHIDO (silhueta sólida — lê como plano, não como contorno).
fn filled(points: &[(f32, f32)], colour: Rgba) -> FlipStroke {
    let mut s = line(points, 0.06, colour);
    s.closed = true;
    s.fill = Some(Fill {
        color: colour,
        opacity: 1.0,
    });
    s
}

/// A serra do FUNDO: dois picos pálidos atravessando a largura, altos na tela.
fn mountains() -> FlipStroke {
    let sky = Rgba::new(0.62, 0.72, 0.86, 1.0);
    filled(
        &[
            (-2.4, 0.20),
            (-1.2, 0.95),
            (-0.1, 0.35),
            (1.1, 0.90),
            (2.4, 0.20),
        ],
        sky,
    )
}

/// A ÁRVORE do meio: um tronco marrom + uma copa verde triangular, em `x = 0`.
fn tree() -> (FlipStroke, FlipStroke) {
    let trunk = line(
        &[(0.0, -0.45), (0.0, 0.05)],
        0.16,
        Rgba::new(0.45, 0.30, 0.18, 1.0),
    );
    let canopy = filled(
        &[(-0.55, 0.05), (0.0, 0.70), (0.55, 0.05)],
        Rgba::new(0.28, 0.62, 0.34, 1.0),
    );
    (trunk, canopy)
}

/// A CERCA da frente: quatro postes saturados embaixo + uma travessa, em warm.
fn fence() -> FlipStroke {
    // Uma polilinha só: sobe/desce por cada poste e cruza a travessa — traço grosso.
    let warm = Rgba::new(0.90, 0.55, 0.22, 1.0);
    line(
        &[
            (-1.2, -1.05),
            (-1.2, -0.35),
            (-0.4, -0.35),
            (-0.4, -1.05),
            (-0.4, -0.55),
            (0.4, -0.55),
            (0.4, -1.05),
            (0.4, -0.35),
            (1.2, -0.35),
            (1.2, -1.05),
        ],
        0.14,
        warm,
    )
}

/// **Monta a cena inteira** — porta única (o gate/mensagem encenam por aqui). As
/// camadas nascem de TRÁS para a FRENTE (índice 0 = fundo); a última (Cerca) é a
/// ATIVA, que é o fallback do bridge. Cada plano é um único quadro estático — a
/// paralaxe é do PAN, não da animação. Devolve as três `(nome, depth)` autoradas.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> [(&'static str, f32); 3] {
    obj.fps = 12.0;
    // Sem fantasmas: um único quadro por plano, e o onion sujaria a leitura da
    // paralaxe (o assunto aqui é o PAN, não o inbetween).
    obj.onion.enabled = false;

    // FUNDO — a serra (depth 0.15).
    let far = obj.add_layer("Ceu");
    if let Some(d) = obj.insert_frame(far, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho")
            .strokes
            .push(mountains());
    }
    if let Some(l) = obj.layer_mut(far) {
        l.depth = 0.15;
    }

    // MEIO — a árvore (depth 0.50).
    let mid = obj.add_layer("Arvore");
    if let Some(d) = obj.insert_frame(mid, 0, Hold::Implicit, KeyKind::Keyframe) {
        let (trunk, canopy) = tree();
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(trunk);
        strokes.push(canopy);
    }
    if let Some(l) = obj.layer_mut(mid) {
        l.depth = 0.50;
    }

    // FRENTE — a cerca (depth 1.0 = flat, o comum). É a ativa (criada por último).
    let near = obj.add_layer("Cerca");
    if let Some(d) = obj.insert_frame(near, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d).expect("desenho").strokes.push(fence());
    }
    // depth fica no default 1.0.

    [("Ceu", 0.15), ("Arvore", 0.50), ("Cerca", 1.00)]
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_multiplane_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Multiplane Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        let planes = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[multiplane-smoke] cena montada: 3 planos {planes:?} (nome, depth). \
             Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela a paralaxe nao e dirigida pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[multiplane-smoke] cena montada'? Se NAO,\n\
             PARE: o smoke nao rodou (arvore ou variavel de ambiente\n\
             errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: uma PAISAGEM em tres planos.\n\
               - FUNDO  : uma serra AZUL-PALIDA atravessando o alto.\n\
               - MEIO   : uma ARVORE verde no centro.\n\
               - FRENTE : uma CERCA laranja embaixo (4 postes).\n\
             Parado, os tres estao alinhados (a camera esta sobre a\n\
             origem, e ai todos os planos coincidem -- e' o certo).\n\
             \n\
             ------------------------------------------------------------\n\
             O TESTE: de PAN na camera (arraste o fundo para o lado, ou\n\
             use o gesto de pan do app) para a ESQUERDA e para a DIREITA.\n\
             ------------------------------------------------------------\n\
             A CERCA (frente) deve correr RAPIDO com a camera; a ARVORE\n\
             (meio) na METADE da velocidade; a SERRA (fundo) quase nao se\n\
             move. Ao panhar 3 unidades de mundo isso e', medido:\n\
                 Cerca  216 px  |  Arvore  108 px  |  Ceu  32 px\n\
             ou seja o deslocamento e' EXATAMENTE depth x pan.\n\
             Se os tres se movem JUNTOS (mesma velocidade), a paralaxe\n\
             esta quebrada -- me diga.\n\
             \n\
             ------------------------------------------------------------\n\
             O AJUSTE: no painel Flip, no bloco de cada camada, ha um\n\
             segundo slider abaixo da Opacity -- e' o DEPTH. Arraste o\n\
             Depth do 'Ceu' para 100% e panhe: agora a serra corre junto\n\
             com a cerca (virou flat). Volte para ~15% e ela volta a\n\
             ficar para tras. Cada camada tem o seu.\n"
        );
    }
}
