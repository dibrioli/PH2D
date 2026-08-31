//! Os gates da resolução da arte dos pincéis (plano 36, W3).

use super::*;
use ph2d_vec_scene::{BrushStroke, Rgba8, StrokePaint, StrokeSpec, VecPath, VecVertex};

fn quadrado(x: f64) -> Vec<VecVertex> {
    [[x, 0.0], [x + 1.0, 0.0], [x + 1.0, 1.0], [x, 1.0]]
        .map(VecVertex::corner)
        .to_vec()
}

/// Uma cena com a ARTE e uma forma cujo traço é um pincel que a nomeia.
fn cena(aponta_para_si: bool) -> (VecScene, VecPathId, VecPathId) {
    let mut scene = VecScene::default();
    let arte = scene.push_path(VecPath {
        verts: quadrado(0.0),
        closed: true,
        ..VecPath::default()
    });
    let hospedeira = scene.push_path(VecPath {
        verts: quadrado(5.0),
        closed: true,
        ..VecPath::default()
    });
    let alvo = if aponta_para_si { hospedeira } else { arte };
    if let Some(p) = scene.path_mut(hospedeira) {
        let mut s = StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5);
        s.paint = StrokePaint::Brush(Box::new(BrushStroke {
            art: Some(alvo),
            ..BrushStroke::default()
        }));
        p.stroke = Some(s);
    }
    (scene, hospedeira, arte)
}

/// ⭐⭐ **A ARTE de um pincel é resolvida, e endereçada pela forma HOSPEDEIRA.**
#[test]
fn the_brush_art_resolves_keyed_by_its_host() {
    let (scene, hospedeira, _) = cena(false);
    let mapa = resolve(&scene, &|id| vec![id], &ph2d_vec_scene::VecXforms::new());
    assert!(
        mapa.contains_key(&hospedeira),
        "a arte do pincel nao foi resolvida para a forma hospedeira"
    );
    // CONTROLO: a forma-ARTE não é chave — a chave é quem PINTA, não quem é pintado. Trocá-las
    // faria o desenho procurar a arte pelo id errado e cair sempre na cor de recurso.
    assert_eq!(mapa.len(), 1, "o mapa tem uma entrada por HOSPEDEIRA");
}

/// ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PINCEL.**
///
/// Desenhá-la exigiria as cópias, as cópias exigiriam a arte, e a arte seria ela. ⚠️ **O sintoma não
/// seria um erro**: seria o app a parar. É a mesma recusa PURA que o padrão-forma já tem.
#[test]
fn a_shape_can_never_be_its_own_brush() {
    let (scene, hospedeira, _) = cena(true);
    let mapa = resolve(&scene, &|id| vec![id], &ph2d_vec_scene::VecXforms::new());
    assert!(
        !mapa.contains_key(&hospedeira),
        "uma forma resolveu-se como o proprio pincel - o desenho entraria em recursao"
    );
    // CONTROLO: com a arte a apontar para OUTRA forma, ela resolve — senão este gate ficaria verde
    // sobre uma resolução que nunca devolve nada.
    assert!(
        resolve(
            &cena(false).0,
            &|id| vec![id],
            &ph2d_vec_scene::VecXforms::new()
        )
        .contains_key(&hospedeira)
    );
}

/// ⚠️ **A arte entra COZIDA, não como foi digitada.**
///
/// Um motivo com quina viva ou com pilha de efeitos tem de se repetir como **parece**, não como foi
/// autorado — a mesma lei que a arte-forma de um padrão já obedece.
#[test]
fn the_art_enters_cooked_not_as_authored() {
    let (mut scene, hospedeira, arte) = cena(false);
    // Uma quina viva na arte: o cozido ganha vértices que a fonte não tem.
    let crus = scene.path(arte).map(|p| p.verts.len()).unwrap_or(0);
    if let Some(p) = scene.path_mut(arte) {
        for v in &mut p.verts {
            v.corner_radius = 0.25;
        }
    }
    let mapa = resolve(&scene, &|id| vec![id], &ph2d_vec_scene::VecXforms::new());
    let resolvida = mapa.get(&hospedeira).expect("resolve");
    assert!(
        resolvida[0].verts.len() > crus,
        "a arte entrou AUTORADA ({} vertices, os mesmos da fonte) - um motivo com quina viva \
         repetir-se-ia com a quina afiada",
        resolvida[0].verts.len()
    );
}

