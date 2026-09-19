//! **OS GATES DA MANCHA** — o domínio local dos dois campos.
//!
//! ⚠️ **O oráculo NÃO é *«o campo parece bom»*.** A propriedade central é uma
//! **igualdade ao bit**: correr a lei na pegada e correr a lei na peça inteira
//! com a mesma fronteira pregada têm de dar **o mesmo número** em cada vértice
//! de miolo. Se isso for verdade, tudo o que a cadeia de retopologia já provou
//! sobre a lei vale aqui sem se medir outra vez; se for falso, a mancha é um
//! segundo motor disfarçado.
//!
//! ```text
//! cargo test -p ph2d-quadflow regiao
//! ```

use ph2d_mesh::{Mesh, QueryScratch, shapes};

use super::{mancha, orientacao_semeada};

/// A peça das medições: curvatura real nas duas direcções, e densidade
/// suficiente para uma pegada ter miolo.
fn peca() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

fn pegada(mesh: &Mesh, centro: [f32; 3], raio: f32) -> Vec<u32> {
    let mut scratch = QueryScratch::default();
    let mut out = Vec::new();
    mesh.verts_in_sphere(centro, raio, &mut scratch, &mut out);
    out
}

const TRACO: [f32; 3] = [0.371, 0.642, -0.183];
const ITERACOES: usize = 6;

/// ⭐⭐⭐ **O MIOLO DE UMA MANCHA É A PEÇA INTEIRA, AO BIT.**
///
/// A metade que torna este módulo um domínio e não um motor. Correm-se as duas:
///
/// - na **mancha**, com a franja pregada;
/// - na **peça inteira**, com **tudo menos o miolo daquela mancha** pregado.
///
/// Os dois percorrem o miolo na mesma ordem relativa (a [`Mancha::ids`] é
/// crescente), lêem os mesmos pesos (um vértice de miolo tem o anel inteiro
/// dentro) e partem da mesma semente. ⇒ **igualdade exacta**, sem epsilon.
///
/// ⚠️ **O controlo POSITIVO está dentro**: a mancha tem de ter miolo, senão a
/// asserção percorre o conjunto vazio e fica verde sobre nada.
#[test]
fn o_miolo_de_uma_mancha_e_a_peca_inteira() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    assert!(
        m.miolo() >= 20,
        "a pegada tem de ter miolo para a asserção medir alguma coisa: {}",
        m.miolo()
    );

    let local = orientacao_semeada(&m, TRACO, ITERACOES);

    // A peça inteira, semeada igual e com tudo fora do miolo pregado.
    let normais = mesh.normals();
    let mut global: Vec<[f32; 3]> = normais
        .iter()
        .map(|&n| crate::orientation::project_tangent(TRACO, n))
        .collect();
    let mut fixos = vec![true; mesh.vert_count()];
    for (i, &v) in m.ids.iter().enumerate() {
        if !m.fronteira[i] {
            fixos[v as usize] = false;
        }
    }
    let adj = crate::im_weights::cotangent_adjacency(&mesh);
    crate::orientation::smooth_on_fixed(&mut global, normais, &adj, &fixos, ITERACOES);

    let mut conferidos = 0usize;
    for ((i, &v), &d) in m.ids.iter().enumerate().zip(&local) {
        if m.fronteira[i] {
            continue;
        }
        assert_eq!(
            d, global[v as usize],
            "o vértice de miolo {v} discorda entre a mancha e a peça"
        );
        conferidos += 1;
    }
    assert_eq!(conferidos, m.miolo());
}

/// ⚠️ **A franja não se move** — ela é condição de fronteira, e a promessa do
/// pincel (*fora da pegada, nem um bit*) começa aqui.
#[test]
fn a_franja_fica_onde_a_semente_a_pos() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    let semente: Vec<[f32; 3]> = m
        .nrm
        .iter()
        .map(|&n| crate::orientation::project_tangent(TRACO, n))
        .collect();
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);

    let mut presos = 0usize;
    for (i, (&d, &s)) in dirs.iter().zip(&semente).enumerate() {
        if !m.fronteira[i] {
            continue;
        }
        assert_eq!(d, s, "a franja {i} moveu-se");
        presos += 1;
    }
    assert!(presos > 0, "a pegada tem de ter franja");
}

/// ⭐ **O CAMPO SEGUE O TRAÇO** — na chapa, onde a resposta se sabe de cabeça.
///
/// Sobre um plano toda normal é a mesma, a projecção do traço é a mesma em todo
/// vértice, e a suavização é um **ponto fixo**: o campo é o traço, exactamente.
/// ⚠️ É a régua que separa *«a lei corre»* de *«a lei corre e faz o que diz»*.
#[test]
fn na_chapa_o_campo_e_o_traco() {
    let mesh = chapa(12);
    let ids: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);
    assert!(m.miolo() >= 40, "miolo: {}", m.miolo());

    let n = m.nrm[0];
    let alvo = crate::orientation::project_tangent(TRACO, n);
    for (i, &d) in dirs.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        // 4-RoSy: a direcção vale a menos de um quarto de volta.
        let c = dot(d, alvo).abs().max(dot(d, cross(n, alvo)).abs());
        assert!(
            c > 0.999_9,
            "o campo desviou do traço no vértice {i}: cos = {c}"
        );
    }
}

