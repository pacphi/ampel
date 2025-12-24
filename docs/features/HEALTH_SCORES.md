# Repository Health Scores

## Overview

Health Scores provide automated quality metrics for each repository tracked in Ampel. The system calculates a 0-100 score based on PR merge performance, review times, stale PR counts, and throughput. Health scores help teams identify problematic repositories and track improvement over time.

## Purpose

Health scores answer key questions:

- Which repositories have slow PR merge cycles?
- Where are PRs getting stalled?
- Which teams need process improvements?
- How is our overall PR health trending?

## Architecture

### Database Schema

```
health_scores
├── id (UUID, primary key)
├── repository_id (UUID, references repositories)
├── score (i32, 0-100)
├── avg_time_to_merge (Optional<i32>, seconds)
├── avg_review_time (Optional<i32>, seconds)
├── stale_pr_count (Optional<i32>)
├── failed_check_rate (Optional<f32>, not currently used)
├── pr_throughput (Optional<i32>, PRs merged in period)
└── calculated_at (DateTime)
```

### Related Entities

```
pr_metrics
├── id (UUID, primary key)
├── repository_id (UUID)
├── pull_request_id (UUID)
├── time_to_merge (Optional<i32>, seconds)
├── time_to_first_review (Optional<i32>, seconds)
├── merged_at (DateTime)
└── ... (other metrics)
```

## Calculation Algorithm

Located in `crates/ampel-worker/src/jobs/health_score.rs`:

### Scoring Formula

The health score starts at 100 and applies penalties based on negative indicators:

```rust
fn calculate_score(
    &self,
    avg_time_to_merge: Option<i32>,
    avg_review_time: Option<i32>,
    stale_prs: i32,
    throughput: i32,
) -> i32 {
    let mut score = 100;

    // 1. Merge time penalty (max -30 points)
    if let Some(merge_time) = avg_time_to_merge {
        let hours = merge_time / 3600;
        if hours > 72 {
            score -= 30;  // > 3 days: severe penalty
        } else if hours > 48 {
            score -= 20;  // > 2 days: moderate penalty
        } else if hours > 24 {
            score -= 10;  // > 1 day: minor penalty
        }
    }

    // 2. Review time penalty (max -20 points)
    if let Some(review_time) = avg_review_time {
        let hours = review_time / 3600;
        if hours > 24 {
            score -= 20;  // > 1 day: major penalty
        } else if hours > 8 {
            score -= 10;  // > 8 hours: moderate penalty
        } else if hours > 4 {
            score -= 5;   // > 4 hours: minor penalty
        }
    }

    // 3. Stale PR penalty (max -25 points)
    if stale_prs > 10 {
        score -= 25;  // > 10 stale PRs: severe
    } else if stale_prs > 5 {
        score -= 15;  // 6-10 stale PRs: moderate
    } else if stale_prs > 0 {
        score -= stale_prs * 2;  // Linear penalty
    }

    // 4. Throughput bonus (max +10 points)
    if throughput >= 10 {
        score += 10;  // High throughput: bonus
    } else if throughput >= 5 {
        score += 5;   // Good throughput: small bonus
    }

    score.clamp(0, 100)  // Final score between 0-100
}
```

### Metric Definitions

**Average Time to Merge (last 30 days)**

- Time from PR creation to merge
- Measured in seconds, converted to hours for scoring
- Only includes merged PRs
- Formula: `sum(time_to_merge) / count(merged_prs)`

**Average Review Time (last 30 days)**

- Time from PR creation to first review
- Measured in seconds, converted to hours for scoring
- Only includes PRs with reviews
- Formula: `sum(time_to_first_review) / count(reviewed_prs)`

**Stale PR Count**

- Number of open PRs older than 7 days
- Indicates PRs that are stuck or abandoned
- Query: `COUNT(*) WHERE state='open' AND created_at < 7_days_ago`

**PR Throughput (last 7 days)**

- Number of PRs merged in the last week
- Indicates team velocity
- Query: `COUNT(*) WHERE merged_at >= 7_days_ago`

**Failed Check Rate (planned)**

