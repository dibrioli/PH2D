//! Os gates do censo do alcance (doc 110 §9).

/// **SONDA — o catálogo inteiro, e os params que ninguém alcança.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_unreachable_params -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_unreachable_params() {
    let c = super::censo();
    eprintln!("\n  nó                            | params que NENHUMA combinação revela");
    eprintln!("  ------------------------------|-------------------------------------");
    for (no, presos) in &c.presos {
        eprintln!("  {no:<29} | {}", presos.join(" · "));
    }
    eprintln!(
        "\n  {} acusado(s) · varridos {} nós / {} params · {} nós declaram gate.\n",
        c.presos.len(),
        c.nos,
        c.params,
        c.com_gate
    );
}

/// ⭐⭐⭐ **NENHUM PARAM DO CATÁLOGO ESTÁ ENTERRADO ATRÁS DOS PRÓPRIOS GATES.**
///
/// Um controlo que nenhuma combinação revela é a espécie de knob morto que **nenhuma** sonda deste
/// repo apanhava: a caça de 2026-08-30 seguiu o valor até ao efeito (*o painel escreve onde · quem
/// lê · o leitor decide?*) e esta pergunta vem **antes** dela — *o artista consegue sequer chegar
/// ao controlo?*
///
/// ⚠️⚠️ **O PISO DE POPULAÇÃO é metade deste gate.** Ele lê `0 acusados` quando está são, que é
/// exactamente o que lê quando está partido; sem os três números abaixo, um registry renomeado
/// deixaria este teste verde a varrer o vazio — a catraca que vira licença, um nível acima.
#[test]
fn no_param_in_the_catalogue_is_buried_behind_its_own_gates() {
    let c = super::censo();
    assert!(
        c.nos >= 100 && c.params >= 400,
        "o censo varreu {} nós / {} params -- ele mede o catálogo inteiro, e este numero diz que \
         a varredura se partiu",
        c.nos,
        c.params
    );
    assert!(
        c.com_gate >= 20,
        "só {} nós declaram gate de visibilidade -- sem sujeitos este censo não testa nada",
        c.com_gate
    );
    assert!(
        c.presos.is_empty(),
        "params que nenhuma combinação de gates revela: {:?}",
        c.presos
    );
}

/// ⭐⭐ **DENTRO DE UMA SECÇÃO, DOIS CONTROLOS NÃO PODEM TER O MESMO NOME.**
///
/// ⚠️ Ela é a fatia da lei do vocabulário que uma máquina sabe julgar — ver
/// [`super::rotulos_colididos`] para o porquê de ela não ser mais larga (a versão larga acusa ~40
/// grupos legítimos e vira licença).
///
/// ⚠️ **E o piso de população outra vez:** ela lê `0` colisões quando está sã, que é o que lê
/// quando o registry muda de nome debaixo dela.
#[test]
fn two_controls_in_one_section_never_share_a_name() {
    let m = crate::motion_state::MotionState::new();
    let rotulados: usize = m
        .registry
        .manifests()
        .map(|man| m.registry.param_ui(man.id).unwrap_or(&[]).len())
        .sum();
    assert!(
        rotulados >= 400,
        "só {rotulados} params rotulados no catálogo -- a varredura partiu-se"
    );
    let colididos = super::rotulos_colididos();
    assert!(
        colididos.is_empty(),
        "dois controlos lado a lado com o mesmo nome: {colididos:?}"
    );
}

/// **SONDA — que nós nenhuma folha de conferência nomeia.**
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_unconferred_nodes() {
    let c = super::conferencia();
    eprintln!(
        "\n  {} folha(s) lida(s) · {} nós no registry · {} sem folha:\n",
        c.folhas,
        c.nos,
        c.ausentes.len()
    );
    for n in &c.ausentes {
        eprintln!("  ⚠️  {n}");
    }
    eprintln!();
}

