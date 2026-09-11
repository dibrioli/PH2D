//! **O GIZMO DA VIEWPORT DA ESCULTURA, medido.**
//!
//! ⚠️ **Estes gates NÃO precisam de GPU**, e é o desenho: tudo o que eles
//! afirmam é lei de câmera — as vistas nomeadas, o reconhecimento, a base que
//! alimenta as bolas. O que exige uma cena (o pen-down, o arrasto) vive no
//! `sculpt3d_filter_tests`/`sculpt3d_transform_tests`, com `gpu_or_skip!`.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::navball
//! ```

use super::{aim_of, named_view};
use crate::field3d_views::Standard;
use ph2d_mesh_render::Camera3d;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

/// A câmera apontada àquela vista — pela MESMA porta que o produto usa
/// ([`aim_of`] + [`Camera3d::aim`]).
fn aimed(v: Standard) -> Camera3d {
    let mut c = Camera3d::default();
    let (y, p) = aim_of(v);
    c.aim(y, p);
    c
}

/// ⭐⭐⭐ **CADA VISTA PÕE O OLHO NO EIXO QUE ELA NOMEIA.**
///
/// ⚠️ **A régua é o EIXO, nunca a aritmética que o produziu.** O `(yaw, pitch)`
/// é a porta da casa para um enquadramento nomeado, e um sinal trocado ali dá
/// uma vista que se chama *Frente* e mostra as costas — sem erro de compilação
/// e sem que nenhum gate de ângulo veja. É a mesma lei que o
/// `field3d_views::Standard::eye_axis` já escreve para o módulo vizinho, e a
/// lista de eixos é a **dele**: as duas metades do app não podem discordar
/// sobre o que é a frente.
///
/// ⛔ **O topo e a base NÃO batem exactamente, e o número é a trava do polo**
/// ([`Camera3d::clamp_pitch`]): a câmera guarda `π/2 − 0,01`, que é `0,57°` de
/// desvio. Uma barra apertada aqui seria uma barra sobre uma degenerescência
/// geométrica, não sobre o produto.
#[test]
fn cada_vista_poe_o_olho_no_eixo_que_ela_nomeia() {
    // `cos(0,01) = 0,99995` — a trava do polo, e nada mais folgado que ela.
    let barra = (0.011_f32).cos();
    for v in Standard::ALL {
        let cam = aimed(v);
        let olho: [f32; 3] = cam.view_axis().into();
        let quer = v.eye_axis();
        let d = dot(olho, quer);
        println!("{:>28} olho {olho:?} quer {quer:?} -> cos {d:.6}", v.key());
        assert!(
            d > barra,
            "a vista {} poe o olho em {olho:?} e devia po^-lo em {quer:?} (cos {d:.6}) -- \
             um sinal trocado da' uma vista que se chama assim e mostra o lado oposto",
            v.key()
        );
    }
}

/// ⭐⭐ **UMA VISTA NOMEADA É RECONHECIDA, E O MENOR ARRASTO SOLTA-A.**
///
/// ⚠️ **As duas metades são o gate.** Só a primeira deixaria passar uma
/// tolerância enorme (tudo seria *Frente*); só a segunda deixaria passar uma
/// tolerância zero (nada seria nada). O par prende a barra entre o ruído de
/// `f32` e o menor gesto que existe — **um pixel**, que nesta cena vale
/// `0,01 rad`.
#[test]
fn uma_vista_e_reconhecida_e_o_menor_arrasto_solta_a() {
    for v in Standard::ALL {
        let cam = aimed(v);
        assert_eq!(
            named_view(&cam),
            Some(v),
            "a camera acabou de ser APONTADA a {} e o reconhecedor nao a ve^ -- \
             ele esta' a comparar com o ideal em vez do valor que a camera guarda",
            v.key()
        );
        // Um pixel de arrasto, na horizontal e na vertical.
        for (dx, dy) in [
            (super::ORBIT_RAD_PER_PX, 0.0),
            (0.0, super::ORBIT_RAD_PER_PX),
        ] {
            let mut solta = aimed(v);
            solta.orbit(dx, dy);
            // ⚠️ **O topo e a base saem pela horizontal e NÃO pela vertical**: o
            // `pitch` já está preso ao limite, então somar-lhe não o move. *Uma
            // trava é também um sítio onde um gesto não tem efeito.*
            let preso = matches!(v, Standard::Top | Standard::Bottom) && dy != 0.0;
            if preso {
                continue;
            }
            assert_ne!(
                named_view(&solta),
                Some(v),
                "um pixel de arrasto ({dx}, {dy}) nao soltou a vista {} -- a tolerancia do \
                 reconhecimento e' maior que o menor gesto, e o chip fica aceso a mentir",
                v.key()
            );
        }
    }
}

