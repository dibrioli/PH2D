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

/// ⭐⭐⭐ **A CADEIA É A DO REPORT, MAIS A SIMULAÇÃO QUE O DONO MANDA — E NADA MAIS.**
///
/// ⚠️ **A premissa deste gate MORREU em 2026-09-23 e está aqui à vista:** ele dizia *«um
/// oscilador, um campo ou uma simulação a mais entram na conta do quadro, e a cena passaria a medir
/// uma coisa e a dizer que mede outra»*. O dono ordenou que toda cena de smoke tenha SIMULAÇÃO com
/// campos (doc 103 §1), e o argumento era mais fraco do que dizia: o roteiro compara a DIFERENÇA de
/// `raw` entre duas corridas (com e sem a cura do carimbo), e um custo igual nas duas cancela-se.
///
/// O «e nada mais» continua a valer para o resto: um passageiro que só existisse numa das duas
/// corridas mediria outro programa. A simulação entra nas DUAS por construção (é o mesmo grafo).
#[test]
fn a_cena_e_a_cadeia_do_report_mais_a_simulacao_e_nada_mais() {
    let (doc, _reg, sinks) = cena();
    assert_eq!(sinks.len(), 1, "uma saida so'");
    assert_eq!(
        doc.graph.nodes().len(),
        9,
        "nove nos: grid, shape, dup, output + integrate e os quatro campos"
    );
    let grade = *dos(&doc, "motion.grid").first().expect("ha' uma grelha");
    let forma = *dos(&doc, "source.shape").first().expect("ha' uma forma");
    let ig = *dos(&doc, "motion.integrate")
        .first()
        .expect("ha' uma simulacao");
    let dup = *dos(&doc, "motion.duplicator")
        .first()
        .expect("ha' um duplicador");
    let entradas = |alvo: NodeId| -> Vec<(u16, NodeId)> {
        doc.graph
            .edges()
            .iter()
            .filter(|e| e.to.0 == alvo)
            .map(|e| (e.to.1, e.from.0))
            .collect()
    };
    assert!(
        entradas(dup).contains(&(0, forma)),
        "a FORMA entra na porta 0 do carimbo, e as entradas sao {:?}",
        entradas(dup)
    );
    assert!(
        entradas(dup).contains(&(1, ig)),
        "as posicoes SIMULADAS entram na porta 1, e as entradas sao {:?}",
        entradas(dup)
    );
    assert!(
        entradas(ig).contains(&(0, grade)),
        "a grelha e' o repouso da simulacao"
    );
    let campos = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name.starts_with("force."))
        .count();
    assert_eq!(campos, 4, "a galaxia tem quatro campos de forca");
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

