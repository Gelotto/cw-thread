//! Comprehensive test suite for cw-thread contract.
//!
//! Tests cover authorization, functionality, validation, queries, and edge cases
//! to ensure the contract behaves correctly in all scenarios.

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, coins, from_json, Addr, Coin, Uint128};
    use cw_lib::models::{Owner, TokenAmountV2, TokenV2};

    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{
        ConfigUpdate, ExecuteMsg, InstantiateMsg, NodeEditMsg, NodeReplyMsg, NodeVoteMsg,
        NodesQueryMsg, QueryMsg, ThreadInfoResponse,
    };
    use crate::state::models::{Config, Section, ROOT_ID};

    // ============================================================================
    // Test Helpers
    // ============================================================================

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            owner: Some(Owner::Address(Addr::unchecked("owner"))),
            title: Some("Test Thread".to_string()),
            body: Some("This is a test thread body".to_string()),
            sections: None,
            tags: Some(vec!["test".to_string(), "discussion".to_string()]),
            mentions: Some(vec!["@alice".to_string()]),
            config: Config {
                tip_tokens: vec![TokenV2::Denom("uatom".to_string())],
            },
        }
    }

    fn create_thread() -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        cosmwasm_std::Env,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = default_instantiate_msg();

        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        (deps, env)
    }

    // ============================================================================
    // Authorization Tests (8 tests)
    // ============================================================================

    #[test]
    fn test_instantiate_sets_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let msg = default_instantiate_msg();

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "instantiate"));

        // Query to verify owner was set
        let query_msg = QueryMsg::Thread { sender: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let thread_info: ThreadInfoResponse = from_json(&res).unwrap();
        assert_eq!(thread_info.owner, Owner::Address(Addr::unchecked("owner")));
    }

    #[test]
    fn test_only_creator_can_edit_node() {
        let (mut deps, env) = create_thread();
        let info_creator = mock_info("creator", &[]);
        let info_other = mock_info("other_user", &[]);

        // Creator creates a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Test reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info_creator.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Try to edit as non-creator - should fail
        let edit_msg = NodeEditMsg {
            id: 1,
            body: Some("Edited by non-creator".to_string()),
            title: None,
            sections: None,
            tags: None,
            mentions: None,
        };
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info_other,
            ExecuteMsg::Edit(edit_msg.clone()),
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));

        // Edit as creator - should succeed
        execute(
            deps.as_mut(),
            env,
            info_creator,
            ExecuteMsg::Edit(edit_msg),
        )
        .unwrap();
    }

    #[test]
    fn test_only_creator_or_owner_can_delete_node() {
        let (mut deps, env) = create_thread();
        let info_creator = mock_info("creator", &[]);
        let info_owner = mock_info("owner", &[]);
        let info_other = mock_info("other_user", &[]);

        // Creator creates a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Test reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info_creator.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Try to delete as non-creator/non-owner - should fail
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info_other,
            ExecuteMsg::Delete { id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));

        // Delete as owner - should succeed
        execute(deps.as_mut(), env, info_owner, ExecuteMsg::Delete { id: 1 }).unwrap();
    }

    #[test]
    fn test_only_owner_can_set_config() {
        let (mut deps, env) = create_thread();
        let info_owner = mock_info("owner", &[]);
        let info_other = mock_info("other_user", &[]);

        let config_update = ConfigUpdate {
            tip_tokens: Some(vec![TokenV2::Denom("uosmo".to_string())]),
        };

        // Try as non-owner - should fail
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info_other,
            ExecuteMsg::SetConfig(config_update.clone()),
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));

        // Try as owner - should succeed
        execute(
            deps.as_mut(),
            env,
            info_owner,
            ExecuteMsg::SetConfig(config_update),
        )
        .unwrap();
    }

    #[test]
    fn test_owner_can_delete_any_node() {
        let (mut deps, env) = create_thread();
        let info_user = mock_info("user1", &[]);
        let info_owner = mock_info("owner", &[]);

        // User creates a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "User reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info_user,
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Owner deletes user's reply - should succeed
        execute(deps.as_mut(), env, info_owner, ExecuteMsg::Delete { id: 1 }).unwrap();
    }

    #[test]
    fn test_cannot_tip_yourself() {
        let (mut deps, env) = create_thread();
        let info_creator = mock_info("creator", &coins(100, "uatom"));

        let tip = TokenAmountV2 {
            token: TokenV2::Denom("uatom".to_string()),
            amount: Uint128::new(100),
        };

        let err = execute(deps.as_mut(), env, info_creator, ExecuteMsg::Tip(tip)).unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));
    }

    #[test]
    fn test_anyone_can_reply() {
        let (mut deps, env) = create_thread();
        let info_user = mock_info("random_user", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Anyone can reply!".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };

        let res = execute(deps.as_mut(), env, info_user, ExecuteMsg::Reply(reply_msg)).unwrap();
        assert_eq!(res.attributes[0], attr("action", "reply"));
    }

    #[test]
    fn test_anyone_can_vote() {
        let (mut deps, env) = create_thread();
        let info_user = mock_info("random_user", &[]);

        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Up,
        };

        let res = execute(deps.as_mut(), env, info_user, ExecuteMsg::Vote(vote_msg)).unwrap();
        assert_eq!(res.attributes[0], attr("action", "vote"));
    }

    // ============================================================================
    // Reply/Thread Tests (6 tests)
    // ============================================================================

    #[test]
    fn test_reply_increments_parent_count() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Query initial state
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.n_replies, 0);

        // Create a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "First reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query again
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.n_replies, 1);
    }

    #[test]
    fn test_reply_to_nonexistent_parent_fails() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: 9999, // Non-existent
            body: "Reply to nothing".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::NodeNotFound { .. }));
    }

    #[test]
    fn test_reply_creates_child_relationship() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Child reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query children
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::Children {
            id: ROOT_ID,
            cursor: None,
            limit: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewRepliesPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);
        assert_eq!(response.nodes[0].metadata.parent_id, Some(ROOT_ID));
    }

    #[test]
    fn test_multiple_replies_to_same_parent() {
        let (mut deps, env) = create_thread();

        for i in 0..3 {
            let info = mock_info(&format!("user{}", i), &[]);
            let reply_msg = NodeReplyMsg {
                parent_id: ROOT_ID,
                body: format!("Reply {}", i),
                sections: None,
                tags: None,
                mentions: None,
            };
            execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Reply(reply_msg)).unwrap();
        }

        // Query children
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::Children {
            id: ROOT_ID,
            cursor: None,
            limit: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewRepliesPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 3);
    }

    #[test]
    fn test_nested_replies() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Level 1 reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Level 1".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Level 2 reply
        let reply_msg = NodeReplyMsg {
            parent_id: 1,
            body: "Level 2".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query node 2
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![2],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.depth, 2);
    }

    #[test]
    fn test_reply_updates_activity_score() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Query initial state
        let query_msg = QueryMsg::Thread { sender: None };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let thread_info: ThreadInfoResponse = from_json(&res).unwrap();
        let initial_score = thread_info.activity_score;

        // Create a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Activity!".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query again
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let thread_info: ThreadInfoResponse = from_json(&res).unwrap();
        assert!(thread_info.activity_score > initial_score);
    }

    // ============================================================================
    // Voting Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_upvote_increases_rank() {
        let (mut deps, env) = create_thread();
        let info = mock_info("voter", &[]);

        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Up,
        };

        execute(deps.as_mut(), env, info, ExecuteMsg::Vote(vote_msg)).unwrap();

        // Query node
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.rank, 1);
    }

    #[test]
    fn test_downvote_decreases_rank() {
        let (mut deps, env) = create_thread();
        let info = mock_info("voter", &[]);

        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Down,
        };

        execute(deps.as_mut(), env, info, ExecuteMsg::Vote(vote_msg)).unwrap();

        // Query node
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.rank, -1);
    }

    #[test]
    fn test_toggle_vote_changes_sentiment() {
        let (mut deps, env) = create_thread();
        let info = mock_info("voter", &[]);

        // First upvote
        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Up,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Vote(vote_msg),
        )
        .unwrap();

        // Change to downvote
        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Down,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Vote(vote_msg)).unwrap();

        // Query node - rank should be -1 (changed from +1 to -1, net change of -2)
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.rank, -1);
    }

    #[test]
    fn test_remove_vote() {
        let (mut deps, env) = create_thread();
        let info = mock_info("voter", &[]);

        // Upvote
        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Up,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Vote(vote_msg),
        )
        .unwrap();

        // Remove vote (nil)
        let vote_msg = NodeVoteMsg {
            id: ROOT_ID,
            vote: crate::state::models::Sentiment::Nil,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Vote(vote_msg)).unwrap();

        // Query node
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.rank, 0);
    }

    #[test]
    fn test_vote_many_updates_multiple_nodes() {
        let (mut deps, env) = create_thread();
        let info_user = mock_info("user1", &[]);
        let info_voter = mock_info("voter", &[]);

        // Create two replies
        for i in 0..2 {
            let reply_msg = NodeReplyMsg {
                parent_id: ROOT_ID,
                body: format!("Reply {}", i),
                sections: None,
                tags: None,
                mentions: None,
            };
            execute(
                deps.as_mut(),
                env.clone(),
                info_user.clone(),
                ExecuteMsg::Reply(reply_msg),
            )
            .unwrap();
        }

        // Vote on both
        let votes = vec![
            NodeVoteMsg {
                id: 1,
                vote: crate::state::models::Sentiment::Up,
            },
            NodeVoteMsg {
                id: 2,
                vote: crate::state::models::Sentiment::Up,
            },
        ];

        execute(deps.as_mut(), env, info_voter, ExecuteMsg::VoteMany(votes)).unwrap();

        // Query both nodes
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![1, 2],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.rank, 1);
        assert_eq!(nodes[1].metadata.rank, 1);
    }

    // ============================================================================
    // Deletion Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_delete_removes_from_storage() {
        let (mut deps, env) = create_thread();
        let info_user = mock_info("user1", &[]);

        // Create a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "To be deleted".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info_user.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Delete it
        execute(
            deps.as_mut(),
            env,
            info_user,
            ExecuteMsg::Delete { id: 1 },
        )
        .unwrap();

        // Query should fail
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![1],
            sender: None,
        });
        let err = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(matches!(err, ContractError::NodeNotFound { .. }));
    }

    #[test]
    fn test_delete_with_children_recursive() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create parent reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Parent".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Create child reply
        let reply_msg = NodeReplyMsg {
            parent_id: 1,
            body: "Child".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Delete parent (should delete child too)
        execute(deps.as_mut(), env, info, ExecuteMsg::Delete { id: 1 }).unwrap();

        // Both should be gone
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![1, 2],
            sender: None,
        });
        let err = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(matches!(err, ContractError::NodeNotFound { .. }));
    }

    #[test]
    fn test_delete_updates_parent_count() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Temporary reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Verify parent count is 1
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.n_replies, 1);

        // Delete the reply
        execute(deps.as_mut(), env, info, ExecuteMsg::Delete { id: 1 }).unwrap();

        // Verify parent count is 0
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.n_replies, 0);
    }

    #[test]
    fn test_unauthorized_deletion_fails() {
        let (mut deps, env) = create_thread();
        let info_creator = mock_info("user1", &[]);
        let info_other = mock_info("user2", &[]);

        // Create a reply
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "User1's reply".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info_creator,
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Try to delete as different user
        let err = execute(deps.as_mut(), env, info_other, ExecuteMsg::Delete { id: 1 }).unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));
    }

    #[test]
    fn test_delete_nonexistent_node_fails() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Delete { id: 9999 }).unwrap_err();
        assert!(matches!(err, ContractError::NodeNotFound { .. }));
    }

    // ============================================================================
    // Tag/Mention Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_tags_stored_correctly() {
        let (deps, _env) = create_thread();

        // Query nodes with tag
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithTag {
            tag: "test".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);
    }

    #[test]
    fn test_mentions_stored_correctly() {
        let (deps, _env) = create_thread();

        // Query nodes with mention
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithMention {
            mention: "alice".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);
    }

    #[test]
    fn test_query_nodes_by_tag() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create reply with tag
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Tagged reply".to_string(),
            sections: None,
            tags: Some(vec!["rust".to_string()]),
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query by tag
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithTag {
            tag: "rust".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);
    }

    #[test]
    fn test_query_nodes_by_mention() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create reply with mention
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Mentioned reply".to_string(),
            sections: None,
            tags: None,
            mentions: Some(vec!["@bob".to_string()]),
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query by mention
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithMention {
            mention: "bob".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);
    }

    #[test]
    fn test_edit_updates_tags_and_mentions() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create reply with tags and mentions
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Original".to_string(),
            sections: None,
            tags: Some(vec!["old-tag".to_string()]),
            mentions: Some(vec!["@olduser".to_string()]),
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Edit with new tags and mentions
        let edit_msg = NodeEditMsg {
            id: 1,
            body: Some("Updated".to_string()),
            title: None,
            sections: None,
            tags: Some(vec!["new-tag".to_string()]),
            mentions: Some(vec!["@newuser".to_string()]),
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Edit(edit_msg)).unwrap();

        // Query with new tag should find it
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithTag {
            tag: "new-tag".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 1);

        // Query with old tag should not find it
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::WithTag {
            tag: "old-tag".to_string(),
            cursor: None,
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewByTagPaginationResponse = from_json(&res).unwrap();
        assert_eq!(response.nodes.len(), 0);
    }

    // ============================================================================
    // Tipping Tests (4 tests)
    // ============================================================================

    #[test]
    fn test_tip_with_native_token() {
        let (mut deps, env) = create_thread();
        let info = mock_info("tipper", &coins(100, "uatom"));

        let tip = TokenAmountV2 {
            token: TokenV2::Denom("uatom".to_string()),
            amount: Uint128::new(100),
        };

        let res = execute(deps.as_mut(), env, info, ExecuteMsg::Tip(tip)).unwrap();
        assert_eq!(res.attributes[0], attr("action", "tip"));
    }

    #[test]
    fn test_tip_with_unauthorized_token_fails() {
        let (mut deps, env) = create_thread();
        let info = mock_info("tipper", &coins(100, "unotallowed"));

        let tip = TokenAmountV2 {
            token: TokenV2::Denom("unotallowed".to_string()),
            amount: Uint128::new(100),
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Tip(tip)).unwrap_err();
        assert!(matches!(err, ContractError::UnauthorizedTipToken { .. }));
    }

    #[test]
    fn test_tip_insufficient_funds_fails() {
        let (mut deps, env) = create_thread();
        let info = mock_info("tipper", &coins(50, "uatom")); // Send 50 but claim 100

        let tip = TokenAmountV2 {
            token: TokenV2::Denom("uatom".to_string()),
            amount: Uint128::new(100),
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Tip(tip)).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientFunds { .. }));
    }

    #[test]
    fn test_tip_self_fails() {
        let (mut deps, env) = create_thread();
        let info = mock_info("creator", &coins(100, "uatom"));

        let tip = TokenAmountV2 {
            token: TokenV2::Denom("uatom".to_string()),
            amount: Uint128::new(100),
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Tip(tip)).unwrap_err();
        assert!(matches!(err, ContractError::NotAuthorized { .. }));
    }

    // ============================================================================
    // Validation Tests (8 tests)
    // ============================================================================

    #[test]
    fn test_reject_empty_body() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let mut msg = default_instantiate_msg();
        msg.body = Some("   ".to_string()); // Empty after trim

        let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_oversized_body() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let huge_body = "x".repeat(60_000); // Exceeds MAX_BODY_LENGTH
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: huge_body,
            sections: None,
            tags: None,
            mentions: None,
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_too_many_tags() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let tags: Vec<String> = (0..15).map(|i| format!("tag{}", i)).collect();
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Too many tags".to_string(),
            sections: None,
            tags: Some(tags),
            mentions: None,
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_invalid_tag_characters() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Invalid tag".to_string(),
            sections: None,
            tags: Some(vec!["tag with spaces!".to_string()]),
            mentions: None,
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_mention_without_at_symbol() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Invalid mention".to_string(),
            sections: None,
            tags: None,
            mentions: Some(vec!["alice".to_string()]), // Missing @
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_too_many_mentions() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let mentions: Vec<String> = (0..25).map(|i| format!("@user{}", i)).collect();
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Too many mentions".to_string(),
            sections: None,
            tags: None,
            mentions: Some(mentions),
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_too_many_sections() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let sections: Vec<Section> = (0..25).map(|_| Section::Text("text".to_string())).collect();
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Too many sections".to_string(),
            sections: Some(sections),
            tags: None,
            mentions: None,
        };

        let err = execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    #[test]
    fn test_reject_empty_title() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let mut msg = default_instantiate_msg();
        msg.title = Some("   ".to_string()); // Empty after trim

        let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::ValidationError { .. }));
    }

    // ============================================================================
    // Query Tests (4 tests)
    // ============================================================================

    #[test]
    fn test_query_thread_info() {
        let (deps, _env) = create_thread();

        let query_msg = QueryMsg::Thread { sender: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let thread_info: ThreadInfoResponse = from_json(&res).unwrap();

        assert_eq!(thread_info.title, Some("Test Thread".to_string()));
        assert_eq!(thread_info.owner, Owner::Address(Addr::unchecked("owner")));
    }

    #[test]
    fn test_query_node_by_id() {
        let (deps, _env) = create_thread();

        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![ROOT_ID],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].metadata.id, ROOT_ID);
    }

    #[test]
    fn test_query_children_paginated() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create multiple replies
        for i in 0..5 {
            let reply_msg = NodeReplyMsg {
                parent_id: ROOT_ID,
                body: format!("Reply {}", i),
                sections: None,
                tags: None,
                mentions: None,
            };
            execute(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                ExecuteMsg::Reply(reply_msg),
            )
            .unwrap();
        }

        // Query with limit
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::Children {
            id: ROOT_ID,
            cursor: None,
            limit: Some(3),
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let response: crate::msg::NodeViewRepliesPaginationResponse = from_json(&res).unwrap();

        assert_eq!(response.nodes.len(), 3);
        assert!(response.cursor.is_some());
    }

    #[test]
    fn test_query_ancestors() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create nested replies
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Level 1".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        let reply_msg = NodeReplyMsg {
            parent_id: 1,
            body: "Level 2".to_string(),
            sections: None,
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();

        // Query ancestors from node 2
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::Ancestors {
            id: 2,
            levels: Some(2),
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();

        assert_eq!(nodes.len(), 2); // Should get node 1 and ROOT
    }

    // ============================================================================
    // Edge Case Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_save_and_unsave_node() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Save the root node
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Save(vec![ROOT_ID]),
        )
        .unwrap();

        // Unsave the root node
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::Unsave(vec![ROOT_ID]),
        )
        .unwrap();
    }

    #[test]
    fn test_flag_and_unflag_node() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Flag the root node
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Flag {
                id: ROOT_ID,
                reason: Some("Spam".to_string()),
            },
        )
        .unwrap();

        // Unflag the root node
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::Unflag { id: ROOT_ID },
        )
        .unwrap();
    }

    #[test]
    fn test_edit_removes_old_sections() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        // Create reply with sections
        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Original".to_string(),
            sections: Some(vec![
                Section::Text("Section 1".to_string()),
                Section::Text("Section 2".to_string()),
            ]),
            tags: None,
            mentions: None,
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Reply(reply_msg),
        )
        .unwrap();

        // Edit with new sections
        let edit_msg = NodeEditMsg {
            id: 1,
            body: Some("Updated".to_string()),
            title: None,
            sections: Some(vec![Section::Text("New Section".to_string())]),
            tags: None,
            mentions: None,
        };
        execute(deps.as_mut(), env, info, ExecuteMsg::Edit(edit_msg)).unwrap();

        // Query to verify
        let query_msg = QueryMsg::Nodes(NodesQueryMsg::ByIds {
            ids: vec![1],
            sender: None,
        });
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let nodes: Vec<crate::state::views::NodeView> = from_json(&res).unwrap();
        assert_eq!(nodes[0].metadata.n_sections, 1);
    }

    #[test]
    fn test_multiple_users_can_save_same_node() {
        let (mut deps, env) = create_thread();

        for i in 1..4 {
            let info = mock_info(&format!("user{}", i), &[]);
            execute(
                deps.as_mut(),
                env.clone(),
                info,
                ExecuteMsg::Save(vec![ROOT_ID]),
            )
            .unwrap();
        }

        // All saves should succeed independently
    }

    #[test]
    fn test_valid_alphanumeric_tags_accepted() {
        let (mut deps, env) = create_thread();
        let info = mock_info("user1", &[]);

        let reply_msg = NodeReplyMsg {
            parent_id: ROOT_ID,
            body: "Valid tags".to_string(),
            sections: None,
            tags: Some(vec![
                "rust-lang".to_string(),
                "web3_dev".to_string(),
                "CosmWasm123".to_string(),
            ]),
            mentions: None,
        };

        execute(deps.as_mut(), env, info, ExecuteMsg::Reply(reply_msg)).unwrap();
    }
}
