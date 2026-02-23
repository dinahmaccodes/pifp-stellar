//! Axum REST API handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::db;
use crate::events::EventRecord;

#[derive(Clone)]
pub struct ApiState {
    pub pool: SqlitePool,
}

// ─────────────────────────────────────────────────────────
// Response shapes
// ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct EventsResponse {
    pub project_id: String,
    pub count: usize,
    pub events: Vec<EventRecord>,
}

#[derive(Serialize)]
pub struct AllEventsResponse {
    pub count: usize,
    pub events: Vec<EventRecord>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ─────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────

/// `GET /health`
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// `GET /projects/:id/events`
///
/// Returns all indexed events for the given project identifier.
pub async fn get_project_events(
    State(state): State<Arc<ApiState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    match db::get_events_for_project(&state.pool, &project_id).await {
        Ok(events) => {
            let count = events.len();
            (
                StatusCode::OK,
                Json(serde_json::json!(EventsResponse {
                    project_id,
                    count,
                    events,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse {
                error: e.to_string()
            })),
        )
            .into_response(),
    }
}

/// `GET /events`
///
/// Returns all indexed events across all projects.
pub async fn get_all_events(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    match db::get_all_events(&state.pool).await {
        Ok(events) => {
            let count = events.len();
            (
                StatusCode::OK,
                Json(serde_json::json!(AllEventsResponse { count, events })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(ErrorResponse {
                error: e.to_string()
            })),
        )
            .into_response(),
    }
}
