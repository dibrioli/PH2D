//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do suplente #22 (`Tween` + presets Fade/Flash)**:
//! *a composição de hoje — `Signal` + `SignalActions` + `Timer` + `SequencePlayer` + a timeline —
//! já exprime «esta propriedade vai de A a B em N segundos»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Nas duas waves anteriores desta linha a pergunta REESCREVEU a entrega: o **#3
//! `SensorZone`** estava fechado por composição, e a **W5 do #21** descobriu que o pintor já
//! existia (*publicar, e não pintar*).
//!
//! ⚠️ **Ela mora AQUI, e não na crate do componente, porque esta é a ÚNICA que vê os dois lados**:
//! o `ph2d-timeline` depende do `ph2d-ecs` (o `Timer`, o `SequencePlayer`, os verbos), do
//! `ph2d-anim` (o motor de *easing*) e — atrás da feature `render` — do `ph2d-render` (o `Sprite`,
//! onde a opacidade e a cor de facto moram). *Uma sonda que só visse um lado mediria o espantalho.*
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME; o que ela decide é *o quê* do
//! componente, não se os números batem uma barra.
//!
//! ```text
//! cargo test -p ph2d-timeline --features render --test it \
//!     mede_o_que_a_composicao_ja_da_ao_tween -- --ignored --nocapture
//! ```
//!
//! ⚠️ **Corra-a COM a feature `render`** — sem ela o bloco (D) desaparece, e é ele que mede o
//! concorrente mais forte. A sonda diz em voz alta quando corre pela metade.
//!
//! # As perguntas, uma por bloco
//!
//! A) **Um sinal sabe dizer «ao longo de N segundos»?** — sem isso não há *tween* disparável.
//! B) **Existe um `0 → 1` contínuo por objecto durante uma corrida, e quem o lê?**
//! C) **Que propriedades a timeline sabe animar?** — é onde o *Fade* e o *Flash* se separam.
//! D) **Um `SequencePlayer` + um container autorado JÁ FAZEM um fade?** (o concorrente a sério)
//! E) **…e ele alcança um objecto que NASCEU na corrida?**
//! F) **O motor de curvas existe, e quem o vê?**

use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_timeline::PropKind;
// ⛔⛔ **INTEGRAÇÃO (20/09): as importações do bloco D/E vivem sob o MESMO `cfg` que os usos
//    delas, e sem isso esta crate não compila SOZINHA.** Os blocos que as consomem estão atrás de
//    `#[cfg(feature = "render")]`; com a feature ligada (que é o que toda irmã faz na workspace) o
//    ficheiro compila limpo, e a corrida que o `ship.sh` faz crate-a-crate — sem as features que os
//    vizinhos ligam — lê-as como `unused_imports`, que o `build.warnings` nega. ⇒ é a armadilha
//    *«uma feature não viaja com o código»* do HOWTO, aqui do lado do `use`: *um `use` não gateado
//    é uma dependência declarada por quem não a usa.*
#[cfg(feature = "render")]
use ph2d_anim::{AnimValue, Interp, RationalTime};
#[cfg(feature = "render")]
use ph2d_ecs::{Name, SimWorld, Transform};
#[cfg(feature = "render")]
use ph2d_timeline::{StackHost, StripSource, TimelineState, apply_container};

/// O `ph2d-anim` ao alcance de quem escreve o componente? — o manifesto responde, e ele é lido
/// em tempo de COMPILAÇÃO, logo não pode envelhecer sem o ficheiro mudar.
const MANIFESTO_ECS: &str = include_str!("../../../ph2d-ecs/Cargo.toml");
/// Idem para a família dos componentes, que é onde as pontes das waves #11..#21 moram.
const MANIFESTO_FAMILIA: &str = include_str!("../../../ph2d-app-components/Cargo.toml");

