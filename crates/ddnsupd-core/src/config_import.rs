//! Pure, non-executing upstream configuration analysis. Source credentials and
//! arbitrary options never enter the normalized plan or its diagnostics.
use crate::{public_ip, valid_name};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteRecord {
    pub name: String,
    pub record_type: String,
    pub record_id: String,
    pub value: String,
    pub ttl: u32,
    pub proxied: bool,
    pub bindable: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub source_index: usize,
    pub fqdn: String,
    pub name: String,
    pub record_type: String,
    pub record_id: String,
    pub managed_ip: String,
    pub interval_seconds: i32,
    pub ttl: u32,
    pub proxied: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ImportPlan {
    pub provider_kind: String,
    pub zone_id: String,
    pub zone_name: String,
    pub items: Vec<ImportItem>,
}
pub struct ImportOptions<'a> {
    pub provider_kind: &'a str,
    pub zone_id: &'a str,
    pub zone_name: &'a str,
    pub ambiguous_family: Option<&'a str>,
    pub interval_seconds: i32,
}
#[derive(Debug, thiserror::Error)]
#[error("invalid or unsupported import configuration")]
pub struct ImportError;

pub fn analyze(
    config: &Value,
    options: &ImportOptions<'_>,
    remote: &[RemoteRecord],
) -> Result<ImportPlan, ImportError> {
    if !matches!(options.provider_kind, "cloudflare" | "hetznercloud")
        || !valid_name(options.zone_name, false)
        || !(60..=86400).contains(&options.interval_seconds)
        || options
            .ambiguous_family
            .is_some_and(|family| !matches!(family, "A" | "AAAA"))
    {
        return Err(ImportError);
    }
    let root = config.as_object().ok_or(ImportError)?;
    if root.keys().any(|key| key != "settings") {
        return Err(ImportError);
    }
    let settings = root
        .get("settings")
        .and_then(Value::as_array)
        .ok_or(ImportError)?;
    if settings.is_empty() || settings.len() > 200 {
        return Err(ImportError);
    }
    let mut plan = ImportPlan {
        provider_kind: options.provider_kind.into(),
        zone_id: options.zone_id.into(),
        zone_name: options.zone_name.into(),
        items: vec![],
    };
    for (source_index, setting) in settings.iter().enumerate() {
        let empty = serde_json::Map::new();
        let object = setting.as_object().unwrap_or(&empty);
        let domains = object.get("domain").and_then(Value::as_str).unwrap_or("");
        let owner = object
            .get("owner")
            .or_else(|| object.get("host"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let names: Vec<String> = if owner.is_empty() {
            domains.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            owner
                .split(',')
                .map(|s| {
                    if s.trim() == "@" {
                        domains.into()
                    } else {
                        format!("{}.{}", s.trim(), domains)
                    }
                })
                .collect()
        };
        if names.len() > 50 || plan.items.len() + names.len() > 200 {
            return Err(ImportError);
        }
        for domain in names {
            let mut item = ImportItem {
                source_index,
                fqdn: String::new(),
                name: String::new(),
                record_type: String::new(),
                record_id: String::new(),
                managed_ip: String::new(),
                interval_seconds: options.interval_seconds,
                ttl: 0,
                proxied: false,
                errors: vec![],
                warnings: vec![],
            };
            if !setting.is_object() {
                item.errors.push("invalid_entry".into())
            }
            let provider = object.get("provider").and_then(Value::as_str).unwrap_or("");
            if provider == "hetzner" {
                item.errors.push("legacy_provider_removed".into())
            } else if !matches!(provider, "hetznercloud" | "cloudflare") {
                item.errors.push("unsupported_provider".into())
            } else if provider != options.provider_kind {
                item.errors.push("provider_mismatch".into())
            }
            let allowed = [
                "provider",
                "domain",
                "owner",
                "host",
                "ip_version",
                "ipv6_suffix",
                "token",
                "email",
                "key",
                "user_service_key",
                "zone_identifier",
                "ttl",
                "proxied",
                "interval_seconds",
            ];
            if object.keys().any(|key| !allowed.contains(&key.as_str())) {
                item.errors.push("unsupported_option".into())
            }
            if ["token", "email", "key", "user_service_key"]
                .iter()
                .any(|key| object.contains_key(*key))
            {
                item.warnings.push("credentials_ignored".into())
            }
            if object.contains_key("host") || object.contains_key("owner") {
                item.warnings.push("legacy_name_converted".into())
            }
            if object.get("owner").is_some()
                && object.get("host").is_some()
                && object["owner"] != object["host"]
            {
                item.errors.push("conflicting_legacy_name".into())
            }
            for key in ["owner", "host"] {
                if object.get(key).is_some_and(|v| !v.is_string()) {
                    item.errors.push("invalid_entry".into())
                }
            }
            if object
                .get("ipv6_suffix")
                .is_some_and(|v| v.as_str() != Some(""))
            {
                item.errors.push("unsupported_ipv6_suffix".into())
            }
            if let Some(value) = object.get("interval_seconds") {
                match value.as_i64() {
                    Some(v) if (60..=86400).contains(&v) => item.interval_seconds = v as i32,
                    _ => item.errors.push("invalid_interval".into()),
                }
            }
            let domain = domain.strip_suffix('.').unwrap_or(&domain).to_lowercase();
            if valid_name(&domain, true) {
                if domain == options.zone_name {
                    item.fqdn = domain;
                    item.name = "@".into()
                } else if let Some(name) = domain.strip_suffix(&format!(".{}", options.zone_name)) {
                    item.fqdn = domain.clone();
                    item.name = name.into()
                } else {
                    item.errors.push("outside_zone".into())
                }
            } else {
                item.errors.push("invalid_domain".into())
            }
            item.record_type = match object
                .get("ip_version")
                .and_then(Value::as_str)
                .unwrap_or("ipv4 or ipv6")
            {
                "ipv4" => "A".into(),
                "ipv6" => "AAAA".into(),
                "ipv4 or ipv6" => {
                    item.warnings.push("family_selected_explicitly".into());
                    match options.ambiguous_family {
                        Some(family) => family.into(),
                        None => {
                            item.errors.push("family_choice_required".into());
                            String::new()
                        }
                    }
                }
                _ => {
                    item.errors.push("invalid_ip_version".into());
                    String::new()
                }
            };
            if object
                .get("ip_version")
                .is_some_and(|value| !value.is_string())
            {
                item.errors.push("invalid_ip_version".into())
            }
            if let Some(zone_id) = object.get("zone_identifier")
                && zone_id.as_str() != Some(options.zone_id)
            {
                item.errors.push("zone_identifier_mismatch".into())
            }
            if !item.name.is_empty() && !item.record_type.is_empty() {
                let matches: Vec<_> = remote
                    .iter()
                    .filter(|r| r.name == item.name && r.record_type == item.record_type)
                    .collect();
                match matches.as_slice() {
                    [] => item.errors.push("remote_record_missing".into()),
                    [record] => {
                        if !record.bindable || !public_ip(&record.value, &record.record_type) {
                            item.errors.push("invalid_existing_address".into())
                        }
                        item.record_id = record.record_id.clone();
                        item.managed_ip = record.value.clone();
                        item.ttl = record.ttl;
                        item.proxied = record.proxied;
                        if provider == "cloudflare" && record.record_id.is_empty() {
                            item.errors.push("remote_record_missing".into())
                        }
                        if object
                            .get("ttl")
                            .is_some_and(|v| v.as_u64() != Some(u64::from(record.ttl)))
                            || object
                                .get("proxied")
                                .is_some_and(|v| v.as_bool() != Some(record.proxied))
                        {
                            item.warnings.push("provider_settings_preserved".into())
                        }
                    }
                    _ => item.errors.push("ambiguous_remote_record".into()),
                }
            }
            item.warnings.push("import_paused".into());
            plan.items.push(item);
        }
    }
    let mut occurrences = HashMap::<(String, String), usize>::new();
    for item in &plan.items {
        if !item.name.is_empty() && !item.record_type.is_empty() {
            *occurrences
                .entry((item.name.clone(), item.record_type.clone()))
                .or_default() += 1
        }
    }
    for item in &mut plan.items {
        if occurrences
            .get(&(item.name.clone(), item.record_type.clone()))
            .is_some_and(|count| *count > 1)
        {
            item.errors.push("duplicate_entry".into())
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn options() -> ImportOptions<'static> {
        ImportOptions {
            provider_kind: "cloudflare",
            zone_id: "zone1",
            zone_name: "example.org",
            ambiguous_family: None,
            interval_seconds: 300,
        }
    }
    fn remote() -> Vec<RemoteRecord> {
        vec![RemoteRecord {
            name: "home".into(),
            record_type: "A".into(),
            record_id: "record1".into(),
            value: "8.8.8.8".into(),
            ttl: 300,
            proxied: true,
            bindable: true,
        }]
    }
    #[test]
    fn redacts_credentials_and_converts_legacy_names() {
        let config = json!({"settings":[{"provider":"cloudflare","domain":"example.org","host":"home","ip_version":"ipv4","token":"must-not-leak","ttl":60}]});
        let plan = analyze(&config, &options(), &remote()).unwrap();
        assert!(plan.items[0].errors.is_empty());
        assert_eq!(plan.items[0].fqdn, "home.example.org");
        assert!(
            plan.items[0]
                .warnings
                .contains(&"provider_settings_preserved".into())
        );
        assert!(
            !serde_json::to_string(&plan)
                .unwrap()
                .contains("must-not-leak")
        );
    }
    #[test]
    fn family_choice_is_required() {
        let config = json!({"settings":[{"provider":"cloudflare","domain":"home.example.org"}]});
        let plan = analyze(&config, &options(), &remote()).unwrap();
        assert!(
            plan.items[0]
                .errors
                .contains(&"family_choice_required".into())
        );
        let mut opts = options();
        opts.ambiguous_family = Some("A");
        assert!(
            analyze(&config, &opts, &remote()).unwrap().items[0]
                .errors
                .is_empty()
        );
    }
    #[test]
    fn rejects_unsafe_scope_and_options() {
        for entry in [
            json!({"provider":"cloudflare","domain":"evil.org","ip_version":"ipv4"}),
            json!({"provider":"custom","domain":"home.example.org","url":"http://127.0.0.1/secret"}),
            json!({"provider":"cloudflare","domain":"home.example.org","ipv6_suffix":"::1/64"}),
        ] {
            let plan = analyze(&json!({"settings":[entry]}), &options(), &remote()).unwrap();
            assert!(!plan.items[0].errors.is_empty());
            assert!(!serde_json::to_string(&plan).unwrap().contains("127.0.0.1"));
        }
    }
    #[test]
    fn duplicates_and_multiple_values_are_conflicts() {
        let item = json!({"provider":"cloudflare","domain":"home.example.org","ip_version":"ipv4"});
        let plan = analyze(
            &json!({"settings":[item.clone(),item.clone()]}),
            &options(),
            &remote(),
        )
        .unwrap();
        assert!(
            plan.items
                .iter()
                .all(|i| i.errors.contains(&"duplicate_entry".into()))
        );
        let mut records = remote();
        records.push(records[0].clone());
        assert!(
            analyze(&json!({"settings":[item]}), &options(), &records)
                .unwrap()
                .items[0]
                .errors
                .contains(&"ambiguous_remote_record".into())
        );
    }
}
