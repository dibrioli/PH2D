//! ⭐⭐⭐ **O CORPO do painel do ESQUELETO** — as linhas que ele desenha.
//!
//! O que ele oferece é o que o gesto do modo Osso **não** pode dar: prender formas ao esqueleto,
//! soltá-las, e os números de um osso.
//!
//! # Por que Keep Pose E Release
//!
//! É o mesmo par do Envelope, e pela mesma razão: prender sem soltar é **porta de mão única**. Os
//! dois soltam a forma e diferem numa pergunta — *qual geometria fica?* **Keep Pose** materializa a
//! deformada (o que o artista está a ver); **Release** devolve a autorada (o que ele desenhou).
//! Adivinhar qual dos dois ele quer é que não.
//!
//! ⚠️ Os dois só são **pintados** com uma forma presa na seleção (`state::skinned`, publicado pela
//! shell) — *um botão que só sabe recusar é pior que um botão ausente*. O **Bind** é pintado sempre:
//! ele age sobre a seleção, e recusar em voz alta é mais honesto que esconder a porta de entrada.
//!
//! # ⚠️ Funções LIVRES sobre o [`RowCtx`], e não métodos
//!
//! O vocabulário de linhas vive na `ph2d-editor-core` e o Rust não deixa uma crate acrescentar
//! métodos inerentes a um tipo de outra. *A regra do órfão é o que escolhe a forma aqui* — e ela
//! não custa nada: o `r` é o mesmo contexto que o painel de vetor passa aos dele.

use crate::section_campos::{
    ANGLE_STEP, LENGTH_STEP, STRENGTH_STEP, campos_da_ancora, campos_do_osso,
};
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::RowCtx;
use ph2d_i18n::tr;

/// Seção **SKELETON** — prender ao esqueleto, as duas saídas, e o osso em foco.
/// ⭐⭐⭐ **ESTE SEGMENTO ESTÁ ACESO?** — a lei que o dono descreveu, numa porta.
///
/// ⛔⛔ **Ordem do dono, 2026-09-09:**
///
/// | estado | o que acende |
/// |---|---|
/// | *«se não há ossos no mundo»* (nada armado) | **nenhum**, até ele carregar em *Create* |
/// | *«ao seleccionar o osso»* | **Transform** (a shell arma-o na aresta do foco) |
///
/// ⚠️ **Ela é uma função e não duas comparações no meio da pintura** porque é a única coisa aqui
/// que um gate consegue observar: o estado *aceso* de um segmento **não vive no `WidgetStore`** —
/// ele é passado ao pintor a cada quadro. *Uma lei que só existe dentro de uma chamada de pintura
/// não tem como ser medida.*
///
/// ⚠️ **`None` é «nada armado», e não um terceiro estado do enum**: «armado» já é o modo Osso estar
/// na mão, e um terceiro valor diria a mesma coisa duas vezes.
#[must_use]
pub(crate) fn aceso(armado: Option<usize>, segmento: usize) -> bool {
    armado == Some(segmento)
}

/// ⭐⭐ **OS DOIS SEGMENTOS DA DIRECÇÃO DO PINCEL DE PESO** — `(id, chave do rótulo, aceso)`.
///
/// ⚠️ **Ela existe para a escolha ser CHAMÁVEL de um teste.** Escrita no sítio onde é pintada, a
/// única forma de a julgar seria ler o ficheiro como texto ou olhar para pixels — e o que se quer
/// afirmar é *«o índice que a shell publica acende o segmento certo»*, que é uma pergunta sobre uma
/// conta. ⛔ E a conta é a [`aceso`], a mesma da fileira dos verbos: *duas respostas a «qual
/// segmento acende» divergem no dia do primeiro ajuste.*
///
/// ⛔⛔ **O ID e o RÓTULO viajam EMPARELHADOS, e isso nasceu de uma mutação SOBREVIVENTE:** com a
/// lista de ids de um lado e a de rótulos do outro, trocar a ordem de UMA delas não reprovava nada
/// — e o resultado é o botão que diz *Add* a mandar `Subtract`. *Um controlo que faz o contrário do
/// que o rótulo dele diz é pior do que um controlo morto: o morto não engana.*
pub(crate) fn segmentos_da_direccao(lado: usize) -> [(NodeId, &'static str, bool); 2] {
    [
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ADD,
            "panel.vector.bone.weight.add",
            aceso(Some(lado), 0),
        ),
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_SUB,
            "panel.vector.bone.weight.subtract",
            aceso(Some(lado), 1),
        ),
    ]
}

