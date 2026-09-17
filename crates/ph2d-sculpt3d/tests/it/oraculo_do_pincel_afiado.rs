//! ⭐⭐⭐ **A BANCADA DO PINCEL AFIADO** — os gates da
//! `docs/3D/cleanroom/SPEC_pincel_afiado.md` §12, contra as **80** fixturas do
//! oráculo. Aqui moram a LEI (um dab, as cadeias, a forma fechada); o TRAÇO
//! arrastado e a régua do produto vivem no irmão
//! [`super::oraculo_do_pincel_afiado_produto`].
//!
//! ⚠️ **O corpus é DADO, não código:** `gzip` de texto, cabeçalho de
//! proveniência e blocos (`r` repouso · `n` normais · `s` saída · `c` cursores ·
//! `m` máscara · `d<j>`/`p<k>` as fotos intermédias). Quem o regenera é o
//! subagente-E — a zona dele é negada a esta janela.
//!
//! ⚠️⚠️ **DOIS pontos em que este corpus não se lê como o do pincel de plano**,
//! e os dois vieram da auditoria R-pré:
//!
//! 1. **As normais de vértice do caminho por SCRIPT são as NOSSAS, congeladas.**
//!    O bloco `n` é a normal que o alvo guarda no repouso — a média pesada pelo
//!    ÂNGULO do canto —, e a lei que reproduz o traço dele é a média **sem
//!    peso**, que é a da nossa casa. Medido pelo R: a cadeia das bossas passa de
//!    `1,6e-6` para `5,8e-8` ao usar a nossa. ⇒ a bancada **não injecta** o
//!    bloco `n`; ela deixa a malha calcular as suas e PREGA-AS para o resto da
//!    cadeia, porque *este caminho do alvo não as refresca entre dabs* e o nosso
//!    refresca sempre (sem pregar, a cadeia desvia `4,45e-2` por uma razão que
//!    não é a lei).
//! 2. **Os blocos são ESPARSOS onde a saída é esparsa** (`s <índice> x y z`) e
//!    densos onde ela é densa. Um leitor que presumisse um dos dois mediria
//!    outra malha — ou entraria em pânico à primeira fixtura da outra família.

use ph2d_mesh::{Face, Mesh, Ray};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A barra das fixturas planas e de cilindro (§12, G-1).
pub const BARRA_PLANA: f32 = 2e-6;
/// A barra das fixturas de bossas — elas exigem a normal da área do §2.3, e é
/// também a barra das cadeias (§12, G-2), que a escada do [`barra`] escolhe pela
/// superfície em vez de por família.
///
/// ⚠️ **Uma constante por FAMÍLIA seria uma segunda resposta** à mesma pergunta —
/// *quanta folga esta superfície pede* —, e as duas divergiriam no dia em que a
/// bancada ganhasse uma cadeia numa superfície nova.
pub const BARRA_BOSSAS: f32 = 1e-5;
/// Quanto um vértice tem de andar para contar como MOVIDO (o mesmo da bancada
/// do pincel de plano).
pub const LIMIAR: f32 = 1e-7;

fn pasta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/pincel_afiado")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da do pincel de plano.
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

/// Uma fixtura lida: o cabeçalho, o repouso, e cada bloco de posições já
/// **densificado** contra o repouso.
pub struct Fixtura {
    pub nome: String,
    pub cab: BTreeMap<String, String>,
    pub repouso: Vec<[f32; 3]>,
    pub normais: Vec<[f32; 3]>,
    pub cursores: Vec<[f32; 3]>,
    pub mascara: Vec<f32>,
    /// `s` · `d1`.. · `p1`.. — cada um com o tamanho do repouso.
    pub blocos: BTreeMap<String, Vec<[f32; 3]>>,
}

impl Fixtura {
    pub fn chave(&self, k: &str) -> &str {
        self.cab
            .get(k)
            .unwrap_or_else(|| panic!("{}: o cabecalho nao tem `{k}`", self.nome))
    }

    /// O primeiro número de uma linha do cabeçalho (várias trazem a unidade ao
    /// lado, entre parênteses).
    pub fn num(&self, k: &str) -> f32 {
        let v = self.chave(k);
        v.split_whitespace()
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(|| panic!("{}: `{k}` = `{v}` nao e' numero", self.nome))
    }

