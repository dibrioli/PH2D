//! ⏱️ **DUAS IMAGENS NO MESMO ESQUELETO** — filho do [`super`] para herdar as fixturas do braço.
//!
//! ⭐⭐⭐ **É a pergunta que separa «um braço» de «um PERSONAGEM».** Um boneco é um tronco, dois
//! braços, duas pernas e uma cabeça — peças SEPARADAS presas ao **mesmo** rig. Todas as cenas de
//! smoke deste módulo dão a cada imagem a corrente DELA, logo *a capacidade que o artista precisa
//! nunca foi exercida por nenhuma delas*.
//!
//! ⚠️ **A dúvida é legítima e não retórica:** o [`super::super::skin_live::bind_image`] recebe um
//! osso SEMENTE e prende à árvore dele, o que sugere que partilhar sai de graça — e *«sai de graça
//! por construção»* foi exactamente a frase que a medição desmentiu duas vezes em 2026-09-18 (a
//! nota da dobra e o enquadramento da cena). ⇒ mede-se.

use super::*;

/// Duas sprites presas ao **mesmo** osso semente, dobradas `graus` por junta.
///
/// Devolve `(malha de cima, malha de baixo)` — `None` em qualquer uma quer dizer que ela não recebeu
/// pele nenhuma, que é o modo de falha que esta sonda existe para apanhar.
///
/// ⚠️ **As duas ficam em ALTURAS diferentes e a corrente corre no meio**, de propósito: com as duas
/// no mesmo sítio elas teriam os mesmos pesos e a sonda não distinguiria *«as duas seguem o rig»* de
/// *«a segunda é uma cópia da primeira»*.
type Presa = (Option<SpriteMesh>, [f32; 2], usize);

fn duas_no_mesmo_rig(graus: f32) -> (Presa, Presa) {
    let ([w, h], arte) = braco_px();
    let celula = [f32::from(w as u16) / PPM, f32::from(h as u16) / PPM];
    let mut sim = SimWorld::default();

    // ⛔⛔ **A CORRENTE ISCA, e ela é o que torna esta sonda honesta.** Com uma corrente só no
    // mundo, prender ao osso semente e prender a `None` dão **o mesmo** resultado: o
    // [`super::super::skin_live::skeleton_of`] responde *«todos os ossos da cena»* quando a semente
    // não serve. ⇒ *uma sonda de PARTILHA montada num mundo de um esqueleto só não distingue
    // «partilham o rig» de «apanham o que houver»* — e foi uma mutação que o mostrou.
    //
    // ⚠️⚠️ **E a DISTÂNCIA dela NÃO é load-bearing, medido:** aproximá-la da figura deixa o gate
    // verde (mutação `M5`), porque quem mede a partilha é a **CONTAGEM de ossos do bind** e não uma
    // magnitude. Os 10 m são legibilidade — um osso longe pesa `~0` em toda a arte, logo as duas
    // metades de excursão continuam a medir só a corrente que foi dobrada. *Uma mutação que não
    // sangra sobre um número escrito num comentário é o comentário a ser refutado, não o gate.*
    let mut isca = None;
    for k in 0..3 {
        let x = 10.0 + f64::from(k);
        let osso = crate::bone::create(&mut sim, isca, [x, 0.0], [x + 1.0, 0.0]).expect("isca");
        isca = Some(Entity::from_bits(osso));
    }

    // UMA corrente, no meio das duas imagens.
    let mut pai = None;
    let mut ossos = Vec::new();
    for k in 0..3 {
        let passo = f64::from(celula[0]) / 3.0;
        let x0 = -f64::from(celula[0]) / 2.0;
        let osso = crate::bone::create(
            &mut sim,
            pai,
            [x0 + passo * f64::from(k), 0.0],
            [x0 + passo * f64::from(k + 1), 0.0],
        )
        .expect("osso");
        let e = Entity::from_bits(osso);
        ossos.push(e);
        pai = Some(e);
    }

    let meia = f64::from(celula[1]) * 0.6;
    let mut presas = Vec::new();
    for (k, y) in [meia, -meia].into_iter().enumerate() {
        let s = sprite(celula[0], celula[1], 0.0, y as f32);
        let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
        let ok = crate::skin_live::bind_image(
            &mut sim,
            e,
            &arte,
            [w, h],
            PPM,
            ph2d_poly2d::GridOptions::default(),
            // ⭐ A SEMENTE é a MESMA nas duas: é isto que se está a medir.
            ossos.first().copied(),
        );
        presas.push((e, s, ok, k));
    }

    for osso in ossos.iter().skip(1) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(*osso) {
            t.rotation += graus.to_radians();
        }
    }

    let mut present = PresentWorld::new();
    let mut alvos = Vec::new();
    for (e, s, ok, _) in &presas {
        assert!(*ok, "o bind recusou uma das duas imagens");
        let ri = instancia_de(s);
        alvos.push(present.world_mut().spawn((SimRef(*e), ri)).id());
    }
    attach_skin_meshes(&sim, &mut present, PPM, &[]);
    // ⭐⭐⭐ **QUANTOS OSSOS CADA PELE PRENDEU — é ISTO que mede a partilha.** As excursões dizem
    // *«as duas seguem os ossos que foram dobrados»* e ficam verdes mesmo quando o bind apanha o
    // esqueleto inteiro da cena (a isca incluída), porque um osso a 10 m pesa ~0 em toda a arte.
    // *A diferença entre «prendi ao rig que nomeei» e «prendi ao que havia» é uma CONTAGEM* — e foi
    // uma mutação (o `Some(semente)` trocado por `None`) que a exigiu.
    let ossos_da_pele = |e: Entity| -> usize {
        sim.world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .and_then(|b| postcard::from_bytes::<crate::skinned_mesh::SkinnedMesh>(&b.source).ok())
            .map_or(0, |m| m.ossos())
    };
    let contagens: Vec<usize> = presas
        .iter()
        .map(|(e, _, _, _)| ossos_da_pele(*e))
        .collect();
    let mut out = alvos
        .into_iter()
        .zip(presas.iter().map(|(_, s, _, _)| s.anchor))
        .zip(contagens)
        .map(|((p, a), n)| (present.world().get::<SpriteMesh>(p).cloned(), a, n));
    (
        out.next().expect("a de cima"),
        out.next().expect("a de baixo"),
    )
}

