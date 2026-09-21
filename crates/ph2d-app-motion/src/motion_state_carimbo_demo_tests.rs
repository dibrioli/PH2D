//! Gates da cena `=126` — **o campo de estrelas**.
//!
//! ⚠️⚠️ **O que estes gates NÃO podem medir, e porquê:** a cena passa por um `source.shape`, e a
//! geometria dele é assada pela SHELL — num arnês headless o duplicador coze `n = 0` (medido sobre
//! a `=121` e registado nos gates da `=124`). ⇒ *a corrente da forma é do smoke do dono; aqui
//! prova-se a ESTRUTURA do grafo e a NUVEM que alimenta o carimbo.*
//!
//! ⛔ **E o RELÓGIO não mora aqui**: a distância entre as duas rotas é medida pelas sondas do
//! [`crate::motion_carimbo_relogio_probe`], com o `loadavg` ao lado. Um gate que a medisse nesta
//! suíte seria mais um membro da família de flakes de carga do `CLAUDE.md` §5.0.

use super::*;
use ph2d_node_registry::NodeRegistry;

fn cena() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Os nós de um tipo, por id.
fn dos(doc: &MotionDoc, tipo: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == tipo)
        .map(|n| n.id)
        .collect()
}

/// ⭐⭐⭐ **A CADEIA É A DO REPORT, E NADA MAIS.**
///
/// ⚠️ **A metade que interessa é o «e nada mais»:** um oscilador, um campo ou uma simulação a mais
/// entram na conta do quadro, e a cena passaria a medir uma coisa e a dizer que mede outra. *Uma
/// cena de performance com um passageiro é uma medição de outro programa.*
#[test]
fn a_cena_e_a_cadeia_do_report_e_nada_mais() {
    let (doc, _reg, sinks) = cena();
    assert_eq!(sinks.len(), 1, "uma saida so'");
    assert_eq!(
        doc.graph.nodes().len(),
        4,
        "quatro nos: grid, shape, dup, output"
    );
    let grade = *dos(&doc, "motion.grid").first().expect("ha' uma grelha");
    let forma = *dos(&doc, "source.shape").first().expect("ha' uma forma");
    let dup = *dos(&doc, "motion.duplicator")
        .first()
        .expect("ha' um duplicador");
    let entradas: Vec<(u16, NodeId)> = doc
        .graph
        .edges()
        .iter()
        .filter(|e| e.to.0 == dup)
        .map(|e| (e.to.1, e.from.0))
        .collect();
    assert!(
        entradas.contains(&(0, forma)),
        "a FORMA entra na porta 0 do carimbo, e as entradas sao {entradas:?}"
    );
    assert!(
        entradas.contains(&(1, grade)),
        "as POSICOES entram na porta 1, e as entradas sao {entradas:?}"
    );
    assert!(
        doc.graph
            .edges()
            .iter()
            .any(|e| e.from.0 == dup && e.to.0 == sinks[0]),
        "o carimbo tem de chegar a' saida"
    );
}

/// ⭐⭐ **A FORMA É UMA ESTRELA, e o número gravado no documento prova-o de VOLTA.**
///
/// ⛔⛔ Este é o gate que a auditoria de 2026-09-20 não tinha: ali o índice vinha de um RÓTULO que
/// virara chave de i18n, a procura devolvia `None`, o param ficava no default e *toda a medição
/// mediu um CÍRCULO*. A régua é a volta — o número que está no documento, lido como `kind`, tem de
/// ser a `Star`.
#[test]
fn a_forma_gravada_no_documento_e_uma_estrela() {
    let (doc, ..) = cena();
    let forma = *dos(&doc, "source.shape").first().expect("ha' uma forma");
    let kind = doc
        .graph
        .node_params()
        .get(&forma)
        .and_then(|m| m.get(ph2d_node_motion_shape::param::KIND))
        .copied()
        .expect("a cena escreve o `kind`");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "um indice de enum que a propria cena escreveu"
    )]
    let i = kind as usize;
    assert_eq!(
        ph2d_node_motion_shape::ALL_KINDS.get(i),
        Some(&ph2d_node_motion_shape::ShapeKind::Star),
        "o `kind` gravado e' {kind}, que nao e' a `Star`"
    );
    // ⚠️ E o TAMANHO tem de ser o derivado — sem ele a cena desenha a estrela de fábrica (`size`
    // `1,0`), que a este vão é dezoito vezes maior do que o vão e entrega uma mancha.
    let size = doc
        .graph
        .node_params()
        .get(&forma)
        .and_then(|m| m.get(ph2d_node_motion_shape::param::SIZE))
        .copied()
        .expect("a cena escreve o `size`");
    assert!(
        (size - TAMANHO).abs() < 1e-6,
        "o `size` gravado e' {size} e o derivado e' {TAMANHO}"
    );
}

