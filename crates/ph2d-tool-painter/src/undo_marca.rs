//! **A marca do artista na história do Painter** (W9 `docs/Macros/07_plano_W9.md`, W10
//! `docs/Macros/08_plano_W10_cobertura.md`) — filho de [`super`]. Cada passo REGISTADO compõe na marca a
//! tinta que o artista pôs nele:
//!
//! - **com a força medida** (o carimbo escreveu-a no plano — Digital, lei «por cima», pilha trivial): cada
//!   texel recebe a força com que foi pintado, e o que estava por baixo no 1.º toque;
//! - **pela água do artista** (W12, Wet Paint: a sessão molhada nasce com a marca, a tinta dele por cima
//!   da imagem): cada composite da água recompõe na marca o `pa` que a tinta tem AGORA ([`Molhada`]);
//! - **sem ela** (a água da gravação, o relevo, os outros modos e meios): a lei da W9 — os texels que o passo mudou,
//!   salvo os que só a água mudou; os que o pincel cobriu e mudaram; os que cobriu DENTRO da água; e o
//!   miolo do pincel (força ≥ ½, [`crate::macro_track::forca`]) — ficam com `c = 255`.
//!
//! A água que corre sozinha não é passo; a reescrita da macro e o «Recomeçar» suspendem a marca. O desfazer
//! devolve os ladrilhos de antes do passo.

use super::{ModelSnapshot, UndoController, UndoEntry};
use crate::compositor::Region;
use crate::layers::LayerId as RtLayerId;
use crate::macro_track::forca::ForcaDoPincel;
use crate::macro_track::marca::{self, DeltaDaMarca, MarcaDoArtista};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Os ladrilhos de uma marca (o retrato do desfazer).
type Retrato = Arc<BTreeMap<(u32, u32), Arc<marca::Ladrilho>>>;

/// A marca da imagem que o Painter tem, e o passo em curso.
#[derive(Clone, Debug, Default)]
pub(crate) struct MarcaViva {
    pub(crate) marca: Option<MarcaDoArtista>,
    /// A marca antes da 1.ª escrita provisória do passo (`None` = o passo ainda não marcou).
    antes_do_passo: Option<Retrato>,
    /// Os texels que o pincel cobriu neste passo.
    discos: Vec<u64>,
    /// Os que ele cobriu DENTRO da água (o Wet Paint): a tinta aparece pela água, às vezes depois de o
    /// passo se registar — marcam sem esperar que mudem.
    na_agua: Vec<u64>,
    /// Os texels que a água mudou desde o último registo.
    agua: Vec<u64>,
    /// A força com que o pincel cobriu cada texel neste passo, pelo perfil (a lei da W9: o miolo).
    pub(crate) forca: ForcaDoPincel,
    /// **A força MEDIDA**: o plano (do tamanho da tela) que o carimbo escreve, e a caixa do que escreveu.
    plano: Vec<u16>,
    plano_caixa: Option<marca::Caixa>,
    /// O lote em curso pode pintar pelo plano (`stamp_dabs` arma-o pela lei).
    armada: bool,
    /// Uma rota do carimbo tomou o plano neste lote.
    tomada: bool,
    /// `Some(true)` = toda a tinta do passo veio com a força medida; `Some(false)` = alguma não.
    exata: Option<bool>,
    /// O papel sob a camada (a imagem é a camada sobre ele): o que estava por baixo é a imagem.
    papel: Option<[u8; 3]>,
    /// O passo que se regista agora é da macro (a reescrita de uma etapa, o «Recomeçar»).
    pub(crate) suspensa: bool,
    /// A marca antes de a reescrita a mudar por conta própria (o que estava por baixo, W10).
    antes_da_reescrita: Option<Retrato>,
    /// A sessão molhada viva é do artista (W12): a cobertura dela recompõe-se a cada composite.
    molhada: Option<Molhada>,
}