    pub fn sim(&self, k: &str) -> bool {
        matches!(self.chave(k), "True" | "true" | "sim")
    }

    pub fn saida(&self) -> &[[f32; 3]] {
        &self.blocos["s"]
    }
}

/// Uma linha de bloco antes de ser densificada: o índice (só nos ESPARSOS) e o
/// ponto.
type LinhaCrua = (Option<usize>, [f32; 3]);

fn v3(c: &[&str]) -> [f32; 3] {
    [
        c[0].parse().expect("x"),
        c[1].parse().expect("y"),
        c[2].parse().expect("z"),
    ]
}

pub fn ler(familia: &str, nome: &str) -> Fixtura {
    let texto = inflar(&format!("{familia}/{nome}.txt.gz"));
    let mut cab = BTreeMap::new();
    let (mut repouso, mut normais, mut cursores, mut mascara) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    // Os blocos de posição ficam CRUS até o repouso estar lido — um bloco
    // esparso não se densifica sem ele.
    let mut crus: BTreeMap<String, Vec<LinhaCrua>> = BTreeMap::new();
    for l in texto.lines() {
        if let Some(resto) = l.strip_prefix("# ") {
            if let Some((k, v)) = resto.split_once(": ") {
                cab.insert(k.trim().to_string(), v.trim().to_string());
            }
            continue;
        }
        let campos: Vec<&str> = l.split_whitespace().collect();
        let Some(tag) = campos.first().copied() else {
            continue;
        };
        match tag {
            "r" => repouso.push(v3(&campos[1..])),
            "n" => normais.push(v3(&campos[1..])),
            "c" => cursores.push(v3(&campos[1..])),
            "m" => mascara.push(campos[1].parse().expect("mascara")),
            _ => {
                // ⚠️ **A FORMA decide, nunca o nome do bloco:** `s x y z` é
                // denso e `s i x y z` é esparso, e as duas formas convivem no
                // mesmo corpus (o `lei/` publica denso, o `produto/` esparso).
                let esparso = campos.len() == 5;
                let ponto = v3(&campos[campos.len() - 3..]);
                let idx = esparso.then(|| campos[1].parse().expect("indice"));
                crus.entry(tag.to_string()).or_default().push((idx, ponto));
            }
        }
    }
    let n: usize = cab
        .get("vertices")
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("{nome}: o cabecalho nao tem `vertices`"));
    assert_eq!(repouso.len(), n, "{nome}: bloco `r` nao bate o cabecalho");
    assert_eq!(normais.len(), n, "{nome}: bloco `n` nao bate o cabecalho");
    let mut blocos = BTreeMap::new();
    for (tag, linhas) in crus {
        // Um bloco ESPARSO parte do repouso e sobrescreve o que mudou; um DENSO
        // vem inteiro, na ordem dos vértices. As duas formas convivem no corpus.
        let denso = if linhas.iter().all(|(i, _)| i.is_some()) {
            let mut v = repouso.clone();
            for (idx, p) in linhas {
                v[idx.expect("esparso")] = p;
            }
            v
        } else {
            assert!(
                linhas.iter().all(|(i, _)| i.is_none()),
                "{nome}: o bloco `{tag}` mistura linhas densas e esparsas"
            );
            let v: Vec<[f32; 3]> = linhas.into_iter().map(|(_, p)| p).collect();
            assert_eq!(v.len(), n, "{nome}: bloco `{tag}` denso nao bate `r`");
            v
        };
        blocos.insert(tag, denso);
    }
    assert!(blocos.contains_key("s"), "{nome}: sem bloco `s`");
    Fixtura {
        nome: nome.to_string(),
        cab,
        repouso,
        normais,
        cursores,
        mascara,
        blocos,
    }
}