/// ⭐⭐⭐ **O ROTEIRO NÃO CITA UM NÚMERO DE RELÓGIO QUE ELE NÃO DERIVE.**
///
/// ⛔⛔⛔ **Ele nasceu de um defeito REAL, e ele passou despercebido durante um tecto inteiro.**
/// O roteiro dizia *«o `raw` perto de `96`»* e *«cai para `70` e poucos»* — dois absolutos lidos
/// no app a `90 000` estrelas. Quando o tecto do dono mudou a população (para `16 384` em 21/09 e
/// `32 761` em 22/09) os dois ficaram errados e **nada** podia dizê-lo: o gate irmão afirma que o
/// roteiro nomeia as PALAVRAS `fps`/`raw`, nunca que os números ao lado delas descrevem a cena.
///
/// ⚠️⚠️ **E a cura não podia ser «re-medir»**, porque o `raw` é do QUADRO INTEIRO (`1000/cpu`, com
/// o custo fixo do chrome dentro) e as constantes desta cena são por CÓPIA — *um absoluto derivado
/// de um declive é falso noutra população*. A grandeza que a cena PODE afirmar é a DIFERENÇA entre
/// as duas rotas, que cancela o custo fixo; ver `DIFERENCA_NS`.
///
/// ⇒ este gate afirma a **DERIVAÇÃO**: o passo (4) tem de imprimir o valor calculado e não um
/// literal. ⚠️ Aqui a proveniência **é** a verdade, ao contrário do caso da cena `=107`: um número
/// derivado da população actual e de duas medições por-cópia está certo por construção, enquanto
/// uma `const` de milissegundos medida noutra cena continua a ser um literal com nome.
#[test]
fn o_roteiro_deriva_o_numero_de_relogio_em_vez_de_o_escrever() {
    let texto = include_str!("motion_state_carimbo_demo.rs");
    let anuncio = texto
        .split_once("pub(super) fn announce()")
        .expect("a cena tem um roteiro")
        .1;
    assert!(
        anuncio.contains("{DIFERENCA_MS:"),
        "o passo (4) tem de IMPRIMIR a diferenca derivada; um numero de ms escrito a' mao ali \
         envelhece no dia em que o tecto de instancias se mexer, e foi isso que aconteceu"
    );
    // A metade NEGATIVA: os dois absolutos que envelheceram não podem voltar.
    for morto in ["perto de `96`", "cai para `70`"] {
        assert!(
            !anuncio.contains(morto),
            "o roteiro voltou a citar {morto:?}, que foi medido a 90 000 estrelas -- a cena \
             produz {} hoje",
            super::ESTRELAS
        );
    }
    // E a diferença que ele imprime tem de ser LEGÍVEL no readout. ⚠️ Ela é lida da PORTA
    // (`DIFERENCA_NS`) e não recalculada aqui: repetir `ESTRELAS × (ANTES − HOJE)` neste ficheiro
    // seria uma segunda resposta à mesma pergunta, e as duas divergiriam no dia em que uma das
    // três constantes se mexesse — que é exactamente o defeito que este gate existe para impedir.
    let ms = super::DIFERENCA_MS;
    assert!(
        ms >= 1.0,
        "as duas rotas diferem {ms:.2} ms: o dono vai comparar dois `raw` que leem igual"
    );
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

/// ⭐⭐ **A GALÁXIA GIRA E FICA DO TAMANHO DO CAMPO** — a régua que afinou os campos, virada gate.
///
/// Duas metades, porque cada uma sozinha mente:
/// - **mexe** (`> 0,3 m` de afastamento médio da grelha de partida): sem ela uma cena com a
///   simulação desligada passava — a queixa do dono de 2026-09-23;
/// - **cabe** (nenhuma estrela a mais de `1,25 ×` o meio-lado do campo de partida): sem ela os
///   campos atiravam as estrelas para fora, e o passo do roteiro *«afaste até o campo INTEIRO caber
///   no ecrã»* deixava de ser possível.
///
/// 10 s a 60 Hz pelo `Cook` da CPU (o extremo assenta aos `~4 s`) (a cena vai à CPU de qualquer modo: a estrela é uma forma viva).
#[test]
fn a_galaxia_gira_e_fica_do_tamanho_do_campo() {
    let (doc, reg, _) = cena();
    let ig = *dos(&doc, "motion.integrate")
        .first()
        .expect("ha' um integrador");
    let meio_lado = (LADO - 1.0) * VAO / 2.0;
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let dt = 1.0 / 60.0;
    let (mut inicio, mut pior, mut andou, mut medidas) = (None, 0.0f32, 0.0f32, 0usize);
    for tick in 0..=600u32 {
        let t = f64::from(tick) * dt;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        let s = cook.cook(&doc.graph, &reg, ig, t).expect("coze");
        if tick % 60 != 0 {
            continue;
        }
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s[0].as_stream().get("P") else {
            panic!("o integrador entrega `P`")
        };
        let base: &Vec<[f32; 2]> = inicio.get_or_insert_with(|| p.clone());
        let media = p
            .iter()
            .zip(base)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .sum::<f32>()
            / p.len().max(1) as f32;
        andou = andou.max(media);
        pior = p
            .iter()
            .map(|q| q[0].abs().max(q[1].abs()))
            .fold(pior, f32::max);
        medidas += p.len();
        eprintln!(
            "t={t:>4.1}s afastamento medio {media:.3} m · extremo {pior:.2} m (meio-lado {meio_lado:.2})"
        );
    }
    assert!(
        medidas as u64 > ESTRELAS,
        "a régua não viu as estrelas ({medidas})"
    );
    assert!(
        andou > 0.3,
        "a galaxia quase nao se mexeu ({andou:.3} m) -- a simulacao nao corre"
    );
    assert!(
        pior < 1.25 * meio_lado,
        "uma estrela chegou a {pior:.2} m do centro, e o campo de partida tem {meio_lado:.2} m de \
         meio-lado -- o passo de AFASTAR ate' caber tudo deixa de ser possivel"
    );
}

/// **SONDA: quanto a galáxia custa por tique** (a simulação, sem o carimbo — o carimbo tem a sua
/// sonda no `motion_carimbo_relogio_probe`). `--release`, com o `loadavg` ao lado.
#[test]
#[ignore = "sonda de relogio, nao um gate"]
fn sonda_custo_da_galaxia() {
    let (doc, reg, _) = cena();
    let ig = *dos(&doc, "motion.integrate")
        .first()
        .expect("ha' um integrador");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let dt = 1.0 / 60.0;
    let mut tempos = Vec::new();
    for tick in 0..240u32 {
        let t = f64::from(tick) * dt;
        let t0 = std::time::Instant::now();
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        let _ = cook.cook(&doc.graph, &reg, ig, t).expect("coze");
        if tick >= 60 {
            tempos.push(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    tempos.sort_by(f64::total_cmp);
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "galaxia: {} estrelas · por tique p50 {:.2} ms · p90 {:.2} ms · loadavg {}",
        ESTRELAS,
        tempos[tempos.len() / 2],
        tempos[tempos.len() * 9 / 10],
        load.split_whitespace().next().unwrap_or("?")
    );
}
