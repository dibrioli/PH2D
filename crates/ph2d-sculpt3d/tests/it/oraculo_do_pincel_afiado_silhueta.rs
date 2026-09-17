//! ⭐⭐⭐ **A BANCADA DA SILHUETA** — o report do dono de 2026-09-16 (*«se fizer o
//! traço do canto para o início da esfera, no canto fica meio pontilhado»*) e a
//! cura dele. Espec: `SPEC_pincel_afiado.md` §16.
//!
//! # O que esta bancada mede, e porque é uma RÉGUA NOVA
//!
//! A régua do §7 mede a secção **transversal** do vinco (profundidade, largura,
//! nitidez) e é **cega a este defeito**: um vinco contínuo e um pontilhado têm a
//! mesma secção no fundo de cada dab. A régua desta emenda mede **ao longo** do
//! traço — a ondulação da profundidade de estação para estação.
//!
//! # As três corridas, e a ordem delas é o argumento
//!
//! 1. **O CONTROLO da régua:** a saída publicada do ALVO, lida com a régua
//!    escrita aqui do zero, tem de reproduzir a tabela do §16.2. *Sem isto, tudo
//!    o resto mede a minha régua e não o produto.*
//! 2. **A fidelidade:** o nosso motor com o passo de ECRÃ (o que shipava) tem de
//!    pontilhar como o alvo — o defeito é herdado, não nosso.
//! 3. **A cura:** o mesmo gesto com o passo medido SOBRE A SUPERFÍCIE
//!    ([`ph2d_sculpt3d::CaminhoNoMundo`]) tem de deixar de pontilhar **sem**
//!    mudar o vinco no meio da peça.
//!
//! ⚠️ **As fixturas desta família publicam um SUBCONJUNTO** de uma malha gerada
//! por fórmula (a malha tem `185 977` vértices e publicar todas custaria
//! megabytes). O cabeçalho traz a fórmula inteira; esta bancada reconstrói a
//! malha dela e **confere** o subconjunto publicado contra a reconstrução antes
//! de medir o que quer que seja.

use super::oraculo_do_pincel_afiado_produto::LeiDoCursor;
use ph2d_mesh::{Face, Mesh, Ray};
use ph2d_sculpt3d::{Brush, CaminhoNoMundo, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// ⭐ **Quantas bandas de `u` são REGIME do traço** — as duas últimas são o
/// arranque dele (o vinco ainda a nascer), e afirmar sobre elas seria afirmar
/// sobre ONDE o traço começa, que é outra pergunta.
///
/// ⛔ Ele vive aqui, num sítio só, porque **dois** gates o usam: a cura do
/// pontilhado e a invariância à taxa de eventos. *Escrito duas vezes, o dia em
/// que a régua ganhasse uma banda deixaria um dos dois a afirmar outra coisa.*
const BANDAS_DO_REGIME: usize = 6;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// O olho: a vista é ortográfica de topo e o observador olha ao longo de `-z`.
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

fn pasta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/3D/cleanroom/fixtures/pincel_afiado/silhueta")
}

