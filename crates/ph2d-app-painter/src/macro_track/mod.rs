//! **As macros do Painter em cena** (ADR-0186, W4 item 5.5 D): cada entidade com um `ph2d_ecs::Macro`
//! cuja trilha `"painter"` a Timeline conduz neste quadro (`ph2d_ecs::MacroTempo`) toca num
//! [`MacroEmCena`] próprio, e a tela dele troca a textura da sprite-alvo no extract — pela MESMA porta
//! da pré-visualização do editor (`PreviewOverride`), à frente dela: uma sprite que a macro toca mostra
//! a macro, nunca o traço do editor.
//!
//! Um palco nasce quando o `Macro` muda (o tique de mudança do componente) e fica em cache enquanto a
//! entidade o tiver; um quadro sem `MacroTempo` tira-o da cena sem o recozer. A ranhura de textura de
//! um palco que morre ou renasce devolve-se no quadro seguinte.
//!
//! **A imagem é do artista** (W9, `docs/Macros/07_plano_W9.md`; a decisão do dono em
//! `docs/Macros/06_auditoria_W8_record.md`): onde o artista escreveu depois da gravação (a marca,
//! [`ph2d_tool_painter::macro_track::marca`]), a ranhura mostra a imagem ATUAL; nos outros texels, o
//! palco. A água que corre depois do Parar não marca: o Rewind mostra o clipe, não a imagem final.

use ph2d_ecs::{Entity, Macro, MacroTempo, StableId, World};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_macro::MacroTime;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::macro_track::marca::{self as marca_do_artista, MarcaDoArtista};
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use std::collections::BTreeMap;
use std::sync::Arc;

/// `PH2D_MACRO_SMOKE=6` — o caminho do dono, sozinho, e a linha de diagnóstico do palco.
pub mod caminho;
mod cauda;
/// O encaixe vivo de uma macro aplicada por cima (W18) e a imagem que uma aplicada faz nascer.
pub mod encaixe_vivo;
mod fim;
mod marcas;
mod mistura;
use mistura::{mistura_na_caixa, premultiplica_na_caixa, recorta};
mod reescrita;
mod trilhas;
pub use reescrita::{EscreveSprite, LeSprite};
use trilhas::{ChaveDoPalco, trilha_do_painter};
#[cfg(test)]
#[path = "palcos_arrasto_tests.rs"]
mod palcos_arrasto_tests;
#[cfg(test)]
#[path = "palcos_custo_tests.rs"]
mod palcos_custo_tests;
#[cfg(test)]
#[path = "palcos_tests.rs"]
mod palcos_tests;
#[cfg(test)]
#[path = "palcos_trilhas_tests.rs"]
mod palcos_trilhas_tests;
/// `PH2D_MACRO_SMOKE=1|4|5` — as cenas do Splash (tocar, o tutorial, a imagem do artista).
pub mod smoke;

/// As ranhuras de textura da cena (o `IndividualTextureStore` do renderer; um duplo nos gates).
pub trait RanhurasDaCena {
    /// Uma ranhura nova com `rgba` premultiplicado.
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn adquire(&mut self, w: u32, h: u32, rgba: &[u8]) -> Result<u32, String>;
    /// Substitui os pixels (e o tamanho) da ranhura `id`.
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn substitui(&mut self, id: u32, w: u32, h: u32, rgba: &[u8]) -> Result<(), String>;
    /// Substitui só o retângulo `(x, y, w, h)` da ranhura `id` (`rgba` é o retângulo, premultiplicado).
    ///
    /// # Errors
    /// O porquê, para o artista.
    fn substitui_regiao(
        &mut self,
        id: u32,
        caixa: (u32, u32, u32, u32),
        rgba: &[u8],
    ) -> Result<(), String>;
    fn solta(&mut self, id: u32);
}

