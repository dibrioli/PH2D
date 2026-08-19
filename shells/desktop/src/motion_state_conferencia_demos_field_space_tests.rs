//! Os gates da cena `=60` — o espaço do campo.
//!
//! ⚠️ **Dois destes gates existem porque o SMOKE reprovou a v1** (*"não tem nada girado nem
//! na diagonal"*), e são os que um gate de comportamento não apanharia: eles medem se a cena
//! é **legível**, não se ela computa. *Um gate mede o que a cena produz; só o olho mede o que
//! ela mostra — mas depois de o olho falhar, o número que ele achou vira gate.*

use super::*;
use ph2d_eval_motion::MotionCookPump;

/// O TAMANHO de cada ponto de uma banda — que nesta cena **é** o campo.
fn band_field(band: usize) -> Vec<f32> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_field_space_demo_document(&mut doc, &reg).expect("a cena monta");

    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_scoped(
        &doc.graph,
        &reg,
        std::slice::from_ref(&sinks[band]),
        0,
        |k| k as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &Default::default(),
    );
    pump.instances.iter().map(|i| i.size[0]).collect()
}

/// A COR de cada ponto de uma banda, como um só número: a luminância do `tint`.
///
/// ⚠️ Na v3 é a cor que carrega o campo, então é ela que precisa de gate. Ler o `tint`
/// **no instance** (e não o `t` que entra no ramp) é o que prova que a cor atravessa o
/// lowering — que é onde a v2 teria falhado se a cor tivesse sido a resposta desde o início.
fn band_tint(band: usize) -> Vec<f32> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_field_space_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_scoped(
        &doc.graph,
        &reg,
        std::slice::from_ref(&sinks[band]),
        0,
        |k| k as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &Default::default(),
    );
    pump.instances
        .iter()
        .map(|i| 0.2126 * i.tint[0] + 0.7152 * i.tint[1] + 0.0722 * i.tint[2])
        .collect()
}

/// O pior `|Δ|` entre dois campos.
fn worst(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()))
}

/// `(menor, maior)` de um campo.
fn range(v: &[f32]) -> (f32, f32) {
    v.iter()
        .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)))
}

/// **AS QUATRO BANDAS EXISTEM, e a mensagem tem quatro rótulos.**
#[test]
fn the_scene_builds_the_four_bands_its_message_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_field_space_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas");
    assert_eq!(band_labels().count(), 4, "quatro rotulos");
}

/// **OS PONTOS NUNCA SE TOCAM — a cena é LEGÍVEL.**
///
/// ⚠️ **Este gate é o smoke reprovado, escrito como número.** Na v1 o campo empurrava a
/// POSIÇÃO e a coluna `size` ficava ausente — e ausente **não é «pequeno», é `1,0`** (o
/// `SIZE_IDENTITY` do shell). Com um vão de `0,32`, cada quadrado cobria **3,1×** o vizinho:
/// o bloco era uma placa sólida e não havia o que ver. Nenhum gate de comportamento notou,
/// porque o cook estava certo.
#[test]
fn the_dots_never_touch_so_the_field_is_readable() {
    for (i, _) in band_labels() {
        let (lo, hi) = range(&band_field(i));
        assert!(
            hi < gap(),
            "banda {}: o maior ponto mede {hi} contra um vao de {} -- eles se sobrepoem",
            i + 1,
            gap()
        );
        // ⚠️ **O PISO é o que a v2 não conseguiu manter.** Lá o menor ponto media 3,4 px
        // porque o tamanho carregava o campo sozinho; aqui quem o carrega é a cor, e o
        // tamanho só precisa de nunca sumir. `0,12 · (220/21 / 0,26) ≈ 4,9 px`.
        assert!(
            lo > 0.12,
            "banda {}: o menor ponto mede {lo} -- na tela isto e' menos de 5 px",
            i + 1
        );
    }
}

