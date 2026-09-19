//! Os gates de [`super::dab_alcance`] — a máscara de alcance.
//!
//! ⚠️ **Eles vivem DENTRO da crate de propósito:** a porta é `pub(crate)`, e a
//! metade que mais importa — *«numa peça convexa a máscara não corta nada»* —
//! precisa de a chamar **directamente**, com e sem. Pela env não se consegue: o
//! `OnceLock` fixa-a no primeiro toque do processo, então um teste que a
//! alternasse mediria o vizinho.

use super::{ALCANCE_TECTO, Alcance};
use ph2d_mesh::{Face, Mesh, QueryScratch, shapes};

/// Duas esferas soltas na MESMA malha, separadas por `folga` no equador — o caso
/// do report: *«esculpir um dedo mexe o vizinho»*.
///
/// ⚠️ **A superfície NÃO as liga**, e é isso que faz esta fixtura decidir: a
/// distância pela superfície é `∞` e nenhum epsilon a explica. ⛔ O
/// `shapes::cylinder` da casa não serve — ele tem dois anéis e não há vértice
/// nenhum a meia altura (a sonda pagou isso com oito células a `NaN`).
fn dois_dedos(folga: f32) -> Mesh {
    let uma = shapes::uv_sphere(48, 96, 1.0);
    let n = uma.vert_count() as u32;
    let dx = 2.0 + folga;
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(uma.vert_count() * 2);
    for p in uma.positions() {
        pos.push([p[0] - dx * 0.5, p[1], p[2]]);
    }
    for p in uma.positions() {
        pos.push([p[0] + dx * 0.5, p[1], p[2]]);
    }
    let mut faces = uma.faces().to_vec();
    for f in uma.faces() {
        let v: Vec<u32> = f.verts().iter().map(|i| i + n).collect();
        faces.push(if v.len() == 3 {
            Face::tri(v[0], v[1], v[2])
        } else {
            Face::quad(v[0], v[1], v[2], v[3])
        });
    }
    Mesh::from_parts(pos, faces).expect("duas esferas soltas continuam uma malha valida")
}

fn pegada(mesh: &Mesh, centro: [f32; 3], raio: f32) -> Vec<u32> {
    let mut s = QueryScratch::default();
    let mut out = Vec::new();
    mesh.verts_in_sphere(centro, raio, &mut s, &mut out);
    out
}

/// ⭐⭐⭐ **A METADE QUE PROTEGE TUDO O QUE JÁ ESTÁ APROVADO: numa peça convexa a
/// máscara corta ZERO.**
///
/// É esta que faz o caminho de omissão ser byte-idêntico — o peso de quem fica
/// nunca muda, então se ninguém sai, a saída é a mesma ao bit. ⛔ Um futuro
/// aperto do [`ALCANCE_TECTO`] reprova aqui, e é a intenção: a primeira coluna
/// da tabela dele mede exactamente o que se perde ao apertá-lo.
#[test]
fn numa_peca_convexa_a_mascara_nao_corta_nada() {
    let m = shapes::uv_sphere(64, 128, 1.0);
    let mut a = Alcance::default();
    for r in [0.10f32, 0.20, 0.35, 0.50] {
        let mut p = pegada(&m, [0.0, 0.0, 1.0], r);
        let antes = p.len();
        assert!(
            antes > 8,
            "a fixtura tem de apanhar vertices (raio {r}): {antes}"
        );
        let cortados = a.corta(&m, [0.0, 0.0, 1.0], r, &mut p);
        assert_eq!(
            cortados, 0,
            "a mascara cortou {cortados} de {antes} numa ESFERA CONVEXA (raio {r}) -- \
             ali a superficie alcanca tudo, e cortar peso e' a cura a mexer no que \
             ja' esta' certo. O tecto e' {ALCANCE_TECTO}xR."
        );
    }
}