impl RanhurasDaCena for ph2d_render::SpriteRenderer {
    fn adquire(&mut self, w: u32, h: u32, rgba: &[u8]) -> Result<u32, String> {
        self.acquire_individual(w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn substitui(&mut self, id: u32, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
        self.replace_individual_pixels(id, w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn substitui_regiao(
        &mut self,
        id: u32,
        (x, y, w, h): (u32, u32, u32, u32),
        rgba: &[u8],
    ) -> Result<(), String> {
        self.replace_individual_pixels_region(id, x, y, w, h, rgba)
            .map_err(|e| e.to_string())
    }

    fn solta(&mut self, id: u32) {
        self.individual_mut().release(id);
    }
}

/// A sprite cuja textura um palco troca neste quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrocaDaSprite {
    pub entity_bits: u64,
    /// A ranhura com a tela do palco, premultiplicada.
    pub texture_id: u32,
}

/// **De onde vem a imagem ATUAL de uma sprite** (W7 item 1): o Painter do editor quando está ligado a
/// ela (o traço em voo incluído), senão os píxeis dela.
pub struct Atuais<'a, R: ?Sized> {
    pub painter: Option<&'a PainterTool>,
    /// Lê a sprite (bits) em RGBA8 straight; chamado quando o palco acorda.
    #[expect(clippy::type_complexity, reason = "o fecho da shell")]
    pub le_sprite: Option<&'a mut dyn FnMut(u64, &mut R) -> Option<(Vec<u8>, u32, u32)>>,
}

impl<R: ?Sized> Default for Atuais<'_, R> {
    fn default() -> Self {
        Self {
            painter: None,
            le_sprite: None,
        }
    }
}

impl<'a, R: ?Sized> Atuais<'a, R> {
    /// O Painter do registo (se lá estiver) e o leitor da sprite — a composição da shell.
    #[expect(clippy::type_complexity, reason = "o fecho da shell")]
    pub fn de(
        tools: &'a mut ph2d_editor_core::ToolRegistry,
        le: &'a mut dyn FnMut(u64, &mut R) -> Option<(Vec<u8>, u32, u32)>,
    ) -> Self {
        Self {
            painter: crate::painter_bridge_queries::o_painter(tools).map(|t| &*t),
            le_sprite: Some(le),
        }
    }
}

/// De onde a imagem atual de um palco foi lida.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fonte {
    /// `(content_revision, canvas_version)` do Painter ligado.
    Painter((u64, u64), u64),
    Sprite,
}

struct Atual {
    fonte: Fonte,
    /// `None` = não se leu (fica o palco sozinho).
    img: Option<Arc<fim::Imagem>>,
}

/// A chave da marca de uma imagem: o `StableId` da sprite e a época da gravação.
type ChaveDaMarca = (u64, u64);

/// A fonte da textura de uma sprite e a identidade dos píxeis próprios dela: muda a cada edição da
/// imagem (o funil `commit_edited_texture` da shell cunha os dois).
type PixeisDaSprite = (
    Option<ph2d_render::SpriteSource>,
    Option<ph2d_ecs::SpritePixels>,
);

struct PalcoDaPintura {
    em_cena: MacroEmCena,
    /// A sprite-alvo neste quadro (resolvida pelo `StableId` da trilha).
    sprite: Option<u64>,
    /// A identidade dos píxeis dela: a fonte da textura e o `SpritePixels` (W8 item 6).
    pixeis_da_sprite: Option<PixeisDaSprite>,
    ranhura: Option<u32>,
    /// O tamanho da tela na ranhura (uma subida parcial só cabe na mesma).
    tamanho: Option<(u32, u32)>,
    toca: bool,
    /// O que a sprite mostra, entre os palcos do mesmo dono sobre ela ([`trilhas::escolhe_os_mostrados`]).
    mostra: bool,
    fim: fim::FimDaGravacao,
    /// O fim que a imagem ainda tem, depois de uma edição de etapa (W8 item 2b).
    por_reescrever: Option<reescrita::PorReescrever>,
    /// A época da gravação ([`marca_do_artista::epoca_da_gravacao`]).
    epoca: u64,
    /// A chave da marca da sprite-alvo (resolvida no quadro).
    chave: Option<ChaveDaMarca>,
    atual: Option<Atual>,
    /// Sobe a cada [`Atual`] nova.
    versao_da_atual: u64,
    /// A tela do palco premultiplicada, inteira, e o tamanho dela.
    espelho: Vec<u8>,
    tamanho_do_espelho: Option<(u32, u32)>,
    /// O que a ranhura tem: a mistura `(versao_da_atual, versão da marca)` (`None` = só o palco) e a
    /// versão do filme da máscara do Painter ligado por cima (`None` = sem filme).
    mistura_subida: (Option<(u64, u64)>, Option<u64>),
    /// A caixa da marca na última mistura subida (o que volta ao palco se a marca encolher).
    caixa_misturada: Option<Caixa>,
    /// A caixa que a última leitura do Painter trouxe (o resto da imagem atual não se lê).
    caixa_lida: Option<Caixa>,
}

