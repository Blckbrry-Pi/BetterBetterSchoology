pub enum Event {
    Navigation {
        target_type: TargetResourceType,
        id: Option<u64>,
    },
    
}

pub enum TargetResourceType {
    Main,
    Class,
    ClassItem,
}