/// **A sessão molhada do artista** (W12, `docs/Macros/10_W12_cruzamentos_em_wet_paint.md`): nasceu com a
/// marca posta, sobre a imagem de então, e a tinta dela é composta «por cima» dessa imagem
/// (`atual = P + (1 − pa)·base`) — a cobertura do texel é a marca de então composta com o `pa` de agora.
#[derive(Clone, Debug)]
struct Molhada {
    /// A sessão (o `marca` dela diz qual).
    id: u64,
    /// A marca quando a sessão nasceu (e os passos de fora da água desde então).
    nascimento: Retrato,
    /// A marca no último registo (ou absorção) — o «depois» que o histórico conhece.
    registo: Retrato,
    /// A marca quando o passo em curso começou.
    antes_do_passo: Option<Retrato>,
    /// O papel sob a camada: o que está por baixo é a tela da sessão sobre ele.
    papel: Option<[u8; 3]>,
}

/// As sessões molhadas que nascem com a marca: nunca duas com o mesmo id.
static SESSOES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Zera `v` com `n` palavras.
fn zera(v: &mut Vec<u64>, n: usize) {
    v.clear();
    v.resize(n, 0);
}

impl MarcaViva {
    pub(crate) fn poe(&mut self, m: Option<MarcaDoArtista>) {
        self.marca = m;
        self.antes_do_passo = None;
        self.antes_da_reescrita = None;
        self.discos.clear();
        self.na_agua.clear();
        self.agua.clear();
        self.forca.limpa();
        self.plano = Vec::new();
        self.plano_caixa = None;
        self.exata = None;
        self.molhada = None;
    }

    /// **Uma sessão molhada nasce** sobre a imagem de agora: se há uma marca a escrever, a sessão é do
    /// artista e devolve o id que ela guarda ([`Self::molhada_viva`]).
    pub(crate) fn nasce_a_molhada(&mut self, papel: Option<[u8; 3]>) -> Option<u64> {
        if !self.ativa() || self.suspensa {
            self.molhada = None;
            return None;
        }
        let agora = Arc::clone(self.marca.as_ref()?.ladrilhos());
        let id = SESSOES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.molhada = Some(Molhada {
            id,
            nascimento: Arc::clone(&agora),
            registo: agora,
            antes_do_passo: None,
            papel,
        });
        Some(id)
    }

    /// A sessão molhada acabou (assou).
    pub(crate) fn acaba_a_molhada(&mut self) {
        self.molhada = None;
    }

    /// A sessão `id` é a do artista, viva, e a marca escreve-se.
    pub(crate) fn molhada_viva(&self, id: Option<u64>) -> bool {
        self.ativa() && !self.suspensa && id.is_some() && self.molhada.as_ref().map(|m| m.id) == id
    }

    /// **O composite da sessão `id` escreveu** a caixa `(x0, y0, x1, y1)` da tela (largura `w`): `plano` é
    /// a cobertura de cada texel dela (`0..=CHEIA`), `base` a tela congelada da sessão.
    pub(crate) fn agua_cobre(
        &mut self,
        id: Option<u64>,
        caixa: (u32, u32, u32, u32),
        plano: &[u16],
        base: &[u8],
    ) {
        if !self.molhada_viva(id) {
            return;
        }
        let (Some(m), Some(mo)) = (self.marca.as_mut(), self.molhada.as_ref()) else {
            return;
        };
        let papel = mo.papel;
        m.recompoe_da_agua(&mo.nascimento, caixa, plano, |i| {
            let mut px = base
                .get(i * 4..i * 4 + 4)
                .map_or([0; 4], |p| [p[0], p[1], p[2], p[3]]);
            if let Some(p) = papel {
                crate::tool::papel::sobre_o_papel(&mut px, p);
            }
            px
        });
    }

