//! Os gates do [`crate::peso_a_mao`] — a MANCHA que o artista pinta.
//!
//! A **lei** da correcção (a bossa `(1−x²)²`, a renormalização, o no-op da lista vazia) é do módulo
//! ([`ph2d_skeleton`]) e está gateada lá. Aqui mede-se o que só existe com um mundo ECS: a
//! **tradução pose → repouso** que é a razão de o gesto existir, as três recusas, a fusão, o tecto,
//! e que o olho lê a MESMA lei que o quadro.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder, Transform};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook};

/// O `pixels_per_meter` do projecto. Só a mídia IMAGEM o lê; nas fixturas de forma é inerte.
const PPM: f32 = 100.0;

/// Uma cena com UMA forma (rectângulo de `(0,0)` a `(40,10)`) e um esqueleto de dois ossos ao
/// longo dela — a mesma do [`crate::skin_live_tests`], e de propósito: *duas fixturas do mesmo
/// palco divergem no primeiro ajuste*.
fn palco() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    (sim, scene, map, id, [raiz, ponta])
}

/// ⭐ **O MESMO palco com uma ESTRELA no lugar do rectângulo** — a fixtura do ARRASTO.
///
/// ⚠️ **Ela existe porque um rectângulo tem QUATRO pontos**, e um arrasto sobre quatro pontos não
/// mede o que uma lista de manchas faz quando o artista anda com o pincel. *Uma régua de arrasto
/// precisa de arte por onde andar* — e a estrela de dez pontas dá-a sem inventar geometria.
fn palco_estrela() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::Star, [0.0, 0.0], [40.0, 40.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 20.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    (sim, scene, map, id, [raiz, ponta])
}

fn osso(sim: &mut SimWorld, nome: &str, pos: [f32; 2], len: f64, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            ph2d_skeleton_ecs::Bone {
                length: len,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// A entidade da forma.
fn forma(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(*map.get(&id).expect("a forma tem entidade"))
}

fn gira(sim: &mut SimWorld, e: Entity, graus: f32) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .rotation = graus.to_radians();
}

/// ⭐⭐⭐ **A MANCHA É ANCORADA NO REPOUSO DO PONTO QUE O DEDO APONTA, e não no cursor.**
///
/// Esta é a razão de existir do módulo. Com o esqueleto POSADO, o ponto que o artista vê está longe
/// de onde ele vive no repouso — guardar o cursor cru poria a correcção no sítio errado, e ela
/// andaria com a pose no quadro seguinte.
///
/// ⚠️ **O CONTROLO é a metade que torna a medição legível:** na pose de repouso os dois coincidem,
/// logo o desvio medido na pose dobrada é do MAPA e não da fixtura.
///
/// (Mutação: guardar `mundo` em vez de `ponto.repouso` ⇒ RED com `2,4e+1`.)
#[test]
fn a_mancha_e_ancorada_no_repouso_e_nao_no_cursor() {
    let (mut sim, mut scene, map, id, ossos) = palco();
    let alvo = forma(&map, id);
    // Onde vive, no REPOUSO, a quina direita de baixo do rectângulo.
    // ⚠️ **Um VÉRTICE, e não um ponto qualquer da aresta:** a pele deforma os pontos que o
    // desenho TEM, e `(40, 5)` — o meio da aresta — não é um deles. *Uma régua que nomeia um
    // ponto que a arte não tem mede a distância dele ao vizinho mais perto e chama-lhe erro.*
    let repouso_da_ponta = [40.0, 0.0];

    // (a) CONTROLO — sem pose, o que se vê é o que se guarda.
    let vivo = ponto_de(&sim, alvo, ossos[1], repouso_da_ponta).mundo;
    assert!(
        dist(vivo, repouso_da_ponta) < 1.0,
        "no repouso o ponto vivo devia coincidir com o de repouso, e leu {vivo:?}"
    );

    // (b) A pose leva a ponta para longe; a mancha tem de ficar no repouso dela.
    gira(&mut sim, ossos[1], 90.0);
    crate::skin_live::recook(&sim, &mut scene);
    let posado = ponto_de(&sim, alvo, ossos[1], repouso_da_ponta).mundo;
    let afastou = dist(posado, repouso_da_ponta);
    assert!(
        afastou > 5.0,
        "a fixtura nao dobra: o ponto posado esta' a {afastou} do repouso, e o gate nao mede nada"
    );
    let r = crate::peso_a_mao::pinta(&mut sim, alvo, ossos[1], PPM, posado, 4.0, 0.5);
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "{r:?}"
    );
    let c = manchas(&sim, alvo);
    assert_eq!(c.len(), 1);
    let erro = dist(c[0].centro, repouso_da_ponta);
    assert!(
        erro < 1.0,
        "a mancha foi guardada em {:?}, a {erro} do repouso do ponto apontado",
        c[0].centro
    );
}

