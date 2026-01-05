use compact_str::CompactString;
use uuid::Uuid;

/// Access JWT token string
#[derive(Clone, PartialEq, Eq)]
pub struct AccessToken(String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AccessToken {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn new(inner: impl AsRef<str>) -> Self {
        Self(inner.as_ref().to_owned())
    }
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccessToken([REDACTED])")
    }
}

/// Refresh JWT token string
#[derive(Clone, PartialEq, Eq)]
pub struct RefreshToken(String);

impl AsRef<str> for RefreshToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl RefreshToken {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn new(inner: impl AsRef<str>) -> Self {
        Self(inner.as_ref().to_owned())
    }
}

impl std::fmt::Debug for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RefreshToken([REDACTED])")
    }
}

/// Claims stored in generated JWT tokens
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenClaims {
    /// User ID
    pub sub: Uuid,
    /// Session ID
    pub sid: Uuid,
    pub exp: usize,
    pub iss: CompactString,
    pub aud: CompactString,
}
