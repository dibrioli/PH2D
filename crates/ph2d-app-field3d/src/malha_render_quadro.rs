//! ⭐⭐ **O QUADRO DO RENDER POR MALHA** — do estado do modelador à `Cena` do desenhista de jogo
//! ([`ph2d_mesh_forward`]), desenhada NA HORA e em resolução CHEIA: nada de fila, nada de resolução
//! que cai a mexer, nada a assentar.
//!
//! ⚠️ **A câmara é a MESMA conta de pixel do traçado** ([`ph2d_field_render::Orbit::project`] e o
//! `Screen::pixel_of`): a malha cai onde o gizmo e o contorno a desenham. Há gate
//! (`a_projecao_e_a_do_orbit`).

use std::cell::RefCell;
use std::sync::{Arc, Mutex, OnceLock};

use ph2d_field::FieldDoc;
use ph2d_field_render::{Lens, Orbit};
use ph2d_mesh_forward::{Ambiente, Camera, Cena, Forward, Instancia, Luz, Malha};

/// O que o Render por malha fez neste quadro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feito {
    /// Um quadro NOVO, RGBA premultiplicado no tamanho pedido.
    Novo(Vec<u8>),
    /// Nada mudou: o quadro que o viewport tem continua certo.
    Mesmo,
    /// A primeira extração ainda não chegou: o viewport mostra o que tinha.
    Espera,
    /// Não há aparelho para o desenhista — quem chama volta ao traçado.
    SemAparelho,
}

#[derive(Clone, PartialEq)]
struct Assinatura {
    geracao: u64,
    modelos: Vec<[[f32; 4]; 4]>,
    materiais: Vec<[f32; ph2d_material::wgsl::PACKED]>,
    camera: Camera,
    luzes: Vec<Luz>,
    chao: Option<f32>,
    look: ph2d_view_transform::Look,
    brilho: ph2d_bloom::Bloom,
    /// O céu escolhido (saneado) — o atlas dele já está montado quando a assinatura nasce.
    ceu: crate::ceu_foto::Ceu,
    estilo: ph2d_style::Style,
    /// A textura de cada material (`crate::texturas`).
    texturas: Vec<Option<ph2d_mesh_forward::TexturaMaterial>>,
    raio: f32,
    /// A chave das curvaturas com que este quadro foi desenhado.
    curv: Option<crate::malha_render_estado::ChaveCurv>,
    tamanho: (u32, u32),
}

/// O desenhista e o que já subiu para ele (a geração, e quantas malhas ela tinha).
struct Desenhista {
    fw: Forward,
    subida: (u64, usize),
    /// A chave das curvaturas subidas (`None` = as malhas acabaram de subir, com zeros).
    curv_subida: Option<crate::malha_render_estado::ChaveCurv>,
    /// O céu fotográfico cujo atlas está subido.
    ceu_subido: Option<ph2d_sky::Embarcado>,
    /// As camadas de textura já subidas.
    texturas_subidas: std::collections::BTreeSet<u32>,
}

/// ⛔ **GLOBAL, e não `thread_local`** — medido (02/10): um `Forward` numa `thread_local` é largado
/// na DESTRUIÇÃO da thread, e o `Drop` do wgpu toca noutra `thread_local` já destruída ⇒ pânico
/// dentro de um destrutor ⇒ o processo ABORTA ao fechar. É a mesma lei do `gpu_frame` (o traçador
/// também é global). `None` dentro = não há aparelho (a recusa fica guardada).
static DESENHISTA: OnceLock<Option<Mutex<Desenhista>>> = OnceLock::new();

thread_local! {
    static ULTIMA: RefCell<Vec<Option<Assinatura>>> = const { RefCell::new(Vec::new()) };
}

/// ⭐ **Esta placa desenha o brilho no Render por malha?** — quem pinta o painel pergunta aqui
/// antes de oferecer as fileiras dele (um botão que não muda nada é pior do que nenhum). Fica de
/// fora da trava do desenhista: a resposta nasce com ele e nunca muda.
#[must_use]
pub(crate) fn tem_brilho() -> bool {
    desenhista().is_some() && TEM_BRILHO.get().copied().unwrap_or(false)
}

