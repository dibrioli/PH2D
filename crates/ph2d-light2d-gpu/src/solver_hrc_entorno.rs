//! ⭐ O entorno da holográfica (W5d; bug 6, handoff W6g §32–§33): o miolo que um ENTORNO não recolhe e a
//! janela do arquivo dele; o entorno de uma VISTA (a soma da c0 ou a borda). Módulo-filho de `solver_hrc.rs`.

use super::*;

/// ⭐ bug 6 (handoff W6g §32) — o entorno visto pela borda: o arquivo dos R dele, a grade da holográfica
/// dele, as janelas e as bases dos níveis nelas.
pub(crate) struct Fora<'a> {
    pub arquivo: &'a wgpu::Buffer,
    pub grade: [u32; 2],
    pub janelas: [hrc::Janela; 4],
    pub bases: [[u32; hrc::NIVEIS_MAX]; 4],
}

impl Holografica {
    /// ⭐ W5d — o miolo (texels da grade) cuja emissão este solver não recolhe; `None` = nenhum.
    /// ⭐ §34.3 (handoff W6g): pela borda só a 2.ª passagem (o arquivo, o miolo VÁCUO) o usa — a 1.ª é a
    /// fonte do rebote dos refletores de fora, que recebem também a luz que a vista emite e devolve.
    pub(crate) fn miolo(&mut self, queue: &wgpu::Queue, miolo: Option<[f32; 4]>) {
        let m = miolo.unwrap_or(SEM_MIOLO);
        let borda = self.janela_max.is_some();
        let janelas = self
            .janela_max
            .map(|max| hrc::janelas_do_miolo(self.grade, m, max));
        if self
            .uniformes
            .iter()
            .find(|u| !borda || u.arquivo[3] == 1)
            .is_some_and(|u| u.miolo == m)
            && janelas.is_none_or(|j| j == self.janelas)
        {
            return;
        }
        if let Some(j) = janelas {
            self.janelas = j;
            self.bases = hrc::bases_na_janela(self.grade, j).0;
        }
        for u in &mut self.uniformes {
            u.miolo = if borda && u.arquivo[3] == 0 {
                SEM_MIOLO
            } else {
                m
            };
            if self.janela_max.is_some() {
                let q = u.quadrante as usize;
                u.janela = self.janelas[q];
                if u.arquivo[1] == 1 {
                    u.arquivo[0] = self.bases[q][u.nivel as usize];
                }
            }
        }
        self.escrever(queue);
    }

    /// ⭐ W5d — o entorno: a vista da c0 dele e `desloc, escala` do texel desta grade para o dele; `None` =
    /// nenhum. ⭐ bug 6: com o [`Fora`] (o arquivo dos R dele), a luz de fora entra pela BORDA — o R dele no
    /// lugar do céu — e a c0 NÃO se soma na saída; sem ele, a soma da W5d.
    pub(crate) fn entorno(
        &mut self,
        device: crate::Placa<'_>,
        queue: &wgpu::Queue,
        entorno: Option<(&wgpu::TextureView, [f32; 3], Option<Fora<'_>>)>,
    ) {
        let (vista, e, fora) = match entorno {
            Some((v, [dx, dy, s], fora)) => {
                let soma = if fora.is_some() { 0.0 } else { 1.0 };
                (v.clone(), [dx, dy, s, soma], fora)
            }
            None => (self.vazia.clone(), [0.0; 4], None),
        };
        let arquivo = fora.as_ref().map_or(&self.arquivo, |f| f.arquivo);
        self.g2 = g2_de(device, &self.g2_layout, &vista, &self.amostra, arquivo);
        type PorQuadrante = ([u32; 4], [u32; 4], [u32; 4], [f32; 4], [[u32; 4]; 5]);
        let [dx, dy, s, _] = e;
        let (wv, we) = (
            self.grade.map(|n| n as f32),
            fora.as_ref().map(|f| f.grade.map(|n| n as f32)),
        );
        let por_quadrante: [PorQuadrante; 4] = std::array::from_fn(|q| {
            let (Some(f), Some(we)) = (&fora, we) else {
                return ([0; 4], [0; 4], [0; 4], [0.0; 4], [[0; 4]; 5]);
            };
            let quad = Quadrante::de(q as u32, f.grade);
            let vista = Quadrante::de(q as u32, self.grade);
            // O nível mais fino em ângulo cuja coluna de sondas cabe dentro desta grade (o `de_fora`).
            let cabe = (vista.x.min(vista.y) as f32 * s).max(1.0).log2().floor() as u32;
            // O canónico desta → o do entorno: `c_e = c · s + af` (as rotações do `na_grade` compostas).
            let af = match q {
                1 => [dy, we[0] - s * wv[0] - dx],
                2 => [we[0] - s * wv[0] - dx, we[1] - s * wv[1] - dy],
                3 => [we[1] - s * wv[1] - dy, dx],
                _ => [dx, dy],
            };
            let b = f.bases[q];
            (
                [0; 4],
                [quad.x, quad.y, (quad.n - 1).min(cabe), 1],
                f.janelas[q],
                [af[0], af[1], s, 0.0],
                std::array::from_fn(|k| std::array::from_fn(|i| b[4 * k + i])),
            )
        });
        let mut mudou = false;
        for u in &mut self.uniformes {
            let (a, fo, ja, af, fb) = por_quadrante[u.quadrante as usize];
            // A 2.ª passagem de um ENTORNO (`arquivo.w = 1`) é dele: ninguém a reescreve.
            let a = if u.arquivo[3] == 1 { u.arquivo } else { a };
            let novo = (e, a, fo, ja, af, fb);
            if (
                u.entorno,
                u.arquivo,
                u.fora,
                u.janela,
                u.fora_af,
                u.fora_base,
            ) != novo
            {
                (
                    u.entorno,
                    u.arquivo,
                    u.fora,
                    u.janela,
                    u.fora_af,
                    u.fora_base,
                ) = novo;
                mudou = true;
            }
        }
        if mudou {
            self.escrever(queue);
        }
    }
}