/// ⭐⭐⭐ **PINTAR MUDA O PESO ALI, E SÓ ALI** — a metade que prova que o gesto chega à lei.
///
/// ⚠️ **As duas metades são obrigatórias:** sem a segunda, uma mancha de raio infinito passaria (ela
/// mudaria o peso do ponto apontado *e* do resto da arte), e o que o artista quer é o oposto.
///
/// (Mutações: `delta = 0` ⇒ RED na 1.ª; a mancha com `raio` do desenho inteiro ⇒ RED na 2.ª.)
#[test]
fn pintar_muda_o_peso_naquele_ponto_e_so_ali() {
    let (mut sim, _scene, map, id, ossos) = palco();
    let alvo = forma(&map, id);
    let perto = [40.0, 0.0];
    let longe = [0.0, 0.0];
    let antes_perto = peso_em(&sim, alvo, ossos[0], perto);
    let antes_longe = peso_em(&sim, alvo, ossos[0], longe);

    let r = crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, perto, 4.0, 0.8);
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "{r:?}"
    );

    let depois_perto = peso_em(&sim, alvo, ossos[0], perto);
    let depois_longe = peso_em(&sim, alvo, ossos[0], longe);
    assert!(
        depois_perto - antes_perto > 0.05,
        "o peso onde se pintou foi de {antes_perto} para {depois_perto} - o gesto nao chega a` lei"
    );
    assert!(
        (depois_longe - antes_longe).abs() < 1e-12,
        "a mancha alcancou o outro lado da arte: {antes_longe} -> {depois_longe}"
    );
}

/// ⭐⭐ **INSISTIR NO MESMO SÍTIO EMPURRA MAIS, e continua a ser UMA mancha.**
///
/// ⚠️⚠️ **Esta metade NÃO mede a [`FUSAO`], e a distinção foi paga por uma mutação SOBREVIVENTE:**
/// o centro de uma mancha é *o ponto da pele mais perto*, logo duas pinceladas no mesmo sítio
/// caem no **mesmo ponto** e fundem por igualdade EXACTA — com `FUSAO = 0` este gate continua
/// verde. Quem mede a constante é a irmã [`a_fusao_junta_manchas_de_pontos_vizinhos`].
///
/// (Mutação: `c.delta = delta` em vez de somar ⇒ RED no empurrão.)
#[test]
fn uma_segunda_pincelada_no_mesmo_sitio_funde_e_empurra_mais() {
    let (mut sim, _scene, map, id, ossos) = palco();
    let alvo = forma(&map, id);
    let p = [40.0, 0.0];
    crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, p, 4.0, 0.2);
    let um = manchas(&sim, alvo);
    crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, p, 4.0, 0.2);
    let dois = manchas(&sim, alvo);
    assert_eq!(
        dois.len(),
        1,
        "a 2.a pincelada no mesmo sitio criou outra mancha"
    );
    assert!(
        dois[0].delta > um[0].delta + 0.1,
        "o delta nao somou: {} -> {}",
        um[0].delta,
        dois[0].delta
    );
    // ⚠️ E ele **satura**: o peso vive em `0..1`, logo guardar `+5` mentiria sobre quanto falta
    // para desfazer.
    for _ in 0..20 {
        crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, p, 4.0, 0.5);
    }
    assert!(
        (manchas(&sim, alvo)[0].delta - 1.0).abs() < 1e-12,
        "o delta nao saturou em 1: {}",
        manchas(&sim, alvo)[0].delta
    );
}

