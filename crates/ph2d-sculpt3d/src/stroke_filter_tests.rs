//! **O FILTRO** — a W9a, e a propriedade que ela existe para pinar.
//!
//! ⚠️ **Não é *"o filtro move a malha"*** — qualquer coisa move a malha. É que
//! ele **roda a MESMA LEI do pincel** (o campo vetorial dos dois é o mesmo a
//! menos de UM escalar), que o arrasto é **reversível ao bit**, e que a
//! máscara o freia.
//!
//! ⚠️ **A fixture é a `mesh_for(verb)` do pai, e é ela que contém o fenômeno:**
//! um verbo de anel sobre uma esfera LISA está no caso degenerado dele —
//! alisar o que já está liso mede vácuo, e três dos quatro filtros são de anel.

use super::*;

/// O pincel de fábrica desta suíte: curva CONSTANTE (o peso não depende de onde
/// o vértice cai), sem front-face, sem auto-smooth.
///
/// ⚠️ **Os três desarmes não são higiene, são o que torna a comparação
/// POSSÍVEL:** a curva constante põe todo vértice no mesmo peso (é o que faz a
/// razão entre os dois campos ser **um** escalar e não um por vértice), o
/// front-face mataria metade da esfera pelo olho (que um filtro não tem), e o
/// auto-smooth é um passe que corre DEPOIS do dab e que o filtro não roda.
fn filter_brush(verb: Verb) -> Brush {
    Brush {
        verb,
        radius: 10.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        front_faces_only: false,
        auto_smooth: 0.0,
        ..Brush::default()
    }
}

/// **A lei que o VERBO semeia** — a mesma porta que o editor usa ao soltar o
/// gesto ([`Verb::filter_kind`]).
///
/// ⚠️ **O motor recebe a LEI, não o verbo**, desde que o catálogo de filtros
/// deixou de ser a projecção dos verbos (o `Scale`, o `Sphere` e o `Random`
/// não têm verbo nenhum). Estes gates continuam a exercitar a rota do verbo
/// porque é a que eles sempre mediram; as três leis sem verbo têm gates
/// próprios.
fn seeded(b: &Brush) -> crate::FilterKind {
    b.verb.filter_kind().expect("o verbo deste gate filtra")
}

/// Os verbos que a tabela nomeia como filtros — perguntados ao motor, nunca
/// escritos aqui.
///
/// ⚠️ **Enumerá-los neste arquivo seria a segunda cópia da tabela**, e ela
/// ficaria VERDE sobre o dia em que o quinto entrasse — a lição que o
/// `stroke_apply_tests` já pagou com a lista de grips.
fn filtering_verbs() -> Vec<Verb> {
    Verb::ALL.into_iter().filter(|v| v.filters_mesh()).collect()
}

/// O deslocamento de cada vértice contra `before`.
fn deltas(before: &[[f32; 3]], mesh: &Mesh) -> Vec<[f32; 3]> {
    before
        .iter()
        .zip(mesh.positions())
        .map(|(b, p)| [p[0] - b[0], p[1] - b[1], p[2] - b[2]])
        .collect()
}

