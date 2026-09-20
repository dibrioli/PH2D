//! ⭐⭐⭐ **EM TORNO DE QUE PONTO CADA PEDAÇO DA ARTE RODA** — a cura do entalhe do cotovelo.
//!
//! # ⛔⛔⛔ O defeito, medido
//!
//! A mistura linear (`Σ wᵢ · Mᵢ p`) interpola POSIÇÕES, e a interpolação de duas posições de uma
//! rotação é a **CORDA** do arco, nunca o arco. ⇒ um ponto a meio caminho entre dois ossos que
//! divergem `θ` é puxado para `cos(θ/2)` da distância à junta, e a arte **colapsa** — é o
//! *«candy-wrapper»* de toda a literatura, e o entalhe que o dono fotografou em 2026-09-19.
//!
//! Medido numa barra de `7 × 1` com dois ossos (o PESCOÇO é o sítio mais estreito da forma, e em
//! repouso ele é a espessura dela):
//!
//! | dobra | pescoço, mistura linear | área |
//! |---|---|---|
//! | `60°` | `0,4882` | `91,3 %` |
//! | `90°` | `0,3343` | `82,6 %` |
//! | `120°` | `0,1643` | `73,9 %` |
//!
//! # ⭐⭐ A lei: em 2D o centro de rotação NÃO se estima — ele é SABIDO
//!
//! A literatura estima-o (*Optimized Centers of Rotation*, Le & Hodgins 2016: uma média das
//! posições de repouso pesada pela SEMELHANÇA entre vectores de peso, com um `σ` a afinar). ⛔ Aqui
//! ele não precisa de ser estimado: **um esqueleto 2D é uma árvore de segmentos que PARTILHAM
//! pontas**, e dois ossos que disputam um ponto partilham uma junta. ⇒ o centro é a média das
//! juntas de cada par, pesada pelo produto dos pesos do par.
//!
//! Medido lado a lado, na mesma barra com a junta FORA do centróide:
//!
//! | dobra | mistura linear | núcleo de semelhança (`σ² = 5e-4`) | **a JUNTA** |
//! |---|---|---|---|
//! | `60°` | `0,4882` | `0,9496` | **`0,9432`** |
//! | `90°` | `0,3343` | `0,8615` | **`0,8659`** |
//! | `120°` | `0,1643` | `0,3317` | **`0,3763`** |
//!
//! ⇒ *a junta iguala ou bate o estimador, **sem um parâmetro e sem um byte guardado**.*
//!
//! ⚠️⚠️ **A 1.ª fixtura escondeu o mecanismo**: com a junta no MEIO da barra, o centróide da forma
//! calha nela, e o estimador devolvia `(3,500 · 0,500)` para **todo** peso — `σ²` de `2e-5` a `100`
//! dava o mesmo número. *Uma fixtura simétrica aprova um estimador que não estima nada.*
//!
//! # ⭐ Porque não há descontinuidade onde um osso manda sozinho
//!
//! Com `w = (1, 0, …)` a transformação é RÍGIDA, e uma rotação rígida leva `p` ao mesmo sítio
//! **qualquer que seja o centro**: `R(θ₁)(p − c) + M₁(c) = M₁(p)` para todo `c`. ⇒ o centro deixa de
//! importar exactamente onde ele deixa de existir, e a lei é contínua por construção.
//!
//! ⛔ **E o `dual quaternion` (o `se(2)`) é RECUSA MEDIDA deste repo** — ele piora a dobra de uma
//! imagem (`2,52 % → 4,47 %` dobrada sobre si mesma). Ele resolve o mesmo defeito escolhendo a
//! ROTAÇÃO certa e deixando o centro ao acaso; esta lei escolhe o CENTRO e deixa a rotação ser a
//! média.

use crate::{Skin, SkinBone};