/// ⭐⭐⭐ **A FUSÃO JUNTA MANCHAS DE PONTOS VIZINHOS** — a lei da [`crate::peso_a_mao::FUSAO`].
///
/// ⛔⛔ **Ela existe porque a régua irmã NÃO a media.** Com o centro snapado ao ponto da pele mais
/// perto, duas pinceladas no MESMO sítio fundem por igualdade exacta — a constante só decide
/// quando o dedo apanha **pontos DIFERENTES**, que é o que acontece num arrasto sobre uma malha
/// densa. *Uma mutação que sobrevive é a régua a dizer onde ela não olha.*
///
/// ⚠️ **O raio é DERIVADO da fixtura** (`3 ×` a distância entre os dois pontos escolhidos) e não um
/// número: com um raio fixo, adensar a estrela um dia poria os dois pontos fora da janela de fusão
/// e o gate passaria a medir outra coisa.
///
/// (Mutação: `FUSAO = 0.0` ⇒ RED com `2` manchas.)
#[test]
fn a_fusao_junta_manchas_de_pontos_vizinhos() {
    let (mut sim, _scene, map, id, ossos) = palco_estrela();
    let alvo = forma(&map, id);
    // Dois pontos DISTINTOS da pele, os mais próximos um do outro que a fixtura tem.
    let pts = crate::peso_a_mao::pontos_da_pele(&sim, alvo, ossos[0], PPM);
    let mut par: Option<(f64, [f64; 2], [f64; 2])> = None;
    for (i, a) in pts.iter().enumerate() {
        for b in &pts[i + 1..] {
            let d = dist(a.repouso, b.repouso);
            if d > 1e-9 && par.is_none_or(|(m, _, _)| d < m) {
                par = Some((d, a.mundo, b.mundo));
            }
        }
    }
    let (d, a, b) = par.expect("a estrela tem ao menos dois pontos distintos");
    // ⚠️ `3 × d` ⇒ a janela de fusão (`FUSAO × raio`) vale `1,5 × d` e cobre o par — e o `d` é
    // MEDIDO na fixtura, não escolhido.
    let raio = 3.0 * d;
    crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, a, raio, 0.2);
    crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, b, raio, 0.2);
    let n = manchas(&sim, alvo).len();
    assert_eq!(
        n,
        1,
        "duas pinceladas a {d} uma da outra, com janela de fusao {}, deixaram {n} manchas",
        crate::peso_a_mao::FUSAO * raio
    );
}