fn norm(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// **A tabela tem conteúdo** — o anti-vácuo de todo gate abaixo que a varre.
#[test]
fn the_filter_table_names_the_four_laws_and_nothing_else() {
    let verbs = filtering_verbs();
    assert_eq!(
        verbs.len(),
        4,
        "a tabela do filtro mudou de tamanho: {verbs:?}"
    );
    // ⚠️ A lei de cada um é perguntada, e a asserção é que as QUATRO são
    // DISTINTAS: duas entradas com a mesma lei seriam duas portas para o mesmo
    // deslocamento, que é exactamente por que o Sharpen e o Layer ficaram fora.
    let mut kinds: Vec<crate::FilterKind> = verbs
        .iter()
        .map(|v| v.filter_kind().expect("lei"))
        .collect();
    let n = kinds.len();
    kinds.dedup();
    assert_eq!(kinds.len(), n, "duas entradas do filtro rodam a MESMA lei");
}

/// ⭐ **O gate da wave: o filtro e o pincel rodam a MESMA LEI.**
///
/// Os dois campos vetoriais têm de ser o mesmo a menos de **UM** escalar — não
/// um por vértice. É isso que *"não há kernel novo"* quer dizer em código: o
/// pincel compõe o alvo com o peso do dab e o atenua pelo `accum`, o filtro
/// enche o `accum` de uma vez, e o que sobra da diferença é uma constante.
///
/// ⚠️ **A razão é UM escalar e não uma tolerância frouxa:** com a curva
/// constante todo vértice leva o mesmo peso, então `brush_delta = k ·
/// filter_delta` vale **por construção** se — e só se — as duas rotas
/// escreverem o mesmo alvo. Um kernel trocado (Smooth por Sharpen) inverte o
/// sinal de `k` num vértice e não noutro, e o resíduo dispara.
#[test]
fn the_filter_runs_the_same_law_as_the_brush() {
    for verb in filtering_verbs() {
        let brush = filter_brush(verb);

        // (a) O PINCEL: um dab só, cobrindo a malha inteira.
        let mut m_brush = mesh_for(verb);
        let before = snapshot(&m_brush);
        let mut s_brush = SculptStroke::default();
        s_brush.begin(&m_brush);
        let dab = dab_at([0.0, 0.0, 0.0], brush.radius);
        let hit = s_brush.dab(&mut m_brush, &brush, &dab, Symmetry::default());
        assert!(hit > 0, "{verb:?}: o dab não tocou nada");
        let d_brush = deltas(&before, &m_brush);

        // (b) O FILTRO, sobre a MESMA malha de partida.
        let mut m_filter = mesh_for(verb);
        let mut s_filter = SculptStroke::default();
        s_filter.filter_begin(&m_filter);
        let wrote = s_filter.filter(&mut m_filter, &brush, seeded(&brush), 0.5);
        assert_eq!(
            wrote,
            m_filter.vert_count(),
            "{verb:?}: o filtro tem de escrever a malha inteira"
        );
        let d_filter = deltas(&before, &m_filter);

        // A razão sai do vértice que o FILTRO mais moveu — o de melhor razão
        // sinal/ruído, e não um índice escolhido à mão.
        let (pivot, best) = d_filter
            .iter()
            .enumerate()
            .map(|(i, d)| (i, norm(*d)))
            .fold((0, 0.0f32), |a, b| if b.1 > a.1 { b } else { a });
        assert!(best > 1e-5, "{verb:?}: o filtro não moveu nada");
        let k = norm(d_brush[pivot]) / best;
        assert!(k > 0.0, "{verb:?}: as duas rotas discordam da MAGNITUDE");

        let scale = d_brush.iter().map(|d| norm(*d)).fold(0.0f32, f32::max);
        let worst = d_brush
            .iter()
            .zip(&d_filter)
            .map(|(b, f)| norm([b[0] - k * f[0], b[1] - k * f[1], b[2] - k * f[2]]))
            .fold(0.0f32, f32::max);
        // ⚠️ **A barra é RELATIVA à excursão do próprio verbo**, e não um número
        // absoluto: os quatro deslocam ordens de grandeza diferentes (o
        // SurfaceSmooth mede 150× menos que o Smooth na mesma esfera, medido no
        // `mesh_for`), e um número absoluto seria frouxo num e inatingível
        // noutro.
        assert!(
            worst <= 2e-3 * scale,
            "{verb:?}: os dois campos NÃO são o mesmo a menos de um escalar \
             (resíduo {worst:.6e} contra excursão {scale:.6e}, k = {k:.6})"
        );
    }
}

/// **O arrasto de volta devolve a pose EXACTA.**
///
/// É o que o `accum = 1` mais a âncora no `pre` compram, e o que a referência
/// paga com o `reset_translations_to_original`. Sem ele um arrasto seria uma
/// COMPOSIÇÃO de filtros e o resultado dependeria de quantos eventos o rato
/// mandou.
///
/// ⚠️ **A comparação é `==` e não `to_bits()`**: um vértice em `-0.0` sai como
/// `+0.0` da soma neutra, e os dois **são** o mesmo ponto.
#[test]
fn the_drag_back_returns_the_pose_exactly() {
    for verb in filtering_verbs() {
        let brush = filter_brush(verb);
        let mut mesh = mesh_for(verb);
        let before = snapshot(&mesh);
        let mut stroke = SculptStroke::default();
        stroke.filter_begin(&mesh);

        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.6);
        let moved = max_shift(&before, &mesh);
        assert!(moved > 1e-5, "{verb:?}: a ida não moveu nada");
        // Um passo intermédio, para o caminho de volta não ser o mesmo da ida.
        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.2);
        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.0);

        for (i, (b, p)) in before.iter().zip(mesh.positions()).enumerate() {
            assert!(
                b[0] == p[0] && b[1] == p[1] && b[2] == p[2],
                "{verb:?}: o vértice {i} não voltou ({b:?} contra {p:?})"
            );
        }
    }
}

