//! **O MAR da cena `=95`** — os gates da fileira de baixo (doc 89, folha 02).
//!
//! ⚠️ **Ficheiro irmão de propósito.** As leis do mar são metade dos gates desta cena — os
//! outros três pares cabem em `..._tests.rs` —, e mantê-las juntas levava aquele ficheiro a
//! **684 linhas** contra o alvo de `600` do repo (o portão executável está em `700`, e um
//! integrador que apendesse ali rebentava-o).
//!
//! O que se afirma aqui, e porquê, está no [`Bug #6`](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md):
//! duas causas independentes fazem a MESMA imagem no ecrã — peças a atravessar a banda sem
//! nunca subirem nem descerem — e as réguas que a cena tinha partilhavam essa assinatura com
//! elas.

use super::tests::{DT, scene};
use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// **QUÃO FUNDO cada boia está**, em decis `[p10, mediana, p90]`.
///
/// ⚠️ **A pose chega em coordenadas de MUNDO e a simulação correu em LOCAIS** — o `finish`
/// desloca a banda para o quadrante dela depois de tudo. Comparar a pose com a superfície
/// sem desfazer esse deslocamento mediria o quadrante, e não a água.
pub(super) fn submersions(p: &[[f32; 2]], band: usize, t: f32) -> [f32; 3] {
    let at = band_at(band);
    let (amp, lambda, speed, _draft, _sub) = sea_authored();
    // A esquerda é a senoide única; a direita é o espectro.
    let waves = if band.is_multiple_of(2) {
        1.0
    } else {
        authored().2
    };
    let mut d: Vec<f32> = p
        .iter()
        .map(|q| {
            let (x, y) = (q[0] - at[0], q[1] - at[1]);
            ph2d_node_force_buoyancy::surface_at(x, t, 0.0, amp, lambda, speed, waves) - y
        })
        .collect();
    d.sort_by(f32::total_cmp);
    let pick = |f: f32| d[((d.len() - 1) as f32 * f) as usize];
    [pick(0.1), pick(0.5), pick(0.9)]
}

/// **AS CRISTAS de uma linha de pontos** — as alturas dos máximos locais.
///
/// ⚠️ **`win` não é cosmético.** As boias vêm em DUAS fileiras e derivam de lado, então há
/// dois pontos quase no mesmo `x` com alturas ligeiramente diferentes: um teste de vizinho
/// imediato chamaria crista a metade deles. Um máximo tem de o ser numa JANELA — e a janela
/// certa é uma fracção da onda mais fina que se quer resolver.
pub(super) fn crest_heights(p: &[[f32; 2]], win: f32) -> Vec<f32> {
    let mut q = p.to_vec();
    q.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut out: Vec<[f32; 2]> = Vec::new();
    for v in &q {
        if q.iter().any(|w| (w[0] - v[0]).abs() <= win && w[1] > v[1]) {
            continue;
        }
        // Um planalto emite vizinhos: fica o mais alto de cada grupo.
        match out.last_mut() {
            Some(l) if (v[0] - l[0]).abs() <= win => {
                if v[1] > l[1] {
                    *l = *v;
                }
            }
            _ => out.push(*v),
        }
    }
    // ⛔ **AS PONTAS DA JANELA SÃO SEMPRE MÁXIMOS**, e foi assim que esta régua nasceu
    // errada: uma senoide PURA — cujas cristas são idênticas por definição — mediu
    // variedade `1,39`, porque a crista de bordo está cortada a meio. *O controlo que a
    // apanhou é o caso em que a resposta certa é ZERO.* Fora a mais externa de cada lado.
    if out.len() > 2 {
        out.remove(0);
        out.pop();
    } else {
        out.clear();
    }
    out.into_iter().map(|c| c[1]).collect()
}

