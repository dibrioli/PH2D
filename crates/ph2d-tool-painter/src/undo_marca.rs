//! **A marca do artista na história do Painter** (W9, `docs/Macros/07_plano_W9.md`) — filho de
//! [`super`]. Cada passo REGISTADO marca o que o artista escreveu: os texels que o passo mudou, salvo os
//! que só a água mudou; os que o pincel cobriu e mudaram (mesmo que a água também os tenha mudado); e os
//! que o pincel cobriu DENTRO da água (o Wet Paint), mudados ou não. A água que corre sozinha não é
//! passo; a reescrita da macro e o «Recomeçar» suspendem a marca. O desfazer desliga o que o passo ligou.

use super::{ModelSnapshot, UndoController, UndoEntry};
use crate::compositor::Region;
use crate::layers::LayerId as RtLayerId;
use crate::macro_track::marca::{self, DeltaDaMarca, MarcaDoArtista};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// A marca da imagem que o Painter tem, e o passo em curso.
#[derive(Clone, Debug, Default)]
pub(crate) struct MarcaViva {
    pub(crate) marca: Option<MarcaDoArtista>,
    /// As palavras antes do 1.º disco do passo (`None` = o passo ainda não marcou).
    antes_do_passo: Option<Arc<Vec<u64>>>,
    /// Os texels que o pincel cobriu neste passo.
    discos: Vec<u64>,
    /// Os que ele cobriu DENTRO da água (o Wet Paint): a tinta aparece pela água, às vezes depois de o
    /// passo se registar — marcam sem esperar que mudem.
    na_agua: Vec<u64>,
    /// Os texels que a água mudou desde o último registo.
    agua: Vec<u64>,
    /// O passo que se regista agora é da macro (a reescrita de uma etapa, o «Recomeçar»).
    pub(crate) suspensa: bool,
}

/// Zera `v` com `n` palavras.
fn zera(v: &mut Vec<u64>, n: usize) {
    v.clear();
    v.resize(n, 0);
}

impl MarcaViva {
    pub(crate) fn poe(&mut self, m: Option<MarcaDoArtista>) {
        self.marca = m;
        self.antes_do_passo = None;
        self.discos.clear();
        self.na_agua.clear();
        self.agua.clear();
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

    /// O pincel cobre o disco `(cx, cy, r)` (`na_agua` = o dab entrou na água): mostra-se já; o registo
    /// do passo decide.
    pub(crate) fn disco(&mut self, cx: f32, cy: f32, r: f32, na_agua: bool) {
        let n = self.palavras();
        let Some(m) = self.marca.as_mut() else {
            return;
        };
        let bits = if na_agua {
            &mut self.na_agua
        } else {
            &mut self.discos
        };
        if bits.len() != n {
            zera(bits, n);
        }
        if self.antes_do_passo.is_none() {
            self.antes_do_passo = Some(Arc::clone(m.palavras()));
        }
        let (w, h) = m.tamanho();
        marca::liga_disco(bits, w, h, cx, cy, r);
        m.marca_disco(cx, cy, r);
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

    /// Um passo começa: a água de antes dele já não conta.
    pub(crate) fn comeca_o_passo(&mut self) {
        self.agua.fill(0);
    }

    fn limpa_o_passo(&mut self) {
        self.antes_do_passo = None;
        self.discos.fill(0);
        self.na_agua.fill(0);
        self.agua.fill(0);
    }

    /// Devolve a marca ao que era antes dos discos do passo em curso.
    fn tira_o_provisorio(&mut self) {
        if let (Some(m), Some(pre)) = (self.marca.as_mut(), self.antes_do_passo.take()) {
            m.poe_palavras(pre);
        }
    }

    /// **O passo regista-se**: marca o que o artista escreveu nele e devolve o que ligou.
    fn fecha_o_passo(
        &mut self,
        before: &ModelSnapshot,
        after: &ModelSnapshot,
        hint: Option<Region>,
    ) -> Option<DeltaDaMarca> {
        self.tira_o_provisorio();
        let n = self.palavras();
        let tamanho = self.marca.as_ref().map(MarcaDoArtista::tamanho);
        if self.suspensa
            || tamanho.is_none()
            || before.canvas_size != after.canvas_size
            || tamanho != Some(after.canvas_size)
        {
            self.limpa_o_passo();
            return None;
        }
        let mut s = vec![0u64; n];
        texels_mudados(before, after, hint, &mut s);
        for (k, b) in s.iter_mut().enumerate() {
            let agua = self.agua.get(k).copied().unwrap_or(0);
            let disco = self.discos.get(k).copied().unwrap_or(0);
            let na_agua = self.na_agua.get(k).copied().unwrap_or(0);
            *b = (*b & (!agua | disco)) | na_agua;
        }
        self.limpa_o_passo();
        let d = self.marca.as_mut()?.une_bits(&s);
        (!d.palavras.is_empty()).then_some(d)
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
