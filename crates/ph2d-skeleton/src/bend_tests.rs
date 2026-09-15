//! Os gates do **osso que dobra**. ⚠️ Cada nome diz o SINTOMA que o defeito produz, nunca a função
//! que ele exercita.
//!
//! ⚠️ **O que NÃO está aqui:** de onde vêm as alças (é lei da hierarquia, e a hierarquia não existe
//! nesta crate) e como o osso curvo se DESENHA (é lei de uma mídia).

use super::bend::{
    Bend, BoneSpec, MAX_SEGMENTS, arc_length, frame, point_at, polyline, segments_of, share,
};
use super::{Skin, SkinBone, Xform};

/// Um osso deitado no eixo X, da origem a `(len, 0)`, em repouso e sem pose.
fn deitado(len: f64) -> (Xform, f64) {
    (Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]), len)
}

/// A pele de UM osso, com `segments` sub-ossos e a curvatura pedida.
fn pele(len: f64, strength: f64, segments: u8, curva: Bend, mundo: Xform) -> Skin {
    let (rest, _) = deitado(len);
    let mut ossos = Vec::new();
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength,
            segments,
            curve: curva,
        },
        mundo,
        Xform::IDENTITY,
        &mut ossos,
    );
    Skin::new(ossos).expect("o osso existe")
}