fn inflar(nome: &str) -> String {
    let p = pasta().join(format!("{nome}.txt.gz"));
    let raw = std::fs::read(&p).unwrap_or_else(|e| panic!("{nome}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{nome}: nao e' gzip"
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
        .unwrap_or_else(|e| panic!("{nome}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

/// Uma fixtura da silhueta: o cabeçalho e os blocos ESPARSOS, indexados contra a
/// malha inteira.
struct Fixtura {
    nome: String,
    cab: BTreeMap<String, String>,
    repouso: BTreeMap<usize, [f32; 3]>,
    saida: BTreeMap<usize, [f32; 3]>,
}

impl Fixtura {
    fn chave(&self, k: &str) -> &str {
        self.cab
            .get(k)
            .unwrap_or_else(|| panic!("{}: o cabecalho nao tem `{k}`", self.nome))
    }

    fn num(&self, k: &str) -> f32 {
        let v = self.chave(k);
        v.split_whitespace()
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(|| panic!("{}: `{k}` = `{v}` nao e' numero", self.nome))
    }

    fn inteiro(&self, k: &str) -> usize {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            self.num(k).round() as usize
        }
    }
}

fn ler(nome: &str) -> Fixtura {
    let texto = inflar(nome);
    let mut f = Fixtura {
        nome: nome.to_string(),
        cab: BTreeMap::new(),
        repouso: BTreeMap::new(),
        saida: BTreeMap::new(),
    };
    for l in texto.lines() {
        if let Some(resto) = l.strip_prefix("# ") {
            if let Some((k, v)) = resto.split_once(": ") {
                f.cab.insert(k.trim().to_string(), v.trim().to_string());
            }
            continue;
        }
        let c: Vec<&str> = l.split_whitespace().collect();
        if c.len() != 5 {
            continue;
        }
        let i: usize = c[1].parse().expect("indice");
        let p = [
            c[2].parse().expect("x"),
            c[3].parse().expect("y"),
            c[4].parse().expect("z"),
        ];
        match c[0] {
            "r" => {
                f.repouso.insert(i, p);
            }
            "s" => {
                f.saida.insert(i, p);
            }
            _ => {}
        }
    }
    assert_eq!(
        f.repouso.len(),
        f.inteiro("vertices_publicados"),
        "{nome}: o bloco `r` nao bate o cabecalho"
    );
    f
}

/// A malha da fórmula do cabeçalho — a cúpula (meia-cana) e a esfera.
struct Formula {
    nu: usize,
    nv: usize,
    raio: f32,
    umax_graus: f32,
}

/// **RECONSTRÓI a malha da fórmula publicada**, e **confere** o subconjunto.
///
/// ⚠️ **A conferência não é zelo:** se a fórmula que eu leio não for a que gerou
/// a fixtura, tudo o que esta bancada medir a seguir compara duas superfícies
/// diferentes e chama a isso um desvio do produto.
fn malha_da_formula(f: &Fixtura) -> (Mesh, Formula) {
    assert_eq!(
        f.chave("malha_gerada_por_formula")
            .split_whitespace()
            .next(),
        Some("sim"),
        "{}: esta fixtura nao declara a formula",
        f.nome
    );
    let tipo = f.chave("malha_tipo").to_string();
    let g = Formula {
        nu: f.inteiro("malha_celulas_u"),
        nv: f.inteiro("malha_celulas_v"),
        raio: f.num("malha_raio"),
        umax_graus: f.num("malha_meio_angulo_graus"),
    };
    let comprimento = f.num("malha_comprimento");
    let mut pos = Vec::with_capacity((g.nu + 1) * (g.nv + 1));
    for j in 0..=g.nv {
        for i in 0..=g.nu {
            #[allow(clippy::cast_precision_loss)]
            let u = (-g.umax_graus + 2.0 * g.umax_graus * i as f32 / g.nu as f32).to_radians();
            match tipo.as_str() {
                "cupula" => {
                    #[allow(clippy::cast_precision_loss)]
                    let y = -comprimento / 2.0 + comprimento * j as f32 / g.nv as f32;
                    pos.push([g.raio * u.sin(), y, g.raio * u.cos()]);
                }
                "esfera" => {
                    // A esfera é amostrada por dois ângulos: `u` ao longo do
                    // traço e `v` a volta dele.
                    #[allow(clippy::cast_precision_loss)]
                    let v =
                        (-g.umax_graus + 2.0 * g.umax_graus * j as f32 / g.nv as f32).to_radians();
                    pos.push([
                        g.raio * u.sin() * v.cos(),
                        g.raio * v.sin(),
                        g.raio * u.cos() * v.cos(),
                    ]);
                }
                outro => panic!("{}: malha `{outro}` sem tradução", f.nome),
            }
        }
    }
    // A conferência do subconjunto contra a fórmula.
    let mut pior = 0.0f32;
    for (&i, p) in &f.repouso {
        let q = pos[i];
        pior = pior.max((0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max));
    }
    assert!(
        pior <= 2e-6,
        "{}: a fórmula do cabeçalho não reproduz o repouso publicado ({pior:.3e})",
        f.nome
    );
    let l = u32::try_from(g.nu + 1).expect("nu");
    let mut faces = Vec::with_capacity(g.nu * g.nv);
    for j in 0..g.nv {
        for i in 0..g.nu {
            let a = u32::try_from(j * (g.nu + 1) + i).expect("indice");
            faces.push(Face::quad(a, a + 1, a + 1 + l, a + l));
        }
    }
    (Mesh::from_parts(pos, faces).expect("malha da formula"), g)
}

/// O pincel que o cabeçalho descreve — os valores de fábrica do afiado.
fn pincel(f: &Fixtura) -> Brush {
    assert_eq!(f.chave("pincel"), "DRAW_SHARP", "{}: outro pincel", f.nome);
    assert_eq!(f.chave("curva"), "POW4", "{}: outra curva", f.nome);
    assert_eq!(
        f.chave("direccao"),
        "SUBTRACT",
        "{}: outra direcção",
        f.nome
    );
    Brush {
        verb: Verb::DrawSharp,
        mode: ph2d_sculpt3d::RefMode::B,
        falloff: Falloff::Sharper,
        radius: f.num("raio_efectivo_objecto"),
        strength: f.num("forca"),
        hardness: f.num("dureza"),
        accumulate: false,
        normal_radius_frac: f.num("raio_da_normal"),
        traco_arrastado: true,
        // ⭐⭐⭐ **O ESPAÇAMENTO DO ALVO, do cabeçalho — NUNCA o nosso.** Desde
        // 16/09 o produto ship `2 %` por ordem do dono e o alvo tem `5 %`:
        // arrastar o traço DELE com o NOSSO espaçamento não mede paridade
        // nenhuma, mede dois pincéis diferentes. ⚠️ E a metade que engana é a
        // ATENUAÇÃO, que sai do mesmo número.
        espacamento_pct: Some(f.num("espacamento_pct_do_diametro")),
        ..Brush::default()
    }
}

fn na_superficie(m: &Mesh, x: f32, y: f32) -> Option<[f32; 3]> {
    m.raycast(&Ray::new([x, y, 10.0], OLHO)).map(|h| h.point)
}

/// **O ARRASTO do report**, pelas TRÊS leis do cursor.
///
/// ⭐ A lei é a MESMA que a bancada do produto mede ([`LeiDoCursor`]) — duas
/// bancadas com dois vocabulários para a mesma escolha divergiriam no dia em que
/// uma delas ganhasse um terceiro estado.
fn arrastar(f: &Fixtura, b: &Brush, lei: LeiDoCursor, px_por_evento: f32) -> Mesh {
    arrastar_por(f, b, lei, px_por_evento, Percurso::ParaDentro)
}

/// ⭐⭐⭐ **OS DOIS PERCURSOS, e o segundo é o que a fixtura do report não tinha.**
///
/// ⛔⛔ O corpus da silhueta arrasta **da borda para DENTRO**, e nesse percurso o
/// cursor sai do regime quase-tangente ao fim de meia dúzia de dabs — foi por
/// isso que a granularidade adaptativa dos candidatos ficou, durante uma wave
/// inteira, **sem uma medição que a defendesse** (a mutação que a apagava
/// sobrevivia a tudo).
///
/// ⛔⛔⛔ **E o tangencial foi CONSTRUÍDO, MEDIDO e NÃO ADOPTADO como gate:** ali
/// o traço mal carimba (profundidade `0,006` contra `0,205` do percurso do
/// report, `3 %` de um raio), porque junto à borda a pegada do pincel cai quase
/// toda **fora** da peça. *Uma régua sobre um traço que não esculpe mede ruído*,
/// e a barra teria de sair desse ruído. ⇒ o regime que a adaptação defende é
/// medido por UNIDADE, sobre a própria lei
/// (`a_granularidade_adaptativa_nao_perde_dabs_numa_superficie_que_dispara`), e
/// este percurso fica como instrumento.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Percurso {
    /// Da borda para o meio da peça — o percurso do report do dono.
    ParaDentro,
    /// Ao longo da borda, perpendicular ao anterior no plano do ecrã.
    AoLongoDaBorda,
}

fn arrastar_por(
    f: &Fixtura,
    b: &Brush,
    lei: LeiDoCursor,
    px_por_evento: f32,
    percurso: Percurso,
) -> Mesh {
    // ⛔⛔ **A bancada PERGUNTA AO VERBO, e a lei só escolhe entre o que ele
    // oferece.** Uma bancada que derivasse `no_mundo` só da `lei` afirmaria a
    // lei e não o PRODUTO: a mutação que tira o afiado de
    // [`Verb::mede_o_passo_no_mundo`] deixava este gate VERDE — medido.
    let no_mundo = b.verb.mede_o_passo_no_mundo() && lei != LeiDoCursor::Ecra;
    let leva = b.verb.o_dab_segue_o_barro() && lei == LeiDoCursor::PassoEBarro;
    let (mut m, _) = malha_da_formula(f);
    let congelada = m.clone();
    let ppu = f.num("vista_px_por_unidade");
    let pd: Vec<f32> = f
        .chave("pixel_do_pen_down_no_mundo")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("pixel do pen-down"))
        .collect();
    let um: Vec<f32> = f
        .chave("um_pixel_para_a_direita")
        .split_whitespace()
        .take(2)
        .map(|s| s.parse().expect("um pixel"))
        .collect();
    // O percurso: do pen-down até ao fim da linha do traço, que o cabeçalho dá
    // em mundo.
    let fim: Vec<f32> = f
        .chave("caminho_do_traco")
        .split(" a (")
        .nth(1)
        .and_then(|s| s.split(')').next())
        .expect("o fim do caminho")
        .split_whitespace()
        .map(|s| s.parse().expect("ponto do fim"))
        .collect();
    let total_px = ((fim[0] - pd[0]) / um[0]).abs();
    let sentido = if fim[0] >= pd[0] { 1.0 } else { -1.0 };
    let mut s = SculptStroke::default();
    s.begin(&m);
    // ⚠️ O tangencial é a perpendicular do outro NO PLANO DO ECRÃ, e a meia
    // altura: a peça acaba, e um traço que saia dela mede a ausência de malha.
    let (dx, dy, alcance) = match percurso {
        Percurso::ParaDentro => (um[0] * sentido, um[1] * sentido, total_px),
        Percurso::AoLongoDaBorda => (-um[1], um[0], total_px * 0.5),
    };
    let total_px = alcance;
    let mundo = |t: f32| (pd[0] + dx * t, pd[1] + dy * t);
    let carimbar = |m: &mut Mesh, s: &mut SculptStroke, t: f32| {
        let (wx, wy) = mundo(t);
        if let Some(c) = na_superficie(m, wx, wy) {
            s.dab(m, b, &Dab::at(c, b.radius, OLHO), Symmetry::default());
        }
    };
    carimbar(&mut m, &mut s, 0.0);
    let passo_de_ecra = ph2d_sculpt3d::passo_do_traco(b, b.radius * ppu);
    let passo_de_mundo =
        ph2d_sculpt3d::passo_no_mundo(b, b.radius).expect("o afiado mede no mundo");
    let mut caminho = CaminhoNoMundo::novo();
    let mut ancora = [0.0f32, 0.0];
    let mut t = 0.0f32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let eventos = (total_px / px_por_evento).ceil() as usize;
    for _ in 0..eventos {
        t = (t + px_por_evento).min(total_px);
        if no_mundo {
            // ⭐ **A lei NOVA, pela porta do MOTOR** — a mesma que o app corre
            // (`percorre_no_mundo`), com a granularidade adaptativa dos
            // candidatos. ⛔ Uma segunda cópia dela aqui faria esta bancada medir
            // outro programa.
            let mut alvo = CarimboDaBancada {
                malha: &mut m,
                traco: &mut s,
                congelada: &congelada,
                pincel: b,
                mundo: &mundo,
                anterior: ancora[0],
                segue_o_barro: leva,
            };
            caminho.percorre(ancora[0], t, passo_de_mundo, &mut alvo);
            ancora = [t, 0.0];
        } else {
            let Some(passos) = ph2d_sculpt3d::walk(ancora, [t, 0.0], passo_de_ecra) else {
                continue;
            };
            ancora = passos.anchor();
            for q in passos {
                carimbar(&mut m, &mut s, q[0]);
            }
        }
    }
    m
}

/// **A BANCADA a responder às duas perguntas da lei** — a gémea do
/// `CarimboDaCena` do app, e ela existe pela mesma razão: a lei vive no motor.
struct CarimboDaBancada<'a> {
    malha: &'a mut Mesh,
    traco: &'a mut SculptStroke,
    congelada: &'a Mesh,
    pincel: &'a Brush,
    mundo: &'a dyn Fn(f32) -> (f32, f32),
    anterior: f32,
    /// O centro do dab segue o BARRO (a lei do produto) em vez do raio.
    segue_o_barro: bool,
}

impl ph2d_sculpt3d::CarimboDoCaminho for CarimboDaBancada<'_> {
    fn congelado(&mut self, t: f32) -> Option<[f32; 3]> {
        let (x, y) = (self.mundo)(t);
        na_superficie(self.congelada, x, y)
    }

    fn carimba(&mut self, t: f32) -> bool {
        let (x, y) = (self.mundo)(t);
        self.anterior = t;
        // ⭐ **O centro do dab sai da MESMA porta do produto** — o acerto na
        // superfície congelada, levado pela deformação até onde o barro está
        // agora ([`ph2d_sculpt3d::levado_pela_deformacao`]). ⛔ Uma segunda
        // cópia da lei aqui faria esta bancada medir outro programa.
        let alvo = if self.segue_o_barro {
            self.congelada
                .raycast(&Ray::new([x, y, 10.0], OLHO))
                .and_then(|h| ph2d_sculpt3d::levado_pela_deformacao(self.congelada, self.malha, &h))
        } else {
            na_superficie(self.malha, x, y)
        };
        if let Some(c) = alvo {
            self.traco.dab(
                self.malha,
                self.pincel,
                &Dab::at(c, self.pincel.radius, OLHO),
                Symmetry::default(),
            );
        }
        true
    }
}

