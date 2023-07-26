use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub system_message: String,
    pub instructions: Vec<String>,
    pub instruction_groups: Option<Vec<InstructionGroupOptions>>,
    pub prompt_formats: Vec<PromptFormatOptions>,
    pub models: Vec<ModelOptions>,
    pub generation_parameters: Vec<TextgenParameters>,
    pub output_folder: String,
    pub api_url: String,
    pub api_timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ModelOptions {
    pub name: String,
    pub format: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PromptFormatOptions {
    pub name: String,
    pub format: String,
    pub stop_sequence: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct InstructionGroupOptions {
    pub name: String,
    pub substitutes: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TextgenParameters {
    pub name: String,
    pub max_length: u16,
    pub temperature: f32,
    pub top_k: u8,
    pub top_p: f32,
    pub typical_p: f32,
    pub rep_pen: f32,
    pub seed: i64,
    pub max_context_length: u16,
}

pub fn get_app_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    // open the configuration file and read it into a string
    let file_data_str = std::fs::read_to_string(filename)?;
    let toml_result: Config = toml::from_str(&file_data_str)?;
    Ok(toml_result)
}
