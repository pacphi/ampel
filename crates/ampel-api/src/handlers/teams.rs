use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::entities::{team, team_member};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTeamRequest {
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: String, // admin, member, viewer
}

/// List teams for the current user
pub async fn list_teams(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<TeamResponse>>>, ApiError> {
    let memberships = team_member::Entity::find()
        .filter(team_member::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await?;

    let team_ids: Vec<Uuid> = memberships.iter().map(|m| m.team_id).collect();

    let teams = team::Entity::find()
        .filter(team::Column::Id.is_in(team_ids))
        .all(&state.db)
        .await?;

    let mut responses = Vec::new();
    for t in teams {
        let member_count = team_member::Entity::find()
            .filter(team_member::Column::TeamId.eq(t.id))
            .count(&state.db)
            .await? as i64;

        responses.push(TeamResponse {
            id: t.id,
            organization_id: t.organization_id,
            name: t.name,
            slug: t.slug,
            description: t.description,
            member_count,
            created_at: t.created_at,
        });
    }

    Ok(Json(ApiResponse::success(responses)))
}

/// Create a new team
pub async fn create_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TeamResponse>>), ApiError> {
    let slug = req.name.to_lowercase().replace(' ', "-");
    let now = Utc::now();

    let team_model = team::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(req.organization_id),
        name: Set(req.name.clone()),
        slug: Set(slug.clone()),
        description: Set(req.description),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let team = team_model.insert(&state.db).await?;

    // Add creator as admin
    let member = team_member::ActiveModel {
        id: Set(Uuid::new_v4()),
        team_id: Set(team.id),
        user_id: Set(auth.user_id),
        role: Set("admin".to_string()),
        joined_at: Set(now),
    };
    member.insert(&state.db).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(TeamResponse {
            id: team.id,
            organization_id: team.organization_id,
            name: team.name,
            slug: team.slug,
            description: team.description,
            member_count: 1,
            created_at: team.created_at,
        })),
    ))
}

/// Get team details
pub async fn get_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<ApiResponse<TeamResponse>>, ApiError> {
    // Verify membership
    team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team_id))
        .filter(team_member::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Team not found"))?;

    let team = team::Entity::find_by_id(team_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Team not found"))?;

    let member_count = team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team_id))
        .count(&state.db)
        .await? as i64;

    Ok(Json(ApiResponse::success(TeamResponse {
        id: team.id,
        organization_id: team.organization_id,
        name: team.name,
        slug: team.slug,
        description: team.description,
        member_count,
        created_at: team.created_at,
    })))
}

/// Add member to team
pub async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, ApiError> {
    // Verify admin access
    let membership = team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team_id))
        .filter(team_member::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Team not found"))?;

    if membership.role != "admin" {
        return Err(ApiError::unauthorized("Admin access required"));
    }

    let member = team_member::ActiveModel {
        id: Set(Uuid::new_v4()),
        team_id: Set(team_id),
        user_id: Set(req.user_id),
        role: Set(req.role),
        joined_at: Set(Utc::now()),
    };
    member.insert(&state.db).await?;

    Ok(StatusCode::CREATED)
}

/// Remove member from team
pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    // Verify admin access
    let membership = team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team_id))
        .filter(team_member::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Team not found"))?;

    if membership.role != "admin" && auth.user_id != user_id {
        return Err(ApiError::unauthorized("Admin access required"));
    }

    team_member::Entity::delete_many()
        .filter(team_member::Column::TeamId.eq(team_id))
        .filter(team_member::Column::UserId.eq(user_id))
        .exec(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