/// As bandas de `u` da tabela do §16.2, em graus.
const BANDAS: [(f32, f32); 8] = [
    (5.0, 15.0),
    (25.0, 35.0),
    (45.0, 55.0),
    (60.0, 68.0),
    (68.0, 74.0),
    (74.0, 79.0),
    (79.0, 83.0),
    (83.0, 88.0),
];

/// **A RÉGUA do §16.1** — `D/R` e a ondulação, por banda de `u`.
///
/// A estação é uma coluna da malha ao longo do traço, a coordenada é o **arco de
/// mundo** (⛔ nunca `x` de ecrã: junto à silhueta um píxel vale graus), e a
/// ondulação é `(max − min)/max` de `D` numa janela de **um período de dab
/// local** — o período sai da lei em vigor, não de um número escolhido.
///
/// ⛔ **`r` só é definida onde a janela INTEIRA cai dentro da faixa coberta**, e
/// com `≥ 5` amostras: sem esta cláusula a régua lê `1,000` nas PONTAS do traço,
/// onde o vinco acaba, e chamaria a isso defeito.
fn regua(
    f: &Fixtura,
    g: &Formula,
    repouso: &[[f32; 3]],
    pos: &dyn Fn(usize) -> [f32; 3],
    raio_do_pincel: f32,
) -> Vec<(f32, f32)> {
    let fila = g.nv / 2; // a fila do traço, `j = nv/2`
    let ppu = f.num("vista_px_por_unidade");
    let passo_base = raio_do_pincel * 2.0 * 0.05; // 5 % do diâmetro, em mundo
    let _ = ppu;
    // Estação a estação: o arco e a profundidade.
    let mut estacoes: Vec<(f32, f32, f32)> = Vec::with_capacity(g.nu + 1); // (u_graus, arco, D)
    for i in 0..=g.nu {
        let k = fila * (g.nu + 1) + i;
        let r = repouso[k];
        let p = pos(k);
        // A normal de repouso da cúpula/esfera é radial.
        let n = {
            let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            [r[0] / l, r[1] / l, r[2] / l]
        };
        let d = -(0..3).map(|c| (p[c] - r[c]) * n[c]).sum::<f32>();
        #[allow(clippy::cast_precision_loss)]
        let u = -g.umax_graus + 2.0 * g.umax_graus * i as f32 / g.nu as f32;
        estacoes.push((u, g.raio * u.to_radians(), d));
    }
    // A faixa coberta: onde alguma coisa se moveu.
    let tocadas: Vec<usize> = (0..estacoes.len())
        .filter(|&i| estacoes[i].2.abs() > 1e-6)
        .collect();
    let (ini, fim) = (
        estacoes[*tocadas.first().expect("o traço não tocou nada")].1,
        estacoes[*tocadas.last().expect("o traço não tocou nada")].1,
    );
    let mut saida = Vec::with_capacity(BANDAS.len());
    for (a, b) in BANDAS {
        let mut ds = Vec::new();
        let mut rs = Vec::new();
        for (idx, &(u, arco, d)) in estacoes.iter().enumerate() {
            let au = u.abs();
            if au < a || au > b {
                continue;
            }
            // O período local da lei em vigor: o passo de mundo dividido por
            // `cos u` — o encurtamento.
            let periodo = passo_base / au.to_radians().cos();
            let (lo, hi) = (arco - periodo / 2.0, arco + periodo / 2.0);
            if lo < ini.min(fim) || hi > ini.max(fim) {
                continue;
            }
            let janela: Vec<f32> = estacoes
                .iter()
                .filter(|(_, s, _)| *s >= lo && *s <= hi)
                .map(|(_, _, d)| *d)
                .collect();
            if janela.len() < 5 {
                continue;
            }
            let (mx, mn) = (
                janela.iter().copied().fold(f32::MIN, f32::max),
                janela.iter().copied().fold(f32::MAX, f32::min),
            );
            if mx <= 0.0 {
                continue;
            }
            ds.push(d);
            rs.push((mx - mn) / mx);
            let _ = idx;
        }
        let mediana = |mut v: Vec<f32>| -> f32 {
            if v.is_empty() {
                return f32::NAN;
            }
            v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
            v[v.len() / 2]
        };
        saida.push((mediana(ds) / raio_do_pincel, mediana(rs)));
    }
    saida
}

