mod config;
use config::TextgenParameters;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::{path::{Path, PathBuf}, io::Write, collections::HashMap};


const CONFIG_FILENAME: &str = "config.toml";
const CONFIG_SYSTEM_PROMPT_TAG: &str = "{SYSTEM}";
const CONFIG_INSTRUCTION_PROMPT_TAG: &str = "{INSTRUCTION}";
const RAW_FOLDER_NAME: &str = "raw";
const REPORT_FILENAME: &str = "summary.md";


// executes the job passed to it and returns true if execution
// of further jobs should be continue.
fn run_job(config: &config::Config, job: &mut TextgenJob) -> bool {
    // adjust the timeout of the API calls -- configurable
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(config.api_timeout))
        .build().expect("Failed to create the blocking reqwest client.");
    
    // get the current model from the server
    let model_query_url = format!("{}{}", config.api_url, "/api/v1/model");
    log::debug!("Attempting to query the endpoint to determine the active model: {}", &model_query_url);
    let active_model_json = client.get(&model_query_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send().expect("API call failed for querying the loaded model.")
        .text().expect("API call didn't return text for querying the loaded model.");
    let active_model_v: serde_json::Value = serde_json::from_str(&active_model_json)
        .expect("Unable to deserialize JSON for querying the loaded model.");
    let active_model = active_model_v["result"].as_str().expect("Unable to deserialize JSON for querying the loaded model.");
    
    // if this model is different than the job's requested model then load the job's model
    if active_model.eq_ignore_ascii_case(&job.model.name.as_str()) == false {
        log::info!("Attempting to change active model to {}", &job.model.name);
        let req_body = format!("{{\"action\": \"load\", \"model_name\": \"{0}\"}}", &job.model.name);
        let loading_resp = client.post(&model_query_url).body(req_body)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .send().expect("API call failed for loading a different model.");
        if loading_resp.status() != reqwest::StatusCode::OK {
            log::error!("Failed to change the model to {0} at endpoint {1}. Status: {2}", 
                &job.model.name, &model_query_url, loading_resp.status());
            return false;
        }
        // the response bytes has to get called in order for it to wait while the server switches models.
        let loading_json = loading_resp.bytes();
        log::trace!("JSON returned from model load request: {:?}", loading_json);
        log::info!("Model change request was executed.");
    }

    // setup the prompt based on the information in the job
    let prompt = job.prompt_format.format.clone()
        .replace(CONFIG_SYSTEM_PROMPT_TAG, &job.system_message)
        .replace(CONFIG_INSTRUCTION_PROMPT_TAG, &job.instruction);
  
    // make the request data structures
    let textgen_url = format!("{}{}", config.api_url, "/api/v1/generate");
    let textgen_request = TextgenRemoteRequest {
        prompt,
        max_length: job.parameters.max_length,
        temperature: job.parameters.temperature,
        top_k: job.parameters.top_k,
        top_p: job.parameters.top_p,
        typical_p: job.parameters.typical_p,
        rep_pen: job.parameters.rep_pen,
        seed: job.parameters.seed,
        max_context_length: job.parameters.max_context_length,
        stop_sequence: job.prompt_format.stop_sequence.clone(),
    };
    let textgen_request_json = serde_json::to_string(&textgen_request)
        .expect("Failed to serialize the API parameters for the text generation request.");

    // make the text generation request and pull out the generated string
    let textgen_resp = client.post(&textgen_url).body(textgen_request_json)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .send().expect("API call failed for generating text from a prompt");
    if textgen_resp.status() != reqwest::StatusCode::OK {
        log::error!("Failed to generate text for the given prompt. Status: {}", textgen_resp.status());
        return false;
    }
    let textgen_resp_text = textgen_resp.text()
        .expect("Failed to get the JSON from the text generation response body.");
    let textgen_resp: TextgenResponseBody = serde_json::from_str(&textgen_resp_text)
        .expect("Failed to deserialize the JSON from the text generation response body.");
    if textgen_resp.results.is_empty() {
        log::error!("Failed to generate text for the given prompt. Empty result was returned.");
        return false;
    }

    // update the job with the generated text
    job.generated_text_output = Some(textgen_resp.results[0].text.clone());

    return true;
}

