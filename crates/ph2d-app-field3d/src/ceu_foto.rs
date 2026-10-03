//! ⭐⭐⭐ **O CÉU FOTOGRÁFICO do Render por malha** — a escolha do artista (que céu, girado quanto, com
//! que força, com a luz-chave a que peso, e se aparece atrás da peça) e os atlas já montados.
//!
//! ⚠️ **É VISTA, como o brilho e o estilo** ([`crate::smoke::view::View`]): uma preferência de bancada que
//! ninguém pode reconstruir. ⛔ Não entra no documento (`PROJECT_SCHEMA` intacto).
//!
//! ⭐ **O «Studio» de fábrica é o céu de SEMPRE** (a rampa + a caixa do [`crate::studio`]), ao bit: só
//! um céu fotográfico escolhido muda o quadro. Os oito fotográficos são os do [`ph2d_sky::Embarcado`].

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, OnceLock};

use ph2d_sky::Embarcado;

/// Quantos números a escolha tem — ver [`Ceu::pack`].
pub const PACKED: usize = 6;

/// ⭐ **A escolha do céu** — `qual = 0` é o estúdio de sempre; `1..=8` o
/// [`Embarcado::TODOS`]`[qual − 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ceu {
    pub qual: u8,
    /// O giro em torno do eixo vertical, em graus.
    pub giro: f32,
    /// A força em stops sobre a luz do estúdio (`0` = a mesma luz média que o estúdio).
    pub forca: f32,
    /// O peso da luz-chave sob o céu fotográfico: o SOL dele (ou a lâmpada mais forte), a luz que faz
    /// sombra — `1` = o céu como foi fotografado.
    pub caixa: f32,
    /// O céu aparece atrás da peça?
    pub fundo: bool,
    /// O desfoque do céu atrás da peça, `0..1` (é o `√α` do lóbulo).
    pub desfoque: f32,
}

impl Default for Ceu {
    fn default() -> Self {
        Self {
            qual: 0,
            giro: 0.0,
            forca: 0.0,
            caixa: 1.0,
            fundo: true,
            // ⚠️ **Desfocado de fábrica** (02/10, a foto da cena 38): um céu `1K` atrás da peça a
            // 1080p é ampliado `~6×` e uma borda de alto contraste da foto fica em degraus; o
            // Blender abre a pré-visualização com o fundo desfocado pela mesma razão. `0` = nítido.
            // Fotografado: `0` degraus · `0,3` já apaga a foto (um lóbulo de `~7°`) · `0,15` lê-se.
            desfoque: 0.15,
        }
    }
}

/// O tecto do giro (graus).
pub const GIRO_MAX: f32 = 360.0;
/// ⚠️ A força em STOPS e simétrica: `±4` são `16×` para cada lado — a faixa do olhar da casa
/// (`ph2d_view_transform::Look`), e quem precisar de mais tem a exposição.
pub const FORCA_MAX: f32 = 4.0;
/// O tecto do peso da luz-chave: o dobro do sol fotografado.
pub const CAIXA_MAX: f32 = 2.0;

impl Ceu {
    /// O céu embarcado escolhido, ou `None` = o estúdio de sempre.
    #[must_use]
    pub fn embarcado(&self) -> Option<Embarcado> {
        (self.qual as usize)
            .checked_sub(1)
            .and_then(|i| Embarcado::TODOS.get(i).copied())
    }

    /// Os números na ordem das fileiras do painel.
    #[must_use]
    pub fn pack(&self) -> [f32; PACKED] {
        [
            f32::from(self.qual),
            self.giro,
            self.forca,
            self.caixa,
            f32::from(u8::from(self.fundo)),
            self.desfoque,
        ]
    }

    /// O inverso do [`Ceu::pack`], já SANEADO.
    #[must_use]
    pub fn unpack(v: &[f32; PACKED]) -> Self {
        Self {
            qual: v[0].round().clamp(0.0, Embarcado::TODOS.len() as f32) as u8,
            giro: v[1],
            forca: v[2],
            caixa: v[3],
            fundo: v[4] >= 0.5,
            desfoque: v[5],
        }
        .sanitized()
    }