struct Lido {
    tique: u32,
    /// `Ok(None)` = nada gravado, ou a gravação não tem trilha do Painter.
    palco: Result<Option<PalcoDaPintura>, ErroDaMacro>,
}

impl Lido {
    fn ranhura(&self) -> Option<u32> {
        self.palco.as_ref().ok()?.as_ref()?.ranhura
    }
}

/// Os palcos das macros do Painter, por entidade (bits) e trilha do Painter dela.
#[derive(Default)]
pub struct PalcosDoPainter {
    lidos: BTreeMap<ChaveDoPalco, Lido>,
    /// Ranhuras de palcos que morreram ou renasceram, a devolver no próximo [`Self::sobe`].
    soltas: Vec<u32>,
    /// O que sobe à ranhura (reusado entre quadros).
    premul: Vec<u8>,
    /// A transação da última reescrita de uma etapa editada, à espera do passo global (W8 item 2b).
    transacao: Option<u64>,
    /// As marcas do artista por imagem e gravação (W9; [`marcas`]).
    marcas: BTreeMap<ChaveDaMarca, marcas::Guardada>,
    /// As cenas `=5`/`=6`: o que cada ranhura recebeu e as trocas da última subida ([`caminho::Espia`]).
    espia: Option<caminho::Espia>,
    /// O que a água anda no fim de um clipe neste quadro ([`cauda`]).
    dt_da_cauda_ms: f32,
    /// Os gates trocam o cálculo do fim (um que falha).
    #[cfg(test)]
    calcula_o_fim: Option<fim::Calculo>,
}

fn palco_de(m: &Macro, k: usize) -> Result<Option<PalcoDaPintura>, ErroDaMacro> {
    // No tutorial «fazer» a sprite mostra o que o aluno pinta.
    if m.doc.is_empty() || m.modo == ph2d_ecs::ModoDaMacro::Fazer {
        return Ok(None);
    }
    let razao = ph2d_tool_painter::macro_track::instantaneos::razao();
    let Some(em_cena) = MacroEmCena::da_trilha(&m.doc, k, razao)? else {
        return Ok(None);
    };
    let Some(trilha) = trilha_do_painter(&m.doc, k) else {
        return Ok(None);
    };
    let epoca = marca_do_artista::epoca_da_gravacao(&trilha);
    Ok(Some(PalcoDaPintura {
        em_cena,
        sprite: None,
        pixeis_da_sprite: None,
        ranhura: None,
        tamanho: None,
        toca: false,
        mostra: true,
        fim: fim::FimDaGravacao::de(trilha),
        por_reescrever: None,
        epoca,
        chave: None,
        atual: None,
        versao_da_atual: 0,
        espelho: Vec::new(),
        tamanho_do_espelho: None,
        mistura_subida: (None, None),
        caixa_misturada: None,
        caixa_lida: None,
    }))
}