/// ⭐⭐⭐ **O CONTROLO DA RÉGUA — ela reproduz a tabela do §16.2 sobre a saída
/// PUBLICADA do alvo.**
///
/// ⛔ **Sem este gate os outros dois medem a minha régua**, não o produto: uma
/// régua escrita aqui que lesse outra coisa daria números plausíveis e erradosem
/// toda a bancada.
#[test]
fn a_regua_reproduz_a_tabela_do_alvo() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let (m, g) = malha_da_formula(&f);
    let repouso = m.positions().to_vec();
    let saida = |k: usize| *f.saida.get(&k).unwrap_or(&repouso[k]);
    let lida = regua(&f, &g, &repouso, &saida, f.num("raio_efectivo_objecto"));
    // A tabela do §16.2 (a meia-cana), banda a banda.
    const ALVO: [(f32, f32); 8] = [
        (0.2019, 0.002),
        (0.1860, 0.006),
        (0.1497, 0.023),
        (0.1092, 0.085),
        (0.0831, 0.182),
        (0.0608, 0.314),
        (0.0346, 0.570),
        (0.0389, 0.977),
    ];
    for (k, ((d, r), (ad, ar))) in lida.iter().zip(ALVO).enumerate() {
        println!(
            "banda {k}: D/R {d:.4} (espec {ad:.4}) · ondulação {r:.3} (espec {ar:.3})",
            k = k
        );
        assert!(
            (d - ad).abs() / ad <= 0.10,
            "banda {k}: D/R {d:.4} contra os {ad:.4} da espec"
        );
        assert!(
            (r - ar).abs() <= 0.06,
            "banda {k}: ondulação {r:.3} contra os {ar:.3} da espec"
        );
    }
}