/// ⭐⭐⭐ **A POPULAÇÃO QUE A JANELA PRENDE É A QUE ESTÁ MONTADA — e o VÃO chega ao nó.**
///
/// ⛔⛔ **A segunda metade é a que apanha o defeito MUDO:** um `set_param` com o nome errado é um
/// no-op silencioso que monta, valida, cozinha a contagem CERTA e entrega outra cena — foi assim
/// que quatro cenas desta casa escreveram `spacing` num nó que declara `gap_x`/`gap_y` e saíram
/// com grades `2×` mais largas. ⇒ a régua é a LARGURA da nuvem cozida, não a contagem.
#[test]
fn a_nuvem_cozida_tem_a_populacao_e_o_vao_derivados() {
    let (doc, reg, _) = cena();
    let grade = *dos(&doc, "motion.grid").first().expect("ha' uma grelha");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&doc.graph, &reg, 0.0).expect("avanca");
    let s = cook.cook(&doc.graph, &reg, grade, 0.0).expect("coze");
    let c = s[0].as_stream();
    assert_eq!(
        c.count() as u64,
        ESTRELAS,
        "a grelha cozinha {} posicoes e a janela da populacao prende {ESTRELAS}",
        c.count()
    );
    let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = c.get("P") else {
        panic!("a grelha tem de entregar a coluna `P`")
    };
    let (xl, xh) = p
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q[0]), h.max(q[0])));
    let esperada = (LADO - 1.0) * VAO;
    assert!(
        ((xh - xl) - esperada).abs() < 1e-3,
        "a nuvem mede {} de largura e o vao derivado pede {esperada} -- se o param do vao \
         mudou de nome, isto e' o unico sitio que o diz",
        xh - xl
    );
}

/// ⭐⭐ **A CENA É ALCANÇÁVEL PELO NÚMERO QUE O ROTEIRO PROMETE** (`PH2D_GPU_COOK_DEMO=126`).
///
/// ⚠️ **Sem este gate a cena podia existir, montar e ser inalcançável:** o braço do roteador é uma
/// linha que ninguém compila contra a cena, e um smoke que nomeia um número errado é um passo
/// impossível com o dono a segui-lo.
#[test]
fn o_roteador_monta_esta_cena_no_126() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = super::super::demo_router::build_level(Some("126"), &mut doc, &reg);
    assert_eq!(sinks.len(), 1, "o `=126` monta a cena e ela tem uma saida");
    assert_eq!(
        dos(&doc, "motion.duplicator").len(),
        1,
        "e o que ele monta e' o campo de estrelas"
    );
    // ⚠️ **O TECTO da varredura tem de a alcançar** — as três travessias do roteador param nele, e
    // uma cena acima do tecto existe e **nunca é diagnosticada**. `const` ⇒ isto é de compilação.
    const _: () = assert!(super::super::demo_router::MAX_DEMO_LEVEL >= 126);
}

