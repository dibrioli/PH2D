//! **A cena pronta para o smoke do TEXTURE PATTERN** — `PH2D_BUILD_SMOKE=76` (plano 33).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `knot_smoke`/`twist_smoke`.
//!
//! ⭐ **A arte é SINTETIZADA aqui**, e é de propósito: o smoke não pode pedir um ficheiro ao Enio.
//! Ele corre com um comando só, e a arte é assimétrica nos DOIS eixos (uma barra em cima e uma
//! meia-diagonal), porque um motivo simétrico não deixa ver desfasamento, espelho nem rotação.
//! ⚠️ E ela tem um quadrante **transparente**: um padrão só-opaco esconde a lei do alfa, que é onde
//! a família do Bug #4 do Motion vive.
//!
//! As seis formas da fileira de cima, da esquerda para a direita (e uma **sétima** em baixo):
//!
//! 1. **Grade** (o controlo, e o HERÓI já selecionado) — a repetição simples.
//! 2. **Tijolo 1/2** — as linhas desfasam-se meia célula. O ladrilho assado tem **duas** linhas.
//! 3. **Colmeia** — o mesmo desfasamento, mas com o espaçamento vertical `√3/2` que põe os seis
//!    vizinhos à mesma distância.
//! 4. **Espelho** — a cada repetição a arte inverte; a costura desaparece mesmo em arte não
//!    periódica.
//! 5. **Buraco** (composto, regra `EvenOdd`) — ⚠️ o padrão **não pode** pintar o buraco. Foi a
//!    pedra em que o `fill_multipoint` tropeçou, e o `VectorScene::fill_path` ainda tem o defeito.
//! 6. **Esticada** — a MESMA grade numa forma escalada só num eixo: o padrão **esmaga com ela**, ao
//!    contrário da caneta do traço (bug #27). As duas leis estão certas e são diferentes.
//! 7. ⭐⭐ **Em baixo: a arte é uma FORMA do documento** (W7) — o triângulo ao lado dela. Mexer nos
//!    nós do triângulo re-assa o ladrilho **na hora**, que é o *"pattern fills are dynamic"* do
//!    Figma. ⚠️ O motivo fica **visível de propósito**: escondê-lo é o gesto do olho na Hierarquia,
//!    e uma fonte invisível por omissão seria uma forma que o artista não sabe que tem.

use ph2d_vec_pattern::{PatternMode, TileKind};
use ph2d_vec_scene::{
    FillRule, Paint, PatternFill, PatternSource, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex,
};

/// O lado da arte, em pixels.
const ART: u32 = 32;
/// O lado de cada forma, em unidades de mundo.
const BOX: f64 = 2.2;
/// O passo entre formas.
const STEP: f64 = 2.6;
/// A largura do contorno, em unidades de mundo — a mesma ordem de grandeza dos outros smokes
/// vetoriais desta casa (`0,012`–`0,02`), subida porque aqui ela tem de **ler-se por cima de uma
/// arte com detalhe**, e não por cima de um preenchimento chapado.
const STROKE_W: f64 = 0.03; // LITERAL-PX-OK: largura no domínio do documento

/// ⛔⛔ **TODA forma desta cena NASCE COM CONTORNO** (Enio, 2026-08-27: *"o contorno funciona com as
/// shapes que eu desejo, mas não funcionam com os teus desenhos"*).
///
/// ⚠️ **Era o smoke, não o produto** — e foi o report que fechou uma caça de três mensagens. A
/// ferramenta de forma escreve `path.stroke = Some(..)` **sempre**
/// ([`ph2d_vec_edit`](../../../crates/ph2d-vec-edit/src/shape.rs)), então toda forma que o artista
/// desenha tem contorno; estas nasciam de `..VecPath::default()`, que é `stroke: None`. E o
/// `restyle_selected_strokes` **recusa por desenho** quem não tem um (*"ganhar um traço do nada
/// seria a UI inventando geometria"*) ⇒ a secção *Stroke* ficava **pintada e inerte** só aqui, o
/// que se lê exactamente como *"o padrão anulou o contorno"*.
///
/// ⚠️⚠️ **A lição é da CENA, não do padrão:** uma cena de smoke montada por código não herda o que
/// a ferramenta de autoria garante — *ela tem de nascer no estado em que o artista a encontraria*,
/// senão o smoke mede um objecto que o produto nunca produz.
fn contorno() -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(35, 35, 45, 255), STROKE_W)
}

