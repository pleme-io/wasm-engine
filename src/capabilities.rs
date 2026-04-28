//! Capability dispatch table.
//!
//! Every WASM/WASI module declares its capability tokens via the
//! `Program.spec.capabilities` list. The wasm-operator passes the list
//! to the engine as `PROGRAM_CAPABILITIES` (newline-separated). The
//! engine refuses any host import call without a matching token.
//!
//! See `theory/WASM-STACK.md` §V for the full capability vocabulary.
//!
//! Skeleton-quality: parse + classify; the actual host-import gating
//! lands in M1 alongside the wasmtime engine wiring.

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// `http-in:0.0.0.0:8080` — inbound HTTP listener (service shape).
    HttpIn { addr: String, port: u16 },
    /// `http-out:host` — egress HTTP/HTTPS to a host.
    HttpOut { host: String },
    /// `kube-secret-read@<ns>/<name>`.
    KubeSecretRead { namespace: String, name: String },
    /// `kube-cr-watch@<group>/<kind>`.
    KubeCrWatch { group: String, kind: String },
    /// `kube-resource-list@<group>/<kind>`.
    KubeResourceList { group: String, kind: String },
    /// `kube-resource-patch@<group>/<kind>`.
    KubeResourcePatch { group: String, kind: String },
    /// `kube-pvc-list`, `kube-pvc-patch`, `kube-event-emit`, `kube-downward-api`, …
    KubeBuiltin(String),
    /// `prom-query@<service>:<port>`.
    PromQuery { service: String, port: u16 },
    /// `fs:<path>:<rw>`.
    Fs { path: String, mode: FsMode },
    /// Anything else — surfaced verbatim so we never silently drop tokens.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FsMode { Read, Write, ReadWrite }

#[derive(Debug, Default, Clone)]
pub struct CapabilitySet {
    set: HashSet<Capability>,
}

impl CapabilitySet {
    #[must_use]
    pub fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut set = HashSet::new();
        for tok in iter {
            set.insert(parse(&tok));
        }
        Self { set }
    }

    /// True if any capability matches the predicate.
    pub fn any<F: FnMut(&Capability) -> bool>(&self, mut pred: F) -> bool {
        self.set.iter().any(|c| pred(c))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Capability> { self.set.iter() }

    #[must_use]
    pub fn len(&self) -> usize { self.set.len() }

    #[must_use]
    pub fn is_empty(&self) -> bool { self.set.is_empty() }
}

/// Parse a single capability token. Skeleton — full validation lands
/// alongside the host-import gates.
#[must_use]
pub fn parse(tok: &str) -> Capability {
    if let Some(rest) = tok.strip_prefix("http-in:") {
        if let Some((addr, port)) = rest.rsplit_once(':') {
            if let Ok(p) = port.parse::<u16>() {
                return Capability::HttpIn { addr: addr.to_string(), port: p };
            }
        }
    }
    if let Some(rest) = tok.strip_prefix("http-out:") {
        return Capability::HttpOut { host: rest.to_string() };
    }
    if let Some(rest) = tok.strip_prefix("kube-secret-read@") {
        if let Some((ns, name)) = rest.split_once('/') {
            return Capability::KubeSecretRead { namespace: ns.to_string(), name: name.to_string() };
        }
    }
    if let Some(rest) = tok.strip_prefix("kube-cr-watch@") {
        if let Some((g, k)) = rest.split_once('/') {
            return Capability::KubeCrWatch { group: g.to_string(), kind: k.to_string() };
        }
    }
    if let Some(rest) = tok.strip_prefix("kube-resource-list@") {
        if let Some((g, k)) = rest.split_once('/') {
            return Capability::KubeResourceList { group: g.to_string(), kind: k.to_string() };
        }
    }
    if let Some(rest) = tok.strip_prefix("kube-resource-patch@") {
        if let Some((g, k)) = rest.split_once('/') {
            return Capability::KubeResourcePatch { group: g.to_string(), kind: k.to_string() };
        }
    }
    if tok.starts_with("kube-") {
        return Capability::KubeBuiltin(tok.to_string());
    }
    if let Some(rest) = tok.strip_prefix("prom-query@") {
        if let Some((svc, port)) = rest.rsplit_once(':') {
            if let Ok(p) = port.parse::<u16>() {
                return Capability::PromQuery { service: svc.to_string(), port: p };
            }
        }
    }
    Capability::Other(tok.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_in() {
        assert_eq!(
            parse("http-in:0.0.0.0:8080"),
            Capability::HttpIn { addr: "0.0.0.0".to_string(), port: 8080 }
        );
    }

    #[test]
    fn parses_kube_secret_read() {
        assert_eq!(
            parse("kube-secret-read@flux-system/sops-age"),
            Capability::KubeSecretRead {
                namespace: "flux-system".to_string(),
                name: "sops-age".to_string(),
            }
        );
    }

    #[test]
    fn parses_kube_builtin() {
        assert!(matches!(parse("kube-downward-api"), Capability::KubeBuiltin(_)));
    }
}