/// ⭐⭐ **A FIDELIDADE: com o passo de ECRÃ o nosso motor pontilha como o alvo.**
///
/// É o G-16 da espec, e ele é o que diz ao dono que o defeito é **herdado**.
#[test]
fn com_o_passo_de_ecra_pontilhamos_como_o_alvo() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let b = pincel(&f);
    let (base, g) = malha_da_formula(&f);
    let repouso = base.positions().to_vec();
    let nossa = arrastar(&f, &b, LeiDoCursor::Ecra, 1.0);
    let alvo = |k: usize| *f.saida.get(&k).unwrap_or(&repouso[k]);
    let nossos = |k: usize| nossa.positions()[k];
    let la = regua(&f, &g, &repouso, &alvo, b.radius);
    let ln = regua(&f, &g, &repouso, &nossos, b.radius);
    for (k, ((nd, nr), (ad, ar))) in ln.iter().zip(&la).enumerate() {
        println!("banda {k}: D/R {nd:.4} contra {ad:.4} · ondulação {nr:.3} contra {ar:.3}");
        assert!(
            (nd - ad).abs() / ad <= 0.12,
            "banda {k}: a nossa profundidade {nd:.4} contra a dele {ad:.4}"
        );
        assert!(
            (nr - ar).abs() <= 0.10,
            "banda {k}: a nossa ondulação {nr:.3} contra a dele {ar:.3}"
        );
    }
}

