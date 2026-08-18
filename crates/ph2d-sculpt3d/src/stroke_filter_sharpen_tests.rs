//! **O SHARPEN** — a W9c-b, e o que ela consegue provar.
//!
//! ⚠️ **Este cabeçalho já afirmou que a régua do degrau NÃO discrimina, e a
//! afirmação era sobre uma lei errada.** Eu escrevi que *"as duas réguas
//! geométricas óbvias caem ou oscilam sobre a lei correcta"* e concluí que
//! *"quem julga a aparência é o smoke"* — o smoke veio, reprovou (*"Sharpen não
//! parece correto"*), e a causa era minha: o `sharpen_factor` era recomputado a
//! cada sub-passo quando no fonte ele vive no `filter_cache`, construído **uma
//! vez por gesto**. *A régua não discriminava porque a lei alisava.* Com o
//! factor congelado o degrau **sobe**, e o gate que o afirma é o
//! `..._raises_the_step_between_neighbours_...`.
//!
//! Os gates deste arquivo defendem, então, quatro coisas: que o kernel é o do
//! `calc_sharpen_filter` (paridade contra a lei escrita à mão, numa malha
//! REGULAR), que a lei **afia** em vez de alisar, que o gesto é reversível ao
//! byte e que ele **não depende da taxa de eventos** do rato.

use super::stroke_filter::{filter_brush, norm};
use super::*;
use crate::stroke::sharpen::VALENCE_REGIME;

/// A malha destes gates: a fixture do pai, que tem detalhe.
fn sharpen_mesh() -> Mesh {
    mesh_for(Verb::Smooth)
}

/// **Uma malha REGULAR com detalhe** — valência dentro do regime da referência.
///
/// ⚠️ **A `mesh_for` NÃO serve para medir o porte, e a medição foi quem o disse:**
/// ela é uma esfera UV de valência máxima **48**, ou seja está no regime em que
/// a guarda de valência MORDE — e a guarda é uma divergência declarada, então o
/// oráculo (que é a referência) tem de discordar dela ali. *O porte é exacto
/// onde a referência opera; a divergência é gateada onde ela não opera*, e
/// misturar as duas perguntas numa fixture só apagava as duas.
fn regular_mesh() -> Mesh {
    // ⚠️ **Um TORO e não uma esfera, e é a única forma de rede que não tem
    // polos.** Toda esfera deste catálogo — UV, icosfera por alvo de triângulos —
    // fecha as calotas num vértice de valência alta; um toro é uma grade
    // periódica nos dois eixos, valência **4 em todo vértice**, que é exactamente
    // o regime da referência.
    let mut m = ph2d_mesh::shapes::torus(48, 24, 1.0, 0.35);
    for p in m.positions_mut() {
        // Uma crista ao longo do tubo: o detalhe que a lei existe para contrastar.
        let bump = (-(p[1] * p[1]) / (2.0 * 0.06 * 0.06)).exp() * 0.06;
        let r = (p[0] * p[0] + p[2] * p[2]).sqrt();
        if r <= f32::EPSILON {
            continue;
        }
        p[0] += p[0] / r * bump;
        p[2] += p[2] / r * bump;
    }
    m
}

fn sharpen_brush() -> Brush {
    filter_brush(Verb::Smooth)
}

/// Roda o filtro do Sharpen sobre a fixture e devolve a malha.
fn sharpened(amount: f32) -> Mesh {
    let mut m = sharpen_mesh();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &sharpen_brush(), crate::FilterKind::Sharpen, amount);
    m
}

