use super::*;

fn agente() -> InspectorNavAgent {
    InspectorNavAgent {
        alvo_modo: NavAlvoModo::Objecto,
        alvo_nome: "Hero".into(),
        alvo_perdido: false,
        alvo_tag: 0,
        alvo_ponto: [0.0, 0.0],
        radius: 0.0,
        arrive: 0.1,
        repath: 0.5,
        stuck_after: 1.0,
        active: true,
        avoidance: true,
        avoid_harm: true,
        has_health: true,
        on_arrived: String::new(),
        on_no_path: String::new(),
        on_stuck: String::new(),
        has_body: true,
        has_mover: true,
        mover_reads_keys: false,
        has_platformer: false,
        in_region: true,
        agora: None,
    }
}

/// ⚠️ **Todo campo dos dois componentes tem uma edição** — um campo sem variante é um knob que o
/// painel mostra e ninguém pode mexer. ⛔ A lista é escrita à mão de propósito: ela é a SEGUNDA
/// leitura dos componentes, e é a discordância entre as duas que acusa o esquecimento.
#[test]
fn todo_campo_dos_dois_componentes_tem_uma_edicao() {
    let variantes = [
        NavFieldEdit::HalfW(0.0),
        NavFieldEdit::HalfH(0.0),
        NavFieldEdit::ObstacleLayers(0),
        NavFieldEdit::AlvoModo(NavAlvoModo::Nenhum),
        NavFieldEdit::AlvoNome(String::new()),
        NavFieldEdit::AlvoX(0.0),
        NavFieldEdit::AlvoY(0.0),
        NavFieldEdit::Radius(0.0),
        NavFieldEdit::Arrive(0.0),
        NavFieldEdit::Repath(0.0),
        NavFieldEdit::StuckAfter(0.0),
        NavFieldEdit::Active(true),
        NavFieldEdit::OnArrived(String::new()),
        NavFieldEdit::OnNoPath(String::new()),
        NavFieldEdit::OnStuck(String::new()),
        NavFieldEdit::Avoidance(true),
        NavFieldEdit::AlvoTag(0),
        NavFieldEdit::AvoidHarm(true),
        NavFieldEdit::CostAreaCost(1.0),
        NavFieldEdit::CostAreaForbidden(false),
        NavFieldEdit::LinkTo(String::new()),
        NavFieldEdit::LinkTwoWay(false),
        NavFieldEdit::LinkTeleport(true),
        NavFieldEdit::LinkCost(0.0),
        NavFieldEdit::LinkOnCrossed(String::new()),
    ];
    assert_eq!(
        variantes.len(),
        25,
        "a `NavRegion` tem DOIS campos (meias-extensões contam por dois) e a máscara; o `NavAgent` \
         tem o alvo (modo · nome · ponto x/y · tag), quatro números, o interruptor, os dois desvios \
         (W5 outros · W7 dano) e três nomes; (W7) a `NavCostArea` tem custo e proibida, e o \
         `NavLink` tem saída, dois sentidos, teleporte, custo e o sinal — se um nasceu, ele precisa \
         de uma variante aqui e de uma row no painel"
    );
}

/// ⭐⭐⭐ **A ordem das queixas do agente É a lei** — da mais específica para a mais geral.
///
/// ⚠️ Cada passo liga **todas** as queixas mais gerais ao mesmo tempo: é a ORDEM que decide qual
/// sai, e uma fixtura com uma queixa só não a testaria.
///
/// **Mutações que devem sangrar:** trocar dois braços · `AlvoPerdido` antes de `SemAlvo` vazio.
#[test]
fn a_queixa_do_agente_vai_da_mais_especifica_para_a_mais_geral() {
    assert_eq!(
        agente().queixa(),
        None,
        "um agente com tudo no sítio não se queixa"
    );

    let mut tudo = agente();
    tudo.has_body = false;
    tudo.has_mover = false;
    tudo.mover_reads_keys = true;
    tudo.has_platformer = true;
    tudo.active = false;
    tudo.alvo_modo = NavAlvoModo::Nenhum;
    tudo.in_region = false;
    let esperado = [
        AgentQueixa::SemCorpo,
        AgentQueixa::SemMover,
        AgentQueixa::MoverLeTeclado,
        AgentQueixa::ComPlataforma,
        AgentQueixa::Desligado,
        AgentQueixa::SemAlvo,
        AgentQueixa::ForaDaRegiao,
    ];
    for q in esperado {
        assert_eq!(
            tudo.queixa(),
            Some(q),
            "com todas as mais gerais ligadas, sai {q:?}"
        );
        match q {
            AgentQueixa::SemCorpo => tudo.has_body = true,
            AgentQueixa::SemMover => tudo.has_mover = true,
            AgentQueixa::MoverLeTeclado => tudo.mover_reads_keys = false,
            AgentQueixa::ComPlataforma => tudo.has_platformer = false,
            AgentQueixa::Desligado => tudo.active = true,
            AgentQueixa::SemAlvo => tudo.alvo_modo = NavAlvoModo::Ponto,
            AgentQueixa::AlvoPerdido | AgentQueixa::SemForma => unreachable!(),
            AgentQueixa::ForaDaRegiao => tudo.in_region = true,
        }
    }
    assert_eq!(tudo.queixa(), None);
}

