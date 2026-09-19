//! **A TINTA do gizmo de uma corrente de posições** — a geometria vive em [`super::ponto_gizmo`];
//! aqui mora só o desenho, com o **vocabulário que o artista já aprendeu** (as constantes do
//! `warp_overlay`: a mesma cor, o mesmo casing escuro, a mesma espessura). *Um manipulador que ele
//! já leu noutro sítio não se reaprende.*
//!
//! ## As três feições
//!
//! | feição | o que se vê |
//! |---|---|
//! | **Osso** | a silhueta de armadura: larga na junta que manda, afilada para a ponta, com um anel na junta |
//! | **Corda** | a polilinha dos consecutivos, com uma marca em cada nó |
//! | **Ponto** | uma CRUZ e — **se o grafo der direcção** — a agulha que a mostra |
//!
//! ⚠️ **O osso é um LOSANGO afilado e não uma linha**, e é isso que faz uma cadeia ler-se como um
//! esqueleto em vez de um arame: a direcção é visível sem se seguir a ordem dos pontos.
//!
//! ## ⭐⭐⭐ As DUAS leis que a ordem do dono de 2026-09-19 impõe
//!
//! > *«os gizmos devem ter tamanho absoluto (não relativo ao zoom) e precisam responder aos grafos
//! > (como o scale do oscilador). Ou seja, eles não aparecem em runtime mas no canvas simulam
//! > qualquer grafo normalmente.»*
//!
//! ⭐⭐⭐ **AS DUAS SÃO UMA CONTA SÓ, e ela é a [`pegada_px`]:** *o tamanho que esta peça teria na
//! tela com o zoom de FÁBRICA*. Ela é proporcional ao `size` que o grafo escreve (lei 2) e divide
//! pela altura de referência da câmara e **nunca** pela de agora (lei 1).
//!
//! **1. TAMANHO ABSOLUTO.** Todo glifo é construído já em coordenadas de TELA e traçado com
//! `Affine::IDENTITY` — `stroke` **multiplica** a espessura pelo transform (a lei do cabeçalho do
//! `warp_overlay`). ⚠️ **O que SEGUE o zoom é a GEOMETRIA** — onde as juntas estão, quão comprido é
//! um osso, por onde a corda passa —, e isso é obrigatório: elas são factos de MUNDO.
//!
//! **2. O GLIFO RESPONDE AO GRAFO.** O `size` dá a pegada e a `rot` dá a **agulha da direcção** —
//! é isso que faz o `Scale` de um `motion.oscillator` PULSAR e o `Rotation` GIRAR num canvas sem
//! forma nenhuma ligada.
//!
//! ⛔⛔ **E a 1.ª tentativa falhou nas TRÊS coisas ao mesmo tempo por ler o `size` como um
//! multiplicador de um pixel escolhido** — ver [`pegada_px`], que tem os números.
//!
//! ⛔ **A agulha só é desenhada quando a corrente TRAZ a coluna `rot`** — uma agulha a apontar para
//! a direita em toda a nuvem seria ruído sobre um grafo que nunca falou de direcção.
//!
//! ⛔ **O `tint` NÃO entra**, e é decisão declarada: o gizmo é chrome e a cor dele é o que o torna
//! legível sobre qualquer arte; uma corrente com alfa `0` apagaria o gizmo e o artista leria
//! *«o nó parou de funcionar»*, que é o defeito que esta wave inteira existe para não ter.

use super::ponto_gizmo::{Grupo, PontoGizmoView};
use super::warp_overlay::{CASE_PX, CASE_RGBA, HANDLE_RGBA, OUTLINE_PX};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// A tolerância com que um círculo vira curvas.
const CIRCULO_TOL_PX: f64 = 0.1;

/// Metade do braço da cruz de um ponto solto, em fracção da pegada.
const CRUZ_DA_PECA: f64 = 0.5;
/// O comprimento da agulha da direcção, em fracção da pegada.
const AGULHA_DA_PECA: f64 = 1.0;