/// **A COR CARREGA O CAMPO — e ela chega ao instance.**
///
/// ⚠️ Este gate é a v3 escrita como número. A v2 falhou porque o único canal do campo era o
/// tamanho, e um ponto de 3,4 px não desenha imagem nenhuma. O contraste que se exige agora
/// é de **luminância**, que se lê num ponto de 8 px — e ele é medido **no `tint` do
/// instance**, depois do lowering, não no valor que entra no `motion.color_ramp`.
#[test]
fn the_colour_carries_the_field_all_the_way_to_the_instance() {
    for (i, _) in band_labels() {
        let t = band_tint(i);
        let (lo, hi) = range(&t);
        assert!(
            hi - lo > 0.25,
            "banda {}: a luminancia varia so' {:.3} ({lo:.3}..{hi:.3}) -- o campo nao se le' \
             na cor, e o tamanho sozinho ja' falhou no smoke",
            i + 1,
            hi - lo
        );
    }
    // E as bandas têm de diferir NA COR, não só no tamanho — é a cor que o olho lê.
    for (i, j) in [(0, 1), (0, 2), (2, 3)] {
        let d = worst(&band_tint(i), &band_tint(j));
        assert!(
            d > 0.1,
            "as bandas {} e {} tem de diferir na COR, e diferem {d}",
            i + 1,
            j + 1
        );
    }
}

/// **HÁ MANCHAS QUE CHEGUEM PARA UMA ROTAÇÃO SE VER.**
///
/// ⚠️ O segundo achado do smoke: a v1 punha **2,5** células de ruído no bloco inteiro, e duas
/// manchas não mostram rotação — não há padrão para virar. O gate conta quantas vezes o campo
/// atravessa a própria média ao longo da fileira central, que é quantas vezes ele sobe e desce.
#[test]
fn the_block_holds_enough_blobs_for_a_rotation_to_read() {
    let f = band_field(0);
    let side = side();
    let mid = side / 2;
    let row: Vec<f32> = (0..side).map(|c| f[mid * side + c]).collect();
    let mean = row.iter().sum::<f32>() / row.len() as f32;
    let crossings = row
        .windows(2)
        .filter(|w| (w[0] - mean) * (w[1] - mean) < 0.0)
        .count();
    assert!(
        crossings >= 3,
        "a fileira central cruza a media {crossings} vezes -- com menos de 3 nao ha' padrao \
         suficiente para uma rotacao se ver (a v1 tinha 2,5 celulas no bloco INTEIRO)"
    );
    eprintln!("[=60] a fileira central cruza a media {crossings} vezes");
}

/// **AS QUATRO SÃO DIFERENTES ENTRE SI, e nenhuma é «mais forte».**
///
/// ⚠️ As duas metades, e a segunda é a que custa: *"as bandas diferem"* ficaria verde numa
/// cena em que o espaço mudasse a AMPLITUDE, e aí o artista leria «aquele tem pontos maiores»
/// em vez de «o campo virou».
#[test]
fn every_band_is_a_different_field_and_none_is_merely_louder() {
    let bands: Vec<Vec<f32>> = (0..4).map(band_field).collect();
    for (i, a) in bands.iter().enumerate() {
        for (j, b) in bands.iter().enumerate().skip(i + 1) {
            let d = worst(a, b);
            assert!(
                d > 0.02,
                "as bandas {} e {} tem de amostrar o campo em sitios diferentes, e diferem {d}",
                i + 1,
                j + 1
            );
        }
    }
    let spans: Vec<f32> = bands
        .iter()
        .map(|b| {
            let (lo, hi) = range(b);
            hi - lo
        })
        .collect();
    let (lo, hi) = spans
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
    assert!(
        hi < lo * 2.0,
        "nenhuma banda pode ser «mais forte»: as excursoes vao de {lo} a {hi}"
    );
}

