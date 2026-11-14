//! The wet COMPOSITE half (child of [`super`] — workspace file-LOC cap):
//! pigment (with the doc-22 visual terms) OVER the frozen base, the
//! Selection/protection/alpha-lock gates, and the show-wet veil. A metade de cima (a óptica, o pigmento por
//! célula) é a função pura de [`super::composite_otica`]; aqui fica a orquestração e o laço de píxel.

use super::*;
use ph2d_wet_paint::render::RenderLayer;

/// O que o composite lê da ferramenta antes da óptica ([`PainterTool::wetpaint_antes_da_otica`]).
pub(super) struct AntesDaOtica {
    tela: (usize, usize),
    gates: (
        super::super::watercolor_accum::SplatGate,
        super::super::watercolor_accum::SplatGate,
        super::super::watercolor_accum::SplatGate,
    ),
    camada: Option<crate::layers::LayerId>,
    linhas: Option<bool>,
    marca: Option<u64>,
    exata: bool,
    pub(super) factos: super::otica_factos::FactosDaOtica,
}

impl PainterTool {
    /// Composite the engine's dirty rect: pigment (straight alpha) OVER the
    /// frozen base, written into `canvas_rgba`, preview marked.
    ///
    /// Selection / protection / alpha-lock (W2.5) are enforced HERE, at the
    /// single place the module writes the canvas: the fluid flows wherever
    /// the SIM takes it, and the gated texels keep-lerp back to the frozen
    /// base (`splat_keep` — the watercolor composite's exact law, a hard
    /// guarantee independent of where the water reached). Alpha-lock pins
    /// the layer's α to the frozen base (colour deposits into existing
    /// paint; the silhouette never moves). All gates off = byte-identical.
    pub(super) fn wetpaint_composite(&mut self) {
        self.wetpaint_composite_veiled(self.paint.wetpaint.show_wet);
    }

