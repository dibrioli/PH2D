//! ⭐ **AS MEDIDAS DO GIZMO, EM PIXELS** — e cada uma diz de que RECURSO ela é.
//!
//! ⚠️ **O corte é por assunto e o ficheiro é quase todo doc**: metade destas constantes são
//! **derivadas** (o `MIN_ARM_PX` do agarre e da folga, o `RING_MIN_DOT` do agarre e do braço, o
//! `VERTEX_HALF_PX` do punho) e a razão de cada uma vale mais do que o número. O [`super`] passou as
//! `600` do gate de LOC do shell na W133 — ⛔ *split, nunca allowlist*.
//!
//! ⚠️ **Módulo-filho com `pub use ...::*` no pai**: todos os caminhos que já existiam
//! (`field3d_gizmo::GRAB_PX`, `::ARM_PX`, …) continuam a resolver, e os irmãos que fazem
//! `use super::*` continuam a vê-las.

/// **O comprimento do braço, EM PIXELS** — o gizmo tem tamanho de tela constante, como o do Blender.
///
/// ⚠️ Constante na tela e não no mundo, de propósito: um gizmo de tamanho de mundo fixo fica maior
/// do que a janela ao aproximar e some ao afastar, e é a mesma peça que se está a manipular nos dois
/// casos. O comprimento em mundo sai daqui dividido por [`Screen::px_per_world`].
pub const ARM_PX: f32 = 90.0;

/// A folga no centro. Nada é desenhado nem apontável dentro dela — é ela que separa as três setas
/// umas das outras e do disco de vista.
pub const INNER_PX: f32 = 15.0;

/// O raio de agarre: a que distância do traço um clique ainda é daquela alça.
pub const GRAB_PX: f32 = 9.0;

/// ⭐ **Meia-aresta do quadradinho de um vértice** (W133) — e ela é **DERIVADA**, não escolhida.
///
/// É metade do [`GRIP_HALF_PX`], o punho de tamanho. ⚠️ **A razão é a hierarquia do que se vê:** um
/// vértice é o menor sujeito manipulável desta janela — há `N` deles ao mesmo tempo, e eles pousam
/// **sobre a silhueta da peça**, onde competem com o contorno pela atenção. O punho é único e vive
/// no vazio. Um quadradinho do tamanho do punho faria uma peça de 12 vértices parecer uma fileira de
/// botões.
///
/// ⚠️ **O AGARRE é maior do que o desenho**, e é a mesma folga do punho (`+ GRAB_PX/2`): um alvo que
/// agarra exactamente onde pinta obriga a mão a acertar no pixel, e um vértice é o alvo mais pequeno
/// que esta janela oferece.
pub const VERTEX_HALF_PX: f32 = GRIP_HALF_PX * 0.5;

/// Comprimento e meia-largura da ponta da seta.
pub const HEAD_PX: f32 = 17.0;
pub const HEAD_HALF_W_PX: f32 = 5.5;

/// Espessura do traço da haste (e das argolas).
pub const SHAFT_HALF_W_PX: f32 = 1.3;

/// Onde fica o quadrado de plano, em fração do braço, e o lado dele.
pub const PLANE_AT: f32 = 0.38;
pub const PLANE_SIDE: f32 = 0.22;

/// ⚠️ **O comprimento projetado abaixo do qual uma seta deixa de ser uma alça** — e o número é
/// **derivado**, não escolhido.
///
/// Uma seta apontada para o observador projeta-se curta. A partir de certo ponto a região que a
/// agarra deixa de ser distinguível do centro: a haste começa em [`INNER_PX`] e o agarre tem
/// [`GRAB_PX`] de raio dos dois lados, então uma haste mais curta do que `INNER_PX + 2·GRAB_PX`
/// **não tem um único pixel que seja só dela**. Aí ela não é um controle — é uma lotaria entre três.
///
/// Escondê-la é o que o Blender faz, e o efeito colateral é bom: com a seta escondida sobra o
/// quadrado de plano perpendicular a ela, que é exatamente o gesto que aquele enquadramento pede.
pub const MIN_ARM_PX: f32 = INNER_PX + 2.0 * GRAB_PX;

/// Em quantos pedaços uma argola é amostrada. Ela é um **círculo do mundo**, e o que se pinta e se
/// aponta é a projeção dele — uma elipse, que só uma poligonal aproxima.
pub const RING_SEGMENTS: usize = 48;

/// ⚠️ **O quanto uma argola tem de estar virada para o observador** — também **derivado**.
///
/// Vista de perfil, uma argola projeta-se numa reta: o eixo menor da elipse mede
/// `ARM_PX · |cos θ|`, com θ o ângulo entre o eixo dela e a direção da vista. Abaixo de
/// [`GRAB_PX`] ela deixa de ser uma argola apontável e passa a ser um traço — e, pior, o arrasto
/// degenera junto (o plano de rotação fica de perfil e o raio do cursor não o encontra).
///
/// A saída existe e é a [`Handle::ViewRing`]: a argola do plano da tela nunca fica de perfil
/// consigo mesma.
pub const RING_MIN_DOT: f32 = GRAB_PX / ARM_PX;

/// ⚠️ **O piso que decide o que está «atrás», e ele nomeia o recurso: a precisão da representação.**
///
/// A argola de VISTA fica, por construção, **exatamente** no plano da câmera: a profundidade de todo
/// ponto dela é zero. Em `f32` esse zero sai como ±10⁻⁷ aleatório, e um teste `>= 0` transformaria a
/// argola numa fieira de pedaços soltos — medido (o gate `the_front_half_of_a_ring_is_one_unbroken_run`
/// apanhou-a a sair com **3 pontos** de 48).
///
/// 10⁻⁵ está duas ordens acima do ruído e cinco abaixo de qualquer fronteira real de meia-argola: o
/// pior que ele faz é deixar passar um segmento a mais na borda, que ninguém vê.
///
/// (É o irmão do `PRECISION_FLOOR` do traçador, e pelo mesmo motivo.)
pub const RING_FRONT_EPS: f32 = 1.0e-5;

/// O raio da argola de vista, em frações do braço. Ela fica **por fora** das três, como a branca do
/// Blender — é a de fora que se agarra sem pensar.
pub const VIEW_RING_R: f32 = 1.18;

/// Meia-aresta do punho de tamanho.
pub const GRIP_HALF_PX: f32 = 6.5;

/// ⚠️ **A direção do punho de tamanho é de TELA, e ela é cosmética.**
///
/// Ele não é um eixo — é um punho, como o canto de uma janela —, e a lei do arrasto depende só do
/// **raio** ao centro, nunca desta direção. Pô-lo em cima e à direita é a convenção de todo punho de
/// redimensionar; movê-lo para outro canto não mudaria uma linha da conta.
///
/// (`y` cresce para BAIXO em pixels, daí o sinal.)
pub const GRIP_DIR: [f32; 2] = [
    std::f32::consts::FRAC_1_SQRT_2,
    -std::f32::consts::FRAC_1_SQRT_2,
];
