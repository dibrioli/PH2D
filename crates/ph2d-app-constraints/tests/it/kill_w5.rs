//! **Kill-criterion do resto da W5** (`docs/BlenderConstraints/06_W5_resto.md` §5): 0,5 ms por quadro
//! para 1 000 de cada peça (Valor, *behaviours*, HUD), na MESMA rodada — variantes no mesmo processo, intercaladas em blocos,
//! resumo = o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).
//!
//! `#[ignore]`: o relógio só vale sozinho e no perfil `smoke` —
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-constraints --test it kill_w5 -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::valor::valores;
use ph2d_constraints::comportamento::Perfil;
use ph2d_constraints::{Kind, Valor};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_hud_layout::{Arranjo, Caixa, Elemento, Forca, Lado, Relacao};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::TimelineDoc;

const N: u64 = 1_000;
const TECTO: Duration = Duration::from_micros(500);
const BLOCOS: usize = 7;
const QUADROS: u32 = 40;

/// Um alvo que sobe e desce e 1 000 donos, cada um com um Valor de mola sobre a altura dele.
fn cena_valor(sim: &mut SimWorld) -> Entity {
    let w = sim.world_mut();
    let alvo = w.spawn((StableId(1), Transform::IDENTITY)).id();
    for i in 0..N {
        let mut c = Constraints::default();
        c.push(Kind::Valor(Valor::default()), Some(StableId(1)));
        w.spawn((StableId(10 + i), Transform::IDENTITY, c));
    }
    alvo
}

