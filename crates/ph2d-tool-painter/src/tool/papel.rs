//! **O PAPEL do documento** (pedido do dono, 2026-10-05: *«A cor do papel na watercolor precisa ser
//! revisto pois funciona mal. Deveria funcionar para todos os modos e deveria ter um botão para
//! aplicar no papel como um todo.»*; escolhas: a cor substitui o branco do papel, nos quatro meios,
//! ao vivo, invisível no painel de camadas).
//!
//! Medido antes (`cor_do_papel_tests::diag_a_cor_do_papel_em_cada_meio`): a cor só existia como o
//! chão óptico da aquarela — no Digital, no Impasto e no Wet Paint mudava `0` texels; na aquarela, `0`
//! sobre a tela branca opaca, e numa camada transparente tingia o PRÓPRIO TRAÇO (`1 211` texels) com
//! o papel em volta sem cor.
//!
//! O papel é uma propriedade do DOCUMENTO (`PainterTool::papel`): o composite de todas as portas é
//! *camadas SOBRE o papel*, antes da luz do relevo (o dente ilumina o papel), e o chão da aquarela é
//! ele. **Escolher a cor no seletor aplica o papel** (o botão que existiu saiu, 2026-10-06): a 1.ª
//! cor separa o papel da tinta na camada de fundo ([`separa_o_branco`]: o branco puro vira
//! transparente e a orla de cada traço volta a ser a tinta dele com o alfa que tinha, a tinta opaca do
//! miolo fica) — e as seguintes repintam-no ao vivo; o arrasto inteiro é um passo de desfazer.
//! Pintar e depois escolher o papel dá a imagem de escolher o papel e depois pintar, ±1 nível (BUGS
//! #42: antes só o branco PURO saía, e a orla AA guardava a mistura — um fio claro sobre papel de cor).
//!
//! **O papel existe desde o primeiro instante** (BUGS #45, o estado da arte medido no Rebelle, doc 47):
//! uma sprite toda branca nasce papel branco com a camada VAZIA ([`PainterTool::abre_a_sprite`]), então
//! cada traço guarda o alfa do pincel e escurecer o papel depois dá o mesmo que pintar sobre ele; e o
//! papel nunca entra na tinta, nem como o chão óptico da aguada ([`PainterTool::cor_do_chao`]). A
//! separação acima fica para a arte que chegou branca.

use super::PainterTool;
use crate::compositor::Region;
use std::sync::Arc;

/// O papel de um desenho novo.
pub(crate) const PAPEL_BRANCO: [u8; 3] = [255, 255, 255];

/// **Uma tela inteiramente branca opaca é papel branco sem tinta** — a porta onde uma sprite vira
/// documento ([`PainterTool::abre_a_sprite`]) guarda-a assim: camada transparente sobre o papel
/// branco, que se mostra e se assa igual ao byte, e cada traço guarda o alfa que o pincel lhe deu (o
/// Rebelle, doc 47). Uma tela com qualquer outro píxel é arte, e fica como chegou ([`separa_o_branco`]).
pub(crate) fn e_papel_em_branco(
    rgba: &[u8], // COLOR-RAW-OK: o plano RGBA8 cru da sprite (o do `set_source`)
) -> bool {
    use rayon::prelude::*;
    !rgba.is_empty() && rgba.par_chunks(FATIA).all(|c| c.iter().all(|&b| b == 255))
}

/// A camada não tem tinta nenhuma (todo alfa a zero).
pub(crate) fn camada_vazia(rgba: &[u8], // COLOR-RAW-OK: o plano RGBA8 cru da camada
) -> bool {
    use rayon::prelude::*;
    rgba.par_chunks(FATIA)
        .all(|c| c.as_chunks::<4>().0.iter().all(|px| px[3] == 0))
}

/// A fatia das varreduras da tela inteira em `rayon` (múltipla de 4): abrir uma sprite de 4096² em
/// branco custava `16` ms num núcleo (`diag_o_quadro_de_um_desenho_novo`).
const FATIA: usize = 1 << 18;

/// **A lei única: um píxel RGBA8 (alfa direito) SOBRE o papel `p`** — opaco no fim. Inteira, com
/// arredondamento ao mais próximo, para ser a mesma em toda porta.
#[inline]
pub(crate) fn sobre_o_papel(px: &mut [u8; 4], p: [u8; 3]) {
    let a = u32::from(px[3]);
    if a == 255 {
        return;
    }
    for c in 0..3 {
        px[c] = ((u32::from(px[c]) * a + u32::from(p[c]) * (255 - a) + 127) / 255) as u8;
    }
    px[3] = 255;
}