/// **A MALHA de entrada**, derivada da contagem de vértices e do cabeçalho.
///
/// ⚠️⚠️ **Os quadriláteros carregam-se como QUADRILÁTEROS** (espec §2.3.1): a
/// normal de vértice da nossa casa é a média sem peso das normais das faces, e
/// triangular antes de carregar muda-a — medido, o G-3b da grelha 48² reprova
/// (`3,25e-2` contra a barra de `3e-2`). ⇒ só triangula quem o cabeçalho diz que
/// **já vem** triangulado.
pub fn malha(f: &Fixtura) -> Mesh {
    let n = f.repouso.len();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lado = (n as f64).sqrt().round() as usize;
    assert_eq!(lado * lado, n, "{}: {n} vertices nao e' uma grelha", f.nome);
    let triangulada = f.chave("superficie").contains("PARTIDO em dois triangulos");
    let l32 = u32::try_from(lado).expect("lado");
    let mut faces = Vec::with_capacity((lado - 1) * (lado - 1));
    for j in 0..lado - 1 {
        for i in 0..lado - 1 {
            let a = u32::try_from(j * lado + i).expect("indice");
            let (b, c, d) = (a + 1, a + 1 + l32, a + l32);
            if triangulada {
                // A diagonal do cabeçalho: `(i,j)–(i+1,j+1)`.
                faces.push(Face::tri(a, b, c));
                faces.push(Face::tri(a, c, d));
            } else {
                faces.push(Face::quad(a, b, c, d));
            }
        }
    }
    Mesh::from_parts(f.repouso.clone(), faces).expect("malha")
}

/// **A DIRECÇÃO em que o raio de picagem viaja**, do cabeçalho — negada, como na
/// bancada do plano: a chave publica a direcção da vista no sentido em que ela
/// aponta para quem olha, e o nosso [`Dab::eye`] aponta para dentro da cena.
pub fn olho(f: &Fixtura) -> [f32; 3] {
    let v = f.chave("vista");
    assert!(
        v.contains("olha ao longo de -z") || v.contains("olha para -z"),
        "{}: vista `{v}` sem tradução",
        f.nome
    );
    [0.0, 0.0, -1.0]
}

/// **O PINCEL que o cabeçalho descreve.**
///
/// ⚠️⚠️ **Todo knob sai do CABEÇALHO, e uma chave que falte FAZ PANIC** — é a
/// única forma de um leitor de 80 fixturas não medir outro programa em silêncio.
pub fn pincel(f: &Fixtura) -> Brush {
    let verb = match f.chave("pincel") {
        "DRAW_SHARP" => Verb::DrawSharp,
        "DRAW" => Verb::Draw,
        outro => panic!("{}: pincel `{outro}` sem tradução", f.nome),
    };
    let falloff = match f.chave("curva") {
        // Os rótulos são os da API **pública** do alvo, que é a chave de
        // regeneração das fixturas (§4.1.13 da skill os cobre).
        "POW4" => Falloff::Sharper,
        "SHARP" => Falloff::Sharp,
        "SMOOTH" => Falloff::Smooth,
        "CONSTANT" => Falloff::Constant,
        outra => panic!("{}: curva `{outra}` sem tradução", f.nome),
    };
    // ⭐⭐ **A direcção EFECTIVA sai de DUAS chaves, e o nosso `invert` é
    // RELATIVO ao verbo.** No alvo o pincel tem uma direcção própria e o
    // modificador inverte-a; nesta casa o `invert` quer dizer *«ao contrário da
    // direcção de FÁBRICA deste verbo»* — e ela é **afundar** no afiado e
    // **levantar** no desenho comum.
    //
    // ⚠️ **Escrevê-lo sem o termo do verbo dá `0,44` de erro na fixtura de
    // controlo** (medido): a mesma linha do cabeçalho pede `invert = false` num
    // pincel e `true` no outro.
    let ctrl = f.chave("modificador").starts_with("Ctrl");
    let afunda = (f.chave("direccao") == "SUBTRACT") != ctrl;
    let soma = afunda != verb.afunda_de_fabrica();
    let raio = if f.cab.contains_key("raio_objeto") {
        f.num("raio_objeto")
    } else {
        f.num("raio_efectivo_objecto")
    };
    // O caminho por script do alvo **não** aplica o espaçamento nem a atenuação;
    // o arrastado aplica os dois. O cabeçalho publica os dois factos e eles têm
    // de concordar.
    let arrastado = f.chave("metodo_do_traco") == "SPACE";
    assert_eq!(
        arrastado,
        f.sim("atenuacao_por_espacamento"),
        "{}: o método do traço e a atenuação discordam no cabeçalho",
        f.nome
    );
    if arrastado {
        assert_eq!(
            f.num("espacamento_pct_do_diametro"),
            ph2d_sculpt3d::ESPACAMENTO_DO_AFIADO_PCT,
            "{}: o espaçamento da fixtura não é o de fábrica",
            f.nome
        );
    }
    Brush {
        verb,
        // ⭐⭐ **O MODO é o `B` em TODA fixtura deste corpus, e a razão é a
        // proveniência:** ele foi corrido no alvo restrito, logo a lei que o
        // governa é a da referência que ESSE alvo é — para os dois verbos.
        //
        // ⚠️ **Medido:** com o modo de fábrica do `Brush` a fixtura de controlo
        // do desenho comum desvia `3,6e-1`, porque no `S` o `Draw` tem o alcance
        // do SculptGL (`0,1 R`) e não o `1,0 R` desta referência. *Uma fixtura
        // de outro programa lida com a lei do programa errado mede outra coisa.*
        mode: ph2d_sculpt3d::RefMode::B,
        falloff,
        radius: raio,
        strength: f.num("forca"),
        hardness: f.num("dureza"),
        accumulate: f.sim("acumula"),
        front_faces_only: f.sim("so_faces_de_frente"),
        normal_radius_frac: f.num("raio_da_normal"),
        auto_smooth: f.num("auto_alisamento"),
        surface_only: f.sim("so_ligados"),
        invert: soma,
        traco_arrastado: arrastado,
        ..Brush::default()
    }
}

