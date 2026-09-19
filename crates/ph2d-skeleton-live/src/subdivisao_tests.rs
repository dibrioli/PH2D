//! Os gates da subdivisão do bind. ⚠️ A lei é **pura** (uma forma e um passo), logo mede-se sem
//! mundo nenhum — e é por isso que ela vive aqui e não dentro do `bind`.

use super::{DIVISOES_POR_OSSO, VERTICES_MAX, alvo_dos_eixos, comprimento, subdivide};


/// ⭐⭐ **A RÉGUA DESTES GATES — e ela é do GATE, não da lei.**
///
/// A lei guarda-se pela CAUSA (o recuo da quina, `O(1)`); estes gates medem a CONSEQUÊNCIA (o
/// desenho), que é a afirmação que interessa e custa `O(n²)`. *Uma régua cara num gate é barata;
/// dentro de um gesto ela pendura a suíte* — foi o que a 1.ª redacção desta wave fez.
///
/// ⚠️ **A densidade é a do RESULTADO e não a do segmento:** amostrar `N` por segmento afina a
/// polilinha à medida que se subdivide, e a comparação lê «mexeu» sobre um corte exacto (medido:
/// `1,3e-2` na elipse, puro artefacto).
fn densa(p: &VecPath) -> Vec<[f64; 2]> {
    const POR_UNIDADE: f64 = 4000.0;
    let cozido = p.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((v, fechado)) = cozido.contour(c) else { continue };
        let n = v.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let (a, b) = (&v[i], &v[(i + 1) % n]);
            let amostras = ((comprimento(a, b) * POR_UNIDADE) as usize).clamp(8, 200_000);
            for k in 0..amostras {
                let t = k as f64 / amostras as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0 * a.anchor[0] + w1 * a.out_handle[0] + w2 * b.in_handle[0] + w3 * b.anchor[0],
                    w0 * a.anchor[1] + w1 * a.out_handle[1] + w2 * b.in_handle[1] + w3 * b.anchor[1],
                ]);
            }
        }
    }
    out
}

/// A maior distância de um ponto de `a` à POLILINHA de `b`.
///
/// ⚠️ **A barra destes gates sai da resolução DESTA régua**: com `4 000` amostras por unidade, a
/// flecha de uma corda sobre um arco de raio `0,5` é `(2,5e-4)²/(8·0,5) ≈ 1,6e-8`. *Uma barra
/// abaixo da resolução da própria régua reprova sobre geometria exacta* — foi o que a 1.ª
/// redacção fez, com `1e-6` sobre uma régua que resolvia `5,5e-6`.
fn afastamento(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    // ⚠️ **Os pontos de CONSULTA são subamostrados e os de REFERÊNCIA não.** A grandeza é «quanto o
    // desenho novo se afasta do velho», e um máximo sobre `2 000` sítios bem espalhados é o mesmo
    // que sobre `100 000` — enquanto o lado contra o qual se mede continua denso. *Sem isto o gate
    // é `O(n²)` sobre `n` grande e passa dos 60 s que esta casa dá a um teste.*
    const CONSULTAS: usize = 2_000;
    let salto = a.len().div_ceil(CONSULTAS).max(1);
    let d2 = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        let (vx, vy) = (r[0] - q[0], r[1] - q[1]);
        let l2 = vx * vx + vy * vy;
        let t = if l2 <= f64::EPSILON { 0.0 } else { (((p[0] - q[0]) * vx + (p[1] - q[1]) * vy) / l2).clamp(0.0, 1.0) };
        let (x, y) = (q[0] + t * vx - p[0], q[1] + t * vy - p[1]);
        x * x + y * y
    };
    a.iter()
        .step_by(salto)
        .map(|p| (0..b.len()).map(|i| d2(*p, b[i], b[(i + 1) % b.len()])).fold(f64::INFINITY, f64::min).sqrt())
        .fold(0.0, f64::max)
}

/// A barra, derivada da resolução da régua acima (com folga de `6×`).
const EXACTO: f64 = 1e-7;

use ph2d_vec_scene::{ShapeKind, VecPath, cook};

/// A barra da cena do dono.
fn barra() -> VecPath {
    cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5])
}

