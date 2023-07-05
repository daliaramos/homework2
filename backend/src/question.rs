use std::fmt;

use derive_more::Display;
use serde_derive::{Deserialize, Serialize};

// This uses the `derive_more` crate to reduce the Display boilerplate (see below)
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[display(
fmt = "id: {}, title: {}, content: {}, tags: {:?}",
id,
title,
content,
tags
)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
}

impl Question {
    pub fn new(id: QuestionId, title: String, content: String, tags: Option<String>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
pub struct QuestionId(pub usize);

// Clients use this to create new requests
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuestion {
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct GetQuestionById {
    pub question_id: usize,
}


