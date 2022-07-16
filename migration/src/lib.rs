pub use sea_orm_migration::prelude::*;

mod m20220704_145355_create_tasks_table;
mod m20220716_113001_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220716_113001_create_users_table::Migration),
            Box::new(m20220704_145355_create_tasks_table::Migration),
        ]
    }
}
