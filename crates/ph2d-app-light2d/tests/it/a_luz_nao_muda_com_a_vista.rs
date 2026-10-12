//! ⭐⭐⭐ **A luz num ponto do mundo não muda com a VISTA** (smoke do dono, 10/10: *«ao dar zoom e pan o
//! aspecto da iluminação muda»* — duas fotos da cena 4 com a mesma área à direita da parede pequena,
//! preta numa vista e cinza na outra). A grade de luz cobre a vista e uma margem; a lâmpada DIRETA
//! ilumina de qualquer distância, e o opaco FORA da grade no caminho até ela tem de a tapar (W5b, F4:
//! `ph2d_light2d_gpu::distantes`).
//!
//! A régua: pontos fixos do mundo, lidos pelo caminho do PRODUTO (UM `LuzDoQuadro` que a vista
//! reposiciona, como o editor no pan) ao longo de um pan contínuo em duas alturas, contra o gabarito
//! — na sombra, na penumbra e na luz; lâmpada de disco e de ponto; o opaco como polígono, como SPRITE
//! (a máscara do alfa, com uma fenda transparente) e como caminho VETORIAL. Cada caso confere que
//! mede o que diz: há vistas com o opaco e a lâmpada fora da grade, o quadro diz que testou objetos
//! distantes nelas, e cada controlo (sem o opaco; o sprite como o quad dele) muda o gabarito ali.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, Name, PresentWorld, SimRef, Transform, VecPathRef};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{BloqueiaLuz, CenaDeLuz, Luz, NaoRecebeLuz, Oclusor, TipoDeLuz};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer};
use ph2d_vec_scene::{VecScene, VecXforms, rectangle};

use crate::comum::{fluencia_lida, quad};

const FHD: (u32, u32) = (1920, 1080);

/// O teto: o pior erro de um ponto em qualquer vista, da LUZ QUE OS PONTOS RECEBERIAM sem o opaco (o
/// 1.º controlo) — a escala que o olho vê ali. A régua dos controlos (cada um muda > 10 × isto).
const TETO: f64 = 0.02;
/// ⭐ MEDIDO (W5b, RTX 5060 Ti, `smoke`, 1920×1080, `DEGRAU_PADRAO`) + 10 %: na cena 4 o pior é a
/// penumbra da lâmpada GRANDE com a parede fora da grade, `0,0032` (o ponto: `0,0004`; antes da cura o
/// do dono dava `0,176`); a fenda do sprite e do vetor, `0,0044` (o centro do feixe).
const TETO_CENA4: f64 = 0.0036;
const TETO_FENDA: f64 = 0.0049;

/// A instância que o extract da shell monta para `sprite` em `g`.
fn instancia(tex: u32, sprite: &Sprite, g: &GlobalTransform) -> RenderInstance {
    let [ax, ay, bx, by, tx, ty] = g.affine();
    let mut i = quad(tex, [tx, ty]);
    i.size = sprite.size;
    i.anchor = sprite.anchor;
    i.basis = [ax, ay, bx, by];
    i
}

