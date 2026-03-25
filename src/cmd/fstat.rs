
use std::collections::HashMap;

use serde::Deserialize;

use crate::*;

// Command struct //////////////////////////////////////////////////////////////

/// Get detailed file information for files in the depot.
///
/// <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_fstat.html#p4_fstat>
#[derive(Debug, Clone, Default)]
pub struct P4FStat {
    wildcards: Vec<String>,
    filter: Option<String>,                  // -F filter
    fast_lookup: bool,                       // -L
    fields: Option<String>,                  // -T fields
    max: Option<u32>,                        // -m max
    reverse: bool,                           // -r
    at_changelist: Option<u64>,              // -c change
    in_changelist: Option<u64>,              // -e change
    attribute_pattern: Option<String>,       // -A pattern
    include_unload: bool,                    // -U
    stream_views: bool,                      // --streamviews
    output_opts: Vec<P4FStatOptionalOutput>, // -O{x}
    restrict_opts: Vec<P4FStatRestrict>,     // -R{x}
    sort_opts: Vec<P4FStatSort>,             // -S{x}
}

// Output //////////////////////////////////////////////////////////////////////

/// Content output of the [`P4FStat`] command.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct P4FStatOut {
    // Always present
    pub depot_file: String,

    // Present if mapped to current workspace
    pub client_file: Option<String>,
    pub is_mapped: Option<String>,

    // Head revision info (present if file is in depot)
    // accessible via .head
    head_action: Option<String>,
    head_change: Option<String>,
    head_rev: Option<String>,
    head_type: Option<String>,
    head_time: Option<String>,
    head_mod_time: Option<String>,

    // Workspace sync info
    pub have_rev: Option<String>,

    // Open file info (present if opened in workspace)
    // accessible via .opened
    action: Option<String>,
    change: Option<String>,
    #[serde(rename = "type")]
    file_type: Option<String>,
    action_owner: Option<String>,
    work_rev: Option<String>,

    // Resolve info
    pub resolved: Option<String>,
    pub unresolved: Option<String>,

    // Other users
    pub other_open: Option<String>,
    pub other_lock: Option<String>,

    // Move info
    pub moved_file: Option<String>,
    pub moved_rev: Option<String>,

    // Shelved
    pub shelved: Option<String>,

    // Requires -Ol
    pub file_size: Option<String>,
    pub digest: Option<String>,

    // Path info
    pub is_missing: Option<String>,
    pub path: Option<String>,
    pub path_source: Option<String>,
    pub path_type: Option<String>,
    pub path_permissions: Option<String>,

    // Component info
    pub effective_component_type: Option<String>,

    #[serde(flatten)]
    #[allow(dead_code)]
    unhandled: HashMap<String, String>,
}

#[derive(Debug)]
pub struct P4FstatOutHead {
    pub action: String,
    pub change: u64,
    pub rev: u64,
    pub action_type: String,
    pub time: u64,
    pub mod_time: u64,
}

#[derive(Debug)]
struct _P4FstatOutOpened {
    // TODO
}

#[derive(Debug)]
struct _P4FstatOtherOpen {
    // TODO
    pub action: String,     // otherAction0
    pub changelist_id: u64, // otherChange0
    pub user: String,       // otherOpen0
}

impl P4FStatOut {
    // Head (depot) revision info (None if file is not on depot)
    pub fn head(&self) -> Option<P4FstatOutHead> {
        self.head_action.as_ref().map(|_| P4FstatOutHead {
            action: self.head_action.clone().unwrap_or_default(),
            change: super::parse_or_default(self.head_change.clone()),
            rev: super::parse_or_default(self.head_rev.clone()),
            action_type: self.head_type.clone().unwrap_or_default(),
            time: super::parse_or_default(self.head_time.clone()),
            mod_time: super::parse_or_default(self.head_mod_time.clone()),
        })
    }

    // TODO: Open file info (none if not opened in workspace)
    fn _opened(&self) -> Option<_P4FstatOutOpened> {
        todo!()
    }

    // TODO: Info about other people who have the file checked out or locked
    fn _other_open(&self) -> Vec<_P4FstatOtherOpen> {
        // self.other_open has the count while self.unhandled, keys have to be constructed.
        todo!()
    }
}

// Trait impls /////////////////////////////////////////////////////////////////

impl From<P4FStat> for P4RawCmd {
    fn from(value: P4FStat) -> Self {
        let mut raw = P4RawCmd::new("fstat");

        if let Some(filter) = &value.filter {
            raw = raw.with_arg_value("-F", filter);
        }
        if value.fast_lookup {
            raw = raw.with_arg("-L");
        }
        if let Some(fields) = &value.fields {
            raw = raw.with_arg_value("-T", fields);
        }
        if let Some(max) = value.max {
            raw = raw.with_arg_value("-m", max);
        }
        if value.reverse {
            raw = raw.with_arg("-r");
        }
        if let Some(cl) = value.at_changelist {
            raw = raw.with_arg_value("-c", cl);
        }
        if let Some(cl) = value.in_changelist {
            raw = raw.with_arg_value("-e", cl);
        }
        if let Some(pattern) = &value.attribute_pattern {
            raw = raw.with_arg_value("-A", pattern);
        }
        if value.include_unload {
            raw = raw.with_arg("-U");
        }
        if value.stream_views {
            raw = raw.with_arg("--streamviews");
        }

        if !value.output_opts.is_empty() {
            let suffix: String = value.output_opts.iter().map(|o| o.as_char()).collect();
            raw = raw.with_arg(format!("-O{suffix}"));
        }
        if !value.restrict_opts.is_empty() {
            let suffix: String = value.restrict_opts.iter().map(|r| r.as_char()).collect();
            raw = raw.with_arg(format!("-R{suffix}"));
        }
        if !value.sort_opts.is_empty() {
            let suffix: String = value.sort_opts.iter().map(|s| s.as_char()).collect();
            raw = raw.with_arg(format!("-S{suffix}"));
        }

        raw.wildcards = value.wildcards;

        raw
    }
}