/// ⭐⭐⭐ **OS DOIS SEGMENTOS DO MODO DE ATRIBUIR PESO** — `(id, chave do rótulo, aceso)`, irmã da
/// [`segmentos_da_direccao`] e pela MESMA razão: *uma lei que só existe dentro de uma chamada de
/// pintura não tem como ser medida*, e o par id↔rótulo viaja junto para que trocar a ordem de uma
/// lista não deixe um botão a fazer o contrário do que diz.
pub(crate) fn segmentos_do_modo(modo: usize) -> [(NodeId, &'static str, bool); 2] {
    [
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_CUMUL,
            "panel.vector.bone.weight.mode.cumulative",
            aceso(Some(modo), 0),
        ),
        (
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_ABS,
            "panel.vector.bone.weight.mode.absolute",
            aceso(Some(modo), 1),
        ),
    ]
}

/// ⭐⭐⭐ **A FILEIRA DA DIRECÇÃO É PINTADA?** — não, no modo absoluto (ordem do dono, 2026-09-19:
/// *«neste modo os botões Add e Subtract ficam inactivos»*).
///
/// ⚠️ **É uma função e não uma comparação no meio da pintura, e pela mesma razão da [`aceso`]:**
/// *uma lei que só existe dentro de uma chamada de pintura não tem como ser medida*.
///
/// ⚠️ ***Inactivo* aqui é AUSENTE e não cinzento** — a lei da casa (*esconde-se o que se pode; diz-se
/// a razão onde uma cerca de produto proíbe esconder*), e nada proíbe: a fileira chama-se
/// *Direction*, e num pincel absoluto não há direcção nenhuma para nomear.
pub(crate) fn pinta_a_direccao(modo: usize) -> bool {
    modo != ph2d_tool_vector::WeightMode::Absolute.indice()
}

/// ⭐⭐⭐ **A CHAVE DO RÓTULO DO NÚMERO, PELO MODO** — *Strength* no cumulativo, *Weight* no absoluto.
///
/// ⚠️⚠️ **O mesmo controlo, dois significados, e é a ordem do dono que o exige** (*«o valor de
/// *Brush Strength* é posto imediatamente no osso»*): no modo cumulativo o número é *quanto
/// empurrar*, no absoluto é *que valor pôr*. ⛔ Dois campos seriam duas superfícies sobre um valor
/// — a armadilha que os três chips do `Detail` da escultura pagaram —, e um rótulo fixo seria um
/// controlo que mente sobre metade do curso.
///
/// ⭐ **É uma função e não um `if` dentro da pintura** porque é isto que um gate consegue observar.
pub(crate) fn rotulo_do_numero(modo: usize) -> &'static str {
    if modo == ph2d_tool_vector::WeightMode::Absolute.indice() {
        "panel.vector.bone.weight.target"
    } else {
        "panel.vector.bone.weight.amount"
    }
}