/// Onde a vertical pelo ponto pedido corta a superfície VIVA — o que o raio do
/// rato faz.
pub fn na_superficie(m: &Mesh, x: f32, y: f32, olho: [f32; 3]) -> Option<[f32; 3]> {
    let origem = [x - olho[0] * 10.0, y - olho[1] * 10.0, -olho[2] * 10.0];
    m.raycast(&Ray::new(origem, olho)).map(|h| h.point)
}

pub fn maior_distancia(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    assert_eq!(a.len(), b.len(), "populações diferentes");
    a.iter()
        .zip(b)
        .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max))
        .fold(0.0, f32::max)
}

pub fn movidos(repouso: &[[f32; 3]], pos: &[[f32; 3]]) -> usize {
    repouso
        .iter()
        .zip(pos)
        .filter(|(r, p)| (0..3).any(|k| (r[k] - p[k]).abs() > LIMIAR))
        .count()
}

/// **CORRE os `n` primeiros dabs da fixtura pelo caminho do PRODUTO.**
///
/// ⚠️ **As normais são PREGADAS entre dabs** — ver o cabeçalho deste ficheiro.
/// ⚠️ **O cursor sai do cabeçalho:** `DADO` usa o ponto `c` como está; `VIVO`
/// lança o raio na superfície de agora, que é o que o alvo faz.
pub fn correr(f: &Fixtura, dabs: usize) -> Vec<[f32; 3]> {
    let mut m = malha(f);
    let b = pincel(f);
    let e = olho(f);
    if !f.mascara.is_empty() {
        m.masks_mut().copy_from_slice(&f.mascara);
    }
    let normais0 = m.normals().to_vec();
    let vivo = f.chave("cursor").starts_with("VIVO");
    let mut s = SculptStroke::default();
    s.begin(&m);
    for c in f.cursores.iter().take(dabs) {
        let centro = if vivo {
            na_superficie(&m, c[0], c[1], e).expect("o ponto pedido nao corta a superficie")
        } else {
            *c
        };
        s.dab(
            &mut m,
            &b,
            &Dab::at(centro, b.radius, e),
            Symmetry::default(),
        );
        m.pregar_normais_para_teste(&normais0);
    }
    m.positions().to_vec()
}

/// A barra desta fixtura: as de bossas exigem a normal da área e têm a sua.
fn barra(f: &Fixtura) -> f32 {
    if f.chave("superficie").contains("bossas") {
        BARRA_BOSSAS
    } else {
        BARRA_PLANA
    }
}

