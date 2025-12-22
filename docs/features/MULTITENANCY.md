# Multitenancy Feature

## Overview

Ampel's multitenancy feature allows organizations to collaborate on pull request management through teams. Users can create organizations, set up teams within those organizations, and manage team members with role-based access control.

## Architecture

### Database Schema

The multitenancy feature is built on three core entities:

```
organizations
├── id (UUID, primary key)
├── owner_id (UUID, references users)
├── name (String)
├── slug (String, unique identifier)
├── description (Optional<String>)
├── logo_url (Optional<String>)
├── created_at (DateTime)
└── updated_at (DateTime)

teams
├── id (UUID, primary key)
├── organization_id (UUID, references organizations)
├── name (String)
├── slug (String, unique identifier)
├── description (Optional<String>)
├── created_at (DateTime)
└── updated_at (DateTime)

team_members
├── id (UUID, primary key)
├── team_id (UUID, references teams)
├── user_id (UUID, references users)
├── role (String: "admin" | "member" | "viewer")
└── joined_at (DateTime)
```

### Entity Relationships

- **Organization → Owner**: Each organization has one owner (User)
- **Team → Organization**: Teams belong to organizations
- **Team ↔ Users**: Many-to-many relationship through team_members table

## API Endpoints

All team endpoints are authenticated and located under `/api/teams`.

### List Teams

Get all teams the authenticated user is a member of.

**Endpoint:** `GET /api/teams`

**Authentication:** Required

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "organization_id": "uuid",
      "name": "Engineering",
      "slug": "engineering",
      "description": "Engineering team",
      "member_count": 5,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### Create Team

Create a new team within an organization. The creator is automatically added as an admin.

**Endpoint:** `POST /api/teams`

**Authentication:** Required

**Request Body:**

```json
{
  "organization_id": "uuid",
  "name": "Engineering",
  "description": "Engineering team"
}
```

**Response:** `201 Created`

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "organization_id": "uuid",
    "name": "Engineering",
    "slug": "engineering",
    "description": "Engineering team",
    "member_count": 1,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

**Implementation Details:**

- Slug is automatically generated from name (lowercase, spaces replaced with hyphens)
- Creator is added as admin member
- Returns HTTP 201 on success

### Get Team Details

Retrieve details for a specific team.

**Endpoint:** `GET /api/teams/:team_id`

**Authentication:** Required (must be team member)

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "organization_id": "uuid",
    "name": "Engineering",
    "slug": "engineering",
    "description": "Engineering team",
    "member_count": 5,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

**Authorization:**

- Only team members can view team details
- Returns 404 if user is not a member

### Add Team Member

Add a user to the team with a specific role.

**Endpoint:** `POST /api/teams/:team_id/members`

**Authentication:** Required (admin only)

**Request Body:**

```json
{
  "user_id": "uuid",
  "role": "member"
}
```

**Roles:**

- `admin`: Full team management permissions
- `member`: Standard team access
- `viewer`: Read-only access

**Response:** `201 Created`

**Authorization:**

- Only team admins can add members
- Returns 401 if user is not an admin

### Remove Team Member

Remove a user from the team.

**Endpoint:** `DELETE /api/teams/:team_id/members/:user_id`

**Authentication:** Required

**Response:** `204 No Content`

**Authorization:**

- Team admins can remove any member
- Users can remove themselves
- Returns 401 for unauthorized attempts

## Implementation Details

### Handler Functions

Located in `crates/ampel-api/src/handlers/teams.rs`:

```rust
// List teams for current user
pub async fn list_teams(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<TeamResponse>>>, ApiError>

// Create a new team
pub async fn create_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TeamResponse>>), ApiError>

// Get team details
pub async fn get_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<ApiResponse<TeamResponse>>, ApiError>

// Add member to team
pub async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, ApiError>

// Remove member from team
pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError>
```

### Entity Definitions

Located in `crates/ampel-db/src/entities/`:

- `organization.rs`: Organization entity with owner relationship
- `team.rs`: Team entity with organization relationship
- `team_member.rs`: Team membership join table

## Usage Examples

### Creating an Organization and Team

```bash
# Create organization (assumes organization creation endpoint)
curl -X POST http://localhost:8080/api/organizations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corp",
    "description": "Engineering organization"
  }'

# Create team within organization
curl -X POST http://localhost:8080/api/teams \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org-uuid",
    "name": "Engineering",
    "description": "Engineering team"
  }'
```

### Managing Team Members

```bash
# Add member to team
curl -X POST http://localhost:8080/api/teams/team-uuid/members \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user-uuid",
    "role": "member"
  }'

# Remove member from team
curl -X DELETE http://localhost:8080/api/teams/team-uuid/members/user-uuid \
  -H "Authorization: Bearer $TOKEN"
```

### Listing User's Teams

