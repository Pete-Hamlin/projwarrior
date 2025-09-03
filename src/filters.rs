use uuid::Uuid;

use crate::project::State;

#[derive(Debug, Default)]
pub struct ProjectFilter<'a> {
    pub state: Option<&'a State>,
    pub uuid: Option<&'a Uuid>,
    pub id: Option<&'a u32>,
    pub name: Option<&'a String>,
}

impl<'a> ProjectFilter<'a> {
    pub fn builder() -> FilterBuilder<'a> {
        FilterBuilder::new()
    }
}

pub struct FilterBuilder<'a> {
    filter: ProjectFilter<'a>,
}

impl<'a> FilterBuilder<'a> {
    pub fn new() -> Self {
        FilterBuilder {
            filter: ProjectFilter::default(),
        }
    }

    pub fn state(mut self, state: &'a State) -> Self {
        self.filter.state = Some(state);
        self
    }

    pub fn uuid(mut self, uuid: &'a Uuid) -> Self {
        self.filter.uuid = Some(uuid);
        self
    }

    pub fn id(mut self, id: &'a u32) -> Self {
        self.filter.id = Some(id);
        self
    }

    pub fn name(mut self, name: &'a String) -> Self {
        self.filter.name = Some(name);
        self
    }

    pub fn build(self) -> ProjectFilter<'a> {
        self.filter
    }
}
