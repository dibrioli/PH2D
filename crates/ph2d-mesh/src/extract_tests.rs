//! Ver o `extract.rs` — este arquivo é o `mod tests` dele.
//!
//! ⚠️ O oráculo central **não** confere que a costura foi escrita como eu a
//! derivei: ele pergunta se a peça que saiu é uma **superfície fechada e
//! coerentemente enrolada**. Uma ponte com o enrolamento trocado passa por
//! qualquer contagem de faces e é reprovada por essa pergunta — e é ela, e não a
//! minha derivação, que decide.

use super::*;
use crate::shapes;

/// Mascara o hemisfério `y > 0`: uma calota com fronteira longa, que é onde a
/// costura vive.
fn dome(seg: usize, ring: usize) -> Mesh {
    let mut m = shapes::uv_sphere(seg, ring, 1.0);
    let n = m.vert_count();
    let up: Vec<bool> = (0..n).map(|i| m.positions()[i][1] > 0.0).collect();
    let mask = m.masks_mut();
    for i in 0..n {
        mask[i] = f32::from(u8::from(up[i]));
    }
    m
}

/// Uma calota com um **RISCO** de um vértice de largura atravessando a máscara
/// — a fixture que separa a lei da fronteira da heurística da referência.
///
/// ⚠️ Ela contém o fenômeno de propósito, e o gesto é real: uma máscara pintada
/// à mão deixa falhas. As faces dos DOIS lados do risco entram no recorte (cada
/// uma toca um vértice mascarado), então uma aresta ao longo do risco tem DUAS
/// faces recortadas — miolo — enquanto as duas pontas dela estão fora da
/// máscara, que é exatamente o que a regra por-VÉRTICE lê como fronteira.
fn gapped_dome() -> Mesh {
    let mut m = shapes::uv_sphere(24, 32, 1.0);
    let n = m.vert_count();
    let keep: Vec<bool> = (0..n)
        .map(|i| {
            let y = m.positions()[i][1];
            // A calota, menos um risco fino no meio dela.
            y > 0.0 && !(0.45..0.55).contains(&y)
        })
        .collect();
    let mask = m.masks_mut();
    for i in 0..n {
        mask[i] = f32::from(u8::from(keep[i]));
    }
    m
}

/// Quantas faces contêm cada aresta, sem direção.
fn edge_faces(mesh: &Mesh) -> Vec<(u64, usize)> {
    let mut keys: Vec<u64> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            keys.push(key(v[k], v[(k + 1) % v.len()]));
        }
    }
    keys.sort_unstable();
    let mut out: Vec<(u64, usize)> = Vec::new();
    let mut i = 0;
    while i < keys.len() {
        let run = keys[i..].iter().take_while(|&&x| x == keys[i]).count();
        out.push((keys[i], run));
        i += run;
    }
    out
}

/// O volume COM SINAL de uma malha fechada — positivo quando ela é enrolada
/// para FORA.
///
/// ⚠️ É o teorema da divergência, e é o único oráculo que separa *"a casca
/// fechou"* de *"a casca fechou do lado certo"*: inverter a peça inteira mantém
/// toda aresta com duas faces em sentidos opostos e só troca o sinal disto.
fn signed_volume(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let mut v = 0.0f32;
    for f in mesh.faces() {
        let idx = f.verts();
        for k in 1..idx.len() - 1 {
            let (a, b, c) = (
                p[idx[0] as usize],
                p[idx[k] as usize],
                p[idx[k + 1] as usize],
            );
            let cr = [
                b[1] * c[2] - b[2] * c[1],
                b[2] * c[0] - b[0] * c[2],
                b[0] * c[1] - b[1] * c[0],
            ];
            v += a[0] * cr[0] + a[1] * cr[1] + a[2] * cr[2];
        }
    }
    v / 6.0
}

/// As arestas DIRIGIDAS, para a pergunta do enrolamento.
fn directed(mesh: &Mesh) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            out.push((v[k], v[(k + 1) % v.len()]));
        }
    }
    out
}