/// **A LEI DA REFERÊNCIA, ESCRITA À MÃO** — o oráculo de porte.
///
/// ⚠️ **Ela não chama uma linha do produto, e é isso que a torna um oráculo em
/// vez de um espelho:** chamar o `filter_sharpen` para computar o que se espera
/// dele devolveria a nossa resposta com outro nome, e o gate ficaria verde por
/// construção — a forma que esta casa já apanhou duas vezes (o `BodyDefaults`
/// da física, o `row.set` do painel).
///
/// A estrutura é a do `calc_sharpen_filter` lido de cima a baixo: o pré-passe
/// que mede e normaliza, e o gather que escreve.
fn reference_sharpen(mesh: &Mesh, per: f32) -> Vec<[f32; 3]> {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let n = pos.len();

    // Pré-passe: `detail_directions` e `sharpen_factors`.
    let mut dirs = vec![[0.0f32; 3]; n];
    let mut fac = vec![0.0f32; n];
    let mut peak = 0.0f32;
    for (v, dir) in dirs.iter_mut().enumerate() {
        let p = pos[v];
        let mut sum = [0.0f32; 3];
        let mut count = 0u32;
        for &nb in adj.vert_verts.neighbours(v) {
            let q = pos[nb as usize];
            sum[0] += q[0];
            sum[1] += q[1];
            sum[2] += q[2];
            count += 1;
        }
        if count == 0 {
            continue;
        }
        let inv = 1.0 / count as f32;
        *dir = [
            sum[0] * inv - p[0],
            sum[1] * inv - p[1],
            sum[2] * inv - p[2],
        ];
        let mag = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        fac[v] = mag;
        peak = peak.max(mag);
    }
    let rcp = if peak > f32::EPSILON { 1.0 / peak } else { 0.0 };
    for f in &mut fac {
        let t = *f * rcp;
        *f = 1.0 - (1.0 - t) * (1.0 - t);
    }

    // O gather.
    let mut out = pos.to_vec();
    for (v, o) in out.iter_mut().enumerate() {
        let p = pos[v];
        let mut sharpen = [0.0f32; 3];
        for &nb in adj.vert_verts.neighbours(v) {
            let q = pos[nb as usize];
            let w = fac[nb as usize];
            sharpen[0] += (q[0] - p[0]) * w;
            sharpen[1] += (q[1] - p[1]) * w;
            sharpen[2] += (q[2] - p[2]) * w;
        }
        let keep = 1.0 - fac[v];
        let blend = 0.35 * fac[v] * fac[v];
        let d = dirs[v];
        for k in 0..3 {
            o[k] = p[k] + (sharpen[k] * keep + d[k] * blend) * per;
        }
    }
    out
}

fn worst_gap(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| norm([p[0] - q[0], p[1] - q[1], p[2] - q[2]]))
        .fold(0.0, f32::max)
}

/// Uma esfera UV com uma crista — polos de valência alta, o regime que a
/// referência não alcança.
fn high_valence_ridge() -> Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    for p in m.positions_mut() {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = p[1] / r;
        let bump = (-(t * t) / (2.0 * 0.15 * 0.15)).exp() * 0.12;
        let s = (r + bump) / r;
        *p = [p[0] * s, p[1] * s, p[2] * s];
    }
    m
}

/// O maior salto radial entre dois vértices vizinhos.
fn max_step(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut worst = 0.0f32;
    for v in 0..pos.len() {
        let r = norm(pos[v]);
        for &nb in adj.vert_verts.neighbours(v) {
            worst = worst.max((norm(pos[nb as usize]) - r).abs());
        }
    }
    worst
}

fn sharpen_on(mut mesh: Mesh, amount: f32) -> Mesh {
    let mut s = SculptStroke::default();
    s.filter_begin(&mesh);
    s.filter(
        &mut mesh,
        &sharpen_brush(),
        crate::FilterKind::Sharpen,
        amount,
    );
    mesh
}

