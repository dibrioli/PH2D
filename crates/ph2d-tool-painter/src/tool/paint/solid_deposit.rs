//! **O depósito de uma forma SÓLIDA** — a metade de tool do `Style: Solid` (plano 38 §1.1, W7).
//!
//! O motor (`ph2d_painter_brush::solid`) responde *que cobertura este caminho fechado tem*; aqui
//! decide-se *o que fazer com ela*: onde escrever, com que cor, e — a parte que importa — **pela
//! mesma porta de proteção que os dabs atravessam** (`gate_scoped`).
//!
//! ⚠️ **O caminho é acumulado pelo TOOL, não pela lista de dabs**, e a razão é medível: a lista de
//! dabs de um evento é o pedaço do gesto desde o último ponteiro, não o gesto — preencher o polígono
//! dela daria a mancha de um fragmento. O que uma forma sólida cerca é o percurso INTEIRO.
//!
//! # ⚠️ O Solid deixou de ESCONDER o traço (W7, ordem do Enio 2026-08-15)
//!
//! Até aqui `Solid` era *a região **EM VEZ** do traço*: os dabs eram suprimidos e a §1.1 do plano
//! registrava a decisão de então — *"para forma sólida a espessura da linha passa a não ser
//! considerada"*. O pedido novo é o oposto e é do mesmo autor: ***"Solid deve usar o pincel com o
//! falloff e espessura do traço como no modo flip"*.**
//!
//! No Flip um traço tem `fill: Option<Fill>` e **continua a ter a largura e a dureza dele** — o
//! preenchimento é a região, e o contorno é o pincel. É esse o modelo agora: a mancha é a região
//! CERCADA, e o traço é carimbado como em qualquer outro gesto. Isso entrega as três metades do
//! pedido de uma vez, porque as três eram a mesma:
//!
//!   1. o **falloff e a espessura** voltam — eles são o pincel, e o pincel voltou a carimbar;
//!   2. **todo tipo de linha** passa a funcionar sob Solid (Speed, Sketchy, Wire, Ribbon, Rough
//!      DECORAM o traço, e não havia traço para decorar);
//!   3. **Symmetry e Tiling** alcançam o traço **de graça** — eles moram na porta do dab —, e esta
//!      wave só teve de os levar ao PREENCHIMENTO e aos FIOS.
//!
//! # ⚠️ A ordem entre a mancha e o traço NÃO é observável, e é por isso que as duas famílias podem
//! usar transações diferentes
//!
//! As duas tintas são a **mesma cor**, e o `over` de duas fontes da mesma cor é **comutativo** em
//! cor e em alfa: `a₁ ⊕ a₂ = a₁ + a₂ − a₁a₂` é simétrico, e a cor sai `c` nos dois casos. Logo
//! *fill-por-baixo* e *fill-por-cima* dão a MESMA imagem (ao arredondamento de `u8`), e cada família
//! de depósito pode usar a transação que já tem:
//!
//! - **métodos de RE-CARIMBO** (Drag Dot / Anchored / Line e os cinco shape editors): o
//!   preenchimento viaja **dentro** do `stamp_drag_preview`, entre o `save` e o carimbo — porque ali
//!   os dabs também são transitórios, e duas transações encadeadas no mesmo slot deixariam só a
//!   última de pé;
//! - **métodos CUMULATIVOS** (mão livre, Airbrush, Dots, Grid Stamp): os dabs são permanentes, então
//!   o preenchimento é uma transação PRÓPRIA, e o `stamp_dabs` a **descasca antes** do lote e a
//!   **re-escreve depois** ([`Self::freehand_solid_fill_live`]). O snapshot tem de conter todo dab e
//!   nenhum fill — se ele envelhecer, o restore do quadro seguinte apaga tinta do artista.

use super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::{Dab, solid};

