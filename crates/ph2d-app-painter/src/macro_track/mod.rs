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
//! **A imagem é do artista** (W7 item 1, `docs/Macros/04_plano_W7.md`; W8 item 2a,
//! `docs/Macros/05_plano_W8.md`): numa imagem que RECEBEU a gravação, onde a imagem ATUAL difere da do fim
//! ([`fim`]: a trilha que toca, a água assentada), a ranhura mostra a atual — apagar até ao papel fica
//! apagado; numa que nunca a recebeu, só onde ela difere do fim E do início (o clipe anima por baixo da
//! pintura). Nos outros píxeis, o palco.

use ph2d_ecs::{Entity, Macro, MacroTempo, StableId, World};
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use std::collections::BTreeMap;
use std::sync::Arc;

mod fim;
mod reescrita;
pub use reescrita::{EscreveSprite, LeSprite};
#[cfg(test)]
#[path = "palcos_custo_tests.rs"]
mod palcos_custo_tests;
#[cfg(test)]
#[path = "palcos_tests.rs"]
mod palcos_tests;
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
    fim: fim::FimDaGravacao,
    /// O fim que a imagem ainda tem, depois de uma edição de etapa (W8 item 2b).
    por_reescrever: Option<reescrita::PorReescrever>,
    /// A imagem atual recebeu a gravação, para a chave `(versao_da_atual, versão do fim)`.
    recebeu: Option<((u64, u64), bool)>,
    atual: Option<Atual>,
    /// Sobe a cada [`Atual`] nova.
    versao_da_atual: u64,
    /// A tela do palco premultiplicada, inteira, e o tamanho dela.
    espelho: Vec<u8>,
    tamanho_do_espelho: Option<(u32, u32)>,
    /// O que a ranhura tem: a mistura `(versao_da_atual, versão do fim)` (`None` = só o palco) e a
    /// versão do filme da máscara do Painter ligado por cima (`None` = sem filme).
    mistura_subida: (Option<(u64, u64)>, Option<u64>),
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

/// Os palcos das macros do Painter, por entidade (bits).
#[derive(Default)]
pub struct PalcosDoPainter {
    lidos: BTreeMap<u64, Lido>,
    /// Ranhuras de palcos que morreram ou renasceram, a devolver no próximo [`Self::sobe`].
    soltas: Vec<u32>,
    /// O que sobe à ranhura (reusado entre quadros).
    premul: Vec<u8>,
    /// A transação da última reescrita de uma etapa editada, à espera do passo global (W8 item 2b).
    transacao: Option<u64>,
    /// Os gates seguram o fio do fim para ver o palco antes de ele chegar.
    #[cfg(test)]
    segura_o_fim: bool,
    /// Os gates trocam o cálculo do fim (um que falha).
    #[cfg(test)]
    calcula_o_fim: Option<fim::Calculo>,
}

/// A trilha do Painter de um `Macro.doc` (com as edições das etapas).
fn trilha_do_painter(bytes: &[u8]) -> Option<ph2d_macro::Track> {
    let doc = ph2d_macro::codec::read(bytes).ok()?;
    let familia = ph2d_tool_painter::macro_track::FAMILIA;
    doc.tracks.into_iter().find(|t| t.family == familia)
}

fn palco_de(m: &Macro) -> Result<Option<PalcoDaPintura>, ErroDaMacro> {
    // No tutorial «fazer» a sprite mostra o que o aluno pinta.
    if m.doc.is_empty() || m.modo == ph2d_ecs::ModoDaMacro::Fazer {
        return Ok(None);
    }
    let Some(em_cena) = MacroEmCena::de(&m.doc)? else {
        return Ok(None);
    };
    let Some(trilha) = trilha_do_painter(&m.doc) else {
        return Ok(None);
    };
    Ok(Some(PalcoDaPintura {
        em_cena,
        sprite: None,
        pixeis_da_sprite: None,
        ranhura: None,
        tamanho: None,
        toca: false,
        fim: fim::FimDaGravacao::de(trilha),
        por_reescrever: None,
        recebeu: None,
        atual: None,
        versao_da_atual: 0,
        espelho: Vec::new(),
        tamanho_do_espelho: None,
        mistura_subida: (None, None),
    }))
}