/// ⭐⭐⭐ **G-1 — a lei de UM dab reproduz o oráculo**, nas `16` fixturas que a
/// espec §12 conta (as `17` de `lei/` menos a pegada projectada, que está na
/// catraca dos pendentes de decisão do dono).
#[test]
fn a_lei_de_um_dab_reproduz_o_oraculo() {
    const CELULAS: [&str; 16] = [
        "um_dab_curva_afiada",
        "um_dab_curva_suave_somar",
        "um_dab_curva_quadratica",
        "um_dab_curva_constante",
        "um_dab_forca_meia",
        "um_dab_dureza_meia",
        "um_dab_controlo_desenho_comum",
        "plano_do_pen_down_inerte",
        "mascara_metade",
        "cilindro_um_dab",
        "so_faces_de_frente_cilindro",
        "so_faces_de_frente_cilindro_controlo",
        "normal_da_area_raio_0_25",
        "normal_da_area_raio_0_5",
        "normal_da_area_raio_1_0",
        "normal_da_area_raio_2_0",
    ];
    let mut pior = 0.0f32;
    for nome in CELULAS {
        let f = ler("lei", nome);
        assert_eq!(f.cursores.len(), 1, "{nome}: nao e' um dab so'");
        let nossa = correr(&f, 1);
        let d = maior_distancia(&nossa, f.saida());
        let b = barra(&f);
        assert!(d <= b, "{nome}: max|Δ| {d:.3e} passa a barra {b:.0e}");
        let (n, dele) = (movidos(&f.repouso, &nossa), movidos(&f.repouso, f.saida()));
        assert_eq!(
            n, dele,
            "{nome}: {n} vertices movidos contra {dele} do alvo"
        );
        assert!(n > 0, "{nome}: o dab nao moveu nada — a celula esta' vazia");
        pior = pior.max(d);
    }
    println!("G-1: pior de {} celulas = {pior:.3e}", CELULAS.len());
}

/// ⭐⭐⭐ **G-2 — as CADEIAS reproduzem o oráculo, estado a estado** (`36`
/// estados em `5` cadeias).
///
/// ⚠️ **Cada `d<j>` é uma corrida PRÓPRIA** com os `j` primeiros cursores, e o
/// cabeçalho di-lo: correr uma cadeia só e fotografar pelo caminho mediria uma
/// composição que a fixtura não publica.
#[test]
fn as_cadeias_reproduzem_o_oraculo() {
    const CADEIAS: [&str; 5] = [
        "plano_cursor_vivo",
        "bossas_cursor_vivo",
        "mesmo_ponto_cursor_vivo",
        "mesmo_ponto_cursor_dado",
        "cilindro_cursor_vivo",
    ];
    let (mut estados, mut pior) = (0usize, 0.0f32);
    for nome in CADEIAS {
        let f = ler("cadeia", nome);
        let b = barra(&f);
        for j in 1..=f.cursores.len() {
            let alvo = if j == f.cursores.len() {
                f.saida()
            } else {
                &f.blocos[&format!("d{j}")]
            };
            let nossa = correr(&f, j);
            let d = maior_distancia(&nossa, alvo);
            assert!(
                d <= b,
                "{nome}, dab {j}: max|Δ| {d:.3e} passa a barra {b:.0e}"
            );
            pior = pior.max(d);
            estados += 1;
        }
    }
    assert_eq!(estados, 36, "a populacao do G-2 mudou");
    println!("G-2: pior de {estados} estados = {pior:.3e}");
}

/// ⭐⭐ **G-7 — a auto-limitação é a da FORMA FECHADA** (espec §4.1):
/// `z_{k+1} = z_k + R·S²·a·C(z_k/R)`, com `a = 1` no caminho por script.
///
/// ⭐ **E o CONTROLO é a metade que a torna uma afirmação:** com o cursor
/// IMPOSTO na superfície de repouso a mesma cadeia é **linear** — cada dab vale
/// o primeiro —, e é a diferença entre as duas que diz que o limite vem do
/// cursor descer com o vinco, e não da curva.
#[test]
fn a_auto_limitacao_e_a_da_forma_fechada() {
    let f = ler("cadeia", "mesmo_ponto_cursor_vivo");
    let b = pincel(&f);
    let (r, s) = (b.radius, b.strength);
    // A recorrência, escrita da espec e avaliada com a NOSSA curva.
    let mut z = 0.0f32;
    let mut lei = Vec::new();
    for _ in 0..f.cursores.len() {
        z += r * s * s * b.falloff.weight(z / r);
        lei.push(z);
    }
    // O fundo do vinco: o vértice que mais desceu, dab a dab.
    let mut medido = Vec::new();
    for j in 1..=f.cursores.len() {
        let pos = correr(&f, j);
        let fundo = f
            .repouso
            .iter()
            .zip(&pos)
            .map(|(a, p)| a[2] - p[2])
            .fold(0.0f32, f32::max);
        medido.push(fundo);
    }
    for (k, (a, b)) in lei.iter().zip(&medido).enumerate() {
        let d = (a - b).abs();
        assert!(
            d <= 1e-6,
            "dab {}: a forma fechada da' {a:.6} e a cadeia {b:.6} (Δ {d:.3e})",
            k + 1
        );
    }
    // O CONTROLO: com o cursor imposto, o aprofundamento é linear.
    let g = ler("cadeia", "mesmo_ponto_cursor_dado");
    let pos = correr(&g, g.cursores.len());
    let fundo = g
        .repouso
        .iter()
        .zip(&pos)
        .map(|(a, p)| a[2] - p[2])
        .fold(0.0f32, f32::max);
    let linear = r * s * s * g.cursores.len() as f32;
    assert!(
        (fundo - linear).abs() <= 1e-6,
        "o controlo devia ser linear: {fundo:.6} contra {linear:.6}"
    );
    assert!(
        fundo - medido[medido.len() - 1] >= 1e-2,
        "o controlo e a cadeia viva deviam SEPARAR-SE — sem isso o gate nao afirma o limite"
    );
}

