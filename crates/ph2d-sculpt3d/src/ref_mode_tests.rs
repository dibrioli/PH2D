//! **Os gates da tabela dos modos** — que ela diz o que a FONTE diz, que não há
//! segunda porta, e que nenhum chip morto pode nascer dela.

use super::*;
use crate::falloff::Falloff;
use crate::{Brush, Pass};

/// **A coluna `S` é o que o SculptGL declara**, tool a tool, com arquivo e linha.
///
/// ⚠️ **Estes números foram LIDOS da fonte, não afinados** — e o gate os repete
/// aqui de propósito: a tabela é `const` e um dedo escorregado nela não falha
/// nada, muda o pincel de fábrica de todo mundo em silêncio.
///
/// ⚠️ E ele pina o **raio como FRAÇÃO**: um `25.0` escrito à mão passaria
/// enquanto o raio-base fosse 50 e mentiria no dia seguinte.
#[test]
fn the_s_column_is_what_the_sculptgl_sources_declare() {
    /// Uma linha lida da fonte do SculptGL. ⚠️ O `radius_px` é o número do
    /// ORIGINAL (não a nossa fração) — é o que se confere contra o arquivo.
    struct Row {
        verb: Verb,
        strength: Option<f32>,
        radius_px: Option<f32>,
        accumulate: Option<bool>,
        src: &'static str,
    }
    const fn row(
        verb: Verb,
        strength: Option<f32>,
        radius_px: f32,
        accumulate: Option<bool>,
        src: &'static str,
    ) -> Row {
        Row {
            verb,
            strength,
            radius_px: Some(radius_px),
            accumulate,
            src,
        }
    }

    let want = [
        row(Verb::Draw, Some(0.5), 50.0, Some(true), "Brush.js:11-16"),
        row(Verb::Clay, Some(0.5), 50.0, Some(true), "Brush.js:11-16"),
        row(
            Verb::Inflate,
            Some(0.3),
            50.0,
            Some(false),
            "Inflate.js:9-11",
        ),
        row(
            Verb::Smooth,
            Some(0.75),
            50.0,
            Some(false),
            "Smooth.js:10-13",
        ),
        row(
            Verb::Flatten,
            Some(0.75),
            50.0,
            Some(true),
            "Flatten.js:9-11",
        ),
        row(Verb::Fill, Some(0.75), 50.0, Some(true), "Flatten.js:9-11"),
        row(
            Verb::Scrape,
            Some(0.75),
            50.0,
            Some(true),
            "Flatten.js:9-11",
        ),
        row(Verb::Pinch, Some(0.75), 50.0, Some(false), "Pinch.js:9-11"),
        row(
            Verb::Magnify,
            Some(0.75),
            50.0,
            Some(false),
            "Pinch.js:9-11",
        ),
        row(
            Verb::Crease,
            Some(0.75),
            25.0,
            Some(false),
            "Crease.js:9-11",
        ),
        row(Verb::Mask, Some(1.0), 50.0, Some(false), "Masking.js:13-16"),
        row(Verb::Move, Some(1.0), 150.0, None, "Move.js:10-11"),
        row(Verb::SnakeHook, None, 150.0, None, "Drag.js:10"),
        row(Verb::Twist, None, 75.0, None, "Twist.js:10"),
        row(Verb::LocalScale, None, 50.0, None, "LocalScale.js:8"),
    ];

    for r in &want {
        let src = r.src;
        let p = r
            .verb
            .profile(RefMode::S)
            .unwrap_or_else(|| panic!("{} tem perfil `S` — {src}", r.verb.label()));
        assert_eq!(p.strength, r.strength, "{} força — {src}", r.verb.label());
        assert_eq!(
            p.accumulate,
            r.accumulate,
            "{} accumulate — {src}",
            r.verb.label()
        );
        assert_eq!(
            p.radius_factor.map(|f| f * 50.0),
            r.radius_px,
            "{} raio — {src}",
            r.verb.label()
        );
        // ⚠️ Toda tool de geometria do original compartilha a quártica.
        assert_eq!(
            p.falloff,
            Some(Falloff::Plateau),
            "{} usa a curva da referência",
            r.verb.label()
        );
    }
}

