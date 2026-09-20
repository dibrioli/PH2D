//! ⭐⭐⭐ **O TERCEIRO ELO DO PINCEL DE PESO: o GESTO chega à arte ENTRE os nós.**
//!
//! # ⛔⛔⛔ Porque este ficheiro existe
//!
//! Em 2026-09-19 a [`ph2d_vec_skin::curva`] (F30) fez a arte seguir o peso **entre** dois nós, e o
//! dono reportou no mesmo dia: *«o que vc mandou fazer não funcionou»*. A lei estava certa — o gate
//! dela mede `0,000000` contra `0,836850` — e **o gesto não a alcançava**: o pincel ancorava a
//! mancha no NÓ mais perto e recusava (`ForaDaArte`) quando o dedo estava mais longe do que o raio.
//!
//! Medido na barra da cena do dono, antes da cura:
//!
//! | o dedo | nó mais perto | raio de fábrica | veredito |
//! |---|---|---|---|
//! | no MEIO da barra | `3,041` | `0,40` | **`ForaDaArte`** |
//! | idem, raio a `400 px` | `3,041` | `4,00` | `Pintada`, **centro na QUINA** |
//!
//! ⇒ *o censo prova que a PORTA faz efeito; a costura prova que o clique chega ao BARRAMENTO; e
//! nada perguntava se o GESTO consegue pedir o que a lei sabe fazer* — o terceiro elo do
//! `CLAUDE.md` §5.0, desta vez sobre uma wave minha.
//!
//! ⚠️ **Os gates do irmão [`super::peso_a_mao_tests`] não podiam apanhar isto:** eles pintam **em
//! cima de um vértice**, que é o caso em que as duas leis concordam. *Uma fixtura que aponta sempre
//! para um nó não testa o que acontece entre eles.*

use crate::barra_da_cena_tests_support::{PPM, barra_da_cena_com, forma, raio_de_fabrica};
use crate::peso_a_mao::{Pincelada, pinta, pontos_de_peso};
use ph2d_ecs::Transform;

/// O MEIO da barra da cena — o sítio do report.
const MEIO: [f64; 2] = [-5.0, 2.5];

/// ⛔⛔ **O sujeito desta suíte é a barra GROSSA — a de oito nós**, que é a forma sobre a qual o
/// dono reportou em 2026-09-19 e o que um ficheiro gravado antes daquele dia traz.
///
/// ⚠️ **A subdivisão do bind, da mesma data, tira o fenómeno da forma de OMISSÃO** (o nó mais perto
/// do meio da barra passa de `3,04` para `0,50`, contra um pincel de `0,40`) — e é isso que torna
/// esta constante obrigatória: *sem ela estes gates ficariam verdes por vácuo, e a lei da âncora
/// deixaria de ter quem a defenda no dia em que alguém tocasse nela.*
const GROSSA: bool = false;

/// ⭐⭐⭐ **PINTAR NO MEIO DA BARRA É ACEITE, E A MANCHA POUSA ENTRE OS NÓS.**
///
/// ⚠️ **A 1.ª asserção é o CONTROLO e ela vem primeiro:** sem provar que o nó mais perto está longe
/// do dedo, as outras duas seriam verdades triviais numa arte com nós por todo o lado. É essa
/// distância — `3,04` contra um pincel de `0,40` — que *é* o report.
///
/// (Mutações: ancorar no ponto de peso mais perto ⇒ RED na 3.ª; devolver a âncora sem a cerca do
/// raio ⇒ RED no irmão `um_clique_do_outro_lado_da_tela_continua_recusado`.)
#[test]
fn pintar_no_meio_da_barra_e_aceite_e_a_mancha_pousa_entre_os_nos() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena_com(GROSSA);
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();

    let nos: Vec<[f64; 2]> = pontos_de_peso(&sim, alvo, ossos[1], PPM)
        .iter()
        .map(|p| p.mundo)
        .collect();
    let perto = nos
        .iter()
        .map(|p| (p[0] - MEIO[0]).hypot(p[1] - MEIO[1]))
        .fold(f64::INFINITY, f64::min);
    assert!(
        perto > raio * 4.0,
        "o no' mais perto do meio da barra esta' a {perto} com um pincel de {raio} — a fixtura \
         deixou de conter o fenomeno do report, e as asserções abaixo passam a ser triviais"
    );

    let r = pinta(&mut sim, alvo, ossos[1], PPM, MEIO, raio, 0.15);
    assert!(
        matches!(r, Pincelada::Pintada { .. }),
        "o pincel recusou o meio da barra ({r:?}) — e' o report de 19/09 inteiro: a lei da curva \
         move a arte entre os nos e nenhum gesto consegue pedir-lho"
    );

    let manchas = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("a pele")
        .correcoes
        .clone();
    assert_eq!(manchas.len(), 1, "esperava UMA mancha");
    let c = manchas[0].centro;
    let ao_no = nos
        .iter()
        .map(|p| (p[0] - c[0]).hypot(p[1] - c[1]))
        .fold(f64::INFINITY, f64::min);
    assert!(
        ao_no > raio * 4.0,
        "a mancha pousou a {ao_no} de um no' — ela voltou a ser ancorada no no' mais perto, e o \
         meio da aresta continua inalcancavel"
    );
    assert!(
        (c[0] - MEIO[0]).abs() < raio,
        "a mancha pousou em {c:?}, fora do alcance do dedo em {MEIO:?}"
    );
}

