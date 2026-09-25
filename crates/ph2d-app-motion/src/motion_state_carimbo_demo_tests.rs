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
        "nove nos: grid, shape, dup, output + integrate, o NUCLEO e os tres campos"
    );
    // ⭐ O NÚCLEO (o `motion.falloff` invertido) tem de vir ANTES das forças: é ele que faz a
    // rotação crescer com a distância e o disco não colapsar (ver o doc da `GALAXIA`). Um núcleo
    // depois delas não pesaria nenhuma, e a galáxia voltava a encolher até um anel.
    let nucleo = *dos(&doc, "motion.falloff").first().expect("ha' um nucleo");
    let vortex = *dos(&doc, "force.vortex")
        .first()
        .expect("ha' um redemoinho");
    assert!(
        entradas_de(&doc, vortex).contains(&(0, nucleo)),
        "o redemoinho le' o NUCLEO na porta 0"
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
    assert_eq!(campos, 3, "a galaxia tem tres campos de forca");
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
    // ⛔ Só o ROTEIRO, nunca o ficheiro inteiro (auditoria do fecho, 2026-09-24): `Grid` e
    // `Duplicator` aparecem no código que MONTA a cena, logo contra o ficheiro este gate ficava
    // verde com o roteiro a não os nomear — a régua que o irmão de baixo já usava.
    let texto = include_str!("motion_state_carimbo_demo.rs")
        .split_once("pub(super) fn announce()")
        .expect("a cena tem um roteiro")
        .1;
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

/// ⭐⭐ **A GALÁXIA GIRA E NÃO COLAPSA** — a régua que afinou os campos, virada gate.
///
/// Três metades, porque cada uma sozinha mente:
/// - **mexe** (`> 0,3 m` de afastamento médio da grelha de partida): sem ela uma cena com a
///   simulação desligada passava — a queixa do dono de 2026-09-23;
/// - **cabe** (nenhuma estrela a mais de `1,25 ×` o RAIO do canto do campo de partida): sem ela os
///   campos atiravam as estrelas para fora, e o passo *«afaste até o campo INTEIRO caber»* deixava
///   de ser possível. ⚠️ É o RAIO e não o meio-lado: um quadrado que GIRA põe o canto em cima do
///   eixo, e uma régua de Chebyshev acusava a rotação de fuga;
/// - ⛔⛔ **não COLAPSA** (a INCLINAÇÃO: as estrelas dentro da janela de arranque não mudam mais de
///   `1 %` entre os 2 s e os 10 s).
///   **Esta metade não existia e foi o report do dono** (*«estrelas com corner radius de 1 provoca
///   queda de raw»*, 2026-09-23): a 1.ª galáxia (vortex + ímã + arrasto em aceleração) tinha UMA
///   órbita de equilíbrio, `r* = (v/k)²/a ≈ 1,7 m`, e o campo inteiro encolhia até ela — `8 957`
///   estrelas na janela ao arranque, `19 712` aos 10 s, **as `32 761` aos 40 s** (sonda
///   `sonda_a_galaxia_a_longo_prazo`). O recorte por câmara deixava de ter o que cortar, e cada
///   estrela de cantos redondos custa `2,5×` os segmentos: o `raw` caía com o TEMPO, e caía mais
///   com os cantos. *A metade «cabe» ficava verde por cima, porque um colapso é o contrário de uma
///   fuga.* Aos 10 s a galáxia velha já lia `+120 %`.
///
/// ⚠️ **A barra é uma INCLINAÇÃO, e sai do requisito e não de um palpite:** com o núcleo o colapso
/// passa a ser LENTO (o ímã velho, `0,6`, desvia `+7 %` em 10 s e a 1.ª redacção deste gate, a
/// `10 %` absolutos, deixou-o passar — mutação SOBREVIVENTE). O requisito é *«a população não
/// muda mais de `±10 %` numa sessão de smoke de 2 minutos»*, e a deriva medida é recta depois do
/// arranque ⇒ entre os 2 s e os 10 s ela pode andar no máximo `10 % × 8/120 ≈ 0,7 %`. A barra é
/// `1 %`, no vale MEDIDO: a galáxia que shipa lê `+0,3 %`; ímã `0,30` → `+1,7 %`; `0,12` →
/// `−2,0 %`. ⛔ O primeiro segundo fica de fora: ali a janela perde `~1,5 %` em TODAS as variantes
/// (o arranque, antes de o redemoinho assentar a velocidade), e contá-lo mediria o arranque.
///
/// 10 s a 60 Hz pelo `Cook` da CPU (a cena vai à CPU de qualquer modo: a estrela é uma forma viva).
#[test]
fn a_galaxia_gira_e_nao_colapsa() {
    let (doc, reg, _) = cena();
    let ig = *dos(&doc, "motion.integrate")
        .first()
        .expect("ha' um integrador");
    let canto = (LADO - 1.0) * VAO / 2.0 * std::f32::consts::SQRT_2;
    let na_janela = |p: &[[f32; 2]]| {
        p.iter()
            .filter(|q| q[0].abs() < JANELA_DE_ARRANQUE[0] && q[1].abs() < JANELA_DE_ARRANQUE[1])
            .count()
    };
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let dt = 1.0 / 60.0;
    let (mut inicio, mut pior, mut andou, mut medidas) = (None, 0.0f32, 0.0f32, 0usize);
    let (mut janela2, mut janela10) = (0usize, 0usize);
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
        pior = p.iter().map(|q| q[0].hypot(q[1])).fold(pior, f32::max);
        let janela = na_janela(p);
        match tick {
            120 => janela2 = janela,
            600 => janela10 = janela,
            _ => {}
        }
        medidas += p.len();
        eprintln!(
            "t={t:>4.1}s afastamento medio {media:.3} m · raio max {pior:.2} m (canto {canto:.2}) \
             · na janela {janela}"
        );
    }
    assert!(
        medidas as u64 > ESTRELAS && janela2 > 1000,
        "a régua não viu as estrelas ({medidas}, {janela2} na janela)"
    );
    assert!(
        andou > 0.3,
        "a galaxia quase nao se mexeu ({andou:.3} m) -- a simulacao nao corre"
    );
    assert!(
        pior < 1.25 * canto,
        "uma estrela chegou a {pior:.2} m do centro, e o canto do campo de partida esta' a \
         {canto:.2} m -- o passo de AFASTAR ate' caber tudo deixa de ser possivel"
    );
    #[expect(clippy::cast_precision_loss, reason = "contagens de estrelas")]
    let deriva = janela10 as f32 / janela2 as f32 - 1.0;
    assert!(
        deriva.abs() < 0.01,
        "as estrelas na janela de arranque andaram {:+.1} % entre os 2 s e os 10 s -- a galaxia \
         colapsa (ou foge), e o `raw` que o dono le' passa a depender de QUANDO ele olha",
        deriva * 100.0
    );
}

