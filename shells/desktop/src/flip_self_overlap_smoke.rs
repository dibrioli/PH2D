//! **A cena pronta para o smoke do SELF OVERLAP** (`PH2D_FLIP_SELF_OVERLAP_SMOKE=1`, 03 §8).
//!
//! Auto-sobreposição com ACÚMULO: quando um traço cruza a si mesmo, a tinta ESCURECE no
//! cruzamento (marcador/nanquim expressivo) em vez de ficar a união chapada. Só se VÊ com
//! **opacidade < 1** (a tinta opaca já satura), então os dois traços são desenhados a
//! **opacity 0.5**.
//!
//! A cena põe o MESMO traço — um LAÇO que cruza a si mesmo UMA vez — lado a lado:
//!
//! - **ESQUERDA (OFF)** — `self_overlap = false`, o traço de sempre: o cruzamento fica
//!   IGUAL aos braços (a união chapada). É o default, byte-idêntico ao Flip de sempre.
//! - **DIREITA (ON)** — `self_overlap = true`: o nó do cruzamento fica MAIS ESCURO (duas
//!   camadas compostas), e a passagem mais NOVA aparece por cima.
//!
//! **Números MEDIDOS** (a sonda headless é o gate GPU `a_stroke_with_self_overlap_accumulates_
//! at_the_crossing`, cruzamento em X a opacity 0.5): o braço (1 passagem) fica em **~128 de
//! 255**; o cruzamento OFF **~128** (== braço, sem acúmulo), o cruzamento ON **~191** (= 0.75,
//! a composição `over` de 2 camadas: `0.5 + 0.5·(1−0.5)`). A quina afiada ON também acumula, por
//! design (bleed de marcador); curva suave NÃO (as fitas mitram/abutam).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_SELF_OVERLAP_SMOKE").is_some())
}

/// UM traço que cruza a si mesmo UMA vez (um laço "de rabo de porco"): a linha de baixo e a
/// descida final se cruzam num ponto NÃO-adjacente. `dx` desloca a cópia em x; `self_overlap`
/// liga o acúmulo. Opacity 0.5 — sem isso o acúmulo é invisível (tinta opaca já satura).
fn loop_stroke(dx: f32, self_overlap: bool) -> FlipStroke {
    let ink = Rgba::new(0.20, 0.55, 0.85, 1.0); // um azul de tinta, o mesmo nos dois
    let mut s = FlipStroke::new();
    // O laço: entra pela base, sobe pela direita, curva por cima para a esquerda e DESCE
    // cruzando a própria base. O cruzamento cai em ~(dx-0.13, -0.3), não-adjacente.
    for &(x, y) in &[
        (-1.0, -0.3),
        (0.3, -0.3),
        (0.9, 0.5),
        (0.2, 0.9),
        (-0.4, 0.4),
        (0.1, -0.9),
    ] {
        s.push_point(Point {
            pos: Vec2::new(x + dx, y),
            width: 0.12,
            opacity: 0.5,
            color: ink,
        });
    }
    s.hardness = 0.9;
    s.self_overlap = self_overlap;
    s
}

/// **Monta a cena** — porta única (o gate/mensagem encenam por aqui). Uma camada, um quadro:
/// o laço OFF à esquerda, o laço ON à direita. Devolve `(x_off, x_on)` das duas cópias.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> (f32, f32) {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do nó.

    let (x_off, x_on) = (-1.7, 1.7);
    let layer = obj.add_layer("Self Overlap");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(loop_stroke(x_off, false)); // ESQUERDA: OFF (o de sempre)
        strokes.push(loop_stroke(x_on, true)); // DIREITA: ON (acumula)
    }
    (x_off, x_on)
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_self_overlap_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Self Overlap Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let (x_off, x_on) = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[self-overlap-smoke] cena montada: 2 lacos a opacity 0.5 -- OFF em x={x_off}, \
             ON em x={x_on}. Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[self-overlap-smoke] cena montada'? Se NAO,\n\
             PARE: o smoke nao rodou (arvore ou variavel de ambiente\n\
             errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: o MESMO laco (um traco que cruza a si\n\
             mesmo uma vez), a opacity 0.5, desenhado DUAS vezes:\n\
               - ESQUERDA : Self Overlap OFF -- o no do cruzamento fica\n\
                            IGUAL aos bracos (a uniao chapada de sempre).\n\
               - DIREITA  : Self Overlap ON  -- o no do cruzamento fica\n\
                            MAIS ESCURO (duas camadas de tinta compostas),\n\
                            e a passagem mais NOVA aparece por cima.\n\
             \n\
             ------------------------------------------------------------\n\
             Medido (sonda headless, cruzamento a opacity 0.5):\n\
                 braco (1 passagem)   ~128 de 255\n\
                 cruzamento OFF       ~128  (== braco, sem acumulo)\n\
                 cruzamento ON        ~191  (0.75 = 2 camadas 'over')\n\
             ------------------------------------------------------------\n\
             \n\
             O AJUSTE: no painel Flip (modo DRAW), abaixo da linha Tip,\n\
             ha o toggle 'Self Overlap'. Desenhe um rabisco que volte\n\
             sobre si mesmo com opacidade < 1: ligado, o cruzamento\n\
             escurece; desligado, fica a uniao chapada. Uma QUINA afiada\n\
             tambem escurece com ele ligado (bleed de marcador, por\n\
             design); a curva suave NAO. Se algo disso destoar, me diga.\n"
        );
    }
}