    /// **A absorção da água no passo anterior** (`undo_absorb`): o delta `d` dele estica-se até onde o
    /// passo em curso começou — a tinta que a água espalhou depois do registo é desse passo.
    pub(crate) fn estende_pela_agua(&mut self, d: Option<DeltaDaMarca>) -> Option<DeltaDaMarca> {
        let (Some(m), Some(mo)) = (self.marca.as_ref(), self.molhada.as_mut()) else {
            return d;
        };
        let fim = mo
            .antes_do_passo
            .clone()
            .unwrap_or_else(|| Arc::clone(m.ladrilhos()));
        let mais = DeltaDaMarca::entre(m.epoca(), &mo.registo, &fim);
        mo.registo = fim;
        if mais.vazio() {
            return d;
        }
        let mut d = d.unwrap_or_else(|| DeltaDaMarca {
            epoca: m.epoca(),
            ladrilhos: Vec::new(),
        });
        d.junta(&mais);
        Some(d)
    }

    /// Há uma marca a escrever (a imagem tem uma macro).
    pub(crate) fn ativa(&self) -> bool {
        self.marca.is_some()
    }

    fn palavras(&self) -> usize {
        self.marca
            .as_ref()
            .map_or(0, |m| marca::palavras_para(m.tamanho().0, m.tamanho().1))
    }

    fn guarda_o_antes(&mut self) {
        if self.antes_do_passo.is_none()
            && let Some(m) = &self.marca
        {
            self.antes_do_passo = Some(Arc::clone(m.ladrilhos()));
        }
    }

    /// O pincel cobre o disco `(cx, cy, r)` (`na_agua` = o dab entrou na água): mostra-se já (`c = 255`
    /// até o registo do passo decidir).
    pub(crate) fn disco(&mut self, cx: f32, cy: f32, r: f32, na_agua: bool) {
        let n = self.palavras();
        if n == 0 {
            return;
        }
        self.guarda_o_antes();
        let bits = if na_agua {
            &mut self.na_agua
        } else {
            &mut self.discos
        };
        if bits.len() != n {
            zera(bits, n);
        }
        let Some(m) = self.marca.as_mut() else {
            return;
        };
        let (w, h) = m.tamanho();
        marca::liga_disco(bits, w, h, cx, cy, r);
        m.marca_disco(cx, cy, r);
    }

    /// **O lote seguinte** pinta pela lei «por cima» (`armar`): uma rota do carimbo pode tomar o plano.
    pub(crate) fn arma_o_lote(&mut self, armar: bool, papel: Option<[u8; 3]>) {
        self.armada = armar && self.ativa();
        self.tomada = false;
        if self.armada {
            self.papel = papel;
        }
    }

    /// O plano da força para uma rota do carimbo numa tela `w`×`h` (`None` = o lote não está armado).
    pub(crate) fn toma_o_plano(&mut self, (w, h): (u32, u32)) -> Option<Vec<u16>> {
        if !self.armada || self.marca.as_ref().map(MarcaDoArtista::tamanho) != Some((w, h)) {
            return None;
        }
        self.tomada = true;
        let n = w as usize * h as usize;
        let mut p = std::mem::take(&mut self.plano);
        if p.len() != n {
            p.clear();
            p.resize(n, 0);
        }
        Some(p)
    }

    /// A rota devolve o plano e o que escreveu nele.
    pub(crate) fn devolve_o_plano(&mut self, p: Option<Vec<u16>>, tocou: Option<Region>) {
        if let Some(p) = p {
            self.plano = p;
            if let Some(r) = tocou {
                self.plano_caixa = Some(marca::une(self.plano_caixa, (r.x, r.y, r.w, r.h)));
            }
        }
    }

    /// O lote acabou: `depositou` = havia tinta a pôr. Sem uma rota que medisse a força, o passo deixa
    /// de ser exacto.
    pub(crate) fn fecha_o_lote(&mut self, depositou: bool) {
        if depositou && self.ativa() {
            let medido = self.armada && self.tomada;
            self.exata = Some(self.exata.unwrap_or(true) && medido);
        }
        self.armada = false;
        self.tomada = false;
    }

