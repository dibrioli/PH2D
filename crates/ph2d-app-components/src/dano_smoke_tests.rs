//! Os gates da cena do GOLPE — ver o cabeçalho de [`super`].
//!
//! ⚠️ **Eles medem a CENA como DADOS e a LEI pela porta do produto** (`resolve_signal_actions`).
//! ⛔ O que nenhum deles mede é o CLIQUE: o gesto real é do dono, e o `PH2D_DANO_SMOKE=1` é onde
//! ele vive.

use super::*;
use ph2d_ecs::{Disparo, SimWorld, StableId};
use ph2d_tags::TagTree;

fn mundo() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut tree = TagTree::new();
    let _ = montar(sim.world_mut(), &mut tree, 1);
    sim
}

/// A tabela de uma das duas receitas, pelo nome dela.
fn tabela(sim: &mut SimWorld, nome: &str) -> SignalActions {
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, &SignalActions)>();
    q.iter(w)
        .find(|(n, _)| n.as_str() == nome)
        .map(|(_, t)| t.clone())
        .unwrap_or_else(|| panic!("a receita «{nome}» nao esta' na cena"))
}

/// ⭐⭐⭐ **As duas fileiras diferem numa CERCA, e em mais nada que decida.**
///
/// ⚠️ **As duas metades:** os verbos e os alvos das duas tabelas são IGUAIS (senão a cena
/// demonstrava outra coisa), e o `from` é DIFERENTE (senão não demonstrava nada).
///
/// **Mutações que devem sangrar:** pôr `Myself` nas duas receitas · trocar um `Destroy` por `Hide`
/// · dar o MESMO nome de sinal às duas.
#[test]
fn as_duas_fileiras_diferem_so_na_cerca() {
    let mut sim = mundo();
    let c = tabela(&mut sim, "Alvo (com cerca)");
    let s = tabela(&mut sim, "Alvo (sem cerca)");
    assert_eq!(c.0.len(), 2, "a tabela cercada tem de ter as DUAS linhas");
    assert_eq!(
        s.0.len(),
        c.0.len(),
        "as duas tabelas tem de ter o mesmo n.º"
    );
    for (a, b) in c.0.iter().zip(s.0.iter()) {
        assert_eq!(
            a.verb, b.verb,
            "os verbos divergiram — a cena mede outra coisa"
        );
        assert_eq!(a.target_by, b.target_by, "os alvos divergiram");
        assert_eq!(a.verb, SignalVerb::Destroy, "o verbo tem de ser o que TIRA");
    }
    assert_eq!(
        c.0[0].target_by,
        SignalTarget::Named,
        "o 1.º alvo e' ele mesmo"
    );
    assert_eq!(
        c.0[1].target_by,
        SignalTarget::Other,
        "o 2.º alvo e' a BALA"
    );
    assert!(
        c.0.iter().all(|a| a.from == SignalFrom::Myself),
        "a fileira de cima perdeu a cerca — deixa de ser a metade que funciona"
    );
    assert!(
        s.0.iter().all(|a| a.from == SignalFrom::Anyone),
        "o CONTROLO ganhou a cerca — sem ele a cena nao prova nada"
    );
    // ⛔ E os NOMES dos dois sinais são diferentes, senão as duas experiências misturam-se: a linha
    // sem cerca ouve TUDO o que se chame assim, e um tiro em cima apagaria a fileira de baixo.
    assert_ne!(
        c.0[0].on, s.0[0].on,
        "as duas fileiras ouvem o MESMO nome — um tiro em cima apagaria a de baixo"
    );
}