    /// ⚠️ **A cerca da porta**: a partir daqui os números viajam para o desenhista, que não tem cerca.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let fin = |x: f32, lo: f32, hi: f32, d: f32| if x.is_finite() { x.clamp(lo, hi) } else { d };
        Self {
            qual: self.qual.min(Embarcado::TODOS.len() as u8),
            giro: fin(self.giro, 0.0, GIRO_MAX, 0.0),
            forca: fin(self.forca, -FORCA_MAX, FORCA_MAX, 0.0),
            caixa: fin(self.caixa, 0.0, CAIXA_MAX, 1.0),
            fundo: self.fundo,
            desfoque: fin(self.desfoque, 0.0, 1.0, 0.0),
        }
    }

    /// ⭐⭐ **O que o desenhista recebe** — com a força NORMALIZADA: `0` stops dá a este céu (com o
    /// sol) a mesma radiância média que o estúdio inteiro tem (trocar de céu não estoura nem apaga a
    /// cena; quem quer mais escuro ou mais claro tem a fileira).
    #[must_use]
    pub fn foto(&self, ceu: &ph2d_sky::Ceu) -> ph2d_mesh_forward::Foto {
        let rad = self.giro.to_radians();
        ph2d_mesh_forward::Foto {
            giro: [rad.cos(), rad.sin()],
            forca: self.forca.exp2() * normalizacao(ceu),
            caixa: self.caixa,
            fundo: self.fundo.then_some(self.desfoque * self.desfoque),
        }
    }
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// ⭐ **O fator que põe um céu à luz do estúdio** — a radiância média do estúdio INTEIRO (a rampa tem
/// média `AMBIENT · ENV_BASE` sobre a esfera, porque o declive em `y` se anula; a caixa só tira a
/// fracção `share` da base e a devolve) sobre a média do panorama inteiro (com o sol): a foto
/// substitui as DUAS partes do céu do estúdio.
#[must_use]
pub fn normalizacao(ceu: &ph2d_sky::Ceu) -> f32 {
    let estudio = ph2d_light::AMBIENT * luma(ph2d_light::ENV_BASE);
    estudio / luma(ceu.media()).max(1.0e-9)
}

#[derive(Default)]
struct Cache {
    prontos: BTreeMap<Embarcado, Arc<ph2d_sky::Ceu>>,
    a_caminho: BTreeSet<Embarcado>,
}

fn cache() -> &'static Mutex<Cache> {
    static C: OnceLock<Mutex<Cache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(Cache::default()))
}

/// ⭐⭐ **O atlas do céu `e`, com o SOL à parte, se já estiver montado** — da 1.ª vez lança a montagem
/// NOUTRA thread (`~0,7 s` em release: é trabalho de CPU) e devolve `None`; o quadro espera, como a
/// curvatura.
#[must_use]
pub fn pronto(e: Embarcado) -> Option<Arc<ph2d_sky::Ceu>> {
    let mut c = cache().lock().ok()?;
    if let Some(a) = c.prontos.get(&e) {
        return Some(Arc::clone(a));
    }
    if c.a_caminho.insert(e) {
        let lancou = std::thread::Builder::new()
            .name("ph2d-ceu-foto".to_owned())
            .spawn(move || {
                let a = Arc::new(ph2d_sky::Ceu::com_sol(&e.panorama()));
                if let Ok(mut c) = cache().lock() {
                    c.a_caminho.remove(&e);
                    c.prontos.insert(e, a);
                }
            });
        if lancou.is_err() {
            c.a_caminho.remove(&e);
        }
    }
    None
}

/// ⭐ **O céu `e` não tem sol?** — `true` só quando o atlas já está montado e a lei não achou sol
/// ([`ph2d_sky::sol::FRACAO_MIN`]: um céu nublado). Não monta nada: quem pinta o painel pergunta aqui.
#[must_use]
pub fn sem_sol(e: Embarcado) -> bool {
    cache()
        .lock()
        .ok()
        .and_then(|c| c.prontos.get(&e).map(|a| a.sol().is_none()))
        .unwrap_or(false)
}

/// ⚙️ A escolha que o `PH2D_FIELD_SKY=<nome>` pede ao abrir (para FOTOGRAFAR, como o
/// `PH2D_FIELD_BLOOM=1`): o nome é o id do [`Embarcado::chave`]; o céu aparece atrás.
#[must_use]
pub fn do_ambiente(v: Ceu) -> Ceu {
    do_ambiente_em(v, std::env::var("PH2D_FIELD_SKY").ok().as_deref())
}

/// O mesmo, com o nome DADO (a sonda lê outra chave).
#[must_use]
pub fn do_ambiente_em(v: Ceu, nome: Option<&str>) -> Ceu {
    let Some(nome) = nome else {
        return v;
    };
    Embarcado::TODOS
        .iter()
        .position(|e| e.chave() == nome)
        .map_or(v, |i| Ceu {
            qual: (i + 1) as u8,
            ..v
        })
}

/// ⚠️ **Aqui e não no `smoke_state.rs`**: a porta é da família do céu, e aquele ficheiro está no
/// tecto de LOC — a cura é mover, nunca subir o número.
impl crate::smoke::Smoke {
    /// ⭐⭐⭐ **Troca o CÉU fotográfico** — a irmã do `Smoke::set_bloom`; saneia na porta e larga o
    /// pedido guardado de todos os viewports.
    pub fn set_ceu(&mut self, ceu: Ceu) {
        let ceu = ceu.sanitized();
        if self.ceu == ceu {
            return;
        }
        self.ceu = ceu;
        self.forget_requests();
    }
}

#[cfg(test)]
#[path = "ceu_foto_tests.rs"]
mod tests;
