//! Os gates do traço que **a topologia dinâmica** trouxe.
//!
//! ⚠️ Irmão e não um bloco a mais no `stroke_tests.rs`, e o corte é por
//! ASSUNTO: ali mora *o que um dab faz*, aqui *o que acontece quando a malha
//! cresce debaixo do traço*. O pai bateu no teto de LOC no dia em que estes
//! dois entraram, e juntá-los teria sido o argumento errado — o tamanho é o
//! sintoma, o assunto é a razão.

use super::*;

/// **A LEI DO TRAÇO SOBREVIVE À MALHA CRESCER** — o gate da topologia dinâmica
/// do lado do traço.
///
/// ⚠️ O que está em julgamento é o `pre`: depois de um refino, um vértice que já
/// tinha sido tocado tem de continuar medindo a partir de onde ele estava no
/// PEN-DOWN, e não de onde o dab anterior o deixou. A alternativa que parece
/// óbvia — chamar `begin` outra vez depois de refinar — é exatamente a doença do
/// produto-por-dab que esta casa curou quatro vezes no relevo do Painter, e ela
/// passa despercebida porque o traço continua *funcionando*: ele só fica mais
/// forte quanto mais o refino disparar.
///
/// ⚠️ **A fixture refina de VERDADE** (`ph2d_mesh::refine_in_sphere`), e a
/// primeira versão dela não: ela crescia o traço sem crescer a malha, o que o
/// `assert` do `dab` recusa — corretamente. Um par `grow_to`/malha que não anda
/// junto não é o que o produto faz.
#[test]
fn growing_the_stroke_keeps_the_frozen_base() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]);
    stroke.dab(&mut mesh, &Brush::default(), &dab, Symmetry::default());

    let touched = stroke.touched().to_vec();
    let base = stroke.base_positions().to_vec();
    assert!(!touched.is_empty(), "a fixture contém o fenômeno");
    let n_before = mesh.vert_count();

    let target = ph2d_mesh::edge_target(0.5, 1.0);
    let mut births = Vec::new();
    let r = ph2d_mesh::refine_in_sphere(
        &mut mesh,
        [0.0, 0.0, 1.0],
        0.5,
        target,
        &mut births,
        &mut scratch(),
    );
    assert!(
        matches!(r, ph2d_mesh::Refine::Done { .. }),
        "a fixture TEM de refinar: {r:?}"
    );
    stroke.grow_with(&mesh, &births);

    // ⚠️ **A janela CRESCE, e o que tem de sobreviver é o PREFIXO.** Os
    // vértices nascidos no refino entram nela — eles fazem parte deste traço e
    // vão ser movidos por ele —, mas nenhum dos que já lá estavam pode mudar de
    // posição na lista nem de `pre`. Um `begin` aqui zeraria os dois.
    assert!(
        stroke.touched().len() > touched.len(),
        "os nascidos entram na janela"
    );
    assert_eq!(
        &stroke.touched()[..touched.len()],
        &touched[..],
        "a janela do traço sobrevive"
    );
    assert_eq!(
        &stroke.base_positions()[..base.len()],
        &base[..],
        "e o `pre` de cada vértice já tocado é o MESMO"
    );

    // O dab seguinte continua medindo do `pre`: os vértices JÁ TOCADOS não se
    // movem de novo. (Os nascidos no refino movem-se, e devem: para eles este é
    // o primeiro toque.)
    let after_one: Vec<[f32; 3]> = mesh.positions()[..n_before].to_vec();
    stroke.dab(&mut mesh, &Brush::default(), &dab, Symmetry::default());
    let moved: f32 = touched
        .iter()
        .map(|&v| {
            let (a, b) = (mesh.positions()[v as usize], after_one[v as usize]);
            (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
        })
        .sum();
    assert!(
        moved < 1e-4,
        "o mesmo dab sobre o mesmo `pre` é idempotente; os tocados andaram {moved} — \
         o traço está compondo"
    );
}

