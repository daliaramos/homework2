use std::sync::{Arc, Mutex, RwLock};

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::answer::{Answer, AnswerId};
use crate::comment::{Comment};
use crate::error::AppError;
use crate::pagePackage::{PagePackage};
use crate::question::{IntoQuestionId, Question, QuestionId, UpdateQuestion};

#[derive(Clone)]
pub struct Store {
    pub conn_pool: PgPool,
    pub questions: Arc<Mutex<Vec<Question>>>,
    pub answers: Arc<RwLock<Vec<Answer>>>,
    pub comments:Arc<RwLock<Vec<Comment>>>,
}

pub async fn new_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap()
}

impl Store {
    pub fn with_pool(pool: PgPool) -> Self {
        Self {
            conn_pool: pool,
            questions: Default::default(),
            answers: Default::default(),
            comments: Default::default(),
        }
    }

    pub async fn test_database(&self) -> Result<(), sqlx::Error> {
        let row: (i64, ) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&self.conn_pool)
            .await?;

        info!("{}", &row.0);

        assert_eq!(row.0, 150);
        Ok(())
    }

    pub async fn add_answer(
        &mut self,
        content: String,
        question_id: i32,
    ) -> Result<Answer, AppError> {
        dbg!("IN ADD ANSWER");


        sqlx::query!(
            r#"
    INSERT INTO answers (content, question_id)
    VALUES ($1, $2)
    "#,
            content,
            question_id,
        )
            .execute(&self.conn_pool)
            .await?;

        let row = sqlx::query!(
    "SELECT * FROM answers WHERE id = $1",
    question_id
)
            .fetch_one(&self.conn_pool)
            .await?;

        let answer = Answer {
            id: AnswerId(row.id),
            content: row.content,
            question_id: QuestionId(row.question_id.unwrap()),
        };

        Ok(answer)
    }


    pub fn add_comment(
        &mut self,
        content: String,
    ) -> Result<Comment, AppError> {
        let mut comment = self.comments.write().unwrap();
        let len = comment.len() as u32;

        let new_comment = Comment {
            id: len.into(),
            content,
        };
        comment.push(new_comment.clone());
        Ok(new_comment)
    }

    pub async fn get_all_questions(&mut self) -> Result<Vec<Question>, AppError> {
        let rows = sqlx::query!(
            r#"
SELECT * FROM questions
"#
        )
            .fetch_all(&self.conn_pool)
            .await?;

        let mut questions = Vec::new();

        for row in rows {
            let question = Question {
                id: row.id.into(), // Assuming you have a From<u32> for QuestionId
                title: row.title,
                content: row.content,
                tags: row.tags,
            };
            questions.push(question);
        }

        Ok(questions)
    }

    pub async fn get_question_by_id<T: IntoQuestionId>(
        &mut self,
        id: T,
    ) -> Result<Question, AppError> {
        let id = id.into_question_id();

        let row = sqlx::query!(
            r#"
    SELECT * FROM questions WHERE id = $1
    "#,
            id.0,
        )
            .fetch_one(&self.conn_pool)
            .await?;

        let question = Question {
            id: row.id.into(), // Assuming you have a From<u32> for QuestionId
            title: row.title,
            content: row.content,
            tags: row.tags,
        };

        Ok(question)
    }

    pub async fn get_question_comment_answer_by_id<T: IntoQuestionId>(
        &mut self,
        id: T,
    ) -> Result<PagePackage, AppError> {
        let id = id.into_question_id();

        let questionRow = sqlx::query!(
            r#"
    SELECT * FROM questions WHERE id = $1
    "#,
            id.0,
        )
            .fetch_one(&self.conn_pool)
            .await?;

        let answerRow = sqlx::query!(
            r#"
    SELECT * FROM answers WHERE question_id = $1
    "#,
            id.0,
        )
            .fetch_one(&self.conn_pool)
            .await?;
/*
        let commentRow = sqlx::query!(
            r#"
    SELECT * FROM comments WHERE id = $1
    "#,
            id.0,
        )
            .fetch_one(&self.conn_pool)
            .await?;
*/
        let question = Question {
            id: questionRow.id.into(), // Assuming you have a From<u32> for QuestionId
            title: questionRow.title,
            content: questionRow.content,
            tags: questionRow.tags,
        };

        let answer = Answer {
            id: answerRow.id.into(),
            content: answerRow.content,
            question_id: questionRow.id.into(),
        };
/*
        let comment = Comment {
            id: row.id.into(),
            content: row.content,
        };
*/
        let page = PagePackage {
            question: question,
            //question_comments: Option<Vec<Comment>>,
            answers: answer,
            //answer_comments: Option<Vec<Comment>>
        };
        Ok(page)
    }

    pub async fn add_question(
        &mut self,
        title: String,
        content: String,
        tags: Option<Vec<String>>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO "questions"(title, content, tags)
           VALUES ($1, $2, $3)
        "#,
            title,
            content,
            tags.as_deref()
        )
            .execute(&self.conn_pool)
            .await?;

        Ok(())
    }

    pub async fn update_question(
        &mut self,
        new_question: UpdateQuestion,
    ) -> Result<Question, AppError> {
        sqlx::query!(
            r#"
    UPDATE questions
    SET title = $1, content = $2, tags = $3
    WHERE id = $4
    "#,
            new_question.title,
            new_question.content,
            new_question.tags.as_deref(),
            new_question.id.0,
        )
            .execute(&self.conn_pool)
            .await?;

        let row = sqlx::query!(
        r#"
SELECT title, content, id, tags FROM questions WHERE id = $1
"#,
        new_question.id.0,
    )
            .fetch_one(&self.conn_pool)
            .await?;

        let question = Question {
            title: row.title,
            content: row.content,
            id: QuestionId(row.id),
            tags: row.tags,
        };

        Ok(question)
    }

    pub async fn delete_question(
        &mut self,
        question_id: i32,
    ) -> Result<(), AppError> {
        let question_id = question_id.into_question_id();
        println!("DELETE - Question id is {}", &question_id);
        sqlx::query!(
            r#"
    DELETE FROM questions WHERE id = $1
    "#,
            question_id.0,
        )
            .execute(&self.conn_pool)
            .await.unwrap();

        Ok(())
    }
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
