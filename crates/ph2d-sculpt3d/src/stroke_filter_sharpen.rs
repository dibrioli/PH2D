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
//! **O pré-passe**, uma vez por **GESTO** (o `filter_cache` da referência é
//! construído no `sculpt_filter_specific_init`) — ⚠️ **este cabeçalho já disse
//! *"por sub-passo"*, que era a lei ERRADA que a wave existiu para corrigir**;
//! o item 1 é o único VIVO a cada sub-passo:
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

/// **Quanto o realce de detalhe pesa** — o `sharpen_intensify_detail_strength`.
///
/// ⚠️ **O valor é o da referência, e a minha teoria sobre ele foi REFUTADA por
/// medição.** Quando o smoke reprovou o Sharpen (*"parece alisar o mesh"*) eu
/// escrevi que a causa era ter omitido este termo — ele chama-se *"Intensify
/// Details"* na UI do Blender, e os outros dois apontam ambos para a média.
/// Construí-o e varri o valor (`measure_sharpen_intensify.rs`):
///
/// | INTENSIFY | degrau a força 4 |
/// |---|---|
/// | 0,0 (a referência) | 1,046× |
/// | 0,25 | 1,048× |
/// | 1,0 | 1,057× |
/// | 2,0 | 1,069× |
///
/// **Ele compra 2% de degrau a oito vezes o default.** Não é ele que faz a lei
/// afiar, e a hipótese ficou registada aqui em vez de virar um knob que não
/// resolve nada. O termo FICA porque é a lei da referência e o neutro é exacto
/// (`blend − 0,0·f` é `blend` em IEEE-754); o que a wave aprendeu está no
/// cabeçalho, na secção *O QUE ESTA LEI NÃO FAZ*.
const INTENSIFY: f32 = 0.0;

/// **A porta que NÃO passa pelo clamp da faixa** — só para medição.
///
/// ⚠️ **Ela existe porque a tabela que justificava o `SHARPEN_MAX` mediu-se a si
/// mesma.** O [`SculptStroke::filter_sharpen`] clampa a entrada pelo próprio
/// teto ANTES de qualquer aritmética, então toda observação acima dele — de
/// sonda, de gate ou de doc — via o clamp e não a lei: a "saturação em 4,0" que
/// o doc afirmava era o teto a devolver o mesmo número para qualquer entrada
/// maior. Erguer o teto mostra que a lei **não satura** (a excursão quase dobra
/// de 4 para 8 e continua a subir).
///
/// ⚠️ **`doc(hidden)` e não `cfg(test)`:** as sondas desta linha vivem em
/// `tests/`, que é outra crate — sob `cfg(test)` elas não a alcançariam, e a
/// alternativa seria uma segunda cópia da lei na sonda, que é exactamente o que
/// deixou a tabela antiga descrever outro produto. ⛔ **Um caminho de produto
/// que a chamasse estaria a ignorar a faixa que o artista vê.**
#[doc(hidden)]
pub fn sharpen_total_for_measurement(
    stroke: &mut SculptStroke,
    mesh: &mut Mesh,
    brush: &Brush,
    total: f32,
) -> usize {
    stroke.filter_sharpen_unclamped(mesh, brush, total)
}

