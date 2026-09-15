//! **O ARNÊS DO CENSO DOS KNOBS** — a peça, o gesto, o traço e a régua.
//!
//! ⚠️ **Filho (`#[path]`) do [`super`], e o corte é de RESPONSABILIDADE:** lá
//! moram as PERGUNTAS (as catracas dos mortos e dos adormecidos, os censos de
//! população, as sondas) e aqui mora *como se produz uma resposta* — que peça
//! cada verbo precisa, que gesto cada grip sabe receber, o que o pen-down
//! fotografaria, e o que «duas saídas diferentes» quer dizer.
//!
//! ⛔ **O corte foi forçado pelo tecto de LOC** (`826` contra `700`) quando o
//! [`Verb::Density`] acordou, e é melhor por isso: a régua e o que ela mede
//! passaram a ter endereços diferentes. *A cura de um tecto é o corte — nunca
//! uma entrada nova no `FILE_OVERAGE_OK`* (`CLAUDE.md` §5.0).

use ph2d_mesh::Mesh;
use ph2d_panel_sculpt3d::rows::{Place, SECTIONS, rows};
use ph2d_panel_sculpt3d::slots::VerbSlot;
use ph2d_panel_sculpt3d::state::Sculpt3dUi;
use ph2d_panel_sculpt3d::state_modes::UiLevel;
use ph2d_sculpt3d::{Amount, Brush, Dab, Falloff, Grip, SculptStroke, Symmetry, Verb};

/// A peça do censo — fina o bastante para um dab tocar centenas de vértices.
///
/// ⚠️ **Nem todo verbo pode ser medido nesta**, e a pergunta é feita à PORTA do
/// motor: um verbo que [`Verb::precisa_de_bordo_aberto`] não move um único
/// vértice numa peça fechada, porque a região dele **começa na borda**.
pub(super) fn peca_de(verb: Verb) -> Mesh {
    if verb.precisa_de_bordo_aberto() {
        // ⭐⭐ **A TIGELA vem pela porta do PRODUTO** (a mesma da cena `=42`), e
        // não construída aqui — ⛔ a 1.ª tentativa montou-a com `from_parts`
        // sobre as posições originais e ficou com **721 vértices ÓRFÃOS**: a
        // busca da âncora aterrava num vértice sem arestas e a lei recusava o
        // traço inteiro (`2,98e-8`, ruído de `f32`). *É o mesmo defeito de
        // fixtura que aquela cena já pagou, reproduzido por eu ter reconstruído
        // o que já tinha porta.*
        return crate::scenes::boundary::tigela();
    }
    ph2d_mesh::shapes::uv_sphere(24, 32, 1.0)
}

pub(super) const PONTA: [f32; 3] = [0.0, 0.0, 1.0];
pub(super) const OLHO: [f32; 3] = [0.0, 0.0, -1.0];
pub(super) const RAIO: f32 = 0.5;

/// **ONDE O TRAÇO COMEÇA** — o ápice da peça deste verbo.
///
/// ⚠️ Para quem precisa de bordo é a **BEIRA** (o vértice mais alto da tigela é
/// da boca dela), e para os outros é o pólo. *Um cursor longe do sujeito do
/// verbo mede um gesto que não acontece.*
pub(super) fn onde(verb: Verb, mesh: &Mesh) -> [f32; 3] {
    if verb.precisa_de_bordo_aberto() {
        return mesh
            .positions()
            .iter()
            .copied()
            .max_by(|a, b| a[1].total_cmp(&b[1]))
            .unwrap_or(PONTA);
    }
    PONTA
}

