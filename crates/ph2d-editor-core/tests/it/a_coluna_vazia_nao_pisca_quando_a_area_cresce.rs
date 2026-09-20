//! ⭐⭐⭐ **A COLUNA VAZIA NÃO PISCA** — report do dono de 2026-09-20: *«pisca do lado direito
//! quando escondemos o inspector e aumentamos muito a área do grafo de nós»*.
//!
//! ⛔⛔ **A régua da ocupação fechava um CICLO consigo mesma:** com a coluna da direita vazia a
//! área de desenho cresce para dentro dela — e o painel que vive na área passa a publicar um rect
//! que a cobre. No quadro seguinte a coluna lia-se **ocupada**, a área encolhia, o painel encolhia
//! com ela, e no quadro a seguir a coluna voltava a ler-se livre. Período dois, a 60 Hz: uma faixa
//! da largura do Inspector a aparecer e a desaparecer.
//!
//! ⚠️ **Nenhum gate desta casa o via, e a razão é a FIXTURA:** os que existem alimentam o
//! `from_published` com rects **escolhidos à mão** (o rect da coluna, um popover a roçar) e medem
//! UM quadro. O defeito é uma realimentação entre quadros — *uma régua de um quadro só não pode
//! ver um ciclo de dois*.
//!
//! A tabela do regime está em [`super::diag_a_coluna_vazia_pisca`], a sonda versionada.

use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::zones::Rect;

/// A janela do report. ⚠️ **A altura é load-bearing com a timeline docada:** a fronteira do
/// defeito é a coluna ser coberta a meio, e a `1012` ela lê `0,487` — *a um fio*. Medido, o
/// piscar começava a partir de `1100`, que é o ecrã do dono.
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1200.0,
};

fn bandas() -> ChromeBands {
    let mut b = ChromeBands::DEFAULT;
    b.rail_w = 0.0; // o chrome de produção deita o trilho
    b
}

/// Um quadro do ciclo: recebe o que o anterior PUBLICOU, devolve `docks` e o que este publica.
///
/// ⚠️ **O Inspector está ESCONDIDO** — é o estado do report, logo ninguém publica na coluna
/// direita a não ser quem transbordar da área.
fn um_quadro(t: f32, timeline: bool, publicado: &[Rect]) -> (DockSides, Vec<Rect>) {
    let monta = |docks| {
        let mut l = HeroLayout::for_viewport_bands(
            JANELA,
            false,
            bandas(),
            CenterSplit::Horizontal { t },
            docks,
        );
        if timeline {
            l.dock_timeline_into_motion();
        }
        l
    };
    let (esq, dir) = monta(DockSides::BOTH).side_columns();
    let docks = DockSides::from_published(esq, dir, publicado.iter().copied());
    let l = monta(docks);
    (docks, vec![l.hierarchy, l.motion_graph])
}

/// Corre `n` quadros e devolve a leitura de `docks.right` em cada um.
fn corre(t: f32, timeline: bool, semente: Vec<Rect>) -> Vec<bool> {
    let mut publicado = semente;
    (0..8)
        .map(|_| {
            let (docks, proximo) = um_quadro(t, timeline, &publicado);
            publicado = proximo;
            docks.right
        })
        .collect()
}

/// ⭐⭐⭐ **O grafo no máximo, com a coluna vazia, ASSENTA** — e assenta em *livre*, que é a
/// resposta certa: sem Inspector não há ninguém naquela coluna.
///
/// ⚠️ **As DUAS metades são obrigatórias.** A 1.ª é o report (a leitura tem de parar de alternar);
/// a 2.ª é o CONTROLO que impede a cura barata — se alguém fizesse o `from_published` devolver
/// sempre `false`, o ciclo também parava, e a área voltaria a crescer **por cima de um painel
/// aberto**, que é o defeito que aquele módulo existe para curar.
///
/// FALSIFICADO por devolver o `takes` à metade de cima sozinha (a 1.ª cai: `.X.X.X.X`), ou por
/// fazê-lo recusar um inquilino a sério (a 2.ª cai).
#[test]
fn a_coluna_vazia_nao_pisca_quando_a_area_cresce_para_dentro_dela() {
    for timeline in [false, true] {
        for passo in 0..=20 {
            let t = passo as f32 / 20.0;
            let leituras = corre(t, timeline, Vec::new());
            let cauda = &leituras[4..];
            assert!(
                cauda.iter().all(|&v| v == cauda[0]),
                "t = {t:.2} (timeline docada = {timeline}): a coluna da direita ALTERNA entre \
                 ocupada e livre — {leituras:?}"
            );
        }
    }

    // ⭐ O CONTROLO: com um inquilino a sério publicado, a coluna lê-se OCUPADA e FICA.
    let (esq, dir) = HeroLayout::for_viewport_bands(
        JANELA,
        false,
        bandas(),
        CenterSplit::Horizontal { t: 0.0 },
        DockSides::BOTH,
    )
    .side_columns();
    let _ = esq;
    let com_inquilino = corre(0.0, true, vec![dir]);
    assert!(
        com_inquilino[0],
        "o rect da coluna publicado ocupa a coluna — {com_inquilino:?}"
    );
}

/// ⭐⭐ **E a METADE NOVA, isolada:** um rect que é a coluna toma-a; o MESMO rect esticado pela
/// área fora não.
///
/// ⚠️ A barra saiu de um vale MEDIDO (`0,185` do painel da área contra `1,000` de um inquilino,
/// `5,4×`), e esta é a metade que a mutação mata sem tocar na geometria do produto.
#[test]
fn um_painel_da_area_que_transborda_nao_toma_a_coluna() {
    let (esq, dir) =
        HeroLayout::for_viewport_bands(JANELA, false, bandas(), CenterSplit::None, DockSides::BOTH)
            .side_columns();

    assert!(
        DockSides::from_published(esq, dir, [dir]).right,
        "um inquilino publica o rect da coluna, e isso TOMA a coluna"
    );

    // A mesma altura, a começar bem à esquerda: a coluna esta' toda coberta e e' 1/5 do rect.
    let transborda = Rect::new(dir.x - dir.w * 4.0, dir.y, dir.w * 5.0, dir.h);
    assert!(
        !DockSides::from_published(esq, dir, [transborda]).right,
        "um painel da area que cobre a coluna de passagem NAO a ocupa"
    );

    // ⭐⭐ E a metade de CIMA, que esta wave achou SEM RÉGUA: uma prova de mutação que a apagava
    // sobrevivia a esta suite inteira, porque a fixtura do popover do gate irmão (um rect a roçar
    // a coluna pela borda) ja' e' recusada pela metade nova. O caso que so' ela recusa e um
    // flutuante INTEIRAMENTE DENTRO da coluna e pequeno — o Grid Snap, um popover, a galeria.
    let flutuante = Rect::new(dir.x + 10.0, dir.y + 10.0, dir.w - 20.0, dir.h * 0.25);
    assert!(
        !DockSides::from_published(esq, dir, [flutuante]).right,
        "um flutuante DENTRO da coluna, pequeno, nao a ocupa — senao a regua saltava enquanto o \
         artista arrasta um popover por cima dela"
    );
}
