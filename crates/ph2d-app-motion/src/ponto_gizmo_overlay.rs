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

use super::ponto_gizmo::{Feicao, Grupo, PontoGizmoView};
use super::warp_overlay::{CASE_PX, CASE_RGBA, HANDLE_RGBA, OUTLINE_PX, TANGENT_RGBA};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Shape, Stroke, VectorScene};

/// A tolerância com que um círculo vira curvas.
const CIRCULO_TOL_PX: f64 = 0.1;

/// **Que fracção da pegada da peça o corpo do osso ocupa.** Um osso mais gordo do que isto deixa
/// de se ler como armadura e vira um losango.
const OSSO_DA_PECA: f64 = 0.5;
/// **E ele nunca é mais gordo do que esta fracção do PRÓPRIO comprimento** — senão uma cadeia de
/// juntas juntas (o caso normal de uma corda em repouso) fica coberta de manchas.
const OSSO_DO_COMPRIMENTO: f64 = 0.35;
/// O raio do anel de uma junta, em fracção da pegada.
const JUNTA_DA_PECA: f64 = 0.30;
/// Metade do braço da cruz de um ponto solto, em fracção da pegada.
const CRUZ_DA_PECA: f64 = 0.5;
/// O comprimento da agulha da direcção, em fracção da pegada.
const AGULHA_DA_PECA: f64 = 1.0;

/// ⚠️ **Um osso curto demais desenha-se como uma JUNTA e mais nada** — e «curto» é medido em
/// pixels de REFERÊNCIA (o mundo no zoom de fábrica), nunca no ecrã de agora: com o ecrã, afastar a
/// câmara fazia a cadeia inteira DESAPARECER. Sem esta cerca, o losango de
/// um segmento de comprimento ~0 fica com as duas pontas do lado errado da largura e pinta uma
/// gravata — uma cadeia com juntas coincidentes (o caso normal de uma corda em repouso) ficaria
/// coberta de borrões.
const OSSO_MIN_PX: f64 = 8.0;

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
const GLIFO_MIN_PX: f64 = CIRCULO_TOL_PX;

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

/// ⭐⭐⭐ **A PEGADA ABSOLUTA — o número do artista como BASE, e o grafo como MULTIPLICADOR.**
///
/// > **Report do dono, 2026-09-19:** *«Se coloco o tamanho, para de animar.»*
///
/// ⛔⛔⛔ **Ele tinha razão, e era um defeito de DESENHO meu, não um bug.** A 1.ª redacção da
/// secção do gizmo fez o `Gizmo Size` **GANHAR** da peça: preenchido, ele *substituía* a pegada
/// derivada, e com ela ia embora a coluna `size` — que é exactamente o que um `motion.oscillator`
/// anima. ⇒ *o absoluto matava a animação*, e a §32.4-bis tinha declarado por escrito que as duas
/// leis não brigam.
///
/// ⭐ **As duas coisas que ele pediu são compatíveis, e a composição é a resposta:** o tamanho
/// absoluto é a pegada **na IDENTIDADE da corrente**, e a escala do grafo multiplica-a a partir
/// dali. Um grafo que não fala de escala entrega `1` e o glifo mede exactamente o que o artista
/// escreveu; um `motion.scale(0,4)` entrega `0,4` e o glifo fica a 40 %; um oscilador a pulsar
/// entre `0,8` e `1,2` faz o glifo **pulsar**, que é o pedido original.
///
/// ⚠️⚠️ **E a REFERÊNCIA é a identidade porque o nó que CARREGA este controlo emite na
/// identidade.** As 14 fontes de posições (`quem_e_como_o_grid`) emitem `P`/`Index`/`Count` e
/// **nenhuma coluna `size`** ⇒ toda `size` que o sink vê foi escrita **a jusante**, logo ela *é* a
/// modulação do grafo. Não é uma convenção escolhida: é uma propriedade da população, e o
/// `const _` acima prende-a.
///
/// ⛔⛔ **E é por isso que a referência NÃO pode ser uma estatística da corrente.** A mediana (ou a
/// média, ou o elemento `0`) **anularia uma pulsação UNIFORME** — se toda a nuvem pulsa junta,
/// `size_i / mediana` é `1` o tempo todo e o glifo fica parado, que é o defeito que esta função
/// existe para curar. *Uma referência tirada do mesmo instante cancela exactamente o que se quer
/// ver.*
///
/// ⚠️ **Um `size` não-finito devolve `0`**, como a [`pegada_px`] — o glifo desaparece em vez de
/// pintar um caminho degenerado.
#[must_use]
pub(crate) fn pegada_absoluta(tamanho_px: f32, escala: f32) -> f64 {
    let s = f64::from(escala);
    if !s.is_finite() {
        return 0.0;
    }
    // ÷ `SIZE_IDENTITY` (= 1) — ver o `const _` acima.
    f64::from(tamanho_px) * s.abs()
}

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
    let (cheios, tracos) = caminhos(
        v,
        &|w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1])),
        f64::from(area.height),
    );
    if cheios.is_empty() && tracos.is_empty() {
        return;
    }
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    // O casing PRIMEIRO, mais grosso — a lei do `warp_overlay`.
    for caminho in [&cheios, &tracos] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            caminho,
        );
    }
    vector_scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &dim, None, &cheios);
    for caminho in [&cheios, &tracos] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &brush,
            None,
            caminho,
        );
    }
}