    /// A água escreveu a caixa `c` = `(x0, y0, x1, y1)` de `canvas` (largura `w`); `antes` é a caixa
    /// como estava, linha a linha.
    pub(crate) fn agua_mudou(
        &mut self,
        antes: &[u8],
        canvas: &[u8],
        w: usize,
        c: (usize, usize, usize, usize),
    ) {
        let n = self.palavras();
        if n == 0 {
            return;
        }
        if self.agua.len() != n {
            zera(&mut self.agua, n);
        }
        let (x0, y0, x1, y1) = c;
        let lw = (x1 - x0) * 4;
        for (j, linha) in antes.chunks_exact(lw).enumerate().take(y1 - y0) {
            let y = y0 + j;
            let a = (y * w + x0) * 4;
            let Some(agora) = canvas.get(a..a + lw) else {
                return;
            };
            for (k, (p, q)) in linha
                .as_chunks::<4>()
                .0
                .iter()
                .zip(agora.as_chunks::<4>().0)
                .enumerate()
            {
                if p != q {
                    marca::liga(&mut self.agua, y * w + x0 + k);
                }
            }
        }
    }

    /// A água recebe uma COBERTURA (a mancha do Solid, os fios): `cov` (`0..=255`) na caixa `(x, y, w, h)`
    /// — os texels cobertos marcam como os dabs que entram na água.
    pub(crate) fn cobertura_na_agua(&mut self, (x, y, w, h): (u32, u32, u32, u32), cov: &[u8]) {
        let n = self.palavras();
        if n == 0 {
            return;
        }
        self.guarda_o_antes();
        if self.na_agua.len() != n {
            zera(&mut self.na_agua, n);
        }
        let Some(m) = self.marca.as_mut() else {
            return;
        };
        let (mw, mh) = m.tamanho();
        let mut bits = vec![0u64; n];
        for j in 0..h.min(mh.saturating_sub(y)) {
            for i in 0..w.min(mw.saturating_sub(x)) {
                if cov.get((j * w + i) as usize).is_some_and(|&c| c > 0) {
                    let k = ((y + j) * mw + x + i) as usize;
                    marca::liga(&mut self.na_agua, k);
                    marca::liga(&mut bits, k);
                }
            }
        }
        m.satura_bits(&bits);
    }

    /// A pré-visualização carimba-se de novo inteira (Line, Drag Dot, as formas): os discos e a força
    /// da anterior deixam de valer.
    pub(crate) fn recomeca_os_discos(&mut self) {
        self.tira_o_provisorio();
        self.discos.fill(0);
        self.forca.limpa();
        self.limpa_o_plano();
    }

    /// Um passo começa: a água de antes dele já não conta.
    pub(crate) fn comeca_o_passo(&mut self) {
        self.agua.fill(0);
        if let (Some(m), Some(mo)) = (self.marca.as_ref(), self.molhada.as_mut()) {
            mo.antes_do_passo = Some(Arc::clone(m.ladrilhos()));
        }
    }

    fn limpa_o_plano(&mut self) {
        let w = self.marca.as_ref().map_or(0, |m| m.tamanho().0) as usize;
        if let (Some((x, y, cw, ch)), true) = (self.plano_caixa.take(), w > 0) {
            for j in y as usize..(y + ch) as usize {
                if let Some(linha) = self
                    .plano
                    .get_mut(j * w + x as usize..j * w + (x + cw) as usize)
                {
                    linha.fill(0);
                }
            }
        }
        self.exata = None;
    }

    fn limpa_o_passo(&mut self) {
        self.antes_do_passo = None;
        self.discos.fill(0);
        self.na_agua.fill(0);
        self.agua.fill(0);
        self.forca.limpa();
        self.limpa_o_plano();
    }

    /// Devolve a marca ao que era antes das escritas provisórias do passo em curso.
    fn tira_o_provisorio(&mut self) {
        if let (Some(m), Some(pre)) = (self.marca.as_mut(), self.antes_do_passo.take()) {
            m.poe_ladrilhos(pre);
        }
    }

