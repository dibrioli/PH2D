//! Os gates do [`super`] — a lei da POSIÇÃO de um som, medida caso a caso.
//!
//! ⚠️ **Tocar não se testa aqui**, e é a fronteira do módulo: a voz é um recurso do dispositivo e
//! o dispositivo vive na shell. Aqui prova-se o que a lei DECIDE — quanto se ouve, de que lado, e
//! quem são as orelhas.

use super::*;
use crate::{StableId, Transform, assign_missing_stable_ids};

/// Uma fonte com o alcance dado e tudo o resto no default.
fn fonte(max_distance: f32) -> AudioSource2D {
    AudioSource2D {
        sound: "passo.wav".into(),
        max_distance,
        ..AudioSource2D::default()
    }
}

// ─────────────────────────────── o que cala uma fonte ───────────────────────────────

/// ⛔ **As TRÊS maneiras de uma fonte ficar muda sem nada estar partido.**
///
/// ⚠️ As três no mesmo gate porque são a mesma pergunta do painel — *«porque é que não ouço
/// nada?»* — e responder só a uma deixaria as outras duas a parecer um defeito.
///
/// **Mutação que deve sangrar:** tirar qualquer um dos três braços do `is_mute`.
#[test]
fn a_source_is_mute_without_a_file_without_range_or_at_the_bottom_of_the_slider() {
    assert!(
        AudioSource2D::default().is_mute(),
        "o default nao nomeia ficheiro nenhum — ele TEM de se declarar mudo"
    );
    assert!(
        !fonte(10.0).is_mute(),
        "uma fonte com ficheiro e alcance nao esta' muda"
    );
    let mut sem_alcance = fonte(0.0);
    assert!(sem_alcance.is_mute(), "alcance zero e' inaudivel");
    sem_alcance.max_distance = 10.0;
    sem_alcance.volume_db = -80.0;
    assert!(sem_alcance.is_mute(), "o fundo do slider e' silencio");
    // ⚠️ E o espaço em branco conta como vazio: um nome que é só espaços não abre ficheiro nenhum.
    let so_espacos = AudioSource2D {
        sound: "   ".into(),
        ..fonte(10.0)
    };
    assert!(so_espacos.is_mute(), "so' espacos nao e' um ficheiro");
}

/// **`-80 dB` é silêncio EXACTO, e `0 dB` é o som como foi gravado.**
///
/// ⚠️ A metade que importa é o **zero exacto**: um número muito pequeno somado por 64 vozes
/// ouve-se, e *«no mínimo do slider»* tem de querer dizer silêncio.
///
/// **Mutação que deve sangrar:** trocar o `return 0.0` do `db_to_linear` pela conta.
#[test]
fn the_bottom_of_the_volume_slider_is_exact_silence() {
    assert!(
        (db_to_linear(0.0) - 1.0).abs() < 1e-6,
        "0 dB tem de ser ganho 1"
    );
    assert_eq!(db_to_linear(-80.0), 0.0, "-80 dB tem de ser ZERO exacto");
    assert_eq!(db_to_linear(-120.0), 0.0, "abaixo do piso continua zero");
    // −6 dB é meia amplitude, a meio ULP da conta.
    assert!(
        (db_to_linear(-6.0) - 0.501_187).abs() < 1e-4,
        "-6 dB deveria dar ~0,5012, deu {}",
        db_to_linear(-6.0)
    );
}

// ─────────────────────────────── sem ouvinte ───────────────────────────────

/// ⭐ **Sem ouvinte, o som toca SEM POSIÇÃO** — ao centro, no volume que o artista escreveu.
///
/// ⛔ A alternativa recusada está no doc do `AudioListener2D`: usar a origem do mundo calaria uma
/// cena inteira construída longe dela, **em silêncio e sem explicação**.
///
/// **Mutação que deve sangrar:** trocar o `None => sem posição` por `None => origem`.
#[test]
fn with_no_listener_the_sound_plays_with_no_position() {
    let cfg = AudioSource2D {
        volume_db: -6.0,
        ..fonte(10.0)
    };
    // Longe da origem de propósito: se o `None` caísse na origem, isto vinha mudo.
    let s = spatialize([500.0, -300.0], None, &cfg);
    assert_eq!(s.pan, 0.0, "sem orelhas nao ha' lado");
    assert!(
        (s.gain - db_to_linear(-6.0)).abs() < 1e-6,
        "o ganho tem de ser so' o volume autorado, deu {}",
        s.gain
    );
}

