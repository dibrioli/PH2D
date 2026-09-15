//! ⭐⭐⭐ **O CENSO DOS KNOBS QUE CHEGAM — por VERBO, e medido no BARRO.**
//!
//! # ⛔⛔ A pergunta que nenhum instrumento deste repo fazia
//!
//! O `CLAUDE.md` §5.0 escreve-o com todas as letras: *«nenhum instrumento do
//! repo pergunta se o VALOR chega a um consumidor»*. O
//! `architecture_panel_wiring_parity` mede **focalizabilidade**, os `seam_*`
//! provam que o clique **chega à ferramenta**, e o
//! [`crate::censo_das_fileiras_tests`](ph2d_panel_sculpt3d) prova que **todo
//! valor do motor tem chip**. Nenhum deles olha para o barro.
//!
//! ⚠️ **E o módulo passou de `24` para `32` verbos em três dias**, com o painel
//! a pintar as mesmas quatro fileiras sempre. *Um knob pintado que o verbo em
//! mãos não lê é a espécie que o dono reporta como «não vejo efeito» — e três
//! dos últimos reports dele foram exactamente isso.*
//!
//! # A régua
//!
//! Para cada `(verbo, knob)`: **o mesmo gesto, duas posições do knob, as
//! posições comparadas bit a bit**. É a régua que o irmão
//! `measure_where_the_curve_knobs_reach` já usa para **dois** knobs em **três**
//! regimes; aqui ela é varrida sobre a matriz inteira.
//!
//! ⛔ **A régua é o PRODUTO** (`SculptStroke::dab`), nunca as funções soltas:
//! um `Falloff::weight` correcto não prova um pincel que o consome.
//!
//! ⚠️⚠️ **E o gesto é o do GRIP, não um dab genérico** — um `Dab::at` entregue
//! a um verbo de âncora tem `pull` nulo e o verbo é **inerte por lei**. *Um
//! censo que mede um verbo inerte lê `0,000` em toda a linha e acusa cinco
//! knobs mortos que estão vivos.* É a armadilha que a `alvo_sintetico` e a
//! `referencia_sintetica` já pagaram nesta crate, aqui numa terceira forma.
//!
//! # As DUAS metades, e a acusação é a interseção
//!
//! | | o knob CHEGA | o knob NÃO chega |
//! |---|---|---|
//! | o painel **PINTA** | ✅ | ⛔ **o morto** |
//! | o painel **esconde** | ⛔ o inalcançável | ✅ |
//!
//! ⚠️ **As duas colunas erradas têm curas OPOSTAS** (§5.0: *o morto liga-se, o
//! órfão apaga-se*), e é por isso que este censo mede as duas e não uma.

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
fn peca_de(verb: Verb) -> Mesh {
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

const PONTA: [f32; 3] = [0.0, 0.0, 1.0];
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];
const RAIO: f32 = 0.5;

