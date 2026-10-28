//! A ferramenta do arranjo do HUD pelo gesto real: press → arrasto → soltar → quadro, sobre o retrato que o passe
//! publica (câmera identidade: mundo = ecrã).

use super::{Par, frame, plano, press};
use crate::state::VecState;
use ph2d_ecs::hud_arranjo::{Eixo, FilhoVisto, Forca, Lado, Relacao, UiArranjo, Visto};
use ph2d_ecs::{Entity, SimWorld, StableId, UiCanvas};
use ph2d_panel_vector::state_hud::{self, PedidoHud, TipoSeta};
use ph2d_tool_vector::DrawMode;
use ph2d_vector::Affine;

const PONT: StableId = StableId(11);
const MOEDA: StableId = StableId(12);
const MAPA: StableId = StableId(13);

/// Pontuação (texto) à esquerda, a moeda 0,5 à direita dela, o minimapa longe; escala 2 (mundo por unidade local).
fn mesa() -> (SimWorld, VecState, Entity) {
    let mut sim = SimWorld::default();
    let canvas = sim.world_mut().spawn(UiCanvas::default()).id();
    let filho = |n: u64, id, lo: [f64; 2], hi: [f64; 2], texto| FilhoVisto {
        kid: Entity::from_bits(n),
        id,
        caixa: (lo, hi),
        texto,
    };
    let mut vec = VecState::default();
    vec.draw_config.mode = DrawMode::HudLayout;
    vec.hud_visto = vec![Visto {
        canvas,
        escala: [2.0, 2.0],
        lacos: Vec::new(),
        filhos: vec![
            filho(1 << 32 | 1, PONT, [0.0, 0.0], [40.0, 10.0], true),
            filho(1 << 32 | 2, MOEDA, [41.0, 1.0], [49.0, 9.0], false),
            filho(1 << 32 | 3, MAPA, [200.0, -20.0], [240.0, 10.0], false),
        ],
    }];
    desenha(&vec);
    (sim, vec, canvas)
}

fn desenha(vec: &VecState) {
    *vec.hud_tool.plano.borrow_mut() = Some(plano(&vec.hud_visto, Affine::IDENTITY, 9.0));
}

/// O gesto inteiro de `de` a `para` (mundo), e o quadro que o fecha.
fn puxa(sim: &mut SimWorld, vec: &mut VecState, de: [f64; 2], para: [f64; 2]) {
    assert!(press(vec, de), "o press é sempre da ferramenta");
    assert!(
        super::drag(vec, para),
        "com uma seta puxada o arrasto é dela"
    );
    vec.shape_handles.release();
    frame(sim, vec);
}

fn lacos(sim: &SimWorld, canvas: Entity) -> Vec<(Relacao<StableId>, Forca)> {
    sim.world()
        .get::<UiArranjo>(canvas)
        .map(|a| a.lacos.clone())
        .unwrap_or_default()
}

/// Republica o retrato com as relações gravadas (o que o passe faria no quadro seguinte) e redesenha.
fn quadro_seguinte(sim: &SimWorld, vec: &mut VecState, canvas: Entity) {
    vec.hud_visto[0].lacos = lacos(sim, canvas);
    desenha(vec);
    frame(&mut SimWorld::default(), vec);
}

#[test]
fn dragging_the_coin_onto_the_score_holds_it_beside_without_moving_the_drawing() {
    let (mut sim, mut vec, canvas) = mesa();
    puxa(&mut sim, &mut vec, [45.0, 5.0], [20.0, 5.0]);
    let l = lacos(&sim, canvas);
    assert_eq!(
        l[0],
        (
            Relacao::AoLado {
                a: MOEDA,
                b: PONT,
                lado: Lado::Direita,
                folga: 0.5
            },
            Forca::Requerida
        ),
        "a folga de agora, em unidades LOCAIS (1 de mundo / escala 2)"
    );
    assert_eq!(
        l[1],
        (Relacao::LarguraDoConteudo { a: PONT }, Forca::Media),
        "o texto ganha a largura dele"
    );
    assert_eq!(l.len(), 2);
    assert_eq!(
        vec.hud_tool.escolhida,
        Some(Par {
            canvas,
            a: MOEDA,
            b: PONT,
            eixo: Eixo::X
        })
    );
    // Juntar não mexe no desenho: o solver, sobre as caixas de agora, devolve-as.
    let v = &vec.hud_visto[0];
    let els: Vec<_> = v
        .filhos
        .iter()
        .map(|f| ph2d_hud_layout::Elemento {
            caixa: ph2d_hud_layout::Caixa {
                x: f.caixa.0[0] / 2.0,
                y: f.caixa.0[1] / 2.0,
                w: (f.caixa.1[0] - f.caixa.0[0]) / 2.0,
                h: (f.caixa.1[1] - f.caixa.0[1]) / 2.0,
            },
            conteudo: f.texto.then_some((f.caixa.1[0] - f.caixa.0[0]) / 2.0),
        })
        .collect();
    let rel: Vec<_> = l
        .iter()
        .filter_map(|(r, f)| Some((r.mapeia(|id| v.filhos.iter().position(|x| x.id == id))?, *f)))
        .collect();
    let tela = ph2d_hud_layout::Caixa {
        x: -500.0,
        y: -500.0,
        w: 1000.0,
        h: 1000.0,
    };
    let r = ph2d_hud_layout::resolve(tela, &els, &rel);
    assert_eq!(r, els.iter().map(|e| e.caixa).collect::<Vec<_>>());
}

