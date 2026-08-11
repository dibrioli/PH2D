//! Gates do **Snake Hook** — a segunda lei, e o que ela mantém da primeira.
//!
//! Filho do MESMO pai que os outros gates de verbo, então `use super::*`
//! alcança o `sphere` compartilhado.
//!
//! ⚠️ **A fixture aqui é o LAÇO DO SHELL, não um dab solto.** O Hook é uma soma
//! sobre a lista de dabs, então um único dab não contém o fenômeno: ele mede a
//! mesma coisa que um Grab de um evento. Todo gate abaixo arrasta um caminho.

use super::*;

fn hook(radius: f32) -> Brush {
    Brush {
        verb: Verb::SnakeHook,
        radius,
        strength: 1.0,
        ..Brush::default()
    }
}

/// O laço que o shell roda, em unidades de MUNDO: um caminho reto de `len`
/// entregue em `events` eventos, cada um percorrido pelo walk do espaçamento.
///
/// Devolve a malha e as posições de antes.
fn drag_hook(
    radius: f32,
    len: f32,
    events: usize,
    brush: &Brush,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let at = [0.0, 0.0, 1.0];
    let spacing = crate::min_spacing(radius);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut anchor = [0.0f32, 0.0];
    for e in 1..=events {
        let to = [len * e as f32 / events as f32, 0.0];
        let Some(steps) = crate::walk(anchor, to, spacing) else {
            continue;
        };
        let mut prev = anchor;
        for step in steps {
            stroke.dab(
                &mut mesh,
                brush,
                &Dab::hooking(
                    [at[0] + step[0], at[1], at[2]],
                    radius,
                    [0.0, 0.0, -1.0],
                    [step[0] - prev[0], step[1] - prev[1], 0.0],
                ),
                Symmetry::default(),
            );
            prev = step;
        }
        // ⚠️ **A âncora é o ÚLTIMO DAB, e este driver é uma CÓPIA do do shell.**
        // Ele nasceu escrevendo `= to`, e é por isso que este gate ficou
        // VERMELHO na wave do passo exato enquanto o produto ficava certo: um
        // gate com driver próprio mede a re-expressão, não o produto. O que o
        // mantém honesto é perguntar [`crate::Walk::anchor`] — a mesma porta que
        // o `sculpt3d_input` pergunta.
        anchor = steps.anchor();
    }
    (mesh, before)
}

/// **A entrega da wave, e o oráculo é a diferença entre as DUAS LEIS.**
///
/// Ida e volta pelo mesmo caminho: o [`crate::Grip::Hold`] devolve o barro ao
/// lugar (o alvo dele é função do `pre`, e o `pre` não mudou) e o
/// [`crate::Grip::Hook`] deixa a ponta lá, porque ele TRANSPORTOU matéria. Não é
/// um descuido a corrigir — é o que esticar significa, e é a única coisa que um
/// artista precisa saber para escolher entre as duas teclas.
///
/// ⚠️ **Medir só o Hook seria verde por vácuo**: qualquer verbo que mexa na
/// malha deixa resíduo se ninguém provar que o irmão dele NÃO deixa. O Grab é o
/// controle, e ele mede **0,00000**.
#[test]
fn the_hook_leaves_a_spike_where_the_grab_gives_the_clay_back() {
    let (radius, reach) = (0.4f32, 0.6f32);
    let at = [0.0, 0.0, 1.0];
    let eye = [0.0, 0.0, -1.0];

    let mut left = Vec::new();
    for verb in [Verb::Move, Verb::SnakeHook] {
        let mut mesh = sphere();
        let before = snapshot(&mesh);
        let brush = Brush {
            verb,
            radius,
            strength: 1.0,
            ..Brush::default()
        };
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        // Sobe até `reach` e volta a zero, em passos iguais.
        let mut prev = 0.0f32;
        let ups = (0..=12).map(|k| k as f32 * reach / 12.0);
        let downs = (0..12).rev().map(|k| k as f32 * reach / 12.0);
        for y in ups.chain(downs) {
            let dab = match verb {
                Verb::Move => Dab::pulling(at, radius, eye, [0.0, y, 0.0]),
                _ => Dab::hooking([at[0], at[1] + y, at[2]], radius, eye, [0.0, y - prev, 0.0]),
            };
            stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
            prev = y;
        }
        left.push(max_shift(&before, &mesh));
    }
    let (grab, hooked) = (left[0], left[1]);
    assert!(
        grab < 1e-4,
        "o Grab tem de devolver o barro ao lugar, e sobrou {grab}"
    );
    assert!(
        hooked > 0.25,
        "o Hook tem de deixar um espinho, e sobrou {hooked}"
    );
}

