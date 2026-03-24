//! Integration tests for the command pipeline machinery.
//!
//! Tests the full path at three levels:
//!   Raw:     P4RawCmd -> IterCmd -> RunningIterCmd -> P4RunningCmd -> P4Out<String>
//!   Command: CmdStruct -> P4Cmd trait -> P4TypedRunningCmd -> P4Out<T>
//!   Focused: P4FocusedCmd -> trait passthrough -> same pipeline
//!
//! Uses the `fake_p4` binary (feature `mock-p4`) instead of a real Perforce server.
//!
//! ```sh
//! cargo test --features mock-p4 --test pipeline
//! ```

#![cfg(feature = "mock-p4")]

use p4_oxi::cmd::{P4Change, P4ChangeForm, P4FStat, P4FStatOut, P4Info};
use p4_oxi::{P4Cmd, P4RawCmd};

struct FakeP4Guard;

impl FakeP4Guard {
    fn setup() -> Self {
        unsafe {
            std::env::set_var("_FAKE_P4_BIN", env!("CARGO_BIN_EXE_fake_p4"));
        }
        Self
    }
}

impl Drop for FakeP4Guard {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var("_FAKE_P4_BIN");
        }
    }
}

// Raw command pipeline ////////////////////////////////////////////////////////
// Tests: IterCmd spawning, RunningIterCmd line iteration, P4RunningCmd categorization

#[test]
fn raw_single_line_output() {
    let _guard = FakeP4Guard::setup();
    let items: Vec<_> = P4RawCmd::new("info").run().unwrap().collect();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_content());
}

#[test]
fn raw_multiple_line_output() {
    let _guard = FakeP4Guard::setup();
    let items: Vec<_> = P4RawCmd::new("fstat")
        .with_wildcard("//my_depot/...")
        .run()
        .unwrap()
        .collect();
    assert_eq!(items.len(), 3);
    for item in &items {
        assert!(item.is_content());
    }
}

// Typed command pipeline //////////////////////////////////////////////////////
// Tests: + P4TypedRunningCmd deserialization, consumption methods (first, single, finish)

#[test]
fn cmd_single_result_deserializes() {
    let _guard = FakeP4Guard::setup();
    let out = P4Info::default()
        .run()
        .unwrap()
        .single()
        .unwrap()
        .take_content()
        .unwrap();
    assert_eq!(out.user_name, "my_user");
    assert_eq!(out.client_name, "my_workspace");
    assert_eq!(out.server_address, "my_server:1666");
}

#[test]
fn cmd_first_result_deserializes() {
    let _guard = FakeP4Guard::setup();
    let out = P4Info::default()
        .run()
        .unwrap()
        .first()
        .unwrap()
        .take_content()
        .unwrap();
    assert_eq!(out.user_name, "my_user");
    assert_eq!(out.client_name, "my_workspace");
    assert_eq!(out.server_address, "my_server:1666");
}

#[test]
fn cmd_multiple_results_iterate_and_deserialize() {
    let _guard = FakeP4Guard::setup();
    let results: Vec<P4FStatOut> = P4FStat::of(&["//my_depot/..."])
        .run()
        .unwrap()
        .filter_map(|item| item.take_content())
        .collect();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].depot_file, "//my_depot/my_stream/my_file_a.txt");
    assert_eq!(results[0].head().unwrap().change, 100);
    assert_eq!(results[1].depot_file, "//my_depot/my_stream/my_file_b.txt");
    assert_eq!(results[1].head().unwrap().action, "add");
    assert_eq!(results[2].depot_file, "//my_depot/my_stream/my_file_c.txt");
    assert_eq!(results[2].head().unwrap().rev, 2);
}

// Typestate command pipeline /////////////////////////////////////////////////////
// Tests: + P4FocusedCmd passthrough to P4Cmd / P4FormCmd trait methods

#[test]
fn focused_form_read() {
    let _guard = FakeP4Guard::setup();
    let form = P4Change::for_read(None).read().unwrap();
    assert_eq!(form.change, "new");
    assert_eq!(form.user, "my_user");
    assert_eq!(form.workspace, "my_workspace");
    assert_eq!(form.status, "new");
}

#[test]
fn focused_form_submit() {
    let _guard = FakeP4Guard::setup();
    let form = P4ChangeForm {
        change: "new".to_string(),
        workspace: "my_workspace".to_string(),
        user: "my_user".to_string(),
        status: "new".to_string(),
        description: "Test changelist".to_string(),
        date: "2024/01/01".to_string(),
        change_type: "public".to_string(),
        ..Default::default()
    };
    let changelist_id = P4Change::for_submit().submit(form).unwrap();
    assert_eq!(changelist_id, 12345);
}

#[test]
fn focused_finish() {
    let _guard = FakeP4Guard::setup();
    P4Change::for_delete(1).run().unwrap().finish().unwrap();
}
