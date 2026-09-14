//! ⭐⭐⭐ **O CENSO DAS FILEIRAS DE CHIP** — o painel oferece exactamente o que o
//! motor tem.
//!
//! ⛔⛔ **Este ficheiro existe porque o gate que ele contém era NOMEADO em dois
//! sítios e NÃO EXISTIA.** Os doc-comments de `ph2d_sculpt3d::Falloff::ALL` e de
//! `ids::SCULPT3D_FALLOFF` prometiam, cada um, que
//! `the_panel_offers_every_falloff_the_engine_has` compara os dois arrays e fica
//! vermelho quando uma curva nova não passa pelo painel. Ninguém o tinha
//! escrito. *Uma promessa de gate lê-se exactamente como um gate, e a diferença
//! só aparece no dia em que ele devia sangrar.*
//!
//! ⚠️ **A régua é a CONTAGEM dos dois lados**, e é isso que a torna um censo: um
//! valor novo no `ALL` do motor sem id correspondente reprova, e um id a mais
//! sem valor no motor também — o segundo é o chip que aponta para nada.

use ph2d_sculpt3d::{ClothArea, ClothForceFalloff, ClothMode, Falloff};

/// **GATE — o painel oferece TODA curva de falloff que o motor tem.**
#[test]
fn the_panel_offers_every_falloff_the_engine_has() {
    assert_eq!(
        crate::ids::SCULPT3D_FALLOFF.len(),
        Falloff::ALL.len(),
        "o painel tem {} chips de falloff e o motor tem {} curvas -- uma curva sem \
         id nasce inalcancavel, e um id sem curva e' um chip que aponta para nada",
        crate::ids::SCULPT3D_FALLOFF.len(),
        Falloff::ALL.len()
    );
}

/// **GATE — o painel oferece TODO modo de deformação do tecido.**
///
/// ⚠️ **É o gate que faltava quando o pincel tinha oito comportamentos e um
/// alcançável**: a lei da referência respondia aos oito e nenhum id os
/// desenhava. Um modo novo agora nasce vermelho aqui.
#[test]
fn the_panel_offers_every_cloth_mode_the_engine_has() {
    assert_eq!(
        crate::ids::SCULPT3D_CLOTH_MODE.len(),
        ClothMode::ALL.len(),
        "o painel tem {} chips de deformacao e o motor tem {} modos",
        crate::ids::SCULPT3D_CLOTH_MODE.len(),
        ClothMode::ALL.len()
    );
    assert_eq!(
        crate::ids::SCULPT3D_CLOTH_AREA.len(),
        ClothArea::ALL.len(),
        "o painel tem {} chips de area e o motor tem {}",
        crate::ids::SCULPT3D_CLOTH_AREA.len(),
        ClothArea::ALL.len()
    );
    assert_eq!(
        crate::ids::SCULPT3D_CLOTH_FORCE_FALLOFF.len(),
        ClothForceFalloff::ALL.len(),
        "o painel tem {} chips de forma de queda e o motor tem {}",
        crate::ids::SCULPT3D_CLOTH_FORCE_FALLOFF.len(),
        ClothForceFalloff::ALL.len()
    );
}

/// **GATE — o painel oferece TODO motor de retopologia que o botão tem.**
///
/// ⛔ **O doc de `ids::SCULPT3D_RETOPO_MODE` NOMEAVA este gate e ele NÃO
/// EXISTIA** — medido 2026-09-13: o histórico do git não tem um commit que o
/// tenha escrito, com o controlo positivo (um gate vizinho) a devolver o dele. É
/// a mesma forma que fez nascer este ficheiro, pela terceira vez. E o motor que
/// ele guarda já pagou o defeito uma vez: o `Local` passou a wave inteira do
/// pivô alcançável só por uma variável de ambiente.
#[test]
fn the_panel_offers_every_retopo_mode_the_engine_has() {
    assert_eq!(
        crate::ids::SCULPT3D_RETOPO_MODE.len(),
        crate::state::RetopoMode::ALL.len(),
        "o painel tem {} chips de motor de retopologia e o botao tem {} motores -- um \
         motor sem id nasce inalcancavel, e um id sem motor e' um chip que aponta para nada",
        crate::ids::SCULPT3D_RETOPO_MODE.len(),
        crate::state::RetopoMode::ALL.len()
    );
}

