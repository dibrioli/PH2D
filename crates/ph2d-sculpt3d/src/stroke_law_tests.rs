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
fn the_envelope_is_order_free_where_the_footprint_cannot_move() {
    // O `max` é comutativo e todo alvo sai do estado congelado, então a máquina
    // do envelope **não tem histórico**: a mesma lista de dabs em qualquer ordem
    // dá o mesmo resultado, ao bit.
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
    for (k, r) in results.iter().enumerate().skip(1) {
        assert_eq!(r, &results[0], "a ordem {k} deu outro resultado");
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
        let w = Falloff::Smooth.weight(dist / r) * brush.strength;
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

#[test]
fn re_stamping_the_same_dab_list_changes_nothing_where_the_envelope_is_still_the_law() {
    let brush = Brush {
        verb: Verb::Mask,
        radius: 0.3,
        strength: 0.7,
        ..Brush::default()
    };
    assert_eq!(
        brush.verb.grip(),
        crate::Grip::Paint,
        "este gate mede a lei do envelope, e o verbo escolhido saiu dela"
    );
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let once = mesh.masks().expect("a máscara não foi escrita").to_vec();
    assert!(
        once.iter().any(|&m| m > 0.01),
        "a fixture não mascarou nada"
    );
    for _ in 0..12 {
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    assert_eq!(
        once,
        mesh.masks().expect("a máscara sumiu"),
        "o mesmo dab repetido intensificou"
    );
    // E ele não faz TRABALHO: `last_moved` é o que alimenta o refit do octree e
    // o upload incremental, então um empate que "vencesse" mandaria a pegada
    // inteira para a GPU a cada frame sem um pixel mudar.
    assert!(
        stroke.last_moved().is_empty(),
        "o dab repetido re-escreveu {} vértices",
        stroke.last_moved().len()
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
