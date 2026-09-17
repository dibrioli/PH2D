//! **A bancada de paridade do pincel de POSE** — a nossa lei contra `69` traços
//! do oráculo, sobre malhas **nossas**.
//!
//! ⚠️⚠️ **A barra NÃO é um epsilon de conforto — ela sai da CLASSE da fixtura**
//! (espec §12.3), e as classes existem porque a lei tem descontinuidades reais:
//!
//! | classe | barra | porquê |
//! |---|---|---|
//! | **normal** | `1e-6` absoluto | `~10×` o arredondamento de `f32` observado |
//! | **sobre a descontinuidade** | ⛔ **não asserir paridade** | um `<` estrito não tem barra: a resposta **salta** |
//! | **perto do polo da escala** | relativa, `1e-4` do deslocamento máximo | o polo amplifica o arredondamento sem limite |
//! | **degenerada** | o **veredito** (deslocamento nulo), não números | o alvo devolve `0` exacto |
//!
//! ⛔ **Bit-parity não é a meta e não é prometida** (cerca da casa, ADR-0162).

use std::collections::BTreeMap;
use std::path::PathBuf;

use ph2d_pose::vetor::{V3, distancia};
use ph2d_pose::{Controlos, Modo, Pose, Vizinhanca};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/pose")
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

fn v3(campos: &[&str]) -> V3 {
    [
        campos[0].parse().expect("x"),
        campos[1].parse().expect("y"),
        campos[2].parse().expect("z"),
    ]
}

/// A malha de entrada. ⚠️ **Uma face tem TRÊS ou QUATRO índices** — quem assumir
/// um dos dois lê outra malha, e o erro é mudo.
struct Superficie {
    pos: Vec<V3>,
    faces: Vec<Vec<u32>>,
}

fn superficie(nome: &str) -> Superficie {
    let texto = inflar(&format!("{nome}.malha.txt.gz"));
    let (mut pos, mut faces) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos.first().copied() {
            Some("v") => pos.push(v3(&campos[1..])),
            Some("f") => faces.push(
                campos[1..]
                    .iter()
                    .map(|c| c.parse::<u32>().expect("indice"))
                    .collect(),
            ),
            _ => {}
        }
    }
    Superficie { pos, faces }
}