/// Os sprites de `w` que bloqueiam, cada um com a textura `tex` (o caminho por texel do produto).
fn instancias(w: &mut World, tex: u32) -> PresentWorld {
    let mut present = PresentWorld::new();
    let mut q = w
        .query_filtered::<(Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<BloqueiaLuz>>(
        );
    for (e, s, g) in q.iter(w) {
        present.world_mut().spawn((instancia(tex, s, g), SimRef(e)));
    }
    present
}

/// Um caso: a cena, o que o quadro recebe, os pontos, as vistas e o opaco que sai da grade.
struct Caso<'a> {
    nome: &'a str,
    mundo: &'a World,
    /// Os sprites por texel (vazio = os quads como polígonos).
    present: PresentWorld,
    vetores: Option<(&'a VecScene, &'a VecXforms)>,
    /// Quantas MÁSCARAS (sprites por texel, caminhos vetoriais) o quadro tem de ter, no mínimo, nas
    /// vistas com o opaco fora da grade (0 = o opaco chega como arestas).
    mascaras: usize,
    /// O gabarito (o vetor e a fenda do sprite como os polígonos deles).
    gabarito: CenaDeLuz,
    /// Cenas de controlo: cada uma tem de mudar a resposta nos pontos. A 1.ª é a sem o opaco: a
    /// maior luz dela nos pontos é a escala do erro.
    controlos: Vec<(&'a str, CenaDeLuz)>,
    pontos: Vec<[f64; 2]>,
    vistas: Vec<([f32; 2], f32)>,
    /// A caixa (x0, y0, x1, y1, metros) do opaco e o centro da lâmpada que saem da grade.
    opaco: [f64; 4],
    /// O opaco fica INTEIRO na grade (só a lâmpada sai): a captura tem-no, e nada pode ser distante.
    opaco_dentro: bool,
    lampada: [f64; 2],
}

/// O pior erro do caso, do máximo.
fn medir(gpu: &ph2d_gpu::GpuContext, r: &SpriteRenderer, mut c: Caso<'_>) -> f64 {
    let fl = |cena: &CenaDeLuz, p: [f64; 2]| ph2d_light2d_ref::fluencia(cena, p)[1];
    let gab: Vec<f64> = c.pontos.iter().map(|&p| fl(&c.gabarito, p)).collect();
    let max = c
        .pontos
        .iter()
        .map(|&p| fl(&c.controlos[0].1, p))
        .fold(0.0, f64::max);
    eprintln!("{}: gabarito {gab:.4?} · escala {max:.4}", c.nome);
    for (nome, cena) in &c.controlos {
        let d = c
            .pontos
            .iter()
            .zip(&gab)
            .map(|(&p, g)| (fl(cena, p) - g).abs() / max)
            .fold(0.0, f64::max);
        assert!(
            d > 10.0 * TETO,
            "{}: o controlo «{nome}» não muda os pontos ({d:.4})",
            c.nome
        );
    }
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    let (mut pior, mut fora, mut onde) = (0.0_f64, 0, String::new());
    // Quantas vezes cada ponto foi medido numa vista com o opaco e a lâmpada fora da grade.
    let mut medidos = vec![0; c.pontos.len()];
    for &(centro, altura) in &c.vistas {
        let afim =
            Camera2d::new(centro, altura).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        assert!(l.preparar(c.mundo, afim, FHD));
        if let Some((doc, xf)) = c.vetores {
            l.desenhar_vetores(gpu, doc, xf);
        }
        l.gravar(gpu, &mut c.present, r);
        let g = l.grade().expect("grade");
        let (x0, y0) = (g.origem_m[0], g.origem_m[1]);
        let (x1, y1) = (
            x0 + f64::from(g.largura) * g.texel_m,
            y0 + f64::from(g.altura) * g.texel_m,
        );
        let dentro = |p: [f64; 2]| (x0..=x1).contains(&p[0]) && (y0..=y1).contains(&p[1]);
        let o = c.opaco;
        let opaco_na_grade = dentro([o[0], o[1]]) && dentro([o[2], o[3]]);
        let fora_da_grade = opaco_na_grade == c.opaco_dentro && !dentro(c.lampada);
        if fora_da_grade {
            fora += 1;
            let rel = l.relatorio().expect("relatório");
            if c.opaco_dentro {
                assert_eq!(
                    rel.distantes, 0,
                    "{}: vista {centro:?} × {altura}: o opaco inteiro na grade foi tomado por distante",
                    c.nome
                );
            } else {
                assert!(
                    rel.mascaras >= c.mascaras,
                    "{}: vista {centro:?} × {altura}: o opaco não chegou como máscara ({rel:?})",
                    c.nome
                );
                assert!(
                    rel.distantes > 0,
                    "{}: vista {centro:?} × {altura}: nenhum objeto distante",
                    c.nome
                );
            }
        }
        let f = fluencia_lida(gpu, l.solver().expect("solver"));
        // ⚠️ Só a JANELA se calcula (o resto da grade guarda o valor velho): mede-se o que se vê.
        let meia = [altura * FHD.0 as f32 / FHD.1 as f32 / 2.0, altura / 2.0];
        let na_janela =
            |p: [f64; 2]| (0..2).all(|e| (p[e] - f64::from(centro[e])).abs() <= f64::from(meia[e]));
        for (k, (&p, q)) in c.pontos.iter().zip(&gab).enumerate() {
            if !na_janela(p) {
                continue;
            }
            medidos[k] += usize::from(fora_da_grade);
            let v = f.amostra(p);
            let e = (v - q).abs() / max;
            if e > pior {
                pior = e;
                onde = format!(
                    "vista {centro:?} × {altura} m, ponto {k} {p:?}: placa {v:.4} × gabarito {q:.4}"
                );
            }
        }
    }
    eprintln!(
        "{}: {} vistas, {fora} com o opaco e a lâmpada fora da grade · medidos {medidos:?} · pior {pior:.4} do máximo ({onde})",
        c.nome,
        c.vistas.len()
    );
    assert!(
        fora > 0,
        "{}: nenhuma vista tirou o opaco e a lâmpada da grade",
        c.nome
    );
    assert!(
        medidos.iter().all(|&n| n > 0),
        "{}: pontos nunca medidos na janela com o opaco fora da grade: {medidos:?}",
        c.nome
    );
    pior
}

/// O pan contínuo do centro de `x0` a `x1` (a `y`), em cada altura.
fn pan(x0: f32, x1: f32, y: f32, alturas: &[f32]) -> Vec<([f32; 2], f32)> {
    let n = 16;
    alturas
        .iter()
        .flat_map(|&a| (0..=n).map(move |i| ([x0 + (x1 - x0) * i as f32 / n as f32, y], a)))
        .collect()
}

fn tira(w: &mut World, nome: &str) {
    let e = w
        .query::<(Entity, &Name)>()
        .iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .expect("existe");
    w.despawn(e);
}

/// A cena 4 sem a ambiente (só as lâmpadas contam); `ponto` = as lâmpadas sem tamanho; sem `fora`.
fn cena4(ponto: bool, fora: Option<&str>) -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 4));
    tira(&mut w, "Luz ambiente");
    if let Some(n) = fora {
        tira(&mut w, n);
    }
    if ponto {
        for mut l in w.query::<&mut Luz>().iter_mut(&mut w) {
            l.tamanho_m = 0.0;
        }
    }
    w
}