/// A arte de referência: barra em cima, meia-diagonal, um quadrante transparente.
///
/// ⚠️ **Assimétrica nos dois eixos** — um motivo simétrico esconde desfasamento, espelho e rotação,
/// que são metade do que esta cena existe para mostrar.
fn art_rgba() -> Vec<u8> {
    let mut px = Vec::with_capacity((ART * ART * 4) as usize);
    for y in 0..ART {
        for x in 0..ART {
            let c = if y < ART / 8 {
                // A barra do topo: laranja opaco. É ela que denuncia uma rotação ou um espelho.
                [230u8, 140, 60, 255]
            } else if x + y < ART {
                // A meia-diagonal: azul opaco.
                [70, 120, 210, 255]
            } else if x > ART * 3 / 4 && y > ART * 3 / 4 {
                // ⚠️ O quadrante TRANSPARENTE — e com cor por baixo, que é o que todo PNG comum
                // tem. Um assador que componha `0/0` apaga este RGB e a grade deixa de ser
                // byte-idêntica sem que nenhum gate opaco dê por isso.
                [200, 40, 40, 0]
            } else {
                [235, 232, 225, 255]
            };
            px.extend_from_slice(&c);
        }
    }
    px
}

/// ⛔⛔ **ONDE UM PADRÃO DESTA CENA NASCE** — o canto da FORMA, nunca a origem do mundo.
///
/// ⚠️ É a mesma lei que o produto obedece (`texture_pattern_pick::default_placement` devolve o `lo`
/// da caixa) e que um report do Enio já pagou uma vez, em 27/08: com o `Clamp`, um ladrilho na
/// origem do mundo fazia a forma amostrar `uv` a centenas de texels e sair **em branco**.
///
/// ⚠️⚠️ **E numa FAIXA FINA ela morde mesmo em `Tile`** (report de 28/08): a fase do reticulado sob
/// um contorno passa a não ter relação nenhuma com a forma, então mover um nó desliza a arte por
/// baixo da linha e — em `Mirror` — troca a paridade do espelho. *O que num preenchimento é uma
/// fase invisível, numa banda de 20 % da forma é a aparência inteira.*
///
/// ⚠️⚠️ **E ele é uma conta SEPARADA da do [`rect`], de propósito.** A 1.ª redacção fazia o `rect`
/// derivar daqui — *uma porta, dois consumidores* — e o gate que os comparava virou **tautologia**:
/// a mutação `[cx - half, cy]` **sobreviveu**, porque a forma seguia o canto para onde ele fosse.
/// *Concordância só se mede entre duas afirmações independentes; derivar as duas de uma anula o
/// instrumento.* Aqui a geometria é a verdade e este canto tem de a acertar — e há gate a medi-lo.
fn canto(cx: f64, cy: f64, half: f64) -> [f64; 2] {
    [cx - half, cy - half]
}

fn rect(cx: f64, cy: f64, half: f64) -> Vec<VecVertex> {
    [
        [cx - half, cy - half],
        [cx + half, cy - half],
        [cx + half, cy + half],
        [cx - half, cy + half],
    ]
    .map(VecVertex::corner)
    .to_vec()
}