/// ⚠️ **Uma cena SEM pincéis não paga nada** — nem uma entrada.
#[test]
fn a_scene_without_brushes_costs_nothing() {
    let mut scene = VecScene::default();
    scene.push_path(VecPath {
        verts: quadrado(0.0),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5)),
        ..VecPath::default()
    });
    assert!(resolve(&scene, &|id| vec![id], &ph2d_vec_scene::VecXforms::new()).is_empty());
}

/// ⭐⭐⭐ **A ARTE DE UM PINCEL PODE SER UM GRUPO, e a recusa de ciclo é sobre PERTENÇA.**
///
/// Report do Enio (2026-08-30) — ele pediu-o para a estampa, e o pincel é a mesma metade noutra
/// tinta. O documento endereça a arte por um `VecPathId` e um grupo **não tem um**: o que muda é a
/// **resolução**, e ela passa pela porta que a estampa já usa
/// ([`crate::texture_pattern_live::art_members`]).
///
/// # As duas metades
///
/// **Expandir**: um id que pertence a um grupo traz o grupo INTEIRO.
///
/// **Recusar**: e a recusa deixou de ser `art == host` — com um grupo, o anfitrião pode ser um
/// **membro** da arte. Desenhá-lo exigiria as cópias, as cópias exigiriam a arte, e a arte seria
/// ele. ⚠️ *O sintoma não seria um erro: seria o app a parar.*
///
/// ⚠️ **A segunda metade é o CONTROLO da primeira**: uma expansão que devolvesse sempre tudo
/// passaria a primeira e reprovaria esta.
#[test]
fn a_group_is_a_brush_art_and_the_cycle_refusal_is_about_membership() {
    let (scene, hospedeira, arte) = cena(false);
    // Um irmão da arte: os dois formam o grupo.
    let mut scene = scene;
    let irmao = scene.push_path(VecPath {
        verts: quadrado(2.0),
        closed: true,
        ..VecPath::default()
    });
    let grupo_da_arte = move |id: VecPathId| {
        if id == arte || id == irmao {
            vec![arte, irmao]
        } else {
            vec![id]
        }
    };
    let mapa = resolve(&scene, &grupo_da_arte, &ph2d_vec_scene::VecXforms::new());
    assert_eq!(
        mapa.get(&hospedeira).map(Vec::len),
        Some(2),
        "a arte do pincel nao expandiu o GRUPO - so' o caminho apontado chegou ao desenho"
    );
    // ⚠️ CONTROLO: sem a expansão, o mesmo pedido dá UM — a fixtura contém o fenómeno.
    assert_eq!(
        resolve(&scene, &|id| vec![id], &ph2d_vec_scene::VecXforms::new())
            .get(&hospedeira)
            .map(Vec::len),
        Some(1)
    );
    // ⛔ E o grupo que CONTÉM o anfitrião é recusado inteiro: é o ciclo que pararia o app.
    let grupo_com_o_host = move |id: VecPathId| {
        if id == arte || id == hospedeira {
            vec![arte, hospedeira]
        } else {
            vec![id]
        }
    };
    assert!(
        !resolve(&scene, &grupo_com_o_host, &ph2d_vec_scene::VecXforms::new())
            .contains_key(&hospedeira),
        "um grupo que contem o proprio anfitriao foi aceite - desenha-lo exige as copias, e as \
         copias exigem a arte: o app PARA"
    );
}

