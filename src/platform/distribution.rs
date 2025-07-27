use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

// Allow dead code for distribution detection functionality that will be used by TUI in future
#[allow(dead_code)]
/// Supported Linux distributions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionType {
    Ubuntu,
    Debian,
    Arch,
    Manjaro,
    Fedora,
    OpenSUSE,
    NixOS,
    Gentoo,
    Alpine,
    CentOS,
    Rhel,
    Unknown(String),
}

impl std::fmt::Display for DistributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionType::Ubuntu => write!(f, "Ubuntu"),
            DistributionType::Debian => write!(f, "Debian"),
            DistributionType::Arch => write!(f, "Arch Linux"),
            DistributionType::Manjaro => write!(f, "Manjaro"),
            DistributionType::Fedora => write!(f, "Fedora"),
            DistributionType::OpenSUSE => write!(f, "openSUSE"),
            DistributionType::NixOS => write!(f, "NixOS"),
            DistributionType::Gentoo => write!(f, "Gentoo"),
            DistributionType::Alpine => write!(f, "Alpine Linux"),
            DistributionType::CentOS => write!(f, "CentOS"),
            DistributionType::Rhel => write!(f, "Red Hat Enterprise Linux"),
            DistributionType::Unknown(name) => write!(f, "Unknown ({})", name),
        }
    }
}

/// Information about the detected distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionInfo {
    pub distribution_type: DistributionType,
    pub version: Option<String>,
    pub version_id: Option<String>,
    pub name: String,
    pub pretty_name: Option<String>,
    pub id: String,
    pub id_like: Option<Vec<String>>,
    pub home_url: Option<String>,
    pub support_url: Option<String>,
    pub bug_report_url: Option<String>,
}

impl DistributionInfo {
    /// Check if this distribution is based on another distribution
    pub fn is_based_on(&self, base: &DistributionType) -> bool {
        if &self.distribution_type == base {
            return true;
        }

        if let Some(ref id_like) = self.id_like {
            let base_id = match base {
                DistributionType::Debian => "debian",
                DistributionType::Ubuntu => "ubuntu",
                DistributionType::Arch => "arch",
                DistributionType::Fedora => "fedora",
                DistributionType::Rhel => "rhel",
                _ => return false,
            };

            id_like.iter().any(|id| id == base_id)
        } else {
            false
        }
    }
}

/// Cache for distribution detection results
static DISTRIBUTION_CACHE: Mutex<Option<DistributionInfo>> = Mutex::new(None);

/// Distribution detector that parses system files to identify the Linux distribution
pub struct DistributionDetector;

impl DistributionDetector {
    /// Detect the current Linux distribution
    pub fn detect() -> Result<DistributionInfo> {
        // Check cache first
        {
            let cache = DISTRIBUTION_CACHE.lock().unwrap();
            if let Some(ref cached) = *cache {
                return Ok(cached.clone());
            }
        }

        // Detect distribution
        let info = Self::detect_internal()?;

        // Cache the result
        {
            let mut cache = DISTRIBUTION_CACHE.lock().unwrap();
            *cache = Some(info.clone());
        }

        Ok(info)
    }

    /// Clear the detection cache (useful for testing)
    pub fn clear_cache() {
        let mut cache = DISTRIBUTION_CACHE.lock().unwrap();
        *cache = None;
    }

    /// Internal detection logic
    fn detect_internal() -> Result<DistributionInfo> {
        // Try os-release first (systemd standard)
        if let Ok(info) = Self::parse_os_release("/etc/os-release") {
            return Ok(info);
        }

        // Fallback to usr/lib/os-release
        if let Ok(info) = Self::parse_os_release("/usr/lib/os-release") {
            return Ok(info);
        }

        // Fallback to lsb-release
        if let Ok(info) = Self::parse_lsb_release("/etc/lsb-release") {
            return Ok(info);
        }

        // Last resort: use os_info crate
        Self::detect_with_os_info()
    }

    /// Parse /etc/os-release or /usr/lib/os-release file
    fn parse_os_release<P: AsRef<Path>>(path: P) -> Result<DistributionInfo> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read {:?}", path.as_ref()))?;