// ─────────────────────────────── a queda com a distância ───────────────────────────────

/// **Fora do alcance o ganho é ZERO EXACTO** — e alcance zero é inaudível, não ilimitado.
///
/// ⚠️ **As duas metades juntas** porque a segunda é a leitura errada da primeira: um campo por
/// preencher lido como *«sem limite»* faria o valor vazio ser o mais barulhento de todos.
///
/// **Mutação que deve sangrar:** trocar o `d >= max_distance` por uma cauda que tende a zero, ou o
/// `max_distance <= 0` por `ganho cheio`.
#[test]
fn beyond_the_range_the_gain_is_exactly_zero_and_zero_range_is_inaudible() {
    let cfg = fonte(10.0);
    assert_eq!(
        spatialize([10.0, 0.0], Some([0.0, 0.0]), &cfg).gain,
        0.0,
        "na borda ja' nao se ouve"
    );
    assert_eq!(
        spatialize([0.0, 40.0], Some([0.0, 0.0]), &cfg).gain,
        0.0,
        "longe nao pode deixar uma cauda a gastar uma voz"
    );
    assert_eq!(
        spatialize([0.0, 0.0], Some([0.0, 0.0]), &fonte(0.0)).gain,
        0.0,
        "alcance zero e' alcance zero, mesmo em cima do ouvinte"
    );
}

/// **A queda é MONÓTONA e vai de cheia a zero** — varrida, não amostrada em dois pontos.
///
/// **Mutação que deve sangrar:** inverter o `1.0 - d / max_distance`.
#[test]
fn the_falloff_only_ever_goes_down() {
    let cfg = fonte(10.0);
    let ears = Some([0.0, 0.0]);
    let mut anterior = f32::INFINITY;
    for i in 0..=100 {
        let d = i as f32 * 0.1;
        let g = spatialize([d, 0.0], ears, &cfg).gain;
        assert!(
            g <= anterior + 1e-6,
            "o ganho SUBIU ao afastar: {g} em d={d} contra {anterior} antes"
        );
        anterior = g;
    }
    assert!(
        (spatialize([0.0, 0.0], ears, &cfg).gain - 1.0).abs() < 1e-6,
        "em cima do ouvinte o ganho e' cheio"
    );
    assert_eq!(anterior, 0.0, "no fim do varrimento tem de estar em zero");
}

/// **O expoente muda a FORMA da queda e não as PONTAS.**
///
/// ⚠️ É a régua que separa *«mais atenuação»* de *«mais baixo»*: as duas curvas têm de concordar em
/// cima do ouvinte e na borda, e discordar no meio. Sem a primeira metade, um expoente que
/// multiplicasse o volume passaria.
///
/// **Mutação que deve sangrar:** ignorar o `attenuation` (usar `falloff` cru).
#[test]
fn the_attenuation_exponent_bends_the_curve_without_moving_its_ends() {
    let suave = AudioSource2D {
        attenuation: 1.0,
        ..fonte(10.0)
    };
    let seco = AudioSource2D {
        attenuation: 3.0,
        ..fonte(10.0)
    };
    let ears = Some([0.0, 0.0]);
    for ponta in [0.0_f32, 10.0] {
        let a = spatialize([ponta, 0.0], ears, &suave).gain;
        let b = spatialize([ponta, 0.0], ears, &seco).gain;
        assert!(
            (a - b).abs() < 1e-6,
            "as duas curvas discordam na ponta d={ponta}: {a} contra {b}"
        );
    }
    let a = spatialize([5.0, 0.0], ears, &suave).gain;
    let b = spatialize([5.0, 0.0], ears, &seco).gain;
    assert!(
        b < a * 0.5,
        "a meio, o expoente 3 tinha de calar MUITO mais que o 1: {b} contra {a}"
    );
}

// ─────────────────────────────── o pan, e o defeito que ele cura ───────────────────────────────