/// ⛔⛔ **MOVER UM MEMBRO DO GRUPO-ARTE TEM DE MUDAR O PINCEL** — o report de 2026-08-30 na estampa
/// (*"ao mover os objetos do grupo que serve como shape, a pattern não atualiza em tempo real"*),
/// vivo na outra tinta.
///
/// # Porque ele sobreviveu à wave que trouxe o grupo
///
/// Desde o ADR-0110 a geometria de um `VecPath` é **local** e quem a põe no mundo é o `Xform`. A
/// `resolve` lia só os caminhos ⇒ o arranjo dos membros era o que a geometria **autorada** dizia, e
/// não onde o artista os pôs. Uma dupla desenhada e agrupada funcionava por acidente (o desenho
/// escreve os vértices já separados); **mexer** num membro depois não movia nada.
///
/// ⚠️ *A wave do grupo não criou isto — ela tornou-o alcançável.* Com arte de UMA forma a pose não
/// tem sujeito: o motivo é re-enquadrado na guia de qualquer maneira. Com um GRUPO, a pose **é** a
/// disposição.
#[test]
fn moving_a_member_of_the_art_group_changes_the_brush() {
    let (mut scene, hospedeira, arte) = cena(false);
    let irmao = scene.push_path(VecPath {
        verts: quadrado(2.0),
        closed: true,
        ..VecPath::default()
    });
    let grupo = move |id: VecPathId| {
        if id == arte || id == irmao {
            vec![arte, irmao]
        } else {
            vec![id]
        }
    };
    let em = |dy: f64| {
        let mut xf = ph2d_vec_scene::VecXforms::new();
        xf.insert(arte, ph2d_vec_scene::Xform::IDENTITY);
        xf.insert(irmao, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 0.0, dy]));
        xf
    };
    let arranjo = |dy: f64| -> Vec<[f64; 2]> {
        resolve(&scene, &grupo, &em(dy))[&hospedeira]
            .iter()
            .map(|p| p.verts[0].anchor)
            .collect()
    };
    let parado = arranjo(0.0);
    let movido = arranjo(7.0);
    assert_ne!(
        parado, movido,
        "mover um membro do grupo-arte nao mudou o pincel - a disposicao que o artista ve' e' a \
         POSE, e a resolucao so' lia a geometria autorada"
    );
    // ⚠️ CONTROLO: o deslocamento é o que se pediu, e só no membro que se moveu.
    assert_eq!(parado.len(), 2, "a fixtura nao tem DOIS membros");
    assert_eq!(parado[0], movido[0], "o membro parado mexeu-se");
    assert!(
        (movido[1][1] - parado[1][1] - 7.0).abs() < 1e-9,
        "o membro movido nao andou os 7.0 pedidos: {:?} -> {:?}",
        parado[1],
        movido[1]
    );
}

