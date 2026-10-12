//! **Os registos de um elemento** ([ADR-0181]) — a unidade da fusão ao vivo e do desfazer que não pisa
//! a edição alheia. Duas mudanças a registos DIFERENTES do mesmo elemento ficam as duas; ao mesmo
//! registo, ganha uma inteira. A caixa `x, y, w, h` é UM registo: mexer e redimensionar pelo canto
//! escrevem-na junta e não se podem misturar.
//!
//! O valor de um registo viaja e compara-se em bytes `postcard` (exactos: um `f64` pelos bits).
//! `id`, `version` e `nonce` não são registos (o id é a chave; os outros dois derivam dos carimbos).
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Element, ElementKind, Org};

/// Um registo de um [`Element`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Field {
    Box,
    Angle,
    Z,
    Frame,
    Deleted,
    Fill,
    Stroke,
    StrokeWidth,
    Dash,
    Round,
    Opacity,
    TextColor,
    FontSize,
    Sketch,
    FillStyle,
    Cartoonist,
    Align,
    VAlign,
    Letter,
    ShapeType,
    Text,
    Start,
    End,
    Route,
    HeadStart,
    HeadEnd,
    Label,
    Waypoints,
    InkBody,
    FrameName,
    Groups,
    Locked,
    Link,
    Tags,
    Mind,
    Stamps,
    Seed,
    /// W14 — o ficheiro, o recorte, o espelho e (W15) o filtro de uma imagem (mudam juntos: recortar
    /// move a imagem).
    Image,
    /// W18 — o link de um vídeo e o que ele é (mudam juntos).
    Embed,
}

impl Field {
    /// Todos, pela ordem do enum.
    pub const ALL: [Field; 39] = [
        Field::Box,
        Field::Angle,
        Field::Z,
        Field::Frame,
        Field::Deleted,
        Field::Fill,
        Field::Stroke,
        Field::StrokeWidth,
        Field::Dash,
        Field::Round,
        Field::Opacity,
        Field::TextColor,
        Field::FontSize,
        Field::Sketch,
        Field::FillStyle,
        Field::Cartoonist,
        Field::Align,
        Field::VAlign,
        Field::Letter,
        Field::ShapeType,
        Field::Text,
        Field::Start,
        Field::End,
        Field::Route,
        Field::HeadStart,
        Field::HeadEnd,
        Field::Label,
        Field::Waypoints,
        Field::InkBody,
        Field::FrameName,
        Field::Groups,
        Field::Locked,
        Field::Link,
        Field::Tags,
        Field::Mind,
        Field::Stamps,
        Field::Seed,
        Field::Image,
        Field::Embed,
    ];

    /// Uma contagem que SOMA entre pessoas (dois carimbos ao mesmo tempo são dois): a sessão ao vivo
    /// transmite a diferença, não o valor. (O resultado de uma votação não é um registo do elemento:
    /// é uma sessão da equipa, ADR-0183.)
    #[must_use]
    pub fn is_counter(self) -> bool {
        matches!(self, Field::Stamps)
    }
}

fn enc<T: Serialize + ?Sized>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard escreve qualquer valor do modelo")
}

fn dec<T: DeserializeOwned>(b: &[u8]) -> Option<T> {
    postcard::from_bytes(b).ok()
}

impl Element {
    /// O valor do registo `f` em bytes, ou `None` se este tipo de elemento não o tem.
    #[must_use]
    pub fn read(&self, f: Field) -> Option<Vec<u8>> {
        let st = self.style();
        let none = Org::default();
        let org = self.org.as_deref().unwrap_or(&none);
        Some(match f {
            Field::Box => enc(&[self.x, self.y, self.w, self.h]),
            Field::Angle => enc(&self.angle),
            Field::Z => enc(&self.z),
            Field::Frame => enc(&self.frame),
            Field::Deleted => enc(&self.deleted),
            Field::Fill => enc(&st.fill),
            Field::Stroke => enc(&st.stroke),
            Field::StrokeWidth => enc(&st.stroke_width),
            Field::Dash => enc(&st.dash),
            Field::Round => enc(&st.round),
            Field::Opacity => enc(&st.opacity),
            Field::TextColor => enc(&st.text_color),
            Field::FontSize => enc(&st.font_size),
            Field::Sketch => enc(&st.sketch),
            Field::FillStyle => enc(&st.look.fill_style),
            Field::Cartoonist => enc(&st.look.cartoonist),
            Field::Align => enc(&st.look.align),
            Field::VAlign => enc(&st.look.valign),
            Field::Letter => enc(&st.look.letter),
            Field::ShapeType => enc(&self.shape()?.kind),
            Field::Text => enc(&self.shape()?.text),
            Field::Start => enc(&self.connector()?.start),
            Field::End => enc(&self.connector()?.end),
            Field::Route => enc(&self.connector()?.route),
            Field::HeadStart => enc(&self.connector()?.heads[0]),
            Field::HeadEnd => enc(&self.connector()?.heads[1]),
            Field::Label => enc(&self.connector()?.label),
            Field::Waypoints => enc(&self.connector()?.waypoints),
            Field::InkBody => {
                let i = self.ink()?;
                enc(&(i.pen, &i.points, i.base, i.pressure))
            }
            Field::FrameName => enc(&self.as_frame()?.name),
            Field::Groups => enc(&org.groups),
            Field::Locked => enc(&org.locked),
            Field::Link => enc(&org.link),
            Field::Tags => enc(&org.tags),
            Field::Mind => enc(&org.mind),
            Field::Stamps => enc(&org.stamps),
            Field::Seed => enc(&org.seed),
            Field::Image => {
                let i = self.image()?;
                enc(&(i.file, i.crop, i.flip, i.filter))
            }
            Field::Embed => {
                let e = self.embed()?;
                enc(&(&e.link, &e.video))
            }
        })
    }