```bash
curl http://localhost:8080/api/teams \
  -H "Authorization: Bearer $TOKEN"
```

## Frontend Integration

While there are no dedicated team management UI components yet, the API endpoints are ready for integration. A typical team management interface would include:

### Recommended UI Components

1. **Team List View**
   - Display all teams user belongs to
   - Show member counts
   - Quick access to team settings

2. **Team Creation Form**
   - Organization selector
   - Team name and description inputs
   - Automatic slug generation preview

3. **Team Members Panel**
   - List of team members with roles
   - Add/remove member actions (admin only)
   - Role management dropdown (admin only)

4. **Team Settings**
   - Edit team name/description
   - Team deletion (owner only)
   - Access control settings

### Example React Integration

```typescript
import { useQuery, useMutation } from '@tanstack/react-query';

// List user's teams
const { data: teams } = useQuery({
  queryKey: ['teams'],
  queryFn: async () => {
    const res = await fetch('/api/teams', {
      headers: { Authorization: `Bearer ${token}` },
    });
    return res.json();
  },
});

// Create team
const createTeamMutation = useMutation({
  mutationFn: async (teamData) => {
    const res = await fetch('/api/teams', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(teamData),
    });
    return res.json();
  },
});
```

## Security & Authorization

### Role-Based Access Control

- **Admin**: Full team management including member management
- **Member**: Standard team access
- **Viewer**: Read-only access

### Verification Flow

1. All endpoints verify JWT authentication
2. Team membership is verified for access
3. Admin role is checked for management operations
4. Users can remove themselves regardless of role

### Data Isolation

- Users only see teams they belong to
- Team data is filtered by membership
- Organization ownership is enforced

## Future Enhancements

### Planned Features

1. **Organization API Endpoints**
   - Create organization
   - List user's organizations
   - Update organization settings
   - Transfer ownership

2. **Enhanced Permissions**
   - Custom roles beyond admin/member/viewer
   - Permission templates
   - Fine-grained access control

3. **Team Features**
   - Team-level PR filters
   - Team dashboards
   - Team notifications
   - Activity feeds

4. **Invitations**
   - Invite users via email
   - Invitation expiry
   - Pending invitations list

5. **Audit Logging**
   - Track team changes
   - Member activity logs
   - Access history

## Database Queries

### Common Queries

```rust
// Find teams for user
team_member::Entity::find()
    .filter(team_member::Column::UserId.eq(user_id))
    .all(&db)
    .await?;

// Get team member count
team_member::Entity::find()
    .filter(team_member::Column::TeamId.eq(team_id))
    .count(&db)
    .await?;

// Verify membership
team_member::Entity::find()
    .filter(team_member::Column::TeamId.eq(team_id))
    .filter(team_member::Column::UserId.eq(user_id))
    .one(&db)
    .await?;

// Check admin access
let membership = team_member::Entity::find()
    .filter(team_member::Column::TeamId.eq(team_id))
    .filter(team_member::Column::UserId.eq(user_id))
    .one(&db)
    .await?;

if membership.role != "admin" {
    return Err(ApiError::unauthorized("Admin access required"));
}
```

## Testing

### Unit Tests

Test team operations in `crates/ampel-api/tests/`:

```rust
#[tokio::test]
async fn test_create_team() {
    let state = create_test_state().await;
    let user = create_test_user(&state.db).await;
    let org = create_test_organization(&state.db, user.id).await;

    let req = CreateTeamRequest {
        organization_id: org.id,
        name: "Engineering".to_string(),
        description: Some("Engineering team".to_string()),
    };

    let response = create_team(
        State(state),
        AuthUser { user_id: user.id },
        Json(req)
    ).await;

    assert!(response.is_ok());
    let (status, json) = response.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json.0.data.unwrap().name, "Engineering");
}
```

### Integration Tests

```bash
# Test team creation
make test-backend

# Run specific team tests
cargo test --package ampel-api --test teams
```

## Troubleshooting

### Common Issues

**Problem:** Cannot create team

- **Solution:** Verify organization_id exists and user has permissions

**Problem:** Cannot add member

- **Solution:** Ensure you have admin role in the team

**Problem:** Team not appearing in list

- **Solution:** Verify team membership exists in team_members table

**Problem:** Slug conflicts

- **Solution:** Team slugs are organization-scoped, ensure unique names within org

## Related Files

- Backend:
  - `crates/ampel-db/src/entities/organization.rs`
  - `crates/ampel-db/src/entities/team.rs`
  - `crates/ampel-db/src/entities/team_member.rs`
  - `crates/ampel-api/src/handlers/teams.rs`
  - `crates/ampel-api/src/routes/mod.rs`

- Future Frontend:
  - `frontend/src/pages/Teams.tsx` (to be created)
  - `frontend/src/components/teams/TeamList.tsx` (to be created)
  - `frontend/src/api/teams.ts` (to be created)
