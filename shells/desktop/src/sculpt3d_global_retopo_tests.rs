//! **OS GATES DA CADEIA GLOBAL, PELO GESTO** (ADR-0161, F5.2).
//!
//! Irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! [`super::tests`]: um gesto exige uma cena e uma cena exige um device, então
//! estes gates são `#[ignore]` + `gpu_or_skip!`.
//!
//! ⛔ **Este ficheiro nasce de uma auditoria que encontrou o produto DESTRUÍDO com
//! 10.515 gates verdes** (2026-08-21). Duas causas, e as duas moram aqui:
//!
//! 1. **Nenhuma asserção olhava uma coordenada.** `quads`, `non_quads`,
//!    `boundary_edges` e `irregular` são função pura dos ÍNDICES — uma malha com as
//!    posições embaralhadas dá os mesmos números, bit a bit. As duas asserções de
//!    ARESTA de [`the_button_delivers_the_global_chain`] são a cura.
//! 2. ⚠️ **Os gates do slider e do duplo clique apontavam para o motor LOCAL.**
//!    Quando o botão foi repontado para a cadeia global, eles não foram — e a
//!    `quad_remesh_global` ficou com **um** chamador de teste em toda a workspace:
//!    um clique, `detail = 0,5`, sobre a única fixtura que caía do lado que não
//!    partia. *Repontar um botão sem repontar os gates dele é trocar o que o
//!    produto faz sem trocar o que se afirma sobre ele.*
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::history::global_retopo -- --ignored --nocapture
//! ```

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar.
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// Uma cena com a esfera amassada — a fixtura mais próxima de uma escultura.
fn wrinkled(device: &wgpu::Device) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, crate::sculpt3d::fixtures::wrinkled_sphere(), 1.0);
    s.viewport = (900, 700);
    s
}

/// Quantas arestas da malha têm **uma** face só — a casca aberta.
fn open_edges(mesh: &ph2d_mesh::Mesh) -> usize {
    use std::collections::BTreeMap;
    let mut e: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            *e.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    e.values().filter(|c| **c == 1).count()
}