/// ⭐⭐ **A PORTA ÚNICA que constrói os caminhos** — devolve `(o que se preenche, o que se traça)`,
/// já em pixels de tela. Dois leitores: o [`draw`] e os gates, que a chamam com DOIS `to_screen`
/// diferentes para medir a lei do tamanho absoluto.
///
/// ⚠️ **Dois caminhos e não um por grupo:** o Vello encoda um caminho de uma vez, e uma cena com
/// treze ilhas (a `=110`) pagaria treze codificações por feição.
pub(crate) fn caminhos(
    v: &PontoGizmoView,
    pt: &dyn Fn([f32; 2]) -> Point,
    altura_da_area: f64,
) -> (BezPath, BezPath) {
    let ppu = ppu_de_referencia(altura_da_area);
    let mut cheios = BezPath::new();
    let mut tracos = BezPath::new();
    for g in &v.grupos {
        // ⭐⭐⭐ **O TAMANHO ABSOLUTO É A BASE, E O GRAFO MODULA-A** — ver [`pegada_absoluta`],
        // que tem o report do dono (*«Se coloco o tamanho, para de animar»*) e o mecanismo.
        // ⛔ A 1.ª redacção fazia o número do artista GANHAR da peça, e ao fazê-lo deitava fora a
        // coluna `size` — que é justamente o que um oscilador anima.
        let peg = |i: usize| {
            let escala = g.escala_em(i);
            g.tamanho_em(i).map_or_else(
                || pegada_px(escala, altura_da_area),
                |t| pegada_absoluta(t, escala),
            )
        };
        match g.feicao {
            Feicao::Osso => desenha_ossos(g, pt, &peg, ppu, &mut cheios, &mut tracos),
            Feicao::Corda => desenha_corda(g, pt, &peg, &mut tracos),
            Feicao::Ponto => desenha_pontos(g, pt, &peg, &mut tracos),
        }
    }
    (cheios, tracos)
}

