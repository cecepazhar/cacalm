//! CACalm Pro license verification and management.
//!
//! Provides offline cryptographic license verification using HMAC-SHA256,
//! cache persistence in `settings/pro_cache.json`, feature gating models,
//! and activation / deactivation lifecycle functions.

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::path::{Path, PathBuf};

type HmacSha256 = Hmac<Sha256>;

pub const FEATURE_ALL_CHANNELS: &str = "all_channels";
pub const FEATURE_SPATIAL_PAN: &str = "spatial_audio_pan";
pub const FEATURE_UNLIMITED_PRESETS: &str = "unlimited_presets";
pub const FEATURE_CUSTOM_SOUND_IMPORT: &str = "custom_sound_import";

pub const FREE_CHANNELS: &[&str] = &[
    "rain",
    "waves",
    "wind",
    "white_noise",
];

pub const ALL_CHANNELS: &[&str] = &[
    "rain",
    "waves",
    "wind",
    "white_noise",
    "birds",
    "fireplace",
    "cafe",
    "stream",
    "binaural_beats",
];

pub const DEFAULT_FREE_PRESET_LIMIT: usize = 2;

/// Default compile-time secret salt for offline license signature verification.
/// Can be overridden via environment variable `CACALM_PRO_SECRET` at runtime.
const DEFAULT_SIGNING_SECRET: &[u8] = b"cacalm_pro_license_secret_key_v1";

/// Pre-configured founder lifetime license key.
pub const FOUNDER_LIFETIME_KEY: &str =
    "CACALM-LIFETIME-FOUNDER-0-be60fb3c6450e73aacc2cad6863c7911dbad73779a170fb504cdaa4523559bc5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProStatus {
    pub is_pro: bool,
    pub plan: String,
    pub license_key: Option<String>,
    pub expires_at: Option<i64>,
    pub features: Vec<String>,
}

impl ProStatus {
    pub fn free() -> Self {
        Self {
            is_pro: false,
            plan: "free".to_string(),
            license_key: None,
            expires_at: None,
            features: vec![
                "rain".to_string(),
                "waves".to_string(),
                "wind".to_string(),
                "white_noise".to_string(),
                "standard_timer".to_string(),
                "preset_limit_2".to_string(),
            ],
        }
    }

    pub fn pro(plan: impl Into<String>, license_key: Option<String>, expires_at: Option<i64>) -> Self {
        Self {
            is_pro: true,
            plan: plan.into(),
            license_key,
            expires_at,
            features: vec![
                FEATURE_ALL_CHANNELS.to_string(),
                FEATURE_SPATIAL_PAN.to_string(),
                FEATURE_UNLIMITED_PRESETS.to_string(),
                FEATURE_CUSTOM_SOUND_IMPORT.to_string(),
                "rain".to_string(),
                "waves".to_string(),
                "wind".to_string(),
                "white_noise".to_string(),
                "birds".to_string(),
                "fireplace".to_string(),
                "cafe".to_string(),
                "stream".to_string(),
                "binaural_beats".to_string(),
                "standard_timer".to_string(),
            ],
        }
    }

    pub fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if self.is_pro {
            return true;
        }
        let ch = channel_id.trim().to_lowercase();
        FREE_CHANNELS.iter().any(|&c| c == ch)
    }

    pub fn max_presets(&self) -> Option<usize> {
        if self.is_pro {
            None
        } else {
            Some(DEFAULT_FREE_PRESET_LIMIT)
        }
    }

    pub fn can_use_spatial_pan(&self) -> bool {
        self.is_pro
    }

    pub fn can_import_custom_sounds(&self) -> bool {
        self.is_pro
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f.eq_ignore_ascii_case(feature))
    }
}

