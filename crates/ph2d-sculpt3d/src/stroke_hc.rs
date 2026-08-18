//! **O `b` DO HC** — o deslocamento que o passo laplaciano TIROU, e a média
//! dele sobre o anel.
//!
//! Vollmer, Mencl & Müller, EG 1999, *Improved Laplacian Smoothing of Noisy
//! Surface Meshes*; o port é o `surface_smooth.cc` +
//! `sculpt_smooth.cc::surface_smooth_{laplacian,displace}_step` do Blender.
//!
//! ⚠️ **Este arquivo existe porque o [`Verb::SurfaceSmooth`] é o ÚNICO verbo que
//! não cabe numa função pura por-vértice**, e a §7.6 do plano 21 já o dizia
//! antes de existir código: *"o HC guarda um vetor `b` por vértice e faz uma
//! SEGUNDA passada sobre o anel para tirar a média dele"*. `média(b)` é um
//! operador de **segunda ordem** — ele precisa do `b` dos VIZINHOS —, e depois
//! de o passo laplaciano correr nenhum deles é recuperável da posição.
//!
//! # Por que UM passe e não dois
//!
//! O Blender roda dois `foreach_index` sobre os nós porque o array de `b` tem de
//! estar COMPLETO antes de alguém o mediar. Escrito por extenso, o que os dois
//! passes dele computam é:
//!
//! ```text
//! b_i     = média(q)_i − [α·o_i + (1−α)·q_i]
//! p_i     = q_i + f_i·(média(q)_i − q_i) − f_i·[(1−β)·média(b)_i + β·b_i]
//! ```
//!
//! — e **as duas linhas só leem `q`, `o` e `b`**, ou seja o estado de ANTES do
//! dab. ⇒ aqui o `b` é preenchido numa passada de preparação (irmã do
//! `fit_plane` e do `alpha_frame`, que também correm uma vez por dab) e o alvo
//! sai **inteiro** de um passe só. O [`crate::Brush::passes`] fica com o par
//! λ|μ do Taubin como único consumidor, que é o que ele nasceu para exprimir.
//!
//! ⚠️ **O buffer é do TRAÇO e não do dab, como no original** (`ss.cache
//! ->surface_smooth_laplacian_disp`, alocado a zeros uma vez por traço): um
//! vizinho que este dab não alcança contribui com o `b` que o dab ANTERIOR lhe
//! deixou, e um que ninguém alcançou contribui **zero**. A segunda metade é o
//! que a `resize` com zeros dá de graça; a primeira é a lei da referência, e ela
//! importa exactamente na casca da pegada — onde o peso já é ~0.

use super::*;

/// **α — *Shape Preservation*.** Quanto do ponto de referência do `b` é a pose
/// do **pen-down** em vez da posição de agora
/// (`surface_smooth_shape_preservation`, `rna_brush.cc:3357`, faixa `[0, 1]`).
///
/// ⚠️ **O valor de FÁBRICA do Blender NÃO é legível** — ele vive num `.blend`
/// binário desde o 4.3 (o `DNA_brush_types.h` inicializa os três campos a `0`,
/// que é o zero do C e não um default), e a §7.0 do plano 21 já registou essa
/// parede para a W1 inteira. ⇒ **este número é MEDIDO**, e a medição diz que
/// **não há óptimo**: o knob é uma troca monótona e SUAVE em toda a faixa
/// (`measure_what_the_alpha_holds_on_to`, 32 dabs, esfera rugosa, força cheia):
///
/// | α | deriva média do pen-down | deriva pior | rugosidade que resta |
/// |---|---|---|---|
/// | 0,00 | 0,004298 | 0,028722 | 66,8 % |
/// | 0,50 | 0,003147 | 0,023005 | 69,2 % |
/// | 1,00 | 0,002693 | 0,019267 | 71,6 % |
///
/// ⇒ **segurar mais a pose custa 1,6× menos deriva por 7 % menos alisamento**,
/// sem joelho e sem cliff em lado nenhum. Sobre uma troca assim o default
/// honesto é o **meio de um factor**, e é isso que este `0,5` é — dito, e não
/// vestido de medição.
///
/// ⚠️ **E o oráculo desta tabela NÃO é o raio.** A primeira varredura media o
/// encolhimento e o α movia-o de `−0,022 %` para `−0,006 %`: ruído. O `α` ancora
/// o `b` na pose do **pen-down**, logo o que ele governa é a DERIVA em relação a
/// ela — *o knob estava a ser julgado por uma régua que não mede o que ele faz*.
///
/// ⚠️ **Ele é INERTE no primeiro dab, por aritmética:** ali `q == o`, então
/// `α·o + (1−α)·q == q` para todo α. Um gate que medisse um dab só afirmaria
/// *"o α não faz nada"* sobre um knob correto.
pub const HC_SHAPE_DEFAULT: f32 = 0.5;