/// **O PISO do glifo, e ele nomeia o recurso: a TOLERÂNCIA com que uma curva é achatada.**
///
/// Um círculo de raio abaixo dela não tem como virar segmentos — o caminho sai degenerado e o
/// traçador pode não emitir nada. ⭐ **Acima disso não é preciso piso nenhum**, e a razão está
/// medida no próprio desenho: o traço tem `OUTLINE_PX` de espessura mais o casing, logo um anel de
/// raio `0,1` ainda pinta uma marca de `~4 px` — *o que garante a visibilidade é a ESPESSURA, não
/// o raio*.
///
/// ⛔⛔ **A 1.ª redacção usava o `OUTLINE_PX` (`1,5`) e um gate reprovou-a:** com `size = 0,5` o
/// anel do ponto pede `1,0 px` e o piso devolvia `1,5` ⇒ **o gizmo deixava de responder ao grafo
/// exactamente na faixa que o artista usa**. *Um piso de legibilidade que morde no regime normal
/// não protege a legibilidade: revoga a lei.*
pub(crate) const GLIFO_MIN_PX: f64 = CIRCULO_TOL_PX;

/// ⭐⭐⭐ **A PEGADA DA PEÇA, em pixels — a âncora das duas leis, e ela é DERIVADA.**
///
/// É *«o tamanho que esta peça teria na tela com o zoom de fábrica»*:
/// `size × (altura da área / altura de referência da câmara)`.
///
/// ⛔⛔ **A 1.ª redacção usava o `size` como multiplicador DIRECTO de um pixel escolhido, e os
/// três relatos do dono de 2026-09-19 são essa escolha:** as cenas autoram `size` em **unidades de
/// MUNDO** — a `=120` usa `0,10`–`0,16` —, logo o glifo saía a **10 %–16 %** do símbolo e colapsava
/// num ponto de tinta do tamanho do próprio traço. Daí *«piorou os desenhos»*, daí *«não são
/// animados em scale»* (uma variação de `0,26` para `0,30 px` por baixo de um traço de `1,5` é
/// invisível), e daí a leitura de que *«continuam relativos ao zoom»*: sem glifo legível, o que
/// muda à vista é só o espalhamento — que é geometria, e essa **tem** de seguir o zoom.
///
/// ⚠️ **A ALTURA DE REFERÊNCIA é a da câmara de fábrica, LIDA dela** (`Camera2d::default()`), nunca
/// um literal: ela é o que faz uma peça autorada para se ver bem no arranque ter um glifo que se vê
/// bem. E **é a câmara de FÁBRICA e não a de agora** — é isso, e só isso, que mantém a lei 1: o
/// zoom do artista não entra nesta conta.
/// **Quantos pixels vale uma unidade de MUNDO no zoom de FÁBRICA.** A porta de que a pegada e o
/// comprimento de um osso saem — as duas têm de vir daqui, senão uma é absoluta e a outra não.
#[must_use]
pub(crate) fn ppu_de_referencia(altura_da_area: f64) -> f64 {
    altura_da_area / f64::from(Camera2d::default().height_world)
}

#[must_use]
pub(crate) fn pegada_px(escala: f32, altura_da_area: f64) -> f64 {
    let s = f64::from(escala);
    if !s.is_finite() {
        return 0.0;
    }
    s.abs() * ppu_de_referencia(altura_da_area)
}

/// ⚠️⚠️ **A REFERÊNCIA do tamanho absoluto é a IDENTIDADE da coluna `size`, e isto é ERRO DE
/// COMPILAÇÃO se ela deixar de ser `1`.** Um `assert!` de teste sobre uma const é dobrado pelo
/// compilador antes de correr; o que morde é esta linha.
const _: () = assert!(
    ph2d_nodegraph::attr::SIZE_IDENTITY[0] == 1.0 && ph2d_nodegraph::attr::SIZE_IDENTITY[1] == 1.0,
    "a pegada absoluta divide pela identidade da coluna `size`; se ela deixar de ser 1, \
     `pegada_absoluta` tem de a dividir explicitamente"
);

/// O glifo que ocupa `fracao` da pegada, com o piso de legibilidade.
#[must_use]
pub(crate) fn glifo_px(fracao: f64, pegada: f64) -> f64 {
    (fracao * pegada).max(GLIFO_MIN_PX)
}

