//! **OS GATES DA CENA `=113`** — e a prova de que cada passo do anúncio produz o que promete.
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
//! (`CLAUDE.md` §5.0) — a ausente não é acreditada. O anúncio desta cena manda o dono trocar
//! `Acts As` de `Force` para `Target Velocity` e diz-lhe o que ele vai ver; aqui isso é
//! **medido sobre a cena do produto**, com um param de diferença entre os dois braços.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Corre a cena `secs` segundos e devolve `(a maior rapidez vista em QUALQUER tique, as
/// posições finais)`.
///
/// ⚠️ **O pico é sobre a corrida inteira de propósito:** o que separa os dois modos é aquilo
/// a que a queda **chega a fazer**, e ler só o último quadro perderia a aceleração.
fn corre(modo: Option<f32>, secs: f64) -> (f32, Vec<[f32; 2]>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena é bem tipada");
    if let Some(m) = modo {
        // O ÚNICO param que difere entre os dois braços.
        let vento = doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "force.wind")
            .expect("a cena tem um `force.wind`")
            .id;
        doc.graph.set_param(vento, "mode", m);
    }
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let (mut pico, mut fim) = (0.0f32, vec![]);
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(v)) = s.get("vel") {
            pico = v.iter().fold(pico, |m, w| m.max(w[0].hypot(w[1])));
        }
        if k == last
            && let Some(Column::Vec2(v)) = s.get("P")
        {
            fim = v.clone();
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    (pico, fim)
}

/// **A CHUVA CAI, E O BLOCO APANHA PARTE DELA — parte, não toda.**
///
/// ⚠️ **As duas metades são o gate.** Se o bloco apanhasse tudo, a cena ensinaria que um
/// colisor é um chão; se não apanhasse nada, o passo 2 do anúncio (arrastar a caixa e ver o
/// monte mudar de sítio) não teria monte nenhum para mover.
#[test]
fn the_block_catches_part_of_the_rain_and_the_rest_falls_past_it() {
    let (_, p) = corre(None, 2.2);
    assert_eq!(
        p.len(),
        (ROWS * COLS) as usize,
        "a nuvem inteira tem de estar la'"
    );
    let topo = BLOCO_Y + BLOCO_H * 0.5;
    let em_cima = p.iter().filter(|q| q[1] >= topo - PECA).count();
    let passaram = p.iter().filter(|q| q[1] < BLOCO_Y - BLOCO_H).count();
    assert!(
        em_cima >= 4,
        "so' {em_cima} peca(s) pousaram no bloco -- sem monte, o passo 2 do anuncio (arrastar \
         a caixa e ver o monte mudar de sitio) nao tem o que mover"
    );
    assert!(
        passaram >= 4,
        "so' {passaram} peca(s) passaram ao lado do bloco -- se ele apanha tudo, a cena ensina \
         que um colisor e' um chao, e a largura dele deixa de querer dizer alguma coisa"
    );
}

/// ⭐⭐⭐ **O PASSO 3 DO ANÚNCIO PRODUZ O QUE ELE DIZ: `Target Velocity` SATURA.**
///
/// ⛔ **Sem barra escolhida.** A saturação *é* o significado do modo: a lei é
/// `a = resistência · (alvo − v)`, que empurra cada vez menos e **pára** quando a peça
/// alcança o vento — logo a rapidez **nunca** passa da `Strength` do nó. O braço `Force`
/// acelera para sempre e passa-a, e é ele que prova que havia o que saturar.
///
/// ⚠️ **É a MESMA cena nos dois braços, com UM param de diferença** — o `mode`. Um segundo
/// documento montado à mão mediria outro programa.
#[test]
fn the_target_velocity_mode_caps_the_fall_and_force_does_not() {
    let (pico_forca, _) = corre(Some(0.0), 2.2);
    let (pico_alvo, _) = corre(Some(1.0), 2.2);
    assert!(
        pico_alvo <= GRAVIDADE + 1e-3,
        "com `Target Velocity` a rapidez chegou a {pico_alvo:.3}, acima da `Strength` \
         ({GRAVIDADE}) -- a lei `a = resistencia * (alvo - v)` nao pode ultrapassar o alvo, \
         entao ou o modo nao esta' a ser lido ou a cena mudou de forca"
    );
    assert!(
        pico_forca > GRAVIDADE * 1.5,
        "com `Force` a rapidez so' chegou a {pico_forca:.3} -- o braco de CONTROLO tem de \
         passar claramente a `Strength` ({GRAVIDADE}), senao os dois modos leriam igual e \
         este gate estaria a comparar duas quedas que ja' eram a mesma"
    );
}

