//! Gates do **BLOB** — o Crease com o aperto invertido e o depósito para cima.
//!
//! Irmão do `verb_crease_tests.rs` pelo mesmo corte: o assunto é UM verbo, as
//! fixtures são caras (`uv_sphere(128,192)` para resolver o perfil radial) e
//! nenhum outro verbo precisa delas.
//!
//! ⚠️ **O gate que carrega o arquivo é o `..._is_not_the_inverted_crease`.** Um
//! Blob implementado como *"o Crease com o sinal do `sign` trocado"* compila,
//! sobe barro, parece certo numa screenshot e **é um verbo morto** — o `Ctrl` no
//! Crease já entrega aquilo. É o RADIAL que os separa, e é por isso que toda
//! medição deste arquivo é sobre a distância ao EIXO do dab, não sobre altura.

use super::*;
use crate::RefMode;

/// Quanto barro sai (ou entra) do eixo do dab, e quanto ele sobe (ou desce).
///
/// Devolve `(radial, vertical)`: o `radial` é a soma do deslocamento em relação
/// ao eixo `+Z` (positivo = o barro se afastou do eixo) e o `vertical` é o
/// deslocamento MÁXIMO ao longo de `+Z` com sinal (positivo = subiu).
///
/// ⚠️ **O `radial` é SOMA e o `vertical` é MÁXIMO, e a assimetria é do que cada
/// um descreve:** empurrar para fora é um efeito de REGIÃO (todo o anel anda um
/// pouco, e um único vértice não representa), enquanto o depósito é um PICO — o
/// `shape⁴` o concentra no meio, e uma soma sobre a pegada o dissolveria na
/// borda, que é justamente onde o expoente o zera.
fn profile(verb: Verb, invert: bool, pinch: f32) -> (f32, f32) {
    let radius = 0.4f32;
    let mut mesh = ph2d_mesh::shapes::uv_sphere(128, 192, 1.0);
    let base = snapshot(&mesh);
    let b = Brush {
        verb,
        radius,
        strength: 1.0,
        invert,
        pinch,
        falloff: Falloff::Smooth,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(
        &mut mesh,
        &b,
        &dab_at([0.0, 0.0, 1.0], radius),
        Symmetry::default(),
    );

    let axis_r = |p: &[f32; 3]| (p[0] * p[0] + p[1] * p[1]).sqrt();
    let mut radial = 0.0f32;
    let mut vertical = 0.0f32;
    for (i, p) in mesh.positions().iter().enumerate() {
        radial += axis_r(p) - axis_r(&base[i]);
        let dz = p[2] - base[i][2];
        if dz.abs() > vertical.abs() {
            vertical = dz;
        }
    }
    (radial, vertical)
}

/// O Blob EMPURRA o barro para fora do eixo; o Crease o PUXA para dentro.
///
/// É o `invert_strength` do `do_crease_or_blob_brush` medido no barro: lá o
/// termo lateral é `(centro − posição) · força`, e a força do Blob é a do Crease
/// com o sinal trocado.
///
/// ⚠️ **A fixture usa `pinch: 1.0` de propósito** — o termo lateral é escalado
/// pelo knob, e num `pinch` baixo os dois verbos convergem para o mesmo
/// depósito. Um gate que não declara a premissa mediria dois números quase
/// iguais e passaria a barra por sorte.
#[test]
fn the_blob_pushes_out_where_the_crease_pulls_in() {
    let (blob_r, _) = profile(Verb::Blob, false, 1.0);
    let (crease_r, _) = profile(Verb::Crease, false, 1.0);
    println!("radial: blob {blob_r:+.6} · crease {crease_r:+.6}");
    assert!(
        blob_r > 0.0,
        "o Blob tinha de afastar o barro do eixo e a soma radial deu {blob_r:+.6}"
    );
    assert!(
        crease_r < 0.0,
        "o Crease tinha de aproximar o barro do eixo e a soma radial deu {crease_r:+.6}"
    );
}

/// O Blob SOBE onde o Crease CAVA.
///
/// ⚠️ **Esta metade sozinha NÃO justifica um verbo** — o `Ctrl` no Crease
/// entrega um depósito para cima. Ela existe para que a direção que a §4 nos
/// obrigou a escolher ([`Verb::Blob`]) fique pinada: *um blob é um monte*, e o
/// dia em que alguém a inverter, isto sangra.
#[test]
fn the_blob_raises_where_the_crease_digs() {
    let (_, blob_v) = profile(Verb::Blob, false, 1.0);
    let (_, crease_v) = profile(Verb::Crease, false, 1.0);
    println!("vertical: blob {blob_v:+.6} · crease {crease_v:+.6}");
    assert!(
        blob_v > 0.0,
        "o Blob tinha de subir e o deslocamento máximo deu {blob_v:+.6}"
    );
    assert!(
        crease_v < 0.0,
        "o Crease tinha de cavar e o deslocamento máximo deu {crease_v:+.6}"
    );
}

/// **O Blob NÃO é o Crease invertido** — é o gate que o torna um verbo.
///
/// Os dois SOBEM (`Ctrl` no Crease troca o depósito), então a altura não os
/// distingue e um gate que só a medisse ficaria verde sobre um alias. O que os
/// separa é o lado para onde o barro sai do eixo, e nenhum ajuste do Crease
/// alcança o do Blob: o `abs` da referência (`std::abs(cache.bstrength)`) existe
/// exatamente para o `Ctrl` **não** virar o aperto.
#[test]
fn the_blob_is_not_the_inverted_crease() {
    let (blob_r, blob_v) = profile(Verb::Blob, false, 1.0);
    let (inv_r, inv_v) = profile(Verb::Crease, true, 1.0);
    println!("blob {blob_r:+.6}/{blob_v:+.6} · crease invertido {inv_r:+.6}/{inv_v:+.6}");
    // O CONTROLE: os dois sobem, logo a altura não é oráculo aqui.
    assert!(
        blob_v > 0.0 && inv_v > 0.0,
        "a premissa deste gate é que os DOIS sobem — blob {blob_v:+.6}, \
         crease invertido {inv_v:+.6}"
    );
    assert!(
        blob_r > 0.0 && inv_r < 0.0,
        "o Blob e o Crease invertido têm de sair para lados OPOSTOS do eixo — \
         blob {blob_r:+.6}, crease invertido {inv_r:+.6}; se os dois têm o mesmo \
         sinal, o Blob é um alias e o chip dele é morto"
    );
}

/// **Sem aperto, o Blob É o Crease invertido** — o controle do gate acima.
///
/// Ele nomeia exatamente o que o termo lateral contribui: zerado o `pinch`,
/// sobra o depósito, e aí os dois verbos *são* a mesma aritmética. Sem esta
/// linha, o gate anterior poderia estar a medir uma diferença que vem de
/// qualquer outro lugar do alvo.
#[test]
fn with_no_pinch_the_blob_is_the_inverted_crease() {
    let (blob_r, blob_v) = profile(Verb::Blob, false, 0.0);
    let (inv_r, inv_v) = profile(Verb::Crease, true, 0.0);
    println!("sem pinch: blob {blob_r:+.6}/{blob_v:+.6} · crease inv {inv_r:+.6}/{inv_v:+.6}");
    assert!(
        (blob_v - inv_v).abs() < 1e-6,
        "sem aperto o depósito tinha de ser o MESMO: {blob_v:+.8} vs {inv_v:+.8}"
    );
    assert!(
        (blob_r - inv_r).abs() < 1e-4,
        "sem aperto não há termo lateral, então o radial tinha de coincidir: \
         {blob_r:+.8} vs {inv_r:+.8}"
    );
}

/// O domo do Blob é mais ESTREITO que o de um Draw de mesmo alcance.
///
/// É o `shape⁴` medido, e o **Draw é o controle** — mesma curva de falloff,
/// mesma lei de acúmulo, o mesmo dab: a única coisa que pode separá-los é o
/// expoente sobre o coeficiente da normal. Espelho exato do
/// [`the_crease_cuts_a_narrower_groove_than_an_inverted_draw_of_the_same_reach`],
/// e pela mesma razão: um pico maior não é um domo estreito.
///
/// ⚠️ **`pinch: 0.0` é PREMISSA, não conveniência.** Com aperto o termo lateral
/// contribui para o `z` de todo vértice fora do eixo, e a medição passaria a
/// misturar os dois termos — o gate mediria a soma e chamaria ao resultado *o
/// expoente*.
///
/// ⚠️ **E o primeiro oráculo deste gate estava ERRADO, com o gate VERDE.** Ele
/// comparava o domo com o EMPURRÃO do próprio Blob (*"o domo é mais estreito que
/// o empurrão"*) — e o termo lateral é `centro − posição`, que vale **zero no
/// eixo**: o empurrão é um **ANEL**, com o pico fora do centro. Um domo é mais
/// estreito que um anel para QUALQUER expoente, então a mutação `shape⁴ → shape`
/// passava (razão 2,911× → 1,881×, contra uma barra de 1,5). *Duas grandezas de
/// FORMA diferente não se comparam por meia-largura.*
#[test]
fn the_blobs_dome_is_narrower_than_a_draws_of_the_same_reach() {
    let radius = 0.4f32;
    let profile = |verb: Verb| -> (f32, f32) {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(256, 512, 1.0);
        let base = snapshot(&mesh);
        let b = Brush {
            verb,
            radius,
            strength: 1.0,
            pinch: 0.0,
            falloff: Falloff::Smooth,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], radius),
            Symmetry::default(),
        );
        let mut samples: Vec<(f32, f32)> = Vec::new();
        for (i, p) in mesh.positions().iter().enumerate() {
            let lift = p[2] - base[i][2];
            if lift > 1e-6 {
                samples.push(((p[0] * p[0] + p[1] * p[1]).sqrt(), lift));
            }
        }
        let peak = samples.iter().map(|s| s.1).fold(0.0f32, f32::max);
        let half = peak * 0.5;
        let width = samples
            .iter()
            .filter(|s| s.1 >= half)
            .map(|s| s.0)
            .fold(0.0f32, f32::max);
        (peak, width / radius)
    };

    let (pb, wb) = profile(Verb::Blob);
    let (pd, wd) = profile(Verb::Draw);
    println!(
        "blob pico {pb:.5} largura {wb:.3} R · draw pico {pd:.5} largura {wd:.3} R · \
         estreitamento {:.3}x · altura {:.3}x",
        wd / wb,
        pb / pd
    );
    assert!(
        wb > 0.0 && wd > 0.0,
        "a fixture não resolve a largura — anéis de menos"
    );
    assert!(
        wd > wb * 1.5,
        "o domo do Blob ({wb:.3} R) tinha de ser bem mais estreito que o do Draw \
         ({wd:.3} R) — é o `shape⁴` do termo normal"
    );
    // A ALTURA é a mesma razão de constantes que o Crease carrega, pelo mesmo
    // motivo: os dois herdam o `CREASE_FRACTION` contra o `REACH_FRACTION`.
    let want = crate::CREASE_FRACTION / crate::REACH_FRACTION;
    let got = pb / pd;
    assert!(
        (got / want - 1.0).abs() < 0.05,
        "a altura do Blob é {got:.3}x a do Draw, e as constantes pedem {want:.3}x"
    );
}

/// O SculptGL não declara o Blob, então ele não o governa.
///
/// Irmão exato do gate da faixa: o `Crease.js` é o parente mais próximo que
/// aquela referência tem, e ele não carrega o `invert_strength` que faz de um
/// Blob um Blob. A lei que governa é a do `B`, que TEM a ferramenta.
#[test]
fn sculptgl_does_not_declare_the_blob_so_it_does_not_govern_it() {
    assert!(
        !RefMode::S.declares(Verb::Blob),
        "o SculptGL não tem Blob — o chip `S` dele seria um nome próprio sobre \
         uma tabela que a fonte nunca escreveu"
    );
    assert_eq!(
        RefMode::S.kernel_for(Verb::Blob),
        RefMode::B.kernel(),
        "um verbo que o modo não declara cai na lei da referência que o TEM"
    );
    assert!(
        RefMode::offered_for(Verb::Blob).all(|m| m != RefMode::S),
        "o painel não pode oferecer o chip `S` para o Blob"
    );
}
