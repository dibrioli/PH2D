//! **O SHARPEN** — a única lei da W9c que não existia, e o primeiro filtro
//! deste módulo com **PRÉ-PASSE**.
//!
//! Os outros oito filtros respondem por vértice: dado o `pre` daquele ponto e o
//! seu anel, qual o alvo? Este não consegue — o deslocamento de um vértice é
//! pesado pela **curvatura dos VIZINHOS**, então nenhum deles pode ser escrito
//! antes de todos serem medidos. É por isso que ele bifurca do laço genérico do
//! [`super::mesh_filter`] em vez de entrar no `match` dele: acrescentar um
//! `match` que às vezes precisa de um passe anterior é como o próximo caso
//! especial nasce escondido.
//!
//! # A lei, do `calc_sharpen_filter` (`sculpt_filter_mesh.cc:1590-1712`)
//!
//! **O pré-passe**, uma vez por sub-passo:
//!
//! 1. `d[i] = média_do_anel(i) − p[i]` — o deslocamento laplaciano, que é a
//!    mesma grandeza que o [`crate::FilterKind::EnhanceDetails`] usa inteira
//!    (`detail_directions` no fonte);
//! 2. `f[i] = |d[i]|`, normalizado pelo **MAIOR** da malha (`safe_rcp(max)`);
//! 3. `f[i] = 1 − (1 − f[i])²` — a curva que empurra o meio da faixa para cima.
//!
//! **O passe**, por vértice:
//!
//! ```text
//! gather = Σ_vizinhos (p[n] − p[i]) · f[n]   ×  (1 − f[i])
//! média  = d[i] · SMOOTH_RATIO · f[i]²
//! p'[i]  = p[i] + (gather + média) · w
//! ```
//!
//! ⚠️ **O que a lei FAZ não se lê do nome, e é o que a torna um afiador:** os
//! dois termos são governados por `f[i]` em direções OPOSTAS. Onde há detalhe
//! (`f[i] → 1`) o termo do gather **desaparece** e o vértice é puxado para a
//! média do próprio anel — ele **ALISA**. Onde não há (`f[i] → 0`) a média
//! desaparece e o vértice é puxado pelos vizinhos que TÊM detalhe. *Os picos
//! são achatados e o terreno em volta é arrastado até eles*: o que muda não é a
//! posição de uma feição, é o **CONTRASTE** entre ela e a vizinhança — que é o
//! que afiar significa e o que nenhum dos outros oito faz.
//!
//! ⚠️ **Os dois termos que os defaults desligam ficam FORA, e não é omissão.**
//! O `sharpen_intensify_detail_strength` nasce em `0` e o
//! `sharpen_curvature_smooth_iterations` em `0` (`sculpt_filter_mesh.cc:2716` e
//! `:2727`), então o terceiro termo (`detail_directions × −intensify`) e as N
//! passadas de alisamento do pré-passe **não correm no produto da referência**.
//! Escrevê-los seria construir dois controles que ninguém pediu para um kernel
//! cuja lei ainda não foi julgada num smoke; eles entram no dia em que o Enio
//! os pedir, e o fonte está citado aqui para isso custar uma leitura.
//!
//! # ⚠️ A DIVERGÊNCIA, e por que ela não é a do Smooth
//!
//! O cabeçalho do [`super::mesh_filter`] declara que para o `Smooth` e o
//! `Relax` a referência **REPLAYA** a cadeia de eventos do arrasto enquanto nós
//! aplicamos um passo a partir do `pre`, e nomeia o preço: *o nosso Smooth
//! satura*. O `Sharpen` é o **terceiro membro da mesma família** — ele está no
//! `sculpt_mesh_filter_is_continuous` (`:315`) ao lado dos dois —, mas aqui a
//! iteração **não é opcional**, e a referência diz por quê num comentário ao
//! lado do próprio clamp:
//!
//! > *"This filter can't work at full strength as it needs multiple iterations
//! > to reach a stable state."* — `sculpt_filter_mesh.cc:1661`
//!
//! ⇒ `clamp_factors(factors, 0.0f, 0.5f)`. Um passo de força alta desta lei não
//! é *"menos afiado"*, é **outra coisa**: o gather é uma soma sobre o anel, e
//! sem teto ele ultrapassa a vizinhança e inverte a feição.
//!
//! ⚠️ **E a lei da referência depende da TAXA DE POLLING.** Ela não restaura a
//! pose entre eventos (é isso que `is_continuous` significa), então o número de
//! iterações que um arrasto produz é o número de eventos de rato que o sistema
//! operativo entregou — *o mesmo arrasto, na mesma máquina, com o rato a 125 Hz
//! e a 1000 Hz, afia quantidades diferentes*. É a lei que esta casa recusou
//! seis vezes (o relevo do Painter, a mordida do arado, o traço do Flip): **o
//! resultado é fato do GESTO, nunca de quão fino o motor o amostrou.**
//!
//! **A nossa forma honra as duas metades.** O arrasto pede uma força total, e
//! ela é entregue em `n = ceil(total / MAX_STEP)` sub-passos **determinísticos**
//! de `total/n` cada — nenhum acima do teto que a referência declara, e o
//! número deles é função pura do arrasto. Consequências, cada uma com gate:
//!
//! * até `MAX_STEP` é **UM** passo, ou seja a referência com um evento, ao bit;
//! * o `restore_frozen_pose` segue a ser a primeira coisa que um filtro faz, e
//!   por isso **o arrasto de volta continua exacto** e a idempotência dos
//!   outros oito não é tocada;
//! * e ela é a prescrição que o cabeçalho do irmão já escrevia — *"um knob de
//!   ITERAÇÕES com um número medido ao lado, nunca o histórico"* — com o knob a
//!   ser o próprio arrasto em vez de um segundo controle.