/// ⭐⭐⭐ **AS TRÊS RECUSAS SÃO DISTINTAS** — *«a ferramenta não faz nada»* tem três curas, e elas são
/// outras.
///
/// (Mutação: colapsar qualquer par num só valor ⇒ RED.)
#[test]
fn as_tres_recusas_dizem_qual_entrada_falta() {
    use crate::peso_a_mao::Pincelada;
    let (mut sim, mut scene, mut map, id, ossos) = palco();
    let alvo = forma(&map, id);

    // (a) SEM PELE — uma forma que ninguém prendeu.
    let solta = scene.push_path(cook(ShapeKind::Rectangle, [80.0, 0.0], [90.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let e_solta = forma(&map, solta);
    assert_eq!(
        crate::peso_a_mao::pinta(&mut sim, e_solta, ossos[0], PPM, [85.0, 5.0], 4.0, 0.5),
        Pincelada::SemPele
    );

    // (b) OSSO DE FORA — um osso de outro esqueleto, que esta pele nunca viu.
    let estranho = osso(&mut sim, "Alheio", [200.0, 0.0], 5.0, None);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert_eq!(
        crate::peso_a_mao::pinta(&mut sim, alvo, estranho, PPM, [40.0, 0.0], 4.0, 0.5),
        Pincelada::OssoDeFora
    );

    // (c) FORA DA ARTE — o dedo longe de todo ponto da pele.
    assert_eq!(
        crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, [999.0, 999.0], 4.0, 0.5),
        Pincelada::ForaDaArte
    );
}

/// ⭐⭐ **O RAIO É CONVERTIDO PELA ESCALA DA COISA** — o artista pinta em unidades do que vê, e o
/// que se guarda vive no espaço do bind.
///
/// ⛔ Sem a conversão, uma forma a metade da escala recebia uma mancha do DOBRO do tamanho que o
/// pincel mostrava — e o artista veria a correcção alastrar para fora do anel.
///
/// (Mutação: `raio = raio_mundo` ⇒ RED com `4` contra `2`.)
#[test]
fn o_raio_e_convertido_pela_escala_da_coisa() {
    let (mut sim, scene, map, id, ossos) = palco();
    let alvo = forma(&map, id);
    // A forma passa a desenhar-se ao DOBRO: um raio de 4 em mundo é 2 no bind.
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(alvo)
            .expect("Transform");
        t.scale = ph2d_core::Vec2::new(2.0, 2.0);
    }
    drop(scene);
    let ponta = ponto_de(&sim, alvo, ossos[1], [40.0, 0.0]).mundo;
    let r = crate::peso_a_mao::pinta(&mut sim, alvo, ossos[1], PPM, ponta, 4.0, 0.5);
    assert!(
        matches!(r, crate::peso_a_mao::Pincelada::Pintada { .. }),
        "{r:?}"
    );
    let c = manchas(&sim, alvo);
    assert!(
        (c[0].raio - 2.0).abs() < 1e-9,
        "o raio guardado foi {} e a escala da coisa e' 2",
        c[0].raio
    );
}

/// ⭐⭐ **UM ARRASTO NUNCA PASSA DO TECTO** — a lei do [`MANCHAS_MAX`].
///
/// ⚠️ **A metade do PISO é o que impede a cura barata:** um tecto de `1` passaria o `<=` e apagaria
/// o trabalho do artista a cada passo; o gate exige que a varredura deixe rasto.
///
/// ⭐ **E o piso mede também a FUSÃO a fazer o trabalho dela:** `1 600` pinceladas sobre a arte
/// deixam DEZENAS de manchas e não milhares, porque o centro de cada uma é *o ponto da pele mais
/// perto* — logo duas pinceladas que caem no mesmo ponto são a mesma mancha, por construção.
///
/// (Mutação: apagar o corte do tecto ⇒ RED com o número de pontos distintos da estrela.)
#[test]
fn pintar_por_toda_a_arte_nunca_passa_do_tecto() {
    let (mut sim, _scene, map, id, ossos) = palco_estrela();
    let alvo = forma(&map, id);
    // Uma varredura da caixa inteira da estrela — ⚠️ e não uma linha: ao longo de UMA linha a
    // arte tem poucos pontos, e a régua mediria a fixtura em vez da lei.
    for kx in 0..40 {
        for ky in 0..40 {
            let p = [f64::from(kx), f64::from(ky)];
            crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, p, 1.5, 0.2);
        }
    }
    let n = manchas(&sim, alvo).len();
    assert!(
        n <= crate::peso_a_mao::MANCHAS_MAX,
        "a varredura deixou {n} manchas, acima do tecto de {}",
        crate::peso_a_mao::MANCHAS_MAX
    );
    assert!(n > 4, "a varredura deixou {n} manchas - o rasto evaporou");
}