    /// [`Self::wetpaint_composite`] with the show-wet veil explicit — the
    /// end-session door forces `false` so the veil never bakes.
    pub(super) fn wetpaint_composite_veiled(&mut self, veil: bool) {
        if self.wet_tela_alheia() {
            return; // uma operação de outra ferramenta ainda está aberta: a tela é dela
        }
        #[cfg(test)]
        if super::otica_fora_porta::ligada() {
            self.wetpaint_composite_pelo_espelho(veil);
            return;
        }
        let antes = self.wetpaint_antes_da_otica();
        let compostas = self.wet_compostas_que_valem(antes.factos);
        let Some(entrada) = self.wetpaint_entrada_da_otica(antes.factos) else {
            return;
        };
        let sess = self
            .paint
            .wetpaint
            .session
            .as_mut()
            .expect("a entrada só existe com sessão");
        // A metade de cima (a óptica): só a entrada e as camadas, escrita nos planos ([`super::composite_otica`]).
        let layers: Vec<RenderLayer<'_>> = sess
            .engine
            .layers
            .iter()
            .map(|l| RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        let saida = super::composite_otica::compoe_a_otica(&entrada, &layers, &mut sess.planos);
        drop(layers);
        // F2h: o laço de píxel das ópticas que o operário compôs, sobre os planos já com a de agora.
        for c in compostas {
            #[cfg(test)]
            super::sessao_do_wet::COLADAS.set(super::sessao_do_wet::COLADAS.get() + 1);
            self.wetpaint_depois_da_otica(&antes, c, veil, super::veu::VeuDe::Motor, None);
        }
        self.wetpaint_depois_da_otica(&antes, saida, veil, super::veu::VeuDe::Motor, None);
    }

    /// O que o composite lê da ferramenta ANTES de trazer o motor para casa (gates, Smooth Edges, marca, factos).
    pub(super) fn wetpaint_antes_da_otica(&mut self) -> AntesDaOtica {
        // Qualquer composição leva o sujo inteiro do motor — o pendente do quadro também ([`super::por_quadro`]).
        self.paint.wetpaint.composite_pendente = false;
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        let gates = self.wet_splat_gates();
        let camada = self.layers.active().filter(|&id| !self.layers.is_mask(id));
        #[cfg(test)]
        let camada = camada.filter(|_| !self.sem_vidro);
        // As linhas que o Smooth Edges procura dentro da tinta: as da GRAVIDADE (o eixo mais perto dela).
        let wp = &self.paint.wetpaint;
        let linhas = (wp.tilt_on && wp.tilt_ring > 0).then(|| {
            let [gx, gy] = ph2d_wet_paint::sim::tilt_dir_for_spoke(wp.tilt_spoke);
            gy.abs() >= gx.abs()
        });
        // A água do artista (W12): a cobertura de cada texel escrito é o `pa` — exacta quando a tinta é
        // composta «por cima» da tela congelada (Mix, a camada única, sem vidro nem alfa preso); fora disso
        // o texel pintado fica cheio.
        let marca = wp.session.as_ref().and_then(|s| s.marca);
        let marca = marca.filter(|_| self.undo.marca.molhada_viva(marca));
        let exata = marca.is_some()
            && wp.session.as_ref().map(|s| s.blend) == Some(ph2d_painter_brush::BrushBlend::Mix)
            && gates.2.is_none()
            && camada.is_none_or(|c| !self.vidros.contains_key(&c))
            && self.preview_layer_stack_is_trivial()
            && !self.papel_atravessa_vidro();
        AntesDaOtica {
            tela: (w, h),
            gates,
            camada,
            linhas,
            marca,
            exata,
            factos: self.factos_da_otica(),
        }
    }

    /// Traz o motor para casa e tira dele a entrada da óptica (o sujo, os `Params`, a marca). `None` = sem sessão.
    pub(super) fn wetpaint_entrada_da_otica(
        &mut self,
        factos: super::otica_factos::FactosDaOtica,
    ) -> Option<super::composite_otica::EntradaDaOtica> {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            self.paint.wetpaint.oleo_na_tela = factos.oleo_da_tela(&[]).is_some();
            return None;
        };
        // ⚠️ Duas perguntas, como antes do J13b P3 (a decisão do óleo trazia o motor para casa ANTES): com uma fita de
        // macro a tocar, cada `bring_home` no roteiro consome uma resposta dela.
        if !factos.oleo_adiado {
            sess.bring_home();
        }
        sess.bring_home();
        Some(entrada_da_sessao(sess, factos, (w, h)))
    }