/// ⭐⭐ **O BOTÃO ENTREGA A PROMESSA DA CADEIA GLOBAL** — 100 % de quads, casca
/// fechada, e a contagem de irregulares perto do chão topológico.
///
/// ⚠️ **É o gate da PORTA e não do algoritmo.** A cadeia já é gateada, sem GPU, na
/// `ph2d-quadfill`; o que só se pode afirmar aqui é que **o gesto do artista**
/// chega àquela cadeia — que o `detail` do painel atravessa, que a peça é
/// substituída e que o Ctrl+Z a devolve.
///
/// ⚠️ **A barra dos quads é `100 %` e não *"a maioria"***, ao contrário do irmão
/// local: é a promessa inteira desta família de algoritmos, e afrouxá-la seria
/// deixar de medir a única coisa que o pivô comprou.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_button_delivers_the_global_chain() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(
        &gpu.device,
        crate::sculpt3d::fixtures::wrinkled_sphere(),
        1.0,
    );
    s.viewport = (900, 700);
    let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    let r = s
        .quad_remesh_global(0.5)
        .expect("a cadeia global correu sobre uma peca de um nivel");
    eprintln!(
        "[sculpt3d] GLOBAL: {} vertices, {} quads, {} nao-quads, {} irregulares, bordo {}, \
         aresta mediana {:.2}x e MAXIMA {:.2}x do alvo, quad de {:.4}, {:.0} ms",
        r.verts,
        r.quads,
        r.non_quads,
        r.irregular,
        r.holes,
        r.edge_median_ratio,
        r.edge_max_ratio,
        r.edge,
        r.ms
    );

    assert_eq!(
        r.non_quads, 0,
        "sairam {} faces que nao sao quads",
        r.non_quads
    );
    assert!(r.quads > 0, "a cadeia devolveu uma malha vazia");
    // ⚠️ Uma aresta com UMA face só é a assinatura da casca rasgada, e nenhum
    // render a mostra.
    assert_eq!(
        r.holes, 0,
        "a casca saiu ABERTA: {} arestas de bordo",
        r.holes
    );
    // ⭐ O chão topológico de uma esfera é 8. A barra é `3×` — a mesma da
    // `ph2d-quadfill`, e pela mesma razão: é a CONTAGEM que é estrutural, e
    // medi-la em percentagem faz o número melhorar sozinho com a densidade.
    assert!(
        (8..=24).contains(&r.irregular),
        "{} vertices irregulares: o chao de uma esfera e' 8 e a barra e' 24",
        r.irregular
    );
    // ⭐⭐ **AS DUAS ASSERÇÕES GEOMÉTRICAS, e sem elas este gate é CEGO.**
    //
    // ⛔ Todas as de cima — quads, não-quads, bordo, irregulares — são função pura
    // dos ÍNDICES: uma malha com as posições embaralhadas dá exactamente os mesmos
    // números. Foi assim que este gate ficou verde sobre a foto que o artista
    // mandou em 2026-08-21: a malha voltou destruída, com lascas finas e arestas a
    // atravessar a peça de lado a lado, e ele **não tinha como reprovar**.
    //
    // Medido, na mesma fixtura: caminho correcto **máx 3,0×** e mediana 0,9× do
    // alvo; caminho destruído **18×** (= o diâmetro da peça) e mediana 4,6×.
    assert!(
        r.edge_max_ratio <= 4.0,
        "a aresta MAIS LONGA saiu {:.2}x o alvo (barra 4x): alguma coisa na malha \
         atravessa a peca -- e' o sintoma de geometria montada sobre indices de OUTRA malha",
        r.edge_max_ratio
    );
    assert!(
        (0.5..=2.0).contains(&r.edge_median_ratio),
        "a aresta MEDIANA saiu {:.2}x o alvo (faixa 0,5x-2,0x): a grade nao tem o passo pedido",
        r.edge_median_ratio
    );
    // ⚠️ **AS DUAS, e não uma.** Prova de mutação com o defeito reintroduzido:
    // nesta fixtura a **máxima** saiu `1,64×` — **debaixo da barra de 4×**, e
    // sozinha teria deixado passar; quem apanhou foi a **mediana** (`0,27×`). Na
    // fixtura de 98 k que a auditoria mediu foi ao contrário: máxima `18×`. *O
    // dano geométrico não escolhe sempre a mesma régua.*
    assert_ne!(
        s.mesh().positions(),
        &before[..],
        "a retopologia global nao mudou a malha"
    );
    assert!(
        s.undo_stroke(),
        "a cadeia global nao gravou passo de undo: o artista perde a peca"
    );
    assert_eq!(
        s.mesh().positions(),
        &before[..],
        "o Ctrl+Z nao devolveu a malha de antes"
    );
}