pub(crate) fn body(r: &mut RowCtx, y: f32) -> f32 {
    // ⭐⭐⭐ **REVELAR-AO-FOCAR** (report do dono, 2026-09-08: *«selecionar o bone nem sempre
    // abre a secção de skeleton»*). ⚠️ **O pedido consome-se AQUI, antes da porta**: se o corpo
    // não é pintado neste quadro não há nada a revelar, e um pedido que sobrevive rolaria o
    // painel muito depois do gesto que o armou.
    //
    // ⚠️ **A porta que decidia se a SECÇÃO existia saiu daqui** (2026-09-09): quem decide se este
    // painel é pintado é a shell (`panel_visible`), pela mesma lei que a secção seguia — *um painel
    // que fala de algo que não existe é ruído*.
    // ⛔ **Sem cabeçalho de secção**, e a ausência é a decisão (2026-09-09): num painel PRÓPRIO o
    // título dele **é** o cabeçalho, e uma secção única lá dentro escreveria o nome duas vezes — e
    // daria uma dobra que colapsa um painel que já se fecha.
    let mut y = y;
    // ⭐⭐⭐ **CRIAR × TRANSFORMAR, no TOPO — e ele é a PORTA, não um refinamento.**
    //
    // ⛔⛔ **Ordem do dono, 2026-09-09:** o pill `Bone` saiu da fileira de modos do painel de vector
    // (*«melhor tirar de lá»*), e com ele foi-se a única porta para o modo. Estes dois segmentos
    // passaram a **trocar de modo E de verbo**.
    //
    // ⚠️ **Ele é pintado SEMPRE, e é aí que mora a regra que o dono descreveu:**
    //
    // | estado | o que acende |
    // |---|---|
    // | nenhum osso no mundo, painel aberto pelo menu | **nada**, até ele carregar em *Create* |
    // | um osso seleccionado | **Transform** (a shell arma-o na aresta do foco) |
    //
    // ⇒ o que acende sai de `state::bone_tool()`, que é `None` fora do modo Osso. *Um terceiro
    // estado do enum diria a mesma coisa duas vezes* — «armado» já é o modo estar na mão.
    //
    // ⚠️ Ele vem antes dos verbos porque **decide o que os outros controlos significam**: com
    // *Criar* o arrasto faz osso, com *Transformar* ele posa — e ler isso depois de já ter carregado
    // é tarde.
    let armado = state::bone_tool();
    {
        let rotulos = [
            tr("panel.vector.bone.create"),
            tr("panel.vector.bone.transform"),
            tr("panel.vector.bone.weight"),
        ];
        let acoes: [(NodeId, &str, bool); 3] = [
            (
                ph2d_tool_vector::ids::VECTOR_BONE_ACT_CREATE,
                rotulos[0],
                aceso(armado, 0),
            ),
            (
                ph2d_tool_vector::ids::VECTOR_BONE_ACT_TRANSFORM,
                rotulos[1],
                aceso(armado, 1),
            ),
            (
                ph2d_tool_vector::ids::VECTOR_BONE_ACT_WEIGHT,
                rotulos[2],
                aceso(armado, 2),
            ),
        ];
        y = r.segmented(tr("panel.vector.bone.action"), &acoes, y);
    }
    // ⭐⭐⭐ **O PINCEL DE PESO — a DIRECÇÃO e os dois números, e SÓ com ele na mão.**
    //
    // ⚠️ **Escondidos fora do verbo, e é a lei da casa** (*«o painel mostra o que serve à
    // FERRAMENTA na mão»*, `section_scope`): pintados sempre, eles seriam controlos que não fazem
    // nada em dois dos três verbos — que é a espécie de morto que o §5.0 nomeia.
    //
    // ⛔⛔⛔ **Até 2026-09-19 esta linha dizia *«o `Amount` é COM SINAL, e é isso que faz o gesto
    // ser um só: negativo TIRA peso. ⛔ Um segundo chip apagar seria a segunda maneira de dizer a
    // mesma coisa»*.** A premissa morreu por **ordem do dono**: *«no lugar de valores negativos em
    // Brush Strength prefiro botões Add e Subtract»*. A objecção fica **registada e não vencida**,
    // e o que ela não via está escrito na [`ph2d_tool_vector::WeightDirection`]: enquanto o sinal
    // vivia no número, *«tirar peso»* era um **estado invisível** — o artista tinha de ler um menos
    // para saber o que o próximo arrasto ia fazer.
    //
    // ⚠️ **A direcção vem ANTES dos números, e é a mesma razão da fileira dos verbos acima:** ela
    // decide o que o `Strength` significa, e ler isso depois de já ter arrastado é tarde. ⭐ E os
    // dois números ficam JUNTOS, que é o que eles são.
    if armado == Some(2) {
        let (_, _, lado, modo) = state::bone_weight();
        // ⭐⭐⭐ **O MODO VEM PRIMEIRO, e é a mesma razão da fileira dos verbos acima:** ele decide o
        // que os outros controlos SIGNIFICAM — no cumulativo o número é *quanto empurrar* e a
        // direcção escolhe o sinal; no absoluto ele é *que valor pôr* e a direcção não tem sujeito.
        let acesos = segmentos_do_modo(modo);
        let modos: [(NodeId, &str, bool); 2] =
            std::array::from_fn(|i| (acesos[i].0, tr(acesos[i].1), acesos[i].2));
        y = r.segmented(tr("panel.vector.bone.weight.mode"), &modos, y);
        // ⚠️ **A fileira é DERIVADA** ([`segmentos_da_direccao`]) — um `lado == 0` escrito aqui
        // seria a terceira resposta a *«qual segmento acende»*, ao lado da [`aceso`] que a fileira
        // dos verbos já usa. ⛔ E o RÓTULO vem de lá emparelhado com o id, nunca de uma segunda
        // lista ao lado: *duas listas na mesma ordem trocam-se uma sem a outra, e o sintoma é um
        // botão que faz o contrário do que diz.*
        //
        // ⛔⛔ **E ela SOME no modo absoluto** (ordem do dono, 2026-09-19: *«neste modo os botões
        // Add e Subtract ficam inactivos»*). ⚠️ *Inactivo* aqui é **ausente** e não cinzento, que é
        // a lei da casa — *esconde-se o que se pode; diz-se a razão onde uma cerca de produto
        // proíbe esconder* (o `Density` da escultura é o precedente) —, e nada proíbe: a fileira
        // chama-se *Direction* e num pincel absoluto não há direcção nenhuma para nomear.
        if pinta_a_direccao(modo) {
            let acesos = segmentos_da_direccao(lado);
            let direccoes: [(NodeId, &str, bool); 2] =
                std::array::from_fn(|i| (acesos[i].0, tr(acesos[i].1), acesos[i].2));
            y = r.segmented(tr("panel.vector.bone.weight.direction"), &direccoes, y);
        }
        y = r.labeled_number_field(
            tr("panel.vector.bone.weight.radius"),
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_RADIUS,
            LENGTH_STEP,
            y,
        );
        // ⚠️ **O RÓTULO segue o MODO** — ver [`rotulo_do_numero`]. O id é o MESMO: é um número só, e
        // dois campos seriam duas superfícies sobre um valor.
        y = r.labeled_number_field(
            tr(rotulo_do_numero(modo)),
            ph2d_tool_vector::ids::VECTOR_BONE_WEIGHT_AMOUNT,
            STRENGTH_STEP,
            y,
        );
    }
    // Tabela tipada como a do Blend e a do Envelope (HR-12): o `action_button` delega ao
    // `paint_button` canónico, que é quem costura o AccessKit — e nomear o [`ph2d_a11y::NodeId`]
    // aqui é o idioma que o gate `every_widget_file_wires_a11y` lê.
    let verbos: [(ph2d_a11y::NodeId, &str); 1] =
        [(ids::VECTOR_BONE_BIND, tr("panel.vector.bone.bind"))];
    for (id, label) in verbos {
        y = r.action_button(id, label, y);
    }
    // ⭐⭐⭐ **AS SAÍDAS APARECEM PARA AS DUAS MÍDIAS — o *Expand* só para UMA.**
    //
    // ⛔⛔ **O `Expand` não alcança uma imagem, e a razão é da MÍDIA e não da fiação:** ele troca o
    // desenho autorado pela geometria deformada de agora, e uma imagem **não tem geometria
    // autorada** — a malha é derivada da própria tinta, por quadro. Assar a deformação nos pixels é
    // **outra** operação, e ela não existe. ⇒ ele é **escondido**, que é o que a casa faz quando
    // pode (a alternativa — pintá-lo e recusar — deixa um controlo morto sob o dedo).
    let presa = state::skinned();
    if presa.alguma() {
        if presa.vector {
            y = r.action_button(ids::VECTOR_BONE_EXPAND, tr("panel.vector.bone.expand"), y);
        }
        y = r.action_button(ids::VECTOR_BONE_RELEASE, tr("panel.vector.bone.release"), y);
        y = skin_law_row(r, y);
    }
    // ⛔⛔⛔ **A FILEIRA `Deform` SAIU** (ordem do dono, 2026-09-17: *«Pode apagar a secção
    // deform»*), e o que a matou foi uma MEDIÇÃO.
    //
    // Ela nasceu do report de 2026-09-10 (*«ao dobrar a articulação temos arestas retas na
    // imagem»*), quando a malha do bind era uma grelha uniforme e os pesos eram euclidianos. As
    // duas waves seguintes — a grelha **graduada pelas articulações** e os pesos do
    // **padrão-ouro** com a lei de Hermite — curaram a faceta na própria malha do bind, e ninguém
    // reconferiu a nota. Medido em 2026-09-17, em pixels de ECRÃ (a unidade do olho, e não o desvio
    // ao campo em pixels da ARTE, que é o que todas as réguas desta linha mediam): as duas leis
    // punham a tinta a **`0,04 px` na mediana e `0,34 px` no pior ponto**, e o dono reportou
    // (repetidamente) que eram *«sempre idênticas»*. Ele tinha razão.
    //
    // ⇒ o knob inteiro morreu — o enum, o campo do `VectorDrawConfig`, os dois ids, o espelho da
    // shell e esta fileira. ⛔ **Apagado e não escondido:** um controlo vivo e inalcançável é a
    // classe que o `CLAUDE.md` §5.0 nomeia, e o precedente é o `Ctrl` do *Scene Project* (15/09),
    // onde retirar o gesto retirou a capacidade.
    // Os dois números do OSSO em foco. Sem osso não há sujeito — e um campo sem sujeito é a
    // classe de controlo morto que o `CLAUDE.md` §5.0 nomeia.
    if let Some(campos) = campos_do_osso() {
        // ⭐⭐⭐ **A POSE DE REPOUSO, ANTES dos números** — e a ordem é a lei: estes dois são sobre
        // *onde o osso volta*, e os números abaixo são sobre *o que ele é*. ⚠️ Pintá-los depois
        // dos campos poria o gesto mais frequente do rig (experimentar uma pose e voltar) no fim
        // de uma lista que rola.
        //
        // ⚠️ **Só com osso em foco**, como todo o resto deste bloco: um *Rest Pose* sem sujeito
        // seria a classe de controlo morto que o `CLAUDE.md` §5.0 nomeia.
        // ⚠️ **Os TRÊS agem sobre este osso E a descendência dele**, e é o que os junta numa
        // tabela: os dois primeiros sobre a POSE do ramo, o terceiro constrói o ramo do outro lado.
        let repouso: [(ph2d_a11y::NodeId, &str); 3] = [
            (
                ids::VECTOR_BONE_REST_APPLY,
                tr("panel.vector.bone.rest.apply"),
            ),
            (ids::VECTOR_BONE_REST_SET, tr("panel.vector.bone.rest.set")),
            (ids::VECTOR_BONE_MIRROR, tr("panel.vector.bone.mirror")),
        ];
        for (id, label) in repouso {
            y = r.action_button(id, label, y);
        }
        for (id, label, step, _valor) in campos {
            y = r.labeled_number_field(label, id, step, y);
        }
        y = handles_row(r, y);
        y = crate::section_tip::tip_row(r, y);
        y = limit_rows(r, y);
        y = crate::section_smart::smart_rows(r, y);
        y = ik_rows(r, y);
    }
    y
}

