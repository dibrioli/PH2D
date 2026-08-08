//! **OS GATES DA ESCALA** — o que a recomendação promete, medido.
//!
//! ⚠️ **Eles viajaram junto com o assunto deles.** Nasceram no `alpha_tests.rs`,
//! quando `recommended_scale` morava no módulo do padrão; hoje ela é do
//! [`super`], e um gate que citasse os privados dela de fora **não compila** — o
//! que é a forma certa de descobrir que um teste ficou órfão do que ele mede.

use super::*;

/// **A recomendação é RESOLVIDA pela malha de onde ela saiu** — as duas metades.
///
/// Ela é a mais grossa de duas restrições, e um gate que medisse só uma delas
/// deixaria passar exatamente o defeito que o smoke pegou:
///
/// * numa malha DENSA manda o LOOK, e o padrão tem de atravessar o modelo ~33
///   vezes — foi a falta desta metade que pôs oito crateras numa esfera;
/// * numa malha GROSSA manda a lei das dez arestas, e o padrão sai grosso
///   porque a malha não comporta outro — honesto, e a cura é subdividir.
#[test]
fn the_recommended_scale_is_resolved_by_the_mesh_it_came_from() {
    use ph2d_mesh::shapes;
    for (u, v, dense) in [
        (24usize, 36usize, false),
        (96, 144, false),
        (700, 1050, true),
    ] {
        let mesh = shapes::uv_sphere(u, v, 1.0);
        let s = crate::recommended_scale(&mesh);
        let edge = sampled_edge(&mesh);
        // ⚠️ **O `min` com o teto NÃO é folga, é um REGIME que este gate
        // descobriu.** Na esfera 24×36 a aresta mede `0,131`, então a lei das dez
        // arestas pediria `1,31` — mais que o modelo inteiro. Não há escala que
        // salve aquela malha: ela **não carrega padrão nenhum**, e a única cura é
        // subdividir. O que a recomendação faz é pousar no teto, que é o estado
        // reconhecível; escolher um valor no meio seria fingir que resolveu.
        let want = (EDGES_PER_FEATURE * edge).min(MAX_ALPHA_SCALE);
        assert!(
            s >= want * 0.999,
            "uv_sphere({u},{v}): a recomendação {s} é mais fina que 10 arestas ({edge}) \
             — o padrão sairia como chuvisco"
        );
        // ⚠️ **E o LOOK é um PISO da escala, não um alvo** — esta metade nasceu
        // de uma mutação SOBREVIVENTE: com a recomendação reduzida à lei das dez
        // arestas (`floor` sozinho) o gate ficava verde, porque a asserção era
        // *"atravessa mais de 20 features"* e numa malha densa o `floor` sozinho
        // atravessa muito mais que isso. A afirmação certa é a que o `max` faz:
        // **a recomendação nunca é mais fina que `span ÷ 33`**, por mais densa
        // que a malha seja. Sem ela, uma peça de um milhão de vértices receberia
        // um padrão fino demais para o olho — o oposto exato do defeito que o
        // smoke reportou, e igualmente inútil.
        let look = 2.0 / FEATURES_ACROSS;
        assert!(
            s >= look * 0.999,
            "uv_sphere({u},{v}): a recomendação {s} é mais fina que `span ÷ 33` ({look}) \
             — o padrão fica fino demais para se ver"
        );
        // O modelo mede 2 (esfera unitária), então `2 / s` é quantas features o
        // padrão atravessa. Numa malha densa a restrição de LOOK tem de mandar,
        // e a fixture é escolhida FOLGADAMENTE do lado dela: em 533×800 as duas
        // restrições empatam (0,059 contra 0,061) e nenhuma mutação sangraria.
        let across = 2.0 / s;
        if dense {
            assert!(
                across > 20.0,
                "uv_sphere({u},{v}) comporta detalhe e a recomendação só atravessa \
                 {across:.0} features — isso lê como cratera, não como textura"
            );
        }
        assert!(
            (MIN_ALPHA_SCALE..=MAX_ALPHA_SCALE).contains(&s),
            "a recomendação {s} caiu fora da pista"
        );
    }
}

/// **E ela é BARATA**, porque roda num clique.
///
/// ⚠️ A mediana exata sobre 425 k vértices ordena ~1,2 M comprimentos; um
/// engasgo visível ao escolher um padrão seria pior que a precisão que ele
/// compra. O oráculo é a RAZÃO contra uma malha 11× menor: se a estimativa
/// percorresse a malha inteira em vez de amostrar, ela cresceria com o modelo.
#[test]
fn the_recommendation_does_not_walk_the_whole_mesh() {
    use ph2d_mesh::shapes;
    use std::time::Instant;
    let small = shapes::uv_sphere(48, 72, 1.0);
    let big = shapes::uv_sphere(160, 240, 1.0);
    assert!(
        big.vert_count() > small.vert_count() * 8,
        "a fixture não escala"
    );

    let t = Instant::now();
    for _ in 0..20 {
        std::hint::black_box(crate::recommended_scale(&small));
    }
    let a = t.elapsed().as_secs_f64();
    let t = Instant::now();
    for _ in 0..20 {
        std::hint::black_box(crate::recommended_scale(&big));
    }
    let b = t.elapsed().as_secs_f64();
    assert!(
        b < a * 4.0,
        "a recomendação custa {:.2}× mais numa malha 11× maior — ela está \
         percorrendo a malha em vez de amostrá-la",
        b / a
    );
}