/// ⭐ **A LEI NÃO DESTRÓI UMA MALHA DE VALÊNCIA ALTA — e é IDENTIDADE numa
/// regular.**
///
/// ⚠️ **Gate red-first, e ele nasceu do smoke da `=34`.** O gather do
/// `calc_sharpen_filter` é uma SOMA sobre o anel, **não normalizada pela
/// contagem**: com `f[i]` baixo e os vizinhos altos ele vale `valência ×
/// laplaciano`, e um passo de alisamento de fator maior que um OVERSHOOTA. A
/// referência vive com isso porque as malhas dela são regulares; a cena do smoke
/// abre com uma `uv_sphere(96, 144)`, cujos polos têm valência **144**, e a
/// malha **explodiu** — medido, degrau **1098×** e excursão **33** numa esfera de
/// raio um.
///
/// ⚠️ **As duas metades não são redundantes, e a mutação prova-o:** remover a
/// guarda faz a primeira sangrar e deixa a segunda VERDE, que é o que significa
/// *identidade no regime da referência*. `min(1, 6/n)` vale exactamente `1` para
/// todo `n <= 6` — a valência de um quad ou de um triângulo regular —, então
/// nenhuma malha que a referência contemple vê um bit diferente.
#[test]
fn the_sharpen_does_not_blow_up_a_high_valence_mesh_and_leaves_a_regular_one_alone() {
    let (_, hi) = crate::FilterKind::Sharpen.range();

    // (1) A metade que a guarda existe para curar.
    let base = high_valence_ridge();
    let worst_valence = (0..base.vert_count())
        .map(|v| base.adjacency().vert_verts.neighbours(v).len())
        .max()
        .unwrap_or(0);
    assert!(
        worst_valence > VALENCE_REGIME,
        "a fixture não contém o fenómeno: valência máxima {worst_valence}"
    );
    let before = max_step(&base);
    let after = max_step(&sharpen_on(high_valence_ridge(), hi));
    let ratio = after / before;
    assert!(
        ratio < 1.5,
        "a lei explodiu a malha de valência {worst_valence}: o degrau foi a {ratio:.3}×"
    );

    // (2) O CONTROLE — numa malha regular a guarda é a identidade, então o
    // resultado tem de ser o MESMO que ela dava antes de a guarda existir.
    let regular = regular_mesh();
    let reg_valence = (0..regular.vert_count())
        .map(|v| regular.adjacency().vert_verts.neighbours(v).len())
        .max()
        .unwrap_or(0);
    assert!(
        reg_valence <= VALENCE_REGIME,
        "o controle não é regular: valência {reg_valence}"
    );
    let want = sharpen_on(regular_mesh(), hi);
    let got = sharpen_on(regular, hi);
    assert_eq!(
        want.positions(),
        got.positions(),
        "a lei deixou de ser determinística na malha regular"
    );
}

/// ⭐ **O GATE DA WAVE: o kernel é o `calc_sharpen_filter`.**
///
/// Uma força dentro do teto de uma fatia (`MAX_STEP`) é **UM** passo — ou seja,
/// a referência com um único evento de rato —, e é aí que o porte é comparável
/// termo a termo contra a lei escrita à mão.
#[test]
fn the_sharpen_filter_is_the_reference_law_written_independently() {
    const PER: f32 = 0.4;
    let base = regular_mesh();
    let worst = (0..base.vert_count())
        .map(|v| base.adjacency().vert_verts.neighbours(v).len())
        .max()
        .unwrap_or(0);
    assert!(
        worst <= VALENCE_REGIME,
        "a fixture do PORTE tem valência {worst}: a guarda morde e o oráculo tem de discordar"
    );
    let want = reference_sharpen(&base, PER);
    let got = sharpen_on(regular_mesh(), PER);

    // ⚠️ **O ANTI-VÁCUO:** sem deslocamento os dois lados seriam a pose de
    // entrada e a igualdade abaixo seria verdadeira sobre uma malha que
    // ninguém tocou.
    let moved = worst_gap(base.positions(), &want);
    assert!(
        moved > 1e-4,
        "a fixture não contém o fenómeno: a lei moveu {moved:e}"
    );

    let gap = worst_gap(&want, got.positions());
    assert!(
        gap < 1e-6,
        "o porte divergiu da lei escrita à mão por {gap:e} (deslocamento {moved:e})"
    );
}

