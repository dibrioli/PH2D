//! ⭐⭐⭐ **AS TEXTURAS DAS FORMAS** no Render por malha ([`ph2d_triplanar`], handoff `AS_TEXTURAS`).
//!
//! | pergunta | porta |
//! |---|---|
//! | que textura tem cada folha, e onde está a folha? | [`sync`] → [`folhas`] (a ordem é a dos materiais) |
//! | os mapas já estão prontos? | [`estado`]: decodifica NOUTRA thread da 1.ª vez; o quadro espera |
//! | o artista importa um ficheiro | [`pede_importar`] (o painel) → [`atende_pedido`] (o app abre o diálogo) → [`aplica_escolhidos`] (a ponte escreve no mundo) |
//!
//! ⚠️ Os ficheiros viajam por CAMINHO (o precedente da escultura importada); um que sumiu ou não
//! decodifica diz-se uma vez e a forma fica sem textura.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_field_ecs::FieldTexture;
use ph2d_triplanar::{Embarcada, Mapas};

/// De onde vêm os mapas de uma textura.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fonte {
    /// O índice em [`Embarcada::TODAS`].
    Embarcada(usize),
    /// Os caminhos importados (`""` = sem esse mapa).
    Ficheiros {
        cor: String,
        normal: String,
        rugosidade: String,
    },
}

/// ⭐ A textura de UMA folha neste quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct DaFolha {
    pub fonte: Fonte,
    /// O tamanho do ladrilho já resolvido (`0` no documento = o tamanho real da embutida).
    pub tile: f32,
    pub blend: f32,
    pub bump: f32,
    /// Mundo → espaço da folha (linhas de uma afim `3 × 4`).
    pub mundo_para_folha: [[f32; 4]; 3],
}

/// A fonte de uma [`FieldTexture`] — `None` sem textura, ou «de ficheiro» sem cor.
#[must_use]
pub fn fonte_de(t: &FieldTexture) -> Option<Fonte> {
    match t.source {
        0 => None,
        s if s == ph2d_field::TEXTURE_FROM_FILE => (!t.color_file.is_empty()).then(|| Fonte::Ficheiros {
            cor: t.color_file.clone(),
            normal: t.normal_file.clone(),
            rugosidade: t.roughness_file.clone(),
        }),
        s => {
            let i = usize::from(s) - 1;
            (i < Embarcada::TODAS.len()).then_some(Fonte::Embarcada(i))
        }
    }
}

/// ⭐ Mundo → folha: `p_folha = Rᵀ (p − t) / s`, em linhas.
#[must_use]
pub fn mundo_para_folha(x: ph2d_field::Xform) -> [[f32; 4]; 3] {
    let r = ph2d_field::Xform {
        translation: [0.0; 3],
        scale: 1.0,
        ..x
    };
    let s = if x.scale != 0.0 { 1.0 / x.scale } else { 1.0 };
    let cols = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]].map(|e| r.apply_dir(e));
    let t = x.translation;
    std::array::from_fn(|i| {
        let l = cols[i].map(|c| c * s);
        [l[0], l[1], l[2], -(l[0] * t[0] + l[1] * t[1] + l[2] * t[2])]
    })
}

/// ⭐ **A textura de cada folha**, na ordem de [`crate::materials::folhas`] (o índice do material).
#[must_use]
pub fn das_folhas(world: &World, root: Entity) -> Vec<Option<DaFolha>> {
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter(|(e, _)| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .map(|(e, _)| {
            let t = world.get::<FieldTexture>(e)?;
            let fonte = fonte_de(t)?;
            let tile = match (&fonte, t.tile > 0.0) {
                (_, true) => t.tile,
                (Fonte::Embarcada(i), false) => Embarcada::TODAS[*i].tamanho_real(),
                (Fonte::Ficheiros { .. }, false) => 1.0,
            };
            Some(DaFolha {
                fonte,
                tile,
                blend: t.blend,
                bump: t.bump,
                mundo_para_folha: mundo_para_folha(ph2d_field_ecs::world_xform(world, e)),
            })
        })
        .collect()
}

thread_local! {
    static FOLHAS: RefCell<Vec<Option<DaFolha>>> = const { RefCell::new(Vec::new()) };
}

/// ⭐ Uma vez por quadro, ao lado do [`crate::materials::sync`]: as texturas seguem a peça.
pub(crate) fn sync(sim: &mut ph2d_ecs::SimWorld) {
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).next().map(|(e, _)| e)
    };
    let v = root.map_or_else(Vec::new, |r| das_folhas(sim.world(), r));
    FOLHAS.with(|f| *f.borrow_mut() = v);
}

/// As texturas do último [`sync`].
#[must_use]
pub fn folhas() -> Vec<Option<DaFolha>> {
    FOLHAS.with(|f| f.borrow().clone())
}

