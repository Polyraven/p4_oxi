//! Fake p4 binary for integration testing.
//!
//! Responds to commands based on args, just like the real p4 would.
//! Gated behind the `fake-p4` feature.

use std::io::Read;

const INFO_RESPONSE: &str = r#"{"userName":"my_user","clientName":"my_workspace","clientHost":"my_host","clientCwd":"/my_root","clientRoot":"/my_root","peerAddress":"127.0.0.1","clientAddress":"127.0.0.1","serverAddress":"my_server:1666","serverRoot":"/my_server_root","serverDate":"2024/01/01","serverUptime":"00:00:00","serverVersion":"P4D/LINUX/2024.1","serverLicense":"none","serverServices":"standard","caseHandling":"sensitive","clientVersion":"2024.1"}"#;

const FSTAT_RESPONSES: &[&str] = &[
    r#"{"depotFile":"//my_depot/my_stream/my_file_a.txt","clientFile":"//my_workspace/my_stream/my_file_a.txt","isMapped":"","headAction":"edit","headChange":"100","headRev":"5","headType":"text","headTime":"1700000000","headModTime":"1700000000","haveRev":"5"}"#,
    r#"{"depotFile":"//my_depot/my_stream/my_file_b.txt","clientFile":"//my_workspace/my_stream/my_file_b.txt","isMapped":"","headAction":"add","headChange":"99","headRev":"1","headType":"text","headTime":"1700000000","headModTime":"1700000000","haveRev":"1"}"#,
    r#"{"depotFile":"//my_depot/my_stream/my_file_c.txt","clientFile":"//my_workspace/my_stream/my_file_c.txt","isMapped":"","headAction":"edit","headChange":"98","headRev":"2","headType":"text","headTime":"1700000000","headModTime":"1700000000","haveRev":"2"}"#,
];

const CHANGE_FORM_RESPONSE: &str = r#"{"Change":"new","Client":"my_workspace","User":"my_user","Status":"new","Description":"<enter description here>","Date":"2024/01/01 00:00:00","Type":"public","ImportedBy":"","Identity":"","Stream":"","Jobs":"","Files":""}"#;

const CHANGE_SUBMIT_RESPONSE: &str = "Change 12345 created.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str());
    let has_flag = |f: &str| args.iter().any(|a| a == f);

    match cmd {
        Some("info") => println!("{INFO_RESPONSE}"),
        Some("fstat") => {
            for line in FSTAT_RESPONSES {
                println!("{line}");
            }
        }
        Some("change") if has_flag("-o") => println!("{CHANGE_FORM_RESPONSE}"),
        Some("change") if has_flag("-i") => {
            let mut _input = String::new();
            std::io::stdin().read_to_string(&mut _input).unwrap();
            println!("{CHANGE_SUBMIT_RESPONSE}");
        }
        Some("change") if has_flag("-d") => {}
        _ => {
            eprintln!("fake_p4: unhandled command: {args:?}");
            std::process::exit(1);
        }
    }
}