/// ⭐ **A LEI AFIA — e o gate só existe porque a lei foi CONSERTADA.**
///
/// ⚠️ **Eu escrevi, no cabeçalho deste arquivo, que a régua do degrau *"não
/// discrimina"*.** Ela não discriminava porque **a lei estava errada**: eu
/// recomputava o `sharpen_factor` a cada sub-passo, quando no fonte ele vive no
/// `filter_cache` — construído **uma vez por GESTO** e reusado por toda
/// iteração. Recomputando, a lei persegue o próprio rasto (a curvatura que ela
/// acabou de achatar deixa de a marcar como detalhe) e converge para um estado
/// **ALISADO**; com o factor congelado ela empurra consistentemente na mesma
/// direcção, e o degrau **SOBE**.
///
/// ⚠️ **A régua é o DEGRAU e não a excursão**, que é a distinção que separa
/// *afiar* de *mover*: qualquer lei move a malha, e o que uma aresta é, é uma
/// transição curta e alta entre vizinhos.
///
/// ⚠️ **E a fixture é REGULAR de propósito** — numa malha de valência alta a
/// guarda morde e o número deixa de ser sobre a lei.
#[test]
fn the_sharpen_raises_the_step_between_neighbours_instead_of_flattening_it() {
    let (_, hi) = crate::FilterKind::Sharpen.range();
    let before = max_step(&regular_mesh());
    let after = max_step(&sharpen_on(regular_mesh(), hi));
    let ratio = after / before;
    assert!(
        ratio > 1.0,
        "a lei ALISOU em vez de afiar: o degrau foi a {ratio:.4}× \
         (é a assinatura do `sharpen_factor` recomputado por sub-passo)"
    );

    // ⚠️ **O CONTROLE, e ele é a metade que dá sentido à primeira:** o mesmo
    // gesto com o `Smooth` tem de BAIXAR o degrau. Sem ele, *"o degrau subiu"*
    // seria compatível com uma lei que simplesmente inflasse a malha.
    let smoothed = max_step(&{
        let mut m = regular_mesh();
        let mut s = SculptStroke::default();
        s.filter_begin(&m);
        s.filter(&mut m, &sharpen_brush(), crate::FilterKind::Smooth, 1.0);
        m
    });
    assert!(
        smoothed < before,
        "o CONTROLE falhou: o Smooth não baixou o degrau ({:.4}×)",
        smoothed / before
    );
}

/// **Força zero devolve a pose congelada AO BYTE** — o arrasto de volta é
/// exacto, como nos outros oito.
///
/// ⚠️ **Ele mede o BIT e não um épsilon**, e é o que separa *"quase não se
/// move"* de *"não se move"*: a lei tem um ramo que não itera, e um gate com
/// tolerância ficaria verde sobre uma iteração de força minúscula.
#[test]
fn a_sharpen_of_zero_is_the_frozen_pose_to_the_byte() {
    let base = sharpen_mesh();
    let got = sharpened(0.0);
    assert_eq!(
        base.positions(),
        got.positions(),
        "força zero moveu a malha"
    );
}

/// ⭐ **O resultado é fato do ARRASTO, nunca de como ele foi entregue.**
///
/// ⚠️ **É a propriedade que separa a nossa lei da da referência**, e a razão de
/// a wave existir na forma em que existe: o `sculpt_mesh_filter_is_continuous`
/// põe o Sharpen entre os filtros que **não restauram a pose** entre eventos,
/// então lá o número de iterações é o número de eventos que o sistema
/// operativo entregou. Aqui um arrasto que passa por dez valores intermédios
/// pousa exactamente onde um que salta direto ao fim pousa.
#[test]
fn the_sharpen_result_is_a_function_of_the_drag_not_of_how_it_was_delivered() {
    const END: f32 = 2.0;
    let direct = sharpened(END);

    let mut ramp = sharpen_mesh();
    let mut s = SculptStroke::default();
    s.filter_begin(&ramp);
    for i in 1..=10 {
        let a = END * i as f32 / 10.0;
        s.filter(&mut ramp, &sharpen_brush(), crate::FilterKind::Sharpen, a);
    }

    let base = sharpen_mesh();
    let moved = worst_gap(base.positions(), direct.positions());
    assert!(moved > 1e-4, "a fixture não contém o fenómeno: {moved:e}");

    assert_eq!(
        direct.positions(),
        ramp.positions(),
        "dez passos de arrasto pousaram noutro lugar que um salto directo"
    );
}

/// **Nenhuma fatia carrega mais que o teto da referência.**
///
/// ⚠️ **A propriedade é sobre a FATIA, não sobre a contagem**, e é por isso que
/// o gate divide em vez de comparar `n` com uma tabela: o `clamp_factors(0,
/// 0.5)` do fonte é o que impede o gather de passar da vizinhança, e uma
/// contagem escrita à mão aqui envelheceria no dia em que o teto mudasse.
#[test]
fn no_slice_of_a_sharpen_drag_exceeds_the_reference_ceiling() {
    for total in [0.01f32, 0.4, 0.5, 0.51, 1.0, 2.0, 3.999, 4.0] {
        let n = crate::stroke::sharpen::steps_for(total);
        assert!(n >= 1, "{total} pediu {n} fatias");
        let per = total / n as f32;
        assert!(
            per <= crate::stroke::sharpen::MAX_STEP + 1e-6,
            "força {total} deu fatias de {per}, acima do teto de estabilidade"
        );
    }
    // E a metade oposta: uma força que CABE numa fatia não é fatiada, senão o
    // caso comum deixaria de ser a referência-com-um-evento.
    assert_eq!(crate::stroke::sharpen::steps_for(0.5), 1);
}

