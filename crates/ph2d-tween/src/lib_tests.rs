//! Os gates do [`super`] — a lei pura do tween, medida caso a caso.
//!
//! ⚠️ **Cada gate morde uma decisão NOMEADA no doc do [`super::valor`]**, e o plano diz qual lei da
//! casa a paga. Se você mexer aqui, refaça as mutações do handoff.

use super::*;

fn fade() -> Tween {
    Tween::linear(Canal::Opacity, 1.0, 0.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// As quatro respostas do relógio
// ─────────────────────────────────────────────────────────────────────────────

/// **A meio de um fade linear, metade** — o caso normal, e o controlo de que a lei corre.
#[test]
fn a_meio_de_um_fade_linear_e_metade() {
    let t = fade();
    assert_eq!(
        valor(&t, Relogio::a_correr(0.0)),
        Some([1.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        valor(&t, Relogio::a_correr(0.5)),
        Some([0.5, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        valor(&t, Relogio::a_correr(1.0)),
        Some([0.0, 0.0, 0.0, 0.0])
    );
}

/// ⭐⭐⭐ **Por arrancar, ele NÃO ESCREVE NADA** — e é isso que faz o objecto ficar como o artista o
/// autorou. *Escrever `de` sobre um objecto que ninguém mandou animar seria a ferramenta a mudar a
/// cena sozinha.*
#[test]
fn por_arrancar_ele_nao_escreve_nada() {
    assert_eq!(valor(&fade(), Relogio::PARADO), None);
}

/// **`Hold` fica onde chegou; `Rewind` devolve o autorado** — as duas metades do campo, e as duas
/// são precisas: um fade-out quer a primeira, um flash quer a segunda.
#[test]
fn o_fim_tem_duas_respostas_e_as_duas_sao_precisas() {
    let h = fade();
    assert_eq!(h.ao_acabar, AoAcabar::Hold, "o default e' o do fade");
    assert_eq!(valor(&h, Relogio::ACABOU), Some([0.0, 0.0, 0.0, 0.0]));

    let r = Tween {
        ao_acabar: AoAcabar::Rewind,
        ..h
    };
    assert_eq!(
        valor(&r, Relogio::ACABOU),
        None,
        "`Rewind` deixa de escrever, e o ledger repoe"
    );
}

/// ⭐ **PAUSAR CONGELA A IMAGEM** — pausar não é acabar, e um objecto pausado a meio de um fade que
/// saltasse de volta ao autorado leria-se como um defeito do motor.
#[test]
fn uma_pausa_a_meio_congela_a_imagem() {
    assert_eq!(
        valor(&fade(), Relogio::em_pausa(0.25)),
        Some([0.75, 0.0, 0.0, 0.0])
    );
}

/// ⛔ **A FRONTEIRA DECLARADA: uma pausa em `0` é indistinguível de «nunca arrancou».**
///
/// O [`ph2d_ecs::timer::stop`] não mexe no `elapsed_us`, logo os dois leem `progresso == 0` — e a
/// lei devolve `None` aos dois de propósito. ⚠️ **O gate existe para a decisão ser VISÍVEL**: quem
/// a mudar tem de o reprovar, e não descobri-la por um report.
#[test]
fn uma_pausa_no_zero_le_se_como_por_arrancar_e_isso_e_declarado() {
    assert_eq!(valor(&fade(), Relogio::em_pausa(0.0)), None);
    assert_eq!(valor(&fade(), Relogio::PARADO), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// O motor, e o que a lei NÃO reescreve
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐ **A curva é a do [`ph2d_anim`], e o gate mede-a pelo PRODUTO.**
///
/// ⚠️ A régua não é «o número bate»: é que **variar a família MUDA a saída** e que o `Linear`
/// devolve o `u` cru. *Uma lei que ignorasse o `easing` passaria num gate que só olhasse o
/// `Linear`* — que é a forma exacta do defeito que o corpus no ponto neutro de um knob já pagou
/// nesta casa.
#[test]
fn a_curva_e_a_do_motor_e_o_canal_neutro_e_o_cru() {
    let linear = fade();
    assert_eq!(valor(&linear, Relogio::a_correr(0.25)).unwrap()[0], 0.75);

    let mut distintas = std::collections::BTreeSet::new();
    for f in EasingFamily::ALL {
        let t = Tween {
            easing: Easing::new(f, EasingMode::Out),
            ..linear
        };
        let v = valor(&t, Relogio::a_correr(0.25)).unwrap()[0];
        distintas.insert(v.to_bits());
    }
    assert!(
        distintas.len() >= 8,
        "as {} familias deram so' {} saidas distintas — a curva nao esta' a ser lida",
        EasingFamily::ALL.len(),
        distintas.len()
    );
}

/// ⚠️ **O valor do fim é `easing(1)` e não `para` directamente** — e as `33` curvas fecham em `1`
/// por construção, incluindo as duas que ULTRAPASSAM a meio (`Back` e `Elastic`).
///
/// ⭐ É o controlo que torna a escolha honesta: se alguma família não fechasse, escrever `para`
/// seria uma segunda resposta que divergiria dela.
#[test]
fn toda_curva_fecha_no_fim_e_e_por_isso_que_a_lei_a_avalia() {
    for f in EasingFamily::ALL {
        for m in EasingMode::ALL {
            let t = Tween {
                easing: Easing::new(f, m),
                ..fade()
            };
            let v = valor(&t, Relogio::ACABOU).unwrap()[0];
            assert!(
                v.abs() < 1e-6,
                "{f:?}/{m:?} nao fecha: o fim de um fade 1→0 leu {v}"
            );
        }
    }
    // E o controlo do meio: `Back`/`Elastic` de facto ultrapassam, senão o gate acima
    // afirmaria sobre uma população que nao contem o fenomeno.
    let back = Tween {
        easing: Easing::new(EasingFamily::Back, EasingMode::Out),
        ..Tween::linear(Canal::Opacity, 0.0, 1.0)
    };
    let pico = valor(&back, Relogio::a_correr(0.5)).unwrap()[0];
    assert!(
        pico > 1.0,
        "controlo: o `Back` tinha de ultrapassar, leu {pico}"
    );
}

/// ⚠️ **As componentes acima da aridade viajam a `0,0`** — escrever quatro números sobre um canal
/// escalar apagaria três campos que não são dele.
#[test]
fn um_canal_escalar_so_escreve_a_primeira_componente() {
    let t = Tween {
        de: [1.0, 9.0, 9.0, 9.0],
        para: [0.0, 9.0, 9.0, 9.0],
        ..fade()
    };
    assert_eq!(
        valor(&t, Relogio::a_correr(0.5)),
        Some([0.5, 0.0, 0.0, 0.0])
    );
}

/// **Uma cor mistura as QUATRO componentes.**
#[test]
fn uma_cor_mistura_as_quatro() {
    let t = Tween::cor(Canal::Tint, [1.0, 1.0, 1.0, 1.0], [1.0, 0.0, 0.0, 0.5]);
    let t = Tween {
        easing: Easing::LINEAR,
        ..t
    };
    assert_eq!(
        valor(&t, Relogio::a_correr(0.5)),
        Some([1.0, 0.5, 0.5, 0.75])
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O catálogo — as duas leis derivadas
// ─────────────────────────────────────────────────────────────────────────────

/// **A aridade é DERIVADA e a tag é a POSIÇÃO** — as duas leis que o painel herda sem uma segunda
/// lista, com a ida-e-volta a provar que todo canal é alcançável.
#[test]
fn o_catalogo_deriva_a_aridade_e_a_tag() {
    for (i, c) in Canal::ALL.iter().enumerate() {
        assert_eq!(c.tag() as usize, i, "{c:?}: a tag nao e' a posicao");
        assert_eq!(Canal::from_tag(c.tag()), *c, "{c:?}: a ida-e-volta partiu");
        assert!(matches!(c.aridade(), 1 | 4), "{c:?}: aridade estranha");
        assert!(!c.label().is_empty());
    }
    // Um tag fora de alcance cai no primeiro, nunca em panico.
    assert_eq!(Canal::from_tag(200), Canal::Opacity);
    // As duas familias, com PISO: um censo que lesse zero de um lado passaria por vacuidade.
    let cor = Canal::ALL.iter().filter(|c| c.aridade() == 4).count();
    let pose = Canal::ALL.iter().filter(|c| c.e_da_pose()).count();
    assert!(cor >= 2, "so' {cor} canais de cor");
    assert!(pose >= 5, "so' {pose} canais de pose");
    assert_eq!(
        cor + pose + 1,
        Canal::ALL.len(),
        "ha' um canal que nao e' nem cor nem pose nem a opacidade"
    );
}

/// ⚠️ **Nenhum canal de POSE escreve no `Sprite`, e nenhum de APARÊNCIA escreve no `Transform`** —
/// a partição que decide qual das duas entradas do ledger a ponte usa.
#[test]
fn a_particao_entre_pose_e_aparencia_e_total() {
    for c in Canal::ALL {
        let aparencia = matches!(c, Canal::Opacity | Canal::Tint | Canal::Silhueta);
        assert_ne!(
            c.e_da_pose(),
            aparencia,
            "{c:?} cai nos dois lados ou em nenhum"
        );
    }
}

/// **O `AoAcabar` obedece às mesmas duas leis** — tag pela posição, ida-e-volta.
#[test]
fn o_ao_acabar_deriva_a_tag() {
    for (i, a) in AoAcabar::ALL.iter().enumerate() {
        assert_eq!(a.tag() as usize, i);
        assert_eq!(AoAcabar::from_tag(a.tag()), *a);
    }
    assert_eq!(AoAcabar::from_tag(200), AoAcabar::Hold);
}

/// ⚠️ **O default NÃO é inerte** — um componente acabado de anexar que não faz nada lê-se como
/// partido, que é a lei que o `Timer::default` já escreve para o irmão que o faz correr.
#[test]
fn o_default_faz_alguma_coisa() {
    let t = Tween::default();
    assert_ne!(t.de, t.para, "o default nasceu inerte");
    let a = valor(&t, Relogio::a_correr(0.0)).unwrap();
    let b = valor(&t, Relogio::a_correr(1.0)).unwrap();
    assert_ne!(a, b, "o default nao move nada");
}

// ─────────────────────────────────────────────────────────────────────────────
// A MEDIÇÃO do §6.1 do plano — em que espaço a cor interpola
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A decisão do espaço de cor, com o número ao lado** (plano §6.1).
///
/// A alternativa é real: o [`ph2d_anim::AnimValue::Color`] guarda OKLCH **com arco curto de
/// matiz**, e o levantamento chama a combinação de diferenciador. ⇒ ela tem de ser **CORRIDA** ao
/// lado da nossa, não discutida.
///
/// ⚠️ **Sonda, não gate**: ela imprime, e quem decide é quem lê.
///
/// ```text
/// cargo test -p ph2d-tween a_cor_em_dois_espacos -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a medicao do §6.1 do plano"]
fn a_cor_em_dois_espacos() {
    use ph2d_color::{LinearRgba, OklabColor};

    println!("\n══════ a rampa BRANCO → VERMELHO, nos dois espacos ══════");
    let de = [1.0_f32, 1.0, 1.0, 1.0];
    let para = [1.0_f32, 0.0, 0.0, 1.0];
    let t = Tween {
        easing: Easing::LINEAR,
        ..Tween::cor(Canal::Tint, de, para)
    };

    let a = OklabColor::from_linear(LinearRgba::new(de[0], de[1], de[2], de[3]));
    let b = OklabColor::from_linear(LinearRgba::new(para[0], para[1], para[2], para[3]));

    let mut pior = 0.0_f32;
    for i in 0_u8..=8 {
        let u = f32::from(i) / 8.0_f32;
        let nosso = valor(&t, Relogio::a_correr(u)).unwrap();
        let k = u;
        let ok = OklabColor::new(
            a.l + (b.l - a.l) * k,
            a.a + (b.a - a.a) * k,
            a.b + (b.b - a.b) * k,
            a.alpha + (b.alpha - a.alpha) * k,
        )
        .to_linear();
        let d = (nosso[0] - ok.r())
            .abs()
            .max((nosso[1] - ok.g()).abs())
            .max((nosso[2] - ok.b()).abs());
        pior = pior.max(d);
        println!(
            "  u = {u:.3}  |  no SINK [{:.4} {:.4} {:.4}]  |  OKLab [{:.4} {:.4} {:.4}]  |  Δ {d:.4}",
            nosso[0],
            nosso[1],
            nosso[2],
            ok.r(),
            ok.g(),
            ok.b()
        );
    }
    println!("  ⇒ desvio MAXIMO entre os dois espacos: {pior:.4}");

    // ⭐⭐⭐ O ACHADO, e ele nao era a hipotese com que esta sonda foi escrita.
    let mut fora = 0_usize;
    for i in 0_u8..=8 {
        let k = f32::from(i) / 8.0_f32;
        let ok = OklabColor::new(
            a.l + (b.l - a.l) * k,
            a.a + (b.a - a.a) * k,
            a.b + (b.b - a.b) * k,
            a.alpha + (b.alpha - a.alpha) * k,
        )
        .to_linear();
        if ok.r() > 1.0 || ok.g() > 1.0 || ok.b() > 1.0 {
            fora += 1;
        }
    }
    println!("  ⇒ amostras da rampa OKLab FORA do gamute [0, 1]: {fora} de 9  (pico r = 1,1099)");
    println!(
        "  ⇒ e' ISTO que decide, e nao o desvio: um `tint` e' um MULTIPLICADOR do texel, logo\n     \
         um valor acima de `1` nao e' «outra cor», e' o sprite a CLAREAR — e limitar a` entrada\n     \
         deformaria a curva, que era o ponto inteiro de usar OKLab. A lei que ship interpola no\n     \
         espaco do SINK, onde a rampa e' fechada por construcao; e o canal `Opacity` da timeline\n     \
         ja' interpola assim (um lerp cru em `tint[3]`), logo duas leis dariam duas respostas a\n     \
         «que cor e' esta a meio?»."
    );
    assert!(pior > 0.0, "controlo: os dois espacos TE^M de discordar");
    assert!(
        fora > 0,
        "controlo: a rampa OKLab TEM de sair do gamute nesta fixtura"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O CICLO (W8) — o *ping-pong* que o dono pediu em 2026-09-19.
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A DOBRA é um triângulo, e o `Reinicia` é a IDENTIDADE AO BIT.**
///
/// ⚠️ **A segunda metade é a que protege tudo o que já shipava:** se o caminho de omissão não for
/// byte-idêntico, este campo muda toda cena gravada — e mudaria em silêncio, porque um tween
/// *quase* igual continua a parecer um tween.
///
/// **Mutações que devem sangrar:** `1 - |2u - 1|` virar `|2u - 1|` (o triângulo ao contrário) ·
/// `Reinicia` devolver `u.clamp(0,1)` em vez de `u`.
#[test]
fn a_dobra_do_pingpong_e_um_triangulo_e_o_reinicia_e_identidade() {
    for (u, esperado) in [(0.0, 0.0), (0.25, 0.5), (0.5, 1.0), (0.75, 0.5), (1.0, 0.0)] {
        let lido = Ciclo::PingPong.dobra(u);
        assert!(
            (lido - esperado).abs() < 1e-12,
            "dobra({u}) = {lido}, esperado {esperado}"
        );
    }
    // ⛔ **AO BIT**, e não «perto»: é a promessa de que ligar este campo não mexeu no produto.
    for i in 0..=1000 {
        let u = f64::from(i) / 1000.0;
        assert_eq!(
            Ciclo::Reinicia.dobra(u),
            u,
            "o `Reinicia` mexeu no progresso"
        );
    }
}

/// ⭐⭐ **Um ping-pong VAI E VOLTA dentro do mesmo período** — o que a sonda do §5.0 mediu que a
/// composição não dá (`0` de `33` curvas reflectem; o `repeat` do relógio salta).
///
/// ⚠️ **A régua é a SIMETRIA sobre o meio**, e é ela que separa isto de uma serra: `valor(u)` tem
/// de ser igual a `valor(1 − u)`, e o meio tem de ser o extremo.
#[test]
fn um_pingpong_vai_e_volta_no_mesmo_periodo() {
    let t = Tween {
        ciclo: Ciclo::PingPong,
        ..Tween::linear(Canal::Opacity, 0.0, 1.0)
    };
    let v = |u: f32| valor(&t, Relogio::a_correr(u)).expect("a correr, ele escreve")[0];
    assert!((v(0.0) - 0.0).abs() < 1e-6, "comeca em `de`");
    assert!((v(0.5) - 1.0).abs() < 1e-6, "a MEIO ele esta' em `para`");
    assert!((v(1.0) - 0.0).abs() < 1e-6, "no fim voltou a `de`");
    for i in 0_u8..=10 {
        let u = f32::from(i) / 10.0;
        assert!(
            (v(u) - v(1.0 - u)).abs() < 1e-6,
            "a ida e a volta discordam em u = {u}"
        );
    }
    // ⛔ O CONTROLO: o mesmo tween com `Reinicia` NÃO volta — ele acaba em `para`.
    let serra = Tween {
        ciclo: Ciclo::Reinicia,
        ..t
    };
    let fim = valor(&serra, Relogio::a_correr(1.0)).expect("escreve")[0];
    assert!(
        (fim - 1.0).abs() < 1e-6,
        "controlo: a serra acaba em `para`"
    );
}

/// ⭐⭐⭐ **A DOBRA VEM ANTES DA CURVA** — e a régua separa as duas ordens.
///
/// ⚠️ Dobrar **antes** faz a ida e a volta percorrerem a MESMA forma (o progresso dobrado é
/// simétrico sobre o meio, logo o valor também é). Dobrar **depois** aplicaria a curva ao tempo
/// cru, e `curva(u)` e `curva(1 − u)` são coisas diferentes em toda curva que não seja simétrica
/// ⇒ a volta teria outro aspecto que a ida.
///
/// **Mutação que deve sangrar:** `eval(dobra(u))` virar `dobra(eval(u))`.
#[test]
fn a_dobra_vem_antes_da_curva() {
    let t = Tween {
        ciclo: Ciclo::PingPong,
        easing: Easing::new(EasingFamily::Quad, EasingMode::In),
        ..Tween::linear(Canal::Opacity, 0.0, 1.0)
    };
    let v = |u: f32| valor(&t, Relogio::a_correr(u)).expect("escreve")[0];
    for i in 0_u8..=10 {
        let u = f32::from(i) / 10.0;
        assert!(
            (v(u) - v(1.0 - u)).abs() < 1e-6,
            "com a dobra DEPOIS da curva, a volta teria outra forma (u = {u})"
        );
    }
    // ⛔ O CONTROLO da própria régua: a curva escolhida TEM de ser assimétrica, senão as duas
    //    ordens dariam o mesmo e este gate não afirmava nada.
    let c = Easing::new(EasingFamily::Quad, EasingMode::In);
    assert!(
        (c.eval(0.25) - c.eval(0.75)).abs() > 0.1,
        "controlo: a curva do gate e' simetrica, e entao ele nao discrimina"
    );
}

/// ⚠️ **No FIM, um ping-pong com `Hold` fica em `de`** — ele foi e voltou, e é ali que ficou.
///
/// ⛔ *Não* é o mesmo que `Rewind`: ali o motor **deixa de escrever** e o ledger repõe o valor
/// autorado, que pode não ser o `de`.
#[test]
fn um_pingpong_que_fica_acaba_onde_comecou() {
    let t = Tween {
        ciclo: Ciclo::PingPong,
        ao_acabar: AoAcabar::Hold,
        ..Tween::linear(Canal::Opacity, 0.0, 1.0)
    };
    assert_eq!(valor(&t, Relogio::ACABOU), Some([0.0, 0.0, 0.0, 0.0]));
    // O CONTROLO: com `Reinicia`, o mesmo `Hold` fica em `para`.
    let r = Tween {
        ciclo: Ciclo::Reinicia,
        ..t
    };
    assert_eq!(valor(&r, Relogio::ACABOU), Some([1.0, 0.0, 0.0, 0.0]));
}

/// ⚠️ **A tag de um ciclo fecha nos dois sentidos**, e a POSIÇÃO no `ALL` é a tag — reordenar o
/// array faria um clique escrever outro valor, e compila.
#[test]
fn a_tag_de_um_ciclo_fecha_nos_dois_sentidos() {
    for (i, c) in Ciclo::ALL.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let tag = i as u8;
        assert_eq!(c.tag(), tag);
        assert_eq!(Ciclo::from_tag(tag), *c);
    }
    assert_eq!(
        Ciclo::from_tag(200),
        Ciclo::default(),
        "uma tag fora cai no primeiro"
    );
    assert_eq!(
        Ciclo::default(),
        Ciclo::Reinicia,
        "o default e' o que ja' shipava"
    );
}
