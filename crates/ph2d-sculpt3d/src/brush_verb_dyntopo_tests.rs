//! **OS GATES DA TABELA DE QUEM MEXE NA TOPOLOGIA** — irmão (`#[path]`,
//! `cfg(test)`) do [`super`].
//!
//! ⚠️ **O corte é de RESPONSABILIDADE**, forçado pelo tecto de LOC: lá moram as
//! respostas (a tabela medida nos dois oráculos e decidida pelo dono), aqui as
//! provas delas.

use super::Verb;

/// ⭐⭐⭐ **A DENSIDADE LEVA A MALHA AO ALVO NOS DOIS SENTIDOS, SEMPRE.**
///
/// ⚠️⚠️ **Este gate mudou DUAS vezes em 2026-09-14, e os dois passos ficam
/// registados porque o segundo só se entende com o primeiro.**
///
/// 1. Ele nasceu a afirmar o CONTRÁRIO — *«a densidade LIGA o colapso e NÃO
///    liga o partir; um pincel que também subdividisse é outro produto»* —,
///    e quem o desmentiu foi o dono, pelo produto: *«por que não pode
///    aumentar a densidade também?»*. A espec §3.2 traz a tabela-verdade do
///    passe e diz que o pincel **ACRESCENTA a bandeira de colapso** e **não
///    RETIRA** a de partir, que segue o ajuste de refino da cena — medido
///    na mesma malha grossa: `81 → 81` contra **`81 → 101`**.
/// 2. Nasceu então um ajuste com os dois sentidos, e o dono **retirou-o no
///    mesmo dia**: *«não precisamos do modo Thin Only. Deve ser sempre
///    Equalise.»* ⇒ a coluna volta a ser função só do verbo.
///
/// ⭐ *Um gate pode pinar a leitura errada de uma espec tão bem como pina um
/// defeito*, e o que o separou de uma medição foi ninguém ter corrido a
/// outra célula. ⛔ **A recusa medida da espec continua de pé e é OUTRA
/// pergunta:** ela é sobre o pincel **FORÇAR** o partir onde o ajuste da
/// cena o desliga — e nós não temos esse ajuste.
#[test]
fn a_densidade_leva_a_malha_ao_alvo_nos_dois_sentidos() {
    assert!(
        Verb::Density.colapsa_no_dyntopo(),
        "o colapso é a LEI deste pincel — ele acrescenta a bandeira \
         aconteça o que acontecer"
    );
    assert!(
        Verb::Density.refina_no_dyntopo(),
        "a densidade tem de PARTIR também: é a célula `81 -> 101` da espec \
         e o report do dono (*«por que não pode aumentar a densidade \
         também?»*)"
    );
}

/// ⭐⭐⭐ **E ELA CORRE SEM O INTERRUPTOR — ordem do dono.**
///
/// *«Independente se Dynamic topology está ligado ou não, Density faz o seu
/// trabalho. Dynamic topology é para os outros pincéis.»* (2026-09-14)
///
/// ⚠️ **É uma DIVERGÊNCIA DECLARADA da referência** (espec §3.2, 1.ª linha:
/// o modo de detalhe em *Manual* desarma o passe inteiro, este pincel
/// incluído), e o gate afirma-a **pelos dois lados** — senão ele mediria
/// metade e ficaria verde sobre um `true` cravado.
#[test]
fn so_quem_nao_tem_lei_por_vertice_corre_sem_o_interruptor() {
    let livres: Vec<&str> = Verb::ALL
        .into_iter()
        .filter(|v| v.corre_sem_o_interruptor())
        .map(Verb::label)
        .collect();
    assert_eq!(
        livres,
        ["Density"],
        "a porta que dispensa o interruptor alcançou outro verbo — um de \
         carimbo que a herde passa a mudar a topologia com o modo desligado"
    );
    // ⚠️ **O outro lado:** o piso de população impede que um `false` cravado
    // deixe este censo verde a medir uma lista vazia.
    let presos = Verb::ALL
        .into_iter()
        .filter(|v| !v.corre_sem_o_interruptor())
        .count();
    assert!(
        presos >= 25,
        "só {presos} verbos precisam do interruptor — a varredura partiu-se"
    );
}

