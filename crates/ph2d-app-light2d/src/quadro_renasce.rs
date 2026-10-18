//! ⭐ W6b — **a vista renasce quando a PLACA a degrada** ([handoff W6b](../../../docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W6b_2026-10-17_A_VISTA_RENASCE.md)).
//! Um evento interno do driver (1× por processo) deixa a `c2` da vista ~+0,9 ms até ela renascer; depois dele
//! todo nascimento é rápido. O vigia lê a `c2` que o governador já lê a cada quadro e
//! reconhece a ASSINATURA: uma leitura ≥ [`PICO`]× o piso da geração e, nas [`LEITURAS_DEPOIS`] seguintes, o
//! mínimo ≥ piso + max([`MARGEM_MS`], [`MARGEM_REL`]·piso). No máximo [`MAX_POR_PROCESSO`] vezes; o nascimento
//! é determinístico (os bits não mudam). A vista velha SOLTA-SE antes de a nova montar: montada com a velha viva
//! no driver, a nova ia para blocos novos do alocador (12 → 16–17) e ficava +0,15–0,25 ms (W6b §2–3). O renascimento
//! é no INÍCIO do quadro seguinte ao pedido (nenhum quadro sai sem luz), com os receptores já soltos por quem chama
//! ([`super::LuzDoQuadro::soltar_se_renasce`]): presos, seguravam a fluência velha e a nova ia a 13 blocos
//! ([handoff W6c](../../../docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md)).
//! `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0` desliga (a régua ON × OFF).

/// A leitura que DISPARA a vigília: a do evento vai a ~2,6× e ~4,7× o piso (W6a §2).
const PICO: f64 = 2.0;
/// Quantas leituras depois do pico decidem se a vista ficou degradada.
const LEITURAS_DEPOIS: u32 = 30;
/// A degradação mínima, absoluta e relativa ao piso (a do evento: +0,85 a +1,0 ms sobre ~11,5).
const MARGEM_MS: f64 = 0.3;
const MARGEM_REL: f64 = 0.03;
/// O tecto de renascimentos: o evento é 1× por processo; o 2.º cobre um falso positivo.
const MAX_POR_PROCESSO: u32 = 2;

/// O vigia da `c2` da vista ([`Self::ler`] a cada leitura de custo, [`Self::nasceu`] a cada montagem).
#[derive(Debug, Default)]
pub(super) struct Vigia {
    ligado: bool,
    /// O mínimo da `c2` desde o último nascimento.
    piso: Option<f64>,
    /// Vigília aberta: `(piso no pico, leituras que faltam, mínimo delas)`.
    vigilia: Option<(f64, u32, f64)>,
    renascidas: u32,
    /// O renascimento pedido (pelo vigia, ou pelo portão dos bits: [`super::LuzDoQuadro::renascer_a_vista`]).
    pedido: bool,
}

impl Vigia {
    /// Ligado salvo `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0` (lida em execução: o MESMO binário mede os dois).
    pub(super) fn do_ambiente() -> Self {
        let desligado =
            std::env::var("PH2D_LIGHT2D_RENASCE_SE_DEGRADA").is_ok_and(|v| v.trim() == "0");
        Self::com(!desligado)
    }

    pub(super) fn com(ligado: bool) -> Self {
        Self {
            ligado,
            ..Self::default()
        }
    }

    /// Um solver novo da vista: outra geração, outro piso.
    pub(super) fn nasceu(&mut self) {
        self.piso = None;
        self.vigilia = None;
    }

    /// Uma leitura da `c2` (ms). `true` = a vista renasce no próximo quadro ([`Self::renascer`]).
    pub(super) fn ler(&mut self, c2: f64) -> bool {
        if !self.ligado || self.renascidas >= MAX_POR_PROCESSO || !c2.is_finite() {
            return false;
        }
        if let Some((piso, faltam, minimo)) = self.vigilia.as_mut() {
            *minimo = minimo.min(c2);
            *faltam -= 1;
            if *faltam > 0 {
                return false;
            }
            let (piso, minimo) = (*piso, *minimo);
            self.vigilia = None;
            if minimo < piso + MARGEM_MS.max(MARGEM_REL * piso) {
                return false;
            }
            self.renascidas += 1;
            eprintln!(
                "[luz2d] a placa degradou a vista: c2 piso {piso:.2} → mínimo {minimo:.2} ms nas {LEITURAS_DEPOIS} \
                 leituras depois do pico · renasce ({}/{MAX_POR_PROCESSO})",
                self.renascidas
            );
            self.pedido = true;
            return true;
        }
        match self.piso {
            Some(piso) if c2 >= PICO * piso => {
                self.vigilia = Some((piso, LEITURAS_DEPOIS, f64::INFINITY));
            }
            Some(piso) => self.piso = Some(piso.min(c2)),
            None => self.piso = Some(c2),
        }
        false
    }