/// ⭐ Os mapas de uma fonte, e a camada da placa onde eles vivem.
#[derive(Clone)]
pub enum Estado {
    /// A decodificar noutra thread.
    Carregando,
    Pronta {
        camada: u32,
        mapas: Arc<Mapas>,
        /// Altura/largura da imagem de cor.
        aspecto: f32,
    },
    /// Não decodificou (a frase vai para o artista uma vez).
    Falhou(String),
}

#[derive(Default)]
struct Cache {
    estados: BTreeMap<Fonte, Estado>,
    proxima: u32,
}

fn cache() -> &'static Mutex<Cache> {
    static C: OnceLock<Mutex<Cache>> = OnceLock::new();
    C.get_or_init(Mutex::default)
}

/// ⭐⭐ **O estado dos mapas de `f`** — da 1.ª vez lança a decodificação NOUTRA thread.
#[must_use]
pub fn estado(f: &Fonte) -> Estado {
    let Ok(mut c) = cache().lock() else {
        return Estado::Falhou(String::new());
    };
    if let Some(e) = c.estados.get(f) {
        return e.clone();
    }
    c.estados.insert(f.clone(), Estado::Carregando);
    let camada = c.proxima;
    c.proxima += 1;
    let f2 = f.clone();
    let lancou = std::thread::Builder::new()
        .name("ph2d-textura".to_owned())
        .spawn(move || {
            let e = match carrega(&f2) {
                Ok((mapas, aspecto)) => Estado::Pronta {
                    camada,
                    mapas: Arc::new(mapas),
                    aspecto,
                },
                Err(msg) => {
                    crate::notice::say(ph2d_i18n::tr_with(
                        "app.field3d.texture.could_not_read",
                        &[("why", &msg)],
                    ));
                    Estado::Falhou(msg)
                }
            };
            if let Ok(mut c) = cache().lock() {
                c.estados.insert(f2, e);
            }
        });
    if lancou.is_err() {
        c.estados.remove(f);
    }
    Estado::Carregando
}

fn carrega(f: &Fonte) -> Result<(Mapas, f32), String> {
    match f {
        Fonte::Embarcada(i) => Ok((Embarcada::TODAS[*i].mapas()?, 1.0)),
        Fonte::Ficheiros {
            cor,
            normal,
            rugosidade,
        } => {
            let lado = ph2d_triplanar::LADO;
            let c = le(cor)?;
            let aspecto = c.1 as f32 / c.0.max(1) as f32;
            let plana: ph2d_triplanar::Imagem = (1, 1, vec![[128, 128, 255, 255]]);
            let n = if normal.is_empty() { plana } else { le(normal)? };
            let r = if rugosidade.is_empty() {
                None
            } else {
                Some(le(rugosidade)?)
            };
            Ok((
                Mapas {
                    cor: ph2d_triplanar::Mipmaps::de_rgba8(c.0, c.1, &c.2, lado, true),
                    nrh: ph2d_triplanar::junta_nrh(&n, r.as_ref(), lado),
                    tem_normal: !normal.is_empty(),
                    tem_rugosidade: !rugosidade.is_empty(),
                },
                aspecto,
            ))
        }
    }
}

/// Uma camada a subir para a placa.
pub type ASubir = (u32, Arc<Mapas>);

/// ⭐⭐ **As texturas do quadro, no formato do desenhista** (o índice é o do material) e as camadas
/// que elas leem — `None` enquanto alguma ainda decodifica (o quadro espera, como o céu).
#[must_use]
pub fn para_o_desenhista() -> Option<(Vec<Option<ph2d_mesh_forward::TexturaMaterial>>, Vec<ASubir>)> {
    let mut subir: Vec<ASubir> = Vec::new();
    let mut fora = Vec::new();
    for f in folhas() {
        let Some(f) = f else {
            fora.push(None);
            continue;
        };
        match estado(&f.fonte) {
            Estado::Carregando => return None,
            Estado::Falhou(_) => fora.push(None),
            Estado::Pronta {
                camada,
                mapas,
                aspecto,
            } => {
                fora.push(Some(ph2d_mesh_forward::TexturaMaterial {
                    camada,
                    triplanar: ph2d_triplanar::Triplanar {
                        tamanho: f.tile,
                        aspecto,
                        blend: f.blend,
                        relevo: f.bump,
                    },
                    tem_normal: mapas.tem_normal,
                    tem_rugosidade: mapas.tem_rugosidade,
                    mundo_para_folha: f.mundo_para_folha,
                }));
                if !subir.iter().any(|(c, _)| *c == camada) {
                    subir.push((camada, mapas));
                }
            }
        }
    }
    Some((fora, subir))
}

/// Os formatos de imagem que uma textura importada aceita.
pub const EXTENSOES: [&str; 6] = ["png", "jpg", "jpeg", "webp", "tga", "tif"];