/// ⭐⭐⭐ **SUBDIVIDIR NÃO MEXE NO DESENHO** — a promessa de que esta wave depende inteira.
///
/// ⚠️ **A régua é PONTO-A-CURVA e não índice-a-índice:** a 1.ª medição desta wave comparou as duas
/// formas amostrando `N` pontos por SEGMENTO, e a elipse leu `1,3e-2` — puro artefacto, porque
/// subdividir multiplica os segmentos e afina a polilinha. *Uma régua cuja resolução segue o que
/// ela está a medir não mede nada.*
#[test]
fn subdividir_nao_mexe_no_desenho() {
    for (nome, forma, raios) in [
        ("RoundRect", ShapeKind::RoundRect, vec![0.5]),
        ("Ellipse", ShapeKind::Ellipse, vec![]),
        ("Star", ShapeKind::Star, vec![]),
        ("Rectangle", ShapeKind::Rectangle, vec![]),
    ] {
        let base = cook(forma, [-8.5, 2.0], [-1.5, 3.0], &raios);
        let mut sub = base.clone();
        let cortes = subdivide(&mut sub, 0.71, VERTICES_MAX);
        assert!(
            cortes > 0,
            "{nome}: nenhum corte — a fixtura nao contem o fenomeno"
        );
        assert!(
            sub.verts.len() > base.verts.len(),
            "{nome}: os cortes nao deixaram vertices"
        );
        let erro = afastamento(&densa(&sub), &densa(&base));
        assert!(
            erro < EXACTO,
            "{nome}: subdividir moveu o desenho {erro:.3e} ({} -> {} vertices)",
            base.verts.len(),
            sub.verts.len()
        );
    }
}

/// ⭐⭐⭐ **A QUINA VIVA É PROTEGIDA, e a protecção é MEDIDA e não presumida.**
///
/// ⛔⛔ O recuo de uma quina é clampado a **metade da menor corda vizinha**, logo partir o segmento
/// ao lado dela encolhe o recuo e a quina muda — medido `2,055e-2` sem a guarda. ⇒ o corte que
/// mexe no desenho é **revertido**.
///
/// ⚠️ **As duas metades são obrigatórias:** sem a segunda, uma guarda que recusasse TODO corte
/// passaria — e a forma ficaria com os oito nós de sempre, que é o report do dono de volta.
#[test]
fn uma_quina_viva_nao_e_estragada_e_o_resto_da_forma_e_subdividido() {
    let mut viva = cook(ShapeKind::Rectangle, [-8.5, 2.0], [-1.5, 3.0], &[]);
    for v in &mut viva.verts {
        v.corner_radius = 0.2;
    }
    assert!(
        viva.has_live_corner(),
        "a fixtura nao tem quina viva — ela nao contem o fenomeno"
    );
    let antes = densa(&viva);
    let mut sub = viva.clone();
    let cortes = subdivide(&mut sub, 0.71, VERTICES_MAX);
    let erro = afastamento(&densa(&sub), &antes);
    assert!(
        erro < EXACTO,
        "a quina viva foi estragada: o desenho moveu {erro:.3e}"
    );
    assert!(
        cortes > 0 && sub.verts.len() > viva.verts.len(),
        "a guarda recusou TODOS os cortes ({cortes}) — a forma fica com {} nos, que e' o estado que \
         o dono reprovou",
        sub.verts.len()
    );
}

/// ⭐⭐ **O PASSO SAI DO OSSO MAIS CURTO** — e `None` quando não há osso com comprimento.
#[test]
fn o_passo_sai_do_osso_mais_curto() {
    let h = |a: [f64; 2], b: [f64; 2]| ph2d_skin_weights::Handle { a, b };
    let eixos = vec![
        h([0.0, 0.0], [3.0, 0.0]),
        h([3.0, 0.0], [4.0, 0.0]),
        h([4.0, 0.0], [9.0, 0.0]),
    ];
    let alvo = alvo_dos_eixos(&eixos).expect("ha' osso");
    assert!(
        (alvo - 1.0 / DIVISOES_POR_OSSO).abs() < 1e-12,
        "o passo saiu de {alvo} — ele tem de vir do osso mais CURTO (1,0), nao do primeiro nem do \
         mais longo"
    );
    // ⛔ Um osso de comprimento ZERO nao pode fixar o passo: o alvo iria a zero e a subdivisao
    // nunca parava.
    let com_zero = vec![h([0.0, 0.0], [0.0, 0.0]), h([0.0, 0.0], [3.0, 0.0])];
    let alvo = alvo_dos_eixos(&com_zero).expect("o osso vivo fixa o passo");
    assert!((alvo - 3.0 / DIVISOES_POR_OSSO).abs() < 1e-12, "{alvo}");
    assert_eq!(alvo_dos_eixos(&[]), None, "sem eixos nao ha' passo");
    assert_eq!(
        alvo_dos_eixos(&[h([0.0, 0.0], [0.0, 0.0])]),
        None,
        "so' um osso degenerado nao fixa passo nenhum"
    );
}

