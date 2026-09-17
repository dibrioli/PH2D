//! Os gates da CENA da vigia.
//!
//! ⚠️ **O oráculo é o que a CENA MONTA, não o que o doc dela diz** — cada asserção aqui é uma frase
//! do doc-comment do irmão, medida sobre o mundo que a [`super::montar`] devolve.

use ph2d_ecs::{
    Compare, Counter, CounterWatch, Name, SignalActions, SignalVerb, SimWorld, Timers, Visibility,
};

use super::{CONTROLO, HEROI, VIDAS, montar};

fn cena() -> (SimWorld, ph2d_ecs::Entity, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let (h, c) = montar(sim.world_mut());
    (sim, h, c)
}

/// ⭐⭐⭐ **O HERÓI tem vigia e o CONTROLO não** — a única diferença entre os dois.
///
/// ⚠️ **A premissa deste gate MUDOU e ele foi reescrito com a morte visível:** ele dizia *«a vigia
/// tem UMA regra»*, e a cena passou a ter **três** (uma por luz) quando a foto mostrou que um
/// placar de texto não se desenha. *Quantas* regras é assunto do irmão
/// [`as_tres_regras_estao_em_limiares_diferentes`]; aqui pergunta-se só quem tem e quem não tem.
///
/// **Mutação que deve sangrar:** dar a vigia ao controlo, ou tirá-la ao herói.
#[test]
fn so_o_heroi_tem_a_vigia() {
    let (sim, h, c) = cena();
    let w = sim.world();
    let vigia = w
        .get::<CounterWatch>(h)
        .expect("o heroi tem de ter a vigia");
    assert!(!vigia.0.is_empty());
    for r in &vigia.0 {
        assert_eq!(
            r.compare,
            Compare::AtMost,
            "conta-se PARA BAIXO ate' ao limiar"
        );
    }
    assert!(
        w.get::<CounterWatch>(c).is_none(),
        "o CONTROLO nao pode ter vigia — e' ele que torna a cena legivel"
    );
}

/// ⭐⭐⭐ **Os dois contadores têm NOMES DIFERENTES.**
///
/// ⚠️ Um contador é somado **por nome em toda a cena**: chamar os dois `vidas` daria `6` e nenhum
/// dos dois chegaria a zero — a cena ensinaria que o componente não funciona, que é a espécie que
/// o `CLAUDE.md` §5.0 chama de **pior que uma cena ausente**.
///
/// **Mutação que deve sangrar:** dar o mesmo nome aos dois.
#[test]
fn os_dois_contadores_nao_se_somam() {
    let (sim, h, c) = cena();
    let w = sim.world();
    let ch = w.get::<Counter>(h).expect("contador do heroi");
    let cc = w.get::<Counter>(c).expect("contador do controlo");
    assert_eq!(ch.name, HEROI);
    assert_eq!(cc.name, CONTROLO);
    assert_ne!(
        ch.name, cc.name,
        "contadores homonimos SOMAM — os dois nunca chegariam a zero"
    );
    assert_eq!(ch.start, VIDAS);
    assert_eq!(cc.start, VIDAS);
}

/// ⭐⭐ **Os dois perdem vidas pelo MESMO mecanismo** — relógio que repete + `AddToCounter(-1)`.
///
/// ⚠️ **É esta a metade que faz do controlo um controlo:** se ele não perdesse vidas, a cena não
/// provaria nada (um objecto parado também não desaparece).
#[test]
fn os_dois_perdem_vidas_pelo_mesmo_mecanismo() {
    let (sim, h, c) = cena();
    let w = sim.world();
    for (quem, e) in [("heroi", h), ("controlo", c)] {
        let t = w
            .get::<Timers>(e)
            .unwrap_or_else(|| panic!("{quem} sem relogio"));
        assert_eq!(t.0.len(), 1);
        assert!(t.0[0].repeat, "{quem}: o relogio tem de repetir");
        assert!(t.0[0].autostart, "{quem}: ele tem de arrancar sozinho");
        let a = w
            .get::<SignalActions>(e)
            .unwrap_or_else(|| panic!("{quem} sem accoes"));
        let soma =
            a.0.iter()
                .find(|r| r.verb == SignalVerb::AddToCounter)
                .unwrap_or_else(|| panic!("{quem} nao tira vidas"));
        assert_eq!(
            soma.on, t.0[0].signal,
            "{quem}: o sinal do relogio tem de ser o que tira a vida"
        );
        assert_eq!(soma.arg, "-1");
    }
}

/// ⚠️ **Ninguém nasce escondido** — senão o veredito da auto-conferência seria verdadeiro no
/// quadro zero.
#[test]
fn ninguem_nasce_escondido() {
    let (sim, h, c) = cena();
    let w = sim.world();
    for e in [h, c] {
        assert!(w.get::<Visibility>(e).is_none_or(|v| !v.hidden));
    }
}

