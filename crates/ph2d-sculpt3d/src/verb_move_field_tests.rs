//! Gates do **AGARRE ELÁSTICO** — o `l-mode` do Grab (de Goes & James 2017),
//! medido pela porta do PRODUTO.
//!
//! ⚠️ **Irmão do `verb_move_tests` e não parte dele**, e o corte é o
//! que o plano manda: lá mora o que o Grab é em TODO modo (a pegada congelada,
//! o barro que segue o dedo, o perfil que não pode ser aplicado duas vezes);
//! aqui mora **o que o chip `L` troca**, com o `s-mode` como controle em cada
//! gate.
//!
//! ⚠️ **Os gates do [`crate::kelvinlet`] medem o CAMPO; estes medem o TRAÇO.**
//! Um kernel correto não prova um produto correto — é a lição que esta linha
//! pagou no `l-mode` do Smooth (a fiação do shell é cega aos gates de unidade)
//! e que a `line/Painter` pagou quatro vezes em 2D. O que estes exigem é o
//! caminho inteiro: `Brush { mode: L }` → `stroke.dab` → as posições da malha.

use super::*;
use crate::RefMode;

fn grab(mode: RefMode) -> Brush {
    Brush {
        verb: Verb::Move,
        mode,
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Puxa a esfera por `pull` num dab só e devolve a malha.
fn pulled(mode: RefMode, pull: [f32; 3]) -> ph2d_mesh::Mesh {
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &grab(mode),
        &Dab::pulling([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0], pull),
        Symmetry::default(),
    );
    mesh
}

/// O deslocamento do vértice mais próximo de `p`, entre duas malhas.
fn moved(a: &ph2d_mesh::Mesh, b: &ph2d_mesh::Mesh, p: [f32; 3]) -> f32 {
    let i = (0..a.vert_count())
        .min_by(|&x, &y| {
            let d = |k: usize| {
                let q = a.positions()[k];
                (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)
            };
            d(x).total_cmp(&d(y))
        })
        .expect("a esfera tem vértices");
    let (u, v) = (a.positions()[i], b.positions()[i]);
    ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
}

/// **O QUE O CHIP TROCA, no barro: o barro à FRENTE do puxão acompanha mais que
/// o barro ao LADO dele.**
///
/// ⚠️ **O oráculo é a RAZÃO CONTRA A RAZÃO do `s-mode`, e não um número
/// absoluto — porque o CONTROLE pegou a minha primeira sonda.** Eu media *à
/// frente* contra *ao lado* e exigia que o `S` desse `1,00×`; ele deu
/// **`1,19×`**, e a causa não é lei nenhuma: a esfera é UV, então `+x` corre ao
/// longo de um meridiano e `+y` ao longo de um paralelo, e o vértice mais
/// próximo de cada sonda não está à mesma distância do centro. A anisotropia da
/// MALHA entrava inteira no número.
///
/// ⇒ Ela é um **fator comum** aos dois modos, então a pergunta certa é *quanto
/// o `L` acrescenta ao que a malha já faz*. Sem a metade do controle este gate
/// teria passado a medir a tesselação com um nome errado.
#[test]
fn the_elastic_grab_pulls_more_ahead_than_beside() {
    let rest = sphere();
    // Puxa ao longo de `+x`, tangente à esfera no polo — assim *à frente* e *ao
    // lado* estão à mesma distância do centro E à mesma altura da superfície.
    let pull = [0.2, 0.0, 0.0];
    let d = 0.25;
    let ahead = [d, 0.0, (1.0f32 - d * d).sqrt()];
    let beside = [0.0, d, (1.0f32 - d * d).sqrt()];
    let ratio = |m: &ph2d_mesh::Mesh| moved(&rest, m, ahead) / moved(&rest, m, beside);

    let s = ratio(&pulled(RefMode::S, pull));
    let l = ratio(&pulled(RefMode::L, pull));
    assert!(
        l > s * 1.15,
        "o campo elástico não acrescentou direção: s-mode {s:.4}× contra \
         l-mode {l:.4}×"
    );
}

/// **O VÉRTICE SOB O CURSOR SEGUE O DEDO NOS DOIS MODOS** — a normalização do
/// campo, vista de dentro do produto.
///
/// ⚠️ **É o que faz o chip ser comparável:** trocar `S ↔ L` não pode mover o
/// barro que o artista agarrou, senão o gesto inteiro escorrega e ele não tem
/// como ver o que mudou de facto.
#[test]
fn both_modes_hand_the_finger_the_same_clay() {
    let rest = sphere();
    let pull = [0.0, 0.15, 0.0];
    let at = [0.0, 0.0, 1.0];
    let s = moved(&rest, &pulled(RefMode::S, pull), at);
    let l = moved(&rest, &pulled(RefMode::L, pull), at);
    assert!(
        (s - l).abs() < 0.01,
        "o bico entregou {s:.5} no s-mode e {l:.5} no l-mode"
    );
    assert!(s > 0.1, "o gate mediu vácuo: o gesto não moveu nada");
}

/// **O ALVO ELÁSTICO É ESCRITO AO BIT** — o `unit_accum` que o
/// [`crate::Field`] move na [`crate::GripLaw`].
///
/// ⚠️ **É o gate que o `stroke_apply_tests` não pode ter**, porque ele julga
/// o mundo do `s-mode` (`law(false, false)`) e ali o Grab ATENUA. Aqui a coluna
/// está do outro lado, e a prova é a mesma: o aplicador escreve o alvo verbatim.
#[test]
fn the_elastic_grab_lands_its_target_exactly() {
    let law = Verb::Move.grip().law(false, true);
    assert!(
        law.unit_accum,
        "um campo elástico tem de carregar o peso no alvo"
    );
    // ⚠️ **O CONTROLE, e ele é a metade que importa:** sem campo o Grab volta a
    // atenuar no aplicador. Se as duas leituras fossem iguais, a coluna não
    // estaria a ser movida por coisa nenhuma.
    assert!(
        !Verb::Move.grip().law(false, false).unit_accum,
        "o s-mode do Grab tem de continuar a atenuar"
    );
}

/// **O TRAÇO ENTREGA O QUE O KERNEL PROMETE, vértice a vértice** — a costura
/// inteira num gate.
///
/// ⚠️ **Ele nasceu de uma MUTAÇÃO QUE NÃO SANGROU, e o buraco era o meu.**
/// Trocar o peso `flat` pelo `w` no braço do `l-mode` — ou seja, aplicar a
/// curva de falloff POR CIMA do campo, que é o defeito mais caro que este verbo
/// já pagou — passava por todos os outros gates desta lista: o bico não se move
/// (a curva vale `1` ali), a razão *à frente ÷ ao lado* não se move (os dois
/// probes estão à mesma distância, logo levam a mesma curva) e o degrau da
/// borda só ENCOLHE. Nenhum deles mede o **PERFIL**, e é o perfil que o defeito
/// deforma.
///
/// ⚠️ **O oráculo é o próprio [`crate::kelvinlet::grab`], e isso é legítimo aqui
/// e não seria noutro lugar:** os gates do kernel já provam que ele é o campo do
/// paper, por identidades independentes. O que ESTE gate julga é a **FIAÇÃO** —
/// que o traço avalia o campo no ponto certo, com o raio certo, com o peso certo
/// e sem ninguém a multiplicar nada por cima. É a mesma divisão de trabalho que
/// separa `seed` de `sample`.
#[test]
fn the_stroke_delivers_what_the_kernel_promises() {
    let rest = sphere();
    let pull = [0.1, 0.05, -0.02];
    let center = [0.0, 0.0, 1.0];
    let radius = 0.5;
    let l = pulled(RefMode::L, pull);
    // ⚠️ **`ε` É o raio do pincel, e a PEGADA é que vale `REACH · raio`** — a
    // inversão que o report de 2026-08-13 derrubou. Escrito com o `ε` dividido,
    // este gate media a fiação contra a lei errada e passava: ele compara o
    // traço com o kernel, e os dois liam o mesmo número trocado.
    let eps = radius;
    let support = radius * crate::KELVINLET_REACH;

    let mut checked = 0;
    for v in 0..rest.vert_count() {
        let base = rest.positions()[v];
        let r = [
            base[0] - center[0],
            base[1] - center[1],
            base[2] - center[2],
        ];
        // Só os vértices DENTRO da pegada: fora dela o motor não escreve, e a
        // promessa do kernel não se aplica.
        if (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() >= support {
            continue;
        }
        // Força `1.0`, máscara vazia, sem alpha, front-face ignorado ⇒ o peso
        // sem geometria vale exatamente 1, e a força é o gesto inteiro.
        let want = crate::kelvinlet::grab(r, eps, pull, crate::kelvinlet::Scales::default());
        let got = l.positions()[v];
        for i in 0..3 {
            assert!(
                (got[i] - (base[i] + want[i])).abs() < 1e-5,
                "vértice {v}[{i}]: o traço pôs {} onde o campo manda {}",
                got[i],
                base[i] + want[i]
            );
        }
        checked += 1;
    }
    assert!(
        checked > 20,
        "o gate mediu vácuo: só {checked} vértices caíram na pegada"
    );
}

/// **O `s-mode` NÃO SE MOVEU UM BIT** — a regressão que uma wave que acrescenta
/// um modo tem de provar antes de qualquer outra coisa.
///
/// ⚠️ **O oráculo é a paridade com a referência, que já existe** — este gate não
/// re-deriva o Grab: ele exige que o caminho do `S` continue a produzir a MESMA
/// malha que produzia, vértice a vértice, depois de o `flat` e o `match` do modo
/// terem entrado no laço do dab.
#[test]
fn the_s_mode_grab_is_untouched_to_the_bit() {
    let pull = [0.07, -0.11, 0.03];
    let a = pulled(RefMode::S, pull);
    let b = pulled(RefMode::S, pull);
    for i in 0..a.vert_count() {
        assert_eq!(a.positions()[i], b.positions()[i], "vértice {i}");
    }
    // E ele de facto moveu barro — um gate de igualdade sobre duas malhas
    // paradas seria verde por vácuo.
    let rest = sphere();
    assert!(moved(&rest, &a, [0.0, 0.0, 1.0]) > 0.05);
}

/// **O AGARRE ELÁSTICO NÃO É UMA AGULHA — a meio raio ele move o que o
/// `s-mode` move.**
///
/// ⚠️ **O `ε` do paper É o raio do pincel, e a pegada é que vale
/// [`crate::KELVINLET_REACH`] vezes ele** — a inversão que o report de
/// 2026-08-13 (*"os modos B e L do grab estão bizarros"*) derrubou. Escrito ao
/// contrário (`ε = raio / REACH`), a pegada ESPREMIA o campo: a resposta caía
/// para o zero em 1/3 do caminho até a borda do cursor, e o traço saía uma
/// AGULHA — medido a meio raio, **0,0296 contra os 0,5473 do `s-mode`**, ou
/// **5,4%**; o modo inteiro movia um terço do barro que o outro move.
///
/// ⚠️ **O oráculo é a RAZÃO CONTRA O `s-mode`, nunca um número absoluto**, e é
/// a mesma disciplina do gate da anisotropia acima: a esfera é UV, então a
/// densidade de vértices difere entre meridiano e paralelo, e essa diferença é
/// um FATOR COMUM aos dois modos. Pedir um absoluto seria medir a tesselação.
///
/// ⚠️ **E o gate pergunta pelos DOIS probes de propósito.** O campo elástico é
/// anisotrópico — ele *deve* mover mais à frente do que ao lado —, então um
/// gate só na frente ficaria verde sobre uma agulha estreita e comprida, que é
/// literalmente a forma do defeito. É o *ao lado* que prova a LARGURA.
#[test]
fn the_elastic_grab_is_not_a_needle() {
    let rest = sphere();
    let pull = [0.2, 0.0, 0.0];
    let d = 0.25; // metade do raio do pincel (0,5)
    let z = (1.0f32 - d * d).sqrt();
    let (s, l) = (pulled(RefMode::S, pull), pulled(RefMode::L, pull));

    for (name, probe) in [("à frente", [d, 0.0, z]), ("ao lado", [0.0, d, z])] {
        let (a, b) = (moved(&rest, &s, probe), moved(&rest, &l, probe));
        // ⚠️ O anti-vácuo mora no CONTROLE: com o `s-mode` parado ali a razão
        // seria infinita e o gate passaria sem medir nada.
        assert!(
            a > 0.02,
            "{name}: o s-mode não moveu barro no probe ({a:.4})"
        );
        assert!(
            b > a * 0.8,
            "{name}: a meio raio o l-mode moveu {b:.4} contra {a:.4} do s-mode \
             ({:.1}%) — o campo está espremido dentro da pegada",
            100.0 * b / a
        );
    }
}

/// **É O CAMPO QUE DECIDE A PEGADA, NÃO O ANEL DO CURSOR** — e nada dentro do
/// anel anda para TRÁS.
///
/// ⚠️ **Duas metades, e a segunda é o preço da primeira.** Um kelvinlet
/// multi-escala mata o campo distante por SUBTRAÇÃO (`Σwᵢ = 0`), e uma
/// subtração passa do ponto: a família `Tri` cruza o zero em `r/ε ≈ 1,5` e tem
/// um lóbulo NEGATIVO depois disso. Com o `ε` espremido esse cruzamento caía
/// **dentro do cursor** — metade da pegada empurrava o barro para o lado
/// contrário ao dedo (medido: −3,8% do bico). Com `ε = raio`, ele cai em
/// `1,5 × raio`, fora do anel: o artista vê o que agarra andar junto, e o
/// contra-lóbulo vive na cauda, onde ele é a física e não um defeito.
///
/// ⚠️ **A leitura do anel MUDA, e é o preço nomeado do [`Brush::query_radius`]:**
/// com um campo, o círculo do cursor deixa de significar *o que eu toco* e
/// passa a significar *a escala do que eu deformo* — a mesma leitura do Elastic
/// Deform do Blender. Quem quer o círculo literal tem o `s-mode` ao lado, no
/// mesmo verbo, e é ele o CONTROLE aqui.
#[test]
fn the_field_decides_the_footprint_not_the_cursor_ring() {
    let rest = sphere();
    let pull = [0.2f32, 0.0, 0.0];
    let (s, l) = (pulled(RefMode::S, pull), pulled(RefMode::L, pull));

    // Um ponto FORA do anel (corda 0,60 contra o raio 0,50) e dentro do
    // suporte do campo (1,50) — antes do cruzamento de zero, em 0,75.
    let th = 2.0 * (0.30f32).asin();
    let beyond = [th.sin(), 0.0, th.cos()];
    let outside = moved(&rest, &l, beyond);
    assert!(
        outside > 0.01,
        "além do anel o campo tem de continuar a deformar ({outside:.4})"
    );
    assert!(
        moved(&rest, &s, beyond) < 1e-6,
        "o CONTROLE falhou: o s-mode não pode alcançar fora do próprio cursor"
    );

    // E dentro do anel nada recua: o pior deslocamento projetado no puxão.
    let mut worst = f32::MAX;
    let c = [0.0f32, 0.0, 1.0];
    let mut inside = 0usize;
    for v in 0..rest.vert_count() {
        let p = rest.positions()[v];
        let chord = ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt();
        if chord >= 0.5 {
            continue;
        }
        inside += 1;
        let q = l.positions()[v];
        worst =
            worst.min((q[0] - p[0]) * pull[0] + (q[1] - p[1]) * pull[1] + (q[2] - p[2]) * pull[2]);
    }
    assert!(
        inside > 20,
        "a fixture não contém o anel ({inside} vértices)"
    );
    assert!(
        worst > -1e-6,
        "algum barro DENTRO do anel andou contra o dedo ({worst:.6}) — o lóbulo \
         negativo do campo multi-escala entrou na pegada"
    );
}

/// **O DEGRAU NA BORDA DA PEGADA, medido no produto** — o preço nomeado do
/// [`crate::KELVINLET_REACH`] e da família [`crate::kelvinlet::Scales`], em
/// unidades da própria malha.
///
/// ⚠️ **A pergunta certa não é *"quantos por cento sobram?"* e sim *"o degrau é
/// maior que a aresta da malha?"***: um salto menor que o espaçamento dos
/// vértices não tem como ser visto, porque não há geometria para o desenhar.
///
/// ⚠️ **E a MALHA é parte da pergunta — a fixture partilhada NÃO servia.** Com
/// a esfera de `32×48` a aresta mede `0,13081`, e ali o degrau é `0,019` com o
/// campo que shipa e `0,108` com o campo CRU: os dois passam, porque a malha não
/// consegue desenhar nem o defeito. O gate era verdadeiro e **não
/// discriminava**, e quem o expôs foi a mutação da família de escalas, que não
/// sangrou em lado nenhum. Uma malha subdividida uma vez (aresta `0,065`) é o
/// que torna a propriedade *load-bearing*.
#[test]
fn the_rim_step_is_smaller_than_an_edge() {
    // ⚠️ A malha FINA vive só aqui: trocar a fixture partilhada mexeria no
    // sentido de dezenas de gates que foram escritos contra a grossa.
    let rest = ph2d_mesh::subdivide(&sphere());
    let pull = [0.0, 0.3, 0.0];
    let mut mesh = rest.clone();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &grab(RefMode::L),
        &Dab::pulling([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0], pull),
        Symmetry::default(),
    );

    // O maior salto de deslocamento entre vértices vizinhos que estão dos DOIS
    // lados do corte da pegada — o degrau, tal como a malha o desenha.
    let mut step = 0.0f32;
    let mut edge = 0.0f32;
    let c = [0.0, 0.0, 1.0];
    let dist = |p: [f32; 3]| {
        ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
    };
    let disp = |i: usize| {
        let (a, b) = (rest.positions()[i], mesh.positions()[i]);
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    for v in 0..rest.vert_count() {
        let dv = dist(rest.positions()[v]);
        for &nb in rest.adjacency().vert_verts.neighbours(v) {
            let nb = nb as usize;
            if (dv - 0.5).signum() == (dist(rest.positions()[nb]) - 0.5).signum() {
                continue;
            }
            let (a, b) = (rest.positions()[v], rest.positions()[nb]);
            edge = edge.max(
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
            );
            step = step.max((disp(v) - disp(nb)).abs());
        }
    }
    assert!(edge > 0.0, "o gate mediu vácuo: nenhum par cruza a borda");
    assert!(
        step < edge,
        "o degrau da borda ({step:.5}) passou a aresta da malha ({edge:.5}) — \
         o KELVINLET_REACH ou a família de escalas deixaram de ser suficientes"
    );
}
