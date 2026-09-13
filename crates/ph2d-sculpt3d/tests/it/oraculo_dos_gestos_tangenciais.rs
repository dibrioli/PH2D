//! ⭐⭐⭐ **PARIDADE COM O ORÁCULO — os dois gestos TANGENCIAIS** (o polegar e o
//! empurrão), sobre malhas NOSSAS
//! ([`docs/3D/cleanroom/fixtures/pull/`](../../../docs/3D/cleanroom/fixtures/pull/README.md)),
//! corridos pelo NOSSO motor e comparados vértice a vértice.
//!
//! A lei está em [`docs/3D/cleanroom/SPEC_pull_brushes.md`](../../../docs/3D/cleanroom/SPEC_pull_brushes.md)
//! (§5 e §6); aqui mora a PROVA.
//!
//! ⚠️ **As fixtures são DADOS, não expressão** (GPLv2 §0: posições de vértices
//! de uma malha nossa não são obra baseada no programa). O I lê-as; regenerá-las
//! é acto do E.
//!
//! # ⛔ A barra é DERIVADA, e não paridade ao bit
//!
//! Ver a [`TOL`]: `1e-6` em unidades de OBJECTO, que é o que a re-associação de
//! cinco factores em `f32` dá sobre estas malhas. ⛔ Bit-parity não é a meta num
//! pipeline T2, por decisão registada da casa (ADR-0162).
//!
//! # O que esta bancada AFIRMA e o que ela deixa ABERTO
//!
//! | | afirma | aberto (com o número e a catraca) |
//! |---|---|---|
//! | **polegar** | o plano, 7 fixtures | a esfera, 7 — ver [`POLEGAR_CURVO`] |
//! | **empurrão** | o plano, 8 fixtures | a esfera, 5 — ver [`EMPURRAO_CURVO`] |
//!
//! ⚠️ **As duas listas abertas têm CATRACA**: se o resíduo cair abaixo da barra,
//! o teste delas **reprova** e manda mover a fixture para o gate de paridade.
//! *Uma dívida sem censo de obsolescência vira licença.*