    /// A reescrita da macro vai mudar o que está por baixo de texels do artista: o desfazer dela devolve-o.
    pub(crate) fn comeca_a_reescrita(&mut self) {
        self.antes_da_reescrita = self.marca.as_ref().map(|m| Arc::clone(m.ladrilhos()));
    }

    /// **O passo regista-se**: compõe na marca a tinta do artista e devolve o que mudou.
    fn fecha_o_passo(
        &mut self,
        before: &ModelSnapshot,
        after: &ModelSnapshot,
        hint: Option<Region>,
    ) -> Option<DeltaDaMarca> {
        self.tira_o_provisorio();
        let tamanho = self.marca.as_ref().map(MarcaDoArtista::tamanho);
        if let Some(antes) = self.antes_da_reescrita.take() {
            self.limpa_o_passo();
            let m = self.marca.as_ref()?;
            let d = DeltaDaMarca::entre(m.epoca(), &antes, m.ladrilhos());
            return (!d.vazio()).then_some(d);
        }
        if self.suspensa
            || tamanho.is_none()
            || before.canvas_size != after.canvas_size
            || tamanho != Some(after.canvas_size)
        {
            self.limpa_o_passo();
            return None;
        }
        let n = self.palavras();
        let mut s = vec![0u64; n];
        texels_mudados(before, after, hint, &mut s);
        let mut miolo = vec![0u64; n];
        self.forca.liga_o_miolo(&mut miolo, after.canvas_size);
        for (b, m) in s.iter_mut().zip(&miolo) {
            *b |= m;
        }
        for (k, b) in s.iter_mut().enumerate() {
            let agua = self.agua.get(k).copied().unwrap_or(0);
            let disco = self.discos.get(k).copied().unwrap_or(0);
            let na_agua = self.na_agua.get(k).copied().unwrap_or(0);
            *b = (*b & (!agua | disco)) | na_agua;
        }
        let (w, h) = (after.canvas_size.0 as usize, after.canvas_size.1 as usize);
        let exata = self.exata == Some(true) && self.plano.len() == w * h;
        let papel = self.papel;
        let por_baixo = |i: usize| -> [u8; 4] {
            let mut px = before
                .canvas_rgba
                .get(i * 4..i * 4 + 4)
                .map_or([0; 4], |p| [p[0], p[1], p[2], p[3]]);
            if let Some(p) = papel {
                crate::tool::papel::sobre_o_papel(&mut px, p);
            }
            px
        };
        let forca = |i: usize| -> Option<f32> {
            self.plano
                .get(i)
                .filter(|&&p| exata && p > 0)
                .map(|&p| f32::from(p) / 65535.0)
        };
        // Os texels da lei sem força (`c = 255` onde o plano não diz) e os que só o plano tem.
        let mut itens: Vec<(usize, f32, [u8; 4])> = Vec::new();
        for (k, &p) in s.iter().enumerate() {
            let mut p = p;
            while p != 0 {
                let i = k * 64 + p.trailing_zeros() as usize;
                p &= p - 1;
                itens.push((i, forca(i).unwrap_or(1.0), por_baixo(i)));
            }
        }
        if let (true, Some((x, y, cw, ch))) = (exata, self.plano_caixa) {
            for j in y as usize..((y + ch) as usize).min(h) {
                for i in j * w + x as usize..j * w + ((x + cw) as usize).min(w) {
                    if let (Some(a), false) = (forca(i), marca::ligado(&s, i)) {
                        itens.push((i, a, por_baixo(i)));
                    }
                }
            }
        }
        let antes = Arc::clone(self.marca.as_ref()?.ladrilhos());
        let m = self.marca.as_mut()?;
        if let Some(mo) = self.molhada.as_mut() {
            // A água do artista: o que o passo mudou FORA dela compõe-se também no nascimento (o composite
            // seguinte recompõe a partir dele), e o passo vai de onde começou até aqui.
            let mut n = MarcaDoArtista::nova(w as u32, h as u32, m.epoca());
            n.poe_ladrilhos(Arc::clone(&mo.nascimento));
            n.compoe_muitos(itens.iter().copied());
            mo.nascimento = Arc::clone(n.ladrilhos());
            m.compoe_muitos(itens);
            let de = mo
                .antes_do_passo
                .take()
                .unwrap_or_else(|| Arc::clone(&mo.registo));
            let d = DeltaDaMarca::entre(m.epoca(), &de, m.ladrilhos());
            mo.registo = Arc::clone(m.ladrilhos());
            self.limpa_o_passo();
            return (!d.vazio()).then_some(d);
        }
        m.compoe_muitos(itens);
        let d = DeltaDaMarca::entre(m.epoca(), &antes, m.ladrilhos());
        self.limpa_o_passo();
        (!d.vazio()).then_some(d)
    }
}

