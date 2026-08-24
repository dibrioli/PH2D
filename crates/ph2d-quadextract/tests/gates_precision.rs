//! ⭐⭐⭐ **GATE 1 — a TRUNCAGEM DE PRECISÃO acontece, e ela é necessária** — mais o
//! **GATE 3**, o ponto fixo de uma singularidade.
//!
//! ⛔⛔ **O §2.3 é o passo que ninguém adivinha, e a sua necessidade não é uma
//! opinião: é aritmética.** As coordenadas do mesmo vértice em cartas diferentes têm
//! expoentes diferentes; aplicar `x ↦ R(r)·x + t` com `t` grande e `x` pequeno
//! **perde os bits baixos**, e ao dar a volta ao leque não se regressa ao valor de
//! partida. Um ponto de grade cai então na fenda numérica entre dois triângulos — ou
//! **nos dois**.
//!
//! ⚠️ **Um gate que só afirmasse «o resultado está certo» não distinguiria isto de
//! sorte.** Os dois testes abaixo têm a forma de um controlo positivo: primeiro
//! mostram que o fenómeno **existe** nos dados reais, e só depois cobram que a
//! extracção o tenha removido.

mod support;

use ph2d_quadextract::exact::Xf;
use ph2d_quadextract::extract;
use ph2d_quadextract::mapa::Mapa;

/// A rotação de um quarto de volta em `f64` — troca e negação, exacta.
fn rot(r: u8, p: [f64; 2]) -> [f64; 2] {
    match r & 3 {
        0 => p,
        1 => [-p[1], p[0]],
        2 => [-p[0], -p[1]],
        _ => [p[1], -p[0]],
    }
}

/// A transição de uma aresta partilhada, lida das DUAS imagens cruas dela.
fn raw_transition(a1: [f64; 2], b1: [f64; 2], a2: [f64; 2], b2: [f64; 2]) -> (u8, [f64; 2]) {
    let d1 = [b1[0] - a1[0], b1[1] - a1[1]];
    let d2 = [b2[0] - a2[0], b2[1] - a2[1]];
    let den = d1[0].mul_add(d1[0], d1[1] * d1[1]);
    let re = d2[0].mul_add(d1[0], d2[1] * d1[1]) / den;
    let im = d2[1].mul_add(d1[0], -(d2[0] * d1[1])) / den;
    let k = (im.atan2(re) / core::f64::consts::FRAC_PI_2).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = k.rem_euclid(4.0) as u8;
    let q = rot(r, a1);
    (r, [(a2[0] - q[0]).round(), (a2[1] - q[1]).round()])
}

/// Quantas arestas interiores **não** satisfazem a lei ao bit nas coordenadas cruas.
fn inexact_on_raw(m: &Mapa) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut side: BTreeMap<(u32, u32), Vec<(usize, usize)>> = BTreeMap::new();
    for (f, t) in m.tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            side.entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push((f, k));
        }
    }
    let (mut bad, mut total) = (0usize, 0usize);
    for (_, l) in side {
        if l.len() != 2 {
            continue;
        }
        let ((f, k), (g, j)) = (l[0], l[1]);
        let (a1, b1) = (m.uv[f][k], m.uv[f][(k + 1) % 3]);
        let (a2, b2) = (m.uv[g][(j + 1) % 3], m.uv[g][j]);
        if a1 == b1 || a2 == b2 {
            continue;
        }
        total += 1;
        let (r, t) = raw_transition(a1, b1, a2, b2);
        let ap = rot(r, a1);
        let bp = rot(r, b1);
        if [ap[0] + t[0], ap[1] + t[1]] != a2 || [bp[0] + t[0], bp[1] + t[1]] != b2 {
            bad += 1;
        }
    }
    (bad, total)
}