/// ⭐⭐ **O GESTO QUE ESTE GRIP SABE RECEBER**, no `k`-ésimo passo do traço.
///
/// ⛔ Sem isto metade da tabela leria `0,000` em toda a linha — não porque os
/// knobs estejam mortos, mas porque **o verbo não recebeu gesto nenhum**.
pub(super) fn gesto(verb: Verb, raio: f32, centro: [f32; 3], k: usize) -> Dab {
    let t = k as f32 + 1.0;
    match verb.grip() {
        Grip::Hold => Dab::pulling(centro, raio, OLHO, [0.12 * t, 0.0, 0.0]),
        Grip::Hook => Dab::hooking(centro, raio, OLHO, [0.12 * t, 0.0, 0.0]),
        Grip::Turn(Amount::Angle) => Dab::turning(centro, raio, OLHO, 0.3 * t),
        Grip::Turn(Amount::Fraction) => Dab::scaling(centro, raio, OLHO, 0.15 * t),
        _ => Dab::at(centro, raio, OLHO),
    }
}

/// A superfície de referência da multiresolução, sintética — ver o gémeo em
/// `verb_tests.rs`: **sem ela os dois verbos de deslocamento são inertes por
/// lei**, e o censo leria a ausência de entrada como um knob morto.
///
/// ⚠️⚠️ **Ela tem RELEVO, e a 1.ª redacção não tinha.** Uma referência
/// uniformemente encolhida é lisa, e o [`Verb::SmearMultires`] **transporta
/// relevo** — sobre uma superfície sem nenhum ele mede o que sobra do
/// arredondamento. Medido: `4,53e-3` liso contra `7,53e-3` com relevo. *Uma
/// fixtura que não contém o fenómeno não afirma nada sobre ele.*
pub(super) fn referencia(mesh: &Mesh) -> Vec<[f32; 3]> {
    mesh.positions()
        .iter()
        .map(|p| {
            let k = 0.8 * (1.0 + 0.08 * (p[0] * 9.0).sin() * (p[1] * 9.0).cos());
            [p[0] * k, p[1] * k, p[2] * k]
        })
        .collect()
}

/// A outra peça da cena, sintética — **uma esfera que ENVOLVE a peça**, para
/// que qualquer raio acerte. Mesma razão da de cima.
pub(super) fn alvo() -> Vec<(Mesh, ph2d_mesh::Pose)> {
    vec![(
        ph2d_mesh::shapes::uv_sphere(12, 16, 3.0),
        ph2d_mesh::Pose::IDENTITY,
    )]
}

/// **QUANTOS DABS O TRAÇO TEM.**
///
/// ⭐⭐⭐ **DOIS, e a diferença entre um e dois é QUATRO verbos.** Um carimbo
/// isolado não tem **caminho**: o [`Dab::path`] sai da diferença entre centros
/// de dabs consecutivos, e quatro leis lêem-no — o polegar e o raspador de
/// planos derivam dele o eixo de inclinação, o esfregão a direcção de
/// transporte, e o pano o passo do solver. Com **um** dab os quatro liam
/// `0,000e0` em toda a linha e o censo declarava-os *«não medidos»*.
///
/// Medido ao acordá-los: `Clay Thumb` `4,12e-3` · `Multiplane Scrape`
/// `7,92e-3` · `Smear Displacement` `7,53e-3` · `Cloth` `2,34e-2`.
///
/// ⚠️ **Não são três nem dez:** dois é o mínimo que produz um caminho, e cada
/// dab a mais é ruído de composição a entrar numa régua que compara **duas
/// posições de um knob** — *o censo mede o knob, não o traço*.
pub(super) const DABS_DO_TRACO: usize = 2;