/// ⭐⭐ **O «morri» do herói move DOIS verbos: esconder E parar o relógio.**
///
/// ⚠️ **A segunda metade foi medida na cena a correr:** sem o `StopTimer`, o relógio continuava a
/// bater depois da morte e o placar descia para `−1`, `−2`, `−3`… — *um morto que continua a perder
/// vidas ensina o contrário do que a cena diz*.
///
/// **Mutação que deve sangrar:** apagar a linha do `StopTimer`.
#[test]
fn a_morte_do_heroi_para_tambem_o_relogio() {
    let (sim, h, _) = cena();
    let a = sim.world().get::<SignalActions>(h).expect("accoes");
    let sobre_morri: Vec<SignalVerb> =
        a.0.iter()
            .filter(|r| r.on == "morri")
            .map(|r| r.verb)
            .collect();
    assert!(
        sobre_morri.contains(&SignalVerb::Hide),
        "a morte tem de o esconder"
    );
    assert!(
        sobre_morri.contains(&SignalVerb::StopTimer),
        "a morte tem de PARAR o relogio — senao o placar desce para sempre depois dela"
    );
}

/// ⭐⭐⭐ **AS TRÊS LUZES existem, estão VISÍVEIS, e cada uma tem quem a apague.**
///
/// ⛔⛔ **Este gate nasceu de uma FOTO, e a premissa dele MUDOU no caminho.** Ele começou por
/// medir *«cada placar fica junto do objecto que conta»* — porque a 1.ª versão pendurava rótulos
/// de texto numa raiz de HUD com coordenadas de referência e eles aterravam a `−358` no mundo,
/// com os cinco gates verdes. ⚠️ A foto seguinte mostrou que, mesmo no sítio certo, um `UiLabel`
/// **sem um texto autorado por baixo desenha-se como um ANEL VAZIO** — ele troca o que um texto
/// MOSTRA, não cria o texto. ⇒ o placar saiu e as LUZES entraram, e o gate mudou com a premissa.
///
/// ⚠️ **As três metades:** existir · nascer visível (senão o veredito seria verdadeiro no quadro
/// zero) · e ter quem a apague (senão ela é decoração).
///
/// **Mutação que deve sangrar:** apagar uma das três linhas de `apaga` do herói.
#[test]
fn as_tres_luzes_do_heroi_tem_cada_uma_quem_a_apague() {
    let (mut sim, h, _) = cena();
    let alvos: Vec<String> = sim
        .world()
        .get::<SignalActions>(h)
        .expect("accoes")
        .0
        .iter()
        .filter(|r| r.verb == SignalVerb::Hide && !r.target.is_empty())
        .map(|r| r.target.clone())
        .collect();
    let world = sim.world_mut();
    let mut q = world.query::<(&Name, &Visibility)>();
    let nomes: Vec<(String, bool)> = q
        .iter(world)
        .map(|(n, v)| (n.0.clone(), v.hidden))
        .collect();
    for i in 1..=VIDAS {
        let nome = format!("{}{i}", super::HEROI_LUZ);
        let (_, escondida) = nomes
            .iter()
            .find(|(n, _)| *n == nome)
            .unwrap_or_else(|| panic!("a luz «{nome}» nao existe na cena"));
        assert!(!escondida, "«{nome}» nasce escondida");
        assert!(
            alvos.contains(&nome),
            "ninguem apaga «{nome}» — ela seria decoracao"
        );
    }
}

/// ⭐⭐⭐ **As três regras do herói estão em limiares DIFERENTES**, e cobrem `2 · 1 · 0`.
///
/// ⚠️ É isto que a cena mostra sem um número: uma vigia dispara **a um limiar**, e não *«quando o
/// número mudar»*. Três regras iguais apagariam as três luzes de uma vez.
///
/// **Mutação que deve sangrar:** pôr duas regras no mesmo limiar.
#[test]
fn as_tres_regras_estao_em_limiares_diferentes() {
    let (sim, h, _) = cena();
    let v = sim.world().get::<CounterWatch>(h).expect("a vigia");
    assert_eq!(v.0.len(), 3, "sao TRES regras, uma por luz");
    let mut limiares: Vec<i64> = v.0.iter().map(|r| r.value).collect();
    limiares.sort_unstable();
    assert_eq!(
        limiares,
        vec![0, 1, 2],
        "os tres limiares tem de ser distintos e cobrir a descida inteira"
    );
    for r in &v.0 {
        assert_eq!(r.counter, HEROI, "as tres olham o MESMO contador");
        assert!(r.once, "cada luz apaga-se uma vez");
    }
}

/// ⭐⭐ **O CONTROLO tem a MESMA fiação e ninguém diz os nomes que ela espera.**
///
/// ⚠️ É esta a forma certa do controlo: não lhe falta quem apague as luzes — falta quem **DIGA**
/// que é hora de as apagar.
///
/// **Mutação que deve sangrar:** dar ao controlo os mesmos nomes de sinal do herói.
#[test]
fn o_controlo_tem_a_mesma_fiacao_e_ninguem_lhe_fala() {
    let (sim, h, c) = cena();
    let w = sim.world();
    let escutas = |e: ph2d_ecs::Entity| -> Vec<String> {
        w.get::<SignalActions>(e)
            .expect("accoes")
            .0
            .iter()
            .filter(|r| r.verb == SignalVerb::Hide)
            .map(|r| r.on.clone())
            .collect()
    };
    let (dele, doutro) = (escutas(c), escutas(h));
    assert_eq!(
        dele.len(),
        doutro.len(),
        "o controlo tem de ter TANTAS linhas de esconder como o heroi"
    );
    for nome in &dele {
        assert!(
            !doutro.contains(nome),
            "o controlo escuta «{nome}», que o heroi tambem diz — ele desapareceria junto"
        );
    }
    // E o que DIZ aqueles nomes seria uma vigia, que ele não tem.
    assert!(w.get::<CounterWatch>(c).is_none());
}
