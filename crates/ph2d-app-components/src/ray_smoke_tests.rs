//! Os gates da cena do OLHO — ver o cabeçalho de [`super`].
//!
//! ⚠️ **Eles medem a CENA como DADOS e a LEI pela porta do produto** (a ponte de física), e um
//! deles mede a **GEOMETRIA**: *o dono tem região utilizável?* — a régua que a cena `=45` da
//! escultura pagou com um report (*«resultado bem bizarro»*) sobre uma cena em que **nenhuma**
//! posição do ecrã produzia o efeito.
//!
//! ⛔ O que nenhum deles mede é o CLIQUE: o gesto real é do dono, e o `PH2D_RAY_SMOKE=1` é onde ele
//! vive.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn mundo() -> SimWorld {
    let mut sim = SimWorld::new();
    let _ = montar(sim.world_mut(), 1);
    sim
}

/// A entidade com este nome.
fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("«{nome}» nao esta' na cena"))
}

/// O `RaySensor` de um olho.
fn sensor(sim: &mut SimWorld, nome: &str) -> RaySensor {
    let e = por_nome(sim, nome);
    *sim.world()
        .get::<RaySensor>(e)
        .unwrap_or_else(|| panic!("«{nome}» perdeu o componente"))
}

/// ⭐⭐⭐ **Os dois olhos diferem na DIRECÇÃO, e em mais nada que decida.**
///
/// ⚠️ **As duas metades:** tudo o resto é IGUAL (senão a cena demonstrava outra coisa) e o `dir` é
/// **oposto** (senão não demonstrava nada). ⭐ E a terceira, que é a que quase escapou: os dois
/// olhos têm de estar **dentro da altura da caixa** — um deles acima dela e a diferença passava a
/// ser *a posição*, com a direcção a apanhar boleia.
///
/// **Mutações que devem sangrar:** dar ao controlo o MESMO `dir` · dar-lhe outro `reach` · pôr um
/// dos olhos fora da altura da caixa.
#[test]
fn os_dois_olhos_diferem_so_na_direccao() {
    let mut sim = mundo();
    let a = sensor(&mut sim, "Olho que ve");
    let b = sensor(&mut sim, "Olho que olha para tras");
    assert_eq!(a.reach, b.reach, "os alcances divergiram");
    assert_eq!(a.origin, b.origin, "as origens divergiram");
    assert_eq!(a.layer, b.layer, "as camadas divergiram");
    assert_eq!(
        a.dir,
        Vec2::new(-b.dir.x, -b.dir.y),
        "o controlo tem de olhar exactamente ao CONTRARIO: {:?} contra {:?}",
        a.dir,
        b.dir
    );

    // ⭐ E os dois dentro da altura da caixa — senão o silêncio do controlo tem duas causas.
    for nome in ["Olho que ve", "Olho que olha para tras"] {
        let e = por_nome(&mut sim, nome);
        let y = sim.world().get::<Transform>(e).expect("pose").translation.y;
        assert!(
            y.abs() < CAIXA_MEIA[1],
            "«{nome}» esta' em y = {y}, fora da altura da caixa (±{}) — o controlo passaria a \
             calar-se por POSICAO e nao por direccao",
            CAIXA_MEIA[1]
        );
    }
}