- Percentage of PRs with failed CI checks
- Not currently implemented
- Would penalize repositories with poor test quality

## Scoring Thresholds

### Score Ranges

- **90-100**: Excellent
  - Fast merge times (<24h)
  - Quick reviews (<4h)
  - No stale PRs
  - High throughput

- **70-89**: Good
  - Reasonable merge times (24-48h)
  - Timely reviews (4-8h)
  - Few stale PRs (1-5)
  - Moderate throughput

- **50-69**: Needs Improvement
  - Slow merge times (48-72h)
  - Delayed reviews (8-24h)
  - Some stale PRs (5-10)
  - Low throughput

- **0-49**: Critical
  - Very slow merge times (>72h)
  - Very delayed reviews (>24h)
  - Many stale PRs (>10)
  - Poor throughput

## Background Job Implementation

### Job Execution

Health scores are calculated by a background worker job:

```rust
pub struct HealthScoreJob;

impl HealthScoreJob {
    pub async fn execute(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        let repos = repository::Entity::find().all(db).await?;

        tracing::info!("Calculating health scores for {} repositories", repos.len());

        for repo in repos {
            if let Err(e) = self.calculate_repo_health(db, &repo).await {
                tracing::error!(
                    "Failed to calculate health for repo {}: {}",
                    repo.full_name,
                    e
                );
            }
        }

        Ok(())
    }

    async fn calculate_repo_health(
        &self,
        db: &DatabaseConnection,
        repo: &repository::Model,
    ) -> anyhow::Result<()> {
        // Calculation logic...
    }
}
```

### Scheduling

Configured in the worker process (likely using Apalis scheduler):

```rust
// Example scheduler configuration (implementation may vary)
scheduler
    .schedule(
        Schedule::from_str("0 0 * * *").unwrap(), // Daily at midnight
        HealthScoreJob,
    )
    .await?;
```

### Data Collection

For each repository:

1. **Fetch 30-day metrics**

   ```rust
   let metrics = pr_metrics::Entity::find()
       .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
       .filter(pr_metrics::Column::MergedAt.gte(thirty_days_ago))
       .all(db)
       .await?;
   ```

2. **Calculate averages**

   ```rust
   let avg_time_to_merge = if !metrics.is_empty() {
       let total: i64 = metrics
           .iter()
           .filter_map(|m| m.time_to_merge)
           .map(|t| t as i64)
           .sum();
       let count = metrics.iter().filter(|m| m.time_to_merge.is_some()).count();
       Some((total / count as i64) as i32)
   } else {
       None
   };
   ```

3. **Count stale PRs**

   ```rust
   let stale_prs = pull_request::Entity::find()
       .filter(pull_request::Column::RepositoryId.eq(repo.id))
       .filter(pull_request::Column::State.eq("open"))
       .filter(pull_request::Column::CreatedAt.lt(seven_days_ago))
       .count(db)
       .await? as i32;
   ```

4. **Calculate throughput**

   ```rust
   let recent_metrics = pr_metrics::Entity::find()
       .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
       .filter(pr_metrics::Column::MergedAt.gte(seven_days_ago))
       .count(db)
       .await? as i32;
   ```

5. **Compute score and save**

   ```rust
   let score = self.calculate_score(
       avg_time_to_merge,
       avg_review_time,
       stale_prs,
       recent_metrics,
   );

   let health = health_score::ActiveModel {
       id: Set(Uuid::new_v4()),
       repository_id: Set(repo.id),
       score: Set(score),
       avg_time_to_merge: Set(avg_time_to_merge),
       avg_review_time: Set(avg_review_time),
       stale_pr_count: Set(Some(stale_prs)),
       pr_throughput: Set(Some(recent_metrics)),
       calculated_at: Set(now),
       // ...
   };

   health.insert(db).await?;
   ```

## API Endpoints

### Get Repository Health

**Endpoint:** `GET /api/repositories/:repo_id/health`

**Authentication:** Required

**Response:**

