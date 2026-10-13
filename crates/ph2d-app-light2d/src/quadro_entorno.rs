//! O ENTORNO da luz do quadro (W5d, W5i) — ficheiro irmão do `quadro.rs` pelo tecto de 700 LOC (W5j).

use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{Config, Solver};

use crate::grade::{Degrau, MARGEM_DO_ENTORNO, grade_do_quadro};

impl super::LuzDoQuadro {
    /// ⭐ W5d — o ENTORNO deste quadro: criado na grade dele ([`Degrau::entorno`] × o texel,
    /// [`MARGEM_DO_ENTORNO`] de cada lado), reposicionado se ela só andou, apagado sem ele (o degrau, a
    /// holográfica, `PH2D_LIGHT2D_ENTORNO=0` para comparar).
    /// ⭐ W5i — a configuração do ENTORNO nesta vista (`None` = sem ele).
    pub(super) fn config_do_entorno(&self, gpu: &GpuContext) -> Option<Config> {
        let holografica = self
            .degrau
            .holografica
            .then_some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO);
        let ligado = self.degrau.entorno > 0.0
            && holografica.is_some()
            && std::env::var("PH2D_LIGHT2D_ENTORNO").map_or(true, |v| v.trim() != "0");
        let de = Degrau {
            c0_px: self.degrau.c0_px * self.degrau.entorno,
            margem: MARGEM_DO_ENTORNO,
            ..self.degrau
        };
        let ge = ligado
            .then(|| grade_do_quadro(self.mundo_para_tela, self.alvo_px, de))
            .flatten()?;
        Some(Config {
            holografica,
            rebote: super::rebote_ligado(),
            // ⭐ W5o: da vista só se lê a c0 do entorno (`Solver::somar_entorno`).
            so_a_c0: super::cortes_ligados(),
            hrc_nasce_duas_vezes: std::env::var("PH2D_LIGHT2D_HRC_DUAS")
                .map_or(true, |v| v.trim() != "0"),
            ..Config::padrao_para(&gpu.device, ge)
        })
    }

    pub(super) fn preparar_entorno(&mut self, gpu: &GpuContext) {
        let Some(cfg) = self.config_do_entorno(gpu) else {
            self.entorno = None;
            return;
        };
        let ge = cfg.grade;
        let mesma = self.entorno.as_ref().is_some_and(|e| {
            let c = e.config();
            (c.grade.largura, c.grade.altura) == (ge.largura, ge.altura)
        });
        if mesma {
            let e = self.entorno.as_mut().expect("verificado acima");
            if e.reposicionar(ge.origem_m, ge.texel_m).is_err() {
                self.entorno = None;
            }
            return;
        }
        // ⭐ W5i: a frio, à estufa — a vista acende sem a luz de fora até ele ficar pronto.
        if !self.forno.pronta(&cfg) {
            self.forno.pedir(cfg);
            self.forno.andar(gpu, &self.fabrica);
            self.entorno = None;
            return;
        }
        self.entorno = Solver::novo_com(gpu, cfg, &self.fabrica)
            .map(|mut s| {
                // Só as cascatas e a fonte do rebote: a grade fina dele não se lê.
                s.regiao_fina([0; 2], [0; 2]);
                s
            })
            .map_err(|e| eprintln!("[luz2d] o entorno recusou a grade {ge:?}: {e:?}"))
            .ok();
    }
}
