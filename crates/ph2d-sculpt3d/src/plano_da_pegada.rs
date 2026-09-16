//! ⭐⭐⭐ **O PLANO DO [`Verb::Plane`]** — a normal, o centro e o deslocamento
//! (`SPEC_pincel_de_plano.md` §2).
//!
//! ⚠️⚠️ **Ele NÃO é o [`super::plane`], e as diferenças são a wave inteira.**
//! Aquele serve os quatro verbos de plano portados de uma referência MIT e pesa
//! pela **máscara** sobre a **pegada inteira**; este pesa por uma **curva suave**
//! sobre **dois raios próprios**, e o centro dele não é uma média ponderada de
//! coisa nenhuma. Medido, a diferença chega a **`17,1 %` do raio** no centro e a
//! **`31,2°`** na normal — não é afinação, é outra lei.
//!
//! ⛔⛔ **E não se funde os dois «para não repetir código».** O estimador antigo
//! tem paridade medida a **1 ULP** contra a referência MIT em quatro verbos; uma
//! função com um `if` para escolher a ponderação poria as duas leis num sítio
//! onde uma edição futura alcança as duas. *O que é partilhado — a curva de
//! amostragem e os dois baldes — é partilhado por PORTA; o resto é separado de
//! propósito.*
//!
//! # A metade que JÁ EXISTIA nesta casa
//!
//! ⭐ A lei da **normal** (§2.1) é, letra por letra, a que o
//! [`super::normal_do_gesto`] já corre para os pincéis de puxar: raio próprio,
//! curva **suave fixa** (`3p² − 2p³`), e **dois baldes** com o da frente a ganhar
//! sempre. É a mesma referência, então é a mesma lei — e por isso a curva de
//! amostragem sai de uma porta só ([`super::normal_do_gesto::peso_da_amostra`]).
//!
//! ⚠️ **O que muda é a POSE de leitura**, que aqui vem da porta
//! [`Verb::le_a_superficie_viva`]: este verbo **consulta o acumular**, ao lado
//! dos três cuja referência é a mesma. ⛔ Os quatro verbos de plano da casa lêem
//! o vivo sempre, e mandá-lo para lá reabriria um controlo morto que esta casa já
//! removeu uma vez.

use super::*;

/// O plano que um dab do [`Verb::Plane`] ajusta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlanoDaPegada {
    /// O centro, **já com o deslocamento aplicado** (§2.4).
    pub(super) centro: [f32; 3],
    /// A normal, unitária.
    pub(super) normal: [f32; 3],
}

impl SculptStroke {
    /// **O plano da pegada**, ou `None` quando a superfície não respondeu.
    ///
    /// ⚠️ **`None` não é um erro a esconder:** quem chama cai na pegada de disco,
    /// ou seja no pincel sem tectos — a única resposta finita quando não há
    /// amostra nenhuma. Devolver um plano `NaN` envenenaria a malha inteira.
    pub(super) fn plano_da_pegada(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
    ) -> Option<PlanoDaPegada> {
        let viva = brush.verb.le_a_superficie_viva(brush.accumulate);
        let normal = self.normal_da_area(mesh, brush, dab, viva)?;
        let centro = self.centro_da_area(mesh, brush, dab, viva)?;
        // ⭐ **O DESLOCAMENTO** (§2.4), medido exactamente: com `+0,2` num pincel
        // de raio `0,4` o plano anda `+0,08000` ao longo da normal — razão
        // `0,2000` do raio, ao dígito impresso.
        //
        // ⚠️ **Ele NÃO transforma este verbo noutro.** Na nossa casa o mesmo knob
        // faz de um `Flatten` um `Clay`; aqui ele apenas levanta ou baixa o
        // plano, e quem decide quem é tocado são os dois tectos.
        let lift = dab.radius * brush.plane_offset;
        Some(PlanoDaPegada {
            centro: [
                centro[0] + normal[0] * lift,
                centro[1] + normal[1] * lift,
                centro[2] + normal[2] * lift,
            ],
            normal,
        })
    }