/// ⛔ **O TECTO SEGURA** — e o que ele impede é o caso degenerado, não o caminho normal.
#[test]
fn o_tecto_segura_e_a_barra_da_cena_nao_chega_perto_dele() {
    let mut louca = barra();
    // Um osso minusculo sobre a forma inteira — o caso que o tecto existe para impedir.
    let cortes = subdivide(&mut louca, 1e-4, VERTICES_MAX);
    assert!(
        louca.verts.len() <= VERTICES_MAX,
        "o tecto nao segurou: {} vertices",
        louca.verts.len()
    );
    assert!(cortes > 0, "o caso degenerado nao cortou nada");

    // E o caminho NORMAL fica muito abaixo dele.
    let mut normal = barra();
    subdivide(&mut normal, 2.133 / DIVISOES_POR_OSSO, VERTICES_MAX);
    assert!(
        normal.verts.len() < VERTICES_MAX / 32,
        "a barra da cena saiu com {} vertices — o tecto deixou de ser generoso",
        normal.verts.len()
    );
}

/// ⭐⭐ **SUBDIVIDIR DUAS VEZES NÃO CRESCE** — o bind pode correr outra vez sobre o que ele próprio
/// escreveu (o recook devolve a forma subdividida à cena).
#[test]
fn subdividir_duas_vezes_nao_cresce() {
    let mut p = barra();
    subdivide(&mut p, 0.71, VERTICES_MAX);
    let n = p.verts.len();
    let cortes = subdivide(&mut p, 0.71, VERTICES_MAX);
    assert_eq!(cortes, 0, "a 2.a passagem cortou {cortes} vezes");
    assert_eq!(p.verts.len(), n, "a 2.a passagem mudou a contagem");
}

/// ⭐ **NENHUM SEGMENTO FICA ACIMA DO PASSO** — a promessa que o nome da função faz.
///
/// ⚠️⚠️ **O ARCO é medido AQUI, por achatamento, e nunca pela [`comprimento`] da lei** — e a
/// distinção nasceu de uma mutação SOBREVIVENTE. Trocar o polígono de controlo pela CORDA deixava
/// este gate verde, porque ele media com a mesma função que estava a julgar: *uma régua que é a
/// própria lei aprova qualquer lei.*
///
/// ⛔ E a fixtura leva uma ELIPSE: numa aresta RECTA a corda e o polígono de controlo são o mesmo
/// número, logo o rectângulo não distingue as duas.
#[test]
fn nenhum_segmento_fica_acima_do_passo() {
    /// O arco de um segmento, por 256 cordas — independente da lei.
    fn arco(a: &ph2d_vec_scene::VecVertex, b: &ph2d_vec_scene::VecVertex) -> f64 {
        const N: usize = 256;
        let mut s = 0.0;
        let mut ant = a.anchor;
        for k in 1..=N {
            let t = k as f64 / N as f64;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            let q = [
                w0 * a.anchor[0] + w1 * a.out_handle[0] + w2 * b.in_handle[0] + w3 * b.anchor[0],
                w0 * a.anchor[1] + w1 * a.out_handle[1] + w2 * b.in_handle[1] + w3 * b.anchor[1],
            ];
            s += (q[0] - ant[0]).hypot(q[1] - ant[1]);
            ant = q;
        }
        s
    }
    for (nome, forma, raios) in [
        ("RoundRect", ShapeKind::RoundRect, vec![0.5]),
        ("Ellipse", ShapeKind::Ellipse, vec![]),
        ("Star", ShapeKind::Star, vec![]),
    ] {
        let alvo = 0.71;
        let mut p = cook(forma, [-8.5, 2.0], [-1.5, 3.0], &raios);
        subdivide(&mut p, alvo, VERTICES_MAX);
        let mut curvo = false;
        for c in 0..p.contour_count() {
            let Some((v, fechado)) = p.contour(c) else {
                continue;
            };
            let n = v.len();
            let ultimo = if fechado { n } else { n.saturating_sub(1) };
            for i in 0..ultimo {
                let (a, b) = (&v[i], &v[(i + 1) % n]);
                let (l, corda) = (
                    arco(a, b),
                    (a.anchor[0] - b.anchor[0]).hypot(a.anchor[1] - b.anchor[1]),
                );
                if l > corda * 1.01 {
                    curvo = true;
                }
                assert!(
                    l <= alvo + 1e-9,
                    "{nome}: o segmento {i} ficou com ARCO {l} > {alvo}"
                );
            }
        }
        // ⭐ O CONTROLO da fixtura: alguma coisa nela tem de ser CURVA, senão a corda e o polígono
        // de controlo coincidem e a mutação da [`comprimento`] fica invisível.
        if nome == "Ellipse" {
            assert!(curvo, "a elipse saiu sem um unico segmento curvo");
        }
    }
}

