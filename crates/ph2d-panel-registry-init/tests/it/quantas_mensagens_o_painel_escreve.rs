//! ⭐⭐⭐ **QUANTAS FRASES O INSPECTOR ESCREVE DE UMA VEZ — e quantas dizem a MESMA coisa.**
//!
//! ⛔⛔ **Report do dono, 2026-09-21:** *«vários componentes cheios de mensagens»*. Nenhum
//! instrumento desta casa sabia responder-lhe — o censo de texto pergunta *«vem da tabela?»*, o
//! das elisões pergunta *«coube?»*, o de ids pergunta *«é alcançável?»*, e **nenhum** perguntava
//! ***quantas frases o painel escreve***.
//!
//! ⚠️ **A fixtura tem de conter o fenómeno.** A `o_inspector_armado::arma_tudo` declarava
//! `selected_count: 1` em **28** sítios, e a frase que este gate persegue é gateada em `> 1` ⇒
//! *com a fixtura pregada em `1` nenhuma régua desta casa jamais a via*. É por isso que a
//! [`super::o_inspector_armado::com_seleccao_de`] existe.

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_panel_inspector::censo_dos_avisos::Aviso;
use ph2d_ui_testkit::MockPanelHost;

use super::quantas_entradas_tem_cada_painel::{VIEWPORT, abre_tudo};

/// Pinta o Inspector inteiro com `n` objectos seleccionados e devolve os avisos escritos.
fn avisos_com_seleccao_de(n: usize) -> Vec<Aviso> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("inspector");
        super::o_inspector_armado::com_seleccao_de(n);
        super::o_inspector_armado::arma_tudo();
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let (_, avisos) = ph2d_panel_inspector::censo_dos_avisos::medindo(|| {
            host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        out = avisos;
        super::o_inspector_armado::desarma_tudo();
        super::o_inspector_armado::com_seleccao_de(1);
    });
    out
}

