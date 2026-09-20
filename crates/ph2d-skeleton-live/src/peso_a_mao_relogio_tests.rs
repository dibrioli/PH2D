//! ⏱️ **OS DOIS GATES DE RELÓGIO DO PINCEL DE PESO** — filho do [`super`] para herdar as fixturas
//! dele (`palco_estrela`, `forma`, `ponto_de`, `peso_em`).
//!
//! ⭐⭐ **A pergunta é OUTRA e por isso o ficheiro é outro.** O pai mede *o que uma mancha FAZ ao
//! peso*; aqui mede-se *o que ela CUSTA* — e as duas famílias partem-se por razões diferentes: uma
//! reprova quando a lei muda, a outra quando a máquina está com carga.
//!
//! ⚠️ **O corte foi forçado pelo tecto de LOC** (`762` contra `700`) e é melhor do que o ficheiro
//! era: ⛔ a cura de um tecto vermelho é o CORTE por responsabilidade, nunca uma entrada no
//! `FILE_OVERAGE_OK`.
//!
//! ⚠️⚠️ **Os dois são candidatos à família de flakes de fan-out do `CLAUDE.md` §5.0** — eles
//! comparam duas medianas de um RECURSO partilhado. *Re-rode-os SOZINHOS, com o `/proc/loadavg`
//! impresso ao lado, antes de olhar para o seu commit.*

use super::*;

/// ⭐⭐⭐ **O TECTO DE MANCHAS CABE NO ORÇAMENTO DO QUADRO** — a medição que fixa o [`MANCHAS_MAX`].
///
/// ⚠️ **A régua é uma RAZÃO contra o recook sem correcção nenhuma**, e não um relógio de parede: um
/// número absoluto aqui seria mais um membro da família de flakes de fan-out que o `CLAUDE.md` §5.0
/// enumera. O que se afirma é *quanto o tecto MULTIPLICA o custo do quadro*.
///
/// ⚠️⚠️ **A lista é ENCHIDA À MÃO, e é de propósito:** o que o quadro paga é `O(pontos × manchas)`,
/// e a conta não sabe como a lista se encheu. *Uma medição que passasse pelo gesto mediria quantas
/// manchas a FIXTURA consegue produzir* — que na estrela são umas dezenas — e não o tecto, que é o
/// número que esta régua existe para justificar.
#[test]
fn o_tecto_de_manchas_custa_uma_razao_e_nao_uma_ordem_de_grandeza() {
    let (mut sim, mut scene, map, id, ossos) = palco_estrela();
    let alvo = forma(&map, id);
    let limpo = mede(|| crate::skin_live::recook(&sim, &mut scene));
    let bone = *sim
        .world()
        .get::<ph2d_ecs::StableId>(ossos[0])
        .expect("o bind semeia a identidade dos ossos");
    let mut skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("a forma esta' presa")
        .clone();
    for k in 0..crate::peso_a_mao::MANCHAS_MAX {
        let a = f64::from(u32::try_from(k).unwrap_or(0)) * 0.5;
        skin.correcoes.push(ph2d_skeleton_ecs::CorreccaoDePeso {
            bone,
            centro: [20.0 + 15.0 * a.cos(), 20.0 + 15.0 * a.sin()],
            raio: 3.0,
            especie: ph2d_skeleton::Especie::Soma(0.05),
        });
    }
    let n = skin.correcoes.len();
    sim.world_mut().entity_mut(alvo).insert(skin);
    let cheio = mede(|| crate::skin_live::recook(&sim, &mut scene));
    let razao = cheio.as_secs_f64() / limpo.as_secs_f64().max(1e-9);
    eprintln!("[peso] {n} manchas: {limpo:?} -> {cheio:?} ({razao:.1}x)");
    assert!(
        razao < 40.0,
        "{n} manchas multiplicam o recook por {razao:.1}x ({limpo:?} -> {cheio:?})"
    );

    // ⭐⭐ **E a METADE ABSOLUTA, na pior pele que esta casa produz.** A razão acima é sobre a
    // estrela (trinta pontos); o que decide o tecto é a **malha graduada de uma imagem**, que vive
    // nas centenas. ⚠️ `PONTOS` é generoso de propósito — *um tecto derivado do caso médio é um
    // tecto que o caso mau não respeita*.
    const PONTOS: usize = 2_000;
    let pele = crate::skin_live::skin_of(&sim, alvo).expect("a pele resolve");
    let correcoes = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("a pele")
        .correcoes_resolvidas();
    let mut w = pele.scratch();
    let pts: Vec<[f64; 2]> = (0..PONTOS)
        .map(|k| {
            let a = f64::from(u32::try_from(k).unwrap_or(0)) * 0.13;
            [20.0 + 18.0 * a.cos(), 20.0 + 18.0 * a.sin()]
        })
        .collect();
    let com = mede(|| {
        for &p in &pts {
            std::hint::black_box(pele.point_corrected(p, None, &mut w, &correcoes));
        }
    });
    let sem = mede(|| {
        for &p in &pts {
            std::hint::black_box(pele.point_corrected(p, None, &mut w, &[]));
        }
    });
    let fator = com.as_secs_f64() / sem.as_secs_f64().max(1e-12);
    eprintln!("[peso] {PONTOS} pontos: {sem:?} -> {com:?} ({fator:.1}x) com {n} manchas");
    // ⚠️⚠️ **A asserção que corre SEMPRE é a RAZÃO** — um relógio de parede aqui mede o PERFIL DE
    // BUILD (medido: `205 µs` em `--release` contra `~2 ms` em debug, `10×`), e a suíte corre em
    // debug. *Um tecto absoluto num gate de debug afirma sobre outro programa.*
    assert!(
        fator < 60.0,
        "o tecto de {n} manchas multiplica o custo por ponto em {fator:.1}x"
    );
    // ⭐ E o ORÇAMENTO absoluto, só onde ele significa alguma coisa: um décimo de um quadro de
    // 60 Hz, no perfil em que o artista corre o app.
    if !cfg!(debug_assertions) {
        assert!(
            com < std::time::Duration::from_micros(1_670),
            "{PONTOS} pontos com o tecto cheio custam {com:?} - acima de 1/10 de um quadro"
        );
    }
}