/// **UMA CASCA É FECHADA E COERENTEMENTE ENROLADA** — o oráculo que decide a
/// ponte, e não a derivação que a escreveu.
///
/// ⚠️ **Duas metades, e nenhuma basta sozinha.** *Fechada* diz que não sobrou
/// beira (toda aresta tem duas faces); *coerente* diz que as duas faces a
/// percorrem em sentidos OPOSTOS. Uma ponte com o enrolamento trocado deixa a
/// primeira metade verde — a aresta continua com duas faces — e é vista só pela
/// segunda, que é onde a luz de fato quebra.
#[test]
fn a_shell_is_closed_and_consistently_wound() {
    for thickness in [0.08_f32, -0.08] {
        let src = dome(24, 32);
        let out = extract_masked(
            &src,
            Extract {
                thickness,
                smooth: 0,
            },
        )
        .expect("a calota mascarada tem o que extrair");

        let open: Vec<_> = edge_faces(&out)
            .iter()
            .filter(|&&(_, n)| n != 2)
            .copied()
            .collect();
        assert!(
            open.is_empty(),
            "espessura {thickness}: {} aresta(s) sem exatamente duas faces -- a casca nao fechou",
            open.len()
        );

        let dir = directed(&out);
        let mut bad = 0;
        for &(a, b) in &dir {
            if !dir.contains(&(b, a)) {
                bad += 1;
            }
        }
        assert_eq!(
            bad, 0,
            "espessura {thickness}: {bad} aresta(s) percorridas no MESMO sentido pelas duas faces \
             -- a costura esta' com o enrolamento trocado"
        );

        // ⚠️ **A TERCEIRA metade, e ela não é redundante:** inverter a peça
        // INTEIRA deixa as duas anteriores verdes (toda aresta segue com duas
        // faces em sentidos opostos) e entrega uma casca com a face de fora
        // olhando para dentro — a luz a desenha vazada. Só o sinal do volume vê
        // isso.
        let vol = signed_volume(&out);
        println!("espessura {thickness}: volume com sinal {vol:+.5}");
        assert!(
            vol > 0.0,
            "espessura {thickness}: volume {vol:+.5} -- a casca fechou pelo AVESSO"
        );
    }
}

/// **O RELAXAMENTO NÃO DEPENDE DA ORDEM DA LISTA** — a forma de Jacobi.
///
/// ⚠️ Uma varredura que lê o que ela mesma acabou de escrever desloca a costura
/// na direção em que varre, e a lista de vértices sai de uma travessia de faces:
/// ela mudaria no dia em que o recorte mudasse de forma, e a peça sairia
/// diferente sem ninguém tocar num knob.
#[test]
fn relaxing_does_not_depend_on_the_order_of_the_list() {
    let src = dome(24, 32);
    let build = || {
        extract_masked(
            &src,
            Extract {
                thickness: 0.0,
                smooth: 0,
            },
        )
        .expect("ha' o que extrair")
    };
    let (mut a, mut b) = (build(), build());
    let bound = {
        let mut kept: Vec<u32> = Vec::new();
        let masks = src.masks().expect("mascarada");
        for (fi, f) in src.faces().iter().enumerate() {
            if f.verts().iter().any(|&v| masks[v as usize] >= MASK_CLAMP) {
                kept.push(u32::try_from(fi).unwrap());
            }
        }
        Boundary::of(&src, &kept)
    };
    let mut forward: Vec<u32> = (0..a.vert_count() as u32)
        .filter(|&v| a.adjacency().is_border(v as usize))
        .collect();
    assert!(!forward.is_empty(), "a folha tem beira");
    let _ = &bound;
    let backward: Vec<u32> = forward.iter().rev().copied().collect();
    forward.dedup();

    relax(&mut a, &forward, 4);
    relax(&mut b, &backward, 4);
    assert_eq!(
        a.positions(),
        b.positions(),
        "a mesma costura relaxada em ordens diferentes deu geometrias diferentes -- \
         a passada le' o que ela mesma escreveu"
    );
}

/// **UMA FOLHA SAI OLHANDO PARA O MESMO LADO DA SUPERFÍCIE DE ONDE VEIO.**
///
/// Sem espessura não há casca a inverter, e inverter mesmo assim daria um trecho
/// que a luz desenha pelo avesso — visível na hora, e exatamente o tipo de coisa
/// que um smoke reprova sem saber nomear.
#[test]
fn a_sheet_keeps_the_orientation_of_the_surface_it_came_from() {
    let src = dome(24, 32);
    let out = extract_masked(
        &src,
        Extract {
            thickness: 0.0,
            smooth: 0,
        },
    )
    .expect("ha' o que extrair");

    // A calota é do hemisfério de cima de uma esfera na origem: a normal de todo
    // vértice aponta para FORA, logo `n . p > 0`.
    let mut wrong = 0;
    for (p, n) in out.positions().iter().zip(out.normals()) {
        if p[0] * n[0] + p[1] * n[1] + p[2] * n[2] <= 0.0 {
            wrong += 1;
        }
    }
    assert_eq!(
        wrong, 0,
        "{wrong} vertice(s) da folha com a normal para DENTRO -- ela saiu pelo avesso"
    );
}

