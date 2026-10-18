//! O `Dispatch` de cada objecto do protocolo: a tradução 1:1 para [`ProtocolEvent`] (filho de
//! `wayland`; a lei está na `Sessao`).

use super::*;

impl Dispatch<ZwpTabletToolV2, ()> for Worker {
    fn event(
        w: &mut Self,
        proxy: &ZwpTabletToolV2,
        event: zwp_tablet_tool_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use zwp_tablet_tool_v2::Event as E;
        let id = proxy.id().protocol_id();
        let ev = match event {
            E::Type { tool_type } => {
                let code = match tool_type {
                    WEnum::Value(t) => t as u32,
                    WEnum::Unknown(c) => c,
                };
                if let Some(t) = w.tools.get_mut(&id) {
                    t.kind = ToolKind::from_protocol_type(code);
                }
                return;
            }
            E::ProximityIn {
                serial, surface, ..
            } => {
                let ours = surface.id().protocol_id() == w.surface;
                let Some(t) = w.tools.get_mut(&id) else {
                    return;
                };
                t.entrou = ours.then_some(serial);
                let kind = t.kind;
                if ours {
                    w.aplica_cursor(id, qh);
                }
                ProtocolEvent::ProximityIn {
                    on_our_surface: ours,
                    kind,
                }
            }
            E::ProximityOut => {
                if let Some(t) = w.tools.get_mut(&id) {
                    t.entrou = None;
                }
                ProtocolEvent::ProximityOut
            }
            E::Down { .. } => ProtocolEvent::Down,
            E::Up => ProtocolEvent::Up,
            E::Motion { x, y } => ProtocolEvent::Motion { x, y },
            E::Pressure { pressure } => ProtocolEvent::Pressure(pressure),
            E::Distance { distance } => ProtocolEvent::Distance(distance),
            E::Tilt { tilt_x, tilt_y } => ProtocolEvent::Tilt {
                x: tilt_x,
                y: tilt_y,
            },
            E::Button { button, state, .. } => ProtocolEvent::Button {
                code: button,
                pressed: matches!(
                    state,
                    WEnum::Value(zwp_tablet_tool_v2::ButtonState::Pressed)
                ),
            },
            E::Frame { time } => ProtocolEvent::Frame { time_ms: time },
            E::Removed => {
                w.sessao.remove(u64::from(id), &mut w.buf);
                if let Some(f) = w.tools.remove(&id).and_then(|t| t.forma) {
                    f.destroy();
                }
                proxy.destroy();
                return;
            }
            // hardware_serial, hardware_id_wacom, capability, done, rotation, slider, wheel
            _ => return,
        };
        w.feed(id, ev);
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for Worker {
    fn event(
        w: &mut Self,
        _: &ZwpTabletSeatV2,
        event: zwp_tablet_seat_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_tablet_seat_v2::Event::ToolAdded { id } => {
                w.tools.insert(
                    id.id().protocol_id(),
                    Ferramenta {
                        proxy: id,
                        kind: ToolKind::Pen,
                        forma: None,
                        entrou: None,
                    },
                );
            }
            zwp_tablet_seat_v2::Event::TabletAdded { id } => w.tablets.push(id),
            zwp_tablet_seat_v2::Event::PadAdded { id } => w.pads.push(id),
            _ => {}
        }
    }

    event_created_child!(Worker, ZwpTabletSeatV2, [
        zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (ZwpTabletV2, ()),
        zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (ZwpTabletToolV2, ()),
        zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (ZwpTabletPadV2, ()),
    ]);
}

impl Dispatch<ZwpTabletV2, ()> for Worker {
    fn event(
        w: &mut Self,
        proxy: &ZwpTabletV2,
        event: zwp_tablet_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwp_tablet_v2::Event::Removed = event {
            w.tablets.retain(|t| t != proxy);
            proxy.destroy();
        }
    }
}

/// Os botões e o anel da MESA ficam fora do ADR-0189 (lista 26): só se vive com eles.
impl Dispatch<ZwpTabletPadV2, ()> for Worker {
    fn event(
        w: &mut Self,
        proxy: &ZwpTabletPadV2,
        event: zwp_tablet_pad_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_tablet_pad_v2::Event::Group { pad_group } => {
                w.pad_parts.push(Box::new(move || pad_group.destroy()));
            }
            zwp_tablet_pad_v2::Event::Removed => {
                w.pads.retain(|p| p != proxy);
                proxy.destroy();
            }
            _ => {}
        }
    }

    event_created_child!(Worker, ZwpTabletPadV2, [
        zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (ZwpTabletPadGroupV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadGroupV2, ()> for Worker {
    fn event(
        w: &mut Self,
        _: &ZwpTabletPadGroupV2,
        event: zwp_tablet_pad_group_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_tablet_pad_group_v2::Event::Ring { ring } => {
                w.pad_parts.push(Box::new(move || ring.destroy()));
            }
            zwp_tablet_pad_group_v2::Event::Strip { strip } => {
                w.pad_parts.push(Box::new(move || strip.destroy()));
            }
            zwp_tablet_pad_group_v2::Event::Dial { dial } => {
                w.pad_parts.push(Box::new(move || dial.destroy()));
            }
            _ => {}
        }
    }

    // O `dial` é da versão 2: capados na 1 nunca chega, mas fica declarado.
    event_created_child!(Worker, ZwpTabletPadGroupV2, [
        zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (ZwpTabletPadRingV2, ()),
        zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (ZwpTabletPadStripV2, ()),
        zwp_tablet_pad_group_v2::EVT_DIAL_OPCODE => (ZwpTabletPadDialV2, ()),
    ]);
}

/// Os objectos sem nada a ler: só existem para a fila os poder despachar.
macro_rules! ignora {
    ($($t:ty),* $(,)?) => {$(
        impl Dispatch<$t, ()> for Worker {
            fn event(
                _: &mut Self,
                _: &$t,
                _: <$t as Proxy>::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }
    )*};
}

ignora!(
    WlSeat,
    ZwpTabletManagerV2,
    ZwpTabletPadRingV2,
    ZwpTabletPadStripV2,
    ZwpTabletPadDialV2,
    WpCursorShapeManagerV1,
    WpCursorShapeDeviceV1,
);

/// Assentos que aparecem ou somem depois do arranque.
impl Dispatch<WlRegistry, GlobalListContents> for Worker {
    fn event(
        w: &mut Self,
        reg: &WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } if interface == WlSeat::interface().name => w.liga_assento(reg, name, version, qh),
            wl_registry::Event::GlobalRemove { name } => {
                if let Some(i) = w.assentos.iter().position(|a| a.nome == name) {
                    let a = w.assentos.remove(i);
                    a.tablet_seat.destroy();
                    if a.seat.version() >= 5 {
                        a.seat.release();
                    }
                }
            }
            _ => {}
        }
    }
}