/// Uma curvatura que levanta a ponta: as duas alças sobem.
fn curva_para_cima(h: f64) -> Bend {
    Bend {
        inn: [0.0, h],
        out: [0.0, h],
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O ponto neutro
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **UM OSSO RECTO CONTINUA A SER UM OSSO** — a lei da casa (todo motor novo é no-op no ponto
/// neutro), aqui na versão FORTE: não é *«quase igual»*, é `assert_eq!` sobre o osso inteiro.
///
/// ⚠️ **O sintoma que ele apanha é o pior de todos: nenhum.** Se a fábrica emitisse `N` sub-ossos
/// para uma curva recta, todo desenho já ligado a um esqueleto mudaria no último bit — e ninguém
/// veria, até um golden de outra linha reprovar sem uma linha de produto ter mudado.
#[test]
fn a_straight_bone_is_still_exactly_one_bone() {
    let (rest, len) = deitado(10.0);
    let mundo = Xform([0.6, 0.8, -0.8, 0.6, 3.0, -2.0]);
    let sozinho = SkinBone::new(rest, len, 1.5, mundo, Xform::IDENTITY).expect("recto");
    for segments in [0u8, 1, 2, 8, 32, 200] {
        let mut ossos = Vec::new();
        SkinBone::bent(
            rest,
            BoneSpec {
                length: len,
                strength: 1.5,
                segments,
                curve: Bend::STRAIGHT,
            },
            mundo,
            Xform::IDENTITY,
            &mut ossos,
        );
        assert_eq!(ossos.len(), 1, "segments = {segments} numa curva recta");
        assert_eq!(ossos[0], sozinho, "segments = {segments}");
    }
}

/// ⭐⭐ **UM SÓ SEGMENTO NÃO DOBRA NADA** — a outra metade do neutro: mesmo com a curvatura
/// autorada, `segments = 1` devolve o osso de sempre, ao bit.
///
/// ⚠️ Sem isto, ligar a curvatura num osso com o slider de segmentos em `1` faria a arte saltar
/// para a corda da curva — um salto, não uma dobra.
///
/// ⭐⭐ **E as DUAS metades do curto-circuito não são a mesma coisa.** Medido por mutação: apagar a
/// metade dos **segmentos** do [`BoneSpec::is_rigid`] deixa a suíte inteira verde, porque com `n = 1`
/// a fábrica de frames devolve a IDENTIDADE por construção — a corda vai da raiz à ponta e a razão
/// dá `1,0` ao bit. ⇒ *aquela metade é uma poupança de CUSTO, não uma correcção*, e um produtor que
/// se esqueça dela continua correcto e só mais lento. A asserção do `frame` abaixo é o que torna
/// essa equivalência uma LEI em vez de um acidente — sem ela, a mutação seria indistinguível de um
/// buraco. ⛔ Apagar a metade da **curvatura**, essa, mata dois gates.
#[test]
fn one_segment_never_bends_whatever_the_handles_say() {
    let (rest, len) = deitado(10.0);
    let mundo = Xform([1.0, 0.0, 0.0, 1.0, 2.0, 5.0]);
    let sozinho = SkinBone::new(rest, len, 2.0, mundo, Xform::IDENTITY).expect("recto");
    let mut ossos = Vec::new();
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength: 2.0,
            segments: 1,
            curve: curva_para_cima(4.0),
        },
        mundo,
        Xform::IDENTITY,
        &mut ossos,
    );
    assert_eq!(ossos, vec![sozinho]);
    // A LEI que torna o curto-circuito dos segmentos uma poupança e não uma correcção: com um
    // segmento só, a corda É o eixo, e o frame é a identidade **ao bit** para qualquer curvatura.
    for curva in [
        curva_para_cima(4.0),
        Bend {
            inn: [3.0, -7.0],
            out: [-1.0, 9.0],
        },
    ] {
        assert_eq!(frame(len, 1, curva, 0), Xform::IDENTITY, "{curva:?}");
    }
}

/// ⭐⭐⭐ **A CURVA RECTA É O SEGMENTO AO BIT** — a identidade polinomial que faz todo o resto ser
/// exacto em vez de tolerado.
///
/// ⚠️ **Ela mede a ESCRITA da fórmula, não a matemática.** A base de Bernstein crua daria
/// `1e-16` de resíduo e todos os gates a jusante passariam a precisar de uma barra; escrita como
/// *«a recta mais a correcção»*, a correcção é `0.0` e a soma é exacta.
#[test]
fn a_straight_curve_is_the_axis_bit_for_bit() {
    let len = 7.0;
    for k in 0..=64 {
        let t = f64::from(k) / 64.0;
        assert_eq!(point_at(len, Bend::STRAIGHT, t), [len * t, 0.0], "t = {t}");
    }
    for n in [1u8, 2, 3, 5, 8, 17, 32] {
        for k in 0..n {
            assert_eq!(
                frame(len, n, Bend::STRAIGHT, k),
                Xform::IDENTITY,
                "n = {n}, k = {k}"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A curva
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A ARTE NÃO ABRE FENDA NAS JUNTAS** — o frame de cada sub-osso põe o nó `k+1` do eixo
/// exactamente onde o frame seguinte põe o nó `k` dele.
///
/// ⚠️ **É esta propriedade que obriga o frame a ter ESCALA** e não só rotação: com rotação pura o
/// nó seguinte aterra à distância do EIXO em vez de à da CORDA, e cada junta ganha um degrau do
/// tamanho da diferença — a fenda que a régua da 2.ª mídia já cobrou uma vez.
#[test]
fn the_art_does_not_tear_at_the_joints() {
    let (len, n) = (10.0, 8u8);
    let curva = curva_para_cima(6.0);
    let mut pior = 0.0_f64;
    for k in 0..n - 1 {
        let (f0, f1) = (frame(len, n, curva, k), frame(len, n, curva, k + 1));
        let x = len * f64::from(k + 1) / f64::from(n);
        let (a, b) = (f0.apply([x, 0.0]), f1.apply([x, 0.0]));
        pior = pior.max((a[0] - b[0]).abs()).max((a[1] - b[1]).abs());
    }
    // Barra: o arredondamento de uma divisão e de duas multiplicações sobre coordenadas de ~10.
    assert!(pior < 1e-12, "fenda de {pior} entre juntas vizinhas");
}

/// ⭐⭐ **OS NÓS DO EIXO ATERRAM NA CURVA** — o frame `k` leva o nó `k` ao ponto `k` da Bézier.
///
/// ⚠️ Sem isto o osso dobraria *alguma coisa*, mas não a curva que o artista desenhou: a alça
/// deixaria de ser a alça.
#[test]
fn every_axis_node_lands_on_the_curve_it_was_given() {
    let (len, n) = (10.0, 6u8);
    let curva = Bend {
        inn: [1.0, 3.0],
        out: [-2.0, 5.0],
    };
    for k in 0..n {
        let t = f64::from(k) / f64::from(n);
        let esperado = point_at(len, curva, t);
        let obtido = frame(len, n, curva, k).apply([len * t, 0.0]);
        assert!(
            (esperado[0] - obtido[0]).abs() < 1e-12 && (esperado[1] - obtido[1]).abs() < 1e-12,
            "nó {k}: {obtido:?} != {esperado:?}"
        );
    }
}

/// ⭐⭐⭐ **A CURVATURA ARQUEIA O CORPO E NÃO MEXE A PONTA** — o controlo positivo da wave, e ao
/// mesmo tempo a declaração da semântica.
///
/// ⚠️⚠️ **A minha primeira redacção deste gate exigia que a PONTA subisse, e a medição derrubou-a:**
/// a Bézier começa na raiz e acaba na ponta **do osso**, então arquear as alças levanta o miolo e
/// deixa as duas extremidades onde estavam. É o que a referência faz e é o que tem de ser — quem
/// manda na ponta é o comprimento e a rotação do osso, e o filho dele está pendurado ali. *Se a
/// curvatura movesse a ponta, a corrente abria uma fenda em cada junta ao dobrar.*
#[test]
fn the_curve_bows_the_body_and_leaves_the_tip_alone() {
    let len = 10.0;
    let p = pele(len, 1.0, 8, curva_para_cima(6.0), Xform::IDENTITY);
    let mut w = p.scratch();
    let meio = p.point([len * 0.5, 0.0], &mut w);
    let ponta = p.point([len, 0.0], &mut w);
    let raiz = p.point([0.0, 0.0], &mut w);
    assert!(meio[1] > 1.0, "o miolo não arqueou: {meio:?}");
    assert!(
        raiz[0].abs() < 1e-12 && raiz[1].abs() < 1e-12,
        "a raiz mexeu-se: {raiz:?}"
    );
    assert!(
        (ponta[0] - len).abs() < 1e-12 && ponta[1].abs() < 1e-12,
        "a ponta mexeu-se: {ponta:?}"
    );
}

/// ⭐⭐⭐ **A ARTE NÃO ENGORDA AO DOBRAR** — a espessura perpendicular ao osso é preservada **ao
/// bit**, e é por isso que a escala do frame é axial e não uniforme.
///
/// ⚠️ **O defeito que ele apanha é o mais visível de todos e o mais fácil de escrever sem querer:**
/// com uma semelhança (rotação + escala uniforme) a arte fica `+87 %` mais gorda nas pontas de um
/// arco forte, e a régua da dobra — que mede inversão, não espessura — não vê nada.
#[test]
fn bending_does_not_fatten_the_art() {
    let (len, n) = (10.0, 8u8);
    let curva = curva_para_cima(6.0);
    for k in 0..n {
        let f = frame(len, n, curva, k);
        // A coluna do `y` do afim É a imagem do versor perpendicular ao osso.
        let [_, _, c, d, _, _] = f.0;
        let espessura = c.hypot(d);
        assert!(
            (espessura - 1.0).abs() < 1e-12,
            "o sub-osso {k} engorda a arte {espessura}×"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A quota
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **PARTIR UM OSSO NÃO O ENGORDA** — a quota é uma partição da unidade, logo a força total do
/// osso é a mesma com 1 ou com 32 segmentos.
///
/// ⚠️ **É o gate que justifica os sub-ossos partilharem o eixo de repouso.** Com um eixo próprio
/// por sub-osso, o `bump` de cada um somaria e um osso de 8 segmentos ganharia ~8× de influência
/// contra o vizinho — o braço puxaria o tronco, e nenhuma régua de dobra veria porquê.
#[test]
fn splitting_a_bone_does_not_multiply_its_pull() {
    for n in [1u8, 2, 3, 7, 8, 32] {
        for i in 0..=40 {
            let u = f64::from(i) / 40.0;
            let soma: f64 = (0..n).map(|k| share(k, n, u)).sum();
            assert!(
                (soma - 1.0).abs() < 1e-12,
                "n = {n}, u = {u}: as quotas somam {soma}"
            );
        }
    }
}

/// ⭐⭐ **UM OSSO SEM SEGMENTOS TEM A QUOTA INTEIRA, AO BIT** — é o que faz o caminho antigo passar
/// por esta multiplicação sem mudar um bit (`peso * 1.0 == peso`).
#[test]
fn a_bone_without_segments_keeps_the_whole_share() {
    for i in 0..=40 {
        let u = f64::from(i) / 40.0;
        assert_eq!(share(0, 1, u), 1.0, "u = {u}");
    }
}

/// ⭐⭐ **O TECTO SATURA EM SILÊNCIO** — pedir 200 segmentos devolve [`MAX_SEGMENTS`], não um
/// pânico e não 200.
///
/// ⚠️ Um slider arrastado ao fim não é um erro do artista; recusar seria transformar um gesto
/// normal num diálogo.
#[test]
fn asking_for_too_many_segments_saturates_instead_of_exploding() {
    assert_eq!(segments_of(0), 1);
    assert_eq!(segments_of(1), 1);
    assert_eq!(segments_of(MAX_SEGMENTS), MAX_SEGMENTS);
    assert_eq!(segments_of(u8::MAX), MAX_SEGMENTS);
    let mut ossos = Vec::new();
    let (rest, len) = deitado(10.0);
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength: 1.0,
            segments: u8::MAX,
            curve: curva_para_cima(3.0),
        },
        Xform::IDENTITY,
        Xform::IDENTITY,
        &mut ossos,
    );
    assert_eq!(ossos.len(), usize::from(MAX_SEGMENTS));
}

/// ⭐⭐ **OS SUB-OSSOS SAEM EM ORDEM E CONTÍGUOS** — o desempate do órfão acha o grupo a partir de
/// um membro por aritmética de índice, e uma emissão fora de ordem daria a ele o grupo errado, em
/// silêncio.
#[test]
fn the_sub_bones_come_out_in_order_and_contiguous() {
    let mut ossos = Vec::new();
    let (rest, len) = deitado(10.0);
    // Um osso recto ANTES, para o grupo não começar no índice zero.
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength: 1.0,
            segments: 1,
            curve: Bend::STRAIGHT,
        },
        Xform::IDENTITY,
        Xform::IDENTITY,
        &mut ossos,
    );
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength: 1.0,
            segments: 5,
            curve: curva_para_cima(3.0),
        },
        Xform::IDENTITY,
        Xform::IDENTITY,
        &mut ossos,
    );
    assert_eq!(ossos[0].sub, (0, 1));
    for (k, b) in ossos[1..].iter().enumerate() {
        assert_eq!(b.sub, (u8::try_from(k).expect("k pequeno"), 5));
    }
}

/// ⭐⭐⭐ **UM ÓRFÃO LONGE DA PONTA SEGUE A PONTA** — quando o ponto está fora do raio de todo osso,
/// o osso mais próximo leva-o, e num osso CURVO quem o leva é a fatia certa da curva.
///
/// ⚠️ **O defeito que ele apanha é mudo e específico:** os sub-ossos partilham o eixo de repouso,
/// logo a distância é a MESMA nos `n` — sem a quota, quem ganhava o desempate era o primeiro da
/// lista, e um ponto órfão junto da ponta saltava para a raiz da curva.
#[test]
fn an_orphan_near_the_tip_follows_the_tip_not_the_root() {
    let (len, n) = (10.0, 8u8);
    // `strength` minúsculo ⇒ o raio é uma migalha e TODA a arte é órfã.
    let p = pele(len, 0.01, n, curva_para_cima(6.0), Xform::IDENTITY);
    let ossos = p.bones();
    let mut w = p.scratch();
    let (na_raiz, na_ponta) = ([0.0, 5.0], [len, 5.0]);
    // ⚠️ Igualdade EXACTA: no desempate a quota vale `1` num sub-osso só, e a mistura de um termo é
    // o próprio termo. Uma barra aqui esconderia um peso repartido pelo grupo errado.
    assert_eq!(
        p.point(na_raiz, &mut w),
        ossos[0].pose.apply(na_raiz),
        "o órfão da raiz não foi levado pelo primeiro sub-osso"
    );
    assert_eq!(
        p.point(na_ponta, &mut w),
        ossos[usize::from(n) - 1].pose.apply(na_ponta),
        "o órfão da ponta não foi levado pelo último sub-osso"
    );
    // E o controlo: os dois sub-ossos fazem coisas DIFERENTES, senão o gate acima é trivial.
    assert_ne!(ossos[0].pose, ossos[usize::from(n) - 1].pose);
}

/// ⭐⭐⭐ **UMA PELE DE OSSOS RECTOS NÃO MUDOU UM BIT** — o gate de regressão da wave inteira, do
/// lado do produto: a mesma pele de sempre, deformada ponto a ponto, dá exactamente o que dava.
///
/// ⚠️ A referência não é um número gravado: é a mesma pele construída pela porta ANTIGA
/// ([`SkinBone::new`]) contra a pele construída pela porta NOVA com a curvatura recta — *um golden
/// escrito à mão mediria a minha aritmética, não a igualdade dos dois caminhos*.
#[test]
fn a_skin_of_straight_bones_did_not_move_a_single_bit() {
    let (rest, len) = deitado(6.0);
    let mundo = Xform([0.8, 0.6, -0.6, 0.8, 1.0, 2.0]);
    let antiga = Skin::new(vec![
        SkinBone::new(rest, len, 1.5, mundo, Xform::IDENTITY).expect("recto"),
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, 6.0, 0.0]),
            len,
            1.5,
            mundo,
            Xform::IDENTITY,
        )
        .expect("recto"),
    ])
    .expect("2 ossos");
    let mut nova_ossos = Vec::new();
    SkinBone::bent(
        rest,
        BoneSpec {
            length: len,
            strength: 1.5,
            segments: 8,
            curve: Bend::STRAIGHT,
        },
        mundo,
        Xform::IDENTITY,
        &mut nova_ossos,
    );
    SkinBone::bent(
        Xform([1.0, 0.0, 0.0, 1.0, 6.0, 0.0]),
        BoneSpec {
            length: len,
            strength: 1.5,
            segments: 8,
            curve: Bend::STRAIGHT,
        },
        mundo,
        Xform::IDENTITY,
        &mut nova_ossos,
    );
    let nova = Skin::new(nova_ossos).expect("2 ossos");
    let (mut wa, mut wb) = (antiga.scratch(), nova.scratch());
    for i in 0..=20 {
        for j in 0..=20 {
            let p = [f64::from(i) * 0.7 - 2.0, f64::from(j) * 0.7 - 4.0];
            assert_eq!(
                antiga.point(p, &mut wa),
                nova.point(p, &mut wb),
                "p = {p:?}"
            );
        }
    }
}

