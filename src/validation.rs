//! Input validation functions for thread content.
//!
//! All validation functions return ContractError::ValidationError on failure.
//! These enforce limits on content length, count, and format to prevent abuse
//! and ensure consistent data quality.

use crate::{
    error::ContractError,
    state::{
        models::Section,
        storage::{
            MAX_BODY_LENGTH, MAX_MENTIONS, MAX_SECTIONS, MAX_TAGS, MAX_TAG_LENGTH,
            MAX_TITLE_LENGTH,
        },
    },
};

/// Validates that a title is non-empty and within length limits.
///
/// Enforces MAX_TITLE_LENGTH (200 characters) to ensure titles remain concise
/// and displayable in UI lists.
pub fn validate_title(title: &str) -> Result<(), ContractError> {
    if title.trim().is_empty() {
        return Err(ContractError::ValidationError {
            reason: "Title cannot be empty".to_owned(),
        });
    }
    if title.len() > MAX_TITLE_LENGTH {
        return Err(ContractError::ValidationError {
            reason: format!(
                "Title length exceeds maximum of {} characters",
                MAX_TITLE_LENGTH
            ),
        });
    }
    Ok(())
}

/// Validates that a body is non-empty and within length limits.
///
/// Enforces MAX_BODY_LENGTH (50,000 characters) to prevent storage abuse
/// while allowing substantial content.
pub fn validate_body(body: &str) -> Result<(), ContractError> {
    if body.trim().is_empty() {
        return Err(ContractError::ValidationError {
            reason: "Body cannot be empty".to_owned(),
        });
    }
    if body.len() > MAX_BODY_LENGTH {
        return Err(ContractError::ValidationError {
            reason: format!(
                "Body length exceeds maximum of {} characters",
                MAX_BODY_LENGTH
            ),
        });
    }
    Ok(())
}

/// Validates tags: count, length, and alphanumeric format.
///
/// Enforces:
/// - MAX_TAGS (10) to prevent tag spam
/// - MAX_TAG_LENGTH (30 characters) per tag
/// - Alphanumeric format (plus hyphens and underscores) for clean indexing
pub fn validate_tags(tags: &Option<Vec<String>>) -> Result<(), ContractError> {
    if let Some(tag_list) = tags {
        if tag_list.len() > MAX_TAGS {
            return Err(ContractError::ValidationError {
                reason: format!("Number of tags exceeds maximum of {}", MAX_TAGS),
            });
        }

        for tag in tag_list {
            if tag.trim().is_empty() {
                return Err(ContractError::ValidationError {
                    reason: "Tag cannot be empty".to_owned(),
                });
            }
            if tag.len() > MAX_TAG_LENGTH {
                return Err(ContractError::ValidationError {
                    reason: format!(
                        "Tag '{}' exceeds maximum length of {} characters",
                        tag, MAX_TAG_LENGTH
                    ),
                });
            }
            // Tags should be alphanumeric (plus hyphens and underscores)
            if !tag
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(ContractError::ValidationError {
                    reason: format!(
                        "Tag '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores allowed",
                        tag
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Validates mentions: count and @ prefix format.
///
/// Enforces:
/// - MAX_MENTIONS (20) to prevent mention spam
/// - All mentions must start with @ symbol
/// - Mention must have content after the @ symbol
pub fn validate_mentions(mentions: &Option<Vec<String>>) -> Result<(), ContractError> {
    if let Some(mention_list) = mentions {
        if mention_list.len() > MAX_MENTIONS {
            return Err(ContractError::ValidationError {
                reason: format!("Number of mentions exceeds maximum of {}", MAX_MENTIONS),
            });
        }

        for mention in mention_list {
            if !mention.starts_with('@') {
                return Err(ContractError::ValidationError {
                    reason: format!("Mention '{}' must start with @", mention),
                });
            }
            if mention.len() <= 1 {
                return Err(ContractError::ValidationError {
                    reason: "Mention cannot be just @".to_owned(),
                });
            }
        }
    }
    Ok(())
}

/// Validates sections: count and basic content limits.
///
/// Enforces:
/// - MAX_SECTIONS (20) to limit rich content complexity
/// - Text/Code sections must be non-empty and within MAX_BODY_LENGTH
/// - Image/Link URLs must be non-empty
/// - Link text and code language (if specified) must be non-empty
pub fn validate_sections(sections: &Option<Vec<Section>>) -> Result<(), ContractError> {
    if let Some(section_list) = sections {
        if section_list.len() > MAX_SECTIONS {
            return Err(ContractError::ValidationError {
                reason: format!("Number of sections exceeds maximum of {}", MAX_SECTIONS),
            });
        }

        for (idx, section) in section_list.iter().enumerate() {
            // Validate section content based on type
            match section {
                Section::Text(text) => {
                    if text.trim().is_empty() {
                        return Err(ContractError::ValidationError {
                            reason: format!("Section {} text content cannot be empty", idx),
                        });
                    }
                    if text.len() > MAX_BODY_LENGTH {
                        return Err(ContractError::ValidationError {
                            reason: format!(
                                "Section {} text exceeds maximum length of {} characters",
                                idx, MAX_BODY_LENGTH
                            ),
                        });
                    }
                }
                Section::Image(url) => {
                    if url.trim().is_empty() {
                        return Err(ContractError::ValidationError {
                            reason: format!("Section {} image URL cannot be empty", idx),
                        });
                    }
                }
                Section::Code { lang, text } => {
                    if text.trim().is_empty() {
                        return Err(ContractError::ValidationError {
                            reason: format!("Section {} code content cannot be empty", idx),
                        });
                    }
                    if text.len() > MAX_BODY_LENGTH {
                        return Err(ContractError::ValidationError {
                            reason: format!(
                                "Section {} code exceeds maximum length of {} characters",
                                idx, MAX_BODY_LENGTH
                            ),
                        });
                    }
                    if let Some(language) = lang {
                        if language.trim().is_empty() {
                            return Err(ContractError::ValidationError {
                                reason: format!(
                                    "Section {} code language cannot be empty if specified",
                                    idx
                                ),
                            });
                        }
                    }
                }
                Section::Link { text, url } => {
                    if url.trim().is_empty() {
                        return Err(ContractError::ValidationError {
                            reason: format!("Section {} link URL cannot be empty", idx),
                        });
                    }
                    if let Some(link_text) = text {
                        if link_text.trim().is_empty() {
                            return Err(ContractError::ValidationError {
                                reason: format!(
                                    "Section {} link text cannot be empty if specified",
                                    idx
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
