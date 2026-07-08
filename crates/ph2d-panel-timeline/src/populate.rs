//! Timeline panel widget registration (called once at panel install).
//!
//! W2.E0 registers no interactive widgets — the transport buttons, chips, ruler
//! and key lanes land in W2.E2+. The close (X) button is painted directly on the
//! chrome via the hit-index (not a store widget), so nothing is registered here
//! yet.

use ph2d_editor_core::interaction::WidgetStore;

pub(crate) fn populate(_store: &mut WidgetStore) {
    // Intentionally empty until W2.E2 (transport + lanes).
}