/// **Aplicar duas vezes é aplicar uma vez** — o `pre` é congelado, então o
/// resultado é função do arrasto e não da contagem de eventos. É a mesma
/// propriedade que o envelope dá ao carimbo, um gesto acima.
#[test]
fn applying_the_same_amount_twice_is_applying_it_once() {
    for verb in filtering_verbs() {
        let brush = filter_brush(verb);
        let mut mesh = mesh_for(verb);
        let mut stroke = SculptStroke::default();
        stroke.filter_begin(&mesh);
        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.4);
        let once = snapshot(&mesh);
        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.4);
        for (i, (a, p)) in once.iter().zip(mesh.positions()).enumerate() {
            assert!(
                a[0] == p[0] && a[1] == p[1] && a[2] == p[2],
                "{verb:?}: o vértice {i} andou na segunda aplicação"
            );
        }
    }
}

/// **O ARRASTO é o gesto, e ele passa por forças intermédias** — chegar a `0,4`
/// pela rampa `0,1 → 0,25 → 0,4` tem de pousar onde uma chamada única de `0,4`
/// pousa, **AO BIT**.
///
/// ⚠️ **Estritamente mais forte que o irmão da idempotência**, e é por isso que
/// os dois existem: aquele repete a MESMA força, este atravessa forças
/// diferentes — que é o que um rato de facto manda enquanto a mão anda. Um
/// filtro que lesse o anel VIVO passaria a somar a difusão de cada passo, e o
/// desenho final dependeria de quantos eventos o arrasto produziu.
///
/// ⚠️ **A barra é a igualdade EXACTA e não um épsilon**, porque a lei não é
/// *"converge"* e sim *"cada chamada parte da MESMA pose com a MESMA força"* ⇒
/// as duas produzem a mesma aritmética, na mesma ordem, sobre os mesmos bits.
#[test]
fn a_ramped_drag_lands_where_a_single_step_lands() {
    for verb in filtering_verbs() {
        let brush = filter_brush(verb);

        let mut direct = mesh_for(verb);
        let mut a = SculptStroke::default();
        a.filter_begin(&direct);
        a.filter(&mut direct, &brush, seeded(&brush), 0.4);
        let target = snapshot(&direct);

        let mut ramped = mesh_for(verb);
        let before = snapshot(&ramped);
        let mut b = SculptStroke::default();
        b.filter_begin(&ramped);
        for f in [0.1, 0.25, 0.4] {
            b.filter(&mut ramped, &brush, seeded(&brush), f);
        }

        // CONTROLE: sem isto, um filtro que não movesse nada satisfaria a
        // comparação por vácuo.
        assert!(
            max_shift(&before, &ramped) > 1e-5,
            "{verb:?}: a rampa não moveu nada"
        );
        for (i, (t, p)) in target.iter().zip(ramped.positions()).enumerate() {
            assert!(
                t[0] == p[0] && t[1] == p[1] && t[2] == p[2],
                "{verb:?}: o vértice {i} depende do CAMINHO do arrasto ({t:?} contra {p:?})"
            );
        }
    }
}

