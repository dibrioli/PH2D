//! **A LEI DE UM CARIMBO** — o que sobrevive a como o motor amostrou o caminho,
//! e o que a família do carimbo trocou em 2026-08-11.
//!
//! Filho do [`super`] pelo mesmo motivo dos irmãos `verb_*`: o pai cruzou o teto
//! de LOC quando os dois gates da troca de lei entraram, e a linha de corte já
//! existia no assunto — aqui mora *o que é verdade sobre um traço inteiro*
//! (invariância de amostragem, ordem, re-stamp, o `pre` congelado), e lá o que
//! um dab faz com máscara, simetria, undo e normais.

use super::*;

#[test]
fn the_stroke_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    // O MESMO caminho, amostrado 8× e 64×. Sob a lei do envelope o resultado
    // converge; sob um produto por-dab ele CRESCE com a taxa de amostragem, e
    // "passar devagar deposita mais" vira uma propriedade do mouse.
    // ⚠️ **Força BAIXA de propósito, e é o que dá dentes ao gate.** Com força
    // alta um acumulador somado satura em `1` nas DUAS densidades e o gate fica
    // verde sobre a doença — foi exatamente o que uma mutação `+=` clampada
    // provou. O regime em que "somar" e "envelopar" divergem é o não-saturado.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.08,
        ..Brush::default()
    };
    let (a, b) = ([0.0, -0.2, 1.0], [0.0, 0.2, 1.0]);

    let mut coarse = sphere();
    let base = snapshot(&coarse);
    sweep(&mut coarse, &brush, a, b, 8);
    let coarse_shift = max_shift(&base, &coarse);

    let mut fine = sphere();
    sweep(&mut fine, &brush, a, b, 64);
    let fine_shift = max_shift(&base, &fine);

    let ratio = fine_shift / coarse_shift;
    assert!(
        (ratio - 1.0).abs() < 0.05,
        "8 dabs deram {coarse_shift:.5} e 64 deram {fine_shift:.5} ({ratio:.3}×) — \
         o traço virou função do ESPAÇAMENTO"
    );
    // E o traço de fato aconteceu: sem isto o gate ficaria verde com um pincel
    // que não move nada (0/0 controlado, o vácuo que deixa razão sadia sobre
    // dois doentes).
    assert!(
        coarse_shift > 1e-3,
        "o pincel não moveu nada: {coarse_shift}"
    );
}

#[test]
fn smoothing_is_a_fact_of_the_path_too_because_the_ring_is_read_frozen() {
    // O irmão do gate acima para os verbos que leem a VIZINHANÇA. Se o Smooth
    // lesse as posições vivas, cada dab suavizaria sobre o que o anterior já
    // suavizou — o produto por-dab entrando pela porta dos fundos, e a
    // superfície derretendo mais quanto mais devagar a mão passa.
    let brush = Brush {
        verb: Verb::Smooth,
        radius: 0.30,
        strength: 0.15,
        ..Brush::default()
    };
    let (a, b) = ([0.0, -0.25, 0.97], [0.0, 0.25, 0.97]);

    let mut coarse = shapes::uv_sphere(24, 36, 1.0);
    let base = snapshot(&coarse);
    sweep(&mut coarse, &brush, a, b, 6);
    let coarse_shift = max_shift(&base, &coarse);

    let mut fine = shapes::uv_sphere(24, 36, 1.0);
    sweep(&mut fine, &brush, a, b, 48);
    let fine_shift = max_shift(&base, &fine);

    assert!(
        coarse_shift > 1e-4,
        "o Smooth não moveu nada: {coarse_shift}"
    );
    let ratio = fine_shift / coarse_shift;
    assert!(
        (ratio - 1.0).abs() < 0.10,
        "6 dabs deram {coarse_shift:.6} e 48 deram {fine_shift:.6} ({ratio:.3}×) — \
         o Smooth virou função do ESPAÇAMENTO"
    );
}

