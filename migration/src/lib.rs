pub use sea_orm_migration::prelude::*;

mod m20220704_145355_create_tasks_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220704_145355_create_tasks_table::Migration),
        ]
    }
}