/// ⭐⭐⭐ **A TABELA, CÉLULA A CÉLULA — e metade dela já é ORÁCULO.**
///
/// ⚠️⚠️ **Este gate mudou em 2026-09-14 porque o ESTUDO CHEGOU**, que é
/// exactamente o que a redacção anterior mandava fazer (*«se foi o estudo a
/// chegar, reescreva este gate célula a célula com a tabela medida ao
/// lado»*). Ele dizia *«só a máscara se afasta do comportamento de hoje»*;
/// hoje são **sete** células, e cada uma tem a proveniência escrita:
///
/// | verbo | colunas | de onde veio |
/// |---|---|---|
/// | **Mask** | `false` | ⭐ **raciocínio de domínio**, curado em 14/09 — e o oráculo livre depois CONCORDOU |
/// | **Smooth** | `false` | ⭐⭐ **oráculo LIVRE** (MIT) — ele não chama a topologia dinâmica, e o dono nomeou-o à letra |
/// | **Snake Hook** | `true` | ⭐⭐ **oráculo LIVRE** — chama-a |
/// | **Move** | `true` | ⭐⭐⭐ **o DONO** — e contra as duas referências |
/// | **Thumb** | `true` | ⭐⭐⭐ **o DONO** — e custou a pegada congelada sobreviver ao refino |
/// | **Erase Displacement** | `false` | ⛔ **construção**: pilha de níveis e malha que muda de contagem não coexistem (espec §5.6) |
/// | **Smear Displacement** | `false` | ⛔ idem |
///
/// ⛔ **E o resto continua a ser o comportamento de hoje**, que é o valor
/// conservador — a metade atrás da parede ainda não chegou.
///
/// ⚠️ **A régua é a PROVENIÊNCIA e não a contagem:** um verbo novo nesta
/// lista reprova aqui, e quem o puser tem de escrever de onde veio a
/// resposta. *Uma tabela sem proveniência por célula é um palpite com cara
/// de lei.*
#[test]
fn cada_desvio_do_comportamento_de_hoje_tem_proveniencia() {
    /// `(verbo, refina?, de onde veio)` — as células que se afastam do que
    /// o produto fazia antes do estudo.
    const DESVIOS: [(&str, bool, &str); 12] = [
        // ⭐⭐ **AS DUAS REFERENCIAS CONCORDAM** — a livre (MIT, lida) e a
        // medida (corrida sem interface pela janela E).
        // ⛔⛔ **NENHUMA referencia responde por este, e a ausencia e' a
        // resposta: ele nao e' um pincel.** O Box Trim nao carimba barro --
        // ele intercepta o arrasto e muda a malha inteira numa booleana, no
        // pen-up. Os motores de refino correm POR DAB, e nao ha' dab.
        // ⚠️ Ele cai no default `!anchors()`, que responderia `true`.
        (
            "Box Trim",
            false,
            "dominio: nao ha' dab (o corte e' uma booleana no pen-up)",
        ),
        ("Smooth", false, "as DUAS: nao mexe (medida: 441 -> 441)"),
        ("Snake Hook", true, "as DUAS: mexe (medida: 441 -> 2 723)"),
        // ⭐ **DOMINIO** — curado um dia antes do estudo, e a referencia
        // livre depois SOBRESCREVEU a lei geral dela para dizer o mesmo.
        ("Mask", false, "dominio; e as DUAS referencias confirmaram"),
        // ⭐⭐⭐ **O DONO, com os olhos no smoke `=14`** — e contra as DUAS
        // referencias, que dizem que o agarrar nao mexe. Ver o braco delas.
        (
            "Move / Grab",
            true,
            "o DONO (14/09): «move/drag deve subdividir»",
        ),
        // ⚠️ **`Thumb`, nao `Clay Thumb`** — sao verbos DIFERENTES, e o
        // segundo e' de carimbo e ja' mexia. O que o dono pediu e' o
        // ANCORADO, que e' o unico que congela a pegada.
        (
            "Thumb",
            true,
            "o DONO (14/09): «Thumb se for possivel, deveria subdividir»",
        ),
        // ⭐ **SO' A MEDIDA responde** (nao existem na referencia livre).
        ("Slide Relax", false, "medida: 441 -> 441 nos dois extremos"),
        (
            "Surface Smooth",
            false,
            "medida: 441 -> 441 nos dois extremos",
        ),
        ("Nudge", true, "medida: 441 -> 2 853 com o alvo fino"),
        // ⛔ **CONSTRUCAO** — nao e' pergunta de oraculo nenhum.
        (
            "Erase Displacement",
            false,
            "espec §5.6: multirresolucao exclui dyntopo",
        ),
        (
            "Smear Displacement",
            false,
            "espec §5.6: multirresolucao exclui dyntopo",
        ),
        // ⭐ **O ORACULO DESTE PINCEL responde a celula dele**, e a leitura
        // e' a mesma dos irmaos: com a topologia dinamica ligada o alvo nao
        // mexe na malha com ele.
        (
            "Draw Sharp",
            false,
            "oraculo do pincel afiado (espec §2.8 e §9.4): nem refina nem colapsa",
        ),
    ];
    let mut corrigidos = Vec::new();
    for v in Verb::ALL {
        // O comportamento de ANTES do estudo: o refino tinha um chamador só
        // — o braço do carimbo —, e quem tem âncora entrava por outro
        // caminho.
        let antes = !v.anchors();
        if v.refina_no_dyntopo() != antes || v.colapsa_no_dyntopo() != antes {
            corrigidos.push(v.label());
        }
    }
    // ⚠️ **A comparação é por CONJUNTO e não por ordem**: a varredura sai na
    // ordem do `Verb::ALL` e a lista está agrupada por PROVENIÊNCIA, que é a
    // informação que ela existe para carregar. *Obrigar as duas ordens a
    // coincidir faria a tabela ser arrumada pela ordem do catálogo, onde a
    // proveniência deixa de se ler.*
    let mut esperados: Vec<&str> = DESVIOS.iter().map(|(n, _, _)| *n).collect();
    esperados.sort_unstable();
    corrigidos.sort_unstable();
    assert_eq!(
        corrigidos, esperados,
        "a tabela mudou uma célula sem passar por aqui. Toda célula que se \
         afasta do comportamento de antes do estudo tem de estar na lista \
         DESVIOS, com a PROVENIÊNCIA ao lado — senão é um palpite com cara \
         de lei"
    );
    // ⭐ **E o valor tem de bater com a proveniência**, senão a tabela
    // documentava uma coisa e o produto fazia outra.
    for (nome, refina, porque) in DESVIOS {
        let v = Verb::ALL
            .into_iter()
            .find(|v| v.label() == nome)
            .unwrap_or_else(|| panic!("`{nome}` saiu do catálogo e a lista ficou para trás"));
        assert_eq!(
            v.refina_no_dyntopo(),
            refina,
            "`{nome}` devia responder {refina} ({porque})"
        );
    }
}

