//! ⭐⭐⭐ **OS GATES DOS DOIS MODOS DE ATRIBUIR PESO** (F29, ordem do dono de 2026-09-19).
//!
//! ⚠️ **Irmão do [`super::peso_a_mao_tests`] por ASSUNTO:** aquele mede o GESTO (onde a mancha
//! pousa, as três recusas, o tecto) e este mede o que a ESPÉCIE dela muda — a fusão, a recência e o
//! peso que a arte acaba por ter.
//!
//! ⛔ **A LEI das duas espécies é da [`ph2d_skeleton`] e está gateada lá**
//! ([`ph2d_skeleton::Especie`]). Aqui mede-se só o que existe deste lado: a LISTA guardada e o
//! caminho do produto.

use crate::barra_da_cena_tests_support::{PPM, barra_da_cena, forma, raio_de_fabrica};
use crate::peso_a_mao::{Pincelada, pinta, pontos_de_peso};
use ph2d_skeleton::Especie;

fn manchas(
    sim: &ph2d_ecs::SimWorld,
    alvo: ph2d_ecs::Entity,
) -> Vec<ph2d_skeleton_ecs::CorreccaoDePeso> {
    sim.world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .map(|s| s.correcoes.clone())
        .unwrap_or_default()
}

/// ⭐⭐⭐ **UMA ABSOLUTA NÃO SE FUNDE NUMA CUMULATIVA** — elas respondem perguntas diferentes.
///
/// ⚠️ **Sem esta cerca a segunda pincelada era engolida pela primeira:** o artista trocava de modo,
/// pintava no mesmo sítio, e o que ficava guardado era uma `Soma` com o número dele lá dentro — *a
/// escolha que ele acabou de fazer desaparecia sem aviso*.
///
/// (Mutação: apagar o `discriminant` da busca ⇒ RED aqui, com UMA mancha.)
#[test]
fn uma_absoluta_nao_se_funde_numa_cumulativa() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();
    let no = pontos_de_peso(&sim, alvo, ossos[1], PPM)
        .first()
        .map(|p| p.mundo)
        .expect("a barra tem nos");

    assert!(matches!(
        pinta(&mut sim, alvo, ossos[1], PPM, no, raio, Especie::Soma(0.2)),
        Pincelada::Pintada { .. }
    ));
    assert!(matches!(
        pinta(&mut sim, alvo, ossos[1], PPM, no, raio, Especie::Alvo(0.7)),
        Pincelada::Pintada { .. }
    ));

    let cs = manchas(&sim, alvo);
    assert_eq!(
        cs.len(),
        2,
        "as duas especies fundiram-se numa so': {cs:?} — a escolha do artista evaporou"
    );
    assert!(
        matches!(cs[0].especie, Especie::Soma(_)) && matches!(cs[1].especie, Especie::Alvo(_)),
        "a ordem ou as especies da lista nao sao as que o artista pintou: {cs:?}"
    );
}

/// ⭐⭐⭐ **INSISTIR NUMA ABSOLUTA NÃO EMPURRA MAIS** — é isso que «absoluto» quer dizer.
///
/// ⚠️ **É a metade OPOSTA da lei cumulativa**, e o gate irmão
/// (`uma_segunda_pincelada_no_mesmo_sitio_funde_e_empurra_mais`) mede a outra: ali insistir soma,
/// aqui insistir **repõe**. *As duas leis sobre o mesmo gesto, e o modo é quem escolhe.*
///
/// (Mutação: somar em vez de substituir no braço `Alvo` ⇒ RED com `1,4`.)
#[test]
fn insistir_numa_absoluta_nao_empurra_mais() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();
    let no = pontos_de_peso(&sim, alvo, ossos[1], PPM)
        .first()
        .map(|p| p.mundo)
        .expect("a barra tem nos");

    for _ in 0..5 {
        pinta(&mut sim, alvo, ossos[1], PPM, no, raio, Especie::Alvo(0.7));
    }
    let cs = manchas(&sim, alvo);
    assert_eq!(
        cs.len(),
        1,
        "cinco pinceladas no mesmo sitio deixaram {} manchas",
        cs.len()
    );
    assert!(
        matches!(cs[0].especie, Especie::Alvo(v) if (v - 0.7).abs() < 1e-12),
        "o valor absoluto deixou de ser 0,7 depois de cinco pinceladas: {:?}",
        cs[0].especie
    );
}