/// ⭐⭐⭐ **A CURA — o vinco deixa de pontilhar E deixa de ser raso, sem mexer no
/// meio da peça.**
///
/// As bandas medidas são as **`0` a `5`** (`u` de `5°` a `79°`), que é onde o
/// traço corre em regime. ⚠️ **As duas últimas são o ARRANQUE do traço e não o
/// defeito:** o pen-down desta fixtura está a `84°`, logo metade das estações de
/// `83–88°` fica *depois* do primeiro dab e o que a régua lê ali é o vinco a
/// acabar — a recusa medida §16.1.3 já o diz (*«a régua lê `1,000` nas pontas do
/// traço»*). Os números delas ficam IMPRESSOS, nunca afirmados.
#[test]
fn a_lei_do_passo_no_mundo_cura_o_pontilhado() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let b = pincel(&f);
    let (base, g) = malha_da_formula(&f);
    let repouso = base.positions().to_vec();
    let ecra = arrastar(&f, &b, LeiDoCursor::Ecra, 1.0);
    let mundo = arrastar(&f, &b, LeiDoCursor::PassoEBarro, 1.0);
    let le = regua(&f, &g, &repouso, &|k| ecra.positions()[k], b.radius);
    let lm = regua(&f, &g, &repouso, &|k| mundo.positions()[k], b.radius);
    for (k, ((ed, er), (md, mr))) in le.iter().zip(&lm).enumerate() {
        println!("banda {k}: ecrã D/R {ed:.4} r {er:.3} → barro D/R {md:.4} r {mr:.3}");
    }
    // ⭐ **A ONDULAÇÃO, nas seis bandas de regime.** A barra sai do vale medido:
    // a lei de ecrã lê até `0,314` e a nova até `0,064`.
    let pior_nova = lm[..BANDAS_DO_REGIME]
        .iter()
        .map(|(_, r)| *r)
        .fold(0.0f32, f32::max);
    let pior_velha = le[..BANDAS_DO_REGIME]
        .iter()
        .map(|(_, r)| *r)
        .fold(0.0f32, f32::max);
    assert!(
        pior_nova <= 0.10,
        "a ondulação em regime ainda é {pior_nova:.3} (a lei de ecrã lê {pior_velha:.3})"
    );
    assert!(
        pior_velha >= 0.30,
        "o CONTROLO deixou de conter o defeito: a lei de ecrã lê {pior_velha:.3}"
    );
    // ⭐⭐ **A PROFUNDIDADE deixa de cair com a inclinação** — que é a segunda
    // metade do que o dono viu: com os dabs separados, cada ponto recebia UM dab
    // em vez da sobreposição de vinte.
    let razao_nova = lm[5].0 / lm[0].0;
    let razao_velha = le[5].0 / le[0].0;
    println!("profundidade a 74–79° / a 5–15°: {razao_nova:.2} (era {razao_velha:.2})");
    assert!(
        razao_nova >= 0.90,
        "junto à silhueta o vinco é {razao_nova:.2}× o do meio"
    );
    assert!(
        razao_velha <= 0.50,
        "o CONTROLO deixou de conter o defeito: a lei de ecrã dá {razao_velha:.2}"
    );
    // ⭐ **E o MEIO da peça fica intocado** — uma cura que endireitasse a borda
    // mexendo no meio trocaria um defeito por outro, e o meio é onde o artista
    // trabalha.
    assert!(
        (lm[0].0 - le[0].0).abs() / le[0].0 <= 0.02,
        "a cura mexeu no meio da peça: {:.4} contra {:.4}",
        lm[0].0,
        le[0].0
    );
}

/// ⭐⭐⭐ **G-19 — o traço é facto do CAMINHO**, e a catraca que proíbe importar a
/// lei do alvo.
///
/// O mesmo caminho entregue a `1`, `4` e `16` píxeis por evento tem de dar o
/// mesmo vinco. ⛔ É isto que a lei do alvo perde (`−30 %` a `−51 %` medidos na
/// espec §16.9), porque ela mede contra uma superfície que o próprio traço move.
///
/// ⛔⛔ **A régua é uma REFERÊNCIA FINA (`¼` de píxel por evento), e NÃO uma taxa
/// contra a outra** — e a diferença foi medida, não escolhida. Comparar `1` px
/// com `4` px é um ESPELHO: a mutação que apaga a granularidade adaptativa dos
/// candidatos faz as duas taxas amostrarem grosso **da mesma maneira**, logo elas
/// concordam e o gate fica VERDE sobre a lei apagada. Contra uma referência mais
/// fina do que qualquer candidato que a lei possa escolher, essa mutação SANGRA.
/// *Um oráculo que partilha a lei do que julga é um espelho.*
#[test]
fn o_traco_e_facto_do_caminho_e_nao_da_taxa_de_eventos() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let b = pincel(&f);
    let (base, g) = malha_da_formula(&f);
    let repouso = base.positions().to_vec();
    // A referência: eventos de ¼ de píxel, mais finos que o [`PASSO_INICIAL`] de
    // qualquer candidato ⇒ ela não pode herdar o grão que julga.
    for percurso in [Percurso::ParaDentro] {
        let fina = arrastar_por(&f, &b, LeiDoCursor::PassoEBarro, 0.25, percurso);
        let lf = regua(&f, &g, &repouso, &|k| fina.positions()[k], b.radius);
        for px in [1.0f32, 4.0, 16.0] {
            let m = arrastar_por(&f, &b, LeiDoCursor::PassoEBarro, px, percurso);
            let l = regua(&f, &g, &repouso, &|k| m.positions()[k], b.radius);
            let mut comparadas = 0usize;
            for (k, ((d, _), (df, _))) in l.iter().zip(&lf).enumerate() {
                // ⚠️ **Um traço TANGENCIAL vive numa banda de `u` só** — as outras
                // não têm estação nenhuma e a régua devolve `NaN` ali. *Comparar um
                // `NaN` é afirmar sobre o que não foi medido*, e o piso abaixo é o
                // que impede este gate de ficar verde por não ter medido nada.
                if !d.is_finite() || !df.is_finite() || *df <= 0.0 {
                    continue;
                }
                let desvio = (d - df).abs() / df;
                println!(
                    "{percurso:?} banda {k} a {px} px: {d:.4} contra {df:.4} ({:.1} %)",
                    desvio * 100.0
                );
                // ⚠️ **As duas últimas bandas são o ARRANQUE do traço, não o regime**
                // — ali o vinco ainda está a nascer, e uma amostragem mais fina põe
                // o primeiro dab num sítio um cabelo diferente (medido: `24 %` na
                // banda 6, contra `≤ 0,9 %` nas seis do regime). Afirmar sobre elas
                // seria afirmar sobre onde o traço COMEÇA, que é outra pergunta.
                if k >= BANDAS_DO_REGIME {
                    continue;
                }
                comparadas += 1;
                assert!(
                    desvio <= 0.05,
                    "{percurso:?} banda {k}: {px} px por evento muda a profundidade {:.1} % contra \
                 a referência fina",
                    desvio * 100.0
                );
            }
            assert!(
                comparadas >= 1,
                "{percurso:?} a {px} px: NENHUMA banda foi comparada — este gate não afirma nada"
            );
        }
    }
}