/// ⚠️ **O `None` do Sharpen é uma AFIRMAÇÃO: o SculptGL não tem essa tool.**
///
/// Sem este gate, alguém "completaria a tabela" com um número inventado e ele
/// shiparia com a autoridade de uma referência que não o declara.
#[test]
fn the_reference_has_no_sharpen_and_the_table_says_so() {
    assert_eq!(Verb::Sharpen.profile(RefMode::S), None);
    // E o controle: o vizinho de família TEM perfil, então o `None` acima é
    // sobre o Sharpen e não sobre a tabela estar vazia.
    assert!(Verb::Smooth.profile(RefMode::S).is_some());
}

/// **Um chip existe se e somente se o perfil existe** — a lei anti-chip-morto do
/// plano §3, afirmada sobre a ÚNICA lista que a decide.
///
/// ⚠️ **O corpo dele era o RETRATO da W0 e não a lei** (*"`B` e `L` são `None`
/// em todo verbo"*), então a W3 o derrubou ao dar ao `B` o que declarar — que é
/// o gate a funcionar, não a falhar. O que sobrevive é a propriedade; o CENSO
/// abaixo é que se move por wave, e move de propósito.
#[test]
fn every_offered_chip_has_a_profile_behind_it() {
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            // A porta é UMA: quem oferece e quem honra perguntam a mesma coisa.
            let p = verb.profile(mode);
            if let Some(p) = p {
                // Um perfil que não afirma NADA seria um chip morto com outro
                // nome — ele tem de declarar ao menos um campo.
                let says_something = p.falloff.is_some()
                    || p.strength.is_some()
                    || p.radius_factor.is_some()
                    || p.accumulate.is_some()
                    || p.strength_curve != StrengthCurve::Linear;
                assert!(
                    says_something,
                    "{} × {}: perfil que não declara nada é um chip morto",
                    verb.label(),
                    mode.label()
                );
            }
        }
    }
}

/// ⚠️ **O CENSO — quantos chips cada modo oferece HOJE.**
///
/// Ele existe para uma wave que preencha uma célula ter de vir aqui dizer o
/// número novo: um perfil que aparece sem passar por esta linha é um chip que
/// nasceu sem ninguém decidir que ele devia nascer.
#[test]
fn the_census_of_offered_chips() {
    let count = |m: RefMode| Verb::ALL.iter().filter(|v| v.profile(m).is_some()).count();
    // ⚠️ **O SculptGL tem tudo menos o Sharpen, o Clay Strips E o Blob**
    // (`Verb::ALL` tem 18). O número **não se moveu duas vezes seguidas** — a
    // faixa e depois o Blob acrescentaram cada um um verbo E um `None`, e as
    // duas vezes um censo lido só pelo número teria dito *"nada mudou"* sobre
    // uma tabela que mudou duas vezes.
    //
    // ⇒ Este `15` é o exemplo de que **um censo de CONTAGEM não é um censo de
    // CONTEÚDO**: quem o move é o `B` logo abaixo, e quem diria o resto seria a
    // lista por nome que o gate da literatura carrega. A frase fica porque a
    // coincidência já reincidiu.
    //
    // ⚠️ **E ela reincidiu de novo, com o Slide Relax — a QUINTA vez, e a
    // primeira em que eu escrevi o número errado antes de o gate me corrigir.**
    // Eu contei `21 − 5` pela lista do `declares` e este censo mede outra
    // tabela: ele conta PERFIS (`profile(m).is_some()`), e o `profile_s` cala
    // sobre **seis** verbos, porque o Sharpen entra nele e não na outra lista. A
    // aritmética certa é `21 − 6 = 15`, e o número não se move pela quinta vez
    // seguida. *Duas tabelas com o mesmo aspecto e listas diferentes são
    // exactamente a razão de um censo de contagem precisar da frase ao lado.*
    // ⚠️ O número NÃO se move com o Surface Smooth, e é isso que a frase tem de
    // dizer: o catálogo cresceu de 21 para 22 e a lista de exclusões cresceu
    // junto, então *um censo de contagem sozinho seria verde sobre um verbo que
    // ninguém classificou*.
    assert_eq!(
        count(RefMode::S),
        15,
        "S: todos menos o Sharpen, o Clay Strips, o Blob, o Clay Thumb, o Multiplane Scrape, o Slide Relax e o Surface Smooth"
    );
    // ⚠️ O `B` alcança TODO verbo com uma coisa só — o `alpha = root_alpha²` do
    // `brush_strength`, que é o funil de todas as tools.
    //
    // ⚠️ **Os DEFAULTS dele seguem bloqueados, e o motivo NÃO é o clone** — o
    // `brush.cc` foi trazido e respondeu que a resposta não está nele: o
    // `BKE_brush_sculpt_reset` **não existe mais em C** desde o Blender 4.3
    // (§7.0 do plano). Este número é sobre a LEI, não sobre os defaults.
    //
    // ⚠️ **17 desde a W6, 18 desde o Blob, 19 desde o Clay Thumb, 20 desde o
    // Multiplane Scrape, 21 desde o Slide Relax, 22 desde o Surface Smooth:** os
    // seis são tools do BLENDER (`clay_strips.cc`, `crease.cc`,
    // `clay_thumb.cc`, `multiplane_scrape.cc`, `relax.cc`,
    // `surface_smooth.cc`), então o `alpha = root_alpha²` do `brush_strength` —
    // que é o funil de todas elas — vale para as seis por construção.
    assert_eq!(count(RefMode::B), 22, "B: a lei da força vale para todos");
    // A literatura chega paper a paper, nas waves W4/W5/W7.
    //
    // ⚠️ **E o Surface Smooth NÃO o move, embora SEJA um paper** (Vollmer,
    // Mencl & Müller, EG 1999): o que portámos é o `surface_smooth.cc`, ou seja
    // *a rendição do Blender do próprio paper* — as duas leis são a MESMA, e um
    // segundo chip declarando-a seria um controle que não muda um vértice.
    // Declarar `L` ali afirmaria um porte a partir do artigo que não foi feito.
    assert_eq!(count(RefMode::L), 0, "L: nenhum paper portado ainda");
}

