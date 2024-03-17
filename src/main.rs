#[macro_use] extern crate rocket;

use migration::MigratorTrait;
use todo_app_for_article::pool::Db;
use todo_app_for_article::{render_routes, task_routes, user_routes};
use rocket::{fairing::{AdHoc, self}, Rocket, Build, fs::{FileServer, relative}};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;

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
        .mount("/", routes![
            render_routes::index,
            render_routes::index_redirect,
            render_routes::edit_task_page, 
            render_routes::edit_task_page_redirect,
            render_routes::signup_page,
            render_routes::login_page,
            task_routes::add_task,
            task_routes::add_task_redirect, 
            task_routes::edit_task, 
            task_routes::edit_task_redirect,
            task_routes::delete_task, 
            task_routes::delete_task_redirect,
            user_routes::create_account, 
            user_routes::verify_account,
            user_routes::logout,
        ])
        .attach(Template::fairing())
}