impl PainterTool {
    /// O gesto está sendo depositado como forma SÓLIDA?
    ///
    /// ⚠️ **Porta única, e ela pergunta ao MODO também** — o Solid escreve pigmento pela rota própria,
    /// então onde o depósito não é pigmento (Sculpt, Smear/Blur/Clone, a máscara, a aquarela, o
    /// fluido) ele não tem o que preencher, e deixá-lo entrar seria o botão fazendo outra coisa em
    /// cada ferramenta. `false` ⇒ tudo é byte-idêntico ao mundo sem esta feature.
    pub(super) fn solid_owns_the_gesture(&self) -> bool {
        self.paint.brush.style_solid
            && matches!(self.paint.paint_mode, super::PaintMode::Paint)
            && !self.paint.eraser
            && !self.paint.brush.watercolor
            && !self.paint.wetpaint.armed
    }

    /// **O preenchimento é uma transação PRÓPRIA neste gesto?** — a testemunha que faz o `stamp_dabs`
    /// bracketar o lote (descascar antes, re-escrever depois).
    ///
    /// ⚠️ Ela pergunta pelo **caminho acumulado** e pelo **método**, nunca por uma lista de sítios: o
    /// ciclo de traço carimba dabs em **seis** lugares (pen-down, move, o tick do airbrush, o settle
    /// do estabilizador, o finish e o pen-up), e bracketar os que eu lembrei é como o sétimo nasce
    /// apagando a tinta do artista — o restore de um snapshot velho não *deixa de desenhar*, ele
    /// **desfaz**. O caminho só é semeado pelo `paint_begin`, e os cinco shape editors nem passam
    /// por ele (`canvas_pointer` os desvia antes), então só um gesto cumulativo cai aqui.
    ///
    /// ⚠️ **`is_incremental` é o que separa as duas transações da §doc do módulo:** Drag Dot,
    /// Anchored e Line também são gestos de ponteiro, mas os dabs deles são transitórios e o
    /// preenchimento viaja dentro do `stamp_drag_preview`.
    ///
    /// ⚠️ **Ela NÃO pergunta se o caminho já tem pontos, e não pode**: desde a W8 quem alimenta o
    /// caminho é esta mesma porta ([`Self::note_ink_path`]), então exigir um caminho não-vazio para
    /// entrar seria a condição que **impede o primeiro ponto de ser gravado** — o gesto inteiro
    /// ficaria sem mancha, com todos os gates de unidade verdes.
    pub(super) fn freehand_solid_fill_live(&self) -> bool {
        self.paint.brush.stroke_method.is_incremental() && self.solid_owns_the_gesture()
    }

    /// **Grava onde a TINTA deste lote caiu** — a raia primária, que é o caminho que a mancha cerca.
    ///
    /// # Por que a tinta, e não o ponteiro (W8, report do Enio 2026-08-15)
    ///
    /// Até aqui o caminho era `ev.pos`, o percurso da MÃO. Mas metade dos tipos de linha **move a
    /// tinta para longe da mão** — é literalmente a razão de existirem —, então a mancha e o traço
    /// descreviam curvas diferentes:
    ///
    /// - **Speed** arremessa a tinta à frente (`v · T`, centenas de px num gesto rápido) ⇒ a mancha
    ///   ficava para trás e MENOR que o contorno, que é a foto do report;
    /// - **Ribbon** deixa a tinta atrás do dedo ⇒ o mesmo, ao contrário;
    /// - **Rough** move o CAMINHO, e o desvio dele é pequeno perto da espessura ⇒ o traço cobria a
    ///   discordância, que é por que ele foi o único aprovado (*"Rough com Solid funcionou"*).
    ///
    /// A lei nova é a do Alchemy, onde todo gesto é uma forma preenchida e o efeito mora no CAMINHO:
    /// ***a mancha é o polígono da tinta***. Um tipo que joga a tinta para o lado passa a jogar a
    /// FRONTEIRA junto, que é o que faz as pontas do Speed virarem espinhos da forma em vez de um
    /// contorno solto ao lado dela.
    ///
    /// ⚠️ **A raia primária sai por passo de [`ph2d_painter_brush::SymmetrySettings::copies`]** — o
    /// lote que chega ao carimbo já traz as cópias interleavadas, e a mancha aplica a simetria por
    /// conta própria (`symmetric_loops`). Gravar o lote inteiro daria um polígono em ziguezague
    /// ATRAVÉS das raias; gravar e não espelhar deixaria a mancha numa raia só.
    pub(super) fn note_ink_path(&mut self, dabs: &[Dab]) {
        let step = self.paint.brush.symmetry.copies().max(1);
        for d in dabs.iter().step_by(step) {
            self.paint.solid_path.push(d.center);
        }
    }

