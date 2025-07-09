#![allow(non_snake_case)]

use crate::state_machine::{State, StateContext, StateHandler};
use mysql::prelude::*;
use mysql::*;
#[derive(Debug, Clone, FromRow)]
pub struct ImportJob {
    #[allow(non_snake_case)]
    pub Job_ID: i32,
    #[allow(non_snake_case)]
    pub Data_Source: String,
    #[allow(non_snake_case)]
    pub Target_Table: String,
    #[allow(non_snake_case)]
    pub Table_ID: i32,
    #[allow(non_snake_case)]
    pub Phase: String,
    #[allow(non_snake_case)]
    pub Status: String,
    #[allow(non_snake_case)]
    pub Source_File_Size: String,
    #[allow(non_snake_case)]
    pub Imported_Rows: i64,
    #[allow(non_snake_case)]
    pub Result_Message: String,
    #[allow(non_snake_case)]
    pub Create_Time: mysql::Value,
    #[allow(non_snake_case)]
    pub Start_Time: mysql::Value,
    #[allow(non_snake_case)]
    pub End_Time: mysql::Value,
    #[allow(non_snake_case)]
    pub Created_By: String,
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
                if matches!(job.End_Time, mysql::Value::NULL) {
                    active_jobs.push(job.Job_ID.to_string());
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