/// **GATE — cada rótulo é distinto, e nenhum é vazio.**
///
/// ⚠️ Dois chips com o mesmo texto são um controlo que o artista não consegue
/// escolher, e um chip vazio é um botão sem nome. As duas coisas passam por
/// qualquer censo de CONTAGEM.
#[test]
fn no_two_chips_of_a_row_carry_the_same_label() {
    let modos: Vec<&str> = ClothMode::ALL.iter().map(|m| m.label()).collect();
    let areas: Vec<&str> = ClothArea::ALL.iter().map(|a| a.label()).collect();
    let quedas: Vec<&str> = ClothForceFalloff::ALL.iter().map(|f| f.label()).collect();
    for (nome, rotulos) in [
        ("deformacao", &modos),
        ("area", &areas),
        ("forma de queda", &quedas),
    ] {
        for (i, a) in rotulos.iter().enumerate() {
            assert!(!a.trim().is_empty(), "{nome}: o chip {i} nao tem rotulo");
            for b in rotulos.iter().skip(i + 1) {
                assert_ne!(a, b, "{nome}: dois chips dizem {a:?}");
            }
        }
    }
}

/// **GATE — o modo decide o braço do gesto, e os DOIS lados existem.**
///
/// A shell escolhe entre re-apanhar o cursor na superfície e andar no plano de
/// profundidade perguntando ao modo (espec §4.3). ⚠️ **O anti-vácuo é metade do
/// gate:** se a pergunta devolvesse sempre o mesmo valor, a escolha da shell
/// seria uma constante disfarçada de decisão.
#[test]
fn the_cloth_mode_decides_which_arm_the_gesture_takes() {
    let repicam = ClothMode::ALL.iter().filter(|m| m.repica()).count();
    assert!(
        repicam > 0 && repicam < ClothMode::ALL.len(),
        "{repicam} de {} modos re-apanham o cursor -- a pergunta e' uma constante",
        ClothMode::ALL.len()
    );
    assert!(
        !ClothMode::Grab.repica() && !ClothMode::SnakeHook.repica(),
        "os dois modos de ANCORA nao re-apanham o cursor (espec §4.3)"
    );
    assert!(ClothMode::Drag.repica(), "os modos de FORCA re-apanham");
}