/// A janela de câmara de arranque, em meios-lados de mundo — a do cabeçalho da cena (`21,8 × 6,8`).
const JANELA_DE_ARRANQUE: [f32; 2] = [10.9, 3.4];

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

/// **SONDA: a galáxia a LONGO prazo** — o gate acima olha 10 s e a foto da app aos `35 s` mostrou
/// o campo COLAPSADO num fio espiral. Imprime, de 5 em 5 s até 60 s: o raio p50/p90, o extremo, e
/// quantas estrelas caem dentro da janela de arranque (`21,8 × 6,8`, a do cabeçalho da cena) — que
/// é o que o recorte por câmara NÃO consegue tirar da placa.
#[test]
#[ignore = "sonda, nao um gate"]
fn sonda_a_galaxia_a_longo_prazo() {
    let (doc, reg, _) = cena();
    let ig = *dos(&doc, "motion.integrate")
        .first()
        .expect("ha' um integrador");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let dt = 1.0 / 60.0;
    for tick in 0..=3600u32 {
        let t = f64::from(tick) * dt;
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
        let s = cook.cook(&doc.graph, &reg, ig, t).expect("coze");
        if tick % 300 != 0 {
            continue;
        }
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s[0].as_stream().get("P") else {
            panic!("o integrador entrega `P`")
        };
        let mut r: Vec<f32> = p.iter().map(|q| q[0].hypot(q[1])).collect();
        r.sort_by(f32::total_cmp);
        let dentro = p
            .iter()
            .filter(|q| q[0].abs() < JANELA_DE_ARRANQUE[0] && q[1].abs() < JANELA_DE_ARRANQUE[1])
            .count();
        eprintln!(
            "t={t:>4.0}s  raio p50 {:.2}  p90 {:.2}  max {:.2}  dentro da janela {dentro}/{}",
            r[r.len() / 2],
            r[r.len() * 9 / 10],
            r[r.len() - 1],
            p.len()
        );
    }
}

/// As entradas de um nó, como `(porta, de quem)`.
fn entradas_de(doc: &MotionDoc, alvo: NodeId) -> Vec<(u16, NodeId)> {
    doc.graph
        .edges()
        .iter()
        .filter(|e| e.to.0 == alvo)
        .map(|e| (e.to.1, e.from.0))
        .collect()
}
