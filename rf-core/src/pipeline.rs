use crate::{
    config::Paths,
    error::{Result, RiceForgeError},
};
use serde::Deserialize;
use std::{fs, process::Command};

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    #[serde(default)]
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub run: String,
    #[serde(default)]
    pub when: PipelineWhen,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PipelineWhen {
    Install,
    Remove,
    #[default]
    Always,
}

pub struct PipelineManager;

impl PipelineManager {
    pub fn load(rice_id: &str) -> Result<Option<Pipeline>> {
        let path = Paths::rices_dir().join(rice_id).join("pipeline.toml");
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let pipeline: Pipeline = toml::from_str(&content)?;
        Ok(Some(pipeline))
    }

    pub fn run_steps(pipeline: &Pipeline, phase: &PipelineWhen, rice_id: &str) -> Result<()> {
        let work_dir = Paths::rices_dir().join(rice_id);

        for step in &pipeline.steps {
            if step.when != PipelineWhen::Always && &step.when != phase {
                continue;
            }

            let status = Command::new("sh")
                .arg("-c")
                .arg(&step.run)
                .current_dir(&work_dir)
                .status()
                .map_err(|e| {
                    RiceForgeError::Pipeline(format!("step '{}' failed to launch: {e}", step.name))
                })?;

            if !status.success() {
                return Err(RiceForgeError::Pipeline(format!(
                    "step '{}' exited with {status}",
                    step.name
                )));
            }
        }

        Ok(())
    }
}
