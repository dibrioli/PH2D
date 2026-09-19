//! ⛔⛔⛔⛔ **A REPRODUÇÃO DO REPORT DE 2026-09-19** — *«Snake Hook se dá muito
//! mal com `Connected Only`, deformando com má remesh ou má topologia a face
//! POSTERIOR do traço»* (foto: a bossa puxada com as costas **pretas**,
//! rasgadas, e um refino explodido à volta delas).
//!
//! # ⚠️ Porque a sonda mora AQUI e não na `ph2d-sculpt3d`
//!
//! A irmã de lá ([`sonda_do_gancho`](../../ph2d-sculpt3d/tests/it/sonda_do_gancho.rs))
//! corre o gancho **sem** passe de topologia e mede a máscara a cortar
//! **ZERO** numa esfera lisa — o que está certo e não é o report. A foto tem as
//! **duas** metades a correr juntas, e o passe é do EDITOR: ele parte a aresta
//! LONGA, logo *um rasgo a montante fabrica o refino a jusante*.
//!
//! ⇒ a sonda corre o laço do produto (`passe_nos_motores` → `dab`, com os dois
//! canais do traço em voo), que é o mesmo que o [`crate::scenes_pente_tests`]
//! já usa — ⛔ e **não** o `Sculpt3dScene`, que pede um device e poria a
//! resposta atrás de um `#[ignore]` que o CI nunca corre.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// O olho da cena — a câmara olha no `−z`.
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

fn norma(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// O que um traço de gancho deixa, medido como a foto se lê.
pub(crate) struct Leitura {
    pub verts: usize,
    /// O maior deslocamento — o puxão, que é suposto ser grande.
    pub puxao: f32,
    /// ⭐ **O RASGO:** a maior diferença de deslocamento entre dois vértices
    /// ligados por uma aresta, em unidades da aresta média de repouso.
    pub rasgo: f32,
    /// A maior aresta no fim, em unidades da aresta média de repouso.
    pub estica: f32,
    /// Faces cuja normal aponta ao contrário da vizinhança — o preto da foto.
    pub avesso: usize,
    /// O pior triângulo (menor ângulo, em graus) — a lasca.
    pub lasca: f32,
    /// A PONTA da bossa: a maior distância à origem. ⚠️ **Não é o maior `x`** —
    /// a 1.ª redacção usou-o e leu `1,018` sobre um puxão real, porque a esfera
    /// de repouso já mede `1,0` em `x` no equador. A esfera mede `1,0` em toda
    /// parte, logo o que passa disso é barro que saiu da peça.
    pub ponta: f32,
    /// Quantos vértices a peça tinha ANTES — o piso de população do refino.
    pub verts0: usize,
}

/// Um arrasto de gancho com o passe de topologia a correr, como o produto faz:
/// **refina e depois carimba**, com os dois canais (renumeração · nascimentos) a
/// chegarem ao traço em voo.
pub(crate) fn gancho(raio: f32, len: f32, dabs: usize, mascara: bool, alvo: f32) -> Leitura {
    gancho_com(raio, len, dabs, mascara, alvo, true)
}

/// ⚠️ **`com_rasgo` existe por RELÓGIO e está declarado:** a régua do rasgo mede
/// cada vértice do fim contra a nuvem de repouso (`O(n²)` sobre `~97 k`), e uma
/// ATRIBUIÇÃO corre isto quatro vezes. As colunas que a foto mostra — o preto, a
/// lasca e o refino — não precisam dela.
pub(crate) fn gancho_com(
    raio: f32,
    len: f32,
    dabs: usize,
    mascara: bool,
    alvo: f32,
    com_rasgo: bool,
) -> Leitura {
    traco_com(Gesto {
        verbo: Verb::SnakeHook,
        raio,
        len,
        dabs,
        rumo: [1.0, 0.0, 0.0],
        mascara,
        alvo,
        com_rasgo,
    })
}

/// ⚠️ **O gesto é PARAMETRIZADO porque o report seguinte foi *«algumas vezes
/// correto, algumas vezes bugado»***: um defeito intermitente é um defeito cujo
/// regime ninguém varreu, e o que muda entre duas pinceladas do dono é o VERBO,
/// o RUMO e a TAXA de eventos.
pub(crate) struct Gesto {
    pub verbo: Verb,
    pub raio: f32,
    pub len: f32,
    pub dabs: usize,
    /// A direcção do arrasto, em MUNDO. `+x` é tangencial à peça no polo; uma
    /// componente em `+z` puxa na direcção do artista.
    pub rumo: [f32; 3],
    pub mascara: bool,
    pub alvo: f32,
    pub com_rasgo: bool,
}

pub(crate) fn traco_com(g: Gesto) -> Leitura {
    let Gesto {
        verbo,
        raio,
        len,
        dabs,
        rumo,
        mascara,
        alvo,
        com_rasgo,
    } = g;
    let mut malha = ph2d_mesh::shapes::sculpt_sphere(1.0);
    malha.triangulate();
    let repouso: Vec<[f32; 3]> = malha.positions().to_vec();
    let unidade = aresta_media(&malha);
    let brush = Brush {
        verb: verbo,
        radius: raio,
        strength: 1.0,
        surface_only: mascara,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();

    // ⚠️ **O caminho é o da foto:** o cursor arranca do ponto virado ao artista
    // e arrasta de lado, e o CENTRO segue o cursor (o gancho puxa o barro atrás
    // dele).
    let n_rumo = norma(rumo).max(1e-9);
    let u = [rumo[0] / n_rumo, rumo[1] / n_rumo, rumo[2] / n_rumo];
    let passo = len / dabs as f32;
    let mut centro = [0.0f32, 0.0, 1.0];
    for _ in 0..dabs {
        centro = [
            centro[0] + u[0] * passo,
            centro[1] + u[1] * passo,
            centro[2] + u[2] * passo,
        ];
        let (cut, done, _) = crate::dyntopo::passe_nos_motores(
            &mut malha,
            brush.verb,
            alvo,
            centro,
            brush.radius,
            crate::dyntopo::Rascunho {
                remap: &mut remap,
                births: &mut births,
                region: &mut region,
            },
            None,
        );
        if cut {
            stroke.shrink_with(&remap);
        }
        if done {
            stroke.grow_with(&malha, &births);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::hooking(
                centro,
                raio,
                OLHO,
                [u[0] * passo, u[1] * passo, u[2] * passo],
            ),
            Symmetry::default(),
        );
    }
    let mut l = mede(&malha, &repouso, unidade, com_rasgo);
    l.verts0 = repouso.len();
    l
}

fn aresta_media(m: &Mesh) -> f32 {
    let pos = m.positions();
    let viz = &m.adjacency().vert_verts;
    let (mut soma, mut n) = (0.0f64, 0usize);
    for u in 0..pos.len() {
        for &v in viz.neighbours(u) {
            let (a, b) = (pos[u], pos[v as usize]);
            soma += f64::from(norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]));
            n += 1;
        }
    }
    if n == 0 {
        1.0
    } else {
        (soma / n as f64) as f32
    }
}