/// ⚠️ **O modo `Objecto` tem DUAS formas de não ter alvo**: o nome vazio (ninguém foi escolhido) e
/// o nome PERDIDO (o objecto foi apagado). As curas são diferentes, e a do perdido vem primeiro —
/// um alvo perdido também tem o nome vazio (o painel não pode mostrar o texto de um hash).
#[test]
fn o_alvo_por_nome_tem_duas_formas_de_faltar() {
    let mut vazio = agente();
    vazio.alvo_nome = String::new();
    assert_eq!(vazio.queixa(), Some(AgentQueixa::SemAlvo));

    let mut perdido = agente();
    perdido.alvo_nome = String::new();
    perdido.alvo_perdido = true;
    assert_eq!(perdido.queixa(), Some(AgentQueixa::AlvoPerdido));

    // E o modo PONTO nunca está sem alvo.
    let mut ponto = agente();
    ponto.alvo_modo = NavAlvoModo::Ponto;
    ponto.alvo_nome = String::new();
    assert_eq!(ponto.queixa(), None);
}

/// ⭐ **(W6) A patrulha tem as MESMAS duas faltas do nome** — vazio é «sem alvo», perdido é «nenhuma
/// forma desenhada tem esse nome», e o perdido vem primeiro; **a tag** por escolher é «sem alvo».
///
/// **Mutações que devem sangrar:** tirar qualquer dos três braços novos · trocar `SemForma` por
/// `AlvoPerdido` (a cura é outra: escrever o nome de uma FORMA).
#[test]
fn a_patrulha_e_a_tag_dizem_o_que_lhes_falta() {
    let mut patrulha = agente();
    patrulha.alvo_modo = NavAlvoModo::Patrulha;
    assert_eq!(patrulha.queixa(), None, "uma ronda com forma não se queixa");
    patrulha.alvo_nome = String::new();
    assert_eq!(patrulha.queixa(), Some(AgentQueixa::SemAlvo));
    patrulha.alvo_perdido = true;
    assert_eq!(patrulha.queixa(), Some(AgentQueixa::SemForma));

    let mut tag = agente();
    tag.alvo_modo = NavAlvoModo::Tag;
    assert_eq!(
        tag.queixa(),
        Some(AgentQueixa::SemAlvo),
        "a tag por escolher"
    );
    tag.alvo_tag = 9;
    assert_eq!(tag.queixa(), None);
}

/// A região: o tamanho antes das paredes — uma região sem tamanho não tem paredes a perguntar.
#[test]
fn a_queixa_da_regiao_vai_do_tamanho_as_paredes() {
    let boa = InspectorNavRegion {
        half_w: 4.0,
        half_h: 3.0,
        obstacle_layers: 1,
    };
    assert_eq!(boa.queixa(), None);
    let mut sem = boa;
    sem.half_h = 0.0;
    sem.obstacle_layers = 0;
    assert_eq!(sem.queixa(), Some(RegionQueixa::SemTamanho));
    sem.half_h = 3.0;
    assert_eq!(sem.queixa(), Some(RegionQueixa::SemParedes));
}

/// ⭐ (W7) **A área sem forma queixa-se antes da que tem um corpo que anda** — e a boa cala-se.
///
/// **Mutações que devem sangrar:** trocar a ordem dos dois `if` · esquecer o `body_moves`.
#[test]
fn a_queixa_da_area_vai_da_forma_ao_corpo_que_anda() {
    let boa = InspectorNavCostArea {
        cost: 3.0,
        forbidden: false,
        has_shape: true,
        body_moves: false,
    };
    assert_eq!(boa.queixa(), None);
    let anda = InspectorNavCostArea {
        body_moves: true,
        ..boa
    };
    assert_eq!(anda.queixa(), Some(CostAreaQueixa::CorpoQueAnda));
    let sem = InspectorNavCostArea {
        has_shape: false,
        ..anda
    };
    assert_eq!(sem.queixa(), Some(CostAreaQueixa::SemForma));
}

/// ⭐ (W7) **O atalho tem duas formas de não levar a lado nenhum**, e a perdida vem primeiro.
#[test]
fn o_atalho_sem_saida_e_o_de_saida_perdida() {
    let bom = InspectorNavLink {
        to_nome: "Exit".into(),
        to_perdido: false,
        two_way: false,
        teleport: true,
        cost: 0.0,
        on_crossed: String::new(),
    };
    assert_eq!(bom.queixa(), None);
    let vazio = InspectorNavLink {
        to_nome: String::new(),
        ..bom.clone()
    };
    assert_eq!(vazio.queixa(), Some(LinkQueixa::SemSaida));
    let perdido = InspectorNavLink {
        to_perdido: true,
        ..vazio
    };
    assert_eq!(perdido.queixa(), Some(LinkQueixa::SaidaPerdida));
}

/// ⭐ (W7) ***Avoid Harm* sem `Health` não muda nada** — e desligado também não se queixa.
#[test]
fn evitar_dano_sem_vida_nao_muda_nada() {
    let mut a = agente();
    assert!(
        !a.evitar_dano_nao_muda_nada(),
        "com Health e ligado: faz efeito"
    );
    a.has_health = false;
    assert!(a.evitar_dano_nao_muda_nada());
    a.avoid_harm = false;
    assert!(!a.evitar_dano_nao_muda_nada(), "desligado não promete nada");
}
