pub enum Event {
    Navigation {
        target_type: TargetResourceType,
        id: Option<u64>,
    },
    NewDataReady { data_type: DataType },
}

pub enum TargetResourceType {
    Main,
    Class,
    ClassMaterial,
}

pub enum DataType {
    ClassListing,
    SingleClassListingTeachers { id: u64 },
    ClassData { id: u64 },
    MaterialData { id: u64 },
}