/// ⚠️ **O deslocamento mede-se contra o REPOUSO por POSIÇÃO e não por índice:**
/// o passe renumera, logo o vértice `i` do fim não é o vértice `i` do princípio.
/// A régua é a distância ao ponto mais próximo da malha de repouso, que é a
/// mesma que o §31 desta linha já usa.
fn mede(m: &Mesh, repouso: &[[f32; 3]], unidade: f32, com_rasgo: bool) -> Leitura {
    let pos = m.positions();
    let desl: Vec<f32> = if com_rasgo {
        pos.iter().map(|p| perto(*p, repouso)).collect()
    } else {
        vec![0.0; pos.len()]
    };
    let puxao = desl.iter().cloned().fold(0.0f32, f32::max);

    let viz = &m.adjacency().vert_verts;
    let (mut rasgo, mut estica) = (0.0f32, 0.0f32);
    for u in 0..pos.len() {
        for &v in viz.neighbours(u) {
            let vi = v as usize;
            rasgo = rasgo.max((desl[u] - desl[vi]).abs());
            let (a, b) = (pos[u], pos[vi]);
            estica = estica.max(norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]));
        }
    }

    let nrm = m.normals();
    let mut avesso = 0usize;
    let mut lasca = 180.0f32;
    for f in m.faces() {
        let vs = f.verts();
        if vs.len() < 3 {
            continue;
        }
        lasca = lasca.min(menor_angulo(
            pos[vs[0] as usize],
            pos[vs[1] as usize],
            pos[vs[2] as usize],
        ));
        let (mut acc, mut k) = ([0.0f32; 3], 0.0f32);
        for &v in vs {
            for &w in viz.neighbours(v as usize) {
                let n = nrm[w as usize];
                acc = [acc[0] + n[0], acc[1] + n[1], acc[2] + n[2]];
                k += 1.0;
            }
        }
        if k == 0.0 {
            continue;
        }
        let media = [acc[0] / k, acc[1] / k, acc[2] / k];
        let lm = norma(media);
        if lm < 1e-6 {
            continue;
        }
        let fa = nrm[vs[0] as usize];
        if (fa[0] * media[0] + fa[1] * media[1] + fa[2] * media[2]) / lm < -0.2 {
            avesso += 1;
        }
    }

    Leitura {
        verts: pos.len(),
        ponta: pos.iter().map(|p| norma(*p)).fold(0.0f32, f32::max),
        verts0: 0,
        puxao,
        rasgo: rasgo / unidade,
        estica: estica / unidade,
        avesso,
        lasca,
    }
}