/// `grow_with` sem nascimentos é NO-OP — e o gate existe porque encolher seria
/// a forma silenciosa de perder a janela: os índices altos sumiriam do `slot` e
/// o dab seguinte os trataria como nunca-vistos, capturando um `pre` que é o
/// resultado do dab anterior.
///
/// ⚠️ **A primeira versão deste gate media `capacity_bytes`, e a mutação passou
/// por ela:** `Vec::resize` para menos NÃO devolve capacidade, então o oráculo
/// não podia ver o encolhimento que ele julgava. O oráculo certo é o produto —
/// o dab seguinte, que compara `slot.len()` com a contagem de vértices e panica
/// quando elas discordam.
#[test]
fn growing_with_nothing_new_is_a_noop() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(8, 12, 1.0);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.grow_with(&mesh, &[]);
    let moved = stroke.dab(
        &mut mesh,
        &Brush::default(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert!(moved > 0, "o traço continua dimensionado para ESTA malha");
}

/// **UM VÉRTICE NASCIDO NO MEIO DO TRAÇO FICA ENTRE OS PAIS DELE.**
///
/// Ele nasce no ponto médio de dois pais **que este mesmo traço já mexeu**, e há
/// duas maneiras de errar o `pre` dele, uma para cada lado:
///
/// - tratá-lo como **nunca-visto** captura a posição já deslocada como `pre`, e
///   o dab soma o deslocamento outra vez — uma AGULHA da altura do traço;
/// - herdar `accum = 0` faz o primeiro dab FRACO que o alcance escrevê-lo quase
///   no `pre` enquanto os vizinhos ficam levantados — uma CRATERA.
///
/// ⚠️ **O oráculo é a POSIÇÃO RELATIVA AOS PAIS**, e é isso que o torna imune ao
/// verbo e ao falloff: seja qual for o alvo, um vértice no meio de uma aresta
/// tem de acabar entre os dois extremos dela. Medir a altura absoluta exigiria
/// conhecer a lei do `Draw`, e o gate seria um espelho dela.
///
/// ⚠️ **São DOIS gestos, e cada um vê um dos erros — sozinho, nenhum vê os dois.**
/// PUXANDO (um Move), a superfície viaja a distância do gesto, as arestas
/// esticam de verdade e o refino nasce entre pais já levantados: é ali que a
/// agulha aparece. VARRENDO com um Draw, o vértice nasce perto da BORDA da
/// pegada, onde o peso deste dab é quase zero e o dos anteriores não era: é ali
/// que a cratera aparece.
///
/// ⚠️ **A primeira versão deste gate apertava com um `Draw` de pressão
/// crescente, e as duas mutações passaram por ela.** O `Draw` desloca uma
/// fração do raio, então as arestas esticam poucos por cento e o refino **nunca
/// re-dispara** — todos os nascimentos aconteciam no primeiro dab, sobre pais
/// que ainda não se tinham movido. A fixture media 0,044 da aresta com e sem a
/// herança: ela não continha o fenômeno.
#[test]
fn a_vertex_born_mid_stroke_lands_between_its_parents() {
    let press = worst_birth_offset(Gesture::Pull);
    let sweep = worst_birth_offset(Gesture::Sweep);
    let finer = worst_birth_offset(Gesture::Finer);
    let masked = worst_birth_offset(Gesture::Masked);
    // ⚠️ **As barras são MEDIDAS, e a folga de cada lado também.** Certo:
    // **0,053** puxando e **0,108** varrendo — o que sobra é a curvatura que o
    // refino preserva de propósito. Errado: **0,720** (sem a herança do `pre`,
    // a agulha) e **0,446** (com `accum` herdado em zero, a cratera).
    //
    // ⚠️ A primeira barra que escrevi foi 0,45, e a cratera media 0,446: ela
    // teria passado por um triz. Uma barra escolhida antes de ver os dois lados
    // da mutação é um palpite com casas decimais.
    assert!(
        press < 0.25,
        "puxando, o pior vértice novo saiu a {press:.3} da aresta"
    );
    assert!(
        sweep < 0.25,
        "varrendo, o pior vértice novo saiu a {sweep:.3} da aresta"
    );
    assert!(
        finer < 0.25,
        "afinando o detalhe, o pior vértice novo saiu a {finer:.3} da aresta"
    );
    assert!(
        masked < 0.25,
        "sob máscara, o pior vértice novo saiu a {masked:.3} da aresta"
    );
}

/// Os três gestos que fazem um vértice nascer no meio de um traço. Cada um vê um
/// erro que os outros não veem — ver o gate.
#[derive(Clone, Copy)]
enum Gesture {
    /// Um Move que ARRASTA: as arestas esticam de verdade e o refino re-dispara
    /// sobre pais já levantados.
    Pull,
    /// Um Draw que ANDA: o vértice nasce na borda da pegada, onde o peso deste
    /// dab é quase zero e o dos anteriores não era.
    Sweep,
    /// A região está MASCARADA. Um vértice que nasce ali e herda máscara ZERO
    /// recebe peso cheio enquanto os vizinhos dele estão congelados: ele sobe
    /// sozinho, que é a mesma agulha por um terceiro caminho.
    Masked,
    /// O artista APERTA `U` no meio do traço. É a única rota que faz o refino
    /// re-disparar sem que a geometria estique, e por isso é a que exercita um
    /// verbo cujo alvo lê a NORMAL congelada (o Inflate) sobre território que o
    /// traço já deformou.
    Finer,
}

/// Roda um gesto e devolve o pior deslocamento de um vértice recém-nascido em
/// relação ao meio dos pais dele, em frações do comprimento da aresta.
fn worst_birth_offset(gesture: Gesture) -> f32 {
    let sweep = matches!(gesture, Gesture::Sweep);
    let mut mesh = ph2d_mesh::shapes::uv_sphere(10, 14, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    let brush = Brush {
        verb: match gesture {
            Gesture::Pull => Verb::Move,
            Gesture::Sweep => Verb::Draw,
            Gesture::Finer => Verb::Inflate,
            Gesture::Masked => Verb::Draw,
        },
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    if matches!(gesture, Gesture::Masked) {
        // A máscara é do DOCUMENTO, pintada antes deste traço — é o estado em
        // que o artista deixa a peça quando protege uma região.
        for m in mesh.masks_mut() {
            *m = 1.0;
        }
    }
    let mut stroke = SculptStroke::default();
    let mut births = Vec::new();
    stroke.begin(&mesh);

    let mut worst = 0.0f32;
    let mut checked = 0;
    const DABS: u8 = 12;
    for k in 0..DABS {
        let t = f32::from(k) / f32::from(DABS - 1);
        let centre = if sweep {
            let x = -0.5 + t;
            [x, (1.0 - x * x).max(0.0).sqrt(), 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        // ⚠️ No `Finer` o alvo APERTA na metade do gesto — é o `U` do artista, e
        // é o que faz o refino nascer sobre território já deformado sem que a
        // geometria precise esticar.
        let detail = if matches!(gesture, Gesture::Finer | Gesture::Masked) && t > 0.5 {
            0.95
        } else {
            0.7
        };
        let target = ph2d_mesh::edge_target(brush.radius, detail);
        ph2d_mesh::refine_in_sphere(
            &mut mesh,
            centre,
            brush.radius,
            target,
            &mut births,
            &mut scratch(),
        );
        let born = births.clone();
        stroke.grow_with(&mesh, &births);
        let eye = [-centre[0], -centre[1], -centre[2]];
        let d = if !matches!(gesture, Gesture::Pull) {
            Dab::at(centre, brush.radius, eye)
        } else {
            // ⚠️ **PUXAR, e não carimbar.** O `Draw` desloca uma fração do raio,
            // o que estica as arestas em poucos por cento — o refino nunca
            // re-dispara e a fixture não conteria o fenômeno (medido: o pior
            // deslocamento fica em 0,044 da aresta com e sem a herança). Um
            // Grab arrasta a superfície pela distância do gesto, então a malha
            // estica de verdade e o refino nasce entre pais já levantados, que
            // é exatamente onde tratar o novo como nunca-visto conta o
            // deslocamento duas vezes.
            Dab::pulling(centre, brush.radius, eye, [0.0, 0.9 * t, 0.0])
        };
        stroke.dab(&mut mesh, &brush, &d, Symmetry::default());

        for b in &born {
            let p = mesh.positions();
            let (pa, pb, pm) = (p[b.a as usize], p[b.b as usize], p[b.vert as usize]);
            let mid = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let off = [pm[0] - mid[0], pm[1] - mid[1], pm[2] - mid[2]];
            let e = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let len = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt();
            if len > 1e-6 {
                worst =
                    worst.max((off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt() / len);
                checked += 1;
            }
        }
    }
    assert!(
        checked > 20,
        "a fixture TEM de conter o fenômeno: {checked}"
    );
    worst
}

fn scratch() -> ph2d_mesh::RegionScratch {
    ph2d_mesh::RegionScratch::default()
}

/// **O TRAÇO SOBREVIVE A UM COLAPSO** — o `pre` continua a descrever os mesmos
/// vértices depois de a malha renumerar.
///
/// ⚠️ **O oráculo é a IDENTIDADE, e ela tem de ser rastreada de fora.** Depois de
/// um colapso o vértice 45 pode ser o que era 255, então comparar `pre` por
/// índice compararia coisas diferentes com o mesmo nome. A fixture aplica o mesmo
/// remap a um vetor de identidades e pergunta se o `pre` que o traço guarda para
/// *o vértice que hoje se chama v* é o do vértice que ele ERA.
///
/// ⚠️ **E ele mede as duas pontas da ligação de mão dupla**: `touched[slot[v]]`
/// tem de voltar a `v`. Compactar um lado só deixa `slot` a apontar para o `pre`
/// de outro vértice — sem erro, sem aviso, e com a escultura a puxar o barro do
/// lugar errado.
#[test]
fn a_stroke_survives_a_collapse_and_keeps_the_pre_of_each_vertex() {
    let mut m = ph2d_mesh::shapes::uv_sphere(14, 20, 1.0);
    m.triangulate();
    let mut s = SculptStroke::default();
    s.begin(&m);
    // Um dab de verdade, para haver captura: sem ele o traço não guarda `pre`
    // nenhum e o gate ficaria verde sobre uma compactação de lista vazia.
    let emin = 1.2 * mean_edge(&m);
    let brush = Brush::default();
    // ⚠️ **O dab cobre a esfera INTEIRA, e sem isso o gate não continha o
    // fenômeno.** Os vértices que a compactação MOVE são os do FIM do vetor, e
    // numa esfera UV eles ficam no polo oposto ao dab — com uma pegada pequena
    // nenhum deles estava capturado, e a mutação *"não apague a marca de
    // origem"* não tinha o que corromper.
    let moved = s.dab(
        &mut m,
        &brush,
        &Dab::at([0.0, 0.0, 1.0], 3.0, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert!(moved > 0, "o controle: o dab tem de ter tocado alguém");
    assert_eq!(
        s.touched().len(),
        m.vert_count(),
        "o controle: a pegada tem de ser a malha inteira"
    );

    // Onde cada vértice estava, na numeração de ANTES.
    let pre_before: Vec<[f32; 3]> = (0..m.vert_count())
        .map(|v| pre_of(&s, &m, v as u32))
        .collect();
    let mut ident: Vec<u32> = (0..m.vert_count() as u32).collect();

    let mut remap = ph2d_mesh::Remap::default();
    let r = ph2d_mesh::collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut ph2d_mesh::RegionScratch::default(),
    );
    assert!(
        matches!(r, ph2d_mesh::Collapse::Done { .. }),
        "o controle: a fixture tem de colapsar ({r:?})"
    );
    for &(from, to) in &remap.vert_moves {
        ident[to as usize] = ident[from as usize];
    }
    ident.truncate(remap.verts);

    s.shrink_with(&remap);

    assert_eq!(s.touched().len(), s.base_positions().len());
    // ⚠️ **A ligação de mão dupla é perguntada do lado do VÉRTICE**, e a primeira
    // versão deste gate a perguntava do lado do slot — o que é um oráculo
    // CIRCULAR: `pre_of` procura `v` dentro do próprio `touched`, então um
    // ponteiro de volta obsoleto concorda consigo mesmo. Este módulo é FILHO do
    // `stroke`, e é por isso que ele alcança `slot`/`stamp` sem uma porta nova.
    for v in 0..m.vert_count() {
        if s.stamp[v] != s.epoch {
            continue;
        }
        assert_eq!(
            s.touched[s.slot[v] as usize] as usize, v,
            "o ponteiro de volta do vértice {v} aponta para outro"
        );
    }
    for (slot, &v) in s.touched().iter().enumerate() {
        assert!(
            (v as usize) < m.vert_count(),
            "um slot aponta para fora da malha"
        );
        assert_eq!(
            s.base_positions()[slot],
            pre_before[ident[v as usize] as usize],
            "o `pre` do vértice que hoje é {v} não é o de quem ele era"
        );
    }
    // E o dab seguinte não pode panicar no `assert` de dimensão.
    let _ = s.dab(
        &mut m,
        &brush,
        &Dab::at([0.0, 0.0, 1.0], 0.9, [0.0, 0.0, 1.0]),
        Symmetry::default(),
    );
}

/// O `pre` que o traço guarda para `v` — a mesma resposta que os verbos leem.
fn pre_of(s: &SculptStroke, m: &ph2d_mesh::Mesh, v: u32) -> [f32; 3] {
    s.touched()
        .iter()
        .position(|&t| t == v)
        .map_or(m.positions()[v as usize], |slot| s.base_positions()[slot])
}

fn mean_edge(m: &ph2d_mesh::Mesh) -> f32 {
    let pos = m.positions();
    let (mut sum, mut n) = (0.0f32, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
    }
    sum / n.max(1) as f32
}