/// ⭐⭐⭐ **RE-PINTAR UMA ABSOLUTA SOBE-A PARA O FIM DA LISTA** — *«a última manda»* só é verdade se
/// a última **pintada** for a última **da lista**.
///
/// ⛔⛔ **Sem isto o artista carregava e via menos do que pediu:** voltar a uma mancha antiga
/// actualizava o valor dela e deixava-a **antes** de uma vizinha mais nova, que continuava a puxar
/// o ponto. A lei aplica a lista por ORDEM (ver [`ph2d_skeleton::Especie::Alvo`]), logo a posição
/// **é** a recência.
///
/// ⚠️ **A metade do CONTROLO é obrigatória:** sem provar que as duas primeiras NÃO se fundiram, o
/// teste passaria sobre uma lista de uma mancha só, onde a ordem não significa nada.
///
/// (Mutação: apagar o `remove`/`push` do braço `Alvo` ⇒ RED aqui.)
#[test]
fn re_pintar_uma_absoluta_sobe_a_para_o_fim_da_lista() {
    let bone = ph2d_ecs::StableId(7);
    let mut lista = Vec::new();
    let raio = 40.0;
    // ⚠️ `60` é maior que a distância de fusão (`FUSAO · raio = 20`) ⇒ elas NÃO se juntam.
    crate::peso_a_mao::funde(&mut lista, bone, [0.0, 0.0], raio, Especie::Alvo(0.9));
    crate::peso_a_mao::funde(&mut lista, bone, [60.0, 0.0], raio, Especie::Alvo(0.1));
    assert_eq!(
        lista.len(),
        2,
        "o CONTROLO caiu: as duas fundiram-se e a ordem deixa de significar"
    );
    assert!((lista[1].centro[0] - 60.0).abs() < 1e-9);

    // Voltar à primeira: ela funde-se (distância `5` < `20`) e tem de SUBIR.
    crate::peso_a_mao::funde(&mut lista, bone, [5.0, 0.0], raio, Especie::Alvo(0.4));
    assert_eq!(
        lista.len(),
        2,
        "a re-pintura criou uma mancha em vez de se fundir"
    );
    assert!(
        (lista[1].centro[0] - 0.0).abs() < 1e-9,
        "a mancha re-pintada nao subiu para o fim da lista: {lista:?}"
    );
    assert!(
        matches!(lista[1].especie, Especie::Alvo(v) if (v - 0.4).abs() < 1e-12),
        "o valor da mancha re-pintada nao foi substituido: {:?}",
        lista[1].especie
    );
}

/// ⚠️ **E uma CUMULATIVA re-pintada fica ONDE ESTÁ** — a assimetria é deliberada e está no doc do
/// [`crate::peso_a_mao::funde`].
///
/// Entre `Soma`s a ordem é quase irrelevante (elas somam num acumulador; só o `clamp` a torna
/// observável), e mexer nela mudaria bits da lei que o dono já aprovou em smoke **sem comprar
/// nada**. ⛔ *Isto é um gate sobre uma DECISÃO, não sobre uma lei do domínio* — quem a mudar tem de
/// o fazer em voz alta.
#[test]
fn uma_cumulativa_re_pintada_fica_onde_esta() {
    let bone = ph2d_ecs::StableId(7);
    let mut lista = Vec::new();
    let raio = 40.0;
    crate::peso_a_mao::funde(&mut lista, bone, [0.0, 0.0], raio, Especie::Soma(0.2));
    crate::peso_a_mao::funde(&mut lista, bone, [60.0, 0.0], raio, Especie::Soma(0.2));
    crate::peso_a_mao::funde(&mut lista, bone, [5.0, 0.0], raio, Especie::Soma(0.2));
    assert_eq!(lista.len(), 2);
    assert!(
        (lista[0].centro[0] - 0.0).abs() < 1e-9,
        "a mancha cumulativa re-pintada MUDOU de posicao na lista: {lista:?}"
    );
    assert!(
        matches!(lista[0].especie, Especie::Soma(v) if (v - 0.4).abs() < 1e-12),
        "ela devia ter SOMADO para 0,4: {:?}",
        lista[0].especie
    );
}

/// ⭐⭐⭐ **O MODO ABSOLUTO CHEGA À ARTE** — o gesto inteiro, pela porta do produto.
///
/// ⚠️⚠️ **Esta é a metade que os gates da lei não podem dar:** eles correm a
/// [`ph2d_skeleton::Skin`] directamente, e *uma paridade medida a montante de uma conversão não
/// afirma nada sobre a conversão* — a lição que a ponte da curva do pincel de contorno pagou duas
/// vezes nesta casa. Aqui o peso é lido pela MESMA porta que o quadro usa.
///
/// ⛔ **O CONTROLO vem primeiro:** sem provar que o peso de partida está longe do pedido, isto
/// passaria sobre um pincel inerte.
#[test]
fn o_modo_absoluto_poe_o_peso_pedido_na_arte() {
    let (mut sim, _scene, map, id, ossos) = barra_da_cena();
    let alvo = forma(&map, id);
    let raio = raio_de_fabrica();
    // ⚠️ O nó onde o osso do MEIO não manda sozinho — senão `Σoutros == 0` e a lei do dono torna o
    // gesto um no-op (que é outro gate, na crate da lei).
    let (no, antes) = pontos_de_peso(&sim, alvo, ossos[1], PPM)
        .into_iter()
        .map(|p| (p.mundo, p.peso))
        .find(|(_, w)| *w > 0.05 && *w < 0.9)
        .expect("a barra tem um no' partilhado entre dois ossos");
    assert!(
        (antes - 0.75).abs() > 0.1,
        "o no' ja' nasce em {antes}, perto do alvo — o gate nao mede nada"
    );

    let r = pinta(&mut sim, alvo, ossos[1], PPM, no, raio, Especie::Alvo(0.75));
    assert!(matches!(r, Pincelada::Pintada { .. }), "{r:?}");

    let depois = pontos_de_peso(&sim, alvo, ossos[1], PPM)
        .into_iter()
        .map(|p| (p.mundo, p.peso))
        .find(|(q, _)| (q[0] - no[0]).hypot(q[1] - no[1]) < 1e-9)
        .map_or(0.0, |(_, w)| w);
    assert!(
        (depois - 0.75).abs() < 1e-6,
        "o peso do no' pintado devia ser 0,75 e leu {depois} (era {antes})"
    );
}