/// ⭐⭐⭐ **E a metade que CURA: o dedo vizinho sai do carimbo.**
///
/// ⚠️ **O olho vem de `+z` e o carimbo pousa no alto do dedo da ESQUERDA, junto
/// ao vinco** — que é o gesto real. ⛔ Um olho ao longo de `x` nunca produz este
/// carimbo: dali o raio bate no lado de fora do dedo da direita, e o vinco fica
/// ocluso.
#[test]
fn o_dedo_vizinho_sai_do_carimbo() {
    let folga = 0.05f32;
    let m = dois_dedos(folga);
    // Um ponto no alto do dedo da esquerda, a 20° do equador dele.
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut p = pegada(&m, centro, raio);
    let direita = |v: u32, p: &Mesh| p.positions()[v as usize][0] > 0.0;
    let antes_dir = p.iter().filter(|&&v| direita(v, &m)).count();
    assert!(
        antes_dir > 0,
        "a fixtura nao produz o fenomeno: o carimbo nao alcanca o dedo da direita"
    );
    let antes_esq = p.len() - antes_dir;
    assert!(antes_esq > 0, "e tem de apanhar o dedo de CA' tambem");

    let mut a = Alcance::default();
    a.corta(&m, centro, raio, &mut p);

    let depois_dir = p.iter().filter(|&&v| direita(v, &m)).count();
    let depois_esq = p.len() - depois_dir;
    assert_eq!(
        depois_dir, 0,
        "sobraram {depois_dir} de {antes_dir} vertices do dedo VIZINHO no carimbo -- \
         a superficie nao os liga de forma alguma"
    );
    assert_eq!(
        depois_esq,
        antes_esq,
        "a mascara comeu {} vertices do dedo de CA' -- ela cortaria o lado que o \
         artista esta' a esculpir, que e' defeito pior que o que ela cura",
        antes_esq - depois_esq
    );
}

/// ⭐⭐ **A SEMENTE cai no dedo de CÁ mesmo com a folga MUITO menor que uma
/// aresta — e é este gate que provou que o `SEMENTE_RECUO` era peso morto.**
///
/// A folga aqui é `0,002` contra uma aresta de `~0,065` no equador: se o vértice
/// mais próximo pudesse saltar de folha, era aqui. Ele não salta, e a razão está
/// escrita na nota apagada do [`super`] — *o centro do dab está SOBRE a folha de
/// cá, e todo vértice da de lá é um da de cá mais a espessura.*
#[test]
fn com_a_folga_menor_que_uma_aresta_a_semente_cai_no_dedo_de_ca() {
    let folga = 0.002f32; // a aresta da esfera 48x96 no equador mede ~0,065
    let m = dois_dedos(folga);
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut p = pegada(&m, centro, raio);
    let antes_esq = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] < 0.0)
        .count();
    let mut a = Alcance::default();
    a.corta(&m, centro, raio, &mut p);
    let depois_esq = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] < 0.0)
        .count();
    let depois_dir = p.len() - depois_esq;
    assert_eq!(depois_dir, 0, "o dedo vizinho ficou no carimbo");
    assert_eq!(
        depois_esq, antes_esq,
        "a semente caiu no dedo ERRADO: a mascara cortou o lado que se esculpe"
    );
}

/// ⚠️⚠️ **A época pode dar a volta, e nesse tique a máscara PÁRA DE MASCARAR.**
///
/// Com a época em `0`, a marca `0` do vector recém-criado volta a valer «visto
/// nesta corrida» ⇒ o `retain` **guarda tudo**, o dedo vizinho incluído. É a
/// mesma cerca que o [`ph2d_mesh::QueryScratch`] documenta, e este gate corre-a.
///
/// ⛔⛔ **A 1.ª redacção deste gate media uma ESFERA CONVEXA e a mutação que
/// apaga a cerca SOBREVIVEU** — ali não há nada para cortar, então «parou de
/// cortar» é invisível. *A fixtura tem de conter o fenómeno, e a de um gate de
/// cerca é a peça onde a cerca tem consequência.*
///
/// ⚠️ E a chamada de AQUECIMENTO é obrigatória: numa `Alcance` nova o primeiro
/// `corta` aloca os buffers e **reescreve a época para `0`**, o que apagaria o
/// `u32::MAX` que este gate acabou de forçar.
#[test]
fn a_volta_da_epoca_nao_apaga_a_mascara() {
    let folga = 0.05f32;
    let m = dois_dedos(folga);
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut a = Alcance::default();

    // (a) aquecer — e' esta chamada que aloca os buffers.
    let mut aquecer = pegada(&m, centro, raio);
    a.corta(&m, centro, raio, &mut aquecer);

    // (b) um passo antes do wrap, e a corrida a seguir cai em `0`.
    a.forcar_epoca_para_teste(u32::MAX);
    let mut p = pegada(&m, centro, raio);
    let antes_dir = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] > 0.0)
        .count();
    assert!(antes_dir > 0, "a fixtura nao produz o fenomeno");
    a.corta(&m, centro, raio, &mut p);
    let depois_dir = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] > 0.0)
        .count();
    assert_eq!(
        depois_dir, 0,
        "na volta da epoca a mascara guardou {depois_dir} de {antes_dir} vertices do \
         dedo vizinho -- ela parou de mascarar, uma vez a cada quatro bilhoes de \
         dabs e impossivel de reproduzir sem esta porta"
    );
}