/// ⭐⭐⭐ **A GEOMETRIA da cena: a caixa começa FORA do alcance e acaba DENTRO.**
///
/// ⚠️ **É a régua que a cena `=45` da escultura pagou com um report do dono:** ali `d/R` lia
/// `5,70`–`7,85` em todo o ecrã e **nenhuma** posição produzia o efeito — a cena estava certa como
/// dados e era impossível como gesto. Aqui a pergunta é a mesma, nas duas pontas:
///
/// - no princípio a caixa tem de estar **além** da ponta da linha, senão o passo (2) mente;
/// - no fim tem de estar **dentro**, com folga, senão o passo (4) nunca acontece.
///
/// **Mutações que devem sangrar:** fazer a caixa nascer dentro do alcance · encurtar a viagem.
#[test]
fn a_caixa_comeca_fora_do_alcance_e_acaba_dentro() {
    let ponta = X_POSTE + ALCANCE;
    // A face que o raio encontra primeiro é a ESQUERDA da caixa.
    let face_no_inicio = X_CAIXA - CAIXA_MEIA[0];
    let face_no_fim = X_CAIXA - VIAGEM - CAIXA_MEIA[0];
    assert!(
        face_no_inicio > ponta + 1.0,
        "a caixa nasce a {face_no_inicio} e a linha acaba em {ponta} — o passo (2) precisa de a \
         ver FORA, e com folga para o dono reparar"
    );
    assert!(
        face_no_fim < ponta - 1.0,
        "a caixa pa'ra com a face em {face_no_fim} e a linha acaba em {ponta} — ela tinha de \
         entrar bem dentro"
    );
    assert!(
        face_no_fim > X_POSTE + 1.0,
        "a caixa pa'ra em cima do poste ({face_no_fim} contra {X_POSTE}) — o tracinho do acerto \
         ficaria colado a` origem e nao se leria"
    );
    // ⭐ E a viagem tem de DURAR: o tracinho a deslizar é o passo (3), e ele precisa de segundos.
    let segundos = VIAGEM / RAPIDEZ;
    assert!(
        (1.5..8.0).contains(&segundos),
        "a caixa leva {segundos} s a chegar — depressa demais nao se ve', devagar demais cansa"
    );
}

/// ⭐⭐⭐ **A CENA INTEIRA CABE NA JANELA ÚTIL — e o CONTROLO é a peça que o exige.**
///
/// ⚠️ **A 1.ª redacção REPROVAVA aqui, e quem o descobriu foi a FOTO:** com o poste em `x = −6`, a
/// luz do controlo ficava em `−7,6` — **atrás da Hierarquia**. O passo (5) do roteiro afirma *«a luz
/// da ESQUERDA nunca acende»*, e uma luz que o dono não vê não distingue *«não acendeu»* de *«não
/// existe*». *Um controlo que não se vê não é um controlo.*
///
/// ⛔ A régua é a [`JANELA_UTIL`], medida e escrita ao lado da sua própria constante.
///
/// **Mutações que devem sangrar:** devolver o poste a `−6` · afastar uma luz · pôr a caixa a nascer
/// fora do ecrã.
#[test]
fn a_cena_inteira_cabe_na_janela_util() {
    let (x0, x1, y0, y1) = JANELA_UTIL;
    let mut sim = mundo();
    // ⭐ **Cada peça com a MEIA-LARGURA dela**, e não só o centro: uma luz cujo centro cabe e cuja
    // metade esquerda não, lê-se cortada.
    let meia_luz = LUZ_LADO * 1.4 / 2.0;
    let pecas: [(&str, f32, f32); 8] = [
        ("Poste", 0.25, 0.9),
        ("Luz da frente", LUZ_LADO / 2.0, LUZ_LADO / 2.0),
        ("Luz de tras", LUZ_LADO / 2.0, LUZ_LADO / 2.0),
        // ⭐ Os SUPORTES são maiores que as luzes, logo são eles que decidem a borda.
        ("Suporte: Luz da frente", meia_luz, meia_luz),
        ("Suporte: Luz de tras", meia_luz, meia_luz),
        ("Olho que ve", 0.0, 0.0),
        ("Olho que olha para tras", 0.0, 0.0),
        ("Caixa", CAIXA_MEIA[0], CAIXA_MEIA[1]),
    ];
    for (nome, mx, my) in pecas {
        let e = por_nome(&mut sim, nome);
        let t = *sim.world().get::<Transform>(e).expect("pose");
        let (x, y) = (t.translation.x, t.translation.y);
        assert!(
            x - mx >= x0 && x + mx <= x1,
            "«{nome}» esta' em x = {x} (±{mx}) e a janela util e' [{x0}; {x1}] — o dono nao o ve'"
        );
        assert!(
            y - my >= y0 && y + my <= y1,
            "«{nome}» esta' em y = {y} (±{my}) e a janela util e' [{y0}; {y1}] — o dono nao o ve'"
        );
    }
    // ⭐ E as DUAS pontas do que a caixa percorre, que é o que o passo (2) e o (3) mostram.
    for (quando, x) in [("nasce", X_CAIXA), ("pa'ra", X_CAIXA - VIAGEM)] {
        assert!(
            x - CAIXA_MEIA[0] >= x0 && x + CAIXA_MEIA[0] <= x1,
            "a caixa {quando} em x = {x} e sai da janela util [{x0}; {x1}]"
        );
    }
    // ⛔ E a PONTA da linha do olho da frente, que é o que o passo (2) manda comparar com a caixa.
    let ponta = X_POSTE + ALCANCE;
    assert!(
        ponta <= x1,
        "a ponta da linha cai em x = {ponta}, fora da janela util — o passo (2) manda ver a caixa \
         ALEM dela, e o dono nao ve' onde ela acaba"
    );
}