/// ⭐⭐⭐ **COMO A MISTURA RESOLVE O ÂNGULO MÉDIO DE VÁRIOS OSSOS.**
///
/// # ⛔⛔⛔ Porque existe uma escolha aqui
///
/// A auditoria de 2026-09-20 (ordem do dono, ao report *«não houve nenhuma melhora na deformação
/// do vetor»*) mediu o caminho vectorial contra um padrão-ouro construído de raiz — a mesma forma
/// deformada como malha densa sobre o campo do domínio — e o veredito foi contra a suspeita:
///
/// | o que | medido |
/// |---|---|
/// | o caminho vectorial contra o padrão-ouro, até `90°` | `≤ 0,46 %` da espessura |
/// | os NÓS | exactos ao bit |
/// | a quina nas junções (a promessa do `reconcilia`) | `0,0000°` |
/// | **e o padrão-ouro tem o MESMO vinco** | `40,7°` a `70°`, `180°` a `90°` |
///
/// ⇒ *o defeito dominante não é a curva nem o peso: é a LEI.* A aresta de DENTRO de um cotovelo é
/// esmagada por `|1 − θ̄′·r|`, e quando `θ̄′·r = 1` ela colapsa num BICO — na barra da cena do dono
/// isso dá **`≈ 92,7°`**, com previsão e medição a bater a 3–4 casas decimais.
///
/// # ⭐⭐ A terceira saída, que NUNCA tinha sido medida
///
/// O cabeçalho deste módulo declarava a mistura fechada: *«o `dual quaternion` é RECUSA MEDIDA»*.
/// Ele é — e a recusa responde a **outra** pergunta (ele escolhe a ROTAÇÃO e deixa o centro ao
/// acaso; esta lei escolhe o CENTRO). A saída que ninguém tinha corrido é trocar a média em
/// CÍRCULO por uma média LINEAR sobre os ângulos **DESDOBRADOS** ao longo da cadeia.
///
/// ⚠️ **Ela nasce DESLIGADA** ([`MisturaDoAngulo::Circulo`] é o `Default`) e o caminho de omissão
/// é **byte-idêntico**, com gate a afirmá-lo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MisturaDoAngulo {
    /// `atan2(Σ w sin θ, Σ w cos θ)` — o que o produto ship, e o que o shader implementa.
    #[default]
    Circulo,
    /// `Σ w θ̃`, com `θ̃` o ângulo escolhido na volta mais próxima do osso ANTERIOR da cadeia.
    ///
    /// ⛔ **Ela NÃO tem a degenerescência da irmã** (duas poses a `180°` exactos com pesos iguais,
    /// onde a soma dos vectores é zero e a lei cai na mistura linear): desdobrados, `+180°` e
    /// `−180°` são o **mesmo** ângulo e a média deles é ele próprio.
    Desdobrado,
}

impl SkinBone {
    /// O segmento de repouso **deste sub-osso** — ver [`SkinBone::sub`].
    ///
    /// ⚠️ Os sub-ossos de um osso que dobra partilham `rest_a`/`rest_b`; o que os distingue é a
    /// fatia. *Sem isto, dois sub-ossos consecutivos teriam o eixo inteiro como junta partilhada e
    /// a média cairia no meio do osso em vez de na dobra.*
    #[must_use]
    pub fn eixo_do_sub(&self) -> ([f64; 2], [f64; 2]) {
        let n = f64::from(self.sub.1.max(1));
        let k = f64::from(self.sub.0);
        let lerp = |t: f64| {
            [
                (self.rest_b[0] - self.rest_a[0]).mul_add(t, self.rest_a[0]),
                (self.rest_b[1] - self.rest_a[1]).mul_add(t, self.rest_a[1]),
            ]
        };
        (lerp(k / n), lerp((k + 1.0) / n))
    }

    /// O ângulo da parte linear da [`SkinBone::pose`] — quanto este osso RODOU do repouso.
    #[must_use]
    pub fn angulo_da_pose(&self) -> f64 {
        let [a, b, ..] = self.pose.0;
        b.atan2(a)
    }
}