/// ⭐ **O PREÇO DE UM OSSO CURVO, e o que ele diz sobre o tecto** — a medição que autoriza o
/// [`MAX_SEGMENTS`].
///
/// ⛔ **`#[ignore]`, e não é um gate**: ele IMPRIME. Um tecto de relógio aqui seria mais um membro
/// da família de flakes de recurso do `CLAUDE.md` §5.0 — o que se quer é a ORDEM DE GRANDEZA contra
/// um quadro de `16,7 ms`, e ela decide-se uma vez.
///
/// O recurso é o custo por PONTO: a [`Skin::weights_at`] percorre todos os ossos, então `N`
/// segmentos multiplicam por `N` o custo daquele osso.
#[test]
#[ignore = "sonda de relógio: imprime, não julga"]
fn bend_measure_the_ceiling() {
    // A carga: 4 ossos (um deles curvo) sobre uma malha de imagem generosa.
    const PONTOS: usize = 20_000;
    let pontos: Vec<[f64; 2]> = (0..PONTOS)
        .map(|i| {
            let t = i as f64 * 0.001;
            [t * 10.0 % 40.0, (t * 7.0).sin() * 6.0]
        })
        .collect();
    println!("[bend] {PONTOS} pontos x 4 ossos (um curvo), mediana de 5 corridas:");
    let mut base = 0.0_f64;
    for n in [1u8, 2, 4, 8, 16, 32, 64] {
        let mut ossos = Vec::new();
        for i in 0..4 {
            let rest = Xform([1.0, 0.0, 0.0, 1.0, f64::from(i) * 10.0, 0.0]);
            let (segs, curva) = if i == 1 {
                (n, curva_para_cima(3.0))
            } else {
                (1, Bend::STRAIGHT)
            };
            SkinBone::bent(
                rest,
                BoneSpec {
                    length: 10.0,
                    strength: 1.5,
                    segments: segs,
                    curve: curva,
                },
                rest,
                Xform::IDENTITY,
                &mut ossos,
            );
        }
        let pele = Skin::new(ossos).expect("4 ossos");
        let mut amostras = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let mut p = pontos.clone();
            pele.deform_points(p.iter_mut());
            std::hint::black_box(&p);
            amostras.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        amostras.sort_by(f64::total_cmp);
        let ms = amostras[2];
        if n == 1 {
            base = ms;
        }
        println!(
            "  segmentos {n:>3}  ossos {:>3}  {ms:>7.3} ms  {:>5.2}x  {:>5.1} % de um quadro",
            pele.len(),
            ms / base,
            ms / 16.7 * 100.0
        );
    }
}