/// O palco de um `Macro` que mudou: o que já tocava REEDITA-SE (recomeça do instantâneo mais tardio que
/// a trilha nova ainda tem, `MacroEmCena::reedita`); o resto nasce do zero. A ranhura de um palco que se
/// vai fica a devolver.
fn relido(velho: Option<Lido>, m: &Macro, tique: u32, soltas: &mut Vec<u32>) -> Lido {
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
        if let Some(trilha) = trilha_do_painter(&m.doc)
            && fim::versao_da_trilha(&trilha) != p.fim.versao
        {
            let mesma_gravacao = fim::versao_crua(&trilha) == p.fim.crua;
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
        palco: palco_de(m),
    }
}

/// Acrescenta a `out` o retângulo `(x, y, w, h)` de uma tela RGBA de largura `largura`, linha a linha.
fn recorta(rgba: &[u8], largura: u32, (x, y, w, h): (u32, u32, u32, u32), out: &mut Vec<u8>) {
    let (x, w) = (x as usize, w as usize);
    let linhas = rgba.chunks_exact(largura as usize * 4).skip(y as usize);
    for linha in linhas.take(h as usize) {
        out.extend_from_slice(&linha[x * 4..(x + w) * 4]);
    }
}

/// Copia o retângulo `c` de `rgba` (straight) para `espelho`, premultiplicado; as duas de largura `largura`.
fn premultiplica_na_caixa(espelho: &mut [u8], rgba: &[u8], largura: u32, c: Caixa) {
    let (lw, x, w) = (largura as usize, c.0 as usize, c.2 as usize);
    for j in c.1 as usize..(c.1 + c.3) as usize {
        let a = (j * lw + x) * 4;
        let linha = &mut espelho[a..a + w * 4];
        linha.copy_from_slice(&rgba[a..a + w * 4]);
        ph2d_render::premul::premultiply_rgba8(linha);
    }
}

/// **A imagem `atual` recebeu a gravação** `g` (W8 item 2a): onde a gravação pinta (o fim difere do
/// início), há mais píxeis iguais ao fim do que ao início. A desfeita, a de outra imagem e a que nunca a
/// teve ficam do lado do início.
fn recebeu_a_gravacao(g: &fim::Gravacao, atual: &[u8]) -> bool {
    let (f, i, at) = (
        g.fim.rgba.as_chunks::<4>().0,
        g.inicio.as_chunks::<4>().0,
        atual.as_chunks::<4>().0,
    );
    let (mut no_fim, mut no_inicio) = (0usize, 0usize);
    for ((f, i), a) in f.iter().zip(i).zip(at) {
        if f != i {
            no_fim += usize::from(a == f);
            no_inicio += usize::from(a == i);
        }
    }
    no_fim > no_inicio
}

/// **A lei da mistura**: em `out` (o retângulo `c` do palco), cada píxel onde a imagem atual difere da do
/// fim — e, se ela não `recebeu` a gravação, também da do início — passa a ser o da atual. Todas
/// premultiplicadas, de largura `largura`.
fn mistura_na_caixa(
    out: &mut [u8],
    g: &fim::Gravacao,
    atual: &[u8],
    recebeu: bool,
    largura: u32,
    c: Caixa,
) {
    let (lw, x, w) = (largura as usize, c.0 as usize, c.2 as usize);
    for (j, o) in out.chunks_exact_mut(w * 4).enumerate().take(c.3 as usize) {
        let a = ((c.1 as usize + j) * lw + x) * 4;
        let r = a..a + w * 4;
        let (f, i, at) = (&g.fim.rgba[r.clone()], &g.inicio[r.clone()], &atual[r]);
        let (f, i, at) = (
            f.as_chunks::<4>().0,
            i.as_chunks::<4>().0,
            at.as_chunks::<4>().0,
        );
        for (k, o) in o.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            if f[k] != at[k] && (recebeu || i[k] != at[k]) {
                *o = at[k];
            }
        }
    }
}