/// A junta que dois sub-ossos partilham, em repouso — a **ponta mais próxima** de um ao outro.
///
/// ⭐ Para um par pai→filho e para dois sub-ossos consecutivos ela é **exacta** (as duas pontas
/// coincidem). Para dois ossos de ramos diferentes é o ponto médio das pontas mais próximas, que é
/// a leitura honesta de *«por onde estes dois se encontram»*.
fn junta(x: &SkinBone, y: &SkinBone) -> [f64; 2] {
    let (xa, xb) = x.eixo_do_sub();
    let (ya, yb) = y.eixo_do_sub();
    let d2 = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    let mut melhor = (f64::INFINITY, [0.0, 0.0]);
    for p in [xa, xb] {
        for q in [ya, yb] {
            let d = d2(p, q);
            if d < melhor.0 {
                melhor = (d, [(p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5]);
            }
        }
    }
    melhor.1
}

impl Skin {
    /// ⭐⭐⭐ **A TABELA DAS JUNTAS, `n × n` em ordem de linha** — `t[i·n + j]` é a
    /// [`junta`] do par `(i, j)`, no espaço LOCAL da pele.
    ///
    /// ⚠️ **Ela existe para o payload do DISPOSITIVO** ([`ph2d_render::SpriteMeshSkin`]): a lei do
    /// [`Skin::blend`] precisa da junta de cada par de ossos que manda num vértice, e um shader não
    /// pode chamar a [`junta`]. ⛔ **O produtor do payload NÃO a reimplementa** — ele pede-a aqui,
    /// que é o que impede a segunda resposta à pergunta *«por onde estes dois se encontram?»*.
    ///
    /// ⚠️ **É simétrica por construção** (a [`junta`] não distingue a ordem), e a diagonal é a ponta
    /// partilhada de um osso consigo mesmo — nunca lida, porque o laço do centro só vê `j > i`.
    ///
    /// ⚠️ **Custo `O(n²)` e ela é do BIND**, não do quadro: os eixos de repouso não mudam com a
    /// pose. *Quem a recalcular por quadro paga um quadrado por nada.*
    #[must_use]
    pub fn tabela_de_juntas(&self) -> Vec<[f64; 2]> {
        let n = self.bones.len();
        let mut t = vec![[0.0_f64; 2]; n * n];
        for (i, a) in self.bones.iter().enumerate() {
            for (j, b) in self.bones.iter().enumerate() {
                t[i * n + j] = junta(a, b);
            }
        }
        t
    }

    /// ⭐⭐ **O `(cos θ, sin θ)` de cada osso**, com `θ` o [`SkinBone::angulo_da_pose`].
    ///
    /// ⚠️ **O par sai daqui já resolvido, e não do afim:** o shader precisa do ângulo da parte
    /// linear CRUA, e o afim que ele recebe está **conjugado** para o espaço do quad — o `atan2`
    /// dele daria outro ângulo. ⛔ *Uma grandeza que se lê antes da conjugação não se re-deriva
    /// depois dela.*
    #[must_use]
    pub fn angulos_das_poses(&self) -> Vec<[f64; 2]> {
        self.bones
            .iter()
            .map(|b| {
                let t = b.angulo_da_pose();
                [t.cos(), t.sin()]
            })
            .collect()
    }

    /// ⭐⭐⭐ **O CENTRO EM TORNO DO QUAL UM PONTO COM ESTES PESOS RODA.**
    ///
    /// `None` quando **um só** osso manda (não há par, logo não há junta) — e ali o centro é
    /// irrelevante, porque a transformação é rígida. Ver o cabeçalho do módulo.
    #[must_use]
    pub fn centro_de_rotacao(&self, w: &[f64]) -> Option<[f64; 2]> {
        let (mut num, mut den) = ([0.0_f64, 0.0], 0.0_f64);
        for (i, a) in self.bones.iter().enumerate() {
            let wi = w.get(i).copied().unwrap_or(0.0);
            if wi <= 0.0 {
                continue;
            }
            for (j, b) in self.bones.iter().enumerate().skip(i + 1) {
                let wj = w.get(j).copied().unwrap_or(0.0);
                if wj <= 0.0 {
                    continue;
                }
                let q = wi * wj;
                let c = junta(a, b);
                num[0] = q.mul_add(c[0], num[0]);
                num[1] = q.mul_add(c[1], num[1]);
                den += q;
            }
        }
        (den > 0.0 && num[0].is_finite() && num[1].is_finite())
            .then(|| [num[0] / den, num[1] / den])
    }

    /// ⭐⭐⭐ **A MISTURA — rodar em torno da JUNTA, e transladar pela mistura linear DELA.**
    ///
    /// ⚠️ **Ela tem o nome `blend` de propósito:** era esse o nome da lei antiga, e toda a casa o
    /// chama. *Trocar o corpo e deixar o nome é o que faz a cura chegar às quatro mídias sem um
    /// mapa de excepções* — a antiga ficou como [`Skin::blend_linear`], que é o CONTROLO.
    ///
    /// `p' = R(θ̄)·(p − c) + Σ wᵢ Mᵢ(c)`
    ///
    /// ⚠️ **O ângulo é a média em CÍRCULO por omissão** (`atan2(Σ w sin, Σ w cos)`), e não a soma
    /// `Σ w θ`: os ângulos vêm de um `atan2` e vivem em `(−π, π]`, logo somá-los **crus** salta
    /// quando um osso passa meia volta. ⭐ A saída que os soma **DESDOBRADOS** existe desde
    /// 2026-09-20 e é a [`MisturaDoAngulo::Desdobrado`], ligada por
    /// `ph2d_skeleton_live::skin_live::mistura_do_ambiente`.
    ///
    /// ⛔⛔⛔ **E a degenerescência que esta prosa declarava NÃO é alcançável:** ela dizia que com
    /// duas rotações a `180°` exactas e pesos iguais *«a soma é zero e a lei cai na mistura
    /// linear»*. Medido — `sin(π)` em `f64` vale **`1,2246e-16`**, logo `Σ w sin` é
    /// `+6,123234e-17`, a guarda `sx == 0 && sy == 0` **nunca arma**, e o `atan2` devolve **`+90°`:
    /// uma rotação inteira tirada do resíduo de um arredondamento.** *Uma promessa de fallback num
    /// doc-comment é o pior sítio para uma guarda morta — quem a lê deixa de procurar o caso.*
    /// A guarda FICA (ela é barata e a soma pode ser zero por outro caminho), e o que a descreve
    /// agora é o gate `na_meia_volta_o_guarda_do_circulo_nao_arma`.
    #[must_use]
    pub fn blend(&self, p: [f64; 2], w: &[f64]) -> [f64; 2] {
        self.blend_com(p, w, self.mistura)
    }

    /// ⭐⭐⭐ **A MISTURA COM A LEI DO ÂNGULO ESCOLHIDA À MÃO** — a porta dos gates e das sondas.
    ///
    /// ⚠️⚠️ **Ela existe para a lei ser um PARÂMETRO e nunca o ambiente.** Este repo já pagou a
    /// lição no colisor do Motion: *uma lei que só é alcançável pelo ambiente não é gateável, e um
    /// gate que lê o ambiente mede a máquina*. O produto escolhe uma vez, na porta que constrói a
    /// [`Skin`]; um gate chama esta função com as duas leis e compara-as.
    #[must_use]
    pub fn blend_com(&self, p: [f64; 2], w: &[f64], lei: MisturaDoAngulo) -> [f64; 2] {
        let Some(c) = self.centro_de_rotacao(w) else {
            return self.blend_linear(p, w);
        };
        let Some((co, si)) = self.direccao_media(w, lei) else {
            return self.blend_linear(p, w);
        };
        let base = self.blend_linear(c, w);
        let d = [p[0] - c[0], p[1] - c[1]];
        [
            si.mul_add(-d[1], co.mul_add(d[0], base[0])),
            si.mul_add(d[0], co.mul_add(d[1], base[1])),
        ]
    }

    /// O `(cos θ̄, sin θ̄)` da rotação média, ou `None` quando a lei não tem resposta e o chamador
    /// tem de cair na [`Skin::blend_linear`].
    ///
    /// ⛔⛔ **No braço DESDOBRADO o `continue` do peso zero vem DEPOIS de a cadeia avançar, e isso é
    /// a lei inteira:** desdobrar é uma propriedade da CADEIA, não do ponto. Saltar um osso sem
    /// peso antes de actualizar a referência faria o ângulo do osso seguinte depender de **quais**
    /// ossos aquele ponto por acaso reclama — dois pontos vizinhos com pesos diferentes leriam
    /// voltas diferentes, e a arte rasgava na fronteira entre eles.
    fn direccao_media(&self, w: &[f64], lei: MisturaDoAngulo) -> Option<(f64, f64)> {
        match lei {
            MisturaDoAngulo::Circulo => {
                let (mut sx, mut sy, mut soma) = (0.0_f64, 0.0_f64, 0.0_f64);
                for (b, &peso) in self.bones.iter().zip(w.iter()) {
                    if peso == 0.0 {
                        continue;
                    }
                    let t = b.angulo_da_pose();
                    sx = peso.mul_add(t.cos(), sx);
                    sy = peso.mul_add(t.sin(), sy);
                    soma += peso;
                }
                if soma == 0.0 || (sx == 0.0 && sy == 0.0) {
                    return None;
                }
                let n = sx.hypot(sy);
                Some((sx / n, sy / n))
            }
            MisturaDoAngulo::Desdobrado => {
                let (mut ang, mut soma, mut ant) = (0.0_f64, 0.0_f64, 0.0_f64);
                for (b, &peso) in self.bones.iter().zip(w.iter()) {
                    let mut t = b.angulo_da_pose();
                    while t - ant > std::f64::consts::PI {
                        t -= std::f64::consts::TAU;
                    }
                    while t - ant < -std::f64::consts::PI {
                        t += std::f64::consts::TAU;
                    }
                    ant = t;
                    if peso == 0.0 {
                        continue;
                    }
                    ang = peso.mul_add(t, ang);
                    soma += peso;
                }
                (soma != 0.0)
                    .then(|| (ang / soma).sin_cos())
                    .map(|(s, c)| (c, s))
            }
        }
    }
}

#[cfg(test)]
#[path = "centro_tests.rs"]
mod centro_tests;
