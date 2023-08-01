use serde::{Deserialize, Serialize};

use crate::question::Question;
//use crate::comment::Comment;
use crate::answer::Answer;

#[derive(Debug, Serialize, Deserialize)]
pub struct PagePackage {
    question: Question,
    //question_comments: Option<Vec<Comment>>,
    answers: Option<Vec<Answer>>,
    //answer_comments: Option<Vec<Comment>>
}