/// Sonda: o que a máscara corta na GRELHA PLANA do corpus do projectar — onde a
/// resposta certa é «nada», porque uma chapa é convexa por dentro.
#[test]
#[ignore = "sonda: imprime, nao afirma nada"]
fn diag_o_que_a_mascara_corta_numa_grelha_plana() {
    const LADO: usize = 41;
    let passo = 2.0 / (LADO - 1) as f32;
    let mut pos = Vec::new();
    for i in 0..LADO {
        for j in 0..LADO {
            pos.push([-1.0 + passo * i as f32, -1.0 + passo * j as f32, 0.0]);
        }
    }
    let at = |i: usize, j: usize| (i * LADO + j) as u32;
    let mut faces = Vec::new();
    for i in 0..LADO - 1 {
        for j in 0..LADO - 1 {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    let mesh = Mesh::from_parts(pos, faces).expect("a grelha");
    let centro = [0.0, 0.0, 0.0];
    for raio in [0.35f32, 0.40] {
        let p = pegada(&mesh, centro, raio);
        let mut q = p.clone();
        let mut a = Alcance::default();
        let cortou = a.corta(&mesh, centro, raio, &mut q);
        println!("raio {raio:.2}: pegada {} · cortou {cortou}", p.len());
        if cortou > 0 {
            let ficou: std::collections::BTreeSet<u32> = q.iter().copied().collect();
            for &v in p.iter().filter(|v| !ficou.contains(v)).take(6) {
                let d = mesh.positions()[v as usize];
                let ar = ((d[0] - centro[0]).powi(2) + (d[1] - centro[1]).powi(2)).sqrt();
                println!("   cortado v={v} ar={ar:.4} ({:.2}x o raio)", ar / raio);
            }
        }
    }
}

/// ⭐⭐⭐ **O TECTO ABSOLUTO governa o TRABALHO, e é aí que ele tem de ser
/// medido.**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Desde que a lei do
/// corte é a [`super::RAZAO_MAXIMA`], apagar o [`super::ALCANCE_TECTO`] **não
/// muda um único vértice da saída** — os quatro gates deste módulo ficam verdes
/// — e passa a varrer a malha INTEIRA a cada dab. *Um tecto que deixou de
/// governar a resposta e passou a governar só o custo precisa de mudar de
/// régua junto.*
///
/// ⭐ **A régua é uma CONTAGEM e não um relógio**, de propósito: uma contagem é
/// determinista, e um gate de razão entre dois tempos é candidato à família de
/// flakes de fan-out que o `CLAUDE.md` §5.0 mantém.
#[test]
fn a_varredura_e_limitada_pela_pegada_e_nao_pela_malha() {
    let mesh = shapes::uv_sphere(128, 256, 1.0);
    let n = mesh.vert_count();
    let centro = [0.0, 0.0, 1.0];
    let raio = 0.10;
    let mut p = pegada(&mesh, centro, raio);
    let mut a = Alcance::default();
    a.corta(&mesh, centro, raio, &mut p);
    let vistos = a.visitados_no_teste();
    assert!(
        vistos > 20,
        "a varredura mal correu ({vistos} de {n}) — o gate esta' a medir um no-op"
    );
    // ⚠️ O tecto é `2 × 0,10 = 0,20` numa esfera de raio `1`, ou seja uma calota
    // de `~1 %` da area. Sem o tecto isto lê `n`.
    assert!(
        vistos * 20 < n,
        "a varredura fixou {vistos} de {n} vertices com um pincel de raio {raio} — \
         o tecto absoluto deixou de limitar o trabalho, e o custo passou a ser \
         O(malha) por dab"
    );
}