    /// O início do quadro seguinte ao pedido: a vista larga-se, a placa espera destruí-la e o MESMO quadro monta a
    /// nova nos mesmos blocos (com os receptores soltos: [`super::LuzDoQuadro::soltar_se_renasce`]).
    pub(super) fn renascer(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        vista: &mut Option<super::Resolvido>,
    ) {
        if std::mem::take(&mut self.pedido) {
            *vista = None;
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
    }
}

impl super::LuzDoQuadro {
    /// ⭐ W6b — PORTÃO: a vista renasce no próximo [`Self::gravar_com`] pelo MESMO caminho do vigia
    /// (sem o evento do driver, que não se provoca daqui).
    #[doc(hidden)]
    pub fn renascer_a_vista(&mut self) {
        self.vigia.pedido = true;
    }

    /// ⭐ W6c — ANTES do [`Self::gravar_com`]: havendo renascimento pedido, os receptores largam a fluência velha
    /// (o `ligar` deste quadro acende-os com a nova). `true` = solte também o blit do Flip.
    #[must_use]
    pub fn soltar_se_renasce(
        &self,
        gpu: &ph2d_gpu::GpuContext,
        renderer: &mut ph2d_render::SpriteRenderer,
        colagem: &mut ph2d_render::BandBlit,
    ) -> bool {
        let solta = self.vigia.pedido;
        if solta {
            renderer.receber_luz(None);
            colagem.receber_luz(gpu, None);
        }
        solta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O vigia ligado depois de `n` leituras a `piso`.
    fn assente(piso: f64, n: usize) -> Vigia {
        let mut v = Vigia::com(true);
        for _ in 0..n {
            assert!(!v.ler(piso));
        }
        v
    }

    /// A assinatura do evento (W6a §2): o pico a ~30 e ~54 ms (2,6× e 4,7× o piso), e a vista +0,9 ms depois.
    fn evento(v: &mut Vigia, piso: f64, depois: f64) -> bool {
        let mut serie = vec![2.6 * piso, 4.7 * piso];
        serie.extend(std::iter::repeat_n(depois, LEITURAS_DEPOIS as usize));
        // Dispara na leitura 30 depois do pico, nunca antes.
        let disparos: Vec<usize> = serie
            .iter()
            .enumerate()
            .filter(|&(_, &c2)| v.ler(c2))
            .map(|(i, _)| i)
            .collect();
        assert!(disparos.len() <= 1, "disparou {disparos:?}");
        if let Some(&i) = disparos.first() {
            assert_eq!(i, LEITURAS_DEPOIS as usize, "disparou na leitura {i}");
        }
        !disparos.is_empty()
    }

    #[test]
    fn o_evento_do_driver_faz_a_vista_renascer() {
        let mut v = assente(11.5, 5);
        assert!(evento(&mut v, 11.5, 12.4));
    }

    #[test]
    fn um_pico_sem_degradacao_nao_renasce() {
        // A contenção solta: o pico e a vista de volta ao piso (+0,2 < 0,3 ms).
        let mut v = assente(11.5, 5);
        assert!(!evento(&mut v, 11.5, 11.7));
        // Um único quadro a voltar ao piso na vigília basta para não renascer.
        let mut v = assente(11.5, 5);
        assert!(!v.ler(30.0));
        for i in 0..LEITURAS_DEPOIS {
            assert!(!v.ler(if i == 17 { 11.5 } else { 12.4 }));
        }
    }

    #[test]
    fn a_degradacao_sem_pico_nao_renasce() {
        // A cena mais cara (o artista acrescentou luz): sobe sem a paragem.
        let mut v = assente(11.5, 5);
        for _ in 0..200 {
            assert!(!v.ler(12.4));
        }
    }

    #[test]
    fn a_margem_e_a_maior_das_duas() {
        // Piso alto: os 3 % (0,6 ms) mandam sobre os 0,3 ms.
        let mut v = assente(20.0, 5);
        assert!(!evento(&mut v, 20.0, 20.5));
        let mut v = assente(20.0, 5);
        assert!(evento(&mut v, 20.0, 20.7));
        // Piso baixo: os 0,3 ms mandam sobre os 3 % (0,15 ms).
        let mut v = assente(5.0, 5);
        assert!(!evento(&mut v, 5.0, 5.25));
    }

    #[test]
    fn renasce_no_maximo_duas_vezes_e_so_ligado() {
        let mut v = assente(11.5, 5);
        for vez in 0..4 {
            let renasceu = evento(&mut v, 11.5, 12.4);
            assert_eq!(renasceu, vez < MAX_POR_PROCESSO, "vez {vez}");
            v.nasceu();
            for _ in 0..5 {
                assert!(!v.ler(11.5));
            }
        }
        let mut v = Vigia::com(false);
        for _ in 0..5 {
            assert!(!v.ler(11.5));
        }
        assert!(!evento(&mut v, 11.5, 12.4));
    }

    #[test]
    fn cada_nascimento_tem_o_seu_piso() {
        // A grade nova (outro nível) custa outra coisa: o piso velho não dispara nada.
        let mut v = assente(4.0, 5);
        v.nasceu();
        for _ in 0..100 {
            assert!(!v.ler(11.5));
        }
        // Sem leitura nenhuma depois do nascimento não há piso: o 1.º valor é o piso.
        let mut v = Vigia::com(true);
        assert!(!v.ler(30.0));
        assert!(!v.ler(12.4));
    }
}
