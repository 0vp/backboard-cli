use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

impl TodoStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
}

impl TodoPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub status: TodoStatus,
    pub priority: Option<TodoPriority>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
pub struct TodoStore {
    items: Arc<Mutex<Vec<TodoItem>>>,
}

impl TodoStore {
    pub fn create(
        &self,
        title: String,
        status: TodoStatus,
        priority: Option<TodoPriority>,
    ) -> TodoItem {
        let (completed, completed_at) = if status == TodoStatus::Completed {
            (true, Some(Utc::now()))
        } else {
            (false, None)
        };

        let item = TodoItem {
            id: Uuid::new_v4().to_string(),
            title,
            completed,
            status,
            priority,
            created_at: Utc::now(),
            completed_at,
        };
        let mut items = self.items.lock().expect("todo mutex poisoned");
        items.push(item.clone());
        item
    }

    pub fn update(
        &self,
        id: &str,
        title: Option<String>,
        status: Option<TodoStatus>,
        priority: Option<Option<TodoPriority>>,
    ) -> Option<TodoItem> {
        let mut items = self.items.lock().expect("todo mutex poisoned");
        let item = items.iter_mut().find(|v| v.id == id)?;

        if let Some(title) = title {
            item.title = title;
        }

        if let Some(status) = status {
            item.status = status;
            match status {
                TodoStatus::Completed => {
                    item.completed = true;
                    item.completed_at = Some(Utc::now());
                }
                _ => {
                    item.completed = false;
                    item.completed_at = None;
                }
            }
        }

        if let Some(priority) = priority {
            item.priority = priority;
        }

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
        item.status = TodoStatus::Completed;
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