/// **A metade da lei do traço que o revezamento MANTÉM**, e ela não é de graça:
/// o Hook é uma soma sobre a lista de dabs, então sem o walk fixando o passo na
/// geometria arrastar devagar esticaria mais que arrastar rápido pelo mesmo
/// traçado — a doença que este módulo inteiro existe para não ter.
///
/// ⚠️ **A barra é MEDIDA e a promessa é mais fraca que a do envelope**, de
/// propósito: uma integral de linha *converge* com o espaçamento, não é exata em
/// qualquer um. Varrido de 1 a 128 eventos sobre o mesmo caminho, a ponta viaja
/// **1,19475 · 1,19475 · 1,19475 · 1,19792 · 1,19792 · 1,17583** — espalhamento
/// de **1,6%**. A barra fica em 5%: folgada o bastante para não flakar na
/// fronteira do `floor` do walk, apertada o bastante para que um dab por EVENTO
/// (o produto sem o walk) não passe.
#[test]
fn the_spike_is_a_fact_of_the_path_not_of_the_pointer_polling_rate() {
    let (radius, len) = (0.4f32, 1.2f32);
    let brush = hook(radius);
    let mut seen = Vec::new();
    for events in [1usize, 2, 4, 8, 32, 128] {
        let (mesh, before) = drag_hook(radius, len, events, &brush);
        seen.push(max_shift(&before, &mesh));
    }
    let lo = seen.iter().copied().fold(f32::MAX, f32::min);
    let hi = seen.iter().copied().fold(0.0f32, f32::max);
    assert!(
        hi <= lo * 1.05,
        "a ponta viajou {seen:?} nas taxas 1..128 — espalhamento de {:.1}%",
        (hi / lo - 1.0) * 100.0
    );
    // O CONTROLE, sem o qual o gate não pode falhar: um dab por evento (o
    // produto antes do walk) faz a ponta depender da taxa de polling, porque o
    // número de parcelas da soma passa a ser o número de eventos.
    let naive = |events: usize| {
        let mut mesh = sphere();
        let before = snapshot(&mesh);
        let at = [0.0, 0.0, 1.0];
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let mut prev = 0.0f32;
        for e in 1..=events {
            let x = len * e as f32 / events as f32;
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::hooking(
                    [at[0] + x, at[1], at[2]],
                    radius,
                    [0.0, 0.0, -1.0],
                    [x - prev, 0.0, 0.0],
                ),
                Symmetry::default(),
            );
            prev = x;
        }
        max_shift(&before, &mesh)
    };
    let (few, many) = (naive(2), naive(64));
    assert!(
        many > few * 1.30,
        "o controle tem de conter o fenômeno: sem o walk, 2 eventos deram {few} \
         e 64 deram {many}"
    );
}

/// **O undo continua trivial na segunda lei**, e é a propriedade que decidiu o
/// desenho: `base` é o `pre` congelado de todo vértice que a pegada tocou em
/// QUALQUER momento do gesto, mesmo os que ela já deixou para trás.
///
/// ⚠️ É aqui que a janela do Hook difere da do Grab — ela é MAIOR que a pegada
/// atual, porque a pegada anda e deixa gente para trás. Um undo que só
/// conhecesse a pegada de agora devolveria a ponta e deixaria o rastro.
#[test]
fn undoing_a_hook_restores_every_vertex_the_moving_footprint_ever_touched() {
    let radius = 0.4f32;
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let at = [0.0, 0.0, 1.0];
    let brush = hook(radius);
    let spacing = crate::min_spacing(radius);

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut prev = [0.0f32, 0.0];
    for step in crate::walk([0.0, 0.0], [1.2, 0.0], spacing).expect("anda") {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::hooking(
                [at[0] + step[0], at[1], at[2]],
                radius,
                [0.0, 0.0, -1.0],
                [step[0] - prev[0], step[1] - prev[1], 0.0],
            ),
            Symmetry::default(),
        );
        prev = step;
    }
    assert!(
        max_shift(&before, &mesh) > 0.5,
        "a fixture tem de ter esculpido de fato"
    );
    let touched = stroke.touched().to_vec();
    let bases = stroke.base_positions().to_vec();
    // ⚠️ **A janela é medida contra UMA PEGADA, não contra um número que eu
    // escolhi.** A primeira versão cravava `> 200` e falhou sobre produto
    // correto: a esfera desta suíte é grosseira e o gesto inteiro toca 66
    // vértices. O que o gate quer dizer é *a janela é maior que a pegada
    // ATUAL*, e isso é uma razão — imune à densidade da fixture.
    //
    // ⚠️ **E ela NÃO cresce com o comprimento do caminho, que era a minha
    // segunda suposição errada.** Medido: 66 tocados contra 39 de uma pegada, ou
    // **1,69×**, para um caminho três vezes o raio. O motivo é o que o Hook É —
    // ele arrasta uma ESFERA pelo espaço, não um acerto de superfície, então
    // depois de alguns passos o centro já saiu do modelo e a consulta só
    // devolve os vértices que o próprio gesto levou consigo. A janela é a
    // pegada inicial mais quem escorregou para dentro dela enquanto ela partia.
    let mut one = SculptStroke::default();
    let mut probe = sphere();
    one.begin(&probe);
    one.dab(
        &mut probe,
        &brush,
        &Dab::hooking(at, radius, [0.0, 0.0, -1.0], [0.05, 0.0, 0.0]),
        Symmetry::default(),
    );
    let footprint = one.touched().len();
    assert!(
        touched.len() * 2 > footprint * 3,
        "a janela do Hook é maior que a pegada atual: {} tocados contra \
         {footprint} de uma pegada (medido 66 contra 39)",
        touched.len()
    );

    let out = mesh.positions_mut();
    for (&v, p) in touched.iter().zip(&bases) {
        out[v as usize] = *p;
    }
    let worst = max_shift(&before, &mesh);
    assert!(
        worst < 1e-6,
        "desfazer pelo `base` tem de devolver a malha exata, e sobrou {worst}"
    );
}