/// A lei de um padrão, com o tamanho de cópia pedido — a porta única, para o preenchimento e o
/// traço nascerem da MESMA conta (plano 35, wave D).
fn lei(
    source: PatternSource,
    kind: TileKind,
    mode: PatternMode,
    fallback: [u8; 3],
    lado: f64,
    origem: [f64; 2],
) -> PatternFill {
    let mut f = PatternFill::new(
        source,
        [lado, lado],
        Rgba8::new(fallback[0], fallback[1], fallback[2], 255),
    );
    f.kind = kind;
    // ⛔⛔ **O `f.offset_denom = 2` SAIU DAQUI, e a remoção é a metade que importa** (2026-08-30).
    //
    // Esta cena punha-o à mão, e o produto nascia com `1` — então ela demonstrava tijolos e
    // colmeias a ladrilhar **sobre um produto em que os chips *Brick* e *Column* eram inertes**.
    // *Uma cena de smoke que compensa o defeito do produto aprova-o*, e esta esteve verde durante
    // toda a vida da feature.
    //
    // ⇒ o valor vem agora do construtor (`PatternFill::new`), que é o que a autoria real usa. A
    // cena passa a ser o **detector** do defeito em vez do véu dele.
    f.mode = mode;
    // ⛔ O canto da FORMA — ver [`canto`]. Sem isto a cena mede um objecto que o produto nunca
    // produz, porque a autoria real ancora na forma (`default_placement`).
    f.origin = origem;
    f
}

fn pattern(
    source: PatternSource,
    kind: TileKind,
    mode: PatternMode,
    fallback: [u8; 3],
    origem: [f64; 2],
) -> Paint {
    Paint::Pattern(Box::new(lei(
        source,
        kind,
        mode,
        fallback,
        BOX / 3.0,
        origem,
    )))
}