/// Os píxeis RGBA da camada `id` de `s` (a ativa vive na tela).
fn rgba(s: &ModelSnapshot, id: RtLayerId) -> Option<&[u8]> {
    if s.layers.active() == Some(id) {
        Some(s.canvas_rgba.as_slice())
    } else {
        s.images.get(&id).map(|i| i.rgba8.as_slice())
    }
}

/// Liga em `out` os texels onde os planos `a` e `b` (de `k` elementos por texel, `n` texels) diferem,
/// dentro das linhas `linhas` e das colunas `colunas` (de largura `w`). Um plano só de um lado: os
/// texels dele que não são o de fábrica.
#[expect(clippy::too_many_arguments, reason = "a janela e a forma do plano")]
fn compara<T: PartialEq + Default>(
    a: Option<&[T]>,
    b: Option<&[T]>,
    k: usize,
    w: usize,
    n: usize,
    linhas: std::ops::Range<usize>,
    colunas: std::ops::Range<usize>,
    out: &mut [u64],
) {
    let zero = T::default();
    let difere = |i: usize| -> bool {
        match (a, b) {
            (Some(a), Some(b)) => a[i * k..(i + 1) * k] != b[i * k..(i + 1) * k],
            (Some(x), None) | (None, Some(x)) => x[i * k..(i + 1) * k].iter().any(|v| *v != zero),
            (None, None) => false,
        }
    };
    match (a, b) {
        (Some(a), Some(b)) if std::ptr::eq(a.as_ptr(), b.as_ptr()) && a.len() == b.len() => return,
        (Some(a), Some(b)) if a.len() != n * k || b.len() != n * k => return,
        (Some(x), None) | (None, Some(x)) if x.len() != n * k => return,
        _ => {}
    }
    for y in linhas {
        for x in colunas.clone() {
            let i = y * w + x;
            if difere(i) {
                marca::liga(out, i);
            }
        }
    }
}

/// Um plano preguiçoso por camada (`None` = nunca escrito).
fn de<T>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>, id: RtLayerId) -> Option<&[T]> {
    m.get(&id).map(|v| v.as_slice())
}

/// Liga em `out` os texels onde algum plano de alguma camada difere de `b` para `a`; a tela da camada
/// ativa dos dois lados só dentro de `hint` (a janela que as escritas declararam desde o último registo).
fn texels_mudados(b: &ModelSnapshot, a: &ModelSnapshot, hint: Option<Region>, out: &mut [u64]) {
    let (w, h) = (a.canvas_size.0 as usize, a.canvas_size.1 as usize);
    let n = w * h;
    let mut ids: BTreeSet<RtLayerId> = b.images.keys().chain(a.images.keys()).copied().collect();
    ids.extend(b.layers.active());
    ids.extend(a.layers.active());
    let tudo = (0..h, 0..w);
    for &id in &ids {
        let mesma_tela = b.layers.active() == Some(id) && a.layers.active() == Some(id);
        let (linhas, colunas) = match hint.filter(|_| mesma_tela) {
            Some(r) => (
                r.y as usize..((r.y + r.h) as usize).min(h),
                r.x as usize..((r.x + r.w) as usize).min(w),
            ),
            None => tudo.clone(),
        };
        compara(rgba(b, id), rgba(a, id), 4, w, n, linhas, colunas, out);
    }
    // O relevo descrito sem ser segurado não testemunha (`elide`).
    if b.relief_elided.is_empty() && a.relief_elided.is_empty() {
        let ids: BTreeSet<RtLayerId> = [&b.heights, &a.heights]
            .into_iter()
            .flat_map(BTreeMap::keys)
            .chain(b.covers.keys().chain(a.covers.keys()))
            .chain(b.mats.keys().chain(a.mats.keys()))
            .copied()
            .collect();
        for id in ids {
            let (l, c) = tudo.clone();
            compara(de(&b.heights, id), de(&a.heights, id), 1, w, n, l, c, out);
            let (l, c) = tudo.clone();
            compara(de(&b.covers, id), de(&a.covers, id), 1, w, n, l, c, out);
            let (l, c) = tudo.clone();
            compara(de(&b.mats, id), de(&a.mats, id), 1, w, n, l, c, out);
        }
    }
}