/// A resposta do [`tem_brilho`], escrita UMA vez quando o desenhista nasce.
static TEM_BRILHO: OnceLock<bool> = OnceLock::new();

fn desenhista() -> Option<&'static Mutex<Desenhista>> {
    DESENHISTA
        .get_or_init(|| {
            let constantes = crate::studio_wgsl::constants();
            let tabela = crate::studio_wgsl::tables();
            Forward::no_aparelho(&Ambiente {
                wgsl: crate::studio_wgsl::SOURCE,
                constantes: &constantes,
                tabela: &tabela,
                piso_luz: ph2d_field_render::POINT_LAMP_MIN_DISTANCE,
            })
            .map(|fw| {
                let _ = TEM_BRILHO.set(fw.tem_brilho());
                Mutex::new(Desenhista {
                    fw,
                    subida: (0, 0),
                    curv_subida: None,
                    ceu_subido: None,
                    texturas_subidas: std::collections::BTreeSet::new(),
                })
            })
        })
        .as_ref()
}

/// O ângulo da caixa de luz do estúdio — a penumbra da sombra.
fn caixa_tan() -> f32 {
    crate::studio::SOFTBOX_RADIUS_DEG.to_radians().tan()
}

/// ⭐ **A câmara do traçado como matriz** — mundo → recorte, coluna a coluna.
///
/// O traçado leva o ponto `p` ao pixel por `u = (p−alvo)·direita`, `v = (p−alvo)·cima`, a
/// convergência `k = d / (d − (p−alvo)·frente)` na perspectiva, e `Screen::pixel_of`. Em recorte:
/// `x = sx·d·u`, `y = sy·d·v`, `w = d − (p−alvo)·frente`, com `sx = 2·meio/(he·W)`.
#[must_use]
pub fn camera(cam: &Orbit, (w, h): (u32, u32)) -> Camera {
    let (right, up, fwd) = cam.basis();
    let t = cam.target;
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let meio = (w.min(h) as f32) * 0.5;
    let he = cam.half_extent.max(1.0e-6);
    let (sx, sy) = (2.0 * meio / (he * w as f32), 2.0 * meio / (he * h as f32));
    // Linhas da matriz (recorte = L · [p, 1]); depois transpostas para colunas.
    let (lx, ly, lz, lw): ([f32; 4], [f32; 4], [f32; 4], [f32; 4]) =
        match (cam.lens, cam.eye_distance()) {
            (Lens::Perspective { .. }, Some(d)) => {
                let lw = [-fwd[0], -fwd[1], -fwd[2], d + dot(t, fwd)];
                let (n, f) = (d * 1.0e-3, d + 200.0 * he);
                let a = f / (f - n);
                (
                    [
                        sx * d * right[0],
                        sx * d * right[1],
                        sx * d * right[2],
                        -sx * d * dot(t, right),
                    ],
                    [
                        sy * d * up[0],
                        sy * d * up[1],
                        sy * d * up[2],
                        -sy * d * dot(t, up),
                    ],
                    [a * lw[0], a * lw[1], a * lw[2], a * lw[3] - a * n],
                    lw,
                )
            }
            _ => {
                // Ortográfica: o raio parte de `alvo + frente·ORTHO_START`; a profundidade é a distância
                // a esse começo, sobre um alcance largo.
                let start = ph2d_field_render::ORTHO_START;
                let f = 2.0 * start + 200.0 * he;
                (
                    [
                        sx * right[0],
                        sx * right[1],
                        sx * right[2],
                        -sx * dot(t, right),
                    ],
                    [sy * up[0], sy * up[1], sy * up[2], -sy * dot(t, up)],
                    [
                        -fwd[0] / f,
                        -fwd[1] / f,
                        -fwd[2] / f,
                        (start + dot(t, fwd)) / f,
                    ],
                    [0.0, 0.0, 0.0, 1.0],
                )
            }
        };
    let col = |k: usize| [lx[k], ly[k], lz[k], lw[k]];
    Camera {
        view_proj: [col(0), col(1), col(2), col(3)],
        olho: cam.eye().unwrap_or(t),
        perspectiva: cam.eye().is_some(),
        dir_vista: [-fwd[0], -fwd[1], -fwd[2]],
    }
}