/// **A FAIXA FINA NÃO GANHA UMA PONTE PELO MEIO** — a lei da fronteira, e a
/// razão de ela ser sobre ARESTAS.
///
/// ⚠️ O gate mede também o que a regra por-VÉRTICE faria na mesma fixture, e
/// imprime os dois números: sem isso, *"a topologia é melhor"* seria uma
/// afirmação sobre um caso que ninguém construiu.
#[test]
fn the_thin_strip_gets_no_bridge_through_its_middle() {
    let src = gapped_dome();
    let out = extract_masked(
        &src,
        Extract {
            thickness: 0.06,
            smooth: 0,
        },
    )
    .expect("a calota riscada tem o que extrair");

    let bad: Vec<_> = edge_faces(&out)
        .iter()
        .filter(|&&(_, n)| n != 2)
        .copied()
        .collect();
    assert!(
        bad.is_empty(),
        "{} aresta(s) fora de duas faces na calota riscada -- ha' costura no MIOLO",
        bad.len()
    );

    // O que a regra da referência marcaria: TODA aresta cujas duas pontas estão
    // fora da máscara ou na beira.
    let masks = src.masks().expect("a fixture mascara");
    let adj = src.adjacency();
    let fringe = |v: u32| masks[v as usize] < MASK_CLAMP || adj.is_border(v as usize);
    let mut by_vertex = 0usize;
    let mut by_topology = 0usize;
    let mut kept: Vec<u32> = Vec::new();
    for (fi, f) in src.faces().iter().enumerate() {
        if f.verts().iter().any(|&v| masks[v as usize] >= MASK_CLAMP) {
            kept.push(u32::try_from(fi).unwrap());
        }
    }
    let bound = Boundary::of(&src, &kept);
    for &fi in &kept {
        let v = src.faces()[fi as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if fringe(a) && fringe(b) {
                by_vertex += 1;
            }
            if bound.is_boundary(a, b) {
                by_topology += 1;
            }
        }
    }
    println!(
        "calota riscada: a regra por VERTICE costuraria {by_vertex} arestas, a por TOPOLOGIA costura \
         {by_topology}"
    );
    assert!(
        by_vertex > by_topology,
        "a fixture nao contem o fenomeno: as duas regras concordaram ({by_vertex} = {by_topology}), \
         entao este gate nao esta' medindo a diferenca que ele existe para medir"
    );
}

/// **O ERGUIMENTO É FRAÇÃO DA ARESTA, NÃO UM COMPRIMENTO** — o número que a
/// Fase 0 mediu.
///
/// ⚠️ O oráculo é a RAZÃO entre duas malhas de resolução diferente: um
/// comprimento absoluto daria o mesmo erguimento nas duas, e a fina ficaria com
/// a casca flutuando. A razão dos erguimentos tem de seguir a razão das arestas.
#[test]
fn the_lift_is_a_fraction_of_the_edge_not_a_length() {
    let lift_of = |seg, ring| {
        let src = dome(seg, ring);
        let out = extract_masked(
            &src,
            Extract {
                thickness: 0.0,
                smooth: 0,
            },
        )
        .expect("ha' o que extrair");
        // O polo (`+Y`) sobrevive a todo recorte: a distância dele à origem
        // menos o raio 1 É o erguimento.
        let top = out
            .positions()
            .iter()
            .max_by(|a, b| a[1].partial_cmp(&b[1]).unwrap())
            .copied()
            .unwrap();
        (top[0] * top[0] + top[1] * top[1] + top[2] * top[2]).sqrt() - 1.0
    };
    let coarse = lift_of(24, 32);
    let fine = lift_of(128, 192);
    // As arestas medianas medidas na Fase 0: 0,13081 e 0,02454 -> razão 5,33.
    let ratio = coarse / fine;
    println!("erguimento: grosseira {coarse:.5}  fina {fine:.5}  razao {ratio:.2}");
    assert!(
        (4.5..6.5).contains(&ratio),
        "a razao dos erguimentos e' {ratio:.2}, fora da razao das arestas (5,33): o erguimento \
         deixou de seguir a resolucao da malha"
    );
}

/// **SEM MÁSCARA NÃO HÁ O QUE EXTRAIR** — e recusar é a resposta, não uma peça
/// vazia que o artista teria de apagar.
#[test]
fn without_a_mask_there_is_nothing_to_extract() {
    let plain = shapes::uv_sphere(24, 32, 1.0);
    assert!(
        plain.masks().is_none(),
        "a fixture nasce sem plano de mascara"
    );
    assert!(extract_masked(&plain, Extract::default()).is_none());

    let mut zeroed = shapes::uv_sphere(24, 32, 1.0);
    zeroed.masks_mut().fill(0.0);
    assert!(
        extract_masked(&zeroed, Extract::default()).is_none(),
        "uma mascara toda em zero produziu peca"
    );
}

