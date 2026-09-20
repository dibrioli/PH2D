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

/// **A LISTA DOS MORTOS CONHECIDOS** — ver [`mortos`]. Irmã (`#[path]`),
/// cortada pelo ASSUNTO e pelo tecto de LOC: *uma tabela de dívida com a razão
/// de cada entrada cresce por wave, e o ficheiro que a lê não tem de crescer
/// com ela.*
#[path = "censo_dos_knobs_mortos.rs"]
mod mortos;
use mortos::MORTOS_CONHECIDOS;

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
            d.label_key()
        );
    }
}

/// ⭐⭐⭐ **G-13 — CADA VERBO LÊ O CORTE QUE O NOSSO PAINEL LHE OFERECE**
/// (`SPEC_pincel_de_plano.md` §12).
///
/// ⛔ **É o gate contra o KNOB MORTO, e ele é o irmão de MAGNITUDE do censo
/// deste ficheiro.** O censo pergunta *«a saída muda ao BIT?»* e essa pergunta
/// tem um ponto cego: um knob que mude o barro em `1e-9` está vivo **ao bit** e
/// morto **para o artista**. ⚠️ *É a mesma cegueira que o `Density` pagou com
/// uma foto do dono* — a régua dizia `depois < antes` e a colheita medida era de
/// `1 %`, invisível. ⇒ aqui a barra é uma **quantidade**.
///
/// # A barra, e de onde ela vem
///
/// **`1e-4`**, e ela sai do **lado aprovado**: a menor mudança que o alvo produz
/// num knob VIVO entre duas fixturas **publicadas** é `7,3e-03`
/// (`corte/corte_tiras_05` contra `corte/corte_tiras_desligado`) ⇒ a barra fica
/// **`73×` abaixo** dela. ⛔ A 1.ª redacção da espec citava `1,1e-02`, medido mas
/// **sem fixtura publicada** — *uma barra derivada de um número que a página não
/// carrega não é verificável por quem a lê*.
///
/// # A população é DERIVADA do painel, nunca escrita aqui
///
/// Para cada `(verbo, knob)` a pergunta é *«o painel PINTA isto com este verbo na
/// mão?»*, respondida pela tabela de fileiras ([`arnes::pintado`]). ⛔ Uma lista
/// à mão divergiria da tabela na primeira wave que mexesse numa das duas, e a que
/// o artista vê é a que envelhece.
///
/// ⚠️ **Os dois tectos varrem-se com o OUTRO preso em `1`**, e isso é a espec
/// §3.1: com os dois a zero o pincel fica **inerte sem deixar de existir** (o
/// *nada* do controlo, que o G-8 mede), e uma varredura `0/0` contra `1/0` leria
/// a morte do pincel como se fosse a morte do knob.
#[test]
fn cada_verbo_le_o_corte_que_o_nosso_painel_lhe_oferece() {
    /// `73×` abaixo da menor mudança publicada num knob vivo do alvo.
    const BARRA: f32 = 1e-4;
    /// ⭐ **PISO DE POPULAÇÃO** — `6` células, **CONTADAS e não escolhidas** (o
    /// gate imprime-as). Sem ele, esconder os três knobs deixaria o gate **verde
    /// a medir nada** (`CLAUDE.md` §5.0).
    ///
    /// ⚠️⚠️ **Ela foi a `7` e voltou a `6` no mesmo dia.** Este gate imprimiu a
    /// população e mostrou que o `Plane × plane_offset` não estava lá; eu li isso
    /// como um **controlo inalcançável** e liguei a fileira — e o smoke do dono
    /// devolveu-a: *«Plane Offset com resultado completamente errado»*. ⛔ A
    /// ausência era **certa**; o que lhe faltava era o **motivo escrito**.
    ///
    /// ⭐ *E este gate não me podia proteger disso:* ele pergunta se o knob MOVE
    /// o barro, e aquele move de mais. Quem afirma a ausência agora é o irmão
    /// [`o_deslocamento_do_plano_nao_e_oferecido_ao_pincel_de_plano`], com a
    /// tabela medida no doc do `Verb::uses_plane`.
    const PISO: usize = 6;

    /// Como se põe um knob num dos dois extremos da varredura.
    type Varredura = fn(&mut Brush, bool);

    /// `(rótulo da fileira, como se varre o knob)` — os três que a §12 nomeia.
    const CORTE_E_DESLOCAMENTO: [(&str, Varredura); 3] = [
        ("panel.sculpt3d.plano_altura", |b, alto| {
            b.plano_altura = if alto { 1.0 } else { 0.0 };
            b.plano_profundidade = 1.0;
        }),
        ("panel.sculpt3d.plano_profundidade", |b, alto| {
            b.plano_profundidade = if alto { 1.0 } else { 0.0 };
            b.plano_altura = 1.0;
        }),
        ("panel.sculpt3d.plane_offset", |b, alto| {
            b.plane_offset = if alto { 0.5 } else { 0.0 };
        }),
    ];

    let mut medidos = 0usize;
    let mut celulas: Vec<String> = Vec::new();
    let mut pior = (f32::INFINITY, String::new());
    for verb in Verb::ALL {
        let ui = painel_com(verb);
        for (rotulo, aplicar) in CORTE_E_DESLOCAMENTO {
            if !pintado(&ui, rotulo) {
                continue;
            }
            let (mut a, mut b) = (pincel(verb), pincel(verb));
            aplicar(&mut a, true);
            aplicar(&mut b, false);
            let d = desvio(&corre(&a), &corre(&b));
            assert!(
                d >= BARRA,
                "{verb:?} × `{rotulo}`: varrer o knob move o barro {d:.3e}, e a \
                 barra e' {BARRA:.0e} — o painel oferece um controlo que este \
                 verbo nao le^, que e' o knob MORTO do §5.0"
            );
            if d < pior.0 {
                pior = (d, format!("{verb:?} × {rotulo}"));
            }
            celulas.push(format!(
                "{verb:?}/{}",
                rotulo.rsplit('.').next().unwrap_or(rotulo)
            ));
            medidos += 1;
        }
    }
    assert_eq!(
        medidos, PISO,
        "o G-13 mediu {medidos} celulas e a populacao pintada e' {PISO} — se ela \
         encolheu, um knob deixou de ser oferecido; se cresceu, um verbo novo \
         passou a oferecer o corte e ninguem o mediu"
    );
    println!(
        "G-13: {medidos} celulas [{}], a mais fraca move {:.3e} em {}",
        celulas.join(" · "),
        pior.0,
        pior.1
    );
}

