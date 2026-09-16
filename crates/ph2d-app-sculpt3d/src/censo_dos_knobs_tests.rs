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

/// O ARNÊS — a peça, o gesto, o traço e a régua. Ver [`arnes`].
#[path = "censo_dos_knobs_arnes.rs"]
mod arnes;

use arnes::{KNOBS, acorda_neste_arnes, corre, desvio, painel_com, pincel, pintado, quanto_move};
use ph2d_panel_sculpt3d::rows::{Place, SECTIONS};
use ph2d_sculpt3d::{Brush, Falloff, Verb};

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
        Verb::Density,
        "panel.sculpt3d.falloff",
        "ELE NÃO TEM LEI POR-VÉRTICE: o efeito inteiro dele é sobre a \
         TOPOLOGIA, e o `dab` sai antes de a cadeia de peso existir. A fileira \
         da curva é a única que o painel pinta SEMPRE (cerca de produto medida \
         e gateada), logo ela não pode ser escondida como o `Strength` e o \
         `Auto-Smooth` dele foram — e é por isso que ela leva a RAZÃO À VISTA \
         (`ph2d_sculpt3d::Brush::curva_inerte`), que a terceira metade do gate \
         abaixo exige de toda entrada desta lista. ⚠️ Ele só entrou aqui em \
         2026-09-15 porque até então era ADORMECIDO — e um verbo que o censo \
         não acorda tem os cinco knobs por medir, não zero mortos",
    ),
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
        // ⛔⛔⛔ **E ELA VOLTOU A UM em 2026-09-15, no mesmo dia**, com o verbo
        // que a esvaziara ainda fresco. O [`Verb::BoxTrim`] está aqui **por
        // LEI, e pela razão mais forte de toda a lista**: os outros tinham lei
        // noutro sítio do mesmo gesto; este **não tem gesto de carimbo
        // nenhum** — o arrasto dele é interceptado antes de haver dab, e o que
        // muda a peça é uma booleana sobre a malha inteira, no pen-up.
        //
        // ⚠️ **Não há saída prescrita, e isso é a diferença:** a entrada do
        // `Density` nomeava a dela (*correr o passe e comparar a contagem*) e
        // foi por ali que ele saiu. Medir este num dab é medir o programa
        // errado — o arnês teria de desenhar uma forma de ecrã e correr uma
        // booleana, que é o que a bancada da `ph2d-trim` já faz do lado onde a
        // lei vive.
        (
            Verb::BoxTrim,
            "por LEI: nao ha' dab -- o gesto e' interceptado e o corte e' uma \
             booleana no pen-up (a lei tem bancada propria na `ph2d-trim`)",
        ),
        // ⭐⭐⭐ **ELE É O ÚNICO QUE SOBRA, e a catraca desceu de SEIS para um
        // em 2026-09-15.** Os cinco que saíram não foram reclassificados —
        // **acordaram**, e a causa foi uma só: o arnês entregava um CARIMBO e
        // quatro daquelas leis precisam de um TRAÇO (o [`Dab::path`] sai da
        // diferença entre centros consecutivos), e o quinto precisava de uma
        // peça com **bordo aberto**, pela porta do produto.
        //
        // ⭐⭐⭐ **E ELA FOI A ZERO no mesmo dia, com o último a sair pela saída
        // que a própria entrada dele nomeava.** O [`Verb::Density`] esteve aqui
        // *por LEI*: o efeito dele é sobre a TOPOLOGIA, e o passe de topologia
        // não corre dentro do `dab` — *não é um verbo inerte, é um verbo cuja
        // lei não vive ali*. A entrada prescrevia **correr o passe e comparar a
        // CONTAGEM em vez das posições**, e é exactamente o que o
        // [`corre`] passou a fazer, pela porta do produto
        // ([`crate::dyntopo::passe_nos_motores`], com dois chamadores).
        //
        // ⚠️⚠️ **Uma catraca VAZIA não é uma catraca morta — é a mais apertada
        // que existe:** a metade de cima reprova **qualquer** verbo inerte que
        // nasça, e já não há uma linha onde alguém o possa escrever calado.
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
    // ⭐⭐ **E o piso, que é a metade que impede um verbo de adormecer calado:**
    // a afirmação é *«o arnês acorda TODOS menos os que a catraca nomeia»*, e
    // ela reprova por qualquer um dos lados — um verbo a mais adormecido, ou
    // uma entrada da catraca que já não descreve nada.
    //
    // ⚠️⚠️ **A redacção anterior dizia *«a catraca está VAZIA»* e comparava com
    // `Verb::ALL.len()`.** Ela nasceu no dia em que a lista chegou a zero e
    // morreu no dia seguinte, com o `BoxTrim` — *uma asserção escrita sobre o
    // estado de HOJE reprova sobre produto correcto no dia em que o estado
    // legítimo muda*. O que fica é a relação, que vale nos dois estados.
    assert_eq!(
        Verb::ALL.iter().filter(|v| acorda_neste_arnes(**v)).count(),
        Verb::ALL.len() - ADORMECIDOS.len(),
        "algum verbo deixou de acordar sem passar pela catraca — e enquanto ele \
         dormir, os knobs dele ficam POR MEDIR, que é onde o próximo «não vejo \
         efeito» nasce"
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
        // ⛔⛔⛔ **A POPULAÇÃO são os verbos que CARIMBAM, e ela deixou de ser
        // `Verb::ALL` em 2026-09-15** — com o `Verb::BoxTrim` a lista dos
        // sempre-visíveis ficou **VAZIA** (ele não tem raio nem força), e esta
        // metade passou a medir o vácuo. *A pergunta sempre foi «que knob todo
        // PINCEL pinta?», e a resposta era `Verb::ALL` só enquanto todo verbo
        // era um pincel.*
        .filter(|r| {
            Verb::ALL
                .iter()
                .filter(|v| v.writes_through_applicator())
                .all(|&v| r.visible(&painel_com(v)))
        })
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
    // ⛔⛔ **O piso de população era `2`, encolheu para `1` em 2026-09-15 e no
    // MESMO DIA a lista foi a ZERO** — e as duas quedas têm causas diferentes.
    // A primeira: o `Strength` deixou de ser incondicional porque o `Density`
    // não o lê (medido `0,000e0` no barro). A segunda: o `BoxTrim` não tem raio
    // NEM força, e com ele dentro da população nem o raio sobrava.
    //
    // ⭐ **A cura não foi baixar o piso outra vez — foi corrigir a POPULAÇÃO**
    // (ver o filtro acima). *Um piso que segurasse o número enquanto a lista
    // esvaziava mediria uma lista que já não existe* — é a forma que o
    // `CLAUDE.md` §5 nomeia: **o piso segura o NÚMERO enquanto a POPULAÇÃO
    // troca por baixo dele** —, e desta vez o que estava errado era quem entrava
    // na conta.
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
