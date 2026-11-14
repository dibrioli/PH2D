//! **A LISTA dos dezenove planos canvas-shaped de um `ModelSnapshot`** — o irmão de
//! [`crate::undo_delta`], que é o motor.
//!
//! O corte entre os dois arquivos é por responsabilidade: *como um plano é guardado* (a janela, o diff,
//! o fallback) mora lá; *quais são os planos, e com que stride* mora aqui, ao lado da única coisa que
//! precisa mudar quando um vigésimo nascer.

use crate::layers::LayerId as RtLayerId;
use crate::undo_delta::{StoredImages, StoredMap, StoredPlane, Strides};
use ph2d_painter_brush::material::MaterialBytes;
use std::sync::Arc;

/// O que uma entrada retém e o que clonar aloca — as contas do tecto em BYTES.
#[path = "undo_planes_bytes.rs"]
mod bytes;
/// Os instrumentos que leem a lista (relatórios de variante e de alcance, e as divergências de dois
/// snapshots) — só de teste e debug, cortados daqui pelo tecto de LOC, POR ASSUNTO.
#[path = "undo_planes_inspect.rs"]
mod inspect;
/// O teste «a linha é a neutra» de cada plano do relevo.
#[path = "undo_planes_neutra.rs"]
mod neutra;
use neutra::{
    altura_neutra, cobertura_neutra, linha_de_altura_neutra, linha_de_cobertura_neutra,
    linha_de_material_neutra,
};

/// **De onde sai o lado `before` do RELEVO** — dos journals do passo, em vez de um segundo snapshot
/// (doc 28 §5.58.2, degrau 2).
///
/// ⚠️ **Não é gateado por `cfg`, e a razão é a mesma do [`ReliefPlane`](crate::undo::window::ReliefPlane):**
/// ele atravessa a assinatura de [`PlaneDeltas::split`], que o commit chama em **qualquer** perfil. Um
/// tipo que só existisse em debug daria duas formas à porta — e uma porta com duas formas é o começo de
/// duas respostas.
///
/// ⚠️ **E o `expect(dead_code)` que morava aqui MORREU na promoção, como ele próprio prometia:** ele
/// dizia que em release os três campos só eram lidos no ramo de debug, e que o degrau 4 os faria
/// serem lidos — aí o atributo viraria erro e obrigaria a removê-lo. Foi exatamente o que aconteceu.
/// Um `allow` teria sobrevivido calado.
pub(crate) struct ReliefSource<'a> {
    pub(crate) state: &'a crate::undo::window::WriteState,
    /// O contador de escritas do `before` — a metade de PROVENIÊNCIA do guard.
    pub(crate) writes: u64,
    /// A camada de que os journals falam.
    pub(crate) layer: RtLayerId,
}

thread_local! {
    /// A×B do «+» sem diff ([`PlaneDeltas::tira_a_tela_nova`]; `false` = o `split` varre a tela, a rota
    /// antiga). Só a sonda e o gate o desligam.
    #[doc(hidden)]
    pub static MAIS_SEM_DIFF: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(test)]