#[test]
fn a_lei_nao_vale_ao_bit_nas_coordenadas_cruas_e_vale_depois_do_saneamento() {
    for (name, m) in [("gancho", support::hooked()), ("toro", support::torus())] {
        // ── O CONTROLO POSITIVO: o fenómeno existe nestes dados.
        //
        // ⚠️ **E ele é MINORITÁRIO, o que é uma medição e não uma decepção.** Medido
        // em 2026-08-24: `147` de `10 151` arestas do gancho e `85` de `6 143` do
        // toro (~1,4 %). ⛔ A primeira redacção deste gate exigia a maioria e ficou
        // vermelha — a barra era um palpite meu, não um facto do mapa. *Uma aresta
        // basta para pôr um ponto de grade na fenda entre dois triângulos.*
        let (bad, total) = inexact_on_raw(&m);
        assert!(total > 5000, "{name}: {total} arestas interiores");
        assert!(
            bad >= 20,
            "{name}: o mapa CRU falhou a lei ao bit em apenas {bad} de {total} arestas \
             — se ele ja' fosse exacto, este gate nao estaria a medir o passo que \
             existe para o tornar exacto"
        );

        // ⭐⭐ **E o fenómeno CRESCE com o expoente, que é a afirmação do §2.3.**
        //
        // Re-calibrar o mapa por translações inteiras não muda o mapa — muda a
        // magnitude dos números. Se a perda de bits fosse acidental, o número não se
        // moveria; ele multiplica-se.
        let far = support::regauge(&m, 1 << 16);
        let (bad_far, total_far) = inexact_on_raw(&far);
        assert_eq!(
            total_far, total,
            "{name}: a re-calibracao nao muda a topologia"
        );
        assert!(
            bad_far > bad * 4,
            "{name}: com as cartas afastadas, a lei tinha de falhar MUITO mais \
             ({bad_far} contra {bad}) — se nao falha, a perda de bits nao vem do \
             expoente e o §2.3 estaria a curar outra coisa"
        );

        // ── E a extracção remove-o: TODA transição se relê ao bit dos valores
        // saneados, para os dois vértices da aresta ao mesmo tempo.
        let (mesh, r) = extract(&m.as_map(), None).unwrap();
        assert_eq!(
            r.inexact_transitions, 0,
            "{name}: {} transicoes nao se releram exactamente",
            r.inexact_transitions
        );
        assert_eq!(
            r.holonomy_broken, 0,
            "{name}: leques regulares que nao fecham"
        );
        // ⚠️ A grade comum tem de existir e ter sub-célula — sem pelo menos um bit
        // abaixo da célula, o ponto fixo de uma singularidade (que é uma METADE) não
        // se representa.
        assert!(
            r.grid_exponent >= 1,
            "{name}: a grade tem de ter sub-celula, e o expoente e' {}",
            r.grid_exponent
        );

        // ⭐⭐⭐ **E a saída é INVARIANTE à re-calibração.** É a prova de que o
        // saneamento absorveu a magnitude em vez de a herdar.
        let (mesh_far, r_far) = extract(&far.as_map(), None).unwrap();
        assert_eq!(r_far.inexact_transitions, 0, "{name}: re-calibrado");
        assert_eq!(
            r_far.quads, r.quads,
            "{name}: a re-calibracao por inteiros mudou a contagem de quads \
             ({} contra {})",
            r_far.quads, r.quads
        );
        assert_eq!(
            ph2d_quadextract::euler_characteristic(&mesh_far),
            ph2d_quadextract::euler_characteristic(&mesh),
            "{name}: a re-calibracao mudou a topologia da saida"
        );
    }
}

#[test]
fn somar_e_tirar_um_inteiro_grande_a_um_numero_pequeno_perde_bits() {
    // ⭐ O mecanismo, na sua forma mais nua — é isto que acontece ao dar a volta a um
    // leque cujas cartas vivem em expoentes diferentes.
    let x = 0.300_000_000_000_000_04_f64;
    let t = 55.0_f64;
    assert_ne!(
        (x + t) - t,
        x,
        "se isto fosse igual, o §2.3 nao teria razao de existir nesta maquina"
    );
    // ⭐⭐ E a cura: truncar a mantissa para a grade que a carta MAIOR consegue
    // representar. Aí a ida e a volta são exactas.
    //
    // ⚠️ A grade é `2^(M−51)` com `M` o maior expoente binário em jogo — aqui `5`,
    // porque `55 < 64`.
    let grid = f64::from_bits(u64::from(1023_i32.wrapping_add(5 - 51) as u32) << 52);
    let g = (x / grid).trunc() * grid;
    assert_eq!((g + t) - t, g, "depois da truncagem a ida e a volta fecham");
    assert!(
        (g - x).abs() < grid,
        "e a truncagem custa menos de um passo da grade"
    );
}

#[test]
fn o_ponto_fixo_e_mesmo_fixo() {
    // ── GATE 3, na forma algébrica: numa singularidade a transição acumulada FIXA a
    // imagem saneada. ⚠️ Em unidades internas, e é por isso que a grade tem de ter
    // sub-célula: o ponto fixo é uma METADE.
    for r in 1..4u8 {
        for t in [
            [0i64, 0],
            [2, 0],
            [0, -2],
            [6, -10],
            [1 << 30, -(1 << 28)],
            [-(1 << 20), 1 << 20],
        ] {
            let g = Xf { r, t };
            let p = g.fixed_point().expect("r != 0 tem ponto fixo");
            assert_eq!(g.apply(p), p, "r={r} t={t:?}");
        }
    }
    assert!(
        Xf { r: 0, t: [4, 0] }.fixed_point().is_none(),
        "uma translacao pura NAO tem ponto fixo, e dizer que tem seria a resposta errada \
         para a singularidade de valencia multipla de 4"
    );
}

#[test]
fn a_valencia_grosseira_distingue_o_que_precisa_de_distinguir() {
    // ⚠️ **Só é preciso distinguir `4` de `≥ 8`.** O que se cobra aqui é que a
    // contagem própria da crate reencontra as singularidades que as peças têm — e
    // que a esmagadora maioria dos vértices é regular, que é a forma de uma malha sã.
    for (name, m) in [("gancho", support::hooked()), ("toro", support::torus())] {
        let (_, r) = extract(&m.as_map(), None).unwrap();
        let total: usize = r.valence.iter().sum();
        assert!(total > 2000, "{name}: {total} vertices");
        let regular = r.valence[4];
        assert!(
            regular * 100 > total * 98,
            "{name}: so' {regular} de {total} vertices sao regulares"
        );
        let singular: usize = total - regular;
        assert!(
            singular > 0,
            "{name}: nenhuma singularidade — a peca nao contem o fenomeno"
        );
        assert_eq!(
            r.pinned_fixed + r.pinned_integer,
            singular,
            "{name}: toda singularidade tem de ser pregada, por uma das duas leis"
        );
    }
}
