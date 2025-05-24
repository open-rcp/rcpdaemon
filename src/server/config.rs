use crate::server::error::Result;
use rcpcore::DEFAULT_PORT;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the RCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server address to bind to
    #[serde(default = "default_address")]
    pub address: String,

    /// Server port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// TLS configuration
    #[serde(default)]
    pub tls: TlsConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Session configuration
    #[serde(default)]
    pub session: SessionConfig,

    /// Application configuration
    #[serde(default)]
    pub application: ApplicationConfig,
}

/// Default address to bind to
fn default_address() -> String {
    "0.0.0.0".to_string()
}

/// Default port to listen on
fn default_port() -> u16 {
    DEFAULT_PORT
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    pub enabled: bool,

    /// Path to the certificate file
    pub cert_path: String,

    /// Path to the key file
    pub key_path: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is required
    #[serde(default = "default_auth_required")]
    pub required: bool,

    /// Pre-shared key for authentication
    pub psk: Option<String>,

    /// Allowed client IDs
    #[serde(default)]
    pub allowed_clients: Vec<String>,

    /// Authentication provider type (internal, native, ldap, oauth)
    #[serde(default)]
    pub provider: String,

    /// Whether to fall back to internal authentication if native fails
    #[serde(default)]
    pub fallback_to_internal: bool,

    /// Native authentication configuration
    #[serde(default)]
    pub native: NativeAuthConfig,
}

/// Native authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeAuthConfig {
    /// Whether to allow all OS users
    #[serde(default)]
    pub allow_all_users: bool,

    /// Required OS group for RCP access
    pub require_group: Option<String>,

    /// Whether to map OS groups to RCP permissions
    #[serde(default = "default_true")]
    pub permission_mapping: bool,

    /// OS groups with admin privileges
    #[serde(default = "default_admin_groups")]
    pub admin_groups: Vec<String>,

    /// Custom permission mappings (group -> permissions)
    #[serde(default)]
    pub permission_mappings: std::collections::HashMap<String, Vec<String>>,
}

fn default_true() -> bool {
    true
}

fn default_admin_groups() -> Vec<String> {
    vec![
        "administrators".to_string(),
        "wheel".to_string(),
        "sudo".to_string(),
        "admin".to_string(),
    ]
}

fn default_auth_required() -> bool {
    true
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            required: true,
            psk: None,
            allowed_clients: Vec::new(),
            provider: "internal".to_string(),
            fallback_to_internal: false,
            native: NativeAuthConfig::default(),
        }
    }
}

impl Default for NativeAuthConfig {
    fn default() -> Self {
        Self {
            allow_all_users: false,
            require_group: None,
            permission_mapping: true,
            admin_groups: default_admin_groups(),
            permission_mappings: std::collections::HashMap::new(),
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum number of sessions
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,

    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub timeout: u64,
}

fn default_max_sessions() -> usize {
    100
}

fn default_session_timeout() -> u64 {
    3600
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: default_max_sessions(),
            timeout: default_session_timeout(),
        }
    }
}

/// Application configuration - simplified to avoid proc-macro issues
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// Whether to enable application management
    pub enabled: bool,
    /// Application directory
    pub app_dir: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_dir: "apps".to_string(),
        }
    }
}

impl serde::Serialize for ApplicationConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ApplicationConfig", 2)?;
        state.serialize_field("enabled", &self.enabled)?;
        state.serialize_field("app_dir", &self.app_dir)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ApplicationConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ApplicationConfigVisitor;

        impl<'de> serde::de::Visitor<'de> for ApplicationConfigVisitor {
            type Value = ApplicationConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a struct ApplicationConfig")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ApplicationConfig, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut enabled = None;
                let mut app_dir = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "enabled" => {
                            enabled = Some(map.next_value()?);
                        }
                        "app_dir" => {
                            app_dir = Some(map.next_value()?);
                        }
                        _ => {
                            // Skip unknown fields
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(ApplicationConfig {
                    enabled: enabled.unwrap_or(false),
                    app_dir: app_dir.unwrap_or_else(|| "apps".to_string()),
                })
            }
        }

        deserializer.deserialize_map(ApplicationConfigVisitor)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: default_address(),
            port: default_port(),
            tls: TlsConfig::default(),
            auth: AuthConfig::default(),
            session: SessionConfig::default(),
            application: ApplicationConfig::default(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .add_source(config::Environment::with_prefix("RCP_SERVER"));

        let config = builder
            .build()
            .map_err(crate::server::error::Error::Config)?;

        let server_config: ServerConfig = config
            .try_deserialize()
            .map_err(crate::server::error::Error::Config)?;

        Ok(server_config)
    }

    /// Save configuration to a file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml =
            toml::to_string(self).map_err(|e| crate::server::error::Error::Other(e.to_string()))?;

        std::fs::write(path, toml).map_err(|e| e.into())
    }
}
