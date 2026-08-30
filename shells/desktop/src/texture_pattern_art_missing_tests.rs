//! ⭐⭐⭐ **A ARTE QUE SUMIU** — os gates do vínculo morto (plano 33, W11).
//!
//! Irmão do [`super::tests`], e separado dele por RESPONSABILIDADE: aquele mede o **memo do
//! assado** (o que re-assa, o que reaproveita, o que a chave contém); este mede o que acontece
//! quando a forma que serve de arte **deixa de existir**.
//!
//! ⚠️ O ficheiro único batia no tecto de LOC do shell — que vive em `shells/desktop/tests/` e
//! **não é alcançado por `cargo test --bins`**.

use super::*;
use ph2d_vec_scene::{PatternFill, Rgba8, VecPath, VecVertex};

/// [`art_is_missing`] num mundo **sem ECS**: cada caminho é o próprio objecto.
///
/// ⚠️ Estes gates medem a lei da ARTE QUE SUMIU, não a expansão de grupo — que tem gates próprios.
fn art_is_missing_solo(scene: &VecScene, host: VecPathId, source: &PatternSource) -> bool {
    art_is_missing(scene, host, source, &|id| vec![id])
}

/// ⭐⭐⭐ **APAGAR A FORMA-FONTE deixa a estampa sem arte, e o app tem de o saber** (plano 33, W11).
///
/// # O defeito que isto fecha
///
/// Sem esta pergunta, apagar a forma que serve de arte faz a estampa voltar a **cor chapada** — e
/// cor chapada é exactamente o que um preenchimento sólido correcto parece. O `PatternSource` não
/// tem variante vazia, então a secção do painel sobe inteira e normal por cima de um vínculo morto.
///
/// # As quatro respostas, e a terceira é a que uma consulta directa erraria
///
/// ⚠️ `art_is_missing` pergunta pela porta que ASSA, e essa porta recusa também a
/// **auto-referência**: uma forma que se nomeia a si própria como arte não tem arte utilizável (o
/// ladrilho exigiria desenhá-la, e desenhá-la exigiria o ladrilho). Um `scene.path(id).is_none()`
/// escrito à mão daria essa forma como **presente**, e a secção continuaria muda.
#[test]
fn a_deleted_source_shape_is_reported_as_missing_art() {
    let mut scene = VecScene::default();
    let motivo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let host = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });

    let viva = PatternSource::Shape(motivo);
    assert!(
        !art_is_missing_solo(&scene, host, &viva),
        "a forma-fonte EXISTE e foi dada como apagada - a seccao ia acusar toda estampa viva"
    );

    let morta = PatternSource::Shape(motivo + 1_000);
    assert!(
        art_is_missing_solo(&scene, host, &morta),
        "um id que nao resolve tem de ser 'arte apagada' - senao a estampa volta a cor chapada e \
         nada no ecra' diz porque'"
    );

    assert!(
        art_is_missing_solo(&scene, host, &PatternSource::Shape(host)),
        "uma forma que se nomeia a si propria nao tem arte utilizavel - a porta que assa recusa-a, \
         e uma consulta directa `scene.path(id).is_none()` da-la-ia como presente"
    );

    let imagem = PatternSource::Image(ph2d_asset::AssetId::from_bytes(&[7; 32]));
    assert!(
        !art_is_missing_solo(&scene, host, &imagem),
        "uma fonte-IMAGEM nunca acusa: os pixels dela viajam no ficheiro, e uma ausencia aqui e' \
         transitoria (a carregar). Um aviso permanente sobre estado transitorio ensina a ignorar."
    );
}

/// ⭐⭐ **O VÍNCULO SOBREVIVE AO UNDO, e é isso que limita o estrago** (plano 33, W11).
///
/// O `VecPathId` é um `u64` guardado **dentro** da `VecScene`, e o `ProjectState` leva a cena
/// inteira como dado — o undo repõe-na verbatim, com os ids originais. ⇒ desfazer o apagar devolve
/// a forma com o MESMO id e o vínculo cura-se sozinho.
///
/// ⚠️ **Sem esta folha a afirmação seria leitura, não medição** — e ela é o que separa *"perde-se
/// trabalho para sempre"* de *"perde-se se apagar, gravar e só reparar depois"*. Se um dia a cena
/// passar a re-atribuir ids ao restaurar (um `push_path` no caminho do restore basta), este gate
/// reprova e o aviso do painel deixa de ser a cura suficiente.
#[test]
fn the_art_link_survives_the_round_trip_that_undo_uses() {
    let mut scene = VecScene::default();
    let motivo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let (scene, host) = {
        let mut s = scene;
        let h = s.push_path(VecPath {
            verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(Paint::Pattern(Box::new(PatternFill::new(
                PatternSource::Shape(motivo),
                [1.0, 1.0],
                Rgba8::new(10, 20, 30, 255),
            )))),
            ..VecPath::default()
        });
        (s, h)
    };
    // O undo captura e repõe a cena inteira como DADO — o `clone` é essa viagem, e o postcard é a
    // do ficheiro. As duas têm de devolver o mesmo id.
    let clonada = scene.clone();
    let bytes = postcard::to_allocvec(&scene).expect("a cena serializa");
    let lida: VecScene = postcard::from_bytes(&bytes).expect("a cena volta");

    for (nome, s) in [
        ("o clone (undo)", &clonada),
        ("o postcard (ficheiro)", &lida),
    ] {
        let Some(Paint::Pattern(p)) = s.path(host).and_then(|p| p.fill.as_ref()) else {
            panic!("{nome}: a estampa nao voltou");
        };
        assert_eq!(
            p.source,
            PatternSource::Shape(motivo),
            "{nome}: o id da arte MUDOU - desfazer o apagar deixaria de curar o vinculo, e o \
             estrago passaria de recuperavel a permanente"
        );
        assert!(
            !art_is_missing_solo(s, host, &p.source),
            "{nome}: a arte deixou de resolver depois da viagem"
        );
    }
}