fn perto(p: [f32; 3], nuvem: &[[f32; 3]]) -> f32 {
    let mut melhor = f32::INFINITY;
    for q in nuvem {
        let d = norma([p[0] - q[0], p[1] - q[1], p[2] - q[2]]);
        if d < melhor {
            melhor = d;
        }
    }
    melhor
}

fn menor_angulo(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let l = [
        norma([b[0] - c[0], b[1] - c[1], b[2] - c[2]]),
        norma([a[0] - c[0], a[1] - c[1], a[2] - c[2]]),
        norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]),
    ];
    let mut pior = 180.0f32;
    for i in 0..3 {
        let (x, y, z) = (l[i], l[(i + 1) % 3], l[(i + 2) % 3]);
        if y < 1e-12 || z < 1e-12 {
            return 0.0;
        }
        let cos = ((y * y + z * z - x * x) / (2.0 * y * z)).clamp(-1.0, 1.0);
        pior = pior.min(cos.acos().to_degrees());
    }
    pior
}

/// ⭐⭐⭐⭐ **A TABELA DO REPORT** — o mesmo gancho, com e sem `Connected Only`,
/// com o passe de topologia a correr nos dois.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
///   diag_o_gancho_com_topologia -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do report, nao afirma nada"]
fn diag_o_gancho_com_topologia() {
    println!("\n== O GANCHO × CONNECTED ONLY, COM TOPOLOGIA DINAMICA ==");
    println!("   esfera de escultura triangulada · olho em -z · arrasto no +x\n");
    println!(
        "{:>6} {:>6} {:>5} | {:>8} {:>8} {:>8} {:>8} {:>7} {:>7}",
        "raio", "len", "mask", "verts", "puxao", "rasgo", "estica", "avesso", "lasca"
    );
    println!("{}", "-".repeat(84));
    let alvo = 0.03;
    for raio in [0.25f32, 0.35] {
        for len in [0.3f32, 0.6, 0.9] {
            for mascara in [false, true] {
                let l = gancho(raio, len, 12, mascara, alvo);
                println!(
                    "{raio:>6.2} {len:>6.2} {:>5} | {:>8} {:>8.4} {:>8.2} {:>8.2} {:>7} {:>7.2}",
                    if mascara { "ON" } else { "off" },
                    l.verts,
                    l.puxao,
                    l.rasgo,
                    l.estica,
                    l.avesso,
                    l.lasca
                );
            }
        }
    }
    println!(
        "\n   rasgo/estica em unidades da ARESTA MEDIA de repouso · lasca = menor\n   \
         angulo de triangulo, em graus. A foto lê-se em `avesso` (o preto) e em\n   \
         `verts` (o refino explodido)."
    );
}

/// ⭐⭐⭐⭐ **DE QUEM É O CORTE** — a célula pior da tabela irmã, uma corrida só.
///
/// ⚠️ **Ela existe separada da irmã porque o RELÓGIO é de atribuição:** a tabela
/// acima custa `357 s` (a régua do rasgo é `O(n²)` sobre `~97 k` vértices), e uma
/// atribuição corre-se **quatro vezes** — uma por condição da máscara. Aqui as
/// colunas são as que não pedem a malha de repouso, e são exactamente as que a
/// foto mostra: o **preto** (`avesso`), a **lasca** e o **refino** (`verts`).
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
///   diag_de_quem_e_o_corte -- --ignored --nocapture --test-threads=1
/// ```
///
/// O protocolo da atribuição é o da casa — *backup → mutar a constante →
/// correr → restaurar* —, com uma condição da [`ph2d_sculpt3d::dab_alcance`]
/// neutralizada de cada vez (`ALCANCE_TECTO`, `RAZAO_MAXIMA`, `NORMAL_LIMIAR`).
#[test]
#[ignore = "sonda: a celula da atribuicao, nao afirma nada"]
fn diag_de_quem_e_o_corte() {
    println!("\n== A CELULA DA ATRIBUICAO (raio 0,25 · len 0,90) ==");
    println!(
        "{:>5} | {:>8} {:>8} {:>7} {:>7}",
        "mask", "verts", "estica", "avesso", "lasca"
    );
    println!("{}", "-".repeat(42));
    for mascara in [false, true] {
        let l = gancho_com(0.25, 0.9, 12, mascara, 0.03, false);
        println!(
            "{:>5} | {:>8} {:>8.2} {:>7} {:>7.2}",
            if mascara { "ON" } else { "off" },
            l.verts,
            l.estica,
            l.avesso,
            l.lasca
        );
    }
}