/// ⭐⭐⭐ **OS NOVE VERBOS COM ÂNCORA, TODOS MEDIDOS — e quatro deles mexem.**
///
/// ⚠️⚠️ **Este gate mudou de PREMISSA em 2026-09-14.** Ele chamava-se *«os
/// ancorados por medir continuam a não mexer»* e o `false` deles era o
/// **valor conservador**; hoje os nove têm resposta, e a lista deixou de ser
/// uma dívida para ser uma **tabela**. *Uma catraca cuja população foi
/// inteiramente respondida tem de mudar de forma, senão ela vira licença.*
///
/// | verbo | mexe? | de onde |
/// |---|---|---|
/// | **Snake Hook** | ✅ | as DUAS referências |
/// | **Nudge** | ✅ | a medida (`441 → 2 853`) |
/// | **Move · Thumb** | ✅ | ⭐⭐⭐ **o DONO**, contra as duas referências |
/// | **Twist · Local Scale** | ⛔ | ⭐⭐⭐ **o DONO** desempatou, a favor da medida |
/// | Pose · Boundary · Cloth | ⛔ | as duas, onde as duas existem |
///
/// ⚠️⚠️ **A CONTAGEM não se mexeu e a POPULAÇÃO trocou por baixo dela** — os
/// que mexem continuam a ser **quatro** e são outros dois. *É exactamente
/// como um piso segura o número enquanto a lista que ele descreve muda*, e é
/// por isso que o corpo deste gate compara a LISTA, nunca o tamanho dela.
#[test]
fn os_nove_ancorados_tem_resposta_e_quatro_deles_mexem() {
    /// Os ancorados que MEXEM, com a proveniência no gate irmão.
    const MEXEM: [&str; 4] = ["Snake Hook", "Nudge", "Move / Grab", "Thumb"];
    let ancorados: Vec<&str> = Verb::ALL
        .into_iter()
        .filter(|v: &Verb| v.anchors())
        .map(Verb::label)
        .collect();
    assert!(
        ancorados.len() >= 9,
        "o censo varreu só {} verbos com âncora — a varredura partiu-se: {ancorados:?}",
        ancorados.len()
    );
    // ⭐ **O piso dos que NÃO mexem**: sem ele, ligar a porta a todo gesto
    // ancorado deixaria este gate verde sobre uma lista vazia.
    let parados: Vec<&str> = ancorados
        .iter()
        .copied()
        .filter(|n| !MEXEM.contains(n))
        .collect();
    assert!(
        parados.len() >= 5,
        "só {} ancorados parados — ligar a porta a todos passaria aqui: {parados:?}",
        parados.len()
    );
    for v in Verb::ALL.into_iter().filter(|v| v.anchors()) {
        let deve = MEXEM.contains(&v.label());
        assert_eq!(
            v.refina_no_dyntopo(),
            deve,
            "`{}` tem âncora e a tabela medida diz {deve}",
            v.label()
        );
    }
}