/// Lê e decodifica uma imagem plana (8 bits) do disco.
fn le(caminho: &str) -> Result<ph2d_triplanar::Imagem, String> {
    use ph2d_imageio::{ImageImporter, ImportOpts, MagicHint, MagicMatch};
    let b = std::fs::read(caminho).map_err(|e| format!("{caminho}: {e}"))?;
    let importadores: [&dyn ImageImporter; 4] = [
        &ph2d_imageio_png::PngImporter,
        &ph2d_imageio_jpeg::JpegImporter,
        &ph2d_imageio_webp::WebpImporter,
        &ph2d_imageio_tga::TgaImporter,
    ];
    let imp = importadores
        .into_iter()
        .find(|i| i.supports(MagicHint::Bytes(&b)) != MagicMatch::None)
        .ok_or_else(|| format!("{caminho}: formato"))?;
    match imp
        .import(&b, &ImportOpts::default())
        .map_err(|e| format!("{caminho}: {e}"))?
    {
        ph2d_imageio::DecodedImage::Flat(i) => {
            Ok((i.width, i.height, i.pixels.iter().map(|p| p.0).collect()))
        }
        _ => Err(format!("{caminho}: não é uma imagem plana")),
    }
}

thread_local! {
    static SEMENTE: RefCell<Option<Vec<Option<FieldTexture>>>> = const { RefCell::new(None) };
}

/// ⭐ A textura que uma cena de smoke pede, folha a folha — plantada UMA vez ([`planta`]), como a
/// semente dos materiais.
pub fn semeia(v: Option<Vec<Option<FieldTexture>>>) {
    SEMENTE.with(|s| *s.borrow_mut() = v);
}

/// Planta a semente nas folhas da peça (a ordem do [`das_folhas`]) e gasta-a.
pub(crate) fn planta(world: &mut World, root: Entity) {
    let Some(v) = SEMENTE.with(|s| s.borrow_mut().take()) else {
        return;
    };
    let folhas: Vec<Entity> = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter(|(e, _)| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            )
        })
        .map(|(e, _)| e)
        .collect();
    for (e, t) in folhas.into_iter().zip(v) {
        if let Some(t) = t {
            world.entity_mut(e).insert(t);
        }
    }
}

/// Que mapa um pedido de importação escolhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Canal {
    Cor,
    Normal,
    Rugosidade,
}

thread_local! {
    static PEDIDO: std::cell::Cell<Option<(u64, Canal)>> = const { std::cell::Cell::new(None) };
    static ESCOLHIDO: RefCell<Option<(u64, Canal, String)>> = const { RefCell::new(None) };
}

/// ⭐ O painel pede o diálogo (o 1.º salto).
pub fn pede_importar(entity: u64, canal: Canal) {
    PEDIDO.with(|p| p.set(Some((entity, canal))));
}

/// O pedido, tirado uma vez (os gates de costura leem-no).
#[must_use]
pub fn take_pedido() -> Option<(u64, Canal)> {
    PEDIDO.with(std::cell::Cell::take)
}

/// ⭐⭐ **O app abre o diálogo** (o 2.º salto) e confere que a imagem decodifica antes de a aceitar.
pub fn atende_pedido(toasts: &mut ph2d_editor_core::ToastQueue) {
    let Some((entity, canal)) = take_pedido() else {
        return;
    };
    let dialog = rfd::FileDialog::new().add_filter(
        ph2d_i18n::tr("app.field3d.texture.images"),
        &EXTENSOES,
    );
    let Some(path) = ph2d_app_host::modal::pick_file(dialog) else {
        return;
    };
    let caminho = path.to_string_lossy().into_owned();
    match le(&caminho) {
        Ok(_) => escolhe(entity, canal, caminho),
        Err(msg) => {
            toasts.push(ph2d_editor_core::Toast::error(ph2d_i18n::tr_with(
                "app.field3d.texture.could_not_read",
                &[("why", &msg)],
            )));
        }
    }
}

/// O ficheiro escolhido à espera da ponte (testes de costura).
pub fn escolhe(entity: u64, canal: Canal, caminho: String) {
    ESCOLHIDO.with(|e| *e.borrow_mut() = Some((entity, canal, caminho)));
}

/// ⭐⭐⭐ **A ponte escreve o ficheiro escolhido** (o 3.º salto), na selecção inteira como o
/// material (`alcance` = o `material_reach` do painel) — e é um passo de undo como qualquer edição.
pub(crate) fn aplica_escolhidos(world: &mut World, alcance: impl Fn(&World, u64) -> Vec<Entity>) {
    let Some((entity, canal, caminho)) = ESCOLHIDO.with(|e| e.borrow_mut().take()) else {
        return;
    };
    for alvo in alcance(world, entity) {
        let mut t = world.get::<FieldTexture>(alvo).cloned().unwrap_or_default();
        match canal {
            Canal::Cor => {
                t.color_file.clone_from(&caminho);
                t.source = ph2d_field::TEXTURE_FROM_FILE;
            }
            Canal::Normal => t.normal_file.clone_from(&caminho),
            Canal::Rugosidade => t.roughness_file.clone_from(&caminho),
        }
        if let Ok(mut e) = world.get_entity_mut(alvo) {
            e.insert(t);
        }
    }
    crate::smoke::mark_authored_change();
}

#[cfg(test)]
#[path = "texturas_tests.rs"]
mod tests;