/// ⭐⭐⭐⭐ **O GATE DO REPORT: com `Connected Only` ligado, o gancho deixa a peça
/// como a deixa sem ele.**
///
/// A cura vive na [`ph2d_sculpt3d::dab_alcance::MemoriaDoTraco`] — *a máscara
/// decide quem ENTRA no traço; ela nunca decide quem SAI* —, e este gate é a
/// reprodução da foto pelo caminho do produto.
///
/// ⛔⛔ **As três metades de vacuidade, e cada uma já foi um defeito nesta casa:**
///
/// 1. **o verbo OFERECE a máscara** — se alguém a retirar ao gancho, o gate
///    passaria a comparar duas corridas iguais e ficaria verde a medir nada;
/// 2. **o gancho PUXOU** (`ponta`) — sem o puxão não há tubo, e sem tubo não há
///    face de trás que se vire;
/// 3. **o passe REFINOU** (`verts > verts0`) — o refino é a segunda metade da
///    foto, e sem ele a fixtura não contém o fenómeno.
#[test]
fn o_gancho_com_a_mascara_nao_rasga_as_costas() {
    let brush = Brush {
        verb: Verb::SnakeHook,
        surface_only: true,
        ..Brush::default()
    };
    assert!(
        brush.offers_surface_only(),
        "o gancho deixou de oferecer o `Connected Only` — este gate passaria a \
         comparar duas corridas identicas e ficaria verde a medir nada"
    );

    let off = gancho_com(0.25, 0.9, 12, false, 0.03, false);
    let on = gancho_com(0.25, 0.9, 12, true, 0.03, false);

    assert!(
        on.ponta > 1.15 && off.ponta > 1.15,
        "o gancho nao puxou (ponta {:.3} / {:.3} contra a esfera de repouso em \
         1,0): sem tubo nao ha' face de tras para se virar",
        on.ponta,
        off.ponta
    );
    // ⚠️ **O passe CORREU — e correr não quer dizer crescer.** Com o alvo de
    // aresta a `0,03` sobre a esfera de escultura ele COLAPSA no total
    // (`98 306 → 97 636`), e a 1.ª redacção deste gate exigia `verts > verts0`
    // e reprovou sobre produto correcto. *O que a fixtura tem de conter é o
    // passe a mexer na topologia, não um sinal escolhido para ele.*
    assert!(
        off.verts != off.verts0 && on.verts != on.verts0,
        "o passe de topologia nao mexeu na malha ({} -> {} / {})",
        on.verts0,
        on.verts,
        off.verts
    );
    // ⭐ **E o REFINO EXPLODIDO da foto é esta linha:** antes da cura a máscara
    // levava a peça a `99 699` contra os `97 636` do controlo (`+2,1 %`), porque
    // o passe parte a aresta longa que o rasgo abriu.
    assert!(
        on.verts as f64 <= off.verts as f64 * 1.005,
        "ligar a mascara inflou a malha de {} para {} vertices — e' o refino \
         explodido da foto",
        off.verts,
        on.verts
    );

    // ⭐ **A coluna que a foto mostra:** faces a apontar contra a vizinhança — o
    // preto. Medido antes da cura: `0` sem a máscara e **`296`** com ela.
    assert_eq!(
        on.avesso, 0,
        "a mascara deixou {} faces do avesso (o controlo sem ela deixa {}): as \
         costas do traco rasgaram, que e' o report de 19/09",
        on.avesso, off.avesso
    );

    // ⚠️ **E as duas colunas de forma, contra o CONTROLO e não contra um número
    // escolhido** — o que se afirma é *«ligar a máscara não piora»*, e a barra é
    // a corrida sem ela. Medido antes da cura: `estica` `3,44 → 8,10` e a lasca
    // `1,88° → 0,33°`.
    assert!(
        on.estica <= off.estica * 1.05,
        "a aresta mais longa passou de {:.2} para {:.2} ao ligar a mascara — e' \
         dela que o refino explodido da foto nasce",
        off.estica,
        on.estica
    );
    assert!(
        on.lasca >= off.lasca * 0.95,
        "o pior triangulo passou de {:.2}° para {:.2}° ao ligar a mascara",
        off.lasca,
        on.lasca
    );
}

