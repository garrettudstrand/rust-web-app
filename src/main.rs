#[macro_use] extern crate rocket;

mod pool;


use migration::{MigratorTrait, tests_cfg::json};
use pool::Db;
use rocket::{fairing::{AdHoc, self}, Rocket, Build, form::Form, serde::json::{Json}, http::Status, response::{Responder, self, Flash, Redirect}, Request, request::FlashMessage, fs::{FileServer, relative}};
use rocket_dyn_templates::Template;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, QueryOrder, DeleteResult, PaginatorTrait};
use sea_orm_rocket::{Database, Connection};

use entity::tasks;
use entity::tasks::Entity as Tasks;

struct DatabaseError(sea_orm::DbErr);

impl<'r> Responder<'r, 'r> for DatabaseError {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {
        Err(Status::InternalServerError)
    }
}

impl From<sea_orm::DbErr> for DatabaseError {
    fn from(error: sea_orm::DbErr) -> Self {
        DatabaseError(error)
    }
}

#[post("/addtask", data="<task_form>")]
async fn add_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>) -> Flash<Redirect> {
    let db = conn.into_inner();
    let task = task_form.into_inner();

    let active_task: tasks::ActiveModel = tasks::ActiveModel {
        item: Set(task.item),
        ..Default::default()
    };

    match active_task.insert(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue creating the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task created!")
}

#[put("/edittask", data="<task_form>")]
async fn edit_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>) -> Flash<Redirect>{
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

#[delete("/deletetask/<id>")]
async fn delete_task(conn: Connection<'_, Db>, id: i32) -> Flash<Redirect> {
    let db = conn.into_inner();
    let _result = match Tasks::delete_by_id(id).exec(db).await {
        Ok(value) => value,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue deleting the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task succesfully deleted!")
}

#[get("/?<page>&<tasks_per_page>")]
async fn index(conn: Connection<'_, Db>, flash: Option<FlashMessage<'_>>, page: Option<usize>, tasks_per_page: Option<usize>) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let page = page.unwrap_or(0);
    let tasks_per_page = tasks_per_page.unwrap_or(5);

    let paginator = Tasks::find()
                            .order_by_asc(tasks::Column::Id)
                            .paginate(db, tasks_per_page);
    let number_of_pages = paginator.num_pages().await?;
    let tasks = paginator.fetch_page(page).await?;


    Ok(Template::render(
        "todo_list",
        json!({
            "tasks": tasks,
            "flash": flash.map(FlashMessage::into_inner),
            "number_of_pages": number_of_pages,
            "current_page": page
        })
    ))
}

#[get("/edit/<id>")]
async fn edit_task_page(conn: Connection<'_, Db>, id: i32) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let task = Tasks::find_by_id(id).one(db).await?.unwrap();

    Ok(Template::render(
        "edit_task_form", 
        json!({
            "task": task
        })
    ))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("/public")))
        .mount("/", routes![index, add_task, edit_task, delete_task, edit_task_page])
        .attach(Template::fairing())
}