/// ⭐⭐⭐ **Três cópias da MESMA receita: com a cerca morre uma, sem ela morrem três.**
///
/// ⚠️ **A fixtura clona a tabela da receita para três entidades** — que é exactamente o que a
/// fábrica faz —, e o golpe é publicado por UMA delas, com a bala do outro lado. *Escrever a tabela
/// à mão aqui mediria uma tabela que a cena não tem.*
///
/// ⚠️ **Os efeitos contam-se por ALVO**, e não por linha: o que o dono vê é quantos objectos saem.
///
/// **Mutação que deve sangrar:** o `from` da receita cercada a virar `Anyone`.
#[test]
fn um_golpe_cercado_atinge_um_alvo_e_o_solto_atinge_os_tres() {
    for (nome, esperado) in [("Alvo (com cerca)", 1), ("Alvo (sem cerca)", 3)] {
        let tab = tabela(&mut mundo(), nome);
        let mut w = ph2d_ecs::World::new();
        let copias: Vec<Entity> = (0..3)
            .map(|i| {
                w.spawn((
                    Transform::default(),
                    Name::new(format!("{nome} {i}")),
                    StableId(i + 1),
                    tab.clone(),
                ))
                .id()
            })
            .collect();
        let bala = w
            .spawn((Transform::default(), Name::new("Bala"), StableId(99)))
            .id();

        let golpe = tab.0[0].on.clone();
        let efeitos = ph2d_ecs::resolve_signal_actions(
            &mut w,
            &TagTree::new(),
            &[Disparo {
                nome: &golpe,
                quem: Some(copias[1]),
                outro: Some(bala),
            }],
        );
        let alvos: Vec<Entity> = efeitos
            .iter()
            .filter(|e| e.target != bala)
            .map(|e| e.target)
            .collect();
        assert_eq!(
            alvos.len(),
            esperado,
            "{nome}: {} alvo(s) atingido(s), esperava {esperado}",
            alvos.len()
        );
        if esperado == 1 {
            assert_eq!(alvos[0], copias[1], "morreu quem NAO levou o tiro");
        }
        // ⭐ E a BALA sai nos dois casos — é a 2.ª linha, e é ela que impede o tiro de atravessar.
        assert!(
            efeitos.iter().any(|e| e.target == bala),
            "{nome}: a bala nao foi atingida — ela atravessaria o alvo"
        );
    }
}

/// ⭐⭐ **As duas fábricas apontam às duas receitas, e trazem TRÊS cada uma.**
///
/// ⚠️ **É a costura que nenhuma das outras vê:** com um `master` por resolver (`0`) a cena abre
/// vazia e as duas leis acima continuam verdes — *uma cena certa como dados e impossível como
/// gesto*, o defeito que o `#15` pagou.
///
/// **Mutações que devem sangrar:** apagar o `resolver_receitas` · trocar o `burst`.
#[test]
fn as_duas_fabricas_apontam_as_duas_receitas() {
    let mut sim = mundo();
    let w = sim.world_mut();
    let mut ids = std::collections::BTreeMap::new();
    {
        let mut q = w.query::<(&Name, &StableId)>();
        for (n, s) in q.iter(w) {
            ids.insert(n.as_str().to_string(), s.0);
        }
    }
    let mut q = w.query::<(&Name, &Factory)>();
    let mut vistas = 0;
    for (n, f) in q.iter(w) {
        let receita = match n.as_str() {
            "Fileira de cima" => "Alvo (com cerca)",
            "Fileira de baixo" => "Alvo (sem cerca)",
            "Heroi" => "Bala",
            _ => continue,
        };
        let esperado = *ids.get(receita).expect("a receita tem identidade");
        assert_ne!(esperado, 0, "{receita}: a receita ficou sem identidade");
        assert_eq!(f.master, esperado, "{n:?} nao aponta a «{receita}»");
        if n.as_str() != "Heroi" {
            assert_eq!(f.burst, POR_FILEIRA, "{n:?}: a fileira mudou de tamanho");
        }
        vistas += 1;
    }
    assert_eq!(
        vistas, 3,
        "faltam fabricas na cena (as duas fileiras e a arma)"
    );
}

