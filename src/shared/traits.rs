use uuid::Uuid;

pub trait Translatable {
    fn filter_by_language(&mut self, language_id: Uuid);
}