/// ⭐⭐⭐ **O LIMITE DE ÂNGULO** da junta em foco — a porta de entrada, ou os dois extremos.
///
/// ⚠️ **Vem ANTES da âncora**, e a ordem diz o que ele é: o limite é uma propriedade da JUNTA
/// (vale com IK e sem ela), a âncora é uma restrição que se põe e se tira. Pô-lo dentro do
/// bloco da IK ensinaria que ele é parte dela — que é exactamente o desenho do Godot, e o que
/// esta casa decidiu não fazer.
fn limit_rows(r: &mut RowCtx, y: f32) -> f32 {
    let Some(_) = state::current_bone_limit() else {
        return r.action_button(
            ids::VECTOR_BONE_LIMIT_ADD,
            tr("panel.vector.bone.limit.add"),
            y,
        );
    };
    let mut y = r.action_button(
        ids::VECTOR_BONE_LIMIT_REMOVE,
        tr("panel.vector.bone.limit.remove"),
        y,
    );
    let campos: [(ph2d_a11y::NodeId, &str); 2] = [
        (
            ids::VECTOR_BONE_LIMIT_MIN,
            tr("panel.vector.bone.limit.min"),
        ),
        (
            ids::VECTOR_BONE_LIMIT_MAX,
            tr("panel.vector.bone.limit.max"),
        ),
    ];
    for (id, label) in campos {
        y = r.labeled_number_field(label, id, ANGLE_STEP, y);
    }
    y
}