/// **Não há segunda porta para a força de fábrica.**
///
/// ⚠️ O gate compara a delegação com a TABELA, não com uma cópia dos números:
/// uma expectativa escrita à mão aqui seria exatamente a segunda porta que a
/// delegação existe para matar.
#[test]
fn the_factory_strength_is_the_table_and_nothing_else() {
    for verb in Verb::ALL {
        let from_table = verb
            .profile(RefMode::S)
            .and_then(|p| p.strength)
            .unwrap_or(0.5);
        assert_eq!(
            verb.default_strength(),
            from_table,
            "{}: a força de fábrica DELEGA",
            verb.label()
        );
    }
}

/// **A delegação do Accumulate é BYTE-IDÊNTICA ao `matches!` que ela
/// substituiu** — a wave move a força e **não** move o accumulate, e esta é a
/// linha que prova que ela não moveu.
#[test]
fn the_accumulate_delegation_changed_nothing() {
    for verb in Verb::ALL {
        let before = matches!(
            verb,
            Verb::Draw | Verb::Clay | Verb::Flatten | Verb::Fill | Verb::Scrape
        );
        assert_eq!(
            verb.default_accumulate(),
            before,
            "{}: o accumulate de fábrica não se moveu nesta wave",
            verb.label()
        );
    }
}

/// ⚠️ **O que esta wave MUDA, num número** — para o smoke saber o que procurar
/// e para uma reversão custar um gate vermelho em vez de passar calada.
///
/// Antes desta tabela o app shipava `0,5` em toda geometria (o **D3** do doc
/// 20). O Draw é o único que sobrevive intacto.
#[test]
fn the_wave_moves_the_factory_strength_of_nine_verbs_and_only_these() {
    let moved: Vec<&str> = Verb::ALL
        .iter()
        .filter(|v| {
            let old = if v.paints_mask() { 1.0 } else { 0.5 };
            (v.default_strength() - old).abs() > f32::EPSILON
        })
        .map(|v| v.label())
        .collect();
    assert_eq!(
        moved,
        vec![
            "Inflate",
            "Smooth",
            "Flatten",
            "Fill",
            "Scrape",
            "Pinch",
            "Magnify",
            "Crease",
            "Move / Grab",
        ],
        "a lista do que a referência move; o Draw e a Mask já batiam"
    );
}