/// ⭐⭐⭐ **O ROTEIRO NOMEIA RÓTULOS QUE EXISTEM — a lei que o `#15` e o tutorial do `#15` pagaram.**
///
/// Um passo que manda o dono procurar `Signal Actions` no painel **AFIRMA que aquele texto está na
/// tela**, e ele aprova o smoke com a afirmação dentro. ⚠️ *Um rótulo renomeado noutra linha deixa
/// o roteiro a mandar procurar uma coisa que já não existe, e nenhum outro gate o vê.*
///
/// ⭐ **A fonte de cada rótulo é quem o PINTA**, nunca uma cópia: quatro vêm da tabela de `ph2d-i18n`
/// e o quinto vem do próprio motor (`SignalVerb::label`, a fronteira de dados que o HR-15 declara
/// aberta por escrito).
///
/// **Mutações que devem sangrar:** trocar `From Myself` por `Só eu` no roteiro · apagar a chave
/// `panel.inspector.actions.who_hit` · renomear o `Destroy` do motor.
#[test]
fn o_roteiro_nomeia_rotulos_que_existem() {
    const FONTE: &str = include_str!("dano_smoke.rs");
    // ⛔⛔ **A ÂNCORA É A ASPA, e a 1.ª redacção não a tinha** — ela procurava `[dano-smoke]` nu, e
    // a primeira ocorrência disso no ficheiro é o **comentário de módulo da linha 42**: o gate lia
    // `14 201` bytes de prosa em vez dos `~1 200` do roteiro, e o controlo de comprimento passava
    // trivialmente. *Uma régua que mede o ficheiro inteiro aprova um roteiro que não nomeia nada,
    // porque a prosa à volta dele cita os mesmos rótulos* — apanhado por uma mutação que
    // SOBREVIVEU (19/09): tirar o chip `Reset` do passo (5) deixava o gate verde, porque a palavra
    // continuava no comentário que EXPLICA o passo.
    let i = FONTE
        .find("\"[dano-smoke]")
        .expect("o literal do roteiro existe");
    let fim = FONTE[i..].find("\n    );").expect("o println fecha");
    let roteiro = &FONTE[i..i + fim];
    let rotulos = [
        ph2d_i18n::tr("panel.inspector.actions.signal_actions"),
        ph2d_i18n::tr("panel.inspector.actions.from_myself"),
        ph2d_i18n::tr("panel.inspector.actions.from_anyone"),
        ph2d_i18n::tr("panel.inspector.actions.who_hit"),
        SignalVerb::Destroy.label(),
        // ⭐⭐ **Os dois chips do TRANSPORTE que o passo (5) nomeia** — `Reset` e `Play`, pintados
        // com o rótulo por cima (`topbar/cluster_painter.rs`, `TopBarCluster::Play`).
        //
        // ⛔ A 1.ª redacção deste roteiro mandava carregar em `STOP` (que **não é pintado em lado
        // nenhum**) e no `Home` (que é o *frame selection* do editor) — report do dono, 19/09.
        // *Dois passos impossíveis na mesma frase, e nenhuma régua desta cena os via.*
        //
        // ⭐⭐⭐ **E a 2.ª redacção nomeava o `Pause`, que era redundante** — segundo report do dono
        // no mesmo dia (*«por que pause? não deveria ser rewind?»*): o `Reset` **já** pára o
        // relógio (`ph2d_transport::apply` faz `rewind()` **e** `pause()`), logo o passo a mais
        // tornava falsa a frase seguinte — com o relógio parado os alvos **não** voltam. Quem os
        // traz de volta é o `Play`, e é por isso que ele está aqui no lugar do `Pause`.
        ph2d_i18n::tr("chrome.topbar.reset"),
        ph2d_i18n::tr("chrome.topbar.play"),
    ];
    for r in &rotulos {
        assert!(
            !r.is_empty() && !r.starts_with("panel.") && !r.starts_with("chrome."),
            "o rotulo resolveu para a CHAVE («{r}») — a entrada de i18n foi apagada"
        );
        assert!(
            roteiro.contains(r),
            "o roteiro da `=1` nao nomeia «{r}» — ou o passo saiu, ou o rotulo mudou de nome \
             noutra linha e o dono vai procurar uma coisa que ja' nao esta' na tela"
        );
    }
    // ⛔ O CONTROLO da régua, em TRÊS metades — as duas primeiras já existiam e **não chegavam**.
    // A terceira é a que prova que estamos DENTRO do literal: um comentário nunca chega ao terminal
    // do dono, e foi por a região os conter que a mutação do chip `Reset` sobreviveu.
    assert!(
        roteiro.len() > 600 && roteiro.contains("(6) deu errado se"),
        "a regua deixou de medir o roteiro — ela esta' a ler {} bytes",
        roteiro.len()
    );
    assert!(
        !roteiro.contains("//!") && !roteiro.contains("    // "),
        "a regua saiu do literal e voltou a ler PROSA — os rotulos abaixo passariam a ser \
         satisfeitos pelo comentario que EXPLICA o passo, nao pelo passo"
    );
}

