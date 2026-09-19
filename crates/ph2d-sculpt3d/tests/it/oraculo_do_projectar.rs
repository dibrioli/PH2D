//! **A BANCADA DO PINCEL DE PROJECTAR NA CENA** — `SPEC_unblocked_brushes.md` §6,
//! corrida contra o corpus de `24` fixturas do oráculo.
//!
//! # ⭐⭐⭐ Ela pôde existir, e o esfregão não — a diferença está MEDIDA
//!
//! As fixturas desta família **não trazem o objecto-ALVO**: nem o cabeçalho nem
//! o `README` da pasta dizem onde ele está. Era a mesma forma de bloqueio que
//! parou a bancada do esfregão (lá faltava a conectividade), e a resposta honesta
//! era medir antes de declarar. Medido, **a cena é recuperável com resíduo
//! ZERO**:
//!
//! | o que | como se recupera | resíduo |
//! |---|---|---|
//! | a malha de entrada | grelha `41×41` sobre `[−1,1]²` em `z = 0`, **row-major** | `0,000000` contra a grelha nominal |
//! | a direcção da vista | o deslocamento é inteiramente em `z` | — |
//! | o plano-alvo | a fixtura de **curva constante e força 1** move o miolo o vão INTEIRO, e os `146` vértices pousam **todos** em `z = −0,500000` | `0` de dispersão |
//!
//! ⭐⭐ **E a recuperação é CONFIRMADA por uma segunda fixtura que não a
//! produziu:** `projectar_forca05_constante_1passo` pousa em `z = −0,125000`, que
//! é `0,5 × 0,5²` — *a lei da força ao quadrado (§1.1) e a posição do alvo a
//! confirmarem-se uma à outra, ao último dígito impresso.*
//!
//! ⚠️⚠️ **E as `24` partilham a MESMA malha de entrada** (o bloco `r` tem o
//! mesmo `sha256` nas `24`), logo a calibração **transfere por construção** — não
//! por suposição. ⛔ *Sem esse facto isto seria «adivinhar a permutação», que é
//! exactamente o que a bancada do esfregão recusou fazer.*
//!
//! # ⛔ As SEIS que ficam de fora, e o que falta a cada uma
//!
//! | fixtura | o que o cabeçalho não grava |
//! |---|---|
//! | `…_alvo_esfera`, `…_alvo_esfera_grossa`, `…_alvo_esfera_grossa_subsurf2` | a **tesselação** do alvo — e ela **É** a medição (`0,4588` · `0,4804` · `0,4990`, §6.1) |
//! | `…_alvo_inclinado_escalado` | a pose do alvo — e a nossa [`ph2d_mesh::Pose`] **não tem rotação** para a exprimir |
//! | `…_normal_plano_x` | a orientação que faz a normal da área apontar `+X` numa peça que é o plano `z = 0` |
//! | `…_simetria_x` | a altura do alvo não é separável do efeito da simetria (`−0,35` contra os `−0,5` das irmãs) |
//!
//! ⏳ **Dívida NOMEADA, e é acto do E:** uma emenda que emita um bloco com a
//! geometria e a pose de cada alvo. Com ele estas seis entram sem uma linha de
//! bancada nova.

use std::path::PathBuf;

use ph2d_mesh::{Face, Mesh, Pose};
use ph2d_sculpt3d::{Brush, Dab, Falloff, RefMode, SculptStroke, Symmetry, Verb};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/3D/cleanroom/fixtures/unblocked/projectar")
}