/// ⭐⭐⭐ **E A ARTE MEXE-SE** — o elo que fecha a corrente, pelo recook do produto.
///
/// ⛔⛔⛔ **Ele mede a barra do PRODUTO, e a troca é de 2026-09-19 (2.ª ordem do dono).** A
/// redacção anterior media a barra GROSSA e o controlo era a lei dos pontos de controlo a ler
/// `0,000000` — *a F30 em pessoa*. Duas coisas mataram essa premissa no mesmo dia:
///
/// 1. **A subdivisão do bind** — na barra do produto o ponto do contorno **mais longe** de um nó
///    está a `0,3378`, contra um pincel de fábrica de `0,40`. ⇒ *«entre os nós» deixou de existir
///    como sítio onde o dedo pode cair*, e é por isso que os dois lados passam a ler o mesmo
///    número.
/// 2. **A correcção das alças** substituiu o refit ([`ph2d_vec_skin::curva`]), e ela não segue uma
///    feição mais fina do que um segmento — numa barra de oito nós uma mancha de raio `0,4` no meio
///    de uma aresta de `6` move **zero**. ⚠️ *Ali nenhuma das duas leis chega perto da verdade*
///    (medido: `0,71` e `0,85` de erro sobre uma barra de espessura `1`), e a cura daquele mundo é
///    a subdivisão, não a lei.
///
/// ⇒ o que se afirma é o que o artista tem: **na forma que o `Bind` produz, arrastar o pincel move
/// a arte**. O sujeito da F31 (a mancha a pousar entre dois nós) fica nos irmãos, que pedem a barra
/// grossa pelo nome.
#[test]
fn um_arrasto_pelo_meio_da_barra_move_a_arte() {
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(!GROSSA);
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();

    sim.world_mut()
        .get_mut::<Transform>(ossos[2])
        .expect("o osso tem pose")
        .rotation += 0.8;
    crate::skin_live::recook(&sim, &mut scene);
    let antes = arte(&scene);

    // ⭐ **O CONTROLO que explica a troca:** na barra do produto não há ponto do contorno fora do
    // alcance do pincel. *Sem esta linha, a mudança de fixtura leria-se como um gate afrouxado.*
    let contorno = crate::peso_a_mao::contorno_da_arte(&sim, alvo, PPM);
    let nos: Vec<[f64; 2]> = pontos_de_peso(&sim, alvo, ossos[2], PPM)
        .iter()
        .map(|p| p.mundo)
        .collect();
    let mais_longe = contorno
        .iter()
        .map(|q| {
            nos.iter()
                .map(|p| (p[0] - q[0]).hypot(p[1] - q[1]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0_f64, f64::max);
    assert!(
        mais_longe < raio,
        "o ponto do contorno mais longe de um no' esta' a {mais_longe} com um pincel de {raio} — a \
         subdivisao do bind deixou de cobrir a forma, e «entre os nos» voltou a existir"
    );

    // O arrasto do dono: seis pinceladas ao longo do contorno.
    let passo = (contorno.len() / 8).max(1);
    for (i, q) in contorno.iter().step_by(passo).take(6).enumerate() {
        let r = pinta(&mut sim, alvo, ossos[2], PPM, *q, raio, 0.15);
        assert!(
            matches!(r, Pincelada::Pintada { .. }),
            "a pincelada {i} em {q:?} foi recusada: {r:?}"
        );
    }
    crate::skin_live::recook(&sim, &mut scene);
    let moveu = desvio(&antes, &arte(&scene));
    eprintln!("[peso-entre-os-nos] o arrasto pela barra do produto moveu a arte: {moveu:.6}");
    assert!(
        moveu > 0.1,
        "o arrasto moveu a arte {moveu} — o gesto chega a` lei e a lei nao chega ao desenho"
    );
}

/// ⛔ **A cerca que o dedo tem de passar continua de pé** — um clique longe da arte é recusado.
///
/// ⚠️ Ela é a metade que a cura NÃO podia levar: sem ela um clique do outro lado da tela ancorava a
/// mancha no contorno mais próximo, longe e sem o artista o pedir.
#[test]
fn um_clique_do_outro_lado_da_tela_continua_recusado() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena_com(GROSSA);
    let alvo = forma(&map, id);
    let r = pinta(
        &mut sim,
        alvo,
        ossos[1],
        PPM,
        [MEIO[0], MEIO[1] + 40.0],
        raio_de_fabrica(),
        0.15,
    );
    assert_eq!(r, Pincelada::ForaDaArte, "o pincel aceitou um clique no vazio");
}

/// ⭐⭐ **ARRASTAR PELO MIOLO DA BARRA CONTA COMO ESTAR NA ARTE.**
///
/// A barra tem meia unidade de meia-altura contra um pincel de `0,40`: uma régua que só medisse a
/// distância ao CONTORNO recusaria exactamente a linha por onde o artista arrasta. ⚠️ O par de
/// asserções é obrigatório — a segunda prova que a fixtura contém o caso (o miolo está mesmo FORA
/// do alcance do contorno), senão a primeira passaria por acidente.
#[test]
fn o_miolo_da_barra_conta_como_arte() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena_com(GROSSA);
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();
    let r = pinta(&mut sim, alvo, ossos[1], PPM, MEIO, raio, 0.15);
    assert!(matches!(r, Pincelada::Pintada { .. }), "{r:?}");
    let c = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .expect("a pele")
        .correcoes[0]
        .centro;
    let ao_dedo = (c[0] - MEIO[0]).hypot(c[1] - MEIO[1]);
    assert!(
        ao_dedo > raio,
        "o contorno passa a {ao_dedo} do miolo com um pincel de {raio} — a fixtura deixou de medir \
         o caso «dentro da forma, longe da linha»"
    );
}

/// ⭐⭐ **A ARTE AMOSTRADA POR COMPRIMENTO DE ARCO** — a régua que sobrevive a um REFIT.
///
/// ⛔⛔ **A 1.ª redacção amostrava por SEGMENTO e devolvia `inf` assim que os dois lados tinham
/// contagens diferentes** — e é precisamente isso que a lei da curva faz: ela **reescreve** o
/// contorno. *Uma régua que desiste quando a representação muda não mede a única coisa que
/// interessa aqui, que é a FORMA.* ⇒ os dois lados são reamostrados em `N` pontos igualmente
/// espaçados ao longo do desenho, logo uma reparametrização pura lê `~0` e uma mudança de forma lê
/// a distância que o olho vê.
fn arte(scene: &ph2d_vec_scene::VecScene) -> Vec<[f64; 2]> {
    const N: usize = 256;
    let bruta = achatada(scene);
    if bruta.len() < 2 {
        return bruta;
    }
    let mut acum = vec![0.0_f64];
    for i in 1..bruta.len() {
        let d = (bruta[i][0] - bruta[i - 1][0]).hypot(bruta[i][1] - bruta[i - 1][1]);
        acum.push(acum[i - 1] + d);
    }
    let total = *acum.last().expect("acum nao e' vazio");
    if total <= f64::EPSILON {
        return bruta;
    }
    let mut out = Vec::with_capacity(N);
    let mut j = 0;
    for k in 0..N {
        #[expect(clippy::cast_precision_loss, reason = "k < N, um punhado")]
        let alvo = k as f64 / N as f64 * total;
        while j + 1 < acum.len() && acum[j + 1] < alvo {
            j += 1;
        }
        let (a, b) = (acum[j], acum[(j + 1).min(acum.len() - 1)]);
        let t = if (b - a).abs() <= f64::EPSILON {
            0.0
        } else {
            (alvo - a) / (b - a)
        };
        let (p, q) = (bruta[j], bruta[(j + 1).min(bruta.len() - 1)]);
        out.push([t.mul_add(q[0] - p[0], p[0]), t.mul_add(q[1] - p[1], p[1])]);
    }
    out
}

/// O desenho COZIDO da barra, achatado — a matéria-prima da [`arte`].
fn achatada(scene: &ph2d_vec_scene::VecScene) -> Vec<[f64; 2]> {
    const N: usize = 64;
    let p = &scene.paths()[0];
    let cozido = p.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            for k in 0..N {
                #[expect(clippy::cast_precision_loss, reason = "k < N, um punhado")]
                let t = k as f64 / N as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w3.mul_add(
                        b.anchor[0],
                        w2.mul_add(b.in_handle[0], w1.mul_add(a.out_handle[0], w0 * a.anchor[0])),
                    ),
                    w3.mul_add(
                        b.anchor[1],
                        w2.mul_add(b.in_handle[1], w1.mul_add(a.out_handle[1], w0 * a.anchor[1])),
                    ),
                ]);
            }
        }
    }
    out
}

/// O maior afastamento entre duas amostragens da mesma arte, ponto a ponto.
fn desvio(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    assert_eq!(a.len(), b.len(), "as duas amostragens tem de ter o mesmo N");
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0, f64::max)
}