    /// A metade de baixo: aplica a [`super::composite_otica::SaidaDaOtica`] na ferramenta e corre o laço de píxel
    /// sobre os planos da sessão. `veu` = de onde o véu lê; tudo o que não é [`super::veu::VeuDe::DaImagem`] é uma
    /// composição LOCAL e a caixa dela entra na `uniao_local` ([`super::otica_no_tique`]).
    pub(super) fn wetpaint_depois_da_otica(
        &mut self,
        antes: &AntesDaOtica,
        saida: super::composite_otica::SaidaDaOtica,
        veil: bool,
        veu: super::veu::VeuDe<'_>,
        pronto: Option<&[u8]>,
    ) {
        let (w, h) = antes.tela;
        let (gate_sel, gate_prot, gate_alock) = &antes.gates;
        let (camada, linhas, marca, exata) = (antes.camada, antes.linhas, antes.marca, antes.exata);
        // O que a óptica escreve na ferramenta volta na saída.
        let oleo = saida.oleo;
        self.paint.wetpaint.oleo_na_tela = oleo.is_some();
        self.paint.wetpaint.composta = Some(saida.marca);
        // W3b: o dente do papel que a Transmission assa na tinta ([`super::grao_no_filme`]); sem papel escolhido, o
        // papel de fábrica (U2). Só o laço de píxel o lê; a óptica só perguntou se há (`factos.tem_grao`).
        let papel = self.papel_na_escala(super::super::substrate_relief::papel_do_documento(
            self.paint.brush.paper,
        ));
        let grao = oleo.and_then(|_| {
            super::grao_no_filme::GraoNoFilme::do_papel(
                papel,
                (self.paint.substrate_depth, self.paint.substrate_rough),
                self.paint.paper_image.as_ref().map(|i| i.as_mask()),
                &self.paint.impasto_rig,
                (w, h),
            )
            .map(|g| g.com_ganho(self.fator_aparencia))
        });
        let Some(super::composite_otica::CaixasDaOtica {
            caixa_cel,
            caixa_px: (px0, py0, px1, py1),
            caixa_suave: _,
            sem_oleo,
        }) = saida.caixas
        else {
            return;
        };
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let ratio = sess.ratio;
        let (gw, gh) = sess.grid;
        if !matches!(veu, super::veu::VeuDe::DaImagem(_)) {
            sess.otica.compos_local(caixa_cel);
        }
        // A marca do artista (W9): o que a água muda aqui não é do artista — guarda-se a caixa como está.
        let agua_antes: Option<Vec<u8>> = self.undo.marca.ativa().then(|| {
            let mut v = Vec::with_capacity((px1 - px0) * (py1 - py0) * 4);
            for y in py0..py1 {
                v.extend_from_slice(&self.canvas_rgba[(y * w + px0) * 4..(y * w + px1) * 4]);
            }
            v
        });
        let stride = w * 4;
        // ⚠️ **A FRONTEIRA DO MOTOR.** Tudo acima lê `sess.engine` (as camadas, o
        // `params`, o dirty); tudo abaixo lê `sess.planos`, `sess.base` e o
        // `canvas` — **e mais nada do motor**, exceto o véu no fim.
        //
        // ⛔ **MEDIDO E REJEITADO — não refaça: devolver o motor ao worker AQUI.**
        // O log do Enio (2026-08-01, janela assistindo) mostra `worker: busy 67 %
        // **away 24 %** sleep 9 %` com a água a 34,0 Hz contra 40 nominais, e o
        // `split` abaixo diz que **55,8 % do composite não toca o motor** na razão
        // 1. A conclusão óbvia — *o worker está parado à toa, devolva antes* — foi
        // construída (uma porta `wetpaint_composite_releasing` para o tick, com o
        // véu vetando) e **o relógio a negou**: A/B costas-com-costas em máquina
        // calma, três amostras cada, `heavy_puddle` a 4096²:
        //
        // | | água | tick p50 | tick max |
        // |---|---|---|---|
        // | sem | 35,6 / 36,2 / 36,1 Hz | 4,1-5,5 | **12,9 / 15,9 / 14,3 ms** |
        // | com | 36,6 / 36,6 / 36,9 Hz | 4,1-5,8 | **20,5 / 16,7 / 27,5 ms** |
        //
        // **+0,6 Hz de água (2 %) ao preço de +50 % no PIOR TICK** — um hitch que
        // o artista vê trocado por uma taxa que ele não distingue.
        //
        // ⚠️ **O mecanismo, e é ele que vale mais que o número:** o `away` **não é
        // núcleo ocioso**. As duas metades já são row-parallel (ADR-0109 no
        // composite, ADR-0145/0147 no solver) e saturam os 32 núcleos, então
        // sobrepô-las não cria capacidade — só move a contenção para DENTRO do
        // frame. *Um balde de espera só é oportunidade quando o recurso que ele
        // espera está parado.*
        #[cfg(test)]
        let t_pixel = std::time::Instant::now();
        // Straight-alpha OVER the frozen base, cell (cx,cy) → pixel (cx-1,cy-1).
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let alock_on = gate_alock.is_some();
        // O VIDRO de uma aguada por baixo (doc 48): o fluido cobre-a por `pa` e cada canal atenua-se
        // ([`crate::compositor::vidro::cobre`]), a partir do vidro da base congelada da sessão. Só numa
        // camada que JÁ tem vidro: sem aguada o alfa único é exacto, e o plano não nasce aqui.
        let camada = camada.filter(|c| self.vidros.contains_key(c));
        let vidro = camada.and_then(|c| {
            super::watercolor_vidro::abre_o_vidro(
                &mut self.vidros,
                &mut self.vidro_congelado.molhada,
                &mut self.vidro_congelado.aguada,
                c,
                self.source_size,
                &sess.base,
                &self.undo.write_state,
                Some(Region {
                    x: px0 as u32,
                    y: py0 as u32,
                    w: (px1 - px0) as u32,
                    h: (py1 - py0) as u32,
                }),
            )
            .map(|(p, b)| (c, p, b))
        });
        let (vcamada, mut vplano, vbase) = match vidro {
            Some((c, p, b)) => (Some(c), Some(p), Some(b)),
            None => (None, None, None),
        };
        let vbase = vbase.as_ref();
        let vlinhas: Vec<Option<&mut [crate::compositor::vidro::Vidro]>> = match vplano.as_mut() {
            Some(p) => Arc::get_mut(p).expect("o plano é só deste quadro")[py0 * w..py1 * w]
                .chunks_mut(w)
                .map(Some)
                .collect(),
            None => (py0..py1).map(|_| None).collect(),
        };
        // A escrita é da própria sessão: a porta do canvas não a retém (reter copiaria a tela a cada composição).
        self.undo.write_state.vigia_a_tela(None);
        // O composite e o véu só escrevem `px0..px1 × py0..py1` (a mesma caixa que o `declare_wrote` abaixo
        // publica): o desfazer guarda os tiles dela, não a tela inteira (33 MB a 4K por traço).
        let caixa_da_escrita = crate::tool::paint::plane_fork::CAPTURA_PELA_CAIXA
            .get()
            .then_some(Region {
                x: px0 as u32,
                y: py0 as u32,
                w: (px1 - px0) as u32,
                h: (py1 - py0) as u32,
            });
        let canvas = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            caixa_da_escrita,
        );
        // ROW-PARALLEL, pelo MESMO ADR-0109 e pela mesma razão do render acima: um map puro por-pixel
        // cujas linhas são independentes — cada uma lê `pigment`/`base` no seu próprio offset e escreve
        // só o seu, e nenhuma lê a saída de outra. **Byte-idêntico por construção**: muda qual thread
        // avalia qual linha, nunca o que a linha diz.
        //
        // ⚠️ **Ele era o que sobrava serial no tick**, e o tick é O(ÁREA MOLHADA) — medido a 4096²,
        // traço de 3600 px: **sim 10,06 ms + composite 6,11**, com 3% da tela molhada. Um frame de
        // 60 fps inteiro. O `WET_MAX_STEPS` fechou o LAÇO (a contagem); isto ataca o custo por passo.
        //
        // A metade da SIM continua serial e assim fica: o engine é port 1:1 com fingerprint pinado e o
        // ADR-0134 declara o solver serial POR SEMÂNTICA — o ADR-0109 é inaplicável lá, de propósito.
        let (pigment, base, sombra) = (&sess.planos.pigment, &sess.base, &sess.planos.sombra);
        let congelada = sess.congelada.as_deref().filter(|_| !veil);
        let grao_cel = &sess.planos.grao;
        let plano_suave: &[u8] = &sess.planos.suave;
        let rr = usize::from(ratio);
        let modo = sess.blend;
        let base_da_marca = marca.map(|_| Arc::clone(&sess.base));
        let cw = px1 - px0;
        let mut cob: Vec<u16> = if marca.is_some() {
            vec![0; cw * (py1 - py0)]
        } else {
            Vec::new()
        };
        let cob_linhas: Vec<Option<&mut [u16]>> = if marca.is_some() {
            cob.chunks_mut(cw).map(Some).collect()
        } else {
            (py0..py1).map(|_| None).collect()
        };
        let vazio = sess.vazio; // L7: o modo de cor sobre o vazio pousa a cor (W3C geral)
        // O amostrador do upsample. ⚠️ A pergunta *"um pixel É uma célula?"* é
        // feita UMA vez, fora do laço: na razão 1 o corpo abaixo lê os quatro
        // bytes no MESMO offset que sempre leu, com a MESMA aritmética de
        // `f32`, então aquele caminho é byte-idêntico ao que shipava — não por
        // sorte numérica, por ser literalmente a mesma expressão.
        let sampler = grid_map::SampleU::new(ratio, gw, gh);
        let orla = oleo.is_none();
        if let Some(rgba) = pronto {
            // B11.3: o compositor já correu o laço (sem vidro, sem marca): o fio da tela só copia.
            super::offthread::otica::pixel::cola_na_tela(canvas, w, (px0, py0, px1, py1), rgba);
        } else {
            #[cfg(test)]
            let orla = orla || super::smooth_edges::so_na_cor::get();
            #[cfg(test)]
            let plano_suave: &[u8] = if super::smooth_edges::sem_suave::get() {
                &[]
            } else {
                plano_suave
            };
            let suavizada = super::smooth_edges::Suaves::nova(
                sampler,
                (pigment, stride),
                (plano_suave, gw, gh),
                (w, h),
                (px0, py0, px1, py1),
                rr,
                linhas,
                orla,
            );
            #[cfg(test)]
            let suavizada = match super::smooth_edges::raio_escalar::get() {
                Some(r) => Some(super::smooth_edges::Suaves::escalar(
                    super::smooth_edges::Suave::nova(
                        sampler,
                        (pigment, stride),
                        (w, h),
                        (px0, py0, px1, py1),
                        r,
                        linhas,
                        orla,
                    ),
                    (w, h),
                )),
                None => suavizada,
            };
            let laco = pixel::LacoDePixel {
                w,
                caixa_px: (px0, py0, px1, py1),
                grade: (gw, gh),
                rr,
                sampler,
                pigment,
                base,
                sombra,
                grao_cel,
                congelada,
                suavizada: suavizada.as_ref(),
                sem_oleo: &sem_oleo,
                orla,
                grao: grao.as_ref(),
                gsel,
                gprot,
                alock_on,
                modo,
                vazio,
                exata,
                vbase,
            };
            pixel::laco_de_pixel(
                &laco,
                &mut canvas[py0 * stride..py1 * stride],
                vlinhas,
                cob_linhas,
            );
        }
        if let (Some(c), Some(p)) = (vcamada, vplano) {
            self.vidros.insert(c, p);
        }
        #[cfg(test)]
        split::note_pixel(t_pixel.elapsed());
        if veil {
            let g = match veu {
                super::veu::VeuDe::Motor => {
                    super::veu::FilmeEMolha::da_grade(sess.engine.active_grid())
                }
                #[cfg(test)]
                super::veu::VeuDe::Grade(g) => super::veu::FilmeEMolha::da_grade(g),
                super::veu::VeuDe::DaImagem(l) => l,
            };
            let caixa = (px0, py0, px1, py1);
            super::veu::pinta_o_veu(&g, canvas, (w, usize::from(ratio)), (gw, gh), caixa);
        }
        // Re-arm the guard with the allocation our own make_mut may have
        // re-seated (weak — [`WetSession::canvas`]).
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        self.undo.write_state.vigia_a_tela(Some(&self.canvas_rgba));
        // A região é a de PIXELS que este composite de fato escreveu (o dirty
        // do motor é em células; publicar aquele retângulo verbatim sob razão
        // > 1 declararia sujo `ratio²` menos pixels do que mudaram).
        let region = Region {
            x: px0 as u32,
            y: py0 as u32,
            w: (px1 - px0) as u32,
            h: (py1 - py0) as u32,
        };
        // ⭐ **E a mesma região DECLARA a escrita** (`crate::undo::window`): o `fork_canvas` acima abriu
        // um acesso, e as duas passadas que escrevem — o composite e o véu — só tocam `px0..px1 ×
        // py0..py1`. Sem esta linha o acesso ficava aberto e o histórico varria a tela inteira no
        // commit do traço E na absorção do escorrido (medido 2026-09-24: `16–26` acessos por traço
        // sem declaração, todos daqui). Em DEBUG o `split` confere que a janela verdadeira cabe nesta.
        self.declare_wrote(Some(region));
        self.mark_dirty(region);
        #[cfg(test)]
        super::fps_j13_tests::gancho(self, caixa_cel, (px0, py0, px1, py1));
        if let Some(antes) = agua_antes {
            self.undo
                .marca
                .agua_mudou(&antes, &self.canvas_rgba, w, (px0, py0, px1, py1));
        }
        if let Some(b) = base_da_marca {
            let caixa = (px0 as u32, py0 as u32, px1 as u32, py1 as u32);
            self.undo.marca.agua_cobre(marca, caixa, &cob, &b);
        }
    }
}

