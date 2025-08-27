#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RhexUrl {
    pub scheme: String,
    pub scope: String,
    pub hash_alias: String,
    pub version: Option<String>,
    pub field_name: Option<String>,
}

impl RhexUrl {
    pub fn new(
        scheme: &str,
        scope: &str,
        hash_alias: &str,
        version: Option<&str>,
        field_name: Option<&str>,
    ) -> Self {
        Self {
            scheme: scheme.to_string(),
            scope: scope.to_string(),
            hash_alias: hash_alias.to_string(),
            version: version.map(|s| s.to_string()),
            field_name: field_name.map(|s| s.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        let mut out = format!("{}://{}/", self.scheme, self.scope);
        out.push_str(&self.hash_alias);

        if let Some(ver) = &self.version {
            out.push('@');
            out.push_str(ver);
        }

        if let Some(field) = &self.field_name {
            out.push('#');
            out.push_str(field);
        }

        out
    }

    pub fn from_string(input: &str) -> anyhow::Result<Self> {
        let (scheme, rest) = input
            .split_once("://")
            .ok_or_else(|| anyhow::anyhow!("missing scheme"))?;

        let (scope, remainder) = rest
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("missing scope/hash"))?;

        let (hash_and_ver, field_name) = match remainder.split_once('#') {
            Some((hv, f)) => (hv, Some(f)),
            None => (remainder, None),
        };

        let (hash_alias, version) = match hash_and_ver.split_once('@') {
            Some((h, v)) => (h, Some(v)),
            None => (hash_and_ver, None),
        };

        Ok(Self {
            scheme: scheme.to_string(),
            scope: scope.to_string(),
            hash_alias: hash_alias.to_string(),
            version: version.map(|s| s.to_string()),
            field_name: field_name.map(|s| s.to_string()),
        })
    }
}
