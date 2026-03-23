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

## Example:

### Typed Commands

- Functional approach: 
```rust, ignore
let fstats: Vec<P4FStatOut> = P4FStat::of(&["//depot/main/..."])
    .with_max(5)
    .run()?
    .filter_map(|v| v.take_content()) // discards non-content variants
    .collect();
```

- Imperative approach:
```rust, ignore
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

One can also use: `.first()`, `.single()` & `.finish()` shorthand commands to avoid dealing with the iterator when not necessary.

### Raw Commands
```rust, ignore
// Here is the command equivalent:
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
```rust, ignore
let changelist_form = P4ChangeForm::new("my_description")?;
let new_changelist_id = P4Change::for_submit().submit(changelist_form);
```

---

## What's done and what's missing:

Most work and thought so far went into the command building machinery and that is considered working. The implemented commands serve as proof of concept of the approach, but only a few are covered so far. For the rest, fall back to P4RawCmd.

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.