/// ⭐⭐ **UM CONTORNO COM PADRÃO** (plano 35) — a faixa recebe o ladrilho.
///
/// ⚠️ **A largura é DERIVADA do ladrilho, não escolhida**: a `STROKE_W` de `0,03` é fina demais
/// para o motivo se ler (menos de um décimo de uma cópia), e um smoke em que a feature é invisível
/// não é um smoke. Aqui a faixa fica em `1,2 ×` o lado de uma cópia — larga o bastante para se ver
/// **o que** repete, estreita o bastante para se ver **que** repete ao longo do perímetro (~24
/// cópias nos `8,8` de contorno de uma destas formas).
///
/// ⛔ E ela **não** manda no padrão: engrossar a linha muda a faixa, nunca o motivo (plano 35 §2.3).
fn contorno_com_padrao(
    source: PatternSource,
    kind: TileKind,
    fallback: [u8; 3],
    origem: [f64; 2],
) -> StrokeSpec {
    let lado = BOX / 6.0;
    let mut s = StrokeSpec::new(
        Rgba8::new(fallback[0], fallback[1], fallback[2], 255),
        lado * 1.2,
    );
    // ⛔ **`Tile`, e não `Mirror`** (report de 28/08). O espelho é uma lei legítima e já se
    // demonstra na 4.ª forma da fileira de cima, onde há reticulado que se leia; numa faixa de 20 %
    // da forma vê-se **uma fatia** do reticulado, e a paridade do espelho troca ao mover um nó —
    // o que se lê como *"o contorno inverteu"*. ⚠️ *Uma cena de smoke escolhe o modo que deixa a
    // FEATURE visível, não o que exercita mais código.*
    s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(lei(
        source,
        kind,
        PatternMode::Tile,
        fallback,
        lado,
        origem,
    )));
    s
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => select_hero(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    // A arte entra pelo MESMO endereçamento que a autoria usa (`insert_image_rgba8`), senão o smoke
    // provaria um caminho que o produto não tem.
    let source = PatternSource::Image(gfx.asset_db.insert_image_rgba8(ART, ART, art_rgba()));
    let scene = &mut gfx.vec_scene;
    let half = BOX * 0.5;
    let x = |i: usize| -2.5 * STEP + (i as f64) * STEP;

    // 1..4 — as leis de reticulado e de repetição.
    for (i, (kind, mode, fb)) in [
        (TileKind::Grid, PatternMode::Tile, [90, 90, 110]),
        (TileKind::BrickRow, PatternMode::Tile, [110, 90, 90]),
        (TileKind::Hex, PatternMode::Tile, [90, 110, 90]),
        (TileKind::Grid, PatternMode::Mirror, [110, 110, 80]),
    ]
    .into_iter()
    .enumerate()
    {
        scene.push_path(VecPath {
            verts: rect(x(i), 0.0, half),
            closed: true,
            fill: Some(pattern(source, kind, mode, fb, canto(x(i), 0.0, half))),
            stroke: Some(contorno()),
            ..VecPath::default()
        });
    }

    // 5 — o COMPOSTO com buraco, regra `EvenOdd`. O contorno de dentro tem de ficar VAZIO.
    let hole = ph2d_vec_scene::Contour {
        verts: rect(x(4), 0.0, half * 0.45),
        closed: true,
    };
    scene.push_path(VecPath {
        verts: rect(x(4), 0.0, half),
        closed: true,
        subpaths: vec![hole],
        fill_rule: FillRule::EvenOdd,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [80, 100, 120],
            canto(x(4), 0.0, half),
        )),
        stroke: Some(contorno()),
        ..VecPath::default()
    });

    // ⭐⭐ 7 — a ARTE é uma FORMA DO DOCUMENTO (W7, o modelo do Figma). O motivo fica ao lado,
    // visível e editável: mexer nos nós dele re-assa o ladrilho na hora.
    let motivo = scene.push_path(VecPath {
        verts: [[x(5) - 0.4, -3.5], [x(5) + 0.4, -3.5], [x(5), -2.6]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(90, 190, 220, 255))),
        stroke: Some(contorno()),
        ..VecPath::default()
    });
    scene.push_path(VecPath {
        verts: rect(x(4), -3.0, half),
        closed: true,
        fill: Some(pattern(
            PatternSource::Shape(motivo),
            TileKind::BrickRow,
            PatternMode::Tile,
            [70, 90, 110],
            canto(x(4), -3.0, half),
        )),
        stroke: Some(contorno()),
        ..VecPath::default()
    });

    // ⭐⭐ 8 — **SÓ CONTORNO, com padrão** (plano 35). Sem `fill` nenhum: é o caso que prova que a
    // faixa é o sujeito, e é também o que obriga o `Clamp` a enquadrar pela caixa do TRAÇO — um
    // enquadramento pela do preenchimento não teria o que ler aqui.
    scene.push_path(VecPath {
        verts: rect(x(0), -3.0, half),
        closed: true,
        fill: None,
        stroke: Some(contorno_com_padrao(
            source,
            TileKind::Grid,
            [40, 40, 55],
            canto(x(0), -3.0, half),
        )),
        ..VecPath::default()
    });

    // ⭐⭐ 9 — **OS DOIS**, com leis DIFERENTES (grade no preenchimento, espelho no traço). É esta
    // forma que faz aparecer a fileira `Fill | Stroke` no topo da secção *Pattern*: com um alvo só
    // não há escolha a oferecer, e o chip não é pintado.
    scene.push_path(VecPath {
        verts: rect(x(1), -3.0, half),
        closed: true,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [100, 100, 120],
            canto(x(1), -3.0, half),
        )),
        // ⚠️ **Reticulado DIFERENTE do preenchimento** (tijolo contra grade): é o que mostra que as
        // duas tintas carregam leis independentes, e mostra-o de forma **estável** — ao contrário
        // do espelho, cuja paridade troca ao mover um nó (report de 28/08).
        stroke: Some(contorno_com_padrao(
            source,
            TileKind::BrickRow,
            [40, 55, 40],
            canto(x(1), -3.0, half),
        )),
        ..VecPath::default()
    });

    // 6 — a mesma grade numa forma ESTICADA só em x. O padrão esmaga COM ela.
    let mut wide = VecPath {
        verts: rect(x(5) + half, 0.0, half),
        closed: true,
        fill: Some(pattern(
            source,
            TileKind::Grid,
            PatternMode::Tile,
            [120, 80, 110],
            canto(x(5) + half, 0.0, half),
        )),
        stroke: Some(contorno()),
        ..VecPath::default()
    };
    wide.id = VecPathId::default();
    let id = scene.push_path(wide);
    scene.scale_path(id, 1.9, 0.7, [x(5) + half, 0.0]);
}

