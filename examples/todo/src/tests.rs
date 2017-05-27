extern crate parking_lot;
extern crate rand;

use super::task::Task;
use self::parking_lot::Mutex;
use self::rand::{Rng, thread_rng};

use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, ContentType};

// We use a lock to synchronize between tests so DB operations don't collide.
// For now. In the future, we'll have a nice way to run each test in a DB
// transaction so we can regain concurrency.
static DB_LOCK: Mutex<()> = Mutex::new(());

macro_rules! run_test {
    (|$rocket:ident, $conn:ident| $block:expr) => ({
        let _lock = DB_LOCK.lock();
        let ($rocket, db) = super::rocket();
        let $conn = db.expect("failed to get database connection for testing");
        $block
    })
}

#[test]
fn test_insertion_deletion() {
    run_test!(|rocket, conn| {
        // Get the tasks before making changes.
        let init_tasks = Task::all(&conn);

        // Issue a request to insert a new task.
        let mut req = MockRequest::new(Post, "/todo")
            .header(ContentType::Form)
            .body("description=My+first+task");
        req.dispatch_with(&rocket);

        // Ensure we have one more task in the database.
        let new_tasks = Task::all(&conn);
        assert_eq!(new_tasks.len(), init_tasks.len() + 1);

        // Ensure the task is what we expect.
        assert_eq!(new_tasks[0].description, "My first task");
        assert_eq!(new_tasks[0].completed, false);

        // Issue a request to delete the task.
        let id = new_tasks[0].id.unwrap();
        let mut req = MockRequest::new(Delete, format!("/todo/{}", id));
        req.dispatch_with(&rocket);

        // Ensure it's gone.
        let final_tasks = Task::all(&conn);
        assert_eq!(final_tasks.len(), init_tasks.len());
        if final_tasks.len() > 0 {
            assert_ne!(final_tasks[0].description, "My first task");
        }
    })
}

#[test]
fn test_toggle() {
    run_test!(|rocket, conn| {
        // Issue a request to insert a new task; ensure it's not yet completed.
        let mut req = MockRequest::new(Post, "/todo")
            .header(ContentType::Form)
            .body("description=test_for_completion");
        req.dispatch_with(&rocket);

        let task = Task::all(&conn)[0].clone();
        assert_eq!(task.completed, false);

        // Issue a request to toggle the task; ensure it is completed.
        let mut req = MockRequest::new(Put, format!("/todo/{}", task.id.unwrap()));
        req.dispatch_with(&rocket);
        assert_eq!(Task::all(&conn)[0].completed, true);

        // Issue a request to toggle the task; ensure it's not completed again.
        let mut req = MockRequest::new(Put, format!("/todo/{}", task.id.unwrap()));
        req.dispatch_with(&rocket);
        assert_eq!(Task::all(&conn)[0].completed, false);
    })
}

#[test]
fn test_many_insertions() {
    const ITER: usize = 100;

    let mut rng = thread_rng();
    run_test!(|rocket, conn| {
        // Get the number of tasks initially.
        let init_num = Task::all(&conn).len();
        let mut descs = Vec::new();

        for i in 0..ITER {
            // Issue a request to insert a new task with a random description.
            let desc: String = rng.gen_ascii_chars().take(12).collect();
            let mut req = MockRequest::new(Post, "/todo")
                .header(ContentType::Form)
                .body(format!("description={}", desc));
            req.dispatch_with(&rocket);

            // Record the description we choose for this iteration.
            descs.insert(0, desc);

            // Ensure the task was inserted properly and all other tasks remain.
            let tasks = Task::all(&conn);
            assert_eq!(tasks.len(), init_num + i + 1);

            for j in 0..i {
                assert_eq!(descs[j], tasks[j].description);
            }
        }
    })
}

#[test]
fn test_bad_form_submissions() {
    run_test!(|rocket, _conn| {
        // Submit an empty form. We should get a 422 but no flash error.
        let mut req = MockRequest::new(Post, "/todo").header(ContentType::Form);
        let res = req.dispatch_with(&rocket);
        assert_eq!(res.status(), Status::UnprocessableEntity);
        let mut cookies = res.headers().get("Set-Cookie");
        assert!(!cookies.any(|value| value.contains("error")));

        // Submit a form with an empty description.
        let mut req = MockRequest::new(Post, "/todo")
            .header(ContentType::Form)
            .body("description=");

        // We look for 'error' in the cookies which corresponds to flash message
        // being set as an error.
        let res = req.dispatch_with(&rocket);
        let mut cookies = res.headers().get("Set-Cookie");
        assert!(cookies.any(|value| value.contains("error")));

        // Submit a form without a description. Expect a 422 but no flash error.
        let mut req = MockRequest::new(Post, "/todo")
            .header(ContentType::Form)
            .body("evil=smile");
        let res = req.dispatch_with(&rocket);
        assert_eq!(res.status(), Status::UnprocessableEntity);
        let mut cookies = res.headers().get("Set-Cookie");
        assert!(!cookies.any(|value| value.contains("error")));
    })
}
