# Pull Requests Implementation - Step 1 Complete

## Server Functions (`src/server/prs.rs`)

### Implemented Functions

1. **`create_pull_request(repo_id, title, body, head_branch, base_branch)`**
   - Creates a new pull request
   - Returns the PR UUID
   - PR default status: 'open'

2. **`list_pull_requests(repo_id, status)`**
   - Lists PRs for a repository, filtered by status
   - Default status filter: 'open'
   - Returns Vec<PullRequest>

3. **`get_pull_request(pr_id)`**
   - Gets full PR details with repo name and author info
   - Returns PullRequestDetail

4. **`get_repo_id_by_name(username, reponame)`**
   - Helper to get repo UUID from username/reponame
   - Used to avoid exposing UUIDs to frontend

5. **`merge_pull_request(pr_id, user_id)`**
   - Sets PR status to 'merged'
   - Returns error if PR not found or already merged/closed

6. **`close_pull_request(pr_id)`**
   - Sets PR status to 'closed'
   - Updates timestamp

7. **`get_branch_list_for_pr(username, reponame)`**
   - Returns list of branch names from git repo
   - Used for branch selection in create PR form

### Type Definitions

```rust
pub struct PullRequest {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct PullRequestDetail {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
```

### Files Modified

- **`src/lib.rs`**: Added `pub mod server;`
- **`src/server/mod.rs`**: Created new module
- **`src/server/prs.rs`**: Created with all server functions
- **`migrations/004_pull_requests.sql`**: Already exists, schema matches

### Next Steps

Continue with:
- Step 2: Update frontend pages to use PR server functions
- Step 3: Add PR tabs to RepoTabBar
- Step 4: Add PR sidebar to overview page
- Step 5: Implement Create PR form
- Step 6: Implement PR list page
- Step 7: Implement PR detail page

See `PULL_REQUESTS_IMPLEMENTATION.md` for full plan.