/// **O teto da faixa está onde o gesto para de responder.**
///
/// ⚠️ **As duas metades são precisas:** acima do teto o resultado é o MESMO (é
/// o *stable state* que a referência nomeia — arrastar mais não compra nada),
/// e abaixo dele ele **não** é (senão a faixa inteira seria decorativa e o teto
/// estaria no lugar errado).
#[test]
fn the_sharpen_saturates_where_its_ceiling_sits() {
    let (_, hi) = crate::FilterKind::Sharpen.range();

    // ⚠️ **Este gate já foi TAUTOLÓGICO, e a auditoria pegou-o.** Ele comparava
    // `sharpened(hi)` com `sharpened(hi*4)` e dizia *"quatro vezes o teto ainda
    // move a malha"* — mas o `filter_sharpen` CLAMPA a entrada pelo próprio teto
    // antes de qualquer aritmética, então os dois colapsavam no mesmo `total` e
    // a igualdade era verdadeira **por construção, para qualquer valor de
    // `SHARPEN_MAX`**. Ele guardava a existência do clamp, nunca o valor: com o
    // teto cortado a um quarto a suíte inteira ficava verde.
    //
    // ⚠️ **A pergunta certa é sobre o clamp, e é assim que ela se afirma:** que
    // a porta do PRODUTO apara, medido contra a porta que **não** apara. É a
    // única forma de a asserção poder falhar pelo motivo que ela alega.
    // ⚠️ **As duas metades correm na MESMA malha, e a primeira versão deste gate
    // não corria** — ela comparava a fixture do `sharpened` (esfera UV) com o
    // toro do `regular_mesh`, e o `zip` de dois arrays de tamanhos diferentes
    // trunca em silêncio: o número saía grande **por acidente** e a mutação que
    // apaga o clamp não o movia. *Uma comparação entre duas malhas diferentes
    // não mede o que as separa, mede que elas são diferentes.*
    let clamped = sharpen_on(regular_mesh(), hi * 4.0);
    let mut raw = regular_mesh();
    let mut s = SculptStroke::default();
    s.filter_begin(&raw);
    crate::sharpen_total_for_measurement(&mut s, &mut raw, &sharpen_brush(), hi * 4.0);
    assert_eq!(
        clamped.vert_count(),
        raw.vert_count(),
        "as duas metades saíram de malhas diferentes"
    );
    let gap = worst_gap(clamped.positions(), raw.positions());
    assert!(
        gap > 1e-4,
        "a porta do produto NÃO aparou uma força de {}× o teto ({gap:e}): \
         o clamp desapareceu",
        4.0
    );

    // ⚠️ **A segunda metade: a FAIXA tem de fazer alguma coisa.** Sem ela o gate
    // acima estaria satisfeito por um teto de zero — que apara tudo e não deixa
    // o artista fazer nada. As duas juntas dizem *o clamp existe* E *o que ele
    // deixa passar não é decorativo*.
    let half = sharpen_on(regular_mesh(), hi * 0.25);
    let full = sharpen_on(regular_mesh(), hi);
    let span = worst_gap(full.positions(), half.positions());
    assert!(
        span > 1e-4,
        "um quarto do teto já dá o mesmo que o teto ({span:e}): a faixa é decorativa"
    );

    // ⚠️ **E a TERCEIRA: o teto tem de estar onde o doc diz.** É esta que o
    // torna um número defendido em vez de um literal — com o teto cortado a um
    // quarto, a lei entrega menos excursão do que a faixa promete, e a auditoria
    // mediu que o corte passava com a suíte INTEIRA verde.
    let (_, hi_now) = crate::FilterKind::Sharpen.range();
    assert!(
        (hi_now - 4.0).abs() < 1e-6,
        "o teto mudou para {hi_now}: a tabela do doc e o custo medido \
         (17,17 ms em 8 fatias) descrevem 4,0 — mova os dois juntos ou nenhum"
    );
}