/// **UM TRAÇO PELO CAMINHO DO MOTOR**, com tudo o que o pen-down fotografaria.
///
/// ⚠️⚠️ **O raio do dab É o `Brush::radius`, e isso é a LEI do produto, não uma
/// conveniência do arnês** — [`o_raio_do_dab_sai_do_pincel`] prende-a. A 1.ª
/// redacção deste ficheiro carregava um raio **próprio** ao lado do pincel, e o
/// que ela mediu foi outro programa: o `Verb::Pose` lê `brush.radius` (a lei
/// dele não tem dab por-vértice nenhum), logo mexer só no raio do dab deixava-o
/// a `0,000e0` e o censo acusava-o de morto. *Dois números onde o produto tem
/// um é a forma mais barata de uma régua mentir.*
pub(super) fn corre(b: &Brush) -> Mesh {
    let mut mesh = peca_de(b.verb);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    if b.verb.precisa_de_referencia() {
        s.reference = referencia(&mesh);
    }
    if b.precisa_das_pecas_da_cena() {
        s.pecas_da_cena = alvo();
        s.pose_activa = ph2d_mesh::Pose::IDENTITY;
    }
    let centro = onde(b.verb, &mesh);
    // ⭐⭐⭐ **A LEI DE QUEM NÃO TEM LEI POR-VÉRTICE corre AQUI, e ela é a mesma
    // porta do produto** ([`crate::dyntopo::passe_nos_motores`]).
    //
    // ⛔⛔ O [`Verb::Density`] esteve na catraca dos ADORMECIDOS *por LEI*: o
    // efeito dele é sobre a TOPOLOGIA, e o `SculptStroke::dab` desvia antes de a
    // cadeia de peso existir — *não é um verbo inerte, é um verbo cuja lei não
    // vive ali*. Sem esta metade o censo media **zero** das células dele, que é
    // onde o próximo *«não vejo efeito»* nasceria.
    //
    // ⚠️ **A TRIANGULAÇÃO é do PRODUTO e não do arnês:** os dois motores
    // recusam quads por geometria, e quem os tritura é o pen-down
    // (`open_dyntopo_stroke`). A peça deste censo é uma esfera UV, que é toda
    // quads — sem isto o passe seria um **no-op silencioso** e o verbo continuava
    // adormecido com o censo a dizer que acordou.
    let topologia = b.verb.sem_lei_por_vertice();
    if topologia {
        mesh.triangulate();
    }
    let (mut remap, mut births, mut region) = (
        ph2d_mesh::Remap::default(),
        Vec::new(),
        ph2d_mesh::RegionScratch::default(),
    );
    for k in 0..DABS_DO_TRACO {
        // ⚠️ Os centros ANDAM — é a diferença entre eles que vira o
        // [`Dab::path`], e é o `path` que acorda os quatro verbos de caminho.
        let passo = 0.12 * k as f32;
        let mut d = gesto(b.verb, b.radius, centro, k);
        d.center = [centro[0] + passo, centro[1], centro[2]];
        // ⚠️ **ANTES do dab, que é a ordem do produto** (o braço do carimbo
        // chama o `refine_for_dab` e só depois carimba). Para todo verbo com lei
        // por-vértice isto nem corre.
        if topologia {
            let alvo = ph2d_mesh::edge_target_for_mesh(&mesh, b.density_detail);
            crate::dyntopo::passe_nos_motores(
                &mut mesh,
                b.verb,
                alvo,
                d.center,
                b.radius,
                &mut remap,
                &mut births,
                &mut region,
            );
            s.begin(&mesh);
        }
        s.dab(&mut mesh, b, &d, Symmetry::default());
    }
    mesh
}