/// Seleciona a PRIMEIRA forma — o painel abre com o chip **Pattern** aceso.
fn select_hero(app: &mut crate::App) {
    let first: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().first().map(|p| p.id));
    if let Some(id) = first {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] texture pattern: 6 formas. (1) GRADE, ja' selecionada - o chip **Pattern** esta' \
         aceso na seccao Fill Type. (2) TIJOLO 1/2: as linhas desfasam meia celula. (3) COLMEIA: o \
         mesmo desfasamento com o espacamento sqrt(3)/2. (4) ESPELHO: a arte inverte a cada \
         repeticao. (5) BURACO (EvenOdd): o miolo tem de ficar VAZIO - se o padrao o pintar, a \
         regra de preenchimento nao viajou. (6) ESTICADA: a mesma grade numa forma escalada so' em \
         x - o padrao ESMAGA com ela, ao contrario do traco. A arte tem um quadrante TRANSPARENTE \
         (canto inferior direito de cada copia): ele tem de deixar ver o fundo, nao pintar vermelho. \
         ⭐ E EM BAIXO: um quadrado cuja ARTE e' o TRIANGULO ao lado dele (uma forma do documento). \
         Mexa nos nos do triangulo com a ferramenta Node -- o padrao tem de mudar NA HORA. \
         ⭐ TODO o ajuste vive no painel, na seccao Pattern: Tile, Offset, Width, Height, \
         Lock Aspect, Gap, Shift X, Shift Y, Angle e Repeat. Com o CADEADO ligado (o default) mexer \
         num eixo leva o outro; desligado, a arte ACHATA de proposito. As barras SHIFT X/Y deslizam a arte dentro de UMA repeticao \
         (0..100%, e 100 e' o mesmo que 0). No modo Clamp elas somem, com as outras que ele nao le^. \
         ⭐ E TODA forma desta cena nasce COM CONTORNO (escuro, fino) -- antes nasciam sem nenhum, e \
         a seccao Stroke ficava inerte SO' AQUI. Troque Fill Type entre Solid e Pattern: o contorno \
         tem de continuar la', e a largura/cor dele tem de responder ao painel. \
         ⭐⭐ E EMBAIXO A' ESQUERDA, DUAS FORMAS NOVAS: a 1a e' SO' CONTORNO, e o contorno dela e' \
         feito da arte (sem preenchimento nenhum). A 2a tem padrao NOS DOIS -- grade no miolo, \
         TIJOLO no contorno (reticulados diferentes, para se ver que sao duas leis independentes). \
         Mova os NOS destas duas com a ferramenta Node: a arte fica ANCORADA na forma e nao \
         inverte -- ela desliza por baixo da faixa como um papel de parede que a linha revela. Selecione a 2a: a seccao Stroke ganha a fileira **Type** \
         (Solid | Pattern) e a seccao Pattern ganha, no topo, a fileira **Target** \
         (Fill | Stroke) -- e' ela que diz qual dos dois os knobs abaixo estao a editar. Na 1a \
         forma o Target NAO aparece, porque com um alvo so' nao ha' escolha. Engrosse o contorno \
         com a barra Width: a faixa engrossa e o MOTIVO nao muda de tamanho. \
         ⭐⭐⭐ NOVO (30/08) -- OS CHIPS *TILE* ESTAVAM MORTOS: selecione qualquer forma com padrao \
         e, na seccao Pattern, percorra os quatro chips de Tile (Grid | Brick | Column | Hex). Os \
         QUATRO tem de mudar o desenho: Brick desfasa as LINHAS meia celula, Column desfasa as \
         COLUNAS, Hex poe os seis vizinhos a' mesma distancia. Ate' hoje o Brick e o Column davam \
         exactamente a mesma coisa que o Grid. E a fileira **Offset** logo abaixo (1/2 .. 1/8) tem \
         de mudar o tamanho do desfasamento. \
         ⭐⭐ E DEPOIS GRAVE: Ficheiro > Save As..., feche o programa, abra-o e carregue o ficheiro. \
         Tudo tem de voltar exactamente como estava -- as estampas com a arte delas, nao uma cor \
         chapada."
    );
}