/// ⭐⭐⭐ **A POLILINHA DE UM OSSO RECTO SÃO DOIS PONTOS, AO BIT** — o mesmo colapso da fábrica de
/// sub-ossos, e a razão de todo consumidor que só lê a raiz e a ponta poder derivar-se daqui sem
/// mudar um bit.
#[test]
fn a_rigid_bones_polyline_is_exactly_its_axis() {
    for segments in [0u8, 1, 8, 32] {
        for curve in [Bend::STRAIGHT, curva_para_cima(5.0)] {
            let spec = BoneSpec {
                length: 7.0,
                strength: 1.0,
                segments,
                curve,
            };
            if !spec.is_rigid() {
                continue;
            }
            assert_eq!(
                polyline(spec),
                vec![[0.0, 0.0], [7.0, 0.0]],
                "segments = {segments}, curve = {curve:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **A POLILINHA COMEÇA NA RAIZ E ACABA NA PONTA, SEMPRE** — a lei que deixa a ponta de um
/// osso curvo continuar a ser a ponta.
///
/// ⚠️ **Sem ela, o filho de um osso arqueado descolava do pai:** a hierarquia pendura o filho em
/// `translation.x = length`, e se a curvatura movesse o último nó, o desenho diria uma coisa e a
/// cinemática outra.
#[test]
fn the_polyline_always_starts_at_the_root_and_ends_at_the_tip() {
    for segments in [2u8, 3, 8, 32] {
        let spec = BoneSpec {
            length: 7.0,
            strength: 1.0,
            segments,
            curve: Bend {
                inn: [2.0, 5.0],
                out: [-3.0, 4.0],
            },
        };
        let p = polyline(spec);
        assert_eq!(p.len(), usize::from(segments) + 1, "n = {segments}");
        assert_eq!(p[0], [0.0, 0.0], "n = {segments}");
        assert_eq!(*p.last().expect("tem nós"), [7.0, 0.0], "n = {segments}");
    }
}

/// ⭐⭐ **ARQUEAR ALONGA O CAMINHO** — o comprimento percorrido de um osso curvo é maior que o do
/// eixo, e é por isso que o desenho e o dedo não podem dimensionar-se pela CORDA.
#[test]
fn bowing_a_bone_makes_the_walk_longer_than_the_chord() {
    let recto = BoneSpec::straight(10.0, 1.0);
    assert!((arc_length(&polyline(recto)) - 10.0).abs() < 1e-12);
    let curvo = BoneSpec {
        segments: 16,
        curve: curva_para_cima(6.0),
        ..recto
    };
    let andado = arc_length(&polyline(curvo));
    assert!(andado > 11.0, "o arco mal cresceu: {andado}");
}

/// ⭐⭐⭐ **O DEDO MEDE O CORPO, NÃO A CORDA** — num osso arqueado a distância à polilinha e a
/// distância ao eixo raiz→ponta são coisas diferentes, e a que o artista vê é a primeira.
///
/// ⚠️ **Com dois nós as duas são a MESMA função ao bit**, e é isso que faz um osso recto continuar
/// a ser agarrado exactamente como sempre foi.
#[test]
fn the_finger_measures_the_body_not_the_chord() {
    use super::{dist2_to_polyline, dist2_to_segment};
    let recto = polyline(BoneSpec::straight(10.0, 1.0));
    for p in [[5.0, 3.0], [-2.0, 0.0], [14.0, 1.0], [5.0, 0.0]] {
        assert_eq!(
            dist2_to_polyline(p, &recto),
            dist2_to_segment(p, [0.0, 0.0], [10.0, 0.0]),
            "p = {p:?}"
        );
    }
    // Arqueado: um ponto sobre o miolo da curva está LONGE do eixo e EM CIMA do corpo.
    let curvo = polyline(BoneSpec {
        segments: 16,
        curve: curva_para_cima(6.0),
        ..BoneSpec::straight(10.0, 1.0)
    });
    let no_arco = curvo[8];
    assert!(
        dist2_to_polyline(no_arco, &curvo) < 1e-20,
        "um no da propria polilinha tem de estar sobre ela"
    );
    assert!(
        dist2_to_segment(no_arco, [0.0, 0.0], [10.0, 0.0]) > 4.0,
        "a fixtura nao arqueia o suficiente para separar as duas reguas"
    );
}

/// ⭐⭐⭐ **O MESMO RIG DEZ VEZES MAIOR DOBRA IGUAL** — a irmã do
/// `the_same_rig_ten_times_bigger_weighs_exactly_the_same`, e a razão de as alças serem MÚLTIPLOS
/// do comprimento em vez de distâncias.
///
/// ⚠️ **Com unidades absolutas este gate reprova**, e o sintoma seria mudo e caro: escalar um
/// personagem endireitaria todos os ossos dele — o desenho ficaria certo no tamanho em que foi
/// autorado e progressivamente recto em qualquer outro.
#[test]
fn the_same_rig_ten_times_bigger_bends_exactly_the_same() {
    let curva = Bend {
        inn: [0.1, 0.4],
        out: [-0.2, 0.3],
    };
    let pequeno = polyline(BoneSpec {
        segments: 8,
        curve: curva,
        ..BoneSpec::straight(3.0, 1.0)
    });
    let grande = polyline(BoneSpec {
        segments: 8,
        curve: curva,
        ..BoneSpec::straight(30.0, 1.0)
    });
    for (p, g) in pequeno.iter().zip(grande.iter()) {
        assert!(
            (p[0] * 10.0 - g[0]).abs() < 1e-12 && (p[1] * 10.0 - g[1]).abs() < 1e-12,
            "{p:?} x10 != {g:?}"
        );
    }
    // Controlo: a fixtura arqueia mesmo, senão o gate compara duas rectas.
    assert!(
        pequeno[4][1].abs() > 0.5,
        "a fixtura nao arqueou: {pequeno:?}"
    );
}