/// ⭐⭐⭐ **O VEREDITO DO DONO SOBRE CINCO CÉLULAS, PRESO AQUI** — 2026-09-14,
/// depois de ele correr o smoke `=14`:
///
/// > *«acho que layer, move/drag deve subdividir. Thumb se for possível,
/// > deveria subdividir. Twist com dynamic topology fica com resultado muito
/// > ruim.»*
///
/// ⚠️⚠️ **ESTE GATE EXISTE PORQUE TRÊS DELAS COINCIDEM COM O VALOR DE
/// FÁBRICA, e uma coincidência não é uma decisão.** O `Layer` é de carimbo
/// e o `_ => !self.anchors()` já lhe responde `true`; a `Twist` e a `Local
/// Scale` têm âncora e ele já lhes responde `false`. ⇒ escrever-lhes um
/// braço no `match` seria uma linha que **a mutação não consegue matar** —
/// o defeito que a densidade já pagou neste mesmo ficheiro. Mas deixá-las
/// sem nada faria a decisão do dono depender de o `anchors()` nunca mudar,
/// e **nada liga as duas perguntas**: quem mexer num grip amanhã inverte um
/// veredito de produto sem que uma linha do diff o diga.
///
/// ⭐ *A decisão vai para onde ela pode ser AFIRMADA — um gate —, e não para
/// onde ela por acaso já é verdade.*
///
/// ⚠️ **A `Local Scale` é a célula que ele NÃO nomeou**, e está aqui de
/// propósito: ela seguiu a irmã (mesmo grip, mesma célula nas duas
/// referências, mesma família na medida). Se ele a quiser de volta a
/// adensar, é esta linha que muda — *uma herança silenciosa e uma decisão
/// leem-se igual numa tabela, e é a lista que as separa.*
/// ⭐⭐⭐⭐ **A TOPOLOGIA DINÂMICA VALE PARA OS PINCÉIS DE PINTURA — ordem do
/// dono** (2026-09-19: *«permita que o dynamic topology funciona para os 3
/// pincéis»*).
///
/// ⛔⛔ **E ela CONTRADIZ o argumento com que a MÁSCARA foi curada, que está
/// escrito nesta mesma tabela:** *«um gesto que não escreve posição não tem
/// porque mudar a topologia»*. Os verbos de pintura também não escrevem
/// posição — e mesmo assim têm de adensar.
///
/// ⭐ **O mecanismo que os separa, e que o argumento da máscara não continha:**
/// a cor por vértice é uma IMAGEM, e a resolução dela **é** a resolução da
/// malha. Pintar numa peça grossa dá manchas do tamanho dos triângulos, e a
/// única maneira de o artista ganhar detalhe é a malha ganhar vértices debaixo
/// do pincel. A máscara é uma SELECÇÃO — ela não tem de desenhar nada — e é
/// isso que a deixa do outro lado. *O discriminador nunca foi «escreve
/// posição?»: é «este canal carrega um DESENHO?».*
///
/// ⚠️ **O valor coincide com o de fábrica** (`_ => !self.anchors()`, e nenhum
/// deles ancora) ⇒ um braço no `match` seria uma linha que a mutação não mata,
/// e é por isso que a decisão mora AQUI — a mesma lei que o irmão abaixo
/// aplica às cinco células de 14/09.
#[test]
fn o_dono_manda_a_topologia_dinamica_valer_para_os_pinceis_de_pintura() {
    let pintura: Vec<Verb> = Verb::ALL.into_iter().filter(|v| v.paints_color()).collect();
    assert!(
        !pintura.is_empty(),
        "piso de população: nenhum verbo declara pintar cor — a extracção \
         partiu-se, e o veredito do dono ficou a afirmar sobre o vazio"
    );
    for v in pintura {
        assert!(
            v.refina_no_dyntopo(),
            "`{}` tem de REFINAR: o dono mandou a topologia dinâmica valer para \
             os pincéis de pintura, e a resolução da cor É a da malha",
            v.label()
        );
        assert!(
            v.colapsa_no_dyntopo(),
            "`{}`: as duas colunas separaram-se debaixo de um veredito que as \
             tratava juntas — se for de propósito, é este gate que muda",
            v.label()
        );
    }
    // ⚠️ **O CONTROLO é a MÁSCARA**, e sem ele isto não afirma nada: se algum
    // dia todo verbo passar a refinar, as asserções acima ficam verdes por
    // vácuo. Ela é o outro lado do discriminador — um canal que não desenha.
    assert!(
        !Verb::Mask.refina_no_dyntopo(),
        "CONTROLO: a máscara voltou a refinar — o discriminador «este canal \
         carrega um DESENHO?» deixou de separar alguém"
    );
}

