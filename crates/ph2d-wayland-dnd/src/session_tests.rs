use super::*;
use std::path::PathBuf;

const LIST: &[u8] = b"file:///home/a.png\nfile:///home/b.png\n";

fn uri() -> Vec<String> {
    vec!["text/plain".into(), crate::URI_LIST_MIME.into()]
}

fn two() -> Vec<PathBuf> {
    vec![PathBuf::from("/home/a.png"), PathBuf::from("/home/b.png")]
}

fn drop_of(fx: &[Effect]) -> Option<(&UriList, (f64, f64))> {
    fx.iter().find_map(|e| match e {
        Effect::Emit(DndEvent::Drop { list, at }) => Some((list, *at)),
        _ => None,
    })
}

#[test]
fn enter_motion_drop_delivers_the_files_where_it_was_released_and_finishes() {
    let mut s = Session::default();
    let (t, fx) = s.enter(true, &uri(), (10.0, 20.0));
    let t = t.expect("a uri-list on our window is accepted");
    assert!(fx.is_empty());
    let fx = s.data_ready(t, LIST);
    assert_eq!(
        fx,
        [Effect::Emit(DndEvent::Hover {
            paths: two(),
            at: (10.0, 20.0)
        })]
    );
    assert_eq!(
        s.motion((30.5, 40.25)),
        [Effect::Emit(DndEvent::Motion { at: (30.5, 40.25) })]
    );
    let fx = s.drop_performed();
    let (list, at) = drop_of(&fx).expect("drop delivered");
    assert_eq!(list.paths, two());
    assert_eq!(at, (30.5, 40.25));
    assert_eq!(fx.last(), Some(&Effect::Finish(t)));
    assert!(
        s.leave().is_empty(),
        "a leave after the drop has nothing left to cancel"
    );
}

#[test]
fn a_leave_between_the_drop_and_the_list_does_not_lose_the_drop() {
    let mut s = Session::default();
    let (t, _) = s.enter(true, &uri(), (1.0, 2.0));
    let t = t.unwrap();
    s.motion((5.0, 6.0));
    assert!(
        s.drop_performed().is_empty(),
        "nothing to deliver before the list arrives"
    );
    assert!(
        s.leave().is_empty(),
        "the KWin leave after drop must not cancel it"
    );
    let fx = s.data_ready(t, LIST);
    let (list, at) = drop_of(&fx).expect("the landing drop is delivered when the list arrives");
    assert_eq!(list.paths, two());
    assert_eq!(at, (5.0, 6.0));
    assert_eq!(fx.last(), Some(&Effect::Finish(t)));
}

#[test]
fn a_leave_without_drop_clears_the_hover_and_forgets_the_offer() {
    let mut s = Session::default();
    let t = s.enter(true, &uri(), (0.0, 0.0)).0.unwrap();
    s.data_ready(t, LIST);
    assert_eq!(
        s.leave(),
        [Effect::Emit(DndEvent::Leave), Effect::Cancel(t)]
    );
    assert!(
        s.drop_performed().is_empty(),
        "a drop with no drag in flight delivers nothing"
    );
}

#[test]
fn a_leave_before_the_list_arrives_cancels_the_read_and_a_late_list_is_ignored() {
    let mut s = Session::default();
    let t = s.enter(true, &uri(), (0.0, 0.0)).0.unwrap();
    assert_eq!(
        s.leave(),
        [Effect::Cancel(t)],
        "no Hover was shown, so no Leave"
    );
    assert!(s.data_ready(t, LIST).is_empty());
}

#[test]
fn another_surface_or_no_file_list_is_refused() {
    let mut s = Session::default();
    assert_eq!(s.enter(false, &uri(), (0.0, 0.0)).0, None);
    assert_eq!(s.enter(true, &["text/plain".into()], (0.0, 0.0)).0, None);
    assert!(s.motion((1.0, 1.0)).is_empty());
    assert!(s.drop_performed().is_empty());
}

#[test]
fn a_new_enter_closes_the_previous_drag_first() {
    let mut s = Session::default();
    let a = s.enter(true, &uri(), (0.0, 0.0)).0.unwrap();
    s.data_ready(a, LIST);
    let (b, fx) = s.enter(true, &uri(), (9.0, 9.0));
    assert_eq!(fx, [Effect::Emit(DndEvent::Leave), Effect::Cancel(a)]);
    assert_ne!(b.unwrap(), a);
}

#[test]
fn a_list_with_no_local_file_shows_no_hover_but_its_drop_still_arrives() {
    let mut s = Session::default();
    let t = s.enter(true, &uri(), (0.0, 0.0)).0.unwrap();
    assert!(s.data_ready(t, b"https://example.org/a.png\n").is_empty());
    assert!(s.motion((3.0, 3.0)).is_empty(), "no Motion without a Hover");
    let fx = s.drop_performed();
    let (list, _) = drop_of(&fx).expect("the app must say it refused the drop");
    assert!(list.paths.is_empty());
    assert_eq!(list.not_files, 1);
}

#[test]
fn a_failed_read_still_lands_the_drop() {
    let mut s = Session::default();
    let t = s.enter(true, &uri(), (0.0, 0.0)).0.unwrap();
    s.drop_performed();
    let fx = s.data_ready(t, b"");
    assert_eq!(
        drop_of(&fx).map(|(l, _)| l.clone()),
        Some(UriList::default())
    );
    assert_eq!(fx.last(), Some(&Effect::Finish(t)));
}
