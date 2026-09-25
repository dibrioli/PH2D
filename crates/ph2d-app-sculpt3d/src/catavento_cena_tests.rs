//! **OS GATES DA CENA `=53`** — o catavento, do lado que não precisa de placa.
//!
//! ⚠️ Os dois medem coisas que a suíte inteira deixaria passar e que só o DONO veria, cada uma
//! como uma cena que ensina o contrário do que diz (a espécie que o `CLAUDE.md` §5.0 chama de
//! *pior que uma cena ausente*).

use crate::scenes::GIRO_DA_CENA;

/// ⭐⭐⭐ **A PEÇA DA CENA NÃO PODE SER INVARIANTE À ROTAÇÃO** — o controlo da própria cena.
///
/// ⛔ Uma esfera LISA tem raio constante, logo rodá-la em torno do centro devolve **a mesma
/// superfície**: o catavento giraria e a imagem não mudaria, e a `=53` ensinaria que a rota B não
/// faz nada. *O que separa uma cena que prova a wave de uma que a desmente é esta propriedade da
/// MALHA, e ela não está escrita em lado nenhum senão aqui.*
///
/// ⚠️ **A régua é o RAIO e não um render**: um campo de raios constante é a definição de
/// invariância a toda rotação em torno do centro — exacto, barato, e sem GPU. A barra (`10 %` do
/// raio médio) sai do vale medido: a `ridged_sphere` lê `0,262` e uma esfera lisa lê `0,000`.
///
/// **Mutação que deve sangrar:** `catavento_scene()` fora da lista do `scenes_mesh.rs` que devolve
/// a `ridged_sphere` (a cena cai na esfera lisa do default).
#[test]
fn a_peca_do_catavento_tem_relevo_que_a_rotacao_revela() {
    let m = crate::fixtures::ridged_sphere();
    let centro = {
        let ps = m.positions();
        let n = ps.len() as f32;
        let s = ps
            .iter()
            .fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
        [s[0] / n, s[1] / n, s[2] / n]
    };
    let raios: Vec<f32> = m
        .positions()
        .iter()
        .map(|p| {
            let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        })
        .collect();
    assert!(
        raios.len() > 1_000,
        "a fixtura nao tem malha: {}",
        raios.len()
    );
    let medio = raios.iter().sum::<f32>() / raios.len() as f32;
    let (lo, hi) = raios
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &r| (a.min(r), b.max(r)));
    let excursao = (hi - lo) / medio;
    eprintln!("relevo da peca da =53: excursao {excursao:.3} do raio medio {medio:.3}");
    assert!(
        excursao > 0.10,
        "a peca da =53 e' quase esferica (excursao {excursao:.3}): rodar nao muda a imagem, e a \
         cena ensinaria que a rota B nao faz nada"
    );
    // ⚠️⚠️ **A SEGUNDA METADE, e sem ela a primeira afirma sobre uma fixtura que a cena pode não
    // usar:** medir o relevo da `ridged_sphere` não prova que é ela que a `=53` abre. O elo é a
    // lista do `scenes_mesh`, e ele é lido por texto porque escolher a malha lê a env — que esta
    // crate não pode armar (ela proíbe `unsafe`).
    let fonte = include_str!("scenes_mesh.rs");
    assert!(
        fonte.contains("|| catavento_scene()"),
        "a =53 saiu da lista que devolve a esfera com CRISTAS: ela passaria a abrir com a esfera \
         lisa do default, que e' invariante a' rotacao"
    );
}