impl UndoController {
    /// A marca que o passo que se regista agora liga (`None` = nenhuma).
    pub(super) fn marca_do_passo(
        &mut self,
        before: &ModelSnapshot,
        after: &ModelSnapshot,
        hint: Option<Region>,
    ) -> Option<DeltaDaMarca> {
        if !self.marca.ativa() {
            return None;
        }
        self.marca.fecha_o_passo(before, after, hint)
    }

    /// O desfazer (`liga == false`) ou o refazer de `entry` chega à marca.
    pub(super) fn marca_anda(&mut self, entry: &UndoEntry, liga: bool) {
        self.marca.tira_o_provisorio();
        if let (Some(m), Some(d)) = (self.marca.marca.as_mut(), entry.marca.as_ref()) {
            m.aplica(d, liga);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::tool::RasterEditTool;

    /// ⭐ **Um passo que a água atravessou marca só o que é do artista**: o passo muda duas linhas da tela;
    /// a água escreveu a 1.ª (fora de qualquer disco) e a 2.ª é do artista ⇒ só a 2.ª fica marcada; um
    /// disco do pincel sobre a 1.ª devolve-lhe o que ele cobre. (M1 da prova de 17/10 sobrevivia.)
    #[test]
    fn um_passo_marca_o_que_mudou_salvo_o_que_so_a_agua_mudou() {
        const L: u32 = 16;
        let mut t = crate::PainterTool::default();
        t.set_source(vec![255; (L * L * 4) as usize], L, L);
        for disco in [false, true] {
            let mut c = UndoController::default();
            c.marca.poe(Some(MarcaDoArtista::nova(L, L, 1)));
            let before = t.snapshot_model();
            let mut after = before.clone();
            let mut px = before.canvas_rgba.as_ref().clone();
            let linha = |y: u32| (y * L) as usize * 4..((y + 1) * L) as usize * 4;
            px[linha(0)].fill(10);
            px[linha(8)].fill(20);
            after.canvas_rgba = Arc::new(px.clone());
            // A água escreveu a linha 0 (a caixa como estava, e a tela depois).
            c.marca.agua_mudou(
                &before.canvas_rgba[linha(0)],
                &px,
                L as usize,
                (0, 0, L as usize, 1),
            );
            if disco {
                c.marca.disco(8.0, 0.5, 0.6, false);
            }
            c.record_structural_hinted(before, after, None);
            let m = c.marca.marca.as_ref().expect("a marca");
            let marcados: Vec<usize> = (0..(L * L) as usize).filter(|&i| m.marcado(i)).collect();
            let artista: Vec<usize> = ((8 * L) as usize..(9 * L) as usize).collect();
            if disco {
                assert!(marcados.contains(&8) && marcados.ends_with(&artista));
                assert_eq!(
                    marcados.len(),
                    artista.len() + 2,
                    "o disco devolve só o que cobre"
                );
            } else {
                assert_eq!(marcados, artista, "a água marcou");
            }
        }
    }
}
