//! ⭐⭐⭐ **A BASE DA CADEIA MOLHADA** — a cor que cada amostra tinha ANTES do
//! primeiro traço de uma cadeia de traços de água, guardada de traço em traço
//! (report do dono, 30/09, sobre a cura de 29/09: *«melhorou mas não curou
//! perfeitamente»*).
//!
//! # O defeito que ela cura
//!
//! A lei da tela semeada é `nova = base + k·(c − s)` ([`super`]): a `base` é a
//! amostra antes do traço e `s` o retrato no ponto dela. Isso só devolve
//! `nova ≈ c` quando `base − s ≈ 0`, e é o que acontece num traço na MESMA
//! vista — a amostra foi escrita pela mesma tela, lida no mesmo ponto. Depois
//! de RODAR a peça, a amostra traz a frente da água de antes com o detalhe da
//! vista de antes, e o retrato novo é essa mesma cor amostrada nos centros de
//! píxel da vista NOVA: numa frente dura as duas diferem, e a lei SOMAVA essa
//! diferença a cada quadro enquanto a água passava — a linha clara (ou escura)
//! onde a frente estava. Com a semente de 29/09 (o retrato em vez da tela
//! levada) a diferença passou de «um píxel inteiro» para «meio píxel
//! interpolado»: melhorou, e não se anulou.
//!
//! # A lei que fica
//!
//! Numa cadeia, a base de uma amostra é a cor dela ANTES DA CADEIA, e `s` é o
//! retrato da peça SEM a cadeia — `nova = antes + k·(c − s₀)`. A tela de água
//! do Painter é o pigmento de toda a cadeia por cima de `s₀` (é essa a base
//! congelada dela), logo `c − s₀` é exactamente o pigmento, e a frente de antes
//! deixa de existir como resíduo: ela só vive no pigmento, que corre.
//!
//! ⚠️ **Na mesma vista a lei nova é a antiga, somada:** a antiga era
//! `(antes + k·(c₁ − s₀)) + k·(c − c₁)` — um traço reaproveitado semeado com a
//! última tela —, e isso é `antes + k·(c − s₀)`.
//!
//! ⚠️ **Uma amostra da cadeia é repintada mesmo com `c − s₀ = 0`** — é quando
//! a água já SAIU dali, e a amostra volta a ser o que era antes dela (a lei
//! antiga fazia-o com `c − c₁ ≠ 0`).
//!
//! ⚠️ **Rodar deixa na cadeia só o que a vista nova herdou da de antes**
//! ([`BaseDaCadeia::so_o_que_a_vista_levou`]): a água de um sítio que a vista
//! de antes não via não viajou, e a tinta que ali pousou passa a ser da peça
//! (sem isso ela seria APAGADA, porque o retrato sem a cadeia não a tem e a
//! tela também não).

use std::collections::BTreeMap;

use super::Vista;

/// A cor antes da cadeia e o ponto LOCAL da amostra (para a perguntar à vista
/// seguinte).
#[derive(Clone, Copy, Debug)]
struct Antes {
    cor: [f32; 3],
    p: [f32; 3],
}

/// ⭐⭐ **A base de uma cadeia de traços molhados** — ver o cabeçalho.
#[derive(Clone, Debug, Default)]
pub struct BaseDaCadeia {
    /// O tamanho do destino (amostras do plano, ou vértices) a que os índices
    /// se referem: um destino de outro tamanho não é esta cadeia.
    destino: usize,
    antes: BTreeMap<u32, Antes>,
}

impl BaseDaCadeia {
    /// Uma cadeia vazia sobre um destino de `destino` entradas.
    #[must_use]
    pub fn vazia(destino: usize) -> Self {
        Self {
            destino,
            antes: BTreeMap::new(),
        }
    }

    /// O tamanho do destino a que os índices se referem.
    #[must_use]
    pub fn destino(&self) -> usize {
        self.destino
    }

    /// Quantas amostras a cadeia já tocou.
    #[must_use]
    pub fn len(&self) -> usize {
        self.antes.len()
    }

    /// A cadeia ainda não tocou nada?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.antes.is_empty()
    }

    /// A amostra `idx` já é da cadeia?
    pub(crate) fn contem(&self, idx: u32) -> bool {
        self.antes.contains_key(&idx)
    }

    /// ⭐ **A base de `idx`** — a de antes da cadeia; na 1.ª vez que a cadeia a
    /// toca, `pre` (a cor antes deste traço) passa a sê-lo.
    pub(crate) fn base(&mut self, idx: u32, pre: [f32; 3], p: [f32; 3]) -> [f32; 3] {
        self.antes.entry(idx).or_insert(Antes { cor: pre, p }).cor
    }

    /// ⭐⭐ **Rodar a vista: fica na cadeia só o que a vista nova herdou.**
    /// `mapa` é o [`crate::tela_origem::origem`] da vista nova (por píxel dela,
    /// onde a de antes via o mesmo sítio). Uma amostra cujo píxel novo não tem
    /// origem sai: a água dela não viajou, e a tinta que pousou fica na peça.
    /// ⚠️ Uma amostra que a vista nova não mostra também sai — a água dela
    /// também não viaja (a grade da água vive nos píxeis do ecrã).
    pub fn so_o_que_a_vista_levou(&mut self, nova: &Vista, mapa: &[Option<[f32; 2]>]) {
        let (w, h) = nova.tamanho();
        let (wu, hu) = (w as usize, h as usize);
        if wu == 0 || hu == 0 || mapa.len() != wu * hu {
            self.antes.clear();
            return;
        }
        self.antes.retain(|_, a| {
            let Some([x, y]) = nova.ecra(a.p) else {
                return false;
            };
            // A orla da vista (`x == largura`) é do último píxel — a mesma
            // regra da `Tela`, que repete a borda.
            if !(x >= 0.0 && y >= 0.0 && x <= w as f32 && y <= h as f32) {
                return false;
            }
            let (i, j) = ((x as usize).min(wu - 1), (y as usize).min(hu - 1));
            mapa[j * wu + i].is_some()
        });
    }

    /// Troca, em `destino`, a cor viva de cada amostra da cadeia pela de antes
    /// dela — chamada DUAS vezes à volta de um retrato devolve tudo ao bit
    /// (cada troca é a sua própria inversa). Um destino de outro tamanho não é
    /// tocado, e devolve `false`.
    pub(crate) fn troca(&mut self, destino: &mut [[f32; 3]]) -> bool {
        if destino.len() != self.destino {
            return false;
        }
        for (&i, a) in &mut self.antes {
            if let Some(viva) = destino.get_mut(i as usize) {
                std::mem::swap(viva, &mut a.cor);
            }
        }
        true
    }
}