/// A máscara protege o que ela protege, também na segunda lei — ela entra pelo
/// `keep` do `w`, que é o mesmo peso que o revezamento usa para o incremento.
#[test]
fn the_mask_protects_against_the_hook_too() {
    let radius = 0.4f32;
    let brush = hook(radius);
    let (free, before) = drag_hook(radius, 1.2, 8, &brush);
    let moved_free = max_shift(&before, &free);

    let mut mesh = sphere();
    // Protege TUDO e arrasta por cima.
    let n = mesh.vert_count();
    let out = mesh.masks_mut();
    for m in &mut out[..n] {
        *m = 1.0;
    }
    let before = snapshot(&mesh);
    let at = [0.0, 0.0, 1.0];
    let spacing = crate::min_spacing(radius);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut prev = [0.0f32, 0.0];
    for step in crate::walk([0.0, 0.0], [1.2, 0.0], spacing).expect("anda") {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::hooking(
                [at[0] + step[0], at[1], at[2]],
                radius,
                [0.0, 0.0, -1.0],
                [step[0] - prev[0], step[1] - prev[1], 0.0],
            ),
            Symmetry::default(),
        );
        prev = step;
    }
    assert!(
        moved_free > 0.5,
        "o controle sem máscara tem de esticar: {moved_free}"
    );
    let masked = max_shift(&before, &mesh);
    assert!(
        masked < 1e-6,
        "sob máscara cheia nada pode andar, e andou {masked}"
    );
}

/// ⚠️ **O `accum` do revezamento vale 1 por construção**, e é isso que o faz
/// caber no MESMO aplicador (`lerp(base, target, 1) == target`). O gate mede a
/// consequência observável: o segundo dab de um gesto **soma** ao primeiro em
/// vez de ser engolido pelo early-out do envelope.
///
/// Sem a exceção do [`Verb::pulls`], `w <= accum` seria verdade sempre (todo
/// peso é ≤ 1) e o espinho teria exatamente o tamanho de um dab.
#[test]
fn every_step_of_the_relay_adds_to_the_one_before_it() {
    let radius = 0.4f32;
    let brush = hook(radius);
    let mut seen = Vec::new();
    for steps in [1usize, 2, 4, 8] {
        let mut mesh = sphere();
        let before = snapshot(&mesh);
        let at = [0.0, 0.0, 1.0];
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for _ in 0..steps {
            // O MESMO incremento, dab após dab: um envelope saturaria no
            // primeiro, e o revezamento tem de somar.
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::hooking(at, radius, [0.0, 0.0, -1.0], [0.0, 0.05, 0.0]),
                Symmetry::default(),
            );
        }
        seen.push(max_shift(&before, &mesh));
    }
    for k in 1..seen.len() {
        assert!(
            seen[k] > seen[k - 1] * 1.5,
            "cada passo tem de acrescentar ao anterior, e mediu {seen:?}"
        );
    }
}