thread_local! {
    /// **Quantas vezes o relevo saiu do JOURNAL neste thread** — o contador que impede o gate de igualdade
    /// de ser verde por VÁCUO.
    ///
    /// ⚠️ Sem ele, um gate que compara as duas rotas passa perfeitamente quando **as duas** caem no
    /// fallback: ele compararia o caminho de sempre contra ele mesmo, sobre uma rota que nunca rodou. É a
    /// armadilha que o ADR-0120 nomeou e que o oráculo de undo do ADR-0124 pagou uma segunda vez.
    ///
    /// **Por thread e não global com trava:** esta crate não roda o `split` sob rayon, então cada teste
    /// conta o próprio trabalho — e um contador por-thread não tem a lista de "quem precisa travar" que a
    /// §5.57 viu apodrecer no contador da LUT do `ph2d-painter-brush`.
    pub(crate) static RELIEF_FROM_JOURNAL: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// O mesmo contador para o vidro (ADR-0205): o gate do round-trip não pode ser verde por vácuo.
    pub(crate) static VIDRO_FROM_JOURNAL: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// **Os DEZENOVE planos canvas-shaped de um `ModelSnapshot`, guardados como delta.**
///
/// ⚠️ A lista é uma só, e os dois lados dela ([`Self::split`] e [`Self::side`]) andam juntos por
/// construção. Os dois modos de falha de esquecer um plano são bem diferentes, e é por isso que cada um
/// tem o seu gate:
///
/// - **esquecer no `split`** = o plano fica guardado INTEIRO nos dois endpoints. Falha *segura* — o undo
///   continua correto — e **invisível**, porque o único sintoma é memória. Quem pega é
///   `measure_undo_memory`, que mede o retido depois de 24 traços.
/// - **esquecer no `side`** = o plano volta como o do CURSOR, isto é, o undo não desfaz aquele plano.
///   Quem pega é `every_plane_of_a_snapshot_survives_the_round_trip`, que faz os dezenove diferirem.
#[derive(Clone, Debug, Default)]
pub(crate) struct PlaneDeltas {
    canvas_rgba: StoredPlane<u8>,
    images: StoredImages,
    heights: StoredMap<f32>,
    covers: StoredMap<u8>,
    mats: StoredMap<MaterialBytes>,
    /// O vidro por camada (`[u8; 7]` por texel, um por pixel).
    vidros: StoredMap<crate::compositor::vidro::Vidro>,
    mask_scratch: StoredPlane<u8>,
    selection_mask: StoredPlane<u8>,
    selection_crisp: StoredPlane<u8>,
    deform_disp: StoredPlane<[f32; 2]>,
    deform_pre: StoredPlane<u8>,
    deform_pre_h: StoredPlane<f32>,
    deform_pre_cover: StoredPlane<u8>,
    deform_pre_mats: StoredPlane<MaterialBytes>,
    sculpt_pre: StoredPlane<f32>,
    sculpt_amount: StoredPlane<f32>,
    sculpt_plane_sum: StoredPlane<f32>,
    sculpt_pre_cover: StoredPlane<u8>,
    sculpt_pre_mats: StoredPlane<MaterialBytes>,
    sculpt_pre_rgba: StoredPlane<u8>,
    /// **Este passo não pôde descrever o relevo** — o `before` o elidiu e o journal recusou (degrau 4,
    /// doc 28 §5.60). Não é um plano: é o veredito de que os três acima **não** dizem a verdade sobre
    /// um `before` que já não existe em lugar nenhum, e quem chama tem de descartar a história em vez
    /// de guardar uma entrada que apagaria relevo ao ser desfeita.
    pub(crate) relief_indescribable: bool,
}

impl PlaneDeltas {
    /// **A região do canvas a que este passo está confinado**, ou `None` para *não confinado*.
    ///
    /// Quem responde `Some(r)` habilita o undo/redo a publicar `r` como dirty-rect em vez de mandar a
    /// tela inteira ser refeita — medido, um quadro pós-Ctrl+Z a 4096² com impasto cai de **386,74 ms
    /// para a ordem de 0,6 ms**, que é o que o MESMO quadro custa no meio de um traço (doc 28 §5.62).
    ///
    /// ⚠️ **Os campos são desestruturados um a um, sem `..`, DE PROPÓSITO.** Um plano novo que ninguém
    /// classificasse aqui seria incluído numa reivindicação de confinamento sem nunca ter sido
    /// examinado — e o modo de falha disso é servir pixels velhos em volta do retângulo, corrupção
    /// visual que nenhum gate de conteúdo pega (o conteúdo DENTRO da janela está certo). Com o
    /// destructure exaustivo, acrescentar um plano **não compila** até ele ser classificado.
    ///
    /// ⚠️ **`relief_indescribable` força `None`**: um passo cujo relevo não pôde ser descrito não pode
    /// prometer que só mexeu num retângulo.
    #[cfg(test)]
    pub(crate) fn confined_region(&self, canvas_w: u32) -> Option<crate::compositor::Region> {
        self.confined_sujo(canvas_w).map(|(r, _)| r)
    }

    /// [`Self::confined_region`] e as caixas que a cobrem (B17.5; vazia quando uma só basta).
    pub(crate) fn confined_sujo(
        &self,
        canvas_w: u32,
    ) -> Option<(crate::compositor::Region, Vec<crate::compositor::Region>)> {
        use crate::undo_delta::confine::Confine;
        let Self {
            canvas_rgba,
            images,
            heights,
            covers,
            mats,
            vidros,
            mask_scratch,
            selection_mask,
            selection_crisp,
            deform_disp,
            deform_pre,
            deform_pre_h,
            deform_pre_cover,
            deform_pre_mats,
            sculpt_pre,
            sculpt_amount,
            sculpt_plane_sum,
            sculpt_pre_cover,
            sculpt_pre_mats,
            sculpt_pre_rgba,
            relief_indescribable,
        } = self;
        if *relief_indescribable {
            return None;
        }
        let mut c = Confine::new(canvas_w);
        c.add_plano(canvas_rgba);
        c.add_images(images);
        c.add_relevo(heights, linha_de_altura_neutra, altura_neutra);
        c.add_relevo(covers, linha_de_cobertura_neutra, cobertura_neutra);
        let neutro = ph2d_painter_brush::material::Material::NEUTRAL.to_bytes();
        let modelo = vec![neutro; canvas_w as usize];
        c.add_relevo(
            mats,
            |l| linha_de_material_neutra(l, &modelo),
            move |m| *m == neutro,
        );
        c.add_vidros(vidros);
        c.add_plano(mask_scratch);
        c.add_plano(selection_mask);
        c.add_plano(selection_crisp);
        c.add_plano(deform_disp);
        c.add_plano(deform_pre);
        c.add_plano(deform_pre_h);
        c.add_plano(deform_pre_cover);
        c.add_plano(deform_pre_mats);
        c.add_plano(sculpt_pre);
        c.add_plano(sculpt_amount);
        c.add_plano(sculpt_plane_sum);
        c.add_plano(sculpt_pre_cover);
        c.add_plano(sculpt_pre_mats);
        c.add_plano(sculpt_pre_rgba);
        c.finish_caixas()
    }

    /// **O «+» transparente** (n.º 7 item 3, chave [`MAIS_SEM_DIFF`]): tira as duas telas dos endpoints
    /// e devolve a de antes, se os dois a seguram (sem elisão) com o mesmo comprimento; os endpoints ficam
    /// com o MESMO plano vazio (o `split` lê `Unchanged`) e [`Self::poe_a_tela_nova`] guarda a de antes.
    /// `None` = deixa tudo como estava (o `split` varre, a rota antiga).
    pub(crate) fn tira_a_tela_nova(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
    ) -> Option<Arc<Vec<u8>>> {
        if !MAIS_SEM_DIFF.get()
            || before.canvas_elided.is_some()
            || after.canvas_elided.is_some()
            || before.canvas_rgba.len() != after.canvas_rgba.len()
            || Arc::ptr_eq(&before.canvas_rgba, &after.canvas_rgba)
        {
            return None;
        }
        debug_assert!(
            after.canvas_rgba.iter().all(|&b| b == 0),
            "a tela da camada nova não nasceu neutra: o refazer do «+» perderia píxeis"
        );
        let vazio = Arc::new(Vec::new());
        after.canvas_rgba = Arc::clone(&vazio);
        Some(std::mem::replace(&mut before.canvas_rgba, vazio))
    }

    /// Ver [`Self::tira_a_tela_nova`].
    pub(crate) fn poe_a_tela_nova(&mut self, before: Arc<Vec<u8>>) {
        self.canvas_rgba = StoredPlane::Nova { before };
    }

    /// Extrai os deltas dos dois endpoints e **esvazia os dois**: depois disto os `ModelSnapshot`
    /// guardados carregam só metadados, e os pixels vivem aqui.
    pub(crate) fn split(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        hint: Option<crate::compositor::Region>,
        relief: Option<ReliefSource<'_>>,
    ) -> Self {
        // O stride vem do endpoint que está sendo guardado. Um passo que muda o TAMANHO do canvas cai
        // sozinho no `Whole` (os comprimentos não batem), então basta um dos dois.
        let s = Strides::of(before.canvas_size.0);
        // A janela chega em PIXELS do canvas e cada plano a mede em ELEMENTOS (o RGBA carrega quatro por
        // pixel, os escalares um). `Strides` é quem sabe a conversão, então ela é feita aqui e uma vez.
        let cw = before.canvas_size.0;
        let win_rgba =
            hint.and_then(|r| crate::undo_delta::PlaneWindow::from_region(r, cw, s.rgba));
        let win_scalar =
            hint.and_then(|r| crate::undo_delta::PlaneWindow::from_region(r, cw, s.scalar));
        let tiles = relief.as_ref().and_then(|r| r.state.tiles_for(hint));
        let (canvas_rgba, canvas_lost) = match &before.canvas_elided {
            None => (
                StoredPlane::split_tiled(
                    &mut before.canvas_rgba,
                    &mut after.canvas_rgba,
                    s.rgba,
                    win_rgba,
                    tiles.as_deref().map(|t| (t, cw)),
                ),
                false,
            ),
            Some(_) => Self::canvas_from_journal(
                before,
                after,
                (s.rgba, win_rgba),
                tiles.as_deref().map(|t| (t, cw)),
                relief.as_ref(),
            ),
        };
        crate::sonda_partes::parte("split.tela");
        // O vidro ANTES do relevo: o `relief_maps` zera as testemunhas que lê (ADR-0205 D4).
        let (mut vidros, vidro_indescribable) =
            Self::vidro_map(before, after, s.scalar, win_scalar, relief.as_ref());
        crate::sonda_partes::parte("split.vidro");
        let (mut heights, mut covers, mut mats, relief_indescribable) =
            Self::relief_maps(before, after, s.scalar, win_scalar, relief);
        crate::sonda_partes::parte("split.relevo");
        heights.guarda_janelas_que_aparecem(win_scalar, linha_de_altura_neutra, altura_neutra);
        covers.guarda_janelas_que_aparecem(win_scalar, linha_de_cobertura_neutra, cobertura_neutra);
        let neutro = ph2d_painter_brush::material::Material::NEUTRAL.to_bytes();
        let modelo = vec![neutro; cw as usize];
        mats.guarda_janelas_que_aparecem(
            win_scalar,
            |l| linha_de_material_neutra(l, &modelo),
            move |m| *m == neutro,
        );
        vidros.guarda_janelas_que_aparecem(
            win_scalar,
            crate::undo_delta::confine::linha_sem_vidro,
            crate::undo_delta::confine::vidro_neutro,
        );
        if crate::undo::window::vidro_pelo_journal() {
            vidros.larga_os_planos_que_aparecem();
        }
        crate::sonda_partes::parte("split.janelas");
        let images = StoredImages::split(&mut before.images, &mut after.images);
        crate::sonda_partes::parte("split.imagens");
        Self {
            relief_indescribable: relief_indescribable || canvas_lost || vidro_indescribable,
            canvas_rgba,
            images,
            vidros,
            heights,
            covers,
            mats,
            mask_scratch: StoredPlane::split(
                &mut before.mask_scratch,
                &mut after.mask_scratch,
                s.rgba,
                win_rgba,
            ),
            selection_mask: StoredPlane::split(
                &mut before.selection_mask,
                &mut after.selection_mask,
                s.scalar,
                win_scalar,
            ),
            selection_crisp: StoredPlane::split(
                &mut before.selection_crisp,
                &mut after.selection_crisp,
                s.scalar,
                win_scalar,
            ),
            deform_disp: StoredPlane::split(
                &mut before.deform.disp,
                &mut after.deform.disp,
                s.scalar,
                win_scalar,
            ),
            deform_pre: StoredPlane::split(
                &mut before.deform.pre,
                &mut after.deform.pre,
                s.rgba,
                win_rgba,
            ),
            deform_pre_h: StoredPlane::split(
                &mut before.deform.pre_h,
                &mut after.deform.pre_h,
                s.scalar,
                win_scalar,
            ),
            deform_pre_cover: StoredPlane::split(
                &mut before.deform.pre_cover,
                &mut after.deform.pre_cover,
                s.scalar,
                win_scalar,
            ),
            deform_pre_mats: StoredPlane::split(
                &mut before.deform.pre_mats,
                &mut after.deform.pre_mats,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre: StoredPlane::split(
                &mut before.sculpt.pre,
                &mut after.sculpt.pre,
                s.scalar,
                win_scalar,
            ),
            sculpt_amount: StoredPlane::split(
                &mut before.sculpt.amount,
                &mut after.sculpt.amount,
                s.scalar,
                win_scalar,
            ),
            sculpt_plane_sum: StoredPlane::split(
                &mut before.sculpt.plane_sum,
                &mut after.sculpt.plane_sum,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_cover: StoredPlane::split(
                &mut before.sculpt.pre_cover,
                &mut after.sculpt.pre_cover,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_mats: StoredPlane::split(
                &mut before.sculpt.pre_mats,
                &mut after.sculpt.pre_mats,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_rgba: StoredPlane::split(
                &mut before.sculpt.pre_rgba,
                &mut after.sculpt.pre_rgba,
                s.rgba,
                win_rgba,
            ),
        }
    }

    /// **Os TRÊS mapas de relevo** — pelos journals do passo quando eles o descrevem, pelo `split` de
    /// dois snapshots quando não (doc 28 §5.58.2, degrau 2).
    ///
    /// ⚠️ **Tudo-ou-nada, e o esvaziamento é o que a torna assim:** as três famílias são um fato só
    /// sobre um traço (a altura, a cobertura e o material — doc 18 §10.4), e qualquer uma pode recusar.
    /// O [`StoredMap::from_journal`] por isso **não** esvazia; quem esvazia é esta função, depois de as
    /// três passarem. Esvaziar à medida deixaria o `heights` drenado quando o `mats` recusasse, e o
    /// caminho de fallback partiria de mapas vazios — perdendo o relevo em silêncio.
    ///
    /// ⚠️ **Em release ela toma a rota do JOURNAL**, e é isso que faz o 1º dab escrever no lugar: o
    /// degrau 4 promoveu o journal do relevo junto com a elisão do `before`, e um sem o outro é
    /// regressão nos dois sentidos. Só o journal do CANVAS ficou `cfg(debug)`.
    ///
    /// ⚠️ **Este parágrafo afirmava o oposto até 2026-08-02** (*"em release ela é hoje SEMPRE o caminho
    /// de sempre, porque o journal ainda é `cfg(debug)`"* — verdade no degrau 2, falsa depois da
    /// promoção), e foi ele que fez uma investigação de posse mirar a porta errada e publicar um
    /// veredito falso (doc 28 §5.67).
    /// **A tela de um `before` ELIDIDO** (B17.4): o lado de antes é o journal da tela, válido quando ele descreve
    /// ESTE passo e a testemunha é o plano do `after` (o vivo). Senão `(_, true)`: o antes não existe em lugar
    /// nenhum, e quem chama descarta a história (o molde do relevo, `discard_if_relief_is_lost`).
    fn canvas_from_journal(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        (stride, win): (usize, Option<crate::undo_delta::PlaneWindow>),
        tiles: Option<(&[crate::compositor::Region], u32)>,
        src: Option<&ReliefSource<'_>>,
    ) -> (StoredPlane<u8>, bool) {
        let ok = src.is_some_and(|src| {
            src.state.journal_describes_step_at(src.writes) && !src.state.on_foreign_plane()
        }) && (src.is_some_and(|s| !s.state.canvas_untouched())
            || before
                .canvas_elided
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .is_some_and(|p| std::sync::Arc::ptr_eq(&p, &after.canvas_rgba)));
        let built = ok
            .then(|| {
                src.and_then(|src| {
                    let j = src.state.canvas_journal();
                    // Captura INTEIRA (quem não sabia onde ia escrever): o antes já está completo, e a janela
                    // declarada + os tiles marcados guardam só o que mudou, como o `split` de um antes segurado.
                    if let Some(w) = j.whole().filter(|w| w.len() == after.canvas_rgba.len()) {
                        let mut b = std::sync::Arc::clone(w);
                        let mut a = std::sync::Arc::clone(&after.canvas_rgba);
                        return Some(StoredPlane::split_tiled(&mut b, &mut a, stride, win, tiles));
                    }
                    StoredPlane::from_journal(&after.canvas_rgba, &j, win)
                })
            })
            .flatten();
        before.canvas_elided = None;
        before.canvas_rgba = std::sync::Arc::new(Vec::new());
        after.canvas_rgba = std::sync::Arc::new(Vec::new());
        match built {
            Some(p) => (p, false),
            None => (StoredPlane::Unchanged, true),
        }
    }

    /// **O delta do vidro** (ADR-0205 D4): com o `before` a DESCREVER o vidro, pelo journal por tile
    /// ([`StoredMap::from_journal`]); a segurá-lo (a chave desligada, um snapshot estrutural), pelo `split`.
    /// Elidido e o journal a recusar = indescritível: o `split` leria toda chave como `OnlyAfter` e o undo
    /// APAGARIA o vidro (o mesmo braço do relevo, abaixo).
    fn vidro_map(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        stride: usize,
        win: Option<crate::undo_delta::PlaneWindow>,
        relief: Option<&ReliefSource<'_>>,
    ) -> (StoredMap<crate::compositor::vidro::Vidro>, bool) {
        let el = std::mem::take(&mut before.relief_elided.vidros);
        if el.is_empty() {
            return (
                StoredMap::split(&mut before.vidros, &mut after.vidros, stride, win),
                false,
            );
        }
        let built = relief
            .and_then(|src| {
                src.state
                    .with_vidro_before(src.writes, src.layer, |l, absent, j| {
                        StoredMap::from_journal(
                            &before.vidros,
                            &el,
                            &after.vidros,
                            l,
                            absent,
                            j,
                            win,
                        )
                    })
            })
            .flatten();
        before.vidros.clear();
        after.vidros.clear();
        match built {
            Some(m) => {
                #[cfg(test)]
                VIDRO_FROM_JOURNAL.with(|c| c.set(c.get() + 1));
                (m, false)
            }
            None => (StoredMap::default(), true),
        }
    }

    fn relief_maps(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        stride: usize,
        win: Option<crate::undo_delta::PlaneWindow>,
        relief: Option<ReliefSource<'_>>,
    ) -> (
        StoredMap<f32>,
        StoredMap<u8>,
        StoredMap<MaterialBytes>,
        bool,
    ) {
        {
            if let Some(src) = relief {
                let el = &before.relief_elided;
                let built = src
                    .state
                    .with_relief_before(src.writes, src.layer, |j| {
                        let l = src.layer;
                        Some((
                            StoredMap::from_journal(
                                &before.heights,
                                &el.heights,
                                &after.heights,
                                l,
                                j.absent[0],
                                j.heights,
                                win,
                            )?,
                            StoredMap::from_journal(
                                &before.covers,
                                &el.covers,
                                &after.covers,
                                l,
                                j.absent[1],
                                j.covers,
                                win,
                            )?,
                            StoredMap::from_journal(
                                &before.mats,
                                &el.mats,
                                &after.mats,
                                l,
                                j.absent[2],
                                j.mats,
                                win,
                            )?,
                        ))
                    })
                    .flatten();
                if let Some(maps) = built {
                    #[cfg(test)]
                    RELIEF_FROM_JOURNAL.with(|c| c.set(c.get() + 1));
                    before.heights.clear();
                    after.heights.clear();
                    before.covers.clear();
                    after.covers.clear();
                    before.mats.clear();
                    after.mats.clear();
                    before.relief_elided = crate::undo::elide::ElidedRelief::default();
                    return (maps.0, maps.1, maps.2, false);
                }
            }
        }
        // ⚠️ **A recusa não pode cair no `split` quando o `before` ELIDIU** (degrau 4, doc 28 §5.60).
        // O `split` lê os mapas fortes, que num snapshot elidido estão vazios: toda chave sairia como
        // `OnlyAfter` = *"não existia antes"*, e desfazer **apagaria o relevo**. Um `before` elidido não
        // tem os planos em lugar nenhum — nem aqui nem no tool, que os escreveu no LUGAR — então a
        // resposta honesta é dizer que este passo é indescritível e deixar quem chama descartar a
        // história. *Lento nunca, errado jamais; e quando nem lento existe, LOUD.*
        let indescribable = !before.relief_elided.is_empty();
        (
            StoredMap::split(&mut before.heights, &mut after.heights, stride, win),
            StoredMap::split(&mut before.covers, &mut after.covers, stride, win),
            StoredMap::split(&mut before.mats, &mut after.mats, stride, win),
            indescribable,
        )
    }

    /// Reenche `out` (o endpoint guardado, esvaziado) a partir do **cursor** — e, para os três planos
    /// de RELEVO, a partir do plano **VIVO** (`live`).
    ///
    /// ⚠️ **Duas bases e não uma, e é o degrau 3 do S3** (doc 28 §5.58.2). O cursor larga o relevo no
    /// degrau 4, então quem responde *"de que estado o undo parte?"* para essas três famílias passa a
    /// ser o tool. Hoje as duas respostas são a MESMA — a identidade da §5.28 (*o vivo É o cursor, pelo
    /// mesmo buffer*, 233/233) —, e o `debug_assert` abaixo é quem a confere a cada undo da suíte em vez
    /// de a assumir.
    ///
    /// ⚠️ **A base tem de valer para tudo FORA da janela do patch**, que é o que `side` copia dela. É
    /// por isso que a pergunta certa é *o vivo é o cursor?* e não *o vivo é parecido?*.
    ///
    /// ⚠️ **E é por isso que só o UNDO/REDO passa o vivo.** O
    /// [`absorb_foreign_writes`](crate::undo::UndoController::absorb_foreign_writes) e a extensão de um
    /// run coalescido materializam o `before` de uma entrada contra o cursor ANTIGO, e no primeiro deles
    /// o vivo difere do cursor **por construção** — é exatamente a divergência que ele existe para
    /// reconciliar. Os dois passam o cursor nas duas pontas, e a primeira versão desta wave passava o
    /// vivo: o `debug_assert` abaixo nasceu vermelho em três gates (warp e os dois filtros do sculpt) e
    /// nomeou o sítio. *A base é o estado ADJACENTE à entrada, e nem sempre ele é o de agora.*
    ///
    /// `None` = a entrada não pode ser honrada com esta base; quem chama descarta o histórico.
    pub(crate) fn side(
        &self,
        cursor: &crate::undo::ModelSnapshot,
        live: &crate::undo::ModelSnapshot,
        out: &mut crate::undo::ModelSnapshot,
        want_before: bool,
    ) -> Option<()> {
        let b = want_before;
        #[cfg(debug_assertions)]
        Self::debug_relief_base_is_the_cursor(cursor, live);
        out.canvas_rgba = self.canvas_rgba.side(&cursor.canvas_rgba, b)?;
        out.images = self.images.side(&cursor.images, b)?;
        out.heights = self.heights.side(&live.heights, b)?;
        out.covers = self.covers.side(&live.covers, b)?;
        out.mats = self.mats.side(&live.mats, b)?;
        // O vidro do VIVO, como o relevo: o cursor descreve-o sem o segurar (ADR-0205 D2).
        let vbase = if cursor.relief_elided.vidros.is_empty() {
            &cursor.vidros
        } else {
            &live.vidros
        };
        out.vidros = self.vidros.side(vbase, b)?;
        out.mask_scratch = self.mask_scratch.side(&cursor.mask_scratch, b)?;
        out.selection_mask = self.selection_mask.side(&cursor.selection_mask, b)?;
        out.selection_crisp = self.selection_crisp.side(&cursor.selection_crisp, b)?;
        out.deform.disp = self.deform_disp.side(&cursor.deform.disp, b)?;
        out.deform.pre = self.deform_pre.side(&cursor.deform.pre, b)?;
        out.deform.pre_h = self.deform_pre_h.side(&cursor.deform.pre_h, b)?;
        out.deform.pre_cover = self.deform_pre_cover.side(&cursor.deform.pre_cover, b)?;
        out.deform.pre_mats = self.deform_pre_mats.side(&cursor.deform.pre_mats, b)?;
        out.sculpt.pre = self.sculpt_pre.side(&cursor.sculpt.pre, b)?;
        out.sculpt.amount = self.sculpt_amount.side(&cursor.sculpt.amount, b)?;
        out.sculpt.plane_sum = self.sculpt_plane_sum.side(&cursor.sculpt.plane_sum, b)?;
        out.sculpt.pre_cover = self.sculpt_pre_cover.side(&cursor.sculpt.pre_cover, b)?;
        out.sculpt.pre_mats = self.sculpt_pre_mats.side(&cursor.sculpt.pre_mats, b)?;
        out.sculpt.pre_rgba = self.sculpt_pre_rgba.side(&cursor.sculpt.pre_rgba, b)?;
        Some(())
    }

    /// **A rede do degrau 3: a base do relevo É o cursor.**
    ///
    /// Enquanto o cursor ainda carrega os três planos (até o degrau 4), a troca de base é
    /// byte-idêntica — e isto o afirma a cada undo/redo que a suíte encena, em vez de o assumir. Um
    /// mapa VAZIO no cursor é o próprio degrau 4 e sai calado: aí não há com que comparar, e a
    /// correção passa a se apoiar na cadeia (`absorb_foreign_writes`).
    #[cfg(debug_assertions)]
    fn debug_relief_base_is_the_cursor(
        cursor: &crate::undo::ModelSnapshot,
        live: &crate::undo::ModelSnapshot,
    ) {
        fn same<T: PartialEq>(
            c: &std::collections::BTreeMap<RtLayerId, std::sync::Arc<Vec<T>>>,
            l: &std::collections::BTreeMap<RtLayerId, std::sync::Arc<Vec<T>>>,
        ) -> bool {
            c.is_empty() // degrau 4: o cursor largou o relevo, não há com que comparar
                || (c.len() == l.len()
                    && c.iter().all(|(k, cv)| {
                        l.get(k)
                            .is_some_and(|lv| std::sync::Arc::ptr_eq(cv, lv) || **cv == **lv)
                    }))
        }
        debug_assert!(
            same(&cursor.heights, &live.heights)
                && same(&cursor.covers, &live.covers)
                && same(&cursor.mats, &live.mats),
            "a base do relevo NAO e o cursor: a materializacao escreveria a janela certa sobre um \
             fundo errado, e nada no sistema piscaria (doc 28 §5.58.2, degrau 3)"
        );
    }

    /// O plano do CANVAS, para a absorção re-parti-lo sem materializar a tela
    /// (`crate::undo::absorb`). Os outros planos continuam a ter uma porta só: o `split`/`side`.
    pub(crate) fn canvas(&self) -> &StoredPlane<u8> {
        &self.canvas_rgba
    }

    /// A mesma porta, para escrever o plano que a absorção calculou.
    pub(crate) fn canvas_mut(&mut self) -> &mut StoredPlane<u8> {
        &mut self.canvas_rgba
    }

    /// O mapa do vidro — a absorção barata do escorrido o compõe com o journal (ADR-0205).
    pub(crate) fn vidros(&self) -> &StoredMap<crate::compositor::vidro::Vidro> {
        &self.vidros
    }

    pub(crate) fn vidros_mut(&mut self) -> &mut StoredMap<crate::compositor::vidro::Vidro> {
        &mut self.vidros
    }
}