/// **O E13 — o slider é a RAIZ do peso no `B`, e o próprio número no `S`.**
///
/// ⚠️ A afirmação vale no PRODUTO, não na tabela: um gate que só perguntasse
/// `strength_curve` seria a tabela conferindo a si mesma. Ele passa pela porta
/// [`Brush::weight`], que é o que o `stroke.rs` consome.
#[test]
fn the_blender_slider_is_the_square_root_of_the_weight() {
    let mut b = crate::Brush {
        strength: 0.5,
        ..Default::default()
    };
    assert!(
        (b.weight() - 0.5).abs() < 1e-6,
        "S: o slider É o peso; veio {}",
        b.weight()
    );
    b.mode = RefMode::B;
    assert!(
        (b.weight() - 0.25).abs() < 1e-6,
        "B: `sculpt.cc:2339` eleva ao quadrado; veio {}",
        b.weight()
    );
    // ⚠️ E as pontas COINCIDEM (`0²=0`, `1²=1`) — é por isso que a fixture usa
    // meio curso: nos extremos os dois modos são indistinguíveis e o gate seria
    // verde por vácuo.
    for s in [0.0, 1.0] {
        b.strength = s;
        b.mode = RefMode::S;
        let lin = b.weight();
        b.mode = RefMode::B;
        assert!(
            (b.weight() - lin).abs() < 1e-6,
            "as pontas coincidem em {s}"
        );
    }
}

/// **E o traço de fato deposita menos** — o `weight()` chega ao barro.
///
/// ⚠️ Sem isto o gate acima prova só aritmética: a porta poderia existir, estar
/// certa, e o `stroke.rs` seguir lendo `strength` cru.
#[test]
fn the_mode_reaches_the_clay() {
    let travel = |mode: RefMode| {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
        let brush = crate::Brush {
            verb: Verb::Draw,
            mode,
            strength: 0.5,
            radius: 0.5,
            ..Default::default()
        };
        let before: Vec<[f32; 3]> = mesh.positions().to_vec();
        let mut st = crate::SculptStroke::default();
        st.begin(&mesh);
        let dab = crate::Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]);
        st.dab(&mut mesh, &brush, &dab, crate::Symmetry::default());
        before
            .iter()
            .zip(mesh.positions())
            .map(|(a, b)| {
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max)
    };
    let s = travel(RefMode::S);
    let b = travel(RefMode::B);
    assert!(s > 1e-6, "o controle tem de mover barro; veio {s}");
    assert!(
        b < s * 0.75,
        "o B deposita MENOS a meio curso: S={s} contra B={b}"
    );
}