```json
{
  "success": true,
  "data": {
    "score": 85,
    "avgTimeToMerge": 28800,
    "avgReviewTime": 7200,
    "stalePrCount": 2,
    "prThroughput": 8,
    "calculatedAt": "2024-01-15T00:00:00Z"
  }
}
```

### Get Health Overview

**Endpoint:** `GET /api/analytics/health`

**Authentication:** Required

**Response:**

```json
{
  "success": true,
  "data": {
    "repositories": [
      {
        "id": "uuid",
        "name": "owner/repo1",
        "score": 92,
        "trend": "up"
      },
      {
        "id": "uuid",
        "name": "owner/repo2",
        "score": 68,
        "trend": "down"
      }
    ],
    "avgScore": 80
  }
}
```

## Display & Visualization

### Health Badge

Color coding based on score:

```typescript
function getHealthColor(score: number): string {
  if (score >= 90) return 'green'; // Excellent
  if (score >= 70) return 'yellow'; // Good
  if (score >= 50) return 'orange'; // Needs improvement
  return 'red'; // Critical
}
```

### Dashboard Cards

```tsx
<Card>
  <CardHeader>
    <CardTitle>Repository Health</CardTitle>
  </CardHeader>
  <CardContent>
    <div className="flex items-center gap-4">
      <div className={`text-4xl font-bold text-${getHealthColor(score)}-500`}>{score}</div>
      <div className="flex-1">
        <div className="text-sm text-muted-foreground">
          Avg merge time: {formatDuration(avgTimeToMerge)}
        </div>
        <div className="text-sm text-muted-foreground">Stale PRs: {stalePrCount}</div>
      </div>
    </div>
  </CardContent>
</Card>
```

### Trend Charts

Historical health scores can be displayed as line charts:

```typescript
const { data: healthHistory } = useQuery({
  queryKey: ['repository', repoId, 'health-history'],
  queryFn: () => analyticsApi.getHealthHistory(repoId, '30d'),
});

<LineChart data={healthHistory} xAxis="calculatedAt" yAxis="score" />
```

## Impact on PR Status

Health scores are informational and do not affect PR status calculations. However, they can inform team decisions:

- Repositories with low scores may need process improvements
- High stale PR counts suggest review bottlenecks
- Slow merge times indicate approval delays or CI issues

## Improving Health Scores

### Reduce Merge Time

- Automate more checks
- Set up auto-merge rules
- Reduce approval requirements for low-risk changes
- Use bulk merge for ready PRs

### Reduce Review Time

- Set up review assignments
- Enable notifications
- Implement review SLAs
- Use code owners

### Reduce Stale PRs

- Regular PR cleanup sessions
- Automated stale PR notifications
- Close abandoned PRs
- Break up large PRs

### Increase Throughput

- Encourage smaller PRs
- Reduce PR scope
- Parallel development
- Use feature flags

## Performance Considerations

### Calculation Frequency

- Default: Daily at midnight
- Configurable via scheduler
- Can be triggered manually for specific repos

### Query Optimization

```rust
// Efficient aggregation queries
let metrics = pr_metrics::Entity::find()
    .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
    .filter(pr_metrics::Column::MergedAt.gte(thirty_days_ago))
    .all(db)
    .await?;

// Single count query for stale PRs
let stale_prs = pull_request::Entity::find()
    .filter(pull_request::Column::RepositoryId.eq(repo.id))
    .filter(pull_request::Column::State.eq("open"))
    .filter(pull_request::Column::CreatedAt.lt(seven_days_ago))
    .count(db)
    .await?;
```

### Caching

Health scores are pre-calculated and stored, so API requests are fast:

- No real-time calculation overhead
- Latest score is always available
- Historical scores can be queried

## Testing

### Unit Tests