/// **DESCOMPRIME um `.gz`** — a gémea da do [`super::oraculo_do_esfregao`].
fn inflar(caminho: &std::path::Path) -> String {
    let raw = std::fs::read(caminho).unwrap_or_else(|e| panic!("{caminho:?}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{caminho:?}: nao e' gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        off += 2 + (usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8));
    }
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    let bytes = miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8])
        .unwrap_or_else(|e| panic!("{caminho:?}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

/// Uma fixtura lida: o cabeçalho e os três blocos que esta família traz.
struct Fix {
    cab: std::collections::BTreeMap<String, String>,
    r: Vec<[f32; 3]>,
    s: Vec<[f32; 3]>,
    c: Vec<[f32; 3]>,
}

impl Fix {
    fn ler(nome: &str) -> Self {
        let texto = inflar(&fixture_dir().join(format!("{nome}.txt.gz")));
        let (mut cab, mut r, mut s, mut c) =
            (std::collections::BTreeMap::new(), vec![], vec![], vec![]);
        for l in texto.lines() {
            if let Some(resto) = l.strip_prefix("# ") {
                if let Some((k, v)) = resto.split_once(':') {
                    cab.insert(k.trim().to_owned(), v.trim().to_owned());
                }
                continue;
            }
            let mut it = l.split_whitespace();
            let Some(tag) = it.next() else { continue };
            let p: Vec<f32> = it.filter_map(|x| x.parse().ok()).collect();
            if p.len() < 3 {
                continue;
            }
            let v = [p[0], p[1], p[2]];
            match tag {
                "r" => r.push(v),
                "s" => s.push(v),
                "c" => c.push(v),
                _ => {}
            }
        }
        Self { cab, r, s, c }
    }

    fn txt(&self, k: &str) -> &str {
        self.cab
            .get(k)
            .unwrap_or_else(|| panic!("o cabeçalho não tem `{k}`"))
    }

    fn num(&self, k: &str) -> f32 {
        self.txt(k)
            .parse()
            .unwrap_or_else(|_| panic!("`{k}` não é número"))
    }

    fn bool(&self, k: &str) -> bool {
        self.txt(k) == "True"
    }
}

/// O lado da grelha de entrada — **contado**, não escolhido: `41² = 1681`, que é
/// o `vertices` que as `24` declaram.
const LADO: usize = 41;

/// ⭐⭐ **A MALHA DE ENTRADA, reconstruída** — e o gate
/// [`a_malha_de_entrada_e_a_grelha_que_o_corpus_declara`] é quem prova que ela é
/// esta e não outra.
///
/// ⚠️ **A DIAGONAL de cada quad é uma escolha, e ela é INOBSERVÁVEL nesta
/// família** — há gate a medi-lo. Num plano chato a normal da área é `+Z` com
/// qualquer das duas, a pegada sai de posições, e a máscara de alcance liga os
/// mesmos vértices. *Uma escolha que muda o resultado seria um palpite; uma que
/// não muda é uma nota.*
fn grelha(diagonal_invertida: bool) -> Mesh {
    let mut pos = Vec::with_capacity(LADO * LADO);
    for j in 0..LADO {
        for i in 0..LADO {
            let f = |k: usize| -1.0 + (k as f32) * (2.0 / (LADO as f32 - 1.0));
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let mut faces = Vec::with_capacity((LADO - 1) * (LADO - 1) * 2);
    for j in 0..LADO - 1 {
        for i in 0..LADO - 1 {
            let idx = |a: usize, b: usize| u32::try_from(b * LADO + a).expect("cabe");
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            if diagonal_invertida {
                faces.push(Face::tri(a, b, d));
                faces.push(Face::tri(b, c, d));
            } else {
                faces.push(Face::tri(a, b, c));
                faces.push(Face::tri(a, c, d));
            }
        }
    }
    Mesh::from_parts(pos, faces).expect("a grelha do corpus")
}

/// **O PLANO-ALVO, em `z = alt`** — grande o bastante para que todo raio da
/// pegada o encontre.
fn alvo(alt: f32) -> (Mesh, Pose) {
    let l = 4.0;
    let m = Mesh::from_parts(
        vec![[-l, -l, alt], [l, -l, alt], [l, l, alt], [-l, l, alt]],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("o plano-alvo");
    (m, Pose::IDENTITY)
}

/// A vista do corpus — medida: o deslocamento é inteiramente em `z`.
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

fn pincel(f: &Fix) -> Brush {
    Brush {
        verb: Verb::SceneProject,
        // ⚠️ **O modo é o `B`**: este verbo é da referência restrita, e o `S`
        // não o declara — ele nem tem o conceito de outra peça da cena.
        mode: RefMode::B,
        radius: f.num("raio_objeto"),
        strength: f.num("forca"),
        falloff: match f.txt("curva") {
            "CONSTANT" => Falloff::Constant,
            "SMOOTH" => Falloff::Smooth,
            c => panic!("curva {c} nao exercitada por esta bancada"),
        },
        hardness: f.num("dureza"),
        project_mode: match f.txt("direccao_do_raio") {
            "VIEW_NORMAL" => ph2d_sculpt3d::ProjectMode::View,
            "PLANE_NORMAL" => ph2d_sculpt3d::ProjectMode::Plane,
            d => panic!("direccao {d} nao mapeada"),
        },
        project_min_distance: f.num("distancia_minima"),
        project_bidirectional: f.bool("nos_dois_sentidos"),
        invert: f.txt("sentido") == "SUBTRACT",
        accumulate: f.bool("acumula"),
        ..Brush::default()
    }
}

/// **O TRAÇO, conduzido como o oráculo foi conduzido:** um dab por ponto do
/// cursor, carimbando na superfície debaixo dele.
///
/// ⚠️ **O cursor vem em `z = 0` e a peça começa em `z = 0`**, logo o ponto do
/// cursor **é** o ponto da superfície no primeiro dab. Nos seguintes o barro já
/// desceu — e é essa a re-medição da §6.5 que a lei faz de propósito.
fn correr(f: &Fix, alvos: Vec<(Mesh, Pose)>, diagonal: bool) -> Mesh {
    let brush = pincel(f);
    correr_com(f, &brush, alvos, diagonal)
}

/// A irmã da [`correr`] com o pincel dado — ela existe porque a inversão de
/// `…_invertido` veio pelo GESTO e não pelo cabeçalho.
fn correr_com(f: &Fix, brush: &Brush, alvos: Vec<(Mesh, Pose)>, diagonal: bool) -> Mesh {
    let mut mesh = grelha(diagonal);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.pecas_da_cena = alvos;
    stroke.pose_activa = Pose::IDENTITY;
    for &c in &f.c {
        stroke.dab(
            &mut mesh,
            brush,
            &Dab::at(c, brush.radius, OLHO),
            Symmetry::default(),
        );
    }
    mesh
}

/// O que uma comparação com o oráculo devolve — ver [`comparar`].
struct Veredito {
    /// O pior desvio entre os vértices que **os DOIS** moveram.
    desvio: f32,
    /// Quantos um moveu e o outro não.
    discordam: usize,
    /// Quantos vértices estão na **banda de empate** — ver [`comparar`].
    empates: usize,
    nossos: usize,
    deles: usize,
}

/// A largura da banda de empate, em unidades do objecto.
///
/// ⚠️ **Ela não é escolhida: é a resolução do `f32` nestas coordenadas.** Medido
/// no calibrador, três vértices da grelha caem a `6e-9` do raio — `0,349999994`
/// contra `0,35` —, e é o último bit de um `f32` perto de `0,35` que decide se
/// eles entram na pegada.
const BANDA_DE_EMPATE: f32 = 1e-7;

/// ⭐⭐⭐ **COMPARAR COM O ORÁCULO, e a régua tem de ter DUAS metades porque a
/// curva CONSTANTE amplifica um empate de último bit no VÃO INTEIRO.**
///
/// ⛔⛔ **Este foi o primeiro achado desta bancada, e ele muda o desenho dela:**
/// com a curva *Constant* o peso é `1` dentro do raio e `0` fora, sem
/// transição — logo um vértice que esteja a `6e-9` da borda ou anda `0,5` ou não
/// anda nada. Medido: o oráculo move `146`, nós movemos `147`, e o vértice extra
/// está a `6e-9` **dentro** do raio. *Uma barra de posição leria `5,0e-1` sobre
/// uma lei que está certa ao sétimo decimal em todos os outros 146.*
///
/// ⇒ a comparação é:
///
/// | metade | o que ela afirma |
/// |---|---|
/// | **a LEI** | nos vértices que os dois moveram, o pior desvio |
/// | **a BORDA** | quem discorda tem de estar na banda de empate do raio |
///
/// ⚠️ **A segunda metade é um TECTO e não um «≥ 0»:** ela conta os vértices na
/// banda a partir da GEOMETRIA (a distância aos cursores contra o raio), logo
/// uma divergência de lei que mova o vértice errado **não cabe** nela.
///
/// ⛔⛔ **CADA LADO É MEDIDO CONTRA O PRÓPRIO REPOUSO, e a primeira redacção
/// desta função não o fazia** — ela perguntava *«a nossa saída difere do
/// repouso DELES?»*. A reconstrução da grelha bate a do corpus a `<1e-6`, que é
/// **exacto para a geometria e enorme para um teste de igualdade**: `1069` dos
/// `1681` vértices liam-se como movidos por nós sem se terem mexido um
/// nanómetro. *Uma régua que compara duas coisas certas de lados diferentes
/// mede a diferença entre as réguas.*
fn comparar(nosso: &Mesh, entrada: &Mesh, f: &Fix) -> Veredito {
    assert_eq!(nosso.positions().len(), f.s.len(), "contagens diferentes");
    let mexeu = |a: &[f32; 3], b: &[f32; 3]| {
        (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max) > 1e-9
    };
    let raio = f.num("raio_objeto");
    let (mut desvio, mut discordam, mut empates) = (0.0f32, 0usize, 0usize);
    let (mut nossos, mut deles) = (0usize, 0usize);
    for (i, (((n, o), r), e)) in nosso
        .positions()
        .iter()
        .zip(&f.s)
        .zip(&f.r)
        .zip(entrada.positions())
        .enumerate()
    {
        let (a, b) = (mexeu(n, e), mexeu(o, r));
        nossos += usize::from(a);
        deles += usize::from(b);
        // A banda: a menor distância deste vértice a um cursor, contra o raio.
        let perto =
            f.c.iter()
                .map(|c| {
                    let d = [r[0] - c[0], r[1] - c[1], r[2] - c[2]];
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
                })
                .fold(f32::INFINITY, f32::min);
        if (perto - raio).abs() < BANDA_DE_EMPATE {
            empates += 1;
        }
        if a != b {
            discordam += 1;
            let _ = i;
        } else if a {
            desvio = desvio.max((0..3).map(|k| (n[k] - o[k]).abs()).fold(0.0f32, f32::max));
        }
    }
    Veredito {
        desvio,
        discordam,
        empates,
        nossos,
        deles,
    }
}

/// ⭐⭐⭐ **A MALHA DE ENTRADA É A GRELHA QUE O CORPUS DECLARA** — o gate que
/// torna esta bancada honesta em vez de um palpite.
///
/// ⛔⛔ **Sem ele, toda a bancada seria «adivinhar a permutação»** — a forma
/// exacta que a bancada do esfregão recusou, e ali com razão (`528` colisões).
/// Aqui a reconstrução é **exacta**, e é isto que o afirma.
#[test]
fn a_malha_de_entrada_e_a_grelha_que_o_corpus_declara() {
    let m = grelha(false);
    let mut vistas = 0usize;
    for e in std::fs::read_dir(fixture_dir()).expect("a pasta") {
        let caminho = e.expect("entrada").path();
        if caminho.extension().is_none_or(|x| x != "gz") {
            continue;
        }
        vistas += 1;
        let nome = caminho.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let f = Fix::ler(nome.trim_end_matches(".txt"));
        assert_eq!(
            f.r.len(),
            m.positions().len(),
            "{nome}: contagem de vértices diferente da grelha"
        );
        let pior =
            f.r.iter()
                .zip(m.positions())
                .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max))
                .fold(0.0f32, f32::max);
        assert!(
            pior < 1e-6,
            "{nome}: a malha de entrada não é a grelha nominal (pior {pior:.3e}) — \
             a calibração desta bancada deixou de transferir"
        );
    }
    assert_eq!(
        vistas, 24,
        "o piso de população: a espec §7 conta 24 fixturas de `projectar`"
    );
}

/// ⭐⭐ **A DIAGONAL DA GRELHA É INOBSERVÁVEL** — a nota que torna a
/// reconstrução completa.
///
/// ⚠️ A triangulação de um quad é a única coisa que o corpus não determina, e
/// este gate mede que ela **não muda uma saída**. *Uma escolha que muda o
/// resultado seria um palpite a fingir de medição.*
#[test]
fn a_diagonal_da_grelha_nao_muda_uma_saida() {
    let f = Fix::ler("projectar_base");
    let a = correr(&f, vec![alvo(-0.5)], false);
    let b = correr(&f, vec![alvo(-0.5)], true);
    let pior = a
        .positions()
        .iter()
        .zip(b.positions())
        .map(|(x, y)| (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0f32, f32::max))
        .fold(0.0f32, f32::max);
    assert!(
        pior < 1e-6,
        "a diagonal da grelha mudou a saída em {pior:.3e} — a reconstrução \
         passou a depender de uma escolha que o corpus não grava"
    );
}

/// ⭐⭐⭐ **O CALIBRADOR — e ele é o gate mais forte desta bancada.**
///
/// Curva **constante**, força `1`, **um** dab: o peso vale `1` no miolo e a lei
/// fica **sozinha no numerador** (espec §8.4). A resposta certa é *o vão
/// inteiro*, e o miolo tem de pousar **no plano**.
///
/// ⭐ **A segunda célula confirma a força ao quadrado (§1.1) com a MESMA cena:**
/// a metade da força leva o vértice a `0,5²` do caminho — `0,125` de `0,5` —, e
/// isso é medido, não inferido. *Duas fixturas a confirmarem uma à outra a
/// posição do alvo e a lei da força.*
#[test]
fn o_calibrador_pousa_no_plano_e_a_forca_entra_ao_quadrado() {
    for (nome, esperado) in [
        ("projectar_constante_1passo", -0.5f32),
        ("projectar_forca05_constante_1passo", -0.125),
    ] {
        let f = Fix::ler(nome);
        let entrada = grelha(false);
        let nosso = correr(&f, vec![alvo(-0.5)], false);
        let v = comparar(&nosso, &entrada, &f);
        assert!(
            v.deles > 100,
            "{nome}: a fixtura deixou de conter o fenómeno — o oráculo moveu {}",
            v.deles
        );
        assert!(
            v.desvio < 2e-6,
            "{nome}: desvio {:.3e} no miolo (o vão é `0,5`, o esperado é pousar \
             em {esperado}) — {} nossos contra {} deles",
            v.desvio,
            v.nossos,
            v.deles
        );
        assert!(
            v.discordam <= v.empates,
            "{nome}: {} vértices discordam e só {} estão na banda de empate do \
             raio — isso é lei, não borda",
            v.discordam,
            v.empates
        );
    }
}

/// **SONDA** — quantos vértices se mexem, e onde a cadeia morre.
#[test]
#[ignore]
fn diag_o_calibrador() {
    let f = Fix::ler("projectar_constante_1passo");
    let b = pincel(&f);
    eprintln!(
        "pincel: raio {} forca {} dureza {} curva {:?} surface_only {} front_faces {}",
        b.radius, b.strength, b.hardness, b.falloff, b.surface_only, b.front_faces_only
    );
    eprintln!("cursores: {:?}", f.c);
    let alvos = vec![alvo(-0.5)];
    let d = ph2d_sculpt3d::distancia_de_projeccao_para_teste(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        Pose::IDENTITY,
        &alvos,
        false,
        0.0,
    );
    eprintln!("distancia crua no centro: {d:?}");
    let nosso = correr(&f, alvos, false);
    let movidos = nosso
        .positions()
        .iter()
        .zip(&f.r)
        .filter(|(a, b)| (a[2] - b[2]).abs() > 1e-9)
        .count();
    let deles =
        f.s.iter()
            .zip(&f.r)
            .filter(|(a, b)| (a[2] - b[2]).abs() > 1e-9)
            .count();
    eprintln!("movidos: nosso {movidos}, oraculo {deles}");
}

/// A cena de cada fixtura reconstrutível: as alturas dos planos-alvo, e se a
/// inversão veio pelo GESTO (que não viaja no cabeçalho — ver o `README` da
/// pasta).
///
/// ⚠️ **As alturas são MEDIDAS, não escolhidas:** cada uma sai do deslocamento
/// máximo da própria fixtura, e o calibrador prova que a leitura é exacta.
///
/// ⚠️ **Onde há DOIS alvos, o segundo é o que a espec §6.3 torna
/// INOBSERVÁVEL** — ganha o de menor `|d|`, logo a posição do perdedor não entra
/// na resposta. O gate [`o_alvo_perdedor_e_inobservavel`] mede-o em vez de o
/// supor.
const CENAS: [(&str, &[f32]); 14] = [
    ("projectar_base", &[-0.5]),
    ("projectar_base_repete", &[-0.5]),
    ("projectar_constante_1passo", &[-0.5]),
    ("projectar_forca05_constante_1passo", &[-0.5]),
    ("projectar_forca05", &[-0.5]),
    ("projectar_dureza05", &[-0.5]),
    ("projectar_normal_plano_area", &[-0.5]),
    ("projectar_mindist01", &[-0.5]),
    ("projectar_mindist06", &[-0.5]),
    ("projectar_acima_bidir", &[0.5]),
    ("projectar_acima_sem_bidir", &[0.5]),
    ("projectar_mindist01_acima_bidir", &[0.5]),
    ("projectar_dois_abaixo", &[-0.3, -0.9]),
    ("projectar_dois_lados_bidir", &[0.3, -0.5]),
];

/// ⛔⛔⛔ **O QUE O ORÁCULO FAZ E NÓS DELIBERADAMENTE NÃO FAZEMOS** —
/// divergência DECLARADA, por ordem do dono (15/09), depois do smoke da `=45`:
/// *«não vi utilidade na feature Scene Project + CTRL. Melhor retirá-la e
/// documentá-la como indesejada.»*
///
/// ⚠️ **As duas estavam VERDES quando saíram** (`2,384e-7` cada, contra a
/// [`BARRA`] de `2e-6`): elas não saíram por não as conseguirmos reproduzir,
/// saíram porque **o produto deixou de ter a capacidade**. *Uma fixtura que sai
/// por DECISÃO e uma que sai por DERROTA leem-se igual numa lista — o que as
/// separa é esta frase e o número ao lado dela.*
///
/// ⭐ **Duas portas para a mesma inversão**, e é por isso que são duas fixturas:
/// numa ela vinha pelo GESTO (`Ctrl` durante o traço, logo **não** aparece no
/// cabeçalho) e na outra como PROPRIEDADE (`sentido: SUBTRACT`). As duas mediam
/// `0,0` de diferença entre si — e no produto nenhuma tem porta, porque o único
/// escritor do `Brush::invert` é o pen-down e o painel não o oferece.
///
/// Os ficheiros **ficam na pasta**: são saída de oráculo e voltam a valer no dia
/// em que a decisão mudar — e quem a mudar tem o gate abaixo a dizer-lho.
const FORA_POR_DECISAO_DO_DONO: [(&str, &str); 2] = [
    (
        "projectar_invertido",
        "a inversao vinha pelo GESTO (Ctrl durante o traco)",
    ),
    (
        "projectar_subtrair",
        "a MESMA inversao como PROPRIEDADE (`sentido: SUBTRACT`)",
    ),
];

/// ⛔⛔ **GATE — o gesto é INERTE, e as duas fixturas estão fora do corpus vivo.**
///
/// ⚠️ **A metade que interessa não é o predicado: é o BARRO.** Perguntar ao
/// [`Verb::honours_invert`] afirmaria o que a tabela diz de si mesma; o que este
/// gate afirma é que correr a fixtura com `invert = true` dá **exactamente** o
/// mesmo bloco de vértices que com `invert = false`. *Um predicado é um resumo
/// do produto, e um resumo pode estar à frente dele.*
///
/// ⚠️ **E a metade da OBSOLESCÊNCIA** (`CLAUDE.md` §5.0: *uma catraca sem
/// censo de obsolescência vira LICENÇA*): no dia em que a decisão do dono mudar
/// e o verbo voltar a honrar o gesto, este gate reprova a dizer que as duas
/// fixturas têm de **voltar** ao [`CENAS`] e o [`VERDE_N`] de subir — em vez de
/// elas ficarem esquecidas numa lista de exclusão que ninguém relê.
#[test]
fn o_ctrl_saiu_do_projectar_e_o_corpus_diz_quais_fixturas_isso_custou() {
    assert!(
        !Verb::SceneProject.honours_invert(),
        "o `Ctrl` voltou a este verbo (ordem do dono de 15/09 revertida?) —          reponha as {} fixturas de `FORA_POR_DECISAO_DO_DONO` no `CENAS` e suba          o `VERDE_N`, que elas mediam `2,384e-7`",
        FORA_POR_DECISAO_DO_DONO.len()
    );
    for (nome, porque) in FORA_POR_DECISAO_DO_DONO {
        assert!(
            !CENAS.iter().any(|(n, _)| *n == nome),
            "{nome} está nos DOIS sítios — ou ela é corpus vivo, ou é divergência              declarada ({porque})"
        );
        // ⭐ A prova no BARRO: o gesto não muda um único bit.
        let f = Fix::ler(nome);
        let alvos: Vec<(Mesh, Pose)> = [-0.5f32].iter().map(|&h| alvo(h)).collect();
        let mut b = pincel(&f);
        b.invert = false;
        let sem = correr_com(&f, &b, alvos.clone(), false);
        b.invert = true;
        let com = correr_com(&f, &b, alvos, false);
        let d = sem
            .positions()
            .iter()
            .zip(com.positions())
            .map(|(x, y)| (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max);
        assert!(
            d == 0.0,
            "{nome}: com `invert` a saída mudou {d:.3e} — o gesto voltou a ter              efeito neste verbo, e a decisão do dono diz que não tem"
        );
    }
}

/// **A BARRA APERTADA** — o desvio de um dab, onde a lei está sozinha.
///
/// ⚠️ Ela não é escolhida: é a ordem do `f32` nestas coordenadas, e as duas
/// fixturas de um passo medem `7,078e-8`.
const BARRA: f32 = 2e-6;

/// Quantas fixturas do corpus reconstrutível caem dentro da [`BARRA`] hoje.
///
/// ⛔⛔ **CATRACA: este número só SOBE**, e ele subiu de `3` para `12` e depois
/// para **`14`** em 2026-09-15.
///
/// ⭐⭐⭐ **E a subida não foi afinação: era UMA COLUNA da tabela de grips.** A
/// redacção anterior lia a partição *«um dab bate ao sétimo decimal, seis dabs
/// desviam»* como sendo a **composição do traço** — e era o `GripLaw::from_live`
/// preso ao interruptor `Accumulate`. A espec §6.5 diz que este verbo mede *«a
/// partir de onde o vértice está AGORA»* **sempre**, e com a coluna pregada
/// **nove** fixturas de seis dabs saltaram de `5,9e-2`–`2,6e-1` para
/// `8,9e-8`–`2,0e-7`.
///
/// ⚠️ *Uma partição limpa diz ONDE procurar e não O QUE procurar* — a leitura
/// «é a composição» era a primeira hipótese compatível com ela, e a errada.
///
/// ⭐⭐⭐ **E DEPOIS `12 → 14`, com a MESMA forma de defeito uma camada acima:**
/// o par `…_invertido` / `…_subtrair` desviava `1,415e-1`, que é **exactamente**
/// o número que a espec §6.6 publica para `max abs(d + d′)` entre a base e a
/// invertida. *Ler o próprio desvio no número que a espec dá para a assimetria
/// do alvo é o diagnóstico inteiro* — significava que a NOSSA saída era o
/// espelho exacto da base, e a dele não.
///
/// ⛔⛔ **A causa eram DUAS suposições erradas que encaixavam uma na outra:** a
/// cena destas duas tinha sido reconstruída com o alvo **ACIMA** (lido do
/// deslocamento máximo, que é para cima) e a lei fazia a inversão **virar o
/// raio** — e cada uma «provava» a outra, porque com o alvo em cima a negação
/// no fim mede `0` vértices movidos. O que as separou foi o **cabeçalho**: a
/// base e a invertida são idênticas campo a campo (`sentido: ADD` nas duas) e a
/// base diz *«contra um plano ABAIXO»*. ⇒ alvo a `−0,5` nas duas, e a inversão
/// **nega a translação** (espec §1.2: *«o sinal entra no factor»*). Medido:
/// `1,415e-1 → 2,384e-7`.
///
/// ⚠️ **A varredura da ALTURA foi o que fechou a porta antes da cura:** com a
/// lei do raio virado, **nenhuma** altura de `−1,0` a `+1,0` põe a fixtura
/// dentro da barra (o melhor é `1,250e-1` a `h = +0,625`) — *se nenhum valor do
/// parâmetro livre encaixa, o que está errado é a lei, não a cena.*
///
/// ⭐⭐⭐ **E DEPOIS `14 → 15`: a DIRECÇÃO DO RAIO não pode derivar com a
/// trincheira que o próprio traço abre.** O `…_normal_plano_area` desviava
/// `1,281e-1` e o desvio era **inteiramente LATERAL** (o nosso `dx`/`dy` lia
/// `1,281e-1` contra `0,000e0` do oráculo) — no corpus aquela fixtura é
/// **byte-idêntica** à `…_base` (`max abs(Δ) = 0,0` nos `1 681` vértices), ou
/// seja a direcção do plano e a da vista coincidem e **continuam a coincidir**
/// com o barro já a afundar.
///
/// ⚠️ **A cura tem DUAS metades, e sem a segunda ela pára a meio:** ajustar o
/// plano sobre a superfície congelada (a lista por verbo que o
/// [`ph2d_sculpt3d::Verb::ClayStrips`] já usava) leva o desvio a `1,036e-2`, e
/// o que sobra é que o `base_nrm` é o **PRIMEIRO TOQUE** e não o pen-down — um
/// vértice que entra na pegada ao 3.º dab é fotografado já inclinado. Com a
/// fotografia das normais no primeiro dab: **`1,639e-7`**, o número da `…_base`.
///
/// ⭐⭐⭐⭐ **E A ÚLTIMA QUE FALTAVA FECHOU EM 2026-09-19, sem uma linha deste
/// verbo se mexer** — a `…_dureza05` passou de **`2,367e-2` para `4,619e-7`**
/// quando a máscara de alcance ganhou a lei *«a máscara decide quem ENTRA no
/// traço; ela nunca decide quem SAI»*
/// ([`ph2d_sculpt3d::dab_alcance::MemoriaDoTraco`], escrita por dois reports do
/// dono sobre o GANCHO).
///
/// ⚠️⚠️ **E o diagnóstico que estava escrito aqui era o certo pela razão
/// errada.** Ele dizia: *«`3` vértices de `301` na BORDA da pegada, que com o
/// `from_live` se move enquanto o barro afunda — o dab em que um vértice sai da
/// esfera é decidido ao último bit»*. A borda que se movia **não era a da
/// esfera de consulta: era a da MÁSCARA**, que re-julgava a cada dab o barro que
/// já estava a andar. *Uma partição pode nomear o sítio certo e o mecanismo
/// errado, e só a cura do mecanismo o separa.*
/// ⛔⛔⛔ **E DEPOIS `15 → 13`, e ISTO NÃO É UMA DESCIDA DA CATRACA: é a
/// POPULAÇÃO a encolher por ordem do dono.** Ele smokou a `=45` e decidiu:
/// *«não vi utilidade na feature Scene Project + CTRL. Melhor retirá-la e
/// documentá-la como indesejada.»* ⇒ as duas fixturas que exercitavam a
/// inversão saíram do [`CENAS`] para o [`FORA_POR_DECISAO_DO_DONO`] **verdes, a
/// `2,384e-7` cada**.
///
/// ⚠️ **A distinção é a coisa toda:** uma catraca que desce porque o produto
/// regrediu e uma que desce porque duas perguntas deixaram de ser feitas leem-se
/// **igual no número**. O que as separa é o denominador — `13` de **`14`**
/// contra `15` de `16` — e o gate
/// `o_ctrl_saiu_do_projectar_e_o_corpus_diz_quais_fixturas_isso_custou`, que
/// reprova no dia em que a decisão mudar sem as fixturas voltarem. *Escrever
/// `13` sem escrever porquê seria a catraca a virar LICENÇA no sentido
/// contrário: a próxima pessoa afrouxaria o número outra vez e chamaria-lhe
/// história.*
const VERDE_N: usize = 14;

/// ⭐⭐⭐ **O CORPUS INTEIRO DO QUE É RECONSTRUTÍVEL** — `14` das `24`, cada uma
/// com a cena medida da própria saída.
///
/// # ⭐⭐ O que está PROVADO, e é mais do que um número de paridade
///
/// **As `14` movem EXACTAMENTE o mesmo conjunto de vértices que o oráculo**
/// (`301` contra `301`, zero discordâncias fora da banda de empate). Isso não é
/// um detalhe: é a pegada, a direcção do raio, a regra dos dois sentidos, a
/// escolha entre dois alvos, a inversão e as três recusas da §6.3 — **todas**
/// estruturalmente certas. *Um conjunto igual com magnitudes diferentes é um
/// diagnóstico muito mais preciso do que um número agregado.*
///
/// ⚠️ **E a `inversão` saiu desta frase em 15/09** — ela continua estruturalmente
/// certa e **deixou de ser NOSSA**: ver [`FORA_POR_DECISAO_DO_DONO`].
///
/// # ✅ O que está ABERTO: **NADA** — as `14` batem a barra desde 19/09
///
/// ⚠️ **A tabela abaixo fica com a `…_dureza05` NA LINHA DELA**, hoje a
/// `4,619e-7`: *o número que ela media enquanto estava aberta é o que torna
/// legível o que a cura comprou.*
///
/// | fixtura | dabs | curva | desvio |
/// |---|---|---|---|
/// | `…_acima_sem_bidir` | 6 | *Smooth* | **`0`** (nada se move, dos dois lados) |
/// | `…_constante_1passo` · `…_forca05_constante_1passo` · `…_forca05` | 1–6 | *Constant*/*Smooth* | `7,078e-8` |
/// | `…_mindist06` | 6 | *Smooth* | `8,941e-8` |
/// | `…_dois_abaixo` · `…_dois_lados_bidir` | 6 | *Smooth* | `1,043e-7` |
/// | `…_mindist01` | 6 | *Smooth* | `1,341e-7` |
/// | `…_base` · `…_base_repete` · `…_acima_bidir` · `…_normal_plano_area` | 6 | *Smooth* | `1,639e-7` |
/// | `…_mindist01_acima_bidir` | 6 | *Smooth* | `2,012e-7` |
/// | **`…_dureza05`** | 6 | *Smooth* | **`4,619e-7`** — era `2,367e-2`, a única fora da barra, até a lei da memória do traço |
///
/// ⛔⛔ **ESTA TABELA ESTEVE ERRADA E A PROSA DEBAIXO DELA TAMBÉM, até 15/09.**
/// Ela listava `…_base` a `1,902e-1` e conclía que *«tudo o que corre em UM dab
/// bate ao sétimo decimal e tudo o que corre em SEIS desvia ⇒ o que falta é a
/// composição por dab»* — e essa leitura foi **REFUTADA** no mesmo ficheiro, uma
/// dezena de linhas acima, quando a coluna `GripLaw::from_live` pôs nove
/// fixturas de seis dabs dentro da barra. ⚠️ *Duas leituras do MESMO corpus a
/// discordar na MESMA página — e a que envelhece é sempre a tabela, porque ela
/// é a que ninguém recalcula ao curar.* ⇒ os números acima saem da sonda
/// `diag_o_placar`, corrida nesta árvore.
///
/// ⛔⛔ **A barra NÃO foi afrouxada para engolir a que falta**, e essa é a
/// decisão: uma barra de `3e-1` faria este gate ficar verde sobre qualquer coisa.
/// *Uma barra que aceita o desvio que se tem mede o desvio que se tem.*
#[test]
fn o_corpus_reconstrutivel_bate_o_oraculo() {
    let entrada = grelha(false);
    let (mut verdes, mut relatorio) = (0usize, Vec::new());
    for (nome, alturas) in CENAS {
        let f = Fix::ler(nome);
        let alvos: Vec<(Mesh, Pose)> = alturas.iter().map(|&h| alvo(h)).collect();
        let b = pincel(&f);
        let nosso = correr_com(&f, &b, alvos, false);
        let v = comparar(&nosso, &entrada, &f);
        relatorio.push(format!(
            "{nome}: desvio {:.3e} · {} contra {} movidos · {} discordam ({} na banda)",
            v.desvio, v.nossos, v.deles, v.discordam, v.empates
        ));
        // ⭐⭐ **A metade ESTRUTURAL, e ela vale para as CATORZE:** o conjunto
        // de vértices que se move tem de ser o mesmo, a menos dos que estão na
        // banda de empate do raio.
        assert!(
            v.discordam <= v.empates,
            "{nome}: {} vértices discordam e só {} estão na banda de empate — \
             isso é lei, não borda\n{}",
            v.discordam,
            v.empates,
            relatorio.join("\n")
        );
        if v.desvio < BARRA {
            verdes += 1;
        }
    }
    assert!(
        verdes >= VERDE_N,
        "o placar desceu: {verdes} dentro da barra contra {VERDE_N}\n{}",
        relatorio.join("\n")
    );
    // ⛔ **A outra metade da catraca:** no dia em que a composição por dab for
    // curada, este gate reprova e manda subir o número — em vez de o ganho
    // passar despercebido.
    assert_eq!(
        verdes,
        VERDE_N,
        "o placar SUBIU para {verdes} — actualize `VERDE_N` e a tabela do \
         cabeçalho, que é onde a partição de um-dab contra seis-dabs se lê\n{}",
        relatorio.join("\n")
    );
}

/// ⭐⭐ **O ALVO PERDEDOR É INOBSERVÁVEL** — a nota que completa a reconstrução
/// das duas fixturas de dois alvos.
///
/// ⚠️ A espec §6.3 diz que ganha o de **menor `|d|`**, logo a posição do
/// perdedor não pode entrar na resposta. *Sem este gate, a altura que eu escrevi
/// para ele seria um palpite a viajar dentro de uma bancada que se diz medida.*
#[test]
fn o_alvo_perdedor_e_inobservavel() {
    let f = Fix::ler("projectar_dois_abaixo");
    let entrada = grelha(false);
    let b = pincel(&f);
    let base = correr_com(&f, &b, vec![alvo(-0.3), alvo(-0.9)], false);
    for longe in [-0.8f32, -1.5, -3.0] {
        let outro = correr_com(&f, &b, vec![alvo(-0.3), alvo(longe)], false);
        let d = base
            .positions()
            .iter()
            .zip(outro.positions())
            .map(|(x, y)| (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max);
        assert!(
            d < 1e-9,
            "mover o alvo perdedor para {longe} mudou a saída em {d:.3e} — ele \
             deixou de ser inobservável, e a cena desta fixtura passou a ser um \
             palpite"
        );
    }
    let _ = entrada;
}

/// **SONDA** — o desvio de cada uma das `14`, impresso sem reprovar.
#[test]
#[ignore]
fn diag_o_placar() {
    let entrada = grelha(false);
    for (nome, alturas) in CENAS {
        let f = Fix::ler(nome);
        let alvos: Vec<(Mesh, Pose)> = alturas.iter().map(|&h| alvo(h)).collect();
        let b = pincel(&f);
        let nosso = correr_com(&f, &b, alvos, false);
        let v = comparar(&nosso, &entrada, &f);
        eprintln!(
            "{nome:<38} desvio {:.3e} · {} contra {} movidos · {} discordam ({} banda)",
            v.desvio, v.nossos, v.deles, v.discordam, v.empates
        );
    }
}

/// **SONDA** — varre a ALTURA do alvo para uma fixtura, com o pincel dela.
#[test]
#[ignore]
fn diag_varre_a_altura() {
    for nome in ["projectar_invertido", "projectar_normal_plano_area"] {
        let f = Fix::ler(nome);
        let entrada = grelha(false);
        let mut b = pincel(&f);
        if nome == "projectar_invertido" {
            b.invert = true;
        }
        eprintln!("--- {nome} ---");
        let mut melhor = (f32::INFINITY, 0.0f32);
        for k in -40..=40i32 {
            let h = f32::from(i16::try_from(k).expect("cabe")) * 0.025;
            if h.abs() < 0.05 {
                continue;
            }
            let nosso = correr_com(&f, &b, vec![alvo(h)], false);
            let v = comparar(&nosso, &entrada, &f);
            if v.nossos == 0 {
                continue;
            }
            if v.desvio < melhor.0 && v.discordam <= v.empates {
                melhor = (v.desvio, h);
            }
            if k % 4 == 0 {
                eprintln!(
                    "  h {h:+.3}: desvio {:.3e} · {} contra {} · {} discordam",
                    v.desvio, v.nossos, v.deles, v.discordam
                );
            }
        }
        eprintln!("  MELHOR: h {:+.4} com desvio {:.3e}", melhor.1, melhor.0);
    }
}

/// **SONDA** — o perfil do desvio de uma fixtura, por distância ao cursor.
#[test]
#[ignore]
fn diag_o_perfil_do_desvio() {
    for (nome, alturas) in CENAS {
        if !matches!(nome, "projectar_dureza05" | "projectar_normal_plano_area") {
            continue;
        }
        let f = Fix::ler(nome);
        let b = pincel(&f);
        let alvos: Vec<(Mesh, Pose)> = alturas.iter().map(|&h| alvo(h)).collect();
        let nosso = correr_com(&f, &b, alvos, false);
        let raio = f.num("raio_objeto");
        eprintln!("--- {nome} (dureza {} raio {raio}) ---", f.num("dureza"));
        let mut lateral = 0.0f32;
        let mut linhas: Vec<(f32, f32, f32)> = Vec::new();
        for ((n, o), r) in nosso.positions().iter().zip(&f.s).zip(&f.r) {
            lateral = lateral.max((n[0] - r[0]).abs()).max((n[1] - r[1]).abs());
            let perto =
                f.c.iter()
                    .map(|c| {
                        let d = [r[0] - c[0], r[1] - c[1], r[2] - c[2]];
                        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
                    })
                    .fold(f32::INFINITY, f32::min);
            if (n[2] - r[2]).abs() > 1e-9 || (o[2] - r[2]).abs() > 1e-9 {
                linhas.push((perto / raio, n[2] - r[2], o[2] - r[2]));
            }
        }
        let lateral_deles =
            f.s.iter()
                .zip(&f.r)
                .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
                .fold(0.0f32, f32::max);
        eprintln!("  lateral: nosso {lateral:.3e} · oraculo {lateral_deles:.3e}");
        linhas.sort_by(|a, b| (b.1 - b.2).abs().total_cmp(&(a.1 - a.2).abs()));
        eprintln!("  OS 10 PIORES");
        eprintln!(
            "  {:>7} {:>11} {:>11} {:>11}",
            "t", "nosso", "oraculo", "delta"
        );
        for &(t, n, o) in linhas.iter().take(10) {
            eprintln!("  {t:7.3} {n:11.6} {o:11.6} {:11.6}", n - o);
        }
    }
}
