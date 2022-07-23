use argon2::Config;
use entity::users::{self, USER_PASSWORD_SALT};
use rocket::{response::{Flash, Redirect}, http::{CookieJar, Cookie}, request::{FromRequest, Outcome}, Request};
use rocket::form::Form;
use sea_orm::{Set, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
use sea_orm_rocket::Connection;

use entity::users::Entity as Users;
use crate::pool::Db;

pub struct AuthenticatedUser {
    pub user_id: i32
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = anyhow::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let user_id_cookie = match get_user_id_cookie(cookies) {
            Some(result) => result,
            None => return Outcome::Forward(())
        };

        let logged_in_user_id = match user_id_cookie.value()
            .parse::<i32>() {
                Ok(result) => result,
                Err(_err) => return Outcome::Forward(())
            };

        return Outcome::Success(AuthenticatedUser { user_id: logged_in_user_id });
    }
}

fn get_user_id_cookie<'a>(cookies: &'a CookieJar) -> Option<Cookie<'a>> {
    cookies.get_private("user_id")
}

fn set_user_id_cookie(cookies: & CookieJar, user_id: i32) {
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
}

fn remove_user_id_cookie(cookies: & CookieJar) {
    cookies.remove_private(Cookie::named("user_id"));
}

#[post("/logout")]
pub async fn logout(cookies: & CookieJar<'_>) -> Flash<Redirect> {
    remove_user_id_cookie(cookies);
    Flash::success(Redirect::to("/login"), "Logged out succesfully!")
}

fn login_error() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Incorrect username or password")
}

#[post("/createaccount", data="<user_form>")]
pub async fn create_account(conn: Connection<'_, Db>, user_form: Form<users::Model>) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    if user.username.is_empty() || user.password.is_empty() {
        return Flash::error(Redirect::to("/signup"), "Please enter a valid username and password");
    }

    let hash_config = Config::default();
    let hash = match argon2::hash_encoded(user.password.as_bytes(), USER_PASSWORD_SALT, &hash_config) {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/signup"), "Issue creating account");
        }
    };

    let active_user = users::ActiveModel {
        username: Set(user.username),
        password: Set(hash),
        ..Default::default()
    };

    match active_user.insert(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/signup"), "Issue creating account");
        }
    };

    Flash::success(Redirect::to("/login"), "Account created succesfully!")
}

#[post("/verifyaccount", data="<user_form>")]
pub async fn verify_account(conn: Connection<'_, Db>, cookies: & CookieJar<'_>, user_form: Form<users::Model>) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    let stored_user = match Users::find()
        .filter(users::Column::Username.contains(&user.username))
        .one(db)
        .await {
            Ok(model_or_null) => {
                match model_or_null {
                    Some(model) => model,
                    None => {
                        return login_error();
                    }
                }
            },
            Err(_) => {
                return login_error();
            }
        };
    
    let is_password_correct = match argon2::verify_encoded(&stored_user.password, user.password.as_bytes()) {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/login"), "Encountered an issue processing your account")
        }
    };

    if !is_password_correct {
        return login_error();
    }

    set_user_id_cookie(cookies, stored_user.id);
    Flash::success(Redirect::to("/"), "Logged in succesfully!")
}