/// ⭐⭐ **A CENA SEMEIA UM CATAVENTO QUE GIRA — e o componente de fábrica fica PARADO.**
///
/// ⚠️ **As duas metades, e cada uma sozinha mente:** só a primeira e alguém poria o giro no
/// `Default` do componente, e toda peça do app passaria a girar; só a segunda e a cena semearia um
/// catavento imóvel, que se lê exactamente como a rota B partida.
///
/// ⚠️ A terceira metade é de TEXTO e não de execução: esta crate proíbe `unsafe`, logo um teste
/// não pode armar a env para medir que é a `=53` **e não outra cena** que pede o catavento. O que
/// ele afirma é a FIAÇÃO — que a porta com a lei é guardada pelo predicado da cena —, e quem mede
/// o predicado em si é o censo do roteador.
///
/// **Mutação que deve sangrar:** `spin: super::scenes::GIRO_DA_CENA` → `spin: 0.0` no
/// `donation::catavento_da_cena`.
#[test]
fn a_cena_pede_giro_e_o_componente_nasce_parado() {
    assert_eq!(
        ph2d_ecs::Mesh3D::default().spin,
        0.0,
        "um Mesh3D de fabrica tem de nascer PARADO: o giro e' da cena, nunca do componente"
    );
    let pedido = crate::donation::catavento_da_cena();
    assert!(
        pedido.spin.abs() > 0.0,
        "a cena =53 semeia um catavento PARADO — ela mostraria a rota B a nao fazer nada"
    );
    assert!(
        (pedido.spin - GIRO_DA_CENA).abs() < f32::EPSILON,
        "o giro da cena tem de ser o da constante que o doc mede"
    );
    let fonte = include_str!("donation.rs");
    assert!(
        fonte.contains("super::scenes::catavento_scene().then(catavento_da_cena)"),
        "a lei do catavento tem de ser guardada pelo predicado da CENA: sem essa linha toda tela \
         do app nasceria a girar"
    );
}

/// ⭐⭐⭐ **O ROTEIRO MANDA DAR PLAY, LOGO A CENA TEM DE ABRIR A RÉGUA DO TEMPO.**
///
/// ⛔⛔ **Achado por uma FOTO e não por um gate** (21/09): a `=53` abria **sem timeline nenhuma**,
/// o passo (5) mandava *«dê PLAY (a régua do tempo)»*, e sem transporte a andar o `playhead` fica
/// em `0` ⇒ **o catavento nunca gira**. *O dono julgaria a wave sem nunca a ver*, e os seis gates
/// da cena estavam verdes — eles medem a lei, e o que faltava era um PAINEL.
///
/// ⚠️ **As duas metades, e cada uma sozinha mente:** o roteiro pode deixar de pedir Play (e a
/// visibilidade vira ruído) ou a cena pode deixar de abrir a régua (e o passo vira impossível).
///
/// **Mutação que deve sangrar:** apagar o `hero.panel_visibility.insert("timeline", true)` da
/// `fase_sculpt3d_donation_smoke`.
#[test]
fn o_roteiro_pede_play_e_a_cena_abre_a_regua() {
    let roteiro = include_str!("scripts.rs");
    // ⚠️ **A âncora é o NÚMERO da cena e não o título**, e a razão foi medida em 21/09: ao
    // traduzir o roteiro para inglês (ordem do dono — *«o app é em inglês»*) este gate reprovou,
    // porque procurava `=53 O CATAVENTO`. *Uma agulha que contém prosa traduzível reprova na
    // tradução, e o defeito que ela existe para apanhar continua lá.*
    let i = roteiro
        .find("=53 ")
        .expect("o roteiro da =53 tem de existir");
    let bloco = &roteiro[i..i + 1800.min(roteiro.len() - i)];
    assert!(
        bloco.contains("PLAY"),
        "o roteiro da =53 deixou de pedir PLAY — se o giro passou a ser visível sem transporte, \
         esta metade e a da visibilidade da régua deixam as duas de descrever o produto"
    );
    // ⚠️ A outra ponta vive na SHELL, que esta crate não compila — logo é lida por TEXTO, que é a
    // mesma forma que os gates de fiação desta casa usam quando as duas metades não se encontram
    // num teste. O caminho é relativo a ESTE ficheiro.
    let cena =
        include_str!("../../../shells/desktop/src/render_loop/fase_sculpt3d_donation_smoke.rs");
    assert!(
        cena.contains(r#"hero.panel_visibility.insert("timeline", true)"#),
        "a cena =53 deixou de abrir a regua do tempo: o passo (5) do roteiro manda carregar num \
         botao que nao esta' na tela, e o catavento nunca gira"
    );
}