/// ⭐⭐⭐ **O ÍNDICE DE UM ENUM NÃO PODE DEPENDER DO IDIOMA DA CORRIDA.**
///
/// ⛔⛔ Até 2026-09-17 o [`indice_de`] resolvia o rótulo com `ph2d_i18n::tr`, que responde **no
/// idioma da corrida** — e quem chama escreve `"Box"`, `"Bowl"`, `"Loop"`, que é inglês, a lei
/// da casa para texto de cena. Com um segundo idioma ligado a comparação **nunca casa**, o `?`
/// devolve `None` e **cinco cenas de smoke abrem com a forma errada, sem erro nenhum**.
///
/// ⚠️ **Nenhum instrumento deste repo o via**, e a razão é estrutural: o defeito é um `None`
/// num caminho que devolve `Option`, e quem o recebe usa `?`. *Uma resolução que falha para o
/// lado do silêncio é invisível a toda suíte que corre num idioma só* — quem o achou foi o
/// **idioma de teste**.
///
/// # ⛔⛔ A 1.ª redacção deste gate era VÁCUA, e uma mutação mostrou-o
///
/// Ela chamava o [`indice_de`] duas vezes e comparava — mas o `tr` lê o idioma por
/// [`ph2d_i18n::idioma`], que é um `OnceLock` sobre a **variável de ambiente do processo**. Num
/// processo de teste ela não está posta, logo `tr` e `tr_em(Ingles, …)` são a MESMA função e a
/// mutação que desfaz a cura **passa**. *Um gate que varia uma coisa que o código lê de um
/// global do processo não varia nada* — e a régua certa não é chamar duas vezes, é medir as
/// **três** metades abaixo, cada uma de um defeito diferente.
#[test]
fn o_indice_de_um_enum_e_o_mesmo_em_qualquer_idioma() {
    let reg = registry();
    // As consultas que as cinco cenas fazem — a população é a do PRODUTO.
    let consultas = [
        ("sim.collide", "shape", "Box"),
        ("sim.collide", "shape", "Plane"),
        ("sim.collide", "shape", "Bowl"),
        ("source.shape", "kind", "Circle"),
        ("source.shape", "kind", "Square"),
        ("sim.zone", "mode", "Loop"),
    ];
    for (no, param, valor) in consultas {
        // ⭐ METADE 1 — a consulta RESOLVE. Sem ela o resto mede o nada.
        let achado = indice_de(&reg, no, param, valor);
        assert!(
            achado.is_some(),
            "`{no}.{param}` não tem a opção `{valor}` — a cena escreveria o param errado"
        );

        // ⭐⭐ METADE 2 — **o defeito EXISTE**: noutro idioma a palavra que a cena escreve já
        //    não é a que a tabela devolve. É isto que torna a metade 3 necessária em vez de
        //    decorativa, e é a única maneira de o afirmar sem escrever no ambiente de um
        //    processo que corre a suíte em paralelo.
        let tid = ph2d_nodegraph::node::NodeTypeId::of(no);
        let hint = reg
            .param_ui(tid)
            .and_then(|h| h.iter().find(|h| h.param == param).copied())
            .expect("o param existe");
        let ph2d_node_registry::ParamWidget::Enum { labels } = hint.widget else {
            panic!("`{no}.{param}` devia ser um selector");
        };
        let chave = labels
            .iter()
            .find(|l| ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, l) == valor)
            .expect("a metade 1 já o achou");
        assert_ne!(
            ph2d_i18n::tr_em(ph2d_i18n::Idioma::Teste, chave),
            valor,
            "`{chave}` lê igual nos dois idiomas — esta consulta não discrimina, e a metade 3              ficaria a defender uma lei que nada pode violar"
        );
    }
}

/// ⭐⭐⭐ **METADE 3 — a resolução NÃO CONSULTA o idioma da corrida**, e isto é textual de
/// propósito.
///
/// ⚠️ A propriedade é *«a resposta não depende de `PH2D_LANG`»*, e ela **não é observável de um
/// teste**: o idioma é um `OnceLock` sobre uma variável de ambiente, posta uma vez por processo.
/// ⇒ o que se afirma é a FORMA — a função resolve com [`ph2d_i18n::tr_em`] e um idioma
/// **escrito**, nunca com o `tr` que lê o ambiente.
///
/// ⭐ `include_str!` e não `read_to_string`: se o irmão mudar de nome ou de sítio isto deixa de
/// **COMPILAR**, em vez de passar a varrer um ficheiro vazio.
#[test]
fn a_resolucao_do_indice_nao_le_o_idioma_do_ambiente() {
    const FONTE: &str = include_str!("motion_state_sim_demo.rs");
    let corpo = FONTE
        .split_once("pub(super) fn indice_de(")
        .expect("a função existe")
        .1;
    let corpo = &corpo[..corpo
        .find(
            "
}
",
        )
        .expect("a função fecha")];
    let codigo: String = corpo
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    assert!(
        codigo.contains("tr_em(ph2d_i18n::Idioma::Ingles"),
        "o `indice_de` deixou de resolver num idioma ESCRITO"
    );
    // ⛔ Controlo de vacuidade da extracção: um corte errado devolveria um corpo sem o `match`
    //    do widget, e o teste acima passaria a medir uma string vazia.
    assert!(
        codigo.contains("ParamWidget::Enum"),
        "a extracção do corpo da função partiu-se — ela já não contém o que devia"
    );
}