/// `(quantas cristas, espalhamento das alturas em fracção da AMPLITUDE da vaga)`.
///
/// ⚠️ **Normalizado pela amplitude**, e não pela altura média: a média é uma coordenada de
/// mundo (a banda vive a `y ≈ −6`), e dividir por ela mediria o quadrante.
pub(super) fn crest_variety(p: &[[f32; 2]], win: f32, amp: f32) -> (usize, f32) {
    let h = crest_heights(p, win);
    if h.len() < 2 {
        return (h.len(), 0.0);
    }
    let lo = h.iter().copied().fold(f32::MAX, f32::min);
    let hi = h.iter().copied().fold(f32::MIN, f32::max);
    (h.len(), (hi - lo) / amp)
}

/// A superfície AUTORADA, amostrada fino — a VERDADE que as boias tentam desenhar.
pub(super) fn surface_line(t: f32, waves: f32) -> Vec<[f32; 2]> {
    let (amp, lambda, speed, ..) = sea_authored();
    (0..1024)
        .map(|i| {
            let x = -3.5 + 7.0 * i as f32 / 1023.0;
            [
                x,
                ph2d_node_force_buoyancy::surface_at(x, t, 0.0, amp, lambda, speed, waves),
            ]
        })
        .collect()
}

/// Quantos tiques o mar precisa para ASSENTAR — medido, não escolhido: o mergulho inicial
/// ainda domina no tique 150 (submersão mediana `0,29`), e a partir de ~375 a mediana já
/// está a 5% do valor de equilíbrio. `600` dá margem sem esticar o relógio da suíte.
pub(super) const SEA_TICKS: usize = 600;

/// Corre só as duas bandas do mar, e devolve a pose delas em dois instantes.
fn sea_poses(
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    at: &[usize],
) -> Vec<Vec<Vec<[f32; 2]>>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..SEA_TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if at.contains(&k)
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                out[i].push(p.clone());
            }
        }
    }
    out
}

