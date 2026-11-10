//! P13 T: as linhas de som da Timeline (o painel pede, a família aplica: um passo cada).
use ph2d_pixel_audio::{AudioEngine, canal as c, motor as m};

impl crate::PixelApp {
    /// Uma vez por quadro, depois do tocar.
    pub fn som(&mut self) {
        let key = self.chave.unwrap_or(u32::MAX);
        for p in c::toma() {
            let tipo = ph2d_i18n::tr("panel.pixel.timeline.audio_kind");
            let f = rfd::FileDialog::new().add_filter(tipo, &c::EXTENSOES);
            let escolhe = || ph2d_app_host::modal::pick_file(f);
            let op = (self.docs.get(&key)).and_then(|d| c::op(&d.sprite, p, escolhe));
            self.passo_nomeado(key, op, "pixel.history.palette_data");
        }
        c::publica(self.docs.get(&key).map(|d| &d.sprite), None);
    }

    /// O som do Play no mixer do host, antes de o Play andar; devolve o `dt` dele.
    pub fn som_do_play(&mut self, mixer: Option<&mut AudioEngine>, dt: u32) -> u32 {
        let d = self.chave.and_then(|k| Some((k, self.docs.get(&k)?)));
        let v =
            d.map(|(k, d)| m::Vista::de(k, &d.sprite, d.frame, d.play.as_ref().map(|p| p.index)));
        m::quadro(mixer, v, dt)
    }
}
