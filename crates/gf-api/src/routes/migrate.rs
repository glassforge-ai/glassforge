//! Migration API: POST /migrations, GET /migrations/:id

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/migrations", post(create_migration))
        .route("/migrations/{id}", get(get_migration))
}

#[derive(Deserialize)]
struct CreateMigrationRequest {
    scan_id: String,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Serialize)]
struct MigrationResponse {
    #[serde(flatten)]
    run: gf_db::MigrationRun,
    tasks: Vec<gf_db::DbMigrationTask>,
}

async fn create_migration(
    State(state): State<AppState>,
    Json(body): Json<CreateMigrationRequest>,
) -> Result<Json<MigrationResponse>, axum::response::Response> {
    // Get the scan to find repo_path and artifact.
    let scan = state.scan_repo.get(&body.scan_id).map_err(api_error)?;

    let artifact_json = scan.artifact_json.as_deref().ok_or_else(|| {
        api_error(gf_core::ForgeError::Validation(
            "scan has no artifact — is it still running?".into(),
        ))
    })?;

    // Parse artifact and plan migration.
    let artifact = gf_migrate::parse_artifact_str(artifact_json).map_err(|e| {
        api_error(gf_core::ForgeError::Validation(format!(
            "failed to parse scan artifact: {e}"
        )))
    })?;

    let plan = gf_migrate::plan_migration(&artifact);

    // Create DB records.
    let run = state
        .migration_run_repo
        .create(&body.scan_id, &scan.repo_path, plan.tasks.len() as i64)
        .map_err(api_error)?;

    let mut db_tasks = Vec::new();
    for task in &plan.tasks {
        let priority = format!("{}", task.priority);
        let db_task = state
            .migration_run_repo
            .create_task(
                &run.id,
                &task.file_path,
                &priority,
                task.persona_id.as_deref(),
                task.findings.len() as i64,
            )
            .map_err(api_error)?;
        db_tasks.push(db_task);
    }

    if body.dry_run {
        let run = state
            .migration_run_repo
            .update_status(&run.id, "completed")
            .map_err(api_error)?;
        return Ok(Json(MigrationResponse {
            run,
            tasks: db_tasks,
        }));
    }

    // Mark as running. Actual agent execution happens via the CLI for now.
    let _ = state
        .migration_run_repo
        .update_status(&run.id, "running")
        .map_err(api_error)?;

    tracing::info!(
        run_id = run.id,
        repo_path = scan.repo_path,
        "migration planned — run `glassforge migrate` CLI to execute agents"
    );

    Ok(Json(MigrationResponse {
        run,
        tasks: db_tasks,
    }))
}

async fn get_migration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MigrationResponse>, axum::response::Response> {
    let run = state.migration_run_repo.get(&id).map_err(api_error)?;
    let tasks = state.migration_run_repo.list_tasks(&id).map_err(api_error)?;
    Ok(Json(MigrationResponse { run, tasks }))
}
