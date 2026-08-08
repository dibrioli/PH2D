//! Ver o `sculpt3d_scenes.rs` — este arquivo é o `mod tests` dele, cortado
//! quando o pai cruzou o teto de 600 LOC do HR-18.

use super::*;

use super::*;

/// **A cena `=16` só significa alguma coisa se a malha dela RESOLVER o
/// padrão** — e isso é um fato sobre a GEOMETRIA que nenhum arch-gate de
/// fonte enxerga.
///
/// A lei das dez arestas (`ph2d_sculpt3d::DEFAULT_ALPHA_SCALE`) mede pela
/// razão `célula ÷ aresta`: abaixo de ~8 a correlação entre vértices
/// vizinhos desmorona e o que chega ao barro é chuvisco por vértice. A
/// esfera 96×144 que o resto do módulo abre daria ~7,6 — o smoke julgaria o
/// aliasing e chamaria de alpha.
#[test]
fn the_alpha_scene_opens_dense_enough_to_resolve_the_pattern() {
    let mesh = ph2d_mesh::shapes::uv_sphere(533, 800, 1.0);
    let pos = mesh.positions();
    let ring = mesh.adjacency();
    let mut lens: Vec<f32> = Vec::new();
    for v in 0..pos.len() {
        for &n in ring.vert_verts.neighbours(v) {
            if n as usize > v {
                let (a, b) = (pos[v], pos[n as usize]);
                let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                lens.push(d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt());
            }
        }
    }
    lens.sort_by(f32::total_cmp);
    let edge = lens[lens.len() / 2];
    let ratio = ph2d_sculpt3d::DEFAULT_ALPHA_SCALE / edge;
    assert!(
        ratio >= 8.0,
        "a malha da cena `=16` tem aresta {edge:.4}, e na escala default a \
         razão é {ratio:.1} — abaixo de 8 o padrão sai como chuvisco"
    );
}

/// **A `=21` abre com a MESMA malha que a `=16`, e as duas resolvem pela
/// MESMA condição.**
///
/// ⚠️ **Um arch-gate de FONTE, e não um teste que dirige o roteador** — o
/// shell é `forbid(unsafe_code)` e mover a env var (a única entrada do
/// roteador) exige um bloco `unsafe` desde a edição 2024. A alternativa
/// óbvia — reconstruir `uv_sphere(533, 800, 1.0)` aqui e comparar — é o
/// espelho que o doc do [`smoke_mesh`] proíbe em letra: ele mediria a malha
/// que EU escrevi, não a que a cena abre, e ficaria verde no dia em que a
/// cena mudasse de esfera.
///
/// ⚠️ **A propriedade afirmada é o `||`**, e ela é o que impede o modo de
/// falha real: um `if` novo entre as duas cenas as separaria em silêncio, a
/// `=21` cairia na esfera de 96×144 do fim da função, e o smoke do EIXO
/// passaria a julgar um estrato picado — que é indistinguível de o eixo não
/// fazer nada. O controle positivo é a linha da `=16`: sem ela a varredura
/// estaria a medir o vácuo.
#[test]
fn the_axis_scene_opens_on_the_same_dense_mesh_as_the_alpha_scene() {
    // ⚠️ Relativo ao PACOTE e não `file!()`: a cwd de um teste é o diretório
    // do `Cargo.toml`, e o `file!()` vem da raiz da workspace — é a mesma
    // régua que o `no_two_sculpt3d_scenes_claim_the_same_level` usa.
    // ⚠️ **O arquivo do PAI, e não este.** Enquanto o `mod tests` morava dentro
    // do `sculpt3d_scenes.rs` os dois eram o mesmo arquivo; o corte de LOC os
    // separou, e um `file!()` aqui passaria a ler o arquivo dos TESTES — que não
    // tem roteador nenhum, e onde a busca acharia o vácuo. O caminho é relativo
    // ao PACOTE (a cwd de um teste é o diretório do `Cargo.toml`), a mesma régua
    // do `no_two_sculpt3d_scenes_claim_the_same_level`.
    let src = std::fs::read_to_string("src/sculpt3d_scenes.rs")
        .expect("o roteador de cenas é legível a partir do pacote");
    // ⚠️ **A busca é pela CONDIÇÃO, e o `starts_with("if ")` é o que a torna
    // honesta** — a primeira versão deste gate procurou só as duas chamadas
    // na mesma linha e casou com `pub(crate) fn directional_alpha_scene()`,
    // porque *"directional_alpha_scene()"* **contém** *"alpha_scene()"* como
    // substring. Verde sobre a definição, sem nunca olhar o roteador.
    let shared = src.lines().map(str::trim_start).find(|l| {
        l.starts_with("if ")
            && l.contains("alpha_scene()")
            && l.contains("directional_alpha_scene()")
    });
    assert!(
        shared.is_some(),
        "a `=21` deixou de resolver a malha na MESMA condição da `=16` — \
         ela vai abrir na esfera grossa e o estrato sai picado"
    );
}

