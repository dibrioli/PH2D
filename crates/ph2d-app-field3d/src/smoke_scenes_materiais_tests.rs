//! ⭐⭐ **QUEM SEMEIA MATERIAL NAS CENAS DO SMOKE** — o censo e o elo que leva o material às
//! entidades.
//!
//! ⚠️ Mudou-se do `cor_da_profundidade_tests` (a medição da cor do jade no Render traçado, retirado
//! em 03/10) para junto das cenas: o censo é das CENAS, não do desenho.

/// A cor autorada das duas fixturas da cena `=34` (base e subsuperfície).
const COR: [f32; 3] = [0.75, 0.35, 0.35];

/// ⭐⭐ **O que a cena declara é o que a MEDIÇÃO usou** — ⛔ não uma cor bonita.
///
/// ⚠️ E a escala por canal tem de ser **IGUAL nos três**: com a de omissão (`1 · 0,5 · 0,25`) a
/// matiz também mudaria por cada canal viajar a sua distância, e a cena deixaria de responder a uma
/// pergunta só.
#[test]
fn a_cena_da_cor_declara_o_material_da_medicao() {
    let pedidos = crate::smoke::scenes::materiais_da_cena(34).expect("a `=34` declara materiais");
    for (m, esperado) in pedidos
        .iter()
        .zip(crate::smoke::scenes::edge::PROFUNDIDADES_DA_COR)
    {
        assert!(
            (m.subsurface_radius - esperado).abs() < 1e-6,
            "a profundidade saiu da tabela do oráculo"
        );
        assert_eq!(m.subsurface_color, COR, "a cor é a das fixturas");
        assert_eq!(
            m.subsurface_radius_scale, [1.0; 3],
            "raios IGUAIS nos três canais"
        );
        assert!(
            m.subsurface_weight >= 1.0,
            "subsuperfície pura, como a medição"
        );
        assert!(
            m.specular_weight <= 0.0,
            "sem realce a lavar o que se quer ver"
        );
    }
    // ⭐⭐⭐ **E O CENSO — a premissa dele MORREU em 2026-09-19, que é para o que ele existe.**
    //
    // ⛔ Ele dizia *«nenhuma OUTRA cena pede material — o mecanismo nasceu para esta, e um segundo
    // consumidor silencioso mudaria uma cena que alguém já aprovou»*, e reprovou no dia em que o
    // segundo chegou: a **`=36`** (o BRILHO, `docs/Render3d/12`) semeia três luzes de forças
    // diferentes. *Ele não foi contornado — foi reescrito com a morte da premissa à vista no diff*,
    // e a metade que continua a valer é a que importa: **quem pede material está nesta lista**.
    //
    // ⚠️ E a lista é de PARES, não um `!=`: um censo escrito como *«só a 34 e a 36»* aceitaria a
    // 36 a semear o material da 34.
    const QUEM_SEMEIA: [(u32, &str); 7] = [
        (34, "a cor que a profundidade deixa"),
        (36, "as três luzes do brilho"),
        (37, "as seis cores dos objetos do Render por malha"),
        (
            38,
            "o cromo, o ouro, o azul brilhante e o vermelho fosco do céu",
        ),
        (39, "os materiais BRANCOS das texturas (a chapa é metal)"),
        (40, "os tons médios do contacto (caixa, bolas, toro, o L)"),
        (41, "o branco fosco, o cromo e o cinzento do chão que tapa"),
    ];
    for n in 1..=crate::smoke::scenes::CENAS {
        let esperado = QUEM_SEMEIA.iter().find(|(k, _)| *k == n);
        assert_eq!(
            crate::smoke::scenes::materiais_da_cena(n).is_some(),
            esperado.is_some(),
            "a cena {n} mudou de material sem ninguém dizer"
        );
    }
    // ⭐ E a metade NEGATIVA do censo: uma entrada que já não descreve nada tem de sair. *Uma lista
    // de tolerância sem censo de obsolescência não desce — ela vira licença* (`CLAUDE.md` §5.0).
    for (n, porque) in QUEM_SEMEIA {
        assert!(
            crate::smoke::scenes::materiais_da_cena(n).is_some(),
            "a entrada «{n}: {porque}» já não descreve nada — apague-a"
        );
    }
}

/// ⭐⭐⭐ **O MATERIAL CHEGA ÀS ESFERAS — pelo caminho do produto, não pela tabela.**
///
/// ⛔⛔ As duas metades acima provam que a cena **declara** o material certo e que esse material
/// **contém** o fenómeno. As duas são cegas ao elo do meio: *alguém tem de PÔR o material nas
/// entidades*. Um `Vec` declarado e deitado fora lê-se exactamente como um `Vec` aplicado — e é o
/// mesmo buraco que o `CLAUDE.md` §5.0 nomeia sobre si mesmo (*«nenhum instrumento pergunta se o
/// VALOR chega a um consumidor»*).
///
/// ⚠️ Ele entra pela [`crate::scene::sync_scene_and_birth`], que é a porta que o smoke percorre, e
/// não por uma montagem à mão do mundo.
#[test]
fn o_material_da_cena_chega_as_esferas() {
    let doc = crate::smoke::scenes::edge::cena_34().expect("a cena `=34`");
    let pedidos = crate::smoke::scenes::materiais_da_cena(34).expect("os materiais dela");
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|s| s.seed_materials = Some(pedidos.clone()));
    let mut sim = ph2d_ecs::SimWorld::new();
    crate::scene::sync_scene_and_birth(&mut sim, Some(&doc), &[], 0.0, &crate::scene::no_drawing());
    let world = sim.world_mut();
    let mut q = world.query::<&ph2d_field_ecs::FieldMaterial>();
    let mut raios: Vec<f32> = q.iter(world).map(|m| m.subsurface_radius).collect();
    let cores: Vec<[f32; 3]> = {
        let mut q2 = world.query::<&ph2d_field_ecs::FieldMaterial>();
        q2.iter(world).map(|m| m.subsurface_color).collect()
    };
    crate::smoke::set_armed_by_panel(false);

    assert_eq!(
        raios.len(),
        4,
        "só {} das 4 esferas receberam material — o elo do meio está partido",
        raios.len()
    );
    raios.sort_by(f32::total_cmp);
    let mut alvo = crate::smoke::scenes::edge::PROFUNDIDADES_DA_COR;
    alvo.sort_by(f32::total_cmp);
    for (a, b) in raios.iter().zip(alvo) {
        assert!(
            (a - b).abs() < 1e-6,
            "as profundidades que chegaram ({raios:?}) não são as que a cena pediu ({alvo:?})"
        );
    }
    // ⚠️ E a COR também — um material com o raio certo e a cor de omissão voltaria a ser cinzento,
    // que é exactamente o defeito que esta cena existe para não ter.
    for c in &cores {
        assert_eq!(*c, COR, "uma esfera ficou com a cor de omissão");
    }
}