type Caixa = (u32, u32, u32, u32);

/// As pontas da gravação, a imagem atual, e a chave da mistura (versão da atual, versão do fim).
type Mistura = (Arc<fim::Gravacao>, Arc<fim::Imagem>, (u64, u64));

/// O fio do fim de `p`, colhido (pelo cálculo dos gates, se o trocaram).
fn colhe_o_fim(p: &mut PalcoDaPintura, calculo: Option<fim::Calculo>) -> Option<Arc<fim::Gravacao>> {
    match calculo {
        Some(f) => p.fim.colhe_com(f),
        None => p.fim.colhe(),
    }
    .map(Arc::clone)
}

/// A imagem do fim e a atual de `p`, quando as duas existem com o tamanho do espelho, e a chave da
/// mistura. Lê a atual quando a fonte muda (o Painter ligado mudou, ou o palco acordou).
fn mistura_de<R: ?Sized>(
    p: &mut PalcoDaPintura,
    atuais: &mut Atuais<'_, R>,
    ranhuras: &mut R,
    calculo: Option<fim::Calculo>,
) -> Option<Mistura> {
    let gravacao = colhe_o_fim(p, calculo)?;
    let fim = &gravacao.fim;
    let sprite = p.sprite?;
    let fonte = match atuais
        .painter
        .filter(|t| t.bound_document() == Some(sprite))
    {
        Some(t) => Fonte::Painter(t.content_revision(), t.canvas_version()),
        None => Fonte::Sprite,
    };
    if p.atual.as_ref().map(|a| a.fonte) != Some(fonte) {
        let lida = match fonte {
            Fonte::Painter(..) => atuais.painter.and_then(PainterTool::imagem_aplicada),
            Fonte::Sprite => atuais
                .le_sprite
                .as_mut()
                .and_then(|le| le(sprite, ranhuras)),
        };
        p.versao_da_atual += 1;
        p.atual = Some(Atual {
            fonte,
            img: lida.map(|(mut rgba, w, h)| {
                ph2d_render::premul::premultiply_rgba8(&mut rgba);
                Arc::new(fim::Imagem { rgba, w, h })
            }),
        });
    }
    let atual = Arc::clone(p.atual.as_ref()?.img.as_ref()?);
    let dims = p.tamanho_do_espelho?;
    ((fim.w, fim.h) == dims && (atual.w, atual.h) == dims && fim.rgba.len() == atual.rgba.len())
        .then(|| {
            (
                Arc::clone(&gravacao),
                atual,
                (p.versao_da_atual, p.fim.versao),
            )
        })
}

impl PalcosDoPainter {
    /// **O erro da macro do objecto `bits`** — a gravação que não se lê, ou o passo em que o palco
    /// parou (o painel MACROS di-lo ao artista, W5d).
    #[must_use]
    pub fn erro_de(&self, bits: u64) -> Option<&ErroDaMacro> {
        match &self.lidos.get(&bits)?.palco {
            Err(e) => Some(e),
            Ok(p) => {
                let p = p.as_ref()?;
                p.em_cena.erro().or_else(|| p.fim.erro())
            }
        }
    }