/// O palco de um `Macro` que mudou: o que já tocava REEDITA-SE (recomeça do instantâneo mais tardio que
/// a trilha nova ainda tem, `MacroEmCena::reedita`); o resto nasce do zero. A ranhura de um palco que se
/// vai fica a devolver.
fn relido(velho: Option<Lido>, m: &Macro, k: usize, tique: u32, soltas: &mut Vec<u32>) -> Lido {
    if let Some(Lido {
        palco: Ok(Some(mut p)),
        ..
    }) = velho
    {
        let palco = if m.doc.is_empty() {
            Ok(None)
        } else {
            p.em_cena.reedita(&m.doc).map(|ainda| ainda.then_some(()))
        };
        if let Some(trilha) = trilha_do_painter(&m.doc, k)
            && fim::versao_da_trilha(&trilha) != p.fim.versao
        {
            let mesma_gravacao = fim::versao_crua(&trilha) == p.fim.crua;
            // Gravar de novo é outra época: outra marca (a antiga fica guardada para o desfazer).
            p.epoca = marca_do_artista::epoca_da_gravacao(&trilha);
            let antigo = std::mem::replace(&mut p.fim, fim::FimDaGravacao::de(trilha));
            // Uma edição de etapa: a imagem ainda tem o fim antigo (o mais antigo, se já esperava).
            if !mesma_gravacao {
                p.por_reescrever = None;
            } else if p.por_reescrever.is_none() {
                p.por_reescrever = Some(reescrita::PorReescrever::de(antigo));
            }
        }
        return match palco {
            Ok(Some(())) => Lido {
                tique,
                palco: Ok(Some(p)),
            },
            outro => {
                soltas.extend(p.ranhura);
                Lido {
                    tique,
                    palco: outro.map(|_| None),
                }
            }
        };
    }
    soltas.extend(velho.and_then(|l| l.ranhura()));
    Lido {
        tique,
        palco: palco_de(m, k),
    }
}

type Caixa = (u32, u32, u32, u32);

/// O que sobe: nada, a tela inteira, ou só um retângulo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Subida {
    Nada,
    Inteira,
    Caixa(Caixa),
}

impl Subida {
    fn com(self, outra: Self) -> Self {
        match (self, outra) {
            (Self::Nada, x) | (x, Self::Nada) => x,
            (Self::Inteira, _) | (_, Self::Inteira) => Self::Inteira,
            (Self::Caixa(a), Self::Caixa(b)) => Self::Caixa(marca_do_artista::une(Some(a), b)),
        }
    }

    fn da(caixa: Option<Caixa>) -> Self {
        caixa.map_or(Self::Nada, Self::Caixa)
    }
}

/// **A imagem atual de `p` e a marca que vale sobre ela**, quando há o que misturar: a marca do Painter
/// ligado à sprite (lida dele, viva) ou a da sprite ([`marcas`]). Do Painter lê-se só a caixa da marca,
/// e só com ela por vazia (W9, A3: a água que corre sem marca não custa nada; com marca, a caixa).
fn atual_e_marca<R: ?Sized>(
    p: &mut PalcoDaPintura,
    guardadas: &mut BTreeMap<ChaveDaMarca, marcas::Guardada>,
    atuais: &mut Atuais<'_, R>,
    ranhuras: &mut R,
) -> Option<(Arc<fim::Imagem>, MarcaDoArtista)> {
    let (sprite, chave) = (p.sprite?, p.chave?);
    let painter = atuais
        .painter
        .filter(|t| t.bound_document() == Some(sprite));
    let marca = match painter {
        Some(t) => t
            .marca_do_artista()
            .filter(|m| m.epoca() == chave.1)
            .cloned()?,
        None => {
            marcas::le_a_sprite(p, guardadas, atuais.le_sprite.as_deref_mut(), ranhuras);
            guardadas.get(&chave)?.efetiva.clone()?
        }
    };
    if marca.vazia() {
        return None;
    }
    if let Some(t) = painter {
        let fonte = Fonte::Painter(t.content_revision(), t.canvas_version());
        let caixa = marca.caixa()?;
        if p.atual.as_ref().map(|a| a.fonte) != Some(fonte) || p.caixa_lida != Some(caixa) {
            let (w, h) = p.tamanho_do_espelho?;
            // A caixa é em texels do Painter: só se copia com a imagem do tamanho do palco.
            if t.canvas_size() != (w, h) || marca.tamanho() != (w, h) {
                return None;
            }
            let mut img = match p.atual.take() {
                Some(Atual {
                    fonte: Fonte::Painter(..),
                    img: Some(a),
                }) if (a.w, a.h) == (w, h) => Arc::unwrap_or_clone(a),
                _ => fim::Imagem {
                    rgba: vec![0; (w * h * 4) as usize],
                    w,
                    h,
                },
            };
            let lida = t.imagem_aplicada_na_caixa(caixa).map(|mut c| {
                ph2d_render::premul::premultiply_rgba8(&mut c);
                let (x, y, cw, _) = caixa;
                for (j, linha) in c.chunks_exact((cw * 4) as usize).enumerate() {
                    let a = (((y as usize + j) * w as usize) + x as usize) * 4;
                    img.rgba[a..a + linha.len()].copy_from_slice(linha);
                }
            });
            p.versao_da_atual += 1;
            p.caixa_lida = lida.map(|()| caixa);
            p.atual = Some(Atual {
                fonte,
                img: p.caixa_lida.map(|_| Arc::new(img)),
            });
        }
    }
    let atual = Arc::clone(p.atual.as_ref()?.img.as_ref()?);
    let dims = p.tamanho_do_espelho?;
    ((atual.w, atual.h) == dims && marca.tamanho() == dims && atual.rgba.len() == p.espelho.len())
        .then_some((atual, marca))
}