use super::*;

/// **Quanta força uma iteração pode carregar** — o `clamp_factors(factors, 0,
/// 0.5)` da referência (`sculpt_filter_mesh.cc:1662`), verbatim.
///
/// ⚠️ **É um limite de ESTABILIDADE do kernel e não uma faixa de gosto**, e é a
/// única razão pela qual o fatiamento existe: acima dele o gather passa da
/// vizinhança e a feição inverte. O comentário que o acompanha no fonte é a
/// justificação inteira, e está citado no cabeçalho deste módulo.
pub(super) const MAX_STEP: f32 = 0.5;

/// **Quanto da média do próprio anel entra onde há detalhe** — o
/// `sharpen_smooth_ratio` da referência, no valor de fábrica dela
/// (`sculpt_filter_mesh.cc:2707`).
///
/// ⚠️ **Constante e não param, pelo mesmo motivo dos outros dois** — ver o
/// cabeçalho: ele nasce como um número da referência, e vira controle no dia em
/// que um smoke pedir, com o custo de uma row.
const SMOOTH_RATIO: f32 = 0.35;

impl SculptStroke {
    /// **O SHARPEN, com o arrasto fatiado em iterações estáveis.**
    ///
    /// Devolve quantos vértices foram escritos, como o irmão genérico.
    ///
    /// ⚠️ **O `restore_frozen_pose` corre UMA vez, antes do primeiro
    /// sub-passo** — é o que faz do arrasto inteiro uma operação a partir do
    /// pen-down (a metade da lei que o doc dele nomeia). Restaurar entre
    /// sub-passos apagaria a cadeia, que é precisamente o que este filtro
    /// existe para ter.
    pub(super) fn filter_sharpen(&mut self, mesh: &mut Mesh, brush: &Brush, amount: f32) -> usize {
        let (lo, hi) = crate::FilterKind::Sharpen.range();
        let total = amount.clamp(lo, hi);
        self.restore_frozen_pose(mesh);

        // ⚠️ **A lista de escritos é a de TOCADOS e é montada uma vez:** um
        // filtro cobre a malha (o `filter` já o exigiu), e o `apply_positions`
        // percorre o `moved` a cada sub-passo. Reconstruí-la por sub-passo daria
        // a mesma lista pelo preço de `n` varreduras.
        self.moved.clear();
        self.moved.extend_from_slice(&self.touched);

        // ⚠️ **Força zero escreve a POSE CONGELADA e não sai cedo**, e a
        // distinção é o arrasto de volta: sair aqui deixaria a malha onde o
        // sub-passo anterior a pôs, e o gesto deixaria de ser reversível. Com o
        // `restore` acima já feito, o caminho certo é simplesmente não iterar.
        if total > 0.0 {
            let n = steps_for(total);
            let per = total / n as f32;
            for _ in 0..n {
                self.sharpen_step(mesh, brush, per);
            }
        }

        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// **UM sub-passo:** o pré-passe sobre a pose VIVA, depois o gather.
    fn sharpen_step(&mut self, mesh: &mut Mesh, brush: &Brush, per: f32) {
        self.sharpen_prepass(mesh, brush);

        let pos = mesh.positions();
        for i in 0..self.touched.len() {
            let v = self.touched[i] as usize;
            let p = pos[v];
            let fi = self.sharp_f[i];

            // O gather sobre o anel, pesado pela curvatura de CADA VIZINHO —
            // é esta leitura que obriga o pré-passe a existir.
            let mut acc = [0.0f32; 3];
            for &nb in mesh.adjacency().vert_verts.neighbours(v) {
                let fs = self.sharp_f[self.slot[nb as usize] as usize];
                let q = pos[nb as usize];
                for k in 0..3 {
                    acc[k] += (q[k] - p[k]) * fs;
                }
            }
            let hold = 1.0 - fi;
            let blend = SMOOTH_RATIO * fi * fi;
            let d = self.sharp_d[i];
            for k in 0..3 {
                acc[k] = acc[k] * hold + d[k] * blend;
            }

            // ⚠️ **A ORDEM é a da referência** (`scale_factors` e só então
            // `clamp_factors`), a mesma do laço genérico: o fator parte da
            // máscara, é escalado, e **então** é aparado. Clampar antes daria a
            // um vértice meio-mascarado mais que a um livre no extremo.
            let w = (per * crate::mask_ops::free_weight(self.base_mask[i])).clamp(0.0, MAX_STEP);
            let s = self.slot[v] as usize;
            self.target[s] = [p[0] + acc[0] * w, p[1] + acc[1] * w, p[2] + acc[2] * w];
        }

        // ⚠️ **`coat = true` escreve o alvo CRU, e é a porta certa:** um
        // sub-passo encadeado parte da pose VIVA, não do `base`, então o
        // `toward(base, alvo, accum)` do outro ramo responderia à pergunta
        // errada — e a porta já existe, com consumidor (a demão da W8), em vez
        // de uma segunda escrita nascer aqui.
        self.apply_positions(mesh, true);
    }

    /// **O PRÉ-PASSE** — mede o detalhe de cada vértice e normaliza pelo maior.
    ///
    /// ⚠️ **A normalização é pelo MÁXIMO DA MALHA, não por um número fixo**, e
    /// é o que torna a lei independente de escala: dobrar o tamanho do objeto
    /// dobra todo `|d|` e deixa todo `f` onde estava. Um limiar absoluto faria
    /// o mesmo modelo afiar diferente conforme a unidade em que foi importado.
    ///
    /// ⚠️ **A guarda de `max = 0` é REAL mas quase inalcançável, e a minha
    /// primeira afirmação sobre ela estava ERRADA.** Eu escrevi que *"uma malha
    /// lisa não move um vértice"* e a sonda mediu **`2,8e-2` de excursão** numa
    /// esfera UV sem ruído nenhum: os anéis perto dos polos são mais densos que
    /// os do equador, então o laplaciano de uma esfera UV **não é zero** — ele
    /// é pequeno e VARIÁVEL, e o pré-passe normaliza pelo maior, o que traz
    /// todo `f` de volta para a faixa cheia. *Liso ao olho não é liso ao
    /// laplaciano.* A guarda vale para o caso exacto (`max` abaixo do épsilon,
    /// que uma malha construída à mão pode ter) e devolve zero, que já é a
    /// resposta certa; o que ela **não** é, é a explicação de um produto calmo.
    fn sharpen_prepass(&mut self, mesh: &Mesh, brush: &Brush) {
        let n = self.touched.len();
        self.sharp_f.clear();
        self.sharp_f.resize(n, 0.0);
        self.sharp_d.clear();
        self.sharp_d.resize(n, [0.0; 3]);

        let mut max = 0.0f32;
        for i in 0..n {
            let v = self.touched[i];
            let p = mesh.positions()[v as usize];
            let avg = self.neighbour_average(mesh, brush, v, p);
            let d = [avg[0] - p[0], avg[1] - p[1], avg[2] - p[2]];
            self.sharp_d[i] = d;
            let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            self.sharp_f[i] = mag;
            if mag > max {
                max = mag;
            }
        }

        let inv = if max > f32::EPSILON { 1.0 / max } else { 0.0 };
        for f in &mut self.sharp_f {
            let t = *f * inv;
            // `1 − (1 − t)²`, a curva da referência (`:2085`).
            let u = 1.0 - t;
            *f = 1.0 - u * u;
        }
    }
}

/// **Quantos sub-passos uma força total pede** — o menor `n` cujo passo cabe no
/// [`MAX_STEP`].
///
/// ⚠️ **Determinístico, e é a metade que separa a nossa lei da da referência:**
/// ele é função pura da força pedida, então o mesmo arrasto afia a mesma coisa
/// em qualquer taxa de eventos e em qualquer máquina.
pub(super) fn steps_for(total: f32) -> u32 {
    (total / MAX_STEP).ceil().max(1.0) as u32
}