/// ⛔ **Sem direcção não há lei** — e o campo sai VAZIO em vez de um eixo
/// inventado. A degenerescência que o pente já declara.
#[test]
fn uma_direccao_nula_nao_inventa_eixo() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    assert!(orientacao_semeada(&m, [0.0, 0.0, 0.0], ITERACOES).is_empty());
}

/// ⚠️ **Os pesos de uma mancha que cobre tudo são os da peça** — a prova de que
/// a porta nova não é uma segunda lei.
#[test]
fn os_pesos_de_uma_mancha_que_cobre_tudo_sao_os_da_peca() {
    let mesh = peca();
    let todas: Vec<u32> = (0..mesh.face_count() as u32).collect();
    let pesos = crate::im_weights::cotangent_edge_weights_on(&mesh, &todas);

    let mut adj: Vec<Vec<crate::im_weights::Link>> = vec![Vec::new(); mesh.vert_count()];
    for ((a, b), w) in pesos {
        adj[a as usize].push(crate::im_weights::Link { id: b, weight: w });
        adj[b as usize].push(crate::im_weights::Link { id: a, weight: w });
    }
    for list in &mut adj {
        list.sort_by_key(|l| l.id);
    }

    assert_eq!(adj, crate::im_weights::cotangent_adjacency(&mesh));
}

/// ⚠️ **Uma mancha pode não ter miolo, e isso é um FACTO, não uma falha** — uma
/// pegada mais fina que uma aresta é toda franja. Quem consome tem de conseguir
/// distinguir *«não fez nada»* de *«não havia o que fazer»*.
#[test]
fn uma_pegada_fina_demais_e_toda_franja() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.05);
    let m = mancha(&mesh, &ids);
    assert!(!m.is_empty(), "a pegada não pode ser vazia: seria outro caso");
    assert_eq!(m.miolo(), 0, "miolo: {}", m.miolo());
}

/// Uma chapa `n × n` no plano `z = 0`, triangulada por leque de quadrado.
fn chapa(n: usize) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let x = i as f32 / (n - 1) as f32 - 0.5;
            let y = j as f32 / (n - 1) as f32 - 0.5;
            pos.push([x, y, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let b = a + 1;
            let c = a + n as u32;
            let d = c + 1;
            faces.push(ph2d_mesh::Face::tri(a, b, d));
            faces.push(ph2d_mesh::Face::tri(a, d, c));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa é bem formada")
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// ⭐⭐⭐ **O ALVO DA RETÍCULA É TANGENTE E LIMITADO** — as duas propriedades que
/// tornam esta lei utilizável por um pincel.
///
/// Tangente: a malha **desliza**, não incha nem encolhe. Limitada: o alvo é o
/// ponto da grelha **mais perto** do vértice, logo nunca está a mais de meia
/// diagonal — *um pincel não pode atirar barro para o outro lado da peça*.
#[test]
fn o_alvo_da_reticula_e_tangente_e_limitado() {
    let mesh = peca();
    let ids = pegada(&mesh, [0.0, 0.0, 1.0], 0.55);
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, TRACO, ITERACOES);
    let passo = 0.08;
    let alvos = super::posicao_da_mancha(&m, &dirs, passo, ITERACOES);
    assert_eq!(alvos.len(), m.len());

    let (mut pior_viagem, mut pior_normal) = (0.0f32, 0.0f32);
    for (i, &a) in alvos.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        let d = [a[0] - m.pos[i][0], a[1] - m.pos[i][1], a[2] - m.pos[i][2]];
        pior_viagem = pior_viagem.max(dot(d, d).sqrt() / passo);
        pior_normal = pior_normal.max(dot(d, m.nrm[i]).abs() / passo);
    }
    // Meia diagonal de uma célula quadrada, mais a folga de `f32`.
    assert!(
        pior_viagem <= 0.7072,
        "o alvo saiu da célula: {pior_viagem} passos"
    );
    assert!(
        pior_normal <= 1.0e-5,
        "o alvo saiu do plano tangente: {pior_normal} passos"
    );
}

/// ⭐⭐ **NUMA GRELHA JÁ CERTA A RETÍCULA NÃO MOVE NADA** — o ponto fixo.
///
/// Uma chapa regular de passo `h`, com o traço ao longo de um eixo dela, **já é**
/// a saída desta lei. ⚠️ É a metade que separa *«ela arruma»* de *«ela mexe»*:
/// sem este gate, uma lei que empurrasse tudo meia célula passaria nas outras.
#[test]
fn numa_grelha_ja_certa_a_reticula_nao_move_nada() {
    let n = 12;
    let mesh = chapa(n);
    let h = 1.0 / (n - 1) as f32;
    let ids: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let m = mancha(&mesh, &ids);
    let dirs = orientacao_semeada(&m, [1.0, 0.0, 0.0], ITERACOES);
    let alvos = super::posicao_da_mancha(&m, &dirs, h, ITERACOES);

    let mut pior = 0.0f32;
    let mut conferidos = 0usize;
    for (i, &a) in alvos.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        let d = [a[0] - m.pos[i][0], a[1] - m.pos[i][1], a[2] - m.pos[i][2]];
        pior = pior.max(dot(d, d).sqrt() / h);
        conferidos += 1;
    }
    assert!(conferidos >= 40, "miolo: {conferidos}");
    assert!(pior <= 1.0e-4, "a chapa certa moveu-se: {pior} passos");
}