use ph2d_mesh::{Face, Mesh};
use ph2d_sculpt3d::{Brush, Dab, Falloff, Grip, RefMode, SculptStroke, Symmetry, Verb};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// **A BARRA — `2e-6` em unidades de OBJECTO, e ela é DERIVADA.**
///
/// **O recurso é a precisão de `f32`, e a grandeza é a POSIÇÃO do vértice.** As
/// malhas desta bancada vivem em `±1,5`, onde um `ulp` vale `1,19e-7`. A nossa
/// cadeia compõe cinco factores numa ordem que não é a do oráculo — a casa já
/// mediu que re-associar um produto de **três** move ~1 ulp em 30 % dos casos —
/// e o resultado acumula evento a evento.
///
/// **Medido nas 15 fixtures planas (2026-09-13):**
///
/// | | pior desvio | em ulps |
/// |---|---|---|
/// | traços curtos (2 a 6 eventos) | `4,5e-7` | 4 |
/// | traços de 12 a 24 eventos | `7,2e-7` | 6 |
/// | as duas truncagens de **11** eventos | **`1,31e-6`** | **11** |
///
/// ⭐ **E o desvio CRESCE com o número de eventos**, que é a assinatura da
/// acumulação e não de um erro de lei — um erro de lei escalaria com o
/// deslocamento, e o deslocamento da `k11` é menor que o do traço inteiro.
///
/// ⇒ a barra é **`2e-6` (≈ 17 ulps)**: a medição com folga de meia ordem. ⛔ Não
/// um epsilon de conforto — a prova de mutação sangra `10⁴×` acima dela. ⛔ E não
/// paridade ao bit, que o ADR-0162 recusa prometer num pipeline T2.
const TOL: f32 = 2e-6;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/pull")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da bancada do tecido: o
/// cabeçalho do gzip tem tamanho variável e o rabo são oito bytes.
fn inflar(nome: &str) -> String {
    let raw = std::fs::read(fixture_dir().join(nome)).unwrap_or_else(|e| panic!("{nome}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{nome}: nao e' gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        let xlen = usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8);
        off += 2 + xlen;
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
        .unwrap_or_else(|e| panic!("{nome}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

fn v3(campos: &[&str]) -> [f32; 3] {
    [
        campos[0].parse().expect("x"),
        campos[1].parse().expect("y"),
        campos[2].parse().expect("z"),
    ]
}

/// As posições de repouso de uma superfície, na ordem em que o oráculo as leu.
fn repouso(superficie: &str) -> Vec<[f32; 3]> {
    inflar(&format!("{superficie}.repouso.txt.gz"))
        .lines()
        .filter(|l| l.starts_with("r "))
        .map(|l| v3(&l.split_whitespace().skip(1).collect::<Vec<_>>()))
        .collect()
}

/// Um traço do oráculo: o cabeçalho, o percurso do cursor e as posições finais.
struct Traco {
    chaves: BTreeMap<String, String>,
    caminho: Vec<[f32; 3]>,
    depois: Vec<[f32; 3]>,
}

fn traco(nome: &str) -> Traco {
    let texto = inflar(&format!("{nome}.deformado.txt.gz"));
    let mut chaves = BTreeMap::new();
    let (mut caminho, mut depois) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        if l.starts_with('#') || l.trim().is_empty() {
            continue;
        }
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos[0] {
            "c" => caminho.push(v3(&campos[1..])),
            "d" => depois.push(v3(&campos[1..])),
            k => {
                if campos.len() >= 2 {
                    chaves.insert(k.to_string(), campos[1].to_string());
                }
            }
        }
    }
    Traco {
        chaves,
        caminho,
        depois,
    }
}

impl Traco {
    fn s(&self, k: &str) -> &str {
        self.chaves.get(k).map_or("", String::as_str)
    }
    fn f(&self, k: &str) -> f32 {
        self.s(k)
            .parse()
            .unwrap_or_else(|_| panic!("chave {k} ausente ou ilegivel"))
    }
}

/// **A MALHA DA FIXTURE — lida do ficheiro, com o CONTROLO que a torna
/// afirmável.**
///
/// ⚠️⚠️ **A topologia vem do oráculo e as posições são CONFERIDAS contra o
/// `.repouso`:** se a ordem dos vértices divergisse, a comparação final
/// compararia vértices diferentes e leria como erro de LEI. *Uma bancada que não
/// prova que mede o mesmo sujeito não prova nada.*
///
/// ⚠️ **Uma face tem TRÊS ou QUATRO índices** — a esfera traz leques
/// triangulares nos pólos e quads no resto. Quem assumir quatro lê a esfera
/// errada, e o erro é mudo (a malha fecha na mesma).
fn superficie(nome: &str) -> Mesh {
    let texto = inflar(&format!("{nome}.malha.txt.gz"));
    let (mut pos, mut faces) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos.first().copied() {
            Some("v") => pos.push(v3(&campos[1..])),
            Some("f") => {
                let i = |k: usize| campos[k].parse::<u32>().expect("indice");
                faces.push(match campos.len() {
                    4 => Face::tri(i(1), i(2), i(3)),
                    5 => Face::quad(i(1), i(2), i(3), i(4)),
                    n => panic!("{nome}: face com {} indices", n - 1),
                });
            }
            _ => {}
        }
    }
    let esperado = repouso(nome);
    assert_eq!(pos.len(), esperado.len(), "{nome}: contagem de vertices");
    for (i, alvo) in esperado.iter().enumerate() {
        for k in 0..3 {
            assert!(
                (pos[i][k] - alvo[k]).abs() <= 1e-6,
                "{nome}: vertice {i} eixo {k}: {} contra {}",
                pos[i][k],
                alvo[k]
            );
        }
    }
    Mesh::from_parts(pos, faces).expect("malha da fixture")
}

/// A direcção do olhar: **do olho para a cena**, que é a convenção do [`Dab`].
fn olho(vista: &str) -> [f32; 3] {
    match vista {
        "topo" => [0.0, 0.0, -1.0],
        "frente" => [0.0, 1.0, 0.0],
        outra => panic!("vista {outra} nao mapeada"),
    }
}

fn pincel(t: &Traco) -> Brush {
    Brush {
        verb: match t.s("gesto") {
            "polegar" => Verb::Thumb,
            "empurrao" => Verb::Nudge,
            // ⭐ O agarrar entra só pela SONDA (ver
            // [`sonda_do_agarrar_que_ja_shipamos`]): ele é um verbo que já
            // shipa, e medi-lo aqui é a primeira vez que ele tem um lado
            // aprovado.
            "agarrar" => Verb::Move,
            g => panic!("gesto {g} nao e' desta bancada"),
        },
        // ⚠️ **O modo é o `B`**: estes dois verbos são da referência restrita e o
        // `S` não os declara (ver `RefMode::declares`).
        mode: RefMode::B,
        radius: t.f("raio"),
        strength: t.f("forca"),
        falloff: match t.s("curva") {
            "suave" => Falloff::Smooth,
            c => panic!("curva {c} nao exercitada por esta bancada"),
        },
        hardness: t.f("dureza"),
        normal_radius_frac: t.f("fator_raio_da_normal"),
        // Todas as fixtures desta bancada correm sem acumulação (cabeçalho).
        accumulate: false,
        ..Brush::default()
    }
}

fn menos(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// **O GESTO VIVE NO PLANO DO ECRÃ** — a componente ao longo da vista sai.
///
/// ⚠️⚠️ **Isto não é cosmética: é o §1 da espec, e sem ele a esfera lê errado.**
/// O percurso gravado são os pontos que o cursor apanhou **na superfície**, e
/// numa esfera a corda entre dois deles mergulha para dentro da peça. O gesto
/// que a lei usa é a desprojecção do cursor no plano paralelo ao ECRÃ — logo a
/// componente ao longo do olhar não faz parte dele. Num plano visto de cima a
/// diferença é zero, e é por isso que só a esfera a revelou (`1,7e-1` de desvio
/// antes desta linha existir).
fn no_plano_do_ecra(v: [f32; 3], eye: [f32; 3]) -> [f32; 3] {
    let dot = v[0] * eye[0] + v[1] * eye[1] + v[2] * eye[2];
    [
        v[0] - eye[0] * dot,
        v[1] - eye[1] * dot,
        v[2] - eye[2] * dot,
    ]
}

/// **O TRAÇO, conduzido como o oráculo foi conduzido:** um dab por evento.
///
/// ⚠️ **E a diferença entre os dois gestos vive aqui, não no kernel:** o
/// ancorado carimba sempre na âncora com o puxão TOTAL; o que viaja carimba sob
/// o cursor com o INCREMENTO. É a mesma partição que o `Grip` já faz.
fn correr(t: &Traco) -> Mesh {
    let mut mesh = superficie(t.s("superficie"));
    let brush = pincel(t);
    let eye = olho(t.s("vista"));
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 1..t.caminho.len() {
        // ⚠️⚠️ **A pergunta é ao GRIP, e não ao VERBO — e isto foi um defeito
        // desta bancada, medido:** com `verb == Thumb` no lugar desta linha, o
        // agarrar (que é o mesmo grip) recebia o INCREMENTO em vez do total e
        // lia `0,054` contra os `0,600` do oráculo. Eu estive a um passo de
        // registar uma divergência de `11×` num verbo que já shipa — e o que a
        // desfez foi o discriminador mais barato: o mesmo puxão total entregue
        // em `1`, `2`, `4` e `12` eventos dá **`0,600000` nas quatro**.
        // *Uma régua que pergunta pelo NOME erra na primeira família nova.*
        let (centro, puxao) = if matches!(brush.verb.grip(), Grip::Hold) {
            (t.caminho[0], menos(t.caminho[k], t.caminho[0]))
        } else {
            (t.caminho[k], menos(t.caminho[k], t.caminho[k - 1]))
        };
        let puxao = no_plano_do_ecra(puxao, eye);
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::pulling(centro, brush.radius, eye, puxao),
            Symmetry::default(),
        );
    }
    mesh
}

/// O pior desvio contra a fixture, **em unidades de OBJECTO** (absoluto).
///
/// ⚠️⚠️ **Ele era relativo ao deslocamento da fixture, e essa régua estava
/// errada** — a primeira corrida desta bancada leu `1,09e-5` numa truncagem e
/// `1,19e-6` no traço inteiro, e os dois são **o mesmo erro absoluto**
/// (`~6e-7`): o ruído de `f32` vive na POSIÇÃO do vértice, não no quanto ele
/// andou. Dividir pelo deslocamento fazia uma fixture curta parecer dez vezes
/// pior por ter andado dez vezes menos. *Uma régua normalizada pela grandeza
/// errada fabrica uma tabela de dívida.*
fn desvio(t: &Traco, malha: &Mesh) -> f32 {
    assert_eq!(malha.vert_count(), t.depois.len(), "contagem de vertices");
    let mut pior = 0.0f32;
    for (i, alvo) in t.depois.iter().enumerate() {
        let nosso = malha.positions()[i];
        let d = menos(nosso, *alvo);
        pior = pior.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    pior
}

/// O deslocamento máximo que o NOSSO motor produziu (a régua que o cabeçalho da
/// fixture também traz, para os controlos que não comparam vértice a vértice).
fn pico_nosso(t: &Traco, malha: &Mesh) -> f32 {
    let base = repouso(t.s("superficie"));
    let mut pico = 0.0f32;
    for (i, r) in base.iter().enumerate() {
        let d = menos(malha.positions()[i], *r);
        pico = pico.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    pico
}

/// As fixtures planas de cada gesto — ⛔ a lista é ESCRITA porque o directório
/// tem fixtures de outros gestos e de superfícies sem topologia; o gate
/// `as_fixturas_desta_bancada_existem_todas` é que a mantém honesta.
/// ⛔ **`polegar_plano_ancorado` FICA DE FORA, e a razão é uma fixture que não
/// se lê pelo cabeçalho:** ela tem o cabeçalho **idêntico** ao da
/// `polegar_plano_origem` — as vinte chaves, uma a uma — e o **mesmo percurso**,
/// e ainda assim o oráculo move `509` vértices onde a outra move `177`. A única
/// coisa que a distingue é o NOME: ela corre com o *método de traço ancorado*,
/// em que o raio cresce com o arrasto — um método que o nosso motor não tem (um
/// dab por evento, raio fixo). ⇒ está NOMEADA como aberta, e a emenda pedida ao
/// E é o cabeçalho passar a gravar o método. *Uma fixture cuja variável
/// distintiva não está no cabeçalho não é legível por um gate.*
const POLEGAR: [&str; 7] = [
    "polegar_plano_origem",
    "polegar_plano_forca05",
    "polegar_plano_passos24",
    "polegar_plano_diagonal",
    "polegar_plano_origem_k02",
    "polegar_plano_origem_k06",
    "polegar_plano_origem_k11",
];

/// ⏳ **A ESFERA do gesto ancorado — ABERTA, com a atribuição MEDIDA e por
/// fechar.**
///
/// ⭐ **O que já está fechado aqui:** os dois lados movem **exactamente** os
/// mesmos `492` vértices, e **até ao 8.º evento a direcção do deslocamento bate
/// ao bit** (`cos = 1,000000`, magnitude a `1,4e-4`). ⇒ a pegada, os pesos, a
/// normal do gesto e a projecção do gesto no plano do ecrã estão certas.
///
/// ⭐⭐⭐ **E a maior metade JÁ FECHOU, com a causa medida do NOSSO lado:** a
/// pegada era consultada nas posições VIVAS, e um gesto que desloca `0,38` num
/// pincel de raio `0,35` leva os próprios vértices para fora dela ⇒ a média que
/// dá a normal encolhia com o traço e a direcção rodava `1,2°`. Congelada a
/// pegada no pen-down (o que a espec §4 sempre disse), o desvio cai de
/// **`8,5e-3` para `1,9e-4`** — `44×` — e o PLANO não move um bit.
///
/// ⚠️ **Quem apontou o dedo foi o oráculo, não o nosso raciocínio:** o E mediu
/// nove comprimentos do mesmo traço e a razão `pico/|Δ|` dele é **constante**
/// (`0,9549005`, `5,7e-6` de dispersão) enquanto a nossa crescia. *Um invariante
/// do lado aprovado transforma «não bate» em «o erro é da magnitude, e cresce
/// com os eventos».*
///
/// ⏳ **O que ainda fica:** `1,3e-5` a `1,9e-4`, e ele **também** cresce com o
/// número de eventos (`2,1e-5` a 2 eventos · `1,5e-4` a 8). A espec §6.3-bis
/// mede, no gesto que viaja, que a normal do alvo **se atrasa** ao longo do
/// traço — é a mesma assinatura, e é a próxima pergunta.
///
/// ⚠️ **A sonda que separa direcção de magnitude é a
/// [`sonda_do_residuo_da_esfera`]** — e foi ela que tornou esta entrada uma
/// pergunta estreita em vez de *«a esfera não fecha»*.
const POLEGAR_CURVO: [&str; 7] = [
    "polegar_esfera_topo",
    "polegar_esfera_topo_raionormal03",
    "polegar_esfera_frente",
    "polegar_esfera_topo_k02",
    "polegar_esfera_topo_k04",
    "polegar_esfera_topo_k06",
    "polegar_esfera_topo_k08",
];

const EMPURRAO_PLANO: [&str; 8] = [
    "empurrao_plano_origem",
    "empurrao_plano_forca05",
    "empurrao_plano_passos24",
    "empurrao_plano_parado",
    "empurrao_plano_ida_volta",
    "empurrao_plano_origem_k02",
    "empurrao_plano_origem_k05",
    "empurrao_plano_origem_k11",
];

/// ⏳ **AS CURVAS DO EMPURRÃO, que a espec declara ABERTAS** (§6.3): o resíduo é
/// de `~10 %` e a causa **não é a lei** — quatro variantes da normal foram
/// medidas e dão o mesmo número. O que falta é o centro que o oráculo usou em
/// cada evento (ele reamostra a superfície VIVA, que se deforma debaixo do
/// traço). Aqui elas são **regressão**, nunca paridade.
const EMPURRAO_CURVO: [&str; 5] = [
    "empurrao_esfera_topo",
    "empurrao_esfera_topo_k04",
    "empurrao_esfera_topo_k08",
    "empurrao_esfera_topo_raionormal03",
    "empurrao_esfera_frente",
];

/// Corre a lista inteira, IMPRIME a tabela e só então cobra a barra — *um
/// `assert` no meio do laço esconde as fixtures que vinham a seguir*.
fn cobrar(lista: &[&str], gesto: &str) {
    println!("  fixture                           |   desvio   | pico nosso | pico dele");
    let mut pior = (0.0f32, "");
    for &nome in lista {
        let t = traco(nome);
        assert_eq!(t.s("gesto"), gesto, "{nome}: gesto do cabecalho");
        let malha = correr(&t);
        let d = desvio(&t, &malha);
        println!(
            "  {nome:<33} | {d:10.3e} | {:10.6} | {:9.6}",
            pico_nosso(&t, &malha),
            t.f("max_deslocamento")
        );
        if d > pior.0 {
            pior = (d, nome);
        }
    }
    println!("  pior: {:.3e} ({})   barra: {TOL:.0e}", pior.0, pior.1);
    assert!(
        pior.0 <= TOL,
        "{}: desvio {:.3e} acima da barra {TOL:.0e}",
        pior.1,
        pior.0
    );
}

/// ⭐ **O POLEGAR reproduz o oráculo — no plano E na esfera.**
#[test]
fn o_polegar_reproduz_o_oraculo() {
    cobrar(&POLEGAR, "polegar");
}

/// ⭐ **O EMPURRÃO reproduz o oráculo no plano.**
#[test]
fn o_empurrao_reproduz_o_oraculo_no_plano() {
    cobrar(&EMPURRAO_PLANO, "empurrao");
}

/// ⏳ **O EMPURRÃO EM SUPERFÍCIE CURVA — o item ABERTO, medido e com CATRACA.**
///
/// ⚠️ **Ele não afirma paridade; ele impede a REGRESSÃO e obriga a fechar o
/// item quando ele fechar:** se o resíduo cair abaixo da barra de paridade, este
/// teste **reprova** e manda mover as fixtures para o gate de cima. *Uma dívida
/// sem censo de obsolescência vira licença.*
#[test]
fn as_superficies_curvas_ficam_abertas_com_o_numero() {
    // ⚠️ **Um tecto por LISTA, e não um só**, porque os dois restos têm tamanhos
    // muito diferentes depois de a pegada congelar: um tecto único (o do pior)
    // deixaria o polegar com duas ordens de grandeza de folga — e uma folga é um
    // ponto cego. Os dois são o pior MEDIDO em 2026-09-13 com folga de ~2×.
    println!("  fixture                           |   desvio (ABERTO)");
    for (nome, tecto) in POLEGAR_CURVO
        .iter()
        .map(|n| (n, 5e-4f32))
        .chain(EMPURRAO_CURVO.iter().map(|n| (n, 5e-2f32)))
    {
        let t = traco(nome);
        let d = desvio(&t, &correr(&t));
        println!("  {nome:<33} | {d:10.3e}");
        assert!(
            d <= tecto,
            "{nome}: piorou para {d:.3e} (tecto {tecto:.0e})"
        );
        assert!(
            d > TOL,
            "{nome}: o resíduo caiu para {d:.3e} — este item FECHOU. \
             Mova a fixture para o gate de paridade e apague-a desta lista."
        );
    }
}

/// ⭐⭐ **A FORÇA ENTRA AO QUADRADO — medido no NOSSO motor.**
///
/// ⚠️ **Este gate não compara com a fixture: ele compara duas corridas nossas.**
/// É o controlo que separa *"reproduzimos o número dele"* de *"temos a lei
/// dele"* — um motor que multiplicasse a força uma vez só passaria nas fixtures
/// de força `1,0` (onde `s² = s`) e falharia aqui.
#[test]
fn a_forca_entra_ao_quadrado_nos_dois_gestos() {
    for (cheia, meia) in [
        ("polegar_plano_origem", "polegar_plano_forca05"),
        ("empurrao_plano_origem", "empurrao_plano_forca05"),
    ] {
        let (a, b) = (traco(cheia), traco(meia));
        assert_eq!(a.f("forca"), 1.0, "{cheia}: forca do cabecalho");
        assert_eq!(b.f("forca"), 0.5, "{meia}: forca do cabecalho");
        let nosso = pico_nosso(&b, &correr(&b)) / pico_nosso(&a, &correr(&a));
        // ⚠️⚠️ **A razão do ORÁCULO é a régua, e ela NÃO é `0,25` nos dois.**
        // No gesto ancorado é (o alvo é linear no puxão total); no que VIAJA o
        // transporte é uma integral de linha, e meia força não dá um quarto do
        // pico — dá `0,174`. *Escrever `0,25` para os dois seria fabricar a
        // régua*, e foi o que a primeira versão deste gate fez.
        let dele = b.f("max_deslocamento") / a.f("max_deslocamento");
        println!("  {meia}: nosso {nosso:.6} · dele {dele:.6}");
        assert!(
            (nosso - dele).abs() <= 1e-3,
            "{meia}: a nossa razao e' {nosso:.6} e a dele {dele:.6}"
        );
    }
    // ⭐ E o valor ABSOLUTO, onde ele é afirmável: no gesto ancorado meia força
    // dá **um quarto**, que é a assinatura do quadrado. Um motor linear daria
    // `0,5`, e um que multiplicasse a força duas vezes daria `0,0625` — os dois
    // números que esta bancada já leu antes de a lei ficar certa.
    let cheio = traco("polegar_plano_origem");
    let meio = traco("polegar_plano_forca05");
    let razao = pico_nosso(&meio, &correr(&meio)) / pico_nosso(&cheio, &correr(&cheio));
    assert!(
        (razao - 0.25).abs() <= 1e-3,
        "o polegar a meia forca deveria dar um quarto, e deu {razao:.6}"
    );
}

/// ⭐⭐ **O ANCORADO É IDEMPOTENTE E O QUE VIAJA NÃO — medido no NOSSO motor.**
///
/// A propriedade mais importante da espec (§4), e a que separa as duas famílias:
/// o mesmo caminho com o dobro dos eventos dá o MESMO resultado num gesto e um
/// resultado DIFERENTE no outro.
#[test]
fn so_o_gesto_ancorado_ignora_quantos_eventos_houve() {
    let p12 = pico_nosso(
        &traco("polegar_plano_origem"),
        &correr(&traco("polegar_plano_origem")),
    );
    let p24 = pico_nosso(
        &traco("polegar_plano_passos24"),
        &correr(&traco("polegar_plano_passos24")),
    );
    println!("  polegar : 12 eventos {p12:.6} · 24 eventos {p24:.6}");
    assert!(
        (p12 - p24).abs() <= 1e-6,
        "o polegar deveria ignorar a amostragem: {p12:.6} contra {p24:.6}"
    );

    let e12 = pico_nosso(
        &traco("empurrao_plano_origem"),
        &correr(&traco("empurrao_plano_origem")),
    );
    let e24 = pico_nosso(
        &traco("empurrao_plano_passos24"),
        &correr(&traco("empurrao_plano_passos24")),
    );
    println!("  empurrao: 12 eventos {e12:.6} · 24 eventos {e24:.6}");
    assert!(
        (e12 - e24).abs() > 1e-4,
        "o empurrao deveria DEPENDER da amostragem: {e12:.6} contra {e24:.6}"
    );
}

/// ⭐ **O KNOB DA NORMAL ESTÁ VIVO — e este gate existe porque uma MUTAÇÃO
/// SOBREVIVEU.**
///
/// ⚠️⚠️ **A paridade sozinha não o alcançava:** num PLANO a normal é a mesma
/// seja qual for o raio da amostragem, e as duas fixtures curvas que movem o
/// knob (`…raionormal03`) estão na lista ABERTA, cujo tecto é largo. ⇒ apagar a
/// fracção do raio (ler sempre a pegada inteira) deixava as sete fixtures planas
/// **verdes**. *Um corpus que não varia um knob não testa esse knob* — e a
/// mutação é a única coisa que diz isso.
///
/// A lei que ele afirma é a mínima e a honesta: **numa superfície curva, mudar a
/// fracção MUDA o resultado**, muito acima da barra de paridade. Sobre o que o
/// valor certo é, quem responde são as fixtures.
#[test]
fn a_fraccao_do_raio_da_normal_muda_o_resultado_numa_superficie_curva() {
    let t = traco("polegar_esfera_topo");
    let correr_com = |frac: f32| {
        let mut mesh = superficie(t.s("superficie"));
        let mut brush = pincel(&t);
        brush.normal_radius_frac = frac;
        let eye = olho(t.s("vista"));
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 1..t.caminho.len() {
            let puxao = no_plano_do_ecra(menos(t.caminho[k], t.caminho[0]), eye);
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::pulling(t.caminho[0], brush.radius, eye, puxao),
                Symmetry::default(),
            );
        }
        mesh
    };
    let (a, b) = (correr_com(0.5), correr_com(1.0));
    let mut pior = 0.0f32;
    for i in 0..a.vert_count() {
        let d = menos(a.positions()[i], b.positions()[i]);
        pior = pior.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    println!("  fracção 0,5 contra 1,0: maior diferença {pior:.3e} (barra {TOL:.0e})");
    assert!(
        pior > TOL * 100.0,
        "o knob da fracção do raio da normal é INERTE nesta superfície: {pior:.3e}"
    );
}

/// ⭐⭐⭐ **O AGARRAR QUE JÁ SHIPÁVAMOS, AGORA COM LADO APROVADO.**
///
/// ⚠️ **Estas fixtures são do [`Verb::Move`], que não é desta wave** — elas
/// entram porque o oráculo as gravou e porque ninguém as tinha corrido: o
/// agarrar existe há meses e **nunca** tinha sido comparado com a referência.
/// Medido, ele fecha à força cheia dentro do ruído de `f32`.
///
/// ⛔⛔ **E foi assim que se achou a divergência da CURVA DA FORÇA** (a cura
/// vive em `ref_profiles::blender_strength_curve`): o `B` elevava o slider ao
/// quadrado em **todos** os verbos, e a referência fá-lo por verbo. Ver
/// [`a_forca_do_agarrar_e_linear_e_a_do_polegar_nao`].
///
/// ⚠️ **As duas fixtures com a âncora em VÉRTICE ficam de fora** — essa opção
/// não está implementada (espec §7.2), e compará-las hoje mediria a ausência
/// dela, não a lei.
#[test]
fn o_agarrar_reproduz_o_oraculo() {
    println!("  fixture                           |   desvio   | pico nosso | pico dele");
    let mut pior = (0.0f32, "");
    for nome in [
        "agarrar_plano_alvo_geometria",
        "agarrar_plano_silhueta_nao",
        "agarrar_grelha8_vertativo_nao",
    ] {
        let t = traco(nome);
        assert_eq!(t.f("vertice_activo"), 0.0, "{nome}: ancora em vertice");
        assert_eq!(t.f("silhueta"), 0.0, "{nome}: silhueta ligada");
        let malha = correr(&t);
        let d = desvio(&t, &malha);
        println!(
            "  {nome:<33} | {d:10.3e} | {:10.6} | {:9.6}",
            pico_nosso(&t, &malha),
            t.f("max_deslocamento")
        );
        if d > pior.0 {
            pior = (d, nome);
        }
    }
    assert!(
        pior.0 <= TOL,
        "{}: desvio {:.3e} acima da barra {TOL:.0e}",
        pior.1,
        pior.0
    );
}

/// ⭐⭐ **A CURVA DA FORÇA É POR VERBO — o agarrar é LINEAR e o polegar é
/// QUADRÁTICO, no mesmo modo e no mesmo slider.**
///
/// ⚠️ **A régua é a RAZÃO, e não o valor absoluto**, porque a única fixture do
/// agarrar a meia força traz também a âncora em vértice (que não temos). A
/// razão é imune a isso: o oráculo mede `0,200000 / 0,500001 = 0,400`, ou seja
/// **linear**; um motor quadrático daria `0,16`. E ao lado, o polegar dá `0,25`
/// sobre `0,5` — *o mesmo slider, duas leis, e é isso que esta casa tinha
/// escrito como uma só.*
#[test]
fn a_forca_do_agarrar_e_linear_e_a_do_polegar_nao() {
    let razao_de = |nome: &str, forca: f32| {
        let t = traco(nome);
        let cheio = pico_nosso(&t, &correr(&t));
        let mut brush = pincel(&t);
        brush.strength = forca;
        let mut mesh = superficie(t.s("superficie"));
        let eye = olho(t.s("vista"));
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 1..t.caminho.len() {
            let (centro, puxao) = if matches!(brush.verb.grip(), Grip::Hold) {
                (t.caminho[0], menos(t.caminho[k], t.caminho[0]))
            } else {
                (t.caminho[k], menos(t.caminho[k], t.caminho[k - 1]))
            };
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::pulling(centro, brush.radius, eye, no_plano_do_ecra(puxao, eye)),
                Symmetry::default(),
            );
        }
        let base = repouso(t.s("superficie"));
        let mut meio = 0.0f32;
        for i in 0..base.len() {
            let d = menos(mesh.positions()[i], base[i]);
            meio = meio.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
        meio / cheio
    };
    let agarrar = razao_de("agarrar_plano_alvo_geometria", 0.4);
    let polegar = razao_de("polegar_plano_origem", 0.5);
    println!("  agarrar a 0,4 da forca: razao {agarrar:.6} (linear = 0,400)");
    println!("  polegar a 0,5 da forca: razao {polegar:.6} (quadratica = 0,250)");
    assert!(
        (agarrar - 0.4).abs() <= 1e-3,
        "o agarrar deveria ser LINEAR no slider: razao {agarrar:.6} (quadratico daria 0,16)"
    );
    assert!(
        (polegar - 0.25).abs() <= 1e-3,
        "o polegar deveria ser QUADRATICO: razao {polegar:.6}"
    );
}

/// 🔬 **SONDA — o AGARRAR contra o oráculo, com as colunas todas.**
#[test]
#[ignore = "sonda"]
fn sonda_do_agarrar_que_ja_shipamos() {
    println!("  fixture                           |   desvio   | pico nosso | pico dele | forca");
    for nome in [
        "agarrar_plano_alvo_geometria",
        "agarrar_plano_silhueta_nao",
        "agarrar_grelha8_vertativo_nao",
        "agarrar_grelha8_vertativo_sim_forca04",
    ] {
        let t = traco(nome);
        let malha = correr(&t);
        println!(
            "  {nome:<33} | {:10.3e} | {:10.6} | {:9.6} | {:.2}",
            desvio(&t, &malha),
            pico_nosso(&t, &malha),
            t.f("max_deslocamento"),
            t.f("forca")
        );
    }
    println!("  ⇒ se a linha de forca 0,4 divergir ~2,5x e as de forca 1,0 fecharem,");
    println!("    a curva de forca do modo B e' por MODO onde a referencia a tem por VERBO.");

    // ⭐ O discriminador: o MESMO puxão total, entregue em N eventos.
    println!();
    println!("  o mesmo puxao total (0,6) em N eventos:");
    let t = traco("agarrar_plano_alvo_geometria");
    for n in [1usize, 2, 4, 12] {
        let mut mesh = superficie(t.s("superficie"));
        let brush = pincel(&t);
        let eye = olho(t.s("vista"));
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 1..=n {
            let total = 0.6 * (k as f32 / n as f32);
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::pulling(t.caminho[0], brush.radius, eye, [total, 0.0, 0.0]),
                Symmetry::default(),
            );
        }
        let base = repouso(t.s("superficie"));
        let mut pico = 0.0f32;
        for i in 0..base.len() {
            let d = menos(mesh.positions()[i], base[i]);
            pico = pico.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
        println!("    N = {n:>2}: pico {pico:.6}");
    }
    println!("  ⇒ se o pico CAIR com N, o gesto ancorado esta' a perder vertices da pegada");
    println!("    a cada evento (a consulta e' na malha VIVA, e o gesto leva-os para fora).");
}

/// 🔬 **SONDA — onde mora o resíduo da esfera:** direcção ou magnitude?
///
/// ⚠️ **A pergunta é essa e não «quanto»**, porque as duas respostas mandam
/// procurar em sítios diferentes: um desvio de DIREÇÃO acusa a normal do gesto
/// (§5.2), um de MAGNITUDE acusa o peso ou a força (§2, §3). *Medir o ângulo
/// custa quatro linhas e poupa uma jornada a mexer no sítio errado.*
#[test]
#[ignore = "sonda"]
fn sonda_do_residuo_da_esfera() {
    println!(
        "  fixture                           | mov nosso/dele | pico n/d | cos(ang) | |n|/|d|"
    );
    for nome in [
        "polegar_esfera_topo",
        "polegar_esfera_topo_k08",
        "polegar_plano_origem",
    ] {
        let t = traco(nome);
        let malha = correr(&t);
        let base = repouso(t.s("superficie"));
        // O vértice que MAIS andou, do nosso lado e do lado dele.
        let (mut nosso_i, mut nosso_d) = (0usize, 0.0f32);
        let (mut dele_i, mut dele_d) = (0usize, 0.0f32);
        let (mut mov_n, mut mov_d) = (0usize, 0usize);
        let norma = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        for i in 0..base.len() {
            let dn = norma(menos(malha.positions()[i], base[i]));
            let dd = norma(menos(t.depois[i], base[i]));
            if dn > 1e-9 {
                mov_n += 1;
            }
            if dd > 1e-9 {
                mov_d += 1;
            }
            if dn > nosso_d {
                (nosso_i, nosso_d) = (i, dn);
            }
            if dd > dele_d {
                (dele_i, dele_d) = (i, dd);
            }
        }
        // No MESMO vértice (o pico dele), o ângulo entre os dois deslocamentos.
        let a = menos(malha.positions()[dele_i], base[dele_i]);
        let b = menos(t.depois[dele_i], base[dele_i]);
        let cos = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (norma(a) * norma(b)).max(1e-30);
        println!(
            "  {nome:<33} | {mov_n:6}/{mov_d:<7} | {}{:.5}/{:.5} | {cos:.6} | {:.6}",
            if nosso_i == dele_i { " " } else { "*" },
            nosso_d,
            dele_d,
            norma(a) / norma(b).max(1e-30)
        );
    }
    println!("  * = o pico caiu em vertices DIFERENTES");
    println!("  cos < 1 ⇒ a DIREÇÃO diverge (a normal do gesto, §5.2)");
    println!("  |n|/|d| ≠ 1 com cos = 1 ⇒ a MAGNITUDE diverge (o peso ou a forca)");
}

/// ⛔ **O piso de população** — uma lista escrita à mão que aponte para um
/// ficheiro que deixou de existir mede ZERO e passa verde.
#[test]
fn as_fixturas_desta_bancada_existem_todas() {
    for nome in POLEGAR
        .iter()
        .chain(POLEGAR_CURVO.iter())
        .chain(EMPURRAO_PLANO.iter())
        .chain(EMPURRAO_CURVO.iter())
    {
        let caminho = fixture_dir().join(format!("{nome}.deformado.txt.gz"));
        assert!(caminho.exists(), "fixture ausente: {}", caminho.display());
    }
    assert_eq!(
        POLEGAR.len() + POLEGAR_CURVO.len() + EMPURRAO_PLANO.len() + EMPURRAO_CURVO.len(),
        27
    );
}
