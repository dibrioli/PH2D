//! **As settings do PROJETO viajam no arquivo** (doc 88, D3) — irmão do
//! [`crate::project`] pelo teto de LOC (HR-18), e o corte é por assunto: aqui mora
//! *como a escala e a unidade do projeto sobrevivem a um save*.
//!
//! # O que estava errado
//!
//! `ProjectSettings` carrega a **escala do mundo** (`pixels_per_meter`), a
//! **unidade que o artista lê** (`display_unit`), os dois snaps do gizmo e o modo
//! de filtragem — e nada disso era gravado. Um projeto de pixel art afinado em
//! `32 px/m` reabria em `100`, e um artista que escolheu **Pixels** no menu
//! Settings reabria em Pixels só por coincidência (é o default). São knobs que
//! ESQUECEM, e desde a fronteira de display dos params de Motion a consequência
//! deixou de ser cosmética: `pixels_per_meter` é o número pelo qual toda row de
//! comprimento é lida, então perdê-lo muda os números na tela sem que ninguém
//! tenha tocado no documento.
//!
//! # Fica FORA do `ProjectState`, como `physics`/`motion`/`timeline`
//!
//! O `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não deve
//! rebobinar a escala do mundo nem a unidade de leitura — é o mesmo motivo que
//! mantém os outros três fora dele. O preço honesto é o mesmo: trocar a unidade
//! não entra na fila do Ctrl+Z.
//!
//! # Um tipo PRÓPRIO do arquivo, e não o de runtime
//!
//! ⚠️ Exatamente pela razão que o irmão [`crate::project_tokens`] já escreveu: a
//! `ph2d-editor-core` não depende de `serde` para o valor de runtime dela, e fazer
//! o formato do arquivo herdar o layout de um tipo de runtime é o que transforma
//! um refactor interno numa quebra de save. O custo desse espelho — uma segunda
//! lista de campos — é fechado pelo gate de round-trip, que compara os
//! `ProjectSettings` INTEIROS por `PartialEq`: um campo novo que o espelho não
//! carregue faz o teste falhar, em vez de deixar de persistir em silêncio.

use ph2d_editor::project::{DisplayUnit, ImageFilterMode, ProjectSettings};

/// As settings do projeto, na forma que o arquivo guarda.
///
/// Os dois enums viajam como **discriminante `u8`**, e não como o próprio tipo:
/// é o que permite acrescentar um modo no fim sem que o layout posicional do
/// postcard mude de significado.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedSettings {
    /// Pixels de imagem por metro de mundo — a escala do import E a régua pela
    /// qual toda row de comprimento do Motion é lida.
    pixels_per_meter: f32,
    /// Passo do snap de translação, em metros de mundo.
    snap_move_meters: f32,
    /// Passo do snap de rotação, em graus.
    snap_rotate_deg: f32,
    /// [`DisplayUnit`] como `u8` — a unidade em que o artista LÊ um comprimento.
    display_unit: u8,
    /// [`ImageFilterMode`] como `u8`.
    image_filter: u8,
}

/// A unidade a partir do byte guardado. **Porta única** da direção inversa.
///
/// ⚠️ Um byte que o enum não tem cai no default em vez de recusar o arquivo
/// inteiro — a mesma lei do `theme_from_u8` do irmão: quem recusa formato é o
/// `PROJECT_SCHEMA`, nunca um campo.
const fn display_unit_from_u8(b: u8) -> DisplayUnit {
    match b {
        0 => DisplayUnit::Meters,
        _ => DisplayUnit::Pixels,
    }
}

/// O modo de filtragem a partir do byte guardado — irmão do de cima.
const fn image_filter_from_u8(b: u8) -> ImageFilterMode {
    match b {
        0 => ImageFilterMode::PixelArt,
        _ => ImageFilterMode::Smooth,
    }
}

/// O que o save grava.
pub(crate) fn collect(p: ProjectSettings) -> SavedSettings {
    SavedSettings {
        pixels_per_meter: p.pixels_per_meter,
        snap_move_meters: p.snap_move_meters,
        snap_rotate_deg: p.snap_rotate_deg,
        display_unit: p.display_unit as u8,
        image_filter: p.image_filter as u8,
    }
}