/// ⭐⭐⭐ **OS NÚMEROS DO FILTRO APARECEM COM O FILTRO, NÃO COM O PINCEL.**
///
/// ⛔⛔ **É o defeito que a pergunta do dono expôs** (2026-09-08): os números do
/// tecido só eram pintados com `verb == Verb::Cloth`, e desde a W9b o **filtro
/// corre com qualquer verbo na mão** (a lei deixou de ser derivada do verbo).
/// ⇒ com o Draw na mão eles mexiam na simulação **sem nada na tela os mostrar**
/// — *vivo e inalcançável*, que é o espelho do knob morto e a espécie que
/// nenhuma sonda deste repo vê.
///
/// ⚠️ **As duas metades são o gate:** só a primeira deixaria passar uma fileira
/// pintada sempre (ruído sobre quem esculpe); só a segunda deixaria passar a
/// pergunta antiga.
///
/// ⚠️ **E a lista é por NOME**: uma contagem literal obrigaria toda wave futura a
/// editar este número, e diria *«são 5 e não 6»* sem dizer qual.
#[test]
fn os_numeros_do_filtro_aparecem_com_o_filtro_e_nao_com_o_pincel() {
    use crate::rows::rows;
    use crate::state::Sculpt3dUi;
    use ph2d_sculpt3d::{ClothFilterKind, FilterKind, FilterLaw, Verb};

    // ⚠️⚠️ **A lista é NOMEADA, e a 1.ª redacção contava `4` à mão.** Uma contagem
    // literal faz cada número novo editar o teste de outra pessoa e não diz QUAL
    // falta; com os nomes, quem acrescentar um número do filtro tem de o
    // declarar aqui — e a mensagem diz-lhe exactamente o que fazer.
    const ESPERADOS: [&str; 8] = [
        // ⭐ O único deles que o alvo também tem (censo do painel dele, 09/09).
        "panel.sculpt3d.cfilter_strength",
        "panel.sculpt3d.cfilter_stretch",
        "panel.sculpt3d.cfilter_volume",
        "panel.sculpt3d.cfilter_bend",
        "panel.sculpt3d.cfilter_mass",
        "panel.sculpt3d.cfilter_damping",
        "panel.sculpt3d.cfilter_plasticity",
        "panel.sculpt3d.cfilter_sweeps",
    ];
    let cfilter: Vec<&str> = rows()
        .map(|r| r.label)
        .filter(|l| l.starts_with("panel.sculpt3d.cfilter_"))
        .collect();
    for e in ESPERADOS {
        assert!(cfilter.contains(&e), "`{e}` sumiu do painel: {cfilter:?}");
    }
    for c in &cfilter {
        assert!(
            ESPERADOS.contains(c),
            "`{c}` e' um numero do filtro que este censo nao conhece -- acrescente-o a \
             ESPERADOS depois de conferir que ele obedece as duas metades abaixo"
        );
    }

    // (1) — com a lei de TECIDO escolhida e um verbo QUALQUER na mão, eles aparecem.
    let mut u = Sculpt3dUi::default();
    u.brush.verb = Verb::Draw;
    u.filter_law = FilterLaw::Cloth(ClothFilterKind::Gravity);
    for r in rows().filter(|r| r.label.starts_with("panel.sculpt3d.cfilter_")) {
        assert!(
            (r.show)(&u),
            "`{}` nao e' oferecida com a lei de tecido escolhida e o Draw na mao -- e' \
             exactamente o estado em que o filtro corre",
            r.label
        );
    }

    // (2) — com uma lei de MALHA escolhida eles somem, mesmo com o pincel de
    // tecido na mão: eles são do FILTRO, e ali o filtro não é de tecido.
    u.brush.verb = Verb::Cloth;
    u.filter_law = FilterLaw::Mesh(FilterKind::Smooth);
    for r in rows().filter(|r| r.label.starts_with("panel.sculpt3d.cfilter_")) {
        assert!(
            !(r.show)(&u),
            "`{}` e' oferecida com uma lei de MALHA escolhida -- ela nao mexe um vertice ali",
            r.label
        );
    }
}

/// ⭐⭐ **A *Quality* DO PINCEL É DO PINCEL** — e o alvo não a tem.
///
/// ⚠️ Ela é o gémeo da do filtro e a pergunta de visibilidade é a **outra**: um
/// knob do pincel aparece com o pincel na mão. *As duas existem, com omissões
/// iguais e donos diferentes, e trocar as perguntas devolveria o defeito ao
/// contrário.*
#[test]
fn a_qualidade_do_pincel_aparece_com_o_pincel() {
    use crate::rows::rows;
    use crate::state::Sculpt3dUi;
    use ph2d_sculpt3d::{ClothFilterKind, FilterLaw, Verb};

    let r = rows()
        .find(|r| r.label == "panel.sculpt3d.cloth_sweeps")
        .expect("a *Quality* do pincel tem de existir");
    let mut u = Sculpt3dUi::default();
    u.brush.verb = Verb::Cloth;
    assert!(
        (r.show)(&u),
        "com o pincel de tecido na mao ela tem de aparecer"
    );
    u.brush.verb = Verb::Draw;
    u.filter_law = FilterLaw::Cloth(ClothFilterKind::Gravity);
    assert!(
        !(r.show)(&u),
        "a *Quality* do PINCEL apareceu com o Draw na mao -- ela e' do pincel, e quem o filtro \
         le^ e' a `cfilter_sweeps`"
    );
}