#[cfg(test)]
mod tests {
    use super::{BOX, canto, lei, rect};
    use ph2d_vec_pattern::{PatternMode, TileKind};
    use ph2d_vec_scene::{PatternFill, PatternSource, Rgba8};

    /// ⛔⛔ **UM PADRÃO NASCE ONDE A FORMA COMEÇA** — report do Enio, 2026-08-28.
    ///
    /// ⚠️ **O default do construtor é a origem do MUNDO**, e é dele que esta cena tem de fugir: com
    /// a arte ancorada em `[0,0]` e a forma a três unidades dali, a fase do reticulado sob o
    /// contorno não tem relação nenhuma com a forma. Num preenchimento isso é invisível; numa faixa
    /// de 20 % da forma é a aparência inteira, e mover um nó desliza a arte por baixo da linha.
    ///
    /// ⭐ **Uma porta, dois consumidores:** o canto que o [`rect`] começa a desenhar É o canto em
    /// que o padrão nasce. Duas contas divergiriam no dia em que uma das formas mudasse de sítio.
    #[test]
    fn a_pattern_is_born_where_its_shape_starts() {
        let (cx, cy, half) = (3.0, -1.5, 1.1);
        let c = canto(cx, cy, half);
        let v = rect(cx, cy, half);
        // ⚠️ **A régua é uma PROPRIEDADE da geometria, não uma repetição da conta**: o canto é
        // menor ou igual a todo vértice, e É um deles. A 1.ª redacção comparava `v[0].anchor` com
        // `canto(..)` e uma mutação SOBREVIVEU, porque o `rect` derivava do `canto` — a forma
        // seguia-o para onde ele fosse. *Só duas contas independentes se podem contradizer.*
        assert!(
            v.iter().all(|p| p.anchor[0] >= c[0] && p.anchor[1] >= c[1]),
            "o canto do padrao nao e' o MINIMO da forma: {c:?} contra {:?}",
            v.iter().map(|p| p.anchor).collect::<Vec<_>>()
        );
        assert!(
            v.iter().any(|p| p.anchor == c),
            "o canto do padrao nao e' um vertice da forma"
        );
        let f = lei(
            PatternSource::Shape(1),
            TileKind::Grid,
            PatternMode::Tile,
            [1, 2, 3],
            BOX / 6.0,
            c,
        );
        assert_eq!(f.origin, c, "a lei nao carrega o canto que recebeu");
        // ⚠️ **CONTROLO — o default é a origem do MUNDO.** Sem ele, este gate ficaria verde num dia
        // em que o construtor passasse a ancorar sozinho, e a cena deixaria de provar o que prova.
        assert_eq!(
            PatternFill::new(
                PatternSource::Shape(1),
                [0.5, 0.5],
                Rgba8::new(1, 2, 3, 255)
            )
            .origin,
            [0.0, 0.0],
            "o construtor deixou de nascer na origem do mundo - este gate perdeu o sujeito"
        );
        assert_ne!(c, [0.0, 0.0], "a fixtura tem de estar LONGE da origem");
    }
}

#[cfg(test)]
mod lattice_tests {
    use super::{BOX, lei};
    use ph2d_vec_pattern::{PatternMode, TileKind};
    use ph2d_vec_scene::{PatternFill, PatternSource, Rgba8, VecPathId};

    fn fonte() -> PatternSource {
        PatternSource::Shape(VecPathId::from(1u64))
    }

    fn da_cena(kind: TileKind) -> PatternFill {
        lei(
            fonte(),
            kind,
            PatternMode::Tile,
            [1, 2, 3],
            BOX / 3.0,
            [0.0, 0.0],
        )
    }