    /// Escreve o registo `f` com `bytes` (de [`Self::read`]). `false` = este tipo de elemento não o
    /// tem, ou os bytes não são de um valor dele (nada muda).
    pub fn write(&mut self, f: Field, bytes: &[u8]) -> bool {
        let done = self.write_inner(f, bytes).is_some();
        // Um `org` todo por omissão é o mesmo que nenhum: os dois lados de uma fusão escrevem-no igual.
        if self.org.as_deref().is_some_and(|o| *o == Org::default()) {
            self.org = None;
        }
        done
    }

    fn write_inner(&mut self, f: Field, b: &[u8]) -> Option<()> {
        fn put<T: DeserializeOwned>(slot: &mut T, b: &[u8]) -> Option<()> {
            *slot = dec(b)?;
            Some(())
        }
        match f {
            Field::Box => {
                let [x, y, w, h]: [f64; 4] = dec(b)?;
                (self.x, self.y, self.w, self.h) = (x, y, w, h);
                Some(())
            }
            Field::Angle => put(&mut self.angle, b),
            Field::Z => put(&mut self.z, b),
            Field::Frame => put(&mut self.frame, b),
            Field::Deleted => put(&mut self.deleted, b),
            Field::Fill => put(&mut self.style_mut().fill, b),
            Field::Stroke => put(&mut self.style_mut().stroke, b),
            Field::StrokeWidth => put(&mut self.style_mut().stroke_width, b),
            Field::Dash => put(&mut self.style_mut().dash, b),
            Field::Round => put(&mut self.style_mut().round, b),
            Field::Opacity => put(&mut self.style_mut().opacity, b),
            Field::TextColor => put(&mut self.style_mut().text_color, b),
            Field::FontSize => put(&mut self.style_mut().font_size, b),
            Field::Sketch => put(&mut self.style_mut().sketch, b),
            Field::FillStyle => put(&mut self.style_mut().look.fill_style, b),
            Field::Cartoonist => put(&mut self.style_mut().look.cartoonist, b),
            Field::Align => put(&mut self.style_mut().look.align, b),
            Field::VAlign => put(&mut self.style_mut().look.valign, b),
            Field::Letter => put(&mut self.style_mut().look.letter, b),
            Field::ShapeType => put(&mut self.shape_mut()?.kind, b),
            Field::Text => put(&mut self.shape_mut()?.text, b),
            Field::Start => put(&mut self.connector_mut()?.start, b),
            Field::End => put(&mut self.connector_mut()?.end, b),
            Field::Route => put(&mut self.connector_mut()?.route, b),
            Field::HeadStart => put(&mut self.connector_mut()?.heads[0], b),
            Field::HeadEnd => put(&mut self.connector_mut()?.heads[1], b),
            Field::Label => put(&mut self.connector_mut()?.label, b),
            Field::Waypoints => put(&mut self.connector_mut()?.waypoints, b),
            Field::InkBody => {
                let ElementKind::Ink(i) = &mut self.kind else {
                    return None;
                };
                (i.pen, i.points, i.base, i.pressure) = dec(b)?;
                Some(())
            }
            Field::FrameName => put(&mut self.as_frame_mut()?.name, b),
            Field::Groups => put(&mut self.org_mut().groups, b),
            Field::Locked => put(&mut self.org_mut().locked, b),
            Field::Link => put(&mut self.org_mut().link, b),
            Field::Tags => put(&mut self.org_mut().tags, b),
            Field::Mind => put(&mut self.org_mut().mind, b),
            Field::Stamps => put(&mut self.org_mut().stamps, b),
            Field::Seed => put(&mut self.org_mut().seed, b),
            Field::Image => {
                let i = self.image_mut()?;
                (i.file, i.crop, i.flip, i.filter) = dec(b)?;
                Some(())
            }
            Field::Embed => {
                let e = self.embed_mut()?;
                (e.link, e.video) = dec(b)?;
                Some(())
            }
        }
    }

    /// Os registos que diferem entre `self` e `other` (do mesmo id e tipo).
    #[must_use]
    pub fn changed_fields(&self, other: &Element) -> Vec<Field> {
        Field::ALL
            .into_iter()
            .filter(|f| self.read(*f) != other.read(*f))
            .collect()
    }
}

#[cfg(test)]
#[path = "fields_tests.rs"]
mod tests;
