//! **O canal entre as janelas e a família.** A família diz o que há ([`Contexto`], [`Imagem`]) e qual
//! janela abrir; a janela devolve o verbo ([`Pedido`]) e as opções da folha, para a prévia ao vivo.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

use ph2d_editor_core::progress::{Job, JobQueue, Progress};
use ph2d_pixel_core::{LayerId, Sprite};
use ph2d_pixel_io::abrir::Grelha;
use ph2d_pixel_io::exporta::{self, ErroGravar, Export, Gravado};
use ph2d_pixel_io::sheet::OpcoesFolha;

/// A janela aberta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tela {
    #[default]
    ExportAs,
    Folha,
    Import,
}

/// O que as janelas precisam de saber do sprite em Draw.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contexto {
    pub lados: (u32, u32),
    pub quadros: u32,
    pub camadas: Vec<(LayerId, String)>,
    pub tags: Vec<String>,
}

/// Uma imagem para um cartão (RGBA de alfa reto): a folha ao vivo ou a do Import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imagem {
    pub rgba: Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
    /// Muda a cada imagem nova (a chave da imagem da GPU).
    pub seq: u64,
    /// O tamanho que sai no ficheiro (a prévia pode vir reduzida).
    pub lados_reais: (u32, u32),
    /// Quantos quadros a folha tem; 0 no Import.
    pub entradas: usize,
    /// A grelha que a família sugere (só no Import).
    pub sugestao: Option<Grelha>,
}

/// O verbo de uma janela.
#[derive(Debug, Clone, PartialEq)]
pub enum Pedido {
    Exportar(Export),
    Importar(Grelha),
}

#[derive(Default)]
struct Canal {
    abrir: Option<Tela>,
    contexto: Contexto,
    imagem: Option<Imagem>,
    pedido: Option<Pedido>,
    folha_pedida: Option<OpcoesFolha>,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

/// A família pede esta janela.
pub fn abrir(t: Tela) {
    com(|c| c.abrir = Some(t));
}

pub(crate) fn take_abrir() -> Option<Tela> {
    com(|c| c.abrir.take())
}

pub fn publish_contexto(x: Contexto) {
    com(|c| c.contexto = x);
}

#[must_use]
pub fn contexto() -> Contexto {
    com(|c| c.contexto.clone())
}

pub fn publish_imagem(i: Option<Imagem>) {
    com(|c| c.imagem = i);
}

#[must_use]
pub fn imagem() -> Option<Imagem> {
    com(|c| c.imagem.clone())
}

/// O verbo de uma janela (o clique, ou uma cena).
pub fn pede(p: Pedido) {
    com(|c| c.pedido = Some(p));
}

/// Toma (e limpa) o verbo da janela.
pub fn take_pedido() -> Option<Pedido> {
    com(|c| c.pedido.take())
}

/// As opções da folha aberta (a janela, a cada quadro).
pub fn pede_folha(o: Option<OpcoesFolha>) {
    com(|c| c.folha_pedida = o);
}

/// As opções da folha aberta, a cada quadro; `None` = a janela da folha não está aberta.
#[must_use]
pub fn folha_pedida() -> Option<OpcoesFolha> {
    com(|c| c.folha_pedida.clone())
}

/// O que as janelas leem do sprite: as camadas de cima para baixo, como na linha do tempo.
pub fn publish_contexto_de(s: &Sprite) {
    publish_contexto(Contexto {
        lados: (s.width(), s.height()),
        quadros: u32::try_from(s.frames().len()).unwrap_or(u32::MAX),
        camadas: s
            .layers()
            .iter()
            .rev()
            .map(|l| (l.id, l.name.clone()))
            .collect(),
        tags: s.tags().iter().map(|t| t.name.clone()).collect(),
    });
}

/// O maior lado de uma imagem de cartão: a caixa mede menos de 400 px, e acima disto a folha só
/// gastaria memória da placa. A folha que sai no ficheiro não passa por aqui.
const LADO_DA_PREVIA: u32 = 2048;

type ChaveDaFolha = (u32, u64, OpcoesFolha);

/// **A folha ao vivo, fora da linha da tela.** Medido (M10, `diag_quanto_custa_a_folha_grande`): uma
/// folha 1024² × 256 monta em ~2 s, muito acima dos 100 ms do plano. Monta-se só quando as opções ou o
/// sprite mudam; mudar só as opções reaproveita a foto do sprite.
#[derive(Default)]
pub struct PreviaDaFolha {
    feita: Option<ChaveDaFolha>,
    /// A foto do sprite de `(doc, stamp)`.
    foto: Option<(u32, u64, Arc<Sprite>)>,
    trabalho: Option<(ChaveDaFolha, Job<Imagem>)>,
    seq: u64,
}

impl PreviaDaFolha {
    /// O próximo número de imagem (o Import usa a mesma contagem).
    pub fn proximo_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Esquece a última: a próxima pergunta remonta, e a foto do sprite sai da memória.
    pub fn esquece(&mut self) {
        self.feita = None;
        self.foto = None;
    }

    /// Há uma folha a montar?
    #[must_use]
    pub fn montando(&self) -> bool {
        self.trabalho.is_some()
    }