/// Distância máxima de uma cor à reta branco→tinta para ser essa tinta numa opacidade menor: `2`
/// níveis (o arredondamento de dois `u8`, `~1,7`) mais `20 %` da intensidade — cada dab arredonda, e
/// na cauda de um traço mole um píxel de intensidade `~20` sai da reta por `3,4` níveis (medido no
/// laço de `papel_orla_tests`; com `2` fixos a cadeia partia ali e o píxel ficava opaco: `184` níveis).
const TOLERANCIA_DA_RETA: (f32, f32) = (2.0, 0.2);

/// **Separa o BRANCO do papel da tinta de uma camada pintada sobre ele** — o matting com o fundo
/// conhecido (branco): sobre branco, um píxel da orla de um traço é `c = F·a + branco·(1 − a)`, e
/// conhecida a tinta `F` ele volta a ser `(F, a)`, que é o que o traço deixaria numa camada
/// transparente. O branco puro vira transparente.
///
/// `F` sai do próprio traço: a partir do branco puro alcança-se cada vizinho (8) MAIS escuro, e de
/// cada píxel sobe-se pelo vizinho mais escuro na mesma reta branco→cor (a mesma tinta mais opaca) até
/// à CRISTA, que dá `F` e fica como está (a tinta opaca do miolo cobre o papel, escolha do dono). Uma
/// área lisa fica fora: o caminho só entra nela pela orla, e o interior, sem vizinho mais escuro, é
/// crista. Um píxel cuja subida acaba noutra tinta (fora da reta dela) fica. Só os opacos entram.
///
/// ⛔ Não é o «color to alpha» do GIMP: sem `F` ele escolhe a mistura mais transparente
/// (`a = 1 − min(c)`), e o miolo opaco de um vermelho (`220,40,40`) ficaria a `84 %`.
pub(crate) fn separa_o_branco(rgba: &mut [u8], w: usize, h: usize) {
    let n = w * h;
    debug_assert_eq!(rgba.len(), n * 4);
    // «branco − cor» de um píxel, e a distância AO QUADRADO ao branco — inteira (≤ `3·255²`), o que
    // deixa ordenar por contagem.
    let cor = |rgba: &[u8], i: usize| {
        [
            255 - i32::from(rgba[i * 4]),
            255 - i32::from(rgba[i * 4 + 1]),
            255 - i32::from(rgba[i * 4 + 2]),
        ]
    };
    let d2 = |c: [i32; 3]| (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]) as u32;
    // `m` na reta branco→`k` (ambos como «branco − cor»), com a tolerância de `TOLERANCIA_DA_RETA`.
    let na_reta = |m: [i32; 3], k: [i32; 3]| {
        let nk2 = d2(k) as f32;
        if nk2 <= 0.0 {
            return false;
        }
        let nm2 = d2(m) as f32;
        let dot = (m[0] * k[0] + m[1] * k[1] + m[2] * k[2]) as f32;
        let (fixa, relativa) = TOLERANCIA_DA_RETA;
        let tol = fixa + relativa * nm2.sqrt();
        nm2 - dot * dot / nk2 <= tol * tol
    };
    let vizinhos = |i: usize| {
        let (x, y) = (i % w, i / w);
        let (x0, x1, y0, y1) = (
            x.saturating_sub(1),
            (x + 1).min(w - 1),
            y.saturating_sub(1),
            (y + 1).min(h - 1),
        );
        (y0..=y1)
            .flat_map(move |vy| (x0..=x1).map(move |vx| vy * w + vx))
            .filter(move |&k| k != i)
    };
    let branco = |rgba: &[u8], i: usize| rgba[i * 4..i * 4 + 4] == [255, 255, 255, 255];
    let candidato = |rgba: &[u8], i: usize| rgba[i * 4 + 3] == 255 && !branco(rgba, i);
    // 1) A orla e o que ela alcança: semeada pelos píxeis encostados ao branco puro, e cada vizinho
    //    mais escuro a seguir.
    let mut na_orla = vec![false; n];
    let mut fila: Vec<usize> = {
        use rayon::prelude::*;
        let r: &[u8] = rgba;
        (0..n)
            .into_par_iter()
            .filter(|&i| candidato(r, i) && vizinhos(i).any(|k| branco(r, k)))
            .collect()
    };
    fila.iter().for_each(|&i| na_orla[i] = true);
    let mut orla = Vec::with_capacity(fila.len());
    while let Some(m) = fila.pop() {
        orla.push(m);
        let tm = d2(cor(rgba, m));
        for k in vizinhos(m) {
            if !na_orla[k] && candidato(rgba, k) && d2(cor(rgba, k)) > tm {
                na_orla[k] = true;
                fila.push(k);
            }
        }
    }
    // 2) A tinta de cada um: a do vizinho mais escuro na reta, da crista para fora — por distância ao
    //    branco DECRESCENTE (contagem), então o vizinho já tem a sua.
    let maximo = d2([255, 255, 255]) as usize;
    let mut conta = vec![0u32; maximo + 2];
    for &i in &orla {
        conta[maximo - d2(cor(rgba, i)) as usize + 1] += 1;
    }
    for b in 1..conta.len() {
        conta[b] += conta[b - 1];
    }
    let mut ordem = vec![0usize; orla.len()];
    for &i in &orla {
        let b = maximo - d2(cor(rgba, i)) as usize;
        ordem[conta[b] as usize] = i;
        conta[b] += 1;
    }
    let mut tinta = vec![usize::MAX; n];
    for &i in &ordem {
        let ci = cor(rgba, i);
        let ti = d2(ci);
        let mais_escuro = vizinhos(i)
            .filter(|&k| na_orla[k] && d2(cor(rgba, k)) > ti && na_reta(ci, cor(rgba, k)))
            .max_by_key(|&k| d2(cor(rgba, k)));
        tinta[i] = mais_escuro.map_or(i, |k| tinta[k]);
    }
    // 3) Cada píxel da orla na reta da sua tinta volta a ser `(F, a)`; a crista fica. O branco puro
    //    sai. Os novos valores saem todos da tela de ANTES (lidos em paralelo), depois escrevem-se.
    use rayon::prelude::*;
    let r: &[u8] = rgba;
    let novos: Vec<(usize, [u8; 4])> = ordem
        .par_iter()
        .filter_map(|&i| {
            let f = tinta[i];
            let (ci, cf) = (cor(r, i), cor(r, f));
            if f == i || !na_reta(ci, cf) {
                return None;
            }
            let a = (ci[0] * cf[0] + ci[1] * cf[1] + ci[2] * cf[2]) as f32 / d2(cf) as f32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a8 = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            Some((i, [r[f * 4], r[f * 4 + 1], r[f * 4 + 2], a8]))
        })
        .collect();
    for (i, px) in novos {
        rgba[i * 4..i * 4 + 4].copy_from_slice(&px);
    }
    rgba.par_chunks_mut(4).for_each(|px| {
        if px == [255, 255, 255, 255] {
            px.fill(0);
        }
    });
}