        let mut fields = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = Self::unquote_value(value.trim());
                fields.insert(key.to_string(), value);
            }
        }

        let id = fields
            .get("ID")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let name = fields.get("NAME").cloned().unwrap_or_else(|| id.clone());

        let distribution_type = Self::determine_distribution_type(&id, &fields);

        let id_like = fields
            .get("ID_LIKE")
            .map(|value| value.split_whitespace().map(|s| s.to_string()).collect());

        Ok(DistributionInfo {
            distribution_type,
            version: fields.get("VERSION").cloned(),
            version_id: fields.get("VERSION_ID").cloned(),
            name,
            pretty_name: fields.get("PRETTY_NAME").cloned(),
            id,
            id_like,
            home_url: fields.get("HOME_URL").cloned(),
            support_url: fields.get("SUPPORT_URL").cloned(),
            bug_report_url: fields.get("BUG_REPORT_URL").cloned(),
        })
    }

    /// Parse /etc/lsb-release file (fallback)
    fn parse_lsb_release<P: AsRef<Path>>(path: P) -> Result<DistributionInfo> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read {:?}", path.as_ref()))?;

        let mut fields = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = Self::unquote_value(value.trim());
                fields.insert(key.to_string(), value);
            }
        }

        let id = fields
            .get("DISTRIB_ID")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
            .to_lowercase();

        let name = fields
            .get("DISTRIB_DESCRIPTION")
            .or_else(|| fields.get("DISTRIB_ID"))
            .cloned()
            .unwrap_or_else(|| id.clone());

        let distribution_type = Self::determine_distribution_type(&id, &fields);

        Ok(DistributionInfo {
            distribution_type,
            version: fields.get("DISTRIB_RELEASE").cloned(),
            version_id: fields.get("DISTRIB_RELEASE").cloned(),
            name,
            pretty_name: fields.get("DISTRIB_DESCRIPTION").cloned(),
            id,
            id_like: None,
            home_url: None,
            support_url: None,
            bug_report_url: None,
        })
    }

    /// Use os_info crate as last resort
    fn detect_with_os_info() -> Result<DistributionInfo> {
        let info = os_info::get();
        let os_type = info.os_type();
        let version = info.version().to_string();

        let (distribution_type, id, name) = match os_type {
            os_info::Type::Ubuntu => (
                DistributionType::Ubuntu,
                "ubuntu".to_string(),
                "Ubuntu".to_string(),
            ),
            os_info::Type::Debian => (
                DistributionType::Debian,
                "debian".to_string(),
                "Debian".to_string(),
            ),
            os_info::Type::Arch => (
                DistributionType::Arch,
                "arch".to_string(),
                "Arch Linux".to_string(),
            ),
            os_info::Type::Fedora => (
                DistributionType::Fedora,
                "fedora".to_string(),
                "Fedora".to_string(),
            ),
            os_info::Type::CentOS => (
                DistributionType::CentOS,
                "centos".to_string(),
                "CentOS".to_string(),
            ),
            os_info::Type::Alpine => (
                DistributionType::Alpine,
                "alpine".to_string(),
                "Alpine Linux".to_string(),
            ),
            _ => {
                let name = format!("{:?}", os_type);
                (
                    DistributionType::Unknown(name.clone()),
                    "unknown".to_string(),
                    name,
                )
            }
        };

        Ok(DistributionInfo {
            distribution_type,
            version: Some(version.clone()),
            version_id: Some(version),
            name,
            pretty_name: None,
            id,
            id_like: None,
            home_url: None,
            support_url: None,
            bug_report_url: None,
        })
    }

    /// Determine distribution type from ID and fields
    fn determine_distribution_type(id: &str, fields: &HashMap<String, String>) -> DistributionType {
        match id.to_lowercase().as_str() {
            "ubuntu" => DistributionType::Ubuntu,
            "debian" => DistributionType::Debian,
            "arch" => DistributionType::Arch,
            "manjaro" => DistributionType::Manjaro,
            "fedora" => DistributionType::Fedora,
            "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => DistributionType::OpenSUSE,
            "nixos" => DistributionType::NixOS,
            "gentoo" => DistributionType::Gentoo,
            "alpine" => DistributionType::Alpine,
            "centos" => DistributionType::CentOS,
            "rhel" => DistributionType::Rhel,
            _ => {
                // Check ID_LIKE for derived distributions
                // Priority order: more specific distributions first
                if let Some(id_like) = fields.get("ID_LIKE") {
                    let likes: Vec<&str> = id_like.split_whitespace().collect();

                    // Check for Ubuntu first (more specific than Debian)
                    if likes.contains(&"ubuntu") {
                        return DistributionType::Ubuntu;
                    }

                    // Then check other distributions
                    for like in likes {
                        match like {
                            "debian" => return DistributionType::Debian,
                            "arch" => return DistributionType::Arch,
                            "fedora" => return DistributionType::Fedora,
                            "rhel" => return DistributionType::Rhel,
                            _ => {}
                        }
                    }
                }

                let name = fields
                    .get("NAME")
                    .cloned()
                    .unwrap_or_else(|| id.to_string());
                DistributionType::Unknown(name)
            }
        }
    }

    /// Remove quotes from values in key=value pairs
    fn unquote_value(value: &str) -> String {
        let value = value.trim();

        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_unquote_value() {
        assert_eq!(
            DistributionDetector::unquote_value("\"Ubuntu 20.04\""),
            "Ubuntu 20.04"
        );
        assert_eq!(
            DistributionDetector::unquote_value("'Arch Linux'"),
            "Arch Linux"
        );
        assert_eq!(DistributionDetector::unquote_value("Fedora"), "Fedora");
        assert_eq!(DistributionDetector::unquote_value(""), "");
    }

    #[test]
    fn test_determine_distribution_type() {
        let mut fields = HashMap::new();

        assert_eq!(
            DistributionDetector::determine_distribution_type("ubuntu", &fields),
            DistributionType::Ubuntu
        );

        assert_eq!(
            DistributionDetector::determine_distribution_type("debian", &fields),
            DistributionType::Debian
        );

        assert_eq!(
            DistributionDetector::determine_distribution_type("arch", &fields),
            DistributionType::Arch
        );

        assert_eq!(
            DistributionDetector::determine_distribution_type("nixos", &fields),
            DistributionType::NixOS
        );

        // Test ID_LIKE fallback
        fields.insert("ID_LIKE".to_string(), "debian ubuntu".to_string());
        assert_eq!(
            DistributionDetector::determine_distribution_type("pop", &fields),
            DistributionType::Ubuntu
        );

        fields.insert("NAME".to_string(), "Pop!_OS".to_string());
        assert_eq!(
            DistributionDetector::determine_distribution_type("unknown", &HashMap::new()),
            DistributionType::Unknown("unknown".to_string())
        );
    }

    #[test]
    fn test_parse_ubuntu_os_release() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let os_release_path = temp_dir.path().join("os-release");

        let content = r#"
NAME="Ubuntu"
VERSION="20.04.3 LTS (Focal Fossa)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 20.04.3 LTS"
VERSION_ID="20.04"
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
"#;

        fs::write(&os_release_path, content)?;

        let info = DistributionDetector::parse_os_release(&os_release_path)?;

        assert_eq!(info.distribution_type, DistributionType::Ubuntu);
        assert_eq!(info.id, "ubuntu");
        assert_eq!(info.name, "Ubuntu");
        assert_eq!(info.version_id, Some("20.04".to_string()));
        assert_eq!(info.pretty_name, Some("Ubuntu 20.04.3 LTS".to_string()));
        assert_eq!(info.id_like, Some(vec!["debian".to_string()]));

        Ok(())
    }

    #[test]
    fn test_parse_arch_os_release() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let os_release_path = temp_dir.path().join("os-release");

        let content = r#"
NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
ID=arch
BUILD_ID=rolling
ANSI_COLOR="38;2;23;147;209"
HOME_URL="https://archlinux.org/"
DOCUMENTATION_URL="https://wiki.archlinux.org/"
SUPPORT_URL="https://bbs.archlinux.org/"
BUG_REPORT_URL="https://bugs.archlinux.org/"
LOGO=archlinux-logo
"#;

        fs::write(&os_release_path, content)?;

        let info = DistributionDetector::parse_os_release(&os_release_path)?;

        assert_eq!(info.distribution_type, DistributionType::Arch);
        assert_eq!(info.id, "arch");
        assert_eq!(info.name, "Arch Linux");
        assert_eq!(info.pretty_name, Some("Arch Linux".to_string()));
        assert_eq!(info.id_like, None);

        Ok(())
    }

    #[test]
    fn test_parse_nixos_os_release() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let os_release_path = temp_dir.path().join("os-release");

        let content = r#"
BUG_REPORT_URL="https://github.com/NixOS/nixpkgs/issues"
BUILD_ID="23.11.20231130.316cf8e"
DOCUMENTATION_URL="https://nixos.org/learn.html"
HOME_URL="https://nixos.org/"
ID=nixos
LOGO="nix-snowflake"
NAME=NixOS
PRETTY_NAME="NixOS 23.11 (Tapir)"
SUPPORT_URL="https://nixos.org/community.html"
VERSION="23.11 (Tapir)"
VERSION_CODENAME=tapir
VERSION_ID="23.11"
"#;

        fs::write(&os_release_path, content)?;

        let info = DistributionDetector::parse_os_release(&os_release_path)?;

        assert_eq!(info.distribution_type, DistributionType::NixOS);
        assert_eq!(info.id, "nixos");
        assert_eq!(info.name, "NixOS");
        assert_eq!(info.version_id, Some("23.11".to_string()));
        assert_eq!(info.pretty_name, Some("NixOS 23.11 (Tapir)".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_fedora_os_release() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let os_release_path = temp_dir.path().join("os-release");

        let content = r#"
NAME="Fedora Linux"
VERSION="39 (Workstation Edition)"
ID=fedora
VERSION_ID=39
VERSION_CODENAME=""
PLATFORM_ID="platform:f39"
PRETTY_NAME="Fedora Linux 39 (Workstation Edition)"
ANSI_COLOR="0;38;2;60;110;180"
LOGO=fedora-logo-icon
CPE_NAME="cpe:/o:fedoraproject:fedora:39"
DEFAULT_HOSTNAME="fedora"
HOME_URL="https://fedoraproject.org/"
DOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/latest/system-administrators-guide/"
SUPPORT_URL="https://ask.fedoraproject.org/"
BUG_REPORT_URL="https://bugzilla.redhat.com/"
REDHAT_BUGZILLA_PRODUCT="Fedora"
REDHAT_BUGZILLA_PRODUCT_VERSION=39
REDHAT_SUPPORT_PRODUCT="Fedora"
REDHAT_SUPPORT_PRODUCT_VERSION=39
"#;

        fs::write(&os_release_path, content)?;

        let info = DistributionDetector::parse_os_release(&os_release_path)?;

        assert_eq!(info.distribution_type, DistributionType::Fedora);
        assert_eq!(info.id, "fedora");
        assert_eq!(info.name, "Fedora Linux");
        assert_eq!(info.version_id, Some("39".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_lsb_release() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lsb_release_path = temp_dir.path().join("lsb-release");

        let content = r#"
DISTRIB_ID=Ubuntu
DISTRIB_RELEASE=20.04
DISTRIB_CODENAME=focal
DISTRIB_DESCRIPTION="Ubuntu 20.04.3 LTS"
"#;

        fs::write(&lsb_release_path, content)?;

        let info = DistributionDetector::parse_lsb_release(&lsb_release_path)?;

        assert_eq!(info.distribution_type, DistributionType::Ubuntu);
        assert_eq!(info.id, "ubuntu");
        assert_eq!(info.name, "Ubuntu 20.04.3 LTS");
        assert_eq!(info.version_id, Some("20.04".to_string()));

        Ok(())
    }

    #[test]
    fn test_distribution_info_is_based_on() {
        let ubuntu_info = DistributionInfo {
            distribution_type: DistributionType::Ubuntu,
            version: None,
            version_id: None,
            name: "Ubuntu".to_string(),
            pretty_name: None,
            id: "ubuntu".to_string(),
            id_like: Some(vec!["debian".to_string()]),
            home_url: None,
            support_url: None,
            bug_report_url: None,
        };

        assert!(ubuntu_info.is_based_on(&DistributionType::Ubuntu));
        assert!(ubuntu_info.is_based_on(&DistributionType::Debian));
        assert!(!ubuntu_info.is_based_on(&DistributionType::Arch));

        let arch_info = DistributionInfo {
            distribution_type: DistributionType::Arch,
            version: None,
            version_id: None,
            name: "Arch Linux".to_string(),
            pretty_name: None,
            id: "arch".to_string(),
            id_like: None,
            home_url: None,
            support_url: None,
            bug_report_url: None,
        };

        assert!(arch_info.is_based_on(&DistributionType::Arch));
        assert!(!arch_info.is_based_on(&DistributionType::Debian));
    }

    #[test]
    fn test_distribution_type_display() {
        assert_eq!(DistributionType::Ubuntu.to_string(), "Ubuntu");
        assert_eq!(DistributionType::Arch.to_string(), "Arch Linux");
        assert_eq!(DistributionType::NixOS.to_string(), "NixOS");
        assert_eq!(
            DistributionType::Unknown("Custom".to_string()).to_string(),
            "Unknown (Custom)"
        );
    }
}
