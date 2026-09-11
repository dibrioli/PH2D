//! **Todo componente de física registrado tem de ser autorável pela UI.**
//!
//! Enio, depois do smoke das oito waves: *"tudo isso está exposto na UI e é possível
//! criar essas cenas todas usando apenas os parâmetros expostos?"*. A resposta era sim —
//! conferida componente a componente — mas **prosa envelhece**, e a nona wave vai ser
//! escrita por alguém que não estava nesta conversa.
//!
//! Um componente que chega ao motor sem chegar à §11 é o **órfão** que a DIRETIVA §2
//! proíbe: ele funciona em toda cena de smoke (que constrói com código) e é inalcançável
//! no produto. É o modo de falha exato que o painel de MUNDO teve no W2b — tudo a
//! jusante funcionando sobre um painel que não existia no build.
//!
//! O gate é **estrutural, sobre o fonte**: para cada nome registrado em
//! `register_physics_components`, alguém no caminho de ESCRITA da UI tem de nomeá-lo.
//! Não prova que o controle está pintado (isso é o `architecture_panel_wiring_parity` e
//! os seams que CLICAM), prova que ele **existe** — e é a metade que faltava.

use std::fs;

/// Os arquivos por onde uma edição do Inspector vira componente. Se um dia houver mais
/// um, ele entra aqui — e o gate falha até entrar, que é o ponto.
///
/// ⚠️ O quarto chegou (W-AreaFalloff): o cap de 600 LOC do shell obrigou a separar as
/// rows de ZONA (*o que esta ÁREA faz a outros corpos*) do resto (*o que ESTE corpo é*),
/// e o gate nasceu VERMELHO nomeando os seis componentes de área — o corte moveu os
/// escritores para fora da lista. Foi a falha ALTA que a lista existe para produzir.
const WRITERS: [&str; 9] = [
    "src/render_loop/inspector_physics_apply.rs",
    "src/render_loop/inspector_physics_area.rs",
    // ⚠️ **O nono chegou pelo mesmo caminho do quarto** (W-Surface): o cap de
    // 600 LOC do shell obrigou a separar *de que esta SUPERFÍCIE é feita* do
    // resto, e o gate nasceu VERMELHO nomeando o `WalkSurface` no instante do
    // corte — o escritor tinha saído da lista sem sair do produto. A falha alta
    // funcionando pela segunda vez.
    "src/render_loop/inspector_physics_surface.rs",
    "src/render_loop/inspector_physics_markers.rs",
    "src/render_loop/inspector_joint.rs",
    // ⚠️ **Nem todo caminho de autoria é uma ROW.** A roldana (W-Pulley W1) é
    // criada por um botão que SPAWNA um objeto e dimensionada por uma ALÇA de
    // canvas — dois gestos que não passam por `apply_physics_edit`, e é por isso
    // que os dois arquivos entram na lista. Um componente cuja única UI é uma
    // alça continua sendo alcançável no produto, que é o que este gate mede.
    "src/render_loop/inspector_joint_wheel.rs",
    "src/joint_anchor_drag.rs",
    // ⚠️ **E nem todo caminho de autoria é um NÚMERO.** O nome do sinal (W-Signal)
    // é uma STRING, então ele não passa pelo `PhysicsFieldEdit` — vai pelo mesmo
    // pipeline canônico de componente que o nome da entidade usa (fila de comandos
    // + registro), que mora aqui. Um braço de string dentro do enum de campos
    // numéricos seria um segundo formato de edição vivendo dentro do primeiro.
    "src/render_loop/inspector_commits.rs",
    // ⚠️ **O oitavo chegou pela porta que este gate existe para vigiar** (W5): o
    // `PlatformPlayer` foi registrado na W2 e a §14 só nasceu três waves depois,
    // então o gate ficou VERMELHO nesse intervalo inteiro e foi ele quem
    // ANTECIPOU a wave de autoria. É a falha alta funcionando: o componente
    // rodava em toda cena de smoke (que constrói com código) e era inalcançável
    // no produto.
    "src/render_loop/inspector_player.rs",
];

#[test]
fn every_registered_physics_component_has_a_ui_writer() {
    let registry = fs::read_to_string("../../crates/ph2d-physics-ecs/src/lib.rs")
        .expect("o registry de física");
    let written: String = WRITERS
        .iter()
        .map(|f| fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}")))
        .collect();

    // Os nomes canônicos exatamente como o registry os cunha — a mesma string que o
    // `queue_set` precisa para achar o `type_id`, então comparar por ela é comparar a
    // coisa que de fato liga os dois lados.
    //
    // ⚠️ **DUAS grafias registram, e a segunda chegou na integração de 2026-08-24.** O
    // descritor de componente (ADR-0164 F0) trouxe o `register_default::<T>`, que
    // acrescenta um construtor à vtable e **não** muda o que fica registrado; 30 das 32
    // chamadas da física passaram a usá-lo, e `MassOverride`/`Dominance` ficaram na forma
    // antiga por não implementarem `Default`.
    //
    // ⛔ **Este gate leu 2 nomes e disparou o `>= 22` abaixo com a própria mensagem que
    // previa o caso** (*"ou o parse quebrou"*) — a rede fez exatamente o que existe para
    // fazer, e a cura é ensinar o parse as duas grafias, **nunca** baixar a barra. Uma
    // terceira grafia futura volta a encolher a contagem e a barra volta a apanhá-la.
    let mut registered: Vec<&str> = registry
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            l.strip_prefix("reg.register::<")
                .or_else(|| l.strip_prefix("reg.register_default::<"))
        })
        .filter_map(|l| l.split('"').nth(1))
        .collect();
    registered.sort_unstable();
    assert!(
        registered.len() >= 22,
        "o registry encolheu ({} nomes) — ou o parse quebrou, e um gate que não lê nada \
         passa sempre",
        registered.len()
    );

    // ⚠️ **Duas formas contam como escrever, e a segunda não é frouxidão.** O
    // caminho comum é `queue_set` com o NOME CANÔNICO (a string que acha o
    // `type_id`), e é ele que a primeira metade procura. Mas um componente
    // autorado por um GESTO — uma alça de canvas que escreve no lugar, um botão
    // que SPAWNA o objeto — nunca nomeia aquela string, porque ela existe para a
    // fila de comandos; ele nomeia o TIPO. Exigir só a string diria *"a roldana é
    // órfã"* sobre um componente que o artista cria com um clique e dimensiona
    // arrastando, que é o oposto do que este gate mede.
    let orphans: Vec<&str> = registered
        .iter()
        .copied()
        .filter(|name| {
            let ty = name.rsplit("::").next().unwrap_or(name);
            !written.contains(name) && !written.contains(ty)
        })
        .collect();
    assert!(
        orphans.is_empty(),
        "componentes de física que NENHUM caminho da UI escreve: {orphans:?}\n\n\
         Um componente sem UI funciona em toda cena de smoke (que constrói com código) e \
         é inalcançável no produto — o órfão que a DIRETIVA §2 proíbe. Ou dê a ele uma \
         row na §11 (e um arm em `apply_physics_edit` / `apply_marker_edit`), ou não o \
         registre ainda."
    );
}