#[test]
fn o_veredito_do_dono_sobre_cinco_celulas() {
    /// `(verbo, mexe?, o que ele disse)`.
    const VEREDITO: [(&str, bool, &str); 5] = [
        ("Layer", true, "«acho que layer … deve subdividir»"),
        ("Move / Grab", true, "«move/drag deve subdividir»"),
        ("Thumb", true, "«Thumb se for possivel, deveria subdividir»"),
        ("Twist", false, "«Twist … fica com resultado muito ruim»"),
        (
            "Local Scale",
            false,
            "nao nomeado: seguiu a Twist, que e' o mesmo grip e a mesma \
             celula nas duas referencias",
        ),
    ];
    for (nome, mexe, disse) in VEREDITO {
        let v = Verb::ALL
            .into_iter()
            .find(|v| v.label() == nome)
            .unwrap_or_else(|| panic!("`{nome}` saiu do catálogo e o veredito ficou para trás"));
        assert_eq!(
            v.refina_no_dyntopo(),
            mexe,
            "`{nome}` devia responder {mexe} — o dono julgou-o no smoke `=14`: {disse}"
        );
        assert_eq!(
            v.colapsa_no_dyntopo(),
            mexe,
            "`{nome}`: as duas colunas separaram-se debaixo de um veredito \
             de produto"
        );
    }
}

/// ⛔⛔ **OS DOIS VERBOS QUE NENHUMA DAS DUAS REFERÊNCIAS RESPONDE, e eles
/// ficam NOMEADOS em vez de silenciosos.**
///
/// O **Sharpen** e o **Magnify** não têm equivalente na referência livre nem
/// tipo próprio na medida — a corrida da janela E cobriu **27** tipos e
/// nenhum é um deles. ⇒ os dois ficam com o comportamento de antes do
/// estudo (**mexem**, por serem carimbo), e isso é o **valor conservador**,
/// não uma resposta.
///
/// ⚠️ **Sem este gate eles leem-se como decididos** — uma célula sem
/// proveniência e uma com proveniência têm exactamente o mesmo aspecto numa
/// tabela. *É a mesma doença do `❌ recusado com motivo` contra o `❌ ninguém
/// fez` que o §5 deste repo já nomeia.*
#[test]
fn os_dois_verbos_sem_oraculo_ficam_nomeados() {
    const SEM_ORACULO: [&str; 2] = ["Sharpen", "Magnify"];
    for nome in SEM_ORACULO {
        let v = Verb::ALL
            .into_iter()
            .find(|v| v.label() == nome)
            .unwrap_or_else(|| panic!("`{nome}` saiu do catálogo e esta lista ficou para trás"));
        assert!(
            v.refina_no_dyntopo() && v.colapsa_no_dyntopo(),
            "`{nome}` mudou de valor e continua na lista dos SEM ORÁCULO — \
             se alguém o mediu, a entrada tem de sair com a medição ao lado"
        );
    }
}