/// ⭐⭐⭐ **A ÂNCORA DE IK** do osso em foco — a porta de entrada, ou os três números dela.
///
/// ⚠️ **É um OU exclusivo, e é a lei do controlo morto:** *Add IK* só aparece em quem não tem
/// âncora, e *Remove IK* mais os três números só em quem tem. Oferecer as duas portas ao mesmo
/// tempo daria um botão que só sabe recusar — e o gesto recusa-o também, então o painel estaria
/// a prometer o que o app não faz.
/// ⭐⭐⭐ **O RÓTULO DO CHIP `Auto`, que DIZ o lado que ele está a derivar** (report do dono,
/// 2026-09-18: *«IK Bend não funcionou com Auto IK e trocando CCw por CW»*).
///
/// ⛔⛔⛔ **A causa está MEDIDA e não era a fiação:** com o `Chain` de fábrica (`2`) o `Auto` e o `Cw`
/// dão a **MESMA pose, ao bit** (soma das diferenças de rotação `0,0000`; com `Chain = 3` ela é
/// `2,9991`), e a cena do osso captura `Cw`. ⇒ o artista clicava em **dois** dos quatro chips e não
/// via nada mudar — indistinguível de um controlo partido. *Só o `Ccw` move (`3,0000`).*
///
/// ⚠️ **Não se esconde o `Auto`:** ele significa *«deriva o lado da pose que chega»* e coincide
/// NESTA pose, não sempre. ⇒ ele **diz**, e o artista vê sem clicar que pedir esse lado é um no-op.
///
/// ⚠️ **É uma função PURA e não um `format!` dentro do pintor**, porque o testkit desta casa não
/// tem leitor de texto pintado: *quando a lei fica dentro do pintor, o gate dela não existe*.
/// ⏳ A dívida fica nomeada — o pixel não é alcançável de um teste, e o que o liga é o gate
/// `include_str!` que exige o pintor a chamar esta porta.
fn rotulo_do_auto(derivado: Option<usize>) -> String {
    // ⛔ Só `Ccw` (1) e `Cw` (2) são lados nomeáveis: o `Keep` (0) seria circular e o `Mixed` (3)
    // não tem um lado só. *Um rótulo que inventa um lado é pior que nenhum.*
    let lado = derivado.and_then(|i| match i {
        1 => Some(tr("panel.vector.bone.ik.bend.ccw")),
        2 => Some(tr("panel.vector.bone.ik.bend.cw")),
        _ => None,
    });
    match lado {
        Some(l) => ph2d_i18n::tr_with("panel.vector.bone.ik.bend.auto_is", &[("lado", &l)]),
        None => tr("panel.vector.bone.ik.bend.auto").to_string(),
    }
}