    /// **De onde conferir a trilha `track` da macro do objecto `bits`** — o instantâneo do palco dele
    /// ([`MacroEmCena::partida`]); `None` = do zero.
    #[must_use]
    pub fn partida(
        &self,
        bits: u64,
        track: &ph2d_macro::Track,
    ) -> Option<ph2d_tool_painter::macro_track::etapas::Partida> {
        match &self.lidos.get(&bits)?.palco {
            Ok(Some(p)) => p.em_cena.partida(track),
            _ => None,
        }
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
        self.lidos.retain(|bits, l| {
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
            let fresco = self
                .lidos
                .get(&e.to_bits())
                .is_none_or(|l| l.tique != tique);
            if fresco && let Some(m) = world.get::<Macro>(e) {
                let velho = self.lidos.remove(&e.to_bits());
                let lido = relido(velho, m, tique, &mut self.soltas);
                self.lidos.insert(e.to_bits(), lido);
            }
            let Some(Lido {
                palco: Ok(Some(p)), ..
            }) = self.lidos.get_mut(&e.to_bits())
            else {
                continue;
            };
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
            let Some(s) = segundos else {
                continue;
            };
            // Um passo que falha pára a macro nele e fica em `MacroEmCena::erro`: a sprite continua a
            // mostrar o que a macro pintou até lá.
            let _ = p.em_cena.segue(s);
        }
        // Uma mudança depois desta leitura leva um tique NOVO (a shell não corre um `Schedule`).
        world.increment_change_tick();
    }

    /// **Sobe a tela de cada palco que toca e mudou** — só o retângulo que mudou desde a subida anterior,
    /// quando a ranhura já tem a tela daquele tamanho — e devolve as sprites a trocar no extract. Devolve
    /// primeiro as ranhuras dos palcos que se foram.
    ///
    /// `cede` = a sprite que o Painter está a gravar: mostra o traço vivo do editor, não a macro.
    /// `atuais` = de onde vem a imagem atual da sprite (a mistura do W7 item 1).
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
        #[cfg(test)]
        let (segura, calculo) = (self.segura_o_fim, self.calcula_o_fim);
        #[cfg(not(test))]
        let (segura, calculo) = (false, None);
        let mut trocas = Vec::new();
        for l in self.lidos.values_mut() {
            let Ok(Some(p)) = &mut l.palco else {
                continue;
            };
            if !p.toca {
                continue;
            }
            // `Some(None)` = sobe a tela inteira; `Some(Some(c))` = só o retângulo `c`.
            let mut a_subir: Option<Option<Caixa>> = None;
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
                a_subir = Some(caixa);
            }
            let mistura = if segura {
                None
            } else {
                mistura_de(p, &mut atuais, ranhuras, calculo)
            };
            // O Painter ligado a esta sprite com a máscara viva: o filme dele por cima (W8 item 6b).
            let mascara = atuais
                .painter
                .filter(|t| p.sprite.is_some() && t.bound_document() == p.sprite)
                .and_then(|t| Some((t, t.versao_da_mascara()?)));
            let chave = (mistura.as_ref().map(|m| m.2), mascara.map(|m| m.1));
            let recebeu = mistura.as_ref().map(|(g, atual, chave)| {
                match p.recebeu {
                    Some((c, r)) if c == *chave => r,
                    _ => {
                        let r = recebeu_a_gravacao(g, &atual.rgba);
                        p.recebeu = Some((*chave, r));
                        r
                    }
                }
            });
            if chave != p.mistura_subida {
                a_subir = Some(None);
            }
            if let (Some(caixa), Some((w, h))) = (a_subir, p.tamanho_do_espelho) {
                let caixa = caixa.filter(|_| p.tamanho == Some((w, h)));
                let c = caixa.unwrap_or((0, 0, w, h));
                self.premul.clear();
                recorta(&p.espelho, w, c, &mut self.premul);
                if let Some((gravacao, atual, _)) = &mistura {
                    let r = recebeu.unwrap_or(false);
                    mistura_na_caixa(&mut self.premul, gravacao, &atual.rgba, r, w, c);
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
                (p.tamanho, p.mistura_subida) = match (&subida, caixa) {
                    (Ok(()), None) => (Some((w, h)), chave),
                    (Ok(()), Some(_)) => (p.tamanho, chave),
                    (Err(_), _) => (None, chave),
                };
                if let Err(e) = subida {
                    toasts.push(Toast::error(ph2d_i18n::tr_with(
                        "app.painter.painter_bridge_upload.preview_upload_failed",
                        &[("e", &e)],
                    )));
                }
            }
            if let (Some(entity_bits), Some(texture_id)) = (p.sprite, p.ranhura)
                && Some(entity_bits) != cede
            {
                trocas.push(TrocaDaSprite {
                    entity_bits,
                    texture_id,
                });
            }
        }
        trocas
    }
}