/// ⚠️⚠️ **AS DUAS COLUNAS COINCIDEM HOJE, E ISSO NÃO É UMA LEI.**
///
/// ⭐ **Este gate já morreu e ressuscitou**, e o ciclo é o registo: ele
/// existia a dizer exactamente isto, **reprovou** em 14/09 quando o pincel
/// de densidade ganhou um ajuste que as separava, foi reescrito com a morte
/// da premissa no diff — e voltou no mesmo dia, quando o dono retirou o
/// ajuste. *Uma premissa que morre e renasce em doze horas é a melhor prova
/// de que ela tinha de estar num gate e não num comentário.*
///
/// ⇒ o estudo (`docs/3D/22`) pode separá-las outra vez, e quando o fizer é
/// aqui que a mudança aparece.
#[test]
fn as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei() {
    let separados: Vec<&str> = Verb::ALL
        .into_iter()
        .filter(|v| v.refina_no_dyntopo() != v.colapsa_no_dyntopo())
        .map(Verb::label)
        .collect();
    assert!(
        separados.is_empty(),
        "as duas colunas separaram-se em {separados:?}. Se foi o ESTUDO a \
         chegar, reescreva este gate com a tabela medida ao lado; se não \
         foi, é uma regressão"
    );
}

/// ⭐⭐⭐ **A FORMA dos cinco que IGNORAM o pente ainda descreve a lista** —
/// o gate que o doc do [`Verb::honra_o_pente`] promete por escrito.
///
/// # O que ele afirma, e porque são TRÊS metades
///
/// A lista é **explícita** porque é um facto MEDIDO no oráculo, verbo a
/// verbo. Um gate que a copiasse não afirmaria nada; este afirma a **forma**
/// que a espec §6.3 dá para ela — *lê as posições de repouso, ou trabalha
/// ancorado, ou não escreve posição nenhuma* — e reprova no dia em que ela
/// deixar de a descrever, **que é o dia em que alguém tem de voltar ao
/// oráculo**.
///
/// ⛔⛔ **A segunda metade é a que impede a lista de ser DERIVADA, e ela tem
/// número:** a forma é **necessária e NÃO suficiente** — seis verbos
/// ancorados (`SnakeHook` · `LocalScale` · `Cloth` · `Nudge` · `Pose` ·
/// `Boundary`) **honram** o pente. *Derivá-la de [`Verb::anchors`] faria a
/// resposta mudar no dia em que alguém mexesse naquela outra pergunta, sem
/// ninguém recontar* — e com seis contra-exemplos medidos ninguém o pode
/// fazer por engano.
///
/// ⚠️ **E o piso de população:** cinco. Sem ele, um `honra_o_pente` que
/// respondesse `true` a toda a gente deixaria as duas metades acima
/// **trivialmente** verdes — o censo por vácuo que este repo já pagou.
#[test]
fn os_cinco_que_ignoram_o_pente_tem_a_forma_que_a_espec_da() {
    // A forma, escrita uma vez: as três propriedades da tabela da espec.
    // ⚠️ `o_dab_segue_o_barro` é a do pincel AFIADO — a lei dele mede a
    // queda das posições do **pen-down**, que é «lê as posições de repouso»
    // dita no vocabulário que o motor já tem.
    let tem_a_forma = |v: Verb| v.anchors() || v.paints_mask() || v.o_dab_segue_o_barro();

    let ignoram: Vec<Verb> = Verb::ALL
        .iter()
        .copied()
        .filter(|v| !v.honra_o_pente())
        .collect();
    assert_eq!(
        ignoram.len(),
        5,
        "o oráculo mediu CINCO verbos a ignorar o pente e hoje são {}              ({ignoram:?}) — se a medição mudou, a espec §6.3 muda com ela",
        ignoram.len()
    );
    let sem_forma: Vec<Verb> = ignoram
        .iter()
        .copied()
        .filter(|&v| !tem_a_forma(v))
        .collect();
    assert!(
        sem_forma.is_empty(),
        "{sem_forma:?} ignora(m) o pente e não cabe(m) na forma que a espec              §6.3 dá (ancorado · pinta máscara · lê as posições do pen-down) —              a forma deixou de descrever a lista, e isso é uma viagem ao oráculo"
    );

    // ⛔ A forma NÃO é suficiente, e é por isso que a lista é explícita.
    let com_forma_e_honram: Vec<Verb> = Verb::ALL
        .iter()
        .copied()
        .filter(|&v| tem_a_forma(v) && v.honra_o_pente())
        .collect();
    assert!(
        com_forma_e_honram.len() >= 6,
        "só {} verbo(s) têm a forma E honram o pente ({com_forma_e_honram:?})              — com menos que isso a lista passaria a parecer DERIVÁVEL de              `anchors()`, e o dia em que alguém mexesse naquela pergunta a              resposta desta mudava sozinha",
        com_forma_e_honram.len()
    );
}