#[test]
fn a_click_or_a_drop_on_nothing_writes_nothing() {
    let (mut sim, mut vec, canvas) = mesa();
    puxa(&mut sim, &mut vec, [45.0, 5.0], [45.5, 5.0]); // solto sobre si mesma
    puxa(&mut sim, &mut vec, [45.0, 5.0], [120.0, 90.0]); // solto no vazio
    puxa(&mut sim, &mut vec, [120.0, 90.0], [20.0, 5.0]); // começado no vazio
    assert!(lacos(&sim, canvas).is_empty());
    assert_eq!(vec.hud_tool.puxada, None);
}

#[test]
fn the_bar_turns_the_arrow_into_dont_overlap_centres_it_and_deletes_it() {
    let (mut sim, mut vec, canvas) = mesa();
    puxa(&mut sim, &mut vec, [45.0, 5.0], [20.0, 5.0]);
    quadro_seguinte(&sim, &mut vec, canvas);
    assert_eq!(
        state_hud::hud_vista().escolhida,
        Some((TipoSeta::AoLado, false))
    );
    assert!(
        state_hud::hud_vista().ancora.is_some(),
        "a ④ pousa no selo da seta"
    );
    state_hud::pede(PedidoHud::Tipo(TipoSeta::NaoInvade));
    frame(&mut sim, &mut vec);
    assert!(matches!(
        lacos(&sim, canvas)[1].0,
        Relacao::NaoInvade {
            a: MOEDA,
            b: PONT,
            lado: Lado::Esquerda,
            ..
        }
    ));
    state_hud::pede(PedidoHud::Centro);
    frame(&mut sim, &mut vec);
    assert!(lacos(&sim, canvas).iter().any(|(r, _)| *r
        == Relacao::Centrado {
            a: MOEDA,
            b: PONT,
            eixo: Eixo::Y
        }));
    quadro_seguinte(&sim, &mut vec, canvas);
    assert_eq!(
        state_hud::hud_vista().escolhida,
        Some((TipoSeta::NaoInvade, true))
    );
    state_hud::pede(PedidoHud::Centro);
    frame(&mut sim, &mut vec);
    assert!(
        !lacos(&sim, canvas)
            .iter()
            .any(|(r, _)| matches!(r, Relacao::Centrado { .. })),
        "o centro desliga"
    );
    state_hud::pede(PedidoHud::Apaga);
    frame(&mut sim, &mut vec);
    assert_eq!(
        lacos(&sim, canvas),
        vec![(Relacao::LarguraDoConteudo { a: PONT }, Forca::Media)]
    );
    assert_eq!(vec.hud_tool.escolhida, None);
}

#[test]
fn an_arrow_that_would_close_a_circle_is_refused_and_the_hint_says_so() {
    let (mut sim, mut vec, canvas) = mesa();
    // Gravado (por outro caminho): o minimapa ANTES da moeda e a moeda antes da pontuação.
    let gravado = vec![
        (
            Relacao::AoLado {
                a: MAPA,
                b: MOEDA,
                lado: Lado::Esquerda,
                folga: 0.0,
            },
            Forca::Requerida,
        ),
        (
            Relacao::AoLado {
                a: MOEDA,
                b: PONT,
                lado: Lado::Esquerda,
                folga: 0.0,
            },
            Forca::Requerida,
        ),
    ];
    sim.world_mut().entity_mut(canvas).insert(UiArranjo {
        lacos: gravado.clone(),
    });
    // Desenhado: o minimapa à direita da pontuação ⇒ «mapa depois da pontuação» fecha o círculo.
    puxa(&mut sim, &mut vec, [220.0, 0.0], [20.0, 5.0]);
    assert_eq!(lacos(&sim, canvas), gravado, "recusada: nada mudou");
    assert!(
        vec.hud_tool.recusada && state_hud::hud_vista().recusada,
        "a dica diz porquê"
    );
    press(&mut vec, [120.0, 90.0]);
    frame(&mut sim, &mut vec);
    assert!(
        !state_hud::hud_vista().recusada,
        "o próximo gesto apaga o aviso"
    );
}

#[test]
fn the_hit_rect_is_the_painted_rect() {
    let (mut sim, mut vec, canvas) = mesa();
    puxa(&mut sim, &mut vec, [45.0, 5.0], [20.0, 5.0]);
    quadro_seguinte(&sim, &mut vec, canvas);
    let pl = vec.hud_tool.plano.borrow().clone().unwrap();
    let s = &pl.setas[0];
    // A seta vai da borda esquerda da moeda à direita da pontuação, a meia altura da sobreposição.
    assert_eq!((s.de[0], s.para[0]), (41.0, 40.0));
    let c = [
        f64::from(s.selo.x + s.selo.w * 0.5),
        f64::from(s.selo.y + s.selo.h * 0.5),
    ];
    vec.hud_tool.escolhida = None;
    press(&mut vec, c);
    assert_eq!(
        vec.hud_tool.escolhida,
        Some(s.par),
        "clicar o selo pintado escolhe a seta"
    );
    assert_eq!(vec.hud_tool.puxada, None, "e não começa outra");
}

#[test]
fn without_the_tool_in_hand_nothing_is_published_or_written() {
    let (mut sim, mut vec, canvas) = mesa();
    press(&mut vec, [45.0, 5.0]);
    vec.draw_config.mode = DrawMode::Select;
    vec.shape_handles.release();
    frame(&mut sim, &mut vec);
    assert!(lacos(&sim, canvas).is_empty());
    assert_eq!(state_hud::hud_vista(), state_hud::HudVista::default());
    assert!(
        ph2d_panel_vector::bars::TOOLS
            .iter()
            .any(|t| t.0 == DrawMode::HudLayout && t.1 == ph2d_tool_vector::ids::VECTOR_MODE_HUD)
    );
}