/// **A DÍVIDA DA CONFERÊNCIA — nomeada, com censo de obsolescência, e só encolhe.**
///
/// ⛔⛔ Estes nós existem no registry e **nenhuma folha lhes dá uma LINHA**: ou nasceram depois da
/// folha da família deles, ou são citados só dentro da linha de outro nó (como a CURA que fechou
/// aquela célula). Nos dois casos o placar lê `0 aberto` para eles **por ausência**, e não por
/// estarem conferidos.
///
/// ⚠️ **Cada entrada diz a FOLHA a que pertence** — sem isso a lista é um lamento, e com isso é um
/// trabalho endereçado.
///
/// ⭐ **Ela já desceu uma vez: `15 → 9`.** O ciclo 6 W4 conferiu os seis do seu grupo
/// (`pulse.adsr` · `pulse.level` · `pulse.signal` · `value.cursor` · `value.number` ·
/// `value.table`) nas folhas 12 e 15, e foi a **metade de obsolescência** deste gate que exigiu
/// apagá-los daqui — em voz alta, a nomear os seis. *Uma catraca sem essa metade não desce: ela
/// vira licença* (`CLAUDE.md` §5.0).
const SEM_LINHA_DE_CONFERENCIA: &[(&str, &str)] = &[
    ("audio.bands", "07_tempo_estilisticos"),
    // ⚠️⚠️ **Nasceu em 2026-09-19, dez dias depois da folha 16** — o caso que o
    // doc-comment acima nomeia pelo nome (*«ou nasceram depois da folha da família deles»*). A
    // lista subiu de `9` para `10` e isso **não é uma folga**: a dívida por nó não afrouxou, a
    // POPULAÇÃO do catálogo é que cresceu. ⛔ E conferi-lo aqui e agora seria pior: uma linha de
    // conferência responde *«o que é que ELE não tem contra a referência?»*, e a resposta exige
    // CORRER o Spine/Rive/Blender (§0.9), não lembrá-los.
    ("rig.bones", "16_rig"),
    ("motion.bezier_warp", "04_deformers"),
    ("motion.proximity", "08_stream_utilidade"),
    ("motion.randomize", "05_transform"),
    ("motion.sub_uv", "11_fx_raster"),
    ("motion.velocity", "08_stream_utilidade"),
    ("source.lsystem", "14_source"),
    ("source.table", "14_source"),
    ("source.text", "14_source"),
];

/// ⭐⭐ **TODO NÓ DO CATÁLOGO TEM UMA LINHA DE CONFERÊNCIA — ou está nesta lista, com a folha.**
///
/// ⚠️⚠️ **As DUAS metades, senão a catraca vira licença** (`CLAUDE.md` §5.0): ela reprova quando
/// alguém nasce fora das folhas **e** quando uma entrada desta lista já não descreve nada (o nó foi
/// conferido, ou morreu) — *uma lista de dívida que ninguém pode apagar é uma licença permanente.*
///
/// ⚠️ **E o piso de população:** as folhas são lidas em RUNTIME (não há `include_str!` com glob),
/// então um caminho partido devolveria zero folhas e a lista inteira — o gate diz isso em voz alta.
#[test]
fn every_node_has_a_conference_row_or_is_named_in_the_debt() {
    let c = super::conferencia();
    assert!(
        c.folhas >= 17 && c.nos >= 100,
        "leu {} folha(s) e {} nós -- o caminho das folhas ou o registry partiu-se",
        c.folhas,
        c.nos
    );
    let devendo: Vec<&str> = SEM_LINHA_DE_CONFERENCIA.iter().map(|(n, _)| *n).collect();
    let novos: Vec<&str> = c
        .ausentes
        .iter()
        .filter(|n| !devendo.contains(n))
        .copied()
        .collect();
    assert!(
        novos.is_empty(),
        "nó(s) sem linha de conferência e fora da dívida declarada: {novos:?} -- confira-o na \
         folha da família dele, ou acrescente-o a SEM_LINHA_DE_CONFERENCIA com a folha"
    );
    // A metade que faz a catraca DESCER.
    let obsoletos: Vec<&str> = devendo
        .iter()
        .filter(|n| !c.ausentes.contains(n))
        .copied()
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estas entradas de SEM_LINHA_DE_CONFERENCIA já não descrevem nada (o nó ganhou linha, ou \
         deixou de existir) -- apague-as: {obsoletos:?}"
    );
}