/// Os dois casos da cena 4 (o do dono e a sala da grande vista de perto), com os sprites como
/// polígonos (`por_texel = false`) ou pelo alfa da textura.
fn casos_da_cena4(gpu: &ph2d_gpu::GpuContext, por_texel: bool) -> Vec<f64> {
    let mut r = crate::comum::renderer(gpu);
    let tex = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let mut piores = Vec::new();
    for ponto in [false, true] {
        let tipo = if ponto { "ponto" } else { "disco" };
        let sufixo = if por_texel {
            "sprites por texel"
        } else {
            "quads"
        };
        // 1. O do dono: na sala da pequena, à direita da parede pequena; só a grande, do outro lado
        //    do MURO, poderia acender os pontos.
        let mut w = cena4(ponto, None);
        let present = if por_texel {
            instancias(&mut w, tex)
        } else {
            PresentWorld::new()
        };
        let nome = format!("cena 4 · o muro · {tipo} · {sufixo}");
        piores.push(medir(
            gpu,
            &r,
            Caso {
                nome: &nome,
                mundo: &w,
                present,
                vetores: None,
                mascaras: usize::from(por_texel),
                gabarito: cena_de_luz(&w),
                controlos: vec![("sem o muro", cena_de_luz(&cena4(ponto, Some("Muro"))))],
                pontos: vec![[-2.8, -1.0], [-2.5, -1.8], [-2.2, -2.3], [-1.2, -2.0]],
                vistas: pan(-3.4, -1.8, -2.0, &[1.5, 2.5]),
                opaco: [-0.1, -4.25, 0.1, 4.25],
                opaco_dentro: false,
                lampada: [1.1, 1.5],
            },
        ));
        // 2. A sala da grande vista de perto: os pontos na penumbra (e, com o ponto, na sombra e na
        //    luz) da PAREDE da grande, que sai da grade com a lâmpada.
        let mut w = cena4(ponto, None);
        let present = if por_texel {
            instancias(&mut w, tex)
        } else {
            PresentWorld::new()
        };
        let nome = format!("cena 4 · a parede da grande · {tipo} · {sufixo}");
        piores.push(medir(
            gpu,
            &r,
            Caso {
                nome: &nome,
                mundo: &w,
                present,
                vetores: None,
                mascaras: usize::from(por_texel),
                gabarito: cena_de_luz(&w),
                controlos: vec![(
                    "sem a parede",
                    cena_de_luz(&cena4(ponto, Some("Parede da grande"))),
                )],
                pontos: vec![
                    [4.8, -1.6],
                    [5.1, -2.2],
                    [5.4, -1.0],
                    [5.4, -2.2],
                    [5.7, -1.0],
                ],
                vistas: pan(4.7, 5.9, -1.6, &[1.5, 2.0]),
                opaco: [2.4, -2.5, 2.6, 0.5],
                opaco_dentro: false,
                lampada: [1.1, 1.5],
            },
        ));
    }
    piores
}