/// Quanto a malha `m` se afastou do repouso, em metros (o maior deslocamento de um vértice).
///
/// ⚠️ **O repouso é a UV levada de volta ao quad** — `local = âncora + (u − ½, ½ − v) · tamanho`,
/// a inversa exacta da [`SpriteMesh::uv_at`] —, e não uma segunda cópia da malha: duas fontes do
/// repouso divergem no dia em que uma mudar.
///
/// ⛔⛔ **A ÂNCORA é obrigatória, e a 1.ª redacção esqueceu-a:** o controlo com a corrente PARADA
/// leu `0,360000 m` de excursão, que é **exactamente** o deslocamento em `y` da sprite. *Uma régua
/// que confunde ONDE a sprite está com QUANTO ela se deformou acusa toda a cena de estar torta* —
/// e foi o controlo que a apanhou, pela terceira vez neste dia.
fn excursao(m: &SpriteMesh, ancora: [f32; 2], tamanho: [f32; 2]) -> f64 {
    m.local
        .iter()
        .zip(&m.uv)
        .map(|(l, uv)| {
            let rx = f64::from(ancora[0] + (uv[0] - 0.5) * tamanho[0]);
            let ry = f64::from(ancora[1] + (0.5 - uv[1]) * tamanho[1]);
            let (dx, dy) = (f64::from(l[0]) - rx, f64::from(l[1]) - ry);
            dx.hypot(dy)
        })
        .fold(0.0_f64, f64::max)
}

/// ⭐⭐⭐ **DUAS IMAGENS PRESAS AO MESMO ESQUELETO DOBRAM AS DUAS** — a capacidade que faz um
/// personagem, e que nenhuma cena de smoke deste módulo exercia.
///
/// ⛔ **O CONTROLO é a corrente PARADA:** sem ele, um gate que só pede *«as duas mexeram»* fica
/// verde sobre uma malha que nasceu torta, e sobre um bind que copiou a primeira para a segunda.
#[test]
fn duas_imagens_no_mesmo_rig_dobram_as_duas() {
    let tamanho = [
        f32::from(braco_px().0[0] as u16) / PPM,
        f32::from(braco_px().0[1] as u16) / PPM,
    ];
    let (parada_a, parada_b) = duas_no_mesmo_rig(0.0);
    let ((a, anc_a, ossos_a), (b, anc_b, ossos_b)) = duas_no_mesmo_rig(25.0);

    // ⭐ A METADE QUE MEDE A PARTILHA: as duas prendem à corrente que foi NOMEADA (3 ossos), e não
    // ao esqueleto inteiro da cena (3 + 3 da isca).
    assert_eq!(
        (ossos_a, ossos_b),
        (3, 3),
        "as duas peles nao prenderam a corrente NOMEADA: {ossos_a} e {ossos_b} ossos contra 3 — \
         com 6 elas apanharam tambem a corrente ISCA, que e' «prendi ao que havia»"
    );
    let (a, b) = (
        a.expect("a de cima nao recebeu pele"),
        b.expect("a de baixo nao recebeu pele"),
    );

    // O CONTROLO: com a corrente parada nenhuma das duas se afasta do repouso.
    for ((m, anc, _), quem) in [(&parada_a, "cima"), (&parada_b, "baixo")] {
        let m = m.as_ref().expect("controlo: sem pele");
        let e = excursao(m, *anc, tamanho);
        assert!(
            e < 1e-3,
            "controlo: com a corrente PARADA a imagem de {quem} ja' esta' deslocada {e:.6} m — \
             a malha nasceu torta e o gate de baixo mediria o nada"
        );
    }
    for ((m, anc), quem) in
        [(&a, anc_a, "cima"), (&b, anc_b, "baixo")].map(|(m, anc, quem)| ((m, anc), quem))
    {
        let e = excursao(m, anc, tamanho);
        assert!(
            e > 0.05,
            "a imagem de {quem} mexeu-se so' {e:.6} m com a corrente dobrada 25° por junta — ela \
             NAO esta' a seguir o esqueleto partilhado"
        );
    }
    // ⚠️ E elas não são a MESMA malha: estão em alturas diferentes, logo os pesos diferem e a
    // excursão também. *Sem isto, um bind que copiasse a primeira para a segunda passaria.*
    let (ea, eb) = (excursao(&a, anc_a, tamanho), excursao(&b, anc_b, tamanho));
    assert!(
        (ea - eb).abs() > 1e-6,
        "as duas mexeram-se EXACTAMENTE o mesmo ({ea:.9} m) — elas estao em alturas diferentes, \
         logo isto le-se como a segunda ser uma copia da primeira e nao uma pele propria"
    );
    println!("cima {ea:.6} m | baixo {eb:.6} m");
}