    /// **A PORTA ÚNICA de *"que laços este gesto preenche AGORA?"*** — já replicados pela Symmetry e
    /// pelo Tiling, prontos para uma única passada de `fill_coverage`.
    ///
    /// A cena de FORMAS responde primeiro (um shape editor autora a geometria diretamente,
    /// `super::solid_shapes`); na ausência dela, o caminho acumulado do gesto à mão livre.
    ///
    /// ⚠️ **Uma passada só sobre o conjunto INTEIRO, e não é economia — é CORREÇÃO:** o
    /// preenchimento é `nonzero` sobre o conjunto, então formas que se sobrepõem (as cópias de
    /// simetria inclusive) fundem sem costura. Preenchê-las uma a uma comporia a borda anti-aliased
    /// **duas vezes** e deixaria uma linha escura exatamente onde elas se tocam.
    pub(super) fn solid_fill_loops(&self) -> Vec<Vec<[f32; 2]>> {
        if !self.solid_owns_the_gesture() {
            return Vec::new();
        }
        let mut loops = self.solid_loops();
        if loops.is_empty() && self.paint.solid_path.len() >= 2 {
            loops.push(self.paint.solid_path.clone());
        }
        if loops.is_empty() {
            return loops;
        }
        // ⚠️ A ordem é SYMMETRY e depois TILING, a mesma que a lista de dabs percorre: o motor
        // espelha na emissão (`push_symmetric`) e o `stamp_dabs_routed` envolve as cópias na costura
        // depois. Invertida, uma cópia espelhada que caísse fora da tela não seria envolvida, e a
        // mancha e o traço passariam a discordar sobre onde a tile continua.
        let mirrored =
            ph2d_painter_brush::symmetry::symmetric_loops(&loops, &self.paint.brush.symmetry);
        if self.paint.tiling[0] || self.paint.tiling[1] {
            super::tiling::tiled_loops(&mirrored, self.source_size, self.paint.tiling)
        } else {
            mirrored
        }
    }