/// Um palco segue o quadro: a sprite e a marca dele, o encaixe vivo e o relógio da macro.
fn segue_o_palco(p: &mut PalcoDaPintura, world: &World, e: Entity, segundos: Option<f64>) {
    let acordou = segundos.is_some() && !p.toca;
    p.toca = segundos.is_some();
    // A sprite resolve-se mesmo parado: a reescrita de uma etapa editada é fora do Play.
    let alvo = p.em_cena.alvo();
    let sprite = if alvo == 0 {
        Some(e)
    } else {
        ph2d_ecs::entity_with_stable_id(world, StableId(alvo))
    }
    .map(Entity::to_bits);
    // A sprite lê-se de novo quando o palco acorda ou os píxeis dela mudam.
    let pixeis = sprite.map(Entity::from_bits).map(|s| {
        (
            world.get::<ph2d_render::Sprite>(s).map(|x| x.source),
            world.get::<ph2d_ecs::SpritePixels>(s).copied(),
        )
    });
    if acordou || sprite != p.sprite || pixeis != p.pixeis_da_sprite {
        p.atual = None;
    }
    p.sprite = sprite;
    p.pixeis_da_sprite = pixeis;
    // A marca é da sprite e da gravação: o `StableId` dela (o do próprio objecto, sem alvo).
    let sid = if alvo == 0 {
        world.get::<StableId>(e).map(|s| s.0)
    } else {
        Some(alvo)
    };
    p.chave = sid.or(sprite).map(|s| (s, p.epoca));
    // Por cima (W18): o gizmo do objecto Macro diz, a cada quadro, onde a gravação cai no alvo.
    if let (Some(g), Some(s)) = (p.em_cena.tamanho_gravado(), sprite) {
        let tela = p.em_cena.tocador().tool().source_size();
        let e = encaixe_vivo::do_mundo(world, e, Entity::from_bits(s), tela, g);
        p.em_cena.poe_encaixe(e);
    }
    let Some(s) = segundos else {
        return;
    };
    // Um passo que falha pára a macro nele e fica em `MacroEmCena::erro`: a sprite continua a
    // mostrar o que a macro pintou até lá.
    let _ = p.em_cena.segue(s);
}

impl PalcosDoPainter {
    /// **O erro da macro do objecto `bits`** — a gravação que não se lê, ou o passo em que o palco
    /// parou (o painel MACROS di-lo ao artista, W5d).
    #[must_use]
    pub fn erro_de(&self, bits: u64) -> Option<&ErroDaMacro> {
        self.lidos
            .range((bits, 0)..=(bits, usize::MAX))
            .find_map(|(_, l)| match &l.palco {
                Err(e) => Some(e),
                Ok(p) => {
                    let p = p.as_ref()?;
                    p.em_cena.erro().or_else(|| p.fim.erro())
                }
            })
    }