// Used to deserialize the text generation response body JSON into a strongly typed object.
#[derive(Deserialize, Debug, Clone)]
pub struct TextgenResponseBody {
    results: Vec<TextgenResponseBodyResult>
}

// Used to deserialize the text generation response body JSON into a strongly typed object.
#[derive(Deserialize, Debug, Clone)]
pub struct TextgenResponseBodyResult {
    text: String
}

// Used to serialize into JSON from a strongly typed object for API calls.
#[derive(Serialize, Debug, Clone)]
pub struct TextgenRemoteRequest {
    pub prompt: String,
    pub max_length: u16,
    pub temperature: f32,
    pub top_k: u8,
    pub top_p: f32,
    pub typical_p: f32,
    pub rep_pen: f32,
    pub seed: i64,
    pub max_context_length: u16,
    pub stop_sequence: Option<Vec<String>>,
}

// Encapsulates all the data needed to generate text for a combination of parameters.
// This structure also gets serialized and deserialized to the filesystem as a record
// of the source material for the report.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
struct TextgenJob {
    system_message: String,
    instruction: String,
    model: config::ModelOptions,
    prompt_format: config::PromptFormatOptions,
    parameters: config::TextgenParameters,
    generated_text_output: Option<String>,
}


// utility function that pulls the matching PromptFormatOptions from the configuration
// based on the name used in the ModelOptions.
fn get_prompt_format(config: &config::Config, model: &config::ModelOptions) -> Option<config::PromptFormatOptions> {
    for pf in &config.prompt_formats {
        if pf.name.eq_ignore_ascii_case(model.format.as_str()) {
            return Some(pf.clone());
        }
    }
    None
}

// loops through the configuration and builds a job for each combination of
// instruction * model * generation_parameters
fn build_jobs(config: &config::Config) -> Vec<TextgenJob> {
    let mut jobs: Vec<TextgenJob> = Vec::new();

    // loop through all the models
    for model in &config.models {
        let pf_opt = get_prompt_format(config, model);
        if let Some(pf) = pf_opt {
            // loop through all the instructions
            for inst in &config.instructions {
                // loop through all of the parameter sets
                for params in &config.generation_parameters {
                    let new_job = TextgenJob{
                        system_message: config.system_message.clone(),
                        instruction: inst.clone(),
                        model: model.clone(),
                        prompt_format: pf.clone(),
                        parameters: params.clone(),
                        generated_text_output: None,
                        };
                    jobs.push(new_job);
                }
            }
        } else {
            log::error!("Could not find a configured 'prompt_formats' with a name matching the model's delcared format: {}.", model.format);
            log::error!("Skipping tasks for model '{}' due to this error.", model.name);
            continue;
        }
    }

    return jobs
}

// for a given directory path, search out non-recursively to find all .json files
// and attempt to deserialize them. errors will not propogate from here and will
// only be logged out.
fn deserialize_all_job_files_for_dir(dir_path: &Path) -> Vec<TextgenJob> {
    let mut matched_json_files: Vec<PathBuf> = Vec::new();
    for entry in dir_path.read_dir().unwrap() {
        if let Ok(entry) = entry {
            let file_type = entry.file_type();
            if let Ok(file_type) = file_type {
                if file_type.is_file() {
                    if let Some(file_ext) = entry.path().extension() {
                        if file_ext.eq_ignore_ascii_case("json") {
                            matched_json_files.push(entry.path())
                        }
                    }
                }
            }
        }
    }

    // attempt to deserialize all matching files as jobs
    let mut thawed_jobs: Vec<TextgenJob> = Vec::new();
    for json_file_path in matched_json_files {
        match std::fs::read_to_string(&json_file_path) {
            Err(err) => log::error!("Unable to read a JSON file ({}) for the report: {}", &json_file_path.to_string_lossy(), err),
            Ok(file_data_str) => {
                match serde_json::from_str::<TextgenJob>(&file_data_str) {
                    Err(err) => log::error!("Unable to deserialize JSON file ({}) for the report: {}", &json_file_path.to_string_lossy(), err),
                    Ok(dethawed_job) => thawed_jobs.push(dethawed_job)
                }
            }
        }
        log::trace!("Deserialized job file from JSON: {}", &json_file_path.to_string_lossy())
    }

    thawed_jobs
}