/// **A ASSINATURA OBSERVÁVEL de um modo sobre um verbo** — tudo o que o artista
/// pode ver mudar ao trocar o chip, e nada além.
///
/// ⚠️ **Ela existe porque a comparação óbvia é FALSA, e eu a escrevi primeiro:**
/// o gate comparava `verb.profile(a) != verb.profile(b)`, e o [`VerbProfile`]
/// carrega quatro campos (falloff, força, fração de raio, accumulate) que
/// **NENHUM caminho lê pelo modo corrente** — os defaults são armados da coluna
/// `S`, sempre (`the_factory_strength_is_the_table_and_nothing_else`). Ou seja:
/// `S` e `L` no Smooth têm perfis diferentes, e essa diferença **não move um
/// vértice**. O gate teria ficado verde por um discriminante que não é o da
/// feature.
///
/// O que de fato alcança o barro são QUATRO coisas: a [`KernelLaw`], a curva
/// que traduz o slider em peso, **quantos passes o dab faz** e **que campo
/// elástico governa o deslocamento**.
///
/// ⚠️ **O quarto eixo entrou na W5, e a entrada dele foi o gate a MORDER:** o
/// `l-mode` do Grab não muda lei de kernel, não muda curva e faz **um** passe —
/// sob a assinatura de três eixos ele era, letra por letra, o `s-mode`, e este
/// arquivo dizia *"Move / Grab: S e L são o mesmo modo"* sobre dois modos que
/// movem o barro de formas diferentes. É a mesma lição que o doc acima já
/// carregava, cobrada uma segunda vez: **uma assinatura que não contém um eixo
/// observável é um discriminante falso**, e a única defesa é ela crescer no
/// mesmo commit que o eixo.
/// ⚠️ **O [`crate::LateralPull`] entrou aqui em 2026-08-15, e a ausência dele era
/// um buraco:** a lei lateral deixou de ser um campo do [`KernelLaw`] e passou a
/// ser por VERBO ([`RefMode::lateral_for`]), porque o Blender tem duas
/// ferramentas nesta família e duas leis. Sem esta coluna a assinatura de um
/// chip ignora o eixo que a wave tornou o mais visível.
///
/// ⚠️ **E ela é uma defesa em camadas que HOJE não é observável — a mutação que
/// a removeria não sangra, e isso fica escrito em vez de ser contrabandeado.**
/// O `S` e o `B` já diferem no [`KernelLaw`] (`plane` e `front_face`), então
/// nenhum par de modos colide nas outras colunas e a falta desta não produz um
/// falso *"duplicata"* hoje. Ela existe para o dia em que dois modos coincidirem
/// em tudo menos no aperto — e nesse dia o gate teria declarado um chip morto
/// que não é. Precedente do repo: quando a corrida não existe no regime que
/// shipa, documenta-se.
fn signature(
    verb: Verb,
    mode: RefMode,
) -> (
    KernelLaw,
    crate::LateralPull,
    StrengthCurve,
    &'static [Pass],
    Option<crate::Field>,
) {
    let brush = Brush {
        verb,
        mode,
        ..Brush::default()
    };
    let curve = verb
        .profile(mode)
        .map_or(StrengthCurve::Linear, |p| p.strength_curve);
    (
        mode.kernel(),
        mode.lateral_for(verb),
        curve,
        brush.passes(),
        mode.field(verb),
    )
}

/// **UM CHIP EXISTE SE E SOMENTE SE O MODO EXISTE** — a lei §3 do plano, agora
/// com a porta que o painel pergunta.
///
/// ⚠️ Um chip que produz exatamente o que o vizinho produz é um botão que não
/// faz nada, e o artista descobre isso clicando. A propriedade é escrita sobre
/// a [`signature`], não enumerada.
#[test]
fn a_mode_is_offered_only_where_it_is_not_a_duplicate_of_an_earlier_one() {
    for verb in Verb::ALL {
        let offered: Vec<RefMode> = RefMode::offered_for(verb).collect();
        for (i, a) in offered.iter().enumerate() {
            for b in offered.iter().skip(i + 1) {
                assert_ne!(
                    signature(verb, *a),
                    signature(verb, *b),
                    "{}: {} e {} são o mesmo modo",
                    verb.label(),
                    a.label(),
                    b.label()
                );
            }
        }
    }
}

