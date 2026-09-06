//! ⭐⭐⭐ **N PREENCHIMENTOS E N CONTORNOS NUMA FORMA** — `PH2D_VEC_STACK_SMOKE=1` (estudo 42, item 4).
//!
//! # O que a cena fecha
//!
//! Até 2026-09-05 uma forma tinha **um** preenchimento e **um** contorno. Cada camada de estilo a
//! mais obrigava a **duplicar o objecto** — e duas cópias de uma forma são duas geometrias que
//! divergem no primeiro ponto que o artista mexe: ele corrige uma e a outra fica para trás.
//!
//! # A cena, e o que cada peça prova
//!
//! | O quê | O que ela prova |
//! |---|---|
//! | **A ETIQUETA**: uma estrela com contorno branco largo por baixo de um preto fino | a largura é POR CAMADA (é isto que o Figma não faz: lá as N tintas partilham uma geometria de traço) |
//! | **O CARRIL**: uma linha com três contornos de larguras decrescentes | a pilha ordena-se, e o de cima desenha por último |
//! | **A MISTURA**: um disco com um 2.º preenchimento em `Multiply` a 60 % | opacidade e mistura são de CADA camada, e compõem-se DENTRO da forma |
//! | **A SOMBRA**: um cartão com o 2.º preenchimento DESLOCADO | ONDE cada camada desenha é da CAMADA (v21) — sem isso dois preenchimentos ocupam os mesmos pixels |
//! | O par de referência ao lado | a mesma arte feita à moda antiga (duas formas empilhadas) — para se ver que agora é **uma** |
//!
//! ⚠️ Se a linha `[vec-stack-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_vec_scene::{
    BlendMode, Opacity, Paint, PaintEntry, Rgba8, ShapeKind, StrokeSpec, VecPath,
};

/// Uma forma da cena: a arte, e a pilha que ela leva.
fn com(mut p: VecPath, camadas: Vec<PaintEntry>) -> VecPath {
    p.paints = camadas;
    p
}

fn traco(rgb: [u8; 3], w: f64) -> PaintEntry {
    PaintEntry::stroke(StrokeSpec::new(Rgba8::new(rgb[0], rgb[1], rgb[2], 255), w))
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_stack_smoke(&mut self) {
        if self.vec_stack_smoke_done || std::env::var_os("PH2D_VEC_STACK_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec_stack_smoke_done = true;
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        let scene = &mut gfx.vec_scene;

        // Uma faixa escura ao fundo, para o branco da etiqueta e a mistura terem contra o que
        // compor. ⚠️ Sem ela o `Multiply` do disco seria invisível — a mesma lei da cena do item 2.
        scene.push_path(crate::build_smoke::shape(
            ShapeKind::Rectangle,
            [-9.0, -6.0],
            [9.0, 5.0],
            &[],
            [70, 80, 110],
        ));

        // ⭐ A ETIQUETA: UMA estrela, com o contorno branco largo POR BAIXO do preto fino.
        let estrela = crate::build_smoke::shape(
            ShapeKind::Star,
            [-8.0, 0.0],
            [-3.0, 4.5],
            &[5.0, 0.45, 0.0],
            [240, 200, 60],
        );
        scene.push_path(com(
            estrela,
            vec![traco([255, 255, 255], 0.55), traco([20, 20, 20], 0.18)],
        ));

        // ⭐ O CARRIL: uma linha com três contornos de larguras decrescentes.
        let linha = crate::build_smoke::shape(
            ShapeKind::Rectangle,
            [-1.5, 1.6],
            [8.0, 1.9],
            &[],
            [200, 60, 60],
        );
        scene.push_path(com(
            linha,
            vec![
                traco([30, 30, 40], 0.9),
                traco([230, 230, 235], 0.55),
                traco([200, 60, 60], 0.2),
            ],
        ));

        // ⭐ A MISTURA: um disco com um 2.º preenchimento em Multiply a 60 %.
        let disco = crate::build_smoke::shape(
            ShapeKind::Ellipse,
            [-1.0, -5.0],
            [4.0, 0.0],
            &[],
            [235, 215, 90],
        );
        let mut mistura = PaintEntry::fill(Paint::Solid(Rgba8::new(80, 160, 220, 255)));
        mistura.opacity = Opacity::new(0.6);
        mistura.blend = BlendMode::Multiply;
        scene.push_path(com(disco, vec![mistura]));

        // ⭐⭐⭐ A SOMBRA: UMA forma, e o 2.º preenchimento desenha DESLOCADO (v21).
        //
        // ⚠️ **A receita é INVERTIDA, e é isso que a cena ensina.** Uma camada extra desenha sempre
        // POR CIMA da base (a base é o chão da pilha, por desenho), então para uma sombra ATRÁS o
        // que se desloca é a *forma*, não a sombra: a BASE leva a cor da sombra e a camada extra
        // leva a cor viva, deslocada para cima e para a esquerda. O olho lê um cartão claro com
        // sombra dura em baixo à direita — que é o que se queria.
        let cartao = crate::build_smoke::shape(
            ShapeKind::Rectangle,
            [-8.2, -5.0],
            [-3.6, -1.4],
            &[],
            [30, 30, 45],
        );
        let mut viva = PaintEntry::fill(Paint::Solid(Rgba8::new(245, 235, 210, 255)));
        viva.offset = [-0.35, 0.35];
        scene.push_path(com(cartao, vec![viva]));

        // O par de REFERÊNCIA: a mesma etiqueta feita à moda antiga — duas formas empilhadas.
        let mut fundo = crate::build_smoke::shape(
            ShapeKind::Star,
            [4.5, -5.0],
            [9.5, -0.5],
            &[5.0, 0.45, 0.0],
            [240, 200, 60],
        );
        fundo.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.55));
        scene.push_path(fundo);
        let mut cima = crate::build_smoke::shape(
            ShapeKind::Star,
            [4.5, -5.0],
            [9.5, -0.5],
            &[5.0, 0.45, 0.0],
            [240, 200, 60],
        );
        cima.fill = None;
        cima.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.18));
        scene.push_path(cima);

        eprintln!(
            "[vec-stack-smoke] a ETIQUETA (1 forma, 2 contornos), o CARRIL (1 forma, 3 contornos), \
             a MISTURA (1 forma, 2 preenchimentos) e a SOMBRA (1 forma, o 2.o preenchimento \
             DESLOCADO) — mais o par de REFERENCIA a' direita, que e' \
             a mesma etiqueta em DUAS formas empilhadas. PEGUE a ferramenta Vector, clique numa \
             forma, e a seccao Appearance lista as camadas dela."
        );
    }
}
