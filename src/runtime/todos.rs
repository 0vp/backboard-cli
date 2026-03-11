use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
pub struct TodoStore {
    items: Arc<Mutex<Vec<TodoItem>>>,
}

impl TodoStore {
    pub fn create(&self, title: String) -> TodoItem {
        let item = TodoItem {
            id: Uuid::new_v4().to_string(),
            title,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        };
        let mut items = self.items.lock().expect("todo mutex poisoned");
        items.push(item.clone());
        item
    }

    pub fn update(&self, id: &str, title: String) -> Option<TodoItem> {
        let mut items = self.items.lock().expect("todo mutex poisoned");
        let item = items.iter_mut().find(|v| v.id == id)?;
        item.title = title;
        Some(item.clone())
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut items = self.items.lock().expect("todo mutex poisoned");
        let before = items.len();
        items.retain(|v| v.id != id);
        before != items.len()
    }

    pub fn complete(&self, id: &str) -> Option<TodoItem> {
        let mut items = self.items.lock().expect("todo mutex poisoned");
        let item = items.iter_mut().find(|v| v.id == id)?;
        item.completed = true;
        item.completed_at = Some(Utc::now());
        Some(item.clone())
    }

    pub fn list(&self) -> Vec<TodoItem> {
        self.items.lock().expect("todo mutex poisoned").clone()
    }

    pub fn clear(&self) {
        self.items.lock().expect("todo mutex poisoned").clear();
    }
}