/// **A LITERATURA É OFERECIDA EXATAMENTE ONDE ELA DECLARA UMA LEI** — a
/// bi-implicação, nas duas direções.
///
/// ⚠️ **Ele SUBSTITUI o `the_l_mode_is_withheld_because_nothing_of_its_own_was_built_yet`,
/// que era o mesmo gate com a resposta do outro lado** — e é o gate a funcionar,
/// não a falhar: ele foi escrito para **cobrar a decisão** no dia em que o `L`
/// ganhasse conteúdo, e a W4 é esse dia. O que aquele pinava (nenhum perfil ·
/// kernel por fallback · chip retido) descrevia um estado que acabou.
///
/// ⚠️ **As duas metades são necessárias e nenhuma implica a outra:**
///
/// - *oferecido ⇒ declara* — sem ela, um chip aparece e não faz nada;
/// - *declara ⇒ oferecido* — sem ela, o par λ|μ existe no kernel e **não tem
///   como ser escolhido**, que é a feature invisível. Foi exatamente o estado
///   em que o `L` viveu três waves.
///
/// ⚠️ **A COINCIDÊNCIA CAIU NA W5, exatamente como o doc antigo previu**, e o
/// que estava escrito aqui era: *"o Kelvinlets é um campo de UM passe — no dia
/// em que ele chegar, este gate cai e obriga quem o traz a vir aqui dizer o que
/// passou a ser o discriminante"*. Este é o dia, e a resposta é que **não há um
/// discriminante derivável**: o `L` declara uma lei quando faz mais de um passe
/// (Taubin) **ou** quando entrega um [`crate::Field`] (Kelvinlets), e a próxima
/// família da literatura pode não ser nem uma coisa nem outra.
///
/// ⇒ O que este gate pina hoje é a **BI-IMPLICAÇÃO contra a lista de mecanismos
/// conhecidos**, e o censo abaixo é o que obriga a decisão a passar por aqui.
/// Um paper cuja lei não seja nenhum dos dois faz este gate falhar — que é o
/// comportamento certo: ele não sabe adivinhar o mecanismo do próximo.
#[test]
fn the_literature_mode_is_offered_exactly_where_it_declares_a_law() {
    let mut declared = Vec::new();
    for verb in Verb::ALL {
        let offered = RefMode::offered_for(verb).any(|m| m == RefMode::L);
        let brush = Brush {
            verb,
            mode: RefMode::L,
            ..Brush::default()
        };
        // Os DOIS mecanismos pelos quais uma referência da literatura pode
        // governar o barro hoje: um par de passes, ou um campo elástico.
        let has_law = brush.passes().len() > 1 || RefMode::L.field(verb).is_some();
        assert_eq!(
            offered,
            has_law,
            "{}: o chip L e a lei dele discordam",
            verb.label()
        );
        if offered {
            declared.push(verb.label());
        }
    }
    // O CENSO: uma wave que dê conteúdo ao `L` noutro verbo tem de vir aqui
    // dizer o número novo. Um paper que aparece sem passar por esta linha é um
    // chip que nasceu sem ninguém decidir que ele devia nascer.
    assert_eq!(
        declared,
        vec![
            "Smooth",
            "Magnify",
            "Move / Grab",
            "Snake Hook",
            "Twist",
            "Local Scale"
        ],
        // ⚠️ **O censo ENCOLHEU em 2026-08-15, e é a primeira vez.** Pinch,
        // Crease e Blob saíram com o `Field::Pinch`: um chip `L` neles vestia
        // uma lei que **nenhuma das três fontes declara** (o
        // `elastic_deform.cc` do Blender porta o mesmo paper e traz cinco
        // famílias, nenhuma delas o pinch), e o preço estava medido — 44 a 62 %
        // do gesto caía fora do anel do cursor. Ver
        // [`crate::Verb::elastic_field`] para a tabela inteira.
        "a literatura portada até hoje"
    );
}

/// **O `L` DECLARA UMA LEI PRÓPRIA, e ela não é a de nenhum vizinho.**
///
/// ⚠️ **A [`KernelLaw`] dele coincide com a do `S` ao campo, e isso é uma
/// DECLARAÇÃO e não uma herança** — o Taubin é um filtro de malha e não sabe
/// onde está a câmera, então `front_face: Ignored` é o que o paper diz. É por
/// isso que a assinatura dos dois **ainda difere**: o discriminante é o par
/// λ|μ, e é ele que este gate obriga a existir.
#[test]
fn the_literature_law_differs_from_its_neighbours_only_where_the_paper_speaks() {
    // A única diferença observável entre `S` e `L` no Smooth é o par de passes
    // — é isso que o chip promete, e mais nada viaja de carona.
    assert_eq!(
        RefMode::L.kernel(),
        RefMode::S.kernel(),
        "o L não pode arrastar um portão de face que o paper não menciona"
    );
    assert_ne!(
        RefMode::L.kernel(),
        RefMode::B.kernel(),
        "o L deixou de ser o fallback à lei do B na W4"
    );
    assert_ne!(
        signature(Verb::Smooth, RefMode::S),
        signature(Verb::Smooth, RefMode::L),
        "sem o par λ|μ o chip L seria o S com outro nome"
    );
}