/// ⭐⭐⭐ **O olho da frente vê a caixa, e o CONTROLO fica calado do princípio ao fim.**
///
/// ⚠️ **Corrido pela porta do produto** (a ponte), tique a tique: um gate que lesse só o fim não
/// distinguiria *«nunca viu»* de *«viu e deixou de ver»* no controlo.
///
/// **Mutações que devem sangrar:** dar ao controlo o mesmo `dir` · parar a caixa antes do alcance.
#[test]
fn o_olho_ve_a_caixa_e_o_controlo_fica_calado() {
    let mut sim = mundo();
    let frente = por_nome(&mut sim, "Olho que ve");
    let tras = por_nome(&mut sim, "Olho que olha para tras");
    let caixa = por_nome(&mut sim, "Caixa");
    let mut bridge = PhysicsBridge::new();

    // 1 s antes de a caixa chegar ao alcance: o da frente ainda não vê nada.
    bridge.dispatch(&mut sim, true, 30);
    assert!(
        bridge.ray_sensor_hits().get(&frente).is_none(),
        "no principio a caixa esta' FORA do alcance — o passo (2) depende disso"
    );

    let mut o_controlo_falou = false;
    let mut distancias = Vec::new();
    for t in 31..=260u64 {
        bridge.dispatch(&mut sim, true, t);
        o_controlo_falou |= bridge.ray_sensor_hits().contains_key(&tras);
        if let Some(h) = bridge.ray_sensor_hits().get(&frente) {
            assert_eq!(h.body, caixa, "o olho viu outra coisa que nao a caixa");
            distancias.push(h.distance);
        }
    }

    assert!(
        !o_controlo_falou,
        "o CONTROLO viu alguma coisa — sem o silencio dele a cena nao prova nada"
    );
    assert!(
        distancias.len() > 30,
        "o olho da frente so' viu a caixa em {} tiques — o passo (4) precisa de tempo para o \
         numero MUDAR a` vista",
        distancias.len()
    );
    // ⭐ E a distância ENCOLHE — é isso que o tracinho a deslizar mostra.
    let (primeira, ultima) = (distancias[0], distancias[distancias.len() - 1]);
    assert!(
        primeira - ultima > 1.0,
        "a leitura tem de ANDAR: comecou em {primeira} e acabou em {ultima}"
    );
    assert!(
        ultima < ALCANCE && ultima > 1.0,
        "no fim a caixa tem de estar parada bem dentro do alcance: {ultima} m de {ALCANCE}"
    );
    // ⛔ E ela FICA: um projéctil autorado é documento, logo o fim do alcance não o apaga.
    assert!(
        sim.world().get::<Transform>(caixa).is_some(),
        "a caixa desapareceu — ela e' DOCUMENTO e o fim do alcance nao a pode tirar da cena"
    );
}

