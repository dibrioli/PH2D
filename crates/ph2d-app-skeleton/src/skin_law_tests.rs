//! Os gates da porta que escolhe a LEI de pele por DESENHO.

use super::*;
use ph2d_ecs::Transform;
use ph2d_skeleton_ecs::{SkinBind, Tendon};

/// Um mundo com `n` coisas presas (e uma solta, que é o controlo).
fn palco(n: usize) -> (SimWorld, Vec<u64>, u64) {
    let mut sim = SimWorld::default();
    let mut presas = Vec::new();
    for _ in 0..n {
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                SkinBind::new(
                    Vec::new(),
                    vec![Tendon {
                        bone: ph2d_ecs::StableId::NONE,
                        rest: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    }],
                ),
            ))
            .id();
        presas.push(e.to_bits());
    }
    let solta = sim.world_mut().spawn(Transform::IDENTITY).id().to_bits();
    (sim, presas, solta)
}

/// ⭐⭐⭐ **O SUJEITO SÃO AS DUAS MÍDIAS, e uma coisa SEM pele não é sujeito de nada.**
///
/// ⚠️ **As três metades são três defeitos:** sem a 1.ª o chip não alcança uma imagem (o report de
/// 2026-09-18, que já custou os botões de saída); sem a 2.ª ele escreve numa coisa que não tem
/// pele; sem a 3.ª uma forma que esteja nos DOIS selectores conta duas vezes.
#[test]
fn o_sujeito_e_o_que_esta_escolhido_e_tem_pele() {
    let (sim, presas, solta) = palco(3);
    // Uma pelo pen, uma pelo gizmo, e a solta em ambos.
    let lidas = super::escolhidas(&sim, [presas[0], solta], [presas[1], presas[2], solta]);
    assert_eq!(
        lidas.len(),
        3,
        "a porta leu {} peles de 3: ou ela ignora um dos selectores, ou deixou entrar a coisa sem \
         pele",
        lidas.len()
    );
    // A MESMA forma nos dois selectores conta UMA vez.
    let repetida = super::escolhidas(&sim, [presas[0]], [presas[0]]);
    assert_eq!(
        repetida.len(),
        1,
        "uma forma escolhida pelos DOIS selectores foi contada duas vezes"
    );
}

/// ⭐⭐⭐ **A ESCRITA É POR DESENHO, e o vizinho não é tocado** — é isso que a ordem do dono
/// (*«por desenho»*) quer dizer, e sem gate ela é indistinguível de *«por esqueleto»*.
#[test]
fn a_lei_escreve_so_no_desenho_escolhido() {
    let (mut sim, presas, _) = palco(3);
    let n = super::escreve(
        &mut sim,
        [presas[0]],
        [],
        ph2d_skeleton_ecs::SkinLaw::Envelope,
    );
    assert_eq!(n, 1, "a escrita tocou {n} desenhos e devia tocar 1");
    let lei = |b: u64| {
        sim.world()
            .get::<SkinBind>(ph2d_ecs::Entity::from_bits(b))
            .expect("pele")
            .law
    };
    assert_eq!(lei(presas[0]), ph2d_skeleton_ecs::SkinLaw::Envelope);
    for outro in &presas[1..] {
        assert_eq!(
            lei(*outro),
            ph2d_skeleton_ecs::SkinLaw::Auto,
            "um desenho que NAO estava escolhido mudou de lei — a escolha passou a ser por \
             esqueleto, e a ordem do dono era «por desenho»"
        );
    }
    // ⚠️ E escrever a lei que já lá está não é um acontecimento.
    let zero = super::escreve(
        &mut sim,
        [presas[0]],
        [],
        ph2d_skeleton_ecs::SkinLaw::Envelope,
    );
    assert_eq!(zero, 0, "re-escrever a MESMA lei contou como mudanca");
}

/// ⭐⭐ **O chip acende se ALGUMA estiver por alcance** — a escolha declarada no doc da porta.
///
/// ⛔ Com `all` em vez de `any`, uma selecção MISTA leria «tudo no padrão-ouro» e o artista não
/// saberia que metade dela está noutra lei.
#[test]
fn o_chip_acende_com_uma_so_por_alcance() {
    let (mut sim, presas, _) = palco(2);
    let todas = || [presas[0], presas[1]];
    assert!(
        !super::alguma_por_alcance(&sim, todas(), []),
        "no nascimento nenhuma esta' por alcance, e o chip acendeu"
    );
    super::escreve(
        &mut sim,
        [presas[1]],
        [],
        ph2d_skeleton_ecs::SkinLaw::Envelope,
    );
    assert!(
        super::alguma_por_alcance(&sim, todas(), []),
        "com UMA das duas por alcance o chip ficou apagado: uma seleccao MISTA passa a mentir"
    );
}