/// A silhueta de cada osso mais o anel de cada junta.
///
/// ⚠️ **A LARGURA responde ao `size` e a DIRECÇÃO não lê o `rot`** — ela já está na geometria: numa
/// cadeia o ângulo de cada junta é o que PÔS as posições onde elas estão (a cinemática já correu),
/// e aplicá-lo outra vez ao losango contaria a mesma rotação duas vezes.
fn desenha_ossos(
    g: &Grupo,
    pt: &dyn Fn([f32; 2]) -> Point,
    peg: &dyn Fn(usize) -> f64,
    ppu: f64,
    cheios: &mut BezPath,
    tracos: &mut BezPath,
) {
    for [de, para] in &g.segmentos {
        let (a, b) = (pt(g.pontos[*de]), pt(g.pontos[*para]));
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let comp = dx.hypot(dy);
        // ⛔⛔⛔ **O COMPRIMENTO QUE DECIDE A GORDURA É O DE MUNDO, medido no zoom de FÁBRICA** —
        // report do dono, 2026-09-19: *«os gizmos estão relativos ao zoom»*, e **ele tinha razão**.
        // A 1.ª redacção limitava a meia-largura por `comp`, que é o comprimento em pixels de
        // ECRÃ: afastar a câmara encolhia `comp`, o limite mordia, e o osso **afinava com o zoom**.
        // *Uma cerca em pixels de ecrã dentro de uma lei que se diz absoluta é a lei revogada.*
        let (wx, wy) = (
            f64::from(g.pontos[*para][0] - g.pontos[*de][0]),
            f64::from(g.pontos[*para][1] - g.pontos[*de][1]),
        );
        let comp_ref = wx.hypot(wy) * ppu;
        if comp_ref < OSSO_MIN_PX {
            continue; // ver `OSSO_MIN_PX`
        }
        if comp <= f64::EPSILON {
            continue; // no ecrã as duas juntas caíram no mesmo pixel: não há direcção a desenhar
        }
        // A meia-largura é a do FILHO: é o elemento que este osso representa. ⚠️ E é limitada
        // pelo PRÓPRIO comprimento — uma peça grande numa cadeia curta desenharia um losango mais
        // largo do que longo, que já não é um osso.
        let meia = glifo_px(OSSO_DA_PECA, peg(*para)).min(comp_ref * OSSO_DO_COMPRIMENTO);
        let (nx, ny) = (-dy / comp * meia, dx / comp * meia);
        // O ombro fica a um quinto do caminho: é onde a armadura do referencial o põe, e é o
        // que dá a direcção sem engordar a cadeia inteira.
        let ombro = Point::new(a.x + dx * 0.2, a.y + dy * 0.2);
        cheios.move_to(a);
        cheios.line_to(Point::new(ombro.x + nx, ombro.y + ny));
        cheios.line_to(b);
        cheios.line_to(Point::new(ombro.x - nx, ombro.y - ny));
        cheios.close_path();
    }
    for (i, p) in g.pontos.iter().enumerate() {
        anel(tracos, pt(*p), glifo_px(JUNTA_DA_PECA, peg(i)));
    }
}

/// A polilinha dos consecutivos, com uma marca em cada nó.
fn desenha_corda(
    g: &Grupo,
    pt: &dyn Fn([f32; 2]) -> Point,
    peg: &dyn Fn(usize) -> f64,
    tracos: &mut BezPath,
) {
    let mut aberto = false;
    for [de, para] in &g.segmentos {
        if !aberto {
            tracos.move_to(pt(g.pontos[*de]));
            aberto = true;
        }
        tracos.line_to(pt(g.pontos[*para]));
    }
    for (i, p) in g.pontos.iter().enumerate() {
        anel(tracos, pt(*p), glifo_px(JUNTA_DA_PECA * 0.7, peg(i)));
    }
}

/// **Uma CRUZ em cada posição** e — se o grafo der direcção — a agulha que a mostra.
///
/// ⛔⛔ **A cruz VOLTOU por veredito do dono** (2026-09-19: *«vc piorou os desenhos dos gizmos que
/// estavam bons»*): a 1.ª tentativa trocou-a por um anel, com o argumento de que *«uma cruz rodada
/// `90°` é a MESMA cruz»* — o argumento é verdadeiro e a conclusão era errada, porque **quem mostra
/// a rotação é a AGULHA e não a marca**. A cruz é o que diz *«aqui está um elemento»*, e é ela que
/// ele reconhece.
///
/// ⚠️ **A cruz não gira**: ela é chrome e mantém-se alinhada aos eixos; girá-la faria a marca
/// piscar de quarto em quarto de volta sem dizer nada que a agulha não diga melhor.
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
        // ⭐⭐⭐ **AS TRÊS FORMAS** (ordem do dono, 2026-09-19). ⚠️ A escada vem da crate que as
        // declara — um `match` sobre literais aqui seria a segunda resposta à mesma pergunta, e a
        // que envelhece no dia da quarta forma.
        match g.forma_em(i) {
            f if f >= ph2d_gizmo_params::RECT => {
                tracos.move_to(Point::new(c.x - braco, c.y - braco));
                tracos.line_to(Point::new(c.x + braco, c.y - braco));
                tracos.line_to(Point::new(c.x + braco, c.y + braco));
                tracos.line_to(Point::new(c.x - braco, c.y + braco));
                tracos.close_path();
            }
            f if f >= ph2d_gizmo_params::CIRCULO => anel(tracos, c, braco),
            _ => {
                tracos.move_to(Point::new(c.x - braco, c.y));
                tracos.line_to(Point::new(c.x + braco, c.y));
                tracos.move_to(Point::new(c.x, c.y - braco));
                tracos.line_to(Point::new(c.x, c.y + braco));
            }
        }
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

fn anel(caminho: &mut BezPath, c: Point, r: f64) {
    caminho.extend(Circle::new(c, r).path_elements(CIRCULO_TOL_PX));
}
