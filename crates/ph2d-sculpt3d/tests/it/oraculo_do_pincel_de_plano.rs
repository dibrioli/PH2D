//! ⭐⭐⭐ **A BANCADA DO PINCEL DE PLANO** — os gates da
//! `docs/3D/cleanroom/SPEC_pincel_de_plano.md` §12, contra as **100** fixturas
//! do oráculo.
//!
//! ⚠️ **O corpus é DADO, não código:** `gzip` de texto, cabeçalho de
//! proveniência e quatro blocos (`r` repouso · `n` normais · `s` saída · `c`
//! cursores, mais `m` onde há máscara). Quem o regenera é o subagente-E — a zona
//! dele é negada a esta janela.
//!
//! ⚠️⚠️ **O enquadramento sai do CABEÇALHO, nunca desta prosa.** A espec §11
//! pagou isto: um censo achou um grupo de fixturas com enquadramento publicado
//! **byte-idêntico e três saídas diferentes**, porque duas grandezas não
//! viajavam. Hoje viajam — e um leitor que siga o cabeçalho reproduz a saída sem
//! ler uma linha de prosa.

use ph2d_mesh::{Face, Mesh};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn pasta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/3D/cleanroom/fixtures/pincel_de_plano")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da do `ph2d-cloth`: o
/// cabeçalho do gzip tem tamanho variável e o rabo são oito bytes.
fn inflar(rel: &str) -> String {
    let p = pasta().join(rel);
    let raw = std::fs::read(&p).unwrap_or_else(|e| panic!("{rel}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{rel}: nao e' gzip"
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
        .unwrap_or_else(|e| panic!("{rel}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

/// Uma fixtura lida: o cabeçalho e os quatro blocos.
pub struct Fixtura {
    pub nome: String,
    pub cab: BTreeMap<String, String>,
    pub repouso: Vec<[f32; 3]>,
    pub normais: Vec<[f32; 3]>,
    pub saida: Vec<[f32; 3]>,
    pub cursores: Vec<[f32; 3]>,
    pub mascara: Vec<f32>,
}

impl Fixtura {
    /// O valor de uma chave do cabeçalho — ⛔ **`panic` e não um default**: uma
    /// chave ausente é a fixtura a não enquadrar o traço, e um default aqui
    /// mediria outro programa em silêncio.
    pub fn chave(&self, k: &str) -> &str {
        self.cab
            .get(k)
            .unwrap_or_else(|| panic!("{}: falta a chave `{k}` no cabecalho", self.nome))
    }

    pub fn num(&self, k: &str) -> f32 {
        self.chave(k)
            .parse()
            .unwrap_or_else(|e| panic!("{}: `{k}` nao e' numero: {e}", self.nome))
    }

    pub fn bool(&self, k: &str) -> bool {
        match self.chave(k) {
            "True" | "true" | "sim" => true,
            "False" | "false" | "nao" => false,
            outro => panic!("{}: `{k}` = `{outro}` nao e' booleano", self.nome),
        }
    }
}

fn v3(c: &[&str]) -> [f32; 3] {
    [
        c[0].parse().expect("x"),
        c[1].parse().expect("y"),
        c[2].parse().expect("z"),
    ]
}

pub fn ler(familia: &str, nome: &str) -> Fixtura {
    let texto = inflar(&format!("{familia}/{nome}.txt.gz"));
    let mut f = Fixtura {
        nome: nome.to_string(),
        cab: BTreeMap::new(),
        repouso: Vec::new(),
        normais: Vec::new(),
        saida: Vec::new(),
        cursores: Vec::new(),
        mascara: Vec::new(),
    };
    for l in texto.lines() {
        if let Some(resto) = l.strip_prefix("# ") {
            if let Some((k, v)) = resto.split_once(": ") {
                f.cab.insert(k.trim().to_string(), v.trim().to_string());
            }
            continue;
        }
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos.first().copied() {
            Some("r") => f.repouso.push(v3(&campos[1..])),
            Some("n") => f.normais.push(v3(&campos[1..])),
            Some("s") => f.saida.push(v3(&campos[1..])),
            Some("c") => f.cursores.push(v3(&campos[1..])),
            Some("m") => f.mascara.push(campos[1].parse().expect("mascara")),
            _ => {}
        }
    }
    let n: usize = f.chave("vertices").parse().expect("vertices");
    assert_eq!(f.repouso.len(), n, "{nome}: bloco `r` nao bate o cabecalho");
    assert_eq!(f.normais.len(), n, "{nome}: bloco `n` nao bate o cabecalho");
    assert_eq!(f.saida.len(), n, "{nome}: bloco `s` nao bate o cabecalho");
    f
}

/// **A TOPOLOGIA da malha de entrada**, derivada da contagem de vértices.
///
/// ⚠️ **O corpus não publica faces**, e não precisa: o README dele declara que as
/// malhas são **grelhas de quadriláteros** geradas pelo nosso harness, com `x` a
/// variar primeiro. ⇒ `2 401 = 49 × 49`, e os quadriláteros saem da grelha.
///
/// ⛔ **A raiz é CONFERIDA e não suposta:** uma contagem que não seja um quadrado
/// perfeito é outra família de malha (a esfera do corpus), e adivinhar-lhe a
/// topologia daria um gate a medir uma superfície que não é a da fixtura.
/// ⭐⭐ **A MESMA grelha, DESLOCADA no espaço** — a fixtura que faz o G-6
/// discriminar.
///
/// ⛔⛔ **Ela existe porque uma MUTAÇÃO SOBREVIVEU:** ancorar o plano na ORIGEM
/// (`fit.point = [0,0,0]`) deixava o `o_plano_segue_o_cursor_e_nao_a_origem`
/// **verde** — o corpus inteiro vive perto da origem, logo *«segue o cursor»* e
/// *«está na origem»* dão o mesmo número ali. *Uma fixtura que não contém o
/// fenómeno não afirma nada sobre ele*, e o nome do gate prometia exactamente
/// esse fenómeno.
///
/// ⚠️ **O deslocamento é GRANDE de propósito** (`3` unidades, contra uma peça de
/// extensão `~1`): ele tem de ser muito maior que a barra em raios, senão a
/// metade discriminante mede ruído.
pub fn grelha_deslocada(f: &Fixtura, d: [f32; 3]) -> Option<Mesh> {
    let g = grelha(f)?;
    let pos: Vec<[f32; 3]> = g
        .positions()
        .iter()
        .map(|p| [p[0] + d[0], p[1] + d[1], p[2] + d[2]])
        .collect();
    Mesh::from_parts(pos, g.faces().to_vec()).ok()
}

pub fn grelha(f: &Fixtura) -> Option<Mesh> {
    let n = f.repouso.len();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lado = (n as f64).sqrt().round() as usize;
    if lado * lado != n {
        return None;
    }
    let mut faces = Vec::with_capacity((lado - 1) * (lado - 1));
    for j in 0..lado - 1 {
        for i in 0..lado - 1 {
            let a = u32::try_from(j * lado + i).expect("indice");
            let b = a + 1;
            let c = b + u32::try_from(lado).expect("lado");
            let d = a + u32::try_from(lado).expect("lado");
            faces.push(Face::quad(a, b, c, d));
        }
    }
    Some(Mesh::from_parts(f.repouso.clone(), faces).expect("grelha"))
}

/// **O PINCEL que o cabeçalho da fixtura descreve.**
///
/// ⚠️⚠️ **Todo knob sai do CABEÇALHO, e as chaves que faltam FAZEM PANIC** — é a
/// única forma de um leitor de 100 fixturas não medir outro programa em silêncio.
/// A espec §11 pagou esta lição: um censo achou fixturas com enquadramento
/// publicado idêntico e três saídas diferentes, porque duas grandezas não
/// viajavam.
pub fn pincel(f: &Fixtura) -> ph2d_sculpt3d::Brush {
    use ph2d_sculpt3d::{Brush, Falloff, PlanoInversao, Verb};
    assert_eq!(f.chave("pincel"), "PLANE", "{}: outro pincel", f.nome);
    let curva = match f.chave("curva") {
        "CONSTANT" => Falloff::Constant,
        "SMOOTH" => Falloff::Smooth,
        outra => panic!("{}: curva `{outra}` sem tradução", f.nome),
    };
    let inversao = match f.chave("modo_de_inversao") {
        // ⚠️ Os dois valores são da API **pública** do alvo, que é a chave de
        // regeneração das fixturas — a espec §4.1.13 cobre-os expressamente.
        "INVERT_DISPLACEMENT" => PlanoInversao::Afastar,
        "SWAP_DEPTH_AND_HEIGHT" => PlanoInversao::TrocarTectos,
        outro => panic!("{}: modo de inversão `{outro}` sem tradução", f.nome),
    };
    Brush {
        verb: Verb::Plane,
        falloff: curva,
        radius: f.num("raio_objeto"),
        strength: f.num("forca"),
        hardness: f.num("dureza"),
        accumulate: f.bool("acumula"),
        front_faces_only: f.bool("so_faces_de_frente"),
        normal_radius_frac: f.num("raio_da_normal"),
        area_radius_frac: f.num("raio_da_area"),
        plane_offset: f.num("deslocamento_do_plano"),
        plano_altura: f.num("altura"),
        plano_profundidade: f.num("profundidade"),
        plano_inversao: inversao,
        invert: f.bool("modificador_carregado"),
        plano_firmeza_normal: f.num("firmeza_da_normal"),
        plano_firmeza_centro: f.num("firmeza_do_centro"),
        ..Brush::default()
    }
}

/// **A DIRECÇÃO em que o raio de picagem viaja**, do cabeçalho.
///
/// ⚠️ **NEGADA, e a escolha está NOMEADA porque o corpus não a observa:** a chave
/// publica a direcção da vista no sentido em que ela aponta **para quem olha**, e
/// o nosso [`ph2d_sculpt3d::Dab::eye`] aponta **para dentro** da cena. ⛔ Em todas
/// as fixturas de grelha a escolha é **inobservável** — a superfície inteira olha
/// para o mesmo lado, logo os dois baldes da §2.5 não se separam e o balde de
/// fallback dá o mesmo plano. *Uma convenção que o corpus não distingue fica
/// escrita como convenção, não como facto medido.*
pub fn olho(f: &Fixtura) -> [f32; 3] {
    let c: Vec<&str> = f.chave("direccao_da_vista").split_whitespace().collect();
    let v = v3(&c);
    [-v[0], -v[1], -v[2]]
}

/// **CORRE o traço da fixtura** e devolve as posições finais.
///
/// ⭐ **Pelo caminho do PRODUTO** (`SculptStroke::dab`), nunca por uma cópia da
/// lei: uma bancada que reimplementa o que mede afirma que a cópia concorda com o
/// oráculo, e é exactamente o defeito que esta casa nomeia como *«a régua
/// partilhava a lei do produto»*.
pub fn correr(f: &Fixtura) -> Vec<[f32; 3]> {
    use ph2d_sculpt3d::{Dab, SculptStroke, Symmetry};
    let mut m = grelha(f).expect("grelha");
    let b = pincel(f);
    let e = olho(f);
    let mut s = SculptStroke::default();
    s.begin(&m);
    for c in &f.cursores {
        s.dab(&mut m, &b, &Dab::at(*c, b.radius, e), Symmetry::default());
    }
    m.positions().to_vec()
}

/// O pior desvio entre a nossa saída e a do oráculo, e quantos vértices cada lado
/// moveu.
pub fn comparar(f: &Fixtura, nossa: &[[f32; 3]]) -> (f32, usize, usize) {
    let mut pior = 0.0f32;
    let (mut nossos, mut dele) = (0usize, 0usize);
    for (i, n) in nossa.iter().enumerate() {
        let d = (0..3)
            .map(|k| (n[k] - f.saida[i][k]).abs())
            .fold(0.0f32, f32::max);
        pior = pior.max(d);
        if (0..3).any(|k| n[k] != f.repouso[i][k]) {
            nossos += 1;
        }
        if (0..3).any(|k| f.saida[i][k] != f.repouso[i][k]) {
            dele += 1;
        }
    }
    (pior, nossos, dele)
}

/// ⭐⭐ **A PRIMEIRA MEDIÇÃO desta bancada, e ela decide o DESENHO dos gates: as
/// nossas normais de vértice concordam com as do oráculo?**
///
/// A lei do plano (espec §2.1) come **normais de vértice**. Se as nossas
/// divergirem das dele, a paridade da lei falha por uma razão que não é a lei —
/// e o gate estaria a medir o nosso cálculo de normais.
///
/// ⛔ **É por isso que esta medição vem antes de qualquer barra:** *uma régua que
/// mistura duas leis não afirma nada sobre nenhuma delas.*
#[test]
fn as_nossas_normais_de_vertice_concordam_com_as_do_oraculo() {
    let mut pior = 0.0f32;
    let mut pior_nome = String::new();
    let mut medidas = 0usize;
    for nome in [
        "lei_crista",
        "lei_rampa",
        "lei_degrau",
        "lei_bossas",
        "lei_primeiro_dab",
    ] {
        let f = ler("lei", nome);
        let Some(m) = grelha(&f) else { continue };
        medidas += 1;
        let mut pior_desta = 0.0f32;
        let mut pior_perto = 0.0f32;
        let raio = f.num("raio_objeto");
        let cur = f.cursores[0];
        for (i, nossa) in m.normals().iter().enumerate() {
            let dele = f.normais[i];
            let cos =
                (nossa[0] * dele[0] + nossa[1] * dele[1] + nossa[2] * dele[2]).clamp(-1.0, 1.0);
            let g = cos.acos().to_degrees();
            pior_desta = pior_desta.max(g);
            let p = f.repouso[i];
            let d = ((p[0] - cur[0]).powi(2) + (p[1] - cur[1]).powi(2) + (p[2] - cur[2]).powi(2))
                .sqrt();
            if d <= raio * 2.0 {
                pior_perto = pior_perto.max(g);
            }
        }
        println!(
            "  {nome:22} malha inteira {pior_desta:.8}°  |  a <=2R do cursor {pior_perto:.8}°"
        );
        for (i, nossa) in m.normals().iter().enumerate() {
            let dele = f.normais[i];
            let cos =
                (nossa[0] * dele[0] + nossa[1] * dele[1] + nossa[2] * dele[2]).clamp(-1.0, 1.0);
            let graus = cos.acos().to_degrees();
            if graus > pior {
                pior = graus;
                pior_nome = format!("{nome}[{i}]");
            }
        }
    }
    assert!(medidas >= 5, "so' {medidas} fixturas de grelha foram lidas");
    println!("pior desvio angular: {pior:.6}° em {pior_nome} ({medidas} fixturas)");
    assert!(
        pior < 1.0,
        "as nossas normais desviam {pior:.4}° das do oraculo (pior: {pior_nome}) — \
         a paridade da LEI nao pode ser medida sobre normais que discordam"
    );
}

/// **As 19 fixturas de UM dab efectivo** — a população do G-1, contada do
/// directório e partida pela espec §12.
///
/// ⚠️ **As `28` das duas pastas menos as `2` que não movem nada** (sujeitos do
/// G-2 e do G-8, e não há reprodução a fazer sobre uma malha intocada) **menos os
/// `7` traços inteiros** de 8 dabs, que esta reprodução não alcança por reproduzir
/// só o primeiro dab efectivo. ⇒ `19 = 13 + 6`.
const UM_DAB: [(&str, &str); 19] = [
    ("lei", "lei_crista"),
    ("lei", "lei_rampa"),
    ("lei", "lei_degrau"),
    ("lei", "lei_bossas"),
    ("lei", "lei_crista_raio02"),
    ("lei", "lei_crista_raio06"),
    ("lei", "lei_area025"),
    ("lei", "lei_area10"),
    ("lei", "lei_area20"),
    ("lei", "lei_normal025"),
    ("lei", "lei_normal10"),
    ("lei", "lei_deslocado_p02"),
    ("lei", "lei_deslocado_m02"),
    ("lados", "lados_altura02"),
    ("lados", "lados_altura05"),
    ("lados", "lados_altura08"),
    ("lados", "lados_ambos"),
    ("lados", "lados_so_abaixo"),
    ("lados", "lados_so_acima"),
];

/// **A barra de paridade, DERIVADA** (espec §12): `1e-6` em unidades de objecto é
/// `8,4` ULP de `f32` na maior coordenada das fixturas (`1,0`) e `34` na escala do
/// raio — e **`12×`** o pior resíduo que a espec mediu (`8,0e-08`).
///
/// ⛔ **Não é um epsilon de conforto:** é uma casa de ULP no formato em que o
/// oráculo respondeu.
const BARRA: f32 = 1e-6;

/// ⭐⭐⭐ **G-1 — A LEI DO PLANO REPRODUZ O ORÁCULO**, pelo caminho do produto,
/// sobre as `19` fixturas de um dab efectivo.
///
/// ⭐⭐ **E as `19` entram na barra COMUM — a espec previu-o e a medição
/// confirmou.** Ela dava barra própria (`1e-4`) à `lei_bossas`, porque a
/// reprodução independente do subagente-E lia lá `3,2e-05` sobre uma malha de
/// quadriláteros **empenados**, onde *a própria definição da normal de um vértice
/// é uma escolha a montante desta lei*. Medido nesta implementação:
/// **`7,451e-08`**, dentro da barra comum — e a espec §12 escreve, com todas as
/// letras, que *«quem implementar deve ver as 19 na barra comum, e o gate diz isso
/// de si mesmo»*. ⇒ **é isto que ele diz.**
///
/// | fixtura | `max │Δ│` |
/// |---|---|
/// | as 18 restantes | `1,49e-08 … 8,94e-08` |
/// | `lei_bossas` (a que a espec isentava) | **`7,45e-08`** |
///
/// ⛔⛔ **E uma célula do corpus apanhou um defeito REAL antes de este gate
/// existir:** a `lei_area20` (`R_c = 2 R`) desviava **`1,703e-01`** com `279`
/// vértices movidos contra `276`, porque a consulta não cobria a EXTENSÃO DE
/// AMOSTRAGEM — ver [`ph2d_sculpt3d::Footprint::tectos_query_factor`]. *Sem essa
/// célula o defeito seria mudo: `279` contra `276` lê-se como ruído.*
#[test]
fn a_lei_do_plano_reproduz_o_oraculo() {
    let mut pior = (0.0f32, String::new());
    let mut medidas = 0usize;
    for (familia, nome) in UM_DAB {
        let f = ler(familia, nome);
        assert!(
            grelha(&f).is_some(),
            "{nome}: a malha nao e' uma grelha — o G-1 mede grelhas"
        );
        let nossa = correr(&f);
        let (d, nossos, dele) = comparar(&f, &nossa);
        assert_eq!(
            nossos, dele,
            "{nome}: movemos {nossos} vertices e o oraculo moveu {dele} — \
             a POPULACAO tocada tem de bater antes de a posicao importar"
        );
        assert!(
            d <= BARRA,
            "{nome}: max|Δ| {d:.3e} passa a barra {BARRA:.0e} ({nossos} vertices)"
        );
        if d > pior.0 {
            pior = (d, nome.to_string());
        }
        medidas += 1;
    }
    // ⭐ **O PISO DE POPULAÇÃO** — sem ele, uma lista que encolhesse deixaria o
    // gate verde a medir quase nada, que é a forma que o `CLAUDE.md` §5.0 nomeia.
    assert_eq!(
        medidas, 19,
        "o G-1 correu {medidas} fixturas e a populacao declarada e' 19"
    );
    println!("G-1: 19 fixturas, pior {:.3e} em {}", pior.0, pior.1);
}

/// ⭐⭐ **G-2 — O PRIMEIRO DAB NÃO MOVE NADA** (espec §1).
///
/// Um traço de **um** dab deixa a malha **byte-idêntica**: o quadro local nasce da
/// direcção do traço, e no primeiro dab ela não existe. ⛔ Igualdade exacta, e não
/// uma barra: o alvo mede `0` de `2 401`, e *«quase nada» é outra afirmação*.
#[test]
fn o_primeiro_dab_nao_move_nada() {
    let f = ler("lei", "lei_primeiro_dab");
    assert_eq!(
        f.cursores.len(),
        1,
        "a fixtura do primeiro dab tem UM cursor"
    );
    let nossa = correr(&f);
    assert_eq!(nossa, f.repouso, "o primeiro dab moveu barro");
    assert_eq!(
        nossa, f.saida,
        "o oraculo e nos discordamos no primeiro dab"
    );
}

/// ⭐⭐⭐ **G-7 — OS DOIS TECTOS ESCOLHEM O LADO, E SÓ O LADO** (espec §3.1).
///
/// ⚠️ **A régua é uma CONTAGEM e um SINAL, não um peso:** `altura 1 /
/// profundidade 0` toca **só** acima do plano, `0 / 1` **só** abaixo, e `1 / 1`
/// toca a **união exacta** dos dois. É isto que transforma três verbos num só.
#[test]
fn os_dois_tectos_escolhem_o_lado_e_so_o_lado() {
    let mut contas = Vec::new();
    for nome in ["lados_so_acima", "lados_so_abaixo", "lados_ambos"] {
        let f = ler("lados", nome);
        let nossa = correr(&f);
        let (_, nossos, dele) = comparar(&f, &nossa);
        assert_eq!(nossos, dele, "{nome}: populacao");
        contas.push(nossos);
    }
    let (acima, abaixo, ambos) = (contas[0], contas[1], contas[2]);
    assert!(acima > 0 && abaixo > 0, "um dos lados nao toca nada");
    assert_eq!(
        ambos,
        acima + abaixo,
        "os dois lados juntos ({ambos}) nao sao a UNIAO exacta de {acima} + {abaixo} — \
         se houver interseccao ou buraco, a silhueta nao e' um par de meios-elipsoides"
    );
    println!("G-7: acima {acima} + abaixo {abaixo} = ambos {ambos}");
}

/// ⭐⭐ **G-8 — O PINCEL INERTE É ALCANÇÁVEL** (espec §3.1).
///
/// `altura 0` **e** `profundidade 0` ⇒ zero vértices movidos. ⚠️ É o *nada* do
/// controlo, e ele é **alcançável**: um artista que baixe os dois tem um pincel
/// que não faz nada em vez de um que faz metade.
#[test]
fn altura_zero_e_profundidade_zero_e_um_no_op() {
    let f = ler("lados", "traco_inerte");
    assert_eq!(f.num("altura"), 0.0, "a fixtura inerte tem altura zero");
    assert_eq!(f.num("profundidade"), 0.0, "e profundidade zero");
    let nossa = correr(&f);
    assert_eq!(nossa, f.repouso, "o pincel de tectos zero moveu barro");
    assert_eq!(nossa, f.saida, "o oraculo e nos discordamos no inerte");
}

/// ⭐⭐⭐ **G-9 — A INVERSÃO POR TROCA É O PAR TROCADO** (espec §5).
///
/// ⚠️ **Igualdade EXACTA, e é a medição mais limpa da espec:** inverter um pincel
/// de `altura 1 / profundidade 0` no modo *trocar* dá a saída do **mesmo pincel
/// não invertido** com `altura 0 / profundidade 1`. ⇒ *não é «parecido com trocar
/// os tectos»: é trocar os tectos.*
///
/// ⭐ E a metade simétrica: com `altura = profundidade` o modo *trocar* é um
/// **no-op**, também ao bit.
#[test]
fn a_inversao_por_troca_e_o_par_trocado() {
    use ph2d_sculpt3d::{Dab, SculptStroke, Symmetry};
    let f = ler("inversao", "inversao_troca");
    // O par trocado, construído do MESMO cabeçalho: o que a inversão promete.
    let mut b = pincel(&f);
    assert!(b.invert, "a fixtura da troca tem o modificador carregado");
    let (h, d) = (b.plano_altura, b.plano_profundidade);
    assert_ne!(h, d, "um par simetrico nao distingue trocar de nao trocar");
    b.invert = false;
    b.plano_altura = d;
    b.plano_profundidade = h;
    let mut m = grelha(&f).expect("grelha");
    let mut s = SculptStroke::default();
    s.begin(&m);
    for c in &f.cursores {
        s.dab(
            &mut m,
            &b,
            &Dab::at(*c, b.radius, olho(&f)),
            Symmetry::default(),
        );
    }
    assert_eq!(
        m.positions(),
        correr(&f).as_slice(),
        "inverter no modo TROCAR nao deu o par trocado"
    );
}

/// ⭐⭐ **G-10 — O RAIO DA ÁREA A ZERO CAI NO RAIO DA NORMAL** (espec §2.3).
///
/// ⚠️ **Um PAR, e não um valor:** *um zero que cai noutro knob não se vê num
/// valor — vê-se num par.* Com as duas fracções **iguais**, a saída com a fracção
/// da área a `0` é **byte-idêntica** à com ela a `0,5`; com a fracção da normal em
/// `1,0`, as mesmas duas **divergem**.
#[test]
fn o_raio_da_area_a_zero_cai_no_raio_da_normal() {
    let iguais = (
        correr(&ler("amostragem", "amostragem_area00")),
        correr(&ler("amostragem", "amostragem_area05")),
    );
    assert_eq!(
        iguais.0, iguais.1,
        "com as duas fraccoes iguais, a queda tinha de dar saida IDENTICA"
    );
    let cruzado = (
        correr(&ler("amostragem", "amostragem_area0_normal10")),
        correr(&ler("amostragem", "amostragem_area05_normal10")),
    );
    let dif = cruzado
        .0
        .iter()
        .zip(&cruzado.1)
        .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max))
        .fold(0.0f32, f32::max);
    assert!(
        dif >= 1e-3,
        "com a fraccao da normal em 1,0 as duas tinham de DIVERGIR, e leram {dif:.3e} — \
         sem esta metade o gate passaria com a queda a ignorar a fraccao da normal"
    );
    println!("G-10: identicas na queda, e {dif:.3e} de divergencia no par cruzado");
}

/// ⭐⭐ **G-11 — UMA SUPERFÍCIE JÁ PLANA NÃO SE MEXE** (espec §8) — o auto-limite.
///
/// O plano ajustado **É** a superfície, logo toda distância com sinal é zero e o
/// pincel **pára sozinho quando acabou**. ⛔ Igualdade exacta.
#[test]
fn uma_superficie_ja_plana_nao_se_mexe() {
    let f = ler("superficies", "superficie_lisa");
    let nossa = correr(&f);
    let (_, nossos, dele) = comparar(&f, &nossa);
    assert_eq!(dele, 0, "a fixtura lisa nao e' o auto-limite do oraculo");
    assert_eq!(
        nossos, 0,
        "movemos {nossos} vertices numa superficie ja plana"
    );
}

#[test]
fn sonda_dos_tracos_inteiros() {
    for (fam, nome) in [
        ("lados", "traco_so_acima"),
        ("lados", "traco_so_abaixo"),
        ("lados", "traco_ambos"),
        ("lados", "traco_altura02"),
        ("lados", "traco_altura05"),
        ("lados", "traco_h05_d05"),
        ("lados", "traco_h1_d05"),
        ("inversao", "inversao_afasta"),
        ("inversao", "inversao_troca"),
        ("inversao", "inversao_troca_simetrica"),
        ("cadeia", "cadeia_passo2"),
        ("cadeia", "cadeia_passo4"),
        ("cadeia", "cadeia_passo8"),
        ("cadeia", "cadeia_forca05_passo2"),
        ("cadeia", "cadeia_forca05_passo4"),
        ("cadeia", "cadeia_forca05_passo8"),
        ("superficies", "superficie_degrau"),
        ("superficies", "superficie_rampa"),
        ("superficies", "opcao_dureza05"),
        ("superficies", "opcao_curva_constante"),
        ("superficies", "opcao_acumula"),
        ("superficies", "opcao_so_frente"),
        ("superficies", "superficie_mascara"),
        ("superficies", "opcao_sem_atenuacao"),
        ("superficies", "opcao_pegada_projectada"),
    ] {
        let f = ler(fam, nome);
        if grelha(&f).is_none() {
            println!("  {nome:26} (nao e' grelha)");
            continue;
        }
        let d = f.chave("dabs").to_string();
        let (pior, nossos, dele) = comparar(&f, &correr(&f));
        println!("  {nome:26} dabs {d:>2}  max|Δ| {pior:.3e}  movidos {nossos:5} / {dele:5}");
    }
}

/// ⭐⭐⭐ **G-1b — OS TRAÇOS INTEIROS REPRODUZEM O ORÁCULO**, e isto **paga uma
/// dívida que a espec deixou NOMEADA**.
///
/// A §12 declara o G-1b como *«dívida nomeada, não um gate»*: os traços de `8`
/// dabs estavam publicados e nenhum gate os corria, porque *«reproduzir uma
/// cadeia de dabs exige encadear o estimador de plano dab a dab, que é obra da
/// implementação»*. ⭐ **A implementação existe agora — e a cadeia fecha.**
///
/// ⛔⛔ **E encadear apanhou um defeito que UM dab nunca mostraria:** nesta casa o
/// interruptor de acumular **é** a coluna *de onde a queda mede a distância*, e na
/// lei deste pincel ele decide **de que superfície o plano é lido** — duas
/// perguntas com a mesma palavra. Com as duas na mesma coluna, uma cadeia de `2`
/// dabs (ou seja UM efectivo) reproduzia a `2,980e-08` e as de `4` e `8` desviavam
/// **`1,634e-01`** e **`3,022e-01`**. *O primeiro dab estava certo e o segundo já
/// não* — ver [`ph2d_sculpt3d::Verb::le_a_superficie_viva`].
#[test]
fn os_tracos_inteiros_reproduzem_o_oraculo() {
    let corpus = [
        ("lados", "traco_so_acima"),
        ("lados", "traco_so_abaixo"),
        ("lados", "traco_ambos"),
        ("lados", "traco_altura02"),
        ("lados", "traco_altura05"),
        ("lados", "traco_h05_d05"),
        ("lados", "traco_h1_d05"),
        ("cadeia", "cadeia_passo2"),
        ("cadeia", "cadeia_passo3"),
        ("cadeia", "cadeia_passo4"),
        ("cadeia", "cadeia_passo6"),
        ("cadeia", "cadeia_passo8"),
        ("cadeia", "cadeia_forca05_passo2"),
        ("cadeia", "cadeia_forca05_passo4"),
        ("cadeia", "cadeia_forca05_passo8"),
        ("superficies", "superficie_degrau"),
        ("superficies", "superficie_rampa"),
        ("superficies", "opcao_dureza05"),
        ("superficies", "opcao_curva_constante"),
        ("superficies", "opcao_so_frente"),
        ("superficies", "opcao_sem_atenuacao"),
        ("inversao", "inversao_troca"),
        ("inversao", "inversao_troca_simetrica"),
    ];
    let mut pior = (0.0f32, String::new());
    for (fam, nome) in corpus {
        let f = ler(fam, nome);
        let (d, nossos, dele) = comparar(&f, &correr(&f));
        assert_eq!(nossos, dele, "{nome}: populacao {nossos} contra {dele}");
        assert!(
            d <= BARRA,
            "{nome}: max|Δ| {d:.3e} passa a barra {BARRA:.0e}"
        );
        if d > pior.0 {
            pior = (d, nome.to_string());
        }
    }
    assert_eq!(
        corpus.len(),
        23,
        "a populacao dos tracos encolheu — um gate que corre menos afirma menos"
    );
    println!("G-1b: 23 tracos, pior {:.3e} em {}", pior.0, pior.1);
}

/// ⭐⭐ **G-4 — A FORÇA ENTRA AO QUADRADO** (espec §4).
///
/// ⛔⛔ **Este gate nasceu de um defeito REAL que vinte e duas fixturas verdes não
/// viam**, e a razão é a armadilha que esta casa já nomeou uma vez: **`s² = s` em
/// `1`**, logo *um corpus na força máxima não testa a curva da força*. A tabela
/// que decide de que referência um verbo herda é uma **LISTA NEGRA**, e o verbo
/// novo nasceu a reivindicar uma referência que não o tem — o peso caía no
/// **slider cru**. Medido: com a força a `0,5` o alvo desloca `0,052399` e nós
/// deslocávamos `0,104798`, **exactamente `2×`**.
#[test]
fn a_forca_entra_ao_quadrado() {
    let cheia = ler("cadeia", "cadeia_passo2");
    let meia = ler("cadeia", "cadeia_forca05_passo2");
    assert_eq!(cheia.num("forca"), 1.0);
    assert_eq!(meia.num("forca"), 0.5);
    let (a, b) = (correr(&cheia), correr(&meia));
    let pico = |f: &Fixtura, s: &[[f32; 3]]| {
        (0..f.repouso.len())
            .map(|i| {
                (0..3)
                    .map(|k| (s[i][k] - f.repouso[i][k]).abs())
                    .fold(0.0f32, f32::max)
            })
            .fold(0.0f32, f32::max)
    };
    let razao = pico(&meia, &b) / pico(&cheia, &a);
    assert!(
        (razao - 0.25).abs() <= 1e-5,
        "meia forca deslocou {razao:.6} do que a forca cheia desloca — a lei e' o QUADRADO \
         (0,250000), e ler 0,5 aqui e' a curva a ser LINEAR"
    );
}

/// ⭐⭐ **G-3 — COM A CURVA CONSTANTE OS TOCADOS ATERRAM NO PLANO** (espec §4).
///
/// Força `1` e curva constante ⇒ **um único plano**: cada vértice caminha a
/// fracção inteira da própria distância, e `factor = 1` em toda a pegada.
///
/// ⚠️ **UM dab efectivo, e é por isso que o corpus é o `lei/*`:** num traço
/// inteiro a pegada ANDA, logo os tocados aterram em **vários** planos e a régua
/// não afirma nada. *A 1.ª redacção deste gate corria sobre um traço de oito
/// dabs e reprovava sobre produto correcto.*
///
/// ⚠️ **O ajuste é uma ALTURA (`z = a·x + b·y + c`), não um autovector**, e a
/// cerca está escrita: todas as superfícies do corpus são campos de altura com o
/// plano quase horizontal, onde o ajuste linear É o plano de mínimos quadrados.
/// ⛔ Numa parede vertical ele degenera — e o corpus não tem nenhuma.
#[test]
fn com_a_curva_constante_os_tocados_aterram_no_plano() {
    let mut pior = (0.0f64, String::new());
    for nome in ["lei_crista", "lei_rampa", "lei_degrau", "lei_bossas"] {
        let f = ler("lei", nome);
        assert_eq!(
            f.chave("curva"),
            "CONSTANT",
            "{nome}: a curva nao e' constante"
        );
        assert_eq!(f.num("forca"), 1.0, "{nome}: a forca nao e' cheia");
        let nossa = correr(&f);
        let tocados: Vec<[f64; 3]> = (0..f.repouso.len())
            .filter(|&i| nossa[i] != f.repouso[i])
            .map(|i| {
                [
                    f64::from(nossa[i][0]),
                    f64::from(nossa[i][1]),
                    f64::from(nossa[i][2]),
                ]
            })
            .collect();
        assert!(tocados.len() > 50, "{nome}: so' {} tocados", tocados.len());
        // Mínimos quadrados de `z = a·x + b·y + c`, pelas equações normais.
        let n = tocados.len() as f64;
        let (mut sx, mut sy, mut sz) = (0.0, 0.0, 0.0);
        for p in &tocados {
            sx += p[0];
            sy += p[1];
            sz += p[2];
        }
        let (mx, my, mz) = (sx / n, sy / n, sz / n);
        let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for p in &tocados {
            let (dx, dy, dz) = (p[0] - mx, p[1] - my, p[2] - mz);
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
            sxz += dx * dz;
            syz += dy * dz;
        }
        let det = sxx * syy - sxy * sxy;
        assert!(det.abs() > 1e-12, "{nome}: a pegada degenerou numa linha");
        let a = (sxz * syy - syz * sxy) / det;
        let b = (syz * sxx - sxz * sxy) / det;
        let residuo = tocados
            .iter()
            .map(|p| (p[2] - mz - a * (p[0] - mx) - b * (p[1] - my)).abs())
            .fold(0.0f64, f64::max)
            / (1.0 + a * a + b * b).sqrt();
        if residuo > pior.0 {
            pior = (residuo, nome.to_string());
        }
    }
    assert!(
        pior.0 <= 1e-6,
        "os tocados nao aterram num plano: residuo {:.3e} em {}",
        pior.0,
        pior.1
    );
    println!("G-3: pior residuo ao plano {:.3e} em {}", pior.0, pior.1);
}

/// ⛔⛔⛔ **A DIVERGÊNCIA DECLARADA — o modo *AFASTAR* não reproduz o alvo, e o
/// número está aqui.**
///
/// ⚠️ **A espec NUNCA reclamou paridade para este modo.** A §5 mede-o assim: *«o
/// modo afastar, esse, dá saída diferente (`7,0e-02`)»* — e esse número é a
/// distância entre a saída **invertida** e a **base** do PRÓPRIO alvo, não uma
/// barra contra nós. ⭐ **Este gate reproduz esse número ao dígito** (`7,0410e-02`
/// sobre as duas fixturas), que é a prova de que a leitura da espec está certa.
///
/// ⛔ **E a nossa lei diverge da dele em `1,007e-01`**, com a **mesma população
/// tocada** (`560` dos dois lados) e o **mesmo sentido** em cada vértice. O
/// mecanismo está medido e é o crescimento: a nossa afasta-se e a distância ao
/// plano **cresce**, então cada dab empurra mais (`0,0222 → 0,0361 → 0,0471 →
/// 0,0862 → 0,1272`, a saturar quando o vértice sai do elipsóide); a do alvo pára
/// perto do valor de **um** dab (`0,0292` no fim de sete efectivos).
///
/// ⏳ **ABERTO e nomeado**, com as duas hipóteses já REFUTADAS por medição: não é
/// o espelho exacto da base (`|d_inv + d_base| = 2,3e-02`, não zero) e não é a
/// troca do tecto que governa (isso mudaria a população para `442`, e ela é
/// `560`). *A lei que fica é a desta casa — o `Ctrl` afasta —, e ela é
/// **defensável e declarada**, não uma paridade falhada em silêncio.*
///
/// ⚠️ **O gate trava o número:** se a nossa lei mudar, ele reprova e obriga quem a
/// mudou a re-medir — uma divergência sem catraca vira uma nota que envelhece.
#[test]
fn o_afastar_e_uma_divergencia_declarada_com_numero() {
    let inv = ler("inversao", "inversao_afasta");
    let base = ler("lados", "traco_so_acima");
    assert_eq!(inv.repouso, base.repouso, "as duas partem do mesmo repouso");
    // (a) o número que a espec publica, reproduzido do CORPUS.
    let entre_os_dois_lados_dele = (0..inv.repouso.len())
        .map(|i| {
            (0..3)
                .map(|k| {
                    ((inv.saida[i][k] - inv.repouso[i][k])
                        - (base.saida[i][k] - base.repouso[i][k]))
                        .abs()
                })
                .fold(0.0f32, f32::max)
        })
        .fold(0.0f32, f32::max);
    assert!(
        (entre_os_dois_lados_dele - 7.0e-2).abs() <= 1e-3,
        "a espec publica 7,0e-02 entre os dois lados do alvo e o corpus da' {entre_os_dois_lados_dele:.4e}"
    );
    // (b) a nossa divergência, travada.
    let (nossa_divergencia, nossos, dele) = comparar(&inv, &correr(&inv));
    assert_eq!(
        nossos, dele,
        "a populacao tocada tem de bater: {nossos}/{dele}"
    );
    assert!(
        (0.09..=0.11).contains(&nossa_divergencia),
        "a divergencia declarada do AFASTAR era 1,007e-01 e agora e' {nossa_divergencia:.4e} — \
         se a lei mudou, re-meca e reescreva a nota; se nao mudou, algo a montante mudou"
    );
}

/// ⭐⭐⭐ **G-6 — O PLANO SEGUE O CURSOR E NÃO A ORIGEM** (espec §12).
///
/// A distância do **cursor** ao plano que o dab ajustou, medida **ao longo da
/// normal dele** e em raios de pincel, fica limitada. ⛔ *Sem isto, uma lei que
/// ancorasse o plano na origem da peça — ou no primeiro dab do traço — passaria
/// em toda a paridade de UM dab e só se revelaria num traço longo.*
///
/// # A barra, e a parte dela que é NOSSA
///
/// `0,75 R + |deslocamento| · R`, e os dois termos têm proveniências diferentes:
///
/// * o **segundo é EXACTO** — o deslocamento move o plano por `offset × R`
///   (espec §2.4), logo as duas células deslocadas trazem-no por construção;
/// * o **`0,75` é NOSSO e declarado**: a espec mede `0,0000 … 0,4947` raios
///   sobre as células de `lei/*` (mediana `0,1649`, o máximo na de menor raio),
///   e `0,75` é **`1,5×`** esse máximo — *porque um corpus de UM dab efectivo
///   não limita o que um traço longo faz*.
///
/// **MEDIDO aqui:** pior **`0,4947`** raios, na `lei_crista_raio02` — **o número
/// que a espec publica, ao dígito**, e na célula que ela nomeia (a de menor raio).
///
/// ⛔⛔⛔ **E a 1.ª redacção deste gate lia `0,2645` com uma explicação FALSA
/// escrita ao lado.** Ela dizia que o número menor vinha da régua ser mais larga
/// (*«mede antes de cada dab e fica com o pior»*) — plausível, e errado: ele vinha
/// de a porta de bancada devolver **o plano da CASA**, que o
/// [`ph2d_sculpt3d::Verb::Plane`] não usa. As duas leis diferem `17,1 %` do raio
/// no centro, e a barra de `0,75` raios engolia a diferença.
///
/// ⚠️⚠️ **Quem o apanhou foi o G-5**, que mede a mesma porta contra o plano do
/// oráculo com uma barra `750 000×` mais apertada: ali o desvio lê-se como
/// `6,8e-2` e não há folga onde ele se esconda. ⇒ *uma barra larga não é só uma
/// afirmação fraca — ela é o sítio onde uma régua errada sobrevive*, e a defesa é
/// ter na mesma porta um gate cuja barra não tenha folga nenhuma.
///
/// ⭐ Reproduzir o `0,4947` da espec **com a porta corrigida** é o que prova que
/// a régua passou a medir a lei certa: o número não foi ajustado, ele apareceu.
///
/// ⚠️⚠️ **A população que este gate alcança são `13` células e a espec diz `11`.**
/// A diferença são as **duas deslocadas**, que a espec conta à parte por o termo
/// do deslocamento ser delas — aqui elas entram na mesma varredura **porque a
/// barra já as contém pelo segundo termo**. *Medir mais células com a mesma
/// barra é estritamente mais forte; o que seria fraude era medir menos.*
///
/// ⛔ **A medição passa pela porta do produto**
/// ([`ph2d_sculpt3d::SculptStroke::plano_do_dab_para_teste`]) e **não** deriva o
/// plano da malha de saída: derivá-lo seria medi-lo *através* da cadeia de peso,
/// e um desvio não diria qual das duas falhou — a lei que a §8.4 desta mesma
/// espec já escreve para o corpus.
#[test]
fn o_plano_segue_o_cursor_e_nao_a_origem() {
    use ph2d_sculpt3d::{Dab, SculptStroke, Symmetry};

    /// O termo NOSSO da barra — ver o doc.
    const FOLGA_EM_RAIOS: f32 = 0.75;

    let mut pior = (0.0f32, String::new());
    let mut medidas = 0usize;
    for (familia, nome) in UM_DAB {
        if familia != "lei" {
            continue;
        }
        let f = ler(familia, nome);
        let mut m = grelha(&f).expect("grelha");
        let b = pincel(&f);
        let e = olho(&f);
        let mut s = SculptStroke::default();
        s.begin(&m);

        // ⚠️⚠️ **O plano mede-se em CADA dab, e o pior fica** — e a 1.ª redacção
        // media só o `cursores[0]`. Uma célula pode abrir com um dab que não move
        // nada (é o que a `lei_primeiro_dab` nomeia), e ali o plano medido **não
        // governa coisa nenhuma**: o meu próprio controlo apanhou isso na
        // primeira corrida, sobre a `lei_crista`.
        //
        // ⛔⛔ **E ele lê-se DEPOIS do dab, não antes** — o plano deste verbo sai
        // da PEGADA, e quem a monta é o dab, logo fora do gesto ele não existe.
        // A 2.ª redacção media-o antes e lia a pegada do dab ANTERIOR; quem a
        // apanhou foi o G-5, no `lei_area20`, cujo raio de área é `2 R`.
        let mut mexeu = false;
        for cur in &f.cursores {
            let dab = Dab::at(*cur, b.radius, e);
            let antes = m.positions().to_vec();
            s.dab(&mut m, &b, &dab, Symmetry::default());
            mexeu |= m.positions().iter().zip(&antes).any(|(a, r)| a != r);

            let (ponto, normal) = s
                .plano_do_ultimo_dab_para_teste()
                .expect("a superficie do corpus responde sempre");
            let d = (0..3).map(|k| (cur[k] - ponto[k]) * normal[k]).sum::<f32>();
            let em_raios = d.abs() / b.radius;

            // O termo exacto: o deslocamento move o plano por `offset × R`.
            let deslocamento = f.num("deslocamento_do_plano").abs();
            let barra = FOLGA_EM_RAIOS + deslocamento;
            assert!(
                em_raios <= barra,
                "{nome}: o cursor esta' a {em_raios:.4} raios do plano, e a barra \
                 e' {barra:.4} (0,75 + deslocamento {deslocamento:.4}) — o plano \
                 deixou de seguir o cursor"
            );
            if em_raios > pior.0 {
                pior = (em_raios, nome.to_string());
            }
        }
        medidas += 1;

        // ⭐ **O CONTROLO**: a célula tem de mover alguma coisa, senão a medição
        // acima é sobre um plano que o produto nunca chega a usar.
        assert!(
            mexeu,
            "{nome}: nenhum dab moveu um vertice — os planos medidos nesta \
             celula nao governam nada"
        );
    }
    // ⭐ **PISO DE POPULAÇÃO** — sem ele uma lista que encolhesse deixaria o gate
    // verde a medir quase nada (`CLAUDE.md` §5.0).
    assert_eq!(
        medidas, 13,
        "o G-6 correu {medidas} celulas de `lei/*` e a populacao alcancavel e' 13"
    );
    println!("G-6: 13 celulas, pior {:.4} raios em {}", pior.0, pior.1);

    // ⛔⛔ **A METADE QUE DISCRIMINA, e ela nasceu de uma MUTAÇÃO SOBREVIVENTE:**
    // ancorar o plano na ORIGEM (`fit.point = [0,0,0]`) deixava tudo acima
    // **verde**, porque o corpus vive perto da origem e ali *«segue o cursor»* e
    // *«está na origem»* dão o mesmo número. ⇒ a mesma célula, **deslocada `3`
    // unidades**, onde as duas leis divergem por construção.
    //
    // ⚠️ **A asserção é a MESMA barra**, e é isso que a torna uma lei e não um
    // caso especial: a distância do cursor ao plano é **invariante à
    // translação**, logo deslocar a peça não pode mudar o número.
    const LONGE: [f32; 3] = [3.0, 0.0, 0.0];
    let mut discriminantes = 0usize;
    for (familia, nome) in UM_DAB {
        if familia != "lei" {
            continue;
        }
        let f = ler(familia, nome);
        let mut m = grelha_deslocada(&f, LONGE).expect("grelha deslocada");
        let b = pincel(&f);
        let e = olho(&f);
        let mut s = SculptStroke::default();
        s.begin(&m);
        let deslocamento = f.num("deslocamento_do_plano").abs();
        let barra = FOLGA_EM_RAIOS + deslocamento;
        for cur in &f.cursores {
            let alvo = [cur[0] + LONGE[0], cur[1] + LONGE[1], cur[2] + LONGE[2]];
            let dab = Dab::at(alvo, b.radius, e);
            s.dab(&mut m, &b, &dab, Symmetry::default());
            let (ponto, normal) = s
                .plano_do_ultimo_dab_para_teste()
                .expect("a superficie do corpus responde sempre");
            let d = (0..3)
                .map(|k| (alvo[k] - ponto[k]) * normal[k])
                .sum::<f32>();
            let em_raios = d.abs() / b.radius;
            assert!(
                em_raios <= barra,
                "{nome} DESLOCADA: o cursor esta' a {em_raios:.4} raios do plano \
                 (barra {barra:.4}) — o plano esta' ancorado em qualquer coisa \
                 que NAO e' o cursor"
            );
        }
        discriminantes += 1;
    }
    assert_eq!(
        discriminantes, 13,
        "a metade discriminante correu {discriminantes} celulas e devia correr 13"
    );
    println!("G-6: e as mesmas 13 DESLOCADAS {LONGE:?} passam a mesma barra");
}

/// **AS `11` CÉLULAS do G-5, e a coluna `discrimina?` da tabela da espec §2.2.**
///
/// ⚠️ **A população CONTA-SE do directório menos duas partições declaradas:** as
/// `14` de `lei/*` menos a que não move nada (`lei_primeiro_dab`, que é o sujeito
/// do G-2) e menos as **duas deslocadas**, que a espec conta à parte porque ali o
/// plano é movido por `offset × R` e a régua deixaria de medir só o centro.
///
/// ⛔ **A coluna vem da espec e não de uma corrida:** escrevê-la do que o gate
/// mede seria a régua a copiar a resposta. O mecanismo de cada uma das três que
/// **não** discriminam está publicado ali — a superfície antissimétrica e as duas
/// células em que o conjunto do alvo tem **um** vértice.
const CELULAS_DO_G5: [(&str, bool); 11] = [
    ("lei_crista", true),
    ("lei_rampa", true),
    ("lei_degrau", true),
    ("lei_bossas", false),
    ("lei_crista_raio02", false),
    ("lei_crista_raio06", true),
    ("lei_area025", false),
    ("lei_area10", true),
    ("lei_area20", true),
    ("lei_normal025", true),
    ("lei_normal10", true),
];

/// O plano que o **ALVO** de facto usou, recuperado por ajuste da saída DELE.
///
/// ⭐ **Possível em TODAS as `11` porque o corpus de `lei/*` é curva CONSTANTE e
/// força cheia** — medido, as `14` declaram-no no cabeçalho: ali cada vértice
/// tocado caminha a distância inteira e **aterra no plano**, logo o ajuste é o
/// próprio plano e não uma aproximação. ⛔ Com curva suave isto não valeria, e a
/// régua teria de re-derivar a lei que está a julgar.
///
/// Devolve `(a, b, mx, my, mz)` de `z = mz + a·(x−mx) + b·(y−my)`.
fn plano_do_alvo(f: &Fixtura) -> (f64, f64, f64, f64, f64) {
    let tocados: Vec<[f64; 3]> = (0..f.repouso.len())
        .filter(|&i| f.saida[i] != f.repouso[i])
        .map(|i| {
            [
                f64::from(f.saida[i][0]),
                f64::from(f.saida[i][1]),
                f64::from(f.saida[i][2]),
            ]
        })
        .collect();
    assert!(
        tocados.len() > 50,
        "{}: so' {} tocados — o ajuste do plano do alvo mediria ruido",
        f.nome,
        tocados.len()
    );
    let n = tocados.len() as f64;
    let (mut sx, mut sy, mut sz) = (0.0, 0.0, 0.0);
    for p in &tocados {
        sx += p[0];
        sy += p[1];
        sz += p[2];
    }
    let (mx, my, mz) = (sx / n, sy / n, sz / n);
    let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in &tocados {
        let (dx, dy, dz) = (p[0] - mx, p[1] - my, p[2] - mz);
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
        sxz += dx * dz;
        syz += dy * dz;
    }
    let det = sxx * syy - sxy * sxy;
    assert!(
        det.abs() > 1e-12,
        "{}: a pegada degenerou numa linha",
        f.nome
    );
    (
        (sxz * syy - syz * sxy) / det,
        (syz * sxx - sxz * sxy) / det,
        mx,
        my,
        mz,
    )
}

/// A altura (com sinal) de um ponto contra o plano recuperado, em unidades de
/// objecto.
fn altura(p: [f64; 3], (a, b, mx, my, mz): (f64, f64, f64, f64, f64)) -> f64 {
    (p[2] - mz - a * (p[0] - mx) - b * (p[1] - my)) / (1.0 + a * a + b * b).sqrt()
}

/// **AS TRÊS CANDIDATAS REJEITADAS** (espec §2.2), na ordem da tabela: a lei que
/// esta casa já tinha, a média simples sobre `R_c` e a média ponderada sobre `R_c`.
///
/// ⛔⛔ **Elas vivem AQUI e não no produto, de propósito.** São leis que a medição
/// recusou: pô-las atrás de uma porta do motor daria três caminhos vivos para uma
/// pergunta que já tem resposta — e o que o gate precisa delas é só que sejam
/// **calculáveis**, para provar que a escolhida não foi escolhida à sorte.
fn candidatas_rejeitadas(f: &Fixtura, cursor: [f32; 3]) -> [[f64; 3]; 3] {
    let b = pincel(f);
    let e = olho(f);
    let raio = b.radius;
    let frac = if b.area_radius_frac > 0.0 {
        b.area_radius_frac
    } else {
        b.normal_radius_frac
    };
    let rc = raio * frac;
    let de_frente = |i: usize| {
        let n = f.normais[i];
        n[0] * e[0] + n[1] * e[1] + n[2] * e[2] <= 0.0
    };
    let dist = |i: usize| {
        let p = f.repouso[i];
        ((p[0] - cursor[0]).powi(2) + (p[1] - cursor[1]).powi(2) + (p[2] - cursor[2]).powi(2))
            .sqrt()
    };
    // ⚠️ **O mesmo peso do produto** (`stroke_normal_do_gesto::peso_da_amostra`):
    // `smoothstep(1 − d/r)`. Escrever outro aqui faria a candidata «ponderada»
    // ser uma quarta lei, e não a que a espec tabela.
    let peso = |d: f32, r: f32| -> f64 {
        let t = 1.0 - d / r;
        f64::from((t * t * (3.0 - 2.0 * t)).clamp(0.0, 1.0))
    };

    let mut casa = ([0.0f64; 3], 0usize);
    let mut media_rc = ([0.0f64; 3], 0usize);
    let mut pond_rc = ([0.0f64; 3], 0.0f64);
    for i in 0..f.repouso.len() {
        if !de_frente(i) {
            continue;
        }
        let d = dist(i);
        let p = f.repouso[i];
        if d <= raio {
            for (soma, c) in casa.0.iter_mut().zip(p) {
                *soma += f64::from(c);
            }
            casa.1 += 1;
        }
        if d <= rc {
            for (soma, c) in media_rc.0.iter_mut().zip(p) {
                *soma += f64::from(c);
            }
            media_rc.1 += 1;
            let w = peso(d, rc);
            for (soma, c) in pond_rc.0.iter_mut().zip(p) {
                *soma += f64::from(c) * w;
            }
            pond_rc.1 += w;
        }
    }
    assert!(
        casa.1 > 0 && media_rc.1 > 0 && pond_rc.1 > 0.0,
        "{}: uma candidata ficou sem amostras — a celula nao discrimina nada",
        f.nome
    );
    let dividir = |s: [f64; 3], n: f64| [s[0] / n, s[1] / n, s[2] / n];
    [
        dividir(casa.0, casa.1 as f64),
        dividir(media_rc.0, media_rc.1 as f64),
        dividir(pond_rc.0, pond_rc.1),
    ]
}

/// ⭐⭐⭐ **G-5 — O CENTRO DA ÁREA É A MÉDIA DAS POSIÇÕES PUXADAS PARA O CURSOR**
/// (espec §2.2 e §12), em DUAS metades.
///
/// A lei é a que nenhuma intuição dá: *o peso não multiplica a posição — ele PUXA
/// a posição para o cursor, e a média é SIMPLES*. Um vértice no miolo da pegada
/// (peso `1`) contribui com o **cursor**; um na borda (peso `0`) contribui com a
/// **própria posição**.
///
/// # (a) A candidata da §2.2 cai no plano do oráculo
///
/// Medida pela **porta do produto** ([`ph2d_sculpt3d::SculptStroke::plano_do_dab_para_teste`])
/// contra o plano que o alvo de facto usou, recuperado por ajuste da saída DELE.
/// Barra `1e-6`, que é a de aceitação do G-1.
///
/// # (b) ⛔⛔ As TRÊS outras REPROVAM, e é isto que torna a (a) uma afirmação
///
/// *Sem esta metade o gate dizia «a nossa lei concorda com o alvo» sem dizer que
/// outra lei plausível NÃO concordaria* — e é exactamente a forma que esta casa
/// já pagou seis vezes (uma régua que o produto satisfaz por construção).
///
/// As três são: a **lei que esta casa já tinha** (média simples da pegada
/// inteira, que os outros quatro verbos de plano usam), a **média simples sobre
/// `R_c`** e a **média ponderada sobre `R_c`**.
///
/// Barra **`1e-3`**, e ela é **derivada e não escolhida**: a tabela da §2.2
/// publica o piso das oito células discriminantes em **`0,00269`**, logo a barra
/// fica `2,69×` abaixo dele e `1 000×` acima da de aceitação.
///
/// # ⚠️ O PISO DE POPULAÇÃO, e porque ele tem DOIS números
///
/// **MEDIDO:** (a) pior `6,521e-8` no `lei_area20` — a lei do produto cai no
/// plano do oráculo ao nível da paridade do G-1; (b) piso **`0,00269`**, no
/// `lei_area20`, que é **exactamente o número que a §2.2 publica** e na célula
/// que ela nomeia. ⭐ *As três candidatas rejeitadas foram reprogramadas aqui do
/// enunciado da espec, e o piso saiu igual ao dela — é essa coincidência que
/// prova que são as MESMAS três, e não três leis parecidas.*
///
/// `11` células medidas e `8` discriminantes. ⛔ Sem o segundo, degenerar o corpus
/// (ou a coluna da espec) deixaria o gate verde a julgar três células — e as três
/// que não discriminam **não discriminam por MECANISMO publicado**: a superfície
/// antissimétrica, onde a altura média de qualquer conjunto simétrico é zero por
/// construção, e as duas em que o conjunto do alvo tem **um** vértice.
#[test]
fn o_centro_da_area_e_a_media_das_posicoes_puxadas_para_o_cursor() {
    use ph2d_sculpt3d::{Dab, SculptStroke};

    /// A barra de aceitação (a do G-1).
    const ACEITA: f64 = 1e-6;
    /// A barra da discriminação — `2,69×` abaixo do piso publicado na §2.2.
    const DISCRIMINA: f64 = 1e-3;

    let (mut pior_a, mut pior_a_nome) = (0.0f64, String::new());
    let (mut pior_b, mut pior_b_nome) = (f64::INFINITY, String::new());
    let (mut medidas, mut discriminantes) = (0usize, 0usize);

    for (nome, esperado_discrimina) in CELULAS_DO_G5 {
        let f = ler("lei", nome);
        assert_eq!(
            f.chave("curva"),
            "CONSTANT",
            "{nome}: a curva deixou de ser constante — o `plano_do_alvo` deixaria \
             de recuperar o plano e passaria a recuperar uma fraccao dele"
        );
        assert_eq!(
            f.num("deslocamento_do_plano"),
            0.0,
            "{nome}: ha' deslocamento"
        );
        let plano = plano_do_alvo(&f);

        // ⚠️ **O dab EFECTIVO é o segundo**: o primeiro é o pen-down, que não
        // move nada (espec §1, e é o sujeito do G-2). Medir no primeiro leria um
        // plano que o produto nunca chega a usar.
        assert_eq!(f.cursores.len(), 2, "{nome}: a celula deixou de ter 2 dabs");
        let cursor = f.cursores[1];

        // (a) — a candidata da §2.2, pela PORTA DO PRODUTO, sobre a malha em
        // repouso (é onde o dab efectivo a encontra).
        let mut m = grelha(&f).expect("grelha");
        let b = pincel(&f);
        let e = olho(&f);
        let mut s = SculptStroke::default();
        s.begin(&m);
        // ⚠️ **O pen-down CORRE antes da medição, e não é decoração:** ele não
        // move um vértice (G-2) **e semeia a memória do plano** (espec §6). Sem
        // ele a porta leria um plano sem memória, que é outro plano no dia em que
        // uma fixtura declarar firmeza — *e as `11` de hoje declaram `0`, o que
        // torna a omissão invisível exactamente até deixar de o ser*.
        s.dab(
            &mut m,
            &b,
            &Dab::at(f.cursores[0], b.radius, e),
            ph2d_sculpt3d::Symmetry::default(),
        );
        assert_eq!(
            m.positions(),
            &f.repouso[..],
            "{nome}: o pen-down moveu barro — a medicao seguinte seria sobre outra malha"
        );
        s.dab(
            &mut m,
            &b,
            &Dab::at(cursor, b.radius, e),
            ph2d_sculpt3d::Symmetry::default(),
        );
        let (ponto, _) = s
            .plano_do_ultimo_dab_para_teste()
            .expect("a superficie do corpus responde sempre");
        let h = altura(
            [
                f64::from(ponto[0]),
                f64::from(ponto[1]),
                f64::from(ponto[2]),
            ],
            plano,
        )
        .abs();
        assert!(
            h <= ACEITA,
            "{nome}: a lei do produto cai {h:.3e} FORA do plano que o alvo usou \
             (barra {ACEITA:.0e}) — a candidata da §2.2 deixou de reproduzir"
        );
        if h > pior_a {
            pior_a = h;
            pior_a_nome = nome.to_string();
        }

        // (b) — as três rejeitadas.
        let alturas: Vec<f64> = candidatas_rejeitadas(&f, cursor)
            .into_iter()
            .map(|c| altura(c, plano).abs())
            .collect();
        let menor = alturas.iter().copied().fold(f64::INFINITY, f64::min);
        if esperado_discrimina {
            assert!(
                menor >= DISCRIMINA,
                "{nome}: a candidata errada mais proxima erra so' {menor:.5} \
                 (barra {DISCRIMINA:.0e}) — a espec declara esta celula \
                 DISCRIMINANTE e ela deixou de o ser"
            );
            discriminantes += 1;
            if menor < pior_b {
                pior_b = menor;
                pior_b_nome = nome.to_string();
            }
        } else {
            // ⛔ **A metade NEGATIVA, e ela é metade do valor:** a espec declara
            // TRÊS células não-discriminantes com mecanismo publicado. Se uma
            // delas passasse a discriminar, a tabela deixou de descrever o
            // corpus — e uma tabela que não descreve o corpus é a licença que o
            // `CLAUDE.md` §5.0 nomeia.
            assert!(
                menor < DISCRIMINA,
                "{nome}: a espec declara esta celula NAO-discriminante (mecanismo \
                 publicado na §2.2) e ela erra {menor:.5} — a tabela deixou de \
                 descrever o corpus"
            );
        }
        medidas += 1;
    }

    assert_eq!(
        medidas, 11,
        "o G-5 correu {medidas} celulas e a populacao da §2.2 e' 11"
    );
    assert_eq!(
        discriminantes, 8,
        "o G-5 achou {discriminantes} celulas discriminantes e a §2.2 publica 8"
    );
    println!(
        "G-5: 11 celulas, 8 discriminantes · (a) pior {:.3e} em {} · (b) piso {:.5} em {}",
        pior_a, pior_a_nome, pior_b, pior_b_nome
    );
}