    /// ⛔⛔⛔ **A CENA NÃO COMPENSA O PRODUTO** — o gate que faltava, e a ausência dele custou a
    /// vida inteira de uma feature.
    ///
    /// Esta cena escrevia `f.offset_denom = 2` à mão, e o construtor do produto nascia com `1`.
    /// ⇒ ela demonstrava tijolos e colmeias a ladrilhar **sobre um produto em que os chips *Brick*
    /// e *Column* eram inertes** — o artista carregava neles e via uma grade. A cena esteve verde
    /// o tempo todo, porque não tinha gate nenhum.
    ///
    /// ⚠️ **A lei geral:** uma cena de smoke tem de nascer no estado em que o artista a
    /// encontraria. Já custou um report do Enio uma vez (as formas desta mesma cena nasciam **sem
    /// contorno**, e a secção *Stroke* ficava inerte só aqui); esta é a segunda ocorrência no MESMO
    /// ficheiro, com o sujeito trocado.
    #[test]
    fn the_scene_does_not_hand_set_what_the_constructor_decides() {
        let cru = PatternFill::new(fonte(), [1.0, 1.0], Rgba8::new(1, 2, 3, 255));
        for kind in [
            TileKind::Grid,
            TileKind::BrickRow,
            TileKind::BrickCol,
            TileKind::Hex,
        ] {
            assert_eq!(
                da_cena(kind).offset_denom,
                cru.offset_denom,
                "{kind:?}: a cena escreve um desfasamento que o produto nao escreve - ela esta' a \
                 compensar o construtor, e um chip morto passaria por ela"
            );
        }
    }

    /// ⭐⭐⭐ **OS QUATRO RETICULADOS DESTA CENA LADRILHAM DE FACTO** — a régua sobre a lei ASSADA.
    ///
    /// A grade é uma célula por construção; os outros três **têm** de precisar de mais do que uma,
    /// senão são uma grade com outro nome. É esta linha que apanha o defeito se ele voltar por
    /// qualquer caminho (um construtor mudado, um `period()` mudado, uma cena mudada).
    #[test]
    fn every_lattice_in_this_scene_actually_tiles() {
        let px = [16u32, 16];
        assert_eq!(
            da_cena(TileKind::Grid).law(px).cells(),
            [1, 1],
            "a grade deixou de ser o neutro"
        );
        for kind in [TileKind::BrickRow, TileKind::BrickCol, TileKind::Hex] {
            let cells = da_cena(kind).law(px).cells();
            assert!(
                cells[0] * cells[1] > 1,
                "{kind:?} assa {cells:?} - e' uma grade com outro nome, e o chip nao muda um pixel"
            );
        }
    }
}

/// ⭐⭐⭐ **A ARTE DESTA CENA NÃO ENCAIXA CONSIGO PRÓPRIA — e é ISSO que a torna o smoke do aviso**
/// (plano 33, W10).
///
/// A dica de costura do painel só tem sujeito quando o ladrilho salta na volta. Esta arte salta —
/// a barra laranja do topo encosta no fundo branco/azul, e a meia-diagonal encosta no vazio — e o
/// salto não foi escolhido para isso: ele já lá estava desde a W5, porque a arte foi desenhada
/// **assimétrica nos dois eixos** para denunciar rotação e espelho.
///
/// ⚠️ **Sem este gate a cena podia ficar muda sem ninguém dar por isso.** Alguém a "arrumar" —
/// fechar a diagonal, uniformizar a barra — apagaria o aviso do smoke e o próximo leitor concluiria
/// que a feature não funciona. *Uma cena de smoke que deixa de conter o fenómeno aprova a ausência
/// dele.*
#[cfg(test)]
mod seam_hint_tests {
    use super::{ART, art_rgba};

    #[test]
    fn this_scenes_art_does_not_tile_and_that_is_what_the_hint_needs() {
        let t = ph2d_vec_pattern::bake(&art_rgba(), ART, ART, &ph2d_vec_pattern::TileLaw::grid())
            .expect("a grade encostada assa");
        let salto = ph2d_vec_pattern::wrap_seam(&t);
        assert!(
            ph2d_vec_pattern::seam_is_visible(salto),
            "a arte da cena passou a ENCAIXAR (salto {salto}, joelho {}) - o aviso de costura \
             deixou de ter sujeito e o smoke dele ficou mudo",
            ph2d_vec_pattern::SEAM_VISIBLE
        );
        println!("salto da arte da cena =76: {salto} niveis");
    }
}
