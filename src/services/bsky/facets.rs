use super::structs::{Facet, Feature, FeatureType, Index};
use crate::database::models::User;
use regex::bytes::Regex;
use std::{collections::HashMap, ops::Deref, sync::LazyLock};

const URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*[-a-zA-Z0-9@%_\+~#//=])?)")
        .unwrap()
});

const MENTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(@([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)").unwrap()
});

const TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|\s)(#[^\d\s]\S*)").unwrap());

struct Span {
    start: u64,
    end: u64,
    span: String,
}

fn parse_tags(text: &str) -> Vec<Span> {
    let text_bytes = text.as_bytes();
    let mut spans: Vec<Span> = Vec::new();

    for m in TAG_REGEX.find_iter(text_bytes) {
        let start = m.start() as u64;
        let end = m.end() as u64;

        if let Some(capture) = TAG_REGEX.captures(m.as_bytes()) {
            let tag = capture.get(1).unwrap().as_bytes()[1..].to_vec();
            let tag = String::from_utf8(tag).unwrap();
            spans.push(Span {
                start,
                end,
                span: tag,
            });
        }
    }

    spans
}

fn parse_mentions(text: &str) -> Vec<Span> {
    let text_bytes = text.as_bytes();
    let mut spans: Vec<Span> = Vec::new();

    for m in MENTION_REGEX.find_iter(text_bytes) {
        let start = m.start() as u64;
        let end = m.end() as u64;

        if let Some(capture) = MENTION_REGEX.captures(m.as_bytes()) {
            let handle = capture.get(1).unwrap().as_bytes()[1..].to_vec();
            let handle = String::from_utf8(handle).unwrap();
            spans.push(Span {
                start,
                end,
                span: handle,
            });
        }
    }

    spans
}

fn parse_urls(text: &str) -> Vec<Span> {
    let text_bytes = text.as_bytes();
    let mut spans: Vec<Span> = Vec::new();

    for m in URL_REGEX.find_iter(text_bytes) {
        let start = m.start() as u64;
        let end = m.end() as u64;
        if let Some(capture) = URL_REGEX.captures(m.as_bytes()) {
            let url = String::from_utf8(capture.get(1).unwrap().as_bytes().to_vec()).unwrap();
            spans.push(Span {
                start,
                end,
                span: url,
            });
        }
    }

    spans
}

pub fn parse_facets_with_users(text: &String, users: &Vec<User>) -> Vec<Facet> {
    let mention_spans = parse_mentions(text.as_str());
    let url_spans = parse_urls(text.as_str());
    let tag_spans = parse_tags(text.as_str());

    let handles_map: HashMap<&String, &String> =
        users.iter().map(|user| (&user.handle, &user.did)).collect();

    let mut facets: Vec<Facet> = Vec::new();

    for mention in mention_spans {
        if let Some(&handle) = handles_map.get(&mention.span) {
            facets.push(Facet {
                index: Index {
                    byte_start: mention.start,
                    byte_end: mention.end,
                },
                features: vec![Feature {
                    feature_type: FeatureType::Mention,
                    did: Some(handle.deref().to_string()),
                    uri: None,
                    tag: None,
                }],
            })
        } else {
            continue;
        }
    }

    for link in url_spans {
        facets.push(Facet {
            index: Index {
                byte_start: link.start,
                byte_end: link.end,
            },
            features: vec![Feature {
                feature_type: FeatureType::Link,
                did: None,
                uri: Some(link.span),
                tag: None,
            }],
        })
    }

    for tag in tag_spans {
        facets.push(Facet {
            index: Index {
                byte_start: tag.start,
                byte_end: tag.end,
            },
            features: vec![Feature {
                feature_type: FeatureType::Tag,
                did: None,
                uri: None,
                tag: Some(tag.span),
            }],
        })
    }

    facets
}