fn ik_rows(r: &mut RowCtx, y: f32) -> f32 {
    let Some((_, _, _, lado)) = state::current_bone_ik() else {
        // ⭐⭐⭐ **DUAS portas de entrada, e a diferença é o que a restrição FAZ:** *Add IK* dá uma
        // corrente que **ALCANÇA** o alvo (o membro, dois ossos), *Look At* dá um osso que
        // **APONTA** para ele (o olhar, a cabeça, o canhão de uma torre).
        //
        // ⚠️ **Não são dois motores** — a medição está na `sonda_do_apontar_tests`: a mesma lei do
        // alcance, com a corrente resolvida em UM, já apontava com erro `0,000000°`. *O que faltava
        // era o nome.*
        let y = r.action_button(ids::VECTOR_BONE_IK_ADD, tr("panel.vector.bone.ik.add"), y);
        return r.action_button(ids::VECTOR_BONE_LOOK_AT, tr("panel.vector.bone.look_at"), y);
    };
    let mut y = r.action_button(
        ids::VECTOR_BONE_IK_REMOVE,
        tr("panel.vector.bone.ik.remove"),
        y,
    );
    // ⚠️ **A lista já vem filtrada pela lente** — ver [`campos_da_ancora`]. ⛔ Filtrar aqui faria a
    // SEMEADURA dos valores (que percorre a mesma porta, no `paint`) e a PINTURA discordarem: um
    // campo semeado e não pintado é inofensivo, um pintado e não semeado mostra o `0` com que
    // nasceu — que é o report do dono de 2026-09-14, à letra.
    for (id, label, step, _valor) in campos_da_ancora().unwrap_or_else(|| {
        unreachable!("a âncora existe: o `let Some(..) = current_bone_ik()` acima devolveu cedo")
    }) {
        y = r.labeled_number_field(label, id, step, y);
    }
    // ⛔ **E o lado da dobra sai com o resto no apontar:** não há cotovelo num osso só, logo não há
    // lado para dobrar — medido inerte na `sonda_do_apontar_tests`.
    if state::current_bone_aim().is_some() {
        return y;
    }
    // ⭐⭐⭐ **PARA QUE LADO O JOELHO DOBRA** — e ele vem DEPOIS de `Chain` porque é o `Chain`
    // que decide quantos ossos têm lado: ler *«de que lado»* antes de saber *«de que corrente»*
    // é ler a resposta antes da pergunta.
    //
    // ⚠️ A fileira é construída a partir de [`ph2d_skeleton::BendSide::ALL`] e da tabela de ids
    // **ao mesmo tempo**, por índice: uma variante nova na lei sem um id ao lado é erro de
    // compilação (os dois arrays têm de ter o mesmo comprimento), em vez de um segmento que
    // desaparece em silêncio.
    // ⚠️⚠️ **A promessa de «erro de compilação» era FALSA até 2026-09-14**: as duas listas eram
    // indexadas lado a lado e um comprimento diferente dava **pânico em runtime**, apanhado só pelo
    // gate de costura. Este `assert!` de `const` torna-a verdadeira — e foi preciso acrescentar uma
    // variante (`Mixed`) para o descobrir.
    const _: () = assert!(
        ids::VECTOR_BONE_BEND_IDS.len() == ph2d_skeleton::BendSide::ALL.len(),
        "um lado novo na LEI precisa de um id ao lado dele"
    );
    let auto = rotulo_do_auto(state::current_bone_ik_auto_side());
    let rotulos = [
        auto.as_str(),
        tr("panel.vector.bone.ik.bend.ccw"),
        tr("panel.vector.bone.ik.bend.cw"),
        tr("panel.vector.bone.ik.bend.mixed"),
    ];
    let mut lados: [(ph2d_a11y::NodeId, &str, bool); ph2d_skeleton::BendSide::ALL.len()] =
        [(ids::VECTOR_BONE_BEND_IDS[0], "", false); ph2d_skeleton::BendSide::ALL.len()];
    for (i, slot) in lados.iter_mut().enumerate() {
        *slot = (ids::VECTOR_BONE_BEND_IDS[i], rotulos[i], i == lado);
    }
    r.segmented(tr("panel.vector.bone.ik.bend"), &lados, y)
}

