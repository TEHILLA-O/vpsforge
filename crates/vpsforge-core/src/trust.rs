use serde::{Deserialize, Serialize};

/// Every third-party source VPSForge may add must declare provenance.
///
/// The installer never silently pipes remote scripts into a shell.
/// Download → verify HTTPS → verify checksum/signature when available →
/// show the source → install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedSource {
    pub name: String,
    pub homepage: String,
    pub url: String,
    pub signing_key_url: Option<String>,
    pub fingerprint: Option<String>,
    pub checksum: Option<String>,
    pub checksum_url: Option<String>,
    pub version: Option<String>,
    pub notes: String,
}

impl TrustedSource {
    pub fn new(name: impl Into<String>, homepage: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            homepage: homepage.into(),
            url: url.into(),
            signing_key_url: None,
            fingerprint: None,
            checksum: None,
            checksum_url: None,
            version: None,
            notes: String::new(),
        }
    }

    pub fn requires_https(&self) -> bool {
        self.url.starts_with("https://")
            && self
                .signing_key_url
                .as_deref()
                .is_none_or(|u| u.starts_with("https://"))
    }

    pub fn has_integrity(&self) -> bool {
        self.checksum.is_some()
            || self.checksum_url.is_some()
            || (self.signing_key_url.is_some() && self.fingerprint.is_some())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRecord {
    pub source: TrustedSource,
    pub https_ok: bool,
    pub integrity_ok: bool,
    pub shown_to_user: bool,
}