/// **β — *Per Vertex Displacement*.** Que fração da correção vem do `b` do
/// PRÓPRIO vértice em vez da média dos vizinhos
/// (`surface_smooth_current_vertex`, `rna_brush.cc:3364`).
///
/// ⚠️ **A faixa do Blender é `[0, 1]` e a NOSSA começa em `0,5`, porque abaixo
/// disso o operador AMPLIFICA** — e isto tem forma fechada, não é gosto.
/// Escrevendo `A` como a média do anel e `a` um autovalor dela, um passo do HC
/// com `α = 0` responde
///
/// ```text
/// f(a) = (1 − β)·(2a − a²) + β  =  1 − (1 − β)·(1 − a)²
/// ```
///
/// — que vale exactamente `1` em `a = 1` (o DC, ou seja **a forma**, que é a
/// razão de o verbo existir) e `−3 + 4β` no outro extremo `a = −1`. ⇒
/// `|f| ≤ 1` em toda a banda **sse `β ≥ 0,5`**.
///
/// ⚠️ **MEDIDO, e o piso algébrico é CONSERVADOR** — numa malha real o
/// autovalor mais negativo não chega a `−1`, então a fronteira observada cai
/// antes (`measure_where_the_beta_stops_contracting`, rugosidade relativa ao
/// piso, esfera rugosa):
///
/// | β | 4 dabs | 16 dabs | |
/// |---|---|---|---|
/// | 0,300 | 0,7989 | **43,84** | DIVERGE |
/// | 0,375 | 0,7370 | **2,06** | DIVERGE |
/// | 0,400 | 0,7315 | 0,8942 | DIVERGE |
/// | 0,425 | 0,7296 | 0,6972 | contrai |
/// | 0,500 | 0,7339 | 0,6806 | contrai |
///
/// ⇒ o piso fica em **`0,5`**: ele é o que a álgebra garante para QUALQUER
/// malha, e a medição mostra que ele sobra ~0,08 nesta. *Um slider que alcança
/// `0,3` é um slider que rebenta a malha em dezasseis dabs*, e o recurso que
/// este limite nomeia é a **contractividade do operador**.
///
/// ⚠️ **`β = 1` é o outro fim, e ali o pincel deixa de mexer na malha:** a
/// correção passa a ser exactamente o que o passo laplaciano somou, e o par
/// telescopa para `α·(o − q)` — com `α = 0` isso é a **identidade** (medido:
/// pior deslocamento `6,0e-8`, um ulp), e com `α > 0` é um *voltar ao pen-down*
/// cujo ponto fixo é `q = o`. É o `strength = 0` deste verbo, escrito no outro
/// eixo, e o gate `the_hc_can_be_tuned_into_a_no_op` pina-o para ninguém o
/// descobrir como *"o Surface Smooth parou de funcionar"*.
///
/// **O default é o piso**, que é onde ele alisa mais: `0,500` deixa `0,6806` da
/// rugosidade removível contra `0,6903` em `0,650`, e abaixo dele não há ganho
/// (0,475 dá 0,6795 — dentro do ruído) mas há a garantia perdida.
pub const HC_VERTEX_DEFAULT: f32 = 0.5;

/// **O PISO do β** — ver [`HC_VERTEX_DEFAULT`]. O motor CLAMPA por ele, então um
/// documento que traga um valor menor é corrigido em vez de rebentar.
pub const HC_VERTEX_MIN: f32 = 0.5;