/// ⭐⭐⭐⭐ **POR QUE LEI ESTE DESENHO SE DEFORMA** — a escolha que o dono mandou construir
/// (2026-09-19: *«como se escolhe se os envelopes vão ou não influenciar?»* ⇒ *«construa. por
/// desenho»*).
///
/// ⛔⛔⛔ **Antes disto NÃO SE ESCOLHIA, e a régua mediu-o:** uma forma com interior (ou uma imagem
/// que resolve) ia para o padrão-ouro e o alcance ficava **inerte** (amplitude `0,000000` numa faixa
/// de `80 ×`); um traço ABERTO caía na lei euclidiana, onde o alcance manda. *Qual lei deforma o
/// personagem é uma decisão de RIG, e ela estava escondida dentro de uma decisão de DESENHO.*
///
/// ⚠️ **O SUJEITO É A SELECÇÃO DE FORMAS, não o osso em foco** — é isso que faz dela uma escolha
/// *por desenho*, e é por isso que os dois ids vivem em
/// [`ph2d_tool_vector::ids::VECTOR_BONE_ON_SELECTION`]. ⇒ ela é pintada com o *Release*, ao lado
/// das outras duas coisas que agem sobre o que está escolhido, e **só quando há algo preso**:
/// *sem pele não há lei de pele, e um selector sem sujeito é a classe de controlo morto que o
/// `CLAUDE.md` §5.0 nomeia.*
///
/// ⚠️ **E ela é pintada para as DUAS mídias**, ao contrário do *Expand*: a pergunta é a mesma para
/// uma forma e para uma imagem, e a lei que a responde é uma só
/// (`ph2d_skeleton_ecs::SkinBind::pesos_do_quadro`).
fn skin_law_row(r: &mut RowCtx, y: f32) -> f32 {
    // ⚠️ A MESMA lei de alinhamento por índice das duas fileiras de chips abaixo, e o mesmo
    // `assert!` de `const`: uma lei nova sem um id ao lado **não compila**.
    const _: () = assert!(
        ids::VECTOR_BONE_SKIN_LAW_IDS.len() == 2,
        "uma lei de pele nova precisa de um id ao lado dela"
    );
    let envelope = state::skin_law_envelope();
    let leis: [(NodeId, &str, bool); 2] = [
        (
            ids::VECTOR_BONE_SKIN_LAW_AUTO,
            tr("panel.vector.bone.skin_law.auto"),
            !envelope,
        ),
        (
            ids::VECTOR_BONE_SKIN_LAW_ENVELOPE,
            tr("panel.vector.bone.skin_law.envelope"),
            envelope,
        ),
    ];
    r.segmented(tr("panel.vector.bone.skin_law"), &leis, y)
}

