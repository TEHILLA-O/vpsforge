use vpsforge_core::{ForgeError, ForgeResult, TrustedSource};

/// Official, documented third-party sources. Never `curl | bash`.
pub fn known_sources() -> Vec<TrustedSource> {
    vec![
        docker_ce(),
        nvidia_container_toolkit(),
        uv(),
        ollama(),
        minio(),
    ]
}

pub fn source_for(id: &str) -> Option<TrustedSource> {
    match id {
        "docker" => Some(docker_ce()),
        "nvidia-container-toolkit" => Some(nvidia_container_toolkit()),
        "uv" => Some(uv()),
        "ollama" => Some(ollama()),
        "minio" => Some(minio()),
        _ => None,
    }
}

pub fn verify_source(source: &TrustedSource) -> ForgeResult<()> {
    if !source.url.starts_with("https://") {
        return Err(ForgeError::Trust(format!(
            "{} is not served over HTTPS: {}",
            source.name, source.url
        )));
    }
    if let Some(key) = &source.signing_key_url {
        if !key.starts_with("https://") {
            return Err(ForgeError::Trust(format!(
                "{} signing key is not HTTPS: {key}",
                source.name
            )));
        }
    }
    if !source.has_integrity() {
        tracing::warn!(
            source = %source.name,
            "no checksum or signing fingerprint registered; installer will require explicit acknowledgement"
        );
    }
    Ok(())
}

fn docker_ce() -> TrustedSource {
    let mut src = TrustedSource::new(
        "Docker CE",
        "https://docs.docker.com/engine/install/",
        "https://download.docker.com/linux/ubuntu",
    );
    src.signing_key_url = Some("https://download.docker.com/linux/ubuntu/gpg".into());
    src.fingerprint = Some("9DC8 5822 9FC7 DD38 854A E2D8 8D81 803C 0EBF CD88".into());
    src.notes = "Official Docker APT/YUM repository. Key fingerprint is published by Docker.".into();
    src
}

fn nvidia_container_toolkit() -> TrustedSource {
    let mut src = TrustedSource::new(
        "NVIDIA Container Toolkit",
        "https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html",
        "https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list",
    );
    src.signing_key_url = Some(
        "https://nvidia.github.io/libnvidia-container/gpgkey".into(),
    );
    src.notes =
        "NVIDIA-published repository. Added only when an NVIDIA GPU is detected.".into();
    src
}

fn uv() -> TrustedSource {
    let mut src = TrustedSource::new(
        "uv",
        "https://github.com/astral-sh/uv",
        "https://github.com/astral-sh/uv/releases",
    );
    src.checksum_url = Some(
        "https://github.com/astral-sh/uv/releases (SHA256SUMS attached to each release)".into(),
    );
    src.notes =
        "Downloaded from GitHub Releases and checksum-verified. Installer never pipes the curl installer into bash.".into();
    src
}

fn ollama() -> TrustedSource {
    let mut src = TrustedSource::new(
        "Ollama",
        "https://ollama.com",
        "https://ollama.com/download",
    );
    src.notes =
        "Official Ollama packages. VPSForge downloads the published archive, shows the source, and verifies HTTPS before install.".into();
    src
}

fn minio() -> TrustedSource {
    let mut src = TrustedSource::new(
        "MinIO",
        "https://min.io",
        "https://dl.min.io/server/minio/release/",
    );
    src.notes = "Official MinIO binary distribution over HTTPS.".into();
    src
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_is_https_and_signed() {
        let src = docker_ce();
        assert!(src.requires_https());
        assert!(src.has_integrity());
        assert!(verify_source(&src).is_ok());
    }
}