#[test]
fn a_luz_num_ponto_nao_muda_com_a_vista() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let piores = casos_da_cena4(&gpu, false);
    let pior = piores.iter().copied().fold(0.0, f64::max);
    assert!(
        pior <= TETO_CENA4,
        "a luz muda com a vista (quads): {piores:.4?} (teto {TETO_CENA4})"
    );
}

#[test]
fn a_luz_num_ponto_nao_muda_com_a_vista_com_os_sprites_por_texel() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let piores = casos_da_cena4(&gpu, true);
    let pior = piores.iter().copied().fold(0.0, f64::max);
    assert!(
        pior <= TETO_CENA4,
        "a luz muda com a vista (sprites por texel): {piores:.4?} (teto {TETO_CENA4})"
    );
}

/// A cena da FENDA: uma lâmpada de disco à esquerda, fora da tela; um opaco de 0,4 × 3 m em x = −5
/// com uma fenda de 0,2 m no meio; os pontos à direita, no feixe e na penumbra dele.
const FENDA_Y: f64 = 0.1;
const OPACO: [f64; 4] = [-5.2, -1.5, -4.8, 1.5];
const LAMPADA_DA_FENDA: [f64; 2] = [-8.0, 0.0];

fn lampada_da_fenda(w: &mut World) {
    let mut t = Transform::IDENTITY;
    t.translation.x = LAMPADA_DA_FENDA[0] as f32;
    w.spawn((
        Luz {
            cor: [1.0, 0.95, 0.85],
            intensidade: 8.0,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        GlobalTransform::from_transform(t),
    ));
}

/// O centro do outro sprite da cena da fenda (0,3 × 0,2 m): dentro do caminho até a lâmpada, abaixo
/// dos raios dos pontos.
const OUTRO: [f32; 2] = [-6.5, -0.5];

/// O gabarito da fenda: os dois retângulos opacos (e o outro sprite, se a cena o tem).
fn gabarito_da_fenda(w: &World) -> CenaDeLuz {
    let mut c = cena_de_luz(w);
    let [x0, y0, x1, y1] = OPACO.map(|v| v as f32);
    let f = FENDA_Y as f32;
    let ret = |x0: f32, a: f32, x1: f32, b: f32| Oclusor {
        vertices: vec![[x0, a], [x1, a], [x1, b], [x0, b]],
        opacidade: 1.0,
        albedo: [0.0; 3],
    };
    let outro = c.oclusores.len() > 1;
    c.oclusores = vec![ret(x0, y0, x1, -f), ret(x0, f, x1, y1)];
    if outro {
        let [ox, oy] = OUTRO;
        c.oclusores
            .push(ret(ox - 0.15, oy - 0.1, ox + 0.15, oy + 0.1));
    }
    c.sprites.clear();
    c.vetores.clear();
    c
}

fn so_a_lampada() -> CenaDeLuz {
    let mut w = World::new();
    lampada_da_fenda(&mut w);
    cena_de_luz(&w)
}

const PONTOS_DA_FENDA: [[f64; 2]; 6] = [
    [1.2, -0.6],
    [1.2, -0.3],
    [1.5, 0.0],
    [1.5, 0.2],
    [1.8, 0.4],
    [1.8, 0.7],
];

#[test]
fn um_sprite_com_fenda_fora_da_tela_tapa_pelo_alfa() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut r = crate::comum::renderer(&gpu);
    let mut w = World::new();
    lampada_da_fenda(&mut w);
    let mut present = PresentWorld::new();
    // ⭐ Outro sprite, outra textura, no caminho mas fora dos raios dos pontos — ele pega a 1.ª janela
    // da atlas e a fenda a 2.ª (a janela de cada máscara conta).
    let mut t = Transform::IDENTITY;
    t.translation.x = OUTRO[0];
    t.translation.y = OUTRO[1];
    let g = GlobalTransform::from_transform(t);
    let outro = Sprite::atlas(0, [0.3, 0.2], [1.0; 4]);
    let e = w.spawn((outro, BloqueiaLuz { opacidade: 1.0 }, t, g)).id();
    let tex = r
        .acquire_individual(1, 1, &[200, 0, 0, 255])
        .expect("textura");
    present
        .world_mut()
        .spawn((instancia(tex, &outro, &g), SimRef(e)));
    let mut t = Transform::IDENTITY;
    t.translation.x = -5.0;
    let g = GlobalTransform::from_transform(t);
    let sprite = Sprite::atlas(0, [0.4, 3.0], [1.0; 4]);
    let e = w.spawn((sprite, BloqueiaLuz { opacidade: 1.0 }, t, g)).id();
    // 1×15: a linha 7 transparente — a fenda y ∈ [−0,1, 0,1].
    let rgba: Vec<u8> = (0..15)
        .flat_map(|j| [90, 90, 90, if j == 7 { 0 } else { 255 }])
        .collect();
    let tex = r.acquire_individual(1, 15, &rgba).expect("textura");
    present
        .world_mut()
        .spawn((instancia(tex, &sprite, &g), SimRef(e)));
    let quad = cena_de_luz(&w);
    let pior = medir(
        &gpu,
        &r,
        Caso {
            nome: "sprite com fenda",
            mundo: &w,
            present,
            vetores: None,
            mascaras: 2,
            gabarito: gabarito_da_fenda(&w),
            controlos: vec![("sem os sprites", so_a_lampada()), ("o quad inteiro", quad)],
            pontos: PONTOS_DA_FENDA.to_vec(),
            vistas: pan(0.9, 2.1, 0.0, &[1.5, 2.0]),
            opaco: OPACO,
            opaco_dentro: false,
            lampada: LAMPADA_DA_FENDA,
        },
    );
    assert!(
        pior <= TETO_FENDA,
        "o sprite com fenda fora da tela: {pior:.4} (teto {TETO_FENDA})"
    );
}