/// ⭐ **G-9 — o PRIMEIRO dab é o do desenho comum, ao bit** (espec §1): o que
/// separa os dois pincéis nasce do **segundo** dab, e é uma propriedade do
/// TRAÇO e não do dab.
#[test]
fn o_primeiro_dab_e_o_do_desenho_comum() {
    let afiado = ler("lei", "um_dab_curva_afiada");
    let comum = ler("lei", "um_dab_controlo_desenho_comum");
    // ⚠️ **A fixtura do controlo é um `DRAW` com a curva AFIADA** — é o que a
    // torna o controlo certo: com curvas diferentes a igualdade seria falsa por
    // um motivo que não é o da espec.
    assert_eq!(pincel(&comum).verb, Verb::Draw, "o controlo mudou de verbo");
    assert_eq!(
        pincel(&comum).falloff,
        pincel(&afiado).falloff,
        "o controlo tem de trazer a MESMA curva"
    );
    let d = maior_distancia(&correr(&afiado, 1), &correr(&comum, 1));
    assert_eq!(d, 0.0, "o primeiro dab dos dois devia ser igual AO BIT");
    let dele = maior_distancia(afiado.saida(), comum.saida());
    assert_eq!(
        dele, 0.0,
        "no alvo eles são iguais ao bit — a fixtura mudou"
    );
}