    /// **De onde conferir a trilha `track` da macro do objecto `bits`** — o instantâneo do palco dele
    /// ([`MacroEmCena::partida`]); `None` = do zero.
    #[must_use]
    pub fn partida(
        &self,
        bits: u64,
        track: &ph2d_macro::Track,
    ) -> Option<ph2d_tool_painter::macro_track::etapas::Partida> {
        // A partida só vale sobre a mesma inicial e o mesmo alvo: o palco da própria trilha.
        self.lidos
            .range((bits, 0)..=(bits, usize::MAX))
            .find_map(|(_, l)| match &l.palco {
                Ok(Some(p)) => p.em_cena.partida(track),
                _ => None,
            })
    }

    /// **Por quadro**, depois de a Timeline escrever o `MacroTempo` e antes do extract: (re)cria o
    /// palco de um `Macro` que mudou e segue o relógio de cada um que a Timeline conduz.
    pub fn segue(&mut self, world: &mut World) {
        let mut q = world.query::<(Entity, &Macro, Option<&MacroTempo>)>();
        let vistos: Vec<(Entity, Option<f64>)> = q
            .iter(world)
            .map(|(e, _, t)| (e, t.map(|t| t.segundos)))
            .collect();
        let soltas = &mut self.soltas;
        self.lidos.retain(|(bits, _), l| {
            let vivo = vistos.iter().any(|(e, _)| e.to_bits() == *bits);
            if !vivo {
                soltas.extend(l.ranhura());
            }
            vivo
        });
        for (e, segundos) in vistos {
            let tique = world
                .entity(e)
                .get_change_ticks::<Macro>()
                .map_or(0, |t| t.changed.get());
            let bits = e.to_bits();
            let fresco = self.lidos.get(&(bits, 0)).is_none_or(|l| l.tique != tique);
            if fresco && let Some(m) = world.get::<Macro>(e) {
                // Uma trilha do Painter, um palco; a 1.ª existe sempre (diz o erro de um doc que não se lê).
                let n = trilhas::do_painter(&m.doc).len().max(1);
                let sobram: Vec<ChaveDoPalco> = trilhas::do_dono(&mut self.lidos, bits)
                    .map(|(c, _)| *c)
                    .filter(|c| c.1 >= n)
                    .collect();
                for c in sobram {
                    self.soltas
                        .extend(self.lidos.remove(&c).and_then(|l| l.ranhura()));
                }
                for k in 0..n {
                    let velho = self.lidos.remove(&(bits, k));
                    let lido = relido(velho, m, k, tique, &mut self.soltas);
                    self.lidos.insert((bits, k), lido);
                }
            }
            for (_, l) in trilhas::do_dono(&mut self.lidos, bits) {
                let Ok(Some(p)) = &mut l.palco else {
                    continue;
                };
                segue_o_palco(p, world, e, segundos);
            }
            if let Some(s) = segundos {
                trilhas::escolhe_os_mostrados(&mut self.lidos, bits, MacroTime::from_secs_f64(s));
            }
        }
        // Uma mudança depois desta leitura leva um tique NOVO (a shell não corre um `Schedule`).
        world.increment_change_tick();
    }

