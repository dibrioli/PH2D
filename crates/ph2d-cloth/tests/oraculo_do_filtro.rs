//! ⭐⭐⭐ **A BANCADA DO FILTRO DE TECIDO** — a nossa lei contra as corridas do
//! oráculo, sobre malhas NOSSAS
//! ([`docs/3D/cleanroom/fixtures/cloth/filtro/`](../../../docs/3D/cleanroom/fixtures/cloth/README.md)).
//!
//! ⚠️ **As fixtures são DADOS, não expressão** (GPLv2 §0: a saída do programa só
//! é obra derivada se o CONTEÚDO dela o for, e posições de vértices de uma malha
//! nossa não são).
//!
//! # Por que ela é um ficheiro PRÓPRIO, e não mais um bloco no arnês do pincel
//!
//! ⛔ **Um traço de filtro não é corrível pelo arnês do pincel** — não há pincel,
//! nem raio, nem banda, nem caminho de cursor sobre a peça. O corpus dele vive
//! num **subdirectório** (`filtro/`) exactamente por isso: o censo do arnês do
//! pincel exige que as duas listas dele descrevam o corpus inteiro do
//! directório, e ele varre a raiz **sem recursão**. Pôr um traço de filtro na
//! raiz deixaria aquele censo vermelho ou obrigaria a inscrevê-lo numa lista
//! onde ele não pode ser medido.
//!
//! ⇒ **o corpus do filtro tem censo PRÓPRIO**, e ele está aqui
//! ([`o_censo_do_corpus_do_filtro`]).
//!
//! # O que o corpus fixa
//!
//! A lei da força fecha ao último dígito (espec §10.17): com escala de UI `1`,
//! força `1` e avanço de `90` px por passo, `Σ_j (k−j+1)·S_j·Δt` prevê
//! `0,108000` ao fim de oito passos e o oráculo entrega `0.108000`. ⭐ **No plano
//! as corridas são bit-reprodutíveis** (dispersão `0,000000` em duas
//! realizações), o que faz do plano o sítio de toda régua exacta; a **esfera** é
//! a única que sorteia (`0,001015`), e esse número tem de entrar em qualquer
//! barra que a use.
//!
//! # ⭐⭐⭐ As INVOCAÇÕES REPETIDAS (emenda Q23, 2026-09-09 — espec §10.18)
//!
//! O corpus passou de `17` para `27` corridas, e as dez novas medem o que
//! atravessa **um gesto**: o filtro aberto e largado `3` vezes seguidas sobre a
//! mesma peça. ⚠️ **A simulação nasce e morre com cada invocação** (espec §6.3),
//! logo a seguinte constrói as restrições da malha **deformada** — e é por isso
//! que a deformação compõe.
//!
//! ⭐⭐ **Três delas são um trio, e só juntas decidem:** a que compõe, a que lê a
//! **base persistente** (e por isso **RECUA** na 3.ª invocação — `0,2125` para
//! `0,1806`) e o **controlo** da opção ligada com a base por gravar, que sai
//! byte a byte igual à de opção desligada. *Sem o controlo, a leitura ingénua
//! atribuiria o efeito ao interruptor em vez de à base.*
//!
//! ⛔⛔ **E as duas de esfera têm um piso que não é nosso** — a dispersão do
//! próprio alvo entre realizações: `0,003184` a 24 passos e **`0,073751`** a 36
//! (`5,6 %` do máximo dele). *Uma barra abaixo disso mede o sorteio do oráculo.*