/// ⭐⭐⭐ **NENHUM NOME DO CATÁLOGO É CORTADO NUMA CÁPSULA** — ordem do dono (2026-09-19): *«fonts
/// de tamanho único bem alinhadas no centro da cápsula e sem 3 pontos (…)»*.
///
/// ⛔⛔ **A cápsula não corta o nome porque a FONTE foi dimensionada para o pior nome caber** — e
/// essa derivação repousa numa ESTIMATIVA de avanço por caractere. Este censo é o que a torna uma
/// propriedade CONFERIDA: ele mede **cada um dos tipos registados** com o medidor REAL (o mesmo
/// que o corte com reticências consulta), do lado onde o catálogo vive.
///
/// ⚠️ **Um censo de CONTAGEM não serviria:** dezasseis `M` medem `261` unidades e dezasseis letras
/// de um nome real medem `136` — *ele aprovaria um nome que o pintor cortaria*.
///
/// ⚠️ **E o piso de população é obrigatório:** um registo vazio faria este gate varrer zero nomes
/// e passar por vácuo.
#[test]
fn nenhum_nome_do_catalogo_e_cortado_numa_capsula() {
    let m = crate::motion_state::MotionState::new();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut vistos = 0usize;
    let mut nao_cabem: Vec<&'static str> = Vec::new();
    for man in m.registry.manifests() {
        let nome = m
            .registry
            .ui_manifest(man.id)
            .map_or(man.name, |u| ph2d_i18n::tr(u.display_key));
        vistos += 1;
        if !ph2d_panel_motion_graph::nome_cabe_na_capsula(&mut ts, nome) {
            nao_cabem.push(nome);
        }
    }
    assert!(
        vistos >= 130,
        "so' {vistos} tipos no registo — este censo esta' a varrer o vazio"
    );
    assert!(
        nao_cabem.is_empty(),
        "a ESTIMATIVA de largura da capsula e' mais ESTREITA que estes nomes: {nao_cabem:?} — no \
         quadro em que ninguem os mediu eles sairiam CORTADOS. Suba o `AVANCO_POR_CHAR` do \
         `geom_card` (ele paga enchimento, nunca um corte) e re-meca com a sonda \
         `mede_o_corpo_maximo_da_capsula`"
    );
}