/// Um traço do oráculo: cabeçalho, percurso do cursor e posições finais.
struct Traco {
    chaves: BTreeMap<String, String>,
    caminho: Vec<V3>,
    depois: Vec<V3>,
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
            "p" => depois.push(v3(&campos[1..])),
            // ⚠️ **Uma linha de cabeçalho pode trazer MAIS DE UM par.** A das
            // máscaras traz dois (`mascara_valor` e `mascara_x_acima_de`), e um
            // leitor que só apanhe o primeiro perde o segundo **em silêncio** —
            // depois a fixtura corre sem máscara e lê-se como erro de LEI, que
            // foi exactamente o que aconteceu na primeira corrida desta bancada.
            _ => {
                for par in campos.chunks(2) {
                    if let [k, v] = par {
                        chaves.insert((*k).to_string(), (*v).to_string());
                    }
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
    fn b(&self, k: &str) -> bool {
        self.f(k) != 0.0
    }
}

fn constante(_p: f32) -> f32 {
    1.0
}

/// ⚠️ **Ler o cabeçalho, nunca uma lista de excepções** — cada fixtura traz, sem
/// excepção, as 19 grandezas que a definem. ⛔ E um valor desconhecido **PARA**
/// em vez de cair num default: uma curva lida como «a de omissão» por engano
/// mede outra lei e lê-se como erro nosso.
/// ⛔ Um valor desconhecido **PARA** em vez de cair na de omissão: uma curva
/// lida como «a de sempre» por engano mede outra lei e lê-se como erro nosso.
fn curva_de(t: &Traco) -> fn(f32) -> f32 {
    match t.s("curva") {
        "smooth" => ph2d_pose::suave,
        "constante" | "constant" => constante,
        c => panic!("curva desconhecida: {c}"),
    }
}

fn controlos(t: &Traco) -> Controlos {
    Controlos {
        modo: match t.s("modo") {
            "girar_torcer" => Modo::GirarTorcer,
            "escalar_transladar" => Modo::EscalarTransladar,
            "espremer_esticar" => Modo::EspremerEsticar,
            m => panic!("modo desconhecido: {m}"),
        },
        segmentos: t.f("segmentos") as u32,
        desvio_da_origem: t.f("desvio_da_origem"),
        suavizacoes_do_peso: t.f("suavizacoes_do_peso") as u32,
        ancorado: t.b("ancorado"),
        trava_rotacao: t.b("trava_rotacao"),
        // ⛔⛔ **A bancada corre SEMPRE a lei da espec, e nunca o que um cabecalho
        // diga:** o corpus foi gravado com a projeccao no osso, logo e' ela que a
        // paridade mede. O modo `Completo` e' uma divergencia DECLARADA do nosso
        // lado (ordem do dono, 17/09) e nao tem lado aprovado para comparar — pedi-lo
        // aqui seria medir outro pincel.
        lei_do_arrasto: ph2d_pose::Arrasto::AoLongoDoOsso,
        raio: t.f("raio"),
        forca: t.f("forca"),
        invertido: t.b("invertido"),
        simetria: [t.b("simetria_x"), false, false],
        so_conectado: t.b("so_conectado"),
        distancia_max_entre_pecas: t.f("distancia_max_entre_pecas"),
    }
}

/// O resultado de uma corrida: o pior desvio contra o oráculo, e as réguas
/// baratas que o cabeçalho já traz.
struct Corrida {
    pior: f32,
    raio: f32,
    max_deslocamento_nosso: f32,
    max_deslocamento_oraculo: f32,
    movidos_nossos: usize,
    movidos_oraculo: usize,
}

fn correr(nome: &str) -> Corrida {
    let t = traco(nome);
    let sup = superficie(t.s("superficie"));
    let ctrl = controlos(&t);
    assert_eq!(
        sup.pos.len(),
        t.depois.len(),
        "{nome}: contagem de vertices"
    );

    let escondido = vec![false; sup.pos.len()];
    let mut viz = Vizinhanca::construir(
        sup.pos.len(),
        sup.faces.iter().map(Vec::as_slice),
        &escondido,
    );
    if !ctrl.so_conectado {
        viz.ligar_pecas(&sup.pos, ctrl.distancia_max_entre_pecas);
    }

    // §9 — a máscara do corpus é um **semi-espaço**: `mascara_valor` nos
    // vértices com `x` acima de `mascara_x_acima_de`. ⚠️ Ela escala o
    // DESLOCAMENTO, não os pesos nem o pivô — e é isso que estas duas fixturas
    // medem (mesmo pivô, deslocamento máximo `0,192` e `0,108` contra `0,385`).
    let mascara: Option<Vec<f32>> = t.chaves.get("mascara_valor").map(|v| {
        let valor: f32 = v.parse().expect("mascara_valor");
        let limiar = t.f("mascara_x_acima_de");
        sup.pos
            .iter()
            .map(|p| if p[0] > limiar { valor } else { 0.0 })
            .collect()
    });
    let fatores = ph2d_pose::Fatores {
        mascara: mascara.as_deref(),
        ..Default::default()
    };

    let c0 = t.caminho[0];
    let eleito = ph2d_pose::cadeia::mais_proximo_global(&sup.pos, &escondido, c0)
        .expect("malha com vertices");
    let mut pose = Pose::comecar(&viz, &sup.pos, &escondido, eleito, c0, &ctrl);

    let ppu = t.f("pixels_por_unidade");
    let curva = curva_de(&t);
    let mut saida = Vec::new();
    for ponto in &t.caminho {
        let ev = ph2d_pose::Evento {
            arrasto: [ponto[0] - c0[0], ponto[1] - c0[1], ponto[2] - c0[2]],
            dx_pixels: (ponto[0] - c0[0]) * ppu,
        };
        pose.evento(&ctrl, &ev, &curva);
    }
    pose.posicoes(&ctrl, &sup.pos, fatores, &mut saida);

    let mut pior = 0.0f32;
    let (mut dn, mut do_, mut mn, mut mo) = (0.0f32, 0.0f32, 0usize, 0usize);
    for ((nossa, alvo), repouso) in saida.iter().zip(&t.depois).zip(&sup.pos) {
        let d = distancia(*nossa, *alvo);
        if d > pior {
            pior = d;
        }
        let nosso = distancia(*nossa, *repouso);
        let oraculo = distancia(*alvo, *repouso);
        dn = dn.max(nosso);
        do_ = do_.max(oraculo);
        if nosso > 1e-6 {
            mn += 1;
        }
        if oraculo > 1e-6 {
            mo += 1;
        }
    }
    Corrida {
        pior,
        raio: ctrl.raio,
        max_deslocamento_nosso: dn,
        max_deslocamento_oraculo: do_,
        movidos_nossos: mn,
        movidos_oraculo: mo,
    }
}

fn corpus() -> Vec<String> {
    let mut nomes: Vec<String> = std::fs::read_dir(fixture_dir())
        .expect("pasta de fixtures")
        .filter_map(|e| {
            let f = e.ok()?.file_name().to_string_lossy().into_owned();
            f.strip_suffix(".deformado.txt.gz").map(str::to_string)
        })
        .collect();
    nomes.sort();
    nomes
}

/// A corrida do corpus inteiro, **a imprimir a tabela**. Este é o instrumento;
/// os gates com barra vêm depois dele, quando a tabela disser onde pô-los.
#[test]
fn mede_o_corpus_contra_o_oraculo() {
    let nomes = corpus();
    assert!(
        nomes.len() >= 60,
        "piso de populacao: {} fixturas e' pouco — a pasta mudou?",
        nomes.len()
    );
    let mut linhas = Vec::new();
    for nome in &nomes {
        let r = correr(nome);
        linhas.push((r.pior, nome.clone(), r));
    }
    linhas.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!(
        "\n{:<52} {:>10} {:>10} {:>10} {:>7} {:>7}",
        "fixtura", "pior", "desl.nosso", "desl.orac", "mov.n", "mov.o"
    );
    for (pior, nome, r) in &linhas {
        println!(
            "{nome:<52} {pior:>10.3e} {:>10.4} {:>10.4} {:>7} {:>7}",
            r.max_deslocamento_nosso,
            r.max_deslocamento_oraculo,
            r.movidos_nossos,
            r.movidos_oraculo
        );
    }
    let dentro = linhas.iter().filter(|(p, _, _)| *p <= 1e-5).count();
    println!("\n{} de {} fixturas a <= 1e-5\n", dentro, linhas.len());
}

/// ⭐ **A sonda do vale**: o comprimento do 1.º segmento em TODO o corpus.
/// É dela que sai a barra do §11.1 — ⛔ nunca de um epsilon escolhido.
#[test]
fn sonda_o_vale_do_comprimento_do_primeiro_segmento() {
    let mut linhas = Vec::new();
    for nome in corpus() {
        let t = traco(&nome);
        let sup = superficie(t.s("superficie"));
        let ctrl = controlos(&t);
        let escondido = vec![false; sup.pos.len()];
        let viz = Vizinhanca::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        let c0 = t.caminho[0];
        let eleito =
            ph2d_pose::cadeia::mais_proximo_global(&sup.pos, &escondido, c0).expect("malha");
        let pose = Pose::comecar(&viz, &sup.pos, &escondido, eleito, c0, &ctrl);
        let s = &pose.cadeia().segmentos[0];
        let escala = s
            .cabeca_inicial
            .iter()
            .chain(s.origem_inicial.iter())
            .fold(0.0f32, |m, c| m.max(c.abs()));
        linhas.push((
            s.comprimento,
            s.comprimento / (escala.max(1.0) * f32::EPSILON),
            nome,
        ));
    }
    linhas.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (c, ulps, nome) in &linhas {
        println!("{c:>12.4e}  {ulps:>12.1} ulps  {nome}");
    }
}

/// A classe de paridade de uma fixtura (espec §12.3). ⛔ **Não é uma lista de
/// conforto:** cada entrada fora de `Normal` nomeia o MECANISMO que impede a
/// barra absoluta, e o censo abaixo reprova se alguma delas deixar de precisar.
#[derive(PartialEq, Debug)]
enum Classe {
    /// Barra **absoluta**. O corpo do corpus.
    Normal,
    /// Barra **relativa**: o quociente de escala amplifica o arredondamento da
    /// entrada sem que nenhum dos lados esteja errado.
    RelativaDaEscala,
    /// ⛔ **Paridade não asserível** — asserimos a FORMA: o mesmo conjunto de
    /// vértices movidos e o mesmo deslocamento máximo a `1 %`.
    SemParidade(&'static str),
    /// ⛔ **O veredito, não números:** o alvo devolve aqui um deslocamento
    /// **quase nulo**, e o resultado é descontínuo na posição do cursor. Uma
    /// banda relativa sobre um número que é ruído não afirmaria nada — o que se
    /// afirma é que os **dois lados** ficam colados a zero.
    QuaseNulo,
}

fn classe(nome: &str) -> Classe {
    match nome {
        // §11.2 — o polo ATRAVESSADO: o quociente muda de sinal, e um `<`
        // estrito não tem barra porque a resposta salta.
        "figura_escalar_dedo_atravessa_origem" => Classe::SemParidade("polo da escala atravessado"),
        // §11.3 — cursor exactamente sobre o plano de espelho: o alvo devolve
        // deslocamento quase nulo e o resultado é DESCONTÍNUO na posição do
        // cursor (deslocá-lo `1e-6` muda a saída em `2,4e-1`).
        "figura_girar_cabeca_no_plano_simetria_x" => Classe::QuaseNulo,
        // ⏳ **A única que a espec deixa POR EXPLICAR** (§12.3), com dois
        // mecanismos já eliminados por medição: ⛔ não é descontinuidade
        // (perturbar raio ou cursor em `1e-6` move a saída `≤ 2,4e-6`) e ⛔ não
        // é a confusão das duas sementes (apagar o pré-peso muda `0,000e+00`).
        "figura_girar_cabeca_no_plano_sem_simetria" => {
            Classe::SemParidade("desvio sistematico por explicar")
        }
        // §15 — a auto-suavização do alvo NÃO está modelada, de propósito: ela
        // só age dentro do raio inicial enquanto a deformação alcança muito mais
        // longe, e os autores registam-no como defeito. ⭐ Se a oferecermos, ela
        // segue os PESOS, não o raio.
        "figura_girar_dedo_suavizacao05" => Classe::SemParidade("auto-suavizacao nao modelada"),
        // ⛔⛔ **NÃO classificar por padrão de NOME.** A primeira redacção deste
        // predicado dizia `contains("escalar") || contains("esticar")` e arrastou
        // **12** fixturas para a barra relativa quando só **8** precisam dela —
        // as outras quatro passariam a ter uma folga de `100×` que ninguém
        // pediu. *Uma classe definida por padrão é uma licença com cara de
        // mecanismo*, e foi o censo de obsolescência que a apanhou.
        "figura_escalar_dedo_diagonal"
        | "figura_escalar_dedo_diagonal_solto"
        | "figura_escalar_dedo_diagonal_travado"
        | "figura_escalar_dedo_esticar"
        | "figura_escalar_dedo_simetria_x"
        | "figura_esticar_dedo"
        | "figura_esticar_dedo_invertido"
        | "braco3d_esticar" => Classe::RelativaDaEscala,
        _ => Classe::Normal,
    }
}

/// A barra absoluta do corpo do corpus.
///
/// ⭐ **`1e-6`, e o número sai da medição:** o corpo fecha entre `5,96e-8` e
/// `6,74e-7` — a barra é `~1,5×` o pior medido e `~10×` o arredondamento de
/// `f32` típico. ⛔ Não é bit-parity, que o ADR-0162 recusa prometer em T2.
const BARRA_ABSOLUTA: f32 = 1e-6;
/// A barra relativa da família da escala: medido `≤ 1,7e-5` do deslocamento.
const BARRA_RELATIVA: f32 = 1e-4;

#[test]
fn a_paridade_fecha_por_classe() {
    let mut falhas = Vec::new();
    for nome in corpus() {
        let r = correr(&nome);
        let ok = match classe(&nome) {
            Classe::Normal => r.pior <= BARRA_ABSOLUTA,
            Classe::RelativaDaEscala => {
                r.pior <= BARRA_RELATIVA * r.max_deslocamento_oraculo.max(1e-6)
            }
            // ⚠️ Sem paridade **não é sem asserção**: exigimos que os DOIS lados
            // movam o mesmo conjunto de vértices e que o deslocamento máximo
            // concorde a `1 %` — o que apanha uma lei trocada sem prometer um
            // número que a descontinuidade não pode dar.
            Classe::SemParidade(_) => {
                r.movidos_nossos == r.movidos_oraculo
                    && (r.max_deslocamento_nosso - r.max_deslocamento_oraculo).abs()
                        <= 0.01 * r.max_deslocamento_oraculo.max(1e-3)
            }
            // O alvo mede `8e-4` sobre um raio de `0,25`; nós `1,0e-3`. A barra
            // é `1 %` do raio — ⭐ derivada do que «quase nulo» quer dizer
            // naquela peça, não do número que calhou sair.
            Classe::QuaseNulo => {
                let piso = 0.01 * r.raio;
                r.max_deslocamento_nosso <= piso && r.max_deslocamento_oraculo <= piso
            }
        };
        if !ok {
            falhas.push(format!(
                "{nome}: pior={:.3e} nosso={:.4} oraculo={:.4} mov={}/{} classe={:?}",
                r.pior,
                r.max_deslocamento_nosso,
                r.max_deslocamento_oraculo,
                r.movidos_nossos,
                r.movidos_oraculo,
                classe(&nome)
            ));
        }
    }
    assert!(falhas.is_empty(), "paridade:\n  {}", falhas.join("\n  "));
}

/// ⛔⛔ **O CENSO DE OBSOLESCÊNCIA da lista de excepções.**
///
/// *Uma catraca sem censo de obsolescência não desce: ela vira LICENÇA.* Este
/// gate reprova nos **dois** sentidos: se o corpus encolher (piso de população)
/// e se alguma fixtura fora de `Normal` passar a caber na barra absoluta — nesse
/// caso a linha dela tem de ser **apagada**, não mantida «por segurança».
#[test]
fn o_censo_das_excepcoes_nao_descreve_nada_de_obsoleto() {
    let nomes = corpus();
    assert_eq!(nomes.len(), 69, "o corpus tem 69 tracos publicados");
    let mut obsoletas = Vec::new();
    let mut n_normal = 0;
    for nome in &nomes {
        if classe(nome) == Classe::Normal {
            n_normal += 1;
            continue;
        }
        let r = correr(nome);
        if r.pior <= BARRA_ABSOLUTA {
            obsoletas.push(format!("{nome} ({:.3e})", r.pior));
        }
    }
    assert!(
        n_normal >= 57,
        "o corpo do corpus encolheu: {n_normal} fixturas normais"
    );
    assert!(
        obsoletas.is_empty(),
        "estas ja cabem na barra absoluta — APAGUE a linha delas em `classe()`:\n  {}",
        obsoletas.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// A SONDA DO INDICADOR (o «osso» que se desenha antes de premir)
// ---------------------------------------------------------------------------

/// ⭐⭐ **O que uma construção custa, e quanto o osso ANDA quando o cursor anda.**
///
/// ⚠️ **É uma SONDA, não um gate** — ela imprime a tabela de que sai o limiar
/// com que o indicador decide reconstruir-se, e o número tem de ser lido no
/// perfil em que o produto corre. `#[ignore]` porque a célula do pior caso
/// (`20` segmentos × `100` suavizações) é cara de propósito.
///
/// ```text
/// cargo test -p ph2d-pose --test it -- --ignored --nocapture mede_o_indicador
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do custo e da sensibilidade do indicador"]
fn mede_o_indicador_da_pose() {
    let mut malhas: Vec<String> = corpus()
        .iter()
        .map(|n| traco(n).s("superficie").to_string())
        .collect();
    malhas.sort();
    malhas.dedup();

    println!("\n=== CUSTO DE UMA CONSTRUCAO (ms) ===");
    println!(
        "{:<24} {:>7} {:>10} {:>10} {:>10}",
        "malha", "verts", "default", "seg20", "seg20s100"
    );
    for nome in &malhas {
        let sup = superficie(nome);
        let escondido = vec![false; sup.pos.len()];
        let viz = Vizinhanca::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        // Um cursor fora do centro: perto da ponta mais afastada do centroide,
        // que é onde um artista de facto põe o pincel deste verbo.
        let cursor = ponta_da_peca(&sup.pos);
        let eleito = ph2d_pose::cadeia::mais_proximo_global(&sup.pos, &escondido, cursor)
            .expect("malha com vertices");
        let mut linha = format!("{:<24} {:>7}", nome, sup.pos.len());
        for ctrl in [
            Controlos::default(),
            Controlos {
                segmentos: 20,
                ..Default::default()
            },
            Controlos {
                segmentos: 20,
                suavizacoes_do_peso: 100,
                ..Default::default()
            },
        ] {
            let t0 = std::time::Instant::now();
            let pose = Pose::comecar(&viz, &sup.pos, &escondido, eleito, cursor, &ctrl);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            std::hint::black_box(&pose);
            linha.push_str(&format!(" {ms:>10.2}"));
        }
        println!("{linha}");
    }

    println!("\n=== SENSIBILIDADE: quanto o OSSO anda por unidade de CURSOR ===");
    println!(
        "{:<24} {:>8} {:>10} {:>10} {:>10}",
        "malha", "f", "|dcursor|", "|dosso|max", "razao"
    );
    for nome in &malhas {
        let sup = superficie(nome);
        let escondido = vec![false; sup.pos.len()];
        let viz = Vizinhanca::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        let ctrl = Controlos::default();
        let cursor = ponta_da_peca(&sup.pos);
        let base = ossos_em(&viz, &sup.pos, &escondido, cursor, &ctrl);
        for f in [0.01f32, 0.02, 0.05, 0.10] {
            let d = ctrl.raio * f;
            let mut pior = 0.0f32;
            for eixo in 0..3 {
                for sinal in [-1.0f32, 1.0] {
                    let mut c = cursor;
                    c[eixo] += sinal * d;
                    let outro = ossos_em(&viz, &sup.pos, &escondido, c, &ctrl);
                    for (a, b) in base.iter().zip(&outro) {
                        pior = pior.max(distancia(a[0], b[0])).max(distancia(a[1], b[1]));
                    }
                }
            }
            println!(
                "{:<24} {:>8.2} {:>10.4} {:>10.4} {:>10.2}",
                nome,
                f,
                d,
                pior,
                pior / d.max(1e-9)
            );
        }
    }
}

/// Um ponto sobre a peça longe do centroide — onde a franja é assimétrica e o
/// pivô é empurrado para dentro (§11.1: no meio de uma superfície lisa a cadeia
/// nasce inerte e a sonda mediria o nada).
fn ponta_da_peca(pos: &[V3]) -> V3 {
    let n = pos.len() as f32;
    let mut c = [0.0f32; 3];
    for p in pos {
        for (a, b) in c.iter_mut().zip(p) {
            *a += b / n;
        }
    }
    *pos.iter()
        .max_by(|a, b| distancia(**a, c).total_cmp(&distancia(**b, c)))
        .expect("malha com vertices")
}

fn ossos_em(
    viz: &Vizinhanca,
    pos: &[V3],
    escondido: &[bool],
    cursor: V3,
    ctrl: &Controlos,
) -> Vec<[V3; 2]> {
    let eleito =
        ph2d_pose::cadeia::mais_proximo_global(pos, escondido, cursor).expect("malha com vertices");
    let pose = Pose::comecar(viz, pos, escondido, eleito, cursor, ctrl);
    let mut saida = Vec::new();
    pose.ossos(ctrl, &mut saida);
    saida
}

/// ⭐⭐ **O que a lei faz quando o DESVIO DA ORIGEM passa o tecto do alvo.**
///
/// O alvo oferece `0..2` (§1.1) e o dono pediu `3` (2026-09-14). ⚠️ *Um tecto
/// que se sobe sem medir é um palpite à espera de um smoke* — esta sonda mede o
/// que há do outro lado: o comprimento do 1.º segmento (o braço da alavanca),
/// o deslocamento máximo de um arrasto fixo, e o que a construção custa.
///
/// ```text
/// cargo test -p ph2d-pose --test it -- --ignored --nocapture sonda_o_desvio
/// ```
#[test]
#[ignore = "sonda: o que ha' do outro lado do tecto do desvio da origem"]
fn sonda_o_desvio_da_origem_alem_do_tecto() {
    let mut malhas: Vec<String> = corpus()
        .iter()
        .map(|n| traco(n).s("superficie").to_string())
        .collect();
    malhas.sort();
    malhas.dedup();

    println!(
        "\n{:<24} {:>6} {:>10} {:>12} {:>12} {:>8}",
        "malha", "desvio", "comp. seg0", "desloc. max", "construcao", "finito"
    );
    for nome in &malhas {
        let sup = superficie(nome);
        let escondido = vec![false; sup.pos.len()];
        let viz = Vizinhanca::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        let cursor = ponta_da_peca(&sup.pos);
        let eleito = ph2d_pose::cadeia::mais_proximo_global(&sup.pos, &escondido, cursor)
            .expect("malha com vertices");
        for desvio in [0.0f32, 1.0, 2.0, 2.5, 3.0, 4.0] {
            let ctrl = Controlos {
                desvio_da_origem: desvio,
                ..Default::default()
            };
            let t0 = std::time::Instant::now();
            let mut pose = Pose::comecar(&viz, &sup.pos, &escondido, eleito, cursor, &ctrl);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let seg0 = pose.cadeia().segmentos[0].comprimento;
            pose.evento(
                &ctrl,
                &ph2d_pose::Evento {
                    arrasto: [0.0, 0.2, 0.0],
                    dx_pixels: 0.0,
                },
                &ph2d_pose::suave,
            );
            let mut saida = Vec::new();
            pose.posicoes(&ctrl, &sup.pos, Default::default(), &mut saida);
            let mut pior = 0.0f32;
            let mut finito = true;
            for (a, b) in sup.pos.iter().zip(&saida) {
                finito &= b.iter().all(|c| c.is_finite());
                pior = pior.max(distancia(*a, *b));
            }
            println!(
                "{nome:<24} {desvio:>6.1} {seg0:>10.4} {pior:>12.4} {ms:>10.2} ms {finito:>8}"
            );
        }
    }
}
