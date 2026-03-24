# p4_oxi

An unofficial, work-in-progress Perforce (Helix / P4) version control system wrapper written for and in Rust.

Current implementation wraps the P4-CLI, minimum supported version: `2016.1`

---

## Key design principles:

- Builder pattern for constructing commands.
- Iterable command execution; enabling data processing while p4 is still running the command.
- Reflecting each returned value with its corresponding type (P4Info, P4Warning, etc.).
- Rich return type to differentiate between fatal error and per file warnings `Result<P4Out<T>, P4Error>`.
- Typestate pattern for ensuring invariance and command validity at compile time.
- Fall back to raw commands (P4RawCmd) when a typed command isn't implemented or doesn't fit your needs.

---

## Examples:

### Getting a single result:

```rust
use p4_oxi::{*, cmd::P4FStat};

match P4FStat::of(&["//depot/main/my_file.txt"]).run()?.single()? {
    P4Out::Content(fstat_out) =>  println!("{}", fstat_out.depot_file),
    P4Out::Warning(w) => println!("Warning: {w}"),
    other => println!("Unexpected: {other:?}"),
}
```

### Iterators
Calling .run() starts the p4 process and returns an iterator that yields results as they arrive, allowing you to process or bail out before the command finishes.

- Functional approach:
```rust
use p4_oxi::{*, cmd::{P4FStat, P4FStatOut}};

let fstats: Vec<P4FStatOut> = P4FStat::of(&["//depot/main/..."])
    .with_max(5)
    .run()?
    .filter_map(|v| v.take_content()) // discards non-content variants
    .collect();
```

- Imperative approach:
```rust
use p4_oxi::{*, cmd::{P4FStat, P4FStatOut}};

let mut fstats: Vec<P4FStatOut> = Vec::new();
for item in P4FStat::of(&["//depot/main/..."]).with_max(5).run()? {
    // demonstrating how the rich return type can be used to decide on different paths
    match item {
        // Demonstrates stopping the command, but note that P4 commands are not transactional.
        P4Out::ParseError(_) | P4Out::Failure(_) => { break },

        // Non fatal per warning, like `file(s) up-to-date.` during sync etc.
        P4Out::Warning(v) => { println!("{v}") },

        // Valid value, collecting it.
        P4Out::Content(fstat_out) => { fstats.push(fstat_out) },

        P4Out::Info(_) => { continue; },
    }
}
```

- Raw commands:
```rust
use p4_oxi::{*, cmd::{P4FStat, P4FStatOut}};

let running_raw_cmd = P4RawCmd::new("fstat")
      .with_arg_value("-m", 5)
      .with_wildcard("//depot/main/...")
      .run()?;

// Note: the iterator here yields P4Out<String> instead of P4Out<P4FStatOut>,
// but it can be casted, and this is what actually the typed command does as well:
let fstats: Vec<P4FStatOut> = running_raw_cmd
    .cast::<P4FStatOut>()
    .filter_map(|v| v.take_content())
    .collect();
```

### Forms / Specs

Form uploading like when creating changelists or workspaces is also possible using the `.submit_form()` and `.read_form()` methods.
```rust
use p4_oxi::{*, cmd::{P4Change, P4ChangeForm}};

let changelist_form = P4ChangeForm::new("my_description")?;
let new_changelist_id = P4Change::for_submit().submit(changelist_form);
```

### Fire and forget
When you don't need individual results, .finish() runs the command to completion and returns an error if anything failed.
  
```rust
use p4_oxi::{*, cmd::P4Change};

P4Change::for_delete(123456).run()?.finish()?;
```

---

## What's done and what's missing:

Most work and thought so far went into the command building machinery and that is considered working. The implemented commands serve as proof of concept of the approach, but only a few are covered so far. For the rest, fall back to P4RawCmd.

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.
