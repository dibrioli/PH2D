//! ⭐ **F6: os dedos navegam pelas portas do RATO, nunca por uma 2.ª câmera.** O reconhecedor pede
//! `NavBegin`/`Pan`/`Zoom`/`NavEnd`; a ponte da shell (`touch_input.rs`) tem de os levar ao botão
//! do MEIO (pousar, arrastar, levantar) e à RODA (a pinça), e o zoom à volta dos dedos é um pan
//! pela porta do pan (`anchor_pan_px`) — não uma escrita directa na câmera.
//!
//! Mutações que têm de sangrar: um braço de `TouchOut::Nav*`/`Pan`/`Zoom` que some ou que escreva
//! `gfx.camera` à mão; a pinça que deixe de passar pelo `on_mouse_wheel`; a compensação que suma.

use std::fs;

fn bridge() -> String {
    fs::read_to_string("src/touch_input.rs").expect("src/touch_input.rs")
}

fn arm<'a>(src: &'a str, head: &str) -> &'a str {
    let i = src
        .find(head)
        .unwrap_or_else(|| panic!("o braço `{head}` sumiu da ponte do toque"));
    &src[i..(i + 420).min(src.len())]
}

#[test]
fn the_two_finger_pan_is_the_middle_button_drag() {
    let src = bridge();
    assert!(
        arm(&src, "TouchOut::NavBegin(").contains("ElementState::Pressed, MouseButton::Middle"),
        "pousar dois dedos nao pousa o botao do meio"
    );
    assert!(
        arm(&src, "TouchOut::Pan(").contains("self.on_cursor_moved("),
        "o ponto medio a andar nao anda o cursor"
    );
    assert!(
        arm(&src, "TouchOut::NavEnd").contains("ElementState::Released, MouseButton::Middle"),
        "levantar os dedos nao levanta o botao do meio"
    );
}

#[test]
fn the_pinch_is_a_wheel_notch_plus_a_pan_through_the_pan_door() {
    let src = bridge();
    assert!(
        arm(&src, "TouchOut::Zoom {").contains("self.pinch("),
        "a pinca nao chega ao braco dela"
    );
    let pinch = &src[src.find("fn pinch(").expect("o braço da pinça sumiu")..];
    assert!(
        pinch.contains("self.on_mouse_wheel(MouseScrollDelta::PixelDelta("),
        "a pinca nao entra pela roda"
    );
    assert!(
        pinch.contains("Camera2d::wheel_px_for(factor)"),
        "a pinca inventa uma lei em vez da inversa da roda"
    );
    assert!(
        pinch.contains("Camera2d::anchor_pan_px(") && pinch.contains("self.on_cursor_moved("),
        "o zoom a volta dos dedos nao e' um pan pela porta do pan"
    );
}

#[test]
fn the_touch_bridge_never_writes_the_camera_itself() {
    let src = bridge();
    for forbidden in [
        "camera.zoom(",
        "camera.center",
        "pan_screen_delta(",
        "height_world =",
    ] {
        assert!(
            !src.contains(forbidden),
            "a ponte do toque escreve a camera a mao (`{forbidden}`): e' uma 2.a camera"
        );
    }
}
