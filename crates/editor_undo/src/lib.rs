//! Shared undo/redo primitives for editor mutation paths.

#[derive(Debug, Clone)]
pub struct UndoSnapshot<T> {
    label: String,
    value: T,
}

impl<T> UndoSnapshot<T> {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct UndoResult<T> {
    pub label: String,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct UndoStack<T> {
    undo: Vec<UndoSnapshot<T>>,
    redo: Vec<UndoSnapshot<T>>,
    limit: usize,
}

impl<T: Clone> UndoStack<T> {
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn push(&mut self, label: impl Into<String>, current: &T) {
        self.push_value(label, current.clone());
    }

    pub fn push_value(&mut self, label: impl Into<String>, value: T) {
        self.undo.push(UndoSnapshot {
            label: label.into(),
            value,
        });
        if self.undo.len() > self.limit {
            let drop_count = self.undo.len() - self.limit;
            self.undo.drain(0..drop_count);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, redo_label: impl Into<String>, current: &T) -> Option<UndoResult<T>> {
        let snapshot = self.undo.pop()?;
        self.redo.push(UndoSnapshot {
            label: redo_label.into(),
            value: current.clone(),
        });
        Some(UndoResult {
            label: snapshot.label,
            value: snapshot.value,
        })
    }

    pub fn redo(&mut self, undo_label: impl Into<String>, current: &T) -> Option<UndoResult<T>> {
        let snapshot = self.redo.pop()?;
        self.undo.push(UndoSnapshot {
            label: undo_label.into(),
            value: current.clone(),
        });
        Some(UndoResult {
            label: snapshot.label,
            value: snapshot.value,
        })
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

impl<T: Clone> Default for UndoStack<T> {
    fn default() -> Self {
        Self::new(80)
    }
}

pub fn init() -> anyhow::Result<()> {
    log::info!("initialized: editor_undo");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::UndoStack;

    #[test]
    fn restores_prior_state_and_redoes_current_state() {
        let mut history = UndoStack::new(8);
        let mut value = vec![1, 2, 3];

        history.push("add four", &value);
        value.push(4);

        let undo = history.undo("redo add four", &value).expect("undo");
        assert_eq!(undo.label, "add four");
        value = undo.value;
        assert_eq!(value, vec![1, 2, 3]);

        let redo = history.redo("undo add four", &value).expect("redo");
        assert_eq!(redo.label, "redo add four");
        value = redo.value;
        assert_eq!(value, vec![1, 2, 3, 4]);
    }

    #[test]
    fn push_clears_redo_stack() {
        let mut history = UndoStack::new(8);
        let mut value = 1;

        history.push("set two", &value);
        value = 2;
        value = history.undo("redo set two", &value).unwrap().value;
        assert_eq!(value, 1);
        assert!(history.can_redo());

        history.push("set three", &value);
        assert!(!history.can_redo());
    }

    #[test]
    fn honors_history_limit() {
        let mut history = UndoStack::new(2);
        history.push("one", &1);
        history.push("two", &2);
        history.push("three", &3);

        assert_eq!(history.undo_len(), 2);
        assert_eq!(history.undo("redo", &4).unwrap().label, "three");
        assert_eq!(history.undo("redo", &3).unwrap().label, "two");
        assert!(history.undo("redo", &2).is_none());
    }
}
