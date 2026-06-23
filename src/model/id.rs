use std::fmt;

use anyhow::{Result, bail};

macro_rules! string_id {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if value.is_empty()
                    || !value
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
                {
                    bail!("invalid {} identifier '{}'", $label, value);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

string_id!(ProxyId, "proxy");
string_id!(CertificateId, "certificate");
string_id!(HostId, "host");
string_id!(ServiceId, "service");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainName(String);

impl DomainName {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let normalized = value.trim_end_matches('.').to_ascii_lowercase();
        let labels = normalized.strip_prefix("*.").unwrap_or(&normalized);
        let valid = !labels.is_empty()
            && normalized.len() <= 253
            && labels.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '-')
            });
        if !valid {
            bail!("invalid domain name '{}'", value);
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_reject_ambiguous_values() {
        assert!(ProxyId::new("edge-proxy").is_ok());
        assert!(ProxyId::new("edge proxy").is_err());
        assert!(CertificateId::new("").is_err());
    }

    #[test]
    fn domains_are_normalized_and_validated() {
        assert_eq!(
            DomainName::new("Jellyfin.Suhail.Ink.").unwrap().as_str(),
            "jellyfin.suhail.ink"
        );
        assert!(DomainName::new("bad_domain.suhail.ink").is_err());
    }
}
