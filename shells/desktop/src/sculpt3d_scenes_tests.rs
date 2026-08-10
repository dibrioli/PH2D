//! Ver o `sculpt3d_scenes.rs` — este arquivo é o `mod tests` dele, cortado
//! quando o pai cruzou o teto de 600 LOC do HR-18.

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

/// **A cena do transform tem de CONTER a banda** — sem ela o smoke julgaria uma
/// propriedade que não está na tela.
///
/// ⚠️ É a lição das fixtures dos gates do kernel, aplicada à cena: com máscara
/// DURA as duas leis de interpolação concordam em todo vértice, e o defeito que
/// esta wave corrige (o colapso do lerp) só é visível onde o peso é PARCIAL. Uma
/// cena de smoke é uma fixture, e uma fixture que não contém o fenômeno faz o
/// artista aprovar o que ele não viu.
#[test]
fn the_transform_scene_has_a_soft_band_to_judge() {
    let (band, total) = crate::sculpt3d::soft_masked_counts();
    assert!(total > 10_000, "a esfera da cena tem so' {total} vertices");
    // ⚠️ **A barra é um QUARTO, e o número saiu da medição depois de ela
    // derrubar a minha conta.** Eu tinha escrito *"a maior parte da esfera"* (>
    // metade) supondo que o peso `0,5 + y` distribuísse os vértices por `y`; uma
    // `uv_sphere` os distribui uniformemente no ÂNGULO POLAR, e `|y| < 0,48` são
    // 59° de 180° — **medido, 4464 de 13682 = 32,6%**. Um terço é banda de
    // sobra para julgar; a barra fica abaixo dele com folga, porque o que ela
    // guarda é *a cena contém o fenômeno*, não a aritmética da esfera.
    assert!(
        band * 4 > total,
        "a banda tem {band} de {total} vertices -- a cena nao contem o fenomeno"
    );
    // ⚠️ E os DOIS extremos têm de existir: sem vértice de peso 0 não há o que
    // fique parado, e sem peso 1 não há o que se mova inteiro.
    assert!(band < total, "a esfera nao tem extremo nenhum");
}

/// **O REMESH REPETIDO COLAPSA A PEÇA** — o report do Enio, com a fixture que
/// ele tem em mãos.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bins does_repeating_the_remesh -- --ignored --nocapture
/// ```
///
/// ⚠️ **Esta sonda existe porque a minha varredura anterior mediu a malha
/// ERRADA.** Eu varri 480..512 numa `uv_sphere` e numa esfera com bico, achei
/// zero colapsos e declarei o vazamento parcial descartado. O log do produto diz
/// `567828 -> 40 vertices ... 62597843 celulas` — nem a contagem de células nem
/// a linhagem batem com o que varri: o caminho REAL é *o remesh de um remesh*, e
/// a saída de Surface Nets já nasce alinhada a uma grade, contra uma grade nova
/// quase-comensurável. A fixture tem de ser a CADEIA, na peça da cena.
#[test]
#[ignore = "sonda de medição"]
fn does_repeating_the_remesh_collapse_the_piece() {
    let mut mesh = hooked_sphere();
    eprintln!("entrada: {} v / {} f", mesh.vert_count(), mesh.face_count());
    for round in 1..=8 {
        let before = mesh.vert_count();
        match ph2d_sdf::remesh(&mesh, 512) {
            Ok((out, r)) => {
                eprintln!(
                    "  #{round}: {before} -> {} v ({} celulas)",
                    out.vert_count(),
                    r.cells
                );
                if out.vert_count() * 100 < before {
                    eprintln!("  #{round}: >>> COLAPSO <<<");
                    return;
                }
                mesh = out;
            }
            Err(e) => {
                eprintln!("  #{round}: RECUSA -- {e}");
                return;
            }
        }
    }
    eprintln!("  8 rodadas sem colapso");
}

/// **O vazamento é LOTERIA de alinhamento, ou estrutural ao remesh-de-remesh?**
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bins how_often_a_remeshed_mesh -- --ignored --nocapture
/// ```
///
/// As duas respostas pedem curas OPOSTAS. Se poucas resoluções vazam, o campo
/// tropeça num alinhamento raro e a cura é perturbar a grade (a saída padrão
/// para degenerescência em geometria computacional). Se a MAIORIA vaza, a saída
/// do Surface Nets é hostil ao voxelizador por construção, e perturbar não
/// salva — a cura teria de estar na marca de travessia.
#[test]
#[ignore = "sonda de medição"]
fn how_often_a_remeshed_mesh_leaks() {
    // O 1º remesh é o que o produto faz sem reclamar; a peça DELE é a entrada.
    let (once, _) = ph2d_sdf::remesh(&hooked_sphere(), 512).expect("o 1o remesh passa");
    eprintln!(
        "malha remeshada: {} v / {} f",
        once.vert_count(),
        once.face_count()
    );
    let mut bad = Vec::new();
    let total = 500u32..=520;
    let n = total.clone().count();
    for res in total {
        match ph2d_sdf::remesh(&once, res) {
            Ok((out, _)) if out.vert_count() * 100 >= once.vert_count() => {}
            Ok((out, _)) => bad.push((res, format!("caco {} v", out.vert_count()))),
            Err(e) => bad.push((res, format!("{e}"))),
        }
    }
    eprintln!("  {} de {n} resolucoes falham: {bad:?}", bad.len());
}

/// **RECONSTRUIR NUNCA DESTRÓI A PEÇA EM SILÊNCIO** — o report do Enio, virado
/// gate, na fixture da cena `=6`.
///
/// Log do produto (2026-08-10): `567828 -> 40 vertices`. O guard que shipava
/// perguntava *sobrou alguma coisa?*, e um campo que vaza QUASE todo responde
/// *sim* — a extração devolve um caco, o chamador o instala, e o log diz
/// SUCESSO. É a pior forma de errado, porque parece que funcionou.
///
/// ⚠️ **A afirmação é a DISJUNÇÃO, não o sucesso.** Este gate não exige que o
/// remesh funcione em toda resolução: o vazamento é do campo e curá-lo é outra
/// pergunta. Ele exige que o resultado seja *uma peça de verdade* **ou** *uma
/// recusa nomeada* — nunca um caco com `Ok`. Um gate que pedisse sucesso ficaria
/// vermelho sobre um produto que está a proteger o artista corretamente.
///
/// ⚠️ E ele encadeia, porque **a cadeia é o fenômeno**: a saída do Surface Nets
/// nasce alinhada a uma grade, e é contra a grade SEGUINTE que ela degenera. Um
/// remesh isolado não reproduz.
#[test]
fn remeshing_over_and_over_never_installs_a_shard() {
    let mut mesh = hooked_sphere();
    for round in 1..=4 {
        let before = mesh.vert_count();
        match ph2d_sdf::remesh(&mesh, 512) {
            Ok((out, _)) => {
                assert!(
                    out.vert_count() * 100 >= before,
                    "rodada {round}: `Ok` com {} vertices contra {before} -- \
                     o chamador instala isto e a escultura SOME com log de sucesso",
                    out.vert_count()
                );
                mesh = out;
            }
            // Uma recusa e' o resultado CERTO aqui: a peca fica como esta'.
            Err(_) => return,
        }
    }
}