/// **A valência que a lei da referência pressupõe** — a de um quad (4) ou de um
/// triângulo regular (6), com folga para o maior dos dois.
///
/// ⚠️ **É o ponto em que a guarda de valência começa a morder, e abaixo dele ela
/// é a IDENTIDADE** — `min(1, 6/n) == 1` para todo `n <= 6` em aritmética exacta,
/// então nenhuma malha regular vê um bit diferente.
pub(super) const VALENCE_REGIME: usize = 6;

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
        // ⚠️ **O clamp é a ÚNICA coisa que esta porta acrescenta**, e é por isso
        // que a de medição pode delegar: uma segunda cópia da lei divergiria da
        // que shipa, que é precisamente o defeito que a medição existe para não
        // repetir.
        self.filter_sharpen_unclamped(mesh, brush, amount.clamp(lo, hi))
    }

    /// A lei, sem a faixa. Ver [`sharpen_total_for_measurement`].
    pub(crate) fn filter_sharpen_unclamped(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        total: f32,
    ) -> usize {
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
            // ⚠️ **O `f` é CONGELADO e corre UMA vez, e a distinção custou um
            // smoke.** No fonte o `sharpen_factor` vive no `filter_cache` —
            // construído no `sculpt_filter_specific_init`, ou seja **uma vez por
            // GESTO** — e toda iteração seguinte reusa o MESMO array. Eu
            // recomputava-o por sub-passo, e a consequência não é sutil: a lei
            // passa a perseguir o próprio rasto (a curvatura que ela acabou de
            // achatar deixa de a marcar como detalhe) e converge para um estado
            // **ALISADO**, que é exactamente o que o smoke reprovou.
            self.sharpen_freeze(mesh, brush);
            let n = steps_for(total);
            let per = total / n as f32;
            for _ in 0..n {
                self.sharpen_step(mesh, brush, per);
            }
        }

        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// **UM sub-passo:** a média VIVA do anel, depois o gather.
    ///
    /// ⚠️ **As duas metades do pré-passe têm tempos de vida DIFERENTES, e
    /// colapsá-las era o defeito.** O `sharpen_factor` é congelado no init do
    /// gesto; o `smooth_positions` que alimenta o termo médio é recomputado a
    /// cada evento (`neighbor_data_average_mesh_check_loose(position_data.eval,
    /// …)`, `:1673`) — ele mede a pose de AGORA. *Uma delas descreve onde o
    /// detalhe ESTAVA, a outra onde a superfície ESTÁ.*
    fn sharpen_step(&mut self, mesh: &mut Mesh, brush: &Brush, per: f32) {
        self.sharpen_live_average(mesh, brush);

        let pos = mesh.positions();
        for i in 0..self.touched.len() {
            let v = self.touched[i] as usize;
            let p = pos[v];
            let fi = self.sharp_f[i];

            // O gather sobre o anel, pesado pela curvatura de CADA VIZINHO —
            // é esta leitura que obriga o pré-passe a existir.
            let mut acc = [0.0f32; 3];
            let ring = mesh.adjacency().vert_verts.neighbours(v);
            for &nb in ring {
                let fs = self.sharp_f[self.slot[nb as usize] as usize];
                let q = pos[nb as usize];
                for k in 0..3 {
                    acc[k] += (q[k] - p[k]) * fs;
                }
            }
            // ⚠️ **A GUARDA DE VALÊNCIA — divergência DECLARADA, e ela nasceu de
            // a malha do smoke explodir.** O gather da referência é uma SOMA
            // sobre o anel, **não normalizada pela contagem**: com `f[i]` baixo e
            // os vizinhos altos ele vale `valência × laplaciano`, e um passo de
            // alisamento de fator maior que um OVERSHOOTA. A referência vive com
            // isso porque as malhas dela são regulares; a nossa importa OBJ/PLY/
            // STL, e uma esfera UV tem polos de valência igual ao número de
            // segmentos. Medido na crista, com o teto de fatia da referência:
            //
            // | malha | valência máx | degrau a força 4 |
            // |---|---|---|
            // | cubo subdividido (a do produto) | 4 | **1,046×** |
            // | esfera UV 48×64 | 64 | **1098×** (excursão 33 num raio 1) |
            //
            // ⚠️ **Ela é IDENTIDADE no regime da referência** (`min(1, 6/n)` vale
            // exactamente `1` para todo `n <= 6`, que é a valência de um quad ou
            // de um triângulo regular) ⇒ **a malha do produto é byte-idêntica** e
            // a divergência só existe onde a lei já não era utilizável.
            //
            // ⛔ **Normalizar pela valência SEMPRE foi a alternativa e está
            // recusada:** ela divide o efeito por ~4 no caso comum, onde ele já é
            // discreto, para curar um regime que a referência nem alcança.
            let scale = if ring.len() > VALENCE_REGIME {
                VALENCE_REGIME as f32 / ring.len() as f32
            } else {
                1.0
            };
            let hold = (1.0 - fi) * scale;
            let blend = SMOOTH_RATIO * fi * fi;
            let d = self.sharp_d[i];
            // ⚠️ **O TERMO QUE FAZ DELE UM AFIADOR** — `detail_directions ×
            // −intensify × f` (`:1605-1609`). Os outros dois apontam AMBOS para
            // a média (o topo da feição desce, a borda é puxada para dentro
            // dela), então sem este a lei só ESTREITA a feição: medido, **4,6%**
            // de degrau no teto do arrasto, invisível — e foi o que o smoke
            // reprovou (*"sharpen filter parece alisar o mesh"*).
            //
            // ⚠️ **Ele SUBTRAI o laplaciano** — é o `calc_enhance_details_filter`
            // escalado pela curvatura —, e é por isso que ele é o único dos três
            // que empurra o detalhe para FORA.
            let push = INTENSIFY * fi;
            for k in 0..3 {
                acc[k] = acc[k] * hold + d[k] * (blend - push);
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
    fn sharpen_freeze(&mut self, mesh: &Mesh, brush: &Brush) {
        self.sharpen_live_average(mesh, brush);

        let n = self.touched.len();
        self.sharp_f.clear();
        self.sharp_f.resize(n, 0.0);
        let mut max = 0.0f32;
        for i in 0..n {
            let d = self.sharp_d[i];
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

    /// **A média do anel na pose de AGORA** — o `smooth_positions` da
    /// referência, guardado como o deslocamento até ela.
    ///
    /// ⚠️ **Ele é VIVO, ao contrário do `sharp_f`**: a referência o recomputa a
    /// cada evento, e é por isso que o termo médio continua a puxar o vértice
    /// para a média do anel que ele tem AGORA em vez de para a que ele tinha no
    /// pen-down.
    fn sharpen_live_average(&mut self, mesh: &Mesh, brush: &Brush) {
        let n = self.touched.len();
        self.sharp_d.clear();
        self.sharp_d.resize(n, [0.0; 3]);
        for i in 0..n {
            let v = self.touched[i];
            let p = mesh.positions()[v as usize];
            let avg = self.neighbour_average(mesh, brush, v, p);
            self.sharp_d[i] = [avg[0] - p[0], avg[1] - p[1], avg[2] - p[2]];
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