/// **A BANDA 3 É ANISOTRÓPICA e o CONTROLE não é** — a metade que prova o `scale_y`.
///
/// ⚠️ **A DIREÇÃO é contra-intuitiva e a medição a corrigiu.** Um `scale_y` MAIOR faz o mesmo
/// passo de mundo cobrir mais espaço de ruído, então o campo varia **mais depressa** em Y e
/// as manchas ficam **baixas e largas** — listras deitadas. Logo `dx/dy` tem de **CAIR**.
///
/// Sem o controle, *"a banda 3 varia diferente nos dois eixos"* ficaria verde sobre um campo
/// qualquer — um Perlin de uma oitava **não** é perfeitamente isotrópico numa amostra finita,
/// e é por isso que a barra é uma RAZÃO ENTRE BANDAS, não um valor absoluto.
#[test]
fn the_stretched_band_is_anisotropic_where_the_control_is_not() {
    let side = side();
    let ratio = |band: usize| {
        let d = band_field(band);
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        for r in 0..side {
            for c in 0..side {
                let i = r * side + c;
                if c + 1 < side {
                    dx += (d[i + 1] - d[i]).abs();
                }
                if r + 1 < side {
                    dy += (d[i + side] - d[i]).abs();
                }
            }
        }
        dx / dy.max(1e-6)
    };
    let plain = ratio(0);
    let stretched = ratio(2);
    assert!(
        (plain - 1.0).abs() < 0.6,
        "o CONTROLE tem de variar parecido nos dois eixos, e a razao e' {plain}"
    );
    assert!(
        stretched < plain * 0.6,
        "a banda esticada tem de variar MUITO mais em Y do que em X: {stretched} contra {plain}"
    );
    eprintln!("[=60] razao dx/dy: controle {plain:.3}, esticada {stretched:.3}");
}

/// **A sonda que a mensagem cita** — ela imprime, não afirma.
#[test]
#[ignore = "sonda: imprime os numeros que a mensagem da cena cita"]
fn measure_what_the_scene_shows() {
    eprintln!("\n[=60] o que a cena monta (o campo E' a COR do ponto)");
    for (i, label) in band_labels() {
        let f = band_field(i);
        let (lo, hi) = range(&f);
        let (tlo, thi) = range(&band_tint(i));
        eprintln!(
            "  banda {}: {} pontos, tamanho {lo:.3}..{hi:.3}, luminancia {tlo:.3}..{thi:.3}  ({label})",
            i + 1,
            f.len()
        );
    }
}

/// A variação média do campo ao longo das quatro direções da grade: `[→, ↓, ↘, ↙]`.
/// As faixas correm na direção de **MENOR** variação.
fn variation_by_direction(band: usize) -> [f32; 4] {
    let side = side();
    let d = band_field(band);
    let mut acc = [0.0f32; 4];
    let mut cnt = [0usize; 4];
    for r in 0..side {
        for c in 0..side {
            let i = r * side + c;
            let mut add = |k: usize, j: usize| {
                acc[k] += (d[j] - d[i]).abs();
                cnt[k] += 1;
            };
            if c + 1 < side {
                add(0, i + 1);
            }
            if r + 1 < side {
                add(1, i + side);
            }
            if c + 1 < side && r + 1 < side {
                add(2, i + side + 1);
            }
            if c > 0 && r + 1 < side {
                add(3, i + side - 1);
            }
        }
    }
    std::array::from_fn(|k| acc[k] / cnt[k].max(1) as f32)
}

