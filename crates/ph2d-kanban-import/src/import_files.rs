//! **Anexos e capas** da importação: os URLs `ph2d://` da exportação voltam ao `Link` que eram, os
//! ficheiros carregados entram pelos bytes da [`FileSource`] (sem ela, como links) e a capa de
//! imagem só entra se o anexo dela entrou com os bytes.

use std::collections::BTreeMap;

use ph2d_kanban_model::{
    Attachment, AttachmentId, CardId, Cover, CoverFill, FileId, FracKey, KanbanId, Link, Meta,
    Moment, Record,
};
use serde_json::Value;

use crate::import::{Ctx, arr, flag, hue_field, pos, rank, s, time_of};
use crate::report::LossKind;
use crate::trello::{CARD_SCHEME, FILE_SCHEME};

fn hex32(h: &str) -> Option<[u8; 32]> {
    if h.len() != 64 || !h.is_ascii() {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&h[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

/// Os URLs `ph2d://` que a exportação escreve voltam ao `Link` que eram.
fn ph2d_link(url: &str) -> Option<Link> {
    if let Some(h) = url.strip_prefix(FILE_SCHEME) {
        return hex32(h).map(|b| Link::File(FileId(b)));
    }
    let rest = url.strip_prefix(CARD_SCHEME)?;
    let mut parts = rest.split('/');
    let k = KanbanId(parts.next()?.parse().ok()?);
    match (parts.next(), parts.next(), parts.next()) {
        (None, ..) => Some(Link::Kanban(k)),
        (Some("card"), Some(c), None) => Some(Link::Card(k, CardId(c.parse().ok()?))),
        _ => None,
    }
}

fn link(cx: &mut Ctx, a: &Value, name: &str) -> Link {
    let url = s(a, "url").unwrap_or("");
    if let Some(l) = ph2d_link(url) {
        return l;
    }
    if flag(a, "isUpload")
        && let Some(src) = cx.source
    {
        let file = s(a, "fileName").unwrap_or(name);
        match src.bytes(s(a, "id").unwrap_or(""), file) {
            Some(bytes) => {
                let fid = FileId::of(&bytes);
                if cx.file_ids.insert(fid) {
                    cx.files.push(bytes);
                }
                return Link::File(fid);
            }
            None => cx.report.lose(LossKind::AttachmentFileMissing, name),
        }
    }
    Link::Url(url.to_owned())
}

/// Anexos de cada cartão aceite; devolve id do anexo no Trello → o link (a capa lê-o).
pub(crate) fn attachments(cx: &mut Ctx, cards: &[(u64, &Value)]) -> BTreeMap<String, Link> {
    let mut links = BTreeMap::new();
    for &(card, c) in cards {
        let mut got = Vec::new();
        for a in arr(c, "attachments") {
            let Some(tid) = s(a, "id") else { continue };
            let Some(id) = cx.fresh(tid) else { continue };
            let name = s(a, "name").unwrap_or("").to_owned();
            let link = link(cx, a, &name);
            links.insert(tid.to_owned(), link.clone());
            let at = time_of(cx, a, "date", &name)
                .or_else(|| cx.cards.get(&card).map(|c| c.created))
                .unwrap_or(Moment(0));
            got.push((
                pos(a),
                Attachment {
                    id: AttachmentId(id),
                    card: CardId(card),
                    pos: FracKey::between(None, None),
                    name,
                    link,
                    at,
                    meta: Meta::default(),
                },
            ));
        }
        for (k, mut a) in rank(got) {
            a.pos = k;
            cx.records.push(Record::Attachment(a));
        }
    }
    links
}

pub(crate) fn covers(cx: &mut Ctx, cards: &[(u64, &Value)], links: &BTreeMap<String, Link>) {
    for &(card, c) in cards {
        let Some(cover) = c.get("cover").filter(|v| v.is_object()) else {
            continue;
        };
        let full = s(cover, "size") == Some("full");
        let fill = if let Some(att) = s(cover, "idAttachment") {
            match links.get(att) {
                Some(Link::File(f)) => Some(CoverFill::Image(*f)),
                _ => {
                    let t = cx
                        .cards
                        .get(&card)
                        .map(|c| c.title.clone())
                        .unwrap_or_default();
                    cx.report.lose(LossKind::CoverImage, t);
                    None
                }
            }
        } else if s(cover, "idUploadedBackground").is_some() {
            cx.report
                .lose(LossKind::CoverImage, s(c, "name").unwrap_or(""));
            None
        } else {
            hue_field(cx, cover).map(CoverFill::Color)
        };
        if let (Some(fill), Some(card)) = (fill, cx.cards.get_mut(&card)) {
            card.cover = Some(Cover { fill, full });
        }
    }
}