use ph2d_cloth::V3;
use ph2d_cloth::verlet::Solver;
use ph2d_cloth::verlet_gesto::{
    Accionamento, Area, Modo, Passo, Pincel, PincelTecido, Referencial,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/cloth")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore.
///
/// ⚠️ **É a gémea da do arnês do pincel, e a duplicação é DELIBERADA:** as duas
/// lêem o mesmo formato, e uma divergência entre elas aparece como uma falha de
/// leitura ALTA (o parse rebenta), nunca como uma resposta errada em silêncio —
/// que é o critério que separa uma cópia tolerável de uma segunda fonte da
/// verdade. ⛔ Partilhá-las obrigaria a editar o arnês de `86` traços que já
/// shipa, e o risco disso é maior que o desta cópia de trinta linhas.
fn inflar(caminho: &str) -> String {
    let raw =
        std::fs::read(fixture_dir().join(caminho)).unwrap_or_else(|e| panic!("{caminho}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{caminho}: nao e' gzip"
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
        .unwrap_or_else(|e| panic!("{caminho}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

fn v3(campos: &[&str]) -> V3 {
    [
        campos[0].parse().expect("x"),
        campos[1].parse().expect("y"),
        campos[2].parse().expect("z"),
    ]
}

fn repouso(superficie: &str) -> Vec<V3> {
    inflar(&format!("{superficie}.repouso.txt.gz"))
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| v3(&l.split_whitespace().skip(1).collect::<Vec<_>>()))
        .collect()
}

/// As faces do ALVO — a ordem delas é a ordem do anel (espec §3.1).
fn faces(superficie: &str) -> Vec<Vec<u32>> {
    let mut fs = Vec::new();
    for l in inflar(&format!("{superficie}.faces.txt.gz")).lines() {
        if let Some(resto) = l.strip_prefix("f ") {
            fs.push(
                resto
                    .split_whitespace()
                    .map(|t| t.parse().expect("indice"))
                    .collect(),
            );
        }
    }
    assert!(!fs.is_empty(), "{superficie}.faces sem faces");
    fs
}

/// A ordem de visita da malha (espec §3.1-bis) — ela fixa a ordem da lista de
/// restrições, e Gauss-Seidel não comuta.
fn ordem_de_visita(superficie: &str) -> Vec<u32> {
    let mut ordem = Vec::new();
    for l in inflar(&format!("{superficie}.celulas.txt.gz")).lines() {
        if let Some(resto) = l.strip_prefix("cv ") {
            ordem.extend(
                resto
                    .split_whitespace()
                    .skip(2)
                    .map(|t| t.parse::<u32>().expect("indice de vertice")),
            );
        }
    }
    assert!(!ordem.is_empty(), "{superficie}.celulas sem proprios");
    ordem
}

fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(u32::try_from(q).unwrap_or(u32::MAX));
            a[q].push(u32::try_from(p).unwrap_or(u32::MAX));
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// Uma corrida do filtro.
struct Corrida {
    chaves: BTreeMap<String, String>,
    caminho: Vec<V3>,
    depois: Vec<V3>,
}

impl Corrida {
    fn s(&self, k: &str) -> &str {
        self.chaves.get(k).map_or("", String::as_str)
    }
    fn f(&self, k: &str) -> f64 {
        self.s(k).parse().unwrap_or_else(|_| panic!("chave {k}"))
    }
    /// Uma chave INTEIRA com omissão — as chaves da emenda Q23 (`invocacoes`,
    /// `opcao_persistente`) não existem nas `17` corridas do §10.17, e a omissão
    /// delas é o que aquelas corridas afirmam de si próprias.
    fn i(&self, k: &str, omissao: usize) -> usize {
        match self.chaves.get(k) {
            Some(v) => v.parse().unwrap_or_else(|_| panic!("chave {k}")),
            None => omissao,
        }
    }
    /// A lista `S₁..S_n` do cabeçalho — ⭐ **ela existe para o arrasto ser
    /// reconstruível sem adivinhar** o píxel em que o botão foi premido.
    fn forcas(&self) -> Vec<f64> {
        self.s("forca_por_passo")
            .split(',')
            .map(|t| t.parse().expect("forca por passo"))
            .collect()
    }
}

fn corrida(nome: &str) -> Corrida {
    let texto = inflar(&format!("filtro/{nome}.deformado.txt.gz"));
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
    Corrida {
        chaves,
        caminho,
        depois,
    }
}

/// **A MÁSCARA, do CABEÇALHO** — ela é uma REGRA, não um bloco de dados.
///
/// ⚠️ *Uma máscara lida da saída seria circular* (mediria a nossa lei contra o
/// que ela própria moveu). O cabeçalho declara a regra, e a contagem de vértices
/// livres é conferida contra a chave `movidos` do oráculo — que é o controlo.
fn mascara(regra: &str, rest: &[V3]) -> Vec<f64> {
    if regra == "nenhuma" || regra.is_empty() {
        return Vec::new();
    }
    let raio: f64 = regra
        .strip_prefix("disco_exterior_")
        .unwrap_or_else(|| panic!("regra de mascara desconhecida: {regra}"))
        .parse()
        .expect("raio do disco");
    rest.iter()
        .map(|p| {
            // ⚠️ O disco é medido no PLANO da folha (`xy`): a regra nomeia um
            // disco, e a peça é plana em `z`.
            let d = (p[0] * p[0] + p[1] * p[1]).sqrt();
            if d > raio { 1.0 } else { 0.0 }
        })
        .collect()
}

/// **O REFERENCIAL do traço**, já em coordenadas de mundo.
///
/// ⚠️ **O harness filma a folha de frente**: o `+X` do ecrã é o `+X` do mundo e o
/// `+Y` do ecrã é o `+Y` do mundo, com a profundidade em `Z`. É por isso que a
/// orientação *View* põe o «baixo» da gravidade em `−Y` e a *Local* em `−Z` — as
/// duas fixtures que separam isso são o `_local` e o `_vista`.
fn referencial(orientacao: &str, eixos: &str) -> (V3, Referencial) {
    let activo = match eixos {
        "xyz" => [true; 3],
        "x" => [true, false, false],
        "y" => [false, true, false],
        "z" => [false, false, true],
        e => panic!("eixos desconhecidos: {e}"),
    };
    let base = Referencial {
        eixos: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        activo,
    };
    match orientacao {
        "local" | "mundo" => ([0.0, 0.0, -1.0], base),
        "vista" => ([0.0, -1.0, 0.0], base),
        o => panic!("orientacao desconhecida: {o}"),
    }
}

fn modo_de(filtro: &str) -> Modo {
    match filtro {
        "gravidade" => Modo::Gravidade,
        "inflar" => Modo::Inflar,
        "expandir" => Modo::Expandir,
        "apertar" => Modo::ApertarPonto,
        "escala" => Modo::Escala,
        f => panic!("tipo de filtro desconhecido: {f}"),
    }
}

/// Normais por vértice das posições ACTUAIS — o filtro refresca-as a cada passo
/// (espec §7 contra §4.2-ter), e é isso que separa a lei dele da do traço.
fn normais(pos: &[V3], faces: &[Vec<u32>]) -> Vec<V3> {
    let mut n = vec![[0.0; 3]; pos.len()];
    for f in faces {
        let (a, b, c) = (pos[f[0] as usize], pos[f[1] as usize], pos[f[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fnv = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        for &i in f {
            for k in 0..3 {
                n[i as usize][k] += fnv[k];
            }
        }
    }
    for v in &mut n {
        let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if m > 1e-12 {
            for c in v.iter_mut() {
                *c /= m;
            }
        }
    }
    n
}

/// Corre a NOSSA lei sobre a mesma entrada, e devolve `(nosso, oráculo, repouso)`.
fn correr(nome: &str) -> (Vec<V3>, Vec<V3>, Vec<V3>) {
    let c = corrida(nome);
    let sup = c.s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup);
    let an = aneis(rest.len(), &fs);
    assert_eq!(
        rest.len(),
        c.depois.len(),
        "{nome}: a fixture e o repouso tem contagens diferentes"
    );
    let (eixo_g, refer) = referencial(c.s("orientacao"), c.s("eixos"));
    let pincel = Pincel {
        modo: modo_de(c.s("filtro")),
        area: Area::Global,
        forca: 1.0,
        dureza: 0.0,
        pino: false,
        accionamento: Accionamento::Filtro { s: 0.0 },
        eixo_da_gravidade: eixo_g,
        referencial: refer,
        solver: Solver {
            massa: c.f("massa"),
            amortecimento: c.f("amortecimento"),
            plasticidade: 0.0,
            ..Solver::default()
        },
        ..Pincel::default()
    };
    // ⭐ O ponto do aperto é o da ABERTURA — o 1.º ponto do caminho — e ele **não
    // segue o cursor** (espec §7). Os outros quatro tipos não o lêem.
    let abertura = abertura_de(&c, &rest);
    let mut pos = rest.clone();
    let anel = |v: u32| an[v as usize].clone();
    // ⭐⭐⭐ **AS INVOCAÇÕES REPETIDAS** (espec §10.18) — o filtro é aberto e largado
    // `n` vezes seguidas sobre a MESMA peça.
    //
    // ⚠️⚠️ **A simulação nasce e morre com cada uma** (espec §6.3): a invocação
    // seguinte constrói as restrições **da malha deformada**, e é por isso que a
    // deformação compõe. ⛔ Reaproveitar a sessão mediria outro programa.
    //
    // ⭐ **A base persistente é a excepção, e é UM campo:** com a opção ligada e a
    // base gravada no repouso, a construção lê aquelas posições em vez das de
    // agora — as quatro leituras da §6.4, que na nossa lei são o
    // [`ph2d_cloth::verlet::Verlet::base`].
    let invocacoes = c.i("invocacoes", 1);
    let com_base = c.i("opcao_persistente", 0) == 1 && c.s("base_gravada") == "repouso";
    let ordem = ordem_de_visita(&sup);
    for _ in 0..invocacoes {
        let mut t = PincelTecido::pen_down(pincel, &pos, abertura, ordem.clone());
        if com_base {
            t.sim.base.clone_from(&rest);
        }
        t.mascara = mascara(c.s("mascara"), &rest);
        // ⭐⭐⭐ **A CARGA**: a espec §7 constrói as restrições «ao carregar», e a lista
        // de fases do passo dela começa na **fase 2**. O 1.º `passo` faz essa
        // construção e devolve `false`; sem ele o 1.º movimento do rato não simulava.
        let n0 = normais(&pos, &fs);
        let carga = Passo {
            cursor: abertura,
            delta: [0.0; 3],
            delta_3d: [0.0; 3],
            parado: false,
            vista: [0.0, 0.0, 1.0],
            normais: &n0,
            pressao: 1.0,
        };
        assert!(
            !t.passo(&pos, &anel, &carga),
            "{nome}: a carga nao pode simular"
        );
        for s in c.forcas() {
            t.pincel.accionamento = Accionamento::Filtro { s };
            let n = normais(&pos, &fs);
            let p = Passo {
                cursor: abertura,
                delta: [0.0; 3],
                delta_3d: [0.0; 3],
                parado: false,
                vista: [0.0, 0.0, 1.0],
                normais: &n,
                pressao: 1.0,
            };
            if t.passo(&pos, &anel, &p) {
                pos.copy_from_slice(&t.sim.x);
            }
        }
    }
    (pos, c.depois, rest)
}

/// **O PONTO DA ABERTURA** — onde o botão foi premido, EXTRAPOLADO do caminho.
///
/// ⚠️⚠️ **Ele NÃO está no ficheiro, e a razão é o gesto:** os `c` gravados são as
/// posições do cursor **depois de cada movimento**, e o clique de abertura é
/// anterior ao primeiro deles. Com avanço constante (`avanco_por_passo_px`), o
/// ponto premido é `c₀ − (c₁ − c₀)` — e no corpus isso dá `x ≈ 0`, o centro da
/// folha, que é onde o harness abriu o filtro.
///
/// ⭐ **Depois ele ENCOSTA no vértice mais próximo:** a espec §7 diz que o alvo do
/// aperto é o **vértice activo** no instante da abertura, e não um ponto solto no
/// espaço. *A diferença é pequena e não é ruído — é a diferença entre apertar
/// para um ponto da malha e apertar para um ponto que não pertence a ela.*
fn abertura_de(c: &Corrida, rest: &[V3]) -> V3 {
    let p = match (c.caminho.first(), c.caminho.get(1)) {
        (Some(a), Some(b)) => [
            a[0] - (b[0] - a[0]),
            a[1] - (b[1] - a[1]),
            a[2] - (b[2] - a[2]),
        ],
        (Some(a), None) => *a,
        _ => [0.0; 3],
    };
    let mut melhor = (f64::INFINITY, p);
    for q in rest {
        let d = norma([q[0] - p[0], q[1] - p[1], q[2] - p[2]]);
        if d < melhor.0 {
            melhor = (d, *q);
        }
    }
    melhor.1
}

fn norma(v: V3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// **O ERRO de um traço**: o pior desvio por vértice, em unidades do maior
/// deslocamento que o ORÁCULO produziu.
///
/// ⚠️ **Adimensional de propósito** — traços com forças e superfícies diferentes
/// ficam comparáveis, e é a mesma régua que o arnês do pincel usa.
fn erro(nome: &str) -> (f64, f64) {
    let (nosso, oraculo, rest) = correr(nome);
    let escala = oraculo
        .iter()
        .zip(&rest)
        .map(|(a, b)| norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0, f64::max);
    let pior = nosso
        .iter()
        .zip(&oraculo)
        .map(|(a, b)| norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0, f64::max);
    let nosso_max = nosso
        .iter()
        .zip(&rest)
        .map(|(a, b)| norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        .fold(0.0, f64::max);
    if std::env::var("PH2D_FILTRO_COLHEITA").is_ok() {
        println!("    (nosso maximo {nosso_max:.6})");
    }
    (pior / escala.max(1e-12), escala)
}

// ————————————————————————————— o corpus —————————————————————————————

/// **O CORPUS CORRÍVEL, com a barra MEDIDA de cada traço.**
///
/// ⛔⛔ **Estes números foram IMPRESSOS primeiro e escritos depois** (`CLAUDE.md`
/// §0.9). Quem mexer na lei re-mede e re-escreve — ⛔ **não afrouxa**.
///
/// ⭐ **Seis traços saem a `1e-9`**, que é dizer *ao bit*: a lei da força do
/// filtro fecha ao último dígito contra o oráculo (`0.108000` previsto e
/// medido), a massa `2` dá metade exacta, a força `0` não move um vértice, e as
/// bandeiras de eixo **não tocam** nas forças (byte a byte igual ao `_local`).
///
/// ⚠️ **A barra da ESFERA tem de ficar acima do sorteio DELA** — o oráculo
/// dispersa `0,001015` sobre uma escala de `0,095222`, ou seja **`0,0107`
/// relativos**, entre quatro realizações da MESMA configuração. Uma barra abaixo
/// disso mediria a lotaria do alvo e chamar-lhe-ia defeito nosso.
const CORPUS: &[(&str, f64)] = &[
    // ⭐ ao bit
    ("plano_filtro_gravidade_local", 1e-9),
    ("plano_filtro_gravidade_massa2", 1e-9),
    ("plano_filtro_gravidade_forca0", 1e-9),
    ("plano_filtro_gravidade_eixox", 1e-9),
    ("plano_filtro_inflar", 1e-9),
    ("plano_filtro_inflar_1passo", 1e-9),
    // à resolução do ficheiro (seis decimais ⇒ piso de arredondamento `5e-7`)
    ("plano_filtro_gravidade_vista", 2e-5),
    ("plano_filtro_gravidade_mascarado", 2e-5),
    // dentro da barra
    ("plano_filtro_escala", 0.009),
    ("plano_filtro_escala_eixox", 0.011),
    ("plano_filtro_apertar", 0.012),
    ("plano_filtro_inflar_mascarado", 0.015),
    ("plano_filtro_expandir_negativo", 0.022),
    // ⚠️ acima do sorteio do próprio oráculo (`0,0107`), e é a única curva
    ("esfera_filtro_inflar", 0.035),
    // ————— as INVOCAÇÕES REPETIDAS (espec §10.18, emenda Q23 de 09/09) —————
    // ⭐ ao bit: o *Repeat* que a família de filtros regista é MORTO neste filtro
    // (o oráculo dá `5` byte a byte igual a `1`), e a nossa lei não o tem —
    // logo a corrida dele tem de bater com a de uma repetição.
    ("plano_filtro_gravidade_repeticoes5", 1e-9),
    // à resolução do ficheiro — ⭐⭐⭐ e as TRÊS juntas são a prova de que a nossa
    // composição de gestos é a do alvo: a que compõe, a que lê a BASE
    // PERSISTENTE (e por isso RECUA na 3.ª invocação) e o controlo da opção
    // ligada sem base gravada.
    ("plano_filtro_gravidade_mascarado_3invocacoes", 2e-5),
    (
        "plano_filtro_gravidade_mascarado_3invocacoes_persistente",
        2e-5,
    ),
    (
        "plano_filtro_gravidade_mascarado_3invocacoes_persistente_sem_base",
        2e-5,
    ),
    // dentro da barra — ⚠️ a barra de cada uma é o erro MEDIDO com margem, e ela
    // é RELATIVAMENTE mais apertada que a da irmã de uma invocação
    // (`plano_filtro_inflar_mascarado` erra `0,0134` sobre uma escala de
    // `0,1088`, e esta erra `0,0190` sobre `0,3206` — metade, em proporção).
    ("plano_filtro_escala_3invocacoes", 0.009),
    (
        "plano_filtro_inflar_mascarado_3invocacoes_persistente",
        0.010,
    ),
    ("plano_filtro_inflar_mascarado_3invocacoes", 0.021),
    // ⚠️⚠️ **AS DUAS DE ESFERA TÊM UM PISO QUE NÃO É NOSSO: a DISPERSÃO do
    // próprio oráculo.** Quatro realizações da mesma configuração diferem entre
    // si até `0,003184` (24 passos) e **`0,073751`** (36 passos, `5,6 %` do
    // próprio máximo). ⛔ Uma barra abaixo disso mediria o sorteio do alvo, não
    // a nossa lei.
    ("esfera_filtro_inflar_3invocacoes", 0.028),
    // ⭐ `0,0946` medido contra um piso de `0,0738` — ou seja **`1,28×` o ruído
    // de realização do próprio alvo**, na corrida em que ele estica `21,55×`.
    ("esfera_filtro_inflar_36passos", 0.100),
];

/// **O QUE O CORPUS TEM E A NOSSA LEI AINDA NÃO REPRODUZ**, com a razão e o
/// número de cada um.
///
/// ⚠️ **Dois deles não são defeitos: são AUSÊNCIAS declaradas.** A escultura
/// desta casa não tem gravidade de cena nem conjuntos de faces, então não há de
/// onde o valor viria — *uma lei sem entrada não pode ser medida*.
const ABERTOS: &[(&str, &str)] = &[
    (
        "plano_filtro_expandir_3invocacoes",
        "erro 0,382781 -- e' o MESMO defeito do irmao de uma invocacao, composto tres vezes: \
         a quantidade esta' certa (o nosso maximo contra o do oraculo) e o que difere e' o \
         PADRAO da flambagem. ⚠️ Ele entra aqui e nao no corpus porque curar o de uma \
         invocacao cura este -- sao um item, nao dois",
    ),
    (
        "plano_filtro_expandir",
        "⭐⭐ erro 0,688534 e a QUANTIDADE esta' certa: o nosso maximo e' 0,379332 contra \
         0,379347 do oraculo (4e-5). O que difere e' o PADRAO -- uma folha que cresce \
         ENCURVA, e para que lado ela encurva e' decidido por assimetrias minusculas. \
         O irmao NEGATIVO bate (0,0197), o que estreita a pergunta: o defeito e' da \
         flambagem, nao do desvio de repouso",
    ),
    (
        "plano_filtro_gravidade_conjuntos_de_faces",
        "AUSENCIA: o app nao tem conjuntos de faces, e o traco liga tambem a gravidade da \
         CENA -- que, medida pelo oraculo, NAO passa pelo factor por vertice (o lado \
         excluido move-se na mesma)",
    ),
    (
        "plano_filtro_escala_gravidade_cena",
        "AUSENCIA: a escultura desta casa nao tem ajuste de gravidade de cena, entao nao ha' \
         de onde o valor viria. A espec tinha TRES afirmacoes erradas sobre ela (sinal, \
         espaco e a dependencia do arrasto), corrigidas na emenda de 07/09",
    ),
];

/// ⭐⭐ **O CENSO — as duas listas descrevem o corpus INTEIRO.**
///
/// ⚠️ **Ele é próprio do filtro**, e a razão é estrutural: o censo do arnês do
/// pincel varre a raiz das fixtures **sem recursão**, e um traço de filtro não é
/// corrível por aquele arnês. *Um corpus sem censo é um corpus onde um traço novo
/// entra e ninguém o corre.*
#[test]
fn o_censo_do_corpus_do_filtro() {
    let dir = fixture_dir().join("filtro");
    let mut no_disco: Vec<String> = std::fs::read_dir(&dir)
        .expect("o corpus do filtro existe")
        .filter_map(|e| {
            let n = e.ok()?.file_name().to_string_lossy().into_owned();
            n.strip_suffix(".deformado.txt.gz").map(str::to_string)
        })
        .collect();
    no_disco.sort();
    let mut inscritos: Vec<String> = CORPUS
        .iter()
        .map(|(n, _)| (*n).to_string())
        .chain(ABERTOS.iter().map(|(n, _)| (*n).to_string()))
        .collect();
    inscritos.sort();
    assert_eq!(
        no_disco, inscritos,
        "o corpus do filtro e as duas listas discordam -- um traco novo entrou e ninguem o corre, \
         ou uma lista nomeia um traco que ja' nao existe"
    );
    println!("corpus do filtro: {} tracos", no_disco.len());
}

/// ⭐⭐⭐ **A PARIDADE COM O ORÁCULO NÃO REGRIDE.**
///
/// ⚠️ **Ele corre o corpus INTEIRO antes de acusar**, e não pára no primeiro ✗ —
/// uma tabela cortada no primeiro vermelho esconde se o defeito é de um traço ou
/// da lei toda, que é a diferença que decide onde procurar.
#[test]
fn a_paridade_do_filtro_nao_regride() {
    let mut acusados = Vec::new();
    let mut pior = 0.0_f64;
    for (nome, barra) in CORPUS {
        let (e, escala) = erro(nome);
        println!("{nome:<42} erro {e:.6}  barra {barra:.6}  (escala do oraculo {escala:.6})");
        pior = pior.max(e);
        if e > *barra {
            acusados.push(format!("{nome}: {e:.6} > {barra:.6}"));
        }
    }
    println!("pior do corpus: {pior:.6}  ({} tracos)", CORPUS.len());
    assert!(
        acusados.is_empty(),
        "a paridade do filtro REGREDIU em {} traco(s):\n  {}",
        acusados.len(),
        acusados.join("\n  ")
    );
}

/// ⛔ **OS ABERTOS CONTINUAM ABERTOS** — o gate que impede a lista de mentir.
///
/// ⚠️ **Ele existe porque uma lista de dívida não encolhe sozinha:** se alguém
/// curar um destes e não o mover para o [`CORPUS`], a dívida fica escrita para
/// sempre sobre trabalho já pago. ⭐ Os dois que são AUSÊNCIA são saltados com o
/// motivo — eles não têm entrada de onde correr.
#[test]
fn os_abertos_do_filtro_ainda_estao_abertos() {
    for (nome, razao) in ABERTOS {
        if razao.starts_with("AUSENCIA") {
            println!("{nome:<42} AUSENCIA declarada -- nao ha' o que correr");
            continue;
        }
        let (e, _) = erro(nome);
        println!("{nome:<42} erro {e:.6} (aberto)");
        assert!(
            e > 0.05,
            "{nome} caiu para {e:.6} -- ele DEIXOU de estar aberto, e a lista tem de o dizer: \
             mova-o para o CORPUS com a barra medida"
        );
    }
}
