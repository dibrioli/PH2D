//! ⭐⭐ **O GIZMO DE NAVEGAÇÃO** — as seis bolas de eixo no canto da janela 3D (W49).
//!
//! # A escolha, e a pesquisa que a decidiu (Enio, 2026-08-23)
//!
//! Enio pediu um gizmo moderno *"como o do Fusion"* e mandou o do Blender em anexo, com a instrução
//! certa: **pesquisar antes de construir**. Duas famílias, e a divisão é limpa:
//!
//! | | quem usa | gesto |
//! |---|---|---|
//! | **ViewCube** (cubo) | Fusion 360, AutoCAD, Inventor, Maya, Onshape, SolidWorks | clica face/aresta/canto; arrasta orbita |
//! | **bolas de eixo** | Blender, Unity, Godot, Cinema 4D, Plasticity | clica a bola; arrasta orbita |
//!
//! ⛔ **O ViewCube está sob patente VIVA:** [US 7.782.319] (*"Three-dimensional orientation
//! indicator and controller"*, Autodesk, depositada 2007-03-28), **ativa até 2029-03-06**, com uma
//! família à volta ([US 8.314.789], [US 9.021.400]).
//!
//! ⭐⭐ **E o que faz o widget funcionar não é o formato — é o ARRASTO.** A própria pesquisa da
//! Autodesk que criou o ViewCube mediu que os utilizadores são *"quase 2× mais rápidos"* a arrastar
//! do que a clicar, **«independentemente das várias representações examinadas»**. ⇒ o ganho medido
//! não vem do cubo; vem de o widget ser uma alça que se puxa. É por isso que aqui **arrastar orbita**
//! e o clique é o caminho secundário — e não o contrário.
//!
//! Decisão do Enio, com os dois factos na mão: **bolas de eixo**.
//!
//! # ⚠️ Os eixos são os NOSSOS
//!
//! Este módulo é **Y para cima**; o Fusion e o Blender são Z para cima. A bola `+Z` é a **frente**
//! aqui, e seria o topo lá. Cada bola **é** uma [`Standard`] da W47 — a mesma lista, a mesma lei de
//! reconhecimento —, então não há uma segunda ideia de *"o que é a vista de frente"*.
//!
//! # A lei mora aqui, os pixels no irmão
//!
//! A mesma separação do gizmo 3D: esta metade responde *"onde ficam as bolas e qual está sob o
//! cursor?"* sem janela nenhuma, e é ela que os gates dirigem.
//!
//! [US 7.782.319]: https://patents.google.com/patent/US7782319B2/en
//! [US 8.314.789]: https://patents.google.com/patent/US8314789B2/en
//! [US 9.021.400]: https://patents.google.com/patent/US9021400

use ph2d_editor::zones::Rect as EditorRect;
use ph2d_field_render::Orbit;

use crate::field3d_views::Standard;

/// O raio de uma bola, em pixels.
///
/// ⚠️ **Derivado, não escolhido**: é o [`crate::field3d_gizmo::GRAB_PX`] — *"a que distância do
/// traço um clique ainda é daquela alça"*. Um alvo de canvas deste módulo já tem um tamanho
/// declarado, com a razão ao lado; inventar um segundo seria ter duas ideias de *"o que é fácil de
/// acertar com o rato"* no mesmo widget.
pub(crate) const BALL_R_PX: f32 = crate::field3d_gizmo::GRAB_PX;

/// A distância do centro do gizmo a uma bola.
///
/// ⚠️ **Derivada do raio da bola**, e o critério é geométrico: duas bolas **opostas** (`+X` e `−X`)
/// ficam a `2·NAV_R` uma da outra, e o pior caso é o enquadramento em que um eixo aponta quase para
/// o observador — aí as duas colapsam para perto do centro. Com `4·BALL_R` de braço, dois eixos
/// perpendiculares na tela ficam a `4·BALL_R·√2 ≈ 5,7·BALL_R` de distância, folga de sobra; e o
/// widget inteiro cabe num quadrado de `2·(NAV_R + BALL_R) = 10·BALL_R`.
pub(crate) const NAV_R_PX: f32 = 4.0 * BALL_R_PX;