/// ⛔⛔⛔ **O DESLOCAMENTO DO PLANO NÃO É OFERECIDO AO PINCEL DE PLANO — e isso é
/// uma DECISÃO, não um esquecimento.**
///
/// Veredito do dono, 2026-09-17, com foto: *«Plane Offset com resultado
/// completamente errado. Plane Offset = 0 correto; −0,5 bizarro»*.
///
/// # As DUAS metades, e a segunda é a que torna a primeira honesta
///
/// 1. o painel **não pinta** a fileira com este verbo na mão;
/// 2. a **LEI continua a lê-lo** — o mesmo gesto com `0` e com `−0,5` dá barro
///    diferente. ⛔ Sem esta metade, alguém leria a ausência como *«o verbo não
///    tem deslocamento»* e apagaria a lei, levando as duas fixturas do corpus
///    (`lei_deslocado_m02` e `_p02`) e a §2.4 da espec com ela.
///
/// ⚠️ *Esconder um knob VIVO e esconder um knob MORTO leem-se igual numa tabela;
/// o que os separa é a medição escrita ao lado* — e ela está no doc do
/// [`ph2d_sculpt3d::Verb::uses_plane`], com as sete posições do slider.
///
/// ⏳ **O que falta para o reabrir é um NÚMERO:** a faixa `−1 … +1` que a fileira
/// herdaria dos quatro verbos da casa tem **metade do curso inerte** e a outra
/// metade a **dobrar o corte**; uma faixa mais estreita teria de sair de um
/// recurso medido, e o que a limita aqui é a altura do relevo em raios de
/// pincel, que é da PEÇA e não do produto.
#[test]
fn o_deslocamento_do_plano_nao_e_oferecido_ao_pincel_de_plano() {
    let ui = painel_com(Verb::Plane);
    assert!(
        !pintado(&ui, "panel.sculpt3d.plane_offset"),
        "o painel voltou a oferecer o `Plane Offset` ao pincel de plano — e a \
         medicao que o tirou esta' no doc do `Verb::uses_plane`: metade do curso \
         e' inerte e a outra dobra o corte por passagem"
    );

    // ⭐ **A metade que impede a leitura errada:** a lei LÊ o knob, e é por isso
    // que o que está escondido é um controlo e não uma capacidade.
    let (mut a, mut b) = (pincel(Verb::Plane), pincel(Verb::Plane));
    a.plane_offset = 0.0;
    b.plane_offset = -0.5;
    let d = desvio(&corre(&a), &corre(&b));
    assert!(
        d > 1e-3,
        "o deslocamento deixou de mover o barro deste verbo ({d:.3e}) — entao o \
         que esta' escondido ja' nao e' um controlo, e' uma lei morta: ou ela \
         volta, ou as duas fixturas `lei_deslocado_*` deixam de medir o produto"
    );
    println!("o deslocamento do pincel de plano: escondido, e VIVO ({d:.3e})");
}