/// ⛔⛔⛔ **NENHUM olho tem `Sprite` — e este gate nasceu a afirmar o CONTRÁRIO.**
///
/// A 1.ª redacção chamava-se `cada_olho_tem_corpo_e_os_dois_nao_se_sobrepoem` e exigia o sprite,
/// pelo argumento certo do `#15` (*«o que tem cérebro tem CORPO»*, senão o dedo do dono não lhe
/// chega). ⚠️ **A FOTO da cena matou a premissa (19/09):** um `Sprite` traz ao Inspector as secções
/// **RENDER SOURCE**, **COLOR & TINT** e **SPRITE SHEET**, e a secção `Ray Sensor` que o roteiro
/// manda ler caía três ecrãs abaixo da dobra — *um passo que manda procurar uma linha AFIRMA que
/// ela está na tela*.
///
/// ⭐ **A premissa do `#15` não se aplica aqui, e o discriminador é o ROTEIRO:** lá o passo era
/// *carregue na porta*; aqui **nenhum passo pede um clique de canvas** — o olho da frente nasce
/// escolhido e o de baixo tem linha própria na Hierarquia. ⇒ o gate passa a afirmar a ausência, e
/// a metade que o torna honesto é a **segunda**: quem tem de ser visto no canvas — o poste, as duas
/// luzes e a caixa — continua a ter corpo.
///
/// **Mutações que devem sangrar:** devolver o `Sprite` a um olho · tirar o do poste ou o de uma luz.
#[test]
fn os_olhos_nao_tem_sprite_e_o_resto_da_cena_tem() {
    let mut sim = mundo();
    for nome in ["Olho que ve", "Olho que olha para tras"] {
        let e = por_nome(&mut sim, nome);
        assert!(
            sim.world().get::<Sprite>(e).is_none(),
            "«{nome}» ganhou um Sprite — ele enterra a seccao `Ray Sensor` tres ecras abaixo da \
             dobra, e o roteiro manda le^-la"
        );
    }
    // ⭐ O CONTROLO, sem o qual este gate aprovaria uma cena INVISÍVEL: o que o dono tem de ver
    // continua desenhado.
    for nome in [
        "Poste",
        "Luz da frente",
        "Luz de tras",
        "Suporte: Luz da frente",
        "Suporte: Luz de tras",
        "Caixa",
        "Ground",
    ] {
        let e = por_nome(&mut sim, nome);
        assert!(
            sim.world().get::<Sprite>(e).is_some(),
            "«{nome}» perdeu o corpo — a cena passa a ser duas linhas sobre o nada"
        );
    }
}

/// ⭐⭐ **As duas luzes ouvem nomes DIFERENTES, e cada uma acende e apaga com o seu par.**
///
/// ⚠️ Com o mesmo nome as duas acendiam juntas — um sinal é um **nome global**, a lição que a cena
/// do golpe já pagou.
///
/// **Mutações que devem sangrar:** dar o mesmo nome às duas · trocar o `Show` por `Hide`.
#[test]
fn as_duas_luzes_ouvem_nomes_diferentes() {
    let mut sim = mundo();
    let mut tabelas = Vec::new();
    for nome in ["Luz da frente", "Luz de tras"] {
        let e = por_nome(&mut sim, nome);
        let t = sim
            .world()
            .get::<SignalActions>(e)
            .unwrap_or_else(|| panic!("«{nome}» perdeu a tabela"))
            .clone();
        assert_eq!(t.0.len(), 2, "«{nome}»: acender e apagar sao DUAS linhas");
        assert_eq!(t.0[0].verb, SignalVerb::Show, "«{nome}»: a 1.ª acende");
        assert_eq!(t.0[1].verb, SignalVerb::Hide, "«{nome}»: a 2.ª apaga");
        assert_ne!(
            t.0[0].on, t.0[1].on,
            "«{nome}» acende e apaga com o MESMO nome"
        );
        // ⛔ E ela nasce APAGADA, senão o passo (3) não tem o que mostrar.
        assert!(
            sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden),
            "«{nome}» nasce acesa — o passo (3) e' ela a ACENDER"
        );
        // ⭐⭐⭐ **E o SUPORTE dela nasce VISÍVEL, no MESMO sítio** — sem ele, *«nao acendeu»* e
        // *«nao ha' luz nenhuma deste lado»* dao ao dono o mesmo ecra, e o CONTROLO deixa de
        // provar o que quer que seja (medido pela foto de 19/09).
        let sup = por_nome(&mut sim, &format!("Suporte: {nome}"));
        assert!(
            sim.world()
                .get::<Visibility>(sup)
                .is_some_and(|v| !v.hidden),
            "o suporte de «{nome}» nasce escondido — uma luz apagada tem de SE VER apagada"
        );
        let (a, b) = (
            sim.world().get::<Transform>(e).expect("pose").translation,
            sim.world().get::<Transform>(sup).expect("pose").translation,
        );
        assert!(
            (a.x - b.x).abs() < 1.0e-6 && (a.y - b.y).abs() < 1.0e-6,
            "o suporte de «{nome}» nao esta' debaixo dela: {a:?} contra {b:?}"
        );
        tabelas.push(t);
    }
    for linha_a in &tabelas[0].0 {
        assert!(
            !tabelas[1].0.iter().any(|b| b.on == linha_a.on),
            "as duas luzes ouvem «{}» — elas acenderiam JUNTAS e o controlo deixava de o ser",
            linha_a.on
        );
    }
}

