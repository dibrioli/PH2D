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
///
/// ⚠️ `pub(super)` porque as sondas irmãs ([`super::global_retopo_probes`]) a usam:
/// duas cópias de *"o que é uma aresta de bordo"* divergiriam no dia em que uma
/// delas passasse a ignorar faces degeneradas.
pub(super) fn open_edges(mesh: &ph2d_mesh::Mesh) -> usize {
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
///
/// # ⛔⛔ VERMELHO ABERTO — a aresta mediana do CLIQUE 3, e ele é PRÉ-EXISTENTE
///
/// | o que mudou | clique 1 | clique 2 | **clique 3** |
/// |---|---|---|---|
/// | `abs · 1/t` (a lei de `5ec438e17`) | 1,05× | 0,76× | ⛔ **0,32×** |
/// | `quad · 1/t²` (o custo novo) | 1,04× | 0,64× | ⛔ **0,43×** |
/// | + achatamento por patch | 1,03× | 0,81× | ⛔ **0,16×** |
///
/// ⚠️ **Ele já estava vermelho antes de tudo isto** — as medições são controladas,
/// a mesma máquina, uma variável de cada vez. O `assert` nasceu em `3d51cf18c` e o
/// `5ec438e17` (peso relativo) partiu-o **sem que ninguém soubesse**: este gate é
/// `#[ignore]` + GPU, então o lote do `ship.sh` **nunca o corre**. *Um gate que só
/// corre à mão fica verde na memória de quem o escreveu.*
///
/// ⭐⭐ **A causa, medida e agora afiada:** ao 3.º clique a entrada é a saída
/// grosseira do 2.º (218 vértices), então o alvo do slider fica **grande**; o F1
/// remalha para `α × diagonal` e devolve um `work` **fino**; e o layout traçado
/// sobre ele tem arcos muito mais curtos que o alvo. Com `ArcSpec::min = 1`, cada
/// um leva **um** segmento — ⇒ **a densidade da saída passa a ser a do LAYOUT e não
/// a do slider**, e a mediana cai para a aresta do `work`.
///
/// ⛔ **A cura NÃO é afrouxar a barra**, e ⛔ **não é derivar o alvo do `work`** —
/// isso foi construído e medido no mesmo dia: os cinco pontos do slider passaram a
/// dar `405 · 405 · 406 · 406 · 451` quads contra `405 … 1 336`, porque o extremo
/// fino do curso passava a ancorar-se numa constante. A cura é **grosseirar o
/// layout** quando ele é mais fino do que o alvo pede. Ver `PLAN.md` §4-septdecies.
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

/// ⭐⭐ **OS DOIS BACKENDS, LADO A LADO, NA MESMA PEÇA.**
///
/// ⚠️ **É a medição que decide qual deles o botão deve chamar HOJE**, e ela não
/// existia: o global entrou por decisão de arquitetura (ADR-0161) e nunca foi
/// comparado com o local **no mesmo gesto, na mesma malha**.
///
/// As duas colunas que importam são de espécies diferentes:
/// * o LOCAL entrega ~70 % de quads e casca furada, mas a grade **segue a
///   curvatura**;
/// * o GLOBAL entrega 100 % de quads e casca fechada, e **dobra** onde o patch é
///   grande.
///
/// ⚠️ **A contagem de dobras passou a vir do relatório da fase**
/// ([`ph2d_quadfill::folded_against`]) e não de um teste RADIAL escrito aqui. O
/// radial — *"a normal concorda com o raio a partir do centro da caixa?"* — só é
/// válido num sólido **estrelado**, e a esfera com BICO não é um.
///
/// ⭐ **E MEDIDO: aqui ele acertava.** Radial `11 · 17 · 19 · 22` contra
/// `12 · 17 · 19 · 23` da régua da referência, nas quatro densidades. *Trocar não
/// foi para corrigir um número: foi para a régua continuar válida quando a fixtura
/// deixar de ser estrelada — e para os DOIS backends serem medidos pela mesma.*
///
/// ⛔ **O que estava errado era a minha LEITURA da tabela.** Eu reportei
/// *"17 buracos"* onde a linha dizia `0 bordo` e `17 dobradas`, e passei a
/// diagnosticar uma casca aberta que nunca existiu. *Seis números lado a lado sem
/// rótulo em cada um é uma sonda que se pode ler ao contrário.*
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_two_backends_measured_on_the_same_piece() {
    let gpu = gpu_or_skip!();
    // ⚠️ **As TRÊS fixturas do corpus que têm feição**, e não só a amassada: a
    // `hooked` é a esfera-com-bico do diagnóstico, e a `ridged` tem cristas. Uma
    // fixtura de ondulação rasa **não contém o fenómeno** que a foto do Enio
    // mostra — um vinco fundo com a grade a passar por cima.
    for (fixture, make) in [("amassada", 0u8), ("com BICO", 1), ("com CRISTAS", 2)] {
        for detail in [0.25f32, 0.5, 0.75, 1.0] {
            let mesh = || match make {
                1 => crate::sculpt3d::fixtures::hooked_sphere(),
                2 => crate::sculpt3d::fixtures::ridged_sphere(),
                _ => crate::sculpt3d::fixtures::wrinkled_sphere(),
            };
            let mut a = {
                let mut s = Sculpt3dScene::new(&gpu.device, mesh(), 1.0);
                s.viewport = (900, 700);
                s
            };
            let g = a.quad_remesh_global(detail);
            let mut b = {
                let mut s = Sculpt3dScene::new(&gpu.device, mesh(), 1.0);
                s.viewport = (900, 700);
                s
            };
            let l = b.quad_remesh(detail, 0.0);
            match (g, l) {
                (Ok(gr), Ok(lr)) => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        println!(
                            "{fixture:<12} d={detail:.2} | GLOBAL {:<5} quads ({:.0}% quads) {:<4} irreg \
                         {:<4} bordo {:<4} DOBRADAS ({:.1} %) {:.0} ms",
                            gr.quads,
                            100.0 * gr.quads as f64 / (gr.quads + gr.non_quads).max(1) as f64,
                            gr.irregular,
                            gr.holes,
                            gr.folded,
                            100.0 * gr.folded as f64 / gr.quads.max(1) as f64,
                            gr.ms
                        );
                        println!(
                            "             | LOCAL  {:<5} quads ({:.0}% quads) {:<4} irreg \
                         {:<4} bordo {:<4} DOBRADAS ({:.1} %) {:.0} ms",
                            lr.quads,
                            100.0 * lr.quads as f64 / (lr.quads + lr.non_quads).max(1) as f64,
                            0,
                            lr.holes,
                            lr.folded,
                            100.0 * lr.folded as f64 / lr.quads.max(1) as f64,
                            lr.ms
                        );
                    }
                }
                (g, l) => println!("{fixture} detail={detail:.2} | global {g:?} local {l:?}"),
            }
        }
        println!("── {fixture} ──");
    }
}