/// ⛔⛔ **Esta função existe porque a 1.ª redacção da sonda MENTIU, e a mentira foi a favor do
/// desenho que eu já tinha na cabeça.** Ela perguntava `manifesto.contains("ph2d-anim")` e leu
/// **`true`** para o `ph2d-ecs` — mas a ocorrência é um **COMENTÁRIO** a descrever de que é feita
/// a `ph2d-morph-machine` (*«É uma FOLHA (serde + ph2d-spring + ph2d-anim…)»*), e não uma
/// dependência. *Um censo textual que não separa prosa de código mede a prosa* — a família de
/// defeitos que este repo tem escrita, agora paga por mim, dentro de uma sonda cujo trabalho é
/// exactamente não deixar uma premissa passar por medida.
///
/// ⇒ a régua passa a ler só a secção `[dependencies]`, **sem as linhas de comentário**, e a
/// exigir a FORMA de uma dependência (`nome = …` ou `nome.workspace = …`).
fn declara_dependencia(manifesto: &str, crate_: &str) -> bool {
    let mut dentro = false;
    for linha in manifesto.lines() {
        let l = linha.trim();
        if l.starts_with('[') {
            dentro = l == "[dependencies]";
            continue;
        }
        if !dentro || l.starts_with('#') {
            continue;
        }
        let chave = l.split(['=', '.']).next().unwrap_or("").trim();
        if chave == crate_ {
            return true;
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// A) Um sinal sabe dizer «ao longo de N segundos»?
// ─────────────────────────────────────────────────────────────────────────────

/// O que cada verbo faz com o `arg` — a única porta por onde um número entra numa acção.
fn censo_dos_verbos() -> Vec<(&'static str, bool)> {
    ph2d_ecs::SignalVerb::ALL
        .iter()
        .map(|v| (v.label(), v.uses_arg()))
        .collect()
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn a_o_sinal_sabe_dizer_ao_longo_de_n_segundos() {
    println!("\n══════ (A) o que um SINAL sabe mandar fazer ══════");
    let verbos = censo_dos_verbos();
    for (label, usa_arg) in &verbos {
        println!("  {label:<20} lê o argumento: {usa_arg}");
    }
    println!("  ⇒ {} verbos", verbos.len());

    // Os três que leem o `arg` leem-no como NOME DE RELÓGIO ou como DELTA DE CONTADOR — nunca
    // como uma duração. A prova é o produto: pôr `"0.5"` num `Show` não muda nada, porque o
    // `Show` nem lhe toca.
    let com_arg: Vec<&str> = verbos.iter().filter(|(_, u)| *u).map(|(l, _)| *l).collect();
    println!("  os que leem o argumento: {com_arg:?}");
    assert_eq!(
        com_arg,
        vec!["Start Timer", "Stop Timer", "Add to Counter"],
        "a sonda tem de ser recontada: a lista dos verbos com argumento mudou"
    );
    println!(
        "  ⛔ NENHUM verbo carrega uma DURAÇÃO — `Show` e `Hide` são um degrau, não uma rampa."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// B) O 0 → 1 contínuo por objecto
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn b_existe_um_zero_a_um_continuo_e_quem_o_le() {
    println!("\n══════ (B) o relógio CONTÍNUO de um objecto ══════");
    let relogio = ph2d_ecs::Timer {
        duration_us: 1_000_000,
        ..Default::default()
    };
    let mut estado = ph2d_ecs::timer::born(&relogio);
    ph2d_ecs::timer::start(&mut estado);

    // Um tique de 1/60 s, dezasseis vezes — o passo fixo da casa.
    let passo_us = 1_000_000 / 60;
    let mut leituras = Vec::new();
    for i in 0..=16 {
        if i > 0 {
            let _ = ph2d_ecs::timer::advance(&relogio, &mut estado, passo_us);
        }
        leituras.push(relogio.progress(&estado));
    }
    println!(
        "  progress() ao longo de 16 tiques: {:.3} {:.3} {:.3} … {:.3}",
        leituras[0], leituras[4], leituras[8], leituras[16]
    );
    assert!(
        leituras[8] > leituras[4] && leituras[4] > leituras[0],
        "o relógio tem de ser contínuo para a sonda dizer alguma coisa"
    );
    println!("  ⭐ existe, é DERIVADO, arranca por sinal e renasce no rebobinar.");

    // E o que SAI dele num tique é discreto — a única coisa que o resto do produto vê.
    let saida = ph2d_ecs::timer::advance(&relogio, &mut estado, 5_000_000);
    println!(
        "  o que um tique PUBLICA: fires = {}, finished = {}  (⛔ dois inteiros, zero fracções)",
        saida.fires, saida.finished
    );
    println!("  ⇒ o 0→1 existe DENTRO do timer e o único canal para fora dele é um EVENTO.");
}

// ─────────────────────────────────────────────────────────────────────────────
// C) Que propriedades a timeline sabe animar
// ─────────────────────────────────────────────────────────────────────────────

/// Todos os canais que o documento sabe endereçar, pela porta que o produto usa.
fn censo_dos_canais() -> Vec<PropKind> {
    (0..64_u64)
        .filter_map(|id| PropKind::from_target(ph2d_anim::AnimTarget::new(id)))
        .collect()
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn c_que_propriedades_a_timeline_sabe_animar() {
    println!("\n══════ (C) os canais que a TIMELINE anima ══════");
    let canais = censo_dos_canais();
    for c in &canais {
        println!("  {c:?}");
    }
    println!("  ⇒ {} canais", canais.len());

    // O lado que EXISTE: a opacidade é um canal de primeira classe (e o ledger já tem a
    // entrada `Driver::SpriteAlpha` para ela).
    assert!(
        canais.contains(&PropKind::Opacity),
        "controlo positivo: sem `Opacity` a sonda não estaria a medir a timeline certa"
    );
    println!("  ⭐ `Opacity` está lá  ⇒ um FADE é exprimível pela timeline.");

    // O lado que NÃO existe: nenhum canal de COR. O `Flash` do report do levantamento
    // (`tint_fill` + `self_tint`) não tem por onde ser animado.
    let cor: Vec<&PropKind> = canais
        .iter()
        .filter(|c| {
            let n = format!("{c:?}").to_ascii_lowercase();
            n.contains("tint") || n.contains("color") || n.contains("colour") || n.contains("fill")
        })
        .collect();
    println!("  canais de COR: {cor:?}");
    assert!(
        cor.is_empty(),
        "a sonda tem de ser recontada: apareceu um canal de cor na timeline"
    );
    println!(
        "  ⛔ ZERO canais de cor ⇒ o FLASH (`tint_fill` + `self_tint`) é INEXPRIMÍVEL por aqui."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// D) e E) O concorrente a sério: `SequencePlayer` + container autorado
// ─────────────────────────────────────────────────────────────────────────────

/// Uma cena com **dois** sprites iguais: o `Autorado`, que o container conhece, e o `Nascido`,
/// que é o que uma fábrica poria na cena durante a corrida.
///
/// O container `Fade` tem uma rampa de opacidade `1 → 0` sobre meio segundo, ligada **ao
/// autorado**, e uma tira que o toca no interior dele.
#[cfg(feature = "render")]
fn cena_do_fade() -> (SimWorld, TimelineState, u64, u64, usize) {
    let mut sim = SimWorld::new();
    let branco = [1.0, 1.0, 1.0, 1.0];
    let autorado = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Autorado"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], branco),
        ))
        .id()
        .to_bits();
    let nascido = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Nascido"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], branco),
        ))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    for (t, v) in [(0.0_f64, 1.0_f32), (0.5, 0.0)] {
        doc.upsert_key(
            autorado,
            PropKind::Opacity,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    let fade = doc.add_container("Fade".into());
    let host = StackHost::Container(fade);
    doc.add_lane_in(host, "in".into()).unwrap();
    doc.add_strip_to(host, 0, StripSource::Clip(0), 0.0, 0.5)
        .unwrap();
    (sim, st, autorado, nascido, fade)
}

#[cfg(feature = "render")]
fn alfa(sim: &SimWorld, bits: u64) -> f32 {
    sim.world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(bits))
        .unwrap()
        .tint[3]
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
#[cfg(feature = "render")]
fn d_um_sequence_player_ja_faz_um_fade() {
    println!("\n══════ (D) o FADE pela composição que existe ══════");
    let (mut sim, mut st, autorado, _nascido, fade) = cena_do_fade();
    for t in [0.0_f64, 0.125, 0.25, 0.375, 0.5] {
        apply_container(sim.world_mut(), &mut st.doc, fade, t, |_| false);
        println!("  t = {t:.3} s  →  alfa = {:.4}", alfa(&sim, autorado));
    }
    apply_container(sim.world_mut(), &mut st.doc, fade, 0.25, |_| false);
    let meio = alfa(&sim, autorado);
    assert!(
        (meio - 0.5).abs() < 1e-3,
        "a meio da rampa a opacidade tinha de ser 0,5 — leu {meio}"
    );
    println!("  ⭐⭐⭐ SIM: o fade JÁ se autora hoje, e a meio ele mede exactamente 0,5.");
    println!(
        "  ⚠️ o PREÇO: autorar um container com nome, keyar duas vezes, anexar um `Timer` e um"
    );
    println!("     `SequencePlayer`, e arrancá-lo por `Start Timer`.");
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
#[cfg(feature = "render")]
fn e_e_ele_alcanca_um_objecto_nascido_na_corrida() {
    println!("\n══════ (E) …e um objecto NASCIDO na corrida? ══════");
    let (mut sim, mut st, autorado, nascido, fade) = cena_do_fade();
    apply_container(sim.world_mut(), &mut st.doc, fade, 0.25, |_| false);
    let (a, n) = (alfa(&sim, autorado), alfa(&sim, nascido));
    println!("  autorado (o container conhece-o): alfa = {a:.4}");
    println!("  nascido  (cópia da corrida):      alfa = {n:.4}");
    println!(
        "  ligações do documento: {} — cada uma NOMEIA uma entidade",
        st.doc.bindings().len()
    );
    assert!(
        (a - 0.5).abs() < 1e-3,
        "controlo positivo: o autorado desvanece"
    );
    assert!(
        (n - 1.0).abs() < 1e-6,
        "a sonda tem de ser recontada: o nascido desvaneceu sem ligação nenhuma"
    );
    println!(
        "  ⛔ NÃO: uma ligação é AUTORADA e nomeia UM objecto. Vinte inimigos iguais precisariam"
    );
    println!("     de vinte ligações, e uma cópia que nasce numa corrida não tem nenhuma.");
}

// ─────────────────────────────────────────────────────────────────────────────
// F) O motor de curvas
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn f_o_motor_de_curvas_existe_e_quem_o_ve() {
    println!("\n══════ (F) o motor de CURVAS ══════");
    let mut total = 0_usize;
    let mut deterministas = 0_usize;
    for f in EasingFamily::ALL {
        for m in EasingMode::ALL {
            let e = Easing::new(f, m);
            total += 1;
            if e.is_deterministic() {
                deterministas += 1;
            }
        }
    }
    println!(
        "  {} famílias × {} modos = {total} curvas, {deterministas} delas SEM transcendentais",
        EasingFamily::ALL.len(),
        EasingMode::ALL.len()
    );
    let amostra = [
        EasingFamily::Linear,
        EasingFamily::Cubic,
        EasingFamily::Back,
        EasingFamily::Elastic,
    ];
    for f in amostra {
        let e = Easing::new(f, EasingMode::Out);
        println!(
            "  {:<8} Out: u=0,25 → {:.4}   u=0,50 → {:.4}   u=0,75 → {:.4}   determinista: {}",
            // ⚠️ `{:?}` e não a chave: a `line/UIUX` trocou `label()` por `label_key()`, que
            //    devolve `anim.easing.family.linear` — 28 caracteres numa coluna de 8. Esta é uma
            //    SONDA que imprime uma tabela, e o nome do variante é o que ela quer dizer.
            //    ⛔ Acrescentar a `ph2d-i18n` a esta crate por causa de um `println!` seria pagar
            //    uma aresta de dependência por uma linha de diagnóstico.
            format!("{f:?}"),
            e.eval(0.25),
            e.eval(0.50),
            e.eval(0.75),
            e.is_deterministic()
        );
    }
    println!("  ⭐ o motor está PAGO — e `is_deterministic` já separa o que o passo fixo aceita.");

    // …e quem NÃO o vê. É esta linha que decide onde a lei do componente pode morar.
    let ecs_ve = declara_dependencia(MANIFESTO_ECS, "ph2d-anim");
    let familia_ve = declara_dependencia(MANIFESTO_FAMILIA, "ph2d-anim");
    println!("  `ph2d-ecs` vê o `ph2d-anim`?            {ecs_ve}");
    println!("  `ph2d-app-components` vê o `ph2d-anim`? {familia_ve}");
    // Controlo positivo da régua: sem ele, uma leitura partida devolveria `false` a tudo e
    // a sonda leria-se como confirmada. O `ph2d-core` é dependência das duas, por inspecção.
    assert!(
        declara_dependencia(MANIFESTO_ECS, "ph2d-core")
            && declara_dependencia(MANIFESTO_FAMILIA, "ph2d-core"),
        "controlo positivo: a régua de manifesto não está a ler a secção `[dependencies]`"
    );
    assert!(
        !ecs_ve && !familia_ve,
        "a sonda tem de ser recontada: alguém já ligou o motor de curvas a estas crates"
    );
    println!("  ⛔ nenhuma das duas o vê hoje ⇒ a aresta é uma DECISÃO desta wave, não um acaso.");
}

/// ⚠️ **Sem a feature `render` esta sonda corre pela METADE**, e o silêncio leria-se como
/// *«medido»*. Este teste diz em voz alta qual é a metade que falta.
#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
#[cfg(not(feature = "render"))]
fn d_e_e_precisam_da_feature_render() {
    println!(
        "\n⛔ os blocos (D) e (E) não correram: o `Sprite` mora no `ph2d-render` e a feature \
         `render` está desligada. Repita com `--features render`."
    );
}