/// Agrupa por TEXTO e devolve `(texto, quantas vezes, secções)`, do mais repetido para o menos.
fn por_texto(avisos: &[Aviso]) -> Vec<(String, usize, Vec<String>)> {
    let mut mapa: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for a in avisos {
        mapa.entry(a.texto.clone())
            .or_default()
            .push(a.seccao().to_string());
    }
    let mut v: Vec<(String, usize, Vec<String>)> = mapa
        .into_iter()
        .map(|(t, s)| {
            let n = s.len();
            (t, n, s)
        })
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

#[test]
#[ignore = "sonda de diagnóstico — corre à mão com --ignored --nocapture"]
fn diag_quantas_mensagens_o_painel_escreve() {
    for n in [1usize, 2] {
        let avisos = avisos_com_seleccao_de(n);
        eprintln!(
            "\n=== {n} objecto(s) seleccionado(s): {} avisos ===",
            avisos.len()
        );
        for (texto, quantas, seccoes) in por_texto(&avisos) {
            let marca = if quantas > 1 { "⛔" } else { "  " };
            let corte: String = texto.chars().take(64).collect();
            eprintln!("{marca} {quantas:3}×  {corte:?}");
            if quantas > 1 {
                eprintln!("        {}", seccoes.join(" · "));
            }
        }
    }
}

/// ⭐⭐⭐ **NENHUM FACTO DO PAINEL É ESCRITO DUAS VEZES NO MESMO QUADRO.**
///
/// ⛔⛔⛔ **A frase que este gate nasceu a perseguir é *«estás a editar só a selecção
/// primária»***, e ela não é um facto sobre o componente: é um facto sobre a **SELECÇÃO**, que é
/// a mesma para o painel inteiro naquele instante. **Vinte e uma** secções liam o mesmo número e
/// escreviam-no cada uma por si, em **quatro** redacções e por **seis** pintores diferentes — *o
/// artista com quatro destes componentes no objecto lia a mesma frase quatro vezes*, que é o
/// report do dono à letra.
///
/// ⚠️ **É a QUARTA vez que esta casa paga a mesma forma** — uma grandeza partilhada copiada por
/// ficheiro, cada cópia a afirmar que concorda com as irmãs, sem portão: o `CHECKBOX_BOX_PX = 18`
/// (21/09), o `SwatchSize::Md` como largura de linha (22/09), as dezoito declarações de «a
/// altura» (22/09) — e agora uma FRASE.
///
/// ⚠️ **Ele mede o PRODUTO e não o fonte:** um gate textual sobre `selected_count > 1` ficaria
/// verde no dia em que alguém escrevesse a mesma frase por outro caminho — que foi exactamente o
/// que as seis famílias de pintor fizeram durante meses.
///
/// (Mutação: repor o aviso em duas secções ⇒ RED, com as duas nomeadas.)
#[test]
fn nenhum_facto_do_painel_e_escrito_duas_vezes() {
    let avisos = avisos_com_seleccao_de(2);
    assert!(
        avisos.len() >= PISO_DE_AVISOS,
        "o Inspector armado escreveu {} avisos — abaixo do piso de {PISO_DE_AVISOS}. A fixtura \
         deixou de conter o fenómeno e este gate passou a medir o nada.",
        avisos.len()
    );
    let repetidos: Vec<(String, usize, Vec<String>)> = por_texto(&avisos)
        .into_iter()
        .filter(|(_, n, _)| *n > 1)
        .collect();
    // ⭐⭐ **O CENSO DE OBSOLESCÊNCIA da tolerância** — uma catraca sem ele vira LICENÇA.
    for (frase, _) in TOLERADAS {
        assert!(
            repetidos.iter().any(|(t, _, _)| t.contains(frase)),
            "a tolerância {frase:?} já não descreve nada — ela deixou de se repetir. ⛔ Apague a \
             linha: uma lista de dívida que ninguém encolhe é a catraca a virar licença."
        );
    }
    let maus: Vec<String> = repetidos
        .iter()
        .filter(|(t, _, _)| !TOLERADAS.iter().any(|(f, _)| t.contains(f)))
        .map(|(t, n, s)| {
            let corte: String = t.chars().take(72).collect();
            format!("{n}× {corte:?}  ({})", s.join(" · "))
        })
        .collect();
    assert!(
        maus.is_empty(),
        "o mesmo texto foi escrito mais de uma vez no MESMO quadro do Inspector — se ele é um \
         facto do PAINEL (a selecção), ele pinta-se UMA vez no cartão do topo:\n  {}",
        maus.join("\n  ")
    );
}

/// ⭐ **As frases que DUAS secções dizem por direito** — e porquê.
///
/// ⚠️ **A fronteira é o SUJEITO.** *«Estás a editar uma de três»* é um facto do PAINEL: ele é o
/// mesmo para todas as secções no mesmo instante, logo repeti-lo é ruído. *«O relógio é o Timer
/// 1»* é um facto da SECÇÃO: duas secções podem apontar ao mesmo relógio, e cada uma diz-no
/// debaixo do próprio cabeçalho, onde ele não é ambíguo.
///
/// ⛔ **Só ENCOLHE.** Uma entrada nova pede a medição ao lado, como estas.
const TOLERADAS: &[(&str, &str)] = &[(
    "same as in Timers",
    "o relógio de um componente — duas secções podem apontar ao mesmo Timer, e cada uma diz-no \
     debaixo do próprio cabeçalho",
)];

/// ⚠️ **Piso de população.** Sem ele, uma fixtura que deixasse de armar as secções devolveria
/// zero avisos e `maus.is_empty()` seria trivialmente verdadeiro — *zero lê-se como aprovação*,
/// a lei que esta casa já pagou em cada censo que varre por prefixo.
const PISO_DE_AVISOS: usize = 15;

/// ⭐⭐⭐ **TODO AVISO DO PAINEL PASSA PELA PORTA.**
///
/// ⛔⛔ Havia **onze** declarações de *«pintar uma linha de aviso»* neste painel: a porta mais
/// sete funções livres e três fechos, **as dez cópias idênticas byte a byte** e todas com os dois
/// mesmos defeitos — chamavam o `paint_text` (que **CORTA**) em vez do `paint_text_block` (que
/// **QUEBRA**), e devolviam **uma linha de altura** qualquer que fosse a frase, logo a segunda
/// linha era escrita por cima do que vinha abaixo. *Os dois defeitos exactos que o gate da porta
/// existe para impedir, num sítio onde ele nunca olhou.*
///
/// ⚠️ **Esta metade é a do PRODUTO; a textual vive na crate do painel**
/// (`nenhuma_seccao_pinta_o_proprio_aviso`). As duas são obrigatórias: a textual vê uma cópia que
/// ainda não tem chamador, e esta vê uma cópia escrita por um caminho que a agulha não conhece.
#[test]
fn todo_aviso_passa_pela_porta() {
    let avisos = avisos_com_seleccao_de(2);
    let seccoes: std::collections::BTreeSet<&str> = avisos.iter().map(Aviso::seccao).collect();
    assert!(
        seccoes.len() >= PISO_DE_SECCOES,
        "só {} secção(ões) registaram um aviso ({:?}) — o piso é {PISO_DE_SECCOES}. Ou a fixtura \
         deixou de as armar, ou alguém escreveu um pintor de aviso PRÓPRIO, que não passa pela \
         porta e é invisível a este censo.",
        seccoes.len(),
        seccoes
    );
}

/// ⚠️ **O piso é de SECÇÕES e não de avisos**, de propósito: é uma cópia do pintor que este gate
/// persegue, e uma cópia mora sempre numa secção inteira.
const PISO_DE_SECCOES: usize = 10;

/// ⭐⭐⭐ **A FRASE DA SELECÇÃO É PINTADA — UMA VEZ, E PELO CARTÃO.**
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a primeira, apagar o cartão deixaria o gate
/// irmão VERDE (zero cópias é zero repetições) — *zero lê-se como aprovação*. Sem a segunda, ela
/// podia voltar a ser pintada por uma secção.
#[test]
fn a_frase_da_seleccao_e_pintada_uma_vez_pelo_cartao() {
    let sozinho = avisos_com_seleccao_de(1);
    assert!(
        !sozinho.iter().any(|a| a.texto.contains("selected \u{b7}")),
        "com UM objecto seleccionado o painel ainda avisa sobre a selecção"
    );
    let dois = avisos_com_seleccao_de(2);
    let da_seleccao: Vec<&Aviso> = dois
        .iter()
        .filter(|a| a.texto.contains("selected \u{b7}"))
        .collect();
    assert_eq!(
        da_seleccao.len(),
        1,
        "a frase da selecção foi escrita {} vez(es): {:?}",
        da_seleccao.len(),
        da_seleccao
    );
    assert!(
        da_seleccao[0].ficheiro.ends_with("paint_cards.rs"),
        "a frase da selecção foi escrita por {} — ela é um facto do PAINEL e mora no cartão do topo",
        da_seleccao[0].ficheiro
    );
    assert!(
        da_seleccao[0].texto.starts_with('2'),
        "a frase não diz de QUANTOS: {:?} — *«estás a editar só uma»* sem o número obriga o \
         artista a contar a selecção",
        da_seleccao[0].texto
    );
}