impl Default for ProStatus {
    fn default() -> Self {
        Self::free()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProCache {
    pub license_key: String,
    pub activated_at: i64,
    pub status: ProStatus,
    pub cache_signature: String,
}

fn signing_secret() -> Vec<u8> {
    std::env::var("CACALM_PRO_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| DEFAULT_SIGNING_SECRET.to_vec())
}

pub fn compute_hmac_hex(secret: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

fn verify_hmac(secret: &[u8], data: &[u8], sig_hex: &str) -> Result<(), String> {
    let sig_bytes = hex::decode(sig_hex.trim())
        .map_err(|_| "Invalid signature encoding (expected hex string)".to_string())?;
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.verify_slice(&sig_bytes)
        .map_err(|_| "Cryptographic signature verification failed".to_string())
}

/// Generate an offline license key with HMAC-SHA256 signature.
/// Format: `CACALM-<PLAN>-<CLIENT_ID>-<EXPIRY_UNIX_OR_0>-<SIG_HEX>`
pub fn generate_license_key(plan: &str, expires_at: Option<i64>, client_id: &str) -> String {
    let clean_plan = plan.trim().to_uppercase();
    let clean_id = client_id.trim().to_uppercase();
    let exp_val = expires_at.unwrap_or(0);
    let payload = format!("CACALM-{clean_plan}-{clean_id}-{exp_val}");
    let secret = signing_secret();
    let sig_hex = compute_hmac_hex(&secret, payload.as_bytes());
    format!("{payload}-{sig_hex}")
}

/// Verify a license key offline using cryptographic HMAC-SHA256 verification and expiry checking.
pub fn verify_license(key: &str) -> Result<ProStatus, String> {
    let clean_key = key.trim();
    if clean_key.is_empty() {
        return Err("License key cannot be empty".to_string());
    }

    let secret = signing_secret();

    // Support Format 1: Dash separated `(CACALM|CALM)-<PLAN>-<CLIENT_ID...>-<EXPIRY>-<SIG_HEX>`
    if clean_key.starts_with("CACALM-")
        || clean_key.starts_with("CALM-")
        || clean_key.starts_with("cacalm-")
        || clean_key.starts_with("calm-")
    {
        let last_dash = clean_key
            .rfind('-')
            .ok_or_else(|| "Invalid license key format: missing delimiter".to_string())?;
        if last_dash == 0 || last_dash == clean_key.len() - 1 {
            return Err("Invalid license key format".to_string());
        }

        let payload_part = &clean_key[..last_dash];
        let sig_part = &clean_key[last_dash + 1..];

        let normalized_payload = payload_part.to_uppercase();
        verify_hmac(&secret, normalized_payload.as_bytes(), sig_part)?;

        let parts: Vec<&str> = normalized_payload.split('-').collect();
        if parts.len() < 4 {
            return Err("Invalid license key payload: insufficient segments".to_string());
        }

        let plan_raw = parts[1];
        let expiry_str = parts[parts.len() - 1];
        let expiry_val: i64 = expiry_str.parse().map_err(|_| {
            format!("Invalid expiration timestamp in license key: '{expiry_str}'")
        })?;

        let now = Utc::now().timestamp();
        if expiry_val > 0 && now > expiry_val {
            return Err(format!("License expired at {expiry_val}, current timestamp {now}"));
        }

        let expires_at = if expiry_val > 0 { Some(expiry_val) } else { None };
        let plan = match plan_raw {
            "LIFE" | "LIFETIME" | "FOUNDER" => "lifetime",
            "MONTH" | "MONTHLY" | "SUB" => "monthly",
            "YEAR" | "ANNUAL" => "annual",
            other => other,
        }
        .to_lowercase();

        return Ok(ProStatus::pro(plan, Some(clean_key.to_string()), expires_at));
    }

    // Support Format 2: Dot separated `CACALM.<PAYLOAD_HEX>.<SIG_HEX>` or `<PAYLOAD_HEX>.<SIG_HEX>`
    if clean_key.contains('.') {
        let parts: Vec<&str> = clean_key.split('.').collect();
        let (payload_str, sig_str) = match parts.len() {
            2 => (parts[0], parts[1]),
            3 if parts[0].eq_ignore_ascii_case("CACALM") || parts[0].eq_ignore_ascii_case("CALM") => {
                (parts[1], parts[2])
            }
            _ => return Err("Invalid token format: expected 2 or 3 dot-separated parts".to_string()),
        };

        verify_hmac(&secret, payload_str.as_bytes(), sig_str)?;

        let payload_bytes = hex::decode(payload_str)
            .map_err(|_| "Failed to decode license payload hex".to_string())?;
        let payload_json = String::from_utf8(payload_bytes)
            .map_err(|_| "Invalid UTF-8 in license payload".to_string())?;

        #[derive(Deserialize)]
        struct TokenPayload {
            plan: String,
            #[serde(default)]
            expires_at: Option<i64>,
        }

        let token: TokenPayload = serde_json::from_str(&payload_json)
            .map_err(|e| format!("Failed to parse license JSON payload: {e}"))?;

        let now = Utc::now().timestamp();
        if let Some(exp) = token.expires_at
            && exp > 0
            && now > exp
        {
            return Err(format!("License expired at {exp}, current timestamp {now}"));
        }

        return Ok(ProStatus::pro(token.plan, Some(clean_key.to_string()), token.expires_at));
    }

    Err("Invalid license key format: unknown schema".to_string())
}

/// Primary cache file path: `settings/pro_cache.json` in the app data dir.
pub fn pro_cache_path() -> Result<PathBuf, std::io::Error> {
    let data_dir = crate::paths::app_data_dir()?;
    let settings_dir = data_dir.join("settings");
    if !settings_dir.exists() {
        let _ = fs::create_dir_all(&settings_dir);
    }
    Ok(settings_dir.join("pro_cache.json"))
}

/// Fallback legacy cache file path: `pro_cache.json` in root app data dir.
pub fn legacy_pro_cache_path() -> Result<PathBuf, std::io::Error> {
    let data_dir = crate::paths::app_data_dir()?;
    Ok(data_dir.join("pro_cache.json"))
}

/// Activate a license key and write the signed cache to the specified file path.
pub fn activate_license_with_path(key: &str, cache_path: &Path) -> Result<ProStatus, String> {
    let status = verify_license(key)?;
    let now = Utc::now().timestamp();
    let cache_data = format!("{}:{}:{}", key.trim(), now, status.plan);
    let secret = signing_secret();
    let cache_signature = compute_hmac_hex(&secret, cache_data.as_bytes());

    let cache = ProCache {
        license_key: key.trim().to_string(),
        activated_at: now,
        status: status.clone(),
        cache_signature,
    };

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    fs::write(cache_path, json).map_err(|e| e.to_string())?;
    Ok(status)
}

/// Load and verify pro status from a cache file path.
pub fn get_pro_status_from_path(cache_path: &Path) -> ProStatus {
    if !cache_path.exists() {
        return ProStatus::free();
    }

    let Ok(content) = fs::read_to_string(cache_path) else {
        return ProStatus::free();
    };

    let Ok(cache) = serde_json::from_str::<ProCache>(&content) else {
        return ProStatus::free();
    };

    // Re-verify the cached key offline (ensures signature is still valid and not expired)
    verify_license(&cache.license_key).unwrap_or_default()
}

/// Deactivate license by removing the cache file at the specified path.
pub fn deactivate_license_at_path(cache_path: &Path) -> Result<(), String> {
    if cache_path.exists() {
        fs::remove_file(cache_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Activate a license key and persist it into the app's `settings/pro_cache.json`.
pub fn activate_license(key: &str) -> Result<ProStatus, String> {
    let path = pro_cache_path().map_err(|e| e.to_string())?;
    activate_license_with_path(key, &path)
}

/// Deactivate the current license and remove cached files.
pub fn deactivate_license() -> Result<(), String> {
    let mut err = None;
    if let Ok(path) = pro_cache_path()
        && let Err(e) = deactivate_license_at_path(&path)
    {
        err = Some(e);
    }
    if let Ok(legacy) = legacy_pro_cache_path() {
        let _ = deactivate_license_at_path(&legacy);
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(())
    }
}

/// Retrieve the active Pro status, reading and re-verifying the offline cache.
pub fn get_pro_status() -> ProStatus {
    if let Ok(path) = pro_cache_path()
        && path.exists()
    {
        let status = get_pro_status_from_path(&path);
        if status.is_pro {
            return status;
        }
    }
    if let Ok(legacy) = legacy_pro_cache_path()
        && legacy.exists()
    {
        let status = get_pro_status_from_path(&legacy);
        if status.is_pro {
            return status;
        }
    }
    ProStatus::free()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_free_tier() {
        let status = ProStatus::free();
        assert!(!status.is_pro);
        assert_eq!(status.plan, "free");
        assert!(status.license_key.is_none());
        assert!(status.expires_at.is_none());
        assert_eq!(status.max_presets(), Some(2));
        assert!(!status.can_use_spatial_pan());
        assert!(!status.can_import_custom_sounds());

        // Free channels allowed
        assert!(status.is_channel_allowed("rain"));
        assert!(status.is_channel_allowed("waves"));
        assert!(status.is_channel_allowed("wind"));
        assert!(status.is_channel_allowed("white_noise"));

        // Pro channels not allowed
        assert!(!status.is_channel_allowed("birds"));
        assert!(!status.is_channel_allowed("fireplace"));
        assert!(!status.is_channel_allowed("binaural_beats"));
    }

    #[test]
    fn test_valid_lifetime_license() {
        let key = generate_license_key("lifetime", None, "CECEP-AZHAR");
        let status = verify_license(&key).expect("Key should verify successfully");

        assert!(status.is_pro);
        assert_eq!(status.plan, "lifetime");
        assert_eq!(status.license_key, Some(key));
        assert!(status.expires_at.is_none());
        assert_eq!(status.max_presets(), None); // unlimited
        assert!(status.can_use_spatial_pan());
        assert!(status.can_import_custom_sounds());

        // All channels allowed for Pro
        assert!(status.is_channel_allowed("rain"));
        assert!(status.is_channel_allowed("birds"));
        assert!(status.is_channel_allowed("fireplace"));
        assert!(status.is_channel_allowed("cafe"));
        assert!(status.is_channel_allowed("stream"));
        assert!(status.is_channel_allowed("binaural_beats"));
    }

    #[test]
    fn test_valid_subscription_license() {
        let future_exp = Utc::now().timestamp() + 86400 * 30; // 30 days later
        let key = generate_license_key("monthly", Some(future_exp), "USER-001");
        let status = verify_license(&key).expect("Active subscription should verify");

        assert!(status.is_pro);
        assert_eq!(status.plan, "monthly");
        assert_eq!(status.expires_at, Some(future_exp));
        assert!(status.has_feature(FEATURE_ALL_CHANNELS));
        assert!(status.has_feature(FEATURE_SPATIAL_PAN));
        assert!(status.has_feature(FEATURE_UNLIMITED_PRESETS));
        assert!(status.has_feature(FEATURE_CUSTOM_SOUND_IMPORT));
    }

    #[test]
    fn test_expired_subscription_license() {
        let past_exp = Utc::now().timestamp() - 3600; // 1 hour ago
        let key = generate_license_key("monthly", Some(past_exp), "USER-002");
        let result = verify_license(&key);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("License expired"), "Error was: {err}");
    }

    #[test]
    fn test_tampered_signature_fails() {
        let key = generate_license_key("lifetime", None, "ALICE");
        let mut tampered = key.clone();
        // Mutate the last character of the signature
        let last_char = tampered.pop().unwrap();
        let replacement = if last_char == '0' { '1' } else { '0' };
        tampered.push(replacement);

        let result = verify_license(&tampered);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Cryptographic signature verification failed"),
            "Error was: {err}"
        );
    }

    #[test]
    fn test_tampered_payload_fails() {
        let key = generate_license_key("monthly", Some(2000000000), "BOB");
        // Replace 'BOB' with 'MALLORY' without re-signing
        let tampered = key.replace("BOB", "MALLORY");

        let result = verify_license(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_formats() {
        assert!(verify_license("").is_err());
        assert!(verify_license("    ").is_err());
        assert!(verify_license("INVALID-KEY").is_err());
        assert!(verify_license("CACALM-RANDOM").is_err());
        assert!(verify_license("CALM-SOMETHING-NOT-ENOUGH-PARTS").is_err());
    }

    #[test]
    fn test_preconfigured_founder_key() {
        let status = verify_license(FOUNDER_LIFETIME_KEY).expect("Founder key must be valid");
        assert!(status.is_pro);
        assert_eq!(status.plan, "lifetime");
        assert!(status.can_use_spatial_pan());
    }

    #[test]
    fn test_cache_activation_and_deactivation_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("cacalm-pro-test-{}", uuid::Uuid::now_v7()));
        let cache_file = temp_dir.join("settings").join("pro_cache.json");

        // Before activation: free tier
        let initial_status = get_pro_status_from_path(&cache_file);
        assert!(!initial_status.is_pro);

        // Activate valid lifetime key
        let key = generate_license_key("lifetime", None, "OFFLINE-TEST-USER");
        let activated = activate_license_with_path(&key, &cache_file).expect("Activation must succeed");
        assert!(activated.is_pro);
        assert!(cache_file.exists());

        // Query status: should reflect Pro
        let current_status = get_pro_status_from_path(&cache_file);
        assert!(current_status.is_pro);
        assert_eq!(current_status.plan, "lifetime");
        assert_eq!(current_status.license_key, Some(key));

        // Deactivate
        deactivate_license_at_path(&cache_file).expect("Deactivation must succeed");
        assert!(!cache_file.exists());

        // Query status: reverted back to free
        let post_deactivate = get_pro_status_from_path(&cache_file);
        assert!(!post_deactivate.is_pro);
        assert_eq!(post_deactivate.plan, "free");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