impl SculptStroke {
    /// **PREENCHE o `b` da pegada deste dab** — a preparação que o
    /// [`Verb::SurfaceSmooth`] exige, e um no-op para os outros vinte e um.
    ///
    /// ⚠️ **O buffer só é ALOCADO por quem o usa:** o `capture` não o empurra
    /// junto dos cinco planos por-slot, então um traço de Draw numa malha de
    /// 5 M vértices não paga 12 bytes por vértice tocado por um verbo que ele
    /// não escolheu. A `resize` com zeros faz as duas coisas de que a lei
    /// precisa — cresce com o `touched` e dá `[0,0,0]` a quem nunca foi
    /// processado, que é o *"array zerado no início do traço"* do original.
    /// ⚠️ **O `needed` é do CHAMADOR porque a pergunta é DIFERENTE nos dois, e
    /// esta função já respondeu a errada uma vez.** Ela perguntava
    /// `brush.verb != Verb::SurfaceSmooth`, o que era exacto enquanto a LEI de
    /// um filtro fosse derivada do verbo em mãos — a W9b pôs um picker, o
    /// artista passou a escolher a lei com qualquer pincel, e o early-out
    /// deixou o `hc_b` **vazio** debaixo de um leitor: `index out of bounds:
    /// the len is 0 but the index is 1` (reportado no smoke da `=34`).
    ///
    /// O dab pergunta *"o verbo em mãos lê isto?"*, o filtro pergunta *"a lei
    /// escolhida lê isto?"*, e **as duas respostas vivem em sítios diferentes**
    /// — uma condição que enumera os seus leitores apodrece no dia em que o
    /// segundo nasce, que é literalmente o que aconteceu.
    ///
    /// ⛔ **A cura preguiçosa era recuar no LEITOR** (devolver zeros com o
    /// buffer vazio) e ela é pior que o panic: o chip ficaria pintado, vivo sob
    /// o rato e **inerte**, que é um controle morto que ninguém vê. O buffer
    /// segue **preguiçoso** — só que agora a preguiça é função de quem lê.
    pub(super) fn fill_hc_disp(&mut self, mesh: &Mesh, brush: &Brush, needed: bool) {
        if !needed {
            return;
        }
        self.hc_b.resize(self.touched.len(), [0.0; 3]);
        let alpha = brush.hc_shape.clamp(0.0, 1.0);
        // Índice e não `for &v in &self.footprint`: o laço escreve em
        // `self.hc_b`, e o empréstimo imutável da lista fecharia a porta. É o
        // mesmo laço que a captura, três linhas acima do chamador, já usa.
        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            let vi = v as usize;
            let s = self.slot[vi] as usize;
            let q = mesh.positions()[vi];
            let avg = self.neighbour_average(mesh, brush, v, q);
            let o = self.base_pos[s];
            let mut b = [0.0f32; 3];
            for k in 0..3 {
                // `d = α·o + (1−α)·q` — o ponto de referência de que o `b` mede
                // a distância. Com `α = 0` ele é a posição de agora (e o `b` é o
                // deslocamento laplaciano puro); com `α = 1` é a pose do
                // pen-down, e o verbo puxa de volta para ela.
                let d = alpha * o[k] + (1.0 - alpha) * q[k];
                b[k] = avg[k] - d;
            }
            self.hc_b[s] = b;
        }
    }

    /// O `b` deste slot.
    pub(super) fn hc_disp(&self, s: usize) -> [f32; 3] {
        self.hc_b.get(s).copied().unwrap_or([0.0; 3])
    }

    /// **A MÉDIA do `b` sobre o anel de `v`.**
    ///
    /// ⚠️ **Pelo MESMO [`ph2d_mesh::ring_average`] que as posições**, e não por
    /// uma média crua — a referência usa a média crua nos dois lados
    /// (`neighbor_data_average_mesh`), e é aqui que divergimos de propósito. As
    /// duas regras de borda daquela porta existem para que um vértice de beira
    /// se mova ao longo da curva de borda em vez de ser sugado para dentro; o
    /// `b` de um vértice de beira é, por construção, um deslocamento ao longo
    /// dessa curva, e mediá-lo sobre vizinhos INTERIORES — cujo `b` aponta para
    /// dentro — desfaria a regra exactamente onde ela foi paga.
    ///
    /// ⚠️ **A base é o `b` do próprio vértice**, o que é a resposta certa nos
    /// casos degenerados que a porta trata (anel de dois, beira sem dois
    /// vizinhos de beira): ali a média das posições também devolve a própria
    /// posição ⇒ o passo laplaciano não moveu nada, e subtrair `b_i` inteiro
    /// devolve o mesmo `q_i`. *O verbo não mexe em quem ele não sabe alisar.*
    ///
    /// ⚠️ **Um vizinho FORA do traço conta ZERO**, não é pulado: é o array
    /// zerado do original, e pulá-lo mudaria o DIVISOR — o anel de um vértice
    /// da casca da pegada passaria a ser medido só pela metade que já está
    /// dentro dela, e a correção saltaria na fronteira do pincel.
    pub(super) fn hc_disp_average(&self, mesh: &Mesh, v: u32, own: [f32; 3]) -> [f32; 3] {
        ph2d_mesh::ring_average(mesh.adjacency(), v, own, |nb| {
            let ni = nb as usize;
            if self.stamp[ni] == self.epoch {
                self.hc_b[self.slot[ni] as usize]
            } else {
                [0.0; 3]
            }
        })
    }
}