/// ⭐⭐⭐⭐ **DE ONDE VÊM AS DUAS ALÇAS DE CURVATURA** — o *Handle Type* do *Bendy Bone*.
///
/// ⚠️ **Só é pintada num osso que a sabe LER**, pela mesma régua e pela mesma razão que os quatro
/// números da curvatura: com um segmento só a curvatura é **provadamente inerte**, e um segmentado
/// que grava no documento sem mudar um pixel é o painel a mentir.
///
/// ⚠️ **Vem DEPOIS dos quatro números e ANTES do limite**, e a ordem diz o que ela é: ela decide
/// **quem escreve** aqueles quatro — ler *«quanto»* antes de *«quem»* é ler a resposta antes da
/// pergunta, que é a mesma ordem que o `Chain`/`Bend` da âncora já segue.
fn handles_row(r: &mut RowCtx, y: f32) -> f32 {
    let Some(osso) = state::current_bone() else {
        return y;
    };
    if osso.is_rigid() && ph2d_skeleton::bend::segments_of(osso.segments) <= 1 {
        return y;
    }
    // ⚠️ A MESMA lei de alinhamento por índice da fileira do lado da dobra, e o mesmo `assert!` de
    // `const`: um modo novo na lei sem um id ao lado **não compila**.
    const _: () = assert!(
        ids::VECTOR_BONE_HANDLES_IDS.len() == 2,
        "um modo novo de alca precisa de um id ao lado dele"
    );
    let auto = state::current_bone_handles() == Some(1);
    let modos: [(NodeId, &str, bool); 2] = [
        (
            ids::VECTOR_BONE_HANDLES_AUTHORED,
            tr("panel.vector.bone.handles.authored"),
            !auto,
        ),
        (
            ids::VECTOR_BONE_HANDLES_AUTO,
            tr("panel.vector.bone.handles.auto"),
            auto,
        ),
    ];
    r.segmented(tr("panel.vector.bone.handles"), &modos, y)
}

#[cfg(test)]
mod tests {
    use super::aceso;

    /// ⭐⭐⭐ **NADA ACESO ATÉ O ARTISTA CARREGAR** — a ordem do dono, palavra por palavra.
    ///
    /// ⛔ *«Se não há ossos no mundo nenhum botão fica selecionado até o usuário apertar Create»*
    /// (2026-09-09). ⚠️ **A metade que importa é a PRIMEIRA:** um default aceso faria o painel
    /// afirmar um verbo que a ferramenta não tem armado — e o arrasto no canvas faria outra coisa
    /// que a fileira diz.
    #[test]
    fn nothing_is_lit_until_the_artist_arms_it() {
        assert!(
            !aceso(None, 0) && !aceso(None, 1),
            "um segmento nasceu aceso sem nada armado"
        );
        assert!(aceso(Some(0), 0) && !aceso(Some(0), 1), "*Create* armado");
        assert!(
            !aceso(Some(1), 0) && aceso(Some(1), 1),
            "*Transform* armado"
        );
    }

    /// ⭐⭐⭐ **O ÍNDICE QUE A SHELL PUBLICA ACENDE O SEGMENTO CERTO** — a direcção do pincel de
    /// peso (ordem do dono, 2026-09-19).
    ///
    /// ⚠️ **As três metades:** o segmento publicado acende, o outro **não** (senão os dois acesos
    /// leem-se como «nenhum escolhido»), e a fileira sai da lista de ids na ordem dela — *um par
    /// trocado acenderia o `Add` quando a ferramenta está a tirar peso, e o artista veria o
    /// contrário do que o próximo arrasto faz.*
    #[test]
    fn o_indice_publicado_acende_o_lado_do_pincel() {
        for publicado in 0..2 {
            let fileira = super::segmentos_da_direccao(publicado);
            for (i, (id, chave, on)) in fileira.iter().enumerate() {
                assert_eq!(
                    *id,
                    crate::ids::VECTOR_BONE_WEIGHT_DIR_IDS[i],
                    "a fileira e a lista que o `populate` regista deixaram de concordar na ORDEM — \
                     um segmento passa a carregar o id do outro"
                );
                // ⭐ E o RÓTULO segue o id: sem isto, o botao que diz «Add» pode mandar «Subtract».
                assert_eq!(
                    *chave,
                    [
                        "panel.vector.bone.weight.add",
                        "panel.vector.bone.weight.subtract"
                    ][i],
                    "o rotulo do segmento {i} deixou de emparelhar com o id dele"
                );
                assert_eq!(
                    *on,
                    i == publicado,
                    "publicado {publicado}: o segmento {i} devia estar {}",
                    if i == publicado { "ACESO" } else { "apagado" }
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "section_rotulo_tests.rs"]
mod rotulo_do_auto_tests;

/// ⭐ Os gates dos DOIS MODOS do pincel de peso (F29) — irmãos por assunto.
#[cfg(test)]
#[path = "section_modo_do_peso_tests.rs"]
mod modo_do_peso_tests;