/// **A máscara freia o filtro** — e a metade que importa é a do CONTROLE: um
/// vértice livre da mesma malha tem de andar.
#[test]
fn the_mask_holds_the_filter_back() {
    for verb in filtering_verbs() {
        let brush = filter_brush(verb);
        let mut mesh = mesh_for(verb);
        // Metade da malha protegida, metade livre.
        let split = mesh.vert_count() / 2;
        {
            let m = mesh.masks_mut();
            for v in m.iter_mut().take(split) {
                *v = 1.0;
            }
        }
        let before = snapshot(&mesh);
        let mut stroke = SculptStroke::default();
        stroke.filter_begin(&mesh);
        stroke.filter(&mut mesh, &brush, seeded(&brush), 0.6);

        let d = deltas(&before, &mesh);
        let masked = d[..split].iter().map(|v| norm(*v)).fold(0.0f32, f32::max);
        let free = d[split..].iter().map(|v| norm(*v)).fold(0.0f32, f32::max);
        assert_eq!(masked, 0.0, "{verb:?}: a máscara não freou o filtro");
        assert!(free > 1e-5, "{verb:?}: o CONTROLE não andou");
    }
}

/// **Um verbo que não é filtro não SEMEIA lei nenhuma** — e as duas metades
/// têm de concordar.
///
/// ⚠️ **A PREMISSA DESTE GATE MUDOU com a W9b, e a mudança está escrita em vez
/// de escondida.** Ele afirmava *"o motor devolve `0` para um verbo que não
/// filtra"*, e o motor deixou de ter opinião sobre verbos quando passou a
/// receber a LEI (o `Scale`, o `Sphere` e o `Random` não têm verbo por onde
/// passar). A recusa mudou de casa: quem a faz é o shell, no
/// `filter_at`, e ela tem gate PRÓPRIO lá.
///
/// ⚠️ **O que sobra aqui é a metade que o crate PODE afirmar, e ela é
/// load-bearing:** o [`Verb::filters_mesh`] é lido pelo painel para OFERECER o
/// gesto e o [`Verb::filter_kind`] pelo shell para o HONRAR — dois leitores de
/// uma resposta. Se eles divergirem, o artista vê um botão que não faz nada,
/// ou perde um que faria.
#[test]
fn the_seed_and_the_offer_agree_on_every_verb() {
    for verb in Verb::ALL {
        assert_eq!(
            verb.filter_kind().is_some(),
            verb.filters_mesh(),
            "{verb:?}: o que o painel OFERECE e o que o shell HONRA divergem"
        );
    }
}

/// **Sem [`SculptStroke::filter_begin`] o filtro não corre**, e a guarda é
/// DERIVADA (a captura cobre a malha?) em vez de um flag.
///
/// ⚠️ **O caso perigoso é o segundo:** um traço NORMAL deixa o `touched` com a
/// pegada dele, e um filtro que corresse ali escreveria sobre o `pre` de outro
/// gesto — parcialmente, só nos vértices que o pincel tocou.
#[test]
fn the_filter_refuses_a_stroke_that_did_not_freeze_the_whole_mesh() {
    let brush = filter_brush(Verb::Smooth);
    let mut mesh = mesh_for(Verb::Smooth);
    let before = snapshot(&mesh);

    // (a) traço nunca começado.
    let mut fresh = SculptStroke::default();
    assert_eq!(fresh.filter(&mut mesh, &brush, seeded(&brush), 0.5), 0);

    // (b) um traço NORMAL, cuja pegada é uma calota.
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let small = Brush {
        radius: 0.25,
        ..filter_brush(Verb::Smooth)
    };
    let dab = dab_at([0.0, 0.0, 1.0], small.radius);
    assert!(stroke.dab(&mut mesh, &small, &dab, Symmetry::default()) > 0);
    let after_dab = snapshot(&mesh);
    assert!(
        after_dab.len() > stroke.footprint_len(),
        "a fixture não contém o fenômeno: o dab cobriu a malha inteira"
    );
    assert_eq!(stroke.filter(&mut mesh, &brush, seeded(&brush), 0.5), 0);
    for (a, p) in after_dab.iter().zip(mesh.positions()) {
        assert!(a[0] == p[0] && a[1] == p[1] && a[2] == p[2]);
    }
    assert!(max_shift(&before, &mesh) > 0.0, "o CONTROLE não pintou");
}

