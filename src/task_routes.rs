use entity::tasks;
use rocket::response::{Redirect, Flash};
use sea_orm::{EntityTrait, Set, ActiveModelTrait};
use sea_orm_rocket::Connection;
use rocket::form::Form;

use entity::tasks::Entity as Tasks;

use crate::{pool::Db, user_routes::AuthenticatedUser};

pub fn redirect_to_login() -> Redirect {
    Redirect::to("/login")
}

#[post("/addtask", data="<task_form>")]
pub async fn add_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>, user: AuthenticatedUser) -> Flash<Redirect> {
    let db = conn.into_inner();
    let task = task_form.into_inner();

    println!("User ID: {}", user.user_id);

    let active_task: tasks::ActiveModel = tasks::ActiveModel {
        item: Set(task.item),
        user_id: Set(user.user_id),
        ..Default::default()
    };

    match active_task.insert(db).await {
        Ok(result) => result,
        Err(err) => return Flash::error(Redirect::to("/"), "Issue creating the task")
    };

    Flash::success(Redirect::to("/"), "Task created!")
}

#[post("/addtask", rank = 2)]
pub async fn add_task_redirect() -> Redirect {
    redirect_to_login()
}

#[put("/edittask", data="<task_form>")]
pub async fn edit_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>, _user: AuthenticatedUser) -> Flash<Redirect>{
    let db = conn.into_inner();
    let task = task_form.into_inner();

    let task_to_update = match Tasks::find_by_id(task.id).one(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue editing the task");
        }
    };
    let mut task_to_update: tasks::ActiveModel = task_to_update.unwrap().into();
    task_to_update.item = Set(task.item);
    match task_to_update.update(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue editing the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task edited succesfully!")
}

#[put("/edittask", rank = 2)]
pub async fn edit_task_redirect() -> Redirect {
    redirect_to_login()
}

#[delete("/deletetask/<id>")]
pub async fn delete_task(conn: Connection<'_, Db>, id: i32, _user: AuthenticatedUser) -> Flash<Redirect> {
    let db = conn.into_inner();
    let _result = match Tasks::delete_by_id(id).exec(db).await {
        Ok(value) => value,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue deleting the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task succesfully deleted!")
}

#[delete("/deletetask/<id>", rank = 2)]
pub async fn delete_task_redirect(id: i32) -> Redirect {
    redirect_to_login()
}