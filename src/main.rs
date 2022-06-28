#[macro_use] extern crate rocket;

use core::fmt;
use std::{fs::{OpenOptions, File}, io::{Write, BufReader, BufRead}, num::ParseIntError};
use rocket::{serde::{Deserialize, json::Json, Serialize}, response::{Responder, self}, http::{Status}, Request};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Task {
    id: u8,
    item: String
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskItem<'r> {
    item: &'r str
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskId {
    id: u8
}

#[derive(Debug)]
enum FileParseError {
    IoError(std::io::Error),
    ParseError(ParseIntError)
}

impl fmt::Display for FileParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "{}", err),
            Self::ParseError(err) => write!(f, "{}", err)
        }
    }
}

impl From<std::io::Error> for FileParseError {
    fn from(error: std::io::Error) -> Self {
        FileParseError::IoError(error)
    }
}

impl From<ParseIntError> for FileParseError {
    fn from(error: ParseIntError) -> Self {
        FileParseError::ParseError(error)
    }
}

impl<'r> Responder<'r, 'r> for FileParseError {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        match self {
            Self::IoError(err) => {
                err.respond_to(request)
            },
            Self::ParseError(_err) => {
                Err(Status::InternalServerError)
            }
        }
    }
}

fn open_tasks_file() -> Result<File, std::io::Error> {
    OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open("tasks.txt")
}

fn open_temp_file() -> Result<File, std::io::Error> {
    OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open("temp.txt")
}

fn save_temp_as_tasks() -> Result<(), std::io::Error> {
    std::fs::remove_file("tasks.txt")?;
    std::fs::rename("temp.txt", "tasks.txt")?;
    Ok(())
}

#[post("/addtask", data="<task>")]
fn add_task(task: Json<TaskItem<'_>>) -> Result<Json<Task>, FileParseError> {
    let mut tasks =  open_tasks_file()?;

    let reader = BufReader::new(&tasks);
    let id = match reader.lines().last() {
        None => 0,
        Some(last_line) => {
            let last_line_parsed = last_line?;
            let last_line_pieces: Vec<&str> = last_line_parsed.split(",").collect();
            last_line_pieces[0].parse::<u8>()? + 1
        }
    };

    let task_item_string = format!("{},{}\n", id, task.item);
    let task_item_bytes = task_item_string.as_bytes();

    tasks.write(task_item_bytes)?;
    Ok(Json(Task {
        id: id,
        item: task.item.to_string()
    }))
}

#[get("/readtasks")]
fn read_tasks() -> Result<Json<Vec<Task>>, FileParseError> {
    let tasks = open_tasks_file()?;
    let reader = BufReader::new(tasks);

    let parsed_lines: Result<Vec<Task>, FileParseError> = reader.lines()
                        .map(|line| {
                            let line_string: String = line?;
                            let line_pieces: Vec<&str> = line_string.split(",").collect();
                            let line_id: u8 = line_pieces[0].parse::<u8>()?;

                            Ok(Task {
                                id: line_id,
                                item: line_pieces[1].to_string()
                            })
                        })
                        .collect();

    Ok(Json(parsed_lines?))
}

#[put("/edittask", data="<task_update>")]
fn edit_task(task_update: Json<Task>) -> Result<Json<Task>, FileParseError> {
    let tasks = open_tasks_file()?; 
    let mut temp = open_temp_file()?;

    let reader = BufReader::new(tasks);
    for line in reader.lines() {
        let line_string: String = line?;
        let line_pieces: Vec<&str> = line_string.split(",").collect();

        if line_pieces[0].parse::<u8>()? == task_update.id {
            let task_items: [&str; 2] = [line_pieces[0], &task_update.item];
            let task = format!("{}\n", task_items.join(","));
            temp.write(task.as_bytes())?;
        }
        else {
            let task = format!("{}\n", line_string);
            temp.write(task.as_bytes())?;
        }
    }

    save_temp_as_tasks()?;
    Ok(task_update)
}

#[delete("/deletetask", data="<task_id>")]
fn delete_task(task_id: Json<TaskId>) -> Result<Json<TaskId>, FileParseError> {
    let tasks = open_tasks_file()?; 
    let mut temp = open_temp_file()?;

    let reader = BufReader::new(tasks);

    for line in reader.lines() {
        let line_string: String = line?;
        let line_pieces: Vec<&str> = line_string.split(",").collect();

        if line_pieces[0].parse::<u8>()? != task_id.id {
            let task = format!("{}\n", line_string);
            temp.write(task.as_bytes())?;
        }
    }

    save_temp_as_tasks()?;
    Ok(task_id)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add_task, read_tasks, edit_task, delete_task])
}