#[test]
fn the_channel_is_order_free_up_to_the_rounding_of_its_own_sum() {
    // A soma é comutativa e todo peso sai do estado CONGELADO, então a máquina
    // do canal **não tem histórico**: a mesma lista de dabs em qualquer ordem dá
    // o mesmo resultado.
    //
    // ⚠️ **"Ao bit" era verdade sob o ENVELOPE e deixou de ser** (2026-08-12):
    // o `max` é exato em qualquer ordem, a SOMA em `f32` não é associativa. O
    // que se perdeu é uma arredondada por dab; o que se mantém — e é o que este
    // gate existe para pegar — é que a ordem não muda a LEI. Uma dependência
    // real (ler o estado vivo, por exemplo) não chega perto desta barra: ela
    // move o canal em fração, não em ULP.
    //
    // ⚠️ **Medido no verbo de MÁSCARA, e a escolha é a coisa importante deste
    // gate.** Os verbos de geometria não podem prometer isto e não é defeito da
    // lei: a PEGADA é consultada nas posições VIVAS — o pincel age onde a
    // superfície está agora, que é o que o artista vê e o que Blender e SculptGL
    // fazem — então mover a superfície muda quem cai sob o dab seguinte. O
    // acoplamento entra pela CONSULTA, nunca pelo acumulador. O Mask não move
    // geometria, logo ali a pegada é fixa e a afirmação vira exata.
    let path = [
        [0.00, -0.30, 0.95],
        [0.00, -0.10, 1.00],
        [0.00, 0.10, 1.00],
        [0.00, 0.30, 0.95],
        [0.20, 0.00, 0.97],
    ];
    // Pesos deliberadamente DESIGUAIS: com todos iguais o desempate por "o
    // primeiro vence" tornaria a ordem observável mesmo na lei correta, e o gate
    // estaria afirmando algo falso.
    let radii = [0.34f32, 0.22, 0.30, 0.26, 0.38];
    let orders: [[usize; 5]; 3] = [[0, 1, 2, 3, 4], [4, 3, 2, 1, 0], [2, 0, 4, 1, 3]];

    let mut results = Vec::new();
    for order in orders {
        let mut mesh = sphere();
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        for i in order {
            st.dab(
                &mut mesh,
                &Brush {
                    verb: Verb::Mask,
                    radius: radii[i],
                    strength: 0.9,
                    ..Brush::default()
                },
                &dab_at(path[i], radii[i]),
                Symmetry::default(),
            );
        }
        results.push(mesh.masks().expect("o canal foi pintado").to_vec());
    }
    // ⚠️ **A BARRA É DERIVADA DA FIXTURE, não escolhida.** O canal vive em
    // `[0, 1]`, então UMA arredondada de `f32` custa no máximo
    // `f32::EPSILON`, e uma lista de `N` dabs arredonda no máximo `N` vezes.
    // Medido: a ordem invertida sai **EXATA** (`0,0`) e a embaralhada a **um
    // ULP** (`5,960e-8`) — a soma reordenada, e nada mais. Uma barra escolhida
    // à mão aqui seria um número que não sabe dizer quando está errado.
    let bar = path.len() as f32 * f32::EPSILON;
    for (k, r) in results.iter().enumerate().skip(1) {
        let gap = r
            .iter()
            .zip(&results[0])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            gap <= bar,
            "a ordem {k} deu outro resultado: {gap:.3e} > {bar:.3e} — isso é \
             LEI, não arredondada"
        );
    }
    // E o traço fez algo: sem isto três máscaras vazias seriam "iguais".
    assert!(results[0].iter().copied().fold(0.0f32, f32::max) > 0.5);
}