/// **A ORIGEM NÃO É TOCADA** — nem a geometria dela, nem a máscara.
///
/// Extrair de novo com outra espessura é o gesto normal (é o que o ZBrush faz),
/// e ele exige que a máscara siga lá.
#[test]
fn the_source_is_not_touched() {
    let src = dome(24, 32);
    let before_pos = src.positions().to_vec();
    let before_mask = src.masks().expect("mascarada").to_vec();
    let (bv, bf) = (src.vert_count(), src.face_count());

    let _ = extract_masked(&src, Extract::default()).expect("ha' o que extrair");

    assert_eq!((src.vert_count(), src.face_count()), (bv, bf));
    assert_eq!(src.positions(), &before_pos[..]);
    assert_eq!(src.masks().expect("segue mascarada"), &before_mask[..]);
}

/// **A COSTURA RELAXADA DESLIZA AO LONGO DE SI MESMA, EM VEZ DE SUBIR PARA
/// DENTRO DA FOLHA** — a regra de borda do
/// [`ring_average`](crate::ring_average) trabalhando dentro do extract.
///
/// ⚠️ **A primeira versão deste gate afirmava a coisa errada e a medição a
/// derrubou:** eu escrevi *"relaxar não ENCOLHE"* e ele reprovou com o alcance
/// caindo 3,4% em seis passes. O encolhimento é REAL e é geometria, não defeito
/// — a borda de uma calota é um círculo, e a média de um ponto com os dois
/// vizinhos dele num círculo cai na CORDA, que é por dentro. É o encurtamento de
/// curva, e todo relaxamento de borda o tem.
///
/// O que a regra de borda de fato compra é outra coisa, e é ela que o gate mede:
/// sem ela o vértice da beira medeia também com os vizinhos do anel de DENTRO,
/// que estão mais acima na calota, e a boca **SOBE para dentro da folha** — a
/// peça perde a fronteira que o artista pintou. Com ela os vizinhos estão no
/// mesmo anel, e a beira alisa ao longo dela mesma.
#[test]
fn the_relaxed_seam_slides_along_itself_instead_of_climbing_into_the_sheet() {
    let src = dome(24, 32);
    let sheet = |smooth| {
        extract_masked(
            &src,
            Extract {
                thickness: 0.0,
                smooth,
            },
        )
        .expect("ha' o que extrair")
    };
    let plain = sheet(0);
    let relaxed = sheet(6);

    // A altura média da BEIRA: é ela que sobe quando o anel de dentro entra na
    // média.
    let lip = |m: &Mesh| {
        let adj = m.adjacency();
        let ys: Vec<f32> = (0..m.vert_count())
            .filter(|&i| adj.is_border(i))
            .map(|i| m.positions()[i][1])
            .collect();
        assert!(!ys.is_empty(), "uma folha tem beira");
        ys.iter().sum::<f32>() / ys.len() as f32
    };
    // O alcance no plano do equador: o encolhimento, que é MEDIDO e nomeado em
    // vez de negado.
    let reach = |m: &Mesh| {
        m.positions()
            .iter()
            .map(|p| p[0].mul_add(p[0], p[2] * p[2]).sqrt())
            .fold(0.0_f32, f32::max)
    };

    let (lip0, lip6) = (lip(&plain), lip(&relaxed));
    let (r0, r6) = (reach(&plain), reach(&relaxed));
    let climb = lip6 - lip0;
    println!(
        "beira: y medio {lip0:.4} -> {lip6:.4} (subiu {climb:+.4})   alcance {r0:.4} -> {r6:.4} \
         ({:+.1}%)",
        (r6 / r0 - 1.0) * 100.0
    );
    assert!(
        climb < 0.02,
        "a beira SUBIU {climb:+.4} em seis passes: ela esta' medindo com o anel de dentro e a \
         folha perdeu a fronteira que o artista pintou"
    );
}

/// **A ESPESSURA CHEGA À PEÇA** — e o sinal escolhe de que lado da superfície
/// ela cresce.
#[test]
fn the_thickness_is_the_thickness() {
    let src = dome(24, 32);
    let out_far = extract_masked(
        &src,
        Extract {
            thickness: 0.20,
            smooth: 0,
        },
    )
    .expect("ha' o que extrair");
    let out_in = extract_masked(
        &src,
        Extract {
            thickness: -0.20,
            smooth: 0,
        },
    )
    .expect("ha' o que extrair");

    let span = |m: &Mesh| {
        let r: Vec<f32> = m
            .positions()
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
            .collect();
        (
            r.iter().copied().fold(f32::MAX, f32::min),
            r.iter().copied().fold(0.0_f32, f32::max),
        )
    };
    let (lo_out, hi_out) = span(&out_far);
    let (lo_in, hi_in) = span(&out_in);
    println!("para fora: raio {lo_out:.4}..{hi_out:.4}   para dentro: {lo_in:.4}..{hi_in:.4}");
    assert!(
        hi_out > 1.19,
        "espessura +0,20 nao chegou para fora (raio maximo {hi_out:.4})"
    );
    assert!(
        lo_in < 0.81,
        "espessura -0,20 nao chegou para dentro (raio minimo {lo_in:.4})"
    );
}