    /// Colhe a folha pronta e começa a pedida pela janela, se mudou.
    pub fn publica(&mut self, doc: u32, stamp: u64, s: &Sprite) {
        let pronto = self.trabalho.as_ref().is_some_and(|(_, j)| j.is_finished());
        if pronto && let Some((chave, mut j)) = self.trabalho.take() {
            // A janela já mudou para o Import? A imagem dele fica.
            if let Some(mut i) = j.try_take().filter(|_| folha_pedida().is_some()) {
                i.seq = self.proximo_seq();
                publish_imagem(Some(i));
            }
            self.feita = Some(chave);
        }
        let Some(o) = folha_pedida() else {
            return;
        };
        let chave = (doc, stamp, o);
        if self.trabalho.is_some() || self.feita.as_ref() == Some(&chave) {
            return;
        }
        let foto = match &self.foto {
            Some((d, t, f)) if (*d, *t) == (doc, stamp) => f.clone(),
            _ => Arc::new(s.clone()),
        };
        self.foto = Some((doc, stamp, foto.clone()));
        let opcoes = chave.2.clone();
        let job = Job::spawn(String::new(), move |_| monta_a_previa(&foto, &opcoes));
        self.trabalho = Some((chave, job));
    }
}

fn monta_a_previa(s: &Sprite, o: &OpcoesFolha) -> Imagem {
    let f = ph2d_pixel_io::sheet::montar(s, o, "", "png");
    let lado = f.w.max(f.h);
    let (w, h, rgba) = if lado > LADO_DA_PREVIA {
        let e = f64::from(LADO_DA_PREVIA) / f64::from(lado);
        ph2d_pixel_io::render::escalado(&f.rgba, f.w, f.h, e)
    } else {
        (f.w, f.h, f.rgba)
    };
    Imagem {
        rgba: Arc::new(rgba),
        w,
        h,
        seq: 0,
        lados_reais: (f.w, f.h),
        entradas: f.entradas.len(),
        sugestao: None,
    }
}

/// Um export a gravar: o documento, o pedido, o caminho e o trabalho.
struct Gravacao {
    doc: u32,
    export: Export,
    caminho: PathBuf,
    job: Job<Result<Gravado, ErroGravar>>,
}

/// **Os exports em curso.** Gravar corre fora da linha da tela, com a barra da casa.
#[derive(Default)]
pub struct Gravacoes {
    em_curso: Vec<Gravacao>,
    barras: Vec<Progress>,
}

/// Um export acabado; `None` = o trabalho caiu.
pub type Acabado = (u32, Export, PathBuf, Option<Result<Gravado, ErroGravar>>);

impl Gravacoes {
    /// Começa a gravar uma foto do sprite: o artista pode continuar a desenhar.
    pub fn comeca(&mut self, doc: u32, s: Sprite, export: Export, caminho: PathBuf) {
        let (e, c) = (export.clone(), caminho.clone());
        let nome = caminho.file_name().unwrap_or_default().to_string_lossy();
        let nome = ph2d_i18n::tr_with("pixel.io.exporting", &[("name", &nome)]);
        let job = Job::spawn(nome, move |p| {
            exporta::gravar_com(&s, &e, &c, &|f| p.set(f))
        });
        self.barras.push(job.progress().clone());
        self.em_curso.push(Gravacao {
            doc,
            export,
            caminho,
            job,
        });
    }

    #[must_use]
    pub fn vazia(&self) -> bool {
        self.em_curso.is_empty()
    }

    /// As barras novas, para a fila da casa.
    pub fn drena_barras(&mut self, q: &mut JobQueue) {
        for b in self.barras.drain(..) {
            q.push(b);
        }
    }

    /// **Repeat Last Export:** começa o último export do sprite. Devolve a recusa, se houver.
    pub fn repete(&mut self, doc: u32, s: &Sprite) -> Option<String> {
        match exporta::o_ultimo(s)? {
            (Some(e), c) => {
                self.comeca(doc, s.clone(), e, c);
                None
            }
            (None, _) => Some(crate::textos::erro_gravar(&ErroGravar::Preset)),
        }
    }

    /// Colhe os acabados: o que correu bem fica como o último daquele sprite. Devolve as frases
    /// `(é erro, texto)`.
    pub fn colhe_em(
        &mut self,
        mut lembra: impl FnMut(u32, &Export, &std::path::Path),
    ) -> Vec<(bool, String)> {
        let mut frases = Vec::new();
        for (d, e, c, r) in self.colhe() {
            let Some(r) = r else {
                frases.push((true, ph2d_i18n::tr("pixel.io.err.stopped").to_owned()));
                continue;
            };
            if r.is_ok() {
                lembra(d, &e, &c);
            }
            frases.extend(crate::textos::do_gravado(r));
        }
        frases
    }

    /// Os que acabaram desde o último quadro.
    pub fn colhe(&mut self) -> Vec<Acabado> {
        let (prontos, resto): (Vec<Gravacao>, Vec<Gravacao>) = std::mem::take(&mut self.em_curso)
            .into_iter()
            .partition(|g| g.job.is_finished());
        self.em_curso = resto;
        prontos
            .into_iter()
            .map(|mut g| {
                let r = g.job.try_take();
                (g.doc, g.export, g.caminho, r)
            })
            .collect()
    }
}

/// A imagem do Import: a 1.ª do documento, com a grelha que ela mostra.
#[must_use]
pub fn imagem_de_import(s: &Sprite, seq: u64) -> Imagem {
    let rgba = ph2d_pixel_io::frame_rgba(s, s.frames()[0].id);
    let (w, h) = (s.width(), s.height());
    Imagem {
        sugestao: ph2d_pixel_io::abrir::sugerir_grelha(&rgba, w, h),
        rgba: Arc::new(rgba),
        w,
        h,
        seq,
        lados_reais: (w, h),
        entradas: 0,
    }
}