/// **As settings do documento anterior morrem aqui, e as do arquivo entram.**
///
/// ⚠️ A escala passa pela porta que CLAMPA (`set_pixels_per_meter`), nunca pelo
/// campo cru: um arquivo com um número fora da faixa suportada — corrompido, ou
/// escrito por uma versão que ainda não a limitava — instalaria um valor ilegal
/// que a UI não consegue nem mostrar, e toda conversão de comprimento passaria a
/// sair dele. O clamp é a única porta desse número desde que ele existe.
pub(crate) fn install(dst: &mut ProjectSettings, saved: &SavedSettings) {
    dst.set_pixels_per_meter(saved.pixels_per_meter);
    dst.snap_move_meters = saved.snap_move_meters;
    dst.snap_rotate_deg = saved.snap_rotate_deg;
    dst.display_unit = display_unit_from_u8(saved.display_unit);
    dst.image_filter = image_filter_from_u8(saved.image_filter);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Todo campo sobrevive ao arquivo — e este gate é o que torna o espelho
    /// seguro.**
    ///
    /// A fixture põe TODO campo num valor diferente do de fábrica e compara os
    /// `ProjectSettings` INTEIROS depois da ida e volta. Acrescentar um campo ao
    /// tipo de runtime quebra a construção da fixture (o literal fica incompleto),
    /// o autor é obrigado a lhe dar um valor, e se o espelho não o carregar o
    /// `PartialEq` falha aqui — que é exatamente o modo de falha que um espelho
    /// escrito à mão teria em silêncio.
    #[test]
    fn every_setting_survives_the_round_trip() {
        let authored = ProjectSettings {
            pixels_per_meter: 32.0,
            snap_move_meters: 0.16,
            snap_rotate_deg: 15.0,
            display_unit: DisplayUnit::Meters,
            image_filter: ImageFilterMode::PixelArt,
        };
        assert_ne!(
            authored,
            ProjectSettings::default(),
            "a fixture igual ao default não prova travessia nenhuma"
        );

        let bytes = postcard::to_allocvec(&collect(authored)).expect("serializa");
        let back: SavedSettings = postcard::from_bytes(&bytes).expect("desserializa");
        let mut loaded = ProjectSettings::default();
        install(&mut loaded, &back);

        assert_eq!(
            loaded, authored,
            "algum campo não atravessou o arquivo — o espelho está incompleto"
        );
    }

    /// A escala entra pela porta que clampa. Um arquivo com um número absurdo dá
    /// um projeto utilizável, não um em que toda leitura de comprimento é lixo.
    #[test]
    fn a_corrupt_scale_is_clamped_on_the_way_in_not_installed_raw() {
        let mut ruined = ProjectSettings::default();
        ruined.set_pixels_per_meter(1.0e30);
        let bytes = postcard::to_allocvec(&SavedSettings {
            pixels_per_meter: 1.0e30,
            snap_move_meters: 0.0,
            snap_rotate_deg: 0.0,
            display_unit: 1,
            image_filter: 1,
        })
        .expect("serializa");
        let back: SavedSettings = postcard::from_bytes(&bytes).expect("desserializa");
        let mut loaded = ProjectSettings::default();
        install(&mut loaded, &back);
        assert_eq!(
            loaded.pixels_per_meter, ruined.pixels_per_meter,
            "a escala tem de entrar pela mesma porta que a UI usa"
        );
        assert!(loaded.pixels_per_meter.is_finite());
    }

    /// Um byte que nenhum enum reconhece cai no default em vez de recusar o
    /// projeto. Recusar o arquivo inteiro por causa de um modo desconhecido é a
    /// resposta errada — quem recusa formato é o `PROJECT_SCHEMA`.
    #[test]
    fn an_unknown_enum_byte_falls_back_instead_of_rejecting_the_file() {
        assert_eq!(display_unit_from_u8(200), DisplayUnit::Pixels);
        assert_eq!(image_filter_from_u8(200), ImageFilterMode::Smooth);
        // E os bytes que os enums TÊM continuam mapeando para si mesmos — sem
        // isto o teste acima passaria com a função devolvendo sempre o default.
        assert_eq!(display_unit_from_u8(0), DisplayUnit::Meters);
        assert_eq!(image_filter_from_u8(0), ImageFilterMode::PixelArt);
    }
}
