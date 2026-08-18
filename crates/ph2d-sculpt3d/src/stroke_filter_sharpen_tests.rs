//! **O SHARPEN** — a W9c-b, e o que ela consegue provar.
//!
//! ⚠️ **O gate central é de PORTE, não de aparência, e a distinção é honesta.**
//! *"Isto afia?"* é uma pergunta sobre o que o olho vê, e a sonda
//! `tests/measure_sharpen_law.rs` mediu que as duas réguas geométricas óbvias
//! (o degrau entre vizinhos e a largura da crista) **não a respondem** — elas
//! caem ou oscilam sobre a lei correcta, porque metade do mecanismo desta lei é
//! *achatar o pico* e a outra metade é *puxar o terreno até ele*. Um gate
//! escrito sobre uma dessas réguas seria um gate que não pode falhar pelo
//! motivo que alega. **Quem julga a aparência é o smoke**; o que estes gates
//! defendem é que o kernel é o do `calc_sharpen_filter`, que o gesto é
//! reversível, e que ele **não depende da taxa de eventos**.

use super::stroke_filter::{filter_brush, norm};
use super::*;

/// A malha destes gates: a fixture do pai, que tem detalhe.
fn sharpen_mesh() -> Mesh {
    mesh_for(Verb::Smooth)
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

/// ⭐ **O GATE DA WAVE: o kernel é o `calc_sharpen_filter`.**
///
/// Uma força dentro do teto de uma fatia (`MAX_STEP`) é **UM** passo — ou seja,
/// a referência com um único evento de rato —, e é aí que o porte é comparável
/// termo a termo contra a lei escrita à mão.
#[test]
fn the_sharpen_filter_is_the_reference_law_written_independently() {
    const PER: f32 = 0.4;
    let base = sharpen_mesh();
    let want = reference_sharpen(&base, PER);
    let got = sharpened(PER);

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
    let at = sharpened(hi);
    let over = sharpened(hi * 4.0);
    assert_eq!(
        at.positions(),
        over.positions(),
        "quatro vezes o teto ainda move a malha: o teto está cedo demais"
    );

    let half = sharpened(hi * 0.25);
    let gap = worst_gap(at.positions(), half.positions());
    assert!(
        gap > 1e-4,
        "um quarto do teto já dá o mesmo que o teto ({gap:e}): a faixa é decorativa"
    );
}
