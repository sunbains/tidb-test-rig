#![allow(non_snake_case)]

use crate::state_machine::{State, StateContext, StateHandler, StateError};
use mysql::prelude::*;
use mysql::*;
use chrono::{NaiveDateTime, Utc};
use std::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

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
    pub Create_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub Start_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub End_Time: Option<NaiveDateTime>,
    #[allow(non_snake_case)]
    pub Created_By: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobInfo {
    pub job_id: String,
    pub connection_id: String,
    pub phase: String,
    pub status: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Context specific to job monitoring
#[derive(Clone)]
pub struct JobMonitorContext {
    pub active_import_jobs: Vec<String>,
    pub monitor_duration: u64,
}

impl JobMonitorContext {
    pub fn new(monitor_duration: u64) -> Self {
        Self {
            active_import_jobs: Vec::new(),
            monitor_duration,
        }
    }
}

/// Job monitor state machine
pub struct JobMonitor {
    state_machine: crate::state_machine::StateMachine,
}

impl JobMonitor {
    pub fn new(monitor_duration: u64) -> Self {
        let mut state_machine = crate::state_machine::StateMachine::new();
        
        // Register job monitoring handlers
        state_machine.register_handler(State::CheckingImportJobs, Box::new(CheckingImportJobsHandler));
        state_machine.register_handler(State::ShowingImportJobDetails, Box::new(ShowingImportJobDetailsHandler::new(monitor_duration)));
        
        // Initialize context
        state_machine.get_context_mut().set_handler_context(State::CheckingImportJobs, JobMonitorContext::new(monitor_duration));
        
        Self { state_machine }
    }
    
    pub async fn run(&mut self) -> Result<(), StateError> {
        self.state_machine.run().await
    }
    
    pub fn get_context(&self) -> &StateContext {
        self.state_machine.get_context()
    }
    
    pub fn get_context_mut(&mut self) -> &mut StateContext {
        self.state_machine.get_context_mut()
    }
}

/// Handler for checking import jobs
pub struct CheckingImportJobsHandler;

#[async_trait]
impl StateHandler for CheckingImportJobsHandler {
    async fn enter(&self, context: &mut StateContext) -> Result<State, StateError> {
        println!("Checking for active import jobs...");
        // Initialize job monitor context
        context.set_handler_context(State::CheckingImportJobs, JobMonitorContext::new(0));
        Ok(State::CheckingImportJobs)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
        if let Some(ref mut conn) = context.connection {
            // Execute SHOW IMPORT JOBS
            let query = "SHOW IMPORT JOBS";
            let results: Vec<ImportJob> = conn.exec(query, ())?;
            
            // Extract job IDs where End_Time is NULL
            let mut active_jobs = Vec::new();
            for job in results {
                if job.End_Time.is_none() {
                    active_jobs.push(job.Job_ID.to_string());
                }
            }
            
            // Update the job monitor context
            if let Some(job_context) = context.get_handler_context_mut::<JobMonitorContext>(&State::CheckingImportJobs) {
                job_context.active_import_jobs = active_jobs.clone();
            }
            
            // Check if we have active jobs
            if active_jobs.is_empty() {
                println!("✓ No active import jobs found");
                Ok(State::Completed)
            } else {
                println!("✓ Found {} active import job(s)", active_jobs.len());
                Ok(State::ShowingImportJobDetails)
            }
        } else {
            return Err("No connection available for checking import jobs".into());
        }
    }

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        // Move the context to the next state
        context.move_handler_context::<JobMonitorContext>(&State::CheckingImportJobs, State::ShowingImportJobDetails);
        Ok(())
    }
}

/// Handler for showing import job details
pub struct ShowingImportJobDetailsHandler {
    monitor_duration: u64,
}

impl ShowingImportJobDetailsHandler {
    pub fn new(monitor_duration: u64) -> Self {
        Self { monitor_duration }
    }
}

#[async_trait]
impl StateHandler for ShowingImportJobDetailsHandler {
    async fn enter(&self, context: &mut StateContext) -> Result<State, StateError> {
        // Update the monitor duration in the context
        if let Some(job_context) = context.get_handler_context_mut::<JobMonitorContext>(&State::ShowingImportJobDetails) {
            job_context.monitor_duration = self.monitor_duration;
            println!("Monitoring {} active import job(s) for {} seconds...", 
                    job_context.active_import_jobs.len(), job_context.monitor_duration);
        }
        Ok(State::ShowingImportJobDetails)
    }

    async fn execute(&self, context: &mut StateContext) -> Result<State, StateError> {
        // Extract context data first to avoid borrowing conflicts
        let (monitor_duration, active_jobs) = if let Some(job_context) = context.get_handler_context::<JobMonitorContext>(&State::ShowingImportJobDetails) {
            (job_context.monitor_duration, job_context.active_import_jobs.clone())
        } else {
            return Err("Job monitor context not found".into());
        };

        if let Some(ref mut conn) = context.connection {
            let start_time = std::time::Instant::now();
            let duration = Duration::from_secs(monitor_duration);
            
            while start_time.elapsed() < duration {
                println!("\n--- Import Job Status Update ({}s remaining) ---", 
                        (duration - start_time.elapsed()).as_secs());
                
                for job_id in &active_jobs {
                    let query = format!("SHOW IMPORT JOB {job_id}");
                    let results: Vec<ImportJob> = conn.exec(&query, ())?;
                    for job in results {
                        if job.End_Time.is_none() {
                            // Calculate time elapsed using UTC for consistency
                            let now = Utc::now().naive_utc();
                            let start_time = job.Start_Time.unwrap_or(now);
                            let elapsed = now - start_time;
                            let elapsed_h = elapsed.num_hours();
                            let elapsed_m = (elapsed.num_minutes() % 60).abs();
                            let elapsed_s = (elapsed.num_seconds() % 60).abs();
                            println!(
                                "Job_ID: {} | Phase: {} | Start_Time: {} | Source_File_Size: {} | Imported_Rows: {} | Time elapsed: {:02}:{:02}:{:02}",
                                job.Job_ID,
                                job.Phase,
                                job.Start_Time.map(|t| t.to_string()).unwrap_or_else(|| "N/A".to_string()),
                                job.Source_File_Size,
                                job.Imported_Rows,
                                elapsed_h, elapsed_m, elapsed_s
                            );
                        }
                    }
                }
                
                // Sleep for 5 seconds before next update
                sleep(Duration::from_secs(5)).await;
            }
            
            println!("\n✓ Import job monitoring completed after {monitor_duration} seconds");
        } else {
            return Err("No connection available for showing import job details".into());
        }
        Ok(State::Completed)
    }

    async fn exit(&self, context: &mut StateContext) -> Result<(), StateError> {
        // Clean up the context
        context.remove_handler_context(&State::ShowingImportJobDetails);
        Ok(())
    }
} 