/// ⛔ **Um passo impossível não corta nada** — e não pendura.
#[test]
fn um_passo_impossivel_nao_corta_nada() {
    for alvo in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
        let mut p = barra();
        let n = p.verts.len();
        assert_eq!(subdivide(&mut p, alvo, VERTICES_MAX), 0, "alvo {alvo}");
        assert_eq!(p.verts.len(), n, "alvo {alvo}");
    }
}

use crate::barra_da_cena_tests_support::{PPM, barra_da_cena, barra_da_cena_com, forma};

/// ⭐⭐⭐ **PRENDER A BARRA DEIXA OS PONTOS À VISTA** — a ordem do dono, medida no fim da corrente.
///
/// *«Sem saber onde os pontos estão não fica legal. Melhor criar a subdivisão visível logo na
/// associação com os ossos»* (2026-09-19). ⚠️ **A régua é a do INDICADOR** — a mesma porta que
/// pinta os pontos coloridos —, e não a contagem de vértices: *o que o dono pediu foi VER, e ver é
/// o que aquela porta responde.*
///
/// ⛔ **O piso é `30` e não `9`:** a barra tinha oito nós, os oito nas duas pontas, e um único a
/// mais não muda o que ele reportou.
#[test]
fn prender_a_barra_deixa_os_pontos_a_vista() {
    let (sim_vazio, scene, map, id, _) = barra_da_cena();
    let antes = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("a barra")
        .verts
        .len();
    assert_eq!(antes, 8, "a barra da cena deixou de ter oito nos");
    drop(sim_vazio);

    let (sim, _scene, map2, id2, ossos) = barra_da_cena();
    let _ = (&map, id);
    let pontos = crate::peso_a_mao::pontos_de_peso(&sim, forma(&map2, id2), ossos[1], PPM);
    assert!(
        pontos.len() >= 30,
        "depois do bind o indicador mostra {} ponto(s) — o dono reportou exactamente este estado",
        pontos.len()
    );

    // ⭐ E eles são ÚTEIS: ao longo da barra o peso do osso do meio tem de VARIAR. *Trinta pontos
    // todos com o mesmo peso seriam trinta pontos a dizer a mesma coisa.*
    let (lo, hi) = pontos.iter().fold((f64::MAX, f64::MIN), |(l, h), p| {
        (l.min(p.peso), h.max(p.peso))
    });
    assert!(
        hi - lo > 0.5,
        "o peso do osso do meio varia so' {:.4} ao longo da barra ({lo:.4}..{hi:.4}) — os pontos \
         novos nao dizem nada",
        hi - lo
    );
}

/// ⭐⭐⭐ **E O DESENHO NÃO SE MEXE AO SER PRESO** — a promessa de que tudo isto depende.
///
/// ⚠️ **A régua compara o desenho DEPOIS do recook contra a forma original**, e não os vértices:
/// a contagem muda de propósito. *Comparar contagens leria «mudou» sobre a wave a funcionar.*
#[test]
fn prender_nao_mexe_no_desenho_da_barra() {
    let original = cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5]);
    let (sim, mut scene, _map, id, _ossos) = barra_da_cena();
    crate::skin_live::recook(&sim, &mut scene);
    let depois = scene.paths().iter().find(|p| p.id == id).expect("a barra");
    assert!(
        depois.verts.len() >= 30,
        "o recook nao devolveu a forma subdividida a` cena ({} nos) — o artista nao os VE^",
        depois.verts.len()
    );
    let erro = afastamento(&densa(depois), &densa(&original));
    assert!(
        erro < EXACTO,
        "prender moveu o desenho {erro:.3e} — a barra mudou de forma so' por ser presa"
    );
}

use ph2d_ecs::Transform;