    /// **A NORMAL DA ÁREA** (§2.1) — a média das normais dos VÉRTICES, pesada
    /// pela curva suave da distância, sobre `R_n`.
    ///
    /// ⚠️ **É a normal do VÉRTICE que entra, não a da face.**
    ///
    /// ⚠️ **A ponderação e a EXTENSÃO foram DISCRIMINADAS por medição, não
    /// escolhidas:** a candidata desta função lê `0,0000°` de desvio angular em
    /// **todas** as `11` configurações do corpus, contra `0,00°`–`31,2°` da nossa
    /// lei real (máscara, pegada inteira) e `0,00°`–`7,99°` de uma média simples
    /// sobre o mesmo `R_n`. *Três candidatas, uma régua, uma sobrevivente.*
    fn normal_da_area(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        viva: bool,
    ) -> Option<[f32; 3]> {
        let r = dab.radius * brush.normal_radius_frac;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }
        let (mut frente, mut verso) = ([0.0f64; 3], [0.0f64; 3]);
        for &v in &self.footprint {
            let s = self.slot[v as usize] as usize;
            let vi = v as usize;
            let p = if viva {
                mesh.positions()[vi]
            } else {
                self.base_pos[s]
            };
            let Some(peso) = peso_dentro(p, dab, r) else {
                continue;
            };
            let n = if viva {
                mesh.normals()[vi]
            } else {
                self.base_nrm[s]
            };
            let balde = if de_frente(n, dab) {
                &mut frente
            } else {
                &mut verso
            };
            for k in 0..3 {
                balde[k] += f64::from(n[k]) * peso;
            }
        }
        normalizar(frente).or_else(|| normalizar(verso))
    }

    /// ⭐⭐⭐ **O CENTRO DA ÁREA — a lei que NENHUMA intuição dá** (§2.2).
    ///
    /// ```text
    /// peso(v)     = suave(1 − dist(v, cursor) / R_c)      [a MESMA curva da normal]
    /// ajustado(v) = cursor + (posição(v) − cursor) · (1 − peso(v))
    /// centro      = média_aritmética_simples( ajustado(v) )
    /// ```
    ///
    /// ⭐⭐⭐ **Leia-o duas vezes: o peso NÃO multiplica a posição — ele PUXA a
    /// posição para o cursor, e a média é SIMPLES.** Um vértice no miolo da
    /// pegada (peso `1`) contribui com o **cursor**; um na borda (peso `0`)
    /// contribui com a **própria posição**. É o oposto do que a palavra *peso*
    /// sugere, e é por isso que nenhuma das três candidatas óbvias acerta.
    ///
    /// ⚠️ **A régua que as separou é a ALTURA de cada candidata contra o plano
    /// que o alvo de facto usou**, recuperado por ajuste: esta cai **exactamente**
    /// nele em todas as `11` células; as outras erram por uma fracção do raio que
    /// o produto sente (piso `0,00269`, de onde sai a barra do gate G-5).
    ///
    /// ⚠️ **Os dois baldes valem aqui também**, e pela mesma razão do irmão: sem
    /// eles um dab perto da silhueta ajusta o plano com vértices do outro lado da
    /// peça.
    fn centro_da_area(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        viva: bool,
    ) -> Option<[f32; 3]> {
        // ⭐ **A QUEDA** (§2.3): a fracção da área a zero cai na fracção da
        // normal. ⚠️ **Provada por um PAR, não lida** — com as duas iguais a
        // saída é byte-idêntica, e com a da normal em `1,0` as mesmas duas
        // divergem `5,6e-2`. *Um zero que cai noutro knob não se vê num valor,
        // vê-se num par.*
        let frac = if brush.area_radius_frac > 0.0 {
            brush.area_radius_frac
        } else {
            brush.normal_radius_frac
        };
        let r = dab.radius * frac;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }
        let mut baldes = [([0.0f64; 3], 0usize), ([0.0f64; 3], 0usize)];
        for &v in &self.footprint {
            let s = self.slot[v as usize] as usize;
            let vi = v as usize;
            let p = if viva {
                mesh.positions()[vi]
            } else {
                self.base_pos[s]
            };
            let Some(peso) = peso_dentro(p, dab, r) else {
                continue;
            };
            let n = if viva {
                mesh.normals()[vi]
            } else {
                self.base_nrm[s]
            };
            let (soma, n_amostras) = &mut baldes[usize::from(!de_frente(n, dab))];
            // ⚠️ **O COMPLEMENTO do peso**, e é aqui que a lei vive.
            let puxao = 1.0 - peso;
            for k in 0..3 {
                soma[k] +=
                    f64::from(dab.center[k]) + (f64::from(p[k]) - f64::from(dab.center[k])) * puxao;
            }
            *n_amostras += 1;
        }
        // ⚠️ **A ordem dos baldes é FIXA: o da frente ganha se não estiver
        // vazio** — a mesma lei da normal, e a mesma que o [`super::plane`] já
        // aplica. Ela não é uma preferência: é o que impede um dab na silhueta de
        // ajustar o plano com a casca de trás.
        let (soma, n_amostras) = baldes.into_iter().find(|(_, n)| *n > 0)?;
        #[allow(clippy::cast_precision_loss)]
        let inv = 1.0 / n_amostras as f64;
        Some([
            (soma[0] * inv) as f32,
            (soma[1] * inv) as f32,
            (soma[2] * inv) as f32,
        ])
    }
}

/// **O peso da amostra, ou `None` quando ela está fora de `r`.**
///
/// ⚠️ **Porta e não duas cópias:** a normal e o centro amostram com raios
/// DIFERENTES e a mesma curva, e escrever o teste duas vezes seria a forma de um
/// deles ganhar um `<=` no dia em que alguém mexesse no outro.
fn peso_dentro(p: [f32; 3], dab: &Dab, r: f32) -> Option<f64> {
    let (dx, dy, dz) = (
        p[0] - dab.center[0],
        p[1] - dab.center[1],
        p[2] - dab.center[2],
    );
    let d = (dx * dx + dy * dy + dz * dz).sqrt();
    if d > r {
        return None;
    }
    Some(f64::from(super::normal_do_gesto::peso_da_amostra(d, r)))
}

/// O vértice olha para quem vê?
///
/// ⚠️ **O `<= 0` é o mesmo teste que o [`super::plane`] e o
/// [`super::normal_do_gesto`] usam** — o `eye` aponta da câmara para a cena, logo
/// um vértice virado para quem olha tem produto **negativo**. Escrevê-lo ao
/// contrário poria o balde do verso a ganhar, e o plano sairia invertido numa
/// peça fina.
fn de_frente(n: [f32; 3], dab: &Dab) -> bool {
    n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] <= 0.0
}

/// `None` quando a soma degenera — o mesmo contrato dos irmãos, e pela mesma
/// razão: um `NaN` aqui envenena a malha inteira.
fn normalizar(v: [f64; 3]) -> Option<[f32; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 {
        return None;
    }
    let inv = 1.0 / len;
    Some([
        (v[0] * inv) as f32,
        (v[1] * inv) as f32,
        (v[2] * inv) as f32,
    ])
}
