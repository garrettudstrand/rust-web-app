use migration::tests_cfg::json;
use rocket::{request::FlashMessage, response::{Redirect, self, Responder}, Request, http::Status};
use rocket_dyn_templates::Template;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder, PaginatorTrait};
use sea_orm_rocket::Connection;

use entity::tasks::{Entity as Tasks, self};

use crate::{pool::Db, user_routes::AuthenticatedUser, task_routes};

pub struct DatabaseError(sea_orm::DbErr);

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

#[get("/?<page>&<tasks_per_page>")]
pub async fn index(conn: Connection<'_, Db>, flash: Option<FlashMessage<'_>>, page: Option<usize>, tasks_per_page: Option<usize>, user: AuthenticatedUser) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let page = page.unwrap_or(0);
    let tasks_per_page = tasks_per_page.unwrap_or(5);

    let paginator = Tasks::find()
                            .filter(tasks::Column::UserId.eq(user.user_id))
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

#[get("/?<page>&<tasks_per_page>", rank = 2)]
pub async fn index_redirect(page: Option<usize>, tasks_per_page: Option<usize>) -> Redirect {
    task_routes::redirect_to_login()
}

#[get("/edit/<id>")]
pub async fn edit_task_page(conn: Connection<'_, Db>, id: i32, _user: AuthenticatedUser) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let task = Tasks::find_by_id(id).one(db).await?.unwrap();

    Ok(Template::render(
        "edit_task_form", 
        json!({
            "task": task
        })
    ))
}

#[get("/edit/<id>", rank = 2)]
pub async fn edit_task_page_redirect(id: i32) -> Redirect {
    task_routes::redirect_to_login()
}

#[get("/signup")]
pub async fn signup_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "signup_page", 
        json!({
            "flash": flash.map(FlashMessage::into_inner)
        })
    )
}

#[get("/login")]
pub async fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "login_page", 
        json!({
            "flash": flash.map(FlashMessage::into_inner)
        })
    )
}