/// ⭐⭐⭐ **O ROTEIRO NOMEIA RÓTULOS QUE EXISTEM — a lei que o `#15` e o tutorial dele pagaram.**
///
/// Um passo que manda o dono procurar `Ray Sensor` no painel **AFIRMA que aquele texto está na
/// tela**, e ele aprova o smoke com a afirmação dentro. ⚠️ *Um rótulo renomeado noutra linha deixa o
/// roteiro a mandar procurar uma coisa que já não existe, e nenhum outro gate o vê.*
///
/// ⛔⛔ **A âncora é a ASPA e não o `[ray-smoke]` nu** — a primeira ocorrência disso no ficheiro é o
/// comentário de módulo, e uma régua ancorada nele lê a PROSA à volta do roteiro, que cita os mesmos
/// rótulos. Foi uma mutação SOBREVIVENTE na cena do golpe que o expôs (19/09).
///
/// **Mutações que devem sangrar:** traduzir `Reach` no roteiro · apagar a chave `ray.direction` ·
/// tirar o passo do `Sees nothing right now.`.
#[test]
fn o_roteiro_nomeia_rotulos_que_existem() {
    const FONTE: &str = include_str!("ray_smoke.rs");
    let i = FONTE
        .find("\"[ray-smoke]")
        .expect("o literal do roteiro existe");
    let fim = FONTE[i..].find("\n    );").expect("o println fecha");
    let roteiro = &FONTE[i..i + fim];
    // ⭐ **A leitura viva entra pelo PREFIXO derivado**, e não escrito à mão: o rótulo dela é
    // `"Sees {name}, at {dist} m"`, logo o que cabe num roteiro é o pedaço antes do primeiro
    // marcador. *Copiá-lo à mão daria a segunda resposta a «como começa esta frase».*
    let viva = ph2d_i18n::tr("panel.inspector.ray.sees_x_at_y_m");
    let prefixo = viva.split('{').next().unwrap_or_default().trim_end();
    let rotulos = [
        ph2d_i18n::tr("panel.inspector.ray.ray_sensor"),
        ph2d_i18n::tr("panel.inspector.ray.direction"),
        ph2d_i18n::tr("panel.inspector.ray.reach_m"),
        ph2d_i18n::tr("panel.inspector.ray.sees_nothing_right_now"),
        prefixo,
    ];
    for r in &rotulos {
        assert!(
            !r.is_empty() && !r.starts_with("panel."),
            "o rotulo resolveu para a CHAVE («{r}») — a entrada de i18n foi apagada"
        );
        assert!(
            roteiro.contains(r),
            "o roteiro da `=1` nao nomeia «{r}» — ou o passo saiu, ou o rotulo mudou de nome \
             noutra linha e o dono vai procurar uma coisa que ja' nao esta' na tela"
        );
    }
    // ⛔ O CONTROLO da régua, em TRÊS metades — a terceira prova que estamos DENTRO do literal: um
    // comentário nunca chega ao terminal do dono.
    assert!(
        roteiro.len() > 600 && roteiro.contains("(7) deu errado se"),
        "a regua deixou de medir o roteiro — ela esta' a ler {} bytes",
        roteiro.len()
    );
    assert!(
        !roteiro.contains("//!") && !roteiro.contains("    // "),
        "a regua saiu do literal e voltou a ler PROSA — os rotulos passariam a ser satisfeitos \
         pelo comentario que EXPLICA o passo, nao pelo passo"
    );
}
