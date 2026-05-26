//! Runtime collections of importers/exporters. **Not part of the
//! arch-gated contract surface** — these are simple `Vec`-backed
//! registries that grow as `ph2d-imageio-<slug>::register_importer` and
//! `::register_exporter` calls push into them at boot.
//!
//! Why here and not in `ph2d-imageio-registry-init`: the registry types
//! belong to the foundation crate (mirror of `ph2d_tool_registry::Registry`
//! sitting next to `ToolManifest`). `ph2d-imageio-registry-init` only
//! owns the codegen'd `register_all_*` functions; the *types* live with
//! the traits they collect.

use crate::{ExportOpts, ImageExporter, ImageImporter, MagicHint};

/// Boot-time collection of [`ImageImporter`]s. Built by
/// `ph2d_imageio_registry_init::register_all_importers` (codegen'd by
/// `tools/ph2d-imageio-sync`).
///
/// Lookup is linear — N stays in the low double digits (16 formats
/// in v1) so `O(N)` is fine, and avoids the `HashMap` ban (ADR-0022).
#[derive(Default)]
pub struct ImporterRegistry {
    importers: Vec<Box<dyn ImageImporter>>,
}

impl ImporterRegistry {
    /// Empty registry. Use [`Self::register`] to populate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an importer. Order doesn't affect correctness (first match
    /// wins in [`Self::find_for`]), but the codegen emits registrations
    /// in `ls`-sorted order so two builds produce identical state.
    pub fn register(&mut self, importer: Box<dyn ImageImporter>) {
        self.importers.push(importer);
    }

    /// First importer whose [`ImageImporter::supports`] returns `true`
    /// for `hint`. `None` = no recognized format.
    #[must_use]
    pub fn find_for(&self, hint: MagicHint<'_>) -> Option<&dyn ImageImporter> {
        self.importers
            .iter()
            .find(|i| i.supports(hint))
            .map(std::convert::AsRef::as_ref)
    }

    /// Total number of registered importers — used by the staleness
    /// gate (count = `crates/ph2d-imageio-*` count).
    #[must_use]
    pub fn len(&self) -> usize {
        self.importers.len()
    }

    /// `true` if no importers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.importers.is_empty()
    }
}

/// Boot-time collection of [`ImageExporter`]s. Symmetric to
/// [`ImporterRegistry`].
#[derive(Default)]
pub struct ExporterRegistry {
    exporters: Vec<Box<dyn ImageExporter>>,
}

impl ExporterRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an exporter.
    pub fn register(&mut self, exporter: Box<dyn ImageExporter>) {
        self.exporters.push(exporter);
    }

    /// First exporter whose [`ImageExporter::supports_format`] returns
    /// `true` for `opts.format`. `None` = no exporter handles that
    /// format.
    #[must_use]
    pub fn find_for(&self, opts: &ExportOpts) -> Option<&dyn ImageExporter> {
        self.exporters
            .iter()
            .find(|e| e.supports_format(&opts.format))
            .map(std::convert::AsRef::as_ref)
    }

    /// Total number of registered exporters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.exporters.len()
    }

    /// `true` if no exporters are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.exporters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DecodedImage, Error, ExportOpts, ImportOpts};

    struct AlwaysYesImporter;
    impl ImageImporter for AlwaysYesImporter {
        fn supports(&self, _hint: MagicHint<'_>) -> bool {
            true
        }
        fn import(&self, _src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
            Err(Error::Decode("test stub".into()))
        }
    }

    #[test]
    fn empty_importer_registry_finds_nothing() {
        let reg = ImporterRegistry::new();
        assert!(reg.is_empty());
        assert!(reg.find_for(MagicHint::Extension("png")).is_none());
    }

    #[test]
    fn registered_importer_is_findable() {
        let mut reg = ImporterRegistry::new();
        reg.register(Box::new(AlwaysYesImporter));
        assert_eq!(reg.len(), 1);
        assert!(reg.find_for(MagicHint::Extension("anything")).is_some());
    }

    struct PngLikeExporter;
    impl ImageExporter for PngLikeExporter {
        fn supports_format(&self, fmt: &str) -> bool {
            fmt == "png"
        }
        fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
            Ok(vec![0x89, b'P', b'N', b'G'])
        }
    }

    #[test]
    fn exporter_registry_dispatches_by_format() {
        let mut reg = ExporterRegistry::new();
        reg.register(Box::new(PngLikeExporter));
        let opts_png = ExportOpts {
            format: "png".into(),
            ..ExportOpts::default()
        };
        let opts_jpeg = ExportOpts {
            format: "jpeg".into(),
            ..ExportOpts::default()
        };
        assert!(reg.find_for(&opts_png).is_some());
        assert!(reg.find_for(&opts_jpeg).is_none());
    }
}