/// ⭐⭐⭐ **A BASE QUE ALIMENTA AS BOLAS É UMA BASE.**
///
/// ⚠️ **Sem isto o widget mente em silêncio.** As bolas caem em
/// `(d·direita, −d·cima)` com profundidade `d·frente`, e uma base que não fosse
/// ortonormal punha eixos perpendiculares do mundo a sobrepor-se na tela — o
/// artista perde exactamente a informação que o widget existe para dar.
///
/// ⚠️ **E a orientação importa**: `direita × cima` tem de apontar para o
/// OBSERVADOR, senão a bola do eixo que aponta para a câmera é pintada por
/// baixo da que aponta para trás, e o gizmo lê ao contrário.
#[test]
fn a_base_do_gizmo_e_ortonormal_e_aponta_ao_observador() {
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1].mul_add(b[2], -(a[2] * b[1])),
            a[2].mul_add(b[0], -(a[0] * b[2])),
            a[0].mul_add(b[1], -(a[1] * b[0])),
        ]
    }
    // Enquadramentos variados, o de omissão incluído — e os polos, que é onde
    // uma base derivada de um produto vectorial degenera.
    for (yaw, pitch) in [
        (0.5, 0.4),
        (0.0, 0.0),
        (2.7, -1.1),
        (-1.3, 1.5),
        (0.0, std::f32::consts::FRAC_PI_2),
        (0.0, -std::f32::consts::FRAC_PI_2),
    ] {
        let mut cam = Camera3d::default();
        cam.aim(yaw, pitch);
        let (r, u) = cam.screen_basis();
        let (right, up): ([f32; 3], [f32; 3]) = (r.into(), u.into());
        let fwd: [f32; 3] = cam.view_axis().into();
        for (nome, v) in [("direita", right), ("cima", up), ("frente", fwd)] {
            let n = dot(v, v).sqrt();
            assert!(
                (n - 1.0).abs() < 1e-4,
                "a {nome} de ({yaw}, {pitch}) nao e' unitaria: |v| = {n:.6}"
            );
        }
        assert!(
            dot(right, up).abs() < 1e-4,
            "direita e cima nao sao perpendiculares em ({yaw}, {pitch})"
        );
        assert!(
            dot(right, fwd).abs() < 1e-4,
            "direita e frente nao sao perpendiculares em ({yaw}, {pitch})"
        );
        assert!(
            dot(up, fwd).abs() < 1e-4,
            "cima e frente nao sao perpendiculares em ({yaw}, {pitch})"
        );
        let d = dot(cross(right, up), fwd);
        assert!(
            d > 0.99,
            "em ({yaw}, {pitch}) a base esta' INVERTIDA (direita x cima . frente = {d:.4}): as \
             bolas da frente seriam pintadas por baixo das de tras"
        );
    }
}

