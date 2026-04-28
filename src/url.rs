//! URL grammar for module sources.
//!
//! Mirrors the Nix flake URL grammar — see `theory/WASM-PACKAGING.md` §II.
//! Forms recognized:
//!
//! ```text
//! ./local/path.tlisp                                      file path
//! github:owner/repo/path/to/program.tlisp[?ref=…]         GitHub
//! gitlab:owner/repo/path.tlisp[?ref=…]                    GitLab
//! codeberg:owner/repo/path.tlisp[?ref=…]                  Codeberg
//! oci://ghcr.io/pleme-io/programs:tag                     OCI registry
//! https://example.com/program.wasm[#blake3=…]             direct fetch
//! git+https://example.com/repo.git[?dir=programs]         generic git
//! ```
//!
//! Skeleton-quality: only the variant tags + a stub `parse` that
//! returns `Source::Unknown` for anything non-trivial. Production
//! parsing lands in M1 alongside fetch+resolve+blake3-cache.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Local { path: PathBuf },
    GitHub { owner: String, repo: String, path: PathBuf, rev: Option<String> },
    GitLab { owner: String, repo: String, path: PathBuf, rev: Option<String> },
    Codeberg { owner: String, repo: String, path: PathBuf, rev: Option<String> },
    Git { url: String, dir: Option<String>, rev: Option<String> },
    Http { url: String, blake3: Option<String> },
    Oci { reference: String, blake3: Option<String> },
    Unknown(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("empty url")]
    Empty,
    #[error("malformed url: {0}")]
    Malformed(String),
}

/// Parse a module source URL. Skeleton — currently routes by leading
/// scheme/keyword; full per-form decode lands in M1.
pub fn parse(input: &str) -> Result<Source, ParseError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    if s.starts_with("./") || s.starts_with('/') {
        return Ok(Source::Local { path: PathBuf::from(s) });
    }
    if let Some(rest) = s.strip_prefix("github:") {
        return Ok(parse_forge(rest, "github").map(|(o, r, p, rev)| Source::GitHub {
            owner: o, repo: r, path: p, rev,
        }).unwrap_or(Source::Unknown(s.to_string())));
    }
    if let Some(rest) = s.strip_prefix("gitlab:") {
        return Ok(parse_forge(rest, "gitlab").map(|(o, r, p, rev)| Source::GitLab {
            owner: o, repo: r, path: p, rev,
        }).unwrap_or(Source::Unknown(s.to_string())));
    }
    if let Some(rest) = s.strip_prefix("codeberg:") {
        return Ok(parse_forge(rest, "codeberg").map(|(o, r, p, rev)| Source::Codeberg {
            owner: o, repo: r, path: p, rev,
        }).unwrap_or(Source::Unknown(s.to_string())));
    }
    if s.starts_with("oci://") {
        return Ok(Source::Oci { reference: s.to_string(), blake3: None });
    }
    if s.starts_with("git+") {
        return Ok(Source::Git { url: s.to_string(), dir: None, rev: None });
    }
    if s.starts_with("https://") || s.starts_with("http://") {
        return Ok(Source::Http { url: s.to_string(), blake3: None });
    }

    Ok(Source::Unknown(s.to_string()))
}

/// Skeleton forge-URL parser — returns (owner, repo, path, optional rev).
/// Real implementation will decode `?ref=…&dir=…` query strings; this stub
/// only splits on `/` and pulls `?ref=` if present.
fn parse_forge(rest: &str, _kind: &str) -> Option<(String, String, PathBuf, Option<String>)> {
    let (path_part, rev) = match rest.split_once('?') {
        Some((p, q)) => {
            let rev = q
                .split('&')
                .find_map(|kv| kv.strip_prefix("ref="))
                .map(str::to_string);
            (p, rev)
        }
        None => (rest, None),
    };
    let mut parts = path_part.splitn(3, '/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    let path = PathBuf::from(parts.next().unwrap_or(""));
    Some((owner, repo, path, rev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github() {
        let s = parse("github:pleme-io/programs/hello-world/main.tlisp?ref=v0.1.0").unwrap();
        match s {
            Source::GitHub { owner, repo, path, rev } => {
                assert_eq!(owner, "pleme-io");
                assert_eq!(repo, "programs");
                assert_eq!(path, PathBuf::from("hello-world/main.tlisp"));
                assert_eq!(rev.as_deref(), Some("v0.1.0"));
            }
            _ => panic!("expected GitHub variant"),
        }
    }

    #[test]
    fn parses_oci() {
        let s = parse("oci://ghcr.io/pleme-io/programs:hello-v0.1.0").unwrap();
        assert!(matches!(s, Source::Oci { .. }));
    }

    #[test]
    fn parses_local() {
        let s = parse("./foo.tlisp").unwrap();
        assert!(matches!(s, Source::Local { .. }));
    }
}