```rust
#[test]
fn test_score_calculation() {
    let job = HealthScoreJob;

    // Perfect score
    let score = job.calculate_score(None, None, 0, 10);
    assert_eq!(score, 100);

    // Slow merge time
    let score = job.calculate_score(Some(259200), None, 0, 0); // 72h
    assert_eq!(score, 70); // -30 penalty

    // Many stale PRs
    let score = job.calculate_score(None, None, 15, 0);
    assert_eq!(score, 75); // -25 penalty

    // High throughput bonus
    let score = job.calculate_score(None, None, 0, 12);
    assert_eq!(score, 100); // +10 bonus, clamped to 100
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_health_score_calculation() {
    let db = create_test_db().await;
    let repo = create_test_repository(&db).await;

    // Create merged PRs with metrics
    create_pr_metrics(&db, repo.id, 3600, 1800).await; // 1h merge, 30m review

    // Create stale PR
    create_old_pr(&db, repo.id, 10).await; // 10 days old

    let job = HealthScoreJob;
    job.calculate_repo_health(&db, &repo).await.unwrap();

    let health = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert!(health.score >= 80 && health.score <= 90);
}
```

## Troubleshooting

### Common Issues

**Problem:** Health score is 0

- **Solution:** Check if repository has any merged PRs in last 30 days

**Problem:** Score hasn't updated

- **Solution:** Verify worker job is running and check job logs

**Problem:** Inaccurate metrics

- **Solution:** Ensure pr_metrics are being collected properly for merged PRs

**Problem:** All repositories have low scores

- **Solution:** Check time zone settings and metric collection

### Debug Logging

```bash
export RUST_LOG=debug,ampel_worker::jobs::health_score=trace
make dev-worker
```

View calculated scores:

```rust
tracing::debug!(
    "Health score for {}: {} (stale: {}, throughput: {})",
    repo.full_name,
    score,
    stale_prs,
    recent_metrics
);
```

## Future Enhancements

### Planned Improvements

1. **Failed Check Rate**
   - Track CI failure percentage
   - Penalize repositories with frequent failures
   - Encourage better test quality

2. **Team Comparisons**
   - Compare health across teams
   - Identify best practices from high-scoring repos
   - Team-level health dashboards

3. **Configurable Weights**
   - Allow admins to adjust scoring formula
   - Different weights for different teams
   - Industry-specific scoring profiles

4. **Trend Analysis**
   - Track score changes over time
   - Identify improving/declining repositories
   - Alert on significant score drops

5. **Health Actions**
   - Automated suggestions for improvement
   - Link to relevant automation rules
   - Integration with notification system

6. **Custom Metrics**
   - Add repository-specific metrics
   - Custom scoring formulas
   - Integration with external tools

## Related Files

- Backend:
  - `crates/ampel-worker/src/jobs/health_score.rs` (calculation logic)
  - `crates/ampel-db/src/entities/health_score.rs` (entity definition)
  - `crates/ampel-db/src/entities/pr_metrics.rs` (metrics source)
  - `crates/ampel-api/src/handlers/analytics.rs` (API endpoints)

- Frontend:
  - `frontend/src/pages/Analytics.tsx` (health dashboard, to be created)
  - `frontend/src/components/analytics/HealthScore.tsx` (to be created)
  - `frontend/src/api/analytics.ts` (API client)

## Example Queries

### Get Latest Health Score

```sql
SELECT * FROM health_scores
WHERE repository_id = 'uuid'
ORDER BY calculated_at DESC
LIMIT 1;
```

### Get Health Trend (Last 30 Days)

```sql
SELECT calculated_at, score
FROM health_scores
WHERE repository_id = 'uuid'
  AND calculated_at >= NOW() - INTERVAL '30 days'
ORDER BY calculated_at ASC;
```

### Find Unhealthy Repositories

```sql
SELECT r.full_name, h.score, h.stale_pr_count
FROM repositories r
JOIN health_scores h ON h.repository_id = r.id
WHERE h.calculated_at = (
    SELECT MAX(calculated_at)
    FROM health_scores
    WHERE repository_id = r.id
)
AND h.score < 50
ORDER BY h.score ASC;
```

### Average Health by User

```sql
SELECT u.email, AVG(h.score) as avg_health
FROM users u
JOIN repositories r ON r.user_id = u.id
JOIN health_scores h ON h.repository_id = r.id
WHERE h.calculated_at >= NOW() - INTERVAL '7 days'
GROUP BY u.id, u.email
ORDER BY avg_health DESC;
```