/// ⭐⭐ **O SEGUNDO CLIQUE** — e a auditoria mediu que ele era **panic certo**.
///
/// ⚠️ **É a rota alcançável mais barata para o defeito da raiz.** O clique 1
/// substitui a peça por umas centenas de vértices; o clique 2 toma-a como
/// superfície, o F1 **refina** (o alvo dele é `α × diagonal da caixa`, logo ele
/// densifica toda entrada mais grossa que ~2.500 vértices), e o layout passa a ter
/// índices **acima** do alcance da malha que o F5 amostrava. `index out of bounds`
/// no meio de um gesto do artista, com a peça por gravar — e `catch_unwind` não
/// cobre este caminho.
///
/// ⚠️ **O gate irmão do motor LOCAL existia** (`two_clicks_without_undo_still_
/// return_a_piece`) e **não foi repontado** quando o botão mudou de motor. Este é
/// o que faltava.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn three_clicks_in_a_row_still_return_a_usable_piece() {
    let gpu = gpu_or_skip!();
    let mut s = wrinkled(&gpu.device);
    for click in 1..=3u32 {
        let before: Vec<[f32; 3]> = s.mesh().positions().to_vec();
        let r = match s.quad_remesh_global(1.0) {
            Ok(r) => r,
            // ⭐⭐ **O CONTRATO QUE DE FACTO IMPORTA: uma recusa NUNCA destrói.**
            //
            // ⛔ **Medido em 2026-08-21, e é defeito aberto:** o clique 3 recusa
            // com `Broken { patch: 6, side: 12 }` — a fronteira de um patch não
            // fecha. A cadeia nunca tinha corrido sobre uma malha que ela própria
            // produziu, e o F1 grosseira a peça a cada clique (270 vértices ao
            // clique 2), até o traçado sair degenerado.
            //
            // ⚠️ **Este braço NÃO é a barra afrouxada.** Ele afirma a promessa que
            // o artista precisa e que o panic anterior quebrava: *a peça fica
            // exactamente como estava.* Antes da cura da raiz isto era
            // `index out of bounds` — a janela morria com o trabalho por gravar.
            Err(e) => {
                eprintln!("[sculpt3d] clique {click}: RECUSOU com {e:?}");
                assert_eq!(
                    s.mesh().positions(),
                    &before[..],
                    "clique {click}: a recusa MEXEU na peca -- e' a unica coisa \
                     que uma recusa nao pode fazer"
                );
                assert!(
                    click >= 3,
                    "clique {click}: a cadeia recusou cedo demais ({e:?}) -- os dois \
                     primeiros cliques tem de fechar"
                );
                continue;
            }
        };
        eprintln!(
            "[sculpt3d] clique {click}: {} vertices, {} quads, {} irregulares, \
             aresta mediana {:.2}x e MAXIMA {:.2}x do alvo",
            r.verts, r.quads, r.irregular, r.edge_median_ratio, r.edge_max_ratio
        );
        assert!(r.verts > 0, "clique {click}: a peca sumiu");
        assert_eq!(
            r.non_quads, 0,
            "clique {click}: sairam faces que nao sao quads"
        );
        assert_eq!(
            open_edges(s.mesh()),
            0,
            "clique {click}: a casca saiu ABERTA"
        );
        // ⭐ E a geometria, que é a régua que a auditoria mostrou que faltava.
        assert!(
            r.edge_max_ratio <= 4.0,
            "clique {click}: a aresta mais longa saiu {:.2}x o alvo",
            r.edge_max_ratio
        );
        assert!(
            (0.5..=2.0).contains(&r.edge_median_ratio),
            "clique {click}: a aresta mediana saiu {:.2}x o alvo",
            r.edge_median_ratio
        );
    }
}

/// ⭐ **TODO PONTO DO SLIDER devolve uma peça** — o irmão global do
/// `every_point_of_the_detail_slider_returns_a_piece`.
///
/// ⚠️ **Um eixo só, e é honesto:** a cadeia global **não consome** o `adapt`, então
/// varrer os dois seria varrer 25 células em que cinco valores não mudam nada. O
/// gate do motor local varre a grelha dos dois porque lá o segundo eixo existe.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn every_point_of_the_detail_slider_returns_a_piece_on_the_global_chain() {
    let gpu = gpu_or_skip!();
    let mut counts: Vec<usize> = Vec::new();
    for step in 0..=4u32 {
        #[allow(clippy::cast_precision_loss)]
        let detail = step as f32 / 4.0;
        let mut s = wrinkled(&gpu.device);
        let r = s
            .quad_remesh_global(detail)
            .unwrap_or_else(|e| panic!("detail={detail:.2}: a cadeia global recusou ({e:?})"));
        eprintln!(
            "[sculpt3d] detail={detail:.2}: {} quads, {} irregulares, aresta mediana {:.2}x \
             e MAXIMA {:.2}x do alvo",
            r.quads, r.irregular, r.edge_median_ratio, r.edge_max_ratio
        );
        assert!(r.verts > 0, "detail={detail:.2}: a peca sumiu");
        assert_eq!(r.non_quads, 0, "detail={detail:.2}: sairam nao-quads");
        assert_eq!(
            open_edges(s.mesh()),
            0,
            "detail={detail:.2}: a casca saiu ABERTA"
        );
        assert!(
            r.edge_max_ratio <= 4.0,
            "detail={detail:.2}: a aresta mais longa saiu {:.2}x o alvo",
            r.edge_max_ratio
        );
        counts.push(r.quads);
    }
    // ⚠️ **E o slider tem de FAZER alguma coisa.** Cinco pontos que devolvem a
    // mesma contagem seriam um knob morto passando por um gate verde.
    assert!(
        counts.iter().max() > counts.iter().min(),
        "os cinco pontos do slider deram a MESMA contagem {counts:?}: o knob nao faz nada"
    );
}