/// ⭐⭐⭐ **A SUBDIVISÃO APROXIMA O DESENHO DA VERDADE — `26×`.** É o produto desta wave, e não um
/// efeito colateral dela.
///
/// ⚠️⚠️ **Quem o encontrou foi uma FOTOGRAFIA, e não um gate:** ao refotografar a cena do smoke, a
/// barra laranja apareceu **dobrada** onde antes estava quase recta. A primeira leitura possível era
/// *«a wave estragou a cena»* — e a medição diz o contrário: a barra de oito nós **não conseguia
/// seguir os ossos**, e o que a foto mostra é ela a passar a segui-los.
///
/// A verdade é a mesma cadeia sobre a mesma forma com o passo `48×` mais fino:
///
/// | a barra, na pose em S da cena | nós | erro contra a verdade |
/// |---|---|---|
/// | GROSSA (o caminho de antes) | `8` | **`0,3767`** |
/// | do PRODUTO (`osso / 3`) | `34` | **`0,0142`** |
///
/// ⇒ `38 %` da espessura da barra contra `1,4 %`. ⛔ **E isto é com a lei da CURVA ligada nos dois
/// lados:** a F30 corrige o interior de um segmento, e não inventa os pontos de controlo que a
/// forma não tem.
#[test]
fn a_subdivisao_aproxima_o_desenho_da_verdade() {
    let desenho = |passo: Option<f64>| {
        let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(false);
        if let Some(a) = passo {
            let p = scene
                .paths_mut()
                .iter_mut()
                .find(|p| p.id == id)
                .expect("a barra");
            subdivide(p, a, VERTICES_MAX * 4);
        }
        crate::skin_live::bind_com(&mut sim, &scene, &map, &[id], None, false);
        // A pose em S da cena do dono — sem ela as três respostas coincidem ao bit.
        sim.world_mut()
            .get_mut::<Transform>(ossos[1])
            .expect("pose")
            .rotation = -0.45;
        sim.world_mut()
            .get_mut::<Transform>(ossos[2])
            .expect("pose")
            .rotation = 0.45;
        crate::skin_live::recook(&sim, &mut scene);
        scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .expect("a barra")
            .clone()
    };
    const OSSO: f64 = 6.4 / 3.0;
    let grossa = desenho(None);
    let produto = desenho(Some(OSSO / DIVISOES_POR_OSSO));
    let verdade = desenho(Some(OSSO / 48.0));
    let erro_grossa = crate::test_support::pior_desvio_do_desenho(&grossa, &verdade);
    let erro_produto = crate::test_support::pior_desvio_do_desenho(&produto, &verdade);
    eprintln!(
        "[subdivisao] contra a verdade ({} nos): grossa ({} nos) = {erro_grossa:.6} · produto ({} \
         nos) = {erro_produto:.6}",
        verdade.verts.len(),
        grossa.verts.len(),
        produto.verts.len()
    );
    // ⭐ O CONTROLO vem primeiro: sem um erro grande do lado grosso, a linha de baixo é trivial.
    assert!(
        erro_grossa > 0.2,
        "a barra GROSSA ja' segue os ossos ({erro_grossa}) — a fixtura deixou de conter o fenomeno"
    );
    assert!(
        erro_produto < erro_grossa / 10.0,
        "a subdivisao aproximou so' {:.1}x ({erro_produto} contra {erro_grossa})",
        erro_grossa / erro_produto
    );
}

/// ⛔⛔ **UMA FORMA COM EFEITOS NÃO É SUBDIVIDIDA — limitação DECLARADA, e gateada.**
///
/// A saída de um efeito é função do **contorno inteiro** (aparar, ondular, repetir), e esta wave
/// não mediu o que acrescentar pontos de controlo lhe faz. ⇒ ali o `Bind` deixa a forma como está.
///
/// ⚠️ **Ela nasceu de uma mutação SOBREVIVENTE:** a limitação estava escrita no cabeçalho e no
/// código, e apagá-la não reprovava nada. *Uma cerca sem gate é um comentário com sintaxe de
/// código.*
///
/// ⭐ **As duas metades:** sem a segunda, um `subdivide` que recusasse TUDO passaria aqui.
#[test]
fn uma_forma_com_efeitos_nao_e_subdividida() {
    let mut com_efeito = barra();
    com_efeito
        .effects
        .push(ph2d_vec_scene::effect::FxEntry::new(
            ph2d_vec_scene::effect::PathEffect::Trim(ph2d_vec_scene::fx_trim::TrimSpec::default()),
        ));
    let n = com_efeito.verts.len();
    assert_eq!(
        subdivide(&mut com_efeito, 0.71, VERTICES_MAX),
        0,
        "uma forma com efeitos foi subdividida"
    );
    assert_eq!(com_efeito.verts.len(), n, "a contagem mudou");

    // ⭐ O CONTROLO: a MESMA forma sem o efeito é subdividida.
    let mut sem = barra();
    assert!(
        subdivide(&mut sem, 0.71, VERTICES_MAX) > 0,
        "a forma sem efeito tambem nao foi subdividida — este gate passaria com a lei apagada"
    );
}