/// **A BANDA 4 TEM FAIXAS NA DIAGONAL, e a 3 na horizontal** — a rotação tem de girar a
/// ANISOTROPIA, não só a fatia de ruído que se vê.
///
/// ⚠️ **Este gate é um report do Enio, e ele derrubou a lei que eu tinha escrito.** A v3
/// aplicava *escala primeiro, rotação depois*, com um comentário a defendê-la — e medido, a
/// banda 4 saía com faixas **horizontais**, iguais às da banda 3 (variação `→ 0,0077` contra
/// `↓ 0,0218`). O motivo é geométrico: com `M = R·S` as feições do mundo são a pré-imagem de
/// manchas redondas, `S⁻¹R⁻¹(círculo)`, e os eixos dessa elipse são os de `S⁻¹` — ou seja **os
/// eixos do MUNDO**. A rotação só troca *qual* pedaço de ruído se vê; a anisotropia fica
/// colada à tela. *Um knob que não move o que promete é um knob que mente.*
///
/// Com `M = S·R` (rodar o ponto **primeiro**), `M⁻¹ = R⁻¹S⁻¹` e os eixos da elipse saem
/// girados de `−θ` — as faixas viram. ⚠️ E as bandas 1–3 **não mudam**: com escala uniforme
/// `S = s·I` comuta com `R`, e na banda 3 a rotação é zero.
///
/// ⚠️ **Ele SUBSTITUI um gate que dava falso conforto.** O anterior
/// (`stretch_then_rotate_is_not_rotate_then_stretch`) construía as duas ordens à mão e
/// afirmava que elas **diferem** — verdade, e inútil: ele nunca perguntou **qual** delas o
/// nó embarcava. Um gate que prova que duas escolhas são distintas não defende a escolha.
/// Este mede a direção que o olho lê, que é a afirmação que o produto faz.
///
/// Medido depois da correção: banda 4 `→ 0,0185 · ↓ 0,0186 · ↘ 0,0202 · ↙ 0,0110`.
#[test]
fn the_fourth_band_runs_its_stripes_on_the_diagonal() {
    let flat = variation_by_direction(2);
    let least = |v: [f32; 4]| (0..4).min_by(|a, b| v[*a].total_cmp(&v[*b])).unwrap();
    assert_eq!(
        least(flat),
        0,
        "a banda 3 tem de ter faixas HORIZONTAIS (a menor variacao no →), e mede {flat:?}"
    );
    let turned = variation_by_direction(3);
    let k = least(turned);
    assert!(
        k == 2 || k == 3,
        "a banda 4 tem de ter faixas na DIAGONAL, e a menor variacao dela esta' no {} \
         ({turned:?}) -- se for a mesma da banda 3, a rotacao nao girou a anisotropia",
        ["→", "↓", "↘", "↙"][k]
    );
}

/// **SONDA — em que DIREÇÃO as faixas correm?** Imprime a variação média do campo ao longo
/// de quatro direções da grade; as faixas correm na de MENOR variação.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn measure_the_stripe_direction() {
    let side = side();
    eprintln!("\n[=60] variacao media do campo, por direcao (a faixa corre na MENOR)");
    for (b, label) in band_labels() {
        let d = band_field(b);
        let mut acc = [0.0f32; 4];
        let mut cnt = [0usize; 4];
        for r in 0..side {
            for c in 0..side {
                let i = r * side + c;
                let mut add = |k: usize, j: usize| {
                    acc[k] += (d[j] - d[i]).abs();
                    cnt[k] += 1;
                };
                if c + 1 < side {
                    add(0, i + 1); // →  (horizontal)
                }
                if r + 1 < side {
                    add(1, i + side); // ↓  (vertical)
                }
                if c + 1 < side && r + 1 < side {
                    add(2, i + side + 1); // ↘
                }
                if c > 0 && r + 1 < side {
                    add(3, i + side - 1); // ↙
                }
            }
        }
        let m: Vec<f32> = (0..4).map(|k| acc[k] / cnt[k].max(1) as f32).collect();
        let least = (0..4).min_by(|a, b| m[*a].total_cmp(&m[*b])).unwrap();
        let name = ["horizontal", "vertical", "diagonal ↘", "diagonal ↙"][least];
        eprintln!(
            "  banda {}: → {:.4}  ↓ {:.4}  ↘ {:.4}  ↙ {:.4}   => faixas na {name}  ({label})",
            b + 1,
            m[0],
            m[1],
            m[2],
            m[3]
        );
    }
}