/// **ONDE O TRAÇO COMEÇA** — o ápice da peça deste verbo.
///
/// ⚠️ Para quem precisa de bordo é a **BEIRA** (o vértice mais alto da tigela é
/// da boca dela), e para os outros é o pólo. *Um cursor longe do sujeito do
/// verbo mede um gesto que não acontece.*
fn onde(verb: Verb, mesh: &Mesh) -> [f32; 3] {
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
fn gesto(verb: Verb, raio: f32, centro: [f32; 3], k: usize) -> Dab {
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
fn referencia(mesh: &Mesh) -> Vec<[f32; 3]> {
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
fn alvo() -> Vec<(Mesh, ph2d_mesh::Pose)> {
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
const DABS_DO_TRACO: usize = 2;

/// **UM TRAÇO PELO CAMINHO DO MOTOR**, com tudo o que o pen-down fotografaria.
///
/// ⚠️⚠️ **O raio do dab É o `Brush::radius`, e isso é a LEI do produto, não uma
/// conveniência do arnês** — [`o_raio_do_dab_sai_do_pincel`] prende-a. A 1.ª
/// redacção deste ficheiro carregava um raio **próprio** ao lado do pincel, e o
/// que ela mediu foi outro programa: o `Verb::Pose` lê `brush.radius` (a lei
/// dele não tem dab por-vértice nenhum), logo mexer só no raio do dab deixava-o
/// a `0,000e0` e o censo acusava-o de morto. *Dois números onde o produto tem
/// um é a forma mais barata de uma régua mentir.*
fn corre(b: &Brush) -> Mesh {
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
    for k in 0..DABS_DO_TRACO {
        // ⚠️ Os centros ANDAM — é a diferença entre eles que vira o
        // [`Dab::path`], e é o `path` que acorda os quatro verbos de caminho.
        let passo = 0.12 * k as f32;
        let mut d = gesto(b.verb, b.radius, centro, k);
        d.center = [centro[0] + passo, centro[1], centro[2]];
        s.dab(&mut mesh, b, &d, Symmetry::default());
    }
    mesh
}

/// O maior desvio entre duas saídas — **posição E máscara**, porque há verbos
/// que só escrevem o canal (`Verb::Mask`) e um censo que olhasse só posições
/// acusaria todos os knobs dele.
fn desvio(a: &Mesh, b: &Mesh) -> f32 {
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
fn pincel(verb: Verb) -> Brush {
    Brush {
        radius: RAIO,
        ..VerbSlot::for_verb(verb).brush
    }
}

/// Um KNOB do painel: o rótulo que ele mostra, as duas posições que o censo
/// varre, e a pergunta *«o painel pinta-o com este verbo?»*.
struct Knob {
    /// A chave de i18n, que é como a fileira se chama no painel.
    rotulo: &'static str,
    /// As duas posições. ⚠️ Elas são **extremos da faixa do próprio painel**,
    /// nunca números escolhidos: um par apertado lê `0,000` sobre um knob vivo.
    a: fn(&mut Brush),
    b: fn(&mut Brush),
}

/// **QUANTO BARRO ESTE KNOB MOVE COM ESTE VERBO** — as duas posições, o mesmo
/// gesto, e o maior desvio entre as duas saídas.
///
/// ⚠️ Vive numa porta só porque a sonda e o portão fazem a **mesma** pergunta:
/// duas cópias divergiriam na primeira wave que mexesse numa delas, e a que o
/// portão usa é a que decide.
fn quanto_move(verb: Verb, k: &Knob) -> f32 {
    let (mut x, mut y) = (pincel(verb), pincel(verb));
    (k.a)(&mut x);
    (k.b)(&mut y);
    desvio(&corre(&x), &corre(&y))
}

/// ⚠️ **São os knobs que o painel pinta SEM perguntar pelo verbo** — os outros
/// já têm `show` por verbo e portanto não podem estar mortos por construção.
/// A lista sai da tabela `BRUSH` de `rows.rs`, e o gate
/// [`a_lista_do_censo_cobre_os_knobs_incondicionais`] prende as duas.
const KNOBS: &[Knob] = &[
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
fn acorda_neste_arnes(verb: Verb) -> bool {
    let x = Brush {
        // ⚠️ O *auto-smooth* é DESARMADO aqui de propósito: ele move barro
        // sozinho, e com ele ligado todo verbo leria «acorda».
        auto_smooth: 0.0,
        ..pincel(verb)
    };
    desvio(&peca_de(verb), &corre(&x)) > 0.0
}

/// O retrato do painel com este verbo na mão — o que ele PINTARIA.
fn painel_com(verb: Verb) -> Sculpt3dUi {
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
fn pintado(ui: &Sculpt3dUi, rotulo: &str) -> bool {
    // A curva não é um `Row` — ela é a fileira de chips, e a cerca dela é
    // incondicional por decisão portada (ver o `Knob` dela).
    if rotulo == "panel.sculpt3d.falloff" {
        return true;
    }
    rows()
        .find(|r| r.label == rotulo)
        .is_some_and(|r| r.visible(ui))
}

/// **SONDA — a matriz inteira**, para a tabela poder ser lida de uma vez.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --lib diag_o_censo_dos_knobs -- --ignored --nocapture
/// ```
#[test]
#[ignore]
fn diag_o_censo_dos_knobs() {
    eprint!("{:<18}", "verbo");
    for k in KNOBS {
        eprint!("{:>22}", k.rotulo.trim_start_matches("panel.sculpt3d."));
    }
    eprintln!();
    let mut mortos = 0usize;
    let mut adormecidos = 0usize;
    for &verb in &Verb::ALL {
        let ui = painel_com(verb);
        let vivo = acorda_neste_arnes(verb);
        if !vivo {
            adormecidos += 1;
        }
        eprint!("{:<18}{}", verb.label(), if vivo { " " } else { "z" });
        for k in KNOBS {
            let d = quanto_move(verb, k);
            let p = pintado(&ui, k.rotulo);
            let marca = match (vivo, p, d > 0.0) {
                // ⛔ O verbo não acorda neste arranjo: a linha inteira é NÃO
                // MEDIDA, e chamar-lhe «morta» seria acusar knobs vivos.
                (false, _, _) => "?",
                (true, true, true) => "ok",
                (true, true, false) => {
                    mortos += 1;
                    "MORTO"
                }
                (true, false, true) => "escondido/vivo",
                (true, false, false) => "-",
            };
            eprint!("{:>14.3e} {marca:>7}", d);
        }
        eprintln!();
    }
    eprintln!(
        "\nknobs PINTADOS que não chegam ao barro: {mortos}\n         verbos que este arnês não acorda (linha `z`, NÃO MEDIDA): {adormecidos}"
    );
}

/// ⛔⛔ **O RAIO DO DAB SAI DO PINCEL — a lei que este censo ADIVINHOU e errou.**
///
/// A 1.ª redacção carregava um raio próprio ao lado do `Brush`, porque a fileira
/// do painel escreve em `radius_px` e não no pincel. A cadeia real, porém, junta
/// os dois **antes** do dab: o `armed_brush` converte pixels em mundo, escreve
/// `Brush::radius`, e **todo** construtor de `Dab` do produto lê esse campo. ⇒ o
/// arnês media um programa em que o raio do pincel ficava parado, e o
/// `Verb::Pose` — cuja lei lê `brush.radius` e não tem dab por-vértice nenhum —
/// aparecia com o raio MORTO.
///
/// ⚠️ **`include_str!` e não uma lista escrita aqui:** se um construtor mudar de
/// ficheiro isto **deixa de compilar**, em vez de ficar verde a medir menos
/// (`HOWTO §2.6`).
#[test]
fn o_raio_do_dab_sai_do_pincel() {
    // Os dois ficheiros por onde TODO gesto de escultura passa: o carimbo
    // (`sculpt_at`) e os quatro grips de arrasto.
    const FONTES: &[(&str, &str)] = &[
        ("input.rs", include_str!("input.rs")),
        ("pull.rs", include_str!("pull.rs")),
    ];
    let mut achados = 0usize;
    for (nome, src) in FONTES {
        for linha in src.lines() {
            let Some(resto) = linha.split_once("Dab::").map(|(_, r)| r) else {
                continue;
            };
            let Some(args) = resto.split_once('(').map(|(_, a)| a) else {
                continue;
            };
            // `Dab::<construtor>(centro, RAIO, ...)` — o raio é o 2.º argumento.
            let Some(raio) = args.split(',').nth(1).map(str::trim) else {
                continue;
            };
            achados += 1;
            assert!(
                raio.ends_with(".radius"),
                "{nome}: `{linha}` entrega ao dab um raio que não é o do \
                 pincel — um segundo raio deixa o `Verb::Pose` (que lê \
                 `brush.radius`) a discordar do carimbo, e o censo dos knobs \
                 passa a medir outro programa"
            );
        }
    }
    assert!(
        achados >= 5,
        "achei só {achados} construções de dab nestes ficheiros — o piso de \
         população: se elas mudarem de sítio este gate fica verde a varrer nada"
    );
}

/// ⛔⛔⛔ **A CATRACA DOS KNOBS MORTOS — e é ela que faz disto um PORTÃO e não um
/// relatório.**
///
/// Cada entrada é um `(verbo, knob)` que o painel **PINTA** e que o barro
/// **não sente**, com o motivo ao lado. As duas metades obrigatórias:
///
/// * um morto **NOVO** reprova ⇒ um verbo não pode nascer a oferecer um knob
///   que ele não lê;
/// * um morto **CURADO** reprova ⇒ a lista só desce, e uma entrada que já não
///   descreve nada é a catraca a virar **licença** (§5.0).
///
/// ⚠️ **O motivo é o que separa uma DIVERGÊNCIA de uma DÍVIDA**, e as duas
/// entradas que sobram são **divergências medidas**: as duas são a fileira da
/// CURVA, que o painel pinta **sempre** por uma cerca de produto escrita e
/// gateada (ver [`pintado`] e o cabeçalho do `paint/brush.rs`), sobre verbos que
/// a leem noutro regime ou não a leem de todo.
///
/// ⭐⭐⭐ **A lista desceu de `5` para `2` em 2026-09-15, e NENHUMA das três que
/// saíram saiu por ser reclassificada** — cada uma teve uma causa medida:
/// - `Pose × radius` era **a régua**: este ficheiro carregava um raio próprio ao
///   lado do pincel e o produto tem **um** ([`o_raio_do_dab_sai_do_pincel`]);
/// - `Pose × hardness` era **a lente do painel**, mais larga que a do
///   consumidor: a fileira já tinha a porta certa (`shapes_the_distance`) e
///   faltava-lhe o lado do VERBO
///   ([`ph2d_sculpt3d::Verb::a_lei_le_a_distancia_ao_cursor`]);
/// - `Pose × auto_smooth` era **dívida real e foi CONSTRUÍDA**: a espec do
///   pincel prescreve a lei (§15 e item 18 — *«ela segue os pesos, não o
///   raio»*), e hoje ela corre em `ph2d_sculpt3d::stroke_pose::alisa_a_pose`.
///
/// ⚠️⚠️ **E na MESMA jornada o arnês acordou cinco verbos e as `25` células
/// novas acusaram DOIS mortos — os dois curados por ESCONDER, não por ligar:**
/// `Cloth × auto_smooth` e `Boundary × auto_smooth` liam `0,000e0` porque os
/// dois desviam antes do laço por-vértice onde o passe corre. ⛔ Nenhuma das
/// duas especs prescreve auto-suavização para aquele pincel, e *inventar uma lei
/// para um pincel de clean-room sem referência é o que a parede existe para
/// impedir* ⇒ o painel deixa de a pintar
/// ([`ph2d_sculpt3d::Verb::o_auto_smooth_chega`], com as duas saídas nomeadas lá
/// dentro). *Um censo que mede mais encontra mais, e é por isso que acordar um
/// verbo vale mais do que curar um knob.*
const MORTOS_CONHECIDOS: &[(Verb, &str, &str)] = &[
    (
        Verb::Mask,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada: o canal tem a SEGUNDA curva da referência, e a \
         do carimbo não o alcança (recusa medida, com dois gates a defendê-la)",
    ),
    (
        Verb::Pose,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada, e o knob NÃO está morto: a espec dele (§1.2) diz \
         que só o modo de TORÇÃO lê a curva, e este censo mede o modo de \
         OMISSÃO. Onde ela é lida, ela chega — gate \
         `a_curva_do_pincel_chega_ao_modo_de_torcao`, que foi escrito porque ela \
         NÃO chegava (a ponte entre as duas convenções não invertia o argumento, \
         e a torção com o valor de fábrica era inerte). ⚠️ A fileira é pintada \
         sempre por cerca de produto MEDIDA, e esconder um knob vivo noutro modo \
         seria o defeito oposto",
    ),
];

/// **GATE — a lista dos mortos é EXACTA nos dois sentidos.**
#[test]
fn o_censo_dos_knobs_mortos_so_desce() {
    let mut medidos: Vec<(Verb, &'static str)> = Vec::new();
    for &verb in &Verb::ALL {
        if !acorda_neste_arnes(verb) {
            continue;
        }
        let ui = painel_com(verb);
        for k in KNOBS {
            if pintado(&ui, k.rotulo) && quanto_move(verb, k) == 0.0 {
                medidos.push((verb, k.rotulo));
            }
        }
    }
    let novos: Vec<_> = medidos
        .iter()
        .filter(|(v, r)| !MORTOS_CONHECIDOS.iter().any(|(w, q, _)| w == v && q == r))
        .map(|(v, r)| (v.label(), *r))
        .collect();
    assert!(
        novos.is_empty(),
        "knobs PINTADOS que o barro não sente e que ninguém nomeou: {novos:?} — \
         um controlo que o artista arrasta e não faz nada é a espécie que ele \
         reporta como «não vejo efeito»"
    );
    let obsoletos: Vec<_> = MORTOS_CONHECIDOS
        .iter()
        .filter(|(v, r, _)| !medidos.iter().any(|(w, q)| w == v && q == r))
        .map(|(v, r, _)| (v.label(), *r))
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estes knobs JÁ chegam ao barro e a lista não desceu: {obsoletos:?} — \
         apague-os, senão a catraca vira licença"
    );
    // ⭐ **O piso de população:** se esta lista esvaziar sem o censo a varrer
    // nada, o gate ficava verde sobre o VÁCUO — a forma que o §5.0 nomeia.
    assert!(
        Verb::ALL.iter().filter(|v| acorda_neste_arnes(**v)).count() >= 20,
        "o arnês deixou de acordar a maioria dos verbos — o censo passou a \
         medir quase nada e este gate ficaria verde por vácuo"
    );
    // ⭐⭐⭐ **E A TERCEIRA METADE, desde 2026-09-15: um knob morto que o painel
    // PINTA tem de ser EXPLICADO na tela.**
    //
    // ⛔⛔ *Nomear um morto num comentário de teste não o cura para o artista* —
    // ele continua a arrastar o controlo e a não ver nada. Os dois que sobram
    // são fileiras da CURVA, que o painel pinta **sempre** por cerca de produto
    // medida e gateada; a saída que não viola a cerca é a razão à vista, e ela
    // existe desde então ([`ph2d_sculpt3d::Brush::curva_inerte`]).
    //
    // ⚠️ **A régua é a PORTA que o painel consulta**, não uma segunda lista: se
    // a lei e o painel discordarem, quem o artista vê é o painel.
    for (verb, rotulo, _) in MORTOS_CONHECIDOS {
        if *rotulo != "panel.sculpt3d.falloff" {
            continue;
        }
        let b = pincel(*verb);
        assert!(
            b.curva_inerte().is_some(),
            "o {} está na lista dos mortos da CURVA e o painel não tem razão \
             nenhuma a mostrar — o artista arrasta os doze chips e o barro não \
             se mexe, sem uma palavra na tela",
            verb.label()
        );
    }
}

/// ⛔⛔ **OS VERBOS QUE ESTE ARNÊS NÃO ACORDA SÃO NOMEADOS, NUNCA SILENCIADOS.**
///
/// ⚠️ **É uma CATRACA de dívida com censo de obsolescência nos dois sentidos**
/// (§5.0): quem ensinar o arnês a acordar um deles **tem de o apagar daqui**, e
/// um verbo novo que nasça inerte **reprova** em vez de entrar calado na lista.
///
/// ⛔ **A alternativa era EXCLUIR estes verbos do censo, e um censo que exclui
/// um verbo deixa de o testar** — a mesma frase que a `alvo_sintetico` já
/// carrega. Aqui a lista é a **dívida escrita**, com o motivo de cada um.
#[test]
fn o_censo_nomeia_os_verbos_que_este_arnes_nao_acorda() {
    /// Cada entrada diz **porque** o arnês não o acorda — e é isso que separa
    /// uma dívida de uma isenção.
    const ADORMECIDOS: &[(Verb, &str)] = &[
        // ⭐⭐⭐ **ELE É O ÚNICO QUE SOBRA, e a catraca desceu de SEIS para um
        // em 2026-09-15.** Os cinco que saíram não foram reclassificados —
        // **acordaram**, e a causa foi uma só: o arnês entregava um CARIMBO e
        // quatro daquelas leis precisam de um TRAÇO (o [`Dab::path`] sai da
        // diferença entre centros consecutivos), e o quinto precisava de uma
        // peça com **bordo aberto**, pela porta do produto.
        //
        // ⛔ **Este não acorda por LEI, e não por dívida:** o efeito inteiro
        // dele é sobre a TOPOLOGIA, e o passe de topologia não corre dentro do
        // `dab`. *Não é um verbo inerte — é um verbo cuja lei não vive aqui.*
        //
        // ⏳ **A saída está nomeada:** o arnês teria de correr o
        // `refine_for_dab` (o passe que o `Density` arma) e comparar a
        // CONTAGEM de vértices em vez das posições — uma segunda régua, com
        // outra grandeza, e por isso um trabalho próprio e não uma linha aqui.
        (Verb::Density, "a lei é o passe de topologia, fora do `dab`"),
    ];
    let medidos: Vec<&'static str> = Verb::ALL
        .iter()
        .filter(|v| !acorda_neste_arnes(**v))
        .filter(|v| !ADORMECIDOS.iter().any(|(w, _)| w == *v))
        .map(|v| v.label())
        .collect();
    assert!(
        medidos.is_empty(),
        "verbos INERTES neste arnês e fora da lista: {medidos:?} — enquanto \
         eles não acordarem, o censo lê os knobs deles como mortos e acusa \
         controlos vivos"
    );
    let obsoletos: Vec<&'static str> = ADORMECIDOS
        .iter()
        .filter(|(v, _)| acorda_neste_arnes(*v))
        .map(|(v, _)| v.label())
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estes JÁ acordam e a catraca não desceu: {obsoletos:?} — apague-os da \
         lista, senão ela vira licença"
    );
}

/// ⛔⛔ **A LISTA DO CENSO COBRE OS KNOBS INCONDICIONAIS** — o piso de
/// população que impede este ficheiro de ficar verde a medir menos do que
/// promete.
///
/// ⚠️ **Sem ele, um knob novo `show: always` nasceria fora do censo e o censo
/// ficaria verde sobre ele** — a forma que o `CLAUDE.md` §5.0 chama de *censo
/// que varre zero e fica verde*, aqui na versão *varre menos*.
#[test]
fn a_lista_do_censo_cobre_os_knobs_incondicionais() {
    // Um verbo por FAMÍLIA de grip: um knob `show: always` aparece em todos, e
    // um que dependa do verbo não sobrevive à interseção.
    // ⚠️⚠️ **A população é a secção do PINCEL, e o gate ensinou-o na 1.ª
    // corrida:** ele acusou `extract_thickness`, `cavity`, `ao`, `ssao`,
    // `dyn_detail`, `remesh_res`, `quad_detail` e `quad_adapt` — nove rows
    // **sempre visíveis** que **não são knobs de pincel nenhum**: elas são
    // argumentos de BOTÕES (o extract, o remesh) e de PASSES (a sombra, a
    // topologia), e um dab não as lê **por desenho**.
    //
    // ⇒ *«sempre pintado» não é o mesmo que «pintado PARA o pincel»*, e o censo
    // que não os separasse acusaria oito controlos vivos de uma vez — a mesma
    // forma que o controlo positivo por verbo acabou de curar um nível abaixo.
    // ⚠️ A população sai da SECÇÃO declarada, nunca de uma lista escrita aqui.
    let seccao = SECTIONS
        .iter()
        .find(|s| s.id == ph2d_panel_sculpt3d::ids::SCULPT3D_SEC_BRUSH)
        .expect("o painel tem a secção do pincel");
    let sempre: Vec<&'static str> = seccao
        .rows
        .iter()
        // ⚠️⚠️ **E dentro da secção, só o BLOCO DE KNOBS** — a 2.ª corrida
        // acusou `extract_thickness` e `extract_smooth`, que vivem aqui e são
        // `Place::AfterExtract`: *argumentos de um BOTÃO*, colados a ele de
        // propósito. Um dab não os lê **por desenho**, e a `Place` é a porta
        // que o painel já declara — não uma lista escrita aqui.
        .filter(|r| r.place == Place::Knobs)
        .filter(|r| Verb::ALL.iter().all(|&v| r.visible(&painel_com(v))))
        .map(|r| r.label)
        .collect();
    let faltam: Vec<&&str> = sempre
        .iter()
        .filter(|l| !KNOBS.iter().any(|k| k.rotulo == **l))
        .collect();
    assert!(
        faltam.is_empty(),
        "o painel pinta {faltam:?} com TODO verbo e o censo não os varre — um \
         knob fora do censo é um knob que pode estar morto sem ninguém ver"
    );
    // ⛔⛔ **O piso de população era `2` e a POPULAÇÃO encolheu para `1` em
    // 2026-09-15**, sem o censo perder força: o `Strength` deixou de ser
    // incondicional porque o `Density` não o lê (medido `0,000e0` no barro), e o
    // que sobra sempre-visível é o RAIO. *Um piso que segurasse o número `2`
    // enquanto a lista era `{radius}` mediria uma lista que já não existe* — é a
    // forma que o `CLAUDE.md` §5 nomeia num censo de outra família: **o piso
    // segurou o NÚMERO enquanto a POPULAÇÃO trocava por baixo dele**.
    assert!(
        sempre.contains(&"panel.sculpt3d.radius"),
        "o RAIO deixou de ser incondicional ({sempre:?}) — se nem ele o for, \
         esta metade do censo passou a medir o vácuo"
    );
    // ⭐ **A anti-vácuo mudou de grandeza e não de força:** a população é a dos
    // knobs que o painel pinta para **quase** todo verbo, e é ela que tem de
    // estar coberta pelo censo.
    let quase_sempre: Vec<&'static str> = seccao
        .rows
        .iter()
        .filter(|r| r.place == Place::Knobs)
        .filter(|r| {
            Verb::ALL
                .iter()
                .filter(|&&v| r.visible(&painel_com(v)))
                .count()
                >= Verb::ALL.len() - 2
        })
        .map(|r| r.label)
        .collect();
    let faltam: Vec<&&str> = quase_sempre
        .iter()
        .filter(|l| !KNOBS.iter().any(|k| k.rotulo == **l))
        .collect();
    assert!(
        faltam.is_empty(),
        "o painel pinta {faltam:?} com quase todo verbo e o censo não os varre"
    );
    assert!(
        quase_sempre.len() >= 2,
        "o piso de população: o painel tem de ter pelo menos dois knobs que ele \
         pinta para quase todo verbo, e achei {quase_sempre:?}"
    );
}

/// **SONDA** — a curva do pincel chega ao barro em qual das cinco deformações
/// da pose? E o `Strength`?
#[test]
#[ignore]
fn diag_a_pose_por_deformacao() {
    for (d, segs) in ph2d_sculpt3d::PoseDeformacao::ALL
        .into_iter()
        .flat_map(|d| [1u32, 2, 4, 8].map(move |s| (d, s)))
    {
        let mede = |a: fn(&mut Brush), b: fn(&mut Brush)| {
            let (mut x, mut y) = (pincel(Verb::Pose), pincel(Verb::Pose));
            x.pose.deformacao = d;
            y.pose.deformacao = d;
            x.pose.segmentos = segs;
            y.pose.segmentos = segs;
            x.pose.arrasto_x_pixels = 40.0;
            y.pose.arrasto_x_pixels = 40.0;
            a(&mut x);
            b(&mut y);
            desvio(&corre(&x), &corre(&y))
        };
        let curva = mede(
            |x| x.falloff = Falloff::Constant,
            |x| x.falloff = Falloff::Sharper,
        );
        let forca = mede(|x| x.strength = 0.1, |x| x.strength = 1.0);
        let dureza = mede(|x| x.hardness = 0.0, |x| x.hardness = 0.95);
        eprintln!(
            "{:<17} seg {segs}: curva {curva:.3e} · forca {forca:.3e} · dureza {dureza:.3e}",
            d.label()
        );
    }
}