/// ⭐⭐⭐ **O INDICADOR CUSTA O QUE O QUADRO JÁ PAGA PELA MESMA ARTE** — a régua do report de
/// 2026-09-19, que fez dele um passe permanente em vez de um que só acende sob o dedo.
///
/// ⚠️ **A régua é uma RAZÃO contra o `recook`**, e não um relógio de parede: o recook é o que o
/// quadro já gasta a deformar esta mesma arte, e o indicador faz a mesma conta de peso mais uma
/// mistura. *Um número absoluto aqui seria mais um membro da família de flakes de fan-out do
/// `CLAUDE.md` §5.0.*
///
/// ⛔⛔ **E é por isto que o preço de mostrar SEMPRE é pequeno:** antes ele corria sobre a arte sob
/// o dedo — que na cena do smoke é a imagem de `925` pontos —, e agora corre sobre toda a arte do
/// osso. *Deixou de haver um quadro barato (o dedo no vão) e um caro (o dedo na arte); todos os
/// quadros passaram a custar o caro*, que é o que esta razão limita.
#[test]
fn o_indicador_custa_o_que_o_quadro_ja_paga_pela_mesma_arte() {
    let (sim, mut scene, map, id, ossos) = palco_estrela();
    let _ = forma(&map, id);
    let quadro = mede(|| {
        crate::skin_live::recook(&sim, &mut scene);
    });
    let visto = mede(|| {
        let v = crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(ossos[0]));
        assert!(!v.is_empty(), "a fixtura deixou de mostrar pontos");
    });
    let razao = visto.as_secs_f64() / quadro.as_secs_f64().max(1e-9);
    eprintln!("[peso] indicador {visto:?} contra recook {quadro:?} ({razao:.2}x)");
    assert!(
        razao < 4.0,
        "o indicador custa {razao:.2}x o recook da mesma arte ({visto:?} contra {quadro:?}) — ele \
         corre TODO quadro com o pincel na mao, e este e' o orcamento que o justifica"
    );
}

fn mede(mut f: impl FnMut()) -> std::time::Duration {
    // A mediana de cinco — o mínimo de ruído que uma máquina partilhada permite.
    let mut v: Vec<std::time::Duration> = (0..5)
        .map(|_| {
            let t = std::time::Instant::now();
            f();
            t.elapsed()
        })
        .collect();
    v.sort_unstable();
    v[2]
}