/// ⭐⭐ **O `Numpad` DA ESCULTURA E O DA MODELAGEM CONCORDAM.**
///
/// ⚠️ **Um censo e não uma inspecção**: a tabela de teclas é a do módulo
/// vizinho, e este gate afirma que a escultura a lê inteira — se alguém lá
/// acrescentar uma vista, a lista de eixos cresce nos dois sítios ou este gate
/// nomeia a que ficou para trás.
#[test]
fn as_seis_teclas_de_vista_chegam_todas_a_uma_vista() {
    use winit::keyboard::KeyCode as K;
    let mut vistas = Vec::new();
    for code in [K::Numpad1, K::Numpad3, K::Numpad7] {
        for ctrl in [false, true] {
            let v = crate::field3d_views::view_for_key(code, ctrl)
                .unwrap_or_else(|| panic!("{code:?} (ctrl {ctrl}) nao mapeia vista nenhuma"));
            // A tecla tem de produzir uma câmera que se reconhece a si mesma.
            assert_eq!(named_view(&aimed(v)), Some(v));
            vistas.push(v);
        }
    }
    assert_eq!(
        vistas.len(),
        Standard::ALL.len(),
        "as teclas alcancam {} vistas e a lista tem {} -- uma vista sem tecla e' inalcancavel \
         pelo teclado, e uma tecla a mais aponta duas vezes para a mesma",
        vistas.len(),
        Standard::ALL.len()
    );
}

/// ⭐⭐⭐ **UM CLIQUE NA BOLA MUDA A CÂMERA — e um ARRASTO não.**
///
/// ⛔⛔ **Um widget não está pronto quando pinta: está pronto quando um teste
/// CLICA nele.** Esta casa pagou essa lição no `seam_bool` do vetor, onde os
/// quatro chips estavam pintados, indexados e **mortos sob o ponteiro**, e o
/// gate de registo estava verde. Os três gates acima medem a LEI (os eixos, a
/// base, o reconhecimento) e nenhum deles toca no gesto.
///
/// ⚠️ **As duas metades são o gate**, porque elas competem pelo mesmo botão: se
/// o clique saltasse já no pen-**down**, todo arrasto começaria com um corte de
/// câmera — o widget ficaria «a funcionar» e impossível de usar para orbitar,
/// que é o gesto que a própria pesquisa da Autodesk mediu como o rápido.
#[test]
fn um_clique_na_bola_muda_a_camera_e_um_arrasto_nao() {
    use ph2d_editor::zones::Rect;
    use ph2d_mesh::shapes::uv_sphere;

    let gpu = match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
        Ok(g) => g,
        Err(_) => {
            eprintln!("no GPU adapter on this machine — nothing to assert");
            return;
        }
    };
    let mut s = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, uv_sphere(12, 18, 1.0), 1.0);
    // A área é a do canvas e o `safe` é ela inteira — sem painéis, o widget vai
    // para a quina de cima à direita.
    let area = Rect::new(0.0, 0.0, 900.0, 700.0);
    s.note_canvas(area);
    s.note_nav(area, (0.0, 0.0));

    // Onde a bola do TOPO está, agora, segundo a própria lei.
    let alvo = Standard::Top;
    let bola = s
        .navball(area, area)
        .into_iter()
        .find(|b| b.view == alvo)
        .expect("as seis bolas sao sempre produzidas");
    let (bx, by) = (area.x + bola.at[0], area.y + bola.at[1]);

    // (1) — o CLIQUE: down e up no mesmo pixel.
    assert!(
        s.nav_pointer_down(bx, by),
        "o pen-down sobre a bola {} nao foi tomado pelo gizmo -- ele esta' MORTO sob o ponteiro",
        alvo.key()
    );
    assert!(s.nav_pointer_up(), "o pen-up nao foi tomado pelo gizmo");
    assert_eq!(
        named_view(&s.camera),
        Some(alvo),
        "clicar a bola {} nao levou a camera a` vista dela",
        alvo.key()
    );

    // (2) — o ARRASTO: começa na MESMA bola, mas anda.
    //
    // ⚠️⚠️ **A régua é o VALOR do yaw, e não «mudou»** — e a diferença é uma
    // mutação que sobrevivia: fazer o pen-**down** já saltar para a vista
    // deixaria «mudou» verde, porque a órbita que vem a seguir muda-o na mesma.
    // O que separa as duas é *de ONDE a órbita partiu*: se o gizmo saltou, ela
    // parte do `yaw` da bola; se não saltou, parte do `yaw` que a câmera tinha.
    // ⇒ a fixture começa numa orientação LIVRE, longe de qualquer vista
    // nomeada, e o gate exige o valor exacto.
    let livre = 0.9_f32;
    s.camera.aim(livre, 0.3);
    const PASSOS: i32 = 10;
    const PX: f32 = 6.0;
    assert!(s.nav_pointer_down(bx, by));
    for k in 1..=PASSOS {
        assert!(
            s.nav_pointer_move(PX.mul_add(k as f32, bx), by),
            "o gizmo largou o arrasto a meio -- a captura esta' partida"
        );
    }
    assert!(s.nav_pointer_up());
    // O sentido é o da peça: arrastar para a direita pede `yaw -= dx`.
    let esperado = (PX * PASSOS as f32).mul_add(-super::ORBIT_RAD_PER_PX, livre);
    println!(
        "clique -> {alvo:?} | arrasto: yaw {livre} -> {} (esperado {esperado})",
        s.camera.yaw
    );
    assert!(
        (s.camera.yaw - esperado).abs() < 1e-4,
        "o arrasto no gizmo deixou o yaw em {} e a orbita a partir de {livre} da' {esperado} --          se a diferenca for o yaw de {}, o pen-DOWN saltou para a vista e todo arrasto comeca          com um corte de camera",
        s.camera.yaw,
        alvo.key()
    );
    assert_eq!(
        named_view(&s.camera),
        None,
        "um ARRASTO no gizmo pousou numa vista nomeada -- ele nao devia saltar para nenhuma"
    );
}

