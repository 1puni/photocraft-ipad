//! Native document snapshots for the browser host; no renderer or lossy interchange.
use photocraft_doc::LayerId;
use photocraft_engine::Session;
use serde_json::{Value, json};

pub struct Snapshot {
    pub metadata: String,
    pub documents: Vec<Vec<u8>>,
}

pub fn signature(session: &Session) -> String {
    json!({"active":session.active_index(), "documents":session.documents().iter().map(|s| {
        json!([s.doc.id.0,s.revision,s.doc.name,s.active_layer.map(|id|id.0),s.selected_layers.iter().map(|id|id.0).collect::<Vec<_>>()])
    }).collect::<Vec<_>>()}).to_string()
}

pub fn capture(session: &Session) -> Result<Snapshot, String> {
    let documents = session
        .documents()
        .iter()
        .map(|s| {
            photocraft_format::save_to_bytes(&s.doc, &Default::default()).map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let metadata = json!({"version":1,"active":session.active_index(),"documents":session.documents().iter().map(|s| {
        json!({"activeLayer":s.active_layer.map(|id|id.0),"selectedLayers":s.selected_layers.iter().map(|id|id.0).collect::<Vec<_>>()})
    }).collect::<Vec<_>>()}).to_string();
    Ok(Snapshot {
        metadata,
        documents,
    })
}

/// Decode everything before admitting any document. Corrupt recovery must not replace good data.
pub fn restore(session: &mut Session, snapshot: Snapshot) -> Result<usize, String> {
    if !session.documents().is_empty() {
        return Err("editor is already in use; recovery retained".into());
    }
    let meta: Value = serde_json::from_str(&snapshot.metadata).map_err(|e| e.to_string())?;
    let entries = meta["documents"]
        .as_array()
        .ok_or("invalid recovery manifest")?;
    if meta["version"] != 1 || entries.len() != snapshot.documents.len() {
        return Err("unsupported recovery snapshot".into());
    }
    let docs = snapshot
        .documents
        .iter()
        .map(|bytes| photocraft_format::load_from_bytes(bytes).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let count = docs.len();
    for (doc, entry) in docs.into_iter().zip(entries) {
        session.add_document(doc, None);
        let s = session.active_mut().expect("just admitted");
        s.saved_revision = 0; // A browser recovery copy is not a downloaded user file.
        let valid = |id: &LayerId| s.doc.layer(*id).is_some();
        s.selected_layers = entry["selectedLayers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_u64().map(LayerId))
            .filter(valid)
            .collect();
        s.active_layer = entry["activeLayer"].as_u64().map(LayerId).filter(valid);
        s.layer_anchor = s.active_layer;
    }
    if let Some(index) = meta["active"].as_u64() {
        session.set_active(index as usize);
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_ui_egui::{PhotocraftApp, Services};
    #[test]
    fn layered_documents_and_active_selection_roundtrip_without_marking_saved() {
        let mut app = PhotocraftApp::new(Session::new(), Services::default());
        app.run("file.new", json!({"width":16,"height":12,"name":"First"}))
            .unwrap();
        app.run("layer.new.layer", json!({"name":"Ink"})).unwrap();
        app.run("edit.fill", json!({"color":"#cc8844"})).unwrap();
        app.run("layer.setProps", json!({"opacity":0.45,"visible":false}))
            .unwrap();
        app.run("layer.translate", json!({"dx":3,"dy":2})).unwrap();
        app.run("file.new", json!({"width":8,"height":6,"name":"Second"}))
            .unwrap();
        app.session.set_active(0);
        let before = app.session.active().unwrap().doc.clone();
        let layer = app.session.active().unwrap().active_layer;
        let snapshot = capture(&app.session).unwrap();
        let original_bytes = snapshot.documents.clone();
        let mut restored = Session::new();
        assert_eq!(restore(&mut restored, snapshot).unwrap(), 2);
        assert_eq!(capture(&restored).unwrap().documents, original_bytes);
        assert_eq!(restored.active_index(), Some(0));
        assert_eq!(restored.active().unwrap().active_layer, layer);
        assert_eq!(restored.active().unwrap().doc.name, before.name);
        assert_eq!(
            restored.active().unwrap().doc.layers.len(),
            before.layers.len()
        );
        assert!(restored.documents().iter().all(|d| d.is_dirty()));
        assert!(restored.documents().iter().all(|d| d.path.is_none()));
    }
    #[test]
    fn corrupt_snapshot_is_all_or_nothing_and_existing_work_is_preserved() {
        let mut app = PhotocraftApp::new(Session::new(), Services::default());
        app.run("file.new", json!({"width":4,"height":4})).unwrap();
        let mut snapshot = capture(&app.session).unwrap();
        snapshot.documents[0] = vec![0, 1, 2];
        let mut empty = Session::new();
        assert!(restore(&mut empty, snapshot).is_err());
        assert!(empty.documents().is_empty());
        let snapshot = capture(&app.session).unwrap();
        assert!(restore(&mut app.session, snapshot).is_err());
        assert_eq!(app.session.documents().len(), 1);
    }
}