/// **O Smooth lê a vizinhança VIVA** — e este gate afirmava o contrário.
///
/// ⚠️ **Ele nasceu certo e a lei o inverteu** (2026-08-11): sob o envelope o
/// alvo tinha de ser função do estado congelado, senão dois dabs no mesmo lugar
/// davam pesos diferentes e a idempotência caía. A família do carimbo compõe
/// agora, e o `Smooth.js` da referência lê `vAr` — a posição viva. Ler o anel
/// congelado faria o segundo dab suavizar em direção a uma superfície que já
/// não existe, e o verbo pararia de convergir.
///
/// ⚠️ **O oráculo continua ANALÍTICO, e é por isso que ele alcança a
/// propriedade:** a diferença entre *vivo* e *congelado* só aparece num vértice
/// cujo anel um dab ANTERIOR já mexeu, e nenhuma varredura de comportamento
/// garante que a fixture contenha esse vértice. Aqui a resposta certa é
/// CALCULADA a partir do estado intermediário e comparada.
///
/// ⚠️ **A distância continua saindo do `pre`** (`from_live` só é verdadeiro com
/// o Accumulate armado), então o PESO é frozen e a POSIÇÃO é viva — as duas
/// metades entram no oráculo pelo lado que lhes cabe.
#[test]
fn the_smooth_target_is_the_live_neighbourhood_not_the_frozen_one() {
    let r = 0.30f32;
    let (c1, c2) = ([0.0, -0.12, 0.99], [0.0, 0.05, 1.0]);
    let brush = Brush {
        verb: Verb::Smooth,
        radius: r,
        strength: 1.0,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let mut st = SculptStroke::default();
    st.begin(&mesh);
    st.dab(&mut mesh, &brush, &dab_at(c1, r), Symmetry::default());
    // `BTreeSet` e não `HashSet`: a lint estrutural do repo, e aqui ela também
    // torna a varredura do gate reproduzível na mesma ordem.
    let moved_by_first: std::collections::BTreeSet<u32> = st.last_moved().iter().copied().collect();
    assert!(!moved_by_first.is_empty(), "o 1º dab não moveu nada");
    // O estado que o SEGUNDO dab de fato encontra.
    let mid = snapshot(&mesh);
    st.dab(&mut mesh, &brush, &dab_at(c2, r), Symmetry::default());

    // Um vértice que o SEGUNDO dab moveu e cujo anel o PRIMEIRO já tinha
    // mexido: é o único lugar onde "congelado" e "vivo" dão respostas
    // diferentes, e a fixture tem de conter esse vértice ou o gate é vácuo.
    let mut checked = 0;
    let mut separated = 0;
    for &v in st.last_moved() {
        let ring = mesh.adjacency().vert_verts.neighbours(v as usize);
        if !ring.iter().any(|n| moved_by_first.contains(n)) {
            continue;
        }
        let bv = base[v as usize];
        let lv = mid[v as usize];
        let mut avg_live = [0.0f32; 3];
        let mut avg_frozen = [0.0f32; 3];
        for &n in ring {
            for k in 0..3 {
                avg_live[k] += mid[n as usize][k];
                avg_frozen[k] += base[n as usize][k];
            }
        }
        let inv = 1.0 / ring.len() as f32;
        // A distância sai do `pre`: o peso é frozen mesmo com a posição viva.
        let d = [bv[0] - c2[0], bv[1] - c2[1], bv[2] - c2[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        // ⚠️ **A curva sai do PINCEL sob teste, nunca de um literal.** Ela dizia
        // `Falloff::Smooth` — o default de fábrica no dia em que o gate nasceu —
        // e a wave dos modos (plano 21 W0) mudou esse default para o que a
        // REFERÊNCIA declara: o oráculo passou a discordar do produto por um
        // número que ele não controla. O que este gate afirma é o anel VIVO
        // contra o CONGELADO, e isso é verdade em qualquer curva.
        let w = brush.falloff.weight(dist / r) * brush.strength;
        for k in 0..3 {
            let want = lv[k] + (avg_live[k] * inv - lv[k]) * w;
            let stale = bv[k] + (avg_frozen[k] * inv - bv[k]) * w;
            let got = mesh.positions()[v as usize][k];
            assert!(
                (want - got).abs() < 1e-5,
                "vértice {v} eixo {k}: previsto {want} pelo anel VIVO, obtido \
                 {got} (o anel congelado teria dado {stale})"
            );
            // ⚠️ **A metade que impede o gate de ser verde por coincidência:**
            // onde as duas leis dão o MESMO número ele não distingue nada, e
            // a fixture tem de conter vértices em que elas se SEPARAM.
            if (want - stale).abs() > 1e-5 {
                separated += 1;
            }
        }
        checked += 1;
    }
    assert!(
        checked > 10,
        "só {checked} vértices continham o fenômeno — a fixture é fraca"
    );
    assert!(
        separated > 10,
        "as duas leis deram o mesmo número em toda parte ({separated} \
         separações): a fixture não distingue vivo de congelado"
    );
}

/// **ESFREGAR CONSTRÓI, e depois PARA** — as duas metades da lei aditiva do
/// canal, e o gate que este substitui afirmava a primeira ao contrário.
///
/// ⚠️ **Ele chamava-se `..._where_the_envelope_is_still_the_law` e media
/// exatamente o defeito que o Enio reportou** (2026-08-12): sob o envelope,
/// repetir o MESMO dab doze vezes deixava o canal onde um dab o deixara, e o
/// gate exigia essa igualdade. Um `max` sobre dabs idênticos no mesmo lugar é
/// constante *por construção* — a máscara não acumulava, e o gate pinava isso
/// como se fosse a entrega.
///
/// ⚠️ **O que se PERDE com a troca está aqui em vez de na prosa:** re-carimbar
/// a mesma lista de dabs deixou de ser no-op, exatamente como na família do
/// carimbo (o gate irmão logo abaixo). Nenhuma rota deste módulo re-carimba um
/// traço, e o `base_mask` congelado mantém o undo trivial.
///
/// ⚠️ **E o TRABALHO continua limitado**, que era a segunda metade do gate
/// antigo: um vértice saturado não recebe mais nada, então o early-out
/// devolve-o à quietude e a pegada para de ir ao refit do octree e ao upload —
/// só que agora por SATURAÇÃO, não por empate.
#[test]
fn rubbing_the_channel_builds_it_and_then_stops() {
    let brush = Brush {
        verb: Verb::Mask,
        radius: 0.3,
        strength: 0.2,
        ..Brush::default()
    };
    assert_eq!(
        brush.verb.grip(),
        crate::Grip::Paint,
        "este gate mede a lei ADITIVA do canal, e o verbo escolhido saiu dela"
    );
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    let peak = |m: &Mesh| {
        m.masks()
            .expect("a máscara não foi escrita")
            .iter()
            .copied()
            .fold(0.0f32, f32::max)
    };

    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let once = peak(&mesh);
    assert!(
        (once - 0.2).abs() < 1e-5,
        "um dab a 0,2 tem de pintar 0,2 no miolo, e pintou {once}"
    );

    // ESFREGAR CONSTRÓI — a metade que o gate antigo negava.
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let twice = peak(&mesh);
    assert!(
        (twice - 0.4).abs() < 1e-5,
        "duas esfregadas a 0,2 têm de dar 0,4, e deram {twice}"
    );

    // E CHEGA AO TETO — o que a lei do envelope tornava inalcançável por
    // qualquer número de esfregadas.
    for _ in 0..8 {
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    let full = peak(&mesh);
    assert!(
        full >= 1.0,
        "dez esfregadas a 0,2 têm de saturar, e pararam em {full}"
    );

    // E DEPOIS PARA: o miolo saturado não é mais re-escrito.
    let saturated = mesh.masks().expect("a máscara sumiu").to_vec();
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let centre = saturated
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).expect("sem NaN no canal"))
        .expect("o canal não está vazio")
        .0 as u32;
    assert!(
        !stroke.last_moved().contains(&centre),
        "o vértice saturado voltou ao upload sem ter o que receber"
    );
}

/// **E a família do CARIMBO faz o oposto, de propósito.**
///
/// ⚠️ **Este é o preço da lei que compõe, escrito como asserção em vez de
/// prosa.** Uma pincelada que passa duas vezes no mesmo lugar deposita duas
/// vezes — é o que uma pincelada FAZ, e é a estrutura do kernel da referência
/// (`vAr[ind] = vx + anx * fallOff`). O que se perde é a idempotência sob
/// re-stamp, e **nada neste módulo a consome**: nenhuma rota re-carimba um
/// traço, e o `base` congelado mantém o undo trivial.
///
/// ⚠️ **Ele existe para a troca custar um vermelho nos DOIS sentidos.** Sem
/// ele, voltar o [`crate::Grip::Stamp`] ao envelope deixaria o gate irmão
/// VERDE — e a paridade com a referência morreria em silêncio, que é
/// exatamente como ela viveu quebrada até aqui.
#[test]
fn re_stamping_the_stamp_family_compounds_and_that_is_the_price_of_the_law() {
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 0.7,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let base = sphere();
    let once = max_shift(&snapshot(&base), &mesh);
    for _ in 0..11 {
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    let twelve = max_shift(&snapshot(&base), &mesh);
    assert!(
        twelve > once * 5.0,
        "doze dabs no mesmo lugar depositaram {twelve:.6} contra {once:.6} de \
         um — o carimbo voltou a ser um envelope"
    );
}

#[test]
fn a_new_stroke_forgets_the_previous_envelope_and_builds_on_top() {
    // O outro lado da idempotência: soltar e desenhar de novo TEM de somar,
    // senão a ferramenta fica presa num teto que o artista não pediu.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 0.7,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    let mut stroke = SculptStroke::default();

    stroke.begin(&mesh);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let after_one = max_shift(&base, &mesh);

    stroke.begin(&mesh);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let after_two = max_shift(&base, &mesh);

    assert!(
        after_two > after_one * 1.8,
        "dois traços deram {after_two:.4} contra {after_one:.4} de um só"
    );
}

/// A **rugosidade** do conjunto `of`: a magnitude do laplaciano em cada vértice,
/// no pior caso.
///
/// ⚠️ **É exatamente a grandeza que um Smooth MINIMIZA**, e é também a que o
/// matcap desenha: um vértice longe da média dos vizinhos é uma quina, e uma
/// quina é o que a foto do report mostra. Medir "as duas malhas diferem" seria
/// satisfeito por um passe que fizesse qualquer coisa; medir a rugosidade
/// afirma a DIREÇÃO.
fn ring_roughness(mesh: &Mesh, of: &[usize]) -> f32 {
    let pos = mesh.positions();
    let ring = &mesh.adjacency().vert_verts;
    let mut worst = 0.0f32;
    for &v in of {
        let ns = ring.neighbours(v);
        if ns.is_empty() {
            continue;
        }
        let mut avg = [0.0f32; 3];
        for &n in ns {
            let p = pos[n as usize];
            avg[0] += p[0];
            avg[1] += p[1];
            avg[2] += p[2];
        }
        let k = ns.len() as f32;
        let p = pos[v];
        let d = [p[0] - avg[0] / k, p[1] - avg[1] / k, p[2] - avg[2] / k];
        worst = worst.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    worst
}

#[test]
fn the_second_pass_runs_inside_the_stroke_and_it_is_the_rim_that_it_rounds() {
    // ⚠️ **Este gate existe porque uma MUTAÇÃO SOBREVIVEU.** Trocar
    // `brush.auto_smooth_brush()` por `None::<Brush>` dentro do laço de simetria
    // deixou a suíte inteira VERDE (280 de 280): os três gates que a wave tinha
    // escrito perguntam à PORTA — *ela devolve `None` no neutro? pula os dois
    // verbos que a referência pula? troca o verbo e a força e mais nada?* — e
    // **nenhum exercitava a fiação dela**. Um pincel pode responder as três
    // corretamente e ninguém chamá-lo. É a lição do repo na forma mais barata de
    // a cometer: *sobrevivente = gate faltando*.
    //
    // ⚠️ **A falloff é `Constant` de propósito, e é o regime onde o passe tem o
    // que fazer.** Ela deposita um PLATÔ — peso 1 dentro do raio, 0 fora —, logo
    // um degrau discreto na borda; sob `Smooth` o depósito já chega afinado e o
    // segundo passe mede quase nada (medido na sonda: `Plateau` praticamente
    // inalterada, `Constant` p99 do diedro **79,22° → 34,05°**). Uma fixture com
    // a curva macia deixaria este gate verde sobre um passe inerte.
    //
    // ⚠️ **E o ponto de operação é MEDIDO, não escolhido** — o mesmo traço
    // (Draw, raio 0,30, força 0,5, `Constant`, 12 dabs) varrido no knob:
    //
    // | `auto_smooth` | crista | rugosidade |
    // |---|---|---|
    // | 0,00 | 0,11853 | 0,029113 |
    // | 0,05 | 0,11568 | 0,026023 |
    // | 0,10 | 0,11248 | 0,024232 |
    // | **0,25** | **0,10158** | **0,021050** |
    // | 0,50 | 0,08186 | 0,016140 |
    // | 1,00 | 0,04286 | 0,014097 |
    //
    // Em `0,25` o verbo sobrevive (**86%** do depósito) e o alisamento é
    // inequívoco (**−28%** de rugosidade) — é a faixa em que a referência é de
    // facto usada. As duas barras abaixo saem desta tabela.
    //
    // ⚠️ **Num TRAÇO o segundo passe custa depósito, e num toque único não** —
    // a sonda de um carimbo mede a crista indo de 0,1709 a 0,1692 (1%) com
    // `auto_smooth` em 1,0, e aqui o mesmo 1,0 come 64%. O mecanismo é a
    // sobreposição: o alisamento roda **uma vez por dab** (é a posição da
    // referência, `sculpt.cc:3635`) e os dabs de um traço caem uns sobre os
    // outros, então ele compõe sobre os MESMOS vértices enquanto o depósito
    // envelopa. É por isto que o default da referência é **zero** e que a faixa
    // de trabalho dela é 0,1–0,3, e não um defeito do porte.
    const AUTO_SMOOTH: f32 = 0.25;
    let base_brush = crate::Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.5,
        falloff: crate::Falloff::Constant,
        ..crate::Brush::default()
    };
    let (a, b) = ([0.0, -0.2, 1.0], [0.0, 0.2, 1.0]);

    let mut off = sphere();
    let base = snapshot(&off);
    sweep(&mut off, &base_brush, a, b, 12);

    let mut on = sphere();
    sweep(
        &mut on,
        &crate::Brush {
            auto_smooth: AUTO_SMOOTH,
            ..base_brush.clone()
        },
        a,
        b,
        12,
    );

    // O conjunto medido é o MESMO nos dois lados — quem o define é o traço SEM
    // o passe, senão cada coluna mediria uma vizinhança diferente.
    let touched: Vec<usize> = base
        .iter()
        .zip(off.positions())
        .enumerate()
        .filter(|(_, (p0, p1))| {
            let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 1e-10
        })
        .map(|(i, _)| i)
        .collect();

    // CONTROLE 1 — o traço aconteceu. Sem isto, *"mais liso"* é satisfeito por
    // um pincel que não moveu nada: a esfera de fábrica já é lisa.
    let shift_off = max_shift(&base, &off);
    assert!(
        shift_off > 1e-2 && touched.len() > 20,
        "o traço não aconteceu: {shift_off:.5} em {} vértices",
        touched.len()
    );

    // CONTROLE 2 — o passe ALISA, ele não APAGA. Um segundo passe que devolvesse
    // a esfera original seria "perfeitamente liso" e teria destruído o verbo. A
    // barra sai da tabela acima (0,857 medido), não de gosto.
    let shift_on = max_shift(&base, &on);
    assert!(
        shift_on > shift_off * 0.80,
        "o segundo passe comeu o traço: {shift_on:.5} contra {shift_off:.5}"
    );

    let rough_off = ring_roughness(&off, &touched);
    let rough_on = ring_roughness(&on, &touched);
    assert!(
        rough_on < rough_off * 0.85,
        "o segundo passe não alisou nada: rugosidade {rough_on:.6} com ele \
         contra {rough_off:.6} sem — se a razão for 1,000 exata, ele NÃO ESTÁ \
         SENDO CHAMADO (a mutação que este gate existe para matar)"
    );
}

#[test]
fn the_second_pass_reaches_every_mirrored_copy_not_just_the_last_one() {
    // O irmão do gate acima para a POSIÇÃO do passe, e ele existe porque aquele
    // não a enxerga: com `Symmetry::default()` o laço de espelho dá uma volta só,
    // e *dentro* e *fora* dele são a mesma coisa. A afirmação do doc — que a
    // referência o chama no fim do `do_brush_action`, **uma vez por cópia**
    // (`sculpt.cc:3635`) — só é observável quando há mais de uma.
    //
    // ⚠️ **O oráculo é a APARÊNCIA das duas metades, não um pareamento de
    // índices.** Se o passe rodasse fora do laço, ele alisaria a última cópia e
    // deixaria a primeira crua — uma metade lisa e a outra facetada, que é o que
    // o artista veria. Comparar as duas rugosidades pergunta isso direto.
    let brush = crate::Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.5,
        falloff: crate::Falloff::Constant,
        auto_smooth: 0.25,
        ..crate::Brush::default()
    };
    // FORA do plano do espelho, senão as duas cópias caem uma sobre a outra e
    // não há duas metades para comparar.
    let (a, b) = ([0.55, -0.15, 0.82], [0.55, 0.15, 0.82]);

    let mut mesh = sphere();
    let base = snapshot(&mesh);
    sweep_sym(&mut mesh, &brush, a, b, 12, crate::Symmetry::MIRROR_X);

    let (mut left, mut right) = (Vec::new(), Vec::new());
    for (i, (p0, p1)) in base.iter().zip(mesh.positions()).enumerate() {
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 1e-10 {
            if p0[0] < 0.0 { &mut left } else { &mut right }.push(i);
        }
    }
    // CONTROLE — as DUAS metades foram tocadas. Sem isto o gate ficaria verde
    // sobre um espelho que não expandiu nada (0 contra 0, e `0 < 0 * k` falso,
    // mas a razão de duas rugosidades vazias é `0/0`).
    assert!(
        left.len() > 20 && right.len() > 20,
        "o espelho não produziu duas metades: {} e {}",
        left.len(),
        right.len()
    );

    let (rl, rr) = (ring_roughness(&mesh, &left), ring_roughness(&mesh, &right));
    let ratio = if rl > rr { rl / rr } else { rr / rl };
    assert!(
        ratio < 1.10,
        "as duas metades não receberam o mesmo tratamento: rugosidade \
         {rl:.6} × {rr:.6} ({ratio:.3}×) — um passe que roda FORA do laço de \
         simetria alisa só a última cópia"
    );
}
