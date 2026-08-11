//! **O RELEVO DO DEPÓSITO** — a silhueta do pincel esculpe a espessura da tinta que ela deposita.
//!
//! ## Por que isto existe (Enio, 2026-08-10)
//!
//! *"Criar o Relief para a deposição do pigmento com Shape exatamente como faz Wet Paint: o depósito
//! de pigmento com pouca água é visto como relevo."*
//!
//! O emboss do Wet Paint é `((m_r−m_l)·0,5 + (m_d−m_u)·1,0)·k` — `dot(∇m, L)` sobre a MASSA de
//! pigmento, somado dentro da cor. Ele não transplanta, e o irmão [`super::emboss_probe`] mediu por
//! quê: **tinta digital opaca SATURA**, e um gradiente sobre um platô é zero. O que transplanta é a
//! PERGUNTA — *"o pigmento que assentou tem corpo?"* —, e a resposta aqui passa pela luz que o
//! [`super::impasto_shade`] já tem, que é o padrão-ouro do qual a lei do Wet Paint é a versão crua.
//!
//! ## Por que não é o impasto, embora escreva no mesmo plano
//!
//! O impasto escala a altura pelo RAIO (`derive_height`, `size_scale`), de propósito: a razão de
//! aspecto do domo tem de ficar constante, senão um pincel grande vira uma poça que a luz desenha
//! chata. **Um filme de pigmento não engrossa porque o pincel é maior** — e o preço de herdar aquela
//! lei está medido: pela rota do impasto o MESMO filme mede 14,04 no raio 5 e **96,39** no raio 40,
//! sete vezes; por esta, 14,04 / 14,46 / 13,68 / 16,13.
//!
//! ## Onde ele mora, e por que mudou de casa
//!
//! A primeira versão o chamou **Paint** e o pôs na seção **Paper**, ao lado do dente — no Wet Paint as
//! duas metades saem do mesmo checkbox. O smoke mudou isso: *"deveria ter nome mais adequado, deveria
//! ter sido colocado na seção de Shape"*. E a mudança de casa é o argumento: o número não é do
//! substrato, é da **silhueta que deposita**, que é o assunto daquela seção. O que ficou do parentesco
//! é a CALIBRAÇÃO — [`MAX_FILM_PX`] é ancorada em [`super::substrate_relief::MAX_TOOTH_PX`] porque um
//! filme de pigmento é da mesma ordem que o grão em que ele assenta.

use super::impasto_light::DEPTH_UNIT_PX;
use crate::tool::PainterTool;

/// **A espessura do filme de pigmento, em pixels de relevo, no Relief máximo.**
///
/// ⚠️ **Ancorada no dente, e não escolhida:** um filme de pigmento é *tinta sobre o papel*, não corpo
/// de tinta — a mesma ordem que o grão em que assenta ([`super::substrate_relief::MAX_TOOTH_PX`]).
/// Medido pela sonda [`super::film_probe`] com uma Shape listrada (o que o pedido diz: *"a deposição do
/// pigmento com Shape"*), o que o relevo acrescenta é:
///
/// | Relief | pior | média |
/// |---|---|---|
/// | 0,25 | 3,06 | 1,26 |
/// | 0,50 | 7,13 | 2,28 |
/// | 1,00 | **14,46** | 4,58 |
///
/// contra os ~23 níveis de excursão do papel sozinho: a mesma ordem, que é o que o pedido *"exatamente
/// como faz Wet Paint"* quer dizer (lá o emboss é somado À cor do pigmento, nunca uma segunda camada
/// por cima dele).
pub(super) const MAX_FILM_PX: f32 = 1.0;

impl PainterTool {
    /// **Relief do depósito** — quanta espessura a silhueta deixa na tinta que ela deposita.
    ///
    /// ⚠️ **O fan-out é o mesmo do papel** (`set_paper_field`): sem ele, pegar a Faca — que tem slot
    /// próprio — desligaria o relevo debaixo da obra por um gesto que não fala de depósito nenhum.
    ///
    /// ⚠️ **Ele é VIVO no último traço, e só sob *Adjust Last Stroke*.** O `refresh_live_relief`
    /// re-deriva a espessura a partir dos INGREDIENTES que o traço guardou, exatamente como o Depth do
    /// impasto; e ele mesmo pergunta pelo `impasto_live_edit()`, que nasce DESMARCADO desde
    /// 2026-07-19 (*tinta pronta fica pronta*). Sobre traços mais antigos o slider nunca volta atrás:
    /// o relevo deles já está na camada.
    pub fn set_shape_relief(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        self.paint.shape_relief = v;
        // A conversão para a unidade do plano de relevo entra AQUI, na fronteira — o buffer é medido em
        // CARGAS e o número do artista é em pixels de relevo, exatamente como o `tooth_loads` faz para o
        // dente ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
        let loads = v * MAX_FILM_PX / DEPTH_UNIT_PX;
        self.set_paper_field(|b| b.film_depth = loads);
        self.refresh_live_relief();
    }

    /// O Relief vigente do depósito — o painel lê por aqui.
    #[must_use]
    pub fn shape_relief(&self) -> f32 {
        self.paint.shape_relief
    }

    /// Roteia as duas rows do DEPÓSITO. Devolve `true` quando consumiu o evento.
    ///
    /// ⚠️ **O Shine cai no `set_impasto_shine`, o setter que já existia**, e essa é a metade que impede
    /// a segunda porta: a row do card **Material** e esta escrevem o MESMO
    /// `BrushSpec::impasto_shine`, pelo MESMO caminho (que faz o fan-out pelos slots de relevo e
    /// re-assa o material do último traço). Duas vistas, um valor.
    pub(crate) fn route_shape_deposit_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SHAPE_RELIEF => {
                self.set_shape_relief(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SHAPE_SHINE => {
                self.set_impasto_shine(*v as f32);
                true
            }
            _ => false,
        }
    }
}
