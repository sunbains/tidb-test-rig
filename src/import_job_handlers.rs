use crate::state_machine::{State, StateContext, StateHandler};
use mysql::prelude::*;
use mysql::*;
use serde::Deserialize;

#[derive(Debug, Clone, FromRow, Deserialize)]
pub struct ImportJob {
    #[serde(rename = "Job_ID")]
    pub job_id: i32,
    #[serde(rename = "Data_Source")]
    pub data_source: String,
    #[serde(rename = "Target_Table")]
    pub target_table: String,
    #[serde(rename = "Table_ID")]
    pub table_id: i32,
    #[serde(rename = "Phase")]
    pub phase: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Source_File_Size")]
    pub source_file_size: String,
    #[serde(rename = "Imported_Rows")]
    pub imported_rows: i64,
    #[serde(rename = "Result_Message")]
    pub result_message: String,
    #[serde(rename = "Create_Time")]
    pub create_time: String,
    #[serde(rename = "Start_Time")]
    pub start_time: String,
    #[serde(rename = "End_Time")]
    pub end_time: Option<String>,
    #[serde(rename = "Created_By")]
    pub created_by: String,
}

/// Handler for checking import jobs
pub struct CheckingImportJobsHandler;

impl StateHandler for CheckingImportJobsHandler {
    fn enter(&self, _context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Checking for active import jobs...");
        Ok(State::CheckingImportJobs)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref mut conn) = context.connection {
            // Execute SHOW IMPORT JOBS
            let query = "SHOW IMPORT JOBS";
            let results: Vec<ImportJob> = conn.exec(query, ())?;
            
            // Extract job IDs where End_Time is NULL
            let mut active_jobs = Vec::new();
            for job in results {
                if job.end_time.is_none() {
                    println!("Found active import job: {}", job.job_id);
                    active_jobs.push(job.job_id.to_string());
                }
            }
            
            context.active_import_jobs = active_jobs;
            
            if context.active_import_jobs.is_empty() {
                println!("✓ No active import jobs found");
                Ok(State::Completed)
            } else {
                println!("✓ Found {} active import job(s)", context.active_import_jobs.len());
                Ok(State::ShowingImportJobDetails)
            }
        } else {
            return Err("No connection available for checking import jobs".into());
        }
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Handler for showing import job details
pub struct ShowingImportJobDetailsHandler;

impl StateHandler for ShowingImportJobDetailsHandler {
    fn enter(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        println!("Showing details for {} active import job(s)...", context.active_import_jobs.len());
        Ok(State::ShowingImportJobDetails)
    }

    fn execute(&self, context: &mut StateContext) -> Result<State, Box<dyn std::error::Error>> {
        if let Some(ref mut conn) = context.connection {
            for job_id in &context.active_import_jobs {
                println!("\n--- Import Job {} ---", job_id);
                
                let query = format!("SHOW IMPORT JOB {}", job_id);
                let results: Vec<(String, String)> = conn.exec(&query, ())?;
                
                for (key, value) in results {
                    println!("  {}: {}", key, value);
                }
            }
            
            println!("\n✓ Import job details displayed");
        } else {
            return Err("No connection available for showing import job details".into());
        }
        
        Ok(State::Completed)
    }

    fn exit(&self, _context: &mut StateContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
} 