    /// **Sobe a tela de cada palco que toca e mudou** — só o retângulo que mudou desde a subida anterior
    /// (o do palco e o da marca), quando a ranhura já tem a tela daquele tamanho — e devolve as sprites a
    /// trocar no extract. Devolve primeiro as ranhuras dos palcos que se foram.
    ///
    /// `cede` = a sprite que o Painter está a gravar: mostra o traço vivo do editor, não a macro.
    /// `atuais` = de onde vem a imagem atual da sprite (a mistura pela marca, W9).
    pub fn sobe<R: RanhurasDaCena + ?Sized>(
        &mut self,
        ranhuras: &mut R,
        toasts: &mut ToastQueue,
        cede: Option<u64>,
        mut atuais: Atuais<'_, R>,
    ) -> Vec<TrocaDaSprite> {
        for id in self.soltas.drain(..) {
            ranhuras.solta(id);
        }
        let mut trocas = Vec::new();
        for l in self.lidos.values_mut() {
            let Ok(Some(p)) = &mut l.palco else {
                continue;
            };
            if !p.toca {
                continue;
            }
            cauda::anda(p, self.dt_da_cauda_ms, &self.marcas, atuais.painter);
            let mut a_subir = Subida::Nada;
            if let Some((rgba, w, h)) = p.em_cena.toma_a_tela() {
                let caixa = p.em_cena.caixa_da_tela().filter(|&(_, _, cw, ch)| {
                    p.tamanho_do_espelho == Some((w, h)) && cw > 0 && ch > 0
                });
                match caixa {
                    Some(c) => premultiplica_na_caixa(&mut p.espelho, &rgba, w, c),
                    None => {
                        p.espelho.clear();
                        p.espelho.extend_from_slice(&rgba);
                        ph2d_render::premul::premultiply_rgba8(&mut p.espelho);
                        p.tamanho_do_espelho = Some((w, h));
                    }
                }
                a_subir = caixa.map_or(Subida::Inteira, Subida::Caixa);
            }
            let mistura = atual_e_marca(p, &mut self.marcas, &mut atuais, ranhuras);
            // O Painter ligado a esta sprite com a máscara viva: o filme dele por cima (W8 item 6b).
            let mascara = atuais
                .painter
                .filter(|t| p.sprite.is_some() && t.bound_document() == p.sprite)
                .and_then(|t| Some((t, t.versao_da_mascara()?)));
            let caixa_da_marca = mistura.as_ref().and_then(|(_, m)| m.caixa());
            let chave = (
                mistura
                    .as_ref()
                    .map(|(_, m)| (p.versao_da_atual, m.versao())),
                mascara.map(|m| m.1),
            );
            if chave.1 != p.mistura_subida.1 {
                a_subir = Subida::Inteira;
            } else if chave.0 != p.mistura_subida.0 {
                // Só o que a marca cobre agora ou cobria na subida anterior muda.
                a_subir = a_subir
                    .com(Subida::da(caixa_da_marca))
                    .com(Subida::da(p.caixa_misturada));
            }
            if let (true, Some((w, h))) = (a_subir != Subida::Nada, p.tamanho_do_espelho) {
                let caixa = match a_subir {
                    Subida::Caixa(c) if p.tamanho == Some((w, h)) => Some(c),
                    _ => None,
                };
                let c = caixa.unwrap_or((0, 0, w, h));
                self.premul.clear();
                recorta(&p.espelho, w, c, &mut self.premul);
                if let Some((atual, marca)) = &mistura {
                    mistura_na_caixa(&mut self.premul, marca, &atual.rgba, w, c);
                }
                if let Some((t, _)) = mascara {
                    ph2d_render::premul::unpremultiply_rgba8(&mut self.premul);
                    t.tinge_com_a_mascara(&mut self.premul, c);
                    ph2d_render::premul::premultiply_rgba8(&mut self.premul);
                }
                let subida = match (p.ranhura, caixa) {
                    (Some(id), Some(c)) => ranhuras.substitui_regiao(id, c, &self.premul),
                    (Some(id), None) => ranhuras.substitui(id, w, h, &self.premul),
                    (None, _) => ranhuras.adquire(w, h, &self.premul).map(|id| {
                        p.ranhura = Some(id);
                    }),
                };
                if let (Some(e), Ok(()), Some(id)) = (&mut self.espia, &subida, p.ranhura) {
                    e.copia(id, w, caixa, &self.premul);
                }
                (p.tamanho, p.mistura_subida, p.caixa_misturada) = match (&subida, caixa) {
                    (Ok(()), None) => (Some((w, h)), chave, caixa_da_marca),
                    (Ok(()), Some(_)) => (p.tamanho, chave, caixa_da_marca),
                    (Err(_), _) => (None, chave, caixa_da_marca),
                };
                if let Err(e) = subida {
                    toasts.push(Toast::error(ph2d_i18n::tr_with(
                        "app.painter.painter_bridge_upload.preview_upload_failed",
                        &[("e", &e)],
                    )));
                }
            }
            if let (true, Some(entity_bits), Some(texture_id)) = (p.mostra, p.sprite, p.ranhura)
                && Some(entity_bits) != cede
            {
                trocas.push(TrocaDaSprite {
                    entity_bits,
                    texture_id,
                });
            }
        }
        if let Some(e) = &mut self.espia {
            e.trocas.clone_from(&trocas);
        }
        trocas
    }
}