#[test]
fn um_caminho_vetorial_fora_da_tela_tapa() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let r = crate::comum::renderer(&gpu);
    let mut w = World::new();
    lampada_da_fenda(&mut w);
    let mut doc = VecScene::new();
    let [x0, y0, x1, y1] = OPACO;
    for (a, b) in [(y0, -FENDA_Y), (FENDA_Y, y1)] {
        let id = doc.push_path(rectangle([x0, a], [x1, b]));
        w.spawn((VecPathRef(id), BloqueiaLuz { opacidade: 1.0 }));
    }
    let xforms = VecXforms::new();
    let pior = medir(
        &gpu,
        &r,
        Caso {
            nome: "caminho vetorial",
            mundo: &w,
            present: PresentWorld::new(),
            vetores: Some((&doc, &xforms)),
            mascaras: 2,
            gabarito: gabarito_da_fenda(&w),
            controlos: vec![("sem o caminho", so_a_lampada())],
            pontos: PONTOS_DA_FENDA.to_vec(),
            vistas: pan(0.9, 2.1, 0.0, &[1.5, 2.0]),
            opaco: OPACO,
            opaco_dentro: false,
            lampada: LAMPADA_DA_FENDA,
        },
    );
    assert!(
        pior <= TETO_FENDA,
        "o caminho vetorial fora da tela: {pior:.4} (teto {TETO_FENDA})"
    );
}