fn materiais(smoke: &crate::smoke::Smoke) -> Vec<[f32; ph2d_material::wgsl::PACKED]> {
    let pack = |s: &ph2d_material::Surface| {
        ph2d_material::wgsl::pack(s, ph2d_material::wgsl::EnvLobe::of(s))
    };
    match &smoke.materials {
        Some(t) if !t.surfaces.is_empty() => t.surfaces.iter().map(pack).collect(),
        _ => vec![pack(&crate::materials::surface_of(
            ph2d_field_ecs::FieldMaterial::default(),
        ))],
    }
}

/// ⭐⭐ **Desenha o viewport `i`** no tamanho cheio dele.
///
/// `tem_quadro` = o viewport já mostra um quadro (sem ele, mesmo um pedido igual é desenhado).
pub(crate) fn desenha(
    smoke: &mut crate::smoke::Smoke,
    i: usize,
    tamanho: (u32, u32),
    doc: &FieldDoc,
    tem_quadro: bool,
) -> Feito {
    let Some((geracao, objetos, modelos, esperando)) = crate::malha_render_estado::com(|e| {
        (
            e.geracao,
            Arc::clone(&e.objetos),
            e.modelos.clone(),
            e.esperando(),
        )
    }) else {
        return Feito::Espera;
    };
    if esperando {
        return Feito::Espera;
    }
    let Some(desenhista) = desenhista() else {
        return Feito::SemAparelho;
    };

    let reg = crate::smoke::sampled_registry();
    let chao = crate::floor::anchored(&mut smoke.floor, crate::shading::Shading::Render, doc, &reg)
        .map(|g| g.height);
    let luzes: Vec<Luz> = crate::lights::lamps_of(&smoke.lights)
        .into_iter()
        .map(|l| Luz {
            posicao: l.world,
            radiancia_a_um: l.radiance_at_one,
        })
        .collect();
    // ⭐ O raio da PEÇA e os dois passos da curvatura — as MESMAS portas do Render traçado
    // (`smoke_draw_thread`: a bola do documento; `curvatura::assar_canais`: os passos).
    let raio = ph2d_field_eval::bounds::bounding_ball(doc, &reg).map_or(1.0, |b| b.radius);
    let pres = ph2d_field_render::Presentation {
        piece_radius: raio,
        style: smoke.style,
        ..ph2d_field_render::Presentation::of(smoke.look)
    };
    let material_le = smoke.materials.as_ref().is_some_and(|t| {
        ph2d_field_render::curvatura::material_le(&ph2d_field_render::Surfaces {
            all: &t.surfaces,
            owners: None,
        })
    });
    let pedida = crate::malha_render_estado::ChaveCurv {
        geracao,
        eps_material: if material_le {
            ph2d_field_render::curvatura::eps_para(raio)
        } else {
            0.0
        },
        eps_estilo: if pres.reads_curvature() {
            pres.curvature_eps()
        } else {
            0.0
        },
    };
    let curv = if pedida.eps_material > 0.0 || pedida.eps_estilo > 0.0 {
        let c = crate::malha_render_estado::curvatura(pedida).filter(|c| c.0.geracao == geracao);
        if c.is_none() {
            // A 1.ª curvatura destes objetos ainda vem a caminho: sem ela a tinta sairia lisa.
            return Feito::Espera;
        }
        c
    } else {
        None
    };
    // ⭐ O CÉU fotográfico: o atlas monta-se noutra thread da 1.ª vez; até lá o quadro espera, como a
    // curvatura (mostrar o estúdio por um instante seria um piscar que ninguém pediu).
    let ceu = smoke.ceu.sanitized();
    let foto = match ceu.embarcado() {
        Some(e) => match crate::ceu_foto::pronto(e) {
            Some(atlas) => Some((e, atlas)),
            None => return Feito::Espera,
        },
        None => None,
    };
    // ⭐ As TEXTURAS: decodificam noutra thread da 1.ª vez, e o quadro espera por elas como pelo céu.
    let Some((texturas, a_subir)) = crate::texturas::para_o_desenhista() else {
        return Feito::Espera;
    };
    let assinatura = Assinatura {
        geracao,
        texturas,
        modelos,
        materiais: materiais(smoke),
        camera: camera(&smoke.vps[i].cam, tamanho),
        luzes,
        chao,
        look: smoke.look,
        brilho: smoke.bloom.sanitized(),
        ceu,
        estilo: smoke.style.sanitized(),
        raio,
        curv: curv.as_ref().map(|c| c.0),
        tamanho,
    };
    let igual = ULTIMA.with(|u| {
        let mut u = u.borrow_mut();
        if u.len() <= i {
            u.resize(i + 1, None);
        }
        let igual = u[i].as_ref() == Some(&assinatura) && tem_quadro;
        u[i] = Some(assinatura.clone());
        igual
    });
    if igual {
        return Feito::Mesmo;
    }

    let rgba = {
        let Ok(mut d) = desenhista.lock() else {
            return Feito::SemAparelho;
        };
        let (ja, n_antes) = d.subida;
        let fw = &mut d.fw;
        if ja != geracao {
            for (k, o) in objetos.iter().enumerate() {
                let m = &o.malha;
                fw.sobe(
                    k as u64,
                    &Malha {
                        posicoes: &m.posicoes,
                        normais: &m.normais,
                        ao: &m.ao,
                        material: &m.material,
                        indices: &m.indices,
                    },
                );
                if let Some(g) = &o.contacto {
                    fw.sobe_contacto(k as u64, g);
                }
            }
            for k in objetos.len()..n_antes {
                fw.esquece(k as u64);
            }
            d.subida = (geracao, objetos.len());
            d.curv_subida = None;
        }
        if let Some((chave, ks)) = &curv
            && d.curv_subida != Some(*chave)
        {
            for (k, v) in ks.iter().enumerate() {
                d.fw.sobe_curvatura(k as u64, v);
            }
            d.curv_subida = Some(*chave);
        }
        if let Some((e, atlas)) = &foto
            && d.ceu_subido != Some(*e)
        {
            d.fw.sobe_ceu(atlas);
            d.ceu_subido = Some(*e);
        }
        for (camada, mapas) in &a_subir {
            if d.texturas_subidas.insert(*camada) {
                d.fw.sobe_textura(*camada, mapas);
            }
        }
        let fw = &mut d.fw;
        let instancias: Vec<Instancia> = assinatura
            .modelos
            .iter()
            .enumerate()
            .map(|(k, m)| Instancia {
                malha: k as u64,
                modelo: *m,
            })
            .collect();
        fw.quadro(&Cena {
            objetos: &instancias,
            materiais: &assinatura.materiais,
            camera: assinatura.camera,
            luzes: &assinatura.luzes,
            chao: assinatura.chao,
            caixa_tan: Some(caixa_tan()),
            exposicao: assinatura.look.exposure_stops,
            vista: ph2d_view_transform::wgsl::view_code(assinatura.look.view),
            tamanho,
            brilho: assinatura.brilho,
            estilo: assinatura.estilo,
            raio_da_peca: assinatura.raio,
            foto: foto.as_ref().map(|(_, atlas)| assinatura.ceu.foto(atlas)),
            texturas: &assinatura.texturas,
        })
    };
    rgba.map_or(Feito::Espera, Feito::Novo)
}

#[cfg(test)]
#[path = "malha_render_quadro_tests.rs"]
mod tests;