    /// O retângulo do canvas que este preenchimento escreve, ou `None` se ele não toca a tela.
    pub(super) fn solid_fill_rect(&self, loops: &[Vec<[f32; 2]>]) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || loops.is_empty() {
            return None;
        }
        let [bx, by, bw, bh] = solid::loops_bbox(loops, w as usize, h as usize)?;
        #[allow(clippy::cast_possible_truncation)]
        Some(Region {
            x: bx as u32,
            y: by as u32,
            w: bw as u32,
            h: bh as u32,
        })
    }

    /// **A transação do preenchimento SOZINHO** — restaurar → medir → salvar → escrever, para os
    /// métodos CUMULATIVOS (a §doc do módulo diz por que só eles).
    ///
    /// Chamada pelo `stamp_dabs` **depois** do lote, com o preview já descascado: o snapshot que ela
    /// guarda contém todo dab do gesto e nenhum fill, que é o invariante inteiro.
    pub(super) fn stamp_solid_preview(&mut self) {
        let loops = self.solid_fill_loops();
        let chord = self.closing_chord_dabs();
        self.stamp_solid_loops_with_chord(&loops, &chord);
    }

    /// **A CORDA que fecha o laço, carimbada com o PINCEL** — do último ponto de tinta de volta ao
    /// primeiro (report do Enio 2026-08-15: *"a linha reta da área não fechada de Solid deve levar o
    /// falloff também"*).
    ///
    /// Um gesto aberto vira uma forma fechada porque o preenchimento fecha o laço implicitamente
    /// (`solid::fill_coverage`, a decisão §5.3 do plano 38). Só que o pincel caminhava **o caminho**,
    /// nunca a corda — então a fronteira inteira era macia e essa aresta, e só ela, era o corte duro
    /// do rasterizador. Agora ela é tinta como o resto.
    ///
    /// ⚠️ **Os dabs saem do PINCEL, não do último dab do lote**, e é decisão: a corda é um trecho que
    /// a mão nunca percorreu, então não há pressão, nem jitter, nem heading dela para herdar — o que
    /// existe é a espessura e a dureza que o artista escolheu. Herdá-los do último dab faria a corda
    /// afinar quando o gesto acabasse leve, o que é o taper de uma ponta que não existe.
    ///
    /// ⚠️ **As cópias saem da porta do MOTOR** (`push_symmetric`): a corda tem de cair nas mesmas
    /// raias que os dabs do gesto, e o tiling vem de graça porque o dispatch do carimbo já o envolve.
    fn closing_chord_dabs(&self) -> Vec<Dab> {
        let path = &self.paint.solid_path;
        if path.len() < 3 || !self.solid_owns_the_gesture() {
            return Vec::new();
        }
        let (a, b) = (path[path.len() - 1], path[0]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let r = self.paint.brush.clamped_radius();
        // Um vão menor que um passo já está coberto pelos dabs das duas pontas.
        let step = (2.0 * r * self.paint.brush.spacing).max(1.0);
        if len <= step {
            return Vec::new();
        }
        let dir = [d[0] / len, d[1] / len];
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (len / step).floor() as usize;
        let mut out = Vec::with_capacity(n * self.paint.brush.symmetry.copies());
        let template = Dab {
            center: a,
            radius_px: r,
            coverage: self.paint.brush.strength.clamp(0.0, 1.0),
            color: self.paint.brush.color,
            rotation: [1.0, 0.0],
            dir,
            arc_len: 0.0,
            stroke_radius_px: r,
        };
        for k in 0..=n {
            #[allow(clippy::cast_precision_loss)]
            let t = k as f32 * step;
            let mut dab = template;
            dab.center = [a[0] + dir[0] * t, a[1] + dir[1] * t];
            dab.arc_len = t;
            ph2d_painter_brush::symmetry::push_symmetric(&mut out, dab, &self.paint.brush.symmetry);
        }
        out
    }

    /// Preenche os laços dados **e a corda que os fecha**, restaurando o preview do quadro anterior —
    /// a MESMA dança do `stamp_drag_preview` (restaurar → medir → salvar → escrever), porque um Solid
    /// é um re-carimbo por construção: a cada ponto novo o polígono INTEIRO muda de forma.
    ///
    /// ⚠️ **A corda entra AQUI e não pela porta cumulativa do carimbo**, e é o que a torna correta:
    /// ela muda de lugar a cada ponto novo (liga o fim do caminho ao começo), então tinta cumulativa
    /// deixaria um LEQUE de cordas velhas pelo gesto inteiro. Dentro desta transação ela é restaurada
    /// e re-carimbada com a mancha, que é exactamente o que um re-carimbo é.
    ///
    /// ⚠️ **E o retângulo cresce pela pegada REAL da corda, perguntada às portas do depósito.** A
    /// caixa dos laços contém a corda (as duas pontas são vértices do laço) mas não o que os dabs
    /// dela escrevem — sem a folga, o `save` não cobre a tinta da corda e o restore do quadro
    /// seguinte deixa um rastro.
    ///
    /// ⚠️ **E a folga NÃO pode ser meia-espessura, porque o Tiling tem régua PRÓPRIA** (auditoria do
    /// Enio, 2026-08-15). Um laço é replicado quando a CAIXA dele passa a costura; um dab, quando
    /// `centro ± raio` passa. Um caminho colado à borda tem a caixa DENTRO da tela e dabs de corda
    /// cuja pegada passa dela: a cópia envolvida cai na borda OPOSTA, a um span inteiro de distância,
    /// fora de qualquer folga de raio. Medido, um caminho a 8 px da borda deixava **279 texels** que
    /// o restore nunca alcançava — e o desenho passava a depender da TAXA DE EVENTOS, que é a lei que
    /// este módulo já pagou quatro vezes no relevo. A cura é não ter uma segunda régua: a região sai
    /// de [`Self::tiled_chord_region`], que pergunta às MESMAS portas que o carimbo vai usar.
    pub(super) fn stamp_solid_loops_with_chord(&mut self, loops: &[Vec<[f32; 2]>], chord: &[Dab]) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        let Some(mut rect) = self.solid_fill_rect(loops) else {
            return;
        };
        if let Some(chord_rect) = self.tiled_chord_region(chord) {
            rect = super::union_region(rect, chord_rect);
        }
        if rect.w == 0 || rect.h == 0 {
            return;
        }
        let pixels = self.save_region(&rect);
        self.stamp_solid(loops, rect);
        if !chord.is_empty() {
            self.stamp_dabs_dispatch(chord);
        }
        self.paint.drag_preview = Some(super::DragPreview { rect, pixels });
    }

    /// **A região que a corda vai de facto escrever** — já replicada pelo Tiling, pelas MESMAS portas
    /// que o carimbo usa (`tiling::tiled_dabs` → `dab_batch_region`).
    ///
    /// ⚠️ **Ela não redescobre nada, e é isso que a torna correta:** perguntar *"o que esta corda
    /// pinta?"* com uma régua própria é como a régua do laço e a do dab passam a discordar sobre a
    /// costura — que é exactamente o defeito que a fez nascer.
    pub(super) fn tiled_chord_region(&self, chord: &[Dab]) -> Option<Region> {
        if chord.is_empty() {
            return None;
        }
        if self.paint.tiling[0] || self.paint.tiling[1] {
            let wrapped = super::tiling::tiled_dabs(chord, self.source_size, self.paint.tiling);
            self.dab_batch_region(&wrapped)
        } else {
            self.dab_batch_region(chord)
        }
    }

    /// Escreve a região **pela porta do gate**, exactamente como um lote de dabs: a proteção e a
    /// seleção são um fator por texel aplicado UMA vez sobre a tinta acumulada livre, e é isso que
    /// impede o Solid de ser a segunda semântica de proteção do módulo.
    pub(super) fn stamp_solid(&mut self, loops: &[Vec<[f32; 2]>], rect: Region) {
        let mask_gate = self.mask_protection_active();
        let sel_gate = self.selection_restricts_paint();
        if mask_gate || sel_gate {
            self.gate_scoped(rect, mask_gate, sel_gate, |t| t.write_solid(loops, rect));
        } else {
            self.write_solid(loops, rect);
        }
    }

    /// A escrita crua: `over` da cor do pincel pesada pela cobertura exata.
    fn write_solid(&mut self, loops: &[Vec<[f32; 2]>], rect: Region) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let cov = solid::fill_coverage(loops, rect.w as usize, rect.h as usize, origin);
        let col = self.paint.brush.color;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rgb = [
            (col[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
        ];
        // ⚠️ A opacidade do pincel entra AQUI. A ESPESSURA não entra nesta mancha e nunca entrará:
        // ela é o que o TRAÇO desenha, e desde a W7 o traço é carimbado ao lado dela (§doc do
        // módulo). Fazer a mancha crescer meia-espessura sozinha seria uma SEGUNDA resposta a *"que
        // borda este pincel tem"*, divergindo do falloff no dia em que alguém afinasse um dos dois.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let strength = (self.paint.brush.strength.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
        // ⚠️ A janela DECLARADA é o retângulo que este preenchimento escreve — é ela que faz o commit
        // de undo usar a janela em vez de varrer os planos (doc 28 §5.17), e é por isso que a porta
        // de fork a recebe em vez de a redescobrir.
        let width_px = self.source_size.0;
        let canvas = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            width_px,
            Some(rect),
        );
        for row in 0..rect.h as usize {
            let y = rect.y as usize + row;
            if y >= h {
                break;
            }
            for cx in 0..rect.w as usize {
                let x = rect.x as usize + cx;
                if x >= w {
                    break;
                }
                let a = u32::from(cov[row * rect.w as usize + cx]) * strength / 255;
                if a == 0 {
                    continue;
                }
                let p = (y * w + x) * 4;
                for ch in 0..3 {
                    let d = u32::from(canvas[p + ch]);
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        canvas[p + ch] = ((rgb[ch] * a + d * (255 - a)) / 255) as u8;
                    }
                }
                let da = u32::from(canvas[p + 3]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    canvas[p + 3] = (a + da * (255 - a) / 255) as u8;
                }
            }
        }
        self.declare_wrote(Some(rect));
        self.mark_dirty(rect);
    }
}