/// ⚠️ **A cena `=6` só significa alguma coisa se o bico dela estiver
/// ESTICADO** — e a forma sobrevive ao remesh nos dois casos, então a
/// densidade é a única coisa que separa a feature funcionando da morta. O
/// oráculo é a maior ARESTA, que é a medida do esticamento.
#[test]
fn the_remesh_scene_opens_with_a_stretched_spike() {
    let mesh = hooked_sphere();
    let pos = mesh.positions();
    let mut tris = Vec::new();
    mesh.triangle_indices(&mut tris);
    let mut longest = 0.0f32;
    for t in &tris {
        for k in 0..3 {
            let a = pos[t[k] as usize];
            let b = pos[t[(k + 1) % 3] as usize];
            longest = longest.max(
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
            );
        }
    }
    // A esfera de 48×72 tem aresta ~0.09 em repouso; o gancho tem de
    // multiplicar isso, senão não há barro gasto a demonstrar.
    assert!(
        longest > 0.15,
        "a maior aresta mede {longest:.4}: o gancho nao esticou nada"
    );
    // E a ponta tem de ter SAÍDO da esfera — um bico que não anda é um
    // esticamento que o olho não encontra.
    let far = mesh
        .positions()
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .fold(0.0f32, f32::max);
    assert!(far > 1.5, "a ponta chegou so' a {far:.3} de raio");
}

/// ⚠️ **A cena `=5` só significa alguma coisa se a esfera dela TIVER cristas**,
/// e isso é um fato sobre geometria que nenhum arch-gate de fonte enxerga —
/// o mesmo argumento do gate da cena `=3`, que pina que ela é construída
/// subdividindo.
///
/// ⚠️ **O oráculo tem duas metades, e a segunda é a que importa:** a crista
/// tem de subir E a região LISA tem de ficar lisa. Só a primeira ficaria
/// verde se o traço vazasse pela esfera inteira — e aí a fixture não teria
/// forma a seguir, que é exatamente o que ela existe para dar.
#[test]
fn the_turn_scene_opens_with_a_sphere_that_has_ridges() {
    let mesh = ridged_sphere();
    let (mut on, mut off) = (0.0f32, 0.0f32);
    for p in mesh.positions() {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        // A cruz vive na calota `+Z`, ao longo dos planos `y = 0` e `x = 0`.
        if p[2] < 0.7 {
            continue;
        }
        if p[0].abs() < 0.05 || p[1].abs() < 0.05 {
            on = on.max(r - 1.0);
        } else if p[0].abs() > 0.3 && p[1].abs() > 0.3 {
            off = off.max((r - 1.0).abs());
        }
    }
    assert!(
        on > 0.04,
        "a crista subiu só {on:.4} do raio — numa esfera de diâmetro 2 isso não se segue com o olho"
    );
    assert!(
        off < 0.005,
        "a região LISA subiu {off:.4}: o traço vazou, e a fixture perdeu a forma que ela existe para dar"
    );
}