/// ⭐⭐ **O DESPACHO DO PONTEIRO PERGUNTA AO GIZMO — nas TRÊS metades.**
///
/// ⚠️ **Censo de FONTE, e ele existe porque a costura que interessa não é
/// alcançável de um teste**: o `App::sculpt3d_pointer_down` precisa de uma
/// surface de janela real. O que se pode afirmar sem ela é que as três portas do
/// gizmo são chamadas de dentro das três portas do ponteiro — *e as três são
/// obrigatórias*: sem o `down` ele está morto, sem o `move` o arrasto larga no
/// primeiro pixel, e sem o `up` um clique nunca vira vista.
#[test]
fn as_tres_portas_do_gizmo_sao_chamadas_pelo_despacho_do_ponteiro() {
    // ⚠️ **AS DUAS METADES DO DESPACHO**: o pen-down vive num irmão desde que o
    // tecto de LOC obrigou ao corte. *Um censo que nomeia um FICHEIRO envelhece
    // com o primeiro corte — e um censo cego lê-se como aprovado.*
    let fonte = concat!(
        include_str!("input_down.rs"),
        "\n",
        include_str!("input.rs")
    );
    // ⚠️⚠️ **E em 2026-09-11 (W2/L3-A2) as três portas deixaram de ter a MESMA FORMA:** o
    // `move` e o `up` só precisavam da cena e viraram funções LIVRES (`fn pointer_move`, à
    // coluna 0), enquanto o `down` arbitra quem fica com o gesto — lê `gfx`, `last_pointer` e
    // `modifiers` — e continua método. *O censo tem de aceitar as duas formas*, e é por isso
    // que o terminador da janela abaixo procura **as duas indentações**: a heurística antiga
    // (`"\n    pub(crate) fn "`) nunca casaria depois de uma função livre, e a janela engoliria
    // o resto do ficheiro — um censo que mede DEMAIS lê-se tão aprovado como um que mede nada.
    for (porta, dentro) in [
        ("nav_pointer_down", "fn sculpt3d_pointer_down"),
        ("nav_pointer_move", "fn pointer_move"),
        ("nav_pointer_up", "fn pointer_up"),
    ] {
        let Some(inicio) = fonte.find(dentro) else {
            panic!("o despacho `{dentro}` mudou de nome ou de ficheiro -- este censo ficou cego");
        };
        // A janela é da função até à seguinte, em qualquer das duas formas.
        let resto = &fonte[inicio..];
        let fim = resto[1..]
            .find("\n    pub(crate) fn ")
            .into_iter()
            .chain(resto[1..].find("\npub(crate) fn "))
            .min()
            .map_or(resto.len(), |i| i + 1);
        assert!(
            resto[..fim].contains(porta),
            "`{dentro}` nao chama `{porta}` -- o gizmo de navegacao fica pintado e MORTO nessa \
             metade do gesto"
        );
    }
}
