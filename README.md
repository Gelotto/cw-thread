# CW-Thread: On-Chain Discussion Forum

A CosmWasm smart contract that implements a full-featured hierarchical discussion forum directly on-chain. Think Reddit or HackerNews but with all posts, votes, tips, and relationships living in contract storage.

## Overview

CW-Thread provides a complete social discussion platform as a smart contract, treating threads as directed acyclic graphs where nodes represent posts/replies, edges represent parent-child relationships, and votes compute dynamic ranking. The contract supports rich content (images, code blocks, links), social features (@mentions, #tags, save/flag), and integrates with cw-table for cross-thread discovery.

**Key Capabilities:**
- Hierarchical threaded discussions with unlimited nesting (up to depth 255)
- Vote-based ranking system for content quality
- Native and CW20 token tipping for thread creators
- Rich content sections (text, images, code, links)
- Social features: tags, mentions, save, flag
- Multi-dimensional indexing for O(1) relationship lookups
- Recursive child deletion with referential integrity
- Input validation and authorization controls
- Integration with cw-table for discovery and lifecycle management

## Features

### Content Management
- **Thread Creation**: Initialize with title, body, sections, tags, and mentions
- **Nested Replies**: Create unlimited reply chains with automatic depth tracking
- **Rich Sections**: Support for text, images, code blocks (with syntax highlighting), and links
- **Edit Posts**: Creators can update body, sections, tags, and mentions
- **Delete Posts**: Recursive deletion removes node and all descendants

### Social Interactions
- **Voting**: Upvote/downvote posts with dynamic rank calculation
- **Tipping**: Send native or CW20 tokens to thread creators
- **Tags**: Organize content with up to 10 tags per post
- **Mentions**: Reference users with @ mentions (up to 20 per post)
- **Save/Unsave**: Bookmark posts for later reference
- **Flag/Unflag**: Report content with optional reason

### Authorization Model
- **Owner**: Can delete any post, update configuration, managed via Owner::Address or Owner::Acl
- **Post Creator**: Can edit and delete their own posts
- **All Users**: Can reply, vote, tip, save, and flag
- **Table Contract**: Controls lifecycle operations (setup, teardown, suspend, resume)

## Architecture

### Storage Model

The contract uses a node-based storage model where each post is a node with an auto-incrementing ID starting from ROOT_ID (0).

**Core Data Structures:**
- `NodeMetadata`: ID, timestamps, creator, parent, rank, depth, reply/flag counts
- `NodeBody`: Main HTML content
- `NodeSections`: Optional rich content (images, code, links)
- `NodeTags/Mentions`: Social metadata for discovery

**Multi-Dimensional Indices:**
- `IX_CHILD(parent_id, child_id)`: Forward parent-child lookup
- `IX_RANKED_CHILD(parent_id, rank, child_id)`: Sorted children by rank
- `IX_TAG_NODE(tag, node_id)`: Find nodes by tag
- `IX_MENTION_NODE(mention, node_id)`: Find nodes by mention
- `IX_NODE_TAG(node_id, tag)`: Find tags for node (reverse index)
- `IX_NODE_MENTION(node_id, mention)`: Find mentions for node (reverse index)

### Node Hierarchy

```
ROOT (id=0, depth=0)
├── Reply 1 (id=1, depth=1)
│   ├── Reply 1.1 (id=2, depth=2)
│   └── Reply 1.2 (id=3, depth=2)
└── Reply 2 (id=4, depth=1)
    └── Reply 2.1 (id=5, depth=2)
```

### Voting & Ranking

Posts have an integer rank that changes based on votes:
- Upvote: `rank += 1`
- Downvote: `rank -= 1`
- Remove vote: adjusts rank back to previous state
- Toggle vote: removes old vote and applies new vote

Children are ordered by `(parent_id, rank DESC, child_id)` for efficient sorted queries.

### Activity Score

The thread maintains a global activity score that increases with each reply:
```
activity_score += (255 / reply_depth)
```
Deeper replies contribute less to activity, encouraging top-level discussion.

## Usage Examples

### Instantiate

```bash
{
  "owner": { "address": "juno1..." },
  "title": "Welcome to CW-Thread",
  "body": "This is the main discussion thread",
  "sections": [
    { "text": "Introduction paragraph" },
    { "image": "https://example.com/banner.png" },
    { "code": {
        "lang": "rust",
        "text": "fn main() { println!(\"Hello\"); }"
      }
    }
  ],
  "tags": ["announcement", "welcome"],
  "mentions": ["@admin", "@moderator"],
  "config": {
    "tip_tokens": [
      { "denom": "ujuno" },
      { "address": "juno1cw20contractaddr..." }
    ]
  }
}
```

### Reply to Post

```bash
{
  "reply": {
    "parent_id": 0,
    "body": "Great discussion! Here's my take...",
    "sections": [
      { "link": {
          "text": "Related article",
          "url": "https://example.com/article"
        }
      }
    ],
    "tags": ["discussion", "feedback"],
    "mentions": ["@originalPoster"]
  }
}
```

### Vote on Post

```bash
# Upvote
{ "vote": { "id": 1, "vote": "up" } }

# Downvote
{ "vote": { "id": 1, "vote": "down" } }

# Remove vote
{ "vote": { "id": 1, "vote": "nil" } }
```

### Vote on Multiple Posts

```bash
{
  "vote_many": [
    { "id": 1, "vote": "up" },
    { "id": 2, "vote": "up" },
    { "id": 3, "vote": "down" }
  ]
}
```

### Edit Post

```bash
{
  "edit": {
    "id": 1,
    "body": "Updated content with corrections",
    "sections": [
      { "text": "New section added" }
    ],
    "tags": ["updated", "corrected"],
    "mentions": []
  }
}
```

### Tip Thread Creator

```bash
# Native token tip
{
  "tip": {
    "token": { "denom": "ujuno" },
    "amount": "1000000"
  }
}

# CW20 token tip (requires prior approval)
{
  "tip": {
    "token": { "address": "juno1cw20..." },
    "amount": "1000000"
  }
}
```

### Delete Post

```bash
# Deletes post and all descendants recursively
{ "delete": { "id": 1 } }

# Delete root (purges entire contract state)
{ "delete": { "id": 0 } }
```

### Save/Unsave Posts

```bash
# Save posts
{ "save": [1, 2, 3] }

# Unsave posts
{ "unsave": [1, 2] }
```

### Flag/Unflag Content

```bash
# Flag with reason
{
  "flag": {
    "id": 1,
    "reason": "Spam content"
  }
}

# Unflag
{ "unflag": { "id": 1 } }
```

### Query Thread Info

```bash
{
  "thread": {
    "sender": "juno1..." # Optional: includes user-specific data
  }
}
```

### Query Nodes by ID

```bash
{
  "nodes": {
    "by_ids": {
      "ids": [0, 1, 2],
      "sender": "juno1..." # Optional
    }
  }
}
```

### Query Child Nodes (Paginated)

```bash
{
  "nodes": {
    "children": {
      "id": 0,
      "cursor": null, # Or [parent_id, rank, child_id] for pagination
      "limit": 25,
      "sender": "juno1..."
    }
  }
}
```

### Query Ancestor Nodes

```bash
{
  "nodes": {
    "ancestors": {
      "id": 5,
      "levels": 3, # Number of levels to traverse
      "sender": null
    }
  }
}
```

### Query Nodes by Tag

```bash
{
  "nodes": {
    "with_tag": {
      "tag": "rust",
      "cursor": null, # Or node_id for pagination
      "sender": null
    }
  }
}
```

### Query Nodes by Mention

```bash
{
  "nodes": {
    "with_mention": {
      "mention": "alice", # Without @ prefix
      "cursor": null,
      "sender": null
    }
  }
}
```

## Validation Limits

The contract enforces the following limits to prevent abuse:

| **Constraint** | **Limit** | **Reason** |
|----------------|-----------|------------|
| Title length | 200 characters | Keep titles concise for UI display |
| Body length | 50,000 characters | Allow substantial content while preventing storage abuse |
| Tags per post | 10 | Prevent tag spam |
| Tag length | 30 characters | Keep tags concise |
| Tag format | Alphanumeric + hyphens + underscores | Clean indexing and display |
| Mentions per post | 20 | Prevent mention spam |
| Mention format | Must start with @ | Consistent syntax |
| Sections per post | 20 | Limit rich content complexity |
| Section content | Same as body (50k chars) | Consistent limits |
| Reply depth | 255 levels | Practical limit (u8::MAX) |

All validation is performed upfront before state changes, ensuring:
- Invalid data never enters storage
- Clear error messages for users
- Consistent data quality across the contract

## Security

### Authorization Rules

1. **Instantiation**: Anyone can create a thread
2. **Reply**: Anyone can reply to any post
3. **Vote**: Anyone can vote on any post
4. **Edit**: Only the post creator can edit their post
5. **Delete**: Post creator OR contract owner can delete
6. **Tip**: Anyone except the thread creator can tip
7. **SetConfig**: Only the owner can update configuration
8. **Lifecycle**: Only the table contract can control lifecycle

### Owner vs Creator

- **Owner** (`OWNER` state): Controls the entire thread contract, can delete any post
- **Creator** (`created_by` in NodeMetadata): Created a specific post, can edit/delete only their own posts

The owner can be:
- `Owner::Address(Addr)`: Single address with full control
- `Owner::Acl(Addr)`: ACL contract for fine-grained permissions

### Input Validation

All user input is validated before storage:
- Length limits prevent resource exhaustion
- Format constraints ensure consistent data
- Authorization checks prevent unauthorized actions
- Referential integrity maintained via parent checks

### Recursive Deletion

When deleting a post with children:
1. Depth-first traversal collects all descendant IDs
2. Descendants deleted bottom-up to avoid orphans
3. Parent reply counts updated atomically
4. Table activity score updated if applicable

This ensures:
- No orphaned nodes
- Consistent parent-child relationships
- Proper cleanup of all associated data (tags, mentions, votes, flags)

## Integration

### CW-Table Integration

CW-Thread integrates with [cw-table](../cw-table) for cross-thread discovery and lifecycle management:

**Setup Phase:**
```
Table Contract → Thread.Setup() → Thread saves table metadata
```

**Operations:**
- Thread updates table on replies (activity score index)
- Thread updates table on tips (tip amount indices)
- Table can suspend/resume/teardown thread

**Teardown:**
- Table calls `Teardown` to remove thread from table
- Root deletion purges all thread state

**Indices Updated:**
- `activity: u32` - Updated on each reply
- `tip:{token_key}: u128` - Updated on each tip

### ACL Integration

When owner is `Owner::Acl(addr)`, authorization checks query the ACL contract:

```rust
// Check if principal can perform action
acl.is_allowed(querier, principal, "/thread/delete") -> bool
```

Common actions:
- `/thread/delete` - Delete any post
- `/thread/config` - Update configuration

## Development

### Prerequisites

- Rust 1.70+
- `wasm32-unknown-unknown` target
- CosmWasm dependencies (see Cargo.toml)

### Build

```bash
# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Optimize the wasm
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0
```

### Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_reply_increments_parent_count

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- 56+ comprehensive tests
- Authorization (8 tests)
- Reply/Thread operations (6 tests)
- Voting (5 tests)
- Deletion (5 tests)
- Tags/Mentions (5 tests)
- Tipping (4 tests)
- Validation (8 tests)
- Queries (4 tests)
- Edge cases (5 tests)

### Generate Schemas

```bash
cargo schema
```

Schemas are exported to `schema/`:
- `instantiate_msg.json`
- `execute_msg.json`
- `query_msg.json`

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Data Model

### Node States

A node can be in several states:

**Lifecycle:**
- Active: Normal operational state
- Deleted: Removed from storage (no longer queryable)

**User Interactions (per address):**
- Voted: User has upvoted/downvoted
- Saved: User has bookmarked
- Flagged: User has reported content

### Section Types

```rust
pub enum Section {
    Text(String),
    Image(String), // URL
    Code { lang: Option<String>, text: String },
    Link { text: Option<String>, url: String },
}
```

### Sentiment (Vote)

```rust
pub enum Sentiment {
    Nil,  // No vote or vote removed
    Up,   // Upvote
    Down, // Downvote
}
```

## Gas Considerations

### Expensive Operations

- **Recursive Deletion**: O(N) where N is descendants count
  - Batched in reverse order to avoid parent reference issues
  - Deep trees with many descendants may approach gas limits
  - Recommended: document max tree size in production

- **Tag/Mention Indexing**: O(M) where M is tag/mention count
  - Forward and reverse indices both updated
  - Bounded by MAX_TAGS (10) and MAX_MENTIONS (20)

- **Vote Many**: O(V) where V is vote count
  - Each vote updates node metadata and ranked child index
  - Unbounded, use with caution on large batches

### Optimizations

- Node-scoped queries avoid full map iterations
- Depth-first traversal uses iterative stack (not recursion)
- Batched storage operations in delete (20 keys at a time)

## License

Apache-2.0

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Support

For questions, issues, or feature requests, please open an issue on the repository.
