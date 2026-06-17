//! Integration test: `SnapshotStorage::OnDisk(PathBuf)` postcard roundtrip
//! cobertura. Audit T1.8 L2-F6.

use std::path::PathBuf;

use ph2d_painter_stroke::{LayerSnapshot, SNAPSHOT_VERSION, SnapshotStorage, device::PersistLayerId};

#[test]
fn snapshot_ondisk_roundtrips_postcard_utf8_path() {
    let snap = LayerSnapshot {
        at_seq: 100,
        layer_id: PersistLayerId(3),
        texture_blake3: [0xAB; 32],
        texture_data: SnapshotStorage::OnDisk(PathBuf::from("/tmp/ph2d-cache/layer-3-100.bin")),
        timestamp_ms: 0,
        // Audit T1.8 L3-G7: usar constant em vez de literal pra detectar
        // bump de version inadvertido.
        version: SNAPSHOT_VERSION,
    };
    let bytes = postcard::to_allocvec(&snap).expect("UTF-8 path serializes");
    let back: LayerSnapshot = postcard::from_bytes(&bytes).expect("roundtrip");
    assert_eq!(snap, back);
    match back.texture_data {
        SnapshotStorage::OnDisk(p) => {
            assert_eq!(p, PathBuf::from("/tmp/ph2d-cache/layer-3-100.bin"))
        }
        _ => panic!("variant changed on roundtrip"),
    }
}

#[test]
fn snapshot_inmemory_roundtrips_postcard() {
    let snap = LayerSnapshot::new_in_memory(PersistLayerId(1), 50, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let bytes = postcard::to_allocvec(&snap).expect("serialize");
    let back: LayerSnapshot = postcard::from_bytes(&bytes).expect("roundtrip");
    assert_eq!(snap, back);
}