/// ⭐⭐⭐ **O TECTO SEGURA, E QUEM CEDE O LUGAR É A MANCHA MAIS ANTIGA DO MESMO OSSO.**
///
/// ⛔⛔ **A régua irmã (a varredura) NÃO mede isto, e foi uma mutação SOBREVIVENTE que o disse:**
/// a estrela tem umas dezenas de pontos distintos, logo a lista **nunca chega ao tecto** e apagar
/// o corte deixava-a verde. *Um tecto que a fixtura não alcança é um tecto que nenhum gate afirma*
/// — e uma peça com centenas de pontos (a malha graduada de uma imagem) alcança-o à primeira.
///
/// ⚠️ **Ele mede a LEI directamente** ([`crate::peso_a_mao::funde`]) e não o produto, e é a
/// granularidade certa: o tecto é uma propriedade da LISTA, e construir uma arte com `> 128` pontos
/// distintos só para lá chegar mediria o cozimento da estrela.
///
/// ⚠️ **A 2.ª metade é a que impede a cura barata:** um `truncate` no fim da lista respeitaria o
/// tecto e apagaria a pincelada que o artista acabou de dar.
///
/// (Mutação: apagar o corte ⇒ RED na 1.ª, com `200`. `lista.pop()` ⇒ RED na 2.ª.)
#[test]
fn o_tecto_segura_e_quem_cede_e_a_mancha_mais_antiga() {
    use crate::peso_a_mao::MANCHAS_MAX;
    let bone = ph2d_ecs::StableId(7);
    let mut lista = Vec::new();
    for k in 0..(MANCHAS_MAX + 72) {
        let x = f64::from(u32::try_from(k).unwrap_or(0));
        // ⚠️ Cada centro a `100` do vizinho ⇒ **nenhuma** funde, e o que se mede é só o tecto.
        crate::peso_a_mao::funde(&mut lista, bone, [x * 100.0, 0.0], 1.0, 0.1);
    }
    assert!(
        lista.len() <= MANCHAS_MAX,
        "a lista chegou a {} manchas, acima do tecto de {MANCHAS_MAX}",
        lista.len()
    );
    // ⭐ A ÚLTIMA pincelada sobrevive — e a primeira não.
    let ultima = f64::from(u32::try_from(MANCHAS_MAX + 71).unwrap_or(0)) * 100.0;
    assert!(
        lista.iter().any(|c| (c.centro[0] - ultima).abs() < 1e-9),
        "a mancha que o artista acabou de pintar foi a que cedeu o lugar"
    );
    assert!(
        !lista.iter().any(|c| c.centro[0].abs() < 1e-9),
        "a mancha mais ANTIGA sobreviveu ao tecto - a lista corta pelo lado errado"
    );
}

/// ⭐⭐⭐ **O OLHO LÊ A MESMA LEI QUE O QUADRO** — a pré-visualização mostra o peso **corrigido**.
///
/// ⛔ *Uma pré-visualização que ignora o trabalho feito é uma segunda resposta à mesma pergunta*, e
/// o artista pintaria duas vezes o que já estava pintado.
///
/// (Mutação: `weights` cru em vez de `weights_corrected` no [`crate::peso_a_mao::pontos_da_pele`]
/// ⇒ RED.)
#[test]
fn o_olho_le_o_peso_ja_corrigido() {
    let (mut sim, _scene, map, id, ossos) = palco();
    let alvo = forma(&map, id);
    let p = [40.0, 0.0];
    let antes = peso_visto(&sim, alvo, ossos[0], p);
    crate::peso_a_mao::pinta(&mut sim, alvo, ossos[0], PPM, p, 4.0, 0.8);
    let depois = peso_visto(&sim, alvo, ossos[0], p);
    assert!(
        depois - antes > 0.05,
        "o olho continua a ver {antes} depois de a mancha levar o peso a {depois}"
    );
}

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
            delta: 0.05,
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