/// Um alvo que sobe e desce e 1 000 donos, cada um com um *behaviour* — a mistura dos quatro (o
/// Atrair e o Orbitar puxados pelo alvo).
/// Com `base`, o mesmo mundo com um Limit (a lei mais barata) em cada dono: o quadro da pilha sem
/// *behaviour*, que se desconta — a peça custa a diferença.
fn cena_comportamento(sim: &mut SimWorld, base: bool) -> Entity {
    let w = sim.world_mut();
    let alvo = w.spawn((StableId(1), Transform::IDENTITY)).id();
    for i in 0..N {
        let perfil = Perfil::ALL[(i % 4) as usize];
        let mut c = Constraints::default();
        let k = if base {
            Kind::Limit(ph2d_constraints::Limit::default())
        } else {
            Kind::Comportamento(perfil.numeros())
        };
        c.push(k, Some(StableId(1)));
        w.spawn((StableId(10 + i), Transform::IDENTITY, c));
    }
    alvo
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
fn sobe(sim: &mut SimWorld, alvo: Entity, f: u32) {
    sim.world_mut()
        .get_mut::<Transform>(alvo)
        .unwrap()
        .translation
        .y = (f64::from(f % 48) / 47.0) as f32;
}

/// 1 000 elementos de HUD em 500 pares (a moeda AO LADO da pontuação, que tem a largura do texto) — o pior caso do
/// arranjo vivo: TODAS as caixas e TODOS os conteúdos mudam a cada quadro.
fn hud(n: usize) -> (Vec<Elemento>, Vec<(Relacao, Forca)>) {
    let mut els = Vec::new();
    let mut rel = Vec::new();
    for i in 0..n / 2 {
        #[expect(clippy::cast_precision_loss, reason = "índice pequeno")]
        let y = (i % 40) as f64 * 9.0;
        els.push(Elemento {
            caixa: Caixa {
                x: 10.0,
                y,
                w: 40.0,
                h: 8.0,
            },
            conteudo: Some(40.0),
        });
        els.push(Elemento {
            caixa: Caixa {
                x: 52.0,
                y,
                w: 8.0,
                h: 8.0,
            },
            conteudo: None,
        });
        rel.push((Relacao::LarguraDoConteudo { a: 2 * i }, Forca::Media));
        rel.push((
            Relacao::AoLado {
                a: 2 * i + 1,
                b: 2 * i,
                lado: Lado::Direita,
                folga: 2.0,
            },
            Forca::Requerida,
        ));
    }
    (els, rel)
}

/// O quadro `f` do HUD: o texto de cada pontuação cresce e encolhe, a moeda vem do sítio antigo.
fn hud_quadro(els: &mut [Elemento], f: u32) {
    let w = 40.0 + f64::from(f % 16) * 1.5;
    for (i, e) in els.iter_mut().enumerate() {
        if i % 2 == 0 {
            e.caixa.w = w;
            e.conteudo = Some(w);
        } else {
            e.caixa.x = 52.0 + f64::from(f % 7);
        }
    }
}

const TELA: Caixa = Caixa {
    x: 0.0,
    y: 0.0,
    w: 640.0,
    h: 400.0,
};

#[test]
#[ignore = "relógio: corre sozinho, --profile smoke"]
fn w5_pieces_under_half_a_millisecond_per_thousand() {
    let doc = TimelineDoc::new();
    let fps = doc.fps_display;
    let mut sv = SimWorld::new();
    let alvo = cena_valor(&mut sv);
    let mut sc = SimWorld::new();
    let alvo_c = cena_comportamento(&mut sc, false);
    let mut sb = SimWorld::new();
    let alvo_b = cena_comportamento(&mut sb, true);
    let (mut drive, mut drive_b) = (PreviewDrive::default(), PreviewDrive::default());
    let cena = ph2d_vec_scene::VecScene::new();
    let (mut valor, mut comportamento, mut base) = (Duration::MAX, Duration::MAX, Duration::MAX);
    let (mut els, rel) = hud(N as usize);
    let mut arranjo = Arranjo::new(TELA, &els, &rel);
    let mut hud_t = Duration::MAX;
    let mut f = 0;
    for _bloco in 0..BLOCOS {
        for _ in 0..QUADROS {
            f += 1;
            let r = Relogio {
                doc: &doc,
                t: f64::from(f) / fps,
                laco: None,
            };
            sobe(&mut sv, alvo, f);
            let t0 = Instant::now();
            valores(&mut sv, Some(r));
            valor = valor.min(t0.elapsed());
            // O quadro INTEIRO da família (preparar + a pilha), não só a lei.
            sobe(&mut sc, alvo_c, f);
            let mut cena = cena.clone();
            let t0 = Instant::now();
            ph2d_app_constraints::quadro(&mut sc, &mut cena, &mut drive, Some(r));
            comportamento = comportamento.min(t0.elapsed());
            sobe(&mut sb, alvo_b, f);
            let mut cena = cena.clone();
            let t0 = Instant::now();
            ph2d_app_constraints::quadro(&mut sb, &mut cena, &mut drive_b, Some(r));
            base = base.min(t0.elapsed());
            hud_quadro(&mut els, f);
            let t0 = Instant::now();
            std::hint::black_box(arranjo.atualiza_elementos(TELA, &els));
            hud_t = hud_t.min(t0.elapsed());
        }
    }
    let peca = comportamento.saturating_sub(base);
    println!(
        "kill: valor {valor:?} · comportamento {peca:?} (quadro {comportamento:?} − pilha {base:?}) · HUD {hud_t:?} / 1 000 itens (mínimo de {BLOCOS}×{QUADROS})"
    );
    assert!(valor <= TECTO, "valor {valor:?} > {TECTO:?}");
    assert!(peca <= TECTO, "comportamento {peca:?} > {TECTO:?}");
    assert!(hud_t <= TECTO, "HUD {hud_t:?} > {TECTO:?}");
}

/// Instrumento (não portão): o custo de UM grupo em cadeia (cada elemento ao lado do anterior) por tamanho — a tabela
/// do tecto por grupo (ADR-0194 §custo). `new` uma vez, `atualiza` com tudo a mudar, o mínimo de 7×10.
#[test]
#[ignore = "relógio: corre sozinho, --profile smoke"]
fn w5_hud_group_cost_by_size() {
    for n in [8_usize, 16, 32, 64, 128] {
        let mut els: Vec<Elemento> = (0..n)
            .map(|i| {
                #[expect(clippy::cast_precision_loss, reason = "índice pequeno")]
                let x = i as f64 * 5.0;
                Elemento {
                    caixa: Caixa {
                        x,
                        y: 10.0,
                        w: 4.0,
                        h: 4.0,
                    },
                    conteudo: Some(4.0),
                }
            })
            .collect();
        let mut rel: Vec<_> = (1..n)
            .map(|i| {
                (
                    Relacao::AoLado {
                        a: i,
                        b: i - 1,
                        lado: Lado::Direita,
                        folga: 1.0,
                    },
                    Forca::Requerida,
                )
            })
            .collect();
        rel.extend((0..n).map(|i| (Relacao::LarguraDoConteudo { a: i }, Forca::Media)));
        let t0 = Instant::now();
        let mut arr = Arranjo::new(TELA, &els, &rel);
        let novo = t0.elapsed();
        let mut melhor = Duration::MAX;
        for f in 0..70_u32 {
            let w = 4.0 + f64::from(f % 5) * 0.5;
            for e in &mut els {
                e.caixa.w = w;
                e.conteudo = Some(w);
            }
            let t0 = Instant::now();
            std::hint::black_box(arr.atualiza_elementos(TELA, &els));
            melhor = melhor.min(t0.elapsed());
        }
        println!("grupo {n:>4}: new {novo:?} · atualiza {melhor:?}");
    }
}