impl P4Cmd for P4FStat {
    type Output = P4FStatOut;
}

// Command Builders ////////////////////////////////////////////////////////////

/// Sub-flags for `-O{x}` controlling what extra data fstat outputs.
#[derive(Debug, Clone, Copy)]
pub enum P4FStatOptionalOutput {
    /// `-Ob` Archive file path, revision, type, and local paths.
    Archive,
    /// `-Od` MD5 digest of attribute values.
    DigestAttr,
    /// `-Oe` Attribute values encoded as hex.
    HexEncode,
    /// `-Of` All revisions for the given files.
    AllRevs,
    /// `-Ol` File size and MD5 digest fields.
    FileSizeAndDigest,
    /// `-Om` Other ditto-mapped client files.
    DittoMap,
    /// `-On` Attribute trait value storage location.
    TraitLocation,
    /// `-Op` Client file path in depot syntax and local syntax.
    DepotPath,
    /// `-Or` Pending integration record data for open files.
    PendingInteg,
    /// `-Os` Shorten output by excluding client workspace data.
    Short,
    /// `-OL` Global lock info (edge server).
    GlobalLock,
}

impl P4FStatOptionalOutput {
    fn as_char(&self) -> char {
        match self {
            Self::Archive => 'b',
            Self::DigestAttr => 'd',
            Self::HexEncode => 'e',
            Self::AllRevs => 'f',
            Self::FileSizeAndDigest => 'l',
            Self::DittoMap => 'm',
            Self::TraitLocation => 'n',
            Self::DepotPath => 'p',
            Self::PendingInteg => 'r',
            Self::Short => 's',
            Self::GlobalLock => 'L',
        }
    }
}

/// Sub-flags for `-R{x}` restricting which files are included in output.
#[derive(Debug, Clone, Copy)]
pub enum P4FStatRestrict {
    /// `-Rc` Files mapped into the current workspace.
    Mapped,
    /// `-Rh` Files on the have list.
    HaveList,
    /// `-Rn` Files opened not at the head revision.
    NotHead,
    /// `-Ro` Files open in the current workspace.
    Open,
    /// `-Rr` Open files that have been resolved.
    Resolved,
    /// `-Rs` Shelved files (requires `-e changelist`).
    Shelved,
    /// `-Ru` Open files that are unresolved.
    Unresolved,
}

impl P4FStatRestrict {
    fn as_char(&self) -> char {
        match self {
            Self::Mapped => 'c',
            Self::HaveList => 'h',
            Self::NotHead => 'n',
            Self::Open => 'o',
            Self::Resolved => 'r',
            Self::Shelved => 's',
            Self::Unresolved => 'u',
        }
    }
}

/// Sub-flags for `-S{x}` controlling sort order.
#[derive(Debug, Clone, Copy)]
pub enum P4FStatSort {
    /// `-Sd` Sort by date.
    Date,
    /// `-Sh` Sort by have revision.
    HaveRev,
    /// `-Sr` Sort by head revision.
    HeadRev,
    /// `-Ss` Sort by file size.
    FileSize,
    /// `-St` Sort by file type.
    FileType,
}

impl P4FStatSort {
    fn as_char(&self) -> char {
        match self {
            Self::Date => 'd',
            Self::HaveRev => 'h',
            Self::HeadRev => 'r',
            Self::FileSize => 's',
            Self::FileType => 't',
        }
    }
}

impl P4FStat {
    pub fn of(wildcards: &[impl ToString]) -> Self {
        Self {
            wildcards: wildcards.iter().map(|w| w.to_string()).collect(),
            ..Default::default()
        }
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    pub fn with_fast_lookup(mut self) -> Self {
        self.fast_lookup = true;
        self
    }

    pub fn with_fields(mut self, fields: impl Into<String>) -> Self {
        self.fields = Some(fields.into());
        self
    }

    pub fn with_max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    pub fn with_at_changelist(mut self, changelist: u64) -> Self {
        self.at_changelist = Some(changelist);
        self
    }

    pub fn with_in_changelist(mut self, changelist: u64) -> Self {
        self.in_changelist = Some(changelist);
        self
    }

    pub fn with_attribute_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.attribute_pattern = Some(pattern.into());
        self
    }

    pub fn with_include_unload(mut self) -> Self {
        self.include_unload = true;
        self
    }

    pub fn with_stream_views(mut self) -> Self {
        self.stream_views = true;
        self
    }

    pub fn with_optional_output(mut self, opt: P4FStatOptionalOutput) -> Self {
        self.output_opts.push(opt);
        self
    }

    pub fn with_restrict(mut self, opt: P4FStatRestrict) -> Self {
        self.restrict_opts.push(opt);
        self
    }

    pub fn with_sort(mut self, opt: P4FStatSort) -> Self {
        self.sort_opts.push(opt);
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