/// ⭐ Um opaco TRANSLÚCIDO fora da tela deixa passar a sua parte: o muro da cena 4 com `BloqueiaLuz`
/// ½ — cada ENTRADA multiplica por `1 − opacidade` (a lei do gabarito), pelos quads (as arestas) e
/// pelos sprites por texel (a máscara).
#[test]
fn um_opaco_translucido_fora_da_tela_deixa_passar_a_sua_parte() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut r = crate::comum::renderer(&gpu);
    let tex = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let translucido = || {
        let mut w = cena4(false, None);
        let muro = w
            .query::<(Entity, &Name)>()
            .iter(&w)
            .find(|(_, n)| n.0 == "Muro")
            .map(|(e, _)| e)
            .expect("o muro");
        w.entity_mut(muro).insert(BloqueiaLuz { opacidade: 0.5 });
        w
    };
    let mut piores = Vec::new();
    for por_texel in [false, true] {
        let mut w = translucido();
        let present = if por_texel {
            instancias(&mut w, tex)
        } else {
            PresentWorld::new()
        };
        let nome = format!(
            "o muro a ½ · {}",
            if por_texel {
                "sprites por texel"
            } else {
                "quads"
            }
        );
        piores.push(medir(
            &gpu,
            &r,
            Caso {
                nome: &nome,
                mundo: &w,
                present,
                vetores: None,
                mascaras: usize::from(por_texel),
                gabarito: cena_de_luz(&w),
                controlos: vec![
                    ("sem o muro", cena_de_luz(&cena4(false, Some("Muro")))),
                    ("o muro opaco", cena_de_luz(&cena4(false, None))),
                ],
                pontos: vec![[-2.8, -1.0], [-2.5, -1.8], [-2.2, -2.3], [-1.2, -2.0]],
                vistas: pan(-3.4, -1.8, -2.0, &[1.5, 2.5]),
                opaco: [-0.1, -4.25, 0.1, 4.25],
                opaco_dentro: false,
                lampada: [1.1, 1.5],
            },
        ));
    }
    let pior = piores.iter().copied().fold(0.0, f64::max);
    assert!(
        pior <= TETO_CENA4,
        "o muro translúcido fora da tela: {piores:.4?} (teto {TETO_CENA4})"
    );
}

/// ⭐ Um opaco translúcido INTEIRO na grade, com a lâmpada fora dela: a captura tem-no, e ele não
/// pode ser contado outra vez como distante (cada entrada × `1 − opacidade` UMA vez).
#[test]
fn um_opaco_inteiro_na_grade_nao_conta_duas_vezes() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut r = crate::comum::renderer(&gpu);
    let tex = r.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let cena = |opacidade: Option<f32>| {
        let mut w = World::new();
        lampada_da_fenda(&mut w);
        if let Some(o) = opacidade {
            let mut t = Transform::IDENTITY;
            t.translation.x = 0.5;
            let g = GlobalTransform::from_transform(t);
            // ⭐ W5k: o translúcido reflete `α·a` e a régua (`ref::fluencia`) não tem o rebote — a lei
            // deste portão é a transmissão contada UMA vez, não o rebote (esse tem os seus, g9).
            w.spawn((
                Sprite::atlas(0, [0.2, 3.0], [1.0; 4]),
                BloqueiaLuz { opacidade: o },
                NaoRecebeLuz,
                t,
                g,
            ));
        }
        w
    };
    let mut piores = Vec::new();
    for por_texel in [false, true] {
        let mut w = cena(Some(0.5));
        let present = if por_texel {
            instancias(&mut w, tex)
        } else {
            PresentWorld::new()
        };
        let nome = format!(
            "translúcido inteiro na grade · {}",
            if por_texel {
                "sprites por texel"
            } else {
                "quads"
            }
        );
        piores.push(medir(
            &gpu,
            &r,
            Caso {
                nome: &nome,
                mundo: &w,
                present,
                vetores: None,
                mascaras: 0,
                gabarito: cena_de_luz(&w),
                controlos: vec![
                    ("sem ele", cena_de_luz(&cena(None))),
                    ("opaco", cena_de_luz(&cena(Some(1.0)))),
                ],
                pontos: vec![[1.2, -0.5], [1.5, 0.0], [1.8, 0.5]],
                vistas: pan(0.9, 2.1, 0.0, &[1.5, 2.0]),
                opaco: [0.4, -1.5, 0.6, 1.5],
                opaco_dentro: true,
                lampada: LAMPADA_DA_FENDA,
            },
        ));
    }
    let pior = piores.iter().copied().fold(0.0, f64::max);
    assert!(
        pior <= TETO_CENA4,
        "o translúcido inteiro na grade: {piores:.4?} (teto {TETO_CENA4})"
    );
}