/// ⭐⭐⭐ **UM GRUPO É A ARTE, e a arte são TODOS os membros dele** (Enio, 2026-08-30).
///
/// O documento continua a endereçar a arte por um `VecPathId` — um grupo **não tem um** — e o que
/// mudou é a RESOLUÇÃO: o id passa a nomear o OBJECTO a que o caminho pertence, que é a lei de
/// selecção que o app já tem. ⭐ É por isso que **o schema não se mexe**.
#[test]
fn an_art_that_is_a_group_resolves_to_all_of_its_members_in_z_order() {
    let mut scene = VecScene::default();
    let tri = |x: f64| VecPath {
        verts: [[x, 0.0], [x + 1.0, 0.0], [x, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let a = scene.push_path(tri(0.0));
    let b = scene.push_path(tri(2.0));
    let host = scene.push_path(tri(9.0));

    // O objecto que contém `a` são os dois membros, pela ordem do documento.
    let grupo = |_id: VecPathId| vec![a, b];
    let membros = super::source_shape(&scene, host, &PatternSource::Shape(a), &grupo);
    assert_eq!(
        membros.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![a, b],
        "a arte resolveu para um membro so' (ou fora de ordem) - a estampa desenharia meio grupo, \
         e a ordem e' a de z: troca-la poe o membro errado por cima"
    );
}

/// ⛔⛔ **O CICLO passou a ser sobre PERTENÇA, e sem isso o app PARA.**
///
/// Antes bastava `id == host`. Com um grupo, o anfitrião pode ser **um membro** da arte — assá-la
/// exigiria desenhá-lo, desenhá-lo exigiria o ladrilho, e o ladrilho exigiria assá-la. ⚠️ O sintoma
/// não seria um erro: seria o app a parar.
#[test]
fn a_shape_inside_the_group_it_wears_is_refused() {
    let mut scene = VecScene::default();
    let tri = |x: f64| VecPath {
        verts: [[x, 0.0], [x + 1.0, 0.0], [x, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let a = scene.push_path(tri(0.0));
    let host = scene.push_path(tri(2.0));
    // O anfitrião está DENTRO do grupo que ele veste.
    let grupo = |_id: VecPathId| vec![a, host];

    assert!(
        super::source_shape(&scene, host, &PatternSource::Shape(a), &grupo).is_empty(),
        "a arte foi aceite com o anfitriao la' dentro - isto nao devolve um desenho errado, \
         devolve uma recursao infinita"
    );
    assert!(
        art_is_missing(&scene, host, &PatternSource::Shape(a), &grupo),
        "e o painel tem de o DIZER: para o artista, uma estampa que nao pode resolver a arte e' \
         uma estampa sem arte"
    );
}

/// ⭐⭐ **Editar QUALQUER membro do grupo re-assa o ladrilho** — a promessa de que o padrão é vivo.
///
/// ⚠️ É por isso que a chave guarda os membros e não só o caminho clicado: com um só, mexer no
/// IRMÃO deixava a tela parada, que é o defeito exacto que o `FxKey` da crate irmã documenta.
#[test]
fn editing_any_member_of_the_group_changes_what_the_memo_sees() {
    let mut scene = VecScene::default();
    let tri = |x: f64| VecPath {
        verts: [[x, 0.0], [x + 1.0, 0.0], [x, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let a = scene.push_path(tri(0.0));
    let b = scene.push_path(tri(2.0));
    let host = scene.push_path(tri(9.0));
    let grupo = |_id: VecPathId| vec![a, b];

    let antes = super::source_shape(&scene, host, &PatternSource::Shape(a), &grupo);
    // Mexe no IRMÃO — o membro que o id da fonte NÃO nomeia.
    if let Some(p) = scene.paths_mut().iter_mut().find(|p| p.id == b) {
        p.verts.push(VecVertex::corner([5.0, 5.0]));
    }
    let depois = super::source_shape(&scene, host, &PatternSource::Shape(a), &grupo);
    assert_ne!(
        antes, depois,
        "mexer no IRMAO nao mudou o que a chave ve^ - o ladrilho ficaria parado numa versao \
         anterior do grupo, e o padrao deixaria de ser vivo"
    );
}