impl PainterTool {
    /// **A sprite vira documento** — a porta de [`Self::bind_document`] (o produto: o *New Image…* em
    /// branco, a troca de sprite, o merge). Uma sprite toda branca opaca nasce papel branco sem tinta
    /// ([`e_papel_em_branco`], BUGS #45); qualquer outra é a camada, sem papel, como o
    /// [`super::RasterEditTool::set_source`] cru.
    pub(crate) fn abre_a_sprite(
        &mut self,
        mut rgba: Vec<u8>, // COLOR-RAW-OK: a fonte crua do `set_source`
        w: u32,
        h: u32,
    ) {
        use super::RasterEditTool;
        let em_branco = e_papel_em_branco(&rgba);
        if em_branco {
            use rayon::prelude::*;
            rgba.par_chunks_mut(FATIA).for_each(|c| c.fill(0));
        }
        self.set_source(rgba, w, h);
        if em_branco {
            self.papel = Some(PAPEL_BRANCO);
        }
    }

    /// O papel do documento ligado (`None` = sem papel: o composite é o de sempre, ao byte).
    #[must_use]
    pub fn papel(&self) -> Option<[u8; 3]> {
        self.papel
    }

    /// **Compõe a região `r` do composite `rgba` (do tamanho da região) sobre o papel** — a porta
    /// que toda saída do documento chama ENTRE o composite e a luz do relevo.
    pub(crate) fn compoe_sobre_o_papel(&self, rgba: &mut [u8], r: Region) {
        let Some(p) = self.papel else {
            return;
        };
        debug_assert_eq!(rgba.len(), (r.w as usize) * (r.h as usize) * 4);
        for px in rgba.as_chunks_mut::<4>().0 {
            sobre_o_papel(px, p);
        }
    }