/// O maior desvio entre duas saídas — **posição E máscara**, porque há verbos
/// que só escrevem o canal (`Verb::Mask`) e um censo que olhasse só posições
/// acusaria todos os knobs dele.
pub(super) fn desvio(a: &Mesh, b: &Mesh) -> f32 {
    // ⭐⭐⭐ **A CONTAGEM PRIMEIRO — dois barros de tamanhos diferentes são
    // diferentes**, e sem esta metade o `zip` abaixo compararia o **PREFIXO** e
    // leria `0,0` sobre uma malha que ganhou dez mil vértices.
    //
    // ⚠️ **Duas GRANDEZAS numa porta, e é de propósito:** o censo só pergunta
    // *«é zero?»*, e a resposta honesta para um verbo de topologia é *quantos
    // vértices e faces a mais/menos*. Escrever uma segunda régua ao lado faria o
    // censo ter de escolher qual chamar por verbo — que é a lista paralela que
    // este ficheiro existe para não ter.
    let dv = a.positions().len().abs_diff(b.positions().len());
    let df = a.faces().len().abs_diff(b.faces().len());
    if dv + df > 0 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a resposta é «é zero?», e a magnitude só serve à sonda"
        )]
        return (dv + df) as f32;
    }
    let pos = a
        .positions()
        .iter()
        .zip(b.positions())
        .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0f32, f32::max))
        .fold(0.0f32, f32::max);
    // ⚠️⚠️ **Um lado SEM canal é um canal de ZEROS, nunca «não comparável»** —
    // e isto foi apanhado pelo controlo positivo: a peça por tocar não tem
    // máscara, logo o `Verb::Mask` lia-se **INERTE** contra ela e os cinco
    // knobs dele entravam como não-medidos. *Uma ausência e um zero são a mesma
    // coisa para este canal, e tratá-los como coisas diferentes apaga o único
    // verbo que só escreve nele.*
    let em = |m: Option<&[f32]>, i: usize| m.map_or(0.0, |v| v[i]);
    let n = a.positions().len().min(b.positions().len());
    let canal = (0..n)
        .map(|i| (em(a.masks(), i) - em(b.masks(), i)).abs())
        .fold(0.0f32, f32::max);
    pos.max(canal)
}

/// O pincel deste verbo como o painel o entrega, com o raio na régua do censo.
///
/// ⚠️ **A fileira do raio não escreve no `Brush` — ela escreve no `radius_px`
/// da vista**, e quem os une é o [`crate::Sculpt3dScene::armed_brush`], que
/// converte pixels em mundo e **põe o resultado no `Brush::radius`**. O censo
/// entra na cadeia depois dessa conversão, que é verbo-cega por construção.
pub(super) fn pincel(verb: Verb) -> Brush {
    Brush {
        radius: RAIO,
        ..VerbSlot::for_verb(verb).brush
    }
}

/// Um KNOB do painel: o rótulo que ele mostra, as duas posições que o censo
/// varre, e a pergunta *«o painel pinta-o com este verbo?»*.
pub(super) struct Knob {
    /// A chave de i18n, que é como a fileira se chama no painel.
    pub(super) rotulo: &'static str,
    /// As duas posições. ⚠️ Elas são **extremos da faixa do próprio painel**,
    /// nunca números escolhidos: um par apertado lê `0,000` sobre um knob vivo.
    pub(super) a: fn(&mut Brush),
    pub(super) b: fn(&mut Brush),
}

/// **QUANTO BARRO ESTE KNOB MOVE COM ESTE VERBO** — as duas posições, o mesmo
/// gesto, e o maior desvio entre as duas saídas.
///
/// ⚠️ Vive numa porta só porque a sonda e o portão fazem a **mesma** pergunta:
/// duas cópias divergiriam na primeira wave que mexesse numa delas, e a que o
/// portão usa é a que decide.
pub(super) fn quanto_move(verb: Verb, k: &Knob) -> f32 {
    let (mut x, mut y) = (pincel(verb), pincel(verb));
    (k.a)(&mut x);
    (k.b)(&mut y);
    desvio(&corre(&x), &corre(&y))
}

/// ⚠️ **São os knobs que o painel pinta SEM perguntar pelo verbo** — os outros
/// já têm `show` por verbo e portanto não podem estar mortos por construção.
/// A lista sai da tabela `BRUSH` de `rows.rs`, e o gate
/// [`a_lista_do_censo_cobre_os_knobs_incondicionais`] prende as duas.
pub(super) const KNOBS: &[Knob] = &[
    Knob {
        rotulo: "panel.sculpt3d.radius",
        a: |x| x.radius = RAIO,
        b: |x| x.radius = RAIO * 1.6,
    },
    Knob {
        rotulo: "panel.sculpt3d.strength",
        a: |x| x.strength = 0.1,
        b: |x| x.strength = 1.0,
    },
    Knob {
        rotulo: "panel.sculpt3d.hardness",
        a: |x| x.hardness = 0.0,
        b: |x| x.hardness = 0.95,
    },
    Knob {
        rotulo: "panel.sculpt3d.auto_smooth",
        a: |x| x.auto_smooth = 0.0,
        b: |x| x.auto_smooth = 1.0,
    },
    // ⚠️ **A CURVA não é uma fileira de slider — é a fileira de chips** —, e
    // por isso ela não tem `Row`. Ela é pintada por
    // `paint/brush.rs::paint_falloff_row` com uma cerca ESCRITA (*«o painel de
    // queda é dobrado, nunca ausente»*, decisão portada), logo ela é o knob
    // incondicional por excelência.
    Knob {
        rotulo: "panel.sculpt3d.falloff",
        a: |x| x.falloff = Falloff::Constant,
        b: |x| x.falloff = Falloff::Sharper,
    },
];