/// O que cada boia FAZ no regime já assentado, por banda: `(excursão vertical mediana,
/// deriva LÍQUIDA da banda)`.
///
/// ⚠️ **As duas juntas, e não uma delas.** A excursão horizontal sozinha não distingue
/// ORBITAR de PARTIR — uma boia que vai e vem meia onda mede o mesmo que uma que anda meia
/// onda e nunca volta.
fn sea_motion(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<(f32, f32)> {
    let mut cook = Cook::new();
    let mut frames: Vec<Vec<Vec<[f32; 2]>>> = vec![Vec::new(); sinks.len()];
    for k in 0..SEA_TICKS + 300 {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
        for (i, &s) in sinks.iter().enumerate() {
            let o = cook.cook(&doc.graph, reg, s, t).expect("coze");
            if k >= SEA_TICKS
                && k % 5 == 0
                && let Some(Column::Vec2(p)) = o[0].as_stream().get("P")
            {
                frames[i].push(p.clone());
            }
        }
    }
    frames
        .iter()
        .map(|f| {
            let n = f[0].len();
            let mut v: Vec<f32> = (0..n)
                .map(|i| {
                    let lo = f.iter().map(|q| q[i][1]).fold(f32::MAX, f32::min);
                    let hi = f.iter().map(|q| q[i][1]).fold(f32::MIN, f32::max);
                    hi - lo
                })
                .collect();
            v.sort_by(f32::total_cmp);
            let mean_x = |q: &Vec<[f32; 2]>| q.iter().map(|p| p[0]).sum::<f32>() / q.len() as f32;
            let net = mean_x(f.last().expect("frames")) - mean_x(&f[0]);
            (v[n / 2], net)
        })
        .collect()
}

/// ⭐⭐⭐ **O PAR 4 — O GATE QUE FALTAVA: as boias CAVALGAM a vaga, não são LEVADAS por ela.**
///
/// ⛔ **É o gate que teria apanhado o defeito que o Enio viu — nas DUAS formas em que ele
/// apareceu.** As duas leem-se igual no ecrã (*«não parece mar, parece partículas ao
/// vento»*) porque têm a mesma assinatura: as peças atravessam a banda **sem nunca subirem
/// nem descerem**.
///
/// | | causa | excursão vertical | deriva em 5 s |
/// |---|---|---|---|
/// | 1.ª versão | **sem gravidade** no grafo: o empuxo lança tudo | — | a média de `y` subia `0,58` por 25 tiques, sem abrandar |
/// | 2.ª versão | **armadilha de cava**: a boia encaixa e viaja com a onda | `0,0056` | `4,92` — `0,98` da velocidade da onda |
/// | hoje | — | `0,38`, que é `0,82` da altura da vaga | `0,067` |
///
/// ⚠️ **Nenhuma das réguas anteriores as via.** Dispersão e distância entre as duas bandas
/// são grandezas que um mar e uma nuvem lançada PARTILHAM; e uma régua só de `y` não vê a
/// segunda, que é horizontal. *Uma cena que mostra uma superfície tem de afirmar as duas
/// coisas que fazem dela uma superfície: que ela SOBE E DESCE, e que ela FICA.*
#[test]
fn the_floats_ride_the_wave_instead_of_being_carried_by_it() {
    let (doc, reg, sinks) = scene();
    let (amp, ..) = sea_authored();
    let height = 2.0 * amp;
    for (i, (bob, net)) in sea_motion(&doc, &reg, &sinks[6..8]).into_iter().enumerate() {
        assert!(
            bob > height * 0.3,
            "banda {}: a boia mediana sobe e desce {bob:.4} de uma vaga de {height:.4} -- \
             este mar esta' PRESO, e preso desliza de lado como uma linha rigida",
            6 + i
        );
        assert!(
            net.abs() < 0.3,
            "banda {}: a banda andou {net:.4} de lado em 5 s -- ela esta' a ser LEVADA",
            6 + i
        );
    }
}

/// ⭐⭐ **O ARRASTO LIMPA O LIMIAR DA ARMADILHA** — aritmética pura sobre os autorados.
///
/// ⚠️ **É a lei, e não a medição.** O gate acima mede o SINTOMA (a banda ficou?); este afirma
/// o MECANISMO, e por isso dispara na cara de quem baixar o arrasto, subir a esbelteza,
/// subir a densidade ou acrescentar camadas — quatro maneiras de reabrir a armadilha, e três
/// delas não parecem ter nada a ver com ela.
#[test]
fn the_drag_clears_the_trapping_threshold() {
    let bar = sea_trap_threshold(authored().2);
    let drag = sea_drag();
    assert!(
        drag > bar,
        "arrasto {drag:.3} contra um limiar de armadilha de {bar:.3}"
    );
    // ⚠️ E o limiar CRESCE com as camadas: a fileira de 4 ondas é a exigente, e é ela que
    // tem de mandar no número. Uma margem medida contra a de 1 onda deixaria a outra presa.
    assert!(
        sea_trap_threshold(4.0) > sea_trap_threshold(1.0),
        "o espectro tinha de tornar a armadilha MAIS facil, nao menos"
    );
    // ⭐⭐ **E o arrasto tem um SEGUNDO dono, que hoje é quem manda: o AMORTECIMENTO.** Uma
    // boia sub-amortecida ressoa e inventa cristas — medido, a `ζ = 0,55` ela desenha `12`
    // onde a superfície tem `8`, e a `ζ = 0,61` desce a `7`. ⚠️ A barra é o degrau MEDIDO
    // entre esses dois, e não uma folga escolhida.
    let zeta = sea_damping_ratio();
    assert!(
        zeta > 0.58,
        "amortecimento {zeta:.4}: abaixo de ~0,58 a boia RESSOA e inventa cristas"
    );
}

/// ⭐ **E as boias estão NA ÁGUA** — nem a voar por cima, nem afundadas.
///
/// ⛔ **A afirmação que eu tinha aqui era mais forte e estava ERRADA.** Ela dizia que a
/// mediana da submersão bate o equilíbrio estático `(gravidade/densidade) · calado`, e batia
/// — a `0,8%`. Só que ela **só batia porque as boias estavam PRESAS**: um corpo encaixado na
/// cava não se mexe, logo assenta no equilíbrio estático. Assim que ele passa a cavalgar a
/// vaga ele é FORÇADO, e o ponto dele deixa de ser o estático (medido: `0,29` contra os
/// `0,167` da conta). *Um gate que só passa quando a cena está morta é um gate a favor da
/// cena morta* — e este passou por cima do defeito que o Enio viu.
#[test]
fn the_floats_are_in_the_water() {
    let (doc, reg, sinks) = scene();
    let (amp, _, _, draft, _) = sea_authored();
    let poses = sea_poses(&doc, &reg, &sinks[6..8], &[SEA_TICKS - 1]);
    let t = (SEA_TICKS - 1) as f32 * DT as f32;
    for (i, p) in poses.iter().enumerate() {
        let d = submersions(&p[0], 6 + i, t);
        // ⚠️ **A ESCALA da barra MUDOU em 2026-08-25, e a mudança é a cura do Bug #7.**
        // Enquanto o calado era `0,5` contra uma vaga de `0,47`, «a boia está dentro de água»
        // media-se em CALADOS. Hoje o calado é `0,20` — **mais pequeno que a vaga** —, e uma
        // crista de `0,47` cobre por completo uma boia de `0,20`: é o que uma rolha faz, e uma
        // barra em calados chamaria defeito a isso. A escala passa a ser `calado + vaga`, que
        // são as duas únicas distâncias em jogo.
        // Medido: banda 7 `[+0,031 · 0,305 · 0,516]`, banda 6 `[+0,027 · 0,107 · 0,206]`.
        let deepest = draft + 2.0 * amp;
        assert!(
            d[0] > -0.5 * draft,
            "banda {}: o decil de cima esta' no AR ({:.4}, calado {draft:.4})",
            6 + i,
            d[0]
        );
        assert!(
            d[2] < deepest,
            "banda {}: o decil de baixo AFUNDOU alem do que a vaga o pode enterrar \
             ({:.4} contra {deepest:.4} = calado + vaga)",
            6 + i,
            d[2]
        );
        assert!(
            d[1] > 0.0 && d[1] < 2.0 * amp,
            "banda {}: a boia MEDIANA tinha de estar dentro de agua e nao afundada ({:.4})",
            6 + i,
            d[1]
        );
    }
}

/// ⭐⭐⭐ **O GATE DO [Bug #7]: o ESPECTRO vê-se no que as boias DESENHAM.**
///
/// ⛔ **É o report do Enio virado número** — *«no 8 as cristas não parecem diferentes»*. Nenhum
/// gate desta cena o podia ver: todos mediam se as boias **bóiam** (excursão, deriva,
/// submersão) e nenhum media a **FORMA** que a fileira delas desenha.
///
/// ⚠️ **Mede as DUAS pontas, e a diferença é o defeito**: a variedade de alturas de crista
/// que a superfície TEM, e a que as boias REPRODUZEM. Antes da cura: superfície `1,94`,
/// boias **`0,0002`** — elas apagavam ~100% da estrutura. Depois: **`0,59`**.
///
/// ⚠️ **A CONTAGEM de cristas é a segunda régua, e sem ela a primeira mente:** uma boia
/// sub-amortecida RESSOA e inventa cristas, o que INFLA a variedade sem desenhar o mar
/// (medido a arrasto `12`: variedade `2,77` com **23** cristas onde a superfície tem `8`).
/// *Uma régua de variedade sozinha premeia o ruído.*
///
/// ⚠️ E os dois **CONTROLOS** vivem aqui de propósito: a senoide pura tem de dar `0` nos dois
/// lados (as cristas de um seno são idênticas **por definição** — foi este controlo que
/// apanhou a régua a contar as pontas da janela como máximos, e a dar `1,39` onde a resposta
/// certa é `0`).
#[test]
fn the_spectrum_is_visible_in_what_the_floats_draw() {
    let (doc, reg, sinks) = scene();
    let (amp, lambda, ..) = sea_authored();
    let t = (SEA_TICKS - 1) as f32 * DT as f32;
    let poses = sea_poses(&doc, &reg, &sinks[6..8], &[SEA_TICKS - 1]);
    let win = |w: f32| ph2d_node_force_buoyancy::finest_wavelength(lambda, w) / 3.0;

    // CONTROLO — a senoide pura, dos dois lados.
    let (_, plain_surface) = crest_variety(&surface_line(t, 1.0), win(1.0), amp);
    let (_, plain_floats) = crest_variety(&poses[0][0], win(1.0), amp);
    assert!(
        plain_surface < 0.05,
        "CONTROLE: as cristas de um seno sao identicas por definicao ({plain_surface:.4})"
    );
    assert!(
        plain_floats < 0.05,
        "CONTROLE: a fileira de UMA onda tinha de desenhar cristas iguais ({plain_floats:.4})"
    );

    let waves = authored().2;
    let (sn, sv) = crest_variety(&surface_line(t, waves), win(waves), amp);
    let (fnum, fv) = crest_variety(&poses[1][0], win(waves), amp);
    assert!(
        sv > 1.5,
        "CONTROLE: a superficie de {waves} ondas TEM estrutura ({sv:.4})"
    );
    assert!(
        fv > 0.25,
        "as boias desenham {fv:.4} de uma superficie com {sv:.4} -- o espectro nao se ve'"
    );
    // ⚠️ A contagem: nem mudas (a boia nao segue) nem a inventar (a boia ressoa).
    assert!(
        fnum * 2 >= sn && fnum <= sn * 2,
        "as boias desenham {fnum} cristas onde a superficie tem {sn} -- \
         a menos e' passa-baixo, a mais e' RESSONANCIA"
    );
}

/// ⭐ **AS BOIAS RESOLVEM A ONDA MAIS FINA** — Nyquist, sobre os números autorados.
///
/// ⚠️ **A onda mais fina não é escolhida por esta cena**: ela cai da razão entre camadas e do
/// tecto de camadas, que são decisões do `force.buoyancy` — daí `finest_wavelength`. Com as
/// `48` colunas da primeira versão havia **1,96 boias por período**, abaixo dos dois que
/// separam «uma onda amostrada» de «ruído».
///
/// ⛔ Este gate é aritmética pura sobre constantes, e é de propósito: ele dispara quando
/// alguém encolher a contagem de boias, alargar a banda **ou** subir o tecto de camadas —
/// as três estragam a mesma coisa, e só uma delas mora neste arquivo.
#[test]
fn the_floats_resolve_the_finest_wave() {
    let (_, lambda, _, _, _) = sea_authored();
    let finest = ph2d_node_force_buoyancy::finest_wavelength(lambda, authored().2);
    let per = finest / float_spacing();
    assert!(
        per >= 4.0,
        "so' {per:.2} boias na onda mais fina ({finest:.4}) -- ela sai como ruido"
    );
}

/// ⭐ **As duas superfícies não são a mesma**, e a diferença é a que o olho tem de apanhar.
///
/// ⚠️ **A afirmação sobre a FORMA das cristas vive no gate do crate**
/// (`the_spectrum_breaks_the_single_wavelength`, que mede a distância entre cristas
/// vizinhas). Aqui o que se afirma é que a fileira da direita **desenha outra água**.
#[test]
fn the_two_seas_are_not_the_same_sea() {
    let (doc, reg, sinks) = scene();
    let poses = sea_poses(&doc, &reg, &sinks[6..8], &[SEA_TICKS - 1]);
    assert_eq!(poses[0][0].len(), poses[1][0].len(), "a mesma contagem");
    let apart = poses[0][0]
        .iter()
        .zip(&poses[1][0])
        .map(|(a, b)| (a[1] - b[1]).abs())
        .fold(0.0_f32, f32::max);
    let (amp, ..) = sea_authored();
    assert!(
        apart > amp * 0.5,
        "as duas superficies quase coincidiram ({apart:.4} contra uma amplitude de {amp:.4})"
    );
}