/// A folga entre o widget e a quina da área.
pub(crate) const NAV_MARGIN_PX: f32 = 2.0 * BALL_R_PX;

/// **Uma bola projetada**: que vista ela dá, onde está, e quão perto do observador.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ball {
    pub(crate) view: Standard,
    /// O centro, em pixels **no referencial da área desenhada**.
    pub(crate) at: [f32; 2],
    /// `> 0` quando o eixo aponta para o observador. É ele que decide a ordem de pintura, o realce
    /// e o desempate do clique.
    pub(crate) depth: f32,
}

/// O centro do gizmo, na quina **superior direita** da área — onde o Blender e o Unity o põem.
pub(crate) fn centre(area: EditorRect) -> [f32; 2] {
    let inset = NAV_R_PX + NAV_MARGIN_PX;
    [area.w - inset, inset]
}

/// ⭐ **As seis bolas, ORDENADAS DE TRÁS PARA A FRENTE.**
///
/// ⚠️ A ordem é a de pintura, e é o que faz o widget ler como um objeto sólido: pintar na ordem do
/// enum poria um eixo que está atrás por cima do que está à frente, e o artista perde a noção de
/// qual lado do modelo está a ver — que é a única coisa que este widget existe para dizer.
///
/// A projeção é a da câmera: um eixo do mundo `d` cai em `(d·direita, −d·cima)`, e a profundidade é
/// `d·frente`. ⚠️ O `y` da tela cresce **para baixo**, e é daí que vem o sinal do meio.
pub(crate) fn balls(cam: &Orbit, area: EditorRect) -> Vec<Ball> {
    let (right, up, fwd) = cam.basis();
    let c = centre(area);
    let mut out: Vec<Ball> = Standard::ALL
        .into_iter()
        .map(|view| {
            let d = view.eye_axis();
            let x = dot(d, right);
            let y = dot(d, up);
            Ball {
                view,
                at: [NAV_R_PX.mul_add(x, c[0]), NAV_R_PX.mul_add(-y, c[1])],
                depth: dot(d, fwd),
            }
        })
        .collect();
    // ⚠️ `total_cmp` e não `partial_cmp().unwrap()`: a ordem tem de ser total mesmo se um `NaN`
    // aparecer numa base degenerada — um `unwrap` ali entraria em pânico dentro da pintura.
    out.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    out
}

/// ⭐ **Que bola está sob o ponto** — `None` fora de todas.
///
/// ⚠️ **De frente para trás**, ao contrário da pintura: quando duas bolas se sobrepõem, a que se vê
/// é a da frente, e é essa que o clique tem de escolher. *A ordem de apontar é a inversa da de
/// desenhar*, e escrevê-las com a mesma seria o defeito clássico do gizmo que responde pelo eixo
/// escondido.
pub(crate) fn pick(balls: &[Ball], at: [f32; 2]) -> Option<Standard> {
    balls
        .iter()
        .rev()
        .find(|b| {
            let (dx, dy) = (at[0] - b.at[0], at[1] - b.at[1]);
            dx.hypot(dy) <= BALL_R_PX
        })
        .map(|b| b.view)
}

/// **O ponto está dentro do widget?** — a pergunta que o gesto faz antes de tudo.
///
/// ⚠️ Ela é do **widget inteiro**, não de uma bola: arrastar a partir de qualquer sítio dele orbita
/// (é o gesto que a medição da referência diz ser o rápido), e sem esta pergunta um arrasto começado
/// no vazio entre duas bolas seria um arrasto na **peça**.
pub(crate) fn hits_widget(area: EditorRect, at: [f32; 2]) -> bool {
    let c = centre(area);
    let (dx, dy) = (at[0] - c[0], at[1] - c[1]);
    dx.hypot(dy) <= NAV_R_PX + BALL_R_PX
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

#[cfg(test)]
#[path = "field3d_navball_tests.rs"]
mod tests;