/// **O filtro de afiar É o de alisar arrastado para trás** — a medição que
/// tornou a exclusão do [`Verb::Sharpen`] um número em vez de uma opinião.
///
/// ⚠️ **As duas expressões são a mesma álgebra e NÃO os mesmos bits:**
/// `smooth(−f)` escreve `live·(1+f) − média·f` e `sharpen(f)` escreve
/// `live + (live − média)·f`. A barra mede o piso do `f32` sobre esta malha, e o
/// **CONTROLE** — a distância ao `smooth(+f)`, que é o mesmo verbo para o outro
/// lado — é o que impede o gate de passar por vácuo.
#[test]
fn the_sharpen_filter_is_the_smooth_filter_dragged_backwards() {
    let mesh0 = mesh_for(Verb::Smooth);
    let before = snapshot(&mesh0);

    // (a) O filtro do Smooth, arrastado para TRÁS.
    let mut back = mesh0.clone();
    let mut s = SculptStroke::default();
    s.filter_begin(&back);
    s.filter(
        &mut back,
        &filter_brush(Verb::Smooth),
        seeded(&filter_brush(Verb::Smooth)),
        -1.0,
    );

    // (b) O PINCEL do Sharpen, com peso cheio — `accum = 1` nos dois, então o
    // alvo é a posição final dos dois lados.
    let mut sharp = mesh0.clone();
    let mut t = SculptStroke::default();
    t.begin(&sharp);
    let brush = filter_brush(Verb::Sharpen);
    let dab = dab_at([0.0, 0.0, 0.0], brush.radius);
    assert!(t.dab(&mut sharp, &brush, &dab, Symmetry::default()) > 0);

    // (c) O CONTROLE: o mesmo filtro para a FRENTE.
    let mut fwd = mesh0.clone();
    let mut u = SculptStroke::default();
    u.filter_begin(&fwd);
    u.filter(
        &mut fwd,
        &filter_brush(Verb::Smooth),
        seeded(&filter_brush(Verb::Smooth)),
        1.0,
    );

    let d_back = deltas(&before, &back);
    let d_sharp = deltas(&before, &sharp);
    let d_fwd = deltas(&before, &fwd);
    let scale = d_back.iter().map(|v| norm(*v)).fold(0.0f32, f32::max);
    assert!(scale > 1e-4, "o arrasto para trás não moveu nada");

    let same = d_back
        .iter()
        .zip(&d_sharp)
        .map(|(a, b)| norm([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0f32, f32::max);
    let control = d_back
        .iter()
        .zip(&d_fwd)
        .map(|(a, b)| norm([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0f32, f32::max);
    assert!(
        same <= 1e-5 * scale.max(1.0),
        "afiar e alisar-para-trás divergiram por {same:.6e} (excursão {scale:.6e})"
    );
    assert!(
        control > 0.5 * scale,
        "o CONTROLE não separa nada: alisar para a frente difere por {control:.6e}"
    );
}

/// **Arrastar para o lado errado não faz nada no Relax nem no HC**, e é a lei
/// da referência (`clamp_factors(factors, 0.0f, 1.0f)`): não existe a operação
/// inversa de redistribuir. O **CONTROLE** é o mesmo verbo para o lado certo.
#[test]
fn the_one_sided_filters_ignore_a_backwards_drag() {
    for verb in [Verb::SlideRelax, Verb::SurfaceSmooth] {
        let brush = filter_brush(verb);
        let mut mesh = mesh_for(verb);
        let before = snapshot(&mesh);
        let mut stroke = SculptStroke::default();
        stroke.filter_begin(&mesh);

        stroke.filter(&mut mesh, &brush, seeded(&brush), -1.0);
        assert_eq!(
            max_shift(&before, &mesh),
            0.0,
            "{verb:?} andou para o lado que a referência clampa"
        );
        stroke.filter(&mut mesh, &brush, seeded(&brush), 1.0);
        assert!(
            max_shift(&before, &mesh) > 1e-5,
            "{verb:?}: o CONTROLE não andou"
        );
    }
}