/// ⭐ **O ponto da pele cujo REPOUSO está mais perto de `alvo_repouso`** — a régua que permite
/// nomear um ponto da arte sem depender de onde a pose o pôs.
///
/// ⚠️ Ela procura pelo REPOUSO de propósito: procurar pelo mundo seria usar a grandeza que o gate
/// está a medir.
fn ponto_de(
    sim: &SimWorld,
    alvo: Entity,
    osso: Entity,
    alvo_repouso: [f64; 2],
) -> crate::peso_a_mao::PontoDaPele {
    crate::peso_a_mao::pontos_da_pele(sim, alvo, osso, PPM)
        .into_iter()
        .min_by(|a, b| {
            dist(a.repouso, alvo_repouso)
                .partial_cmp(&dist(b.repouso, alvo_repouso))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("a pele tem pontos")
}

fn manchas(sim: &SimWorld, alvo: Entity) -> Vec<ph2d_skeleton_ecs::CorreccaoDePeso> {
    sim.world()
        .get::<ph2d_skeleton_ecs::SkinBind>(alvo)
        .map(|s| s.correcoes.clone())
        .unwrap_or_default()
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dy.mul_add(dy, dx * dx).sqrt()
}

/// O peso do osso no ponto de repouso mais perto de `p`, lido pela porta do PRODUTO.
fn peso_em(sim: &SimWorld, alvo: Entity, osso: Entity, p: [f64; 2]) -> f64 {
    peso_visto(sim, alvo, osso, p)
}

/// ⭐⭐⭐ **O INDICADOR MOSTRA A ARTE DESTE TRAÇO — e o traço PERTENCE À ARTE EM QUE COMEÇOU.**
///
/// ⚠️ **Esta lei vivia no laço de desenho da shell, onde teste nenhum lhe chega** — e ela tem
/// quatro braços que se leem todos como *«apareceram pontos»*: o alvo congelado ganha do dedo · o
/// dedo escolhe quando não há traço · sem osso não há de quem mostrar peso · o dedo no vão não
/// mostra nada.
///
/// ⛔⛔ **A fixtura tem DUAS peles de propósito:** com uma só, «o congelado ganha» e «o dedo
/// escolhe» devolvem o MESMO bloco de pontos — *uma fixtura que não contém o fenómeno não prova
/// nada*, que é o defeito que esta jornada já pagou duas vezes.
#[test]
fn o_indicador_segue_a_arte_do_traco_e_nao_o_dedo() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let perto = scene.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    let longe = scene.push_path(cook(ShapeKind::Rectangle, [200.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    crate::skin_live::bind(&mut sim, &scene, &map, &[perto, longe], None);
    let (a, b) = (forma(&map, perto), forma(&map, longe));
    let sobre_b = [200.0, 0.0];
    let em_a = |v: &[([f64; 2], f64)]| v.iter().all(|(p, _)| p[0] < 100.0);

    // O dedo escolhe quando NÃO há traço: ele está sobre a `b`, e é a `b` que aparece.
    let pelo_dedo =
        crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(raiz), None, Some(sobre_b), 30.0);
    assert!(
        !pelo_dedo.is_empty() && !em_a(&pelo_dedo),
        "o dedo sobre a segunda arte devia mostrar a segunda arte: {} ponto(s)",
        pelo_dedo.len()
    );
    // ⭐ O DISCRIMINADOR: o mesmo dedo, com um traço preso na `a`, mostra a `a`.
    let preso =
        crate::peso_a_mao::pontos_do_indicador(&sim, PPM, Some(raiz), Some(a), Some(sobre_b), 30.0);
    assert!(
        !preso.is_empty() && em_a(&preso),
        "o traço começou na primeira arte e o indicador saltou para onde o dedo está"
    );
    assert_ne!(
        preso, pelo_dedo,
        "as duas metades devolveram o mesmo bloco — a fixtura não contém o fenómeno"
    );
    // Sem osso em foco não há de quem mostrar peso; e o dedo no vão não mostra nada.
    assert!(
        crate::peso_a_mao::pontos_do_indicador(&sim, PPM, None, Some(b), Some(sobre_b), 30.0)
            .is_empty(),
        "sem osso em foco o indicador inventou pesos"
    );
    assert!(
        crate::peso_a_mao::pontos_do_indicador(
            &sim,
            PPM,
            Some(raiz),
            None,
            Some([0.0, 5_000.0]),
            30.0
        )
        .is_empty(),
        "o dedo no vão devia não mostrar nada"
    );
}

fn peso_visto(sim: &SimWorld, alvo: Entity, osso: Entity, p: [f64; 2]) -> f64 {
    crate::peso_a_mao::ponto_sob_o_cursor(sim, alvo, osso, PPM, p).map_or(0.0, |q| q.peso)
}

/// ⭐⭐⭐ **A ESCALA DA BARRA DA CENA — a tabela de que o raio de fábrica do pincel foi derivado.**
///
/// ⛔⛔ **Ela existe porque o número que ela justifica estava errado por `2,85 ×`** (report do dono,
/// 2026-09-19: *«os pontos não ficam coloridos»*). O `WEIGHT_RADIUS_DEFAULT` era `20` em unidades
/// de MUNDO — `2 000` px — contra uma peça de `702` px: um clique agarrava todos os pontos dela.
///
/// ⚠️ **O sujeito é a peça REAL da cena** (o `RoundRect` do braço de `PH2D_VEC_BONE_SMOKE=1`), e
/// não um rectângulo de conveniência: *uma tabela medida noutra arte descreve outro programa*.
///
/// ⚠️ **Ela afirma os DOIS extremos e não só um:** o raio de fábrica tem de ser maior que a
/// distância entre dois pontos vizinhos (senão o pincel é um apontador) e muito menor que a peça
/// (senão ele não aponta a sítio nenhum). *Uma sonda que só imprimisse seria uma nota que
/// envelhece* — e esta reprova no dia em que a arte da cena mudar de escala.
#[test]
fn a_escala_da_barra_e_a_que_a_tabela_do_raio_cita() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    // ⚠️ A cadeia real e' feita de poses LOCAIS: so' a raiz esta' no mundo, e cada filho nasce na
    // ponta do pai. Com absolutas ela dobra-se sobre si mesma e os pesos colapsam no primeiro osso
    // — foi o que a 1.a redaccao desta sonda mediu, e lia-se como um defeito do produto.
    let passo = (-1.8f64 - -8.2) / 3.0;
    let mut pai = None;
    for k in 0..3 {
        let pos = if k == 0 {
            [-8.2, 2.5]
        } else {
            [passo as f32, 0.0]
        };
        pai = Some(osso(&mut sim, &format!("Bone {}", k + 1), pos, passo, pai));
    }
    crate::skin_live::bind(&mut sim, &scene, &map, &[id], None);
    let pts = crate::peso_a_mao::repousos(&sim, forma(&map, id), PPM);
    let mut d: Vec<[f64; 2]> = Vec::new();
    for q in &pts {
        if !d.iter().any(|x| dist(*x, *q) < 1e-9) {
            d.push(*q);
        }
    }
    assert!(
        d.len() >= 8,
        "a barra da cena tem {} posicoes distintas — a tabela do raio foi medida sobre 14, e com \
         menos que isto ela deixou de a descrever",
        d.len()
    );
    let mut viz = f64::INFINITY;
    let mut peca: f64 = 0.0;
    for i in 0..d.len() {
        for j in (i + 1)..d.len() {
            let s = dist(d[i], d[j]);
            if s > 1e-9 {
                viz = viz.min(s);
            }
            peca = peca.max(s);
        }
    }
    let (viz_px, peca_px) = (viz * f64::from(PPM), peca * f64::from(PPM));
    // O raio de fábrica, repetido AQUI de propósito: esta crate não conhece a `ph2d-tool-vector`
    // (a dependência seria ao contrário), então o que o gate pode afirmar é a FAIXA em que o
    // número tem de cair. ⛔ Mudá-lo lá sem o mudar aqui deixa a tabela do doc a mentir.
    const RAIO_DE_FABRICA_PX: f64 = 40.0;
    assert!(
        RAIO_DE_FABRICA_PX > viz_px * 1.2,
        "o raio de fabrica ({RAIO_DE_FABRICA_PX} px) nao chega a cobrir uma vizinhanca: dois \
         pontos vizinhos desta peca estao a {viz_px:.1} px — o pincel virou um apontador"
    );
    assert!(
        RAIO_DE_FABRICA_PX < peca_px * 0.25,
        "o raio de fabrica ({RAIO_DE_FABRICA_PX} px) e' grande demais para a peca ({peca_px:.1} \
         px): era assim que o `20.0` em unidades de MUNDO agarrava todos os pontos de uma vez"
    );
}