/// ⭐⭐⭐ **AS DUAS LEIS DE NORMAL DE VÉRTICE, e porque a bancada usa a NOSSA**
/// (espec §2.3.1).
///
/// O bloco `n` de cada fixtura é a normal que o alvo guarda **no repouso**: a
/// média das normais unitárias das faces vizinhas **pesada pelo ângulo do
/// canto**. A lei que reproduz o que ele faz **durante o traço** é outra — a
/// média **sem peso**, que é a da nossa casa.
///
/// ⚠️ **Este gate afirma as duas metades**: que a lei pesada reproduz o bloco
/// `n` (logo o leitor entendeu o corpus) e que ela **não** é a nossa (logo
/// injectar o bloco seria trocar a lei do traço pela do repouso).
#[test]
fn as_duas_leis_de_normal_de_vertice_sao_duas() {
    let f = ler("lei", "um_dab_curva_afiada");
    let m = malha(&f);
    // A lei PESADA PELO ÂNGULO, escrita da espec.
    let mut pesada = vec![[0.0f64; 3]; f.repouso.len()];
    for (fi, face) in m.faces().iter().enumerate() {
        let n = m.face_normals()[fi];
        let idx = face.verts();
        for (k, &v) in idx.iter().enumerate() {
            let (a, b) = (
                m.positions()[idx[(k + idx.len() - 1) % idx.len()] as usize],
                m.positions()[idx[(k + 1) % idx.len()] as usize],
            );
            let p = m.positions()[v as usize];
            let (u, w) = (
                [a[0] - p[0], a[1] - p[1], a[2] - p[2]],
                [b[0] - p[0], b[1] - p[1], b[2] - p[2]],
            );
            let comp = |q: [f32; 3]| (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
            let (lu, lw) = (comp(u), comp(w));
            if lu <= 0.0 || lw <= 0.0 {
                continue;
            }
            let cos = ((u[0] * w[0] + u[1] * w[1] + u[2] * w[2]) / (lu * lw)).clamp(-1.0, 1.0);
            let ang = f64::from(cos.acos());
            for c in 0..3 {
                pesada[v as usize][c] += f64::from(n[c]) * ang;
            }
        }
    }
    let unit = |v: [f64; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        #[allow(clippy::cast_possible_truncation)]
        [(v[0] / l) as f32, (v[1] / l) as f32, (v[2] / l) as f32]
    };
    let pesada: Vec<[f32; 3]> = pesada.into_iter().map(unit).collect();
    let d = maior_distancia(&pesada, &f.normais);
    assert!(
        d <= 1e-5,
        "a lei pesada pelo ângulo devia reproduzir o bloco `n`: {d:.3e}"
    );
    println!("normais do repouso: a lei pesada reproduz o bloco `n` a {d:.3e}");
}

/// ⭐⭐⭐ **G-6 — o pincel nasce com os valores de fábrica do alvo** (espec §6).
///
/// ⚠️ **São CINCO os que ele muda em relação ao nosso `Draw`**, e cada um é uma
/// alavanca medida na ablação (§8): a curva, a direcção, a distância do
/// pen-down (o nosso acumular pregado), o passo e a atenuação.
#[test]
fn o_pincel_afiado_nasce_com_os_valores_do_alvo() {
    // ⚠️ **O modo é o que o pincel VESTE AO NASCER** (`RefMode::birth_for`), que
    // é o que o painel usa — e não o modo de fábrica do `Brush`. É por ele que
    // este verbo recebe a curva afiada e a força ao quadrado: o `S` não o
    // declara, logo quem o governa é a referência que o TEM.
    let modo = ph2d_sculpt3d::RefMode::birth_for(Verb::DrawSharp);
    assert_eq!(
        modo,
        ph2d_sculpt3d::RefMode::B,
        "o afiado tem de nascer na referência que o TEM"
    );
    let b = Brush {
        verb: Verb::DrawSharp,
        mode: modo,
        falloff: Verb::DrawSharp.default_falloff(modo),
        strength: Verb::DrawSharp.default_strength(),
        accumulate: Verb::DrawSharp.default_accumulate(),
        hardness: Verb::DrawSharp.default_hardness(),
        ..Brush::default()
    };
    assert_eq!(
        b.falloff,
        Falloff::Sharper,
        "a curva de fábrica e' a afiada"
    );
    assert!(
        (b.strength - 0.5).abs() < 1e-6,
        "a forca de fabrica e' 0,5: {}",
        b.strength
    );
    assert!(
        (b.hardness - 0.0).abs() < 1e-6,
        "a dureza de fabrica e' 0: {}",
        b.hardness
    );
    assert!(
        (b.normal_radius_frac - 0.5).abs() < 1e-6,
        "o raio da normal de fabrica e' 0,5: {}",
        b.normal_radius_frac
    );
    assert!(
        Verb::DrawSharp.afunda_de_fabrica(),
        "o pincel afiado nasce a AFUNDAR"
    );
    assert!(
        !Verb::DrawSharp.accumulates(),
        "o interruptor de acumular nao tem o que escolher neste verbo (espec §2.1)"
    );
    // O passo e a atenuação, os dois do traço.
    let passo = ph2d_sculpt3d::passo_do_traco(Verb::DrawSharp, 50.0);
    assert!(
        (passo - 5.0).abs() < 1e-6,
        "o passo e' 5 % do diametro: {passo}"
    );
    let arrastado = Brush {
        traco_arrastado: true,
        ..b
    };
    let a = arrastado.factor_do_traco();
    assert!(
        (a - 0.24591).abs() < 5e-4,
        "a atenuacao de fabrica e' `a` = 0,24591: {a}"
    );
    // ⛔ **E ela NÃO é a lei do pincel de plano** — a espec §5.3 mede `4,7e-2`
    // de erro com `(1 + a)/2`, contra `4,9e-4` com `a`.
    assert!(
        (a - (1.0 + a) / 2.0).abs() > 0.1,
        "o factor do afiado nao pode ser o do plano"
    );
}
