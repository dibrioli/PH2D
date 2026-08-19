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

/// **A ORDEM importa: «esticar e rodar» ≠ «rodar e esticar»** — e a banda 4 é a primeira.
///
/// ⚠️ Este gate defende a lei escrita no `FieldSpace::at`. Ele constrói a ordem CONTRÁRIA à
/// mão e exige que ela dê outro ponto de amostragem — se as duas coincidissem, a ordem seria
/// uma escolha sem consequência e o comentário que a justifica seria uma nota a envelhecer.
#[test]
fn stretch_then_rotate_is_not_rotate_then_stretch() {
    let (turn, scale, scale_y) = knobs();
    let ph = turn / 360.0;
    let sin_c = |p: f32| {
        let f = p - p.floor();
        let q = if f < 0.5 {
            let u = f * 2.0;
            4.0 * u * (1.0 - u)
        } else {
            let u = (f - 0.5) * 2.0;
            -4.0 * u * (1.0 - u)
        };
        0.225 * (q * q.abs() - q) + q
    };
    let (c, s) = (sin_c(ph + 0.25), sin_c(ph));
    // Um ponto FORA dos eixos — neles as duas ordens coincidem por simetria.
    let (px, py) = (1.7f32, 0.9f32);
    let ours = {
        let (x, y) = (px * scale, py * scale_y);
        (x * c - y * s, x * s + y * c)
    };
    let other = {
        let (x, y) = (px * c - py * s, px * s + py * c);
        (x * scale, y * scale_y)
    };
    let d = (ours.0 - other.0).abs().max((ours.1 - other.1).abs());
    assert!(
        d > 0.05,
        "as duas ordens tem de dar pontos de amostragem diferentes, e diferem {d}"
    );
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