/// ⛔⛔⛔ **A RECUSA MEDIDA do percurso tangencial** — o instrumento que a
/// produziu, guardado para quem voltar a esta pergunta.
///
/// O gesto que corre **ao longo** da borda é o único que fica em regime
/// quase-tangente do princípio ao fim, e por isso parecia a fixtura óbvia para
/// defender a granularidade adaptativa dos candidatos.
///
/// ⛔⛔ **Medido, o que não serve é a RÉGUA, e não o gesto.** A `regua` do §16.1
/// ordena as estações pela coordenada de arco do percurso **para dentro**; num
/// traço tangencial essas estações atravessam o vinco em vez de o seguirem, e o
/// que elas leem já não é ondulação: é o PERFIL do pincel. A tabela di-lo
/// sozinha — as bandas `0`–`4` ficam sem estação nenhuma (`NaN`), e nas três que
/// sobram a profundidade sobe `0,006 → 0,058 → 0,100` (que é a queda do pincel
/// a ser percorrida de lado) com a ondulação **saturada em `0,95`–`0,99` mesmo
/// com a cura ligada**. *Uma régua cujo valor «são» é feito do artefacto dela
/// própria não tem onde pôr uma barra* — a mesma forma que a `tip_deviation` da
/// retopologia já tinha pago.
///
/// ⚠️ **A primeira redacção desta recusa dizia «o pincel mal esculpe», e o
/// próprio instrumento a desmentiu** (`0,100` contra `0,205`, metade e não
/// `3 %`). *Escrever a razão antes de a imprimir é como uma recusa medida
/// envelhece no dia em que nasce.*
///
/// ⇒ o mecanismo é defendido por UNIDADE, sobre a própria lei
/// (`a_granularidade_adaptativa_nao_perde_dabs_numa_superficie_que_dispara`), e
/// este percurso fica como instrumento.
#[test]
#[ignore = "instrumento: imprime a recusa medida do percurso tangencial"]
fn diag_o_percurso_ao_longo_da_borda_mal_esculpe() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let b = pincel(&f);
    let (base, g) = malha_da_formula(&f);
    let repouso = base.positions().to_vec();
    for percurso in [Percurso::ParaDentro, Percurso::AoLongoDaBorda] {
        let m = arrastar_por(&f, &b, LeiDoCursor::PassoEBarro, 1.0, percurso);
        let l = regua(&f, &g, &repouso, &|k| m.positions()[k], b.radius);
        for (k, (d, r)) in l.iter().enumerate() {
            println!("{percurso:?} banda {k}: D/R {d:.4} · ondulação {r:.4}");
        }
    }
}