/// ⭐⭐ **O pan é a DIRECÇÃO, e não a distância** — e é isso que o torna independente do alcance.
///
/// ⚠️ **Este gate nasceu de uma lei ERRADA que o irmão apanhou.** A primeira redacção normalizava o
/// pan pelo `max_distance`, e então uma fonte um metro à direita pandeava `0,1` com alcance de dez
/// metros e `0,001` com alcance de mil — *a direcção de um som passava a depender de um knob que
/// fala de volume*. É o defeito «um parâmetro com dois papéis», o mesmo do `deadzone` do Godot.
///
/// **Mutação que deve sangrar:** voltar a `dx / max_distance`.
#[test]
fn the_pan_is_the_direction_not_the_distance() {
    let ears = Some([0.0, 0.0]);
    let perto = fonte(10.0);
    let longe = fonte(1_000.0);
    for cfg in [&perto, &longe] {
        let p = spatialize([1.0, 0.0], ears, cfg).pan;
        assert!(
            (p - 1.0).abs() < 1e-6,
            "um metro a` direita e' TODO a` direita, seja qual for o alcance ({}): deu {p}",
            cfg.max_distance
        );
    }
    assert!(
        (spatialize([-3.0, 0.0], ears, &perto).pan + 1.0).abs() < 1e-6,
        "e do outro lado, simetrico"
    );
    assert!(
        spatialize([0.0, 9.0], ears, &perto).pan.abs() < 1e-6,
        "uma fonte em cima do ouvinte nao tem lado"
    );
    // A 45°, meio caminho — o seno do azimute, e não a distância.
    let diag = spatialize([3.0, 3.0], ears, &perto).pan;
    assert!(
        (diag - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5,
        "a 45 graus o pan e' o seno do azimute (~0,707), deu {diag}"
    );
    // ⚠️ E em cima do ouvinte não se inventa um lado.
    assert_eq!(
        spatialize([0.0, 0.0], ears, &perto).pan,
        0.0,
        "distancia zero nao tem direccao"
    );
}

/// **`panning_strength = 0` dá MONO e não mexe no volume.**
///
/// ⚠️ A segunda metade é a que faz o controlo valer a pena: ele existe para música de ambiente que
/// deve **atenuar** com a distância sem andar de um ouvido para o outro. Se ele calasse, seria
/// outro volume.
///
/// **Mutação que deve sangrar:** aplicar o `panning_strength` ao ganho.
#[test]
fn zero_panning_strength_is_mono_and_leaves_the_volume_alone() {
    let cheio = fonte(10.0);
    let mono = AudioSource2D {
        panning_strength: 0.0,
        ..fonte(10.0)
    };
    let ears = Some([0.0, 0.0]);
    let a = spatialize([7.0, 0.0], ears, &cheio);
    let b = spatialize([7.0, 0.0], ears, &mono);
    assert!(
        b.pan.abs() < 1e-6,
        "mono tem de ter pan zero, deu {}",
        b.pan
    );
    assert!(
        (a.gain - b.gain).abs() < 1e-6,
        "o pan nao pode mexer no ganho: {} contra {}",
        a.gain,
        b.gain
    );
}

/// ⭐⭐⭐ **ATRAVESSAR O OUVINTE NÃO FAZ O PAN SALTAR** — o *«pan pulando»* que o
/// `non_spatialized_radius` existe para curar.
///
/// # ⚠️ O gate mede a FIXTURA antes de medir o produto
///
/// Ele varre a fonte por cima do ouvinte em passos minúsculos e mede o **maior salto** de pan entre
/// dois passos. **Sem** o raio, esse salto é grande — é o defeito. **Com** ele, é pequeno. Medir só
/// o segundo deixaria o gate a passar por a cena ser mansa, e não por a lei funcionar.
///
/// **Mutação que deve sangrar:** pôr o `spread` a `1.0` sempre (ignorar o raio).
#[test]
fn crossing_the_listener_does_not_make_the_pan_jump() {
    let ears = Some([0.0, 0.0]);
    let passo = 0.01_f32;
    let percurso = |cfg: &AudioSource2D| {
        let mut pior = 0.0_f32;
        let mut anterior: Option<f32> = None;
        let mut x = -1.0_f32;
        while x <= 1.0 {
            // ⚠️ **`y` não é zero**: uma fonte que passa EXACTAMENTE pela origem tem distância zero
            // no cruzamento, e o caso interessante é o rasante — que é o que um objecto a andar de
            // facto faz.
            let p = spatialize([x, 0.05], ears, cfg).pan;
            if let Some(a) = anterior {
                pior = pior.max((p - a).abs());
            }
            anterior = Some(p);
            x += passo;
        }
        pior
    };
    let sem_raio = fonte(10.0);
    let com_raio = AudioSource2D {
        non_spatialized_radius: 0.5,
        ..fonte(10.0)
    };
    let a = percurso(&sem_raio);
    let b = percurso(&com_raio);
    // **Metade 1 — a fixtura produz o fenómeno.** Sem o raio, o azimute inverte-se num passo e o
    // pan varre uma fatia enorme da faixa. Sem esta metade o gate passaria por a cena ser mansa.
    assert!(
        a > 0.1,
        "a fixtura NAO produz o fenomeno: sem raio o pior salto foi so' {a} — este gate estaria a \
         passar por a cena ser mansa, nao por a lei funcionar"
    );
    // **Metade 2 — o raio cura-o, e por uma ordem de grandeza.**
    assert!(
        b < a * 0.2,
        "o raio nao curou o salto: {b} contra {a} sem ele"
    );
}

/// ⭐⭐ **O raio é uma MISTURA, e por isso não troca um salto por outro.**
///
/// ⚠️ **A armadilha que este gate fecha:** *«dentro do raio, pan zero»* cura o cruzamento e cria um
/// salto novo na **borda do raio**. Aqui mede-se a continuidade ao longo de todo o varrimento, e as
/// três âncoras da lei: centro no raio, cheio a dois raios, e nada entre elas a andar para trás.
///
/// **Mutação que deve sangrar:** trocar o `spread` gradual por `if d < r { 0.0 } else { 1.0 }`.
#[test]
fn the_non_spatialised_radius_blends_instead_of_switching() {
    let r = 2.0_f32;
    let cfg = AudioSource2D {
        non_spatialized_radius: r,
        ..fonte(20.0)
    };
    let ears = Some([0.0, 0.0]);
    // Na borda do raio: ainda ao centro. A DOIS raios: já com o pan cheio da geometria.
    assert!(
        spatialize([r, 0.0], ears, &cfg).pan.abs() < 1e-6,
        "na borda do raio o som ainda nao tem lado"
    );
    // ⚠️ **A dois raios a mistura acabou**, e o pan é o geométrico cru — que sobre o eixo X é
    // `dx/d = 1`, o lado inteiro. É a âncora que prova que o raio ATRASA a espacialização em vez de
    // a substituir por outra coisa.
    let cheio = spatialize([2.0 * r, 0.0], ears, &cfg).pan;
    assert!(
        (cheio - 1.0).abs() < 1e-6,
        "a dois raios o pan tinha de ser o geometrico cru (1,0), deu {cheio}"
    );
    // E nada salta pelo caminho.
    let mut pior = 0.0_f32;
    let mut anterior: Option<f32> = None;
    let mut x = 0.0_f32;
    while x <= 3.0 * r {
        let p = spatialize([x, 0.0], ears, &cfg).pan;
        if let Some(a) = anterior {
            pior = pior.max((p - a).abs());
        }
        anterior = Some(p);
        x += 0.01;
    }
    assert!(
        pior < 0.01,
        "o pan saltou {pior} num passo de 0,01 m — a mistura virou um degrau"
    );
}

/// ⭐ **Raio ZERO é BYTE-IDÊNTICO a não haver raio nenhum.**
///
/// ⚠️ É a cerca que impede a feature nova de mudar o que já existia: um `0` no campo tem de deixar
/// a lei exactamente como ela era antes de o campo existir.
///
/// **Mutação que deve sangrar:** tirar o ramo `> 0.0` e deixar a conta correr com `r = 0`
/// (divisão por zero ⇒ `NaN`/`inf`).
#[test]
fn a_zero_radius_is_bit_identical_to_having_no_radius() {
    let cfg = AudioSource2D {
        non_spatialized_radius: 0.0,
        ..fonte(10.0)
    };
    let ears = Some([0.0, 0.0]);
    for i in -50..=50 {
        let x = i as f32 * 0.1;
        let p = spatialize([x, 0.3], ears, &cfg).pan;
        let cru = x / x.hypot(0.3);
        assert_eq!(
            p.to_bits(),
            cru.to_bits(),
            "com raio zero o pan em x={x} tinha de ser o geometrico cru: {p} contra {cru}"
        );
    }
}

// ─────────────────────────────── as orelhas ───────────────────────────────

/// ⭐⭐ **Com vários ouvintes ganha o de menor `StableId`, nunca o primeiro da query.**
///
/// ⚠️ **A ordem de uma query é a ordem dos ARQUÉTIPOS e muda quando um componente é inserido** — um
/// ouvinte que troca de objecto porque alguém anexou uma sprite noutro sítio é um defeito que não
/// se reproduz. É a mesma lei que a resolução da tabela de acções paga.
///
/// **Mutação que deve sangrar:** trocar o `min_by_key` por `next()`.
#[test]
fn the_ears_are_the_lowest_stable_id_never_the_query_order() {
    let mut w = World::new();
    let primeiro = w.spawn((Transform::default(), AudioListener2D)).id();
    let segundo = w.spawn((Transform::default(), AudioListener2D)).id();
    assign_missing_stable_ids(&mut w);
    // Quem tem o id MENOR é quem manda — e é escrito à mão para o gate não depender da ordem em
    // que o `assign_missing_stable_ids` os visitou.
    w.entity_mut(primeiro).insert(StableId(7));
    w.entity_mut(segundo).insert(StableId(3));
    assert_eq!(
        listener_of(&mut w),
        Some(segundo),
        "ganhou o da query em vez do de menor id"
    );
    assert_eq!(
        listener_count(&mut w),
        2,
        "a contagem e' o que o painel diz"
    );
    // ⚠️ Inserir um componente noutra entidade muda a ordem dos arquétipos — e não pode mudar isto.
    w.entity_mut(primeiro).insert(crate::Name::new("ouvido"));
    assert_eq!(
        listener_of(&mut w),
        Some(segundo),
        "a resposta mudou porque alguem anexou um componente noutro sitio"
    );
}

/// **Uma cena sem ouvinte responde `None`, e não a origem.**
#[test]
fn a_scene_with_no_listener_has_no_ears() {
    let mut w = World::new();
    w.spawn((Transform::default(), fonte(10.0)));
    assign_missing_stable_ids(&mut w);
    assert_eq!(listener_of(&mut w), None);
    assert_eq!(listener_count(&mut w), 0);
}

// ─────────────────────────────── o barramento ───────────────────────────────

/// **A POSIÇÃO em `ALL` é a tag** — nos dois sentidos, e uma tag desconhecida cai no default.
///
/// ⚠️ A última metade não é conveniência: a tag chega de um ficheiro, e recusar o load inteiro por
/// causa de um barramento trocaria um som no sítio errado por um projecto que não abre.
///
/// **Mutação que deve sangrar:** reordenar o `ALL` sem mexer no `tag()`.
#[test]
fn the_bus_tag_is_its_position_in_the_list() {
    for (i, b) in AudioBus::ALL.iter().enumerate() {
        assert_eq!(
            b.tag() as usize,
            i,
            "{:?} diz-se da posicao {} e esta' na {i}",
            b,
            b.tag()
        );
        assert_eq!(AudioBus::from_tag(b.tag()), *b, "a volta nao fecha");
    }
    assert_eq!(
        AudioBus::from_tag(200),
        AudioBus::default(),
        "uma tag de um ficheiro futuro tem de cair no default, nao recusar"
    );
}

/// ⛔ **O barramento do CHROME não é alcançável a partir da cena.**
///
/// ⚠️ Ele é a voz do editor — os quatro sons sintetizados do D1 —, e uma preferência do utilizador
/// (`ui_sound=0`) desliga-a. Um som de JOGO encaminhado para lá seria calado por uma preferência
/// que fala de outra coisa. *Um controlo cujo efeito depende de uma preferência que nomeia outro
/// assunto é um controlo que mente.*
///
/// **Mutação que deve sangrar:** acrescentar um `Ui` ao `AudioBus`.
#[test]
fn the_chrome_bus_is_not_reachable_from_the_scene() {
    for b in AudioBus::ALL {
        assert!(
            !b.label().eq_ignore_ascii_case("ui"),
            "o barramento do chrome ficou alcancavel pela cena"
        );
    }
}

// ─────────────────────────────── o default ───────────────────────────────

/// ⛔ **Anexar o componente NÃO faz barulho** — `autoplay` nasce desligado.
///
/// ⚠️ É a mesma lei do `+` da tabela de acções: *um default que disparasse em alguma coisa faria
/// anexar um componente MUDAR a cena*.
///
/// **Mutação que deve sangrar:** pôr `autoplay: true` no `Default`.
#[test]
fn attaching_the_component_makes_no_sound() {
    let d = AudioSource2D::default();
    assert!(!d.autoplay, "anexar nao pode comecar a tocar");
    assert!(d.never_starts_by_itself());
    assert_eq!(d.max_polyphony, 1, "o caso comum SUBSTITUI a voz anterior");
    assert_eq!(d.bus(), AudioBus::Sfx, "um som de cena e' um efeito");
    assert!(
        d.max_distance > 0.0 && d.max_distance < AUDIO_MAX_DISTANCE_M,
        "o default nao pode ser o teto: isso torna o campo inerte por omissao"
    );
}
