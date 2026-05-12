use std::{fs, path::PathBuf, sync::RwLock, time::SystemTime};

use serde::Deserialize;

use crate::RepositoryError;

#[derive(Clone, Debug)]
pub struct PolicyInput {
    pub role: String,
    pub action: String,
    pub resource_owner: String,
    pub subject_id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Rule {
    role: Option<String>,
    action: String,
    owner_only: bool,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct PolicyFile {
    rules: Vec<Rule>,
}

#[derive(Debug)]
struct PolicyState {
    policy: PolicyFile,
    mtime: Option<SystemTime>,
}

#[derive(Debug)]
pub struct DynamicPolicyEngine {
    path: PathBuf,
    state: RwLock<PolicyState>,
}

impl DynamicPolicyEngine {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, RepositoryError> {
        let path = path.into();
        let (policy, mtime) = load_policy(&path)?;
        Ok(Self {
            path,
            state: RwLock::new(PolicyState { policy, mtime }),
        })
    }

    pub fn evaluate(&self, input: &PolicyInput) -> Result<bool, RepositoryError> {
        self.reload_if_changed()?;
        let state = self
            .state
            .read()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        for rule in &state.policy.rules {
            if rule.action != "*" && rule.action != input.action {
                continue;
            }
            if let Some(role) = &rule.role
                && role != &input.role
            {
                continue;
            }
            if rule.owner_only && input.subject_id != input.resource_owner {
                continue;
            }
            return Ok(true);
        }

        Ok(false)
    }

    fn reload_if_changed(&self) -> Result<(), RepositoryError> {
        let metadata = fs::metadata(&self.path)
            .map_err(|e| RepositoryError::Internal(format!("policy metadata: {e}")))?;
        let current_mtime = metadata.modified().ok();

        let should_reload = {
            let state = self
                .state
                .read()
                .map_err(|e| RepositoryError::Internal(e.to_string()))?;
            state.mtime != current_mtime
        };

        if should_reload {
            let (policy, mtime) = load_policy(&self.path)?;
            let mut state = self
                .state
                .write()
                .map_err(|e| RepositoryError::Internal(e.to_string()))?;
            state.policy = policy;
            state.mtime = mtime;
        }
        Ok(())
    }
}

fn load_policy(path: &PathBuf) -> Result<(PolicyFile, Option<SystemTime>), RepositoryError> {
    let raw = fs::read_to_string(path)
        .map_err(|e| RepositoryError::Internal(format!("policy read: {e}")))?;
    let policy: PolicyFile = serde_json::from_str(&raw)
        .map_err(|e| RepositoryError::Internal(format!("policy parse: {e}")))?;
    let mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());
    Ok((policy, mtime))
}
