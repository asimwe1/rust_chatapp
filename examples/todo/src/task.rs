use diesel;
use serde::Serialize;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use self::schema::tasks;
use self::schema::tasks::dsl::{tasks as all_tasks, completed as task_completed};

mod schema {
    infer_schema!("db/db.sql");
}

fn db() -> SqliteConnection {
    SqliteConnection::establish("db/db.sql").expect("Failed to connect to db.")
}

#[insertable_into(tasks)]
#[derive(Serialize, Queryable, FromForm, Debug, Clone)]
pub struct Task {
    id: Option<i32>,
    pub description: String,
    pub completed: Option<bool>
}

impl Task {
    pub fn all() -> Vec<Task> {
        all_tasks.order(tasks::id.desc()).load::<Task>(&db()).unwrap()
    }

    pub fn insert(&self) -> bool {
        diesel::insert(self).into(tasks::table).execute(&db()).is_ok()
    }

    pub fn toggle_with_id(id: i32) -> bool {
        let task = all_tasks.find(id).get_result::<Task>(&db());
        if task.is_err() {
            return false;
        }

        Task::update_with_id(id, !task.unwrap().completed.unwrap())
    }

    pub fn update_with_id(id: i32, completed: bool) -> bool {
        let task = diesel::update(all_tasks.find(id));
        task.set(task_completed.eq(completed)).execute(&db()).is_ok()
    }

    pub fn delete_with_id(id: i32) -> bool {
        diesel::delete(all_tasks.find(id)).execute(&db()).is_ok()
    }
}