/// **SONDA (medição, 2026-09-20): qual é o MAIOR corpo de fonte em que TODO nome do catálogo
/// ainda cabe numa cápsula?** — a ordem do dono foi *«fonts maiores»*, e §0.0 manda medir antes
/// de escrever o número.
///
/// ⛔⛔ **A 1.ª redacção desta sonda mediu a corpo `100` e dividiu**, supondo a largura LINEAR no
/// corpo — e o gate do painel reprovou-a: *«Simulation Zone»* mede `7,3608` por unidade a corpo
/// `100` e **`7,603`** a corpo `22`, **`+3,3 %`**. O arredondamento de métricas por tamanho não é
/// linear, logo *uma medição feita a um corpo não afirma nada sobre outro* ⇒ a busca é **no corpo
/// REAL**, com o medidor que o pintor consulta.
///
/// `cargo test -p ph2d-app-motion -- --ignored --nocapture mede_o_corpo_maximo_da_capsula`
#[test]
#[ignore = "medicao"]
fn mede_o_corpo_maximo_da_capsula() {
    const CARD_W: f32 = 190.0;
    const MARGEM_X: f32 = 12.0;
    let disponivel = CARD_W - 2.0 * MARGEM_X;
    let m = crate::motion_state::MotionState::new();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let nomes: Vec<&'static str> = m
        .registry
        .manifests()
        .map(|man| {
            m.registry
                .ui_manifest(man.id)
                .map_or(man.name, |u| ph2d_i18n::tr(u.display_key))
        })
        .collect();
    eprintln!("  tipos medidos: {}", nomes.len());

    // O pior nome A ESTE corpo — e ele pode MUDAR com o corpo, que é meia razão para a busca.
    let pior_a = |ts: &mut ph2d_text::TextSystem, corpo: f32| -> (f32, &'static str) {
        nomes
            .iter()
            .map(|n| {
                (
                    ts.prefix_width_weighted(n, corpo, ph2d_text::FontWeight::SEMI_BOLD),
                    *n,
                )
            })
            .fold((0.0, ""), |a, b| if b.0 > a.0 { b } else { a })
    };

    // Busca binária sobre o corpo, com o medidor REAL em cada passo.
    let (mut lo, mut hi) = (8.0_f32, 40.0_f32);
    for _ in 0..24 {
        let meio = 0.5 * (lo + hi);
        if pior_a(&mut ts, meio).0 <= disponivel {
            lo = meio;
        } else {
            hi = meio;
        }
    }
    let (w, nome) = pior_a(&mut ts, lo);
    eprintln!("  disponivel {disponivel:.1}");
    eprintln!(
        "  corpo MAXIMO {lo:.3}  (pior: {nome:?} a {w:.2}, folga {:.2})",
        disponivel - w
    );

    // ⭐ **QUANTOS NOMES SERIAM CORTADOS a cada corpo** — a coluna que decide o preço de
    // «fonts 30 % maiores» (ordem do dono, 2026-09-20).
    eprintln!("  --- quantos dos {} seriam CORTADOS ---", nomes.len());
    for corpo in [21.5_f32, 24.0, 25.0, 26.0, 27.95, 30.0] {
        let cortados: Vec<&str> = nomes
            .iter()
            .filter(|n| {
                ts.prefix_width_weighted(n, corpo, ph2d_text::FontWeight::SEMI_BOLD) > disponivel
            })
            .copied()
            .collect();
        eprintln!(
            "  corpo {corpo:>6.2} ({:+.0}%) => {:>3} cortados  {:?}",
            (corpo / 21.5 - 1.0) * 100.0,
            cortados.len(),
            &cortados[..cortados.len().min(6)]
        );
    }

    // ⭐⭐⭐ **A LARGURA SEGUE O NOME** (ordem do dono, 2026-09-20: *«aumenta a largura do
    // retângulo conforme o tamanho do nome»*) ⇒ a fonte deixa de ser limitada pelo pior nome, e
    // o que se mede passa a ser o AVANÇO POR CARACTERE, que é o que a geometria (sem medidor de
    // texto) pode estimar.
    let corpo = 27.95_f32;
    let mut pior_avanco = (0.0_f32, "");
    let mut mais_largo = (0.0_f32, "");
    for n in &nomes {
        let w = ts.prefix_width_weighted(n, corpo, ph2d_text::FontWeight::SEMI_BOLD);
        #[expect(
            clippy::cast_precision_loss,
            reason = "contagem de caracteres cabe num f32"
        )]
        let chars = n.chars().count() as f32;
        let avanco = w / (chars * corpo);
        if avanco > pior_avanco.0 {
            pior_avanco = (avanco, n);
        }
        if w > mais_largo.0 {
            mais_largo = (w, n);
        }
    }
    eprintln!("  --- a corpo {corpo} ---");
    eprintln!(
        "  pior AVANCO por caractere: {:.4} ({:?})",
        pior_avanco.0, pior_avanco.1
    );
    eprintln!(
        "  nome mais LARGO: {:.1} unidades ({:?}) => cartao {:.1}",
        mais_largo.0,
        mais_largo.1,
        mais_largo.0 + 2.0 * MARGEM_X
    );
    for avanco in [0.52_f32, 0.55, 0.58] {
        let estreitos: Vec<&str> = nomes
            .iter()
            .filter(|n| {
                #[expect(clippy::cast_precision_loss, reason = "contagem cabe num f32")]
                let chars = n.chars().count() as f32;
                ts.prefix_width_weighted(n, corpo, ph2d_text::FontWeight::SEMI_BOLD)
                    > chars * corpo * avanco
            })
            .copied()
            .collect();
        eprintln!(
            "  avanco {avanco:.2} => {:>3} nomes ficariam CORTADOS  {:?}",
            estreitos.len(),
            &estreitos[..estreitos.len().min(4)]
        );
    }
    // ⭐⭐ **O PIOR GLIFO** — a estimativa de recurso tem de o aguentar, porque o artista pode
    // renomear um nó para o que quiser.
    eprintln!("  --- pior GLIFO (a estimativa de recurso calibra-se aqui) ---");
    let mut pior = (0.0_f32, ' ');
    for c in "MWmw@%#&QO0AB".chars() {
        let s: String = std::iter::repeat_n(c, 16).collect();
        let w = ts.prefix_width_weighted(&s, corpo, ph2d_text::FontWeight::SEMI_BOLD);
        let avanco = w / (16.0 * corpo);
        if avanco > pior.0 {
            pior = (avanco, c);
        }
        eprintln!("  {c:?} x16 => {w:>7.1}  ({avanco:.4}/caractere)");
    }
    eprintln!("  PIOR: {:?} a {:.4} por caractere", pior.1, pior.0);
}