/// ⛔⛔⛔⛔ **QUANDO É QUE AINDA RASGA** — a varredura que o report *«algumas
/// vezes correto, algumas vezes bugado»* (2026-09-19, 2.ª foto: uma fita escura
/// e esfarelada ao longo da ARESTA DE CIMA do chifre) obriga a fazer.
///
/// ⚠️ *Um defeito intermitente é um defeito cujo regime ninguém varreu.* O que
/// muda entre duas pinceladas do dono é o **VERBO**, o **RUMO** do arrasto e a
/// **TAXA** de eventos — e nenhuma das três estava na tabela.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
///   diag_quando_o_gancho_ainda_rasga -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore = "sonda: varre o regime, nao afirma nada"]
fn diag_quando_o_gancho_ainda_rasga() {
    println!("\n== QUANDO E' QUE AINDA RASGA (avesso: off -> ON) ==");
    println!("   esfera de escultura · olho em -z · len 0,90 · raio 0,25\n");
    println!(
        "{:>12} {:>14} {:>6} | {:>9} {:>9} | {:>9} {:>9}",
        "verbo", "rumo", "dabs", "avesso", "avesso", "estica", "estica"
    );
    println!(
        "{:>12} {:>14} {:>6} | {:>9} {:>9} | {:>9} {:>9}",
        "", "", "", "off", "ON", "off", "ON"
    );
    println!("{}", "-".repeat(80));
    let rumos: [(&str, [f32; 3]); 4] = [
        ("tangencial", [1.0, 0.0, 0.0]),
        ("45° p/ olho", [1.0, 0.0, 1.0]),
        ("p/ o olho", [0.0, 0.0, 1.0]),
        ("obliquo", [1.0, 0.6, 0.35]),
    ];
    for verbo in [Verb::SnakeHook, Verb::Move, Verb::Thumb, Verb::Nudge] {
        for (nome, rumo) in rumos {
            for dabs in [6usize, 12, 24] {
                let f = |mascara| {
                    traco_com(Gesto {
                        verbo,
                        raio: 0.25,
                        len: 0.9,
                        dabs,
                        rumo,
                        mascara,
                        alvo: 0.03,
                        com_rasgo: false,
                    })
                };
                let (off, on) = (f(false), f(true));
                let marca = if on.avesso > off.avesso { " ⛔" } else { "" };
                println!(
                    "{:>12} {nome:>14} {dabs:>6} | {:>9} {:>9} | {:>9.2} {:>9.2}{marca}",
                    format!("{verbo:?}"),
                    off.avesso,
                    on.avesso,
                    off.estica,
                    on.estica
                );
            }
        }
    }
    println!("\n   ⛔ marca as celulas em que LIGAR a mascara piora.");
}

/// ⛔⛔⛔ **O PUXÃO LONGO** — a 2.ª foto do dono mostra um chifre **muito mais
/// comprido** que a bossa da 1.ª, com uma fita escura ao longo da aresta de
/// cima. Esta varredura mantém o PASSO constante (que é o que o `walk` do
/// produto garante) e alonga o gesto.
#[test]
#[ignore = "sonda: varre o comprimento, nao afirma nada"]
fn diag_o_puxao_longo() {
    println!("\n== O PUXAO LONGO (passo constante, como o walk do produto) ==");
    println!(
        "{:>7} {:>7} {:>6} | {:>9} {:>9} | {:>8} {:>8} | {:>8} {:>8}",
        "len",
        "passo",
        "dabs",
        "avesso off",
        "avesso ON",
        "lasca of",
        "lasca ON",
        "est off",
        "est ON"
    );
    println!("{}", "-".repeat(92));
    for passo in [0.075f32, 0.0375] {
        for len in [0.9f32, 1.8, 2.7] {
            let dabs = (len / passo).round() as usize;
            let f = |mascara| {
                traco_com(Gesto {
                    verbo: Verb::SnakeHook,
                    raio: 0.25,
                    len,
                    dabs,
                    rumo: [1.0, 0.0, 0.0],
                    mascara,
                    alvo: 0.03,
                    com_rasgo: false,
                })
            };
            let (off, on) = (f(false), f(true));
            println!(
                "{len:>7.2} {passo:>7.4} {dabs:>6} | {:>9} {:>9} | {:>8.2} {:>8.2} | {:>8.2} {:>8.2}",
                off.avesso, on.avesso, off.lasca, on.lasca, off.estica, on.estica
            );
        }
    }
}