/// ⭐⭐⭐ **O MEMO NÃO RE-RESOLVE O QUE NÃO MUDOU — E RE-RESOLVE TUDO O QUE MUDA.**
///
/// # O número que o justifica
///
/// Medido em 2026-08-30: `custo ≈ P × G × cooked(arte)`, e um `cooked()` de arte **viva** custa
/// `17 529 ns` contra `46,6 ns` de arte simples (**376×**). Com `50` pincéis, grupos de `16` e arte
/// viva, a resolução comia **14,28 ms — 85,5% de um quadro**; um pincel só com grupo de `16` já
/// custava `1,80%`. O memo paga-se `881–1484×`.
///
/// # ⛔⛔ CADA CASO PARTE DE UM MEMO FRESCO, e a 1.ª redacção NÃO partia
///
/// Ela encadeava os casos num memo só e comparava cada um com a base — mas o memo carrega o estado
/// do caso ANTERIOR, então uma mudança era detectada porque a chave do passo de trás já diferia, e
/// não pelo campo em teste. ⚠️ **Três mutações sobreviveram e foi assim que se soube**: tornar
/// `membros`, `conteudo` e `art` constantes não matava nada. *Um gate que testa N coisas em cadeia
/// testa uma só — a última.*
///
/// ⇒ cada caso monta o seu memo, prima-o com a **BASE**, e só então resolve sobre a cena mudada.
///
/// ⚠️⚠️ **E a 2.ª redacção primava com a cena JÁ ALTERADA em dois dos casos** — aí a mudança nunca
/// foi uma mudança, foi a base, e o memo acertava por não ter com o que comparar. Outra mutação
/// sobrevivente (`conteudo` constante) foi quem o disse. *Isolar um caso é escolher o que fica IGUAL
/// entre as duas corridas, não só o que muda.*
#[test]
fn the_brush_memo_reuses_what_did_not_change_and_notices_everything_that_did() {
    let (base, hospedeira, arte) = cena(false);
    let mut scene = base;
    let irmao = scene.push_path(VecPath {
        verts: quadrado(2.0),
        closed: true,
        ..VecPath::default()
    });
    let de_fora = scene.push_path(VecPath {
        verts: quadrado(9.0),
        closed: true,
        ..VecPath::default()
    });
    let par = move |id: VecPathId| {
        if id == arte || id == irmao {
            vec![arte, irmao]
        } else {
            vec![id]
        }
    };
    let sem_pose = ph2d_vec_scene::VecXforms::new();

    // ⭐ A METADE DE CIMA: nada mudou ⇒ a resposta é a MESMA.
    let mut memo = BrushLive::default();
    let a = memo.resolve(&scene, &par, &sem_pose)[&hospedeira].clone();
    let b = memo.resolve(&scene, &par, &sem_pose)[&hospedeira].clone();
    assert_eq!(a, b, "o memo devolveu outra coisa sem nada ter mudado");

    // ⭐⭐ A METADE DE BAIXO — cada caso NUM MEMO PRÓPRIO, primado com a base.
    let novo = |s: &VecScene| {
        let mut m = BrushLive::default();
        m.resolve(s, &par, &sem_pose);
        m
    };
    // 1. a POSE de um membro — sem tocar em geometria nenhuma.
    let mut com_pose = ph2d_vec_scene::VecXforms::new();
    com_pose.insert(irmao, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 0.0, 5.0]));
    assert_ne!(
        a,
        novo(&scene).resolve(&scene, &par, &com_pose)[&hospedeira],
        "a POSE nao invalidou - mover um membro congelaria o pincel"
    );
    // 2. a LISTA (o grupo desfez-se), sem tocar em geometria nenhuma.
    assert_ne!(
        a,
        novo(&scene).resolve(&scene, &|id| vec![id], &sem_pose)[&hospedeira],
        "a LISTA nao invalidou - desagrupar congelaria a arte"
    );
    // 3. o CONTEÚDO de um membro.
    let mut editada = scene.clone();
    if let Some(p) = editada.path_mut(irmao) {
        p.verts[2].anchor = [77.0, 77.0];
    }
    assert_ne!(
        a,
        novo(&scene).resolve(&editada, &par, &sem_pose)[&hospedeira],
        "o CONTEUDO nao invalidou - editar um no' congelaria a arte"
    );
    // 4. o ART autorado — e ele entra pela LISTA, que é o que o `conteudo` já carrega.
    let mut trocada = scene.clone();
    if let Some(p) = trocada.path_mut(hospedeira)
        && let Some(s) = p.stroke.as_mut()
        && let Some(br) = s.brush()
    {
        let mut nb = br.clone();
        nb.art = Some(de_fora);
        s.paint = ph2d_vec_scene::StrokePaint::Brush(Box::new(nb));
    }
    assert_ne!(
        a,
        novo(&scene).resolve(&trocada, &par, &sem_pose)[&hospedeira],
        "trocar o ART autorado nao invalidou"
    );
    // 5. ⚠️ E a VARREDURA tem as duas metades: um traço que deixa de ser pincel SAI do mapa.
    let mut solida = scene.clone();
    if let Some(p) = solida.path_mut(hospedeira)
        && let Some(s) = p.stroke.as_mut()
    {
        s.paint = ph2d_vec_scene::StrokePaint::Solid(Rgba8::new(1, 2, 3, 255));
    }
    let mut m = novo(&scene);
    assert!(
        !m.resolve(&solida, &par, &sem_pose)
            .contains_key(&hospedeira),
        "a arte de um traco que deixou de ser pincel ficou no mapa - ela seria desenhada para sempre"
    );
}