/// **O SPLIT DO COMPOSITE, medido no código que SHIPA** (só sob `cfg(test)`).
///
/// A pergunta que ele responde é de PRODUTO, não de curiosidade: o tick segura o
/// motor durante o composite inteiro e só o devolve no `hand_off_sim` do fim, então
/// **a metade que não precisa do motor é tempo em que o worker fica parado à toa** —
/// o balde `away` do log do Enio (24 % da janela dele, 2026-08-01).
///
/// ⚠️ **Isto é instrumentação do PRODUTO, não uma sonda com laço próprio** — a
/// distinção que a §5.11 e a §5.44 pagaram: um laço meu ficaria cego à porta, e
/// atribuir a uma LINHA dentro de um bloco é inferência de segunda ordem. Aqui os
/// dois blocos já são separados pela fronteira do motor, e o relógio só diz de que
/// lado dela o tempo caiu.
#[cfg(test)]
pub(in crate::tool::paint) mod split;

/// O laço de píxel como função pura das entradas (B11.3).
pub(in crate::tool::paint::wetpaint) mod pixel;

/// A entrada da óptica tirada da sessão com o motor EM CASA (o sujo, os `Params`, a marca): a do composite e a do
/// operário do depósito (N10 F2h, [`super::dab_route`]'s `fila_do_dab`), a mesma lei nos dois fios.
pub(super) fn entrada_da_sessao(
    sess: &mut WetSession,
    factos: super::otica_factos::FactosDaOtica,
    (w, h): (usize, usize),
) -> super::composite_otica::EntradaDaOtica {
    // ⚠️ **Células e pixels deixaram de ser a mesma coisa** ([`grid_map`]):
    // a grade tem `ratio` pixels de canvas por célula. O dirty do motor é
    // em CÉLULAS e é clampado à GRADE; o retângulo que este composite
    // escreve — e que o `mark_dirty` publica — é em PIXELS.
    let ratio = sess.ratio;
    let (gw, gh) = sess.grid;
    // A tela fica a mostrar a água NESTE estado (W14: onde ela pára se a sessão acabar já).
    let marca_da_agua = fita::MarcaDaAgua::de(&sess.engine);
    let caixa = match sess.engine.take_dirty() {
        Dirty::Clean => None,
        Dirty::Full => Some((1, 1, gw, gh)),
        Dirty::Rect { x0, y0, x1, y1 } => Some((
            (x0.max(1) as usize).min(gw),
            (y0.max(1) as usize).min(gh),
            (x1.max(1) as usize).min(gw),
            (y1.max(1) as usize).min(gh),
        )),
    };
    let caixa = caixa.filter(|_| sess.base.len() == w * h * 4);
    super::composite_otica::EntradaDaOtica {
        factos,
        marca: marca_da_agua,
        trabalho: caixa.map(|caixa| super::composite_otica::TrabalhoDaOtica {
            caixa,
            params: sess.engine.sim.gather_params(&sess.engine.tuning),
        }),
        ratio,
        grade: (gw, gh),
        tela: (w, h),
        revisao: sess.engine.revisao(),
    }
}