/// ⭐⭐⭐ **SÓ UMA BALA ACORDA UM ALVO — a metade da cura que o suicídio da 1.ª redacção pagou.**
///
/// A cena grita `golpe` a partir do ALVO, e a cerca `From Myself` faz o gritante morrer. Sem este
/// filtro, **qualquer** coisa que lhe toque mata-o: dois alvos encostados matavam-se um ao outro no
/// primeiro quadro (medido, com o log a dizê-lo em duas linhas), e o herói a passar por cima de um
/// alvo mata-o também.
///
/// ⚠️ **Ele é medido pela porta que a física de facto corre** (`ph2d_ecs::tags::belongs`, a mesma do
/// `signal_passes`) e **lido do MUNDO**, nunca re-derivado da fixtura — a lição que o gate irmão da
/// `=2` das Tags escreveu depois de uma mutação lhe sobreviver.
///
/// ⭐ **A 3.ª asserção é a que vale:** ela não pergunta pelo herói **pelo nome** — pergunta se
/// existe **algum** corpo com colisor, fora a receita da bala, que passe a cerca. *Uma lista de
/// suspeitos envelhece no dia em que a cena ganhar um corpo novo; um censo não.*
///
/// **Mutações que devem sangrar:** apagar o `SignalTagFilter` da receita · pô-lo a apontar `Posto` ·
/// dar a tag `Bala` a um alvo.
#[test]
fn so_a_bala_passa_a_cerca_do_alvo() {
    let mut sim = SimWorld::new();
    let mut tree = TagTree::new();
    let _ = montar(sim.world_mut(), &mut tree, 1);
    let bala = tree.find("Bala").expect("a cena autora a tag da bala");

    let filtros: Vec<u64> = {
        let w = sim.world_mut();
        let mut q = w.query::<&ph2d_physics_ecs::SignalTagFilter>();
        q.iter(w).map(|f| f.0).collect()
    };
    assert_eq!(
        filtros,
        vec![bala.0; 2],
        "as DUAS receitas de alvo declaram a cerca da BALA"
    );

    let corpos: Vec<(String, bool, bool)> = {
        let w = sim.world_mut();
        let mut q = w.query::<(&Name, &ph2d_physics_ecs::Collider, Option<&Tags>)>();
        q.iter(w)
            .map(|(n, _, t)| {
                let passa = t.is_some_and(|t| ph2d_ecs::tags::belongs(t, &tree, bala));
                (n.as_str().to_string(), t.is_some(), passa)
            })
            .collect()
    };
    let passam: Vec<&String> = corpos
        .iter()
        .filter(|(_, _, p)| *p)
        .map(|(n, _, _)| n)
        .collect();
    assert_eq!(
        passam,
        vec![&"Bala".to_string()],
        "quem acorda um alvo tem de ser SO' a bala — passaram {passam:?} de {} corpos com colisor",
        corpos.len()
    );
    // ⛔ O CONTROLO da régua: sem corpos com colisor na consulta, a asserção acima seria sobre uma
    // lista vazia comparada com uma de um — vermelha —, mas com a tag apagada de TODOS ela ficaria
    // verde por vácuo no dia em que alguém trocasse o `vec![...]` por um `is_empty` invertido.
    assert!(
        corpos.len() >= 4,
        "a cena perdeu corpos com colisor ({}) — a cerca esta' a ser medida sobre quase nada",
        corpos.len()
    );
}

/// ⭐⭐ **DOIS ALVOS NUNCA SE TOCAM — a metade GEOMÉTRICA da mesma cura.**
///
/// A 1.ª redacção espalhava os alvos numa FAIXA (`SpawnAt::Area`) e a semente pôs dois a
/// encostarem-se. Hoje eles nascem em pontos MARCADOS, e o que este gate afirma é a propriedade que
/// torna isso seguro: **todo par de pontos está mais afastado do que um alvo é largo**.
///
/// ⚠️ **Ele mede os PONTOS da cena e o `LADO` do produto** — não a constante `ESPACO`: as duas
/// fileiras partilham o `x` e o que as separa é o `y`, logo uma conta sobre `ESPACO` sozinho não
/// veria o dia em que alguém aproximasse as fileiras.
///
/// **Mutações que devem sangrar:** `ESPACO` a `1.0` · as duas fileiras no mesmo `y` · `LADO` a `3.0`.
#[test]
fn dois_alvos_nunca_se_tocam() {
    let mut sim = SimWorld::new();
    let mut tree = TagTree::new();
    let _ = montar(sim.world_mut(), &mut tree, 1);
    let posto = tree.find("Posto").expect("a cena autora os postos");
    let pontos: Vec<[f32; 2]> = ph2d_ecs::tags::tagged(sim.world(), &tree, posto)
        .into_iter()
        .filter_map(|e| sim.world().get::<Transform>(e))
        .map(|t| [t.translation.x, t.translation.y])
        .collect();
    assert_eq!(
        pontos.len(),
        (POR_FILEIRA as usize) * 2,
        "os postos das duas fileiras: {pontos:?}"
    );
    for (i, a) in pontos.iter().enumerate() {
        for b in &pontos[i + 1..] {
            // ⚠️ **A régua é a de Chebyshev e não a euclidiana**: dois quadrados alinhados aos eixos
            // sobrepõem-se quando as DUAS separações são menores que o lado — a distância em linha
            // recta diria «longe» sobre um par que partilha uma aresta na diagonal.
            let sep = (a[0] - b[0]).abs().max((a[1] - b[1]).abs());
            assert!(
                sep > LADO,
                "os postos {a:?} e {b:?} estao a {sep} um do outro, e um alvo mede {LADO} — eles \
                 tocam-se, gritam o golpe um do outro e matam-se no primeiro quadro"
            );
        }
    }
}