/// ⭐⭐⭐ **G-21 — o NOSSO espaçamento de fábrica deixa o sulco mais contínuo que
/// o do alvo, e quase não mexe na profundidade.**
///
/// ⛔⛔ **Este gate existe porque nenhum outro mede o que o produto ship.** Todo
/// o corpus arrasta com o espaçamento do **alvo** (é o que o cabeçalho fixa, e é
/// o que torna a paridade uma medição em vez de uma comparação de dois pincéis
/// diferentes) ⇒ *sem esta metade, a ordem do dono de 16/09 — «o spacing está
/// alto e fica meio pontilhada. Reduza o spacing» — não teria régua nenhuma, e
/// reverter o número passaria calado.*
///
/// A régua é a do §16.1, com a cura ligada dos dois lados: o que muda entre as
/// colunas é **só** o espaçamento.
///
/// ⚠️⚠️ **RESSALVA da régua, escrita porque ela existe:** a ondulação do §16.1 é
/// normalizada por **UM período de dab**, e o período segue o espaçamento ⇒ parte
/// da melhoria daquela coluna é *definicional* e não do produto. É por isso que
/// a afirmação que este gate faz sobre a continuidade **não é a ondulação**: é o
/// **passo**, que é aritmética e não tem janela nenhuma — dois dabs consecutivos
/// passam a ficar `2,5×` mais perto, logo sobrepõem-se `2,5×` mais. A ondulação
/// entra como a coluna que **corrobora** e que reprovaria se a troca piorasse o
/// sulco. *Uma régua cuja janela segue o número que se está a variar não pode ser
/// a única testemunha da variação.*
#[test]
fn o_nosso_espacamento_de_fabrica_deixa_o_sulco_mais_continuo() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let como_o_alvo = pincel(&f);
    assert!(
        como_o_alvo.espacamento_pct == Some(ph2d_sculpt3d::ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT),
        "a fixtura tem de trazer o espaçamento do alvo"
    );
    // `None` = o que o verbo declara, que é o que o pincel VESTE ao nascer.
    let nosso = Brush {
        espacamento_pct: None,
        ..como_o_alvo.clone()
    };
    let (base, g) = malha_da_formula(&f);
    let repouso = base.positions().to_vec();
    let medir = |b: &Brush| {
        let m = arrastar(&f, b, LeiDoCursor::PassoEBarro, 1.0);
        regua(&f, &g, &repouso, &|k| m.positions()[k], b.radius)
    };
    let (la, ln) = (medir(&como_o_alvo), medir(&nosso));
    let mut pior_alvo = 0.0f32;
    let mut pior_nosso = 0.0f32;
    for (k, ((da, ra), (dn, rn))) in la.iter().zip(&ln).take(BANDAS_DO_REGIME).enumerate() {
        println!("banda {k}: alvo D/R {da:.4} r {ra:.4} → nosso D/R {dn:.4} r {rn:.4}");
        pior_alvo = pior_alvo.max(*ra);
        pior_nosso = pior_nosso.max(*rn);
        // ⚠️ **A profundidade quase não se mexe, e é isso que torna a troca
        // barata:** quem limita este vinco é a auto-limitação (a queda mede-se
        // das posições do pen-down), não o número de dabs.
        let delta = (dn - da).abs() / da;
        assert!(
            delta <= 0.05,
            "banda {k}: o nosso espaçamento mudou a profundidade {:.1} %",
            delta * 100.0
        );
    }
    // ⭐ **A metade que NÃO depende de régua nenhuma**: o passo do produto é
    // estritamente mais fino que o do alvo, na mesma unidade e no mesmo raio.
    let passo_alvo = ph2d_sculpt3d::passo_no_mundo(&como_o_alvo, como_o_alvo.radius)
        .expect("o afiado mede no mundo");
    let passo_nosso =
        ph2d_sculpt3d::passo_no_mundo(&nosso, nosso.radius).expect("o afiado mede no mundo");
    let razao = passo_alvo / passo_nosso;
    println!("passo: alvo {passo_alvo:.5} → nosso {passo_nosso:.5} ({razao:.2}× mais fino)");
    assert!(
        razao >= 2.0,
        "o espaçamento de fábrica tem de ser pelo menos o DOBRO do fino: {razao:.2}×"
    );
    println!("ondulação pior: alvo {pior_alvo:.4} → nosso {pior_nosso:.4}");
    assert!(
        pior_nosso < pior_alvo,
        "o nosso espaçamento tem de deixar o sulco MAIS contínuo: {pior_nosso:.4} contra \
         {pior_alvo:.4}"
    );
}

/// SONDA: o relógio de um traço aos dois espaçamentos.
#[test]
#[ignore = "sonda: imprime o custo do traço"]
fn diag_o_custo_do_traco_aos_dois_espacamentos() {
    let f = ler("produto_cupula_fabrica_da_silhueta");
    let como_o_alvo = pincel(&f);
    let nosso = Brush {
        espacamento_pct: None,
        ..como_o_alvo.clone()
    };
    let (base, _) = malha_da_formula(&f);
    println!("malha: {} vertices", base.positions().len());
    for (nome, b) in [("alvo 5 %", &como_o_alvo), ("nosso 2 %", &nosso)] {
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let _ = arrastar(&f, b, LeiDoCursor::PassoEBarro, 1.0);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
        }
        // O traço é entregue a 1 px por evento ⇒ o custo por EVENTO é o que o
        // quadro paga, e é ele que se compara com o orçamento de 8 ms.
        let px = f.num("vista_px_por_unidade");
        let pd: Vec<f32> = f
            .chave("pixel_do_pen_down_no_mundo")
            .split_whitespace()
            .take(2)
            .map(|s| s.parse().expect("pd"))
            .collect();
        let um: Vec<f32> = f
            .chave("um_pixel_para_a_direita")
            .split_whitespace()
            .take(2)
            .map(|s| s.parse().expect("um"))
            .collect();
        let fim: Vec<f32> = f
            .chave("caminho_do_traco")
            .split(" a (")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .expect("fim")
            .split_whitespace()
            .map(|s| s.parse().expect("p"))
            .collect();
        let eventos = ((fim[0] - pd[0]) / um[0]).abs();
        let _ = px;
        println!(
            "{nome}: traço inteiro {melhor:.1} ms sobre {eventos:.0} eventos ⇒ {:.3} ms/evento",
            melhor / f64::from(eventos)
        );
    }
}