/// **Desenha o gizmo publicado.** No-op sem grupos.
pub fn draw(
    v: &PontoGizmoView,
    camera: &Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`. Ela é lida UMA vez
    // e serve as duas coisas: o `to_screen` (onde) e a altura (quão grande é a pegada). Duas
    // leituras seriam duas respostas à mesma pergunta.
    let area = super::warp_gizmo::scene_window(center_split, full_window);
    let to_screen = camera.world_to_screen_affine(area);
    let tracos = caminhos(
        v,
        &|w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1])),
        f64::from(area.height),
    );
    if tracos.is_empty() {
        return;
    }
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    // ⚠️⚠️ **O caminho PREENCHIDO saiu com as feições que o usavam** (a silhueta do osso): uma
    // cruz é só traço. Um `fill` sobre um caminho aberto pintaria uma área que ninguém desenhou.
    //
    // O casing PRIMEIRO, mais grosso — a lei do `warp_overlay`.
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
        Affine::IDENTITY,
        &case,
        None,
        &tracos,
    );
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &brush,
        None,
        &tracos,
    );
}

/// ⭐⭐ **A PORTA ÚNICA que constrói o caminho** — os traços de todas as nuvens, já em pixels de
/// tela. Dois leitores: o [`draw`] e os gates.
///
/// ⚠️ **UM caminho e não um por grupo:** o Vello encoda um caminho de uma vez, e uma cena com
/// treze ilhas (a `=110`) pagaria treze codificações.
pub(crate) fn caminhos(
    v: &PontoGizmoView,
    pt: &dyn Fn([f32; 2]) -> Point,
    altura_da_area: f64,
) -> BezPath {
    let mut tracos = BezPath::new();
    for g in &v.grupos {
        // ⭐⭐⭐ **O TAMANHO ABSOLUTO É A BASE, E O GRAFO MODULA-A** — ver [`pegada_absoluta`],
        // que tem o report do dono (*«Se coloco o tamanho, para de animar»*) e o mecanismo.
        // ⛔ A 1.ª redacção fazia o número do artista GANHAR da peça, e ao fazê-lo deitava fora a
        // coluna `size` — que é justamente o que um oscilador anima.
        // ⚠️⚠️ **O TAMANHO ABSOLUTO SAIU com o param que o escrevia** (ordem do dono,
        // 2026-09-19: os controlos de gizmo saíram do cartão). O que fica é a pegada derivada da
        // PEÇA, que é a que responde ao `size` que o grafo autora — e é ela que faz o `scale` de
        // um oscilador PULSAR no canvas sem uma forma ligada.
        let peg = |i: usize| pegada_px(g.escala_em(i), altura_da_area);
        desenha_pontos(g, pt, &peg, &mut tracos);
    }
    tracos
}

fn desenha_pontos(
    g: &Grupo,
    pt: &dyn Fn([f32; 2]) -> Point,
    peg: &dyn Fn(usize) -> f64,
    tracos: &mut BezPath,
) {
    for (i, p) in g.pontos.iter().enumerate() {
        let c = pt(*p);
        let pegada = peg(i);
        let braco = glifo_px(CRUZ_DA_PECA, pegada);
        // ⭐⭐⭐ **A CRUZ, e só ela** — ordem do dono, 2026-09-19: *«coloque gizmos de pequenos
        // pontos visíveis para as posições dos nós»*.
        //
        // ⚠️⚠️ **Havia TRÊS formas escolhidas por um param do cartão, e elas saíram com ele.** O
        // dono mandou retirar os controlos de gizmo desses nós na mesma jornada; o que resta é o
        // que ele pediu de volta, e uma cruz é a marca que um editor usa para dizer *«uma posição
        // está aqui»* sem se confundir com nada que o produto desenhe.
        tracos.move_to(Point::new(c.x - braco, c.y));
        tracos.line_to(Point::new(c.x + braco, c.y));
        tracos.move_to(Point::new(c.x, c.y - braco));
        tracos.line_to(Point::new(c.x, c.y + braco));
        if let Some(graus) = g.rot_em(i) {
            // ⚠️ A coluna é em GRAUS — a unidade de ângulo autorada desta casa (a mesma conversão
            // que o lowering faz, e no mesmo sítio: a borda onde a base é construída).
            let (sin, cos) = f64::from(graus).to_radians().sin_cos();
            let r = glifo_px(AGULHA_DA_PECA, pegada);
            // ⚠️ O `y` da TELA cresce para baixo; o sinal aqui é o mesmo que a base do lowering
            // escreve (`[cos, sin, -sin, cos]`), senão a agulha giraria ao contrário da arte.
            tracos.move_to(c);
            tracos.line_to(Point::new(c.x + cos * r, c.y + sin * r));
        }
    }
}