    /// **O papel de cor atravessa o vidro da aguada** — a saída compõe-se pela passada com a
    /// transparência por canal ([`Self::composto_pelo_vidro`]). O produtor de GPU não a conhece, e
    /// recusa-se enquanto isto for verdade (a CPU produz, como com a proteção).
    #[must_use]
    pub fn papel_atravessa_vidro(&self) -> bool {
        self.papel
            .is_some_and(|p| p != PAPEL_BRANCO && !self.vidros.is_empty())
    }

    /// **A região `r` sobre o papel PELO VIDRO** ([`crate::compositor::vidro`]): `Some` quando o papel é
    /// de cor e a aguada pintou (o papel atravessa cada canal pelo alfa dele); `None` = a porta de
    /// sempre (o composite e [`Self::compoe_sobre_o_papel`]), ao byte.
    pub(crate) fn composto_pelo_vidro(
        &self,
        src: &super::internal::ToolPixelSource<'_>,
        r: Region,
    ) -> Option<Vec<u8>> {
        let p = self.papel.filter(|_| self.papel_atravessa_vidro())?;
        Some(crate::compositor::composite_region_sobre_o_papel(
            &self.layers,
            src,
            &self.vidros,
            self.source_size,
            r,
            p,
        ))
    }

    /// A cor do chão que a óptica da aquarela vê onde nada está pintado por baixo: o BRANCO de
    /// referência, com qualquer papel. O papel nunca entra na tinta — a aguada guarda o alfa dela e o
    /// papel compõe-se por baixo, então mudar o papel depois dá o mesmo que pintar sobre ele (BUGS #45;
    /// com o papel no chão, a aguada pintada no branco escurecia `0,735` do que devia, pior `62`).
    #[must_use]
    pub(crate) fn cor_do_chao(&self) -> [u8; 3] {
        PAPEL_BRANCO
    }

    /// **Aplica o papel com a cor do seletor** — a porta programática (o produto aplica pelo próprio
    /// seletor, [`Self::papel_segue_a_cor`]). Um passo de desfazer.
    pub fn aplica_o_papel(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        if self.papel.is_none() {
            self.tira_o_branco_do_fundo();
        }
        self.papel = Some(self.paper_color_rgb8());
        self.edited_since_bind = true;
        self.commit_structural_edit(before);
        self.invalidate_composite();
    }

    /// A camada raster de baixo deixa o branco do papel ([`separa_o_branco`]) — onde o papel aparece.
    fn tira_o_branco_do_fundo(&mut self) {
        let fundo = self.layers.z_order_bottom_up().into_iter().find(|&id| {
            matches!(
                self.layers.get(id).map(|l| &l.kind),
                Some(crate::layers::LayerKind::Raster(_))
            )
        });
        let Some(fundo) = fundo else {
            return;
        };
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if self.layers.active() == Some(fundo) {
            let mut c = self.canvas_rgba.as_ref().clone();
            separa_o_branco(&mut c, w, h);
            self.replace_canvas(Arc::new(c));
        } else if let Some(img) = self.images.get(&fundo) {
            let mut img = img.as_ref().clone();
            let (iw, ih) = (img.width as usize, img.height as usize);
            separa_o_branco(&mut img.rgba8, iw, ih);
            self.images.insert(fundo, Arc::new(img));
        }
        self.bump_layer_pixels(Some(fundo));
    }

    /// **A cor do papel mudou no SELETOR: o papel segue-a ao vivo** (dono, 2026-10-06: *«o botão apply
    /// to paper parece supérfluo. não seria melhor aplicar ao usar o próprio seletor de cor?»*). A 1.ª
    /// cor aplica o papel (o branco puro do fundo sai), as seguintes repintam-no; a rajada inteira do
    /// arrasto — a aplicação incluída — é UM passo de desfazer
    /// ([`crate::undo::CoalesceKind::CorDoPapel`]). Branco num documento sem papel não faz nada (o
    /// branco do fundo já é esse papel).
    pub(crate) fn papel_segue_a_cor(&mut self) {
        let nova = self.paper_color_rgb8();
        if self.papel == Some(nova) || (self.papel.is_none() && nova == [255, 255, 255]) {
            return;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        if self.papel.is_none() {
            self.tira_o_branco_do_fundo();
        }
        self.papel = Some(nova);
        self.edited_since_bind = true;
        self.commit_structural_edit_coalesced(crate::undo::CoalesceKind::CorDoPapel, before);
        self.invalidate_composite();
    }
}
