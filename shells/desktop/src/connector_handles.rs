//! **As duas alças das pontas do conector** — o GESTO (os `impl App`, no módulo irmão).
//!
//! A REGRA (pura: onde a ponta encosta, o que acontece ao largar, onde um waypoint novo entra)
//! mora em [`ph2d_vec_entities::connector_handles`], e é lá que ela é testada.

#[path = "connector_handles_gesture.rs"]
mod gesture;
pub(crate) use gesture::view;
pub(crate) use ph2d_vec_entities::connector_handles::waypoint_view;