// writes a markdown file in the output folder that contains the prompts and outputs
// for all of the text generation jobs.
fn generate_report(jobs: Vec<TextgenJob>, report_dir_path: &Path) {
    // create the report file to write to
    let report_filepath = report_dir_path.join(REPORT_FILENAME);
    let mut report_file = match std::fs::File::create(&report_filepath) {
        Err(err) => {
            log::error!("Unable to create the report file ({}): {}", report_filepath.to_string_lossy(), err);
            return;
        }
        Ok(f) => f
    };

    // in this process we're going to keep track of the parameters used by just matching
    // on the parameter set name. we'll do this by keeping encountered parameter sets in a hashmap.
    let mut gen_parameters: HashMap<String, TextgenParameters> = HashMap::new();

    // write the preamble for the file
    let preamble = "# Kontour Report\nThis is a summary document containing all of the generated text organized by instructions.\n\n";
    report_file.write_all(preamble.as_bytes()).expect("Failed to write the preample of the report.");


    // sort the vector by instruction then model name then parameters name so that all 
    // of the generations for a given instruction using the same model are grouped together.
    // makes the report look nicer.
    let mut sorted_jobs = jobs;
    sorted_jobs.sort_by(|a, b| {
        if a.instruction == b.instruction {
            if a.model.name == b.model.name {
                a.parameters.name.cmp(&b.parameters.name)
            } else {
                a.model.name.cmp(&b.model.name)
            }
        } else {
            a.instruction.cmp(&b.instruction)
        }
    });

    // the vector should be sorted, so now we iterate over all of them and keep track
    // of when the instruction changes so we can write a new header.
    let mut current_instruction = String::new();
    for matched_job in sorted_jobs {
        if current_instruction != matched_job.instruction {
            // instruction change detected so write out the new instruction header
            current_instruction = matched_job.instruction;
            let instr_header_text = format!("\n## Instruction Generation Set\n\nThese are the results for the following prompt:\n\n```\n{}\n```\n\n", current_instruction);
            report_file.write_all(instr_header_text.as_bytes()).expect("Failed to write the instruction header of the report.");
        }

        let output_text = matched_job.generated_text_output.clone().unwrap_or("<Did not generate a result.>".to_string());
        report_file.write_all(format!("### {} (parameters: {})\n\n{}\n\n",
            matched_job.model.name,
            matched_job.parameters.name,
            output_text).as_bytes()).expect("Failed to write generated output to instruction section of report.");

        // keep track of the used parameters if it isn't already tracked
        if !gen_parameters.contains_key(matched_job.parameters.name.as_str()) {
            log::trace!("Tracking generation parameter set '{}'", matched_job.parameters.name.as_str());
            gen_parameters.insert(matched_job.parameters.name.clone(), matched_job.parameters.clone());
        }
    }

    
    // write out the appendix with the generation parameters used
    let appendix_text = format!("\n\n## Appendix\n\n");
    report_file.write_all(appendix_text.as_bytes()).expect("Failed to write the appendix header of the report.");

    for (param_set_name, param_set) in gen_parameters {
        report_file.write_all(format!("### Parameter set: ({})\n\n{:?}\n\n",param_set_name, param_set).as_bytes())
            .expect("Failed to write generated output to instruction section of report.");
    }
}


fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Info).env().with_colors(true).init().unwrap();

    // parse the command-line arguments
    let cmd_arg_matches = clap::Command::new("kontour")
        .about("Kontour: a configurable large language model sampler.")
        .arg(clap::Arg::new("config-file")
            .short('f')
            .long("config-file")
            .action(clap::ArgAction::Set)
            .value_name("FILE")
            .help("Use the specified configuration file instead of the default config.toml file."))
        .arg(clap::Arg::new("regenerate-report")
            .long("regenerate-report")
            .value_name("json directory path")
            .action(clap::ArgAction::Append)
            .help("Only generate a report for the generated JSON files in the specified directory."))
        .get_matches();

    // load up the configuration file
    let cfg_filename = match cmd_arg_matches.get_one::<String>("config-file") {
        Some(arg_config_file) => arg_config_file,
        None => CONFIG_FILENAME
    };
    let app_config = match config::get_app_config(cfg_filename) {
        Ok(c) => c,
        Err(err) => {
            log::error!("Couldn't load the configuration file: {err}");
            std::process::exit(1);
        }
    };
    log::info!("Config file loaded: {0} instruction(s) over {1} model(s) with {2} parameter configuration set(s)",
        app_config.instructions.len(),
        app_config.models.len(),
        app_config.generation_parameters.len());

    // are we to only generate the report?
    if let Some(report_dir) = cmd_arg_matches.get_one::<String>("regenerate-report") {
        log::info!("Generating the report for files in the directory: {}", report_dir);

        // find all valid directory entries that are a file with a .json extension
        let mut report_dir_path = Path::new(report_dir);
        let dethawed_jobs = deserialize_all_job_files_for_dir(&report_dir_path);
        log::info!("Found {} job(s) in the directory supplied.", dethawed_jobs.len());

        // now we put the report into the parent directory, if there is one.
        if let Some(p) = report_dir_path.parent() {
            report_dir_path = p;
        }

        // generate the report and quit
        generate_report(dethawed_jobs, report_dir_path);
        std::process::exit(0);
    }

    // build up the job list based on our configuration
    let mut jobs = build_jobs(&app_config);
    log::info!("Built a job list with {} task(s).", jobs.len());


    // make sure the output folders exist
    let now = chrono::Local::now();
    let output_dir_name = now.format("%Y%m%d%H%M%S").to_string();
    let raw_folder_path = Path::new(&app_config.output_folder).join(output_dir_name).join(RAW_FOLDER_NAME);
    std::fs::create_dir_all(&raw_folder_path).expect("Failed to create the output folder structure in the filesystem.");


    // execute the jobs
    let mut job_index = 0;
    for job in jobs.iter_mut() {
        job_index += 1;
        
        log::info!("Starting execution of job #{}", job_index);
        run_job(&app_config, job);
        
        // do the file name calculations
        let run_filename = format!("{:08}.json", job_index);
        let full_output_filepath = raw_folder_path.join(&run_filename);
        let job_raw_json = serde_json::to_string_pretty(job)
            .expect("Failed to serialize the Job structure to write out as a raw JSON file.");

        {
            // serialize the result out to the raw file
            let mut raw_file = std::fs::File::create(&full_output_filepath)
                .expect(format!("Failed to create the output file: {:?}", &full_output_filepath).as_str());
            raw_file.write_all(job_raw_json.as_bytes()).expect("Failed to write all of the raw JSON to the job output file.");
            raw_file.flush().expect("Failed to flush the output for the raw JSON job file.");
        }
        
        log::debug!("Finished execution of job #{}", job_index);
    }
    log::info!("Finished running the jobs.");

    // generate the report and quit
    let report_dir_path = raw_folder_path.parent().unwrap();
    log::info!("Generating the report for files in the directory: {}", &report_dir_path.to_string_lossy());
    generate_report(jobs, report_dir_path);

    log::info!("Report has been generated.")
}
