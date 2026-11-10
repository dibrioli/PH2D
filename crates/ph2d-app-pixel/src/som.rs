//! P13 T: as linhas de som da Timeline (o painel pede, a família aplica: um passo cada).
use ph2d_pixel_audio::canal as c;

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
}