/// **O roteiro nomeia o que existe: os cartões da tela, a barra de baixo e a porta de bissecção.**
///
/// ⚠️ **A metade dos números da barra é DERIVADA do i18n**, não escrita à mão: o roteiro manda o
/// dono ler `fps` e `raw`, e essas palavras são as que o HUD pinta. No dia em que o rótulo mudar,
/// este gate reprova em vez de o roteiro passar a apontar para um número que não está lá.
#[test]
fn o_roteiro_nomeia_o_que_o_dono_vai_ver() {
    let texto = include_str!("motion_state_carimbo_demo.rs");
    for nome in ["Grid", "Duplicator", "PH2D_CARIMBO_PREPARADO"] {
        assert!(
            texto.contains(nome),
            "o roteiro tem de nomear {nome:?}, que e' o que o artista procura"
        );
    }
    let hud = ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, "chrome.hud.fps");
    for palavra in ["fps", "raw"] {
        assert!(
            hud.contains(palavra),
            "a barra de baixo deixou de dizer {palavra:?} (ela diz {hud:?}) -- o roteiro manda \
             o dono ler esse numero"
        );
        assert!(
            texto.contains(palavra),
            "o roteiro tem de nomear {palavra:?}, que e' o numero que a cena existe para comparar"
        );
    }
}

/// ⭐⭐⭐ **OS DOIS INSTRUMENTOS DE BISSECÇÃO NÃO MUDAM A CENA QUANDO NINGUÉM LHES TOCA.**
///
/// Eles existem por causa do report do dono de 2026-09-20 (o `Corner Radius`) e do erro de
/// calibração que ele expôs: um gesto de painel **não é alcançável de um teste**, e sem uma porta
/// a única forma de medir o quadro que ele viu era clicar — coisa que esta casa não faz.
///
/// ⚠️⚠️ **A metade que interessa é a NEGATIVA:** *uma porta de bissecção que mude a cena quando
/// ninguém a arma deixa de bissectar coisa nenhuma* — ela passaria a ser um segundo produto, e as
/// duas colunas de um A/B mediriam cenas diferentes.
///
/// ⚠️ E ela entra pela LEI PURA e não pela leitura do ambiente: *um gate que lê o ambiente mede a
/// máquina em que corre*.
#[test]
fn os_instrumentos_de_bisseccao_nao_mudam_a_cena_de_omissao() {
    // Ausente ⇒ a cena de sempre, nas duas portas.
    assert!(
        (super::corner_por(None) - 0.0).abs() < f32::EPSILON,
        "sem a variavel a estrela tem de ser a PONTIAGUDA (corner 0)"
    );
    assert_eq!(
        super::lado_por(None),
        super::LADO_N,
        "sem a variavel a grelha tem de ser a da cena"
    );
    // ⚠️⚠️ **As DUAS portas tratam «fora da faixa» de maneira OPOSTA, e é deliberado — este gate
    // reprovou a escrevê-lo ao contrário.** O `corner` SATURA porque a faixa `0..1` é o domínio da
    // própria lei (o `build_shape_path` já faz `clamp(0,1)`), logo saturar aqui devolve exactamente
    // o que o produto faria. O `lado` FILTRA porque ali a faixa é um TECTO DE RECURSO: um `5000`
    // escrito por engano montaria `25 M` estrelas e o app não voltava — *saturar em `2000` daria
    // uma cena que ninguém pediu, e é pior do que ignorar o número*.
    for lixo in ["", "  ", "abc", "NaN"] {
        assert!(
            (super::corner_por(Some(lixo)) - 0.0).abs() < f32::EPSILON,
            "{lixo:?} nao e' um numero: tinha de cair no corner de omissao"
        );
    }
    for (fora, esperado) in [("-1", 0.0f32), ("1e9", 1.0)] {
        assert!(
            (super::corner_por(Some(fora)) - esperado).abs() < f32::EPSILON,
            "{fora:?} e' um numero FORA da faixa: ele satura em {esperado}, como a lei da forma"
        );
    }
    for lixo in ["", "zero", "0", "1", "2001", "-5"] {
        assert_eq!(
            super::lado_por(Some(lixo)),
            super::LADO_N,
            "{lixo:?} nao e' um lado utilizavel: tinha de cair no lado de omissao"
        );
    }
    // E armados, eles ARMAM — senão a cerca acima seria satisfeita por uma porta morta.
    assert!(
        (super::corner_por(Some("0.5")) - 0.5).abs() < f32::EPSILON,
        "com a variavel armada o corner tem de chegar"
    );
    assert_eq!(
        super::lado_por(Some("400")),
        400,
        "com a variavel armada o lado tem de chegar"
    );
}