/// ⛔⛔ **O CONTROLO POSITIVO POR VERBO: este verbo FAZ alguma coisa neste
/// arnês?**
///
/// ⚠️⚠️ **Sem ele o censo MENTE, e a 1.ª corrida provou-o:** ele leu `32` knobs
/// mortos, e o `Clay Thumb` aparecia com **quatro de cinco** — um verbo de
/// carimbo, com lei por-vértice, cujos `Radius`/`Strength`/`Hardness`/curva
/// liam todos `0,000e0`. A causa não era nenhum knob: **o verbo não move um
/// único vértice neste arranjo**, e o que se mexia era o *auto-smooth* (todos
/// eles liam exactamente `4,001e-2`, que é a assinatura de o verbo contribuir
/// zero).
///
/// ⇒ *um censo que mede um verbo INERTE acusa cinco knobs vivos de uma vez*, e
/// é a terceira forma desta armadilha nesta crate (a `alvo_sintetico` e a
/// `referencia_sintetica` são as outras duas).
///
/// A régua é a mais crua possível: o verbo com os defaults dele contra a peça
/// **por tocar**. Se nada se mexe, a linha dele não é *«morta»* — é **NÃO
/// MEDIDA**, e o gate [`o_censo_nomeia_os_verbos_que_este_arnes_nao_acorda`]
/// obriga-a a ser nomeada em vez de silenciada.
pub(super) fn acorda_neste_arnes(verb: Verb) -> bool {
    let x = Brush {
        // ⚠️ O *auto-smooth* é DESARMADO aqui de propósito: ele move barro
        // sozinho, e com ele ligado todo verbo leria «acorda».
        auto_smooth: 0.0,
        ..pincel(verb)
    };
    desvio(&peca_de(verb), &corre(&x)) > 0.0
}

/// O retrato do painel com este verbo na mão — o que ele PINTARIA.
pub(super) fn painel_com(verb: Verb) -> Sculpt3dUi {
    let slot = VerbSlot::for_verb(verb);
    Sculpt3dUi {
        brush: slot.brush,
        radius_px: slot.radius_px,
        // ⚠️ **O nível mais LARGO de propósito:** no `Basic` o painel dobra
        // metade das fileiras, e um censo corrido ali leria *«escondido»* sobre
        // knobs que o artista alcança com um clique. *A pergunta é o que ele
        // PODE ver, não o que vê agora.*
        ui_level: UiLevel::Pro,
        ..Sculpt3dUi::default()
    }
}

/// O painel pinta esta fileira com este verbo na mão?
///
/// ⚠️ **A resposta sai da TABELA do painel**, nunca de uma lista escrita aqui:
/// uma segunda cópia da condição divergiria na primeira wave que mexesse numa
/// delas, e a que o artista vê é a que envelhece.
pub(super) fn pintado(ui: &Sculpt3dUi, rotulo: &str) -> bool {
    // A curva não é um `Row` — ela é a fileira de chips, e a cerca dela é
    // incondicional por decisão portada (ver o `Knob` dela).
    if rotulo == "panel.sculpt3d.falloff" {
        return true;
    }
    rows()
        .find(|r| r.label == rotulo)
        .is_some_and(|r| r.visible(ui))
}
